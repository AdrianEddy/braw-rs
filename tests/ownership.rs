// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The SDK keeps raw pointers to memory and devices it is given — a frame's
//! bitstream buffer, a pipeline device's context and command queue — without
//! holding them. These drive the real SDK through the safe API to show that the
//! bindings hold them instead, for as long as anything can use them.
//!
//! Same requirements as `custom_io_decode`.

mod common;

use braw::*;
use common::*;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{ AtomicUsize, Ordering };
use std::task::{ Context, Poll, Waker };

/// A bitstream buffer counting its drops.
struct Counted(BitStreamBuffer, Arc<AtomicUsize>);
impl AsMut<[u8]> for Counted {
    fn as_mut(&mut self) -> &mut [u8] { &mut self.0 }
}
impl Drop for Counted {
    fn drop(&mut self) { self.1.fetch_add(1, Ordering::SeqCst); }
}

#[test]
fn a_frame_read_into_a_buffer_owns_it_and_decodes_from_it() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    let expected = pollster::block_on(decode_hash(&clip, 0))?;

    let ex = clip.ex()?;
    let dropped = Arc::new(AtomicUsize::new(0));
    let buffer = Counted(BitStreamBuffer::new(ex.max_bit_stream_size_bytes()? as usize), dropped.clone());
    let frame = pollster::block_on(ex.read_frame(0, buffer))?;
    assert_eq!(dropped.load(Ordering::SeqCst), 0, "the frame holds its buffer");

    let processed = pollster::block_on(frame.decode_and_process(None, None))?;
    assert_eq!(fnv1a(processed.resource_cpu()?), expected, "the frame decodes from its buffer like a path-opened read");

    drop((frame, processed));
    codec.flush_jobs()?;
    drop((ex, clip, codec));
    assert_eq!(dropped.load(Ordering::SeqCst), 1, "the buffer goes with the last user");
    Ok(())
}

#[test]
fn a_dropped_read_future_leaves_its_buffer_to_the_job() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    let ex = clip.ex()?;
    let dropped = Arc::new(AtomicUsize::new(0));
    let size = ex.max_bit_stream_size_bytes()? as usize;

    const READS: usize = 16;
    for _ in 0..READS {
        // Submit the read, then give up on it while the SDK may still be writing.
        let mut read = Box::pin(ex.read_frame(0, Counted(BitStreamBuffer::new(size), dropped.clone())));
        if let Poll::Ready(frame) = read.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
            drop(frame?);
        }
        drop(read);
    }
    codec.flush_jobs()?;
    drop((ex, clip, codec));
    assert_eq!(dropped.load(Ordering::SeqCst), READS, "every buffer is freed, once, when its job is done");
    Ok(())
}

/// The first device of `pipeline`, if this machine has one the SDK can use.
fn gpu_device(factory: &Factory, pipeline: BlackmagicRawPipeline) -> Result<Option<BlackmagicRawPipelineDevice>, BrawError> {
    let Ok(mut devices) = factory.pipeline_device_iter(pipeline, BlackmagicRawInterop::None) else { return Ok(None) };
    match devices.next() {
        Some(item) => Ok(item.create_device().ok()),
        None => Ok(None),
    }
}

/// A CUDA codec frees the GPU resources it made on the device's context only after
/// releasing its callback, late in its destruction: the device must outlive all of
/// it, whichever of the caller's objects goes last — or a job still in flight.
#[test]
fn a_gpu_device_outlives_the_codec_destruction() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    if gpu_device(&factory, BlackmagicRawPipeline::CUDA)?.is_none() {
        eprintln!("skipped: no CUDA device");
        return Ok(());
    }
    for order in 0..3 {
        let codec = factory.create_codec()?;
        // A device of its own, which the caller lets go of at once.
        let device = gpu_device(&factory, BlackmagicRawPipeline::CUDA)?.expect("a CUDA device");
        codec.configuration()?.set_from_device(&device)?;
        drop(device);
        let clip = codec.open_clip(sample_path().to_str().unwrap())?;
        let frame = pollster::block_on(clip.read_frame(0))?;
        let processed = pollster::block_on(frame.decode_and_process(None, None))?;
        assert_eq!(processed.resource_type()?, BlackmagicRawResourceType::BufferCUDA);
        match order {
            // The codec goes with the caller's last object.
            0 => drop((processed, frame, codec, clip)),
            1 => drop((codec, clip, frame, processed)),
            // ... or with a job still in flight, its completion callback the last to let go.
            _ => {
                let pending = frame.create_decode_process_future(None, None)?;
                drop((pending, processed, frame, clip, codec));
            }
        }
    }
    // The last codec may still be on its way out, on a thread of its own.
    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

#[test]
fn the_codec_keeps_what_it_is_given_until_it_is_destroyed() -> Result<(), BrawError> {
    struct Probe(Arc<AtomicUsize>);
    impl Drop for Probe {
        fn drop(&mut self) { self.0.fetch_add(1, Ordering::SeqCst); }
    }
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let dropped = Arc::new(AtomicUsize::new(0));
    let codec = factory.create_codec()?;
    codec.keep_alive(Probe(dropped.clone()));
    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    drop(codec);
    pollster::block_on(decode_hash(&clip, 0))?;
    assert_eq!(dropped.load(Ordering::SeqCst), 0, "the clip still holds the codec");
    drop(clip);
    // The last of the jobs' completion states may still be on its way out on an SDK
    // thread, taking the codec — and then the probe — with it.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while dropped.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(dropped.load(Ordering::SeqCst), 1, "released after the codec, once");
    Ok(())
}

#[test]
fn the_codec_keeps_the_device_it_is_configured_with() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let device = factory
        .pipeline_device_iter(BlackmagicRawPipeline::CPU, BlackmagicRawInterop::None)?
        .next()
        .expect("the CPU pipeline has a device")
        .create_device()?;
    codec.configuration()?.set_from_device(&device)?;
    pollster::block_on(codec.prepare_pipeline_for_device(&device)?)?;
    // The caller's handle goes; the codec still decodes with the device.
    drop(device);

    let clip = codec.open_clip(sample_path().to_str().unwrap())?;
    pollster::block_on(decode_hash(&clip, 0))?;
    Ok(())
}
