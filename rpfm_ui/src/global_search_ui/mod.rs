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
Module with all the code related to the `GlobalSearchSlots`.

This module contains all the code needed to initialize the Global Search Panel.
!*/

use qt_widgets::q_abstract_item_view::{ScrollHint, ScrollMode};
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDockWidget;
use qt_widgets::QGroupBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QPushButton;
use qt_widgets::QRadioButton;
use qt_widgets::QTabWidget;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::{CaseSensitivity, DockWidgetArea, Orientation, SortOrder};
use qt_core::QObject;
use qt_core::QRegExp;
use qt_core::QSignalBlocker;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;

use cpp_core::Ptr;

use std::rc::Rc;

use rpfm_error::ErrorKind;
use rpfm_lib::global_search::{GlobalSearch, MatchHolder, SearchSource, schema::SchemaMatches, table::{TableMatches, TableMatch}, text::TextMatches};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::locale::qtr;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::QString;
use crate::references_ui::ReferencesUI;
use crate::utils::{create_grid_layout, show_dialog};
use crate::UI_STATE;

pub mod connections;
pub mod shortcuts;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Global Search panel.
pub struct GlobalSearchUI {
    pub global_search_dock_widget: QBox<QDockWidget>,
    pub global_search_search_combobox: QBox<QComboBox>,
    pub global_search_search_button: QBox<QPushButton>,

    pub global_search_replace_line_edit: QBox<QLineEdit>,
    pub global_search_replace_button: QBox<QPushButton>,
    pub global_search_replace_all_button: QBox<QPushButton>,

    pub global_search_clear_button: QBox<QPushButton>,
    pub global_search_case_sensitive_checkbox: QBox<QCheckBox>,
    pub global_search_use_regex_checkbox: QBox<QCheckBox>,

    pub global_search_search_source_packfile: QBox<QRadioButton>,
    pub global_search_search_source_parent: QBox<QRadioButton>,
    pub global_search_search_source_game: QBox<QRadioButton>,
    pub global_search_search_source_asskit: QBox<QRadioButton>,

    pub global_search_search_on_all_checkbox: QBox<QCheckBox>,
    pub global_search_search_on_dbs_checkbox: QBox<QCheckBox>,
    pub global_search_search_on_locs_checkbox: QBox<QCheckBox>,
    pub global_search_search_on_texts_checkbox: QBox<QCheckBox>,
    pub global_search_search_on_schemas_checkbox: QBox<QCheckBox>,

    pub global_search_matches_tab_widget: QBox<QTabWidget>,

    pub global_search_matches_db_tree_view: QBox<QTreeView>,
    pub global_search_matches_loc_tree_view: QBox<QTreeView>,
    pub global_search_matches_text_tree_view: QBox<QTreeView>,
    pub global_search_matches_schema_tree_view: QBox<QTreeView>,

    pub global_search_matches_db_tree_filter: QBox<QSortFilterProxyModel>,
    pub global_search_matches_loc_tree_filter: QBox<QSortFilterProxyModel>,
    pub global_search_matches_text_tree_filter: QBox<QSortFilterProxyModel>,
    pub global_search_matches_schema_tree_filter: QBox<QSortFilterProxyModel>,

    pub global_search_matches_db_tree_model: QBox<QStandardItemModel>,
    pub global_search_matches_loc_tree_model: QBox<QStandardItemModel>,
    pub global_search_matches_text_tree_model: QBox<QStandardItemModel>,
    pub global_search_matches_schema_tree_model: QBox<QStandardItemModel>,

    pub global_search_matches_filter_db_line_edit: QBox<QLineEdit>,
    pub global_search_matches_filter_loc_line_edit: QBox<QLineEdit>,
    pub global_search_matches_filter_text_line_edit: QBox<QLineEdit>,
    pub global_search_matches_filter_schema_line_edit: QBox<QLineEdit>,

    pub global_search_matches_case_sensitive_db_button: QBox<QPushButton>,
    pub global_search_matches_case_sensitive_loc_button: QBox<QPushButton>,
    pub global_search_matches_case_sensitive_text_button: QBox<QPushButton>,
    pub global_search_matches_case_sensitive_schema_button: QBox<QPushButton>,

