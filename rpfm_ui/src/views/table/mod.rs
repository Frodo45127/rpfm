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
Module with all the code for managing the view for Tables.
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

use qt_core::QAbstractItemModel;
use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::CheckState;
use qt_core::QFlags;
use qt_core::AlignmentFlag;
use qt_core::QSortFilterProxyModel;
use qt_core::QStringList;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::MatchFlag;
use qt_core::QPtr;
use qt_core::QSignalBlocker;
use qt_core::QObject;

use cpp_core::Ptr;

use std::collections::BTreeMap;
use std::{fmt, fmt::Debug};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::rc::Rc;

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::common::parse_str_as_bool;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::table::{DependencyData, anim_fragment::AnimFragment, animtable::AnimTable, DecodedData, db::DB, loc::Loc, matched_combat::MatchedCombat, Table};
use rpfm_lib::schema::{Definition, FieldType, Schema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, qtre, tr};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::utils::atomic_from_ptr;
use crate::utils::create_grid_layout;
use crate::utils::show_dialog;
use crate::utils::ptr_from_atomic;

use self::slots::*;
use self::utils::*;

mod connections;
pub mod slots;
mod raw;
mod shortcuts;
mod tips;
pub mod utils;

// Column default sizes.
pub static COLUMN_SIZE_BOOLEAN: i32 = 100;
pub static COLUMN_SIZE_NUMBER: i32 = 140;
pub static COLUMN_SIZE_STRING: i32 = 350;


pub static ITEM_IS_KEY: i32 = 20;
pub static ITEM_IS_ADDED: i32 = 21;
pub static ITEM_IS_MODIFIED: i32 = 22;
pub static ITEM_HAS_ERROR: i32 = 25;
pub static ITEM_HAS_WARNING: i32 = 26;
pub static ITEM_HAS_INFO: i32 = 27;
pub static ITEM_HAS_SOURCE_VALUE: i32 = 30;
pub static ITEM_SOURCE_VALUE: i32 = 31;
pub static ITEM_IS_SEQUENCE: i32 = 35;
pub static ITEM_SEQUENCE_DATA: i32 = 36;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum is used to distinguish between the different types of tables we can decode.
#[derive(Clone, Debug)]
pub enum TableType {
    AnimFragment(AnimFragment),
    AnimTable(AnimTable),
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    Loc(Loc),
    MatchedCombat(MatchedCombat),

    /// This one is for random views that just need a table with advanced behavior.
    NormalTable(Table),
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

/// This struct contains pointers to all the widgets in a Table View.
pub struct TableView {
    table_view_primary: QBox<QTableView>,
    table_view_frozen: QBox<QTableView>,
    table_filter: QBox<QSortFilterProxyModel>,
    table_model: QBox<QStandardItemModel>,

    filter_base_widget: QBox<QWidget>,
    filters: Arc<RwLock<Vec<Arc<FilterView>>>>,

    column_sort_state: Arc<RwLock<(i32, i8)>>,

    context_menu: QBox<QMenu>,
    context_menu_add_rows: QPtr<QAction>,
    context_menu_insert_rows: QPtr<QAction>,
    context_menu_delete_rows: QPtr<QAction>,
    context_menu_delete_rows_not_in_filter: QPtr<QAction>,
    context_menu_clone_and_append: QPtr<QAction>,
    context_menu_clone_and_insert: QPtr<QAction>,
    context_menu_copy: QPtr<QAction>,
    context_menu_copy_as_lua_table: QPtr<QAction>,
    context_menu_paste: QPtr<QAction>,
    context_menu_paste_as_new_row: QPtr<QAction>,
    context_menu_invert_selection: QPtr<QAction>,
    context_menu_reset_selection: QPtr<QAction>,
    context_menu_rewrite_selection: QPtr<QAction>,
    context_menu_generate_ids: QPtr<QAction>,
    context_menu_undo: QPtr<QAction>,
    context_menu_redo: QPtr<QAction>,
    context_menu_import_tsv: QPtr<QAction>,
    context_menu_export_tsv: QPtr<QAction>,
    context_menu_resize_columns: QPtr<QAction>,
    context_menu_sidebar: QPtr<QAction>,
    context_menu_search: QPtr<QAction>,
    context_menu_cascade_edition: QPtr<QAction>,
    context_menu_patch_column: QPtr<QAction>,
    smart_delete: QBox<QAction>,

    _context_menu_go_to: QBox<QMenu>,
    context_menu_go_to_definition: QPtr<QAction>,
    context_menu_go_to_loc: Vec<QPtr<QAction>>,

    sidebar_scroll_area: QBox<QScrollArea>,
    search_widget: QBox<QWidget>,

    sidebar_hide_checkboxes: Vec<QBox<QCheckBox>>,
    sidebar_hide_checkboxes_all: QBox<QCheckBox>,
    sidebar_freeze_checkboxes: Vec<QBox<QCheckBox>>,
    sidebar_freeze_checkboxes_all: QBox<QCheckBox>,

    search_search_line_edit: QBox<QLineEdit>,
    search_replace_line_edit: QBox<QLineEdit>,
    search_search_button: QBox<QPushButton>,
    search_replace_current_button: QBox<QPushButton>,
    search_replace_all_button: QBox<QPushButton>,
    search_close_button: QBox<QPushButton>,
    search_prev_match_button: QBox<QPushButton>,
    search_next_match_button: QBox<QPushButton>,
    search_matches_label: QBox<QLabel>,
    search_column_selector: QBox<QComboBox>,
    search_case_sensitive_button: QBox<QPushButton>,
    search_data: Arc<RwLock<TableSearch>>,

    _table_status_bar: QBox<QWidget>,
    table_status_bar_line_counter_label: QBox<QLabel>,

    table_name: Option<String>,
    table_uuid: Option<String>,
    data_source: Arc<RwLock<DataSource>>,
    packed_file_path: Option<Arc<RwLock<Vec<String>>>>,
    packed_file_type: Arc<PackedFileType>,
    table_definition: Arc<RwLock<Definition>>,
    dependency_data: Arc<RwLock<BTreeMap<i32, DependencyData>>>,
    banned_table: bool,

    pub save_lock: Arc<AtomicBool>,
    pub undo_lock: Arc<AtomicBool>,

    undo_model: QBox<QStandardItemModel>,
    history_undo: Arc<RwLock<Vec<TableOperations>>>,
    history_redo: Arc<RwLock<Vec<TableOperations>>>,

    pub timer_delayed_updates: QBox<QTimer>,
}

/// This struct contains the stuff needed for a filter row.
pub struct FilterView {
    filter_widget: QBox<QWidget>,
    filter_match_group_selector: QBox<QComboBox>,
    filter_case_sensitive_button: QBox<QPushButton>,
    filter_column_selector: QBox<QComboBox>,
    filter_show_blank_cells_button: QBox<QPushButton>,
    filter_timer_delayed_updates: QBox<QTimer>,
    filter_line_edit: QBox<QLineEdit>,
    filter_add: QBox<QPushButton>,
    filter_remove: QBox<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `TableView`.
impl TableView {

