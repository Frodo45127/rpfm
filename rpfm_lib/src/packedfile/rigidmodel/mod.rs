//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with RigidModel PackedFiles.

RigidModel PackedFiles are 3D models used by Total War games since Empire.
This is basically a rewrite in Rust of the work done by Phazer on his tool.
Because I want to avoid more C++ libs if posible.
!*/

use half::f16;
use serde_derive::{Serialize, Deserialize};

use rpfm_error::{ErrorKind, Result};

use crate::common::{decoder::Decoder, encoder::Encoder};

/// This represents the value that every RigidModel PackedFile has in their 0-4 bytes. A.k.a it's signature or preamble.
const PACKED_FILE_TYPE: &str = "RMV2";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains a RigidModel decoded in memory.
#[derive(Clone, Debug,PartialEq, Serialize, Deserialize)]
pub struct RigidModel {
    pub header: Header,
    //pub packed_file_data: RigidModelData,
}

/// This struct represents the header of a RigidModel.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Header {

    /// The version of the RigidModel. The supported versions per game are:
    /// - `6`: Attila or older.
    /// - `7`: Warhammer 1 & 2.
    /// - `8`: Three Kingdoms.
    version: u32,

    /// The skeleton used by this RigidModel.
    skeleton_id: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Implementation of RigidModel.
impl RigidModel {

    /// This function creates a new empty `Decal` RigidModel.
    pub fn new_decal() -> Self {
        Self {
            header: Header::default(),
        }
    }
}

/// Struct "RigidModelHeader". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RigidModelHeader {
    pub packed_file_header_signature: String,
    pub packed_file_header_model_type: u32,
    pub packed_file_header_lods_count: u32,
    pub packed_file_data_base_skeleton: (String, usize),
}

/// Struct "RigidModelData". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RigidModelData {
    pub packed_file_data_lods_header: Vec<RigidModelLodHeader>,
    pub packed_file_data_lods_data: Vec<u8>,
}

/// Struct "RigidModelLodHeader". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RigidModelLodHeader {
    pub groups_count: u32,
    pub vertices_data_length: u32,
    pub indices_data_length: u32,
    pub start_offset: u32,
    pub lod_zoom_factor: f32,
    pub mysterious_data_1: Option<u32>,
    pub mysterious_data_2: Option<u32>,
}

/// Implementation of "RigidModel"
impl RigidModel {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModel. This CAN FAIL,
    /// so we return Result<RigidModel, Error>.
    pub fn read(packed_file_data: &[u8]) -> Result<RigidModel> {
        let packed_file_header = RigidModelHeader::read(&packed_file_data[..140])?;
        let packed_file_data = RigidModelData::read(&packed_file_data[140..], packed_file_header.packed_file_header_model_type, packed_file_header.packed_file_header_lods_count)?;

        Ok(RigidModel {
            packed_file_header,
            packed_file_data,
        })
    }

    /// This function reads the data from a RigidModel and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_data: &RigidModel) -> Result<Vec<u8>> {
        let mut packed_file_data_encoded = RigidModelData::save(&rigid_model_data.packed_file_data);
        let mut packed_file_header_encoded = RigidModelHeader::save(&rigid_model_data.packed_file_header)?;

