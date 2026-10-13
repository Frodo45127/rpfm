//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::particle_emitter::ParticleEmitter;

pub use self::marker::ParticleEmitterMarker;
use super::*;

mod marker;
mod particle_emitter;
mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ParticleEmitterList {
    serialise_version: u16,
    particle_emitters: Vec<ParticleEmitter>,

    /// Particle emitters as Attila stores them. Later games use `particle_emitters` instead.
    markers: Vec<ParticleEmitterMarker>,
}

impl ParticleEmitterList {

    /// Reads the particle emitters of Attila: a count and that many markers.
    pub(crate) fn read_markers<R: ReadBytes>(&mut self, data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<()> {
        for _ in 0..data.read_u32()? {
            self.markers.push(ParticleEmitterMarker::decode(data, extra_data)?);
        }

        Ok(())
    }

    /// Writes the particle emitters of Attila: a count and that many markers.
    pub(crate) fn write_markers<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.markers.len() as u32)?;
        for marker in &mut self.markers {
            marker.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}

//---------------------------------------------------------------------------//
//                   Implementation of ParticleEmitterList
//---------------------------------------------------------------------------//

impl Decodeable for ParticleEmitterList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("ParticleEmitterList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for ParticleEmitterList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("ParticleEmitterList"), self.serialise_version)),
        }

        Ok(())
    }
}
