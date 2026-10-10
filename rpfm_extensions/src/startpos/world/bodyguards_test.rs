//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::HashMap;

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::{array_items, field_str, field_u32, test_data_tables, test_startpos, world, world_factions, TEST_CAMPAIGN};
use crate::startpos::world::CampaignIndex;
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the commander tables to the fixture: `fact_a` can use a faction unit (an upgrade of the subculture unit),
/// the subculture unit, a culture unit (an upgrade too) and a ship.
fn bodyguard_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "commander_unit_permissions", &["culture_key", "faction_key", "subculture_key", "unit_key"], &[
        &["", "fact_a", "", "royal_guard"],
        &["", "fact_a", "", "guard_ship"],
        &["", "", "sub_a", "nobles"],
        &["cult_a", "", "", "warlord"],
        &["cult_a", "", "", "forbidden"],
    ]);
    add_table(&mut tables, "units_to_groupings_military_permissions", &["unit", "military_group"], &[
        &["royal_guard", "mg_a"],
        &["guard_ship", "mg_a"],
        &["nobles", "mg_a"],
        &["warlord", "mg_a"],
    ]);
    add_table(&mut tables, "technology_unit_upgrades", &["unit", "target_unit"], &[
        &["nobles", "royal_guard"],
        &["nobles", "warlord"],
    ]);
    add_table(&mut tables, "main_units", &["unit", "is_naval", "num_men"], &[
        &["royal_guard", "false", "80"],
        &["guard_ship", "true", "40"],
        &["nobles", "false", "160"],
        &["warlord", "false", "120"],
    ]);
    tables
}

#[test]
fn test_bodyguard_is_the_root_of_the_upgrade_chain() {
    let tables = bodyguard_tables();
    let index = fixture_index(&tables);

    let bodyguard = bodyguard(&tables, index.faction("fact_a").unwrap()).unwrap().unwrap();

    assert_eq!(bodyguard.unit(), "nobles");
    assert_eq!(bodyguard.men(), 160);
}

#[test]
fn test_no_candidates_gives_no_bodyguard() {
    let tables = bodyguard_tables();
    let index = fixture_index(&tables);

    assert_eq!(bodyguard(&tables, index.faction("fact_b").unwrap()).unwrap(), None);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_bodyguards_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();

    // Bodyguards of the generals without an army, by faction.
    let mut expected = HashMap::new();
    for faction in world_factions(world(test_startpos())) {
        let Some(characters) = child_record(faction, "CHARACTER_ARRAY") else { continue };
        for character in array_items(characters, "CHARACTER") {
            if field_str(character, 3) != "general" || field_u32(character, 4) != 0 {
                continue;
            }
            let details = child_record(character, "CHARACTER_DETAILS").and_then(|details| child_record(details, "GENERAL_BODYGUARD_DETAILS")).unwrap();
            expected.insert(field_str(faction, 1).to_owned(), (field_str(details, 0).to_owned(), field_u32(details, 1)));
        }
    }

    let matching = expected.iter()
        .filter(|(faction, (unit, men))| {
            let bodyguard = bodyguard(tables, index.faction(faction).unwrap()).unwrap().unwrap();
            bodyguard.unit() == unit && bodyguard.men() == *men
        })
        .count();

    assert_eq!(expected.len(), 43);
    assert_eq!(matching, 43);
}
