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

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".cs2.parsed";

#[cfg(test)] mod cs2_parsed_test;

mod versions;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Cs2Parsed {
    version: u32,
    str_1: String,
    matrix_1: Transform4x4,
    int_1: i32,
    pieces: Vec<Piece>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Piece {
    str_2: String,
    str_3: String,
    matrix_2: Transform4x4,
    int_3: i32,
    int_4: i32,                 // Only in v21.
    destructs: Vec<Destruct>,
    f_6: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Destruct {
    key: String,
    i_1: u32,
    collision_outlines: Vec<CollisionOutline>,
    orange_thingies: Vec<Vec<OrangeThingy>>,
    platforms: Vec<Platform>,
    i_2: i32,
    m_1: Transform3x4,
    pipes: Vec<Pipe>,
    ef_lines: Vec<EFLine>,
    docking_lines: Vec<DockingLine>,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CollisionOutline {
    key: String,
    line: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct OrangeThingy {
    f_1: f32,
    f_2: f32,
    u_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Platform {
    f_1: f32,
    f_2: f32,
    f_3: f32,
    line: Outline3d,
    b_1: bool,
    b_2: bool,
    b_3: bool,
}


#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Pipe {
    key: String,
    line: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct EFLine {
    key: String,
    uk_1: u32,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
    f_6: f32,
    f_7: f32,
    f_8: f32,
    f_9: f32,
    uk_2: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct DockingLine {
    key: String,
    f_0: f32,
    f_1: f32,
    f_2: f32,
    f_3: f32,
    f_4: f32,
    f_5: f32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Cs2Parsed
//---------------------------------------------------------------------------//

impl Decodeable for Cs2Parsed {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.version = data.read_u32()?;

        match decoded.version {
            21 => decoded.read_v21(data)?,
            20 => decoded.read_v20(data)?,
             _ => return Err(RLibError::DecodingUnsupportedVersion(decoded.version as usize)),
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for Cs2Parsed {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            21 => self.write_v21(buffer)?,
            20 => self.write_v20(buffer)?,
            _ => unimplemented!()
        }


        Ok(())
    }
}
