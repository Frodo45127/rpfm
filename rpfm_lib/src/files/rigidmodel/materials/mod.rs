//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Material system for RigidModel files.
//!
//! # Overview
//!
//! Materials define how meshes are rendered in Total War games. Each material type
//! has specific properties that control textures, shaders, vertex formats, and
//! rendering behavior. The material system supports 40+ different material types
//! for various rendering scenarios.
//!
//! # Material Types
//!
//! Materials are categorized by their rendering purpose:
//!
//! ## Standard Rendering
//! - **DefaultMaterial (68)**: Full-featured material for most meshes
//! - **Decal (71)**: Applied decals on surfaces
//! - **Tree (74)**, **TreeLeaf (75)**: Vegetation rendering
//! - **Grass (69)**: Grass
//! - **Water (83)**: Water surfaces
//! - **Unlit (84)**: No lighting calculations
//!
//! ## Skeletal Animation (Weighted)
//! - **WeightedSkin (70)**: Character skin with bone weights
//! - **WeightedCloth (58)**, **Cloth (60)**: Cloth simulation
//! - **Weighted (65)**: Generic weighted material
//!
//! ## Terrain
//! - **RsTerrain (66)**: Minimal terrain material
//! - **CustomTerrain (49)**, **GlobalTerrain (98)**: Terrain variants
//! - **WeightedTextureBlend (96)**, **TerrainBlend (86)**: Texture blending
//! - **TiledDirtmap (63)**: Tiled dirt textures
//!
//! ## Special Effects
//! - **Collision (61)**, **CollisionShape (62)**: Collision meshes (invisible)
//! - **DebugGeometry (46)**: Debug visualization
//! - **PointLight (38)**, **StaticPointLight (45)**: Light sources
//! - **Rope (93)**: Rope rendering
//!
//! ## Projected Decals
//! - **ProjectedDecal (67)**, **ProjectedDecalV2 (87)**, **ProjectedDecalV3 (95)**, **ProjectedDecalV4 (97)**
//!
//! # Material Data Structure
//!
//! Materials contain:
//! - **Vertex format**: Determines vertex data layout
//! - **Textures**: Diffuse, normal, specular, masks, etc.
//! - **Transformation matrices**: 3x4 matrices for positioning/attachment
//! - **Attachment points**: Named locations for effects/weapons
//! - **Shader parameters**: Strings, floats, integers, vectors
//! - **Cloth-specific data**: Physics simulation parameters (cloth materials only)
//!
//! # Texture Types
//!
//! Materials reference textures by type:
//! - **Diffuse (0)**: Base color texture
//! - **Normal (1)**: Normal mapping for surface detail
//! - **Specular (11)**, **GlossMap (12)**: Reflectivity
//! - **Mask (3)**: Alpha/transparency mask
//! - **AmbientOcclusion (5)**: Baked ambient lighting
//! - **Decal variants**: Special decal textures
//!
//! # Implementation Details
//!
//! Different material types have different data structures:
//! - **Default materials**: Full data (textures, params, attachment points)
//! - **RsTerrain**: Minimal (name + 5 unknown u32 fields)
//! - **WeightedTextureBlend**: Name + 6 unknown u32 fields
//! - **AlphaBlend**: Name only
//! - **Cloth**: Default data + cloth simulation sections (uk_7, uk_8, uk_9)
//!
//! See individual material variant modules for format-specific details.

use std::io::Cursor;

use getset::{Getters, MutGetters, Setters};
use nalgebra::{Matrix3x4, Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};

use super::vertices::VertexFormat;
use super::{PADDED_SIZE_32, PADDED_SIZE_256};

mod alpha_blend;
mod cloth;
mod collision_shape;
mod custom_terrain;
mod default;
mod rs_terrain;
mod rope;
mod rs_river;
mod static_point_light;
mod texture_combo;
mod uk_27;
mod uk_54;
mod weighted_cloth;
mod projected_decal;
mod projected_decal_v4;
mod weighted_texture_blend;

