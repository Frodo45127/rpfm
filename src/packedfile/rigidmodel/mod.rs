// In this file we define the PackedFile type RigidModel for decoding and encoding it.
// This is the type used by 3D model files of units and buildings. Both are different, so we need to
// take the type in count while processing them.

/*
--------------------------------------------------------
                RigidModel Structure
--------------------------------------------------------

------------------------------------
           Header
------------------------------------
- 4 bytes [String]: signature. Should always be "RMV2"!
- 4 bytes [u32]: model_type. 6 for Attila, 7 for Warhammer 1&2.
- 4 bytes [u32]: lods_count. Amount of lods in the RigidModel. Usually, there is a max of 5 in buildings. Units have more.
- 128 bytes [String] (0-Padded): base_skeleton. Used to import the default bones.
------------------------------------

------------------------------------
  Lods Headers (One after another)
------------------------------------
- 4 bytes [u32]: groups_count. Amount of groups in the Lod.
- 4 bytes [u32]: vertices_data_length. Length of the "vertices_data" of the Lod in bytes.
- 4 bytes [u32]: indices_data_length. Length of the "indices_data" of the Lod in bytes.
- 4 bytes [u32]: start_offset. Bytes from the beginning of the file to the beginning of the Lod's data.
- 4 bytes [f32]: zoom_factor. Max zoom of the Lod. For 4 Lods, this is always 100.0, 200.0, 400.0, 500.0.
- 4 bytes [u32]: mysterious_data_1 (only Warhammer 1&2). Seems to be an index (starting at 0) of the lods.
- 4 bytes [u32]: mysterious_data_2 (only Warhammer 1&2). No idea. Usually is 0 or a 4 byte encoded color (ARGB).
------------------------------------

------------------------------------
   Lods Data (One after another)
------------------------------------

////------------------------------------
       Lod Groups (One after another)
////------------------------------------
    - 4 bytes [u32]: rigid_material ID.
    - 4 bytes [u32]: lod_group_length. Length of the current Lod Group in bytes.
    - 4 bytes [u32]: lod_group_length_without_vertices_and_indices_data. Same as before minus the length in bytes of the "vertices_data" and "indices_data" sections.
    - 4 bytes [u32]: vertices_count. Amount of vertices in the Lod.
    - 4 bytes [u32]: lod_group_length_without_indices_data. Same as lod_group_length, minus the length of the "indices_data" section in bytes.
    - 4 bytes [u32]: indices_count. Amount of indices in the Lod.
    - 4 bytes [f32]: group_min_x.
    - 4 bytes [f32]: group_min_y.
    - 4 bytes [f32]: group_min_z.
    - 4 bytes [f32]: group_max_x.
    - 4 bytes [f32]: group_max_y.
    - 4 bytes [f32]: group_max_z.
    - 32 bytes [String] (0-Padded): shader_name. It's usually "default_dry". It usually stores some random useless info, we don't know why.
    - 2 bytes [u16]: No idea about this one. Sometimes it's 3, sometimes it's 0.
    - 32 bytes [String] (0-Padded): group_name. Name of the current group.
    - 256 bytes [String] (0-Padded): texture_directory. Directory for the textures.
    - 422 bytes [Vec<u8>]: No idea (maybe some shader stuff?), so we just store it in a Vec<u8>.
    - 4 bytes [u32]: supplementary_bones_count. Amount of "supplementary_bones" in the Lod.
    - 4 bytes [u32]: textures_count. Amount of "textures" in the Lod.
    - 140 bytes [Vec<u8>]: No idea, so we just store it in a Vec<u8>.

////////------------------------------------
     Supplementary Bones (One after another)
////////------------------------------------
        - 32 bytes [String] (0-Padded): bone_name. Name of the current bone.
        - 48 bytes [Vec<u8>]: No idea. Probably position, rotation and other things.
        - 4 bytes [u32]: bone_id.

////////------------------------------------

////////------------------------------------
     Supplementary Bones (One after another)
////////------------------------------------
        - 4 bytes [u32]: texture_type. The possible types are: 0 (Diffuse), 1 (Normal), 11 (Specular), 12 (Gloss), 3/10 (Mask).
        - 32 bytes [String] (0-Padded): bone_name. Name of the current bone.

////////------------------------------------
    - 4 bytes [u32]: separator. It's always 00 00 00 00, so 0u32.
    - 4 bytes [u32]: alpha_mode. The alpha mode of the Lod. The possible types are:
        - 00 00 00 00: Alpha mode 0 (alpha channel off).
        - 00 00 00 01: Alpha mode 1 (alpha channel on).
        - 00 00 00 02: Alpha mode 2 (alpha channel on).
        - FF FF FF FF: no idea about this one.

////////------------------------------------
          Vertices Data (One after another)
////////------------------------------------
        - 2 bytes [f16]: pos_x. Position X of the current vertice.
        - 2 bytes [f16]: pos_y. Position Y of the current vertice.
        - 2 bytes [f16]: pos_z. Position Z of the current vertice.
        - 2 bytes [u16]: separator. Always 00 00, so 0u16.
        - 1 bytes [u8]: first_bone_id.
        - 1 bytes [u8]: second_bone_id.
        - 1 bytes [u8]: vertex_weight. To get the proper value of this: "vertex_weight" / 255 = X.Xf
        - 1 bytes [u8]: separator. Always 00, so 0u8.
        - 1 bytes [u8]: vertex_normal_x. To get the proper value of this: (2 * "vertex_normal_x" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_normal_y. To get the proper value of this: (2 * "vertex_normal_y" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_normal_z. To get the proper value of this: (2 * "vertex_normal_z" / 255) - 1 = X.Xf
        - 1 bytes [u8]: separator. Always 00, so 0u8.
        - 2 bytes [f16]: pos_u. Position U of the current vertice.
        - 2 bytes [f16]: pos_v. Position V of the current vertice. To get the proper value of this: 1.0f - pos_v = X.Xf
        - 1 bytes [u8]: vertex_tangent_x. To get the proper value of this: (2 * "vertex_tangent_x" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_tangent_y. To get the proper value of this: (2 * "vertex_tangent_y" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_tangent_z. To get the proper value of this: (2 * "vertex_tangent_z" / 255) - 1 = X.Xf
        - 1 bytes [u8]: separator. Always 00, so 0u8.
        - 1 bytes [u8]: vertex_binormal_x. To get the proper value of this: (2 * "vertex_binormal_x" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_binormal_y. To get the proper value of this: (2 * "vertex_binormal_y" / 255) - 1 = X.Xf
        - 1 bytes [u8]: vertex_binormal_z. To get the proper value of this: (2 * "vertex_binormal_z" / 255) - 1 = X.Xf
        - 1 bytes [u8]: separator. Always 00, so 0u8.

////////------------------------------------

////////------------------------------------
          Indices Data (One after another). These are in packs of 3, so to get them you need to use (indices_count / 3).
////////------------------------------------
        - 2 bytes [u16]: index_1.
        - 2 bytes [u16]: index_2.
        - 2 bytes [u16]: index_3.

////////------------------------------------
    - extra_data [Vec<u8>]: bytes from here to the end of the Lod Group are unknown, so we just store them in Vec<u8>.

////------------------------------------

------------------------------------

--------------------------------------------------------
*/

