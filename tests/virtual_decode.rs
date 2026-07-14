// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Phase 2 milestone (impl-hookfs.md §10/§12/§13): decode `sample.braw` from an
//! in-memory `Cursor` through a synthetic path that does not exist on disk, and
//! prove it matches the physical decode bit-for-bit — including concurrent read
//! jobs and repeated opens — while no virtual path ever touches the real FS.
//!
//! Requires the Blackmagic RAW SDK shipped in this repo and the `sdk/Media`
//! sample. Run with: `cargo test --features hookfs --test virtual_decode`.
//!
//! Cross-platform: on Windows the SDK lives at `sdk/Win/Libraries`, on Linux the
//! `.so` set lives at the repo root. The decode is CPU-only (no GPU pipeline is
//! prepared), so it runs headless. The parity invariant is
//! **physical == virtual on the same platform** — frame hashes are compared
//! against the physical decode on the *current* OS, never across platforms (the
//! SDK's SIMD/codec paths differ between OSes).
//!
//! On Linux the SDK `dlopen`s its decoder plugins (`libDecoder*.so`,
//! `libInstructionSetServices*.so`) by bare name at runtime, so the directory
//! holding them must be on the loader path — set `LD_LIBRARY_PATH` to the repo
//! root when invoking (the standard BMD deployment mechanism; the test performs
//! no environment mutation of its own).

#![cfg(all(feature = "hookfs", any(target_os = "windows", target_os = "linux")))]

use braw::*;
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// FNV-1a over a byte buffer — a compact, deterministic frame fingerprint.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn sdk_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Absolute path to the Blackmagic RAW SDK shared library shipped in this repo.
/// The whole test module is gated to Windows/Linux, so exactly one arm is live.
fn library_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        sdk_dir().join("sdk/Win/Libraries/BlackmagicRawAPI.dll")
    }
    #[cfg(target_os = "linux")]
    {
        sdk_dir().join("libBlackmagicRawAPI.so")
    }
}

fn clip_bytes() -> Vec<u8> {
    std::fs::read(sdk_dir().join("sdk/Media/sample.braw")).expect("read sample.braw")
}

fn sidecar_bytes() -> Vec<u8> {
    std::fs::read(sdk_dir().join("sdk/Media/sample.sidecar")).expect("read sample.sidecar")
}

/// Clip-level metadata captured for physical-vs-virtual comparison.
#[derive(Debug, PartialEq)]
struct ClipMeta {
    width: u32,
    height: u32,
    frame_count: u64,
    frame_rate: f32,
    timecode0: String,
    camera: String,
}

fn read_meta(clip: &BlackmagicRawClip) -> Result<ClipMeta, BrawError> {
    Ok(ClipMeta {
        width: clip.width()?,
        height: clip.height()?,
        frame_count: clip.frame_count()?,
        frame_rate: clip.frame_rate()?,
        timecode0: clip.timecode_for_frame(0)?,
        camera: clip.camera_type()?,
    })
}

/// Decode frame `idx` to CPU pixels and return their FNV-1a hash.
async fn decode_hash(clip: &BlackmagicRawClip, idx: u64) -> Result<u64, BrawError> {
    let frame = clip.read_frame(idx).await?;
    let processed = frame.decode_and_process(None, None).await?;
    Ok(fnv1a(processed.resource_cpu()?))
}

