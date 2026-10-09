// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Observe a codec's jobs through a [`BrawCallback`] while awaiting their futures.
//!
//! Usage: `cargo run --example callback -- <clip.braw>`

use braw::*;

struct MyHandler;
#[allow(unused_variables)]
impl BrawCallback for MyHandler  {
    fn read_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) {
        println!("Read complete: job={:?}, result={:#X}, frame={:?}", job, result, frame);
    }
    fn decode_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn process_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT, processed_image: *mut IBlackmagicRawProcessedImage) { }
    fn trim_progress(&self, job: *mut IBlackmagicRawJob, progress: f32) { }
    fn trim_complete(&self, job: *mut IBlackmagicRawJob, result: HRESULT) { }
    fn sidecar_metadata_parse_warning(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }
    fn sidecar_metadata_parse_error(&self, clip: *mut IBlackmagicRawClip, file_name: String, line_number: u32, info: String) { }
    fn prepare_pipeline_complete(&self, user_data: *mut core::ffi::c_void, result: HRESULT) { }
}

fn main() -> Result<(), BrawError> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: callback <clip.braw>");
        std::process::exit(2);
    };
    pollster::block_on(async {
        let braw = Factory::load_from(default_library_name())?;
        let codec = braw.create_codec()?;
        codec.set_callback(MyHandler)?;

        let clip = codec.open_clip(&path)?;

        let frame = clip.read_frame(0).await?;
        println!("Frame {}", frame.frame_index()?);

        Ok(())
    })
}