#[cfg(test)] mod materials_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Material type identifier determining rendering behavior and data structure.
///
/// Each material type corresponds to a specific shader and vertex format combination.
/// The numeric value is stored in the RigidModel file format as a u16.
///
/// # Categories
///
/// - **22-46**: Effects and special materials (BowWave, PointLight, DebugGeometry)
/// - **49-64**: Terrain and campaign materials
/// - **65-82**: Weighted (skeletal) materials and variants
/// - **83-100**: Water, unlit, terrain blends, projected decals
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MaterialType {
    /// Boat bow wave effect (water displacement).
    BowWave = 22,
    /// Non-rendered material (collision, occlusion, etc.).
    NonRenderable = 26,
    /// Unknown material, seen in an old test skybox.
    Uk27 = 27,
    /// Texture combo with vertex-based wind animation.
    TextureComboVertexWind = 29,
    /// Combined texture material.
    TextureCombo = 30,
    /// Waterfall decal effect.
    DecalWaterfall = 31,
    /// Simplified standard material.
    StandardSimple = 32,
    /// Campaign map tree rendering.
    CampaignTrees = 34,
    /// Dynamic point light source.
    PointLight = 38,
    /// Static baked point light.
    StaticPointLight = 45,
    /// Debug visualization geometry (wireframe, normals, etc.).
    DebugGeometry = 46,
    /// Outfield meshes of river tiles, with no material data.
    RiverOutfield = 48,
    /// Unknown material, seen in some statues. Has no vertex format.
    Uk54 = 54,
    /// Unknown material, seen in the burning variant of some buildings. Has no vertex data, only indices into the vertices of another mesh.
    Uk56 = 56,
    /// Unknown material, seen in the rain variant of some buildings. Has no vertex data, only indices into the vertices of another mesh.
    Uk57 = 57,
    /// Custom terrain material variant.
    CustomTerrain = 49,
    /// Cloth material with skeletal weights and physics.
    WeightedCloth = 58,
    /// Cloth material with physics simulation.
    Cloth = 60,
    /// Collision mesh (invisible, physics only).
    Collision = 61,
    /// Collision shape variant.
    CollisionShape = 62,
    /// Tiled dirt texture mapping.
    TiledDirtmap = 63,
    /// Ship ambient mapping (possibly incorrect identification).
    ShipAmbientmap = 64,
    /// Generic weighted material (skeletal animation).
    Weighted = 65,
    /// Minimal terrain material (Rome 2 style).
    RsTerrain = 66,
    /// Projected decal (first version).
    ProjectedDecal = 67,
    /// Default full-featured material (most common).
    #[default]
    DefaultMaterial = 68,
    /// Grass and vegetation material.
    Grass = 69,
    /// Weighted skin material (characters with bone weights).
    WeightedSkin = 70,
    /// Surface decal material.
    Decal = 71,
    /// Decal with dirt mapping.
    DecalDirtmap = 72,
    /// Dirt map material.
    Dirtmap = 73,
    /// Tree trunk/branch material.
    Tree = 74,
    /// Tree leaf material (alpha transparency).
    TreeLeaf = 75,
    /// Weighted decal (animated decals on characters).
    WeightedDecal = 77,
    /// Weighted decal with dirt mapping.
    WeightedDecalDirtmap = 78,
    /// Weighted dirt mapping.
    WeightedDirtmap = 79,
    /// Weighted skin with decal overlay.
    WeightedSkinDecal = 80,
    /// Weighted skin with decal and dirt mapping.
    WeightedSkinDecalDirtmap = 81,
    /// Weighted skin with dirt mapping.
    WeightedSkinDirtmap = 82,
    /// Water surface material.
    Water = 83,
    /// Unlit material (no lighting calculations, full bright).
    Unlit = 84,
    /// Weighted unlit material.
    WeightedUnlit = 85,
    /// Terrain texture blending material.
    TerrainBlend = 86,
    /// Projected decal version 2.
    ProjectedDecalV2 = 87,
    /// Ignored/placeholder material.
    Ignore = 88,
    /// Billboard-style tree material (always faces camera).
    TreeBillboardMaterial = 89,
    /// River rendering material (Rome 2 style).
    RsRiver = 90,
    /// Water displacement volume (physics interaction).
    WaterDisplaceVolume = 91,
    /// Rope rendering material.
    Rope = 93,
    /// Campaign map vegetation.
    CampaignVegetation = 94,
    /// Projected decal version 3.
    ProjectedDecalV3 = 95,
    /// Weighted texture blending.
    WeightedTextureBlend = 96,
    /// Projected decal version 4 (latest).
    ProjectedDecalV4 = 97,
    /// Global terrain material.
    GlobalTerrain = 98,
    /// Overlay decal material.
    DecalOverlay = 99,
    /// Alpha-blended material.
    AlphaBlend = 100,
}

