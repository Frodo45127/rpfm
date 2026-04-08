//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutierrez Gonzalez. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for Assembly Kit table definition parsing.

use crate::integrations::assembly_kit::table_definition::RawDefinitionV0;

use super::*;

#[test]
fn test_parse_v2_definition() {
    let ak_folder = Path::new("../test_files/ak_test");
    let test_def_path = ak_folder.join("TWaD__kv_battle_ai_ability_usage_variables.xml");

    let definition = RawDefinition::read(&test_def_path, 2).unwrap();
    assert_eq!(definition.name, Some("_kv_battle_ai_ability_usage_variables.xml".to_string()));
    assert_eq!(definition.fields.len(), 3);

    let key_field = definition.fields.iter().find(|f| f.name == "key").unwrap();
    assert_eq!(key_field.field_type, "text");
    assert_eq!(key_field.primary_key, "1");

    let value_field = definition.fields.iter().find(|f| f.name == "value").unwrap();
    assert_eq!(value_field.field_type, "double");
}

#[test]
fn test_parse_v0_xsd_definition() {
    let ak_folder = Path::new("../test_files/ak_test");
    let test_xsd_path = ak_folder.join("achievements.xsd");

    let definition = RawDefinitionV0::read(&test_xsd_path).unwrap();
    assert!(definition.is_some());

    let definition = definition.unwrap();
    assert_eq!(definition.xsd_element.len(), 2);

    // Second element has the table fields.
    let table_element = &definition.xsd_element[1];
    assert_eq!(table_element.name, Some("achievements".to_string()));

    // Check that it converts to a RawDefinition properly.
    let raw_def = RawDefinition::from(&definition);
    assert_eq!(raw_def.name, Some("achievements.xml".to_string()));
    assert_eq!(raw_def.fields.len(), 3);

    let key_field = raw_def.fields.iter().find(|f| f.name == "key").unwrap();
    assert_eq!(key_field.primary_key, "1");
    assert_eq!(key_field.field_type, "text");

    let title_field = raw_def.fields.iter().find(|f| f.name == "title").unwrap();
    assert_eq!(title_field.primary_key, "0");
    assert_eq!(title_field.field_type, "text");
}
