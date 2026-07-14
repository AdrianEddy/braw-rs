# braw-rs - Safe, modern Rust bindings for the Blackmagic RAW SDK

Safe, ergonomic, and async-first Rust bindings for **Blackmagic RAW SDK** - no bindgen, no ffi, just idiomatic Rust.

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
* **Async runtime agnostic**: works with any executor; use `pollster` to block when you want simplicity.
* **Native docs**: all structs and enums are documented based on the official SDK documentation pdf
* **Cross-platform**: Windows, Linux, macOS, and iOS.
* **Virtual files** *(optional `hookfs` feature)*: decode — and write — `.braw` clips from arbitrary `Read + Seek` streams (in-memory, network, encrypted, …) without ever touching disk.

**Based on Blackmagic RAW SDK 5.0.0**.

---

## Quick start

### Requirements

* Install **Blackmagic RAW SDK** for your platform.
* Ensure the SDK library is discoverable at runtime:

  * **Windows**: `BlackmagicRawAPI.dll` in the executable dir or on `PATH`.
  * **Linux**: `libBlackmagicRawAPI.so` on `LD_LIBRARY_PATH` or `rpath`.
  * **macOS/iOS**: `BlackmagicRawAPI.framework` in `@rpath` or `Frameworks`.

### Cargo

```toml
[dependencies]
braw = "0.1"
pollster = "0.3" # optional, for simple blocking
```

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

---

## Virtual files & streaming (`hookfs`)

The Blackmagic RAW SDK exposes clip access **only by path string** — there is no
`IStream` / open-from-buffer hook. The optional **`hookfs`** feature bridges that gap:
it hands the SDK a *synthetic* path and transparently intercepts the file-I/O calls the
SDK makes against it, servicing them from a stream **you** provide. The synthetic path
never exists on disk.

This lets you decode `.braw` clips straight from memory, a network source, an encrypted
blob, an archive, or any other `Read + Seek` stream — no temp files, no copies. Writes
(sidecar save/reload, `CreateJobTrim` output) are serviced entirely from an in-memory
VFS too, so they never hit the physical filesystem either.

Available on **Windows, Linux, macOS, and iOS** — the same platforms as the SDK
bindings. The hooking engine patches the SDK's file I/O on each platform's native
binary format (PE imports on Windows, ELF/Mach-O on Linux/Apple).

### Enable it

```toml
[dependencies]
braw = { version = "0.1", features = ["hookfs"] }
```

### Decode from an in-memory stream

The simplest case — one clip, one stream:

```rust ignore
use braw::*;
use std::io::Cursor;

fn main() -> Result<(), BrawError> {
    let braw = Factory::load_from(default_library_name())?;
    let codec = braw.create_codec()?;

    // Bytes from anywhere: an HTTP body, a decrypted buffer, an mmap, …
    let bytes = std::fs::read("A001.braw").unwrap();

    // `open_clip_from` mounts the stream under a synthetic path and opens it.
    // The returned `VirtualClip` derefs to `BlackmagicRawClip`, so every clip
    // method is available directly.
    let clip = codec.open_clip_from("A001.braw", Cursor::new(bytes))?;
    println!("{}x{}, {} frames", clip.width()?, clip.height()?, clip.frame_count()?);
    Ok(())
}
```

### Clips with sidecars or multicard parts

A clip's `.sidecar` is read during `OpenClip`, and spanned/multicard clips reference
sibling files. Mount every sibling with the builder **before** calling `open()`:

```rust ignore
use braw::*;
use std::io::Cursor;

let clip = codec
    .virtual_clip("A001.braw")?
    .file("A001.braw",    Cursor::new(braw_bytes))?
    .file("A001.sidecar", Cursor::new(sidecar_bytes))?  // read during OpenClip
    .file("A001_2.braw",  Cursor::new(part2_bytes))?    // multicard / spanned sibling
    .open()?;
```

Every sibling is namespaced under a unique per-clip directory, so opening two clips with
the same logical name (e.g. two cards' shared `A001.braw`) never collides, and they can
be decoded concurrently on separate threads.

### Writable output — no disk

Reserve an in-memory writable file, let the SDK write to it (sidecar re-save, a trim
output), then read the bytes back — all without touching disk:

```rust ignore
let clip = codec
    .virtual_clip("A001.braw")?
    .file("A001.braw", Cursor::new(braw_bytes))?
    .open()?;

// Edit metadata and save the sidecar into the virtual FS.
clip.set_metadata("reel", VariantValue::String("HERO".into()))?;
clip.save_sidecar_file()?;

// Read the saved sidecar bytes straight back from memory — never touching disk.
let sidecar_bytes = clip.read_virtual_path(&clip.sidecar_path());
```

### How it works

`hookfs` installs its hooks into the loaded SDK module **once per process**, discovering
the module by the address of a known SDK export (never a fragile basename) so late-loaded
decoder plugins are patched too. Hooks persist for the process lifetime; mounts come and
go with your `VirtualClip`s. See `src/virtualfs.rs` for the full API surface
(`Factory::enable_virtual_files`, `BlackmagicRaw::virtual_clip` / `open_clip_from`,
`VirtualClip`, `VirtualClipBuilder`).

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
Yes. Use `codec.set_callback()` and implement `BrawCallback` for your type.

**Can I decode a clip that isn't on disk (in memory, over the network, encrypted)?**
Yes — enable the `hookfs` feature and use `open_clip_from` / `virtual_clip`. See
[Virtual files & streaming](#virtual-files--streaming-hookfs).

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
