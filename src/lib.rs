#![allow(non_snake_case)]

#![doc = include_str!("../README.md")]

use core::ffi::c_void;
use std::sync::Arc;

mod callback; pub use callback::*;
mod com;      pub use com::*;
mod error;    pub use error::*;
mod future;   pub use future::*;
mod iterators;pub use iterators::*;
mod os;       pub use os::*;
mod sdk;      pub use sdk::*;
mod string;   pub use string::*;
mod variant;  pub use variant::*;

pub fn default_library_name() -> &'static str {
    #[cfg(target_os = "windows")] { "BlackmagicRawAPI.dll" }
    #[cfg(any(target_os = "macos", target_os = "ios"))] { "BlackmagicRawAPI.framework/BlackmagicRawAPI" }
    #[cfg(target_os = "linux")] { "libBlackmagicRawAPI.so" }
}

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
    pub fn load<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, libloading::Error> {
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
pub struct Factory {
    factory: ComPtr<IBlackmagicRawFactory>,
    // This must be last
    lib: RawLibrary
}

impl Factory {
    pub fn load_from(path: impl AsRef<std::path::Path>) -> Result<Self, BrawError> {
        let lib = RawLibrary::load(path.as_ref())?;
        let factory = lib.create_factory()?;
        Ok(Self { lib, factory })
    }

    /// Create a codec from the factory
    pub fn create_codec<'lib>(&'lib self) -> Result<BlackmagicRaw<'lib>, BrawError> {
        let raw = braw_out_ptr!(|pp| self.factory.CreateCodec(pp));

        let handler = CallbackHandle::new(DefaultCallback::default());
        let codec = BlackmagicRaw {
            raw,
            lib: &self.lib,
            callback: handler,
        };
        let _ = codec.raw.SetCallback(codec.callback.as_mut_ptr())?;

        Ok(codec)
    }

    /// Create a pipeline iterator
    pub fn pipeline_iter<'lib>(&'lib self, interop: BlackmagicRawInterop) -> Result<PipelineIterator<'lib>, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.factory.CreatePipelineIterator(interop, &mut out) {
            Ok(_)  => Ok(PipelineIterator { raw: ComPtr::new(out)?, lib: &self.lib, is_first: true }),
            Err(e) => Err(e),
        }
    }
    /// Create a pipeline device iterator
    pub fn pipeline_device_iter<'lib>(&'lib self, pipeline: BlackmagicRawPipeline, interop: BlackmagicRawInterop) -> Result<PipelineDeviceIterator<'lib>, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.factory.CreatePipelineDeviceIterator(pipeline, interop, &mut out) {
            Ok(_)  => Ok(PipelineDeviceIterator { raw: ComPtr::new(out)?, lib: &self.lib, is_first: true, current_index: Arc::new(std::sync::atomic::AtomicUsize::new(0)) }),
            Err(e) => Err(e),
        }
    }
    /// Create empty clip geometry object
    pub fn create_clip_geometry<'lib>(&'lib self) -> Result<BlackmagicRawClipGeometry<'lib>, BrawError> {
        let geom = braw_out_ptr!(|pp| self.factory.CreateClipGeometry(pp));
        Ok(BlackmagicRawClipGeometry { raw: geom, lib: &self.lib })
    }
}

impl<'lib> BlackmagicRaw<'lib> {
    pub fn open_clip<'codec>(&'codec self, path: &str) -> Result<BlackmagicRawClip<'codec>, BrawError> {
        let in_str = BrawString::from(path);
        let clip = braw_out_ptr!(|pp| self.raw.OpenClip(in_str.as_raw(), pp));
        Ok(BlackmagicRawClip { raw: clip, lib: self.lib })
    }
    pub fn open_clip_with_geometry<'codec>(&'codec self, path: &str, geometry: BlackmagicRawClipGeometry) -> Result<BlackmagicRawClip<'codec>, BrawError> {
        let in_str = BrawString::from(path);
        let clip = braw_out_ptr!(|pp| self.raw.OpenClipWithGeometry(in_str.as_raw(), geometry.as_raw(), pp));
        Ok(BlackmagicRawClip { raw: clip, lib: self.lib })
    }

    pub fn set_callback<T: BrawCallback>(&mut self, callback: T) -> Result<(), BrawError> {
        let dcb = unsafe { &mut *(self.callback.as_mut_ptr() as *mut CallbackBox<DefaultCallback>) };
        dcb.state.user_callback = Some(Box::new(callback));
        Ok(())
    }

    /// Asynchronously prepares the current pipeline
    pub async fn prepare_pipeline(&self, pipeline: u32, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void) -> Result<(), BrawError> {
        let state = std::sync::Arc::new(State::<()>::new());

        let _ = self.raw.PreparePipeline(pipeline, pipeline_context, pipeline_command_queue, Arc::as_ptr(&state) as *mut c_void)?;

        CallbackFuture { state, job: None }.await
    }

    /// Asynchronously prepares the current pipeline
    pub async fn prepare_pipeline_for_device(&self, device: BlackmagicRawPipelineDevice<'lib>) -> Result<(), BrawError> {
        let state = std::sync::Arc::new(State::<()>::new());
        let _ = self.raw.PreparePipelineForDevice(device.as_raw(), Arc::as_ptr(&state) as *mut c_void)?;
        CallbackFuture { state, job: None }.await
    }
}

