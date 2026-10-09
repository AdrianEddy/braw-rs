// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Calls into the methods SDK 6.0 added to existing interfaces — `sdk_layout`
//! proves their vtable slots, this proves their signatures against the real SDK.
//!
//! Same requirements as `custom_io_decode`.

mod common;

use braw::*;
use common::*;
use std::sync::Arc;

fn open_sample(codec: &BlackmagicRaw) -> Result<BlackmagicRawClip, BrawError> {
    codec.open_clip(sample_path().to_str().unwrap())
}

#[test]
fn clip_ex_trims_with_a_frame_step_and_rate() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = open_sample(&codec)?;
    let output = Arc::new(MemoryFile::new("stepped.braw"));
    pollster::block_on(clip.ex()?.trim_to_file(&BlackmagicRawFile::standalone(output.clone()), 0, 1, 1, 25.0, None, None))?;

    let trimmed = codec.open_clip_from_file(&BlackmagicRawFile::standalone(Arc::new(BytesFile::new("stepped.braw", output.contents()))))?;
    assert_eq!(trimmed.frame_count()?, 1);
    assert_eq!(trimmed.frame_rate()?, 25.0, "the trim is retagged to the requested playback rate");
    Ok(())
}

#[test]
fn frame_image_location_is_inside_the_file() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = open_sample(&codec)?;
    let (size, offset, file_index) = clip.ex()?.frame_image_offset_and_size(0)?;
    let file_len = std::fs::metadata(sample_path()).unwrap().len();
    assert_eq!(file_index, 0, "a single-card clip has one file");
    assert!(size > 0 && offset + u64::from(size) <= file_len, "frame 0 spans {offset}+{size} of a {file_len}-byte file");
    Ok(())
}

/// The sidecar / embedded LUT getters reach the right LUTs, and the setters
/// replace them — data marshalled as a float `SAFEARRAY` of RGB points.
#[test]
fn sidecar_and_embedded_luts_are_reachable() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let codec = factory.create_codec()?;
    let clip = open_sample(&codec)?;
    let attributes = clip.clone_clip_processing_attributes()?;

    // The sample carries a 33-point LUT in the clip and a 17-point one in its sidecar.
    assert_eq!(attributes.embedded_post_3d_lut()?.size()?, 33);
    assert_eq!(attributes.sidecar_post_3d_lut()?.size()?, 17);

    let identity: Vec<f32> = (0..17 * 17 * 17)
        .flat_map(|i| [(i % 17) as f32 / 16.0, (i / 17 % 17) as f32 / 16.0, (i / 289) as f32 / 16.0])
        .collect();
    attributes.set_sidecar_post_3d_lut("identity.cube", "Identity", 17, VariantValue::ArrayF32(identity.clone()))?;
    let sidecar = attributes.sidecar_post_3d_lut()?;
    assert_eq!((sidecar.size()?, sidecar.name()?, sidecar.title()?), (17, "identity.cube".into(), "Identity".into()));

    let identity9: Vec<f32> = (0..9 * 9 * 9)
        .flat_map(|i| [(i % 9) as f32 / 8.0, (i / 9 % 9) as f32 / 8.0, (i / 81) as f32 / 8.0])
        .collect();
    attributes.set_embedded_post_3d_lut("identity9.cube", "Identity 9", 9, VariantValue::ArrayF32(identity9))?;
    let embedded = attributes.embedded_post_3d_lut()?;
    assert_eq!((embedded.size()?, embedded.name()?), (9, "identity9.cube".into()));

    // Data of the wrong length is refused, not truncated.
    assert!(attributes.set_sidecar_post_3d_lut("short.cube", "Short", 17, VariantValue::ArrayF32(identity[..17 * 17 * 17].to_vec())).is_err());
    Ok(())
}

#[test]
fn pipeline_devices_report_an_identifier() -> Result<(), BrawError> {
    let Some(factory) = load_sdk()? else { return Ok(()) };
    let mut count = 0;
    for item in factory.pipeline_device_iter(BlackmagicRawPipeline::CPU, BlackmagicRawInterop::None)? {
        let device = item.create_device()?;
        println!("{} ({}): unique identifier {:#x}", device.name()?, device.pipeline_name()?, device.unique_identifier()?);
        count += 1;
    }
    assert!(count > 0, "the CPU pipeline has a device");
    Ok(())
}
