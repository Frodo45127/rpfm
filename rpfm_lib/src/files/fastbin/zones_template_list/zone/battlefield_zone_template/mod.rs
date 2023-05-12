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
pub struct BattlefiedZoneTemplate {
    outline: Outline,
    zone_name: String,
    entity_formation_template: EntityFormationTemplate
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EntityFormationTemplate {
    name: String,
    lines: Vec<Line>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Line {
    label: String,
    start: Point3d,
    end: Point3d,
    purpose: String,
    orientation: f32,
}

//---------------------------------------------------------------------------//
//                Implementation of BattlefiedZoneTemplate
//---------------------------------------------------------------------------//

impl Decodeable for BattlefiedZoneTemplate {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        decoded.outline = Outline::decode(data, extra_data)?;
        decoded.zone_name = data.read_sized_string_u8()?;
        decoded.entity_formation_template.name = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            let mut line = Line::default();
            line.label = data.read_sized_string_u8()?;
            line.start = Point3d::decode(data, extra_data)?;
            line.end = Point3d::decode(data, extra_data)?;
            line.purpose = data.read_sized_string_u8()?;
            line.orientation = data.read_f32()?;

            decoded.entity_formation_template.lines.push(line);
        }

        Ok(decoded)
    }
}

impl Encodeable for BattlefiedZoneTemplate {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.outline.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.zone_name)?;
        buffer.write_sized_string_u8(&self.entity_formation_template.name)?;

        buffer.write_u32(self.entity_formation_template.lines.len() as u32)?;
        for line in &mut self.entity_formation_template.lines {
            buffer.write_sized_string_u8(&line.label)?;

            line.start.encode(buffer, extra_data)?;
            line.end.encode(buffer, extra_data)?;

            buffer.write_sized_string_u8(&line.purpose)?;
            buffer.write_f32(line.orientation)?;
        }

        Ok(())
    }
}

