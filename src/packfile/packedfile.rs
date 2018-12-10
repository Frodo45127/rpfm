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
    pub is_compressed: bool,
    pub is_encrypted: Option<PFHVersion>,
    data: PackedFileData,
}

/// This enum represents the data of a PackedFile.
///
/// - `OnMemory`: the data is loaded to memory and the variant holds it.
/// - `OnDisk`: the data is not loaded to memory and the variant holds the file, position and size of the data on the disk.
#[derive(Clone, Debug)]
pub enum PackedFileData {
    OnMemory(Vec<u8>),
    OnDisk(Arc<Mutex<BufReader<File>>>, u64, u32),
} 

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it, getting his data from a `Vec<u8>`.
    pub fn read_from_vec(path: Vec<String>, timestamp: i64, is_compressed: bool, data: Vec<u8>) -> Self {
        Self {
            path,
            timestamp,
            is_compressed,
            is_encrypted: None,
            data: PackedFileData::OnMemory(data),
        }
    }

    /// This function receive all the info of a PackedFile and creates a `PackedFile` with it, getting his data from a `PackedFileData`.
    pub fn read_from_data(path: Vec<String>, timestamp: i64, is_compressed: bool, is_encrypted: Option<PFHVersion>, data: PackedFileData) -> Self {
        Self {
            path,
            timestamp,
            is_compressed,
            is_encrypted,
            data,
        }
    }

    /// This function loads the data from the disk if it's not loaded yet. It also takes care of decrypting/decompressing the PackedFile if needed.
    /// Common sense says that if you want a compressed and encrypted PackFile, we should first compress, then encrypt. So first we check to undo
    /// the encryption, then we check to decompress it if it's needed. And I say common sense because we have yet to encounter a PackedFile with both. 
    pub fn load_data(&mut self) -> Result<()> {
        let data_on_memory = if let PackedFileData::OnDisk(ref file, position, size) = self.data {
            let mut data = vec![0; size as usize];
            file.lock().unwrap().seek(SeekFrom::Start(position))?;
            file.lock().unwrap().read_exact(&mut data)?;
            if self.is_encrypted.is_some() { data = decrypt_packed_file(&data); }
            if self.is_compressed { PackedFileData::OnMemory(decompress_data(data.to_vec())?) }
            else { PackedFileData::OnMemory(data) }
        } else { return Ok(()) };
        
        self.data = data_on_memory;
        Ok(())
    }

    /// This function reads the data from the disk if it's not loaded yet, and return it. This does not store the data in memory.
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data) => Ok(data.to_vec()),
            PackedFileData::OnDisk(ref file, position, size) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if self.is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if self.is_compressed { Ok(decompress_data(data.to_vec())?) }
                else { Ok(data) }
            }
        }
    }

    /// This function reads the data from the disk if it's not loaded yet (or from memory otherwise), and keep it in memory for faster access.
    pub fn get_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref data) => return Ok(data.to_vec()),
            PackedFileData::OnDisk(ref file, position, size) => {
                let mut data = vec![0; size as usize];
                file.lock().unwrap().seek(SeekFrom::Start(position))?;
                file.lock().unwrap().read_exact(&mut data)?;
                if self.is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if self.is_compressed { decompress_data(data.to_vec())? }
                else { data }
            }
        };

        self.data = PackedFileData::OnMemory(data.clone());
        Ok(data)
    }

    /// This function loads the data from the disk if it's not loaded yet.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = PackedFileData::OnMemory(data);
    }

    /// This function returns the size of the data of the PackedFile.
    pub fn get_size(&self) -> u32 {
        match self.data {
            PackedFileData::OnMemory(ref data) => data.len() as u32,
            PackedFileData::OnDisk(_, _, size) => size,
        }
    }
}
