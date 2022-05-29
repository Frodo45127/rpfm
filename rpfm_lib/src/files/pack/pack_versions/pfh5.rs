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

use crate::binary::{ReadBytes, WriteBytes};
use crate::encryption::Decryptable;
use crate::error::{RLibError, Result};
use crate::games::pfh_version::PFHVersion;
use crate::files::{pack::*, RFile};

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
        self.header.timestamp = u64::from(buffer_mem.read_u32()?);

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

        // Get if the files are encrypted or not.
        let files_are_encrypted = if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) {
            Some(self.header.pfh_version)
        } else {
            None
        };

        // Get the Files in the Pack.
        for files_to_read in (0..files_count).rev() {

            // Get his size. If it's encrypted, decrypt it first.
            let size = if self.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                buffer_mem.decrypt_u32(files_to_read as u32)?
            } else {
                buffer_mem.read_u32()?
            };

            // Some Packs keep the timestamps of their files. If we have them, get them.
            let timestamp = u64::from(if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
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
            let file = RFile::new_from_container(self, size, is_compressed, files_are_encrypted, data_pos, timestamp, &path);
            self.add_file(file)?;

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

    /// This function writes a `Pack` of version 5 into the provided buffer.
    pub(crate) fn write_pfh5<W: WriteBytes>(&mut self, buffer: &mut W, sevenzip_exe_path: Option<&Path>, test_mode: bool) -> Result<()> {

        let mut sorted_files = self.files.iter_mut().collect::<Vec<(&String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        let files_data = sorted_files.par_iter_mut().flat_map(|(_, file)| {
            let mut data = file.encode(true, true).unwrap().unwrap();

            if self.compress && file.is_compressible() {
                if let Some(sevenzip_exe_path) = sevenzip_exe_path {
                    data = data.compress(sevenzip_exe_path).unwrap();
                }
            }

            data
        }).collect::<Vec<u8>>();

        // First we encode the indexes and the data (just in case we compressed it).
        let mut dependencies_index = vec![];

        for dependency in &self.dependencies {
            dependencies_index.write_string_u8_0terminated(dependency)?;
        }

        let files_index = sorted_files.par_iter_mut().flat_map(|(path, file)| {
            let mut files_index_entry = Vec::with_capacity(6 + path.len());
            files_index_entry.write_u32(file.size().unwrap() as u32).unwrap();

            // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
            // In PFH5 case, we don't support saving encrypted PackFiles for Arena. So we'll default to Warhammer 2 format.
            if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                files_index_entry.write_u32(file.timestamp().unwrap_or(0) as u32).unwrap();
            }

            files_index_entry.write_bool(self.compress && file.is_compressible()).unwrap();

            // TODO: fix this
            files_index_entry.write_string_u8_0terminated(path).unwrap();
            files_index_entry
        }).collect::<Vec<u8>>();

        // Write the entire header.
        let mut header = vec![];
        header.write_string_u8(self.header.pfh_version.value())?;
        header.write_u32(self.header.bitmask.bits | self.header.pfh_file_type.value())?;
        header.write_u32(self.dependencies.len() as u32)?;
        header.write_u32(dependencies_index.len() as u32)?;
        header.write_u32(sorted_files.len() as u32)?;
        header.write_u32(files_index.len() as u32)?;

        // Update the creation time, then save it. PFH0 files don't have timestamp in the headers.
        if !test_mode {
            self.header.timestamp = current_time()?;
        }

        header.write_u32(self.header.timestamp as u32)?;

        // Write the indexes and the data of the PackedFiles. No need to keep the data, as it has been preloaded before.
        buffer.write_all(&header)?;
        buffer.write_all(&dependencies_index)?;
        buffer.write_all(&files_index)?;
        buffer.write_all(&files_data)?;

        // If nothing has failed, return success.
        Ok(())
    }
}
