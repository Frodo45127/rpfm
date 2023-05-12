//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::flags::Flags;

use super::*;

mod flags;
mod v4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CustomMaterialMesh {
    serialise_version: u16,
    vertices: Vec<Point3d>,
    indices: Vec<u16>,
    material: String,
    height_mode: String,
    flags: Flags,
    transform: Transform3x4,
    snow_inside: bool,
    snow_outside: bool,
    destruction_inside: bool,
    destruction_outside: bool,
    visible_in_shroud: bool,
    visible_without_shroud: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of CustomMaterialMesh
//---------------------------------------------------------------------------//

impl Decodeable for CustomMaterialMesh {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            4 => decoded.read_v4(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CustomMaterialMesh"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for CustomMaterialMesh {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            4 => self.write_v4(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("CustomMaterialMesh"), self.serialise_version)),
        }

        Ok(())
    }
}
