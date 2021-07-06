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

        // Get the UTF-16 Strings for all the subnodes.
        let strings_count_utf16 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let mut strings_utf16 = BTreeMap::new();
        for _ in 0..strings_count_utf16 {
            let name = packed_file_data.decode_packedfile_string_u16(offset, &mut offset)?;
            let index = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            strings_utf16.insert(index, name);
        }

        // Get the UTF-8 Strings for all the subnodes.
        let strings_count_utf8 = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
        let mut strings_utf8 = BTreeMap::new();
        for _ in 0..strings_count_utf8 {
            let name = packed_file_data.decode_packedfile_string_u8(offset, &mut offset)?;
            let index = packed_file_data.decode_packedfile_integer_u32(offset, &mut offset)?;
            strings_utf8.insert(index, name);
        }

        // If we're not at the end of the file, something failed.
        if offset != packed_file_data.len() {
            return Err(ErrorKind::ESFIncompleteDecoding.into());
        }

        // Restore the index before continuing.
        offset = nodes_offset;

        // This file is a big tree hanging from the root node, so just decode everything recursively.
        let root_node = Self::read_node(&packed_file_data, &mut offset, true, &record_names, &strings_utf8, &strings_utf16)?;

        // If we're not at the exact end of the nodes, something failed.
        if offset != record_names_offset as usize {
            return Err(ErrorKind::ESFIncompleteDecoding.into());
        }

        let esf = Self{
            signature,
            unknown_1,
            creation_date,
            root_node,
        };

        // Code for debugging decoding/encoding errors outside of RPFM.
        // Re-encodes the decoded file and saves it to disk.
        //use std::io::Write;
        //let mut x = std::fs::File::create("encoded_starpos.esf")?;
        //x.write_all(&esf.save())?;

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
        let mut strings_utf8 = vec![];
        let mut strings_utf16 = vec![];
        Self::read_string_from_node(&self.root_node, &mut record_names, &mut strings_utf8, &mut strings_utf16);

        // Next, encode the nodes. We need them (and the strings) encoded in order to know their offsets.
        let mut nodes_data = Self::save_node(&self.root_node, true, &record_names, &strings_utf8, &strings_utf16);

        // Then, encode the strings.
        let mut strings_data: Vec<u8> = vec![];
        strings_data.encode_integer_u16(record_names.len() as u16);

        // First record names.
        for name in record_names {
            strings_data.encode_packedfile_string_u8(&name);
        }

        // Then UTF-16 Strings.
        strings_data.encode_integer_u32(strings_utf16.len() as u32);
        for (index, string) in strings_utf16.iter().enumerate() {
            strings_data.encode_packedfile_string_u16(&string);
            strings_data.encode_integer_u32(index as u32);
        }

        // Then UTF-8 Strings.
        strings_data.encode_integer_u32(strings_utf8.len() as u32);
        for (index, string) in strings_utf8.iter().enumerate() {
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
        strings_utf8: &BTreeMap<u32, String>,
        strings_utf16: &BTreeMap<u32, String>
    ) -> Result<NodeType> {
        let next_byte = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
        let is_record = next_byte & RecordNodeFlags::IS_RECORD_NODE.bits == RecordNodeFlags::IS_RECORD_NODE.bits;

        // Get the node type. If it's a record, process it separately from the rest, as records are significantly more complex than standard nodes.
        let node_type = if is_record {

            // Get the record flags, and decode it depending on what flags it has.
            let record_flags = RecordNodeFlags::from_bits_truncate(next_byte);
            let has_non_optimized_info = record_flags.contains(RecordNodeFlags::HAS_NON_OPTIMIZED_INFO) || is_root_node;
            let name_index;
            let version;

            if has_non_optimized_info {
                name_index = packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?;
                version = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
            }

            // If it's not the root node, the data is encoded in 2 bytes using bitwise.
            // From left to right:
            // - 0..=2: Flags.
            // - 3..6: Version.
            // - 7..16: Name index.
            else {
                version = (next_byte & 0x1E) >> 1;
                name_index = (((next_byte & 1) as u16) << 8) + packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u16;
            }

            let name = match record_names.get(name_index as usize) {
                Some(name) => name.to_owned(),
                None => return Err(ErrorKind::ESFRecordNameNotFound(name_index as u32).into())
            };

            // Get the block size, to know what data do we have to decode exactly.
            let block_size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
            let group_count = if record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?
            } else { 1 };
            let final_block_offset = *offset + block_size as usize;
            let mut children = Vec::with_capacity(group_count as usize);

            // Get the record data. This process differs depending if we have nested blocks or not.
            // If we have nested blocks, we decode group by group. If we don't, we just treat it as a single group.
            for _ in 0..group_count {
                let final_entry_offset = if record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                    let entry_size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    *offset + entry_size as usize
                } else {
                    final_block_offset
                };

                let mut node_list = vec![];
                while *offset < final_entry_offset {
                    node_list.push(Self::read_node(&packed_file_data[..final_entry_offset], &mut offset, false, record_names, strings_utf8, strings_utf16)?);
                }

                // Make sure we decoded exactly the data we wanted.
                if *offset != final_entry_offset {
                    return Err(ErrorKind::ESFIncompleteDecoding.into())
                }

                children.push(node_list);
            }

            // Make sure we decoded exactly the data we wanted.
            if *offset != final_block_offset {
                return Err(ErrorKind::ESFIncompleteDecoding.into())
            }

            let node_data = RecordNode {
                record_flags,
                version,
                name,
                children,
            };

            NodeType::Record(node_data)
        }

        // If its not a record node, get the type from the type byte and decode it.
        else {
            match next_byte {

                // Invalid node. This is always an error.
                INVALID => return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),

                //------------------------------------------------//
                // Primitive nodes.
                //------------------------------------------------//
                BOOL => NodeType::Bool(BoolNode {
                    value: packed_file_data.decode_packedfile_bool(*offset, &mut offset)?,
                    optimized: false,
                }),
                I8 => NodeType::I8(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?),
                I16 => NodeType::I16(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?),
                I32 => NodeType::I32(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?),
                I64 => NodeType::I64(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?),
                U8 => NodeType::U8(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?),
                U16 => NodeType::U16(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?),
                U32 => NodeType::U32(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?),
                U64 => NodeType::U64(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?),
                F32 => NodeType::F32(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?),
                F64 => NodeType::F64(packed_file_data.decode_packedfile_float_f64(*offset, &mut offset)?),

                //------------------------------------------------//
                // Complex/Specialized nodes.
                //------------------------------------------------//
                COORD_2D =>{
                    NodeType::Coord2d(Coordinates2DNode{
                        x: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                        y: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?
                    })
                },
                COORD_3D =>{
                    NodeType::Coord3d(Coordinates3DNode{
                        x: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                        y: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                        z: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?
                    })
                },

                //------------------------------------------------//
                // String nodes.
                //------------------------------------------------//
                UTF16 => {
                    let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                    match strings_utf16.get(&string_index) {
                        Some(string) => NodeType::Utf16(string.to_owned()),
                        None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                    }
                },
                ASCII => {
                    let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                    match strings_utf8.get(&string_index) {
                        Some(string) => NodeType::Ascii(string.to_owned()),
                        None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                    }
                },
                ANGLE => NodeType::Angle(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?),

                //------------------------------------------------//
                // Optimized primitive nodes.
                //------------------------------------------------//
                BOOL_TRUE => NodeType::Bool(BoolNode {
                    value: true,
                    optimized: true,
                }),
                BOOL_FALSE => NodeType::Bool(BoolNode {
                    value: false,
                    optimized: true,
                }),
                U32_ZERO => NodeType::U32Zero,
                U32_ONE => NodeType::U32One,
                U32_BYTE => NodeType::U32Byte(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u32),
                U32_16BIT => NodeType::U32_16bit(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as u32),
                U32_24BIT => NodeType::U32_24bit(packed_file_data.decode_packedfile_integer_u24(*offset, &mut offset)? as u32),
                I32_ZERO => NodeType::I32Zero,
                I32_BYTE => NodeType::I32Byte(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)? as i32),
                I32_16BIT =>NodeType::I32_16bit(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)? as i32),
                I32_24BIT =>NodeType::I32_24bit(packed_file_data.decode_packedfile_integer_i24(*offset, &mut offset)? as i32),
                F32_ZERO => NodeType::F32Zero,


                //------------------------------------------------//
                // Unknown nodes.
                //------------------------------------------------//
                UNKNOWN_21 => NodeType::Unknown21(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?),
                UNKNOWN_23 => NodeType::Unknown23(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?),
                //UNKNOWN_24 =>{},
                UNKNOWN_25 => NodeType::Unknown25(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?),

                // Very weird type.
                UNKNOWN_26 => {
                    let mut node_data = vec![];
                    let first_byte = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?;
                    node_data.push(first_byte);

                    if first_byte % 8 == 0 && first_byte != 0 {
                        node_data.extend_from_slice(&packed_file_data[*offset..*offset + first_byte as usize]);
                        *offset += first_byte as usize;
                    } else {
                        node_data.extend_from_slice(&packed_file_data[*offset..*offset + 7]);
                        *offset += 7;
                    }

                    let last_byte = packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset);
                    if last_byte.is_ok() && last_byte.unwrap() != 0x9C {
                        *offset -= 1;
                    }

                    NodeType::Unknown26(node_data)
                },

                //------------------------------------------------//
                // Arrays of primitive nodes.
                //------------------------------------------------//
                BOOL_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_bool(*offset, &mut offset)?);
                    }

                    NodeType::BoolArray(node_data)
                },
                I8_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)?)
                    }
                    NodeType::I8Array(node_data)

                },
                I16_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)
                    }
                    NodeType::I16Array(node_data)

                },
                I32_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i32(*offset, &mut offset)?);
                    }

                    NodeType::I32Array(node_data)

                },
                I64_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i64(*offset, &mut offset)?)
                    }

                    NodeType::I64Array(node_data)

                },
                U8_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)?)
                    }

                    NodeType::U8Array(node_data)

                },
                U16_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)?)
                    }

                    NodeType::U16Array(node_data)

                },
                U32_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?);
                    }

                    NodeType::U32Array(node_data)

                },
                U64_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u64(*offset, &mut offset)?)
                    }

                    NodeType::U64Array(node_data)

                },
                F32_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?);
                    }

                    NodeType::F32Array(node_data)
                },
                F64_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_float_f64(*offset, &mut offset)?);
                    }

                    NodeType::F64Array(node_data)
                },

                //------------------------------------------------//
                // Array of complex/specialized nodes.
                //------------------------------------------------//
                COORD_2D_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(Coordinates2DNode{
                            x: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                            y: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?
                        });
                    }

                    NodeType::Coord2dArray(node_data)

                },
                COORD_3D_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(Coordinates3DNode{
                            x: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                            y: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?,
                            z: packed_file_data.decode_packedfile_float_f32(*offset, &mut offset)?
                        });
                    }

                    NodeType::Coord3dArray(node_data)

                },

                //------------------------------------------------//
                // Array of string nodes.
                //------------------------------------------------//
                UTF16_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                        match strings_utf16.get(&string_index) {
                            Some(string) => node_data.push(string.to_owned()),
                            None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                        }
                    }
                    NodeType::Utf16Array(node_data)
                },
                ASCII_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        let string_index = packed_file_data.decode_packedfile_integer_u32(*offset, &mut offset)?;
                        match strings_utf8.get(&string_index) {
                            Some(string) => node_data.push(string.to_owned()),
                            None => return Err(ErrorKind::ESFStringNotFound(string_index).into()),
                        }
                    }
                    NodeType::AsciiArray(node_data)

                },

                //------------------------------------------------//
                // Array of unknown nodes.
                //------------------------------------------------//
                ANGLE_ARRAY =>{
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)?)
                    }

                    NodeType::AngleArray(node_data)
                },

                //------------------------------------------------//
                // Arrays of optimized primitive nodes.
                //------------------------------------------------//
                U32_BYTE_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u8(*offset, &mut offset)? as u32);
                    }

                    NodeType::U32ByteArray(node_data)
                },
                U32_16BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u16(*offset, &mut offset)? as u32);
                    }

                    NodeType::U32_16bitArray(node_data)
                },
                U32_24BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_u24(*offset, &mut offset)? as u32);
                    }

                    NodeType::U32_24bitArray(node_data)
                },
                I32_BYTE_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i8(*offset, &mut offset)? as i32);
                    }

                    NodeType::I32ByteArray(node_data)
                },
                I32_16BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i16(*offset, &mut offset)? as i32);
                    }

                    NodeType::I32_16bitArray(node_data)
                },
                I32_24BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = packed_file_data.decode_packedfile_integer_cauleb128(&mut offset)?;
                    let end_offset = *offset + size as usize;

                    while *offset < end_offset {
                        node_data.push(packed_file_data.decode_packedfile_integer_i24(*offset, &mut offset)? as i32);
                    }

                    NodeType::I32_24bitArray(node_data)
                },

                // Anything else is not yet supported.
                _ => return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into()),
            }
        };


        // Debugging code: re-save every slot and compare it with it's source data.
        // To check for read/save integrity.
        //let data = Self::save_node(&node_type, is_root_node, record_names, &strings_utf8.values().map(|x| x.to_owned()).collect::<Vec<String>>(), &strings_utf16.values().map(|x| x.to_owned()).collect::<Vec<String>>());
        //if data != packed_file_data[initial_offset..*offset] {
        //    dbg!(next_byte);
        //    dbg!(*offset);
        //    let max = data.len() / 10;
        //    dbg!(&data[..10]);
        //    dbg!(&packed_file_data[initial_offset..(initial_offset + 10)]);
        //    return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into());
        //}

        Ok(node_type)
    }

    /// This function takes care of reading a node's data into the appropiate NodeType.
    fn save_node(node_type: &NodeType, is_root_node: bool, record_names: &[String], strings_utf8: &[String], strings_utf16: &[String]) -> Vec<u8> {
        let mut data = vec![];
        match node_type {

            // Crash with this for now.
            NodeType::Invalid => unimplemented!(),

            //------------------------------------------------//
            // Primitive nodes.
            //------------------------------------------------//
            NodeType::Bool(value) => {
                if *value.get_ref_optimized() {
                    if *value.get_ref_value() {
                        data.push(BOOL_TRUE);
                    } else {
                        data.push(BOOL_FALSE);
                    }
                } else {
                    data.push(BOOL);
                    data.encode_bool(*value.get_ref_value());
                }
            },
            NodeType::I8(value) => {
                data.push(I8);
                data.encode_integer_i8(*value);
            },
            NodeType::I16(value) => {
                data.push(I16);
                data.encode_integer_i16(*value);
            },
            NodeType::I32(value) => {
                data.push(I32);
                data.encode_integer_i32(*value);
            },
            NodeType::I64(value) => {
                data.push(I64);
                data.encode_integer_i64(*value);
            },
            NodeType::U8(value) => {
                data.push(U8);
                data.push(*value);
            },
            NodeType::U16(value) => {
                data.push(U16);
                data.encode_integer_u16(*value);
            },
            NodeType::U32(value) => {
                data.push(U32);
                data.encode_integer_u32(*value);
            },
            NodeType::U64(value) => {
                data.push(U64);
                data.encode_integer_u64(*value);
            },
            NodeType::F32(value) => {
                data.push(F32);
                data.encode_float_f32(*value);
            },
            NodeType::F64(value) => {
                data.push(F64);
                data.encode_float_f64(*value);
            },

            //------------------------------------------------//
            // Complex nodes.
            //------------------------------------------------//
            NodeType::Coord2d(value) => {
                data.push(COORD_2D);
                data.encode_float_f32(value.x);
                data.encode_float_f32(value.y);
            },
            NodeType::Coord3d(value) => {
                data.push(COORD_3D);
                data.encode_float_f32(value.x);
                data.encode_float_f32(value.y);
                data.encode_float_f32(value.z);
            },

            //------------------------------------------------//
            // String nodes.
            //------------------------------------------------//
            NodeType::Utf16(value) => {
                data.push(UTF16);
                data.encode_integer_u32(strings_utf16.iter().position(|x| x == value).unwrap() as u32);
            },
            NodeType::Ascii(value) => {
                data.push(ASCII);
                data.encode_integer_u32(strings_utf8.iter().position(|x| x == value).unwrap() as u32);
            },
            NodeType::Angle(value) => {
                data.push(ANGLE);
                data.encode_integer_i16(*value);
            },

            //------------------------------------------------//
            // Optimized primitive nodes.
            //------------------------------------------------//
            NodeType::BoolTrue => data.push(BOOL_TRUE),
            NodeType::BoolFalse => data.push(BOOL_FALSE),
            NodeType::U32Zero => data.push(U32_ZERO),
            NodeType::U32One => data.push(U32_ONE),
            NodeType::U32Byte(value) => {
                data.push(U32_BYTE);
                data.push(*value as u8);
            },
            NodeType::U32_16bit(value) => {
                data.push(U32_16BIT);
                data.encode_integer_u16(*value as u16);
            },
            NodeType::U32_24bit(value) => {
                data.push(U32_24BIT);
                data.encode_integer_u24(*value);
            },
            NodeType::I32Zero => data.push(I32_ZERO),
            NodeType::I32Byte(value) => {
                data.push(I32_BYTE);
                data.encode_integer_i8(*value as i8);
            },
            NodeType::I32_16bit(value) => {
                data.push(I32_16BIT);
                data.encode_integer_i16(*value as i16);
            },
            NodeType::I32_24bit(value) => {
                data.push(I32_24BIT);
                data.encode_integer_i24(*value);
            },
            NodeType::F32Zero => data.push(F32_ZERO),

            //------------------------------------------------//
            // Unknown nodes.
            //------------------------------------------------//
            NodeType::Unknown21(value) => {
                data.push(UNKNOWN_21);
                data.encode_integer_u32(*value);
            },
            NodeType::Unknown23(value) => {
                data.push(UNKNOWN_23);
                data.push(*value);
            },
            //NodeType::Unknown_24(bool),
            NodeType::Unknown25(value) => {
                data.push(UNKNOWN_25);
                data.encode_integer_u32(*value);
            },
            NodeType::Unknown26(value) => {
                data.push(UNKNOWN_26);
                data.extend_from_slice(value);
            }

            //------------------------------------------------//
            // Arrays of primitive nodes.
            //------------------------------------------------//
            NodeType::BoolArray(value) => {
                data.push(BOOL_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_bool(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::I8Array(value) => {
                data.push(I8_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i8(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::I16Array(value) => {
                data.push(I16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i16(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::I32Array(value) => {
                data.push(I32_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i32(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::I64Array(value) => {
                data.push(I64_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i64(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::U8Array(value) => {
                data.push(U8_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.push(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::U16Array(value) => {
                data.push(U16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u16(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::U32Array(value) => {
                data.push(U32_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u32(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::U64Array(value) => {
                data.push(U64_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u64(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::F32Array(value) => {
                data.push(F32_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_float_f32(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::F64Array(value) => {
                data.push(F64_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_float_f64(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },

            //------------------------------------------------//
            // Array of complex/specialized nodes.
            //------------------------------------------------//
            NodeType::Coord2dArray(value) => {
                data.push(COORD_2D_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| {
                    list.encode_float_f32(x.x);
                    list.encode_float_f32(x.y);
                });

                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::Coord3dArray(value) => {
                data.push(COORD_3D_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| {
                    list.encode_float_f32(x.x);
                    list.encode_float_f32(x.y);
                    list.encode_float_f32(x.z);
                });

                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },

            //------------------------------------------------//
            // Array of string nodes.
            //------------------------------------------------//
            NodeType::Utf16Array(value) => {
                data.push(UTF16_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|y| {
                    list.encode_integer_u32(strings_utf16.iter().position(|x| x == y).unwrap() as u32);
                });

                data.encode_integer_u32(list.len() as u32);
                data.extend_from_slice(&list);
            },
            NodeType::AsciiArray(value) => {
                data.push(ASCII_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|y| {
                    list.encode_integer_u32(strings_utf8.iter().position(|x| x == y).unwrap() as u32);
                });

                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },

            //------------------------------------------------//
            // Array of unknown nodes.
            //------------------------------------------------//
            NodeType::AngleArray(value) => {
                data.push(ANGLE_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i16(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            },

            //------------------------------------------------//
            // Arrays of optimized primitive nodes.
            //------------------------------------------------//
            NodeType::U32ByteArray(value) => {
                data.push(U32_BYTE_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.push(*x as u8));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            NodeType::U32_16bitArray(value) => {
                data.push(U32_16BIT_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u16(*x as u16));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            NodeType::U32_24bitArray(value) => {
                data.push(U32_24BIT_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_u24(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            NodeType::I32ByteArray(value) => {
                data.push(I32_BYTE_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i8(*x as i8));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            NodeType::I32_16bitArray(value) => {
                data.push(I32_16BIT_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i16(*x as i16));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            NodeType::I32_24bitArray(value) => {
                data.push(I32_24BIT_ARRAY);

                let mut list = vec![];
                value.iter().for_each(|x| list.encode_integer_i24(*x));
                data.encode_integer_cauleb128(list.len() as u32);
                data.extend_from_slice(&list);
            }

            //------------------------------------------------//
            // Record nodes.
            //------------------------------------------------//
            NodeType::Record(value) => {
                if value.get_ref_record_flags().contains(RecordNodeFlags::HAS_NON_OPTIMIZED_INFO) || is_root_node {
                    data.push(value.get_ref_record_flags().bits());

                    data.encode_integer_u16(record_names.iter().position(|x| x == &value.name).unwrap() as u16);
                    data.push(value.version);
                }

                // If it's not the root node or uses optimized data, it needs special encoding.
                else {
                    let mut info: u16 = (value.get_ref_record_flags().bits() as u16) << 8;
                    info |= (value.version as u16) << 9;
                    info |= record_names.iter().position(|x| x == &value.name).unwrap() as u16;

                    data.push(info as u8);
                }

                let mut childs_data = vec![];

                if value.record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                    for group_node in &value.children {
                        let mut group_node_data = vec![];
                        for node in group_node {
                            let child_node = Self::save_node(&node, false, record_names, strings_utf8, strings_utf16);
                            group_node_data.extend_from_slice(&child_node);
                        }

                        childs_data.encode_integer_cauleb128(group_node_data.len() as u32);
                        childs_data.extend_from_slice(&group_node_data);
                    }

                    data.encode_integer_cauleb128(childs_data.len() as u32);
                    data.encode_integer_cauleb128(value.children.len() as u32);
                } else {

                    // For non-nested nodes, we just get the first and only children group.
                    let children_len = if let Some(children) = value.children.get(0) {
                        for node in children {
                            let child_node = Self::save_node(&node, false, record_names, strings_utf8, strings_utf16);
                            childs_data.extend_from_slice(&child_node);
                        }

                        children.len()
                    } else { 0 };

                    data.encode_integer_cauleb128(childs_data.len() as u32);
                    data.encode_integer_cauleb128(children_len as u32);
                }
                data.extend_from_slice(&childs_data);
            },
        }

        data
    }

    //---------------------------------------------------------------------------//
    //                       Utility functions for CAAB
    //---------------------------------------------------------------------------//

    /// This function reads the strings from the provided node and all its children.
    ///
    /// This function is recursive: if you pass it the root node, it'll read all the strings in the ESF file.
    fn read_string_from_node(node_type: &NodeType, record_names: &mut Vec<String>, strings_utf8: &mut Vec<String>, strings_utf16: &mut Vec<String>) {
        match node_type {
            NodeType::Utf16(value) => if !strings_utf16.contains(value) { strings_utf16.push(value.to_owned()) },
            NodeType::Ascii(value) => if !strings_utf8.contains(value) { strings_utf8.push(value.to_owned()) },
            NodeType::Utf16Array(value) => value.iter().for_each(|value| if !strings_utf16.contains(&value) { strings_utf16.push(value.to_owned()) }),
            NodeType::AsciiArray(value) => value.iter().for_each(|value| if !strings_utf8.contains(&value) { strings_utf8.push(value.to_owned()) }),
            NodeType::Record(value) => {
                if !record_names.contains(&value.name) {
                    record_names.push(value.name.to_owned());
                }
                for node_group in &value.children {
                    for node in node_group {
                        Self::read_string_from_node(&node, record_names, strings_utf8, strings_utf16);
                    }
                }
            },

            // Skip any other node.
            _ => {}
        }
    }

    /*
    fn decompress_compressed_data(data: &[u8], uncompressed_size: Option<&u32>, properties: &[u8]) -> Result<Vec<u8>> {
        use xz2::read::XzDecoder;
        use xz2::write::XzEncoder;
        use xz2::stream::Stream;
        use xz2::stream::LzmaOptions;
        use xz2::stream::Action;
        use std::io::{BufReader, Read, SeekFrom};



            //int lc = properties[0] % 9;
            //int remainder = properties[0] / 9;
            //int lp = remainder % 5;
            //int pb = remainder / 5;
            //if (pb > Base.kNumPosStatesBitsMax)
            //    throw new InvalidParamException();
            //UInt32 dictionarySize = 0;
            //for (int i = 0; i < 4; i++)
            //    dictionarySize += ((UInt32)(properties[1 + i])) << (i * 8);
            //SetDictionarySize(dictionarySize);
            //SetLiteralProperties(lp, lc);
            //SetPosBitsProperties(pb);


        if properties.len() != 5 {
            return Err(ErrorKind::PackedFileDataCouldNotBeDecompressed.into());
        }

        let lc = properties[0] % 9;
        let remainder = properties[0] / 9;
        let lp = remainder % 5;
        let pb = remainder / 5;

        let mut dictionary_size: u32 = 0;
        for (index, property) in properties[1..].iter().enumerate() {
            dictionary_size += (*property as u32) << (index * 8);
        }

        let mut lzma_options = LzmaOptions::new_preset(6).unwrap();
        lzma_options.dict_size(dictionary_size);
        lzma_options.position_bits(pb as u32);
        lzma_options.literal_position_bits(lp as u32);
        lzma_options.literal_context_bits(lc as u32);

        let mut output = vec![];
        let mut encoder_stream = Stream::new_lzma_encoder(&lzma_options).unwrap();
        {
            let mut encoder = XzEncoder::new_stream(&mut output, encoder_stream);
            let _ = encoder.write(&[]);
        }
        dbg!(&output);


        let mut stream = Stream::new_lzma_decoder(u64::MAX).map_err(|_| Error::from(ErrorKind::PackedFileDataCouldNotBeDecompressed))?;

        dbg!(&output);
        output.extend_from_slice(data);

                use std::io::Write;
        let mut x = std::fs::File::create("compressed_esf_data.lzma")?;
        x.write_all(&output)?;
        let mut encoder = XzDecoder::new_stream(&*output, stream);
        let mut compressed_data = match uncompressed_size {
            Some(size) => Vec::with_capacity(*size as usize),
            None => vec![],
        };

        match encoder.read_to_end(&mut compressed_data) {
            Ok(_) => Ok(compressed_data),
            Err(_) => Err(ErrorKind::PackedFileDataCouldNotBeDecompressed.into())
        }
    }*/
}

