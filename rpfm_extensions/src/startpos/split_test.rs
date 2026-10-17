//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeSet;

use rpfm_lib::files::{Container, DecodeableExtraData, RFileDecoded};
use rpfm_lib::files::pack::Pack;
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::games::supported_games::{SupportedGames, KEY_ATTILA};
use rpfm_lib::integrations::assembly_kit::table_data::RawTable;
use rpfm_lib::integrations::assembly_kit::table_definition::RawDefinition;
use rpfm_lib::schema::{Definition, Field, FieldType};

use super::*;
use crate::startpos::startpos_test::{test_data_path, TEST_CAMPAIGN};
use crate::startpos::tables::tables_test::text;

/// Returns a definition with string columns, the first one the key, and references by column.
fn definition(columns: &[&str], references: &[(&str, &str, &str)]) -> Definition {
    let mut definition = Definition::new(1, None);
    definition.set_fields(columns.iter().enumerate().map(|(position, column)| {
        let reference = references.iter().find(|(name, ..)| name == column).map(|(_, table, target)| (table.to_string(), target.to_string()));
        Field { name: column.to_string(), field_type: FieldType::StringU8, is_key: position == 0, is_reference: reference, ..Default::default() }
    }).collect());
    definition
}

/// Returns a table with the given definition and rows.
fn db(name: &str, definition: &Definition, rows: &[&[&str]]) -> DB {
    let mut db = DB::new(definition, None, &format!("{name}_tables"));
    db.set_data(&rows.iter().map(|row| row.iter().map(|value| text(value)).collect()).collect::<Vec<_>>()).unwrap();
    db
}

/// Returns the keys of a table's rows.
fn keys(db: &DB) -> Vec<String> {
    db.data().iter().map(|row| row[0].data_to_string().into_owned()).collect()
}

#[test]
fn test_rows_follow_their_campaign_and_references() {
    let factions = definition(&["ID", "campaign"], &[]);
    let characters = definition(&["ID", "faction"], &[("faction", "start_pos_factions", "ID")]);
    let links = definition(&["character", "settlement"], &[("character", "start_pos_characters", "ID"), ("settlement", "start_pos_settlements", "id")]);
    let deals = definition(&["id", "instigator", "action"], &[("instigator", "start_pos_factions", "ID"), ("action", "start_pos_diplomacy_deal_actions", "id")]);
    let actions = definition(&["id"], &[]);
    let mut schema = Schema::default();
    for (name, definition) in [("factions", &factions), ("characters", &characters), ("character_to_settlements", &links), ("diplomacy_deals", &deals), ("diplomacy_deal_actions", &actions)] {
        schema.add_definition(&format!("start_pos_{name}_tables"), definition);
    }

    let mut tables = vec![
        db("start_pos_factions", &factions, &[&["1", "camp_a"], &["2", "camp_b"]]),
        db("start_pos_characters", &characters, &[&["10", "1"], &["11", "2"], &["12", "3"]]),
        db("start_pos_character_to_settlements", &links, &[&["10", "99"], &["11", "98"]]),
        db("start_pos_diplomacy_deals", &deals, &[&["20", "1", "move"], &["21", "2", "stay"]]),
        db("start_pos_diplomacy_deal_actions", &actions, &[&["move"], &["stay"], &["unused"]]),
    ];
    let mut all = StartposTables::new();
    for table in &tables {
        all.add_db(table).unwrap();
    }

    let rows = CampaignRows::new(&all, &schema, "camp_a").unwrap();
    let removed = tables.iter_mut().map(|table| rows.retain(table).unwrap()).collect::<Vec<_>>();

    assert_eq!(removed, vec![1, 2, 1, 1, 2]);
    assert_eq!(tables.iter().map(keys).collect::<Vec<_>>(), vec![vec!["1"], vec!["10"], vec!["10"], vec!["20"], vec!["move"]]);
}

#[test]
fn test_other_tables_are_left_as_they_are() {
    let definition = definition(&["key", "campaign"], &[]);
    let mut table = db("campaigns", &definition, &[&["a", "camp_b"]]);

    let removed = CampaignRows::default().retain(&mut table).unwrap();

    assert_eq!(removed, 0);
    assert_eq!(table.data().len(), 1);
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_assembly_kit_tables_split_like_bob() {
    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();
    let schema = Schema::load(&test_data_path("schema.ron"), None).unwrap();

    // Every campaign's rows, from the Assembly Kit.
    let raw_folder = test_data_path("game/assembly_kit/raw_data/db");
    let mut assembly_kit = vec![];
    for entry in std::fs::read_dir(&raw_folder).unwrap().flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let Some(table) = file_name.strip_suffix(".xml").filter(|table| table.starts_with(STARTPOS_TABLE_PREFIX)) else { continue };
        let raw_definition = RawDefinition::read(&raw_folder.join(format!("TWaD_{table}.xml")), *game.raw_db_version()).unwrap();
        let raw_table = RawTable::read(&raw_definition, &raw_folder, *game.raw_db_version()).unwrap();
        let definition = schema.definitions_by_table_name(&format!("{table}_tables")).and_then(|definitions| definitions.first());
        assembly_kit.push(raw_table.to_db(definition).unwrap());
    }
    let mut all = StartposTables::new();
    for table in &assembly_kit {
        all.add_db(table).unwrap();
    }

    // The rows BOB exports for the campaign.
    let mut bob = Pack::read_and_merge(&[test_data_path("main_attila.pack")], game, true, false, false).unwrap();
    let mut extra = DecodeableExtraData::default();
    extra.set_schema(Some(&schema));
    let extra = Some(extra);
    let mut bob_keys = HashMap::new();
    for path in bob.files().keys().filter(|path| path.starts_with("db/start_pos_")).cloned().collect::<Vec<_>>() {
        let file = bob.file_mut(&path, false).unwrap();
        if let Ok(Some(RFileDecoded::DB(db))) = file.decode(&extra, false, true) {
            bob_keys.insert(db.table_name().to_owned(), row_keys(&db));
        }
    }

    let rows = CampaignRows::new(&all, &schema, TEST_CAMPAIGN).unwrap();
    for mut table in assembly_kit {
        rows.retain(&mut table).unwrap();
        let expected = bob_keys.get(table.table_name()).cloned().unwrap_or_default();
        assert_eq!(row_keys(&table), expected, "{} differs from BOB's", table.table_name());
    }
}

/// Returns the key values of each row of a table, in column name order (definitions order them differently), with
/// numbers normalised so integers and floats compare.
fn row_keys(db: &DB) -> BTreeSet<Vec<String>> {
    let fields = db.definition().fields_processed();
    let mut keys = db.definition().key_column_positions();
    keys.sort_by_key(|&position| fields[position].name().to_owned());
    db.data().iter().map(|row| keys.iter().map(|&position| match &row[position] {
        DecodedData::F32(value) => value.to_string(),
        other => other.data_to_string().into_owned(),
    }).collect()).collect()
}
