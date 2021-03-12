//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with infividual PackedFiles.

This module contains all the code related with the interaction with individual PackFiles,
meaning the code that takes care of loading/writing their data from/to disk.

You'll rarely have to touch anything here.
!*/

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::{BufReader, Read, SeekFrom};
use std::fs::File;
use std::sync::{Arc, Mutex};

use rpfm_error::Error;

use crate::packedfile::animpack::AnimPacked;
use crate::packfile::*;
use crate::packfile::compression::decompress_data;
use crate::packedfile::{DecodedPackedFile, PackedFileType};
use crate::packedfile::table::{db::DB, loc::Loc};
use crate::schema::Schema;
use crate::SCHEMA;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a `PackedFile` in memory.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedFile {
    raw: RawPackedFile,
    decoded: DecodedPackedFile,
}

/// This struct represents a `PackedFile` in memory in his raw form.
#[derive(Clone, Debug, PartialEq)]
pub struct RawPackedFile {

    /// The path of the `PackedFile` inside the `PackFile`.
    path: Vec<String>,

    /// Name of the original `PackFile` containing it. To know from where a `PackedFile` came when loading multiple PackFiles as one.
    packfile_name: String,

    /// The '*Last Modified Date*' of the `PackedFile`, encoded in `i64`. Only in PackFiles with the appropiate flag enabled..
    timestamp: i64,

    /// If the data should be compressed when saving it to disk. Only available from `PFHVersion::PFH5` onwards.
    should_be_compressed: bool,

    /// If the data should be encrypted when saving it to disk. If it should, it contains `Some(PFHVersion)`, being `PFHVersion` the one of the game this `PackedFile` is for.
    should_be_encrypted: Option<PFHVersion>,

    /// the data of the PackedFile. Use the getter/setter functions to interact with it.
    data: PackedFileData,
}

/// This enum represents the data of a `PackedFile`, in his current state.
#[derive(Clone, Debug)]
pub enum PackedFileData {

    /// The data is loaded to memory and the variant holds the data and info about the current state of the data (data, is_compressed, is_encrypted).
    OnMemory(Vec<u8>, bool, Option<PFHVersion>),

    /// The data is not loaded to memory and the variant holds the info needed to get the data loaded to memory on demand.
    OnDisk(RawOnDisk),
}

/// This struct contains the stuff needed to read the data of a particular PackedFile from disk.
#[derive(Clone, Debug)]
pub struct RawOnDisk {

    /// Reader over the PackFile containing the PackedFile.
    reader: Arc<Mutex<BufReader<File>>>,
    start: u64,
    size: u32,
    is_compressed: bool,
    is_encrypted: Option<PFHVersion>,

    /// Hash of the PackedFile's data, to ensure we don't grab the wrong data.
    hash: Arc<Mutex<u64>>,
}

/// This struct represents the detailed info about the `PackedFile` we can provide to whoever request it.
#[derive(Clone, Debug, Default)]
pub struct PackedFileInfo {

    /// This is the path of the `PackedFile`.
    pub path: Vec<String>,

    /// This is the name of the `PackFile` this file belongs to.
    pub packfile_name: String,

    /// This is the ***Last Modified*** time.
    pub timestamp: i64,

    /// If the `PackedFile` is compressed or not.
    pub is_compressed: bool,

    /// If the `PackedFile` is encrypted or not.
    pub is_encrypted: bool,

    /// If the `PackedFile` has been cached or not.
    pub is_cached: bool,

    /// The type of the cached `PackedFile`.
    pub cached_type: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `PackedFile`.
impl PackedFile {

    /// This function creates a new empty `PackedFile` in the provided path.
    pub fn new(
        path: Vec<String>,
        packfile_name: String
    ) -> Self {
        Self {
            raw: RawPackedFile {
                path,
                packfile_name,
                timestamp: 0,
                should_be_compressed: false,
                should_be_encrypted: None,
                data: PackedFileData::OnMemory(vec![], false, None),
            },
            decoded: DecodedPackedFile::Unknown,
        }
    }

