//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use self::building::Building;

use super::*;

mod building;
mod v1;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BattlefieldBuildingList {
    serialise_version: u16,
    buildings: Vec<Building>
}

//---------------------------------------------------------------------------//
//                   Implementation of BattlefieldBuildingList
//---------------------------------------------------------------------------//

impl Decodeable for BattlefieldBuildingList {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            1 => decoded.read_v1(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("BattlefieldBuildingList"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for BattlefieldBuildingList {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            1 => self.write_v1(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("BattlefieldBuildingList"), self.serialise_version)),
        }

        Ok(())
    }
}

impl ToLayer for BattlefieldBuildingList {
    fn to_layer(&self, parent: &Bmd) -> Result<String> {
        let mut layer = String::new();

        for (index, building) in self.buildings().iter().enumerate() {
            layer.push_str(&format!("
        <entity id=\"{:x}\">",
                building.uid())
            );

            layer.push_str(&format!("
            <ECBuilding key=\"{}\" damage=\"{}\" indestructible=\"{}\" toggleable=\"{}\" key_building=\"{}\" hide_tooltip=\"{}\" settlement_level_configurable=\"{}\" capture_location=\"{}\" export_as_prop=\"false\" visible_beyond_outfield=\"false\"/>",
                building.building_key(),
                building.properties().starting_damage_unary(),
                building.properties().indestructible(),
                building.properties().toggleable(),
                building.properties().key_building(),
                building.properties().hide_tooltip(),
                building.properties().settlement_level_configurable(),

                // For capture location, we need to find it by searching in the capture locations themselfs.
                match parent.capture_location_set().capture_location_sets()
                    .iter()
                    .find_map(|locations| locations.capture_locations()
                        .iter()
                        .find_map(|location| location.building_links()
                            .iter()
                            .find_map(|link|
                                if *link.building_index() == index as i32 || link.uid() == building.uid() {
                                    Some(*location.id())
                                } else {
                                    None
                                })
                        )
                    ) {
                    Some(id) => format!("{:x}", id),
                    None => String::new()
                }
            ));

            layer.push_str(&format!("
            <ECMeshRenderSettings cast_shadow=\"{}\" receive_decals=\"true\" render_into_skydome_fog=\"false\"/>",
                building.properties().cast_shadows()
            ));

            layer.push_str("
            <ECVisibilitySettingsBattle visible_in_tactical_view=\"true\" visible_in_tactical_view_only=\"false\"/>");
            layer.push_str("
            <ECWater is_water=\"false\"/>");

            let rotation_matrix = building.transform().rotation_matrix();
            let scales = Transform3x4::extract_scales(rotation_matrix);
            let normalized_rotation_matrix = Transform3x4::normalize_rotation_matrix(rotation_matrix, scales);
            let angles = Transform3x4::rotation_matrix_to_euler_angles(normalized_rotation_matrix, true);

            layer.push_str(&format!("
            <ECTransform position=\"{:.5} {:.5} {:.5}\" rotation=\"{:.5} {:.5} {:.5}\" scale=\"{:.5} {:.5} {:.5}\" pivot=\"0 0 0\"/>",
                building.transform().m30(), building.transform().m31(), building.transform().m32(),
                angles.0, angles.1, angles.2,
                scales.0, scales.1, scales.2
            ));

            layer.push_str(&format!("
            <ECTerrainClamp active=\"{}\" clamp_to_sea_level=\"false\" terrain_oriented=\"{}\" fit_height_to_terrain=\"false\"/>",
                *building.properties().clamp_to_surface() || building.height_mode() == "BHM_TERRAIN" || building.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION",
                building.height_mode() == "BHM_TERRAIN_ALIGN_ORIENTATION"
            ));

            layer.push_str(&format!("
            <ECPrefabOverride enabled=\"false\" id=\"{}\"/>",
                building.building_key())
            );

            layer.push_str("
        </entity>"
            );
        }

        Ok(layer)
    }
}