/// Complete material definition with textures, transforms, and shader parameters.
///
/// Materials contain all rendering data except geometry. The exact fields present
/// depend on the material type - some types use only a subset of these fields.
///
/// # Field Organization
///
/// - **Identification**: `vertex_format`, `name`
/// - **Unknown fields**: `uk_1` through `uk_6` (only in certain material types)
/// - **Textures**: `texture_directory`, `textures` list
/// - **Transforms**: `v_pivot`, `matrix1/2/3`, matrix indices
/// - **Attachments**: `attachment_points` for effects/weapons
/// - **Shader params**: `params_string`, `params_f32`, `params_i32`, `params_vector4df32`
/// - **Cloth physics**: `uk_7`, `uk_8`, `uk_9` (cloth materials only)
///
/// # Matrix Storage
///
/// 3x4 matrices are stored in the same format as skeleton bind pose matrices.
/// The implicit fourth row `[0, 0, 0, 1]` must be added when converting to 4x4.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Material {
    /// Vertex format determining vertex data layout.
    vertex_format: VertexFormat,
    
    /// Material name (human-readable identifier).
    name: String,

    /// Second name, only in materials of types 56 and 57: the name of the mesh followed by a suffix, like `burning` or `rain`.
    name2: String,

    /// Bytes found after the end of `name` in its padded field, usually leftovers of the program that wrote the file.
    /// They're kept so the file is written back as it was.
    name_padding: Vec<u8>,

    /// Unknown field 1 (only in RsTerrain and projected decal materials).
    uk_1: u32,
    /// Unknown field 2 (only in RsTerrain and projected decal materials).
    uk_2: u32,
    /// Unknown field 3 (only in RsTerrain and projected decal materials).
    uk_3: u32,
    /// Unknown field 4 (only in RsTerrain and projected decal materials).
    uk_4: u32,
    /// Unknown field 5 (only in RsTerrain and projected decal materials).
    uk_5: u32,
    /// Unknown field 6 (only in RsTerrain and projected decal materials).
    uk_6: u32,

    /// Directory path for texture files (relative to game data directory).
    texture_directory: String,

    /// Filter settings (NOT part of file format, runtime-only).
    filters: String,

    /// Padding byte 0 (alignment/reserved).
    padding_byte0: u8,
    /// Padding byte 1 (alignment/reserved).
    padding_byte1: u8,

    /// Pivot point for transformations.
    v_pivot: Vector3<f32>,

    /// Center of the box a projected decal is projected in.
    decal_center: Vector3<f32>,

    /// Size of the box a projected decal is projected in.
    decal_size: Vector3<f32>,

    /// Center of the sphere around a collision shape.
    sphere_center: Vector3<f32>,

    /// Radius of the sphere around a collision shape.
    sphere_radius: f32,

    /// Bones of the cloth simulation of weighted cloth materials.
    cloth_bones: Vec<ClothBone>,

    /// Simulation data of rope materials.
    rope: Rope,
    
    /// Transform matrix 1 (3x4, implicit 4th row: [0, 0, 0, 1]).
    matrix1: Matrix3x4<f32>,
    /// Transform matrix 2 (3x4, implicit 4th row: [0, 0, 0, 1]).
    matrix2: Matrix3x4<f32>,
    /// Transform matrix 3 (3x4, implicit 4th row: [0, 0, 0, 1]).
    matrix3: Matrix3x4<f32>,

    /// Matrix index in hierarchy.
    i_matrix_index: i32,
    /// Parent matrix index for hierarchical transforms.
    i_parent_matrix_index: i32,

    /// Additional padding bytes (variable length).
    sz_padding: Vec<u8>,

    /// Named attachment points for weapons, effects, banners, etc.
    attachment_points: Vec<AttachmentPointEntry>,
    
    /// Texture list with type and path for each texture.
    textures: Vec<Texture>,
    
    /// String shader parameters (index, value pairs).
    params_string: Vec<(i32, String)>,
    /// Float shader parameters (index, value pairs).
    params_f32: Vec<(i32, f32)>,
    /// Integer shader parameters (index, value pairs).
    params_i32: Vec<(i32, i32)>,
    /// Vector4 shader parameters (index, value pairs).
    params_vector4df32: Vec<(i32, Vector4<f32>)>,

    /// Cloth physics data section 1 (cloth materials only, format undocumented).
    uk_7: Vec<Uk7>,
    /// Cloth physics data section 2 (cloth materials only, format undocumented).
    uk_8: Vec<Uk8>,
    /// Cloth physics data section 3 (cloth materials only, format undocumented).
    uk_9: Vec<Uk9>,
}

