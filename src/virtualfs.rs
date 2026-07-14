// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Virtual-file decoding via [`hookfs`]: open `.braw` clips from arbitrary
//! `Read + Seek` streams (in-memory, network, encrypted, …) instead of on-disk
//! paths.
//!
//! The Blackmagic RAW API exposes clip access **only** by path string, with no
//! `IStream`/open-from-buffer hook. `hookfs` bridges that gap by intercepting the
//! Win32 file calls the SDK issues after being handed a synthetic path, servicing
//! them from the mounted stream (see `impl-hookfs.md` §10).
//!
//! ```no_run
//! # use braw::*;
//! # fn demo() -> Result<(), BrawError> {
//! let braw = Factory::load_from(default_library_name())?;
//! let codec = braw.create_codec()?;
//! let clip = codec.open_clip_from("A001.braw", std::io::Cursor::new(std::fs::read("A001.braw").unwrap()))?;
//! # let _ = clip;
//! # Ok(())
//! # }
//! ```

use super::{BlackmagicRaw, BlackmagicRawClip, BrawError, Factory, RawLibrary};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// A process-monotonic per-clip mount segment (`clip-<hex>`).
///
/// Every [`VirtualClip`] mounts its siblings under `<reserved-root>/<mount-id>/…`,
/// so two clips opened with the **same** `logical_name` — e.g. two cards' shared
/// `A001.braw`, or two independent opens of `clip.braw` — can never map to the
/// same key in the shared, process-global mount context (`impl-hookfs.md` §8: a
/// mount is its own small directory tree under a unique mount id). Combined with
/// the install's process-random reserved root, the segment is collision-proof; a
/// monotonic counter (not randomness) is sufficient and deterministic within a
/// process, and the clip's siblings (`.sidecar`, multicard parts) stay grouped in
/// one directory the SDK sees as an ordinary, stable path.
fn next_mount_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("clip-{:016x}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// The process-wide active installation. Hooks are installed **once** and kept
/// for the process lifetime (the simplest, safest lifecycle — `impl-hookfs.md`
/// §6.4 R12): a single mount context grows and shrinks as clips come and go,
/// while the import patches stay put. Restoration is deferred to a future phase.
struct Active {
    fs: hookfs::Hookfs,
    _install: hookfs::InstallGuard,
    /// The exact SDK image the import patches were written into, pinned for the
    /// process lifetime. Install-for-lifetime (R12) never restores those slots, so
    /// it relies on the hooked image staying mapped: if the only `Factory`
    /// referencing the SDK were dropped, `dlclose`/`FreeLibrary` would unload the
    /// image and free its patched slots, yet `active()` would still report
    /// "installed". A later `Factory::load_from` would then reload the SDK into a
    /// fresh, **un-patched** image while `ensure_installed` short-circuits on the
    /// stale `Active` — silently un-virtualizing the SDK's file I/O. Holding an
    /// owning reference keeps the image mapped (and thus patched) so every later
    /// `Factory::load_from` re-attaches (`dlopen` returns the same, already-hooked
    /// image) rather than reloading an un-hooked one.
    _pinned_lib: Arc<RawLibrary>,
}

fn active() -> &'static Mutex<Option<Active>> {
    static ACTIVE: OnceLock<Mutex<Option<Active>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// Ensure the SDK's file calls are hooked, returning the shared mount context.
///
/// Installs on first use, discovering the SDK module by the **address** of
/// `CreateBlackmagicRawFactoryInstance` (never a basename), and enabling
/// `auto_rescan` so late-loaded decoder plugins are patched too.
///
/// The synthetic volume is installed **writable** (`allow_writes`, Phase 6): the SDK
/// only ever writes when the application explicitly saves a sidecar or trims a clip,
/// and those writes are serviced entirely from the in-memory VFS — a synthetic path
/// still never touches disk. Read-only clips are unaffected.
fn ensure_installed(factory: &Factory) -> Result<hookfs::Hookfs, BrawError> {
    let mut guard = active().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(existing) = guard.as_ref() {
        return Ok(existing.fs.clone());
    }
    let address = factory.lib.factory_instance_address()?;
    let fs = hookfs::Hookfs::new();
    let install = fs
        .install(hookfs::Options::for_module(address).auto_rescan(true).allow_writes(true))
        .map_err(|e| BrawError::Other(format!("hookfs install failed: {e}")))?;
    *guard = Some(Active { fs: fs.clone(), _install: install, _pinned_lib: factory.lib.clone() });
    Ok(fs)
}

/// A handle confirming the SDK's file I/O is virtualized. Held to document intent
/// and to reach the reserved virtual directory; dropping it does **not** uninstall
/// the hooks (they persist for the process lifetime — see [`Active`]).
pub struct HookFsGuard {
    fs: hookfs::Hookfs,
}

impl HookFsGuard {
    /// The reserved synthetic directory clips are mounted under.
    #[must_use]
    pub fn virtual_dir(&self) -> &Path {
        self.fs.virtual_dir()
    }

    /// The synthetic path a `logical_name` maps to.
    #[must_use]
    pub fn path_for(&self, logical_name: &str) -> PathBuf {
        self.fs.path_for(logical_name)
    }

    /// A byte-for-byte copy of the current contents of the writable virtual file at
    /// synthetic `path` (e.g. a sidecar the SDK just saved), or `None` if it is not a
    /// writable in-memory file. Read back without touching disk (Phase 6).
    #[must_use]
    pub fn read_virtual_path(&self, path: &Path) -> Option<Vec<u8>> {
        self.fs.read_virtual_path(path)
    }
}

impl std::fmt::Debug for HookFsGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookFsGuard").field("virtual_dir", &self.fs.virtual_dir()).finish()
    }
}

