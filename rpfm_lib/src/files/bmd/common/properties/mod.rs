//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Building and entity property definitions for BMD files.
//!
//! This module defines the [`Properties`] structure containing various properties
//! that control building behavior, gameplay mechanics, and rendering settings.
//!
//! # Property Categories
//!
//! - **State**: `on_fire`, `start_disabled`, `starting_damage_unary`
//! - **Gameplay**: `weak_point`, `ai_breachable`, `indestructible`, `dockable`, `toggleable`
//! - **Rendering**: `lite`, `cast_shadows`, `clamp_to_surface`, `include_in_fog`
//! - **Strategic**: `key_building`, `key_building_use_fort`, `settlement_level_configurable`
//! - **Misc**: `dont_merge_building`, `is_prop_in_outfield`, `hide_tooltip`, `tint_inherit_from_parent`
//!
//! # Supported Versions
//!
//! - **Version 4**: Early format
//! - **Version 6**: Mid format
//! - **Version 7**: Enhanced format
//! - **Version 11**: Current format
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::common::properties::Properties;
//! use rpfm_lib::files::Decodeable;
//!
//! let properties = Properties::decode(&mut reader, &None)?;
//! println!("Building ID: {}", properties.building_id());
//! if *properties.indestructible() {
//!     println!("This building cannot be destroyed");
//! }
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v4;
mod v6;
mod v7;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Building and entity properties controlling behavior and rendering.
///
/// This structure contains a comprehensive set of properties that define how
/// buildings and entities behave in the game, including damage states, AI
/// interactions, rendering settings, and strategic importance.
///
/// # Property Categories
///
/// ## State Properties
/// - `building_id`: Unique identifier for the building
/// - `starting_damage_unary`: Initial damage level (0.0 = undamaged, 1.0 = destroyed)
/// - `on_fire`: Whether the building starts on fire
/// - `start_disabled`: Whether the building starts in disabled state
///
/// ## Gameplay Properties
/// - `weak_point`: Whether this is a weak point for siege battles
/// - `ai_breachable`: Whether AI can breach through this building
/// - `indestructible`: Whether the building cannot be destroyed
/// - `dockable`: Whether siege engines can dock at this building
/// - `toggleable`: Whether the building can be toggled on/off
///
/// ## Rendering Properties
/// - `lite`: Use simplified rendering (lower detail)
/// - `clamp_to_surface`: Clamp building position to terrain surface
/// - `cast_shadows`: Whether the building casts shadows
/// - `include_in_fog`: Whether the building is affected by fog of war
/// - `tint_inherit_from_parent`: Inherit color tint from parent entity
///
/// ## Strategic Properties
/// - `key_building`: Whether this is a key strategic building
/// - `key_building_use_fort`: Whether key building uses fort mechanics
/// - `settlement_level_configurable`: Whether properties vary by settlement level
///
/// ## Miscellaneous
/// - `dont_merge_building`: Prevent building mesh merging optimization
/// - `is_prop_in_outfield`: Whether this prop is in the outfield area
/// - `hide_tooltip`: Hide tooltip UI for this building
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::properties::Properties;
///
/// let mut props = Properties::default();
/// props.set_serialise_version(11);
/// props.set_building_id("main_gate".to_string());
/// props.set_indestructible(false);
/// props.set_weak_point(true);
/// props.set_starting_damage_unary(0.0);
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Properties {
    /// Format version number (4, 6, 7, or 11).
    serialise_version: u16,

    /// Unique identifier for this building.
    building_id: String,

    /// Initial damage level (0.0 = undamaged, 1.0 = destroyed).
    starting_damage_unary: f32,

    /// Whether the building starts on fire.
    on_fire: bool,

    /// Whether the building starts in disabled state.
    start_disabled: bool,

    /// Whether this is a weak point for siege battles.
    weak_point: bool,

    /// Whether AI can breach through this building.
    ai_breachable: bool,

    /// Whether the building cannot be destroyed.
    indestructible: bool,

    /// Whether siege engines can dock at this building.
    dockable: bool,

    /// Whether the building can be toggled on/off.
    toggleable: bool,

    /// Use simplified rendering (lower detail).
    lite: bool,

    /// Clamp building position to terrain surface.
    clamp_to_surface: bool,

    /// Whether the building casts shadows.
    cast_shadows: bool,

    /// Prevent building mesh merging optimization.
    dont_merge_building: bool,

    /// Whether this is a key strategic building.
    key_building: bool,

    /// Whether key building uses fort mechanics.
    key_building_use_fort: bool,

    /// Whether this prop is in the outfield area.
    is_prop_in_outfield: bool,

    /// Whether properties vary by settlement level.
    settlement_level_configurable: bool,

    /// Hide tooltip UI for this building.
    hide_tooltip: bool,

    /// Whether the building is affected by fog of war.
    include_in_fog: bool,

    /// Inherit color tint from parent entity.
    tint_inherit_from_parent: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Properties
//---------------------------------------------------------------------------//

impl Decodeable for Properties {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut flags = Self::default();
        flags.serialise_version = data.read_u16()?;

        match flags.serialise_version {
            4 => flags.read_v4(data, extra_data)?,
            6 => flags.read_v6(data, extra_data)?,
            7 => flags.read_v7(data, extra_data)?,
            11 => flags.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Properties"), flags.serialise_version)),
        }

        Ok(flags)
    }
}

impl Encodeable for Properties {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            6 => self.write_v6(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Properties"), self.serialise_version)),
        }

        Ok(())
    }
}
