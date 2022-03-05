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
Module with all the code related to the `DiagnosticsUI`.
!*/

use qt_widgets::q_abstract_item_view::{ScrollHint, SelectionMode};
use qt_widgets::QCheckBox;
use qt_widgets::QDockWidget;
use qt_widgets::{QFrame, q_frame::{Shadow, Shape}};
use qt_widgets::QGroupBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLabel;
use qt_widgets::QMainWindow;
use qt_widgets::QPushButton;
use qt_widgets::QScrollArea;
use qt_widgets::QTableView;
use qt_widgets::QWidget;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::{AlignmentFlag, CaseSensitivity, ContextMenuPolicy, DockWidgetArea, Orientation, SortOrder};
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;
use qt_core::QObject;
use qt_core::QSignalBlocker;

use cpp_core::CppBox;
use cpp_core::Ptr;

use std::rc::Rc;

use rpfm_lib::diagnostics::{*, anim_fragment::*, config::*, table::*, dependency_manager::*, packfile::*};
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::games::supported_games::*;
use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use rpfm_macros::{GetRef, GetRefMut, Set};

use crate::AppUI;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::CENTRAL_COMMAND;
use crate::dependencies_ui::DependenciesUI;
use crate::ffi::{new_tableview_filter_safe, trigger_tableview_filter_safe};
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, qtre, tr};
use crate::pack_tree::{PackTree, get_color_info, get_color_warning, get_color_error, get_color_info_pressed, get_color_warning_pressed, get_color_error_pressed, TreeViewOperation};
use crate::packedfile_views::{DataSource, PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use crate::utils::create_grid_layout;
use crate::views::table::{ITEM_HAS_ERROR, ITEM_HAS_WARNING, ITEM_HAS_INFO};

pub mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Diagnostics panel.
#[derive(GetRef, GetRefMut, Set)]
pub struct DiagnosticsUI {

    //-------------------------------------------------------------------------------//
    // `Diagnostics` Dock Widget.
    //-------------------------------------------------------------------------------//
    diagnostics_dock_widget: QBox<QDockWidget>,
    diagnostics_table_view: QBox<QTableView>,
    diagnostics_table_filter: QBox<QSortFilterProxyModel>,
    diagnostics_table_model: QBox<QStandardItemModel>,

    //-------------------------------------------------------------------------------//
    // Filters section.
    //-------------------------------------------------------------------------------//
    diagnostics_button_check_packfile: QBox<QPushButton>,
    diagnostics_button_check_current_packed_file: QBox<QPushButton>,
    diagnostics_button_error: QBox<QPushButton>,
    diagnostics_button_warning: QBox<QPushButton>,
    diagnostics_button_info: QBox<QPushButton>,
    diagnostics_button_only_current_packed_file: QBox<QPushButton>,
    diagnostics_button_show_more_filters: QBox<QPushButton>,

    sidebar_scroll_area: QBox<QScrollArea>,
    checkbox_all: QBox<QCheckBox>,
    checkbox_outdated_table: QBox<QCheckBox>,
    checkbox_invalid_reference: QBox<QCheckBox>,
    checkbox_empty_row: QBox<QCheckBox>,
    checkbox_empty_key_field: QBox<QCheckBox>,
    checkbox_empty_key_fields: QBox<QCheckBox>,
    checkbox_duplicated_combined_keys: QBox<QCheckBox>,
    checkbox_no_reference_table_found: QBox<QCheckBox>,
    checkbox_no_reference_table_nor_column_found_pak: QBox<QCheckBox>,
    checkbox_no_reference_table_nor_column_found_no_pak: QBox<QCheckBox>,
    checkbox_invalid_escape: QBox<QCheckBox>,
    checkbox_duplicated_row: QBox<QCheckBox>,
    checkbox_invalid_dependency_packfile: QBox<QCheckBox>,
    checkbox_invalid_loc_key: QBox<QCheckBox>,
    checkbox_dependencies_cache_not_generated: QBox<QCheckBox>,
    checkbox_invalid_packfile_name: QBox<QCheckBox>,
    checkbox_table_name_ends_in_number: QBox<QCheckBox>,
    checkbox_table_name_has_space: QBox<QCheckBox>,
    checkbox_table_is_datacoring: QBox<QCheckBox>,
    checkbox_dependencies_cache_outdated: QBox<QCheckBox>,
    checkbox_dependencies_cache_could_not_be_loaded: QBox<QCheckBox>,
    checkbox_field_with_path_not_found: QBox<QCheckBox>,
    checkbox_incorrect_game_path: QBox<QCheckBox>,
    checkbox_banned_table: QBox<QCheckBox>,
    checkbox_value_cannot_be_empty: QBox<QCheckBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUI`.
impl DiagnosticsUI {

    /// This function creates an entire `DiagnosticsUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Self {

        //-----------------------------------------------//
        // `DiagnosticsUI` DockWidget.
        //-----------------------------------------------//
        let diagnostics_dock_widget = QDockWidget::from_q_widget(main_window);
        let diagnostics_dock_inner_widget = QWidget::new_1a(&diagnostics_dock_widget);
        let diagnostics_dock_layout = create_grid_layout(diagnostics_dock_inner_widget.static_upcast());
        diagnostics_dock_widget.set_widget(&diagnostics_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::BottomDockWidgetArea, diagnostics_dock_widget.as_ptr());
        diagnostics_dock_widget.set_window_title(&qtr("gen_loc_diagnostics"));
        diagnostics_dock_widget.set_object_name(&QString::from_std_str("diagnostics_dock"));

        // Create and configure the filters section.
        let filter_frame = QGroupBox::from_q_widget(&diagnostics_dock_inner_widget);
        let filter_grid = create_grid_layout(filter_frame.static_upcast());
        filter_grid.set_contents_margins_4a(4, 0, 4, 0);

        let diagnostics_button_check_packfile = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_check_packfile"), &filter_frame);
        let diagnostics_button_check_current_packed_file = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_check_current_packed_file"), &filter_frame);
        let diagnostics_button_error = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_error"), &filter_frame);
        let diagnostics_button_warning = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_warning"), &filter_frame);
        let diagnostics_button_info = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_info"), &filter_frame);
        let diagnostics_button_only_current_packed_file = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_only_current_packed_file"), &filter_frame);
        let diagnostics_button_show_more_filters = QPushButton::from_q_string_q_widget(&qtr("diagnostics_button_show_more_filters"), &filter_frame);

        let line_separator = QFrame::new_1a(&filter_frame);
        line_separator.set_frame_shape(Shape::VLine);
        line_separator.set_frame_shadow(Shadow::Sunken);

        diagnostics_button_error.set_checkable(true);
        diagnostics_button_warning.set_checkable(true);
        diagnostics_button_info.set_checkable(true);
        diagnostics_button_only_current_packed_file.set_checkable(true);
        diagnostics_button_show_more_filters.set_checkable(true);
        diagnostics_button_error.set_checked(true);

        diagnostics_button_info.set_style_sheet(&QString::from_std_str(&format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_info(), get_color_info_pressed())));

        diagnostics_button_warning.set_style_sheet(&QString::from_std_str(&format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_warning(), get_color_warning_pressed())));

        diagnostics_button_error.set_style_sheet(&QString::from_std_str(&format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_error(), get_color_error_pressed())));

        filter_grid.add_widget_5a(&diagnostics_button_check_packfile, 0, 0, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_check_current_packed_file, 0, 1, 1, 1);
        filter_grid.add_widget_5a(&line_separator, 0, 2, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_error, 0, 3, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_warning, 0, 4, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_info, 0, 5, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_only_current_packed_file, 0, 6, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_show_more_filters, 0, 7, 1, 1);

        let diagnostics_table_view = QTableView::new_1a(&diagnostics_dock_inner_widget);
        let diagnostics_table_filter = new_tableview_filter_safe(diagnostics_dock_inner_widget.static_upcast());
        let diagnostics_table_model = QStandardItemModel::new_1a(&diagnostics_dock_inner_widget);
        diagnostics_table_filter.set_source_model(&diagnostics_table_model);
        diagnostics_table_view.set_model(&diagnostics_table_filter);
        diagnostics_table_view.set_selection_mode(SelectionMode::ExtendedSelection);
        diagnostics_table_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

        if SETTINGS.read().unwrap().settings_bool["tight_table_mode"] {
            diagnostics_table_view.vertical_header().set_minimum_section_size(22);
            diagnostics_table_view.vertical_header().set_maximum_section_size(22);
            diagnostics_table_view.vertical_header().set_default_section_size(22);
        }

        diagnostics_dock_layout.add_widget_5a(&filter_frame, 0, 0, 1, 1);
        diagnostics_dock_layout.add_widget_5a(&diagnostics_table_view, 1, 0, 1, 1);

        main_window.set_corner(qt_core::Corner::BottomLeftCorner, qt_core::DockWidgetArea::LeftDockWidgetArea);
        main_window.set_corner(qt_core::Corner::BottomRightCorner, qt_core::DockWidgetArea::RightDockWidgetArea);

        //-------------------------------------------------------------------------------//
        // Sidebar section.
        //-------------------------------------------------------------------------------//

        // Create the search and hide/show/freeze widgets.
        let sidebar_scroll_area = QScrollArea::new_1a(&diagnostics_dock_inner_widget);
        let sidebar_widget = QWidget::new_1a(&sidebar_scroll_area);
        let sidebar_grid = create_grid_layout(sidebar_widget.static_upcast());
        sidebar_scroll_area.set_widget(&sidebar_widget);
        sidebar_scroll_area.set_widget_resizable(true);
        sidebar_scroll_area.horizontal_scroll_bar().set_enabled(false);
        sidebar_grid.set_contents_margins_4a(4, 0, 4, 4);
        sidebar_grid.set_spacing(4);

        let header_column = QLabel::from_q_string_q_widget(&qtr("diagnostic_type"), &sidebar_scroll_area);
        let header_hidden = QLabel::from_q_string_q_widget(&qtr("diagnostic_show"), &sidebar_scroll_area);

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_column, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_hidden, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(&header_column, 0, 0, 1, 1);
        sidebar_grid.add_widget_5a(&header_hidden, 0, 1, 1, 1);

        let label_all = QLabel::from_q_string_q_widget(&qtr("all"), &sidebar_scroll_area);
        let label_outdated_table = QLabel::from_q_string_q_widget(&qtr("label_outdated_table"), &sidebar_scroll_area);
        let label_invalid_reference = QLabel::from_q_string_q_widget(&qtr("label_invalid_reference"), &sidebar_scroll_area);
        let label_empty_row = QLabel::from_q_string_q_widget(&qtr("label_empty_row"), &sidebar_scroll_area);
        let label_empty_key_field = QLabel::from_q_string_q_widget(&qtr("label_empty_key_field"), &sidebar_scroll_area);
        let label_empty_key_fields = QLabel::from_q_string_q_widget(&qtr("label_empty_key_fields"), &sidebar_scroll_area);
        let label_duplicated_combined_keys = QLabel::from_q_string_q_widget(&qtr("label_duplicated_combined_keys"), &sidebar_scroll_area);
        let label_no_reference_table_found = QLabel::from_q_string_q_widget(&qtr("label_no_reference_table_found"), &sidebar_scroll_area);
        let label_no_reference_table_nor_column_found_pak = QLabel::from_q_string_q_widget(&qtr("label_no_reference_table_nor_column_found_pak"), &sidebar_scroll_area);
        let label_no_reference_table_nor_column_found_no_pak = QLabel::from_q_string_q_widget(&qtr("label_no_reference_table_nor_column_found_no_pak"), &sidebar_scroll_area);
        let label_invalid_escape = QLabel::from_q_string_q_widget(&qtr("label_invalid_escape"), &sidebar_scroll_area);
        let label_duplicated_row = QLabel::from_q_string_q_widget(&qtr("label_duplicated_row"), &sidebar_scroll_area);
        let label_invalid_dependency_packfile = QLabel::from_q_string_q_widget(&qtr("label_invalid_dependency_packfile"), &sidebar_scroll_area);
        let label_invalid_loc_key = QLabel::from_q_string_q_widget(&qtr("label_invalid_loc_key"), &sidebar_scroll_area);
        let label_dependencies_cache_not_generated = QLabel::from_q_string_q_widget(&qtr("label_dependencies_cache_not_generated"), &sidebar_scroll_area);
        let label_invalid_packfile_name = QLabel::from_q_string_q_widget(&qtr("label_invalid_packfile_name"), &sidebar_scroll_area);
        let label_table_name_ends_in_number = QLabel::from_q_string_q_widget(&qtr("label_table_name_ends_in_number"), &sidebar_scroll_area);
        let label_table_name_has_space = QLabel::from_q_string_q_widget(&qtr("label_table_name_has_space"), &sidebar_scroll_area);
        let label_table_is_datacoring = QLabel::from_q_string_q_widget(&qtr("label_table_is_datacoring"), &sidebar_scroll_area);
        let label_dependencies_cache_outdated = QLabel::from_q_string_q_widget(&qtr("label_dependencies_cache_outdated"), &sidebar_scroll_area);
        let label_dependencies_cache_could_not_be_loaded = QLabel::from_q_string_q_widget(&qtr("label_dependencies_cache_could_not_be_loaded"), &sidebar_scroll_area);
        let label_field_with_path_not_found = QLabel::from_q_string_q_widget(&qtr("label_field_with_path_not_found"), &sidebar_scroll_area);
        let label_incorrect_game_path = QLabel::from_q_string_q_widget(&qtr("label_incorrect_game_path"), &sidebar_scroll_area);
        let label_banned_table = QLabel::from_q_string_q_widget(&qtr("label_banned_table"), &sidebar_scroll_area);
        let label_value_cannot_be_empty = QLabel::from_q_string_q_widget(&qtr("label_value_cannot_be_empty"), &sidebar_scroll_area);

        let checkbox_all = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_outdated_table = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_invalid_reference = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_empty_row = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_empty_key_field = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_empty_key_fields = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_duplicated_combined_keys = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_no_reference_table_found = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_no_reference_table_nor_column_found_pak = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_no_reference_table_nor_column_found_no_pak = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_invalid_escape = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_duplicated_row = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_invalid_dependency_packfile = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_invalid_loc_key = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_dependencies_cache_not_generated = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_invalid_packfile_name = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_table_name_ends_in_number = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_table_name_has_space = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_table_is_datacoring = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_dependencies_cache_outdated = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_dependencies_cache_could_not_be_loaded = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_field_with_path_not_found = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_incorrect_game_path = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_banned_table = QCheckBox::from_q_widget(&sidebar_scroll_area);
        let checkbox_value_cannot_be_empty = QCheckBox::from_q_widget(&sidebar_scroll_area);

        checkbox_all.set_checked(true);
        checkbox_outdated_table.set_checked(true);
        checkbox_invalid_reference.set_checked(true);
        checkbox_empty_row.set_checked(true);
        checkbox_empty_key_field.set_checked(true);
        checkbox_empty_key_fields.set_checked(true);
        checkbox_duplicated_combined_keys.set_checked(true);
        checkbox_no_reference_table_found.set_checked(true);
        checkbox_no_reference_table_nor_column_found_pak.set_checked(true);
        checkbox_no_reference_table_nor_column_found_no_pak.set_checked(true);
        checkbox_invalid_escape.set_checked(true);
        checkbox_duplicated_row.set_checked(true);
        checkbox_invalid_dependency_packfile.set_checked(true);
        checkbox_invalid_loc_key.set_checked(true);
        checkbox_dependencies_cache_not_generated.set_checked(true);
        checkbox_invalid_packfile_name.set_checked(true);
        checkbox_table_name_ends_in_number.set_checked(true);
        checkbox_table_name_has_space.set_checked(true);
        checkbox_table_is_datacoring.set_checked(true);
        checkbox_dependencies_cache_outdated.set_checked(true);
        checkbox_dependencies_cache_could_not_be_loaded.set_checked(true);
        checkbox_field_with_path_not_found.set_checked(true);
        checkbox_incorrect_game_path.set_checked(true);
        checkbox_banned_table.set_checked(true);
        checkbox_value_cannot_be_empty.set_checked(true);

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_all, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_outdated_table, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_invalid_reference, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_empty_row, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_empty_key_field, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_empty_key_fields, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_duplicated_combined_keys, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_no_reference_table_found, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_no_reference_table_nor_column_found_pak, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_no_reference_table_nor_column_found_no_pak, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_invalid_escape, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_duplicated_row, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_invalid_dependency_packfile, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_invalid_loc_key, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_dependencies_cache_not_generated, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_invalid_packfile_name, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_table_name_ends_in_number, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_table_name_has_space, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_table_is_datacoring, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_dependencies_cache_outdated, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_dependencies_cache_could_not_be_loaded, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_field_with_path_not_found, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_incorrect_game_path, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_banned_table, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&checkbox_value_cannot_be_empty, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(&label_all, 1, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_outdated_table, 2, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_invalid_reference, 3, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_empty_row, 4, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_empty_key_field, 5, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_empty_key_fields, 6, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_duplicated_combined_keys, 7, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_no_reference_table_found, 8, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_no_reference_table_nor_column_found_pak, 9, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_no_reference_table_nor_column_found_no_pak, 10, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_invalid_escape, 11, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_duplicated_row, 12, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_invalid_dependency_packfile, 13, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_invalid_loc_key, 14, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_dependencies_cache_not_generated, 15, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_invalid_packfile_name, 16, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_table_name_ends_in_number, 17, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_table_name_has_space, 18, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_table_is_datacoring, 19, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_dependencies_cache_outdated, 20, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_dependencies_cache_could_not_be_loaded, 21, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_field_with_path_not_found, 22, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_incorrect_game_path, 23, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_banned_table, 24, 0, 1, 1);
        sidebar_grid.add_widget_5a(&label_value_cannot_be_empty, 25, 0, 1, 1);

        sidebar_grid.add_widget_5a(&checkbox_all, 1, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_outdated_table, 2, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_invalid_reference, 3, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_empty_row, 4, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_empty_key_field, 5, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_empty_key_fields, 6, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_duplicated_combined_keys, 7, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_no_reference_table_found, 8, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_no_reference_table_nor_column_found_pak, 9, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_no_reference_table_nor_column_found_no_pak, 10, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_invalid_escape, 11, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_duplicated_row, 12, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_invalid_dependency_packfile, 13, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_invalid_loc_key, 14, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_dependencies_cache_not_generated, 15, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_invalid_packfile_name, 16, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_table_name_ends_in_number, 17, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_table_name_has_space, 18, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_table_is_datacoring, 19, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_dependencies_cache_outdated, 20, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_dependencies_cache_could_not_be_loaded, 21, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_field_with_path_not_found, 22, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_incorrect_game_path, 23, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_banned_table, 24, 1, 1, 1);
        sidebar_grid.add_widget_5a(&checkbox_value_cannot_be_empty, 25, 1, 1, 1);

        // Add all the stuff to the main grid and hide the search widget.
        diagnostics_dock_layout.add_widget_5a(&sidebar_scroll_area, 0, 1, 2, 1);
        diagnostics_dock_layout.set_column_stretch(0, 10);
        sidebar_scroll_area.hide();

        Self {

            //-------------------------------------------------------------------------------//
            // `Diagnostics` Dock Widget.
            //-------------------------------------------------------------------------------//
            diagnostics_dock_widget,
            diagnostics_table_view,
            diagnostics_table_filter,
            diagnostics_table_model,

            //-------------------------------------------------------------------------------//
            // Filters section.
            //-------------------------------------------------------------------------------//
            diagnostics_button_check_packfile,
            diagnostics_button_check_current_packed_file,
            diagnostics_button_error,
            diagnostics_button_warning,
            diagnostics_button_info,
            diagnostics_button_only_current_packed_file,
            diagnostics_button_show_more_filters,

            sidebar_scroll_area,
            checkbox_all,
            checkbox_outdated_table,
            checkbox_invalid_reference,
            checkbox_empty_row,
            checkbox_empty_key_field,
            checkbox_empty_key_fields,
            checkbox_duplicated_combined_keys,
            checkbox_no_reference_table_found,
            checkbox_no_reference_table_nor_column_found_pak,
            checkbox_no_reference_table_nor_column_found_no_pak,
            checkbox_invalid_escape,
            checkbox_duplicated_row,
            checkbox_invalid_dependency_packfile,
            checkbox_invalid_loc_key,
            checkbox_dependencies_cache_not_generated,
            checkbox_invalid_packfile_name,
            checkbox_table_name_ends_in_number,
            checkbox_table_name_has_space,
            checkbox_table_is_datacoring,
            checkbox_dependencies_cache_outdated,
            checkbox_dependencies_cache_could_not_be_loaded,
            checkbox_field_with_path_not_found,
            checkbox_incorrect_game_path,
            checkbox_banned_table,
            checkbox_value_cannot_be_empty
        }
    }

    /// This function takes care of checking the entire PackFile for errors.
    pub unsafe fn check(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>) {
        app_ui.menu_bar_packfile.set_enabled(false);

        let receiver = CENTRAL_COMMAND.send_background(Command::DiagnosticsCheck);
        diagnostics_ui.diagnostics_table_model.clear();
        let response = CentralCommand::recv_try(&receiver);
        match response {
            Response::Diagnostics(diagnostics) => {
                Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.get_ref_diagnostics());
                Self::filter(app_ui, diagnostics_ui);
                Self::update_level_counts(diagnostics_ui, diagnostics.get_ref_diagnostics());
                UI_STATE.set_diagnostics(&diagnostics);
            }
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        app_ui.menu_bar_packfile.set_enabled(true);
    }

    /// This function takes care of updating the results of a diagnostics check for the provided paths.
    pub unsafe fn check_on_path(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, diagnostics_ui: &Rc<Self>, paths: Vec<PathType>) {
        let diagnostics = UI_STATE.get_diagnostics();
        let receiver = CENTRAL_COMMAND.send_background(Command::DiagnosticsUpdate((diagnostics, paths)));
        let response = CentralCommand::recv_try(&receiver);
        match response {
            Response::DiagnosticsVecPackedFileInfo(diagnostics, packed_files_info) => {
                diagnostics_ui.diagnostics_table_model.clear();
                Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.get_ref_diagnostics());
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);

                Self::filter(app_ui, diagnostics_ui);
                Self::update_level_counts(diagnostics_ui, diagnostics.get_ref_diagnostics());
                UI_STATE.set_diagnostics(&diagnostics);
            }
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }

    /// This function takes care of loading the results of a diagnostic check into the table.
    unsafe fn load_diagnostics_to_ui(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {

        // First, clean the current diagnostics.
        Self::clean_diagnostics_from_views(app_ui);

        if !diagnostics.is_empty() {
            let blocker = QSignalBlocker::from_q_object(&diagnostics_ui.diagnostics_table_model);
            for (index, diagnostic_type) in diagnostics.iter().enumerate() {

                // Unlock in the last step.
                if index == diagnostics.len() - 1 {
                    blocker.unblock();
                }

                match diagnostic_type {
                    DiagnosticType::AnimFragment(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let cells_affected = QStandardItem::new();
                            let path = QStandardItem::new();
                            let message = QStandardItem::new();
                            let report_type = QStandardItem::new();
                            let (result_type, color) = match result.level {
                                DiagnosticLevel::Info => ("Info".to_owned(), get_color_info()),
                                DiagnosticLevel::Warning => ("Warning".to_owned(), get_color_warning()),
                                DiagnosticLevel::Error => ("Error".to_owned(), get_color_error()),
                            };

                            level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                            level.set_text(&QString::from_std_str(result_type));
                            diag_type.set_text(&QString::from_std_str(&format!("{}", diagnostic_type)));
                            cells_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.cells_affected).unwrap())), 2);
                            path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            cells_affected.set_editable(false);
                            path.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Set the tooltips to the diag type and description columns.
                            Self::set_tooltips_anim_fragment(&[&level, &path, &message], &result.report_type);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&cells_affected.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.into_ptr().as_ref().unwrap());
                        }
                    }
                    DiagnosticType::DB(ref diagnostic) |
                    DiagnosticType::Loc(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let cells_affected = QStandardItem::new();
                            let path = QStandardItem::new();
                            let message = QStandardItem::new();
                            let report_type = QStandardItem::new();
                            let (result_type, color) = match result.level {
                                DiagnosticLevel::Info => ("Info".to_owned(), get_color_info()),
                                DiagnosticLevel::Warning => ("Warning".to_owned(), get_color_warning()),
                                DiagnosticLevel::Error => ("Error".to_owned(), get_color_error()),
                            };

                            level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                            level.set_text(&QString::from_std_str(result_type));
                            diag_type.set_text(&QString::from_std_str(&format!("{}", diagnostic_type)));
                            cells_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.cells_affected).unwrap())), 2);
                            path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            cells_affected.set_editable(false);
                            path.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Set the tooltips to the diag type and description columns.
                            Self::set_tooltips_table(&[&level, &path, &message], &result.report_type);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&cells_affected.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.into_ptr().as_ref().unwrap());
                        }
                    }

                    DiagnosticType::PackFile(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let fill1 = QStandardItem::new();
                            let fill2 = QStandardItem::new();
                            let message = QStandardItem::new();
                            let report_type = QStandardItem::new();
                            let (result_type, color) = match result.level {
                                DiagnosticLevel::Info => ("Info".to_owned(), get_color_info()),
                                DiagnosticLevel::Warning => ("Warning".to_owned(), get_color_warning()),
                                DiagnosticLevel::Error => ("Error".to_owned(), get_color_error()),
                            };

                            level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                            level.set_text(&QString::from_std_str(result_type));
                            diag_type.set_text(&QString::from_std_str(&format!("{}", diagnostic_type)));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            fill1.set_editable(false);
                            fill2.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Set the tooltips to the diag type and description columns.
                            Self::set_tooltips_packfile(&[&level, &fill2, &message], &result.report_type);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                        }
                    }
                    DiagnosticType::DependencyManager(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let cells_affected = QStandardItem::new();
                            let path = QStandardItem::new();
                            let message = QStandardItem::new();
                            let report_type = QStandardItem::new();
                            let (result_type, color) = match result.level {
                                DiagnosticLevel::Info => ("Info".to_owned(), get_color_info()),
                                DiagnosticLevel::Warning => ("Warning".to_owned(), get_color_warning()),
                                DiagnosticLevel::Error => ("Error".to_owned(), get_color_error()),
                            };

                            level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                            level.set_text(&QString::from_std_str(result_type));
                            diag_type.set_text(&QString::from_std_str(&format!("{}", diagnostic_type)));
                            cells_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.cells_affected).unwrap())), 2);
                            path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            cells_affected.set_editable(false);
                            path.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Set the tooltips to the diag type and description columns.
                            Self::set_tooltips_dependency_manager(&[&level, &path, &message], &result.report_type);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&cells_affected.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                        }
                    }

                    DiagnosticType::Config(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let fill1 = QStandardItem::new();
                            let fill2 = QStandardItem::new();
                            let message = QStandardItem::new();
                            let report_type = QStandardItem::new();
                            let (result_type, color) = match result.level {
                                DiagnosticLevel::Info => ("Info".to_owned(), get_color_info()),
                                DiagnosticLevel::Warning => ("Warning".to_owned(), get_color_warning()),
                                DiagnosticLevel::Error => ("Error".to_owned(), get_color_error()),
                            };

                            level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                            level.set_text(&QString::from_std_str(result_type));
                            diag_type.set_text(&QString::from_std_str(&format!("{}", diagnostic_type)));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            fill1.set_editable(false);
                            fill2.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Set the tooltips to the diag type and description columns.
                            Self::set_tooltips_config(&[&level, &fill2, &message], &result.report_type);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                        }
                    }
                }

                // After that, check if the table is open, and paint the results into it.
                Self::paint_diagnostics_to_table(app_ui, diagnostic_type);
            }

            diagnostics_ui.diagnostics_table_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_level")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_diag")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_cells_affected")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_path")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_message")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(5, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_report_type")));

            // Hide the column number column for tables.
            diagnostics_ui.diagnostics_table_view.hide_column(1);
            diagnostics_ui.diagnostics_table_view.hide_column(2);
            diagnostics_ui.diagnostics_table_view.hide_column(5);
            diagnostics_ui.diagnostics_table_view.sort_by_column_2a(3, SortOrder::AscendingOrder);

            diagnostics_ui.diagnostics_table_view.horizontal_header().set_stretch_last_section(true);
            diagnostics_ui.diagnostics_table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    pub unsafe fn open_match(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<Self>,
        dependencies_ui: &Rc<DependenciesUI>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let item_path = model.item_2a(model_index.row(), 3);
        let path = item_path.text().to_std_string();
        let path: Vec<String> = if path.is_empty() { vec![] } else { path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect() };
        let tree_index = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path, DataSource::PackFile);

        let diagnostic_type = model.item_2a(model_index.row(), 1).text().to_std_string();
        if diagnostic_type == "DependencyManager" {
            AppUI::open_dependency_manager(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui);
        } else if !path.is_empty() {

            // Manually select the open PackedFile, then open it. This means we can open PackedFiles nor in out filter.
            UI_STATE.set_packfile_contents_read_only(true);

            if let Some(ref tree_index) = tree_index {
                if tree_index.is_valid() {
                    pack_file_contents_ui.packfile_contents_tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                    pack_file_contents_ui.packfile_contents_tree_view.selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                }
            }

            UI_STATE.set_packfile_contents_read_only(false);
            AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, Some(path.to_vec()), false, false, DataSource::PackFile);
        }

        // If it's a table, focus on the matched cell.
        match &*model.item_2a(model_index.row(), 1).text().to_std_string() {
            "AnimFragment" => {

                if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == path) {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    if let ViewType::Internal(View::AnimFragment(view)) = packed_file_view.get_view() {
                        let table_view = view.get_ref_table_view_2();
                        let table_view = table_view.get_mut_ptr_table_view_primary();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        table_selection_model.clear_selection();
                        let cells_affected: Vec<(i32, i32)> = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        for (row, column) in cells_affected {
                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.set_focus_0a();
                                table_view.set_current_index(table_model_index_filtered.as_ref());
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                            }
                        }
                    }
                }
            }

            "DB" | "Loc" | "DependencyManager" => {

                if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|x| *x.get_ref_path() == path) {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    if let ViewType::Internal(View::Table(view)) = packed_file_view.get_view() {
                        let table_view = view.get_ref_table();
                        let table_view = table_view.get_mut_ptr_table_view_primary();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        table_selection_model.clear_selection();
                        let cells_affected: Vec<(i32, i32)> = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        for (row, column) in cells_affected {
                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.set_focus_0a();
                                table_view.set_current_index(table_model_index_filtered.as_ref());
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                            }
                        }
                    } else if let ViewType::Internal(View::DependenciesManager(view)) = packed_file_view.get_view() {

                        let table_view = view.get_ref_table();
                        let table_view = table_view.get_mut_ptr_table_view_primary();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        table_selection_model.clear_selection();
                        let cells_affected: Vec<(i32, i32)> = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        for (row, column) in cells_affected {
                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.set_focus_0a();
                                table_view.set_current_index(table_model_index_filtered.as_ref());
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                            }
                        }
                    }
                }
            }

            // Config matches have to open their relevant config issue.
            "Config" => {
                match &*model.item_2a(model_index.row(), 5).text().to_std_string() {

                    // For these, we will trigger the action to generate the dependencies cache.
                    "DependenciesCacheNotGenerated" |
                    "DependenciesCacheOutdated" |
                    "DependenciesCacheCouldNotBeLoaded" => {
                        match &*GAME_SELECTED.read().unwrap().get_game_key_name() {
                            KEY_WARHAMMER_3 => app_ui.special_stuff_wh3_generate_dependencies_cache.trigger(),
                            KEY_TROY => app_ui.special_stuff_troy_generate_dependencies_cache.trigger(),
                            KEY_THREE_KINGDOMS => app_ui.special_stuff_three_k_generate_dependencies_cache.trigger(),
                            KEY_WARHAMMER_2 => app_ui.special_stuff_wh2_generate_dependencies_cache.trigger(),
                            KEY_WARHAMMER => app_ui.special_stuff_wh_generate_dependencies_cache.trigger(),
                            KEY_THRONES_OF_BRITANNIA => app_ui.special_stuff_tob_generate_dependencies_cache.trigger(),
                            KEY_ATTILA => app_ui.special_stuff_att_generate_dependencies_cache.trigger(),
                            KEY_ROME_2 => app_ui.special_stuff_rom2_generate_dependencies_cache.trigger(),
                            KEY_SHOGUN_2 => app_ui.special_stuff_sho2_generate_dependencies_cache.trigger(),
                            KEY_NAPOLEON => app_ui.special_stuff_nap_generate_dependencies_cache.trigger(),
                            KEY_EMPIRE => app_ui.special_stuff_emp_generate_dependencies_cache.trigger(),
                            _ => {}
                        }
                    }
                    "IncorrectGamePath" => app_ui.packfile_preferences.trigger(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// This function tries to paint the results from the provided diagnostics into their file view, if the file is open.
    pub unsafe fn paint_diagnostics_to_table(
        app_ui: &Rc<AppUI>,
        diagnostic: &DiagnosticType,
    ) {

        let path = match diagnostic {
            DiagnosticType::AnimFragment(ref diagnostic) => diagnostic.get_path(),
            DiagnosticType::DB(ref diagnostic) |
            DiagnosticType::Loc(ref diagnostic) => diagnostic.get_path(),
            DiagnosticType::DependencyManager(ref diagnostic) => diagnostic.get_path(),
            _ => return,
        };

        if let Some(view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).find(|view| view.get_path() == path) {
            if app_ui.tab_bar_packed_file.index_of(view.get_mut_widget()) != -1 {

                // In case of tables, we have to get the logical row/column of the match and select it.
                let internal_table_view = if let ViewType::Internal(View::Table(view)) = view.get_view() { view.get_ref_table() }
                else if let ViewType::Internal(View::DependenciesManager(view)) = view.get_view() { view.get_ref_table() }
                else if let ViewType::Internal(View::AnimFragment(view)) = view.get_view() { view.get_ref_table_view_2() }
                else { return };

                let table_view = internal_table_view.get_mut_ptr_table_view_primary();
                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                match diagnostic {
                    DiagnosticType::DB(ref diagnostic) |
                    DiagnosticType::Loc(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            for (row, column) in &result.cells_affected {
                                if *row != -1 || *column != -1 {
                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row as i32, column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row as i32, *column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row as i32, *column as i32);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    DiagnosticType::DependencyManager(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            for (row, column) in &result.cells_affected {
                                if *row != -1 || *column != -1 {

                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row as i32, column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row as i32, *column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row as i32, *column as i32);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    DiagnosticType::AnimFragment(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            for (row, column) in &result.cells_affected {
                                if *row != -1 || *column != -1 {
                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row as i32, column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row as i32, *column as i32);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row as i32, *column as i32);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => return,
                }

                // Unblock the model and update it. Otherwise, the painted cells wont show up until something else updates the view.
                blocker.unblock();
                table_view.viewport().repaint();
            }
        }
    }

    pub unsafe fn clean_diagnostics_from_views(app_ui: &Rc<AppUI>) {
        for view in UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile) {

            // Only update the visible tables.
            if app_ui.tab_bar_packed_file.index_of(view.get_mut_widget()) != -1 {

                // In case of tables, we have to get the logical row/column of the match and select it.
                if let ViewType::Internal(View::Table(view)) = view.get_view() {
                    let table_view = view.get_ref_table().get_mut_ptr_table_view_primary();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);

                            if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                            }
                            if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                            }
                            if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                } else if let ViewType::Internal(View::AnimFragment(view)) = view.get_view() {
                    let table_view = view.get_ref_table_view_2().get_mut_ptr_table_view_primary();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);

                            if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                            }
                            if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                            }
                            if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                }

                else if let ViewType::Internal(View::DependenciesManager(view)) = view.get_view() {
                    let table_view = view.get_ref_table().get_mut_ptr_table_view_primary();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);

                            if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                            }
                            if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                            }
                            if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                }
            }
        }
    }

    pub unsafe fn filter(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>) {
        let mut columns = vec![];
        let mut patterns = vec![];
        let mut sensitivity = vec![];

        let info_state = diagnostics_ui.diagnostics_button_info.is_checked();
        let warning_state = diagnostics_ui.diagnostics_button_warning.is_checked();
        let error_state = diagnostics_ui.diagnostics_button_error.is_checked();
        let pattern_level = match (info_state, warning_state, error_state) {
            (true, true, true) => "Info|Warning|Error",
            (true, true, false) => "Info|Warning",
            (true, false, true) => "Info|Error",
            (false, true, true) => "Warning|Error",
            (true, false, false) => "Info",
            (false, false, true) => "Error",
            (false, true, false) => "Warning",
            (false, false, false) => "-1",
        };

        columns.push(0);
        patterns.push(QString::from_std_str(pattern_level).into_ptr());
        sensitivity.push(CaseSensitivity::CaseSensitive);

        // Check for currently open files filter.
        if diagnostics_ui.diagnostics_button_only_current_packed_file.is_checked() {
            let open_packedfiles = UI_STATE.get_open_packedfiles();
            let open_packedfiles_ref = open_packedfiles.iter()
                .filter(|x| x.get_data_source() == DataSource::PackFile && app_ui.tab_bar_packed_file.index_of(x.get_mut_widget()) != -1)
                .collect::<Vec<&PackedFileView>>();
            let mut pattern = String::new();
            for open_packedfile in &open_packedfiles_ref {
                if !pattern.is_empty() {
                    pattern.push('|');
                }
                pattern.push_str(&open_packedfile.get_ref_path().join("/"));
            }

            // This makes sure the check works even if we don't have anything open.
            if pattern.is_empty() {
                pattern.push_str("empty");
            }

            columns.push(3);
            patterns.push(QString::from_std_str(pattern).into_ptr());
            sensitivity.push(CaseSensitivity::CaseSensitive);
        }

        // Checks for the diagnostic type filter.
        let mut diagnostic_type_pattern = String::new();

        if diagnostics_ui.checkbox_outdated_table.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::OutdatedTable));
        }
        if diagnostics_ui.checkbox_invalid_reference.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidReference));
        }
        if diagnostics_ui.checkbox_empty_row.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyRow));
        }
        if diagnostics_ui.checkbox_empty_key_field.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyKeyField));
        }
        if diagnostics_ui.checkbox_empty_key_fields.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyKeyFields));
        }
        if diagnostics_ui.checkbox_duplicated_combined_keys.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::DuplicatedCombinedKeys));
        }
        if diagnostics_ui.checkbox_no_reference_table_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableFound));
        }
        if diagnostics_ui.checkbox_no_reference_table_nor_column_found_pak.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak));
        }
        if diagnostics_ui.checkbox_no_reference_table_nor_column_found_no_pak.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak));
        }
        if diagnostics_ui.checkbox_invalid_escape.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidEscape));
        }
        if diagnostics_ui.checkbox_duplicated_row.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::DuplicatedRow));
        }
        if diagnostics_ui.checkbox_invalid_loc_key.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidLocKey));
        }
        if diagnostics_ui.checkbox_table_name_ends_in_number.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableNameEndsInNumber));
        }
        if diagnostics_ui.checkbox_table_name_has_space.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableNameHasSpace));
        }
        if diagnostics_ui.checkbox_table_is_datacoring.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableIsDataCoring));
        }
        if diagnostics_ui.checkbox_field_with_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::FieldWithPathNotFound));
        }
        if diagnostics_ui.checkbox_banned_table.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::BannedTable));
        }
        if diagnostics_ui.checkbox_value_cannot_be_empty.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::ValueCannotBeEmpty));
        }


        if diagnostics_ui.checkbox_invalid_dependency_packfile.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", DependencyManagerDiagnosticReportType::InvalidDependencyPackFileName));
        }

        if diagnostics_ui.checkbox_dependencies_cache_not_generated.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheNotGenerated));
        }
        if diagnostics_ui.checkbox_dependencies_cache_outdated.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheOutdated));
        }
        if diagnostics_ui.checkbox_dependencies_cache_could_not_be_loaded.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded("".to_owned())));
        }
        if diagnostics_ui.checkbox_incorrect_game_path.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::IncorrectGamePath));
        }

        if diagnostics_ui.checkbox_invalid_packfile_name.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PackFileDiagnosticReportType::InvalidPackFileName));
        }

        diagnostic_type_pattern.pop();

        if diagnostic_type_pattern.is_empty() {
            diagnostic_type_pattern.push_str("empty");
        }

        columns.push(5);
        patterns.push(QString::from_std_str(diagnostic_type_pattern).into_ptr());
        sensitivity.push(CaseSensitivity::CaseSensitive);
        let show_blank_lines = vec![false; sensitivity.len()];
        let match_groups = vec![0; sensitivity.len()];

        // Filter whatever it's in that column by the text we got.
        trigger_tableview_filter_safe(&diagnostics_ui.diagnostics_table_filter, &columns, patterns, &sensitivity, &show_blank_lines, &match_groups);
    }

    pub unsafe fn update_level_counts(diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {
        let info = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragment(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Info))
                    .count(),
                 DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Info))
                    .count(),
                 DiagnosticType::Config(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Info))
                    .count()
            }).sum::<usize>();

        let warning = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragment(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::Config(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Warning))
                    .count()
            }).sum::<usize>();


        let error = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragment(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::Config(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| matches!(y.level, DiagnosticLevel::Error))
                    .count()
            }).sum::<usize>();

        diagnostics_ui.diagnostics_button_info.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_info"), info)));
        diagnostics_ui.diagnostics_button_warning.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_warning"), warning)));
        diagnostics_ui.diagnostics_button_error.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_error"), error)));
    }

    pub unsafe fn set_tooltips_anim_fragment(items: &[&CppBox<QStandardItem>], report_type: &AnimFragmentDiagnosticReportType) {
        let tool_tip = match report_type {
            AnimFragmentDiagnosticReportType::FieldWithPathNotFound => qtr("field_with_path_not_found_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_table(items: &[&CppBox<QStandardItem>], report_type: &TableDiagnosticReportType) {
        let tool_tip = match report_type {
            TableDiagnosticReportType::OutdatedTable => qtr("outdated_table_explanation"),
            TableDiagnosticReportType::InvalidReference => qtr("invalid_reference_explanation"),
            TableDiagnosticReportType::EmptyRow => qtr("empty_row_explanation"),
            TableDiagnosticReportType::EmptyKeyField => qtr("empty_key_field_explanation"),
            TableDiagnosticReportType::EmptyKeyFields => qtr("empty_key_fields_explanation"),
            TableDiagnosticReportType::DuplicatedCombinedKeys => qtr("duplicated_combined_keys_explanation"),
            TableDiagnosticReportType::NoReferenceTableFound => qtr("no_reference_table_found_explanation"),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak => qtr("no_reference_table_nor_column_found_pak_explanation"),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak => qtr("no_reference_table_nor_column_found_no_pak_explanation"),
            TableDiagnosticReportType::InvalidEscape => qtr("invalid_escape_explanation"),
            TableDiagnosticReportType::DuplicatedRow => qtr("duplicated_row_explanation"),
            TableDiagnosticReportType::InvalidLocKey => qtr("invalid_loc_key_explanation"),
            TableDiagnosticReportType::TableNameEndsInNumber => qtr("table_name_ends_in_number_explanation"),
            TableDiagnosticReportType::TableNameHasSpace => qtr("table_name_has_space_explanation"),
            TableDiagnosticReportType::TableIsDataCoring => qtr("table_is_datacoring_explanation"),
            TableDiagnosticReportType::FieldWithPathNotFound => qtr("field_with_path_not_found_explanation"),
            TableDiagnosticReportType::BannedTable => qtr("banned_table_explanation"),
            TableDiagnosticReportType::ValueCannotBeEmpty => qtr("value_cannot_be_empty_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_dependency_manager(items: &[&CppBox<QStandardItem>], report_type: &DependencyManagerDiagnosticReportType) {
        let tool_tip = match report_type {
            DependencyManagerDiagnosticReportType::InvalidDependencyPackFileName => qtr("invalid_dependency_pack_file_name_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_config(items: &[&CppBox<QStandardItem>], report_type: &ConfigDiagnosticReportType) {
        let tool_tip = match report_type {
            ConfigDiagnosticReportType::DependenciesCacheNotGenerated => qtr("dependencies_cache_not_generated_explanation"),
            ConfigDiagnosticReportType::DependenciesCacheOutdated => qtr("dependencies_cache_outdated_explanation"),
            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error) => qtre("dependencies_cache_could_not_be_loaded_explanation", &[error]),
            ConfigDiagnosticReportType::IncorrectGamePath => qtr("incorrect_game_path_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_packfile(items: &[&CppBox<QStandardItem>], report_type: &PackFileDiagnosticReportType) {
        let tool_tip = match report_type {
            PackFileDiagnosticReportType::InvalidPackFileName => qtr("invalid_packfile_name_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }
}
