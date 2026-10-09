// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Custom file I/O: read clips from — and write sidecars, trims and cube files
//! to — anything that is not a path on a mounted volume (memory, a network
//! stream, an encrypted container, …), through the SDK's `IBlackmagicRawFile` /
//! `IBlackmagicRawFilesystem` interfaces.
//!
//! Implement [`BrawFile`] for a byte source (or use [`BytesFile`], [`StreamFile`]
//! or [`MemoryFile`]) and [`BrawFilesystem`] for where its companion files live
//! (or use [`FileSet`]), wrap them in a [`BlackmagicRawFile`], and hand that to
//! [`BlackmagicRaw::open_clip_from_file`]:
//!
//! ```no_run
//! # use braw::*;
//! # use std::sync::Arc;
//! # fn demo() -> Result<(), BrawError> {
//! let braw = Factory::load_from(default_library_name())?;
//! let codec = braw.create_codec()?;
//! let bytes = std::fs::read("A001.braw")?;
//! let file = BlackmagicRawFile::standalone(Arc::new(BytesFile::new("A001.braw", bytes)));
//! let clip = codec.open_clip_from_file(&file)?;
//! # let _ = clip;
//! # Ok(())
//! # }
//! ```
//!
//! The SDK calls these objects from its own worker threads, concurrently, so every
//! implementation is `Send + Sync` and synchronises internally.

use super::*;
use core::ffi::c_void;
use std::collections::HashMap;
use std::io::{ self, Read, Seek, SeekFrom };
use std::sync::{ Arc, Mutex, PoisonError, RwLock };

/// A file the SDK reads a clip from, or writes a sidecar, trim or cube file to.
///
/// The SDK calls these methods from several worker threads at once: reads and
/// writes are positional, so an implementation needs no notion of a cursor, but it
/// must synchronise internally.
pub trait BrawFile: Send + Sync + 'static {
    /// The file's name. The SDK names a clip's companion files after it: for
    /// `A001.braw` it asks the [`BrawFilesystem`] for `A001.sidecar`.
    fn name(&self) -> &str;

    /// The file's length in bytes.
    fn length(&self) -> io::Result<u64>;

    /// Read up to `buf.len()` bytes at `offset` into `buf`, returning how many were
    /// read. A short read is retried, so returning `0` means end of file.
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize>;

    /// Write `buf` at `offset`, extending the file as needed, and return how many
    /// bytes were written. Read-only files keep the default, `Unsupported`.
    fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<usize> {
        let _ = (offset, buf);
        Err(io::ErrorKind::Unsupported.into())
    }

    /// Truncate or extend the file to `length` bytes.
    fn set_length(&self, length: u64) -> io::Result<()> {
        let _ = length;
        Err(io::ErrorKind::Unsupported.into())
    }

    /// Flush written data out of any write cache. May be called repeatedly while
    /// the file is being written.
    fn flush(&self) -> io::Result<()> { Ok(()) }

    /// Called once when the SDK has finished writing the file and expects no
    /// further writes — on every file it writes: a saved sidecar from
    /// [`BrawFilesystem::create_companion`], and the destination handed to a trim or
    /// a cube write. The place to finalise or upload it.
    fn commit(&self) -> io::Result<()> { Ok(()) }

    /// The byte alignment the SDK should use for the best I/O performance.
    fn preferred_io_alignment(&self) -> u32 { 4096 }

    /// Read the whole file into a `Vec`.
    fn read_to_vec(&self) -> io::Result<Vec<u8>> {
        let length = usize::try_from(self.length()?).map_err(|_| io::Error::from(io::ErrorKind::OutOfMemory))?;
        let mut out = vec![0u8; length];
        let mut done = 0;
        while done < length {
            match self.read_at(done as u64, &mut out[done..])? {
                0 => break,
                n => done += n,
            }
        }
        out.truncate(done);
        Ok(out)
    }
}

/// Where a clip's companion files live: its `.sidecar`, the other cards of a
/// multi-card recording, and the files the SDK creates beside it.
pub trait BrawFilesystem: Send + Sync + 'static {
    /// Open `name`, a companion of `parent`. A companion that does not exist is a
    /// `NotFound` error — the SDK probes for optional ones, such as the sidecar.
    fn open_companion(&self, parent: &dyn BrawFile, name: &str) -> io::Result<Arc<dyn BrawFile>>;

    /// Create `name` beside `parent`, for the SDK to write (e.g. a saved sidecar).
    /// With `replace_existing == false`, an existing file is an `AlreadyExists`
    /// error. Read-only filesystems keep the default, `Unsupported`.
    fn create_companion(&self, parent: &dyn BrawFile, name: &str, replace_existing: bool) -> io::Result<Arc<dyn BrawFile>> {
        let _ = (parent, name, replace_existing);
        Err(io::ErrorKind::Unsupported.into())
    }
}