/// Named attachment point for visual effects, weapons, or equipment.
///
/// Attachment points define locations on a model where other objects can be attached,
/// such as weapon hardpoints, banner poles, or particle effect emitters.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AttachmentPointEntry {
    /// Attachment point name (e.g., "weapon_1", "engine_exhaust").
    name: String,
    
    /// 3x4 transform matrix positioning the attachment point (implicit 4th row: [0, 0, 0, 1]).
    matrix: Matrix3x4<f32>,
    
    /// Bone ID for skeletal attachment (0 for non-skeletal).
    bone_id: u32,
}

/// Texture reference with type and file path.
/// The simulation data of a rope material: a chain of nodes joined by links.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Rope {

    /// Unknown indices, or -1. Likely what the rope is attached to.
    attachments: Vector4<i32>,

    /// Unknown value, around 0.03.
    uk_1: f32,

    /// Unknown value, usually a power of two or 0. Likely flags.
    uk_2: u32,

    /// Nodes of the rope, from one end to the other.
    nodes: Vec<RopeNode>,

    /// Links between consecutive nodes. There's always one fewer than nodes.
    links: Vec<RopeLink>,
}

/// A node of a rope.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RopeNode {

    /// Position of the node.
    position: Vector3<f32>,

    /// Weight of the node in the simulation: 0 for the pinned ends, 1 for the rest.
    weight: f32,

    /// Unknown value, decreasing along the rope.
    uk_1: f32,
}

/// A link between two nodes of a rope.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RopeLink {

    /// Index of the first node.
    node_a: u32,

    /// Index of the second node.
    node_b: u32,

    /// Length of the link.
    length: f32,
}

/// A bone of the cloth simulation of a weighted cloth material, attached to a vertex of its mesh.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ClothBone {

    /// Name of the bone.
    name: String,

    /// Bytes found after the end of `name` in its padded field, usually leftovers of the program that wrote the file.
    /// They're kept so the file is written back as it was.
    name_padding: Vec<u8>,

    /// Unknown values. (100, 200, 400) in every Attila file.
    uk_1: Vector3<f32>,

    /// Unknown value. 12 in every Attila file.
    uk_2: u32,

    /// Unknown value. 64 in every Attila file.
    uk_3: u32,

    /// Unknown value, the same for all the bones of a mesh.
    uk_4: u32,

    /// Unknown value. 24 or 25.
    uk_5: u32,

    /// Unknown value. 24 or 25.
    uk_6: u32,

    /// Unknown value. 0 or 1.
    uk_7: f32,

    /// Transform of the bone.
    matrix: Matrix3x4<f32>,

    /// Index of the parent bone, or -1.
    parent_index: i32,

    /// Index of the vertex of the mesh the bone is attached to.
    vertex_index: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Texture {
    /// Texture type/slot (diffuse, normal, specular, etc.).
    tex_type: TextureType,
    
    /// Relative path to texture file (from `texture_directory`).
    path: String,
}

/// Texture type identifier for shader texture slots.
///
/// Determines which shader texture slot this texture is bound to during rendering.
/// The numeric value is stored as an i32 in the file format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(i32)]
pub enum TextureType {
    /// Base color/albedo texture.
    #[default]
    Diffuse = 0,
    /// Normal map for surface detail (tangent space).
    Normal = 1,
    /// Alpha/transparency mask.
    Mask = 3,
    /// Material map of older games, like Rome II and Attila.
    LegacyMaterialMap = 4,
    /// Baked ambient occlusion.
    AmbientOcclusion = 5,
    /// Tiling dirt texture (UV set 2).
    TilingDirtUV2 = 7,
    /// Dirt alpha mask.
    DirtAlphaMask = 8,
    /// Skin mask texture.
    SkinMask = 10,
    /// Specular reflectivity map.
    Specular = 11,
    /// Gloss/smoothness map (specular roughness).
    GlossMap = 12,
    /// Decal dirt map.
    DecalDirtmap = 13,
    /// Decal dirt mask.
    DecalDirtmask = 14,
    /// Decal alpha mask.
    DecalMask = 15,
    /// Damaged diffuse texture variant.
    DiffuseDamage = 17,
    /// PBR base color.
    BaseColor = 27,
    /// PBR material properties (metallic/roughness/AO packed).
    MaterialMap = 29,
}

