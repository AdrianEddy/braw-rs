// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! `From<HRESULT> for BrawError` must recognise BOTH of the SDK's COM ABIs.
//!
//! The BRAW SDK is real COM on Windows (`winerror.h` values) but ships its own
//! `LinuxCOM.h` for the macOS / Linux / iPadOS dispatch builds, which defines a
//! COMPLETELY DIFFERENT numeric set (`sdk/Linux/Include/LinuxCOM.h:72-81`).
//!
//! Knowing only the LinuxCOM set left EVERY Windows failure as
//! `OtherHresult(_)`. That is not cosmetic: `E_ABORT` — which the SDK returns for
//! a job the caller deliberately `Abort()`ed (the seek-drain path) — never mapped
//! to [`BrawError::Abort`], so consumers classified a normal cancellation as a
//! hard failure. Likewise `E_FAIL` from a rejected `OpenClip` arrived as an
//! unknown code.
//!
//! The two sets are disjoint, so both are recognised unconditionally.

use braw::BrawError;

fn of(hr: u32) -> BrawError {
    BrawError::from(hr as braw::HRESULT)
}

#[test]
fn windows_winerror_hresults_are_recognised() {
    assert!(matches!(of(0x8000_4001), BrawError::NotImplemented), "E_NOTIMPL");
    assert!(matches!(of(0x8007_000E), BrawError::OutOfMemory), "E_OUTOFMEMORY");
    assert!(matches!(of(0x8007_0057), BrawError::InvalidArgument), "E_INVALIDARG");
    assert!(matches!(of(0x8000_4002), BrawError::NoInterface), "E_NOINTERFACE");
    assert!(matches!(of(0x8000_4003), BrawError::Pointer), "E_POINTER");
    assert!(matches!(of(0x8007_0006), BrawError::Handle), "E_HANDLE");
    assert!(matches!(of(0x8000_4004), BrawError::Abort), "E_ABORT");
    assert!(matches!(of(0x8000_4005), BrawError::Fail), "E_FAIL");
    assert!(matches!(of(0x8007_0005), BrawError::AccessDenied), "E_ACCESSDENIED");
}

#[test]
fn blackmagic_linuxcom_hresults_are_still_recognised() {
    assert!(matches!(of(0x8000_0001), BrawError::NotImplemented));
    assert!(matches!(of(0x8000_0002), BrawError::OutOfMemory));
    assert!(matches!(of(0x8000_0003), BrawError::InvalidArgument));
    assert!(matches!(of(0x8000_0004), BrawError::NoInterface));
    assert!(matches!(of(0x8000_0005), BrawError::Pointer));
    assert!(matches!(of(0x8000_0006), BrawError::Handle));
    assert!(matches!(of(0x8000_0007), BrawError::Abort));
    assert!(matches!(of(0x8000_0008), BrawError::Fail));
    assert!(matches!(of(0x8000_0009), BrawError::AccessDenied));
}

#[test]
fn shared_and_device_lost_codes_are_recognised() {
    // E_UNEXPECTED is identical on both ABIs.
    assert!(matches!(of(0x8000_FFFF), BrawError::Unexpected));
    // DXGI device-removed / -hung / -reset.
    for hr in [0x887A_0005u32, 0x887A_0006, 0x887A_0007] {
        assert!(matches!(of(hr), BrawError::DeviceLost), "{hr:#010X} must be DeviceLost");
    }
}

#[test]
fn an_unknown_code_still_falls_through_verbatim() {
    let hr = 0x1234_5678u32;
    match of(hr) {
        BrawError::OtherHresult(v) => assert_eq!(v as u32, hr),
        other => panic!("expected OtherHresult, got {other}"),
    }
}

/// On Windows a `HRESULT_FROM_WIN32` code is the Win32 error itself — among them the
/// file codes this crate reports to the SDK for a custom file's `NotFound` and
/// `AlreadyExists` — while the generic codes in that facility keep their variants.
#[cfg(target_os = "windows")]
#[test]
fn win32_file_errors_become_io_errors() {
    use std::io::ErrorKind;
    for (hr, code, kind) in [(0x8007_0002u32, 2, ErrorKind::NotFound), (0x8007_0050, 0x50, ErrorKind::AlreadyExists), (0x8007_0003, 3, ErrorKind::NotFound)] {
        match of(hr) {
            BrawError::Io(e) => assert!(e.raw_os_error() == Some(code) && e.kind() == kind, "{hr:#010X}: {e:?}"),
            other => panic!("{hr:#010X}: expected Io, got {other}"),
        }
    }
    assert!(matches!(of(0x8007_0005), BrawError::AccessDenied));
    assert!(matches!(of(0x8007_000E), BrawError::OutOfMemory));
}

#[test]
fn io_errors_convert_with_the_question_mark() {
    fn read() -> Result<Vec<u8>, BrawError> {
        Ok(std::fs::read("this file does not exist.braw")?)
    }
    assert!(matches!(read(), Err(BrawError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound));
}
