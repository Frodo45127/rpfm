//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

/// Version of the civilian shelter records. No other is known.
const SHELTER_VERSION: u16 = 1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Places where civilians shelter during a battle.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianShelterList {
    civilian_shelter_list: Vec<CivilianShelter>,
}

/// A place where civilians shelter.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianShelter {

    /// Area of the shelter.
    area: Rectangle,

    /// How many civilians fit in the shelter. `u32::MAX` in most shelters, likely meaning no limit.
    capacity: u32,
}

//---------------------------------------------------------------------------//
//                Implementation of CivilianShelterList
//---------------------------------------------------------------------------//

impl Decodeable for CivilianShelterList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            let version = data.read_u16()?;
            if version != SHELTER_VERSION {
                return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CivilianShelter"), version));
            }

            decoded.civilian_shelter_list.push(CivilianShelter {
                area: Rectangle::decode(data, extra_data)?,
                capacity: data.read_u32()?,
            });
        }

        Ok(decoded)
    }
}

impl Encodeable for CivilianShelterList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.civilian_shelter_list.len() as u32)?;

        for shelter in &mut self.civilian_shelter_list {
            buffer.write_u16(SHELTER_VERSION)?;
            shelter.area.encode(buffer, extra_data)?;
            buffer.write_u32(shelter.capacity)?;
        }

        Ok(())
    }
}
