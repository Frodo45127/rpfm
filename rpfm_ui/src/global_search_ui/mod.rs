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
Module with all the code related to the `GlobalSearchSlots`.

This module contains all the code needed to initialize the Global Search Panel.
!*/

use qt_widgets::q_abstract_item_view::ScrollMode;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDockWidget;
use qt_widgets::QGroupBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QPushButton;
use qt_widgets::QTabWidget;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::{CaseSensitivity, DockWidgetArea, Orientation, SortOrder};
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;

use cpp_core::MutPtr;
use cpp_core::Ptr;

use std::rc::Rc;
use std::cell::RefCell;

use rpfm_error::ErrorKind;

use rpfm_lib::packfile::PathType;
use rpfm_lib::global_search::{GlobalSearch, schema::SchemaMatches, table::TableMatches, text::TextMatches};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::{new_treeview_filter_safe, trigger_treeview_filter_safe};
use crate::locale::qtr;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{TheOneSlot, View};
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::QString;
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
#[derive(Copy, Clone)]
pub struct GlobalSearchUI {
    pub global_search_dock_widget: MutPtr<QDockWidget>,
    pub global_search_search_line_edit: MutPtr<QLineEdit>,
    pub global_search_search_button: MutPtr<QPushButton>,

    pub global_search_replace_line_edit: MutPtr<QLineEdit>,
    pub global_search_replace_button: MutPtr<QPushButton>,
    pub global_search_replace_all_button: MutPtr<QPushButton>,

    pub global_search_clear_button: MutPtr<QPushButton>,
    pub global_search_case_sensitive_checkbox: MutPtr<QCheckBox>,
    pub global_search_use_regex_checkbox: MutPtr<QCheckBox>,

    pub global_search_search_on_all_checkbox: MutPtr<QCheckBox>,
    pub global_search_search_on_dbs_checkbox: MutPtr<QCheckBox>,
    pub global_search_search_on_locs_checkbox: MutPtr<QCheckBox>,
    pub global_search_search_on_texts_checkbox: MutPtr<QCheckBox>,
    pub global_search_search_on_schemas_checkbox: MutPtr<QCheckBox>,

    pub global_search_matches_tab_widget: MutPtr<QTabWidget>,

    pub global_search_matches_db_tree_view: MutPtr<QTreeView>,
    pub global_search_matches_loc_tree_view: MutPtr<QTreeView>,
    pub global_search_matches_text_tree_view: MutPtr<QTreeView>,
    pub global_search_matches_schema_tree_view: MutPtr<QTreeView>,

    pub global_search_matches_db_tree_filter: MutPtr<QSortFilterProxyModel>,
    pub global_search_matches_loc_tree_filter: MutPtr<QSortFilterProxyModel>,
    pub global_search_matches_text_tree_filter: MutPtr<QSortFilterProxyModel>,
    pub global_search_matches_schema_tree_filter: MutPtr<QSortFilterProxyModel>,

    pub global_search_matches_db_tree_model: MutPtr<QStandardItemModel>,
    pub global_search_matches_loc_tree_model: MutPtr<QStandardItemModel>,
    pub global_search_matches_text_tree_model: MutPtr<QStandardItemModel>,
    pub global_search_matches_schema_tree_model: MutPtr<QStandardItemModel>,

    pub global_search_matches_filter_db_line_edit: MutPtr<QLineEdit>,
    pub global_search_matches_filter_loc_line_edit: MutPtr<QLineEdit>,
    pub global_search_matches_filter_text_line_edit: MutPtr<QLineEdit>,
    pub global_search_matches_filter_schema_line_edit: MutPtr<QLineEdit>,

    pub global_search_matches_case_sensitive_db_button: MutPtr<QPushButton>,
    pub global_search_matches_case_sensitive_loc_button: MutPtr<QPushButton>,
    pub global_search_matches_case_sensitive_text_button: MutPtr<QPushButton>,
    pub global_search_matches_case_sensitive_schema_button: MutPtr<QPushButton>,

