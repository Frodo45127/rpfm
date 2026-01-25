//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Vertex format definitions and I/O for RigidModel files.
//!
//! # Overview
//!
//! Vertices define the 3D geometry of meshes. RigidModel files support multiple vertex
//! formats optimized for different use cases, with extensive use of data compression
//! to reduce file size.
//!
//! # Vertex Formats
//!
//! | Format    | ID | Purpose                          | Bone Support |
//! |-----------|---.|----------------------------------|--------------|
//! | Static    | 0  | Standard non-animated geometry   | No           |
//! | Collision | 1  | Simplified collision meshes      | No           |
//! | Weighted  | 3  | Skeletal animation (2 bones)     | Yes (2)      |
//! | Cinematic | 4  | High-quality cutscenes (4 bones) | Yes (4)      |
//! | Grass     | 5  | Vegetation/grass rendering       | No           |
//! | Uk8       | 8  | Unknown (seen in water planes)   | Unknown      |
//! | Uk12      | 12 | Unknown (seen in coral shrubs)   | Unknown      |
//! | ClothSim  | 25 | Cloth physics simulation         | Yes          |
//!
//! # Compression Techniques
//!
//! Vertices use multiple compression methods to reduce storage:
//!
//! ## Half-Precision Floats (f16)
//! - Positions and UVs stored as 16-bit floats instead of 32-bit
//! - Reduces size by 50% with minimal precision loss
//!
//! ## Normalized Vectors (u8)
//! - Normals, tangents, bitangents stored as unsigned bytes
//! - Converted from [-1.0, 1.0] range to [0, 255] range
//! - Formula: `u8_value = (float_value + 1.0) * 127.5`
//!
//! ## Percentage Encoding (Bone Weights)
//! - Bone weights stored as u8 representing percentages
//! - Converted from [0.0, 1.0] to [0, 255]
//! - Formula: `u8_value = float_value * 255.0`
//!
//! # Format-Specific Fields
//!
//! Different vertex formats read/write different subsets of vertex data:
//!
//! - **Static**: Position, UVs, normal, tangent, bitangent
//! - **Weighted**: Static fields + bone indices + bone weights (2 bones)
//! - **Cinematic**: Static fields + bone indices + bone weights (4 bones)
//! - **Collision**: Position only (minimal data)
//! - **Grass**: Position + UVs + normal + special grass data
//! - **ClothSim**: Similar to weighted with cloth-specific data
//!
//! # Material-Dependent Variations
//!
//! The exact fields read also depend on the material type. For example:
//! - RsTerrain materials read minimal vertex data
//! - DefaultMaterial reads full vertex attributes
//! - Cloth materials include additional physics data

use getset::{Getters, MutGetters, Setters};
use nalgebra::{Vector2, Vector4};
use serde::{Deserialize, Serialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};

use super::materials::MaterialType;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Vertex format identifier determining data layout and compression.
///
/// Different formats optimize for different use cases. The numeric value is stored
/// as a u16 in the file format.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u16)]
pub enum VertexFormat {
    /// Standard static geometry without animation.
    Static = 0,
    /// Simplified collision mesh vertices (position only).
    Collision = 1,
    /// Skeletal animation vertices with bone weights (2 bones per vertex).
    Weighted = 3,
    /// High-quality cinematic vertices (4 bones per vertex).
    Cinematic = 4,
    /// Grass/vegetation vertices with special rendering data.
    Grass = 5,
    /// Unknown format (observed in glb_water_planes models).
    Uk8 = 8,
    /// Unknown format (observed in sea_coral_shrubs_02 models).
    Uk12 = 12,
    /// Cloth simulation vertices with physics data.
    ClothSim = 25,
}

impl Default for VertexFormat {
    fn default() -> Self {
        Self::Static
    }
}

