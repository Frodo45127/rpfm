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
use nalgebra::{Matrix3x4, Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};

use super::vertices::VertexFormat;
use super::{PADDED_SIZE_32, PADDED_SIZE_256};

const MATERIAL_TYPE_BOW_WAVE: u16 = 22;
const MATERIAL_TYPE_NON_RENDERABLE: u16 = 26;
const MATERIAL_TYPE_TEXTURE_COMBO_VERTEX_WIND: u16 = 29;
const MATERIAL_TYPE_TEXTURE_COMBO: u16 = 30;
const MATERIAL_TYPE_DECAL_WATERFALL: u16 = 31;
const MATERIAL_TYPE_STANDARD_SIMPLE: u16 = 32;
const MATERIAL_TYPE_CAMPAIGN_TREES: u16 = 34;
const MATERIAL_TYPE_POINT_LIGHT: u16 = 38;
const MATERIAL_TYPE_STATIC_POINT_LIGHT: u16 = 45;
const MATERIAL_TYPE_DEBUG_GEOMETRY: u16 = 46;
const MATERIAL_TYPE_CUSTOM_TERRAIN: u16 = 49;
const MATERIAL_TYPE_WEIGHTED_CLOTH: u16 = 58;
const MATERIAL_TYPE_CLOTH: u16 = 60;
const MATERIAL_TYPE_COLLISION: u16 = 61;
const MATERIAL_TYPE_COLLISION_SHAPE: u16 = 62;
const MATERIAL_TYPE_TILED_DIRTMAP: u16 = 63;
const MATERIAL_TYPE_SHIP_AMBIENTMAP: u16 = 64;      // Possibly wrong.
const MATERIAL_TYPE_WEIGHTED: u16 = 65;
const MATERIAL_TYPE_RS_TERRAIN: u16 = 66;
const MATERIAL_TYPE_PROJECTED_DECAL: u16 = 67;
const MATERIAL_TYPE_DEFAULT_MATERIAL: u16 = 68;
const MATERIAL_TYPE_GRASS: u16 = 69;
const MATERIAL_TYPE_WEIGHTED_SKIN: u16 = 70;
const MATERIAL_TYPE_DECAL: u16 = 71;
const MATERIAL_TYPE_DECAL_DIRTMAP: u16 = 72;
const MATERIAL_TYPE_DIRTMAP: u16 = 73;
const MATERIAL_TYPE_TREE: u16 = 74;
const MATERIAL_TYPE_TREE_LEAF: u16 = 75;
const MATERIAL_TYPE_WEIGHTED_DECAL: u16 = 77;
const MATERIAL_TYPE_WEIGHTED_DECAL_DIRTMAP: u16 = 78;
const MATERIAL_TYPE_WEIGHTED_DIRTMAP: u16 = 79;
const MATERIAL_TYPE_WEIGHTED_SKIN_DECAL: u16 = 80;
const MATERIAL_TYPE_WEIGHTED_SKIN_DECAL_DIRTMAP: u16 = 81;
const MATERIAL_TYPE_WEIGHTED_SKIN_DIRTMAP: u16 = 82;
const MATERIAL_TYPE_WATER: u16 = 83;
const MATERIAL_TYPE_UNLIT: u16 = 84;
const MATERIAL_TYPE_WEIGHTED_UNLIT: u16 = 85;
const MATERIAL_TYPE_TERRAIN_BLEND: u16 = 86;
const MATERIAL_TYPE_PROJECTED_DECAL_V2: u16 = 87;
const MATERIAL_TYPE_IGNORE: u16 = 88;
const MATERIAL_TYPE_TREE_BILLBOARD_MATERIAL: u16 = 89;
const MATERIAL_TYPE_RS_RIVER: u16 = 90;
const MATERIAL_TYPE_WATER_DISPLACE_VOLUME: u16 = 91;
const MATERIAL_TYPE_ROPE: u16 = 93;
const MATERIAL_TYPE_CAMPAIGN_VEGETATION: u16 = 94;
const MATERIAL_TYPE_PROJECTED_DECAL_V3: u16 = 95;
const MATERIAL_TYPE_WEIGHTED_TEXTURE_BLEND: u16 = 96;
const MATERIAL_TYPE_PROJECTED_DECAL_V4: u16 = 97;
const MATERIAL_TYPE_GLOBAL_TERRAIN: u16 = 98;
const MATERIAL_TYPE_DECAL_OVERLAY: u16 = 99;
const MATERIAL_TYPE_ALPHA_BLEND: u16 = 100;

