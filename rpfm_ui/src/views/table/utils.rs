//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QDialog;
use qt_widgets::QTableView;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QIcon;
use qt_gui::QListOfQStandardItem;
use qt_gui::QPixmap;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::ItemFlag;
use qt_core::QByteArray;
use qt_core::QListOfQModelIndex;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;
use qt_core::QObject;
use qt_core::CheckState;
use qt_core::QString;
use qt_core::SortOrder;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;

use csv::ReaderBuilder;
use rayon::prelude::*;

use std::borrow::Cow;
use std::cmp::{Ordering, Reverse};
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{atomic::AtomicPtr, RwLock};

use rpfm_extensions::dependencies::TableReferences;

use rpfm_lib::binary::WriteBytes;
use rpfm_lib::files::{ContainerPath, RFileDecoded, table::Table};
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::{Definition, DefinitionPatch, Field, FieldType};

use rpfm_ui_common::locale::{qtr, tr, tre};
use rpfm_ui_common::SETTINGS;

use crate::ffi::*;
use crate::packedfile_views::DataSource;
use crate::QVARIANT_TRUE;
use crate::QVARIANT_FALSE;

use super::*;

//----------------------------------------------------------------------------//
//                       Undo/Redo helpers for tables
//----------------------------------------------------------------------------//

/// This function is used to update the background or undo table when a change is made in the main table.
pub unsafe fn update_undo_model(model: &QPtr<QStandardItemModel>, undo_model: &QPtr<QStandardItemModel>) {
    undo_model.block_signals(true);
    undo_model.clear();
    for row in 0..model.row_count_0a() {
        for column in 0..model.column_count_0a() {
            let item = model.item_2a(row, column);
            if item.is_null() {
                error!("Null on item model? WTF? Row: {}, Column: {}", row, column);
            } else {
                undo_model.set_item_3a(row, column, (*item).clone());
            }
        }
    }
    undo_model.block_signals(false);
}

//----------------------------------------------------------------------------//
//                       Index helpers for tables
//----------------------------------------------------------------------------//

/// This function returns the real indexes for the VISIBLE SELECTION of a view, sorted visually. This means all filtered out rows and hidden columns are not returned, even if selected.
pub unsafe fn get_real_indexes_from_visible_selection_sorted(view: &QPtr<QTableView>, filter: &QPtr<QSortFilterProxyModel>) -> Vec<CppBox<QModelIndex>> {
    let indexes = view.selection_model().selection().indexes();
    let indexes_sorted = get_visible_selection_sorted(&indexes, view);
    get_real_indexes(&indexes_sorted, filter)
}

/// This function returns the VISIBLE SELECTION of a view, sorted visually. This means all filtered out rows and hidden columns are not returned, even if selected.
pub unsafe fn get_visible_selection_sorted(indexes: &CppBox<QListOfQModelIndex>, view: &QPtr<QTableView>) -> Vec<Ref<QModelIndex>> {
    let mut indexes_sorted = get_visible_selection_unsorted(indexes, view);
    sort_indexes_visually(&mut indexes_sorted, view);
    indexes_sorted
}

/// This function returns the VISIBLE SELECTION of a view, unsorted. This means all filtered out rows and hidden columns are not returned, even if selected.
pub unsafe fn get_visible_selection_unsorted(indexes: &CppBox<QListOfQModelIndex>, view: &QPtr<QTableView>) -> Vec<Ref<QModelIndex>> {
    let hidden_columns = (0..view.model().column_count_0a()).filter(|index| view.is_column_hidden(*index)).collect::<Vec<i32>>();
    (0..indexes.count_0a()).filter_map(|x| {
        let filter_index = indexes.at(x);
        if !filter_index.is_valid() {
            None
        } else if hidden_columns.contains(&filter_index.column()) {
            None
        } else {
            Some(filter_index)
        }
    }).collect::<Vec<Ref<QModelIndex>>>()
}

/// This function sorts the VISUAL SELECTION. That means, the selection just as you see it on screen.
/// This should be provided with the indexes OF THE VIEW/FILTER, NOT THE MODEL.
pub unsafe fn sort_indexes_visually(indexes_sorted: &mut [Ref<QModelIndex>], table_view: &QPtr<QTableView>) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    let horizontal_header = table_view.horizontal_header();
    let vertical_header = table_view.vertical_header();
    indexes_sorted.sort_unstable_by(|a, b| {
        let cmp = vertical_header.visual_index(a.row()).cmp(&vertical_header.visual_index(b.row()));
        match cmp {
            Ordering::Equal => if horizontal_header.visual_index(a.column()) < horizontal_header.visual_index(b.column()) { Ordering::Less } else { Ordering::Greater },
            _ => cmp,
        }
    });
}

/// This function sorts the MODEL SELECTION. That means, the real selection over the model.
/// This should be provided with the indexes OF THE MODEL, NOT THE VIEW/FILTER.
pub unsafe fn sort_indexes_by_model(indexes_sorted: &mut [Ref<QModelIndex>]) {

    // Sort the indexes so they follow the visual index, not their logical one.
    // This should fix situations like copying a row and getting a different order in the cells,
    // or copying a sorted table and getting a weird order in the copied cells.
    indexes_sorted.sort_unstable_by(|a, b| {
        let cmp = a.row().cmp(&b.row());
        match cmp {
            Ordering::Equal => if a.column() < b.column() { Ordering::Less } else { Ordering::Greater },
            _ => cmp,
        }
    });
}


