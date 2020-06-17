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
Module with all the code for managing the view for Raw part of AnimFragment PackedFiles.
!*/

use qt_widgets::QAction;
use qt_widgets::QLineEdit;
use qt_widgets::QTableView;
use qt_widgets::QMenu;

use qt_gui::QBrush;
use qt_gui::QStandardItemModel;
use qt_gui::QStandardItem;

use qt_core::QSignalBlocker;
use qt_core::QSortFilterProxyModel;

use cpp_core::MutPtr;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use rpfm_lib::schema::Definition;

use crate::app_ui::AppUI;
use crate::pack_tree::get_color_modified;
use crate::packedfile_views::table::TableOperations;
use crate::packedfile_views::table::utils::*;
use crate::packedfile_views::utils::set_modified;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::atomic_from_mut_ptr;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the raw version of each pointer in `PackedFileAnimFragmentView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileAnimFragmentView`.
#[derive(Clone)]
pub struct PackedFileAnimFragmentViewRaw {
    pub table_1: MutPtr<QTableView>,
    pub table_2: MutPtr<QTableView>,
    pub integer_1: MutPtr<QLineEdit>,
    pub integer_2: MutPtr<QLineEdit>,
    pub path: Arc<RwLock<Vec<String>>>,
    pub definition: Arc<RwLock<Definition>>,

    pub column_sort_state_1: Arc<RwLock<(i32, i8)>>,
    pub column_sort_state_2: Arc<RwLock<(i32, i8)>>,

    pub context_menu_1: MutPtr<QMenu>,
    pub context_menu_2: MutPtr<QMenu>,

    pub context_menu_enabler_1: MutPtr<QAction>,
    pub context_menu_add_rows_1: MutPtr<QAction>,
    pub context_menu_insert_rows_1: MutPtr<QAction>,
    pub context_menu_delete_rows_1: MutPtr<QAction>,
    pub context_menu_clone_and_append_1: MutPtr<QAction>,
    pub context_menu_clone_and_insert_1: MutPtr<QAction>,
    pub context_menu_copy_1: MutPtr<QAction>,
    pub context_menu_copy_as_lua_table_1: MutPtr<QAction>,
    pub context_menu_paste_1: MutPtr<QAction>,
    pub context_menu_invert_selection_1: MutPtr<QAction>,
    pub context_menu_reset_selection_1: MutPtr<QAction>,
    pub context_menu_rewrite_selection_1: MutPtr<QAction>,
    pub context_menu_undo_1: MutPtr<QAction>,
    pub context_menu_redo_1: MutPtr<QAction>,
    pub context_menu_resize_columns_1: MutPtr<QAction>,

    pub context_menu_enabler_2: MutPtr<QAction>,
    pub context_menu_add_rows_2: MutPtr<QAction>,
    pub context_menu_insert_rows_2: MutPtr<QAction>,
    pub context_menu_delete_rows_2: MutPtr<QAction>,
    pub context_menu_clone_and_append_2: MutPtr<QAction>,
    pub context_menu_clone_and_insert_2: MutPtr<QAction>,
    pub context_menu_copy_2: MutPtr<QAction>,
    pub context_menu_copy_as_lua_table_2: MutPtr<QAction>,
    pub context_menu_paste_2: MutPtr<QAction>,
    pub context_menu_invert_selection_2: MutPtr<QAction>,
    pub context_menu_reset_selection_2: MutPtr<QAction>,
    pub context_menu_rewrite_selection_2: MutPtr<QAction>,
    pub context_menu_undo_2: MutPtr<QAction>,
    pub context_menu_redo_2: MutPtr<QAction>,
    pub context_menu_resize_columns_2: MutPtr<QAction>,

    pub smart_delete_1: MutPtr<QAction>,
    pub smart_delete_2: MutPtr<QAction>,

    pub save_lock: Arc<AtomicBool>,
    pub undo_lock: Arc<AtomicBool>,

    pub undo_model_1: MutPtr<QStandardItemModel>,
    pub undo_model_2: MutPtr<QStandardItemModel>,

