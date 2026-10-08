//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the schema and its table definitions: processed fields, patches, and serialization.

use super::*;

fn field(name: &str) -> Field {
    Field { name: name.to_owned(), field_type: FieldType::StringU8, is_key: true, ..Default::default() }
}

#[test]
fn test_definition_by_name_and_fields_prefers_version_keeping_most_columns() {
    let table = "ceo_initial_datas_tables";
    let mut schema = Schema::default();
    schema.add_definition(table, &Definition::new_with_fields(1, &[field("key")], &[], None));
    schema.add_definition(table, &Definition::new_with_fields(0, &[field("key"), field("template_manager")], &[], None));

    let two_columns = schema.definition_by_name_and_fields(table, &["key", "template_manager"]).unwrap();
    assert_eq!(*two_columns.version(), 0);

    let one_column = schema.definition_by_name_and_fields(table, &["key"]).unwrap();
    assert_eq!(*one_column.version(), 1);

    assert!(schema.definition_by_name_and_fields("missing_tables", &["key"]).is_none());
}

/// Definition with a bitwise field, a colour triplet, a sequence and a patch on the key column.
fn processed_test_definition() -> Definition {
    let sequence = Definition::new_with_fields(0, &[Field { name: "flags".to_owned(), field_type: FieldType::I32, is_bitwise: 2, ..Default::default() }], &[], None);
    let fields = [
        field("key"),
        Field { name: "flags".to_owned(), field_type: FieldType::I32, is_bitwise: 2, ..Default::default() },
        Field { name: "colour_r".to_owned(), field_type: FieldType::I32, is_part_of_colour: Some(1), ..Default::default() },
        Field { name: "colour_g".to_owned(), field_type: FieldType::I32, is_part_of_colour: Some(1), ..Default::default() },
        Field { name: "colour_b".to_owned(), field_type: FieldType::I32, is_part_of_colour: Some(1), ..Default::default() },
        Field { name: "entries".to_owned(), field_type: FieldType::SequenceU16(Box::new(sequence)), ..Default::default() },
    ];

    let patches = HashMap::from([("key".to_owned(), HashMap::from([
        ("is_key".to_owned(), "false".to_owned()),
        ("description".to_owned(), "Patched description".to_owned()),
        ("unused".to_owned(), "true".to_owned()),
    ]))]);

    Definition::new_with_fields(3, &fields, &[], Some(&patches))
}

#[test]
fn test_fields_processed_are_built_with_patches_applied() {
    let definition = processed_test_definition();

    let names = definition.fields_processed().iter().map(|field| field.name()).collect::<Vec<_>>();
    assert_eq!(names, ["key", "flags_1", "flags_2", "entries", "colour_hex"]);

    let key = &definition.fields_processed()[0];
    assert!(!key.is_key(None));
    assert!(key.unused(None));
    assert_eq!(key.description(None), "Patched description");

    // The raw fields keep their unpatched values.
    assert!(definition.fields()[0].is_key(None));
}

#[test]
fn test_fields_processed_are_rebuilt_on_changes() {
    let mut definition = processed_test_definition();

    definition.set_patches(DefinitionPatch::new());
    assert!(definition.fields_processed()[0].is_key(None));

    definition.edit_fields(|fields| fields.push(field("extra")));
    assert_eq!(definition.fields_processed().last().map(|field| field.name()), Some("colour_hex"));
    assert!(definition.fields_processed().iter().any(|field| field.name() == "extra"));

    definition.set_fields(vec![field("only")]);
    assert_eq!(definition.fields_processed().len(), 1);
}

#[test]
fn test_definition_json_round_trip_keeps_patches_and_rebuilds_processed_fields() {
    let definition = processed_test_definition();

    let mut json = serde_json::to_value(&definition).unwrap();
    assert!(json["fields_processed"].is_array());
    assert!(json["patches"]["key"].is_object());

    // Processed fields sent by a client are ignored, and rebuilt from the raw ones.
    json["fields_processed"] = serde_json::Value::Array(vec![]);
    let decoded: Definition = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, definition);
}

#[cfg(feature = "support_error_bitcode")]
#[test]
fn test_definition_bitcode_round_trip() {
    let definition = processed_test_definition();

    let data = bitcode::serialize(&definition).unwrap();
    let decoded: Definition = bitcode::deserialize(&data).unwrap();
    assert_eq!(decoded, definition);
}

#[test]
fn test_stored_definitions_only_keep_raw_parts() {
    let definition = processed_test_definition();
    let definitions = HashMap::from([("test_tables".to_owned(), definition.clone())]);

    let stored = Schema::export_definitions_to_str(&definitions).unwrap();
    assert!(!stored.contains("fields_processed"));
    assert!(!stored.contains("patches"));
    assert!(!stored.contains("unused"));

    // Loading them back gets the same definition, minus the patches.
    let mut loaded = Schema::load_definitions_from_str(&stored).unwrap().remove("test_tables").unwrap();
    loaded.set_patches(definition.patches().clone());
    assert_eq!(loaded, definition);
}
