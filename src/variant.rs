// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;

#[cfg(target_os = "linux")]
use std::ffi::CString;

#[derive(Debug, Clone)]
pub enum VariantValue {
    Empty,
    U8(u8),
    S16(i16),
    U16(u16),
    S32(i32),
    U32(u32),
    F32(f32),
    String(String),
    ArrayU8(Vec<u8>),
    ArrayI16(Vec<i16>),
    ArrayU16(Vec<u16>),
    ArrayI32(Vec<i32>),
    ArrayU32(Vec<u32>),
    ArrayF32(Vec<f32>),
    F64(f64),
}

impl RawLibrary {
    pub fn variant_to_rust(&self, mut v: VARIANT) -> VariantValue {
        let _self = &self;
        #[cfg(not(target_os = "windows"))] let VariantClear = |a| -> HRESULT { unsafe { (_self.VariantClear)(a) } };
        unsafe {
            let vt = v.Anonymous.Anonymous.vt;
            let ret = match vt {
                VT_EMPTY => VariantValue::Empty,
                VT_UI1   => VariantValue::U8(v.Anonymous.Anonymous.Anonymous.uiVal as u8),
                VT_I2    => VariantValue::S16(v.Anonymous.Anonymous.Anonymous.iVal),
                VT_UI2   => VariantValue::U16(v.Anonymous.Anonymous.Anonymous.uiVal),
                VT_I4    => VariantValue::S32(v.Anonymous.Anonymous.Anonymous.intVal),
                VT_UI4   => VariantValue::U32(v.Anonymous.Anonymous.Anonymous.uintVal),
                VT_R4    => VariantValue::F32(v.Anonymous.Anonymous.Anonymous.fltVal),
                VT_BSTR  => {
                    // String
                    #[cfg(target_os = "windows")] {
                        let b = v.Anonymous.Anonymous.Anonymous.bstrVal as BSTR;
                        if b.is_null() { VariantValue::String(String::new()) } else {
                            use windows_sys::Win32::Foundation::SysStringLen;
                            let len = SysStringLen(b) as usize;
                            let slice = std::slice::from_raw_parts(b, len);
                            VariantValue::String(String::from_utf16_lossy(slice))
                        }
                    }
                    #[cfg(any(target_os = "macos", target_os = "ios"))] {
                        let cf = v.Anonymous.Anonymous.Anonymous.bstrVal as CFStringRef;
                        let tmp = BrawString(cf);
                        let string = tmp.to_string();
                        std::mem::forget(tmp); // Don't free the CFString here, it will be free by VariantClear
                        VariantValue::String(string)
                    }
                    #[cfg(target_os = "linux")] {
                        let p = v.Anonymous.Anonymous.Anonymous.bstrVal as *mut i8;
                        if p.is_null() {
                            VariantValue::String(String::new())
                        } else {
                            VariantValue::String(std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned())
                        }
                    }
                }
                VT_SAFEARRAY => {
                    unsafe fn copy_safearray_elems<T: Copy>(ptr: *const core::ffi::c_void, len: usize) -> Vec<T> {
                        unsafe { core::slice::from_raw_parts(ptr as *const T, len) }.to_vec()
                    }

                    let _this = &self;
                    #[cfg(not(target_os = "windows"))] let SafeArrayGetVartype   = |a, b|    -> HRESULT { (_this.SafeArrayGetVartype  )(a, b) };
                    #[cfg(not(target_os = "windows"))] let SafeArrayGetLBound    = |a, b, c| -> HRESULT { (_this.SafeArrayGetLBound   )(a, b, c) };
                    #[cfg(not(target_os = "windows"))] let SafeArrayGetUBound    = |a, b, c| -> HRESULT { (_this.SafeArrayGetUBound   )(a, b, c) };
                    #[cfg(not(target_os = "windows"))] let SafeArrayAccessData   = |a, b|    -> HRESULT { (_this.SafeArrayAccessData  )(a, b) };
                    //#[cfg(not(target_os = "windows"))] let SafeArrayUnaccessData = |a|       -> HRESULT { (_this.SafeArrayUnaccessData)(a) };

                    let sa = v.Anonymous.Anonymous.Anonymous.parray;
                    if sa.is_null() {
                        return VariantValue::Empty;
                    }

                    // Access data
                    let mut data_ptr: *mut core::ffi::c_void = core::ptr::null_mut();
                    let mut hr = SafeArrayAccessData(sa, &mut data_ptr);
                    if hr != S_OK {
                        eprintln!("Failed to access SafeArray data (hr=0x{:08X})", hr as u32);
                        return VariantValue::Empty;
                    }

                    // Always unaccess on exit
                    struct Unaccess<'a> { sa: *mut SAFEARRAY, _this: &'a RawLibrary }
                    #[cfg(target_os = "windows")]
                    impl Drop for Unaccess<'_> { fn drop(&mut self) { unsafe { let _ = SafeArrayUnaccessData(self.sa); } } }
                    #[cfg(not(target_os = "windows"))]
                    impl Drop for Unaccess<'_> { fn drop(&mut self) { unsafe { let _ = (self._this.SafeArrayUnaccessData)(self.sa); } } }
                    let _guard = Unaccess { sa, _this: self };

