//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to deal with the raw version of the tables.
!*/

use itertools::Itertools;

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QSpinBox;
use qt_widgets::QTextEdit;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QGuiApplication;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QFlags;
use qt_core::QItemSelection;
use qt_core::QString;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;
use qt_core::Orientation;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QSignalBlocker;
use qt_core::QPtr;

use qt_ui_tools::QUiLoader;

use cpp_core::CppBox;
use cpp_core::Ref;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::Ordering;

use rpfm_error::Error;

use rpfm_lib::packedfile::table::db::CascadeEdition;
use rpfm_lib::packedfile::table::Table;
use rpfm_lib::schema::patch::SchemaPatch;

use crate::ASSETS_PATH;
use crate::dependencies_ui::DependenciesUI;
use crate::ffi::*;
use crate::locale::tr;
use crate::packedfile_views::utils::set_modified;
use crate::packedfile_views::DataSource;
use crate::pack_tree::*;
use crate::UI_STATE;
use crate::utils::{atomic_from_ptr, create_grid_layout, find_widget, log_to_status_bar};
use super::*;

const PATCH_COLUMN_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_schema_patch_dialog.ui";
const PATCH_COLUMN_VIEW_RELEASE: &str = "ui/new_schema_patch_dialog.ui";

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TableView`.
impl TableView {

    /// This function updates the state of the actions in the context menu.
    pub unsafe fn context_menu_update(&self) {

        // Disable everything, just in case.
        self.context_menu_add_rows.set_enabled(false);
        self.context_menu_insert_rows.set_enabled(false);
        self.context_menu_clone_and_append.set_enabled(false);
        self.context_menu_clone_and_insert.set_enabled(false);
        self.context_menu_delete_rows.set_enabled(false);
        self.context_menu_delete_rows_not_in_filter.set_enabled(false);
        self.context_menu_paste.set_enabled(false);
        self.context_menu_paste_as_new_row.set_enabled(false);
        self.context_menu_rewrite_selection.set_enabled(false);
        self.context_menu_generate_ids.set_enabled(false);
        self.context_menu_undo.set_enabled(false);
        self.context_menu_redo.set_enabled(false);
        self.context_menu_import_tsv.set_enabled(false);
        self.context_menu_cascade_edition.set_enabled(false);
        self.context_menu_patch_column.set_enabled(true);
        self.smart_delete.set_enabled(false);

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselves.
        let indexes = self.table_filter.map_selection_to_source(&self.table_view_primary.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            self.context_menu_copy.set_enabled(true);
            self.context_menu_copy_as_lua_table.set_enabled(true);

            if *self.packed_file_type == PackedFileType::DB {
                self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(true));
            } else {
                self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(false));
            }

            if [PackedFileType::DB, PackedFileType::Loc].contains(&self.packed_file_type) {
                self.context_menu_go_to_definition.set_enabled(true);
            } else {
                self.context_menu_go_to_definition.set_enabled(false);
            }
        }

        // Otherwise, disable them.
        else {
            self.context_menu_copy.set_enabled(false);
            self.context_menu_copy_as_lua_table.set_enabled(false);
            self.context_menu_go_to_definition.set_enabled(false);
            self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(false));
        }

        // Only enable editing if the table is ours and not banned.
        if let DataSource::PackFile = self.get_data_source() {
            if !self.banned_table {

                // These ones are always enabled if the table is editable.
                self.context_menu_add_rows.set_enabled(true);
                self.context_menu_insert_rows.set_enabled(true);
                self.context_menu_delete_rows_not_in_filter.set_enabled(true);
                self.context_menu_paste_as_new_row.set_enabled(true);
                self.context_menu_import_tsv.set_enabled(true);
                self.smart_delete.set_enabled(true);

                // If we have something selected, enable these actions.
                if indexes.count_0a() > 0 {
                    self.context_menu_clone_and_append.set_enabled(true);
                    self.context_menu_clone_and_insert.set_enabled(true);
                    self.context_menu_delete_rows.set_enabled(true);
                    self.context_menu_paste.set_enabled(true);
                    self.context_menu_rewrite_selection.set_enabled(true);
                    self.context_menu_generate_ids.set_enabled(true);
                    self.context_menu_cascade_edition.set_enabled(true);
                }

                if !self.undo_lock.load(Ordering::SeqCst) {
                    self.context_menu_undo.set_enabled(!self.history_undo.read().unwrap().is_empty());
                    self.context_menu_redo.set_enabled(!self.history_redo.read().unwrap().is_empty());
                }
            }
        }
    }

    /// Function to filter the table.
    pub unsafe fn filter_table(&self) {
        let mut columns = vec![];
        let mut patterns = vec![];
        let mut sensitivity = vec![];
        let mut show_blank_cells = vec![];
        let mut match_groups = vec![];

        let filters = self.filters.read().unwrap();
        for filter in filters.iter() {

            // Ignore empty filters.
            if !filter.filter_line_edit.text().to_std_string().is_empty() {

                let column_name = filter.filter_column_selector.current_text();
                for column in 0..self.table_model.column_count_0a() {
                    if self.table_model.header_data_2a(column, Orientation::Horizontal).to_string().compare_q_string_case_sensitivity(&column_name, CaseSensitivity::CaseSensitive) == 0 {
                        columns.push(column);
                        break;
                    }
                }

                // Check if the filter should be "Case Sensitive".
                let case_sensitive = filter.filter_case_sensitive_button.is_checked();
                if case_sensitive { sensitivity.push(CaseSensitivity::CaseSensitive); }
                else { sensitivity.push(CaseSensitivity::CaseInsensitive); }

                // Check if we should filter out blank cells or not.
                show_blank_cells.push(filter.filter_show_blank_cells_button.is_checked());

                patterns.push(filter.filter_line_edit.text().into_ptr());
                match_groups.push(filter.filter_match_group_selector.current_index());
            }
        }

        // Filter whatever it's in that column by the text we got.
        trigger_tableview_filter_safe(&self.table_filter, &columns, patterns, &sensitivity, &show_blank_cells, &match_groups);

        // Update the line count.
        self.update_line_counter();
    }

    /// This function enables/disables showing the lookup values instead of the real ones in the columns that support it.
    pub unsafe fn toggle_lookups(&self) {
        /*
        if SETTINGS.lock().unwrap().settings_bool["disable_combos_on_tables"] {
            let enable_lookups = unsafe { self.table_enable_lookups_button.is_checked() };
            for (column, field) in table_definition.fields.iter().enumerate() {
                if let Some(data) = dependency_data.get(&(column as i32)) {
                    let mut list = QStringList::new(());
                    data.iter().map(|x| if enable_lookups { &x.1 } else { &x.0 }).for_each(|x| list.append(&QString::from_std_str(x)));
                    let list: *mut QStringList = &mut list;
                    unsafe { new_combobox_item_delegate_safe(self.table_view_primary as *mut QObject, column as i32, list as *const QStringList, true, field.max_length)};
                    unsafe { new_combobox_item_delegate_safe(self.table_view_frozen as *mut QObject, column as i32, list as *const QStringList, true, field.max_length)};
                }
            }
        }*/
    }

    /// This function resets the currently selected cells to their original value.
    pub unsafe fn reset_selection(&self) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes_sorted = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        let mut items_reverted = 0;
        for index in &indexes_sorted {
            if index.is_valid() {
                let item = self.table_model.item_from_index(index);
                if item.data_1a(ITEM_HAS_SOURCE_VALUE).to_bool() {
                    let original_data = item.data_1a(ITEM_SOURCE_VALUE);
                    let current_data = item.data_1a(2);
                    if original_data != current_data.as_ref() {
                        item.set_data_2a(&original_data, 2);
                        items_reverted += 1;
                    }
                }
            }
        }

        // Fix the undo history to have all the previous changed merged into one.
        if items_reverted > 0 {
            {
                let mut history_undo = self.history_undo.write().unwrap();
                let mut history_redo = self.history_redo.write().unwrap();

                let len = history_undo.len();
                let mut edits_data = vec![];
                {
                    let mut edits = history_undo.drain((len - items_reverted)..);
                    for edit in &mut edits {
                        if let TableOperations::Editing(mut edit) = edit {
                            edits_data.append(&mut edit);
                        }
                    }
                }

                history_undo.push(TableOperations::Editing(edits_data));
                history_redo.clear();
            }
            self.start_delayed_updates_timer();
            update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
        }
    }

    /// This function rewrite the currently selected cells using the provided formula.
    pub unsafe fn rewrite_selection(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if let Some((is_math_operation, value)) = self.create_rewrite_selection_dialog() {
            let horizontal_header = self.table_view_primary.horizontal_header();

            // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
            let indexes = self.table_view_primary.selection_model().selection().indexes();
            let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
            sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());

            let mut real_cells = vec![];
            let mut values = vec![];

            let mut row = 0;
            let mut prev_row = None;
            for index in &indexes_sorted {
                if index.is_valid() {

                    // Row depends on the selection. If none, it's the first row.
                    if prev_row.is_none() {
                        prev_row = Some(index.row());
                    }

                    // If row changed, + 1.
                    if let Some(ref mut prev_row) = prev_row {
                        if *prev_row != index.row() {
                            *prev_row = index.row();
                            row += 1;
                        }
                    }

                    // Get the column of that cell, the row, the current value, and the new value.
                    let item = self.table_model.item_from_index(&self.table_filter.map_to_source(*index));
                    let column = horizontal_header.visual_index(index.column());
                    let current_value = item.text().to_std_string();
                    let new_value = value.replace("{x}", &current_value).replace("{X}", &current_value)
                        .replace("{y}", &column.to_string()).replace("{Y}", &column.to_string())
                        .replace("{z}", &row.to_string()).replace("{Z}", &row.to_string());

                    let text = if is_math_operation {
                         if let Ok(result) = meval::eval_str(&new_value) {

                            // If we got a current value and it's different, it's a valid cell.
                            match current_value.parse::<f64>() {
                                Ok(value) => {
                                    if (result - value).abs() >= std::f64::EPSILON {
                                        result.to_string()
                                    } else {
                                        current_value.to_owned()
                                    }
                                },
                                Err(_) => result.to_string(),
                            }
                        }

                        // If meval fails, it's not a valid operation for this cell
                        else { continue; }
                    } else { new_value.to_owned() };

                    real_cells.push(self.table_filter.map_to_source(*index));
                    values.push(text);
                }
            }

            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), &*values[index]));
            }
            realer_cells.reverse();

            self.set_data_on_cells(&realer_cells, 0, &[], app_ui, pack_file_contents_ui);
        }
    }

    /// This function fills the currently provided cells with a set of ids.
    pub unsafe fn generate_ids(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());

        // Get the initial value of the dialog.
        let initial_value = if let Some(first) = indexes_sorted.first() {
            if first.is_valid() {
                if let Ok(data) = self.table_filter.map_to_source(*first).data_0a().to_string().to_std_string().parse::<i32>() {
                    data
                } else { 0 }
            } else { 0 }
        } else { 0 };

        if let Some(value) = self.create_generate_ids_dialog(initial_value) {
            let mut real_cells = vec![];
            let mut values = vec![];

            for (id, index) in indexes_sorted.iter().enumerate() {
                if index.is_valid() {
                    real_cells.push(self.table_filter.map_to_source(*index));
                    values.push((value + id as i32).to_string());
                }
            }

            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), &*values[index]));
            }
            realer_cells.reverse();

            self.set_data_on_cells(&realer_cells, 0, &[], app_ui, pack_file_contents_ui);
        }
    }

    /// This function copies the selected cells into the clipboard as a TSV file, so you can paste them in other programs.
    pub unsafe fn copy_selection(&self) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes_sorted = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        // Create a string to keep all the values in a TSV format (x\tx\tx) and populate it.
        let mut copy = String::new();
        let mut row = 0;
        for (cycle, model_index) in indexes_sorted.iter().enumerate() {
            if model_index.is_valid() {

                // If this is the first time we loop, get the row. Otherwise, Replace the last \t with a \n and update the row.
                if cycle == 0 { row = model_index.row(); }
                else if model_index.row() != row {
                    copy.pop();
                    copy.push('\n');
                    row = model_index.row();
                }

                // If it's checkable, we need to get a bool. Otherwise it's a String.
                let item = self.table_model.item_from_index(model_index);
                if item.is_checkable() {
                    match item.check_state() {
                        CheckState::Checked => copy.push_str("true"),
                        CheckState::Unchecked => copy.push_str("false"),
                        _ => return
                    }
                }

                // Fix for weird precision issues on copy.
                else if self.table_definition.read().unwrap().get_fields_processed()[model_index.column() as usize].get_field_type() == FieldType::F32 {
                    copy.push_str(&format!("{}", (item.data_1a(2).to_float_0a() * 1000.0).round() / 1000.0));
                }
                else { copy.push_str(&QString::to_std_string(&item.text())); }

                // Add a \t to separate fields except if it's the last field.
                if cycle < (indexes_sorted.len() - 1) { copy.push('\t'); }
            }
        }

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(copy));
    }

    /// This function copies the selected cells into the clipboard as a LUA Table, so you can use it in LUA scripts.
    pub unsafe fn copy_selection_as_lua_table(&self) {

        // Get the selection sorted visually.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes_sorted = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        // Split the indexes in two groups: those who have a key column selected and those who haven't.
        // Keep in mind this doesn't check what key column we have selected.
        //
        // TODO: Improve this.
        let (intexed_keys, indexes_no_keys): (Vec<Ref<QModelIndex>>, Vec<Ref<QModelIndex>>) = indexes_sorted.iter()
            .filter_map(|x| if x.column() != -1 { Some(x.as_ref()) } else { None })
            .partition(|x|
                indexes_sorted.iter()
                    .filter(|y| y.row() == x.row())
                    .any(|z| self.get_ref_table_definition().get_fields_processed()[z.column() as usize].get_is_key())
            );

        let mut lua_table = self.get_indexes_as_lua_table(&intexed_keys, true);
        lua_table.push('\n');
        lua_table.push_str(&self.get_indexes_as_lua_table(&indexes_no_keys, false));

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(lua_table));
    }

    /// This function allow us to paste the contents of the clipboard into new rows at the end of the table, if the content is compatible with them.
    pub unsafe fn paste_as_new_row(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let mut text = QGuiApplication::clipboard().text().to_std_string();
        if text.ends_with('\n') { text.pop(); }
        let rows = text.split('\n').collect::<Vec<&str>>();
        let rows = rows.iter().map(|x| x.split('\t').collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

        // Then paste the data as it fits. If no indexes are provided, the data is pasted in new rows.
        self.paste_as_it_fits(&rows, &[], app_ui, pack_file_contents_ui);
    }

    /// This function allow us to paste the contents of the clipboard into the selected cells, if the content is compatible with them.
    ///
    /// This function has some... tricky stuff:
    /// - There are several special behaviors when pasting, in order to provide an Excel-Like pasting experience.
    pub unsafe fn paste(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the current selection. We treat it like a TSV, for compatibility with table editors.
        // Also, if the text ends in \n, remove it. Excel things.
        let mut text = QGuiApplication::clipboard().text().to_std_string();
        if text.ends_with('\n') { text.pop(); }
        let rows = text.split('\n').collect::<Vec<&str>>();
        let rows = rows.iter().map(|x| x.split('\t').collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

        // Get the current selection and his, visually speaking, first item (top-left).
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());

        // If nothing is selected, got back to where you came from.
        if indexes_sorted.is_empty() { return }

        // At this point we should have the strings to paste and the selection. Now, clever pasting ahead:
        // - If the entire selection are rows of the same amount of cells and we have only one row of text with the exact same amount
        //   of items as the rows, we paste the same row in each selected row.
        // - If we only have one TSV value in the text and a ton of cells selected, paste the same value everywhere.
        // - In any other case, pick the first selected cell, and paste the TSV using that as cell 0,0.
        let same_amount_of_cells_selected_per_row = if rows.len() == 1 {
            let mut row = -1;
            let mut items = 0;
            let mut is_valid = true;
            for index in &indexes_sorted {
                if row == -1 {
                    row = index.row();
                }

                if index.row() == row {
                    items += 1;
                } else {
                    if items < rows[0].len() {
                        is_valid = false;
                        break;
                    }
                    row = index.row();
                    items = 1
                }

                if items > rows[0].len() {
                    is_valid = false;
                    break;
                }
            }
            is_valid
        } else { false };

        // Amount of rows selected, to ensure certain behavior only triggers when we got the correct number of rows selected.
        let mut rows_selected = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();
        rows_selected.sort_unstable();
        rows_selected.dedup();

        if rows.len() == 1 && rows[0].len() == 1 {
            self.paste_one_for_all(rows[0][0], &indexes_sorted, app_ui, pack_file_contents_ui);
        }

        else if rows.len() == 1 && same_amount_of_cells_selected_per_row && rows_selected.len() > 1 {
            self.paste_same_row_for_all(&rows[0], &indexes_sorted, app_ui, pack_file_contents_ui);
        }

        else {
            self.paste_as_it_fits(&rows, &indexes_sorted, app_ui, pack_file_contents_ui);
        }
    }

    /// This function pastes the value in the clipboard in every selected Cell.
    unsafe fn paste_one_for_all(&self, text: &str, indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let real_cells = indexes.iter().map(|index| {
            (self.table_filter.map_to_source(*index), text)
        }).collect::<Vec<(CppBox<QModelIndex>, &str)>>();
        self.set_data_on_cells(&real_cells, 0, &[], app_ui, pack_file_contents_ui);
    }

    /// This function pastes the row in the clipboard in every selected row that has the same amount of items selected as items in the clipboard we have.
    unsafe fn paste_same_row_for_all(&self, text: &[&str], indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let real_cells = indexes.iter().filter_map(|index| {
            if index.column() == -1 {
                None
            } else {
                text.get(index.column() as usize).map(|text| (self.table_filter.map_to_source(*index), *text))
            }
        }).collect::<Vec<(CppBox<QModelIndex>, &str)>>();

        self.set_data_on_cells(&real_cells, 0, &[], app_ui, pack_file_contents_ui);
    }

    /// This function pastes the provided text into the table as it fits, following a square strategy starting in the first selected index.
    unsafe fn paste_as_it_fits(&self, text: &[Vec<&str>], indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // We're going to try and check in square mode. That means, start in the selected cell, then right
        // until we reach a \n, then return to the initial column. Due to how sorting works, we have to do
        // a test pass first and get all the real AND VALID indexes, then try to paste on them.
        let horizontal_header = self.table_view_primary.horizontal_header();
        let vertical_header = self.table_view_primary.vertical_header();

        // Get the base index of the square. If no index is being provided, we assume we have to paste all in new rows.
        let (base_index_visual, mut visual_row) = if !indexes.is_empty() {
            (Some(&indexes[0]), vertical_header.visual_index(indexes[0].row()))
        } else {
            (None, self.table_model.row_count_0a())
        };

        let definition = self.get_ref_table_definition();
        let fields_processed = definition.get_fields_processed();

        let mut real_cells = vec![];
        let mut added_rows = 0;
        for row in text {
            let mut visual_column = match base_index_visual {
                Some(base_index_visual) => horizontal_header.visual_index(base_index_visual.column()),
                None => 0,
            };

            let mut real_row = self.table_filter.map_to_source(&self.table_filter.index_2a(visual_row, visual_column)).row();

            for text in row {

                // Depending on the column, we try to encode the data in one format or another, or we just skip it.
                let real_column = horizontal_header.logical_index(visual_column);

                if let Some(field) = fields_processed.get(real_column as usize) {

                    // Check if, according to the definition, we have a valid value for the type.
                    let is_valid_data = match field.get_ref_field_type() {
                        FieldType::Boolean => !(text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != &"1" && text != &"0"),
                        FieldType::F32 => text.parse::<f32>().is_ok(),
                        FieldType::F64 => text.parse::<f64>().is_ok(),
                        FieldType::I16 => text.parse::<i16>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::I32 => text.parse::<i32>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::I64 => text.parse::<i64>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::ColourRGB => u32::from_str_radix(text, 16).is_ok(),

                        // All these are Strings, so we can skip their checks....
                        FieldType::StringU8 |
                        FieldType::StringU16 |
                        FieldType::OptionalStringU8 |
                        FieldType::OptionalStringU16 => true,

                        // Ignore sequences.
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => false,
                    };

                    // If it's valid, add it to the real_cells list.
                    if is_valid_data {

                        // If real_row is -1 (invalid), then we need to add an empty row to the model (NOT TO THE FILTER)
                        // because that means we have no row for that position, and we need one.
                        if real_row == -1 {
                            let row = get_new_row(&self.get_ref_table_definition(), self.get_ref_table_name().as_deref());
                            for index in 0..row.count_0a() {
                                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                            }
                            self.table_model.append_row_q_list_of_q_standard_item(&row);
                            real_row = self.table_model.row_count_0a() - 1;
                            added_rows += 1;
                        }
                        real_cells.push((self.table_model.index_2a(real_row, real_column), *text));
                    }
                }
                visual_column += 1;
            }
            visual_row += 1;
        }

        // We need to update the undo model here, because otherwise it'll start triggering crashes
        // in case the first thing to paste is equal to the current value. In that case, the set_data
        // will not trigger, and the update_undo_model will not trigger either, causing a crash if
        // immediately after that we try to paste something in a new line (which will not exist in the undo model).
        {
            update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
        }

        self.set_data_on_cells(&real_cells, added_rows, &[], app_ui, pack_file_contents_ui);
    }

    /// Function to undo/redo an operation in the table.
    ///
    /// If undo = true we are undoing. Otherwise we are redoing.
    /// NOTE: repeat_x_times is for internal recursion!!! ALWAYS PUT A 0 THERE!!!.
    pub unsafe fn undo_redo(
        &self,
        undo: bool,
        mut repeat_x_times: usize,
    ) {
        let filter: QPtr<QSortFilterProxyModel> = self.table_view_primary.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let mut is_carolina = false;

        {
            let (mut history_source, mut history_opposite) = if undo {
                (self.history_undo.write().unwrap(), self.history_redo.write().unwrap())
            } else {
                (self.history_redo.write().unwrap(), self.history_undo.write().unwrap())
            };

            // Get the last operation in the Undo History, or return if there is none.
            let operation = if let Some(operation) = history_source.pop() { operation } else { return };
            log_to_status_bar(&format!("{:?}", operation));
            match operation {
                TableOperations::Editing(editions) => {

                    // Prepare the redo operation, then do the rest.
                    let mut redo_editions = vec![];
                    editions.iter().for_each(|x| redo_editions.push((((x.0).0, (x.0).1), atomic_from_ptr((&*model.item_2a((x.0).0, (x.0).1)).clone()))));
                    history_opposite.push(TableOperations::Editing(redo_editions));

                    self.undo_lock.store(true, Ordering::SeqCst);
                    for (index, ((row, column), item)) in editions.iter().enumerate() {
                        let item = &*ptr_from_atomic(item);
                        model.set_item_3a(*row, *column, item.clone());

                        // If we are going to process the last one, unlock the save.
                        if index == editions.len() - 1 {
                            model.item_2a(*row, *column).set_data_2a(&QVariant::from_int(1i32), 16);
                            model.item_2a(*row, *column).set_data_2a(&QVariant::new(), 16);
                        }
                    }

                    // Select all the edited items.
                    let selection_model = self.table_view_primary.selection_model();
                    selection_model.clear();

                    // TODO: This is still very slow. We need some kind of range optimization.
                    let _blocker = QSignalBlocker::from_q_object(&selection_model);
                    for ((row, column),_) in &editions {
                        let model_index_filtered = filter.map_from_source(&model.index_2a(*row, *column));
                        if model_index_filtered.is_valid() {
                            selection_model.select_q_model_index_q_flags_selection_flag(
                                &model_index_filtered,
                                QFlags::from(SelectionFlag::Select)
                            );
                        }
                    }

                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                // This actions if for undoing "add rows" actions. It deletes the stored rows.
                TableOperations::AddRows(mut rows) => {

                    // Sort them 0->9, so we can process them.
                    rows.sort_unstable();
                    self.undo_lock.store(true, Ordering::SeqCst);
                    let rows_split = delete_rows(&self.get_mut_ptr_table_model(), &rows);
                    history_opposite.push(TableOperations::RemoveRows(rows_split));
                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                // NOTE: the rows list must ALWAYS be in 1->9 order. Otherwise this breaks.
                TableOperations::RemoveRows(mut rows) => {
                    self.undo_lock.store(true, Ordering::SeqCst);
                    self.save_lock.store(true, Ordering::SeqCst);

                    // Make sure the order of these ones is always correct (9->0).
                    rows.sort_by(|x, y| x.0.cmp(&y.0));

                    // First, we re-create the rows and re-insert them.
                    for (index, row_pack) in &rows {
                        for (offset, row) in row_pack.iter().enumerate() {
                            let qlist = QListOfQStandardItem::new();
                            row.iter().for_each(|x| qlist.append_q_standard_item(&ptr_from_atomic(x).as_mut_raw_ptr()));
                            model.insert_row_int_q_list_of_q_standard_item(*index + offset as i32, &qlist);
                        }
                    }

                    // Then, create the redo action for this one.
                    let mut rows_to_add = rows.iter()
                        .map(|(index, row_pack)|
                            row_pack.iter().enumerate()
                                .map(|(x, _)| *index + x as i32)
                                .collect::<Vec<i32>>()
                        )
                        .flatten()
                        .collect::<Vec<i32>>();

                    rows_to_add.reverse();
                    history_opposite.push(TableOperations::AddRows(rows_to_add));

                    // Select all the re-inserted rows that are in the filter. We need to block signals here because the bigger this gets,
                    // the slower it gets. And it gets very slow on high amounts of lines.
                    let selection_model = self.table_view_primary.selection_model();
                    selection_model.clear();
                    for (index, row_pack) in &rows {
                        let initial_model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(*index, 0));
                        let final_model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(*index + row_pack.len() as i32 - 1, 0));
                        if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                            let selection = QItemSelection::new_2a(&initial_model_index_filtered, &final_model_index_filtered);
                            selection_model.select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Select | SelectionFlag::Rows);
                        }
                    }

                    // Trick to tell the model to update everything.
                    self.save_lock.store(false, Ordering::SeqCst);
                    model.item_2a(0, 0).set_data_2a(&QVariant::new(), 16);
                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                // This action is special and we have to manually trigger a save for it.
                TableOperations::ImportTSV(table_data) => {

                    let old_data = self.get_copy_of_table();
                    history_opposite.push(TableOperations::ImportTSV(old_data));

                    let row_count = self.table_model.row_count_0a();
                    self.undo_lock.store(true, Ordering::SeqCst);
                    self.table_model.remove_rows_2a(0, row_count);
                    for row in &table_data {
                        let row = ptr_from_atomic(row);
                        self.table_model.append_row_q_list_of_q_standard_item(row.as_ref().unwrap())
                    }
                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                TableOperations::Carolina(mut operations) => {
                    is_carolina = true;
                    repeat_x_times = operations.len();
                    operations.reverse();
                    history_source.append(&mut operations);
                }
            }

            // We have to manually update these from the context menu due to RwLock deadlocks.
            if undo {
                self.context_menu_undo.set_enabled(!history_source.is_empty());
                self.context_menu_redo.set_enabled(!history_opposite.is_empty());
            }
            else {
                self.context_menu_redo.set_enabled(!history_source.is_empty());
                self.context_menu_undo.set_enabled(!history_opposite.is_empty());
            }
        }

        // If we have repetitions, it means we got a carolina. Repeat all the times we need until all editions are undone.
        // Then, remove all the actions done and put them into a carolina.
        if repeat_x_times > 0 {
            self.undo_redo(undo, repeat_x_times - 1);
            if is_carolina {
                let mut history_opposite = if undo {
                    self.history_redo.write().unwrap()
                } else {
                    self.history_undo.write().unwrap()
                };
                let len = history_opposite.len();
                let mut edits = history_opposite.drain((len - repeat_x_times)..).collect::<Vec<TableOperations>>();
                edits.reverse();
                history_opposite.push(TableOperations::Carolina(edits));
            }
        }

        self.start_delayed_updates_timer();
        self.update_line_counter();
    }

    /// This function returns the provided indexes's data as a LUA table.
    unsafe fn get_indexes_as_lua_table(&self, indexes: &[Ref<QModelIndex>], has_keys: bool) -> String {
        let mut table_data: Vec<(Option<String>, Vec<String>)> = vec![];
        let mut last_row = None;
        for index in indexes {
            if index.column() != -1 {
                let current_row = index.row();
                match last_row {
                    Some(row) => {

                        // If it's the same row as before, take the row from the table data and append it.
                        if current_row == row {
                            let entry = table_data.last_mut().unwrap();
                            let data = self.get_escaped_lua_string_from_index(*index);
                            if entry.0.is_none() && self.get_ref_table_definition().get_fields_processed()[index.column() as usize].get_is_key() {
                                entry.0 = Some(self.escape_string_from_index(*index));
                            }
                            entry.1.push(data);
                        }

                        // If it's not the same row as before, we create it as a new row.
                        else {
                            let mut entry = (None, vec![]);
                            let data = self.get_escaped_lua_string_from_index(*index);
                            entry.1.push(data.to_string());
                            if entry.0.is_none() && self.get_ref_table_definition().get_fields_processed()[index.column() as usize].get_is_key() {
                                entry.0 = Some(self.escape_string_from_index(*index));
                            }
                            table_data.push(entry);
                        }
                    }
                    None => {
                        let mut entry = (None, vec![]);
                        let data = self.get_escaped_lua_string_from_index(*index);
                        entry.1.push(data.to_string());
                        if entry.0.is_none() && self.get_ref_table_definition().get_fields_processed()[index.column() as usize].get_is_key() {
                            entry.0 = Some(self.escape_string_from_index(*index));
                        }
                        table_data.push(entry);
                    }
                }

                last_row = Some(current_row);
            }
        }

        // Create the string of the table.
        let mut lua_table = String::new();

        if !table_data.is_empty() {
            if has_keys {
                lua_table.push_str("TABLE = {\n");
            }

            for (index, row) in table_data.iter().enumerate() {

                // Start the row.
                if let Some(key) = &row.0 {
                    lua_table.push_str(&format!("\t[{}] = {{", key));
                }
                else {
                    lua_table.push('{');
                }

                // For each cell in the row, push it to the LUA Table.
                for cell in row.1.iter() {
                    lua_table.push_str(cell);
                }

                // Take out the last comma.
                lua_table.pop();

                // Close the row.
                if index == row.1.len() - 1 {
                    lua_table.push_str(" }\n");
                }
                else {
                    lua_table.push_str(" },\n");
                }
            }

            if has_keys {
                lua_table.push('}');
            }
        }

        lua_table
    }

    /// This function turns the data from the provided indexes into LUA compatible strings.
    unsafe fn get_escaped_lua_string_from_index(&self, index: Ref<QModelIndex>) -> String {
        format!(" [\"{}\"] = {},", self.get_ref_table_definition().get_fields_processed()[index.column() as usize].get_name(), self.escape_string_from_index(index))
    }

    /// This function escapes the value inside an index.
    unsafe fn escape_string_from_index(&self, index: Ref<QModelIndex>) -> String {
        let item = self.table_model.item_from_index(index);
        let definition = &self.get_ref_table_definition().clone();
        match definition.get_fields_processed()[index.column() as usize].get_ref_field_type() {
            FieldType::Boolean => if let CheckState::Checked = item.check_state() { "true".to_owned() } else { "false".to_owned() },

            // Floats need to be tweaked to fix trailing zeroes and precision issues, like turning 0.5000004 into 0.5.
            FieldType::F32 => {
                let data_str = format!("{}", item.data_1a(2).to_float_0a());

                // If we have more than 3 decimals, we limit it to three, then do magic to remove trailing zeroes.
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 { format!("{}", format!("{:.4}", item.data_1a(2).to_float_0a()).parse::<f32>().unwrap()) }
                    else { data_str }
                }
                else { data_str }
            },

            // Floats need to be tweaked to fix trailing zeroes and precision issues, like turning 0.5000004 into 0.5.
            FieldType::F64 => {
                let data_str = format!("{}", item.data_1a(2).to_float_0a());

                // If we have more than 3 decimals, we limit it to three, then do magic to remove trailing zeroes.
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 { format!("{}", format!("{:.4}", item.data_1a(2).to_double_0a()).parse::<f64>().unwrap()) }
                    else { data_str }
                }
                else { data_str }
            },
            FieldType::I16 |
            FieldType::I32 |
            FieldType::I64 => format!("{}", item.data_1a(2).to_long_long_0a()),
            FieldType::ColourRGB => format!("\"{}\"", item.text().to_std_string().escape_default().to_string()),

            // All these are Strings, so they need to escape certain chars and include commas in Lua.
            FieldType::StringU8 |
            FieldType::StringU16 |
            FieldType::OptionalStringU8 |
            FieldType::OptionalStringU16 => format!("\"{}\"", item.text().to_std_string().escape_default().to_string()),
            FieldType::SequenceU16(_) => "\"SequenceU16\"".to_owned(),
            FieldType::SequenceU32(_) => "\"SequenceU32\"".to_owned(),
        }
    }

    /// This function is used to append new rows to a table.
    ///
    /// If clone = true, the appended rows are copies of the selected ones.
    pub unsafe fn append_rows(&self, clone: bool) {

        // Get the indexes ready for battle.
        let selection = self.table_view_primary.selection_model().selection();
        let indexes = self.table_filter.map_selection_to_source(&selection).indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_by_model(&mut indexes_sorted);
        dedup_indexes_per_row(&mut indexes_sorted);
        let mut row_numbers = vec![];

        let rows = if clone {
            let mut rows = vec![];
            for index in indexes_sorted.iter() {
                row_numbers.push(index.row());

                let columns = self.table_model.column_count_0a();
                let qlist = QListOfQStandardItem::new();
                for column in 0..columns {
                    let original_item = self.table_model.item_2a(index.row(), column);
                    let item = (*original_item).clone();
                    item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                    qlist.append_q_standard_item(&item.as_mut_raw_ptr());
                }

                rows.push(qlist);
            }
            rows
        } else {
            let row = get_new_row(&self.get_ref_table_definition(), self.get_ref_table_name().as_deref());
            for index in 0..row.count_0a() {
                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
            }
            vec![row]
        };

        let selection_model = self.table_view_primary.selection_model();
        selection_model.clear();
        for row in &rows {
            self.table_model.append_row_q_list_of_q_standard_item(row.as_ref());

            // Select the row and scroll to it.
            let model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(self.table_filter.row_count_0a() - 1, 0));
            if model_index_filtered.is_valid() {
                selection_model.select_q_model_index_q_flags_selection_flag(
                    &model_index_filtered,
                    SelectionFlag::Select | SelectionFlag::Rows
                );

                self.table_view_primary.scroll_to_2a(
                    model_index_filtered.as_ref(),
                    ScrollHint::EnsureVisible
                );
            }
        }

        // Update the undo stuff. Cloned rows are the amount of rows - the amount of cloned rows.
        let total_rows = self.table_model.row_count_0a();
        let range = (total_rows - rows.len() as i32..total_rows).collect::<Vec<i32>>();
        self.history_undo.write().unwrap().push(TableOperations::AddRows(range));
        self.history_redo.write().unwrap().clear();
        self.start_delayed_updates_timer();
        self.update_line_counter();
        update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
        //unsafe { undo_redo_enabler.as_mut().unwrap().trigger(); }
    }

    /// This function is used to insert new rows into a table.
    ///
    /// If clone = true, the appended rows are copies of the selected ones.
    pub unsafe fn insert_rows(&self, clone: bool) {

        // Get the indexes ready for battle.
        let selection = self.table_view_primary.selection_model().selection();
        let indexes = self.table_filter.map_selection_to_source(&selection).indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_by_model(&mut indexes_sorted);
        dedup_indexes_per_row(&mut indexes_sorted);
        let mut row_numbers = vec![];

        // If nothing is selected, we just append one new row at the end. This only happens when adding empty rows, so...
        if indexes_sorted.is_empty() {
            let row = get_new_row(&self.get_ref_table_definition(), self.get_ref_table_name().as_deref());
            for index in 0..row.count_0a() {
                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
            }
            self.table_model.append_row_q_list_of_q_standard_item(&row);
            row_numbers.push(self.table_model.row_count_0a() - 1);
        }

        let selection_model = self.table_view_primary.selection_model();
        selection_model.clear();

        for index in indexes_sorted.iter().rev() {
            row_numbers.push(index.row() + (indexes_sorted.len() - row_numbers.len() - 1) as i32);

            // If we want to clone, we copy the currently selected row. If not, we just create a new one.
            let row = if clone {
                let columns = self.table_model.column_count_0a();
                let qlist = QListOfQStandardItem::new();
                for column in 0..columns {
                    let original_item = self.table_model.item_2a(index.row(), column);
                    let item = (*original_item).clone();
                    item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                    qlist.append_q_standard_item(&item.as_mut_raw_ptr());
                }
                qlist
            } else {
                let row = get_new_row(&self.get_ref_table_definition(), self.get_ref_table_name().as_deref());
                for index in 0..row.count_0a() {
                    row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                }
                row
            };
            self.table_model.insert_row_int_q_list_of_q_standard_item(index.row(), &row);

            // Select the row.
            let model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(index.row(), 0));
            if model_index_filtered.is_valid() {
                selection_model.select_q_model_index_q_flags_selection_flag(
                    &model_index_filtered,
                    SelectionFlag::Select | SelectionFlag::Rows
                );
            }
        }

        // The undo mode needs this reversed.
        self.history_undo.write().unwrap().push(TableOperations::AddRows(row_numbers));
        self.history_redo.write().unwrap().clear();
        self.start_delayed_updates_timer();
        self.update_line_counter();
        update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
    }

    /// This function returns a copy of the entire model.
    pub unsafe fn get_copy_of_table(&self) -> Vec<AtomicPtr<QListOfQStandardItem>> {
        let mut old_data = vec![];
        for row in 0..self.table_model.row_count_0a() {
            let qlist = QListOfQStandardItem::new();
            for column in 0..self.table_model.column_count_0a() {
                let item = self.table_model.item_2a(row, column);
                qlist.append_q_standard_item(&(*item).clone().as_mut_raw_ptr());
            }
            old_data.push(atomic_from_ptr(qlist.into_ptr()));
        }
        old_data
    }

    /// This function creates the entire "Rewrite selection" dialog for tables. It returns the rewriting sequence, or None.
    pub unsafe fn create_rewrite_selection_dialog(&self) -> Option<(bool, String)> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view_primary);
        dialog.set_window_title(&qtr("rewrite_selection_title"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string(&qtr("rewrite_selection_instructions_title"));
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string(&qtr("rewrite_selection_instructions"));
        instructions_grid.add_widget_5a(& instructions_label, 0, 0, 1, 1);

        let is_math_op = QCheckBox::from_q_string(&qtr("rewrite_selection_is_math"));
        let rewrite_sequence_line_edit = QLineEdit::new();
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rewrite_selection_placeholder"));
        let accept_button = QPushButton::from_q_string(&qtr("rewrite_selection_accept"));

        main_grid.add_widget_5a(instructions_frame.into_ptr(), 0, 0, 1, 2);
        main_grid.add_widget_5a(&is_math_op, 1, 0, 1, 2);
        main_grid.add_widget_5a(&rewrite_sequence_line_edit, 2, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 2, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = rewrite_sequence_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some((is_math_op.is_checked(), new_text)) }
        } else { None }
    }

    /// This function creates the entire "Generate Ids" dialog for tables. It returns the starting id, or None.
    pub unsafe fn create_generate_ids_dialog(&self, initial_value: i32) -> Option<i32> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view_primary);
        dialog.set_window_title(&qtr("generate_ids_title"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string_q_widget(&qtr("generate_ids_instructions_title"), &dialog);
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string_q_widget(&qtr("generate_ids_instructions"), &instructions_frame);
        instructions_grid.add_widget_5a(& instructions_label, 0, 0, 1, 1);

        let starting_id_spin_box = QSpinBox::new_1a(&dialog);
        starting_id_spin_box.set_minimum(i32::MIN);
        starting_id_spin_box.set_maximum(i32::MAX);
        starting_id_spin_box.set_value(initial_value);
        let accept_button = QPushButton::from_q_string(&qtr("generate_ids_accept"));

        main_grid.add_widget_5a(&instructions_frame, 0, 0, 1, 1);
        main_grid.add_widget_5a(&starting_id_spin_box, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 2, 0, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            Some(starting_id_spin_box.value())
        } else { None }
    }

    /// This function takes care of the "Delete filtered-out rows" feature for tables.
    pub unsafe fn delete_filtered_out_rows(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let visible_columns = (0..self.table_model.column_count_0a()).filter(|index| !self.table_view_primary.is_column_hidden(*index)).collect::<Vec<i32>>();

        // If it's empty, it means everything is hidden, so we delete everything.
        let mut rows_to_delete = vec![];
        for row in 0..self.table_model.row_count_0a() {
            if visible_columns.is_empty() {
                rows_to_delete.push(row);
            } else if !self.table_filter.map_from_source(&self.table_model.index_2a(row, visible_columns[0])).is_valid() {
                rows_to_delete.push(row);
            }
        }

        // Dedup the list and reverse it.
        rows_to_delete.sort_unstable();
        rows_to_delete.dedup();
        rows_to_delete.reverse();

        self.set_data_on_cells(&[], 0, &rows_to_delete, app_ui, pack_file_contents_ui);
    }

    /// This function takes care of the "Smart Delete" feature for tables.
    pub unsafe fn smart_delete(&self, delete_all_rows: bool, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the selected indexes, the split them in two groups: one with full rows selected and another with single cells selected.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes_sorted = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        if delete_all_rows {
            let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

            // Dedup the list and reverse it.
            rows_to_delete.sort_unstable();
            rows_to_delete.dedup();
            rows_to_delete.reverse();

            self.set_data_on_cells(&[], 0, &rows_to_delete, app_ui, pack_file_contents_ui);
        } else {

            let mut cells: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
            for model_index in &indexes_sorted {
                if model_index.is_valid() {
                    let row = model_index.row();
                    let column = model_index.column();

                    // Check if we have any cell in that row and add/insert the new one.
                    match cells.get_mut(&row) {
                        Some(row) => row.push(column),
                        None => { cells.insert(row, vec![column]); },
                    }
                }
            }

            let visible_column_count = (0..self.table_model.column_count_0a()).filter(|index| !self.table_view_primary.is_column_hidden(*index)).count();
            let full_rows = cells.iter()
                .filter(|(_, y)| y.len() >= visible_column_count)
                .map(|(x, _)| *x)
                .collect::<Vec<i32>>();

            let individual_cells = cells.iter()
                .filter(|(_, y)| y.len() < visible_column_count)
                .map(|(x, y)| (*x, y.to_vec()))
                .collect::<Vec<(i32, Vec<i32>)>>();

            let default_str = "".to_owned();
            let default_f32 = "0.0".to_owned();
            let default_f64 = "0.0".to_owned();
            let default_i32 = "0".to_owned();
            let default_bool = "false".to_owned();
            let default_colour_rgb = "000000".to_owned();

            let mut real_cells = vec![];
            let mut values = vec![];
            for (row, columns) in &individual_cells {
                for column in columns {
                    let index = self.table_model.index_2a(*row, *column);
                    if index.is_valid() {
                        match self.get_ref_table_definition().get_fields_processed()[*column as usize].get_ref_field_type() {
                            FieldType::Boolean => values.push(&*default_bool),
                            FieldType::F32 => values.push(&*default_f32),
                            FieldType::F64 => values.push(&*default_f64),
                            FieldType::I16 |
                            FieldType::I32 |
                            FieldType::I64 => values.push(&*default_i32),
                            FieldType::ColourRGB => values.push(&*default_colour_rgb),
                            FieldType::StringU8 |
                            FieldType::StringU16 |
                            FieldType::OptionalStringU8 |
                            FieldType::OptionalStringU16 => values.push(&*default_str),
                            FieldType::SequenceU16(_) |
                            FieldType::SequenceU32(_) => continue,
                        }
                        real_cells.push(index);
                    }
                }
            }


            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), &*values[index]));
            }
            realer_cells.reverse();

            self.set_data_on_cells(&realer_cells, 0, &full_rows, app_ui, pack_file_contents_ui);
        }
    }

    /// Function used to have a generic way to set data on cells/remove rows and setup their undo steps.
    pub unsafe fn set_data_on_cells(
        &self,
        real_cells: &[(CppBox<QModelIndex>, &str)],
        added_rows: i32,
        rows_to_delete: &[i32],
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>
    ) {

        // Block the events so this doesn't take ages. Also, this means we do weird things here for performance.
        let blocker = QSignalBlocker::from_q_object(&self.table_model);
        let blocker_undo = QSignalBlocker::from_q_object(&self.undo_model);
        let mut changed_cells = 0;

        for (real_cell, text) in real_cells {
            if real_cell.is_valid() {

                // Depending on the column, we try to encode the data in one format or another.
                let current_value = self.table_model.data_1a(real_cell).to_string().to_std_string();
                let definition = self.get_ref_table_definition();
                match definition.get_fields_processed()[real_cell.column() as usize].get_ref_field_type() {

                    FieldType::Boolean => {
                        let current_value = self.table_model.item_from_index(real_cell).check_state();
                        let new_value = if text.to_lowercase() == "true" || *text == "1" { CheckState::Checked } else { CheckState::Unchecked };
                        if current_value != new_value {
                            self.table_model.item_from_index(real_cell).set_check_state(new_value);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    // These are a bit special because we have to ignore any difference after the third decimal.
                    FieldType::F32 => {
                        let current_value = format!("{:.4}", self.table_model.data_2a(real_cell, 2).to_float_0a());
                        if let Ok(new_value) = text.parse::<f32>() {
                            let new_value_txt = format!("{:.4}", new_value);
                            if current_value != new_value_txt {
                                self.table_model.set_data_3a(real_cell, &QVariant::from_float(new_value), 2);
                                changed_cells += 1;
                                self.process_edition(self.table_model.item_from_index(real_cell));
                            }
                        }
                    },

                    // Same thing as with F32.
                    FieldType::F64 => {
                        let current_value = format!("{:.4}", self.table_model.data_2a(real_cell, 2).to_double_0a());
                        if let Ok(new_value) = text.parse::<f64>() {
                            let new_value_txt = format!("{:.4}", new_value);
                            if current_value != new_value_txt {
                                self.table_model.set_data_3a(real_cell, &QVariant::from_double(new_value), 2);
                                changed_cells += 1;
                                self.process_edition(self.table_model.item_from_index(real_cell));
                            }
                        }
                    },

                    FieldType::I16 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i16>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i16 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_int(new_value as i32), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    FieldType::I32 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i32>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i32 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_int(new_value), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    FieldType::I64 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i64>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i64 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_i64(new_value), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    _ => {
                        if current_value != *text {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_q_string(&QString::from_std_str(text)), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    }
                }
            }
        }

        blocker.unblock();
        blocker_undo.unblock();

        let deleted_rows = if !rows_to_delete.is_empty() {
            super::utils::delete_rows(&self.get_mut_ptr_table_model(), rows_to_delete)
        } else { vec![] };

        // Fix the undo history to have all the previous changed merged into one. Or that's what I wanted.
        // Sadly, the world doesn't work like that. As we can edit, delete AND add rows, we have to use a combined undo operation.
        // I'll call it... Carolina.
        if changed_cells > 0 || added_rows > 0 || !deleted_rows.is_empty() {
            update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
            {
                let mut history_undo = self.history_undo.write().unwrap();
                let mut history_redo = self.history_redo.write().unwrap();

                let len = history_undo.len();
                let mut carolina = vec![];

                if !deleted_rows.is_empty() {
                    carolina.push(TableOperations::RemoveRows(deleted_rows));
                }

                if changed_cells > 0 {

                    let mut edits_data = vec![];
                    let mut edits = history_undo.drain((len - changed_cells)..);
                    for edit in &mut edits {
                        if let TableOperations::Editing(mut edit) = edit {
                            edits_data.append(&mut edit);
                        }
                    }
                    carolina.push(TableOperations::Editing(edits_data));
                }

                if added_rows > 0 {
                    let mut rows = vec![];
                    ((self.table_model.row_count_0a() - added_rows)..self.table_model.row_count_0a()).rev().for_each(|x| rows.push(x));
                    carolina.push(TableOperations::AddRows(rows));
                }

                history_undo.push(TableOperations::Carolina(carolina));
                history_redo.clear();
            }

            self.post_process_edition(app_ui, pack_file_contents_ui);
        }

        // Trick to properly update the view.
        self.table_view_primary.viewport().repaint();

        self.start_delayed_updates_timer();
    }

    /// Process a single cell edition. Launch this after every edition if the signals are blocked.
    pub unsafe fn process_edition(&self, item: Ptr<QStandardItem>) {
        let item_old = self.undo_model.item_2a(item.row(), item.column());

        // Only trigger this if the values are actually different. Checkable cells are tricky. Nested cells an go to hell.
        if (item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state()) ||
            item_old.data_1a(ITEM_IS_SEQUENCE).to_bool() && 0 != item_old.data_1a(ITEM_SEQUENCE_DATA).to_string().compare_q_string(&item.data_1a(ITEM_SEQUENCE_DATA).to_string()) {
            let edition = vec![((item.row(), item.column()), atomic_from_ptr((&*item_old).clone()))];
            let operation = TableOperations::Editing(edition);
            self.history_undo.write().unwrap().push(operation);

            item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_MODIFIED);
        }
    }

    /// Triggers stuff that should be done once after a bunch of editions.
    pub unsafe fn post_process_edition(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        update_undo_model(&self.get_mut_ptr_table_model(), &self.get_mut_ptr_undo_model());
        self.context_menu_update();
        if let Some(ref packed_file_path) = self.packed_file_path {
            TableSearch::update_search(self);
            if let DataSource::PackFile = *self.data_source.read().unwrap() {
                set_modified(true, &packed_file_path.read().unwrap(), app_ui, pack_file_contents_ui);
            }
        }

        if SETTINGS.read().unwrap().settings_bool["table_resize_on_edit"] {
            self.table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }

        // Re-sort and re-filter the table, as it's not automatically done.
        self.table_filter.set_dynamic_sort_filter(false);
        self.table_filter.set_dynamic_sort_filter(true);

        self.table_filter.invalidate();
        self.filter_table();

        self.table_view_primary.viewport().repaint();
    }

    /// This function triggers a cascade edition through the entire program of the selected cells.
    pub unsafe fn cascade_edition(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // This feature has some... interesting lockups when running alongside a diagnostics check. So, while this runs,
        // we have to avoid triggering the diagnostics check.
        self.timer_delayed_updates.stop();

        // We only want to do this for tables we can identify.
        let edited_table_name = if let Some(table_name) = self.get_ref_table_name() { table_name.to_lowercase() } else { return };

        // Get the selected indexes.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        // Ask the dialog to get the data needed for the replacing.
        if let Some(editions) = self.cascade_edition_dialog(&indexes) {
            app_ui.main_window.set_enabled(false);

            // Trigger editions in our own table.
            let real_cells = editions.iter()
                .map(|(_, new_value, row, column)| (self.table_model.index_2a(*row, *column), &**new_value))
                .collect::<Vec<(CppBox<QModelIndex>, &str)>>();
            self.set_data_on_cells(&real_cells, 0, &[], app_ui, pack_file_contents_ui);

            // Stop the timer again.
            self.timer_delayed_updates.stop();

            // Initialize our cascade editions.
            let mut cascade_editions = CascadeEdition::default();
            cascade_editions.set_edited_table_name(edited_table_name);
            cascade_editions.set_edited_table_definition(self.get_ref_table_definition().clone());

            // Get the tables/rows that need to be edited.
            let schema = SCHEMA.read().unwrap();
            let edited_fields_processed = cascade_editions.get_ref_edited_table_definition().get_fields_processed();
            editions.into_iter().for_each(|(old_data, new_data, _, column)| {
                match cascade_editions.get_ref_mut_data_changes().get_mut(&(column as u32)) {
                    Some(data_changed) => data_changed.push((old_data, new_data)),
                    None => {
                        let data_changed = vec![(old_data, new_data)];
                        cascade_editions.get_ref_mut_data_changes().insert(column as u32, data_changed);

                        if let Some(field) = edited_fields_processed.get(column as usize) {
                            if let Some(results) = Table::get_tables_and_columns_referencing_our_own(
                                &schema,
                                cascade_editions.get_ref_edited_table_name(),
                                field.get_name(),
                                cascade_editions.get_ref_edited_table_definition()
                            ){
                                cascade_editions.get_ref_mut_referenced_tables().insert(column as u32, results);
                            }
                        }
                    },
                }
            });

            // Now that we know what to edit, save all views of referencing files, so we only have to deal with them in the background.
            UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).for_each(|packed_file_view| {

                // Check for tables.
                if let Some(folder) = packed_file_view.get_path().get(0) {
                    if folder.to_lowercase() == "db" {
                        if let Some(table_name) = packed_file_view.get_path().get(1) {
                            if cascade_editions.get_ref_referenced_tables().values().any(|x| x.0.contains_key(table_name)) {
                                let _ = packed_file_view.save(app_ui, pack_file_contents_ui);
                            }
                        }
                    }
                }

                // Check for locs.
                else if cascade_editions.get_ref_referenced_tables().values().any(|x| x.1) {
                    if let Some(file) = packed_file_view.get_path().last() {
                        if !file.is_empty() && file.to_lowercase().ends_with(".loc") {
                            let _ = packed_file_view.save(app_ui, pack_file_contents_ui);
                        }
                    }
                }
            });

            // Then ask the backend to do the heavy work.
            let receiver = CENTRAL_COMMAND.send_background(Command::CascadeEdition(cascade_editions));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::VecVecStringVecPackedFileInfo(edited_paths, packed_files_info) => {

                    // If it worked, get the list of edited PackedFiles and update the TreeView to reflect the change.
                    let edited_path_types = edited_paths.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Modify(edited_path_types.to_vec()), DataSource::PackFile);
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(edited_path_types), DataSource::PackFile);
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);

                    // Before finishing, reload all edited views.
                    let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                    edited_paths.iter().for_each(|path| {
                        if let Some(packed_file_view) = open_packedfiles.iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                            if packed_file_view.reload(path, pack_file_contents_ui).is_err() {
                                let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                            }
                        }
                    });

                    app_ui.main_window.set_enabled(true);

                    // Now it's safe to trigger the timer.
                    self.start_delayed_updates_timer();
                }
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        // If we didn't do anything, but we cut a timer, continue it.
        else if self.timer_delayed_updates.remaining_time() != -1 {
            self.start_delayed_updates_timer();
        }
    }

    /// This function creates the "Cascade Edition" dialog.
    ///
    /// It returns the data for the editions, or `None` if the dialog is canceled or closed.
    pub unsafe fn cascade_edition_dialog(&self, indexes: &[CppBox<QModelIndex>]) -> Option<Vec<(String, String, i32, i32)>> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view_primary);
        dialog.set_window_title(&qtr("cascade_edition_dialog"));
        dialog.set_modal(true);
        dialog.resize_2a(800, 50);

        let main_grid = create_grid_layout(dialog.static_upcast());
        let mut edits = vec![];

        for (row, index) in indexes.iter().enumerate() {
            let old_name_line_edit = QLineEdit::from_q_string_q_widget(&self.table_model.data_1a(index).to_string(), &dialog);
            let new_name_line_edit = QLineEdit::from_q_string_q_widget(&self.table_model.data_1a(index).to_string(), &dialog);

            old_name_line_edit.set_enabled(false);
            main_grid.add_widget_5a(&old_name_line_edit, row as i32, 0, 1, 1);
            main_grid.add_widget_5a(&new_name_line_edit, row as i32, 1, 1, 1);

            edits.push((old_name_line_edit, new_name_line_edit, index));
        }

        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));
        main_grid.add_widget_5a(&accept_button, 99999, 0, 1, 2);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {

            // Filter out unchanged/empty cells.
            let real_edits = edits.into_iter()
                .filter(|(old, new, _)| !new.text().is_empty() && old.text().to_std_string() != new.text().to_std_string())
                .map(|(old, new, index)| (old.text().to_std_string(), new.text().to_std_string(), index.row(), index.column()))
                .collect::<Vec<(String, String, i32, i32)>>();
            if real_edits.is_empty() { None } else { Some(real_edits) }
        } else { None }
    }

    /// This function creates the "Patch Column" dialog and submits a patch of accepted.
    pub unsafe fn patch_column(&self) -> Result<()> {

        // We only want to do this for tables we can identify.
        let edited_table_name = if let Some(table_name) = self.get_ref_table_name() { table_name.to_lowercase() } else { return Err(ErrorKind::DBTableIsNotADBTable.into()) };

        // Get the selected indexes.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, &self.get_mut_ptr_table_view_primary());
        let indexes = get_real_indexes(&indexes_sorted, &self.get_mut_ptr_table_view_filter());

        // Only works with a column selected.
        let columns: Vec<i32> = indexes.iter().map(|x| x.column()).sorted().dedup().collect();
        if indexes.iter().map(|x| x.column()).sorted().dedup().count() != 1 {
            return Err(ErrorKind::Generic.into())
        }

        let column_index = columns[0];
        let field = self.get_ref_table_definition().get_fields_processed().get(column_index as usize).cloned().ok_or(Error::from(ErrorKind::Generic))?;

        // Create and configure the dialog.
        let view = if cfg!(debug_assertions) { PATCH_COLUMN_VIEW_DEBUG } else { PATCH_COLUMN_VIEW_RELEASE };
        let template_path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), view);
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, &self.table_view_primary);

        let schema_patch_instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "schema_patch_instructions_label")?;
        let default_value_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "default_value_label")?;
        let not_empty_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "not_empty_label")?;
        let explanation_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "explanation_label")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let default_value_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "default_value_line_edit")?;
        let not_empty_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "not_empty_checkbox")?;
        let explanation_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "explanation_text_edit")?;

        let dialog = main_widget.static_downcast::<QDialog>();
        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_close());
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Setup translations.
        dialog.set_window_title(&qtr("new_schema_patch_dialog"));
        schema_patch_instructions_label.set_text(&qtr("schema_patch_instructions"));
        default_value_label.set_text(&qtr("default_value"));
        not_empty_label.set_text(&qtr("not_empty"));
        explanation_label.set_text(&qtr("explanation"));
        explanation_text_edit.set_placeholder_text(&qtr("explanation_placeholder_text"));

        // Setup data.
        if let Some(default_value) = field.get_default_value(self.get_ref_table_name().as_deref()) {
            default_value_line_edit.set_text(&QString::from_std_str(&default_value));
        }
        not_empty_checkbox.set_checked(field.get_cannot_be_empty(self.get_ref_table_name().as_deref()));
        explanation_text_edit.set_text(&QString::from_std_str(field.get_schema_patch_explanation(self.get_ref_table_name().as_deref())));

        // Launch.
        if dialog.exec() == 1 {
            let mut column_data = HashMap::new();

            column_data.insert("default_value".to_owned(), default_value_line_edit.text().to_std_string());
            column_data.insert("not_empty".to_owned(), not_empty_checkbox.is_checked().to_string());
            column_data.insert("explanation".to_owned(), explanation_text_edit.to_plain_text().to_std_string());

            let mut table_data = HashMap::new();
            table_data.insert(field.get_name().to_owned(), column_data);

            let mut schema_patch = SchemaPatch::default();
            schema_patch.get_ref_mut_tables().insert(edited_table_name.to_owned(), table_data);

            let receiver = CENTRAL_COMMAND.send_background(Command::UploadSchemaPatch(schema_patch));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Success => show_dialog(&self.table_view_primary, tr("schema_patch_submitted_correctly"), true),
                Response::Error(error) => return Err(error),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
        }

        Ok(())
    }

    /// This function tries to open the source of a reference/loc key, if exits in the PackFile.
    ///
    /// If the source it's not found, it does nothing.
    /// If the source is a read-only dependency, it does nothing yet.
    pub unsafe fn go_to_definition(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) -> Option<String> {

        let mut error_message = String::new();
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        if indexes.count_0a() > 0 {
            let ref_info = match *self.packed_file_type {

                // For DB, we just get the reference data, the first selected cell's data, and use that to search the source file.
                PackedFileType::DB => {
                    let index = self.table_filter.map_to_source(self.table_view_primary.selection_model().selection().indexes().at(0));
                    if let Some(field) = self.get_ref_table_definition().get_fields_processed().get(index.column() as usize) {
                        if let Some((ref_table, ref_column)) = field.get_is_reference() {
                            Some((ref_table.to_owned(), ref_column.to_owned(), index.data_0a().to_string().to_std_string()))
                        } else { None }
                    } else { None }
                }

                // For Locs, we use the column 0 of the row with the selected item.
                PackedFileType::Loc => {
                    let index_row = self.table_filter.map_to_source(self.table_view_primary.selection_model().selection().indexes().at(0)).row();
                    let key = self.table_model.index_2a(index_row, 0).data_0a().to_string().to_std_string();
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetSourceDataFromLocKey(key));
                    let response = CentralCommand::recv_try(&receiver);
                    match response {
                        Response::OptionStringStringString(response) => response,
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
                _ => None,
            };

            if let Some((ref_table, ref_column, ref_data)) = ref_info {

                // Save the tables that may be the source before searching, to ensure their data is updated.
                UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).for_each(|packed_file_view| {
                    if let Some(folder) = packed_file_view.get_path().get(0) {
                        if folder.to_lowercase() == "db" {
                            if let Some(table_name) = packed_file_view.get_path().get(1) {
                                if &ref_table == table_name {
                                    let _ = packed_file_view.save(app_ui, pack_file_contents_ui);
                                }
                            }
                        }
                    }
                });

                // Then ask the backend to do the heavy work.
                let receiver = CENTRAL_COMMAND.send_background(Command::GoToDefinition(ref_table, ref_column, ref_data));
                let response = CentralCommand::recv_try(&receiver);
                match response {

                    // We receive a path/column/row, so we know what to open/select.
                    Response::DataSourceVecStringUsizeUsize(data_source, path, column, row) => {
                        match data_source {
                            DataSource::PackFile => {
                                let tree_index = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path, data_source);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view.static_upcast::<QObject>());
                                        pack_file_contents_ui.packfile_contents_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                                        pack_file_contents_ui.packfile_contents_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ParentFiles |
                            DataSource::AssKitFiles |
                            DataSource::GameFiles => {
                                let tree_index = dependencies_ui.dependencies_tree_view.expand_treeview_to_item(&path, DataSource::GameFiles);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view.static_upcast::<QObject>());
                                        dependencies_ui.dependencies_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                                        dependencies_ui.dependencies_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ExternalFile => {},
                        }

                        // Set the current file as non-preview, so it doesn't close when opening the source one.
                        if let Some(packed_file_path) = self.get_packed_file_path() {
                            if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_path && x.get_data_source() == self.get_data_source()) {
                                packed_file_view.set_is_preview(false);
                            }
                        }

                        // Open the table and select the cell.
                        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, Some(path.to_vec()), true, false, data_source);
                        if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == path && x.get_data_source() == data_source) {
                            if let ViewType::Internal(View::Table(view)) = packed_file_view.get_view() {
                                let table_view = view.get_ref_table();
                                let table_view = table_view.get_mut_ptr_table_view_primary();
                                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                                let table_selection_model = table_view.selection_model();

                                let table_model_index = table_model.index_2a(row as i32, column as i32);
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                                }
                            }
                        }
                    }

                    Response::Error(error) => error_message = error.to_terminal(),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            } else {
                error_message = tr("source_data_for_field_not_found");
            }
        }

        if error_message.is_empty() { None }
        else { Some(error_message) }
    }

    /// This function tries to open the loc data related with the currently selected row.
    ///
    /// If the loc data it's not found, it does nothing.
    /// If the loc data is a read-only dependency, it does nothing yet.
    pub unsafe fn go_to_loc(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        loc_column_name: &str
    ) -> Option<String> {

        // This is only for DB Tables, and we need to have something selected.
        let indexes = self.table_view_primary.selection_model().selection().indexes();
        let mut error_message = String::new();
        if indexes.count_0a() > 0 {
            if let PackedFileType::DB = *self.packed_file_type {

                // Save the currently open locs, to ensure the backend has the most up-to-date data.
                UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).for_each(|packed_file_view| {
                    if let PackedFileType::Loc = packed_file_view.get_packed_file_type() {
                        let _ = packed_file_view.save(app_ui, pack_file_contents_ui);
                    }
                });

                // Get the name of the table and the key of the selected row to know what loc key to search.
                let table_name = if let Some(ref table_name) = self.table_name {
                    table_name.to_owned().drain(..table_name.len() - 7).collect::<String>()
                } else { return Some(tr("loc_key_not_found")) };

                let key = if let Some(column) = self.get_ref_table_definition().get_fields_processed().iter().position(|x| (x.get_name() == "key" || x.get_name() == "id") && x.get_is_key()) {
                    let row = self.table_filter.map_to_source(self.table_view_primary.selection_model().selection().indexes().at(0)).row();
                    self.table_model.index_2a(row, column as i32).data_0a().to_string().to_std_string()
                } else { return Some(tr("loc_key_not_found")) };

                let loc_key = format!("{}_{}_{}", table_name, loc_column_name, key);

                // Then ask the backend to do the heavy work.
                let receiver = CENTRAL_COMMAND.send_background(Command::GoToLoc(loc_key));
                let response = CentralCommand::recv_try(&receiver);
                match response {

                    // We receive a path/column/row, so we know what to open/select.
                    Response::DataSourceVecStringUsizeUsize(data_source, path, column, row) => {
                        match data_source {
                            DataSource::PackFile => {
                                let tree_index = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path, data_source);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view.static_upcast::<QObject>());
                                        pack_file_contents_ui.packfile_contents_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                                        pack_file_contents_ui.packfile_contents_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ParentFiles |
                            DataSource::AssKitFiles |
                            DataSource::GameFiles => {
                                let tree_index = dependencies_ui.dependencies_tree_view.expand_treeview_to_item(&path, DataSource::GameFiles);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view.static_upcast::<QObject>());
                                        dependencies_ui.dependencies_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                                        dependencies_ui.dependencies_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ExternalFile => {},
                        }

                        // Set the current file as non-preview, so it doesn't close when opening the source one.
                        if let Some(packed_file_path) = self.get_packed_file_path() {
                            if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *packed_file_path && x.get_data_source() == self.get_data_source()) {
                                packed_file_view.set_is_preview(false);
                            }
                        }

                        // Open the table and select the cell.
                        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, Some(path.to_vec()), true, false, data_source);
                        if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == path && x.get_data_source() == data_source) {
                            if let ViewType::Internal(View::Table(view)) = packed_file_view.get_view() {
                                let table_view = view.get_ref_table();
                                let table_view = table_view.get_mut_ptr_table_view_primary();
                                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                                let table_selection_model = table_view.selection_model();

                                let table_model_index = table_model.index_2a(row as i32, column as i32);
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                                }
                            }
                        }
                    }

                    Response::Error(error) => error_message = error.to_terminal(),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }

        if error_message.is_empty() { None }
        else { Some(error_message) }
    }

    /// This function clears the markings for added/modified cells.
    pub unsafe fn clear_markings(&self) {
        let table_view = self.get_mut_ptr_table_view_primary();
        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
        let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

        for row in 0..table_model.row_count_0a() {
            for column in 0..table_model.column_count_0a() {
                let item = table_model.item_2a(row, column);

                if item.data_1a(ITEM_IS_ADDED).to_bool() {
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_ADDED);
                }

                if item.data_1a(ITEM_IS_MODIFIED).to_bool() {
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                }
            }
        }
        blocker.unblock();
        table_view.viewport().repaint();
    }
}