/// A filesystem with no companion files: every lookup is `NotFound`, so a clip
/// opened through it has no sidecar.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoCompanions;

impl BrawFilesystem for NoCompanions {
    fn open_companion(&self, _parent: &dyn BrawFile, _name: &str) -> io::Result<Arc<dyn BrawFile>> {
        Err(io::ErrorKind::NotFound.into())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ready-made files and filesystems
// ─────────────────────────────────────────────────────────────────────────────

/// A read-only file over bytes in memory — `Vec<u8>`, `Arc<[u8]>`,
/// `&'static [u8]`, `Cow<'static, [u8]>`, … — read without locking or copying.
pub struct BytesFile<B> {
    name: String,
    bytes: B,
}

impl<B: AsRef<[u8]> + Send + Sync + 'static> BytesFile<B> {
    pub fn new(name: impl Into<String>, bytes: B) -> Self {
        Self { name: name.into(), bytes }
    }
}

impl<B: AsRef<[u8]> + Send + Sync + 'static> BrawFile for BytesFile<B> {
    fn name(&self) -> &str { &self.name }
    fn length(&self) -> io::Result<u64> { Ok(self.bytes.as_ref().len() as u64) }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        Ok(copy_from(self.bytes.as_ref(), offset, buf))
    }
}

/// A read-only file over a `Read + Seek` stream. The stream has a single cursor,
/// so concurrent SDK reads take turns on it.
pub struct StreamFile<R> {
    name: String,
    length: u64,
    stream: Mutex<R>,
}

impl<R: Read + Seek + Send + 'static> StreamFile<R> {
    /// Wrap `stream`, measuring its length by seeking to its end.
    pub fn new(name: impl Into<String>, mut stream: R) -> io::Result<Self> {
        let length = stream.seek(SeekFrom::End(0))?;
        Ok(Self { name: name.into(), length, stream: Mutex::new(stream) })
    }
}

impl<R: Read + Seek + Send + 'static> BrawFile for StreamFile<R> {
    fn name(&self) -> &str { &self.name }
    fn length(&self) -> io::Result<u64> { Ok(self.length) }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        if offset >= self.length { return Ok(0); }
        let mut stream = self.stream.lock().unwrap_or_else(PoisonError::into_inner);
        stream.seek(SeekFrom::Start(offset))?;
        stream.read(buf)
    }
}

/// A readable and writable file held in memory — a destination for sidecars,
/// trims and cube files that must never touch a disk.
pub struct MemoryFile {
    name: String,
    data: RwLock<Vec<u8>>,
}

