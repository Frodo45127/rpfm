//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! BMD Vegetation file format support.
//!
//! BMD Vegetation files (`.vegetation`) are FASTBIN0-format files that define vegetation
//! placement for battle maps in Total War games. They contain tree and grass placement
//! data that complements the main BMD file.
//!
//! # File Format
//!
//! BMD Vegetation files use the FASTBIN0 binary format:
//!
//! ```text
//! [8 bytes]  FASTBIN0 signature
//! [u16]      serialise_version
//! [...]      version-specific data
//! ```
//!
//! # Supported Versions
//!
//! - **Version 2**: Current format used in Warhammer III
//!
//! # File Contents
//!
//! - **Tree List**: Individual tree placements with species, position, scale, rotation
//! - **Grass List**: Grass placement areas with density and appearance settings
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd_vegetation::BmdVegetation;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode a vegetation file
//! let vegetation = BmdVegetation::decode(&mut reader, &None)?;
//!
//! println!("Version: {}", vegetation.serialise_version());
//! println!("Trees: {}", vegetation.tree_list().list().len());
//! println!("Grass patches: {}", vegetation.grass_list().list().len());
//! ```
//!
//! # Integration with BMD
//!
//! Vegetation files are referenced from BMD files via:
//! - [`TreeListReferenceList`](crate::files::bmd::Bmd) - Links to tree placement data
//! - [`GrassListReferenceList`](crate::files::bmd::Bmd) - Links to grass coverage data
//!
//! # Terry Export
//!
//! Vegetation data can be exported to Terry (CA's editor) format via the [`ToLayer`]
//! trait implementation, which generates XML entity definitions for trees and grass.
//!
//! # File Location
//!
//! Vegetation files are typically found in:
//! ```text
//! terrain/battles/*.vegetation
//! terrain/tiles/*.vegetation
//! ```

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Bmd, bmd::ToLayer, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::grass_list::GrassList;
use self::tree_list::TreeList;

use super::DecodeableExtraData;

/// File extension for BMD Vegetation files.
///
/// BMD Vegetation files use the `.vegetation` extension.
pub const EXTENSIONS: [&str; 1] = [
    ".vegetation",
];

/// FASTBIN0 file signature.
///
/// All BMD Vegetation files start with this 8-byte signature: `FASTBIN0`
/// (bytes: `[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30]`)
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

mod grass_list;
mod tree_list;
mod v2;

#[cfg(test)] mod bmd_vegetation_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a BMD Vegetation file decoded in memory.
///
/// This struct contains all vegetation placement data for a battle map, including
/// individual tree instances and grass coverage areas. Vegetation data is stored
/// separately from the main BMD file to allow independent editing.
///
/// # Fields
///
/// - `serialise_version`: File format version (currently only version 2)
/// - `tree_list`: List of individual tree placements
/// - `grass_list`: List of grass coverage areas
///
/// # Version Support
///
/// - **Version 2**: Current format used in Warhammer III
///
/// # Integration
///
/// Vegetation files work in conjunction with BMD files. The BMD file contains
/// references to vegetation data via `TreeListReferenceList` and `GrassListReferenceList`,
/// while this file contains the actual placement data.
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd_vegetation::BmdVegetation;
/// use rpfm_lib::files::Decodeable;
///
/// let vegetation = BmdVegetation::decode(&mut reader, &None)?;
///
/// // Access tree data
/// for tree in vegetation.tree_list().list() {
///     println!("Tree at ({}, {}, {})",
///         tree.position().x(),
///         tree.position().y(),
///         tree.position().z()
///     );
/// }
///
/// // Access grass data
/// for grass in vegetation.grass_list().list() {
///     println!("Grass patch: {}", grass.grass_type());
/// }
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BmdVegetation {
    /// File format version number (currently only version 2).
    serialise_version: u16,

    /// List of individual tree placements.
    tree_list: TreeList,

    /// List of grass coverage areas.
    grass_list: GrassList,
}

//---------------------------------------------------------------------------//
//                           Implementation of BmdVegetation
//---------------------------------------------------------------------------//

impl Decodeable for BmdVegetation {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(8, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingFastBinUnsupportedSignature(signature_bytes));
        }

        let mut fastbin = Self::default();
        fastbin.serialise_version = data.read_u16()?;

        match fastbin.serialise_version {
            2 => fastbin.read_v2(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BmdVegetation"), fastbin.serialise_version)),
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fastbin)
    }
}

impl Encodeable for BmdVegetation {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            2 => self.write_v2(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BmdVegetation"), self.serialise_version)),
        }

        Ok(())
    }
}


impl ToLayer for BmdVegetation {
    fn to_layer(&self, parent: &Bmd) -> Result<String> {
        let mut layer = String::new();

        layer.push_str(&self.tree_list().to_layer(parent)?);
        //layer.push_str(self.grass_list().to_layer(parent)?);

        Ok(layer)
    }
}
