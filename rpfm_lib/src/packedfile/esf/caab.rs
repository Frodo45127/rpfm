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
Module with all the code needed to support the CAAB format for ESF files.
!*/

use std::collections::BTreeMap;

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

/// Implementation of `ESF`. Section of functions specific for the CAAB format.
impl ESF {

    /// This function creates a `ESF` of type CAAB from a `Vec<u8>`.
    pub(crate) fn read_caab(packed_file_data: &[u8]) -> Result<Self> {
        let signature = ESFSignature::CAAB;

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
            unknown_2,
        };

        use std::io::Write;
        let mut x = std::fs::File::create("encoded_ccd.ccd")?;
        x.write_all(&esf.save())?;

        Ok(esf)
    }

    /// This function takes a `ESF` of type CAAB and encodes it to `Vec<u8>`.
    pub(crate) fn save_caab(&self) -> Vec<u8> {
        let mut data = vec![];

        // Encode the header info, except the offsets, because those are calculated later.
        data.extend_from_slice(SIGNATURE_CAAB);
        data.encode_integer_u32(self.unknown_1);
        data.encode_integer_u32(self.creation_date);

        // First, get the strings encoded, as we need to have them in order before encoding the nodes.
        let mut record_names = vec![];
        let mut strings = vec![];
        Self::read_string_from_node(&self.root_node, &mut record_names, &mut strings);

        // Next, encode the nodes. We need them (and the strings) encoded in order to know their offsets.
        let mut nodes_data = Self::save_node(&self.root_node, true, &strings, &record_names);

        // Then, encode the strings.
        let mut strings_data: Vec<u8> = vec![];
        strings_data.encode_integer_u16(record_names.len() as u16);

        for name in record_names {
            strings_data.encode_packedfile_string_u8(&name);
        }

        strings_data.encode_integer_u32(self.unknown_2);
        strings_data.encode_integer_u32(strings.len() as u32);
        for (index, string) in strings.iter().enumerate() {
            strings_data.encode_packedfile_string_u8(&string);
            strings_data.encode_integer_u32(index as u32);
        }

        // And finally, merge everything.
        data.encode_integer_u32((data.len() + nodes_data.len() + 4) as u32);
        data.append(&mut nodes_data);
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

        let node_type = match next_byte {
            INVALID => return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),
            BOOL => NodeType::Bool(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?),
            INT8 => NodeType::Int8(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?),
            INT16 => NodeType::Int16(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?),
            INT32 => NodeType::Int32(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?),
            INT64 => NodeType::Int64(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?),
            UINT8 => NodeType::Uint8(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?),
            UINT16 => NodeType::Uint16(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?),
            UINT32 => NodeType::Uint32(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?),
            UINT64 => NodeType::Uint64(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?),
            SINGLE => NodeType::Single(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?),
            //DOUBLE => Ok(NodeType::Double(packed_file_data.decode_packedfile_float_f64(*offset, &mut offset)?)),
            COORD2D =>{
                let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                NodeType::Coord2d(Coordinates2DNode{
                    x,
                    y
                })
            },
            COORD3D =>{
                let x = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let y = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;
                let z = packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?;

                NodeType::Coord3d(Coordinates3DNode{
                    x,
                    y,
                    z
                })
            },
            UTF16 => NodeType::Utf16(packed_file_data.decode_packedfile_string_u16(*offset, &mut offset)?),
            ASCII => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => NodeType::Ascii(string.to_owned()),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            //ANGLE =>{},
            ASCII_W21 => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => NodeType::AsciiW21(string.to_owned()),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            ASCII_W25 => {
                let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                match strings.get(&string_index) {
                    Some(string) => NodeType::AsciiW25(string.to_owned()),
                    None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                }
            },
            UNKNOWN_23 => NodeType::Unknown23(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?),
            //UNKNOWN_24 =>{},
            //UNKNOWN_26 =>{},
            BOOL_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?)
                }

                NodeType::BoolArray(node_data)
            },
            INT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?)
                }
                NodeType::Int8Array(node_data)

            },
            INT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)
                }
                NodeType::Int16Array(node_data)

            },
            INT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?)
                }

                NodeType::Int32Array(node_data)

            },
            INT64_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?)
                }

                NodeType::Int64Array(node_data)

            },
            UINT8_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)
                }

                NodeType::Uint8Array(node_data)

            },
            UINT16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?)
                }

                NodeType::Uint16Array(node_data)

            },
            UINT32_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?)
                }

                NodeType::Uint32Array(node_data)

            },
            UINT64_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?)
                }

                NodeType::Uint64Array(node_data)

            },
            SINGLE_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?)
                }

                NodeType::SingleArray(node_data)
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

                NodeType::Coord2dArray(node_data)

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

                NodeType::Coord3dArray(node_data)

            },
            UTF16_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    node_data.push(packed_file_data.decode_packedfile_string_u16(*offset, &mut offset)?)
                }

                NodeType::Utf16Array(node_data)
            },
            ASCII_ARRAY => {
                let mut node_data = vec![];
                let size = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                let end_offset = *offset + size as usize;

                while *offset < end_offset {
                    let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                    match strings.get(&string_index) {
                        Some(string) => node_data.push(string.to_owned()),
                        None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                    }
                }
                NodeType::AsciiArray(node_data)

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
                let mut offset_len = *offset;
                let mut end_offset = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                end_offset += *offset as u32;
                offset_len = *offset - offset_len;
                //let data = packed_file_data[*offset..end_offset as usize].to_vec();

                let mut childs: Vec<NodeType> = vec![];
                while *offset < end_offset as usize {
                    childs.push(Self::read_node(&packed_file_data[..end_offset as usize], &mut offset, false, record_names, strings)?);
                }

                let node_data = RecordNode {
                    version,
                    name,
                    childs,
                    offset_len: offset_len as u32,
                };

                NodeType::Record(node_data)
            },

            // Incomplete
            RECORD_BLOCK | _ if has_record_bit != 0 && has_block_bit != 0 => {

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                let version = (next_byte & 31) >> 1;
                let name_index = (((next_byte & 1) as u16) << 8) + packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u16;

                let name = match record_names.get(name_index as usize) {
                    Some(name) => name.to_owned(),
                    None => return Err(ErrorKind::ESFRecordNameNotFound(name_index as u32).into())
                };

                // Explanation for this weird thing:
                // - The end offset starts after the first 0x80 byte.
                // - We have to "sum" all bytes from that until the next 0x80 byte.
                let mut offset_len = *offset;
                let mut end_offset = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                end_offset += *offset as u32;
                offset_len = *offset - offset_len;

                let mut offset_len_2 = *offset;
                let count = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                offset_len_2 = *offset - offset_len_2;

                let mut childs = vec![];
                for x in 0..count {
                    let mut offset_len_3 = *offset;
                    let size = packed_file_data.decode_packedfile_integer_uleb128(&mut offset)?;
                    offset_len_3 = *offset - offset_len_3;

                    let end_offset_2 = *offset + size as usize;
                    let mut node_list = vec![];

                    while *offset < end_offset_2 {
                        node_list.push(Self::read_node(&packed_file_data, &mut offset, false, record_names, strings)?);
                    }

                    childs.push((offset_len_3 as u32, node_list));
                }

                let node_data = RecordBlockNode {
                    version,
                    name,
                    childs,
                    offset_len: offset_len as u32,
                    offset_len_2: offset_len_2 as u32,
                };

                NodeType::RecordBlock(node_data)
            },
            BOOL_TRUE => NodeType::BoolTrue(true),
            BOOL_FALSE => NodeType::BoolFalse(false),
            UINT32_ZERO => {
                NodeType::Uint32Zero(0)
            },
            UINT32_ONE => {
                NodeType::Uint32One(1)
            },
            UINT32_BYTE => {
                NodeType::Uint32Byte(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u32)

            },
            UINT32_SHORT => {
                NodeType::Uint32Short(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as u32)

            },
            UINT32_24BIT => {
                NodeType::Uint32_24bit(packed_file_data.decode_packedfile_integer_u24(*offset, &mut offset)? as u32)
            },
            INT32_ZERO =>{
                NodeType::Int32Zero(0)
            },
            INT32_BYTE =>{
                NodeType::Int32Byte(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)? as i32)
            },
            INT32_SHORT =>{
                NodeType::Int32Short(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)? as i32)
            },

            INT32_24BIT =>{
                NodeType::Int32_24bit(packed_file_data.decode_packedfile_integer_i24(*offset, &mut offset)? as i32)

            },
            SINGLE_ZERO => {
                NodeType::SingleZero(0.0f32)

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
            _ => return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),
        };

        //if next_byte != 128 {
        //    let data = Self::save_node(&node_type, is_root_node, &strings.values().map(|x| x.to_owned()).collect::<Vec<String>>(), record_names);
        //    if data != packed_file_data[initial_offset..*offset] {
        //        dbg!(next_byte);
        //        dbg!(*offset);
        //        dbg!(data);
        //        dbg!(&packed_file_data[initial_offset..*offset]);
        //        //return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into());
        //    }
        //}

        Ok(node_type)
    }

    /// This function takes care of reading a node's data into the appropiate NodeType.
    fn save_node(node_type: &NodeType, is_root_node: bool, strings: &[String], record_names: &[String]) -> Vec<u8> {
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
            NodeType::Utf16(value) => {
                data.push(UTF16);
                data.encode_integer_u32(strings.iter().position(|x| x == value).unwrap() as u32);
            },
            NodeType::Ascii(value) => {
                data.push(ASCII);
                data.encode_integer_u32(strings.iter().position(|x| x == value).unwrap() as u32);
            },
            //NodeType::Angle(bool),
            NodeType::AsciiW21(value) => {
                data.push(ASCII_W21);
                data.encode_integer_u32(strings.iter().position(|x| x == value).unwrap() as u32);
            },
            NodeType::AsciiW25(value) => {
                data.push(ASCII_W25);
                data.encode_integer_u32(strings.iter().position(|x| x == value).unwrap() as u32);
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
                data.encode_integer_uleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::SingleArray(value) => {
                data.push(SINGLE_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_float_f32(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            /*
            NodeType::DoubleArray(_value) => {
                data.push(DOUBLE_ARRAY);
                /*
                let mut list = vec![];
                value.iter().for_each(|x| list.encode_bool(*x));
                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
                */
            },*/
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
                value.iter().for_each(|y| {
                    list.encode_integer_u32(strings.iter().position(|x| x == y).unwrap() as u32);
                });

                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::AsciiArray(value) => {
                data.push(ASCII_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|y| {
                    list.encode_integer_u32(strings.iter().position(|x| x == y).unwrap() as u32);
                });

                data.encode_integer_uleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            //NodeType::Angle_array(bool),
            NodeType::Record(value) => {

                // Root node is the only one that has data in full bytes.
                if is_root_node {
                    data.push(RECORD);
                    data.encode_integer_u16(record_names.iter().position(|x| x == &value.name).unwrap() as u16);
                    data.push(value.version as u8);
                }

                // If it's not the root node, the data is encoded in 2 bytes using bitwise.
                else {
                    let mut info: u16 = (value.version as u16) << 9;
                    info |= record_names.iter().position(|x| x == &value.name).unwrap() as u16;
                    info |= (RECORD as u16) << 8;

                    let byte = ((info >> 8) & 0xFF) as u8;

                    data.push(byte);
                    data.push(info as u8);
                }

                let mut childs_data = vec![];
                for node in &value.childs {
                    childs_data.extend_from_slice(&Self::save_node(&node, false, &strings, &record_names));
                }

                let mut a = vec![];
                a.encode_integer_uleb128(childs_data.len() as u32);
                for x in 0..value.offset_len - a.len() as u32 {
                    data.push(0x80);
                }
                data.encode_integer_uleb128(childs_data.len() as u32);
                data.extend_from_slice(&childs_data);
            },
            NodeType::RecordBlock(value) => {
                //data.push(RECORD_BLOCK);

                let mut info: u16 = (value.version as u16) << 9;
                info |= record_names.iter().position(|x| x == &value.name).unwrap() as u16;
                info |= (BLOCK_BIT as u16) << 8;
                info |= (RECORD as u16) << 8;

                let byte = ((info >> 8) & 0xFF) as u8;

                data.push(byte);
                data.push(info as u8);

                let mut childs_data = vec![];
                for (bytes, group_node) in &value.childs {
                    let mut group_node_data = vec![];
                    for node in group_node {
                        let child_node = Self::save_node(&node, false, strings, record_names);
                        group_node_data.extend_from_slice(&child_node);
                    }

                    let mut a = vec![];
                    a.encode_integer_uleb128(group_node_data.len() as u32);
                    for x in 0..bytes - a.len() as u32 {
                        childs_data.push(0x80);
                    }
                    childs_data.encode_integer_uleb128(group_node_data.len() as u32);
                    childs_data.extend_from_slice(&group_node_data);
                }

                let mut a = vec![];
                a.encode_integer_uleb128(childs_data.len() as u32);
                for x in 0..value.offset_len - a.len() as u32 {
                    data.push(0x80);
                }
                data.encode_integer_uleb128(childs_data.len() as u32);

                let mut a = vec![];
                a.encode_integer_uleb128(value.childs.len() as u32);
                for x in 0..value.offset_len_2 - a.len() as u32 {
                    data.push(0x80);
                }

                data.encode_integer_uleb128(value.childs.len() as u32);
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
                data.push(UINT32_SHORT);
                data.encode_integer_u16(*value as u16);
            },
            NodeType::Uint32_24bit(value) => {
                data.push(UINT32_24BIT);
                data.encode_integer_u24(*value);
            },
            NodeType::Int32Zero(_value) => {
                data.push(INT32_ZERO);
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
                data.encode_integer_i24(*value);
            },
            NodeType::SingleZero(_value) => {
                data.push(SINGLE_ZERO);
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

    //---------------------------------------------------------------------------//
    //                       Utility functions for CAAB
    //---------------------------------------------------------------------------//

    /// This function reads the strings from the provided node and all its children.
    ///
    /// This function is recursive: if you pass it the root node, it'll read all the strings in the ESF file.
    fn read_string_from_node(node_type: &NodeType, record_names: &mut Vec<String>, strings: &mut Vec<String>) {
        match node_type {
            NodeType::Utf16(value) => if !strings.contains(value) { strings.push(value.to_owned()) },
            NodeType::Ascii(value) => if !strings.contains(value) { strings.push(value.to_owned()) },
            NodeType::AsciiW21(value) => if !strings.contains(value) { strings.push(value.to_owned()) },
            NodeType::AsciiW25(value) => if !strings.contains(value) { strings.push(value.to_owned()) },
            NodeType::Utf16Array(value) => value.iter().for_each(|value| if !strings.contains(&value) { strings.push(value.to_owned()) }),
            NodeType::AsciiArray(value) => value.iter().for_each(|value| if !strings.contains(&value) { strings.push(value.to_owned()) }),
            NodeType::Record(value) => {
                if !record_names.contains(&value.name) {
                    record_names.push(value.name.to_owned());
                }
                for node in &value.childs {
                    Self::read_string_from_node(&node, record_names, strings);
                }
            },
            NodeType::RecordBlock(value) => {
                if !record_names.contains(&value.name) {
                    record_names.push(value.name.to_owned());
                }
                for (_, node_group) in &value.childs {
                    for node in node_group {
                        Self::read_string_from_node(&node, record_names, strings);
                    }
                }
            },

            // Skip any other node.
            _ => {}
        }
    }
}