    /// This function creates a new Table View, and sets up his slots and connections.
    ///
    /// NOTE: To open the dependency list, pass it an empty path.
    pub unsafe fn new_view(
        parent: &QBox<QWidget>,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        table_data: TableType,
        packed_file_path: Option<Arc<RwLock<Vec<String>>>>,
        data_source: Arc<RwLock<DataSource>>,
    ) -> Result<Arc<Self>> {

        let (table_definition, table_name, table_uuid, packed_file_type) = match table_data {
            TableType::DependencyManager(_) => {
                if let Some(schema) = &*SCHEMA.read().unwrap() {
                    (schema.get_ref_versioned_file_dep_manager()?.get_version_list()[0].clone(), None, None, PackedFileType::DependencyPackFilesList)
                } else {
                    return Err(ErrorKind::SchemaNotFound.into());
                }
            },
            TableType::DB(ref table) => (table.get_definition(), Some(table.get_table_name()), Some(table.get_uuid()), PackedFileType::DB),
            TableType::Loc(ref table) => (table.get_definition(), None, None, PackedFileType::Loc),
            TableType::MatchedCombat(ref table) => (table.get_definition(), None, None, PackedFileType::MatchedCombat),
            TableType::AnimTable(ref table) => (table.get_definition(), None, None, PackedFileType::AnimTable),
            TableType::AnimFragment(ref table) => (table.get_definition(), None, None, PackedFileType::AnimFragment),
            TableType::NormalTable(ref table) => (table.get_definition(), None, None, PackedFileType::Unknown),
        };

        // Get the dependency data of this Table.
        let table_name_for_ref = if let Some(ref name) = table_name { name.to_owned() } else { "".to_owned() };
        let dependency_data = get_reference_data(&table_name_for_ref, &table_definition)?;

        // Create the locks for undoing and saving. These are needed to optimize the undo/saving process.
        let undo_lock = Arc::new(AtomicBool::new(false));
        let save_lock = Arc::new(AtomicBool::new(false));

        // Prepare the Table and its model.
        let table_filter = new_tableview_filter_safe(parent.static_upcast());
        let table_model = QStandardItemModel::new_1a(parent);
        let undo_model = QStandardItemModel::new_1a(parent);
        table_filter.set_source_model(&table_model);
        let (table_view_primary, table_view_frozen) = new_tableview_frozen_safe(&parent.as_ptr());
        set_frozen_data_model_safe(&table_view_primary.as_ptr(), &table_filter.static_upcast::<QAbstractItemModel>().as_ptr());

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


        // Create the filter's widgets.
        let filter_base_widget = QWidget::new_1a(parent);
        let _filter_base_grid = create_grid_layout(filter_base_widget.static_upcast());

        // Add everything to the grid.
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        let mut banned_table = false;
        if let PackedFileType::DependencyPackFilesList = packed_file_type {
            let warning_message = QLabel::from_q_string_q_widget(&qtr("dependency_packfile_list_label"), parent);
            layout.add_widget_5a(&warning_message, 0, 0, 1, 4);
        } else if let PackedFileType::DB = packed_file_type {
            banned_table = GAME_SELECTED.read().unwrap().is_packedfile_banned(&["db".to_owned(), table_name_for_ref]);
            if banned_table {
                let warning_message = QLabel::from_q_string_q_widget(&qtr("banned_tables_warning"), parent);
                layout.add_widget_5a(&warning_message, 0, 0, 1, 4);
            }
        }

        let table_status_bar = QWidget::new_1a(parent);
        let table_status_bar_grid = create_grid_layout(table_status_bar.static_upcast());
        let table_status_bar_line_counter_label = QLabel::from_q_string_q_widget(&qtre("line_counter", &["0", "0"]), &table_status_bar);
        table_status_bar_grid.add_widget_5a(&table_status_bar_line_counter_label, 0, 0, 1, 1);

        layout.add_widget_5a(&table_view_primary, 1, 0, 1, 1);
        layout.add_widget_5a(&table_status_bar, 2, 0, 1, 2);
        layout.add_widget_5a(&filter_base_widget, 4, 0, 1, 2);

        // Action to make the delete button delete contents.
        let smart_delete = QAction::from_q_object(&table_view_primary);

        // Create the Contextual Menu for the TableView.
        let context_menu = QMenu::from_q_widget(&table_view_primary);
        let context_menu_add_rows = context_menu.add_action_q_string(&qtr("context_menu_add_rows"));
        let context_menu_insert_rows = context_menu.add_action_q_string(&qtr("context_menu_insert_rows"));
        let context_menu_delete_rows = context_menu.add_action_q_string(&qtr("context_menu_delete_rows"));
        let context_menu_delete_rows_not_in_filter = context_menu.add_action_q_string(&qtr("context_menu_delete_filtered_out_rows"));

        let context_menu_clone_submenu = QMenu::from_q_string_q_widget(&qtr("context_menu_clone_submenu"), &table_view_primary);
        let context_menu_clone_and_insert = context_menu_clone_submenu.add_action_q_string(&qtr("context_menu_clone_and_insert"));
        let context_menu_clone_and_append = context_menu_clone_submenu.add_action_q_string(&qtr("context_menu_clone_and_append"));

        let context_menu_copy_submenu = QMenu::from_q_string_q_widget(&qtr("context_menu_copy_submenu"), &table_view_primary);
        let context_menu_copy = context_menu_copy_submenu.add_action_q_string(&qtr("context_menu_copy"));
        let context_menu_copy_as_lua_table = context_menu_copy_submenu.add_action_q_string(&qtr("context_menu_copy_as_lua_table"));

        let context_menu_paste = context_menu.add_action_q_string(&qtr("context_menu_paste"));
        let context_menu_paste_as_new_row = context_menu.add_action_q_string(&qtr("context_menu_paste_as_new_row"));

        let context_menu_generate_ids = context_menu.add_action_q_string(&qtr("context_menu_generate_ids"));
        let context_menu_rewrite_selection = context_menu.add_action_q_string(&qtr("context_menu_rewrite_selection"));
        let context_menu_invert_selection = context_menu.add_action_q_string(&qtr("context_menu_invert_selection"));
        let context_menu_reset_selection = context_menu.add_action_q_string(&qtr("context_menu_reset_selection"));
        let context_menu_resize_columns = context_menu.add_action_q_string(&qtr("context_menu_resize_columns"));

        let context_menu_import_tsv = context_menu.add_action_q_string(&qtr("context_menu_import_tsv"));
        let context_menu_export_tsv = context_menu.add_action_q_string(&qtr("context_menu_export_tsv"));

        let context_menu_search = context_menu.add_action_q_string(&qtr("context_menu_search"));
        let context_menu_sidebar = context_menu.add_action_q_string(&qtr("context_menu_sidebar"));

        let context_menu_cascade_edition = context_menu.add_action_q_string(&qtr("context_menu_cascade_edition"));
        let context_menu_patch_column = context_menu.add_action_q_string(&qtr("context_menu_patch_column"));

        let context_menu_undo = context_menu.add_action_q_string(&qtr("context_menu_undo"));
        let context_menu_redo = context_menu.add_action_q_string(&qtr("context_menu_redo"));

        let context_menu_go_to = QMenu::from_q_string_q_widget(&qtr("context_menu_go_to"), &table_view_primary);
        let context_menu_go_to_definition = context_menu_go_to.add_action_q_string(&qtr("context_menu_go_to_definition"));
        let mut context_menu_go_to_loc = vec![];

        for (index, loc_column) in table_definition.get_localised_fields().iter().enumerate() {
            let context_menu_go_to_loc_action = context_menu_go_to.add_action_q_string(&qtre("context_menu_go_to_loc", &[loc_column.get_name()]));
            if index == 0 { context_menu_go_to.insert_separator(&context_menu_go_to_loc_action); }
            context_menu_go_to_loc.push(context_menu_go_to_loc_action)
        }

        // Insert some separators to space the menu, and the paste submenu.
        context_menu.insert_menu(&context_menu_paste, &context_menu_clone_submenu);
        context_menu.insert_menu(&context_menu_paste, &context_menu_copy_submenu);
        context_menu.insert_menu(&context_menu_paste, &context_menu_go_to);
        context_menu.insert_separator(&context_menu_rewrite_selection);
        context_menu.insert_separator(&context_menu_import_tsv);
        context_menu.insert_separator(&context_menu_search);
        context_menu.insert_separator(&context_menu_undo);

        //--------------------------------------------------//
        // Search Section.
        //--------------------------------------------------//
        //
        let search_widget = QWidget::new_1a(parent);
        let search_grid = create_grid_layout(search_widget.static_upcast());

        let search_matches_label = QLabel::from_q_widget(&search_widget);
        let search_search_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Search Pattern:"), &search_widget);
        let search_replace_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Replace Pattern:"), &search_widget);
        let search_search_line_edit = QLineEdit::from_q_widget(&search_widget);
        let search_replace_line_edit = QLineEdit::from_q_widget(&search_widget);
        let search_prev_match_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Prev. Match"), &search_widget);
        let search_next_match_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Next Match"), &search_widget);
        let search_search_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Search"), &search_widget);
        let search_replace_current_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Replace Current"), &search_widget);
        let search_replace_all_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Replace All"), &search_widget);
        let search_close_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Close"), &search_widget);
        let search_column_selector = QComboBox::new_1a(&search_widget);
        let search_column_list = QStandardItemModel::new_1a(&search_column_selector);
        let search_case_sensitive_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Case Sensitive"), &search_widget);

        search_search_line_edit.set_placeholder_text(&QString::from_std_str("Type here what you want to search."));
        search_replace_line_edit.set_placeholder_text(&QString::from_std_str("If you want to replace the searched text with something, type the replacement here."));
        search_search_line_edit.set_clear_button_enabled(true);
        search_replace_line_edit.set_clear_button_enabled(true);

        search_column_selector.set_model(&search_column_list);
        search_column_selector.add_item_q_string(&QString::from_std_str("* (All Columns)"));

        let fields = table_definition.get_fields_sorted();
        for column in &fields {
            search_column_selector.add_item_q_string(&QString::from_std_str(&utils::clean_column_names(column.get_name())));
        }
        search_case_sensitive_button.set_checkable(true);

        search_prev_match_button.set_enabled(false);
        search_next_match_button.set_enabled(false);
        search_replace_current_button.set_enabled(false);
        search_replace_all_button.set_enabled(false);

        // Add all the widgets to the search grid.
        search_grid.add_widget_5a(&search_search_label, 0, 0, 1, 1);
        search_grid.add_widget_5a(&search_search_line_edit, 0, 1, 1, 1);
        search_grid.add_widget_5a(&search_prev_match_button, 0, 2, 1, 1);
        search_grid.add_widget_5a(&search_next_match_button, 0, 3, 1, 1);
        search_grid.add_widget_5a(&search_replace_label, 1, 0, 1, 1);
        search_grid.add_widget_5a(&search_replace_line_edit, 1, 1, 1, 3);
        search_grid.add_widget_5a(&search_search_button, 0, 4, 1, 1);
        search_grid.add_widget_5a(&search_replace_current_button, 1, 4, 1, 1);
        search_grid.add_widget_5a(&search_replace_all_button, 2, 4, 1, 1);
        search_grid.add_widget_5a(&search_close_button, 2, 0, 1, 1);
        search_grid.add_widget_5a(&search_matches_label, 2, 1, 1, 1);
        search_grid.add_widget_5a(&search_column_selector, 2, 2, 1, 1);
        search_grid.add_widget_5a(&search_case_sensitive_button, 2, 3, 1, 1);

        layout.add_widget_5a(&search_widget, 3, 0, 1, 2);
        layout.set_column_stretch(0, 10);
        search_widget.hide();

        //--------------------------------------------------//
        // Freeze/Hide Section.
        //--------------------------------------------------//

        // Create the search and hide/show/freeze widgets.
        let sidebar_scroll_area = QScrollArea::new_1a(parent);
        let sidebar_widget = QWidget::new_1a(&sidebar_scroll_area);
        let sidebar_grid = create_grid_layout(sidebar_widget.static_upcast());
        sidebar_scroll_area.set_widget(&sidebar_widget);
        sidebar_scroll_area.set_widget_resizable(true);
        sidebar_scroll_area.horizontal_scroll_bar().set_enabled(false);
        sidebar_grid.set_contents_margins_4a(4, 0, 4, 4);
        sidebar_grid.set_spacing(4);

        let header_column = QLabel::from_q_string_q_widget(&qtr("header_column"), &sidebar_widget);
        let header_hidden = QLabel::from_q_string_q_widget(&qtr("header_hidden"), &sidebar_widget);
        let header_frozen = QLabel::from_q_string_q_widget(&qtr("header_frozen"), &sidebar_widget);

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_column, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_hidden, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_frozen, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(&header_column, 0, 0, 1, 1);
        sidebar_grid.add_widget_5a(&header_hidden, 0, 1, 1, 1);
        sidebar_grid.add_widget_5a(&header_frozen, 0, 2, 1, 1);

        let label_all = QLabel::from_q_string_q_widget(&qtr("all"), &sidebar_widget);
        let sidebar_hide_checkboxes_all = QCheckBox::from_q_widget(&sidebar_widget);
        let sidebar_freeze_checkboxes_all = QCheckBox::from_q_widget(&sidebar_widget);
        sidebar_freeze_checkboxes_all.set_enabled(false);

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&sidebar_hide_checkboxes_all, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&sidebar_freeze_checkboxes_all, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(&label_all, 1, 0, 1, 1);
        sidebar_grid.add_widget_5a(&sidebar_hide_checkboxes_all, 1, 1, 1, 1);
        sidebar_grid.add_widget_5a(&sidebar_freeze_checkboxes_all, 1, 2, 1, 1);

        let mut sidebar_hide_checkboxes = vec![];
        let mut sidebar_freeze_checkboxes = vec![];
        for (index, column) in fields.iter().enumerate() {
            let column_name = QLabel::from_q_string_q_widget(&QString::from_std_str(&utils::clean_column_names(column.get_name())), &sidebar_widget);
            let hide_show_checkbox = QCheckBox::from_q_widget(&sidebar_widget);
            let freeze_unfreeze_checkbox = QCheckBox::from_q_widget(&sidebar_widget);
            freeze_unfreeze_checkbox.set_enabled(false);

            sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&hide_show_checkbox, QFlags::from(AlignmentFlag::AlignHCenter));
            sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&freeze_unfreeze_checkbox, QFlags::from(AlignmentFlag::AlignHCenter));

