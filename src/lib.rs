// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_snake_case)]

#![doc = include_str!("../README.md")]

use core::ffi::c_void;
use std::mem::ManuallyDrop;
use std::sync::{ Arc, Mutex, PoisonError };

#[macro_use]
mod com;       pub use com::*;
mod bitstream; pub use bitstream::*;
mod callback;  pub use callback::*;
mod error;     pub use error::*;
mod file;      pub use file::*;
mod future;    pub use future::*;
mod iterators; pub use iterators::*;
mod os;        pub use os::*;
mod sdk;       pub use sdk::*;
mod string;    pub use string::*;
mod variant;   pub use variant::*;

#[cfg(target_os = "windows")]
use libloading::os::windows as dl;
#[cfg(not(target_os = "windows"))]
use libloading::os::unix as dl;

/// The loaded Blackmagic RAW SDK library and the entry points resolved from it.
///
/// The library is never unloaded: the SDK's worker threads may still be running
/// its code after every object is gone.
#[allow(dead_code)]
pub struct RawLibrary {
    #[cfg(not(target_os = "windows"))] VariantInit:           dl::Symbol<VariantInitFn>,
    #[cfg(not(target_os = "windows"))] VariantClear:          dl::Symbol<VariantClearFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayCreate:       dl::Symbol<SafeArrayCreateFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayGetVartype:   dl::Symbol<SafeArrayGetVartypeFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayGetLBound:    dl::Symbol<SafeArrayGetLBoundFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayGetUBound:    dl::Symbol<SafeArrayGetUBoundFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayAccessData:   dl::Symbol<SafeArrayAccessDataFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayUnaccessData: dl::Symbol<SafeArrayUnaccessDataFn>,
    #[cfg(not(target_os = "windows"))] SafeArrayDestroy:      dl::Symbol<SafeArrayDestroyFn>,

    // Never unloaded: the SDK's worker threads may still be running its code — a job
    // completing, a codec being destroyed — after every Rust object is gone.
    lib: ManuallyDrop<dl::Library>,
}

impl RawLibrary {
    /// Load the BRAW shared library, for the rest of the process.
    pub fn load<P: libloading::AsFilename>(path: P) -> Result<Self, libloading::Error> {
        unsafe {
            let lib = dl::Library::new(path)?;
            Ok(Self {
                #[cfg(not(target_os = "windows"))] VariantInit:           lib.get(b"VariantInit\0")?,
                #[cfg(not(target_os = "windows"))] VariantClear:          lib.get(b"VariantClear\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayCreate:       lib.get(b"SafeArrayCreate\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayGetVartype:   lib.get(b"SafeArrayGetVartype\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayGetLBound:    lib.get(b"SafeArrayGetLBound\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayGetUBound:    lib.get(b"SafeArrayGetUBound\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayAccessData:   lib.get(b"SafeArrayAccessData\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayUnaccessData: lib.get(b"SafeArrayUnaccessData\0")?,
                #[cfg(not(target_os = "windows"))] SafeArrayDestroy:      lib.get(b"SafeArrayDestroy\0")?,

                lib: ManuallyDrop::new(lib),
            })
        }
    }

    /// Create the SDK's factory object.
    pub fn create_factory(&self) -> Result<ComPtr<IBlackmagicRawFactory>, BrawError> {
        unsafe {
            let create: dl::Symbol<BlackmagicCreateFn> = self.lib.get(b"CreateBlackmagicRawFactoryInstance\0")?;
            ComPtr::new(create())
        }
    }
}

/// Use this to create one or more Codec objects.
///
/// The factory is the entry point for creating Blackmagic RAW codec instances and iterating available processing pipelines.
#[derive(Clone)]
pub struct Factory {
    factory: ComPtr<IBlackmagicRawFactory>,

    // Shared, as every object descending from the factory holds a clone.
    lib: Arc<RawLibrary>
}

impl Factory {
    /// Load the SDK library at `path` — usually [`default_library_name`], resolved
    /// through the platform's library search path — and create its factory.
    pub fn load_from(path: impl AsRef<std::path::Path>) -> Result<Self, BrawError> {
        let lib = RawLibrary::load(path.as_ref())?;
        let factory = lib.create_factory()?;
        Ok(Self {
            lib: Arc::new(lib),
            factory,
        })
    }

    /// Create a codec from the factory
    ///
    /// Fails with [`BrawError::UnsupportedSdkVersion`] when the loaded library does
    /// not implement the Blackmagic RAW SDK 6.0 codec interface these bindings are
    /// built on — an older SDK, or a newer one that changed it again.
    pub fn create_codec(&self) -> Result<BlackmagicRaw, BrawError> {
        // SAFETY: `CreateCodec` returns a new codec through its out-parameter.
        let raw: ComPtr<IBlackmagicRaw> = unsafe { out_interface(|out| self.factory.CreateCodec(out))? };

        // The codec's IID changes whenever its vtable does. A library of another
        // version hands back a codec with a different method layout — calling into
        // it (even the `SetCallback` below) would jump to the wrong slot — so confirm
        // it answers to the 6.0 IID before touching anything else. Only
        // `E_NOINTERFACE` means another version; any other failure is the call's own.
        match raw.query_interface::<IBlackmagicRaw>() {
            Err(BrawError::NoInterface) => return Err(BrawError::UnsupportedSdkVersion(self.camera_support_version(&raw))),
            result => drop(result?),
        }

        let callback = Arc::new(CallbackHandle::new(DefaultCallback::default()));
        let core = Arc::new(CodecCore { codec: ManuallyDrop::new(raw.clone()), dependents: Mutex::new(Vec::new()) });
        let codec = BlackmagicRaw {
            raw,
            factory: self.clone(),
            // Every object descending from the codec, and every job, holds its core
            // (see `CodecCore`) and its callback: the SDK holds references to the
            // callback while it can call it, and so, without relying on it, do they.
            parent_guards: vec![keep_alive(core.clone()), callback.add_ref_and_get_guard()].into(),
            callback,
            core,
        };
        // SAFETY: the callback is a live COM object; the SDK takes its own reference.
        unsafe { codec.raw.SetCallback(codec.callback.as_mut_ptr())? };

        Ok(codec)
    }

    /// Create a pipeline iterator
    pub fn pipeline_iter(&self, interop: BlackmagicRawInterop) -> Result<PipelineIterator, BrawError> {
        // SAFETY: returns a new iterator through its out-parameter.
        let raw = unsafe { out_interface(|out| self.factory.CreatePipelineIterator(interop, out))? };
        Ok(PipelineIterator { raw, factory: self.clone(), is_first: true })
    }
    /// Create a pipeline device iterator
    pub fn pipeline_device_iter(&self, pipeline: BlackmagicRawPipeline, interop: BlackmagicRawInterop) -> Result<PipelineDeviceIterator, BrawError> {
        // SAFETY: returns a new iterator through its out-parameter.
        let raw = unsafe { out_interface(|out| self.factory.CreatePipelineDeviceIterator(pipeline, interop, out))? };
        Ok(PipelineDeviceIterator { raw, factory: self.clone(), is_first: true, current_index: Arc::new(std::sync::atomic::AtomicUsize::new(0)) })
    }
    /// Create empty clip geometry object
    pub fn create_clip_geometry(&self) -> Result<BlackmagicRawClipGeometry, BrawError> {
        // SAFETY: returns a new geometry object through its out-parameter.
        let raw = unsafe { out_interface(|out| self.factory.CreateClipGeometry(out))? };
        Ok(BlackmagicRawClipGeometry { raw, factory: self.clone(), parent_guards: vec![].into() } )
    }

