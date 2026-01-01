//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::sound_bank::*;

// Valid from 77 to 141.
const NON_PADDED_SIZE: usize = 0x14;

mod v122;
mod v136;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BKHD {
    version: u32,
    id: u64,
    language: Language,
    feedback_in_bank: u32,
    alignment: u16,
    device_allocated: u16,
    project_id: u32,
    padding: Vec<u8>,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    Sfx = 0x00,
    Arabic = 0x01,
    Bulgarian = 0x02,
    Chinese1 = 0x03, //(HK),
    Chinese2 = 0x04, //(PRC),
    Chinese3 = 0x05, //(Taiwan),
    Czech = 0x06,
    Danish = 0x07,
    Dutch = 0x08,
    English1 = 0x09, //(Australia),
    English2 = 0x0A, //(India),
    English3 = 0x0B, //(UK),
    English4 = 0x0C, //(US),
    Finnish = 0x0D,
    French1 = 0x0E, //(Canada),
    French2 = 0x0F, //(France),
    German = 0x10,
    Greek = 0x11,
    Hebrew = 0x12,
    Hungarian = 0x13,
    Indonesian = 0x14,
    Italian = 0x15,
    Japanese = 0x16,
    Korean = 0x17,
    Latin = 0x18,
    Norwegian = 0x19,
    Polish = 0x1A,
    Portuguese1 = 0x1B, //(Brazil),
    Portuguese2 = 0x1C, //(Portugal),
    Romanian = 0x1D,
    Russian = 0x1E,
    Slovenian = 0x1F,
    Spanish1 = 0x20, //(Mexico),
    Spanish2 = 0x21, //(Spain),
    Spanish3 = 0x22, //(US),
    Swedish = 0x23,
    Turkish = 0x24,
    Ukrainian = 0x25,
    Vietnamese = 0x26,
}

//---------------------------------------------------------------------------//
//                        Implementation of SoundBank
//---------------------------------------------------------------------------//

impl TryFrom<u32> for Language {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Sfx,
            0x01 => Self::Arabic,
            0x02 => Self::Bulgarian,
            0x03 => Self::Chinese1,
            0x04 => Self::Chinese2,
            0x05 => Self::Chinese3,
            0x06 => Self::Czech,
            0x07 => Self::Danish,
            0x08 => Self::Dutch,
            0x09 => Self::English1,
            0x0A => Self::English2,
            0x0B => Self::English3,
            0x0C => Self::English4,
            0x0D => Self::Finnish,
            0x0E => Self::French1,
            0x0F => Self::French2,
            0x10 => Self::German,
            0x11 => Self::Greek,
            0x12 => Self::Hebrew,
            0x13 => Self::Hungarian,
            0x14 => Self::Indonesian,
            0x15 => Self::Italian,
            0x16 => Self::Japanese,
            0x17 => Self::Korean,
            0x18 => Self::Latin,
            0x19 => Self::Norwegian,
            0x1A => Self::Polish,
            0x1B => Self::Portuguese1,
            0x1C => Self::Portuguese2,
            0x1D => Self::Romanian,
            0x1E => Self::Russian,
            0x1F => Self::Slovenian,
            0x20 => Self::Spanish1,
            0x21 => Self::Spanish2,
            0x22 => Self::Spanish3,
            0x23 => Self::Swedish,
            0x24 => Self::Turkish,
            0x25 => Self::Ukrainian,
            0x26 => Self::Vietnamese,
            _ => Err(RLibError::SoundBankUnsupportedLanguageFound(value))?,
        })
    }
}

impl BKHD {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, section_size: usize) -> Result<Self> {

        // CA seems to put an extra byte here for some reason. Remove it to get the proper version.
        let version = data.read_u32()? ^ 0x80_00_00_00;
        match version {
            122 => Self::read_v122(data, version, section_size),
            136 => Self::read_v136(data, version, section_size),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(version, SIGNATURE_BKHD.to_string())),
        }
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {

        // Version is written in each version's function.
        match self.version {
            122 => self.write_v122(buffer),
            136 => self.write_v136(buffer),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(self.version, SIGNATURE_BKHD.to_string())),
        }
    }
}
