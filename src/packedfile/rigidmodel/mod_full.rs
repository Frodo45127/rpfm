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
    pub lod_groups_list: Vec<RigidModelLodGroup>,
}

#[derive(Clone, Debug)]
pub struct RigidModelLodGroup {
    pub rigid_material_id: u32, // rigid_material ID
    pub lod_length: u32, //LODBytes count (offset)
    pub lod_length_without_vertex_and_index_length: u32, // Like the one before, but without the vertex and index lengths
    pub vertex_count: u32, //VerticesCount
    pub lod_length_without_index_length: u32,
    pub index_count: u32,
    pub group_min_x: f32,
    pub group_min_y: f32,
    pub group_min_z: f32,
    pub group_max_x: f32,
    pub group_max_y: f32,
    pub group_max_z: f32,

    pub shader_name: String, // 32 bytes
    pub mysterious_id: u16, // 2 bytes, always 3 in attila.
    pub group_name: String, // 32 bytes
    pub textures_directory: String, // 256 bytes
    pub mysterious_data_1: Vec<u8>, // 422 bytes... we don't have any idea what's this for.

    pub supplementary_bones_count: u32,
    pub textures_count: u32,
    pub mysterious_data_2: Vec<u8>, // 140 bytes... no idea.
    pub supplementary_bones_list: Vec<RigidModelLodGroupSupplementaryBones>,
    pub textures_list: Vec<RigidModelLodGroupTexture>,
    // 00 00 00 00 as separator
    pub alpha_mode: u32, // 00 00 00 00 - Alpha mode 0 (alpha channel off), 00 00 00 01 - alpha mode 1 (alpha channel on), 00 00 00 02 - alpha mode 2 (alpha channel on). Sometimes it's FF FF FF FF (no idea about this one as of yet).

