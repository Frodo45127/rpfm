//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! TableView submodule to provide Search & Replace functionality.

use qt_widgets::QComboBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CheckState;
use qt_core::MatchFlag;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::Ptr;

use anyhow::Result;
use getset::Getters;

use std::sync::{Arc, RwLock};

use rpfm_lib::schema::{Field, FieldType};
use rpfm_lib::utils::parse_str_as_bool;

use rpfm_ui_common::SETTINGS;
use rpfm_ui_common::utils::*;

use crate::packedfile_views::DataSource;
use crate::views::table::utils::clean_column_names;

use self::slots::SearchViewSlots;
use super::{TableOperations, TableView, update_undo_model};

mod connections;
mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/table_search_widget.ui";
const VIEW_RELEASE: &str = "ui/table_search_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the stuff needed to perform a table search. There is one per table, integrated in the view.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SearchView {
    main_widget: QBox<QWidget>,
    search_line_edit: QPtr<QLineEdit>,
    replace_line_edit: QPtr<QLineEdit>,
    search_button: QPtr<QToolButton>,
    prev_match_button: QPtr<QToolButton>,
    next_match_button: QPtr<QToolButton>,
    replace_button: QPtr<QToolButton>,
    replace_all_button: QPtr<QToolButton>,
    close_button: QPtr<QToolButton>,
    matches_label: QPtr<QLabel>,
    column_combobox: QPtr<QComboBox>,
    case_sensitive_button: QPtr<QToolButton>,

    last_search_data: Arc<RwLock<SearchData>>,
}

pub struct SearchData {
    pattern: Ptr<QString>,
    replace: Ptr<QString>,
    regex: bool,
    case_sensitive: bool,
    column: Option<i32>,

    /// This one contains the QModelIndex of the model and the QModelIndex of the filter, if exists.
    matches: Vec<(Ptr<QModelIndex>, Option<Ptr<QModelIndex>>)>,
    current_item: Option<u64>,
}

/// This enum defines the operation to be done when updating something related to the TableSearch.
pub enum TableSearchUpdate {
    Update,
    Search,
    PrevMatch,
    NextMatch,
}

//----------------------------------------------------------------//
// Implementations of `TableSearch`.
//----------------------------------------------------------------//

impl SearchView {

