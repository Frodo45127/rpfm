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

use qt_widgets::QCheckBox;
use qt_widgets::QAction;
use qt_widgets::QComboBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QTableView;
use qt_widgets::QMenu;
use qt_widgets::QWidget;
use qt_widgets::QScrollArea;
use qt_widgets::QLabel;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QModelIndex;
use qt_core::CheckState;
use qt_core::QFlags;
use qt_core::AlignmentFlag;
use qt_core::QSortFilterProxyModel;
use qt_core::QStringList;
use qt_core::QVariant;
use qt_core::QString;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::MatchFlag;

use cpp_core::CppBox;
use cpp_core::MutPtr;

use std::cell::RefCell;
use std::{fmt, fmt::Debug};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicPtr};

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::common::parse_str;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::table::{DecodedData, db::DB, loc::Loc};
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::{Definition, Field, FieldType, Schema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View, ViewType};
use crate::utils::{atomic_from_mut_ptr, mut_ptr_from_atomic};
use crate::utils::create_grid_layout;
use crate::utils::show_dialog;

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

/// This struct contains all the stuff needed to perform a table search. There is one per table, integrated in the view.
#[derive(Clone)]
pub struct TableSearch {
    pattern: MutPtr<QString>,
    replace: MutPtr<QString>,
    regex: bool,
    case_sensitive: bool,
    column: Option<i32>,

    /// This one contains the QModelIndex of the model and the QModelIndex of the filter, if exists.
    matches: Vec<(MutPtr<QModelIndex>, Option<MutPtr<QModelIndex>>)>,
    current_item: Option<u64>,
}

/// This enum defines the operation to be done when updating something related to the TableSearch.
pub enum TableSearchUpdate {
    Update,
    Search,
    PrevMatch,
    NextMatch,
}

/// This struct contains pointers to all the widgets in a Table View.
pub struct PackedFileTableView {
    table_view_primary: AtomicPtr<QTableView>,
    table_view_frozen: AtomicPtr<QTableView>,
    table_model: AtomicPtr<QStandardItemModel>,
    table_enable_lookups_button: AtomicPtr<QPushButton>,
    filter_case_sensitive_button: AtomicPtr<QPushButton>,
    filter_column_selector: AtomicPtr<QComboBox>,
    filter_line_edit: AtomicPtr<QLineEdit>,

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
    context_menu_rewrite_selection: AtomicPtr<QAction>,
    context_menu_undo: AtomicPtr<QAction>,
    context_menu_redo: AtomicPtr<QAction>,
    context_menu_import_tsv: AtomicPtr<QAction>,
    context_menu_export_tsv: AtomicPtr<QAction>,
    context_menu_sidebar: AtomicPtr<QAction>,
    context_menu_search: AtomicPtr<QAction>,
    smart_delete: AtomicPtr<QAction>,

    sidebar_hide_checkboxes: Arc<Vec<AtomicPtr<QCheckBox>>>,
    sidebar_freeze_checkboxes: Arc<Vec<AtomicPtr<QCheckBox>>>,

    search_search_button: AtomicPtr<QPushButton>,
    search_replace_current_button: AtomicPtr<QPushButton>,
    search_replace_all_button: AtomicPtr<QPushButton>,
    search_close_button: AtomicPtr<QPushButton>,
    search_prev_match_button: AtomicPtr<QPushButton>,
    search_next_match_button: AtomicPtr<QPushButton>,

    table_name: String,
    table_definition: Arc<Definition>,

