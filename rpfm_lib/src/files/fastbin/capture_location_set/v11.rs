//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;

use super::*;

//---------------------------------------------------------------------------//
//                    Implementation of CaptureLocationSet
//---------------------------------------------------------------------------//

impl CaptureLocationSet {

    pub(crate) fn read_v11<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            let mut list = vec![];
            for _ in 0..data.read_u32()? {
                list.push(CaptureLocation::decode(data, extra_data)?);
            }
            self.capture_location_sets.push(list);
        }

        Ok(())
    }


    pub(crate) fn write_v11<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.capture_location_sets.len() as u32)?;
        for list in &mut self.capture_location_sets {
            buffer.write_u32(list.len() as u32)?;
            for capture_location in list {
                capture_location.encode(buffer, extra_data)?;
            }
        }

        Ok(())
    }
}
