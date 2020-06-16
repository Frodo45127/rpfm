//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
In this file are all the utility functions we need for the tables to work.
!*/

use qt_widgets::QTableView;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QModelIndex;
use qt_core::QSignalBlocker;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;
use qt_core::QObject;
use qt_core::CheckState;
use qt_core::QString;
use qt_core::Orientation;

use cpp_core::CppBox;
use cpp_core::MutPtr;
use cpp_core::Ref;

use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::sync::RwLock;
use std::sync::atomic::AtomicPtr;

use rpfm_lib::schema::{Definition, Field, FieldType};
use rpfm_lib::SETTINGS;

use crate::DARK_RED;
use crate::EVEN_MORE_WHITY_GREY;
use crate::ffi::*;
use crate::LINK_BLUE;
use crate::locale::{qtr, tr, tre};
use crate::MEDIUM_DARK_GREY;
use crate::utils::*;
use crate::UI_STATE;
use super::*;

//----------------------------------------------------------------------------//
//                       Undo/Redo helpers for tables
//----------------------------------------------------------------------------//

/// This function is used to update the background or undo table when a change is made in the main table.
pub unsafe fn update_undo_model(model: MutPtr<QStandardItemModel>, mut undo_model: MutPtr<QStandardItemModel>) {
    undo_model.clear();
    for row in 0..model.row_count_0a() {
        for column in 0..model.column_count_0a() {
            let item = &*model.item_2a(row, column);
            undo_model.set_item_3a(row, column, item.clone());
        }
    }
}

//----------------------------------------------------------------------------//
//                       Index helpers for tables
//----------------------------------------------------------------------------//

/// This function sorts the VISUAL SELECTION. That means, the selection just as you see it on screen.
/// This should be provided with the indexes OF THE VIEW/FILTER, NOT THE MODEL.
pub unsafe fn sort_indexes_visually(indexes_sorted: &mut Vec<Ref<QModelIndex>>, table_view: MutPtr<QTableView>) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    let horizontal_header = table_view.horizontal_header();
    let vertical_header = table_view.vertical_header();
    indexes_sorted.sort_unstable_by(|a, b| {
        if vertical_header.visual_index(a.row()) == vertical_header.visual_index(b.row()) {
            if horizontal_header.visual_index(a.column()) < horizontal_header.visual_index(b.column()) { Ordering::Less }
            else { Ordering::Greater }
        }
        else if vertical_header.visual_index(a.row()) < vertical_header.visual_index(b.row()) { Ordering::Less }
        else { Ordering::Greater }
    });
}

/// This function sorts the MODEL SELECTION. That means, the real selection over the model.
/// This should be provided with the indexes OF THE MODEL, NOT THE VIEW/FILTER.
pub unsafe fn sort_indexes_by_model(indexes_sorted: &mut Vec<Ref<QModelIndex>>) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    indexes_sorted.sort_unstable_by(|a, b| {
        if a.row() == b.row() {
            if a.column() < b.column() { Ordering::Less }
            else { Ordering::Greater }
        }
        else if a.row() < b.row() { Ordering::Less }
        else { Ordering::Greater }
    });
}


/// This function gives you the model's ModelIndexes from the ones from the view/filter.
pub unsafe fn get_real_indexes(indexes_sorted: &[Ref<QModelIndex>], filter_model: MutPtr<QSortFilterProxyModel>) -> Vec<CppBox<QModelIndex>> {
    indexes_sorted.iter().map(|x| filter_model.map_to_source(*x)).collect()
}

/// This function removes indexes with the same row from a list of indexes.
pub unsafe fn dedup_indexes_per_row(indexes: &mut Vec<Ref<QModelIndex>>) {
    let mut rows_done = vec![];
    let mut indexes_to_remove = vec![];
    for (pos, index) in indexes.iter().enumerate() {
        if rows_done.contains(&index.row()) { indexes_to_remove.push(pos); }
        else { rows_done.push(index.row())}
    }

    for index_to_remove in indexes_to_remove.iter().rev() {
        indexes.remove(*index_to_remove);
    }
}

