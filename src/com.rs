// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;
use core::ffi::c_void;
use core::ptr::NonNull;
use std::ops::{ Deref, DerefMut };

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GUID {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}
impl GUID {
    pub const fn new(bytes: [u8; 16]) -> Self {
        #[cfg(target_os = "windows")]{
            let d1 = ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);
            let d2 = ((bytes[4] as u16) << 8) | (bytes[5] as u16);
            let d3 = ((bytes[6] as u16) << 8) | (bytes[7] as u16);
            let d4 = [bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]];
            GUID { d1, d2, d3, d4 }
        }
        #[cfg(not(target_os = "windows"))]{
            let d1 = ((bytes[3] as u32) << 24) | ((bytes[2] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[0] as u32);
            let d2 = ((bytes[5] as u16) << 8) | (bytes[4] as u16);
            let d3 = ((bytes[7] as u16) << 8) | (bytes[6] as u16);
            let d4 = [bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]];
            GUID { d1, d2, d3, d4 }
        }
    }
}

// Platform-correct QueryInterface signature: pointer on Windows, by-value REFIID elsewhere
#[cfg(target_os = "windows")]
pub type QueryInterfaceFn = unsafe extern "system" fn(this: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
#[cfg(not(target_os = "windows"))]
pub type QueryInterfaceFn = unsafe extern "system" fn(this: *mut c_void, riid: GUID, ppv: *mut *mut c_void) -> HRESULT;

#[repr(C)]
#[allow(non_snake_case)]
pub struct IUnknownVTbl {
    pub QueryInterface: QueryInterfaceFn,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

#[macro_export]
#[doc(hidden)]
macro_rules! braw_interface {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$fn_meta:meta])*
                fn $m:ident ( $($argn:ident : $argt:ty),* ) -> $ret:ty ;
            )*
        }
        $(
            $(
                field $field:ident : $field_type:ty,
            )*
            $(#[$implmeta:meta])*
            impl {
                $(
                    struct $impl_cm:expr => fn $impl_m:ident($impl_self:ty $(, $impl_argn:ident : $impl_argt:ty)*) -> $impl_ret:ty ; $(#[$impl_meta:meta])*
                )*
                $(
                    scalar $impls_cm:expr => fn $impls_m:ident($impls_self:ty $(, $impls_argn:ident : $impls_argt:ty)*) -> $impls_ret:ty ; $(#[$impls_meta:meta])*
                )*
                $(
                    scalar2 $impls2_cm:expr => fn $impls2_m:ident($impls2_self:ty $(, $impls2_argn:ident : $impls2_argt:ty)*) -> ($impls2_ret1:ty, $impls2_ret2:ty) ; $(#[$impls2_meta:meta])*
                )*
                $(

                    scalar3 $impls3_cm:expr => fn $impls3_m:ident($impls3_self:ty $(, $impls3_argn:ident : $impls3_argt:ty)*) -> ($impls3_ret1:ty, $impls3_ret2:ty, $impls3_ret3:ty) ; $(#[$impls3_meta:meta])*
                )*
                $(
                    void $implv_cm:expr => fn $implv_m:ident($implv_self:ty $(, $implv_argn:ident : $implv_argt:ty)*); $(#[$implv_meta:meta])*
                )*
                $(
                    interface fn $impli_m:ident(&self) -> $impli_cm:expr; $(#[$impli_meta:meta])*
                )*
            }
        )?
    ) => {
        paste::paste! {
            $(#[$meta])*
            #[repr(C)]
            #[doc(hidden)]
            pub struct [<I $name>] { pub(crate) vtbl: *const [<I $name VTbl>] }
            #[repr(C)]
            #[doc(hidden)]
            pub struct [<I $name VTbl>] {
                pub parent: IUnknownVTbl,

                $($(#[$fn_meta])* pub $m: unsafe extern "system" fn(this: *mut c_void, $($argn : $argt),*) -> $ret, )*
            }
            impl ComPtr<[<I $name>]> {
                $(
                #[allow(non_snake_case)]
                $(#[$fn_meta])*
                pub fn $m(&self, $($argn : $argt),*) -> Result<$ret, BrawError> {
                    unsafe {
                        let vtbl = &*((*self).vtbl);
                        let hr = (vtbl.$m)(self.as_raw() as *mut _, $($argn),*);
                        check_hr(braw_interface!(@ret hr $ret))?;
                        Ok(hr)
                    }
                }
                )*
            }
            impl [<I $name>] {
                pub const IID: GUID = [<IID_I $name>];
                #[cfg(target_os = "windows")]
                pub const fn iid() -> &'static GUID { &Self::IID }
                #[cfg(not(target_os = "windows"))]
                pub const fn iid() -> GUID { Self::IID }
            }

            $(
                $(#[$implmeta])*
                pub struct $name {
                    pub raw: ComPtr<[<I $name>]>,
                    $( pub(crate) $field : $field_type, )*

                    #[allow(dead_code)]
                    pub(crate) parent_guards: DropOrderVec<ComPtrRefGuard>,

                    // Factory always last, to ensure it outlives all other references
                    pub factory: Factory,
                }
                impl $name {
                    /// Get the raw COM interface pointer
                    pub fn as_raw(&self) -> *mut [<I $name>] { self.raw.as_raw() }

                $(
                    $(#[$impl_meta])*
                    pub fn $impl_m(self: $impl_self, $($impl_argn : braw_interface!(@iarg $impl_argt)),*) -> Result<$impl_ret, BrawError> {
                        unsafe {
                            let mut out: *mut [<I $impl_ret>] = std::mem::zeroed();
                            let _hr = self.raw.$impl_cm($(braw_interface!(@iargpass self; $impl_argt,$impl_argn),)* &mut out)?;
                            Ok($impl_ret {
                                raw: ComPtr::new(out)?,
                                factory: self.factory.clone(),
                                parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()),
                            })
                        }
                    }
                )*
                $(
                    $(#[$impls_meta])*
                    pub fn $impls_m(self: $impls_self, $($impls_argn : braw_interface!(@iarg $impls_argt)),*) -> Result<$impls_ret, BrawError> {
                        let mut out: braw_interface!(@iargpassret $impls_ret) = Default::default();
                        let _hr = self.raw.$impls_cm($(braw_interface!(@iargpass self; $impls_argt,$impls_argn),)* &mut out)?;
                        Ok(braw_interface!(@iargpassret2 self; $impls_ret,out))
                    }
                )*
                $(
                    $(#[$impls2_meta])*
                    pub fn $impls2_m(self: $impls2_self, $($impls2_argn : braw_interface!(@iarg $impls2_argt)),*) -> Result<($impls2_ret1, $impls2_ret2), BrawError> {
                        unsafe {
                            let mut out1: braw_interface!(@iargpassret $impls2_ret1) = std::mem::zeroed();
                            let mut out2: braw_interface!(@iargpassret $impls2_ret2) = std::mem::zeroed();
                            let _hr = self.raw.$impls2_cm($(braw_interface!(@iargpass self; $impls2_argt,$impls2_argn),)* &mut out1, &mut out2)?;
                            Ok((out1, out2))
                        }
                    }
                )*
                $(
                    $(#[$impls3_meta])*
                    pub fn $impls3_m(self: $impls3_self, $($impls3_argn : braw_interface!(@iarg $impls3_argt)),*) -> Result<($impls3_ret1, $impls3_ret2, $impls3_ret3), BrawError> {
                        unsafe {
                            let mut out1: braw_interface!(@iargpassret $impls3_ret1) = std::mem::zeroed();
                            let mut out2: braw_interface!(@iargpassret $impls3_ret2) = std::mem::zeroed();
                            let mut out3: braw_interface!(@iargpassret $impls3_ret3) = std::mem::zeroed();
                            let _hr = self.raw.$impls3_cm($(braw_interface!(@iargpass self; $impls3_argt,$impls3_argn),)* &mut out1, &mut out2, &mut out3)?;
                            Ok((out1, out2, out3))
                        }
                    }
                )*
                $(
                    $(#[$implv_meta])*
                    pub fn $implv_m(self: $implv_self, $($implv_argn : braw_interface!(@iarg $implv_argt)),*) -> Result<(), BrawError> {
                        let _hr = self.raw.$implv_cm($(braw_interface!(@iargpass self; $implv_argt,$implv_argn),)*)?;
                        Ok(())
                    }
                )*
                $(
                    $(#[$impli_meta])*
                    pub fn $impli_m(&self) -> Result<$impli_cm, BrawError> {
                        let mut ptr = std::ptr::null_mut();
                        let hr = unsafe { ((*self.raw.vtbl).parent.QueryInterface)(self.raw.as_raw() as _, [<I $impli_cm>]::iid(), &mut ptr) };
                        check_hr(hr)?;
                        let com = ComPtr::new(ptr as *mut [<I $impli_cm>])?;

                        Ok($impli_cm { raw: com, factory: self.factory.clone(), parent_guards: self.parent_guards.clone_and_add(self.raw.add_ref_and_get_guard()) })
                    }
                )*
                }
            )?
        }
    };
    (@ret $hr:ident $ret:ty) => { S_OK };
    (@ret $hr:ident HRESULT) => { $hr };

    (@iarg String) => { &str };
    (@iarg $t:ty) => { $t };
    (@iargpass $self:ident; String,$e:expr) => { BrawString::from($e).as_raw() };
    (@iargpass $self:ident; BlackmagicRawClipGeometry,$e:expr) => { $e.as_raw() };
    (@iargpass $self:ident; BlackmagicRawProcessedImage,$e:expr) => { $e.as_raw() };
    (@iargpass $self:ident; BlackmagicRawResourceManager,$e:expr) => { $e.as_raw() };
    (@iargpass $self:ident; BlackmagicRawPipelineDevice,$e:expr) => { $e.as_raw() };
    (@iargpass $self:ident; VariantValue,$e:expr) => { $self.factory.lib.variant_from_rust($e).as_raw() };
    (@iargpass $self:ident; $t:ty,$e:expr) => { $e };

    (@iargpassret String) => { *mut c_void };
    (@iargpassret VariantValue) => { VARIANT };
    (@iargpassret $t:ty) => { $t };
    (@iargpassret2 $self:ident; String,$o:expr) => { BrawString($o as *mut _).to_string() };
    (@iargpassret2 $self:ident; VariantValue,$o:expr) => { $self.factory.lib.variant_to_rust($o) };
    (@iargpassret2 $self:ident; $t:ty,$o:expr) => { $o };
}

#[macro_export]
#[doc(hidden)]
macro_rules! braw_out_ptr {
    ($expr:expr $(, $arg:expr)*) => {{
        let mut tmp = std::ptr::null_mut();
        let hr = $expr($($arg,)* &mut tmp)?;
        check_hr(hr)?;
        ComPtr::new(tmp)?
    }};
}

#[repr(C)]
pub struct ComPtr<T> { ptr: NonNull<T> }
impl<T> ComPtr<T> {
    pub fn new(nn: *mut T) -> Result<Self, BrawError> { Ok(Self { ptr: NonNull::new(nn).ok_or(BrawError::NullValue)? }) }
    pub fn as_raw(&self) -> *mut T { self.ptr.as_ptr() }
}
impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            // AddRef
            (self.get_iunknown_vtbl().AddRef)(self.ptr.as_ptr() as *mut c_void);
            Self { ptr: self.ptr }
        }
    }
}
impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            (self.get_iunknown_vtbl().Release)(self.ptr.as_ptr() as *mut c_void);
        }
    }
}
impl<T> ComPtr<T> {
    pub unsafe fn add_ref(&mut self) {
        unsafe { (self.get_iunknown_vtbl().AddRef)(self.ptr.as_ptr() as *mut c_void); }
    }
    pub unsafe fn release(&mut self) {
        unsafe { (self.get_iunknown_vtbl().Release)(self.ptr.as_ptr() as *mut c_void); }
    }
    pub(crate) fn add_ref_and_get_guard(&self) -> ComPtrRefGuard {
        unsafe { (self.get_iunknown_vtbl().AddRef)(self.ptr.as_ptr() as *mut c_void); }
        ComPtrRefGuard { ptr: self.ptr.as_ptr() as *mut c_void, name: std::any::type_name::<T>()}
    }
    unsafe fn get_iunknown_vtbl(&self) -> &IUnknownVTbl {
        // Safety: every COM interface starts with IUnknown vtbl
        unsafe {
            &**(self.ptr.as_ptr() as *mut *const IUnknownVTbl)
        }
    }
}
impl<T> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { self.ptr.as_ref() } }
}
impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { self.ptr.as_mut() } }
}

pub(crate) struct ComPtrRefGuard {
    ptr: *mut c_void,
    name: &'static str,
}
impl Clone for ComPtrRefGuard {
    fn clone(&self) -> Self {
        unsafe {
            if !self.ptr.is_null() {
                let vtbl = &**(self.ptr as *mut *const IUnknownVTbl);
                (vtbl.AddRef)(self.ptr);
            }
            ComPtrRefGuard { ptr: self.ptr, name: self.name }
        }
    }
}
impl Drop for ComPtrRefGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let vtbl = &**(self.ptr as *mut *const IUnknownVTbl);
                (vtbl.Release)(self.ptr);
            }
        }
    }
}

#[derive(Clone)]
pub struct DropOrderVec<T: Clone>(pub Vec<T>);
impl<T: Clone> DropOrderVec<T> {
    pub fn clone_and_add(&self, v: T) -> Self {
        let mut guards = self.clone();
        guards.0.push(v);
        guards
    }
}
impl<T: Clone> Drop for DropOrderVec<T> {
    fn drop(&mut self) {
        // Deterministic drop order: back -> front (LIFO)
        while self.0.pop().is_some() { }
    }
}
impl<T: Clone> From<Vec<T>> for DropOrderVec<T> {
    fn from(arr: Vec<T>) -> Self {
        DropOrderVec(arr)
    }
}
