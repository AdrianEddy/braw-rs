// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_snake_case)]

#![doc = include_str!("../README.md")]

use core::ffi::c_void;
use std::sync::Arc;

mod callback;  pub use callback::*;
mod com;       pub use com::*;
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

    // This must be last
    lib: dl::Library,
}

impl RawLibrary {
    /// Load the BRAW shared library
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

                lib,
            })
        }
    }

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

    // Keep the default callback here to ensure it outlives any codecs created from this factory
    default_callback: Arc<CallbackHandle<DefaultCallback>>,

    // This must be last
    // Arc is used because this may potentially be cloned from the callback, which is called from a different thread
    lib: Arc<RawLibrary>
}

impl Factory {
    pub fn load_from(path: impl AsRef<std::path::Path>) -> Result<Self, BrawError> {
        let lib = RawLibrary::load(path.as_ref())?;
        let factory = lib.create_factory()?;
        let default_callback = Arc::new(CallbackHandle::new(DefaultCallback::default()));
        Ok(Self {
            lib: Arc::new(lib),
            factory,
            default_callback
        })
    }

    /// Create a codec from the factory
    ///
    /// Fails with [`BrawError::UnsupportedSdkVersion`] when the loaded library does
    /// not implement the Blackmagic RAW SDK 6.0 codec interface these bindings are
    /// built on — an older SDK, or a newer one that changed it again.
    pub fn create_codec(&self) -> Result<BlackmagicRaw, BrawError> {
        let raw: ComPtr<IBlackmagicRaw> = braw_out_ptr!(|pp| self.factory.CreateCodec(pp));

        // The codec's IID changes whenever its vtable does. A library of another
        // version hands back a codec with a different method layout — calling into
        // it (even the `SetCallback` below) would jump to the wrong slot — so confirm
        // it answers to the 6.0 IID before touching anything else. Only
        // `E_NOINTERFACE` means another version; any other failure is the call's own.
        let mut current: *mut c_void = std::ptr::null_mut();
        let hr = unsafe { ((*raw.vtbl).parent.QueryInterface)(raw.as_raw() as _, IBlackmagicRaw::iid(), &mut current) };
        if hr == E_NOINTERFACE {
            return Err(BrawError::UnsupportedSdkVersion(self.camera_support_version(&raw)));
        }
        check_hr(hr)?;
        drop(ComPtr::new(current as *mut IBlackmagicRaw)?);

        let codec = BlackmagicRaw {
            raw,
            factory: self.clone(),
            parent_guards: vec![].into(),
        };
        let _ = codec.raw.SetCallback(self.default_callback.as_mut_ptr())?;

        Ok(codec)
    }

    /// Create a pipeline iterator
    pub fn pipeline_iter(&self, interop: BlackmagicRawInterop) -> Result<PipelineIterator, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.factory.CreatePipelineIterator(interop, &mut out) {
            Ok(_)  => Ok(PipelineIterator { raw: ComPtr::new(out)?, factory: self.clone(), is_first: true }),
            Err(e) => Err(e),
        }
    }
    /// Create a pipeline device iterator
    pub fn pipeline_device_iter(&self, pipeline: BlackmagicRawPipeline, interop: BlackmagicRawInterop) -> Result<PipelineDeviceIterator, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.factory.CreatePipelineDeviceIterator(pipeline, interop, &mut out) {
            Ok(_)  => Ok(PipelineDeviceIterator { raw: ComPtr::new(out)?, factory: self.clone(), is_first: true, current_index: Arc::new(std::sync::atomic::AtomicUsize::new(0)) }),
            Err(e) => Err(e),
        }
    }
    /// Create empty clip geometry object
    pub fn create_clip_geometry(&self) -> Result<BlackmagicRawClipGeometry, BrawError> {
        let geom = braw_out_ptr!(|pp| self.factory.CreateClipGeometry(pp));
        Ok(BlackmagicRawClipGeometry { raw: geom, factory: self.clone(), parent_guards: vec![].into() } )
    }

    /// The loaded library's camera support version, read through
    /// `IBlackmagicRawConfiguration` — the one interface whose IID and layout are the
    /// same in every SDK from 4.2 to 6.0, so it is safe to call on a library the
    /// bindings do not otherwise support ("unknown" from one that changed it too).
    /// (`GetVersion` would be the natural choice, but the 5.0 and 6.0 libraries both
    /// answer it with "0.0".)
    fn camera_support_version(&self, codec: &ComPtr<IBlackmagicRaw>) -> String {
        let mut ptr = std::ptr::null_mut();
        let hr = unsafe { ((*codec.vtbl).parent.QueryInterface)(codec.as_raw() as _, IBlackmagicRawConfiguration::iid(), &mut ptr) };
        if hr != S_OK {
            return "unknown".into();
        }
        let Ok(configuration) = ComPtr::new(ptr as *mut IBlackmagicRawConfiguration) else { return "unknown".into() };
        let mut version = std::ptr::null_mut();
        match configuration.GetCameraSupportVersion(&mut version) {
            Ok(_) => unsafe { take_sdk_string(version) },
            Err(_) => "unknown".into(),
        }
    }
}
unsafe impl Send for Factory {}
unsafe impl Sync for Factory {}

