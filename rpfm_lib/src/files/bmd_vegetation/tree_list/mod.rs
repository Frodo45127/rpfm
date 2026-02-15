//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tree placement list for BMD vegetation files.
//!
//! This module defines the tree placement data structures used in BMD vegetation files.
//! Trees are organized by their model path, with each model containing multiple
//! individual tree instances with positions, rotations, and scales.
//!
//! # Structure
//!
//! - [`TreeList`]: Top-level container for all tree placement data
//! - [`BattleTreeItemVector`]: Groups trees by their model path
//! - [`BattleTreeItem`]: Individual tree instance with transform data
//!
//! # Supported Versions
//!
//! - **Version 4**: Current format used in Total War: Warhammer III
//!
//! # Examples
//!
//! ```rust,ignore
//! use rpfm_lib::files::bmd_vegetation::tree_list::*;
//!
//! // Decode a tree list from binary data
//! let tree_list = TreeList::decode(&mut data, &extra_data)?;
//!
//! // Iterate through tree models
//! for tree_vector in tree_list.tree_list() {
//!     println!("Model: {}", tree_vector.key());
//!     for tree in tree_vector.value() {
//!         println!("  Position: ({}, {}, {})", tree.x(), tree.y(), tree.z());
//!     }
//! }
//! ```

use getset::*;
use rand::RngExt;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Bmd, bmd::ToLayer, Decodeable, EncodeableExtraData, Encodeable};

use super::DecodeableExtraData;

mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Tree placement list for a battle map.
///
/// Contains all tree instances organized by their model paths. Each model path
/// maps to a vector of individual tree instances with transform data.
///
/// # Fields
///
/// * `serialise_version` - File format version (currently 4)
/// * `tree_list` - Vector of tree groups, each containing instances of a specific tree model
///
/// # Examples
///
/// ```rust,ignore
/// let tree_list = TreeList::decode(&mut data, &extra_data)?;
/// println!("Tree models: {}", tree_list.tree_list().len());
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TreeList {
    /// File format version number.
    serialise_version: u16,

    /// List of tree instance groups, organized by model path.
    tree_list: Vec<BattleTreeItemVector>,
}

/// A group of tree instances sharing the same model.
///
/// Maps a tree model path (`.rigid_model_v2` file) to a vector of individual
/// tree instances that use that model.
///
/// # Fields
///
/// * `key` - Path to the tree model file (e.g., `BattleTerrain/vegetation/trees/wh_palm/wh_lizardmen_tree_palmbig_e.rigid_model_v2`)
/// * `value` - Vector of individual tree instances using this model
///
/// # Examples
///
/// ```rust,ignore
/// for tree_vector in tree_list.tree_list() {
///     println!("Model: {} ({} instances)", tree_vector.key(), tree_vector.value().len());
/// }
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BattleTreeItemVector {
    /// Path to the tree model file.
    key: String,

    /// Vector of tree instances using this model.
    value: Vec<BattleTreeItem>,
}

/// Individual tree instance with position, rotation, and scale.
///
/// Represents a single tree placed on the battle map with its transform data.
/// The rotation is compressed into a single byte representing Y-axis rotation.
///
/// # Fields
///
/// * `id` - Unique identifier for this tree instance
/// * `x` - X coordinate in world space
/// * `y` - Y coordinate in world space (height)
/// * `z` - Z coordinate in world space
/// * `rotation` - Y-axis rotation compressed to 0-255 range (multiply by 1.40625 to get degrees)
/// * `scale` - Uniform scale factor applied to all axes
/// * `flags` - Behavior flags for this tree instance
///
/// # Examples
///
/// ```rust,ignore
/// let tree = BattleTreeItem {
///     id: 0x1609e7e52919a40,
///     x: -22.92,
///     y: 58.75,
///     z: -3.01,
///     rotation: 72,  // ~102 degrees when converted
///     scale: 1.0,
///     flags: 3,
/// };
/// ```
#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BattleTreeItem {
    /// Unique identifier for this tree instance.
    id: u64,

    /// X coordinate in world space.
    x: f32,

    /// Y coordinate in world space (height).
    y: f32,

    /// Z coordinate in world space.
    z: f32,

    /// Y-axis rotation (0-255 maps to 0-360 degrees).
    ///
    /// To convert to degrees: `rotation as f32 * 1.40625`
    rotation: u8,

    /// Uniform scale factor.
    scale: f32,

    /// Behavior flags for this tree instance.
    flags: u8,
}

//---------------------------------------------------------------------------//
//                           Implementation of TreeList
//---------------------------------------------------------------------------//

impl Decodeable for TreeList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            4 => decoded.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("TreeList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for TreeList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("TreeList"), self.serialise_version)),
        }

        Ok(())
    }
}

