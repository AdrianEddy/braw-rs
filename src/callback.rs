use super::*;
use std::sync::atomic::{ AtomicU32, Ordering };
use core::ffi::c_void;

#[allow(unused_variables)]
pub trait BrawCallback: Send + 'static {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) { }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) { }
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) { }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { } // offending line will be ignored
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }   // entire file will be ignored
    fn prepare_pipeline_complete(&self, user_data: *mut c_void, result: HRESULT) { }
}

#[repr(C)]
pub(crate) struct CallbackBox<T: BrawCallback> {
    vtbl: *const IBlackmagicRawCallbackVTbl,
    refcnt: AtomicU32,
    pub state: T,
}

#[cfg(target_os = "windows")]
type QueryInterfaceRiid = *const GUID;
#[cfg(not(target_os = "windows"))]
type QueryInterfaceRiid = GUID;
#[inline]
fn guid_from_riid(riid: QueryInterfaceRiid) -> GUID {
    #[cfg(target_os = "windows")]
    unsafe { *riid }
    #[cfg(not(target_os = "windows"))]
    { riid }
}

// Helpers to get the typed self back from `this`
unsafe fn cb_from_this<T: BrawCallback>(this: *mut c_void) -> *mut CallbackBox<T> {
    this as *mut CallbackBox<T>
}

// IUnknown — signatures must match platform
unsafe extern "C" fn cb_qi<T: BrawCallback>(this: *mut c_void, riid: QueryInterfaceRiid, ppv: *mut *mut c_void) -> HRESULT {
    const IID_IUNKNOWN: GUID = GUID::new([0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xC0,0x00,0x00,0x00,0x00,0x00,0x00,0x46]);
    unsafe {
        if ppv.is_null() { return E_POINTER; }
        *ppv = std::ptr::null_mut();

        // Accept IUnknown and IID_IBlackmagicRawCallback
        let want_iid = guid_from_riid(riid);
        dbg!(&want_iid);
        if want_iid == IID_IUNKNOWN || want_iid == IID_IBlackmagicRawCallback {
            *ppv = this;
            cb_addref::<T>(this);
            return S_OK;
        }
    }
    0x80004002u32 as i32 // E_NOINTERFACE
}

unsafe extern "C" fn cb_addref<T: BrawCallback>(this: *mut c_void) -> u32 {
    let me = unsafe { &*cb_from_this::<T>(this) };
    me.refcnt.fetch_add(1, Ordering::Relaxed) + 1
}

unsafe extern "C" fn cb_release<T: BrawCallback>(this: *mut c_void) -> u32 {
    unsafe {
        let me = &*cb_from_this::<T>(this);
        let prev = me.refcnt.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(cb_from_this::<T>(this))); // free
            0
        } else {
            prev - 1
        }
    }
}

unsafe extern "C" fn cb_read_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
    unsafe { (*cb_from_this::<T>(this)).state.read_complete(job, result, frame); }
}
unsafe extern "C" fn cb_decode_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT) {
    unsafe { (*cb_from_this::<T>(this)).state.decode_complete(job, result); }
}
unsafe extern "C" fn cb_process_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
    unsafe { (*cb_from_this::<T>(this)).state.process_complete(job, result, processed_image); }
}
unsafe extern "C" fn cb_trim_progress<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, progress: f32) {
    unsafe { (*cb_from_this::<T>(this)).state.trim_progress(job, progress); }
}
unsafe extern "C" fn cb_trim_complete<T: BrawCallback>(this: *mut c_void, job: *mut IBlackmagicRawJob, result: HRESULT) {
    unsafe { (*cb_from_this::<T>(this)).state.trim_complete(job, result); }
}
unsafe extern "C" fn cb_sidecar_metadata_parse_warning<T: BrawCallback>(this: *mut c_void, clip: *mut IBlackmagicRawClip, file_name: *const c_void, line_number: u32, info: *const c_void) {
    unsafe { (*cb_from_this::<T>(this)).state.sidecar_metadata_parse_warning(clip, BrawString(file_name as *mut _).to_string(), line_number, BrawString(info as *mut _).to_string()); }
}
unsafe extern "C" fn cb_sidecar_metadata_parse_error<T: BrawCallback>(this: *mut c_void, clip: *mut IBlackmagicRawClip, file_name: *const c_void, line_number: u32, info: *const c_void) {
    unsafe { (*cb_from_this::<T>(this)).state.sidecar_metadata_parse_error(clip, BrawString(file_name as *mut _).to_string(), line_number, BrawString(info as *mut _).to_string()); }
}
unsafe extern "C" fn cb_prepare_pipeline_complete<T: BrawCallback>(this: *mut c_void, user_data: *mut c_void, result: HRESULT) {
    unsafe { (*cb_from_this::<T>(this)).state.prepare_pipeline_complete(user_data, result); }
}

