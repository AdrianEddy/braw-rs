// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Public-surface contract test for the asynchronous job futures.
//!
//! The detailed *lifecycle* proofs — the SDK-refcount co-ownership (a future
//! dropped before its callback is use-after-free-free), the claim-once guard
//! that makes a double / spurious completion callback a safe no-op, and the
//! panic firewall around result construction — need the crate-private `State`
//! and `deliver_completion` internals. An integration test is a *separate*
//! crate and cannot reach those, so those proofs live in the
//! `#[cfg(test)] mod tests` unit module inside `src/future.rs` (run by the same
//! `cargo test` invocation).
//!
//! This integration test pins the *public* async contract instead: the exact
//! `Send` / `Sync` / `Unpin` classification of the futures and that they are
//! real `Future`s. It needs no codec, clip, or GPU.

use braw::*;
use std::future::Future;
use static_assertions::{ assert_impl_all, assert_not_impl_all };

/// The shareable, awaitable core future. Its `Sync` rests on BMD's documented
/// free-threaded job model (see the `unsafe impl` docs in `src/future.rs`).
#[test]
fn callback_future_public_contract() {
    assert_impl_all!(CallbackFuture<()>: Future, Send, Sync, Unpin);
}

/// The `'static` pipeline futures are single-owner: `Send` + `Unpin` `Future`s,
/// but deliberately **not** `Sync` — their `ComPtrRefGuard` keep-alives are
/// `Send` but not `Sync`, so `&T` must not be shared across threads.
#[test]
fn pipeline_futures_public_contract() {
    assert_impl_all!(ReadFrameFuture:     Future, Send, Unpin);
    assert_impl_all!(DecodeProcessFuture: Future, Send, Unpin);
    assert_not_impl_all!(ReadFrameFuture:     Sync);
    assert_not_impl_all!(DecodeProcessFuture: Sync);
}
