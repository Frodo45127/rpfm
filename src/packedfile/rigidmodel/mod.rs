// In this file we define the PackedFile type RigidModel for decoding and encoding it.
// This is the type used by 3D model files of units and buildings. Both are different, so we need to
// take the type in count while processing them.

use common::coding_helpers;
use error::{ErrorKind, Result};

/// Struct "RigidModel". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RigidModel {
    pub packed_file_header: RigidModelHeader,
    pub packed_file_data: RigidModelData,
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
        let packed_file_data = RigidModelData::read(&packed_file_data[140..], &packed_file_header.packed_file_header_model_type, &packed_file_header.packed_file_header_lods_count)?;

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

        match coding_helpers::decode_string_u8(&packed_file_data[0..4]) {
            Ok(data) => packed_file_header.packed_file_header_signature = data,
            Err(error) => return Err(error)
        }

        // We check this, just in case we try to read some malformed file with a string in the first
        // four bytes (which is not uncommon).
        if packed_file_header.packed_file_header_signature != "RMV2" {
            return Err(ErrorKind::RigidModelNotSupportedFile)?
        }

        match coding_helpers::decode_integer_u32(&packed_file_data[4..8]) {
            Ok(data) => packed_file_header.packed_file_header_model_type = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_integer_u32(&packed_file_data[8..12]) {
            Ok(data) => packed_file_header.packed_file_header_lods_count = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_string_u8_0padded(&packed_file_data[12..140]) {
            Ok(data) => packed_file_header.packed_file_data_base_skeleton = data,
            Err(error) => return Err(error)
        }
        Ok(packed_file_header)
    }

    /// This function reads the data from a RigidModelHeader and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_header: &RigidModelHeader) -> Result<Vec<u8>> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut packed_file_header_signature = coding_helpers::encode_string_u8(&rigid_model_header.packed_file_header_signature);
        let mut packed_file_header_model_type = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_model_type);
        let mut packed_file_header_lods_count = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_lods_count);
        let mut packed_file_data_base_skeleton = coding_helpers::encode_string_u8_0padded(&rigid_model_header.packed_file_data_base_skeleton)?;

        packed_file_data.append(&mut packed_file_header_signature);
        packed_file_data.append(&mut packed_file_header_model_type);
        packed_file_data.append(&mut packed_file_header_lods_count);
        packed_file_data.append(&mut packed_file_data_base_skeleton);
        Ok(packed_file_data)
    }
}

/// Implementation of "RigidModelData"
impl RigidModelData {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelData. This CAN FAIL,
    /// so we return Result<RigidModelData, Error>.
    pub fn read(packed_file_data: &[u8], packed_file_header_model_type: &u32, packed_file_header_lods_count: &u32) -> Result<RigidModelData> {
        let mut packed_file_data_lods_header: Vec<RigidModelLodHeader> = vec![];
        let mut index: usize = 0;
        let offset: usize = match *packed_file_header_model_type {
            6 => 20, // Attila
            7 => 28, // Warhammer 1&2
            _ => return Err(ErrorKind::RigidModelNotSupportedType)?
        };

        // We get the "headers" of every lod.
        for _ in 0..*packed_file_header_lods_count {
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
        match coding_helpers::decode_integer_u32(&packed_file_data[0..4]) {
            Ok(data) => header.groups_count = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32(&packed_file_data[4..8]) {
            Ok(data) => header.vertices_data_length = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32(&packed_file_data[8..12]) {
            Ok(data) => header.indices_data_length = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32(&packed_file_data[12..16]) {
            Ok(data) => header.start_offset = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_float_f32(&packed_file_data[16..20]) {
            Ok(data) => header.lod_zoom_factor = data,
            Err(error) => return Err(error)
        }

        // These two we only decode them if the RigidModel is v7 (Warhammer 1&2), as these doesn't exist
        // in Attila's RigidModels.
        if packed_file_data.len() == 28 {
            match coding_helpers::decode_integer_u32(&packed_file_data[20..24]) {
                Ok(data) => header.mysterious_data_1 = Some(data),
                Err(error) => return Err(error)
            }
            match coding_helpers::decode_integer_u32(&packed_file_data[24..28]) {
                Ok(data) => header.mysterious_data_2 = Some(data),
                Err(error) => return Err(error)
            }
        }

        Ok(header)
    }

    /// This function reads the data from a RigidModelLodHeader and encode it into a Vec<u8>.
    pub fn save(rigid_model_lod: &RigidModelLodHeader) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut groups_count = coding_helpers::encode_integer_u32(rigid_model_lod.groups_count);
        let mut vertex_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.vertices_data_length);
        let mut index_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.indices_data_length);
        let mut start_offset = coding_helpers::encode_integer_u32(rigid_model_lod.start_offset);
        let mut lod_zoom_factor = coding_helpers::encode_float_f32(rigid_model_lod.lod_zoom_factor);

        let mysterious_data_1 = match rigid_model_lod.mysterious_data_1 {
            Some(data) => Some(data),
            None => None,
        };

        let mysterious_data_2 = match rigid_model_lod.mysterious_data_2 {
            Some(data) => Some(data),
            None => None,
        };

        packed_file_data.append(&mut groups_count);
        packed_file_data.append(&mut vertex_data_length);
        packed_file_data.append(&mut index_data_length);
        packed_file_data.append(&mut start_offset);
        packed_file_data.append(&mut lod_zoom_factor);

        // These two are only added if they are something (Warhammer1&2 RigidModels).
        if mysterious_data_1 != None {
            let mut mysterious_data_1 = coding_helpers::encode_integer_u32(mysterious_data_1.unwrap());
            packed_file_data.append(&mut mysterious_data_1);
        }
        if mysterious_data_2 != None {
            let mut mysterious_data_2 = coding_helpers::encode_integer_u32(mysterious_data_2.unwrap());
            packed_file_data.append(&mut mysterious_data_2);
        }
        packed_file_data
    }
}
