//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use self::battlefield_zone_template::BattlefiedZoneTemplate;

use super::*;

mod battlefield_zone_template;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Zone {
    battlefield_zone_template: BattlefiedZoneTemplate,
    transform: Transform4x4,
}

//---------------------------------------------------------------------------//
//                     Implementation of Zone
//---------------------------------------------------------------------------//

impl Decodeable for Zone {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        decoded.battlefield_zone_template = BattlefiedZoneTemplate::decode(data, extra_data)?;
        decoded.transform = Transform4x4::decode(data, extra_data)?;

        Ok(decoded)
    }
}

impl Encodeable for Zone {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {

        self.battlefield_zone_template.encode(buffer, extra_data)?;
        self.transform.encode(buffer, extra_data)?;

        Ok(())
    }
}