                    // Element VARTYPE
                    let mut elem_vt: VARENUM = 0;
                    hr = SafeArrayGetVartype(sa, &mut elem_vt);
                    if hr != S_OK {
                        eprintln!("Failed to get VARTYPE from SafeArray (hr=0x{:08X})", hr as u32);
                        return VariantValue::Empty;
                    }

                    // Bounds (1-based dimension index in COM)
                    let mut lbound: SafeArrayBoundType = 0;
                    let mut ubound: SafeArrayBoundType = -1;
                    hr = SafeArrayGetLBound(sa, 1, &mut lbound);
                    if hr != S_OK {
                        eprintln!("Failed to get LBound from SafeArray (hr=0x{:08X})", hr as u32);
                        return VariantValue::Empty;
                    }
                    hr = SafeArrayGetUBound(sa, 1, &mut ubound);
                    if hr != S_OK {
                        eprintln!("Failed to get UBound from SafeArray (hr=0x{:08X})", hr as u32);
                        return VariantValue::Empty;
                    }

                    let safe_len = if ubound >= lbound { (ubound - lbound + 1) as usize } else { 0 };
                    match elem_vt {
                        VT_UI1 => VariantValue::ArrayU8 (copy_safearray_elems::<u8 >(data_ptr, safe_len)),
                        VT_I2  => VariantValue::ArrayI16(copy_safearray_elems::<i16>(data_ptr, safe_len)),
                        VT_UI2 => VariantValue::ArrayU16(copy_safearray_elems::<u16>(data_ptr, safe_len)),
                        VT_I4  => VariantValue::ArrayI32(copy_safearray_elems::<i32>(data_ptr, safe_len)),
                        VT_UI4 => VariantValue::ArrayU32(copy_safearray_elems::<u32>(data_ptr, safe_len)),
                        VT_R4  => VariantValue::ArrayF32(copy_safearray_elems::<f32>(data_ptr, safe_len)),
                        other => {
                            eprintln!("Unsupported SAFEARRAY element VARTYPE: {}", other);
                            VariantValue::Empty
                        }
                    }
                }
                VT_R8 => VariantValue::F64(v.Anonymous.Anonymous.Anonymous.dblVal),
                _ => VariantValue::Empty,
            };
            VariantClear(&mut v);
            ret
        }
    }
    pub fn variant_from_rust<'lib>(&'lib self, v: VariantValue) -> NativeVariant<'lib> {
        NativeVariant::from_value(self, &v)
    }
}