impl MemoryFile {
    /// An empty file.
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_contents(name, Vec::new())
    }
    /// A file holding `data`.
    pub fn with_contents(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self { name: name.into(), data: RwLock::new(data) }
    }
    /// A copy of the file's current contents.
    pub fn contents(&self) -> Vec<u8> {
        self.data.read().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl BrawFile for MemoryFile {
    fn name(&self) -> &str { &self.name }
    fn length(&self) -> io::Result<u64> {
        Ok(self.data.read().unwrap_or_else(PoisonError::into_inner).len() as u64)
    }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        Ok(copy_from(&self.data.read().unwrap_or_else(PoisonError::into_inner), offset, buf))
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<usize> {
        let start = usize::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let end = start.checked_add(buf.len()).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        let mut data = self.data.write().unwrap_or_else(PoisonError::into_inner);
        if data.len() < end { data.resize(end, 0); }
        data[start..end].copy_from_slice(buf);
        Ok(buf.len())
    }
    fn set_length(&self, length: u64) -> io::Result<()> {
        let length = usize::try_from(length).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        self.data.write().unwrap_or_else(PoisonError::into_inner).resize(length, 0);
        Ok(())
    }
}

/// A filesystem of named files — a clip, its sidecar, the other cards of a
/// multi-card recording. Files the SDK creates are added as [`MemoryFile`]s and
/// can be read back with [`get`](Self::get).
#[derive(Default)]
pub struct FileSet {
    files: Mutex<HashMap<String, Arc<dyn BrawFile>>>,
}

impl FileSet {
    pub fn new() -> Self { Self::default() }

    /// Add `file` under its [`name`](BrawFile::name), returning the file it replaced.
    pub fn insert(&self, file: Arc<dyn BrawFile>) -> Option<Arc<dyn BrawFile>> {
        self.lock().insert(file.name().to_owned(), file)
    }

    /// The file called `name`.
    pub fn get(&self, name: &str) -> Option<Arc<dyn BrawFile>> {
        self.lock().get(name).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<dyn BrawFile>>> {
        self.files.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl BrawFilesystem for FileSet {
    fn open_companion(&self, _parent: &dyn BrawFile, name: &str) -> io::Result<Arc<dyn BrawFile>> {
        self.get(name).ok_or_else(|| io::ErrorKind::NotFound.into())
    }
    fn create_companion(&self, _parent: &dyn BrawFile, name: &str, replace_existing: bool) -> io::Result<Arc<dyn BrawFile>> {
        let mut files = self.lock();
        if !replace_existing && files.contains_key(name) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let file: Arc<dyn BrawFile> = Arc::new(MemoryFile::new(name));
        files.insert(name.to_owned(), file.clone());
        Ok(file)
    }
}

/// Copy `src[offset..]` into `buf`, returning the byte count (0 past the end).
fn copy_from(src: &[u8], offset: u64, buf: &mut [u8]) -> usize {
    let Ok(start) = usize::try_from(offset) else { return 0 };
    let Some(tail) = src.get(start..) else { return 0 };
    let n = tail.len().min(buf.len());
    buf[..n].copy_from_slice(&tail[..n]);
    n
}

// ─────────────────────────────────────────────────────────────────────────────
// The COM objects the SDK sees
// ─────────────────────────────────────────────────────────────────────────────

/// A [`BrawFile`] presented to the SDK as an `IBlackmagicRawFile`, bound to the
/// [`BrawFilesystem`] its companion files resolve through.
///
/// A cheap reference-counted handle: clones share one COM object. A clip opened
/// from it keeps it alive for as long as the clip lives.
#[derive(Clone)]
pub struct BlackmagicRawFile {
    raw: ComPtr<IBlackmagicRawFile>,
}
// SAFETY: the COM object only holds `Arc<dyn BrawFile>` and `Arc<dyn
// BrawFilesystem>` (both `Send + Sync`) behind an atomic refcount, and every
// method on it is safe to call from any thread.
unsafe impl Send for BlackmagicRawFile {}
// SAFETY: as above — the handle exposes no mutable state.
unsafe impl Sync for BlackmagicRawFile {}

impl BlackmagicRawFile {
    /// Present `file` to the SDK, with its companions resolved through `filesystem`.
    pub fn new(file: Arc<dyn BrawFile>, filesystem: Arc<dyn BrawFilesystem>) -> Self {
        let filesystem = FilesystemObject::create(filesystem);
        Self { raw: FileObject::create(file, filesystem) }
    }

    /// Present `file` to the SDK with no companion files (see [`NoCompanions`]).
    pub fn standalone(file: Arc<dyn BrawFile>) -> Self {
        Self::new(file, Arc::new(NoCompanions))
    }

    /// Get the raw COM interface pointer.
    pub fn as_raw(&self) -> *mut IBlackmagicRawFile { self.raw.as_raw() }

    pub(crate) fn add_ref_and_get_guard(&self) -> ComPtrRefGuard { self.raw.add_ref_and_get_guard() }
}

/// The Rust side of an `IBlackmagicRawFile`.
struct FileObject {
    file: Arc<dyn BrawFile>,
    filesystem: ComPtr<IBlackmagicRawFilesystem>,
}

/// The Rust side of an `IBlackmagicRawFilesystem`.
struct FilesystemObject {
    filesystem: Arc<dyn BrawFilesystem>,
}

// SAFETY: `FILE_VTBL` starts with the shared `IUnknown` methods, and every other
// method reaches its state through `file_object(this)`.
unsafe impl ComClass for FileObject {
    type Interface = IBlackmagicRawFile;
    type VTable = IBlackmagicRawFileVTbl;
    const IID: GUID = IID_IBlackmagicRawFile;
    const NAME: &'static str = "IBlackmagicRawFile";
    // A `static`, so that `ComObject::downcast` recognises our files.
    const VTABLE: &'static IBlackmagicRawFileVTbl = &FILE_VTBL;
}

// SAFETY: as for `FileObject`, through `filesystem_object(this)`.
unsafe impl ComClass for FilesystemObject {
    type Interface = IBlackmagicRawFilesystem;
    type VTable = IBlackmagicRawFilesystemVTbl;
    const IID: GUID = IID_IBlackmagicRawFilesystem;
    const NAME: &'static str = "IBlackmagicRawFilesystem";
    const VTABLE: &'static IBlackmagicRawFilesystemVTbl = &FILESYSTEM_VTBL;
}

impl FileObject {
    fn create(file: Arc<dyn BrawFile>, filesystem: ComPtr<IBlackmagicRawFilesystem>) -> ComPtr<IBlackmagicRawFile> {
        ComObject::create(Self { file, filesystem })
    }
}

impl FilesystemObject {
    fn create(filesystem: Arc<dyn BrawFilesystem>) -> ComPtr<IBlackmagicRawFilesystem> {
        ComObject::create(Self { filesystem })
    }
}

/// The HRESULT the SDK should see for an I/O error from user code.
fn hresult_from_io(op: &str, name: &str, e: &io::Error) -> HRESULT {
    let hr = match e.kind() {
        io::ErrorKind::NotFound         => E_FILE_NOT_FOUND,
        io::ErrorKind::AlreadyExists    => E_FILE_EXISTS,
        io::ErrorKind::Unsupported      => E_NOTIMPL,
        io::ErrorKind::InvalidInput     => E_INVALIDARG,
        io::ErrorKind::PermissionDenied => E_ACCESSDENIED,
        io::ErrorKind::OutOfMemory      => E_OUTOFMEMORY,
        _                               => E_FAIL,
    };
    if e.kind() == io::ErrorKind::NotFound {
        log::debug!("BRAW custom I/O: {op} `{name}`: {e}");
    } else {
        log::warn!("BRAW custom I/O: {op} `{name}` failed: {e}");
    }
    hr
}

// ── IBlackmagicRawFile ──

static FILE_VTBL: IBlackmagicRawFileVTbl = IBlackmagicRawFileVTbl {
    parent: ComObject::<FileObject>::IUNKNOWN,
    GetFilesystem: file_get_filesystem,
    GetFileName: file_get_file_name,
    SetFileLength: file_set_file_length,
    GetFileLength: file_get_file_length,
    ReadV: file_read_v,
    WriteV: file_write_v,
    GetPreferredIoAlignment: file_get_preferred_io_alignment,
    FlushWrites: file_flush_writes,
    CommitFile: file_commit_file,
};

/// # Safety
/// `this` must be a live [`FileObject`] — guaranteed by the SDK calling through
/// the vtable of an object we created.
unsafe fn file_object<'a>(this: *mut c_void) -> &'a FileObject { unsafe { ComObject::state(this) } }

unsafe extern "system" fn file_get_filesystem(this: *mut c_void, filesystem_out: *mut *mut IBlackmagicRawFilesystem) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::GetFilesystem", E_UNEXPECTED, move || unsafe {
        if filesystem_out.is_null() { return E_POINTER; }
        // The caller receives (and later releases) its own reference.
        *filesystem_out = file_object(this).filesystem.clone().into_raw();
        S_OK
    })
}
unsafe extern "system" fn file_get_file_name(this: *mut c_void, file_name: *mut *mut c_void) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::GetFileName", E_UNEXPECTED, move || unsafe {
        if file_name.is_null() { return E_POINTER; }
        let name = alloc_sdk_string(file_object(this).file.name());
        *file_name = name;
        if name.is_null() { E_OUTOFMEMORY } else { S_OK }
    })
}
unsafe extern "system" fn file_set_file_length(this: *mut c_void, file_length: u64) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::SetFileLength", E_UNEXPECTED, move || {
        let file = &unsafe { file_object(this) }.file;
        match file.set_length(file_length) {
            Ok(()) => S_OK,
            Err(e) => hresult_from_io("set length of", file.name(), &e),
        }
    })
}
unsafe extern "system" fn file_get_file_length(this: *mut c_void, file_length: *mut u64) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::GetFileLength", E_UNEXPECTED, move || {
        if file_length.is_null() { return E_POINTER; }
        let file = &unsafe { file_object(this) }.file;
        match file.length() {
            Ok(n) => { unsafe { *file_length = n; } S_OK }
            Err(e) => hresult_from_io("get length of", file.name(), &e),
        }
    })
}
unsafe extern "system" fn file_read_v(this: *mut c_void, buffers: *mut BmdIoVec, buffer_count: u32, file_offset: u64, bytes_read: *mut u64) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::ReadV", E_UNEXPECTED, move || {
        if bytes_read.is_null() { return E_POINTER; }
        // The count is stored on every path, failures included, as Blackmagic's
        // own implementation does: it tells the SDK how far a failed read got.
        unsafe { *bytes_read = 0; }
        let Some(vecs) = (unsafe { io_vecs(buffers, buffer_count) }) else { return E_POINTER };
        let (read, hr) = unsafe { read_vectored(&*file_object(this).file, vecs, file_offset) };
        unsafe { *bytes_read = read; }
        hr
    })
}
unsafe extern "system" fn file_write_v(this: *mut c_void, buffers: *mut BmdIoVec, buffer_count: u32, file_offset: u64, bytes_written: *mut u64) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::WriteV", E_UNEXPECTED, move || {
        if bytes_written.is_null() { return E_POINTER; }
        // As in `ReadV`.
        unsafe { *bytes_written = 0; }
        let Some(vecs) = (unsafe { io_vecs(buffers, buffer_count) }) else { return E_POINTER };
        let (written, hr) = unsafe { write_vectored(&*file_object(this).file, vecs, file_offset) };
        unsafe { *bytes_written = written; }
        hr
    })
}

