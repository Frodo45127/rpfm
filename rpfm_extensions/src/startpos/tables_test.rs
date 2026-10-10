//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::table::DecodedData;

use super::*;

/// Shorthand for a text value.
pub(crate) fn text(value: &str) -> DecodedData {
    DecodedData::StringU8(value.to_owned())
}

#[test]
fn test_fragments_merge_by_column_name() {
    let mut tables = StartposTables::new();
    tables.add_rows("units", &["key", "men"], &[vec![text("a"), DecodedData::I32(10)]]).unwrap();
    tables.add_rows("units", &["men", "key"], &[vec![DecodedData::I32(20), text("b")]]).unwrap();

    let table = tables.table("units").unwrap();
    let rows = table.rows().map(|row| (row.str("key").unwrap().into_owned(), row.int("men").unwrap())).collect::<Vec<_>>();

    assert_eq!(rows, vec![("a".to_owned(), 10), ("b".to_owned(), 20)]);
}

#[test]
fn test_missing_column_in_fragment_is_an_error() {
    let mut tables = StartposTables::new();
    tables.add_rows("units", &["key", "men"], &[]).unwrap();

    assert!(tables.add_rows("units", &["key"], &[vec![text("a")]]).is_err());
}

#[test]
fn test_missing_table_and_column_are_errors() {
    let mut tables = StartposTables::new();
    tables.add_rows("units", &["key"], &[vec![text("a")]]).unwrap();

    assert!(tables.table("factions").is_err());
    assert!(tables.table("units").unwrap().rows().next().unwrap().str("men").is_err());
}

#[test]
fn test_typed_values() {
    let mut tables = StartposTables::new();
    tables.add_rows("t", &["b", "f", "i"], &[vec![DecodedData::Boolean(true), DecodedData::F32(0.025_000_002), DecodedData::I16(-3)]]).unwrap();
    let row = tables.table("t").unwrap().rows().next().unwrap();

    assert!(row.bool("b").unwrap());
    assert_eq!(row.f32("f").unwrap(), 0.025_000_002);
    assert_eq!(row.int("i").unwrap(), -3);
    assert!(row.bool("f").is_err());
}
