// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use std::{ future::Future, panic::{ catch_unwind, AssertUnwindSafe }, pin::Pin, sync::{ atomic::{ AtomicBool, Ordering }, Arc, Mutex, PoisonError }, task::{ Context, Poll }};
use futures_util::task::AtomicWaker;
use core::ffi::c_void;
use super::*;

/// Shared state co-owned by a [`CallbackFuture`] and the BMD SDK.
///
/// One `Arc<State<T>>` clone lives in the future; a second, *owned* refcount is
/// handed to the SDK (`Arc::into_raw` in [`CallbackFuture::create_from_job`] /
/// `BlackmagicRaw::prepare_pipeline`) and reclaimed by the completion callback
/// through [`deliver_completion`]. Because the SDK holds its own refcount, the
/// state outlives a future that is dropped before its callback fires — the late
/// callback then operates on live memory instead of a dangling pointer.
pub(crate) struct State<T> {
    pub(crate) waker: AtomicWaker,
    /// Set to `true` (with `Release`) by the winning completion callback *after*
    /// `result` is stored; read by `poll` (with `Acquire`) to know the result is
    /// ready to take.
    pub(crate) done: AtomicBool,
    /// Claim-once flag. The single completion callback that transitions this
    /// `false -> true` (with `AcqRel`) is the *winner* and is solely responsible
    /// for storing the result and reclaiming the SDK's owned refcount; any second
    /// or spurious callback observes `true` and becomes a safe no-op.
    pub(crate) claimed: AtomicBool,
    pub(crate) result: Mutex<Option<Result<T, BrawError>>>,
}
impl<T> State<T> {
    pub(crate) fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            done: AtomicBool::new(false),
            claimed: AtomicBool::new(false),
            result: Mutex::new(None),
        }
    }
}

/// A `'static` future awaiting one asynchronous BMD job (read / decode / process
/// / prepare-pipeline) completion callback.
///
/// # Ownership & lifetime
/// The SDK co-owns the callback [`State`] via an owned refcount handed out at
/// construction (see [`State`] and [`CallbackFuture::create_from_job`]). This is
/// what makes the future `'static` and safe to drop early.
///
/// # Cancellation semantics
/// This type has **no** `Drop` impl that aborts the job. Dropping a *pending*
/// future therefore does **not** cancel the underlying job: the job runs to
/// completion, its buffers are reclaimed by the (still-armed) completion
/// callback, and the produced value is discarded. To cancel, call
/// [`abort()`](CallbackFuture::abort) (best-effort) and then still drive the
/// future to completion by awaiting it — never rely on drop for cancellation.
pub struct CallbackFuture<T> {
    pub(crate) state: Arc<State<T>>,
    pub(crate) job: Option<ComPtr<IBlackmagicRawJob>>,
}
// SAFETY: `CallbackFuture<T>` is not auto-`Send` only because of the raw
// `ComPtr<IBlackmagicRawJob>`. Sending it moves ownership of the job handle to
// another thread; BMD's decode model is documented as free-threaded, so a job
// handle may be owned/submitted/aborted from any single thread at a time. `T`
// crosses the boundary inside the state, hence the `T: Send` bound.
unsafe impl<T: Send> Send for CallbackFuture<T> {}
// SAFETY: `&CallbackFuture<T>` is shareable across threads. The only `&self`
// method is `abort()`, which calls `IBlackmagicRawJob::Abort` — the SDK
// serialises this internally against its own completion delivery (COM objects
// are free-threaded). `State<T>` is `Sync` when `T: Send` (atomics + `Mutex`),
// so concurrent readers of the shared state are sound.
unsafe impl<T: Send> Sync for CallbackFuture<T> {}
impl<T> Future for CallbackFuture<T> {
    type Output = Result<T, BrawError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.state.waker.register(cx.waker());
        if this.state.done.load(Ordering::Acquire) {
            // A poisoned result mutex is not fatal: the callback only ever
            // *stores* into it while holding the lock, so the inner data is
            // consistent — recover it instead of unwinding.
            if let Some(res) = this.state.result.lock().unwrap_or_else(PoisonError::into_inner).take() {
                return Poll::Ready(res);
            }
        }
        Poll::Pending
    }
}

