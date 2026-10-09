// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;
use core::ffi::c_void;
use std::marker::PhantomData;
use std::sync::{ Arc, PoisonError, RwLock };

/// Notifications for a codec's jobs, set with [`BlackmagicRaw::set_callback`].
///
/// The futures this crate returns already deliver every job's result; implement
/// this to observe jobs as they finish, or for the notifications no future carries
/// (trim progress, sidecar parse problems). Each method defaults to doing nothing.
///
/// The SDK calls these from its worker threads, concurrently — a callback being
/// replaced may still be running, or about to run, when `set_callback` returns. The
/// SDK keeps the codec's callback alive until the codec is gone, so a callback that
/// holds the codec, or anything opened from it, keeps both alive forever.
#[allow(unused_variables)]
pub trait BrawCallback: Send + Sync + 'static {
    /// A read-frame job finished.
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) { }
    /// A read-audio job finished.
    fn read_audio_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, audio_buffer: *mut IBlackmagicRawAudioBuffer) { }
    /// A manual decoder's decode job finished.
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    /// A decode-and-process job finished.
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) { }
    /// A trim job reported its progress.
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) { }
    /// A trim job finished.
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    /// A line of a `.sidecar` file failed to parse; the line is ignored.
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }
    /// A `.sidecar` file failed to parse; the whole file is ignored.
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }
    /// A pipeline preparation finished. `user_data` belongs to the future
    /// [`prepare_pipeline`](BlackmagicRaw::prepare_pipeline) returned and may already
    /// be freed: compare it, never dereference it.
    fn prepare_pipeline_complete(&self, user_data: *mut c_void, result: HRESULT) { }
}

/// The `IBlackmagicRawCallback` the SDK calls, forwarding to a [`BrawCallback`].
pub(crate) struct Callback<T: BrawCallback>(T);

// SAFETY: the vtable starts with the shared `IUnknown` methods, and every other
// method reaches its state through `callback::<T>(this)`.
unsafe impl<T: BrawCallback> ComClass for Callback<T> {
    type Interface = IBlackmagicRawCallback;
    type VTable = IBlackmagicRawCallbackVTbl;
    const IID: GUID = IID_IBlackmagicRawCallback;
    const NAME: &'static str = "IBlackmagicRawCallback";
    const VTABLE: &'static IBlackmagicRawCallbackVTbl = &IBlackmagicRawCallbackVTbl {
        parent: ComObject::<Self>::IUNKNOWN,
        ReadComplete: cb_read_complete::<T>,
        ReadAudioComplete: cb_read_audio_complete::<T>,
        DecodeComplete: cb_decode_complete::<T>,
        ProcessComplete: cb_process_complete::<T>,
        TrimProgress: cb_trim_progress::<T>,
        TrimComplete: cb_trim_complete::<T>,
        SidecarMetadataParseWarning: cb_sidecar_metadata_parse_warning::<T>,
        SidecarMetadataParseError: cb_sidecar_metadata_parse_error::<T>,
        PreparePipelineComplete: cb_prepare_pipeline_complete::<T>,
    };
}

/// # Safety
/// `this` must be a live `Callback<T>` object — guaranteed by the SDK calling
/// through its vtable.
unsafe fn callback<'a, T: BrawCallback>(this: *mut c_void) -> &'a T {
    &unsafe { ComObject::<Callback<T>>::state(this) }.0
}

