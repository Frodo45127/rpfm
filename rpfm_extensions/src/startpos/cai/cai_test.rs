//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Helpers shared by the AI writer's tests.

use std::collections::HashMap;

use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::map::{pathfinder_nodes, MapRegion, PathfinderNode};
use super::plan::{CaiPlan, PlanInputs};
use super::technologies::faction_technologies;
use super::world_view::WorldView;
use crate::startpos::startpos_test::{compare, test_data_tables, world, Report, TEST_CAMPAIGN};
use super::CAI_WORLD_PATH;
use crate::startpos::world::CampaignIndex;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::layout::{child_path, GroupView, StartposLayout};
use crate::startpos::startpos_test::{test_data_esf, test_startpos};

/// Returns the regions of the test map and their pathfinder nodes.
pub(crate) fn map_inputs() -> (Vec<MapRegion>, HashMap<usize, Vec<PathfinderNode>>) {
    let map_regions = MapRegion::all_from_map_data(&test_data_esf("map_data.esf")).unwrap();
    let nodes = pathfinder_nodes(&test_data_esf("hlp_data.esf")).unwrap();
    (map_regions, nodes)
}

/// Returns a list of the game startpos's `CAI_WORLD`.
pub(crate) fn game_cai_world_list(name: &str) -> &'static RecordNode {
    child_record(game_cai_record("CAI_WORLD"), name).unwrap()
}

/// Returns a record of the game startpos's `CAI_INTERFACE`.
pub(crate) fn game_cai_record(name: &str) -> &'static RecordNode {
    let mut record = as_record(test_startpos().root_node()).unwrap();
    for name in ["CAMPAIGN_ENV", "CAMPAIGN_MODEL", "CAI_INTERFACE", name] {
        record = child_record(record, name).unwrap();
    }
    record
}

/// Returns the first leaf of a group's child record holding a single value (like `OWNED_DIRECT`).
pub(crate) fn single_field_u32(layout: &StartposLayout, parent: &str, group: &GroupView, key: &str) -> u32 {
    layout.read(&child_path(parent, key), group.record(key).unwrap()).unwrap()[0].u32("^+1").unwrap()
}

/// What the AI writer's tests build from: the game startpos's layouts and world, the test map, and the id plan.
pub(crate) struct TestInputs {
    pub(crate) layout: StartposLayout,
    pub(crate) map_regions: Vec<MapRegion>,
    pub(crate) nodes: HashMap<usize, Vec<PathfinderNode>>,
    pub(crate) world: WorldView,
    pub(crate) technologies: Vec<Vec<String>>,
    pub(crate) plan: CaiPlan,
}

/// Builds the AI writer's inputs from the game startpos's world, so ids line up with the game's.
pub(crate) fn test_inputs() -> TestInputs {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let (map_regions, nodes) = map_inputs();
    let world = WorldView::new(&layout, world(test_startpos())).unwrap();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let technologies = world.factions().iter()
        .map(|faction| faction_technologies(test_data_tables(), TEST_CAMPAIGN, index.faction(faction.key()).unwrap()).unwrap())
        .collect::<Vec<_>>();
    let technology_counts = technologies.iter().map(Vec::len).collect::<Vec<_>>();
    let plan = CaiPlan::new(&PlanInputs { map_regions: &map_regions, world: &world, technology_counts: &technology_counts, trade_splines: 2486 });
    TestInputs { layout, map_regions, nodes, world, technologies, plan }
}

/// Lists whose components the plan numbers differently from the game, which creates beliefs in between.
const SHIFTED_LISTS: [&str; 9] = [
    "CAI_WORLD_CHARACTERS", "CAI_WORLD_RESOURCE_MOBILES", "CAI_WORLD_UNITS", "CAI_WORLD_BUILDING_SLOTS", "CAI_PROVINCES",
    "CAI_FACTION_PROVINCES", "CAI_FACTION_SEA_REGIONS", "CAI_WORLD_GOVERNORSHIPS", "CAI_CHARACTER_POSTS",
];

/// Returns the game's id of each planned id that differs: the n-th id of a kind in the plan is the n-th of the game.
pub(crate) fn id_translation(inputs: &TestInputs) -> HashMap<u32, u32> {
    let plan = &inputs.plan;
    let characters = plan.characters().iter().flatten();
    let ours: [Vec<u32>; 9] = [
        characters.clone().map(|character| character.character()).collect(),
        characters.clone().filter_map(|character| character.mobile()).collect(),
        characters.clone().flat_map(|character| character.units().to_vec()).collect(),
        characters.flat_map(|character| character.horde_slots().to_vec()).collect(),
        plan.provinces().iter().map(|province| province.province()).collect(),
        plan.provinces().iter().flat_map(|province| province.faction_provinces().to_vec()).collect(),
        plan.faction_seas().iter().flatten().copied().collect(),
        plan.governorships().to_vec(),
        plan.posts().iter().flatten().copied().collect(),
    ];

    let mut translation = HashMap::new();
    for (name, ours) in SHIFTED_LISTS.iter().zip(ours) {
        let path = child_path(CAI_WORLD_PATH, name);
        let mut theirs = inputs.layout.read(&path, game_cai_world_list(name)).unwrap().iter()
            .map(|group| inputs.layout.read(&child_path(&path, "CAI_BDI_COMPONENT"), group.record("CAI_BDI_COMPONENT").unwrap()).unwrap()[0].u32("^+1").unwrap())
            .collect::<Vec<_>>();
        theirs.sort_unstable();

        // Building slots also hold the settlements' slots, created before the horde slots.
        let theirs = if *name == "CAI_WORLD_BUILDING_SLOTS" { theirs[theirs.len() - ours.len()..].to_vec() } else { theirs };
        translation.extend(ours.into_iter().zip(theirs).filter(|(ours, theirs)| ours != theirs));
    }
    translation
}

/// Returns a copy of a record with the planned ids its unsigned leaves hold replaced by the game's.
pub(crate) fn translate(record: &RecordNode, translation: &HashMap<u32, u32>) -> RecordNode {
    let mut record = record.clone();
    let children = record.children().iter().map(|group| group.iter().map(|node| match node {
        NodeType::Record(child) => NodeType::Record(Box::new(translate(child, translation))),
        NodeType::U32(value) => {
            let mut value = value.clone();
            let id = *value.value();
            value.set_value(translation.get(&id).copied().unwrap_or(id));
            NodeType::U32(value)
        },
        NodeType::U32Array(values) => {
            let mut values = values.clone();
            let ids = values.value().iter().map(|id| translation.get(id).copied().unwrap_or(*id)).collect();
            values.set_value(ids);
            NodeType::U32Array(values)
        },
        node => node.clone(),
    }).collect()).collect();
    record.set_children(children);
    record
}

/// Compares a written `CAI_WORLD` list with the game's (with planned ids translated to the game's), printing the worst
/// fields, and checks the share of fields that match.
pub(crate) fn compare_list(inputs: &TestInputs, written: &RecordNode, name: &str, min_ratio: f64) {
    let mut report = Report::default();
    let written = translate(written, &id_translation(inputs));
    compare(&inputs.layout, &written, game_cai_world_list(name), &child_path(CAI_WORLD_PATH, name), &mut report);
    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    println!("{name}: {matches}/{total} fields match. Worst:\n{}", report.worst(25));
    assert!(ratio >= min_ratio, "{name}: {matches}/{total} ({ratio:.4}) fields match");
}
