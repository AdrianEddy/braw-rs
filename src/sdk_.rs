pub use crate::*;
use core::ffi::c_void;

///! Based on Blackmagic RAW SDK 5.0.0

pub const IID_IBlackmagicRaw:                          GUID = GUID::new([/* 558ABA39-B344-4E9B-A484-116CF2A4B5C6 */ 0x55,0x8A,0xBA,0x39,0xB3,0x44,0x4E,0x9B,0xA4,0x84,0x11,0x6C,0xF2,0xA4,0xB5,0xC6 ]);
pub const IID_IBlackmagicRawFactory:                   GUID = GUID::new([/* 78DEEB84-98C9-434A-B7E5-7AACC2988399 */ 0x78,0xDE,0xEB,0x84,0x98,0xC9,0x43,0x4A,0xB7,0xE5,0x7A,0xAC,0xC2,0x98,0x83,0x99 ]);
pub const IID_IBlackmagicRawPipelineIterator:          GUID = GUID::new([/* 051ED792-3D9D-4ED0-BB1F-3873E08773CB */ 0x05,0x1E,0xD7,0x92,0x3D,0x9D,0x4E,0xD0,0xBB,0x1F,0x38,0x73,0xE0,0x87,0x73,0xCB ]);
pub const IID_IBlackmagicRawPipelineDeviceIterator:    GUID = GUID::new([/* 32D3385F-06EE-4260-82EB-2BABFFFACED8 */ 0x32,0xD3,0x38,0x5F,0x06,0xEE,0x42,0x60,0x82,0xEB,0x2B,0xAB,0xFF,0xFA,0xCE,0xD8 ]);
pub const IID_IBlackmagicRawOpenGLInteropHelper:       GUID = GUID::new([/* 86444C8A-4398-4364-9166-7D10F41C315E */ 0x86,0x44,0x4C,0x8A,0x43,0x98,0x43,0x64,0x91,0x66,0x7D,0x10,0xF4,0x1C,0x31,0x5E ]);
pub const IID_IBlackmagicRawPipelineDevice:            GUID = GUID::new([/* 5C7B0A9B-CF2C-4AB3-84C1-E1C7902360C8 */ 0x5C,0x7B,0x0A,0x9B,0xCF,0x2C,0x4A,0xB3,0x84,0xC1,0xE1,0xC7,0x90,0x23,0x60,0xC8 ]);
pub const IID_IBlackmagicRawToneCurve:                 GUID = GUID::new([/* 7E40C13D-3575-46B5-B2B7-85DAE1EEFD77 */ 0x7E,0x40,0xC1,0x3D,0x35,0x75,0x46,0xB5,0xB2,0xB7,0x85,0xDA,0xE1,0xEE,0xFD,0x77 ]);
pub const IID_IBlackmagicRawConfiguration:             GUID = GUID::new([/* 267E9866-FB40-4BFB-8BF8-96EA3F7DA36E */ 0x26,0x7E,0x98,0x66,0xFB,0x40,0x4B,0xFB,0x8B,0xF8,0x96,0xEA,0x3F,0x7D,0xA3,0x6E ]);
pub const IID_IBlackmagicRawConfigurationEx:           GUID = GUID::new([/* ACE9078F-ABA0-4B26-A954-EDA108DADA5A */ 0xAC,0xE9,0x07,0x8F,0xAB,0xA0,0x4B,0x26,0xA9,0x54,0xED,0xA1,0x08,0xDA,0xDA,0x5A ]);
pub const IID_IBlackmagicRawClipGeometry:              GUID = GUID::new([/* 22717196-36AE-4B8A-B5CC-24292F9660F0 */ 0x22,0x71,0x71,0x96,0x36,0xAE,0x4B,0x8A,0xB5,0xCC,0x24,0x29,0x2F,0x96,0x60,0xF0 ]);
pub const IID_IBlackmagicRawResourceManager:           GUID = GUID::new([/* 3C5C3C4A-812C-4AF0-99F0-06C6E197C189 */ 0x3C,0x5C,0x3C,0x4A,0x81,0x2C,0x4A,0xF0,0x99,0xF0,0x06,0xC6,0xE1,0x97,0xC1,0x89 ]);
pub const IID_IBlackmagicRawMetadataIterator:          GUID = GUID::new([/* F85AE78D-5DC2-40BC-8C1D-D0D805523ADA */ 0xF8,0x5A,0xE7,0x8D,0x5D,0xC2,0x40,0xBC,0x8C,0x1D,0xD0,0xD8,0x05,0x52,0x3A,0xDA ]);
pub const IID_IBlackmagicRawClipProcessingAttributes:  GUID = GUID::new([/* 1F53C8AE-2295-4C8E-B17F-5931F4F146AC */ 0x1F,0x53,0xC8,0xAE,0x22,0x95,0x4C,0x8E,0xB1,0x7F,0x59,0x31,0xF4,0xF1,0x46,0xAC ]);
pub const IID_IBlackmagicRawFrameProcessingAttributes: GUID = GUID::new([/* 5F7C5C0F-7138-445A-9D0D-6111B6409D17 */ 0x5F,0x7C,0x5C,0x0F,0x71,0x38,0x44,0x5A,0x9D,0x0D,0x61,0x11,0xB6,0x40,0x9D,0x17 ]);
pub const IID_IBlackmagicRawPost3DLUT:                 GUID = GUID::new([/* 86052BC4-0231-48C6-B3C8-C771112AAD68 */ 0x86,0x05,0x2B,0xC4,0x02,0x31,0x48,0xC6,0xB3,0xC8,0xC7,0x71,0x11,0x2A,0xAD,0x68 ]);
pub const IID_IBlackmagicRawProcessedImage:            GUID = GUID::new([/* D87A0F72-A883-42BB-8488-0089411C5035 */ 0xD8,0x7A,0x0F,0x72,0xA8,0x83,0x42,0xBB,0x84,0x88,0x00,0x89,0x41,0x1C,0x50,0x35 ]);
pub const IID_IBlackmagicRawJob:                       GUID = GUID::new([/* 34C05ACF-7118-45EA-8B71-887E0515395D */ 0x34,0xC0,0x5A,0xCF,0x71,0x18,0x45,0xEA,0x8B,0x71,0x88,0x7E,0x05,0x15,0x39,0x5D ]);
pub const IID_IBlackmagicRawReadJobHints:              GUID = GUID::new([/* 1069F99C-A4E2-415A-91C4-5E0CE0C6AF77 */ 0x10,0x69,0xF9,0x9C,0xA4,0xE2,0x41,0x5A,0x91,0xC4,0x5E,0x0C,0xE0,0xC6,0xAF,0x77 ]);
pub const IID_IBlackmagicRawCallback:                  GUID = GUID::new([/* E9F98FAC-33DB-4A65-BB94-8A82B027AED0 */ 0xE9,0xF9,0x8F,0xAC,0x33,0xDB,0x4A,0x65,0xBB,0x94,0x8A,0x82,0xB0,0x27,0xAE,0xD0 ]);
pub const IID_IBlackmagicRawClipAudio:                 GUID = GUID::new([/* 76D4ACED-E0D6-45BB-B547-56B7435B2A1D */ 0x76,0xD4,0xAC,0xED,0xE0,0xD6,0x45,0xBB,0xB5,0x47,0x56,0xB7,0x43,0x5B,0x2A,0x1D ]);
pub const IID_IBlackmagicRawClipAccelerometerMotion:   GUID = GUID::new([/* 983AACBB-F469-40C9-AA81-345B0B7CCD05 */ 0x98,0x3A,0xAC,0xBB,0xF4,0x69,0x40,0xC9,0xAA,0x81,0x34,0x5B,0x0B,0x7C,0xCD,0x05 ]);
pub const IID_IBlackmagicRawClipGyroscopeMotion:       GUID = GUID::new([/* 00543A2C-FDED-4C79-A60C-A460415F0296 */ 0x00,0x54,0x3A,0x2C,0xFD,0xED,0x4C,0x79,0xA6,0x0C,0xA4,0x60,0x41,0x5F,0x02,0x96 ]);
pub const IID_IBlackmagicRawClipPDAFData:              GUID = GUID::new([/* AAE71534-F951-4062-B6A9-69B8808CD267 */ 0xAA,0xE7,0x15,0x34,0xF9,0x51,0x40,0x62,0xB6,0xA9,0x69,0xB8,0x80,0x8C,0xD2,0x67 ]);
pub const IID_IBlackmagicRawFrame:                     GUID = GUID::new([/* A500B253-1808-4EF2-8692-D23C692404EA */ 0xA5,0x00,0xB2,0x53,0x18,0x08,0x4E,0xF2,0x86,0x92,0xD2,0x3C,0x69,0x24,0x04,0xEA ]);
pub const IID_IBlackmagicRawFrameEx:                   GUID = GUID::new([/* F8C6C374-D7FB-4BD3-AD0B-C533464FF450 */ 0xF8,0xC6,0xC3,0x74,0xD7,0xFB,0x4B,0xD3,0xAD,0x0B,0xC5,0x33,0x46,0x4F,0xF4,0x50 ]);
pub const IID_IBlackmagicRawFrameMultiVideo:           GUID = GUID::new([/* 96AE962B-30A4-4904-813D-A9EFFE5953C2 */ 0x96,0xAE,0x96,0x2B,0x30,0xA4,0x49,0x04,0x81,0x3D,0xA9,0xEF,0xFE,0x59,0x53,0xC2 ]);
pub const IID_IBlackmagicRawManualDecoderFlow1:        GUID = GUID::new([/* 278815A6-A3C1-47C7-A0A6-6754DEAE5E7A */ 0x27,0x88,0x15,0xA6,0xA3,0xC1,0x47,0xC7,0xA0,0xA6,0x67,0x54,0xDE,0xAE,0x5E,0x7A ]);
pub const IID_IBlackmagicRawManualDecoderFlow2:        GUID = GUID::new([/* DBEC4C39-B4C2-4A65-AA8C-2B3C7F4777E3 */ 0xDB,0xEC,0x4C,0x39,0xB4,0xC2,0x4A,0x65,0xAA,0x8C,0x2B,0x3C,0x7F,0x47,0x77,0xE3 ]);
pub const IID_IBlackmagicRawClip:                      GUID = GUID::new([/* A2910203-787B-4BF2-A374-B1A459E2D351 */ 0xA2,0x91,0x02,0x03,0x78,0x7B,0x4B,0xF2,0xA3,0x74,0xB1,0xA4,0x59,0xE2,0xD3,0x51 ]);
pub const IID_IBlackmagicRawClipEx:                    GUID = GUID::new([/* D260C7D0-93BD-4D68-B600-93B4CAB7F870 */ 0xD2,0x60,0xC7,0xD0,0x93,0xBD,0x4D,0x68,0xB6,0x00,0x93,0xB4,0xCA,0xB7,0xF8,0x70 ]);
pub const IID_IBlackmagicRawClipMultiVideo:            GUID = GUID::new([/* C699E3E2-3268-4E08-8037-D5C4A2C5CE52 */ 0xC6,0x99,0xE3,0xE2,0x32,0x68,0x4E,0x08,0x80,0x37,0xD5,0xC4,0xA2,0xC5,0xCE,0x52 ]);
pub const IID_IBlackmagicRawClipImmersiveVideo:        GUID = GUID::new([/* 47D287A7-9148-4659-BFCF-C767A089A200 */ 0x47,0xD2,0x87,0xA7,0x91,0x48,0x46,0x59,0xBF,0xCF,0xC7,0x67,0xA0,0x89,0xA2,0x00 ]);
pub const IID_IBlackmagicRawClipResolutions:           GUID = GUID::new([/* C63C290F-525B-4EBE-AB56-87B010CACE19 */ 0xC6,0x3C,0x29,0x0F,0x52,0x5B,0x4E,0xBE,0xAB,0x56,0x87,0xB0,0x10,0xCA,0xCE,0x19 ]);