impl BlackmagicRaw {
    pub fn open_clip(&self, path: &str) -> Result<BlackmagicRawClip, BrawError> {
        let in_str = BrawString::from(path);
        let clip = braw_out_ptr!(|pp| self.raw.OpenClip(in_str.as_raw(), pp));
        Ok(BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }
    pub fn open_clip_with_geometry(&self, path: &str, geometry: BlackmagicRawClipGeometry) -> Result<BlackmagicRawClip, BrawError> {
        let in_str = BrawString::from(path);
        let clip = braw_out_ptr!(|pp| self.raw.OpenClipWithGeometry(in_str.as_raw(), geometry.as_raw(), pp));
        Ok(BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
    }
    /// Open a clip read through `file` (see the [`file`](crate::BrawFile) traits).
    /// The clip keeps `file` alive for its whole lifetime.
    pub fn open_clip_from_file(&self, file: &BlackmagicRawFile) -> Result<BlackmagicRawClip, BrawError> {
        let clip = braw_out_ptr!(|pp| self.raw.OpenClipFromFile(file.as_raw(), pp));
        Ok(self.clip_from_file(clip, file))
    }
    /// Open a clip read through `file`, with `geometry` applied regardless of the clip's metadata.
    pub fn open_clip_from_file_with_geometry(&self, file: &BlackmagicRawFile, geometry: BlackmagicRawClipGeometry) -> Result<BlackmagicRawClip, BrawError> {
        let clip = braw_out_ptr!(|pp| self.raw.OpenClipFromFileWithGeometry(file.as_raw(), geometry.as_raw(), pp));
        Ok(self.clip_from_file(clip, file))
    }
    fn clip_from_file(&self, clip: ComPtr<IBlackmagicRawClip>, file: &BlackmagicRawFile) -> BlackmagicRawClip {
        // The file is the clip's byte source: every read job the clip (or a frame
        // future cloned from it) can still issue must find it alive.
        let parent_guards = self.parent_guards.clone_and_extend([self.raw.add_ref_and_get_guard(), file.add_ref_and_get_guard()]);
        BlackmagicRawClip { raw: clip, factory: self.factory.clone(), parent_guards }
    }

    // TODO: this is replacing the callback for all codec instances, which is not great and not what the user expects
    pub fn set_callback<T: BrawCallback>(&mut self, callback: T) -> Result<(), BrawError> {
        // FIXME: unsound while a callback runs on an SDK thread (see the TODO above).
        let dcb = unsafe { &mut *self.factory.default_callback.state_ptr() };
        dcb.user_callback = Some(Box::new(callback));
        Ok(())
    }

    /// Asynchronously prepares the current pipeline
    ///
    /// This function returns a future which needs to be awaited.
    /// `PreparePipeline` is started immediately when calling this function. You can either `await` the returned future, or use the callback mechanism to get notified when it's done.
    pub fn prepare_pipeline(&self, pipeline: u32, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void) -> Result<CallbackFuture<()>, BrawError> {
        let state = std::sync::Arc::new(State::<()>::new());

        // Owned refcount handed to the SDK; reclaimed in
        // `prepare_pipeline_complete` via `Arc::from_raw`. Reclaim here too if
        // the call fails (no callback will fire) so `State` isn't leaked.
        let raw = Arc::into_raw(state.clone()) as *mut c_void;
        if let Err(e) = self.raw.PreparePipeline(pipeline, pipeline_context, pipeline_command_queue, raw) {
            unsafe { drop(Arc::from_raw(raw as *const State<()>)); }
            return Err(e);
        }

        Ok(CallbackFuture { state, job: None })
    }

    /// Asynchronously prepares the current pipeline
    ///
    /// This function returns a future which needs to be awaited.
    /// `PreparePipeline` is started immediately when calling this function. You can either `await` the returned future, or use the callback mechanism to get notified when it's done.
    pub fn prepare_pipeline_for_device(&self, device: BlackmagicRawPipelineDevice) -> Result<CallbackFuture<()>, BrawError> {
        let state = std::sync::Arc::new(State::<()>::new());
        let raw = Arc::into_raw(state.clone()) as *mut c_void;
        if let Err(e) = self.raw.PreparePipelineForDevice(device.as_raw(), raw) {
            unsafe { drop(Arc::from_raw(raw as *const State<()>)); }
            return Err(e);
        }
        Ok(CallbackFuture { state, job: None })
    }
}

impl BlackmagicRawClip {
    /// Returns an iterator over the metadata entries in the clip
    pub fn metadata_iter(&self) -> Result<MetadataIterator, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.raw.GetMetadataIterator(&mut out) {
            Ok(_)  => Ok(MetadataIterator { raw: ComPtr::new(out)?, factory: self.factory.clone(), is_first: true, parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) }),
            Err(e) => Err(e),
        }
    }