    pub packed_file_path: Arc<RwLock<Vec<String>>>,
    pub history_undo_1: Arc<RwLock<Vec<TableOperations>>>,
    pub history_redo_1: Arc<RwLock<Vec<TableOperations>>>,
    pub history_undo_2: Arc<RwLock<Vec<TableOperations>>>,
    pub history_redo_2: Arc<RwLock<Vec<TableOperations>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimFragmentViewRaw`.
impl PackedFileAnimFragmentViewRaw {

    /// This function updates the state of the actions in the context menu.
    pub unsafe fn context_menu_update_1(&mut self) {

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
        let filter: MutPtr<QSortFilterProxyModel> = self.table_1.model().static_downcast_mut();
        let indexes = filter.map_selection_to_source(&self.table_1.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            self.context_menu_clone_and_append_1.set_enabled(true);
            self.context_menu_clone_and_insert_1.set_enabled(true);
            self.context_menu_copy_1.set_enabled(true);
            self.context_menu_copy_as_lua_table_1.set_enabled(true);
            self.context_menu_delete_rows_1.set_enabled(true);
            self.context_menu_rewrite_selection_1.set_enabled(true);
        }

        // Otherwise, disable them.
        else {
            self.context_menu_rewrite_selection_1.set_enabled(false);
            self.context_menu_clone_and_append_1.set_enabled(false);
            self.context_menu_clone_and_insert_1.set_enabled(false);
            self.context_menu_copy_1.set_enabled(false);
            self.context_menu_copy_as_lua_table_1.set_enabled(false);
            self.context_menu_delete_rows_1.set_enabled(false);
        }

        if !self.undo_lock.load(Ordering::SeqCst) {
            self.context_menu_undo_1.set_enabled(!self.history_undo_1.read().unwrap().is_empty());
            self.context_menu_redo_1.set_enabled(!self.history_redo_1.read().unwrap().is_empty());
        }
    }

    /// This function updates the state of the actions in the context menu.
    pub unsafe fn context_menu_update_2(&mut self) {

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
        let filter: MutPtr<QSortFilterProxyModel> = self.table_2.model().static_downcast_mut();
        let indexes = filter.map_selection_to_source(&self.table_2.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            self.context_menu_clone_and_append_2.set_enabled(true);
            self.context_menu_clone_and_insert_2.set_enabled(true);
            self.context_menu_copy_2.set_enabled(true);
            self.context_menu_copy_as_lua_table_2.set_enabled(true);
            self.context_menu_delete_rows_2.set_enabled(true);
            self.context_menu_rewrite_selection_2.set_enabled(true);
        }

        // Otherwise, disable them.
        else {
            self.context_menu_rewrite_selection_2.set_enabled(false);
            self.context_menu_clone_and_append_2.set_enabled(false);
            self.context_menu_clone_and_insert_2.set_enabled(false);
            self.context_menu_copy_2.set_enabled(false);
            self.context_menu_copy_as_lua_table_2.set_enabled(false);
            self.context_menu_delete_rows_2.set_enabled(false);
        }

        if !self.undo_lock.load(Ordering::SeqCst) {
            self.context_menu_undo_2.set_enabled(!self.history_undo_2.read().unwrap().is_empty());
            self.context_menu_redo_2.set_enabled(!self.history_redo_2.read().unwrap().is_empty());
        }
    }

    pub unsafe fn item_changed(
        &mut self,
        item: MutPtr<QStandardItem>,
        mut app_ui: &mut AppUI,
        mut pack_file_contents_ui: &mut PackFileContentsUI,
        use_table_1: bool,
    ) {

        if !self.undo_lock.load(Ordering::SeqCst) {
            if use_table_1 {

            // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                let filter: MutPtr<QSortFilterProxyModel> = self.table_1.model().static_downcast_mut();
                let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                let item_old = self.undo_model_1.item_2a(item.row(), item.column());

                // Only trigger this if the values are actually different. Checkable cells are tricky.
                if item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state() {
                    let mut edition = Vec::with_capacity(1);
                    edition.push(((item.row(), item.column()), atomic_from_mut_ptr((&*item_old).clone())));
                    let operation = TableOperations::Editing(edition);
                    self.history_undo_1.write().unwrap().push(operation);
                    self.history_redo_1.write().unwrap().clear();

                    {
                        // We block the saving for painting, so this doesn't get rettriggered again.
                        let mut blocker = QSignalBlocker::from_q_object(model);
                        let color = get_color_modified();
                        let mut item = item;
                        item.set_background(&QBrush::from_q_color(color.as_ref().unwrap()));
                        blocker.unblock();
                    }

                    // For pasting, or really any heavy operation, only do these tasks the last iteration of the operation.
                    if !self.save_lock.load(Ordering::SeqCst) {
                        update_undo_model(model, self.undo_model_1);
                        self.context_menu_update_1();
                        set_modified(true, &self.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
                    }
                }
            }

            else {

                // If we are NOT UNDOING, paint the item as edited and add the edition to the undo list.
                let filter: MutPtr<QSortFilterProxyModel> = self.table_2.model().static_downcast_mut();
                let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                let item_old = self.undo_model_2.item_2a(item.row(), item.column());

                // Only trigger this if the values are actually different. Checkable cells are tricky.
                if item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state() {
                    let mut edition = Vec::with_capacity(1);
                    edition.push(((item.row(), item.column()), atomic_from_mut_ptr((&*item_old).clone())));
                    let operation = TableOperations::Editing(edition);
                    self.history_undo_2.write().unwrap().push(operation);
                    self.history_redo_2.write().unwrap().clear();

                    {
                        // We block the saving for painting, so this doesn't get rettriggered again.
                        let mut blocker = QSignalBlocker::from_q_object(model);
                        let color = get_color_modified();
                        let mut item = item;
                        item.set_background(&QBrush::from_q_color(color.as_ref().unwrap()));
                        blocker.unblock();
                    }

                    // For pasting, or really any heavy operation, only do these tasks the last iteration of the operation.
                    if !self.save_lock.load(Ordering::SeqCst) {
                        update_undo_model(model, self.undo_model_2);
                        self.context_menu_update_1();
                        set_modified(true, &self.packed_file_path.read().unwrap(), &mut app_ui, &mut pack_file_contents_ui);
                    }
                }
            }
        }
    }
}
