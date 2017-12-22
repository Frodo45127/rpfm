// In this file we define the PackedFile type RigidModel for decoding and encoding it.
// This is the type used by 3D model files of units and buildings. Both are different, so we need to
// take the type in count while processing them.

/*
4 Bytes [String] - Signature // Should always be "RMV2"!
4 Bytes [UInt32] - ModelType // 6 for Attila, 7 for Warhammer.
4 Bytes [UInt32] - LodsCount // Can be any of: 1, 2, 3 and 4
128 Bytes [0-Padded String] - BaseSkeleton // Used to import the default bones.

for (Int32 i = 0; i < LodsCount; ++i)

     4 Bytes [UInt32] - GroupsCount
     4 Bytes [UInt32] - VerticesData bytes count
     4 Bytes [UInt32] - IndicesData bytes count
     4 Bytes [UInt32] - StartOffset // From file's origin to each of lod's beginning.
     4 Bytes [Float] - ZoomFactor // For 4 LoDs this is always 100.0, 200.0, 400.0, 500.0.

for (Int32 i = 0; i < LodsCount; ++i)
     for (Int32 j = 0; j < Lod[i].GroupsCount; ++j)

          4 Bytes [UInt32] - rigid_material ID.
          4 Bytes [UInt32] - LODBytes count // Amount of bytes per each lod. Some kind of an offset.
          4 Bytes [UInt32] - LODBytes count without Vertices and Indices bytes count. // LODBytes count - (VerticesData count + IndicesData count)
          4 Bytes [UInt32] - VerticesCount
          4 Bytes [UInt32] - LODBytes count without Indices bytes count. // LODBytes count - IndicesData count.
          4 Bytes [UInt32] - IndicesCount
          4 Bytes [Float] - GroupMinimumX
          4 Bytes [Float] - GroupMinimumY
          4 Bytes [Float] - GroupMinimumZ
          4 Bytes [Float] - GroupMaximumX
          4 Bytes [Float] - GroupMaximumY
          4 Bytes [Float] - GroupMaximumZ
          32 Bytes [0-Padded String] - ShaderName // After default_dry there are absolutely random information recorded. Even if you export and covert EXACTLY the same file few times, these values will be always different. Seems like they make no effect and are a part of a shader name's bytes.
          2 Bytes [UInt32] - ? // Probably ID? It's always 3 at the moment.
          32 Bytes [0-Padded String] - GroupName
          256 Bytes [0-Padded String] - TexturesDirectory
          422 Bytes - ? // 422 bytes of perplexity... shader settings? 4 bytes in the middle of this block change if
                           I scale the whole model so it's probably not a single block!
          4 Bytes [UInt32] - SupplementarBonesCount
          4 Bytes [UInt32] - TexturesCount
          140 Bytes - ? // No idea.

          for (Int32 y = 0; y < SupplementarBonesCount; ++y)
               32 Bytes [0-Padded String] - SupplementarBoneName
               48 Bytes - ? // Probably position, rotation and other things.
               4 Bytes [UInt32] - SupplementarBoneID

          for (Int32 y = 0; y < TexturesCount; ++y)
               4 Bytes [UInt32] - TextureType // 0 Diffuse, 1 Normal, 11 Specular, 12 Gloss, Mask 3 or 10
               256 Bytes [0-Padded String]- TexturePath (TexturesDirectory + FileName)

          4 Bytes - Separator // It's always 00.00.00.00.
          4 Bytes - Alpha Mode // 00 00 00 00 - Alpha mode 0 (alpha channel off), 00 00 00 01 - alpha mode 1 (alpha channel on), 00 00 00 02 - alpha mode 2 (alpha channel on). Sometimes it's FF FF FF FF (no idea about this one as of yet).

          for (Int32 y = 0; y < VerticesCount; ++y)
               2 Bytes [Float] - Position X
               2 Bytes [Float] - Position Y
               2 Bytes [Float] - Position Z
               2 Bytes - Separator. It's always 00.00
               Byte [Unsigned Byte] - First bone ID
               Byte [Unsigned Byte] - Second bone ID
               Byte [Unsigned Byte] - Vertex weight // Divide this value on 255 (for example, 127/255 = 0.5f)
               Byte - Separator. It's always 00
               Byte [Unsigned Byte] - Vertex normal X // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex normal Y // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex normal Z // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte - Separator. It's always 00
               2 Bytes [Float] - Position U
               2 Bytes [Float] - Position V // To get a proper value you have to subtract this value from 1. Example: 1.0f - posV(0.8f) = 0.2f - is a correct V position.
               Byte [Unsigned Byte] - Vertex tangent X // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex tangent Y // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex tangent Z // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte - Separator. It's always 00
               Byte [Unsigned Byte] - Vertex binormal X // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex binormal Y // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte [Unsigned Byte] - Vertex binormal Z // To get a proper value use the formula: (2 * vertex normal / 255) - 1
               Byte - Separator. It's always 00

          for (Int32 y = 0; y < (IndicesCount / 3); ++y)
               2 Bytes [UInt16] - Index 1
               2 Bytes [UInt16] - Index 2
               2 Bytes [UInt16] - Index 3
*/