/// Cloth physics data structure 1 (format undocumented).
///
/// This structure appears only in cloth materials and likely contains cloth simulation
/// parameters. The exact meaning of the fields is not yet reverse-engineered.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk7 {
    /// Unknown integer field 1 (possibly constraint/spring index).
    uk1: i32,
    /// Unknown integer field 2 (possibly vertex/node index).
    uk2: i32,
    /// Unknown float field (possibly stiffness/damping coefficient).
    uk3: f32,
}

/// Cloth physics data structure 2 (format undocumented).
///
/// This structure appears only in cloth materials and likely contains cloth simulation
/// parameters. The exact meaning of the field is not yet reverse-engineered.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk8 {
    /// Unknown integer field (possibly fixed vertex/anchor point index).
    uk1: i32,
}

/// Cloth physics data structure 3 (format undocumented).
///
/// This structure appears only in cloth materials and likely contains cloth simulation
/// parameters. The exact meaning of the fields is not yet reverse-engineered.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk9 {
    /// Unknown integer field 1 (possibly triangle/face index).
    uk1: i32,
    /// Unknown integer field 2 (possibly collision group).
    uk2: i32,
    /// Unknown integer field 3 (possibly property flags).
    uk3: i32,
}

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//


impl Material {

    /// Reads a material from binary data based on the material type.
    ///
    /// Different material types have different binary layouts, which may change between
    /// versions of the RigidModel format. This function dispatches to the appropriate reader.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingRigidModelUnsupportedMaterialType`] if the
    /// material type is not supported.
    pub fn read<R: ReadBytes>(data: &mut R, version: u32, mtype: MaterialType) -> Result<Self> {
        Ok(match mtype {
            MaterialType::RsTerrain => Self::read_rs_terrain(data, version)?,
            MaterialType::BowWave |
            MaterialType::DebugGeometry |
            MaterialType::RiverOutfield |
            MaterialType::RsRiver => Self::read_rs_river(data, version)?,
            MaterialType::WeightedTextureBlend => Self::read_weighted_texture_blend(data, version)?,
            MaterialType::ProjectedDecalV4 if version <= 6 => Self::read_projected_decal_v4(data, version)?,
            MaterialType::ProjectedDecal |
            MaterialType::ProjectedDecalV2 |
            MaterialType::ProjectedDecalV3 => Self::read_projected_decal(data, version, mtype)?,
            MaterialType::CustomTerrain => Self::read_custom_terrain(data, version)?,
            MaterialType::CollisionShape => Self::read_collision_shape(data, version)?,
            MaterialType::WeightedCloth if version <= 6 => Self::read_weighted_cloth(data, version)?,
            MaterialType::Rope if version <= 6 => Self::read_rope(data, version)?,
            MaterialType::TextureCombo |
            MaterialType::TextureComboVertexWind => Self::read_texture_combo(data, version)?,
            MaterialType::StaticPointLight => Self::read_static_point_light(data, version)?,
            MaterialType::Uk27 => Self::read_uk_27(data, version)?,
            MaterialType::Uk54 |
            MaterialType::Uk56 |
            MaterialType::Uk57 => Self::read_uk_54(data, version, mtype)?,
            MaterialType::AlphaBlend => Self::read_alpha_blend(data, version)?,
            MaterialType::Cloth => Self::read_cloth(data, version)?,
            MaterialType::ProjectedDecalV4 |
            MaterialType::Water |
            MaterialType::TiledDirtmap |
            MaterialType::ShipAmbientmap |
            MaterialType::TerrainBlend |
            MaterialType::Weighted |
            MaterialType::Grass |
            MaterialType::WeightedSkin |
            MaterialType::Decal |
            MaterialType::DecalDirtmap |
            MaterialType::Dirtmap |
            MaterialType::Tree |
            MaterialType::TreeLeaf |
            MaterialType::WeightedDecal |
            MaterialType::WeightedDecalDirtmap |
            MaterialType::WeightedDirtmap |
            MaterialType::WeightedSkinDirtmap |
            MaterialType::Unlit |
            MaterialType::TreeBillboardMaterial |
            MaterialType::WaterDisplaceVolume |
            MaterialType::CampaignVegetation |
            MaterialType::DefaultMaterial => Self::read_default(data, version)?,
            _ => return Err(RLibError::DecodingRigidModelUnsupportedMaterialType(mtype.into()))
        })
    }

