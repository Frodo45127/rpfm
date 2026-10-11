//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;
use crate::startpos::cai::cai_test::{game_cai_record, id_translation, test_inputs, translate};
use crate::startpos::layout::GroupView;
use crate::startpos::startpos_test::{compare, test_data_tables, Report, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;

/// Share of the fields of the managers at the game's level the writer must match (0.970). The rest is state the game
/// computes on the first update (we write the constructors' values: finance managers, beliefs, stance empire data,
/// update flags), the religion behaviour and the diplomacy managers' war state.
const MIN_MANAGER_MATCH_RATIO: f64 = 0.97;

/// Reads the objects of a pool list of a game manager as (type code, target or 0, id).
fn pool_objects(layout: &StartposLayout, pool: &GroupView, list: &str) -> Vec<(u32, u32, u32)> {
    let path = format!("{}/CAI_BDI_POOL/{list}", managers_path());
    layout.read(&path, pool.record(list).unwrap()).unwrap().iter().map(|object| {
        let code = object.u32("^+1").unwrap();
        let variant = variant_path(&path, code);
        let component = layout.read(&child_path(&variant, "CAI_BDI_COMPONENT"), object.record("CAI_BDI_COMPONENT").unwrap()).unwrap()[0].u32("^+1").unwrap();
        let properties = layout.read(&child_path(&variant, "CAI_BDI_COMPONENT_PROPERTY_SET"), object.record("CAI_BDI_COMPONENT_PROPERTY_SET").unwrap()).unwrap();
        let target = if matches!(code, CHARACTER_SKILLS_INTENTION | FORCE_SKILLS_INTENTION) { properties[0].u32("^+12").unwrap() } else { 0 };
        (code, target, component)
    }).collect()
}

/// Sorts the intentions of every pool by their property set (type and target), which pairs them for comparison.
fn sort_intentions(record: &mut RecordNode) {
    if record.name() == "CAI_BDI_POOL_INTENTIONS" {
        record.children_mut().sort_by_key(|group| format!("{:?}", group.get(1)));
    }
    for node in record.children_mut().iter_mut().flatten() {
        if let NodeType::Record(child) = node {
            sort_intentions(child);
        }
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_managers_match_the_game_startpos() {
    let inputs = test_inputs();
    let keys = inputs.world.factions().iter().map(|faction| faction.key().as_str()).collect::<Vec<_>>();
    let index = CampaignIndex::new(test_data_tables(), TEST_CAMPAIGN).unwrap();
    let stances = StartingStances::new(test_data_tables(), &index, &keys).unwrap();
    let characters = inputs.plan.characters().iter().map(|characters| characters.iter().map(|character| character.character()).collect()).collect::<Vec<_>>();
    let mobiles = inputs.plan.characters().iter().map(|characters| characters.iter().filter_map(|character| character.mobile()).collect()).collect::<Vec<_>>();
    let manager_inputs = ManagerInputs { layout: &inputs.layout, plan: &inputs.plan, stances: &stances, characters: &characters, mobiles: &mobiles };
    let written = write_managers(&manager_inputs).unwrap();

    // Translate our ids to the game's: world objects by kind, pool objects by manager, type and target.
    let layout = &inputs.layout;
    let path = managers_path();
    let game = game_cai_record("CAI_INTERFACE_MANAGERS");
    let mut translation = id_translation(&inputs);
    let game_managers = layout.read(&path, game).unwrap();
    let ours = layout.read(&path, &written).unwrap();
    assert_eq!(game_managers.len(), ours.len());
    let mut kept = vec![];
    for (position, (game_manager, our_manager)) in game_managers.iter().zip(&ours).enumerate() {
        let faction = |manager: &GroupView| layout.read(&child_path(&path, "CAI_FACTION_MANAGER"), manager.record("CAI_FACTION_MANAGER").unwrap()).unwrap()[0].u32("^+1").unwrap();
        let level = |manager: &GroupView| layout.read(&child_path(&path, "CAI_FACTION_MANAGER"), manager.record("CAI_FACTION_MANAGER").unwrap()).unwrap()[0].u32("^+2").unwrap();
        assert_eq!(faction(game_manager), faction(our_manager));
        if level(game_manager) != level(our_manager) {
            continue;
        }
        kept.push(position);

        let pool_path = child_path(&path, "CAI_BDI_POOL");
        let game_pool = layout.read(&pool_path, game_manager.record("CAI_BDI_POOL").unwrap()).unwrap();
        let our_pool = layout.read(&pool_path, our_manager.record("CAI_BDI_POOL").unwrap()).unwrap();
        for list in ["CAI_BDI_POOL_DESIRES", "CAI_BDI_POOL_BELIEFS", "CAI_BDI_POOL_INTENTIONS"] {
            let theirs = pool_objects(layout, &game_pool[0], list).into_iter().map(|(code, target, id)| ((code, target), id)).collect::<HashMap<_, _>>();
            for (code, target, id) in pool_objects(layout, &our_pool[0], list) {
                let target = translation.get(&target).copied().unwrap_or(target);
                if let Some(game_id) = theirs.get(&(code, target)) {
                    translation.insert(id, *game_id);
                }
            }
        }
    }

    // Compare the kept managers.
    let filter = |record: &RecordNode| {
        let mut record = record.clone();
        let groups = record.children().iter().enumerate().filter(|(position, _)| kept.contains(position)).map(|(_, group)| group.clone()).collect();
        record.set_children(groups);
        sort_intentions(&mut record);
        record
    };
    let mut report = Report::default();
    compare(layout, &filter(&translate(&written, &translation)), &filter(game), &path, &mut report);
    let (matches, total) = report.totals();
    let ratio = matches as f64 / total as f64;
    println!("{} managers: {matches}/{total} fields match. Worst:\n{}", kept.len(), report.worst(40));
    assert!(ratio >= MIN_MANAGER_MATCH_RATIO, "{matches}/{total} ({ratio:.4}) fields match");
}
