// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Decode `sample.braw` through the SDK's custom file I/O (`IBlackmagicRawFile` /
//! `IBlackmagicRawFilesystem`) and prove it matches the path-opened decode
//! bit-for-bit — including the sidecar resolved as a companion file, concurrent
//! read jobs, and the clip keeping its file alive.
//!
//! Requires the Blackmagic RAW SDK in `sdk/` (Windows) or the `.so` set at the repo
//! root (Linux — put the repo root on `LD_LIBRARY_PATH` so the SDK finds its
//! decoder plugins). The decode is CPU-only, so it runs headless. Hashes are
//! compared against the path-opened decode on the same platform, never across
//! platforms.

mod common;

use braw::*;
use common::*;
use std::io;
use std::sync::atomic::{ AtomicUsize, Ordering };
use std::sync::{ Arc, Mutex };

fn sidecar_bytes() -> Vec<u8> { std::fs::read(media_dir().join("sample.sidecar")).expect("read sample.sidecar") }

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

/// Frames decoded by every comparison below (the sample is short).
fn frames_to_check(clip: &BlackmagicRawClip) -> Result<Vec<u64>, BrawError> {
    let n = clip.frame_count()?;
    Ok([0, n / 2, n.saturating_sub(1)].into_iter().collect::<std::collections::BTreeSet<_>>().into_iter().collect())
}

/// The path-opened baseline.
struct Baseline {
    meta: ClipMeta,
    /// `(frame index, hash)`
    hashes: Vec<(u64, u64)>,
    sidecar_attached: bool,
}

fn physical_baseline(codec: &BlackmagicRaw) -> Result<Baseline, BrawError> {
    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    let hashes = frames_to_check(&clip)?
        .into_iter()
        .map(|i| pollster::block_on(decode_hash(&clip, i)).map(|h| (i, h)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Baseline { meta: read_meta(&clip)?, hashes, sidecar_attached: clip.sidecar_file_attached()? })
}

/// A filesystem wrapper recording every companion name the SDK asks for.
struct RecordingFilesystem {
    inner: FileSet,
    opened: Mutex<Vec<String>>,
}
impl BrawFilesystem for RecordingFilesystem {
    fn open_companion(&self, parent: &dyn BrawFile, name: &str) -> io::Result<Arc<dyn BrawFile>> {
        self.opened.lock().unwrap().push(name.to_owned());
        self.inner.open_companion(parent, name)
    }
}

/// A file counting the reads served through it and its own drop.
struct CountingFile {
    inner: BytesFile<Vec<u8>>,
    reads: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
}
impl BrawFile for CountingFile {
    fn name(&self) -> &str { self.inner.name() }
    fn length(&self) -> io::Result<u64> { self.inner.length() }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.inner.read_at(offset, buf)
    }
}
impl Drop for CountingFile {
    fn drop(&mut self) { self.dropped.fetch_add(1, Ordering::SeqCst); }
}

#[test]
fn decode_from_memory_matches_physical() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let Baseline { meta: phys_meta, hashes: phys_hashes, sidecar_attached: phys_sidecar } = physical_baseline(&codec)?;
    println!("physical: {phys_meta:?} hashes={phys_hashes:x?} sidecar={phys_sidecar}");
    assert!(phys_sidecar, "precondition: the SDK attaches sample.sidecar when opening by path");

    let fs = Arc::new(RecordingFilesystem { inner: FileSet::new(), opened: Mutex::new(Vec::new()) });
    fs.inner.insert(Arc::new(BytesFile::new("sample.braw", sample_bytes())));
    fs.inner.insert(Arc::new(BytesFile::new("sample.sidecar", sidecar_bytes())));
    let file = BlackmagicRawFile::new(fs.inner.get("sample.braw").unwrap(), fs.clone());

    let clip = codec.open_clip_from_file(&file)?;
    println!("companions requested: {:?}", fs.opened.lock().unwrap());
    assert!(fs.opened.lock().unwrap().iter().any(|n| n == "sample.sidecar"), "the SDK must look the sidecar up through the filesystem");
    assert_eq!(read_meta(&clip)?, phys_meta, "metadata must match the path-opened clip");
    assert!(clip.sidecar_file_attached()?, "the sidecar served as a companion must attach");
    for &(i, h) in &phys_hashes {
        assert_eq!(pollster::block_on(decode_hash(&clip, i))?, h, "frame {i} must decode bit-identically");
    }

    // Several read jobs in flight at once: the SDK reads concurrently through the file.
    let futures = phys_hashes.iter().map(|&(i, _)| clip.create_read_frame_future(i, &[])).collect::<Result<Vec<_>, _>>()?;
    for (fut, &(i, h)) in futures.into_iter().zip(&phys_hashes) {
        let frame = pollster::block_on(fut)?;
        let processed = pollster::block_on(frame.decode_and_process(None, None))?;
        assert_eq!(fnv1a(processed.resource_cpu()?), h, "concurrent read of frame {i}");
    }

    // A second, independent open of the same bytes.
    let again = codec.open_clip_from_file(&BlackmagicRawFile::new(fs.inner.get("sample.braw").unwrap(), fs.clone()))?;
    assert_eq!(pollster::block_on(decode_hash(&again, phys_hashes[0].0))?, phys_hashes[0].1, "re-open decodes identically");
    Ok(())
}

#[test]
fn stream_file_decodes_like_the_path() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let phys_hashes = physical_baseline(&codec)?.hashes;

    let stream = StreamFile::new("sample.braw", std::fs::File::open(sample_path()).unwrap()).unwrap();
    let clip = codec.open_clip_from_file(&BlackmagicRawFile::standalone(Arc::new(stream)))?;
    assert!(!clip.sidecar_file_attached()?, "a file with no companions opens without a sidecar");
    // Without its sidecar the clip develops with its as-shot settings, which a path
    // open (it always finds the sidecar beside the clip) cannot reproduce — so
    // compare against an in-memory open that also has no companions.
    let bytes_clip = codec.open_clip_from_file(&BlackmagicRawFile::standalone(Arc::new(BytesFile::new("sample.braw", sample_bytes()))))?;
    for &(i, _) in &phys_hashes {
        assert_eq!(pollster::block_on(decode_hash(&clip, i))?, pollster::block_on(decode_hash(&bytes_clip, i))?, "frame {i}: stream == bytes");
    }
    Ok(())
}

#[test]
fn reads_go_through_the_file_and_the_clip_keeps_it_alive() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let reads = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    let file = BlackmagicRawFile::standalone(Arc::new(CountingFile {
        inner: BytesFile::new("sample.braw", sample_bytes()),
        reads: reads.clone(),
        dropped: dropped.clone(),
    }));
    let clip = codec.open_clip_from_file(&file)?;
    // The caller's handle goes away; the clip must keep the file alive.
    drop(file);
    let opened_reads = reads.load(Ordering::Relaxed);
    assert!(opened_reads > 0, "opening must read through the file");
    pollster::block_on(decode_hash(&clip, 0))?;
    assert!(reads.load(Ordering::Relaxed) > opened_reads, "decoding a frame must read through the file");
    assert_eq!(dropped.load(Ordering::SeqCst), 0, "the file must outlive the caller's handle while the clip lives");

    drop(clip);
    codec.clone().flush_jobs()?;
    drop(codec);
    assert_eq!(dropped.load(Ordering::SeqCst), 1, "the file must be released once the clip and codec are gone");
    Ok(())
}
