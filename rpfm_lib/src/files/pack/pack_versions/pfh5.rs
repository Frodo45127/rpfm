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
    /*
    /// This function writes a `Pack` of version 5 into the provided buffer.
    pub(crate) fn write_pfh5<W: WriteBytes>(&mut self, data: &mut W) -> Result<()> {

        // If any of the problematic masks in the header is set or is one of CA's, return an error.
        if !self.is_editable(*SETTINGS.read().unwrap().settings_bool.get("allow_editing_of_ca_packfiles").unwrap()) { return Err(ErrorKind::PackFileIsNonEditable.into()) }

        // If we receive a new path, update it. Otherwise, ensure the file actually exists on disk.
        if let Some(path) = new_path { self.set_file_path(&path)?; }
        else if !self.get_file_path().is_file() { return Err(ErrorKind::PackFileIsNotAFile.into()) }

        // We ensure that all the data is loaded and in his right form (compressed/encrypted) before attempting to save.
        // We need to do this here because we need later on their compressed size.
        for packed_file in &mut self.packed_files {

            // If we decoded it, re-encode it. Otherwise, just load it.
            packed_file.encode()?;

            // Remember: first compress (only PFH5), then encrypt.
            let is_compressible = !matches!(PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false), PackedFileType::DB | PackedFileType::Loc);
            let (_, data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted) = packed_file.get_ref_mut_raw().get_data_and_info_from_memory()?;

            // If, in any moment, we enabled/disabled the PackFile compression, compress/decompress the PackedFile. EXCEPT FOR TABLES. NEVER COMPRESS TABLES.
            if !is_compressible {
                *should_be_compressed = false;
            }

            if *should_be_compressed && !*is_compressed {
                *data = compress_data(data)?;
                *is_compressed = true;
            }
            else if !*should_be_compressed && *is_compressed {
                *data = decompress_data(data)?;
                *is_compressed = false;
            }

            // Encryption is not yet supported. Decrypt everything.
            if is_encrypted.is_some() {
                *data = decrypt_packed_file(data);
                *is_encrypted = None;
                *should_be_encrypted = None;
            }
        }

        // Only do this in non-vanilla files.
        if self.pfh_file_type == PFHFileType::Mod || self.pfh_file_type == PFHFileType::Movie {

            // Save notes, if needed.
            if let Some(note) = &self.notes {
                let mut data = vec![];
                data.encode_string_u8(note);
                let raw_data = RawPackedFile::read_from_vec(vec![RESERVED_NAME_NOTES.to_owned()], self.get_file_name(), 0, false, data);
                let packed_file = PackedFile::new_from_raw(&raw_data);
                self.packed_files.push(packed_file);
            }

            // Saving PackFile settings.
            let mut data = vec![];
            data.write_all(to_string_pretty(&self.settings)?.as_bytes())?;
            let raw_data = RawPackedFile::read_from_vec(vec![RESERVED_NAME_SETTINGS.to_owned()], self.get_file_name(), 0, false, data);
            let packed_file = PackedFile::new_from_raw(&raw_data);
            self.packed_files.push(packed_file);
        }

        // For some bizarre reason, if the PackedFiles are not alphabetically sorted they may or may not crash the game for particular people.
        // So, to fix it, we have to sort all the PackedFiles here by path.
        // NOTE: This sorting has to be CASE INSENSITIVE. This means for "ac", "Ab" and "aa" it'll be "aa", "Ab", "ac".
        self.packed_files.sort_unstable_by_key(|a| a.get_path().join("\\").to_lowercase());

        // First we encode the indexes and the data (just in case we compressed it).
        let mut pack_file_index = vec![];
        let mut packed_file_index = vec![];

        for pack_file in &self.pack_files {
            pack_file_index.extend_from_slice(pack_file.as_bytes());
            pack_file_index.push(0);
        }

        for packed_file in &self.packed_files {
            packed_file_index.encode_integer_u32(packed_file.get_ref_raw().get_size());

            // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
            // In PFH5 case, we don't support saving encrypted PackFiles for Arena. So we'll default to Warhammer 2 format.
            match self.pfh_version {
                PFHVersion::PFH6 | PFHVersion::PFH5 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_u32(packed_file.get_ref_raw().get_timestamp() as u32); }
                    if packed_file.get_ref_raw().get_should_be_compressed() { packed_file_index.push(1); } else { packed_file_index.push(0); }
                }
                PFHVersion::PFH4 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_u32(packed_file.get_ref_raw().get_timestamp() as u32); }
                }
                PFHVersion::PFH3 | PFHVersion::PFH2 => {
                    if self.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { packed_file_index.encode_integer_i64(packed_file.get_ref_raw().get_timestamp()); }
                }

                // This one doesn't have timestamps, so we just skip this step.
                PFHVersion::PFH0 => {}
            }

            packed_file_index.append(&mut packed_file.get_path().join("\\").as_bytes().to_vec());
            packed_file_index.push(0);
        }

        // Create the file to save to, and save the header and the indexes.
        let mut file = BufWriter::new(File::create(&self.file_path)?);

        // Write the entire header.
        let mut header = vec![];
        header.encode_string_u8(self.pfh_version.get_value());
        header.encode_integer_u32(self.bitmask.bits | self.pfh_file_type.get_value());
        header.encode_integer_u32(self.pack_files.len() as u32);
        header.encode_integer_u32(pack_file_index.len() as u32);
        header.encode_integer_u32(self.packed_files.len() as u32);
        header.encode_integer_u32(packed_file_index.len() as u32);

        // Update the creation time, then save it. PFH0 files don't have timestamp in the headers.
        self.timestamp = get_current_time();
        match self.pfh_version {
            PFHVersion::PFH6 | PFHVersion::PFH5 | PFHVersion::PFH4 => header.encode_integer_u32(self.timestamp as u32),
            PFHVersion::PFH3 | PFHVersion::PFH2 => header.encode_integer_i64((self.timestamp + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK),
            PFHVersion::PFH0 => {}
        };

        if let PFHVersion::PFH6 = self.pfh_version {
            header.encode_integer_u32(SUBHEADER_MARK);
            header.encode_integer_u32(SUBHEADER_VERSION);

            // Just in case the PackFile is not up-to-date, we update it.
            if let Ok(version_number) = GAME_SELECTED.read().unwrap().get_game_selected_exe_version_number() {
                self.set_game_version(version_number);
            }

            header.encode_integer_u32(self.game_version);
            header.encode_integer_u32(self.build_number);

            // Save it as "Made By CA" if the debug setting for it is enabled.
            if SETTINGS.read().unwrap().settings_bool["spoof_ca_authoring_tool"] {
                self.set_authoring_tool(AUTHORING_TOOL_CA)?;
            }

            header.encode_string_u8_0padded(&(self.authoring_tool.to_owned(), 8))?;
            header.extend_from_slice(&self.extra_subheader_data);
        }

        // Write the indexes and the data of the PackedFiles. No need to keep the data, as it has been preloaded before.
        file.write_all(&header)?;
        file.write_all(&pack_file_index)?;
        file.write_all(&packed_file_index)?;
        for packed_file in &self.packed_files {
            let data = packed_file.get_ref_raw().get_raw_data()?;
            file.write_all(&data)?;
        }

        // Remove again the reserved PackedFiles.
        self.remove_packed_file_by_path(&[RESERVED_NAME_NOTES.to_owned()]);
        self.remove_packed_file_by_path(&[RESERVED_NAME_SETTINGS.to_owned()]);

        // If nothing has failed, return success.
        Ok(())
    }*/
}
