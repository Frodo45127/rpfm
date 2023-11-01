//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write sound_events files from Shogun 2, Napoleon and Empire.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

pub const PATH: &str = "sounds_packed/sound_events";

#[cfg(test)] mod sound_events_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundEvents {
    categories: Vec<Category>,
    event_data: Vec<EventData>,
    event_records: Vec<EventRecord>,
    ambience_records: Vec<AmbienceRecord>,
    extra_data: Vec<ExtraData>,

}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Category {
    name: String,
    float: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventData {
    uk_1: f32,
    uk_2: f32,
    uk_3: f32,
    uk_4: f32,
    uk_5: f32,
    uk_6: f32,
    uk_7: f32,
    uk_8: f32,
    uk_9: f32,
    uk_10: f32,
    uk_11: f32,
    uk_12: f32,
    uk_13: f32,
    uk_14: f32,
    uk_15: f32,
    uk_16: f32,
    uk_17: f32,
    uk_18: f32,
    uk_19: f32,
    uk_20: f32,
    uk_21: f32,
    uk_22: f32,
    uk_23: f32,
    uk_24: f32,
    uk_25: f32,
    uk_26: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct EventRecord {
    category: u32,
    name: Option<String>,
    uk_1: u32,
    sounds: Vec<String>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AmbienceRecord {
    uk_1: u32,
    uk_2: u32,
    uk_3: f32,
    uk_4: f32,
    uk_5: f32,
    uk_6: f32,
    uk_7: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ExtraData {
    uk_1: i32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for SoundEvents {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut sound_events = Self::default();

        // Category records.
        for _ in 0..data.read_u32()? {
            sound_events.categories_mut().push(Category {
                name: data.read_sized_string_u16()?,
                float: data.read_f32()?
            });
        }

        // Event data records?
        for _ in 0..data.read_u32()? {
            sound_events.event_data_mut().push(EventData {
                uk_1: data.read_f32()?,
                uk_2: data.read_f32()?,
                uk_3: data.read_f32()?,
                uk_4: data.read_f32()?,
                uk_5: data.read_f32()?,
                uk_6: data.read_f32()?,
                uk_7: data.read_f32()?,
                uk_8: data.read_f32()?,
                uk_9: data.read_f32()?,
                uk_10: data.read_f32()?,
                uk_11: data.read_f32()?,
                uk_12: data.read_f32()?,
                uk_13: data.read_f32()?,
                uk_14: data.read_f32()?,
                uk_15: data.read_f32()?,
                uk_16: data.read_f32()?,
                uk_17: data.read_f32()?,
                uk_18: data.read_f32()?,
                uk_19: data.read_f32()?,
                uk_20: data.read_f32()?,
                uk_21: data.read_f32()?,
                uk_22: data.read_f32()?,
                uk_23: data.read_f32()?,
                uk_24: data.read_f32()?,
                uk_25: data.read_f32()?,
                uk_26: data.read_f32()?,
            });
        }

        // Event Records
        for _ in 0..data.read_u32()? {
            let mut event = EventRecord::default();
            *event.category_mut() = data.read_u32()?;

            if *event.category() == 1 || *event.category() == 2 || *event.category() == 27 {
                *event.name_mut() = Some(data.read_sized_string_u16()?);
            }

            *event.uk_1_mut() = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                event.sounds_mut().push(data.read_sized_string_u16()?);
            }

            sound_events.event_records_mut().push(event);
        }

        // Ambience records?
        for _ in 0..data.read_u32()? {
            sound_events.ambience_records_mut().push(AmbienceRecord {
                uk_1: data.read_u32()?,
                uk_2: data.read_u32()?,
                uk_3: data.read_f32()?,
                uk_4: data.read_f32()?,
                uk_5: data.read_f32()?,
                uk_6: data.read_f32()?,
                uk_7: data.read_f32()?
            });
        }

        // Data?
        for _ in 0..data.read_u32()? {
            sound_events.extra_data_mut().push(ExtraData {
                uk_1: data.read_i32()?
            });
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(sound_events)
    }
}

impl Encodeable for SoundEvents {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.categories.len() as u32)?;
        for category in self.categories() {
            buffer.write_sized_string_u16(&category.name)?;
            buffer.write_f32(category.float)?;
        }

        buffer.write_u32(self.event_data.len() as u32)?;
        for data in self.event_data() {
            buffer.write_f32(data.uk_1)?;
            buffer.write_f32(data.uk_2)?;
            buffer.write_f32(data.uk_3)?;
            buffer.write_f32(data.uk_4)?;
            buffer.write_f32(data.uk_5)?;
            buffer.write_f32(data.uk_6)?;
            buffer.write_f32(data.uk_7)?;
            buffer.write_f32(data.uk_8)?;
            buffer.write_f32(data.uk_9)?;
            buffer.write_f32(data.uk_10)?;
            buffer.write_f32(data.uk_11)?;
            buffer.write_f32(data.uk_12)?;
            buffer.write_f32(data.uk_13)?;
            buffer.write_f32(data.uk_14)?;
            buffer.write_f32(data.uk_15)?;
            buffer.write_f32(data.uk_16)?;
            buffer.write_f32(data.uk_17)?;
            buffer.write_f32(data.uk_18)?;
            buffer.write_f32(data.uk_19)?;
            buffer.write_f32(data.uk_20)?;
            buffer.write_f32(data.uk_21)?;
            buffer.write_f32(data.uk_22)?;
            buffer.write_f32(data.uk_23)?;
            buffer.write_f32(data.uk_24)?;
            buffer.write_f32(data.uk_25)?;
            buffer.write_f32(data.uk_26)?;
        }

        buffer.write_u32(self.event_records.len() as u32)?;
        for event_record in self.event_records() {
            buffer.write_u32(event_record.category)?;

            if *event_record.category() == 1 || *event_record.category() == 2 || *event_record.category() == 27 {
                if let Some(name) = event_record.name() {
                    buffer.write_sized_string_u16(name)?;
                }
            }

            buffer.write_u32(event_record.uk_1)?;
            buffer.write_u32(event_record.sounds.len() as u32)?;
            for sound in event_record.sounds() {
                buffer.write_sized_string_u16(sound)?;
            }
        }

        buffer.write_u32(self.ambience_records.len() as u32)?;
        for ambience_record in self.ambience_records() {
            buffer.write_u32(ambience_record.uk_1)?;
            buffer.write_u32(ambience_record.uk_2)?;
            buffer.write_f32(ambience_record.uk_3)?;
            buffer.write_f32(ambience_record.uk_4)?;
            buffer.write_f32(ambience_record.uk_5)?;
            buffer.write_f32(ambience_record.uk_6)?;
            buffer.write_f32(ambience_record.uk_7)?;
        }

        buffer.write_u32(self.extra_data.len() as u32)?;
        for extra_data in self.extra_data() {
            buffer.write_i32(extra_data.uk_1)?;
        }

        Ok(())
    }
}