impl<'lib> BlackmagicRawClip<'lib> {
    pub fn metadata_iter(&self) -> Result<MetadataIterator<'lib>, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.raw.GetMetadataIterator(&mut out) {
            Ok(_)  => Ok(MetadataIterator { raw: ComPtr::new(out)?, lib: self.lib, is_first: true }),
            Err(e) => Err(e),
        }
    }

    pub async fn read_frame(&self, frame_index: u64) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_with_hints(frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(frame_index, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
    pub async fn trim(&self, file_name: &str, frame_index: u64, frame_count: u64, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes<'lib>>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes<'lib>>) -> Result<(), BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobTrim(
            file_name.as_ptr() as *const _,
            frame_index,
            frame_count,
            clip_processing_attributes.map_or(std::ptr::null_mut(), |f| f.as_raw()),
            frame_processing_attributes.map_or(std::ptr::null_mut(), |f| f.as_raw()),
            &mut job_ptr
        )?;

        CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await
    }
}

impl<'lib> BlackmagicRawClipEx<'lib> {
    pub async fn read_frame(&self, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_with_hints(frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_with_hints(&self, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
}

impl<'lib> BlackmagicRawFrame<'lib> {
    pub fn metadata_iter(&self) -> Result<MetadataIterator<'lib>, BrawError> {
        let mut out = std::ptr::null_mut();
        match self.raw.GetMetadataIterator(&mut out) {
            Ok(_)  => Ok(MetadataIterator { raw: ComPtr::new(out)?, lib: self.lib, is_first: true }),
            Err(e) => Err(e),
        }
    }
    pub async fn decode_and_process(&self, clip_processing_attributes: Option<BlackmagicRawClipProcessingAttributes<'lib>>, frame_processing_attributes: Option<BlackmagicRawFrameProcessingAttributes<'lib>>) -> Result<BlackmagicRawProcessedImage<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobDecodeAndProcessFrame(clip_processing_attributes.map_or(std::ptr::null_mut(), |f| f.as_raw()), frame_processing_attributes.map_or(std::ptr::null_mut(), |f| f.as_raw()), &mut job_ptr)?;

        let image: ComPtr<IBlackmagicRawProcessedImage> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, &[])?.await?;
        Ok(BlackmagicRawProcessedImage { raw: image, lib: self.lib })
    }
}

