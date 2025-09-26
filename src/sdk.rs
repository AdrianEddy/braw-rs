// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2025 Adrian <adrian.eddy at gmail>

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

// Based on Blackmagic RAW SDK 5.0.0

pub use crate::*;
use core::ffi::c_void;

pub(crate) const IID_IBlackmagicRaw:                          GUID = GUID::new([/* 558ABA39-B344-4E9B-A484-116CF2A4B5C6 */ 0x55,0x8A,0xBA,0x39,0xB3,0x44,0x4E,0x9B,0xA4,0x84,0x11,0x6C,0xF2,0xA4,0xB5,0xC6 ]);
pub(crate) const IID_IBlackmagicRawFactory:                   GUID = GUID::new([/* 78DEEB84-98C9-434A-B7E5-7AACC2988399 */ 0x78,0xDE,0xEB,0x84,0x98,0xC9,0x43,0x4A,0xB7,0xE5,0x7A,0xAC,0xC2,0x98,0x83,0x99 ]);
pub(crate) const IID_IBlackmagicRawPipelineIterator:          GUID = GUID::new([/* 051ED792-3D9D-4ED0-BB1F-3873E08773CB */ 0x05,0x1E,0xD7,0x92,0x3D,0x9D,0x4E,0xD0,0xBB,0x1F,0x38,0x73,0xE0,0x87,0x73,0xCB ]);
pub(crate) const IID_IBlackmagicRawPipelineDeviceIterator:    GUID = GUID::new([/* 32D3385F-06EE-4260-82EB-2BABFFFACED8 */ 0x32,0xD3,0x38,0x5F,0x06,0xEE,0x42,0x60,0x82,0xEB,0x2B,0xAB,0xFF,0xFA,0xCE,0xD8 ]);
pub(crate) const IID_IBlackmagicRawOpenGLInteropHelper:       GUID = GUID::new([/* 86444C8A-4398-4364-9166-7D10F41C315E */ 0x86,0x44,0x4C,0x8A,0x43,0x98,0x43,0x64,0x91,0x66,0x7D,0x10,0xF4,0x1C,0x31,0x5E ]);
pub(crate) const IID_IBlackmagicRawPipelineDevice:            GUID = GUID::new([/* 5C7B0A9B-CF2C-4AB3-84C1-E1C7902360C8 */ 0x5C,0x7B,0x0A,0x9B,0xCF,0x2C,0x4A,0xB3,0x84,0xC1,0xE1,0xC7,0x90,0x23,0x60,0xC8 ]);
pub(crate) const IID_IBlackmagicRawToneCurve:                 GUID = GUID::new([/* 7E40C13D-3575-46B5-B2B7-85DAE1EEFD77 */ 0x7E,0x40,0xC1,0x3D,0x35,0x75,0x46,0xB5,0xB2,0xB7,0x85,0xDA,0xE1,0xEE,0xFD,0x77 ]);
pub(crate) const IID_IBlackmagicRawConfiguration:             GUID = GUID::new([/* 267E9866-FB40-4BFB-8BF8-96EA3F7DA36E */ 0x26,0x7E,0x98,0x66,0xFB,0x40,0x4B,0xFB,0x8B,0xF8,0x96,0xEA,0x3F,0x7D,0xA3,0x6E ]);
pub(crate) const IID_IBlackmagicRawConfigurationEx:           GUID = GUID::new([/* ACE9078F-ABA0-4B26-A954-EDA108DADA5A */ 0xAC,0xE9,0x07,0x8F,0xAB,0xA0,0x4B,0x26,0xA9,0x54,0xED,0xA1,0x08,0xDA,0xDA,0x5A ]);
pub(crate) const IID_IBlackmagicRawClipGeometry:              GUID = GUID::new([/* 22717196-36AE-4B8A-B5CC-24292F9660F0 */ 0x22,0x71,0x71,0x96,0x36,0xAE,0x4B,0x8A,0xB5,0xCC,0x24,0x29,0x2F,0x96,0x60,0xF0 ]);
pub(crate) const IID_IBlackmagicRawResourceManager:           GUID = GUID::new([/* 3C5C3C4A-812C-4AF0-99F0-06C6E197C189 */ 0x3C,0x5C,0x3C,0x4A,0x81,0x2C,0x4A,0xF0,0x99,0xF0,0x06,0xC6,0xE1,0x97,0xC1,0x89 ]);
pub(crate) const IID_IBlackmagicRawMetadataIterator:          GUID = GUID::new([/* F85AE78D-5DC2-40BC-8C1D-D0D805523ADA */ 0xF8,0x5A,0xE7,0x8D,0x5D,0xC2,0x40,0xBC,0x8C,0x1D,0xD0,0xD8,0x05,0x52,0x3A,0xDA ]);
pub(crate) const IID_IBlackmagicRawClipProcessingAttributes:  GUID = GUID::new([/* 1F53C8AE-2295-4C8E-B17F-5931F4F146AC */ 0x1F,0x53,0xC8,0xAE,0x22,0x95,0x4C,0x8E,0xB1,0x7F,0x59,0x31,0xF4,0xF1,0x46,0xAC ]);
pub(crate) const IID_IBlackmagicRawFrameProcessingAttributes: GUID = GUID::new([/* 5F7C5C0F-7138-445A-9D0D-6111B6409D17 */ 0x5F,0x7C,0x5C,0x0F,0x71,0x38,0x44,0x5A,0x9D,0x0D,0x61,0x11,0xB6,0x40,0x9D,0x17 ]);
pub(crate) const IID_IBlackmagicRawPost3DLUT:                 GUID = GUID::new([/* 86052BC4-0231-48C6-B3C8-C771112AAD68 */ 0x86,0x05,0x2B,0xC4,0x02,0x31,0x48,0xC6,0xB3,0xC8,0xC7,0x71,0x11,0x2A,0xAD,0x68 ]);
pub(crate) const IID_IBlackmagicRawProcessedImage:            GUID = GUID::new([/* D87A0F72-A883-42BB-8488-0089411C5035 */ 0xD8,0x7A,0x0F,0x72,0xA8,0x83,0x42,0xBB,0x84,0x88,0x00,0x89,0x41,0x1C,0x50,0x35 ]);
pub(crate) const IID_IBlackmagicRawJob:                       GUID = GUID::new([/* 34C05ACF-7118-45EA-8B71-887E0515395D */ 0x34,0xC0,0x5A,0xCF,0x71,0x18,0x45,0xEA,0x8B,0x71,0x88,0x7E,0x05,0x15,0x39,0x5D ]);
pub(crate) const IID_IBlackmagicRawReadJobHints:              GUID = GUID::new([/* 1069F99C-A4E2-415A-91C4-5E0CE0C6AF77 */ 0x10,0x69,0xF9,0x9C,0xA4,0xE2,0x41,0x5A,0x91,0xC4,0x5E,0x0C,0xE0,0xC6,0xAF,0x77 ]);
pub(crate) const IID_IBlackmagicRawCallback:                  GUID = GUID::new([/* E9F98FAC-33DB-4A65-BB94-8A82B027AED0 */ 0xE9,0xF9,0x8F,0xAC,0x33,0xDB,0x4A,0x65,0xBB,0x94,0x8A,0x82,0xB0,0x27,0xAE,0xD0 ]);
pub(crate) const IID_IBlackmagicRawClipAudio:                 GUID = GUID::new([/* 76D4ACED-E0D6-45BB-B547-56B7435B2A1D */ 0x76,0xD4,0xAC,0xED,0xE0,0xD6,0x45,0xBB,0xB5,0x47,0x56,0xB7,0x43,0x5B,0x2A,0x1D ]);
pub(crate) const IID_IBlackmagicRawClipAccelerometerMotion:   GUID = GUID::new([/* 983AACBB-F469-40C9-AA81-345B0B7CCD05 */ 0x98,0x3A,0xAC,0xBB,0xF4,0x69,0x40,0xC9,0xAA,0x81,0x34,0x5B,0x0B,0x7C,0xCD,0x05 ]);
pub(crate) const IID_IBlackmagicRawClipGyroscopeMotion:       GUID = GUID::new([/* 00543A2C-FDED-4C79-A60C-A460415F0296 */ 0x00,0x54,0x3A,0x2C,0xFD,0xED,0x4C,0x79,0xA6,0x0C,0xA4,0x60,0x41,0x5F,0x02,0x96 ]);
pub(crate) const IID_IBlackmagicRawClipPDAFData:              GUID = GUID::new([/* AAE71534-F951-4062-B6A9-69B8808CD267 */ 0xAA,0xE7,0x15,0x34,0xF9,0x51,0x40,0x62,0xB6,0xA9,0x69,0xB8,0x80,0x8C,0xD2,0x67 ]);
pub(crate) const IID_IBlackmagicRawFrame:                     GUID = GUID::new([/* A500B253-1808-4EF2-8692-D23C692404EA */ 0xA5,0x00,0xB2,0x53,0x18,0x08,0x4E,0xF2,0x86,0x92,0xD2,0x3C,0x69,0x24,0x04,0xEA ]);
pub(crate) const IID_IBlackmagicRawFrameEx:                   GUID = GUID::new([/* F8C6C374-D7FB-4BD3-AD0B-C533464FF450 */ 0xF8,0xC6,0xC3,0x74,0xD7,0xFB,0x4B,0xD3,0xAD,0x0B,0xC5,0x33,0x46,0x4F,0xF4,0x50 ]);
pub(crate) const IID_IBlackmagicRawFrameMultiVideo:           GUID = GUID::new([/* 96AE962B-30A4-4904-813D-A9EFFE5953C2 */ 0x96,0xAE,0x96,0x2B,0x30,0xA4,0x49,0x04,0x81,0x3D,0xA9,0xEF,0xFE,0x59,0x53,0xC2 ]);
pub(crate) const IID_IBlackmagicRawManualDecoderFlow1:        GUID = GUID::new([/* 278815A6-A3C1-47C7-A0A6-6754DEAE5E7A */ 0x27,0x88,0x15,0xA6,0xA3,0xC1,0x47,0xC7,0xA0,0xA6,0x67,0x54,0xDE,0xAE,0x5E,0x7A ]);
pub(crate) const IID_IBlackmagicRawManualDecoderFlow2:        GUID = GUID::new([/* DBEC4C39-B4C2-4A65-AA8C-2B3C7F4777E3 */ 0xDB,0xEC,0x4C,0x39,0xB4,0xC2,0x4A,0x65,0xAA,0x8C,0x2B,0x3C,0x7F,0x47,0x77,0xE3 ]);
pub(crate) const IID_IBlackmagicRawClip:                      GUID = GUID::new([/* A2910203-787B-4BF2-A374-B1A459E2D351 */ 0xA2,0x91,0x02,0x03,0x78,0x7B,0x4B,0xF2,0xA3,0x74,0xB1,0xA4,0x59,0xE2,0xD3,0x51 ]);
pub(crate) const IID_IBlackmagicRawClipEx:                    GUID = GUID::new([/* D260C7D0-93BD-4D68-B600-93B4CAB7F870 */ 0xD2,0x60,0xC7,0xD0,0x93,0xBD,0x4D,0x68,0xB6,0x00,0x93,0xB4,0xCA,0xB7,0xF8,0x70 ]);
pub(crate) const IID_IBlackmagicRawClipMultiVideo:            GUID = GUID::new([/* C699E3E2-3268-4E08-8037-D5C4A2C5CE52 */ 0xC6,0x99,0xE3,0xE2,0x32,0x68,0x4E,0x08,0x80,0x37,0xD5,0xC4,0xA2,0xC5,0xCE,0x52 ]);
pub(crate) const IID_IBlackmagicRawClipImmersiveVideo:        GUID = GUID::new([/* 47D287A7-9148-4659-BFCF-C767A089A200 */ 0x47,0xD2,0x87,0xA7,0x91,0x48,0x46,0x59,0xBF,0xCF,0xC7,0x67,0xA0,0x89,0xA2,0x00 ]);
pub(crate) const IID_IBlackmagicRawClipResolutions:           GUID = GUID::new([/* C63C290F-525B-4EBE-AB56-87B010CACE19 */ 0xC6,0x3C,0x29,0x0F,0x52,0x5B,0x4E,0xBE,0xAB,0x56,0x87,0xB0,0x10,0xCA,0xCE,0x19 ]);

