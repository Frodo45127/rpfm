//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use self::prefab_instance::PrefabInstance;

use super::*;

mod prefab_instance;
mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PrefabInstanceList {
    serialise_version: u16,
    prefab_instances: Vec<PrefabInstance>,
}

//---------------------------------------------------------------------------//
//                Implementation of PrefabInstanceList
//---------------------------------------------------------------------------//

impl Decodeable for PrefabInstanceList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("PrefabInstanceList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for PrefabInstanceList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("PrefabInstanceList"), self.serialise_version)),
        }

        Ok(())
    }
}
 
impl ToLayer for PrefabInstanceList {
    fn to_layer(&self, _parent: &Bmd) -> Result<String> {
        let mut layer = String::new();

        for prefab in self.prefab_instances() {
            layer.push_str(&format!("
        <entity id=\"{:x}\">",
                prefab.uid())
            );

            let prefab_path_split = prefab.key().split('/').collect::<Vec<_>>();
            let prefab_name = prefab_path_split.last().unwrap();
            let prefab_name_split = prefab_name.split('.').collect::<Vec<_>>();
            let prefab_key = prefab_name_split.first().unwrap();

            layer.push_str(&format!("
            <ECPrefab key=\"{}\" use_culture_mask=\"false\" valid_ids=\"1\"/>",
                prefab_key,
            ));

            let rotation_matrix = prefab.transform().rotation_matrix();
            let scales = Transform3x4::extract_scales(rotation_matrix);
            let normalized_rotation_matrix = Transform3x4::normalize_rotation_matrix(rotation_matrix, scales);
            let angles= Transform3x4::rotation_matrix_to_euler_angles(normalized_rotation_matrix, true);

            layer.push_str(&format!("
            <ECTransform position=\"{:.5} {:.5} {:.5}\" rotation=\"{:.5} {:.5} {:.5}\" scale=\"{:.5} {:.5} {:.5}\" pivot=\"0 0 0\"/>",
                prefab.transform().m30(), prefab.transform().m31(), prefab.transform().m32(),
                angles.0, angles.1, angles.2,
                scales.0, scales.1, scales.2
            ));

            layer.push_str(&format!("
            <ECTerrainClamp active=\"{}\" clamp_to_sea_level=\"false\" terrain_oriented=\"{}\" fit_height_to_terrain=\"false\"/>",
                *prefab.clamp_to_surface() || prefab.height_mode() == "BHM_TERRAIN" || prefab.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION",
                prefab.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION"
            ));

            layer.push_str("
        </entity>"
            );
        }

        Ok(layer)
    }
}
