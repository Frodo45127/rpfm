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

use std::{collections::BTreeMap, vec};

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};

/// Extensions used by CEO PackedFiles.
pub const EXTENSION: &str = ".ccd";

/// Signature/Magic Numbers/Whatever of a ESF PackedFile.
pub const SIGNATURE_CAAB: &[u8; 4] = &[0xCA, 0xAB, 0x00, 0x00];

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
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ESF {
    signature: ESFSignature,
    unknown_1: u32,
    creation_date: u32,
    root_node: NodeType,
    record_names: Vec<String>,
    unknown_2: u32,
    strings: BTreeMap<u32, String>,
}

/// This enum contains the different signatures of ESF files.
#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ESFSignature {

    /// Signature found on 3K files.
    CAAB,
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
    DoubleArray(Vec<f64>),
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
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Coordinates2DNode {
    x: f32,
    y: f32,
}

/// TODO: confirm what each number is.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Coordinates3DNode {
    x: f32,
    y: f32,
    z: f32,
}

/// TODO: confirm what each number is.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecordNode {
    version: u8,
    name: String,
    childs: Vec<NodeType>
}

/// TODO: confirm what each number is.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RecordBlockNode {
    version: u8,
    name: String,
    childs: Vec<Vec<NodeType>>
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
        else { return Err(ErrorKind::ESFUnsupportedSignature(format!("{:#X}{:#X}", signature_bytes[0], signature_bytes[1])).into()) };

        let mut offset = 4;
        let unknown_1 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let creation_date = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let record_names_offset = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let nodes_offset = offset;

        // We need this data decoded first, because some nodes reference to it, and we can use that to populate the nodes.
        offset = record_names_offset as usize;

        // Get the name list for the record/record block entries.
        let record_names_count = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        let mut record_names = vec![];
        for _ in 0..record_names_count {
            record_names.push(packed_file_data.decode_packedfile_string_u8(offset, &mut offset)?);
        }

        // Get the strings for all the subnodes.
        let unknown_2 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let strings_count = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let mut strings = BTreeMap::new();
        for _ in 0..strings_count {
            let name = packed_file_data.decode_packedfile_string_u8(offset, &mut offset)?;
            let index = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            strings.insert(index, name);
        }

        // If we're not at the end of the file, something failed.
        if offset != packed_file_data.len() {
            return Err(ErrorKind::ESFIncompleteDecoding.into());
        }

        // Restore the index before continuing.
        offset = nodes_offset;

        // This file is a big tree hanging from the root node, so just decode everything recursively.
        let root_node = Self::read_node(&packed_file_data, &mut offset, true, &record_names, &strings)?;

        // If we're not at the exact end of the nodes, something failed.
        if offset != record_names_offset as usize {
            return Err(ErrorKind::ESFIncompleteDecoding.into());
        }

        let esf = Self{
            signature,
            unknown_1,
            creation_date,
            root_node,
            record_names,
            unknown_2,
            strings,
        };

        use std::io::Write;
        let mut x = std::fs::File::create("encoded_ccd.ccd")?;
        x.write_all(&esf.save())?;

        Ok(esf)
    }

    /// This function takes a `ESF` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {
        let mut data = vec![];
        match self.signature {
            ESFSignature::CAAB => data.extend_from_slice(SIGNATURE_CAAB),
        }

        data.encode_integer_u32(self.unknown_1);
        data.encode_integer_u32(self.creation_date);

        // First, get the strings encoded, as we need to have them in order before encoding the nodes.
        let mut strings_data = self.save_strings();

        // Next, encode the nodes. We need them (and the strings) encoded in order to know their offsets.
        let mut string_count: u32 = 0;
        let mut record_names_count: u32 = 0;
        let mut nodes_data = Self::save_node(&self.root_node, true, &mut string_count, &mut record_names_count);

        // Then, merge everything.
        data.encode_integer_u32(strings_data.len() as u32);
        data.append(&mut nodes_data);
        data.encode_integer_u32(self.unknown_2);
        data.append(&mut strings_data);
        data
    }

    /// This function takes care of reading a node's data into the appropiate NodeType.
    fn read_node(
        packed_file_data: &[u8],
        mut offset: &mut usize,
        is_root_node: bool,
        record_names: &[String],
        strings: &BTreeMap<u32, String>
    ) -> Result<NodeType> {

        let next_byte = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
        let has_record_bit = next_byte & RECORD;
        let has_block_bit = next_byte & BLOCK_BIT;

        match next_byte {
            INVALID => Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),
            BOOL => Ok(NodeType::Bool(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?)),
            INT8 => Ok(NodeType::Int8(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?)),
            INT16 => Ok(NodeType::Int16(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)),
            INT32 => Ok(NodeType::Int32(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?)),
            INT64 => Ok(NodeType::Int64(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?)),
            UINT8 => Ok(NodeType::Uint8(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)),
            UINT16 => Ok(NodeType::Uint16(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?)),
            UINT32 => Ok(NodeType::Uint32(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?)),
            UINT64 => Ok(NodeType::Uint64(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?)),
            SINGLE => Ok(NodeType::Single(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?)),
            //DOUBLE => Ok(NodeType::Double(packed_file_data.decode_packedfile_float_f64(*offset, &mut offset)?)),
            COORD2D =>{
                let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                Ok(NodeType::Coord2d(Coordinates2DNode{
                    x,
                    y
                }))
            },
            COORD3D =>{
                let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let z = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                Ok(NodeType::Coord3d(Coordinates3DNode{
                    x,
                    y,
                    z
                }))
            },
            UTF16 => Ok(NodeType::Utf16(packed_file_data.decode_packedfile_string_u16(*offset, &mut offset)?)),
            ASCII => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => Ok(NodeType::Ascii(string.to_owned())),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            //ANGLE =>{},
            ASCII_W21 => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => Ok(NodeType::AsciiW21(string.to_owned())),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            ASCII_W25 => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => Ok(NodeType::AsciiW25(string.to_owned())),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            UNKNOWN_23 => Ok(NodeType::Unknown23(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)),
            //UNKNOWN_24 =>{},
            //UNKNOWN_26 =>{},
            BOOL_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?)
                }

                Ok(NodeType::BoolArray(node_data))
            },
            INT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?)
                }
                Ok(NodeType::Int8Array(node_data))

            },
            INT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)
                }
                Ok(NodeType::Int16Array(node_data))

            },
            INT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?)
                }

                Ok(NodeType::Int32Array(node_data))

            },
            INT64_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?)
                }

                Ok(NodeType::Int64Array(node_data))

            },
            UINT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint8Array(node_data))

            },
            UINT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint16Array(node_data))

            },
            UINT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint32Array(node_data))

            },
            UINT64_ARRAY => {
                let mut node_data = vec![];
                let size = Self::decode_size_caab(packed_file_data, offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint64Array(node_data))

            },
            SINGLE_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?)
                }

                Ok(NodeType::SingleArray(node_data))
            },
            /*DOUBLE_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_float_f64(*offset, &mut offset)?)
                }

                Ok(NodeType::Double_array(node_data))
            },*/
            COORD2D_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                    let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                    node_data.push(Coordinates2DNode{
                        x,
                        y
                    });
                }

                Ok(NodeType::Coord2dArray(node_data))

            },
            COORD3D_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                    let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                    let z = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                    node_data.push(Coordinates3DNode{
                        x,
                        y,
                        z
                    });
                }

                Ok(NodeType::Coord3dArray(node_data))

            },
            UTF16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_string_u16(*offset, &mut offset)?)
                }

                Ok(NodeType::Utf16Array(node_data))
            },
            ASCII_ARRAY => {
                let mut node_data = vec![];
                let size = Self::decode_size_caab(packed_file_data, offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                    match strings.get(&string_index) {
                        Some(string) => node_data.push(string.to_owned()),
                        None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                    }
                }
                Ok(NodeType::AsciiArray(node_data))

            },
            //ANGLE_ARRAY =>{},
            RECORD | _ if has_record_bit != 0 && has_block_bit == 0 => {

                let name_index;
                let version;

                // Root node is the only one that has data in full bytes.
                if is_root_node {
                    name_index = packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?;
                    version = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
                }

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                else {
                    version = (next_byte & 31) >> 1;
                    name_index = (((next_byte & 1) as u16) << 8) + packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u16;
                }

                let name = match record_names.get(name_index as usize) {
                    Some(name) => name.to_owned(),
                    None => return Err(ErrorKind::ESFRecordNameNotFound(name_index as u32).into())
                };

                // Explanation for this weird thing:
                // - The end offset starts after the first 0x80 byte.
                // - We have to "sum" all bytes from that until the next 0x80 byte.
                let mut end_offset = Self::decode_size_caab(packed_file_data, offset)?;
                end_offset += *offset;

                //let data = packed_file_data[*offset..end_offset as usize].to_vec();

                let mut childs: Vec<NodeType> = vec![];
                while *offset < end_offset {
                    childs.push(Self::read_node(&packed_file_data[..end_offset], &mut offset, false, record_names, strings)?);
                }

                let node_data = RecordNode {
                    version,
                    name,
                    childs
                };

                Ok(NodeType::Record(node_data))
            },

            // Incomplete
            RECORD_BLOCK | _ if has_record_bit != 0 && has_block_bit != 0 => {
                let name_index;
                let version;

                // Root node is the only one that has data in full bytes.
                if is_root_node {
                    name_index = packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?;
                    version = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
                }

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                else {
                    version = (next_byte & 31) >> 1;
                    name_index = (((next_byte & 1) as u16) << 8) + packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u16;
                }

                let name = match record_names.get(name_index as usize) {
                    Some(name) => name.to_owned(),
                    None => return Err(ErrorKind::ESFRecordNameNotFound(name_index as u32).into())
                };

                // Explanation for this weird thing:
                // - The end offset starts after the first 0x80 byte.
                // - We have to "sum" all bytes from that until the next 0x80 byte.
                let _end_offset = Self::decode_size_caab(packed_file_data, offset)?;
                let count = Self::decode_size_caab(packed_file_data, offset)?;

                let mut childs: Vec<Vec<NodeType>> = vec![];
                for _ in 0..count {
                    let size = Self::decode_size_caab(packed_file_data, offset)?;

                    let end_offset_2 = *offset + size;
                    let mut node_list = vec![];

                    while *offset < end_offset_2 {
                        node_list.push(Self::read_node(&packed_file_data, &mut offset, false, record_names, strings)?);
                    }
                    childs.push(node_list);
                }

                let node_data = RecordBlockNode {
                    version,
                    name,
                    childs
                };

                Ok(NodeType::RecordBlock(node_data))
            },
            BOOL_TRUE => Ok(NodeType::BoolTrue(true)),
            BOOL_FALSE => Ok(NodeType::BoolFalse(false)),
            UINT32_ZERO => {
                Ok(NodeType::Uint32Zero(0))
            },
            UINT32_ONE => {
                Ok(NodeType::Uint32Zero(1))
            },
            UINT32_BYTE => {
                Ok(NodeType::Uint32Byte(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u32))

            },
            UINT32_SHORT => {
                Ok(NodeType::Uint32Short(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as u32))

            },
            // Dummy until I fix it.
            UINT32_24BIT => {
                *offset += 1;
                Ok(NodeType::Uint32_24bit(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as u32))
            },
            INT32_ZERO =>{
                Ok(NodeType::Int32Zero(0))
            },
            INT32_BYTE =>{
                Ok(NodeType::Int32Byte(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)? as i32))
            },
            INT32_SHORT =>{
                Ok(NodeType::Int32Short(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)? as i32))
            },

            // Dummy until I fix it.
            INT32_24BIT =>{
                *offset += 1;
                Ok(NodeType::Int32_24bit(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)? as i32))

            },
            SINGLE_ZERO => {
                Ok(NodeType::SingleZero(0.0f32))

            },
            //BOOL_TRUE_ARRAY =>{},
            //BOOL_FALSE_ARRAY =>{},
            //UINT_ZERO_ARRAY =>{},
            //UINT_ONE_ARRAY =>{},
            //UINT32_BYTE_ARRAY =>{},
            //UINT32_SHORT_ARRAY =>{},
            //UINT32_24BIT_ARRAY =>{},
            //INT32_ZERO_ARRAY =>{},
            //INT32_BYTE_ARRAY =>{},
            //INT32_SHORT_ARRAY =>{},
            //INT32_24BIT_ARRAY =>{},
            //SINGLE_ZERO_ARRAY =>{},
            //LONG_RECORD =>{},
            //LONG_RECORD_BLOCK =>{},

            // Anything else is not yet supported.
            _ => Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),
        }
    }

    /// This function takes care of reading a node's data into the appropiate NodeType.
    fn save_node(node_type: &NodeType, is_root_node: bool, string_count: &mut u32, record_names_count: &mut u32) -> Vec<u8> {
        let mut data = vec![];
        match node_type {
            NodeType::Invalid => unimplemented!(),
            NodeType::Bool(value) => {
                data.push(BOOL);
                data.encode_bool(*value);
            },
            NodeType::Int8(value) => {
                data.push(INT8);
                data.encode_integer_i8(*value);
            },
            NodeType::Int16(value) => {
                data.push(INT16);
                data.encode_integer_i16(*value);
            },
            NodeType::Int32(value) => {
                data.push(INT32);
                data.encode_integer_i32(*value);
            },
            NodeType::Int64(value) => {
                data.push(INT64);
                data.encode_integer_i64(*value);
            },
            NodeType::Uint8(value) => {
                data.push(UINT8);
                data.push(*value);
            },
            NodeType::Uint16(value) => {
                data.push(UINT16);
                data.encode_integer_u16(*value);
            },
            NodeType::Uint32(value) => {
                data.push(UINT32);
                data.encode_integer_u32(*value);
            },
            NodeType::Uint64(value) => {
                data.push(UINT64);
                data.encode_integer_u64(*value);
            },
            NodeType::Single(value) => {
                data.push(SINGLE);
                data.encode_float_f32(*value);
            },

            // Fix
            NodeType::Double(_value) => {
                data.push(DOUBLE);
                //data.encode_float_f32(*value);
            },
            NodeType::Coord2d(value) => {
                data.push(COORD2D);
                data.encode_float_f32(value.x);
                data.encode_float_f32(value.y);
            },
            NodeType::Coord3d(value) => {
                data.push(COORD3D);
                data.encode_float_f32(value.x);
                data.encode_float_f32(value.y);
                data.encode_float_f32(value.z);
            },
            NodeType::Utf16(_value) => {
                data.push(UTF16);
                data.encode_integer_u32(*string_count);
                *string_count += 1;
            },
            NodeType::Ascii(_value) => {
                data.push(ASCII);
                data.encode_integer_u32(*string_count);
                *string_count += 1;
            },
            //NodeType::Angle(bool),
            NodeType::AsciiW21(_value) => {
                data.push(ASCII_W21);
                data.encode_integer_u32(*string_count);
                *string_count += 1;
            },
            NodeType::AsciiW25(_value) => {
                data.push(ASCII_W25);
                data.encode_integer_u32(*string_count);
                *string_count += 1;
            },
            NodeType::Unknown23(value) => {
                data.push(UNKNOWN_23);
                data.push(*value);
            },
            //NodeType::Unknown_24(bool),
            //NodeType::Unknown_26(bool),
            NodeType::BoolArray(value) => {
                data.push(BOOL_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_bool(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Int8Array(value) => {
                data.push(INT8_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i8(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Int16Array(value) => {
                data.push(INT16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i16(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Int32Array(value) => {
                data.push(INT32_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i32(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Int64Array(value) => {
                data.push(INT64_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i64(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Uint8Array(value) => {
                data.push(UINT8_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.push(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Uint16Array(value) => {
                data.push(UINT16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u16(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Uint32Array(value) => {
                data.push(UINT32_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u32(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Uint64Array(value) => {
                data.push(UINT64_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u64(*x));
                data.extend_from_slice(&Self::encode_size_caab(list.len() as u32, UINT64_ARRAY));
                data.extend_from_slice(&list);
            },
            NodeType::SingleArray(value) => {
                data.push(SINGLE_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_float_f32(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::DoubleArray(_value) => {
                data.push(DOUBLE_ARRAY);
                /*
                let mut list = vec![];
                value.iter().for_each(|x| list.encode_bool(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
                */
            },
            NodeType::Coord2dArray(value) => {
                data.push(COORD2D_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| {
                    list.encode_float_f32(x.x);
                    list.encode_float_f32(x.y);
                });

                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Coord3dArray(value) => {
                data.push(COORD3D_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| {
                    list.encode_float_f32(x.x);
                    list.encode_float_f32(x.y);
                    list.encode_float_f32(x.z);
                });

                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Utf16Array(value) => {
                data.push(UTF16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|_| {
                    list.encode_integer_u32(*string_count);
                    *string_count += 1;
                });

                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::AsciiArray(value) => {
                data.push(ASCII_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|_| {
                    list.encode_integer_u32(*string_count);
                    *string_count += 1;
                });

                data.extend_from_slice(&Self::encode_size_caab(list.len() as u32, ASCII_ARRAY));
                data.extend_from_slice(&list);
            },
            //NodeType::Angle_array(bool),
            NodeType::Record(value) => {
                data.push(RECORD);

                // Root node is the only one that has data in full bytes.
                if is_root_node {
                    data.encode_integer_u16(*record_names_count as u16);
                    *record_names_count += 1;
                    data.push(value.version as u8);
                }

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                else {
                    data.push(*record_names_count as u8);
                    *record_names_count += 1;
                }

                let mut childs_data = vec![];
                for node in &value.childs {
                    childs_data.extend_from_slice(&Self::save_node(&node, false, string_count, record_names_count));
                }

                data.extend_from_slice(&Self::encode_size_caab(childs_data.len() as u32, RECORD));
                data.extend_from_slice(&childs_data);
            },
            NodeType::RecordBlock(value) => {
                data.push(RECORD_BLOCK);

                // Root node is the only one that has data in full bytes.
                if is_root_node {
                    data.encode_integer_u16(*record_names_count as u16);
                    *record_names_count += 1;
                    data.push(value.version as u8);
                }

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                else {
                    data.push(*record_names_count as u8);
                    *record_names_count += 1;
                }

                let mut childs_data = vec![];
                for group_node in &value.childs {
                    for node in group_node {

                        let child_node = Self::save_node(&node, false, string_count, record_names_count);
                        childs_data.extend_from_slice(&Self::encode_size_caab(child_node.len() as u32, RECORD_BLOCK));
                        childs_data.extend_from_slice(&child_node);
                    }
                }

                data.extend_from_slice(&Self::encode_size_caab(childs_data.len() as u32, RECORD_BLOCK));
                data.extend_from_slice(&Self::encode_size_caab(value.childs.len() as u32, RECORD_BLOCK));
                data.extend_from_slice(&childs_data);
            },
            NodeType::BoolTrue(_value) => {
                data.push(BOOL_TRUE);
            },
            NodeType::BoolFalse(_value) => {
                data.push(BOOL_FALSE);
            },
            NodeType::Uint32Zero(_value) => {
                data.push(UINT32_ZERO);
            },
            NodeType::Uint32One(_value) => {
                data.push(UINT32_ONE);
            },
            NodeType::Uint32Byte(value) => {
                data.push(UINT32_BYTE);
                data.push(*value as u8);
            },
            NodeType::Uint32Short(value) => {
                data.push(*value as u8);
                data.encode_integer_u16(*value as u16);
            },
            NodeType::Uint32_24bit(value) => {
                data.push(UINT32_24BIT);
                data.encode_integer_u16(*value as u16);
            },
            NodeType::Int32Zero(value) => {
                data.push(INT32_ZERO);
                data.encode_integer_i32(*value as i32);
            },
            NodeType::Int32Byte(value) => {
                data.push(INT32_BYTE);
                data.encode_integer_i8(*value as i8);
            },
            NodeType::Int32Short(value) => {
                data.push(INT32_SHORT);
                data.encode_integer_i16(*value as i16);
            },
            NodeType::Int32_24bit(value) => {
                data.push(INT32_24BIT);
                data.encode_integer_i32(*value);
            },
            NodeType::SingleZero(value) => {
                data.push(SINGLE_ZERO);
                data.encode_float_f32(*value);
            },
            //NodeType::Bool_true_array(bool),
            //NodeType::Bool_false_array(bool),
            //NodeType::Uint_zero_array(bool),
            //NodeType::Uint_one_array(bool),
            //NodeType::Uint32_byte_array(bool),
            //NodeType::Uint32_short_array(bool),
            //NodeType::Uint32_24bit_array(bool),
            //NodeType::Int32_zero_array(bool),
            //NodeType::Int32_byte_array(bool),
            //NodeType::Int32_short_array(bool),
            //NodeType::Int32_24bit_array(bool),
            //NodeType::Single_zero_array(bool),
            //NodeType::Long_record(bool),
            //NodeType::Long_record_block(bool),
        }

        data
    }

    /// This function takes care of encoding all the strings in the ESF.
    fn save_strings(&self) -> Vec<u8> {
        let mut record_names: Vec<String> = vec![];
        let mut strings: Vec<String> = vec![];

        Self::read_string_from_node(&self.root_node, &mut record_names, &mut strings);

        // Once we have the strings isolated, encode them.
        let mut data: Vec<u8> = vec![];
        data.encode_integer_u16(record_names.len() as u16);

        for name in record_names {
            data.encode_packedfile_string_u8(&name);
        }

        for (index, string) in strings.iter().enumerate() {
            data.encode_packedfile_string_u8(&string);
            data.encode_integer_u32(index as u32);
        }

        data
    }

    fn read_string_from_node(node_type: &NodeType, record_names: &mut Vec<String>, strings: &mut Vec<String>) {
        match node_type {
            NodeType::Utf16(value) => strings.push(value.to_owned()),
            NodeType::Ascii(value) => strings.push(value.to_owned()),
            NodeType::AsciiW21(value) => strings.push(value.to_owned()),
            NodeType::AsciiW25(value) => strings.push(value.to_owned()),
            NodeType::Utf16Array(value) => strings.extend_from_slice(&value),
            NodeType::AsciiArray(value) => strings.extend_from_slice(&value),
            NodeType::Record(value) => {
                record_names.push(value.name.to_owned());
                for node in &value.childs {
                    Self::read_string_from_node(&node, record_names, strings);
                }
            },
            NodeType::RecordBlock(value) => {
                record_names.push(value.name.to_owned());
                for node_group in &value.childs {
                    for node in node_group {
                        Self::read_string_from_node(&node, record_names, strings);
                    }
                }
            },

            // Skip any other node.
            _ => {}
        }
    }

    /// This function allows to decode the variable-lenght size values found in CAAB ESF files.
    ///
    /// The bitwise magic is extracted from EditESF. I still have to warm my head around it.
    fn decode_size_caab(packed_file_data: &[u8], offset: &mut usize) -> Result<usize> {
        let mut size: usize = 0;
        while(packed_file_data[*offset] & 0x80) != 0 {
            size = (size << 7) + (packed_file_data[*offset] & 0x7f) as usize;
            *offset += 1;
        }

        size = (size << 7) + (packed_file_data[*offset] & 0x7f) as usize;
        *offset += 1;
        Ok(size)
    }

    /// This function allows you to encode a variable-lenght size found in CAAB ESF files.
    fn encode_size_caab(size: u32, node_byte_type: u8) -> Vec<u8> {
        let mut data = vec![];
        let mut temp_data = vec![];

        // If it's 0, just push a 0 and forget.
        if size == 0 {
            data.push(0);
        }

        // Otherwise, time for fun encoding.
        let mut size = size;

        while size != 0 {
            temp_data.push((size & 0x7f) as u8);
            size = size >> 7;
        }

        if temp_data.len() > 1 {
            for _ in 0..temp_data.len() - 1 {
                data.push(node_byte_type);
            }
        }

        while !temp_data.is_empty() {
            match temp_data.pop() {
                Some(mut byte) => {
                    if !temp_data.is_empty() {
                        byte |= 0x80;
                    }
                    data.push(byte);
                }
                None => data.push(0),
            }
        }

        data
    }
}

