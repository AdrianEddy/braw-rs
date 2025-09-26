// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;

#[derive(Debug)]
pub enum BrawError {
    NullValue,
    Unexpected,
    NotImplemented,
    OutOfMemory,
    InvalidArgument,
    NoInterface,
    Pointer,
    Handle,
    Abort,
    Fail,
    AccessDenied,
    OtherHresult(HRESULT),
    Libloading(libloading::Error),
    Other(String),
}

impl std::fmt::Display for BrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrawError::NullValue          => write!(f, "BRAW error: Null value"),
            BrawError::Unexpected         => write!(f, "BRAW error: Unexpected"),
            BrawError::NotImplemented     => write!(f, "BRAW error: Not implemented"),
            BrawError::OutOfMemory        => write!(f, "BRAW error: Out of memory"),
            BrawError::InvalidArgument    => write!(f, "BRAW error: Invalid argument"),
            BrawError::NoInterface        => write!(f, "BRAW error: No interface"),
            BrawError::Pointer            => write!(f, "BRAW error: Pointer error"),
            BrawError::Handle             => write!(f, "BRAW error: Handle error"),
            BrawError::Abort              => write!(f, "BRAW error: Abort"),
            BrawError::Fail               => write!(f, "BRAW error: Fail"),
            BrawError::AccessDenied       => write!(f, "BRAW error: Access denied"),
            BrawError::OtherHresult(hr)   => write!(f, "BRAW error: HRESULT 0x{hr:X}"),
            BrawError::Libloading(e)      => write!(f, "BRAW error: Libloading error: {e}"),
            BrawError::Other(s)           => write!(f, "BRAW error: {s}"),
        }
    }
}
impl std::error::Error for BrawError { }

impl From<HRESULT> for BrawError {
    fn from(hr: HRESULT) -> Self {
        match hr as u32 {
            0x8000FFFF => BrawError::Unexpected,
            0x80000001 => BrawError::NotImplemented,
            0x80000002 => BrawError::OutOfMemory,
            0x80000003 => BrawError::InvalidArgument,
            0x80000004 => BrawError::NoInterface,
            0x80000005 => BrawError::Pointer,
            0x80000006 => BrawError::Handle,
            0x80000007 => BrawError::Abort,
            0x80000008 => BrawError::Fail,
            0x80000009 => BrawError::AccessDenied,
            _ => BrawError::OtherHresult(hr),
        }
    }
}
impl From<libloading::Error> for BrawError {
    fn from(e: libloading::Error) -> Self { BrawError::Libloading(e) }
}

pub type BrawResult = Result<bool, BrawError>;

pub(crate) fn check_hr(hr: HRESULT) -> BrawResult { if hr == S_OK { Ok(true) } else if hr == S_FALSE { Ok(false) } else { Err(BrawError::from(hr)) } }
