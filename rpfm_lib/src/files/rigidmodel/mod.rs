//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! RigidModel file format support for Total War games.
//!
//! # Overview
//!
//! RigidModel files (`.rigid_model_v2`) are the primary 3D model format used by Total War games
//! to store mesh geometry, materials, skeletal animation data, and visual effects. These files
//! contain everything needed to render game assets including characters, buildings, terrain,
//! vegetation, and props.
//!
//! # File Structure
//!
//! A RigidModel file contains:
//!
//! 1. **Header**: File signature (`RMV2`), version number, skeleton ID
//! 2. **LOD (Level of Detail) structures**: Multiple quality levels for distance-based rendering
//! 3. **Mesh blocks**: Individual mesh units, each with geometry and material data
//! 4. **Vertex data**: 3D positions, normals, UVs, bone weights (often compressed)
//! 5. **Material definitions**: Textures, shaders, attachment points, rendering parameters
//!
//! # Supported Versions
//!
//! | Version | Support | Notes                 |
//! |---------|---------|-----------------------|
//! | 6       | ✅ Full | Older format          |
//! | 7       | ✅ Full | Intermediate format   |
//! | 8       | ✅ Full | Current/newest format |
//!
//! # Material System
//!
//! RigidModels support 40+ material types for different rendering needs:
//! - **Standard rendering**: DefaultMaterial, Decal, Tree, Grass, Water
//! - **Skeletal animation**: WeightedSkin, WeightedCloth
//! - **Terrain**: RsTerrain, WeightedTextureBlend, TiledDirtmap
//! - **Special effects**: Collision, DebugGeometry, PointLight
//!
//! Each material type determines:
//! - Which vertex format is used (affects vertex compression and available data)
//! - What textures and parameters are stored
//! - How the material is rendered in-game
//!
//! # Vertex Formats
//!
//! Different vertex formats optimize storage for specific use cases:
//! - **Static (0)**: Standard geometry without animation
//! - **Weighted (3)**: Skeletal animation with bone indices and weights
//! - **Cinematic (4)**: High-quality vertices for cutscenes (supports 4 bones)
//! - **Grass (5)**: Vegetation-specific format
//! - **ClothSim (25)**: Cloth physics simulation vertices
//! - **Collision (1)**: Simplified collision mesh vertices
//!
//! Vertices use various compression techniques:
//! - Half-precision floats (f16) for positions and UVs
//! - Normalized u8 vectors for normals, tangents, bitangents
//! - Percentage encoding for bone weights
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::rigidmodel::RigidModel;
//! use rpfm_lib::files::{Decodeable, Encodeable};
//!
//! // Decode a RigidModel file
//! let model = RigidModel::decode(&mut reader, &None)?;
//!
//! // Access LODs and meshes
//! for lod in model.lods() {
//!     println!("LOD distance: {}", lod.visibility_distance());
//!     for mesh_block in lod.mesh_blocks() {
//!         println!("Mesh: {}", mesh_block.mesh().name());
//!         println!("Vertices: {}", mesh_block.mesh().vertices().len());
//!     }
//! }
//!
//! // Encode back to bytes
//! model.encode(&mut writer, &None)?;
//! ```
//!
//! # Credits
//!
//! Most of the reverse-engineering work for this module was done by Victimized, Phazer, and Ole.
//! Their research enabled the implementation of this format decoder/encoder.

use getset::*;
use nalgebra::Vector3;
use serde_derive::{Serialize, Deserialize};

use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::materials::{Material, MaterialType};
use self::vertices::Vertex;

/// Magic bytes identifying a RigidModel file (`RMV2`).
const SIGNATURE: &[u8; 4] = b"RMV2";

/// File extension for RigidModel files.
pub const EXTENSION: &str = ".rigid_model_v2";

// String field sizes (null-padded to fixed lengths in binary format)
const PADDED_SIZE_32: usize = 32;       // Small strings (mesh names, etc.)
const PADDED_SIZE_64: usize = 64;       // Medium strings
const PADDED_SIZE_128: usize = 128;     // Large strings (skeleton IDs, texture paths)
const PADDED_SIZE_256: usize = 256;     // Extra-large strings

