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
pub struct Outline {
    outline: Vec<Position>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Position {
    x: f32,
    y: f32,
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
    start: Position3D,
    end: Position3D,
    purpose: String,
    orientation: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Position3D {
    x: f32,
    y: f32,
    z: f32
}

//---------------------------------------------------------------------------//
//                Implementation of BattlefiedZoneTemplate
//---------------------------------------------------------------------------//

impl Decodeable for BattlefiedZoneTemplate {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.outline.outline.push(Position {
                x: data.read_f32()?,
                y: data.read_f32()?
            });
        }

        decoded.zone_name = data.read_sized_string_u8()?;
        decoded.entity_formation_template.name = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            let mut line = Line::default();
            line.label = data.read_sized_string_u8()?;
            line.start = Position3D {
                x: data.read_f32()?,
                y: data.read_f32()?,
                z: data.read_f32()?,
            };
            line.end = Position3D {
                x: data.read_f32()?,
                y: data.read_f32()?,
                z: data.read_f32()?,
            };
            line.purpose = data.read_sized_string_u8()?;
            line.orientation = data.read_f32()?;

            decoded.entity_formation_template.lines.push(line);
        }

        Ok(decoded)
    }
}

impl Encodeable for BattlefiedZoneTemplate {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {

        buffer.write_u32(self.outline.outline.len() as u32)?;
        for position in &self.outline.outline {
            buffer.write_f32(position.x)?;
            buffer.write_f32(position.y)?;
        }

        buffer.write_sized_string_u8(&self.zone_name)?;
        buffer.write_sized_string_u8(&self.entity_formation_template.name)?;

        buffer.write_u32(self.entity_formation_template.lines.len() as u32)?;
        for line in &self.entity_formation_template.lines {
            buffer.write_sized_string_u8(&line.label)?;
            buffer.write_f32(line.start.x)?;
            buffer.write_f32(line.start.y)?;
            buffer.write_f32(line.start.z)?;

            buffer.write_f32(line.end.x)?;
            buffer.write_f32(line.end.y)?;
            buffer.write_f32(line.end.z)?;

            buffer.write_sized_string_u8(&line.purpose)?;
            buffer.write_f32(line.orientation)?;
        }

        Ok(())
    }
}