/// This function gives you the model's ModelIndexes from the ones from the view/filter.
pub unsafe fn get_real_indexes(indexes_sorted: &[Ref<QModelIndex>], filter_model: &QPtr<QSortFilterProxyModel>) -> Vec<CppBox<QModelIndex>> {
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
pub unsafe fn delete_rows(model: &QPtr<QStandardItemModel>, rows: &[i32]) -> Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)> {

    // Make sure all rows are sorted 9->0.
    let mut rows = rows.to_vec();
    rows.sort_by_key(|&y| Reverse(y));

    // To optimize this, we remove them in consecutive packs, which really helps when you're deleting a ton of rows at the same time.
    // That way we only trigger one deletion with it's signals instead a ton of them.
    let mut rows_split = vec![];
    let mut current_row_pack = vec![];
    let mut current_row_index = -2;
    for (index, row) in rows.iter().enumerate() {

        // Items are individually cloned because there is no "takeRows" function to take out multiple individual rows.
        let items = (0..model.column_count_0a())
            .map(|column| (*model.item_2a(*row, column)).clone())
            .collect::<Vec<Ptr<QStandardItem>>>();

        // If the current line is not the next of the batch, nor the first one, finish the pack.
        if (*row != current_row_index - 1) && index != 0 {
            current_row_pack.reverse();
            rows_split.push((current_row_index, current_row_pack.to_vec()));
            current_row_pack.clear();
        }

        // Add the new row to the current pack.
        current_row_pack.push(items);
        current_row_index = *row;
    }
    current_row_pack.reverse();
    rows_split.push((current_row_index, current_row_pack));

    // And finally, remove the rows from the table.
    for row_pack in rows_split.iter() {
        model.remove_rows_2a(row_pack.0, row_pack.1.len() as i32);
    }

    // Reverse them, so the final result is full top to bottom, and return them.
    rows_split.reverse();
    rows_split.iter()
        .map(|x| (x.0, x.1.iter()
            .map(|y| y.iter()
                .map(|z| atomic_from_ptr(*z))
                .collect()
            )
            .collect()
        ))
        .collect::<Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)>>()
}

/// This function returns a new default row.
pub unsafe fn get_new_row(table_definition: &Definition) -> CppBox<QListOfQStandardItem> {
    let qlist = QListOfQStandardItem::new();
    let patches = Some(table_definition.patches());
    for field in table_definition.fields_processed() {
        let item = get_default_item_from_field(&field, patches);
        qlist.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
    }
    qlist
}

/// This function generates a *Default* StandardItem for the provided field.
pub unsafe fn get_default_item_from_field(field: &Field, patches: Option<&DefinitionPatch>) -> CppBox<QStandardItem> {
    let item = match field.field_type() {
        FieldType::Boolean => {
            let item = QStandardItem::new();
            item.set_editable(false);
            item.set_checkable(true);
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);

            let check_state = if let Some(default_value) = field.default_value(patches) {
                default_value.to_lowercase() == "true"
            } else { false };

            if check_state {
                item.set_check_state(CheckState::Checked);
                item.set_data_2a(&QVariant::from_bool(true), ITEM_SOURCE_VALUE);
                item.set_tool_tip(&QString::from_std_str(tre("original_data", &["True"])));
            }
            else {
                item.set_check_state(CheckState::Unchecked);
                item.set_data_2a(&QVariant::from_bool(false), ITEM_SOURCE_VALUE);
                item.set_tool_tip(&QString::from_std_str(tre("original_data", &["False"])));
            }
            item
        }
        FieldType::F32 => {
            let item = QStandardItem::new();
            let data = if let Some(default_value) = field.default_value(patches) {
                default_value.parse::<f32>().unwrap_or_default()
            } else {
                0.0f32
            };

            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_float(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_float(data), 2);
            item
        },
        FieldType::F64 => {
            let item = QStandardItem::new();
            let data = if let Some(default_value) = field.default_value(patches) {
                default_value.parse::<f64>().unwrap_or_default()
            } else {
                0.0f64
            };

            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_double(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_double(data), 2);
            item
        },
        FieldType::I16 |
        FieldType::OptionalI16 => {
            let item = QStandardItem::new();
            let data = if let Some(default_value) = field.default_value(patches) {
                if let Ok(default_value) = default_value.parse::<i16>() {
                    default_value as i32
                } else {
                    0_i32
                }
            } else {
                0_i32
            };
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_int(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(data), 2);
            item
        },
        FieldType::I32 |
        FieldType::OptionalI32 => {
            let item = QStandardItem::new();
            let data = if let Some(default_value) = field.default_value(patches) {
                default_value.parse::<i32>().unwrap_or_default()
            } else {
                0i32
            };
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_int(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_int(data), 2);
            item
        },
        FieldType::I64 |
        FieldType::OptionalI64 => {
            let item = QStandardItem::new();
            let data = if let Some(default_value) = field.default_value(patches) {
                default_value.parse::<i64>().unwrap_or_default()
            } else {
                0i64
            };
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&data.to_string()])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_i64(data), ITEM_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_i64(data), 2);
            item
        },
        FieldType::ColourRGB => {
            let text = if let Some(default_value) = field.default_value(patches) {
                if u32::from_str_radix(&default_value, 16).is_ok() {
                    default_value
                } else {
                    "000000".to_owned()
                }
            } else {
                "000000".to_owned()
            };
            let item = QStandardItem::from_q_string(&QString::from_std_str(&text));
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&text])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&text)), ITEM_SOURCE_VALUE);
            item
        },
        FieldType::StringU8 |
        FieldType::StringU16 |
        FieldType::OptionalStringU8 |
        FieldType::OptionalStringU16 => {
            let text = field.default_value(patches).unwrap_or_default();
            let item = QStandardItem::from_q_string(&QString::from_std_str(&text));
            item.set_tool_tip(&QString::from_std_str(tre("original_data", &[&text])));
            item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&text)), ITEM_SOURCE_VALUE);
            item
        },

        FieldType::SequenceU16(_) | FieldType::SequenceU32(_)  => {
            let item = QStandardItem::new();

            item.set_editable(false);
            item.set_text(&qtr("packedfile_editable_sequence"));
            item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_byte_array(&QByteArray::from_slice(&[0, 0, 0, 0])), ITEM_SEQUENCE_DATA);
            item
        }
    };

    if field.is_key(patches) {
        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_KEY);
    }

    item
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