#[allow(dead_code)]
pub struct NativeVariant<'a> {
    raw: VARIANT,
    lib: &'a RawLibrary,
}

impl<'a> NativeVariant<'a> {
    #[inline]
    pub fn as_raw(&mut self) -> *mut VARIANT {
        &mut self.raw as *mut VARIANT
    }

    pub fn from_value(lib: &'a RawLibrary, value: &VariantValue) -> Self {
        let mut nv = Self { raw: unsafe { std::mem::zeroed() }, lib };
        #[cfg(target_os = "windows")]
        unsafe { Self::fill_variant(&mut nv.raw, value) };
        #[cfg(not(target_os = "windows"))]
        unsafe { Self::fill_variant_nonwin(&mut nv.raw, nv.lib, value) };
        nv
    }

    #[cfg(target_os = "windows")]
    unsafe fn fill_variant(dest: &mut VARIANT, value: &VariantValue) {
        *dest = unsafe { std::mem::zeroed() };
        match value {
            VariantValue::Empty => {
                dest.Anonymous.Anonymous.vt = VT_EMPTY;
            }
            VariantValue::U8(x) => {
                dest.Anonymous.Anonymous.vt = VT_UI1;
                dest.Anonymous.Anonymous.Anonymous.bVal = *x;
            }
            VariantValue::S16(x) => {
                dest.Anonymous.Anonymous.vt = VT_I2;
                dest.Anonymous.Anonymous.Anonymous.iVal = *x;
            }
            VariantValue::U16(x) => {
                dest.Anonymous.Anonymous.vt = VT_UI2;
                dest.Anonymous.Anonymous.Anonymous.uiVal = *x;
            }
            VariantValue::S32(x) => {
                dest.Anonymous.Anonymous.vt = VT_I4;
                dest.Anonymous.Anonymous.Anonymous.intVal = *x;
            }
            VariantValue::U32(x) => {
                dest.Anonymous.Anonymous.vt = VT_UI4;
                dest.Anonymous.Anonymous.Anonymous.uintVal = *x;
            }
            VariantValue::F32(x) => {
                dest.Anonymous.Anonymous.vt = VT_R4;
                dest.Anonymous.Anonymous.Anonymous.fltVal = *x;
            }
            VariantValue::F64(x) => {
                dest.Anonymous.Anonymous.vt = VT_R8;
                dest.Anonymous.Anonymous.Anonymous.dblVal = *x;
            }
            VariantValue::String(s) => {
                let utf16: Vec<u16> = s.encode_utf16().collect();
                let b = if utf16.is_empty() {
                    std::ptr::null_mut()
                } else {
                    unsafe { SysAllocStringLen(utf16.as_ptr(), utf16.len() as u32) }
                };
                dest.Anonymous.Anonymous.vt = VT_BSTR;
                dest.Anonymous.Anonymous.Anonymous.bstrVal = b as _;
            }
            VariantValue::ArrayU8(v)  => Self::fill_safearray_scalar(dest, VT_UI1, v),
            VariantValue::ArrayI16(v) => Self::fill_safearray_scalar(dest, VT_I2 , v),
            VariantValue::ArrayU16(v) => Self::fill_safearray_scalar(dest, VT_UI2, v),
            VariantValue::ArrayI32(v) => Self::fill_safearray_scalar(dest, VT_I4 , v),
            VariantValue::ArrayU32(v) => Self::fill_safearray_scalar(dest, VT_UI4, v),
            VariantValue::ArrayF32(v) => Self::fill_safearray_scalar(dest, VT_R4 , v),
        }
    }

