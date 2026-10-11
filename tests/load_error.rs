// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! A library that fails to load is reported as [`BrawError::LibraryLoad`], with the
//! loader's reason: the loader's own message names only the call that failed
//! ("dlopen failed", "LoadLibraryExW failed"), which is no help to show a user.

use braw::{ BrawError, Factory, LoadError };
use static_assertions::assert_impl_all;

assert_impl_all!(BrawError: Send, Sync, Clone);
assert_impl_all!(LoadError: Send, Sync, Clone, std::error::Error);

#[test]
fn a_missing_library_is_reported_with_the_loaders_reason() {
    let Err(err) = Factory::load_from("braw-rs-no-such-library") else { panic!("a library that does not exist loaded") };
    let BrawError::LibraryLoad(load) = &err else { panic!("{err:?}") };

    let message = load.to_string();
    let (_call, reason) = message.split_once("failed: ").unwrap_or_else(|| panic!("no reason in {message:?}"));
    assert!(!reason.trim().is_empty(), "{message:?}");
    assert!(err.to_string().ends_with(&message), "{err}");

    assert!(matches!(err.clone(), BrawError::LibraryLoad(e) if e.to_string() == message));
}
