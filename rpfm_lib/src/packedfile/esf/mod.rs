//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use rpfm_error::{ErrorKind, Result};
use rpfm_macros::*;
use serde_json::to_string_pretty;

use crate::common::decoder::Decoder;

/// Extensions used by CEO/ESF PackedFiles.
pub const EXTENSIONS: [&str; 2] = [".ccd", ".esf"];

/// Signatured/Magic Numbers/Whatever of a ESF PackedFile.
pub const SIGNATURE_CAAB: &[u8; 4] = &[0xCA, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CEAB: &[u8; 4] = &[0xCE, 0xAB, 0x00, 0x00];
pub const SIGNATURE_CFAB: &[u8; 4] = &[0xCF, 0xAB, 0x00, 0x00];

pub mod caab;

//---------------------------------------------------------------------------//
//                              Markers, from ESFEdit
//---------------------------------------------------------------------------//

/// Invalid marker.
pub const INVALID: u8 = 0x00;

/// Primitives
pub const BOOL: u8 = 0x01;
pub const INT8: u8 = 0x02;
pub const INT16: u8 = 0x03;
pub const INT32: u8 = 0x04;
pub const INT64: u8 = 0x05;
pub const UINT8: u8 = 0x06;
pub const UINT16: u8 = 0x07;
pub const UINT32: u8 = 0x08;
pub const UINT64: u8 = 0x09;
pub const SINGLE: u8 = 0x0a;
pub const DOUBLE: u8 = 0x0b;
pub const COORD2D: u8 = 0x0c;
pub const COORD3D: u8 = 0x0d;
pub const UTF16: u8 = 0x0e;
pub const ASCII: u8 = 0x0f;
pub const ANGLE: u8 = 0x10;

/// If not set, record info is encodec in 2 bytes
pub const LONG_INFO: u8 = 0x20;

// RoninX: TW Warhammer, ASCII?
pub const ASCII_W21: u8 = 0x21;
pub const ASCII_W25: u8 = 0x25;
pub const UNKNOWN_23: u8 = 0x23;
pub const UNKNOWN_24: u8 = 0x24;

/// Three Kingdoms DLC Eight Princes types
pub const UNKNOWN_26: u8 = 0x26;

/// if set, this is a array of records
pub const BLOCK_BIT: u8 = 0x40;

/// Arrays
pub const BOOL_ARRAY: u8 = 0x41;
pub const INT8_ARRAY: u8 = 0x42;
pub const INT16_ARRAY: u8 = 0x43;
pub const INT32_ARRAY: u8 = 0x44;
pub const INT64_ARRAY: u8 = 0x45;
pub const UINT8_ARRAY: u8 = 0x46;
pub const UINT16_ARRAY: u8 = 0x47;
pub const UINT32_ARRAY: u8 = 0x48;
pub const UINT64_ARRAY: u8 = 0x49;
pub const SINGLE_ARRAY: u8 = 0x4a;
pub const DOUBLE_ARRAY: u8 = 0x4b;
pub const COORD2D_ARRAY: u8 = 0x4c;
pub const COORD3D_ARRAY: u8 = 0x4d;
pub const UTF16_ARRAY: u8 = 0x4e;
pub const ASCII_ARRAY: u8 = 0x4f;
pub const ANGLE_ARRAY: u8 = 0x50;

/// Records and Blocks
pub const RECORD: u8 = 0x80;
pub const RECORD_BLOCK: u8 = 0x81;

// This one is used in EditSF, but here we don't use it.
//pub const RECORD_BLOCK_ENTRY: u8 = -1;

/// Optimized Primitives
pub const BOOL_TRUE: u8 = 0x12;
pub const BOOL_FALSE: u8 = 0x13;
pub const UINT32_ZERO: u8 = 0x14;
pub const UINT32_ONE: u8 = 0x15;
pub const UINT32_BYTE: u8 = 0x16;
pub const UINT32_SHORT: u8 = 0x17;
pub const UINT32_24BIT: u8 = 0x18;
pub const INT32_ZERO: u8 = 0x19;
pub const INT32_BYTE: u8 = 0x1a;
pub const INT32_SHORT: u8 = 0x1b;
pub const INT32_24BIT: u8 = 0x1c;
pub const SINGLE_ZERO: u8 = 0x1d;

/// Optimized Arrays
pub const BOOL_TRUE_ARRAY: u8 = 0x52; // makes no sense
pub const BOOL_FALSE_ARRAY: u8 = 0x53; // makes no sense
pub const UINT_ZERO_ARRAY: u8 = 0x54; // makes no sense
pub const UINT_ONE_ARRAY: u8 = 0x55; // makes no sense
pub const UINT32_BYTE_ARRAY: u8 = 0x56;
pub const UINT32_SHORT_ARRAY: u8 = 0x57;
pub const UINT32_24BIT_ARRAY: u8 = 0x58;
pub const INT32_ZERO_ARRAY: u8 = 0x59; // makes no sense
pub const INT32_BYTE_ARRAY: u8 = 0x5a;
pub const INT32_SHORT_ARRAY: u8 = 0x5b;
pub const INT32_24BIT_ARRAY: u8 = 0x5c;
pub const SINGLE_ZERO_ARRAY: u8 = 0x5d;  // makes no sense

pub const LONG_RECORD: u8 = 0xa0;
pub const LONG_RECORD_BLOCK: u8 = 0xe0;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire ESF PackedFile decoded in memory.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ESF {
    signature: ESFSignature,
    unknown_1: u32,
    creation_date: u32,
    root_node: NodeType,
    unknown_2: u32,
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
/// NOTE: These are extracted from EditSF.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    Invalid,
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Single(f32),
    Double(f64),
    Coord2d(Coordinates2DNode),
    Coord3d(Coordinates3DNode),
    Utf16(String),
    Ascii(String),
    //Angle(bool),
    AsciiW21(String),
    AsciiW25(String),
    Unknown23(u8),
    //Unknown_24(bool),
    //Unknown_26(bool),
    BoolArray(Vec<bool>),
    Int8Array(Vec<i8>),
    Int16Array(Vec<i16>),
    Int32Array(Vec<i32>),
    Int64Array(Vec<i64>),
    Uint8Array(Vec<u8>),
    Uint16Array(Vec<u16>),
    Uint32Array(Vec<u32>),
    Uint64Array(Vec<u64>),
    SingleArray(Vec<f32>),
    //DoubleArray(Vec<f64>),
    Coord2dArray(Vec<Coordinates2DNode>),
    Coord3dArray(Vec<Coordinates3DNode>),
    Utf16Array(Vec<String>),
    AsciiArray(Vec<String>),
    //Angle_array(bool),
    Record(RecordNode),
    RecordBlock(RecordBlockNode),
    BoolTrue(bool),
    BoolFalse(bool),
    Uint32Zero(u32),
    Uint32One(u32),
    Uint32Byte(u32),
    Uint32Short(u32),
    Uint32_24bit(u32),
    Int32Zero(i32),
    Int32Byte(i32),
    Int32Short(i32),
    Int32_24bit(i32),
    SingleZero(f32),
    //Bool_true_array(bool),
    //Bool_false_array(bool),
    //Uint_zero_array(bool),
    //Uint_one_array(bool),
    //Uint32_byte_array(bool),
    //Uint32_short_array(bool),
    //Uint32_24bit_array(bool),
    //Int32_zero_array(bool),
    //Int32_byte_array(bool),
    //Int32_short_array(bool),
    //Int32_24bit_array(bool),
    //Single_zero_array(bool),
    //Long_record(bool),
    //Long_record_block(bool),
}

/// TODO: confirm what each number is.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Coordinates2DNode {
    x: f32,
    y: f32,
}