/// The buffer list of a `ReadV` / `WriteV` call; `None` if it is missing.
///
/// # Safety
/// `buffers` must be null or point to `count` `BmdIoVec`s that outlive `'a`.
unsafe fn io_vecs<'a>(buffers: *mut BmdIoVec, count: u32) -> Option<&'a [BmdIoVec]> {
    match count {
        0 => Some(&[]),
        _ if buffers.is_null() => None,
        _ => Some(unsafe { std::slice::from_raw_parts(buffers, count as usize) }),
    }
}

/// `n` bytes reported for a buffer with room for `room`: a broken [`BrawFile`],
/// whose count must not reach the SDK, which sizes its copies by it.
fn overrun(op: &str, n: usize, room: usize) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{op} reported {n} bytes for a {room}-byte buffer"))
}

/// Fill `vecs` in order from `offset` until they are full or the file ends.
/// Returns the bytes read — up to the failure, if one stops it — and the status.
///
/// # Safety
/// Each buffer must be null or valid for writes of `iov_len` bytes.
unsafe fn read_vectored(file: &dyn BrawFile, vecs: &[BmdIoVec], offset: u64) -> (u64, HRESULT) {
    let mut total = 0u64;
    for v in vecs {
        let Ok(len) = usize::try_from(v.iov_len) else { return (total, E_INVALIDARG) };
        if len == 0 { continue; }
        if v.iov_base.is_null() { return (total, E_POINTER); }
        let buf = unsafe { std::slice::from_raw_parts_mut(v.iov_base.cast::<u8>(), len) };
        let mut filled = 0;
        while filled < len {
            let Some(at) = offset.checked_add(total) else { return (total, E_INVALIDARG) };
            match file.read_at(at, &mut buf[filled..]) {
                Ok(0) => return (total, S_OK), // end of file
                Ok(n) if n > len - filled => return (total, hresult_from_io("read", file.name(), &overrun("read_at", n, len - filled))),
                Ok(n) => { filled += n; total += n as u64; }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return (total, hresult_from_io("read", file.name(), &e)),
            }
        }
    }
    (total, S_OK)
}

