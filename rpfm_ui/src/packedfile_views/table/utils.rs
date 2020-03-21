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
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;
use qt_core::CheckState;
use qt_core::QString;

use cpp_core::CppBox;
use cpp_core::MutPtr;
use cpp_core::Ref;

use std::cmp::Ordering;
use std::sync::atomic::AtomicPtr;

use rpfm_lib::schema::{Definition, Field, FieldType};

use crate::ffi::add_to_q_list_safe;
use crate::locale::qtr;
use crate::utils::*;

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

/// This function causes the model to use the same colors the undo_model uses. It's for loading the "modified" state
/// of the table when you modify it, close it and open it again.
/// NOTE: This assumes both models are a copy one from another. Any discrepance in their sizes will send the program crashing to hell.
pub unsafe fn load_colors_from_undo_model(model: MutPtr<QStandardItemModel>, undo_model: MutPtr<QStandardItemModel>) {
    for row in 0..undo_model.row_count_0a() {
        for column in 0..undo_model.column_count_0a() {
            let color = &undo_model.item_2a(row, column).background();
            model.item_2a(row, column).set_background(color);
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
        let items = (0..model.column_count_0a()).into_iter()
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
        FieldType::Float => {
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
        FieldType::Integer => {
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
        FieldType::LongInteger => {
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
        FieldType::Sequence(_) => QStandardItem::from_q_string(&qtr("packedfile_noneditable_sequence")),
    }
}
