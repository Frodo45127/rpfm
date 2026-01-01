//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

impl SoundEvents {

    pub(crate) fn read_sho2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        self.master_volume = data.read_f32()?;

        // Category records.
        for _ in 0..data.read_u32()? {
            let mut category = Category::default();

            category.name = data.read_sized_string_u16()?;
            category.uk_1 = data.read_f32()?;

            self.categories_mut().push(category);
        }

        for _ in 0..data.read_u32()? {
            let mut uk_1 = Uk1::default();

            uk_1.uk_1 = data.read_i32()?;

            self.uk_1_mut().push(uk_1);
        }

        for _ in 0..data.read_u32()? {
            let mut uk_4 = Uk4::default();

            uk_4.uk_1 = data.read_i32()?;
            uk_4.uk_2 = data.read_i32()?;

            self.uk_4_mut().push(uk_4);
        }

        for _ in 0..data.read_u32()? {
            let mut uk_5 = Uk5::default();

            uk_5.uk_1 = data.read_f32()?;
            uk_5.uk_2 = data.read_f32()?;
            uk_5.uk_3 = data.read_f32()?;
            uk_5.uk_4 = data.read_f32()?;
            uk_5.uk_5 = data.read_f32()?;
            uk_5.uk_6 = data.read_f32()?;
            uk_5.uk_7 = data.read_f32()?;
            uk_5.uk_8 = data.read_f32()?;

            self.uk_5_mut().push(uk_5);
        }

        self.uk_6 = data.read_u32()?;
        self.uk_7 = data.read_u32()?;

        for _ in 0..31 {
            let mut uk_8 = Uk8::default();

            uk_8.uk_1 = data.read_u32()?;

            self.uk_8_mut().push(uk_8);
        }

        // Event data.
        for _ in 0..data.read_u32()? {
            let mut event_data = EventData::default();

            event_data.uk_1 = data.read_f32()?;
            event_data.uk_2 = data.read_f32()?;
            event_data.uk_3 = data.read_f32()?;
            event_data.uk_4 = data.read_f32()?;
            event_data.uk_5 = data.read_f32()?;
            event_data.uk_6 = data.read_f32()?;
            event_data.uk_7 = data.read_f32()?;
            event_data.uk_8 = data.read_f32()?;
            event_data.uk_9 = data.read_f32()?;
            event_data.uk_10 = data.read_f32()?;
            event_data.uk_11 = data.read_f32()?;
            event_data.uk_12 = data.read_f32()?;
            event_data.uk_13 = data.read_f32()?;
            event_data.uk_14 = data.read_f32()?;
            event_data.uk_15 = data.read_f32()?;
            event_data.uk_16 = data.read_f32()?;
            event_data.uk_17 = data.read_f32()?;
            event_data.uk_18 = data.read_f32()?;
            event_data.uk_19 = data.read_f32()?;
            event_data.uk_20 = data.read_f32()?;
            event_data.uk_21 = data.read_f32()?;
            event_data.uk_22 = data.read_f32()?;
            event_data.uk_23 = data.read_f32()?;
            event_data.uk_24 = data.read_f32()?;
            event_data.uk_25 = data.read_f32()?;
            event_data.uk_26 = data.read_f32()?;
            event_data.uk_27 = data.read_f32()?;
            event_data.uk_28 = data.read_f32()?;
            event_data.uk_29 = data.read_f32()?;
            event_data.uk_30 = data.read_f32()?;
            event_data.uk_31 = data.read_f32()?;
            event_data.uk_32 = data.read_f32()?;
            event_data.uk_33 = data.read_f32()?;
            event_data.uk_34 = data.read_f32()?;
            event_data.uk_35 = data.read_f32()?;
            event_data.uk_36 = data.read_f32()?;
            event_data.uk_37 = data.read_f32()?;
            event_data.uk_38 = data.read_f32()?;
            event_data.uk_39 = data.read_f32()?;
            event_data.uk_40 = data.read_f32()?;
            event_data.uk_41 = data.read_f32()?;
            event_data.uk_42 = data.read_f32()?;
            event_data.uk_43 = data.read_f32()?;
            event_data.uk_44 = data.read_f32()?;
            event_data.uk_45 = data.read_f32()?;
            event_data.uk_46 = data.read_f32()?;
            event_data.uk_47 = data.read_f32()?;

            self.event_data_mut().push(event_data);
        }

        // Event records.
        for _ in 0..data.read_u32()? {
            let mut event = EventRecord::default();
            *event.category_mut() = data.read_u32()?;

            *event.uk_2_mut() = data.read_i32()?;
            *event.uk_3_mut() = data.read_i32()?;
            *event.event_data_index_mut() = data.read_u32()?;

            // This is supposed to come from the previous section. For now leave it hardcoded.
            if *self.event_data()[*event.event_data_index() as usize].uk_29() as i32 == 1 {
                *event.name_mut() = Some(data.read_sized_string_u16()?);
            }

            for _ in 0..data.read_u32()? {
                event.sounds_mut().push(data.read_sized_string_u16()?);
            }

            *event.uk_4_mut() = data.read_u8()?;

            self.event_records_mut().push(event);
        }

