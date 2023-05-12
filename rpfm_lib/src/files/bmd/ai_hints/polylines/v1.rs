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
//                 Implementation of Polylines
//---------------------------------------------------------------------------//

impl Polylines {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.hint_polylines.push(HintPolyline::decode(data, extra_data)?);
        }

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.hint_polylines.len() as u32)?;

        for hint_polyline in &mut self.hint_polylines {
            hint_polyline.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
