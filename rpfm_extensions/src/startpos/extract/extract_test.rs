//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::schema::{Definition, Field, FieldType};

use super::*;
use super::compare::{compare_tables, TableComparison};
use crate::startpos::startpos_test::{test_data_esf, test_data_path, test_data_tables, test_startpos, TEST_CAMPAIGN};
use crate::startpos::writer::writer_test::write_test_startpos;

/// Share of the original values each extracted table must get back. Raised as the extraction covers more. The lowest
/// tables lose values the startpos doesn't keep: settlement links of dead characters, and clamped loyalties.
const MIN_TABLE_MATCH_RATIO: f64 = 0.93;

/// Extracts the tables of a startpos and compares them with the test data's tables.
fn extract_and_compare(startpos: &ESF, layout: &StartposLayout) -> Vec<TableComparison> {
    let schema = Schema::load(&test_data_path("schema.ron"), None).unwrap();
    let extracted = extract_tables(startpos, layout, test_data_tables()).unwrap();
    assert_eq!(extracted.campaign(), TEST_CAMPAIGN);
    compare_tables(test_data_tables(), &extracted, &schema).unwrap()
}

/// Checks that every table gets back enough of the original values, and no rows the original tables lack.
pub(crate) fn assert_matches(comparisons: &[TableComparison]) {
    let report = comparisons.iter().map(ToString::to_string).collect::<String>();
    for comparison in comparisons {
        assert!(comparison.match_ratio() >= MIN_TABLE_MATCH_RATIO, "{} gets back too few values:\n{report}", comparison.table());
        assert!(comparison.extra_rows().is_empty(), "{} has rows the tables lack:\n{report}", comparison.table());
    }
}

#[test]
fn test_rows_become_dbs_with_defaults_for_the_other_columns() {
    let mut definition = Definition::new(1, None);
    definition.set_fields(vec![
        Field { name: "key".to_owned(), field_type: FieldType::StringU8, is_key: true, ..Default::default() },
        Field { name: "value".to_owned(), field_type: FieldType::I32, ..Default::default() },
        Field { name: "flag".to_owned(), field_type: FieldType::Boolean, ..Default::default() },
    ]);
    let mut schema = Schema::default();
    schema.add_definition("start_pos_test_tables", &definition);

    let mut rows = TableRows::new(&["key", "flag", "unknown"]);
    rows.push(vec!["a".to_owned(), "true".to_owned(), "ignored".to_owned()]);
    let mut extracted = ExtractedTables { campaign: TEST_CAMPAIGN.to_owned(), tables: BTreeMap::new() };
    extracted.insert("start_pos_test", rows);

    let dbs = extracted.to_dbs(&schema).unwrap();

    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].table_name(), "start_pos_test_tables");
    assert_eq!(dbs[0].data().to_vec(), vec![vec![DecodedData::StringU8("a".to_owned()), DecodedData::I32(0), DecodedData::Boolean(true)]]);
}

#[test]
fn test_tables_without_definition_are_an_error() {
    let mut extracted = ExtractedTables { campaign: TEST_CAMPAIGN.to_owned(), tables: BTreeMap::new() };
    extracted.insert("start_pos_test", TableRows::new(&["key"]));

    assert!(extracted.to_dbs(&Schema::default()).is_err());
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_tables_extracted_from_the_game_startpos_match_its_tables() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    assert_matches(&extract_and_compare(test_startpos(), &layout));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_tables_extracted_from_a_generated_startpos_match_its_tables() {
    let (startpos, layout) = write_test_startpos();
    assert_matches(&extract_and_compare(&startpos, &layout));
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_tables_are_extracted_from_every_game_startpos() {
    let campaigns = std::fs::read_dir(test_data_path("game/data/campaigns")).unwrap().flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert!(!campaigns.is_empty());
    for campaign in campaigns {
        let startpos = test_data_esf(&format!("game/data/campaigns/{campaign}/startpos.esf"));
        let layout = StartposLayout::from_esf(&startpos).unwrap();

        let extracted = extract_tables(&startpos, &layout, test_data_tables()).unwrap();

        assert_eq!(*extracted.campaign(), campaign);
        for table in ["start_pos_factions", "start_pos_regions", "start_pos_settlements", "start_pos_characters"] {
            assert!(!extracted.table(table).unwrap().rows().is_empty(), "{table} of {} is empty", extracted.campaign());
        }
    }
}
