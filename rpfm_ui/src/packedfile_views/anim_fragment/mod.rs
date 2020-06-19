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
Module with all the code for managing the view for AnimFragment PackedFiles.
!*/

use qt_widgets::QAction;
use qt_widgets::QLineEdit;
use qt_widgets::QTableView;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QStandardItem;

use qt_core::CheckState;
use qt_core::QString;
use qt_core::QSortFilterProxyModel;

use cpp_core::MutPtr;

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;

use rpfm_lib::packedfile::DecodedPackedFile;
use rpfm_lib::packedfile::table::{DecodedData, Table};
use rpfm_lib::packedfile::table::anim_fragment::AnimFragment;
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::{Definition, FieldType};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::packedfile_views::table::{COLUMN_SIZE_BOOLEAN, COLUMN_SIZE_NUMBER, COLUMN_SIZE_STRING};
use crate::packedfile_views::table::utils::*;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::atomic_from_mut_ptr;
use crate::utils::mut_ptr_from_atomic;

use self::raw::PackedFileAnimFragmentViewRaw;
use self::slots::PackedFileAnimFragmentViewSlots;

mod connections;
pub mod slots;
mod raw;
mod shortcuts;
mod utils;

/// Fields that are represented via a bitwise number, and how many bits it uses.
const BITWISE_FIELDS: [(&str, u16); 1] = [
    ("weapon_bone", 6),
];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimFragment PackedFile.
pub struct PackedFileAnimFragmentView {
    table_1: AtomicPtr<QTableView>,
    table_2: AtomicPtr<QTableView>,
    integer_1: AtomicPtr<QLineEdit>,
    integer_2: AtomicPtr<QLineEdit>,
    definition: Arc<RwLock<Definition>>,

    context_menu_add_rows_1: AtomicPtr<QAction>,
    context_menu_insert_rows_1: AtomicPtr<QAction>,
    context_menu_delete_rows_1: AtomicPtr<QAction>,
    context_menu_clone_and_append_1: AtomicPtr<QAction>,
    context_menu_clone_and_insert_1: AtomicPtr<QAction>,
    context_menu_copy_1: AtomicPtr<QAction>,
    context_menu_paste_1: AtomicPtr<QAction>,
    context_menu_invert_selection_1: AtomicPtr<QAction>,
    context_menu_reset_selection_1: AtomicPtr<QAction>,
    context_menu_rewrite_selection_1: AtomicPtr<QAction>,
    context_menu_undo_1: AtomicPtr<QAction>,
    context_menu_redo_1: AtomicPtr<QAction>,
    context_menu_resize_columns_1: AtomicPtr<QAction>,
    smart_delete_1: AtomicPtr<QAction>,

    context_menu_add_rows_2: AtomicPtr<QAction>,
    context_menu_insert_rows_2: AtomicPtr<QAction>,
    context_menu_delete_rows_2: AtomicPtr<QAction>,
    context_menu_clone_and_append_2: AtomicPtr<QAction>,
    context_menu_clone_and_insert_2: AtomicPtr<QAction>,
    context_menu_copy_2: AtomicPtr<QAction>,
    context_menu_paste_2: AtomicPtr<QAction>,
    context_menu_invert_selection_2: AtomicPtr<QAction>,
    context_menu_reset_selection_2: AtomicPtr<QAction>,
    context_menu_rewrite_selection_2: AtomicPtr<QAction>,
    context_menu_undo_2: AtomicPtr<QAction>,
    context_menu_redo_2: AtomicPtr<QAction>,
    context_menu_resize_columns_2: AtomicPtr<QAction>,
    smart_delete_2: AtomicPtr<QAction>,

    undo_model_1: AtomicPtr<QStandardItemModel>,
    undo_model_2: AtomicPtr<QStandardItemModel>,

