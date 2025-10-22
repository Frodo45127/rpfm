//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write RigidModels.
//!
//! Most of the code in this module is based on research done by Victimized, Phazer and Ole.

use getset::*;
use nalgebra::Vector3;
use serde_derive::{Serialize, Deserialize};

use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::materials::{Material, MaterialType};
use self::vertices::Vertex;

/// Signature/Magic Numbers/Whatever of a RigidModel.
const SIGNATURE: &[u8; 4] = b"RMV2";

/// Extension used by RigidModels.
pub const EXTENSION: &str = ".rigid_model_v2";

// Constants for padded sized strings.
//const PADDED_SIZE_10: usize = 10;
//const PADDED_SIZE_12: usize = 12;
const PADDED_SIZE_32: usize = 32;
const PADDED_SIZE_64: usize = 64;
const PADDED_SIZE_128: usize = 128;
const PADDED_SIZE_256: usize = 256;

const HEADER_LENGTH: u32 = 140;

pub mod materials;
mod versions;
pub mod vertices;

#[cfg(test)] mod test_rigidmodel;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains a RigidModel decoded in memory.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RigidModel {
    version: u32,
    uk_1: u16,
    skeleton_id: String,
    lods: Vec<Lod>,
}

/// This struct contains a full lod decoded to memory.
#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Lod {
    visibility_distance: f32,
    authored_lod_number: u32,
    quality_level: u32,

    mesh_blocks: Vec<MeshBlock>,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct MeshBlock {
    mesh: Mesh,
    material: Material,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Mesh {
    name: String,

    material_type: MaterialType,
    shader_params: ShaderParams,

    min_bb: Vector3<f32>,
    max_bb: Vector3<f32>,

    lighting_constants: String,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

#[derive(Clone, Debug, Default, PartialEq, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ShaderParams {
    data: Vec<u8>,
    //name: String,
    //uk_1: Vec<u8>,
    //uk_2: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for RigidModel {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature_bytes = data.read_slice(4, false)?;
        if signature_bytes.as_slice() != SIGNATURE {
            return Err(RLibError::DecodingRigidModelUnsupportedSignature(signature_bytes));
        }

        let mut rigid = Self::default();
        rigid.version = data.read_u32()?;

        match rigid.version {
            8 => rigid.read_v8(data)?,
            7 => rigid.read_v7(data)?,
            6 => rigid.read_v6(data)?,
            _ => Err(RLibError::DecodingRigidModelUnsupportedVersion(rigid.version))?,
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(rigid)
    }
}

impl Encodeable for RigidModel {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u32(self.version)?;

        match self.version {
            8 => self.write_v8(buffer)?,
            7 => self.write_v7(buffer)?,
            6 => self.write_v6(buffer)?,
            _ => Err(RLibError::DecodingRigidModelUnsupportedVersion(self.version))?,
        }

        Ok(())
    }
}
