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
//                           Implementation of TileDatabase
//---------------------------------------------------------------------------//

impl TileDatabase {

    pub(crate) fn read_v1<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        self.render_params = RenderParams::decode(data, extra_data)?;
        self.conversion_params = ConversionParams::decode(data, extra_data)?;

        for _ in 0..data.read_u32()? {
            self.climates.push(Climate::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            self.tile_sets.push(TileSet::decode(data, extra_data)?);
        }

        for _ in 0..data.read_u32()? {
            self.tiles.push(Tile::decode(data, extra_data)?);
        }

        Ok(())
    }

    pub(crate) fn write_v1<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        self.render_params.encode(buffer, extra_data)?;
        self.conversion_params.encode(buffer, extra_data)?;

        buffer.write_u32(self.climates.len() as u32)?;
        for climate in self.climates_mut() {
            climate.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.tile_sets.len() as u32)?;
        for tile_set in self.tile_sets_mut() {
            tile_set.encode(buffer, extra_data)?;
        }

        buffer.write_u32(self.tiles.len() as u32)?;
        for tile in self.tiles_mut() {
            tile.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