const TEXTURE_TYPE_DIFFUSE: i32 = 0;
const TEXTURE_TYPE_NORMAL: i32 = 1;
const TEXTURE_TYPE_MASK: i32 = 3;
const TEXTURE_TYPE_AMBIENT_OCCLUSION: i32 = 5;
const TEXTURE_TYPE_TILING_DIRT_UV2: i32 = 7;
const TEXTURE_TYPE_DIRT_ALPHA_MASK: i32 = 8;
const TEXTURE_TYPE_SKIN_MASK: i32 = 10;
const TEXTURE_TYPE_SPECULAR: i32 = 11;
const TEXTURE_TYPE_GLOSS_MAP: i32 = 12;
const TEXTURE_TYPE_DECAL_DIRTMAP: i32 = 13;
const TEXTURE_TYPE_DECAL_DIRTMASK: i32 = 14;
const TEXTURE_TYPE_DECAL_MASK: i32 = 15;
const TEXTURE_TYPE_DIFFUSE_DAMAGE: i32 = 17;
const TEXTURE_TYPE_BASE_COLOR: i32 = 27;
const TEXTURE_TYPE_MATERIAL_MAP: i32 = 29;

mod alpha_blend;
mod cloth;
mod default;
mod rs_terrain;
mod rs_river;
mod projected_decal_v4;
mod weighted_texture_blend;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum MaterialType {
    BowWave,
    NonRenderable,
    TextureComboVertexWind,
    TextureCombo,
    DecalWaterfall,
    StandardSimple,
    CampaignTrees,
    PointLight,
    StaticPointLight,
    DebugGeometry,
    CustomTerrain,
    WeightedCloth,
    Cloth,
    Collision,
    CollisionShape,
    TiledDirtmap,
    ShipAmbientmap,
    Weighted,
    RsTerrain,
    ProjectedDecal,
    #[default] DefaultMaterial,
    Grass,
    WeightedSkin,
    Decal,
    DecalDirtmap,
    Dirtmap,
    Tree,
    TreeLeaf,
    WeightedDecal,
    WeightedDecalDirtmap,
    WeightedDirtmap,
    WeightedSkinDecal,
    WeightedSkinDecalDirtmap,
    WeightedSkinDirtmap,
    Water,
    Unlit,
    WeightedUnlit,
    TerrainBlend,
    ProjectedDecalV2,
    Ignore,
    TreeBillboardMaterial,
    RsRiver,
    WaterDisplaceVolume,
    Rope,
    CampaignVegetation,
    ProjectedDecalV3,
    WeightedTextureBlend,
    ProjectedDecalV4,
    GlobalTerrain,
    DecalOverlay,
    AlphaBlend,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Material {
    vertex_format: VertexFormat,
    name: String,

    // Only present in RsTerrain materials and projected decals.
    uk_1: u32,
    uk_2: u32,
    uk_3: u32,
    uk_4: u32,
    uk_5: u32,
    uk_6: u32,

    texture_directory: String,

    // !!NOT!! part of file format
    filters: String,

    padding_byte0: u8,
    padding_byte1: u8,

    // 3x4 indentity matrices. They are stored like CA stores the bind pose matrices for skeletons.
    // So the last row is implicit = 0, 0, 0, 1 (you have to put it in yourself)
    v_pivot: Vector3<f32>,
    matrix1: Matrix3x4<f32>,
    matrix2: Matrix3x4<f32>,
    matrix3: Matrix3x4<f32>,

    i_matrix_index: i32,
    i_parent_matrix_index: i32,

    sz_padding: Vec<u8>,

    attachment_points: Vec<AttachmentPointEntry>,
    textures: Vec<Texture>,
    params_string: Vec<(i32, String)>,
    params_f32: Vec<(i32, f32)>,
    params_i32: Vec<(i32, i32)>,
    params_vector4df32: Vec<(i32, Vector4<f32>)>,

    // These are cloth-specific sections of data. No idea what they represent, but they're only in cloth materials.
    uk_7: Vec<Uk7>,
    uk_8: Vec<Uk8>,
    uk_9: Vec<Uk9>,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AttachmentPointEntry {
    name: String,
    matrix: Matrix3x4<f32>,
    bone_id: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Texture {
    tex_type: TextureType,
    path: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum TextureType {
    #[default] Diffuse,
    Normal,
    Mask,
    AmbientOcclusion,
    TilingDirtUV2,
    DirtAlphaMask,
    SkinMask,
    Specular,
    GlossMap,
    DecalDirtmap,
    DecalDirtmask,
    DecalMask,
    DiffuseDamage,
    BaseColor,
    MaterialMap,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk7 {
    uk1: i32,
    uk2: i32,
    uk3: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk8 {
    uk1: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Uk9 {
    uk1: i32,
    uk2: i32,
    uk3: i32,
}

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//


impl Material {
    pub fn read<R: ReadBytes>(data: &mut R, mtype: MaterialType) -> Result<Self> {
        Ok(match mtype {
            MaterialType::RsTerrain => Self::read_rs_terrain(data)?,
            MaterialType::RsRiver => Self::read_rs_river(data)?,
            MaterialType::WeightedTextureBlend => Self::read_weighted_texture_blend(data)?,
            //MaterialType::ProjectedDecalV4 => Self::read_projected_decal_v4(data)?,
            MaterialType::AlphaBlend => Self::read_alpha_blend(data)?,
            MaterialType::Cloth => Self::read_cloth(data)?,
            MaterialType::ProjectedDecalV4 |
            MaterialType::Water |
            MaterialType::TiledDirtmap |
            MaterialType::ShipAmbientmap |
            MaterialType::TerrainBlend |
            MaterialType::Weighted |
            MaterialType::DefaultMaterial => Self::read_default(data)?,
            _ => return Err(RLibError::DecodingRigidModelUnsupportedMaterialType(mtype.into()))
        })
    }

    pub fn write<W: WriteBytes>(&self, buffer: &mut W, mtype: MaterialType) -> Result<()> {
        match mtype {
            MaterialType::RsTerrain => self.write_rs_terrain(buffer)?,
            MaterialType::RsRiver => self.write_rs_river(buffer)?,
            MaterialType::WeightedTextureBlend => self.write_weighted_texture_blend(buffer)?,
            //MaterialType::ProjectedDecalV4 => self.write_projected_decal_v4(buffer)?,
            MaterialType::AlphaBlend => self.write_alpha_blend(buffer)?,
            MaterialType::Cloth => self.write_cloth(buffer)?,
            MaterialType::ProjectedDecalV4 |
            MaterialType::Water |
            MaterialType::TiledDirtmap |
            MaterialType::ShipAmbientmap |
            MaterialType::TerrainBlend |
            MaterialType::Weighted |
            MaterialType::DefaultMaterial => self.write_default(buffer)?,
            _ => return Err(RLibError::DecodingRigidModelUnsupportedMaterialType(mtype.into()))
        }

        Ok(())
    }
}

impl TryFrom<u16> for MaterialType {
    type Error = RLibError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            MATERIAL_TYPE_BOW_WAVE => Ok(Self::BowWave),
            MATERIAL_TYPE_NON_RENDERABLE => Ok(Self::NonRenderable),
            MATERIAL_TYPE_TEXTURE_COMBO_VERTEX_WIND => Ok(Self::TextureComboVertexWind),
            MATERIAL_TYPE_TEXTURE_COMBO => Ok(Self::TextureCombo),
            MATERIAL_TYPE_DECAL_WATERFALL => Ok(Self::DecalWaterfall),
            MATERIAL_TYPE_STANDARD_SIMPLE => Ok(Self::StandardSimple),
            MATERIAL_TYPE_CAMPAIGN_TREES => Ok(Self::CampaignTrees),
            MATERIAL_TYPE_POINT_LIGHT => Ok(Self::PointLight),
            MATERIAL_TYPE_STATIC_POINT_LIGHT => Ok(Self::StaticPointLight),
            MATERIAL_TYPE_DEBUG_GEOMETRY => Ok(Self::DebugGeometry),
            MATERIAL_TYPE_CUSTOM_TERRAIN => Ok(Self::CustomTerrain),
            MATERIAL_TYPE_WEIGHTED_CLOTH => Ok(Self::WeightedCloth),
            MATERIAL_TYPE_CLOTH => Ok(Self::Cloth),
            MATERIAL_TYPE_COLLISION => Ok(Self::Collision),
            MATERIAL_TYPE_COLLISION_SHAPE => Ok(Self::CollisionShape),
            MATERIAL_TYPE_TILED_DIRTMAP => Ok(Self::TiledDirtmap),
            MATERIAL_TYPE_SHIP_AMBIENTMAP => Ok(Self::ShipAmbientmap),
            MATERIAL_TYPE_WEIGHTED => Ok(Self::Weighted),
            MATERIAL_TYPE_RS_TERRAIN => Ok(Self::RsTerrain),
            MATERIAL_TYPE_PROJECTED_DECAL => Ok(Self::ProjectedDecal),
            MATERIAL_TYPE_DEFAULT_MATERIAL => Ok(Self::DefaultMaterial),
            MATERIAL_TYPE_GRASS => Ok(Self::Grass),
            MATERIAL_TYPE_WEIGHTED_SKIN => Ok(Self::WeightedSkin),
            MATERIAL_TYPE_DECAL => Ok(Self::Decal),
            MATERIAL_TYPE_DECAL_DIRTMAP => Ok(Self::DecalDirtmap),
            MATERIAL_TYPE_DIRTMAP => Ok(Self::Dirtmap),
            MATERIAL_TYPE_TREE => Ok(Self::Tree),
            MATERIAL_TYPE_TREE_LEAF => Ok(Self::TreeLeaf),
            MATERIAL_TYPE_WEIGHTED_DECAL => Ok(Self::WeightedDecal),
            MATERIAL_TYPE_WEIGHTED_DECAL_DIRTMAP => Ok(Self::WeightedDecalDirtmap),
            MATERIAL_TYPE_WEIGHTED_DIRTMAP => Ok(Self::WeightedDirtmap),
            MATERIAL_TYPE_WEIGHTED_SKIN_DECAL => Ok(Self::WeightedSkinDecal),
            MATERIAL_TYPE_WEIGHTED_SKIN_DECAL_DIRTMAP => Ok(Self::WeightedSkinDecalDirtmap),
            MATERIAL_TYPE_WEIGHTED_SKIN_DIRTMAP => Ok(Self::WeightedSkinDirtmap),
            MATERIAL_TYPE_WATER => Ok(Self::Water),
            MATERIAL_TYPE_UNLIT => Ok(Self::Unlit),
            MATERIAL_TYPE_WEIGHTED_UNLIT => Ok(Self::WeightedUnlit),
            MATERIAL_TYPE_TERRAIN_BLEND => Ok(Self::TerrainBlend),
            MATERIAL_TYPE_PROJECTED_DECAL_V2 => Ok(Self::ProjectedDecalV2),
            MATERIAL_TYPE_IGNORE => Ok(Self::Ignore),
            MATERIAL_TYPE_TREE_BILLBOARD_MATERIAL => Ok(Self::TreeBillboardMaterial),
            MATERIAL_TYPE_RS_RIVER => Ok(Self::RsRiver),
            MATERIAL_TYPE_WATER_DISPLACE_VOLUME => Ok(Self::WaterDisplaceVolume),
            MATERIAL_TYPE_ROPE => Ok(Self::Rope),
            MATERIAL_TYPE_CAMPAIGN_VEGETATION => Ok(Self::CampaignVegetation),
            MATERIAL_TYPE_PROJECTED_DECAL_V3 => Ok(Self::ProjectedDecalV3),
            MATERIAL_TYPE_WEIGHTED_TEXTURE_BLEND => Ok(Self::WeightedTextureBlend),
            MATERIAL_TYPE_PROJECTED_DECAL_V4 => Ok(Self::ProjectedDecalV4),
            MATERIAL_TYPE_GLOBAL_TERRAIN => Ok(Self::GlobalTerrain),
            MATERIAL_TYPE_DECAL_OVERLAY => Ok(Self::DecalOverlay),
            MATERIAL_TYPE_ALPHA_BLEND => Ok(Self::AlphaBlend),
            _ => Err(RLibError::DecodingRigidModelUnsupportedMaterialType(value))
        }
    }
}

impl From<MaterialType> for u16 {
    fn from(value: MaterialType) -> u16 {
        match value {
            MaterialType::BowWave => MATERIAL_TYPE_BOW_WAVE,
            MaterialType::NonRenderable => MATERIAL_TYPE_NON_RENDERABLE,
            MaterialType::TextureComboVertexWind => MATERIAL_TYPE_TEXTURE_COMBO_VERTEX_WIND,
            MaterialType::TextureCombo => MATERIAL_TYPE_TEXTURE_COMBO,
            MaterialType::DecalWaterfall => MATERIAL_TYPE_DECAL_WATERFALL,
            MaterialType::StandardSimple => MATERIAL_TYPE_STANDARD_SIMPLE,
            MaterialType::CampaignTrees => MATERIAL_TYPE_CAMPAIGN_TREES,
            MaterialType::PointLight => MATERIAL_TYPE_POINT_LIGHT,
            MaterialType::StaticPointLight => MATERIAL_TYPE_STATIC_POINT_LIGHT,
            MaterialType::DebugGeometry => MATERIAL_TYPE_DEBUG_GEOMETRY,
            MaterialType::CustomTerrain => MATERIAL_TYPE_CUSTOM_TERRAIN,
            MaterialType::WeightedCloth => MATERIAL_TYPE_WEIGHTED_CLOTH,
            MaterialType::Cloth => MATERIAL_TYPE_CLOTH,
            MaterialType::Collision => MATERIAL_TYPE_COLLISION,
            MaterialType::CollisionShape => MATERIAL_TYPE_COLLISION_SHAPE,
            MaterialType::TiledDirtmap => MATERIAL_TYPE_TILED_DIRTMAP,
            MaterialType::ShipAmbientmap => MATERIAL_TYPE_SHIP_AMBIENTMAP,
            MaterialType::Weighted => MATERIAL_TYPE_WEIGHTED,
            MaterialType::RsTerrain => MATERIAL_TYPE_RS_TERRAIN,
            MaterialType::ProjectedDecal => MATERIAL_TYPE_PROJECTED_DECAL,
            MaterialType::DefaultMaterial => MATERIAL_TYPE_DEFAULT_MATERIAL,
            MaterialType::Grass => MATERIAL_TYPE_GRASS,
            MaterialType::WeightedSkin => MATERIAL_TYPE_WEIGHTED_SKIN,
            MaterialType::Decal => MATERIAL_TYPE_DECAL,
            MaterialType::DecalDirtmap => MATERIAL_TYPE_DECAL_DIRTMAP,
            MaterialType::Dirtmap => MATERIAL_TYPE_DIRTMAP,
            MaterialType::Tree => MATERIAL_TYPE_TREE,
            MaterialType::TreeLeaf => MATERIAL_TYPE_TREE_LEAF,
            MaterialType::WeightedDecal => MATERIAL_TYPE_WEIGHTED_DECAL,
            MaterialType::WeightedDecalDirtmap => MATERIAL_TYPE_WEIGHTED_DECAL_DIRTMAP,
            MaterialType::WeightedDirtmap => MATERIAL_TYPE_WEIGHTED_DIRTMAP,
            MaterialType::WeightedSkinDecal => MATERIAL_TYPE_WEIGHTED_SKIN_DECAL,
            MaterialType::WeightedSkinDecalDirtmap => MATERIAL_TYPE_WEIGHTED_SKIN_DECAL_DIRTMAP,
            MaterialType::WeightedSkinDirtmap => MATERIAL_TYPE_WEIGHTED_SKIN_DIRTMAP,
            MaterialType::Water => MATERIAL_TYPE_WATER,
            MaterialType::Unlit => MATERIAL_TYPE_UNLIT,
            MaterialType::WeightedUnlit => MATERIAL_TYPE_WEIGHTED_UNLIT,
            MaterialType::TerrainBlend => MATERIAL_TYPE_TERRAIN_BLEND,
            MaterialType::ProjectedDecalV2 => MATERIAL_TYPE_PROJECTED_DECAL_V2,
            MaterialType::Ignore => MATERIAL_TYPE_IGNORE,
            MaterialType::TreeBillboardMaterial => MATERIAL_TYPE_TREE_BILLBOARD_MATERIAL,
            MaterialType::RsRiver => MATERIAL_TYPE_RS_RIVER,
            MaterialType::WaterDisplaceVolume => MATERIAL_TYPE_WATER_DISPLACE_VOLUME,
            MaterialType::Rope => MATERIAL_TYPE_ROPE,
            MaterialType::CampaignVegetation => MATERIAL_TYPE_CAMPAIGN_VEGETATION,
            MaterialType::ProjectedDecalV3 => MATERIAL_TYPE_PROJECTED_DECAL_V3,
            MaterialType::WeightedTextureBlend => MATERIAL_TYPE_WEIGHTED_TEXTURE_BLEND,
            MaterialType::ProjectedDecalV4 => MATERIAL_TYPE_PROJECTED_DECAL_V4,
            MaterialType::GlobalTerrain => MATERIAL_TYPE_GLOBAL_TERRAIN,
            MaterialType::DecalOverlay => MATERIAL_TYPE_DECAL_OVERLAY,
            MaterialType::AlphaBlend => MATERIAL_TYPE_ALPHA_BLEND,
        }
    }
}

impl TryFrom<i32> for TextureType {
    type Error = RLibError;
    fn try_from(value: i32) -> Result<Self> {
        match value {
            TEXTURE_TYPE_DIFFUSE => Ok(Self::Diffuse),
            TEXTURE_TYPE_NORMAL => Ok(Self::Normal),
            TEXTURE_TYPE_MASK => Ok(Self::Mask),
            TEXTURE_TYPE_AMBIENT_OCCLUSION => Ok(Self::AmbientOcclusion),
            TEXTURE_TYPE_TILING_DIRT_UV2 => Ok(Self::TilingDirtUV2),
            TEXTURE_TYPE_DIRT_ALPHA_MASK => Ok(Self::DirtAlphaMask),
            TEXTURE_TYPE_SKIN_MASK => Ok(Self::SkinMask),
            TEXTURE_TYPE_SPECULAR => Ok(Self::Specular),
            TEXTURE_TYPE_GLOSS_MAP => Ok(Self::GlossMap),
            TEXTURE_TYPE_DECAL_DIRTMAP => Ok(Self::DecalDirtmap),
            TEXTURE_TYPE_DECAL_DIRTMASK => Ok(Self::DecalDirtmask),
            TEXTURE_TYPE_DECAL_MASK => Ok(Self::DecalMask),
            TEXTURE_TYPE_DIFFUSE_DAMAGE => Ok(Self::DiffuseDamage),
            TEXTURE_TYPE_BASE_COLOR => Ok(Self::BaseColor),
            TEXTURE_TYPE_MATERIAL_MAP => Ok(Self::MaterialMap),
            _ => Err(RLibError::DecodingRigidModelUnknownTextureType(value))
        }
    }
}

impl TryFrom<TextureType> for i32 {
    type Error = RLibError;
    fn try_from(value: TextureType) -> Result<i32> {
        match value {
            TextureType::Diffuse => Ok(TEXTURE_TYPE_DIFFUSE),
            TextureType::Normal => Ok(TEXTURE_TYPE_NORMAL),
            TextureType::Mask => Ok(TEXTURE_TYPE_MASK),
            TextureType::AmbientOcclusion => Ok(TEXTURE_TYPE_AMBIENT_OCCLUSION),
            TextureType::TilingDirtUV2 => Ok(TEXTURE_TYPE_TILING_DIRT_UV2),
            TextureType::DirtAlphaMask => Ok(TEXTURE_TYPE_DIRT_ALPHA_MASK),
            TextureType::SkinMask => Ok(TEXTURE_TYPE_SKIN_MASK),
            TextureType::Specular => Ok(TEXTURE_TYPE_SPECULAR),
            TextureType::GlossMap => Ok(TEXTURE_TYPE_GLOSS_MAP),
            TextureType::DecalDirtmap => Ok(TEXTURE_TYPE_DECAL_DIRTMAP),
            TextureType::DecalDirtmask => Ok(TEXTURE_TYPE_DECAL_DIRTMASK),
            TextureType::DecalMask => Ok(TEXTURE_TYPE_DECAL_MASK),
            TextureType::DiffuseDamage => Ok(TEXTURE_TYPE_DIFFUSE_DAMAGE),
            TextureType::BaseColor => Ok(TEXTURE_TYPE_BASE_COLOR),
            TextureType::MaterialMap => Ok(TEXTURE_TYPE_MATERIAL_MAP),
        }
    }
}