    /// Writes a material to binary data based on the material type.
    ///
    /// Different material types have different binary layouts, which may change between
    /// versions of the RigidModel format. This function dispatches to the appropriate writer.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::DecodingRigidModelUnsupportedMaterialType`] if the
    /// material type is not supported.
    pub fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32, mtype: MaterialType) -> Result<()> {
        match mtype {
            MaterialType::RsTerrain => self.write_rs_terrain(buffer, version)?,
            MaterialType::BowWave |
            MaterialType::DebugGeometry |
            MaterialType::RiverOutfield |
            MaterialType::RsRiver => self.write_rs_river(buffer, version)?,
            MaterialType::WeightedTextureBlend => self.write_weighted_texture_blend(buffer, version)?,
            MaterialType::ProjectedDecalV4 if version <= 6 => self.write_projected_decal_v4(buffer, version)?,
            MaterialType::ProjectedDecal |
            MaterialType::ProjectedDecalV2 |
            MaterialType::ProjectedDecalV3 => self.write_projected_decal(buffer, version, mtype)?,
            MaterialType::CustomTerrain => self.write_custom_terrain(buffer, version)?,
            MaterialType::CollisionShape => self.write_collision_shape(buffer, version)?,
            MaterialType::WeightedCloth if version <= 6 => self.write_weighted_cloth(buffer, version)?,
            MaterialType::Rope if version <= 6 => self.write_rope(buffer, version)?,
            MaterialType::TextureCombo |
            MaterialType::TextureComboVertexWind => self.write_texture_combo(buffer, version)?,
            MaterialType::StaticPointLight => self.write_static_point_light(buffer, version)?,
            MaterialType::Uk27 => self.write_uk_27(buffer, version)?,
            MaterialType::Uk54 |
            MaterialType::Uk56 |
            MaterialType::Uk57 => self.write_uk_54(buffer, version, mtype)?,
            MaterialType::AlphaBlend => self.write_alpha_blend(buffer, version)?,
            MaterialType::Cloth => self.write_cloth(buffer, version)?,
            MaterialType::ProjectedDecalV4 |
            MaterialType::Water |
            MaterialType::TiledDirtmap |
            MaterialType::ShipAmbientmap |
            MaterialType::TerrainBlend |
            MaterialType::Weighted |
            MaterialType::Grass |
            MaterialType::WeightedSkin |
            MaterialType::Decal |
            MaterialType::DecalDirtmap |
            MaterialType::Dirtmap |
            MaterialType::Tree |
            MaterialType::TreeLeaf |
            MaterialType::WeightedDecal |
            MaterialType::WeightedDecalDirtmap |
            MaterialType::WeightedDirtmap |
            MaterialType::WeightedSkinDirtmap |
            MaterialType::Unlit |
            MaterialType::TreeBillboardMaterial |
            MaterialType::WaterDisplaceVolume |
            MaterialType::CampaignVegetation |
            MaterialType::DefaultMaterial => self.write_default(buffer, version)?,
            _ => return Err(RLibError::DecodingRigidModelUnsupportedMaterialType(mtype.into()))
        }

        Ok(())
    }
}

