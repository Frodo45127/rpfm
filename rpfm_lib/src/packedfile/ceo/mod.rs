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
Module with all the code to interact with CEO PackedFiles.

This is a esf-like format used for ancillaries on 3k.
!*/

use chrono::NaiveDateTime;
use rayon::vec;
use serde_derive::{Serialize, Deserialize};

use rpfm_error::{ErrorKind, Result};

use crate::common::decoder::Decoder;

/// Extensions used by CEO PackedFiles.
pub const EXTENSION: &str = ".ccd";

/// Signature/Magic Numbers/Whatever of a CEO PackedFile.
pub const SIGNATURE: &[u8; 4] = &[0xAC, 0xBA, 0x00, 0x00];

//---------------------------------------------------------------------------//
//                              Markers, from ESFEdit
//---------------------------------------------------------------------------//

// marker type
pub const INVALID: u8 = 0x00;

// Primitives
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

// if not set, record info is encodec in 2 bytes
pub const LONG_INFO: u8 = 0x20;

// RoninX: TW Warhammer, ASCII?
pub const ASCII_W21: u8 = 0x21;
pub const ASCII_W25: u8 = 0x25;
pub const UNKNOWN_23: u8 = 0x23;
pub const UNKNOWN_24: u8 = 0x24;

// Three Kingdoms DLC Eight Princes types
pub const UNKNOWN_26: u8 = 0x26;

// if set, this is a array of records
pub const BLOCK_BIT: u8 = 0x40;

// Arrays
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

// Records and Blocks
pub const RECORD: u8 = 0x80;
pub const RECORD_BLOCK: u8 = 0x81;
pub const RECORD_BLOCK_ENTRY: u8 = 0x82;

// Optimized Primitives
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

// Optimized Arrays
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

/// This holds an entire CEO PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct CEO {
    signature: u32,
    n2: u32,
    date: u32,
    node_offset: u32,
    node_names: Vec<String>,
    node_2_names: Vec<(String, u32)>,
}

/// Node types supported by the CEO.
///
/// NOTE: For now we're using the same name types as used in ESFEdit. Later on we'll change to a more rusted ones.
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
    //Ascii_w21(bool),
    //Ascii_w25(bool),
    //Unknown_23(bool),
    //Unknown_24(bool),
    //Unknown_26(bool),
    Bool_array(Vec<bool>),
    Int8_array(Vec<i8>),
    Int16_array(Vec<i16>),
    Int32_array(Vec<i32>),
    Int64_array(Vec<i64>),
    Uint8_array(Vec<u8>),
    Uint16_array(Vec<u16>),
    Uint32_array(Vec<u32>),
    Uint64_array(Vec<u64>),
    Single_array(Vec<f32>),
    Double_array(Vec<f64>),
    Coord2d_array(Vec<Coordinates2DNode>),
    Coord3d_array(Vec<Coordinates3DNode>),
    Utf16_array(Vec<String>),
    Ascii_array(Vec<String>),
    //Angle_array(bool),
    Record(RecordNode),
    Record_block(RecordNode),
    //Bool_true(bool),
    //Bool_false(bool),
    //Uint32_zero(bool),
    //Uint32_one(bool),
    //Uint32_byte(bool),
    //Uint32_short(bool),
    //Uint32_24bit(bool),
    Int32_zero(i32),
    Int32_byte(i32),
    Int32_short(i32),
    Int32_24bit(i32),
    //Single_zero(bool),
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
    name_index: u16,
    childs: Vec<NodeType>
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct OptimizedIntNode {
    data: u64
}

//---------------------------------------------------------------------------//
//                           Implementation of CEO
//---------------------------------------------------------------------------//

/// Implementation of `CEO`.
impl CEO {

    /*
    /// This function returns if the provided data corresponds to a video or not.
    pub fn is_ceo(data: &[u8]) -> bool {
        match data.decode_string_u8(0, 4) {
            Ok(signature) => signature == SIGNATURE_IVF || signature == SIGNATURE_CAMV,
            Err(_) => false,
        }
    }*/

    /// This function creates a `CEO` from a `Vec<u8>`.
    ///
    /// NOTE: this takes a whole vector, not a reference. The reason is this vector can by enormous and this way
    /// we can avoid duplicates.
    pub fn read(packed_file_data: Vec<u8>) -> Result<Self> {
        let mut offset = 0;


        // TODO: compare with signature.
        let signature = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        dbg!(signature);
        let n2 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        dbg!(n2);
        let date = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let n3_pa = NaiveDateTime::from_timestamp(date as i64, 0);
        let node_offset = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        dbg!(date);
        dbg!(n3_pa);


        let node = Self::read_node(&packed_file_data, &mut offset)?;


        dbg!(offset);
        let node_count = packed_file_data.decode_packedfile_integer_u16(offset, &mut offset)?;
        dbg!(node_count);
        let mut node_names = vec![];
        for _node_index in 0..node_count {
            node_names.push(packed_file_data.decode_packedfile_string_u8(offset, &mut offset)?);

        }
        dbg!(&node_names);

        let _uknown1 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let node_2_count = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let mut node_2_names = vec![];
        for _node_index in 0..node_2_count {
            let name = packed_file_data.decode_packedfile_string_u8(offset, &mut offset)?;
            let uknown2 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            node_2_names.push((name, uknown2));
        }

        node_2_names.sort_by(|x, y| x.1.cmp(&y.1));
        //dbg!(&node_2_names);

        Ok(Self{
            signature,
            n2,
            date,
            node_offset,
            node_names,
            node_2_names,
        })
    }