extern crate half;

use self::half::f16;
use common::coding_helpers;
use std::error;
use std::io::{
    Error, ErrorKind
};

#[derive(Clone, Debug)]
pub struct RigidModel {
    pub packed_file_header: RigidModelHeader,
    pub packed_file_data: RigidModelData,
}

#[derive(Clone, Debug)]
pub struct RigidModelHeader {
    pub packed_file_header_signature: String,
    pub packed_file_header_model_type: u32,
    pub packed_file_header_lods_count: u32,
    pub packed_file_data_base_skeleton: String,
}

#[derive(Clone, Debug)]
pub struct RigidModelData {
    pub packed_file_data_lod_list: Vec<RigidModelLod>,
    pub extra_data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RigidModelLod {
    pub groups_count: u32,
    pub vertex_data_length: u32,
    pub index_data_length: u32,
    pub start_offset: u32, // From file's origin to each of lod's beginning.
    pub lod_zoom_factor: f32,
    pub mysterious_data_1: u32, // these two are only in warhammer?
    pub mysterious_data_2: u32,
}


impl RigidModel {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModel, Error> {
        match RigidModelHeader::read(packed_file_data[..140].to_vec()) {
            Ok(packed_file_header) => {
                match RigidModelData::read(packed_file_data[140..].to_vec(), packed_file_header.packed_file_header_lods_count) {
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
    pub fn save(rigid_model_data: &mut RigidModel) -> Vec<u8> {
        let mut packed_file_data_encoded = RigidModelData::save(rigid_model_data.packed_file_data.clone());
        let mut packed_file_header_encoded = RigidModelHeader::save(rigid_model_data.packed_file_header.clone());

        let mut packed_file_encoded = vec![];
        packed_file_encoded.append(&mut packed_file_header_encoded);
        packed_file_encoded.append(&mut packed_file_data_encoded);

        packed_file_encoded
    }
}


impl RigidModelHeader {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelHeader, Error> {

        let mut packed_file_header = RigidModelHeader {
            packed_file_header_signature: String::new(),
            packed_file_header_model_type: 0,
            packed_file_header_lods_count: 0,
            packed_file_data_base_skeleton: String::new(),
        };

        match coding_helpers::decode_string_u8((&packed_file_data[0..4]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_signature = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_integer_u32((&packed_file_data[4..8]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_model_type = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_integer_u32((&packed_file_data[8..12]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_header_lods_count = data,
            Err(error) => return Err(error)
        }

        match coding_helpers::decode_string_u8((&packed_file_data[12..140]).to_vec()) {
            Ok(data) => packed_file_header.packed_file_data_base_skeleton = data,
            Err(error) => return Err(error)
        }

        Ok(packed_file_header)
    }

    pub fn save(rigid_model_header: RigidModelHeader) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];

        let mut packed_file_header_signature = coding_helpers::encode_string_u8(rigid_model_header.packed_file_header_signature);
        let mut packed_file_header_model_type = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_model_type + 1);
        let mut packed_file_header_lods_count = coding_helpers::encode_integer_u32(rigid_model_header.packed_file_header_lods_count);
        let mut packed_file_data_base_skeleton = coding_helpers::encode_string_u8(rigid_model_header.packed_file_data_base_skeleton);

        packed_file_data.append(&mut packed_file_header_signature);
        packed_file_data.append(&mut packed_file_header_model_type);
        packed_file_data.append(&mut packed_file_header_lods_count);
        packed_file_data.append(&mut packed_file_data_base_skeleton);
        packed_file_data
    }
}

impl RigidModelData {
    pub fn read(packed_file_data: Vec<u8>, packed_file_header_lods_count: u32) -> Result<RigidModelData, Error> {
        let mut index: usize = 0;
        let mut packed_file_data_lod_list: Vec<RigidModelLod> = vec![];

        for _ in 0..packed_file_header_lods_count {
            let lod = match RigidModelLod::read(packed_file_data[index..(index + 20)].to_vec()) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lod_list.push(lod);
            index += 20; // 20 in attila
        }
        let extra_data = packed_file_data[index..].to_vec();

        Ok(RigidModelData {
            packed_file_data_lod_list,
            extra_data
        })
    }

    pub fn save(mut rigid_model_data: RigidModelData) -> Vec<u8> {
        let mut packed_file_data = vec![];

        let mut patch: Vec<(u32, u32)>;
        match rigid_model_data.packed_file_data_lod_list.len() {
            1 => patch = vec![(0,2)],
            2 => patch = vec![(0,2),(4,0)],
            3 => patch = vec![(0,2),(2,0),(4,0)],
            4 => patch = vec![(0,2),(1,0),(2,0),(4,0)],
            5 => patch = vec![(0,2),(1,0),(2,0),(3,0),(4,0)],
            _ => patch = vec![(0,2)],
        }

        let mut index = 0;
        for i in rigid_model_data.packed_file_data_lod_list.iter() {
            let extra_bytes = 8 * (index + 1);
            packed_file_data.append(&mut RigidModelLod::save(i.clone(), patch[index], extra_bytes));
            index += 1;
        }
        packed_file_data.append(&mut rigid_model_data.extra_data);
        packed_file_data
    }
}

impl RigidModelLod {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLod, Error> {
        let mut header = RigidModelLod {
            groups_count: 0,
            vertex_data_length: 0,
            index_data_length: 0,
            start_offset: 0, // From file's origin to each of lod's beginning.
            lod_zoom_factor: 0.0,
            mysterious_data_1: 0,
            mysterious_data_2: 0,
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
        /*match coding_helpers::decode_integer_u32((&packed_file_data[20..24]).to_vec()) {
            Ok(data) => header.mysterious_data_1 = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[24..28]).to_vec()) {
            Ok(data) => header.mysterious_data_2 = data,
            Err(error) => return Err(error)
        }
*/
        Ok(header)
    }

    pub fn save(rigid_model_lod: RigidModelLod, patch: (u32, u32), extra_bytes: usize) -> Vec<u8> {
        let mut packed_file_data: Vec<u8> = vec![];
        println!("{:?}", patch);
        let mut groups_count = coding_helpers::encode_integer_u32(rigid_model_lod.groups_count);
        let mut vertex_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.vertex_data_length);
        let mut index_data_length = coding_helpers::encode_integer_u32(rigid_model_lod.index_data_length);
        let mut start_offset = coding_helpers::encode_integer_u32(rigid_model_lod.start_offset + extra_bytes as u32);
        let mut lod_zoom_factor = coding_helpers::encode_float_u32(rigid_model_lod.lod_zoom_factor);
        let mut mysterious_data_1 = coding_helpers::encode_integer_u32(patch.0);
        let mut mysterious_data_2 = coding_helpers::encode_integer_u32(patch.1);

        packed_file_data.append(&mut groups_count);
        packed_file_data.append(&mut vertex_data_length);
        packed_file_data.append(&mut index_data_length);
        packed_file_data.append(&mut start_offset);
        packed_file_data.append(&mut lod_zoom_factor);
        packed_file_data.append(&mut mysterious_data_1);
        packed_file_data.append(&mut mysterious_data_2);
        packed_file_data

    }
}