impl<T> CallbackFuture<T> {
    /// Request the SDK abort this in-flight job. **Best-effort**: per the BMD
    /// SDK manual, `Abort` "CAN fail if the job has already been started by
    /// the internal decoder", and the job still delivers exactly one
    /// completion callback. So after `abort()` you must still drive the
    /// future to completion (await it) to reclaim its buffers — do not just
    /// drop it. No-op for futures with no job (e.g. `prepare_pipeline`).
    pub fn abort(&self) {
        if let Some(job) = &self.job {
            let _ = job.Abort();
        }
    }

    pub fn create_from_job(job: ComPtr<IBlackmagicRawJob>, hints: &[ReadJobHints]) -> Result<Self, BrawError> {
        let state = Arc::new(State::new());

        // Hand the SDK an *owned* refcount (reclaimed in `deliver_completion`
        // via `Arc::from_raw`). This keeps `State` alive even if the future is
        // dropped before the completion callback fires; otherwise the late
        // callback would dereference freed memory.
        let raw = Arc::into_raw(state.clone()) as *mut c_void;

        // If any step below fails the job is never (successfully) submitted,
        // so no callback will fire — reclaim the refcount to avoid leaking.
        let setup = (|| -> Result<(), BrawError> {
            job.SetUserData(raw)?;

            if !hints.is_empty() {
                let mut ptr = std::ptr::null_mut();
                let hr = unsafe { ((*job.vtbl).parent.QueryInterface)(job.as_raw() as _, IBlackmagicRawReadJobHints::iid(), &mut ptr) };
                check_hr(hr)?;
                let hints_com = ComPtr::new(ptr as *mut IBlackmagicRawReadJobHints)?;
                for hint in hints {
                    match hint {
                        ReadJobHints::None => {},
                        ReadJobHints::Scale(scale) => {
                            hints_com.SetReaderResolutionScale(*scale)?;
                        }
                    }
                }
            }

            job.Submit()?;
            Ok(())
        })();

        if let Err(e) = setup {
            unsafe { drop(Arc::from_raw(raw as *const State<T>)); }
            return Err(e);
        }

        Ok(Self { state, job: Some(job) })
    }
}

/// `'static` future for a BRAW **read-frame** job.
///
/// The pipeline-friendly form of [`BlackmagicRawClip::read_frame`]: the
/// job is submitted at construction (`create_read_frame_future`) and this
/// future borrows nothing, so a scheduler can keep `depth` of them in
/// flight in a pool. On completion it wraps the SDK's
/// `IBlackmagicRawFrame` in the public [`BlackmagicRawFrame`] using the
/// captured `factory` + parent COM keep-alives. `Unpin`.
///
/// Cancellation is `abort()` + await — never drop a pending future (see
/// [`CallbackFuture`] for the ownership / cancellation contract).
pub struct ReadFrameFuture {
    inner:         CallbackFuture<ComPtr<IBlackmagicRawFrame>>,
    factory:       Factory,
    parent_guards: DropOrderVec<ComPtrRefGuard>,
}
// SAFETY: `Send` only — moving the future (and thus the frame handle + its
// keep-alives) to another thread is sound under BMD's free-threaded model.
// It is intentionally **not** `Sync`: `parent_guards` holds `ComPtrRefGuard`s,
// which are `Send` but not `Sync`, so `&ReadFrameFuture` must not be shared.
unsafe impl Send for ReadFrameFuture {}
impl ReadFrameFuture {
    pub(crate) fn new(
        inner:         CallbackFuture<ComPtr<IBlackmagicRawFrame>>,
        factory:       Factory,
        parent_guards: DropOrderVec<ComPtrRefGuard>,
    ) -> Self {
        Self { inner, factory, parent_guards }
    }
    /// Best-effort abort of the in-flight read (see [`CallbackFuture::abort`]).
    /// Drive the future to completion afterwards — do not drop it.
    pub fn abort(&self) { self.inner.abort(); }
}
impl Future for ReadFrameFuture {
    type Output = Result<BlackmagicRawFrame, BrawError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match Pin::new(&mut this.inner).poll(cx) {
            Poll::Ready(Ok(raw)) => Poll::Ready(Ok(BlackmagicRawFrame {
                raw,
                factory:       this.factory.clone(),
                parent_guards: this.parent_guards.clone(),
            })),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending       => Poll::Pending,
        }
    }
}