        let mut packed_file_encoded = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded);

        Ok(packed_file_encoded)
    }

    /// This function is used to patch a RigidModel 3D model from Total War: Attila to work in Total War:
    /// Warhammer 1 and 2. The process to patch a RigidModel is simple:
    /// - We update the version of the RigidModel from 6(Attila) to 7(Warhammer 1&2).
    /// - We add 2 u32 to the Lods: a counter starting at 0, and a 0.
    /// - We increase the start_offset of every Lod by (8*amount_of_lods).
    /// - We may need to increase the zoom_factor of the first Lod to 1000.0, because otherwise sometimes the models
    ///   disappear when you move the camera far from them.
    /// It requires a mut ref to a decoded PackFile, and returns an String (Result<Success, Error>).
    pub fn patch_rigid_model_attila_to_warhammer (&mut self) -> Result<String> {

        // If the RigidModel is an Attila RigidModel, we continue. Otherwise, return Error.
        match self.packed_file_header.packed_file_header_model_type {
            6 => {
                // We update his version.
                self.packed_file_header.packed_file_header_model_type = 7;

                // Next, we change the needed data for every Lod.
                for (index, lod) in self.packed_file_data.packed_file_data_lods_header.iter_mut().enumerate() {
                    lod.mysterious_data_1 = Some(index as u32);
                    lod.mysterious_data_2 = Some(0);
                    lod.start_offset += 8 * self.packed_file_header.packed_file_header_lods_count;
                }
                Ok("RigidModel patched succesfully.".to_owned())
            },
            7 => Err(ErrorKind::RigidModelPatchToWarhammer("This is not an Attila's RigidModel, but a Warhammer one.".to_owned()).into()),
            _ => Err(ErrorKind::RigidModelPatchToWarhammer("I don't even know from what game is this RigidModel.".to_owned()).into()),
        }
    }
}

/// Implementation of "RigidModelHeader"
impl RigidModelHeader {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelHeader. This CAN FAIL,
    /// so we return Result<RigidModelHeader, Error>.
    pub fn read(packed_file_data: &[u8]) -> Result<RigidModelHeader> {

        let mut packed_file_header = RigidModelHeader {
            packed_file_header_signature: String::new(),
            packed_file_header_model_type: 0,
            packed_file_header_lods_count: 0,
            packed_file_data_base_skeleton: (String::new(), 0),
        };

        match packed_file_data.decode_string_u8(0, 4) {
            Ok(data) => packed_file_header.packed_file_header_signature = data,
            Err(error) => return Err(error)
        }

        // We check this, just in case we try to read some malformed file with a string in the first
        // four bytes (which is not uncommon).
        if packed_file_header.packed_file_header_signature != "RMV2" {
            return Err(ErrorKind::RigidModelNotSupportedFile.into())
        }

        match packed_file_data.decode_integer_u32(4) {
            Ok(data) => packed_file_header.packed_file_header_model_type = data,
            Err(error) => return Err(error)
        }

        match packed_file_data.decode_integer_u32(8) {
            Ok(data) => packed_file_header.packed_file_header_lods_count = data,
            Err(error) => return Err(error)
        }

        match packed_file_data.decode_string_u8_0padded(12, 128) {
            Ok(data) => packed_file_header.packed_file_data_base_skeleton = data,
            Err(error) => return Err(error)
        }
        Ok(packed_file_header)
    }

    /// This function reads the data from a RigidModelHeader and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_header: &RigidModelHeader) -> Result<Vec<u8>> {
        let mut packed_file_data: Vec<u8> = vec![];

        packed_file_data.encode_string_u8(&rigid_model_header.packed_file_header_signature);
        packed_file_data.encode_integer_u32(rigid_model_header.packed_file_header_model_type);
        packed_file_data.encode_integer_u32(rigid_model_header.packed_file_header_lods_count);
        packed_file_data.encode_string_u8_0padded(&rigid_model_header.packed_file_data_base_skeleton)?;

        Ok(packed_file_data)
    }
}

/// Implementation of "RigidModelData"
impl RigidModelData {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelData. This CAN FAIL,
    /// so we return Result<RigidModelData, Error>.
    pub fn read(packed_file_data: &[u8], packed_file_header_model_type: u32, packed_file_header_lods_count: u32) -> Result<RigidModelData> {
        let mut packed_file_data_lods_header: Vec<RigidModelLodHeader> = vec![];
        let mut index: usize = 0;
        let offset: usize = match packed_file_header_model_type {
            6 => 20, // Attila
            7 => 28, // Warhammer 1&2
            _ => return Err(ErrorKind::RigidModelNotSupportedType.into())
        };

        // We get the "headers" of every lod.
        for _ in 0..packed_file_header_lods_count {
            let lod_header = match RigidModelLodHeader::read(&packed_file_data[index..(index + offset)]) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lods_header.push(lod_header);
            index += offset;
        }

        let packed_file_data_lods_data = packed_file_data[index..].to_vec();

        Ok(RigidModelData {
            packed_file_data_lods_header,
            packed_file_data_lods_data,
        })
    }

