//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use regex::Regex;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use std::collections::{BTreeSet, HashMap};

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{array_items, field, field_str, field_u32, test_data_path, test_data_tables, test_startpos, world, world_factions, TEST_CAMPAIGN};
use crate::startpos::world::world_test::{add_table, fixture_index, fixture_tables};

/// Adds the legacy tables to the fixture.
fn legacy_tables() -> StartposTables {
    let mut tables = fixture_tables();
    add_table(&mut tables, "military_force_legacy_names", &["key", "subculture", "for_army", "campaign"], &[
        &["wolves", "sub_a", "true", ""],
        &["bears", "sub_a", "true", "camp"],
        &["sharks", "sub_a", "false", ""],
        &["eagles", "sub_a", "true", "other"],
    ]);
    add_table(&mut tables, "military_force_legacy_emblems", &["culure_key", "faction_key", "is_army", "key", "subculture_key", "campaign"], &[
        &["", "fact_a", "true", "emblem_faction", "", ""],
        &["cult_a", "", "true", "emblem_culture", "", ""],
        &["", "", "false", "emblem_ship", "sub_a", ""],
        &["cult_b", "", "true", "emblem_other_culture", "", ""],
    ]);
    tables
}

#[test]
fn test_legacies_of_a_faction() {
    let tables = legacy_tables();
    let index = fixture_index(&tables);
    let faction = index.faction("fact_a").unwrap();
    let force = |kind, general: Option<&str>| LegacyForce { kind, general: general.map(str::to_owned) };
    let forces = [
        force(ForceKind::Army, Some("30")),
        force(ForceKind::Army, Some("31")),
        force(ForceKind::Navy, Some("33")),
        force(ForceKind::Army, Some("32")),
        force(ForceKind::GarrisonArmy, None),
        force(ForceKind::GarrisonNavy, None),
    ];
    let overrides = ["31".to_owned(), "32".to_owned()].into_iter().collect();

    let legacies = legacies(&tables, &index, faction, &forces, &overrides).unwrap();
    let summary = legacies.iter().map(|legacy| (legacy.name().as_str(), legacy.override_name().as_str(), legacy.emblem().as_str(), legacy.number())).collect::<Vec<_>>();

    assert_eq!(summary, vec![
        ("military_force_legacy_names_localised_name_wolvessub_a", "", "emblem_faction", 1),
        ("start_pos_characters_legacy_override_31", "start_pos_characters_legacy_override_31", "emblem_culture", 1),
        ("military_force_legacy_names_localised_name_sharkssub_a", "", "emblem_ship", 1),
        ("start_pos_characters_legacy_override_32", "start_pos_characters_legacy_override_32", "", 2),
        ("random_localisation_strings_string_military_force_legacy_name_garrison_army", "", "", 1),
        ("random_localisation_strings_string_military_force_legacy_name_garrison_navy", "", "", 1),
    ]);
}

#[test]
fn test_legacy_skill_set() {
    let tables = legacy_tables();
    let index = fixture_index(&tables);
    let mut faction = index.faction("fact_a").unwrap().clone();
    faction.subculture = "att_sub_cult_roman_west".to_owned();

    assert_eq!(legacy_skill_set(&faction, false), "att_army_legacy_roman_west");
    assert_eq!(legacy_skill_set(&faction, true), "att_navy_legacy_roman_west");
}

/// Reads the characters with a `legacy_override` text from the `start_pos_characters` XML the Assembly Kit stores
/// in the campaign pack.
pub(crate) fn overrides_from_pack() -> BTreeSet<String> {
    let pack = String::from_utf8_lossy(&std::fs::read(test_data_path("main_attila.pack")).unwrap()).into_owned();
    let record = Regex::new(r"(?s)<start_pos_characters [^>]*>(.*?)</start_pos_characters>").unwrap();
    let id = Regex::new(r"<ID>(\d+)</ID>").unwrap();
    let text = Regex::new(r"<legacy_override[^>]*>([^<]+)</legacy_override>").unwrap();
    record.captures_iter(&pack)
        .filter_map(|record| text.is_match(&record[1]).then(|| id.captures(&record[1]).map(|id| id[1].to_owned())).flatten())
        .collect()
}

/// Returns the name ids of a startpos character.
pub(crate) fn name_ids(character: &RecordNode) -> BTreeSet<String> {
    let names = child_record(character, "CHARACTER_DETAILS")
        .and_then(|details| child_record(details, "CHARACTER_NAME"))
        .and_then(|name| child_record(name, "NAMES_BLOCK"))
        .unwrap();
    names.children().iter()
        .filter_map(|group| group.iter().filter_map(as_record).find(|record| record.name() == "CAMPAIGN_LOCALISATION"))
        .map(|localisation| field_str(localisation, 0).trim_start_matches("names_name_").to_owned())
        .filter(|id| !id.is_empty())
        .collect()
}