/// This function loads the data from a compatible `PackedFile` into a TableView.
pub unsafe fn load_data(
    table_view: &QPtr<QTableView>,
    definition: &Definition,
    table_name: Option<&str>,
    dependency_data: &RwLock<HashMap<i32, TableReferences>>,
    data: &TableType,
    timer: &QBox<QTimer>,
    data_source: DataSource,
    vanilla_data: &[(DB, HashMap<String, i32>)]
) {
    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
    let reference_data = dependency_data.read().unwrap();

    // First, we delete all the data from the `ListStore`. Just in case there is something there.
    // This wipes out header information, so remember to run "build_columns" after this.
    table_model.clear();

    // Build the columns. We do this without data already in to ensure Qt doesn't do unnecessary stuff.
    let resize_after_data = build_columns(table_view, definition, table_name, data);

    // Set the right data, depending on the table type you get.
    let (data, is_translator) = match data {
        TableType::AnimFragmentBattle(data) => (data.data(), false),
        TableType::Atlas(data) => (data.data(), false),
        TableType::DependencyManager(data) => (Cow::from(data), false),
        TableType::DB(data) => (data.data(), false),
        TableType::Loc(data) => (data.data(), false),
        TableType::NormalTable(data) => (data.data(), false),
        #[cfg(feature = "enable_tools")] TableType::TranslatorTable(data) => (data.data(), true),
    };

    if !data.is_empty() {

        // NOTE: We need the blocker because disabling only updates doesn't seem to work.
        table_view.set_updates_enabled(false);
        table_model.block_signals(true);

        let fields_processed = definition.fields_processed();
        let patches = Some(definition.patches());
        let keys = fields_processed.iter().enumerate().filter_map(|(x, y)| if y.is_key(patches) { Some(x as i32) } else { None }).collect::<Vec<i32>>();
        let enable_lookups = SETTINGS.read().unwrap().bool("enable_lookups");
        let enable_icons = SETTINGS.read().unwrap().bool("enable_icons");

        let icons: BTreeMap<i32, (String, HashMap<String, AtomicPtr<QIcon>>)> = if enable_icons {
            let mut map = BTreeMap::new();
            for (column, field) in fields_processed.iter().enumerate() {
                if field.is_filename(patches) {
                    let _ = request_backend_files(&data, column, field, patches, &mut map);
                }
            }
            map
        } else {
            BTreeMap::new()
        };

        let key_pos = definition.key_column_positions();

        // Get each row in a mass loop.
        let qlists = data.par_iter().map(|entry| {
            let qlist = QListOfQStandardItem::new();
            qlist.reserve(entry.len() as i32);

            let keys_joined = key_pos.iter()
                .map(|x| entry[*x].data_to_string())
                .join("");

            for (column, field) in entry.iter().enumerate() {
                let item = get_item_from_decoded_data(field, &keys, column);

                if data_source != DataSource::PackFile || (is_translator && qlist.count_0a() < 4) {
                    item.set_editable(false);

                    // Checkable items do not get properly disabled with the set_editable function.
                    if item.is_checkable() {
                        let mut flags = item.flags().to_int();
                        flags &= !ItemFlag::ItemIsUserCheckable.to_int();
                        item.set_flags(QFlags::from(flags));
                    }
                }

                if enable_lookups {
                    if let Some(column_data) = reference_data.get(&(column as i32)) {
                        if let Some(lookup) = column_data.data().get(&*field.data_to_string()) {
                            if !data.is_empty() {
                                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup)), ITEM_SUB_DATA);
                            }
                        }
                    }
                }

                if enable_icons {
                    if let Some(column_data) = icons.get(&(column as i32)) {
                        let cell_data = entry[column].data_to_string().replace('\\', "/");

                        // For paths, we need to fix the ones in older games starting with / or data/.
                        let mut start_offset = 0;
                        if cell_data.starts_with("/") {
                            start_offset += 1;
                        }
                        if cell_data.starts_with("data/") {
                            start_offset += 5;
                        }
                        let paths_join = column_data.0.replace('%', &cell_data[start_offset..]).to_lowercase();
                        let paths_split = paths_join.split(';');

                        for path in paths_split {
                            if let Some(icon) = column_data.1.get(path) {
                                let icon = ref_from_atomic(icon);
                                item.set_icon(icon);
                                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(path)), 52);
                                break;
                            }
                        }
                    }
                }

                // If we have keys and vanilla/parent data, try to mark the different cells/rows.
                if !vanilla_data.is_empty() && !key_pos.is_empty() {

                    let mut found = false;
                    for (vanilla_db, vanilla_data) in vanilla_data {
                        let vanilla_definition = vanilla_db.definition();
                        let vanilla_processed_fields = vanilla_definition.fields_processed();

                        match vanilla_data.get(&keys_joined) {
                            Some(row) => {
                                item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_ADDED_VS_VANILLA);

                                // Ignore fields that are not in the vanilla table.
                                let column = &fields_processed[column];
                                if let Some(vanilla_field_column) = vanilla_processed_fields.iter().position(|x| x.name() == column.name()) {
                                    match vanilla_db.data()[*row as usize].get(vanilla_field_column) {
                                        Some(value) => {
                                            if value.data_to_string() != field.data_to_string() {
                                                item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_IS_MODIFIED_VS_VANILLA);
                                            } else {
                                                item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA);
                                            }

                                            found = true;
                                            break;
                                        },

                                        None => item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA),
                                    }
                                } else {
                                    item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA);
                                }
                            }

                            None => continue,
                        }
                    }

                    if !found {
                        // Disabled until I figure a way to make it dynamic.
                        //item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_IS_ADDED_VS_VANILLA);
                    }
                }

                qlist.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
            }

            atomic_from_ptr(qlist.into_ptr())
        }).collect::<Vec<_>>();

        // Load the data, row by row.
        for (row, qlist) in qlists.iter().enumerate() {
            let qlist = ptr_from_atomic(qlist);

            if row == qlists.len() - 1 {
                table_model.block_signals(false);
                table_view.set_updates_enabled(true);
            }

            table_model.append_row_q_list_of_q_standard_item(qlist.as_ref().unwrap());
        }
    }

    // If we need to do a loaded data-based resizing, it has to be done here, not at the top.
    if resize_after_data {
        table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
    }

    setup_item_delegates(
        table_view,
        definition,
        &reference_data,
        timer
    );
}