// Enums

/// Variant types that may be stored as metadata
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
    U16       = VT_UI2       as u32,
    S32       = VT_I4        as u32,
    U32       = VT_UI4       as u32,
    Float32   = VT_R4        as u32,
    String    = VT_BSTR      as u32,
    SafeArray = VT_SAFEARRAY as u32,
    Float64   = VT_R8        as u32,
}

/// Used in IBlackmagicRawResourceManager
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]

pub enum BlackmagicRawResourceType {
    #[default] Null = 0,
    BufferCPU    = /* 'cpub' */ 0x63707562, // block of CPU memory
    BufferMetal  = /* 'metb' */ 0x6D657462, // a MTLBuffer
    BufferCUDA   = /* 'cudb' */ 0x63756462, // a CUdeviceptr
    BufferOpenCL = /* 'oclb' */ 0x6F636C62  // a cl_mem
}

/// Used for resource allocation
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResourceFormat {
    #[default] Null = 0,
    RGBAU8       = /* 'rgba' */ 0x72676261,
    BGRAU8       = /* 'bgra' */ 0x62677261,
    RGBU16       = /* '16il' */ 0x3136696C,
    RGBAU16      = /* '16al' */ 0x3136616C,
    BGRAU16      = /* '16la' */ 0x31366C61,
    RGBU16Planar = /* '16pl' */ 0x3136706C,
    RGBF32       = /* 'f32s' */ 0x66333273,
    RGBAF32      = /* 'f32l' */ 0x6633326C,
    BGRAF32      = /* 'f32a' */ 0x66333261,
    RGBF32Planar = /* 'f32p' */ 0x66333270,
    RGBF16       = /* 'f16s' */ 0x66313673,
    RGBAF16      = /* 'f16l' */ 0x6631366C,
    BGRAF16      = /* 'f16a' */ 0x66313661,
    RGBF16Planar = /* 'f16p' */ 0x66313670,
}

/// Used in IBlackmagicRawResourceManager
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]

pub enum BlackmagicRawResourceUsage {
    #[default] Null = 0,
    ReadCPUWriteCPU = /* 'rcwc' */ 0x72637763,
    ReadGPUWriteGPU = /* 'rgwg' */ 0x72677767,
    ReadGPUWriteCPU = /* 'rgwc' */ 0x72677763,
    ReadCPUWriteGPU = /* 'rcwg' */ 0x72637767
}

/// Used in IBlackmagicRawConfiguration. Each pipeline has different mappings to context/commandQueue
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawPipeline {
    #[default] Null = 0,
    CPU    = /* 'cpub' */ 0x63707562,
    CUDA   = /* 'cuda' */ 0x63756461,	// context/commandQueue maps to CUcontext/CUstream
    Metal  = /* 'metl' */ 0x6D65746C,	// context/commandQueue maps to nil/MTLCommandQueue
    OpenCL = /* 'opcl' */ 0x6F70636C	// context/commandQueue maps to cl_context/cl_command_queue
}

/// Used in IBlackmagicRawConfiguration

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawInstructionSet {
    #[default] Null = 0,
    SSE41 = /* 'se41' */ 0x73653431,
    AVX   = /* 'avx_' */ 0x6176785F,
    AVX2  = /* 'avx2' */ 0x61767832,
    NEON  = /* 'neon' */ 0x6E656F6E
}

/// Used in IBlackmagicRawFileAudio
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]

pub enum BlackmagicRawAudioFormat {
    #[default] Null = 0,
    PCMLittleEndian = /* 'pcml' */ 0x70636D6C
}

/// Used in IBlackmagicRawFrame

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawResolutionScale {
    #[default] Null = 0,
    Full    = /* 'full' */ 0x66756C6C,
    Half    = /* 'half' */ 0x68616C66,
    Quarter = /* 'qrtr' */ 0x71727472,
    Eighth  = /* 'eith' */ 0x65697468,
}

/// VARIANT types that may be stored as metadata
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]

pub enum BlackmagicRawClipProcessingAttribute {
    #[default] Null = 0,
    ColorScienceGen          = /* 'csgn' */ 0x6373676E,	// u16
    Gamma                    = /* 'gama' */ 0x67616D61,	// string
    Gamut                    = /* 'gamt' */ 0x67616D74,	// string
    ToneCurveContrast        = /* 'tcon' */ 0x74636F6E,	// float
    ToneCurveSaturation      = /* 'tsat' */ 0x74736174,	// float
    ToneCurveMidpoint        = /* 'tmid' */ 0x746D6964,	// float
    ToneCurveHighlights      = /* 'thih' */ 0x74686968,	// float
    ToneCurveShadows         = /* 'tsha' */ 0x74736861,	// float
    ToneCurveVideoBlackLevel = /* 'tvbl' */ 0x7476626C,	// u16
    ToneCurveBlackLevel      = /* 'tblk' */ 0x74626C6B,	// float
    ToneCurveWhiteLevel      = /* 'twit' */ 0x74776974,	// float
    HighlightRecovery        = /* 'hlry' */ 0x686C7279,	// u16
    AnalogGainIsConstant     = /* 'agic' */ 0x61676963,	// u16
    AnalogGain               = /* 'gain' */ 0x6761696E,	// float
    Post3DLUTMode            = /* 'lutm' */ 0x6C75746D,	// string
    EmbeddedPost3DLUTName    = /* 'emln' */ 0x656D6C6E,	// string
    EmbeddedPost3DLUTTitle   = /* 'emlt' */ 0x656D6C74,	// string
    EmbeddedPost3DLUTSize    = /* 'emls' */ 0x656D6C73,	// u16
    EmbeddedPost3DLUTData    = /* 'emld' */ 0x656D6C64,	// float array, size*size*size*3 elements
    SidecarPost3DLUTName     = /* 'scln' */ 0x73636C6E,	// string
    SidecarPost3DLUTTitle    = /* 'sclt' */ 0x73636C74,	// string
    SidecarPost3DLUTSize     = /* 'scls' */ 0x73636C73,	// u16
    SidecarPost3DLUTData     = /* 'scld' */ 0x73636C64,	// float array, size*size*size*3 elements
    GamutCompressionEnable   = /* 'gace' */ 0x67616365	// u16, 0=disabled, 1=enabled
}