/// Base header size in bytes (signature + version + skeleton_id + lod count fields).
const HEADER_LENGTH: u32 = 140;

pub mod materials;
mod versions;
pub mod vertices;

#[cfg(test)] mod test_rigidmodel;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Root structure representing a complete RigidModel file.
///
/// Contains the file version, associated skeleton, and one or more LOD levels.
/// The skeleton ID links this model to animation data for skeletal meshes.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RigidModel {
    /// File format version (6, 7, or 8).
    version: u32,

    /// Unknown field, purpose unclear.
    uk_1: u16,

    /// Skeleton identifier for skeletal animation (e.g., "humanoid01").
    /// Empty string for static models without animation.
    skeleton_id: String,

    /// Level of Detail structures, ordered from highest to lowest quality.
    /// Typically contains 1-4 LODs for distance-based rendering optimization.
    lods: Vec<Lod>,
}

/// Level of Detail structure containing meshes at a specific quality level.
///
/// LODs allow the game engine to render lower-poly versions of models at greater
/// distances, improving performance. Each LOD contains one or more mesh blocks.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Lod {
    /// Distance in game units at which this LOD becomes visible.
    /// Lower distances = higher detail. Typically: LOD0 = 0.0, LOD1 = 75.0, LOD2 = 150.0, etc.
    visibility_distance: f32,

    /// Authored LOD index (0 = highest quality, 1 = medium, 2 = low, etc.).
    authored_lod_number: u32,

    /// Quality level indicator (purpose not fully documented).
    quality_level: u32,

    /// Individual mesh blocks that make up this LOD.
    /// Each block has its own geometry and material.
    mesh_blocks: Vec<MeshBlock>,
}

/// A single mesh unit with geometry and material data.
///
/// Mesh blocks are the fundamental rendering unit. Each block contains vertices,
/// indices for triangle construction, and a material defining how it's rendered.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct MeshBlock {
    /// Mesh geometry data (vertices, indices, bounding box).
    mesh: Mesh,

    /// Material definition (textures, shaders, rendering parameters).
    material: Material,
}

/// Mesh geometry container with vertices, indices, and metadata.
///
/// Contains all geometric data needed to render a mesh: vertex positions/normals/UVs,
/// triangle indices, bounding box for culling, and shader parameters.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Mesh {
    /// Human-readable mesh name (e.g., "head_mesh", "sword_blade").
    name: String,

    /// Material type determines vertex format and material data structure.
    material_type: MaterialType,

    /// Raw shader parameter data (format not fully documented).
    shader_params: ShaderParams,

    /// Axis-aligned bounding box minimum corner (x, y, z).
    min_bb: Vector3<f32>,

    /// Axis-aligned bounding box maximum corner (x, y, z).
    max_bb: Vector3<f32>,

    /// Lighting configuration string (format not fully documented).
    lighting_constants: String,

    /// Vertex data array. Format depends on `material_type` and vertex format.
    /// See [`Vertex`] for field details.
    vertices: Vec<Vertex>,

    /// Index buffer defining triangles (groups of 3 indices into `vertices`).
    indices: Vec<u16>,
}

/// Raw shader parameter data (not fully decoded).
///
/// Contains binary shader configuration data. The structure of this data
/// is not fully reverse-engineered and is stored as raw bytes.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ShaderParams {
    /// Raw shader parameter bytes.
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for RigidModel {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(4, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingRigidModelUnsupportedSignature(signature_bytes));
        }

        let mut rigid = Self::default();
        rigid.version = data.read_u32()?;

        match rigid.version {
            8 => rigid.read_v8(data)?,
            7 => rigid.read_v7(data)?,
            6 => rigid.read_v6(data)?,
            _ => Err(RLibError::DecodingRigidModelUnsupportedVersion(rigid.version))?,
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(rigid)
    }
}

impl Encodeable for RigidModel {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u32(self.version)?;

        match self.version {
            8 => self.write_v8(buffer)?,
            7 => self.write_v7(buffer)?,
            6 => self.write_v6(buffer)?,
            _ => Err(RLibError::DecodingRigidModelUnsupportedVersion(self.version))?,
        }

        Ok(())
    }
}
