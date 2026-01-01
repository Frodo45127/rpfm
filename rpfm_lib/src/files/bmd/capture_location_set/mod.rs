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
use rand::Rng;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{bmd::building_link::BuildingLink, Decodeable, EncodeableExtraData, Encodeable};

use super::*;

mod v2;
mod v7;
mod v8;
mod v10;
mod v11;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocationSet {
    serialise_version: u16,
    capture_location_sets: Vec<CaptureLocationList>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocationList {
    capture_locations: Vec<CaptureLocation>,
}

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct CaptureLocation {
    id: u64,
    location: Point2d,
    radius: f32,
    valid_for_min_num_players: u32,
    valid_for_max_num_players: u32,
    capture_point_type: String,
    restore_type: String,
    location_points: Vec<Point2d>,
    database_key: String,
    flag_facing: Point2d,
    destroy_building_on_capture: bool,
    disable_building_abilities_when_no_original_owner: bool,
    abilities_affect_globally: bool,
    building_links: Vec<BuildingLink>,
    toggle_slots_links: Vec<u32>,
    ai_hints_links: Vec<u8>,
    script_id: String,
    is_time_based: bool,
}

//---------------------------------------------------------------------------//
//                Implementation of CaptureLocationSet
//---------------------------------------------------------------------------//

impl Decodeable for CaptureLocationSet {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.serialise_version = data.read_u16()?;

        match decoded.serialise_version {
            2 => decoded.read_v2(data, extra_data)?,
            7 => decoded.read_v7(data, extra_data)?,
            8 => decoded.read_v8(data, extra_data)?,
            10 => decoded.read_v10(data, extra_data)?,
            11 => decoded.read_v11(data, extra_data)?,
            _ => return Err(RLibError::DecodingFastBinUnsupportedVersion(String::from("CaptureLocationSet"), decoded.serialise_version)),
        }

        Ok(decoded)
    }
}

impl Encodeable for CaptureLocationSet {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u16(self.serialise_version)?;

        match self.serialise_version {
            2 => self.write_v2(buffer, extra_data)?,
            7 => self.write_v7(buffer, extra_data)?,
            8 => self.write_v8(buffer, extra_data)?,
            10 => self.write_v10(buffer, extra_data)?,
            11 => self.write_v11(buffer, extra_data)?,
            _ => return Err(RLibError::EncodingFastBinUnsupportedVersion(String::from("CaptureLocationSet"), self.serialise_version)),
        }

        Ok(())
    }
}

