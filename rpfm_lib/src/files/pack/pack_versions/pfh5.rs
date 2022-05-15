//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with PackFiles.

This module contains all the code related with PackFiles. If you want to do anything with a PackFile,
this is the place you have to come.

Also, something to take into account. RPFM supports PackFile compression/decompression and decryption,
and that is handled automagically by RPFM. All the data you'll ever see will be decompressed/decrypted,
so you don't have to worry about that.
!*/

use std::io::Cursor;
use crate::files::RFileInnerData;

use std::io::{prelude::*, BufReader, SeekFrom};





use crate::{binary::{ReadBytes}, utils::*};
use crate::games::pfh_version::PFHVersion;
use crate::games::pfh_file_type::PFHFileType;

use crate::files::RFile;

use crate::error::{RLibError, Result};


use crate::files::{OnDisk, pack::*};

impl Pack {
    pub fn read_pfh5<R: ReadBytes>(
        data: &mut R,
        use_lazy_loading: bool
    ) -> Result<Self> {
        let t = std::time::SystemTime::now();
        // Check if what we received is even a `PackFile`.
        //if !file_path.file_name().unwrap().to_string_lossy().to_string().ends_with(".pack") { return Err(ErrorKind::OpenPackFileInvalidExtension.into()) }

        // Prepare the PackFile to be read and the virtual PackFile to be written.
        //let mut pack_file = BufReader::new(File::open(&file_path)?);
        //let pack_file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let mut pack = Self::default();

        // First, we do some quick checkings to ensure it's a valid PackFile.
        // 24 is the bare minimum that we need to check how a PackFile should be internally, so any file with less than that is not a valid PackFile.
        let data_len = data.len()?;
        if data_len < 24 {
            return Err(RLibError::PackFileHeaderNotComplete);
        }

        // Check if it has the weird steam-only header, and skip it if found.
        let start = if data.read_string_u8(3)? == MFH_PREAMBLE { 8 } else { 0 };
        data.seek(SeekFrom::Start(start))?;

        // Create a little buffer to read the basic data from the header of the PackFile.

        // Start populating our decoded PackFile struct.
        //pack_file_decoded.file_path = file_path.to_path_buf();
        pack.header.pfh_version = PFHVersion::version(&data.read_string_u8(4)?)?;

        let pack_type = data.read_u32()?;
        pack.header.pfh_file_type = PFHFileType::try_from(pack_type & 15)?;
        pack.header.bitmask = PFHFlags::from_bits_truncate(pack_type & !15);

        // Read the data about the indexes to use it later.
        let packs_count = data.read_u32()?;
        let packs_index_size = data.read_u32()?;
        let files_count = data.read_u32()?;
        let files_index_size = data.read_u32()?;
        dbg!(data.stream_position()?);
        // Depending on the data we got, prepare to read the header and ensure we have all the bytes we need.
        let extra_header_size = match pack.header.pfh_version {

            // PFH6 contains a subheader with some extra data we want to keep.
            PFHVersion::PFH6 => 284,

            PFHVersion::PFH5 | PFHVersion::PFH4 => {
                if (pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && data_len < 48) ||
                    (!pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) && data_len < 28) { return Err(RLibError::PackFileHeaderNotComplete) }

                if pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { 24 } else { 4 }
            }

            PFHVersion::PFH3 | PFHVersion::PFH2 => 8,
            PFHVersion::PFH0 => 0,
        };

