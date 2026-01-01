//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
//                    Implementation of SoundShape
//---------------------------------------------------------------------------//

impl SoundShape {

    pub(crate) fn read_v10<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key = data.read_sized_string_u8()?;
        self.rtype = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.points.push(Point3d::decode(data, extra_data)?);
        }

        self.inner_radius = data.read_f32()?;
        self.outer_radius = data.read_f32()?;

        self.inner_cube = Cube::decode(data, extra_data)?;
        self.outer_cube = Cube::decode(data, extra_data)?;

        for _ in 0..data.read_u32()? {
            self.river_nodes.push(RiverNode::decode(data, extra_data)?);
        }

        self.clamp_to_surface = data.read_bool()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.campaign_type_mask = data.read_u64()?;
        self.pdlc_mask = data.read_u64()?;

        self.direction = Point3d::decode(data, extra_data)?;

        self.up = Point3d::decode(data, extra_data)?;

        self.scope = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v10<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.key)?;
        buffer.write_sized_string_u8(&self.rtype)?;

        buffer.write_u32(self.points.len() as u32)?;
        for point in &mut self.points {
            point.encode(buffer, extra_data)?;
        }

        buffer.write_f32(self.inner_radius)?;
        buffer.write_f32(self.outer_radius)?;

        self.inner_cube.encode(buffer, extra_data)?;
        self.outer_cube.encode(buffer, extra_data)?;

        buffer.write_u32(self.river_nodes.len() as u32)?;
        for river_node in &mut self.river_nodes {
            river_node.encode(buffer, extra_data)?;
        }

        buffer.write_bool(self.clamp_to_surface)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.campaign_type_mask)?;
        buffer.write_u64(self.pdlc_mask)?;

        self.direction.encode(buffer, extra_data)?;
        self.up.encode(buffer, extra_data)?;

        buffer.write_sized_string_u8(&self.scope)?;

        Ok(())
    }
}
