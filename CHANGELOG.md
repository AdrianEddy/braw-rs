# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Initial release, based on Blackmagic RAW SDK 6.0.

### Added

- Safe bindings for the Blackmagic RAW SDK 6.0 interfaces, written in Rust with no
  bindgen or C++ shim. The SDK library is loaded at runtime, so applications build
  without it and can report its absence.
- Async jobs: reading, decoding and processing frames, reading audio, trimming and
  preparing pipelines return futures that work with any executor.
  `create_read_frame_future` and `create_decode_process_future` return `'static`
  futures for keeping many jobs in flight.
- Custom file I/O: `BrawFile` and `BrawFilesystem`, with the ready-made `BytesFile`,
  `StreamFile`, `MemoryFile`, `FileSet` and `NoCompanions`, open clips from memory or
  any `Read + Seek` source and write sidecars, trims and cube files without
  touching disk.
- Reading frames into caller-provided buffers (`BlackmagicRawClipEx::read_frame` and
  the multi-video and immersive equivalents), which the frame then owns, and
  `BitStreamBuffer`, laid out as the SDK requires.
- A safe API that keeps alive everything the SDK uses without holding it: jobs hold
  what they use until they complete, even when their future is dropped, and a codec
  keeps its pipeline devices — and, through `BlackmagicRaw::keep_alive`, whatever
  owns a GPU context it was given — until its destruction has finished. The raw COM
  layer, and the few methods taking GPU handles or resources, are `unsafe`.
- `BrawCallback`, set per codec with `BlackmagicRaw::set_callback`, to observe jobs as
  they finish.
- `Factory::create_codec` refuses an SDK library whose codec interface is not
  6.0's with `BrawError::UnsupportedSdkVersion` instead of calling through a
  mismatched interface.
- `serde` feature (enabled by default) implementing `Serialize` for `VariantValue`.
- Support for Windows (x64 and ARM64), Linux, macOS and iOS.

[Unreleased]: https://github.com/AdrianEddy/braw-rs/commits/main