    //packed_file_path: Arc<RwLock<Vec<String>>>,
    //history_undo_1: Arc<RwLock<Vec<TableOperations>>>,
    //history_redo_1: Arc<RwLock<Vec<TableOperations>>>,
    //history_undo_2: Arc<RwLock<Vec<TableOperations>>>,
    //history_redo_2: Arc<RwLock<Vec<TableOperations>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Macro to generate all the getters at once.
macro_rules! getter_generator {
    (
        $get_definition:ident,
        $get_mut_ptr_table:ident,
        $get_mut_ptr_context_menu_add_rows:ident,
        $get_mut_ptr_context_menu_insert_rows:ident,
        $get_mut_ptr_context_menu_delete_rows:ident,
        $get_mut_ptr_context_menu_clone_and_append:ident,
        $get_mut_ptr_context_menu_clone_and_insert:ident,
        $get_mut_ptr_context_menu_copy:ident,
        $get_mut_ptr_context_menu_paste:ident,
        $get_mut_ptr_context_menu_invert_selection:ident,
        $get_mut_ptr_context_menu_reset_selection:ident,
        $get_mut_ptr_context_menu_rewrite_selection:ident,
        $get_mut_ptr_context_menu_undo:ident,
        $get_mut_ptr_context_menu_redo:ident,
        $get_mut_ptr_context_menu_resize_columns:ident,
        $get_mut_ptr_smart_delete:ident,

        $table:ident,
        $column_sort_state:ident,
        $context_menu:ident,
        $context_menu_enabler:ident,
        $context_menu_add_rows:ident,
        $context_menu_insert_rows:ident,
        $context_menu_delete_rows:ident,
        $context_menu_clone_and_append:ident,
        $context_menu_clone_and_insert:ident,
        $context_menu_copy:ident,
        $context_menu_paste:ident,
        $context_menu_invert_selection:ident,
        $context_menu_reset_selection:ident,
        $context_menu_rewrite_selection:ident,
        $context_menu_undo:ident,
        $context_menu_redo:ident,
        $context_menu_resize_columns:ident,
        $smart_delete:ident,

        $field:expr,
    ) => {

        /// This function returns a copy of the definition used by the first sequence of this AnimFragment.
        pub fn $get_definition(&self) -> Definition {
            let definition = self.definition.read().unwrap();
            if let FieldType::SequenceU32(definition) = &(*definition).fields[$field].field_type {
                definition.clone()
            }
            else { unimplemented!() }
        }

        /// This function returns a pointer to the TableView of this table.
        pub fn $get_mut_ptr_table(&self) -> MutPtr<QTableView> {
            mut_ptr_from_atomic(&self.$table)
        }

        /// This function returns a pointer to the add rows action.
        pub fn $get_mut_ptr_context_menu_add_rows(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_add_rows)
        }

        /// This function returns a pointer to the insert rows action.
        pub fn $get_mut_ptr_context_menu_insert_rows(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_insert_rows)
        }

        /// This function returns a pointer to the delete rows action.
        pub fn $get_mut_ptr_context_menu_delete_rows(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_delete_rows)
        }

        /// This function returns a pointer to the clone_and_append action.
        pub fn $get_mut_ptr_context_menu_clone_and_append(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_clone_and_append)
        }

        /// This function returns a pointer to the clone_and_insert action.
        pub fn $get_mut_ptr_context_menu_clone_and_insert(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_clone_and_insert)
        }

        /// This function returns a pointer to the copy action.
        pub fn $get_mut_ptr_context_menu_copy(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_copy)
        }

        /// This function returns a pointer to the paste action.
        pub fn $get_mut_ptr_context_menu_paste(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_paste)
        }

        /// This function returns a pointer to the invert selection action.
        pub fn $get_mut_ptr_context_menu_invert_selection(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_invert_selection)
        }

        /// This function returns a pointer to the reset selection action.
        pub fn $get_mut_ptr_context_menu_reset_selection(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_reset_selection)
        }

        /// This function returns a pointer to the rewrite selection action.
        pub fn $get_mut_ptr_context_menu_rewrite_selection(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_rewrite_selection)
        }

        /// This function returns a pointer to the undo action.
        pub fn $get_mut_ptr_context_menu_undo(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_undo)
        }

        /// This function returns a pointer to the redo action.
        pub fn $get_mut_ptr_context_menu_redo(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_redo)
        }

        /// This function returns a pointer to the smart delete action.
        pub fn $get_mut_ptr_smart_delete(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$smart_delete)
        }

        /// This function returns a pointer to the resize columns action.
        pub fn $get_mut_ptr_context_menu_resize_columns(&self) -> MutPtr<QAction> {
            mut_ptr_from_atomic(&self.$context_menu_resize_columns)
        }
    }
}