impl TryFrom<u16> for MaterialType {
    type Error = RLibError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            _ if value == Self::BowWave as u16 => Ok(Self::BowWave),
            _ if value == Self::NonRenderable as u16 => Ok(Self::NonRenderable),
            _ if value == Self::Uk27 as u16 => Ok(Self::Uk27),
            _ if value == Self::TextureComboVertexWind as u16 => Ok(Self::TextureComboVertexWind),
            _ if value == Self::TextureCombo as u16 => Ok(Self::TextureCombo),
            _ if value == Self::DecalWaterfall as u16 => Ok(Self::DecalWaterfall),
            _ if value == Self::StandardSimple as u16 => Ok(Self::StandardSimple),
            _ if value == Self::CampaignTrees as u16 => Ok(Self::CampaignTrees),
            _ if value == Self::PointLight as u16 => Ok(Self::PointLight),
            _ if value == Self::StaticPointLight as u16 => Ok(Self::StaticPointLight),
            _ if value == Self::DebugGeometry as u16 => Ok(Self::DebugGeometry),
            _ if value == Self::RiverOutfield as u16 => Ok(Self::RiverOutfield),
            _ if value == Self::Uk54 as u16 => Ok(Self::Uk54),
            _ if value == Self::Uk56 as u16 => Ok(Self::Uk56),
            _ if value == Self::Uk57 as u16 => Ok(Self::Uk57),
            _ if value == Self::CustomTerrain as u16 => Ok(Self::CustomTerrain),
            _ if value == Self::WeightedCloth as u16 => Ok(Self::WeightedCloth),
            _ if value == Self::Cloth as u16 => Ok(Self::Cloth),
            _ if value == Self::Collision as u16 => Ok(Self::Collision),
            _ if value == Self::CollisionShape as u16 => Ok(Self::CollisionShape),
            _ if value == Self::TiledDirtmap as u16 => Ok(Self::TiledDirtmap),
            _ if value == Self::ShipAmbientmap as u16 => Ok(Self::ShipAmbientmap),
            _ if value == Self::Weighted as u16 => Ok(Self::Weighted),
            _ if value == Self::RsTerrain as u16 => Ok(Self::RsTerrain),
            _ if value == Self::ProjectedDecal as u16 => Ok(Self::ProjectedDecal),
            _ if value == Self::DefaultMaterial as u16 => Ok(Self::DefaultMaterial),
            _ if value == Self::Grass as u16 => Ok(Self::Grass),
            _ if value == Self::WeightedSkin as u16 => Ok(Self::WeightedSkin),
            _ if value == Self::Decal as u16 => Ok(Self::Decal),
            _ if value == Self::DecalDirtmap as u16 => Ok(Self::DecalDirtmap),
            _ if value == Self::Dirtmap as u16 => Ok(Self::Dirtmap),
            _ if value == Self::Tree as u16 => Ok(Self::Tree),
            _ if value == Self::TreeLeaf as u16 => Ok(Self::TreeLeaf),
            _ if value == Self::WeightedDecal as u16 => Ok(Self::WeightedDecal),
            _ if value == Self::WeightedDecalDirtmap as u16 => Ok(Self::WeightedDecalDirtmap),
            _ if value == Self::WeightedDirtmap as u16 => Ok(Self::WeightedDirtmap),
            _ if value == Self::WeightedSkinDecal as u16 => Ok(Self::WeightedSkinDecal),
            _ if value == Self::WeightedSkinDecalDirtmap as u16 => Ok(Self::WeightedSkinDecalDirtmap),
            _ if value == Self::WeightedSkinDirtmap as u16 => Ok(Self::WeightedSkinDirtmap),
            _ if value == Self::Water as u16 => Ok(Self::Water),
            _ if value == Self::Unlit as u16 => Ok(Self::Unlit),
            _ if value == Self::WeightedUnlit as u16 => Ok(Self::WeightedUnlit),
            _ if value == Self::TerrainBlend as u16 => Ok(Self::TerrainBlend),
            _ if value == Self::ProjectedDecalV2 as u16 => Ok(Self::ProjectedDecalV2),
            _ if value == Self::Ignore as u16 => Ok(Self::Ignore),
            _ if value == Self::TreeBillboardMaterial as u16 => Ok(Self::TreeBillboardMaterial),
            _ if value == Self::RsRiver as u16 => Ok(Self::RsRiver),
            _ if value == Self::WaterDisplaceVolume as u16 => Ok(Self::WaterDisplaceVolume),
            _ if value == Self::Rope as u16 => Ok(Self::Rope),
            _ if value == Self::CampaignVegetation as u16 => Ok(Self::CampaignVegetation),
            _ if value == Self::ProjectedDecalV3 as u16 => Ok(Self::ProjectedDecalV3),
            _ if value == Self::WeightedTextureBlend as u16 => Ok(Self::WeightedTextureBlend),
            _ if value == Self::ProjectedDecalV4 as u16 => Ok(Self::ProjectedDecalV4),
            _ if value == Self::GlobalTerrain as u16 => Ok(Self::GlobalTerrain),
            _ if value == Self::DecalOverlay as u16 => Ok(Self::DecalOverlay),
            _ if value == Self::AlphaBlend as u16 => Ok(Self::AlphaBlend),
            _ => Err(RLibError::DecodingRigidModelUnsupportedMaterialType(value))
        }
    }
}

impl From<MaterialType> for u16 {
    fn from(value: MaterialType) -> u16 {
        value as u16
    }
}