unsafe extern "system" fn cb_read_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
    ffi_guard("ReadComplete", (), move || unsafe { callback::<T>(this).read_complete(job, result, frame); });
}
unsafe extern "system" fn cb_read_audio_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT, audio_buffer: *mut IBlackmagicRawAudioBuffer) {
    ffi_guard("ReadAudioComplete", (), move || unsafe { callback::<T>(this).read_audio_complete(job, result, audio_buffer); });
}
unsafe extern "system" fn cb_decode_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT) {
    ffi_guard("DecodeComplete", (), move || unsafe { callback::<T>(this).decode_complete(job, result); });
}
unsafe extern "system" fn cb_process_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
    ffi_guard("ProcessComplete", (), move || unsafe { callback::<T>(this).process_complete(job, result, processed_image); });
}
unsafe extern "system" fn cb_trim_progress<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, progress: f32) {
    ffi_guard("TrimProgress", (), move || unsafe { callback::<T>(this).trim_progress(job, progress); });
}
unsafe extern "system" fn cb_trim_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT) {
    ffi_guard("TrimComplete", (), move || unsafe { callback::<T>(this).trim_complete(job, result); });
}
unsafe extern "system" fn cb_sidecar_metadata_parse_warning<T: BrawCallback>(this: *mut c_void, clip: *mut IBlackmagicRawClip, file_name: *const c_void, line_number: u32, info: *const c_void) {
    ffi_guard("SidecarMetadataParseWarning", (), move || unsafe { callback::<T>(this).sidecar_metadata_parse_warning(clip, read_sdk_string(file_name), line_number, read_sdk_string(info)); });
}
unsafe extern "system" fn cb_sidecar_metadata_parse_error<T: BrawCallback>(this: *mut c_void, clip: *mut IBlackmagicRawClip, file_name: *const c_void, line_number: u32, info: *const c_void) {
    ffi_guard("SidecarMetadataParseError", (), move || unsafe { callback::<T>(this).sidecar_metadata_parse_error(clip, read_sdk_string(file_name), line_number, read_sdk_string(info)); });
}
unsafe extern "system" fn cb_prepare_pipeline_complete<T: BrawCallback>(this: *mut c_void, user_data: *mut c_void, result: HRESULT) {
    ffi_guard("PreparePipelineComplete", (), move || unsafe { callback::<T>(this).prepare_pipeline_complete(user_data, result); });
}

/// A reference to a callback object. The SDK takes its own while it uses the
/// callback, so the object outlives whichever lets go last.
pub(crate) struct CallbackHandle<T: BrawCallback> {
    raw: ComPtr<IBlackmagicRawCallback>,
    _state: PhantomData<T>,
}
// SAFETY: the handle is one reference to an object whose count is atomic and whose
// state, a `Send + Sync` `T`, is only ever reached through `&T`.
unsafe impl<T: BrawCallback> Send for CallbackHandle<T> {}
// SAFETY: as above.
unsafe impl<T: BrawCallback> Sync for CallbackHandle<T> {}
impl<T: BrawCallback> CallbackHandle<T> {
    pub fn new(state: T) -> Self {
        Self { raw: ComObject::create(Callback(state)), _state: PhantomData }
    }
    pub fn as_mut_ptr(&self) -> *mut IBlackmagicRawCallback { self.raw.as_raw() }
    /// The callback state, which SDK threads may be reading concurrently.
    pub fn state(&self) -> &T {
        // SAFETY: the handle's reference keeps the object alive.
        &unsafe { ComObject::<Callback<T>>::state(self.raw.as_raw().cast()) }.0
    }
    /// A reference that keeps the callback alive for as long as the guard lives.
    pub fn add_ref_and_get_guard(&self) -> ComPtrRefGuard { self.raw.add_ref_and_get_guard() }
}

/// The callback each codec is created with: it completes the job futures, then
/// forwards to the [`BrawCallback`] set on the codec, if any.
#[derive(Default)]
pub(crate) struct DefaultCallback {
    user_callback: RwLock<Option<Arc<dyn BrawCallback>>>,
}