#[test]
fn decode_braw_from_memory_matches_physical() -> Result<(), BrawError> {
    assert!(library_path().exists(), "SDK library missing at {}", library_path().display());

    let bytes = clip_bytes();
    let sidecar = sidecar_bytes();
    let factory = Factory::load_from(library_path())?;

    pollster::block_on(async {
        // ---- 1. PHYSICAL baseline (no hooks installed yet) ------------------
        let codec = factory.create_codec()?;
        let physical_path = sdk_dir().join("sdk/Media/sample.braw");
        let phys_clip = codec.open_clip(physical_path.to_str().unwrap())?;
        let phys_meta = read_meta(&phys_clip)?;
        let phys_hash = decode_hash(&phys_clip, 0).await?;
        drop(phys_clip);
        println!("physical: {phys_meta:?} frame0_fnv1a={phys_hash:016x}");

        // ---- 2. VIRTUAL decode from an in-memory Cursor ---------------------
        // Mount the clip + sidecar under a synthetic path that does not exist.
        let virt = codec
            .virtual_clip("sample.braw")?
            .file("sample.braw", Cursor::new(bytes.clone()))?
            .file("sample.sidecar", Cursor::new(sidecar.clone()))?;
        let synthetic_path = virt.primary_path();
        let clip = virt.open()?;

        let virt_meta = read_meta(&clip)?;
        let virt_hash = decode_hash(&clip, 0).await?;
        println!("virtual : {virt_meta:?} frame0_fnv1a={virt_hash:016x}");
        println!("synthetic path: {}", synthetic_path.display());

        assert_eq!(virt_meta, phys_meta, "clip metadata must match the physical decode");
        assert_eq!(virt_hash, phys_hash, "decoded frame hash must match the physical decode");

        // ---- 3. FAIL-CLOSED: the synthetic path never hits the real FS ------
        assert!(
            !synthetic_path.exists(),
            "synthetic path must not exist on disk: {}",
            synthetic_path.display()
        );
        assert!(
            std::fs::File::open(&synthetic_path).is_err(),
            "opening the synthetic path physically must fail while the SDK still decodes it"
        );
        // The reserved root directory itself must not be a real directory.
        let root = synthetic_path.parent().and_then(Path::parent).unwrap();
        assert!(!root.exists(), "reserved root must not exist on disk: {}", root.display());

        // ---- 4. Repeated opens (sequential) on the same context -------------
        for i in 0..3u32 {
            let name = format!("reopen{i}/sample.braw");
            let sidecar_name = format!("reopen{i}/sample.sidecar");
            let reclip = codec
                .virtual_clip(&name)?
                .file(&name, Cursor::new(bytes.clone()))?
                .file(&sidecar_name, Cursor::new(sidecar.clone()))?
                .open()?;
            assert_eq!(decode_hash(&reclip, 0).await?, phys_hash, "reopen {i} hash");
        }

        Ok::<(), BrawError>(())
    })?;

    // ---- 5. Concurrent read jobs + repeated opens across threads -----------
    // Each thread creates its own codec + virtual clip (unique sub-directory,
    // so mounts don't collide) and decodes frame 0 concurrently. Every result
    // must equal the physical hash.
    let phys_hash = {
        let codec = factory.create_codec()?;
        let clip = codec.open_clip(sdk_dir().join("sdk/Media/sample.braw").to_str().unwrap())?;
        pollster::block_on(decode_hash(&clip, 0))?
    };

    let threads: Vec<_> = (0..4u32)
        .map(|i| {
            let factory = factory.clone();
            let bytes = bytes.clone();
            let sidecar = sidecar.clone();
            std::thread::spawn(move || -> Result<u64, BrawError> {
                let codec = factory.create_codec()?;
                let name = format!("concurrent{i}/sample.braw");
                let sidecar_name = format!("concurrent{i}/sample.sidecar");
                let clip = codec
                    .virtual_clip(&name)?
                    .file(&name, Cursor::new(bytes))?
                    .file(&sidecar_name, Cursor::new(sidecar))?
                    .open()?;
                pollster::block_on(decode_hash(&clip, 0))
            })
        })
        .collect();

    for (i, t) in threads.into_iter().enumerate() {
        let hash = t.join().expect("thread panicked")?;
        assert_eq!(hash, phys_hash, "concurrent job {i} hash must match the physical decode");
    }

    println!("all concurrent jobs + repeated opens matched the physical frame hash");
    Ok(())
}

/// Two virtual clips opened with the **same** `logical_name` in the shared,
/// process-global mount context must not collide: per-clip namespacing gives each
/// its own synthetic sub-tree, so both decode their own bytes and dropping one
/// never unmounts the other (impl-hookfs.md §8/§10). Without the fix the second
/// mount overwrites the first's node and dropping either breaks the other.
#[test]
fn same_logical_name_clips_are_independent() -> Result<(), BrawError> {
    assert!(library_path().exists(), "SDK library missing at {}", library_path().display());

    let bytes = clip_bytes();
    let sidecar = sidecar_bytes();
    let factory = Factory::load_from(library_path())?;
    let codec = factory.create_codec()?;

    // Physical baseline hash (no hooks in play).
    let phys_hash = {
        let clip = codec.open_clip(sdk_dir().join("sdk/Media/sample.braw").to_str().unwrap())?;
        pollster::block_on(decode_hash(&clip, 0))?
    };

    // Two clips, IDENTICAL logical names, independent in-memory buffers.
    let a = codec
        .virtual_clip("sample.braw")?
        .file("sample.braw", Cursor::new(bytes.clone()))?
        .file("sample.sidecar", Cursor::new(sidecar.clone()))?;
    let b = codec
        .virtual_clip("sample.braw")?
        .file("sample.braw", Cursor::new(bytes.clone()))?
        .file("sample.sidecar", Cursor::new(sidecar.clone()))?;

    // Per-clip namespacing: same logical name → DISTINCT synthetic paths.
    assert_ne!(
        a.primary_path(),
        b.primary_path(),
        "same logical_name must map to distinct per-clip synthetic paths, got {}",
        a.primary_path().display()
    );

    let clip_a = a.open()?;
    let clip_b = b.open()?;

    // Both decode their own bytes correctly while both are open.
    assert_eq!(pollster::block_on(decode_hash(&clip_a, 0))?, phys_hash, "clip A decode");
    assert_eq!(pollster::block_on(decode_hash(&clip_b, 0))?, phys_hash, "clip B decode");

    // Dropping ONE same-name clip must not unmount or corrupt the other.
    drop(clip_a);
    assert_eq!(
        pollster::block_on(decode_hash(&clip_b, 0))?,
        phys_hash,
        "dropping the same-name sibling clip must not disturb this clip's reads"
    );

    println!("two same-logical-name clips decoded independently; dropping one left the other intact");
    Ok(())
}
