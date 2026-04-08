//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutierrez Gonzalez. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for Assembly Kit table data parsing.

use serde_xml_rs::from_reader;

use crate::integrations::assembly_kit::table_data::RawTableField;

use super::*;

#[test]
fn test_parse_raw_table_field() {
    let xml = "<datafield field_name=\"key\">val</datafield>";
    let field: RawTableField = from_reader(xml.as_bytes()).unwrap();
    assert_eq!(field.field_name, "key");
    assert_eq!(field.field_data, "val");
    assert!(field.state.is_none());
}

#[test]
fn test_parse_raw_table_field_with_state() {
    let xml = "<datafield field_name=\"key\" state=\"1\">val</datafield>";
    let field: RawTableField = from_reader(xml.as_bytes()).unwrap();
    assert_eq!(field.field_name, "key");
    assert_eq!(field.state, Some("1".to_string()));
}

#[test]
fn test_parse_raw_table_from_ak_file() {
    let ak_folder = Path::new("../test_files/ak_test");
    let test_def_path = ak_folder.join("TWaD__kv_battle_ai_ability_usage_variables.xml");

    let definition = RawDefinition::read(&test_def_path, 2).unwrap();
    let table = RawTable::read(&definition, ak_folder, 2).unwrap();

    assert_eq!(table.rows.len(), 5);
    assert_eq!(table.rows[0].fields.len(), 3);

    let key_field = table.rows[0].fields.iter().find(|f| f.field_name == "key").unwrap();
    assert_eq!(key_field.field_data, "battle_currency_saving_max_added_amount");
}
