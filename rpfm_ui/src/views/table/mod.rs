//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::QActionGroup;
use qt_widgets::QAction;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTableView;
use qt_widgets::QTextEdit;
use qt_widgets::QScrollArea;
use qt_widgets::QSpinBox;
use qt_widgets::QWidget;

use qt_gui::QGuiApplication;
use qt_gui::QIcon;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::AlignmentFlag;
use qt_core::CaseSensitivity;
use qt_core::CheckState;
use qt_core::Orientation;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QItemSelection;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSignalBlocker;
use qt_core::QSignalMapper;
use qt_core::QSortFilterProxyModel;
use qt_core::QStringList;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SlotNoArgs;

use qt_ui_tools::QUiLoader;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;

use anyhow::{anyhow, Result};
use getset::Getters;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, BufWriter, Write};
use std::{fmt, fmt::Debug};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::rc::Rc;

use rpfm_extensions::dependencies::TableReferences;

use rpfm_lib::files::{ContainerPath, FileType, db::DB, loc::Loc, table::{*, local::TableInMemory}};
use rpfm_lib::integrations::log::error;
use rpfm_lib::schema::{Definition, Field, FieldType, Schema};

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::locale::{qtr, qtre, tr};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::GAME_SELECTED;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, utils::set_modified, View, ViewType};
use crate::pack_tree::*;
use crate::QVARIANT_FALSE;
use crate::QVARIANT_TRUE;
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::*;
use crate::SCHEMA;
use crate::UI_STATE;
use crate::utils::*;

use self::filter::*;
use self::search::*;
use self::slots::*;
use self::utils::*;

mod connections;
pub mod filter;
mod search;
pub mod slots;
pub mod utils;

// Column default sizes.
pub static COLUMN_SIZE_BOOLEAN: i32 = 100;
pub static COLUMN_SIZE_NUMBER: i32 = 140;
pub static COLUMN_SIZE_STRING: i32 = 350;

pub static ITEM_IS_KEY: i32 = 20;
pub static ITEM_IS_ADDED: i32 = 21;
pub static ITEM_IS_MODIFIED: i32 = 22;
pub static ITEM_IS_ADDED_VS_VANILLA: i32 = 23;
pub static ITEM_IS_MODIFIED_VS_VANILLA: i32 = 24;
pub static ITEM_HAS_ERROR: i32 = 25;
pub static ITEM_HAS_WARNING: i32 = 26;
pub static ITEM_HAS_INFO: i32 = 27;
pub static ITEM_HAS_SOURCE_VALUE: i32 = 30;
pub static ITEM_SOURCE_VALUE: i32 = 31;
pub static ITEM_HAS_VANILLA_VALUE: i32 = 32;
pub static ITEM_VANILLA_VALUE: i32 = 33;
pub static ITEM_IS_SEQUENCE: i32 = 35;
pub static ITEM_SEQUENCE_DATA: i32 = 36;

pub static ITEM_SUB_DATA: i32 = 40;
pub static ITEM_ICON_CACHE: i32 = 50;
pub static ITEM_ICON_PATH: i32 = 52;

const PATCH_COLUMN_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_schema_patch_dialog.ui";
const PATCH_COLUMN_VIEW_RELEASE: &str = "ui/new_schema_patch_dialog.ui";

const NEW_PROFILE_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_table_view_profile_dialog.ui";
const NEW_PROFILE_VIEW_RELEASE: &str = "ui/new_table_view_profile_dialog.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum is used to distinguish between the different types of tables we can decode.
#[derive(Clone, Debug)]
pub enum TableType {
    AnimFragmentBattle(TableInMemory),
    Atlas(TableInMemory),
    DependencyManager(Vec<Vec<DecodedData>>),
    DB(DB),
    Loc(Loc),

    /// This one is for random views that just need a table with advanced behavior.
    NormalTable(TableInMemory),

    /// This one is for the translator view.
    #[cfg(feature = "enable_tools")] TranslatorTable(TableInMemory),
}

/// Enum to know what operation was done while editing tables, so we can revert them with undo.
pub enum TableOperations {

    /// Intended for any kind of item editing. Holds a `Vec<((row, column), AtomicPtr<item>)>`, so we can do this in batches.
    Editing(Vec<((i32, i32), AtomicPtr<QStandardItem>)>),

    /// Intended for when adding/inserting rows. It holds a list of positions where the rows where inserted.
    AddRows(Vec<i32>),

    /// Intended for when removing rows. It holds a list of positions where the rows where deleted and the deleted rows data, in consecutive batches.
    RemoveRows(Vec<(i32, Vec<Vec<AtomicPtr<QStandardItem>>>)>),

    // It holds a copy of the entire table, before importing, and its definition, because it may change.
    //ImportTSV(Vec<AtomicPtr<QListOfQStandardItem>>),

    /// A Jack-of-all-Trades. It holds a `Vec<TableOperations>`, for those situations one is not enough.
    Carolina(Vec<TableOperations>),
}

/// This struct contains pointers to all the widgets in a Table View.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct TableView {
    table_view: QBox<QTableView>,
    table_filter: QBox<QSortFilterProxyModel>,
    table_model: QBox<QStandardItemModel>,

    filter_base_widget: QBox<QWidget>,
    #[getset(skip)]
    filters: Arc<RwLock<Vec<Arc<FilterView>>>>,

    #[getset(skip)]
    column_sort_state: Arc<RwLock<(i32, i8)>>,

    signal_mapper_profile_apply: QBox<QSignalMapper>,
    signal_mapper_profile_delete: QBox<QSignalMapper>,
    signal_mapper_profile_set_as_default: QBox<QSignalMapper>,

    context_menu: QBox<QMenu>,
    context_menu_add_rows: QPtr<QAction>,
    context_menu_insert_rows: QPtr<QAction>,
    context_menu_delete_rows: QPtr<QAction>,
    context_menu_delete_rows_not_in_filter: QPtr<QAction>,
    context_menu_clone_and_append: QPtr<QAction>,
    context_menu_clone_and_insert: QPtr<QAction>,
    context_menu_copy: QPtr<QAction>,
    context_menu_copy_as_lua_table: QPtr<QAction>,
    context_menu_copy_to_filter_value: QPtr<QAction>,
    context_menu_paste: QPtr<QAction>,
    context_menu_paste_as_new_row: QPtr<QAction>,
    context_menu_invert_selection: QPtr<QAction>,
    context_menu_reset_selection: QPtr<QAction>,
    context_menu_rewrite_selection: QPtr<QAction>,
    context_menu_revert_value: QPtr<QAction>,
    context_menu_generate_ids: QPtr<QAction>,
    context_menu_profiles_apply: QBox<QMenu>,
    context_menu_profiles_delete: QBox<QMenu>,
    context_menu_profiles_set_as_default: QBox<QMenu>,
    context_menu_profiles_create: QPtr<QAction>,
    context_menu_undo: QPtr<QAction>,
    context_menu_redo: QPtr<QAction>,
    context_menu_import_tsv: QPtr<QAction>,
    context_menu_export_tsv: QPtr<QAction>,
    context_menu_resize_columns: QPtr<QAction>,
    context_menu_sidebar: QPtr<QAction>,
    context_menu_search: QPtr<QAction>,
    context_menu_find_references: QPtr<QAction>,
    context_menu_cascade_edition: QPtr<QAction>,
    context_menu_patch_column: QPtr<QAction>,
    context_menu_smart_delete: QPtr<QAction>,

    _context_menu_go_to: QBox<QMenu>,
    context_menu_go_to_definition: QPtr<QAction>,
    context_menu_go_to_file: QPtr<QAction>,
    context_menu_go_to_loc: Vec<QPtr<QAction>>,

    sidebar_scroll_area: QBox<QScrollArea>,

    sidebar_hide_checkboxes: Vec<QBox<QCheckBox>>,
    sidebar_hide_checkboxes_all: QBox<QCheckBox>,
    sidebar_freeze_checkboxes: Vec<QBox<QCheckBox>>,
    sidebar_freeze_checkboxes_all: QBox<QCheckBox>,

    _table_status_bar: QBox<QWidget>,
    table_status_bar_line_counter_label: QBox<QLabel>,

    #[getset(skip)]
    search_view: Arc<RwLock<Option<Arc<SearchView>>>>,

    table_name: Option<String>,
    is_translator: bool,
    vanilla_hashed_tables: Arc<RwLock<Vec<(DB, HashMap<String, i32>)>>>,

    #[getset(skip)]
    data_source: Arc<RwLock<DataSource>>,
    #[getset(skip)]
    packed_file_path: Option<Arc<RwLock<String>>>,
    packed_file_type: Arc<FileType>,

    #[getset(skip)]
    table_definition: Arc<RwLock<Definition>>,
    #[getset(skip)]
    dependency_data: Arc<RwLock<HashMap<i32, TableReferences>>>,

    banned_table: bool,

    #[getset(skip)]
    reference_map: Arc<RwLock<HashMap<String, HashMap<String, Vec<String>>>>>,

    profile_default: Arc<RwLock<String>>,
    profiles: Arc<RwLock<HashMap<String, TableViewProfile>>>,

    save_lock: Arc<AtomicBool>,
    undo_lock: Arc<AtomicBool>,

    undo_model: QBox<QStandardItemModel>,

    #[getset(skip)]
    history_undo: Arc<RwLock<Vec<TableOperations>>>,

    #[getset(skip)]
    history_redo: Arc<RwLock<Vec<TableOperations>>>,

    timer_delayed_updates: QBox<QTimer>,
}