/// Write `vecs` in order from `offset`. Returns the bytes written — up to the
/// failure, if one stops it — and the status.
///
/// # Safety
/// Each buffer must be null or valid for reads of `iov_len` bytes.
unsafe fn write_vectored(file: &dyn BrawFile, vecs: &[BmdIoVec], offset: u64) -> (u64, HRESULT) {
    let mut total = 0u64;
    for v in vecs {
        let Ok(len) = usize::try_from(v.iov_len) else { return (total, E_INVALIDARG) };
        if len == 0 { continue; }
        if v.iov_base.is_null() { return (total, E_POINTER); }
        let buf = unsafe { std::slice::from_raw_parts(v.iov_base.cast::<u8>(), len) };
        let mut written = 0;
        while written < len {
            let Some(at) = offset.checked_add(total) else { return (total, E_INVALIDARG) };
            match file.write_at(at, &buf[written..]) {
                Ok(0) => return (total, hresult_from_io("write", file.name(), &io::ErrorKind::WriteZero.into())),
                Ok(n) if n > len - written => return (total, hresult_from_io("write", file.name(), &overrun("write_at", n, len - written))),
                Ok(n) => { written += n; total += n as u64; }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return (total, hresult_from_io("write", file.name(), &e)),
            }
        }
    }
    (total, S_OK)
}
unsafe extern "system" fn file_get_preferred_io_alignment(this: *mut c_void, alignment: *mut u32) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::GetPreferredIoAlignment", E_UNEXPECTED, move || unsafe {
        if alignment.is_null() { return E_POINTER; }
        *alignment = file_object(this).file.preferred_io_alignment().max(1);
        S_OK
    })
}
unsafe extern "system" fn file_flush_writes(this: *mut c_void) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::FlushWrites", E_UNEXPECTED, move || {
        let file = &unsafe { file_object(this) }.file;
        match file.flush() {
            Ok(()) => S_OK,
            Err(e) => hresult_from_io("flush", file.name(), &e),
        }
    })
}
unsafe extern "system" fn file_commit_file(this: *mut c_void) -> HRESULT {
    ffi_guard("IBlackmagicRawFile::CommitFile", E_UNEXPECTED, move || {
        let file = &unsafe { file_object(this) }.file;
        match file.commit() {
            Ok(()) => S_OK,
            Err(e) => hresult_from_io("commit", file.name(), &e),
        }
    })
}

