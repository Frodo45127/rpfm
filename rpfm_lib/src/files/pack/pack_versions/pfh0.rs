//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the Pack functions that are specific to PFH0 Packs.
//!
//! All the functions here are internal, so they should be either private or
//! public only within this crate.

use std::io::{BufReader, Cursor};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{pack::*, RFile};

impl Pack {

    /// This function reads a `Pack` of version 0 from raw data, returning the index where it finished reading.
    pub(crate) fn read_pfh0<R: ReadBytes>(&mut self, data: &mut R, extra_data: &DecodeableExtraData) -> Result<u64> {
        let data_len = if extra_data.data_size > 0 { extra_data.data_size } else { data.len()? };

        // Read the info about the indexes to use it later.
        let packs_count = data.read_u32()?;
        let packs_index_size = data.read_u32()?;
        let files_count = data.read_u32()?;
        let files_index_size = data.read_u32()?;

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

        // Get the Files in the Pack.
        for _ in 0..files_count {
            let size = buffer_mem.read_u32()?;
            let path = buffer_mem.read_string_u8_0terminated()?.replace('\\', "/");

            // Build the File as a LazyLoaded file by default.
            let file = RFile::new_from_container(self, size as u64, false, None, data_pos, 0, &path)?;
            self.insert(file)?;

            data_pos += u64::from(size);
        }

        // Return our current position on the data section for further checks.
        Ok(data_pos)
    }

    /// This function writes a `Pack` of version 0 into the provided buffer.
    pub(crate) fn write_pfh0<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {

        // We need our files sorted before trying to write them. But we don't want to duplicate
        // them on memory. And we also need to load them to memory on the pack. So...  we do this.
        let mut sorted_files = self.files.iter_mut().map(|(key, file)| (key.replace('/', "\\"), file)).collect::<Vec<(String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        // Optimization: we process the sorted files in parallel, so we can speedup loading/compression.
        // Sadly, this requires us to make a double iterator to actually catch the errors.
        let (files_index, files_data): (Vec<_>, Vec<_>) = sorted_files.par_iter_mut()
            .map(|(path, file)| {

                // This unwrap is actually safe.
                let data = file.encode(extra_data, false, false, true)?.unwrap();

                // 5 because 4 (size) + 1 (null).
                let file_index_entry_len = 5 + path.len();
                let mut file_index_entry = Vec::with_capacity(file_index_entry_len);

                // Error on files too big for the Pack.
                if data.len() > u32::MAX as usize {
                    return Err(RLibError::DataTooBigForContainer("Pack".to_owned(), u32::MAX as u64, data.len(), path.to_owned()));
                }

                file_index_entry.write_u32(data.len() as u32)?;
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

        // Write the entire header to a memory buffer.
        let mut header = vec![];
        header.write_string_u8(self.header.pfh_version.value())?;
        header.write_u32(self.header.bitmask.bits() | self.header.pfh_file_type.value())?;
        header.write_u32(dependencies_count)?;
        header.write_u32(dependencies_index.len() as u32)?;
        header.write_u32(sorted_files.len() as u32)?;
        header.write_u32(files_index.par_iter().map(|x| x.len() as u32).sum())?;

        // Finally, write everything in one go.
        buffer.write_all(&header)?;
        buffer.write_all(&dependencies_index)?;

        for index in files_index {
            buffer.write_all(&index)?;
        }

        for data in files_data {
            buffer.write_all(&data)?;
        }

        Ok(())
    }
}