    /// The loaded library's camera support version, read through
    /// `IBlackmagicRawConfiguration` — the one interface whose IID and layout are the
    /// same in every SDK from 4.2 to 6.0, so it is safe to call on a library the
    /// bindings do not otherwise support ("unknown" from one that changed it too).
    /// (`GetVersion` would be the natural choice, but the 5.0 and 6.0 libraries both
    /// answer it with "0.0".)
    fn camera_support_version(&self, codec: &ComPtr<IBlackmagicRaw>) -> String {
        let Ok(configuration) = codec.query_interface::<IBlackmagicRawConfiguration>() else { return "unknown".into() };
        let mut version = std::ptr::null_mut();
        // SAFETY: `version` receives a string the SDK allocates for the caller.
        match unsafe { configuration.GetCameraSupportVersion(&mut version) } {
            // SAFETY: the SDK passed the string's ownership to us.
            Ok(_) => unsafe { take_sdk_string(version) },
            Err(_) => "unknown".into(),
        }
    }
}
unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

impl BlackmagicRaw {
    /// Open the clip at `path`, with its sidecar and any other cards of a
    /// multi-card recording found beside it.
    pub fn open_clip(&self, path: &str) -> Result<BlackmagicRawClip, BrawError> {
        let in_str = BrawString::from(path);
        // SAFETY: a native string, and a new clip through the out-parameter.
        let clip = unsafe { out_interface(|out| self.raw.OpenClip(in_str.as_raw(), out))? };
        Ok(BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }
    /// Open the clip at `path`, with `geometry` applied regardless of the clip's metadata.
    pub fn open_clip_with_geometry(&self, path: &str, geometry: BlackmagicRawClipGeometry) -> Result<BlackmagicRawClip, BrawError> {
        let in_str = BrawString::from(path);
        // SAFETY: a native string, a live geometry object, and a new clip through the
        // out-parameter.
        let clip = unsafe { out_interface(|out| self.raw.OpenClipWithGeometry(in_str.as_raw(), geometry.as_raw(), out))? };
        Ok(BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }
    /// Open a clip read through `file` (see the [`file`](crate::BrawFile) traits).
    /// The clip keeps `file` alive for its whole lifetime.
    pub fn open_clip_from_file(&self, file: &BlackmagicRawFile) -> Result<BlackmagicRawClip, BrawError> {
        // SAFETY: a live file object, which the clip keeps (`clip_from_file`), and a
        // new clip through the out-parameter.
        let clip = unsafe { out_interface(|out| self.raw.OpenClipFromFile(file.as_raw(), out))? };
        Ok(self.clip_from_file(clip, file))
    }
    /// Open a clip read through `file`, with `geometry` applied regardless of the clip's metadata.
    pub fn open_clip_from_file_with_geometry(&self, file: &BlackmagicRawFile, geometry: BlackmagicRawClipGeometry) -> Result<BlackmagicRawClip, BrawError> {
        // SAFETY: as in `open_clip_from_file`, with a live geometry object.
        let clip = unsafe { out_interface(|out| self.raw.OpenClipFromFileWithGeometry(file.as_raw(), geometry.as_raw(), out))? };
        Ok(self.clip_from_file(clip, file))
    }
    fn clip_from_file(&self, clip: ComPtr<IBlackmagicRawClip>, file: &BlackmagicRawFile) -> BlackmagicRawClip {
        // The file is the clip's byte source: every read job the clip (or a frame
        // future cloned from it) can still issue must find it alive.
        let parent_guards = self.parent_guards.clone_and_extend([self.raw.add_ref_and_get_guard(), file.add_ref_and_get_guard()]);
        BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards }
    }

    /// Forward the SDK's notifications for this codec's jobs to `callback`,
    /// replacing any set before. The job futures complete as they otherwise would.
    pub fn set_callback<T: BrawCallback>(&self, callback: T) -> Result<(), BrawError> {
        self.callback.state().set_user_callback(Some(Arc::new(callback)));
        Ok(())
    }

    /// Asynchronously prepare `pipeline` on the GPU context and command queue given,
    /// compiling and binding its kernels ahead of the first decode.
    ///
    /// The preparation starts at once; await the returned future for its outcome, or
    /// see it through [`BrawCallback::prepare_pipeline_complete`].
    ///
    /// # Safety
    /// `pipeline_context` and `pipeline_command_queue` must be null or valid for
    /// `pipeline` (see [`BlackmagicRawConfiguration::set_pipeline`]), and stay valid
    /// until the codec is destroyed — which [`keep_alive`](Self::keep_alive) can
    /// see to. Prefer [`prepare_pipeline_for_device`](Self::prepare_pipeline_for_device),
    /// which upholds this itself.
    pub unsafe fn prepare_pipeline(&self, pipeline: BlackmagicRawPipeline, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void) -> Result<CallbackFuture<()>, BrawError> {
        // SAFETY: the handles are valid, per this function's contract.
        self.prepare(|user_data| unsafe { self.raw.PreparePipeline(pipeline, pipeline_context, pipeline_command_queue, user_data) })
    }

    /// Asynchronously prepare the pipeline of `device`, compiling and binding its
    /// kernels ahead of the first decode. The codec keeps the device alive for as long
    /// as it lives, as the SDK requires.
    ///
    /// The preparation starts at once; await the returned future for its outcome, or
    /// see it through [`BrawCallback::prepare_pipeline_complete`].
    pub fn prepare_pipeline_for_device(&self, device: &BlackmagicRawPipelineDevice) -> Result<CallbackFuture<()>, BrawError> {
        self.core.keep(device.raw.add_ref_and_get_guard());
        // SAFETY: a live device, which the codec keeps alive.
        self.prepare(|user_data| unsafe { self.raw.PreparePipelineForDevice(device.as_raw(), user_data) })
    }

    /// Start a pipeline preparation, handing `start` the user data its completion
    /// callback finds the returned future's state by.
    fn prepare(&self, start: impl FnOnce(*mut c_void) -> Result<HRESULT, BrawError>) -> Result<CallbackFuture<()>, BrawError> {
        let state = Arc::new(State::<()>::new(self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard())));

        // Owned refcount handed to the SDK; reclaimed in
        // `prepare_pipeline_complete` via `Arc::from_raw`. Reclaim here too if
        // the call fails (no callback will fire) so `State` isn't leaked.
        let raw = Arc::into_raw(state.clone()) as *mut c_void;
        if let Err(e) = start(raw) {
            // SAFETY: the `into_raw` above, which the failed call never handed on.
            unsafe { drop(Arc::from_raw(raw as *const State<()>)); }
            return Err(e);
        }
        Ok(CallbackFuture { state, job: None })
    }

