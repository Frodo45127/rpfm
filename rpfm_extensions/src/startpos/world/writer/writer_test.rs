//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Comparison of written records against game-built ones, field by field.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::world::legacy::legacy_test::{linked_character, name_ids, overrides_from_pack};
use crate::startpos::startpos_test::{array_items, compare, field_str, missing_records, Report, test_data_esf, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};

/// Share of the game's `WORLD` fields the writer must match. Raised as the writer covers more records.
const MIN_MATCH_RATIO: f64 = 0.93;

/// `WORLD` records the writer leaves empty on purpose: the shroud fillers (source unknown) and the starting skills
/// of agents (the game's only one is a spy's, rule unknown).
const KNOWN_EMPTY: [&str; 2] = [
    "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CAMPAIGN_SHROUD/CAMPAIGN_SHROUD_FILLERS/",
    "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER/CHARACTER_DETAILS/CAMPAIGN_SKILLS/CAMPAIGN_SKILLS_BLOCK",
];

/// Loads what the world writer needs from the test data: the campaign index, the layouts of the game startpos, the
/// settlement layouts, the region map and the legacy overrides.
pub(crate) fn world_test_data() -> (CampaignIndex, StartposLayout, Vec<SettlementLayout>, RegionMap, HashSet<String>) {
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let map_data = test_data_esf("map_data.esf");
    let settlements = SettlementLayout::all_from_map_data(&map_data, |region| index.settlement_of(region).is_some_and(|settlement| settlement.is_major())).unwrap();
    let region_map = RegionMap::from_map_data(&map_data).unwrap();
    (index, layout, settlements, region_map, overrides_from_pack().into_iter().collect())
}

/// Plans the world with the game startpos's orders of factions, regions and characters (characters linked to their
/// table rows by name). The game takes them from hash tables; using them checks the rules instead of the orders.
pub(crate) fn game_plan(index: &CampaignIndex) -> WorldPlan<'_> {
    let game_world = world(test_startpos());
    let factions = world_factions(game_world).iter().map(|faction| index.faction(field_str(faction, 1)).unwrap()).collect();
    let regions = world_regions(game_world).iter().map(|region| index.region(field_str(region, 1)).unwrap()).collect();
    let mut plan = WorldPlan::with_orders(factions, regions);

    let names = test_data_tables().table("start_pos_characters").unwrap().rows()
        .map(|row| (row.str("ID").unwrap().into_owned(), ["Name", "Surname", "clan_name", "other_name"].iter().map(|column| row.str(column).unwrap().into_owned()).filter(|id| !id.is_empty()).collect::<BTreeSet<_>>()))
        .collect::<HashMap<_, _>>();
    for faction in world_factions(game_world) {
        let key = field_str(faction, 1);
        let order = child_record(faction, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default().into_iter()
            .filter(|character| field_str(character, 3) != "colonel")
            .filter_map(|character| linked_character(index, key, &names, &name_ids(character)))
            .filter_map(|id| index.characters().iter().find(|entry| *entry.id() == id))
            .collect();
        plan.set_character_order(key, order);
    }
    plan
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_world_matches_the_game_startpos() {
    let tables = test_data_tables();
    let (index, layout, settlements, region_map, legacy_overrides) = world_test_data();
    let inputs = WorldInputs { tables, index: &index, layout: &layout, settlements: &settlements, region_map: &region_map, legacy_overrides: &legacy_overrides };
    let plan = game_plan(&index);
    let game_world = world(test_startpos());

    let written = write_world_with_plan(&inputs, &plan).unwrap();
    let mut report = Report::default();
    compare(&layout, &written, game_world, WORLD_PATH, &mut report);

    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    assert!(ratio >= MIN_MATCH_RATIO, "{matches}/{total} ({ratio:.4}) fields match. Worst:\n{}", report.worst(60));

    let missing = missing_records(&written, game_world, WORLD_PATH, &KNOWN_EMPTY);
    assert!(missing.is_empty(), "records the writer leaves empty: {missing:#?}");
}
