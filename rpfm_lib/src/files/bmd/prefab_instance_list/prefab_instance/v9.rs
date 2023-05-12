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
//                    Implementation of PrefabInstance
//---------------------------------------------------------------------------//

impl PrefabInstance {

    pub(crate) fn read_v9<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key = data.read_sized_string_u8()?;
        self.transform = Transform4x4::decode(data, extra_data)?;

        for _ in 0..data.read_u32()? {
            self.property_overrides.push(PropertyOverride::decode(data, extra_data)?);
        }

        self.campaign_type_mask = data.read_u64()?;
        self.campaign_region_key = data.read_sized_string_u8()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.uid = data.read_u64()?;

        Ok(())
    }

    pub(crate) fn write_v9<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.key)?;

        self.transform.encode(buffer, extra_data)?;

        buffer.write_u32(self.property_overrides.len() as u32)?;
        for property_override in &mut self.property_overrides {
            property_override.encode(buffer, extra_data)?;
        }

        buffer.write_u64(self.campaign_type_mask)?;
        buffer.write_sized_string_u8(&self.campaign_region_key)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.uid)?;

        Ok(())
    }
}