/// `'static` future for a BRAW **decode-and-process** job — the
/// pipeline-friendly form of [`BlackmagicRawFrame::decode_and_process`].
/// See [`ReadFrameFuture`] for the ownership / cancellation contract.
pub struct DecodeProcessFuture {
    inner:         CallbackFuture<ComPtr<IBlackmagicRawProcessedImage>>,
    factory:       Factory,
    parent_guards: DropOrderVec<ComPtrRefGuard>,
}
// SAFETY: `Send` only, for the same reasons as [`ReadFrameFuture`]: sending the
// processed-image handle + its keep-alives across threads is sound, but the
// `ComPtrRefGuard`s in `parent_guards` are not `Sync`, so this is not `Sync`.
unsafe impl Send for DecodeProcessFuture {}
impl DecodeProcessFuture {
    pub(crate) fn new(
        inner:         CallbackFuture<ComPtr<IBlackmagicRawProcessedImage>>,
        factory:       Factory,
        parent_guards: DropOrderVec<ComPtrRefGuard>,
    ) -> Self {
        Self { inner, factory, parent_guards }
    }
    /// Best-effort abort of the in-flight decode (see [`CallbackFuture::abort`]).
    /// Drive the future to completion afterwards — do not drop it.
    pub fn abort(&self) { self.inner.abort(); }
}
impl Future for DecodeProcessFuture {
    type Output = Result<BlackmagicRawProcessedImage, BrawError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match Pin::new(&mut this.inner).poll(cx) {
            Poll::Ready(Ok(raw)) => Poll::Ready(Ok(BlackmagicRawProcessedImage {
                raw,
                factory:       this.factory.clone(),
                parent_guards: this.parent_guards.clone(),
            })),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending       => Poll::Pending,
        }
    }
}

/// Resolve a job's callback `State` from its user-data pointer and hand off to
/// [`deliver_completion`].
///
/// `make_result` is a *closure* (not an already-built value) so result
/// construction — which may allocate, `AddRef`, or otherwise panic — runs
/// inside the panic firewall of [`deliver_completion`], guaranteeing the
/// claim-once and refcount reclaim still happen and nothing unwinds across the
/// C++ ABI boundary (R15).
pub(crate) fn callback_complete<T>(job: *mut IBlackmagicRawJob, make_result: impl FnOnce() -> Result<T, BrawError>) {
    if job.is_null() {
        log::error!("BRAW completion callback delivered a null job pointer; ignoring");
        return;
    }
    // SAFETY: `job` is a valid `IBlackmagicRawJob` for the duration of the
    // callback; `GetUserData` only writes the stored pointer through `&mut ud`.
    let mut ud: *mut c_void = std::ptr::null_mut();
    let _ = unsafe { ((*(*job).vtbl).GetUserData)(job as *mut _, &mut ud) };
    deliver_completion::<T>(ud, make_result);
}

