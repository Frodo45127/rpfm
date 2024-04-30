//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

impl SoundBankDatabase {

    pub(crate) fn read_sho2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        for _ in 0..266 {
            let _a = data.read_f32()?;
        }

        dbg!(data.stream_position()?);
        // Sound Bank Records.
        for i in 0..25 {
            dbg!(i, data.stream_position()?);

            // Bank event records
            for _ in 0..data.read_u32()? {
                let event_record_index = data.read_u32()?;

                // Parameter blocks.
                match i {
                    6 => {
                        let a = data.read_u32()?;
                        dbg!(&a);
                        for _ in 0..a {
                            let _a = data.read_u32()?;
                        }

                        let a = data.read_u32()?;
                        dbg!(&a);
                        for _ in 0..a {
                            let _a = data.read_u8()?;
                        }

                        let a = data.read_u32()?;
                        dbg!(&a);
                        for _ in 0..a {
                            let _a = data.read_u32()?;
                        }
                    },
                    _ => {
                        let a = data.read_u32()?;
                        dbg!(&a);
                        for _ in 0..a {
                            let _a = data.read_u32()?;
                        }
                    }
                    ,
                }
            }
        }
        dbg!(data.stream_position()?);

        for _ in 0..297 {
            for _ in 0..data.read_u32()? {
                let _a = data.read_u32()?;
            }
        }

        dbg!(data.stream_position()?);

        Ok(())
    }

    pub(crate) fn write_sho2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {

        Ok(())
    }
}
