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
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct NonTerrainOutlines {
    empire_outline: Vec<Outline>
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Outline {
    outline: Vec<Position>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Position {
    x: u32,
    y: u32,
}

//---------------------------------------------------------------------------//
//                Implementation of NonTerrainOutlines
//---------------------------------------------------------------------------//

impl Decodeable for NonTerrainOutlines {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            let mut outline = Outline::default();

            for _ in 0..data.read_u32()? {
                outline.outline.push(Position {
                    x: data.read_u32()?,
                    y: data.read_u32()?
                });
            }

            decoded.empire_outline.push(outline);
        }

        Ok(decoded)
    }
}

impl Encodeable for NonTerrainOutlines {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.empire_outline.len() as u32)?;
        for outline in &self.empire_outline {

            buffer.write_u32(outline.outline.len() as u32)?;
            for position in &outline.outline {
                buffer.write_u32(position.x)?;
                buffer.write_u32(position.y)?;
            }
        }

        Ok(())
    }
}
