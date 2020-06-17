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
Module with the slots for AnimFragment Views.
!*/

use qt_widgets::SlotOfQPoint;
use qt_widgets::SlotOfIntSortOrder;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QStandardItem;
use qt_gui::QBrush;
use qt_gui::QCursor;
use qt_gui::SlotOfQStandardItem;

use qt_core::QModelIndex;
use qt_core::QItemSelection;
use qt_core::QSignalBlocker;
use qt_core::SortOrder;
use qt_core::{Slot, SlotOfQItemSelectionQItemSelection};
use qt_core::QFlags;

use cpp_core::Ref;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::get_color_modified;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::table::utils::*;
use crate::packedfile_views::utils::set_modified;
use crate::UI_STATE;
use crate::utils::*;

use super::PackedFileAnimFragmentViewRaw;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimFragment PackedFile.
pub struct PackedFileAnimFragmentViewSlots {
    pub sort_order_column_changed_1: SlotOfIntSortOrder<'static>,
    pub sort_order_column_changed_2: SlotOfIntSortOrder<'static>,
    pub show_context_menu_1: SlotOfQPoint<'static>,
    pub show_context_menu_2: SlotOfQPoint<'static>,
    pub context_menu_enabler_1: SlotOfQItemSelectionQItemSelection<'static>,
    pub context_menu_enabler_2: SlotOfQItemSelectionQItemSelection<'static>,
    pub item_changed_1: SlotOfQStandardItem<'static>,
    pub item_changed_2: SlotOfQStandardItem<'static>,
    pub add_rows: Slot<'static>,
    pub insert_rows: Slot<'static>,
    pub delete_rows: Slot<'static>,
    pub clone_and_append: Slot<'static>,
    pub clone_and_insert: Slot<'static>,
    pub copy: Slot<'static>,
    pub copy_as_lua_table: Slot<'static>,
    pub paste: Slot<'static>,
    pub invert_selection: Slot<'static>,
    pub reset_selection: Slot<'static>,
    pub rewrite_selection: Slot<'static>,
    pub save: Slot<'static>,
    pub undo: Slot<'static>,
    pub redo: Slot<'static>,
    pub smart_delete: Slot<'static>,
    pub resize_columns: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimFragmentViewSlots`.
impl PackedFileAnimFragmentViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: PackedFileAnimFragmentViewRaw,
        mut app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI
    )  -> Self {

        let sort_order_column_changed_1 = SlotOfIntSortOrder::new(clone!(
            view => move |column, _| {
                sort_column(view.table_1, column, view.column_sort_state_1.clone());
            }
        ));

        let sort_order_column_changed_2 = SlotOfIntSortOrder::new(clone!(
            view => move |column, _| {
                sort_column(view.table_2, column, view.column_sort_state_2.clone());
            }
        ));

        // When we want to show the context menu.
        let show_context_menu_1 = SlotOfQPoint::new(clone!(
            mut view => move |_| {
            view.context_menu_1.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to show the context menu.
        let show_context_menu_2 = SlotOfQPoint::new(clone!(
            mut view => move |_| {
            view.context_menu_2.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let context_menu_enabler_1 = SlotOfQItemSelectionQItemSelection::new(clone!(
            mut view => move |_,_| {
            view.context_menu_update_1();
        }));
        let context_menu_enabler_2 = SlotOfQItemSelectionQItemSelection::new(clone!(
            mut view => move |_,_| {
            view.context_menu_update_2();
        }));

        // When we want to respond to a change in one item in the model.
        let item_changed_1 = SlotOfQStandardItem::new(clone!(
            mut pack_file_contents_ui,
            mut view => move |item| {
                view.item_changed(item, &mut app_ui, &mut pack_file_contents_ui, true);
            }
        ));
        let item_changed_2 = SlotOfQStandardItem::new(clone!(
            mut pack_file_contents_ui,
            mut view => move |item| {
                view.item_changed(item, &mut app_ui, &mut pack_file_contents_ui, false);
            }
        ));

        // When you want to append a row to the table...
        let add_rows = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
                //view.append_rows(false);
                //set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let insert_rows = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
                //view.insert_rows(false);
                //set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
            }
        ));

        // When you want to delete one or more rows...
        let delete_rows = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
/*
                // Get all the selected rows.
                let selection = view.table_view_primary.selection_model().selection();
                let indexes = view.table_filter.map_selection_to_source(&selection).indexes();
                let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

                // Dedup the list and reverse it.
                rows_to_delete.sort();
                rows_to_delete.dedup();
                rows_to_delete.reverse();
                let rows_splitted = delete_rows(view.table_model, &rows_to_delete);

                // If we deleted something, try to save the PackedFile to the main PackFile.
                if !rows_to_delete.is_empty() {
                    view.history_undo.write().unwrap().push(TableOperations::RemoveRows(rows_splitted));
                    view.history_redo.write().unwrap().clear();
                    update_undo_model(view.table_model, view.undo_model);
                    set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
                }*/
            }
        ));

        // When you want to clone and insert one or more rows.
        let clone_and_append = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
            //view.append_rows(true);
            //set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
        }));

        // When you want to clone and append one or more rows.
        let clone_and_insert = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
            //view.insert_rows(true);
            //set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
        }));

        // When you want to copy one or more cells.
        let copy = Slot::new(clone!(
            view => move || {
            //view.copy_selection();
        }));

        // When you want to copy a table as a lua table.
        let copy_as_lua_table = Slot::new(clone!(
            view => move || {
            //view.copy_selection_as_lua_table();
        }));

        // When you want to copy one or more cells.
        let paste = Slot::new(clone!(
            mut view => move || {
            //view.paste();
        }));

        // When we want to invert the selection of the table.
        let invert_selection = Slot::new(clone!(
            mut view => move || {
                /*
            let rows = view.table_filter.row_count_0a();
            let columns = view.table_filter.column_count_0a();
            if rows > 0 && columns > 0 {
                let mut selection_model = view.table_view_primary.selection_model();
                let first_item = view.table_filter.index_2a(0, 0);
                let last_item = view.table_filter.index_2a(rows - 1, columns - 1);
                let selection = QItemSelection::new_2a(&first_item, &last_item);
                selection_model.select_q_item_selection_q_flags_selection_flag(&selection, QFlags::from(SelectionFlag::Toggle));
            }*/
        }));

        // When we want to reset the selected items of the table to their original value.
        let reset_selection = Slot::new(clone!(
            mut view => move || {
            //view.reset_selection();
        }));

        // When we want to rewrite the selected items using a formula.
        let rewrite_selection = Slot::new(clone!(
            mut view => move || {
            //view.rewrite_selection();
        }));

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let save = Slot::new(clone!(
            view => move || {/*
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *view.packed_file_path.read().unwrap()) {
                    if let Err(error) = packed_file.save(&mut app_ui, global_search_ui, &mut pack_file_contents_ui) {
                        show_dialog(view.table_view_primary, error, false);
                    }
                }
            }*/
        }));

        // When we want to undo the last action.
        let undo = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {/*
                view.undo_redo(true, 0);
                update_undo_model(view.table_model, view.undo_model);
                view.context_menu_update();
                if view.history_undo.read().unwrap().is_empty() {
                    set_modified(false, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
                }*/
            }
        ));

        // When we want to redo the last undone action.
        let redo = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {/*
                view.undo_redo(false, 0);
                update_undo_model(view.table_model, view.undo_model);
                view.context_menu_update();
                set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
            */}
        ));

        // When we want to resize the columns depending on their contents...
        let resize_columns = Slot::new(clone!(view => move || {
            /*
            view.table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
            if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
                view.table_view_primary.horizontal_header().set_stretch_last_section(false);
                view.table_view_primary.horizontal_header().set_stretch_last_section(true);
            }*/
        }));

        // When you want to use the "Smart Delete" feature...
        let smart_delete = Slot::new(clone!(
            mut pack_file_contents_ui,
            mut view => move || {
/*
                // Get the selected indexes, the split them in two groups: one with full rows selected and another with single cells selected.
                let indexes = view.table_view_primary.selection_model().selection().indexes();
                let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                sort_indexes_visually(&mut indexes_sorted, view.table_view_primary);
                let indexes_sorted = get_real_indexes(&indexes_sorted, view.table_filter);

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

                let full_rows = cells.iter()
                    .filter(|(_, y)| y.len() as i32 == view.table_model.column_count_0a())
                    .map(|(x, _)| *x)
                    .collect::<Vec<i32>>();

                let individual_cells = cells.iter()
                    .filter(|(_, y)| y.len() as i32 != view.table_model.column_count_0a())
                    .map(|(x, y)| (*x, y.to_vec()))
                    .collect::<Vec<(i32, Vec<i32>)>>();

                // First, we do the editions. This means:
                // - Checkboxes: unchecked.
                // - Numbers: 0.
                // - Strings: empty.
                let mut editions = 0;
                for (row, columns) in &individual_cells {
                    for column in columns {
                        let mut item = view.table_model.item_2a(*row, *column);
                        let current_value = item.text().to_std_string();
                        match view.get_ref_table_definition().fields[*column as usize].field_type {
                            FieldType::Boolean => {
                                let current_value = item.check_state();
                                if current_value != CheckState::Unchecked {
                                    item.set_check_state(CheckState::Unchecked);
                                    editions += 1;
                                }
                            }

                            FieldType::F32 => {
                                if !current_value.is_empty() {
                                    item.set_data_2a(&QVariant::from_float(0.0f32), 2);
                                    editions += 1;
                                }
                            }

                            FieldType::I16 => {
                                if !current_value.is_empty() {
                                    item.set_data_2a(&QVariant::from_int(0i32), 2);
                                    editions += 1;
                                }
                            }

                            FieldType::I32 => {
                                if !current_value.is_empty() {
                                    item.set_data_2a(&QVariant::from_int(0i32), 2);
                                    editions += 1;
                                }
                            }

                            FieldType::I64 => {
                                if !current_value.is_empty() {
                                    item.set_data_2a(&QVariant::from_i64(0i64), 2);
                                    editions += 1;
                                }
                            }

                            _ => {
                                if !current_value.is_empty() {
                                    item.set_text(&QString::from_std_str(""));
                                    editions += 1;
                                }
                            }
                        }
                    }
                }

                // Then, we delete all the fully selected rows.
                let rows_splitted = super::utils::delete_rows(view.table_model, &full_rows);

                // Then, we have to fix the undo history. For that, we take out all the editions, merge them,
                // then merge them with the table edition into a carolina.
                if editions > 0 || !rows_splitted.is_empty() {

                    // Update the search stuff, if needed.
                    //unsafe { update_search_stuff.as_mut().unwrap().trigger(); }

                     {
                        let mut changes = vec![];
                        if !rows_splitted.is_empty() {
                            changes.push(TableOperations::RemoveRows(rows_splitted));
                        }

                        let len = view.history_undo.read().unwrap().len();
                        let editions: Vec<((i32, i32), AtomicPtr<QStandardItem>)> = view.history_undo.write().unwrap()
                            .drain(len - editions..)
                            .filter_map(|x| if let TableOperations::Editing(y) = x { Some(y) } else { None })
                            .flatten()
                            .collect();

                        if !editions.is_empty() {
                            changes.push(TableOperations::Editing(editions));
                        }

                        if !changes.is_empty() {
                            view.history_undo.write().unwrap().push(TableOperations::Carolina(changes));
                            view.history_redo.write().unwrap().clear();
                            update_undo_model(view.table_model, view.undo_model);
                            view.context_menu_update();
                            set_modified(true, &view.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
                        }
                    }
                }*/
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            sort_order_column_changed_1,
            sort_order_column_changed_2,
            show_context_menu_1,
            show_context_menu_2,
            context_menu_enabler_1,
            context_menu_enabler_2,
            item_changed_1,
            item_changed_2,
            add_rows,
            insert_rows,
            delete_rows,
            clone_and_append,
            clone_and_insert,
            copy,
            copy_as_lua_table,
            paste,
            invert_selection,
            reset_selection,
            rewrite_selection,
            save,
            undo,
            redo,
            smart_delete,
            resize_columns,
        }
    }
}

