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
- 128 bytes [String] (0-Padded): base_skeleton. Used to import the default bones. If something, then it's an unit.
------------------------------------

------------------------------------
  Lods Headers (One after another)
------------------------------------
- 4 bytes [u32]: groups_count. Amount of groups in the Lod.
- 4 bytes [u32]: vertices_data_length. Length of the "vertices_data" of the Lod in bytes. If 0, then it's a decal.
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
    - 12 bytes [String] (0-Padded): shader_name. It's usually "default_dry".
    - 4 bytes [u32]: No idea. Maybe max texture Size?.
    - 4 bytes [u32]: No idea. It's always 32.
    - 12 bytes [Vec<u8>]: mmm maybe a separator?
    - 2 bytes [Option<u16>]: No idea about this one. Sometimes it's 3, sometimes it's 0. It doesn't exist in decals.
    - 32 bytes [Option<String>] (0-Padded): group_name. Name of the current group. It doesn't exist in decals.
    - 256 bytes [String] (0-Padded): texture_directory. Directory for the textures.
    - 422 bytes [Option<Vec<u8>>]: No idea (maybe some shader stuff?), so we just store it in a Vec<u8>. It doesn't exist in decals.
    - 4 bytes [Option<u32>]: supplementary_bones_count. Amount of "supplementary_bones" in the Lod. It doesn't exist in decals.
    - 4 bytes [Option<u32>]: textures_count. Amount of "textures" in the Lod. It doesn't exist in decals.
    - 140 bytes [Option<Vec<u8>>]: No idea, so we just store it in a Vec<u8>. It doesn't exist in decals.

////////------------------------------------
     Supplementary Bones (One after another)
////////------------------------------------
        - 32 bytes [String] (0-Padded): bone_name. Name of the current bone.
        - 48 bytes [Vec<u8>]: No idea. Probably position, rotation and other things.
        - 4 bytes [u32]: bone_id.

////////------------------------------------

////////------------------------------------
            Textures (One after another)
////////------------------------------------
        - 4 bytes [u32]: texture_type. The possible types are: 0 (Diffuse), 1 (Normal), 11 (Specular), 12 (Gloss), 3/10 (Mask).
        - 32 bytes [String] (0-Padded): texture_path. The relative path (to /data) of the texture.

////////------------------------------------
    //////// NOTE: This entire part doesn't exist in Decals.
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

use common::coding_helpers;
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
    pub packed_file_data_lods_header: Vec<RigidModelLodHeader>,
    pub packed_file_data_lods_data: Vec<RigidModelLodData>,
}

/// Struct "RigidModelLodHeader". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelLodHeader {
    pub groups_count: u32,
    pub vertices_data_length: u32,
    pub indices_data_length: u32,
    pub start_offset: u32,
    pub lod_zoom_factor: f32,
    pub mysterious_data_1: Option<u32>,
    pub mysterious_data_2: Option<u32>,
}

/// Struct "RigidModelLodData". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelLodData {
    pub rigid_material_id: u32,
    pub lod_length: u32,
    pub lod_length_without_vertices_and_indices_length: u32,
    pub vertices_count: u32,
    pub lod_length_without_indices_length: u32,
    pub indices_count: u32,
    pub group_min_x: f32,
    pub group_min_y: f32,
    pub group_min_z: f32,
    pub group_max_x: f32,
    pub group_max_y: f32,
    pub group_max_z: f32,
    pub shader_name: (String, usize),
    pub mysterious_u32_1: u32,
    pub mysterious_u32_2: u32,
    pub mysterious_data_1: Vec<u8>,
    pub mysterious_id: Option<u16>,
    pub group_name: Option<(String, usize)>,
    pub textures_directory: (String, usize),
    pub mysterious_data_2: Option<Vec<u8>>,
    pub supplementary_bones_count: Option<u32>,
    pub textures_count: Option<u32>,
    pub mysterious_data_3: Option<Vec<u8>>,
    pub supplementary_bones_list: Option<Vec<u8>>,
    pub textures_list: Option<Vec<RigidModelLodDataTexture>>,
    pub separator: Option<u32>,
    pub alpha_mode: Option<u32>,
    pub vertices_list: Option<Vec<u8>>,
    pub indices_list: Option<Vec<u8>>,
    pub extra_bytes: Option<Vec<u8>>,
}