    pub global_search_matches_column_selector_db_combobox: MutPtr<QComboBox>,
    pub global_search_matches_column_selector_loc_combobox: MutPtr<QComboBox>,
    pub global_search_matches_column_selector_text_combobox: MutPtr<QComboBox>,
    pub global_search_matches_column_selector_schema_combobox: MutPtr<QComboBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchUI`.
impl GlobalSearchUI {

    /// This function creates an entire `GlobalSearchUI` struct.
    pub unsafe fn new(mut main_window: MutPtr<QMainWindow>) -> Self {

        // Create and configure the 'Global Search` Dock Widget and all his contents.
        let mut global_search_dock_widget = QDockWidget::from_q_widget(main_window).into_ptr();
        let global_search_dock_inner_widget = QWidget::new_0a().into_ptr();
        let mut global_search_dock_layout = create_grid_layout(global_search_dock_inner_widget);
        global_search_dock_widget.set_widget(global_search_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::RightDockWidgetArea, global_search_dock_widget);
        global_search_dock_widget.set_window_title(&qtr("global_search"));

        // Create the search & replace section.
        let global_search_search_frame = QGroupBox::from_q_string(&qtr("global_search_info")).into_ptr();
        let mut global_search_search_grid = create_grid_layout(global_search_search_frame.static_upcast_mut());

        let mut global_search_search_line_edit = QLineEdit::new();
        let mut global_search_search_button = QPushButton::from_q_string(&qtr("global_search_search"));

        let mut global_search_replace_line_edit = QLineEdit::new();
        let mut global_search_replace_button = QPushButton::from_q_string(&qtr("global_search_replace"));
        let mut global_search_replace_all_button = QPushButton::from_q_string(&qtr("global_search_replace_all"));

        let mut global_search_clear_button = QPushButton::from_q_string(&qtr("global_search_clear"));
        let mut global_search_case_sensitive_checkbox = QCheckBox::from_q_string(&qtr("global_search_case_sensitive"));
        let mut global_search_use_regex_checkbox = QCheckBox::from_q_string(&qtr("global_search_use_regex"));

        let global_search_search_on_group_box = QGroupBox::from_q_string(&qtr("global_search_search_on")).into_ptr();
        let mut global_search_search_on_grid = create_grid_layout(global_search_search_on_group_box.static_upcast_mut());

        let mut global_search_search_on_all_checkbox = QCheckBox::from_q_string(&qtr("global_search_all"));
        let mut global_search_search_on_dbs_checkbox = QCheckBox::from_q_string(&qtr("global_search_db"));
        let mut global_search_search_on_locs_checkbox = QCheckBox::from_q_string(&qtr("global_search_loc"));
        let mut global_search_search_on_texts_checkbox = QCheckBox::from_q_string(&qtr("global_search_txt"));
        let mut global_search_search_on_schemas_checkbox = QCheckBox::from_q_string(&qtr("global_search_schemas"));

        global_search_search_grid.set_column_stretch(0, 10);

        // Add everything to the Matches's Dock Layout.
        global_search_search_grid.add_widget_5a(&mut global_search_search_line_edit, 0, 0, 1, 2);
        global_search_search_grid.add_widget_5a(&mut global_search_replace_line_edit, 1, 0, 1, 2);
        global_search_search_grid.add_widget_5a(&mut global_search_search_button, 0, 2, 1, 1);
        global_search_search_grid.add_widget_5a(&mut global_search_replace_button, 1, 2, 1, 1);
        global_search_search_grid.add_widget_5a(&mut global_search_replace_all_button, 1, 3, 1, 1);

        global_search_search_grid.add_widget_5a(&mut global_search_clear_button, 0, 3, 1, 1);
        global_search_search_grid.add_widget_5a(&mut global_search_case_sensitive_checkbox, 0, 4, 1, 1);
        global_search_search_grid.add_widget_5a(&mut global_search_use_regex_checkbox, 1, 4, 1, 1);
        global_search_search_grid.add_widget_5a(global_search_search_on_group_box, 2, 0, 1, 10);

        global_search_search_on_grid.add_widget_5a(&mut global_search_search_on_all_checkbox, 0, 0, 1, 1);
        global_search_search_on_grid.add_widget_5a(&mut global_search_search_on_dbs_checkbox, 0, 1, 1, 1);
        global_search_search_on_grid.add_widget_5a(&mut global_search_search_on_locs_checkbox, 0, 2, 1, 1);
        global_search_search_on_grid.add_widget_5a(&mut global_search_search_on_texts_checkbox, 0, 3, 1, 1);
        global_search_search_on_grid.add_widget_5a(&mut global_search_search_on_schemas_checkbox, 0, 4, 1, 1);