/// This function generates a StandardItem for the provided DecodedData.
pub unsafe fn get_item_from_decoded_data(data: &DecodedData, keys: &[i32], column: usize) -> CppBox<QStandardItem> {
    let item = match *data {

        // This one needs a couple of changes before turning it into an item in the table.
        DecodedData::Boolean(ref data) => {
            let item = QStandardItem::new();
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_bool(*data), ITEM_SOURCE_VALUE);
            item.set_editable(false);
            item.set_checkable(true);
            item.set_check_state(if *data { CheckState::Checked } else { CheckState::Unchecked });
            item
        }

        // Floats need to be tweaked to fix trailing zeroes and precision issues, like turning 0.5000004 into 0.5.
        // Also, they should be limited to 3 decimals.
        DecodedData::F32(ref data) => {
            let (data, string) = {
                let data_str = format!("{data}");
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 {
                        let data_str = format!("{data:.4}");
                        (data_str.parse::<f32>().unwrap(), data_str) }
                    else { (*data, data_str) }
                }
                else { (*data, data_str) }
            };

            let qdata = QVariant::from_float(data);
            let item = QStandardItem::new();
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(string)), ITEM_SOURCE_VALUE);
            item.set_data_2a(&qdata, 2);
            item
        },

        DecodedData::F64(ref data) => {
            let (data, string) = {
                let data_str = format!("{data}");
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 {
                        let data_str = format!("{data:.4}");
                        (data_str.parse::<f64>().unwrap(), data_str) }
                    else { (*data, data_str) }
                }
                else { (*data, data_str) }
            };

            let qdata = QVariant::from_double(data);
            let item = QStandardItem::new();
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(string)), ITEM_SOURCE_VALUE);
            item.set_data_2a(&qdata, 2);
            item
        },
        DecodedData::I16(ref data) |
        DecodedData::OptionalI16(ref data) => {
            let item = QStandardItem::new();
            let qdata = QVariant::from_int(*data as i32);
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&qdata, ITEM_SOURCE_VALUE);
            item.set_data_2a(&qdata, 2);
            item
        },
        DecodedData::I32(ref data) |
        DecodedData::OptionalI32(ref data) => {
            let item = QStandardItem::new();
            let qdata = QVariant::from_int(*data);
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&qdata, ITEM_SOURCE_VALUE);
            item.set_data_2a(&qdata, 2);
            item
        },
        DecodedData::I64(ref data) |
        DecodedData::OptionalI64(ref data) => {
            let item = QStandardItem::new();
            let qdata = QVariant::from_i64(*data);
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&qdata, ITEM_SOURCE_VALUE);
            item.set_data_2a(&qdata, 2);
            item
        },

        // All these are Strings, so it can be together,
        DecodedData::ColourRGB(ref data) |
        DecodedData::StringU8(ref data) |
        DecodedData::StringU16(ref data) |
        DecodedData::OptionalStringU8(ref data) |
        DecodedData::OptionalStringU16(ref data) => {
            let qdata = QString::from_std_str(data);
            let item = QStandardItem::from_q_string(&qdata);
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_string(&qdata), ITEM_SOURCE_VALUE);
            item
        },
        DecodedData::SequenceU16(ref data) | DecodedData::SequenceU32(ref data) => {
            let data = QByteArray::from_slice(data);
            let item = QStandardItem::from_q_string(&qtr("packedfile_editable_sequence"));
            item.set_editable(false);
            item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_HAS_SOURCE_VALUE);
            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_IS_SEQUENCE);
            item.set_data_2a(&QVariant::from_q_byte_array(&data), ITEM_SEQUENCE_DATA);
            item
        }
    };

    if keys.contains(&(column as i32)) {
        item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_KEY);
    }

    item
}

/// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
/// His intended use is for just after we load/reload the data to the table.
///
/// Returns if we need to perform a resizing after data is loaded.
pub unsafe fn build_columns(
    table_view: &QPtr<QTableView>,
    definition: &Definition,
    table_name: Option<&str>,
    table_data: &TableType,
) -> bool {
    let filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
    let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

    let schema = SCHEMA.read().unwrap();
    let mut do_we_have_ca_order = false;
    let mut resize_after_data = false;
    let mut keys = vec![];
    let fields_processed = definition.fields_processed();
    let loc_fields = definition.localised_fields();
    model.set_column_count(fields_processed.len() as i32);

    let patches = Some(definition.patches());
    let tooltips = get_column_tooltips(&schema, &fields_processed, loc_fields, patches, table_name);
    let adjust_columns = SETTINGS.read().unwrap().bool("adjust_columns_to_content");
    let header = table_view.horizontal_header();

    let mut columns_to_hide = vec![];
    let hide_unused_columns = SETTINGS.read().unwrap().bool("hide_unused_columns");

    let description_icon = if SETTINGS.read().unwrap().bool("use_dark_theme") {
        QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/description_icon_dark.png", ASSETS_PATH.to_string_lossy())))
    }  else {
        QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/description_icon_light.png", ASSETS_PATH.to_string_lossy())))
    };

    for (index, field) in fields_processed.iter().enumerate() {

        let name = clean_column_names(field.name());
        let item = QStandardItem::from_q_string(&QString::from_std_str(name));
        if let Some(ref tooltip) = tooltips.get(index) {
            item.set_tool_tip(&QString::from_std_str(tooltip));
            if !field.description(patches).is_empty() {
                item.set_icon(&description_icon);
            }
        }   

        model.set_horizontal_header_item(index as i32, item.into_ptr());

        // Depending on his type, set one width or another.
        if !adjust_columns {
            match field.field_type() {
                FieldType::Boolean => table_view.set_column_width(index as i32, COLUMN_SIZE_BOOLEAN),
                FieldType::F32 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::F64 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::I16 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::I32 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::I64 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::OptionalI16 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::OptionalI32 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::OptionalI64 => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::ColourRGB => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::StringU8 => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::StringU16 => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::OptionalStringU8 => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::OptionalStringU16 => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
            }
        } else {

            // Optimized logic to resize columns.
            match table_data {
                TableType::DependencyManager(ref table) => {
                    match field.field_type() {
                        FieldType::StringU8 => {
                            let size = table
                                .par_iter()
                                .max_by_key(|row| row[index].data_to_string().len())
                                .map(|row| row[index].data_to_string().len() * 6)
                                .unwrap_or(COLUMN_SIZE_STRING as usize);
                            table_view.set_column_width(index as i32, size as i32 + 30);
                        }
                        _ => table_view.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                    }
                }
                TableType::DB(ref table) => {

                    // I'm not sure why, but when importing TSVs the resizing sometimes crash due to index out of len errors.
                    // So we only resize if there are enough columns.
                    if table.data().par_iter().any(|row| row.len() < index) {
                        continue;
                    }

                    match field.field_type() {
                        FieldType::Boolean |
                        FieldType::F32 |
                        FieldType::F64 |
                        FieldType::I16 |
                        FieldType::I32 |
                        FieldType::I64 |
                        FieldType::OptionalI16 |
                        FieldType::OptionalI32 |
                        FieldType::OptionalI64 |
                        FieldType::ColourRGB => {
                            let mut size = model.horizontal_header_item(index as i32).text().length() * 6 + 40;

                            // Fix some columns getting their title eaten by description icon.
                            if size < 100 {
                                size = 100;
                            }

                            table_view.set_column_width(index as i32, size);
                        }
                        FieldType::StringU8 |
                        FieldType::StringU16 |
                        FieldType::OptionalStringU8 |
                        FieldType::OptionalStringU16 => {
                            let mut size = table.data()
                                .par_iter()
                                .max_by_key(|row| row[index].data_to_string().len())
                                .map(|row| row[index].data_to_string().len() * 6)
                                .unwrap_or(COLUMN_SIZE_STRING as usize);

                            // Enlarge a bit lookup columns so they show part of the lookup.
                            if field.lookup(patches).is_some() {
                                size += 200;
                            }

                            // Fix some columns getting their title eaten by description icon, and some columns being extremely long.
                            size = size.clamp(60, 800);

                            table_view.set_column_width(index as i32, size as i32 + 30);
                        }
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                    }
                }
                TableType::Loc(ref table) => {
                    match field.field_type() {
                        FieldType::Boolean => table_view.set_column_width(index as i32, model.horizontal_header_item(index as i32).text().length() * 6 + 30),
                        FieldType::StringU16 => {
                            let size = table.data()
                                .par_iter()
                                .max_by_key(|row| row[index].data_to_string().len())
                                .map(|row| row[index].data_to_string().len() * 6)
                                .unwrap_or(COLUMN_SIZE_STRING as usize);
                            table_view.set_column_width(index as i32, size as i32 + 30);
                        }
                        _ => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                    }
                }
                #[cfg(feature = "enable_tools")]
                TableType::TranslatorTable(ref table) => {
                    match field.field_type() {
                        FieldType::Boolean => table_view.set_column_width(index as i32, model.horizontal_header_item(index as i32).text().length() * 6 + 30),
                        FieldType::StringU8 => {
                            let mut size = table.data()
                                .par_iter()
                                .max_by_key(|row| row[index].data_to_string().len())
                                .map(|row| row[index].data_to_string().len() * 6)
                                .unwrap_or(COLUMN_SIZE_STRING as usize);

                            // Fix some columns getting their title eaten by description icon, and some columns being extremely long.
                            size = size.clamp(60, 600);

                            table_view.set_column_width(index as i32, size as i32 + 30);
                        }
                        _ => table_view.set_column_width(index as i32, COLUMN_SIZE_STRING),
                    }
                }

                // Slow logic to resize columns.
                _ => resize_after_data = true,
            }
        }

        // If the field is key, add that column to the "Key" list, so we can move them at the beginning later.
        if field.is_key(patches) { keys.push(index); }
        if field.ca_order() != -1 { do_we_have_ca_order |= true; }

        if hide_unused_columns && field.unused(patches) {
            columns_to_hide.push(index);
        }
    }

    // Now the order. If we have a sort order from the schema, we use that one.
    if !SETTINGS.read().unwrap().bool("tables_use_old_column_order") && do_we_have_ca_order {
        let mut fields = fields_processed.iter()
            .enumerate()
            .map(|(x, y)| (x, y.ca_order()))
            .collect::<Vec<(usize, i16)>>();
        fields.sort_by(|a, b| {
            if a.1 == -1 || b.1 == -1 { Ordering::Equal }
            else { a.1.cmp(&b.1) }
        });

        header.block_signals(true);
        for (new_pos, (logical_index, ca_order)) in fields.iter().enumerate() {
            if *ca_order != -1 {
                let visual_index = header.visual_index(*logical_index as i32);

                header.move_section(visual_index, new_pos as i32);
            }
        }
        header.block_signals(false);
    }

    // Otherwise, if we have any "Key" field, move it to the beginning.
    else if !keys.is_empty() {
        header.block_signals(true);
        for (position, column) in keys.iter().enumerate() {
            header.move_section(*column as i32, position as i32);
        }

        header.block_signals(false);
    }

    for i in &columns_to_hide {
        header.hide_section(*i as i32);
    }

    resize_after_data
}

