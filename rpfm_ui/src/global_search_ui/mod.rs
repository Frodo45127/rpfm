//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::push_button::PushButton;
use qt_widgets::tab_widget::TabWidget;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::object::Object;
use qt_core::qt::{DockWidgetArea};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;

use crate::ffi::new_treeview_filter;
use crate::QString;
use crate::utils::create_grid_layout_unsafe;

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
        global_search_dock_widget.set_window_title(&QString::from_std_str("Global Search"));

        // Create the search & replace section.
        let global_search_search_frame = GroupBox::new(&QString::from_std_str("Search Info"));
        let global_search_search_grid = create_grid_layout_unsafe(global_search_search_frame.as_mut_ptr() as *mut Widget);

        let global_search_search_line_edit = LineEdit::new(());
        let global_search_search_button = PushButton::new(&QString::from_std_str("Search"));

        let global_search_replace_line_edit = LineEdit::new(());
        let global_search_replace_button = PushButton::new(&QString::from_std_str("Replace"));
        let global_search_replace_all_button = PushButton::new(&QString::from_std_str("Replace All"));

        let global_search_case_sensitive_checkbox = CheckBox::new(&QString::from_std_str("Case Sensitive"));
        let global_search_use_regex_checkbox = CheckBox::new(&QString::from_std_str("Use Regex"));

        let global_search_search_on_group_box = GroupBox::new(&QString::from_std_str("Search On"));
        let global_search_search_on_grid = create_grid_layout_unsafe(global_search_search_on_group_box.as_mut_ptr() as *mut Widget);

        let global_search_search_on_all_checkbox = CheckBox::new(&QString::from_std_str("All"));
        let global_search_search_on_dbs_checkbox = CheckBox::new(&QString::from_std_str("DB"));
        let global_search_search_on_locs_checkbox = CheckBox::new(&QString::from_std_str("LOC"));
        let global_search_search_on_texts_checkbox = CheckBox::new(&QString::from_std_str("Text"));
        let global_search_search_on_schemas_checkbox = CheckBox::new(&QString::from_std_str("Schemas"));

        unsafe { global_search_search_grid.as_mut().unwrap().set_column_stretch(0, 10); }

        // Add everything to the Matches's Dock Layout.
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_search_line_edit.as_mut_ptr() as *mut Widget, 0, 0, 1, 2)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 2)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_search_button.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_button.as_mut_ptr() as *mut Widget, 1, 2, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_replace_all_button.as_mut_ptr() as *mut Widget, 1, 3, 1, 1)); }

        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_case_sensitive_checkbox.as_mut_ptr() as *mut Widget, 0, 3, 1, 1)); }
        unsafe { global_search_search_grid.as_mut().unwrap().add_widget((global_search_use_regex_checkbox.as_mut_ptr() as *mut Widget, 0, 4, 1, 1)); }
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
        let mut filter_matches_db_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive"));

        filter_matches_db_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!"));
        unsafe { filter_matches_db_column_selector.set_model(filter_matches_db_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_db_column_selector.add_item(&QString::from_std_str("PackedFile"));
        filter_matches_db_column_selector.add_item(&QString::from_std_str("Column"));
        filter_matches_db_column_selector.add_item(&QString::from_std_str("Row"));
        filter_matches_db_column_selector.add_item(&QString::from_std_str("Match"));
        filter_matches_db_case_sensitive_button.set_checkable(true);

        let mut filter_matches_loc_line_edit = LineEdit::new(());
        let mut filter_matches_loc_column_selector = ComboBox::new();
        let filter_matches_loc_column_list = StandardItemModel::new(());
        let mut filter_matches_loc_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive"));

        filter_matches_loc_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!"));
        unsafe { filter_matches_loc_column_selector.set_model(filter_matches_loc_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_loc_column_selector.add_item(&QString::from_std_str("PackedFile"));
        filter_matches_loc_column_selector.add_item(&QString::from_std_str("Column"));
        filter_matches_loc_column_selector.add_item(&QString::from_std_str("Row"));
        filter_matches_loc_column_selector.add_item(&QString::from_std_str("Match"));
        filter_matches_loc_case_sensitive_button.set_checkable(true);

        let mut filter_matches_text_line_edit = LineEdit::new(());
        let mut filter_matches_text_column_selector = ComboBox::new();
        let filter_matches_text_column_list = StandardItemModel::new(());
        let mut filter_matches_text_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive"));

        filter_matches_text_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!"));
        unsafe { filter_matches_text_column_selector.set_model(filter_matches_text_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_text_column_selector.add_item(&QString::from_std_str("PackedFile"));
        filter_matches_text_column_selector.add_item(&QString::from_std_str("Column"));
        filter_matches_text_column_selector.add_item(&QString::from_std_str("Row"));
        filter_matches_text_column_selector.add_item(&QString::from_std_str("Match"));
        filter_matches_text_case_sensitive_button.set_checkable(true);

        let mut filter_matches_schema_line_edit = LineEdit::new(());
        let mut filter_matches_schema_column_selector = ComboBox::new();
        let filter_matches_schema_column_list = StandardItemModel::new(());
        let mut filter_matches_schema_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive"));

        filter_matches_schema_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!"));
        unsafe { filter_matches_schema_column_selector.set_model(filter_matches_schema_column_list.into_raw() as *mut AbstractItemModel); }
        filter_matches_schema_column_selector.add_item(&QString::from_std_str("PackedFile"));
        filter_matches_schema_column_selector.add_item(&QString::from_std_str("Column"));
        filter_matches_schema_column_selector.add_item(&QString::from_std_str("Row"));
        filter_matches_schema_column_selector.add_item(&QString::from_std_str("Match"));
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

        unsafe { global_search_matches_tab_widget.add_tab((db_matches_widget.into_raw(), &QString::from_std_str("DB Matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((loc_matches_widget.into_raw(), &QString::from_std_str("Loc Matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((text_matches_widget.into_raw(), &QString::from_std_str("Text Matches"))); }
        unsafe { global_search_matches_tab_widget.add_tab((schema_matches_widget.into_raw(), &QString::from_std_str("Schema Matches"))); }

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
}
