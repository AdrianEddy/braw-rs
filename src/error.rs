// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;

/// An error from the SDK, from loading it, or from I/O done on its behalf.
#[derive(Debug)]
#[non_exhaustive]
pub enum BrawError {
    /// The SDK returned a null pointer or value where one was expected.
    NullValue,
    /// `E_UNEXPECTED`: catastrophic failure.
    Unexpected,
    /// `E_NOTIMPL`: not implemented.
    NotImplemented,
    /// `E_OUTOFMEMORY`: an allocation failed.
    OutOfMemory,
    /// `E_INVALIDARG`: an argument was invalid.
    InvalidArgument,
    /// `E_NOINTERFACE`: the object does not implement the requested interface.
    NoInterface,
    /// `E_POINTER`: an invalid pointer was passed.
    Pointer,
    /// `E_HANDLE`: an invalid handle was passed.
    Handle,
    /// `E_ABORT`: the operation was aborted, e.g. a job cancelled with `abort()`.
    Abort,
    /// `E_FAIL`: unspecified failure.
    Fail,
    /// `E_ACCESSDENIED`: access was denied.
    AccessDenied,
    /// The GPU device was lost, removed or reset.
    DeviceLost,
    /// Any other `HRESULT`.
    OtherHresult(HRESULT),
    /// The SDK library failed to load, or lacks an entry point.
    Libloading(libloading::Error),
    /// The loaded library does not implement the Blackmagic RAW SDK 6.0 codec
    /// interface these bindings are built on — it is older, or a newer release
    /// changed the interface again. Carries the library's camera support version
    /// (`"unknown"` if it cannot be read).
    UnsupportedSdkVersion(String),
    /// An I/O error: from a file read or written while working with the SDK, or a
    /// Win32 file error the SDK reported (Windows only).
    Io(std::io::Error),
    /// Any other error, described by its message.
    Other(String),
}
impl Clone for BrawError {
    fn clone(&self) -> Self {
        match self {
            BrawError::NullValue          => BrawError::NullValue,
            BrawError::Unexpected         => BrawError::Unexpected,
            BrawError::NotImplemented     => BrawError::NotImplemented,
            BrawError::OutOfMemory        => BrawError::OutOfMemory,
            BrawError::InvalidArgument    => BrawError::InvalidArgument,
            BrawError::NoInterface        => BrawError::NoInterface,
            BrawError::Pointer            => BrawError::Pointer,
            BrawError::Handle             => BrawError::Handle,
            BrawError::Abort              => BrawError::Abort,
            BrawError::Fail               => BrawError::Fail,
            BrawError::AccessDenied       => BrawError::AccessDenied,
            BrawError::DeviceLost         => BrawError::DeviceLost,
            BrawError::OtherHresult(hr)   => BrawError::OtherHresult(*hr),
            BrawError::Libloading(e)      => BrawError::Other(e.to_string()), // Workaround for libloading::Error not being Clone, which is https://github.com/rust-lang/rust/issues/24135
            BrawError::UnsupportedSdkVersion(v) => BrawError::UnsupportedSdkVersion(v.clone()),
            // `io::Error` is not `Clone` either: rebuild it from its OS code, or its kind and message.
            BrawError::Io(e)              => BrawError::Io(match e.raw_os_error() {
                Some(code) => std::io::Error::from_raw_os_error(code),
                None       => std::io::Error::new(e.kind(), e.to_string()),
            }),
            BrawError::Other(s)           => BrawError::Other(s.clone()),
        }
    }
}

impl std::fmt::Display for BrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrawError::NullValue          => write!(f, "BRAW error: Null value"),
            BrawError::Unexpected         => write!(f, "BRAW error: Unexpected"),
            BrawError::NotImplemented     => write!(f, "BRAW error: Not implemented"),
            BrawError::OutOfMemory        => write!(f, "BRAW error: Out of memory"),
            BrawError::InvalidArgument    => write!(f, "BRAW error: Invalid argument"),
            BrawError::NoInterface        => write!(f, "BRAW error: No interface"),
            BrawError::Pointer            => write!(f, "BRAW error: Pointer error"),
            BrawError::Handle             => write!(f, "BRAW error: Handle error"),
            BrawError::Abort              => write!(f, "BRAW error: Abort"),
            BrawError::Fail               => write!(f, "BRAW error: Fail"),
            BrawError::AccessDenied       => write!(f, "BRAW error: Access denied"),
            BrawError::DeviceLost         => write!(f, "BRAW error: GPU device lost"),
            BrawError::OtherHresult(hr)   => write!(f, "BRAW error: HRESULT 0x{hr:X}"),
            BrawError::Libloading(e)      => write!(f, "BRAW error: Libloading error: {e}"),
            BrawError::UnsupportedSdkVersion(v) => {
                write!(f, "BRAW error: the loaded Blackmagic RAW SDK library (camera support version {v}) does not implement the SDK 6.0 interfaces these bindings require")?;
                match major_minor(v).map(|found| found.cmp(&(6, 0))) {
                    Some(std::cmp::Ordering::Less)    => f.write_str("; it predates 6.0, so load the SDK 6.0 library"),
                    Some(std::cmp::Ordering::Greater) => f.write_str("; it is newer than these bindings, so load the SDK 6.0 library or update the bindings"),
                    _                                 => Ok(()),
                }
            }
            BrawError::Io(e)              => write!(f, "BRAW error: I/O error: {e}"),
            BrawError::Other(s)           => write!(f, "BRAW error: {s}"),
        }
    }
}
impl std::error::Error for BrawError { }