        // Restore the cursor of the BufReader to 0, so we can read the full header in one go. The first 24 bytes are
        // already decoded but, for the sake of clarity in the positions of the rest of the header stuff, we do this.
        let mut buffer_mem = BufReader::new(Cursor::new(data.read_slice((extra_header_size as u64 + packs_index_size as u64 + files_index_size as u64) as usize, true)?));
        //data.seek(SeekFrom::Current(extra_header_size))?;
        dbg!(t.elapsed().unwrap());
        // The creation time is a bit of an asshole. Depending on the PackFile Version/Id/Preamble, it uses a type, another or it doesn't exists.
        // Keep in mind that we store his raw value. If you want his legible value, you have to convert it yourself. PFH0 doesn't have it.
        pack.header.timestamp = match pack.header.pfh_version {
            PFHVersion::PFH6 | PFHVersion::PFH5 | PFHVersion::PFH4 => i64::from(buffer_mem.read_u32()?),
            PFHVersion::PFH3 | PFHVersion::PFH2 => (buffer_mem.read_i64()? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
            PFHVersion::PFH0 => 0
        };
        dbg!(buffer_mem.stream_position()?);

        if let PFHVersion::PFH6 = pack.header.pfh_version {
            let _ = buffer_mem.read_u32()?;
            pack.header.game_version = buffer_mem.read_u32()?;
            pack.header.build_number = buffer_mem.read_u32()?;
            pack.header.authoring_tool = buffer_mem.read_string_u8_0padded(AUTHORING_TOOL_SIZE as usize)?;
            pack.header.extra_subheader_data = buffer_mem.read_slice(256, false)?;
        }

        dbg!(buffer_mem.stream_position()?);
        // Ensure the PackFile has all the data needed for the index. If the PackFile's data is encrypted
        // and the PackFile is PFH5, due to how the encryption works, the data should start in a multiple of 8.
        let mut data_position = data.stream_position()? + buffer_mem.stream_position()? + packs_index_size as u64 + files_index_size as u64;
        if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) &&
            pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) &&
            pack.header.pfh_version == PFHVersion::PFH5 {
            data_position = if (data_position % 8) > 0 { data_position + 8 - (data_position % 8) } else { data_position };
        }
        dbg!(data_len);
        dbg!(data_position);
        if data_len < data_position { return Err(RLibError::PackFileIndexesNotComplete) }

        // Create the buffers for the indexes data. This is waaaaay faster than reading from disk.
        //let mut packs_index = vec![0; packs_index_size as usize];
        //let mut files_index = vec![0; files_index_size as usize];

        // Get the data from both indexes to their buffers.
        //pack_file.read_exact(&mut pack_file_index)?;
        //pack_file.read_exact(&mut packed_file_index)?;

        // Read the PackFile Index.
        //let mut pack_file_index_position: usize = 0;

        // First, we decode every entry in the PackFile index and store it. It's encoded in StringU8 terminated in 00,
        // so we just read them char by char until hitting 0, then decode the next one and so on.
        // NOTE: This doesn't deal with encryption, as we haven't seen any encrypted PackFile with data in this index.
        for _ in 0..packs_count {
            let pack_file_name = buffer_mem.read_string_u8_0terminated()?;
            pack.dependencies.push(pack_file_name);
        }

        // Depending on the version of the PackFile and his bitmask, the PackedFile index has one format or another.
        /*let packed_file_index_path_offset = match pack.header.pfh_version {
            PFHVersion::PFH6 | PFHVersion::PFH5 => {

                // If it has the extended header bit, is an Arena PackFile. These ones use a normal PFH4 index format for some reason.
                if pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
                    if pack.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }
                }

                // Otherwise, it's a Warhammer 2 PackFile. These ones have 4 bytes for the size, 4 for the timestamp and 1 for the compression.
                else if pack.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 9 } else { 5 }
            }

            // If it has the last modified date of the PackedFiles, we default to 8. Otherwise, we default to 4.
            PFHVersion::PFH4 => if pack.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 8 } else { 4 }

            // These are like PFH4, but the timestamp has 8 bytes instead of 4.
            PFHVersion::PFH3 | PFHVersion::PFH2 => if pack.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) { 12 } else { 4 }

            // There isn't seem to be a bitmask in ANY PFH0 PackFile, so we will assume they didn't even use it back then.
            PFHVersion::PFH0 => 4
        };*/

        // Prepare the needed stuff to read the PackedFiles.
        let mut index_position: usize = 0;
        dbg!(t.elapsed().unwrap());
        for packed_files_to_decode in (0..files_count).rev() {
            // Get his size. If it's encrypted, decrypt it first.
            let size = if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                let encrypted_size = buffer_mem.read_u32()?;
                //decrypt_index_item_file_length(encrypted_size, packed_files_to_decode as u32)
                todo!()
            } else {
                buffer_mem.read_u32()?
            };

            // If we have the last modified date of the PackedFiles in the Index, get it. Otherwise, default to 0,
            // so we have something to write in case we want to enable them for our PackFile.
            let timestamp = if pack.header.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS) {
                match pack.header.pfh_version {
                    PFHVersion::PFH6 | PFHVersion::PFH5 | PFHVersion::PFH4 => {
                        let timestamp = i64::from(buffer_mem.read_u32()?);
                        if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                            //i64::from(decrypt_index_item_file_length(timestamp as u32, packed_files_to_decode as u32))
                            todo!()
                        } else { timestamp }
                    }

                    // We haven't found a single encrypted PFH3/PFH0 PackFile to test, so always assume these are unencrypted. Also, PFH0 doesn't seem to have a timestamp.
                    PFHVersion::PFH3 | PFHVersion::PFH2 => (buffer_mem.read_i64()? / WINDOWS_TICK) - SEC_TO_UNIX_EPOCH,
                    PFHVersion::PFH0 => 0,
                }
            } else { 0 };

            // Update his offset, and get his compression data if it has it.
            let is_compressed = if let PFHVersion::PFH5 = pack.header.pfh_version {
                buffer_mem.read_bool()?
            } else { false };

            // Get his path. Like the PackFile index, it's a StringU8 terminated in 00. We get it and split it in folders for easy use.
            let path = if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) {
                //decrypt_index_item_filename(&packed_file_index[index_position..], size as u8, &mut index_position)
                todo!()
            }
            else { buffer_mem.read_string_u8_0terminated()? };

            let on_disk = OnDisk {
                path,
                start: data_position,
                size,
                is_compressed,
                is_encrypted: if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) { Some(pack.header.pfh_version) } else { None },
            };

            let file: RFile = RFile {
                path: on_disk.path.to_owned(),
                timestamp: if timestamp == 0 { None } else { Some(timestamp) },
                data: RFileInnerData::OnDisk(on_disk)
            };

            pack.files.insert(file.path_raw().to_owned(), file);