    pub global_search_matches_column_selector_db_combobox: QBox<QComboBox>,
    pub global_search_matches_column_selector_loc_combobox: QBox<QComboBox>,
    pub global_search_matches_column_selector_text_combobox: QBox<QComboBox>,
    pub global_search_matches_column_selector_schema_combobox: QBox<QComboBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchUI`.
impl GlobalSearchUI {

    /// This function creates an entire `GlobalSearchUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Self {

        // Create and configure the 'Global Search` Dock Widget and all his contents.
        let global_search_dock_widget = QDockWidget::from_q_widget(main_window);
        let global_search_dock_inner_widget = QWidget::new_1a(&global_search_dock_widget);
        let global_search_dock_layout = create_grid_layout(global_search_dock_inner_widget.static_upcast());
        global_search_dock_widget.set_widget(&global_search_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::RightDockWidgetArea, &global_search_dock_widget);
        global_search_dock_widget.set_window_title(&qtr("global_search"));
        global_search_dock_widget.set_object_name(&QString::from_std_str("global_search_dock"));

        // Create the search & replace section.
        let global_search_search_frame = QGroupBox::from_q_string_q_widget(&qtr("global_search_info"), &global_search_dock_inner_widget);
        let global_search_search_grid = create_grid_layout(global_search_search_frame.static_upcast());

        let global_search_search_combobox = QComboBox::new_1a(&global_search_search_frame);
        let global_search_search_button = QPushButton::from_q_string_q_widget(&qtr("global_search_search"), &global_search_search_frame);
        global_search_search_combobox.set_editable(true);

        let global_search_replace_line_edit = QLineEdit::from_q_widget(&global_search_search_frame);
        let global_search_replace_button = QPushButton::from_q_string_q_widget(&qtr("global_search_replace"), &global_search_search_frame);
        let global_search_replace_all_button = QPushButton::from_q_string_q_widget(&qtr("global_search_replace_all"), &global_search_search_frame);

        let global_search_clear_button = QPushButton::from_q_string_q_widget(&qtr("global_search_clear"), &global_search_search_frame);
        let global_search_case_sensitive_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_case_sensitive"), &global_search_search_frame);
        let global_search_use_regex_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_use_regex"), &global_search_search_frame);

        let global_search_search_source_group_box = QGroupBox::from_q_string_q_widget(&qtr("global_search_search_source"), &global_search_search_frame);
        let global_search_search_source_grid = create_grid_layout(global_search_search_source_group_box.static_upcast());

        let global_search_search_source_packfile = QRadioButton::from_q_string_q_widget(&qtr("global_search_source_packfile"), &global_search_search_source_group_box);
        let global_search_search_source_parent = QRadioButton::from_q_string_q_widget(&qtr("global_search_source_parent"), &global_search_search_source_group_box);
        let global_search_search_source_game = QRadioButton::from_q_string_q_widget(&qtr("global_search_source_game"), &global_search_search_source_group_box);
        let global_search_search_source_asskit = QRadioButton::from_q_string_q_widget(&qtr("global_search_source_asskit"), &global_search_search_source_group_box);
        global_search_search_source_packfile.set_checked(true);

        let global_search_search_on_group_box = QGroupBox::from_q_string_q_widget(&qtr("global_search_search_on"), &global_search_search_frame);
        let global_search_search_on_grid = create_grid_layout(global_search_search_on_group_box.static_upcast());

        let global_search_search_on_all_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_all"), &global_search_search_on_group_box);
        let global_search_search_on_dbs_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_db"), &global_search_search_on_group_box);
        let global_search_search_on_locs_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_loc"), &global_search_search_on_group_box);
        let global_search_search_on_texts_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_txt"), &global_search_search_on_group_box);
        let global_search_search_on_schemas_checkbox = QCheckBox::from_q_string_q_widget(&qtr("global_search_schemas"), &global_search_search_on_group_box);
        global_search_search_on_all_checkbox.set_checked(true);
        global_search_search_on_dbs_checkbox.set_disabled(true);
        global_search_search_on_locs_checkbox.set_disabled(true);
        global_search_search_on_texts_checkbox.set_disabled(true);
        global_search_search_on_schemas_checkbox.set_disabled(true);

        global_search_search_grid.set_column_stretch(0, 10);

        // Add everything to the Matches's Dock Layout.
        global_search_search_grid.add_widget_5a(&global_search_search_combobox, 0, 0, 1, 2);
        global_search_search_grid.add_widget_5a(&global_search_replace_line_edit, 1, 0, 1, 2);
        global_search_search_grid.add_widget_5a(&global_search_search_button, 0, 2, 1, 1);
        global_search_search_grid.add_widget_5a(&global_search_replace_button, 1, 2, 1, 1);
        global_search_search_grid.add_widget_5a(&global_search_replace_all_button, 1, 3, 1, 1);

        global_search_search_grid.add_widget_5a(&global_search_clear_button, 0, 3, 1, 1);
        global_search_search_grid.add_widget_5a(&global_search_case_sensitive_checkbox, 0, 4, 1, 1);
        global_search_search_grid.add_widget_5a(&global_search_use_regex_checkbox, 1, 4, 1, 1);
        global_search_search_grid.add_widget_5a(&global_search_search_source_group_box, 2, 0, 1, 10);
        global_search_search_grid.add_widget_5a(&global_search_search_on_group_box, 3, 0, 1, 10);

        global_search_search_source_grid.add_widget_5a(&global_search_search_source_packfile, 0, 0, 1, 1);
        global_search_search_source_grid.add_widget_5a(&global_search_search_source_parent, 0, 1, 1, 1);
        global_search_search_source_grid.add_widget_5a(&global_search_search_source_game, 0, 2, 1, 1);
        global_search_search_source_grid.add_widget_5a(&global_search_search_source_asskit, 0, 3, 1, 1);

        global_search_search_on_grid.add_widget_5a(&global_search_search_on_all_checkbox, 0, 0, 1, 1);
        global_search_search_on_grid.add_widget_5a(&global_search_search_on_dbs_checkbox, 0, 1, 1, 1);
        global_search_search_on_grid.add_widget_5a(&global_search_search_on_locs_checkbox, 0, 2, 1, 1);
        global_search_search_on_grid.add_widget_5a(&global_search_search_on_texts_checkbox, 0, 3, 1, 1);
        global_search_search_on_grid.add_widget_5a(&global_search_search_on_schemas_checkbox, 0, 4, 1, 1);

        // Create the frames for the matches tables.
        let global_search_matches_tab_widget = QTabWidget::new_1a(&global_search_dock_inner_widget);

        let db_matches_widget = QWidget::new_1a(&global_search_matches_tab_widget);
        let db_matches_grid = create_grid_layout(db_matches_widget.static_upcast());

        let loc_matches_widget = QWidget::new_1a(&global_search_matches_tab_widget);
        let loc_matches_grid = create_grid_layout(loc_matches_widget.static_upcast());

        let text_matches_widget = QWidget::new_1a(&global_search_matches_tab_widget);
        let text_matches_grid = create_grid_layout(text_matches_widget.static_upcast());

        let schema_matches_widget = QWidget::new_1a(&global_search_matches_tab_widget);
        let schema_matches_grid = create_grid_layout(schema_matches_widget.static_upcast());

        // `TreeView`s with all the matches.
        let tree_view_matches_db = QTreeView::new_1a(&db_matches_widget);
        let tree_view_matches_loc = QTreeView::new_1a(&loc_matches_widget);
        let tree_view_matches_text = QTreeView::new_1a(&text_matches_widget);
        let tree_view_matches_schema = QTreeView::new_1a(&schema_matches_widget);

        let filter_model_matches_db = new_treeview_filter_safe(tree_view_matches_db.static_upcast());
        let filter_model_matches_loc = new_treeview_filter_safe(tree_view_matches_loc.static_upcast());
        let filter_model_matches_text = new_treeview_filter_safe(tree_view_matches_text.static_upcast());
        let filter_model_matches_schema = new_treeview_filter_safe(tree_view_matches_schema.static_upcast());

        let model_matches_db = QStandardItemModel::new_1a(&tree_view_matches_db);
        let model_matches_loc = QStandardItemModel::new_1a(&tree_view_matches_loc);
        let model_matches_text = QStandardItemModel::new_1a(&tree_view_matches_text);
        let model_matches_schema = QStandardItemModel::new_1a(&tree_view_matches_schema);

        tree_view_matches_db.set_model(&filter_model_matches_db);
        tree_view_matches_loc.set_model(&filter_model_matches_loc);
        tree_view_matches_text.set_model(&filter_model_matches_text);
        tree_view_matches_schema.set_model(&filter_model_matches_schema);

        filter_model_matches_db.set_source_model(&model_matches_db);
        filter_model_matches_loc.set_source_model(&model_matches_loc);
        filter_model_matches_text.set_source_model(&model_matches_text);
        filter_model_matches_schema.set_source_model(&model_matches_schema);

        tree_view_matches_db.set_horizontal_scroll_mode(ScrollMode::ScrollPerPixel);
        tree_view_matches_db.set_sorting_enabled(true);
        tree_view_matches_db.header().set_visible(true);
        tree_view_matches_db.header().set_stretch_last_section(true);

        tree_view_matches_loc.set_horizontal_scroll_mode(ScrollMode::ScrollPerPixel);
        tree_view_matches_loc.set_sorting_enabled(true);
        tree_view_matches_loc.header().set_visible(true);
        tree_view_matches_loc.header().set_stretch_last_section(true);

        tree_view_matches_text.set_horizontal_scroll_mode(ScrollMode::ScrollPerPixel);
        tree_view_matches_text.set_sorting_enabled(true);
        tree_view_matches_text.header().set_visible(true);
        tree_view_matches_text.header().set_stretch_last_section(true);

        tree_view_matches_schema.set_horizontal_scroll_mode(ScrollMode::ScrollPerPixel);
        tree_view_matches_schema.set_sorting_enabled(true);
        tree_view_matches_schema.header().set_visible(true);
        tree_view_matches_schema.header().set_stretch_last_section(true);

        // Filters for the matches `TreeViews`.
        let filter_matches_db_line_edit = QLineEdit::from_q_widget(&db_matches_widget);
        let filter_matches_db_column_selector = QComboBox::new_1a(&db_matches_widget);
        let filter_matches_db_column_list = QStandardItemModel::new_1a(&db_matches_widget);
        let filter_matches_db_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("global_search_case_sensitive"), &db_matches_widget);

