//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDockWidget;
use qt_widgets::QGroupBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QRadioButton;
use qt_widgets::QTabWidget;
use qt_widgets::QToolButton;
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
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::Ptr;

use anyhow::Result;
use getset::Getters;

use std::rc::Rc;

use rpfm_extensions::search::{GlobalSearch, MatchHolder, SearchSource, schema::SchemaMatches, table::{TableMatches, TableMatch}, text::{TextMatches, TextMatch}};
use rpfm_lib::files::FileType;
use rpfm_ui_common::locale::qtr;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::{kline_edit_configure_safe, new_treeview_filter_safe, scroll_to_row_safe, trigger_treeview_filter_safe};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::references_ui::ReferencesUI;
use crate::TREEVIEW_ICONS;
use crate::utils::*;
use crate::UI_STATE;

pub mod connections;
pub mod slots;
pub mod tips;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/global_search_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/global_search_dock_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Global Search panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct GlobalSearchUI {
    dock_widget: QPtr<QDockWidget>,

    search_line_edit: QPtr<QLineEdit>,
    search_button: QPtr<QToolButton>,
    clear_button: QPtr<QToolButton>,
    case_sensitive_checkbox: QPtr<QToolButton>,

    replace_line_edit: QPtr<QLineEdit>,
    replace_button: QPtr<QToolButton>,
    replace_all_button: QPtr<QToolButton>,
    use_regex_checkbox: QPtr<QToolButton>,

    search_source_packfile: QPtr<QRadioButton>,
    search_source_parent: QPtr<QRadioButton>,
    search_source_game: QPtr<QRadioButton>,
    search_source_asskit: QPtr<QRadioButton>,

    search_on_all_checkbox: QPtr<QCheckBox>,
    search_on_dbs_checkbox: QPtr<QCheckBox>,
    search_on_locs_checkbox: QPtr<QCheckBox>,
    search_on_texts_checkbox: QPtr<QCheckBox>,
    search_on_schemas_checkbox: QPtr<QCheckBox>,

    matches_tab_widget: QPtr<QTabWidget>,

    matches_table_and_text_tree_view: QPtr<QTreeView>,
    matches_schema_tree_view: QPtr<QTreeView>,

    matches_table_and_text_tree_model: QBox<QStandardItemModel>,
    matches_schema_tree_model: QBox<QStandardItemModel>,

    matches_filter_table_and_text_line_edit: QPtr<QLineEdit>,
    matches_filter_schema_line_edit: QPtr<QLineEdit>,

    matches_case_sensitive_table_and_text_button: QPtr<QToolButton>,
    matches_case_sensitive_schema_button: QPtr<QToolButton>,

    matches_column_selector_table_and_text_combobox: QPtr<QComboBox>,
    matches_column_selector_schema_combobox: QPtr<QComboBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchUI`.
impl GlobalSearchUI {

    /// This function creates an entire `GlobalSearchUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();

        // Create and configure the 'Global Search` Dock Widget and all his contents.
        main_window.add_dock_widget_2a(DockWidgetArea::RightDockWidgetArea, &dock_widget);
        dock_widget.set_window_title(&qtr("global_search"));
        dock_widget.set_object_name(&QString::from_std_str("global_search_dock"));