/// Deliver a completion outcome to the [`State`] addressed by the raw user-data
/// pointer the SDK hands back, then release the owned refcount that
/// `create_from_job` / `prepare_pipeline` lent the SDK.
///
/// `ud` must be null or a pointer previously produced by
/// `Arc::<State<T>>::into_raw` and not yet reclaimed.
///
/// # Claim-once (race-safe, even under a spurious/concurrent double callback)
/// The BMD contract is *exactly one* completion callback per submitted job, so
/// in practice this runs once. It is nonetheless hardened so a duplicated or
/// spurious callback is a safe no-op:
///
/// 1. **Own a temporary reference first.** Every invocation does
///    `increment_strong_count` + `from_raw` to hold its *own* strong ref for the
///    duration. This keeps `State` alive under a concurrent invocation even if
///    the future was already dropped and the other invocation frees the SDK's
///    permanent reference — no use-after-free on the claim flag.
/// 2. **One atomic claim.** `claimed.swap(true, AcqRel)` is the single
///    linearization point: exactly one invocation reads `false` (the *winner*).
///    Losers return immediately, dropping only their temporary ref.
/// 3. **The winner reclaims exactly once.** Only the winner reconstructs the
///    SDK's permanent `Arc` (the one `from_raw` pairing the one `into_raw`),
///    into a local whose `Drop` runs on every exit — including a panic in
///    `make_result` — so the refcount is balanced exactly once and can never be
///    double-freed.
///
/// # Panic firewall (R15)
/// `make_result` runs inside `catch_unwind`. A panic there is logged, mapped to
/// a [`BrawError::Other`] result (so the awaiting future resolves to an error
/// instead of hanging), and never unwinds across the FFI boundary. The refcount
/// reclaim and wake still happen.
pub(crate) fn deliver_completion<T>(ud: *mut c_void, make_result: impl FnOnce() -> Result<T, BrawError>) {
    if ud.is_null() {
        // The SDK was handed a non-null pointer, so this is a contract
        // violation. We have no pointer to reclaim; the orphaned refcount is an
        // unavoidable one-off leak (better than reclaiming a bogus pointer).
        log::error!("BRAW completion callback delivered a null user-data pointer; ignoring");
        return;
    }
    let ptr = ud as *const State<T>;

    // Step 1 — take our own temporary strong reference.
    //
    // SAFETY: `ptr` originates from `Arc::<State<T>>::into_raw`; at callback
    // time the SDK's permanent reference is still live, so the strong count is
    // >= 1 and it is valid to `increment_strong_count` then `from_raw` the ref
    // we just added.
    let guard: Arc<State<T>> = unsafe {
        Arc::increment_strong_count(ptr);
        Arc::from_raw(ptr)
    };

    // Step 2 — claim-once. Exactly one caller transitions `false -> true`.
    if guard.claimed.swap(true, Ordering::AcqRel) {
        // Second / spurious callback: a safe no-op. `make_result` is never run,
        // and `guard` drops our temporary reference on return.
        return;
    }

    // Step 3 — we won: reclaim the SDK's *permanent* refcount into an owned
    // `Arc` now, so its `Drop` balances the `into_raw` exactly once on every
    // path below (normal return *or* an unwind out of `make_result`).
    //
    // SAFETY: we are the unique claim winner, so this is the single `from_raw`
    // pairing the single `into_raw` handed to the SDK at submission.
    let permanent: Arc<State<T>> = unsafe { Arc::from_raw(ptr) };

    // Build the result behind the panic firewall (R15): a panic must not cross
    // the C++ ABI boundary, and must still leave the future woken (with an
    // error) rather than hung.
    let result = match catch_unwind(AssertUnwindSafe(make_result)) {
        Ok(r)  => r,
        Err(_) => {
            log::error!("panic while constructing a BRAW job result; reporting it as an error");
            Err(BrawError::Other("panic while constructing BRAW job result".into()))
        }
    };

    // Store the result, then publish it: `done` is Released *after* the write so
    // a `poll` that reads `done` with Acquire always sees the stored result.
    *permanent.result.lock().unwrap_or_else(PoisonError::into_inner) = Some(result);
    permanent.done.store(true, Ordering::Release);
    permanent.waker.wake();
    // `permanent` (the SDK's reference) and `guard` (our temporary one) drop
    // here; `State` is freed only once the future's own clone is also gone.
}

#[cfg(test)]
mod tests {
    //! Hardware-free proofs of the `State` co-ownership / claim-once / panic
    //! firewall contract. These exercise the real `State`, the real
    //! `Arc::into_raw` / `Arc::from_raw` balance, and the real
    //! [`deliver_completion`] dispatcher without the SDK ever running a job.
    use super::*;
    use std::sync::atomic::{ AtomicUsize, Ordering };