    /// Keep `value` alive until the codec has been destroyed: to the end of its
    /// destruction, which can finish on an SDK thread after every Rust reference to
    /// the codec is gone. For whatever owns the GPU context and command queue given
    /// to [`set_pipeline`](BlackmagicRawConfiguration::set_pipeline) or
    /// [`prepare_pipeline`](Self::prepare_pipeline).
    pub fn keep_alive<T: Send + 'static>(&self, value: T) {
        self.core.keep(keep_alive(value));
    }

    /// Get the configuration interface for this codec
    pub fn configuration(&self) -> Result<BlackmagicRawConfiguration, BrawError> {
        Ok(BlackmagicRawConfiguration {
            raw: self.raw.query_interface()?,
            core: self.core.clone(),
            factory: self.factory.clone(),
            parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()),
        })
    }
}

/// The codec's own reference, and what the codec uses without holding it — the
/// devices, or whatever else owns the GPU context and command queue it decodes with
/// — kept alive by everything that descends from the codec, its jobs included.
///
/// The SDK requires those to outlive the codec: through the codec's destruction,
/// which frees the GPU resources it made on the context only after letting go of
/// everything else, and which work the SDK still has in flight can delay past the
/// last Rust reference. So with dependents to keep, the codec is released only once
/// no other reference to it is left — its destruction then running to completion
/// inside that release — and the dependents after it.
pub(crate) struct CodecCore {
    codec: ManuallyDrop<ComPtr<IBlackmagicRaw>>,
    dependents: Mutex<Vec<ComPtrRefGuard>>,
}
// SAFETY: the codec is free-threaded, and the devices are behind a lock.
unsafe impl Send for CodecCore {}
// SAFETY: as above.
unsafe impl Sync for CodecCore {}

impl CodecCore {
    /// Keep what `guard` holds alive until the codec is destroyed.
    fn keep(&self, guard: ComPtrRefGuard) {
        self.dependents.lock().unwrap_or_else(PoisonError::into_inner).push(guard);
    }
}

impl Drop for CodecCore {
    fn drop(&mut self) {
        // SAFETY: not used again.
        let codec = unsafe { ManuallyDrop::take(&mut self.codec) };
        let dependents = std::mem::take(self.dependents.get_mut().unwrap_or_else(PoisonError::into_inner));
        if !dependents.is_empty() {
            // SAFETY: the codec is free-threaded.
            unsafe { release_before(codec, dependents) };
        }
    }
}

impl BlackmagicRawConfiguration {
    /// Decode with `device`'s pipeline, context, command queue and instruction set.
    /// The codec keeps the device alive for as long as it lives, as the SDK requires.
    pub fn set_from_device(&mut self, device: &BlackmagicRawPipelineDevice) -> Result<(), BrawError> {
        self.core.keep(device.raw.add_ref_and_get_guard());
        // SAFETY: a live device, which the codec keeps alive.
        unsafe { self.raw.SetFromDevice(device.as_raw())? };
        Ok(())
    }

    /// Decode with `pipeline`, on the GPU context and command queue given. Changing
    /// the pipeline re-creates the default resource manager.
    ///
    /// # Safety
    /// `pipeline_context` and `pipeline_command_queue` must be null or valid for
    /// `pipeline` — `CUcontext` and `CUstream` for CUDA, `cl_context` and
    /// `cl_command_queue` for OpenCL, null and `MTLCommandQueue` for Metal — and stay
    /// valid until the codec is destroyed, which
    /// [`BlackmagicRaw::keep_alive`] can see to. Prefer
    /// [`set_from_device`](Self::set_from_device), which upholds this itself.
    pub unsafe fn set_pipeline(&mut self, pipeline: BlackmagicRawPipeline, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void) -> Result<(), BrawError> {
        // SAFETY: the handles are valid, per this function's contract.
        unsafe { self.raw.SetPipeline(pipeline, pipeline_context, pipeline_command_queue)? };
        Ok(())
    }
}

// Each method takes the GPU context and command queue of a pipeline, and resources
// created on them: none can check what it is given.
impl BlackmagicRawResourceManager {
    /// Create a resource of `size_bytes` bytes, of type `typ`, for `usage`.
    ///
    /// # Safety
    /// `context` and `command_queue` must be null or valid for the pipeline `typ`
    /// belongs to (see [`BlackmagicRawConfiguration::set_pipeline`]).
    pub unsafe fn create_resource(&self, context: *mut c_void, command_queue: *mut c_void, size_bytes: u32, typ: BlackmagicRawResourceType, usage: BlackmagicRawResourceUsage) -> Result<*mut c_void, BrawError> {
        let mut resource = std::ptr::null_mut();
        // SAFETY: per this function's contract, and an out-parameter on this stack.
        unsafe { self.raw.CreateResource(context, command_queue, size_bytes, typ, usage, &mut resource)? };
        Ok(resource)
    }
    /// Release `resource`.
    ///
    /// # Safety
    /// `resource` must be a resource of type `typ` that this manager created on
    /// `context` and `command_queue`, and is not used afterwards.
    pub unsafe fn release_resource(&self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, typ: BlackmagicRawResourceType) -> Result<(), BrawError> {
        // SAFETY: per this function's contract.
        unsafe { self.raw.ReleaseResource(context, command_queue, resource, typ)? };
        Ok(())
    }
    /// Copy `size_bytes` bytes from `source` to `destination`, asynchronously on the
    /// command queue if `copy_async`.
    ///
    /// # Safety
    /// `source` and `destination` must be resources of types `source_type` and
    /// `destination_type`, each of at least `size_bytes` bytes, valid for
    /// `context` and `command_queue`, which must be valid for their pipeline — until
    /// the copy completes, if it is asynchronous.
    #[allow(clippy::too_many_arguments)] // mirrors the SDK call
    pub unsafe fn copy_resource(&self, context: *mut c_void, command_queue: *mut c_void, source: *mut c_void, source_type: BlackmagicRawResourceType, destination: *mut c_void, destination_type: BlackmagicRawResourceType, size_bytes: u32, copy_async: bool) -> Result<(), BrawError> {
        // SAFETY: per this function's contract.
        unsafe { self.raw.CopyResource(context, command_queue, source, source_type, destination, destination_type, size_bytes, copy_async)? };
        Ok(())
    }
    /// The host-addressable memory of `resource`.
    ///
    /// # Safety
    /// `resource` must be a resource of type `resource_type`, valid for `context` and
    /// `command_queue`, which must be valid for its pipeline.
    pub unsafe fn resource_host_pointer(&self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, resource_type: BlackmagicRawResourceType) -> Result<*mut c_void, BrawError> {
        let mut host_pointer = std::ptr::null_mut();
        // SAFETY: per this function's contract, and an out-parameter on this stack.
        unsafe { self.raw.GetResourceHostPointer(context, command_queue, resource, resource_type, &mut host_pointer)? };
        Ok(host_pointer)
    }
}

