// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! A library older than SDK 6.0 lays its codec vtable out differently (6.0
//! inserted `OpenClipFromFile[WithGeometry]` before `SetCallback`), so calling
//! into it would jump to the wrong methods. `Factory::create_codec` must refuse
//! it — before its first call through the codec — and name the version it found.
//!
//! Needs a pre-6.0 library, which the SDK 6.0 download does not include: point
//! `BRAW_LEGACY_SDK_LIBRARY` at one (e.g. the 5.0 `BlackmagicRawAPI.dll`) and run
//! with `--ignored`. It lives in its own test binary because it loads that
//! library into the process.

use braw::*;

#[test]
#[ignore = "needs BRAW_LEGACY_SDK_LIBRARY pointing at a pre-6.0 Blackmagic RAW SDK library"]
fn pre_6_0_library_is_refused() {
    let path = std::env::var_os("BRAW_LEGACY_SDK_LIBRARY").expect("BRAW_LEGACY_SDK_LIBRARY");
    let factory = Factory::load_from(&path).expect("load the legacy library");
    let e = match factory.create_codec() {
        Err(e) => e,
        Ok(_) => panic!("a pre-6.0 library must be refused"),
    };
    let BrawError::UnsupportedSdkVersion(version) = &e else { panic!("expected UnsupportedSdkVersion, got {e}") };
    println!("refused SDK {version}: {e}");
    assert!(!version.starts_with("6") && version != "unknown", "the error names the old library's camera support version, got {version:?}");
    assert!(e.to_string().contains("predates 6.0"), "the message says which way the version is off: {e}");
}
