//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for reading, filtering and editing table rows.

use serde_json::json;

use super::*;

fn filter(op: FilterOp, value: &str, ignore_case: bool) -> PreparedFilter {
    PreparedFilter::new(&RowFilter { column: "key".to_owned(), op, value: value.to_owned(), ignore_case }, 1)
}

#[test]
fn values_become_plain_json() {
    assert_eq!(decoded_to_json(&DecodedData::Boolean(true)), json!(true));
    assert_eq!(decoded_to_json(&DecodedData::F32(0.1)), json!(0.1));
    assert_eq!(decoded_to_json(&DecodedData::F32(f32::NAN)), Value::Null);
    assert_eq!(decoded_to_json(&DecodedData::F64(2.5)), json!(2.5));
    assert_eq!(decoded_to_json(&DecodedData::I16(-3)), json!(-3));
    assert_eq!(decoded_to_json(&DecodedData::OptionalI32(7)), json!(7));
    assert_eq!(decoded_to_json(&DecodedData::I64(1 << 40)), json!(1_i64 << 40));
    assert_eq!(decoded_to_json(&DecodedData::StringU8("wh_main_emp".to_owned())), json!("wh_main_emp"));
    assert_eq!(decoded_to_json(&DecodedData::ColourRGB("FF0000".to_owned())), json!("FF0000"));
}

#[test]
fn filters_compare_the_column_value_as_text() {
    let row = vec![DecodedData::I32(5), DecodedData::StringU8("wh_main_Empire".to_owned())];

    assert!(filter(FilterOp::Equals, "wh_main_Empire", false).matches(&row));
    assert!(!filter(FilterOp::Equals, "wh_main_empire", false).matches(&row));
    assert!(filter(FilterOp::Equals, "WH_MAIN_EMPIRE", true).matches(&row));
    assert!(filter(FilterOp::NotEquals, "other", false).matches(&row));
    assert!(filter(FilterOp::Contains, "main", false).matches(&row));
    assert!(filter(FilterOp::StartsWith, "wh_", false).matches(&row));
    assert!(filter(FilterOp::EndsWith, "empire", true).matches(&row));
    assert!(!filter(FilterOp::EndsWith, "empire", false).matches(&row));
}

fn fields() -> Vec<Field> {
    let mut key = Field::default();
    key.set_name("key".to_owned());
    key.set_field_type(FieldType::StringU8);

    let mut value = Field::default();
    value.set_name("value".to_owned());
    value.set_field_type(FieldType::F32);

    vec![key, value]
}

fn row(key: &str, value: f32) -> Vec<DecodedData> {
    vec![DecodedData::StringU8(key.to_owned()), DecodedData::F32(value)]
}

fn values(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
    pairs.iter().map(|(column, value)| (column.to_string(), value.clone())).collect()
}

fn edit(rows: &mut Vec<Vec<DecodedData>>, edits: &[RowEdit]) -> Result<usize, ApiError> {
    let prepared = prepare_row_edits(edits, &fields(), row("", 0.0), rows.len(), &BTreeMap::new())?;
    Ok(apply_row_edits(rows, prepared))
}

#[test]
fn edits_apply_in_order() {
    let mut rows = vec![row("a", 1.0), row("b", 2.0), row("c", 3.0)];
    let edits = vec![
        RowEdit::Insert { index: Some(0), copy_from: None, values: values(&[("key", json!("new")), ("value", json!(0.5))]) },
        RowEdit::Update { index: 2, values: values(&[("value", json!("2.5"))]) },
        RowEdit::Delete { indexes: vec![3, 1, 3] },
        RowEdit::Insert { index: None, copy_from: None, values: values(&[("key", json!("last"))]) },
    ];

    assert_eq!(edit(&mut rows, &edits), Ok(3));
    assert_eq!(rows, vec![row("new", 0.5), row("b", 2.5), row("last", 0.0)]);
}

#[test]
fn inserts_take_the_copied_values_replaced_by_their_own() {
    let mut rows = vec![row("a", 1.0)];
    let edits = vec![RowEdit::Insert { index: None, copy_from: None, values: values(&[("key", json!("copy"))]) }];
    let copied = BTreeMap::from([(0, vec![(0, DecodedData::StringU8("original".to_owned())), (1, DecodedData::F32(7.0))])]);

    let prepared = prepare_row_edits(&edits, &fields(), row("", 0.0), rows.len(), &copied).unwrap();
    apply_row_edits(&mut rows, prepared);

    assert_eq!(rows, vec![row("a", 1.0), row("copy", 7.0)]);
}

#[test]
fn copied_values_are_converted_to_the_column_type() {
    assert_eq!(copied_value(&FieldType::I32, &DecodedData::I32(5)).unwrap(), DecodedData::I32(5));
    assert_eq!(copied_value(&FieldType::I64, &DecodedData::I32(5)).unwrap(), DecodedData::I64(5));
    assert!(copied_value(&FieldType::I32, &DecodedData::StringU8("five".to_owned())).is_err());
}

#[test]
fn failed_edits_change_nothing() {
    let original = vec![row("a", 1.0), row("b", 2.0)];

    let failing = [
        vec![RowEdit::Update { index: 0, values: values(&[("value", json!(9.0))]) }, RowEdit::Update { index: 2, values: values(&[("value", json!(1.0))]) }],
        vec![RowEdit::Delete { indexes: vec![0] }, RowEdit::Update { index: 1, values: values(&[("value", json!(1.0))]) }],
        vec![RowEdit::Insert { index: Some(3), copy_from: None, values: BTreeMap::new() }],
        vec![RowEdit::Update { index: 0, values: values(&[("nope", json!(1))]) }],
        vec![RowEdit::Update { index: 0, values: values(&[("value", json!("not a number"))]) }],
        vec![RowEdit::Update { index: 0, values: values(&[("value", json!(null))]) }],
    ];

    for edits in failing {
        let mut rows = original.clone();
        assert!(matches!(edit(&mut rows, &edits), Err(ApiError::InvalidParams(_))), "{edits:?}");
        assert_eq!(rows, original);
    }
}

#[test]
fn values_are_converted_to_the_column_type() {
    assert_eq!(json_to_decoded(&FieldType::Boolean, &json!(true)).unwrap(), DecodedData::Boolean(true));
    assert_eq!(json_to_decoded(&FieldType::I32, &json!(42)).unwrap(), DecodedData::I32(42));
    assert_eq!(json_to_decoded(&FieldType::I32, &json!("42")).unwrap(), DecodedData::I32(42));
    assert_eq!(json_to_decoded(&FieldType::F32, &json!(3)).unwrap(), DecodedData::F32(3.0));
    assert_eq!(json_to_decoded(&FieldType::StringU8, &json!("text")).unwrap(), DecodedData::StringU8("text".to_owned()));
    assert!(json_to_decoded(&FieldType::StringU8, &json!(["a"])).is_err());
}

#[test]
fn invalid_values_report_the_type_the_column_expects() {
    let error = json_to_decoded(&FieldType::I32, &json!(4.5)).unwrap_err();

    assert_eq!(error.to_string(), "expected a 32-bit integer, found 4.5.");
}

#[test]
fn filters_on_missing_columns_never_match() {
    let row = vec![DecodedData::I32(5)];

    assert!(!filter(FilterOp::NotEquals, "anything", false).matches(&row));
}