        // Ambience map records.
        for _ in 0..data.read_u32()? {
            let mut map = AmbienceMap::default();

            *map.name_mut() = data.read_sized_string_u16()?;

            for _ in 0..data.read_u32()? {
                let mut ambience = AmbienceRecord::default();

                ambience.uk_1 = data.read_u32()?;
                ambience.event_index = data.read_u32()?;
                ambience.uk_3 = data.read_f32()?;
                ambience.uk_4 = data.read_f32()?;
                ambience.uk_5 = data.read_f32()?;

                map.records_mut().push(ambience);
            }

            self.ambience_map_mut().push(map);
        }

        for _ in 0..data.read_u32()? {
            let mut uk_3 = Uk3::default();

            uk_3.uk_1 = data.read_i32()?;

            self.uk_3_mut().push(uk_3);
        }

        // Movies
        for _ in 0..data.read_u32()? {
            let mut movie = Movie::default();

            movie.file = data.read_sized_string_u16()?;
            movie.volume = data.read_f32()?;

            self.movies_mut().push(movie);
        }

        for _ in 0..data.read_u32()? {
            let mut uk_9 = Uk9::default();

            uk_9.file = data.read_sized_string_u16()?;
            uk_9.uk_1 = data.read_i32()?;

            self.uk_9_mut().push(uk_9);
        }

        Ok(())
    }

    pub(crate) fn write_sho2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {
        buffer.write_f32(self.master_volume)?;

        buffer.write_u32(self.categories.len() as u32)?;
        for category in self.categories() {
            buffer.write_sized_string_u16(&category.name)?;
            buffer.write_f32(category.uk_1)?;
        }

        buffer.write_u32(self.uk_1.len() as u32)?;
        for data in self.uk_1() {
            buffer.write_i32(data.uk_1)?;
        }

        buffer.write_u32(self.uk_4.len() as u32)?;
        for data in self.uk_4() {
            buffer.write_i32(data.uk_1)?;
            buffer.write_i32(data.uk_2)?;
        }

        buffer.write_u32(self.uk_5.len() as u32)?;
        for data in self.uk_5() {
            buffer.write_f32(data.uk_1)?;
            buffer.write_f32(data.uk_2)?;
            buffer.write_f32(data.uk_3)?;
            buffer.write_f32(data.uk_4)?;
            buffer.write_f32(data.uk_5)?;
            buffer.write_f32(data.uk_6)?;
            buffer.write_f32(data.uk_7)?;
            buffer.write_f32(data.uk_8)?;
        }

        buffer.write_u32(self.uk_6)?;
        buffer.write_u32(self.uk_7)?;

        for data in self.uk_8() {
            buffer.write_u32(data.uk_1)?;
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
            buffer.write_f32(data.uk_27)?;
            buffer.write_f32(data.uk_28)?;
            buffer.write_f32(data.uk_29)?;
            buffer.write_f32(data.uk_30)?;
            buffer.write_f32(data.uk_31)?;
            buffer.write_f32(data.uk_32)?;
            buffer.write_f32(data.uk_33)?;
            buffer.write_f32(data.uk_34)?;
            buffer.write_f32(data.uk_35)?;
            buffer.write_f32(data.uk_36)?;
            buffer.write_f32(data.uk_37)?;
            buffer.write_f32(data.uk_38)?;
            buffer.write_f32(data.uk_39)?;
            buffer.write_f32(data.uk_40)?;
            buffer.write_f32(data.uk_41)?;
            buffer.write_f32(data.uk_42)?;
            buffer.write_f32(data.uk_43)?;
            buffer.write_f32(data.uk_44)?;
            buffer.write_f32(data.uk_45)?;
            buffer.write_f32(data.uk_46)?;
            buffer.write_f32(data.uk_47)?;
        }

        buffer.write_u32(self.event_records.len() as u32)?;
        for event_record in self.event_records() {
            buffer.write_u32(event_record.category)?;
            buffer.write_i32(event_record.uk_2)?;
            buffer.write_i32(event_record.uk_3)?;
            buffer.write_u32(event_record.event_data_index)?;

            if let Some(name) = event_record.name() {
                buffer.write_sized_string_u16(name)?;
            }

            buffer.write_u32(event_record.sounds.len() as u32)?;
            for sound in event_record.sounds() {
                buffer.write_sized_string_u16(sound)?;
            }

            buffer.write_u8(event_record.uk_4)?;
        }

        buffer.write_u32(self.ambience_map.len() as u32)?;
        for map in self.ambience_map() {
            buffer.write_sized_string_u16(map.name())?;
            buffer.write_u32(map.records.len() as u32)?;

            for record in map.records() {
                buffer.write_u32(record.uk_1)?;
                buffer.write_u32(record.event_index)?;
                buffer.write_f32(record.uk_3)?;
                buffer.write_f32(record.uk_4)?;
                buffer.write_f32(record.uk_5)?;
            }
        }

        buffer.write_u32(self.uk_3.len() as u32)?;
        for extra_data in self.uk_3() {
            buffer.write_i32(extra_data.uk_1)?;
        }

        buffer.write_u32(self.movies.len() as u32)?;
        for category in self.movies() {
            buffer.write_sized_string_u16(category.file())?;
            buffer.write_f32(category.volume)?;
        }

        buffer.write_u32(self.uk_9.len() as u32)?;
        for uk_9 in self.uk_9() {
            buffer.write_sized_string_u16(uk_9.file())?;
            buffer.write_i32(uk_9.uk_1)?;
        }

        Ok(())
    }
}
