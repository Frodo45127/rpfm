//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Matched Combat files, v1, for Three Kingdoms.
//!
//! For internal use only.

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::matched_combat::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl MatchedCombat {

    pub fn read_v1_3k<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let count = data.read_u32()?;

        // Entries
        for _ in 0..count {
            let mut matched_entry = MatchedEntry::default();
            matched_entry.id = data.read_sized_string_u8()?;

            // These ones do not have participants, so we put all the bundles in a dummy Participant for compatibility.
            let mut participant = Participant::default();

            let entity_bundle_count = data.read_u32()?;
            for _ in 0..entity_bundle_count {
                let mut bundle = EntityBundle::default();

                // Entities in bundle
                let entity_count = data.read_u32()?;
                for _ in 0..entity_count {
                    let mut entity = Entity::default();

                    let filter_count = data.read_u32()?;

                    for _ in 0..filter_count {
                        entity.filters.push(Filter {
                            filter_type: data.read_u32()?,
                            value: data.read_sized_string_u8()?,
                            equals: data.read_bool()?,
                            or: data.read_bool()?,
                        });
                    }

                    entity.animation_filename = data.read_sized_string_u8()?;

                    let metadata_filenames_count = data.read_u32()?;
                    for _ in 0..metadata_filenames_count {
                        entity.metadata_filenames.push(data.read_sized_string_u8()?);
                    }

                    entity.blend_in_time = data.read_f32()?;
                    entity.equipment_display = data.read_u32()?;
                    entity.uk = data.read_u32()?;   // No idea. Always 0 and doesn't seem to match any value in the mirror txt in 3k.

                    bundle.entities.push(entity);
                }

                participant.entity_info.push(bundle);
            }

            matched_entry.participants.push(participant);
            self.entries.push(matched_entry);
        }

        Ok(())
    }

    pub fn write_v1_3k<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_sized_string_u8(&entry.id)?;

            // There should only be one, but loop just in case.
            for participant in &entry.participants {
                buffer.write_u32(participant.entity_info.len() as u32)?;

                for bundle in &participant.entity_info {
                    buffer.write_u32(bundle.entities.len() as u32)?;

                    for entity in &bundle.entities {
                        buffer.write_u32(entity.filters.len() as u32)?;

                        for filter in &entity.filters {
                            buffer.write_u32(filter.filter_type)?;
                            buffer.write_sized_string_u8(&filter.value)?;
                            buffer.write_bool(filter.equals)?;
                            buffer.write_bool(filter.or)?;
                        }

                        buffer.write_sized_string_u8(&entity.animation_filename)?;
                        buffer.write_u32(entity.metadata_filenames.len() as u32)?;

                        for metadata_filename in &entity.metadata_filenames {
                            buffer.write_sized_string_u8(metadata_filename)?;
                        }

                        buffer.write_f32(entity.blend_in_time)?;
                        buffer.write_u32(entity.equipment_display)?;
                        buffer.write_u32(entity.uk)?;
                    }
                }
            }
        }

        Ok(())
    }
}