// ── IBlackmagicRawFilesystem ──

static FILESYSTEM_VTBL: IBlackmagicRawFilesystemVTbl = IBlackmagicRawFilesystemVTbl {
    parent: ComObject::<FilesystemObject>::IUNKNOWN,
    OpenCompanionFile: filesystem_open_companion_file,
    CreateCompanionFile: filesystem_create_companion_file,
};

/// # Safety
/// `this` must be a live [`FilesystemObject`] (see [`file_object`]).
unsafe fn filesystem_object<'a>(this: *mut c_void) -> &'a FilesystemObject { unsafe { ComObject::state(this) } }

/// Shared body of `OpenCompanionFile` / `CreateCompanionFile`: resolve the parent,
/// run `resolve`, and hand the SDK a new file object bound to this filesystem.
unsafe fn companion_file(
    op: &str,
    this: *mut c_void,
    parent_file: *mut IBlackmagicRawFile,
    file_name: *const c_void,
    file_out: *mut *mut IBlackmagicRawFile,
    resolve: impl FnOnce(&dyn BrawFilesystem, &dyn BrawFile, &str) -> io::Result<Arc<dyn BrawFile>>,
) -> HRESULT {
    if file_out.is_null() { return E_POINTER; }
    unsafe { *file_out = std::ptr::null_mut(); }
    // The SDK only ever names a parent it obtained from us.
    let Some(parent) = (unsafe { ComObject::<FileObject>::downcast(parent_file) }) else { return E_INVALIDARG };
    let name = unsafe { read_sdk_string(file_name) };
    let filesystem = unsafe { filesystem_object(this) };
    match resolve(&*filesystem.filesystem, &*parent.file, &name) {
        Ok(file) => {
            // The new file holds its own reference to this filesystem; the SDK
            // receives the file's.
            let filesystem = unsafe { ComObject::<FilesystemObject>::new_ref(this) };
            unsafe { *file_out = FileObject::create(file, filesystem).into_raw(); }
            S_OK
        }
        Err(e) => hresult_from_io(op, &name, &e),
    }
}

unsafe extern "system" fn filesystem_open_companion_file(this: *mut c_void, parent_file: *mut IBlackmagicRawFile, file_name: *const c_void, file_out: *mut *mut IBlackmagicRawFile) -> HRESULT {
    ffi_guard("IBlackmagicRawFilesystem::OpenCompanionFile", E_UNEXPECTED, move || unsafe {
        companion_file("open companion", this, parent_file, file_name, file_out, |fs, parent, name| fs.open_companion(parent, name))
    })
}
unsafe extern "system" fn filesystem_create_companion_file(this: *mut c_void, parent_file: *mut IBlackmagicRawFile, file_name: *const c_void, replace_existing: SdkBool, file_out: *mut *mut IBlackmagicRawFile) -> HRESULT {
    ffi_guard("IBlackmagicRawFilesystem::CreateCompanionFile", E_UNEXPECTED, move || unsafe {
        let replace_existing = sdk_bool(replace_existing);
        companion_file("create companion", this, parent_file, file_name, file_out, |fs, parent, name| fs.create_companion(parent, name, replace_existing))
    })
}

#[cfg(test)]
mod tests {
    //! The COM objects driven through their vtables, as the SDK drives them. No SDK
    //! library is needed, so these also run under Miri.

    use super::*;
    use std::sync::atomic::{ AtomicUsize, Ordering };