impl<'lib> BlackmagicRawClipMultiVideo<'lib> {
    pub async fn read_frame(&self, track_index: u32, frame_index: u64) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_with_hints(track_index, frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, track_index: u32, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrame(track_index, frame_index, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
    pub async fn read_frame_ex(&self, track_index: u32, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_ex_with_hints(track_index, frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_ex_with_hints(&self, track_index: u32, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobReadFrameEx(track_index, frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
}

impl<'lib> BlackmagicRawClipImmersiveVideo<'lib> {
    pub async fn read_frame(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_with_hints(video_track, frame_index, &[]).await
    }
    pub async fn read_frame_with_hints(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobImmersiveReadFrame(video_track, frame_index, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
    pub async fn read_frame_ex(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: &[u8]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        self.read_frame_ex_with_hints(video_track, frame_index, bit_stream, &[]).await
    }
    pub async fn read_frame_ex_with_hints(&self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64, bit_stream: &[u8], hints: &[ReadJobHints]) -> Result<BlackmagicRawFrame<'lib>, BrawError> {
        let mut job_ptr = std::ptr::null_mut();
        self.raw.CreateJobImmersiveReadFrameEx(video_track, frame_index, bit_stream.as_ptr() as *const _ as *mut _, bit_stream.len() as u32, &mut job_ptr)?;

        let frame: ComPtr<IBlackmagicRawFrame> = CallbackFuture::create_from_job(ComPtr::new(job_ptr)?, hints)?.await?;
        Ok(BlackmagicRawFrame { raw: frame, lib: self.lib })
    }
}

impl<'lib> BlackmagicRawProcessedImage<'lib> {
    pub fn resource(&self) -> Result<&[u8], BrawError> {
        unsafe {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            let size = self.resource_size_bytes()? as usize;
            let _ = self.raw.GetResource(&mut ptr)?;
            if ptr.is_null() || size == 0 {
                return Err(BrawError::NullValue);
            }
            Ok(std::slice::from_raw_parts(ptr as *const u8, size))
        }
    }
}

impl<'lib> BlackmagicRawPost3DLUT<'lib> {
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

impl<'lib> BlackmagicRawToneCurve<'lib> {
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

impl<'lib> BlackmagicRawClipAccelerometerMotion<'lib> {
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

impl<'lib> BlackmagicRawClipGyroscopeMotion<'lib> {
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

impl<'lib> BlackmagicRawPipelineDevice<'lib> {
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

impl<'lib> BlackmagicRawClipProcessingAttributes<'lib> {
    pub fn clip_attribute_range(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only = false;
        self.raw.GetClipAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)?;
        Ok((self.lib.variant_to_rust(value_min), self.lib.variant_to_rust(value_max), is_read_only))
    }
    pub fn clip_attribute_list(&self, attribute: BlackmagicRawClipProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only = false;
        self.raw.GetClipAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], is_read_only));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        self.raw.GetClipAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.lib.variant_to_rust(v)).collect();
        Ok((vec, is_read_only))
    }
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only = false;
        self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], is_read_only));
        }
        let mut vec = vec![0u32; count as usize];
        self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        Ok((vec, is_read_only))
    }
}

impl<'lib> BlackmagicRawFrameProcessingAttributes<'lib> {
    pub fn frame_attribute_range(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(VariantValue, VariantValue, bool), BrawError> {
        let mut value_min = VARIANT::default();
        let mut value_max = VARIANT::default();
        let mut is_read_only = false;
        self.raw.GetFrameAttributeRange(attribute, &mut value_min, &mut value_max, &mut is_read_only)?;
        Ok((self.lib.variant_to_rust(value_min), self.lib.variant_to_rust(value_max), is_read_only))
    }
    pub fn frame_attribute_list(&self, attribute: BlackmagicRawFrameProcessingAttribute) -> Result<(Vec<VariantValue>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only = false;
        self.raw.GetFrameAttributeList(attribute, std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], is_read_only));
        }
        let mut vec = vec![VARIANT::default(); count as usize];
        self.raw.GetFrameAttributeList(attribute, vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        let vec = vec.into_iter().map(|v| self.lib.variant_to_rust(v)).collect();
        Ok((vec, is_read_only))
    }
    pub fn iso_list(&self) -> Result<(Vec<u32>, bool), BrawError> {
        let mut count = 0;
        let mut is_read_only = false;
        self.raw.GetISOList(std::ptr::null_mut(), &mut count, &mut is_read_only)?;
        if count == 0 {
            return Ok((vec![], is_read_only));
        }
        let mut vec = vec![0u32; count as usize];
        self.raw.GetISOList(vec.as_mut_ptr(), &mut count, &mut is_read_only)?;
        vec.truncate(count as usize);
        Ok((vec, is_read_only))
    }
}

impl<'lib> BlackmagicRawClipAudio<'lib> {
    pub fn audio_samples(&self, sample_frame_index: i64, max_sample_count: Option<u32>) -> Result<(Vec<u8>, u32), BrawError> {
        let max_sample_count = max_sample_count.unwrap_or(48000);
        let buffer_size_bytes = (max_sample_count * self.audio_channel_count()? * self.audio_bit_depth()?) / 8;
        let mut buffer: Vec<u8> = vec![0; buffer_size_bytes as usize];
        let mut samples_read: u32 = 0;
        let mut bytes_read: u32 = 0;
        self.raw.GetAudioSamples(sample_frame_index, buffer.as_mut_ptr() as *mut c_void, buffer_size_bytes, max_sample_count, &mut samples_read, &mut bytes_read)?;

        buffer.truncate(bytes_read as usize);
        Ok((buffer, samples_read))
    }
}

impl<'lib> BlackmagicRawClipPDAFData<'lib> {
    pub fn get_sample_images(&self, sample_index: u64) -> Result<(Vec<u8>, Vec<u8>), BrawError> {
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