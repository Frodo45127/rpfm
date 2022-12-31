//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! AnimPacks are a container-type file, that usually contains anim-related files,
//! such as [Anims Tables](crate::files::anims_table::AnimsTable),
//! [Anim Fragments](crate::files::anim_fragment::AnimFragment) and
//! [Matched Combat Tables](crate::files::matched_combat::MatchedCombat).

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

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
///
/// AnimPacks are a container-type file, that usually contains anim-related files, such as Anim Tables,
/// Anim Fragments and Matched Combat Tables.
///
/// It's usually found in the `anim` folder of the game, under the extension `.animpack`, hence their name.
///
/// # AnimPack Structure
///
/// | Bytes          | Type                         | Data                                    |
/// | -------------- | ---------------------------- | --------------------------------------- |
/// | 4              | [u32]                        | File Count.                             |
/// | X * File Count | [File](#file-structure) List | List of files inside the AnimPack File. |
///
///
/// # File Structure
///
/// | Bytes       | Type           | Data                  |
/// | ----------- | -------------- | --------------------- |
/// | *           | Sized StringU8 | File Path.            |
/// | 4           | [u32]          | File Length in bytes. |
/// | File Lenght | &\[[u8]\]      | File Data.            |
///
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AnimPack {

    /// File Path on disk of this AnimPack.
    disk_file_path: String,

    /// Offset of this file in the disk file. If the file is not inside another file, it's 0.
    disk_file_offset: u64,

    /// Timestamp of the file. Needed for detecting edits on disk outside our control, in case
    /// we use LazyLoading.
    local_timestamp: u64,

    /// List of files within this AnimPack.
    files: HashMap<String, RFile>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

impl Container for AnimPack {

    /// This function returns a reference to the path on disk of this AnimPack.
    /// If the AnimPack is not yet a file on disk, you may put an empty string.
    ///
    /// Just remember to update it once you save the file to disk.
    fn disk_file_path(&self) -> &str {
       &self.disk_file_path
    }

    /// This function returns a reference to the files inside this AnimPack.
    fn files(&self) -> &HashMap<String, RFile> {
        &self.files
    }

    /// This function returns a mutable reference to the files inside this AnimPack.
    fn files_mut(&mut self) -> &mut HashMap<String, RFile> {
        &mut self.files
    }

    /// This function returns the offset of this AnimPack on the corresponding file on disk.
    ///
    /// If the AnimPack hasn't yet be saved to disk or it's not within another file, this returns 0.
    fn disk_file_offset(&self) -> u64 {
       self.disk_file_offset
    }

    /// This method returns the `Last modified date` the filesystem reports for the container file, in seconds.
    fn local_timestamp(&self) -> u64 {
        self.local_timestamp
    }
}

impl Decodeable for AnimPack {

    /// This function allow us to decode something implementing [ReadBytes](crate::binary::ReadBytes), like a [File]
    /// or a [Vec]<[u8]> into an structured AnimPack.
    ///
    /// About [extra_data](crate::files::DecodeableExtraData), this decode function requires the following fields:
    /// - `lazy_load`: If we want to use Lazy-Loading. If the files within this AnimPack are encrypted, this is ignored.
    /// - `is_encrypted`: If this AnimPack's data is encrypted. If it is, `lazy_load` is ignored.
    /// - `disk_file_path`: If provided, it must correspond to a valid file on disk.
    /// - `disk_file_offset`: If the file is within another file, it's the offset where this AnimPack's data starts. If not, it should be 0.
    /// - `disk_file_size`: The size of the data belonging to this AnimPack.
    /// - `timestamp`: `Last modified date` of this AnimPack, in seconds. If the AnimPack is not a disk file, it should be 0.
    ///
    /// ```rust
    ///use std::fs::File;
    ///use std::io::{BufReader, BufWriter, Write};
    ///
    ///use rpfm_lib::binary::ReadBytes;
    ///use rpfm_lib::files::{*, animpack::AnimPack};
    ///use rpfm_lib::utils::last_modified_time_from_file;
    ///
    ///let path = "../test_files/test_decode.animpack";
    ///let mut reader = BufReader::new(File::open(path).unwrap());
    ///
    ///let mut decodeable_extra_data = DecodeableExtraData::default();
    ///decodeable_extra_data.set_disk_file_path(Some(path));
    ///decodeable_extra_data.set_data_size(reader.len().unwrap());
    ///decodeable_extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref()).unwrap());
    ///
    ///let data = AnimPack::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();
    /// ```
    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;

        // If we're reading from a file on disk, we require a valid path.
        // If we're reading from a file on memory, we don't need a valid path.
        let disk_file_path = match extra_data.disk_file_path {
            Some(path) => {
                let file_path = PathBuf::from_str(path).map_err(|_|RLibError::DecodingMissingExtraDataField("disk_file_path".to_owned()))?;
                if file_path.is_file() {
                    path.to_owned()
                } else {
                    return Err(RLibError::DecodingMissingExtraData)
                }
            }
            None => String::new()
        };

        let disk_file_offset = extra_data.disk_file_offset;
        let disk_file_size = extra_data.data_size;
        let local_timestamp = extra_data.timestamp;
        let is_encrypted = extra_data.is_encrypted;

        // If we don't have a path, or the file is encrypted, we can't lazy-load.
        let lazy_load = !disk_file_path.is_empty() && !is_encrypted && extra_data.lazy_load;
        let file_count = data.read_u32()?;

        let mut anim_pack = Self {
            disk_file_path,
            disk_file_offset,
            local_timestamp,
            files: if file_count < 50_000 { HashMap::with_capacity(file_count as usize) } else { HashMap::new() },
        };

        for _ in 0..file_count {
            let path_in_container = data.read_sized_string_u8()?;
            let size = data.read_u32()?;

            // Encrypted files cannot be lazy-loaded. They must be read in-place.
            if !lazy_load || is_encrypted {
                let data = data.read_slice(size as usize, false)?;
                let file = RFile {
                    path: path_in_container.to_owned(),
                    timestamp: None,
                    file_type: FileType::AnimPack,
                    data: RFileInnerData::Cached(data),
                };

                anim_pack.files.insert(path_in_container, file);
            }

            // Unencrypted and files are not read, but lazy-loaded, unless specified otherwise.
            else {
                let data_pos = data.stream_position()? - disk_file_offset;
                let file = RFile::new_from_container(&anim_pack, size as u64, false, None, data_pos, local_timestamp, &path_in_container)?;
                data.seek(SeekFrom::Current(size as i64))?;

                anim_pack.files.insert(path_in_container, file);
            }
        }

        check_size_mismatch(data.stream_position()? as usize - anim_pack.disk_file_offset as usize, disk_file_size as usize)?;
        Ok(anim_pack)
    }
}

