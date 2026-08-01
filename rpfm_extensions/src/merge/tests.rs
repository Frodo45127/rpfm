//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::db::DB;
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::schema::{Definition, Field, FieldType};

use super::*;

fn keyed_definition() -> Definition {
    let mut definition = Definition::new(1, None);
    definition.set_fields(vec![
        Field { name: "key".to_owned(), field_type: FieldType::StringU8, is_key: true, ..Default::default() },
        Field { name: "value".to_owned(), field_type: FieldType::I32, ..Default::default() },
        Field { name: "name".to_owned(), field_type: FieldType::StringU8, ..Default::default() },
    ]);
    definition
}

fn keyless_definition() -> Definition {
    let mut definition = Definition::new(1, None);
    definition.set_fields(vec![
        Field { name: "value".to_owned(), field_type: FieldType::I32, ..Default::default() },
    ]);
    definition
}

fn row(key: &str, value: i32, name: &str) -> Vec<DecodedData> {
    vec![DecodedData::StringU8(key.to_owned()), DecodedData::I32(value), DecodedData::StringU8(name.to_owned())]
}

fn table(definition: &Definition, rows: Vec<Vec<DecodedData>>) -> DB {
    let mut db = DB::new(definition, None, "test_tables");
    db.set_data(&rows).unwrap();
    db
}

#[test]
fn row_unique_to_one_source_passes_through_untouched() {
    let definition = keyed_definition();
    let a = table(&definition, vec![row("a", 1, "orig")]);
    let b = table(&definition, vec![row("b", 2, "only_in_b")]);

    let (merged, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], None, &[]).unwrap();

    assert!(conflicts.is_empty());
    assert_eq!(merged.data().len(), 2);
    assert!(merged.data().iter().any(|r| r == &row("a", 1, "orig")));
    assert!(merged.data().iter().any(|r| r == &row("b", 2, "only_in_b")));
}

#[test]
fn edit_in_only_one_source_is_taken_automatically() {
    let definition = keyed_definition();
    let baseline = table(&definition, vec![row("a", 1, "orig")]);
    let a = table(&definition, vec![row("a", 2, "orig")]);
    let b = table(&definition, vec![row("a", 1, "orig")]);

    let (merged, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], Some(&baseline), &[]).unwrap();

    assert!(conflicts.is_empty());
    assert_eq!(merged.data().to_vec(), vec![row("a", 2, "orig")]);
}

#[test]
fn identical_edit_in_both_sources_is_taken_automatically() {
    let definition = keyed_definition();
    let baseline = table(&definition, vec![row("a", 1, "orig")]);
    let a = table(&definition, vec![row("a", 2, "orig")]);
    let b = table(&definition, vec![row("a", 2, "orig")]);

    let (merged, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], Some(&baseline), &[]).unwrap();

    assert!(conflicts.is_empty());
    assert_eq!(merged.data().to_vec(), vec![row("a", 2, "orig")]);
}

#[test]
fn genuine_conflict_is_reported_then_resolved() {
    let definition = keyed_definition();
    let a = table(&definition, vec![row("c", 10, "new")]);
    let b = table(&definition, vec![row("c", 20, "new")]);

    let (_, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], None, &[]).unwrap();

    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].row_key, vec!["c".to_owned()]);
    assert_eq!(conflicts[0].field_name, "value");
    assert_eq!(conflicts[0].candidates.len(), 2);

    let resolutions = vec![MergeResolution { row_key: vec!["c".to_owned()], field_name: "value".to_owned(), chosen_value: "20".to_owned() }];
    let (merged, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], None, &resolutions).unwrap();

    assert!(conflicts.is_empty());
    assert_eq!(merged.data().to_vec(), vec![row("c", 20, "new")]);
}

#[test]
fn table_without_key_columns_falls_back_to_concatenation() {
    let definition = keyless_definition();
    let mut a = DB::new(&definition, None, "test_tables");
    a.set_data(&[vec![DecodedData::I32(1)]]).unwrap();
    let mut b = DB::new(&definition, None, "test_tables");
    b.set_data(&[vec![DecodedData::I32(2)]]).unwrap();

    let (merged, conflicts) = delta_merge_db(&[("a.tsv", &a), ("b.tsv", &b)], None, &[]).unwrap();

    assert!(conflicts.is_empty());
    assert_eq!(merged.data().len(), 2);
}
