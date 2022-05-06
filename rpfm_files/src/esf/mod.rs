//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with ESF PackedFiles.

ESF are like savestates of the game.
!*/

use anyhow::{anyhow, Result};
use bitflags::bitflags;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_common::{decoder::Decoder, rpfm_macros::*, schema::Schema};

use crate::{Decodeable, Encodeable, FileType};

/// Extensions used by CEO/ESF PackedFiles.
pub const EXTENSIONS: [&str; 3] = [".ccd", ".esf", ".save"];

/// Signatured/Magic Numbers/Whatever of a ESF PackedFile.
pub const SIGNATURE_CAAB: &[u8; 4] = &[0xCA, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CEAB: &[u8; 4] = &[0xCE, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CFAB: &[u8; 4] = &[0xCF, 0xAB, 0x00, 0x00];

pub mod caab;
//pub mod diff;

//---------------------------------------------------------------------------//
//                              Markers, from ESFEdit
//---------------------------------------------------------------------------//

/// Invalid marker.
pub const INVALID: u8 = 0x00;

/// Primitives
pub const BOOL: u8 = 0x01;
pub const I8: u8 = 0x02;
pub const I16: u8 = 0x03;
pub const I32: u8 = 0x04;
pub const I64: u8 = 0x05;
pub const U8: u8 = 0x06;
pub const U16: u8 = 0x07;
pub const U32: u8 = 0x08;
pub const U64: u8 = 0x09;
pub const F32: u8 = 0x0a;
pub const F64: u8 = 0x0b;
pub const COORD_2D: u8 = 0x0c;
pub const COORD_3D: u8 = 0x0d;
pub const UTF16: u8 = 0x0e;
pub const ASCII: u8 = 0x0f;
pub const ANGLE: u8 = 0x10;

/// Optimized Primitives
pub const BOOL_TRUE: u8 = 0x12;
pub const BOOL_FALSE: u8 = 0x13;
pub const U32_ZERO: u8 = 0x14;
pub const U32_ONE: u8 = 0x15;
pub const U32_BYTE: u8 = 0x16;
pub const U32_16BIT: u8 = 0x17;
pub const U32_24BIT: u8 = 0x18;
pub const I32_ZERO: u8 = 0x19;
pub const I32_BYTE: u8 = 0x1a;
pub const I32_16BIT: u8 = 0x1b;
pub const I32_24BIT: u8 = 0x1c;
pub const F32_ZERO: u8 = 0x1d;

/// Unknown Types
pub const UNKNOWN_21: u8 = 0x21;
pub const UNKNOWN_23: u8 = 0x23;
pub const UNKNOWN_24: u8 = 0x24;
pub const UNKNOWN_25: u8 = 0x25;

/// Three Kingdoms DLC Eight Princes types
pub const UNKNOWN_26: u8 = 0x26;

/// Primitive Arrays
pub const BOOL_ARRAY: u8 = 0x41;
pub const I8_ARRAY: u8 = 0x42;
pub const I16_ARRAY: u8 = 0x43;
pub const I32_ARRAY: u8 = 0x44;
pub const I64_ARRAY: u8 = 0x45;
pub const U8_ARRAY: u8 = 0x46;
pub const U16_ARRAY: u8 = 0x47;
pub const U32_ARRAY: u8 = 0x48;
pub const U64_ARRAY: u8 = 0x49;
pub const F32_ARRAY: u8 = 0x4a;
pub const F64_ARRAY: u8 = 0x4b;
pub const COORD_2D_ARRAY: u8 = 0x4c;
pub const COORD_3D_ARRAY: u8 = 0x4d;
pub const UTF16_ARRAY: u8 = 0x4e;
pub const ASCII_ARRAY: u8 = 0x4f;
pub const ANGLE_ARRAY: u8 = 0x50;

/// Optimized Arrays
pub const BOOL_TRUE_ARRAY: u8 = 0x52; // makes no sense
pub const BOOL_FALSE_ARRAY: u8 = 0x53; // makes no sense
pub const U32_ZERO_ARRAY: u8 = 0x54; // makes no sense
pub const U32_ONE_ARRAY: u8 = 0x55; // makes no sense
pub const U32_BYTE_ARRAY: u8 = 0x56;
pub const U32_16BIT_ARRAY: u8 = 0x57;
pub const U32_24BIT_ARRAY: u8 = 0x58;
pub const I32_ZERO_ARRAY: u8 = 0x59; // makes no sense
pub const I32_BYTE_ARRAY: u8 = 0x5a;
pub const I32_16BIT_ARRAY: u8 = 0x5b;
pub const I32_24BIT_ARRAY: u8 = 0x5c;
pub const F32_ZERO_ARRAY: u8 = 0x5d;  // makes no sense

pub const COMPRESSED_DATA_TAG: &str = "COMPRESSED_DATA";
pub const COMPRESSED_DATA_INFO_TAG: &str = "COMPRESSED_DATA_INFO";

// Blocks have quite a few bits that can toggle their behavior.
bitflags! {

    /// This represents the bitmasks a Record Block can have applied to its type byte.
    #[derive(Default, Serialize, Deserialize)]
    pub struct RecordNodeFlags: u8 {

        /// Used to specify that the type is indeed a record block.
        const IS_RECORD_NODE            = 0b1000_0000;

        /// Used to specify that this block contains nested groups of nodes.
        const HAS_NESTED_BLOCKS         = 0b0100_0000;

        /// Used to specify that this block doesn't use optimized integers for version and name index.
        const HAS_NON_OPTIMIZED_INFO    = 0b0010_0000;
    }
}

const ERROR_INCOMPLETE_DECODING: &str = "There are bytes still to decode, but the decoding process has finished. This means RPFM cannot yet decode this file correctly.
If you see this message, please report it to RPFM's author so support for the file that caused the error can be implemented.";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire ESF PackedFile decoded in memory.
#[derive(GetRef, Set, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ESF {
    signature: ESFSignature,
    unknown_1: u32,
    creation_date: u32,
    root_node: NodeType,
}

/// This enum contains the different signatures of ESF files.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ESFSignature {

    /// Signature found on 3K files.
    CAAB,
    CEAB,
    CFAB
}

