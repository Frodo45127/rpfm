//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write ESF (Empire Safe File) files.
//!
//! ESF files are special files used to hold a variety of data, ranging from trade routes info
//! to entire campaign savestates.
//!
//! Due to the huge complexity of these files, the spec is defined in the submodules containing
//! the logic for each variation of this file.

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// Extensions used by ESF files.
pub const EXTENSIONS: [&str; 6] = [
    ".csc",                 // CSC files.
    ".ccd",                 // CEO files.
    ".esf",                 // ESF files.
    ".save",                // Game save files.
    ".save_multiplayer",    // Game save files, multiplayer.
    ".twc",                 // Character save files.
];

/// Signatured/Magic Numbers/Whatever of a ESF file.
pub const SIGNATURE_CAAB: &[u8; 4] = &[0xCA, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CBAB: &[u8; 4] = &[0xCB, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CEAB: &[u8; 4] = &[0xCE, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CFAB: &[u8; 4] = &[0xCF, 0xAB, 0x00, 0x00];

mod caab;
mod cbab;
mod utils;

#[cfg(test)] mod esf_test;

//---------------------------------------------------------------------------//
//                              Markers, from ESFEdit
//---------------------------------------------------------------------------//

// Invalid marker.
pub const INVALID: u8 = 0x00;

// Primitives
const BOOL: u8 = 0x01;
const I8: u8 = 0x02;
const I16: u8 = 0x03;
const I32: u8 = 0x04;
const I64: u8 = 0x05;
const U8: u8 = 0x06;
const U16: u8 = 0x07;
const U32: u8 = 0x08;
const U64: u8 = 0x09;
const F32: u8 = 0x0a;
const F64: u8 = 0x0b;
const COORD_2D: u8 = 0x0c;
const COORD_3D: u8 = 0x0d;
const UTF16: u8 = 0x0e;
const ASCII: u8 = 0x0f;
const ANGLE: u8 = 0x10;

// Optimized Primitives
const BOOL_TRUE: u8 = 0x12;
const BOOL_FALSE: u8 = 0x13;
const U32_ZERO: u8 = 0x14;
const U32_ONE: u8 = 0x15;
const U32_BYTE: u8 = 0x16;
const U32_16BIT: u8 = 0x17;
const U32_24BIT: u8 = 0x18;
const I32_ZERO: u8 = 0x19;
const I32_BYTE: u8 = 0x1a;
const I32_16BIT: u8 = 0x1b;
const I32_24BIT: u8 = 0x1c;
const F32_ZERO: u8 = 0x1d;

// Unknown Types
const UNKNOWN_21: u8 = 0x21;
const UNKNOWN_23: u8 = 0x23;
const UNKNOWN_24: u8 = 0x24;
const UNKNOWN_25: u8 = 0x25;

// Three Kingdoms DLC Eight Princes types
const UNKNOWN_26: u8 = 0x26;

// Primitive Arrays
const BOOL_ARRAY: u8 = 0x41;
const I8_ARRAY: u8 = 0x42;
const I16_ARRAY: u8 = 0x43;
const I32_ARRAY: u8 = 0x44;
const I64_ARRAY: u8 = 0x45;
const U8_ARRAY: u8 = 0x46;
const U16_ARRAY: u8 = 0x47;
const U32_ARRAY: u8 = 0x48;
const U64_ARRAY: u8 = 0x49;
const F32_ARRAY: u8 = 0x4a;
const F64_ARRAY: u8 = 0x4b;
const COORD_2D_ARRAY: u8 = 0x4c;
const COORD_3D_ARRAY: u8 = 0x4d;
const UTF16_ARRAY: u8 = 0x4e;
const ASCII_ARRAY: u8 = 0x4f;
const ANGLE_ARRAY: u8 = 0x50;

// Optimized Arrays
const BOOL_TRUE_ARRAY: u8 = 0x52; // makes no sense
const BOOL_FALSE_ARRAY: u8 = 0x53; // makes no sense
const U32_ZERO_ARRAY: u8 = 0x54; // makes no sense
const U32_ONE_ARRAY: u8 = 0x55; // makes no sense
const U32_BYTE_ARRAY: u8 = 0x56;
const U32_16BIT_ARRAY: u8 = 0x57;
const U32_24BIT_ARRAY: u8 = 0x58;
const I32_ZERO_ARRAY: u8 = 0x59; // makes no sense
const I32_BYTE_ARRAY: u8 = 0x5a;
const I32_16BIT_ARRAY: u8 = 0x5b;
const I32_24BIT_ARRAY: u8 = 0x5c;
const F32_ZERO_ARRAY: u8 = 0x5d;  // makes no sense

const COMPRESSED_TAGS: [&str; 1] = ["CAMPAIGN_ENV"];
const COMPRESSED_DATA_TAG: &str = "COMPRESSED_DATA";
const COMPRESSED_DATA_INFO_TAG: &str = "COMPRESSED_DATA_INFO";

// Blocks have quite a few bits that can be toggle to change their behavior.
bitflags! {

    /// This represents the bitmasks a Record Block can have applied to its type byte.
    #[derive(PartialEq, Clone, Copy, Default, Debug, Serialize, Deserialize)]
    pub struct RecordNodeFlags: u8 {

        /// Used to specify that the type is indeed a record block.
        const IS_RECORD_NODE            = 0b1000_0000;

        /// Used to specify that this block contains nested groups of nodes.
        const HAS_NESTED_BLOCKS         = 0b0100_0000;

        /// Used to specify that this block doesn't use optimized integers for version and name index.
        const HAS_NON_OPTIMIZED_INFO    = 0b0010_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire ESF decoded in memory.
#[derive(Getters, Setters, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct ESF {

    /// Signature of the ESF.
    signature: ESFSignature,

    /// Unknown value.
    unknown_1: u32,

    /// Creation date of the ESF.
    creation_date: u32,

    /// Root node of the node tree, containing the entire ESF data on it.
    root_node: NodeType,
}

/// This enum represents the different signatures of ESF files.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum ESFSignature {
    #[default]CAAB,
    CBAB,
    CEAB,
    CFAB
}

/// This enum represents all known node types present on ESF files.
///
/// NOTE: These are partially extracted from EditSF.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub enum NodeType {

    // Invalid type.
    #[default]
    Invalid,

    // Primitive nodes.
    Bool(BoolNode),
    I8(i8),
    I16(i16),
    I32(I32Node),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(U32Node),
    U64(u64),
    F32(F32Node),
    F64(f64),
    Coord2d(Coordinates2DNode),
    Coord3d(Coordinates3DNode),
    Utf16(String),
    Ascii(String),
    Angle(i16),

    // Unknown Types
    Unknown21(u32),
    Unknown23(u8),
    Unknown24(u16),
    Unknown25(u32),
    Unknown26(Vec<u8>),

    // Primitive Arrays
    BoolArray(Vec<bool>),
    I8Array(Vec<u8>),
    I16Array(Vec<i16>),
    I32Array(VecI32Node),
    I64Array(Vec<i64>),
    U8Array(Vec<u8>),
    U16Array(Vec<u16>),
    U32Array(VecU32Node),
    U64Array(Vec<u64>),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
    Coord2dArray(Vec<Coordinates2DNode>),
    Coord3dArray(Vec<Coordinates3DNode>),
    Utf16Array(Vec<String>),
    AsciiArray(Vec<String>),
    AngleArray(Vec<i16>),

    // Record nodes
    Record(Box<RecordNode>),
}

/// Node containing a bool value, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BoolNode {
    value: bool,
    optimized: bool,
}

/// Node containing an i32 value, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct I32Node {
    value: i32,
    optimized: bool,
}

