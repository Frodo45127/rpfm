//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `DB` files.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use crate::binary::ReadBytes;
use crate::files::*;
use crate::schema::{Definition, Field, FieldType};

use super::DB;

/// Returns a test definition with all field types for unit testing.
fn test_definition() -> Definition {
    let mut definition = Definition::new(-100, None);
    let mut fields = vec![];

    fields.push(Field { name: "bool".to_owned(), field_type: FieldType::Boolean, default_value: Some("true".to_string()), ..Default::default() });
    fields.push(Field { name: "f32".to_owned(), field_type: FieldType::F32, default_value: Some("1.0".to_string()), ..Default::default() });
    fields.push(Field { name: "f64".to_owned(), field_type: FieldType::F64, default_value: Some("2.0".to_string()), ..Default::default() });
    fields.push(Field { name: "i16".to_owned(), field_type: FieldType::I16, default_value: Some("3".to_string()), ..Default::default() });
    fields.push(Field { name: "i32".to_owned(), field_type: FieldType::I32, default_value: Some("4".to_string()), ..Default::default() });
    fields.push(Field { name: "i64".to_owned(), field_type: FieldType::I64, default_value: Some("5".to_string()), ..Default::default() });
    fields.push(Field { name: "colour".to_owned(), field_type: FieldType::ColourRGB, default_value: Some("ABCDEF".to_string()), ..Default::default() });
    fields.push(Field { name: "stringu8".to_owned(), field_type: FieldType::StringU8, default_value: Some("AAAA".to_string()), ..Default::default() });
    fields.push(Field { name: "stringu16".to_owned(), field_type: FieldType::StringU16, default_value: Some("BBBB".to_string()), ..Default::default() });
    fields.push(Field { name: "optionali16".to_owned(), field_type: FieldType::OptionalI16, default_value: Some("3".to_string()), ..Default::default() });
    fields.push(Field { name: "optionali32".to_owned(), field_type: FieldType::OptionalI32, default_value: Some("4".to_string()), ..Default::default() });
    fields.push(Field { name: "optionali64".to_owned(), field_type: FieldType::OptionalI64, default_value: Some("5".to_string()), ..Default::default() });
    fields.push(Field { name: "optionalstringu8".to_owned(), field_type: FieldType::OptionalStringU8, default_value: Some("Opt".to_string()), ..Default::default() });
    fields.push(Field { name: "optionalstringu16".to_owned(), field_type: FieldType::OptionalStringU16, default_value: Some("Opt".to_string()), ..Default::default() });
    fields.push(Field { name: "sequenceu16".to_owned(), field_type: FieldType::SequenceU16(Box::new(Definition::new(-100, None))), ..Default::default() });
    fields.push(Field { name: "sequenceu32".to_owned(), field_type: FieldType::SequenceU32(Box::new(Definition::new(-100, None))), ..Default::default() });

    // Special fields that use postprocessing.
    fields.push(Field {
        name: "merged_colours_1_r".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("AB".to_string()),
        is_part_of_colour: Some(0),
        ..Default::default()
    });
    fields.push(Field {
        name: "merged_colours_1_g".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("CD".to_string()),
        is_part_of_colour: Some(0),
        ..Default::default()
    });
    fields.push(Field {
        name: "merged_colours_1_b".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("EF".to_string()),
        is_part_of_colour: Some(0),
        ..Default::default()
    });
    fields.push(Field {
        name: "bitwise_values".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("4".to_string()),
        is_bitwise: 5,
        ..Default::default()
    });
    fields.push(Field {
        name: "enum_values".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("8".to_string()),
        enum_values: {
            let mut bt = BTreeMap::new();
            bt.insert(0, "test0".to_owned());
            bt.insert(1, "test1".to_owned());
            bt.insert(2, "test2".to_owned());
            bt.insert(3, "test3".to_owned());
            bt
        },
        ..Default::default()
    });

    fields.push(Field {
        name: "merged_colours_2_r".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("AB".to_string()),
        is_part_of_colour: Some(1),
        ..Default::default()
    });
    fields.push(Field {
        name: "merged_colours_2_g".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("CD".to_string()),
        is_part_of_colour: Some(1),
        ..Default::default()
    });
    fields.push(Field {
        name: "merged_colours_2_b".to_owned(),
        field_type: FieldType::I32,
        default_value: Some("EF".to_string()),
        is_part_of_colour: Some(1),
        ..Default::default()
    });

    // TODO: add combined colour columns for testing.

    definition.set_fields(fields);
    definition
}
/*
#[test]
fn test_generate_table() {
    use crate::files::table::Table;
    use crate::files::table::TableData;

    let definition = test_definition();
    let mut table: DB = From::from(Table::new(&definition, "test_decode_db", false));
    let row_1 = table.new_row(None, None);
    let row_2 = table.new_row(None, None);
    let row_3 = table.new_row(None, None);

    let table_data = TableData::Local(vec![row_1, row_2, row_3]);
    table.table.set_table_data(table_data);

    let mut after = vec![];
    table.encode(&mut after, None).unwrap();
    let mut writer = BufWriter::new(File::create("../test_files/test_decode_db").unwrap());
    writer.write_all(&after).unwrap();
    panic!();
}*/

#[test]
fn test_encode_db_no_sqlite() {
    let path_1 = "../test_files/test_decode_db";
    let path_2 = "../test_files/test_encode_db_no_sqlite";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let mut schema = Schema::default();
    schema.add_definition("test_decode_db", &test_definition());

    let mut decodeable_extra_data = DecodeableExtraData::default();
    decodeable_extra_data.file_name = Some("test_decode_db");
    decodeable_extra_data.table_name = Some("test_decode_db");
    decodeable_extra_data.schema = Some(&schema);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = DB::decode(&mut reader, &Some(decodeable_extra_data)).unwrap();

    let mut after = vec![];
    data.encode(&mut after, &None).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}
