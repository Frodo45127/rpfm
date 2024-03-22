//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl BKHD {

    pub(crate) fn read_v122<R: ReadBytes>(data: &mut R, version: u32, section_size: usize) -> Result<Self> {
        let id = data.read_u32()? as u64;
        let language = Language::try_from(data.read_u32()?)?;
        let feedback_in_bank = data.read_u32()?;
        let project_id = data.read_u32()?;
        let padding = data.read_slice(section_size - NON_PADDED_SIZE, false)?;

        Ok(BKHD {
            version,
            id,
            language,
            feedback_in_bank,
            alignment: 0,
            device_allocated: 0,
            project_id,
            padding,
        })
    }

    pub(crate) fn write_v122<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        let mut encoded_data = vec![];
        encoded_data.write_u32(self.version | 0x80_00_00_00)?;
        encoded_data.write_u32(self.id as u32)?;
        encoded_data.write_u32(self.language as u32)?;
        encoded_data.write_u32(self.feedback_in_bank)?;
        encoded_data.write_u32(self.project_id)?;
        encoded_data.write_all(&self.padding)?;

        buffer.write_string_u8(SIGNATURE_BKHD)?;
        buffer.write_u32(encoded_data.len() as u32)?;
        buffer.write_all(&encoded_data).map_err(From::from)
    }
}
