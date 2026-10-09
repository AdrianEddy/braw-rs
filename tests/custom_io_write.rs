// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Drive the SDK to **write** through the custom file I/O — a sidecar saved into
//! a companion file the filesystem creates, a trim into a [`MemoryFile`], a cube
//! file — and read the results back, proving the bytes are served from memory and
//! never touch a disk.
//!
//! Same requirements as `custom_io_decode` (SDK in `sdk/` on Windows, `.so` set on
//! `LD_LIBRARY_PATH` on Linux).

mod common;

use braw::*;
use common::*;
use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{ AtomicUsize, Ordering };
use std::sync::{ Arc, Mutex };

/// The entries directly under each of `dirs`.
fn snapshot(dirs: &[PathBuf]) -> BTreeSet<PathBuf> {
    dirs.iter().flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path())).collect()
}

/// Fail-closed: nothing appeared in `dirs` since `before` — the write created no
/// file on a real filesystem.
fn assert_no_disk_leak(dirs: &[PathBuf], before: &BTreeSet<PathBuf>, context: &str) {
    let leaked: Vec<PathBuf> = snapshot(dirs).difference(before).cloned().collect();
    assert!(leaked.is_empty(), "{context} leaked file(s) to disk: {leaked:?}");
}

/// Change an editable string metadata key, returning the `(key, value)` that stuck.
fn change_a_metadata_attribute(clip: &BlackmagicRawClip) -> Option<(String, String)> {
    let new_val = "BRAW_RS_CUSTOM_IO".to_string();
    let mut candidates: Vec<String> = clip
        .metadata_iter()
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| matches!(v, VariantValue::String(_)).then_some(k))
        .collect();
    for k in ["reel", "scene", "take", "production_name", "director"] {
        if !candidates.iter().any(|c| c == k) {
            candidates.push(k.to_string());
        }
    }
    candidates.into_iter().find_map(|key| {
        (clip.set_metadata(&key, VariantValue::String(new_val.clone())).is_ok()
            && matches!(clip.metadata(&key), Ok(VariantValue::String(ref s)) if *s == new_val))
        .then(|| (key, new_val.clone()))
    })
}

/// A [`FileSet`] recording every companion the SDK creates.
struct RecordingSet {
    files: FileSet,
    created: Mutex<Vec<(String, bool)>>,
}
impl BrawFilesystem for RecordingSet {
    fn open_companion(&self, parent: &dyn BrawFile, name: &str) -> io::Result<Arc<dyn BrawFile>> {
        self.files.open_companion(parent, name)
    }
    fn create_companion(&self, parent: &dyn BrawFile, name: &str, replace_existing: bool) -> io::Result<Arc<dyn BrawFile>> {
        self.created.lock().unwrap().push((name.to_owned(), replace_existing));
        self.files.create_companion(parent, name, replace_existing)
    }
}

#[test]
fn sidecar_save_reload_round_trip() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;

    // Only the clip: `SaveSidecarFile` must create the sidecar through the filesystem.
    let set = Arc::new(RecordingSet { files: FileSet::new(), created: Mutex::new(Vec::new()) });
    set.files.insert(Arc::new(BytesFile::new("sample.braw", sample_bytes())));
    let clip = codec.open_clip_from_file(&BlackmagicRawFile::new(set.files.get("sample.braw").unwrap(), set.clone()))?;
    assert!(!clip.sidecar_file_attached()?, "no sidecar before the save");

    let (key, new_val) = change_a_metadata_attribute(&clip).expect("an editable string metadata key");
    println!("changed metadata `{key}` = {new_val:?}");

    let dirs = [std::env::current_dir().unwrap(), media_dir()];
    let before = snapshot(&dirs);
    clip.save_sidecar_file()?;
    assert_no_disk_leak(&dirs, &before, "sidecar save");

    println!("companions created: {:?}", set.created.lock().unwrap());
    let sidecar = set.files.get("sample.sidecar").expect("SaveSidecarFile must create sample.sidecar through the filesystem");
    let bytes = sidecar.read_to_vec().unwrap();
    let text = std::str::from_utf8(&bytes).expect("the sidecar is UTF-8");
    assert!(text.trim_start().starts_with('{') && text.trim_end().ends_with('}'), "the sidecar is a JSON object: {text:.80}");
    assert!(text.contains(&new_val), "the saved sidecar carries the edit");

    clip.reload_sidecar_file()?;
    assert!(clip.sidecar_file_attached()?, "the reloaded sidecar attaches");
    assert!(matches!(clip.metadata(&key), Ok(VariantValue::String(ref s)) if *s == new_val), "the edit round-trips through the sidecar");
    Ok(())
}

