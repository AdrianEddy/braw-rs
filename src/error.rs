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
