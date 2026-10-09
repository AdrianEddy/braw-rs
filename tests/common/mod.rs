// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The SDK and media the tests run against, and helpers they share.

#![allow(dead_code)] // each test binary uses a subset

use braw::*;
use std::path::PathBuf;

pub fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The SDK library to test against: `$BRAW_SDK_LIBRARY` if set, else the build in
/// this repository for the target under test — the Windows SDK ships x64, ARM64 and
/// ARM64EC builds side by side, and the Linux x86-64 `.so` set sits at the
/// repository root. `None` on a target the repository has no build for.
pub fn library_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("BRAW_SDK_LIBRARY") {
        return Some(path.into());
    }
    let bundled = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "sdk/Win/Libraries/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "sdk/Win/Libraries/ARM64/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "windows", target_arch = "arm64ec")) {
        "sdk/Win/Libraries/ARM64EC/BlackmagicRawAPI.dll"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "libBlackmagicRawAPI.so"
    } else {
        return None;
    };
    Some(repo_dir().join(bundled))
}

/// Load the SDK under test, or `None` — the test then skips — when there is no
/// library for this target. A library that is there but fails to load is an error.
pub fn load_sdk() -> Result<Option<Factory>, BrawError> {
    let Some(path) = library_path() else {
        eprintln!("skipped: no Blackmagic RAW SDK build for this target; point BRAW_SDK_LIBRARY at one");
        return Ok(None);
    };
    Factory::load_from(path).map(Some)
}

pub fn media_dir() -> PathBuf {
    repo_dir().join("sdk/Media")
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