/// Implementation for `PackedFileAnimFragmentView`.
impl PackedFileAnimFragmentView {

    /// This function creates a new AnimFraagment View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &AppUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, PackedFileInfo)> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::AnimFragmentPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();

        let i1_label = QLabel::from_q_string(&qtr("integer1"));
        let i2_label = QLabel::from_q_string(&qtr("integer2"));

        let mut i1_line_edit = QLineEdit::new();
        let mut i2_line_edit = QLineEdit::new();

        let mut filter_model = QSortFilterProxyModel::new_0a();
        let model_1 = QStandardItemModel::new_0a().into_ptr();
        filter_model.set_source_model(model_1);
        let (mut table_view_1, _) = new_tableview_frozen_safe(&mut packed_file_view.get_mut_widget());
        set_frozen_data_model_safe(&mut table_view_1, &mut filter_model.into_ptr());

        let mut filter_model = QSortFilterProxyModel::new_0a();
        let model_2 = QStandardItemModel::new_0a().into_ptr();
        filter_model.set_source_model(model_2);
        let (mut table_view_2, _) = new_tableview_frozen_safe(&mut packed_file_view.get_mut_widget());
        set_frozen_data_model_safe(&mut table_view_2, &mut filter_model.into_ptr());

        layout.add_widget_5a(i1_label.into_ptr(), 0, 0, 1, 1);
        layout.add_widget_5a(i2_label.into_ptr(), 1, 0, 1, 1);

        layout.add_widget_5a(&mut i1_line_edit, 0, 1, 1, 1);
        layout.add_widget_5a(&mut i2_line_edit, 1, 1, 1, 1);

        layout.add_widget_5a(table_view_1, 0, 2, 2, 1);
        layout.add_widget_5a(table_view_2, 2, 0, 1, 3);


