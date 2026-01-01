//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! For more info about all this stuff, check <https://github.com/bnnm/wwiser/>.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use self::sections::bkhd::BKHD;
use self::sections::hirc::HIRC;

use super::DecodeableExtraData;

/// Extension used by soundbank files.
pub const EXTENSION: &str = ".bnk";

/// Hash Types.
const FNV_NO: &str = "none";// #special value, no hashname allowed
const FNV_BNK: &str = "bank";
const FNV_LNG: &str = "language";
const FNV_EVT: &str = "event";
const FNV_BUS: &str = "bus";
const FNV_SFX: &str = "sfx";
const FNV_TRG: &str = "trigger";
const FNV_GME: &str = "rtpc/game-variable";
const FNV_VAR: &str = "variable";// #switches/states names
const FNV_VAL: &str = "value";// #switches/states values
const FNV_UNK: &str = "???";

const FNV_ORDER: [&str; 10] = [
  FNV_BNK, FNV_LNG, FNV_EVT, FNV_BUS, FNV_SFX, FNV_TRG, FNV_GME, FNV_VAR, FNV_VAL, FNV_UNK
];

const FNV_ORDER_JOIN: [&str; 3] = [
  FNV_BNK, FNV_LNG, FNV_BUS
];

const SIGNATURE_AKBK: &str = "AKBK"; //'AKBK': "Audiokinetic Bank",
const SIGNATURE_BKHD: &str = "BKHD"; //'BKHD': "Bank Header",
const SIGNATURE_HIRC: &str = "HIRC"; //'HIRC': "Hierarchy",
const SIGNATURE_DATA: &str = "DATA"; //'DATA': "Data",
const SIGNATURE_FXPR: &str = "FXPR"; //'FXPR': "FX Parameters",
const SIGNATURE_ENVS: &str = "ENVS"; //'ENVS': "Enviroment Settings",
const SIGNATURE_STID: &str = "STID"; //'STID': "String Mappings",
const SIGNATURE_STMG: &str = "STMG"; //'STMG': "Global Settings",
const SIGNATURE_DIDX: &str = "DIDX"; //'DIDX': "Media Index",
const SIGNATURE_PLAT: &str = "PLAT"; //'PLAT': "Custom Platform",
const SIGNATURE_INIT: &str = "INIT"; //'INIT': "Plugin",

mod common;
mod sections;

#[cfg(test)] mod test_soundbank;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `SoundBank` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundBank {
    sections: Vec<Section>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Section {
    BKHD(BKHD),
    HIRC(HIRC),
}

//---------------------------------------------------------------------------//
//                        Implementation of SoundBank
//---------------------------------------------------------------------------//

impl Decodeable for SoundBank {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        let data_len = data.len()?;

        while let Ok(section_signature) = data.read_string_u8(4) {
            let section_size = data.read_u32()? as u64;
            //let section_start = data.stream_position()?;
            //let section_end = section_start + section_size;

            dbg!(&section_signature);
            dbg!(&section_size);
            decoded.sections.push(match &*section_signature {
                SIGNATURE_AKBK => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_AKBK.to_string())),

                // First node is always a BKHD (BanK HeaDer?).
                SIGNATURE_BKHD => Section::BKHD(BKHD::read(data, section_size as usize)?),
                SIGNATURE_HIRC => {
                    let header = match decoded.sections.first() {
                        Some(Section::BKHD(section)) => section,
                        _ => return Err(RLibError::SoundBankBKHDNotFound),
                    };

                    Section::HIRC(HIRC::read(data, *header.version())?)
                },
                SIGNATURE_DATA => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_DATA.to_string())),
                SIGNATURE_FXPR => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_FXPR.to_string())),
                SIGNATURE_ENVS => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_ENVS.to_string())),
                SIGNATURE_STID => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_STID.to_string())),
                SIGNATURE_STMG => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_STMG.to_string())),
                SIGNATURE_DIDX => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_DIDX.to_string())),
                SIGNATURE_PLAT => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_PLAT.to_string())),
                SIGNATURE_INIT => return Err(RLibError::SoundBankUnsupportedSectionFound(SIGNATURE_INIT.to_string())),
                _ => return Err(RLibError::SoundBankUnsupportedSectionFound(section_signature)),
            });
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for SoundBank {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let header = match self.sections.first() {
            Some(Section::BKHD(section)) => section,
            _ => return Err(RLibError::SoundBankBKHDNotFound),
        };

        for section in self.sections() {
            match section {
                Section::BKHD(data) => data.write(buffer)?,
                Section::HIRC(data) => data.write(buffer, *header.version())?,
            }
        }

        Ok(())
    }
}