impl DefaultCallback {
    pub fn set_user_callback(&self, callback: Option<Arc<dyn BrawCallback>>) {
        let previous = std::mem::replace(&mut *self.user_callback.write().unwrap_or_else(PoisonError::into_inner), callback);
        // Dropped after the lock is released: its `Drop` is user code, which may set
        // another callback.
        drop(previous);
    }
    /// The callback set on the codec. The lock is not held while it runs, so it may
    /// replace itself.
    fn user(&self) -> Option<Arc<dyn BrawCallback>> {
        self.user_callback.read().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl BrawCallback for DefaultCallback {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
        // Result construction (`AddRef`) is deferred into a closure so it runs
        // inside `callback_complete`'s panic firewall. SAFETY (here and below): the
        // SDK passes a live interface, which stays its own; the future gets a new
        // reference.
        callback_complete(job, move || if result == S_OK {
            unsafe { ComPtr::add_ref_from(frame) } // SAFETY: see above
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = self.user() {
            cb.read_complete(job, result, frame);
        }
    }
    fn read_audio_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, audio_buffer: *mut IBlackmagicRawAudioBuffer) {
        callback_complete(job, move || if result == S_OK {
            unsafe { ComPtr::add_ref_from(audio_buffer) } // SAFETY: see above
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = self.user() {
            cb.read_audio_complete(job, result, audio_buffer);
        }
    }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, move || check_hr(result).map(|_| ()));
        if let Some(cb) = self.user() {
            cb.decode_complete(job, result);
        }
    }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
        callback_complete(job, move || if result == S_OK {
            unsafe { ComPtr::add_ref_from(processed_image) } // SAFETY: see above
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = self.user() {
            cb.process_complete(job, result, processed_image);
        }
    }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, move || check_hr(result).map(|_| ()));
        if let Some(cb) = self.user() {
            cb.trim_complete(job, result);
        }
    }
    fn prepare_pipeline_complete(&self, user_data: *mut c_void, result: HRESULT) {
        // Claim-once + reclaim of the owned refcount handed to the SDK in
        // `prepare_pipeline` (balances its `Arc::into_raw`). A null `user_data`
        // — an SDK contract violation, since it was handed a non-null pointer —
        // is logged and treated as a no-op inside `deliver_completion`.
        deliver_completion::<()>(user_data, move || check_hr(result).map(|_| ()));
        if let Some(cb) = self.user() {
            cb.prepare_pipeline_complete(user_data, result);
        }
    }
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) {
        if let Some(cb) = self.user() {
            cb.trim_progress(job, progress);
        }
    }
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) {
        if let Some(cb) = self.user() {
            cb.sidecar_metadata_parse_warning(clip, file_name, line_number, info);
        }
    }
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) {
        if let Some(cb) = self.user() {
            cb.sidecar_metadata_parse_error(clip, file_name, line_number, info);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{ AtomicUsize, Ordering };

    struct Counting(Arc<AtomicUsize>);
    impl BrawCallback for Counting {}
    impl Drop for Counting {
        fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
    }

    #[test]
    fn the_callback_outlives_whichever_of_handle_and_sdk_lets_go_last() {
        let dropped = Arc::new(AtomicUsize::new(0));
        let handle = CallbackHandle::new(Counting(dropped.clone()));
        let raw = handle.as_mut_ptr();
        // The SDK takes a reference in `SetCallback`…
        unsafe { ((*(*raw).vtbl).parent.AddRef)(raw.cast()) };
        drop(handle);
        assert_eq!(dropped.load(Ordering::SeqCst), 0, "the SDK still holds the callback");
        // …and calls into it until it lets go.
        unsafe { ((*(*raw).vtbl).TrimProgress)(raw.cast(), std::ptr::null_mut(), 0.5) };
        assert_eq!(unsafe { ((*(*raw).vtbl).parent.Release)(raw.cast()) }, 0);
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }

    struct CountingProgress(Arc<AtomicUsize>);
    impl BrawCallback for CountingProgress {
        fn trim_progress(&self, _job: *mut IBlackmagicRawJob, _progress: f32) { self.0.fetch_add(1, Ordering::SeqCst); }
    }

    #[test]
    fn a_codec_callback_forwards_to_the_one_set_until_it_is_replaced() {
        let handle = CallbackHandle::new(DefaultCallback::default());
        let raw = handle.as_mut_ptr();
        let progress = |p| unsafe { ((*(*raw).vtbl).TrimProgress)(raw.cast(), std::ptr::null_mut(), p) };
        progress(0.0); // none set yet
        let (first, second) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
        handle.state().set_user_callback(Some(Arc::new(CountingProgress(first.clone()))));
        progress(0.5);
        handle.state().set_user_callback(Some(Arc::new(CountingProgress(second.clone()))));
        progress(1.0);
        assert_eq!((first.load(Ordering::SeqCst), second.load(Ordering::SeqCst)), (1, 1));
    }
}