extern crate half;

use self::half::f16;
use common::coding_helpers;
use std::error;
use std::io::{
    Error, ErrorKind
};

/// Struct "RigidModel". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModel {
    pub packed_file_header: RigidModelHeader,
    pub packed_file_data: RigidModelData,
}

/// Struct "RigidModelHeader". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelHeader {
    pub packed_file_header_signature: String,
    pub packed_file_header_model_type: u32,
    pub packed_file_header_lods_count: u32,
    pub packed_file_data_base_skeleton: (String, usize),
}

/// Struct "RigidModelData". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelData {
    pub packed_file_data_lod_list: Vec<RigidModelLod>,
    pub extra_data: Vec<u8>,
}

/// Struct "RigidModelLod". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelLod {
    pub groups_count: u32,
    pub vertex_data_length: u32,
    pub index_data_length: u32,
    pub start_offset: u32,
    pub lod_zoom_factor: f32,
    pub mysterious_data_1: Option<u32>,
    pub mysterious_data_2: Option<u32>,
}

/// Implementation of "RigidModel"
impl RigidModel {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModel. This CAN FAIL,
    /// so we return Result<RigidModel, Error>.
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModel, Error> {
        match RigidModelHeader::read(packed_file_data[..140].to_vec()) {
            Ok(packed_file_header) => {
                match RigidModelData::read(
                    packed_file_data[140..].to_vec(),
                    &packed_file_header.packed_file_header_model_type,
                    &packed_file_header.packed_file_header_lods_count
                ) {
                    Ok(packed_file_data) =>
                        Ok(RigidModel {
                            packed_file_header,
                            packed_file_data,
                        }),
                    Err(error) => Err(error)
                }
            }
            Err(error) => Err(error)
        }
    }

