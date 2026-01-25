//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Building link data structure for BMD files.
//!
//! This module defines the [`BuildingLink`] structure which links building instances
//! to prefab instances in BMD files. Building links establish relationships between
//! buildings and their associated prefabs.
//!
//! # Supported Versions
//!
//! - **Version 1**: Initial format
//! - **Version 2**: Enhanced format
//! - **Version 3**: Current format
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::common::building_link::BuildingLink;
//! use rpfm_lib::files::Decodeable;
//!
//! let link = BuildingLink::decode(&mut reader, &None)?;
//! println!("Building index: {}", link.building_index());
//! println!("Prefab key: {}", link.prefab_building_key());
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::building_reference::BuildingReference;

use super::*;

mod v1;
mod v2;
mod v3;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Links a building instance to a prefab instance.
///
/// Building links establish relationships between buildings in the battlefield
/// building list and their associated prefab instances. This allows buildings
/// to reference prefab data for models, textures, and other assets.
///
/// # Fields
///
/// - `serialise_version`: Format version (1-3)
/// - `building_index`: Index of the building in the building list
/// - `prefab_index`: Index of the prefab in the prefab list
/// - `prefab_building_key`: String key identifying the prefab building
/// - `uid`: Unique identifier for this building link
/// - `prefab_uid`: Unique identifier of the associated prefab
/// - `building_reference`: Reference data for the building
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::building_link::BuildingLink;
///
/// let mut link = BuildingLink::default();
/// link.set_serialise_version(3);
/// link.set_building_index(5);
/// link.set_prefab_building_key("settlement_wall_01".to_string());
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BuildingLink {
    /// Format version number (1-3).
    serialise_version: u16,

    /// Index of the building in the building list.
    building_index: i32,

    /// Index of the prefab in the prefab list.
    prefab_index: i32,

    /// String key identifying the prefab building.
    prefab_building_key: String,

    /// Unique identifier for this building link.
    uid: u64,

    /// Unique identifier of the associated prefab.
    prefab_uid: u64,

    /// Reference data for the building.
    building_reference: BuildingReference
}

//---------------------------------------------------------------------------//
//                Implementation of BuildingLink
//---------------------------------------------------------------------------//

impl Decodeable for BuildingLink {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            2 => decoded.read_v2(data, extra_data)?,
            3 => decoded.read_v3(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BuildingLink"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for BuildingLink {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            2 => self.write_v2(buffer, extra_data)?,
            3 => self.write_v3(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BuildingLink"), self.serialise_version)),
        }

        Ok(())
    }
}