    #[cfg(target_os = "windows")]
    fn fill_safearray_scalar<T: Copy>(dest: &mut VARIANT, elem_vt: VARENUM, data: &[T]) {
        let bound = SAFEARRAYBOUND {
            cElements: data.len() as u32,
            lLbound: 0,
        };
        let sa = unsafe { SafeArrayCreate(elem_vt, 1, &bound) };

        if !sa.is_null() && !data.is_empty() {
            unsafe {
                let mut p: *mut core::ffi::c_void = std::ptr::null_mut();
                if SafeArrayAccessData(sa, &mut p) == 0 {
                    std::ptr::copy_nonoverlapping(
                        data.as_ptr() as *const u8,
                        p as *mut u8,
                        std::mem::size_of_val(data),
                    );
                    let _ = SafeArrayUnaccessData(sa);
                }
            }
        }

        dest.Anonymous.Anonymous.vt = ((VT_SAFEARRAY as u32) | elem_vt as u32) as u16;
        dest.Anonymous.Anonymous.Anonymous.parray = sa;
    }

    #[cfg(not(target_os = "windows"))]
    unsafe fn fill_variant_nonwin(dest: &mut VARIANT, lib: &RawLibrary, value: &VariantValue) {
        *dest = unsafe { std::mem::zeroed() };

        let set_sa = |dest: &mut VARIANT, elem_vt: VARENUM, elem_size: usize, len: usize, src_ptr: *const u8| {
            let mut bound = SAFEARRAYBOUND { cElements: len as u32, lLbound: 0 };
            unsafe {
                let sa = (lib.SafeArrayCreate)(elem_vt as VARENUM, 1, &mut bound);
                if !sa.is_null() && len > 0 {
                    let mut p: *mut core::ffi::c_void = std::ptr::null_mut();
                    if (lib.SafeArrayAccessData)(sa, &mut p) == S_OK {
                        std::ptr::copy_nonoverlapping(src_ptr, p as *mut u8, elem_size * len);
                        let _ = (lib.SafeArrayUnaccessData)(sa);
                    }
                }
                dest.Anonymous.Anonymous.vt = ((VT_SAFEARRAY as u32) | elem_vt as u32) as VARENUM;
                dest.Anonymous.Anonymous.Anonymous.parray = sa;
            }
        };

        match value {
            VariantValue::Empty => { dest.Anonymous.Anonymous.vt = VT_EMPTY as VARENUM; }
            VariantValue::U8(_)  => { panic!("Blackmagic does not use u8 scalar variants") }
            VariantValue::S16(x) => { dest.Anonymous.Anonymous.vt = VT_I2  as VARENUM; dest.Anonymous.Anonymous.Anonymous.iVal = *x; }
            VariantValue::U16(x) => { dest.Anonymous.Anonymous.vt = VT_UI2 as VARENUM; dest.Anonymous.Anonymous.Anonymous.uiVal = *x; }
            VariantValue::S32(x) => { dest.Anonymous.Anonymous.vt = VT_I4  as VARENUM; dest.Anonymous.Anonymous.Anonymous.intVal = *x; }
            VariantValue::U32(x) => { dest.Anonymous.Anonymous.vt = VT_UI4 as VARENUM; dest.Anonymous.Anonymous.Anonymous.uintVal = *x; }
            VariantValue::F32(x) => { dest.Anonymous.Anonymous.vt = VT_R4  as VARENUM; dest.Anonymous.Anonymous.Anonymous.fltVal = *x; }
            VariantValue::F64(x) => { dest.Anonymous.Anonymous.vt = VT_R8  as VARENUM; dest.Anonymous.Anonymous.Anonymous.dblVal = *x; }
            VariantValue::String(s) => {
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                unsafe {
                    let bytes = s.as_bytes();
                    let cf = CFStringCreateWithBytes(std::ptr::null(), bytes.as_ptr(), bytes.len() as isize, kCFStringEncodingUTF8, false);
                    dest.Anonymous.Anonymous.vt = VT_BSTR as VARENUM;
                    dest.Anonymous.Anonymous.Anonymous.bstrVal = cf as *mut _;
                }
                #[cfg(target_os = "linux")]
                {
                    let c = CString::new(s.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
                    dest.Anonymous.Anonymous.vt = VT_BSTR as VARENUM;
                    dest.Anonymous.Anonymous.Anonymous.bstrVal = c.into_raw() as *mut _;
                }
            }
            // arrays
            VariantValue::ArrayU8(v)  => set_sa(dest, VT_UI1 as VARENUM, std::mem::size_of::<u8>(),  v.len(),  v.as_ptr()  as *const u8),
            VariantValue::ArrayI16(v) => set_sa(dest, VT_I2  as VARENUM, std::mem::size_of::<i16>(), v.len(),  v.as_ptr()  as *const u8),
            VariantValue::ArrayU16(v) => set_sa(dest, VT_UI2 as VARENUM, std::mem::size_of::<u16>(), v.len(),  v.as_ptr()  as *const u8),
            VariantValue::ArrayI32(v) => set_sa(dest, VT_I4  as VARENUM, std::mem::size_of::<i32>(), v.len(),  v.as_ptr()  as *const u8),
            VariantValue::ArrayU32(v) => set_sa(dest, VT_UI4 as VARENUM, std::mem::size_of::<u32>(), v.len(),  v.as_ptr()  as *const u8),
            VariantValue::ArrayF32(v) => set_sa(dest, VT_R4  as VARENUM, std::mem::size_of::<f32>(), v.len(),  v.as_ptr()  as *const u8),
        }
    }
}

impl Drop for NativeVariant<'_> {
    fn drop(&mut self) {
        unsafe {
            #[cfg(target_os = "windows")]
            {
                // VariantClear ignores VT_EMPTY / nulls safely
                let _ = VariantClear(&mut self.raw as *mut VARIANT);
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = (self.lib.VariantClear)(&mut self.raw as *mut VARIANT);
            }
        }
    }
}