            sidebar_grid.add_widget_5a(&column_name, (index + 2) as i32, 0, 1, 1);
            sidebar_grid.add_widget_5a(&hide_show_checkbox, (index + 2) as i32, 1, 1, 1);
            sidebar_grid.add_widget_5a(&freeze_unfreeze_checkbox, (index + 2) as i32, 2, 1, 1);

            sidebar_hide_checkboxes.push(hide_show_checkbox);
            sidebar_freeze_checkboxes.push(freeze_unfreeze_checkbox);
        }

        // Add all the stuff to the main grid and hide the search widget.
        layout.add_widget_5a(&sidebar_scroll_area, 0, 4, 5, 1);
        sidebar_scroll_area.hide();
        sidebar_grid.set_row_stretch(999, 10);

        let timer_delayed_updates = QTimer::new_1a(parent);
        timer_delayed_updates.set_single_shot(true);

        // Create the raw Struct and begin
        let packed_file_table_view = Arc::new(TableView {
            table_view_primary,
            table_view_frozen,
            table_filter,
            table_model,
            //table_enable_lookups_button: table_enable_lookups_button.into_ptr(),
            filters: Arc::new(RwLock::new(vec![])),
            filter_base_widget,
            column_sort_state: Arc::new(RwLock::new((-1, 0))),

            context_menu,
            context_menu_add_rows,
            context_menu_insert_rows,
            context_menu_delete_rows,
            context_menu_delete_rows_not_in_filter,
            context_menu_clone_and_append,
            context_menu_clone_and_insert,
            context_menu_copy,
            context_menu_copy_as_lua_table,
            context_menu_paste,
            context_menu_paste_as_new_row,
            context_menu_invert_selection,
            context_menu_reset_selection,
            context_menu_rewrite_selection,
            context_menu_generate_ids,
            context_menu_undo,
            context_menu_redo,
            context_menu_import_tsv,
            context_menu_export_tsv,
            context_menu_resize_columns,
            context_menu_sidebar,
            context_menu_search,
            context_menu_cascade_edition,
            context_menu_patch_column,
            smart_delete,

            _context_menu_go_to: context_menu_go_to,
            context_menu_go_to_definition,
            context_menu_go_to_loc,

            search_search_line_edit,
            search_replace_line_edit,
            search_search_button,
            search_replace_current_button,
            search_replace_all_button,
            search_close_button,
            search_prev_match_button,
            search_next_match_button,
            search_matches_label,
            search_column_selector,
            search_case_sensitive_button,
            search_data: Arc::new(RwLock::new(TableSearch::default())),

            sidebar_hide_checkboxes,
            sidebar_hide_checkboxes_all,
            sidebar_freeze_checkboxes,
            sidebar_freeze_checkboxes_all,

            sidebar_scroll_area,
            search_widget,

            _table_status_bar: table_status_bar,
            table_status_bar_line_counter_label,

            table_name,
            table_uuid,
            dependency_data: Arc::new(RwLock::new(dependency_data)),
            table_definition: Arc::new(RwLock::new(table_definition)),
            data_source,
            packed_file_path: packed_file_path.clone(),
            packed_file_type: Arc::new(packed_file_type),
            banned_table,

            undo_lock,
            save_lock,

            undo_model,
            history_undo: Arc::new(RwLock::new(vec![])),
            history_redo: Arc::new(RwLock::new(vec![])),

            timer_delayed_updates,
        });