// Enums

/// Variant types that may be stored as metadata.
///
/// These types define the possible data types that can be stored in VARIANT structures when working with metadata in Blackmagic RAW files.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawVariantType {
    #[default]
    /// Undefined type
    Empty     = VT_EMPTY     as u32,
    /// Unsigned 8 bit integer
    U8        = VT_UI1       as u32,
    /// Signed 16 bit integer
    S16       = VT_I2        as u32,
    /// Unsigned 16 bit integer
    U16       = VT_UI2       as u32,
    /// Signed 32 bit integer
    S32       = VT_I4        as u32,
    /// Unsigned 32 bit integer
    U32       = VT_UI4       as u32,
    /// Single precision 32 bit (IEEE 754) floating point number
    Float32   = VT_R4        as u32,
    /// String variable
    String    = VT_BSTR      as u32,
    /// Array variable
    SafeArray = VT_SAFEARRAY as u32,
    /// Double precision 64 bit (IEEE 754) floating point number
    Float64   = VT_R8        as u32,
}

/// Resource types used in IBlackmagicRawResourceManager.
///
/// Defines the different types of memory resources that can be allocated for CPU and GPU processing.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResourceType {
    #[default]
    Null = 0,
    /// Page aligned CPU addressable memory
    BufferCPU    = /* 'cpub' */ 0x63707562,
    /// Metal MTLBuffer
    BufferMetal  = /* 'metb' */ 0x6D657462,
    /// CUDA CUdeviceptr device pointer
    BufferCUDA   = /* 'cudb' */ 0x63756462,
    /// OpenCL cl_mem buffer object
    BufferOpenCL = /* 'oclb' */ 0x6F636C62
}

/// Resource formats used for resource allocation.
///
/// Defines the pixel formats that can be used when allocating image resources
/// for processing Blackmagic RAW frames.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResourceFormat {
    #[default]
    Null = 0,
    /// Unsigned 8bit interleaved RGBA
    RGBAU8       = /* 'rgba' */ 0x72676261,
    /// Unsigned 8bit interleaved BGRA
    BGRAU8       = /* 'bgra' */ 0x62677261,
    /// Unsigned 16bit interleaved RGB
    RGBU16       = /* '16il' */ 0x3136696C,
    /// Unsigned 16bit interleaved RGBA
    RGBAU16      = /* '16al' */ 0x3136616C,
    /// Unsigned 16bit interleaved BGRA
    BGRAU16      = /* '16la' */ 0x31366C61,
    /// Unsigned 16bit planar RGB
    RGBU16Planar = /* '16pl' */ 0x3136706C,
    /// Single precision floating point interleaved RGB
    RGBF32       = /* 'f32s' */ 0x66333273,
    /// Single precision floating point interleaved RGBA
    RGBAF32      = /* 'f32l' */ 0x6633326C,
    /// Single precision floating point interleaved BGRA
    BGRAF32      = /* 'f32a' */ 0x66333261,
    /// Single precision floating point planar RGB
    RGBF32Planar = /* 'f32p' */ 0x66333270,
    /// Half precision floating point interleaved RGB
    RGBF16       = /* 'f16s' */ 0x66313673,
    /// Half precision floating point interleaved RGBA
    RGBAF16      = /* 'f16l' */ 0x6631366C,
    /// Half precision floating point interleaved BGRA
    BGRAF16      = /* 'f16a' */ 0x66313661,
    /// Half precision floating point planar RGB
    RGBF16Planar = /* 'f16p' */ 0x66313670,
}

/// Resource usage modes used in IBlackmagicRawResourceManager.
///
/// Defines how resources will be accessed by the CPU and GPU.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResourceUsage {
    #[default]
    Null = 0,
    /// CPU readable and writable memory
    ReadCPUWriteCPU = /* 'rcwc' */ 0x72637763,
    /// GPU readable and writable memory
    ReadGPUWriteGPU = /* 'rgwg' */ 0x72677767,
    /// GPU readable, CPU writable memory
    ReadGPUWriteCPU = /* 'rgwc' */ 0x72677763,
    /// CPU readable, GPU writable memory
    ReadCPUWriteGPU = /* 'rcwg' */ 0x72637767
}

/// Pipeline types used in IBlackmagicRawConfiguration.
///
/// Each pipeline has different mappings to context/commandQueue for GPU acceleration.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawPipeline {
    #[default]
    Null = 0,
    /// CPU pipeline with no GPU acceleration
    CPU    = /* 'cpub' */ 0x63707562,
    /// CUDA pipeline, context/commandQueue maps to CUcontext/CUstream
    CUDA   = /* 'cuda' */ 0x63756461,
    /// Metal pipeline, context/commandQueue maps to nil/MTLCommandQueue
    Metal  = /* 'metl' */ 0x6D65746C,
    /// OpenCL pipeline, context/commandQueue maps to cl_context/cl_command_queue
    OpenCL = /* 'opcl' */ 0x6F70636C
}

/// CPU instruction sets used in IBlackmagicRawConfiguration.
///
/// Allows selection of specific CPU instruction sets for optimization.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawInstructionSet {
    #[default]
    Null = 0,
    /// SSE 4.1 CPU Instruction Set
    SSE41 = /* 'se41' */ 0x73653431,
    /// AVX CPU Instruction Set
    AVX   = /* 'avx_' */ 0x6176785F,
    /// AVX2 CPU Instruction Set
    AVX2  = /* 'avx2' */ 0x61767832,
    /// NEON CPU Instruction Set
    NEON  = /* 'neon' */ 0x6E656F6E
}

/// Audio formats used in IBlackmagicRawFileAudio.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawAudioFormat {
    #[default]
    Null = 0,
    /// PCM little endian audio
    PCMLittleEndian = /* 'pcml' */ 0x70636D6C
}

/// Resolution scales used in IBlackmagicRawFrame.
///
/// Allows decoding frames at different resolutions for performance optimization.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResolutionScale {
    #[default]
    Null = 0,
    /// Full Resolution
    Full    = /* 'full' */ 0x66756C6C,
    /// Half height and width
    Half    = /* 'half' */ 0x68616C66,
    /// Quarter height and width
    Quarter = /* 'qrtr' */ 0x71727472,
    /// Eighth height and width
    Eighth  = /* 'eith' */ 0x65697468,
}

/// Clip processing attributes that may be stored as metadata.
///
/// These attributes control clip-level color science and processing parameters.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawClipProcessingAttribute {
    #[default]
    Null = 0,
    /// Blackmagic Color Science generation (u16)
    ColorScienceGen          = /* 'csgn' */ 0x6373676E,
    /// The gamma curve (string)
    Gamma                    = /* 'gama' */ 0x67616D61,
    /// The color gamut (string)
    Gamut                    = /* 'gamt' */ 0x67616D74,
    /// Contrast used in Blackmagic Design Custom Gamma (float)
    ToneCurveContrast        = /* 'tcon' */ 0x74636F6E,
    /// Saturation used in Blackmagic Design Custom Gamma (float)
    ToneCurveSaturation      = /* 'tsat' */ 0x74736174,
    /// Midpoint used in Blackmagic Design Custom Gamma (float)
    ToneCurveMidpoint        = /* 'tmid' */ 0x746D6964,
    /// Highlight rolloff used in Blackmagic Design Custom Gamma (float)
    ToneCurveHighlights      = /* 'thih' */ 0x74686968,
    /// Shadow rolloff used in Blackmagic Design Custom Gamma (float)
    ToneCurveShadows         = /* 'tsha' */ 0x74736861,
    /// VideoBlackLevel used in Blackmagic Design Custom Gamma (u16)
    ToneCurveVideoBlackLevel = /* 'tvbl' */ 0x7476626C,
    /// BlackLevel used in Blackmagic Design Custom Gamma (float)
    ToneCurveBlackLevel      = /* 'tblk' */ 0x74626C6B,
    /// WhiteLevel used in Blackmagic Design Custom Gamma (float)
    ToneCurveWhiteLevel      = /* 'twit' */ 0x74776974,
    /// Is highlight recovery enabled (u16)
    HighlightRecovery        = /* 'hlry' */ 0x686C7279,
    /// Is analog gain constant throughout the clip (u16)
    AnalogGainIsConstant     = /* 'agic' */ 0x61676963,
    /// Analog gain for entire clip if analog gain is constant, otherwise analog gain of the first frame (float)
    AnalogGain               = /* 'gain' */ 0x6761696E,
    /// Is the Post 3D LUT being applied embedded, sidecar or disabled (string)
    Post3DLUTMode            = /* 'lutm' */ 0x6C75746D,
    /// Name of embedded 3D LUT (string)
    EmbeddedPost3DLUTName    = /* 'emln' */ 0x656D6C6E,
    /// Title of embedded 3D LUT (string)
    EmbeddedPost3DLUTTitle   = /* 'emlt' */ 0x656D6C74,
    /// Size of embedded 3D LUT (u16)
    EmbeddedPost3DLUTSize    = /* 'emls' */ 0x656D6C73,
    /// Float array of embedded 3D LUT data (float array, size*size*size*3 elements)
    EmbeddedPost3DLUTData    = /* 'emld' */ 0x656D6C64,
    /// Name of sidecar 3D LUT (string)
    SidecarPost3DLUTName     = /* 'scln' */ 0x73636C6E,
    /// Title of sidecar 3D LUT (string)
    SidecarPost3DLUTTitle    = /* 'sclt' */ 0x73636C74,
    /// Size of sidecar 3D LUT (u16)
    SidecarPost3DLUTSize     = /* 'scls' */ 0x73636C73,
    /// Float array of sidecar 3D LUT data (float array, size*size*size*3 elements)
    SidecarPost3DLUTData     = /* 'scld' */ 0x73636C64,
    /// Enable gamut compression (u16, 0=disabled, 1=enabled)
    GamutCompressionEnable   = /* 'gace' */ 0x67616365
}