/// Finds the living table character of a faction with the given startpos name ids. The startpos drops names it
/// can't resolve, so the ids only have to include the table's first name and be among the table's names.
pub(crate) fn linked_character(index: &CampaignIndex, faction: &str, names: &HashMap<String, BTreeSet<String>>, ids: &BTreeSet<String>) -> Option<String> {
    let candidates = index.characters_of(faction)
        .filter(|entry| entry.death_type().is_empty() && ids.is_subset(&names[entry.id()]))
        .filter(|entry| names[entry.id()].iter().any(|name| ids.contains(name)))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [entry] => Some(entry.id().to_owned()),
        _ => candidates.iter().find(|entry| names[entry.id()] == *ids).map(|entry| entry.id().to_owned()),
    }
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_legacies_real_data() {
    let tables = test_data_tables();
    let index = CampaignIndex::new(tables, TEST_CAMPAIGN).unwrap();
    let overrides = overrides_from_pack().into_iter().collect();
    let names = tables.table("start_pos_characters").unwrap().rows()
        .map(|row| (row.str("ID").unwrap().into_owned(), ["Name", "Surname", "clan_name", "other_name"].iter().map(|column| row.str(column).unwrap().into_owned()).filter(|id| !id.is_empty()).collect::<BTreeSet<_>>()))
        .collect::<HashMap<_, _>>();

    let (mut forces_checked, mut mismatches) = (0, vec![]);
    for faction_record in world_factions(world(test_startpos())) {
        let faction = index.faction(field_str(faction_record, 1)).unwrap();
        let characters = child_record(faction_record, "CHARACTER_ARRAY").map(|array| array_items(array, "CHARACTER")).unwrap_or_default();
        let forces = child_record(faction_record, "ARMY_ARRAY").map(|array| array_items(array, "MILITARY_FORCE")).unwrap_or_default();

        let mut inputs = vec![];
        for force in &forces {
            let commander = characters.iter().find(|character| field_u32(character, 1) == field_u32(force, 1)).unwrap();
            let navy = matches!(field(force, 19), NodeType::Bool(value) if *value.value());
            let general = field_str(commander, 3) == "general";
            let ids = name_ids(commander);
            let table_id = general.then(|| linked_character(&index, faction.key(), &names, &ids)).flatten();
            let kind = match (general, navy) {
                (true, false) => ForceKind::Army,
                (true, true) => ForceKind::Navy,
                (false, false) => ForceKind::GarrisonArmy,
                (false, true) => ForceKind::GarrisonNavy,
            };
            inputs.push(LegacyForce { kind, general: table_id });
        }

        let generated = legacies(tables, &index, faction, &inputs, &overrides).unwrap();
        let army_names = name_pool(tables, &index, faction, true).unwrap();
        let navy_names = name_pool(tables, &index, faction, false).unwrap();
        let army_emblems = emblem_pool(tables, &index, faction, true).unwrap();
        let navy_emblems = emblem_pool(tables, &index, faction, false).unwrap();

        for ((force, input), legacy) in forces.iter().zip(&inputs).zip(&generated) {
            let record = child_record(force, "MILITARY_FORCE_LEGACY").unwrap();
            let name = field_str(child_record(record, "CAMPAIGN_LOCALISATION").unwrap(), 0);
            let skills = field_str(child_record(record, "CAMPAIGN_SKILLS").unwrap(), 1);
            let navy = matches!(input.kind, ForceKind::Navy | ForceKind::GarrisonNavy);

            // Drawn names and emblems are RNG choices: they only have to come from the right pool.
            let name_ok = if legacy.override_name().is_empty() && matches!(input.kind, ForceKind::Army | ForceKind::Navy) {
                (if navy { &navy_names } else { &army_names }).iter().any(|pool_name| pool_name == name)
            } else {
                name == legacy.name()
            };
            let emblem = field_str(record, 5);
            let emblem_ok = if legacy.emblem().is_empty() {
                emblem.is_empty()
            } else {
                (if navy { &navy_emblems } else { &army_emblems }).iter().any(|pool_emblem| pool_emblem == emblem)
            };

            forces_checked += 1;
            if !(name_ok && emblem_ok
                && field_str(record, 3) == legacy.override_name()
                && field_u32(record, 4) == input.kind as u32
                && field_u32(record, 6) == legacy.number()
                && skills == legacy_skill_set(faction, navy)) {
                mismatches.push((faction.key().to_owned(), name.to_owned()));
            }
        }
    }

    assert_eq!(forces_checked, 343);
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}
