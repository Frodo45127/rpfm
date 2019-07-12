//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here it goes the logic (Encoding/Decoding) to deal with individual PackedFiles.

use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::sync::{Arc, Mutex};

use crate::packfile::*;
use crate::packfile::compression::decompress_data;

/// This `Struct` stores the data of a PackedFile.
///
/// It contains:
/// - `path`: path of the PackedFile inside the PackFile.
/// - `timestamp`: the '*Last Modified Date*' of the PackedFile, encoded in `i64`.
/// - `is_compressed`: if the data is compressed. Only available from PFH5 onwards.
/// - `is_encrypted`: if the data is encrypted. If some, it contains the PFHVersion of his original PackFile (needed for decryption).
/// - `data`: the data of the PackedFile.
#[derive(Clone, Debug)]
pub struct PackedFile {
    pub path: Vec<String>,
    pub timestamp: i64,
    pub should_be_compressed: bool,
    pub should_be_encrypted: Option<PFHVersion>,
    data: PackedFileData,
}

/// This enum represents the data of a PackedFile.
///
/// - `OnMemory`: the data is loaded to memory and the variant holds the data and info about the current state of the data (is_compressed, is_encrypted).
/// - `OnDisk`: the data is not loaded to memory and the variant holds the file, position and size of the data on the disk, and info about the current 
///   state of the data (is_compressed, is_encrypted).
#[derive(Clone, Debug)]
pub enum PackedFileData {
    OnMemory(Vec<u8>, bool, Option<PFHVersion>),
    OnDisk(Arc<Mutex<BufReader<File>>>, u64, u32, bool, Option<PFHVersion>),
} 

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it, getting his data from a `Vec<u8>`.
    pub fn read_from_vec(path: Vec<String>, timestamp: i64, should_be_compressed: bool, data: Vec<u8>) -> Self {
        Self {
            path,
            timestamp,
            should_be_compressed,
            should_be_encrypted: None,
            data: PackedFileData::OnMemory(data, should_be_compressed, None),
        }
    }

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it, getting his data from a `PackedFileData`.
    pub fn read_from_data(path: Vec<String>, timestamp: i64, should_be_compressed: bool, should_be_encrypted: Option<PFHVersion>, data: PackedFileData) -> Self {
        Self {
            path,
            timestamp,
            should_be_compressed,
            should_be_encrypted,
            data,
        }
    }

    /// This function loads the data from the disk if it's not loaded yet. It just loads the data to memory, without decrypting/decompressing it.
    /// This means we need to take care of that while opening the file.
    pub fn load_data(&mut self) -> Result<()> {
        let data_on_memory = if let PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) = self.data {
            let mut data = vec![0; size as usize];
            file.lock().unwrap().seek(SeekFrom::Start(position))?;
            file.lock().unwrap().read_exact(&mut data)?;
            PackedFileData::OnMemory(data, is_compressed, is_encrypted)
        } else { return Ok(()) };
        
        self.data = data_on_memory;
        Ok(())
    }

    /// This function reads the data from the disk if it's not loaded yet, and return it. This does not store the data in memory.
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data, is_compressed, is_encrypted) => {
                let mut data = data.to_vec();
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { data = decompress_data(&data)?; }
                Ok(data)
            },
            PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { Ok(decompress_data(&data)?) }
                else { Ok(data) }
            }
        }
    }

    /// This function reads the data from the disk if it's not loaded yet (or from memory otherwise), and keep it in memory for faster access.
    pub fn get_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                if is_encrypted.is_some() { *data = decrypt_packed_file(&data); }
                if *is_compressed { *data = decompress_data(&data)?; }
                *is_compressed = false;
                *is_encrypted = None;
                return Ok(data.to_vec())
            },
            PackedFileData::OnDisk(ref file, position, size, is_compressed, is_encrypted) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { decompress_data(&data)? }
                else { data }
            }
        };

        self.data = PackedFileData::OnMemory(data.to_vec(), false, None);
        Ok(data)
    }

    /// This function gets the data and info from memory. Returns an error if the data is not already in memory.
    /// The data returned is "data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted".
    pub fn get_data_and_info_from_memory(&mut self) -> Result<(&mut Vec<u8>, &mut bool, &mut Option<PFHVersion>, &mut bool, &mut Option<PFHVersion>)> {
        match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                Ok((data, is_compressed, is_encrypted, &mut self.should_be_compressed, &mut self.should_be_encrypted))
            },
            PackedFileData::OnDisk(_, _, _, _, _) => {
                Err(ErrorKind::PackedFileDataIsNotInMemory)?
            }
        }
    }

    /// This function loads the data from the disk if it's not loaded yet.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = PackedFileData::OnMemory(data, false, None);
    }

    /// This function returns the size of the data of the PackedFile.
    pub fn get_size(&self) -> u32 {
        match self.data {
            PackedFileData::OnMemory(ref data, _, _) => data.len() as u32,
            PackedFileData::OnDisk(_, _, size, _, _) => size,
        }
    }

    /// This function returns the compression state of a PackedFile.
    pub fn get_compression_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, state, _) => state,
            PackedFileData::OnDisk(_, _, _, state, _) => state,
        }
    }
}
