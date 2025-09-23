use super::*;



pub struct MetadataIterator<'lib> {
    pub(crate) raw: ComPtr<IBlackmagicRawMetadataIterator>,
    pub(crate) is_first: bool,
    pub(crate) lib: &'lib RawLibrary,
}
impl<'lib> Iterator for MetadataIterator<'lib> {
    type Item = (String, VariantValue);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            match self.raw.Next() {
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
        self.raw.GetKey(&mut key_ptr).ok()?;

        let value;
        unsafe {
            let mut var: VARIANT = std::mem::zeroed();
            let _lib = self.lib;
            #[cfg(not(target_os = "windows"))] let VariantInit  = |a| -> HRESULT { (_lib.VariantInit)(a) };

            VariantInit(&mut var);
            self.raw.GetData(&mut var).ok()?;
            value = self.lib.variant_to_rust(var);
        }

        Some((
            BrawString(key_ptr as *mut _).to_string(),
            value
        ))
    }
}

///////////////////////////////////////////////////////


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineIteratorItem {
    pub name: String,
    pub interop: BlackmagicRawInterop,
    pub pipeline: BlackmagicRawPipeline,
}

#[allow(dead_code)]
pub struct PipelineIterator<'lib> {
    pub(crate) raw: ComPtr<IBlackmagicRawPipelineIterator>,
    pub(crate) is_first: bool,
    pub(crate) lib: &'lib RawLibrary,
}
impl<'lib> Iterator for PipelineIterator<'lib> {
    type Item = PipelineIteratorItem;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            match self.raw.Next() {
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
        let mut name_ptr = std::ptr::null_mut();
        self.raw.GetName(&mut name_ptr).ok()?;
        let mut interop = BlackmagicRawInterop::default();
        self.raw.GetInterop(&mut interop).ok()?;
        let mut pipeline = BlackmagicRawPipeline::default();
        self.raw.GetPipeline(&mut pipeline).ok()?;

        Some(PipelineIteratorItem {
            name: BrawString(name_ptr as *mut _).to_string(),
            interop,
            pipeline
        })
    }
}

///////////////////////////////////////////////////////
use std::sync::atomic::AtomicUsize;
pub struct PipelineDeviceIteratorItem<'lib> {
    pub interop: BlackmagicRawInterop,
    pub pipeline: BlackmagicRawPipeline,

    index: usize,
    iter_index: Arc<AtomicUsize>,
    raw: ComPtr<IBlackmagicRawPipelineDeviceIterator>,
    lib: &'lib RawLibrary,
}
impl<'lib> PipelineDeviceIteratorItem<'lib> {
    pub fn create_device(&self) -> Result<BlackmagicRawPipelineDevice<'lib>, BrawError> {
        let mut ptr = std::ptr::null_mut();
        if self.index != self.iter_index.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(BrawError::Other("Devices cannot be created out of order from the iterator".into()));
        }
        let _ = self.raw.CreateDevice(&mut ptr)?;
        Ok(BlackmagicRawPipelineDevice { raw: ComPtr::new(ptr)?, lib: self.lib } )
    }
}

pub struct PipelineDeviceIterator<'lib> {
    pub(crate) raw: ComPtr<IBlackmagicRawPipelineDeviceIterator>,
    pub(crate) is_first: bool,
    pub(crate) lib: &'lib RawLibrary,
    pub(crate) current_index: Arc<AtomicUsize>,
}
impl<'lib> Iterator for PipelineDeviceIterator<'lib> {
    type Item = PipelineDeviceIteratorItem<'lib>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_first {
            match self.raw.Next() {
                Ok(S_FALSE) => return None,
                Err(_) => {
                    log::error!("Failed to advance metadata iterator");
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
        self.raw.GetInterop(&mut interop).ok()?;
        let mut pipeline = BlackmagicRawPipeline::default();
        self.raw.GetPipeline(&mut pipeline).ok()?;

        Some(PipelineDeviceIteratorItem {
            interop,
            pipeline,
            raw: self.raw.clone(),
            index: current_index,
            iter_index: self.current_index.clone(),
            lib: self.lib,
        })
    }
}
