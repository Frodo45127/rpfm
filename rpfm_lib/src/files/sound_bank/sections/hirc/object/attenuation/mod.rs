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

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::sound_bank::common::*;

// Valid between 89 and 141.
const NUM_CURVES: usize = 7;

mod v122;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Attenuation {
    id: u32,

    // f_inside_degrees, f_outside_degrees, f_outside_volume, lo_pass, hi_pass
    is_cone_enabled: Option<(f32, f32, f32, f32, f32)>,
    curve_index: Vec<u8>,
    curves: Vec<RTPCCurve>,
    initial_rtpc: InitialRTPC,
}

//---------------------------------------------------------------------------//
//                              Implementation
//---------------------------------------------------------------------------//

impl Attenuation {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        match version {
            122 => Self::read_v122(data, version),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(version, "Attenuation".to_string())),
        }
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        match version {
            122 => self.write_v122(buffer, version),
            _ => Err(RLibError::SoundBankUnsupportedVersionFound(version, "Attenuation".to_string())),
        }
    }
}
