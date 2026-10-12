//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the campaign map data (`CAMPAIGN_MAP_DATA`): the map's paths and its provinces.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use crate::startpos::layout::child_path;
use crate::startpos::world::writer::provinces::campaign_provinces;
use crate::startpos::world::writer::{WorldInputs, WorldPlan};

/// Path of the map data.
const MAP_DATA_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAMPAIGN_MAP_DATA";

/// Path of the provinces of the map data.
const PROVINCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAMPAIGN_MAP_DATA/PROVINCES_DATA/PROVINCE_DATA_ARRAY/PROVINCE_DATA";

/// Folder of the campaign maps.
const MAPS_FOLDER: &str = "campaign_maps";

/// Folder of a campaign map's display files.
const DISPLAY_FOLDER: &str = "display";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the map data of a campaign: the map's folders and key, and one entry per province, with its regions and its
/// main region (the one with a major settlement, else the first one).
///
/// # Arguments
///
/// * `inputs` - Tables, startpos rows, layouts and map data.
/// * `plan` - Order and runtime ids of the world's objects.
/// * `map` - Campaign map key.
///
/// # Errors
///
/// Returns an error if a table, column or layout is missing.
pub fn write_map_data(inputs: &WorldInputs, plan: &WorldPlan, map: &str) -> Result<RecordNode> {
    let layout = inputs.layout;
    let provinces_path = child_path(MAP_DATA_PATH, "PROVINCES_DATA");
    let mut provinces = layout.builder(&child_path(&provinces_path, "PROVINCE_DATA_ARRAY"))?;
    for (province, regions) in campaign_provinces(inputs.tables, plan)? {
        let mut region_array = layout.builder(&child_path(PROVINCE_PATH, "PROVINCE_DATA_ARRAY"))?;
        for region in &regions {
            region_array.push_group()?.set_str("^+1", region)?;
        }

        let main_region = regions.iter()
            .find(|region| inputs.index.settlement_of(region).is_some_and(|settlement| settlement.is_major()))
            .or(regions.first())
            .map(String::as_str)
            .unwrap_or_default();
        let mut data = layout.builder(PROVINCE_PATH)?;
        data.group()?
            .set_str("^+1", &province)?
            .set_record("PROVINCE_DATA_ARRAY", Some(region_array.build()?))?
            .set_str("PROVINCE_DATA_ARRAY+1", main_region)?;
        provinces.push_group()?.set_record("PROVINCE_DATA", Some(data.build()?))?;
    }

    let mut provinces_data = layout.builder(&provinces_path)?;
    provinces_data.group()?.set_record("PROVINCE_DATA_ARRAY", Some(provinces.build()?))?;

    let mut map_data = layout.builder(MAP_DATA_PATH)?;
    map_data.group()?
        .set_str("^+1", &format!("{MAPS_FOLDER}/{map}"))?
        .set_str("^+2", &format!("{MAPS_FOLDER}/{map}/{DISPLAY_FOLDER}"))?
        .set_str("^+3", map)?
        .set_record("PROVINCES_DATA", Some(provinces_data.build()?))?;
    map_data.build()
}
