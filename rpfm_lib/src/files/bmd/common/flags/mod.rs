//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Flag definitions for BMD entities.
//!
//! This module defines the [`Flags`] structure containing boolean flags that control
//! entity behavior, placement, and visibility in BMD files.
//!
//! # Flag Categories
//!
//! - **Placement**: `allow_in_outfield`, `clamp_to_surface`, `clamp_to_water_surface`
//! - **Seasonal**: `spring`, `summer`, `autumn`, `winter`
//! - **Visibility**: `visible_in_tactical_view`, `visible_in_tactical_view_only`
//!
//! # Supported Versions
//!
//! - **Version 1**: Initial format
//! - **Version 2**: Enhanced format
//! - **Version 3**: Additional flags
//! - **Version 4**: Current format
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::common::flags::Flags;
//! use rpfm_lib::files::Decodeable;
//!
//! let flags = Flags::decode(&mut reader, &None)?;
//! if *flags.spring() && *flags.visible_in_tactical_view() {
//!     println!("Visible in spring tactical view");
//! }
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v1;
mod v2;
mod v3;
mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Boolean flags controlling entity behavior and visibility.
///
/// This structure contains various flags that control how entities (buildings,
/// props, etc.) behave in the game, including placement rules, seasonal visibility,
/// and tactical view settings.
///
/// # Flag Categories
///
/// ## Placement Flags
/// - `allow_in_outfield`: Entity can be placed outside the playable area
/// - `clamp_to_surface`: Entity position is clamped to terrain surface
/// - `clamp_to_water_surface`: Entity position is clamped to water surface
///
/// ## Seasonal Flags
/// - `spring`, `summer`, `autumn`, `winter`: Entity is visible in specified seasons
///
/// ## Visibility Flags
/// - `visible_in_tactical_view`: Entity is visible in tactical camera view
/// - `visible_in_tactical_view_only`: Entity is only visible in tactical view
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::flags::Flags;
///
/// let mut flags = Flags::default();
/// flags.set_serialise_version(4);
/// flags.set_spring(true);
/// flags.set_summer(true);
/// flags.set_clamp_to_surface(true);
/// flags.set_visible_in_tactical_view(true);
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Flags {
    /// Format version number (1-4).
    serialise_version: u16,

    /// Whether the entity can be placed in the outfield (outside playable area).
    allow_in_outfield: bool,

    /// Whether the entity position is clamped to the terrain surface.
    clamp_to_surface: bool,

    /// Whether the entity position is clamped to the water surface.
    clamp_to_water_surface: bool,

    /// Whether the entity is visible during spring season.
    spring: bool,

    /// Whether the entity is visible during summer season.
    summer: bool,

    /// Whether the entity is visible during autumn season.
    autumn: bool,

    /// Whether the entity is visible during winter season.
    winter: bool,

    /// Whether the entity is visible in tactical camera view.
    visible_in_tactical_view: bool,

    /// Whether the entity is only visible in tactical view (hidden in normal view).
    visible_in_tactical_view_only: bool,
}

//---------------------------------------------------------------------------//
//                           Implementation of Flags
//---------------------------------------------------------------------------//

impl Decodeable for Flags {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut flags = Self::default();
        flags.serialise_version = data.read_u16()?;

        match flags.serialise_version {
            1 => flags.read_v1(data, extra_data)?,
            2 => flags.read_v2(data, extra_data)?,
            3 => flags.read_v3(data, extra_data)?,
            4 => flags.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("Flags"), flags.serialise_version)),
        }

        Ok(flags)
    }
}

impl Encodeable for Flags {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            2 => self.write_v2(buffer, extra_data)?,
            3 => self.write_v3(buffer, extra_data)?,
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("Flags"), self.serialise_version)),
        }

        Ok(())
    }
}