/// Node types supported by the ESF.
///
/// NOTE: These are partially extracted from EditSF.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {

    /// Invalid type.
    Invalid,

    /// Primitive nodes.
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

    /// Unknown Types
    Unknown21(u32),
    Unknown23(u8),
    //Unknown24(u32),
    Unknown25(u32),
    Unknown26(Vec<u8>),

    /// Primitive Arrays
    BoolArray(Vec<bool>),
    I8Array(Vec<i8>),
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

    /// Record nodes
    Record(RecordNode),
}

/// Node containing a bool value, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct BoolNode {
    value: bool,
    optimized: bool,
}

/// Node containing an i32 value, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct I32Node {
    value: i32,
    optimized: bool,
}

/// Node containing an u32 value, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct U32Node {
    value: u32,
    optimized: bool,
}

/// Node containing an f32 value, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct F32Node {
    value: f32,
    optimized: bool,
}

/// Node containing a Vec<i32>, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct VecI32Node {
    value: Vec<i32>,
    optimized: bool,
}

/// Node containing a Vec<u32>, and if the node should be optimized or not.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct VecU32Node {
    value: Vec<u32>,
    optimized: bool,
}

/// Node containing a pair of X/Y coordinates.
#[derive(GetRef, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Coordinates2DNode {
    x: f32,
    y: f32,
}

/// Node containing a group of X/Y/Z coordinates.
#[derive(GetRef, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Coordinates3DNode {
    x: f32,
    y: f32,
    z: f32,
}

/// Node containing a record of data. Basically, a node with other nodes attached to it.
#[derive(GetRef, Set, Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecordNode {
    record_flags: RecordNodeFlags,
    version: u8,
    name: String,
    children: Vec<Vec<NodeType>>
}

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

/// Implementation of `ESF`.
impl ESF {

    /// This function returns if the provided data corresponds to a ESF or not.
    pub fn is_esf(data: &[u8]) -> bool {
        match data.decode_bytes_checked(0, 4) {
            Ok(signature) => signature == SIGNATURE_CAAB,
            Err(_) => false,
        }
    }