impl TryFrom<i32> for TextureType {
    type Error = RLibError;
    fn try_from(value: i32) -> Result<Self> {
        match value {
            _ if value == Self::Diffuse as i32 => Ok(Self::Diffuse),
            _ if value == Self::Normal as i32 => Ok(Self::Normal),
            _ if value == Self::Mask as i32 => Ok(Self::Mask),
            _ if value == Self::LegacyMaterialMap as i32 => Ok(Self::LegacyMaterialMap),
            _ if value == Self::AmbientOcclusion as i32 => Ok(Self::AmbientOcclusion),
            _ if value == Self::TilingDirtUV2 as i32 => Ok(Self::TilingDirtUV2),
            _ if value == Self::DirtAlphaMask as i32 => Ok(Self::DirtAlphaMask),
            _ if value == Self::SkinMask as i32 => Ok(Self::SkinMask),
            _ if value == Self::Specular as i32 => Ok(Self::Specular),
            _ if value == Self::GlossMap as i32 => Ok(Self::GlossMap),
            _ if value == Self::DecalDirtmap as i32 => Ok(Self::DecalDirtmap),
            _ if value == Self::DecalDirtmask as i32 => Ok(Self::DecalDirtmask),
            _ if value == Self::DecalMask as i32 => Ok(Self::DecalMask),
            _ if value == Self::DiffuseDamage as i32 => Ok(Self::DiffuseDamage),
            _ if value == Self::BaseColor as i32 => Ok(Self::BaseColor),
            _ if value == Self::MaterialMap as i32 => Ok(Self::MaterialMap),
            _ => Err(RLibError::DecodingRigidModelUnknownTextureType(value))
        }
    }
}

impl TryFrom<TextureType> for i32 {
    type Error = RLibError;
    fn try_from(value: TextureType) -> Result<i32> {
        Ok(value as i32)
    }
}

/// Reads a 0-padded string of `size` bytes, also returning the bytes after its NUL up to the last non-zero one.
///
/// The string is UTF-8, or UTF-16 if `utf16` is set.
fn read_string_0padded_with_tail<R: ReadBytes>(data: &mut R, size: usize, utf16: bool) -> Result<(String, Vec<u8>)> {
    let bytes = data.read_slice(size, false)?;
    let char_size = if utf16 { 2 } else { 1 };
    let len = bytes.chunks_exact(char_size).position(|char| char.iter().all(|x| *x == 0)).map_or(size, |chars| chars * char_size);
    let tail_end = bytes.iter().rposition(|x| *x != 0).map_or(0, |last| last + 1);
    let tail = if tail_end > len + char_size { bytes[len + char_size..tail_end].to_vec() } else { vec![] };

    let string = if utf16 { Cursor::new(&bytes[..len]).read_string_u16(len)? } else { String::from_utf8(bytes[..len].to_vec())? };
    Ok((string, tail))
}

/// Writes a 0-padded string of `size` bytes, with `tail` right after its NUL.
///
/// The string is UTF-8, or UTF-16 if `utf16` is set.
fn write_string_0padded_with_tail<W: WriteBytes>(data: &mut W, string: &str, tail: &[u8], size: usize, utf16: bool) -> Result<()> {
    let char_size = if utf16 { 2 } else { 1 };
    let string_size = if utf16 { string.encode_utf16().count() * 2 } else { string.len() };
    let used = string_size + if tail.is_empty() { 0 } else { char_size + tail.len() };
    if used > size {
        return Err(RLibError::EncodingPaddedStringError("0-Padded String".to_owned(), string.to_owned(), used, size));
    }

    if utf16 {
        data.write_string_u16(string)?;
    } else {
        data.write_string_u8(string)?;
    }

    if !tail.is_empty() {
        data.write_all(&vec![0; char_size])?;
        data.write_all(tail)?;
    }

    data.write_all(&vec![0; size - used]).map_err(From::from)
}

/// Reads a 0-padded string of `size` characters: one byte each, or two in version 5 RigidModels, which use UTF-16.
fn read_padded_string<R: ReadBytes>(data: &mut R, version: u32, size: usize) -> Result<String> {
    if version <= 5 {
        data.read_string_u16_0padded(size * 2)
    } else {
        data.read_string_u8_0padded(size)
    }
}

/// Writes a 0-padded string of `size` characters: one byte each, or two in version 5 RigidModels, which use UTF-16.
fn write_padded_string<W: WriteBytes>(data: &mut W, version: u32, string: &str, size: usize) -> Result<()> {
    if version <= 5 {
        data.write_string_u16_0padded(string, size * 2, true)
    } else {
        data.write_string_u8_0padded(string, size, true)
    }
}