/// Universal vertex structure containing all possible vertex attributes.
///
/// Not all fields are used by all vertex formats - the format and material type
/// determine which fields contain valid data. Fields are stored with various
/// compression techniques in the binary format.
///
/// # Field Usage by Format
///
/// - **Static**: position, UVs, normal, tangent, bitangent
/// - **Weighted**: Static fields + bone_indices (2) + weights (2)
/// - **Cinematic**: Static fields + bone_indices (4) + weights (4)
/// - **Collision**: position only
/// - **Grass**: position, UVs, normal, + grass-specific data
/// - **ClothSim**: Similar to Weighted with cloth data
///
/// # Compression Notes
///
/// When reading from file:
/// - Position is often stored as Vector4 of f16 (half-precision)
/// - UVs stored as f16
/// - Normals/tangents/bitangents stored as u8 normalized vectors
/// - Bone weights stored as u8 percentages
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Vertex {

    /// 3D position of the vertex (x, y, z, w).
    /// Often stored as f16 in file, expanded to f32 in memory.
    position: Vector4<f32>,

    /// Primary texture UV coordinates (u, v).
    /// Often stored as f16 in file, expanded to f32 in memory.
    texture_coordinate: Vector2<f32>,

    /// Secondary texture UV coordinates (u, v) for multi-texturing.
    /// Often stored as f16 in file, expanded to f32 in memory.
    texture_coordinate2: Vector2<f32>,

    /// Vertex normal vector for lighting calculations (x, y, z, w).
    /// Often stored as u8 normalized in file, expanded to f32 in memory.
    normal: Vector4<f32>,

    /// Tangent vector for normal mapping (x, y, z, w).
    /// Often stored as u8 normalized in file, expanded to f32 in memory.
    tangent: Vector4<f32>,

    /// Bitangent vector for normal mapping (x, y, z, w).
    /// Often stored as u8 normalized in file, expanded to f32 in memory.
    bitangent: Vector4<f32>,

    /// Vertex color (r, g, b, a). Typically unused in modern rendering (textures used instead).
    color: Vector4<f32>,

    /// Bone indices for skeletal animation (up to 4 bones).
    /// For Weighted format: only first 2 are used.
    /// For Cinematic format: all 4 are used.
    bone_indices: Vector4<u8>,

    /// Bone weights for skeletal animation (up to 4 weights, should sum to 1.0).
    /// Often stored as u8 percentages in file, expanded to f32 in memory.
    weights: Vector4<f32>,

    /// Unknown field (version 8+ only, purpose undocumented).
    uk_1: Vector4<u8>,
}

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl TryFrom<u16> for VertexFormat {
    type Error = RLibError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            _ if value == Self::Static as u16 => Ok(Self::Static),
            _ if value == Self::Collision as u16 => Ok(Self::Collision),
            _ if value == Self::Weighted as u16 => Ok(Self::Weighted),
            _ if value == Self::Cinematic as u16 => Ok(Self::Cinematic),
            _ if value == Self::Grass as u16 => Ok(Self::Grass),
            _ if value == Self::Uk8 as u16 => Ok(Self::Uk8),
            _ if value == Self::Uk12 as u16 => Ok(Self::Uk12),
            _ if value == Self::ClothSim as u16 => Ok(Self::ClothSim),
            _ => Err(RLibError::DecodingRigidModelUnknownVertexFormat(value))
        }
    }
}

impl From<VertexFormat> for u16 {
    fn from(value: VertexFormat) -> u16 {
        value as u16
    }
}

impl Vertex {

    /// Reads a vertex from binary data based on the vertex format and material type.
    ///
    /// Different combinations of vertex format and material type have different binary
    /// layouts. This function dispatches to the appropriate reading logic based on
    /// the format/material combination.
    ///
    /// # Arguments
    ///
    /// * `data` - Binary data source.
    /// * `version` - RigidModel version (affects layout for some formats).
    /// * `vtype` - Vertex format specifying the data layout.
    /// * `mtype` - Material type (affects layout for static vertices).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The vertex format is unknown ([`RLibError::DecodingRigidModelUnknownVertexFormat`])
    /// - The vertex format is unsupported for the material type
    ///   ([`RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial`])
    pub fn read<R: ReadBytes>(data: &mut R, version: u32, vtype: VertexFormat, mtype: MaterialType) -> Result<Self> {
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
                MaterialType::Decal => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;

                    v.uk_1 = data.read_vector_4_u8()?;
                }
                MaterialType::Dirtmap => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;