/// TODO: confirm what each number is.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Coordinates3DNode {
    x: f32,
    y: f32,
    z: f32,
}

/// TODO: confirm what each number is.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecordNode {
    version: u8,
    name: String,
    offset_len: u32,
    children: Vec<NodeType>
}

/// TODO: confirm what each number is.
#[derive(GetRef, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecordBlockNode {
    version: u8,
    name: String,
    offset_len: u32,
    offset_len_2: u32,
    children: Vec<(u32, Vec<NodeType>)>
}

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

/// Implementation of `ESF`.
impl ESF {

    /// This function returns if the provided data corresponds to a ESF or not.
    pub fn is_esf(data: &[u8]) -> bool {
        match data.get_bytes_checked(0, 4) {
            Ok(signature) => signature == SIGNATURE_CAAB,
            Err(_) => false,
        }
    }

    /// This function creates a `ESF` from a `Vec<u8>`.
    pub fn read(packed_file_data: &[u8]) -> Result<Self> {

        let signature_bytes = packed_file_data.get_bytes_checked(0, 4)?;

        let signature = if signature_bytes == SIGNATURE_CAAB { ESFSignature::CAAB }
        else if signature_bytes == SIGNATURE_CEAB { ESFSignature::CEAB }
        else if signature_bytes == SIGNATURE_CFAB { ESFSignature::CFAB }
        else { return Err(ErrorKind::ESFUnsupportedSignature(format!("{:#X}{:#X}", signature_bytes[0], signature_bytes[1])).into()) };

        let esf = match signature {
            ESFSignature::CAAB => Self::read_caab(packed_file_data)?,
            _ => return  Err(ErrorKind::ESFUnsupportedSignature(format!("{:#X}{:#X}", signature_bytes[0], signature_bytes[1])).into())
        };

        //use std::io::Write;
        //let mut x = std::fs::File::create("ceo.json")?;
        //x.write_all(&serde_json::to_string_pretty(&esf).unwrap().as_bytes())?;

        Ok(esf)
    }

    /// This function takes a `ESF` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {
        match self.signature {
            ESFSignature::CAAB => self.save_caab(),
            _ => return vec![],
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
