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

use crate::error::Result;

use crate::schema::Schema;
use getset::*;
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

    fn decode(packed_file_data: &[u8], _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {
        Ok(Self {
            data: packed_file_data.to_vec(),
        })
    }
}

impl Encodeable for RigidModel {
    fn encode(&self) -> Vec<u8> {
        self.data.to_vec()
    }
}