    /// This function creates a copy of an ESF without the root node..
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

                Self::Record(new_node)
            }

            _ => self.clone()
        }
    }

    /*pub fn get_removed_nodes(&self, vanilla_node: &NodeType) -> NodeType {
        match vanilla_node {
            Self::Record(vanilla_node) => {
                match self {
                    Self::Record(node) => {

                        // If there's a difference in the nodes, it may be due to missing nodes.
                        // We need to dig deeper.
                        if vanilla_node.get_ref_children() != node.get_ref_children() {

                        }
                    }
                }
            }
        }
    }*/
/*
    /// This function checks if the provided NodeType values are "equal", even if the type is different.
    pub fn eq_value(&self, other: &Self) -> bool {
        match self {
           // Invalid type.
            Self::Invalid => other == &Self::Invalid,

            // Primitive nodes.
            Self::Bool(value) => match other {
                Self::Bool(other_value) => value.optimized == other_value.optimized && value.value == other_value.value,
                Self::BoolTrue => value.optimized && value.value,
                Self::BoolFalse => value.optimized && !value.value,
                _ => false
            },
            Self::I8(value) => match other {
                Self::I8(other_value) => value == other_value,
                _ => false
            },
            Self::I16(value) => match other {
                Self::I16(other_value) => value == other_value,
                _ => false
            },
            Self::I32(value) => match other {
                Self::I32Zero => *value == 0,
                Self::I32Byte(other_value) => value == other_value,
                Self::I32_16bit(other_value) => value == other_value,
                Self::I32_24bit(other_value) => value == other_value,
                Self::I32(other_value) => value == other_value,
                _ => false
            },
            Self::I64(value) => match other {
                Self::I64(other_value) => value == other_value,
                _ => false
            },
            Self::U8(value) => match other {
                Self::U8(other_value) => value == other_value,
                _ => false
            },
            Self::U16(value) => match other {
                Self::U16(other_value) => value == other_value,
                _ => false
            },
            Self::U32(value) => match other {
                Self::U32Zero => *value == 0,
                Self::U32One => *value == 1,
                Self::U32Byte(other_value) => value == other_value,
                Self::U32_16bit(other_value) => value == other_value,
                Self::U32_24bit(other_value) => value == other_value,
                Self::U32(other_value) => value == other_value,
                _ => false
            },
            Self::U64(value) => match other {
                Self::U64(other_value) => value == other_value,
                _ => false
            },
            Self::F32(value) => match other {
                Self::F32(other_value) =>  (value - other_value).abs() >= std::f32::EPSILON,
                Self::F32Zero =>  (value - 0.0).abs() >= std::f32::EPSILON,
                _ => false
            },
            Self::F64(value) => match other {
                Self::F64(other_value) => value == other_value,
                _ => false
            },
            Self::Coord2d(value) => match other {
                Self::Coord2d(other_value) => value == other_value,
                _ => false
            },
            Self::Coord3d(value) => match other {
                Self::Coord3d(other_value) => value == other_value,
                _ => false
            },
            Self::Utf16(value) => match other {
                Self::Utf16(other_value) => value == other_value,
                _ => false
            },
            Self::Ascii(value) => match other {
                Self::Ascii(other_value) => value == other_value,
                _ => false
            },
            Self::Angle(value) => match other {
                Self::Angle(other_value) => value == other_value,
                _ => false
            },

            // Optimized Primitives
            Self::BoolTrue => match other {
                Self::Bool(other_value) => other_value.optimized && other_value.value,
                Self::BoolTrue => true,
                _ => false
            },
            Self::BoolFalse => match other {
                Self::Bool(other_value) => other_value.optimized && !other_value.value,
                Self::BoolFalse => true,
                _ => false
            },
            Self::U32Zero => match other {
                Self::U32Zero => true,
                Self::U32One => false,
                Self::U32Byte(other_value) => *other_value == 0,
                Self::U32_16bit(other_value) => *other_value == 0,
                Self::U32_24bit(other_value) => *other_value == 0,
                Self::U32(other_value) => *other_value == 0,
                _ => false
            },
            Self::U32One => match other {
                Self::U32Zero => false,
                Self::U32One => true,
                Self::U32Byte(other_value) => *other_value == 1,
                Self::U32_16bit(other_value) => *other_value == 1,
                Self::U32_24bit(other_value) => *other_value == 1,
                Self::U32(other_value) => *other_value == 1,
                _ => false
            },
            Self::U32Byte(value) => {false},
            Self::U32_16bit(value) => {false},
            Self::U32_24bit(value) => {false},
            Self::I32Zero => {false},
            Self::I32Byte(value) => {false},
            Self::I32_16bit(value) => {false},
            Self::I32_24bit(value) => {false},
            Self::F32Zero => {false},

            // Unknown Types
            Self::Unknown21(value) => {false},
            Self::Unknown23(value) => {false},
            //Self::Unknown24(u32) => {false},
            Self::Unknown25(value) => {false},
            Self::Unknown26(value) => {false},

            // Primitive Arrays
            Self::BoolArray(value) => {false},
            Self::I8Array(value) => {false},
            Self::I16Array(value) => {false},
            Self::I32Array(value) => {false},
            Self::I64Array(value) => {false},
            Self::U8Array(value) => {false},
            Self::U16Array(value) => {false},
            Self::U32Array(value) => {false},
            Self::U64Array(value) => {false},
            Self::F32Array(value) => {false},
            Self::F64Array(value) => {false},
            Self::Coord2dArray(value) => {false},
            Self::Coord3dArray(value) => {false},
            Self::Utf16Array(value) => {false},
            Self::AsciiArray(value) => {false},
            Self::AngleArray(value) => {false},

            // Optimized Arrays
            Self::U32ByteArray(value) => {false},
            Self::U32_16bitArray(value) => {false},
            Self::U32_24bitArray(value) => {false},
            Self::I32ByteArray(value) => {false},
            Self::I32_16bitArray(value) => {false},
            Self::I32_24bitArray(value) => {false},

            // Record nodes
            Self::Record(value) => {false},
        }
    }*/
}