/// VARIANT types that may be stored as metadata
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]

pub enum BlackmagicRawFrameProcessingAttribute {
    #[default] Null = 0,
    WhiteBalanceKelvin = /* 'wbkv' */ 0x77626B76,	// u32
    WhiteBalanceTint   = /* 'wbtn' */ 0x7762746E,	// s16
    Exposure           = /* 'expo' */ 0x6578706F,	// float
    ISO                = /* 'fiso' */ 0x6669736F,	// u32
    AnalogGain         = /* 'agpf' */ 0x61677066	// float
}

/// VARIANT types that may be stored as metadata
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawImmersiveAttribute {
    #[default] Null = 0,
    OpticalLensProcessingDataFileUUID = /* 'oldu' */ 0x6F6C6475,	// string
    OpticalILPDFileName               = /* 'oldf' */ 0x6F6C6466,	// string
    OpticalInteraxial                 = /* 'olix' */ 0x6F6C6978,	// i8
    OpticalProjectionKind             = /* 'olpk' */ 0x6F6C706B,	// string
    OpticalCalibrationType            = /* 'olct' */ 0x6F6C6374,	// string
    OpticalProjectionData             = /* 'olpd' */ 0x6F6C7064	// string
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawInterop {
    #[default] Null = 0,
    None   = /* 'none' */ 0x6E6F6E65,
    OpenGL = /* 'opgl' */ 0x6F70676C
}


#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawAnamorphicRatio {
    #[default] Null = 0,
    FromMetadata   = /* 'meta' */ 0x6D657461,
    Disabled       = /* 'dsbl' */ 0x6473626C,
    Anamorphic133x = /* '133x' */ 0x31333378,
    Anamorphic15x  = /* '15x_' */ 0x3135785F,
    Anamorphic16x  = /* '16x_' */ 0x3136785F,
    Anamorphic166x = /* '166x' */ 0x31363678,
    Anamorphic18x  = /* '18x_' */ 0x3138785F,
    Anamorphic2x   = /* '2x__' */ 0x32785F5F
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawRotation {
    #[default] Null = 0,
    FromMetadata = /* 'meta' */ 0x6D657461,
    Disabled     = /* 'dsbl' */ 0x6473626C,
    Clockwise90  = /* 'r90_' */ 0x7239305F,
    Clockwise180 = /* 'r180' */ 0x72313830,
    Clockwise270 = /* 'r270' */ 0x72323730
}


#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawFlip {
    #[default] Null = 0,
    FromMetadata = /* 'meta' */ 0x6D657461,
    Disabled     = /* 'dsbl' */ 0x6473626C,
    Horizontal   = /* 'flpx' */ 0x666C7078,
    Vertical     = /* 'flpy' */ 0x666C7079,
    Both         = /* 'flpb' */ 0x666C7062
}

