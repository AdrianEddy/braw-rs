# Contributing

Bug reports and pull requests are welcome. For anything larger than a fix, please
open an issue first to discuss the change.

## Building

Rust 1.88 or newer. The crate loads the Blackmagic RAW SDK at runtime, so it builds
without the SDK installed.

## Testing

```sh
cargo test --all-features
```

runs the unit tests, which need no SDK, and the integration tests, which drive the
real SDK and pass without running when it is not there (`-- --nocapture` shows
which). To run those too:

1. Download the [Blackmagic RAW SDK 6.0](https://www.blackmagicdesign.com/developer/products/braw/sdk-and-software).
2. Unpack it into `sdk/` at the repository root (it is ignored by git), so that
   `sdk/` holds the SDK's `Win/`, `Linux/`, `Mac/` and `Media/` directories — or point
   `BRAW_SDK_DIR` at an SDK unpacked elsewhere. `BRAW_SDK_LIBRARY` overrides the path
   of the library itself.
3. On Linux, put `sdk/Linux/Libraries` on `LD_LIBRARY_PATH`: the SDK loads its
   decoder plugins from there by name. The library also needs `libGL.so.1` and
   `libX11.so.6`.

Once `BRAW_SDK_DIR` is set, a missing SDK file fails the tests instead of skipping
them; set `BRAW_REQUIRE_SDK=1` for the same with the SDK in `sdk/`.

Two tests need media or libraries the SDK does not ship and are ignored by default:

```sh
BRAW_AUDIO_CLIP=/path/to/clip-with-audio.braw cargo test --test audio -- --ignored
BRAW_LEGACY_SDK_LIBRARY=/path/to/sdk-5.x/BlackmagicRawAPI.dll cargo test --test sdk_version_gate -- --ignored
```

The COM objects implemented in Rust (`src/file.rs`, `src/callback.rs`) and the job
completion state the SDK co-owns (`src/future.rs`) are also tested under
[Miri](https://github.com/rust-lang/miri), for both of the SDK's ABIs:

```sh
cargo +nightly miri test --lib --all-features
cargo +nightly miri test --lib --all-features --target x86_64-pc-windows-msvc
cargo +nightly miri test --lib --all-features --target x86_64-unknown-linux-gnu
```

CI runs everything except the SDK-backed tests on Windows, Linux and macOS, so please
run those locally when a change touches the SDK interface.

## Code style

- `cargo clippy --all-targets --all-features -- -D warnings` must pass.
- The code is formatted by hand, with aligned columns in places; match the
  surrounding style rather than running `cargo fmt` over a file.
- Every public item is documented (the `missing_docs` lint is on).
- Comments describe the code as it is — what it does and why — not how it came to be.

## Safety rules

- A function that passes the SDK a pointer it cannot vouch for — caller memory, a
  GPU handle, a resource — is an `unsafe fn` with a `# Safety` section. The
  `braw_interface!` accessors generate only safe methods, so such a method is
  written by hand.
- Anything the SDK uses without holding a reference to it — memory it keeps a
  pointer to, a device whose context it uses — is owned by everything using it:
  objects through their `parent_guards`, jobs through the keep-alive their
  completion state holds until the job completes (`submit` in `src/lib.rs`).
  `sdk_buffer` and `keep_alive` in `src/com.rs` make such things shareable guards.
- Every `unsafe` block says why it is sound in a `// SAFETY:` comment.

## Updating to a new SDK release

`src/sdk.rs` mirrors the SDK's `BlackmagicRawAPI.h`, and `tests/sdk_layout.rs`
checks every interface's method order, IID and enumerator values against the header
of the SDK under test. COM calls go through vtable slots by position, so a method
inserted in a new release must be inserted at the same position in the bindings.
`Factory::create_codec` checks the codec's IID to refuse libraries of another
version; update it, the version in the README and `CHANGELOG.md` with the bindings.

## Releasing

1. In `CHANGELOG.md`, rename `[Unreleased]` to the new version and date
   (`## [X.Y.Z] - YYYY-MM-DD`), start a new empty `[Unreleased]` section, and update
   the links at the bottom.
2. Bump `version` in `Cargo.toml` and run `cargo check` to update `Cargo.lock`.
3. Run the full test suite against the SDK on every platform you can, with
   `BRAW_REQUIRE_SDK=1`.
4. Commit, then tag and push both: `git tag vX.Y.Z && git push origin main vX.Y.Z`.

The [release workflow](.github/workflows/release.yml) checks that the tag matches the
crate version, publishes to crates.io through
[trusted publishing](https://crates.io/docs/trusted-publishing) and creates the GitHub
release from the changelog section.

crates.io only lets trusted publishing be configured for a crate that already
exists, so the first release is published by hand: run `cargo publish`, then add
this repository's `release.yml` workflow, with the `release` environment, as a
trusted publisher in the crate's settings on crates.io. Pushing the tag afterwards
creates the GitHub release; the workflow skips publishing a version that is
already on crates.io.
