// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

//! The bindings in `src/sdk.rs` must match the SDK header they are written
//! against — `sdk/Linux/Include/BlackmagicRawAPI.h` (the Windows IDL, the Apple
//! header and the Linux header declare the same interfaces).
//!
//! A COM call goes through a vtable slot chosen by position, so an interface whose
//! methods are declared out of order — or with one missing — calls the wrong
//! function at runtime with no compile-time error. SDK releases insert methods
//! mid-interface, sometimes without changing the interface's IID (6.0 inserted
//! `IBlackmagicRawClip::CreateJobReadAudio` and
//! `IBlackmagicRawCallback::ReadAudioComplete`), so this compares, per interface,
//! the method order, the IID and the enumerator values.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())).replace("\r\n", "\n")
}

/// `name(` → `name`, for the first identifier immediately before `(`.
fn ident_before_paren(s: &str) -> Option<&str> {
    let head = s[..s.find('(')?].trim_end();
    let start = head.rfind(|c: char| !(c.is_alphanumeric() || c == '_')).map_or(0, |i| i + 1);
    (start < head.len()).then(|| &head[start..])
}

/// The UUID inside a `/* XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX */` comment.
fn uuid_comment(s: &str) -> Option<&str> {
    let start = s.find("/* ")? + 3;
    s.get(start..start + 36).filter(|u| u.len() == 36 && u.chars().filter(|&c| c == '-').count() == 4)
}

/// Header: interface name → method names in vtable order.
fn header_interfaces(h: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for chunk in h.split("class BMD_PUBLIC ").skip(1) {
        let name = chunk.split(|c: char| !(c.is_alphanumeric() || c == '_')).next().unwrap().to_owned();
        let body = &chunk[..chunk.find("\n};").expect("class body")];
        let methods = body
            .lines()
            .filter_map(|l| l.trim().strip_prefix("virtual "))
            .filter(|l| !l.trim_start().starts_with('~'))
            .filter_map(|l| ident_before_paren(l).map(str::to_owned))
            .collect();
        out.insert(name, methods);
    }
    out
}

/// Bindings: interface name (with the `I` prefix) → method names in declaration order.
fn binding_interfaces(rs: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for chunk in rs.split("braw_interface! {").skip(1) {
        // Skip doc comments / attributes to reach `Name {`.
        let decl = chunk.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with("///") && !l.starts_with("#[")).unwrap();
        let name = decl.trim_end_matches('{').trim().to_owned();
        let body = &chunk[chunk.find('{').unwrap() + 1..];
        let body = &body[..body.find("\n    }").expect("method list")];
        let methods = body
            .lines()
            .filter_map(|l| l.trim().strip_prefix("fn "))
            .filter_map(|l| ident_before_paren(l).map(str::to_owned))
            .collect();
        out.insert(format!("I{name}"), methods);
    }
    out
}

/// `IID_IName` → UUID, from either source.
fn iids(src: &str, marker: &str) -> BTreeMap<String, String> {
    src.lines()
        .filter(|l| l.contains(marker))
        .filter_map(|l| {
            let name_start = l.find("IID_I")?;
            let name: String = l[name_start..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            Some((name, uuid_comment(&l[name_start..])?.to_owned()))
        })
        .collect()
}

/// Enum name → (enumerator name → value), for enumerators with a hex literal.
fn enums(src: &str, open: &str, item_prefix: impl Fn(&str) -> String) -> BTreeMap<String, BTreeMap<String, u32>> {
    let mut out = BTreeMap::new();
    for chunk in src.split(open).skip(1) {
        let name: String = chunk.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        let body = &chunk[..chunk.find("\n}").expect("enum body")];
        let prefix = item_prefix(&name);
        let items = body
            .lines()
            .filter_map(|l| {
                let (lhs, rhs) = l.split_once('=')?;
                let hex = rhs.split("0x").nth(1)?;
                let hex: String = hex.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                let item = lhs.trim().strip_prefix(&prefix)?.to_owned();
                Some((item, u32::from_str_radix(&hex, 16).ok()?))
            })
            .collect::<BTreeMap<_, _>>();
        out.insert(name, items);
    }
    out
}

#[test]
fn vtables_match_the_sdk_header() {
    let header = header_interfaces(&read("sdk/Linux/Include/BlackmagicRawAPI.h"));
    let bindings = binding_interfaces(&read("src/sdk.rs"));
    assert!(header.len() >= 36, "parsed only {} header interfaces", header.len());
    let mismatches: Vec<String> = header
        .keys()
        .chain(bindings.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|k| header.get(*k) != bindings.get(*k))
        .map(|k| format!("{k}\n  header:   {:?}\n  bindings: {:?}", header.get(k), bindings.get(k)))
        .collect();
    assert!(mismatches.is_empty(), "vtable layout differs from the SDK header:\n{}", mismatches.join("\n"));
}

#[test]
fn iids_match_the_sdk_header() {
    let header = iids(&read("sdk/Linux/Include/BlackmagicRawAPI.h"), "BMD_CONST REFIID IID_");
    let bindings = iids(&read("src/sdk.rs"), "pub(crate) const IID_");
    assert!(header.len() >= 36, "parsed only {} header IIDs", header.len());
    assert_eq!(bindings, header);
}

#[test]
fn enumerators_match_the_sdk_header() {
    let header = enums(&read("sdk/Linux/Include/BlackmagicRawAPI.h"), "enum _", |name| {
        // `blackmagicRawClipProcessingAttributeGamma` for `BlackmagicRawClipProcessingAttribute`.
        let mut prefix = name.to_owned();
        prefix[..1].make_ascii_lowercase();
        prefix
    });
    let bindings = enums(&read("src/sdk.rs"), "pub enum ", |_| String::new());
    let mut checked = 0;
    for (name, items) in &header {
        if items.is_empty() { continue; } // e.g. `BlackmagicRawVariantType`, whose values are `VT_*` names
        let ours = bindings.get(name).unwrap_or_else(|| panic!("no binding for enum {name}"));
        for (item, value) in items {
            // Matched by name where the names agree; a header name a Rust identifier
            // cannot spell (`blackmagicRawAnamorphicRatio133x`) is matched by value.
            match ours.get(item) {
                Some(v) => assert_eq!(v, value, "{name}::{item}"),
                None => assert!(ours.values().any(|v| v == value), "{name}::{item} = {value:#x} has no binding"),
            }
            checked += 1;
        }
        for (item, value) in ours {
            assert!(*value == 0 || items.values().any(|v| v == value), "{name}::{item} = {value:#x} is not in the header");
        }
    }
    // SDK 6.0 declares 96 hex-valued enumerators; far fewer means the parser broke.
    assert!(checked >= 96, "checked only {checked} enumerators");
}
