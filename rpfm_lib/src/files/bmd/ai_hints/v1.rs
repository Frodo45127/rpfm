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
//                 Implementation of AIHints
//---------------------------------------------------------------------------//

impl AIHints {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.separators = Separators::decode(data, extra_data)?;
        self.directed_points = DirectedPoints::decode(data, extra_data)?;
        self.polylines = Polylines::decode(data, extra_data)?;
        self.polylines_list = PolylinesList::decode(data, extra_data)?;

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.separators.encode(buffer, extra_data)?;
        self.directed_points.encode(buffer, extra_data)?;
        self.polylines.encode(buffer, extra_data)?;
        self.polylines_list.encode(buffer, extra_data)?;

        Ok(())
    }
}