/// This function deletes the provided rows from the provided model.
///
/// It returns a list of (first row of the pack's position, list of deleted rows).
/// NOTE: The list of rows must be in 9->0 order.
pub unsafe fn delete_rows(mut model: MutPtr<QStandardItemModel>, rows: &[i32]) -> Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)> {

    // Make sure all rows are sorted 9->0.
    let mut rows = rows.to_vec();
    rows.sort_by(|x, y| y.cmp(&x));

    // To optimize this, we remove them in consecutive packs, which really helps when you're deleting a ton of rows at the same time.
    // That way we only trigger one deletion with it's signals instead a ton of them.
    let mut rows_splitted = vec![];
    let mut current_row_pack = vec![];
    let mut current_row_index = -2;
    for (index, row) in rows.iter().enumerate() {

        // Items are individually cloned because there is no "takeRows" function to take out multiple individual rows.
        let items = (0..model.column_count_0a())
            .map(|column| (&*model.item_2a(*row, column)).clone())
            .collect::<Vec<MutPtr<QStandardItem>>>();

        // If the current line is not the next of the batch, nor the first one, finish the pack.
        if (*row != current_row_index - 1) && index != 0 {
            current_row_pack.reverse();
            rows_splitted.push((current_row_index, current_row_pack.to_vec()));
            current_row_pack.clear();
        }

        // Add the new row to the current pack.
        current_row_pack.push(items);
        current_row_index = *row;
    }
    current_row_pack.reverse();
    rows_splitted.push((current_row_index, current_row_pack));

    // And finally, remove the rows from the table.
    for row_pack in rows_splitted.iter() {
        model.remove_rows_2a(row_pack.0, row_pack.1.len() as i32);
    }

    // Reverse them, so the final result is full top to bottom, and return them.
    rows_splitted.reverse();
    rows_splitted.iter()
        .map(|x| (x.0, x.1.iter()
            .map(|y| y.iter()
                .map(|z| atomic_from_mut_ptr(*z))
                .collect()
            )
            .collect()
        ))
        .collect::<Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)>>()
}

/// This function returns a new default row.
pub unsafe fn get_new_row(table_definition: &Definition) -> CppBox<QListOfQStandardItem> {
    let mut qlist = QListOfQStandardItem::new();
    for field in &table_definition.fields {
        let item = get_default_item_from_field(field);
        add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
    }
    qlist
}

/// This function generates a *Default* StandardItem for the provided field.
unsafe fn get_default_item_from_field(field: &Field) -> CppBox<QStandardItem> {
    match field.field_type {
        FieldType::Boolean => {
            let mut item = QStandardItem::new();
            item.set_editable(false);
            item.set_checkable(true);
            if let Some(default_value) = &field.default_value {
                if default_value.to_lowercase() == "true" {
                    item.set_check_state(CheckState::Checked);
                } else {
                    item.set_check_state(CheckState::Unchecked);
                }
            } else {
                item.set_check_state(CheckState::Unchecked);
            }
            item
        }
        FieldType::F32 => {
            let mut item = QStandardItem::new();
            if let Some(default_value) = &field.default_value {
                if let Ok(default_value) = default_value.parse::<f32>() {
                    item.set_data_2a(&QVariant::from_float(default_value), 2);
                } else {
                    item.set_data_2a(&QVariant::from_float(0.0f32), 2);
                }
            } else {
                item.set_data_2a(&QVariant::from_float(0.0f32), 2);
            }
            item
        },
        FieldType::I16 => {
            let mut item = QStandardItem::new();
            if let Some(default_value) = &field.default_value {
                if let Ok(default_value) = default_value.parse::<i16>() {
                    item.set_data_2a(&QVariant::from_int(default_value as i32), 2);
                } else {
                    item.set_data_2a(&QVariant::from_int(0i32), 2);
                }
            } else {
                item.set_data_2a(&QVariant::from_int(0i32), 2);
            }
            item
        },
        FieldType::I32 => {
            let mut item = QStandardItem::new();
            if let Some(default_value) = &field.default_value {
                if let Ok(default_value) = default_value.parse::<i32>() {
                    item.set_data_2a(&QVariant::from_int(default_value), 2);
                } else {
                    item.set_data_2a(&QVariant::from_int(0i32), 2);
                }
            } else {
                item.set_data_2a(&QVariant::from_int(0i32), 2);
            }
            item
        },
        FieldType::I64 => {
            let mut item = QStandardItem::new();
            if let Some(default_value) = &field.default_value {
                if let Ok(default_value) = default_value.parse::<i64>() {
                    item.set_data_2a(&QVariant::from_i64(default_value), 2);
                } else {
                    item.set_data_2a(&QVariant::from_i64(0i64), 2);
                }
            } else {
                item.set_data_2a(&QVariant::from_i64(0i64), 2);
            }
            item
        },
        FieldType::StringU8 |
        FieldType::StringU16 |
        FieldType::OptionalStringU8 |
        FieldType::OptionalStringU16 => {
            if let Some(default_value) = &field.default_value {
                QStandardItem::from_q_string(&QString::from_std_str(default_value))
            } else {
                QStandardItem::from_q_string(&QString::new())
            }
        },
        FieldType::SequenceU16(_) | FieldType::SequenceU32(_)  => QStandardItem::from_q_string(&qtr("packedfile_noneditable_sequence")),
    }
}