/// Frame processing attributes that may be stored as metadata.
///
/// These attributes control per-frame color correction parameters.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawFrameProcessingAttribute {
    #[default]
    Null = 0,
    /// The white balance Kelvin value (u32)
    WhiteBalanceKelvin = /* 'wbkv' */ 0x77626B76,
    /// The white balance Tint value (s16)
    WhiteBalanceTint   = /* 'wbtn' */ 0x7762746E,
    /// The linear exposure adjustment value in stops (float)
    Exposure           = /* 'expo' */ 0x6578706F,
    /// The ISO gamma curve (u32)
    ISO                = /* 'fiso' */ 0x6669736F,
    /// Analog Gain per-frame value, cannot be changed (float)
    AnalogGain         = /* 'agpf' */ 0x61677066
}

/// Immersive video attributes that may be stored as metadata.
///
/// These attributes contain information about immersive/stereoscopic video properties.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawImmersiveAttribute {
    #[default]
    Null = 0,
    /// UUID of the projection data file (string)
    OpticalLensProcessingDataFileUUID = /* 'oldu' */ 0x6F6C6475,
    /// Name of the ILPD projection data file (string)
    OpticalILPDFileName               = /* 'oldf' */ 0x6F6C6466,
    /// Interaxial lens separation (i8)
    OpticalInteraxial                 = /* 'olix' */ 0x6F6C6978,
    /// Projection kind set to 'fish' to indicate Apple immersive video (string)
    OpticalProjectionKind             = /* 'olpk' */ 0x6F6C706B,
    /// Calibration type set to 'meiRives' to indicate ILPD lens projection (string)
    OpticalCalibrationType            = /* 'olct' */ 0x6F6C6374,
    /// The contents of the projection data file (string)
    OpticalProjectionData             = /* 'olpd' */ 0x6F6C7064
}

/// Interoperability modes for graphics APIs.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawInterop {
    #[default]
    Null = 0,
    /// No interoperability
    None   = /* 'none' */ 0x6E6F6E65,
    /// OpenGL interoperability
    OpenGL = /* 'opgl' */ 0x6F70676C
}

/// Anamorphic desqueeze ratios.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawAnamorphicRatio {
    #[default]
    Null = 0,
    /// Use anamorphic ratio from metadata
    FromMetadata   = /* 'meta' */ 0x6D657461,
    /// Disable anamorphic desqueeze
    Disabled       = /* 'dsbl' */ 0x6473626C,
    /// 1.33x anamorphic desqueeze
    Anamorphic133x = /* '133x' */ 0x31333378,
    /// 1.5x anamorphic desqueeze
    Anamorphic15x  = /* '15x_' */ 0x3135785F,
    /// 1.6x anamorphic desqueeze
    Anamorphic16x  = /* '16x_' */ 0x3136785F,
    /// 1.66x anamorphic desqueeze
    Anamorphic166x = /* '166x' */ 0x31363678,
    /// 1.8x anamorphic desqueeze
    Anamorphic18x  = /* '18x_' */ 0x3138785F,
    /// 2x anamorphic desqueeze
    Anamorphic2x   = /* '2x__' */ 0x32785F5F
}

/// Image rotation options.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawRotation {
    #[default]
    Null = 0,
    /// Use rotation from metadata
    FromMetadata = /* 'meta' */ 0x6D657461,
    /// Disable rotation
    Disabled     = /* 'dsbl' */ 0x6473626C,
    /// Rotate 90 degrees clockwise
    Clockwise90  = /* 'r90_' */ 0x7239305F,
    /// Rotate 180 degrees clockwise
    Clockwise180 = /* 'r180' */ 0x72313830,
    /// Rotate 270 degrees clockwise
    Clockwise270 = /* 'r270' */ 0x72323730
}

/// Image flip options.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawFlip {
    #[default]
    Null = 0,
    /// Use flip from metadata
    FromMetadata = /* 'meta' */ 0x6D657461,
    /// Disable flipping
    Disabled     = /* 'dsbl' */ 0x6473626C,
    /// Flip horizontally
    Horizontal   = /* 'flpx' */ 0x666C7078,
    /// Flip vertically
    Vertical     = /* 'flpy' */ 0x666C7079,
    /// Flip both horizontally and vertically
    Both         = /* 'flpb' */ 0x666C7062
}

/// Size limit modes used in IBlackmagicRawConfigurationEx.
///
/// Controls how images are constrained when they exceed size limits.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawSizeLimit {
    #[default]
    Null = 0,
    /// No size limit
    None  = /* 'szln' */ 0x737A6C6E,
    /// Scale image to fit within limits
    Scale = /* 'szsc' */ 0x737A7363,
    /// Crop image to fit within limits
    Crop  = /* 'szlc' */ 0x737A6C63
}

/// Immersive video track identifiers.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawImmersiveVideoTrack {
    #[default]
    /// Left eye video track
    Left  = 0,
    /// Right eye video track
    Right = 1
}

// Interfaces

braw_interface! {
    BlackmagicRaw {
        /// Opens a clip from the specified file path
        fn OpenClip(fileName: *const c_void, out_clip: *mut *mut IBlackmagicRawClip) -> HRESULT;
        /// Opens a clip with specified geometry properties, overriding metadata
        fn OpenClipWithGeometry(fileName: *const c_void, geometry: *mut IBlackmagicRawClipGeometry, out_clip: *mut *mut IBlackmagicRawClip) -> HRESULT;
        /// Registers a callback with the codec object for receiving processing notifications
        fn SetCallback(cb: *mut IBlackmagicRawCallback) -> HRESULT;
        /// Asynchronously prepares the current pipeline for decoding to reduce first-frame latency
        fn PreparePipeline(pipeline: u32, pipelineContext: *mut c_void, pipelineCommandQueue: *mut c_void, userData: *mut c_void) -> HRESULT;
        /// Asynchronously prepares the current pipeline using a device object
        fn PreparePipelineForDevice(device: *mut IBlackmagicRawPipelineDevice, userData: *mut c_void) -> HRESULT;
        /// Blocking call which will only return once all jobs have been completed
        fn FlushJobs() -> HRESULT;
    }
    /// Each codec interface will have its own memory storage and decoder.
    ///
    /// When decoding multiple clips via one codec, first in first out ordering will apply.
    /// This is the main interface for working with Blackmagic RAW files.
    #[derive(Clone)]
    impl {
        void FlushJobs => fn flush_jobs(&mut Self); /// Blocking call which will only return once all jobs have been completed
        interface fn configuration(&self) -> BlackmagicRawConfiguration; /// Get the configuration interface for this codec
        interface fn configuration_ex(&self) -> BlackmagicRawConfigurationEx; /// Get the extended configuration interface for this codec
        interface fn manual_decoder_flow1(&self) -> BlackmagicRawManualDecoderFlow1; /// Get the manual decoder flow 1 (CPU) interface
        interface fn manual_decoder_flow2(&self) -> BlackmagicRawManualDecoderFlow2; /// Get the manual decoder flow 2 (hybrid CPU/GPU) interface
    }
}

braw_interface! {
    /// Use this to create one or more Codec objects.
    ///
    /// The factory is the entry point for creating Blackmagic RAW codec instances and iterating available processing pipelines.
    BlackmagicRawFactory {
        /// Create a codec from the factory
        fn CreateCodec(out_codec: *mut *mut IBlackmagicRaw) -> HRESULT;
        /// Create a pipeline iterator from the factory
        fn CreatePipelineIterator(interop: BlackmagicRawInterop, out_it: *mut *mut IBlackmagicRawPipelineIterator) -> HRESULT;
        /// Create a pipeline device iterator from the factory
        fn CreatePipelineDeviceIterator(pipeline: BlackmagicRawPipeline, interop: BlackmagicRawInterop, out_it: *mut *mut IBlackmagicRawPipelineDeviceIterator) -> HRESULT;
        /// Create empty clip geometry object
        fn CreateClipGeometry(out_geom: *mut *mut IBlackmagicRawClipGeometry) -> HRESULT;
    }
}

braw_interface! {
    BlackmagicRawPipelineIterator {
        /// Step to next pipeline entry. S_FALSE is returned when called on last entry
        fn Next() -> HRESULT;
        /// Get the name of the pipeline
        fn GetName(pipelineName: *mut *mut c_void) -> HRESULT;
        /// Get the interoperability of the pipeline
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        /// Get the pipeline type
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline) -> HRESULT;
    }
    /// Use this to determine pipelines available for use on the system.
    ///
    /// Iterates through available processing pipelines (CPU, CUDA, Metal, OpenCL) that are supported on the current system.
    #[derive(Clone)]
    impl {
        scalar GetName     => fn name(&Self) -> String; /// Get the name of the pipeline
        scalar GetInterop  => fn interop(&Self) -> BlackmagicRawInterop; /// Get the interoperability of the pipeline
        scalar GetPipeline => fn pipeline(&Self) -> BlackmagicRawPipeline; /// Get the pipeline type
        void Next          => fn next(&mut Self); /// Step to next pipeline entry
    }
}

braw_interface! {
    BlackmagicRawPipelineDeviceIterator {
        /// Step to next device entry, will return S_FALSE when called on last entry
        fn Next() -> HRESULT;
        /// Get the pipeline type
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline) -> HRESULT;
        /// Get the interoperability of the device's pipeline
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        /// Create the pipeline device (container for context and command queue)
        fn CreateDevice(pipelineDevice: *mut *mut IBlackmagicRawPipelineDevice) -> HRESULT;
    }
    /// Use this to determine pipeline devices available for use on the system.
    ///
    /// Iterates through available devices for a specific pipeline type, such as multiple GPUs for CUDA or Metal pipelines.
    #[derive(Clone)]
    impl {
        struct CreateDevice => fn create_device(&Self) -> BlackmagicRawPipelineDevice; /// Create the pipeline device
        scalar GetInterop   => fn interop(&Self) -> BlackmagicRawInterop; /// Get the interoperability of the device's pipeline
        scalar GetPipeline  => fn pipeline(&Self) -> BlackmagicRawPipeline; /// Get the pipeline type
        void Next           => fn next(&mut Self); /// Step to next device entry
    }
}

