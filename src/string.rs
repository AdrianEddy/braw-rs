// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use core::ffi::c_void;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "windows"))]
use crate::os::*;
#[cfg(target_os = "linux")]
use std::ffi::{ CStr, CString };

#[cfg(target_os = "windows")]
type RawStr = BSTR;
#[cfg(any(target_os = "macos", target_os = "ios"))]
type RawStr = CFStringRef;
#[cfg(target_os = "linux")]
type RawStr = *mut i8;

#[repr(transparent)]
#[derive(Debug)]
pub struct BrawString(pub RawStr);

impl BrawString {
    #[inline]
    pub fn as_raw(&self) -> *const c_void {
        self.0 as *const c_void
    }

    #[inline]
    pub fn is_null(&self) -> bool { self.as_raw().is_null() }

    /*// If the SDK says the **caller** must free an *output* string, call this.
    pub unsafe fn free_out(raw: *mut *mut c_void) {
        #[cfg(target_os = "windows")]
        unsafe {
            // If method returned BSTR via out-param, free with SysFreeString
            if !raw.is_null() && !(*raw).is_null() { abi::SysFreeString(*raw as _); *raw = ptr::null_mut(); }
        }
        #[cfg(target_os = "macos")]
        unsafe {
            if !raw.is_null() && !(*raw).is_null() { CFRelease(*raw as _); *raw = ptr::null_mut(); }
        }
        #[cfg(target_os = "linux")]
        unsafe {
            if !raw.is_null() && !(*raw).is_null() { let _ = CString::from_raw(*raw as *mut i8); *raw = ptr::null_mut(); }
        }
    }*/
}
impl From<&str> for BrawString {
    /// Create an *input* string from Rust `&str` appropriate for the platform.
    fn from(s: &str) -> Self {
        #[cfg(target_os = "windows")]
        unsafe{ // Allocate BSTR from UTF-16
            let utf16: Vec<u16> = s.encode_utf16().collect();
            let ptr = SysAllocStringLen(utf16.as_ptr(), utf16.len() as u32);
            Self(ptr)
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        unsafe {
            let bytes = s.as_bytes();
            let cf = CFStringCreateWithBytes(std::ptr::null(), bytes.as_ptr(), bytes.len() as isize, kCFStringEncodingUTF8, false);
            Self(cf)
        }
        #[cfg(target_os = "linux")]
        {
            let c = CString::new(s).expect("CString::new");
            Self(c.into_raw())
        }
    }
}

impl ToString for BrawString {
    /// Convert an *output* platform string to owned Rust `String` by cloning.
    fn to_string(&self) -> String {
        #[cfg(target_os = "windows")]
        unsafe {
            if self.0.is_null() { return String::new(); }
            // BSTR is UTF-16 with length prefix; windows-sys lacks a safe getter, so use SysStringLen via FFI
            let len = SysStringLen(self.0) as usize;
            let slice = std::slice::from_raw_parts(self.0 as *const u16, len);
            String::from_utf16_lossy(slice)
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        unsafe {
            if self.0.is_null() { return String::new(); }
            let len = CFStringGetLength(self.0);
            let cap = CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) + 1;
            let mut buf = vec![0i8; cap as usize];
            let ok = CFStringGetCString(self.0, buf.as_mut_ptr(), cap, kCFStringEncodingUTF8);
            if !ok { return String::new(); }
            let cstr = std::ffi::CStr::from_ptr(buf.as_ptr());
            cstr.to_string_lossy().into_owned()

        }
        #[cfg(target_os = "linux")]
        unsafe {
            if !self.0.is_null() {
                CStr::from_ptr(self.0).to_string_lossy().into_owned()
            } else {
                String::new()
            }
        }
    }
}

impl Drop for BrawString {
    fn drop(&mut self) {
        unsafe {
            #[cfg(target_os = "windows")]
            if !self.0.is_null() { SysFreeString(self.0); self.0 = std::ptr::null_mut(); }
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            if !self.0.is_null() { CFRelease(self.0 as *const _); self.0 = std::ptr::null_mut(); }
            #[cfg(target_os = "linux")]
            if !self.0.is_null() { let _ = CString::from_raw(self.0); self.0 = std::ptr::null_mut(); }
        }
    }
}
