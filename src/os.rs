#[cfg(target_os = "windows")]
mod abi {
    pub use windows_sys::core::{ BSTR, HRESULT };
    pub use windows_sys::Win32::Foundation::{ S_FALSE, S_OK, E_POINTER, SysAllocStringLen, SysFreeString, SysStringLen };
    pub use windows_sys::Win32::System::Variant::*;
    pub use windows_sys::Win32::System::Ole::*;
    pub use windows_sys::Win32::System::Com::{ SAFEARRAY, SAFEARRAYBOUND };

    pub type SafeArrayBoundType = i32;
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
mod abi {
    use core::ffi::{ c_long, c_void };
    pub type HRESULT = i32; // standard COM-style HRESULT
    pub const S_OK: HRESULT = 0;
    pub const S_FALSE: HRESULT = 1;
    //pub const E_FAIL: HRESULT = 0x8000_0008u32 as i32;
    pub const E_POINTER: HRESULT = 0x8000_0005u32 as i32;

    pub type SafeArrayBoundType = c_long;

    // CoreFoundation types we need (opaque)
    #[repr(transparent)]
    pub struct __CFString(c_void);
    pub type CFStringRef = *const __CFString;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[link(name="CoreFoundation", kind="framework")]
    unsafe extern "C" {
        pub fn CFStringCreateWithBytes(alloc: *const c_void, bytes: *const u8, numBytes: isize, encoding: u32, isExternalRepresentation: bool) -> CFStringRef;
        pub fn CFStringGetCStringPtr(s: CFStringRef, encoding: u32) -> *const i8;
        pub fn CFStringGetCString(s: CFStringRef, buffer: *mut i8, bufferSize: isize, encoding: u32) -> bool;
        pub fn CFStringGetLength(s: CFStringRef) -> isize;
        pub fn CFStringGetMaximumSizeForEncoding(len: isize, encoding: u32) -> isize;
        pub fn CFRetain(cf: *const c_void) -> *const c_void;
        pub fn CFRelease(cf: *const c_void);
    }
    pub const kCFStringEncodingUTF8: u32 = 0x0800_0100;

    // Minimal VARIANT/SafeArray compatible definitions (per header)
    #[repr(C)]
    pub struct SAFEARRAYBOUND { pub lLbound: u32, pub cElements: u32 }
    #[repr(C)]
    pub struct SAFEARRAY {
        pub variantType: u32,
        pub cDims: u32,
        pub pvData: *mut u8,
        pub rgsabound: [SAFEARRAYBOUND; 1],
    }

    // VARIANT per header
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub union VARIANT_u {
        pub iVal: i16,
        pub uiVal: u16,
        pub intVal: i32,
        pub uintVal: u32,
        pub fltVal: f32,
        pub dblVal: f64,
        pub bstrVal: *const c_void, // CFStringRef on macOS
        pub parray: *mut SAFEARRAY,
    }
    impl Default for VARIANT_u {
        fn default() -> Self { VARIANT_u { parray: std::ptr::null_mut() } }
    }
    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct VARIANT_0 {
        pub Anonymous: VARIANT_0_0,
    }
    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct VARIANT_0_0 {
        pub vt: u32,
        pub Anonymous: VARIANT_u,
    }
    #[derive(Default, Clone, Copy)]
    #[repr(C)]
    pub struct VARIANT { pub Anonymous: VARIANT_0 }

    pub type VARENUM = u32;

    pub const VT_EMPTY:     VARENUM = 0;
    pub const VT_UI1:       VARENUM = 1;
    pub const VT_I2:        VARENUM = 2;
    pub const VT_UI2:       VARENUM = 3;
    pub const VT_I4:        VARENUM = 4;
    pub const VT_UI4:       VARENUM = 5;
    pub const VT_R4:        VARENUM = 6;
    pub const VT_BSTR:      VARENUM = 7;
    pub const VT_SAFEARRAY: VARENUM = 8;
    pub const VT_R8:        VARENUM = 9;

    pub type VariantInitFn           = unsafe extern "C" fn(variant: *mut VARIANT) -> HRESULT;
    pub type VariantClearFn          = unsafe extern "C" fn(variant: *mut VARIANT) -> HRESULT;
    pub type SafeArrayCreateFn       = unsafe extern "C" fn(variant_type: VARENUM, dimensions: u32, bounds: *mut SAFEARRAYBOUND) -> *mut SAFEARRAY;
    pub type SafeArrayGetVartypeFn   = unsafe extern "C" fn(sa: *mut SAFEARRAY, out_variant_type: *mut VARENUM) -> HRESULT;
    pub type SafeArrayGetLBoundFn    = unsafe extern "C" fn(sa: *mut SAFEARRAY, dimension: u32, out_lbound: *mut c_long) -> HRESULT;
    pub type SafeArrayGetUBoundFn    = unsafe extern "C" fn(sa: *mut SAFEARRAY, dimension: u32, out_ubound: *mut c_long) -> HRESULT;
    pub type SafeArrayAccessDataFn   = unsafe extern "C" fn(sa: *mut SAFEARRAY, out_data: *mut *mut c_void) -> HRESULT;
    pub type SafeArrayUnaccessDataFn = unsafe extern "C" fn(sa: *mut SAFEARRAY) -> HRESULT;
    pub type SafeArrayDestroyFn      = unsafe extern "C" fn(sa: *mut SAFEARRAY) -> HRESULT;
}

pub(crate) use abi::*;
pub use abi::HRESULT;

pub(crate) type BlackmagicCreateFn = unsafe extern "C" fn() -> *mut super::IBlackmagicRawFactory;