impl<T: BrawCallback> CallbackBox<T> {
    pub const VTABLE: IBlackmagicRawCallbackVTbl = IBlackmagicRawCallbackVTbl {
        parent: IUnknownVTbl {
            QueryInterface: cb_qi::<T>,
            AddRef: cb_addref::<T>,
            Release: cb_release::<T>,
        },
        ReadComplete: cb_read_complete::<T>,
        DecodeComplete: cb_decode_complete::<T>,
        ProcessComplete: cb_process_complete::<T>,
        TrimProgress: cb_trim_progress::<T>,
        TrimComplete: cb_trim_complete::<T>,
        SidecarMetadataParseWarning: cb_sidecar_metadata_parse_warning::<T>,
        SidecarMetadataParseError: cb_sidecar_metadata_parse_error::<T>,
        PreparePipelineComplete: cb_prepare_pipeline_complete::<T>,
    };

    pub fn new(state: T) -> Box<Self> {
        Box::new(Self {
            vtbl: &Self::VTABLE,   // &'static to a const, no mutation
            refcnt: AtomicU32::new(1),
            state,
        })
    }
}

#[allow(dead_code)]
pub(crate) struct CallbackHandle<T: BrawCallback> {
    raw: *mut IBlackmagicRawCallback,
    owned: Box<CallbackBox<T>>,
}
impl<T: BrawCallback> CallbackHandle<T> {
    pub fn new(state: T) -> Self {
        let owned = CallbackBox::new(state);
        let raw = (&*owned) as *const _ as *mut IBlackmagicRawCallback;
        Self { raw, owned }
    }
    pub fn as_mut_ptr(&self) -> *mut IBlackmagicRawCallback { self.raw }
}

#[derive(Default)]
pub(crate) struct DefaultCallback {
    pub user_callback: Option<Box<dyn BrawCallback>>,
}

impl BrawCallback for DefaultCallback {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
        callback_complete(job, if result == S_OK {
            ComPtr::new(frame).map(|mut x| unsafe { x.add_ref(); x })
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = &self.user_callback {
            cb.read_complete(job, result, frame);
        }
    }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, check_hr(result).map(|_| ()));
        if let Some(cb) = &self.user_callback {
            cb.decode_complete(job, result);
        }
    }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) {
        callback_complete(job, if result == S_OK {
            ComPtr::new(processed_image).map(|mut x| unsafe { x.add_ref(); x })
        } else {
            check_hr(result).map(|_| unreachable!())
        });
        if let Some(cb) = &self.user_callback {
            cb.process_complete(job, result, processed_image);
        }
    }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) {
        callback_complete(job, check_hr(result).map(|_| ()));
        if let Some(cb) = &self.user_callback {
            cb.trim_complete(job, result);
        }
    }
    fn prepare_pipeline_complete(&self, user_data: *mut c_void, result: HRESULT) {
        if user_data.is_null() {
            return;
        }
        // Safety: `user_data` points to the State inside an Arc held by the Future.
        let state: &State<()> = unsafe { &*(user_data as *const State<()>) };

        // Store it and signal completion
        {
            let mut lock = state.result.lock().unwrap();
            *lock = Some(check_hr(result).map(|_| ()));
        }
        state.done.store(true, Ordering::Release);
        state.waker.wake();
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
