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

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".cs2.collision";

#[cfg(test)] mod cs2_collision_test;

mod versions;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct Cs2Collision {
    magic_number: u32,
    version: u32,
    bounding_box: Cube,
    collisions_3d: Vec<Collision3d>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct Collision3d {
    name: String,        //utf-8
    uk_1: i32,           //an id?
    uk_2: i32,           //0 or 1?
    vertices: Vec<Point3d>,
    triangles: Vec<CollisionTriangle>,
    zero_4: i32,
    bounding_box: Cube,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct CollisionTriangle {
    face_index: i32,
    padding: i8,    //padding, ie. 0
    vertex_1: i32,
    vertex_2: i32,
    vertex_3: i32,

    edge_1_vertex_1: i32,
    edge_1_vertex_2: i32,
    face_index_1: i32,         //the exact same face index as above
    zero_1: i32,               //0
    across_face_index_1: i32,  //the id of the face that lies across this edge (-1 if no such face)

    edge_2_vertex_1: i32,
    edge_2_vertex_2: i32,
    face_index_2: i32,
    zero_2: i32,
    across_face_index_2: i32,

    edge_3_vertex_1: i32,
    edge_3_vertex_2: i32,
    face_index_3: i32,
    zero_3: i32,
    across_face_index_3: i32,

    zero_4: i32,                 //one last 0 for good measure
}

//---------------------------------------------------------------------------//
//                           Implementation of Cs2Collision
//---------------------------------------------------------------------------//

impl Decodeable for Cs2Collision {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.magic_number = data.read_u32()?;
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

impl Encodeable for Cs2Collision {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.magic_number)?;
        buffer.write_u32(self.version)?;

        match self.version {
            21 => self.write_v21(buffer)?,
            20 => self.write_v20(buffer)?,
            _ => unimplemented!()
        }


        Ok(())
    }
}
