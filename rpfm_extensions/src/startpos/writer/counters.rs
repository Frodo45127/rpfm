//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the world's next-id counters (`CAMPAIGN_COMMAND_QUEUE_INDEX_TABLES`): each counter is one past the
//! highest id of its kind in the startpos, so objects the game creates while playing get new ids. Counters of kinds
//! a startpos doesn't hold keep their layout values.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::MODEL_PATH;
use crate::startpos::esf_nav::{as_u32, child_records};
use crate::startpos::layout::{child_path, StartposLayout};

#[cfg(test)] #[path = "counters_test.rs"] mod counters_test;

/// Where the ids of each counter are: the counter's position, and the records (by path under `CAMPAIGN_MODEL`) and
/// child positions holding them.
const COUNTERS: [(usize, &[(&str, usize)]); 19] = [
    (0, &[("WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER", 1)]),
    (1, &[("WORLD/FACTION_ARRAY/FACTION", 0)]),
    (2, &[
        ("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/REGION_SLOT_MANAGER/REGION_SLOT_ARRAY/REGION_SLOT/GARRISON_RESIDENCE", 0),
        ("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/SETTLEMENT/GARRISON_RESIDENCE", 0),
    ]),
    (3, &[("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/REGION_SLOT_MANAGER/REGION_SLOT_ARRAY/REGION_SLOT", 0)]),
    (4, &[("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/UNIT_CONTAINER/UNITS_ARRAY/UNIT", 0)]),
    (5, &[
        ("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/REGION_SLOT_MANAGER/REGION_SLOT_ARRAY/REGION_SLOT/REGION_BUILDING_MANAGER/BUILDING_BASE", 0),
        ("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_HORDE_DETAILS/MILITARY_FORCE_SLOT_MANAGER/SLOT_BLOCK/MILITARY_FORCE_SLOT/MILITARY_FORCE_BUILDING_MANAGER/BUILDING_BASE", 0),
    ]),
    (6, &[("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION", 0)]),
    (7, &[("WORLD/REGION_MANAGER/REGIONS_ARRAY/REGION/SETTLEMENT", 1)]),
    (9, &[("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE", 0)]),
    (12, &[
        ("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_HORDE_DETAILS/RECRUITMENT_MANAGER", 0),
        ("WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE/FACTION_PROVINCE_MANAGER_ARRAY/FACTION_PROVINCE_MANAGER/RECRUITMENT_MANAGER", 0),
        ("WORLD/SEA_REGION_MANAGER/SEA_REGIONS_BLOCK/SEA_REGION/FACTION_SEA_REGION_MANAGER_BLOCK/FACTION_SEA_REGION_MANAGER/RECRUITMENT_MANAGER", 0),
    ]),
    (14, &[("CAMPAIGN_TRADE_MANAGER/INTERNATIONAL_TRADE_ROUTES/FACTION_INTERNATIONAL_TRADE_ROUTES_ARRAY/INTERNATIONAL_TRADE_ROUTE", 0)]),
    (15, &[("WORLD/FACTION_ARRAY/FACTION/GOVERNMENT/CHARACTER_POSTS/CHARACTER_POST", 0)]),
    (17, &[("WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE", 0)]),
    (18, &[("WORLD/PROVINCE_MANAGER/PROVINCE_ARRAY/PROVINCE/FACTION_PROVINCE_MANAGER_ARRAY/FACTION_PROVINCE_MANAGER", 0)]),
    (19, &[("WORLD/SEA_REGION_MANAGER/SEA_REGIONS_BLOCK/SEA_REGION", 0)]),
    (20, &[("WORLD/SEA_REGION_MANAGER/SEA_REGIONS_BLOCK/SEA_REGION/FACTION_SEA_REGION_MANAGER_BLOCK/FACTION_SEA_REGION_MANAGER", 0)]),
    (22, &[("WORLD/FAMILY_TREE/FAMILY_MEMBER", 0)]),
    (23, &[("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE/MILITARY_FORCE_HORDE_DETAILS/MILITARY_FORCE_SLOT_MANAGER/SLOT_BLOCK/MILITARY_FORCE_SLOT", 0)]),
    (24, &[("WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE", 0)]),
];

/// Writes the next-id counters of a campaign model.
///
/// # Arguments
///
/// * `layout` - Layout of the startpos.
/// * `sections` - Records of the campaign model holding ids (`WORLD`, `CAMPAIGN_TRADE_MANAGER`).
///
/// # Errors
///
/// Returns an error if the layout lacks the record.
pub fn write_index_tables(layout: &StartposLayout, sections: &[&RecordNode]) -> Result<RecordNode> {
    let mut record = layout.builder(&child_path(MODEL_PATH, "CAMPAIGN_COMMAND_QUEUE_INDEX_TABLES"))?;
    let group = record.group()?;
    for (counter, fields) in COUNTERS {
        let highest = fields.iter().filter_map(|(path, position)| highest_id(sections, path, *position)).max().unwrap_or_default();
        group.set_u32(&format!("^+{}", counter + 1), highest + 1)?;
    }
    record.build()
}

/// Returns the highest id at a child position of the records at a path, if any record has it.
fn highest_id(sections: &[&RecordNode], path: &str, position: usize) -> Option<u32> {
    let (section, rest) = path.split_once('/').unwrap_or((path, ""));
    let mut records = sections.iter().copied().filter(|record| record.name() == section).collect::<Vec<_>>();
    for name in rest.split('/').filter(|name| !name.is_empty()) {
        records = records.into_iter().flat_map(|record| child_records(record, name)).collect();
    }
    records.iter()
        .flat_map(|record| record.children())
        .filter_map(|group| group.get(position).and_then(as_u32))
        .max()
}