    /// This function reads the data from a RigidModelData and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_data: &RigidModelData) -> Vec<u8> {
        let mut packed_file_data = vec![];

        // For each Lod, we save it, and add it to the "Encoded Data" vector. After that, we add to that
        // vector the extra data, and return it.
        for lod in &rigid_model_data.packed_file_data_lods_header {
            packed_file_data.append(&mut RigidModelLodHeader::save(lod));
        }

        packed_file_data.extend_from_slice(&rigid_model_data.packed_file_data_lods_data);
        packed_file_data
    }
}

/// Implementation of "RigidModelLodHeader"
impl RigidModelLodHeader {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelLodHeader. This CAN FAIL,
    /// so we return Result<RigidModelLodHeader, Error>.
    pub fn read(packed_file_data: &[u8]) -> Result<RigidModelLodHeader> {
        let mut header = RigidModelLodHeader {
            groups_count: 0,
            vertices_data_length: 0,
            indices_data_length: 0,
            start_offset: 0,
            lod_zoom_factor: 0.0,
            mysterious_data_1: None,
            mysterious_data_2: None,
        };
        match packed_file_data.decode_integer_u32(0) {
            Ok(data) => header.groups_count = data,
            Err(error) => return Err(error)
        }
        match packed_file_data.decode_integer_u32(4) {
            Ok(data) => header.vertices_data_length = data,
            Err(error) => return Err(error)
        }
        match packed_file_data.decode_integer_u32(8) {
            Ok(data) => header.indices_data_length = data,
            Err(error) => return Err(error)
        }
        match packed_file_data.decode_integer_u32(12) {
            Ok(data) => header.start_offset = data,
            Err(error) => return Err(error)
        }
        match packed_file_data.decode_float_f32(16) {
            Ok(data) => header.lod_zoom_factor = data,
            Err(error) => return Err(error)
        }

        // These two we only decode them if the RigidModel is v7 (Warhammer 1&2), as these doesn't exist
        // in Attila's RigidModels.
        if packed_file_data.len() == 28 {
            match packed_file_data.decode_integer_u32(20) {
                Ok(data) => header.mysterious_data_1 = Some(data),
                Err(error) => return Err(error)
            }
            match packed_file_data.decode_integer_u32(24) {
                Ok(data) => header.mysterious_data_2 = Some(data),
                Err(error) => return Err(error)
            }
        }

        Ok(header)
    }

    /// This function reads the data from a RigidModelLodHeader and encode it into a Vec<u8>.
    pub fn save(rigid_model_lod: &RigidModelLodHeader) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        packed_file_data.encode_integer_u32(rigid_model_lod.groups_count);
        packed_file_data.encode_integer_u32(rigid_model_lod.vertices_data_length);
        packed_file_data.encode_integer_u32(rigid_model_lod.indices_data_length);
        packed_file_data.encode_integer_u32(rigid_model_lod.start_offset);
        packed_file_data.encode_float_f32(rigid_model_lod.lod_zoom_factor);

        let mysterious_data_1 = match rigid_model_lod.mysterious_data_1 {
            Some(data) => Some(data),
            None => None,
        };

        let mysterious_data_2 = match rigid_model_lod.mysterious_data_2 {
            Some(data) => Some(data),
            None => None,
        };

        // These two are only added if they are something (Warhammer1&2 RigidModels).
        if mysterious_data_1 != None {
            packed_file_data.encode_integer_u32(mysterious_data_1.unwrap());
        }
        if mysterious_data_2 != None {
            packed_file_data.encode_integer_u32(mysterious_data_2.unwrap());
        }
        packed_file_data
    }
}