        // Create the frames for the matches tables.
        let mut global_search_matches_tab_widget = QTabWidget::new_0a();

        let mut db_matches_widget = QWidget::new_0a().into_ptr();
        let mut db_matches_grid = create_grid_layout(db_matches_widget);

        let mut loc_matches_widget = QWidget::new_0a().into_ptr();
        let mut loc_matches_grid = create_grid_layout(loc_matches_widget);

        let mut text_matches_widget = QWidget::new_0a().into_ptr();
        let mut text_matches_grid = create_grid_layout(text_matches_widget);

        let mut schema_matches_widget = QWidget::new_0a().into_ptr();
        let mut schema_matches_grid = create_grid_layout(schema_matches_widget);

        // `TreeView`s with all the matches.
        let mut tree_view_matches_db = QTreeView::new_0a();
        let mut tree_view_matches_loc = QTreeView::new_0a();
        let mut tree_view_matches_text = QTreeView::new_0a();
        let mut tree_view_matches_schema = QTreeView::new_0a();

        let mut filter_model_matches_db = new_treeview_filter_safe(&mut db_matches_widget);
        let mut filter_model_matches_loc = new_treeview_filter_safe(&mut loc_matches_widget);
        let mut filter_model_matches_text = new_treeview_filter_safe(&mut text_matches_widget);
        let mut filter_model_matches_schema = new_treeview_filter_safe(&mut schema_matches_widget);

        let mut model_matches_db = QStandardItemModel::new_0a();
        let mut model_matches_loc = QStandardItemModel::new_0a();
        let mut model_matches_text = QStandardItemModel::new_0a();
        let mut model_matches_schema = QStandardItemModel::new_0a();

        tree_view_matches_db.set_model(filter_model_matches_db);
        tree_view_matches_loc.set_model(filter_model_matches_loc);
        tree_view_matches_text.set_model(filter_model_matches_text);
        tree_view_matches_schema.set_model(filter_model_matches_schema);

        filter_model_matches_db.set_source_model(&mut model_matches_db);
        filter_model_matches_loc.set_source_model(&mut model_matches_loc);
        filter_model_matches_text.set_source_model(&mut model_matches_text);
        filter_model_matches_schema.set_source_model(&mut model_matches_schema);

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
        let mut filter_matches_db_line_edit = QLineEdit::new();
        let mut filter_matches_db_column_selector = QComboBox::new_0a();
        let mut filter_matches_db_column_list = QStandardItemModel::new_0a();
        let mut filter_matches_db_case_sensitive_button = QPushButton::from_q_string(&qtr("global_search_case_sensitive"));

        filter_matches_db_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_db_column_selector.set_model(&mut filter_matches_db_column_list);
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_db_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_db_case_sensitive_button.set_checkable(true);

        let mut filter_matches_loc_line_edit = QLineEdit::new();
        let mut filter_matches_loc_column_selector = QComboBox::new_0a();
        let mut filter_matches_loc_column_list = QStandardItemModel::new_0a();
        let mut filter_matches_loc_case_sensitive_button = QPushButton::from_q_string(&qtr("global_search_case_sensitive"));

        filter_matches_loc_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_loc_column_selector.set_model(&mut filter_matches_loc_column_list);
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_loc_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_loc_case_sensitive_button.set_checkable(true);

        let mut filter_matches_text_line_edit = QLineEdit::new();
        let mut filter_matches_text_column_selector = QComboBox::new_0a();
        let mut filter_matches_text_column_list = QStandardItemModel::new_0a();
        let mut filter_matches_text_case_sensitive_button = QPushButton::from_q_string(&qtr("global_search_case_sensitive"));

        filter_matches_text_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_text_column_selector.set_model(&mut filter_matches_text_column_list);
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_text_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_text_case_sensitive_button.set_checkable(true);