        let packed_file_table_view_slots = TableViewSlots::new(
            &packed_file_table_view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            packed_file_path.clone()
        );

        // Build the first filter.
        FilterView::new(&packed_file_table_view);

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be resetted to 1, 2, 3,... so we do this here.
        load_data(
            &packed_file_table_view.get_mut_ptr_table_view_primary(),
            &packed_file_table_view.get_mut_ptr_table_view_frozen(),
            &packed_file_table_view.table_definition.read().unwrap(),
            &packed_file_table_view.dependency_data,
            &table_data,
            &packed_file_table_view.timer_delayed_updates,
            packed_file_table_view.get_data_source()
        );

        // Initialize the undo model.
        update_undo_model(&packed_file_table_view.get_mut_ptr_table_model(), &packed_file_table_view.get_mut_ptr_undo_model());

        // Build the columns. If we have a model from before, use it to paint our cells as they were last time we painted them.
        let table_name = if let Some(ref path) = packed_file_path {
            path.read().unwrap().get(1).cloned()
        } else { None };

        build_columns(
            &packed_file_table_view.get_mut_ptr_table_view_primary(),
            Some(&packed_file_table_view.get_mut_ptr_table_view_frozen()),
            &packed_file_table_view.table_definition.read().unwrap(),
            table_name.as_ref()
        );

        // Set the connections and return success.
        connections::set_connections(&packed_file_table_view, &packed_file_table_view_slots);
        shortcuts::set_shortcuts(&packed_file_table_view);
        tips::set_tips(&packed_file_table_view);

        // Update the line counter.
        packed_file_table_view.update_line_counter();

        // This fixes some weird issues on first click.
        packed_file_table_view.context_menu_update();