/// Node containing an u32 value, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct U32Node {
    value: u32,
    optimized: bool,
}

/// Node containing an f32 value, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct F32Node {
    value: f32,
    optimized: bool,
}

/// Node containing a `Vec<i32>`, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VecI32Node {
    value: Vec<i32>,
    optimized: bool,
}

/// Node containing a `Vec<u32>`, and if the node should be optimized or not.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VecU32Node {
    value: Vec<u32>,
    optimized: bool,
}

/// Node containing a pair of X/Y coordinates.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Coordinates2DNode {
    x: f32,
    y: f32,
}

/// Node containing a group of X/Y/Z coordinates.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Coordinates3DNode {
    x: f32,
    y: f32,
    z: f32,
}

/// Node containing a record of data. Basically, a node with other nodes attached to it.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RecordNode {

    /// Flags applied to this record node.
    record_flags: RecordNodeFlags,

    /// Version of this record node.
    version: u8,

    /// Name of the record node.
    name: String,

    /// Children nodes of this record node.
    children: Vec<Vec<NodeType>>
}

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

/// Implementation of `ESF`.
impl ESF {

    /// This function creates a copy of an ESF without the root node.
    pub fn clone_without_root_node(&self) -> Self {
        Self {
            signature: self.signature,
            unknown_1: self.unknown_1,
            creation_date: self.creation_date,
            root_node: NodeType::Invalid,
        }
    }
}

