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
Module with all the code for managing the view for a subtable.

A subtable is a recursive, less featured version of a normal table, to be used as nested table.
!*/

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::QMenu;
use qt_widgets::QAction;
use qt_widgets::QTableView;
use qt_widgets::QDialog;
use qt_widgets::QPushButton;

use qt_gui::QBrush;
use qt_gui::QGuiApplication;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CheckState;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QSortFilterProxyModel;
use qt_core::QSignalBlocker;
use qt_core::QString;
use qt_core::q_item_selection_model::SelectionFlag;

use cpp_core::{MutPtr, Ref};

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use rpfm_error::ErrorKind;

use rpfm_lib::packedfile::table::Table;
use rpfm_lib::schema::Definition;
use rpfm_lib::SETTINGS;

use crate::ffi::*;
use crate::locale::qtr;
use crate::packedfile_views::table::utils::{build_columns, setup_item_delegates, sort_indexes_visually, get_real_indexes, get_item_from_decoded_data, check_references, get_new_row, get_table_from_view, sort_indexes_by_model};
use crate::pack_tree::get_color_added_modified;
use crate::pack_tree::get_color_added;
use crate::packedfile_views::table::utils::dedup_indexes_per_row;
use crate::utils::create_grid_layout;
use crate::utils::show_dialog;

use self::slots::SubTableViewSlots;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains pointers to all the widgets in a Table View.
#[derive(Clone)]
pub struct SubTableView {
    pub table_view: MutPtr<QTableView>,
    pub table_filter: MutPtr<QSortFilterProxyModel>,
    pub table_model: MutPtr<QStandardItemModel>,

    pub column_sort_state: Arc<RwLock<(i32, i8)>>,

    pub context_menu: MutPtr<QMenu>,
    pub context_menu_enabler: MutPtr<QAction>,
    pub context_menu_add_rows: MutPtr<QAction>,
    pub context_menu_delete_rows: MutPtr<QAction>,
    pub context_menu_copy: MutPtr<QAction>,

    pub table_definition: Arc<RwLock<Definition>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `SubTableView`.
impl SubTableView {

    /// This function shows the current table in a window, to be editable, separated from the parent table.
    pub unsafe fn show(
        parent: MutPtr<QTableView>,
        table_data: &Table,
    ) -> Option<String> {

        // Create and configure the dialog.
        let mut dialog = QDialog::new_1a(parent);
        dialog.set_window_title(&qtr("nested_table_title"));
        dialog.set_modal(true);
        dialog.resize_2a(600, 200);
        let mut main_grid = create_grid_layout(dialog.as_mut_ptr().static_upcast_mut());

        // Prepare the Table and its model.
        let mut filter_model = QSortFilterProxyModel::new_0a();
        let mut model = QStandardItemModel::new_0a();
        filter_model.set_source_model(&mut model);
        let (mut table_view_primary, table_view_frozen) = new_tableview_frozen_safe(&mut dialog);
        set_frozen_data_model_safe(&mut table_view_primary, &mut filter_model);

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
            table_view_primary.horizontal_header().set_stretch_last_section(true);
            table_view_frozen.horizontal_header().set_stretch_last_section(true);
        }

        // Setup tight mode if the setting is enabled.
        if SETTINGS.read().unwrap().settings_bool["tight_table_mode"] {
            table_view_primary.vertical_header().set_minimum_section_size(22);
            table_view_primary.vertical_header().set_maximum_section_size(22);
            table_view_primary.vertical_header().set_default_section_size(22);

            table_view_frozen.vertical_header().set_minimum_section_size(22);
            table_view_frozen.vertical_header().set_maximum_section_size(22);
            table_view_frozen.vertical_header().set_default_section_size(22);
        }

        // Create the Contextual Menu for the TableView.
        let context_menu_enabler = QAction::new();
        let mut context_menu = QMenu::new().into_ptr();
        let context_menu_add_rows = context_menu.add_action_q_string(&qtr("context_menu_add_rows"));
        let context_menu_delete_rows = context_menu.add_action_q_string(&qtr("context_menu_delete_rows"));
        let context_menu_copy = context_menu.add_action_q_string(&qtr("context_menu_copy"));

        let subtable_view = Self {
            table_view: table_view_primary,
            table_filter: filter_model.into_ptr(),
            table_model: model.into_ptr(),

            column_sort_state: Arc::new(RwLock::new((-1, 0))),

            context_menu,
            context_menu_enabler: context_menu_enabler.into_ptr(),
            context_menu_add_rows,
            context_menu_delete_rows,
            context_menu_copy,

            table_definition: Arc::new(RwLock::new(table_data.get_definition())),
        };

        let slots = SubTableViewSlots::new(&subtable_view);

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        load_data(
            table_view_primary,
            table_view_frozen,
            table_data.get_ref_definition(),
            &RwLock::new(BTreeMap::new()),
            &table_data
        );

