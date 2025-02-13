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
    bounding_box: Transform4x4,
    int_1: i32,
    pieces: Vec<Piece>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Piece {
    name: String,
    node_name: String,
    node_transform: Transform4x4,
    int_3: i32,
    int_4: i32,                 // Only in v21. Array.
    destructs: Vec<Destruct>,
    f_6: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Destruct {
    name: String,
    index: u32,
    collision_outlines: Vec<CollisionOutline>,
    pipes: Vec<Pipe>,
    orange_thingies: Vec<Vec<OrangeThingy>>,
    platforms: Vec<Platform>,
    //uk_1: Vec<u16>,                         // No clue. It's there only sometimes.
    uk_2: i32,
    bounding_box: Cube,                      // Same, no clue. Only there in some files.
    uk_3: i32,
    uk_4: i32,
    uk_5: i32,
    uk_6: i32,
    uk_7: i32,
    file_refs: Vec<FileRef>,
    ef_lines: Vec<EFLine>,
    docking_lines: Vec<DockingLine>,
    f_1: f32,                               // Another array
    action_vfx: Vec<Vfx>,
    action_vfx_attachments: Vec<Vfx>,
    bin_data: Vec<Vec<i16>>,                // no idea, but looks like a list of values.
    f_5: f32,                               // And no idea.
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CollisionOutline {
    name: String,
    vertices: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct FileRef {
    key: String,
    name: String,
    transform: Transform4x4,
    uk_1: i16,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct OrangeThingy {
    vertex: Point2d,
    vertex_type: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Platform {
    normal: Point3d,
    vertices: Outline3d,
    flag_1: bool,
    flag_2: bool,
    flag_3: bool,
}


#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Pipe {
    name: String,
    line: Outline3d,
    line_type: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct EFLine {
    name: String,
    action: u32,
    start: Point3d,
    end: Point3d,
    direction: Point3d,
    parent_index: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct DockingLine {
    key: String,
    start: Point2d,
    end: Point2d,
    direction: Point2d,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Vfx {
    key: String,
    matrix_1: Transform4x4,
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