/// This function sets the tooltip for the provided column header, if the column should have one.
pub unsafe fn get_column_tooltips(
    schema: &Option<Schema>,
    fields: &[Field],
    loc_fields: &[Field],
    patches: Option<&DefinitionPatch>,
    table_name: Option<&str>,
) -> Vec<String> {

    let mut tooltips = vec![];

    // If we passed it a table name, build the tooltip based on it. The logic is simple:
    // - If we have a description, we add it to the tooltip.
    // - If the column references another column, we add it to the tooltip.
    // - If the column is referenced by another column, we add it to the tooltip.
    if let Some(table_name) = table_name {
        if let Some(ref schema) = schema {

            let ref_definitions = schema.definitions();
            tooltips = fields.par_iter().map(|field| {
                let mut tooltip_text = String::new();
                if !field.description(patches).is_empty() {
                    tooltip_text.push_str(&format!("<p>{}</p>", field.description(patches)));
                }

                if field.is_filename(patches) {
                    if let Some(path) = field.filename_relative_path(patches) {
                        tooltip_text.push_str(&format!("<p>{} <ul><li>{}</li></ul></p>", tr("column_tooltip_5"), path.join("</li><li>")));
                    } else {
                        tooltip_text.push_str(&format!("<p>{}</p>", tr("column_tooltip_4")));
                    }
                }

                if let Some(ref lookup) = field.lookup(patches) {
                    if let Some(ref reference) = field.is_reference(patches) {
                        let lookups = lookup.iter()
                            .map(|lookup|
                                if loc_fields.iter().any(|x| x.name() == lookup) {
                                    lookup.to_owned()
                                } else {
                                    format!("{}/{}", reference.0, lookup)
                                }
                            ).join("</i></li><li><i>");

                        tooltip_text.push_str(&format!("<p>{}</p><ul><li><i>{}</i></li></ul>", tr("column_tooltip_lookup_remote"), lookups));
                    } else {
                        tooltip_text.push_str(&format!("<p>{}</p><ul><li><i>{}</i></li></ul>", tr("column_tooltip_lookup_local"), lookup.join("</i></li><li><i>")));
                    }
                }

                if let Some(ref reference) = field.is_reference(patches) {
                    tooltip_text.push_str(&format!("<p>{}</p><p><i>\"{}/{}\"</i></p>", tr("column_tooltip_1"), reference.0, reference.1));
                }

                else {
                    let mut referenced_columns = {
                        let short_table_name = if table_name.ends_with("_tables") { table_name.split_at(table_name.len() - 7).0 } else { table_name };
                        let mut columns = vec![];

                        // We get all the db definitions from the schema, then iterate all of them to find what tables reference our own.
                        for (ref_table_name, ref_definition) in ref_definitions.iter() {
                            let mut found = false;
                            for ref_version in ref_definition {
                                let ref_patches = Some(ref_version.patches());
                                for ref_field in ref_version.fields_processed() {
                                    if let Some((ref_ref_table, ref_ref_field)) = ref_field.is_reference(ref_patches) {
                                        if ref_ref_table == short_table_name && ref_ref_field == field.name() {
                                            found = true;
                                            columns.push((ref_table_name.to_owned(), ref_field.name().to_owned()));
                                        }
                                    }
                                }
                                if found { break; }
                            }
                        }
                        columns
                    };

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
                tooltip_text
            }).collect::<Vec<String>>();
        }
    }

    tooltips
}

pub unsafe fn read_anim_ids_file() -> Result<HashMap<i32, TableReferences>> {
    let path = schemas_path()?.join(format!("anim_ids_{}.csv", GAME_SELECTED.read().unwrap().key()));
    let mut refs_hashmap = HashMap::new();
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .quoting(false)
        .has_headers(false)
        .flexible(false)
        .from_path(path)?;

    reader.records().flatten().for_each(|record| {
        if let Some(id) = record.get(0) {
            if let Some(value) = record.get(3) {
                refs_hashmap.insert(id.trim().to_string(), value.trim().to_string());
            }
        }
    });

    let mut refs = TableReferences::default();
    *refs.data_mut() = refs_hashmap;

    let mut refs_final = HashMap::new();
    refs_final.insert(0, refs);

    Ok(refs_final)
}


pub unsafe fn get_vanilla_hashed_tables(file_type: FileType, table_name: &str) -> Result<Vec<(DB, HashMap<String, i32>)>> {
    match file_type {
        FileType::DB => {

            // Call the backend passing it the files we have open (so we don't get them from the backend too), and get the frontend data while we wait for it to finish.
            let receiver = CENTRAL_COMMAND.send_background(Command::GetTablesFromDependencies(table_name.to_owned()));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::VecRFile(files) => {
                    let mut data = Vec::with_capacity(files.len());
                    for file in files {

                        if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                            let definition = table.definition();
                            let key_pos = definition.key_column_positions();

                            if !key_pos.is_empty() {
                                let mut hashes = HashMap::new();
                                for (index, row) in table.data().iter().enumerate() {
                                    let keys = key_pos.iter()
                                        .map(|x| row[*x].data_to_string())
                                        .join("");

                                    hashes.insert(keys, index as i32);
                                }

                                data.push((table.clone(), hashes));
                            }
                        }
                    }

                    Ok(data)
                },
                Response::Error(error) => Err(error),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        _ => Ok(vec![])
    }
}

/// This function returns the reference data for an entire table.
pub unsafe fn get_reference_data(file_type: FileType, table_name: &str, definition: &Definition) -> Result<HashMap<i32, TableReferences>> {
    match file_type {

        // For AnimFragmentBattle files, return the custom lookups for the animation id column.
        FileType::AnimFragmentBattle => Ok(read_anim_ids_file().unwrap_or_else(|_| HashMap::new())),
        FileType::DB => {

            // Call the backend passing it the files we have open (so we don't get them from the backend too), and get the frontend data while we wait for it to finish.
            let receiver = CENTRAL_COMMAND.send_background(Command::GetReferenceDataFromDefinition(table_name.to_owned(), definition.clone()));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::HashMapI32TableReferences(dependency_data) => Ok(dependency_data),
                Response::Error(error) => Err(error),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        _ => Ok(HashMap::new())
    }
}

/// This function sets up the item delegates for all columns in a table.
pub unsafe fn setup_item_delegates(
    table_view: &QPtr<QTableView>,
    definition: &Definition,
    table_references: &HashMap<i32, TableReferences>,
    timer: &QBox<QTimer>
) {
    let table_object = table_view.static_upcast::<QObject>().as_ptr();
    let enable_lookups = SETTINGS.read().unwrap().bool("enable_lookups");

    for (column, field) in definition.fields_processed().iter().enumerate() {
        let references = table_references.get(&(column as i32));

        // Combos are a bit special, as they may or may not replace other delegates. If we disable them, use the normal delegates.
        if !SETTINGS.read().unwrap().bool("disable_combos_on_tables") && references.is_some() || !field.enum_values().is_empty() {
            let values = QStringList::new();
            let lookups = QStringList::new();
            if let Some(data) = references {
                let mut data = data.data().iter().collect::<Vec<(&String, &String)>>();
                data.sort_by_key(|x| x.0);
                data.iter().for_each(|x| {
                    values.append_q_string(&QString::from_std_str(x.0));
                    if enable_lookups {
                        lookups.append_q_string(&QString::from_std_str(x.1));
                    }
                });
            }

            // TODO: Rework the enum system to work like lookups.
            if !field.enum_values().is_empty() {
                field.enum_values().values().for_each(|x| {
                    values.append_q_string(&QString::from_std_str(x));
                });
            }

            new_combobox_item_delegate_safe(&table_object, column as i32, values.into_ptr(), lookups.into_ptr(), true, &timer.as_ptr(), true);
        }

        else {
            match field.field_type() {
                FieldType::Boolean => new_generic_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
                FieldType::F32 => new_doublespinbox_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
                FieldType::F64 => new_doublespinbox_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
                FieldType::I16 => new_spinbox_item_delegate_safe(&table_object, column as i32, 16, &timer.as_ptr(), true),
                FieldType::I32 => new_spinbox_item_delegate_safe(&table_object, column as i32, 32, &timer.as_ptr(), true),

                // LongInteger uses normal string controls due to QSpinBox being limited to i32.
                FieldType::I64 => new_spinbox_item_delegate_safe(&table_object, column as i32, 64, &timer.as_ptr(), true),
                FieldType::OptionalI16 => new_spinbox_item_delegate_safe(&table_object, column as i32, 16, &timer.as_ptr(), true),
                FieldType::OptionalI32 => new_spinbox_item_delegate_safe(&table_object, column as i32, 32, &timer.as_ptr(), true),

                // LongInteger uses normal string controls due to QSpinBox being limited to i32.
                FieldType::OptionalI64 => new_spinbox_item_delegate_safe(&table_object, column as i32, 64, &timer.as_ptr(), true),
                FieldType::ColourRGB => new_colour_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
                FieldType::StringU8 |
                FieldType::StringU16 |
                FieldType::OptionalStringU8 |
                FieldType::OptionalStringU16 => new_qstring_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
                FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => new_generic_item_delegate_safe(&table_object, column as i32, &timer.as_ptr(), true),
            }
        }
    }
}

/// This function is a generic way to toggle the sort order of a column.
pub unsafe fn sort_column(
    table_view: &QPtr<QTableView>,
    column: i32,
    column_sort_state: Arc<RwLock<(i32, i8)>>
) {
    let mut needs_cleaning = false;
    {
        // We only change the order if it's less than 2. Otherwise, we reset it.
        let mut sort_data = column_sort_state.write().unwrap();
        let mut old_order = if sort_data.0 == column { sort_data.1 } else { 0 };

        if old_order < 2 {
            old_order += 1;
            if old_order == 0 { *sort_data = (-1, old_order); }
            else { *sort_data = (column, old_order); }
        }
        else {
            needs_cleaning = true;
            *sort_data = (-1, -1);
        }
    }

    if needs_cleaning {
        table_view.horizontal_header().set_sort_indicator(-1, SortOrder::AscendingOrder);
    }
}

/// This function is used to build a table struct with the data of a TableView and it's definition.
pub unsafe fn get_table_from_view(
    model: &QPtr<QStandardItemModel>,
    definition: &Definition
) -> Result<Table> {
    let mut entries = vec![];

    for row in 0..model.row_count_0a() {
        let mut new_row: Vec<DecodedData> = vec![];

        // Bitwise columns can span across multiple columns. That means we have to keep track of the column ourselves.
        for (column, field) in definition.fields_processed().iter().enumerate() {
            let item = get_field_from_view(model, field, row, column as i32);
            new_row.push(item);
        }
        entries.push(new_row);
    }

    let mut table = Table::new(definition, None, "");
    table.set_data(&entries)?;
    Ok(table)
}

pub unsafe fn get_field_from_view(model: &QPtr<QStandardItemModel>, field: &Field, row: i32, column: i32) -> DecodedData {
    match field.field_type() {

        // This one needs a couple of changes before turning it into an item in the table.
        FieldType::Boolean => DecodedData::Boolean(model.item_2a(row, column).check_state() == CheckState::Checked),

        // Numbers need parsing, and this can fail.
        FieldType::F32 => DecodedData::F32(model.item_2a(row, column).data_1a(2).to_float_0a()),
        FieldType::F64 => DecodedData::F64(model.item_2a(row, column).data_1a(2).to_double_0a()),
        FieldType::I16 => DecodedData::I16(model.item_2a(row, column).data_1a(2).to_int_0a() as i16),
        FieldType::I32 => DecodedData::I32(model.item_2a(row, column).data_1a(2).to_int_0a()),
        FieldType::I64 => DecodedData::I64(model.item_2a(row, column).data_1a(2).to_long_long_0a()),
        FieldType::OptionalI16 => DecodedData::OptionalI16(model.item_2a(row, column).data_1a(2).to_int_0a() as i16),
        FieldType::OptionalI32 => DecodedData::OptionalI32(model.item_2a(row, column).data_1a(2).to_int_0a()),
        FieldType::OptionalI64 => DecodedData::OptionalI64(model.item_2a(row, column).data_1a(2).to_long_long_0a()),

        // Colours need parsing to turn them into integers.
        FieldType::ColourRGB => DecodedData::ColourRGB(QString::to_std_string(&model.item_2a(row, column).text())),

        // All these are just normal Strings.
        FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.item_2a(row, column).text())),
        FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.item_2a(row, column).text())),
        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.item_2a(row, column).text())),
        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.item_2a(row, column).text())),

        // Sequences in the UI are not yet supported.
        FieldType::SequenceU16(_) => DecodedData::SequenceU16(model.item_2a(row, column).data_1a(ITEM_SEQUENCE_DATA).to_byte_array().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>()),
        FieldType::SequenceU32(_) => DecodedData::SequenceU32(model.item_2a(row, column).data_1a(ITEM_SEQUENCE_DATA).to_byte_array().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>()),
    }
}

