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

use qt_core::{CaseSensitivity, CheckState};
use qt_core::QFlags;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QStringList;
use qt_core::QVariant;
use qt_core::QString;
use qt_core::q_item_selection_model::SelectionFlag;

use cpp_core::CppBox;
use cpp_core::MutPtr;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

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
use self::utils::*;

mod connections;
pub mod slots;
mod utils;

// Column default sizes.
static COLUMN_SIZE_BOOLEAN: i32 = 100;
static COLUMN_SIZE_NUMBER: i32 = 140;
static COLUMN_SIZE_STRING: i32 = 350;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum is used to distinguish between the different types of tables we can decode.
#[derive(Clone)]
pub enum TableType {
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    Loc(Loc),
}

/// Enum to know what operation was done while editing tables, so we can revert them with undo.
pub enum TableOperations {

    /// Intended for any kind of item editing. Holds a Vec<((row, column), AtomicPtr<item>)>, so we can do this in batches.
    Editing(Vec<((i32, i32), AtomicPtr<QStandardItem>)>),

    // Intended for when adding/inserting rows. It holds a list of positions where the rows where inserted.
    //AddRows(Vec<i32>),

    // Intended for when removing rows. It holds a list of positions where the rows where deleted and the deleted rows data, in consecutive batches.
    //RemoveRows((Vec<Vec<(i32, Vec<*mut StandardItem>)>>)),

    // Intended for when we are using the smart delete feature. This is a combination of list of edits and list of removed rows.
    //SmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<Vec<(i32, Vec<*mut StandardItem>)>>)),

    // RevertSmartDelete: Selfexplanatory. This is a combination of list of edits and list of adding rows.
    //RevertSmartDelete((Vec<((i32, i32), *mut StandardItem)>, Vec<i32>)),

    // It holds a copy of the entire table, before importing.
    //ImportTSV(Vec<Vec<DecodedData>>),

    // A Jack-of-all-Trades. It holds a Vec<TableOperations>, for those situations one is not enough.
    //Carolina(Vec<TableOperations>),
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
    context_menu_invert_selection: AtomicPtr<QAction>,
    context_menu_undo: AtomicPtr<QAction>,
    context_menu_redo: AtomicPtr<QAction>,

    dependency_data: Arc<BTreeMap<i32, Vec<(String, String)>>>,
    table_name: String,
    table_definition: Definition,

    save_lock: Arc<AtomicBool>,
    undo_lock: Arc<AtomicBool>,

    undo_model: AtomicPtr<QStandardItemModel>,
    history_undo: Arc<RwLock<Vec<TableOperations>>>,
    history_redo: Arc<RwLock<Vec<TableOperations>>>,
}

/// This struct contains the raw version of each pointer in `PackedFileTableView`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileTableView`.
#[derive(Clone)]
pub struct PackedFileTableViewRaw {
    pub table_view_primary: MutPtr<QTableView>,
    pub table_view_frozen: MutPtr<QTableView>,
    pub table_filter: MutPtr<QSortFilterProxyModel>,
    pub table_model: MutPtr<QStandardItemModel>,
    pub table_enable_lookups_button: MutPtr<QPushButton>,
    pub filter_case_sensitive_button: MutPtr<QPushButton>,
    pub filter_column_selector: MutPtr<QComboBox>,
    pub filter_line_edit: MutPtr<QLineEdit>,

    pub context_menu: MutPtr<QMenu>,
    pub context_menu_enabler: MutPtr<QAction>,
    pub context_menu_invert_selection: MutPtr<QAction>,
    pub context_menu_undo: MutPtr<QAction>,
    pub context_menu_redo: MutPtr<QAction>,

    pub save_lock: Arc<AtomicBool>,
    pub undo_lock: Arc<AtomicBool>,

    pub undo_model: MutPtr<QStandardItemModel>,
    pub history_undo: Arc<RwLock<Vec<TableOperations>>>,
    pub history_redo: Arc<RwLock<Vec<TableOperations>>>,
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
            let name = Self::clean_column_names(&field.name);
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
        //let smart_delete = Action::new(()).into_raw();