braw_interface! {
    BlackmagicRawOpenGLInteropHelper {
        /// Gets the preferred resource format for interaction between the device and OpenGL
        fn GetPreferredResourceFormat(preferredFormat: *mut BlackmagicRawResourceFormat) -> HRESULT;
        /// Copies the processed image into an OpenGL texture
        fn SetImage(processedImage: *mut IBlackmagicRawProcessedImage, openGLTextureName: *mut u32, openGLTextureTarget: *mut i32) -> HRESULT;
    }
    /// Helper interface for OpenGL interoperability.
    ///
    /// Provides functionality to copy processed images into OpenGL textures for direct GPU rendering without CPU roundtrip.
    #[derive(Clone)]
    impl {
        scalar GetPreferredResourceFormat => fn preferred_resource_format(&Self) -> BlackmagicRawResourceFormat; /// Gets the preferred resource format for OpenGL interop
        scalar2 SetImage                  => fn set_image(&mut Self, processed_image: BlackmagicRawProcessedImage) -> (u32, i32); /// Copies the processed image into an OpenGL texture, returns (texture name, texture target)
    }
}

braw_interface! {
    BlackmagicRawPipelineDevice {
        /// Sets the CPU instruction set of the device to be that representing the best capabilities of the system
        fn SetBestInstructionSet() -> HRESULT;
        /// Sets the CPU instruction set to use for the device
        fn SetInstructionSet(instructionSet: BlackmagicRawInstructionSet) -> HRESULT;
        /// Gets the CPU instruction set of the device
        fn GetInstructionSet(instructionSet: *mut BlackmagicRawInstructionSet) -> HRESULT;
        /// Gets the index of the device in the pipeline's device list
        fn GetIndex(deviceIndex: *mut u32) -> HRESULT;
        /// Gets the name of the device
        fn GetName(deviceName: *mut *mut c_void) -> HRESULT;
        /// Gets the API interoperability of the device
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        /// Gets the pipeline configuration information associated with the device
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline, context: *mut *mut c_void, commandQueue: *mut *mut c_void) -> HRESULT;
        /// Gets the name of the pipeline associated with the device
        fn GetPipelineName(pipelineName: *mut *mut c_void) -> HRESULT;
        /// Creates an instance of a helper to get the results of a processed image as an OpenGL texture
        fn GetOpenGLInteropHelper(/* out */ interopHelper: *mut *mut IBlackmagicRawOpenGLInteropHelper) -> HRESULT;
        /// Gets the list of supported resource formats for the device
        fn GetSupportedResourceFormats(/* out */ array: *mut BlackmagicRawResourceFormat, /* in, out */ arrayElementCount: *mut u32) -> HRESULT;
        /// Returns the maximum texture size
        fn GetMaximumTextureSize(maximumWidth: *mut u32, maximumHeight: *mut u32) -> HRESULT;
    }
    /// A device is essentially a container for a context and command queue associated with a pipeline.
    ///
    /// This object is provided so the user need not deal directly with the underlying compute API
    /// in order to provide context and command queue to the codec configuration. The device instance
    /// MUST outlive that of the codec instance on which the device is used.
    #[derive(Clone)]
    impl {
        struct GetOpenGLInteropHelper=> fn opengl_interop_helper(&Self) -> BlackmagicRawOpenGLInteropHelper; /// Creates an OpenGL interop helper
        scalar GetInstructionSet     => fn instruction_set(&Self) -> BlackmagicRawInstructionSet; /// Gets the CPU instruction set
        scalar GetIndex              => fn index(&Self) -> u32; /// Gets the device index
        scalar GetName               => fn name(&Self) -> String; /// Gets the device name
        scalar GetInterop            => fn interop(&Self) -> BlackmagicRawInterop; /// Gets the API interoperability
        scalar GetPipelineName       => fn pipeline_name(&Self) -> String; /// Gets the pipeline name
        scalar2 GetMaximumTextureSize => fn maximum_texture_size(&Self) -> (u32, u32); /// Returns the maximum texture size (width, height)
        scalar3 GetPipeline          => fn pipeline(&Self) -> (BlackmagicRawPipeline, *mut c_void, *mut c_void); /// Gets pipeline configuration (type, context, command queue)
        void   SetInstructionSet     => fn set_instruction_set(&mut Self, instruction_set: BlackmagicRawInstructionSet); /// Sets the CPU instruction set
        void   SetBestInstructionSet => fn set_best_instruction_set(&mut Self); /// Sets the best available instruction set
        // custom impl GetSupportedResourceFormats
    }
}

braw_interface! {
    BlackmagicRawToneCurve {
        /// Query tone curve parameters for a specific camera and gamma. These are only currently available on Gamut: Blackmagic Design, Gamma: Blackmagic Design Film, Blackmagic Design Extended Video, Blackmagic Design Custom
        fn GetToneCurve(cameraType: *const c_void, gamma: *const c_void, gen_: u16 /* Color science gen */, contrast: *mut f32, saturation: *mut f32, midpoint: *mut f32, highlights: *mut f32, shadows: *mut f32, blackLevel: *mut f32, whiteLevel: *mut f32, videoBlackLevel: *mut u16) -> HRESULT;
        /// Evaluates tone curve, returned buffer can be used to visualise curve
        fn EvaluateToneCurve(cameraType: *const c_void, gen_: u16, contrast: f32, saturation: f32, midpoint: f32, highlights: f32, shadows: f32, blackLevel: f32, whiteLevel: f32, videoBlackLevel: u16, array: *mut f32, arrayElementCount: u32) -> HRESULT;
    }
    /// Functions which provide useful tone curve operations.
    ///
    /// If desired, the user application can cache these results.
    #[derive(Clone)]
    impl {
        // custom impl GetToneCurve
        // custom impl EvaluateToneCurve
    }
}

braw_interface! {
    BlackmagicRawConfiguration {
        /// Set pipeline to use for decoding, changing pipeline will cause the default resource manager to be re-created
        fn SetPipeline(pipeline: BlackmagicRawPipeline, pipelineContext: *mut c_void, pipelineCommandQueue: *mut c_void) -> HRESULT;
        /// Get pipeline used for decoding
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline, pipelineContextOut: *mut *mut c_void, pipelineCommandQueueOut: *mut *mut c_void) -> HRESULT;
        /// Verifies relevant hardware/DLLs are available for the specified pipeline
        fn IsPipelineSupported(pipeline: BlackmagicRawPipeline, pipelineSupported: *mut bool) -> HRESULT;
        /// Sets the number of CPU threads to use while decoding (0 for default)
        fn SetCPUThreads(threadCount: u32) -> HRESULT;
        /// Gets the number of CPU threads to use while decoding
        fn GetCPUThreads(threadCount: *mut u32) -> HRESULT;
        /// Query the number of hardware threads available on system
        fn GetMaxCPUThreadCount(threadCount: *mut u32) -> HRESULT;
        /// If true, frame metadata will be written to only the relevant frame
        fn SetWriteMetadataPerFrame(writePerFrame: bool) -> HRESULT;
        /// Gets if the per-frame metadata will be written to only the relevant frame
        fn GetWriteMetadataPerFrame(writePerFrame: *mut bool) -> HRESULT;
        /// Equivalent to querying the device for instruction set, pipeline, context and command queue then calling SetInstructionSet and SetPipeline
        fn SetFromDevice(pipelineDevice: *mut IBlackmagicRawPipelineDevice) -> HRESULT;
        /// Get the Blackmagic RAW SDK version
        fn GetVersion(version: *mut *mut c_void) -> HRESULT;
        /// Get the camera support version
        fn GetCameraSupportVersion(version: *mut *mut c_void) -> HRESULT;
    }
    /// Configuration for Codec Object. Configuration properties are read on first OpenClip().
    ///
    /// The configuration properties are read when the first call to OpenClip() occurs.
    /// After this configuration properties should not be changed, and changes will be ignored.
    #[derive(Clone)]
    impl {
        scalar GetVersion              => fn version(&Self) -> String; /// Get the Blackmagic RAW SDK version
        scalar GetCameraSupportVersion => fn camera_support_version(&Self) -> String; /// Get the camera support version
        scalar GetCPUThreads           => fn cpu_threads(&Self) -> u32; /// Gets the number of CPU threads to use while decoding
        scalar GetMaxCPUThreadCount    => fn max_cpu_thread_count(&Self) -> u32; /// Query the number of hardware threads available on system
        scalar GetWriteMetadataPerFrame=> fn write_metadata_per_frame(&Self) -> bool; /// Gets if metadata is written per-frame
        scalar IsPipelineSupported     => fn is_pipeline_supported(&Self, pipeline: BlackmagicRawPipeline) -> bool; /// Verifies if a pipeline is supported
        scalar3 GetPipeline            => fn pipeline(&Self) -> (BlackmagicRawPipeline, *mut c_void, *mut c_void); /// Get pipeline configuration (type, context, command queue)
        void   SetPipeline             => fn set_pipeline(&mut Self, pipeline: BlackmagicRawPipeline, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void); /// Set pipeline to use for decoding
        void   SetCPUThreads           => fn set_cpu_threads(&mut Self, thread_count: u32); /// Sets the number of CPU threads to use
        void   SetWriteMetadataPerFrame=> fn set_write_metadata_per_frame(&mut Self, write_per_frame: bool); /// Sets if metadata is written per-frame
        void   SetFromDevice           => fn set_from_device(&mut Self, pipeline_device: BlackmagicRawPipelineDevice); /// Configure from a device object
    }
}

braw_interface! {
    BlackmagicRawConfigurationEx {
        /// Get the current resource manager
        fn GetResourceManager(resourceManager: *mut *mut IBlackmagicRawResourceManager) -> HRESULT;
        /// Set the current resource manager, setting null will restore the default resource manager
        fn SetResourceManager(resourceManager: *mut IBlackmagicRawResourceManager) -> HRESULT;
        /// Get the CPU instruction set used by the decoder
        fn GetInstructionSet(instructionSet: *mut BlackmagicRawInstructionSet) -> HRESULT;
        /// Set the CPU instruction set used by the decoder
        fn SetInstructionSet(instructionSet: BlackmagicRawInstructionSet) -> HRESULT;
        /// Get imposed resolution size limitation
        fn GetSizeLimit(sizeLimit: *mut BlackmagicRawSizeLimit, sizeLimitWidth: *mut u32, sizeLimitHeight: *mut u32) -> HRESULT;
        /// Set resolution size limitation, ensuring any resulting imagery does not exceed these limits
        fn SetSizeLimit(sizeLimit: BlackmagicRawSizeLimit, sizeLimitWidth: u32, sizeLimitHeight: u32) -> HRESULT;
    }
    /// Extended Configuration for Codec Object.
    ///
    /// Provides additional configuration options beyond the basic configuration interface.
    #[derive(Clone)]
    impl {
        struct GetResourceManager => fn resource_manager(&Self) -> BlackmagicRawResourceManager; /// Get the current resource manager
        scalar GetInstructionSet  => fn instruction_set(&Self) -> BlackmagicRawInstructionSet; /// Get the CPU instruction set
        scalar3 GetSizeLimit      => fn size_limit(&Self) -> (BlackmagicRawSizeLimit, u32, u32); /// Get size limit (mode, width, height)
        void   SetResourceManager => fn set_resource_manager(&mut Self, resource_manager: BlackmagicRawResourceManager); /// Set the resource manager
        void   SetInstructionSet  => fn set_instruction_set(&mut Self, instruction_set: BlackmagicRawInstructionSet); /// Set the CPU instruction set
        void   SetSizeLimit       => fn set_size_limit(&mut Self, size_limit: BlackmagicRawSizeLimit, size_limit_width: u32, size_limit_height: u32); /// Set resolution size limits
    }
}