/// Default implementation for `ESF`.
impl Default for ESF {
    fn default() -> Self {
        Self {
            signature: ESFSignature::CAAB,
            unknown_1: 0,
            creation_date: 0,
            root_node: NodeType::Invalid,
        }
    }
}

/// Display implementation for `ESFSignature`.
impl Display for ESFSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::CAAB => "CAAB",
            Self::CEAB => "CEAB",
            Self::CFAB => "CFAB",
        }, f)
    }
}

/// Implementation to create an `ESFSignature` from a `&str`.
impl From<&str> for ESFSignature {
    fn from(data: &str) -> Self {
        match data {
            "CAAB" => Self::CAAB,
            "CEAB" => Self::CEAB,
            "CFAB" => Self::CFAB,
            _ => unimplemented!()
        }
    }
}


impl Decodeable for ESF {

    fn file_type(&self) -> FileType {
        FileType::ESF
    }

    fn decode(packed_file_data: &[u8], _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {
        let signature_bytes: &[u8; 4] = packed_file_data.decode_bytes_checked(0, 4)?.try_into()?;

        // Match known signatures.
        let signature = match signature_bytes {
            SIGNATURE_CAAB => ESFSignature::CAAB,
            SIGNATURE_CEAB => ESFSignature::CEAB,
            SIGNATURE_CFAB => ESFSignature::CFAB,
            _ => return Err(anyhow!("Unsupported signature: {:#X}{:#X}", signature_bytes[0], signature_bytes[1])),
        };

        // Match signatures that we can actually decode.
        let esf = match signature {
            ESFSignature::CAAB => Self::read_caab(packed_file_data)?,
            _ => return Err(anyhow!("Unsupported signature: {:#X}{:#X}", signature_bytes[0], signature_bytes[1])),
        };

        //use std::io::Write;
        //let mut x = std::fs::File::create("ceo.json")?;
        //x.write_all(&serde_json::to_string_pretty(&esf).unwrap().as_bytes())?;

        Ok(esf)
    }
}

impl Encodeable for ESF {
    fn encode(&self) -> Vec<u8> {
        match self.signature {
            ESFSignature::CAAB => self.save_caab(),
            _ => return vec![],
        }
    }
}
