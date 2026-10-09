# braw-rs - Safe, modern Rust bindings for the Blackmagic RAW SDK

Safe, ergonomic, and async-first Rust bindings for **Blackmagic RAW SDK** - no bindgen, no C++ shim, just idiomatic Rust.

<p align="center">
  <a href="https://crates.io/crates/braw"><img alt="crates.io" src="https://img.shields.io/crates/v/braw.svg"></a>
  <a href="https://docs.rs/braw"><img alt="docs.rs" src="https://img.shields.io/docsrs/braw"></a>
  <a href="https://github.com/AdrianEddy/braw-rs/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/AdrianEddy/braw-rs/actions/workflows/ci.yml/badge.svg"></a>
  <a href="#license"><img alt="License: MIT/Apache-2.0" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-informational"></a>
  <a href="#requirements"><img alt="MSRV" src="https://img.shields.io/badge/rustc-1.88+-blue"></a>
  <a href="#requirements"><img alt="Platforms" src="https://img.shields.io/badge/platforms-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20iOS-success"></a>
</p>

## Highlights

* **Pure Rust**: no bindgen, no C/C++; hand-crafted, stable Rust API surface.
* **Dynamic loading**: uses `libloading` to load the SDK at runtime.
* **High-level, safe interfaces** for all Blackmagic RAW APIs.
* **Async jobs, no callbacks**: futures replace C++-style callbacks, making async jobs intuitive.
* **Proper error handling**.
* **Idiomatic iterators & enums**.
* **Async runtime agnostic**: works with any executor; use `pollster` to block when you want simplicity.
* **Native docs**: all structs and enums are documented based on the official SDK documentation pdf
* **Cross-platform**: Windows (x64 and ARM64), Linux, macOS, and iOS.
* **Custom file I/O**: decode — and write — `.braw` clips from memory, `Read + Seek` streams, or any byte source you implement (network, encrypted, …) without ever touching disk.

**Based on Blackmagic RAW SDK 6.0**, which it requires: a library whose codec interface differs from 6.0's — any older one, or a newer one that changes it — is refused with `BrawError::UnsupportedSdkVersion` rather than called through a mismatched interface.

---

## Quick start

### Requirements