impl BlackmagicRawClip {
    /// Returns an iterator over the metadata entries in the clip
    pub fn metadata_iter(&self) -> Result<MetadataIterator, BrawError> {
        // SAFETY: returns a new iterator through its out-parameter.
        let raw = unsafe { out_interface(|out| self.raw.GetMetadataIterator(out))? };
        Ok(MetadataIterator { raw, factory: self.factory.clone(), is_first: true, parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }

    /// Read frame `frame_index` from the file, ready to decode.
    pub async fn read_frame(&self, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(frame_index, &[]).await
    }
    /// As [`read_frame`](Self::read_frame), with hints for the read job.
    pub async fn read_frame_with_hints(&self, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        self.create_read_frame_future(frame_index, hints)?.await
    }

    /// Submit a read-frame job and return a `'static` [`ReadFrameFuture`]
    /// for its completion — the pipeline-friendly form of [`read_frame`](Self::read_frame).
    /// The job is submitted immediately; await (or poll) the future for
    /// the `BlackmagicRawFrame`. Unlike `read_frame` the future borrows
    /// nothing from `self`, so a scheduler can keep many in flight.
    pub fn create_read_frame_future(&self, frame_index: u64, hints: &[ReadJobHints]) -> Result<ReadFrameFuture, BrawError> {
        let mut job = std::ptr::null_mut();
        // SAFETY: `job` receives a new read-frame job, which uses only the clip.
        unsafe {
            self.raw.CreateJobReadFrame(frame_index, &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()))
        }
    }
    /// Read up to `max_sample_count` sample frames of audio, starting at `sample_index`.
    pub async fn read_audio(&self, sample_index: u64, max_sample_count: u64) -> Result<BlackmagicRawAudioBuffer, BrawError> {
        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());
        let mut job = std::ptr::null_mut();
        // SAFETY: `job` receives a new read-audio job, whose completion delivers an
        // audio buffer, and which uses only the clip.
        let future = unsafe {
            self.raw.CreateJobReadAudio(sample_index, max_sample_count, &mut job)?;
            submit(job, &[], parent_guards.clone())?
        };
        Ok(BlackmagicRawAudioBuffer { raw: future.await?, factory: self.factory.clone(), parent_guards })
    }
    /// Write `frame_count` frames from `frame_index` to a new `.braw` at `file_name`,
    /// with the sidecar's settings — or the given processing attributes — baked in.
    pub async fn trim(&self, file_name: &str, frame_index: u64, frame_count: u64, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let path = BrawString::from(file_name);
        let mut job = std::ptr::null_mut();
        // SAFETY: a native string and live attribute objects, all held until the job
        // completes, and a new trim job through the out-parameter.
        let future = unsafe {
            self.raw.CreateJobTrim(path.as_raw(), frame_index, frame_count, clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), &mut job)?;
            submit(job, &[], job_guards(&self.parent_guards, self.raw.add_ref_and_get_guard(), &clip_processing_attributes, &frame_processing_attributes, [keep_alive(path)]))?
        };
        future.await
    }
    /// As [`trim`](Self::trim), writing the trimmed clip through `destination`.
    pub async fn trim_to_file(&self, destination: &BlackmagicRawFile, frame_index: u64, frame_count: u64, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job = std::ptr::null_mut();
        // SAFETY: live file and attribute objects, all held until the job completes,
        // and a new trim job through the out-parameter.
        let future = unsafe {
            self.raw.CreateJobTrimToFile(destination.as_raw(), frame_index, frame_count, clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), &mut job)?;
            submit(job, &[], job_guards(&self.parent_guards, self.raw.add_ref_and_get_guard(), &clip_processing_attributes, &frame_processing_attributes, [destination.add_ref_and_get_guard()]))?
        };
        future.await
    }
}

/// Submit the new `job`, holding `keep_alive` — everything it uses — until it
/// completes.
///
/// # Safety
/// `job` must be null or a new, unsubmitted job whose completion delivers a `T` (see
/// `DefaultCallback`), and `keep_alive` must hold everything the job uses.
unsafe fn submit<T>(job: *mut IBlackmagicRawJob, hints: &[ReadJobHints], keep_alive: DropOrderVec<ComPtrRefGuard>) -> Result<CallbackFuture<T>, BrawError> {
    // SAFETY: per this function's contract.
    unsafe { CallbackFuture::create_from_job(ComPtr::new(job)?, hints, keep_alive) }
}

/// Submit the new read-frame `job`; `parent_guards`, which hold everything the job
/// uses, go to the frame it reads.
///
/// # Safety
/// As for [`submit`], for a read-frame job.
unsafe fn read_frame_job(job: *mut IBlackmagicRawJob, hints: &[ReadJobHints], factory: &Factory, parent_guards: DropOrderVec<ComPtrRefGuard>) -> Result<ReadFrameFuture, BrawError> {
    // SAFETY: per this function's contract.
    let inner = unsafe { submit(job, hints, parent_guards.clone())? };
    Ok(ReadFrameFuture::new(inner, factory.clone(), parent_guards))
}

/// The guards of everything a job uses: the object creating it (`creator`) and what
/// that object holds, the processing attributes it is given, and `extra`.
fn job_guards(
    parent_guards: &DropOrderVec<ComPtrRefGuard>,
    creator: ComPtrRefGuard,
    clip_processing_attributes: &Option<BlackmagicRawClipProcessingAttributes>,
    frame_processing_attributes: &Option<BlackmagicRawFrameProcessingAttributes>,
    extra: impl IntoIterator<Item = ComPtrRefGuard>,
) -> DropOrderVec<ComPtrRefGuard> {
    parent_guards.clone_and_extend(
        std::iter::once(creator)
            .chain(clip_processing_attributes.iter().map(|a| a.raw.add_ref_and_get_guard()))
            .chain(frame_processing_attributes.iter().map(|a| a.raw.add_ref_and_get_guard()))
            .chain(extra)
    )
}

/// A buffer length as the SDK's `uint32_t` byte counts take it. A longer buffer is
/// reported as `u32::MAX` bytes: the SDK then uses less of it than it could.
fn byte_count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

