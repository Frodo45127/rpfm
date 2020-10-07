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
Module with all the code related to the `DiagnosticsUI`.
!*/

use qt_widgets::q_abstract_item_view::{ScrollHint, SelectionMode};
use qt_widgets::QCheckBox;
use qt_widgets::QDockWidget;
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
use qt_gui::q_palette::ColorRole;


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

use cpp_core::Ptr;

use std::rc::Rc;

use rpfm_error::ErrorKind;

use rpfm_lib::diagnostics::{*, table::*, dependency_manager::*};
use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use rpfm_macros::{GetRef, GetRefMut, Set};

use crate::AppUI;
use crate::communications::Command;
use crate::CENTRAL_COMMAND;
use crate::ffi::{new_tableview_filter_safe, trigger_tableview_filter_safe};
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, tr};
use crate::pack_tree::{PackTree, get_color_info, get_color_warning, get_color_error, get_color_info_pressed, get_color_warning_pressed, get_color_error_pressed, TreeViewOperation};
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use crate::utils::{create_grid_layout, show_dialog};

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
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUI`.
impl DiagnosticsUI {

    /// This function creates an entire `DiagnosticsUI` struct.
    pub unsafe fn new(main_window: Ptr<QMainWindow>) -> Self {

        //-----------------------------------------------//
        // `DiagnosticsUI` DockWidget.
        //-----------------------------------------------//
        let diagnostics_dock_widget = QDockWidget::from_q_widget(main_window);
        let diagnostics_dock_inner_widget = QWidget::new_0a();
        let diagnostics_dock_layout = create_grid_layout(diagnostics_dock_inner_widget.static_upcast());
        diagnostics_dock_widget.set_widget(&diagnostics_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::BottomDockWidgetArea, diagnostics_dock_widget.as_ptr());
        diagnostics_dock_widget.set_window_title(&qtr("gen_loc_diagnostics"));

        // Create and configure the filters section.
        let filter_frame = QGroupBox::new();
        let filter_grid = create_grid_layout(filter_frame.static_upcast());
        filter_grid.set_contents_margins_4a(4, 0, 4, 0);

        let diagnostics_button_error = QPushButton::from_q_string(&qtr("diagnostics_button_error"));
        let diagnostics_button_warning = QPushButton::from_q_string(&qtr("diagnostics_button_warning"));
        let diagnostics_button_info = QPushButton::from_q_string(&qtr("diagnostics_button_info"));
        let diagnostics_button_only_current_packed_file = QPushButton::from_q_string(&qtr("diagnostics_button_only_current_packed_file"));
        let diagnostics_button_show_more_filters = QPushButton::from_q_string(&qtr("diagnostics_button_show_more_filters"));
        diagnostics_button_error.set_checkable(true);
        diagnostics_button_warning.set_checkable(true);
        diagnostics_button_info.set_checkable(true);
        diagnostics_button_only_current_packed_file.set_checkable(true);
        diagnostics_button_show_more_filters.set_checkable(true);
        diagnostics_button_error.set_checked(true);

        // Hidden until we get this working.
        //diagnostics_button_only_current_packed_file.set_visible(false);

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

        filter_grid.add_widget_5a(&diagnostics_button_error, 0, 0, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_warning, 0, 1, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_info, 0, 2, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_only_current_packed_file, 0, 3, 1, 1);
        filter_grid.add_widget_5a(&diagnostics_button_show_more_filters, 0, 4, 1, 1);

        let diagnostics_table_view = QTableView::new_0a();
        let diagnostics_table_filter = new_tableview_filter_safe(diagnostics_dock_widget.static_upcast());
        let diagnostics_table_model = QStandardItemModel::new_0a();
        diagnostics_table_filter.set_source_model(&diagnostics_table_model);
        diagnostics_table_view.set_model(&diagnostics_table_filter);
        diagnostics_table_view.set_selection_mode(SelectionMode::ExtendedSelection);
        diagnostics_table_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

        if SETTINGS.read().unwrap().settings_bool["tight_table_mode"] {
            diagnostics_table_view.vertical_header().set_minimum_section_size(22);
            diagnostics_table_view.vertical_header().set_maximum_section_size(22);
            diagnostics_table_view.vertical_header().set_default_section_size(22);
        }

        diagnostics_dock_layout.add_widget_5a(filter_frame.into_ptr(), 0, 0, 1, 1);
        diagnostics_dock_layout.add_widget_5a(&diagnostics_table_view, 1, 0, 1, 1);

        main_window.set_corner(qt_core::Corner::BottomLeftCorner, qt_core::DockWidgetArea::LeftDockWidgetArea);
        main_window.set_corner(qt_core::Corner::BottomRightCorner, qt_core::DockWidgetArea::RightDockWidgetArea);

        //-------------------------------------------------------------------------------//
        // Sidebar section.
        //-------------------------------------------------------------------------------//

        // Create the search and hide/show/freeze widgets.
        let sidebar_widget = QWidget::new_0a();
        let sidebar_scroll_area = QScrollArea::new_0a();
        let sidebar_grid = create_grid_layout(sidebar_widget.static_upcast());
        sidebar_scroll_area.set_widget(&sidebar_widget);
        sidebar_scroll_area.set_widget_resizable(true);
        sidebar_scroll_area.horizontal_scroll_bar().set_enabled(false);
        sidebar_grid.set_contents_margins_4a(4, 0, 4, 4);
        sidebar_grid.set_spacing(4);

        let header_column = QLabel::from_q_string(&qtr("diagnostic_type"));
        let header_hidden = QLabel::from_q_string(&qtr("diagnostic_show"));

        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_column, QFlags::from(AlignmentFlag::AlignHCenter));
        sidebar_grid.set_alignment_q_widget_q_flags_alignment_flag(&header_hidden, QFlags::from(AlignmentFlag::AlignHCenter));

        sidebar_grid.add_widget_5a(&header_column, 0, 0, 1, 1);
        sidebar_grid.add_widget_5a(&header_hidden, 0, 1, 1, 1);

        let label_all = QLabel::from_q_string(&qtr("all"));
        let label_outdated_table = QLabel::from_q_string(&qtr("label_outdated_table"));
        let label_invalid_reference = QLabel::from_q_string(&qtr("label_invalid_reference"));
        let label_empty_row = QLabel::from_q_string(&qtr("label_empty_row"));
        let label_empty_key_field = QLabel::from_q_string(&qtr("label_empty_key_field"));
        let label_empty_key_fields = QLabel::from_q_string(&qtr("label_empty_key_fields"));
        let label_duplicated_combined_keys = QLabel::from_q_string(&qtr("label_duplicated_combined_keys"));
        let label_no_reference_table_found = QLabel::from_q_string(&qtr("label_no_reference_table_found"));
        let label_no_reference_table_nor_column_found_pak = QLabel::from_q_string(&qtr("label_no_reference_table_nor_column_found_pak"));
        let label_no_reference_table_nor_column_found_no_pak = QLabel::from_q_string(&qtr("label_no_reference_table_nor_column_found_no_pak"));
        let label_invalid_escape = QLabel::from_q_string(&qtr("label_invalid_escape"));
        let label_duplicated_row = QLabel::from_q_string(&qtr("label_duplicated_row"));
        let label_invalid_dependency_packfile = QLabel::from_q_string(&qtr("label_invalid_dependency_packfile"));

        let checkbox_all = QCheckBox::new();
        let checkbox_outdated_table = QCheckBox::new();
        let checkbox_invalid_reference = QCheckBox::new();
        let checkbox_empty_row = QCheckBox::new();
        let checkbox_empty_key_field = QCheckBox::new();
        let checkbox_empty_key_fields = QCheckBox::new();
        let checkbox_duplicated_combined_keys = QCheckBox::new();
        let checkbox_no_reference_table_found = QCheckBox::new();
        let checkbox_no_reference_table_nor_column_found_pak = QCheckBox::new();
        let checkbox_no_reference_table_nor_column_found_no_pak = QCheckBox::new();
        let checkbox_invalid_escape = QCheckBox::new();
        let checkbox_duplicated_row = QCheckBox::new();
        let checkbox_invalid_dependency_packfile = QCheckBox::new();

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
            checkbox_invalid_dependency_packfile
        }
    }

    /// This function takes care of checking the entire PackFile for errors.
    pub unsafe fn check(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>) {
        if SETTINGS.read().unwrap().settings_bool["enable_diagnostics_tool"] {
            CENTRAL_COMMAND.send_message_qt(Command::DiagnosticsCheck);
            diagnostics_ui.diagnostics_table_model.clear();
            let diagnostics = CENTRAL_COMMAND.recv_message_diagnostics_to_qt_try();
            Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.get_ref_diagnostics());
            Self::filter(app_ui, diagnostics_ui);
            Self::update_level_counts(diagnostics_ui, diagnostics.get_ref_diagnostics());
            UI_STATE.set_diagnostics(&diagnostics);
        }
    }

    /// This function takes care of updating the results of a diagnostics check for the provided paths.
    pub unsafe fn check_on_path(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, diagnostics_ui: &Rc<Self>, paths: Vec<PathType>) {
        if SETTINGS.read().unwrap().settings_bool["enable_diagnostics_tool"] {
            let diagnostics = UI_STATE.get_diagnostics();
            CENTRAL_COMMAND.send_message_qt(Command::DiagnosticsUpdate((diagnostics, paths)));
            let (diagnostics, packed_files_info) = CENTRAL_COMMAND.recv_message_diagnostics_update_to_qt_try();

            diagnostics_ui.diagnostics_table_model.clear();
            Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.get_ref_diagnostics());
            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info));

            Self::filter(app_ui, diagnostics_ui);
            Self::update_level_counts(diagnostics_ui, diagnostics.get_ref_diagnostics());
            UI_STATE.set_diagnostics(&diagnostics);
        }
    }

    /// This function takes care of loading the results of a diagnostic check into the table.
    unsafe fn load_diagnostics_to_ui(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {

        // First, clean the current diagnostics.
        Self::clean_diagnostics_from_views(app_ui);

        if !diagnostics.is_empty() {
            for diagnostic_type in diagnostics {
                match diagnostic_type {
                    DiagnosticType::DB(ref diagnostic) |
                    DiagnosticType::Loc(ref diagnostic) => {
                        for result in diagnostic.get_ref_result() {
                            let qlist_boi = QListOfQStandardItem::new();

                            // Create an empty row.
                            let level = QStandardItem::new();
                            let diag_type = QStandardItem::new();
                            let column = QStandardItem::new();
                            let row = QStandardItem::new();
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
                            column.set_data_2a(&QVariant::from_uint(result.column_number), 2);
                            row.set_data_2a(&QVariant::from_i64(result.row_number + 1), 2);
                            path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            column.set_editable(false);
                            row.set_editable(false);
                            path.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());

                            // Append the new row.
                            diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
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
                            path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&result.message));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            fill1.set_editable(false);
                            fill2.set_editable(false);
                            path.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill1.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill2.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
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
                            let column = QStandardItem::new();
                            let row = QStandardItem::new();
                            let fill3 = QStandardItem::new();
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
                            column.set_data_2a(&QVariant::from_uint(result.column_number), 2);
                            row.set_data_2a(&QVariant::from_i64(result.row_number + 1), 2);
                            message.set_text(&QString::from_std_str(&result.message));
                            report_type.set_text(&QString::from_std_str(&format!("{}", result.report_type)));

                            level.set_editable(false);
                            diag_type.set_editable(false);
                            column.set_editable(false);
                            row.set_editable(false);
                            fill3.set_editable(false);
                            message.set_editable(false);
                            report_type.set_editable(false);

                            // Add an empty row to the list.
                            qlist_boi.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                            qlist_boi.append_q_standard_item(&fill3.into_ptr().as_mut_raw_ptr());
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
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_column")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_column")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_row")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_path")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(5, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_message")));
            diagnostics_ui.diagnostics_table_model.set_header_data_3a(6, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_report_type")));

            // Hide the column number column for tables.
            diagnostics_ui.diagnostics_table_view.hide_column(1);
            diagnostics_ui.diagnostics_table_view.hide_column(2);
            diagnostics_ui.diagnostics_table_view.hide_column(3);
            diagnostics_ui.diagnostics_table_view.hide_column(6);
            diagnostics_ui.diagnostics_table_view.sort_by_column_2a(4, SortOrder::AscendingOrder);

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
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let tree_view = &pack_file_contents_ui.packfile_contents_tree_view;
        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let item_path = model.item_2a(model_index.row(), 4);
        let path = item_path.text().to_std_string();
        let path: Vec<String> = if path.is_empty() { vec![] } else { path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect() };

        // If the path is empty, we're looking for the dependency manager.
        if path.is_empty() {
            AppUI::open_dependency_manager(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui);
        }

        // If not, it's a file we're going to try and open.
        else if let Some(pack_file_contents_model_index) = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path) {
            let pack_file_contents_model_index = pack_file_contents_model_index.as_ref().unwrap();
            let selection_model = tree_view.selection_model();

            // If it's not in the current TreeView Filter we CAN'T OPEN IT.
            //
            // Note: the selection should already trigger the open PackedFile action.
            if pack_file_contents_model_index.is_valid() {
                tree_view.scroll_to_1a(pack_file_contents_model_index);
                selection_model.select_q_model_index_q_flags_selection_flag(pack_file_contents_model_index, QFlags::from(SelectionFlag::ClearAndSelect));
            }
        }

        else {
            show_dialog(app_ui.main_window, ErrorKind::PackedFileNotInFilter, false);
        }

        match &*model.item_2a(model_index.row(), 1).text().to_std_string() {
            "DB" | "Loc" | "DependencyManager" => {

                if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == path) {
                    match packed_file_view.get_view() {

                        // In case of tables, we have to get the logical row/column of the match and select it.
                        ViewType::Internal(view) => if let View::Table(view) = view {
                            let table_view = view.get_ref_table();
                            let table_view = table_view.get_mut_ptr_table_view_primary();
                            let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                            let table_selection_model = table_view.selection_model();

                            let row = model.item_2a(model_index.row(), 3).text().to_std_string().parse::<i32>().unwrap() - 1;
                            let column = model.item_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap();

                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                            }
                        },

                        _ => {},
                    }
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
            DiagnosticType::DB(ref diagnostic) |
            DiagnosticType::Loc(ref diagnostic) => diagnostic.get_path(),
            DiagnosticType::DependencyManager(_) => &[],
            _ => return,
        };

        if let Some(view) = UI_STATE.get_open_packedfiles().iter().find(|view| view.get_path() == path) {
            if app_ui.tab_bar_packed_file.index_of(view.get_mut_widget()) != -1 {
                match view.get_view() {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    ViewType::Internal(view) => if let View::Table(view) = view {
                        let table_view = view.get_ref_table().get_mut_ptr_table_view_primary();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let _blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                        match diagnostic {
                            DiagnosticType::DB(ref diagnostic) |
                            DiagnosticType::Loc(ref diagnostic) => {
                                for result in diagnostic.get_ref_result() {
                                    if result.row_number >= 0 {
                                        let table_model_index = table_model.index_2a(result.row_number as i32, result.column_number as i32);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level {
                                                DiagnosticLevel::Error => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_error())))),
                                                DiagnosticLevel::Warning => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_warning())))),
                                                DiagnosticLevel::Info => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_info())))),
                                            }
                                        }
                                    }
                                }
                            },
                            DiagnosticType::DependencyManager(ref diagnostic) => {
                                for result in diagnostic.get_ref_result() {
                                    if result.row_number >= 0 {
                                        let table_model_index = table_model.index_2a(result.row_number as i32, result.column_number as i32);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level {
                                                DiagnosticLevel::Error => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_error())))),
                                                DiagnosticLevel::Warning => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_warning())))),
                                                DiagnosticLevel::Info => table_model_item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_info())))),
                                            }
                                        }
                                    }
                                }
                            },
                            _ => return,
                        }
                    },

                    _ => {},
                }
            }
        }
    }

    pub unsafe fn clean_diagnostics_from_views(app_ui: &Rc<AppUI>) {
        for view in UI_STATE.get_open_packedfiles().iter() {

            // Only update the visible tables.
            if app_ui.tab_bar_packed_file.index_of(view.get_mut_widget()) != -1 {
                match view.get_view() {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    ViewType::Internal(view) => if let View::Table(view) = view {
                        let table_view = view.get_ref_table().get_mut_ptr_table_view_primary();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let _blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                        // Trick to get the right neutral colors: add an item, get the brush, delete it.
                        let base_qbrush = table_view.palette().brush_1a(ColorRole::Text);

                        for row in 0..table_model.row_count_0a() - 1 {
                            for column in 0..table_model.column_count_0a() {
                                let item = table_model.item_2a(row, column);
                                if item.foreground() != base_qbrush {
                                    item.set_foreground(base_qbrush);
                                }
                            }
                        }
                    }

                    _ => {}
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
        patterns.push(QString::from_std_str(pattern_level));
        sensitivity.push(CaseSensitivity::CaseSensitive);

        // Check for currently open files filter.
        if diagnostics_ui.diagnostics_button_only_current_packed_file.is_checked() {
            let open_packedfiles = UI_STATE.get_open_packedfiles();
            let open_packedfiles_ref = open_packedfiles.iter().filter(|x| app_ui.tab_bar_packed_file.index_of(x.get_mut_widget()) != -1).collect::<Vec<&PackedFileView>>();
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

            columns.push(4);
            patterns.push(QString::from_std_str(pattern));
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

        if diagnostics_ui.checkbox_invalid_dependency_packfile.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", DependencyManagerDiagnosticReportType::InvalidDependencyPackFileName));
        }

        diagnostic_type_pattern.pop();

        if diagnostic_type_pattern.is_empty() {
            diagnostic_type_pattern.push_str("empty");
        }

        columns.push(6);
        patterns.push(QString::from_std_str(diagnostic_type_pattern));
        sensitivity.push(CaseSensitivity::CaseSensitive);

        // Filter whatever it's in that column by the text we got.
        trigger_tableview_filter_safe(&diagnostics_ui.diagnostics_table_filter, &columns, &patterns, &sensitivity);
    }

    pub unsafe fn update_level_counts(diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {
        let info = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Info = y.level { true } else { false })
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Info = y.level { true } else { false })
                    .count(),
                 DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Info = y.level { true } else { false })
                    .count()
            }).sum::<usize>();

        let warning = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Warning = y.level { true } else { false })
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Warning = y.level { true } else { false })
                    .count(),
                DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Warning = y.level { true } else { false })
                    .count()
            }).sum::<usize>();


        let error = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Error = y.level { true } else { false })
                    .count(),
                DiagnosticType::PackFile(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Error = y.level { true } else { false })
                    .count(),
                DiagnosticType::DependencyManager(ref diag) => diag.get_ref_result()
                    .iter()
                    .filter(|y| if let DiagnosticLevel::Error = y.level { true } else { false })
                    .count()
            }).sum::<usize>();

        diagnostics_ui.diagnostics_button_info.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_info"), info)));
        diagnostics_ui.diagnostics_button_warning.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_warning"), warning)));
        diagnostics_ui.diagnostics_button_error.set_text(&QString::from_std_str(&format!("{} ({})", tr("diagnostics_button_error"), error)));
    }
}