        // Create the Contextual Menu for the TableView.
        let context_menu_enabler = QAction::new();
        let mut context_menu = QMenu::new().into_ptr();
        /*let context_menu_add = context_menu.add_action(&QString::from_std_str("&Add Row"));
        let context_menu_insert = context_menu.add_action(&QString::from_std_str("&Insert Row"));
        let context_menu_delete = context_menu.add_action(&QString::from_std_str("&Delete Row"));

        let mut context_menu_apply_submenu = Menu::new(&QString::from_std_str("A&pply..."));
        let context_menu_apply_maths_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Maths to Selection"));
        let context_menu_rewrite_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Rewrite Selection"));

        let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));
        let context_menu_clone = context_menu_clone_submenu.add_action(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu_clone_submenu.add_action(&QString::from_std_str("Clone and &Append"));

        let mut context_menu_copy_submenu = Menu::new(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action(&QString::from_std_str("&Copy as &LUA Table"));

        let mut context_menu_paste_submenu = Menu::new(&QString::from_std_str("&Paste..."));
        let context_menu_paste = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste"));
        let context_menu_paste_as_new_lines = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste as New Rows"));
        let context_menu_paste_to_fill_selection = context_menu_paste_submenu.add_action(&QString::from_std_str("&Paste to Fill Selection"));

        let context_menu_search = context_menu.add_action(&QString::from_std_str("&Search"));
        let context_menu_sidebar = context_menu.add_action(&QString::from_std_str("Si&debar"));

        let context_menu_import = context_menu.add_action(&QString::from_std_str("&Import"));
        let context_menu_export = context_menu.add_action(&QString::from_std_str("&Export"));
*/
        let context_menu_invert_selection = context_menu.add_action_q_string(&QString::from_std_str("Inver&t Selection"));

        let context_menu_undo = context_menu.add_action_q_string(&QString::from_std_str("&Undo"));
        let context_menu_redo = context_menu.add_action_q_string(&QString::from_std_str("&Redo"));

