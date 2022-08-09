//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the Pack functions that are specific to PFH6 Packs.
//!
//! All the functions here are internal, so they should be either private or
//! public only within this crate.

use std::io::{BufReader, Cursor};

use crate::binary::{ReadBytes, WriteBytes};
use crate::encryption::Decryptable;
use crate::error::{RLibError, Result};
use crate::files::{pack::*, RFile};

use super::*;

impl Pack {

    /// This function reads a `Pack` of version 6 from raw data, returning the index where it finished reading.
    pub(crate) fn read_pfh6<R: ReadBytes>(&mut self, data: &mut R, extra_data: &DecodeableExtraData) -> Result<u64> {
        let data_len = if extra_data.data_size > 0 { extra_data.data_size } else { data.len()? };

        // Read the info about the indexes to use it later.
        let packs_count = data.read_u32()?;
        let packs_index_size = data.read_u32()?;
        let files_count = data.read_u32()?;
        let files_index_size = data.read_u32()?;

        self.header.internal_timestamp = u64::from(data.read_u32()?);
        self.files = HashMap::with_capacity(files_count as usize);

        // The rest of the header data is 280 bytes.
        let extra_header_size = 280;

        // Optimization: we only really need the header of the Pack, not the data, and reads, if performed from disk, are expensive.
        // So we get all the data from the header to the end of the indexes to memory and put it in a buffer, so we can read it faster.
        let indexes_size = extra_header_size + packs_index_size + files_index_size;
        let buffer_data = data.read_slice(indexes_size as usize, false)?;
        let mut buffer_mem = BufReader::new(Cursor::new(buffer_data));

        // Check that the position of the data we want to get is actually valid.
        let data_pos = data.stream_position()? - extra_data.disk_file_offset;
        if data_len < data_pos {
            return Err(RLibError::PackFileIndexesNotComplete)
        }

        // Check if we actually have the subheader we expect.
        let subheader_marker = buffer_mem.read_u32()?;
        if subheader_marker != SUBHEADER_MARK {
            return Err(RLibError::PackFileSubHeaderMissing)
        }
        let _subheader_version = buffer_mem.read_u32()?;

        self.header.game_version = buffer_mem.read_u32()?;
        self.header.build_number = buffer_mem.read_u32()?;
        self.header.authoring_tool = buffer_mem.read_string_u8_0padded(AUTHORING_TOOL_SIZE as usize)?;
        self.header.extra_subheader_data = buffer_mem.read_slice(256, false)?;

        // Check that the position of the data we want to get is actually valid.
        let mut data_pos = data.stream_position()? - extra_data.disk_file_offset;
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
            }.replace('\\', "/");

            // Build the File as a LazyLoaded file by default.
            let file = RFile::new_from_container(self, size as u64, is_compressed, files_are_encrypted, data_pos, timestamp, &path)?;
            self.insert(file)?;

            // Then we move our data position.
            data_pos += u64::from(size);
        }

        // Return our current position on the data section for further checks.
        Ok(data_pos)
    }

    /// This function writes a `Pack` of version 6 into the provided buffer.
    pub(crate) fn write_pfh6<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let (sevenzip_exe_path, test_mode) = if let Some(extra_data) = extra_data {
            (extra_data.sevenzip_path, extra_data.test_mode)
        } else {
            (None, false)
        };

        // We need our files sorted before trying to write them. But we don't want to duplicate
        // them on memory. And we also need to load them to memory on the pack. So...  we do this.
        let mut sorted_files = self.files.iter_mut().collect::<Vec<(&String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        // Optimization: we process the sorted files in parallel, so we can speedup loading/compression.
        // Sadly, this requires us to make a double iterator to actually catch the errors.
        let (files_index, files_data): (Vec<_>, Vec<_>) = sorted_files.par_iter_mut()
            .map(|(path, file)| {

                // This unwrap is actually safe.
                let mut data = file.encode(extra_data, false, false, true)?.unwrap();

                if self.compress && file.is_compressible() {
                    if let Some(sevenzip_exe_path) = sevenzip_exe_path {
                        data = data.compress(sevenzip_exe_path)?;
                    }
                }

                // 6 because 4 (size) + 1 (compressed?) + 1 (null), 10 because + 4 (timestamp).
                let file_index_entry_len = if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    10 + path.len()
                } else {
                    6 + path.len()
                };

                let mut file_index_entry = Vec::with_capacity(file_index_entry_len);

                // Error on files too big for the Pack.
                if data.len() > u32::MAX as usize {
                    return Err(RLibError::DataTooBigForContainer("Pack".to_owned(), u32::MAX as u64, data.len(), path.to_owned()));
                }

                file_index_entry.write_u32(data.len() as u32)?;

                if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    file_index_entry.write_u32(file.timestamp().unwrap_or(0) as u32)?;
                }

                file_index_entry.write_bool(self.compress && file.is_compressible())?;
                file_index_entry.write_string_u8_0terminated(&path.replace('/', "\\"))?;
                Ok((file_index_entry, data))
            }).collect::<Result<Vec<(Vec<u8>, Vec<u8>)>>>()?
            .into_par_iter()
            .unzip();

        let files_index = files_index.into_par_iter().flatten().collect::<Vec<_>>();
        let files_data = files_data.into_par_iter().flatten().collect::<Vec<_>>();

        // Build the dependencies index on memory. This one is never big, so no need of par_iter.
        let mut dependencies_index = vec![];
        for dependency in &self.dependencies {
            dependencies_index.write_string_u8_0terminated(dependency)?;
        }

        // Write the entire header to a memory buffer.
        let mut header = vec![];
        header.write_string_u8(self.header.pfh_version.value())?;
        header.write_u32(self.header.bitmask.bits | self.header.pfh_file_type.value())?;
        header.write_u32(self.dependencies.len() as u32)?;
        header.write_u32(dependencies_index.len() as u32)?;
        header.write_u32(sorted_files.len() as u32)?;
        header.write_u32(files_index.len() as u32)?;

        // If we're not in testing mode, update the header timestamp.
        if !test_mode {
            self.header.internal_timestamp = current_time()?;
        }

        header.write_u32(self.header.internal_timestamp as u32)?;
        header.write_u32(SUBHEADER_MARK)?;
        header.write_u32(SUBHEADER_VERSION)?;
        header.write_u32(self.header.game_version)?;
        header.write_u32(self.header.build_number)?;
        header.write_string_u8_0padded(&self.header.authoring_tool, 8, false)?;
        header.write_all(&self.header.extra_subheader_data)?;

        // Finally, write everything in one go.
        buffer.write_all(&header)?;
        buffer.write_all(&dependencies_index)?;
        buffer.write_all(&files_index)?;
        buffer.write_all(&files_data)?;

        Ok(())
    }
}
