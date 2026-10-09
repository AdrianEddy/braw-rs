// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;
use core::ffi::c_void;
use core::ptr::NonNull;
use std::ops::{ Deref, DerefMut };
use std::sync::atomic::{ fence, AtomicU32, Ordering };

/// The SDK's COM boolean out-parameter width.
///
/// On Windows the Blackmagic RAW COM ABI declares every `[out]` boolean as
/// `BOOL*` — a 4-byte `int` (see `sdk/Win/Include/BlackmagicRawAPI.idl`). The
/// Mac/Linux C++ interface genuinely uses `bool*` (1 byte). Declaring the
/// out-parameter as a 1-byte `bool` on Windows lets the DLL's 4-byte store
/// overrun the adjacent stack slot — e.g. the `arrayElementCount` variable
/// declared right after `isReadOnly` in `GetClipAttributeList` — silently
/// zeroing the count under release codegen (a layout-sensitive heisenbug that
/// empties every attribute/ISO value list). Model the out-parameter at its true
/// ABI width and narrow to `bool` in Rust.
#[cfg(windows)]
pub type SdkBool = i32;
#[cfg(not(windows))]
pub type SdkBool = bool;

/// Narrow an SDK boolean out-parameter (`BOOL` on Windows, `bool` elsewhere) to
/// a Rust `bool`.
#[cfg(windows)]
#[inline]
pub fn sdk_bool(v: SdkBool) -> bool { v != 0 }
#[cfg(not(windows))]
#[inline]
pub fn sdk_bool(v: SdkBool) -> bool { v }

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

/// The COM `ULONG` that `AddRef` / `Release` return: 32-bit on Windows and in
/// CoreFoundation's `CFPlugInCOM.h` (Apple), but `unsigned long` — 64-bit on
/// LP64 — in the SDK's `LinuxCOM.h`.
#[cfg(any(target_os = "windows", target_vendor = "apple"))]
pub type ComUlong = u32;
#[cfg(not(any(target_os = "windows", target_vendor = "apple")))]
pub type ComUlong = core::ffi::c_ulong;

#[repr(C)]
#[allow(non_snake_case)]
pub struct IUnknownVTbl {
    pub QueryInterface: QueryInterfaceFn,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> ComUlong,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> ComUlong,
}

/// The `riid` argument of a `QueryInterface` implemented in Rust (see [`QueryInterfaceFn`]).
#[cfg(target_os = "windows")]
pub(crate) type QueryInterfaceRiid = *const GUID;
#[cfg(not(target_os = "windows"))]
pub(crate) type QueryInterfaceRiid = GUID;

#[inline]
pub(crate) unsafe fn guid_from_riid(riid: QueryInterfaceRiid) -> GUID {
    #[cfg(target_os = "windows")]
    unsafe { *riid }
    #[cfg(not(target_os = "windows"))]
    { riid }
}

pub(crate) const IID_IUNKNOWN: GUID = GUID::new([0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xC0,0x00,0x00,0x00,0x00,0x00,0x00,0x46]);

/// Panic firewall for the `extern "system"` COM methods implemented in Rust. A
/// Rust panic must never unwind across the C++ ABI boundary that invoked us:
/// catch it, log it, and return an ABI-valid `fallback` instead of unwinding.
#[inline]
pub(crate) fn ffi_guard<R>(what: impl std::fmt::Display, fallback: R, f: impl FnOnce() -> R) -> R {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(_) => {
            log::error!("panic in BRAW COM method `{what}` swallowed at the FFI boundary");
            fallback
        }
    }
}

/// The Rust state of a COM object implemented in Rust — a callback, a file, a
/// filesystem — and the one interface it implements besides `IUnknown`.
///
/// # Safety
/// [`VTABLE`](Self::VTABLE) must begin with [`ComObject::<Self>::IUNKNOWN`], and
/// each of its other methods must expect `this` to point to a `ComObject<Self>`.
pub(crate) unsafe trait ComClass: Sized + 'static {
    /// The interface type the SDK sees.
    type Interface;
    /// The interface's vtable type.
    type VTable: 'static;
    /// The interface's IID.
    const IID: GUID;
    /// The interface's name, for diagnostics.
    const NAME: &'static str;
    /// The vtable every object of this class carries. Point it at a `static` for
    /// [`ComObject::downcast`] to recognise the objects.
    const VTABLE: &'static Self::VTable;
}