        // Create the search & replace section.
        let search_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "search_line_edit")?;
        let search_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "search_button")?;
        let clear_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "clear_button")?;
        let case_sensitive_checkbox: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "case_sensitive_search_button")?;
        search_line_edit.set_placeholder_text(&qtr("global_search_search_placeholder"));
        search_button.set_tool_tip(&qtr("global_search_search"));
        clear_button.set_tool_tip(&qtr("global_search_clear"));
        case_sensitive_checkbox.set_tool_tip(&qtr("global_search_case_sensitive"));
        kline_edit_configure_safe(&search_line_edit.static_upcast::<QWidget>().as_ptr());

        let replace_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "replace_line_edit")?;
        let replace_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_button")?;
        let replace_all_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_all_button")?;
        let use_regex_checkbox: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "regex_button")?;
        replace_line_edit.set_placeholder_text(&qtr("global_search_replace_placeholder"));
        replace_button.set_tool_tip(&qtr("global_search_replace"));
        replace_all_button.set_tool_tip(&qtr("global_search_replace_all"));
        use_regex_checkbox.set_tool_tip(&qtr("global_search_use_regex"));
        kline_edit_configure_safe(&replace_line_edit.static_upcast::<QWidget>().as_ptr());

        let search_on_group_box: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "search_on_groupbox")?;
        search_on_group_box.set_title(&qtr("global_search_search_on"));

        let search_source_packfile: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_packfile")?;
        let search_source_parent: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_parent")?;
        let search_source_game: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_game")?;
        let search_source_asskit: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_asskit")?;
        search_source_packfile.set_text(&qtr("global_search_source_packfile"));
        search_source_parent.set_text(&qtr("global_search_source_parent"));
        search_source_game.set_text(&qtr("global_search_source_game"));
        search_source_asskit.set_text(&qtr("global_search_source_asskit"));

        let search_source_group_box: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "search_source_groupbox")?;
        search_source_group_box.set_title(&qtr("global_search_search_source"));

        let search_on_all_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_all")?;
        let search_on_dbs_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_db")?;
        let search_on_locs_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_loc")?;
        let search_on_texts_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_txt")?;
        let search_on_schemas_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_schemas")?;
        search_on_all_checkbox.set_text(&qtr("global_search_all"));
        search_on_dbs_checkbox.set_text(&qtr("global_search_db"));
        search_on_locs_checkbox.set_text(&qtr("global_search_loc"));
        search_on_texts_checkbox.set_text(&qtr("global_search_txt"));
        search_on_schemas_checkbox.set_text(&qtr("global_search_schemas"));

        // Create the frames for the matches tables.
        let matches_tab_widget: QPtr<QTabWidget> = find_widget(&main_widget.static_upcast(), "results_tab_widget")?;
        matches_tab_widget.tab_bar().set_expanding(true);

        // Tables and texts.
        let matches_widget_table_and_text: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "tab_table_and_text")?;
        let tree_view_matches_table_and_text: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "table_and_text_tree_view")?;
        let filter_matches_table_and_text_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "table_and_text_filter_line_edit")?;
        let filter_matches_table_and_text_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "table_and_text_filter_case_sensitive_button")?;
        let filter_matches_table_and_text_column_selector: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "table_and_text_column_combo_box")?;
        let filter_matches_table_and_text_column_list = QStandardItemModel::new_1a(&matches_widget_table_and_text);
        filter_matches_table_and_text_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_table_and_text_column_selector.set_model(&filter_matches_table_and_text_column_list);
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_table_and_text_case_sensitive_button.set_tool_tip(&qtr("global_search_case_sensitive"));

        let matches_table_and_text_tree_filter = new_treeview_filter_safe(tree_view_matches_table_and_text.static_upcast());
        let matches_table_and_text_tree_model = QStandardItemModel::new_1a(&tree_view_matches_table_and_text);
        tree_view_matches_table_and_text.set_model(&matches_table_and_text_tree_filter);
        matches_table_and_text_tree_filter.set_source_model(&matches_table_and_text_tree_model);

        // Schema
        let matches_widget_schema: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "tab_schema")?;
        let tree_view_matches_schema: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "schema_tree_view")?;
        let filter_matches_schema_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "schema_filter_line_edit")?;
        let filter_matches_schema_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "schema_filter_case_sensitive_button")?;
        let filter_matches_schema_column_selector: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "schema_column_combo_box")?;
        let filter_matches_schema_column_list = QStandardItemModel::new_1a(&matches_widget_schema);
        filter_matches_schema_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_schema_column_selector.set_model(&filter_matches_schema_column_list);
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_table_name"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_version"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_column_name"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_column"));
        filter_matches_schema_case_sensitive_button.set_tool_tip(&qtr("global_search_case_sensitive"));

        let matches_schema_tree_filter = new_treeview_filter_safe(tree_view_matches_schema.static_upcast());
        let matches_schema_tree_model = QStandardItemModel::new_1a(&tree_view_matches_schema);
        tree_view_matches_schema.set_model(&matches_schema_tree_filter);
        matches_schema_tree_filter.set_source_model(&matches_schema_tree_model);

        matches_tab_widget.set_tab_text(0, &qtr("global_search_file_matches"));
        matches_tab_widget.set_tab_text(1, &qtr("global_search_schema_matches"));

        // Hide this widget by default.
        dock_widget.hide();

        // Create ***Da monsta***.
        Ok(Self {
            dock_widget,
            search_line_edit,
            search_button,

            replace_line_edit,
            replace_button,
            replace_all_button,

            clear_button,
            case_sensitive_checkbox,
            use_regex_checkbox,

            search_source_packfile,
            search_source_parent,
            search_source_game,
            search_source_asskit,

            search_on_all_checkbox,
            search_on_dbs_checkbox,
            search_on_locs_checkbox,
            search_on_texts_checkbox,
            search_on_schemas_checkbox,

            matches_tab_widget,

            matches_table_and_text_tree_view: tree_view_matches_table_and_text,
            matches_schema_tree_view: tree_view_matches_schema,

            matches_table_and_text_tree_model,
            matches_schema_tree_model,

            matches_filter_table_and_text_line_edit: filter_matches_table_and_text_line_edit,
            matches_filter_schema_line_edit: filter_matches_schema_line_edit,

            matches_case_sensitive_table_and_text_button: filter_matches_table_and_text_case_sensitive_button,
            matches_case_sensitive_schema_button: filter_matches_schema_case_sensitive_button,

            matches_column_selector_table_and_text_combobox: filter_matches_table_and_text_column_selector,
            matches_column_selector_schema_combobox: filter_matches_schema_column_selector,
        })
    }

    /// This function is used to search the entire PackFile, using the data in Self for the search.
    pub unsafe fn search(&self, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Create the global search and populate it with all the settings for the search.
        let mut global_search = GlobalSearch {
            pattern: self.search_line_edit.text().to_std_string(),
            case_sensitive: self.case_sensitive_checkbox.is_checked(),
            use_regex: self.use_regex_checkbox.is_checked(),
            ..Default::default()
        };

        // If we don't have text to search, return.
        if global_search.pattern.is_empty() { return; }

        if self.search_source_packfile.is_checked() {
            global_search.source = SearchSource::Pack;
        } else if self.search_source_parent.is_checked() {
            global_search.source = SearchSource::ParentFiles;
        } else if self.search_source_game.is_checked() {
            global_search.source = SearchSource::GameFiles;
        } else if self.search_source_asskit.is_checked() {
            global_search.source = SearchSource::AssKitFiles;
        }

        if self.search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = self.search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = self.search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = self.search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = self.search_on_schemas_checkbox.is_checked();
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearch(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        self.matches_table_and_text_tree_model.clear();
        self.matches_schema_tree_model.clear();

        // Load the results to their respective models. Then, store the GlobalSearch for future checks.
        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {
                self.load_table_matches_to_ui(&global_search.matches_db, FileType::DB);
                self.load_table_matches_to_ui(&global_search.matches_loc, FileType::Loc);
                self.load_text_matches_to_ui(&global_search.matches_text, FileType::Text);
                self.load_schema_matches_to_ui(&global_search.matches_schema);

                // Tweak the table columns for the files tree here, instead on each load function.
                self.matches_table_and_text_tree_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_match_packedfile_column")));
                self.matches_table_and_text_tree_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column_name")));
                self.matches_table_and_text_tree_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_row")));
                self.matches_table_and_text_tree_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_column")));
                self.matches_table_and_text_tree_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_length")));

                // Hide the column number column for tables.
                self.matches_table_and_text_tree_view.hide_column(3);
                self.matches_table_and_text_tree_view.hide_column(4);
                self.matches_table_and_text_tree_view.hide_column(5);
                self.matches_table_and_text_tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);
                self.matches_table_and_text_tree_view.header().resize_sections(ResizeMode::ResizeToContents);

                // Focus on the tree with the results.
                if !global_search.matches_db.is_empty() || !global_search.matches_loc.is_empty() || !global_search.matches_text.is_empty() {
                    self.matches_tab_widget().set_current_index(0);
                }

                else if !global_search.matches_schema.matches().is_empty() {
                    self.matches_tab_widget().set_current_index(1);
                }

                UI_STATE.set_global_search(&global_search);
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(&self.dock_widget, error, false),
            _ => unimplemented!()
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub unsafe fn clear(&self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        self.matches_table_and_text_tree_model.clear();
        self.matches_schema_tree_model.clear();
    }

    /// This function replace the currently selected match with the provided text.
    pub unsafe fn replace_current(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        let mut global_search = UI_STATE.get_global_search();

        if global_search.source != SearchSource::Pack {
            return show_dialog(app_ui.main_window(), "The dependencies are read-only. You cannot do a Global Replace over them.", false);
        }

        global_search.pattern = self.search_line_edit.text().to_std_string();
        global_search.replace_text = self.replace_line_edit.text().to_std_string();
        global_search.case_sensitive = self.case_sensitive_checkbox.is_checked();
        global_search.use_regex = self.use_regex_checkbox.is_checked();

        if self.search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = self.search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = self.search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = self.search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = self.search_on_schemas_checkbox.is_checked();
        }

        let matches = self.matches_from_selection();
        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceMatches(global_search, matches.to_vec()));

        // Before rebuilding the tree, check what items are expanded, to re-expand them later.
        let filter_model: QPtr<QSortFilterProxyModel> = self.matches_table_and_text_tree_view.model().static_downcast();
        let root = self.matches_table_and_text_tree_model.invisible_root_item();
        let mut expanded = vec![];

        for index in 0..root.row_count() {
            let source_index = root.child_1a(index).index();
            let view_index = filter_model.map_from_source(&source_index);
            if view_index.is_valid() && self.matches_table_and_text_tree_view.is_expanded(&view_index) {
                expanded.push(self.matches_table_and_text_tree_model.item_1a(index).text());
            }
        }

        // While we wait for an answer, we need to clear the current results panels.
        self.matches_table_and_text_tree_model.clear();

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {
                UI_STATE.set_global_search(&global_search);
                self.search(pack_file_contents_ui);

                // Update the views of the updated PackedFiles.
                for path in packed_files_info.iter().map(|x| x.path()) {
                    if let Some(file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| &*x.path_read() == path && x.data_source() == DataSource::PackFile) {
                        if let Err(error) = file_view.reload(path, pack_file_contents_ui) {
                            show_dialog(app_ui.main_window(), error, false);
                        }
                    }
                }

                // Re-expand the previously expanded items. We disable animation to avoid the slow opening behaviour of the UI.
                self.matches_table_and_text_tree_view.set_animated(false);

                let root = self.matches_table_and_text_tree_model.invisible_root_item();
                for index in 0..root.row_count() {
                    let source_item = root.child_1a(index);

                    if expanded.iter().any(|old| source_item.text().compare_q_string(old) == 0) {
                        let source_index = source_item.index();
                        let view_index = filter_model.map_from_source(&source_index);
                        if view_index.is_valid() && !self.matches_table_and_text_tree_view.is_expanded(&view_index) {
                            self.matches_table_and_text_tree_view.expand(&view_index)
                        }
                    }
                }

                self.matches_table_and_text_tree_view.set_animated(true);

                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => unimplemented!()
        }
    }

    /// This function replace all the matches in the current search with the provided text.
    pub unsafe fn replace_all(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // To avoid conflicting data, we close all PackedFiles hard and re-search before replacing.
        if let Err(error) = AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui) {
            return show_dialog(app_ui.main_window(), error, false);
        }

        self.search(pack_file_contents_ui);

        let mut global_search = UI_STATE.get_global_search();

        if global_search.source != SearchSource::Pack {
            return show_dialog(app_ui.main_window(), "The dependencies are read-only. You cannot do a Global Replace over them.", false);
        }

        global_search.pattern = self.search_line_edit.text().to_std_string();
        global_search.replace_text = self.replace_line_edit.text().to_std_string();
        global_search.case_sensitive = self.case_sensitive_checkbox.is_checked();
        global_search.use_regex = self.use_regex_checkbox.is_checked();

        if self.search_on_all_checkbox.is_checked() {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = self.search_on_dbs_checkbox.is_checked();
            global_search.search_on_locs = self.search_on_locs_checkbox.is_checked();
            global_search.search_on_texts = self.search_on_texts_checkbox.is_checked();
            global_search.search_on_schema = self.search_on_schemas_checkbox.is_checked();
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceAll(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        self.matches_table_and_text_tree_model.clear();

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {
                UI_STATE.set_global_search(&global_search);
                self.search(pack_file_contents_ui);

                for path in packed_files_info.iter().map(|x| x.path()) {
                    if let Some(file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| &*x.path_read() == path && x.data_source() == DataSource::PackFile) {
                        if let Err(error) = file_view.reload(path, pack_file_contents_ui) {
                            show_dialog(app_ui.main_window(), error, false);
                        }
                    }
                }

                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
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
        let path: String = if is_match {
            let parent = gidhora.parent();

            // Sometimes this is null, not sure why.
            if parent.is_null() { return; }
            parent.text().to_std_string()
        }

        // If not... just expand and open the PackedFile.
        else {
            gidhora.text().to_std_string()
        };

        let global_search = UI_STATE.get_global_search();
        let data_source = match global_search.source {
            SearchSource::Pack => {
                let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&path, DataSource::PackFile);

                // Manually select the open PackedFile, then open it. This means we can open PackedFiles nor in out filter.
                UI_STATE.set_packfile_contents_read_only(true);

                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }

                UI_STATE.set_packfile_contents_read_only(false);
                DataSource::PackFile
            },

            SearchSource::ParentFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::ParentFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::ParentFiles
            },
            SearchSource::GameFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::GameFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::GameFiles
            },
            SearchSource::AssKitFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::AssKitFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::AssKitFiles
            },
        };

        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_owned()), false, false, data_source);

        // If it's a table, focus on the matched cell.
        if is_match {
            if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == data_source).find(|x| *x.path_read() == path) {

                // In case of tables, we have to get the logical row/column of the match and select it.
                if let ViewType::Internal(View::Table(view)) = file_view.view_type() {
                    let parent = gidhora.parent();
                    let table_view = view.get_ref_table();
                    let table_view = table_view.table_view_ptr();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let table_selection_model = table_view.selection_model();

                    let row = parent.child_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap() - 1;
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

                // If it's a text file, scroll to the row in question.
                else if let ViewType::Internal(View::Text(view)) = file_view.view_type() {
                    let parent = gidhora.parent();
                    let row_number = parent.child_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap() - 1;
                    let editor = view.get_mut_editor();
                    scroll_to_row_safe(&editor.as_ptr(), row_number.try_into().unwrap());
                }
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    unsafe fn load_table_matches_to_ui(&self, matches: &[TableMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            if !matches.is_empty() {
                model.block_signals(true);
            }

            let file_type_item = QStandardItem::new();
            file_type_item.set_editable(false);
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));

            for (index, match_table) in matches.iter().enumerate() {
                if !match_table.matches().is_empty() {
                    let path = match_table.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = QStandardItem::new();
                    let fill1 = QStandardItem::new();
                    let fill2 = QStandardItem::new();
                    let fill3 = QStandardItem::new();
                    let fill4 = QStandardItem::new();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);
                    fill4.set_editable(false);

                    for match_row in match_table.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = QStandardItem::new();
                        let column_name = QStandardItem::new();
                        let row = QStandardItem::new();
                        let column_number = QStandardItem::new();
                        let fill5 = QStandardItem::new();

                        text.set_text(&QString::from_std_str(match_row.contents().trim()));
                        column_name.set_text(&QString::from_std_str(match_row.column_name()));
                        row.set_data_2a(&QVariant::from_i64(match_row.row_number() + 1), 2);
                        column_number.set_data_2a(&QVariant::from_uint(*match_row.column_number()), 2);

                        text.set_editable(false);
                        column_name.set_editable(false);
                        row.set_editable(false);
                        column_number.set_editable(false);
                        fill5.set_editable(false);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_number.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&fill5.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill3.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&fill4.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&file_type_item.clone().as_mut_raw_ptr());

                    // Unlock the model before the last insertion.
                    if index == matches.len() - 1 {
                        model.block_signals(false);
                    }

                    model.append_row_q_list_of_q_standard_item(qlist_daddy.as_ref());
                }
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TextMatches` into a model.
    unsafe fn load_text_matches_to_ui(&self, matches: &[TextMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            if !matches.is_empty() {
                model.block_signals(true);
            }

            let file_type_item = QStandardItem::new();
            file_type_item.set_editable(false);
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));

            for (index, match_text) in matches.iter().enumerate() {
                if !match_text.matches().is_empty() {
                    let path = match_text.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = QStandardItem::new();
                    let fill1 = QStandardItem::new();
                    let fill2 = QStandardItem::new();
                    let fill3 = QStandardItem::new();
                    let fill4 = QStandardItem::new();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);
                    fill4.set_editable(false);

                    for match_row in match_text.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Long rows take forever to load. Instead, we truncate them around the match.
                        let text_value = if match_row.text().chars().count() > 100 {
                            QString::from_std_str((match_row.text()[..match_row.text().char_indices().map(|(i, _)| i).nth(100).unwrap()].to_owned() + "...").trim())
                        } else {
                            QString::from_std_str(match_row.text().trim())
                        };

                        // Create an empty row.
                        let text = QStandardItem::from_q_string(&text_value);
                        let fill5 = QStandardItem::new();
                        let row = QStandardItem::new();
                        let column = QStandardItem::new();
                        let len = QStandardItem::new();

                        row.set_data_2a(&QVariant::from_u64(match_row.row() + 1), 2);
                        column.set_data_2a(&QVariant::from_u64(*match_row.column()), 2);
                        len.set_data_2a(&QVariant::from_i64(*match_row.len()), 2);

                        text.set_editable(false);
                        fill5.set_editable(false);
                        row.set_editable(false);
                        column.set_editable(false);
                        len.set_editable(false);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&fill5.into_ptr().as_mut_raw_ptr());
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
                    qlist_daddy.append_q_standard_item(&fill4.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&file_type_item.clone().as_mut_raw_ptr());

                    // Unlock the model before the last insertion.
                    if index == matches.len() - 1 {
                        model.block_signals(false);
                    }

                    model.append_row_q_list_of_q_standard_item(qlist_daddy.as_ref());
                }
            }
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    unsafe fn load_schema_matches_to_ui(&self, matches: &SchemaMatches) {
        let model = &self.matches_schema_tree_model;
        let tree_view = &self.matches_schema_tree_view;

        if !matches.matches().is_empty() {
            for match_schema in matches.matches() {
                let qlist = QListOfQStandardItem::new();
                let table_name = QStandardItem::new();
                let version = QStandardItem::new();
                let column_name = QStandardItem::new();
                let column = QStandardItem::new();

                table_name.set_text(&QString::from_std_str(match_schema.table_name()));
                version.set_data_2a(&QVariant::from_int(*match_schema.version()), 2);
                column_name.set_text(&QString::from_std_str(match_schema.column_name()));
                column.set_data_2a(&QVariant::from_uint(*match_schema.column()), 2);

                table_name.set_editable(false);
                version.set_editable(false);
                column_name.set_editable(false);
                column.set_editable(false);

                qlist.append_q_standard_item(&table_name.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&version.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                qlist.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());

                model.append_row_q_list_of_q_standard_item(qlist.as_ref());
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_table_name")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_version")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column_name")));
            model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column")));

            // Hide the column number column for tables.
            tree_view.hide_column(3);
            tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);

            tree_view.header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_results(
        view: &QPtr<QTreeView>,
        line_edit: &QPtr<QLineEdit>,
        column_combobox: &QPtr<QComboBox>,
        case_sensitive_button: &QPtr<QToolButton>,
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
    unsafe fn matches_from_selection(&self) -> Vec<MatchHolder> {

        let (model, tree_view) = match self.matches_tab_widget.current_index() {
            0 => (&self.matches_table_and_text_tree_model, &self.matches_table_and_text_tree_view),
            _ => return vec![],
        };

        let items = tree_view.get_items_from_selection(true);

        // For each item we follow the following logic:
        // - If it's a parent, it's all the matches on a table.
        // - If it's a child, check if the parent already exists.
        // - If it does, add another entry to it's matches.
        // - If not, create it with only that match.
        let mut table_matches: Vec<TableMatches> = vec![];
        let mut text_matches: Vec<TextMatches> = vec![];
        for item in items {
            if item.column() == 0 {
                let is_match = !item.has_children();

                // If it's a match (not an entire file), get the entry and add it to the tablematches of that table.
                if is_match {
                    let parent = item.parent();
                    let path = parent.text().to_std_string();
                    let file_type_index = parent.index().sibling_at_column(5);
                    let file_type = FileType::from(&*model.item_from_index(&file_type_index).text().to_std_string());

                    let column_name = parent.child_2a(item.row(), 1).text().to_std_string();
                    let column_number = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                    let row_number = parent.child_2a(item.row(), 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                    let text = parent.child_2a(item.row(), 0).text().to_std_string();

                    match file_type {
                        FileType::DB |
                        FileType::Loc => {
                            let match_file = match table_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let table = TableMatches::new(&path);
                                    table_matches.push(table);
                                    table_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = TableMatch::new(&column_name, column_number, row_number, &text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::Text => {
                            let lenght = parent.child_2a(item.row(), 4).text().to_std_string().parse().unwrap();
                            let match_file = match text_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let text = TextMatches::new(&path);
                                    text_matches.push(text);
                                    text_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = TextMatch::new(column_number as u64, row_number as u64, lenght, text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        _ => unimplemented!()
                    }
                }

                // If it's not a particular match, it's an entire file.
                else {
                    let path = item.text().to_std_string();
                    let file_type_index = item.index().sibling_at_column(5);
                    let file_type = FileType::from(&*model.item_from_index(&file_type_index).text().to_std_string());

                    // If it already exists, delete it, as the new one contains the entire set for it.
                    match file_type {
                        FileType::DB |
                        FileType::Loc => {
                            if let Some(position) = table_matches.iter().position(|x| x.path() == &path) {
                                table_matches.remove(position);
                            }

                            let table = TableMatches::new(&path);
                            table_matches.push(table);
                            let match_file = table_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let column_name = item.child_2a(row, 1).text().to_std_string();
                                let column_number = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let match_entry = TableMatch::new(&column_name, column_number, row_number, &text);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::Text => {
                            if let Some(position) = text_matches.iter().position(|x| x.path() == &path) {
                                text_matches.remove(position);
                            }

                            let text = TextMatches::new(&path);
                            text_matches.push(text);
                            let match_file = text_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let column_number = item.child_2a(row, 3).text().to_std_string().parse::<u64>().unwrap();
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let lenght = item.child_2a(row, 4).text().to_std_string().parse().unwrap();
                                let match_entry = TextMatch::new(column_number, row_number as u64, lenght, text);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        _ => unimplemented!()
                    }

                }
            }
        }

        let mut matches = table_matches.into_iter().map(MatchHolder::Table).collect::<Vec<_>>();
        matches.append(&mut text_matches.into_iter().map(MatchHolder::Text).collect::<Vec<_>>());
        matches
    }
}