        Ok(packed_file_table_view)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    ///
    /// NOTE: This allows for a table to change it's definition on-the-fly, so be careful with that!
    pub unsafe fn reload_view(&self, data: TableType) {
        let table_view_primary = &self.get_mut_ptr_table_view_primary();
        let table_view_frozen = &self.get_mut_ptr_table_view_frozen();
        let undo_model = &self.get_mut_ptr_undo_model();

        let filter: QPtr<QSortFilterProxyModel> = table_view_primary.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // Update the stored definition.
        let table_definition = match data {
            TableType::AnimFragment(ref table) => table.get_definition(),
            TableType::AnimTable(ref table) => table.get_definition(),
            TableType::DB(ref table) => table.get_definition(),
            TableType::Loc(ref table) => table.get_definition(),
            TableType::MatchedCombat(ref table) => table.get_definition(),
            TableType::NormalTable(ref table) => table.get_definition(),
            _ => unimplemented!(),
        };

        *self.table_definition.write().unwrap() = table_definition;

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be resetted to 1, 2, 3,... so we do this here.
        load_data(
            table_view_primary,
            table_view_frozen,
            &self.get_ref_table_definition(),
            &self.dependency_data,
            &data,
            &self.timer_delayed_updates,
            self.get_data_source()
        );

        // Prepare the diagnostic pass.
        self.start_delayed_updates_timer();
        self.update_line_counter();

        // Reset the undo model and the undo/redo history.
        update_undo_model(&model, undo_model);
        self.history_undo.write().unwrap().clear();
        self.history_redo.write().unwrap().clear();

        let table_name = if let Some(path) = self.get_packed_file_path() {
            path.get(1).cloned()
        } else { None };

        // Rebuild the column's stuff.
        build_columns(
            table_view_primary,
            Some(table_view_frozen),
            &self.get_ref_table_definition(),
            table_name.as_ref()
        );

        // Rebuild the column list of the filter and search panels, just in case the definition changed.
        // NOTE: We need to lock the signals for the column selector so it doesn't try to trigger in the middle of the rebuild, causing a deadlock.
        for filter in self.get_ref_mut_filters().iter() {
            let _filter_blocker = QSignalBlocker::from_q_object(filter.filter_column_selector.static_upcast::<QObject>());
            filter.filter_column_selector.clear();
            for column in self.table_definition.read().unwrap().get_fields_sorted() {
                let name = QString::from_std_str(&utils::clean_column_names(column.get_name()));
                filter.filter_column_selector.add_item_q_string(&name);
            }
        }

        let search_column_selector = &self.search_column_selector;
        search_column_selector.clear();
        search_column_selector.add_item_q_string(&QString::from_std_str("* (All Columns)"));
        for column in self.table_definition.read().unwrap().get_fields_sorted() {
            let name = QString::from_std_str(&utils::clean_column_names(column.get_name()));
            search_column_selector.add_item_q_string(&name);
        }

        // Reset this setting so the last column gets resized properly.
        table_view_primary.horizontal_header().set_stretch_last_section(!SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"]);
        table_view_primary.horizontal_header().set_stretch_last_section(SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"]);
    }

    /// This function returns a reference to the StandardItemModel widget.
    pub unsafe fn get_mut_ptr_table_model(&self) -> QPtr<QStandardItemModel> {
        self.table_model.static_upcast()
    }

    /// This function returns a reference to the table's model.
    pub unsafe fn get_ref_table_model(&self) -> &QBox<QStandardItemModel> {
        &self.table_model
    }

    // This function returns a mutable reference to the `Enable Lookups` Pushbutton.
    //pub fn get_mut_ptr_enable_lookups_button(&self) -> QPtr<QPushButton> {
    //    q_ptr_from_atomic(&self.table_enable_lookups_button)
    //}

    /// This function returns a pointer to the Primary TableView widget.
    pub unsafe fn get_mut_ptr_table_view_primary(&self) -> QPtr<QTableView> {
        self.table_view_primary.static_upcast()
    }

    /// This function returns a pointer to the Frozen TableView widget.
    pub unsafe fn get_mut_ptr_table_view_frozen(&self) -> QPtr<QTableView> {
        self.table_view_frozen.static_upcast()
    }

    pub unsafe fn get_mut_ptr_table_view_filter(&self) -> QPtr<QSortFilterProxyModel> {
        self.table_filter.static_upcast()
    }

    /// This function returns a pointer to the filter's LineEdit widget.
    pub unsafe fn get_mut_ptr_filter_base_widget(&self) -> QPtr<QWidget> {
        self.filter_base_widget.static_upcast()
    }

    /// This function returns a pointer to the add rows action.
    pub fn get_mut_ptr_context_menu_add_rows(&self) -> &QPtr<QAction> {
        &self.context_menu_add_rows
    }

    /// This function returns a pointer to the insert rows action.
    pub fn get_mut_ptr_context_menu_insert_rows(&self) -> &QPtr<QAction> {
        &self.context_menu_insert_rows
    }

    /// This function returns a pointer to the delete rows action.
    pub fn get_mut_ptr_context_menu_delete_rows(&self) -> &QPtr<QAction> {
        &self.context_menu_delete_rows
    }

    /// This function returns a pointer to the delete rows not in filter action.
    pub fn get_mut_ptr_context_menu_delete_rows_not_in_filter(&self) -> &QPtr<QAction> {
        &self.context_menu_delete_rows_not_in_filter
    }

    /// This function returns a pointer to the clone_and_append action.
    pub fn get_mut_ptr_context_menu_clone_and_append(&self) -> &QPtr<QAction> {
        &self.context_menu_clone_and_append
    }

    /// This function returns a pointer to the clone_and_insert action.
    pub fn get_mut_ptr_context_menu_clone_and_insert(&self) -> &QPtr<QAction> {
        &self.context_menu_clone_and_insert
    }

    /// This function returns a pointer to the copy action.
    pub fn get_mut_ptr_context_menu_copy(&self) -> &QPtr<QAction> {
        &self.context_menu_copy
    }

    /// This function returns a pointer to the copy as lua table action.
    pub fn get_mut_ptr_context_menu_copy_as_lua_table(&self) -> &QPtr<QAction> {
        &self.context_menu_copy_as_lua_table
    }

    /// This function returns a pointer to the paste action.
    pub fn get_mut_ptr_context_menu_paste(&self) -> &QPtr<QAction> {
        &self.context_menu_paste
    }

    /// This function returns a pointer to the paste as new row action.
    pub fn get_mut_ptr_context_menu_paste_as_new_row(&self) -> &QPtr<QAction> {
        &self.context_menu_paste_as_new_row
    }

    /// This function returns a pointer to the invert selection action.
    pub fn get_mut_ptr_context_menu_invert_selection(&self) -> &QPtr<QAction> {
        &self.context_menu_invert_selection
    }

    /// This function returns a pointer to the reset selection action.
    pub fn get_mut_ptr_context_menu_reset_selection(&self) -> &QPtr<QAction> {
        &self.context_menu_reset_selection
    }

    /// This function returns a pointer to the rewrite selection action.
    pub fn get_mut_ptr_context_menu_rewrite_selection(&self) -> &QPtr<QAction> {
        &self.context_menu_rewrite_selection
    }

    /// This function returns a pointer to the fill ids action.
    pub fn get_mut_ptr_context_menu_generate_ids(&self) -> &QPtr<QAction> {
        &self.context_menu_generate_ids
    }

    /// This function returns a pointer to the undo action.
    pub fn get_mut_ptr_context_menu_undo(&self) -> &QPtr<QAction> {
        &self.context_menu_undo
    }

    /// This function returns a pointer to the redo action.
    pub fn get_mut_ptr_context_menu_redo(&self) -> &QPtr<QAction> {
        &self.context_menu_redo
    }

    /// This function returns a pointer to the import TSV action.
    pub fn get_mut_ptr_context_menu_import_tsv(&self) -> &QPtr<QAction> {
        &self.context_menu_import_tsv
    }

    /// This function returns a pointer to the export TSV action.
    pub fn get_mut_ptr_context_menu_export_tsv(&self) -> &QPtr<QAction> {
        &self.context_menu_export_tsv
    }

    /// This function returns a pointer to the smart delete action.
    pub fn get_mut_ptr_smart_delete(&self) -> &QBox<QAction> {
        &self.smart_delete
    }

    /// This function returns a pointer to the resize columns action.
    pub fn get_mut_ptr_context_menu_resize_columns(&self) -> &QPtr<QAction> {
        &self.context_menu_resize_columns
    }

    /// This function returns a pointer to the sidebar action.
    pub fn get_mut_ptr_context_menu_sidebar(&self) -> &QPtr<QAction> {
        &self.context_menu_sidebar
    }

    /// This function returns a pointer to the search action.
    pub fn get_mut_ptr_context_menu_search(&self) -> &QPtr<QAction> {
        &self.context_menu_search
    }

    /// This function returns a pointer to the cascade edition action.
    pub fn get_mut_ptr_context_menu_cascade_edition(&self) -> &QPtr<QAction> {
        &self.context_menu_cascade_edition
    }

    /// This function returns a pointer to the patch column action.
    pub fn get_mut_ptr_context_menu_patch_column(&self) -> &QPtr<QAction> {
        &self.context_menu_patch_column
    }

    /// This function returns a pointer to the go to definition action.
    pub fn get_mut_ptr_context_menu_go_to_definition(&self) -> &QPtr<QAction> {
        &self.context_menu_go_to_definition
    }

    /// This function returns a vector with the entire go to loc action list.
    pub fn get_go_to_loc_actions(&self) -> &[QPtr<QAction>] {
        &self.context_menu_go_to_loc
    }

    /// This function returns a vector with the entire hide/show checkbox list.
    pub fn get_hide_show_checkboxes(&self) -> &[QBox<QCheckBox>] {
        &self.sidebar_hide_checkboxes
    }

    /// This function returns the checkbox to hide them all.
    pub fn get_hide_show_checkboxes_all(&self) -> &QBox<QCheckBox> {
        &self.sidebar_hide_checkboxes_all
    }

    /// This function returns a vector with the entire freeze checkbox list.
    pub fn get_freeze_checkboxes(&self) -> &[QBox<QCheckBox>] {
        &self.sidebar_freeze_checkboxes
    }

    /// This function returns a the checkbox to freeze them all.
    pub fn get_freeze_checkboxes_all(&self) -> &QBox<QCheckBox> {
        &self.sidebar_freeze_checkboxes_all
    }

    /// This function returns a pointer to the search lineedit in the search panel.
    pub fn get_mut_ptr_search_search_line_edit(&self) -> &QBox<QLineEdit> {
        &self.search_search_line_edit
    }

    /// This function returns a pointer to the search button in the search panel.
    pub fn get_mut_ptr_search_search_button(&self) -> &QBox<QPushButton> {
        &self.search_search_button
    }

    /// This function returns a pointer to the prev match button in the search panel.
    pub fn get_mut_ptr_search_prev_match_button(&self) -> &QBox<QPushButton> {
        &self.search_prev_match_button
    }

    /// This function returns a pointer to the next_match button in the search panel.
    pub fn get_mut_ptr_search_next_match_button(&self) -> &QBox<QPushButton> {
        &self.search_next_match_button
    }

    /// This function returns a pointer to the replace current button in the search panel.
    pub fn get_mut_ptr_search_replace_current_button(&self) -> &QBox<QPushButton> {
        &self.search_replace_current_button
    }

    /// This function returns a pointer to the replace all button in the search panel.
    pub fn get_mut_ptr_search_replace_all_button(&self) -> &QBox<QPushButton> {
        &self.search_replace_all_button
    }

    /// This function returns a pointer to the close button in the search panel.
    pub fn get_mut_ptr_search_close_button(&self) -> &QBox<QPushButton> {
        &self.search_close_button
    }

    pub unsafe fn get_mut_ptr_undo_model(&self) -> QPtr<QStandardItemModel> {
        self.undo_model.static_upcast()
    }

    /// This function returns a reference to this table's name.
    pub fn get_ref_table_name(&self) -> &Option<String> {
        &self.table_name
    }

    /// This function returns a reference to this table's uuid.
    pub fn get_ref_table_uuid(&self) -> &Option<String> {
        &self.table_uuid
    }

    /// This function returns a reference to the definition of this table.
    pub fn get_ref_table_definition(&self) -> RwLockReadGuard<Definition> {
        self.table_definition.read().unwrap()
    }

    pub fn get_ref_filters(&self) -> RwLockReadGuard<Vec<Arc<FilterView>>> {
        self.filters.read().unwrap()
    }

    pub fn get_ref_mut_filters(&self) -> RwLockWriteGuard<Vec<Arc<FilterView>>> {
        self.filters.write().unwrap()
    }

    /// This function allows you to set a new dependency data to an already created table.
    pub fn set_dependency_data(&self, data: &BTreeMap<i32, DependencyData>) {
        *self.dependency_data.write().unwrap() = data.clone();
    }

    /// This function returns the path of the PackedFile corresponding to this table, if exists.
    pub fn get_packed_file_path(&self) -> Option<Vec<String>> {
        self.packed_file_path.as_ref().map(|path| path.read().unwrap().clone())
    }

    /// This function returns a copy of the datasource of this table.
    pub fn get_data_source(&self) -> DataSource {
        self.data_source.read().unwrap().clone()
    }

    pub unsafe fn start_delayed_updates_timer(&self) {
        self.timer_delayed_updates.set_interval(1500);
        self.timer_delayed_updates.start_0a();
    }

    pub unsafe fn update_line_counter(&self) {
        let rows_on_filter = self.table_filter.row_count_0a().to_string();
        let rows_on_model = self.table_model.row_count_0a().to_string();
        self.table_status_bar_line_counter_label.set_text(&qtre("line_counter", &[&rows_on_filter, &rows_on_model]));
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
            Self::Editing(items) => Self::Editing(items.iter().map(|(x, y)| (*x, atomic_from_ptr(ptr_from_atomic(y)))).collect()),
            Self::AddRows(rows) => Self::AddRows(rows.to_vec()),
            Self::RemoveRows(rows) => Self::RemoveRows(rows.iter()
                .map(|(x, y)| (*x, y.iter()
                    .map(|y| y.iter()
                        .map(|z| atomic_from_ptr(ptr_from_atomic(z)))
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
        definition: &Definition,
        flags: QFlags<MatchFlag>,
        column: i32
    ) {

        // First, check the column type. Boolean columns need special logic, as they cannot be matched by string.
        let is_bool = definition.get_fields_processed()[column as usize].get_ref_field_type() == &FieldType::Boolean;
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

    /// This function takes care of updating the UI to reflect changes in the table search.
    pub unsafe fn update_search_ui(parent: &TableView, update_type: TableSearchUpdate) {
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

                    parent.search_matches_label.set_text(&QString::from_std_str(&format!("{} of {} in current filter ({} in total)", table_search.current_item.unwrap() + 1, matches_in_filter.len(), matches_in_model.len())));

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

        if parent.get_data_source() != DataSource::PackFile {
            parent.search_replace_current_button.set_enabled(false);
            parent.search_replace_all_button.set_enabled(false);
        }
    }

    /// This function takes care of updating the search data whenever a change that can alter the results happens.
    pub unsafe fn update_search(parent: &TableView) {
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
                None => (0..parent.get_ref_table_definition().get_fields_processed().len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model.as_ptr(), parent.table_filter.as_ptr(), &parent.get_ref_table_definition(), flags, *column);
            }
        }

        Self::update_search_ui(parent, TableSearchUpdate::Update);
    }

    /// This function takes care of searching the patter we provided in the TableView.
    pub unsafe fn search(parent: &TableView) {
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
                else { Some(parent.get_ref_table_definition().get_fields_processed().iter().position(|x| x.get_name() == column).unwrap() as i32) }
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
                None => (0..parent.get_ref_table_definition().get_fields_processed().len()).map(|x| x as i32).collect::<Vec<i32>>(),
            };

            for column in &columns_to_search {
                table_search.find_in_column(parent.table_model.as_ptr(), parent.table_filter.as_ptr(), &parent.get_ref_table_definition(), flags, *column);
            }
        }

        Self::update_search_ui(parent, TableSearchUpdate::Search);
    }

    /// This function takes care of moving the selection to the previous match on the matches list.
    pub unsafe fn prev_match(parent: &TableView) {
        Self::update_search_ui(parent, TableSearchUpdate::PrevMatch);
    }

    /// This function takes care of moving the selection to the next match on the matches list.
    pub unsafe fn next_match(parent: &TableView) {
        Self::update_search_ui(parent, TableSearchUpdate::NextMatch);
    }

    /// This function takes care of replacing the current match with the provided replacing text.
    pub unsafe fn replace_current(parent: &TableView) {

        // NOTE: WE CANNOT HAVE THE SEARCH DATA LOCK UNTIL AFTER WE DO THE REPLACE. That's why there are a lot of read here.
        let text_source = parent.search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {

            // Get the replace data here, as we probably don't have it updated.
            parent.search_data.write().unwrap().replace = parent.search_replace_line_edit.text().into_ptr();
            let text_replace = parent.search_data.read().unwrap().replace.to_std_string();
            if text_source == text_replace { return }

            // And if we got a valid position.
            let item;
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

                    if parent.get_ref_table_definition().get_fields_processed()[model_index.column() as usize].get_ref_field_type() == &FieldType::Boolean {
                        replaced_text = text_replace;
                    }
                    else {
                        let text = item.text().to_std_string();
                        replaced_text = text.replace(&text_source, &text_replace);
                    }

                    // We need to do an extra check to ensure the new text can be in the field.
                    match parent.get_ref_table_definition().get_fields_processed()[model_index.column() as usize].get_ref_field_type() {
                        FieldType::Boolean => if parse_str_as_bool(&replaced_text).is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::F32 => if replaced_text.parse::<f32>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::I16 => if replaced_text.parse::<i16>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::I32 => if replaced_text.parse::<i32>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        FieldType::I64 => if replaced_text.parse::<i64>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                        _ =>  {}
                    }
                } else { return }
            } else { return }

            // At this point, we trigger editions. Which mean, here ALL LOCKS SHOULD HAVE BEEN ALREADY DROP.
            match parent.get_ref_table_definition().get_fields_processed()[item.column() as usize].get_ref_field_type() {
                FieldType::Boolean => item.set_check_state(if parse_str_as_bool(&replaced_text).unwrap() { CheckState::Checked } else { CheckState::Unchecked }),
                FieldType::F32 => item.set_data_2a(&QVariant::from_float(replaced_text.parse::<f32>().unwrap()), 2),
                FieldType::I16 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i16>().unwrap().into()), 2),
                FieldType::I32 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i32>().unwrap()), 2),
                FieldType::I64 => item.set_data_2a(&QVariant::from_i64(replaced_text.parse::<i64>().unwrap()), 2),
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