    /// This function reads the data from a RigidModel and encode it into a Vec<u8>.
    pub fn save(rigid_model_data: &mut RigidModel) -> Vec<u8> {
        let mut packed_file_data_encoded = RigidModelData::save(rigid_model_data.packed_file_data.clone());
        let mut packed_file_header_encoded = RigidModelHeader::save(rigid_model_data.packed_file_header.clone());

        let mut packed_file_encoded = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded);

        packed_file_encoded
    }
}

/// Implementation of "RigidModelHeader"
impl RigidModelHeader {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelHeader. This CAN FAIL,
    /// so we return Result<RigidModelHeader, Error>.
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelHeader, Error> {

        let mut packed_file_header = RigidModelHeader {
            packed_file_header_signature: String::new(),
            packed_file_header_model_type: 0,
            packed_file_header_lods_count: 0,
            packed_file_data_base_skeleton: (String::new(), 0),
        };

        match coding_helpers::decode_string_u8((&packed_file_data[0..4]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_signature = data,
            Err(error) => return Err(error)
        }

        // We check this, just in case we try to read some malformed file with a string in the first
        // four bytes (which is not uncommon).
        if packed_file_header.packed_file_header_signature != "RMV2" {
            return Err(Error::new(ErrorKind::Other, format!("This is not a RMV2 RigidModel.")))
        }

        match coding_helpers::decode_integer_u32((&packed_file_data[4..8]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_model_type = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_integer_u32((&packed_file_data[8..12]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_lods_count = data,
            Err(error) => return Err(error)
        }

        packed_file_header.packed_file_data_base_skeleton = coding_helpers::decode_string_u8_0padded((&packed_file_data[12..140]).to_vec());
        Ok(packed_file_header)
    }

    /// This function reads the data from a RigidModelHeader and encode it into a Vec<u8>.
    pub fn save(rigid_model_header: RigidModelHeader) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut packed_file_header_signature = coding_helpers::encode_string_u8(rigid_model_header.packed_file_header_signature);
        let mut packed_file_header_model_type = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_model_type);
        let mut packed_file_header_lods_count = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_lods_count);
        let mut packed_file_data_base_skeleton = coding_helpers::encode_string_u8_0padded(rigid_model_header.packed_file_data_base_skeleton);

        packed_file_data.append(&mut packed_file_header_signature);
        packed_file_data.append(&mut packed_file_header_model_type);
        packed_file_data.append(&mut packed_file_header_lods_count);
        packed_file_data.append(&mut packed_file_data_base_skeleton);
        packed_file_data
    }
}

/// Implementation of "RigidModelData"
impl RigidModelData {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelData. This CAN FAIL,
    /// so we return Result<RigidModelData, Error>.
    pub fn read(packed_file_data: Vec<u8>, packed_file_header_model_type: &u32, packed_file_header_lods_count: &u32) -> Result<RigidModelData, Error> {
        let mut packed_file_data_lod_list: Vec<RigidModelLod> = vec![];
        let mut index: usize = 0;
        let offset: usize = match *packed_file_header_model_type {
            6 => 20, // Attila
            7 => 28, // Warhammer 1&2
            _ => return Err(Error::new(ErrorKind::Other, format!("RigidModel model not yet decodeable.")))
        };

        // We get the "headers" of every lod.
        for _ in 0..*packed_file_header_lods_count {
            let lod = match RigidModelLod::read(packed_file_data[index..(index + offset)].to_vec()) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lod_list.push(lod);
            index += offset;
        }

        // In the future we want to decode this data properly. For now, we just store it.
        let extra_data = packed_file_data[index..].to_vec();

        Ok(RigidModelData {
            packed_file_data_lod_list,
            extra_data
        })
    }