        let mut filter_matches_schema_line_edit = QLineEdit::new();
        let mut filter_matches_schema_column_selector = QComboBox::new_0a();
        let mut filter_matches_schema_column_list = QStandardItemModel::new_0a();
        let mut filter_matches_schema_case_sensitive_button = QPushButton::from_q_string(&qtr("global_search_case_sensitive"));

        filter_matches_schema_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_schema_column_selector.set_model(&mut filter_matches_schema_column_list);
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_schema_case_sensitive_button.set_checkable(true);

        // Add everything to the Matches's Dock Layout.
        db_matches_grid.add_widget_5a(&mut tree_view_matches_db, 0, 0, 1, 3);
        loc_matches_grid.add_widget_5a(&mut tree_view_matches_loc, 0, 0, 1, 3);
        text_matches_grid.add_widget_5a(&mut tree_view_matches_text, 0, 0, 1, 3);
        schema_matches_grid.add_widget_5a(&mut tree_view_matches_schema, 0, 0, 1, 3);

        db_matches_grid.add_widget_5a(&mut filter_matches_db_line_edit, 1, 0, 1, 1);
        db_matches_grid.add_widget_5a(&mut filter_matches_db_case_sensitive_button, 1, 1, 1, 1);
        db_matches_grid.add_widget_5a(&mut filter_matches_db_column_selector, 1, 2, 1, 1);

        loc_matches_grid.add_widget_5a(&mut filter_matches_loc_line_edit, 1, 0, 1, 1);
        loc_matches_grid.add_widget_5a(&mut filter_matches_loc_case_sensitive_button, 1, 1, 1, 1);
        loc_matches_grid.add_widget_5a(&mut filter_matches_loc_column_selector, 1, 2, 1, 1);

        text_matches_grid.add_widget_5a(&mut filter_matches_text_line_edit, 1, 0, 1, 1);
        text_matches_grid.add_widget_5a(&mut filter_matches_text_case_sensitive_button, 1, 1, 1, 1);
        text_matches_grid.add_widget_5a(&mut filter_matches_text_column_selector, 1, 2, 1, 1);

        schema_matches_grid.add_widget_5a(&mut filter_matches_schema_line_edit, 1, 0, 1, 1);
        schema_matches_grid.add_widget_5a(&mut filter_matches_schema_case_sensitive_button, 1, 1, 1, 1);
        schema_matches_grid.add_widget_5a(&mut filter_matches_schema_column_selector, 1, 2, 1, 1);

        global_search_matches_tab_widget.add_tab_2a(db_matches_widget, &qtr("global_search_db_matches"));
        global_search_matches_tab_widget.add_tab_2a(loc_matches_widget, &qtr("global_search_loc_matches"));
        global_search_matches_tab_widget.add_tab_2a(text_matches_widget, &qtr("global_search_txt_matches"));
        global_search_matches_tab_widget.add_tab_2a(schema_matches_widget, &qtr("global_search_schema_matches"));

        global_search_dock_layout.add_widget_5a(global_search_search_frame, 0, 0, 1, 3);
        global_search_dock_layout.add_widget_5a(&mut global_search_matches_tab_widget, 1, 0, 1, 3);

        // Hide this widget by default.
        global_search_dock_widget.hide();