    /// This function takes care of replacing all the instances of a match with the provided replacing text.
    pub unsafe fn replace_all(parent: &TableView) {

        // NOTE: WE CANNOT HAVE THE SEARCH DATA LOCK UNTIL AFTER WE DO THE REPLACE. That's why there are a lot of read here.
        let text_source = parent.search_data.read().unwrap().pattern.to_std_string();
        if !text_source.is_empty() {

            // Get the replace data here, as we probably don't have it updated.
            parent.search_data.write().unwrap().replace = parent.search_replace_line_edit.text().into_ptr();
            let text_replace = parent.search_data.read().unwrap().replace.to_std_string();
            if text_source == text_replace { return }

            let mut positions_and_texts: Vec<(Ptr<QModelIndex>, String)> = vec![];
            {
                // Here is save to lock, as the lock will be drop before doing the replace.
                let table_search = &mut parent.search_data.read().unwrap();

                // Get the list of all valid ModelIndex for the current filter and the current position.
                let matches_in_model_and_filter = table_search.get_visible_matches_in_model();
                for model_index in &matches_in_model_and_filter {

                    // If the position is still valid (not required, but just in case)...
                    if model_index.is_valid() {
                        let item = parent.table_model.item_from_index(model_index.as_ref().unwrap());
                        let original_text = match parent.get_ref_table_definition().get_fields_processed()[model_index.column() as usize].get_ref_field_type() {
                            FieldType::Boolean => item.data_0a().to_bool().to_string(),
                            FieldType::F32 => item.data_0a().to_float_0a().to_string(),
                            FieldType::I16 => item.data_0a().to_int_0a().to_string(),
                            FieldType::I32 => item.data_0a().to_int_0a().to_string(),
                            FieldType::I64 => item.data_0a().to_long_long_0a().to_string(),
                            _ => item.text().to_std_string(),
                        };

                        let replaced_text = if parent.get_ref_table_definition().get_fields_processed()[model_index.column() as usize].get_ref_field_type() == &FieldType::Boolean {
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
                        match parent.get_ref_table_definition().get_fields_processed()[model_index.column() as usize].get_ref_field_type() {
                            FieldType::Boolean => if parse_str_as_bool(&replaced_text).is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                            FieldType::F32 => if replaced_text.parse::<f32>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                            FieldType::I16 => if replaced_text.parse::<i16>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                            FieldType::I32 => if replaced_text.parse::<i32>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                            FieldType::I64 => if replaced_text.parse::<i64>().is_err() { return show_dialog(&parent.table_view_primary, ErrorKind::DBTableReplaceInvalidData, false) }
                            _ =>  {}
                        }

                        positions_and_texts.push((*model_index, replaced_text));
                    } else { return }
                }
            }

            // At this point, we trigger editions. Which mean, here ALL LOCKS SHOULD HAVE BEEN ALREADY DROP.
            for (model_index, replaced_text) in &positions_and_texts {
                let item = parent.table_model.item_from_index(model_index.as_ref().unwrap());
                match parent.get_ref_table_definition().get_fields_processed()[item.column() as usize].get_ref_field_type() {
                    FieldType::Boolean => item.set_check_state(if parse_str_as_bool(replaced_text).unwrap() { CheckState::Checked } else { CheckState::Unchecked }),
                    FieldType::F32 => item.set_data_2a(&QVariant::from_float(replaced_text.parse::<f32>().unwrap()), 2),
                    FieldType::I16 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i16>().unwrap().into()), 2),
                    FieldType::I32 => item.set_data_2a(&QVariant::from_int(replaced_text.parse::<i32>().unwrap()), 2),
                    FieldType::I64 => item.set_data_2a(&QVariant::from_i64(replaced_text.parse::<i64>().unwrap()), 2),
                    _ => item.set_text(&QString::from_std_str(&replaced_text)),
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
                update_undo_model(&parent.get_mut_ptr_table_model(), &parent.get_mut_ptr_undo_model());
            }
        }
    }
}

impl FilterView {