/// This function creates a new subtable from the current table.
pub unsafe fn open_subtable(
    model_index: Ref<QModelIndex>,
    view: &Arc<TableView>,
    app_ui: &Rc<AppUI>,
    global_search_ui: &Rc<GlobalSearchUI>,
    pack_file_contents_ui: &Rc<PackFileContentsUI>,
    diagnostics_ui: &Rc<DiagnosticsUI>,
    dependencies_ui: &Rc<DependenciesUI>,
    references_ui: &Rc<ReferencesUI>,
    default_selection: Option<(i32, i32)>
) {

    if model_index.data_1a(ITEM_IS_SEQUENCE).to_bool() {
        let mut data = Cursor::new(model_index.data_1a(ITEM_SEQUENCE_DATA).to_byte_array().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
        let definition = view.table_definition();
        let fields_processed = definition.fields_processed();
        if let Some(field) = fields_processed.get(model_index.column() as usize) {
            if let FieldType::SequenceU32(definition) = field.field_type() {
                if let Ok(table) = Table::decode(&mut data, definition, &HashMap::new(), None, false, field.name()) {
                    let table_data = match *view.packed_file_type {
                        FileType::AnimFragmentBattle => TableType::AnimFragmentBattle(table),
                        FileType::DB => TableType::DB(From::from(table)),
                        FileType::Loc => TableType::Loc(From::from(table)),
                        _ => unimplemented!("You forgot to implement subtables for this kind of packedfile"),
                    };

                    // Create and configure the dialog.
                    let dialog = QDialog::new_1a(view.table_view().static_upcast::<QWidget>());
                    dialog.set_window_title(&qtr("nested_table_title"));
                    dialog.set_modal(true);
                    dialog.resize_2a(1200, 600);

                    let main_grid = create_grid_layout(dialog.static_upcast());
                    let main_widget = QWidget::new_1a(&dialog);
                    let _widget_grid = create_grid_layout(main_widget.static_upcast());
                    let accept_button = QPushButton::from_q_string(&qtr("nested_table_accept"));

                    let table_view = TableView::new_view(&main_widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, table_data, None, view.data_source.clone()).unwrap();

                    main_grid.add_widget_5a(&main_widget, 0, 0, 1, 1);
                    main_grid.add_widget_5a(&accept_button, 1, 0, 1, 1);

                    // If we have a default selection, scroll to it and select it.
                    if let Some((x, y)) = default_selection {
                        let item_to_select = table_view.table_model().index_2a(x, y);
                        let item_to_select_filter = table_view.table_filter().map_from_source(&item_to_select);

                        let selection = table_view.table_view().selection_model().selection();
                        table_view.table_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                        table_view.table_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                        table_view.table_view().set_focus_0a();
                        table_view.table_view().set_current_index(item_to_select_filter.as_ref());
                        table_view.table_view().scroll_to_2a(item_to_select_filter.as_ref(), ScrollHint::EnsureVisible);
                    }

                    accept_button.released().connect(dialog.slot_accept());

                    if dialog.exec() == 1 {
                        if let Ok(table) = get_table_from_view(&table_view.table_model.static_upcast(), &table_view.table_definition()) {
                            let mut data = Cursor::new(vec![]);

                            // Subtables come from Sequence fields, and in those we NEED to manually write the amount of rows before the data, or we will get broken data.
                            data.write_u32(table.data().len() as u32).unwrap();
                            let _ = table.encode(&mut data, &None);

                            view.table_filter().set_data_3a(
                                model_index,
                                &QVariant::from_q_byte_array(&QByteArray::from_slice(&data.into_inner())),
                                ITEM_SEQUENCE_DATA
                            );
                        } else {
                            show_dialog(&table_view.table_view, "This should never happen.", false);
                        }
                    }
                }
            }
        }
    }
}

pub unsafe fn request_backend_files(data: &[Vec<DecodedData>], column: usize, field: &Field, patches: Option<&DefinitionPatch>, map: &mut BTreeMap<i32, (String, HashMap<String, AtomicPtr<QIcon>>)>) -> Result<()> {
    let relative_path = field.filename_relative_path(patches);
    let empty_path = vec!["%".to_owned()];
    let base_path = relative_path.as_deref().unwrap_or(&empty_path);
    let paths = data.par_iter()
        .flat_map(|entry| base_path.iter()
            .map(|base_path| {
                let cell_data = entry[column].data_to_string().replace('\\', "/");

                // For paths, we need to fix the ones in older games starting with / or data/.
                let mut start_offset = 0;
                if cell_data.starts_with("/") {
                    start_offset += 1;
                }
                if cell_data.starts_with("data/") {
                    start_offset += 5;
                }

                ContainerPath::File(base_path.replace('%', &cell_data[start_offset..]))
            })
            .collect::<Vec<_>>()
        ).collect::<Vec<_>>();

    if !paths.is_empty() {
        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(paths, true));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                let mut files_merge = HashMap::new();
                if let Some(files) = files.remove(&DataSource::GameFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::ParentFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::PackFile) {
                    files_merge.extend(files);
                }

                let icons = files_merge.par_iter_mut()
                    .filter_map(|(path, file)| {
                        if file.file_type() == FileType::Image {
                            if let Ok(Some(RFileDecoded::Image(data))) = file.decode(&None, false, true) {
                                let byte_array = QByteArray::from_slice(data.data());
                                let image = QPixmap::new();

                                if image.load_from_data_q_byte_array(&byte_array) {
                                    let icon = QIcon::from_q_pixmap(&image);
                                    Some((path.to_owned(), atomic_from_ptr(icon.into_ptr())))
                                } else { None }
                            } else { None }
                        } else { None }
                    })
                    .collect::<HashMap<String, AtomicPtr<QIcon>>>();

                map.insert(column as i32, (base_path.join(";"), icons));
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
    }

    Ok(())
}