impl Default for BattleTreeItem {
    fn default() -> Self {
        Self {
            id: rand::rng().random::<u64>(),
            x: f32::default(),
            y: f32::default(),
            z: f32::default(),
            rotation: u8::default(),
            scale: f32::default(),
            flags: u8::default(),
        }
    }
}

impl ToLayer for TreeList {
    fn to_layer(&self, _parent: &Bmd) -> Result<String> {
        let mut layer = String::new();
/*
            <BATTLE_TREE_ITEM_VECTOR key='BattleTerrain/vegetation/trees/wh_palm/wh_lizardmen_tree_palmbig_e.rigid_model_v2'>
                <value>
                    <BATTLE_TREE_ITEM x='13.430786' y='56.218090' z='64.390381' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='30.591064' y='47.634796' z='64.003784' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-58.119385' y='28.191170' z='-7.750488' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-54.575928' y='44.780090' z='19.081787' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='47.375000' y='40.378334' z='-42.304565' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-46.924316' y='29.631201' z='-61.144165' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='9.264160' y='63.425400' z='56.489380' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='29.056519' y='39.673546' z='-42.304565' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-41.629883' y='58.078644' z='40.268311' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-25.574707' y='43.598698' z='-44.591797' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='48.763184' y='63.282581' z='18.791626' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-65.662598' y='23.835663' z='-43.105835' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='62.529541' y='64.074753' z='-10.465454' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-10.322266' y='58.416050' z='49.374023' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='59.566162' y='41.931854' z='-31.586426' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='44.394775' y='28.780624' z='-50.351074' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='55.461060' y='48.186008' z='30.768555' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-22.920776' y='58.752563' z='-3.012939' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='47.971313' y='72.914719' z='2.633179' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='18.252930' y='62.910236' z='59.119873' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='5.463135' y='39.673546' z='-29.382080' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-29.859375' y='66.881577' z='53.766235' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-15.012939' y='41.917336' z='-44.591797' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-2.984741' y='56.629513' z='67.660400' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-69.040771' y='24.034649' z='-32.138916' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='34.823364' y='60.135620' z='49.923828' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='55.175659' y='68.957443' z='6.442505' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-27.754395' y='48.756340' z='6.032227' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-46.053345' y='54.695683' z='19.081787' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-16.399902' y='58.416046' z='61.042847' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-35.505859' y='64.935822' z='43.448242' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='50.590454' y='48.186008' z='36.953979' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-22.920776' y='58.752563' z='27.214111' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-36.628418' y='41.561756' z='-20.399292' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-22.920654' y='58.752567' z='-12.470703' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-41.240234' y='43.078796' z='-7.750366' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-32.357910' y='30.178255' z='-67.639404' rotation='72' scale='1.000000' flags='3'/>
                    <BATTLE_TREE_ITEM x='-15.012939' y='41.917336' z='-44.591797' rotation='72' scale='1.000000' flags='3'/>
                </value>

            <entity id="1609e7e52919a40">
              <ECVegetation key="BattleTerrain/vegetation/trees/wh_palm/wh_lizardmen_tree_palmbig_e.rigid_model_v2" legacy_random_rotation="true"/>
              <ECTransform position="-22.9207764 58.7525635 -3.01293945" rotation="0 102.000023 0" scale="1 1 1" pivot="0 0 0"/>
              <ECTerrainClamp active="false" clamp_to_sea_level="false" terrain_oriented="false" fit_height_to_terrain="false"/>
            </entity>*/

        for item_vector in self.tree_list() {
            for item in item_vector.value() {

                layer.push_str(&format!("
        <entity id=\"{:x}\">",
                    item.id())
                );

                layer.push_str(&format!("
            <ECVegetation key=\"{}\" legacy_random_rotation=\"true\"/>",
                    item_vector.key(),
                ));

                layer.push_str(&format!("
            <ECTransform position=\"{:.5} {:.5} {:.5}\" rotation=\"{:.5} {:.5} {:.5}\" scale=\"{:.5} {:.5} {:.5}\" pivot=\"0 0 0\"/>",
                    item.x(),
                    item.y(),
                    item.z(),
                    0,
                    *item.rotation() as f32 * 1.40625, // Rotation is only Y-axis, divided by 1.4 to fit a full 360 degree rotation in one byte.
                    0,
                    item.scale(),
                    item.scale(),
                    item.scale(),
                ));

                layer.push_str("
            <ECTerrainClamp active=\"false\" clamp_to_sea_level=\"false\" terrain_oriented=\"false\" fit_height_to_terrain=\"false\"/>");

                layer.push_str("
        </entity>"
                );
            }
        }

        Ok(layer)
    }
}
