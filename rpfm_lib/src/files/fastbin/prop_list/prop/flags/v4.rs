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
//                           Implementation of Text
//---------------------------------------------------------------------------//

impl Flags {

    pub(crate) fn read_v4<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.allow_in_outfield = data.read_bool()?;
        self.clamp_to_water_surface = data.read_bool()?;
        self.spring = data.read_bool()?;
        self.summer = data.read_bool()?;
        self.autumn = data.read_bool()?;
        self.winter = data.read_bool()?;
        self.visible_in_tactical_view = data.read_bool()?;
        self.visible_in_tactical_view_only = data.read_bool()?;

        Ok(())
    }
}

