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

const PARAMETER_BLOCK_SIZES: [u8; 25] = [4, 5, 4, 3, 2, 4, 4, 5, 4, 2, 2, 1, 6, 4, 4, 6, 2, 3, 3, 2, 2, 2, 2, 2, 2];

impl SoundBankDatabase {

    pub(crate) fn read_sho2<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {

        for _ in 0..266 {
            self.uk_1_mut().push(data.read_f32()?);
        }

        // Sound Bank Records.
        for i in 0..25 {
            let mut record = SoundBankRecord::default();

            // Bank event records.
            for _ in 0..data.read_u32()? {
                let mut bank = BankEventRecord::default();

                bank.event_record_index = data.read_u32()?;

                if let Some(blocks) = PARAMETER_BLOCK_SIZES.get(i) {
                    for _ in 0..*blocks {
                        let mut block = ParameterBlockU32::default();
                        for _ in 0..data.read_u32()? {
                            block.params_mut().push(data.read_u32()?);
                        }
                        bank.parameter_blocks_u32.push(block);
                    }

                    if i == 6 {
                        for _ in 0..2 {
                            let mut block = ParameterBlockU8::default();
                            for _ in 0..data.read_u32()? {
                                block.params_mut().push(data.read_u8()?);
                            }
                            bank.parameter_blocks_u8.push(block);
                        }

                        let mut block = ParameterBlockU32::default();
                        for _ in 0..data.read_u32()? {
                            block.params_mut().push(data.read_u32()?);
                        }
                        bank.parameter_blocks_u32.push(block);
                    }
                }

                record.bank_event_records_mut().push(bank);
            }

            self.sound_bank_records.push(record);
        }

        // Unknown data.
        for _ in 0..297 {
            let mut uk_1 = Uk1::default();

            for _ in 0..data.read_u32()? {
                uk_1.uk_1_mut().push(data.read_u32()?);
            }

            self.uk_2_mut().push(uk_1);
        }

        Ok(())
    }

    pub(crate) fn write_sho2<W: WriteBytes>(&mut self, buffer: &mut W) -> Result<()> {

        for uk_1 in self.uk_1() {
            buffer.write_f32(*uk_1)?;
        }

        for (index, record) in self.sound_bank_records().iter().enumerate() {
            buffer.write_u32(record.bank_event_records().len() as u32)?;
            for bank in record.bank_event_records() {
                buffer.write_u32(bank.event_record_index)?;

                let blocks_len = if index == 6 {
                    bank.parameter_blocks_u32.len() - 1
                } else {
                    bank.parameter_blocks_u32.len()
                };

                for block in &bank.parameter_blocks_u32()[..blocks_len] {
                    buffer.write_u32(block.params().len() as u32)?;
                    for param in block.params() {
                        buffer.write_u32(*param)?;
                    }
                }

                if index == 6 {
                    for block in bank.parameter_blocks_u8() {
                        buffer.write_u32(block.params().len() as u32)?;
                        for param in block.params() {
                            buffer.write_u8(*param)?;
                        }
                    }

                    if let Some(block) = bank.parameter_blocks_u32().last() {
                        buffer.write_u32(block.params().len() as u32)?;
                        for param in block.params() {
                            buffer.write_u32(*param)?;
                        }
                    }
                }
            }
        }

        for uk_2 in self.uk_2() {
            buffer.write_u32(uk_2.uk_1().len() as u32)?;

            for uk_1 in uk_2.uk_1() {
                buffer.write_u32(*uk_1)?;
            }
        }


        Ok(())
    }
}
