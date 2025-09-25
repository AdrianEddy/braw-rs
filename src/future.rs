// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use std::{ future::Future, pin::Pin, sync::{ atomic::{ AtomicBool, Ordering}, Arc, Mutex }, task::{ Context, Poll }};
use futures_util::task::AtomicWaker;
use core::ffi::c_void;
use super::*;

pub(crate) struct State<T> {
    pub(crate) waker: AtomicWaker,
    pub(crate) done: AtomicBool,
    pub(crate) result: Mutex<Option<Result<T, BrawError>>>,
}
impl<T> State<T> {
    pub(crate) fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            done: AtomicBool::new(false),
            result: Mutex::new(None),
        }
    }
}
pub struct CallbackFuture<T> {
    pub(crate) state: Arc<State<T>>,
    pub(crate) job: Option<ComPtr<IBlackmagicRawJob>>,
}
impl<T> Future for CallbackFuture<T> {
    type Output = Result<T, BrawError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        this.state.waker.register(cx.waker());
        if this.state.done.load(Ordering::Acquire) {
            if let Some(res) = this.state.result.lock().unwrap().take() {
                return Poll::Ready(res);
            }
        }
        Poll::Pending
    }
}

impl<T> Drop for CallbackFuture<T> {
    fn drop(&mut self) {
        if !self.state.done.load(Ordering::Acquire) {
            if let Some(job) = &self.job {
                let _ = job.Abort();
            }
        }
    }
}

impl<T> CallbackFuture<T> {
    pub fn create_from_job(job: ComPtr<IBlackmagicRawJob>, hints: &[ReadJobHints]) -> Result<Self, BrawError> {
        let state = Arc::new(State::new());

        job.SetUserData(Arc::as_ptr(&state) as *mut c_void)?;

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

        Ok(Self { state, job: Some(job) })
    }
}

pub(crate) fn callback_complete<T>(job: *mut IBlackmagicRawJob, ret: Result<T, BrawError>) {
    if job.is_null() {
        log::error!("Job pointer is null in callback.");
        return;
    }
    let mut ud: *mut c_void = std::ptr::null_mut();
    let _ = unsafe { ((*(*job).vtbl).GetUserData)(job as *mut _, &mut ud) };

    if ud.is_null() {
        log::error!("No user data in job.");
        return;
    }

    // Safety: `ud` points to the State inside an Arc held by the Future.
    let state: &State<T> = unsafe { &*(ud as *const State<T>) };

    // Store the result and signal completion
    {
        let mut lock = state.result.lock().unwrap();
        *lock = Some(ret);
    }
    state.done.store(true, Ordering::Release);
    state.waker.wake();
}

