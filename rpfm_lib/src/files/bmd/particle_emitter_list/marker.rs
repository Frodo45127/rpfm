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

use super::*;

/// Version of the particle emitter markers. No other is known.
const MARKER_VERSION: u16 = 4;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// A particle emitter as Attila stores it: with its key, and its data in a marker of version 4.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ParticleEmitterMarker {

    /// Key of the particle effect.
    key: String,

    /// Transform of the emitter (stored before the emission rate).
    transform: Transform4x4,

    /// Particles emitted per second.
    emission_rate: f32,

    /// Name of this instance of the emitter.
    instance_name: String,

    /// If the emitter starts by itself.
    autoplay: bool,

    /// Scale of the effect.
    scale: f32,

    /// If the emitter is used outside the playable area.
    use_in_outfield: bool,

    /// If the emitter is used with snow, inside the playable area.
    snow_inside: bool,

    /// If the emitter is used with snow, outside the playable area.
    snow_outside: bool,

    /// If the emitter is used when destroyed, inside the playable area.
    destruction_inside: bool,

    /// If the emitter is used when destroyed, outside the playable area.
    destruction_outside: bool,
}

//---------------------------------------------------------------------------//
//                  Implementation of ParticleEmitterMarker
//---------------------------------------------------------------------------//

impl Decodeable for ParticleEmitterMarker {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let key = data.read_sized_string_u8()?;
        let version = data.read_u16()?;
        if version != MARKER_VERSION {
            return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("ParticleEmitterMarker"), version));
        }

        Ok(Self {
            key,
            transform: Transform4x4::decode(data, extra_data)?,
            emission_rate: data.read_f32()?,
            instance_name: data.read_sized_string_u8()?,
            autoplay: data.read_bool()?,
            scale: data.read_f32()?,
            use_in_outfield: data.read_bool()?,
            snow_inside: data.read_bool()?,
            snow_outside: data.read_bool()?,
            destruction_inside: data.read_bool()?,
            destruction_outside: data.read_bool()?,
        })
    }
}

impl Encodeable for ParticleEmitterMarker {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_sized_string_u8(&self.key)?;
        buffer.write_u16(MARKER_VERSION)?;
        self.transform.encode(buffer, extra_data)?;
        buffer.write_f32(self.emission_rate)?;
        buffer.write_sized_string_u8(&self.instance_name)?;
        buffer.write_bool(self.autoplay)?;
        buffer.write_f32(self.scale)?;
        buffer.write_bool(self.use_in_outfield)?;
        buffer.write_bool(self.snow_inside)?;
        buffer.write_bool(self.snow_outside)?;
        buffer.write_bool(self.destruction_inside)?;
        buffer.write_bool(self.destruction_outside)?;

        Ok(())
    }
}