        // Create the raw Struct and begin
        let packed_file_table_view_raw = PackedFileTableViewRaw {
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
            context_menu_invert_selection,
            context_menu_undo,
            context_menu_redo,

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
            &dependency_data
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
            context_menu_invert_selection: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_invert_selection),
            context_menu_undo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_undo),
            context_menu_redo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_redo),

            dependency_data: Arc::new(dependency_data),
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
        packed_file_table_view.load_data(&table_data);

        // Initialize the undo model.
        update_undo_model(mut_ptr_from_atomic(&packed_file_table_view.table_model), mut_ptr_from_atomic(&packed_file_table_view.undo_model));

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        let packed_file_name = if packed_file_path.borrow().len() == 3 &&
            packed_file_path.borrow()[0].to_lowercase() == "db" {
            packed_file_path.borrow()[1].to_owned()
        } else { "".to_owned() };

        packed_file_table_view.build_columns(&packed_file_name);

        // Set the connections and return success.
        connections::set_connections(&packed_file_table_view, &packed_file_table_view_slots);
        packed_file_view.view = View::Table(packed_file_table_view);
        packed_file_view.packed_file_type = packed_file_type;

        // Return success.
        Ok((TheOneSlot::Table(packed_file_table_view_slots), packed_file_info))
    }

    /// This function loads the data from a compatible `PackedFile` into a TableView.
    pub unsafe fn load_data(
        &mut self,
        data: &TableType,
    ) {
        // First, we delete all the data from the `ListStore`. Just in case there is something there.
        // This wipes out header information, so remember to run "build_columns" after this.
        let mut table_model = mut_ptr_from_atomic(&self.table_model);
        table_model.clear();

        // Set the right data, depending on the table type you get.
        let data = match data {
            TableType::DependencyManager(data) => &data,
            TableType::DB(data) => &*data.get_ref_table_data(),
            TableType::Loc(data) => data.get_ref_table_data(),
        };

        // Load the data, row by row.
        for entry in data {
            let mut qlist = QListOfQStandardItem::new();
            for (index, field) in entry.iter().enumerate() {
                let item = Self::get_item_from_decoded_data(field);

                // If we have the dependency stuff enabled, check if it's a valid reference.
                if SETTINGS.lock().unwrap().settings_bool["use_dependency_checker"] && self.table_definition.fields[index].is_reference.is_some() {
                    //Self::check_references(dependency_data, index as i32, item.into_ptr());
                }

                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
            table_model.append_row_q_list_of_q_standard_item(&qlist);
        }

        // If the table it's empty, we add an empty row and delete it, so the "columns" get created.
        if data.is_empty() {
            let mut qlist = QListOfQStandardItem::new();
            for field in &self.table_definition.fields {
                let item = Self::get_default_item_from_field(field);
                add_to_q_list_safe(qlist.as_mut_ptr(), item.into_ptr());
            }
            table_model.append_row_q_list_of_q_standard_item(&qlist);
            table_model.remove_rows_2a(0, 1);
        }

        // Here we assing the ItemDelegates, so each type has his own widget with validation included.
        // LongInteger uses normal string controls due to QSpinBox being limited to i32.
        let enable_lookups = self.get_mut_ptr_enable_lookups_button().is_checked();
        for (column, field) in self.table_definition.fields.iter().enumerate() {

            // Combos are a bit special, as they may or may not replace other delegates. If we disable them, use the normal delegates.
            if SETTINGS.lock().unwrap().settings_bool["disable_combos_on_tables"] {
                match field.field_type {
                    FieldType::Boolean => {},
                    FieldType::Float => {
                        new_doublespinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32);
                        new_doublespinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32);
                    },
                    FieldType::Integer => {
                        new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, 32);
                        new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, 32);
                    },
                    FieldType::LongInteger => {
                        new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, 64);
                        new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, 64);
                    },
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => {
                        new_qstring_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, field.max_length);
                        new_qstring_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, field.max_length);
                    },
                    FieldType::Sequence(_) => {}
                }
            }

            // Otherwise, we have to check first if the column has references. If it does, replace the delegate with a combo.
            else {

                if let Some(data) = self.dependency_data.get(&(column as i32)) {
                    let mut list = QStringList::new();
                    data.iter().map(|x| if enable_lookups { &x.1 } else { &x.0 }).for_each(|x| list.append_q_string(&QString::from_std_str(x)));
                    new_combobox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, list.as_ptr(), true, field.max_length);
                    new_combobox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, list.as_ptr(), true, field.max_length);
                }
                else {
                    match field.field_type {
                        FieldType::Boolean => {},
                        FieldType::Float => {
                            new_doublespinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32);
                            new_doublespinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32);
                        },
                        FieldType::Integer => {
                            new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, 32);
                            new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, 32);
                        },
                        FieldType::LongInteger => {
                            new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, 64);
                            new_spinbox_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, 64);
                        },
                        FieldType::StringU8 |
                        FieldType::StringU16 |
                        FieldType::OptionalStringU8 |
                        FieldType::OptionalStringU16 => {
                            new_qstring_item_delegate_safe(&mut self.get_mut_ptr_table_view_primary(), column as i32, field.max_length);
                            new_qstring_item_delegate_safe(&mut self.get_mut_ptr_table_view_frozen(), column as i32, field.max_length);
                        },
                        FieldType::Sequence(_) => {}
                    }
                }
            }
        }
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

    /// This function returns a pointer to the invert selection action.
    pub fn get_mut_ptr_context_menu_invert_selection(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_invert_selection)
    }

    /// This function returns a pointer to the undo action.
    pub fn get_mut_ptr_context_menu_undo(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_undo)
    }

    /// This function returns a pointer to the redo action.
    pub fn get_mut_ptr_context_menu_redo(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_redo)
    }

    /// This function returns a reference to this table's name.
    pub fn get_ref_table_name(&self) -> &str {
        &self.table_name
    }

    /// This function returns a reference to the definition of this table.
    pub fn get_ref_table_definition(&self) -> &Definition {
        &self.table_definition
    }

    /// This function "process" the column names of a table, so they look like they should.
    fn clean_column_names(field_name: &str) -> String {
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

    /// This function is meant to be used to prepare and build the column headers, and the column-related stuff.
    /// His intended use is for just after we load/reload the data to the table.
    unsafe fn build_columns(
        &mut self,
        table_name: &str,
    ) {
        let mut table_view_primary = self.get_mut_ptr_table_view_primary();
        let table_view_frozen = self.get_mut_ptr_table_view_frozen();
        let filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
        let mut model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
        let schema = SCHEMA.read().unwrap();
        let mut do_we_have_ca_order = false;
        let mut keys = vec![];

        // For each column, clean their name and set their width and tooltip.
        for (index, field) in self.table_definition.fields.iter().enumerate() {

            let name = Self::clean_column_names(&field.name);
            let mut item = QStandardItem::from_q_string(&QString::from_std_str(&name));
            Self::set_tooltip(&schema, &field, table_name, &mut item);
            model.set_horizontal_header_item(index as i32, item.into_ptr());

            // Depending on his type, set one width or another.
            match field.field_type {
                FieldType::Boolean => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_BOOLEAN),
                FieldType::Float => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::Integer => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::LongInteger => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_NUMBER),
                FieldType::StringU8 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::StringU16 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::OptionalStringU8 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::OptionalStringU16 => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
                FieldType::Sequence(_) => table_view_primary.set_column_width(index as i32, COLUMN_SIZE_STRING),
            }


            // If the field is key, add that column to the "Key" list, so we can move them at the beginning later.
            if field.is_key { keys.push(index); }
            if field.ca_order != -1 { do_we_have_ca_order |= true; }
        }

        // Now the order. If we have a sort order from the schema, we use that one.
        if do_we_have_ca_order {
            let mut fields = self.table_definition.fields.iter().enumerate().map(|(x, y)| (x, y.clone())).collect::<Vec<(usize, Field)>>();
            fields.sort_by(|a, b| a.1.ca_order.cmp(&b.1.ca_order));

            let mut header_primary = table_view_primary.horizontal_header();
            let mut header_frozen = table_view_frozen.horizontal_header();
            for (logical_index, field) in &fields {
                if field.ca_order != -1 {
                    let visual_index = header_primary.visual_index(*logical_index as i32);
                    header_primary.move_section(visual_index as i32, field.ca_order as i32);
                    header_frozen.move_section(visual_index as i32, field.ca_order as i32);
                }
            }
        }

        // Otherwise, if we have any "Key" field, move it to the beginning.
        else if !keys.is_empty() {
            let mut header_primary = table_view_primary.horizontal_header();
            let mut header_frozen = table_view_frozen.horizontal_header();
            for (position, column) in keys.iter().enumerate() {
                header_primary.move_section(*column as i32, position as i32);
                header_frozen.move_section(*column as i32, position as i32);
            }
        }
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

    /// This function generates a StandardItem for the provided DecodedData.
    unsafe fn get_item_from_decoded_data(data: &DecodedData) -> CppBox<QStandardItem> {
        match *data {

            // This one needs a couple of changes before turning it into an item in the table.
            DecodedData::Boolean(ref data) => {
                let mut item = QStandardItem::new();
                item.set_editable(false);
                item.set_checkable(true);
                item.set_check_state(if *data { CheckState::Checked } else { CheckState::Unchecked });
                item
            }

            // Floats need to be tweaked to fix trailing zeroes and precission issues, like turning 0.5000004 into 0.5.
            // Also, they should be limited to 3 decimals.
            DecodedData::Float(ref data) => {
                let data = {
                    let data_str = format!("{}", data);
                    if let Some(position) = data_str.find('.') {
                        let decimals = &data_str[position..].len();
                        if *decimals > 3 { format!("{:.3}", data).parse::<f32>().unwrap() }
                        else { *data }
                    }
                    else { *data }
                };

                let mut item = QStandardItem::new();
                item.set_data_2a(&QVariant::from_float(data), 2);
                item
            },
            DecodedData::Integer(ref data) => {
                let mut item = QStandardItem::new();
                item.set_data_2a(&QVariant::from_int(*data), 2);
                item
            },
            DecodedData::LongInteger(ref data) => {
                let mut item = QStandardItem::new();
                item.set_data_2a(&QVariant::from_i64(*data), 2);
                item
            },
            // All these are Strings, so it can be together,
            DecodedData::StringU8(ref data) |
            DecodedData::StringU16(ref data) |
            DecodedData::OptionalStringU8(ref data) |
            DecodedData::OptionalStringU16(ref data) => QStandardItem::from_q_string(&QString::from_std_str(data)),
            DecodedData::Sequence(_) => QStandardItem::from_q_string(&qtr("packedfile_noneditable_sequence")),
        }
    }

    /// This function sets the tooltip for the provided column header, if the column should have one.
    unsafe fn set_tooltip(schema: &Option<Schema>, field: &Field, table_name: &str, item: &mut QStandardItem) {

        // If we passed it a table name, build the tooltip based on it. The logic is simple:
        // - If we have a description, we add it to the tooltip.
        // - If the column references another column, we add it to the tooltip.
        // - If the column is referenced by another column, we add it to the tooltip.
        if !table_name.is_empty() {
            let mut tooltip_text = String::new();
            if !field.description.is_empty() {
                tooltip_text.push_str(&format!("<p>{}</p>", field.description));
            }

            if let Some(ref reference) = field.is_reference {
                tooltip_text.push_str(&format!("<p>This column is a reference to:</p><p><i>\"{}/{}\"</i></p>", reference.0, reference.1));
            }

            else {
                let mut referenced_columns = if let Some(ref schema) = schema {
                    let short_table_name = table_name.split_at(table_name.len() - 7).0;
                    let mut columns = vec![];

                    // We get all the db definitions from the schema, then iterate all of them to find what tables reference our own.
                    for versioned_file in schema.get_ref_versioned_file_db_all() {
                        if let VersionedFile::DB(ref_table_name, ref_definition) = versioned_file {
                            let mut found = false;
                            for ref_version in ref_definition {
                                for ref_field in &ref_version.fields {
                                    if let Some((ref_ref_table, ref_ref_field)) = &ref_field.is_reference {
                                        if ref_ref_table == short_table_name && ref_ref_field == &field.name {
                                            found = true;
                                            columns.push((ref_table_name.to_owned(), ref_field.name.to_owned()));
                                        }
                                    }
                                }
                                if found { break; }
                            }
                        }
                    }
                    columns
                } else { vec![] };

                referenced_columns.sort_unstable();
                if !referenced_columns.is_empty() {
                    tooltip_text.push_str("<p>Fields that reference this column:</p>");
                    for (index, reference) in referenced_columns.iter().enumerate() {
                        tooltip_text.push_str(&format!("<i>\"{}/{}\"</i><br>", reference.0, reference.1));

                        // There is a bug that causes tooltips to be displayed out of screen if they're too big. This fixes it.
                        if index == 50 {
                            tooltip_text.push_str(&format!("<p>And many more. Exactly, {} more. Too many to show them here.</p>nnnn", referenced_columns.len() as isize - 50));
                            break ;
                        }
                    }

                    // Dirty trick to remove the last <br> from the tooltip, or the nnnn in case that text get used.
                    tooltip_text.pop();
                    tooltip_text.pop();
                    tooltip_text.pop();
                    tooltip_text.pop();
                }
            }

            // We only add the tooltip if we got something to put into it.
            if !tooltip_text.is_empty() {
                item.set_tool_tip(&QString::from_std_str(&tooltip_text));
            }
        }
    }
}

