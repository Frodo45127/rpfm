//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the Pack functions that are specific to PFH3 Packs.
//!
//! All the functions here are internal, so they should be either private or
//! public only within this crate.

use std::io::{BufReader, Cursor};

use crate::binary::{ReadBytes, WriteBytes};
use crate::compression::{Compressible, CompressionFormat};
use crate::error::{RLibError, Result};
use crate::files::{pack::*, RFile};
use crate::games::supported_games::KEY_SHOGUN_2_COMPLETE_EDITION;

use super::*;

impl Pack {

    /// This function reads a `Pack` of version 3 from raw data, returning the index where it finished reading.
    pub(crate) fn read_pfh3<R: ReadBytes>(&mut self, data: &mut R, extra_data: &DecodeableExtraData) -> Result<u64> {
        let data_len = if extra_data.data_size > 0 { extra_data.data_size } else { data.len()? };

        // Read the info about the indexes to use it later.
        let packs_count = data.read_u32()?;
        let packs_index_size = data.read_u32()?;
        let files_count = data.read_u32()?;
        let files_index_size = data.read_u32()?;

        // Timestamp is a windows specific one, and with this we map it to the unix one.
        self.header.internal_timestamp = (data.read_u64()? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH;
        self.files = HashMap::with_capacity(files_count as usize);

        // Optimization: we only really need the header of the Pack, not the data, and reads, if performed from disk, are expensive.
        // So we get all the data from the header to the end of the indexes to memory and put it in a buffer, so we can read it faster.
        let indexes_size = packs_index_size + files_index_size;
        let buffer_data = data.read_slice(indexes_size as usize, false)?;
        let mut buffer_mem = BufReader::new(Cursor::new(buffer_data));

        // Check that the position of the data we want to get is actually valid.
        let mut data_pos = data.stream_position()? - extra_data.disk_file_offset;
        if data_len < data_pos {
            return Err(RLibError::PackIndexesNotComplete)
        }

        // Get the Packs this Pack depends on, if any. Note that we ignore this list if we have a dependencies manager file.
        for _ in 0..packs_count {
            self.dependencies.push((true, buffer_mem.read_string_u8_0terminated()?));
        }

        let has_aligned_data = self.header.pfh_version_flags.contains(PFHVersionFlags::HAS_ALIGNED_DATA);
        let has_compressed_data = self.header.bitmask.contains(PFHFlags::HAS_COMPRESSED_DATA);
        let mut data_end = data_pos;
        self.compress = has_compressed_data;

        // Get the Files in the Pack.
        for _ in 0..files_count {

            // Aligned Packs store the absolute offset of each file, as there's padding between them.
            if has_aligned_data {
                data_pos = buffer_mem.read_u64()?;
            }

            let uncompressed_size = if has_compressed_data { Some(buffer_mem.read_u32()?) } else { None };
            let size = buffer_mem.read_u32()?;
            let timestamp = if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                (buffer_mem.read_u64()? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH
            } else { 0 };

            let path = buffer_mem.read_string_u8_0terminated()?.replace('\\', "/");

            // Build the File as a LazyLoaded file by default. Files stored uncompressed have the same size on both fields.
            let uncompressed_size = uncompressed_size.filter(|uncompressed_size| *uncompressed_size != size);
            let mut file = RFile::new_from_container(self, size as u64, uncompressed_size.is_some(), None, data_pos, timestamp, &path)?;
            if let Some(uncompressed_size) = uncompressed_size {
                file.set_lz4_block_uncompressed_size(uncompressed_size as u64);
            }

            self.insert(file)?;

            data_pos += u64::from(size);
            data_end = data_end.max(data_pos);
        }

        // Return the end of the data section for further checks.
        Ok(data_end)
    }

    /// This function writes a `Pack` of version 3 into the provided buffer.
    pub(crate) fn write_pfh3<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let (test_mode, nullify_dates, game_info, mut compression_format) = if let Some(extra_data) = extra_data {
            (extra_data.test_mode, extra_data.nullify_dates, extra_data.game_info, extra_data.compression_format)
        } else {
            (false, false, None, CompressionFormat::None)
        };

        // Shogun 2 Complete Edition Packs must always be compressed with Lz4Block.
        if game_info.is_some_and(|game_info| game_info.key() == KEY_SHOGUN_2_COMPLETE_EDITION) {
            compression_format = self.set_compression_format(CompressionFormat::Lz4Block, game_info.unwrap());
        }

        // Only compress if the format is valid for this Pack's version. The index layout depends on it, so update the flag to match.
        let compress = self.compress && compression_format != CompressionFormat::None && self.supports_compression_format(compression_format);
        self.header.bitmask.set(PFHFlags::HAS_COMPRESSED_DATA, compress);

        // We need our files sorted before trying to write them. But we don't want to duplicate
        // them on memory. And we also need to load them to memory on the pack. So...  we do this.
        let mut sorted_files = self.files.iter_mut().map(|(key, file)| (key.replace('/', "\\"), file)).collect::<Vec<(String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        let has_aligned_data = self.header.pfh_version_flags.contains(PFHVersionFlags::HAS_ALIGNED_DATA);

        // Optimization: we process the sorted files in parallel, so we can speedup loading/compression.
        // Sadly, this requires us to make a double iterator to actually catch the errors.
        let (mut files_index, files_data): (Vec<_>, Vec<_>) = sorted_files.par_iter_mut()
            .map(|(path, file)| {

                // This unwrap is actually safe.
                let mut data = file.encode(extra_data, false, false, true)?.unwrap();

                // 5 because 4 (size) + 1 (null), 13 because + 8 (timestamp). Plus 8 (offset) and 4 (uncompressed size) if they apply.
                let mut file_index_entry_len = if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    13 + path.len()
                } else {
                    5 + path.len()
                };

                if has_aligned_data {
                    file_index_entry_len += 8;
                }

                if compress {
                    file_index_entry_len += 4;
                }

                let mut file_index_entry = Vec::with_capacity(file_index_entry_len);

                // Error on files too big for the Pack.
                if data.len() > u32::MAX as usize {
                    return Err(RLibError::DataTooBigForContainer("Pack".to_owned(), u32::MAX as u64, data.len(), path.to_owned()));
                }

                // The offset is not yet known, so we write a placeholder and patch it once we have all the sizes.
                if has_aligned_data {
                    file_index_entry.write_u64(0)?;
                }

                // Files are stored uncompressed if they're not compressible or compression doesn't make them smaller,
                // as that's how we know on read if they're compressed.
                if compress {
                    file_index_entry.write_u32(data.len() as u32)?;

                    if game_info.is_some_and(|game_info| file.is_compressible(game_info)) {
                        if let Ok(data_compressed) = data.compress(compression_format) {
                            if data_compressed.len() < data.len() {
                                data = data_compressed;
                            }
                        }
                    }
                }

                file_index_entry.write_u32(data.len() as u32)?;

                if self.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                    let timestamp = if nullify_dates {
                        0
                    } else {
                        (file.timestamp().unwrap_or(0) + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK
                    };
                    file_index_entry.write_u64(timestamp)?;
                }

                file_index_entry.write_string_u8_0terminated(path)?;
                Ok((file_index_entry, data))
            }).collect::<Result<Vec<(Vec<u8>, Vec<u8>)>>>()?
            .into_par_iter()
            .unzip();

        // Build the dependencies index on memory. This one is never big, so no need of par_iter.
        let mut dependencies_index = vec![];
        let mut dependencies_count = 0;
        for (hard, dependency) in &self.dependencies {
            if *hard {
                dependencies_index.write_string_u8_0terminated(dependency)?;
                dependencies_count += 1;
            }
        }

        // Write the entire header to a memory buffer. Version flags go in the upper bits of the last byte of the signature.
        let mut signature = self.header.pfh_version.value().as_bytes().to_vec();
        signature[3] |= self.header.pfh_version_flags.bits();

        let files_index_len: u32 = files_index.par_iter().map(|x| x.len() as u32).sum();
        let mut header = vec![];
        header.write_all(&signature)?;
        header.write_u32(self.header.bitmask.bits() | self.header.pfh_file_type.value())?;
        header.write_u32(dependencies_count)?;
        header.write_u32(dependencies_index.len() as u32)?;
        header.write_u32(sorted_files.len() as u32)?;
        header.write_u32(files_index_len)?;

        // If we're not in testing mode, update the header timestamp.
        if nullify_dates {
            self.header.internal_timestamp = 0;
        } else if !test_mode {
            self.header.internal_timestamp = current_time()?;
        }

        header.write_u64((self.header.internal_timestamp + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK)?;

        // In aligned Packs each file starts at a 16-byte boundary, so patch the real offsets into the index.
        let mut offsets = Vec::with_capacity(files_data.len());
        if has_aligned_data {
            let mut offset = (header.len() + dependencies_index.len()) as u64 + u64::from(files_index_len);
            for (index, data) in files_index.iter_mut().zip(files_data.iter()) {
                offset = offset.next_multiple_of(16);
                index[..8].copy_from_slice(&offset.to_le_bytes());
                offsets.push(offset);
                offset += data.len() as u64;
            }
        }

        // Finally, write everything in one go.
        buffer.write_all(&header)?;
        buffer.write_all(&dependencies_index)?;

        for index in &files_index {
            buffer.write_all(index)?;
        }

        let mut position = (header.len() + dependencies_index.len()) as u64 + u64::from(files_index_len);
        for (i, data) in files_data.iter().enumerate() {
            if let Some(offset) = offsets.get(i) {
                buffer.write_all(&vec![0; (offset - position) as usize])?;
                position = *offset;
            }

            buffer.write_all(data)?;
            position += data.len() as u64;
        }

        Ok(())
    }
}