    /// This function takes a `CEO` and encodes it to `Vec<u8>`.
    pub fn save(&self) -> Vec<u8> {
        vec![]
    }

    /// This function takes care of reading a node's data into the appropiate NodeType.
    fn read_node(packed_file_data: &[u8], mut offset: &mut usize) -> Result<NodeType> {
        let next_byte = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
        dbg!(next_byte);
        dbg!(*offset);

        println!("{:#X}", next_byte);

        match next_byte {
            INVALID => Err(ErrorKind::CEOUnsupportedDataType(format!("{}", next_byte)).into()),
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
            ASCII => Ok(NodeType::Ascii(packed_file_data.decode_packedfile_string_u8(*offset, &mut offset)?)),
            //ANGLE =>{},
            //ASCII_W21 =>{},
            //ASCII_W25 =>{},
            //UNKNOWN_23 =>{},
            //UNKNOWN_24 =>{},
            //UNKNOWN_26 =>{},
            BOOL_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?)
                }

                Ok(NodeType::Bool_array(node_data))
            },
            INT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?)
                }
                Ok(NodeType::Int8_array(node_data))

            },
            INT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)
                }
                Ok(NodeType::Int16_array(node_data))

            },
            INT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?)
                }

                Ok(NodeType::Int32_array(node_data))

            },
            INT64_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?)
                }

                Ok(NodeType::Int64_array(node_data))

            },
            UINT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint8_array(node_data))

            },
            UINT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint16_array(node_data))

            },
            UINT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint32_array(node_data))

            },
            UINT64_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?)
                }

                Ok(NodeType::Uint64_array(node_data))

            },
            SINGLE_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?)
                }

                Ok(NodeType::Single_array(node_data))
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

                Ok(NodeType::Coord2d_array(node_data))

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

                Ok(NodeType::Coord3d_array(node_data))

            },
            UTF16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_string_u16(*offset, &mut offset)?)
                }

                Ok(NodeType::Utf16_array(node_data))
            },
            ASCII_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_string_u8(*offset, &mut offset)?)
                }
                Ok(NodeType::Ascii_array(node_data))

            },
            //ANGLE_ARRAY =>{},
            RECORD => {

                let name_index = packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?;
                let version = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;

                // Explanation for this weird thing:
                // - The end offset starts after the first 0x80 byte.
                // - We have to "sum" all bytes from that until the next 0x80 byte.
                let mut end_offset: usize = 0;
                while(packed_file_data[*offset] & 0x80) != 0 {
                    end_offset = (end_offset << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                    *offset += 1;
                }

                end_offset = (end_offset << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                *offset += 1;

                //let data = packed_file_data[*offset..end_offset as usize].to_vec();

                let mut childs: Vec<NodeType> = vec![];
                //while *offset < end_offset {
                //    childs.push(Self::read_node(&packed_file_data[..end_offset], &mut offset)?);
                //}

                *offset += end_offset as usize;

                let node_data = RecordNode {
                    version,
                    name_index,
                    childs
                };


                Ok(NodeType::Record(node_data))
            },

            // Incomplete
            RECORD_BLOCK | 130 => {
                let name_index = packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?;
                let version = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;

                // Explanation for this weird thing:
                // - The end offset starts after the first 0x80 byte.
                // - We have to "sum" all bytes from that until the next 0x80 byte.
                let mut end_offset: usize = 0;
                while(packed_file_data[*offset] & 0x80) != 0 {
                    end_offset = (end_offset << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                    *offset += 1;
                }

                end_offset = (end_offset << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                *offset += 1;


                let mut count: usize = 0;
                while(packed_file_data[*offset] & 0x80) != 0 {
                    count = (count << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                    *offset += 1;
                }

                count = (count << 7) + (packed_file_data[*offset] & 0x7f) as usize;
                *offset += 1;


                //let data = packed_file_data[*offset..end_offset as usize].to_vec();

                let mut childs: Vec<NodeType> = vec![];
                for _ in 0..count {
                    dbg!(&offset);
                    childs.push(Self::read_node(&packed_file_data[..end_offset], &mut offset)?);
                }

                //*offset += end_offset as usize;


                let node_data = RecordNode {
                    version,
                    name_index,
                    childs
                };


                Ok(NodeType::Record_block(node_data))
            },
            //RECORD_BLOCK_ENTRIES => {
            //
            //},
            //BOOL_TRUE =>{},
            //BOOL_FALSE =>{},
            //UINT32_ZERO =>{},
            //UINT32_ONE =>{},
            //UINT32_BYTE =>{},
            //UINT32_SHORT =>{},
            //UINT32_24BIT =>{},
            INT32_ZERO =>{
                Ok(NodeType::Int32_zero(0))
            },
            INT32_BYTE =>{
                Ok(NodeType::Int32_byte(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as i32))
            },
            INT32_SHORT =>{
                Ok(NodeType::Int32_short(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as i32))
            },

            // Dummy until I fix it.
            INT32_24BIT =>{
                *offset += 1;
                Ok(NodeType::Int32_24bit(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as i32))

            },
            //SINGLE_ZERO =>{},
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
            _ => Err(ErrorKind::CEOUnsupportedDataType(format!("{}", next_byte)).into()),
        }
    }
}

