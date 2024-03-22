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

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::sound_bank::*;

use self::object::*;

mod object;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct HIRC {
    objects: Vec<Object>,
}

//---------------------------------------------------------------------------//
//                        Implementation of SoundBank
//---------------------------------------------------------------------------//

impl HIRC {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let mut objects = vec![];

        for _ in 0..data.read_u32()? {
            objects.push(Object::read(data, version)?);
        }

        Ok(HIRC {
            objects
        })
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        let mut temp = vec![];
        for object in &self.objects {
            object.write(&mut temp, version)?;
        }

        buffer.write_string_u8(SIGNATURE_HIRC)?;
        buffer.write_u32(temp.len() as u32 + 4)?;   // +4 because we need to also count the amount of items.
        buffer.write_u32(self.objects.len() as u32)?;
        buffer.write_all(&temp)?;

        Ok(())
    }
}
