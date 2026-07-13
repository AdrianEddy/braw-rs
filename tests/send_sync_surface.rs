// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! Compile-only proof that the per-interface Send / Sync classification
//! in `src/send_sync.rs` matches the audit table in stage-6c.md
//! § 6c.0.2. Any future refactor that accidentally adds a blanket
//! `unsafe impl<T> Send + Sync for ComPtr<T>` would silently re-enable
//! Sync on the Send-only interfaces — the `assert_not_impl_all!` lines
//! below catch that immediately.

use braw::*;
use static_assertions::{assert_impl_all, assert_not_impl_all};

// ─────────────────────────────────────────────────────────────────────
// Send + Sync interfaces — stateless on `&self`.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn factory_is_send_sync() {
    assert_impl_all!(Factory: Send, Sync);
}

#[test]
fn read_only_interfaces_are_send_sync() {
    assert_impl_all!(BlackmagicRawProcessedImage:               Send, Sync);
    assert_impl_all!(BlackmagicRawPipelineDevice:               Send, Sync);
    assert_impl_all!(BlackmagicRawClipResolutions:              Send, Sync);
    assert_impl_all!(BlackmagicRawClipAudio:                    Send, Sync);
    assert_impl_all!(BlackmagicRawClipAccelerometerMotion:      Send, Sync);
    assert_impl_all!(BlackmagicRawClipGyroscopeMotion:          Send, Sync);
    assert_impl_all!(BlackmagicRawClipPDAFData:                 Send, Sync);
    assert_impl_all!(BlackmagicRawPost3DLUT:                    Send, Sync);
    assert_impl_all!(BlackmagicRawToneCurve:                    Send, Sync);
}

#[test]
fn callback_future_is_send_sync() {
    assert_impl_all!(CallbackFuture<()>: Send, Sync);
}

// ─────────────────────────────────────────────────────────────────────
// Send-only interfaces — cursor-state / single-owner mutators.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn stateful_interfaces_are_send_only() {
    // Stateful core interfaces — `&mut` mutators.
    assert_impl_all!(BlackmagicRaw:                             Send);
    assert_impl_all!(BlackmagicRawClip:                         Send);
    assert_impl_all!(BlackmagicRawClipEx:                       Send);
    assert_impl_all!(BlackmagicRawClipMultiVideo:               Send);
    assert_impl_all!(BlackmagicRawClipImmersiveVideo:           Send);
    assert_impl_all!(BlackmagicRawFrame:                        Send);
    assert_impl_all!(BlackmagicRawFrameEx:                      Send);
    assert_impl_all!(BlackmagicRawFrameMultiVideo:              Send);
    assert_impl_all!(BlackmagicRawJob:                          Send);
    assert_impl_all!(BlackmagicRawConfiguration:                Send);
    assert_impl_all!(BlackmagicRawConfigurationEx:              Send);
    assert_impl_all!(BlackmagicRawResourceManager:              Send);
    assert_impl_all!(BlackmagicRawOpenGLInteropHelper:          Send);
    assert_impl_all!(BlackmagicRawManualDecoderFlow1:           Send);
    assert_impl_all!(BlackmagicRawManualDecoderFlow2:           Send);
    assert_impl_all!(BlackmagicRawClipProcessingAttributes:     Send);
    assert_impl_all!(BlackmagicRawFrameProcessingAttributes:    Send);
    assert_impl_all!(BlackmagicRawClipGeometry:                 Send);
    assert_impl_all!(BlackmagicRawReadJobHints:                 Send);

    // Iterators — cursor state.
    assert_impl_all!(BlackmagicRawPipelineIterator:             Send);
    assert_impl_all!(BlackmagicRawPipelineDeviceIterator:       Send);
    assert_impl_all!(BlackmagicRawMetadataIterator:             Send);
    assert_impl_all!(PipelineIterator:                          Send);
    assert_impl_all!(PipelineDeviceIterator:                    Send);
    assert_impl_all!(MetadataIterator:                          Send);

    // `'static` pipeline futures — single-owner, Send-only (their
    // `ComPtrRefGuard` keep-alive is `!Sync`).
    assert_impl_all!(ReadFrameFuture:                           Send);
    assert_impl_all!(DecodeProcessFuture:                       Send);
}

#[test]
fn stateful_interfaces_are_not_sync() {
    // The strong assertion: blanket `unsafe impl<T> Sync for ComPtr<T>`
    // would silently re-enable Sync on these. The negative checks
    // catch that immediately.
    assert_not_impl_all!(BlackmagicRaw:                         Sync);
    assert_not_impl_all!(BlackmagicRawClip:                     Sync);
    assert_not_impl_all!(BlackmagicRawFrame:                    Sync);
    assert_not_impl_all!(BlackmagicRawJob:                      Sync);
    assert_not_impl_all!(BlackmagicRawConfiguration:            Sync);
    assert_not_impl_all!(BlackmagicRawConfigurationEx:          Sync);
    assert_not_impl_all!(BlackmagicRawResourceManager:          Sync);
    assert_not_impl_all!(BlackmagicRawPipelineIterator:         Sync);
    assert_not_impl_all!(BlackmagicRawPipelineDeviceIterator:   Sync);
    assert_not_impl_all!(BlackmagicRawMetadataIterator:         Sync);
    assert_not_impl_all!(MetadataIterator:                      Sync);
    assert_not_impl_all!(PipelineIterator:                      Sync);
    assert_not_impl_all!(PipelineDeviceIterator:                Sync);
    assert_not_impl_all!(ReadFrameFuture:                       Sync);
    assert_not_impl_all!(DecodeProcessFuture:                   Sync);
}
