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
pub struct GoOutlines {
    empire_outline: Vec<Outline2d>
}

//---------------------------------------------------------------------------//
//                Implementation of GoOutlines
//---------------------------------------------------------------------------//

impl Decodeable for GoOutlines {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.empire_outline.push(Outline2d::decode(data, extra_data)?);
        }

        Ok(decoded)
    }
}

impl Encodeable for GoOutlines {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W,extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.empire_outline.len() as u32)?;
        for outline in &mut self.empire_outline {
            outline.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