        filter_matches_db_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_db_column_selector.set_model(&filter_matches_db_column_list);
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_db_case_sensitive_button.set_checkable(true);

        let filter_matches_loc_line_edit = QLineEdit::from_q_widget(&loc_matches_widget);
        let filter_matches_loc_column_selector = QComboBox::new_1a(&loc_matches_widget);
        let filter_matches_loc_column_list = QStandardItemModel::new_1a(&loc_matches_widget);
        let filter_matches_loc_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("global_search_case_sensitive"), &loc_matches_widget);

        filter_matches_loc_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_loc_column_selector.set_model(&filter_matches_loc_column_list);
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_loc_case_sensitive_button.set_checkable(true);

        let filter_matches_text_line_edit = QLineEdit::from_q_widget(&text_matches_widget);
        let filter_matches_text_column_selector = QComboBox::new_1a(&text_matches_widget);
        let filter_matches_text_column_list = QStandardItemModel::new_1a(&text_matches_widget);
        let filter_matches_text_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("global_search_case_sensitive"), &text_matches_widget);

        filter_matches_text_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_text_column_selector.set_model(&filter_matches_text_column_list);
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_text_case_sensitive_button.set_checkable(true);

        let filter_matches_schema_line_edit = QLineEdit::from_q_widget(&schema_matches_widget);
        let filter_matches_schema_column_selector = QComboBox::new_1a(&schema_matches_widget);
        let filter_matches_schema_column_list = QStandardItemModel::new_1a(&schema_matches_widget);
        let filter_matches_schema_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("global_search_case_sensitive"), &schema_matches_widget);

        filter_matches_schema_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_schema_column_selector.set_model(&filter_matches_schema_column_list);
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_schema_case_sensitive_button.set_checkable(true);

        // Add everything to the Matches's Dock Layout.
        db_matches_grid.add_widget_5a(&tree_view_matches_db, 0, 0, 1, 3);
        loc_matches_grid.add_widget_5a(&tree_view_matches_loc, 0, 0, 1, 3);
        text_matches_grid.add_widget_5a(&tree_view_matches_text, 0, 0, 1, 3);
        schema_matches_grid.add_widget_5a(&tree_view_matches_schema, 0, 0, 1, 3);

        db_matches_grid.add_widget_5a(&filter_matches_db_line_edit, 1, 0, 1, 1);
        db_matches_grid.add_widget_5a(&filter_matches_db_case_sensitive_button, 1, 1, 1, 1);
        db_matches_grid.add_widget_5a(&filter_matches_db_column_selector, 1, 2, 1, 1);

        loc_matches_grid.add_widget_5a(&filter_matches_loc_line_edit, 1, 0, 1, 1);
        loc_matches_grid.add_widget_5a(&filter_matches_loc_case_sensitive_button, 1, 1, 1, 1);
        loc_matches_grid.add_widget_5a(&filter_matches_loc_column_selector, 1, 2, 1, 1);

        text_matches_grid.add_widget_5a(&filter_matches_text_line_edit, 1, 0, 1, 1);
        text_matches_grid.add_widget_5a(&filter_matches_text_case_sensitive_button, 1, 1, 1, 1);
        text_matches_grid.add_widget_5a(&filter_matches_text_column_selector, 1, 2, 1, 1);

        schema_matches_grid.add_widget_5a(&filter_matches_schema_line_edit, 1, 0, 1, 1);
        schema_matches_grid.add_widget_5a(&filter_matches_schema_case_sensitive_button, 1, 1, 1, 1);
        schema_matches_grid.add_widget_5a(&filter_matches_schema_column_selector, 1, 2, 1, 1);

        global_search_matches_tab_widget.add_tab_2a(&db_matches_widget, &qtr("global_search_db_matches"));
        global_search_matches_tab_widget.add_tab_2a(&loc_matches_widget, &qtr("global_search_loc_matches"));
        global_search_matches_tab_widget.add_tab_2a(&text_matches_widget, &qtr("global_search_txt_matches"));
        global_search_matches_tab_widget.add_tab_2a(&schema_matches_widget, &qtr("global_search_schema_matches"));

        global_search_dock_layout.add_widget_5a(&global_search_search_frame, 0, 0, 1, 3);
        global_search_dock_layout.add_widget_5a(&global_search_matches_tab_widget, 1, 0, 1, 3);

        // Hide this widget by default.
        global_search_dock_widget.hide();

        // Create ***Da monsta***.
        Self {
            global_search_dock_widget,
            global_search_search_combobox,
            global_search_search_button,

            global_search_replace_line_edit,
            global_search_replace_button,
            global_search_replace_all_button,

            global_search_clear_button,
            global_search_case_sensitive_checkbox,
            global_search_use_regex_checkbox,

            global_search_search_source_packfile,
            global_search_search_source_parent,
            global_search_search_source_game,
            global_search_search_source_asskit,

            global_search_search_on_all_checkbox,
            global_search_search_on_dbs_checkbox,
            global_search_search_on_locs_checkbox,
            global_search_search_on_texts_checkbox,
            global_search_search_on_schemas_checkbox,

            global_search_matches_tab_widget,

            global_search_matches_db_tree_view: tree_view_matches_db,
            global_search_matches_loc_tree_view: tree_view_matches_loc,
            global_search_matches_text_tree_view: tree_view_matches_text,
            global_search_matches_schema_tree_view: tree_view_matches_schema,

            global_search_matches_db_tree_filter: filter_model_matches_db,
            global_search_matches_loc_tree_filter: filter_model_matches_loc,
            global_search_matches_text_tree_filter: filter_model_matches_text,
            global_search_matches_schema_tree_filter: filter_model_matches_schema,

            global_search_matches_db_tree_model: model_matches_db,
            global_search_matches_loc_tree_model: model_matches_loc,
            global_search_matches_text_tree_model: model_matches_text,
            global_search_matches_schema_tree_model: model_matches_schema,

            global_search_matches_filter_db_line_edit: filter_matches_db_line_edit,
            global_search_matches_filter_loc_line_edit: filter_matches_loc_line_edit,
            global_search_matches_filter_text_line_edit: filter_matches_text_line_edit,
            global_search_matches_filter_schema_line_edit: filter_matches_schema_line_edit,

            global_search_matches_case_sensitive_db_button: filter_matches_db_case_sensitive_button,
            global_search_matches_case_sensitive_loc_button: filter_matches_loc_case_sensitive_button,
            global_search_matches_case_sensitive_text_button: filter_matches_text_case_sensitive_button,
            global_search_matches_case_sensitive_schema_button: filter_matches_schema_case_sensitive_button,

            global_search_matches_column_selector_db_combobox: filter_matches_db_column_selector,
            global_search_matches_column_selector_loc_combobox: filter_matches_loc_column_selector,
            global_search_matches_column_selector_text_combobox: filter_matches_text_column_selector,
            global_search_matches_column_selector_schema_combobox: filter_matches_schema_column_selector,
        }
    }

    /// This function is used to search the entire PackFile, using the data in Self for the search.
    pub unsafe fn search(
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<Self>,
    ) {

        // Create the global search and populate it with all the settings for the search.
        let mut global_search = GlobalSearch {
            pattern: global_search_ui.global_search_search_combobox.current_text().to_std_string(),
            case_sensitive: global_search_ui.global_search_case_sensitive_checkbox.is_checked(),
            use_regex: global_search_ui.global_search_use_regex_checkbox.is_checked(),
            ..Default::default()
        };

        // If we don't have text to search, return.
        if global_search.pattern.is_empty() { return; }

        if global_search_ui.global_search_search_source_packfile.is_checked() {
            global_search.source = SearchSource::PackFile;
        } else if global_search_ui.global_search_search_source_parent.is_checked() {
            global_search.source = SearchSource::ParentFiles;
        } else if global_search_ui.global_search_search_source_game.is_checked() {
            global_search.source = SearchSource::GameFiles;
        } else if global_search_ui.global_search_search_source_asskit.is_checked() {
            global_search.source = SearchSource::AssKitFiles;
        }

        if global_search_ui.global_search_search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = global_search_ui.global_search_search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = global_search_ui.global_search_search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = global_search_ui.global_search_search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = global_search_ui.global_search_search_on_schemas_checkbox.is_checked();
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearch(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let tree_view_db = &global_search_ui.global_search_matches_db_tree_view;
        let tree_view_loc = &global_search_ui.global_search_matches_loc_tree_view;
        let tree_view_text = &global_search_ui.global_search_matches_text_tree_view;
        let tree_view_schema = &global_search_ui.global_search_matches_schema_tree_view;

        let model_db = &global_search_ui.global_search_matches_db_tree_model;
        let model_loc = &global_search_ui.global_search_matches_loc_tree_model;
        let model_text = &global_search_ui.global_search_matches_text_tree_model;
        let model_schema = &global_search_ui.global_search_matches_schema_tree_model;

        model_db.clear();
        model_loc.clear();
        model_text.clear();
        model_schema.clear();

        // Load the results to their respective models. Then, store the GlobalSearch for future checks.
        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {
                Self::load_table_matches_to_ui(model_db, tree_view_db, &global_search.matches_db);
                Self::load_table_matches_to_ui(model_loc, tree_view_loc, &global_search.matches_loc);
                Self::load_text_matches_to_ui(model_text, tree_view_text, &global_search.matches_text);
                Self::load_schema_matches_to_ui(model_schema, tree_view_schema, &global_search.matches_schema);
                UI_STATE.set_global_search(&global_search);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            _ => unimplemented!()
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub unsafe fn clear(
        global_search_ui: &Rc<Self>
    ) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        global_search_ui.global_search_matches_db_tree_model.clear();
        global_search_ui.global_search_matches_loc_tree_model.clear();
        global_search_ui.global_search_matches_text_tree_model.clear();
        global_search_ui.global_search_matches_schema_tree_model.clear();
    }

    /// This function replace the currently selected match with the provided text.
    pub unsafe fn replace_current(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, global_search_ui: &Rc<Self>) {

        let mut global_search = UI_STATE.get_global_search();

        if global_search.source != SearchSource::PackFile {
            return show_dialog(&app_ui.main_window, ErrorKind::GlobalReplaceOverDependencies, false);
        }

        global_search.pattern = global_search_ui.global_search_search_combobox.current_text().to_std_string();
        global_search.replace_text = global_search_ui.global_search_replace_line_edit.text().to_std_string();
        global_search.case_sensitive = global_search_ui.global_search_case_sensitive_checkbox.is_checked();
        global_search.use_regex = global_search_ui.global_search_use_regex_checkbox.is_checked();

        if global_search_ui.global_search_search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = global_search_ui.global_search_search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = global_search_ui.global_search_search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = global_search_ui.global_search_search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = global_search_ui.global_search_search_on_schemas_checkbox.is_checked();
        }

        let matches = Self::get_matches_from_selection(global_search_ui);
        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceMatches(global_search, matches.to_vec()));

        // While we wait for an answer, we need to clear the current results panels.
        global_search_ui.global_search_matches_db_tree_model.clear();
        global_search_ui.global_search_matches_loc_tree_model.clear();
        global_search_ui.global_search_matches_text_tree_model.clear();

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {
                UI_STATE.set_global_search(&global_search);
                Self::search(pack_file_contents_ui, global_search_ui);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);

                // Update the views of the updated PackedFiles.
                for replace_match in matches {
                    let path = match replace_match {
                        MatchHolder::Table(matches) => matches.path,
                        MatchHolder::Text(matches) => matches.path,
                        _ => unimplemented!(),
                    };

                    if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == path && x.get_data_source() == DataSource::PackFile) {
                        if let Err(error) = packed_file_view.reload(&path, pack_file_contents_ui) {
                            show_dialog(&app_ui.main_window, error, false);
                        }
                    }

                    // Set them as modified in the UI.
                }
            },
            _ => unimplemented!()
        }
    }

    /// This function replace all the matches in the current search with the provided text.
    pub unsafe fn replace_all(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<Self>,
    ) {

        // To avoid conflicting data, we close all PackedFiles hard and re-search before replacing.
        if let Err(error) = AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui) {
            return show_dialog(&app_ui.main_window, error, false);
        }

        Self::search(pack_file_contents_ui, global_search_ui);

        let mut global_search = UI_STATE.get_global_search();

        if global_search.source != SearchSource::PackFile {
            return show_dialog(&app_ui.main_window, ErrorKind::GlobalReplaceOverDependencies, false);
        }

        global_search.pattern = global_search_ui.global_search_search_combobox.current_text().to_std_string();
        global_search.replace_text = global_search_ui.global_search_replace_line_edit.text().to_std_string();
        global_search.case_sensitive = global_search_ui.global_search_case_sensitive_checkbox.is_checked();
        global_search.use_regex = global_search_ui.global_search_use_regex_checkbox.is_checked();

        if global_search_ui.global_search_search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = global_search_ui.global_search_search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = global_search_ui.global_search_search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = global_search_ui.global_search_search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = global_search_ui.global_search_search_on_schemas_checkbox.is_checked();
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceAll(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let model_db = &global_search_ui.global_search_matches_db_tree_model;
        let model_loc = &global_search_ui.global_search_matches_loc_tree_model;
        let model_text = &global_search_ui.global_search_matches_text_tree_model;

        model_db.clear();
        model_loc.clear();
        model_text.clear();

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {
                UI_STATE.set_global_search(&global_search);
                Self::search(pack_file_contents_ui, global_search_ui);

                for path in packed_files_info.iter().map(|x| &x.path) {
                    if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| &*x.get_ref_path() == path && x.get_data_source() == DataSource::PackFile) {
                        if let Err(error) = packed_file_view.reload(path, pack_file_contents_ui) {
                            show_dialog(&app_ui.main_window, error, false);
                        }
                    }
                }

                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            _ => unimplemented!()
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    ///
    /// Remember, it TRIES to open it. It may fail if the file doesn't exist anymore and the update search
    /// hasn't been triggered, or if the searched text doesn't exist anymore.
    ///
    /// In case the provided ModelIndex is the parent, we open the file without scrolling to the match.
    pub unsafe fn open_match(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        let gidhora = model.item_from_index(&model_index);
        let is_match = !gidhora.has_children();

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let path: Vec<String> = if is_match {
            let parent = gidhora.parent();

            // Sometimes this is null, not sure why.
            if parent.is_null() { return; }
            let path = parent.text().to_std_string();
            path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect()
        }

        // If not... just expand and open the PackedFile.
        else {
            let path = gidhora.text().to_std_string();
            path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect()
        };

        let global_search = UI_STATE.get_global_search();
        let data_source = match global_search.source {
            SearchSource::PackFile => {
                let tree_index = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path, DataSource::PackFile);

                // Manually select the open PackedFile, then open it. This means we can open PackedFiles nor in out filter.
                UI_STATE.set_packfile_contents_read_only(true);

                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        pack_file_contents_ui.packfile_contents_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                        pack_file_contents_ui.packfile_contents_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }

                UI_STATE.set_packfile_contents_read_only(false);
                DataSource::PackFile
            },

            SearchSource::ParentFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view.expand_treeview_to_item(&path, DataSource::ParentFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view.static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::ParentFiles
            },
            SearchSource::GameFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view.expand_treeview_to_item(&path, DataSource::GameFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view.static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::GameFiles
            },
            SearchSource::AssKitFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view.expand_treeview_to_item(&path, DataSource::AssKitFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view.static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::AssKitFiles
            },
        };

        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_vec()), false, false, data_source);

        // If it's a table, focus on the matched cell.
        if is_match {
            if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == data_source).find(|x| *x.get_ref_path() == path) {

                // In case of tables, we have to get the logical row/column of the match and select it.
                if let ViewType::Internal(View::Table(view)) = packed_file_view.get_view() {
                    let parent = gidhora.parent();
                    let table_view = view.get_ref_table();
                    let table_view = table_view.get_mut_ptr_table_view_primary();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let table_selection_model = table_view.selection_model();

                    let row = parent.child_2a(model_index.row(), 1).text().to_std_string().parse::<i32>().unwrap() - 1;
                    let column = parent.child_2a(model_index.row(), 3).text().to_std_string().parse::<i32>().unwrap();

                    let table_model_index = table_model.index_2a(row, column);
                    let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                    if table_model_index_filtered.is_valid() {
                        table_view.set_focus_0a();
                        table_view.set_current_index(table_model_index_filtered.as_ref());
                        table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                        table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    unsafe fn load_table_matches_to_ui(model: &QStandardItemModel, tree_view: &QTreeView, matches: &[TableMatches]) {
        if !matches.is_empty() {

            for match_table in matches {
                if !match_table.matches.is_empty() {
                    let path = match_table.path.join("/");
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = QStandardItem::new();
                    let fill1 = QStandardItem::new();
                    let fill2 = QStandardItem::new();
                    let fill3 = QStandardItem::new();
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_table.matches {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let column_name = QStandardItem::new();
                        let column_number = QStandardItem::new();
                        let row = QStandardItem::new();
                        let text = QStandardItem::new();

                        column_name.set_text(&QString::from_std_str(&match_row.column_name));
                        column_number.set_data_2a(&QVariant::from_uint(match_row.column_number), 2);
                        row.set_data_2a(&QVariant::from_i64(match_row.row_number + 1), 2);
                        text.set_text(&QString::from_std_str(&match_row.contents));

                        column_name.set_editable(false);
                        column_number.set_editable(false);
                        row.set_editable(false);
                        text.set_editable(false);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_number.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill3.into_ptr().as_mut_raw_ptr());

                    model.append_row_q_list_of_q_standard_item(qlist_daddy.as_ref());
                }
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_match_packedfile_column")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_row")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_match")));

            // Hide the column number column for tables.
            tree_view.hide_column(3);
            tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);

            tree_view.header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function takes care of loading the results of a global search of `TextMatches` into a model.
    unsafe fn load_text_matches_to_ui(model: &QStandardItemModel, tree_view: &QTreeView, matches: &[TextMatches]) {
        if !matches.is_empty() {
            for match_text in matches {
                if !match_text.matches.is_empty() {
                    let path = match_text.path.join("/");
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = QStandardItem::new();
                    let fill1 = QStandardItem::new();
                    let fill2 = QStandardItem::new();
                    let fill3 = QStandardItem::new();
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_text.matches {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = QStandardItem::new();
                        let row = QStandardItem::new();
                        let column = QStandardItem::new();
                        let len = QStandardItem::new();

                        text.set_text(&QString::from_std_str(&match_row.text));
                        row.set_data_2a(&QVariant::from_u64(match_row.row + 1), 2);
                        column.set_data_2a(&QVariant::from_u64(match_row.column), 2);
                        len.set_data_2a(&QVariant::from_i64(match_row.len), 2);

                        text.set_editable(false);
                        row.set_editable(false);
                        column.set_editable(false);
                        len.set_editable(false);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&len.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill3.into_ptr().as_mut_raw_ptr());

                    model.append_row_q_list_of_q_standard_item(qlist_daddy.as_ref());
                }
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_match_packedfile_text")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_row")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_column")));
            model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_length")));

            // Hide the column and length numbers on the TreeView.
            tree_view.hide_column(2);
            tree_view.hide_column(3);
            tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);

            tree_view.header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    unsafe fn load_schema_matches_to_ui(model: &QStandardItemModel, tree_view: &QTreeView, matches: &[SchemaMatches]) {
        if !matches.is_empty() {

            for match_schema in matches {
                if !match_schema.matches.is_empty() {
                    let qlist_daddy = QListOfQStandardItem::new();
                    let versioned_file = QStandardItem::new();
                    let fill1 = QStandardItem::new();
                    let fill2 = QStandardItem::new();

                    let name = if let Some(ref name) = match_schema.versioned_file_name {
                        format!("{}/{}", match_schema.versioned_file_type, name)
                    } else { match_schema.versioned_file_type.to_string() };

                    versioned_file.set_text(&QString::from_std_str(&name));
                    versioned_file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);

                    for match_row in &match_schema.matches {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let name = QStandardItem::new();
                        let version = QStandardItem::new();
                        let column = QStandardItem::new();

                        name.set_text(&QString::from_std_str(&match_row.name));
                        version.set_data_2a(&QVariant::from_int(match_row.version), 2);
                        column.set_data_2a(&QVariant::from_uint(match_row.column), 2);

                        name.set_editable(false);
                        version.set_editable(false);
                        column.set_editable(false);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&name.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&version.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        versioned_file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&versioned_file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());

                    model.append_row_q_list_of_q_standard_item(qlist_daddy.as_ref());
                }
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_versioned_file")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_definition_version")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column_index")));

            // Hide the column number column for tables.
            tree_view.hide_column(2);
            tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);

            tree_view.header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_results(
        view: &QBox<QTreeView>,
        line_edit: &QBox<QLineEdit>,
        column_combobox: &QBox<QComboBox>,
        case_sensitive_button: &QBox<QPushButton>,
    ) {

        let pattern = QRegExp::new_1a(&line_edit.text());

        let case_sensitive = case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        let model_filter: QPtr<QSortFilterProxyModel> = view.model().static_downcast();
        model_filter.set_filter_key_column(column_combobox.current_index());
        trigger_treeview_filter_safe(&model_filter, &pattern.as_ptr());
    }

    /// Function to get all the selected matches in the visible selection.
    unsafe fn get_matches_from_selection(global_search_ui: &Rc<Self>) -> Vec<MatchHolder> {

        let tree_view = match global_search_ui.global_search_matches_tab_widget.current_index() {
            0 => &global_search_ui.global_search_matches_db_tree_view,
            1 => &global_search_ui.global_search_matches_loc_tree_view,
            _ => return vec![],
        };

        let items = tree_view.get_items_from_selection(true);

        // For each item we follow the following logic:
        // - If it's a parent, it's all the matches on a table.
        // - If it's a child, check if the parent already exists.
        // - If it does, add another entry to it's matches.
        // - If not, create it with only that match.
        let mut matches: Vec<TableMatches> = vec![];
        for item in items {
            if item.column() == 0 {
                let is_match = !item.has_children();

                // If it's a match (not an entire file), get the entry and add it to the tablematches of that table.
                if is_match {
                    let parent = item.parent();
                    let path = parent.text().to_std_string();
                    let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

                    let match_file = match matches.iter_mut().find(|x| x.path == path) {
                        Some(match_file) => match_file,
                        None => {
                            let table = TableMatches::new(&path);
                            matches.push(table);
                            matches.last_mut().unwrap()
                        }
                    };

                    let column_name = parent.child_2a(item.row(), 0).text().to_std_string();
                    let column_number = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                    let row_number = parent.child_2a(item.row(), 1).text().to_std_string().parse::<i64>().unwrap() - 1;
                    let text = parent.child_2a(item.row(), 2).text().to_std_string();
                    let match_entry = TableMatch::new(&column_name, column_number, row_number, &text);

                    if !match_file.matches.contains(&match_entry) {
                        match_file.matches.push(match_entry);
                    }
                }

                // If it's not a particular match, it's an entire file.
                else {
                    let path = item.text().to_std_string();
                    let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

                    // If it already exists, delete it, as the new one contains the entire set for it.
                    if let Some(position) = matches.iter().position(|x| x.path == path) {
                        matches.remove(position);
                    }

                    let table = TableMatches::new(&path);
                    matches.push(table);
                    let match_file = matches.last_mut().unwrap();

                    // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                    for row in 0..item.row_count() {
                        let column_name = item.child_2a(row, 0).text().to_std_string();
                        let column_number = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                        let row_number = item.child_2a(row, 1).text().to_std_string().parse::<i64>().unwrap() - 1;
                        let text = item.child_2a(row, 2).text().to_std_string();
                        let match_entry = TableMatch::new(&column_name, column_number, row_number, &text);
                        match_file.matches.push(match_entry);
                    }
                }
            }
        }
        matches.iter().map(|x| MatchHolder::Table(x.clone())).collect()
    }
}