braw_interface! {
    BlackmagicRawClipGeometry {
        /// Get the anamorphic ratio
        fn GetAnamorphicRatio(anamorphic: *mut BlackmagicRawAnamorphicRatio) -> HRESULT;
        /// Set the anamorphic ratio
        fn SetAnamorphicRatio(anamorphic: BlackmagicRawAnamorphicRatio) -> HRESULT;
        /// Get the rotation state
        fn GetRotation(rotation: *mut BlackmagicRawRotation) -> HRESULT;
        /// Set the rotation state
        fn SetRotation(rotation: BlackmagicRawRotation) -> HRESULT;
        /// Get the flip state
        fn GetFlip(flip: *mut BlackmagicRawFlip) -> HRESULT;
        /// Set the flip state
        fn SetFlip(flip: BlackmagicRawFlip) -> HRESULT;
    }
    /// Geometry properties of Clip object.
    ///
    /// Controls rotation, flip, and anamorphic properties of the clip.
    #[derive(Clone)]
    impl {
        scalar GetAnamorphicRatio => fn anamorphic_ratio(&Self) -> BlackmagicRawAnamorphicRatio; /// Get the anamorphic ratio
        scalar GetRotation        => fn rotation(&Self) -> BlackmagicRawRotation; /// Get the rotation state
        scalar GetFlip            => fn flip(&Self) -> BlackmagicRawFlip; /// Get the flip state
        void   SetAnamorphicRatio => fn set_anamorphic_ratio(&mut Self, anamorphic: BlackmagicRawAnamorphicRatio); /// Set the anamorphic ratio
        void   SetRotation        => fn set_rotation(&mut Self, rotation: BlackmagicRawRotation); /// Set the rotation state
        void   SetFlip            => fn set_flip(&mut Self, flip: BlackmagicRawFlip); /// Set the flip state
    }
}

braw_interface! {
    BlackmagicRawResourceManager {
        /// Called when a new resource is created
        fn CreateResource(context: *mut c_void, commandQueue: *mut c_void, sizeBytes: u32, typ: BlackmagicRawResourceType, usage: BlackmagicRawResourceUsage, resource: *mut *mut c_void) -> HRESULT;
        /// Release a resource
        fn ReleaseResource(context: *mut c_void, commandQueue: *mut c_void, resource: *mut c_void, typ: BlackmagicRawResourceType) -> HRESULT;
        /// Copy a resource. Utility function, not called by the API
        fn CopyResource(context: *mut c_void, commandQueue: *mut c_void, source: *mut c_void, sourceType: BlackmagicRawResourceType, destination: *mut c_void, destinationType: BlackmagicRawResourceType, sizeBytes: u32, copyAsync: bool) -> HRESULT;
        /// Obtains a pointer to a resource's host addressable memory. Utility function, not called by the API
        fn GetResourceHostPointer(context: *mut c_void, commandQueue: *mut c_void, resource: *mut c_void, resourceType: BlackmagicRawResourceType, hostPointer: *mut *mut c_void) -> HRESULT;
    }
    /// Resource manager allows manual handling of CPU/GPU resource allocation.
    ///
    /// Using this interface the user can create their own Resource manager to allow ownership
    /// over resource allocations. An internal resource manager that implements this interface is provided by default.
    #[derive(Clone)]
    impl {
        scalar CreateResource      => fn create_resource(&Self, context: *mut c_void, command_queue: *mut c_void, size_bytes: u32, typ: BlackmagicRawResourceType, usage: BlackmagicRawResourceUsage) -> *mut c_void; /// Create a new resource
        scalar GetResourceHostPointer => fn resource_host_pointer(&Self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, resource_type: BlackmagicRawResourceType) -> *mut c_void; /// Get host pointer to resource
        void   CopyResource        => fn copy_resource(&mut Self, context: *mut c_void, command_queue: *mut c_void, source: *mut c_void, source_type: BlackmagicRawResourceType, destination: *mut c_void, destination_type: BlackmagicRawResourceType, size_bytes: u32, copy_async: bool); /// Copy between resources
        void   ReleaseResource     => fn release_resource(&mut Self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, typ: BlackmagicRawResourceType); /// Release a resource
    }
}

braw_interface! {
    BlackmagicRawMetadataIterator {
        /// Step to next metadata entry, will return S_FALSE when called on last entry
        fn Next() -> HRESULT;
        /// Query key name of this metadata entry
        fn GetKey(key: *mut *mut c_void) -> HRESULT;
        /// Query data in this metadata entry
        fn GetData(data: *mut VARIANT) -> HRESULT;
    }
    /// Iterating metadata.
    ///
    /// Allows iteration through all metadata key-value pairs in a clip or frame.
    #[derive(Clone)]
    impl {
        scalar GetKey  => fn key(&Self) -> String; /// Get the metadata key
        scalar GetData => fn data(&Self) -> VariantValue; /// Get the metadata value
        void  Next     => fn next(&mut Self); /// Move to next metadata entry
    }
}