/// Struct "RigidModelLodDataTexture". For more info about this, check the comment at the start of "packedfile/
/// rigidmodel/mod.rs".
#[derive(Clone, Debug)]
pub struct RigidModelLodDataTexture {
    pub texture_type: u32,
    pub texture_path: (String, usize),
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

    /// This function reads the data from a RigidModel and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_data: &RigidModel) -> Result<Vec<u8>, Error> {
        let mut packed_file_data_encoded = RigidModelData::save(rigid_model_data.packed_file_data.clone())?;
        let mut packed_file_header_encoded = RigidModelHeader::save(rigid_model_data.packed_file_header.clone())?;

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

    /// This function reads the data from a RigidModelHeader and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_header: RigidModelHeader) -> Result<Vec<u8>, Error> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut packed_file_header_signature = coding_helpers::encode_string_u8(rigid_model_header.packed_file_header_signature);
        let mut packed_file_header_model_type = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_model_type);
        let mut packed_file_header_lods_count = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_lods_count);
        let mut packed_file_data_base_skeleton = coding_helpers::encode_string_u8_0padded(rigid_model_header.packed_file_data_base_skeleton)?;

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
    pub fn read(packed_file_data: Vec<u8>, packed_file_header_model_type: &u32, packed_file_header_lods_count: &u32) -> Result<RigidModelData, Error> {
        let mut packed_file_data_lods_header: Vec<RigidModelLodHeader> = vec![];
        let mut packed_file_data_lods_data: Vec<RigidModelLodData> = vec![];
        let mut index: usize = 0;
        let offset: usize = match *packed_file_header_model_type {
            6 => 20, // Attila
            7 => 28, // Warhammer 1&2
            _ => return Err(Error::new(ErrorKind::Other, format!("RigidModel model not yet decodeable.")))
        };

        // We get the "headers" of every lod.
        for _ in 0..*packed_file_header_lods_count {
            let lod_header = match RigidModelLodHeader::read(packed_file_data[index..(index + offset)].to_vec()) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lods_header.push(lod_header);
            index += offset;
        }

        for lod in 0..*packed_file_header_lods_count {
            let lod_data = match RigidModelLodData::read(packed_file_data[index..].to_vec(), &packed_file_data_lods_header[lod as usize]) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            index += lod_data.lod_length as usize;
            packed_file_data_lods_data.push(lod_data);
        }

        Ok(RigidModelData {
            packed_file_data_lods_header,
            packed_file_data_lods_data,
        })
    }

    /// This function reads the data from a RigidModelData and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_data: RigidModelData) -> Result<Vec<u8>, Error> {
        let mut packed_file_data = vec![];

        // For each Lod, we save it, and add it to the "Encoded Data" vector. After that, we add to that
        // vector the extra data, and return it.
        for lod in rigid_model_data.packed_file_data_lods_header.iter() {
            packed_file_data.append(&mut RigidModelLodHeader::save(&lod));
        }

        for (index, lod) in rigid_model_data.packed_file_data_lods_data.iter().enumerate() {
            packed_file_data.append(&mut RigidModelLodData::save(&lod, &rigid_model_data.packed_file_data_lods_header[index])?);
        }
        Ok(packed_file_data)
    }
}

/// Implementation of "RigidModelLodHeader"
impl RigidModelLodHeader {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelLodHeader. This CAN FAIL,
    /// so we return Result<RigidModelLodHeader, Error>.
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodHeader, Error> {
        let mut header = RigidModelLodHeader {
            groups_count: 0,
            vertices_data_length: 0,
            indices_data_length: 0,
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
            Ok(data) => header.vertices_data_length = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[8..12]).to_vec()) {
            Ok(data) => header.indices_data_length = data,
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

    /// This function reads the data from a RigidModelLodHeader and encode it into a Vec<u8>.
    pub fn save(rigid_model_lod: &RigidModelLodHeader) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut groups_count = coding_helpers::encode_integer_u32(rigid_model_lod.groups_count);
        let mut vertex_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.vertices_data_length);
        let mut index_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.indices_data_length);
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