/// This struct contains data to load a specific status of a view.
#[derive(Debug, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TableViewProfile {
    column_order: Vec<i32>,
    columns_hidden: Vec<i32>,
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
        references_ui: &Rc<ReferencesUI>,
        table_data: TableType,
        packed_file_path: Option<Arc<RwLock<String>>>,
        data_source: Arc<RwLock<DataSource>>,
    ) -> Result<Arc<Self>> {

        let (table_definition, table_name, packed_file_type, is_translator) = match table_data {
            TableType::AnimFragmentBattle(ref table) => (table.definition().clone(), None, FileType::AnimFragmentBattle, false),
            TableType::Atlas(ref table) => (table.definition().clone(), None, FileType::Atlas, false),
            TableType::DependencyManager(_) => {
                let mut definition = Definition::new(-1, None);
                definition.fields_mut().push(Field::new("Load before ingame?".to_owned(), FieldType::Boolean, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
                definition.fields_mut().push(Field::new("Parent Packs".to_owned(), FieldType::StringU8, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
                (definition, None, FileType::Unknown, false)
            },
            TableType::DB(ref table) => (table.definition().clone(), Some(table.table_name()), FileType::DB, false),
            TableType::Loc(ref table) => (table.definition().clone(), None, FileType::Loc, false),
            TableType::NormalTable(ref table) => (table.definition().clone(), None, FileType::Unknown, false),
            #[cfg(feature = "enable_tools")] TableType::TranslatorTable(ref table) => (table.definition().clone(), None, FileType::Unknown, true),
        };

        // Get the dependency data of this Table.
        let table_name_for_ref = if let Some(name) = table_name { name.to_owned() } else { "".to_owned() };
        let dependency_data = get_reference_data(packed_file_type, &table_name_for_ref, &table_definition, false)?;

        // Do not bother getting hashed data for tables that are not modded.
        let vanilla_hashed_tables = {
            let data_source = data_source.read().unwrap();
            if *data_source == DataSource::PackFile || *data_source == DataSource::ParentFiles {
                get_vanilla_hashed_tables(packed_file_type, &table_name_for_ref).unwrap_or_default()
            } else {
                vec![]
            }
        };

        // Create the locks for undoing and saving. These are needed to optimize the undo/saving process.
        let undo_lock = Arc::new(AtomicBool::new(false));
        let save_lock = Arc::new(AtomicBool::new(false));

        // Prepare the Table and its model.
        let table_view = new_tableview_frozen_safe(&parent.as_ptr(), generate_tooltip_message);
        let table_filter = new_tableview_filter_safe(parent.static_upcast());
        let table_model = QStandardItemModel::new_1a(parent);
        let undo_model = QStandardItemModel::new_1a(parent);
        table_filter.set_source_model(&table_model);
        table_view.set_model(&table_filter);

        // Make the last column fill all the available space, if the setting says so.
        if setting_bool("extend_last_column_on_tables") {
            table_view.horizontal_header().set_stretch_last_section(true);
        }

        // Setup tight mode if the setting is enabled.
        if setting_bool("tight_table_mode") {
            table_view.vertical_header().set_minimum_section_size(22);
            table_view.vertical_header().set_maximum_section_size(22);
            table_view.vertical_header().set_default_section_size(22);
        }

        // Create the filter's widgets.
        let filter_base_widget = QWidget::new_1a(parent);
        let _filter_base_grid = create_grid_layout(filter_base_widget.static_upcast());

        // Add everything to the grid.
        let layout: QPtr<QGridLayout> = parent.layout().static_downcast();

        let mut banned_table = false;
        if let TableType::DependencyManager(_) = table_data {
            let warning_message = QLabel::from_q_string_q_widget(&qtr("dependency_packfile_list_label"), parent);
            layout.add_widget_5a(&warning_message, 0, 0, 1, 4);
        } else if let FileType::DB = packed_file_type {
            banned_table = GAME_SELECTED.read().unwrap().is_file_banned(&format!("db/{}", &table_name_for_ref));
            if banned_table {
                let warning_message = QLabel::from_q_string_q_widget(&qtr("banned_tables_warning"), parent);
                layout.add_widget_5a(&warning_message, 0, 0, 1, 4);
            }
        }

        let table_status_bar = QWidget::new_1a(parent);
        let table_status_bar_grid = create_grid_layout(table_status_bar.static_upcast());
        let table_status_bar_line_counter_label = QLabel::from_q_string_q_widget(&qtre("line_counter", &["0", "0"]), &table_status_bar);
        table_status_bar_grid.add_widget_5a(&table_status_bar_line_counter_label, 0, 0, 1, 1);

        layout.add_widget_5a(&table_view, 1, 0, 1, 1);
        layout.add_widget_5a(&table_status_bar, 2, 0, 1, 2);
        layout.add_widget_5a(&filter_base_widget, 4, 0, 1, 2);

        // Action to make the delete button delete contents.
        let context_menu_smart_delete = add_action_to_widget(app_ui.shortcuts().as_ref(), "table_editor", "smart_delete", Some(table_view.static_upcast::<qt_widgets::QWidget>()));

        // Create the Contextual Menu for the TableView.
        let signal_mapper_profile_apply = QSignalMapper::new_1a(&table_view);
        let signal_mapper_profile_delete = QSignalMapper::new_1a(&table_view);
        let signal_mapper_profile_set_as_default = QSignalMapper::new_1a(&table_view);
        let context_menu = QMenu::from_q_widget(&table_view);
        let context_menu_add_rows = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "add_row", "context_menu_add_rows", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_insert_rows = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "insert_row", "context_menu_insert_rows", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_delete_rows = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "delete_row", "context_menu_delete_rows", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_delete_rows_not_in_filter = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "delete_filtered_out_row", "context_menu_delete_filtered_out_rows", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_clone_submenu = QMenu::from_q_string_q_widget(&qtr("context_menu_clone_submenu"), &table_view);
        let context_menu_clone_and_insert = add_action_to_menu(&context_menu_clone_submenu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "clone_and_insert_row", "context_menu_clone_and_insert", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_clone_and_append = add_action_to_menu(&context_menu_clone_submenu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "clone_and_append_row", "context_menu_clone_and_append", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_copy_submenu = QMenu::from_q_string_q_widget(&qtr("context_menu_copy_submenu"), &table_view);
        let context_menu_copy = add_action_to_menu(&context_menu_copy_submenu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "copy", "context_menu_copy", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_copy_as_lua_table = add_action_to_menu(&context_menu_copy_submenu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "copy_as_lua_table", "context_menu_copy_as_lua_table", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_copy_to_filter_value = add_action_to_menu(&context_menu_copy_submenu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "copy_as_filter_value", "context_menu_copy_to_filter_value", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_paste = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "paste", "context_menu_paste", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_paste_as_new_row = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "paste_as_new_row", "context_menu_paste_as_new_row", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_generate_ids = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "generate_ids", "context_menu_generate_ids", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_rewrite_selection = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "rewrite_selection", "context_menu_rewrite_selection", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_revert_value = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "revert_value", "context_menu_revert_value", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_invert_selection = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "invert_selection", "context_menu_invert_selection", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_reset_selection = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "reset_selected_values", "context_menu_reset_selection", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_resize_columns = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "resize_columns", "context_menu_resize_columns", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_import_tsv = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "import_tsv", "context_menu_import_tsv", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_export_tsv = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "export_tsv", "context_menu_export_tsv", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_search = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "search", "context_menu_search", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_sidebar = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "sidebar", "context_menu_sidebar", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_find_references = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "find_references", "context_menu_find_references", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_cascade_edition = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "rename_references", "context_menu_cascade_edition", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_patch_column = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "patch_columns", "context_menu_patch_column", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_profiles_apply = QMenu::from_q_string_q_widget(&qtr("context_menu_profiles_apply"), &table_view);
        let context_menu_profiles_delete = QMenu::from_q_string_q_widget(&qtr("context_menu_profiles_delete"), &table_view);
        let context_menu_profiles_set_as_default = QMenu::from_q_string_q_widget(&qtr("context_menu_profiles_set_as_default"), &table_view);
        let context_menu_profiles_create = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "create_profile", "context_menu_profiles_create", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_undo = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "undo", "context_menu_undo", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_redo = add_action_to_menu(&context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "redo", "context_menu_redo", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_go_to = QMenu::from_q_string_q_widget(&qtr("context_menu_go_to"), &table_view);
        let context_menu_go_to_definition = add_action_to_menu(&context_menu_go_to.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "go_to_definition", "context_menu_go_to_definition", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_go_to_file = add_action_to_menu(&context_menu_go_to.static_upcast(), app_ui.shortcuts().as_ref(), "table_editor", "go_to_file", "context_menu_go_to_file", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let mut context_menu_go_to_loc = vec![];

        for (index, loc_column) in table_definition.localised_fields().iter().enumerate() {
            let context_menu_go_to_loc_action = context_menu_go_to.add_action_q_string(&qtre("context_menu_go_to_loc", &[loc_column.name()]));
            if index == 0 { context_menu_go_to.insert_separator(&context_menu_go_to_loc_action); }
            context_menu_go_to_loc.push(context_menu_go_to_loc_action)
        }

        // Insert some separators to space the menu, and the paste submenu.
        context_menu.insert_menu(&context_menu_paste, &context_menu_clone_submenu);
        context_menu.insert_menu(&context_menu_paste, &context_menu_copy_submenu);
        context_menu.insert_menu(&context_menu_paste, &context_menu_go_to);
        context_menu.insert_separator(&context_menu_profiles_create);
        context_menu.insert_menu(&context_menu_profiles_create, &context_menu_profiles_apply);
        context_menu.insert_menu(&context_menu_profiles_create, &context_menu_profiles_delete);
        context_menu.insert_menu(&context_menu_profiles_create, &context_menu_profiles_set_as_default);
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

        let fields = table_definition.fields_processed_sorted(setting_bool("tables_use_old_column_order"));
        for column in &fields {
            search_column_selector.add_item_q_string(&QString::from_std_str(utils::clean_column_names(column.name())));
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
            let column_name = QLabel::from_q_string_q_widget(&QString::from_std_str(utils::clean_column_names(column.name())), &sidebar_widget);
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

        // Get the reference data for this table, to speedup reference searching.
        let reference_map = if let TableType::NormalTable(_) = table_data {
            HashMap::new()
        } else if let Some(schema) = &*SCHEMA.read().unwrap() {
            if let Some(table_name) = table_name {
                schema.referencing_columns_for_table(table_name, &table_definition)
            } else {
                HashMap::new()
            }
        } else {
            return Err(anyhow!("There is no Schema for the Game Selected."));
        };

        // Create the raw Struct and begin
        let packed_file_table_view = Arc::new(TableView {
            table_view,
            table_filter,
            table_model,
            filters: Arc::new(RwLock::new(vec![])),
            filter_base_widget,
            column_sort_state: Arc::new(RwLock::new((-1, 0))),

            signal_mapper_profile_apply,
            signal_mapper_profile_delete,
            signal_mapper_profile_set_as_default,
            context_menu,
            context_menu_add_rows,
            context_menu_insert_rows,
            context_menu_delete_rows,
            context_menu_delete_rows_not_in_filter,
            context_menu_clone_and_append,
            context_menu_clone_and_insert,
            context_menu_copy,
            context_menu_copy_as_lua_table,
            context_menu_copy_to_filter_value,
            context_menu_paste,
            context_menu_paste_as_new_row,
            context_menu_invert_selection,
            context_menu_reset_selection,
            context_menu_rewrite_selection,
            context_menu_revert_value,
            context_menu_generate_ids,
            context_menu_profiles_apply,
            context_menu_profiles_delete,
            context_menu_profiles_set_as_default,
            context_menu_profiles_create,
            context_menu_undo,
            context_menu_redo,
            context_menu_import_tsv,
            context_menu_export_tsv,
            context_menu_resize_columns,
            context_menu_sidebar,
            context_menu_search,
            context_menu_find_references,
            context_menu_cascade_edition,
            context_menu_patch_column,
            context_menu_smart_delete,

            _context_menu_go_to: context_menu_go_to,
            context_menu_go_to_definition,
            context_menu_go_to_file,
            context_menu_go_to_loc,

            sidebar_hide_checkboxes,
            sidebar_hide_checkboxes_all,
            sidebar_freeze_checkboxes,
            sidebar_freeze_checkboxes_all,

            sidebar_scroll_area,

            _table_status_bar: table_status_bar,
            table_status_bar_line_counter_label,

            search_view: Arc::new(RwLock::new(None)),

            table_name: table_name.map(|x| x.to_owned()),
            is_translator,
            dependency_data: Arc::new(RwLock::new(dependency_data)),
            table_definition: Arc::new(RwLock::new(table_definition)),
            vanilla_hashed_tables: Arc::new(RwLock::new(vanilla_hashed_tables)),
            data_source,
            packed_file_path: packed_file_path.clone(),
            packed_file_type: Arc::new(packed_file_type),
            banned_table,
            reference_map: Arc::new(RwLock::new(reference_map)),
            profile_default: Arc::new(RwLock::new(String::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),

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
            references_ui,
            packed_file_path.clone()
        );

        // Build the first filter.
        FilterView::new(&packed_file_table_view)?;
        SearchView::new(&packed_file_table_view)?;

        packed_file_table_view.load_table_view_profiles()?;

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be resetted to 1, 2, 3,... so we do this here.
        load_data(
            &packed_file_table_view.table_view_ptr(),
            &packed_file_table_view.table_definition.read().unwrap(),
            packed_file_table_view.table_name.as_deref(),
            &packed_file_table_view.dependency_data,
            &table_data,
            &packed_file_table_view.timer_delayed_updates,
            packed_file_table_view.get_data_source(),
            &packed_file_table_view.vanilla_hashed_tables.read().unwrap()
        );

        // Before applying the profile, check the relevant sidebar checks.
        for visual_index in 0..packed_file_table_view.table_view.horizontal_header().length() {
            let logical_index = packed_file_table_view.table_view.horizontal_header().logical_index(visual_index);
            if packed_file_table_view.table_view.horizontal_header().is_section_hidden(logical_index) {
                packed_file_table_view.sidebar_hide_checkboxes()[logical_index as usize].set_checked(true);
            }
        }

        // If we have a default profile, apply it.
        if !packed_file_table_view.profile_default().read().unwrap().is_empty() {
            packed_file_table_view.apply_table_view_profile(&packed_file_table_view.profile_default().read().unwrap());
        }

        // Initialize the undo model.
        update_undo_model(&packed_file_table_view.table_model_ptr(), &packed_file_table_view.undo_model_ptr());

        // Set the connections and return success.
        connections::set_connections(&packed_file_table_view, &packed_file_table_view_slots);

        // Update the line counter.
        packed_file_table_view.update_line_counter();

        // This fixes some weird issues on first click.
        packed_file_table_view.context_menu_update();

        Ok(packed_file_table_view)
    }

    pub unsafe fn apply_table_view_profile(&self, key: &str) {
        let profiles = self.profiles.read().unwrap();
        if let Some(profile) = profiles.get(key) {
            let header = self.table_view.horizontal_header();

            // Block signals so the header doesn't trigger weird things while doing this.
            header.block_signals(true);

            // Column order && hidden columns. Remember to set the sidebar status accordingly.
            for (dest_index, logical_index) in profile.column_order.iter().enumerate() {
                let visual_index = header.visual_index(*logical_index);
                header.move_section(visual_index, dest_index as i32);

                if let Some(checkbox) = self.sidebar_hide_checkboxes().get(*logical_index as usize) {
                    checkbox.set_checked(profile.columns_hidden.contains(logical_index));
                }
            }

            header.block_signals(false);
        }

        // We need to update the view afterwards. Otherwise it'll be stuck with old data due to signals not updating it.
        self.timer_delayed_updates.set_interval(5);
        self.timer_delayed_updates.start_0a();
    }

    pub unsafe fn delete_table_view_profile(&self, key: &str) {
        self.profiles.write().unwrap().remove(key);

        // Reload all profiles in the UI.
        self.load_profiles_to_context_menu();
    }

    pub unsafe fn new_table_view_profile(&self, key: &str) {
        let mut profile = TableViewProfile::default();

        // Column order.
        let header = self.table_view().horizontal_header();
        profile.column_order = (0..header.count())
            .map(|visual_index| header.logical_index(visual_index))
            .collect::<Vec<_>>();

        profile.columns_hidden = (0..header.count())
            .map(|visual_index| header.logical_index(visual_index))
            .filter(|logical_index| header.is_section_hidden(*logical_index))
            .collect::<Vec<_>>();

        self.profiles.write().unwrap().insert(key.to_owned(), profile);

        // Reload all profiles in the UI.
        self.load_profiles_to_context_menu();
    }

    pub unsafe fn load_table_view_profiles(&self) -> Result<()> {
        if let Some(ref table_name) = self.table_name {
            let game = GAME_SELECTED.read().unwrap();
            let mut profiles = self.profiles.write().unwrap();

            let profiles_path = table_profiles_path()?.join(game.key());
            if !profiles_path.is_dir() {
                DirBuilder::new().recursive(true).create(&profiles_path)?;
            }

            let profiles_file_name = format!("table_view_profiles_{table_name}.json");
            let path = profiles_path.join(profiles_file_name);
            if path.is_file() {
                let mut file = BufReader::new(File::open(path)?);
                let mut data = vec![];
                file.read_to_end(&mut data)?;

                let profiles_data: HashMap<String, String> = serde_json::from_slice(&data)?;
                for (key, value) in &profiles_data {
                    if key == "profile_default" {
                        *self.profile_default.write().unwrap() = value.to_owned();
                    } else {
                        profiles.insert(key.to_owned(), serde_json::from_str(value)?);
                    }
                }
            }
        }

        // Once loaded, put them in the ui.
        self.load_profiles_to_context_menu();

        Ok(())
    }

    pub unsafe fn load_profiles_to_context_menu(&self) {
        self.context_menu_profiles_apply.clear();
        self.context_menu_profiles_delete.clear();
        self.context_menu_profiles_set_as_default.clear();

        let default_group = QActionGroup::new(&self.context_menu_profiles_set_as_default);
        let profiles = self.profiles.read().unwrap();
        for key in profiles.keys() {
            let apply = self.context_menu_profiles_apply.add_action_q_string(&QString::from_std_str(key));
            let delete = self.context_menu_profiles_delete.add_action_q_string(&QString::from_std_str(key));
            let set_as_default = self.context_menu_profiles_set_as_default.add_action_q_string(&QString::from_std_str(key));
            set_as_default.set_checkable(true);
            default_group.add_action_q_action(&set_as_default);

            apply.triggered().connect(self.signal_mapper_profile_apply.slot_map());
            self.signal_mapper_profile_apply.set_mapping_q_object_q_string(apply, &QString::from_std_str(key));

            delete.triggered().connect(self.signal_mapper_profile_delete.slot_map());
            self.signal_mapper_profile_delete.set_mapping_q_object_q_string(delete, &QString::from_std_str(key));

            set_as_default.toggled().connect(self.signal_mapper_profile_set_as_default.slot_map());
            self.signal_mapper_profile_set_as_default.set_mapping_q_object_q_string(set_as_default, &QString::from_std_str(key));
        }
    }

    pub unsafe fn save_table_view_profiles(&self) -> Result<()> {
        if let Some(ref table_name) = self.table_name {
            let mut profiles_data = self.profiles.read().unwrap().iter()
                .map(|(key, value)| (key.to_owned(), serde_json::to_string_pretty(value).unwrap()))
                .collect::<HashMap<String, String>>();

            let profile_default = self.profile_default().read().unwrap();
            if !profile_default.is_empty() {
                profiles_data.insert("profile_default".to_owned(), profile_default.to_string());
            }

            let game = GAME_SELECTED.read().unwrap();
            let profiles_path = table_profiles_path()?.join(game.key());
            if !profiles_path.is_dir() {
                DirBuilder::new().recursive(true).create(&profiles_path)?;
            }

            let profiles_file_name = format!("table_view_profiles_{table_name}.json");
            let path = profiles_path.join(profiles_file_name);
            let mut file = BufWriter::new(File::create(path)?);
            file.write_all(serde_json::to_string_pretty(&profiles_data)?.as_bytes())?;
        }

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    ///
    /// NOTE: This allows for a table to change it's definition on-the-fly, so be careful with that!
    pub unsafe fn reload_view(&self, data: TableType) {
        let table_view = &self.table_view_ptr();
        let undo_model = &self.undo_model_ptr();

        let filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();

        // Update the stored definition.
        let table_definition = match data {
            TableType::AnimFragmentBattle(ref table) => table.definition().clone(),
            TableType::Atlas(ref table) => table.definition().clone(),
            TableType::DB(ref table) => table.definition().clone(),
            TableType::Loc(ref table) => table.definition().clone(),
            TableType::NormalTable(ref table) => table.definition().clone(),
            #[cfg(feature = "enable_tools")] TableType::TranslatorTable(ref table) => table.definition().clone(),
            TableType::DependencyManager(_) => {
                let mut definition = Definition::new(-1, None);
                definition.fields_mut().push(Field::new("Load before ingame?".to_owned(), FieldType::Boolean, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
                definition.fields_mut().push(Field::new("Parent Packs".to_owned(), FieldType::StringU8, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
                definition
            }
        };

        *self.table_definition.write().unwrap() = table_definition;

        // Load the data to the Table. For some reason, if we do this after setting the titles of
        // the columns, the titles will be resetted to 1, 2, 3,... so we do this here.
        load_data(
            table_view,
            &self.table_definition(),
            self.table_name.as_deref(),
            &self.dependency_data,
            &data,
            &self.timer_delayed_updates,
            self.get_data_source(),
            &self.vanilla_hashed_tables.read().unwrap()
        );

        // Before applying the profile, check the relevant sidebar checks.
        for visual_index in 0..self.table_view.horizontal_header().length() {
            let logical_index = self.table_view.horizontal_header().logical_index(visual_index);
            if self.table_view.horizontal_header().is_section_hidden(logical_index) {
                self.sidebar_hide_checkboxes()[logical_index as usize].set_checked(true);
            }
        }

        // If we have a default profile, apply it.
        if !self.profile_default().read().unwrap().is_empty() {
            self.apply_table_view_profile(&self.profile_default().read().unwrap());
        }

        // Prepare the diagnostic pass.
        self.start_delayed_updates_timer();

        // Reset the undo model and the undo/redo history.
        update_undo_model(&model, undo_model);
        self.history_undo.write().unwrap().clear();
        self.history_redo.write().unwrap().clear();

        // Rebuild the column list of the filter and search panels, just in case the definition changed.
        // NOTE: We need to lock the signals for the column selector so it doesn't try to trigger in the middle of the rebuild, causing a deadlock.
        for filter in self.filters_mut().iter() {
            filter.column_combobox().block_signals(true);
            filter.column_combobox().model().block_signals(true);
            filter.column_combobox().clear();

            for column in self.table_definition.read().unwrap().fields_processed_sorted(setting_bool("tables_use_old_column_order")) {
                let name = QString::from_std_str(utils::clean_column_names(column.name()));
                filter.column_combobox().add_item_q_string(&name);
            }
            filter.column_combobox().block_signals(false);
            filter.column_combobox().model().block_signals(false);
        };

        if let Some(search_view) = &*self.search_view() {
            search_view.reload(self);
        }

        // Reset this setting so the last column gets resized properly.
        table_view.horizontal_header().set_stretch_last_section(!setting_bool("extend_last_column_on_tables"));
        table_view.horizontal_header().set_stretch_last_section(setting_bool("extend_last_column_on_tables"));
    }

    /// This function returns a reference to the StandardItemModel widget.
    pub unsafe fn table_model_ptr(&self) -> QPtr<QStandardItemModel> {
        self.table_model.static_upcast()
    }

    /// This function returns a pointer to the Primary TableView widget.
    pub unsafe fn table_view_ptr(&self) -> QPtr<QTableView> {
        self.table_view.static_upcast()
    }

    pub unsafe fn table_view_filter_ptr(&self) -> QPtr<QSortFilterProxyModel> {
        self.table_filter.static_upcast()
    }

    /// This function returns a pointer to the filter's LineEdit widget.
    pub unsafe fn filter_base_widget_ptr(&self) -> QPtr<QWidget> {
        self.filter_base_widget.static_upcast()
    }

    pub unsafe fn undo_model_ptr(&self) -> QPtr<QStandardItemModel> {
        self.undo_model.static_upcast()
    }

    pub fn get_packed_file_type(&self) -> &FileType {
        &self.packed_file_type
    }

    /// This function returns a reference to the definition of this table.
    pub fn table_definition(&self) -> RwLockReadGuard<Definition> {
        self.table_definition.read().unwrap()
    }

    pub fn search_view(&self) -> RwLockReadGuard<Option<Arc<SearchView>>> {
        self.search_view.read().unwrap()
    }

    pub fn filters(&self) -> RwLockReadGuard<Vec<Arc<FilterView>>> {
        self.filters.read().unwrap()
    }

    pub fn filters_mut(&self) -> RwLockWriteGuard<Vec<Arc<FilterView>>> {
        self.filters.write().unwrap()
    }

    /// This function allows you to set a new dependency data to an already created table.
    pub fn set_dependency_data(&self, data: &HashMap<i32, TableReferences>) {
        *self.dependency_data.write().unwrap() = data.clone();
    }

    /// This function returns the path of the PackedFile corresponding to this table, if exists.
    pub fn get_packed_file_path(&self) -> Option<String> {
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

    //----------------------------------------------------------------//
    //----------------------------------------------------------------//
    //----------------------------------------------------------------//

    /// This function updates the state of the actions in the context menu.
    pub unsafe fn context_menu_update(&self) {

        // Disable everything, just in case.
        self.context_menu_add_rows.set_enabled(false);
        self.context_menu_insert_rows.set_enabled(false);
        self.context_menu_clone_and_append.set_enabled(false);
        self.context_menu_clone_and_insert.set_enabled(false);
        self.context_menu_delete_rows.set_enabled(false);
        self.context_menu_delete_rows_not_in_filter.set_enabled(false);
        self.context_menu_paste.set_enabled(false);
        self.context_menu_paste_as_new_row.set_enabled(false);
        self.context_menu_rewrite_selection.set_enabled(false);
        self.context_menu_revert_value.set_enabled(false);
        self.context_menu_generate_ids.set_enabled(false);
        self.context_menu_undo.set_enabled(false);
        self.context_menu_redo.set_enabled(false);
        self.context_menu_import_tsv.set_enabled(false);
        self.context_menu_find_references.set_enabled(false);
        self.context_menu_cascade_edition.set_enabled(false);
        self.context_menu_patch_column.set_enabled(true);
        self.context_menu_smart_delete.set_enabled(false);

        // Turns out that this slot doesn't give the the amount of selected items, so we have to get them ourselves.
        let indexes = self.table_filter.map_selection_to_source(&self.table_view.selection_model().selection()).indexes();

        // If we have something selected, enable these actions.
        if indexes.count_0a() > 0 {
            self.context_menu_copy.set_enabled(true);
            self.context_menu_copy_as_lua_table.set_enabled(true);

            if *self.packed_file_type == FileType::DB {
                self.context_menu_find_references.set_enabled(true);
                self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(true));
            } else {
                self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(false));
            }

            if *self.packed_file_type == FileType::Loc {
                self.context_menu_go_to_definition.set_enabled(true);
                self.context_menu_go_to_file.set_enabled(false);
            } else if *self.packed_file_type == FileType::DB {

                // Go to Definition and Go To File only should be enabled if we actually are in a field where they'll work.
                let columns = (0..indexes.count_0a()).map(|x| indexes.at(x).column()).collect::<HashSet<_>>();
                let table_definition = self.table_definition();
                let schema_patches = table_definition.patches();
                let fields_processed = table_definition.fields_processed();

                self.context_menu_go_to_definition.set_enabled(columns.iter()
                    .filter_map(|x| fields_processed.get(*x as usize))
                    .any(|y| y.is_reference(Some(schema_patches)).is_some()));

                self.context_menu_go_to_file.set_enabled(columns.iter()
                    .filter_map(|x| fields_processed.get(*x as usize))
                    .any(|y| y.is_filename(Some(schema_patches))));
            } else {
                self.context_menu_go_to_definition.set_enabled(false);
                self.context_menu_go_to_file.set_enabled(false);
            }
        }

        // Otherwise, disable them.
        else {
            self.context_menu_copy.set_enabled(false);
            self.context_menu_copy_as_lua_table.set_enabled(false);
            self.context_menu_go_to_definition.set_enabled(false);
            self.context_menu_go_to_file.set_enabled(false);
            self.context_menu_go_to_loc.iter().for_each(|x| x.set_enabled(false));
        }

        // Only enable editing if the table is ours and not banned.
        if let DataSource::PackFile = self.get_data_source() {
            if !self.banned_table && !self.is_translator {

                // These ones are always enabled if the table is editable.
                self.context_menu_add_rows.set_enabled(true);
                self.context_menu_insert_rows.set_enabled(true);
                self.context_menu_delete_rows_not_in_filter.set_enabled(true);
                self.context_menu_paste_as_new_row.set_enabled(true);
                self.context_menu_import_tsv.set_enabled(true);
                self.context_menu_smart_delete.set_enabled(true);

                // If we have something selected, enable these actions.
                if indexes.count_0a() > 0 {
                    self.context_menu_clone_and_append.set_enabled(true);
                    self.context_menu_clone_and_insert.set_enabled(true);
                    self.context_menu_delete_rows.set_enabled(true);
                    self.context_menu_paste.set_enabled(true);
                    self.context_menu_rewrite_selection.set_enabled(true);
                    self.context_menu_revert_value.set_enabled(true);
                    self.context_menu_generate_ids.set_enabled(true);
                    self.context_menu_cascade_edition.set_enabled(true);
                }

                if !self.undo_lock.load(Ordering::SeqCst) {
                    self.context_menu_undo.set_enabled(!self.history_undo.read().unwrap().is_empty());
                    self.context_menu_redo.set_enabled(!self.history_redo.read().unwrap().is_empty());
                }
            }
        }
    }

    /// Function to filter the table.
    pub unsafe fn filter_table(&self) {
        let mut columns = vec![];
        let mut patterns = vec![];
        let mut sensitivity = vec![];
        let mut use_nott = vec![];
        let mut use_regex = vec![];
        let mut show_blank_cells = vec![];
        let mut match_groups = vec![];
        let mut variant_to_search = vec![];
        let mut show_edited_cells = vec![];

        let filters = self.filters.read().unwrap();
        for filter in filters.iter() {

            // Replace jumplines with ors, and then filter.
            filter.filter_line_edit().block_signals(true);
            let text = QString::from_std_str(filter.filter_line_edit().text().to_std_string().replace("\r\n", "|").replace('\n', "|"));
            filter.filter_line_edit().undo();
            filter.filter_line_edit().select_all();
            filter.filter_line_edit().insert(&text);
            filter.filter_line_edit().block_signals(false);

            // Ignore empty filters.
            if !filter.filter_line_edit().text().to_std_string().is_empty() {

                let column_name = filter.column_combobox().current_text();
                for column in 0..self.table_model.column_count_0a() {
                    if self.table_model.header_data_2a(column, Orientation::Horizontal).to_string().compare_q_string_case_sensitivity(&column_name, CaseSensitivity::CaseSensitive) == 0 {
                        columns.push(column);
                        break;
                    }
                }

                // Check if the filter should be "Case Sensitive".
                let case_sensitive = filter.case_sensitive_button().is_checked();
                if case_sensitive { sensitivity.push(CaseSensitivity::CaseSensitive); }
                else { sensitivity.push(CaseSensitivity::CaseInsensitive); }

                // Check for regex.
                use_regex.push(filter.use_regex_button().is_checked());

                // Check if we should filter out blank cells or not.
                show_blank_cells.push(filter.show_blank_cells_button().is_checked());

                // Check if we should filter out edited cells or not.
                show_edited_cells.push(filter.show_edited_cells_button().is_checked());

                let pattern = filter.filter_line_edit().text().to_std_string();
                use_nott.push(filter.not_checkbox().is_checked());

                patterns.push(QString::from_std_str(pattern).into_ptr());
                match_groups.push(filter.group_combobox().current_index());

                variant_to_search.push(filter.variant_combobox().current_index());
            }
        }

        // Filter whatever it's in that column by the text we got.
        trigger_tableview_filter_safe(&self.table_filter, &columns, patterns, &use_nott, &use_regex, &sensitivity, &show_blank_cells, &match_groups, &variant_to_search, &show_edited_cells);

        // Update the line count.
        self.update_line_counter();

        // Update the left-side header. This is to workaround a bug that causes said header to vanish on filter.
        self.table_view().vertical_header().set_visible(false);
        self.table_view().vertical_header().set_visible(true);
    }

    /// This function resets the currently selected cells to their original value.
    pub unsafe fn reset_selection(&self) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());

        let mut items_reverted = 0;
        for index in &indexes_sorted {
            if index.is_valid() {
                let item = self.table_model.item_from_index(index);
                if item.data_1a(ITEM_HAS_SOURCE_VALUE).to_bool() {
                    let original_data = item.data_1a(ITEM_SOURCE_VALUE);
                    let current_data = item.data_1a(2);
                    if original_data != current_data.as_ref() {
                        item.set_data_2a(&original_data, 2);
                        items_reverted += 1;
                    }
                }
            }
        }

        // Fix the undo history to have all the previous changed merged into one.
        if items_reverted > 0 {
            {
                let mut history_undo = self.history_undo.write().unwrap();
                let mut history_redo = self.history_redo.write().unwrap();

                let len = history_undo.len();
                let mut edits_data = vec![];
                {
                    let mut edits = history_undo.drain((len - items_reverted)..);
                    for edit in &mut edits {
                        if let TableOperations::Editing(mut edit) = edit {
                            edits_data.append(&mut edit);
                        }
                    }
                }

                history_undo.push(TableOperations::Editing(edits_data));
                history_redo.clear();
            }
            self.start_delayed_updates_timer();
            update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
        }
    }

    /// This function rewrite the currently selected cells using the provided formula.
    pub unsafe fn rewrite_selection(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if let Some((is_math_operation, value)) = self.create_rewrite_selection_dialog() {
            let horizontal_header = self.table_view.horizontal_header();

            // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
            let indexes = self.table_view.selection_model().selection().indexes();
            let indexes_sorted = get_visible_selection_sorted(&indexes, &self.table_view_ptr());

            let mut real_cells = vec![];
            let mut values = vec![];

            let mut row = 0;
            let mut prev_row = None;
            for index in &indexes_sorted {
                if index.is_valid() {

                    // Row depends on the selection. If none, it's the first row.
                    if prev_row.is_none() {
                        prev_row = Some(index.row());
                    }

                    // If row changed, + 1.
                    if let Some(ref mut prev_row) = prev_row {
                        if *prev_row != index.row() {
                            *prev_row = index.row();
                            row += 1;
                        }
                    }

                    // Get the column of that cell, the row, the current value, and the new value.
                    let item = self.table_model.item_from_index(&self.table_filter.map_to_source(*index));
                    let column = horizontal_header.visual_index(index.column());
                    let current_value = item.text().to_std_string();
                    let new_value = value.replace("{x}", &current_value).replace("{X}", &current_value)
                        .replace("{y}", &column.to_string()).replace("{Y}", &column.to_string())
                        .replace("{z}", &row.to_string()).replace("{Z}", &row.to_string());

                    let text = if is_math_operation {
                         if let Ok(result) = meval::eval_str(&new_value) {

                            // If we got a current value and it's different, it's a valid cell.
                            match current_value.parse::<f64>() {
                                Ok(value) => {
                                    if (result - value).abs() >= f64::EPSILON {
                                        result.to_string()
                                    } else {
                                        current_value.to_owned()
                                    }
                                },
                                Err(_) => result.to_string(),
                            }
                        }

                        // If meval fails, it's not a valid operation for this cell
                        else { continue; }
                    } else { new_value.to_owned() };

                    real_cells.push(self.table_filter.map_to_source(*index));
                    values.push(text);
                }
            }

            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), &*values[index]));
            }
            realer_cells.reverse();

            let fields_processed = self.table_definition().fields_processed();
            self.set_data_on_cells(&realer_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);
        }
    }

    /// This function reverts the currently selected cells to their vanilla/parent values, if we have them.
    pub unsafe fn revert_values(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Only do this if we actually have vanilla data to revert to.
        let vanilla_data = self.vanilla_hashed_tables.read().unwrap();
        if vanilla_data.is_empty() {
            return;
        }

        let mut real_cells = vec![];
        let mut values = vec![];

        // Same as with the data: don't bother if we don't have keys.
        let definition = self.table_definition();
        let fields_processed = definition.fields_processed();
        let key_pos = definition.key_column_positions();
        if key_pos.is_empty() {
            return;
        }

        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());
        for index in indexes_sorted {
            if index.is_valid() {
                let mut value = None;

                // Only edit cells that are actually different from vanilla.
                let item = self.table_model.item_from_index(&index);
                if item.data_1a(ITEM_IS_MODIFIED_VS_VANILLA).to_bool() {

                    let keys_joined = key_pos.iter()
                        .map(|x| self.table_model.index_2a(item.row(), *x as i32).data_1a(2).to_string().to_std_string())
                        .join("");

                    let field = &fields_processed[item.column() as usize];
                    for (vanilla_table, hashes) in &*vanilla_data {
                        if let Some(row) = hashes.get(&keys_joined) {
                            let local_data = get_field_from_view(&self.table_model.static_upcast(), field, item.row(), item.column());

                            if let Some(vanilla_data) = vanilla_table.data()[*row as usize].get(item.column() as usize) {
                                if vanilla_data != &local_data {
                                    value = Some(vanilla_data.data_to_string().to_string());
                                }

                                break;
                            }
                        }
                    }
                }

                real_cells.push(index);
                values.push(value);
            }
        }

        let mut realer_cells = vec![];
        for index in (0..real_cells.len()).rev() {
            let cell = real_cells.pop().unwrap();
            if let Some(ref value) = values[index] {
                realer_cells.push((cell, &**value));
            }
        }
        realer_cells.reverse();

        let fields_processed = self.table_definition().fields_processed();
        self.set_data_on_cells(&realer_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);
    }

    /// This function fills the currently provided cells with a set of ids.
    pub unsafe fn generate_ids(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes = self.table_view.selection_model().selection().indexes();
        let indexes_sorted = get_visible_selection_sorted(&indexes, &self.table_view_ptr());

        // Get the initial value of the dialog.
        let initial_value = if let Some(first) = indexes_sorted.first() {
            if first.is_valid() {
                self.table_filter.map_to_source(*first).data_0a().to_string().to_std_string().parse::<i64>().unwrap_or_default()
            } else { 0 }
        } else { 0 };

        let fields_processed = self.table_definition().fields_processed();
        let is_i64 = indexes_sorted.iter().map(|x| x.column()).all(|x| fields_processed[x as usize].field_type() == &FieldType::I64 || fields_processed[x as usize].field_type() == &FieldType::OptionalI64);

        if let Some(value) = self.create_generate_ids_dialog(initial_value, is_i64) {
            let mut real_cells = vec![];
            let mut values = vec![];

            for (id, index) in indexes_sorted.iter().enumerate() {
                if index.is_valid() {
                    real_cells.push(self.table_filter.map_to_source(*index));
                    if is_i64 {
                        values.push((value + id as i64).to_string());
                    } else {
                        values.push((value as i32 + id as i32).to_string());
                    }
                }
            }

            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), &*values[index]));
            }
            realer_cells.reverse();

            self.set_data_on_cells(&realer_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);
        }
    }

    /// This function copies the selected cells into the clipboard as a TSV file, so you can paste them in other programs.
    pub unsafe fn copy_selection(&self) {

        // Get the current selection. As we need his visual order, we get it directly from the table/filter, NOT FROM THE MODEL.
        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());

        // Create a string to keep all the values in a TSV format (x\tx\tx) and populate it.
        let mut copy = String::new();
        let mut row = 0;
        let fields_processed = self.table_definition.read().unwrap().fields_processed();
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
                if fields_processed[model_index.column() as usize].field_type() == &FieldType::Boolean {
                    match item.check_state() {
                        CheckState::Checked => copy.push_str("true"),
                        CheckState::Unchecked => copy.push_str("false"),
                        _ => return
                    }
                }

                // Fix for weird precision issues on copy.
                else if fields_processed[model_index.column() as usize].field_type() == &FieldType::F32 {
                    copy.push_str(&format!("{:.4}", item.data_1a(2).to_float_0a()));
                }
                else { copy.push_str(&QString::to_std_string(&item.text())); }

                // Add a \t to separate fields except if it's the last field.
                if cycle < (indexes_sorted.len() - 1) { copy.push('\t'); }
            }
        }

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(copy));
    }

    /// This function copies the selected cells into the clipboard as a LUA Table, so you can use it in LUA scripts.
    pub unsafe fn copy_selection_as_lua_table(&self) {

        // Get the selection sorted visually.
        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());
        let definition = self.table_definition();
        let fields_processed = definition.fields_processed();
        let patches = Some(definition.patches());

        // Check if the table has duplicated keys, and filter out invalid indexes.
        let mut has_unique_keys = true;
        let mut processed: HashMap<i32, HashSet<String>> = HashMap::new();
        let mut indexes: Vec<_> = vec![];
        for index_sorted in &indexes_sorted {
            let row_index = index_sorted.row();
            let column_index = index_sorted.column();
            if row_index != -1 && column_index != -1 {
                indexes.push(index_sorted.as_ref());
                let data = index_sorted.data_0a().to_string().to_std_string();

                let column_data = processed.get_mut(&index_sorted.column());
                let has_key = fields_processed[index_sorted.column() as usize].is_key(patches);
                if has_key {
                    match column_data {
                        Some(column_data) => {
                            if column_data.get(&data).is_some() {
                                has_unique_keys = false;
                            }
                            column_data.insert(data);
                        },
                        None => {
                            let mut column_data = HashSet::new();
                            column_data.insert(data);
                            processed.insert(index_sorted.column(), column_data);
                        },
                    }
                }
            }
        }

        let lua_table = self.get_indexes_as_lua_table(&indexes, has_unique_keys);

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(lua_table));

        // This can take time, show a message on the status bar.
        log_to_status_bar("Table copied as LUA Table.");
    }

    /// This function copies the selected cells into the clipboard as a filterable string.
    pub unsafe fn copy_selection_to_filter(&self) {

        // Get the selection sorted visually.
        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());

        // Check if the table has duplicated keys, and filter out invalid indexes.
        let mut string = String::new();
        for index_sorted in &indexes_sorted {
            let row_index = index_sorted.row();
            let column_index = index_sorted.column();
            if row_index != -1 && column_index != -1 {
                let data = index_sorted.data_0a().to_string().to_std_string();
                if !data.is_empty() {
                    string.push_str(&data);
                    string.push('|');
                }
            }
        }

        string.pop();

        // Put the baby into the oven.
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(string));
    }

    /// This function allow us to paste the contents of the clipboard into new rows at the end of the table, if the content is compatible with them.
    pub unsafe fn paste_as_new_row(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let mut text = QGuiApplication::clipboard().text().to_std_string();
        if text.ends_with('\n') { text.pop(); }
        let rows = text.split('\n').collect::<Vec<&str>>();
        let rows = rows.iter().map(|x| x.split('\t').collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

        // Then paste the data as it fits. If no indexes are provided, the data is pasted in new rows.
        self.paste_as_it_fits(&rows, &[], app_ui, pack_file_contents_ui);
    }

    /// This function allow us to paste the contents of the clipboard into the selected cells, if the content is compatible with them.
    ///
    /// This function has some... tricky stuff:
    /// - There are several special behaviors when pasting, in order to provide an Excel-Like pasting experience.
    pub unsafe fn paste(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the current selection. We treat it like a TSV, for compatibility with table editors.
        // Also, if the text ends in \n, remove it. Excel things.
        let mut text = QGuiApplication::clipboard().text().to_std_string();
        if text.ends_with('\n') { text.pop(); }
        let rows = text.split('\n').collect::<Vec<&str>>();
        let rows = rows.iter().map(|x| x.split('\t').collect::<Vec<&str>>()).collect::<Vec<Vec<&str>>>();

        // Get the current selection and his, visually speaking, first item (top-left).
        let indexes = self.table_view.selection_model().selection().indexes();
        let indexes_sorted = get_visible_selection_sorted(&indexes, &self.table_view_ptr());

        // If nothing is selected, got back to where you came from.
        if indexes_sorted.is_empty() { return }

        // At this point we should have the strings to paste and the selection. Now, clever pasting ahead:
        // - If the entire selection are rows of the same amount of cells and we have only one row of text with the exact same amount
        //   of items as the rows, we paste the same row in each selected row.
        // - If we only have one TSV value in the text and a ton of cells selected, paste the same value everywhere.
        // - In any other case, pick the first selected cell, and paste the TSV using that as cell 0,0.
        let same_amount_of_cells_selected_per_row = if rows.len() == 1 {
            let mut row = -1;
            let mut items = 0;
            let mut is_valid = true;
            for index in &indexes_sorted {
                if row == -1 {
                    row = index.row();
                }

                if index.row() == row {
                    items += 1;
                } else {
                    if items < rows[0].len() {
                        is_valid = false;
                        break;
                    }
                    row = index.row();
                    items = 1
                }

                if items > rows[0].len() {
                    is_valid = false;
                    break;
                }
            }
            is_valid
        } else { false };

        // Amount of rows selected, to ensure certain behavior only triggers when we got the correct number of rows selected.
        let mut rows_selected = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();
        rows_selected.sort_unstable();
        rows_selected.dedup();

        if rows.len() == 1 && rows[0].len() == 1 {
            self.paste_one_for_all(rows[0][0], &indexes_sorted, app_ui, pack_file_contents_ui);
        }

        else if rows.len() == 1 && same_amount_of_cells_selected_per_row && rows_selected.len() > 1 {
            self.paste_same_row_for_all(&rows[0], &indexes_sorted, app_ui, pack_file_contents_ui);
        }

        else {
            self.paste_as_it_fits(&rows, &indexes_sorted, app_ui, pack_file_contents_ui);
        }
    }

    /// This function pastes the value in the clipboard in every selected Cell.
    unsafe fn paste_one_for_all(&self, text: &str, indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let real_cells = indexes.iter().map(|index| {
            (self.table_filter.map_to_source(*index), text)
        }).collect::<Vec<(CppBox<QModelIndex>, &str)>>();

        let fields_processed = self.table_definition().fields_processed();
        self.set_data_on_cells(&real_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);
    }

    /// This function pastes the row in the clipboard in every selected row that has the same amount of items selected as items in the clipboard we have.
    unsafe fn paste_same_row_for_all(&self, text: &[&str], indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let mut column = 0;
        let real_cells = indexes.iter().filter_map(|index| {
            if index.column() == -1 {
                None
            } else {
                let data = text.get(column).map(|text| (self.table_filter.map_to_source(*index), *text));

                if column == text.len() - 1 {
                    column = 0;
                } else {
                    column += 1;
                }

                data
            }
        }).collect::<Vec<(CppBox<QModelIndex>, &str)>>();

        let fields_processed = self.table_definition().fields_processed();
        self.set_data_on_cells(&real_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);
    }

    /// This function pastes the provided text into the table as it fits, following a square strategy starting in the first selected index.
    unsafe fn paste_as_it_fits(&self, text: &[Vec<&str>], indexes: &[Ref<QModelIndex>], app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // We're going to try and check in square mode. That means, start in the selected cell, then right
        // until we reach a \n, then return to the initial column. Due to how sorting works, we have to do
        // a test pass first and get all the real AND VALID indexes, then try to paste on them.
        let horizontal_header = self.table_view.horizontal_header();
        let vertical_header = self.table_view.vertical_header();

        // Get the base index of the square. If no index is being provided, we assume we have to paste all in new rows.
        let (base_index_visual, mut visual_row) = if !indexes.is_empty() {
            (Some(&indexes[0]), vertical_header.visual_index(indexes[0].row()))
        } else {
            (None, self.table_model.row_count_0a())
        };

        let definition = self.table_definition();
        let fields_processed = definition.fields_processed();

        let mut real_cells = vec![];
        let mut added_rows = 0;
        for row in text {
            let mut visual_column = match base_index_visual {
                Some(base_index_visual) => horizontal_header.visual_index(base_index_visual.column()),
                None => 0,
            };

            let mut real_row = self.table_filter.map_to_source(&self.table_filter.index_2a(visual_row, visual_column)).row();

            for text in row {

                // Ignore hidden columns.
                let mut found = true;
                while horizontal_header.is_section_hidden(horizontal_header.logical_index(visual_column)) {
                    visual_column += 1;

                    if visual_column as usize == fields_processed.len() {
                        found = false;
                        break;
                    }
                }

                // If we found no visible columns to the end of the table, stop pasting this line.
                if !found {
                    break;
                }

                // Depending on the column, we try to encode the data in one format or another, or we just skip it.
                let real_column = horizontal_header.logical_index(visual_column);

                if let Some(field) = fields_processed.get(real_column as usize) {

                    // Check if, according to the definition, we have a valid value for the type.
                    let is_valid_data = match field.field_type() {
                        FieldType::Boolean => !(text.to_lowercase() != "true" && text.to_lowercase() != "false" && text != &"1" && text != &"0"),
                        FieldType::F32 => text.parse::<f32>().is_ok(),
                        FieldType::F64 => text.parse::<f64>().is_ok(),
                        FieldType::I16 => text.parse::<i16>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::I32 => text.parse::<i32>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::I64 => text.parse::<i64>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::OptionalI16 => text.parse::<i16>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::OptionalI32 => text.parse::<i32>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::OptionalI64 => text.parse::<i64>().is_ok() || text.parse::<f32>().is_ok(),
                        FieldType::ColourRGB => u32::from_str_radix(text, 16).is_ok(),

                        // All these are Strings, so we can skip their checks....
                        FieldType::StringU8 |
                        FieldType::StringU16 |
                        FieldType::OptionalStringU8 |
                        FieldType::OptionalStringU16 => true,

                        // Ignore sequences.
                        FieldType::SequenceU16(_) | FieldType::SequenceU32(_) => false,
                    };

                    // If it's valid, add it to the real_cells list.
                    if is_valid_data {

                        // If real_row is -1 (invalid), then we need to add an empty row to the model (NOT TO THE FILTER)
                        // because that means we have no row for that position, and we need one.
                        if real_row == -1 {
                            let row = get_new_row(&self.table_definition());
                            for index in 0..row.count_0a() {
                                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                            }
                            self.table_model.append_row_q_list_of_q_standard_item(&row);
                            real_row = self.table_model.row_count_0a() - 1;
                            added_rows += 1;
                        }
                        real_cells.push((self.table_model.index_2a(real_row, real_column), *text));
                    }
                }
                visual_column += 1;
            }
            visual_row += 1;
        }

        // We need to update the undo model here, because otherwise it'll start triggering crashes
        // in case the first thing to paste is equal to the current value. In that case, the set_data
        // will not trigger, and the update_undo_model will not trigger either, causing a crash if
        // immediately after that we try to paste something in a new line (which will not exist in the undo model).
        {
            update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
        }

        self.set_data_on_cells(&real_cells, added_rows, &[], &fields_processed, app_ui, pack_file_contents_ui);
    }

    /// Function to undo/redo an operation in the table.
    ///
    /// If undo = true we are undoing. Otherwise we are redoing.
    /// NOTE: repeat_x_times is for internal recursion!!! ALWAYS PUT A 0 THERE!!!.
    pub unsafe fn undo_redo(
        &self,
        undo: bool,
        mut repeat_x_times: usize,
    ) {
        let filter: QPtr<QSortFilterProxyModel> = self.table_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let mut is_carolina = false;

        {
            let (mut history_source, mut history_opposite) = if undo {
                (self.history_undo.write().unwrap(), self.history_redo.write().unwrap())
            } else {
                (self.history_redo.write().unwrap(), self.history_undo.write().unwrap())
            };

            // Get the last operation in the Undo History, or return if there is none.
            let operation = if let Some(operation) = history_source.pop() { operation } else { return };
            log_to_status_bar(&format!("{operation:?}"));
            match operation {
                TableOperations::Editing(editions) => {

                    // Prepare the redo operation, then do the rest.
                    let mut redo_editions = vec![];
                    editions.iter().for_each(|x| redo_editions.push((((x.0).0, (x.0).1), atomic_from_ptr((*model.item_2a((x.0).0, (x.0).1)).clone()))));
                    history_opposite.push(TableOperations::Editing(redo_editions));

                    self.undo_lock.store(true, Ordering::SeqCst);
                    for (index, ((row, column), item)) in editions.iter().enumerate() {
                        let item = &*ptr_from_atomic(item);
                        model.set_item_3a(*row, *column, item.clone());

                        // If we are going to process the last one, unlock the save.
                        if index == editions.len() - 1 {
                            model.item_2a(*row, *column).set_data_2a(&QVariant::from_int(1i32), 16);
                            model.item_2a(*row, *column).set_data_2a(&QVariant::new(), 16);
                        }

                        // We need to update the new row vs vanilla status here, because as that one affects all rows, it's not done automatically.
                        let definition = self.table_definition();
                        self.update_row_diff_marker(&definition, *row);
                    }

                    // Select all the edited items.
                    let selection_model = self.table_view.selection_model();
                    selection_model.clear();

                    // TODO: This is still very slow. We need some kind of range optimization.
                    let _blocker = QSignalBlocker::from_q_object(&selection_model);
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
                }

                // This actions if for undoing "add rows" actions. It deletes the stored rows.
                TableOperations::AddRows(mut rows) => {

                    // Sort them 0->9, so we can process them.
                    rows.sort_unstable();
                    self.undo_lock.store(true, Ordering::SeqCst);
                    let rows_split = delete_rows(&self.table_model_ptr(), &rows);
                    history_opposite.push(TableOperations::RemoveRows(rows_split));
                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                // NOTE: the rows list must ALWAYS be in 1->9 order. Otherwise this breaks.
                TableOperations::RemoveRows(mut rows) => {
                    self.undo_lock.store(true, Ordering::SeqCst);
                    self.save_lock.store(true, Ordering::SeqCst);

                    // Make sure the order of these ones is always correct (9->0).
                    rows.sort_by(|x, y| x.0.cmp(&y.0));

                    // First, we re-create the rows and re-insert them.
                    for (index, row_pack) in &rows {
                        for (offset, row) in row_pack.iter().enumerate() {
                            let qlist = QListOfQStandardItem::new();
                            row.iter().for_each(|x| qlist.append_q_standard_item(&ptr_from_atomic(x).as_mut_raw_ptr()));
                            model.insert_row_int_q_list_of_q_standard_item(*index + offset as i32, &qlist);
                        }
                    }

                    // Then, create the redo action for this one.
                    let mut rows_to_add = rows.iter()
                        .flat_map(|(index, row_pack)|
                            row_pack.iter().enumerate()
                                .map(|(x, _)| *index + x as i32)
                                .collect::<Vec<i32>>())
                        .collect::<Vec<i32>>();

                    rows_to_add.reverse();
                    history_opposite.push(TableOperations::AddRows(rows_to_add));

                    // Select all the re-inserted rows that are in the filter. We need to block signals here because the bigger this gets,
                    // the slower it gets. And it gets very slow on high amounts of lines.
                    let selection_model = self.table_view.selection_model();
                    selection_model.clear();
                    for (index, row_pack) in &rows {
                        let initial_model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(*index, 0));
                        let final_model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(*index + row_pack.len() as i32 - 1, 0));
                        if initial_model_index_filtered.is_valid() && final_model_index_filtered.is_valid() {
                            let selection = QItemSelection::new_2a(&initial_model_index_filtered, &final_model_index_filtered);
                            selection_model.select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Select | SelectionFlag::Rows);
                        }
                    }

                    // Trick to tell the model to update everything.
                    self.save_lock.store(false, Ordering::SeqCst);
                    model.item_2a(0, 0).set_data_2a(&QVariant::new(), 16);
                    self.undo_lock.store(false, Ordering::SeqCst);
                }

                // This action is special and we have to manually trigger a save for it.
                //
                // Also, this operation is NOT CHEAP. We need to replace the definition and effectively reload the entire table and associated data.
                /*TableOperations::ImportTSV(table_data) => {

                    let old_data = self.get_copy_of_table();
                    history_opposite.push(TableOperations::ImportTSV(old_data));

                    let row_count = self.table_model.row_count_0a();
                    self.undo_lock.store(true, Ordering::SeqCst);
                    self.table_model.remove_rows_2a(0, row_count);
                    for row in &table_data {
                        let row = ptr_from_atomic(row);
                        self.table_model.append_row_q_list_of_q_standard_item(row.as_ref().unwrap())
                    }
                    self.undo_lock.store(false, Ordering::SeqCst);
                }*/

                TableOperations::Carolina(mut operations) => {
                    is_carolina = true;
                    repeat_x_times = operations.len();
                    operations.reverse();
                    history_source.append(&mut operations);
                }
            }

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

        // If we have repetitions, it means we got a carolina. Repeat all the times we need until all editions are undone.
        // Then, remove all the actions done and put them into a carolina.
        if repeat_x_times > 0 {
            self.undo_redo(undo, repeat_x_times - 1);
            if is_carolina {
                let mut history_opposite = if undo {
                    self.history_redo.write().unwrap()
                } else {
                    self.history_undo.write().unwrap()
                };
                let len = history_opposite.len();
                let mut edits = history_opposite.drain((len - repeat_x_times)..).collect::<Vec<TableOperations>>();
                edits.reverse();
                history_opposite.push(TableOperations::Carolina(edits));
            }
        }

        self.start_delayed_updates_timer();
    }

    /// This function returns the provided indexes's data as a LUA table.
    unsafe fn get_indexes_as_lua_table(&self, indexes: &[Ref<QModelIndex>], has_keys: bool) -> String {
        let mut table_data: Vec<(Option<String>, Vec<String>)> = vec![];
        let mut last_row = None;
        let definition = self.table_definition();
        let patches = Some(definition.patches());
        let fields_processed = definition.fields_processed();
        for index in indexes {
            if index.column() != -1 {
                let current_row = index.row();
                match last_row {
                    Some(row) => {

                        // If it's the same row as before, take the row from the table data and append it.
                        if current_row == row {
                            let entry = table_data.last_mut().unwrap();
                            let data = self.get_escaped_lua_string_from_index(*index, &fields_processed);
                            if entry.0.is_none() && fields_processed[index.column() as usize].is_key(patches) && has_keys {
                                entry.0 = Some(self.escape_string_from_index(*index, &fields_processed));
                            }
                            entry.1.push(data);
                        }

                        // If it's not the same row as before, we create it as a new row.
                        else {
                            let mut entry = (None, vec![]);
                            let data = self.get_escaped_lua_string_from_index(*index, &fields_processed);
                            entry.1.push(data.to_string());
                            if entry.0.is_none() && fields_processed[index.column() as usize].is_key(patches) && has_keys {
                                entry.0 = Some(self.escape_string_from_index(*index, &fields_processed));
                            }
                            table_data.push(entry);
                        }
                    }
                    None => {
                        let mut entry = (None, vec![]);
                        let data = self.get_escaped_lua_string_from_index(*index, &fields_processed);
                        entry.1.push(data.to_string());
                        if entry.0.is_none() && fields_processed[index.column() as usize].is_key(patches) && has_keys {
                            entry.0 = Some(self.escape_string_from_index(*index, &fields_processed));
                        }
                        table_data.push(entry);
                    }
                }

                last_row = Some(current_row);
            }
        }

        // Create the string of the table.
        let mut lua_table = String::new();

        if !table_data.is_empty() {
            if has_keys {
                lua_table.push_str("TABLE = {\n");
            }

            for (index, row) in table_data.iter().enumerate() {

                // Start the row.
                if let Some(key) = &row.0 {
                    lua_table.push_str(&format!("\t[{key}] = {{"));
                }
                else {
                    lua_table.push('{');
                }

                // For each cell in the row, push it to the LUA Table.
                for cell in row.1.iter() {
                    lua_table.push_str(cell);
                }

                // Take out the last comma.
                lua_table.pop();

                // Close the row.
                if index == table_data.len() - 1 {
                    lua_table.push_str(" }\n");
                }
                else {
                    lua_table.push_str(" },\n");
                }
            }

            if has_keys {
                lua_table.push('}');
            }
        }

        lua_table
    }

    /// This function turns the data from the provided indexes into LUA compatible strings.
    unsafe fn get_escaped_lua_string_from_index(&self, index: Ref<QModelIndex>, fields_processed: &[Field]) -> String {
        format!(" [\"{}\"] = {},", fields_processed[index.column() as usize].name(), self.escape_string_from_index(index, fields_processed))
    }

    /// This function escapes the value inside an index.
    unsafe fn escape_string_from_index(&self, index: Ref<QModelIndex>, fields_processed: &[Field]) -> String {
        let item = self.table_model.item_from_index(index);
        match fields_processed[index.column() as usize].field_type() {
            FieldType::Boolean => if let CheckState::Checked = item.check_state() { "true".to_owned() } else { "false".to_owned() },

            // Floats need to be tweaked to fix trailing zeroes and precision issues, like turning 0.5000004 into 0.5.
            FieldType::F32 => {
                let data_str = format!("{}", item.data_1a(2).to_float_0a());

                // If we have more than 3 decimals, we limit it to three, then do magic to remove trailing zeroes.
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 { format!("{}", format!("{:.4}", item.data_1a(2).to_float_0a()).parse::<f32>().unwrap()) }
                    else { data_str }
                }
                else { data_str }
            },

            // Floats need to be tweaked to fix trailing zeroes and precision issues, like turning 0.5000004 into 0.5.
            FieldType::F64 => {
                let data_str = format!("{}", item.data_1a(2).to_float_0a());

                // If we have more than 3 decimals, we limit it to three, then do magic to remove trailing zeroes.
                if let Some(position) = data_str.find('.') {
                    let decimals = &data_str[position..].len();
                    if *decimals > 4 { format!("{}", format!("{:.4}", item.data_1a(2).to_double_0a()).parse::<f64>().unwrap()) }
                    else { data_str }
                }
                else { data_str }
            },
            FieldType::I16 |
            FieldType::I32 |
            FieldType::I64 |
            FieldType::OptionalI16 |
            FieldType::OptionalI32 |
            FieldType::OptionalI64 => format!("{}", item.data_1a(2).to_long_long_0a()),

            // All these are Strings, so they need to escape certain chars and include commas in Lua.
            FieldType::ColourRGB |
            FieldType::StringU8 |
            FieldType::StringU16 |
            FieldType::OptionalStringU8 |
            FieldType::OptionalStringU16 => format!("\"{}\"", item.text().to_std_string().escape_default()),
            FieldType::SequenceU16(_) => "\"SequenceU16\"".to_owned(),
            FieldType::SequenceU32(_) => "\"SequenceU32\"".to_owned(),
        }
    }

    /// This function is used to append new rows to a table.
    ///
    /// If clone = true, the appended rows are copies of the selected ones.
    pub unsafe fn append_rows(&self, clone: bool) {

        // Get the indexes ready for battle.
        let selection = self.table_view.selection_model().selection();
        let indexes = self.table_filter.map_selection_to_source(&selection).indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_by_model(&mut indexes_sorted);
        dedup_indexes_per_row(&mut indexes_sorted);
        let mut row_numbers = vec![];

        let rows = if clone {
            let mut rows = vec![];
            for index in indexes_sorted.iter() {
                row_numbers.push(index.row());

                let columns = self.table_model.column_count_0a();
                let qlist = QListOfQStandardItem::new();
                for column in 0..columns {
                    let original_item = self.table_model.item_2a(index.row(), column);
                    let item = (*original_item).clone();
                    item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                    qlist.append_q_standard_item(&item.as_mut_raw_ptr());
                }

                rows.push(qlist);
            }
            rows
        } else {
            let row = get_new_row(&self.table_definition());
            for index in 0..row.count_0a() {
                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
            }
            vec![row]
        };

        let selection_model = self.table_view.selection_model();
        selection_model.clear();
        for row in &rows {
            self.table_model.append_row_q_list_of_q_standard_item(row.as_ref());

            // Select the row and scroll to it.
            let model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(self.table_model.row_count_0a() - 1, 0));
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

        // Update the undo stuff. Cloned rows are the amount of rows - the amount of cloned rows.
        let total_rows = self.table_model.row_count_0a();
        let range = (total_rows - rows.len() as i32..total_rows).collect::<Vec<i32>>();
        self.history_undo.write().unwrap().push(TableOperations::AddRows(range));
        self.history_redo.write().unwrap().clear();
        self.start_delayed_updates_timer();
        update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
    }

    /// This function is used to insert new rows into a table.
    ///
    /// If clone = true, the appended rows are copies of the selected ones.
    pub unsafe fn insert_rows(&self, clone: bool) {

        // Get the indexes ready for battle.
        let selection = self.table_view.selection_model().selection();
        let indexes = self.table_filter.map_selection_to_source(&selection).indexes();
        let mut indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        sort_indexes_by_model(&mut indexes_sorted);
        dedup_indexes_per_row(&mut indexes_sorted);
        let mut row_numbers = vec![];

        // If nothing is selected, we just append one new row at the end. This only happens when adding empty rows, so...
        if indexes_sorted.is_empty() {
            let row = get_new_row(&self.table_definition());
            for index in 0..row.count_0a() {
                row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
            }
            self.table_model.append_row_q_list_of_q_standard_item(&row);
            row_numbers.push(self.table_model.row_count_0a() - 1);
        }

        let selection_model = self.table_view.selection_model();
        selection_model.clear();

        for index in indexes_sorted.iter().rev() {
            row_numbers.push(index.row() + (indexes_sorted.len() - row_numbers.len() - 1) as i32);

            // If we want to clone, we copy the currently selected row. If not, we just create a new one.
            let row = if clone {
                let columns = self.table_model.column_count_0a();
                let qlist = QListOfQStandardItem::new();
                for column in 0..columns {
                    let original_item = self.table_model.item_2a(index.row(), column);
                    let item = (*original_item).clone();
                    item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                    qlist.append_q_standard_item(&item.as_mut_raw_ptr());
                }
                qlist
            } else {
                let row = get_new_row(&self.table_definition());
                for index in 0..row.count_0a() {
                    row.value_1a(index).set_data_2a(&QVariant::from_bool(true), ITEM_IS_ADDED);
                }
                row
            };
            self.table_model.insert_row_int_q_list_of_q_standard_item(index.row(), &row);

            // Select the row.
            let new_item = row.take_first();
            if !new_item.is_null() {
                let model_index_filtered = self.table_filter.map_from_source(&self.table_model.index_2a(new_item.index().row(), 0));
                if model_index_filtered.is_valid() {
                    selection_model.select_q_model_index_q_flags_selection_flag(
                        &model_index_filtered,
                        SelectionFlag::Select | SelectionFlag::Rows
                    );
                }
            }
        }

        // The undo mode needs this reversed.
        self.history_undo.write().unwrap().push(TableOperations::AddRows(row_numbers));
        self.history_redo.write().unwrap().clear();
        self.start_delayed_updates_timer();
        update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
    }

    /// This function creates the entire "Rewrite selection" dialog for tables. It returns the rewriting sequence, or None.
    pub unsafe fn create_rewrite_selection_dialog(&self) -> Option<(bool, String)> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view);
        dialog.set_window_title(&qtr("rewrite_selection_title"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string(&qtr("rewrite_selection_instructions_title"));
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string(&qtr("rewrite_selection_instructions"));
        instructions_grid.add_widget_5a(& instructions_label, 0, 0, 1, 1);

        let is_math_op = QCheckBox::from_q_string(&qtr("rewrite_selection_is_math"));
        let rewrite_sequence_line_edit = QLineEdit::new();
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rewrite_selection_placeholder"));
        let accept_button = QPushButton::from_q_string(&qtr("rewrite_selection_accept"));

        main_grid.add_widget_5a(instructions_frame.into_ptr(), 0, 0, 1, 2);
        main_grid.add_widget_5a(&is_math_op, 1, 0, 1, 2);
        main_grid.add_widget_5a(&rewrite_sequence_line_edit, 2, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 2, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = rewrite_sequence_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some((is_math_op.is_checked(), new_text)) }
        } else { None }
    }

    /// This function creates the entire "Generate Ids" dialog for tables. It returns the starting id, or None.
    pub unsafe fn create_generate_ids_dialog(&self, initial_value: i64, is_i64: bool) -> Option<i64> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view);
        dialog.set_window_title(&qtr("generate_ids_title"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string_q_widget(&qtr("generate_ids_instructions_title"), &dialog);
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string_q_widget(&qtr("generate_ids_instructions"), &instructions_frame);
        instructions_grid.add_widget_5a(& instructions_label, 0, 0, 1, 1);

        let starting_id_spin_box = if !is_i64 {
            let starting_id_spin_box = QSpinBox::new_1a(&dialog);
            starting_id_spin_box.set_minimum(i32::MIN);
            starting_id_spin_box.set_maximum(i32::MAX);
            starting_id_spin_box.set_value(initial_value as i32);
            starting_id_spin_box.static_upcast()
        } else {
            let starting_id_spin_box = new_q_spinbox_i64_safe(&dialog.static_upcast());
            set_min_q_spinbox_i64_safe(&starting_id_spin_box, i64::MIN);
            set_max_q_spinbox_i64_safe(&starting_id_spin_box, i64::MAX);
            set_value_q_spinbox_i64_safe(&starting_id_spin_box, initial_value);
            starting_id_spin_box
        };
        let accept_button = QPushButton::from_q_string(&qtr("generate_ids_accept"));

        main_grid.add_widget_5a(&instructions_frame, 0, 0, 1, 1);
        main_grid.add_widget_5a(&starting_id_spin_box, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 2, 0, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            if is_i64 {
                Some(value_q_spinbox_i64_safe(&starting_id_spin_box))
            } else {
                Some(starting_id_spin_box.static_downcast::<QSpinBox>().value() as i64)
            }
        } else { None }
    }

    /// This function takes care of the "Delete filtered-out rows" feature for tables.
    pub unsafe fn delete_filtered_out_rows(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let visible_columns = (0..self.table_model.column_count_0a()).filter(|index| !self.table_view.is_column_hidden(*index)).collect::<Vec<i32>>();

        // If it's empty, it means everything is hidden, so we delete everything.
        let mut rows_to_delete = vec![];
        for row in 0..self.table_model.row_count_0a() {
            if visible_columns.is_empty() {
                rows_to_delete.push(row);
            } else if !self.table_filter.map_from_source(&self.table_model.index_2a(row, visible_columns[0])).is_valid() {
                rows_to_delete.push(row);
            }
        }

        // Dedup the list and reverse it.
        rows_to_delete.sort_unstable();
        rows_to_delete.dedup();
        rows_to_delete.reverse();

        let fields_processed = self.table_definition().fields_processed();
        self.set_data_on_cells(&[], 0, &rows_to_delete, &fields_processed, app_ui, pack_file_contents_ui);
    }

    /// This function takes care of the "Smart Delete" feature for tables.
    pub unsafe fn smart_delete(&self, delete_all_rows: bool, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Get the selected indexes, the split them in two groups: one with full rows selected and another with single cells selected.
        let indexes_sorted = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());
        let fields_processed = self.table_definition().fields_processed();

        if delete_all_rows {
            let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

            // Dedup the list and reverse it.
            rows_to_delete.sort_unstable();
            rows_to_delete.dedup();
            rows_to_delete.reverse();

            self.set_data_on_cells(&[], 0, &rows_to_delete, &fields_processed, app_ui, pack_file_contents_ui);
        } else {

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

            let visible_column_count = (0..self.table_model.column_count_0a()).filter(|index| !self.table_view.is_column_hidden(*index)).count();
            let full_rows = cells.iter()
                .filter(|(_, y)| y.len() >= visible_column_count)
                .map(|(x, _)| *x)
                .collect::<Vec<i32>>();

            let individual_cells = cells.iter()
                .filter(|(_, y)| y.len() < visible_column_count)
                .map(|(x, y)| (*x, y.to_vec()))
                .collect::<Vec<(i32, Vec<i32>)>>();

            let default_str = "".to_owned();
            let default_f32 = "0.0".to_owned();
            let default_f64 = "0.0".to_owned();
            let default_i32 = "0".to_owned();
            let default_bool = "false".to_owned();
            let default_colour_rgb = "000000".to_owned();

            let mut real_cells = vec![];
            let mut values = vec![];
            for (row, columns) in &individual_cells {
                for column in columns {
                    let index = self.table_model.index_2a(*row, *column);
                    if index.is_valid() {
                        match fields_processed[*column as usize].field_type() {
                            FieldType::Boolean => values.push(&*default_bool),
                            FieldType::F32 => values.push(&*default_f32),
                            FieldType::F64 => values.push(&*default_f64),
                            FieldType::I16 |
                            FieldType::I32 |
                            FieldType::I64 => values.push(&*default_i32),
                            FieldType::OptionalI16 |
                            FieldType::OptionalI32 |
                            FieldType::OptionalI64 => values.push(&*default_i32),
                            FieldType::ColourRGB => values.push(&*default_colour_rgb),
                            FieldType::StringU8 |
                            FieldType::StringU16 |
                            FieldType::OptionalStringU8 |
                            FieldType::OptionalStringU16 => values.push(&*default_str),
                            FieldType::SequenceU16(_) |
                            FieldType::SequenceU32(_) => continue,
                        }
                        real_cells.push(index);
                    }
                }
            }


            let mut realer_cells = vec![];
            for index in (0..real_cells.len()).rev() {
                realer_cells.push((real_cells.pop().unwrap(), values[index]));
            }
            realer_cells.reverse();

            self.set_data_on_cells(&realer_cells, 0, &full_rows, &fields_processed, app_ui, pack_file_contents_ui);
        }
    }

    /// Function used to have a generic way to set data on cells/remove rows and setup their undo steps.
    pub unsafe fn set_data_on_cells(
        &self,
        real_cells: &[(CppBox<QModelIndex>, &str)],
        added_rows: i32,
        rows_to_delete: &[i32],
        fields: &[Field],
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>
    ) {

        // Block the events so this doesn't take ages. Also, this means we do weird things here for performance.
        self.table_model.block_signals(true);
        self.undo_model.block_signals(true);

        let mut changed_cells = 0;

        for (real_cell, text) in real_cells {
            if real_cell.is_valid() {

                // Depending on the column, we try to encode the data in one format or another.
                let current_value = self.table_model.data_1a(real_cell).to_string().to_std_string();
                match fields[real_cell.column() as usize].field_type() {

                    FieldType::Boolean => {
                        let current_value = self.table_model.item_from_index(real_cell).check_state();
                        let new_value = if text.to_lowercase() == "true" || *text == "1" { CheckState::Checked } else { CheckState::Unchecked };
                        if current_value != new_value {
                            self.table_model.item_from_index(real_cell).set_check_state(new_value);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    // These are a bit special because we have to ignore any difference after the third decimal.
                    FieldType::F32 => {
                        let current_value = format!("{:.4}", self.table_model.data_2a(real_cell, 2).to_float_0a());
                        if let Ok(new_value) = text.parse::<f32>() {
                            let new_value_txt = format!("{new_value:.4}");
                            if current_value != new_value_txt {
                                self.table_model.set_data_3a(real_cell, &QVariant::from_float(new_value), 2);
                                changed_cells += 1;
                                self.process_edition(self.table_model.item_from_index(real_cell));
                            }
                        }
                    },

                    // Same thing as with F32.
                    FieldType::F64 => {
                        let current_value = format!("{:.4}", self.table_model.data_2a(real_cell, 2).to_double_0a());
                        if let Ok(new_value) = text.parse::<f64>() {
                            let new_value_txt = format!("{new_value:.4}");
                            if current_value != new_value_txt {
                                self.table_model.set_data_3a(real_cell, &QVariant::from_double(new_value), 2);
                                changed_cells += 1;
                                self.process_edition(self.table_model.item_from_index(real_cell));
                            }
                        }
                    },

                    FieldType::OptionalI16 |
                    FieldType::I16 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i16>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i16 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_int(new_value as i32), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    FieldType::OptionalI32 |
                    FieldType::I32 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i32>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i32 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_int(new_value), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    FieldType::OptionalI64 |
                    FieldType::I64 => {

                        // To the stupid float conversion problem avoid, this we do.
                        let new_value = if let Ok(new_value) = text.parse::<i64>() { new_value }
                        else if let Ok(new_value) = text.parse::<f32>() { new_value.round() as i64 }
                        else { continue };

                        if current_value != new_value.to_string() {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_i64(new_value), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    },

                    FieldType::ColourRGB => {
                        if u32::from_str_radix(text, 16).is_ok() && current_value != *text {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_q_string(&QString::from_std_str(text)), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    }
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => {
                        if current_value != *text {
                            self.table_model.set_data_3a(real_cell, &QVariant::from_q_string(&QString::from_std_str(text)), 2);
                            changed_cells += 1;
                            self.process_edition(self.table_model.item_from_index(real_cell));
                        }
                    }

                    // Do NOT rewrite sequences.
                    FieldType::SequenceU16(_) |
                    FieldType::SequenceU32(_) => {},
                }
            }
        }

        self.table_model.block_signals(false);
        self.undo_model.block_signals(false);

        let deleted_rows = if !rows_to_delete.is_empty() {
            utils::delete_rows(&self.table_model_ptr(), rows_to_delete)
        } else { vec![] };

        // Fix the undo history to have all the previous changed merged into one. Or that's what I wanted.
        // Sadly, the world doesn't work like that. As we can edit, delete AND add rows, we have to use a combined undo operation.
        // I'll call it... Carolina.
        if changed_cells > 0 || added_rows > 0 || !deleted_rows.is_empty() {
            update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
            {
                let mut history_undo = self.history_undo.write().unwrap();
                let mut history_redo = self.history_redo.write().unwrap();

                let len = history_undo.len();
                let mut carolina = vec![];

                if !deleted_rows.is_empty() {
                    carolina.push(TableOperations::RemoveRows(deleted_rows));
                }

                if changed_cells > 0 {

                    let mut edits_data = vec![];
                    if changed_cells > len {
                        error!("Error: Changed cells greater than lenght. How the fuck did this happen? Fixing it so at least it doesn't crash.");
                        changed_cells = len;
                    }

                    let mut edits = history_undo.drain((len - changed_cells)..);
                    for edit in &mut edits {
                        if let TableOperations::Editing(mut edit) = edit {
                            edits_data.append(&mut edit);
                        }
                    }
                    carolina.push(TableOperations::Editing(edits_data));
                }

                if added_rows > 0 {
                    let mut rows = vec![];
                    ((self.table_model.row_count_0a() - added_rows)..self.table_model.row_count_0a()).rev().for_each(|x| rows.push(x));
                    carolina.push(TableOperations::AddRows(rows));
                }

                history_undo.push(TableOperations::Carolina(carolina));
                history_redo.clear();
            }

            self.post_process_edition(app_ui, pack_file_contents_ui);
        }

        // Trick to properly update the view.
        self.table_view.viewport().repaint();

        self.start_delayed_updates_timer();
    }

    /// Process a single cell edition. Launch this after every edition if the signals are blocked.
    pub unsafe fn process_edition(&self, item: Ptr<QStandardItem>) {
        let item_old = self.undo_model.item_2a(item.row(), item.column());

        // Only trigger this if the values are actually different. Checkable cells are tricky. Nested cells an go to hell.
        if (item_old.text().compare_q_string(item.text().as_ref()) != 0 || item_old.check_state() != item.check_state()) ||
            item_old.data_1a(ITEM_IS_SEQUENCE).to_bool() && 0 != item_old.data_1a(ITEM_SEQUENCE_DATA).to_string().compare_q_string(&item.data_1a(ITEM_SEQUENCE_DATA).to_string()) {
            let edition = vec![((item.row(), item.column()), atomic_from_ptr((*item_old).clone()))];
            let operation = TableOperations::Editing(edition);
            self.history_undo.write().unwrap().push(operation);

            item.set_data_2a(&QVariant::from_bool(true), ITEM_IS_MODIFIED);

            let definition = self.table_definition();
            let patches = Some(definition.patches());
            let fields_processed = definition.fields_processed();
            let field = &fields_processed[item.column() as usize];

            // Update the lookup data while the model is blocked.
            if setting_bool("enable_lookups") {
                let dependency_data = self.dependency_data.read().unwrap();
                if let Some(column_data) = dependency_data.get(&item.column()) {
                    match column_data.data().get(&item.text().to_std_string()) {
                        Some(lookup) => item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup)), ITEM_SUB_DATA),
                        None => item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str("")), ITEM_SUB_DATA),
                    }
                }

                // If the edited field is used as lookup of another field on the same table, update it too.
                //
                // Only non-reference key columns in single-key tables can have lookups, so we only check those.
                let key_amount = fields_processed.iter().filter(|x| x.is_key(patches)).count();
                if key_amount == 1 {

                    for (column, field_ref) in fields_processed.iter().enumerate() {
                        let mut lookup_string = String::new();
                        let item_looking_up = self.table_model.item_2a(item.row(), column as i32);

                        if field_ref.is_key(patches) && field_ref.is_reference(patches).is_none() {
                            if let Some(lookups_ref) = field_ref.lookup(patches) {
                                for lookup_ref in lookups_ref {
                                    if field.name() == lookup_ref {
                                        let data = item.data_1a(2).to_string().to_std_string();

                                        if !lookup_string.is_empty() {
                                            lookup_string.push(':');
                                        }

                                        lookup_string.push_str(&data);
                                    }
                                }
                            }
                        }

                        if !lookup_string.is_empty() {
                            item_looking_up.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(lookup_string)), ITEM_SUB_DATA);
                        }
                    }
                }
            }

            // If the edited column has icons we need to fetch the new icon from the backend and apply it.
            if setting_bool("enable_icons") && field.is_filename(patches) {
                let mut icons = BTreeMap::new();
                let data = vec![vec![get_field_from_view(&self.table_model.static_upcast(), field, item.row(), item.column())]];

                if request_backend_files(&data, 0, field, patches, &mut icons).is_ok() {
                    if let Some(column_data) = icons.get(&0) {

                        let cell_data = data[0][0].data_to_string().replace('\\', "/");

                        // For paths, we need to fix the ones in older games starting with / or data/.
                        let mut start_offset = 0;
                        if cell_data.starts_with("/") {
                            start_offset += 1;
                        }
                        if cell_data.starts_with("data/") {
                            start_offset += 5;
                        }

                        let paths_join = column_data.0.replace('%', &cell_data[start_offset..]).to_lowercase();
                        let paths_split = paths_join.split(';');

                        let mut found = false;
                        for path in paths_split {
                            if let Some(icon) = column_data.1.get(path) {
                                let icon = ref_from_atomic(icon);
                                item.set_icon(icon);
                                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(path)), ITEM_ICON_PATH);
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            item.set_icon(&QIcon::new());
                            item.set_data_2a(&QVariant::new(), ITEM_ICON_PATH);
                        }

                        // For tooltips, we just nuke all the catched pngs. It's simpler than trying to go one by one and finding the ones that need updating.
                        item.set_data_2a(&QVariant::new(), ITEM_ICON_CACHE);
                    }
                }
            }

            self.update_row_diff_marker(&definition, item.row());
        }
    }

    /// Triggers stuff that should be done once after a bunch of editions.
    pub unsafe fn post_process_edition(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        update_undo_model(&self.table_model_ptr(), &self.undo_model_ptr());
        self.context_menu_update();
        if let Some(ref packed_file_path) = self.packed_file_path {
            if let Some(search_view) = &*self.search_view() {
                search_view.update_search(self);
            }
            if let DataSource::PackFile = *self.data_source.read().unwrap() {
                set_modified(true, &packed_file_path.read().unwrap(), app_ui, pack_file_contents_ui);
            }
        }

        if setting_bool("table_resize_on_edit") {
            self.table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }

        // Re-sort and re-filter the table, as it's not automatically done.
        self.table_filter.set_dynamic_sort_filter(false);
        self.table_filter.set_dynamic_sort_filter(true);

        self.table_filter.invalidate();
        self.filter_table();

        self.table_view.viewport().repaint();
    }

    pub unsafe fn update_row_diff_marker(&self, definition: &Definition, row: i32) {
        let fields_processed = definition.fields_processed();
        let key_pos = definition.key_column_positions();

        let vanilla_data = self.vanilla_hashed_tables.read().unwrap();
        if vanilla_data.is_empty() || key_pos.is_empty() {
            return;
        }

        let keys_joined = key_pos.iter()
            .map(|x| self.table_model.index_2a(row, *x as i32).data_1a(2).to_string().to_std_string())
            .join("");

        let mut found = false;
        for (vanilla_table, hashes) in &*vanilla_data {
            if let Some(vanilla_row) = hashes.get(&keys_joined) {
                let vanilla_data = vanilla_table.data();
                let vanilla_definition = vanilla_table.definition();
                let vanilla_processed_fields = vanilla_definition.fields_processed();

                if let Some(vanilla_row_data) = vanilla_data.get(*vanilla_row as usize) {
                    for (column, field) in fields_processed.iter().enumerate() {
                        let local_data = get_field_from_view(&self.table_model.static_upcast(), field, row, column as i32);

                        let item = self.table_model.item_2a(row, column as i32);
                        if !item.is_null() {
                            match vanilla_processed_fields.iter().position(|x| x.name() == field.name()) {
                                Some(vanilla_field_column) => {

                                    // Make sure to check the column, because we may be getting a different definition of our own here.
                                    match vanilla_row_data.get(vanilla_field_column) {
                                        Some(vanilla_data) => {
                                            item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_HAS_VANILLA_VALUE);
                                            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(vanilla_data.data_to_string())), ITEM_VANILLA_VALUE);

                                            if vanilla_data != &local_data {
                                                item.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_IS_MODIFIED_VS_VANILLA);
                                            } else {
                                                item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA);
                                            }
                                        },

                                        None => item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA),
                                    }
                                }

                                // If the field is not in the vanilla table, mark it as not modified.
                                None => item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA),
                            }

                            found = true;
                        }
                    }
                }
            }

            if found {
                break;
            }
        }

        // If we don't have vanilla data, mark all cells as not having vanilla data.
        if !found {
            for column in 0..fields_processed.len() {
                let item = self.table_model.item_2a(row, column as i32);
                if !item.is_null() {
                    item.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_MODIFIED_VS_VANILLA);
                }
            }
        }

        // For this we need to alter all items in the same row.
        //let hitem = self.table_model.horizontal_header_item(row);
        //if found {
            //hitem.set_data_2a(ref_from_atomic(&QVARIANT_FALSE), ITEM_IS_ADDED_VS_VANILLA);
        //} else {
            //hitem.set_data_2a(ref_from_atomic(&QVARIANT_TRUE), ITEM_IS_ADDED_VS_VANILLA);
        //}
    }

    /// This function triggers a cascade edition through the entire program of the selected cells.
    pub unsafe fn cascade_edition(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // This feature has some... interesting lockups when running alongside a diagnostics check. So, while this runs,
        // we have to avoid triggering the diagnostics check.
        self.timer_delayed_updates.stop();

        // We only want to do this for tables we can identify.
        let table_name = if let Some(table_name) = self.table_name() { table_name.to_lowercase() } else { return };

        // Get the selected indexes.
        let indexes = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());

        // Ask the dialog to get the data needed for the replacing.
        if let Some(editions) = self.cascade_edition_dialog(&indexes) {
            app_ui.toggle_main_window(false);

            // Trigger editions in our own table.
            let real_cells = editions.iter()
                .map(|(_, new_value, row, column)| (self.table_model.index_2a(*row, *column), &**new_value))
                .collect::<Vec<(CppBox<QModelIndex>, &str)>>();

            let fields_processed = self.table_definition().fields_processed();
            self.set_data_on_cells(&real_cells, 0, &[], &fields_processed, app_ui, pack_file_contents_ui);

            // Stop the timer again.
            self.timer_delayed_updates.stop();

            // Now that we know what to edit, save all views of referencing files, so we only have to deal with them in the background.
            let _ = AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui);

            // Then ask the backend to do the heavy work.
            let definition = self.table_definition().clone();
            let fields_processed = definition.fields_processed();
            let changes = editions.iter().map(|(value_before, value_after, _, column)|
                (fields_processed[*column as usize].clone(), value_before.to_string(), value_after.to_string()))
                .collect::<Vec<_>>();

            let receiver = CENTRAL_COMMAND.send_background(Command::CascadeEdition(table_name, definition, changes));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::VecContainerPathVecRFileInfo(edited_paths, packed_files_info) => {

                    // If it worked, get the list of edited PackedFiles and update the TreeView to reflect the change.
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Modify(edited_paths.to_vec()), DataSource::PackFile);
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(edited_paths.to_vec()), DataSource::PackFile);
                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);

                    // Before finishing, reload all edited views.
                    let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                    edited_paths.iter().for_each(|path| {
                        if let Some(file_view) = open_packedfiles.iter_mut().find(|x| *x.path_read() == path.path_raw() && x.data_source() == DataSource::PackFile) {
                            if file_view.reload(path.path_raw(), pack_file_contents_ui).is_err() {
                                let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path.path_raw(), DataSource::PackFile, false);
                            }
                        }
                    });

                    app_ui.toggle_main_window(true);

                    // Now it's safe to trigger the timer.
                    self.start_delayed_updates_timer();
                }
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        // If we didn't do anything, but we cut a timer, continue it.
        else if self.timer_delayed_updates.remaining_time() != -1 {
            self.start_delayed_updates_timer();
        }
    }

    /// This function creates the "Cascade Edition" dialog.
    ///
    /// It returns the data for the editions, or `None` if the dialog is canceled or closed.
    pub unsafe fn cascade_edition_dialog(&self, indexes: &[CppBox<QModelIndex>]) -> Option<Vec<(String, String, i32, i32)>> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&self.table_view);
        dialog.set_window_title(&qtr("cascade_edition_dialog"));
        dialog.set_modal(true);
        dialog.resize_2a(800, 50);

        let main_grid = create_grid_layout(dialog.static_upcast());
        let mut edits = vec![];

        for (row, index) in indexes.iter().enumerate() {
            let old_name_line_edit = QLineEdit::from_q_string_q_widget(&self.table_model.data_1a(index).to_string(), &dialog);
            let new_name_line_edit = QLineEdit::from_q_string_q_widget(&self.table_model.data_1a(index).to_string(), &dialog);

            old_name_line_edit.set_enabled(false);
            main_grid.add_widget_5a(&old_name_line_edit, row as i32, 0, 1, 1);
            main_grid.add_widget_5a(&new_name_line_edit, row as i32, 1, 1, 1);

            edits.push((old_name_line_edit, new_name_line_edit, index));
        }

        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));
        main_grid.add_widget_5a(&accept_button, 99999, 0, 1, 2);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {

            // Filter out unchanged/empty cells.
            let real_edits = edits.into_iter()
                .filter(|(old, new, _)| !new.text().is_empty() && old.text().to_std_string() != new.text().to_std_string())
                .map(|(old, new, index)| (old.text().to_std_string(), new.text().to_std_string(), index.row(), index.column()))
                .collect::<Vec<(String, String, i32, i32)>>();
            if real_edits.is_empty() { None } else { Some(real_edits) }
        } else { None }
    }

    /// This function creates the "Patch Column" dialog and submits a patch of accepted.
    pub unsafe fn patch_column(&self) -> Result<()> {

        // We only want to do this for tables we can identify.
        let edited_table_name = match self.table_name() {
            Some(table_name) => table_name.to_lowercase(),
            None => return Err(anyhow!("This is either not a DB Table, or it's a DB Table but it's corrupted.")),
        };

        // Get the selected indexes.
        let indexes = get_real_indexes_from_visible_selection_sorted(&self.table_view_ptr(), &self.table_view_filter_ptr());

        // Only works with a column selected.
        let columns: Vec<i32> = indexes.iter().map(|x| x.column()).sorted().dedup().collect();
        if columns.len() != 1 {
            return Err(anyhow!("Either 0 or more than 1 column selected. This only works with 1 column selected."));
        }

        let column_index = columns[0];
        let field = self.table_definition().fields_processed().get(column_index as usize).cloned().unwrap();

        // Create and configure the dialog.
        let view = if cfg!(debug_assertions) { PATCH_COLUMN_VIEW_DEBUG } else { PATCH_COLUMN_VIEW_RELEASE };
        let template_path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), view);
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, &self.table_view);

        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let is_key_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "is_key_label")?;
        let default_value_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "default_value_label")?;
        let is_filename_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "is_filename_label")?;
        let filename_relative_path_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "filename_relative_path_label")?;
        let is_reference_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "is_reference_label")?;
        let lookup_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "lookup_label")?;
        let not_empty_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "not_empty_label")?;
        let unused_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "unused_label")?;
        let description_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "description_label")?;

        let is_key_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "is_key_checkbox")?;
        let default_value_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "default_value_line_edit")?;
        let is_filename_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "is_filename_checkbox")?;
        let filename_relative_path_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filename_relative_path_line_edit")?;
        let is_reference_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "is_reference_line_edit")?;
        let lookup_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "lookup_line_edit")?;
        let not_empty_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "not_empty_checkbox")?;
        let unused_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "unused_checkbox")?;
        let description_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "description_text_edit")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        let dialog = main_widget.static_downcast::<QDialog>();

        button_box.button(StandardButton::RestoreDefaults).set_text(&qtr("remove_patches_for_table"));
        button_box.button(StandardButton::Reset).set_text(&qtr("remove_patches_for_column"));

        button_box.button(StandardButton::RestoreDefaults).released().connect(&SlotNoArgs::new(self.table_view(), clone!(
            dialog,
            edited_table_name => move || {
                let receiver = CENTRAL_COMMAND.send_background(Command::RemoveLocalSchemaPatchesForTable(edited_table_name.to_owned()));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => show_dialog(&dialog, tr("patch_removed_table"), true),
                    Response::Error(error) => show_dialog(&dialog, error.to_string(), false),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }

                dialog.close();
            }
        )));

        button_box.button(StandardButton::Reset).released().connect(&SlotNoArgs::new(self.table_view(), clone!(
            dialog,
            field,
            edited_table_name => move || {
                let receiver = CENTRAL_COMMAND.send_background(Command::RemoveLocalSchemaPatchesForTableAndField(edited_table_name.to_owned(), field.name().to_owned()));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => show_dialog(&dialog, tr("patch_removed_column"), true),
                    Response::Error(error) => show_dialog(&dialog, error.to_string(), false),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }

                dialog.close();
            }
        )));

        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_close());
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Setup translations.
        dialog.set_window_title(&qtr("new_column_patch_dialog"));
        instructions_label.set_text(&qtr("column_patch_instructions"));
        is_key_label.set_text(&qtr("is_key"));
        default_value_label.set_text(&qtr("default_value"));
        is_filename_label.set_text(&qtr("is_filename"));
        filename_relative_path_label.set_text(&qtr("filename_relative_path"));
        is_reference_label.set_text(&qtr("is_reference"));
        lookup_label.set_text(&qtr("lookup"));
        not_empty_label.set_text(&qtr("not_empty"));
        unused_label.set_text(&qtr("unused"));
        description_label.set_text(&qtr("description"));

        // Setup data.
        let definition = self.table_definition();
        let patches = Some(definition.patches());

        is_key_checkbox.set_checked(field.is_key(patches));

        if let Some(value) = field.default_value(patches) {
            default_value_line_edit.set_text(&QString::from_std_str(value));
        }

        is_filename_checkbox.set_checked(field.is_filename(patches));

        if let Some(value) = field.filename_relative_path(patches) {
            filename_relative_path_line_edit.set_text(&QString::from_std_str(value.join(";")));
        }

        if let Some(value) = field.is_reference(patches) {
            is_reference_line_edit.set_text(&QString::from_std_str(format!("{};{}", value.0, value.1)));
        }

        if let Some(value) = field.lookup(patches) {
            lookup_line_edit.set_text(&QString::from_std_str(value.join(";")));
        }

        not_empty_checkbox.set_checked(field.cannot_be_empty(patches));
        unused_checkbox.set_checked(field.unused(patches));
        description_text_edit.set_text(&QString::from_std_str(field.description(patches)));

        // Launch.
        if dialog.exec() == 1 {
            let mut column_data = HashMap::new();

            // Only save the values that have changed.
            if field.is_key(patches) != is_key_checkbox.is_checked() {
                column_data.insert("is_key".to_owned(), is_key_checkbox.is_checked().to_string());
            }

            let default_value = field.default_value(patches);
            let default_value_new = default_value_line_edit.text().to_std_string();
            if !default_value_new.is_empty() && (default_value.is_none() || default_value.unwrap() != default_value_new) {
                column_data.insert("default_value".to_owned(), default_value_line_edit.text().to_std_string());
            }

            if field.is_filename(patches) != is_filename_checkbox.is_checked() {
                column_data.insert("is_filename".to_owned(), is_filename_checkbox.is_checked().to_string());
            }

            let relative_value = field.filename_relative_path(patches).map(|x| x.join(";"));
            let relative_value_new = filename_relative_path_line_edit.text().to_std_string();
            if !relative_value_new.is_empty() && (relative_value.is_none() || relative_value.unwrap() != relative_value_new) {
                column_data.insert("filename_relative_path".to_owned(), filename_relative_path_line_edit.text().to_std_string());
            }

            let is_reference_value = field.is_reference(patches);
            let is_reference_value_new = is_reference_line_edit.text().to_std_string();
            if !is_reference_value_new.is_empty() && (is_reference_value.is_none() || format!("{};{}", is_reference_value.as_ref().unwrap().0, is_reference_value.as_ref().unwrap().1) != is_reference_value_new) {
                column_data.insert("is_reference".to_owned(), is_reference_line_edit.text().to_std_string());
            }

            let lookup_value = field.lookup(patches);
            let lookup_value_new = lookup_line_edit.text().to_std_string();
            if !lookup_value_new.is_empty() && (lookup_value.is_none() || lookup_value.unwrap().join(";") != lookup_value_new) {
                column_data.insert("lookup".to_owned(), lookup_line_edit.text().to_std_string());
            }

            if field.cannot_be_empty(patches) != not_empty_checkbox.is_checked() {
                column_data.insert("not_empty".to_owned(), not_empty_checkbox.is_checked().to_string());
            }

            if field.unused(patches) != unused_checkbox.is_checked() {
                column_data.insert("unused".to_owned(), unused_checkbox.is_checked().to_string());
            }

            let description_value = field.description(patches);
            let description_value_new = description_text_edit.to_plain_text().to_std_string();
            if !description_value_new.is_empty() && description_value_new != description_value {
                column_data.insert("description".to_owned(), description_value_new);
            }

            let mut patch = HashMap::new();
            let mut table_data = HashMap::new();
            table_data.insert(field.name().to_owned(), column_data);
            patch.insert(edited_table_name.to_owned(), table_data);

            let receiver = CENTRAL_COMMAND.send_background(Command::SaveLocalSchemaPatch(patch));
            let response = CentralCommand::recv(&receiver);
            match response {
                Response::Success => show_dialog(self.table_view(), tr("patch_success"), true),
                Response::Error(error) => return Err(error),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }

        Ok(())
    }

    pub unsafe fn new_profile_dialog(&self) -> Result<()> {

        // We only want to do this for tables we can identify.
        if self.table_name().is_none() {
            return Err(anyhow!("This is either not a DB Table, or it's a DB Table but it's corrupted."));
        }

        // Create and configure the dialog.
        let view_name = if cfg!(debug_assertions) { NEW_PROFILE_VIEW_DEBUG } else { NEW_PROFILE_VIEW_RELEASE };
        let template_path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), view_name);
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, &self.table_view);

        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        let dialog = main_widget.static_downcast::<QDialog>();
        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_close());
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Setup translations.
        dialog.set_window_title(&qtr("new_profile_title"));
        instructions_label.set_text(&qtr("new_profile_instructions"));
        name_line_edit.set_placeholder_text(&qtr("new_profile_placeholder_text"));

        // Launch.
        if dialog.exec() == 1 {
            let name = name_line_edit.text();
            if name.is_empty() || name.to_std_string() == "profile_default" {
                show_dialog(&self.table_view, tr("new_profile_no_name_error"), false);
                return Ok(())
            }

            self.new_table_view_profile(&name.to_std_string());
            self.save_table_view_profiles()?;
        }

        Ok(())
    }

    /// This function tries to open the source of a reference/loc key, if exists.
    ///
    /// If the source it's not found, it does nothing.
    pub unsafe fn go_to_definition(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Option<String> {

        let mut error_message = String::new();
        let indexes = self.table_view.selection_model().selection().indexes();
        if indexes.count_0a() > 0 {
            let ref_info = match *self.packed_file_type {

                // For DB, we just get the reference data, the first selected cell's data, and use that to search the source file.
                FileType::DB => {
                    let index = self.table_filter.map_to_source(self.table_view.selection_model().selection().indexes().at(0));
                    if let Some(field) = self.table_definition().fields_processed().get(index.column() as usize) {
                        if let Some((ref_table, ref_column)) = field.is_reference(Some(self.table_definition().patches())) {
                            Some((ref_table.to_owned(), ref_column.to_owned(), vec![index.data_0a().to_string().to_std_string()]))
                        } else { None }
                    } else { None }
                }

                // For Locs, we use the column 0 of the row with the selected item.
                FileType::Loc => {
                    let index_row = self.table_filter.map_to_source(self.table_view.selection_model().selection().indexes().at(0)).row();
                    let key = self.table_model.index_2a(index_row, 0).data_0a().to_string().to_std_string();
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetSourceDataFromLocKey(key));
                    let response = CENTRAL_COMMAND.recv_try(&receiver);
                    match response {
                        Response::OptionStringStringVecString(response) => response,
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }
                }
                _ => None,
            };

            if let Some((ref_table, ref_column, ref_data)) = ref_info {

                // Save the tables that may be the source before searching, to ensure their data is updated.
                let ref_path = format!("db/{ref_table}");
                UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).for_each(|file_view| {
                    if file_view.path_copy().starts_with(&ref_path) {
                        let _ = file_view.save(app_ui, pack_file_contents_ui);
                    }
                });

                // Then ask the backend to do the heavy work.
                let receiver = CENTRAL_COMMAND.send_background(Command::GoToDefinition(ref_table, ref_column, ref_data));
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {

                    // We receive a path/column/row, so we know what to open/select.
                    Response::DataSourceStringUsizeUsize(data_source, path, column, row) => {
                        match data_source {
                            DataSource::PackFile => {
                                let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&path, data_source);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view().static_upcast::<QObject>());
                                        pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                        pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ParentFiles |
                            DataSource::AssKitFiles |
                            DataSource::GameFiles => {
                                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::GameFiles);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ExternalFile => {},
                        }

                        // Set the current file as non-preview, so it doesn't close when opening the source one.
                        if let Some(packed_file_path) = self.get_packed_file_path() {
                            if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *packed_file_path && x.data_source() == self.get_data_source()) {
                                file_view.set_is_preview(false);
                            }
                        }

                        // Open the table and select the cell.
                        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_owned()), true, false, data_source);
                        if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == path && x.data_source() == data_source) {
                            if let ViewType::Internal(View::Table(view)) = file_view.view_type() {
                                let table_view = view.get_ref_table();
                                let table_view = table_view.table_view_ptr();
                                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                                let table_selection_model = table_view.selection_model();

                                let table_model_index = table_model.index_2a(row as i32, column as i32);
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                                }
                            }
                        }
                    }

                    Response::Error(error) => error_message = error.to_string(),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            } else {
                error_message = tr("source_data_for_field_not_found");
            }
        }

        if error_message.is_empty() { None }
        else { Some(error_message) }
    }

    /// This function tries to open the file referenced by a key, if exists.
    ///
    /// If the file it's not found, it does nothing.
    pub unsafe fn go_to_file(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Option<String> {

        let mut error_message = String::new();
        let indexes = self.table_view.selection_model().selection().indexes();
        if indexes.count_0a() > 0 {
            let paths = match *self.packed_file_type {

                // We just get the path for the first cell selected.
                FileType::DB => {
                    let index = self.table_filter.map_to_source(self.table_view.selection_model().selection().indexes().at(0));
                    let table_definition = self.table_definition();
                    let patches = table_definition.patches();
                    if let Some(field) = self.table_definition().fields_processed().get(index.column() as usize) {
                        if field.is_filename(Some(patches)) {

                            // If there are paths, map them with the cell data.
                            if let Some(raw_paths) = field.filename_relative_path(Some(patches)) {
                                let mut map_paths = vec![];
                                let cell_data = index.data_0a().to_string().to_std_string();
                                for path in raw_paths {
                                    map_paths.push(ContainerPath::File(path.replace("%", &cell_data)));
                                }

                                Some(map_paths)
                            }

                            // If there are not paths, consider the cell data a full path.
                            else { Some(vec![ContainerPath::File(index.data_0a().to_string().to_std_string())]) }
                        } else { None }
                    } else { None }
                }
                _ => None,
            };

            if let Some(paths) = paths {
                if !paths.is_empty() {

                    // Ask the backend to know what paths we have as files.
                    let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(paths.clone(), true));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                            let mut file = None;

                            // Set the current file as non-preview, so it doesn't close when opening the source one.
                            if let Some(packed_file_path) = self.get_packed_file_path() {
                                if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *packed_file_path && x.data_source() == self.get_data_source()) {
                                    file_view.set_is_preview(false);
                                }
                            }

                            if let Some(files) = files.remove(&DataSource::GameFiles) {
                                let mut paths = files.keys().collect::<Vec<_>>();
                                paths.sort();
                                if let Some(path) = paths.first() {
                                    let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(path, DataSource::GameFiles);
                                    if let Some(ref tree_index) = tree_index {
                                        if tree_index.is_valid() {
                                            let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                                            dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                            dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                        }
                                    }

                                    file = Some((DataSource::GameFiles, path.to_string()));
                                }
                            }

                            if let Some(files) = files.remove(&DataSource::ParentFiles) {
                                let mut paths = files.keys().collect::<Vec<_>>();
                                paths.sort();
                                if let Some(path) = paths.first() {
                                    let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(path, DataSource::ParentFiles);
                                    if let Some(ref tree_index) = tree_index {
                                        if tree_index.is_valid() {
                                            let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                                            dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                            dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                        }
                                    }

                                    file = Some((DataSource::ParentFiles, path.to_string()));
                                }
                            }

                            if let Some(files) = files.remove(&DataSource::PackFile) {
                                let mut paths = files.keys().collect::<Vec<_>>();
                                paths.sort();
                                if let Some(path) = paths.first() {
                                    let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(path, DataSource::PackFile);
                                    if let Some(ref tree_index) = tree_index {
                                        if tree_index.is_valid() {
                                            let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view().static_upcast::<QObject>());
                                            pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                            pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));

                                        }
                                    }

                                    file = Some((DataSource::PackFile, path.to_string()));
                                }
                            }

                            // If we have a file and its data source, open it.
                            if let Some((data_source, path)) = file {
                                AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_owned()), true, false, data_source);
                            }
                        },
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    };
                } else {
                    error_message = tr("file_for_field_not_found");
                }
            } else {
                error_message = tr("file_for_field_not_found");
            }
        }

        if error_message.is_empty() { None }
        else { Some(error_message) }
    }

    /// This function tries to open the loc data related with the currently selected row.
    ///
    /// If the loc data it's not found, it does nothing.
    /// If the loc data is a read-only dependency, it does nothing yet.
    pub unsafe fn go_to_loc(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        loc_column_name: &str
    ) -> Option<String> {

        // This is only for DB Tables, and we need to have something selected.
        let indexes = self.table_view.selection_model().selection().indexes();
        let mut error_message = String::new();
        if indexes.count_0a() > 0 {
            if let FileType::DB = *self.packed_file_type {

                // Save the currently open locs, to ensure the backend has the most up-to-date data.
                UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).for_each(|file_view| {
                    if let FileType::Loc = file_view.file_type() {
                        let _ = file_view.save(app_ui, pack_file_contents_ui);
                    }
                });

                // Get the name of the table and the key of the selected row to know what loc key to search.
                let table_name = if let Some(ref table_name) = self.table_name {
                    table_name.to_owned().drain(..table_name.len() - 7).collect::<String>()
                } else { return Some(tr("loc_key_not_found")) };

                let table_definition = self.table_definition();
                let key_field_positions = table_definition.localised_key_order();

                let key = key_field_positions.iter()
                    .map(|column|
                        self.table_model.index_2a(
                            self.table_filter.map_to_source(self.table_view.selection_model().selection().indexes().at(0)).row(),
                            *column as i32
                        ).data_0a().to_string().to_std_string()
                    ).join("");
                let loc_key = format!("{table_name}_{loc_column_name}_{key}");

                // Then ask the backend to do the heavy work.
                let receiver = CENTRAL_COMMAND.send_background(Command::GoToLoc(loc_key));
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {

                    // We receive a path/column/row, so we know what to open/select.
                    Response::DataSourceStringUsizeUsize(data_source, path, column, row) => {
                        match data_source {
                            DataSource::PackFile => {
                                let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&path, data_source);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view().static_upcast::<QObject>());
                                        pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                        pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ParentFiles |
                            DataSource::AssKitFiles |
                            DataSource::GameFiles => {
                                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, data_source);
                                if let Some(ref tree_index) = tree_index {
                                    if tree_index.is_valid() {
                                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    }
                                }
                            },
                            DataSource::ExternalFile => {},
                        }

                        // Set the current file as non-preview, so it doesn't close when opening the source one.
                        if let Some(packed_file_path) = self.get_packed_file_path() {
                            if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == *packed_file_path && x.data_source() == self.get_data_source()) {
                                file_view.set_is_preview(false);
                            }
                        }

                        // Open the table and select the cell.
                        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui,Some(path.to_owned()), true, false, data_source);
                        if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.path_read() == path && x.data_source() == data_source) {
                            if let ViewType::Internal(View::Table(view)) = file_view.view_type() {
                                let table_view = view.get_ref_table();
                                let table_view = table_view.table_view_ptr();
                                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                                let table_selection_model = table_view.selection_model();

                                let table_model_index = table_model.index_2a(row as i32, column as i32);
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                                }
                            }
                        }
                    }

                    Response::Error(error) => error_message = error.to_string(),
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            }
        }

        if error_message.is_empty() { None }
        else { Some(error_message) }
    }

    /// This function clears the markings for added/modified cells.
    pub unsafe fn clear_markings(&self) {
        let table_view = self.table_view_ptr();
        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
        let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

        for row in 0..table_model.row_count_0a() {
            for column in 0..table_model.column_count_0a() {
                let item = table_model.item_2a(row, column);

                if item.data_1a(ITEM_IS_ADDED).to_bool() {
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_ADDED);
                }

                if item.data_1a(ITEM_IS_MODIFIED).to_bool() {
                    item.set_data_2a(&QVariant::from_bool(false), ITEM_IS_MODIFIED);
                }
            }
        }
        blocker.unblock();
        table_view.viewport().repaint();
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
            Self::AddRows(data) => write!(f, "Removing row/s added in position/s {}.", data.iter().map(|x| x.to_string()).join(", ")),
            Self::RemoveRows(data) => write!(f, "Re-adding row/s removed in {} batches.", data.len()),
            //Self::ImportTSV(_) => write!(f, "Imported TSV file."),
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
