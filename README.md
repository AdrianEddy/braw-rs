# braw-rs — Safe, modern Rust bindings for the Blackmagic RAW SDK

Safe, ergonomic, and async-first Rust bindings for **Blackmagic RAW SDK** — no bindgen, no `extern "C"`, just idiomatic Rust.

<p align="center">
  <a href="#"><img alt="License: MIT/Apache-2.0" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-informational"></a>
  <a href="#"><img alt="Rust Edition" src="https://img.shields.io/badge/rust-Edition_2024-blue"></a>
  <a href="#"><img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20iOS-success"></a>
</p>

## Highlights

* **Pure Rust**: no bindgen, no C/C++; hand-crafted, stable Rust API surface.
* **Dynamic loading**: uses `libloading` to load the SDK at runtime.
* **High-level, safe interfaces** for all Blackmagic RAW APIs.
* **Async jobs, no callbacks**: futures replace C++-style callbacks, making async jobs intuitive.
* **Proper error handling**.
* **Idiomatic iterators & enums**.
* **Async-runtime agnostic**: works with any executor; use `pollster` to block when you want simplicity.
* **Native docs**: all structs and enums are documented based on the official SDK documentation pdf
* **Cross-platform**: Windows, Linux, macOS, and iOS.

**Based on Blackmagic RAW SDK 5.0.0**.

---

## Quick start

### Requirements

* Install **Blackmagic RAW SDK** for your platform.
* Ensure the SDK library is discoverable at runtime:

  * **Windows**: `BlackmagicRAWAPI.dll` in the executable dir or on `PATH`.
  * **Linux**: `libBlackmagicRAWAPI.so` on `LD_LIBRARY_PATH` or `rpath`.
  * **macOS/iOS**: `libBlackmagicRAWAPI.dylib` in `@rpath` or `Frameworks`.

### Cargo

```toml
[dependencies]
braw = "0.1"
pollster = "0.3" # optional, for simple blocking
```

---

## Example

A concise walkthrough: load the SDK, open a clip, inspect metadata, read & process a frame.

```rust
use braw::*;

fn main() -> Result<(), BrawError> {
    pollster::block_on(async {
        // Load the SDK dynamically via libloading
        let braw = Factory::load_from(default_library_name())?;

        // Create the codec and inspect configuration
        let codec = braw.create_codec()?;

        let cfg = codec.configuration()?;
        let cfgx = codec.configuration_ex()?;
        println!("Camera support version: {}", cfg.camera_support_version()?);
        println!("CPU Threads: {}", cfg.cpu_threads()?);
        println!("Instruction set: ", cfgx.instruction_set()?);

        // Open a .braw clip
        let clip = codec.open_clip("sample.braw")?;

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

        // Read and process the first frame asynchronously
        if clip.frame_count()? > 0 {
            let frame = clip.read_frame(0).await?;
            let processed = frame.decode_and_process(None, None).await?;
            println!(
                "Frame 0: {}x{} | {:?} {:?}",
                processed.width()?, processed.height()?,
                processed.resource_type()?, processed.resource_format()?
            );
        }

        Ok(())
    })
}
```

---

## Runtime loading details

This crate **does not link** to the SDK at build time. At runtime it uses `libloading` to locate and bind the entry points. This means:

* Your app can start even if the SDK isn’t present, and you can show a friendly error.
* You can ship a single binary and place the SDK library alongside it or bundle it per-platform.

---

## FAQ

**Why not use bindgen?**
The BRAW SDK is based on C++ COM object model. Using bindgen means writing a lot of boilerplate in C to make ffi with Rust happy. Instead of doing the boilerplate in C, I decided to just implement it directly in Rust. This also gives us safer types and easier implementation of idiomatic Rust constructs like async.

**Can I block instead of going async?**
Yes. Use `pollster::block_on` or your runtime’s `block_on`.

**Can I use my own callback?**
Yes. Use `codec.set_callback()` and implement `BrawCallback` for your type.

**Which SDK version is supported?**
**5.0.0**. Other versions may work but are not guaranteed.

---

## TODO

* [ ] More end-to-end examples (transcode, thumbnails, batch decode)
* [ ] Safe methods for ManualDecoderFlow1
* [ ] Safe methods for ManualDecoderFlow2

---

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

> Note: Blackmagic RAW SDK is distributed under its own license/EULA. You must comply with Blackmagic Design’s terms when downloading and redistributing the SDK binaries.
