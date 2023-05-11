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
//                    Implementation of SoundShape
//---------------------------------------------------------------------------//

impl SoundShape {

    pub(crate) fn read_v10<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.key = data.read_sized_string_u8()?;
        self.rtype = data.read_sized_string_u8()?;

        for _ in 0..data.read_u32()? {
            self.points.push(Point {
                x: data.read_f32()?,
                y: data.read_f32()?,
                z: data.read_f32()?,
            });
        }

        self.inner_radius = data.read_f32()?;
        self.outer_radius = data.read_f32()?;

        self.inner_cube = Cube {
            min_x: data.read_f32()?,
            min_y: data.read_f32()?,
            min_z: data.read_f32()?,
            max_x: data.read_f32()?,
            max_y: data.read_f32()?,
            max_z: data.read_f32()?,
        };

        self.outer_cube = Cube {
            min_x: data.read_f32()?,
            min_y: data.read_f32()?,
            min_z: data.read_f32()?,
            max_x: data.read_f32()?,
            max_y: data.read_f32()?,
            max_z: data.read_f32()?,
        };

        for _ in 0..data.read_u32()? {
            self.river_nodes.push(RiverNode::decode(data, extra_data)?);
        }

        self.clamp_to_surface = data.read_bool()?;
        self.height_mode = data.read_sized_string_u8()?;
        self.campaign_type_mask = data.read_u64()?;
        self.pdlc_mask = data.read_u64()?;

        self.direction = Direction {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };

        self.up = Up {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        };

        self.scope = data.read_sized_string_u8()?;

        Ok(())
    }

    pub(crate) fn write_v10<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.key)?;
        buffer.write_sized_string_u8(&self.rtype)?;

        buffer.write_u32(self.points.len() as u32)?;
        for point in &self.points {
            buffer.write_f32(point.x)?;
            buffer.write_f32(point.y)?;
            buffer.write_f32(point.z)?;
        }

        buffer.write_f32(self.inner_radius)?;
        buffer.write_f32(self.outer_radius)?;

        buffer.write_f32(self.inner_cube.min_x)?;
        buffer.write_f32(self.inner_cube.min_y)?;
        buffer.write_f32(self.inner_cube.min_z)?;
        buffer.write_f32(self.inner_cube.max_x)?;
        buffer.write_f32(self.inner_cube.max_y)?;
        buffer.write_f32(self.inner_cube.max_z)?;

        buffer.write_f32(self.outer_cube.min_x)?;
        buffer.write_f32(self.outer_cube.min_y)?;
        buffer.write_f32(self.outer_cube.min_z)?;
        buffer.write_f32(self.outer_cube.max_x)?;
        buffer.write_f32(self.outer_cube.max_y)?;
        buffer.write_f32(self.outer_cube.max_z)?;

        buffer.write_u32(self.river_nodes.len() as u32)?;
        for river_node in &mut self.river_nodes {
            river_node.encode(buffer, extra_data)?;
        }

        buffer.write_bool(self.clamp_to_surface)?;
        buffer.write_sized_string_u8(&self.height_mode)?;
        buffer.write_u64(self.campaign_type_mask)?;
        buffer.write_u64(self.pdlc_mask)?;

        buffer.write_f32(self.direction.x)?;
        buffer.write_f32(self.direction.y)?;
        buffer.write_f32(self.direction.z)?;

        buffer.write_f32(self.up.x)?;
        buffer.write_f32(self.up.y)?;
        buffer.write_f32(self.up.z)?;

        buffer.write_sized_string_u8(&self.scope)?;

        Ok(())
    }
}