impl BlackmagicRawClipEx {
    /// Read frame `frame_index` into `bit_stream`, a buffer with room for its
    /// [`bit_stream_size_bytes`](Self::bit_stream_size_bytes) —
    /// [`max_bit_stream_size_bytes`](Self::max_bit_stream_size_bytes) has room for any
    /// frame's. The SDK reads straight from the file into the buffer, so it must be
    /// [`BIT_STREAM_ALIGNMENT`]-aligned with a length that is a multiple of it, as a
    /// [`BitStreamBuffer`] is; any other fails with [`BrawError::Fail`].
    ///
    /// The frame is decoded from the buffer, so the buffer is moved into the frame,
    /// and dropped once the frame and every job using it are gone. To reuse buffers,
    /// pass a type that returns its storage to a pool when dropped.
    pub async fn read_frame<B: AsMut<[u8]> + Send + 'static>(&self, frame_index: u64, bit_stream: B) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(frame_index, bit_stream, &[]).await
    }
    /// As [`read_frame`](Self::read_frame), with hints for the read job.
    pub async fn read_frame_with_hints<B: AsMut<[u8]> + Send + 'static>(&self, frame_index: u64, bit_stream: B, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let (bytes, len, buffer) = sdk_buffer(bit_stream);
        let mut job = std::ptr::null_mut();
        // SAFETY: the SDK writes the frame's bitstream to `bytes`, then decodes the
        // frame from them; the frame, and every job using it, holds `buffer`, which
        // owns them.
        let future = unsafe {
            self.raw.CreateJobReadFrame(frame_index, bytes.cast(), byte_count(len), &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_extend([self.raw.add_ref_and_get_guard(), buffer]))?
        };
        future.await
    }
    /// Trim every `frame_step`th frame of `frame_count` frames from `frame_index` to a
    /// new `.braw` at `file_path`, played back at `frame_rate`.
    #[allow(clippy::too_many_arguments)] // mirrors the SDK call
    pub async fn trim(&self, file_path: &str, frame_index: u64, frame_count: u64, frame_step: u32, frame_rate: f32, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let path = BrawString::from(file_path);
        let mut job = std::ptr::null_mut();
        // SAFETY: as in `BlackmagicRawClip::trim`.
        let future = unsafe {
            self.raw.CreateJobTrim(path.as_raw(), frame_index, frame_count, frame_step, frame_rate, clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), &mut job)?;
            submit(job, &[], job_guards(&self.parent_guards, self.raw.add_ref_and_get_guard(), &clip_processing_attributes, &frame_processing_attributes, [keep_alive(path)]))?
        };
        future.await
    }
    /// As [`trim`](Self::trim), writing the trimmed clip through `destination`.
    #[allow(clippy::too_many_arguments)] // mirrors the SDK call
    pub async fn trim_to_file(&self, destination: &BlackmagicRawFile, frame_index: u64, frame_count: u64, frame_step: u32, frame_rate: f32, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job = std::ptr::null_mut();
        // SAFETY: as in `BlackmagicRawClip::trim_to_file`.
        let future = unsafe {
            self.raw.CreateJobTrimToFile(destination.as_raw(), frame_index, frame_count, frame_step, frame_rate, clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()), &mut job)?;
            submit(job, &[], job_guards(&self.parent_guards, self.raw.add_ref_and_get_guard(), &clip_processing_attributes, &frame_processing_attributes, [destination.add_ref_and_get_guard()]))?
        };
        future.await
    }
    /// Where the audio chunk holding `sample_index` lives in the file.
    pub fn audio_chunk_info(&self, sample_index: u64) -> Result<AudioChunkInfo, BrawError> {
        let mut info = AudioChunkInfo::default();
        // SAFETY: out-parameters on this stack.
        unsafe { self.raw.GetAudioChunkInfo(sample_index, &mut info.size_bytes, &mut info.offset_bytes, &mut info.sample_count, &mut info.start_sample_index)? };
        Ok(info)
    }
}

/// The location of one audio chunk in a clip's file (see [`BlackmagicRawClipEx::audio_chunk_info`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct AudioChunkInfo {
    /// Size of the chunk in bytes
    pub size_bytes: u32,
    /// Offset of the chunk in the file, in bytes
    pub offset_bytes: u64,
    /// Number of sample frames in the chunk
    pub sample_count: u64,
    /// Index of the chunk's first sample frame
    pub start_sample_index: u64,
}

impl BlackmagicRawFrame {
    /// Returns an iterator over the metadata entries in the frame
    pub fn metadata_iter(&self) -> Result<MetadataIterator, BrawError> {
        // SAFETY: returns a new iterator through its out-parameter.
        let raw = unsafe { out_interface(|out| self.raw.GetMetadataIterator(out))? };
        Ok(MetadataIterator { raw, factory: self.factory.clone(), is_first: true, parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }
    /// Decode the frame and process it into an image, with the clip's settings
    /// unless processing attributes are given.
    pub async fn decode_and_process(&self, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<BlackmagicRawProcessedImage, BrawError> {
        self.create_decode_process_future(clip_processing_attributes, frame_processing_attributes)?.await
    }

    /// Submit a decode-and-process job and return a `'static`
    /// [`DecodeProcessFuture`] for its completion — the pipeline-friendly
    /// form of [`decode_and_process`](Self::decode_and_process). The job is submitted immediately;
    /// await (or poll) the future for the `BlackmagicRawProcessedImage`.
    pub fn create_decode_process_future(&self, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<DecodeProcessFuture, BrawError> {
        // The job reads the frame — and, for a frame read into a caller's buffer,
        // that buffer — and the processing attributes until it completes, so the job
        // holds them all, as does the future.
        let parent_guards = job_guards(&self.parent_guards, self.raw.add_ref_and_get_guard(), &clip_processing_attributes, &frame_processing_attributes, []);
        let mut job = std::ptr::null_mut();
        // SAFETY: live attribute objects (borrowed with `as_ref`: moving an `Option`
        // into the closure would release a sole-owned one before the call), held
        // until the job completes, and a new decode-and-process job, whose completion
        // delivers a processed image, through the out-parameter.
        let inner = unsafe {
            self.raw.CreateJobDecodeAndProcessFrame(
                clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()),
                frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |a| a.as_raw()),
                &mut job,
            )?;
            submit(job, &[], parent_guards.clone())?
        };
        Ok(DecodeProcessFuture::new(inner, self.factory.clone(), parent_guards))
    }
}

impl BlackmagicRawClipMultiVideo {
    /// Read frame `frame_index` of video track `track_index`.
    pub async fn read_frame(&self, track_index: u32, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(track_index, frame_index, &[]).await
    }
    /// As [`read_frame`](Self::read_frame), with hints for the read job.
    pub async fn read_frame_with_hints(&self, track_index: u32, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job = std::ptr::null_mut();
        // SAFETY: `job` receives a new read-frame job, which uses only the clip.
        let future = unsafe {
            self.raw.CreateJobReadFrame(track_index, frame_index, &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()))?
        };
        future.await
    }
    /// Read frame `frame_index` of video track `track_index` into `bit_stream`, a
    /// buffer with room for its [`bit_stream_size_bytes`](Self::bit_stream_size_bytes),
    /// laid out and owned as in [`BlackmagicRawClipEx::read_frame`].
    pub async fn read_frame_ex<B: AsMut<[u8]> + Send + 'static>(&self, track_index: u32, frame_index: u64, bit_stream: B) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_ex_with_hints(track_index, frame_index, bit_stream, &[]).await
    }
    /// As [`read_frame_ex`](Self::read_frame_ex), with hints for the read job.
    pub async fn read_frame_ex_with_hints<B: AsMut<[u8]> + Send + 'static>(&self, track_index: u32, frame_index: u64, bit_stream: B, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let (bytes, len, buffer) = sdk_buffer(bit_stream);
        let mut job = std::ptr::null_mut();
        // SAFETY: as in `BlackmagicRawClipEx::read_frame_with_hints`.
        let future = unsafe {
            self.raw.CreateJobReadFrameEx(track_index, frame_index, bytes.cast(), byte_count(len), &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_extend([self.raw.add_ref_and_get_guard(), buffer]))?
        };
        future.await
    }
}