                    v.uk_1 = data.read_vector_4_u8()?;
                }
                MaterialType::AlphaBlend => {
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                }
                MaterialType::Cloth => {
                    v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                    v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                    v.normal = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.tangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;
                    v.bitangent = data.read_vector_4_f32_normal_from_vector_4_u8()?;

                    v.uk_1 = data.read_vector_4_u8()?;
                }
                _ => return Err(RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial(vtype.into(), mtype.into()))
            },

            VertexFormat::Collision => {
                v.position = data.read_vector_4_f32_from_vec_3_f32()?;
                v.normal = data.read_vector_4_f32_from_vec_3_f32()?;
            }

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

                if version >= 8 {
                    v.uk_1 = data.read_vector_4_u8()?;
                }
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

                if version >= 8 {
                    v.uk_1 = data.read_vector_4_u8()?;
                }
            }
            VertexFormat::Uk8 => {
                v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
            }
            VertexFormat::Uk12 => {
                v.position = data.read_vector_4_f32_normal_from_vector_4_f16()?;
                v.texture_coordinate = data.read_vector_2_f32_from_vector_2_f16()?;
                v.texture_coordinate2 = data.read_vector_2_f32_from_vector_2_f16()?;
                v.uk_1 = data.read_vector_4_u8()?;
            }
            _ => return Err(RLibError::DecodingRigidModelUnknownVertexFormat(vtype.into()))
        }

        Ok(v)
    }

    /// Writes a vertex to binary data based on the vertex format and material type.
    ///
    /// Different combinations of vertex format and material type have different binary
    /// layouts. This function dispatches to the appropriate writing logic based on
    /// the format/material combination.
    ///
    /// # Arguments
    ///
    /// * `data` - Output buffer.
    /// * `version` - RigidModel version (affects layout for some formats).
    /// * `vtype` - Vertex format specifying the data layout.
    /// * `mtype` - Material type (affects layout for static vertices).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The vertex format is unknown ([`RLibError::DecodingRigidModelUnknownVertexFormat`])
    /// - The vertex format is unsupported for the material type
    ///   ([`RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial`])
    pub fn write<W: WriteBytes>(&self, data: &mut W, version: u32, vtype: VertexFormat, mtype: MaterialType) -> Result<()> {
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
                MaterialType::Decal => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                    data.write_vector_4_u8(self.uk_1)?;
                }
                MaterialType::Dirtmap => {
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
                MaterialType::Cloth => {
                    data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                    data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                    data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;
                    data.write_vector_4_u8(self.uk_1)?;
                }
                _ => return Err(RLibError::DecodingRigidModelUnsupportedVertexFormatForMaterial(vtype.into(), mtype.into()))
            },

            VertexFormat::Collision => {
                data.write_vector_4_f32_to_vector_3_f32(self.position)?;
                data.write_vector_4_f32_to_vector_3_f32(self.normal)?;
            }

            VertexFormat::Weighted => {
                data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                data.write_vector_2_u8(Vector2::new(self.bone_indices.x, self.bone_indices.y))?;
                data.write_vector_2_f32_pct_as_vector_2_u8(Vector2::new(self.weights.x, self.weights.y))?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;

                if version >= 8 {
                    data.write_vector_4_u8(self.uk_1)?;
                }
            }
            VertexFormat::Cinematic => {
                data.write_vector_4_f32_normal_as_vector_4_f16(self.position)?;
                data.write_vector_4_u8(self.bone_indices)?;
                data.write_vector_4_f32_pct_as_vector_4_u8(self.weights)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.normal)?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.tangent)?;
                data.write_vector_4_f32_normal_as_vector_4_u8(self.bitangent)?;

                if version >= 8 {
                    data.write_vector_4_u8(self.uk_1)?;
                }
            }
            VertexFormat::Uk8 => {
                data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
            }
            VertexFormat::Uk12 => {
                data.write_vector_4_f32_normal_as_vector_4_f16(*self.position())?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate)?;
                data.write_vector_2_f32_as_vector_2_f16(self.texture_coordinate2)?;
                data.write_vector_4_u8(self.uk_1)?;
            }
            _ => return Err(RLibError::DecodingRigidModelUnknownVertexFormat(vtype.into()))
        }

        Ok(())
    }
}
/*
/// Util to swap the x and z coordinates of a vector.
fn swap_xz(input: &Vector4<f32>) -> Vector4<f32> {
    let mut i = *input;
    i.swap_rows(0, 2);
    i
}*/