    struct DropProbe(Arc<AtomicUsize>);
    impl Drop for DropProbe {
        fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
    }

    /// A future dropped before its callback fires must NOT free the state out
    /// from under the callback; the value the callback stores is freed exactly
    /// once when the SDK's reclaimed refcount is released.
    #[test]
    fn drop_before_callback_is_uaf_free_and_frees_once() {
        let freed = Arc::new(AtomicUsize::new(0));
        let state = Arc::new(State::<DropProbe>::new());
        let ud = Arc::into_raw(state.clone()) as *mut c_void; // SDK's owned ref

        // Caller drops the future before the callback fires (e.g. a seek).
        drop(state);

        // The SDK's reference still keeps the state alive: the callback runs,
        // reclaims that reference, stores its value, and frees exactly once.
        let f = freed.clone();
        deliver_completion::<DropProbe>(ud, move || Ok(DropProbe(f)));
        assert_eq!(freed.load(Ordering::SeqCst), 1, "state + result freed exactly once");
    }

    /// A second / spurious callback for the same job is a safe no-op: the
    /// result is built exactly once and the state is freed exactly once.
    #[test]
    fn double_callback_claims_once() {
        let freed  = Arc::new(AtomicUsize::new(0));
        let builds = Arc::new(AtomicUsize::new(0));
        let state  = Arc::new(State::<DropProbe>::new());
        let ud = Arc::into_raw(state.clone()) as *mut c_void;

        // First (winning) callback: builds the result once. `state` (the
        // future's clone) is kept alive across both callbacks.
        let (f, b) = (freed.clone(), builds.clone());
        deliver_completion::<DropProbe>(ud, move || { b.fetch_add(1, Ordering::SeqCst); Ok(DropProbe(f)) });

        // Second (spurious) callback: must be a no-op — the closure must never
        // run and nothing must be double-freed.
        let b2 = builds.clone();
        deliver_completion::<DropProbe>(ud, move || { b2.fetch_add(1, Ordering::SeqCst); panic!("winner already claimed; this must not run") });

        assert_eq!(builds.load(Ordering::SeqCst), 1, "result built exactly once");
        assert_eq!(freed.load(Ordering::SeqCst),  0, "result still owned by the live future");

        // Take the result out (as `poll` would), then drop the future.
        let taken = state.result.lock().unwrap().take();
        assert!(matches!(taken, Some(Ok(_))), "winning result is present");
        drop(taken);
        assert_eq!(freed.load(Ordering::SeqCst), 1, "value freed exactly once");
        assert_eq!(Arc::strong_count(&state), 1, "only the future's clone remains");
        drop(state);
    }

    /// A panic while constructing the result must be contained (never unwind
    /// across the FFI boundary), still reclaim the refcount, still wake the
    /// future, and surface as an error result.
    #[test]
    fn panicking_result_construction_is_contained() {
        let state = Arc::new(State::<()>::new());
        let ud = Arc::into_raw(state.clone()) as *mut c_void;

        // Silence the default panic hook so the deliberately-panicking closure
        // does not spam the test log; the firewall still catches it.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        deliver_completion::<()>(ud, || panic!("boom in result construction"));
        std::panic::set_hook(prev);

        assert!(state.done.load(Ordering::Acquire), "future was woken despite the panic");
        let taken = state.result.lock().unwrap().take();
        assert!(matches!(taken, Some(Err(BrawError::Other(_)))), "panic surfaced as an error");
        assert_eq!(Arc::strong_count(&state), 1, "SDK refcount reclaimed exactly once (no leak)");
        drop(state);
    }

    /// `abort()` on a future with no job (the `prepare_pipeline` shape) is a
    /// safe no-op.
    #[test]
    fn abort_is_noop_without_job() {
        let fut: CallbackFuture<()> = CallbackFuture { state: Arc::new(State::new()), job: None };
        fut.abort();
    }
}