    undo_model: AtomicPtr<QStandardItemModel>,
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
        app_ui: &AppUI,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<(TheOneSlot, Option<PackedFileInfo>)> {

        // Get the decoded Table.
        if packed_file_path.borrow().is_empty() { CENTRAL_COMMAND.send_message_qt(Command::GetDependencyPackFilesList); }
        else { CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_path.borrow().to_vec())); }

        let response = CENTRAL_COMMAND.recv_message_qt();
        let (table_data, packed_file_info) = match response {
            Response::DBPackedFileInfo((table, packed_file_info)) => (TableType::DB(table), Some(packed_file_info)),
            Response::LocPackedFileInfo((table, packed_file_info)) => (TableType::Loc(table), Some(packed_file_info)),
            Response::VecString(table) => (TableType::DependencyManager(table.iter().map(|x| vec![DecodedData::StringU8(x.to_owned()); 1]).collect::<Vec<Vec<DecodedData>>>()), None),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
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
            Response::BTreeMapI32HashMapStringString(dependency_data) => dependency_data,
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // Create the locks for undoing and saving. These are needed to optimize the undo/saving process.
        let undo_lock = Arc::new(AtomicBool::new(false));
        let save_lock = Arc::new(AtomicBool::new(false));

        // Prepare the Table and its model.
        let mut filter_model = QSortFilterProxyModel::new_0a();
        let mut model = QStandardItemModel::new_0a();
        filter_model.set_source_model(&mut model);
        let (table_view_primary, table_view_frozen) = new_tableview_frozen_safe(&mut packed_file_view.get_mut_widget());

        // Make the last column fill all the available space, if the setting says so.
        if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
            table_view_primary.horizontal_header().set_stretch_last_section(true);
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
        layout.add_widget_5a(table_view_primary, 0, 0, 1, 4);
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

        //let mut context_menu_apply_submenu = Menu::new(&QString::from_std_str("A&pply..."));
        //let context_menu_apply_maths_to_selection = context_menu_apply_submenu.add_action(&QString::from_std_str("&Apply Maths to Selection"));
        let context_menu_rewrite_selection = context_menu.add_action_q_string(&QString::from_std_str("&Rewrite Selection"));

        //let mut context_menu_clone_submenu = Menu::new(&QString::from_std_str("&Clone..."));

        let context_menu_clone_and_insert = context_menu.add_action_q_string(&QString::from_std_str("&Clone and Insert"));
        let context_menu_clone_and_append = context_menu.add_action_q_string(&QString::from_std_str("Clone and &Append"));
        let mut context_menu_copy_submenu = QMenu::from_q_string(&QString::from_std_str("&Copy..."));
        let context_menu_copy = context_menu_copy_submenu.add_action_q_string(&QString::from_std_str("&Copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action_q_string(&QString::from_std_str("&Copy as &LUA Table"));

        let context_menu_paste = context_menu.add_action_q_string(&QString::from_std_str("&Paste"));

        let context_menu_search = context_menu.add_action_q_string(&QString::from_std_str("&Search"));
        let context_menu_sidebar = context_menu.add_action_q_string(&QString::from_std_str("Si&debar"));

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

        //--------------------------------------------------//
        // Search Section.
        //--------------------------------------------------//
        //
        let mut search_widget = QWidget::new_0a().into_ptr();
        let mut search_grid = create_grid_layout(search_widget);

        let mut search_matches_label = QLabel::new();
        let search_search_label = QLabel::from_q_string(&QString::from_std_str("Search Pattern:"));
        let search_replace_label = QLabel::from_q_string(&QString::from_std_str("Replace Pattern:"));
        let mut search_search_line_edit = QLineEdit::new();
        let mut search_replace_line_edit = QLineEdit::new();
        let mut search_prev_match_button = QPushButton::from_q_string(&QString::from_std_str("Prev. Match"));
        let mut search_next_match_button = QPushButton::from_q_string(&QString::from_std_str("Next Match"));
        let mut search_search_button = QPushButton::from_q_string(&QString::from_std_str("Search"));
        let mut search_replace_current_button = QPushButton::from_q_string(&QString::from_std_str("Replace Current"));
        let mut search_replace_all_button = QPushButton::from_q_string(&QString::from_std_str("Replace All"));
        let mut search_close_button = QPushButton::from_q_string(&QString::from_std_str("Close"));
        let mut search_column_selector = QComboBox::new_0a();
        let search_column_list = QStandardItemModel::new_0a();
        let mut search_case_sensitive_button = QPushButton::from_q_string(&QString::from_std_str("Case Sensitive"));

        search_search_line_edit.set_placeholder_text(&QString::from_std_str("Type here what you want to search."));
        search_replace_line_edit.set_placeholder_text(&QString::from_std_str("If you want to replace the searched text with something, type the replacement here."));

        search_column_selector.set_model(search_column_list.into_ptr());
        search_column_selector.add_item_q_string(&QString::from_std_str("* (All Columns)"));
        for column in &fields {
            search_column_selector.add_item_q_string(&QString::from_std_str(&utils::clean_column_names(&column.name)));
        }
        search_case_sensitive_button.set_checkable(true);

        search_prev_match_button.set_enabled(false);
        search_next_match_button.set_enabled(false);
        search_replace_current_button.set_enabled(false);
        search_replace_all_button.set_enabled(false);

        // Add all the widgets to the search grid.
        search_grid.add_widget_5a(search_search_label.into_ptr(), 0, 0, 1, 1);
        search_grid.add_widget_5a(&mut search_search_line_edit, 0, 1, 1, 1);
        search_grid.add_widget_5a(&mut search_prev_match_button, 0, 2, 1, 1);
        search_grid.add_widget_5a(&mut search_next_match_button, 0, 3, 1, 1);
        search_grid.add_widget_5a(search_replace_label.into_ptr(), 1, 0, 1, 1);
        search_grid.add_widget_5a(&mut search_replace_line_edit, 1, 1, 1, 3);
        search_grid.add_widget_5a(&mut search_search_button, 0, 4, 1, 1);
        search_grid.add_widget_5a(&mut search_replace_current_button, 1, 4, 1, 1);
        search_grid.add_widget_5a(&mut search_replace_all_button, 2, 4, 1, 1);
        search_grid.add_widget_5a(&mut search_close_button, 2, 0, 1, 1);
        search_grid.add_widget_5a(&mut search_matches_label, 2, 1, 1, 1);
        search_grid.add_widget_5a(&mut search_column_selector, 2, 2, 1, 1);
        search_grid.add_widget_5a(&mut search_case_sensitive_button, 2, 3, 1, 1);

        layout.add_widget_5a(search_widget, 1, 0, 1, 4);
        layout.set_column_stretch(0, 10);
        search_widget.hide();

        //--------------------------------------------------//
        // Freeze/Hide Section.
        //--------------------------------------------------//

        // Create the search and hide/show/freeze widgets.
        let sidebar_widget = QWidget::new_0a().into_ptr();
        let mut sidebar_scroll_area = QScrollArea::new_0a().into_ptr();
        let mut sidebar_grid = create_grid_layout(sidebar_widget);
        sidebar_scroll_area.set_widget(sidebar_widget);
        sidebar_scroll_area.set_widget_resizable(true);
        sidebar_scroll_area.horizontal_scroll_bar().set_enabled(false);
        sidebar_grid.set_contents_margins_4a(4, 0, 4, 4);
        sidebar_grid.set_spacing(4);

        let mut header_column = QLabel::from_q_string(&QString::from_std_str("<b><i>Column Name</i></b>"));
        let mut header_hidden = QLabel::from_q_string(&QString::from_std_str("<b><i>Hidden</i></b>"));
        let mut header_frozen = QLabel::from_q_string(&QString::from_std_str("<b><i>Frozen</i></b>"));

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&mut header_column, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&mut header_hidden, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&mut header_frozen, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(header_column.into_ptr(), 0, 0, 1, 1);
        sidebar_grid.add_widget_5a(header_hidden.into_ptr(), 0, 1, 1, 1);
        sidebar_grid.add_widget_5a(header_frozen.into_ptr(), 0, 2, 1, 1);

        let mut hide_show_checkboxes = vec![];
        let mut freeze_checkboxes = vec![];
        for (index, column) in fields.iter().enumerate() {
            let column_name = QLabel::from_q_string(&QString::from_std_str(&utils::clean_column_names(&column.name)));
            let mut hide_show_checkbox = QCheckBox::new();
            let mut freeze_unfreeze_checkbox = QCheckBox::new();

            sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&mut hide_show_checkbox, QFlags::from(AlignmentFlag::AlignHCenter));
            sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&mut freeze_unfreeze_checkbox, QFlags::from(AlignmentFlag::AlignHCenter));

            sidebar_grid.add_widget_5a(column_name.into_ptr(), (index + 1) as i32, 0, 1, 1);
            sidebar_grid.add_widget_5a(&mut hide_show_checkbox, (index + 1) as i32, 1, 1, 1);
            sidebar_grid.add_widget_5a(&mut freeze_unfreeze_checkbox, (index + 1) as i32, 2, 1, 1);

            hide_show_checkboxes.push(atomic_from_mut_ptr(hide_show_checkbox.into_ptr()));
            freeze_checkboxes.push(atomic_from_mut_ptr(freeze_unfreeze_checkbox.into_ptr()));
        }

        // Add all the stuff to the main grid and hide the search widget.
        layout.add_widget_5a(sidebar_scroll_area, 0, 4, 3, 1);
        sidebar_scroll_area.hide();
        sidebar_grid.set_row_stretch(999, 10);

        // Create the raw Struct and begin
        let mut packed_file_table_view_raw = PackedFileTableViewRaw {
            table_view_primary,
            table_view_frozen,
            table_filter: filter_model.into_ptr(),
            table_model: model.into_ptr(),
            table_enable_lookups_button: table_enable_lookups_button.into_ptr(),
            filter_line_edit: row_filter_line_edit.into_ptr(),
            filter_case_sensitive_button: row_filter_case_sensitive_button.into_ptr(),
            filter_column_selector: row_filter_column_selector.into_ptr(),
            column_sort_state: Arc::new(RwLock::new((-1, 0))),

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
            context_menu_rewrite_selection,
            context_menu_undo,
            context_menu_redo,
            context_menu_import_tsv,
            context_menu_export_tsv,
            context_menu_sidebar,
            context_menu_search,
            smart_delete,

            search_search_line_edit: search_search_line_edit.into_ptr(),
            search_replace_line_edit: search_replace_line_edit.into_ptr(),
            search_search_button: search_search_button.into_ptr(),
            search_replace_current_button: search_replace_current_button.into_ptr(),
            search_replace_all_button: search_replace_all_button.into_ptr(),
            search_close_button: search_close_button.into_ptr(),
            search_prev_match_button: search_prev_match_button.into_ptr(),
            search_next_match_button: search_next_match_button.into_ptr(),
            search_matches_label: search_matches_label.into_ptr(),
            search_column_selector: search_column_selector.into_ptr(),
            search_case_sensitive_button: search_case_sensitive_button.into_ptr(),
            search_data: Arc::new(RwLock::new(TableSearch::default())),

            sidebar_scroll_area,
            search_widget,

            dependency_data: Arc::new(RwLock::new(dependency_data)),
            table_definition: Arc::new(table_definition),

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
            *app_ui,
            &packed_file_path,
        );

        let mut packed_file_table_view = Self {
            table_view_primary: atomic_from_mut_ptr(packed_file_table_view_raw.table_view_primary),
            table_view_frozen: atomic_from_mut_ptr(packed_file_table_view_raw.table_view_frozen),
            table_model: atomic_from_mut_ptr(packed_file_table_view_raw.table_model),
            table_enable_lookups_button: atomic_from_mut_ptr(packed_file_table_view_raw.table_enable_lookups_button),
            filter_line_edit: atomic_from_mut_ptr(packed_file_table_view_raw.filter_line_edit),
            filter_case_sensitive_button: atomic_from_mut_ptr(packed_file_table_view_raw.filter_case_sensitive_button),
            filter_column_selector: atomic_from_mut_ptr(packed_file_table_view_raw.filter_column_selector),

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
            context_menu_rewrite_selection: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_rewrite_selection),
            context_menu_undo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_undo),
            context_menu_redo: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_redo),
            context_menu_import_tsv: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_import_tsv),
            context_menu_export_tsv: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_export_tsv),
            context_menu_sidebar: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_sidebar),
            context_menu_search: atomic_from_mut_ptr(packed_file_table_view_raw.context_menu_search),
            smart_delete: atomic_from_mut_ptr(packed_file_table_view_raw.smart_delete),

            sidebar_hide_checkboxes: Arc::new(hide_show_checkboxes),
            sidebar_freeze_checkboxes: Arc::new(freeze_checkboxes),

            search_search_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_search_button),
            search_replace_current_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_replace_current_button),
            search_replace_all_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_replace_all_button),
            search_close_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_close_button),
            search_prev_match_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_prev_match_button),
            search_next_match_button: atomic_from_mut_ptr(packed_file_table_view_raw.search_next_match_button),

            table_definition: packed_file_table_view_raw.table_definition.clone(),
            table_name,

            undo_model: atomic_from_mut_ptr(packed_file_table_view_raw.undo_model),
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
        packed_file_view.view = ViewType::Internal(View::Table(packed_file_table_view));
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

    /// This function returns a pointer to the filter's column selector combobox.
    pub fn get_mut_ptr_filter_column_selector(&self) -> MutPtr<QComboBox> {
        mut_ptr_from_atomic(&self.filter_column_selector)
    }

    /// This function returns a pointer to the filter's case sensitive button.
    pub fn get_mut_ptr_filter_case_sensitive_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.filter_case_sensitive_button)
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

    /// This function returns a pointer to the rewrite selection action.
    pub fn get_mut_ptr_context_menu_rewrite_selection(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_rewrite_selection)
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

    /// This function returns a pointer to the sidebar action.
    pub fn get_mut_ptr_context_menu_sidebar(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_sidebar)
    }

    /// This function returns a pointer to the search action.
    pub fn get_mut_ptr_context_menu_search(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.context_menu_search)
    }

    /// This function returns a vector with the entire hide/show checkbox list.
    pub fn get_hide_show_checkboxes(&self) -> Vec<MutPtr<QCheckBox>> {
        self.sidebar_hide_checkboxes.iter()
            .map(|x| mut_ptr_from_atomic(x))
            .collect()
    }

    /// This function returns a vector with the entire freeze checkbox list.
    pub fn get_freeze_checkboxes(&self) -> Vec<MutPtr<QCheckBox>> {
        self.sidebar_freeze_checkboxes.iter()
            .map(|x| mut_ptr_from_atomic(x))
            .collect()
    }

    /// This function returns a pointer to the search button in the search panel.
    pub fn get_mut_ptr_search_search_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_search_button)
    }

    /// This function returns a pointer to the prev match button in the search panel.
    pub fn get_mut_ptr_search_prev_match_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_prev_match_button)
    }

    /// This function returns a pointer to the next_match button in the search panel.
    pub fn get_mut_ptr_search_next_match_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_next_match_button)
    }

    /// This function returns a pointer to the replace current button in the search panel.
    pub fn get_mut_ptr_search_replace_current_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_replace_current_button)
    }

    /// This function returns a pointer to the replace all button in the search panel.
    pub fn get_mut_ptr_search_replace_all_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_replace_all_button)
    }

    /// This function returns a pointer to the close button in the search panel.
    pub fn get_mut_ptr_search_close_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.search_close_button)
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

