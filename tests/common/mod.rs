// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The SDK and media the tests run against, and helpers they share.
//!
//! The tests use an unpacked Blackmagic RAW SDK — `$BRAW_SDK_DIR`, else `sdk/` in
//! the repository — in the SDK's own layout (`Win/`, `Linux/`, `Mac/`, `Media/`).
//! Without one, the tests that need it pass without running (`--nocapture` shows
//! which) — unless `BRAW_SDK_DIR` or `BRAW_REQUIRE_SDK` is set, when they fail.

#![allow(dead_code)] // each test binary uses a subset

use braw::*;
use std::path::PathBuf;

pub fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The root of the unpacked SDK.
pub fn sdk_dir() -> PathBuf {
    std::env::var_os("BRAW_SDK_DIR").map_or_else(|| repo_dir().join("sdk"), PathBuf::from)
}

/// The SDK library to test against: `$BRAW_SDK_LIBRARY` if set, else the SDK's
/// build for the target under test — the Windows SDK ships x64, ARM64 and ARM64EC
/// builds side by side. `None` when the SDK has no build for this target, or is
/// not there.
pub fn library_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("BRAW_SDK_LIBRARY") {
        return Some(path.into());
    }
    let bundled = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "Win/Libraries/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "Win/Libraries/ARM64/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "windows", target_arch = "arm64ec")) {
        "Win/Libraries/ARM64EC/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "Linux/Libraries/libBlackmagicRawAPI.so"
    } else if cfg!(target_os = "macos") {
        "Mac/Libraries/BlackmagicRawAPI.framework/BlackmagicRawAPI"
    } else {
        return None;
    };
    let path = sdk_dir().join(bundled);
    path.exists().then_some(path)
}

/// Skip a test for want of `what` — or fail it, when the SDK was asked for.
fn skip(what: &str) {
    let required = std::env::var_os("BRAW_SDK_DIR").is_some() || std::env::var_os("BRAW_REQUIRE_SDK").is_some();
    assert!(!required, "{what}");
    eprintln!("skipped: {what}; unpack the SDK into `sdk/`, or set BRAW_SDK_DIR");
}

/// Load the SDK under test, or `None` — the test then skips — when there is no
/// library for this target. A library that is there but fails to load is an error.
pub fn load_sdk() -> Result<Option<Factory>, BrawError> {
    let Some(path) = library_path() else {
        skip("no Blackmagic RAW SDK library for this target");
        return Ok(None);
    };
    Factory::load_from(path).map(Some)
}

/// A file of the unpacked SDK, or `None` — the test then skips — when it is not there.
pub fn sdk_file(rel: &str) -> Option<PathBuf> {
    let path = sdk_dir().join(rel);
    if !path.exists() {
        skip(&format!("{} not found", path.display()));
        return None;
    }
    Some(path)
}

pub fn media_dir() -> PathBuf {
    sdk_dir().join("Media")
}

pub fn sample_path() -> PathBuf {
    media_dir().join("sample.braw")
}

pub fn sample_bytes() -> Vec<u8> {
    std::fs::read(sample_path()).expect("read sample.braw")
}

/// FNV-1a over a byte buffer — a compact, deterministic frame fingerprint.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The fingerprint of frame `index`, decoded and processed with the clip's settings.
pub async fn decode_hash(clip: &BlackmagicRawClip, index: u64) -> Result<u64, BrawError> {
    let frame = clip.read_frame(index).await?;
    let processed = frame.decode_and_process(None, None).await?;
    Ok(fnv1a(processed.resource_cpu()?))
}