    pub unsafe fn new(view: &Arc<TableView>) {
        let parent = view.get_mut_ptr_filter_base_widget();

        // Create the filter's widgets.
        let filter_widget = QWidget::new_1a(&parent);
        let filter_grid = create_grid_layout(filter_widget.static_upcast());
        filter_grid.set_column_stretch(0, 99);
        filter_grid.set_column_stretch(3, 0);
        filter_grid.set_column_stretch(4, 0);

        let filter_timer_delayed_updates = QTimer::new_1a(&parent);
        let filter_line_edit = QLineEdit::from_q_widget(&parent);
        let filter_column_selector = QComboBox::new_1a(&parent);
        let filter_match_group_selector = QComboBox::new_1a(&parent);
        let filter_show_blank_cells_button = QPushButton::from_q_string_q_widget(&qtr("table_filter_show_blank_cells"), &parent);
        let filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("table_filter_case_sensitive"), &parent);
        let filter_add = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &parent);
        let filter_remove = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &parent);

        // Reuse the models from the first filterview, as that one will never get destroyed.
        if let Some(first_filter) = view.get_ref_filters().get(0) {
            filter_column_selector.set_model(&first_filter.filter_column_selector.model());
            filter_match_group_selector.set_model(&first_filter.filter_match_group_selector.model());
        }

        else {
            let filter_match_group_list = QStandardItemModel::new_1a(&filter_match_group_selector);
            let filter_column_list = QStandardItemModel::new_1a(&filter_column_selector);

            filter_column_selector.set_model(&filter_column_list);
            filter_match_group_selector.set_model(&filter_match_group_list);

            let fields = view.get_ref_table_definition().get_fields_sorted();
            for field in &fields {
                let name = clean_column_names(field.get_name());
                filter_column_selector.add_item_q_string(&QString::from_std_str(&name));
            }

            filter_match_group_selector.add_item_q_string(&QString::from_std_str(&format!("{} {}", tr("filter_group"), 1)));
        }

        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        filter_case_sensitive_button.set_checkable(true);
        filter_show_blank_cells_button.set_checkable(true);
        filter_timer_delayed_updates.set_single_shot(true);

        // The first filter must never be deleted.
        if view.get_ref_filters().get(0).is_none() {
            filter_remove.set_enabled(false);
        }

        // Add everything to the grid.
        filter_grid.add_widget_5a(&filter_line_edit, 0, 0, 1, 3);
        filter_grid.add_widget_5a(&filter_match_group_selector, 0, 3, 1, 1);
        filter_grid.add_widget_5a(&filter_case_sensitive_button, 0, 4, 1, 1);
        filter_grid.add_widget_5a(&filter_show_blank_cells_button, 0, 5, 1, 1);
        filter_grid.add_widget_5a(&filter_column_selector, 0, 6, 1, 1);
        filter_grid.add_widget_5a(&filter_add, 0, 9, 1, 1);
        filter_grid.add_widget_5a(&filter_remove, 0, 10, 1, 1);

        let parent_grid: QPtr<QGridLayout> = parent.layout().static_downcast();
        parent_grid.add_widget_5a(&filter_widget, view.get_ref_filters().len() as i32 + 3, 0, 1, 2);

        let filter = Arc::new(Self {
            filter_widget,
            filter_line_edit,
            filter_match_group_selector,
            filter_case_sensitive_button,
            filter_show_blank_cells_button,
            filter_column_selector,
            filter_timer_delayed_updates,
            filter_add,
            filter_remove,
        });

        let slots = FilterViewSlots::new(&filter, view);

        connections::set_connections_filter(&filter, &slots);

        view.get_ref_mut_filters().push(filter);
    }

    pub unsafe fn start_delayed_updates_timer(view: &Arc<Self>) {
        view.filter_timer_delayed_updates.set_interval(500);
        view.filter_timer_delayed_updates.start_0a();
    }

    pub unsafe fn add_filter_group(view: &Arc<TableView>) {
        if view.get_ref_filters()[0].filter_match_group_selector.count() < view.get_ref_filters().len() as i32 {
            let name = QString::from_std_str(&format!("{} {}", tr("filter_group"), view.get_ref_filters()[0].filter_match_group_selector.count() + 1));
            view.get_ref_filters()[0].filter_match_group_selector.add_item_q_string(&name);
        }
    }
}