    pub unsafe fn new(view: &Arc<TableView>) -> Result<()> {
        let parent = view.filter_base_widget_ptr();
        let parent_grid: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let search_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "search_label")?;
        let replace_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "replace_label")?;
        let search_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "search_line_edit")?;
        let replace_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "replace_line_edit")?;

        let search_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "search_button")?;
        let next_match_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "next_match_button")?;
        let prev_match_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "prev_match_button")?;
        let replace_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_button")?;
        let replace_all_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_all_button")?;
        let case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "case_sensitive_button")?;
        let close_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "close_button")?;
        let matches_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "matches_label")?;

        // TODO: Move this to translations.
        let column_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "column_combobox")?;
        let search_column_list = QStandardItemModel::new_1a(&column_combobox);
        column_combobox.set_model(&search_column_list);
        column_combobox.add_item_q_string(&QString::from_std_str("* (All Columns)"));

        search_label.set_text(&QString::from_std_str("Search Pattern:"));
        replace_label.set_text(&QString::from_std_str("Replace Pattern:"));
        search_line_edit.set_placeholder_text(&QString::from_std_str("Type here what you want to search."));
        replace_line_edit.set_placeholder_text(&QString::from_std_str("If you want to replace the searched text with something, type the replacement here."));

        search_button.set_tool_tip(&QString::from_std_str("Search"));
        prev_match_button.set_tool_tip(&QString::from_std_str("Prev. Match"));
        next_match_button.set_tool_tip(&QString::from_std_str("Next Match"));
        replace_button.set_tool_tip(&QString::from_std_str("Replace Current"));
        replace_all_button.set_tool_tip(&QString::from_std_str("Replace All"));
        case_sensitive_button.set_tool_tip(&QString::from_std_str("Case Sensitive"));
        close_button.set_tool_tip(&QString::from_std_str("Close"));

        let fields = view.table_definition.read().unwrap().fields_processed_sorted(SETTINGS.read().unwrap().bool("tables_use_old_column_order"));
        for column in &fields {
            column_combobox.add_item_q_string(&QString::from_std_str(clean_column_names(column.name())));
        }

        parent_grid.add_widget_5a(&main_widget, 3, 0, 1, 2);
        parent_grid.set_column_stretch(0, 10);
        main_widget.hide();

        let search = Arc::new(Self {
            main_widget,
            search_line_edit,
            replace_line_edit,
            search_button,
            replace_button,
            replace_all_button,
            close_button,
            prev_match_button,
            next_match_button,
            matches_label,
            column_combobox,
            case_sensitive_button,
            last_search_data: Arc::new(RwLock::new(SearchData::default())),
        });

        let slots = SearchViewSlots::new(&search, view);
        connections::set_connections(&search, &slots);

        *view.search_view.write().unwrap() = Some(search);
        Ok(())
    }

    /// This function reloads the search panel, in case the definition of the table changes.
    pub unsafe fn reload(&self, parent: &TableView) {
        self.column_combobox.clear();
        self.column_combobox.add_item_q_string(&QString::from_std_str("* (All Columns)"));

        let fields = parent.table_definition.read().unwrap().fields_processed_sorted(SETTINGS.read().unwrap().bool("tables_use_old_column_order"));
        for column in &fields {
            self.column_combobox.add_item_q_string(&QString::from_std_str(clean_column_names(column.name())));
        }
    }

    /// This function takes care of updating the UI to reflect changes in the table search.
    pub unsafe fn update_search_ui(&self, parent: &TableView, update_type: TableSearchUpdate) {
        let table_search = &mut self.last_search_data.write().unwrap();
        let matches_in_filter = table_search.get_matches_in_filter();
        let matches_in_model = table_search.get_matches_in_model();
        match update_type {
            TableSearchUpdate::Search => {
                if table_search.pattern.is_empty() {
                    self.matches_label.set_text(&QString::new());
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // If no matches have been found, report it.
                else if table_search.matches.is_empty() {
                    table_search.current_item = None;
                    self.matches_label.set_text(&QString::from_std_str("No matches found."));
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // Otherwise, if no matches have been found in the current filter, but they have been in the model...
                else if matches_in_filter.is_empty() {
                    table_search.current_item = None;
                    self.matches_label.set_text(&QString::from_std_str(format!("{} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // Otherwise, matches have been found both, in the model and in the filter.
                else {
                    table_search.current_item = Some(0);
                    self.matches_label.set_text(&QString::from_std_str(format!("1 of {} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    self.prev_match_button.set_enabled(false);
                    self.replace_button.set_enabled(true);
                    self.replace_all_button.set_enabled(true);

                    if matches_in_filter.len() > 1 {
                        self.next_match_button.set_enabled(true);
                    }
                    else {
                        self.next_match_button.set_enabled(false);
                    }

                    parent.table_view.selection_model().select_q_model_index_q_flags_selection_flag(
                        matches_in_filter[0].as_ref().unwrap(),
                        QFlags::from(SelectionFlag::ClearAndSelect)
                    );
                }

            }
            TableSearchUpdate::PrevMatch => {
                let matches_in_model = table_search.get_matches_in_model();
                let matches_in_filter = table_search.get_matches_in_filter();
                if let Some(ref mut pos) = table_search.current_item {

                    // If we are in an invalid result, return. If it's the first one, disable the button and return.
                    if *pos > 0 {
                        *pos -= 1;
                        if *pos == 0 { self.prev_match_button.set_enabled(false); }
                        else { self.prev_match_button.set_enabled(true); }
                        if *pos as usize >= matches_in_filter.len() - 1 { self.next_match_button.set_enabled(false); }
                        else { self.next_match_button.set_enabled(true); }

                        parent.table_view.selection_model().select_q_model_index_q_flags_selection_flag(
                            matches_in_filter[*pos as usize].as_ref().unwrap(),
                            QFlags::from(SelectionFlag::ClearAndSelect)
                        );
                        self.matches_label.set_text(&QString::from_std_str(format!("{} of {} in current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches_in_model.len())));
                    }
                }
            }
            TableSearchUpdate::NextMatch => {
                let matches_in_model = table_search.get_matches_in_model();
                let matches_in_filter = table_search.get_matches_in_filter();
                if let Some(ref mut pos) = table_search.current_item {

                    // If we are in an invalid result, return. If it's the last one, disable the button and return.
                    if *pos as usize >= matches_in_filter.len() - 1 {
                        self.next_match_button.set_enabled(false);
                    }
                    else {
                        *pos += 1;
                        if *pos == 0 { self.prev_match_button.set_enabled(false); }
                        else { self.prev_match_button.set_enabled(true); }
                        if *pos as usize >= matches_in_filter.len() - 1 { self.next_match_button.set_enabled(false); }
                        else { self.next_match_button.set_enabled(true); }

                        parent.table_view.selection_model().select_q_model_index_q_flags_selection_flag(
                            matches_in_filter[*pos as usize].as_ref().unwrap(),
                            QFlags::from(SelectionFlag::ClearAndSelect)
                        );
                        self.matches_label.set_text(&QString::from_std_str(format!("{} of {} in current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches_in_model.len())));
                    }
                }
            }
            TableSearchUpdate::Update => {
                if table_search.pattern.is_empty() {
                    self.matches_label.set_text(&QString::new());
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // If no matches have been found, report it.
                else if table_search.matches.is_empty() {
                    table_search.current_item = None;
                    self.matches_label.set_text(&QString::from_std_str("No matches found."));
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // Otherwise, if no matches have been found in the current filter, but they have been in the model...
                else if matches_in_filter.is_empty() {
                    table_search.current_item = None;
                    self.matches_label.set_text(&QString::from_std_str(format!("{} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    self.prev_match_button.set_enabled(false);
                    self.next_match_button.set_enabled(false);
                    self.replace_button.set_enabled(false);
                    self.replace_all_button.set_enabled(false);
                }

                // Otherwise, matches have been found both, in the model and in the filter. Which means we have to recalculate
                // our position, and then behave more or less like a normal search.
                else {
                    table_search.current_item = match table_search.current_item {
                        Some(pos) => if (pos as usize) < matches_in_filter.len() { Some(pos) } else { Some(0) }
                        None => Some(0)
                    };

                    self.matches_label.set_text(&QString::from_std_str(format!("{} of {} in current filter ({} in total)", table_search.current_item.unwrap() + 1, matches_in_filter.len(), matches_in_model.len())));

                    if table_search.current_item.unwrap() == 0 {
                        self.prev_match_button.set_enabled(false);
                    }
                    else {
                        self.prev_match_button.set_enabled(true);
                    }

                    if matches_in_filter.len() > 1 && (table_search.current_item.unwrap() as usize) < matches_in_filter.len() - 1 {
                        self.next_match_button.set_enabled(true);
                    }
                    else {
                        self.next_match_button.set_enabled(false);
                    }

                    self.replace_button.set_enabled(true);
                    self.replace_all_button.set_enabled(true);
                }
            }
        }

        if parent.get_data_source() != DataSource::PackFile {
            self.replace_button.set_enabled(false);
            self.replace_all_button.set_enabled(false);
        }
    }

    /// This function takes care of updating the search data whenever a change that can alter the results happens.
    pub unsafe fn update_search(&self, parent: &TableView) {
        {
            let fields_processed = parent.table_definition().fields_processed();
            let table_search = &mut self.last_search_data.write().unwrap();
            table_search.matches.clear();

            let mut flags = if table_search.regex {
                QFlags::from(MatchFlag::MatchRegExp)
            } else {
                QFlags::from(MatchFlag::MatchContains)
            };

            if table_search.case_sensitive {
                flags = flags | QFlags::from(MatchFlag::MatchCaseSensitive);
            }

            let columns_to_search = match table_search.column {
                Some(column) => vec![column],
                None => (0..fields_processed.len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model.as_ptr(), parent.table_filter.as_ptr(), &fields_processed, flags, *column);
            }
        }

        self.update_search_ui(parent, TableSearchUpdate::Update);
    }

    /// This function takes care of searching the patter we provided in the TableView.
    pub unsafe fn search(&self, parent: &TableView) {
        {
            let fields_processed = parent.table_definition().fields_processed();
            let table_search = &mut self.last_search_data.write().unwrap();
            table_search.matches.clear();
            table_search.current_item = None;
            table_search.pattern = self.search_line_edit.text().into_ptr();
            //table_search.regex = self.search_search_line_edit.is_checked();
            table_search.case_sensitive = self.case_sensitive_button.is_checked();
            table_search.column = {
                let column = self.column_combobox.current_text().to_std_string().replace(' ', "_").to_lowercase();
                if column == "*_(all_columns)" { None }
                else { Some(fields_processed.iter().position(|x| x.name() == column).unwrap() as i32) }
            };

            let mut flags = if table_search.regex {
                QFlags::from(MatchFlag::MatchRegExp)
            } else {
                QFlags::from(MatchFlag::MatchContains)
            };

            if table_search.case_sensitive {
                flags = flags | QFlags::from(MatchFlag::MatchCaseSensitive);
            }

            let columns_to_search = match table_search.column {
                Some(column) => vec![column],
                None => (0..fields_processed.len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model.as_ptr(), parent.table_filter.as_ptr(), &fields_processed, flags, *column);
            }
        }

        self.update_search_ui(parent, TableSearchUpdate::Search);
    }

    /// This function takes care of moving the selection to the previous match on the matches list.
    pub unsafe fn prev_match(&self, parent: &TableView) {
        self.update_search_ui(parent, TableSearchUpdate::PrevMatch);
    }

    /// This function takes care of moving the selection to the next match on the matches list.
    pub unsafe fn next_match(&self, parent: &TableView) {
        self.update_search_ui(parent, TableSearchUpdate::NextMatch);
    }

    /// This function takes care of replacing the current match with the provided replacing text.
    pub unsafe fn replace_current(&self, parent: &TableView) {

        // NOTE: WE CANNOT HAVE THE SEARCH DATA LOCK UNTIL AFTER WE DO THE REPLACE. That's why there are a lot of read here.
        let text_source = self.last_search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {
            let fields_processed = parent.table_definition().fields_processed();

            // Get the replace data here, as we probably don't have it updated.
            self.last_search_data.write().unwrap().replace = self.replace_line_edit.text().into_ptr();
            let text_replace = self.last_search_data.read().unwrap().replace.to_std_string();
            if text_source == text_replace { return }

            // And if we got a valid position.
            let item;
            let replaced_text;
            if let Some(ref position) = self.last_search_data.read().unwrap().current_item {

                // Here is save to lock, as the lock will be drop before doing the replace.
                let table_search = &mut self.last_search_data.read().unwrap();

                // Get the list of all valid ModelIndex for the current filter and the current position.
                let matches_in_model_and_filter = table_search.get_visible_matches_in_model();
                let model_index = matches_in_model_and_filter[*position as usize];

                // If the position is still valid (not required, but just in case)...
                if model_index.is_valid() {
                    item = parent.table_model.item_from_index(model_index.as_ref().unwrap());

                    if fields_processed[model_index.column() as usize].field_type() == &FieldType::Boolean {
                        replaced_text = text_replace;
                    }
                    else {
                        let text = item.text().to_std_string();
                        replaced_text = text.replace(&text_source, &text_replace);
                    }

                    // We need to do an extra check to ensure the new text can be in the field.
                    match fields_processed[model_index.column() as usize].field_type() {
                        FieldType::Boolean => if parse_str_as_bool(&replaced_text).is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid Boolean.", false) }
                        FieldType::F32 => if replaced_text.parse::<f32>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid F32.", false) }
                        FieldType::I16 => if replaced_text.parse::<i16>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I16.", false) }
                        FieldType::I32 => if replaced_text.parse::<i32>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I32.", false) }
                        FieldType::I64 => if replaced_text.parse::<i64>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I64.", false) }
                        _ =>  {}
                    }
                } else { return }
            } else { return }

            // At this point, we trigger editions. Which mean, here ALL LOCKS SHOULD HAVE BEEN ALREADY DROP.
            match fields_processed[item.column() as usize].field_type() {
                FieldType::Boolean => item.set_check_state(if parse_str_as_bool(&replaced_text).unwrap() { CheckState::Checked } else { CheckState::Unchecked }),
                FieldType::F32 => item.set_data_2a(&QVariant::from_float(replaced_text.parse::<f32>().unwrap()), 2),
                FieldType::I16 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i16>().unwrap().into()), 2),
                FieldType::I32 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i32>().unwrap()), 2),
                FieldType::I64 => item.set_data_2a(&QVariant::from_i64(replaced_text.parse::<i64>().unwrap()), 2),
                _ => item.set_text(&QString::from_std_str(&replaced_text)),
            }

            // At this point, the edition has been done. We're free to lock again. If we still have matches, select the next match, if any, or the first one.
            let table_search = &mut self.last_search_data.read().unwrap();
            if let Some(pos) = table_search.current_item {
                let matches_in_filter = table_search.get_matches_in_filter();

                parent.table_view.selection_model().select_q_model_index_q_flags_selection_flag(
                    matches_in_filter[pos as usize].as_ref().unwrap(),
                    QFlags::from(SelectionFlag::ClearAndSelect)
                );
            }
        }
    }

    /// This function takes care of replacing all the instances of a match with the provided replacing text.
    pub unsafe fn replace_all(&self, parent: &TableView) {

        // NOTE: WE CANNOT HAVE THE SEARCH DATA LOCK UNTIL AFTER WE DO THE REPLACE. That's why there are a lot of read here.
        let text_source = self.last_search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {
            let fields_processed = parent.table_definition().fields_processed();

            // Get the replace data here, as we probably don't have it updated.
            self.last_search_data.write().unwrap().replace = self.replace_line_edit.text().into_ptr();
            let text_replace = self.last_search_data.read().unwrap().replace.to_std_string();
            if text_source == text_replace { return }

            let mut positions_and_texts: Vec<(Ptr<QModelIndex>, String)> = vec![];
            {
                // Here is save to lock, as the lock will be drop before doing the replace.
                let table_search = &mut self.last_search_data.read().unwrap();

                // Get the list of all valid ModelIndex for the current filter and the current position.
                let matches_in_model_and_filter = table_search.get_visible_matches_in_model();
                for model_index in &matches_in_model_and_filter {

                    // If the position is still valid (not required, but just in case)...
                    if model_index.is_valid() {
                        let item = parent.table_model.item_from_index(model_index.as_ref().unwrap());
                        let original_text = match fields_processed[model_index.column() as usize].field_type() {
                            FieldType::Boolean => item.data_0a().to_bool().to_string(),
                            FieldType::F32 => item.data_0a().to_float_0a().to_string(),
                            FieldType::I16 => item.data_0a().to_int_0a().to_string(),
                            FieldType::I32 => item.data_0a().to_int_0a().to_string(),
                            FieldType::I64 => item.data_0a().to_long_long_0a().to_string(),
                            _ => item.text().to_std_string(),
                        };

                        let replaced_text = if fields_processed[model_index.column() as usize].field_type() == &FieldType::Boolean {
                            text_replace.to_owned()
                        }
                        else {
                            let text = item.text().to_std_string();
                            text.replace(&text_source, &text_replace)
                        };

                        // If no replacement has been done, skip it.
                        if original_text == replaced_text {
                            continue;
                        }

                        // We need to do an extra check to ensure the new text can be in the field.
                        match fields_processed[model_index.column() as usize].field_type() {
                            FieldType::Boolean => if parse_str_as_bool(&replaced_text).is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid Boolean.", false) }
                            FieldType::F32 => if replaced_text.parse::<f32>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid F32.", false) }
                            FieldType::I16 => if replaced_text.parse::<i16>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I16.", false) }
                            FieldType::I32 => if replaced_text.parse::<i32>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I32.", false) }
                            FieldType::I64 => if replaced_text.parse::<i64>().is_err() { return show_dialog(&parent.table_view, "Error replacing data of a cell, because the data is not a valid I64.", false) }
                            _ =>  {}
                        }

                        positions_and_texts.push((*model_index, replaced_text));
                    } else { return }
                }
            }

            // At this point, we trigger editions. Which mean, here ALL LOCKS SHOULD HAVE BEEN ALREADY DROP.
            for (model_index, replaced_text) in &positions_and_texts {
                let item = parent.table_model.item_from_index(model_index.as_ref().unwrap());
                match fields_processed[item.column() as usize].field_type() {
                    FieldType::Boolean => item.set_check_state(if parse_str_as_bool(replaced_text).unwrap() { CheckState::Checked } else { CheckState::Unchecked }),
                    FieldType::F32 => item.set_data_2a(&QVariant::from_float(replaced_text.parse::<f32>().unwrap()), 2),
                    FieldType::I16 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i16>().unwrap().into()), 2),
                    FieldType::I32 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i32>().unwrap()), 2),
                    FieldType::I64 => item.set_data_2a(&QVariant::from_i64(replaced_text.parse::<i64>().unwrap()), 2),
                    _ => item.set_text(&QString::from_std_str(replaced_text)),
                }
            }

            // At this point, the edition has been done. We're free to lock again. As this is a full replace,
            // we have to fix the undo history to compensate the mass-editing and turn it into a single action.
            if !positions_and_texts.is_empty() {
                {
                    let mut history_undo = parent.history_undo.write().unwrap();
                    let mut history_redo = parent.history_redo.write().unwrap();

                    let len = history_undo.len();
                    let mut edits_data = vec![];
                    {
                        let mut edits = history_undo.drain((len - positions_and_texts.len())..);
                        for edit in &mut edits {
                            if let TableOperations::Editing(mut edit) = edit {
                                edits_data.append(&mut edit);
                            }
                        }
                    }

                    history_undo.push(TableOperations::Editing(edits_data));
                    history_redo.clear();
                }
                update_undo_model(&parent.table_model_ptr(), &parent.undo_model_ptr());
            }
        }
    }
}

impl Default for SearchData {
    fn default() -> Self {
        Self {
            pattern: unsafe { QString::new().into_ptr() },
            replace: unsafe { QString::new().into_ptr() },
            regex: false,
            case_sensitive: false,
            column: None,
            matches: vec![],
            current_item: None,
        }
    }
}

impl SearchData {

    /// This function returns the list of matches present in the model.
    fn get_matches_in_model(&self) -> Vec<Ptr<QModelIndex>> {
        self.matches.iter().map(|x| x.0).collect()
    }

    /// This function returns the list of matches visible to the user with the current filter.
    fn get_matches_in_filter(&self) -> Vec<Ptr<QModelIndex>> {
        self.matches.iter().filter_map(|x| x.1).collect()
    }

    /// This function returns the list of matches present in the model that are visible to the user with the current filter.
    fn get_visible_matches_in_model(&self) -> Vec<Ptr<QModelIndex>> {
        self.matches.iter().filter(|x| x.1.is_some()).map(|x| x.0).collect()
    }

    /// This function takes care of searching data within a column, and adding the matches to the matches list.
    unsafe fn find_in_column(
        &mut self,
        model: Ptr<QStandardItemModel>,
        filter: Ptr<QSortFilterProxyModel>,
        fields_processed: &[Field],
        flags: QFlags<MatchFlag>,
        column: i32
    ) {

        // First, check the column type. Boolean columns need special logic, as they cannot be matched by string.
        let is_bool = fields_processed[column as usize].field_type() == &FieldType::Boolean;
        let matches_unprocessed = if is_bool {
            match parse_str_as_bool(&self.pattern.to_std_string()) {
                Ok(boolean) => {
                    let check_state = if boolean { CheckState::Checked } else { CheckState::Unchecked };
                    let items = QListOfQStandardItem::new();
                    for row in 0..model.row_count_0a() {
                        let item = model.item_2a(row, column);
                        if item.check_state() == check_state {
                            items.append_q_standard_item(&item.as_mut_raw_ptr());
                        }
                    }
                    items
                }

                // If this fails, ignore the entire column.
                Err(_) => return,
            }
        }
        else {
            model.find_items_3a(self.pattern.as_ref().unwrap(), flags, column)
        };

        for index in 0..matches_unprocessed.count_0a() {
            let model_index = matches_unprocessed.value_1a(index).index();
            let filter_model_index = filter.map_from_source(&model_index);
            self.matches.push((
                model_index.into_ptr(),
                if filter_model_index.is_valid() { Some(filter_model_index.into_ptr()) } else { None }
            ));
        }
    }

}