/// A reference count no real program reaches: past it the count is being leaked
/// in a loop, and letting it wrap would free a referenced object (as `Arc` guards).
const MAX_REFCOUNT: u32 = u32::MAX / 2;

/// A COM object implemented in Rust, laid out as the SDK expects: the vtable
/// pointer, then the reference count, then the Rust state. Its `IUnknown` methods
/// are shared by every class: the vtable starts with [`IUNKNOWN`](Self::IUNKNOWN).
#[repr(C)]
pub(crate) struct ComObject<T: ComClass> {
    vtbl: &'static T::VTable,
    refcnt: AtomicU32,
    pub(crate) state: T,
}

impl<T: ComClass> ComObject<T> {
    pub(crate) const IUNKNOWN: IUnknownVTbl = IUnknownVTbl {
        QueryInterface: Self::query_interface,
        AddRef: Self::add_ref,
        Release: Self::release,
    };

    /// A new object holding one reference, owned by the returned pointer.
    pub(crate) fn create(state: T) -> ComPtr<T::Interface> {
        let obj = Box::new(Self { vtbl: T::VTABLE, refcnt: AtomicU32::new(1), state });
        // SAFETY: the object begins with its interface's vtable pointer (`repr(C)`),
        // and its one reference passes to the `ComPtr`.
        unsafe { ComPtr::from_nonnull(NonNull::from(Box::leak(obj)).cast()) }
    }

    /// The object `this` points to.
    ///
    /// # Safety
    /// `this` must point to a live `ComObject<T>` that outlives `'a` — as it does for
    /// the duration of a method the SDK calls through this class's vtable.
    unsafe fn from_this<'a>(this: *mut c_void) -> &'a Self {
        unsafe { &*(this as *const Self) }
    }

    /// The state of the object `this` points to.
    ///
    /// # Safety
    /// As for [`from_this`](Self::from_this).
    pub(crate) unsafe fn state<'a>(this: *mut c_void) -> &'a T {
        unsafe { &Self::from_this(this).state }
    }

    /// A new reference to the object `this` points to.
    ///
    /// # Safety
    /// As for [`from_this`](Self::from_this).
    pub(crate) unsafe fn new_ref(this: *mut c_void) -> ComPtr<T::Interface> {
        unsafe {
            Self::add_ref(this);
            ComPtr::from_nonnull(NonNull::new_unchecked(this).cast())
        }
    }

    /// The state of the object `ptr` points to, if it is one of this class's —
    /// recognised by its vtable, which [`ComClass::VTABLE`] must place at a single
    /// address (a `static`).
    ///
    /// # Safety
    /// `ptr` must be null or point to a live COM object that outlives `'a`.
    pub(crate) unsafe fn downcast<'a>(ptr: *mut T::Interface) -> Option<&'a T> {
        // Every COM object begins with its vtable pointer.
        if ptr.is_null() || !std::ptr::eq(unsafe { *(ptr as *const *const T::VTable) }, T::VTABLE) {
            return None;
        }
        Some(unsafe { Self::state(ptr.cast()) })
    }

    unsafe extern "system" fn query_interface(this: *mut c_void, riid: QueryInterfaceRiid, ppv: *mut *mut c_void) -> HRESULT {
        ffi_guard(format_args!("{}::QueryInterface", T::NAME), E_UNEXPECTED, move || unsafe {
            if ppv.is_null() { return E_POINTER; }
            let iid = guid_from_riid(riid);
            if iid == IID_IUNKNOWN || iid == T::IID {
                Self::add_ref(this);
                *ppv = this;
                S_OK
            } else {
                *ppv = std::ptr::null_mut();
                E_NOINTERFACE
            }
        })
    }

    unsafe extern "system" fn add_ref(this: *mut c_void) -> ComUlong {
        let prev = unsafe { Self::from_this(this) }.refcnt.fetch_add(1, Ordering::Relaxed);
        if prev > MAX_REFCOUNT {
            std::process::abort();
        }
        (prev + 1) as ComUlong
    }