/*

            // If this is a notes PackedFile, save the notes and forget about the PackedFile. Otherwise, save the PackedFile.
            if packed_file.get_path() == [RESERVED_NAME_NOTES] {
                if let Ok(data) = packed_file.get_raw_data_and_keep_it() {
                    if let Ok(data) = data.decode_string_u8(0, data.len()) {
                        pack.notes = Some(data);
                    }
                }
            }

            else if packed_file.get_path() == [RESERVED_NAME_SETTINGS] {
                if let Ok(data) = packed_file.get_raw_data_and_keep_it() {
                    pack.settings = if let Ok(settings) = PackFileSettings::load(&data) {
                        settings
                    } else {
                        PackFileSettings::default()
                    };
                }
            }
            else {
                pack.packed_files.push(packed_file);
            }
*/
            // Then we move our data position. For encrypted files in PFH5 PackFiles (only ARENA) we have to start the next one in a multiple of 8.
            if pack.header.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA) &&
                pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) &&
                pack.header.pfh_version == PFHVersion::PFH5 {
                let padding = 8 - (size % 8);
                let padded_size = if padding < 8 { size + padding } else { size };
                data_position += u64::from(padded_size);
            }
            else { data_position += u64::from(size); }
        }
        dbg!(t.elapsed().unwrap());

        // If at this point we have not reached the end of the PackFile, there is something wrong with it.
        // NOTE: Arena PackFiles have extra data at the end. If we detect one of those PackFiles, take that into account.
        if pack.header.pfh_version == PFHVersion::PFH5 && pack.header.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) {
            if data_position + 256 != data_len { return Err(RLibError::DecodingMismatchSizeError(data_len as usize, data_position as usize)) }
        }
        else if data_position != data_len { return Err(RLibError::DecodingMismatchSizeError(data_len as usize, data_position as usize)) }

        // If we disabled lazy-loading, load every PackedFile to memory.
        //if !use_lazy_loading { for packed_file in &mut pack.files { packed_file.get_ref_mut_raw().load_data()?; }}

        // Return our PackFile.
        Ok(pack)
    }
}