/// Implementation of "RigidModelLodData"
impl RigidModelLodData {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelLodData. This CAN FAIL,
    /// so we return Result<RigidModelDataLod, Error>. Also, it needs it's LodHeader to see what type of
    /// model it's.
    pub fn read(packed_file_data: Vec<u8>, packed_file_lod_header: &RigidModelLodHeader) -> Result<RigidModelLodData, Error> {

        let mut index = 0;

        let rigid_material_id = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let lod_length = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let lod_length_without_vertices_and_indices_length = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let vertices_count = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let lod_length_without_indices_length = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let indices_count = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_min_x = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_min_y = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_min_z = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_max_x = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_max_y = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let group_max_z = match coding_helpers::decode_packedfile_float_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let shader_name = coding_helpers::decode_string_u8_0padded((&packed_file_data[index..(index + 12)]).to_vec());
        index += 12;

        let mysterious_u32_1 = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let mysterious_u32_2 = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
            Ok(data) => {
                index = data.1;
                data.0
            },
            Err(error) => return Err(error)
        };
        let mysterious_data_1 = (&packed_file_data[index..(index + 12)]).to_vec();
        index += 12;

        // From here, everything except textures_directory can return None. It depends on the RigidModel Type.
        let mysterious_id;
        let group_name;
        let textures_directory;
        let mysterious_data_2;
        let supplementary_bones_count;
        let textures_count;
        let mysterious_data_3;
        let supplementary_bones_list;
        let textures_list;
        let separator;
        let alpha_mode;
        let vertices_list;
        let indices_list;
        let extra_bytes;

        // If the vertices_data_length is 0, it's a decal.
        if packed_file_lod_header.vertices_data_length == 0 {
            mysterious_id = None;
            group_name = None;
            textures_directory = coding_helpers::decode_string_u8_0padded((&packed_file_data[index..(index + 256)]).to_vec());
            index += 256;

            mysterious_data_2 = None;
            supplementary_bones_count = None;
            textures_count = None;
            mysterious_data_3 = None;
            supplementary_bones_list = None;
            textures_list = None;
            separator = None;
            alpha_mode = None;
            vertices_list = None;
            indices_list = Some((&packed_file_data[index..(index + (lod_length as usize - index))]).to_vec());
            extra_bytes = None;
        }