    unsafe extern "system" fn release(this: *mut c_void) -> ComUlong {
        // Freeing the object runs the state's `Drop` — user code — so keep a panic
        // inside Rust.
        ffi_guard(format_args!("{}::Release", T::NAME), 0, move || {
            // The borrow ends with this statement: nothing may point into the object
            // once it is freed.
            let prev = unsafe { Self::from_this(this) }.refcnt.fetch_sub(1, Ordering::Release);
            if prev != 1 {
                return (prev - 1) as ComUlong;
            }
            // Order every other reference's last use before the free, as `Arc` does.
            fence(Ordering::Acquire);
            // SAFETY: that was the last reference, so nothing else can reach the
            // object, which `create` allocated as a `Box`.
            drop(unsafe { Box::from_raw(this as *mut Self) });
            0
        })
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! braw_interface {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$fn_meta:meta])*
                fn $m:ident ( $($argn:ident : $argt:ty),* ) -> $ret:tt ;
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
                            Ok((braw_interface!(@iargpassret2 self; $impls2_ret1,out1), braw_interface!(@iargpassret2 self; $impls2_ret2,out2)))
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
                            Ok((braw_interface!(@iargpassret2 self; $impls3_ret1,out1), braw_interface!(@iargpassret2 self; $impls3_ret2,out2), braw_interface!(@iargpassret2 self; $impls3_ret3,out3)))
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
    // The status a method's return value carries. `$ret` is matched as a `tt`: a
    // forwarded `ty` fragment is opaque and would never match `HRESULT`, silently
    // discarding every method's failure.
    (@ret $hr:ident HRESULT) => { $hr };
    (@ret $hr:ident $ret:tt) => { S_OK };

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
    // A `bool` out-parameter is a COM `BOOL` (4 bytes) on Windows — allocate at
    // that width so the SDK's store can't overrun the stack (see `SdkBool`).
    (@iargpassret bool) => { $crate::SdkBool };
    (@iargpassret $t:ty) => { $t };
    (@iargpassret2 $self:ident; String,$o:expr) => { unsafe { $crate::take_sdk_string($o) } };
    (@iargpassret2 $self:ident; VariantValue,$o:expr) => { $self.factory.lib.variant_to_rust($o) };
    (@iargpassret2 $self:ident; bool,$o:expr) => { $crate::sdk_bool($o) };
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
    /// Take ownership of one reference to the COM object at `ptr`.
    ///
    /// # Safety
    /// `ptr` must be a live COM interface whose reference the caller transfers.
    pub(crate) unsafe fn from_nonnull(ptr: NonNull<T>) -> Self { Self { ptr } }
    /// Give up the reference without releasing it — to hand it to the SDK.
    pub(crate) fn into_raw(self) -> *mut T { std::mem::ManuallyDrop::new(self).as_raw() }
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
unsafe impl Send for ComPtrRefGuard {}

#[derive(Clone)]
pub struct DropOrderVec<T: Clone>(pub Vec<T>);
impl<T: Clone> DropOrderVec<T> {
    pub fn clone_and_add(&self, v: T) -> Self {
        self.clone_and_extend([v])
    }
    /// A copy with `extra` appended, cloning the existing guards once.
    pub fn clone_and_extend(&self, extra: impl IntoIterator<Item = T>) -> Self {
        let extra = extra.into_iter();
        let mut guards = Vec::with_capacity(self.0.len() + extra.size_hint().0);
        guards.extend(self.0.iter().cloned());
        guards.extend(extra);
        Self(guards)
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

#[cfg(test)]
mod tests {
    use super::DropOrderVec;

    #[test]
    fn clone_and_extend_appends_in_order_and_leaves_the_original() {
        let base = DropOrderVec(vec![1, 2]);
        let more = base.clone_and_extend([3, 4]);
        assert_eq!(more.0, [1, 2, 3, 4]);
        assert_eq!(base.clone_and_add(5).0, [1, 2, 5]);
        assert_eq!(base.0, [1, 2]);
    }
}