/// This function "process" the column names of a table, so they look like they should.
pub fn clean_column_names(field_name: &str) -> String {
    let mut new_name = String::new();
    let mut should_be_uppercase = false;

    for character in field_name.chars() {

        if new_name.is_empty() || should_be_uppercase {
            new_name.push_str(&character.to_uppercase().to_string());
            should_be_uppercase = false;
        }

        else if character == '_' {
            new_name.push(' ');
            should_be_uppercase = true;
        }

        else { new_name.push(character); }
    }

    new_name
}

/// This function returns the color used for wrong referenced data in tables.
pub unsafe fn get_color_wrong_key() -> MutPtr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*DARK_RED)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*DARK_RED)).into_ptr()
    }
}

/// This function returns the color used for data with missing references in tables.
pub unsafe fn get_color_no_ref_data() -> MutPtr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*LINK_BLUE)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*LINK_BLUE)).into_ptr()
    }
}

/// This function returns the color used for correct referenced data in tables.
pub unsafe fn get_color_correct_key() -> MutPtr<QColor> {
    if SETTINGS.read().unwrap().settings_bool["use_dark_theme"] {
        QColor::from_q_string(&QString::from_std_str(*EVEN_MORE_WHITY_GREY)).into_ptr()
    } else {
        QColor::from_q_string(&QString::from_std_str(*MEDIUM_DARK_GREY)).into_ptr()
    }
}

/// Function to check if an specific field's data is in their references.
pub unsafe fn check_references(
    column: i32,
    mut item: MutPtr<QStandardItem>,
    dependency_data: &BTreeMap<i32, BTreeMap<String, String>>,
) {
    // First, check if we have dependency data for that column.
    if let Some(ref_data) = dependency_data.get(&column) {
        let text = item.text().to_std_string();

        // Then, check if the data we have is in the ref data list.
        if ref_data.is_empty() {
            item.set_foreground(&QBrush::from_q_color(get_color_no_ref_data().as_ref().unwrap()));
        }
        else if ref_data.contains_key(&text) {
            item.set_foreground(&QBrush::from_q_color(get_color_correct_key().as_ref().unwrap()));
        }
        else {
            item.set_foreground(&QBrush::from_q_color(get_color_wrong_key().as_ref().unwrap()));
        }
    }
}