        // Else, it's a building or unit RigidModel.
        else {
            mysterious_id = match coding_helpers::decode_packedfile_integer_u16((&packed_file_data[index..(index + 2)]).to_vec(), index) {
                Ok(data) => {
                    index = data.1;
                    Some(data.0)
                },
                Err(error) => return Err(error)
            };
            group_name = Some(coding_helpers::decode_string_u8_0padded((&packed_file_data[index..(index + 32)]).to_vec()));
            index += group_name.clone().unwrap().1;

            textures_directory = coding_helpers::decode_string_u8_0padded((&packed_file_data[index..(index + 256)]).to_vec());
            index += textures_directory.1;

            mysterious_data_2 = Some(packed_file_data[index..(index + 422)].to_vec());
            index += 422;

            supplementary_bones_count = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
                Ok(data) => {
                    index = data.1;
                    Some(data.0)
                },
                Err(error) => return Err(error)
            };
            textures_count = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
                Ok(data) => {
                    index = data.1;
                    Some(data.0)
                },
                Err(error) => return Err(error)
            };
            mysterious_data_3 = Some(packed_file_data[index..(index + 140)].to_vec());
            index += 140;

            if supplementary_bones_count.unwrap() > 0 {
                let supplementary_bones_list_length = (84 * supplementary_bones_count.unwrap()) as usize;
                supplementary_bones_list = Some((&packed_file_data[index..(index + supplementary_bones_list_length)]).to_vec());
                index += supplementary_bones_list_length;
            }
            else {
                supplementary_bones_list = None;
            }

            if textures_count.unwrap() > 0 {
                let mut temp_texture_list = vec![];
                for _ in 0..textures_count.unwrap() {
                    match RigidModelLodDataTexture::read((&packed_file_data[index..(index + 260)]).to_vec()){
                        Ok(data) => {
                            temp_texture_list.push(data);
                            index += 260;
                        }
                        Err(error) => return Err(error),
                    }
                }
                textures_list = Some(temp_texture_list.clone());
            }
            else {
                textures_list = None;
            }

            separator = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
                Ok(data) => {
                    index = data.1;
                    Some(data.0)
                },
                Err(error) => return Err(error)
            };
            alpha_mode = match coding_helpers::decode_packedfile_integer_u32((&packed_file_data[index..(index + 4)]).to_vec(), index) {
                Ok(data) => {
                    index = data.1;
                    Some(data.0)
                },
                Err(error) => return Err(error)
            };

            if vertices_count > 0 {
                let vertices_list_length = (28 * vertices_count) as usize;
                vertices_list = Some((&packed_file_data[index..(index + vertices_list_length)]).to_vec());
                index += vertices_list_length;
            }
            else {
                vertices_list = None;
            }

            if indices_count > 0 {
                let indices_list_length = (3 * indices_count) as usize;
                indices_list = Some((&packed_file_data[index..(index + indices_list_length)]).to_vec());
                index += indices_list_length;
            }
            else {
                indices_list = None;
            }
            extra_bytes = Some((&packed_file_data[index..(index + (lod_length as usize - index))]).to_vec());
        }

        Ok(RigidModelLodData {
            rigid_material_id,
            lod_length,
            lod_length_without_vertices_and_indices_length,
            vertices_count,
            lod_length_without_indices_length,
            indices_count,
            group_min_x,
            group_min_y,
            group_min_z,
            group_max_x,
            group_max_y,
            group_max_z,
            shader_name,
            mysterious_u32_1,
            mysterious_u32_2,
            mysterious_data_1,
            mysterious_id,
            group_name,
            textures_directory,
            mysterious_data_2,
            supplementary_bones_count,
            textures_count,
            mysterious_data_3,
            supplementary_bones_list,
            textures_list,
            separator,
            alpha_mode,
            vertices_list,
            indices_list,
            extra_bytes,
        })
    }

    /// This function reads the data from a RigidModelLodDataTexture and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_lod_data: &RigidModelLodData, rigid_model_lod_header: &RigidModelLodHeader) -> Result<Vec<u8>, Error> {
        let rigid_model_lod_data = rigid_model_lod_data.clone();
        let mut packed_file_data: Vec<u8> = vec![];

        let mut rigid_material_id = coding_helpers::encode_integer_u32(rigid_model_lod_data.rigid_material_id);
        let mut lod_length = coding_helpers::encode_integer_u32(rigid_model_lod_data.lod_length);
        let mut lod_length_without_vertices_and_indices_length = coding_helpers::encode_integer_u32(rigid_model_lod_data.lod_length_without_vertices_and_indices_length);
        let mut vertices_count = coding_helpers::encode_integer_u32(rigid_model_lod_data.vertices_count);
        let mut lod_length_without_indices_length = coding_helpers::encode_integer_u32(rigid_model_lod_data.lod_length_without_indices_length);
        let mut indices_count = coding_helpers::encode_integer_u32(rigid_model_lod_data.indices_count);
        let mut group_min_x = coding_helpers::encode_float_u32(rigid_model_lod_data.group_min_x);
        let mut group_min_y = coding_helpers::encode_float_u32(rigid_model_lod_data.group_min_y);
        let mut group_min_z = coding_helpers::encode_float_u32(rigid_model_lod_data.group_min_z);
        let mut group_max_x = coding_helpers::encode_float_u32(rigid_model_lod_data.group_max_x);
        let mut group_max_y = coding_helpers::encode_float_u32(rigid_model_lod_data.group_max_y);
        let mut group_max_z = coding_helpers::encode_float_u32(rigid_model_lod_data.group_max_z);

        let mut shader_name = coding_helpers::encode_string_u8_0padded(rigid_model_lod_data.shader_name)?;
        let mut mysterious_u32_1 = coding_helpers::encode_integer_u32(rigid_model_lod_data.mysterious_u32_1);
        let mut mysterious_u32_2 = coding_helpers::encode_integer_u32(rigid_model_lod_data.mysterious_u32_2);
        let mut mysterious_data_1 = rigid_model_lod_data.mysterious_data_1.to_vec();

        packed_file_data.append(&mut rigid_material_id);
        packed_file_data.append(&mut lod_length);
        packed_file_data.append(&mut lod_length_without_vertices_and_indices_length);
        packed_file_data.append(&mut vertices_count);
        packed_file_data.append(&mut lod_length_without_indices_length);
        packed_file_data.append(&mut indices_count);
        packed_file_data.append(&mut group_min_x);
        packed_file_data.append(&mut group_min_y);
        packed_file_data.append(&mut group_min_z);
        packed_file_data.append(&mut group_max_x);
        packed_file_data.append(&mut group_max_y);
        packed_file_data.append(&mut group_max_z);

        packed_file_data.append(&mut shader_name);
        packed_file_data.append(&mut mysterious_u32_1);
        packed_file_data.append(&mut mysterious_u32_2);
        packed_file_data.append(&mut mysterious_data_1);

        // If it's a decal.
        if rigid_model_lod_header.vertices_data_length == 0 {
            let mut textures_directory = coding_helpers::encode_string_u8_0padded(rigid_model_lod_data.textures_directory)?;
            let mut indices_list = rigid_model_lod_data.indices_list.unwrap().to_vec();

            packed_file_data.append(&mut textures_directory);
            packed_file_data.append(&mut indices_list);
        }
        else {
            let mut mysterious_id = coding_helpers::encode_integer_u16(rigid_model_lod_data.mysterious_id.unwrap());
            let mut group_name = coding_helpers::encode_string_u8_0padded(rigid_model_lod_data.group_name.unwrap())?;
            let mut textures_directory = coding_helpers::encode_string_u8_0padded(rigid_model_lod_data.textures_directory)?;
            let mut mysterious_data_2 = rigid_model_lod_data.mysterious_data_2.unwrap().to_vec();
            let mut supplementary_bones_count = coding_helpers::encode_integer_u32(rigid_model_lod_data.supplementary_bones_count.unwrap());
            let mut textures_count = coding_helpers::encode_integer_u32(rigid_model_lod_data.textures_count.unwrap());
            let mut mysterious_data_3 = rigid_model_lod_data.mysterious_data_3.unwrap().to_vec();

            packed_file_data.append(&mut mysterious_id);
            packed_file_data.append(&mut group_name);
            packed_file_data.append(&mut textures_directory);
            packed_file_data.append(&mut mysterious_data_2);
            packed_file_data.append(&mut supplementary_bones_count);
            packed_file_data.append(&mut textures_count);
            packed_file_data.append(&mut mysterious_data_3);


            if rigid_model_lod_data.supplementary_bones_count.unwrap() != 0 {
                let mut supplementary_bones_list = rigid_model_lod_data.supplementary_bones_list.unwrap().to_vec();
                packed_file_data.append(&mut supplementary_bones_list);
            }

            if rigid_model_lod_data.textures_count.unwrap() != 0 {
                let mut textures_list: Vec<u8> = vec![];
                for texture in rigid_model_lod_data.textures_list.unwrap().iter() {
                    textures_list.append(&mut RigidModelLodDataTexture::save(texture)?);
                }
                packed_file_data.append(&mut textures_list);
            }

            let mut separator = coding_helpers::encode_integer_u32(rigid_model_lod_data.separator.unwrap());
            let mut alpha_mode = coding_helpers::encode_integer_u32(rigid_model_lod_data.alpha_mode.unwrap());

            let mut vertices_list = rigid_model_lod_data.vertices_list.unwrap().to_vec();
            let mut indices_list = rigid_model_lod_data.indices_list.unwrap().to_vec();
            let mut extra_bytes = rigid_model_lod_data.extra_bytes.unwrap().to_vec();

            packed_file_data.append(&mut separator);
            packed_file_data.append(&mut alpha_mode);
            packed_file_data.append(&mut vertices_list);
            packed_file_data.append(&mut indices_list);
            packed_file_data.append(&mut extra_bytes);
        }
        Ok(packed_file_data)
    }
}

