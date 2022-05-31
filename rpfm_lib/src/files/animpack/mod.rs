//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! AnimPacks are a container-type file, that usually contains anim-related files, such as Anim Tables,
//! Anim Fragments and Matched Combat Tables.
//!
//! It's usually found in the `anim` folder of the game, under the extension `.animpack`, hence their name.
//!
//! # AnimPack Structure
//!
//! | Bytes          | Type                         | Data                                    |
//! | -------------- | ---------------------------- | --------------------------------------- |
//! | 4              | [u32]                        | File Count.                             |
//! | X * File Count | [File](#file-structure) List | List of files inside the AnimPack File. |
//!
//!
//! # File Structure
//!
//! | Bytes       | Type      | Data |
//! | ----------- | --------- | ---- |
//! | *           | StringU8  | File Path. |
//! | 4           | [u32]     | File Length in bytes. |
//! | File Lenght | &\[[u8]\] | File Data. |

use std::collections::HashMap;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::*;

/// Extension used by AnimPacks.
pub const EXTENSION: &str = ".animpack";

#[cfg(test)] mod animpack_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire AnimPack file decoded in memory.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct AnimPack {

    /// File Path on disk of this AnimPack.
    disk_file_path: String,

    /// Offset of this file in the disk file.
    disk_file_offset: u64,

    /// Timestamp of the file.
    timestamp: u64,

    /// List of files within this AnimPack.
    files: HashMap<String, RFile>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

impl Container for AnimPack {
    fn disk_file_path(&self) -> &str {
       &self.disk_file_path
    }

    fn files(&self) -> &HashMap<std::string::String, RFile> {
        &self.files
    }

    fn files_mut(&mut self) -> &mut HashMap<std::string::String, RFile> {
        &mut self.files
    }

    fn disk_file_offset(&self) -> u64 {
       self.disk_file_offset
    }

    fn timestamp(&self) -> u64 {
       self.timestamp
    }
}

impl Decodeable for AnimPack {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.ok_or(RLibError::DecodingMissingExtraData)?;
        let disk_file_path = extra_data.disk_file_path.ok_or(RLibError::DecodingMissingExtraData)?;
        let disk_file_offset = extra_data.disk_file_offset;
        let timestamp = extra_data.timestamp;
        let is_encrypted = extra_data.is_encrypted;

        let file_count = data.read_u32()?;

        let mut anim_pack = Self {
            disk_file_path: disk_file_path.to_string(),
            disk_file_offset,
            timestamp,
            files: if file_count < 50_000 { HashMap::with_capacity(file_count as usize) } else { HashMap::new() },
        };

        for _ in 0..file_count {
            let path = data.read_sized_string_u8()?;
            let size = data.read_u32()?;

            // Encrypted files cannot be lazy-loaded. They must be read in-place.
            if is_encrypted {
                let data = data.read_slice(size as usize, false)?;
                let file = RFile {
                    path: path.to_owned(),
                    timestamp: None,
                    file_type: FileType::AnimPack,
                    data: RFileInnerData::Cached(data),
                };

                anim_pack.files.insert(path, file);
            }

            // Unencrypted files are not read, but lazy-loaded.
            else {
                let file = RFile::new_from_container(&anim_pack, size, false, None, data.stream_position()?, timestamp, &path);
                data.seek(SeekFrom::Current(size as i64))?;

                anim_pack.files.insert(path, file);
            }
        }

        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;
        Ok(anim_pack)
    }
}

impl Encodeable for AnimPack {
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: Option<DecodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.files.len() as u32)?;

        let mut sorted_files = self.files.iter_mut().collect::<Vec<(&String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        for (path, file) in sorted_files {
            buffer.write_sized_string_u8(&path)?;

            let data = file.encode(true, true)?.unwrap();
            buffer.write_u32(data.len() as u32)?;
            buffer.write_all(&data)?;
        }

        Ok(())
    }
}