/// Implementation of `NodeType`.
impl NodeType {

    /// This function creates a copy of a node without its children.
    pub fn clone_without_children(&self) -> Self {
        match self {
            Self::Record(node) => {
                let mut new_node = RecordNode::default();
                new_node.set_name(node.name().to_owned());
                new_node.set_record_flags(*node.record_flags());
                new_node.set_version(*node.version());

                Self::Record(Box::new(new_node))
            }

            _ => self.clone()
        }
    }
}

/// Display implementation for `ESFSignature`.
impl Display for ESFSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::CAAB => "CAAB",
            Self::CBAB => "CBAB",
            Self::CEAB => "CEAB",
            Self::CFAB => "CFAB",
        }, f)
    }
}

impl TryFrom<&str> for ESFSignature {
    type Error = RLibError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CAAB" => Ok(Self::CAAB),
            "CBAB" => Ok(Self::CBAB),
            "CEAB" => Ok(Self::CEAB),
            "CFAB" => Ok(Self::CFAB),
            _ => Err(RLibError::UnknownESFSignature(value.to_string())),
        }
    }
}

impl TryFrom<Vec<u8>> for ESFSignature {
    type Error = RLibError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.as_slice().try_into()? {
            SIGNATURE_CAAB => Ok(Self::CAAB),
            SIGNATURE_CBAB => Ok(Self::CBAB),
            SIGNATURE_CEAB => Ok(Self::CEAB),
            SIGNATURE_CFAB => Ok(Self::CFAB),
            _ => Err(RLibError::UnknownESFSignatureBytes(value[0], value[1])),
        }
    }
}

impl From<ESFSignature> for Vec<u8> {
    fn from(value: ESFSignature) -> Self {
        match value {
            ESFSignature::CAAB => SIGNATURE_CAAB.to_vec(),
            ESFSignature::CBAB => SIGNATURE_CBAB.to_vec(),
            ESFSignature::CEAB => SIGNATURE_CEAB.to_vec(),
            ESFSignature::CFAB => SIGNATURE_CFAB.to_vec(),
        }
    }
}

impl Decodeable for ESF {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut esf = Self::default();

        let sig_bytes = data.read_slice(4, false)?;
        esf.signature = ESFSignature::try_from(sig_bytes.to_vec())?;

        match esf.signature {
            ESFSignature::CAAB => Self::read_caab(&mut esf, data)?,
            ESFSignature::CBAB => Self::read_cbab(&mut esf, data)?,
            _ => return Err(RLibError::DecodingESFUnsupportedSignature(sig_bytes[0], sig_bytes[1])),
        };

        // Debugging code.
        //use std::io::Write;
        //let mut x = std::fs::File::create("ceo.json")?;
        //x.write_all(&serde_json::to_string_pretty(&esf).unwrap().as_bytes())?;

        Ok(esf)
    }
}

impl Encodeable for ESF {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let sig_bytes: Vec<u8> = Vec::from(self.signature);
        buffer.write_all(&sig_bytes)?;

        match self.signature {
            ESFSignature::CAAB => self.save_caab(buffer, extra_data),
            ESFSignature::CBAB => self.save_cbab(buffer, extra_data),
            _ => Err(RLibError::EncodingESFUnsupportedSignature(self.signature.to_string())),
        }
    }
}