impl Factory {
    /// Install the `hookfs` file-I/O hooks so clips can be opened from memory /
    /// custom streams. Idempotent; safe to call before the first `OpenClip`.
    ///
    /// # Errors
    /// Fails if the SDK export address can't be resolved or the hooks can't be
    /// installed into the SDK module.
    pub fn enable_virtual_files(&self) -> Result<HookFsGuard, BrawError> {
        let fs = ensure_installed(self)?;
        Ok(HookFsGuard { fs })
    }
}

/// A clip opened from virtual streams. Owns the [`BlackmagicRawClip`] **and** the
/// mount guards, so the synthetic path and its backing streams live as long as the
/// clip can spawn async read jobs (`impl-hookfs.md` §10). Dereferences to the
/// underlying clip, so all clip methods are available directly.
pub struct VirtualClip {
    // Field order matters: `clip` drops first (stopping any SDK read jobs), then
    // the mounts are released.
    clip: BlackmagicRawClip,
    _mounts: Vec<hookfs::MountGuard>,
    fs: hookfs::Hookfs,
    primary: PathBuf,
}

impl VirtualClip {
    /// Borrow the underlying clip.
    #[must_use]
    pub fn clip(&self) -> &BlackmagicRawClip {
        &self.clip
    }

    /// The synthetic path the primary clip was opened at.
    #[must_use]
    pub fn primary_path(&self) -> &Path {
        &self.primary
    }

    /// The synthetic path of the clip's sidecar (same directory + `.sidecar`), the
    /// path `SaveSidecarFile` writes to.
    #[must_use]
    pub fn sidecar_path(&self) -> PathBuf {
        self.primary.with_extension("sidecar")
    }

    /// The shared mount context, e.g. to read back a written sidecar/output
    /// ([`hookfs::Hookfs::read_virtual_path`]) or derive an output path.
    #[must_use]
    pub fn hookfs(&self) -> &hookfs::Hookfs {
        &self.fs
    }

    /// A byte-for-byte copy of the writable virtual file at synthetic `path` (the
    /// sidecar the SDK saved, a trim output), or `None`. Never touches disk.
    #[must_use]
    pub fn read_virtual_path(&self, path: &Path) -> Option<Vec<u8>> {
        self.fs.read_virtual_path(path)
    }

    /// Split into the underlying [`BlackmagicRawClip`] and an opaque
    /// [`VirtualMounts`] keep-alive, so a caller that stores the clip in its own
    /// field can hold the mounts separately.
    ///
    /// The caller **must** drop the returned clip before the [`VirtualMounts`]:
    /// the mounts back the synthetic path the clip's (possibly async) read jobs
    /// pull from — the same drop-order contract [`VirtualClip`] enforces via its
    /// field order.
    #[must_use]
    pub fn into_parts(self) -> (BlackmagicRawClip, VirtualMounts) {
        (self.clip, VirtualMounts { _mounts: self._mounts, _fs: self.fs })
    }
}

/// Opaque keep-alive for a [`VirtualClip`]'s mounts + hook context, returned by
/// [`VirtualClip::into_parts`]. Hold it for at least as long as the extracted
/// [`BlackmagicRawClip`] is used and drop it only **after** the clip is dropped.
pub struct VirtualMounts {
    _mounts: Vec<hookfs::MountGuard>,
    _fs: hookfs::Hookfs,
}

impl std::ops::Deref for VirtualClip {
    type Target = BlackmagicRawClip;
    fn deref(&self) -> &Self::Target {
        &self.clip
    }
}

