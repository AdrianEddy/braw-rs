use braw::*;

fn main() -> Result<(), BrawError> {
    pollster::block_on(async {
        let braw = Factory::load_from(default_library_name())?;
        let codec = braw.create_codec()?;

        let clip = codec.open_clip(&std::env::args().nth(1).unwrap_or_else(|| "E:/A001_06121551_C014.braw".into()))?;

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