        // Create ***Da monsta***.
        Self {
            global_search_dock_widget,
            global_search_search_line_edit: global_search_search_line_edit.into_ptr(),
            global_search_search_button: global_search_search_button.into_ptr(),

            global_search_replace_line_edit: global_search_replace_line_edit.into_ptr(),
            global_search_replace_button: global_search_replace_button.into_ptr(),
            global_search_replace_all_button: global_search_replace_all_button.into_ptr(),

            global_search_clear_button: global_search_clear_button.into_ptr(),
            global_search_case_sensitive_checkbox: global_search_case_sensitive_checkbox.into_ptr(),
            global_search_use_regex_checkbox: global_search_use_regex_checkbox.into_ptr(),

            global_search_search_on_all_checkbox: global_search_search_on_all_checkbox.into_ptr(),
            global_search_search_on_dbs_checkbox: global_search_search_on_dbs_checkbox.into_ptr(),
            global_search_search_on_locs_checkbox: global_search_search_on_locs_checkbox.into_ptr(),
            global_search_search_on_texts_checkbox: global_search_search_on_texts_checkbox.into_ptr(),
            global_search_search_on_schemas_checkbox: global_search_search_on_schemas_checkbox.into_ptr(),

            global_search_matches_tab_widget: global_search_matches_tab_widget.into_ptr(),

            global_search_matches_db_tree_view: tree_view_matches_db.into_ptr(),
            global_search_matches_loc_tree_view: tree_view_matches_loc.into_ptr(),
            global_search_matches_text_tree_view: tree_view_matches_text.into_ptr(),
            global_search_matches_schema_tree_view: tree_view_matches_schema.into_ptr(),

            global_search_matches_db_tree_filter: filter_model_matches_db,
            global_search_matches_loc_tree_filter: filter_model_matches_loc,
            global_search_matches_text_tree_filter: filter_model_matches_text,
            global_search_matches_schema_tree_filter: filter_model_matches_schema,

            global_search_matches_db_tree_model: model_matches_db.into_ptr(),
            global_search_matches_loc_tree_model: model_matches_loc.into_ptr(),
            global_search_matches_text_tree_model: model_matches_text.into_ptr(),
            global_search_matches_schema_tree_model: model_matches_schema.into_ptr(),

            global_search_matches_filter_db_line_edit: filter_matches_db_line_edit.into_ptr(),
            global_search_matches_filter_loc_line_edit: filter_matches_loc_line_edit.into_ptr(),
            global_search_matches_filter_text_line_edit: filter_matches_text_line_edit.into_ptr(),
            global_search_matches_filter_schema_line_edit: filter_matches_schema_line_edit.into_ptr(),

            global_search_matches_case_sensitive_db_button: filter_matches_db_case_sensitive_button.into_ptr(),
            global_search_matches_case_sensitive_loc_button: filter_matches_loc_case_sensitive_button.into_ptr(),
            global_search_matches_case_sensitive_text_button: filter_matches_text_case_sensitive_button.into_ptr(),
            global_search_matches_case_sensitive_schema_button: filter_matches_schema_case_sensitive_button.into_ptr(),

            global_search_matches_column_selector_db_combobox: filter_matches_db_column_selector.into_ptr(),
            global_search_matches_column_selector_loc_combobox: filter_matches_loc_column_selector.into_ptr(),
            global_search_matches_column_selector_text_combobox: filter_matches_text_column_selector.into_ptr(),
            global_search_matches_column_selector_schema_combobox: filter_matches_schema_column_selector.into_ptr(),
        }
    }

    /// This function is used to search the entire PackFile, using the data in Self for the search.
    pub unsafe fn search(&mut self, pack_file_contents_ui: &mut PackFileContentsUI) {

        // Create the global search and populate it with all the settings for the search.
        let mut global_search = GlobalSearch::default();
        global_search.pattern = self.global_search_search_line_edit.text().to_std_string();
        global_search.case_sensitive = self.global_search_case_sensitive_checkbox.is_checked();
        global_search.use_regex = self.global_search_use_regex_checkbox.is_checked();

        if self.global_search_search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = self.global_search_search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = self.global_search_search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = self.global_search_search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = self.global_search_search_on_schemas_checkbox.is_checked();
        }

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearch(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let mut tree_view_db = self.global_search_matches_db_tree_view;
        let mut tree_view_loc = self.global_search_matches_loc_tree_view;
        let mut tree_view_text = self.global_search_matches_text_tree_view;
        let mut tree_view_schema = self.global_search_matches_schema_tree_view;

        let mut model_db = self.global_search_matches_db_tree_model;
        let mut model_loc = self.global_search_matches_loc_tree_model;
        let mut model_text = self.global_search_matches_text_tree_model;
        let mut model_schema = self.global_search_matches_schema_tree_model;

        model_db.clear();
        model_loc.clear();
        model_text.clear();
        model_schema.clear();

        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {

                // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                Self::load_table_matches_to_ui(&mut model_db, &mut tree_view_db, &global_search.matches_db);
                Self::load_table_matches_to_ui(&mut model_loc, &mut tree_view_loc, &global_search.matches_loc);
                Self::load_text_matches_to_ui(&mut model_text, &mut tree_view_text, &global_search.matches_text);
                Self::load_schema_matches_to_ui(&mut model_schema, &mut tree_view_schema, &global_search.matches_schema);
                UI_STATE.set_global_search(&global_search);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info));
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This function takes care of updating the results of a global search for the provided paths.
    ///
    /// NOTE: This only works in the `editable` search results, which are DB Tables, Locs and Text PackedFiles.
    pub unsafe fn search_on_path(&mut self, pack_file_contents_ui: &mut PackFileContentsUI, paths: Vec<PathType>) {

        // Create the global search and populate it with all the settings for the search.
        let global_search = UI_STATE.get_global_search();

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearchUpdate(global_search, paths));

        // While we wait for an answer, we need to clear the current results panels.
        let mut tree_view_db = self.global_search_matches_db_tree_view;
        let mut tree_view_loc = self.global_search_matches_loc_tree_view;
        let mut tree_view_text = self.global_search_matches_text_tree_view;

        let mut model_db = self.global_search_matches_db_tree_model;
        let mut model_loc = self.global_search_matches_loc_tree_model;
        let mut model_text = self.global_search_matches_text_tree_model;

        model_db.clear();
        model_loc.clear();
        model_text.clear();

        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {

                // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                Self::load_table_matches_to_ui(&mut model_db, &mut tree_view_db, &global_search.matches_db);
                Self::load_table_matches_to_ui(&mut model_loc, &mut tree_view_loc, &global_search.matches_loc);
                Self::load_text_matches_to_ui(&mut model_text, &mut tree_view_text, &global_search.matches_text);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info));
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub unsafe fn clear(&mut self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        self.global_search_matches_db_tree_model.clear();
        self.global_search_matches_loc_tree_model.clear();
        self.global_search_matches_text_tree_model.clear();
        self.global_search_matches_schema_tree_model.clear();
    }

    /// This function replace the currently selected match with the provided text.
    pub unsafe fn replace(&mut self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        self.global_search_matches_db_tree_model.clear();
        self.global_search_matches_loc_tree_model.clear();
        self.global_search_matches_text_tree_model.clear();
        self.global_search_matches_schema_tree_model.clear();
    }

    /// This function replace all the matches in the current search with the provided text.
    pub unsafe fn replace_all(&mut self, app_ui: &mut AppUI, pack_file_contents_ui: &mut PackFileContentsUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) {

        // To avoid conflicting data, we close all PackedFiles hard and re-search before replacing.
        app_ui.purge_them_all(*self, *pack_file_contents_ui, slot_holder);
        self.search(pack_file_contents_ui);

        let mut global_search = UI_STATE.get_global_search();
        global_search.pattern = self.global_search_search_line_edit.text().to_std_string();
        global_search.replace_text = self.global_search_replace_line_edit.text().to_std_string();
        global_search.case_sensitive = self.global_search_case_sensitive_checkbox.is_checked();
        global_search.use_regex = self.global_search_use_regex_checkbox.is_checked();

        if self.global_search_search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = self.global_search_search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = self.global_search_search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = self.global_search_search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = self.global_search_search_on_schemas_checkbox.is_checked();
        }

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearchReplaceAll(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let mut model_db = self.global_search_matches_db_tree_model;
        let mut model_loc = self.global_search_matches_loc_tree_model;
        let mut model_text = self.global_search_matches_text_tree_model;

        model_db.clear();
        model_loc.clear();
        model_text.clear();

        match CENTRAL_COMMAND.recv_message_qt() {
            Response::GlobalSearchVecPackedFileInfo((global_search, packed_files_info)) => {
                UI_STATE.set_global_search(&global_search);
                self.search(pack_file_contents_ui);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info));
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
        mut app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let mut tree_view = pack_file_contents_ui.packfile_contents_tree_view;
        let filter_model: Ptr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: MutPtr<QStandardItemModel> = filter_model.source_model().static_downcast_mut();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        let gidhora = model.item_from_index(&model_index);
        let is_match = !gidhora.has_children();

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        if is_match {
            let parent = gidhora.parent();
            let path = parent.text().to_std_string();
            let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

            if let Some(pack_file_contents_model_index) = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path) {
                let mut selection_model = tree_view.selection_model();

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if pack_file_contents_model_index.is_valid() {
                    selection_model.select_q_model_index_q_flags_selection_flag(pack_file_contents_model_index, QFlags::from(SelectionFlag::ClearAndSelect));
                    tree_view.scroll_to_1a(pack_file_contents_model_index);
                    app_ui.open_packedfile(&mut pack_file_contents_ui, &global_search_ui, &slot_holder, false);

                    if let Some((_, packed_file_view)) = UI_STATE.get_open_packedfiles().iter().find(|x| x.0 == &path) {
                        match packed_file_view.get_view() {

                            // In case of tables, we have to get the logical row/column of the match and select it.
                            View::Table(view) => {
                                let mut table_view = view.get_mut_ptr_table_view_primary();
                                let table_filter: MutPtr<QSortFilterProxyModel> = table_view.model().static_downcast_mut();
                                let table_model: MutPtr<QStandardItemModel> = table_filter.source_model().static_downcast_mut();
                                let mut table_selection_model = table_view.selection_model();

                                let row = parent.child_2a(model_index.row(), 1).text().to_std_string().parse::<i32>().unwrap() - 1;
                                let column = parent.child_2a(model_index.row(), 3).text().to_std_string().parse::<i32>().unwrap();

                                let table_model_index = table_model.index_2a(row, column);
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                                    table_view.scroll_to_1a(table_model_index_filtered.as_ref());
                                }
                            },

                            _ => {},
                        }
                    }
                }
            }
        }

        // If not... just expand and open the PackedFile.
        else {
            let path = gidhora.text().to_std_string();
            let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

            if let Some(model_index) = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path) {
                let mut selection_model = tree_view.selection_model();

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if model_index.is_valid() {
                    selection_model.select_q_model_index_q_flags_selection_flag(model_index, QFlags::from(SelectionFlag::ClearAndSelect));
                    tree_view.scroll_to_1a(model_index);
                    app_ui.open_packedfile(&mut pack_file_contents_ui, &global_search_ui, &slot_holder, false);
                }
            }
            else { show_dialog(app_ui.main_window, ErrorKind::PackedFileNotInFilter, false); }
        }
    }


    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    unsafe fn load_table_matches_to_ui(model: &mut QStandardItemModel, tree_view: &mut QTreeView, matches: &[TableMatches]) {
        if !matches.is_empty() {

            for match_table in matches {
                if !match_table.matches.is_empty() {
                    let path = match_table.path.join("/");
                    let qlist_daddy = QListOfQStandardItem::new();
                    let mut file = QStandardItem::new();
                    let mut fill1 = QStandardItem::new();
                    let mut fill2 = QStandardItem::new();
                    let mut fill3 = QStandardItem::new();
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_table.matches {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let mut column_name = QStandardItem::new();
                        let mut column_number = QStandardItem::new();
                        let mut row = QStandardItem::new();
                        let mut text = QStandardItem::new();

                        column_name.set_text(&QString::from_std_str(&match_row.column_name));
                        column_number.set_data_2a(&QVariant::from_uint(match_row.column_number), 2);
                        row.set_data_2a(&QVariant::from_i64(match_row.row_number + 1), 2);
                        text.set_text(&QString::from_std_str(&match_row.contents));

                        column_name.set_editable(false);
                        column_number.set_editable(false);
                        row.set_editable(false);
                        text.set_editable(false);

                        // Add an empty row to the list.
                        //unsafe { qlist_boi.append_unsafe(&column_name); }
                        //unsafe { qlist_boi.append_unsafe(&row); }
                        //unsafe { qlist_boi.append_unsafe(&text); }
                        //unsafe { qlist_boi.append_unsafe(&column_number); }

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(&qlist_boi);
                    }
                    //unsafe { qlist_daddy.append_unsafe(&file); }
                    //unsafe { qlist_daddy.append_unsafe(&fill1); }
                    //unsafe { qlist_daddy.append_unsafe(&fill2); }
                    //unsafe { qlist_daddy.append_unsafe(&fill3); }
                    model.append_row_q_list_of_q_standard_item(&qlist_daddy);
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
    unsafe fn load_text_matches_to_ui(model: &mut QStandardItemModel, tree_view: &mut QTreeView, matches: &[TextMatches]) {
        if !matches.is_empty() {
            for match_text in matches {
                if !match_text.matches.is_empty() {
                    let path = match_text.path.join("/");
                    let qlist_daddy = QListOfQStandardItem::new();
                    let mut file = QStandardItem::new();
                    let mut fill1 = QStandardItem::new();
                    let mut fill2 = QStandardItem::new();
                    let mut fill3 = QStandardItem::new();
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_text.matches {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let mut text = QStandardItem::new();
                        let mut row = QStandardItem::new();
                        let mut column = QStandardItem::new();
                        let mut len = QStandardItem::new();

                        text.set_text(&QString::from_std_str(&match_row.text));
                        row.set_data_2a(&QVariant::from_u64(match_row.row + 1), 2);
                        column.set_data_2a(&QVariant::from_u64(match_row.column), 2);
                        len.set_data_2a(&QVariant::from_i64(match_row.len), 2);

                        text.set_editable(false);
                        row.set_editable(false);
                        column.set_editable(false);
                        len.set_editable(false);

                        // Add an empty row to the list.
                        //unsafe { qlist_boi.append_unsafe(&text); }
                        //unsafe { qlist_boi.append_unsafe(&row); }
                        //unsafe { qlist_boi.append_unsafe(&column); }
                        //unsafe { qlist_boi.append_unsafe(&len); }

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(&qlist_boi);
                    }
                    //unsafe { qlist_daddy.append_unsafe(&file); }
                    //unsafe { qlist_daddy.append_unsafe(&fill1); }
                    //unsafe { qlist_daddy.append_unsafe(&fill2); }
                    //unsafe { qlist_daddy.append_unsafe(&fill3); }
                    model.append_row_q_list_of_q_standard_item(&qlist_daddy);
                }
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_match_packedfile_text")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_row")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_column")));
            model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_length")));

            // Hide the column and lenght numbers on the TreeView.
            tree_view.hide_column(2);
            tree_view.hide_column(3);
            tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);

            tree_view.header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    unsafe fn load_schema_matches_to_ui(model: &mut QStandardItemModel, tree_view: &mut QTreeView, matches: &[SchemaMatches]) {
        if !matches.is_empty() {

            for match_schema in matches {
                if !match_schema.matches.is_empty() {
                    let qlist_daddy = QListOfQStandardItem::new();
                    let mut versioned_file = QStandardItem::new();
                    let mut fill1 = QStandardItem::new();
                    let mut fill2 = QStandardItem::new();

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
                        let mut name = QStandardItem::new();
                        let mut version = QStandardItem::new();
                        let mut column = QStandardItem::new();

                        name.set_text(&QString::from_std_str(&match_row.name));
                        version.set_data_2a(&QVariant::from_int(match_row.version), 2);
                        column.set_data_2a(&QVariant::from_uint(match_row.column), 2);

                        name.set_editable(false);
                        version.set_editable(false);
                        column.set_editable(false);

                        // Add an empty row to the list.
                        //unsafe { qlist_boi.append_unsafe(&name); }
                        //unsafe { qlist_boi.append_unsafe(&version); }
                        //unsafe { qlist_boi.append_unsafe(&column); }

                        // Append the new row.
                        versioned_file.append_row_q_list_of_q_standard_item(&qlist_boi);
                    }
                    //unsafe { qlist_daddy.append_unsafe(&versioned_file); }
                    //unsafe { qlist_daddy.append_unsafe(&fill1); }
                    //unsafe { qlist_daddy.append_unsafe(&fill2); }
                    model.append_row_q_list_of_q_standard_item(&qlist_daddy);
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
        view: MutPtr<QTreeView>,
        line_edit: MutPtr<QLineEdit>,
        column_combobox: MutPtr<QComboBox>,
        case_sensitive_button: MutPtr<QPushButton>,
    ) {

        let mut pattern = QRegExp::new_1a(&line_edit.text());

        let case_sensitive = case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        let mut model_filter: MutPtr<QSortFilterProxyModel> = view.model().static_downcast_mut();
        model_filter.set_filter_key_column(column_combobox.current_index());
        trigger_treeview_filter_safe(&mut model_filter, &mut pattern);
    }
}