    /// A file over `data` whose reads and writes fail from `fail_at` on, and which
    /// counts its drops.
    struct Probe {
        data: RwLock<Vec<u8>>,
        fail_at: u64,
        dropped: Arc<AtomicUsize>,
    }
    impl Probe {
        fn new(data: Vec<u8>, fail_at: u64, dropped: &Arc<AtomicUsize>) -> Arc<Self> {
            Arc::new(Self { data: RwLock::new(data), fail_at, dropped: dropped.clone() })
        }
        /// How many bytes of a `len`-byte transfer at `offset` happen before the failure point.
        fn allowed(&self, offset: u64, len: usize) -> io::Result<usize> {
            match self.fail_at.checked_sub(offset) {
                None | Some(0) => Err(io::Error::other("injected failure")),
                Some(n) => Ok(len.min(usize::try_from(n).unwrap_or(usize::MAX))),
            }
        }
    }
    impl BrawFile for Probe {
        fn name(&self) -> &str { "probe.braw" }
        fn length(&self) -> io::Result<u64> { Ok(self.data.read().unwrap().len() as u64) }
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
            let n = self.allowed(offset, buf.len())?;
            Ok(copy_from(&self.data.read().unwrap(), offset, &mut buf[..n]))
        }
        fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<usize> {
            let n = self.allowed(offset, buf.len())?;
            let (start, end) = (offset as usize, offset as usize + n);
            let mut data = self.data.write().unwrap();
            if data.len() < end { data.resize(end, 0); }
            data[start..end].copy_from_slice(&buf[..n]);
            Ok(n)
        }
    }
    impl Drop for Probe {
        fn drop(&mut self) { self.dropped.fetch_add(1, Ordering::SeqCst); }
    }

    /// A filesystem serving `companion` under any name, counting its drops.
    struct ProbeFilesystem {
        companion: Arc<dyn BrawFile>,
        dropped: Arc<AtomicUsize>,
    }
    impl BrawFilesystem for ProbeFilesystem {
        fn open_companion(&self, _parent: &dyn BrawFile, _name: &str) -> io::Result<Arc<dyn BrawFile>> {
            Ok(self.companion.clone())
        }
    }
    impl Drop for ProbeFilesystem {
        fn drop(&mut self) { self.dropped.fetch_add(1, Ordering::SeqCst); }
    }

    #[cfg(target_os = "windows")]
    fn riid(iid: &'static GUID) -> QueryInterfaceRiid { iid }
    #[cfg(not(target_os = "windows"))]
    fn riid(iid: &'static GUID) -> QueryInterfaceRiid { *iid }

    /// Release one reference through the object's own vtable, as the SDK does.
    unsafe fn release(object: *mut c_void) -> ComUlong {
        unsafe { ((**(object as *mut *const IUnknownVTbl)).Release)(object) }
    }

    fn io_vecs(buffers: &mut [Vec<u8>]) -> Vec<BmdIoVec> {
        buffers.iter_mut().map(|b| BmdIoVec { iov_base: b.as_mut_ptr().cast(), iov_len: b.len() as u64 }).collect()
    }

    #[test]
    fn references_from_every_path_are_balanced_and_the_last_release_frees_once() {
        let file_drops = Arc::new(AtomicUsize::new(0));
        let fs_drops = Arc::new(AtomicUsize::new(0));
        let file = BlackmagicRawFile::new(
            Probe::new(vec![1, 2, 3], u64::MAX, &file_drops),
            Arc::new(ProbeFilesystem { companion: Probe::new(vec![4], u64::MAX, &file_drops), dropped: fs_drops.clone() }),
        );
        let copy = file.clone();
        let raw = file.as_raw();

        // QueryInterface hands out references for IUnknown and the file interface only.
        let mut unknown = std::ptr::null_mut();
        let mut as_file = std::ptr::null_mut();
        let mut other = std::ptr::NonNull::<c_void>::dangling().as_ptr();
        unsafe {
            let qi = (*(*raw).vtbl).parent.QueryInterface;
            assert_eq!(qi(raw.cast(), riid(&IID_IUNKNOWN), &mut unknown), S_OK);
            assert_eq!(qi(raw.cast(), riid(&IID_IBlackmagicRawFile), &mut as_file), S_OK);
            assert_eq!(qi(raw.cast(), riid(&IID_IBlackmagicRawCallback), &mut other), E_NOINTERFACE);
        }
        assert!(unknown == raw.cast() && as_file == raw.cast() && other.is_null());

        // GetFilesystem hands out a reference to the filesystem; a companion opened
        // through it holds another.
        let mut filesystem = std::ptr::null_mut();
        file.raw.GetFilesystem(&mut filesystem).unwrap();
        let mut companion = std::ptr::null_mut();
        unsafe {
            let open = (*(*filesystem).vtbl).OpenCompanionFile;
            assert_eq!(open(filesystem.cast(), raw, std::ptr::null(), &mut companion), S_OK);
        }
        assert!(!companion.is_null());

        drop(copy);
        unsafe {
            release(unknown);
            release(as_file);
            release(filesystem.cast());
        }
        drop(file);
        assert_eq!(file_drops.load(Ordering::SeqCst), 1, "the clip file goes with its last reference");
        assert_eq!(fs_drops.load(Ordering::SeqCst), 0, "the companion still holds the filesystem");

        assert_eq!(unsafe { release(companion.cast()) }, 0);
        assert_eq!(file_drops.load(Ordering::SeqCst), 2);
        assert_eq!(fs_drops.load(Ordering::SeqCst), 1, "the filesystem goes with the last file bound to it");
    }

    #[test]
    fn read_v_fills_buffers_in_order_and_stops_at_end_of_file() {
        let drops = Arc::new(AtomicUsize::new(0));
        let file = BlackmagicRawFile::standalone(Probe::new((0..10).collect(), u64::MAX, &drops));
        let mut buffers = vec![vec![0u8; 4], vec![0u8; 0], vec![0u8; 8]];
        let mut vecs = io_vecs(&mut buffers);
        let mut read = u64::MAX;
        file.raw.ReadV(vecs.as_mut_ptr(), vecs.len() as u32, 1, &mut read).unwrap();
        assert_eq!(read, 9);
        assert_eq!(buffers, [vec![1, 2, 3, 4], vec![], vec![5, 6, 7, 8, 9, 0, 0, 0]]);
    }

    #[test]
    fn read_v_reports_partial_progress_when_a_read_fails() {
        let drops = Arc::new(AtomicUsize::new(0));
        let file = BlackmagicRawFile::standalone(Probe::new((0..16).collect(), 6, &drops));
        let mut buffers = vec![vec![0u8; 4], vec![0u8; 4]];
        let mut vecs = io_vecs(&mut buffers);
        let mut read = u64::MAX;
        assert!(file.raw.ReadV(vecs.as_mut_ptr(), vecs.len() as u32, 0, &mut read).is_err());
        assert_eq!(read, 6, "the count covers the bytes read before the failure");
        assert_eq!(buffers, [vec![0, 1, 2, 3], vec![4, 5, 0, 0]]);
    }

    #[test]
    fn write_v_reports_partial_progress_when_a_write_fails() {
        let drops = Arc::new(AtomicUsize::new(0));
        let probe = Probe::new(Vec::new(), 6, &drops);
        let file = BlackmagicRawFile::standalone(probe.clone());
        let mut buffers = vec![vec![1u8; 4], vec![2u8; 4]];
        let mut vecs = io_vecs(&mut buffers);
        let mut written = u64::MAX;
        assert!(file.raw.WriteV(vecs.as_mut_ptr(), vecs.len() as u32, 0, &mut written).is_err());
        assert_eq!(written, 6, "the count covers the bytes written before the failure");
        assert_eq!(*probe.data.read().unwrap(), [1, 1, 1, 1, 2, 2]);
    }

    #[test]
    fn an_io_error_reported_to_the_sdk_maps_back_to_its_kind() {
        use io::ErrorKind::*;
        let back = |kind: io::ErrorKind| BrawError::from(hresult_from_io("test", "probe.braw", &kind.into()));
        assert!(matches!(back(Unsupported), BrawError::NotImplemented));
        assert!(matches!(back(InvalidInput), BrawError::InvalidArgument));
        assert!(matches!(back(PermissionDenied), BrawError::AccessDenied));
        assert!(matches!(back(OutOfMemory), BrawError::OutOfMemory));
        assert!(matches!(back(Other), BrawError::Fail));
        // Only Windows' COM error set has codes for these; elsewhere they are `E_FAIL`.
        #[cfg(target_os = "windows")]
        for kind in [NotFound, AlreadyExists] {
            assert!(matches!(back(kind), BrawError::Io(ref e) if e.kind() == kind), "{kind:?}");
        }
        #[cfg(not(target_os = "windows"))]
        for kind in [NotFound, AlreadyExists] {
            assert!(matches!(back(kind), BrawError::Fail), "{kind:?}");
        }
    }

    /// Reports more bytes than it was given room for.
    struct Overreporting;
    impl BrawFile for Overreporting {
        fn name(&self) -> &str { "overreporting.braw" }
        fn length(&self) -> io::Result<u64> { Ok(1 << 20) }
        fn read_at(&self, _offset: u64, buf: &mut [u8]) -> io::Result<usize> { Ok(buf.len() + 1) }
        fn write_at(&self, _offset: u64, buf: &[u8]) -> io::Result<usize> { Ok(buf.len() + 1) }
    }

    #[test]
    fn an_overreported_count_never_reaches_the_sdk() {
        let file = BlackmagicRawFile::standalone(Arc::new(Overreporting));
        let mut buffers = vec![vec![0u8; 8]];
        let mut vecs = io_vecs(&mut buffers);
        let mut count = u64::MAX;
        assert!(file.raw.ReadV(vecs.as_mut_ptr(), 1, 0, &mut count).is_err());
        assert_eq!(count, 0);
        count = u64::MAX;
        assert!(file.raw.WriteV(vecs.as_mut_ptr(), 1, 0, &mut count).is_err());
        assert_eq!(count, 0);
    }

    #[test]
    fn read_v_reports_zero_for_a_malformed_buffer_list() {
        let drops = Arc::new(AtomicUsize::new(0));
        let file = BlackmagicRawFile::standalone(Probe::new(vec![0; 4], u64::MAX, &drops));
        let mut vecs = [BmdIoVec { iov_base: std::ptr::null_mut(), iov_len: 4 }];
        let mut read = u64::MAX;
        assert!(file.raw.ReadV(vecs.as_mut_ptr(), 1, 0, &mut read).is_err());
        assert_eq!(read, 0);
    }
}