/// The `(major, minor)` of a version string such as `"6.0"` or `"5.1.2"`.
fn major_minor(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = match parts.next() { Some(m) => m.parse().ok()?, None => 0 };
    Some((major, minor))
}

/// The COM error codes the BRAW SDK returns, on **both** of its ABIs.
///
/// The SDK is COM on Windows (`winerror.h` values) but ships its own
/// `LinuxCOM.h` for the macOS / Linux / iPadOS dispatch builds, which defines a
/// completely different numeric set. The two sets are disjoint, so recognising
/// both here is unambiguous and keeps the mapping platform-independent — in
/// particular `E_ABORT`, which the SDK returns for a job the caller aborted, is
/// [`BrawError::Abort`] everywhere, so a cancellation can be told from a failure.
///
/// On Windows, any other `HRESULT_FROM_WIN32` code (`0x8007xxxx`) is a Win32 error
/// — among them the `ERROR_FILE_NOT_FOUND` / `ERROR_FILE_EXISTS` this crate reports
/// to the SDK for a custom file's `NotFound` / `AlreadyExists` — and becomes
/// [`BrawError::Io`] carrying it, with its kind and the system's message.
impl From<HRESULT> for BrawError {
    fn from(hr: HRESULT) -> Self {
        match hr as u32 {
            // E_UNEXPECTED — identical on both ABIs.
            0x8000FFFF => BrawError::Unexpected,

            // ── Blackmagic `LinuxCOM.h` (macOS / Linux / iPadOS) ──
            0x80000001 => BrawError::NotImplemented,
            0x80000002 => BrawError::OutOfMemory,
            0x80000003 => BrawError::InvalidArgument,
            0x80000004 => BrawError::NoInterface,
            0x80000005 => BrawError::Pointer,
            0x80000006 => BrawError::Handle,
            0x80000007 => BrawError::Abort,
            0x80000008 => BrawError::Fail,
            0x80000009 => BrawError::AccessDenied,

            // ── Windows `winerror.h` ──
            0x80004001 => BrawError::NotImplemented,  // E_NOTIMPL
            0x8007000E => BrawError::OutOfMemory,     // E_OUTOFMEMORY
            0x80070057 => BrawError::InvalidArgument, // E_INVALIDARG
            0x80004002 => BrawError::NoInterface,     // E_NOINTERFACE
            0x80004003 => BrawError::Pointer,         // E_POINTER
            0x80070006 => BrawError::Handle,          // E_HANDLE
            0x80004004 => BrawError::Abort,           // E_ABORT
            0x80004005 => BrawError::Fail,            // E_FAIL
            0x80070005 => BrawError::AccessDenied,    // E_ACCESSDENIED
            #[cfg(target_os = "windows")]
            code @ 0x80070000..=0x8007FFFF => BrawError::Io(std::io::Error::from_raw_os_error((code & 0xFFFF) as i32)),

            0x000002C8
            | 0x000002BE
            | 0xFFFFFFDE
            | 0xFFFFFFDC
            | 0xFFFFFFDF
            | 0x887A0005
            | 0x887A0006
            | 0x887A0007 => BrawError::DeviceLost,
            _ => BrawError::OtherHresult(hr),
        }
    }
}
impl From<libloading::Error> for BrawError {
    fn from(e: libloading::Error) -> Self { BrawError::Libloading(e) }
}
impl From<std::io::Error> for BrawError {
    fn from(e: std::io::Error) -> Self { BrawError::Io(e) }
}

/// The outcome of a raw SDK call: `Ok(true)` for `S_OK`, `Ok(false)` for `S_FALSE`.
pub type BrawResult = Result<bool, BrawError>;

pub(crate) fn check_hr(hr: HRESULT) -> BrawResult { if hr == S_OK { Ok(true) } else if hr == S_FALSE { Ok(false) } else { Err(BrawError::from(hr)) } }

#[cfg(test)]
mod tests {
    use super::*;

    fn message(version: &str) -> String { BrawError::UnsupportedSdkVersion(version.into()).to_string() }

    #[test]
    fn the_version_message_points_the_right_way() {
        assert!(message("5.0").contains("predates 6.0"));
        assert!(message("4.2.1").contains("predates 6.0"));
        assert!(message("6.1").contains("newer than these bindings"));
        assert!(message("7.0").contains("newer than these bindings"));
        for neutral in ["unknown", "6.0", ""] {
            let m = message(neutral);
            assert!(!m.contains("predates") && !m.contains("newer"), "{m}");
        }
    }

    #[test]
    fn an_io_error_clones_with_its_kind_and_message() {
        let cloned = BrawError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "no such clip")).clone();
        assert!(matches!(&cloned, BrawError::Io(e) if e.kind() == std::io::ErrorKind::NotFound && e.to_string() == "no such clip"));
        let os = BrawError::Io(std::io::Error::from_raw_os_error(2)).clone();
        assert!(matches!(&os, BrawError::Io(e) if e.raw_os_error() == Some(2)));
    }
}
