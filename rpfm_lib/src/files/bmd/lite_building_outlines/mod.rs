
//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct LiteBuildingOutlines {
    lines: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                Implementation of LiteBuildingOutlines
//---------------------------------------------------------------------------//

impl Decodeable for LiteBuildingOutlines {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let decoded = Self::default();

        for _ in 0..data.read_u32()? {

        }

        Ok(decoded)
    }
}

impl Encodeable for LiteBuildingOutlines {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.lines.len() as u32)?;

        Ok(())
    }
}

 