/// Implementation of "RigidModelLocDataTexture"
impl RigidModelLodDataTexture {

    /// This function reads the data from a Vec<u8> and decode it into a RigidModelLodDataTexture. This CAN FAIL,
    /// so we return Result<RigidModelLodDataTexture, Error>.
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodDataTexture, Error> {

        let texture_type = match coding_helpers::decode_integer_u32((&packed_file_data[0..4]).to_vec()) {
            Ok(data) => data,
            Err(error) => return Err(error)
        };
        let texture_path = coding_helpers::decode_string_u8_0padded((&packed_file_data[4..260]).to_vec());

        Ok(RigidModelLodDataTexture {
            texture_type,
            texture_path
        })
    }

    /// This function reads the data from a RigidModelLodDataTexture and encode it into a Vec<u8>. This CAN FAIL,
    /// so we return Result<Vec<u8>, Error>.
    pub fn save(rigid_model_lod_texture: &RigidModelLodDataTexture) -> Result<Vec<u8>, Error> {
        let rigid_model_lod_texture = rigid_model_lod_texture.clone();
        let mut packed_file_data: Vec<u8> = vec![];

        let mut texture_type = coding_helpers::encode_integer_u32(rigid_model_lod_texture.texture_type);
        let mut texture_path = coding_helpers::encode_string_u8_0padded(rigid_model_lod_texture.texture_path)?;

        packed_file_data.append(&mut texture_type);
        packed_file_data.append(&mut texture_path);

        Ok(packed_file_data)
    }
}