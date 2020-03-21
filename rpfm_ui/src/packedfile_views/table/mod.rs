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
Module with all the code for managing the view for Table PackedFiles.
!*/

use qt_widgets::QAction;
use qt_widgets::QComboBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QTableView;
use qt_widgets::QMenu;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CheckState;
use qt_core::QFlags;

use qt_core::QSortFilterProxyModel;
use qt_core::QStringList;
use qt_core::QVariant;
use qt_core::QString;
use qt_core::q_item_selection_model::SelectionFlag;

use cpp_core::CppBox;
use cpp_core::MutPtr;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::{fmt, fmt::Debug};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr};

use rpfm_error::Result;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::table::{DecodedData, db::DB, loc::Loc};
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::{Definition, Field, FieldType, Schema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::utils::{atomic_from_mut_ptr, mut_ptr_from_atomic};

use self::slots::PackedFileTableViewSlots;
use self::raw::*;
use self::utils::*;

mod connections;
pub mod slots;
mod raw;
mod shortcuts;
mod tips;
mod utils;

// Column default sizes.
static COLUMN_SIZE_BOOLEAN: i32 = 100;
static COLUMN_SIZE_NUMBER: i32 = 140;
static COLUMN_SIZE_STRING: i32 = 350;

static ITEM_HAS_SOURCE_VALUE: i32 = 30;
static ITEM_SOURCE_VALUE: i32 = 31;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum is used to distinguish between the different types of tables we can decode.
#[derive(Clone, Debug)]
pub enum TableType {
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    Loc(Loc),
}

/// Enum to know what operation was done while editing tables, so we can revert them with undo.
pub enum TableOperations {

    /// Intended for any kind of item editing. Holds a Vec<((row, column), AtomicPtr<item>)>, so we can do this in batches.
    Editing(Vec<((i32, i32), AtomicPtr<QStandardItem>)>),

    /// Intended for when adding/inserting rows. It holds a list of positions where the rows where inserted.
    AddRows(Vec<i32>),

    /// Intended for when removing rows. It holds a list of positions where the rows where deleted and the deleted rows data, in consecutive batches.
    RemoveRows(Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)>),

    /// It holds a copy of the entire table, before importing.
    ImportTSV(Vec<AtomicPtr<QListOfQStandardItem>>),

    /// A Jack-of-all-Trades. It holds a Vec<TableOperations>, for those situations one is not enough.
    Carolina(Vec<TableOperations>),
}

/// This struct contains pointers to all the widgets in a Table View.
pub struct PackedFileTableView {
    table_view_primary: AtomicPtr<QTableView>,
    table_view_frozen: AtomicPtr<QTableView>,
    table_filter: AtomicPtr<QSortFilterProxyModel>,
    table_model: AtomicPtr<QStandardItemModel>,
    table_enable_lookups_button: AtomicPtr<QPushButton>,
    filter_case_sensitive_button: AtomicPtr<QPushButton>,
    filter_column_selector: AtomicPtr<QComboBox>,
    filter_line_edit: AtomicPtr<QLineEdit>,

    context_menu: AtomicPtr<QMenu>,
    context_menu_enabler: AtomicPtr<QAction>,
    context_menu_add_rows: AtomicPtr<QAction>,
    context_menu_insert_rows: AtomicPtr<QAction>,
    context_menu_delete_rows: AtomicPtr<QAction>,
    context_menu_clone_and_append: AtomicPtr<QAction>,
    context_menu_clone_and_insert: AtomicPtr<QAction>,
    context_menu_copy: AtomicPtr<QAction>,
    context_menu_copy_as_lua_table: AtomicPtr<QAction>,
    context_menu_paste: AtomicPtr<QAction>,
    context_menu_invert_selection: AtomicPtr<QAction>,
    context_menu_reset_selection: AtomicPtr<QAction>,
    context_menu_undo: AtomicPtr<QAction>,
    context_menu_redo: AtomicPtr<QAction>,
    context_menu_import_tsv: AtomicPtr<QAction>,
    context_menu_export_tsv: AtomicPtr<QAction>,
    smart_delete: AtomicPtr<QAction>,

    dependency_data: Arc<RwLock<BTreeMap<i32, Vec<(String, String)>>>>,
    table_name: String,
    table_definition: Definition,

    save_lock: Arc<AtomicBool>,
    undo_lock: Arc<AtomicBool>,

