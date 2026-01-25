//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! CAAB format implementation for ESF files.
//!
//! CAAB is an older ESF format identified by the magic bytes `0xCA 0xAB 0x00 0x00`.
//! The primary difference from CBAB is that string sizes use u16 prefixes instead of u32.
//!
//! # File Layout (CAAB)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Header (16 bytes)                                           │
//! │   [0..4]   Signature: 0xCA 0xAB 0x00 0x00                   │
//! │   [4..8]   Unknown (u32, typically 0)                       │
//! │   [8..12]  Creation date (u32)                              │
//! │   [12..16] Offset to string tables (u32)                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Node Tree                                                   │
//! │   Recursive tree of nodes starting from root record         │
//! │   Each node: type marker (1 byte) + type-specific data      │
//! ├─────────────────────────────────────────────────────────────┤
//! │ String Tables (at offset specified in header)               │
//! │   Record names: u16 count + [u8-sized strings]              │
//! │   UTF-16 strings: u32 count + [u16-sized string + u32 idx]  │
//! │   UTF-8 strings: u32 count + [u8-sized string + u32 idx]    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Compression Handling
//!
//! CAAB files may contain LZMA1-compressed sections for large data blocks.
//! When a record named `CAMPAIGN_ENV` is encountered during encoding, the entire
//! ESF is re-encoded without compression, then LZMA1-compressed, and stored in
//! special `COMPRESSED_DATA` and `COMPRESSED_DATA_INFO` nodes.
//!
//! During decoding, if a `COMPRESSED_DATA` node is found, the data is decompressed
//! and the resulting ESF replaces the outer structure.

use std::collections::BTreeMap;
use std::io::{Cursor, SeekFrom, Write};

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::{Compressible, CompressionFormat, Decompressible};
use crate::error::{RLibError, Result};

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

impl ESF {

    /// Decodes CAAB-format ESF data into this ESF instance.
    ///
    /// This function assumes the caller has already read and validated the 4-byte
    /// signature. It reads the remaining header fields, parses the string tables,
    /// and recursively decodes the node tree.
    ///
    /// # Decoding Process
    ///
    /// 1. Read header: unknown field, creation date, string table offset
    /// 2. Seek to string table offset and read:
    ///    - Record names (used by record nodes)
    ///    - UTF-16 strings (referenced by string nodes)
    ///    - UTF-8 strings (referenced by ASCII nodes)
    /// 3. Seek back to node data and recursively decode the root node
    /// 4. If compressed data is detected, decompress and replace the ESF content
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Read operations fail
    /// - Node tree doesn't end exactly at the string table offset
    /// - String/record name indices are out of bounds
    /// - Decompression fails (for compressed ESF files)
    pub(crate) fn read_caab<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        // Note: this assumes the caller has already read the first 4 bytes of the data.
        self.unknown_1 = data.read_u32()?;
        self.creation_date = data.read_u32()?;
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
        self.root_node = Self::read_node(data, true, &record_names, &strings_utf8, &strings_utf16)?;

        // If we're not at the exact end of the nodes, something failed.
        let curr_pos = data.stream_position()?;
        if curr_pos != record_names_offset as u64 {
            return Err(RLibError::DecodingMismatchSizeError(record_names_offset as usize, curr_pos as usize));
        }

        // Once we're done with the nodes, we need to check if the last children of the root node contains a compressed record.
        // If so, that record will contain an entire ESF which we need to decode, and then replace ours with that one.
        //
        // The reason for this is, I guess, optimization. Some ESF files, specially the startpos ones, may have specific nodes that are enormous.
        // By keeping a compressed copy of the startpos, the game can read all the other nodes without loading the big ones to memory. But it's just a guess.
        if let NodeType::Record(ref mut node) = self.root_node {
            if let Some(child) = node.children_mut().get_mut(0) {
                if let Some(NodeType::Record(cnode)) = child.last_mut() {
                    if cnode.name == COMPRESSED_DATA_TAG {
                        let mut dec_data = vec![];
                        if let Some(NodeType::U8Array(data)) = cnode.children()[0].first() {
                            if let Some(NodeType::Record(hnode)) = cnode.children()[0].get(1) {
                                if hnode.name == COMPRESSED_DATA_INFO_TAG {
                                    if let Some(NodeType::U32(len)) = hnode.children()[0].first() {
                                        if let Some(NodeType::U8Array(magic_number)) = hnode.children()[0].get(1) {

                                            let mut mdata = vec![];
                                            mdata.write_u32(*len.value())?;
                                            mdata.write_all(magic_number)?;
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
                            *self = new_esf;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Encodes this ESF instance to CAAB format.
    ///
    /// # Encoding Process
    ///
    /// 1. Check for compressible nodes (e.g., `CAMPAIGN_ENV`) and handle compression
    /// 2. Collect all strings from the node tree into separate tables
    /// 3. Encode the node tree using collected string indices
    /// 4. Write header with calculated string table offset
    /// 5. Write encoded nodes followed by string tables
    ///
    /// # Compression
    ///
    /// If a `CAMPAIGN_ENV` record is found and compression is not disabled:
    /// - The entire ESF is first encoded without compression
    /// - The result is LZMA1-compressed
    /// - The original node is replaced with `COMPRESSED_DATA` and `COMPRESSED_DATA_INFO` nodes
    /// - After encoding, the original structure is restored
    ///
    /// # Arguments
    ///
    /// * `buffer` - Output buffer to write encoded data to
    /// * `extra_data` - Optional encoding settings (e.g., `disable_compression`)
    ///
    /// # Errors
    ///
    /// Returns an error if write operations fail or compression fails.
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
                        NodeType::Record(Box::new(hnode)),
                    ]],
                };

                // Replace the full node with the compressed one.
                if let NodeType::Record(ref mut root_node) = self.root_node {
                    if let Some(parent) = root_node.children_mut().get_mut(i1) {
                        if let Some(child) = parent.get_mut(i2) {
                            *child = NodeType::Record(Box::new(cnode));
                        }
                    }
                }
            }
        }

        // Encode the header info, except the offsets, because those are calculated later.
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
}
