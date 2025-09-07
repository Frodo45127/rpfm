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
//                            Implementation
//---------------------------------------------------------------------------//

impl Material {
    pub fn read_rs_river<R: ReadBytes>(_data: &mut R) -> Result<Self> {
        Ok(Self::default())
    }
    pub fn write_rs_river<W: WriteBytes>(&self, _data: &mut W) -> Result<()> {
        Ok(())
    }
}
