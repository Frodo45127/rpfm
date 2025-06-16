//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the logic and documentation for the CAAB ESF files.

use std::collections::BTreeMap;
use std::io::{Cursor, SeekFrom, Write};

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::{Compressible, CompressionFormat, Decompressible};
use crate::error::{RLibError, Result};

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

/// Implementation of `ESF`. Section of functions specific for the CAAB format.
impl ESF {

    /// This function creates a `ESF` of type CAAB from a `Vec<u8>`.
    pub(crate) fn read_caab<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let signature = ESFSignature::CAAB;

        // Note: this assumes the caller has already read the first 4 bytes of the data.
        let unknown_1 = data.read_u32()?;
        let creation_date = data.read_u32()?;
        let record_names_offset = data.read_u32()?;
        let nodes_offset = data.stream_position()?;

        // We need this data decoded first, because some nodes reference to it, and we can use that to populate the nodes.
        data.seek(SeekFrom::Start(record_names_offset as u64))?;

        // Get the name list for the record/record block entries.
        let record_names_count = data.read_u16()?;
        let mut record_names = vec![];
        for _ in 0..record_names_count {
            record_names.push(data.read_sized_string_u8()?);
        }

        // Get the UTF-16 Strings for all the subnodes.
        let strings_count_utf16 = data.read_u32()?;
        let mut strings_utf16 = BTreeMap::new();
        for _ in 0..strings_count_utf16 {
            let name = data.read_sized_string_u16()?;
            let index = data.read_u32()?;
            strings_utf16.insert(index, name);
        }

        // Get the UTF-8 Strings for all the subnodes.
        let strings_count_utf8 = data.read_u32()?;
        let mut strings_utf8 = BTreeMap::new();
        for _ in 0..strings_count_utf8 {
            let name = data.read_sized_string_u8()?;
            let index = data.read_u32()?;
            strings_utf8.insert(index, name);
        }

        // If we're not at the end of the file, something failed.
        let data_len = data.len()?;
        let curr_pos = data.stream_position()?;
        if curr_pos != data_len {
            return Err(RLibError::DecodingMismatchSizeError(data_len as usize, curr_pos as usize));
        }

        // Restore the index before continuing.
        data.seek(SeekFrom::Start(nodes_offset))?;

        // This file is a big tree hanging from the root node, so just decode everything recursively.
        let root_node = Self::read_node(data, true, &record_names, &strings_utf8, &strings_utf16)?;

        // If we're not at the exact end of the nodes, something failed.
        let curr_pos = data.stream_position()?;
        if curr_pos != record_names_offset as u64 {
            return Err(RLibError::DecodingMismatchSizeError(record_names_offset as usize, curr_pos as usize));
        }

        let mut esf = Self{
            signature,
            unknown_1,
            creation_date,
            root_node,
        };

        // Once we're done with the nodes, we need to check if the last children of the root node contains a compressed record.
        // If so, that record will contain an entire ESF which we need to decode, and then replace ours with that one.
        //
        // The reason for this is, I guess, optimization. Some ESF files, specially the startpos ones, may have specific nodes that are enormous.
        // By keeping a compressed copy of the startpos, the game can read all the other nodes without loading the big ones to memory. But it's just a guess.
        if let NodeType::Record(ref mut node) = esf.root_node {
            if let Some(child) = node.children_mut().get_mut(0) {
                if let Some(NodeType::Record(cnode)) = child.last_mut() {
                    if cnode.name == COMPRESSED_DATA_TAG {
                        let mut dec_data = vec![];
                        if let Some(NodeType::U8Array(data)) = cnode.children()[0].get(0) {
                            if let Some(NodeType::Record(hnode)) = cnode.children()[0].get(1) {
                                if hnode.name == COMPRESSED_DATA_INFO_TAG {
                                    if let Some(NodeType::U32(len)) = hnode.children()[0].get(0) {
                                        if let Some(NodeType::U8Array(magic_number)) = hnode.children()[0].get(1) {

                                            let mut mdata = vec![];
                                            mdata.write_u32(*len.value())?;
                                            mdata.write_all(&magic_number)?;
                                            mdata.write_all(data)?;

                                            dec_data = mdata.as_slice().decompress()?;

                                            //let path_1 = "../test_files/test_decode_esf_caab.esf_decompressed";
                                            //let mut writer = std::io::BufWriter::new(std::fs::File::create(path_1).unwrap());
                                            //writer.write_all(&dec_data).unwrap();
                                        }
                                    }
                                }
                            }
                        }

                        if !dec_data.is_empty() {
                            let mut dec_datac = Cursor::new(dec_data.clone());
                            let new_esf = ESF::decode(&mut dec_datac, &None)?;
                            esf = new_esf;
                        }
                    }
                }
            }
        }