/// Implementation of `PackedFileTableViewRaw`.
impl PackedFileTableViewRaw {

    unsafe fn context_menu_update(&mut self, table_definition: &Definition) {

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselfs.
        let indexes = self.table_filter.map_selection_to_source(&self.table_view_primary.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            //context_menu_clone.set_enabled(true);
            //context_menu_clone_and_append.set_enabled(true);
            //context_menu_copy.set_enabled(true);
            //context_menu_delete.set_enabled(true);
            //context_menu_rewrite_selection.set_enabled(true);
/*
            // The "Apply" actions have to be enabled only when all the indexes are valid for the operation.
            let mut columns = vec![];
            for index in 0..indexes.count_0a() {
                let model_index = indexes.at(index);
                if model_index.is_valid() { columns.push(model_index.column()); }
            }

            columns.sort();
            columns.dedup();

            let mut can_apply = true;
            for column in &columns {
                let field_type = &table_definition.fields[*column as usize].field_type;
                if *field_type != FieldType::Boolean { continue }
                else { can_apply = false; break }
            }*/
            //context_menu_apply_maths_to_selection.set_enabled(can_apply);
        }

        // Otherwise, disable them.
        else {
            //context_menu_apply_maths_to_selection.set_enabled(false);
            //context_menu_rewrite_selection.set_enabled(false);
            //context_menu_clone.set_enabled(false);
            //context_menu_clone_and_append.set_enabled(false);
            //context_menu_copy.set_enabled(false);
            //context_menu_delete.set_enabled(false);
        }

        if !self.undo_lock.load(Ordering::SeqCst) {
            self.context_menu_undo.set_enabled(!self.history_undo.read().unwrap().is_empty());
            self.context_menu_redo.set_enabled(!self.history_redo.read().unwrap().is_empty());
        }

    }

