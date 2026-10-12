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

use rpfm_lib::files::esf::ESF;

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::tables::StartposTables;
use crate::startpos::world::legacy::legacy_test::{linked_character, name_ids, overrides_from_pack};
use crate::startpos::startpos_test::{array_items, compare, field_str, field_u32, missing_records, Report, test_data_esf, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};

/// Share of the game's `WORLD` fields the writer must match. Raised as the writer covers more records.
const MIN_MATCH_RATIO: f64 = 0.93;

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
    plan_from_startpos(index, test_startpos(), test_data_tables())
}

/// Plans the world with the orders of factions, regions and characters of a game-built startpos (characters linked
/// to their table rows by name).
pub(crate) fn plan_from_startpos<'a>(index: &'a CampaignIndex, startpos: &ESF, tables: &StartposTables) -> WorldPlan<'a> {
    let game_world = world(startpos);
    let factions = world_factions(game_world).iter().filter_map(|faction| index.faction(field_str(faction, 1))).collect();
    let regions = world_regions(game_world).iter().filter_map(|region| index.region(field_str(region, 1))).collect();
    let mut plan = WorldPlan::with_orders(factions, regions);

    let names = tables.table("start_pos_characters").unwrap().rows()
        .map(|row| (row.str("ID").unwrap().into_owned(), ["Name", "Surname", "clan_name", "other_name"].iter().map(|column| row.str(column).unwrap().into_owned()).filter(|id| !id.is_empty()).collect::<BTreeSet<_>>()))
        .collect::<HashMap<_, _>>();
    // Characters married into another faction are listed there, but were created, and numbered, in their own.
    let mut orders = world_factions(game_world).iter().map(|faction| (field_str(faction, 1), vec![])).collect::<HashMap<&str, Vec<(u32, &CharacterEntry)>>>();
    for faction in world_factions(game_world) {
        let key = field_str(faction, 1);
        let origins = std::iter::once(key).chain(index.characters().iter()
            .filter(|entry| entry.husband_faction().as_deref() == Some(key))
            .map(|entry| entry.faction().as_str()));
        let origins = origins.collect::<BTreeSet<_>>();
        for character in child_record(faction, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default() {
            if field_str(character, 3) == "colonel" {
                continue;
            }
            let ids = name_ids(character);
            let Some(entry) = origins.iter()
                .find_map(|origin| linked_character(index, origin, &names, &ids))
                .and_then(|id| index.characters().iter().find(|entry| *entry.id() == id)) else { continue };
            orders.entry(entry.faction().as_str()).or_default().push((field_u32(character, 1), entry));
        }
    }
    for (faction, mut order) in orders {
        order.sort_by_key(|(id, _)| *id);
        plan.set_character_order(faction, order.into_iter().map(|(_, entry)| entry).collect());
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

    let missing = missing_records(&written, game_world, WORLD_PATH, &[]);
    assert!(missing.is_empty(), "records the writer leaves empty: {missing:#?}");
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_rebel_faction_counts_the_regions_without_owner() {
    let (index, layout, ..) = world_test_data();
    let plan = game_plan(&index);
    let path = format!("{WORLD_PATH}/REBEL_FACTION");

    let written = factions::write_rebel_faction(&layout, &plan).unwrap();

    let region_count = |rebels: &RecordNode| {
        let faction = child_record(rebels, "FACTION").unwrap();
        layout.read(&format!("{path}/FACTION"), faction).unwrap()[0].get("FACTION_FLAG_AND_COLOURS@2+14").cloned()
    };
    let game_rebels = child_record(world(test_startpos()), "REBEL_FACTION").unwrap();
    assert_eq!(region_count(&written), region_count(game_rebels));
}
