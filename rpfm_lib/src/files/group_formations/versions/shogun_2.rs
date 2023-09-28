//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

//---------------------------------------------------------------------------//
//                          Implementation of GroupFormations
//---------------------------------------------------------------------------//

impl GroupFormations {

    pub(crate) fn decode_sho_2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        //GroupFormation
        for _ in 0..data.read_u32()? {
            let mut formation = GroupFormation::default();
            formation.name = data.read_sized_string_u16()?;
            formation.ai_priority = data.read_f32()?;
            formation.ai_purpose = AIPurpose::from_bits_truncate(data.read_u32()?);

            // MinUnitCategoryPercentage is one of these
            for _ in 0..data.read_u32()? {
                let mut min_unit_category_percentage = MinUnitCategoryPercentage::default();

                min_unit_category_percentage.category = UnitCategory::from(data.read_u32()?);
                min_unit_category_percentage.percentage = data.read_u32()?;

                formation.min_unit_category_percentage.push(min_unit_category_percentage);
            }

            // AiSupportedFaction is the other one.
            for _ in 0..data.read_u32()? {
                formation.ai_supported_factions.push(data.read_sized_string_u16()?);
            }

            // GroupFormationBlock
            for _ in 0..data.read_u32()? {
                let mut block = GroupFormationBlock::default();
                block.block_id = data.read_u32()?;

                // Possible enum: 0 absolute, 1 relative, 3 spanning
                let block_type = data.read_u32()?;
                match block_type {

                    // ContainerAbsolute
                    0 => {
                        let mut container = ContainerAbsolute::default();
                        container.block_priority = data.read_f32()?;
                        container.entity_arrangement =  EntityArrangement::from(data.read_u32()?);
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        // EntityPreference
                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = Entity::from(data.read_u32()?);

                            container.entity_preferences.push(entity_pref);
                        }

                        block.block = Block::ContainerAbsolute(container);
                    },

                    // ContainerRelative
                    1 => {
                        let mut container = ContainerRelative::default();
                        container.block_priority = data.read_f32()?;
                        container.relative_block_id = data.read_u32()?;
                        container.entity_arrangement = EntityArrangement::from(data.read_u32()?);
                        container.inter_entity_spacing = data.read_f32()?;
                        container.crescent_y_offset = data.read_f32()?;
                        container.position_x = data.read_f32()?;
                        container.position_y = data.read_f32()?;
                        container.minimum_entity_threshold = data.read_i32()?;
                        container.maximum_entity_threshold = data.read_i32()?;

                        // EntityPreference
                        for _ in 0..data.read_u32()? {
                            let mut entity_pref = EntityPreference::default();
                            entity_pref.priority = data.read_f32()?;
                            entity_pref.entity = Entity::from(data.read_u32()?);

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
            buffer.write_u32(formation.ai_purpose.bits())?;

            buffer.write_u32(formation.min_unit_category_percentage.len() as u32)?;
            for min_unit_category_percentage in formation.min_unit_category_percentage() {
                buffer.write_u32(min_unit_category_percentage.category.clone().into())?;
                buffer.write_u32(min_unit_category_percentage.percentage)?;
            }

            buffer.write_u32(formation.ai_supported_factions.len() as u32)?;
            for ai_supported_faction in formation.ai_supported_factions() {
                buffer.write_sized_string_u16(ai_supported_faction)?;
            }

            buffer.write_u32(formation.group_formation_blocks.len() as u32)?;
            for block in formation.group_formation_blocks() {
                buffer.write_u32(block.block_id)?;

                match block.block {
                    Block::ContainerAbsolute(ref block) => {
                        buffer.write_u32(0)?;

                        buffer.write_f32(block.block_priority)?;
                        buffer.write_u32(block.entity_arrangement.clone().into())?;
                        buffer.write_f32(block.inter_entity_spacing)?;
                        buffer.write_f32(block.crescent_y_offset)?;
                        buffer.write_f32(block.position_x)?;
                        buffer.write_f32(block.position_y)?;
                        buffer.write_i32(block.minimum_entity_threshold)?;
                        buffer.write_i32(block.maximum_entity_threshold)?;

                        buffer.write_u32(block.entity_preferences.len() as u32)?;
                        for ent_pref in block.entity_preferences() {
                            buffer.write_f32(ent_pref.priority)?;
                            buffer.write_u32(ent_pref.entity.clone().into())?;
                        }
                    },

                    Block::ContainerRelative(ref block) => {
                        buffer.write_u32(1)?;

                        buffer.write_f32(block.block_priority)?;
                        buffer.write_u32(block.relative_block_id)?;
                        buffer.write_u32(block.entity_arrangement.clone().into())?;
                        buffer.write_f32(block.inter_entity_spacing)?;
                        buffer.write_f32(block.crescent_y_offset)?;
                        buffer.write_f32(block.position_x)?;
                        buffer.write_f32(block.position_y)?;
                        buffer.write_i32(block.minimum_entity_threshold)?;
                        buffer.write_i32(block.maximum_entity_threshold)?;

                        buffer.write_u32(block.entity_preferences.len() as u32)?;
                        for ent_pref in block.entity_preferences() {
                            buffer.write_f32(ent_pref.priority)?;
                            buffer.write_u32(ent_pref.entity.clone().into())?;
                        }
                    },

                    Block::Spanning(ref block) => {
                        buffer.write_u32(3)?;
                        buffer.write_u32(block.spanned_block_ids.len() as u32)?;
                        for id in block.spanned_block_ids() {
                            buffer.write_u32(*id)?;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}