    pub async fn read_frame(&self, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        self.create_read_frame_future(frame_index, hints)?.await
    }

    /// Submit a read-frame job and return a `'static` [`ReadFrameFuture`]
    /// for its completion — the pipeline-friendly form of [`read_frame`](Self::read_frame).
    /// The job is submitted immediately; await (or poll) the future for
    /// the `BlackmagicRawFrame`. Unlike `read_frame` the future borrows
    /// nothing from `self`, so a scheduler can keep many in flight.
    pub fn create_read_frame_future(&self, frame_index: u64, hints: &[ReadJobHints]) -> Result<ReadFrameFuture, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(frame_index, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let inner = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?;
        Ok(ReadFrameFuture::new(inner, self.factory.clone(), parent_guards))
    }
    /// Read up to `max_sample_count` sample frames of audio, starting at `sample_index`.
    pub async fn read_audio(&self, sample_index: u64, max_sample_count: u64) -> Result<BlackmagicRawAudioBuffer, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadAudio(sample_index, max_sample_count, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let buffer: ComPtr<IBlackmagicRawAudioBuffer> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await?;
        Ok(BlackmagicRawAudioBuffer { raw: buffer, factory: self.factory.clone(), parent_guards })
    }
    pub async fn trim(&self, file_name: &str, frame_index: u64, frame_count: u64, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        // Marshal the output path into the platform SDK string (BSTR on Windows,
        // CFString on macOS, NUL-terminated UTF-8 on Linux), exactly as `open_clip`
        // does. Passing `file_name.as_ptr()` — a non-NUL-terminated pointer to the
        // Rust `&str`'s UTF-8 bytes — is a bug: the SDK reads it as its native string
        // type, so on Windows the UTF-8 bytes are reinterpreted as UTF-16 (and, with
        // no terminator, the read runs past the string into adjacent memory),
        // producing a garbled separator-less name that resolves against the CWD.
        // `in_str` is bound for the whole async fn so it outlives the `CreateJobTrim`
        // call and the awaited job.
        let in_str = BrawString::from(file_name);
        // Borrow (`as_ref`) rather than move: consuming the `Option`s here would drop
        // (COM `Release`) a sole-owned attributes object before `CreateJobTrim` runs.
        // As owned params of this async fn they stay alive across the `.await` below,
        // covering the whole job. (See `create_decode_process_future`.)
        self.raw.CreateJobTrim(
            in_str.as_raw(),
            frame_index,
            frame_count,
            clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr
        )?;

        CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await
    }
    /// As [`trim`](Self::trim), writing the trimmed clip through `destination`.
    pub async fn trim_to_file(&self, destination: &BlackmagicRawFile, frame_index: u64, frame_count: u64, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        // `destination` and the attributes are borrowed for the whole async fn, so
        // they outlive the awaited job (see `trim`).
        self.raw.CreateJobTrimToFile(
            destination.as_raw(),
            frame_index,
            frame_count,
            clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr
        )?;

        CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await
    }
}