* Rust **1.88** or newer.
* The [**Blackmagic RAW SDK**](https://www.blackmagicdesign.com/developer/products/braw/sdk-and-software) **6.0** for your platform.
* Ensure the SDK library is discoverable at runtime:

  * **Windows**: `BlackmagicRawAPI.dll` in the executable dir or on `PATH` — from `Win/Libraries` for x64 builds, `Win/Libraries/ARM64` for ARM64 builds (CPU and OpenCL pipelines; the SDK ships no CUDA decoder for ARM64).
  * **Linux**: `libBlackmagicRawAPI.so` on `LD_LIBRARY_PATH` or `rpath`.
  * **macOS/iOS**: `BlackmagicRawAPI.framework`. `default_library_name()` is a relative path, which `dlopen` resolves through `DYLD_FRAMEWORK_PATH`, the working directory, then `~/Library/Frameworks` and `/Library/Frameworks` — not `@rpath`. For a framework bundled in your app, pass its full path (e.g. `…/Contents/Frameworks/BlackmagicRawAPI.framework/BlackmagicRawAPI`) to `Factory::load_from`.

### Cargo

```toml
[dependencies]
braw = "0.1"
pollster = "1" # optional, for simple blocking
```

### Features

* `serde` *(default)*: implements `serde::Serialize` for `VariantValue`, so clip and frame metadata can be serialized directly.

---

## Example

A concise walkthrough: load the SDK, open a clip, inspect metadata, read & process a frame.

```rust no_run
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
        println!("Instruction set: {:?}", cfgx.instruction_set()?);

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

Once loaded, the library stays loaded until the process exits: the SDK's worker threads may still be running its code after every object is gone.

---

## Safety

The safe API upholds Rust's guarantees by itself, including where the SDK keeps raw pointers without holding a reference:

* **Objects** keep alive what they were created from: a frame its clip, a clip its codec and its file.
* **Jobs** keep alive everything they use until they complete, whether or not their future is still around: dropping a pending future never frees memory a job is still writing.
* **Bitstream buffers** a frame is read into are owned by the frame and by every job that decodes it. A `BitStreamBuffer` has the alignment the SDK requires.
* **Pipeline devices** a codec is configured with, or prepared for, are kept until the codec's destruction has finished: the SDK frees GPU resources made on a device's context at the very end of it.

The raw COM layer — the `raw` field of every object, and `ComPtr`'s methods, one per SDK method — is `unsafe`: its callers uphold the SDK's contracts. So are the few methods that take GPU handles or resources, which Rust cannot check: `BlackmagicRawConfiguration::set_pipeline`, `BlackmagicRaw::prepare_pipeline`, `BlackmagicRawPost3DLUT::resource_gpu`, and the methods of `BlackmagicRawResourceManager`. Prefer the device-based equivalents, which are safe; with your own GPU context, pass whatever owns it to `BlackmagicRaw::keep_alive`, which holds it until the codec's destruction has finished.

---

## Custom file I/O

`BlackmagicRaw::open_clip_from_file` opens a clip through the SDK's `IBlackmagicRawFile` /
`IBlackmagicRawFilesystem` interfaces instead of a path, so a clip can come from memory, a
network source, an encrypted container, or anything else that can serve bytes at an
offset. Writes — a saved sidecar, a trim, a cube file — go through the same interfaces, so
they can stay in memory too.

* `BrawFile` is a byte source: a name, a length, and positional reads (plus writes, for
  outputs). Ready-made: `BytesFile` (any `AsRef<[u8]>`, read without copying),
  `StreamFile` (any `Read + Seek`) and `MemoryFile` (readable and writable).
* `BrawFilesystem` resolves a clip's companion files — its `.sidecar`, the other cards of
  a multi-card recording — and creates the ones the SDK writes. `FileSet` is a set of
  named files that keeps created files in memory; `NoCompanions` has none.
* `BlackmagicRawFile` presents the two to the SDK. A clip opened from it keeps it alive.
* The SDK calls `BrawFile::commit` once on every file it finishes writing — the place
  to finalise or upload it.

The SDK calls these from its worker threads, concurrently, so implementations are
`Send + Sync`. I/O errors convert into `BrawError::Io`, so `?` works on both.

### Decode from memory

```rust no_run
use braw::*;
use std::sync::Arc;

fn main() -> Result<(), BrawError> {
    let braw = Factory::load_from(default_library_name())?;
    let codec = braw.create_codec()?;

    // Bytes from anywhere: an HTTP body, a decrypted buffer, an mmap, …
    let bytes = std::fs::read("A001.braw")?;
    let file = BlackmagicRawFile::standalone(Arc::new(BytesFile::new("A001.braw", bytes)));

    let clip = codec.open_clip_from_file(&file)?;
    println!("{}x{}, {} frames", clip.width()?, clip.height()?, clip.frame_count()?);
    Ok(())
}
```

### Clips with sidecars or multicard parts

The SDK names a clip's companions after the clip (`A001.braw` → `A001.sidecar`) and asks
the filesystem for them while opening the clip:

```rust no_run
use braw::*;
use std::sync::Arc;

fn main() -> Result<(), BrawError> {
    let codec = Factory::load_from(default_library_name())?.create_codec()?;

    // The clip from any `Read + Seek` (a socket, a decrypting reader, …), its sidecar from memory.
    let files = Arc::new(FileSet::new());
    files.insert(Arc::new(StreamFile::new("A001.braw", std::fs::File::open("A001.braw")?)?));
    files.insert(Arc::new(BytesFile::new("A001.sidecar", std::fs::read("A001.sidecar")?)));

    let clip = codec.open_clip_from_file(&BlackmagicRawFile::new(files.get("A001.braw").unwrap(), files.clone()))?;
    assert!(clip.sidecar_file_attached()?);
    Ok(())
}
```

### Writing without touching disk

```rust no_run
use braw::*;
use std::sync::Arc;

fn main() -> Result<(), BrawError> {
    let codec = Factory::load_from(default_library_name())?.create_codec()?;
    let files = Arc::new(FileSet::new());
    files.insert(Arc::new(BytesFile::new("A001.braw", std::fs::read("A001.braw")?)));
    let clip = codec.open_clip_from_file(&BlackmagicRawFile::new(files.get("A001.braw").unwrap(), files.clone()))?;

    // Save an edited sidecar: the SDK creates `A001.sidecar` through the filesystem.
    clip.set_metadata("reel", VariantValue::String("HERO".into()))?;
    clip.save_sidecar_file()?;
    let sidecar_bytes = files.get("A001.sidecar").unwrap().read_to_vec()?;

    // Trim into memory.
    let output = Arc::new(MemoryFile::new("A001_trim.braw"));
    pollster::block_on(clip.trim_to_file(&BlackmagicRawFile::standalone(output.clone()), 0, 24, None, None))?;
    let trimmed_bytes = output.contents();

    println!("sidecar: {} bytes, trim: {} bytes", sidecar_bytes.len(), trimmed_bytes.len());
    Ok(())
}
```

> On **Linux**, the SDK `dlopen`s its decoder plugins (`libDecoder*.so`,
> `libInstructionSetServices*.so`) by bare name at runtime, so the directory holding them
> must be on `LD_LIBRARY_PATH` (the standard Blackmagic deployment mechanism).

---

## FAQ

**Why not use bindgen?**
BRAW SDK is based on the C++ COM object model. If we want to use bindgen we'd need to write a lot of boilerplate in C to make ffi with Rust possible. Instead of doing the boilerplate in C, I decided to just implement COM directly in Rust. This also gives us safer types and easier implementation of idiomatic Rust constructs like async.

**Can I block instead of going async?**
Yes. Use `pollster::block_on` or your runtime’s `block_on`.

**Can I use my own callback?**
Yes. Implement `BrawCallback` for your type and pass it to `codec.set_callback()`. It sees the codec's jobs as they finish, alongside the futures, which complete as usual.

**Can I decode a clip that isn't on disk (in memory, over the network, encrypted)?**
Yes — see [Custom file I/O](#custom-file-io).

**Which SDK version is supported?**
**6.0**. SDK 6.0 changed interfaces in place, so older libraries cannot be driven by these bindings — nor can a later one that changes them again. `Factory::create_codec` checks the codec's interface ID, which changes along with its layout, and refuses a library whose codec is not 6.0's with `BrawError::UnsupportedSdkVersion`, naming the version it found.

---

## TODO

* [ ] More end-to-end examples (transcode, thumbnails, batch decode)
* [ ] Safe methods for ManualDecoderFlow1
* [ ] Safe methods for ManualDecoderFlow2

---

## Contributing

Bug reports and pull requests are welcome. See [CONTRIBUTING.md](https://github.com/AdrianEddy/braw-rs/blob/main/CONTRIBUTING.md) for how to run the test suite against the SDK, and [CHANGELOG.md](https://github.com/AdrianEddy/braw-rs/blob/main/CHANGELOG.md) for what changed in each release.

---

## License

Dual-licensed under **MIT** or **Apache-2.0** at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

> Note: Blackmagic RAW SDK is distributed under its own license/EULA. You must comply with Blackmagic Design’s terms when downloading and redistributing the SDK binaries.

This is an independent project, not affiliated with or endorsed by Blackmagic Design. Blackmagic Design and Blackmagic RAW are trademarks of Blackmagic Design Pty. Ltd.