/// Builder for a clip backed by one or more virtual sibling files (the clip, its
/// `.sidecar`, and any multicard parts). Mount every sibling *before*
/// [`open`](Self::open), because the SDK reads the `.sidecar` during `OpenClip`.
///
/// Every sibling is mounted beneath this builder's own [`next_mount_id`]
/// sub-directory, so a clip is fully isolated from every other clip in the shared
/// process-global mount context even when they use identical `logical_name`s.
pub struct VirtualClipBuilder<'a> {
    codec: &'a BlackmagicRaw,
    fs: hookfs::Hookfs,
    /// Unique per-clip directory segment scoping every sibling of this clip.
    mount_id: String,
    primary: String,
    mounts: Vec<hookfs::MountGuard>,
}

impl VirtualClipBuilder<'_> {
    /// Scope a caller-facing `logical_name` into this clip's unique sub-directory.
    fn scoped(&self, logical_name: &str) -> String {
        format!("{}/{}", self.mount_id, logical_name)
    }

    /// Mount a sibling stream under `logical_name` (within this clip's namespace).
    ///
    /// # Errors
    /// Fails if the name is invalid or the stream size can't be probed.
    pub fn file<R: Read + Seek + Send + 'static>(
        mut self,
        logical_name: &str,
        reader: R,
    ) -> Result<Self, BrawError> {
        let guard = self
            .fs
            .mount(&self.scoped(logical_name), reader)
            .map_err(|e| BrawError::Other(format!("mount `{logical_name}`: {e}")))?;
        self.mounts.push(guard);
        Ok(self)
    }

    /// Reserve a **writable** in-memory sibling under `logical_name` (Phase 6): an
    /// empty file the SDK can write (a sidecar to re-save, a trim output) and that is
    /// read back through the same synthetic path — never touching disk. Requires the
    /// hooks to have been installed with `allow_writes` (they always are, via
    /// [`ensure_installed`]).
    ///
    /// # Errors
    /// Fails if the name is invalid.
    pub fn writable(mut self, logical_name: &str) -> Result<Self, BrawError> {
        let guard = self
            .fs
            .mount_writable(&self.scoped(logical_name))
            .map_err(|e| BrawError::Other(format!("mount_writable `{logical_name}`: {e}")))?;
        self.mounts.push(guard);
        Ok(self)
    }

    /// The synthetic path a scoped `logical_name` maps to within this clip's
    /// namespace (e.g. to pass a virtual output path to `CreateJobTrim`).
    #[must_use]
    pub fn scoped_path(&self, logical_name: &str) -> PathBuf {
        self.fs.path_for(&self.scoped(logical_name))
    }

    /// The synthetic path the primary clip will be opened at.
    #[must_use]
    pub fn primary_path(&self) -> PathBuf {
        self.fs.path_for(&self.scoped(&self.primary))
    }

    /// Open the primary clip at its synthetic path.
    ///
    /// # Errors
    /// Propagates any `OpenClip` failure.
    pub fn open(self) -> Result<VirtualClip, BrawError> {
        let path = self.fs.path_for(&self.scoped(&self.primary));
        let path_str = path.to_str().ok_or(BrawError::InvalidArgument)?;
        let clip = self.codec.open_clip(path_str)?;
        Ok(VirtualClip { clip, _mounts: self.mounts, fs: self.fs, primary: path })
    }
}

impl BlackmagicRaw {
    /// Begin building a clip backed by virtual sibling files. Installs the hooks on
    /// first use.
    ///
    /// # Errors
    /// Fails if the hooks can't be installed.
    pub fn virtual_clip(&self, primary_name: &str) -> Result<VirtualClipBuilder<'_>, BrawError> {
        let fs = ensure_installed(&self.factory)?;
        Ok(VirtualClipBuilder {
            codec: self,
            fs,
            mount_id: next_mount_id(),
            primary: primary_name.to_owned(),
            mounts: Vec::new(),
        })
    }

    /// Open a clip from a single in-memory / custom `Read + Seek` stream mounted
    /// under `logical_name` (`impl-hookfs.md` §5.4/§10). For a clip with a sidecar
    /// or spanned segments, use [`virtual_clip`](Self::virtual_clip) and add each
    /// sibling with [`VirtualClipBuilder::file`].
    ///
    /// # Errors
    /// Fails if the hooks can't be installed, the stream can't be mounted, or
    /// `OpenClip` fails.
    pub fn open_clip_from<R: Read + Seek + Send + 'static>(
        &self,
        logical_name: &str,
        reader: R,
    ) -> Result<VirtualClip, BrawError> {
        self.virtual_clip(logical_name)?.file(logical_name, reader)?.open()
    }
}