    /// This function creates a new `PackedFile` from the provided `RawPackedFile`.
    pub fn new_from_raw(data: &RawPackedFile) -> Self {
        Self {
            raw: data.clone(),
            decoded: DecodedPackedFile::Unknown,
        }
    }

    /// This function creates a new `PackedFile` from the provided `DecodedPackedFile` and path.
    pub fn new_from_decoded(data: &DecodedPackedFile, path: &[String]) -> Self {
        Self {
            raw: RawPackedFile {
                path: path.to_owned(),
                packfile_name: "".to_owned(),
                timestamp: 0,
                should_be_compressed: false,
                should_be_encrypted: None,
                data: PackedFileData::OnMemory(vec![], false, None),
            },
            decoded: data.clone(),
        }
    }

    /// This function creates a new `PackedFile` from the provided path on disk and path on a PackFile.
    pub fn new_from_file(path: &Path, packed_file_path: &[String]) -> Result<Self> {
        Ok(Self {
            raw: RawPackedFile::read_from_path(path, packed_file_path.to_vec())?,
            decoded: DecodedPackedFile::Unknown,
        })
    }

    /// This function creates a new empty `PackedFile` of the provided type and path.
    pub fn new_from_type_and_path(
        packed_file_type: PackedFileType,
        path: Vec<String>,
        dependencies: &Dependencies,
    ) -> Result<Self> {

        // Depending on their type, we do different things to prepare the PackedFile and get his data.
        let schema = SCHEMA.read().unwrap();
        let data = match packed_file_type {

            // For locs, we just create them with their last definition.
            PackedFileType::Loc => {
                let definition = match *schema {
                    Some(ref schema) => schema.get_ref_last_definition_loc()?,
                    None => return Err(ErrorKind::SchemaNotFound.into())
                };
                DecodedPackedFile::Loc(Loc::new(&definition))
            },

            // For dbs, we create them with their last definition, if we found one, and their table name.
            PackedFileType::DB => {
                let table_name = path.get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let table_definition = match *schema {
                    Some(ref schema) => schema.get_ref_last_definition_db(table_name, dependencies)?,
                    None => return Err(ErrorKind::SchemaNotFound.into())
                };
                DecodedPackedFile::DB(DB::new(&table_name, None, &table_definition))
            }

            // TODO: Add Text files here.

            // For anything else, just return `Unknown`.
            _ => DecodedPackedFile::Unknown,
        };

        Ok(Self::new_from_decoded(&data, &path))
    }

    /// This function returns a reference to the `RawPackedFile` part of a `PackedFile`.
    pub fn get_ref_raw(&self) -> &RawPackedFile {
        &self.raw
    }

    /// This function returns a reference to the `DecodedPackedFile` part of a `PackedFile`.
    pub fn get_ref_decoded(&self) -> &DecodedPackedFile {
        &self.decoded
    }

    /// This function returns a mutable reference to the `RawPackedFile` part of a `PackedFile`.
    pub fn get_ref_mut_raw(&mut self) -> &mut RawPackedFile {
        &mut self.raw
    }

    /// This function returns a mutable reference to the `DecodedPackedFile` part of a `PackedFile`.
    pub fn get_ref_mut_decoded(&mut self) -> &mut DecodedPackedFile {
        &mut self.decoded
    }

    /// This function returns a copy of the `RawPackedFile` part of a `PackedFile`.
    pub fn get_raw(&self) -> RawPackedFile {
        self.raw.clone()
    }

    /// This function returns a copy of the `DecodedPackedFile` part of a `PackedFile`.
    pub fn get_decoded(&self) -> DecodedPackedFile {
        self.decoded.clone()
    }

    /// This function replace the `RawPackedFile` part of a `PackedFile` with the provided one.
    pub fn set_raw(&mut self, data: &RawPackedFile) {
        self.raw = data.clone();
    }