braw_interface! {
    BlackmagicRawClipProcessingAttributes {
        /// Get the attribute
        fn GetClipAttribute(attribute: BlackmagicRawClipProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        /// Set the attribute
        fn SetClipAttribute(attribute: BlackmagicRawClipProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        /// Get the clip processing attribute range for the specified attribute
        fn GetClipAttributeRange(attribute: BlackmagicRawClipProcessingAttribute, valueMin: *mut VARIANT, valueMax: *mut VARIANT, isReadOnly: *mut bool) -> HRESULT;
        /// Get the clip processing attribute value list for the specified attribute
        fn GetClipAttributeList(attribute: BlackmagicRawClipProcessingAttribute, array: *mut VARIANT, arrayElementCount: *mut u32, isReadOnly: *mut bool) -> HRESULT;
        /// Obtains a list of available ISOs (for the clip's analog gain) for GUI presentation
        fn GetISOList(array: *mut u32, arrayElementCount: *mut u32, isReadOnly: *mut bool) -> HRESULT;
        /// Get the active 3D LUT
        fn GetPost3DLUT(lut: *mut *mut IBlackmagicRawPost3DLUT) -> HRESULT;
    }
    /// Clip attributes used during processing.
    ///
    /// Allows adjustment of clip-level color science and processing parameters.
    #[derive(Clone)]
    impl {
        struct GetPost3DLUT     => fn post_3d_lut(&Self) -> BlackmagicRawPost3DLUT; /// Get the active 3D LUT
        scalar GetClipAttribute => fn attribute(&Self, attribute: BlackmagicRawClipProcessingAttribute) -> VariantValue; /// Get a clip attribute value
        void   SetClipAttribute => fn set_attribute(&Self, attribute: BlackmagicRawClipProcessingAttribute, value: VariantValue); /// Set a clip attribute value
        // custom impl GetClipAttributeRange
        // custom impl GetClipAttributeList
        // custom impl GetISOList
    }
}

braw_interface! {
    BlackmagicRawFrameProcessingAttributes {
        /// Get the attribute
        fn GetFrameAttribute(attribute: BlackmagicRawFrameProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        /// Set the attribute
        fn SetFrameAttribute(attribute: BlackmagicRawFrameProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        /// Get the frame processing attribute range for the specified attribute
        fn GetFrameAttributeRange(attribute: BlackmagicRawFrameProcessingAttribute, valueMin: *mut VARIANT, valueMax: *mut VARIANT, isReadOnly: *mut bool) -> HRESULT;
        /// Get the frame processing attribute value list for the specified attribute
        fn GetFrameAttributeList(attribute: BlackmagicRawFrameProcessingAttribute, array: *mut VARIANT, arrayElementCount: *mut u32, isReadOnly: *mut bool) -> HRESULT;
        /// Obtains a list of available ISOs (for the frame's analog gain) for GUI presentation
        fn GetISOList(array: *mut u32, arrayElementCount: *mut u32, isReadOnly: *mut bool) -> HRESULT;
    }
    /// Frame attributes used during processing.
    ///
    /// Processing attributes which can change per frame.
    #[derive(Clone)]
    impl {
        scalar GetFrameAttribute => fn attribute(&Self, attribute: BlackmagicRawFrameProcessingAttribute) -> VariantValue; /// Get a frame attribute value
        void   SetFrameAttribute => fn set_attribute(&Self, attribute: BlackmagicRawFrameProcessingAttribute, value: VariantValue); /// Set a frame attribute value
        // custom impl GetFrameAttributeRange
        // custom impl GetFrameAttributeList
        // custom impl GetISOList
    }
}

braw_interface! {
    BlackmagicRawPost3DLUT {
        /// Get the name of the 3D LUT
        fn GetName(name: *mut *mut c_void) -> HRESULT;
        /// Get the title of the 3D LUT
        fn GetTitle(title: *mut *mut c_void) -> HRESULT;
        /// Get the size of the LUT (e.g., will return 17 for a 17x17x17 LUT)
        fn GetSize(size: *mut u32) -> HRESULT;
        /// Get pointer to GPU resource the LUT is stored in
        fn GetResourceGPU(context: *mut c_void, commandQueue: *mut c_void, typ_: *mut BlackmagicRawResourceType, resource: *mut *mut c_void) -> HRESULT;
        /// Get pointer to CPU resource the LUT is stored in
        fn GetResourceCPU(resource: *mut *mut c_void) -> HRESULT;
        /// Get size of resource in bytes
        fn GetResourceSizeBytes(sizeBytes: *mut u32) -> HRESULT;
        /// Write LUT as cube file
        fn WriteCubeFile(fileName: *const c_void) -> HRESULT;
    }
    /// 3D LUT object.
    ///
    /// This object provides additional information about LUTs and gives
    /// user the ability to control the lifetime of the resource.
    #[derive(Clone)]
    impl {
        scalar GetName          => fn name         (&Self) -> String; /// Get the LUT name
        scalar GetTitle         => fn title        (&Self) -> String; /// Get the LUT title
        scalar GetSize          => fn size         (&Self) -> u32; /// Get the LUT dimensions
        scalar GetResourceSizeBytes => fn resource_size_bytes(&Self) -> u32; /// Get resource size in bytes
        void   WriteCubeFile    => fn write_cube_file(&Self, file_name: String); /// Export LUT as .cube file
        // custom impl GetResourceGPU
        // custom impl GetResourceCPU
    }
}

braw_interface! {
    BlackmagicRawProcessedImage {
        /// Get the width of the processed image
        fn GetWidth(width: *mut u32) -> HRESULT;
        /// Get the height of the processed image
        fn GetHeight(height: *mut u32) -> HRESULT;
        /// Get pointer to resource the image is stored in
        fn GetResource(resource: *mut *mut c_void) -> HRESULT;
        /// Get type of resource
        fn GetResourceType(type_: *mut BlackmagicRawResourceType) -> HRESULT;
        /// Get format of resource
        fn GetResourceFormat(format: *mut BlackmagicRawResourceFormat) -> HRESULT;
        /// Get size of resource in bytes
        fn GetResourceSizeBytes(sizeBytes: *mut u32) -> HRESULT;
        /// Get context and command queue that the resource was created on
        fn GetResourceContextAndCommandQueue(context: *mut *mut c_void, commandQueue: *mut *mut c_void) -> HRESULT;
    }
    /// Processed image ready to display.
    ///
    /// This object is created by the API and provided via a ProcessComplete() callback.
    #[derive(Clone)]
    impl {
        scalar GetWidth             => fn width              (&Self) -> u32; /// Get image width in pixels
        scalar GetHeight            => fn height             (&Self) -> u32; /// Get image height in pixels
        // custom impl GetResource
        scalar GetResourceType      => fn resource_type      (&Self) -> BlackmagicRawResourceType; /// Get the resource type
        scalar GetResourceFormat    => fn resource_format    (&Self) -> BlackmagicRawResourceFormat; /// Get the pixel format
        scalar GetResourceSizeBytes => fn resource_size_bytes(&Self) -> u32; /// Get resource size in bytes
        scalar2 GetResourceContextAndCommandQueue => fn resource_context_and_command_queue(&Self) -> (*mut c_void, *mut c_void); /// Get context and command queue
    }
}

braw_interface! {
    BlackmagicRawJob {
        /// Submit the job to the decoder's internal queue
        fn Submit() -> HRESULT;
        /// Abort the job (can fail if already started)
        fn Abort() -> HRESULT;
        /// Attach some generic userdata to the job object
        fn SetUserData(userData: *mut c_void) -> HRESULT;
        /// Retrieve previously attached generic userdata from the job object
        fn GetUserData(userData: *mut *mut c_void) -> HRESULT;
    }
    /// Asynchronous job object.
    ///
    /// This is the base object that is returned when any job is created with the SDK.
    /// Use this to control and identify jobs when callbacks occur.
    #[derive(Clone)]
    impl {
        scalar GetUserData => fn user_data(&Self) -> *mut c_void; /// Get attached user data
        void SetUserData   => fn set_user_data(&mut Self, user_data: *mut c_void); /// Set user data
        void Submit        => fn submit     (&mut Self); /// Submit job to decoder queue
        void Abort         => fn abort      (&mut Self); /// Attempt to abort the job

        interface fn read_job_hints(&self) -> BlackmagicRawReadJobHints; /// Get read job hints interface
    }
}

pub enum ReadJobHints {
    None,
    Scale(BlackmagicRawResolutionScale)
}

braw_interface! {
    BlackmagicRawReadJobHints {
        /// Set the reader resolution scale
        fn SetReaderResolutionScale(readerResolutionScale: BlackmagicRawResolutionScale) -> HRESULT;
    }
    /// Read job hints.
    ///
    /// This object can be used to provide optimisation hints to the read job,
    /// such as specifying the scale at which to perform the read.
    #[derive(Clone)]
    impl {
        void SetReaderResolutionScale => fn set_reader_resolution_scale(&mut Self, reader_resolution_scale: BlackmagicRawResolutionScale); /// Set the scale for reading
    }
}

braw_interface! {
    /// Callback for IBlackmagicRaw.
    ///
    /// Central callback object for entire codec. Jobs submitted to any clip created
    /// by this codec will have their results provided through these function calls.
    BlackmagicRawCallback {
        /// Called when a read has completed
        fn ReadComplete(job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) -> ();
        /// Called when a decode has completed (manual decoders only)
        fn DecodeComplete(job: *mut IBlackmagicRawJob, result: HRESULT) -> ();
        /// Called when a process has completed
        fn ProcessComplete(job: *mut IBlackmagicRawJob, result: HRESULT, processedImage: *mut IBlackmagicRawProcessedImage) -> ();
        /// Called as a Trim job is processed to provide status updates
        fn TrimProgress(job: *mut IBlackmagicRawJob, progress: f32) -> ();
        /// Called when a trim has completed
        fn TrimComplete(job: *mut IBlackmagicRawJob, result: HRESULT) -> ();
        /// Called when a parse warning occurred when reading a .sidecar file (offending line will be ignored)
        fn SidecarMetadataParseWarning(clip: *mut IBlackmagicRawClip, fileName: *const c_void, lineNumber: u32, info: *const c_void) -> ();
        /// Called when a parse error occurred when reading a .sidecar file (entire file will be ignored)
        fn SidecarMetadataParseError(clip: *mut IBlackmagicRawClip, fileName: *const c_void, lineNumber: u32, info: *const c_void) -> ();
        /// Called when preparation of the pipeline has completed
        fn PreparePipelineComplete(userData: *mut c_void, result: HRESULT) -> ();
    }
}

braw_interface! {
    BlackmagicRawClipAudio {
        /// Get format the audio was recorded in
        fn GetAudioFormat(format: *mut BlackmagicRawAudioFormat) -> HRESULT;
        /// Get the audio bit depth
        fn GetAudioBitDepth(bitDepth: *mut u32) -> HRESULT;
        /// Get the audio channel count
        fn GetAudioChannelCount(channelCount: *mut u32) -> HRESULT;
        /// Get the audio sample rate
        fn GetAudioSampleRate(sampleRate: *mut u32) -> HRESULT;
        /// Get the audio sample count per channel
        fn GetAudioSampleCount(sampleCount: *mut u64) -> HRESULT;
        /// Get audio samples from the clip
        fn GetAudioSamples(sampleFrameIndex: i64, buffer: *mut c_void, bufferSizeBytes: u32, maxSampleCount: u32, samplesRead: *mut u32, bytesRead: *mut u32) -> HRESULT;
    }
    /// Audio component for an opened movie clip.
    ///
    /// Interface for accessing a clip's audio data.
    #[derive(Clone)]
    impl {
        scalar GetAudioFormat       => fn audio_format      (&Self) -> BlackmagicRawAudioFormat; /// Get audio format
        scalar GetAudioBitDepth     => fn audio_bit_depth   (&Self) -> u32; /// Get bit depth
        scalar GetAudioChannelCount => fn audio_channel_count(&Self) -> u32; /// Get channel count
        scalar GetAudioSampleRate   => fn audio_sample_rate (&Self) -> u32; /// Get sample rate
        scalar GetAudioSampleCount  => fn audio_sample_count(&Self) -> u64; /// Get total sample count
        // custom impl GetAudioSamples
    }
}

braw_interface! {
    BlackmagicRawClipAccelerometerMotion {
        /// Get the motion sample rate (samples per second)
        fn GetSampleRate(sampleRate: *mut f32) -> HRESULT;
        /// Get the motion sample count
        fn GetSampleCount(sampleCount: *mut u32) -> HRESULT;
        /// Get the size (in floats) of each motion sample
        fn GetSampleSize(sampleSize: *mut u32) -> HRESULT;
        /// Get motion samples for the specified range
        fn GetSampleRange(sampleStartIndex: u64, sampleCount: u32, samples: *mut f32, sampleCountOut: *mut u32) -> HRESULT;
    }
    /// Accelerometer motion for an opened movie clip.
    ///
    /// Interface for accessing a clip's accelerometer motion data.
    #[derive(Clone)]
    impl {
        scalar GetSampleRate  => fn sample_rate (&Self) -> f32; /// Get sample rate in Hz
        scalar GetSampleCount => fn sample_count(&Self) -> u32; /// Get total sample count
        scalar GetSampleSize  => fn sample_size (&Self) -> u32; /// Get sample size in floats
        // custom impl GetSampleRange
    }
}

braw_interface! {
    BlackmagicRawClipGyroscopeMotion {
        /// Get the motion sample rate (samples per second)
        fn GetSampleRate(sampleRate: *mut f32) -> HRESULT;
        /// Get the motion sample count
        fn GetSampleCount(sampleCount: *mut u32) -> HRESULT;
        /// Get the size (in floats) of each motion sample
        fn GetSampleSize(sampleSize: *mut u32) -> HRESULT;
        /// Get motion samples for the specified range (radians per second)
        fn GetSampleRange(sampleStartIndex: u64, sampleCount: u32, samples: *mut f32, sampleCountOut: *mut u32) -> HRESULT;
    }
    /// Gyroscope motion for an opened movie clip.
    ///
    /// Interface for accessing a clip's gyroscope motion data.
    #[derive(Clone)]
    impl {
        scalar GetSampleRate  => fn sample_rate (&Self) -> f32; /// Get sample rate in Hz
        scalar GetSampleCount => fn sample_count(&Self) -> u32; /// Get total sample count
        scalar GetSampleSize  => fn sample_size (&Self) -> u32; /// Get sample size in floats
        // custom impl GetSampleRange
    }
}

braw_interface! {
    BlackmagicRawClipPDAFData {
        /// Get the PDAF sample image width in pixels
        fn GetSampleImageWidthInPixels(sampleImageWidthInPixelsOut: *mut u32) -> HRESULT;
        /// Get the PDAF sample image height in pixels
        fn GetSampleImageHeightInPixels(sampleImageHeightInPixelsOut: *mut u32) -> HRESULT;
        /// Get the PDAF sample image bytes per pixel
        fn GetSampleImageBytesPerPixel(sampleImageBytesPerPixelOut: *mut u32) -> HRESULT;
        /// Get the PDAF sample size in bytes
        fn GetSampleSize(sampleSizeOut: *mut u32) -> HRESULT;
        /// Get the PDAF sample count
        fn GetSampleCount(sampleCountOut: *mut u32) -> HRESULT;
        /// Get the images for the specified PDAF sample
        fn GetSampleImages(sampleIndex: u64, leftSampleImageDataOut: *mut u8, rightSampleImageDataOut: *mut u8, sampleImageDataSize: u32) -> HRESULT;
    }
    /// Phase detection autofocus data for an opened movie clip.
    ///
    /// Interface for accessing a clip's phase detection autofocus data.
    #[derive(Clone)]
    impl {
        scalar GetSampleImageWidthInPixels  => fn sample_image_width_in_pixels (&Self) -> u32; /// Get image width
        scalar GetSampleImageHeightInPixels => fn sample_image_height_in_pixels(&Self) -> u32; /// Get image height
        scalar GetSampleImageBytesPerPixel  => fn sample_image_bytes_per_pixel (&Self) -> u32; /// Get bytes per pixel
        scalar GetSampleSize                => fn sample_size                  (&Self) -> u32; /// Get total sample size
        scalar GetSampleCount               => fn sample_count                 (&Self) -> u32; /// Get sample count
        // custom impl GetSampleImages
    }
}

braw_interface! {
    BlackmagicRawFrame {
        /// Get the frameIndex
        fn GetFrameIndex(frameIndex: *mut u64) -> HRESULT;
        /// Get a formatted timecode for this frame
        fn GetTimecode(timecode: *mut *mut c_void) -> HRESULT;
        /// Create a metadata iterator to iterate through the metadata in this frame
        fn GetMetadataIterator(iterator: *mut *mut IBlackmagicRawMetadataIterator) -> HRESULT;
        /// Query a single frame metadata value defined by key
        fn GetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        /// Set metadata to this frame. To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        fn SetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        /// Creates object with current frame processing attributes
        fn CloneFrameProcessingAttributes(frameProcessingAttributes: *mut *mut IBlackmagicRawFrameProcessingAttributes) -> HRESULT;
        /// Set the resolution scale we want to decode this image to
        fn SetResolutionScale(resolutionScale: BlackmagicRawResolutionScale) -> HRESULT;
        /// Get the resolution scale set to the frame
        fn GetResolutionScale(resolutionScale: *mut BlackmagicRawResolutionScale) -> HRESULT;
        /// Set the desired resource format that we want to process this frame into
        fn SetResourceFormat(resourceFormat: BlackmagicRawResourceFormat) -> HRESULT;
        /// Get the resource format this frame will be processed into
        fn GetResourceFormat(resourceFormat: *mut BlackmagicRawResourceFormat) -> HRESULT;
        /// Get the sensor rate with which this frame was recorded
        fn GetSensorRate(sensorRate: *mut f32) -> HRESULT;
        /// Create a job that will decode and process our image. When completed we will receive a ProcessComplete() callback
        fn CreateJobDecodeAndProcessFrame(clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    /// Frame of a clip.
    ///
    /// A frame that has been read but not yet processed. This is returned in the ReadComplete() callback.
    /// From here the user should prepare the frame for processing, and call DecodeAndProcessFrame().
    #[derive(Clone)]
    impl {
        struct CloneFrameProcessingAttributes => fn clone_frame_processing_attributes(&Self) -> BlackmagicRawFrameProcessingAttributes; /// Clone frame processing attributes
        scalar GetFrameIndex                  => fn frame_index(&Self) -> u64; /// Get the frame index
        scalar GetTimecode                    => fn timecode(&Self) -> String; /// Get formatted timecode
        scalar GetMetadata                    => fn metadata(&Self, key: String) -> VariantValue; /// Get metadata value by key
        scalar GetResolutionScale             => fn resolution_scale(&Self) -> BlackmagicRawResolutionScale; /// Get resolution scale
        scalar GetResourceFormat              => fn resource_format(&Self) -> BlackmagicRawResourceFormat; /// Get resource format
        scalar GetSensorRate                  => fn sensor_rate(&Self) -> f32; /// Get sensor rate in fps
        void SetMetadata                      => fn set_metadata(&Self, key: String, value: VariantValue); /// Set metadata value
        void SetResolutionScale               => fn set_resolution_scale(&Self, resolution_scale: BlackmagicRawResolutionScale); /// Set resolution scale
        void SetResourceFormat                => fn set_resource_format(&Self, resource_format: BlackmagicRawResourceFormat ); /// Set resource format
        // custom impl GetMetadataIterator
        // custom impl CreateJobDecodeAndProcessFrame
        interface fn processing_attributes(&self) -> BlackmagicRawFrameProcessingAttributes; /// Get frame processing attributes interface
    }
}

braw_interface! {
    BlackmagicRawFrameEx {
        /// Get the frames bitstream size in bytes we've read off disk
        fn GetBitStreamSizeBytes(bitStreamSizeBytes: *mut u32) -> HRESULT;
        /// Query what the resolution of the processed image will be given the input resolution and ResolutionScale applied
        fn GetProcessedImageResolution(width: *mut u32, height: *mut u32) -> HRESULT;
    }
    /// Provides extra information for use with IBlackmagicRawManualDecoder.
    ///
    /// Query additional information for the frame that is useful when decoding via manual decoders.
    #[derive(Clone)]
    impl {
        scalar GetBitStreamSizeBytes        => fn bit_stream_size_bytes(&Self) -> u32; /// Get bitstream size in bytes
        scalar2 GetProcessedImageResolution => fn processed_image_resolution(&Self) -> (u32, u32); /// Get processed image resolution (width, height)
    }
}

braw_interface! {
    BlackmagicRawFrameMultiVideo {
        /// Get the video track index of this frame
        fn GetVideoTrackIndex(videoTrackIndex: *mut u32) -> HRESULT;
    }
    /// Provides extra information related to frames from multi video track files.
    ///
    /// Query multi video track related information for the frame.
    #[derive(Clone)]
    impl {
        scalar GetVideoTrackIndex => fn video_track_index(&Self) -> u32; /// Get video track index
    }
}

braw_interface! {
    BlackmagicRawManualDecoderFlow1 {
        /// Converts the internal state of IBlackmagicRawFrame to frame state buffer, which is used to perform the decode
        fn PopulateFrameStateBuffer(frame: *mut IBlackmagicRawFrame, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes, frameState: *mut c_void, frameStateSizeBytes: u32) -> HRESULT;
        /// Query the size of the FrameState buffer in bytes
        fn GetFrameStateSizeBytes(frameStateSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the decoded buffer
        fn GetDecodedSizeBytes(frameStateBufferCPU: *mut c_void, decodedSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the processed buffer
        fn GetProcessedSizeBytes(frameStateBufferCPU: *mut c_void, processedSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the post 3D LUT buffer
        fn GetPost3DLUTSizeBytes(frameStateBufferCPU: *mut c_void, post3DLUTSizeBytes: *mut u32) -> HRESULT;
        /// Create a job to decode a frame. This decode completion will be notified via the OnDecodeComplete() callback
        fn CreateJobDecode(frameStateBufferCPU: *mut c_void, bitStreamBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Create a job to process a frame. After this process is complete a final processed image will be provided via OnProcessComplete() callback
        fn CreateJobProcess(frameStateBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, processedBufferCPU: *mut c_void, post3DLUTBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    /// Allowing manual decoding/processing of buffers, Flow1 is a pure CPU solution.
    ///
    /// Manual decoders give you more control over which buffers are used and how things are queued.
    /// Note: these decoders are optional and targeted at advanced users.
    #[derive(Clone)]
    impl {
        scalar GetFrameStateSizeBytes => fn frame_state_size_bytes(&Self) -> u32; /// Get frame state buffer size
        // TODO custom impl GetDecodedSizeBytes
        // TODO custom impl GetProcessedSizeBytes
        // TODO custom impl GetPost3DLUTSizeBytes
        // TODO custom impl CreateJobDecode
        // TODO custom impl CreateJobProcess
        // TODO custom impl PopulateFrameStateBuffer
   }
}

braw_interface! {
    BlackmagicRawManualDecoderFlow2 {
        /// Converts the internal state of IBlackmagicRawFrame to frame state buffer, which is used to perform the decode
        fn PopulateFrameStateBuffer(frame: *mut IBlackmagicRawFrame, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes, frameState: *mut c_void, frameStateSizeBytes: u32) -> HRESULT;
        /// Query the size of the FrameState buffer in bytes
        fn GetFrameStateSizeBytes(frameStateSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the decoded buffer
        fn GetDecodedSizeBytes(frameStateBufferCPU: *mut c_void, decodedSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the working buffer
        fn GetWorkingSizeBytes(frameStateBufferCPU: *mut c_void, workingSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the processed buffer
        fn GetProcessedSizeBytes(frameStateBufferCPU: *mut c_void, processedSizeBytes: *mut u32) -> HRESULT;
        /// Query the size of the post 3D LUT buffer
        fn GetPost3DLUTSizeBytes(frameStateBufferCPU: *mut c_void, post3DLUTSizeBytes: *mut u32) -> HRESULT;
        /// Create a job to decode a frame. This is performed on CPU. This decode completion will be notified via the OnDecodeComplete() callback
        fn CreateJobDecode(frameStateBufferCPU: *mut c_void, bitStreamBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Create a job to process a frame. This is performed on the specified GPU. After this process is complete a final processed image will be provided via a OnProcessComplete() callback
        fn CreateJobProcess(context: *mut c_void, commandQueue: *mut c_void, frameStateBufferCPU: *mut c_void, decodedBufferGPU: *mut c_void, workingBufferGPU: *mut c_void, processedBufferGPU: *mut c_void, post3DLUTBufferGPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    /// Allowing manual decoding/processing of buffers, Flow2 is a hybrid CPU/GPU solution.
    ///
    /// Manual decoders give you more control over which buffers are used and how things are queued.
    /// This will likely be faster than Flow1, however it will depend on the GPU in the users system.
    /// Note: these decoders are optional and targeted at advanced users.
    #[derive(Clone)]
    impl {
        scalar GetFrameStateSizeBytes => fn frame_state_size_bytes(&Self) -> u32; /// Get frame state buffer size
        // TODO custom impl GetDecodedSizeBytes
        // TODO custom impl GetProcessedSizeBytes
        // TODO custom impl GetPost3DLUTSizeBytes
        // TODO custom impl CreateJobDecode
        // TODO custom impl CreateJobProcess
        // TODO custom impl PopulateFrameStateBuffer
    }
}

braw_interface! {
    BlackmagicRawClip {
        /// Get the width of the clip
        fn GetWidth(out: *mut u32) -> HRESULT;
        /// Get the height of the clip
        fn GetHeight(out: *mut u32) -> HRESULT;
        /// Get the frame rate of the clip
        fn GetFrameRate(out: *mut f32) -> HRESULT;
        /// Get the frame count in the clip
        fn GetFrameCount(out: *mut u64) -> HRESULT;
        /// Get the timecode for the specified frame
        fn GetTimecodeForFrame(frameIndex: u64, timecode: *mut *mut c_void) -> HRESULT;
        /// Create a metadata iterator to iterate through the metadata in this clip
        fn GetMetadataIterator(iterator: *mut *mut IBlackmagicRawMetadataIterator) -> HRESULT;
        /// Query a single clip metadata value defined by key
        fn GetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        /// Set metadata to this clip. To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        fn SetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        /// Get the camera type on which this clip was recorded
        fn GetCameraType(cameraType: *mut *mut c_void) -> HRESULT;
        /// Creates object with current clip processing attributes
        fn CloneClipProcessingAttributes(clipProcessingAttributes: *mut *mut IBlackmagicRawClipProcessingAttributes) -> HRESULT;
        /// Queries how many cards this movie was originally recorded on to
        fn GetMulticardFileCount(multicardFileCount: *mut u32) -> HRESULT;
        /// Queries if a particular card file from the original recording is present
        fn IsMulticardFilePresent(index: u32, isMulticardFilePresent: *mut bool) -> HRESULT;
        /// Check for successfully parsed sidecar file, which is automatically loaded upon opening of a clip
        fn GetSidecarFileAttached(isSidecarFileAttached: *mut bool) -> HRESULT;
        /// Save metadata to sidecar file
        fn SaveSidecarFile() -> HRESULT;
        /// Reload metadata from sidecar file
        fn ReloadSidecarFile() -> HRESULT;
        /// Create a job to read a frame
        fn CreateJobReadFrame(frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Create a job to trim/export part of the clip with the sidecar file baked in
        fn CreateJobTrim(fileName: *const c_void, frameIndex: u64, frameCount: u64, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Clone the clip with different geometry settings
        fn CloneWithGeometry(geometry: *mut IBlackmagicRawClipGeometry, clip: *mut *mut IBlackmagicRawClip) -> HRESULT;
    }
    /// Opened movie clip.
    ///
    /// Clip object created by calling IBlackmagicRaw::OpenClip(). Represents an opened
    /// Blackmagic RAW file and provides access to its frames, metadata, and processing attributes.
    #[derive(Clone)]
    impl {
        struct CloneClipProcessingAttributes => fn clone_clip_processing_attributes(&Self) -> BlackmagicRawClipProcessingAttributes; /// Clone clip processing attributes
        struct CloneWithGeometry             => fn clone_with_geometry(&Self, geometry: BlackmagicRawClipGeometry) -> BlackmagicRawClip; /// Clone with new geometry
        scalar GetWidth                      => fn width(&Self) -> u32; /// Get clip width in pixels
        scalar GetHeight                     => fn height(&Self) -> u32; /// Get clip height in pixels
        scalar GetFrameRate                  => fn frame_rate(&Self) -> f32; /// Get frame rate in fps
        scalar GetFrameCount                 => fn frame_count(&Self) -> u64; /// Get total frame count
        scalar GetTimecodeForFrame           => fn timecode_for_frame(&Self, frame_index: u64) -> String; /// Get timecode for frame
        scalar GetMetadata                   => fn metadata(&Self, key: String) -> VariantValue; /// Get metadata value
        scalar GetCameraType                 => fn camera_type(&Self) -> String; /// Get camera type string
        scalar GetMulticardFileCount         => fn multicard_file_count(&Self) -> u32; /// Get multicard file count
        scalar IsMulticardFilePresent        => fn is_multicard_file_present(&Self, index: u32) -> bool; /// Check if multicard file exists
        scalar GetSidecarFileAttached        => fn sidecar_file_attached(&Self) -> bool; /// Check if sidecar is attached
        void SetMetadata                     => fn set_metadata(&Self, key: String, val: VariantValue); /// Set metadata value
        void SaveSidecarFile                 => fn save_sidecar_file(&Self); /// Save sidecar file to disk
        void ReloadSidecarFile               => fn reload_sidecar_file(&Self); /// Reload sidecar from disk
        // custom impl GetMetadataIterator
        // custom impl CreateJobReadFrame
        // custom impl CreateJobTrim
        interface fn ex(&self) -> BlackmagicRawClipEx; /// Get extended clip interface
        interface fn processing_attributes(&self) -> BlackmagicRawClipProcessingAttributes; /// Get clip processing attributes
        interface fn audio(&self) -> BlackmagicRawClipAudio; /// Get audio interface
        interface fn multi_video(&self) -> BlackmagicRawClipMultiVideo; /// Get multi-video interface
        interface fn accelerometer_motion(&self) -> BlackmagicRawClipAccelerometerMotion; /// Get accelerometer data
        interface fn gyroscope_motion(&self) -> BlackmagicRawClipGyroscopeMotion; /// Get gyroscope data
        interface fn pdaf_data(&self) -> BlackmagicRawClipPDAFData; /// Get PDAF data
    }
}

braw_interface! {
    BlackmagicRawClipEx {
        /// Inspects all frames in the movie and will return the maximum bit stream size encountered
        fn GetMaxBitStreamSizeBytes(maxBitStreamSizeBytes: *mut u32) -> HRESULT;
        /// Returns the bitstream size for the provided frame
        fn GetBitStreamSizeBytes(frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        /// Create a job to read the frame's bitstream into memory with custom buffer
        fn CreateJobReadFrame(frameIndex: u64, bitStream: *mut c_void, bitStreamSizeBytes: u32, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Queries the timecode info for the clip
        fn QueryTimecodeInfo(baseFrameIndex: *mut u32, isDropFrameTimecode: *mut bool) -> HRESULT;
    }
    /// Extended use of IBlackmagicRawClip, to pass custom bitstream.
    ///
    /// Provides advanced functionality for working with bitstreams and timecode information.
    #[derive(Clone)]
    impl {
        scalar GetMaxBitStreamSizeBytes => fn max_bit_stream_size_bytes(&Self) -> u32; /// Get maximum bitstream size
        scalar GetBitStreamSizeBytes    => fn bit_stream_size_bytes(&Self, frame_index: u64) -> u32; /// Get bitstream size for frame
        scalar2 QueryTimecodeInfo       => fn timecode_info(&Self) -> (u32, bool); /// Get timecode info (base frame, is drop frame)
        // custom impl CreateJobReadFrame
    }
}

braw_interface! {
    BlackmagicRawClipMultiVideo {
        /// Returns the number of video tracks in the clip
        fn GetVideoTrackCount(videoTrackCount: *mut u32) -> HRESULT;
        /// Returns the number of video frames in the specified track
        fn GetVideoFrameCount(videoTrackIndex: u32, videoFrameCount: *mut u64) -> HRESULT;
        /// Returns the fourCC video track source identifier
        fn GetVideoTrackSource(videoTrackIndex: u32, videoSourceFourCC: *mut u32) -> HRESULT;
        /// Create a job to read a frame
        fn CreateJobReadFrame(videoTrackIndex: u32, frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Returns the bitstream size for the provided frame
        fn GetBitStreamSizeBytes(videoTrackIndex: u32, frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        /// Create a job to read a frame with custom buffer
        fn CreateJobReadFrameEx(videoTrackIndex: u32, frameIndex: u64, bitStream : *mut c_void, bitStreamSizeBytes: u32, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    /// Extended use of IBlackmagicRawClip, to support multi video track video clips.
    ///
    /// Provides access to multiple video tracks within a single Blackmagic RAW file.
    #[derive(Clone)]
    impl {
        scalar GetVideoTrackCount  => fn video_track_count (&Self) -> u32; /// Get number of video tracks
        scalar GetVideoFrameCount  => fn video_frame_count (&Self, video_track_index: u32) -> u64; /// Get frame count for track
        scalar GetVideoTrackSource => fn video_track_source(&Self, video_track_index: u32) -> u32; /// Get track source fourCC
        // custom impl CreateJobReadFrame
        // custom impl CreateJobReadFrameEx
        scalar GetBitStreamSizeBytes => fn bit_stream_size_bytes(&Self, video_track_index: u32, frame_index: u64) -> u32; /// Get bitstream size
    }
}

braw_interface! {
    BlackmagicRawClipImmersiveVideo {
        /// Create a job to read a frame
        fn CreateJobImmersiveReadFrame(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Returns the bitstream size for the provided frame
        fn GetImmersiveBitStreamSizeBytes(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        /// Create a job to read a frame with custom buffer
        fn CreateJobImmersiveReadFrameEx(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, bitStream: *mut c_void, bitStreamSizeBytes: u32, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        /// Returns distance between lenses in micrometers
        fn GetDistanceBetweenLenses(distanceBetweenLenses: *mut u32) -> HRESULT;
        /// Returns comfort disparity adjustment in range [-10000 - +10000]
        fn GetComfortDisparityAdjustment(comfortDisparityAdjustment: *mut i32) -> HRESULT;
        /// Returns horizontal field of view in millidegrees
        fn GetHorizontalFieldOfView(horizontalFieldOfView: *mut u32) -> HRESULT;
        /// Get the immersive attribute
        fn GetImmersiveAttribute(attribute: BlackmagicRawImmersiveAttribute, value: *mut VARIANT) -> HRESULT;
    }
    /// Extended use of IBlackmagicRawClip, to support immersive video clips.
    ///
    /// Provides access to stereoscopic/immersive video features including left/right eye tracks.
    #[derive(Clone)]
    impl {
        scalar GetDistanceBetweenLenses       => fn distance_between_lenses       (&Self) -> u32; /// Get lens separation in micrometers
        scalar GetComfortDisparityAdjustment  => fn comfort_disparity_adjustment  (&Self) -> i32; /// Get disparity adjustment
        scalar GetHorizontalFieldOfView       => fn horizontal_field_of_view      (&Self) -> u32; /// Get FOV in millidegrees
        scalar GetImmersiveAttribute          => fn immersive_attribute           (&Self, attribute: BlackmagicRawImmersiveAttribute) -> VariantValue; /// Get immersive attribute
        // custom impl CreateJobImmersiveReadFrame
        // custom impl CreateJobImmersiveReadFrameEx
        scalar GetImmersiveBitStreamSizeBytes => fn immersive_bit_stream_size_bytes(&Self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64) -> u32; /// Get bitstream size
    }
}

braw_interface! {
    BlackmagicRawClipResolutions {
        /// Returns the number of resolutions at which the clip may be processed
        fn GetResolutionCount(resolutionCount: *mut u32) -> HRESULT;
        /// Returns a resolution at which the clip may be processed
        fn GetResolution(resolutionIndex: u32, resolutionWidthPixels: *mut u32, resolutionHeightPixels: *mut u32) -> HRESULT;
        /// Returns a resolution at which the clip was recorded
        fn GetRecordedResolution(resolutionIndex: u32, recordedResolutionWidthPixels: *mut u32, recordedResolutionHeightPixels: *mut u32) -> HRESULT;
        /// Returns a resolution which most closely matches the given scale
        fn GetClosestResolutionForScale(resolutionScale: BlackmagicRawResolutionScale, resolutionWidthPixels: *mut u32, resolutionHeightPixels: *mut u32) -> HRESULT;
        /// Returns a BlackmagicRawResolutionScale which most closely matches the given resolution
        fn GetClosestScaleForResolution(resolutionWidthPixels: u32, resolutionHeightPixels: u32, resolutionScale: *mut BlackmagicRawResolutionScale) -> HRESULT;
    }
    /// Supports querying of resolutions and/or scales for processed image results.
    ///
    /// Allows querying available resolutions for processing based on hardware capabilities.
    #[derive(Clone)]
    impl {
        scalar GetResolutionCount            => fn resolution_count            (&Self) -> u32; /// Get number of available resolutions
        scalar GetClosestScaleForResolution  => fn closest_scale_for_resolution(&Self, resolution_width_pixels: u32, resolution_height_pixels: u32) -> BlackmagicRawResolutionScale; /// Get closest scale for resolution
        scalar2 GetResolution                => fn resolution                  (&Self, resolution_index: u32) -> (u32, u32); /// Get resolution (width, height)
        scalar2 GetRecordedResolution        => fn recorded_resolution         (&Self, resolution_index: u32) -> (u32, u32); /// Get recorded resolution
        scalar2 GetClosestResolutionForScale => fn closest_resolution_for_scale(&Self, resolution_scale: BlackmagicRawResolutionScale) -> (u32, u32); /// Get closest resolution for scale
    }
}