        // Create the locks for undoing and saving. These are needed to optimize the undo/saving process.
        let undo_lock = Arc::new(AtomicBool::new(false));
        let save_lock = Arc::new(AtomicBool::new(false));

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
            table_view_1.horizontal_header().set_stretch_last_section(true);
            table_view_2.horizontal_header().set_stretch_last_section(true);
        }

        // Setup tight mode if the setting is enabled.
        if SETTINGS.read().unwrap().settings_bool["tight_table_mode"] {
            table_view_1.vertical_header().set_minimum_section_size(22);
            table_view_1.vertical_header().set_maximum_section_size(22);
            table_view_1.vertical_header().set_default_section_size(22);

            table_view_2.vertical_header().set_minimum_section_size(22);
            table_view_2.vertical_header().set_maximum_section_size(22);
            table_view_2.vertical_header().set_default_section_size(22);
        }

        // Action to make the delete button delete contents.
        let smart_delete_1 = QAction::new().into_ptr();
        let smart_delete_2 = QAction::new().into_ptr();

        // Create the Contextual Menu for the TableView.
        let context_menu_enabler_1 = QAction::new();
        let context_menu_enabler_2 = QAction::new();
        let mut context_menu_1 = QMenu::new().into_ptr();
        let mut context_menu_2 = QMenu::new().into_ptr();

        let context_menu_add_rows_1 = context_menu_1.add_action_q_string(&qtr("context_menu_add_rows"));
        let context_menu_insert_rows_1 = context_menu_1.add_action_q_string(&qtr("context_menu_insert_rows"));
        let context_menu_delete_rows_1 = context_menu_1.add_action_q_string(&qtr("context_menu_delete_rows"));
        let mut context_menu_clone_submenu_1 = QMenu::from_q_string(&qtr("context_menu_clone_submenu"));
        let context_menu_clone_and_insert_1 = context_menu_clone_submenu_1.add_action_q_string(&qtr("context_menu_clone_and_insert"));
        let context_menu_clone_and_append_1 = context_menu_clone_submenu_1.add_action_q_string(&qtr("context_menu_clone_and_append"));
        let context_menu_copy_1 = context_menu_1.add_action_q_string(&qtr("context_menu_copy"));
        let context_menu_paste_1 = context_menu_1.add_action_q_string(&qtr("context_menu_paste"));
        let context_menu_rewrite_selection_1 = context_menu_1.add_action_q_string(&qtr("context_menu_rewrite_selection"));
        let context_menu_invert_selection_1 = context_menu_1.add_action_q_string(&qtr("context_menu_invert_selection"));
        let context_menu_reset_selection_1 = context_menu_1.add_action_q_string(&qtr("context_menu_reset_selection"));
        let context_menu_resize_columns_1 = context_menu_1.add_action_q_string(&qtr("context_menu_resize_columns"));
        let context_menu_undo_1 = context_menu_1.add_action_q_string(&qtr("context_menu_undo"));
        let context_menu_redo_1 = context_menu_1.add_action_q_string(&qtr("context_menu_redo"));

        let context_menu_add_rows_2 = context_menu_2.add_action_q_string(&qtr("context_menu_add_rows"));
        let context_menu_insert_rows_2 = context_menu_2.add_action_q_string(&qtr("context_menu_insert_rows"));
        let context_menu_delete_rows_2 = context_menu_2.add_action_q_string(&qtr("context_menu_delete_rows"));
        let mut context_menu_clone_submenu_2 = QMenu::from_q_string(&qtr("context_menu_clone_submenu"));
        let context_menu_clone_and_insert_2 = context_menu_clone_submenu_2.add_action_q_string(&qtr("context_menu_clone_and_insert"));
        let context_menu_clone_and_append_2 = context_menu_clone_submenu_2.add_action_q_string(&qtr("context_menu_clone_and_append"));
        let context_menu_copy_2 = context_menu_2.add_action_q_string(&qtr("context_menu_copy"));
        let context_menu_paste_2 = context_menu_2.add_action_q_string(&qtr("context_menu_paste"));
        let context_menu_rewrite_selection_2 = context_menu_2.add_action_q_string(&qtr("context_menu_rewrite_selection"));
        let context_menu_invert_selection_2 = context_menu_2.add_action_q_string(&qtr("context_menu_invert_selection"));
        let context_menu_reset_selection_2 = context_menu_2.add_action_q_string(&qtr("context_menu_reset_selection"));
        let context_menu_resize_columns_2 = context_menu_2.add_action_q_string(&qtr("context_menu_resize_columns"));
        let context_menu_undo_2 = context_menu_2.add_action_q_string(&qtr("context_menu_undo"));
        let context_menu_redo_2 = context_menu_2.add_action_q_string(&qtr("context_menu_redo"));

        // Insert some separators to space the menu, and the paste submenu.
        context_menu_1.insert_menu(context_menu_paste_1, context_menu_clone_submenu_1.into_ptr());
        context_menu_1.insert_separator(context_menu_rewrite_selection_1);
        context_menu_1.insert_separator(context_menu_undo_1);

        context_menu_2.insert_menu(context_menu_paste_2, context_menu_clone_submenu_2.into_ptr());
        context_menu_2.insert_separator(context_menu_rewrite_selection_2);
        context_menu_2.insert_separator(context_menu_undo_2);

        let mut packed_file_anim_fragment_view_raw = PackedFileAnimFragmentViewRaw {
            table_1: table_view_1,
            table_2: table_view_2,
            integer_1: i1_line_edit.into_ptr(),
            integer_2: i2_line_edit.into_ptr(),
            path: packed_file_view.get_path_raw().clone(),
            definition: Arc::new(RwLock::new(data.get_definition())),

            column_sort_state_1: Arc::new(RwLock::new((-1, 0))),
            column_sort_state_2: Arc::new(RwLock::new((-1, 0))),

            context_menu_1,
            context_menu_2,

            context_menu_enabler_1: context_menu_enabler_1.into_ptr(),
            context_menu_add_rows_1,
            context_menu_insert_rows_1,
            context_menu_delete_rows_1,
            context_menu_clone_and_append_1,
            context_menu_clone_and_insert_1,
            context_menu_copy_1,
            context_menu_paste_1,
            context_menu_invert_selection_1,
            context_menu_reset_selection_1,
            context_menu_rewrite_selection_1,
            context_menu_undo_1,
            context_menu_redo_1,
            context_menu_resize_columns_1,
            smart_delete_1,

            context_menu_enabler_2: context_menu_enabler_2.into_ptr(),
            context_menu_add_rows_2,
            context_menu_insert_rows_2,
            context_menu_delete_rows_2,
            context_menu_clone_and_append_2,
            context_menu_clone_and_insert_2,
            context_menu_copy_2,
            context_menu_paste_2,
            context_menu_invert_selection_2,
            context_menu_reset_selection_2,
            context_menu_rewrite_selection_2,
            context_menu_undo_2,
            context_menu_redo_2,
            context_menu_resize_columns_2,
            smart_delete_2,

            packed_file_path: packed_file_view.get_path_raw().clone(),

            undo_lock,
            save_lock,

            undo_model_1: QStandardItemModel::new_0a().into_ptr(),
            undo_model_2: QStandardItemModel::new_0a().into_ptr(),

            history_undo_1: Arc::new(RwLock::new(vec![])),
            history_redo_1: Arc::new(RwLock::new(vec![])),
            history_undo_2: Arc::new(RwLock::new(vec![])),
            history_redo_2: Arc::new(RwLock::new(vec![])),
        };

        let packed_file_anim_fragment_view_slots = PackedFileAnimFragmentViewSlots::new(
            packed_file_anim_fragment_view_raw.clone(),
            *app_ui,
            *pack_file_contents_ui,
            *global_search_ui,
        );

        let mut packed_file_anim_fragment_view = Self {
            table_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.table_1),
            table_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.table_2),
            integer_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.integer_1),
            integer_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.integer_2),
            definition: packed_file_anim_fragment_view_raw.definition.clone(),

            context_menu_add_rows_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_add_rows_1),
            context_menu_insert_rows_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_insert_rows_1),
            context_menu_delete_rows_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_delete_rows_1),
            context_menu_clone_and_append_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_clone_and_append_1),
            context_menu_clone_and_insert_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_clone_and_insert_1),
            context_menu_copy_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_copy_1),
            context_menu_paste_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_paste_1),
            context_menu_invert_selection_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_invert_selection_1),
            context_menu_reset_selection_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_reset_selection_1),
            context_menu_rewrite_selection_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_rewrite_selection_1),
            context_menu_undo_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_undo_1),
            context_menu_redo_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_redo_1),
            context_menu_resize_columns_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_resize_columns_1),
            smart_delete_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.smart_delete_1),

            context_menu_add_rows_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_add_rows_2),
            context_menu_insert_rows_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_insert_rows_2),
            context_menu_delete_rows_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_delete_rows_2),
            context_menu_clone_and_append_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_clone_and_append_2),
            context_menu_clone_and_insert_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_clone_and_insert_2),
            context_menu_copy_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_copy_2),
            context_menu_paste_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_paste_2),
            context_menu_invert_selection_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_invert_selection_2),
            context_menu_reset_selection_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_reset_selection_2),
            context_menu_rewrite_selection_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_rewrite_selection_2),
            context_menu_undo_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_undo_2),
            context_menu_redo_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_redo_2),
            context_menu_resize_columns_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.context_menu_resize_columns_2),
            smart_delete_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.smart_delete_2),

            undo_model_1: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.undo_model_1),
            undo_model_2: atomic_from_mut_ptr(packed_file_anim_fragment_view_raw.undo_model_2),

            //packed_file_path: packed_file_view.get_path_raw().clone(),
            //history_undo_1: packed_file_anim_fragment_view_raw.history_undo_1.clone(),
            //history_redo_1: packed_file_anim_fragment_view_raw.history_redo_1.clone(),
            //history_undo_2: packed_file_anim_fragment_view_raw.history_undo_2.clone(),
            //history_redo_2: packed_file_anim_fragment_view_raw.history_redo_2.clone(),
        };

        Self::load_data(&mut packed_file_anim_fragment_view_raw, &data)?;

        // Initialize the undo models.
        update_undo_model(model_1, mut_ptr_from_atomic(&packed_file_anim_fragment_view.undo_model_1));
        update_undo_model(model_2, mut_ptr_from_atomic(&packed_file_anim_fragment_view.undo_model_2));

        connections::set_connections(&packed_file_anim_fragment_view, &packed_file_anim_fragment_view_slots);
        shortcuts::set_shortcuts(&mut packed_file_anim_fragment_view);
        packed_file_view.view = ViewType::Internal(View::AnimFragment(packed_file_anim_fragment_view));
        packed_file_view.packed_file_type = PackedFileType::AnimFragment;

        Ok((TheOneSlot::AnimFragment(packed_file_anim_fragment_view_slots), packed_file_info))
    }

    /// This function takes care of loading the data into the AnimFragment View.
    pub unsafe fn load_data(ui: &mut PackedFileAnimFragmentViewRaw, bdata: &AnimFragment) -> Result<()> {
        match bdata.get_table_data().get(0) {
            Some(data) => {
                ui.integer_1.set_text(&QString::from_std_str(&data[1].data_to_string()));
                ui.integer_2.set_text(&QString::from_std_str(&data[2].data_to_string()));

                let filter: MutPtr<QSortFilterProxyModel> = ui.table_1.model().static_downcast_mut();
                let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                if let Some(data) = data.get(0) {
                    if let DecodedData::SequenceU32(data) = data {
                        let definition = data.get_definition();
                        for entry in data.get_table_data(){
                            Self::load_entry(table_model, &entry, &definition);
                        }
                        Self::build_columns(ui.table_1, &data.get_definition());
                    }
                }

                let filter: MutPtr<QSortFilterProxyModel> = ui.table_2.model().static_downcast_mut();
                let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                if let Some(data) = data.get(3) {
                    if let DecodedData::SequenceU32(data) = data {
                        let definition = data.get_definition();
                        for entry in data.get_table_data(){
                            Self::load_entry(table_model, &entry, &definition);
                        }
                        Self::build_columns(ui.table_2, &data.get_definition());
                    }
                }

                Ok(())
            }
            None => Err(ErrorKind::Generic.into()),
        }
    }

    /// This function takes care of loading each entry's data into the provided model.
    unsafe fn load_entry(mut model: MutPtr<QStandardItemModel>, entry: &[DecodedData], definition: &Definition) {
        let mut qlist = QListOfQStandardItem::new();
        for (column, field) in entry.iter().enumerate() {

            // If the column in question is a bitwise field, split it in as many columns as needed.
            if let Some((_, amount)) = BITWISE_FIELDS.iter().find(|x| x.0 == definition.fields[column].name) {
                let data = if let DecodedData::I32(data) = field { data } else { unimplemented!() };
                for index in 0..*amount {
                    let item = get_item_from_decoded_data(&DecodedData::Boolean(data & (1 << index) != 0));
                    add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
                }
            }
            else {
                let item = get_item_from_decoded_data(field);
                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
        }

        model.append_row_q_list_of_q_standard_item(&qlist);
    }

    /// This function takes care of building a DecodedPackedFile from the view's data.
    pub unsafe fn save_data(&self) -> Result<DecodedPackedFile> {
        let mut table = AnimFragment::new(&self.get_definition());
        let mut data = vec![];
        let i1 = DecodedData::I32(mut_ptr_from_atomic(&self.integer_1).text().to_std_string().parse::<i32>()?);
        let i2 = DecodedData::I32(mut_ptr_from_atomic(&self.integer_2).text().to_std_string().parse::<i32>()?);

        let filter: MutPtr<QSortFilterProxyModel> = mut_ptr_from_atomic(&self.table_1).model().static_downcast_mut();
        let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
        let data_1 = Self::get_entries_from_table(table_model, &self.get_definition_1())?;

        let filter: MutPtr<QSortFilterProxyModel> = mut_ptr_from_atomic(&self.table_2).model().static_downcast_mut();
        let table_model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
        let data_2 = Self::get_entries_from_table(table_model, &self.get_definition_2())?;

        data.push(DecodedData::SequenceU32(data_1));
        data.push(i1);
        data.push(i2);
        data.push(DecodedData::SequenceU32(data_2));


        let data = vec![data; 1];
        table.set_table_data(&data)?;
        Ok(DecodedPackedFile::AnimFragment(table))
    }

    /// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
    /// His intended use is for just after we load/reload the data to the table.
    pub unsafe fn build_columns(
        mut table_view_primary: MutPtr<QTableView>,
        definition: &Definition,
    ) {
        let filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
        let mut model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
        let schema = SCHEMA.read().unwrap();

        // For each column, clean their name and set their width and tooltip.
        let mut index = 0;
        for (iteration, field) in definition.fields.iter().enumerate() {

            let columns = if let Some((_, amount)) = BITWISE_FIELDS.iter().find(|x| x.0 == definition.fields[iteration].name) { *amount } else { 1 };

            for column in 0..columns {
                let name = if columns > 1 { format!("{}_{}", field.name, column + 1) } else { field.name.to_owned() };
                let name = clean_column_names(&name);
                let mut item = QStandardItem::from_q_string(&QString::from_std_str(&name));
                set_column_tooltip(&schema, &field, "", &mut item);
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

                index += 1;
            }
        }

        // If we want to let the columns resize themselfs...
        if SETTINGS.read().unwrap().settings_bool["adjust_columns_to_content"] {
            table_view_primary.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function is used to build a table struct with the data of a TableView and it's definition.
    pub unsafe fn get_entries_from_table(model: MutPtr<QStandardItemModel>, definition: &Definition) -> Result<Table> {
        let mut entries = vec![];
        for row in 0..model.row_count_0a() {
            let mut new_row: Vec<DecodedData> = vec![];
            let mut column = 0;
            for (index, field) in definition.fields.iter().enumerate() {

                // Create a new Item.
                let item = if let Some((_, amount)) = BITWISE_FIELDS.iter().find(|x| x.0 == definition.fields[index].name) {
                    let mut data = 0;
                    for iteration in 0..*amount {
                        if model.item_2a(row as i32, column as i32).check_state() == CheckState::Checked {
                            data |= 1 << iteration;
                        }
                        column += 1;
                    }
                    DecodedData::I32(data)
                }

                else {
                    let data = match field.field_type {

                        // This one needs a couple of changes before turning it into an item in the table.
                        FieldType::Boolean => DecodedData::Boolean(model.item_2a(row as i32, column as i32).check_state() == CheckState::Checked),

                        // Numbers need parsing, and this can fail.
                        FieldType::F32 => DecodedData::F32(model.item_2a(row as i32, column as i32).data_1a(2).to_float_0a()),
                        FieldType::I16 => DecodedData::I16(model.item_2a(row as i32, column as i32).data_1a(2).to_int_0a() as i16),
                        FieldType::I32 => DecodedData::I32(model.item_2a(row as i32, column as i32).data_1a(2).to_int_0a()),
                        FieldType::I64 => DecodedData::I64(model.item_2a(row as i32, column as i32).data_1a(2).to_long_long_0a()),

                        // All these are just normal Strings.
                        FieldType::StringU8 => DecodedData::StringU8(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                        FieldType::StringU16 => DecodedData::StringU16(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                        FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),
                        FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(QString::to_std_string(&model.item_2a(row as i32, column as i32).text())),

                        // Sequences in the UI are not yet supported.
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => return Err(ErrorKind::Generic.into()),
                    };
                    column += 1;
                    data
                };
                new_row.push(item);
            }
            entries.push(new_row);
        }

        let mut table = Table::new(definition);
        table.set_table_data(&entries)?;
        Ok(table)
    }

    /// This function returns a copy of the definition of this AnimFragment.
    pub fn get_definition(&self) -> Definition {
        self.definition.read().unwrap().clone()
    }

    getter_generator!(
        get_definition_1,
        get_mut_ptr_table_1,
        get_mut_ptr_context_menu_add_rows_1,
        get_mut_ptr_context_menu_insert_rows_1,
        get_mut_ptr_context_menu_delete_rows_1,
        get_mut_ptr_context_menu_clone_and_append_1,
        get_mut_ptr_context_menu_clone_and_insert_1,
        get_mut_ptr_context_menu_copy_1,
        get_mut_ptr_context_menu_paste_1,
        get_mut_ptr_context_menu_invert_selection_1,
        get_mut_ptr_context_menu_reset_selection_1,
        get_mut_ptr_context_menu_rewrite_selection_1,
        get_mut_ptr_context_menu_undo_1,
        get_mut_ptr_context_menu_redo_1,
        get_mut_ptr_context_menu_resize_columns_1,
        get_mut_ptr_smart_delete_1,

        table_1,
        column_sort_state_1,
        context_menu_1,
        context_menu_enabler_1,
        context_menu_add_rows_1,
        context_menu_insert_rows_1,
        context_menu_delete_rows_1,
        context_menu_clone_and_append_1,
        context_menu_clone_and_insert_1,
        context_menu_copy_1,
        context_menu_paste_1,
        context_menu_invert_selection_1,
        context_menu_reset_selection_1,
        context_menu_rewrite_selection_1,
        context_menu_undo_1,
        context_menu_redo_1,
        context_menu_resize_columns_1,
        smart_delete_1,
        0,
    );
    getter_generator!(
        get_definition_2,
        get_mut_ptr_table_2,
        get_mut_ptr_context_menu_add_rows_2,
        get_mut_ptr_context_menu_insert_rows_2,
        get_mut_ptr_context_menu_delete_rows_2,
        get_mut_ptr_context_menu_clone_and_append_2,
        get_mut_ptr_context_menu_clone_and_insert_2,
        get_mut_ptr_context_menu_copy_2,
        get_mut_ptr_context_menu_paste_2,
        get_mut_ptr_context_menu_invert_selection_2,
        get_mut_ptr_context_menu_reset_selection_2,
        get_mut_ptr_context_menu_rewrite_selection_2,
        get_mut_ptr_context_menu_undo_2,
        get_mut_ptr_context_menu_redo_2,
        get_mut_ptr_context_menu_resize_columns_2,
        get_mut_ptr_smart_delete_2,

        table_2,
        column_sort_state_2,
        context_menu_2,
        context_menu_enabler_2,
        context_menu_add_rows_2,
        context_menu_insert_rows_2,
        context_menu_delete_rows_2,
        context_menu_clone_and_append_2,
        context_menu_clone_and_insert_2,
        context_menu_copy_2,
        context_menu_paste_2,
        context_menu_invert_selection_2,
        context_menu_reset_selection_2,
        context_menu_rewrite_selection_2,
        context_menu_undo_2,
        context_menu_redo_2,
        context_menu_resize_columns_2,
        smart_delete_2,
        3,
    );
}