        Ok(esf)
    }

    /// This function takes a `ESF` of type CAAB and encodes it to `Vec<u8>`.
    pub(crate) fn save_caab<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let mut extra_data = extra_data.clone().unwrap_or_default();
        let disable_compression = extra_data.disable_compression;

        let backup = self.clone();
        let mut revert_compression = false;
        let mut index = None;

        // If we have a known compressed node, encode the esf, compress it, then replace said node with the compressed esf.
        // Note that the operation alters the self, so we need to restore it at the end of this function in order to keep it usable.
        if !disable_compression {
            if let NodeType::Record(ref root_node) = self.root_node {
                for (i1, parent) in root_node.children().iter().enumerate() {
                    for (i2, child) in parent.iter().enumerate() {
                        if let NodeType::Record(ref cnode) = child {
                            if COMPRESSED_TAGS.contains(&&*cnode.name)  {
                                index = Some((i1, i2));
                                break;
                            }
                        }
                    }
                }
            }

            if let Some((i1, i2)) = index {
                let mut ncdata = vec![];

                extra_data.disable_compression = true;
                self.encode(&mut ncdata, &Some(extra_data))?;
                revert_compression = true;

                let mut fdata = ncdata.compress(CompressionFormat::Lzma1)?;
                let cdata = fdata.split_off(9);
                let mut hdata = Cursor::new(fdata);

                let hnode = RecordNode {
                    record_flags: RecordNodeFlags::IS_RECORD_NODE,
                    version: 0,
                    name: COMPRESSED_DATA_INFO_TAG.to_owned(),
                    children: vec![vec![
                        NodeType::U32(U32Node {
                            value: hdata.read_i32()? as u32,
                            optimized: false,
                        }),
                        NodeType::U8Array(hdata.read_slice(5, false)?),
                    ]],
                };

                let cnode = RecordNode {
                    record_flags: RecordNodeFlags::IS_RECORD_NODE,
                    version: 0,
                    name: COMPRESSED_DATA_TAG.to_owned(),
                    children: vec![vec![
                        NodeType::U8Array(cdata),
                        NodeType::Record(hnode),
                    ]],
                };

                // Replace the full node with the compressed one.
                if let NodeType::Record(ref mut root_node) = self.root_node {
                    if let Some(parent) = root_node.children_mut().get_mut(i1) {
                        if let Some(child) = parent.get_mut(i2) {
                            *child = NodeType::Record(cnode);
                        }
                    }
                }
            }
        }

        // Encode the header info, except the offsets, because those are calculated later.
        buffer.write_all(SIGNATURE_CAAB)?;
        buffer.write_u32(self.unknown_1)?;
        buffer.write_u32(self.creation_date)?;

        // First, get the strings encoded, as we need to have them in order before encoding the nodes.
        let mut record_names = vec![];
        let mut strings_utf8 = vec![];
        let mut strings_utf16 = vec![];
        Self::read_string_from_node(&self.root_node, &mut record_names, &mut strings_utf8, &mut strings_utf16);

        // Next, encode the nodes. We need them (and the strings) encoded in order to know their offsets.
        let mut nodes_data = vec![];
        Self::save_node(&mut nodes_data, &self.root_node, true, &record_names, &strings_utf8, &strings_utf16)?;

        // Then, encode the strings.
        let mut strings_data: Vec<u8> = vec![];
        strings_data.write_u16(record_names.len() as u16)?;

        // First record names.
        for name in record_names {
            strings_data.write_sized_string_u8(&name)?;
        }

        // Then UTF-16 Strings.
        strings_data.write_u32(strings_utf16.len() as u32)?;
        for (index, string) in strings_utf16.iter().enumerate() {
            strings_data.write_sized_string_u16(string)?;
            strings_data.write_u32(index as u32)?;
        }

        // Then UTF-8 Strings.
        strings_data.write_u32(strings_utf8.len() as u32)?;
        for (index, string) in strings_utf8.iter().enumerate() {
            strings_data.write_sized_string_u8(string)?;
            strings_data.write_u32(index as u32)?;
        }

        // And finally, merge everything.
        buffer.write_u32((12 + nodes_data.len() + 4) as u32)?;
        buffer.write_all(&nodes_data)?;
        buffer.write_all(&strings_data)?;

        if revert_compression {
            *self = backup;
        }

        Ok(())
    }

    /// This function takes care of reading a node's data into the appropriate NodeType.
    fn read_node<R: ReadBytes>(
        data: &mut R,
        is_root_node: bool,
        record_names: &[String],
        strings_utf8: &BTreeMap<u32, String>,
        strings_utf16: &BTreeMap<u32, String>
    ) -> Result<NodeType> {

        let next_byte = data.read_u8()?;
        let is_record = next_byte & RecordNodeFlags::IS_RECORD_NODE.bits() == RecordNodeFlags::IS_RECORD_NODE.bits();

        // Get the node type. If it's a record, process it separately from the rest, as records are significantly more complex than standard nodes.
        let node_type = if is_record {

            // Get the record flags, and decode it depending on what flags it has.
            let record_flags = RecordNodeFlags::from_bits_truncate(next_byte);
            let has_non_optimized_info = record_flags.contains(RecordNodeFlags::HAS_NON_OPTIMIZED_INFO) || is_root_node;
            let name_index;
            let version;

            if has_non_optimized_info {
                name_index = data.read_u16()?;
                version = data.read_u8()?;
            }

            // If it's not the root node, the data is encoded in 2 bytes using bitwise.
            // From left to right:
            // - 0..=2: Flags.
            // - 3..6: Version.
            // - 7..16: Name index.
            else {
                version = (next_byte & 0x1E) >> 1;
                name_index = (((next_byte & 1) as u16) << 8) + data.read_u8()? as u16;
            }

            let name = match record_names.get(name_index as usize) {
                Some(name) => name.to_owned(),
                None => return Err(RLibError::DecodingESFRecordNameNotFound(name_index)),
            };

            // Get the block size, to know what data do we have to decode exactly.
            let block_size = data.read_cauleb128()?;
            let group_count = if record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                data.read_cauleb128()?
            } else { 1 };

            let final_block_offset = data.stream_position()? as usize + block_size as usize;
            let mut children = Vec::with_capacity(group_count as usize);

            // Get the record data. This process differs depending if we have nested blocks or not.
            // If we have nested blocks, we decode group by group. If we don't, we just treat it as a single group.
            for _ in 0..group_count {
                let final_entry_offset = if record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                    let entry_size = data.read_cauleb128()?;
                    data.stream_position()? as usize + entry_size as usize
                } else {
                    final_block_offset
                };

                let mut node_list = vec![];
                while data.stream_position()? < final_entry_offset as u64 {
                    node_list.push(Self::read_node(data, false, record_names, strings_utf8, strings_utf16)?);
                }

                // Make sure we decoded exactly the data we wanted.
                let curr_pos = data.stream_position()?;
                if curr_pos != final_entry_offset as u64 {
                    return Err(RLibError::DecodingMismatchSizeError(final_entry_offset, curr_pos as usize));
                }

                children.push(node_list);
            }

            // Make sure we decoded exactly the data we wanted.
            let curr_pos = data.stream_position()?;
            if curr_pos != final_block_offset as u64 {
                return Err(RLibError::DecodingMismatchSizeError(final_block_offset, curr_pos as usize));
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
                INVALID => return Err(RLibError::DecodingESFUnsupportedDataType(next_byte)),

                //------------------------------------------------//
                // Primitive nodes.
                //------------------------------------------------//
                BOOL => NodeType::Bool(BoolNode {
                    value: data.read_bool()?,
                    optimized: false,
                }),
                I8 => NodeType::I8(data.read_i8()?),
                I16 => NodeType::I16(data.read_i16()?),
                I32 => NodeType::I32(I32Node {
                    value: data.read_i32()?,
                    optimized: false,
                }),
                I64 => NodeType::I64(data.read_i64()?),
                U8 => NodeType::U8(data.read_u8()?),
                U16 => NodeType::U16(data.read_u16()?),
                U32 => NodeType::U32(U32Node {
                    value: data.read_u32()?,
                    optimized: false,
                }),
                U64 => NodeType::U64(data.read_u64()?),
                F32 => NodeType::F32(F32Node {
                    value: data.read_f32()?,
                    optimized: false,
                }),
                F64 => NodeType::F64(data.read_f64()?),

                //------------------------------------------------//
                // Complex/Specialized nodes.
                //------------------------------------------------//
                COORD_2D =>{
                    NodeType::Coord2d(Coordinates2DNode{
                        x: data.read_f32()?,
                        y: data.read_f32()?
                    })
                },
                COORD_3D =>{
                    NodeType::Coord3d(Coordinates3DNode{
                        x: data.read_f32()?,
                        y: data.read_f32()?,
                        z: data.read_f32()?
                    })
                },

                //------------------------------------------------//
                // String nodes.
                //------------------------------------------------//
                UTF16 => {
                    let string_index = data.read_u32()?;
                    match strings_utf16.get(&string_index) {
                        Some(string) => NodeType::Utf16(string.to_owned()),
                        None => return Err(RLibError::DecodingESFStringNotFound(string_index)),
                    }
                },
                ASCII => {
                    let string_index = data.read_u32()?;
                    match strings_utf8.get(&string_index) {
                        Some(string) => NodeType::Ascii(string.to_owned()),
                        None => return Err(RLibError::DecodingESFStringNotFound(string_index)),
                    }
                },
                ANGLE => NodeType::Angle(data.read_i16()?),

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
                U32_ZERO => NodeType::U32(U32Node {
                    value: 0,
                    optimized: true,
                }),
                U32_ONE => NodeType::U32(U32Node {
                    value: 1,
                    optimized: true,
                }),
                U32_BYTE => NodeType::U32(U32Node {
                    value: data.read_u8()? as u32,
                    optimized: true,
                }),
                U32_16BIT => NodeType::U32(U32Node {
                    value: data.read_u16()? as u32,
                    optimized: true,
                }),
                U32_24BIT => NodeType::U32(U32Node {
                    value: data.read_u24()?,
                    optimized: true,
                }),
                I32_ZERO => NodeType::I32(I32Node {
                    value: 0,
                    optimized: true,
                }),
                I32_BYTE => NodeType::I32(I32Node {
                    value: data.read_i8()? as i32,
                    optimized: true,
                }),
                I32_16BIT => NodeType::I32(I32Node {
                    value: data.read_i16()? as i32,
                    optimized: true,
                }),
                I32_24BIT => NodeType::I32(I32Node {
                    value: data.read_i24()?,
                    optimized: true,
                }),
                F32_ZERO => NodeType::F32(F32Node {
                    value: 0.0,
                    optimized: true,
                }),

                //------------------------------------------------//
                // Unknown nodes.
                //------------------------------------------------//
                UNKNOWN_21 => NodeType::Unknown21(data.read_u32()?),
                UNKNOWN_23 => NodeType::Unknown23(data.read_u8()?),
                UNKNOWN_24 => NodeType::Unknown24(data.read_u16()?),
                UNKNOWN_25 => NodeType::Unknown25(data.read_u32()?),

                // Very weird type.
                UNKNOWN_26 => {
                    let mut node_data = vec![];
                    let first_byte = data.read_u8()?;
                    node_data.push(first_byte);

                    if first_byte % 8 == 0 && first_byte != 0 {
                        node_data.extend_from_slice(&data.read_slice(first_byte as usize, false)?);
                    } else {
                        node_data.extend_from_slice(&data.read_slice(7, false)?);
                    }

                    let last_byte = data.read_u8();
                    if last_byte.is_ok() && last_byte.unwrap() != 0x9C {
                        data.seek(SeekFrom::Current(-1))?;
                    }

                    NodeType::Unknown26(node_data)
                },

                //------------------------------------------------//
                // Arrays of primitive nodes.
                //------------------------------------------------//
                BOOL_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_bool()?);
                    }

                    NodeType::BoolArray(node_data)
                },

                I8_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i8()?)
                    }
                    NodeType::I8Array(node_data)
                },

                I16_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i16()?)
                    }
                    NodeType::I16Array(node_data)
                },

                I32_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i32()?);
                    }

                    NodeType::I32Array(VecI32Node {
                        value: node_data,
                        optimized: false,
                    })
                },

                I64_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i64()?);
                    }

                    NodeType::I64Array(node_data)
                },

                U8_ARRAY => {
                    let size = data.read_cauleb128()?;
                    let node_data = data.read_slice(size as usize, false)?;

                    NodeType::U8Array(node_data)
                },

                U16_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u16()?);
                    }

                    NodeType::U16Array(node_data)
                },

                U32_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u32()?);
                    }

                    NodeType::U32Array(VecU32Node {
                        value: node_data,
                        optimized: false,
                    })
                },

                U64_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u64()?)
                    }

                    NodeType::U64Array(node_data)
                },

                F32_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_f32()?);
                    }

                    NodeType::F32Array(node_data)
                },

                F64_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_f64()?);
                    }

                    NodeType::F64Array(node_data)
                },

                //------------------------------------------------//
                // Array of complex/specialized nodes.
                //------------------------------------------------//
                COORD_2D_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(Coordinates2DNode{
                            x: data.read_f32()?,
                            y: data.read_f32()?
                        });
                    }

                    NodeType::Coord2dArray(node_data)
                },

                COORD_3D_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(Coordinates3DNode{
                            x: data.read_f32()?,
                            y: data.read_f32()?,
                            z: data.read_f32()?
                        });
                    }

                    NodeType::Coord3dArray(node_data)
                },

                //------------------------------------------------//
                // Array of string nodes.
                //------------------------------------------------//
                UTF16_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        let string_index = data.read_u32()?;
                        match strings_utf16.get(&string_index) {
                            Some(string) => node_data.push(string.to_owned()),
                            None => return Err(RLibError::DecodingESFStringNotFound(string_index)),
                        }
                    }
                    NodeType::Utf16Array(node_data)
                },

                ASCII_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        let string_index = data.read_u32()?;
                        match strings_utf8.get(&string_index) {
                            Some(string) => node_data.push(string.to_owned()),
                            None => return Err(RLibError::DecodingESFStringNotFound(string_index)),
                        }
                    }
                    NodeType::AsciiArray(node_data)
                },

                //------------------------------------------------//
                // Array of unknown nodes.
                //------------------------------------------------//
                ANGLE_ARRAY =>{
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i16()?)
                    }

                    NodeType::AngleArray(node_data)
                },

                //------------------------------------------------//
                // Arrays of optimized primitive nodes.
                //------------------------------------------------//
                U32_BYTE_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u8()? as u32);
                    }

                    NodeType::U32Array(VecU32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                U32_16BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u16()? as u32);
                    }

                    NodeType::U32Array(VecU32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                U32_24BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_u24()?);
                    }

                    NodeType::U32Array(VecU32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                I32_BYTE_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i8()? as i32);
                    }

                    NodeType::I32Array(VecI32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                I32_16BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i16()? as i32);
                    }

                    NodeType::I32Array(VecI32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                I32_24BIT_ARRAY => {
                    let mut node_data = vec![];
                    let size = data.read_cauleb128()?;
                    let end_offset = data.stream_position()? + size as u64;

                    while data.stream_position()? < end_offset {
                        node_data.push(data.read_i24()?);
                    }

                    NodeType::I32Array(VecI32Node {
                        value: node_data,
                        optimized: true,
                    })
                },

                // Anything else is not yet supported.
                _ => return Err(RLibError::DecodingESFUnsupportedDataType(next_byte)),
            }
        };

        // Debugging code: re-save every slot and compare it with it's source data.
        // To check for read/save integrity.
        //if *offset > 1040000 {
        //    let data = Self::save_node(&node_type, is_root_node, record_names, &strings_utf8.values().map(|x| x.to_owned()).collect::<Vec<String>>(), &strings_utf16.values().map(|x| x.to_owned()).collect::<Vec<String>>());
        //    if data != data[initial_offset..*offset] {
        //        dbg!(next_byte);
        //        dbg!(*offset);
        //        let max = if data.len() > 20 { 20 } else { data.len() };
        //        dbg!(&data[..max]);
        //        dbg!(&data[initial_offset..(initial_offset + max)]);
        //        //return Err(ErrorKind::ESFUnsupportedDataType(format!("{}", next_byte)).into());
        //    }
        //}

        Ok(node_type)
    }

    /// This function takes care of reading a node's data into the appropriate NodeType.
    fn save_node<W: WriteBytes>(buffer: &mut W, node_type: &NodeType, is_root_node: bool, record_names: &[String], strings_utf8: &[String], strings_utf16: &[String]) -> Result<()> {
        match node_type {

            // Crash with this for now.
            NodeType::Invalid => unimplemented!(),

            //------------------------------------------------//
            // Primitive nodes.
            //------------------------------------------------//
            NodeType::Bool(value) => {
                if *value.optimized() {
                    if *value.value() {
                        buffer.write_u8(BOOL_TRUE)?;
                    } else {
                        buffer.write_u8(BOOL_FALSE)?;
                    }
                } else {
                    buffer.write_u8(BOOL)?;
                    buffer.write_bool(*value.value())?;
                }
            },
            NodeType::I8(value) => {
                buffer.write_u8(I8)?;
                buffer.write_i8(*value)?;
            },
            NodeType::I16(value) => {
                buffer.write_u8(I16)?;
                buffer.write_i16(*value)?;
            },
            NodeType::I32(value) => {
                if *value.optimized() {
                    let value = *value.value();
                    if value == 0 {
                        buffer.write_u8(I32_ZERO)?;
                    }

                    // We can do simple logic for positive numbers, but negative numbers need special logic to get their size correctly.
                    else if value.is_positive() {
                        if value <= i8::MAX as i32 {
                            buffer.write_u8(I32_BYTE)?;
                            buffer.write_i8(value as i8)?;
                        } else if value <= i16::MAX as i32 {
                            buffer.write_u8(I32_16BIT)?;
                            buffer.write_i16(value as i16)?;
                        } else if value <= 8_388_607 {
                            buffer.write_u8(I32_24BIT)?;
                            buffer.write_i24(value)?;
                        } else {
                            buffer.write_u8(I32)?;
                            buffer.write_i32(value)?;
                        }
                    } else if value >= i8::MIN as i32 {
                        buffer.write_u8(I32_BYTE)?;
                        buffer.write_i8(value as i8)?;
                    } else if value >= i16::MIN as i32 {
                        buffer.write_u8(I32_16BIT)?;
                        buffer.write_i16(value as i16)?;
                    } else if value >= -8_388_608 {
                        buffer.write_u8(I32_24BIT)?;
                        buffer.write_i24(value)?;
                    } else {
                        buffer.write_u8(I32)?;
                        buffer.write_i32(value)?;
                    }
                } else {
                    buffer.write_u8(I32)?;
                    buffer.write_i32(*value.value())?;
                }
            },
            NodeType::I64(value) => {
                buffer.write_u8(I64)?;
                buffer.write_i64(*value)?;
            },
            NodeType::U8(value) => {
                buffer.write_u8(U8)?;
                buffer.write_u8(*value)?;
            },
            NodeType::U16(value) => {
                buffer.write_u8(U16)?;
                buffer.write_u16(*value)?;
            },
            NodeType::U32(value) => {
                if *value.optimized() {
                    let value = *value.value();
                    if value == 0 {
                        buffer.write_u8(U32_ZERO)?;
                    } else if value == 1 {
                        buffer.write_u8(U32_ONE)?;
                    } else if value <= 0xFF {
                        buffer.write_u8(U32_BYTE)?;
                        buffer.write_u8(value as u8)?;
                    } else if value <= 0xFFFF {
                        buffer.write_u8(U32_16BIT)?;
                        buffer.write_u16(value as u16)?;
                    } else if value <= 0xFFFFFF {
                        buffer.write_u8(U32_24BIT)?;
                        buffer.write_u24(value)?;
                    } else {
                        buffer.write_u8(U32)?;
                        buffer.write_u32(value)?;
                    }
                } else {
                    buffer.write_u8(U32)?;
                    buffer.write_u32(*value.value())?;
                }
            },
            NodeType::U64(value) => {
                buffer.write_u8(U64)?;
                buffer.write_u64(*value)?;
            },
            NodeType::F32(value) => {
                if *value.optimized() {
                    let value = *value.value();
                    if (value - 0.0).abs() < f32::EPSILON {
                        buffer.write_u8(F32_ZERO)?;
                    } else {
                        buffer.write_u8(F32)?;
                        buffer.write_f32(value)?;
                    }
                } else {
                    buffer.write_u8(F32)?;
                    buffer.write_f32(*value.value())?;
                }
            },
            NodeType::F64(value) => {
                buffer.write_u8(F64)?;
                buffer.write_f64(*value)?;
            },

            //------------------------------------------------//
            // Complex nodes.
            //------------------------------------------------//
            NodeType::Coord2d(value) => {
                buffer.write_u8(COORD_2D)?;
                buffer.write_f32(value.x)?;
                buffer.write_f32(value.y)?;
            },
            NodeType::Coord3d(value) => {
                buffer.write_u8(COORD_3D)?;
                buffer.write_f32(value.x)?;
                buffer.write_f32(value.y)?;
                buffer.write_f32(value.z)?;
            },

            //------------------------------------------------//
            // String nodes.
            //------------------------------------------------//
            NodeType::Utf16(value) => {
                buffer.write_u8(UTF16)?;
                buffer.write_u32(strings_utf16.iter().position(|x| x == value).unwrap() as u32)?;
            },
            NodeType::Ascii(value) => {
                buffer.write_u8(ASCII)?;
                buffer.write_u32(strings_utf8.iter().position(|x| x == value).unwrap() as u32)?;
            },
            NodeType::Angle(value) => {
                buffer.write_u8(ANGLE)?;
                buffer.write_i16(*value)?;
            },

            //------------------------------------------------//
            // Unknown nodes.
            //------------------------------------------------//
            NodeType::Unknown21(value) => {
                buffer.write_u8(UNKNOWN_21)?;
                buffer.write_u32(*value)?;
            },
            NodeType::Unknown23(value) => {
                buffer.write_u8(UNKNOWN_23)?;
                buffer.write_u8(*value)?;
            },
            NodeType::Unknown24(value) => {
                buffer.write_u8(UNKNOWN_24)?;
                buffer.write_u16(*value)?;

            },
            NodeType::Unknown25(value) => {
                buffer.write_u8(UNKNOWN_25)?;
                buffer.write_u32(*value)?;
            },
            NodeType::Unknown26(value) => {
                buffer.write_u8(UNKNOWN_26)?;
                buffer.write_all(value)?;
            }

            //------------------------------------------------//
            // Arrays of primitive nodes.
            //------------------------------------------------//
            NodeType::BoolArray(value) => {
                buffer.write_u8(BOOL_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_bool(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::I8Array(value) => {
                buffer.write_u8(I8_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_i8(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::I16Array(value) => {
                buffer.write_u8(I16_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_i16(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::I32Array(value) => {
                let mut list = vec![];
                if *value.optimized() {
                    if let Some(max_value) = value.value().iter().max() {
                        if let Some(min_value) = value.value().iter().min() {
                            let max_value = std::cmp::max(min_value.abs(), max_value.abs());
                            if max_value <= i8::MAX as i32 {
                                buffer.write_u8(I32_BYTE_ARRAY)?;
                                value.value().iter().try_for_each(|x| list.write_i8(*x as i8))?;
                            } else if max_value <= i16::MAX as i32 {
                                buffer.write_u8(I32_16BIT_ARRAY)?;
                                value.value().iter().try_for_each(|x| list.write_i16(*x as i16))?;
                            } else if max_value <= 8_388_607 {
                                buffer.write_u8(I32_24BIT_ARRAY)?;
                                value.value().iter().try_for_each(|x| list.write_i24(*x))?;
                            } else {
                                buffer.write_u8(I32_ARRAY)?;
                                value.value().iter().try_for_each(|x| list.write_i32(*x))?;
                            }
                        }
                    }
                } else {
                    buffer.write_u8(I32_ARRAY)?;
                    value.value().iter().try_for_each(|x| list.write_i32(*x))?;
                }

                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },

            NodeType::I64Array(value) => {
                buffer.write_u8(I64_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_i64(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::U8Array(value) => {
                buffer.write_u8(U8_ARRAY)?;
                buffer.write_cauleb128(value.len() as u32, 0)?;
                buffer.write_all(value)?;
            },
            NodeType::U16Array(value) => {
                buffer.write_u8(U16_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_u16(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::U32Array(value) => {
                let mut list = vec![];
                if *value.optimized() {
                    if let Some(max_value) = value.value().iter().max() {
                        if max_value < &0xFF {
                            buffer.write_u8(U32_BYTE_ARRAY)?;
                            value.value().iter().for_each(|x| list.push(*x as u8));
                        } else if max_value < &0xFFFF {
                            buffer.write_u8(U32_16BIT_ARRAY)?;
                            value.value().iter().try_for_each(|x| list.write_u16(*x as u16))?;
                        } else if max_value < &0xFFFFFF {
                            buffer.write_u8(U32_24BIT_ARRAY)?;
                            value.value().iter().try_for_each(|x| list.write_u24(*x))?;
                        } else {
                            buffer.write_u8(U32_ARRAY)?;
                            value.value().iter().try_for_each(|x| list.write_u32(*x))?;
                        }
                    }
                } else {
                    buffer.write_u8(U32_ARRAY)?;
                    value.value().iter().try_for_each(|x| list.write_u32(*x))?;
                }

                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::U64Array(value) => {
                buffer.write_u8(U64_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_u64(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::F32Array(value) => {
                buffer.write_u8(F32_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_f32(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::F64Array(value) => {
                buffer.write_u8(F64_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_f64(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },

            //------------------------------------------------//
            // Array of complex/specialized nodes.
            //------------------------------------------------//
            NodeType::Coord2dArray(value) => {
                buffer.write_u8(COORD_2D_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| {
                    let v1 = list.write_f32(x.x);
                    let v2 = list.write_f32(x.y);
                    if v1.is_err() { v1 } else if v2.is_err() { v2 } else { v1 }
                })?;

                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },
            NodeType::Coord3dArray(value) => {
                buffer.write_u8(COORD_3D_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| {
                    let v1 = list.write_f32(x.x);
                    let v2 = list.write_f32(x.y);
                    let v3 = list.write_f32(x.z);
                    if v1.is_err() { v1 } else if v2.is_err() { v2 } else if v3.is_err() { v3 } else { v1 }
                })?;

                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },

            //------------------------------------------------//
            // Array of string nodes.
            //------------------------------------------------//
            NodeType::Utf16Array(value) => {
                buffer.write_u8(UTF16_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|y| {
                    list.write_u32(strings_utf16.iter().position(|x| x == y).unwrap() as u32)
                })?;

                buffer.write_u32(list.len() as u32)?;
                buffer.write_all(&list)?;
            },
            NodeType::AsciiArray(value) => {
                buffer.write_u8(ASCII_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|y| {
                    list.write_u32(strings_utf8.iter().position(|x| x == y).unwrap() as u32)
                })?;

                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },

            //------------------------------------------------//
            // Array of unknown nodes.
            //------------------------------------------------//
            NodeType::AngleArray(value) => {
                buffer.write_u8(ANGLE_ARRAY)?;

                let mut list = vec![];
                value.iter().try_for_each(|x| list.write_i16(*x))?;
                buffer.write_cauleb128(list.len() as u32, 0)?;
                buffer.write_all(&list)?;
            },

            //------------------------------------------------//
            // Record nodes.
            //------------------------------------------------//
            NodeType::Record(value) => {
                if value.record_flags().contains(RecordNodeFlags::HAS_NON_OPTIMIZED_INFO) || is_root_node {
                    buffer.write_u8(value.record_flags().bits())?;

                    buffer.write_u16(record_names.iter().position(|x| x == &value.name).unwrap() as u16)?;
                    buffer.write_u8(value.version)?;
                }

                // If it's not the root node or uses optimized data, it needs special encoding.
                else {
                    let mut info: u16 = (value.record_flags().bits() as u16) << 8;
                    info |= (value.version as u16) << 9;
                    info |= record_names.iter().position(|x| x == &value.name).unwrap() as u16;

                    buffer.write_u16(info.swap_bytes())?;
                }

                let mut children_data = vec![];

                if value.record_flags.contains(RecordNodeFlags::HAS_NESTED_BLOCKS) {
                    for group_node in &value.children {
                        let mut group_node_data = vec![];
                        for node in group_node {
                            Self::save_node(&mut group_node_data, node, false, record_names, strings_utf8, strings_utf16)?;
                        }

                        children_data.write_cauleb128(group_node_data.len() as u32, 0)?;
                        children_data.extend_from_slice(&group_node_data);
                    }

                    buffer.write_cauleb128(children_data.len() as u32, 0)?;
                    buffer.write_cauleb128(value.children.len() as u32, 0)?;
                } else {

                    // For non-nested nodes, we just get the first and only children group.
                    if let Some(children) = value.children.first() {
                        for node in children {
                            Self::save_node(&mut children_data, node, false, record_names, strings_utf8, strings_utf16)?;
                        }
                    }

                    buffer.write_cauleb128(children_data.len() as u32, 0)?;
                }
                buffer.write_all(&children_data)?;
            },
        }

        Ok(())
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
            NodeType::Utf16Array(value) => value.iter().for_each(|value| if !strings_utf16.contains(value) { strings_utf16.push(value.to_owned()) }),
            NodeType::AsciiArray(value) => value.iter().for_each(|value| if !strings_utf8.contains(value) { strings_utf8.push(value.to_owned()) }),
            NodeType::Record(value) => {
                if !record_names.contains(&value.name) {
                    record_names.push(value.name.to_owned());
                }
                for node_group in &value.children {
                    for node in node_group {
                        Self::read_string_from_node(node, record_names, strings_utf8, strings_utf16);
                    }
                }
            },

            // Skip any other node.
            _ => {}
        }
    }
}

