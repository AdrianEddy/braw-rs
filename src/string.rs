// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The SDK's native string type — `BSTR` on Windows, `CFStringRef` on Apple,
//! NUL-terminated UTF-8 `char*` on Linux — and its three ownership cases:
//!
//! * [`BrawString`] — a string *we* allocate and pass `[in]`; we free it.
//! * [`take_sdk_string`] — an `[out]` string the SDK allocated for us; the
//!   receiver frees it, with the SDK's deallocator.
//! * [`read_sdk_string`] — an `[in]` string the SDK passes into one of our COM
//!   objects (a callback, a filesystem); borrowed, never freed.
//!
//! [`alloc_sdk_string`] is the mirror of [`take_sdk_string`]: an `[out]` string
//! we return to the SDK from one of our COM objects, which the SDK frees.
//!
//! The Linux SDK allocates `[out]` strings with `new char[]` and frees the ones
//! it receives with `delete[]` (`COM::NativeStringFree` in the SDK samples).
//! Both resolve to the C allocator, so those strings are exchanged through
//! `malloc` / `free` — never through Rust's global allocator, which an
//! application may have replaced.

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

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

/// A string allocated by Rust and passed to the SDK as an `[in]` parameter.
#[repr(transparent)]
#[derive(Debug)]
pub struct BrawString(RawStr);
// SAFETY: the string is owned, immutable, and freed by an allocator any thread may
// call (`SysFreeString`, `CFRelease`, the C allocator).
unsafe impl Send for BrawString {}

impl BrawString {
    /// The native string, still owned by `self`.
    #[inline]
    pub fn as_raw(&self) -> *const c_void {
        self.0 as *const c_void
    }

    /// Whether allocating the native string failed.
    #[inline]
    pub fn is_null(&self) -> bool { self.as_raw().is_null() }
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

impl std::fmt::Display for BrawString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&unsafe { read_sdk_string(self.as_raw()) })
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

/// Copy a native SDK string into a Rust `String` without taking ownership of it.
///
/// # Safety
/// `raw` must be null or a valid native string (see the module docs) that stays
/// alive for the duration of the call.
pub(crate) unsafe fn read_sdk_string(raw: *const c_void) -> String {
    if raw.is_null() { return String::new(); }
    #[cfg(target_os = "windows")]
    unsafe {
        let len = SysStringLen(raw as BSTR) as usize;
        String::from_utf16_lossy(std::slice::from_raw_parts(raw as *const u16, len))
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    unsafe {
        let cf = raw as CFStringRef;
        let len = CFStringGetLength(cf);
        let cap = CFStringGetMaximumSizeForEncoding(len, kCFStringEncodingUTF8) + 1;
        let mut buf = vec![0i8; cap as usize];
        if !CFStringGetCString(cf, buf.as_mut_ptr(), cap, kCFStringEncodingUTF8) { return String::new(); }
        std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
    }
    #[cfg(target_os = "linux")]
    unsafe {
        CStr::from_ptr(raw as *const i8).to_string_lossy().into_owned()
    }
}

/// Convert an `[out]` string the SDK allocated into a Rust `String`, then free it
/// with the SDK's deallocator.
///
/// # Safety
/// `raw` must be null or a native string the SDK transferred ownership of to the
/// caller; it must not be used afterwards.
pub(crate) unsafe fn take_sdk_string(raw: *mut c_void) -> String {
    let s = unsafe { read_sdk_string(raw) };
    if !raw.is_null() {
        unsafe {
            #[cfg(target_os = "windows")]
            SysFreeString(raw as BSTR);
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            CFRelease(raw as *const _);
            #[cfg(target_os = "linux")]
            free(raw);
        }
    }
    s
}

/// Allocate a native string to return to the SDK through an `[out]` parameter;
/// ownership passes to the SDK, which frees it. Null on allocation failure.
pub(crate) fn alloc_sdk_string(s: &str) -> *mut c_void {
    #[cfg(target_os = "windows")]
    unsafe {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        SysAllocStringLen(utf16.as_ptr(), utf16.len() as u32) as *mut c_void
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    unsafe {
        let bytes = s.as_bytes();
        CFStringCreateWithBytes(std::ptr::null(), bytes.as_ptr(), bytes.len() as isize, kCFStringEncodingUTF8, false) as *mut c_void
    }
    #[cfg(target_os = "linux")]
    unsafe {
        // The SDK reads the name as a C string, so an interior NUL truncates it.
        let bytes = s.as_bytes();
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let p = malloc(len + 1) as *mut u8;
        if !p.is_null() {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, len);
            *p.add(len) = 0;
        }
        p as *mut c_void
    }
}
