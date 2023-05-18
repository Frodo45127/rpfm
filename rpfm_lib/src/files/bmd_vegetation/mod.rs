//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write Bmd Vegetation binary (FASTBIN0) files.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::grass_list::GrassList;
use self::tree_list::TreeList;

use super::DecodeableExtraData;

/// Extensions used by BMD Vegetation files.
pub const EXTENSIONS: [&str; 1] = [
    ".vegetation",
];

/// FASTBIN0
pub const SIGNATURE: &[u8; 8] = &[0x46, 0x41, 0x53, 0x54, 0x42, 0x49, 0x4E, 0x30];

mod grass_list;
mod tree_list;
mod v2;

#[cfg(test)] mod bmd_vegetation_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `BmdVegetation` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BmdVegetation {
    serialise_version: u16,

    tree_list: TreeList,
    grass_list: GrassList,
}

//---------------------------------------------------------------------------//
//                           Implementation of BmdVegetation
//---------------------------------------------------------------------------//

pub trait ToLayer {
    fn to_layer(&self) -> Result<String> {
        Ok(String::new())
    }
}

impl ToLayer for BmdVegetation {
    fn to_layer(&self) -> Result<String> {
        let layer = String::new();
/*
        layer.push_str("
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<layer version=\"41\">
    <entities>"
        );

        layer.push_str(&self.battlefield_building_list().to_layer()?);
        layer.push_str(&self.prefab_instance_list().to_layer()?);

        layer.push_str("
    </entities>
    <associations>
        <Logical/>
        <Transform/>
    </associations>
</layer>
        ");
*/
        Ok(layer)
    }
}

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