/// Used in IBlackmagicRawConfigurationEx
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawSizeLimit {
    #[default] Null = 0,
    None  = /* 'szln' */ 0x737A6C6E,
    Scale = /* 'szsc' */ 0x737A7363,
    Crop  = /* 'szlc' */ 0x737A6C63
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BlackmagicRawImmersiveVideoTrack {
    #[default]
    Left  = 0,
    Right = 1
}

// Interfaces

braw_interface! {
    /// Each codec interface will have its own memory storage and decoder. When decoding multiple clips via one codec, first in first out ordering will apply
    BlackmagicRaw {
        fn OpenClip(fileName: *const c_void, out_clip: *mut *mut IBlackmagicRawClip) -> HRESULT; /// Opens a clip
        fn OpenClipWithGeometry(fileName: *const c_void, geometry: *mut IBlackmagicRawClipGeometry, out_clip: *mut *mut IBlackmagicRawClip) -> HRESULT; /// Opens a clip with specified geometry
        fn SetCallback(cb: *mut IBlackmagicRawCallback) -> HRESULT;
        fn PreparePipeline(pipeline: u32, pipelineContext: *mut c_void, pipelineCommandQueue: *mut c_void, userData: *mut c_void) -> HRESULT; // Asynchronously prepares the current pipeline
        fn PreparePipelineForDevice(device: *mut IBlackmagicRawPipelineDevice, userData: *mut c_void) -> HRESULT; // Asynchronously prepares the current pipeline
        fn FlushJobs() -> HRESULT;
    }
    field callback: CallbackHandle<DefaultCallback>,
    impl {
        void FlushJobs => fn flush_jobs(&mut Self);
        interface fn configuration(&self) -> BlackmagicRawConfiguration;
        interface fn configuration_ex(&self) -> BlackmagicRawConfigurationEx;
        interface fn manual_decoder_flow1(&self) -> BlackmagicRawManualDecoderFlow1;
        interface fn manual_decoder_flow2(&self) -> BlackmagicRawManualDecoderFlow2;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawFactory - Use this to create one or more Codec objects
    BlackmagicRawFactory {
        fn CreateCodec(out_codec: *mut *mut IBlackmagicRaw) -> HRESULT;
        fn CreatePipelineIterator(interop: BlackmagicRawInterop, out_it: *mut *mut IBlackmagicRawPipelineIterator) -> HRESULT;
        fn CreatePipelineDeviceIterator(pipeline: BlackmagicRawPipeline, interop: BlackmagicRawInterop, out_it: *mut *mut IBlackmagicRawPipelineDeviceIterator) -> HRESULT;
        fn CreateClipGeometry(out_geom: *mut *mut IBlackmagicRawClipGeometry) -> HRESULT;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawPipelineIterator - Use this to determine pipelines available for use on the system
    BlackmagicRawPipelineIterator {
        fn Next() -> HRESULT;	// When at last entry, calling Next() will return S_FALSE
        fn GetName(pipelineName: *mut *mut c_void) -> HRESULT; /// Get the name of the pipeline
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline) -> HRESULT;
    }
    impl {
        scalar GetName     => fn name(&Self) -> String; /// Get the name of the pipeline
        scalar GetInterop  => fn interop(&Self) -> BlackmagicRawInterop;
        scalar GetPipeline => fn pipeline(&Self) -> BlackmagicRawPipeline;
        void Next          => fn next(&mut Self);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawPipelineDeviceIterator - Use this to determine pipeline devices available for use on the system
    BlackmagicRawPipelineDeviceIterator {
        fn Next() -> HRESULT;	// When at last entry, calling Next() will return S_FALSE
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline) -> HRESULT;
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        fn CreateDevice(pipelineDevice: *mut *mut IBlackmagicRawPipelineDevice) -> HRESULT;
    }
    impl {
        struct CreateDevice => fn create_device(&Self) -> BlackmagicRawPipelineDevice;
        scalar GetInterop   => fn interop(&Self) -> BlackmagicRawInterop;
        scalar GetPipeline  => fn pipeline(&Self) -> BlackmagicRawPipeline;
        void Next           => fn next(&mut Self);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawOpenGLInteropHelper
    BlackmagicRawOpenGLInteropHelper {
        fn GetPreferredResourceFormat(preferredFormat: *mut BlackmagicRawResourceFormat) -> HRESULT;
        fn SetImage(processedImage: *mut IBlackmagicRawProcessedImage, openGLTextureName: *mut u32, openGLTextureTarget: *mut i32) -> HRESULT;
    }
    impl {
        scalar GetPreferredResourceFormat => fn preferred_resource_format(&Self) -> BlackmagicRawResourceFormat;
        scalar2 SetImage                  => fn set_image(&mut Self, processed_image: BlackmagicRawProcessedImage) -> (u32, i32);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawPipelineDevice
    BlackmagicRawPipelineDevice {
        fn SetBestInstructionSet() -> HRESULT;
        fn SetInstructionSet(instructionSet: BlackmagicRawInstructionSet) -> HRESULT;
        fn GetInstructionSet(instructionSet: *mut BlackmagicRawInstructionSet) -> HRESULT;
        fn GetIndex(deviceIndex: *mut u32) -> HRESULT;
        fn GetName(deviceName: *mut *mut c_void) -> HRESULT;
        fn GetInterop(interop: *mut BlackmagicRawInterop) -> HRESULT;
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline, context: *mut *mut c_void, commandQueue: *mut *mut c_void) -> HRESULT;
        fn GetPipelineName(pipelineName: *mut *mut c_void) -> HRESULT;
        fn GetOpenGLInteropHelper(/* out */ interopHelper: *mut *mut IBlackmagicRawOpenGLInteropHelper) -> HRESULT;
        fn GetSupportedResourceFormats(/* out */ array: *mut BlackmagicRawResourceFormat, /* in, out */ arrayElementCount: *mut u32) -> HRESULT;
        fn GetMaximumTextureSize(maximumWidth: *mut u32, maximumHeight: *mut u32) -> HRESULT;
    }
    impl {
        struct GetOpenGLInteropHelper=> fn opengl_interop_helper(&Self) -> BlackmagicRawOpenGLInteropHelper;
        scalar GetInstructionSet     => fn instruction_set(&Self) -> BlackmagicRawInstructionSet;
        scalar GetIndex              => fn index(&Self) -> u32;
        scalar GetName               => fn name(&Self) -> String;
        scalar GetInterop            => fn interop(&Self) -> BlackmagicRawInterop;
        scalar GetPipelineName       => fn pipeline_name(&Self) -> String;
        scalar2 GetMaximumTextureSize => fn maximum_texture_size(&Self) -> (u32, u32);
        scalar3 GetPipeline          => fn pipeline(&Self) -> (BlackmagicRawPipeline, *mut c_void, *mut c_void);
        void   SetInstructionSet     => fn set_instruction_set(&mut Self, instruction_set: BlackmagicRawInstructionSet);
        void   SetBestInstructionSet => fn set_best_instruction_set(&mut Self);
        // custom impl GetSupportedResourceFormats
    }
}

braw_interface! {
    /// Interface IBlackmagicRawToneCurve - Functions which provide useful tone curve operations
    BlackmagicRawToneCurve {
        fn GetToneCurve(cameraType: *const c_void, gamma: *const c_void, gen_: u16 /* Color science gen */, contrast: *mut f32, saturation: *mut f32, midpoint: *mut f32, highlights: *mut f32, shadows: *mut f32, blackLevel: *mut f32, whiteLevel: *mut f32, videoBlackLevel: *mut u16) -> HRESULT;	// Query tone curve parameters for a specific camera and gamma. These are only currently available on Gamut: Blackmagic Design, Gamma: Blackmagic Design Film, Blackmagic Design Extended Video, Blackmagic Design Custom. Note: Custom gamma can define a tone curve per clip, see BlackmagicRawClipProcessingAttributes::GetToneCurveForCustomGamma()
        fn EvaluateToneCurve(cameraType: *const c_void, gen_: u16, contrast: f32, saturation: f32, midpoint: f32, highlights: f32, shadows: f32, blackLevel: f32, whiteLevel: f32, videoBlackLevel: u16, array: *mut f32, arrayElementCount: u32) -> HRESULT;	// Evaluates tone curve, returned buffer can be used to visualise curve
    }
    impl {
        // custom impl GetToneCurve
        // custom impl EvaluateToneCurve
    }
}

braw_interface! {
    /// Interface IBlackmagicRawConfiguration - Configuration for Codec Object. Configuration properties are read on first OpenClip()
    BlackmagicRawConfiguration {
        fn SetPipeline(pipeline: BlackmagicRawPipeline /* changing pipeline will cause the default resource manager to be re-created */, pipelineContext: *mut c_void, pipelineCommandQueue: *mut c_void) -> HRESULT;
        fn GetPipeline(pipeline: *mut BlackmagicRawPipeline /* optional */, pipelineContextOut: *mut *mut c_void /* optional */, pipelineCommandQueueOut: *mut *mut c_void /* optional */) -> HRESULT;
        fn IsPipelineSupported(pipeline: BlackmagicRawPipeline, pipelineSupported: *mut bool) -> HRESULT;	// Verifies relevant hardware/DLLs are available
        fn SetCPUThreads(threadCount: u32 /* set 0 for default */) -> HRESULT;
        fn GetCPUThreads(threadCount: *mut u32) -> HRESULT;
        fn GetMaxCPUThreadCount(threadCount: *mut u32) -> HRESULT;
        fn SetWriteMetadataPerFrame(/* in */ writePerFrame: bool) -> HRESULT;	// if true, frame metadata will be written to only the relevant frame
        fn GetWriteMetadataPerFrame(/* out */ writePerFrame: *mut bool) -> HRESULT;
        fn SetFromDevice(/* in */ pipelineDevice: *mut IBlackmagicRawPipelineDevice) -> HRESULT;
        fn GetVersion(version: *mut *mut c_void) -> HRESULT;
        fn GetCameraSupportVersion(version: *mut *mut c_void) -> HRESULT;
    }
    impl {
        scalar GetVersion              => fn version(&Self) -> String;
        scalar GetCameraSupportVersion => fn camera_support_version(&Self) -> String;
        scalar GetCPUThreads           => fn cpu_threads(&Self) -> u32;
        scalar GetMaxCPUThreadCount    => fn max_cpu_thread_count(&Self) -> u32;
        scalar GetWriteMetadataPerFrame=> fn write_metadata_per_frame(&Self) -> bool;
        scalar IsPipelineSupported     => fn is_pipeline_supported(&Self, pipeline: BlackmagicRawPipeline) -> bool;
        scalar3 GetPipeline            => fn pipeline(&Self) -> (BlackmagicRawPipeline, *mut c_void, *mut c_void);
        void   SetPipeline             => fn set_pipeline(&mut Self, pipeline: BlackmagicRawPipeline, pipeline_context: *mut c_void, pipeline_command_queue: *mut c_void);
        void   SetCPUThreads           => fn set_cpu_threads(&mut Self, thread_count: u32);
        void   SetWriteMetadataPerFrame=> fn set_write_metadata_per_frame(&mut Self, write_per_frame: bool);
        void   SetFromDevice           => fn set_from_device(&mut Self, pipeline_device: BlackmagicRawPipelineDevice);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawConfigurationEx - Extended Configuration for Codec Object
    BlackmagicRawConfigurationEx {
        fn GetResourceManager(/* out */ resourceManager: *mut *mut IBlackmagicRawResourceManager) -> HRESULT;
        fn SetResourceManager(/* in */ resourceManager: *mut IBlackmagicRawResourceManager /* setting null will restore the default resource manager */) -> HRESULT;	// Set to NULL to return to default resource manager
        fn GetInstructionSet(/* out */ instructionSet: *mut BlackmagicRawInstructionSet) -> HRESULT;
        fn SetInstructionSet(/* in */ instructionSet: BlackmagicRawInstructionSet) -> HRESULT;
        fn GetSizeLimit(/* out */ sizeLimit: *mut BlackmagicRawSizeLimit, sizeLimitWidth: *mut u32, sizeLimitHeight: *mut u32) -> HRESULT;
        fn SetSizeLimit(/* in */ sizeLimit: BlackmagicRawSizeLimit, sizeLimitWidth: u32, sizeLimitHeight: u32) -> HRESULT;
    }
    impl {
        struct GetResourceManager => fn resource_manager(&Self) -> BlackmagicRawResourceManager;
        scalar GetInstructionSet  => fn instruction_set(&Self) -> BlackmagicRawInstructionSet;
        scalar3 GetSizeLimit      => fn size_limit(&Self) -> (BlackmagicRawSizeLimit, u32, u32);
        void   SetResourceManager => fn set_resource_manager(&mut Self, resource_manager: BlackmagicRawResourceManager);
        void   SetInstructionSet  => fn set_instruction_set(&mut Self, instruction_set: BlackmagicRawInstructionSet);
        void   SetSizeLimit       => fn set_size_limit(&mut Self, size_limit: BlackmagicRawSizeLimit, size_limit_width: u32, size_limit_height: u32);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipGeometry - Geometry properties of Clip object
    BlackmagicRawClipGeometry {
        fn GetAnamorphicRatio(anamorphic: *mut BlackmagicRawAnamorphicRatio) -> HRESULT;
        fn SetAnamorphicRatio(anamorphic: BlackmagicRawAnamorphicRatio) -> HRESULT;
        fn GetRotation(rotation: *mut BlackmagicRawRotation) -> HRESULT;
        fn SetRotation(rotation: BlackmagicRawRotation) -> HRESULT;
        fn GetFlip(flip: *mut BlackmagicRawFlip) -> HRESULT;
        fn SetFlip(flip: BlackmagicRawFlip) -> HRESULT;
    }
    impl {
        scalar GetAnamorphicRatio => fn anamorphic_ratio(&Self) -> BlackmagicRawAnamorphicRatio;
        scalar GetRotation        => fn rotation(&Self) -> BlackmagicRawRotation;
        scalar GetFlip            => fn flip(&Self) -> BlackmagicRawFlip;
        void   SetAnamorphicRatio => fn set_anamorphic_ratio(&mut Self, anamorphic: BlackmagicRawAnamorphicRatio);
        void   SetRotation        => fn set_rotation(&mut Self, rotation: BlackmagicRawRotation);
        void   SetFlip            => fn set_flip(&mut Self, flip: BlackmagicRawFlip);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawResourceManager - Resource manager allows manual handling of CPU/GPU resource allocation
    BlackmagicRawResourceManager {
        fn CreateResource(context: *mut c_void, commandQueue: *mut c_void, sizeBytes: u32, typ: BlackmagicRawResourceType, usage: BlackmagicRawResourceUsage, resource: *mut *mut c_void) -> HRESULT;
        fn ReleaseResource(context: *mut c_void, commandQueue: *mut c_void, resource: *mut c_void, typ: BlackmagicRawResourceType) -> HRESULT;
        fn CopyResource(context: *mut c_void, commandQueue: *mut c_void, source: *mut c_void, sourceType: BlackmagicRawResourceType, destination: *mut c_void, destinationType: BlackmagicRawResourceType, sizeBytes: u32, copyAsync: bool) -> HRESULT;	// Utility function, not called by the API
        fn GetResourceHostPointer(context: *mut c_void, commandQueue: *mut c_void, resource: *mut c_void, resourceType: BlackmagicRawResourceType, /* out */ hostPointer: *mut *mut c_void) -> HRESULT;	// Utility function, not called by the API
    }
    impl {
        scalar CreateResource      => fn create_resource(&Self, context: *mut c_void, command_queue: *mut c_void, size_bytes: u32, typ: BlackmagicRawResourceType, usage: BlackmagicRawResourceUsage) -> *mut c_void;
        scalar GetResourceHostPointer => fn resource_host_pointer(&Self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, resource_type: BlackmagicRawResourceType) -> *mut c_void;
        void   CopyResource        => fn copy_resource(&mut Self, context: *mut c_void, command_queue: *mut c_void, source: *mut c_void, source_type: BlackmagicRawResourceType, destination: *mut c_void, destination_type: BlackmagicRawResourceType, size_bytes: u32, copy_async: bool);
        void   ReleaseResource     => fn release_resource(&mut Self, context: *mut c_void, command_queue: *mut c_void, resource: *mut c_void, typ: BlackmagicRawResourceType);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawMetadataIterator - Iterating metadata
    BlackmagicRawMetadataIterator {
        fn Next() -> HRESULT;	// When at last entry, calling Next() will return S_FALSE
        fn GetKey(key: *mut *mut c_void) -> HRESULT;
        fn GetData(data: *mut VARIANT) -> HRESULT;
    }
    impl {
        scalar GetKey  => fn key(&Self) -> String;
        scalar GetData => fn data(&Self) -> VariantValue;
        void  Next     => fn next(&mut Self);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipProcessingAttributes - Clip attributes used during processing
    BlackmagicRawClipProcessingAttributes {
        fn GetClipAttribute(attribute: BlackmagicRawClipProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        fn SetClipAttribute(attribute: BlackmagicRawClipProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        fn GetClipAttributeRange(attribute: BlackmagicRawClipProcessingAttribute, valueMin: *mut VARIANT, valueMax: *mut VARIANT, isReadOnly: *mut bool) -> HRESULT;
        fn GetClipAttributeList(attribute: BlackmagicRawClipProcessingAttribute, array: *mut VARIANT /* optional */, arrayElementCount: *mut u32 /* optional */, isReadOnly: *mut bool) -> HRESULT;
        fn GetISOList(array: *mut u32 /* optional */, /* in, out */ arrayElementCount: *mut u32/* optional */, /* out */ isReadOnly: *mut bool) -> HRESULT;
        fn GetPost3DLUT(lut: *mut *mut IBlackmagicRawPost3DLUT) -> HRESULT;
    }
    #[derive(Clone)]
    impl {
        struct GetPost3DLUT     => fn post_3d_lut(&Self) -> BlackmagicRawPost3DLUT;
        scalar GetClipAttribute => fn attribute(&Self, attribute: BlackmagicRawClipProcessingAttribute) -> VariantValue;
        void   SetClipAttribute => fn set_attribute(&Self, attribute: BlackmagicRawClipProcessingAttribute, value: VariantValue);
        // custom impl GetClipAttributeRange
        // custom impl GetClipAttributeList
        // custom impl GetISOList
    }
}

braw_interface! {
    /// Interface IBlackmagicRawFrameProcessingAttributes - Frame attributes used during processing
    BlackmagicRawFrameProcessingAttributes {
        fn GetFrameAttribute(attribute: BlackmagicRawFrameProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        fn SetFrameAttribute(attribute: BlackmagicRawFrameProcessingAttribute, value: *mut VARIANT) -> HRESULT;
        fn GetFrameAttributeRange(attribute: BlackmagicRawFrameProcessingAttribute, valueMin: *mut VARIANT, valueMax: *mut VARIANT, isReadOnly: *mut bool) -> HRESULT;
        fn GetFrameAttributeList(attribute: BlackmagicRawFrameProcessingAttribute, array: *mut VARIANT /* optional */, arrayElementCount: *mut u32 /* optional */, isReadOnly: *mut bool) -> HRESULT;
        fn GetISOList(array: *mut u32 /* optional */, /* in, out */arrayElementCount: *mut u32/* optional */, isReadOnly: *mut bool) -> HRESULT;
    }
    #[derive(Clone)]
    impl {
        scalar GetFrameAttribute => fn attribute(&Self, attribute: BlackmagicRawFrameProcessingAttribute) -> VariantValue;
        void   SetFrameAttribute => fn set_attribute(&Self, attribute: BlackmagicRawFrameProcessingAttribute, value: VariantValue);
        // custom impl GetFrameAttributeRange
        // custom impl GetFrameAttributeList
        // custom impl GetISOList
    }
}

braw_interface! {
    /// Interface IBlackmagicRawPost3DLUT - 3D LUT object
    BlackmagicRawPost3DLUT {
        fn GetName(name: *mut *mut c_void) -> HRESULT;
        fn GetTitle(title: *mut *mut c_void) -> HRESULT;
        fn GetSize(size: *mut u32) -> HRESULT;
        fn GetResourceGPU(context: *mut c_void, commandQueue: *mut c_void, typ_: *mut BlackmagicRawResourceType, resource: *mut *mut c_void) -> HRESULT;
        fn GetResourceCPU(resource: *mut *mut c_void) -> HRESULT;
        fn GetResourceSizeBytes(sizeBytes: *mut u32) -> HRESULT;
        fn WriteCubeFile(fileName: *const c_void) -> HRESULT;
    }
    impl {
        scalar GetName          => fn name         (&Self) -> String;
        scalar GetTitle         => fn title        (&Self) -> String;
        scalar GetSize          => fn size         (&Self) -> u32;
        scalar GetResourceSizeBytes => fn resource_size_bytes(&Self) -> u32;
        void   WriteCubeFile    => fn write_cube_file(&Self, file_name: String);
        // custom impl GetResourceGPU
        // custom impl GetResourceCPU
    }
}

braw_interface! {
    /// Interface IBlackmagicRawProcessedImage - Processed image ready to display
    BlackmagicRawProcessedImage {
        fn GetWidth(width: *mut u32) -> HRESULT;
        fn GetHeight(height: *mut u32) -> HRESULT;
        fn GetResource(resource: *mut *mut c_void) -> HRESULT;
        fn GetResourceType(type_: *mut BlackmagicRawResourceType) -> HRESULT;
        fn GetResourceFormat(format: *mut BlackmagicRawResourceFormat) -> HRESULT;
        fn GetResourceSizeBytes(sizeBytes: *mut u32) -> HRESULT;
        fn GetResourceContextAndCommandQueue(context: *mut *mut c_void, commandQueue: *mut *mut c_void) -> HRESULT;
    }
    impl {
        scalar GetWidth             => fn width              (&Self) -> u32;
        scalar GetHeight            => fn height             (&Self) -> u32;
        // custom impl GetResource
        scalar GetResourceType      => fn resource_type      (&Self) -> BlackmagicRawResourceType;
        scalar GetResourceFormat    => fn resource_format    (&Self) -> BlackmagicRawResourceFormat;
        scalar GetResourceSizeBytes => fn resource_size_bytes(&Self) -> u32;
        scalar2 GetResourceContextAndCommandQueue => fn resource_context_and_command_queue(&Self) -> (*mut c_void, *mut c_void);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawJob - Asynchronous job object
    BlackmagicRawJob {
        fn Submit() -> HRESULT;
        fn Abort() -> HRESULT;
        fn SetUserData(userData: *mut c_void) -> HRESULT;
        fn GetUserData(userData: *mut *mut c_void) -> HRESULT;
    }
    impl {
        scalar GetUserData => fn user_data(&Self) -> *mut c_void;
        void SetUserData   => fn set_user_data(&mut Self, user_data: *mut c_void);
        void Submit        => fn submit     (&mut Self);
        void Abort         => fn abort      (&mut Self);

        interface fn read_job_hints(&self) -> BlackmagicRawReadJobHints;
    }
}

pub enum ReadJobHints {
    None,
    Scale(BlackmagicRawResolutionScale)
}

braw_interface! {
    /// Interface IBlackmagicRawReadJobHints - Read job hints
    BlackmagicRawReadJobHints {
        fn SetReaderResolutionScale(readerResolutionScale: BlackmagicRawResolutionScale) -> HRESULT;
    }
    impl {
        void SetReaderResolutionScale => fn set_reader_resolution_scale(&mut Self, reader_resolution_scale: BlackmagicRawResolutionScale);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawCallback - Callback for IBlackmagicRaw
    BlackmagicRawCallback {
        fn ReadComplete(job: *mut IBlackmagicRawJob, result: HRESULT, frame: *mut IBlackmagicRawFrame) -> ();
        fn DecodeComplete(job: *mut IBlackmagicRawJob, result: HRESULT) -> ();
        fn ProcessComplete(job: *mut IBlackmagicRawJob, result: HRESULT, processedImage: *mut IBlackmagicRawProcessedImage) -> ();
        fn TrimProgress(job: *mut IBlackmagicRawJob, progress: f32) -> ();
        fn TrimComplete(job: *mut IBlackmagicRawJob, result: HRESULT) -> ();
        fn SidecarMetadataParseWarning(clip: *mut IBlackmagicRawClip, fileName: *const c_void, lineNumber: u32, info: *const c_void) -> ();	// offending line will be ignored
        fn SidecarMetadataParseError(clip: *mut IBlackmagicRawClip, fileName: *const c_void, lineNumber: u32, info: *const c_void) -> ();	// entire file will be ignored
        fn PreparePipelineComplete(userData: *mut c_void, result: HRESULT) -> ();
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipAudio - Audio component for an opened movie clip
    BlackmagicRawClipAudio {
        fn GetAudioFormat(format: *mut BlackmagicRawAudioFormat) -> HRESULT;
        fn GetAudioBitDepth(bitDepth: *mut u32) -> HRESULT;
        fn GetAudioChannelCount(channelCount: *mut u32) -> HRESULT;
        fn GetAudioSampleRate(sampleRate: *mut u32) -> HRESULT;
        fn GetAudioSampleCount(sampleCount: *mut u64) -> HRESULT;
        fn GetAudioSamples(sampleFrameIndex: i64, buffer: *mut c_void, bufferSizeBytes: u32, maxSampleCount: u32, samplesRead: *mut u32 /* optional */, bytesRead: *mut u32 /* optional */) -> HRESULT;
    }
    impl {
        scalar GetAudioFormat       => fn audio_format      (&Self) -> BlackmagicRawAudioFormat;
        scalar GetAudioBitDepth     => fn audio_bit_depth   (&Self) -> u32;
        scalar GetAudioChannelCount => fn audio_channel_count(&Self) -> u32;
        scalar GetAudioSampleRate   => fn audio_sample_rate (&Self) -> u32;
        scalar GetAudioSampleCount  => fn audio_sample_count(&Self) -> u64;
        // custom impl GetAudioSamples
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipAccelerometerMotion - Accelerometer motion for an opened movie clip
    BlackmagicRawClipAccelerometerMotion {
        fn GetSampleRate(sampleRate: *mut f32) -> HRESULT;
        fn GetSampleCount(sampleCount: *mut u32) -> HRESULT;
        fn GetSampleSize(sampleSize: *mut u32) -> HRESULT;
        fn GetSampleRange(sampleStartIndex: u64, sampleCount: u32, samples: *mut f32, sampleCountOut: *mut u32) -> HRESULT;
    }
    impl {
        scalar GetSampleRate  => fn sample_rate (&Self) -> f32;
        scalar GetSampleCount => fn sample_count(&Self) -> u32;
        scalar GetSampleSize  => fn sample_size (&Self) -> u32;
        // custom impl GetSampleRange
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipGyroscopeMotion - Gyroscope motion for an opened movie clip
    BlackmagicRawClipGyroscopeMotion {
        fn GetSampleRate(sampleRate: *mut f32) -> HRESULT;
        fn GetSampleCount(sampleCount: *mut u32) -> HRESULT;
        fn GetSampleSize(sampleSize: *mut u32) -> HRESULT;
        fn GetSampleRange(sampleStartIndex: u64, sampleCount: u32, samples: *mut f32, sampleCountOut: *mut u32) -> HRESULT;
    }
    impl {
        scalar GetSampleRate  => fn sample_rate (&Self) -> f32;
        scalar GetSampleCount => fn sample_count(&Self) -> u32;
        scalar GetSampleSize  => fn sample_size (&Self) -> u32;
        // custom impl GetSampleRange
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipPDAFData - Phase detection autofocus data for an opened movie clip
    BlackmagicRawClipPDAFData {
        fn GetSampleImageWidthInPixels(sampleImageWidthInPixelsOut: *mut u32) -> HRESULT;
        fn GetSampleImageHeightInPixels(sampleImageHeightInPixelsOut: *mut u32) -> HRESULT;
        fn GetSampleImageBytesPerPixel(sampleImageBytesPerPixelOut: *mut u32) -> HRESULT;
        fn GetSampleSize(sampleSizeOut: *mut u32) -> HRESULT;
        fn GetSampleCount(sampleCountOut: *mut u32) -> HRESULT;
        fn GetSampleImages(sampleIndex: u64, leftSampleImageDataOut: *mut u8, rightSampleImageDataOut: *mut u8, sampleImageDataSize: u32) -> HRESULT;
    }
    impl {
        scalar GetSampleImageWidthInPixels  => fn sample_image_width_in_pixels (&Self) -> u32;
        scalar GetSampleImageHeightInPixels => fn sample_image_height_in_pixels(&Self) -> u32;
        scalar GetSampleImageBytesPerPixel  => fn sample_image_bytes_per_pixel (&Self) -> u32;
        scalar GetSampleSize                => fn sample_size                  (&Self) -> u32;
        scalar GetSampleCount               => fn sample_count                 (&Self) -> u32;
        // custom impl GetSampleImages
    }
}

braw_interface! {
    /// Interface IBlackmagicRawFrame - Frame of a clip
    BlackmagicRawFrame {
        fn GetFrameIndex(frameIndex: *mut u64) -> HRESULT;
        fn GetTimecode(timecode: *mut *mut c_void) -> HRESULT;
        fn GetMetadataIterator(iterator: *mut *mut IBlackmagicRawMetadataIterator) -> HRESULT;
        fn GetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        fn SetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;	// To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        fn CloneFrameProcessingAttributes(frameProcessingAttributes: *mut *mut IBlackmagicRawFrameProcessingAttributes) -> HRESULT;	// creates object with current frame processing attributes
        fn SetResolutionScale(resolutionScale: BlackmagicRawResolutionScale ) -> HRESULT;
        fn GetResolutionScale(resolutionScale: *mut BlackmagicRawResolutionScale) -> HRESULT;
        fn SetResourceFormat(resourceFormat: BlackmagicRawResourceFormat ) -> HRESULT;
        fn GetResourceFormat(resourceFormat: *mut BlackmagicRawResourceFormat) -> HRESULT;
        fn GetSensorRate(sensorRate: *mut f32) -> HRESULT;
        fn CreateJobDecodeAndProcessFrame(clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes/* optionally override clipProcessingAttributes */, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes/* optionally override frameProcessingAttributes */, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    impl {
        struct CloneFrameProcessingAttributes => fn clone_frame_processing_attributes(&Self) -> BlackmagicRawFrameProcessingAttributes;	// creates object with current frame processing attributes
        scalar GetFrameIndex                  => fn frame_index(&Self) -> u64;
        scalar GetTimecode                    => fn timecode(&Self) -> String;
        scalar GetMetadata                    => fn metadata(&Self, key: String) -> VariantValue;
        scalar GetResolutionScale             => fn resolution_scale(&Self) -> BlackmagicRawResolutionScale;
        scalar GetResourceFormat              => fn resource_format(&Self) -> BlackmagicRawResourceFormat;
        scalar GetSensorRate                  => fn sensor_rate(&Self) -> f32;
        void SetMetadata                      => fn set_metadata(&Self, key: String, value: VariantValue);	// To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        void SetResolutionScale               => fn set_resolution_scale(&Self, resolution_scale: BlackmagicRawResolutionScale);
        void SetResourceFormat                => fn set_resource_format(&Self, resource_format: BlackmagicRawResourceFormat );
        // custom impl GetMetadataIterator
        // custom impl CreateJobDecodeAndProcessFrame
        interface fn processing_attributes(&self) -> BlackmagicRawFrameProcessingAttributes;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawFrameEx - provides extra information for use with IBlackmagicRawManualDecoder
    BlackmagicRawFrameEx {
        fn GetBitStreamSizeBytes(bitStreamSizeBytes: *mut u32) -> HRESULT;
        fn GetProcessedImageResolution(width: *mut u32, height: *mut u32) -> HRESULT;
    }
    impl {
        scalar GetBitStreamSizeBytes        => fn bit_stream_size_bytes(&Self) -> u32;
        scalar2 GetProcessedImageResolution => fn processed_image_resolution(&Self) -> (u32, u32);
    }
}

braw_interface! {
    /// Interface IBlackmagicRawFrameMultiVideo - provides extra information related to frames from multi video track files
    BlackmagicRawFrameMultiVideo {
        fn GetVideoTrackIndex(videoTrackIndex: *mut u32) -> HRESULT;
    }
    impl {
        scalar GetVideoTrackIndex => fn video_track_index(&Self) -> u32;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawManualDecoderFlow1 - Allowing manual decoding/processing of buffers, Flow1 is a pure CPU solution
    BlackmagicRawManualDecoderFlow1 {
        fn PopulateFrameStateBuffer(frame: *mut IBlackmagicRawFrame, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes/* optionally override clipProcessingAttributes */, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes/* optionally override frameProcessingAttributes */, frameState: *mut c_void, frameStateSizeBytes: u32) -> HRESULT;
        fn GetFrameStateSizeBytes(frameStateSizeBytes: *mut u32) -> HRESULT;
        fn GetDecodedSizeBytes(frameStateBufferCPU: *mut c_void, decodedSizeBytes: *mut u32) -> HRESULT;
        fn GetProcessedSizeBytes(frameStateBufferCPU: *mut c_void, processedSizeBytes: *mut u32) -> HRESULT;
        fn GetPost3DLUTSizeBytes(frameStateBufferCPU: *mut c_void, post3DLUTSizeBytes: *mut u32) -> HRESULT;
        fn CreateJobDecode(frameStateBufferCPU: *mut c_void, bitStreamBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        fn CreateJobProcess(frameStateBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, processedBufferCPU: *mut c_void, post3DLUTBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    impl {
        scalar GetFrameStateSizeBytes => fn frame_state_size_bytes(&Self) -> u32;
        // TODO custom impl GetDecodedSizeBytes
        // TODO custom impl GetProcessedSizeBytes
        // TODO custom impl GetPost3DLUTSizeBytes
        // TODO custom impl CreateJobDecode
        // TODO custom impl CreateJobProcess
        // TODO custom impl PopulateFrameStateBuffer
   }
}

braw_interface! {
    /// Interface IBlackmagicRawManualDecoderFlow2 - Allowing manual decoding/processing of buffers, Flow2 is a hybrid CPU/GPU solution
    BlackmagicRawManualDecoderFlow2 {
        fn PopulateFrameStateBuffer(frame: *mut IBlackmagicRawFrame, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes /* optionally override clipProcessingAttributes */, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes /* optionally override frameProcessingAttributes */, frameState: *mut c_void, frameStateSizeBytes: u32) -> HRESULT;
        fn GetFrameStateSizeBytes(frameStateSizeBytes: *mut u32) -> HRESULT;
        fn GetDecodedSizeBytes(frameStateBufferCPU: *mut c_void, decodedSizeBytes: *mut u32) -> HRESULT;
        fn GetWorkingSizeBytes(frameStateBufferCPU: *mut c_void, workingSizeBytes: *mut u32) -> HRESULT;
        fn GetProcessedSizeBytes(frameStateBufferCPU: *mut c_void, processedSizeBytes: *mut u32) -> HRESULT;
        fn GetPost3DLUTSizeBytes(frameStateBufferCPU: *mut c_void, post3DLUTSizeBytes: *mut u32) -> HRESULT;
        fn CreateJobDecode(frameStateBufferCPU: *mut c_void, bitStreamBufferCPU: *mut c_void, decodedBufferCPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        fn CreateJobProcess(context: *mut c_void, commandQueue: *mut c_void, frameStateBufferCPU: *mut c_void, decodedBufferGPU: *mut c_void, workingBufferGPU: *mut c_void /* additional working buffer */, processedBufferGPU: *mut c_void, post3DLUTBufferGPU: *mut c_void, job: *mut *mut IBlackmagicRawJob) -> HRESULT;	// Create a job to process a frame. This is performed on the specified GPU. After this process is complete a final processed image will be provided via a OnProcessComplete() callback
    }
    impl {
        scalar GetFrameStateSizeBytes => fn frame_state_size_bytes(&Self) -> u32;
        // TODO custom impl GetDecodedSizeBytes
        // TODO custom impl GetProcessedSizeBytes
        // TODO custom impl GetPost3DLUTSizeBytes
        // TODO custom impl CreateJobDecode
        // TODO custom impl CreateJobProcess
        // TODO custom impl PopulateFrameStateBuffer
    }
}


braw_interface! {
    /// Interface IBlackmagicRawClip - Opened movie clip
    BlackmagicRawClip {
        fn GetWidth(out: *mut u32) -> HRESULT;
        fn GetHeight(out: *mut u32) -> HRESULT;
        fn GetFrameRate(out: *mut f32) -> HRESULT;
        fn GetFrameCount(out: *mut u64) -> HRESULT;
        fn GetTimecodeForFrame(frameIndex: u64, timecode: *mut *mut c_void) -> HRESULT;
        fn GetMetadataIterator(iterator: *mut *mut IBlackmagicRawMetadataIterator) -> HRESULT;
        fn GetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;
        fn SetMetadata(key: *const c_void, value: *mut VARIANT) -> HRESULT;	// To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        fn GetCameraType(cameraType: *mut *mut c_void) -> HRESULT;
        fn CloneClipProcessingAttributes(clipProcessingAttributes: *mut *mut IBlackmagicRawClipProcessingAttributes) -> HRESULT;	// creates object with current clip processing attributes
        fn GetMulticardFileCount(multicardFileCount: *mut u32) -> HRESULT;
        fn IsMulticardFilePresent(index: u32, /* out */ isMulticardFilePresent: *mut bool) -> HRESULT;
        fn GetSidecarFileAttached(/* out */ isSidecarFileAttached: *mut bool) -> HRESULT;	// Check for successfully parsed sidecar file, which is automatically loaded upon opening of a clip
        fn SaveSidecarFile() -> HRESULT;	// Save metadata to sidecar file
        fn ReloadSidecarFile() -> HRESULT;	// Reload metadata from sidecar file
        fn CreateJobReadFrame(frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;	// Create a job to read a frame
        fn CreateJobTrim(fileName: *const c_void, frameIndex: u64, frameCount: u64, clipProcessingAttributes: *mut IBlackmagicRawClipProcessingAttributes/* optional */, frameProcessingAttributes: *mut IBlackmagicRawFrameProcessingAttributes/* optional */, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        fn CloneWithGeometry(geometry: *mut IBlackmagicRawClipGeometry, clip: *mut *mut IBlackmagicRawClip) -> HRESULT;
    }
    impl {
        struct CloneClipProcessingAttributes => fn clone_clip_processing_attributes(&Self) -> BlackmagicRawClipProcessingAttributes;	// creates object with current clip processing attributes
        struct CloneWithGeometry             => fn clone_with_geometry(&Self, geometry: BlackmagicRawClipGeometry) -> BlackmagicRawClip;
        scalar GetWidth                      => fn width(&Self) -> u32;
        scalar GetHeight                     => fn height(&Self) -> u32;
        scalar GetFrameRate                  => fn frame_rate(&Self) -> f32;
        scalar GetFrameCount                 => fn frame_count(&Self) -> u64;
        scalar GetTimecodeForFrame           => fn timecode_for_frame(&Self, frame_index: u64) -> String;
        scalar GetMetadata                   => fn metadata(&Self, key: String) -> VariantValue;
        scalar GetCameraType                 => fn camera_type(&Self) -> String;
        scalar GetMulticardFileCount         => fn multicard_file_count(&Self) -> u32;
        scalar IsMulticardFilePresent        => fn is_multicard_file_present(&Self, index: u32) -> bool;
        scalar GetSidecarFileAttached        => fn sidecar_file_attached(&Self) -> bool;	// Check for successfully parsed sidecar file, which is automatically loaded upon opening of a clip
        void SetMetadata                     => fn set_metadata(&Self, key: String, val: VariantValue);	// To clear metadata to movie original, set value to NULL. This data is not saved to disk until SaveSideCar() is called
        void SaveSidecarFile                 => fn save_sidecar_file(&Self);	// Save metadata to sidecar file
        void ReloadSidecarFile               => fn reload_sidecar_file(&Self);	// Reload metadata from sidecar file
        // custom impl GetMetadataIterator
        // custom impl CreateJobReadFrame
        // custom impl CreateJobTrim
        interface fn ex(&self) -> BlackmagicRawClipEx;
        interface fn processing_attributes(&self) -> BlackmagicRawClipProcessingAttributes;
        interface fn audio(&self) -> BlackmagicRawClipAudio;
        interface fn multi_video(&self) -> BlackmagicRawClipMultiVideo;
        interface fn accelerometer_motion(&self) -> BlackmagicRawClipAccelerometerMotion;
        interface fn gyroscope_motion(&self) -> BlackmagicRawClipGyroscopeMotion;
        interface fn pdaf_data(&self) -> BlackmagicRawClipPDAFData;
    }
}


braw_interface! {
    /// Interface IBlackmagicRawClipEx - Extended use of IBlackmagicRawClip, to pass custom bitstream
    BlackmagicRawClipEx {
        fn GetMaxBitStreamSizeBytes(maxBitStreamSizeBytes: *mut u32) -> HRESULT;
        fn GetBitStreamSizeBytes(frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        fn CreateJobReadFrame(frameIndex: u64, bitStream: *mut c_void/* optional */, bitStreamSizeBytes: u32 /* optional */, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        fn QueryTimecodeInfo(baseFrameIndex: *mut u32, isDropFrameTimecode: *mut bool) -> HRESULT;
    }
    impl {
        scalar GetMaxBitStreamSizeBytes => fn max_bit_stream_size_bytes(&Self) -> u32;
        scalar GetBitStreamSizeBytes    => fn bit_stream_size_bytes(&Self, frame_index: u64) -> u32;
        scalar2 QueryTimecodeInfo       => fn timecode_info(&Self) -> (u32, bool);
        // custom impl CreateJobReadFrame
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipMultiVideo - Extended use of IBlackmagicRawClip, to support multi video track video clips
    BlackmagicRawClipMultiVideo {
        fn GetVideoTrackCount(videoTrackCount: *mut u32) -> HRESULT;
        fn GetVideoFrameCount(videoTrackIndex: u32, videoFrameCount: *mut u64) -> HRESULT;
        fn GetVideoTrackSource(videoTrackIndex: u32, videoSourceFourCC: *mut u32) -> HRESULT;
        fn CreateJobReadFrame(videoTrackIndex: u32, frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;	// Create a job o read a frame
        fn GetBitStreamSizeBytes(videoTrackIndex: u32, frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        fn CreateJobReadFrameEx(videoTrackIndex: u32, frameIndex: u64, bitStream : *mut c_void/* optional */, bitStreamSizeBytes: u32 /* optional */, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
    }
    impl {
        scalar GetVideoTrackCount  => fn video_track_count (&Self) -> u32;
        scalar GetVideoFrameCount  => fn video_frame_count (&Self, video_track_index: u32) -> u64;
        scalar GetVideoTrackSource => fn video_track_source(&Self, video_track_index: u32) -> u32;
        // custom impl CreateJobReadFrame
        // custom impl CreateJobReadFrameEx
        scalar GetBitStreamSizeBytes => fn bit_stream_size_bytes(&Self, video_track_index: u32, frame_index: u64) -> u32;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipImmersiveVideo - Extended use of IBlackmagicRawClip, to support immersive video clips
    BlackmagicRawClipImmersiveVideo {
        fn CreateJobImmersiveReadFrame(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, job: *mut *mut IBlackmagicRawJob) -> HRESULT;	// Create a job to read a frame
        fn GetImmersiveBitStreamSizeBytes(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, bitStreamSizeBytes: *mut u32) -> HRESULT;
        fn CreateJobImmersiveReadFrameEx(videoTrack: BlackmagicRawImmersiveVideoTrack, frameIndex: u64, bitStream: *mut c_void/* optional */, bitStreamSizeBytes: u32 /* optional */, job: *mut *mut IBlackmagicRawJob) -> HRESULT;
        fn GetDistanceBetweenLenses(distanceBetweenLenses: *mut u32) -> HRESULT;
        fn GetComfortDisparityAdjustment(comfortDisparityAdjustment: *mut i32) -> HRESULT;
        fn GetHorizontalFieldOfView(horizontalFieldOfView: *mut u32) -> HRESULT;
        fn GetImmersiveAttribute(attribute: BlackmagicRawImmersiveAttribute, value: *mut VARIANT) -> HRESULT;
    }
    impl {
        scalar GetDistanceBetweenLenses       => fn distance_between_lenses       (&Self) -> u32;
        scalar GetComfortDisparityAdjustment  => fn comfort_disparity_adjustment  (&Self) -> i32;
        scalar GetHorizontalFieldOfView       => fn horizontal_field_of_view      (&Self) -> u32;
        scalar GetImmersiveAttribute          => fn immersive_attribute           (&Self, attribute: BlackmagicRawImmersiveAttribute) -> VariantValue;
        // custom impl CreateJobImmersiveReadFrame
        // custom impl CreateJobImmersiveReadFrameEx
        scalar GetImmersiveBitStreamSizeBytes => fn immersive_bit_stream_size_bytes(&Self, video_track: BlackmagicRawImmersiveVideoTrack, frame_index: u64) -> u32;
    }
}

braw_interface! {
    /// Interface IBlackmagicRawClipResolutions - Supports querying of resolutions and/or scales for processed image results
    BlackmagicRawClipResolutions {
        fn GetResolutionCount(resolutionCount: *mut u32) -> HRESULT;
        fn GetResolution(resolutionIndex: u32, resolutionWidthPixels: *mut u32, resolutionHeightPixels: *mut u32) -> HRESULT;
        fn GetRecordedResolution(resolutionIndex: u32, recordedResolutionWidthPixels: *mut u32, recordedResolutionHeightPixels: *mut u32) -> HRESULT;
        fn GetClosestResolutionForScale(resolutionScale: BlackmagicRawResolutionScale, resolutionWidthPixels: *mut u32, resolutionHeightPixels: *mut u32) -> HRESULT;
        fn GetClosestScaleForResolution(resolutionWidthPixels: u32, resolutionHeightPixels: u32, resolutionScale: *mut BlackmagicRawResolutionScale) -> HRESULT;
    }
    impl {
        scalar GetResolutionCount            => fn resolution_count            (&Self) -> u32;
        scalar GetClosestScaleForResolution  => fn closest_scale_for_resolution(&Self, resolution_width_pixels: u32, resolution_height_pixels: u32) -> BlackmagicRawResolutionScale;
        scalar2 GetResolution                => fn resolution                  (&Self, resolution_index: u32) -> (u32, u32);
        scalar2 GetRecordedResolution        => fn recorded_resolution         (&Self, resolution_index: u32) -> (u32, u32);
        scalar2 GetClosestResolutionForScale => fn closest_resolution_for_scale(&Self, resolution_scale: BlackmagicRawResolutionScale) -> (u32, u32);
    }
}
