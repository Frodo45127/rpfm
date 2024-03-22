//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::ReadBytes;
use crate::error::Result;
use crate::files::Decodeable;

use super::*;

//---------------------------------------------------------------------------//
//                           Implementation of BmdVegetation
//---------------------------------------------------------------------------//

impl BmdVegetation {

    pub(crate) fn read_v2<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.tree_list = TreeList::decode(data, extra_data)?;
        self.grass_list = GrassList::decode(data, extra_data)?;

        Ok(())
    }

    pub(crate) fn write_v2<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.tree_list.encode(buffer, extra_data)?;
        self.grass_list.encode(buffer, extra_data)?;

        Ok(())
    }
}
