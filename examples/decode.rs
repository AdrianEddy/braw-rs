// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Decode every frame of a clip and print each processed image's size and format.
//!
//! Usage: `cargo run --example decode -- <clip.braw>`

use braw::*;

fn main() -> Result<(), BrawError> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: decode <clip.braw>");
        std::process::exit(2);
    };
    pollster::block_on(async {
        let braw = Factory::load_from(default_library_name())?;
        let codec = braw.create_codec()?;

        let clip = codec.open_clip(&path)?;

        println!("--- Clip metadata ---");
        println!("Width:       {}", clip.width()?);
        println!("Height:      {}", clip.height()?);
        println!("Frame count: {}", clip.frame_count()?);
        println!("Frame rate:  {}", clip.frame_rate()?);
        println!("Timecode(0): {}", clip.timecode_for_frame(0)?);
        println!("Camera type: {}", clip.camera_type()?);

        for i in 0..clip.frame_count()? {
            let frame = clip.read_frame(i).await?;
            let processed = frame.decode_and_process(None, None).await?;

            println!("Frame {i}: {}x{} | {:?} {:?}", processed.width()?, processed.height()?, processed.resource_type()?, processed.resource_format()?);
        }
        Ok(())
    })
}
