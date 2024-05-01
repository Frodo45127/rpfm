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
            self.uk_1_mut().push(data.read_f32()?);
        }

        // BankEventUk0
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk0::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_0_mut().push(bank);
        }

        // BankEventProjectileFire
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventProjectileFire::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.gun_types_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.shot_types_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.projectile_sizes_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.unit_indexes_mut().push(data.read_u32()?);
            }

            self.bank_event_projectile_fire_mut().push(bank);
        }

        // BankEventUk2
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk2::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_2_mut().push(bank);
        }

        // BankEventUk3
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk3::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_3_mut().push(bank);
        }

        // BankEventUk4
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk4::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_4_mut().push(bank);
        }

        // BankEventUk5
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk5::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_5_mut().push(bank);
        }

        // BankEventUk6
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk6::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_5_mut().push(data.read_u8()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_6_mut().push(data.read_u8()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_7_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_6_mut().push(bank);
        }

        // BankEventUk7
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk7::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_5_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_7_mut().push(bank);
        }

        // BankEventUk8
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk8::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_8_mut().push(bank);
        }

        // BankEventUk9
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk9::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_9_mut().push(bank);
        }

        // BankEventUk10
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk10::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_10_mut().push(bank);
        }

        // BankEventUk11
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk11::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_11_mut().push(bank);
        }

        // BankEventUk12
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk12::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_5_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_6_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_12_mut().push(bank);
        }

        // BankEventUk13
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk13::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_13_mut().push(bank);
        }

        // BankEventUk14
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk14::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_14_mut().push(bank);
        }

        // BankEventUk15
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk15::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_4_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_5_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_6_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_15_mut().push(bank);
        }

        // BankEventUk16
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk16::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_16_mut().push(bank);
        }

        // BankEventUk17
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk17::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_17_mut().push(bank);
        }

        // BankEventUk18
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk18::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_3_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_18_mut().push(bank);
        }

        // BankEventUk19
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk19::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_19_mut().push(bank);
        }

        // BankEventUk20
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk20::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_20_mut().push(bank);
        }

        // BankEventUk21
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk21::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_21_mut().push(bank);
        }

        // BankEventUk22
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk22::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_22_mut().push(bank);
        }

        // BankEventUk23
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk23::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_23_mut().push(bank);
        }

        // BankEventUk24
        for _ in 0..data.read_u32()? {
            let mut bank = BankEventUk24::default();
            bank.event_record_index = data.read_u32()?;

            for _ in 0..data.read_u32()? {
                bank.params_1_mut().push(data.read_u32()?);
            }

            for _ in 0..data.read_u32()? {
                bank.params_2_mut().push(data.read_u32()?);
            }

            self.bank_event_uk_24_mut().push(bank);
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

        buffer.write_u32(self.bank_event_uk_0().len() as u32)?;
        for bank in self.bank_event_uk_0() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_projectile_fire().len() as u32)?;
        for bank in self.bank_event_projectile_fire() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.gun_types().len() as u32)?;
            for param in bank.gun_types() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.shot_types().len() as u32)?;
            for param in bank.shot_types() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.projectile_sizes().len() as u32)?;
            for param in bank.projectile_sizes() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.unit_indexes().len() as u32)?;
            for param in bank.unit_indexes() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_2().len() as u32)?;
        for bank in self.bank_event_uk_2() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_3().len() as u32)?;
        for bank in self.bank_event_uk_3() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_4().len() as u32)?;
        for bank in self.bank_event_uk_4() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_5().len() as u32)?;
        for bank in self.bank_event_uk_5() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_6().len() as u32)?;
        for bank in self.bank_event_uk_6() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_5().len() as u32)?;
            for param in bank.params_5() {
                buffer.write_u8(*param)?;
            }

            buffer.write_u32(bank.params_6().len() as u32)?;
            for param in bank.params_6() {
                buffer.write_u8(*param)?;
            }

            buffer.write_u32(bank.params_7().len() as u32)?;
            for param in bank.params_7() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_7().len() as u32)?;
        for bank in self.bank_event_uk_7() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_5().len() as u32)?;
            for param in bank.params_5() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_8().len() as u32)?;
        for bank in self.bank_event_uk_8() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_9().len() as u32)?;
        for bank in self.bank_event_uk_9() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_10().len() as u32)?;
        for bank in self.bank_event_uk_10() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_11().len() as u32)?;
        for bank in self.bank_event_uk_11() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_12().len() as u32)?;
        for bank in self.bank_event_uk_12() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_5().len() as u32)?;
            for param in bank.params_5() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_6().len() as u32)?;
            for param in bank.params_6() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_13().len() as u32)?;
        for bank in self.bank_event_uk_13() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_14().len() as u32)?;
        for bank in self.bank_event_uk_14() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_15().len() as u32)?;
        for bank in self.bank_event_uk_15() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_4().len() as u32)?;
            for param in bank.params_4() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_5().len() as u32)?;
            for param in bank.params_5() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_6().len() as u32)?;
            for param in bank.params_6() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_16().len() as u32)?;
        for bank in self.bank_event_uk_16() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_17().len() as u32)?;
        for bank in self.bank_event_uk_17() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_18().len() as u32)?;
        for bank in self.bank_event_uk_18() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_3().len() as u32)?;
            for param in bank.params_3() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_19().len() as u32)?;
        for bank in self.bank_event_uk_19() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_20().len() as u32)?;
        for bank in self.bank_event_uk_20() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_21().len() as u32)?;
        for bank in self.bank_event_uk_21() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_22().len() as u32)?;
        for bank in self.bank_event_uk_22() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_23().len() as u32)?;
        for bank in self.bank_event_uk_23() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
            }
        }

        buffer.write_u32(self.bank_event_uk_24().len() as u32)?;
        for bank in self.bank_event_uk_24() {
            buffer.write_u32(bank.event_record_index)?;

            buffer.write_u32(bank.params_1().len() as u32)?;
            for param in bank.params_1() {
                buffer.write_u32(*param)?;
            }

            buffer.write_u32(bank.params_2().len() as u32)?;
            for param in bank.params_2() {
                buffer.write_u32(*param)?;
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
