// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

use super::*;

/// An iterator over the `(key, value)` metadata entries of a clip or frame.
pub struct MetadataIterator {
    pub(crate) raw: ComPtr<IBlackmagicRawMetadataIterator>,
    pub(crate) is_first: bool,

    #[allow(dead_code)]
    pub(crate) parent_guards: DropOrderVec<ComPtrRefGuard>,
    pub(crate) factory: Factory,
}
unsafe impl Send for MetadataIterator {}
impl Iterator for MetadataIterator {
    type Item = (String, VariantValue);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            // SAFETY: `Next` takes no arguments.
            match unsafe { self.raw.Next() } {
                Ok(S_FALSE) => return None,
                Err(_) => {
                    log::error!("Failed to advance metadata iterator");
                    return None;
                }
                _ => { }
            }
        } else {
            self.is_first = false;
        }
        let mut key_ptr = std::ptr::null_mut();
        // SAFETY: `key_ptr` receives a string the SDK allocates for the caller.
        unsafe { self.raw.GetKey(&mut key_ptr) }.ok()?;
        // Own the key at once, so that every return below frees it.
        let key = unsafe { take_sdk_string(key_ptr) };

        let value;
        // SAFETY: an initialised `VARIANT` on this stack receives the value, which
        // `variant_to_rust` takes ownership of.
        unsafe {
            let mut var: VARIANT = std::mem::zeroed();
            let _lib = &self.factory.lib;
            #[cfg(not(target_os = "windows"))] let VariantInit  = |a| -> HRESULT { (_lib.VariantInit)(a) };

            VariantInit(&mut var);
            self.raw.GetData(&mut var).ok()?;
            value = self.factory.lib.variant_to_rust(var);
        }

        Some((key, value))
    }
}

///////////////////////////////////////////////////////


/// A processing pipeline available on this system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineIteratorItem {
    /// The pipeline's name
    pub name: String,
    /// The interoperability the pipeline offers
    pub interop: BlackmagicRawInterop,
    /// The pipeline
    pub pipeline: BlackmagicRawPipeline,
}

/// An iterator over the processing pipelines available on this system (see
/// [`Factory::pipeline_iter`]).
#[allow(dead_code)]
pub struct PipelineIterator {
    pub(crate) raw: ComPtr<IBlackmagicRawPipelineIterator>,
    pub(crate) is_first: bool,
    pub(crate) factory: Factory,
}
unsafe impl Send for PipelineIterator {}
impl Iterator for PipelineIterator {
    type Item = PipelineIteratorItem;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            // SAFETY: `Next` takes no arguments.
            match unsafe { self.raw.Next() } {
                Ok(S_FALSE) => return None,
                Err(_) => {
                    log::error!("Failed to advance pipeline iterator");
                    return None;
                }
                _ => { }
            }
        } else {
            self.is_first = false;
        }
        let mut name_ptr = std::ptr::null_mut();
        // SAFETY: `name_ptr` receives a string the SDK allocates for the caller.
        unsafe { self.raw.GetName(&mut name_ptr) }.ok()?;
        // Own the name at once, so that every return below frees it.
        let name = unsafe { take_sdk_string(name_ptr) };
        let mut interop = BlackmagicRawInterop::default();
        let mut pipeline = BlackmagicRawPipeline::default();
        // SAFETY: out-parameters on this stack.
        unsafe {
            self.raw.GetInterop(&mut interop).ok()?;
            self.raw.GetPipeline(&mut pipeline).ok()?;
        }

        Some(PipelineIteratorItem {
            name,
            interop,
            pipeline
        })
    }
}

///////////////////////////////////////////////////////

use std::sync::atomic::AtomicUsize;
/// A device available for a pipeline.
pub struct PipelineDeviceIteratorItem {
    /// The interoperability the device's pipeline offers
    pub interop: BlackmagicRawInterop,
    /// The device's pipeline
    pub pipeline: BlackmagicRawPipeline,

    index: usize,
    iter_index: Arc<AtomicUsize>,
    raw: ComPtr<IBlackmagicRawPipelineDeviceIterator>,
    factory: Factory,
}
impl PipelineDeviceIteratorItem {
    /// Create the device. Only the iterator's current item can, so call this before
    /// advancing the iterator.
    pub fn create_device(&self) -> Result<BlackmagicRawPipelineDevice, BrawError> {
        if self.index != self.iter_index.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(BrawError::Other("Devices cannot be created out of order from the iterator".into()));
        }
        // SAFETY: returns a new device through its out-parameter.
        let raw = unsafe { out_interface(|out| self.raw.CreateDevice(out))? };
        Ok(BlackmagicRawPipelineDevice { raw, factory: self.factory.clone(), parent_guards: vec![].into() } )
    }
}

/// An iterator over the devices available for a pipeline (see
/// [`Factory::pipeline_device_iter`]).
pub struct PipelineDeviceIterator {
    pub(crate) raw: ComPtr<IBlackmagicRawPipelineDeviceIterator>,
    pub(crate) is_first: bool,
    pub(crate) factory: Factory,
    pub(crate) current_index: Arc<AtomicUsize>,
}
unsafe impl Send for PipelineDeviceIterator {}
impl Iterator for PipelineDeviceIterator {
    type Item = PipelineDeviceIteratorItem;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            // SAFETY: `Next` takes no arguments.
            match unsafe { self.raw.Next() } {
                Ok(S_FALSE) => return None,
                Err(_) => {
                    log::error!("Failed to advance pipeline device iterator");
                    return None;
                }
                _ => {
                    self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }
        } else {
            self.is_first = false;
        }
        let current_index = self.current_index.load(std::sync::atomic::Ordering::SeqCst);
        let mut interop = BlackmagicRawInterop::default();
        let mut pipeline = BlackmagicRawPipeline::default();
        // SAFETY: out-parameters on this stack.
        unsafe {
            self.raw.GetInterop(&mut interop).ok()?;
            self.raw.GetPipeline(&mut pipeline).ok()?;
        }

        Some(PipelineDeviceIteratorItem {
            interop,
            pipeline,
            raw: self.raw.clone(),
            index: current_index,
            iter_index: self.current_index.clone(),
            factory: self.factory.clone(),
        })
    }
}
