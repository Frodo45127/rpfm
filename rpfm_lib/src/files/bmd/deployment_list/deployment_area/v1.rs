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
//                    Implementation of DeploymentArea
//---------------------------------------------------------------------------//

impl DeploymentArea {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.category = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.deployment_zones.push(DeploymentZone::decode(data, extra_data)?);
        }

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.category)?;
        buffer.write_u32(self.deployment_zones.len() as u32)?;

        for zone in &mut self.deployment_zones {
            zone.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