    /// Function to filter the table. If a value is not provided by a slot, we get it from the widget itself.
    unsafe fn filter_table(&mut self) {

        let mut pattern = QRegExp::new_1a(&self.filter_line_edit.text());
        self.table_filter.set_filter_key_column(self.filter_column_selector.current_index());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        self.table_filter.set_filter_reg_exp_q_reg_exp(&pattern);
    }

    /// This function enables/disables showing the lookup values instead of the real ones in the columns that support it.
    unsafe fn toggle_lookups(&self, _table_definition: &Definition, _dependency_data: &BTreeMap<i32, Vec<(String, String)>>) {
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

    /// This function returns a pointer to the Primary TableView widget.
    pub fn get_table_view_primary(&self) -> MutPtr<QTableView> {
        self.table_view_primary
    }

    /// Function to undo/redo an operation in the table.
    ///
    /// If undo = true we are undoing. Otherwise we are redoing.
    unsafe fn undo_redo(&mut self, undo: bool) {
        let table_view_primary = self.get_table_view_primary();
        let filter: MutPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast_mut();
        let mut model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();

        let (mut history_source, mut history_opposite) = if undo {
            (self.history_undo.write().unwrap(), self.history_redo.write().unwrap())
        } else {
            (self.history_redo.write().unwrap(), self.history_undo.write().unwrap())
        };

        // Get the last operation in the Undo History, or return if there is none.
        let operation = if let Some(operation) = history_source.pop() { operation } else { return };
        match operation {
            TableOperations::Editing(editions) => {

                // Prepare the redo operation, then do the rest.
                let mut redo_editions = vec![];
                editions.iter().for_each(|x| redo_editions.push((((x.0).0, (x.0).1), atomic_from_mut_ptr((&*model.item_2a((x.0).0, (x.0).1)).clone()))));
                history_opposite.push(TableOperations::Editing(redo_editions));

                self.undo_lock.store(true, Ordering::SeqCst);
                self.save_lock.store(true, Ordering::SeqCst);
                for (index, ((row, column), item)) in editions.iter().enumerate() {
                    let item = &*mut_ptr_from_atomic(&item);
                    model.set_item_3a(*row, *column, item.clone());

                    // If we are going to process the last one, unlock the save.
                    if index == editions.len() - 1 {
                        self.save_lock.store(false, Ordering::SeqCst);
                        model.item_2a(*row, *column).set_data_2a(&QVariant::from_int(1i32), 16);
                        model.item_2a(*row, *column).set_data_2a(&QVariant::new(), 16);
                    }
                }

                // Select all the edited items.
                let mut selection_model = table_view_primary.selection_model();
                selection_model.clear();
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
/*
            // This action is special and we have to manually trigger a save for it.
            // This actions if for undoing "add rows" actions. It deletes the stored rows.
            // NOTE: the rows list must ALWAYS be in 9->1 order. Otherwise this breaks.
            TableOperations::AddRows(rows) => {

                // Split the row list in consecutive rows, get their data, and remove them in batches.
                let mut rows_splitted = vec![];
                let mut current_row_pack = vec![];
                let mut current_row_index = -2;
                for (index, row) in rows.iter().enumerate() {

                    let mut items = vec![];
                    for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                        let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                        items.push(item.clone());
                    }

                    if (*row == current_row_index - 1) || index == 0 {
                        current_row_pack.push((*row, items));
                        current_row_index = *row;
                    }
                    else {
                        current_row_pack.reverse();
                        rows_splitted.push(current_row_pack.to_vec());
                        current_row_pack.clear();
                        current_row_pack.push((*row, items));
                        current_row_index = *row;
                    }
                }
                current_row_pack.reverse();
                rows_splitted.push(current_row_pack);

                for row_pack in rows_splitted.iter() {
                    unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                }

                rows_splitted.reverse();
                history_opposite.push(TableOperations::RemoveRows(rows_splitted));

                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }

            // NOTE: the rows list must ALWAYS be in 1->9 order. Otherwise this breaks.
            TableOperations::RemoveRows(rows) => {

                // First, we re-insert the pack of empty rows. Then, we put the data into them. And repeat with every Pack.
                for row_pack in &rows {
                    for (row, items) in row_pack {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        unsafe { items.iter().for_each(|x| qlist.append_unsafe(x)); }
                        unsafe { model.as_mut().unwrap().insert_row((*row, &qlist)); }
                    }
                }

                // Create the "redo" action for this one.
                let mut rows_to_add = vec![];
                rows.to_vec().iter_mut().map(|x| x.iter_mut().map(|y| y.0).collect::<Vec<i32>>()).for_each(|mut x| rows_to_add.append(&mut x));
                rows_to_add.reverse();
                history_opposite.push(TableOperations::AddRows(rows_to_add));

                // Select all the re-inserted rows that are in the filter. We need to block signals here because the bigger this gets, the slower it gets. And it gets very slow.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row_pack in &rows {
                    let initial_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack[0].0, 0))) };
                    let final_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack.last().unwrap().0 as i32, 0))) };
                    if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                        let selection = ItemSelection::new((&initial_model_index_filtered, &final_model_index_filtered));
                        unsafe { selection_model.as_mut().unwrap().select((&selection, Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows))); }
                    }
                }

                // Trick to tell the model to update everything.
                *undo_lock.borrow_mut() = true;
                unsafe { model.as_mut().unwrap().item((0, 0)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                *undo_lock.borrow_mut() = false;
            }

            // "rows" has to come in the same format than in RemoveRows.
            TableOperations::SmartDelete((edits, rows)) => {

                // First, we re-insert each pack of rows.
                for row_pack in &rows {
                    for (row, items) in row_pack {
                        let mut qlist = ListStandardItemMutPtr::new(());
                        unsafe { items.iter().for_each(|x| qlist.append_unsafe(x)); }
                        unsafe { model.as_mut().unwrap().insert_row((*row, &qlist)); }
                    }
                }

                // Then, restore all the edits and keep their old state for the undo/redo action.
                *undo_lock.borrow_mut() = true;
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                *undo_lock.borrow_mut() = false;

                // Next, prepare the redo operation.
                let mut rows_to_add = vec![];
                rows.to_vec().iter_mut().map(|x| x.iter_mut().map(|y| y.0).collect::<Vec<i32>>()).for_each(|mut x| rows_to_add.append(&mut x));
                rows_to_add.reverse();
                history_opposite.push(TableOperations::RevertSmartDelete((edits_before, rows_to_add)));

                // Select all the edited items/restored rows.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for row_pack in &rows {
                    let initial_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack[0].0, 0))) };
                    let final_model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index((row_pack.last().unwrap().0 as i32, 0))) };
                    if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                        let selection = ItemSelection::new((&initial_model_index_filtered, &final_model_index_filtered));
                        unsafe { selection_model.as_mut().unwrap().select((&selection, Flags::from_enum(SelectionFlag::Select) | Flags::from_enum(SelectionFlag::Rows))); }
                    }
                }

                for edit in edits.iter() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index(((edit.0).0, (edit.0).1))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select)
                        )); }
                    }
                }

                // Trick to tell the model to update everything.
                *undo_lock.borrow_mut() = true;
                unsafe { model.as_mut().unwrap().item((0, 0)).as_mut().unwrap().set_data((&Variant::new0(()), 16)); }
                *undo_lock.borrow_mut() = false;
            }

            // This action is special and we have to manually trigger a save for it.
            // "rows" has to come in the same format than in AddRows.
            TableOperations::RevertSmartDelete((edits, rows)) => {

                // First, redo all the "edits".
                *undo_lock.borrow_mut() = true;
                let edits_before = unsafe { edits.iter().map(|x| (((x.0).0, (x.0).1), (&*model.as_mut().unwrap().item(((x.0).0, (x.0).1))).clone())).collect::<Vec<((i32, i32), *mut StandardItem)>>() };
                unsafe { edits.iter().for_each(|x| model.as_mut().unwrap().set_item(((x.0).0, (x.0).1, x.1.clone()))); }
                *undo_lock.borrow_mut() = false;

                // Select all the edited items, if any, before removing rows. Otherwise, the selection will not match the editions.
                let selection_model = unsafe { table_view.as_mut().unwrap().selection_model() };
                unsafe { selection_model.as_mut().unwrap().clear(); }
                for edit in edits.iter() {
                    let model_index_filtered = unsafe { filter_model.as_ref().unwrap().map_from_source(&model.as_mut().unwrap().index(((edit.0).0, (edit.0).1))) };
                    if model_index_filtered.is_valid() {
                        unsafe { selection_model.as_mut().unwrap().select((
                            &model_index_filtered,
                            Flags::from_enum(SelectionFlag::Select)
                        )); }
                    }
                }

                // Then, remove the restored tables after undoing a "SmartDelete".
                // Same thing as with "AddRows": split the row list in consecutive rows, get their data, and remove them in batches.
                let mut rows_splitted = vec![];
                let mut current_row_pack = vec![];
                let mut current_row_index = -2;
                for (index, row) in rows.iter().enumerate() {

                    let mut items = vec![];
                    for column in 0..unsafe { model.as_mut().unwrap().column_count(()) } {
                        let item = unsafe { &*model.as_mut().unwrap().item((*row, column)) };
                        items.push(item.clone());
                    }

                    if (*row == current_row_index - 1) || index == 0 {
                        current_row_pack.push((*row, items));
                        current_row_index = *row;
                    }
                    else {
                        current_row_pack.reverse();
                        rows_splitted.push(current_row_pack.to_vec());
                        current_row_pack.clear();
                        current_row_pack.push((*row, items));
                        current_row_index = *row;
                    }
                }
                current_row_pack.reverse();
                rows_splitted.push(current_row_pack);
                if rows_splitted[0].is_empty() { rows_splitted.clear(); }

                for row_pack in rows_splitted.iter() {
                    unsafe { model.as_mut().unwrap().remove_rows((row_pack[0].0, row_pack.len() as i32)); }
                }

                // Prepare the redo operation.
                rows_splitted.reverse();
                history_opposite.push(TableOperations::SmartDelete((edits_before, rows_splitted)));

                // Try to save the PackedFile to the main PackFile.
                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }

            // This action is special and we have to manually trigger a save for it.
            TableOperations::ImportTSV(table_data) => {

                // Prepare the redo operation.
                {
                    let table_type = &mut *table_type.borrow_mut();
                    match table_type {
                        TableType::DependencyManager(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.to_vec()));
                            *data = table_data;
                        },
                        TableType::DB(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.entries.to_vec()));
                            data.entries = table_data;
                        },
                        TableType::LOC(data) => {
                            history_opposite.push(TableOperations::ImportTSV(data.entries.to_vec()));
                            data.entries = table_data;
                        },
                    }
                }

                Self::load_data_to_table_view(table_view, model, &table_type.borrow(), table_definition, &dependency_data);
                Self::build_columns(table_view, table_view_frozen, model, table_definition, enable_header_popups);

                // If we want to let the columns resize themselfs...
                if SETTINGS.lock().unwrap().settings_bool["adjust_columns_to_content"] {
                    unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                }

                // Try to save the PackedFile to the main PackFile.
                Self::save_to_packed_file(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packed_file_path,
                    model,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    table_definition,
                    &mut table_type.borrow_mut(),
                );
            }
            TableOperations::Carolina(operations) => {
                for operation in &operations {
                    history_source.push((*operation).clone());
                    Self::undo_redo(
                        &app_ui,
                        &dependency_data,
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &packed_file_path,
                        table_view,
                        table_view_frozen,
                        model,
                        filter_model,
                        history_source,
                        history_opposite,
                        &global_search_explicit_paths,
                        update_global_search_stuff,
                        &undo_lock,
                        &save_lock,
                        &table_definition,
                        &table_type,
                        enable_header_popups.clone()
                    );
                }
                let len = history_opposite.len();
                let mut edits = history_opposite.drain((len - operations.len())..).collect::<Vec<TableOperations>>();
                edits.reverse();
                history_opposite.push(TableOperations::Carolina(edits));
            }*/
        }
    }
}