/// This function loads the data from a compatible `PackedFile` into a TableView.
pub unsafe fn load_data(
    table_view_primary: MutPtr<QTableView>,
    table_view_frozen: MutPtr<QTableView>,
    definition: &Definition,
    dependency_data: &RwLock<BTreeMap<i32, BTreeMap<String, String>>>,
    data: &TableType,
) {
    let table_filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
    let mut table_model: MutPtr<QStandardItemModel> = table_filter.source_model().static_downcast_mut();

    // First, we delete all the data from the `ListStore`. Just in case there is something there.
    // This wipes out header information, so remember to run "build_columns" after this.
    table_model.clear();

    // Set the right data, depending on the table type you get.
    let data = match data {
        TableType::DependencyManager(data) => &data,
        TableType::DB(data) => &*data.get_ref_table_data(),
        TableType::Loc(data) => data.get_ref_table_data(),
    };

    if !data.is_empty() {

        // Load the data, row by row.
        let mut blocker = QSignalBlocker::from_q_object(table_model.static_upcast_mut::<QObject>());
        for (index, entry) in data.iter().enumerate() {
            let mut qlist = QListOfQStandardItem::new();
            for (index, field) in entry.iter().enumerate() {
                let mut item = get_item_from_decoded_data(field);

                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.read().unwrap().settings_bool["use_dependency_checker"] && definition.fields[index].is_reference.is_some() {
                    check_references(index as i32, item.as_mut_ptr(), &dependency_data.read().unwrap());
                }

                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
            if index == data.len() - 1 {
                blocker.unblock();
            }
            table_model.append_row_q_list_of_q_standard_item(&qlist);
        }
    }

    // If the table it's empty, we add an empty row and delete it, so the "columns" get created.
    else {
        let qlist = get_new_row(&definition);
        table_model.append_row_q_list_of_q_standard_item(&qlist);
        table_model.remove_rows_2a(0, 1);
    }

    setup_item_delegates(
        table_view_primary,
        table_view_frozen,
        definition,
        &dependency_data.read().unwrap()
    )
}

/// This function generates a StandardItem for the provided DecodedData.
pub unsafe fn get_item_from_decoded_data(data: &DecodedData) -> CppBox<QStandardItem> {
    match *data {

        // This one needs a couple of changes before turning it into an item in the table.
        DecodedData::Boolean(ref data) => {
            let mut item = QStandardItem::new();
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(*data), ITEM_SOURCE_VALUE);
            item.set_tool_tip(&QString::from_std_str(&tre("original_data", &[&data.to_string()])));
            item.set_editable(false);
            item.set_checkable(true);
            item.set_check_state(if *data { CheckState::Checked } else { CheckState::Unchecked });
            item
        }

        // Floats need to be tweaked to fix trailing zeroes and precission issues, like turning 0.5000004 into 0.5.
        // Also, they should be limited to 3 decimals.
        DecodedData::F32(ref data) => {
            let data = {
                let data_str = format!("{}", data);
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 3 { format!("{:.3}", data).parse::<f32>().unwrap() }
                    else { *data }
                }
                else { *data }
            };

            let mut item = QStandardItem::new();
            item.set_tool_tip(&QString::from_std_str(&tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_float(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_float(data), 2);
            item
        },
        DecodedData::I16(ref data) => {
            let mut item = QStandardItem::new();
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(*data as i32), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(*data as i32), 2);
            item
        },
        DecodedData::I32(ref data) => {
            let mut item = QStandardItem::new();
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(*data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(*data), 2);
            item
        },
        DecodedData::I64(ref data) => {
            let mut item = QStandardItem::new();
            item.set_tool_tip(&QString::from_std_str(&tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_i64(*data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_i64(*data), 2);
            item
        },
        // All these are Strings, so it can be together,
        DecodedData::StringU8(ref data) |
        DecodedData::StringU16(ref data) |
        DecodedData::OptionalStringU8(ref data) |
        DecodedData::OptionalStringU16(ref data) => {
            let mut item = QStandardItem::from_q_string(&QString::from_std_str(data));
            item.set_tool_tip(&QString::from_std_str(&tre("original_data", &[&data])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(data)), ITEM_SOURCE_VALUE);
            item
        },
        DecodedData::SequenceU16(_) | DecodedData::SequenceU32(_) => {
            let mut item = QStandardItem::from_q_string(&qtr("packedfile_noneditable_sequence"));
            item.set_editable(false);
            item
        }
    }
}

/// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
/// His intended use is for just after we load/reload the data to the table.
pub unsafe fn build_columns(
    mut table_view_primary: MutPtr<QTableView>,
    table_view_frozen: Option<MutPtr<QTableView>>,
    definition: &Definition,
    table_name: &str,
) {
    let filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
    let mut model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
    let schema = SCHEMA.read().unwrap();
    let mut do_we_have_ca_order = false;
    let mut keys = vec![];

    // For each column, clean their name and set their width and tooltip.
    for (index, field) in definition.fields.iter().enumerate() {

        let name = clean_column_names(&field.name);
        let mut item = QStandardItem::from_q_string(&QString::from_std_str(&name));
        set_column_tooltip(&schema, &field, table_name, &mut item);
        model.set_horizontal_header_item(index as i32, item.into_ptr());

        // Depending on his type, set one width or another.
        match field.field_type {
            FieldType::Boolean => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_BOOLEAN),
            FieldType::F32 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
            FieldType::I16 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
            FieldType::I32 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
            FieldType::I64 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
            FieldType::StringU8 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
            FieldType::StringU16 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
            FieldType::OptionalStringU8 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
            FieldType::OptionalStringU16 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
            FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
        }

        // If the field is key, add that column to the "Key" list, so we can move them at the beginning later.
        if field.is_key { keys.push(index); }
        if field.ca_order != -1 { do_we_have_ca_order |= true; }
    }

    // Now the order. If we have a sort order from the schema, we use that one.
    if do_we_have_ca_order {
        let mut header_primary = table_view_primary.horizontal_header();
        let mut fields = definition.fields.iter()
            .enumerate()
            .map(|(x, y)| (x, y.ca_order))
            .collect::<Vec<(usize, i16)>>();
        fields.sort_by(|a, b| a.1.cmp(&b.1));

        let mut new_pos = 0;
        for (logical_index, ca_order) in &fields {
            if *ca_order != -1 {
                let visual_index = header_primary.visual_index(*logical_index as i32);
                header_primary.move_section(visual_index as i32, new_pos);

                if let Some(table_view_frozen) = table_view_frozen {
                    let mut header_frozen = table_view_frozen.horizontal_header();
                    header_frozen.move_section(visual_index as i32, new_pos);
                }
            }
            new_pos += 1;
        }
    }

    // Otherwise, if we have any "Key" field, move it to the beginning.
    else if !keys.is_empty() {
        let mut header_primary = table_view_primary.horizontal_header();
        for (position, column) in keys.iter().enumerate() {
            header_primary.move_section(*column as i32, position as i32);

            if let Some(table_view_frozen) = table_view_frozen {
                let mut header_frozen = table_view_frozen.horizontal_header();
                header_frozen.move_section(*column as i32, position as i32);
            }
        }
    }

    // If we want to let the columns resize themselfs...
    if SETTINGS.read().unwrap().settings_bool["adjust_columns_to_content"] {
        table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
    }
}

/// This function sets the tooltip for the provided column header, if the column should have one.
pub unsafe fn set_column_tooltip(schema: &Option<Schema>, field: &Field, table_name: &str, item: &mut QStandardItem) {

    // If we passed it a table name, build the tooltip based on it. The logic is simple:
    // - If we have a description, we add it to the tooltip.
    // - If the column references another column, we add it to the tooltip.
    // - If the column is referenced by another column, we add it to the tooltip.
    if !table_name.is_empty() {
        let mut tooltip_text = String::new();
        if !field.description.is_empty() {
            tooltip_text.push_str(&format!("<p>{}</p>", field.description));
        }

        if let Some(ref reference) = field.is_reference {
            tooltip_text.push_str(&format!("<p>{}</p><p><i>\"{}/{}\"</i></p>", tr("column_tooltip_1"), reference.0, reference.1));
        }

        else {
            let mut referenced_columns = if let Some(ref schema) = schema {
                let short_table_name = table_name.split_at(table_name.len() - 7).0;
                let mut columns = vec![];

                // We get all the db definitions from the schema, then iterate all of them to find what tables reference our own.
                for versioned_file in schema.get_ref_versioned_file_db_all() {
                    if let VersionedFile::DB(ref_table_name, ref_definition) = versioned_file {
                        let mut found = false;
                        for ref_version in ref_definition {
                            for ref_field in &ref_version.fields {
                                if let Some((ref_ref_table, ref_ref_field)) = &ref_field.is_reference {
                                    if ref_ref_table == short_table_name && ref_ref_field == &field.name {
                                        found = true;
                                        columns.push((ref_table_name.to_owned(), ref_field.name.to_owned()));
                                    }
                                }
                            }
                            if found { break; }
                        }
                    }
                }
                columns
            } else { vec![] };

            referenced_columns.sort_unstable();
            if !referenced_columns.is_empty() {
                tooltip_text.push_str(&format!("<p>{}</p>", tr("column_tooltip_3")));
                for (index, reference) in referenced_columns.iter().enumerate() {
                    tooltip_text.push_str(&format!("<i>\"{}/{}\"</i><br>", reference.0, reference.1));

                    // There is a bug that causes tooltips to be displayed out of screen if they're too big. This fixes it.
                    if index == 50 {
                        tooltip_text.push_str(&format!("<p>{}</p>nnnn", tre("column_tooltip_2", &[&(referenced_columns.len() as isize - 50).to_string()])));
                        break;
                    }
                }

                // Dirty trick to remove the last <br> from the tooltip, or the nnnn in case that text get used.
                tooltip_text.pop();
                tooltip_text.pop();
                tooltip_text.pop();
                tooltip_text.pop();
            }
        }

        // We only add the tooltip if we got something to put into it.
        if !tooltip_text.is_empty() {
            item.set_tool_tip(&QString::from_std_str(&tooltip_text));
        }
    }
}

/// This function returns the reference data for an entire table.
pub unsafe fn get_reference_data(definition: &Definition) -> Result<BTreeMap<i32, BTreeMap<String, String>>> {

    // Call the backend passing it the files we have open (so we don't get them from the backend too), and get the frontend data while we wait for it to finish.
    let files_to_ignore = UI_STATE.get_open_packedfiles().iter().map(|x| x.get_path()).collect();
    CENTRAL_COMMAND.send_message_qt(Command::GetReferenceDataFromDefinition(definition.clone(), files_to_ignore));

    let reference_data = definition.get_reference_data();
    let mut dependency_data_visual = BTreeMap::new();

    let open_packedfiles = UI_STATE.get_open_packedfiles();
    for (index, (table, column, lookup)) in &reference_data {
        let mut dependency_data_visual_column = BTreeMap::new();
        for packed_file_view in open_packedfiles.iter() {
            let path = packed_file_view.get_ref_path();
            if path.len() == 3 && path[0].to_lowercase() == "db" && path[1].to_lowercase() == format!("{}_tables", table) {
                if let ViewType::Internal(view) = packed_file_view.get_view() {
                    if let View::Table(table) = view {
                        let column = clean_column_names(column);
                        let table_model = mut_ptr_from_atomic(&table.table_model);
                        for column_index in 0..table_model.column_count_0a() {
                            if table_model.header_data_2a(column_index, Orientation::Horizontal).to_string().to_std_string() == column {
                                for row in 0..table_model.row_count_0a() {
                                    let item = table_model.item_2a(row, column_index);
                                    let value = item.text().to_std_string();
                                    let lookup_value = match lookup {
                                        Some(columns) => {
                                            let data: Vec<String> = (0..table_model.column_count_0a()).filter(|x| {
                                                columns.contains(&table_model.header_data_2a(*x, Orientation::Horizontal).to_string().to_std_string())
                                            }).map(|x| table_model.item_2a(row, x).text().to_std_string()).collect();
                                            data.join(" ")
                                        },
                                        None => String::new(),
                                    };
                                    dependency_data_visual_column.insert(value, lookup_value);
                                }
                            }
                        }
                    }
                }
            }
        }
        dependency_data_visual.insert(index, dependency_data_visual_column);
    }

    let mut response = CENTRAL_COMMAND.recv_message_qt();
    match response {
        Response::BTreeMapI32BTreeMapStringString(ref mut dependency_data) => {
            for index in reference_data.keys() {
                if let Some(mut column_data_visual) = dependency_data_visual.get_mut(index) {
                    if let Some(column_data) = dependency_data.get_mut(index) {
                        column_data.append(&mut column_data_visual);
                    }
                }
            }

            Ok(dependency_data.clone())
        },
        Response::Error(error) => Err(error),
        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
    }
}

/// This function sets up the item delegates for all columns in a table.
pub unsafe fn setup_item_delegates(
    mut table_view_primary: MutPtr<QTableView>,
    mut table_view_frozen: MutPtr<QTableView>,
    definition: &Definition,
    dependency_data: &BTreeMap<i32, BTreeMap<String, String>>
) {
    let enable_lookups = false; //table_enable_lookups_button.is_checked();
    for (column, field) in definition.fields.iter().enumerate() {

        // Combos are a bit special, as they may or may not replace other delegates. If we disable them, use the normal delegates.
        if !SETTINGS.read().unwrap().settings_bool["disable_combos_on_tables"] && dependency_data.get(&(column as i32)).is_some() {
            if let Some(data) = dependency_data.get(&(column as i32)) {
                let mut list = QStringList::new();
                data.iter().map(|x| if enable_lookups { x.1 } else { x.0 }).for_each(|x| list.append_q_string(&QString::from_std_str(x)));
                new_combobox_item_delegate_safe(&mut table_view_primary, column as i32, list.as_ptr(), true, field.max_length);
                new_combobox_item_delegate_safe(&mut table_view_frozen, column as i32, list.as_ptr(), true, field.max_length);
            }
        }

        else {
            match field.field_type {
                FieldType::Boolean => {},
                FieldType::F32 => {
                    new_doublespinbox_item_delegate_safe(&mut table_view_primary, column as i32);
                    new_doublespinbox_item_delegate_safe(&mut table_view_frozen, column as i32);
                },
                FieldType::I16 => {
                    new_spinbox_item_delegate_safe(&mut table_view_primary, column as i32, 16);
                    new_spinbox_item_delegate_safe(&mut table_view_frozen, column as i32, 16);
                },
                FieldType::I32 => {
                    new_spinbox_item_delegate_safe(&mut table_view_primary, column as i32, 32);
                    new_spinbox_item_delegate_safe(&mut table_view_frozen, column as i32, 32);
                },

                // LongInteger uses normal string controls due to QSpinBox being limited to i32.
                FieldType::I64 => {
                    new_spinbox_item_delegate_safe(&mut table_view_primary, column as i32, 64);
                    new_spinbox_item_delegate_safe(&mut table_view_frozen, column as i32, 64);
                },
                FieldType::StringU8 |
                FieldType::StringU16 |
                FieldType::OptionalStringU8 |
                FieldType::OptionalStringU16 => {
                    new_qstring_item_delegate_safe(&mut table_view_primary, column as i32, field.max_length);
                    new_qstring_item_delegate_safe(&mut table_view_frozen, column as i32, field.max_length);
                },
                FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => {}
            }
        }
    }
}

/// This function checks an entire table for errors.
pub unsafe fn check_table_for_error(
    model: MutPtr<QStandardItemModel>,
    definition: &Definition,
    dependency_data: &BTreeMap<i32, BTreeMap<String, String>>
) {
    let _blocker = QSignalBlocker::from_q_object(model.static_upcast_mut::<QObject>());
    for (column, field) in definition.fields.iter().enumerate() {
        if field.is_reference.is_some() {
            for row in 0..model.row_count_0a() {
                let item = model.item_2a(row, column as i32);
                check_references(column as i32, item, dependency_data);
            }
        }
    }
}