impl std::fmt::Display for VariantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantValue::Empty => write!(f, ""),
            VariantValue::U8(x)  => write!(f, "{x}"),
            VariantValue::S16(x) => write!(f, "{x}"),
            VariantValue::U16(x) => write!(f, "{x}"),
            VariantValue::S32(x) => write!(f, "{x}"),
            VariantValue::U32(x) => write!(f, "{x}"),
            VariantValue::F32(x) => write!(f, "{x}"),
            VariantValue::F64(x) => write!(f, "{x}"),
            VariantValue::String(s) => write!(f, "{s}"),
            VariantValue::ArrayU8(v)  => write!(f, "{v:?}"),
            VariantValue::ArrayI16(v) => write!(f, "{v:?}"),
            VariantValue::ArrayU16(v) => write!(f, "{v:?}"),
            VariantValue::ArrayI32(v) => write!(f, "{v:?}"),
            VariantValue::ArrayU32(v) => write!(f, "{v:?}"),
            VariantValue::ArrayF32(v) => write!(f, "{v:?}"),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for VariantValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer {
        match self {
            VariantValue::Empty => serializer.serialize_none(),
            VariantValue::U8(x)  => serializer.serialize_u8(*x),
            VariantValue::S16(x) => serializer.serialize_i16(*x),
            VariantValue::U16(x) => serializer.serialize_u16(*x),
            VariantValue::S32(x) => serializer.serialize_i32(*x),
            VariantValue::U32(x) => serializer.serialize_u32(*x),
            VariantValue::F32(x) => serializer.serialize_f32(*x),
            VariantValue::F64(x) => serializer.serialize_f64(*x),
            VariantValue::String(s) => serializer.serialize_str(s),
            VariantValue::ArrayU8(v)  => v.serialize(serializer),
            VariantValue::ArrayI16(v) => v.serialize(serializer),
            VariantValue::ArrayU16(v) => v.serialize(serializer),
            VariantValue::ArrayI32(v) => v.serialize(serializer),
            VariantValue::ArrayU32(v) => v.serialize(serializer),
            VariantValue::ArrayF32(v) => v.serialize(serializer),
        }
    }
}