impl BlackmagicRawClipImmersiveVideo {
    /// Read frame `frame_index` of the left or right eye's track.
    pub async fn read_frame(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(video_track, frame_index, &[]).await
    }
    /// As [`read_frame`](Self::read_frame), with hints for the read job.
    pub async fn read_frame_with_hints(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job = std::ptr::null_mut();
        // SAFETY: `job` receives a new read-frame job, which uses only the clip.
        let future = unsafe {
            self.raw.CreateJobImmersiveReadFrame(video_track, frame_index, &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()))?
        };
        future.await
    }
    /// Read frame `frame_index` of the left or right eye's track into `bit_stream`, a
    /// buffer with room for its [`immersive_bit_stream_size_bytes`](Self::immersive_bit_stream_size_bytes),
    /// laid out and owned as in [`BlackmagicRawClipEx::read_frame`].
    pub async fn read_frame_ex<B: AsMut<[u8]> + Send + 'static>(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: B) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_ex_with_hints(video_track, frame_index, bit_stream, &[]).await
    }
    /// As [`read_frame_ex`](Self::read_frame_ex), with hints for the read job.
    pub async fn read_frame_ex_with_hints<B: AsMut<[u8]> + Send + 'static>(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: B, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let (bytes, len, buffer) = sdk_buffer(bit_stream);
        let mut job = std::ptr::null_mut();
        // SAFETY: as in `BlackmagicRawClipEx::read_frame_with_hints`.
        let future = unsafe {
            self.raw.CreateJobImmersiveReadFrameEx(video_track, frame_index, bytes.cast(), byte_count(len), &mut job)?;
            read_frame_job(job, hints, &self.factory, self.parent_guards.clone_and_extend([self.raw.add_ref_and_get_guard(), buffer]))?
        };
        future.await
    }
}

impl BlackmagicRawProcessedImage {
    /// The image's bytes, in its [`resource_format`](Self::resource_format), when it
    /// was processed on the CPU; [`BrawError::NullValue`] otherwise.
    pub fn resource_cpu(&self) -> Result<&[u8], BrawError> {
        match self.resource_type()? {
            BlackmagicRawResourceType::BufferCPU => {
                let mut ptr: *mut c_void = std::ptr::null_mut();
                let size = self.resource_size_bytes()? as usize;
                // SAFETY: an out-parameter on this stack.
                unsafe { self.raw.GetResource(&mut ptr)? };
                if ptr.is_null() || size == 0 {
                    return Err(BrawError::NullValue);
                }
                // SAFETY: the image owns `size` bytes of CPU memory at `ptr` for as long
                // as it lives, which the returned borrow of `self` guarantees.
                Ok(unsafe { std::slice::from_raw_parts(ptr as *const u8, size) })
            },
            _ => Err(BrawError::NullValue),
        }
    }
    /// The GPU buffer holding the image and its type, when it was processed on a GPU
    /// pipeline; [`BrawError::NullValue`] otherwise.
    pub fn resource_gpu(&self) -> Result<(BlackmagicRawResourceType, *const c_void), BrawError> {
        let typ = self.resource_type()?;
        match typ {
            BlackmagicRawResourceType::BufferMetal |
            BlackmagicRawResourceType::BufferCUDA |
            BlackmagicRawResourceType::BufferOpenCL => {
                let mut ptr: *mut c_void = std::ptr::null_mut();
                // SAFETY: an out-parameter on this stack.
                unsafe { self.raw.GetResource(&mut ptr)? };
                if ptr.is_null() {
                    return Err(BrawError::NullValue);
                }
                Ok((typ, ptr as *const c_void))
            },
            _ => Err(BrawError::NullValue),
        }
    }
}

impl BlackmagicRawPost3DLUT {
    /// The LUT's data in CPU memory.
    pub fn resource_cpu(&self) -> Result<&[u8], BrawError> {
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let size = self.resource_size_bytes()? as usize;
        // SAFETY: an out-parameter on this stack.
        unsafe { self.raw.GetResourceCPU(&mut ptr)? };
        if ptr.is_null() || size == 0 {
            return Err(BrawError::NullValue);
        }
        // SAFETY: the LUT owns `size` bytes of CPU memory at `ptr` for as long as it
        // lives, which the returned borrow of `self` guarantees.
        Ok(unsafe { std::slice::from_raw_parts(ptr as *const u8, size) })
    }
    /// The GPU buffer holding the LUT on the pipeline `context` and `command_queue`
    /// belong to, and its type.
    ///
    /// # Safety
    /// `context` and `command_queue` must be null or valid for the pipeline the codec
    /// decodes with (see [`BlackmagicRawConfiguration::set_pipeline`]).
    pub unsafe fn resource_gpu(&self, context: *mut c_void, command_queue: *mut c_void) -> Result<(BlackmagicRawResourceType, *const c_void), BrawError> {
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let mut typ: BlackmagicRawResourceType = Default::default();
        // SAFETY: the handles are valid, per this function's contract, and the
        // out-parameters are on this stack.
        unsafe { self.raw.GetResourceGPU(context, command_queue, &mut typ, &mut ptr)? };
        if ptr.is_null() {
            return Err(BrawError::NullValue);
        }
        Ok((typ, ptr as *const c_void))
    }
    /// Write the LUT as a `.cube` file through `file`.
    pub fn write_cube_to_file(&self, file: &BlackmagicRawFile) -> Result<(), BrawError> {
        // SAFETY: a live file object; the write completes within the call.
        unsafe { self.raw.WriteCubeToFile(file.as_raw())? };
        Ok(())
    }
}

impl BlackmagicRawAudioBuffer {
    /// The buffer's interleaved little-endian PCM samples: sample frames of
    /// [`channel_count`](Self::channel_count) samples, [`bit_depth`](Self::bit_depth)
    /// bits each.
    pub fn samples(&self) -> Result<&[u8], BrawError> {
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let mut size: u32 = 0;
        // SAFETY: out-parameters on this stack.
        unsafe { self.raw.GetAudioSamples(&mut ptr, &mut size)? };
        if size == 0 {
            return Ok(&[]);
        }
        if ptr.is_null() {
            return Err(BrawError::NullValue);
        }
        // SAFETY: the SDK owns `size` bytes at `ptr` for as long as the buffer lives,
        // which the returned borrow of `self` guarantees.
        Ok(unsafe { std::slice::from_raw_parts(ptr as *const u8, size as usize) })
    }
}

/// The parameters of a Blackmagic Design custom gamma curve (see the `ToneCurve*`
/// [`BlackmagicRawClipProcessingAttribute`]s).
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct ToneCurve {
    /// Contrast
    pub contrast: f32,
    /// Saturation
    pub saturation: f32,
    /// Midpoint
    pub midpoint: f32,
    /// Highlight rolloff
    pub highlights: f32,
    /// Shadow rolloff
    pub shadows: f32,
    /// Black level
    pub black_level: f32,
    /// White level
    pub white_level: f32,
    /// Video black level
    pub video_black_level: u16
}