    pub vertex_list: Vec<RigidModelLodGroupVertex>,
    pub index_list: Vec<RigidModelLodGroupIndex>,
    pub extra_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct RigidModelLodGroupSupplementaryBones {
    pub name: String, // 32 bytes
    pub extra_mysterious_stuff: Vec<u8>, // 48 bytes
    pub id: u32,
}

#[derive(Clone, Debug)]
pub struct RigidModelLodGroupTexture {
    pub texture_type: u32, // 0 Diffuse, 1 Normal, 11 Specular, 12 Gloss, Mask 3 or 10
    pub path: String, // (TexturesDirectory + FileName)
}

#[derive(Clone, Debug)]
pub struct RigidModelLodGroupVertex {
    pub pos_x: u16, //f16
    pub pos_y: u16, //f16
    pub pos_z: u16, //f16
    // 00 00 as separator
    pub first_bone_id: u8,
    pub second_bone_id: u8,
    pub vertex_weight: u8,
    // 00 as separator
    pub vertex_normal_x: u8,
    pub vertex_normal_y: u8,
    pub vertex_normal_z: u8,
    // 00 as separator
    pub pos_u: u16, //f16
    pub pos_v: u16, //f16
    pub vertex_tangent_x: u8,
    pub vertex_tangent_y: u8,
    pub vertex_tangent_z: u8,
    // 00 as separator
    pub vertex_binormal_x: u8,
    pub vertex_binormal_y: u8,
    pub vertex_binormal_z: u8,
    // 00 as separator
}

#[derive(Clone, Debug)]
pub struct RigidModelLodGroupIndex {
    pub index_1: u16,
    pub index_2: u16,
    pub index_3: u16,
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
}

impl RigidModelData {
    pub fn read(packed_file_data: Vec<u8>, packed_file_header_lods_count: u32) -> Result<RigidModelData, Error> {
        let mut index: usize = 0;
        let mut packed_file_data_lod_list_temp: Vec<RigidModelLod> = vec![];
        let mut packed_file_data_lod_list: Vec<RigidModelLod> = vec![];

        for _ in 0..packed_file_header_lods_count {
            let lod = match RigidModelLod::read_header(packed_file_data[index..(index + 28)].to_vec()) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lod_list_temp.push(lod);
            index += 28; // 20 in attila
        }

        for i in packed_file_data_lod_list_temp.iter() {
            let lod = match RigidModelLod::read_data(packed_file_data[..].to_vec(), i.clone(), i.start_offset as usize - 140) {
                Ok(data) => data,
                Err(error) => return Err(error)
            };
            packed_file_data_lod_list.push(lod);
        }

        Ok(RigidModelData {
            packed_file_data_lod_list
        })
    }
}

impl RigidModelLod {
    pub fn read_header(packed_file_data: Vec<u8>) -> Result<RigidModelLod, Error> {

        let mut header = RigidModelLod {
            groups_count: 0,
            vertex_data_length: 0,
            index_data_length: 0,
            start_offset: 0, // From file's origin to each of lod's beginning.
            lod_zoom_factor: 0.0,
            mysterious_data_1: 0,
            mysterious_data_2: 0,
            lod_groups_list: vec![],
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
        match coding_helpers::decode_integer_u32((&packed_file_data[20..24]).to_vec()) {
            Ok(data) => header.mysterious_data_1 = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u32((&packed_file_data[24..28]).to_vec()) {
            Ok(data) => header.mysterious_data_2 = data,
            Err(error) => return Err(error)
        }

        Ok(header)
    }

    pub fn read_data(packed_file_data: Vec<u8>, mut rigid_model_lod: RigidModelLod, mut index: usize) -> Result<RigidModelLod, Error> {
        for _ in 0..rigid_model_lod.groups_count {
            let index_original = index.clone();
            let mut lod_group = RigidModelLodGroup {
                rigid_material_id: 0, // rigid_material ID
                lod_length: 0, //LODBytes count (offset)
                lod_length_without_vertex_and_index_length: 0, // Like the one before, but without the vertex and index lengths
                vertex_count: 0, //VerticesCount
                lod_length_without_index_length: 0,
                index_count: 0,
                group_min_x: 0.0,
                group_min_y: 0.0,
                group_min_z: 0.0,
                group_max_x: 0.0,
                group_max_y: 0.0,
                group_max_z: 0.0,

                shader_name: String::new(), // 32 bytes
                mysterious_id: 0, // 2 bytes, always 3 in attila.
                group_name: String::new(), // 32 bytes
                textures_directory: String::new(), // 256 bytes
                mysterious_data_1: vec![], // 422 bytes... we don't have any idea what's this for.

                supplementary_bones_count: 0,
                textures_count: 0,
                mysterious_data_2: vec![], // 140 bytes... no idea.
                supplementary_bones_list: vec![],
                textures_list: vec![],
                // 00 00 00 00 as separator
                alpha_mode: 0, // 00 00 00 00 - Alpha mode 0 (alpha channel off), 00 00 00 01 - alpha mode 1 (alpha channel on), 00 00 00 02 - alpha mode 2 (alpha channel on). Sometimes it's FF FF FF FF (no idea about this one as of yet).

                vertex_list: vec![],
                index_list: vec![],
                extra_bytes: vec![],
            };

            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.rigid_material_id = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.lod_length = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.lod_length_without_vertex_and_index_length = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.vertex_count = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.lod_length_without_index_length = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.index_count = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_min_x = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_min_y = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_min_z = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_max_x = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_max_y = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_float_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.group_max_z = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_string_u8((&packed_file_data[index..(index + 32)]).to_vec()) {
                Ok(data) => lod_group.shader_name = data,
                Err(error) => return Err(error)
            }

            index += 32;
            match coding_helpers::decode_integer_u16((&packed_file_data[index..(index + 2)]).to_vec()) {
                Ok(data) => lod_group.mysterious_id = data,
                Err(error) => return Err(error)
            }
            index += 2;

            match coding_helpers::decode_string_u8((&packed_file_data[index..(index + 32)]).to_vec()) {
                Ok(data) => lod_group.group_name = data,
                Err(error) => return Err(error)
            }

            index += 32;
            match coding_helpers::decode_string_u8((&packed_file_data[index..(index + 256)]).to_vec()) {
                Ok(data) => lod_group.textures_directory = data,
                Err(error) => return Err(error)
            }

            index += 256;
            lod_group.mysterious_data_1 = packed_file_data[index..(index + 422)].to_vec();
            index += 422;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.supplementary_bones_count = data,
                Err(error) => return Err(error)
            }
            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.textures_count = data,
                Err(error) => return Err(error)
            }
            index += 4;
            lod_group.mysterious_data_2 = packed_file_data[800..940].to_vec();
            index += 140;

            for i in 0..lod_group.supplementary_bones_count {
                match RigidModelLodGroupSupplementaryBones::read(packed_file_data[index..index + 74].to_vec()) {
                    Ok(data) => lod_group.supplementary_bones_list.push(data),
                    Err(error) => return Err(error)
                }
                index += 74;
            }
            for i in 0..lod_group.textures_count {
                match RigidModelLodGroupTexture::read(packed_file_data[index..index + 260].to_vec()) {
                    Ok(data) => lod_group.textures_list.push(data),
                    Err(error) => return Err(error)
                }
                index += 260;
            }

            index += 4;
            match coding_helpers::decode_integer_u32((&packed_file_data[index..(index + 4)]).to_vec()) {
                Ok(data) => lod_group.alpha_mode = data,
                Err(error) => return Err(error)
            }
            index += 4;
            for i in 0..lod_group.vertex_count {
                match RigidModelLodGroupVertex::read(packed_file_data[index..index + 28].to_vec()) {
                    Ok(data) => lod_group.vertex_list.push(data),
                    Err(error) => return Err(error)
                }
                index += 28;
            }

            for i in 0..(lod_group.index_count / 3) {
                match RigidModelLodGroupIndex::read(packed_file_data[index..index + 6].to_vec()) {
                    Ok(data) => lod_group.index_list.push(data),
                    Err(error) => return Err(error)
                }
                index += 6;
            }
            let mut extra_bytes = packed_file_data[(index - index_original)..(index_original + lod_group.lod_length as usize)].to_vec();
            lod_group.extra_bytes.append(&mut extra_bytes.to_vec());
            println!("{}", lod_group.extra_bytes.len());
            rigid_model_lod.lod_groups_list.push(lod_group);
        }
        Ok(rigid_model_lod)

    }
}

impl RigidModelLodGroupSupplementaryBones {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodGroupSupplementaryBones, Error> {
        let mut supplementary_bones = RigidModelLodGroupSupplementaryBones {
            name: String::new(), // 32 bytes
            extra_mysterious_stuff: vec![], // 48 bytes
            id: 0,
        };
        match coding_helpers::decode_string_u8((&packed_file_data[0..32]).to_vec()) {
            Ok(data) => supplementary_bones.name = data,
            Err(error) => return Err(error)
        }
        supplementary_bones.extra_mysterious_stuff = packed_file_data[32..70].to_vec();

