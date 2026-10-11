//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::*;
use crate::startpos::esf_nav::{as_str, child_record, leaves};
use crate::startpos::startpos_test::{array_items, field_str, test_data_tables, test_startpos, world, world_factions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the agent tables to the fixture.
fn agent_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "agent_to_agent_abilities", &["agent", "ability"], &[
        &["general", "command_force"],
        &["colonel", "command_force"],
        &["spy", "hinder_agent"],
    ]);
    add_table(&mut tables, "agent_to_agent_attributes", &["agent", "attribute", "default_value"], &[
        &["spy", "authority", "2"],
        &["spy", "subterfuge", "3"],
        &["spy", "zeal", "1"],
    ]);
    add_table(&mut tables, "character_skill_node_sets", &["agent_key", "campaign_key", "faction_key", "key", "subculture"], &[
        &["general", "other", "", "skills_other_campaign", "sub_a"],
        &["general", "", "fact_b", "skills_fact_b", "sub_a"],
        &["general", "", "", "skills_general_a", "sub_a"],
    ]);
    add_table(&mut tables, "agents", &["key", "faction_total_cap"], &[
        &["general", "10"],
        &["spy", "10"],
        &["colonel", "0"],
    ]);
    add_table(&mut tables, "agent_culture_details", &["agent", "culture"], &[
        &["colonel", "cult_a"],
        &["spy", "cult_a"],
        &["general", "cult_a"],
        &["general", "cult_a"],
        &["spy", "cult_b"],
    ]);
    tables
}

#[test]
fn test_abilities_and_attributes_in_startpos_order() {
    let tables = agent_tables();

    assert_eq!(agent_abilities(&tables, "spy").unwrap(), [false, false, false, false, false, true, false, false, false]);
    assert_eq!(agent_attributes(&tables, "spy").unwrap(), [3, 1, 2]);
    assert_eq!(agent_attributes(&tables, "colonel").unwrap(), [0, 0, 0]);
}

#[test]
fn test_skill_set_of_agent_and_subculture() {
    let tables = agent_tables();
    let index = fixture_index(&tables);
    let faction = index.faction("fact_a").unwrap();

    assert_eq!(skill_set(&tables, &index, "general", faction).unwrap().as_deref(), Some("skills_general_a"));
    assert_eq!(skill_set(&tables, &index, "colonel", faction).unwrap(), None);
}

#[test]
fn test_recruitment_pools_skip_uncapped_agents() {
    let tables = agent_tables();
    let index = fixture_index(&tables);

    assert_eq!(recruitment_pool_agents(&tables, index.faction("fact_a").unwrap()).unwrap(), vec!["spy".to_owned(), "general".to_owned()]);
}

/// Returns the `#1` values of the groups of an array record, by their `#0` key.
fn keyed_values(array: &RecordNode) -> Vec<(String, NodeType)> {
    array.children().iter().map(|group| {
        let values = leaves(group).collect::<Vec<_>>();
        (as_str(values[0]).unwrap().to_owned(), values[1].clone())
    }).collect()
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_agents_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let (mut characters, mut matches, mut pool_matches, mut with_skills) = (0, 0, 0, 0);

    for faction_record in world_factions(world(test_startpos())) {
        let faction = index.faction(field_str(faction_record, 1)).unwrap();

        for character in child_record(faction_record, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default() {
            let agent = field_str(character, 3);
            let details = child_record(character, "CHARACTER_DETAILS").unwrap();

            let abilities = keyed_values(child_record(details, "AGENT_ABILITIES").unwrap()).into_iter()
                .map(|(key, value)| (key, matches!(value, NodeType::Bool(value) if *value.value())))
                .collect::<Vec<_>>();
            let attributes = keyed_values(child_record(details, "AGENT_ATTRIBUTES").unwrap()).into_iter()
                .map(|(key, value)| (key, match value { NodeType::I32(value) => i64::from(*value.value()), other => panic!("{other:?}") }))
                .collect::<Vec<_>>();
            let skills_record = child_record(details, "CAMPAIGN_SKILLS").unwrap();
            let skills = field_str(skills_record, 1);
            let starting = child_record(skills_record, "CAMPAIGN_SKILLS_BLOCK").map(|block| block.children().iter().map(|group| match group.as_slice() {
                [NodeType::Ascii(skill), NodeType::U32(tier), NodeType::F32(indent), NodeType::U32(points)] => (skill.to_owned(), *tier.value(), *indent.value(), *points.value()),
                other => panic!("{other:?}"),
            }).collect::<Vec<_>>()).unwrap_or_default();

            let expected_abilities = ABILITIES.iter().map(|key| key.to_string()).zip(agent_abilities(tables, agent).unwrap()).collect::<Vec<_>>();
            let expected_attributes = ATTRIBUTES.iter().map(|key| key.to_string()).zip(agent_attributes(tables, agent).unwrap()).collect::<Vec<_>>();
            let expected_skills = skill_set(tables, &index, agent, faction).unwrap().unwrap_or_default();
            let expected_starting = starting_skills(tables, &index, &expected_skills, faction).unwrap().into_iter()
                .map(|skill| (skill.skill, skill.tier, skill.indent, skill.points))
                .collect::<Vec<_>>();

            characters += 1;
            matches += usize::from(abilities == expected_abilities && attributes == expected_attributes && skills == expected_skills && starting == expected_starting);
            with_skills += usize::from(!starting.is_empty());
        }

        let mut pools = child_record(faction_record, "CHARACTER_RECRUITMENT_POOL_MANAGER")
            .and_then(|manager| child_record(manager, "CHARACTER_RECRUITMENT_POOL_BLOCK"))
            .map(|block| block.children().iter().filter_map(|group| leaves(group).find_map(as_str).map(str::to_owned)).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut expected = recruitment_pool_agents(tables, faction).unwrap();
        pools.sort();
        expected.sort();
        pool_matches += usize::from(pools == expected);
    }

    assert_eq!(characters, 578);
    assert_eq!(matches, 578);
    assert_eq!(with_skills, 1);
    assert_eq!(pool_matches, 118);
}