impl BlackmagicRawToneCurve {
    /// The tone curve for `camera_type` and `gamma` in color science generation `gen_`.
    pub fn get_tone_curve(&self, camera_type: &str, gamma: &str, gen_: u16) -> Result<ToneCurve, BrawError> {
        let camera_type = BrawString::from(camera_type);
        let gamma = BrawString::from(gamma);
        let mut curve = ToneCurve::default();
        // SAFETY: native strings, and out-parameters on this stack.
        unsafe {
            self.raw.GetToneCurve(camera_type.as_raw(), gamma.as_raw(), gen_,
                &mut curve.contrast,
                &mut curve.saturation,
                &mut curve.midpoint,
                &mut curve.highlights,
                &mut curve.shadows,
                &mut curve.black_level,
                &mut curve.white_level,
                &mut curve.video_black_level
            )?;
        }
        Ok(curve)
    }
    /// `curve` sampled at `num_elements` evenly spaced points, e.g. to draw it.
    pub fn evaluate_tone_curve(&self, camera_type: &str, gen_: u16, curve: &ToneCurve, num_elements: u32) -> Result<Vec<f32>, BrawError> {
        let camera_type = BrawString::from(camera_type);
        let mut array = vec![0.0f32; num_elements as usize];
        // SAFETY: a native string, and an array of `num_elements` elements.
        unsafe {
            self.raw.EvaluateToneCurve(camera_type.as_raw(), gen_,
                curve.contrast,
                curve.saturation,
                curve.midpoint,
                curve.highlights,
                curve.shadows,
                curve.black_level,
                curve.white_level,
                curve.video_black_level,
                array.as_mut_ptr(),
                num_elements
            )?;
        }
        Ok(array)
    }
}

impl BlackmagicRawClipAccelerometerMotion {
    /// The samples in `range`, each [`sample_size`](Self::sample_size) floats, flattened.
    pub fn sample_range<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Result<Vec<f32>, BrawError> {
        let (start, count) = sample_range_bounds(range, || self.sample_count())?;
        let sample_size = self.sample_size()? as usize;
        let mut samples = vec![0.0f32; (count as usize).checked_mul(sample_size).ok_or(BrawError::InvalidArgument)?];
        let mut out_count = 0u32;
        // SAFETY: room for `count` samples of `sample_size` floats, and an
        // out-parameter on this stack.
        unsafe { self.raw.GetSampleRange(start, count, samples.as_mut_ptr(), &mut out_count)? };
        samples.truncate(out_count.min(count) as usize * sample_size);
        Ok(samples)
    }
}

impl BlackmagicRawClipGyroscopeMotion {
    /// The samples in `range`, each [`sample_size`](Self::sample_size) floats in
    /// radians per second, flattened.
    pub fn sample_range<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Result<Vec<f32>, BrawError> {
        let (start, count) = sample_range_bounds(range, || self.sample_count())?;
        let sample_size = self.sample_size()? as usize;
        let mut samples = vec![0.0f32; (count as usize).checked_mul(sample_size).ok_or(BrawError::InvalidArgument)?];
        let mut out_count = 0u32;
        // SAFETY: room for `count` samples of `sample_size` floats, and an
        // out-parameter on this stack.
        unsafe { self.raw.GetSampleRange(start, count, samples.as_mut_ptr(), &mut out_count)? };
        samples.truncate(out_count.min(count) as usize * sample_size);
        Ok(samples)
    }
}

/// The first sample and the sample count of `range`, a range of sample indices;
/// `sample_count` gives the end of an unbounded one.
fn sample_range_bounds(range: impl std::ops::RangeBounds<u64>, sample_count: impl FnOnce() -> Result<u32, BrawError>) -> Result<(u64, u32), BrawError> {
    use std::ops::Bound;
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start.checked_add(1).ok_or(BrawError::InvalidArgument)?,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&end) => end.checked_add(1).ok_or(BrawError::InvalidArgument)?,
        Bound::Excluded(&end) => end,
        Bound::Unbounded => u64::from(sample_count()?),
    };
    let count = end.checked_sub(start).ok_or(BrawError::InvalidArgument)?;
    Ok((start, u32::try_from(count).map_err(|_| BrawError::InvalidArgument)?))
}

impl BlackmagicRawPipelineDevice {
    /// The resource formats the device can process frames into.
    pub fn supported_resource_formats(&self) -> Result<Vec<BlackmagicRawResourceFormat>, BrawError> {
        let mut count = 0;
        // SAFETY: a null array asks for the count.
        unsafe { self.raw.GetSupportedResourceFormats(std::ptr::null_mut(), &mut count)? };
        if count == 0 {
            return Ok(vec![]);
        }
        let mut vec = vec![BlackmagicRawResourceFormat::Null; count as usize];
        // SAFETY: an array of `count` elements.
        unsafe { self.raw.GetSupportedResourceFormats(vec.as_mut_ptr(), &mut count)? };
        vec.truncate(count as usize);
        Ok(vec)
    }
}

impl BlackmagicRawClipProcessingAttributes {
    /// The `(minimum, maximum, is_read_only)` of a continuous attribute.
    pub fn clip_attribute_range(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: out-parameters on this stack.
        unsafe { self.raw.GetClipAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)? };
        Ok((self.factory.lib.variant_to_rust(value_min), self.factory.lib.variant_to_rust(value_max), sdk_bool(is_read_only)))
    }
    /// The `(values, is_read_only)` of an attribute with a fixed set of values.
    pub fn clip_attribute_list(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: a null array asks for the count.
        unsafe { self.raw.GetClipAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)? };
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        // SAFETY: an array of `count` elements.
        unsafe { self.raw.GetClipAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)? };
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.factory.lib.variant_to_rust(v)).collect();
        Ok((vec, sdk_bool(is_read_only)))
    }
    /// The `(ISOs, is_read_only)` available for the clip's analog gain.
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: a null array asks for the count.
        unsafe { self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)? };
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![0u32; count as usize];
        // SAFETY: an array of `count` elements.
        unsafe { self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)? };
        vec.truncate(count as usize);
        Ok((vec, sdk_bool(is_read_only)))
    }
}

impl BlackmagicRawFrameProcessingAttributes {
    /// The `(minimum, maximum, is_read_only)` of a continuous attribute.
    pub fn frame_attribute_range(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: out-parameters on this stack.
        unsafe { self.raw.GetFrameAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)? };
        Ok((self.factory.lib.variant_to_rust(value_min), self.factory.lib.variant_to_rust(value_max), sdk_bool(is_read_only)))
    }
    /// The `(values, is_read_only)` of an attribute with a fixed set of values.
    pub fn frame_attribute_list(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: a null array asks for the count.
        unsafe { self.raw.GetFrameAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)? };
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        // SAFETY: an array of `count` elements.
        unsafe { self.raw.GetFrameAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)? };
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.factory.lib.variant_to_rust(v)).collect();
        Ok((vec, sdk_bool(is_read_only)))
    }
    /// The `(ISOs, is_read_only)` available for the frame's analog gain.
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        // SAFETY: a null array asks for the count.
        unsafe { self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)? };
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![0u32; count as usize];
        // SAFETY: an array of `count` elements.
        unsafe { self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)? };
        vec.truncate(count as usize);
        Ok((vec, sdk_bool(is_read_only)))
    }
}