        match coding_helpers::decode_integer_u32((&packed_file_data[70..74]).to_vec()) {
            Ok(data) => supplementary_bones.id = data,
            Err(error) => return Err(error)
        }
        Ok(supplementary_bones)
    }
}

impl RigidModelLodGroupTexture {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodGroupTexture, Error> {
        let mut texture = RigidModelLodGroupTexture {
            texture_type: 0, // 4 bytes
            path: String::new(), // 256 bytes
        };
        match coding_helpers::decode_integer_u32((&packed_file_data[0..4]).to_vec()) {
            Ok(data) => texture.texture_type = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_string_u8((&packed_file_data[4..260]).to_vec()) {
            Ok(data) => texture.path = data,
            Err(error) => return Err(error)
        }

        Ok(texture)
    }
}

impl RigidModelLodGroupVertex {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodGroupVertex, Error> {
        let mut vertex = RigidModelLodGroupVertex {
            pos_x: 0,
            pos_y: 0,
            pos_z: 0,
            // 00 00 as separator
            first_bone_id: 0,
            second_bone_id: 0,
            vertex_weight: 0,
            // 00 as separator
            vertex_normal_x: 0,
            vertex_normal_y: 0,
            vertex_normal_z: 0,
            // 00 as separator
            pos_u: 0,
            pos_v: 0,
            vertex_tangent_x: 0,
            vertex_tangent_y: 0,
            vertex_tangent_z: 0,
            // 00 as separator
            vertex_binormal_x: 0,
            vertex_binormal_y: 0,
            vertex_binormal_z: 0,
            // 00 as separator
        };
        match coding_helpers::decode_integer_u16((&packed_file_data[0..2]).to_vec()) {
            Ok(data) => vertex.pos_x = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u16((&packed_file_data[2..4]).to_vec()) {
            Ok(data) => vertex.pos_y = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u16((&packed_file_data[4..6]).to_vec()) {
            Ok(data) => vertex.pos_z = data,
            Err(error) => return Err(error)
        }

        vertex.first_bone_id = packed_file_data[9];
        vertex.second_bone_id = packed_file_data[10];
        vertex.vertex_weight = packed_file_data[11];

        vertex.vertex_normal_x = packed_file_data[13];
        vertex.vertex_normal_y = packed_file_data[14];
        vertex.vertex_normal_z = packed_file_data[15];

        match coding_helpers::decode_integer_u16((&packed_file_data[17..19]).to_vec()) {
            Ok(data) => vertex.pos_u = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u16((&packed_file_data[19..21]).to_vec()) {
            Ok(data) => vertex.pos_v = data,
            Err(error) => return Err(error)
        }

        vertex.vertex_tangent_x = packed_file_data[21];
        vertex.vertex_tangent_y = packed_file_data[22];
        vertex.vertex_tangent_z = packed_file_data[23];

        vertex.vertex_binormal_x = packed_file_data[25];
        vertex.vertex_binormal_y = packed_file_data[26];
        vertex.vertex_binormal_z = packed_file_data[27];


        Ok(vertex)
    }
}

impl RigidModelLodGroupIndex {
    pub fn read(packed_file_data: Vec<u8>) -> Result<RigidModelLodGroupIndex, Error> {
        let mut indexex = RigidModelLodGroupIndex {
            index_1: 0,
            index_2: 0,
            index_3: 0,
        };
        match coding_helpers::decode_integer_u16((&packed_file_data[0..2]).to_vec()) {
            Ok(data) => indexex.index_1 = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u16((&packed_file_data[2..4]).to_vec()) {
            Ok(data) => indexex.index_2 = data,
            Err(error) => return Err(error)
        }
        match coding_helpers::decode_integer_u16((&packed_file_data[4..6]).to_vec()) {
            Ok(data) => indexex.index_3 = data,
            Err(error) => return Err(error)
        }

        Ok(indexex)
    }
}