impl BlackmagicRawClipEx {
    pub async fn read_frame(&self, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_with_hints(&self, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, factory: self.factory.clone(), parent_guards })
    }
    /// Trim every `frame_step`th frame of `frame_count` frames from `frame_index` to a
    /// new `.braw` at `file_path`, played back at `frame_rate`.
    #[allow(clippy::too_many_arguments)] // mirrors the SDK call
    pub async fn trim(&self, file_path: &str, frame_index: u64, frame_count: u64, frame_step: u32, frame_rate: f32, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        // The path string and the attributes are bound for the whole async fn, so
        // they outlive the awaited job (see `BlackmagicRawClip::trim`).
        let in_str = BrawString::from(file_path);
        self.raw.CreateJobTrim(
            in_str.as_raw(),
            frame_index,
            frame_count,
            frame_step,
            frame_rate,
            clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr
        )?;

        CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await
    }
    /// As [`trim`](Self::trim), writing the trimmed clip through `destination`.
    #[allow(clippy::too_many_arguments)] // mirrors the SDK call
    pub async fn trim_to_file(&self, destination: &BlackmagicRawFile, frame_index: u64, frame_count: u64, frame_step: u32, frame_rate: f32, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<(), BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobTrimToFile(
            destination.as_raw(),
            frame_index,
            frame_count,
            frame_step,
            frame_rate,
            clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr
        )?;

        CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await
    }
    /// Where the audio chunk holding `sample_index` lives in the file.
    pub fn audio_chunk_info(&self, sample_index: u64) -> Result<AudioChunkInfo, BrawError> {
        let mut info = AudioChunkInfo::default();
        self.raw.GetAudioChunkInfo(sample_index, &mut info.size_bytes, &mut info.offset_bytes, &mut info.sample_count, &mut info.start_sample_index)?;
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
    pub fn metadata_iter(&self) -> Result<MetadataIterator, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.raw.GetMetadataIterator(&mut out) {
            Ok(_)  => Ok(MetadataIterator { raw: ComPtr::new(out)?, factory: self.factory.clone(), is_first: true, parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) }),
            Err(e) => Err(e),
        }
    }
    pub async fn decode_and_process(&self, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<BlackmagicRawProcessedImage, BrawError> {
        self.create_decode_process_future(clip_processing_attributes, frame_processing_attributes)?.await
    }

    /// Submit a decode-and-process job and return a `'static`
    /// [`DecodeProcessFuture`] for its completion — the pipeline-friendly
    /// form of [`decode_and_process`](Self::decode_and_process). The job is submitted immediately;
    /// await (or poll) the future for the `BlackmagicRawProcessedImage`.
    pub fn create_decode_process_future(&self, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes>) -> Result<DecodeProcessFuture, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        // Borrow (`as_ref`) the processing attributes for the FFI call. The previous
        // `map_or(.., |f| f.as_raw())` moved each `Option` INTO the closure, so the
        // wrapper was dropped — COM `Release` — BEFORE `CreateJobDecodeAndProcessFrame`
        // ran. A sole-owned frame-attributes object (refcount 1) was therefore freed
        // and the SDK received a dangling pointer; clip attributes only survived
        // because the caller happened to keep the original alive. `as_ref` borrows so
        // both stay alive across the call, and their guards are bound to the returned
        // future below.
        self.raw.CreateJobDecodeAndProcessFrame(
            clip_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.as_ref().map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr,
        )?;

        // The decode reads the processing attributes throughout async processing, not
        // just at submit time, so the job must outlive this call's stack frame. Retain
        // each attribute object's COM refcount for the future's whole lifetime by
        // binding its guard to `parent_guards` — the same keep-alive the clip / frame /
        // codec chain already rides. This makes `Some(attrs)` self-contained: the
        // caller need not keep a separate reference alive until the decode completes.
        let parent_guards = self.parent_guards.clone_and_extend(
            [Some(self.raw.add_ref_and_get_guard()),
             clip_processing_attributes.as_ref().map(|c| c.raw.add_ref_and_get_guard()),
             frame_processing_attributes.as_ref().map(|f| f.raw.add_ref_and_get_guard())]
            .into_iter().flatten()
        );

        let inner = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?;
        Ok(DecodeProcessFuture::new(inner, self.factory.clone(), parent_guards))
    }
}

