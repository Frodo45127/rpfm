//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Building reference data structure for BMD files.
//!
//! This module defines the [`BuildingReference`] structure which provides a reference
//! to a building in the battlefield building list by its index.
//!
//! # Supported Versions
//!
//! - **Version 1**: Current format
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::common::building_reference::BuildingReference;
//! use rpfm_lib::files::Decodeable;
//!
//! let reference = BuildingReference::decode(&mut reader, &None)?;
//! println!("Building index: {}", reference.building_index());
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Reference to a building by its index in the building list.
///
/// Provides a simple index-based reference to a building instance in the
/// battlefield building list. Used by building links and other structures
/// to refer to specific buildings.
///
/// # Fields
///
/// - `serialise_version`: Format version (currently only version 1)
/// - `building_index`: Index of the referenced building
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::building_reference::BuildingReference;
///
/// let mut reference = BuildingReference::default();
/// reference.set_serialise_version(1);
/// reference.set_building_index(42);
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BuildingReference {
    /// Format version number (currently only version 1).
    serialise_version: u16,

    /// Index of the referenced building in the building list.
    building_index: i32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Properties
//---------------------------------------------------------------------------//

impl Decodeable for BuildingReference {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut flags = Self::default();
        flags.serialise_version = data.read_u16()?;

        match flags.serialise_version {
            1 => flags.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BuildingReference"), flags.serialise_version)),
        }

        Ok(flags)
    }
}

impl Encodeable for BuildingReference {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BuildingReference"), self.serialise_version)),
        }

        Ok(())
    }
}
