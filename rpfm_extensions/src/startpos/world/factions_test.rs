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

use std::collections::{BTreeSet, HashMap};

use super::*;
use crate::startpos::esf_nav::{as_record, as_u32, child_record, child_records};
use crate::startpos::startpos_test::{field_str, field_u32, test_data_tables, test_startpos, world, world_factions, world_regions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the victory and diplomacy tables to the fixture: `fact_b` is a vassal of `fact_a`.
fn faction_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "start_pos_victory_conditions", &["start_pos_faction", "region", "victory_type"], &[
        &["1", "reg_b1", "long"],
        &["2", "reg_a1", "long"],
        &["1", "reg_r", "long"],
    ]);
    add_table(&mut tables, "start_pos_diplomacy", &["faction1", "faction2", "stance"], &[
        &["2", "1", "vassal"],
        &["1", "2", "neutral"],
    ]);
    tables
}

#[test]
fn test_flags_and_colours() {
    let tables = faction_tables();

    let [normal, republican] = faction_flags(&tables, "fact_a").unwrap();

    assert_eq!(normal, FactionFlag { path: "flags/a".to_owned(), colours: [[255, 0, 0], [0, 255, 0], [0, 0, 255]] });
    assert_eq!(republican.path(), "flags/a_rep");
    assert_eq!(republican.colours(), [[16, 16, 16], [32, 32, 32], [48, 48, 48]]);
}

#[test]
fn test_summary() {
    let tables = faction_tables();
    let index = fixture_index(&tables);

    let overlord = faction_summary(&tables, &index, index.faction("fact_a").unwrap()).unwrap();
    let vassal = faction_summary(&tables, &index, index.faction("fact_b").unwrap()).unwrap();

    assert_eq!(overlord.capital().as_deref(), Some("reg_a1"));
    assert_eq!(overlord.region_count(), 2);
    assert_eq!(overlord.victory_regions(), &vec!["reg_b1".to_owned(), "reg_r".to_owned()]);
    assert_eq!(overlord.subjects(), &vec!["fact_b".to_owned()]);
    assert!(overlord.has_war_or_vassal());
    assert_eq!(overlord.horde(), None);
    assert_eq!(overlord.progenitor().as_deref(), Some("30"));
    assert_eq!(vassal.vassal_of().as_deref(), Some("fact_a"));
    assert_eq!(vassal.horde(), Some(false));
    assert!(!vassal.without_characters());
}

/// Returns the k-th leaf (1-based, records skipped) after the n-th record (1-based) with a name among a record's
/// children. Optional records between them don't change the count.
fn leaf_after<'a>(record: &'a RecordNode, anchor: &str, occurrence: usize, k: usize) -> &'a NodeType {
    let group = &record.children()[0];
    let start = group.iter().enumerate()
        .filter(|(_, node)| as_record(node).is_some_and(|child| child.name() == anchor))
        .nth(occurrence - 1)
        .unwrap().0;
    group[start + 1..].iter().filter(|node| as_record(node).is_none()).nth(k - 1).unwrap()
}

/// Returns the strings of an `AsciiArray` leaf.
fn ascii_array(node: &NodeType) -> Vec<String> {
    match node {
        NodeType::AsciiArray(values) => values.to_vec(),
        other => panic!("not a string array: {other:?}"),
    }
}

/// Reads a flag record: path and nine colour bytes.
fn startpos_flag(record: &RecordNode) -> FactionFlag {
    let bytes = (1..=9).map(|position| as_u32(&record.children()[0][position]).unwrap() as u8).collect::<Vec<_>>();
    FactionFlag {
        path: field_str(record, 0).to_owned(),
        colours: [[bytes[0], bytes[1], bytes[2]], [bytes[3], bytes[4], bytes[5]], [bytes[6], bytes[7], bytes[8]]],
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_factions_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let world = world(test_startpos());

    let region_keys = world_regions(world).iter().map(|region| (field_u32(region, 0), field_str(region, 1).to_owned())).collect::<HashMap<_, _>>();
    let factions = world_factions(world);
    let faction_keys = factions.iter().map(|faction| (field_u32(faction, 0), field_str(faction, 1).to_owned())).collect::<HashMap<_, _>>();
    let id_key = |node: &NodeType, keys: &HashMap<u32, String>| as_u32(node).and_then(|id| keys.get(&id).cloned());
    let flag_value = |node: &NodeType| matches!(node, NodeType::Bool(value) if *value.value());

    let mut mismatches = vec![];
    for faction in &factions {
        let entry = index.faction(field_str(faction, 1)).unwrap();
        let summary = faction_summary(tables, &index, entry).unwrap();
        let flags = child_records(faction, "FACTION_FLAG_AND_COLOURS").map(startpos_flag).collect::<Vec<_>>();
        let horde = child_record(faction, "FACTION_HORDE_MANAGER").map(|manager| matches!(manager.children()[0][0], NodeType::I32(ref value) if *value.value() == 1));

        let checks = [
            ("flags", flags == faction_flags(tables, entry.key()).unwrap().to_vec()),
            ("capital", id_key(leaf_after(faction, "MORGUE", 1, 3), &region_keys) == *summary.capital()),
            ("victory", ascii_array(leaf_after(faction, "MORGUE", 1, 2)).into_iter().collect::<BTreeSet<_>>() == summary.victory_regions().iter().cloned().collect()),
            ("regions", as_u32(leaf_after(faction, "FACTION_FLAG_AND_COLOURS", 2, 14)) == u32::try_from(summary.region_count()).ok()),
            ("client", id_key(leaf_after(faction, "FACTION_FLAG_AND_COLOURS", 2, 1), &faction_keys) == *summary.client_state_of()),
            ("vassal", id_key(leaf_after(faction, "FACTION_FLAG_AND_COLOURS", 2, 2), &faction_keys) == *summary.vassal_of()),
            ("war", flag_value(leaf_after(faction, "FACTION_FLAG_AND_COLOURS", 2, 5)) == summary.has_war_or_vassal()),
            ("characters", flag_value(leaf_after(faction, "PRESTIGE", 1, 1)) == summary.without_characters()),
            ("subjects", ascii_array(leaf_after(faction, "FACTION_ACTION_LOG_BLOCK", 1, 2)) == *summary.subjects()),
            ("progenitor", (as_u32(leaf_after(faction, "FACTION_ACTION_LOG_BLOCK", 1, 11)) != Some(0)) == summary.progenitor().is_some()),
            ("horde", horde == summary.horde()),
        ];
        mismatches.extend(checks.iter().filter(|(_, ok)| !ok).map(|(name, _)| format!("{}: {name}", entry.key())));
    }

    assert_eq!(factions.len(), 118);
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}