impl BlackmagicRawClipMultiVideo {
    pub async fn read_frame(&self, track_index: u32, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(track_index, frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, track_index: u32, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(track_index, frame_index, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, factory: self.factory.clone(), parent_guards })
    }
    pub async fn read_frame_ex(&self, track_index: u32, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_ex_with_hints(track_index, frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_ex_with_hints(&self, track_index: u32, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrameEx(track_index, frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, factory: self.factory.clone(), parent_guards })
    }
}

impl BlackmagicRawClipImmersiveVideo {
    pub async fn read_frame(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_with_hints(video_track, frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobImmersiveReadFrame(video_track, frame_index, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, factory: self.factory.clone(), parent_guards })
    }
    pub async fn read_frame_ex(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame, BrawError> {
        self.read_frame_ex_with_hints(video_track, frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_ex_with_hints(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobImmersiveReadFrameEx(video_track, frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let parent_guards = self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard());

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, factory: self.factory.clone(), parent_guards })
    }
}

impl BlackmagicRawProcessedImage {
    pub fn resource_cpu(&self) -> Result<&[u8], BrawError> {
        match self.resource_type()? {
            BlackmagicRawResourceType::BufferCPU => {
                unsafe {
                    let mut ptr: *mut c_void = std::ptr::null_mut();
                    let size = self.resource_size_bytes()? as usize;
                    let _ = self.raw.GetResource(&mut ptr)?;
                    if ptr.is_null() || size == 0 {
                        return Err(BrawError::NullValue);
                    }
                    Ok(std::slice::from_raw_parts(ptr as *const u8, size))
                }
            },
            _ => Err(BrawError::NullValue),
        }
    }
    pub fn resource_gpu(&self) -> Result<(BlackmagicRawResourceType, *const c_void), BrawError> {
        let typ = self.resource_type()?;
        match typ {
            BlackmagicRawResourceType::BufferMetal |
            BlackmagicRawResourceType::BufferCUDA |
            BlackmagicRawResourceType::BufferOpenCL => {
                let mut ptr: *mut c_void = std::ptr::null_mut();
                let _ = self.raw.GetResource(&mut ptr)?;
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
    pub fn resource_cpu(&self) -> Result<&[u8], BrawError> {
        unsafe {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            let size = self.resource_size_bytes()? as usize;
            let _ = self.raw.GetResourceCPU(&mut ptr)?;
            if ptr.is_null() || size == 0 {
                return Err(BrawError::NullValue);
            }
            Ok(std::slice::from_raw_parts(ptr as *const u8, size))
        }
    }
    pub fn resource_gpu(&self, context: *mut c_void, command_queue: *mut c_void) -> Result<(BlackmagicRawResourceType, *const c_void), BrawError> {
        let mut ptr: *mut c_void = std::ptr::null_mut();
        let mut typ: BlackmagicRawResourceType = Default::default();
        let _ = self.raw.GetResourceGPU(context, command_queue, &mut typ, &mut ptr)?;
        if ptr.is_null() {
            return Err(BrawError::NullValue);
        }
        Ok((typ, ptr as *const c_void))
    }
    /// Write the LUT as a `.cube` file through `file`.
    pub fn write_cube_to_file(&self, file: &BlackmagicRawFile) -> Result<(), BrawError> {
        self.raw.WriteCubeToFile(file.as_raw())?;
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
        self.raw.GetAudioSamples(&mut ptr, &mut size)?;
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

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct ToneCurve {
    pub contrast: f32,
    pub saturation: f32,
    pub midpoint: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub black_level: f32,
    pub white_level: f32,
    pub video_black_level: u16
}

impl BlackmagicRawToneCurve {
    pub fn get_tone_curve(&self, camera_type: &str, gamma: &str, gen_: u16) -> Result<ToneCurve, BrawError> {
        let camera_type = BrawString::from(camera_type);
        let gamma = BrawString::from(gamma);
        let mut curve = ToneCurve::default();
        let _ = self.raw.GetToneCurve(camera_type.as_raw(), gamma.as_raw(), gen_,
            &mut curve.contrast,
            &mut curve.saturation,
            &mut curve.midpoint,
            &mut curve.highlights,
            &mut curve.shadows,
            &mut curve.black_level,
            &mut curve.white_level,
            &mut curve.video_black_level
        )?;
        Ok(curve)
    }
    pub fn evaluate_tone_curve(&self, camera_type: &str, gen_: u16, curve: &ToneCurve, num_elements: u32) -> Result<Vec<f32>, BrawError> {
        let camera_type = BrawString::from(camera_type);
        let mut array = vec![0.0f32; num_elements as usize];
        let _ = self.raw.EvaluateToneCurve(camera_type.as_raw(), gen_,
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
        Ok(array)
    }
}

impl BlackmagicRawClipAccelerometerMotion {
    pub fn sample_range<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Result<Vec<f32>, BrawError> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => self.sample_count()? as u64,
        };
        let count = end - start;
        let sample_size =  self.sample_size()? as usize;
        let mut samples = vec![0.0f32; count as usize * sample_size];
        let mut out_count = 0u32;

        self.raw.GetSampleRange(start, count as u32, samples.as_mut_ptr(), &mut out_count)?;
        samples.truncate(out_count as usize * sample_size);
        Ok(samples)
    }
}

impl BlackmagicRawClipGyroscopeMotion {
    pub fn sample_range<T: std::ops::RangeBounds<u64>>(&self, range: T) -> Result<Vec<f32>, BrawError> {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => self.sample_count()? as u64,
        };
        let count = end - start;
        let sample_size =  self.sample_size()? as usize;
        let mut samples = vec![0.0f32; count as usize * sample_size];
        let mut out_count = 0u32;

        self.raw.GetSampleRange(start, count as u32, samples.as_mut_ptr(), &mut out_count)?;
        samples.truncate(out_count as usize * sample_size);
        Ok(samples)
    }
}

impl BlackmagicRawPipelineDevice {
    pub fn supported_resource_formats(&self) -> Result<Vec<BlackmagicRawResourceFormat>, BrawError> {
        let mut count = 0;
        self.raw.GetSupportedResourceFormats(std::ptr::null_mut(), &mut count)?;
        if count == 0 {
            return Ok(vec![]);
        }
        let mut vec = vec![BlackmagicRawResourceFormat::Null; count as usize];
        self.raw.GetSupportedResourceFormats(vec.as_mut_ptr(), &mut count)?;
        vec.truncate(count as usize);
        Ok(vec)
    }
}

impl BlackmagicRawClipProcessingAttributes {
    pub fn clip_attribute_range(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetClipAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)?;
        Ok((self.factory.lib.variant_to_rust(value_min), self.factory.lib.variant_to_rust(value_max), sdk_bool(is_read_only)))
    }
    pub fn clip_attribute_list(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetClipAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        self.raw.GetClipAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.factory.lib.variant_to_rust(v)).collect();
        Ok((vec, sdk_bool(is_read_only)))
    }
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![0u32; count as usize];
        self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        Ok((vec, sdk_bool(is_read_only)))
    }
}

impl BlackmagicRawFrameProcessingAttributes {
    pub fn frame_attribute_range(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetFrameAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)?;
        Ok((self.factory.lib.variant_to_rust(value_min), self.factory.lib.variant_to_rust(value_max), sdk_bool(is_read_only)))
    }
    pub fn frame_attribute_list(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetFrameAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        self.raw.GetFrameAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.factory.lib.variant_to_rust(v)).collect();
        Ok((vec, sdk_bool(is_read_only)))
    }
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only: SdkBool = Default::default();
        self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], sdk_bool(is_read_only)));
        }
        let mut vec = vec![0u32; count as usize];
        self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
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
    pub fn samples(&self, sample_frame_index: i64, max_sample_count: Option<u32>) -> Result<(Vec<u8>, u32), BrawError> {
        let max_sample_count = max_sample_count.unwrap_or(48000);
        let required_bytes = required_sample_bytes(max_sample_count, self.channel_count()?, self.bit_depth()?)?;
        // `GetAudioSamples`' `bufferSizeBytes` parameter is a `u32`; a buffer it
        // cannot even describe is unusable, so reject rather than truncate.
        let buffer_size_bytes = u32::try_from(required_bytes).map_err(|_| BrawError::InvalidArgument)?;
        let mut buffer: Vec<u8> = vec![0; buffer_size_bytes as usize];
        let mut samples_read: u32 = 0;
        let mut bytes_read: u32 = 0;
        self.raw.GetAudioSamples(sample_frame_index, buffer.as_mut_ptr() as *mut c_void, buffer_size_bytes, max_sample_count, &mut samples_read, &mut bytes_read)?;

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
        self.raw.GetAudioSamples(sample_frame_index, dst.as_mut_ptr() as *mut c_void, buffer_size_bytes, max_sample_count, &mut samples_read, &mut bytes_read)?;
        Ok(samples_read)
    }
}

impl BlackmagicRawClipPDAFData {
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
        self.raw.GetSampleImages(sample_index, left_buffer.as_mut_ptr(), right_buffer.as_mut_ptr(), sample_image_data_size)?;
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