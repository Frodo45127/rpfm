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

/// Version of the civilian deployment records. No other is known.
const DEPLOYMENT_VERSION: u16 = 1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Areas where civilians are deployed in a battle.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianDeploymentList {
    civilian_deployment_list: Vec<CivilianDeployment>,
}

/// An area where civilians are deployed.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CivilianDeployment {

    /// Kind of area, like `Urban` or `Agricultural`.
    kind: String,

    /// Size of the area, like `small`, `medium` or `large`.
    size: String,

    /// Outline of the area.
    area: Polygon2d,
}

//---------------------------------------------------------------------------//
//                Implementation of CivilianDeploymentList
//---------------------------------------------------------------------------//

impl Decodeable for CivilianDeploymentList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            let version = data.read_u16()?;
            if version != DEPLOYMENT_VERSION {
                return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CivilianDeployment"), version));
            }

            decoded.civilian_deployment_list.push(CivilianDeployment {
                kind: data.read_sized_string_u8()?,
                size: data.read_sized_string_u8()?,
                area: Polygon2d::decode(data, extra_data)?,
            });
        }

        Ok(decoded)
    }
}

impl Encodeable for CivilianDeploymentList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.civilian_deployment_list.len() as u32)?;

        for deployment in &mut self.civilian_deployment_list {
            buffer.write_u16(DEPLOYMENT_VERSION)?;
            buffer.write_sized_string_u8(&deployment.kind)?;
            buffer.write_sized_string_u8(&deployment.size)?;
            deployment.area.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}
