//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with RigidModel PackedFiles.

This is really a dummy module, as all the logic for this is done in the view through Phazer's lib.
!*/

use getset::*;
use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};

use crate::schema::Schema;
use crate::files::{Decodeable, Encodeable, FileType};

/// This represents the value that every RigidModel PackedFile has in their 0-4 bytes. A.k.a it's signature or preamble.
#[allow(dead_code)]
const PACKED_FILE_TYPE: &str = "RMV2";

/// Extension used by RigidModel PackedFiles.
pub const EXTENSION: &str = ".rigid_model_v2";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct contains a RigidModel decoded in memory.
#[derive(Clone, Debug,PartialEq, Getters, Setters)]
pub struct RigidModel {
    data: Vec<u8>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for RigidModel {

    fn file_type(&self) -> FileType {
        FileType::RigidModel
    }

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {
        let len = data.len()?;
        let data = data.read_slice(len as usize, false)?;
        Ok(Self {
            data,
        })
    }
}

impl Encodeable for RigidModel {
    fn encode<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_all(&self.data).map_err(From::from)
    }
}