#[test]
fn trim_writes_a_valid_clip_to_memory() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let source = codec.open_clip_from_file(&BlackmagicRawFile::standalone(Arc::new(BytesFile::new("source.braw", sample_bytes()))))?;
    let source_hash = pollster::block_on(decode_hash(&source, 0))?;

    let output = Arc::new(MemoryFile::new("trimmed.braw"));
    let outputs = Arc::new(FileSet::new());
    outputs.insert(output.clone());
    let destination = BlackmagicRawFile::new(output.clone(), outputs.clone());

    let dirs = [std::env::current_dir().unwrap(), media_dir()];
    let before = snapshot(&dirs);
    pollster::block_on(source.trim_to_file(&destination, 0, 1, None, None))?;
    assert_no_disk_leak(&dirs, &before, "trim");

    let bytes = output.contents();
    println!("trim wrote {} bytes", bytes.len());
    assert!(!bytes.is_empty(), "the trim must write the clip through the file");
    let trimmed = codec.open_clip_from_file(&BlackmagicRawFile::standalone(Arc::new(BytesFile::new("trimmed.braw", bytes))))?;
    assert_eq!((trimmed.width()?, trimmed.height()?, trimmed.frame_count()?), (source.width()?, source.height()?, 1));
    assert_eq!(pollster::block_on(decode_hash(&trimmed, 0))?, source_hash, "the trimmed frame decodes identically");
    Ok(())
}

#[test]
fn cube_is_written_through_the_file() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    let attributes = clip.clone_clip_processing_attributes()?;
    let lut = match attributes.post_3d_lut() {
        Ok(lut) => lut,
        Err(e) => panic!("the sample carries a 3D LUT (embedded or in its sidecar), but post_3d_lut failed: {e}"),
    };
    let cube = Arc::new(MemoryFile::new("lut.cube"));
    lut.write_cube_to_file(&BlackmagicRawFile::standalone(cube.clone()))?;
    let text = String::from_utf8(cube.contents()).expect("a cube file is text");
    println!("cube: {} bytes, head {:?}", text.len(), &text[..text.len().min(80)]);
    assert!(text.contains(&format!("LUT_3D_SIZE {}", lut.size()?)), "a cube file declares its size");
    Ok(())
}

#[test]
fn multicard_presence_resolves_through_the_filesystem() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let set = Arc::new(FileSet::new());
    set.insert(Arc::new(BytesFile::new("A001.braw", sample_bytes())));
    let clip = codec.open_clip_from_file(&BlackmagicRawFile::new(set.get("A001.braw").unwrap(), set.clone()))?;
    let count = clip.multicard_file_count()?;
    println!("multicard: count = {count}");
    assert!(count >= 1, "a clip is recorded on at least one card");
    assert!(clip.is_multicard_file_present(0)?, "the clip's own card is present");
    Ok(())
}

/// A [`MemoryFile`] counting the SDK's `commit` calls.
struct CommitCounting {
    inner: MemoryFile,
    commits: AtomicUsize,
}
impl CommitCounting {
    fn new(name: &str) -> Arc<Self> { Arc::new(Self { inner: MemoryFile::new(name), commits: AtomicUsize::new(0) }) }
    fn commits(&self) -> usize { self.commits.load(Ordering::SeqCst) }
}
impl BrawFile for CommitCounting {
    fn name(&self) -> &str { self.inner.name() }
    fn length(&self) -> io::Result<u64> { self.inner.length() }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> { self.inner.read_at(offset, buf) }
    fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<usize> { self.inner.write_at(offset, buf) }
    fn set_length(&self, length: u64) -> io::Result<()> { self.inner.set_length(length) }
    fn commit(&self) -> io::Result<()> {
        assert!(self.inner.length()? > 0, "`{}` is committed once written", self.name());
        self.commits.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// A filesystem creating [`CommitCounting`] companions.
#[derive(Default)]
struct CommitCountingSet {
    created: Mutex<Vec<Arc<CommitCounting>>>,
}
impl BrawFilesystem for CommitCountingSet {
    fn open_companion(&self, _parent: &dyn BrawFile, _name: &str) -> io::Result<Arc<dyn BrawFile>> {
        Err(io::ErrorKind::NotFound.into())
    }
    fn create_companion(&self, _parent: &dyn BrawFile, name: &str, _replace_existing: bool) -> io::Result<Arc<dyn BrawFile>> {
        let file = CommitCounting::new(name);
        self.created.lock().unwrap().push(file.clone());
        Ok(file)
    }
}

/// `BrawFile::commit` is documented as called once on every file the SDK writes —
/// not only the companions it creates, as Blackmagic's API reference has it: its
/// own S3 trim sample uploads a trim destination it made itself in `CommitFile`.
#[test]
fn the_sdk_commits_every_file_it_writes_exactly_once() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;

    let set = Arc::new(CommitCountingSet::default());
    let clip = codec.open_clip_from_file(&BlackmagicRawFile::new(Arc::new(BytesFile::new("sample.braw", sample_bytes())), set.clone()))?;
    let (key, _) = change_a_metadata_attribute(&clip).expect("an editable string metadata key");
    clip.save_sidecar_file()?;
    let created = set.created.lock().unwrap().clone();
    assert_eq!(created.iter().map(|f| (f.name().to_owned(), f.commits())).collect::<Vec<_>>(), [("sample.sidecar".to_owned(), 1)], "after saving `{key}`");

    let trim = CommitCounting::new("trimmed.braw");
    pollster::block_on(clip.trim_to_file(&BlackmagicRawFile::standalone(trim.clone()), 0, 1, None, None))?;
    assert_eq!(trim.commits(), 1, "the trim destination");

    let lut = codec.open_clip(sample_path().to_str().unwrap())?.clone_clip_processing_attributes()?.post_3d_lut()?;
    let cube = CommitCounting::new("lut.cube");
    lut.write_cube_to_file(&BlackmagicRawFile::standalone(cube.clone()))?;
    assert_eq!(cube.commits(), 1, "the cube file");
    Ok(())
}
