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

use super::*;

mod v9;
mod v10;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CompositeSceneReference {
    serialise_version: u16,
    transform: Transform3x4,
    scene_file: String,
    height_mode: String,
    pdlc_mask: u64,
    autoplay: bool,
    visible_in_shroud: bool,
    no_culling: bool,
    script_id: String,
    parent_script_id: String,
    visible_without_shroud: bool,
    visible_in_tactical_view: bool,
    visible_in_tactical_view_only: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of CompositeSceneReference
//---------------------------------------------------------------------------//

impl Decodeable for CompositeSceneReference {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            9 => decoded.read_v9(data, extra_data)?,
            10 => decoded.read_v10(data, extra_data)?,
            11 => decoded.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CompositeSceneReference"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for CompositeSceneReference {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            9 => self.write_v9(buffer, extra_data)?,
            10 => self.write_v10(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("CompositeSceneReference"), self.serialise_version)),
        }

        Ok(())
    }
}
