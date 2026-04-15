//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;

use super::*;
use super::versions::v1;

impl GroupFormations {

    pub(crate) fn decode_sho_2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        //GroupFormation
        for _ in 0..data.read_u32()? {
            let mut formation = GroupFormation::default();
            formation.name = data.read_sized_string_u16()?;
            formation.ai_priority = data.read_f32()?;
            formation.ai_purpose = AIPurpose::V1(v1::AIPurposeFlags::from_bits_truncate(data.read_u32()?));

            // MinUnitCategoryPercentage
            for _ in 0..data.read_u32()? {
                let mut min_unit_category_percentage = MinUnitCategoryPercentage::default();
                min_unit_category_percentage.category = UnitCategory::try_from(data.read_u32()?)?;
                min_unit_category_percentage.percentage = data.read_u32()?;
                formation.min_unit_category_percentage.push(min_unit_category_percentage);
            }

            // AiSupportedFaction
            for _ in 0..data.read_u32()? {
                formation.ai_supported_factions.push(data.read_sized_string_u16()?);
            }

            // GroupFormationBlock
            for _ in 0..data.read_u32()? {
                let mut block = GroupFormationBlock::default();
                block.block_id = data.read_u32()?;

                let block_type = data.read_u32()?;
                match block_type {

                    // ContainerAbsolute
                    0 => {
                        let mut container = ContainerAbsolute::default();
                        container.block_priority = data.read_f32()?;
                        container.entity_arrangement = EntityArrangement::try_from(data.read_u32()?)?;
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = Entity::V1(v1::EntityType::try_from(data.read_u32()?)?);
                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerAbsolute(container);
                    },

                    // ContainerRelative
                    1 => {
                        let mut container = ContainerRelative::default();
                        container.block_priority = data.read_f32()?;
                        container.relative_block_id = data.read_u32()?;
                        container.entity_arrangement = EntityArrangement::try_from(data.read_u32()?)?;
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = Entity::V1(v1::EntityType::try_from(data.read_u32()?)?);
                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerRelative(container);
                    },

                    // Spanning
                    3 => {
                        let mut container = Spanning::default();
                        for _ in 0..data.read_u32()? {
                            container.spanned_block_ids.push(data.read_u32()?);
                        }
                        block.block = Block::Spanning(container);
                    },
                    _ => todo!("unknown block type {}.", block_type),
                }

                formation.group_formation_blocks.push(block);
            }

            self.formations.push(formation);
        }

        Ok(())
    }

    pub(crate) fn encode_sho_2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.formations.len() as u32)?;
        for formation in self.formations() {
            buffer.write_sized_string_u16(formation.name())?;
            buffer.write_f32(formation.ai_priority)?;

            if let AIPurpose::V1(data) = &formation.ai_purpose {
                buffer.write_u32(data.bits())?;
            }

            buffer.write_u32(formation.min_unit_category_percentage.len() as u32)?;
            for mucp in formation.min_unit_category_percentage() {
                buffer.write_u32(mucp.category.into())?;
                buffer.write_u32(mucp.percentage)?;
            }

            buffer.write_u32(formation.ai_supported_factions.len() as u32)?;
            for s in formation.ai_supported_factions() {
                buffer.write_sized_string_u16(s)?;
            }

            buffer.write_u32(formation.group_formation_blocks.len() as u32)?;
            for block in formation.group_formation_blocks() {
                buffer.write_u32(block.block_id)?;

                match block.block {
                    Block::ContainerAbsolute(ref b) => {
                        buffer.write_u32(0)?;
                        buffer.write_f32(b.block_priority)?;
                        buffer.write_u32(b.entity_arrangement.into())?;
                        buffer.write_f32(b.inter_entity_spacing)?;
                        buffer.write_f32(b.crescent_y_offset)?;
                        buffer.write_f32(b.position_x)?;
                        buffer.write_f32(b.position_y)?;
                        buffer.write_i32(b.minimum_entity_threshold)?;
                        buffer.write_i32(b.maximum_entity_threshold)?;

                        buffer.write_u32(b.entity_preferences.len() as u32)?;
                        for ep in b.entity_preferences() {
                            buffer.write_f32(ep.priority)?;
                            if let Entity::V1(data) = &ep.entity {
                                buffer.write_u32((*data).into())?;
                            }
                        }
                    },

                    Block::ContainerRelative(ref b) => {
                        buffer.write_u32(1)?;
                        buffer.write_f32(b.block_priority)?;
                        buffer.write_u32(b.relative_block_id)?;
                        buffer.write_u32(b.entity_arrangement.into())?;
                        buffer.write_f32(b.inter_entity_spacing)?;
                        buffer.write_f32(b.crescent_y_offset)?;
                        buffer.write_f32(b.position_x)?;
                        buffer.write_f32(b.position_y)?;
                        buffer.write_i32(b.minimum_entity_threshold)?;
                        buffer.write_i32(b.maximum_entity_threshold)?;

                        buffer.write_u32(b.entity_preferences.len() as u32)?;
                        for ep in b.entity_preferences() {
                            buffer.write_f32(ep.priority)?;
                            if let Entity::V1(data) = &ep.entity {
                                buffer.write_u32((*data).into())?;
                            }
                        }
                    },

                    Block::Spanning(ref b) => {
                        buffer.write_u32(3)?;
                        buffer.write_u32(b.spanned_block_ids.len() as u32)?;
                        for id in b.spanned_block_ids() {
                            buffer.write_u32(*id)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