impl Encodeable for AnimPack {

    /// This function allow us to encode an structured AnimPack into something implementing
    /// [WriteBytes](crate::binary::WriteBytes), like a [File] or a [Vec]<[u8]>.
    ///
    /// About [extra_data](crate::files::EncodeableExtraData), its not used in this implementation, so pass a [None].
    ///
    /// ```rust
    ///use std::fs::File;
    ///use std::io::{BufReader, BufWriter, Write};
    ///
    ///use rpfm_lib::binary::ReadBytes;
    ///use rpfm_lib::files::{*, animpack::AnimPack};
    ///
    ///let mut data = AnimPack::default();
    ///let mut encoded = vec![];
    ///data.encode(&mut encoded, &None).unwrap();
    ///
    ///let path = "../test_files/test_encode.animpack";
    ///let mut writer = BufWriter::new(File::create(path).unwrap());
    ///writer.write_all(&encoded).unwrap();
    /// ```
    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.files.len() as u32)?;

        let mut sorted_files = self.files.iter_mut().collect::<Vec<(&String, &mut RFile)>>();
        sorted_files.sort_unstable_by_key(|(path, _)| path.to_lowercase());

        for (path, file) in sorted_files {
            buffer.write_sized_string_u8(path)?;

            let data = file.encode(extra_data, false, false, true)?.unwrap();

            // Error on files too big for the AnimPack.
            if data.len() > u32::MAX as usize {
                return Err(RLibError::DataTooBigForContainer("AnimPack".to_owned(), u32::MAX as u64, data.len(), path.to_owned()));
            }

            buffer.write_u32(data.len() as u32)?;
            buffer.write_all(&data)?;
        }

        Ok(())
    }
}