//----------------------------------------------------------------//
// Implementations of `TableSearch`.
//----------------------------------------------------------------//

/// Default implementation for TableSearch.
impl Default for TableSearch {
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

/// Implementation of `TableSearch`.
impl TableSearch {

    /// This function returns the list of matches present in the model.
    fn get_matches_in_model(&self) -> Vec<MutPtr<QModelIndex>> {
        self.matches.iter().map(|x| x.0).collect()
    }

    /// This function returns the list of matches visible to the user with the current filter.
    fn get_matches_in_filter(&self) -> Vec<MutPtr<QModelIndex>> {
        self.matches.iter().filter_map(|x| x.1).collect()
    }

    /// This function returns the list of matches present in the model that are visible to the user with the current filter.
    fn get_visible_matches_in_model(&self) -> Vec<MutPtr<QModelIndex>> {
        self.matches.iter().filter(|x| x.1.is_some()).map(|x| x.0).collect()
    }

    /// This function takes care of searching data whithin a column, and adding the matches to the matches list.
    unsafe fn find_in_column(
        &mut self,
        model: MutPtr<QStandardItemModel>,
        filter: MutPtr<QSortFilterProxyModel>,
        definition: &Definition,
        flags: QFlags<MatchFlag>,
        column: i32
    ) {

        // First, check the column type. Boolean columns need special logic, as they cannot be matched by string.
        let is_bool = definition.fields[column as usize].field_type == FieldType::Boolean;
        let mut matches_unprocessed = if is_bool {
            match parse_str(&self.pattern.to_std_string()) {
                Ok(boolean) => {
                    let check_state = if boolean { CheckState::Checked } else { CheckState::Unchecked };
                    let mut items = QListOfQStandardItem::new();
                    for row in 0..model.row_count_0a() {
                        let item = model.item_2a(row, column);
                        if item.check_state() == check_state {
                            add_to_q_list_safe(items.as_mut_ptr(), item);
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

        for index in 0..matches_unprocessed.count() {
            let model_index = matches_unprocessed.index(index).as_ref().unwrap().index();
            let filter_model_index = filter.map_from_source(&model_index);
            self.matches.push((
                model_index.into_ptr(),
                if filter_model_index.is_valid() { Some(filter_model_index.into_ptr()) } else { None }
            ));
        }
    }

    /// This function takes care of updating the UI to reflect changes in the table search.
    pub unsafe fn update_search_ui(parent: &mut PackedFileTableViewRaw, update_type: TableSearchUpdate) {
        let table_search = &mut parent.search_data.write().unwrap();
        let matches_in_filter = table_search.get_matches_in_filter();
        let matches_in_model = table_search.get_matches_in_model();
        match update_type {
            TableSearchUpdate::Search => {
                if table_search.pattern.is_empty() {
                    parent.search_matches_label.set_text(&QString::new());
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // If no matches have been found, report it.
                else if table_search.matches.is_empty() {
                    table_search.current_item = None;
                    parent.search_matches_label.set_text(&QString::from_std_str("No matches found."));
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // Otherwise, if no matches have been found in the current filter, but they have been in the model...
                else if matches_in_filter.is_empty() {
                    table_search.current_item = None;
                    parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // Otherwise, matches have been found both, in the model and in the filter.
                else {
                    table_search.current_item = Some(0);
                    parent.search_matches_label.set_text(&QString::from_std_str(&format!("1 of {} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(true);
                    parent.search_replace_all_button.set_enabled(true);

                    if matches_in_filter.len() > 1 {
                        parent.search_next_match_button.set_enabled(true);
                    }
                    else {
                        parent.search_next_match_button.set_enabled(false);
                    }

                    parent.table_view_primary.selection_model().select_q_model_index_q_flags_selection_flag(
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
                        if *pos == 0 { parent.search_prev_match_button.set_enabled(false); }
                        else { parent.search_prev_match_button.set_enabled(true); }
                        if *pos as usize >= matches_in_filter.len() - 1 { parent.search_next_match_button.set_enabled(false); }
                        else { parent.search_next_match_button.set_enabled(true); }

                        parent.table_view_primary.selection_model().select_q_model_index_q_flags_selection_flag(
                            matches_in_filter[*pos as usize].as_ref().unwrap(),
                            QFlags::from(SelectionFlag::ClearAndSelect)
                        );
                        parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} of {} in current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches_in_model.len())));
                    }
                }
            }
            TableSearchUpdate::NextMatch => {
                let matches_in_model = table_search.get_matches_in_model();
                let matches_in_filter = table_search.get_matches_in_filter();
                if let Some(ref mut pos) = table_search.current_item {

                    // If we are in an invalid result, return. If it's the last one, disable the button and return.
                    if *pos as usize >= matches_in_filter.len() - 1 {
                        parent.search_next_match_button.set_enabled(false);
                    }
                    else {
                        *pos += 1;
                        if *pos == 0 { parent.search_prev_match_button.set_enabled(false); }
                        else { parent.search_prev_match_button.set_enabled(true); }
                        if *pos as usize >= matches_in_filter.len() - 1 { parent.search_next_match_button.set_enabled(false); }
                        else { parent.search_next_match_button.set_enabled(true); }

                        parent.table_view_primary.selection_model().select_q_model_index_q_flags_selection_flag(
                            matches_in_filter[*pos as usize].as_ref().unwrap(),
                            QFlags::from(SelectionFlag::ClearAndSelect)
                        );
                        parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} of {} in current filter ({} in total)", *pos + 1, matches_in_filter.len(), matches_in_model.len())));
                    }
                }
            }
            TableSearchUpdate::Update => {
                if table_search.pattern.is_empty() {
                    parent.search_matches_label.set_text(&QString::new());
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // If no matches have been found, report it.
                else if table_search.matches.is_empty() {
                    table_search.current_item = None;
                    parent.search_matches_label.set_text(&QString::from_std_str("No matches found."));
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // Otherwise, if no matches have been found in the current filter, but they have been in the model...
                else if matches_in_filter.is_empty() {
                    table_search.current_item = None;
                    parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} in current filter ({} in total)", matches_in_filter.len(), matches_in_model.len())));
                    parent.search_prev_match_button.set_enabled(false);
                    parent.search_next_match_button.set_enabled(false);
                    parent.search_replace_current_button.set_enabled(false);
                    parent.search_replace_all_button.set_enabled(false);
                }

                // Otherwise, matches have been found both, in the model and in the filter. Which means we have to recalculate
                // our position, and then behave more or less like a normal search.
                else {
                    table_search.current_item = match table_search.current_item {
                        Some(pos) => if (pos as usize) < matches_in_filter.len() { Some(pos) } else { Some(0) }
                        None => Some(0)
                    };

                    parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} of {} in current filter ({} in total)", table_search.current_item.unwrap(), matches_in_filter.len(), matches_in_model.len())));

                    if table_search.current_item.unwrap() == 0 {
                        parent.search_prev_match_button.set_enabled(false);
                    }
                    else {
                        parent.search_prev_match_button.set_enabled(true);
                    }

                    if matches_in_filter.len() > 1 && (table_search.current_item.unwrap() as usize) < matches_in_filter.len() - 1 {
                        parent.search_next_match_button.set_enabled(true);
                    }
                    else {
                        parent.search_next_match_button.set_enabled(false);
                    }

                    parent.search_replace_current_button.set_enabled(true);
                    parent.search_replace_all_button.set_enabled(true);
                }
            }
        }
    }

    /// This function takes care of updating the search data whenever a change that can alter the results happens.
    pub unsafe fn update_search(parent: &mut PackedFileTableViewRaw) {
        {
            let table_search = &mut parent.search_data.write().unwrap();
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
                None => (0..parent.table_definition.fields.len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model, parent.table_filter, &parent.table_definition, flags, *column);
            }
        }

        Self::update_search_ui(parent, TableSearchUpdate::Search);
    }

    /// This function takes care of searching the patter we provided in the TableView.
    pub unsafe fn search(parent: &mut PackedFileTableViewRaw) {
        {
            let table_search = &mut parent.search_data.write().unwrap();
            table_search.matches.clear();
            table_search.current_item = None;
            table_search.pattern = parent.search_search_line_edit.text().into_ptr();
            //table_search.regex = parent.search_search_line_edit.is_checked();
            table_search.case_sensitive = parent.search_case_sensitive_button.is_checked();
            table_search.column = {
                let column = parent.search_column_selector.current_text().to_std_string().replace(' ', "_").to_lowercase();
                if column == "*_(all_columns)" { None }
                else { Some(parent.table_definition.fields.iter().position(|x| x.name == column).unwrap() as i32) }
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
                None => (0..parent.table_definition.fields.len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model, parent.table_filter, &parent.table_definition, flags, *column);
            }
        }

        Self::update_search_ui(parent, TableSearchUpdate::Search);
    }

    /// This function takes care of moving the selection to the previous match on the matches list.
    pub unsafe fn prev_match(parent: &mut PackedFileTableViewRaw) {
        Self::update_search_ui(parent, TableSearchUpdate::PrevMatch);
    }

    /// This function takes care of moving the selection to the next match on the matches list.
    pub unsafe fn next_match(parent: &mut PackedFileTableViewRaw) {
        Self::update_search_ui(parent, TableSearchUpdate::NextMatch);
    }

    /// This function takes care of replacing the current match with the provided replacing text.
    pub unsafe fn replace_current(parent: &mut PackedFileTableViewRaw) {

        // NOTE: WE CANNOT HAVE THE SEARCH DATA LOCK UNTIL AFTER WE DO THE REPLACE. That's why there are a lot of read here.
        let text_source = parent.search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {

            // Get the replace data here, as we probably don't have it updated.
            parent.search_data.write().unwrap().replace = parent.search_replace_line_edit.text().into_ptr();
            let text_replace = parent.search_data.read().unwrap().replace.to_std_string();
            if text_source == text_replace { return }

            // And if we got a valid position.
            let mut item;
            let replaced_text;
            if let Some(ref position) = parent.search_data.read().unwrap().current_item {

                // Here is save to lock, as the lock will be drop before doing the replace.
                let table_search = &mut parent.search_data.read().unwrap();

                // Get the list of all valid ModelIndex for the current filter and the current position.
                let matches_in_model_and_filter = table_search.get_visible_matches_in_model();
                let model_index = matches_in_model_and_filter[*position as usize];

                // If the position is still valid (not required, but just in case)...
                if model_index.is_valid() {
                    item = parent.table_model.item_from_index(model_index.as_ref().unwrap());

                    if parent.table_definition.fields[model_index.column() as usize].field_type == FieldType::Boolean {
                        replaced_text = text_replace;
                    }
                    else {
                        let text = item.text().to_std_string();
                        replaced_text = text.replace(&text_source, &text_replace);
                    }

                    // We need to do an extra check to ensure the new text can be in the field.
                    match parent.table_definition.fields[model_index.column() as usize].field_type {
                        FieldType::Boolean => if parse_str(&replaced_text).is_err() { return show_dialog(parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::Float => if replaced_text.parse::<f32>().is_err() { return show_dialog(parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::Integer => if replaced_text.parse::<i32>().is_err() { return show_dialog(parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::LongInteger => if replaced_text.parse::<i64>().is_err() { return show_dialog(parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        _ =>  {}
                    }
                } else { return }
            } else { return }

            // At this point, we trigger editions. Which mean, here ALL LOCKS SHOULD HAVE BEEN ALREADY DROP.
            match parent.table_definition.fields[item.column() as usize].field_type {
                FieldType::Boolean => item.set_check_state(if parse_str(&replaced_text).unwrap() { CheckState::Checked } else { CheckState::Unchecked }),
                FieldType::Float => item.set_data_2a(&QVariant::from_float(replaced_text.parse::<f32>().unwrap()), 2),
                FieldType::Integer => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i32>().unwrap()), 2),
                FieldType::LongInteger => item.set_data_2a(&QVariant::from_i64(replaced_text.parse::<i64>().unwrap()), 2),
                _ => item.set_text(&QString::from_std_str(&replaced_text)),
            }

            // At this point, the edition has been done. We're free to lock again. If we still have matches, select the next match, if any, or the first one.
            let table_search = &mut parent.search_data.read().unwrap();
            if let Some(pos) = table_search.current_item {
                let matches_in_filter = table_search.get_matches_in_filter();

                parent.table_view_primary.selection_model().select_q_model_index_q_flags_selection_flag(
                    matches_in_filter[pos as usize].as_ref().unwrap(),
                    QFlags::from(SelectionFlag::ClearAndSelect)
                );
            }
        }
    }

    pub unsafe fn replace_all(parent: &mut PackedFileTableViewRaw) {
        let text_source = parent.search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {

        }
    }
}

