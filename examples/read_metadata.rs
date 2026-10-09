// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Print a clip's properties and metadata, and the metadata of its first frame.
//!
//! Usage: `cargo run --example read_metadata -- <clip.braw>`

use braw::*;

fn main() -> Result<(), BrawError> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: read_metadata <clip.braw>");
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
        for (key, value) in clip.metadata_iter()? {
            println!("{: <30}{:?}", key, value);
        }

        if clip.frame_count()? > 0 {
            println!("\n# Frame 0 metadata:");
            let frame = clip.read_frame(0).await?;
            for (key, value) in frame.metadata_iter()? {
                println!("- {: <30}{:?}", key, value);
            }
        }
        Ok(())
    })
}
