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
Module with the slots for Table Views.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QBrush;
use qt_gui::QCursor;
use qt_gui::QGuiApplication;
use qt_gui::SlotOfQStandardItem;

use qt_core::GlobalColor;
use qt_core::QModelIndex;
use qt_core::QItemSelection;
use qt_core::QSignalBlocker;
use qt_core::{SlotOfBool, Slot, SlotOfQString, SlotOfQItemSelectionQItemSelection};

use cpp_core::Ref;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use rpfm_lib::schema::Definition;
use rpfm_lib::SETTINGS;

use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::table::PackedFileTableViewRaw;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::show_dialog;
use crate::UI_STATE;

use super::*;
use super::utils::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of an Table PackedFile.
pub struct PackedFileTableViewSlots {
    pub filter_line_edit: SlotOfQString<'static>,
    pub toggle_lookups: SlotOfBool<'static>,
    pub show_context_menu: SlotOfQPoint<'static>,
    pub context_menu_enabler: SlotOfQItemSelectionQItemSelection<'static>,
    pub item_changed: SlotOfQStandardItem<'static>,
    pub add_rows: Slot<'static>,
    pub insert_rows: Slot<'static>,
    pub delete_rows: Slot<'static>,
    pub copy: Slot<'static>,
    pub copy_as_lua_table: Slot<'static>,
    pub paste: Slot<'static>,
    pub invert_selection: Slot<'static>,
    pub save: Slot<'static>,
    pub undo: Slot<'static>,
    pub redo: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableViewSlots`.
impl PackedFileTableViewSlots {

    /// This function creates the entire slot pack for images.
    pub unsafe fn new(
        packed_file_view: &PackedFileTableViewRaw,
        global_search_ui: GlobalSearchUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        table_definition: &Definition,
        dependency_data: &BTreeMap<i32, Vec<(String, String)>>
    ) -> Self {

        // When we want to filter when changing the pattern to filter with...
        let filter_line_edit = SlotOfQString::new(clone!(
            mut packed_file_view => move |_| {
            packed_file_view.filter_table();
        }));

        // When we want to toggle the lookups on and off.
        let toggle_lookups = SlotOfBool::new(clone!(
            packed_file_view,
            table_definition,
            dependency_data => move |_| {
            packed_file_view.toggle_lookups(&table_definition, &dependency_data);
        }));

        // When we want to show the context menu.
        let show_context_menu = SlotOfQPoint::new(clone!(
            mut packed_file_view => move |_| {
            packed_file_view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(clone!(
            mut packed_file_view,
            mut table_definition => move |_,_| {
            packed_file_view.context_menu_update(&table_definition);
            }
        ));

        // When we want to respond to a change in one item in the model.
        let item_changed = SlotOfQStandardItem::new(clone!(
            mut packed_file_view,
            //packed_file_path,
            //dependency_data,
            mut table_definition => move |item| {

                // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                if !packed_file_view.undo_lock.load(Ordering::SeqCst) {

                    let mut edition = vec![];
                    let item_old = packed_file_view.undo_model.item_2a(item.row(), item.column());
                    edition.push(((item.row(), item.column()), atomic_from_mut_ptr((&*item_old).clone())));
                    let operation = TableOperations::Editing(edition);
                    packed_file_view.history_undo.write().unwrap().push(operation);
                    packed_file_view.history_redo.write().unwrap().clear();

                    {
                        // We block the saving for painting, so this doesn't get rettriggered again.
                        //let mut blocker = QSignalBlocker::from_q_object(packed_file_view.table_model);
                        let color = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkYellow } else { GlobalColor::Yellow };
                        //item.set_background(&QBrush::from_global_color(color));
                        //blocker.unblock();
                    }

                    // For pasting, only update the undo_model the last iteration of the paste.
                    if packed_file_view.save_lock.load(Ordering::SeqCst) {
                        update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
                    }

                    packed_file_view.context_menu_update(&table_definition);
                }


/*
                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] {
                    let column = unsafe { item.as_mut().unwrap().column() };
                    if table_definition.fields[column as usize].field_is_reference.is_some() {
                        Self::check_references(&dependency_data, column, item);
                    }
                }*/

                // If we are editing the Dependency Manager, check for PackFile errors too.
                //if let TableType::DependencyManager(_) = *table_type.borrow() { Self::check_dependency_packfile_errors(model); }
            }
        ));

        // When you want to append a row to the table...
        let add_rows = Slot::new(clone!(
            mut packed_file_view => move || {

                // Create the row and append it.
                let row = get_new_row(&packed_file_view.table_definition);
                packed_file_view.table_model.append_row_q_list_of_q_standard_item(row.as_ref());

                // Add the operation to the undo history.
                packed_file_view.history_undo.write().unwrap().push(TableOperations::AddRows(vec![packed_file_view.table_model.row_count_0a() - 1; 1]));
                packed_file_view.history_redo.write().unwrap().clear();
                update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let insert_rows = Slot::new(clone!(
            mut packed_file_view => move || {

                // Get the indexes ready for battle.
                let selection = packed_file_view.table_view_primary.selection_model().selection();
                let indexes = packed_file_view.table_filter.map_selection_to_source(&selection).indexes();
                let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                sort_indexes_by_model(&mut indexes_sorted);
                dedup_indexes_per_row(&mut indexes_sorted);
                let mut row_numbers = vec![];

                // If nothing is selected, we just append one new row at the end.
                if indexes_sorted.is_empty() {
                    let row = get_new_row(&packed_file_view.table_definition);
                    packed_file_view.table_model.append_row_q_list_of_q_standard_item(&row);
                    row_numbers.push(packed_file_view.table_model.row_count_0a() - 1);
                }

                for index in indexes_sorted.iter().rev() {
                    row_numbers.push(index.row());
                    let row = get_new_row(&packed_file_view.table_definition);
                    packed_file_view.table_model.insert_row_int_q_list_of_q_standard_item(index.row(), &row);
                }

                // The undo mode needs this reversed.
                row_numbers.reverse();
                packed_file_view.history_undo.write().unwrap().push(TableOperations::AddRows(row_numbers));
                packed_file_view.history_redo.write().unwrap().clear();
                update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = Slot::new(clone!(
            mut packed_file_view => move || {

                // Get all the selected rows.
                let selection = packed_file_view.table_view_primary.selection_model().selection();
                let indexes = packed_file_view.table_filter.map_selection_to_source(&selection).indexes();
                let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

                // Dedup the list and reverse it.
                rows_to_delete.sort();
                rows_to_delete.dedup();
                rows_to_delete.reverse();
                let rows_splitted = delete_rows(packed_file_view.table_model, &rows_to_delete);

                // If we deleted something, try to save the PackedFile to the main PackFile.
                if !rows_to_delete.is_empty() {
                    packed_file_view.history_undo.write().unwrap().push(TableOperations::RemoveRows(rows_splitted));
                    packed_file_view.history_redo.write().unwrap().clear();
                    update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
                }
            }
        ));

        // When you want to copy one or more cells.
        let copy = Slot::new(clone!(
            packed_file_view => move || {
            packed_file_view.copy_selection();
        }));

        // When you want to copy a table as a lua table.
        let copy_as_lua_table = Slot::new(clone!(
            packed_file_view => move || {
            packed_file_view.copy_selection_as_lua_table();
        }));

        // When you want to copy one or more cells.
        let paste = Slot::new(clone!(
            mut packed_file_view => move || {
            packed_file_view.paste();
        }));

        // When we want to invert the selection of the table.
        let invert_selection = Slot::new(clone!(
            mut packed_file_view => move || {
            let rows = packed_file_view.table_filter.row_count_0a();
            let columns = packed_file_view.table_filter.column_count_0a();
            if rows > 0 && columns > 0 {
                let mut selection_model = packed_file_view.table_view_primary.selection_model();
                let first_item = packed_file_view.table_filter.index_2a(0, 0);
                let last_item = packed_file_view.table_filter.index_2a(rows - 1, columns - 1);
                let selection = QItemSelection::new_2a(&first_item, &last_item);
                selection_model.select_q_item_selection_q_flags_selection_flag(&selection, QFlags::from(SelectionFlag::Toggle));
            }
        }));

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = Slot::new(clone!(
            packed_file_path,
            packed_file_view => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().get(&*packed_file_path.borrow()) {
                    if let Err(error) = packed_file.save(&packed_file_path.borrow(), global_search_ui, &mut pack_file_contents_ui) {
                        show_dialog(packed_file_view.table_view_primary, error, false);
                    }
                }
            }
        }));

        // When we want to undo the last action.
        let undo = Slot::new(clone!(
            mut table_definition,
            mut packed_file_view => move || {
                packed_file_view.undo_redo(true, 1);
                update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
                packed_file_view.context_menu_update(&table_definition);
            }
        ));

        // When we want to redo the last undone action.
        let redo = Slot::new(clone!(
            mut table_definition,
            mut packed_file_view => move || {
                packed_file_view.undo_redo(false, 1);
                update_undo_model(packed_file_view.table_model, packed_file_view.undo_model);
                packed_file_view.context_menu_update(&table_definition);
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            filter_line_edit,
            toggle_lookups,
            show_context_menu,
            context_menu_enabler,
            item_changed,
            add_rows,
            insert_rows,
            delete_rows,
            copy,
            copy_as_lua_table,
            paste,
            invert_selection,
            save,
            undo,
            redo,
        }
    }
}
