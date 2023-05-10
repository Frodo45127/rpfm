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
pub struct Transform {
    m00: f32,
    m01: f32,
    m02: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m20: f32,
    m21: f32,
    m22: f32,
    m30: f32,
    m31: f32,
    m32: f32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Decodeable for Transform {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut transform = Self::default();

        transform.m00 = data.read_f32()?;
        transform.m01 = data.read_f32()?;
        transform.m02 = data.read_f32()?;
        transform.m10 = data.read_f32()?;
        transform.m11 = data.read_f32()?;
        transform.m12 = data.read_f32()?;
        transform.m20 = data.read_f32()?;
        transform.m21 = data.read_f32()?;
        transform.m22 = data.read_f32()?;
        transform.m30 = data.read_f32()?;
        transform.m31 = data.read_f32()?;
        transform.m32 = data.read_f32()?;

        Ok(transform)
    }
}

impl Encodeable for Transform {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.m00)?;
        buffer.write_f32(self.m01)?;
        buffer.write_f32(self.m02)?;
        buffer.write_f32(self.m10)?;
        buffer.write_f32(self.m11)?;
        buffer.write_f32(self.m12)?;
        buffer.write_f32(self.m20)?;
        buffer.write_f32(self.m21)?;
        buffer.write_f32(self.m22)?;
        buffer.write_f32(self.m30)?;
        buffer.write_f32(self.m31)?;
        buffer.write_f32(self.m32)?;

        Ok(())
    }
}