impl ToLayer for CaptureLocationSet {
    fn to_layer(&self, _parent: &Bmd) -> Result<String> {
        let mut layer = String::new();

        for capture_location_set in self.capture_location_sets() {
            for capture_location in capture_location_set.capture_locations() {
/*
                    <entity id="187662433aa63a7" name="Halbinsel Wall">
      <ECCaptureLocation type="minor_key_building_melee" importance="CLIT_KEY_BUILDING_A" restore_type="PreviousOwner" initial_owner_type="DeploymentArea" flag_position="47 -42" min_players="2" max_players="8" flag_facing_direction="0" destroy_building_on_capture="false"
      disable_building_abilities_when_no_original_owner="false" abilities_affect_globally="true" is_time_based="false" script_id=""/>
      <ECTransform position="2439 80 1703" rotation="0 0 0" scale="1 1 1" pivot="0 0 0"/>
      <ECTransform2D/>
      <ECPolyline>
        <polyline closed="true">
          <point x="0" y="0"/>
          <point x="76" y="0"/>
          <point x="75.7275391" y="-80.2313232"/>
          <point x="0" y="-80"/>
        </polyline>
      </ECPolyline>
    </entity>


    <CAPTURE_LOCATION radius='0.000000' valid_for_min_num_players='2' valid_for_max_num_players='8' capture_point_type='CAPTURE_LOCATION_KEY_BUILDING_A' restore_type='PREVIOUS_OWNER' database_key='minor_key_building_melee' destroy_building_on_capture='false'
    disable_building_abilities_when_no_original_owner='false' abilities_affect_globally='true' script_id='' is_time_based='false'>
        <location x='2486.000000' y='1661.000000'/>
        <location_points>
            <point x='2439.000000' y='1703.000000'/>
            <point x='2515.000000' y='1703.000000'/>
            <point x='2514.727539' y='1622.768677'/>
            <point x='2439.000000' y='1623.000000'/>
        </location_points>
        <flag_facing x='1.000000' y='-0.000000'/>
        <building_links>
            <building_link serialise_version='3' building_index='-1' prefab_index='-1' prefab_building_key='' uid='110191296430928784' prefab_uid='0'/>
            <building_link serialise_version='3' building_index='-1' prefab_index='-1' prefab_building_key='' uid='110212897738279460' prefab_uid='0'/>
            <building_link serialise_version='3' building_index='-1' prefab_index='38' prefab_building_key='wall_straight_20_emplacment' uid='0' prefab_uid='0'/>
        </building_links>
        <toggle_slots_links>
            <toggle_slot_link>0</toggle_slot_link>
            <toggle_slot_link>1</toggle_slot_link>
            <toggle_slot_link>2</toggle_slot_link>
            <toggle_slot_link>3</toggle_slot_link>
            <toggle_slot_link>4</toggle_slot_link>
            <toggle_slot_link>5</toggle_slot_link>
            <toggle_slot_link>32</toggle_slot_link>
        </toggle_slots_links>
        <ai_hints_links/>
    </CAPTURE_LOCATION>
    */
                layer.push_str(&format!("
        <entity id=\"{:x}\" name=\"\">",
                    capture_location.id())
                );

                layer.push_str(&format!("
            <ECCaptureLocation type=\"{}\" importance=\"{}\" restore_type=\"{}\" initial_owner_type=\"DeploymentArea\" flag_position=\"{:.5} {:.5}\" min_players=\"{}\" max_players=\"{}\" flag_facing_direction=\"0\"
            destroy_building_on_capture=\"{}\" disable_building_abilities_when_no_original_owner=\"{}\" abilities_affect_globally=\"{}\" is_time_based=\"{}\" script_id=\"{}\"/>",
                    capture_location.database_key(),
                    capture_location.capture_point_type().replace("CAPTURE_LOCATION_", "CLIT_"),
                    capture_location.restore_type().to_lowercase(),
                    capture_location.location().x() - capture_location.location_points()[0].x(),
                    capture_location.location().y() - capture_location.location_points()[0].y(),
                    capture_location.valid_for_min_num_players(),
                    capture_location.valid_for_max_num_players(),
                    capture_location.destroy_building_on_capture(),
                    capture_location.disable_building_abilities_when_no_original_owner(),
                    capture_location.abilities_affect_globally(),
                    capture_location.is_time_based(),
                    capture_location.script_id()
                ));

                layer.push_str(&format!("
            <ECTransform position=\"{:.5} 0 {:.5}\" rotation=\"0 0 0\" scale=\"1 1 1\" pivot=\"0 0 0\"/>",
                    capture_location.location_points()[0].x(),
                    capture_location.location_points()[0].y(),
                ));

                layer.push_str("
            <ECTransform2D/>");

                layer.push_str("
            <ECPolyline>");

                for point in capture_location.location_points() {
                    layer.push_str("
                <polyline closed=\"true\">");

                    // Points are relative to the first point.
                    layer.push_str(&format!("
                    <point x=\"{:.5}\" y=\"{:.5}\"/>",
                        point.x() - capture_location.location_points()[0].x(),
                        point.y() - capture_location.location_points()[0].y()
                    ));

                    layer.push_str("
                </polyline>");

                }

                layer.push_str("
            </ECPolyline>");

                layer.push_str("
        </entity>"
                );
            }
        }

        Ok(layer)
    }
}

impl Default for CaptureLocation {
    fn default() -> Self {
        Self {
            id: rand::thread_rng().gen::<u64>(),
            location: Point2d::default(),
            radius: f32::default(),
            valid_for_min_num_players: u32::default(),
            valid_for_max_num_players: u32::default(),
            capture_point_type: String::default(),
            restore_type: String::default(),
            location_points: Vec::default(),
            database_key: String::default(),
            flag_facing: Point2d::default(),
            destroy_building_on_capture: bool::default(),
            disable_building_abilities_when_no_original_owner: bool::default(),
            abilities_affect_globally: bool::default(),
            building_links: Vec::default(),
            toggle_slots_links: Vec::default(),
            ai_hints_links: Vec::default(),
            script_id: String::default(),
            is_time_based: bool::default(),
        }
    }
}
