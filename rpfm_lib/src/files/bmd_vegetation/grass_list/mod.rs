//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Grass placement list for BMD vegetation files.
//!
//! This module defines the grass placement data structure used in BMD vegetation files.
//! Grass data is stored as a compressed binary blob that defines grass coverage areas
//! and density across the battle map.
//!
//! # Structure
//!
//! - [`GrassList`]: Container for grass placement data
//!
//! # Supported Versions
//!
//! - **Version 4**: Current format used in Total War: Warhammer III
//!
//! # Implementation Notes
//!
//! The grass placement data is stored as a raw byte array. The internal format
//! is not fully documented but represents compressed grass coverage information.
//!
//! # Examples
//!
//! ```rust,ignore
//! use rpfm_lib::files::bmd_vegetation::grass_list::*;
//!
//! // Decode a grass list from binary data
//! let grass_list = GrassList::decode(&mut data, &extra_data)?;
//!
//! // Access the raw grass data
//! println!("Grass data size: {} bytes", grass_list.grass_list().len());
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::DecodeableExtraData;

mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Grass placement data for a battle map.
///
/// Contains compressed grass coverage information stored as a binary blob.
/// The exact internal format is implementation-specific and represents
/// grass density and distribution across the terrain.
///
/// # Fields
///
/// * `serialise_version` - File format version (currently 4)
/// * `grass_list` - Raw binary data containing grass coverage information
///
/// # Examples
///
/// ```rust,ignore
/// let grass_list = GrassList::decode(&mut data, &extra_data)?;
/// println!("Grass data: {} bytes", grass_list.grass_list().len());
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GrassList {
    /// File format version number.
    serialise_version: u16,

    /// Raw binary data containing grass coverage information.
    grass_list: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                           Implementation of GrassList
//---------------------------------------------------------------------------//

impl Decodeable for GrassList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            4 => decoded.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("GrassList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for GrassList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("GrassList"), self.serialise_version)),
        }

        Ok(())
    }
}