/// The number of bytes `GetAudioSamples` may write for `max_sample_count`
/// interleaved sample-frames.
///
/// The BMD SDK contract (`IBlackmagicRawClipAudio::GetAudioSamples`) delivers
/// interleaved little-endian PCM: for each of the up-to-`max_sample_count`
/// sample-frames it writes one sample per channel, each `bit_depth` bits wide.
/// `GetAudioBitDepth` reports bits (always a multiple of 8 — 16/24/32) and
/// `GetAudioChannelCount` reports the channel count, so the upper bound is
/// `max_sample_count * channel_count * bit_depth / 8` bytes.
///
/// The product is computed in `u64` so it can never wrap (all three factors are
/// `u32`, so their product needs up to 96 bits — well beyond `u32`). A wrapping
/// `u32` product would defeat the buffer-size guard: it could collapse a large
/// `max_sample_count` to a tiny (or zero) required size, letting an
/// undersized/empty buffer pass the check while `GetAudioSamples` is still handed
/// the large `max_sample_count` — a heap overrun. On overflow (only reachable
/// with absurd factors) [`BrawError::InvalidArgument`] is returned.
fn required_sample_bytes(max_sample_count: u32, channel_count: u32, bit_depth: u32) -> Result<u64, BrawError> {
    u64::from(max_sample_count)
        .checked_mul(u64::from(channel_count))
        .and_then(|v| v.checked_mul(u64::from(bit_depth)))
        .map(|bits| bits / 8)
        .ok_or(BrawError::InvalidArgument)
}

impl BlackmagicRawClipAudio {
    /// Read up to `max_sample_count` (default 48000) sample frames from
    /// `sample_frame_index`, returning their interleaved little-endian PCM bytes and
    /// the number of sample frames read.
    pub fn samples(&self, sample_frame_index: i64, max_sample_count: Option<u32>) -> Result<(Vec<u8>, u32), BrawError> {
        let max_sample_count = max_sample_count.unwrap_or(48000);
        let required_bytes = required_sample_bytes(max_sample_count, self.channel_count()?, self.bit_depth()?)?;
        // `GetAudioSamples`' `bufferSizeBytes` parameter is a `u32`; a buffer it
        // cannot even describe is unusable, so reject rather than truncate.
        let buffer_size_bytes = u32::try_from(required_bytes).map_err(|_| BrawError::InvalidArgument)?;
        let mut buffer: Vec<u8> = vec![0; buffer_size_bytes as usize];
        let mut samples_read: u32 = 0;
        let mut bytes_read: u32 = 0;
        // SAFETY: a buffer of `buffer_size_bytes` bytes, room for `max_sample_count`
        // sample frames, and out-parameters on this stack.
        unsafe { self.raw.GetAudioSamples(sample_frame_index, buffer.as_mut_ptr() as *mut c_void, buffer_size_bytes, max_sample_count, &mut samples_read, &mut bytes_read)? };

        buffer.truncate(bytes_read as usize);
        Ok((buffer, samples_read))
    }

    /// Read interleaved little-endian PCM audio samples directly into a caller-provided buffer,
    /// without allocating.
    ///
    /// `dst` MUST be at least `max_sample_count * channel_count() * bit_depth() / 8` bytes long
    /// (the number of bytes the SDK may write for `max_sample_count` samples). If it is too small,
    /// [`BrawError::InvalidArgument`] is returned and no read is performed, rather than letting the
    /// SDK overrun the buffer.
    ///
    /// Returns the number of samples (per channel) actually read into `dst`.
    pub fn samples_into(&self, sample_frame_index: i64, max_sample_count: u32, dst: &mut [u8]) -> Result<u32, BrawError> {
        let required_bytes = required_sample_bytes(max_sample_count, self.channel_count()?, self.bit_depth()?)?;
        if (dst.len() as u64) < required_bytes {
            return Err(BrawError::InvalidArgument);
        }
        // Report the real capacity to the SDK, clamped to the `u32` parameter
        // width (a buffer larger than `u32::MAX` is served in full up to that cap).
        let buffer_size_bytes = u32::try_from(dst.len()).unwrap_or(u32::MAX);
        let mut samples_read: u32 = 0;
        let mut bytes_read: u32 = 0;
        // SAFETY: `dst` has room for `max_sample_count` sample frames (checked above)
        // in its `buffer_size_bytes` or more bytes, and the out-parameters are on this
        // stack.
        unsafe { self.raw.GetAudioSamples(sample_frame_index, dst.as_mut_ptr() as *mut c_void, buffer_size_bytes, max_sample_count, &mut samples_read, &mut bytes_read)? };
        Ok(samples_read)
    }
}

impl BlackmagicRawClipPDAFData {
    /// The `(left, right)` phase detection images of sample `sample_index`.
    pub fn sample_images(&self, sample_index: u64) -> Result<(Vec<u8>, Vec<u8>), BrawError> {
        let sample_image_width = self.sample_image_width_in_pixels()?;
        let sample_image_height = self.sample_image_height_in_pixels()?;
        let sample_image_bytes_per_pixel = self.sample_image_bytes_per_pixel()?;
        let sample_image_data_size = sample_image_width
            .checked_mul(sample_image_height)
            .and_then(|v| v.checked_mul(sample_image_bytes_per_pixel))
            .ok_or(BrawError::Fail)?;
        let mut left_buffer  = vec![0u8; sample_image_data_size as usize];
        let mut right_buffer = vec![0u8; sample_image_data_size as usize];
        // SAFETY: two buffers of `sample_image_data_size` bytes.
        unsafe { self.raw.GetSampleImages(sample_index, left_buffer.as_mut_ptr(), right_buffer.as_mut_ptr(), sample_image_data_size)? };
        Ok((left_buffer, right_buffer))
    }
}

#[cfg(test)]
mod audio_buffer_tests {
    use super::{required_sample_bytes, BrawError};

    #[test]
    fn required_bytes_matches_interleaved_pcm_formula() {
        // 48000 sample-frames, 2 channels, 24-bit → 48000 * 2 * 3 bytes.
        assert_eq!(required_sample_bytes(48_000, 2, 24).unwrap(), 48_000 * 2 * 3);
        // 16-bit stereo.
        assert_eq!(required_sample_bytes(1_024, 2, 16).unwrap(), 1_024 * 2 * 2);
    }

    #[test]
    fn huge_sample_count_is_not_wrapped_and_would_reject_a_small_buffer() {
        // Picked so the *naive all-u32* product `n * channels * bit_depth` is
        // exactly 2^32 and wraps to 0, which is the bug this guards against.
        let n: u32 = 0x0400_0000; // 2^26
        let channels: u32 = 2;
        let bit_depth: u32 = 32; // n * 2 * 32 == 2^32 == 0 (mod 2^32)
        let wrapped = n.wrapping_mul(channels).wrapping_mul(bit_depth) / 8;
        assert_eq!(wrapped, 0, "precondition: the naive u32 product wraps to 0");

        // The overflow-safe computation yields the true 512 MiB requirement.
        let required = required_sample_bytes(n, channels, bit_depth).unwrap();
        assert_eq!(required, 512 * 1024 * 1024);

        // Therefore a small buffer is rejected instead of wrapped-and-accepted.
        // (This mirrors `samples_into`'s `(dst.len() as u64) < required` guard.)
        for small_len in [0usize, 16, 4096] {
            assert!((small_len as u64) < required, "a {small_len}-byte buffer must be rejected");
        }
    }

    #[test]
    fn product_overflow_returns_invalid_argument() {
        // Even the u64 product cannot represent these absurd factors: reject.
        assert!(matches!(
            required_sample_bytes(u32::MAX, u32::MAX, u32::MAX),
            Err(BrawError::InvalidArgument)
        ));
    }
}