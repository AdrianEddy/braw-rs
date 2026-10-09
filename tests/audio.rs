// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The asynchronous read-audio job (`CreateJobReadAudio` → `ReadAudioComplete`)
//! delivers the same samples as the synchronous `IBlackmagicRawClipAudio` read.
//!
//! The SDK's own sample clip has no audio track, so this needs a clip that has
//! one: point `BRAW_AUDIO_CLIP` at it and run with `--ignored`. Same SDK
//! requirements as `custom_io_decode`.

mod common;

use braw::*;
use common::*;

#[test]
#[ignore = "needs BRAW_AUDIO_CLIP pointing at a .braw clip with an audio track"]
fn read_audio_job_matches_the_synchronous_read() -> Result<(), BrawError> {
    let clip_path = std::env::var("BRAW_AUDIO_CLIP").expect("BRAW_AUDIO_CLIP");
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = codec.open_clip(&clip_path)?;
    let audio = clip.audio()?;
    let (channels, bit_depth, total) = (audio.channel_count()?, audio.bit_depth()?, audio.sample_count()?);
    println!("audio: {channels} ch, {bit_depth} bit, {} Hz, {total} samples", audio.sample_rate()?);
    assert!(total > 0, "the clip must carry audio");

    let start = total / 3;
    let count = 4800u32.min((total - start) as u32);
    let (sync_bytes, sync_read) = audio.samples(start as i64, Some(count))?;

    let buffer = pollster::block_on(clip.read_audio(start, u64::from(count)))?;
    assert_eq!((buffer.channel_count()?, buffer.bit_depth()?, buffer.sample_rate()?), (channels, bit_depth, audio.sample_rate()?));
    assert_eq!(buffer.format()?, BlackmagicRawAudioFormat::PCMLittleEndian);
    let frame_bytes = (channels * bit_depth / 8) as usize;
    let job_bytes = buffer.samples()?;
    println!("sync read {sync_read} sample frames, job read {} bytes (GetAudioSampleCount = {})", job_bytes.len(), buffer.sample_count()?);
    assert_eq!(job_bytes.len(), count as usize * frame_bytes, "the job reads exactly the requested sample frames");
    assert_eq!(sync_read, count, "the synchronous read returns the requested sample frames");
    assert_eq!(job_bytes, &sync_bytes[..], "both reads return the same PCM");
    Ok(())
}