    undo_model: AtomicPtr<QStandardItemModel>,
    history_undo: Arc<RwLock<Vec<TableOperations>>>,
    history_redo: Arc<RwLock<Vec<TableOperations>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableView`.
impl PackedFileTableView {

    /// This function creates a new Table View, and sets up his slots and connections.
    ///
    /// NOTE: To open the dependency list, pass it an empty path.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, Option<PackedFileInfo>)> {

        // Get the decoded Table.
        if packed_file_path.borrow().is_empty() { CENTRAL_COMMAND.send_message_qt(Command::GetDependencyPackFilesList); }
        else { CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFileTable(packed_file_path.borrow().to_vec())); }

        let response = CENTRAL_COMMAND.recv_message_qt();
        let (table_data, packed_file_info) = match response {
            Response::DBPackedFileInfo((table, packed_file_info)) => (TableType::DB(table), Some(packed_file_info)),
            Response::LocPackedFileInfo((table, packed_file_info)) => (TableType::Loc(table), Some(packed_file_info)),
            Response::VecString(table) => (TableType::DependencyManager(table.iter().map(|x| vec![DecodedData::StringU8(x.to_owned()); 1]).collect::<Vec<Vec<DecodedData>>>()), None),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let (table_definition, table_name, packed_file_type) = match table_data {
            TableType::DependencyManager(_) => {
                let schema = SCHEMA.read().unwrap();
                (schema.as_ref().unwrap().get_ref_versioned_file_dep_manager().unwrap().get_version_list()[0].clone(), String::new(), PackedFileType::DependencyPackFilesList)
            },
            TableType::DB(ref table) => (table.get_definition(), table.get_table_name(), PackedFileType::DB),
            TableType::Loc(ref table) => (table.get_definition(), String::new(), PackedFileType::Loc),
        };

        // Get the dependency data of this Table.
        CENTRAL_COMMAND.send_message_qt(Command::GetReferenceDataFromDefinition(table_definition.clone()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let dependency_data = match response {
            Response::BTreeMapI32VecStringString(dependency_data) => dependency_data,
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Create the locks for undoing and saving. These are needed to optimize the undo/saving process.
        let undo_lock = Arc::new(AtomicBool::new(false));
        let save_lock = Arc::new(AtomicBool::new(false));

        // Prepare the Table and its model.
        let mut table_view_frozen = QTableView::new_0a();
        let mut filter_model = QSortFilterProxyModel::new_0a();
        let mut model = QStandardItemModel::new_0a();
        filter_model.set_source_model(&mut model);
        let table_view = new_tableview_frozen_safe(&mut filter_model, &mut table_view_frozen);
        table_view_frozen.hide();

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.lock().unwrap().settings_bool["extend_last_column_on_tables"] {
            table_view.horizontal_header().set_stretch_last_section(true);
        }

        // Create the filter's widgets.
        let mut row_filter_line_edit = QLineEdit::new();
        let mut row_filter_column_selector = QComboBox::new_0a();
        let mut row_filter_case_sensitive_button = QPushButton::from_q_string(&qtr("table_filter_case_sensitive"));
        let row_filter_column_list = QStandardItemModel::new_0a().into_ptr();
        let mut table_enable_lookups_button = QPushButton::from_q_string(&qtr("table_enable_lookups"));

        row_filter_column_selector.set_model(row_filter_column_list);

        let mut fields = table_definition.fields.to_vec();
        fields.sort_by(|a, b| a.ca_order.cmp(&b.ca_order));
        for field in &fields {
            let name = clean_column_names(&field.name);
            row_filter_column_selector.add_item_q_string(&QString::from_std_str(&name));
        }

        row_filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        row_filter_case_sensitive_button.set_checkable(true);
        table_enable_lookups_button.set_checkable(true);

        // Add everything to the grid.
        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();
        layout.add_widget_5a(table_view, 0, 0, 1, 4);
        layout.add_widget_5a(&mut row_filter_line_edit, 2, 0, 1, 1);
        layout.add_widget_5a(&mut row_filter_case_sensitive_button, 2, 1, 1, 1);
        layout.add_widget_5a(&mut row_filter_column_selector, 2, 2, 1, 1);
        layout.add_widget_5a(&mut table_enable_lookups_button, 2, 3, 1, 1);

        // Action to make the delete button delete contents.
        let smart_delete = QAction::new().into_ptr();

        // Create the Contextual Menu for the TableView.
        let context_menu_enabler = QAction::new();
        let mut context_menu = QMenu::new().into_ptr();
        let context_menu_add_rows = context_menu.add_action_q_string(&QString::from_std_str("&Add Row"));
        let context_menu_insert_rows = context_menu.add_action_q_string(&QString::from_std_str("&Insert Row"));
        let context_menu_delete_rows = context_menu.add_action_q_string(&QString::from_std_str("&Delete Row"));
/*
        let mut context_menu_apply_submenu = Menu::new(&QString::from_std_str("A&pply..."));
        let context_menu_apply_maths_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Maths to Selection"));
        let context_menu_rewrite_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Rewrite Selection"));

        let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));
*/
        let context_menu_clone_and_insert = context_menu.add_action_q_string(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu.add_action_q_string(&QString::from_std_str("Clone and &Append"));
        let mut context_menu_copy_submenu = QMenu::from_q_string(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action_q_string(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action_q_string(&QString::from_std_str("&Copy as &LUA Table"));

        let context_menu_paste = context_menu.add_action_q_string(&QString::from_std_str("&Paste"));
        /*
        let context_menu_search = context_menu.add_action(&QString::from_std_str("&Search"));
        let context_menu_sidebar = context_menu.add_action(&QString::from_std_str("Si&debar"));
*/
        let context_menu_import_tsv = context_menu.add_action_q_string(&QString::from_std_str("&Import TSV"));
        let context_menu_export_tsv = context_menu.add_action_q_string(&QString::from_std_str("&Export TSV"));

        let context_menu_invert_selection = context_menu.add_action_q_string(&QString::from_std_str("Inver&t Selection"));
        let context_menu_reset_selection = context_menu.add_action_q_string(&QString::from_std_str("Reset &Selection"));

        let context_menu_undo = context_menu.add_action_q_string(&QString::from_std_str("&Undo"));
        let context_menu_redo = context_menu.add_action_q_string(&QString::from_std_str("&Redo"));

        // Insert some separators to space the menu, and the paste submenu.
        //context_menu.insert_separator(context_menu_search);
        //context_menu.insert_menu(context_menu_search, context_menu_apply_submenu.into_raw());
        //context_menu.insert_menu(context_menu_search, context_menu_clone_submenu.into_raw());
        context_menu.insert_menu(context_menu_paste, context_menu_copy_submenu.into_ptr());
        //context_menu.insert_menu(context_menu_search, context_menu_paste_submenu.into_raw());
        //context_menu.insert_separator(context_menu_search);
        //context_menu.insert_separator(context_menu_import);
        //context_menu.insert_separator(context_menu_sidebar);
        context_menu.insert_separator(context_menu_undo);


        // Create the raw Struct and begin
        let mut packed_file_table_view_raw = PackedFileTableViewRaw {
            table_view_primary: table_view,
            table_view_frozen: table_view_frozen.into_ptr(),
            table_filter: filter_model.into_ptr(),
            table_model: model.into_ptr(),
            table_enable_lookups_button: row_filter_case_sensitive_button.into_ptr(),
            filter_line_edit: row_filter_line_edit.into_ptr(),
            filter_case_sensitive_button: table_enable_lookups_button.into_ptr(),
            filter_column_selector: row_filter_column_selector.into_ptr(),

            context_menu,
            context_menu_enabler: context_menu_enabler.into_ptr(),
            context_menu_add_rows,
            context_menu_insert_rows,
            context_menu_delete_rows,
            context_menu_clone_and_append,
            context_menu_clone_and_insert,
            context_menu_copy,
            context_menu_copy_as_lua_table,
            context_menu_paste,
            context_menu_invert_selection,
            context_menu_reset_selection,
            context_menu_undo,
            context_menu_redo,
            context_menu_import_tsv,
            context_menu_export_tsv,
            smart_delete,

            dependency_data: Arc::new(RwLock::new(dependency_data)),
            table_definition: table_definition.clone(),

            undo_lock,
            save_lock,

            undo_model: QStandardItemModel::new_0a().into_ptr(),
            history_undo: Arc::new(RwLock::new(vec![])),
            history_redo: Arc::new(RwLock::new(vec![])),
        };

        let packed_file_table_view_slots = PackedFileTableViewSlots::new(
            &packed_file_table_view_raw,
            *global_search_ui,
            *pack_file_contents_ui,
            &packed_file_path,
            &table_definition,
        );

        let mut packed_file_table_view = Self {
            table_view_primary: atomic_from_mut_ptr(packed_file_table_view_raw.table_view_primary),
            table_view_frozen: atomic_from_mut_ptr(packed_file_table_view_raw.table_view_frozen),
            table_filter: atomic_from_mut_ptr(packed_file_table_view_raw.table_filter),
            table_model: atomic_from_mut_ptr(packed_file_table_view_raw.table_model),
            table_enable_lookups_button: atomic_from_mut_ptr(packed_file_table_view_raw.table_enable_lookups_button),
            filter_line_edit: atomic_from_mut_ptr(packed_file_table_view_raw.filter_line_edit),
            filter_case_sensitive_button: atomic_from_mut_ptr(packed_file_table_view_raw.filter_case_sensitive_button),
            filter_column_selector: atomic_from_mut_ptr(packed_file_table_view_raw.filter_column_selector),

            context_menu: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu),
            context_menu_enabler: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_enabler),
            context_menu_add_rows: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_add_rows),
            context_menu_insert_rows: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_insert_rows),
            context_menu_delete_rows: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_delete_rows),
            context_menu_clone_and_append: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_clone_and_append),
            context_menu_clone_and_insert: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_clone_and_insert),
            context_menu_copy: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_copy),
            context_menu_copy_as_lua_table: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_copy_as_lua_table),
            context_menu_paste: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_paste),
            context_menu_invert_selection: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_invert_selection),
            context_menu_reset_selection: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_reset_selection),
            context_menu_undo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_undo),
            context_menu_redo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_redo),
            context_menu_import_tsv: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_import_tsv),
            context_menu_export_tsv: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_export_tsv),
            smart_delete: atomic_from_mut_ptr(packed_file_table_view_raw.smart_delete),

            dependency_data: packed_file_table_view_raw.dependency_data.clone(),
            table_definition,
            table_name,

            undo_lock: packed_file_table_view_raw.undo_lock.clone(),
            save_lock: packed_file_table_view_raw.save_lock.clone(),

            undo_model: atomic_from_mut_ptr(packed_file_table_view_raw.undo_model),
            history_undo: packed_file_table_view_raw.history_undo.clone(),
            history_redo: packed_file_table_view_raw.history_redo.clone(),
        };

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be reseted to 1, 2, 3,... so we do this here.
        packed_file_table_view_raw.load_data(&table_data);

        // Initialize the undo model.
        update_undo_model(mut_ptr_from_atomic(&packed_file_table_view.table_model), mut_ptr_from_atomic(&packed_file_table_view.undo_model));

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        let packed_file_name = if packed_file_path.borrow().len() == 3 &&
            packed_file_path.borrow()[0].to_lowercase() == "db" {
            packed_file_path.borrow()[1].to_owned()
        } else { "".to_owned() };

        packed_file_table_view_raw.build_columns(&packed_file_name);

        // Set the connections and return success.
        connections::set_connections(&packed_file_table_view, &packed_file_table_view_slots);
        shortcuts::set_shortcuts(&mut packed_file_table_view);
        tips::set_tips(&mut packed_file_table_view);
        packed_file_view.view = View::Table(packed_file_table_view);
        packed_file_view.packed_file_type = packed_file_type;

        // Return success.
        Ok((TheOneSlot::Table(packed_file_table_view_slots), packed_file_info))
    }

    /// This function returns a reference to the StandardItemModel widget.
    pub fn get_mut_ptr_table_model(&self) -> MutPtr<QStandardItemModel> {
        mut_ptr_from_atomic(&self.table_model)
    }

    /// This function returns a mutable reference to the `Enable Lookups` Pushbutton.
    pub fn get_mut_ptr_enable_lookups_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.table_enable_lookups_button)
    }

    /// This function returns a pointer to the Primary TableView widget.
    pub fn get_mut_ptr_table_view_primary(&self) -> MutPtr<QTableView> {
        mut_ptr_from_atomic(&self.table_view_primary)
    }

    /// This function returns a pointer to the Frozen TableView widget.
    pub fn get_mut_ptr_table_view_frozen(&self) -> MutPtr<QTableView> {
        mut_ptr_from_atomic(&self.table_view_frozen)
    }

    /// This function returns a pointer to the filter's LineEdit widget.
    pub fn get_mut_ptr_filter_line_edit(&self) -> MutPtr<QLineEdit> {
        mut_ptr_from_atomic(&self.filter_line_edit)
    }

    /// This function returns a pointer to the add rows action.
    pub fn get_mut_ptr_context_menu_add_rows(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_add_rows)
    }

    /// This function returns a pointer to the insert rows action.
    pub fn get_mut_ptr_context_menu_insert_rows(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_insert_rows)
    }

    /// This function returns a pointer to the delete rows action.
    pub fn get_mut_ptr_context_menu_delete_rows(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_delete_rows)
    }

    /// This function returns a pointer to the clone_and_append action.
    pub fn get_mut_ptr_context_menu_clone_and_append(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_clone_and_append)
    }

    /// This function returns a pointer to the clone_and_insert action.
    pub fn get_mut_ptr_context_menu_clone_and_insert(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_clone_and_insert)
    }

    /// This function returns a pointer to the copy action.
    pub fn get_mut_ptr_context_menu_copy(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_copy)
    }

    /// This function returns a pointer to the copy as lua table action.
    pub fn get_mut_ptr_context_menu_copy_as_lua_table(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_copy_as_lua_table)
    }

    /// This function returns a pointer to the paste action.
    pub fn get_mut_ptr_context_menu_paste(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_paste)
    }

    /// This function returns a pointer to the invert selection action.
    pub fn get_mut_ptr_context_menu_invert_selection(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_invert_selection)
    }

    /// This function returns a pointer to the reset selection action.
    pub fn get_mut_ptr_context_menu_reset_selection(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_reset_selection)
    }

    /// This function returns a pointer to the undo action.
    pub fn get_mut_ptr_context_menu_undo(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_undo)
    }

    /// This function returns a pointer to the redo action.
    pub fn get_mut_ptr_context_menu_redo(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_redo)
    }

    /// This function returns a pointer to the import TSV action.
    pub fn get_mut_ptr_context_menu_import_tsv(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_import_tsv)
    }

    /// This function returns a pointer to the export TSV action.
    pub fn get_mut_ptr_context_menu_export_tsv(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_export_tsv)
    }

    /// This function returns a pointer to the smart delete action.
    pub fn get_mut_ptr_smart_delete(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.smart_delete)
    }

    /// This function returns a reference to this table's name.
    pub fn get_ref_table_name(&self) -> &str {
        &self.table_name
    }

    /// This function returns a reference to the definition of this table.
    pub fn get_ref_table_definition(&self) -> &Definition {
        &self.table_definition
    }
}

//----------------------------------------------------------------//
// Implementations of `TableOperation`.
//----------------------------------------------------------------//

/// Debug implementation of TableOperations, so we can at least guess what is in the history.
impl Debug for TableOperations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Editing(data) => write!(f, "Cell/s edited, starting in row {}, column {}.", (data[0].0).0, (data[0].0).1),
            Self::AddRows(data) => write!(f, "Removing row/s added in position/s {}.", data.iter().map(|x| format!("{}, ", x)).collect::<String>()),
            Self::RemoveRows(data) => write!(f, "Re-adding row/s removed in {} batches.", data.len()),
            Self::ImportTSV(_) => write!(f, "Imported TSV file."),
            Self::Carolina(_) => write!(f, "Carolina, trátame bien, no te rías de mi, no me arranques la piel."),
        }
    }
}

/// CLone implementation for TableOperations.
///
/// NOTE: CAROLINA'S CLONE IS NOT IMPLEMENTED. It'll crash if you try to clone it.
impl Clone for TableOperations {
    fn clone(&self) -> Self {
        match self {
            Self::Editing(items) => Self::Editing(items.iter().map(|(x, y)| (*x, atomic_from_mut_ptr(mut_ptr_from_atomic(y)))).collect()),
            Self::AddRows(rows) => Self::AddRows(rows.to_vec()),
            Self::RemoveRows(rows) => Self::RemoveRows(rows.iter()
                .map(|(x, y)| (*x, y.iter()
                    .map(|y| y.iter()
                        .map(|z| atomic_from_mut_ptr(mut_ptr_from_atomic(z)))
                        .collect()
                    ).collect()
                )).collect()),
            _ => unimplemented!()
        }
    }
}
