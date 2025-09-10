//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters, Setters};
use nalgebra::{Vector2, Vector4};
use serde::{Deserialize, Serialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};

use super::materials::MaterialType;

// Vertex types. TODO: Maybe integrate them in the relevant enums.
const VERTEX_FORMAT_STATIC: u16 = 0;
const VERTEX_FORMAT_COLLISION: u16 = 1;
const VERTEX_FORMAT_WEIGHTED: u16 = 3;
const VERTEX_FORMAT_CINEMATIC: u16 = 4;
const VERTEX_FORMAT_GRASS: u16 = 5;
const VERTEX_FORMAT_UK_8: u16 = 8;          // Seen in glb_water_planes
const VERTEX_FORMAT_CLOTH_SIM: u16 = 25;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum VertexFormat {
    #[default] Static,
    Collision,
    Weighted,
    Cinematic,
    Grass,
    Uk8,
    ClothSim
}

/// Common vertex type. Not all vertex formats use all values of this.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Vertex {

    // Position of the vertex.
    position: Vector4<f32>,

    // Coordinates for the texture UV mapping. Not sure why there are two, they're copied from the rendering widget.
    texture_coordinate: Vector2<f32>,
    texture_coordinate2: Vector2<f32>,

    // Vertex normal, used to determine lighting-related stuff.
    normal: Vector4<f32>,

    // Vertex tangent, used for... deflection of light? According to stackoverflow.
    tangent: Vector4<f32>,
    bitangent: Vector4<f32>,

    // Colour of the mesh, it seems. Unused due to texturing?
    color: Vector4<f32>,
    bone_indices: Vector4<u8>,
    weights: Vector4<f32>,

    uk_1: Vector4<u8>,
}

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl TryFrom<u16> for VertexFormat {
    type Error = RLibError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            VERTEX_FORMAT_STATIC => Ok(Self::Static),
            VERTEX_FORMAT_COLLISION => Ok(Self::Collision),
            VERTEX_FORMAT_WEIGHTED => Ok(Self::Weighted),
            VERTEX_FORMAT_CINEMATIC => Ok(Self::Cinematic),
            VERTEX_FORMAT_GRASS => Ok(Self::Grass),
            VERTEX_FORMAT_UK_8 => Ok(Self::Uk8),
            VERTEX_FORMAT_CLOTH_SIM => Ok(Self::ClothSim),
            _ => Err(RLibError::DecodingRigidModelUnknownVertexFormat(value))
        }
    }
}

impl From<VertexFormat> for u16 {
    fn from(value: VertexFormat) -> u16 {
        match value {
            VertexFormat::Static => VERTEX_FORMAT_STATIC,
            VertexFormat::Collision => VERTEX_FORMAT_COLLISION,
            VertexFormat::Weighted => VERTEX_FORMAT_WEIGHTED,
            VertexFormat::Cinematic => VERTEX_FORMAT_CINEMATIC,
            VertexFormat::Grass => VERTEX_FORMAT_GRASS,
            VertexFormat::Uk8 => VERTEX_FORMAT_UK_8,
            VertexFormat::ClothSim => VERTEX_FORMAT_CLOTH_SIM,
        }
    }
}

impl Vertex {
    pub fn read<R: ReadBytes>(data: &mut R, vtype: VertexFormat, mtype: MaterialType) -> Result<Self> {
        let mut v = Self::default();
        match vtype {
            VertexFormat::Static => match mtype {
                MaterialType::DefaultMaterial => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.uk_1 = data.read_vector_4_u8()?;
                },

                MaterialType::RsTerrain => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                },

                MaterialType::WeightedTextureBlend => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                },

                MaterialType::ProjectedDecalV4 => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                },

                MaterialType::RsRiver => {
                    v.position = data.read_vector_4_f32()?;
                    v.normal = data.read_vector_4_f32()?;
                },

                MaterialType::TerrainBlend => {
                    v.position = data.read_vector_4_f32()?;
                    v.normal = data.read_vector_4_f32()?;
                },

                MaterialType::TiledDirtmap => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;

                    v.uk_1 = data.read_vector_4_u8()?;
                },

                MaterialType::ShipAmbientmap => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;

                    v.uk_1 = data.read_vector_4_u8()?;
                },

                MaterialType::AlphaBlend => {
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                }
                _ => return Err(RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial(vtype.into(), mtype.into()))
            },

            // Possibly not correctly calculated.
            VertexFormat::Weighted => {
                v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;

                let bone_indices = data.read_vector_2_u8()?;
                v.bone_indices = Vector4::new(bone_indices.x, bone_indices.y, 0, 0);

                let weights = data.read_vector_2_f32_pct_from_vector_2_u8()?;
                v.weights = Vector4::new(weights.x, weights.y, 0.0, 0.0);

                v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.color = data.read_vector_4_f32_normal_from_vector_4_u8()?;
            }
            VertexFormat::Cinematic => {
                v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;

                // w is not used in this one.
                v.bone_indices = data.read_vector_4_u8()?;
                v.weights = data.read_vector_4_f32_pct_from_vector_4_u8()?;
                v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                v.color = data.read_vector_4_f32_normal_from_vector_4_u8()?;
            }
            VertexFormat::Uk8 => {
                v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
            }
            _ => return Err(RLibError::DecodingRigidModelUnknownVertexFormat(vtype.into()))
        }

        Ok(v)
    }

    pub fn write<W: WriteBytes>(&self, data: &mut W, vtype: VertexFormat, mtype: MaterialType) -> Result<()> {
        match vtype {
            VertexFormat::Static => match mtype {
                MaterialType::DefaultMaterial => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;

                    // Color? Causes difference when processing if treated as color.
                    data.write_vector_4_u8(self.uk_1)?;
                }

                MaterialType::RsTerrain => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                    data.write_vector_4_f32_normal_as_vector_4_f16(*self.normal())?;
                }

                MaterialType::WeightedTextureBlend => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                    data.write_vector_4_f32_normal_as_vector_4_f16(*self.normal())?;
                }

                MaterialType::ProjectedDecalV4 => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                }

                MaterialType::RsRiver => {
                    data.write_vector_4_f32(*self.position())?;
                    data.write_vector_4_f32(*self.normal())?;
                }

                MaterialType::TerrainBlend => {
                    data.write_vector_4_f32(*self.position())?;
                    data.write_vector_4_f32(*self.normal())?;
                }

                MaterialType::TiledDirtmap => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                    data.write_vector_4_u8(self.uk_1)?;
                }

                MaterialType::ShipAmbientmap => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                    data.write_vector_4_u8(self.uk_1)?;
                }

                MaterialType::AlphaBlend => {
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                }

                _ => return Err(RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial(vtype.into(), mtype.into()))
            },
            VertexFormat::Weighted => {
                data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                data.write_vector_2_u8(Vector2::new(self.bone_indices.x, self.bone_indices.y))?;
                data.write_vector_2_f32_pct_as_vector_2_u8(Vector2::new(self.weights.x, self.weights.y))?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.color)?;

            }
            VertexFormat::Cinematic => {
                data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                data.write_vector_4_u8(self.bone_indices)?;
                data.write_vector_4_f32_pct_as_vector_4_u8(self.weights)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.color)?;
            }
            VertexFormat::Uk8 => {
                data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
            }
            _ => return Err(RLibError::DecodingRigidModelUnknownVertexFormat(vtype.into()))
        }

        Ok(())
    }
}

/// Util to swap the x and z coordinates of a vector.
fn swap_xz(input: &Vector4<f32>) -> Vector4<f32> {
    let mut i = *input;
    i.swap_rows(0, 2);
    i
}
