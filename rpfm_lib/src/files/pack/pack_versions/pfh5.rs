//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the Pack functions that are specific to PFH5 Packs.
//!
//! All the functions here are internal, so they should be either private or
//! public only within this crate.

use std::io::{BufReader, Cursor, prelude::*};

use crate::binary::ReadBytes;
use crate::encryption::Decryptable;
use crate::error::{RLibError, Result};
use crate::games::pfh_version::PFHVersion;
use crate::files::{OnDisk, pack::*,RFileInnerData, RFile};

impl Pack {

    /// This function reads a `Pack` of version 5 from raw data, returning the index where it finished reading.
    pub(crate) fn read_pfh5<R: ReadBytes>(&mut self, data: &mut R) -> Result<u64> {
        let data_len = data.len()?;

        // Read the info about the indexes to use it later.
        let packs_count = data.read_u32()?;
        let packs_index_size = data.read_u32()?;
        let files_count = data.read_u32()?;
        let files_index_size = data.read_u32()?;

        // The rest of the header data depends on certain flags. Check them to see what parts of the header
        // are left to read.
        let extra_header_size = {
            if (self.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && data_len < 48) ||
                (!self.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && data_len < 28) {
                return Err(RLibError::PackFileHeaderNotComplete);
            }

            if self.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { 24 } else { 4 }
        };

        // Optimization: we only really need the header of the Pack, not the data, and reads, if performed from disk, are expensive.
        // So we get all the data from the header to the end of the indexes to memory and put it in a buffer, so we can read it faster.
        let buffer_data = data.read_slice((extra_header_size as u64 + packs_index_size as u64 + files_index_size as u64) as usize, true)?;
        let mut buffer_mem = BufReader::new(Cursor::new(buffer_data));
        self.header.timestamp = i64::from(buffer_mem.read_u32()?);

        // Check that the position of the data we want to get is actually valid.
        let mut data_pos = data.stream_position()? + buffer_mem.stream_position()? + packs_index_size as u64 + files_index_size as u64;

        // If the Pack data is encrypted and it's PFH5, due to how the encryption works the data should start in a multiple of 8.
        if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) &&
            self.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) &&
            self.header.pfh_version == PFHVersion::PFH5 {
            data_pos = if (data_pos % 8) > 0 { data_pos + 8 - (data_pos % 8) } else { data_pos };
        }

        if data_len < data_pos {
            return Err(RLibError::PackFileIndexesNotComplete)
        }

        // Get the Packs this Pack depends on, if any.
        for _ in 0..packs_count {
            self.dependencies.push(buffer_mem.read_string_u8_0terminated()?);
        }

        // Get the Files in the Pack.
        for files_to_read in (0..files_count).rev() {

            // Get his size. If it's encrypted, decrypt it first.
            let size = if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                buffer_mem.decrypt_u32(files_to_read as u32)?
            } else {
                buffer_mem.read_u32()?
            };

            // Some Packs keep the timestamps of their files. If we have them, get them.
            let timestamp = i64::from(if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                    buffer_mem.decrypt_u32(files_to_read as u32)?
                } else { buffer_mem.read_u32()? }
            } else { 0 });

            // Get if the file is compressed or not.
            let is_compressed = buffer_mem.read_bool()?;

            // Get the file's path. If it's encrypted, decrypt it first.
            let path = if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                buffer_mem.decrypt_string(size as u8)?
            } else {
                buffer_mem.read_string_u8_0terminated()?
            };

            // Build the File as a LazyLoaded file by default.
            let on_disk = OnDisk {
                path,
                start: data_pos,
                size,
                is_compressed,
                is_encrypted: if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(self.header.pfh_version) } else { None },
            };

            let file: RFile = RFile {
                path: on_disk.path.to_owned(),
                timestamp: if timestamp == 0 { None } else { Some(timestamp) },
                data: RFileInnerData::OnDisk(on_disk)
            };

            // Add it to the list.
            self.files.insert(file.path_raw().to_owned(), file);
/*

            // If this is a notes PackedFile, save the notes and forget about the PackedFile. Otherwise, save the PackedFile.
            if packed_file.get_path() == [RESERVED_NAME_NOTES] {
                if let Ok(data) = packed_file.get_raw_data_and_keep_it() {
                    if let Ok(data) = data.decode_string_u8(0, data.len()) {
                        self.notes = Some(data);
                    }
                }
            }

            else if packed_file.get_path() == [RESERVED_NAME_SETTINGS] {
                if let Ok(data) = packed_file.get_raw_data_and_keep_it() {
                    self.settings = if let Ok(settings) = PackFileSettings::load(&data) {
                        settings
                    } else {
                        PackFileSettings::default()
                    };
                }
            }
            else {
                self.packed_files.push(packed_file);
            }
*/
            // Then we move our data position. For encrypted files in PFH5 Packs (only ARENA) we have to start the next one in a multiple of 8.
            if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) &&
                self.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) &&
                self.header.pfh_version == PFHVersion::PFH5 {
                let padding = 8 - (size % 8);
                let padded_size = if padding < 8 { size + padding } else { size };
                data_pos += u64::from(padded_size);
            } else {
                data_pos += u64::from(size);
            }
        }

        // Return our PackFile.
        Ok(data_pos)
    }
}
