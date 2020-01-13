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

use qt_widgets::abstract_item_view::ScrollMode;
use qt_widgets::check_box::CheckBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dock_widget::DockWidget;
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::push_button::PushButton;
use qt_widgets::tab_widget::TabWidget;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::flags::Flags;
use qt_core::model_index::ModelIndex;
use qt_core::object::Object;
use qt_core::qt::{DockWidgetArea, Orientation, SortOrder};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::variant::Variant;

use std::rc::Rc;
use std::cell::RefCell;

use rpfm_error::ErrorKind;

use rpfm_lib::packfile::PathType;
use rpfm_lib::global_search::{GlobalSearch, schema::SchemaMatches, table::TableMatches, text::TextMatches};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::new_treeview_filter;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{TheOneSlot, View};
use crate::pack_tree::PackTree;
use crate::QString;
use crate::utils::{create_grid_layout_unsafe, show_dialog};
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
    pub global_search_dock_widget: *mut DockWidget,
    pub global_search_search_line_edit: *mut LineEdit,
    pub global_search_search_button: *mut PushButton,

    pub global_search_replace_line_edit: *mut LineEdit,
    pub global_search_replace_button: *mut PushButton,
    pub global_search_replace_all_button: *mut PushButton,

    pub global_search_clear_button: *mut PushButton,
    pub global_search_case_sensitive_checkbox: *mut CheckBox,
    pub global_search_use_regex_checkbox: *mut CheckBox,

    pub global_search_search_on_all_checkbox: *mut CheckBox,
    pub global_search_search_on_dbs_checkbox: *mut CheckBox,
    pub global_search_search_on_locs_checkbox: *mut CheckBox,
    pub global_search_search_on_texts_checkbox: *mut CheckBox,
    pub global_search_search_on_schemas_checkbox: *mut CheckBox,

    pub global_search_matches_tab_widget: *mut TabWidget,

    pub global_search_matches_db_tree_view: *mut TreeView,
    pub global_search_matches_loc_tree_view: *mut TreeView,
    pub global_search_matches_text_tree_view: *mut TreeView,
    pub global_search_matches_schema_tree_view: *mut TreeView,

    pub global_search_matches_db_tree_filter: *mut SortFilterProxyModel,
    pub global_search_matches_loc_tree_filter: *mut SortFilterProxyModel,
    pub global_search_matches_text_tree_filter: *mut SortFilterProxyModel,
    pub global_search_matches_schema_tree_filter: *mut SortFilterProxyModel,

    pub global_search_matches_db_tree_model: *mut StandardItemModel,
    pub global_search_matches_loc_tree_model: *mut StandardItemModel,
    pub global_search_matches_text_tree_model: *mut StandardItemModel,
    pub global_search_matches_schema_tree_model: *mut StandardItemModel,

    pub global_search_matches_filter_db_line_edit: *mut LineEdit,
    pub global_search_matches_filter_loc_line_edit: *mut LineEdit,
    pub global_search_matches_filter_text_line_edit: *mut LineEdit,
    pub global_search_matches_filter_schema_line_edit: *mut LineEdit,

    pub global_search_matches_case_sensitive_db_button: *mut PushButton,
    pub global_search_matches_case_sensitive_loc_button: *mut PushButton,
    pub global_search_matches_case_sensitive_text_button: *mut PushButton,
    pub global_search_matches_case_sensitive_schema_button: *mut PushButton,

    pub global_search_matches_column_selector_db_combobox: *mut ComboBox,
    pub global_search_matches_column_selector_loc_combobox: *mut ComboBox,
    pub global_search_matches_column_selector_text_combobox: *mut ComboBox,
    pub global_search_matches_column_selector_schema_combobox: *mut ComboBox,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchUI`.
impl GlobalSearchUI {

    /// This function creates an entire `GlobalSearchUI` struct.
    pub fn new(main_window: *mut MainWindow) -> Self {

        // Create and configure the 'Global Search` Dock Widget and all his contents.
        let mut global_search_dock_widget = unsafe { DockWidget::new_unsafe(main_window as *mut Widget) };
        let global_search_dock_inner_widget = Widget::new();
        let global_search_dock_layout = create_grid_layout_unsafe(global_search_dock_inner_widget.as_mut_ptr() as *mut Widget);
        unsafe { global_search_dock_widget.set_widget(global_search_dock_inner_widget.into_raw()); }
        unsafe { main_window.as_mut().unwrap().add_dock_widget((DockWidgetArea::RightDockWidgetArea, global_search_dock_widget.as_mut_ptr())); }
        global_search_dock_widget.set_window_title(&qtr("global_search"));

        // Create the search & replace section.
        let global_search_search_frame = GroupBox::new(&qtr("global_search_info"));
        let global_search_search_grid = create_grid_layout_unsafe(global_search_search_frame.as_mut_ptr() as *mut Widget);

        let global_search_search_line_edit = LineEdit::new(());
        let global_search_search_button = PushButton::new(&qtr("global_search_search"));

        let global_search_replace_line_edit = LineEdit::new(());
        let global_search_replace_button = PushButton::new(&qtr("global_search_replace"));
        let global_search_replace_all_button = PushButton::new(&qtr("global_search_replace_all"));

        let global_search_clear_button = PushButton::new(&qtr("global_search_clear"));
        let global_search_case_sensitive_checkbox = CheckBox::new(&qtr("global_search_case_sensitive"));
        let global_search_use_regex_checkbox = CheckBox::new(&qtr("global_search_use_regex"));

        let global_search_search_on_group_box = GroupBox::new(&qtr("global_search_search_on"));
        let global_search_search_on_grid = create_grid_layout_unsafe(global_search_search_on_group_box.as_mut_ptr() as *mut Widget);

        let global_search_search_on_all_checkbox = CheckBox::new(&qtr("global_search_all"));
        let global_search_search_on_dbs_checkbox = CheckBox::new(&qtr("global_search_db"));
        let global_search_search_on_locs_checkbox = CheckBox::new(&qtr("global_search_loc"));
        let global_search_search_on_texts_checkbox = CheckBox::new(&qtr("global_search_txt"));
        let global_search_search_on_schemas_checkbox = CheckBox::new(&qtr("global_search_schemas"));

        unsafe { global_search_search_grid.as_mut().unwrap().set_column_stretch(0, 10); }

        // Add everything to the Matches's Dock Layout.
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_search_line_edit.as_mut_ptr() as *mut Widget, 0, 0, 1, 2)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 2)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_search_button.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_button.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_all_button.as_mut_ptr() as *mut Widget, 1, 3, 1, 1)); }

        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_clear_button.as_mut_ptr() as *mut Widget, 0, 3, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_case_sensitive_checkbox.as_mut_ptr() as *mut Widget, 0, 4, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_use_regex_checkbox.as_mut_ptr() as *mut Widget, 1, 4, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_search_on_group_box.into_raw() as *mut Widget, 2, 0, 1, 10)); }

        unsafe { global_search_search_on_grid.as_mut().unwrap().add_widget((global_search_search_on_all_checkbox.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { global_search_search_on_grid.as_mut().unwrap().add_widget((global_search_search_on_dbs_checkbox.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { global_search_search_on_grid.as_mut().unwrap().add_widget((global_search_search_on_locs_checkbox.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }
        unsafe { global_search_search_on_grid.as_mut().unwrap().add_widget((global_search_search_on_texts_checkbox.as_mut_ptr() as *mut Widget, 0, 3, 1, 1)); }
        unsafe { global_search_search_on_grid.as_mut().unwrap().add_widget((global_search_search_on_schemas_checkbox.as_mut_ptr() as *mut Widget, 0, 4, 1, 1)); }

        // Create the frames for the matches tables.
        let mut global_search_matches_tab_widget = TabWidget::new();

        let db_matches_widget = Widget::new();
        let db_matches_grid = create_grid_layout_unsafe(db_matches_widget.as_mut_ptr() as *mut Widget);

        let loc_matches_widget = Widget::new();
        let loc_matches_grid = create_grid_layout_unsafe(loc_matches_widget.as_mut_ptr() as *mut Widget);

        let text_matches_widget = Widget::new();
        let text_matches_grid = create_grid_layout_unsafe(text_matches_widget.as_mut_ptr() as *mut Widget);

        let schema_matches_widget = Widget::new();
        let schema_matches_grid = create_grid_layout_unsafe(schema_matches_widget.as_mut_ptr() as *mut Widget);

        // `TreeView`s with all the matches.
        let mut tree_view_matches_db = TreeView::new();
        let mut tree_view_matches_loc = TreeView::new();
        let mut tree_view_matches_text = TreeView::new();
        let mut tree_view_matches_schema = TreeView::new();

        let filter_model_matches_db = unsafe { new_treeview_filter(db_matches_widget.as_mut_ptr() as *mut Object) };
        let filter_model_matches_loc = unsafe { new_treeview_filter(loc_matches_widget.as_mut_ptr() as *mut Object) };
        let filter_model_matches_text = unsafe { new_treeview_filter(text_matches_widget.as_mut_ptr() as *mut Object) };
        let filter_model_matches_schema = unsafe { new_treeview_filter(schema_matches_widget.as_mut_ptr() as *mut Object) };

        let model_matches_db = StandardItemModel::new(());
        let model_matches_loc = StandardItemModel::new(());
        let model_matches_text = StandardItemModel::new(());
        let model_matches_schema = StandardItemModel::new(());

        unsafe { filter_model_matches_db.as_mut().unwrap().set_source_model(model_matches_db.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { filter_model_matches_loc.as_mut().unwrap().set_source_model(model_matches_loc.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { filter_model_matches_text.as_mut().unwrap().set_source_model(model_matches_text.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { filter_model_matches_schema.as_mut().unwrap().set_source_model(model_matches_schema.as_mut_ptr() as *mut AbstractItemModel); }

        unsafe { tree_view_matches_db.set_model(filter_model_matches_db as *mut AbstractItemModel); }
        unsafe { tree_view_matches_loc.set_model(filter_model_matches_loc as *mut AbstractItemModel); }
        unsafe { tree_view_matches_text.set_model(filter_model_matches_text as *mut AbstractItemModel); }
        unsafe { tree_view_matches_schema.set_model(filter_model_matches_schema as *mut AbstractItemModel); }

        tree_view_matches_db.set_horizontal_scroll_mode(ScrollMode::Pixel);
        tree_view_matches_db.set_sorting_enabled(true);
        unsafe { tree_view_matches_db.header().as_mut().unwrap().set_visible(true); }
        unsafe { tree_view_matches_db.header().as_mut().unwrap().set_stretch_last_section(true); }

        tree_view_matches_loc.set_horizontal_scroll_mode(ScrollMode::Pixel);
        tree_view_matches_loc.set_sorting_enabled(true);
        unsafe { tree_view_matches_loc.header().as_mut().unwrap().set_visible(true); }
        unsafe { tree_view_matches_loc.header().as_mut().unwrap().set_stretch_last_section(true); }

        tree_view_matches_text.set_horizontal_scroll_mode(ScrollMode::Pixel);
        tree_view_matches_text.set_sorting_enabled(true);
        unsafe { tree_view_matches_text.header().as_mut().unwrap().set_visible(true); }
        unsafe { tree_view_matches_text.header().as_mut().unwrap().set_stretch_last_section(true); }

        tree_view_matches_schema.set_horizontal_scroll_mode(ScrollMode::Pixel);
        tree_view_matches_schema.set_sorting_enabled(true);
        unsafe { tree_view_matches_schema.header().as_mut().unwrap().set_visible(true); }
        unsafe { tree_view_matches_schema.header().as_mut().unwrap().set_stretch_last_section(true); }

        // Filters for the matches `TreeViews`.
        let mut filter_matches_db_line_edit = LineEdit::new(());
        let mut filter_matches_db_column_selector = ComboBox::new();
        let filter_matches_db_column_list = StandardItemModel::new(());
        let mut filter_matches_db_case_sensitive_button = PushButton::new(&qtr("global_search_case_sensitive"));

        filter_matches_db_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        unsafe { filter_matches_db_column_selector.set_model(filter_matches_db_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_db_column_selector.add_item(&qtr("gen_loc_packedfile"));
        filter_matches_db_column_selector.add_item(&qtr("gen_loc_column"));
        filter_matches_db_column_selector.add_item(&qtr("gen_loc_row"));
        filter_matches_db_column_selector.add_item(&qtr("gen_loc_match"));
        filter_matches_db_case_sensitive_button.set_checkable(true);

        let mut filter_matches_loc_line_edit = LineEdit::new(());
        let mut filter_matches_loc_column_selector = ComboBox::new();
        let filter_matches_loc_column_list = StandardItemModel::new(());
        let mut filter_matches_loc_case_sensitive_button = PushButton::new(&qtr("global_search_case_sensitive"));

        filter_matches_loc_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        unsafe { filter_matches_loc_column_selector.set_model(filter_matches_loc_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_loc_column_selector.add_item(&qtr("gen_loc_packedfile"));
        filter_matches_loc_column_selector.add_item(&qtr("gen_loc_column"));
        filter_matches_loc_column_selector.add_item(&qtr("gen_loc_row"));
        filter_matches_loc_column_selector.add_item(&qtr("gen_loc_match"));
        filter_matches_loc_case_sensitive_button.set_checkable(true);

        let mut filter_matches_text_line_edit = LineEdit::new(());
        let mut filter_matches_text_column_selector = ComboBox::new();
        let filter_matches_text_column_list = StandardItemModel::new(());
        let mut filter_matches_text_case_sensitive_button = PushButton::new(&qtr("global_search_case_sensitive"));

        filter_matches_text_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        unsafe { filter_matches_text_column_selector.set_model(filter_matches_text_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_text_column_selector.add_item(&qtr("gen_loc_packedfile"));
        filter_matches_text_column_selector.add_item(&qtr("gen_loc_column"));
        filter_matches_text_column_selector.add_item(&qtr("gen_loc_row"));
        filter_matches_text_column_selector.add_item(&qtr("gen_loc_match"));
        filter_matches_text_case_sensitive_button.set_checkable(true);

        let mut filter_matches_schema_line_edit = LineEdit::new(());
        let mut filter_matches_schema_column_selector = ComboBox::new();
        let filter_matches_schema_column_list = StandardItemModel::new(());
        let mut filter_matches_schema_case_sensitive_button = PushButton::new(&qtr("global_search_case_sensitive"));

        filter_matches_schema_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        unsafe { filter_matches_schema_column_selector.set_model(filter_matches_schema_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_schema_column_selector.add_item(&qtr("gen_loc_packedfile"));
        filter_matches_schema_column_selector.add_item(&qtr("gen_loc_column"));
        filter_matches_schema_column_selector.add_item(&qtr("gen_loc_row"));
        filter_matches_schema_column_selector.add_item(&qtr("gen_loc_match"));
        filter_matches_schema_case_sensitive_button.set_checkable(true);

        // Add everything to the Matches's Dock Layout.
        unsafe { db_matches_grid.as_mut().unwrap().add_widget((tree_view_matches_db.as_mut_ptr() as *mut Widget, 0, 0, 1, 3)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((tree_view_matches_loc.as_mut_ptr() as *mut Widget, 0, 0, 1, 3)); }
        unsafe { text_matches_grid.as_mut().unwrap().add_widget((tree_view_matches_text.as_mut_ptr() as *mut Widget, 0, 0, 1, 3)); }
        unsafe { schema_matches_grid.as_mut().unwrap().add_widget((tree_view_matches_schema.as_mut_ptr() as *mut Widget, 0, 0, 1, 3)); }

        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_case_sensitive_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_column_selector.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }

        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_case_sensitive_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_column_selector.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }

        unsafe { text_matches_grid.as_mut().unwrap().add_widget((filter_matches_text_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { text_matches_grid.as_mut().unwrap().add_widget((filter_matches_text_case_sensitive_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { text_matches_grid.as_mut().unwrap().add_widget((filter_matches_text_column_selector.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }

        unsafe { schema_matches_grid.as_mut().unwrap().add_widget((filter_matches_schema_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { schema_matches_grid.as_mut().unwrap().add_widget((filter_matches_schema_case_sensitive_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { schema_matches_grid.as_mut().unwrap().add_widget((filter_matches_schema_column_selector.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }

        unsafe { global_search_matches_tab_widget.add_tab((db_matches_widget.into_raw(), &qtr("global_search_db_matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((loc_matches_widget.into_raw(), &qtr("global_search_loc_matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((text_matches_widget.into_raw(), &qtr("global_search_txt_matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((schema_matches_widget.into_raw(), &qtr("global_search_schema_matches"))); }

        unsafe { global_search_dock_layout.as_mut().unwrap().add_widget((global_search_search_frame.into_raw() as *mut Widget, 0, 0, 1, 3)); }
        unsafe { global_search_dock_layout.as_mut().unwrap().add_widget((global_search_matches_tab_widget.as_mut_ptr() as *mut Widget, 1, 0, 1, 3)); }

        // Hide this widget by default.
        global_search_dock_widget.hide();

        // Create ***Da monsta***.
        Self {
            global_search_dock_widget: global_search_dock_widget.into_raw(),
            global_search_search_line_edit: global_search_search_line_edit.into_raw(),
            global_search_search_button: global_search_search_button.into_raw(),

            global_search_replace_line_edit: global_search_replace_line_edit.into_raw(),
            global_search_replace_button: global_search_replace_button.into_raw(),
            global_search_replace_all_button: global_search_replace_all_button.into_raw(),

            global_search_clear_button: global_search_clear_button.into_raw(),
            global_search_case_sensitive_checkbox: global_search_case_sensitive_checkbox.into_raw(),
            global_search_use_regex_checkbox: global_search_use_regex_checkbox.into_raw(),

            global_search_search_on_all_checkbox: global_search_search_on_all_checkbox.into_raw(),
            global_search_search_on_dbs_checkbox: global_search_search_on_dbs_checkbox.into_raw(),
            global_search_search_on_locs_checkbox: global_search_search_on_locs_checkbox.into_raw(),
            global_search_search_on_texts_checkbox: global_search_search_on_texts_checkbox.into_raw(),
            global_search_search_on_schemas_checkbox: global_search_search_on_schemas_checkbox.into_raw(),

            global_search_matches_tab_widget: global_search_matches_tab_widget.into_raw(),

            global_search_matches_db_tree_view: tree_view_matches_db.into_raw(),
            global_search_matches_loc_tree_view: tree_view_matches_loc.into_raw(),
            global_search_matches_text_tree_view: tree_view_matches_text.into_raw(),
            global_search_matches_schema_tree_view: tree_view_matches_schema.into_raw(),

            global_search_matches_db_tree_filter: filter_model_matches_db,
            global_search_matches_loc_tree_filter: filter_model_matches_loc,
            global_search_matches_text_tree_filter: filter_model_matches_text,
            global_search_matches_schema_tree_filter: filter_model_matches_schema,

            global_search_matches_db_tree_model: model_matches_db.into_raw(),
            global_search_matches_loc_tree_model: model_matches_loc.into_raw(),
            global_search_matches_text_tree_model: model_matches_text.into_raw(),
            global_search_matches_schema_tree_model: model_matches_schema.into_raw(),

            global_search_matches_filter_db_line_edit: filter_matches_db_line_edit.into_raw(),
            global_search_matches_filter_loc_line_edit: filter_matches_loc_line_edit.into_raw(),
            global_search_matches_filter_text_line_edit: filter_matches_text_line_edit.into_raw(),
            global_search_matches_filter_schema_line_edit: filter_matches_schema_line_edit.into_raw(),

            global_search_matches_case_sensitive_db_button: filter_matches_db_case_sensitive_button.into_raw(),
            global_search_matches_case_sensitive_loc_button: filter_matches_loc_case_sensitive_button.into_raw(),
            global_search_matches_case_sensitive_text_button: filter_matches_text_case_sensitive_button.into_raw(),
            global_search_matches_case_sensitive_schema_button: filter_matches_schema_case_sensitive_button.into_raw(),

            global_search_matches_column_selector_db_combobox: filter_matches_db_column_selector.into_raw(),
            global_search_matches_column_selector_loc_combobox: filter_matches_loc_column_selector.into_raw(),
            global_search_matches_column_selector_text_combobox: filter_matches_text_column_selector.into_raw(),
            global_search_matches_column_selector_schema_combobox: filter_matches_schema_column_selector.into_raw(),
        }
    }

    /// This function is used to search the entire PackFile, using the data in Self for the search.
    pub fn search(&self) {

        // Create the global search and populate it with all the settings for the search.
        let mut global_search = GlobalSearch::default();
        global_search.pattern = unsafe { self.global_search_search_line_edit.as_ref().unwrap().text().to_std_string() };
        global_search.case_sensitive = unsafe { self.global_search_case_sensitive_checkbox.as_ref().unwrap().is_checked() };
        global_search.use_regex = unsafe { self.global_search_use_regex_checkbox.as_ref().unwrap().is_checked() };

        if unsafe { self.global_search_search_on_all_checkbox.as_ref().unwrap().is_checked() } {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = unsafe { self.global_search_search_on_dbs_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_locs = unsafe { self.global_search_search_on_locs_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_texts = unsafe { self.global_search_search_on_texts_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_schema = unsafe { self.global_search_search_on_schemas_checkbox.as_ref().unwrap().is_checked() };
        }

        let t = std::time::SystemTime::now();
        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearch(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let tree_view_db = unsafe { self.global_search_matches_db_tree_view.as_mut().unwrap() };
        let tree_view_loc = unsafe { self.global_search_matches_loc_tree_view.as_mut().unwrap() };
        let tree_view_text = unsafe { self.global_search_matches_text_tree_view.as_mut().unwrap() };
        let tree_view_schema = unsafe { self.global_search_matches_schema_tree_view.as_mut().unwrap() };

        let model_db = unsafe { self.global_search_matches_db_tree_model.as_mut().unwrap() };
        let model_loc = unsafe { self.global_search_matches_loc_tree_model.as_mut().unwrap() };
        let model_text = unsafe { self.global_search_matches_text_tree_model.as_mut().unwrap() };
        let model_schema = unsafe { self.global_search_matches_schema_tree_model.as_mut().unwrap() };

        model_db.clear();
        model_loc.clear();
        model_text.clear();
        model_schema.clear();

        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::GlobalSearch(global_search) => {

                println!("Time to search from click to search complete: {:?}", t.elapsed().unwrap());

                // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                Self::load_table_matches_to_ui(model_db, tree_view_db, &global_search.matches_db);
                Self::load_table_matches_to_ui(model_loc, tree_view_loc, &global_search.matches_loc);
                Self::load_text_matches_to_ui(model_text, tree_view_text, &global_search.matches_text);
                Self::load_schema_matches_to_ui(model_schema, tree_view_schema, &global_search.matches_schema);
                //println!("{:?}", global_search);
                UI_STATE.set_global_search(&global_search);
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This function takes care of updating the results of a global search for the provided paths.
    ///
    /// NOTE: This only works in the `editable` search results, which are DB Tables, Locs and Text PackedFiles.
    pub fn search_on_path(&self, paths: Vec<PathType>) {

        // Create the global search and populate it with all the settings for the search.
        let global_search = UI_STATE.get_global_search();

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearchUpdate(global_search, paths));

        // While we wait for an answer, we need to clear the current results panels.
        let tree_view_db = unsafe { self.global_search_matches_db_tree_view.as_mut().unwrap() };
        let tree_view_loc = unsafe { self.global_search_matches_loc_tree_view.as_mut().unwrap() };
        let tree_view_text = unsafe { self.global_search_matches_text_tree_view.as_mut().unwrap() };

        let model_db = unsafe { self.global_search_matches_db_tree_model.as_mut().unwrap() };
        let model_loc = unsafe { self.global_search_matches_loc_tree_model.as_mut().unwrap() };
        let model_text = unsafe { self.global_search_matches_text_tree_model.as_mut().unwrap() };

        model_db.clear();
        model_loc.clear();
        model_text.clear();

        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::GlobalSearch(global_search) => {

                // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                Self::load_table_matches_to_ui(model_db, tree_view_db, &global_search.matches_db);
                Self::load_table_matches_to_ui(model_loc, tree_view_loc, &global_search.matches_loc);
                Self::load_text_matches_to_ui(model_text, tree_view_text, &global_search.matches_text);
            }

            // In ANY other situation, it's a message problem.
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub fn clear(&self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        unsafe { self.global_search_matches_db_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_loc_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_text_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_schema_tree_model.as_mut().unwrap().clear() };
    }

    /// This function replace the currently selected match with the provided text.
    pub fn replace(&self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        unsafe { self.global_search_matches_db_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_loc_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_text_tree_model.as_mut().unwrap().clear() };
        unsafe { self.global_search_matches_schema_tree_model.as_mut().unwrap().clear() };
    }

    /// This function replace all the matches in the current search with the provided text.
    pub fn replace_all(&self, app_ui: &AppUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) {

        // To avoid conflicting data, we close all PackedFiles hard and re-search before replacing.
        app_ui.purge_them_all(*self, slot_holder);
        self.search();

        let mut global_search = UI_STATE.get_global_search();
        global_search.pattern = unsafe { self.global_search_search_line_edit.as_ref().unwrap().text().to_std_string() };
        global_search.replace_text = unsafe { self.global_search_replace_line_edit.as_ref().unwrap().text().to_std_string() };
        global_search.case_sensitive = unsafe { self.global_search_case_sensitive_checkbox.as_ref().unwrap().is_checked() };
        global_search.use_regex = unsafe { self.global_search_use_regex_checkbox.as_ref().unwrap().is_checked() };

        if unsafe { self.global_search_search_on_all_checkbox.as_ref().unwrap().is_checked() } {
            global_search.search_on_dbs = true;
            global_search.search_on_locs = true;
            global_search.search_on_texts = true;
            global_search.search_on_schema = true;
        }
        else {
            global_search.search_on_dbs = unsafe { self.global_search_search_on_dbs_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_locs = unsafe { self.global_search_search_on_locs_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_texts = unsafe { self.global_search_search_on_texts_checkbox.as_ref().unwrap().is_checked() };
            global_search.search_on_schema = unsafe { self.global_search_search_on_schemas_checkbox.as_ref().unwrap().is_checked() };
        }

        CENTRAL_COMMAND.send_message_qt(Command::GlobalSearchReplaceAll(global_search));

        // While we wait for an answer, we need to clear the current results panels.
        let model_db = unsafe { self.global_search_matches_db_tree_model.as_mut().unwrap() };
        let model_loc = unsafe { self.global_search_matches_loc_tree_model.as_mut().unwrap() };
        let model_text = unsafe { self.global_search_matches_text_tree_model.as_mut().unwrap() };

        model_db.clear();
        model_loc.clear();
        model_text.clear();

        match CENTRAL_COMMAND.recv_message_qt() {
            Response::GlobalSearch(global_search) => {
                UI_STATE.set_global_search(&global_search);
                self.search();
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
    pub fn open_match(
        app_ui: AppUI,
        pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
        model_index_filtered: &ModelIndex
    ) {

        let tree_view = unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap() };
        let filter_model = unsafe { (model_index_filtered.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter_model.source_model() as *mut StandardItemModel).as_ref().unwrap() };
        let model_index = filter_model.map_to_source(&model_index_filtered);

        let gidhora = unsafe { model.item_from_index(&model_index).as_ref().unwrap() };
        let is_match = !gidhora.has_children();

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        if is_match {
            let parent = unsafe { gidhora.parent().as_ref().unwrap() };
            let path = parent.text().to_std_string();
            let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

            if let Some(pack_file_contents_model_index) = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path) {
                let selection_model = unsafe { tree_view.selection_model().as_mut().unwrap() };

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if pack_file_contents_model_index.is_valid() {
                    selection_model.select((&pack_file_contents_model_index, Flags::from_enum(SelectionFlag::ClearAndSelect)));
                    tree_view.scroll_to(&pack_file_contents_model_index);
                    app_ui.open_packedfile(&pack_file_contents_ui, &global_search_ui, &slot_holder, false);

                    if let Some((_, packed_file_view)) = UI_STATE.get_open_packedfiles().iter().find(|x| x.0 == &path) {
                        match packed_file_view.get_view() {

                            // In case of tables, we have to get the logical row/column of the match and select it.
                            View::Table(view) => {
                                let table_view = view.get_table();
                                let table_filter = unsafe { (table_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
                                let table_model = unsafe { (table_filter.source_model() as *mut StandardItemModel).as_ref().unwrap() };
                                let table_selection_model = unsafe { table_view.selection_model().as_mut().unwrap() };

                                let row = unsafe { parent.child((model_index.row(), 1)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() - 1 };
                                let column = unsafe { parent.child((model_index.row(), 3)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() };

                                let table_model_index = table_model.index((row, column));
                                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                                if table_model_index_filtered.is_valid() {
                                    table_selection_model.select((&table_model_index_filtered, Flags::from_enum(SelectionFlag::ClearAndSelect)));
                                    table_view.scroll_to(&table_model_index_filtered);
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
                let selection_model = unsafe { tree_view.selection_model().as_mut().unwrap() };

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if model_index.is_valid() {
                    selection_model.select((&model_index, Flags::from_enum(SelectionFlag::ClearAndSelect)));
                    tree_view.scroll_to(&model_index);
                    app_ui.open_packedfile(&pack_file_contents_ui, &global_search_ui, &slot_holder, false);
                }
            }
            else { show_dialog(app_ui.main_window as *mut Widget, ErrorKind::PackedFileNotInFilter, false); }
        }
    }


    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    fn load_table_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[TableMatches]) {
        if !matches.is_empty() {

            for match_table in matches {
                if !match_table.matches.is_empty() {
                    let path = match_table.path.join("/");
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());
                    let mut fill3 = StandardItem::new(());
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_table.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut column_name = StandardItem::new(());
                        let mut column_number = StandardItem::new(());
                        let mut row = StandardItem::new(());
                        let mut text = StandardItem::new(());

                        column_name.set_text(&QString::from_std_str(&match_row.column_name));
                        column_number.set_data((&Variant::new2(match_row.column_number), 2));
                        row.set_data((&Variant::new2(match_row.row_number + 1), 2));
                        text.set_text(&QString::from_std_str(&match_row.contents));

                        column_name.set_editable(false);
                        column_number.set_editable(false);
                        row.set_editable(false);
                        text.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&column_name.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&row.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&text.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column_number.into_raw()); }

                        // Append the new row.
                        file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill3.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&qtr("global_search_match_packedfile_column"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&qtr("gen_loc_row"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&qtr("gen_loc_match"))));

            // Hide the column number column for tables.
            tree_view.hide_column(3);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }

    /// This function takes care of loading the results of a global search of `TextMatches` into a model.
    fn load_text_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[TextMatches]) {
        if !matches.is_empty() {
            for match_text in matches {
                if !match_text.matches.is_empty() {
                    let path = match_text.path.join("/");
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());
                    let mut fill3 = StandardItem::new(());
                    file.set_text(&QString::from_std_str(&path));
                    file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);
                    fill3.set_editable(false);

                    for match_row in &match_text.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut text = StandardItem::new(());
                        let mut row = StandardItem::new(());
                        let mut column = StandardItem::new(());
                        let mut len = StandardItem::new(());

                        text.set_text(&QString::from_std_str(&match_row.text));
                        row.set_data((&Variant::new0(match_row.row + 1), 2));
                        column.set_data((&Variant::new0(match_row.column), 2));
                        len.set_data((&Variant::new2(match_row.len), 2));

                        text.set_editable(false);
                        row.set_editable(false);
                        column.set_editable(false);
                        len.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&text.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&row.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&len.into_raw()); }

                        // Append the new row.
                        file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill3.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&qtr("global_search_match_packedfile_text"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&qtr("gen_loc_row"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&qtr("gen_loc_column"))));
            model.set_header_data((3, Orientation::Horizontal, &Variant::new0(&qtr("gen_loc_length"))));

            // Hide the column and lenght numbers on the TreeView.
            tree_view.hide_column(2);
            tree_view.hide_column(3);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    fn load_schema_matches_to_ui(model: &mut StandardItemModel, tree_view: &mut TreeView, matches: &[SchemaMatches]) {
        if !matches.is_empty() {

            for match_schema in matches {
                if !match_schema.matches.is_empty() {
                    let mut qlist_daddy = ListStandardItemMutPtr::new(());
                    let mut versioned_file = StandardItem::new(());
                    let mut fill1 = StandardItem::new(());
                    let mut fill2 = StandardItem::new(());

                    let name = if let Some(ref name) = match_schema.versioned_file_name {
                        format!("{}/{}", match_schema.versioned_file_type, name)
                    } else { match_schema.versioned_file_type.to_string() };

                    versioned_file.set_text(&QString::from_std_str(&name));
                    versioned_file.set_editable(false);
                    fill1.set_editable(false);
                    fill2.set_editable(false);

                    for match_row in &match_schema.matches {

                        // Create a new list of StandardItem.
                        let mut qlist_boi = ListStandardItemMutPtr::new(());

                        // Create an empty row.
                        let mut name = StandardItem::new(());
                        let mut version = StandardItem::new(());
                        let mut column = StandardItem::new(());

                        name.set_text(&QString::from_std_str(&match_row.name));
                        version.set_data((&Variant::new0(match_row.version), 2));
                        column.set_data((&Variant::new2(match_row.column), 2));

                        name.set_editable(false);
                        version.set_editable(false);
                        column.set_editable(false);

                        // Add an empty row to the list.
                        unsafe { qlist_boi.append_unsafe(&name.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&version.into_raw()); }
                        unsafe { qlist_boi.append_unsafe(&column.into_raw()); }

                        // Append the new row.
                        versioned_file.append_row(&qlist_boi);
                    }
                    unsafe { qlist_daddy.append_unsafe(&versioned_file.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill1.into_raw()); }
                    unsafe { qlist_daddy.append_unsafe(&fill2.into_raw()); }
                    model.append_row(&qlist_daddy);
                }
            }

            model.set_header_data((0, Orientation::Horizontal, &Variant::new0(&qtr("global_search_versioned_file"))));
            model.set_header_data((1, Orientation::Horizontal, &Variant::new0(&qtr("global_search_definition_version"))));
            model.set_header_data((2, Orientation::Horizontal, &Variant::new0(&qtr("global_search_column_index"))));

            // Hide the column number column for tables.
            tree_view.hide_column(2);
            tree_view.sort_by_column((0, SortOrder::Ascending));

            unsafe { tree_view.header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        }
    }
}