    /// This function replace the `DecodedPackedFile` part of a `PackedFile` with the provided one.
    pub fn set_decoded(&mut self, data: &DecodedPackedFile) {
        self.decoded = data.clone();
    }

    /// This function tries to get the decoded data from a `PackedFile`, returning an error if the file was not decoded previously.
    pub fn get_decoded_from_memory(&self) -> Result<&DecodedPackedFile> {
        if self.decoded == DecodedPackedFile::Unknown {
            return Err(ErrorKind::PackedFileNotDecoded.into());
        }
        Ok(&self.decoded)
    }

    /// This function returns a reference of the path of a `PackedFile`.
    pub fn get_path(&self) -> &[String] {
        self.raw.get_path()
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`.
    pub fn decode(&mut self) -> Result<()> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode(&mut self.raw)?;
        }
        Ok(())
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`.
    ///
    /// This variant doesn't re-unlock the schema, so you can use it for batch decoding.
    pub fn decode_no_locks(&mut self, schema: &Schema) -> Result<()> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode_no_locks(&mut self.raw, schema)?;
        }
        Ok(())
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`,
    /// and returning a reference to it.
    ///
    /// This takes into account cached decoding so, if it has already been decoded, it doesn't decode it again.
    pub fn decode_return_ref(&mut self) -> Result<&DecodedPackedFile> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode(&mut self.raw)?;
        }
        Ok(&self.decoded)
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`,
    /// and returning a mutable reference to it.
    ///
    /// This takes into account cached decoding so, if it has already been decoded, it doesn't decode it again.
    pub fn decode_return_ref_mut(&mut self) -> Result<&mut DecodedPackedFile> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode(&mut self.raw)?;
        }
        Ok(&mut self.decoded)
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`,
    /// and returning a reference to it.
    ///
    /// This variant doesn't lock the Schema. This means is faster if you're decoding `PackedFiles` in batches.
    pub fn decode_return_ref_no_locks(&mut self, schema: &Schema) -> Result<&DecodedPackedFile> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode_no_locks(&mut self.raw, schema)?;
        }
        Ok(&self.decoded)
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, storing the results in the `Packedfile`,
    /// and returning a mutable reference to it.
    ///
    /// This variant doesn't lock the Schema. This means is faster if you're decoding `PackedFiles` in batches.
    pub fn decode_return_ref_mut_no_locks(&mut self, schema: &Schema) -> Result<&mut DecodedPackedFile> {
        if self.decoded == DecodedPackedFile::Unknown {
            self.decoded = DecodedPackedFile::decode_no_locks(&mut self.raw, schema)?;
        }
        Ok(&mut self.decoded)
    }

    /// This function tries to decode a `RawPackedFile` into a `DecodedPackedFile`, returning the result without holding them
    /// in the cache, and clearing any existing cache.
    ///
    /// This variant doesn't lock the Schema. This means is faster if you're decoding `PackedFiles` in batches.
    pub fn decode_return_clean_cache(&mut self) -> Result<DecodedPackedFile> {
        if self.decoded != DecodedPackedFile::Unknown {
            self.encode_and_clean_cache()?;
        }
        DecodedPackedFile::decode(&mut self.raw)
    }

    /// This function tries to encode a `DecodedPackedFile` into a `RawPackedFile`, storing the results in the `Packedfile`.
    ///
    /// If the PackedFile is not decoded or has no saving support (encode returns None), it does nothing.
    pub fn encode(&mut self) -> Result<()> {
        match self.decoded.encode() {
            Some(data) => self.raw.set_data(&data?),
            None => self.raw.load_data()?,
        }
        Ok(())
    }

    /// This function tries to encode a `DecodedPackedFile` into a `RawPackedFile`, storing the results in the `Packedfile`.
    /// Then, it removes the decoded data from the cache.
    ///
    /// If the PackedFile is not decoded or has no saving support (encode returns None), it does nothing.
    pub fn encode_and_clean_cache(&mut self) -> Result<()> {
        match self.decoded.encode() {
            Some(data) => self.raw.set_data(&data?),
            None => self.raw.load_data()?,
        }
        self.decoded = DecodedPackedFile::Unknown;
        Ok(())
    }

    /// This function tries to encode a `DecodedPackedFile` into a `RawPackedFile`, storing the results in the `Packedfile`,
    /// and returning a reference to it.
    ///
    /// If the PackedFile is not decoded or has no saving support (encode returns None), it does nothing.
    pub fn encode_and_return(&mut self) -> Result<&RawPackedFile> {
        match self.decoded.encode() {
            Some(data) => self.raw.set_data(&data?),
            None => self.raw.load_data()?,
        }
        Ok(&self.raw)
    }

    /// This function returns the size in bytes of the `RawPackedFile` data, if its loaded. If it isn't, it returns 0.
    pub fn get_raw_data_size(&self) -> u32 {
        self.raw.get_size()
    }

    /// This function returns the data of a PackedFile.
    pub fn get_raw_data(&self) -> Result<Vec<u8>> {
        self.raw.get_data()
    }

    /// This function returns the data of a PackedFile.
    pub fn get_raw_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        self.raw.get_data_and_keep_it()
    }

    /// This function returns the data of a PackedFile, making sure we clear the cache before it.
    pub fn get_raw_data_and_clean_cache(&mut self) -> Result<Vec<u8>> {
        if self.decoded != DecodedPackedFile::Unknown {
            self.encode_and_clean_cache()?;
        }
        self.raw.get_data()
    }

    /// This function replaces the raw data of a PackedFile with the provided one.
    pub fn set_raw_data(&mut self, data: &[u8]) {
        self.raw.set_data(data);
    }

    /// This function extracts the provided PackedFile into the provided path.
    pub fn extract_packed_file(&mut self, destination_path: &Path) -> Result<()> {

        // Save it, in case it's cached.
        self.encode()?;

        // We get his internal path without his name.
        let mut internal_path = self.get_path().to_vec();
        let file_name = internal_path.pop().unwrap();

        // Then, we join his internal path with his destination path, so we have his almost-full path (his final path without his name).
        // This way we can create the entire folder structure up to the file itself.
        let mut current_path = destination_path.to_path_buf().join(internal_path.iter().collect::<PathBuf>());
        DirBuilder::new().recursive(true).create(&current_path)?;

        // Finish the path and try to save the file to disk.
        current_path.push(&file_name);
        let mut file = BufWriter::new(File::create(&current_path)?);
        if file.write_all(&self.get_raw_data()?).is_err() {
            return Err(ErrorKind::ExtractError(self.get_path().to_vec()).into());
        }
        Ok(())
    }

    /// This function returns the type of the Provided PackedFile, according to it's path.
    pub fn get_packed_file_type(&self, strict_mode: bool) -> PackedFileType {
        PackedFileType::get_packed_file_type(self.get_ref_raw(), strict_mode)
    }
}

/// Implementation of `RawPackedFile`.
impl RawPackedFile {

    /// This function creates a new `RawPackedFile` from a `Vec<u8>` and some extra data.
    pub fn read_from_vec(
        path: Vec<String>,
        packfile_name: String,
        timestamp: i64,
        should_be_compressed: bool,
        data: Vec<u8>
    ) -> Self {
        Self {
            path,
            packfile_name,
            timestamp,
            should_be_compressed,
            should_be_encrypted: None,
            data: PackedFileData::OnMemory(data, should_be_compressed, None),
        }
    }

    /// This function creates a new `RawPackedFile` from a another's `RawPackedFile`'s data, and some extra data. What an asshole.
    pub fn read_from_data(
        path: Vec<String>,
        packfile_name: String,
        timestamp: i64,
        should_be_compressed: bool,
        should_be_encrypted: Option<PFHVersion>,
        data: PackedFileData
    ) -> Self {
        Self {
            path,
            packfile_name,
            timestamp,
            should_be_compressed,
            should_be_encrypted,
            data,
        }
    }

    /// This function creates a new `RawPackedFile` from a file in the filesystem.
    ///
    /// Keep in mind that you have to set the name of his `PackFile` if you add it to one.
    pub fn read_from_path(
        path_as_file: &Path,
        path_as_packed_file: Vec<String>,
    ) -> Result<Self> {
        let mut file = BufReader::new(File::open(&path_as_file)?);
        let mut data = vec![];
        file.read_to_end(&mut data)?;
        Ok(RawPackedFile::read_from_vec(path_as_packed_file, String::new(), get_last_modified_time_from_file(&file.get_ref()), false, data))
    }

    /// This function loads the data of a `RawPackedFile` to memory, if it isn't loaded already.
    pub fn load_data(&mut self) -> Result<()> {
        let data_on_memory = if let PackedFileData::OnDisk(ref raw_on_disk) = self.data {
            PackedFileData::OnMemory(raw_on_disk.read()?, raw_on_disk.get_compression_state(), raw_on_disk.get_encryption())
        } else { return Ok(()) };

        self.data = data_on_memory;
        Ok(())
    }

    /// This function returns the RAW data of the `RawPackedFile` without loading it to memory.
    ///
    /// This means this data is not decompressed/decrypted. For particular situations.
    pub fn get_raw_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data, _, _) => {
                Ok(data.to_vec())
            },
            PackedFileData::OnDisk(ref raw_on_disk) => {
                raw_on_disk.read()
            }
        }
    }

    /// This function returns the data of the `RawPackedFile` without loading it to memory.
    ///
    /// It's for those situations where you just need to check the data once, then forget about it.
    pub fn get_data(&self) -> Result<Vec<u8>> {
        match self.data {
            PackedFileData::OnMemory(ref data, is_compressed, is_encrypted) => {
                let mut data = data.to_vec();
                if is_encrypted.is_some() { data = decrypt_packed_file(&data); }
                if is_compressed { data = decompress_data(&data)?; }
                Ok(data)
            },
            PackedFileData::OnDisk(ref raw_on_disk) => {
                let mut data = raw_on_disk.read()?;
                if raw_on_disk.get_encryption_state() { data = decrypt_packed_file(&data); }
                if raw_on_disk.get_compression_state() { decompress_data(&data) }
                else { Ok(data) }
            }
        }
    }

    /// This function returns the data of the provided `RawPackedFile` loading it to memory in the process if it isn't already loaded.
    ///
    /// It's for when you need to keep the data for multiple uses.
    pub fn get_data_and_keep_it(&mut self) -> Result<Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                if is_encrypted.is_some() { *data = decrypt_packed_file(&data); }
                if *is_compressed { *data = decompress_data(&data)?; }
                *is_compressed = false;
                *is_encrypted = None;
                return Ok(data.to_vec())
            },
            PackedFileData::OnDisk(ref raw_on_disk) => {
                let mut data = raw_on_disk.read()?;
                if raw_on_disk.get_encryption_state() { data = decrypt_packed_file(&data); }
                if raw_on_disk.get_compression_state() { decompress_data(&data)? }
                else { data }
            }
        };

        self.data = PackedFileData::OnMemory(data.to_vec(), false, None);
        Ok(data)
    }

    /// This function returns a mutable reference to the data of the provided `RawPackedFile`,
    /// loading it to memory in the process if it isn't already loaded.
    ///
    /// It's for when you need to modify the data directly. Try to not abuse it.
    pub fn get_ref_mut_data_and_keep_it(&mut self) -> Result<&mut Vec<u8>> {
        let data = match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                if is_encrypted.is_some() { *data = decrypt_packed_file(&data); }
                if *is_compressed { *data = decompress_data(&data)?; }
                *is_compressed = false;
                *is_encrypted = None;
                return Ok(data)
            },
            PackedFileData::OnDisk(ref raw_on_disk) => {
                let mut data = raw_on_disk.read()?;
                if raw_on_disk.get_encryption_state() { data = decrypt_packed_file(&data); }
                if raw_on_disk.get_compression_state() { decompress_data(&data)? }
                else { data }
            }
        };

        self.data = PackedFileData::OnMemory(data, false, None);
        if let PackedFileData::OnMemory(ref mut data, _, _) = self.data { Ok(data) } else { unimplemented!() }
    }

    /// This function returns the data of the provided `RawPackedFile` from memory. together with his state info.
    ///
    /// The data returned is `path, data, is_compressed, is_encrypted, should_be_compressed, should_be_encrypted`.
    pub fn get_data_and_info_from_memory(&mut self) -> Result<(&[String], &mut Vec<u8>, &mut bool, &mut Option<PFHVersion>, &mut bool, &mut Option<PFHVersion>)> {
        match self.data {
            PackedFileData::OnMemory(ref mut data, ref mut is_compressed, ref mut is_encrypted) => {
                Ok((&self.path, data, is_compressed, is_encrypted, &mut self.should_be_compressed, &mut self.should_be_encrypted))
            },
            PackedFileData::OnDisk(_) => {
                Err(ErrorKind::PackedFileDataIsNotInMemory.into())
            }
        }
    }

    /// This function replaces the data on the `RawPackedFile` with the provided one.
    pub fn set_data(&mut self, data: &[u8]) {
        self.data = PackedFileData::OnMemory(data.to_vec(), false, None);
    }

    /// This function returns the size of the data of the provided `RawPackedFile`.
    pub fn get_size(&self) -> u32 {
        match self.data {
            PackedFileData::OnMemory(ref data, _, _) => data.len() as u32,
            PackedFileData::OnDisk(ref raw_on_disk) => raw_on_disk.get_size(),
        }
    }

    /// This function returns the current compression state of the provided `RawPackedFile`.
    pub fn get_compression_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, state, _) => state,
            PackedFileData::OnDisk(ref raw_on_disk) => raw_on_disk.get_compression_state(),
        }
    }

    /// This function returns if the `RawPackedFile` should be compressed or not.
    pub fn get_should_be_compressed(&self) -> bool{
        self.should_be_compressed
    }

    /// This function sets if the `RawPackedFile` should be compressed or not.
    pub fn set_should_be_compressed(&mut self, state: bool) {
        self.should_be_compressed = state;
    }

    /// This function returns the name of the PackFile this `RawPackedFile` belongs to.
    pub fn get_packfile_name(&self) -> &str {
        &self.packfile_name
    }

    /// This function sets the name of the PackFile this `RawPackedFile` belongs to.
    pub fn set_packfile_name(&mut self, name: &str) {
        self.packfile_name = name.to_owned();
    }

    /// This function returns the current encryption state of the provided `RawPackedFile`.
    pub fn get_encryption_state(&self) -> bool {
        match self.data {
            PackedFileData::OnMemory(_, _, state) => state.is_some(),
            PackedFileData::OnDisk(ref raw_on_disk) => raw_on_disk.get_encryption_state(),
        }
    }

    /// This function returns if the `RawPackedFile` should be encrypted or not.
    ///
    /// If it should, it'll return the `PFHVersion` to encrypt to.
    pub fn get_should_be_encrypted(&self) -> &Option<PFHVersion> {
        &self.should_be_encrypted
    }

    /// This function sets if the `RawPackedFile` should be encrypted or not.
    pub fn set_should_be_encrypted(&mut self, state: Option<PFHVersion>) {
        self.should_be_encrypted = state;
    }

    /// This function returns the timestamp of the provided `RawPackedFile`.
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// This function sets the timestamp of the provided `RawPackedFile`.
    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    /// This function returns a reference to the path of the provided `RawPackedFile`.
    pub fn get_path(&self) -> &[String] {
        &self.path
    }

    /// This function sets the path of the provided `RawPackedFile`.
    ///
    /// This can fail if you pass it an empty path, so make sure you check the result.
    ///
    /// ***WARNING***: DON'T USE THIS IF YOUR PACKEDFILE IS INSIDE A PACKFILE. USE THE `move_packedfile` FUNCTION INSTEAD.
    pub fn set_path(&mut self, path: &[String]) -> Result<()> {
        if path.is_empty() { return Err(ErrorKind::EmptyInput.into()) }
        self.path = path.to_vec();
        Ok(())
    }
}

/// Implementation of RawOnDisk.
impl RawOnDisk {

    /// This function creates a new RawOnDisk.
    pub fn new(
        reader: Arc<Mutex<BufReader<File>>>,
        start: u64,
        size: u32,
        is_compressed: bool,
        is_encrypted: Option<PFHVersion>,
    ) -> Self {
        Self {
            reader,
            start,
            size,
            is_compressed,
            is_encrypted,
            hash: Arc::new(Mutex::new(0)),
        }
    }

    /// This function tries to read and return the raw data of the PackedFile.
    pub fn read(&self) -> Result<Vec<u8>> {
        let mut data = vec![0; self.size as usize];
        let mut file = self.reader.lock().unwrap();
        file.seek(SeekFrom::Start(self.start))?;
        file.read_exact(&mut data)?;

        let mut current_hash = self.hash.lock().unwrap();
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let new_hash = hasher.finish();

        // If we have no hash, it's the first read. Hash it.
        if *current_hash == 0 {
            *current_hash = new_hash;
        }


        // Otherwise, check its hash to ensure we're not fucking up the PackFile.
        else if *current_hash != new_hash {
            return Err(ErrorKind::PackedFileChecksumFailed.into());
        }
        Ok(data)
    }

    /// This function returns the size of the PackedFile.
    pub fn get_size(&self) -> u32 {
        self.size
    }

    /// This function returns if the PackedFile is compressed or not.
    pub fn get_compression_state(&self) -> bool {
        self.is_compressed
    }

    /// This function returns if the PackedFile is encrypted or not.
    pub fn get_encryption_state(&self) -> bool {
        self.is_encrypted.is_some()
    }

    /// This function returns the encryption info of the PackedFile.
    pub fn get_encryption(&self) -> Option<PFHVersion> {
        self.is_encrypted
    }
}

/// Implementation of `PartialEq` for `PackedFileData`.
impl PartialEq for PackedFileData {
    fn eq(&self, other: &PackedFileData) -> bool {
        match (self, other) {
            (
                &PackedFileData::OnMemory(ref data, is_compressed, is_encrypted),
                &PackedFileData::OnMemory(ref data_2, is_compressed_2, is_encrypted_2)) =>
                    data == data_2 &&
                    is_compressed == is_compressed_2 &&
                    is_encrypted == is_encrypted_2,
            _ => false,
        }
    }
}

/// Implementation to create a `PackedFileInfo` from a `PackedFile`.
impl From<&PackedFile> for PackedFileInfo {
    fn from(packedfile: &PackedFile) -> Self {
        let is_cached = !matches!(packedfile.get_ref_decoded(), DecodedPackedFile::Unknown);
        let cached_type = if let DecodedPackedFile::Unknown = packedfile.get_ref_decoded() { "Not Yet Cached".to_owned() }
        else { format!("{:?}", PackedFileType::from(packedfile.get_ref_decoded())) };
        Self {
            path: packedfile.get_path().to_vec(),
            packfile_name: packedfile.get_ref_raw().get_packfile_name().to_owned(),
            timestamp: packedfile.get_ref_raw().get_timestamp(),
            is_compressed: packedfile.get_ref_raw().get_compression_state(),
            is_encrypted: packedfile.get_ref_raw().get_encryption_state(),
            is_cached,
            cached_type,
        }
    }
}

/// Implementation to create a `PackedFile` from a `AnimPacked`.
impl From<&AnimPacked> for PackedFile {
    fn from(anim_packed: &AnimPacked) -> Self {
        let mut packed_file = Self::new(anim_packed.get_ref_path().to_owned(), String::new());
        packed_file.set_raw_data(anim_packed.get_ref_data());
        packed_file
    }
}
