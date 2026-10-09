// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;
use core::ffi::c_void;
use std::marker::PhantomData;

#[allow(unused_variables)]
pub trait BrawCallback: Send + 'static {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) { }
    fn read_audio_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, audio_buffer: *mut IBlackmagicRawAudioBuffer) { }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) { }
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) { }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { } // offending line will be ignored
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }   // entire file will be ignored
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
impl<T: BrawCallback> CallbackHandle<T> {
    pub fn new(state: T) -> Self {
        Self { raw: ComObject::create(Callback(state)), _state: PhantomData }
    }
    pub fn as_mut_ptr(&self) -> *mut IBlackmagicRawCallback { self.raw.as_raw() }
    /// The callback state. Writing through it races any callback an SDK thread is
    /// running.
    pub fn state_ptr(&self) -> *mut T {
        // SAFETY: the handle's reference keeps the object alive.
        unsafe { &raw mut (*(self.raw.as_raw() as *mut ComObject<Callback<T>>)).state.0 }
    }
}

#[derive(Default)]
pub(crate) struct DefaultCallback {
    pub user_callback: Option<Box<dyn BrawCallback>>,
}

impl BrawCallback for DefaultCallback {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
        // Result construction (`AddRef`) is deferred into a closure so it runs
        // inside `callback_complete`'s panic firewall (R15).
        callback_complete(job, move || if result == S_OK {
            ComPtr::new(frame).map(|mut x| unsafe { x.add_ref(); x })
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = &self.user_callback {
            cb.read_complete(job, result, frame);
        }
    }
    fn read_audio_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, audio_buffer: *mut IBlackmagicRawAudioBuffer) {
        callback_complete(job, move || if result == S_OK {
            ComPtr::new(audio_buffer).map(|mut x| unsafe { x.add_ref(); x })
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = &self.user_callback {
            cb.read_audio_complete(job, result, audio_buffer);
        }
    }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, move || check_hr(result).map(|_| ()));
        if let Some(cb) = &self.user_callback {
            cb.decode_complete(job, result);
        }
    }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
        callback_complete(job, move || if result == S_OK {
            ComPtr::new(processed_image).map(|mut x| unsafe { x.add_ref(); x })
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = &self.user_callback {
            cb.process_complete(job, result, processed_image);
        }
    }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, move || check_hr(result).map(|_| ()));
        if let Some(cb) = &self.user_callback {
            cb.trim_complete(job, result);
        }
    }
    fn prepare_pipeline_complete(&self, user_data: *mut c_void, result: HRESULT) {
        // Claim-once + reclaim of the owned refcount handed to the SDK in
        // `prepare_pipeline` (balances its `Arc::into_raw`). A null `user_data`
        // — an SDK contract violation, since it was handed a non-null pointer —
        // is logged and treated as a no-op inside `deliver_completion`.
        deliver_completion::<()>(user_data, move || check_hr(result).map(|_| ()));
        if let Some(cb) = &self.user_callback {
            cb.prepare_pipeline_complete(user_data, result);
        }
    }
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) {
        if let Some(cb) = &self.user_callback {
            cb.trim_progress(job, progress);
        }
    }
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) {
        if let Some(cb) = &self.user_callback {
            cb.sidecar_metadata_parse_warning(clip, file_name, line_number, info);
        }
    }
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) {
        if let Some(cb) = &self.user_callback {
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
}