    /// This function reads the data from a RigidModelData and encode it into a Vec<u8>.
    pub fn save(mut rigid_model_data: RigidModelData) -> Vec<u8> {
        let mut packed_file_data = vec![];

        // For each Lod, we save it, and add it to the "Encoded Data" vector. After that, we add to that
        // vector the extra data, and return it.
        for lod in rigid_model_data.packed_file_data_lod_list.iter() {
            packed_file_data.append(&mut RigidModelLod::save(lod.clone()));
        }
        packed_file_data.append(&mut rigid_model_data.extra_data);
        packed_file_data
    }
}

/// Implementation of "RigidModelLod"
impl RigidModelLod {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelLod. This CAN FAIL,
    /// so we return Result<RigidModelLod, Error>.
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLod, Error> {
        let mut header = RigidModelLod {
            groups_count: 0,
            vertex_data_length: 0,
            index_data_length: 0,
            start_offset: 0,
            lod_zoom_factor: 0.0,
            mysterious_data_1: None,
            mysterious_data_2: None,
        };
        match coding_helpers::decode_integer_u32((&packed_file_data[0..4]).to_vec()) {
            Ok(data) => header.groups_count = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[4..8]).to_vec()) {
            Ok(data) => header.vertex_data_length = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[8..12]).to_vec()) {
            Ok(data) => header.index_data_length = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[12..16]).to_vec()) {
            Ok(data) => header.start_offset = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_float_u32((&packed_file_data[16..20]).to_vec()) {
            Ok(data) => header.lod_zoom_factor = data,
            Err(error) => return Err(error)
        }

        // These two we only decode them if the RigidModel is v7 (Warhammer 1&2), as these doesn't exist
        // in Attila's RigidModels.
        if packed_file_data.len() == 28 {
            match coding_helpers::decode_integer_u32((&packed_file_data[20..24]).to_vec()) {
                Ok(data) => header.mysterious_data_1 = Some(data),
                Err(error) => return Err(error)
            }
            match coding_helpers::decode_integer_u32((&packed_file_data[24..28]).to_vec()) {
                Ok(data) => header.mysterious_data_2 = Some(data),
                Err(error) => return Err(error)
            }
        }

        Ok(header)
    }

    /// This function reads the data from a RigidModelLod and encode it into a Vec<u8>.
    pub fn save(rigid_model_lod: RigidModelLod) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut groups_count = coding_helpers::encode_integer_u32(rigid_model_lod.groups_count);
        let mut vertex_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.vertex_data_length);
        let mut index_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.index_data_length);
        let mut start_offset = coding_helpers::encode_integer_u32(rigid_model_lod.start_offset);
        let mut lod_zoom_factor = coding_helpers::encode_float_u32(rigid_model_lod.lod_zoom_factor);

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