        build_columns(
            table_view_primary,
            Some(table_view_frozen),
            table_data.get_ref_definition(),
            ""
        );

        connections::set_connections(&subtable_view, &slots);
        let mut accept_button = QPushButton::from_q_string(&qtr("nested_table_accept"));
        main_grid.add_widget_5a(table_view_primary, 0, 0, 1, 1);
        main_grid.add_widget_5a(&mut accept_button, 1, 0, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            if let Ok(table) = get_table_from_view(subtable_view.table_model, table_data.get_ref_definition()) {
                Some(serde_json::to_string(&table).unwrap())
            } else {
                show_dialog(table_view_primary, ErrorKind::Generic, false);
                None
            }
        } else { None }
    }

    /// This function returns a reference to the definition of this table.
    pub fn get_ref_table_definition(&self) -> RwLockReadGuard<Definition> {
        self.table_definition.read().unwrap()
    }

    /// This function updates the state of the actions in the context menu.
    pub unsafe fn context_menu_update(&mut self) {

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
        let indexes = self.table_filter.map_selection_to_source(&self.table_view.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            self.context_menu_copy.set_enabled(true);
            self.context_menu_delete_rows.set_enabled(true);
        }

        // Otherwise, disable them.
        else {
            self.context_menu_copy.set_enabled(false);
            self.context_menu_delete_rows.set_enabled(false);
        }
    }

    /// This function copies the selected cells into the clipboard as a TSV file, so you can paste them in other programs.
    pub unsafe fn copy_selection(&self) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes = self.table_view.selection_model().selection().indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_visually(&mut indexes_sorted, self.table_view);
        let indexes_sorted = get_real_indexes(&indexes_sorted, self.table_filter);

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
                else { copy.push_str(&QString::to_std_string(&item.text())); }

                // Add a \t to separate fields except if it's the last field.
                if cycle < (indexes_sorted.len() - 1) { copy.push('\t'); }
            }
        }

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(copy));
    }

    /// This function is used to append new rows to a table.
    ///
    /// If clone = true, the appended rows are copies of the selected ones.
    pub unsafe fn append_rows(&mut self, clone: bool) {

        // Get the indexes ready for battle.
        let selection = self.table_view.selection_model().selection();
        let indexes = self.table_filter.map_selection_to_source(&selection).indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_by_model(&mut indexes_sorted);
        dedup_indexes_per_row(&mut indexes_sorted);
        let mut row_numbers = vec![];

        let rows = if clone {
            let mut rows = vec![];
            let color = get_color_added_modified();
            for index in indexes_sorted.iter() {
                row_numbers.push(index.row());

                let columns = self.table_model.column_count_0a();
                let mut qlist = QListOfQStandardItem::new();
                for column in 0..columns {
                    let original_item = self.table_model.item_2a(index.row(), column);
                    let mut item = (*original_item).clone();
                    item.set_background(&QBrush::from_q_color(color.as_ref().unwrap()));
                    add_to_q_list_safe(qlist.as_mut_ptr(), item);
                }

                rows.push(qlist);
            }
            rows
        } else {
            let color = get_color_added();
            let mut row = get_new_row(&self.get_ref_table_definition());
            for index in 0..row.count() {
                row.index(index).as_mut().unwrap().set_background(&QBrush::from_q_color(color.as_ref().unwrap()));
            }
            vec![row]
        };

        let mut selection_model = self.table_view.selection_model();
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

                self.table_view.scroll_to_2a(
                    model_index_filtered.as_ref(),
                    ScrollHint::EnsureVisible
                );
            }
        }
    }
}

/// This function loads the data from a Table into our TableView.
pub unsafe fn load_data(
    table_view_primary: MutPtr<QTableView>,
    table_view_frozen: MutPtr<QTableView>,
    definition: &Definition,
    dependency_data: &RwLock<BTreeMap<i32, BTreeMap<String, String>>>,
    data: &Table
) {
    let table_filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
    let mut table_model: MutPtr<QStandardItemModel> = table_filter.source_model().static_downcast_mut();

    // First, we delete all the data from the `ListStore`. Just in case there is something there.
    // This wipes out header information, so remember to run "build_columns" after this.
    table_model.clear();

    if !data.get_ref_table_data().is_empty() {

        // Load the data, row by row.
        let mut blocker = QSignalBlocker::from_q_object(table_model.static_upcast_mut::<QObject>());
        for (index, entry) in data.get_ref_table_data().iter().enumerate() {
            let mut qlist = QListOfQStandardItem::new();
            for (index, field) in entry.iter().enumerate() {
                let mut item = get_item_from_decoded_data(field);

                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.read().unwrap().settings_bool["use_dependency_checker"] && definition.fields[index].is_reference.is_some() {
                    check_references(index as i32, item.as_mut_ptr(), &dependency_data.read().unwrap());
                }

                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
            if index == data.get_ref_table_data().len() - 1 {
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
        &dependency_data.read().unwrap(),
    )
}
