//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary Matched Combat files, v1, for Warhammer 3.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::matched_combat::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl MatchedCombat {

    pub fn read_v1_wh3<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let count = data.read_u32()?;
        for _ in 0..count {
            let mut matched_entry = MatchedEntry::default();
            let participants_count = data.read_u32()?;

            // Participants
            for _ in 0..participants_count {
                let mut participant = Participant::default();
                participant.team = data.read_u32()?;
                participant.state.start = StateParticipant::try_from(data.read_u32()?)?;
                participant.state.end = StateParticipant::try_from(data.read_u32()?)?;

                participant.uk3 = data.read_u32()?;     // No idea.
                participant.uk4 = data.read_u32()?;     // No idea.

                let mut entity = Entity::default();
                entity.animation_filename = data.read_sized_string_u8()?;
                entity.mount_filename = data.read_sized_string_u8()?;

                let mut bundle = EntityBundle::default();
                bundle.entities.push(entity);

                participant.entity_info.push(bundle);
                matched_entry.participants.push(participant);
            }

            self.entries.push(matched_entry);
        }

        Ok(())
    }

    pub fn write_v1_wh3<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.entries.len() as u32)?;
        for entry in &self.entries {
            buffer.write_u32(entry.participants.len() as u32)?;
            for participant in &entry.participants {
                buffer.write_u32(participant.team)?;
                buffer.write_u32(participant.state.start as u32)?;
                buffer.write_u32(participant.state.end as u32)?;
                buffer.write_u32(participant.uk3)?;
                buffer.write_u32(participant.uk4)?;
                buffer.write_sized_string_u8(&participant.entity_info[0].entities[0].animation_filename)?;
                buffer.write_sized_string_u8(&participant.entity_info[0].entities[0].mount_filename)?;
            }
        }

        Ok(())
    }
}

