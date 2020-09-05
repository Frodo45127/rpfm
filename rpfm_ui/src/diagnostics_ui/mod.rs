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
use qt_widgets::QDockWidget;
use qt_widgets::QGroupBox;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QMainWindow;
use qt_widgets::QPushButton;
use qt_widgets::QTableView;
use qt_widgets::QWidget;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::{CaseSensitivity, ContextMenuPolicy, DockWidgetArea, Orientation, SortOrder};
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::MutPtr;
use cpp_core::Ptr;

use rpfm_error::ErrorKind;

use rpfm_lib::diagnostics::{Diagnostic, DiagnosticResult};
use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use crate::AppUI;
use crate::communications::Command;
use crate::CENTRAL_COMMAND;
use crate::ffi::add_to_q_list_safe;
use crate::locale::qtr;
use crate::pack_tree::{PackTree, get_color_info, get_color_warning, get_color_error, get_color_info_pressed, get_color_warning_pressed, get_color_error_pressed, TreeViewOperation};
use crate::packedfile_views::{View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use crate::utils::{create_grid_layout, show_dialog};

pub mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Diagnostics panel.
#[derive(Copy, Clone)]
pub struct DiagnosticsUI {

    //-------------------------------------------------------------------------------//
    // `Diagnostics` Dock Widget.
    //-------------------------------------------------------------------------------//
    pub diagnostics_dock_widget: MutPtr<QDockWidget>,
    pub diagnostics_table_view: MutPtr<QTableView>,
    pub diagnostics_table_filter: MutPtr<QSortFilterProxyModel>,
    pub diagnostics_table_model: MutPtr<QStandardItemModel>,

    //-------------------------------------------------------------------------------//
    // Filters section.
    //-------------------------------------------------------------------------------//
    pub diagnostics_button_error: MutPtr<QPushButton>,
    pub diagnostics_button_warning: MutPtr<QPushButton>,
    pub diagnostics_button_info: MutPtr<QPushButton>,
    pub diagnostics_button_only_current_packed_file: MutPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUI`.
impl DiagnosticsUI {

    /// This function creates an entire `DiagnosticsUI` struct.
    pub unsafe fn new(mut main_window: MutPtr<QMainWindow>) -> Self {

        //-----------------------------------------------//
        // `DiagnosticsUI` DockWidget.
        //-----------------------------------------------//
        let mut diagnostics_dock_widget = QDockWidget::from_q_widget(main_window).into_ptr();
        let diagnostics_dock_inner_widget = QWidget::new_0a().into_ptr();
        let mut diagnostics_dock_layout = create_grid_layout(diagnostics_dock_inner_widget);
        diagnostics_dock_widget.set_widget(diagnostics_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::BottomDockWidgetArea, diagnostics_dock_widget);
        diagnostics_dock_widget.set_window_title(&qtr("gen_loc_diagnostics"));

        // Create and configure the filters section.
        let filter_frame = QGroupBox::new().into_ptr();
        let mut filter_grid = create_grid_layout(filter_frame.static_upcast_mut());
        filter_grid.set_contents_margins_4a(4, 0, 4, 0);

        let mut diagnostics_button_error = QPushButton::from_q_string(&qtr("diagnostics_button_error"));
        let mut diagnostics_button_warning = QPushButton::from_q_string(&qtr("diagnostics_button_warning"));
        let mut diagnostics_button_info = QPushButton::from_q_string(&qtr("diagnostics_button_info"));
        let mut diagnostics_button_only_current_packed_file = QPushButton::from_q_string(&qtr("diagnostics_button_only_current_packed_file"));
        diagnostics_button_error.set_checkable(true);
        diagnostics_button_warning.set_checkable(true);
        diagnostics_button_info.set_checkable(true);
        diagnostics_button_only_current_packed_file.set_checkable(true);
        diagnostics_button_error.set_checked(true);

        // Hidden until we get this working.
        diagnostics_button_only_current_packed_file.set_visible(false);

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

        filter_grid.add_widget_5a(&mut diagnostics_button_error, 0, 0, 1, 1);
        filter_grid.add_widget_5a(&mut diagnostics_button_warning, 0, 1, 1, 1);
        filter_grid.add_widget_5a(&mut diagnostics_button_info, 0, 2, 1, 1);
        filter_grid.add_widget_5a(&mut diagnostics_button_only_current_packed_file, 0, 3, 1, 1);

        let mut diagnostics_table_view = QTableView::new_0a();
        let mut diagnostics_table_filter = QSortFilterProxyModel::new_0a();
        let mut diagnostics_table_model = QStandardItemModel::new_0a();
        diagnostics_table_filter.set_source_model(&mut diagnostics_table_model);
        diagnostics_table_view.set_model(&mut diagnostics_table_filter);
        diagnostics_table_view.set_selection_mode(SelectionMode::ExtendedSelection);
        diagnostics_table_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

        if SETTINGS.read().unwrap().settings_bool["tight_table_mode"] {
            diagnostics_table_view.vertical_header().set_minimum_section_size(22);
            diagnostics_table_view.vertical_header().set_maximum_section_size(22);
            diagnostics_table_view.vertical_header().set_default_section_size(22);
        }

        diagnostics_dock_layout.add_widget_5a(filter_frame, 0, 0, 1, 1);
        diagnostics_dock_layout.add_widget_5a(&mut diagnostics_table_view, 1, 0, 1, 1);

        main_window.set_corner(qt_core::Corner::BottomLeftCorner, qt_core::DockWidgetArea::LeftDockWidgetArea);
        main_window.set_corner(qt_core::Corner::BottomRightCorner, qt_core::DockWidgetArea::RightDockWidgetArea);

        Self {

            //-------------------------------------------------------------------------------//
            // `Diagnostics` Dock Widget.
            //-------------------------------------------------------------------------------//
            diagnostics_dock_widget,
            diagnostics_table_view: diagnostics_table_view.into_ptr(),
            diagnostics_table_filter: diagnostics_table_filter.into_ptr(),
            diagnostics_table_model: diagnostics_table_model.into_ptr(),

            //-------------------------------------------------------------------------------//
            // Filters section.
            //-------------------------------------------------------------------------------//
            diagnostics_button_error: diagnostics_button_error.into_ptr(),
            diagnostics_button_warning: diagnostics_button_warning.into_ptr(),
            diagnostics_button_info: diagnostics_button_info.into_ptr(),
            diagnostics_button_only_current_packed_file: diagnostics_button_only_current_packed_file.into_ptr(),
        }
    }

    /// This function takes care of checking the entire PackFile for errors.
    pub unsafe fn check(&mut self) {
        CENTRAL_COMMAND.send_message_qt(Command::DiagnosticsCheck);
        let diagnostics = CENTRAL_COMMAND.recv_message_diagnostics_to_qt_try();
        Self::load_diagnostics_to_ui(&mut self.diagnostics_table_model, &mut self.diagnostics_table_view, diagnostics.get_ref_diagnostics());
        self.filter_by_level();
    }

    /// This function takes care of updating the results of a diagnostics check for the provided paths.
    pub unsafe fn check_on_path(&mut self, pack_file_contents_ui: &mut PackFileContentsUI, paths: Vec<PathType>) {
        let diagnostics = UI_STATE.get_diagnostics();
        CENTRAL_COMMAND.send_message_qt(Command::DiagnosticsUpdate((diagnostics, paths)));
        let (diagnostics, packed_files_info) = CENTRAL_COMMAND.recv_message_diagnostics_update_to_qt_try();

        self.diagnostics_table_model.clear();
        Self::load_diagnostics_to_ui(&mut self.diagnostics_table_model, &mut self.diagnostics_table_view, &diagnostics.get_ref_diagnostics());
        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info));

        self.filter_by_level();
        UI_STATE.set_diagnostics(&diagnostics);
    }

    /// This function takes care of loading the results of a diagnostic check into the table.
    pub unsafe fn load_diagnostics_to_ui(model: &mut QStandardItemModel, table_view: &mut QTableView, diagnostics: &[Diagnostic]) {
        if !diagnostics.is_empty() {
            for diagnostic in diagnostics {
                for result in diagnostic.get_result() {
                    let qlist_boi = QListOfQStandardItem::new().into_ptr();

                    // Create an empty row.
                    let mut level = QStandardItem::new().into_ptr();
                    let mut column = QStandardItem::new().into_ptr();
                    let mut row = QStandardItem::new().into_ptr();
                    let mut path = QStandardItem::new().into_ptr();
                    let mut message = QStandardItem::new().into_ptr();

                    let (result_data, result_type, color) = match result {
                        DiagnosticResult::Info(data) => (data, "Info".to_owned(), get_color_info()),
                        DiagnosticResult::Warning(data) => (data, "Warning".to_owned(), get_color_warning()),
                        DiagnosticResult::Error(data) => (data, "Error".to_owned(), get_color_error()),
                    };

                    level.set_background(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
                    level.set_text(&QString::from_std_str(result_type));
                    column.set_data_2a(&QVariant::from_uint(result_data.column_number), 2);
                    row.set_data_2a(&QVariant::from_i64(result_data.row_number + 1), 2);
                    path.set_text(&QString::from_std_str(&diagnostic.get_path().join("/")));
                    message.set_text(&QString::from_std_str(&result_data.message));

                    level.set_editable(false);
                    column.set_editable(false);
                    row.set_editable(false);
                    path.set_editable(false);
                    message.set_editable(false);

                    // Add an empty row to the list.
                    add_to_q_list_safe(qlist_boi, level);
                    add_to_q_list_safe(qlist_boi, column);
                    add_to_q_list_safe(qlist_boi, row);
                    add_to_q_list_safe(qlist_boi, path);
                    add_to_q_list_safe(qlist_boi, message);

                    // Append the new row.
                    model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref().unwrap());
                }
            }

            model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_level")));
            model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_column")));
            model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_row")));
            model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_path")));
            model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_message")));

            // Hide the column number column for tables.
            table_view.hide_column(1);
            table_view.hide_column(2);
            table_view.sort_by_column_2a(3, SortOrder::AscendingOrder);

            table_view.horizontal_header().set_stretch_last_section(true);
            table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    pub unsafe fn open_match(
        app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let mut tree_view = pack_file_contents_ui.packfile_contents_tree_view;
        let filter_model: Ptr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: MutPtr<QStandardItemModel> = filter_model.source_model().static_downcast_mut();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let item_path = model.item_2a(model_index.row(), 3);
        let path = item_path.text().to_std_string();
        let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();

        if let Some(pack_file_contents_model_index) = pack_file_contents_ui.packfile_contents_tree_view.expand_treeview_to_item(&path) {
            let pack_file_contents_model_index = pack_file_contents_model_index.as_ref().unwrap();
            let mut selection_model = tree_view.selection_model();

            // If it's not in the current TreeView Filter we CAN'T OPEN IT.
            //
            // Note: the selection should already trigger the open PackedFile action.
            if pack_file_contents_model_index.is_valid() {
                tree_view.scroll_to_1a(pack_file_contents_model_index);
                selection_model.select_q_model_index_q_flags_selection_flag(pack_file_contents_model_index, QFlags::from(SelectionFlag::ClearAndSelect));

                if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == path) {
                    match packed_file_view.get_view() {

                        // In case of tables, we have to get the logical row/column of the match and select it.
                        ViewType::Internal(view) => if let View::Table(view) = view {
                            let table_view = view.get_ref_table();
                            let mut table_view = table_view.get_mut_ptr_table_view_primary();
                            let table_filter: MutPtr<QSortFilterProxyModel> = table_view.model().static_downcast_mut();
                            let table_model: MutPtr<QStandardItemModel> = table_filter.source_model().static_downcast_mut();
                            let mut table_selection_model = table_view.selection_model();

                            let row = model.item_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap() - 1;
                            let column = model.item_2a(model_index.row(), 1).text().to_std_string().parse::<i32>().unwrap();

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
        }
        else { show_dialog(app_ui.main_window, ErrorKind::PackedFileNotInFilter, false); }
    }

    pub unsafe fn filter_by_level(&mut self) {
        let info_state = self.diagnostics_button_info.is_checked();
        let warning_state = self.diagnostics_button_warning.is_checked();
        let error_state = self.diagnostics_button_error.is_checked();
        let pattern = match (info_state, warning_state, error_state) {
            (true, true, true) => "Info|Warning|Error",
            (true, true, false) => "Info|Warning",
            (true, false, true) => "Info|Error",
            (false, true, true) => "Warning|Error",
            (true, false, false) => "Info",
            (false, false, true) => "Error",
            (false, true, false) => "Warning",
            (false, false, false) => "-1",
        };

        let pattern = QRegExp::new_1a(&QString::from_std_str(pattern));

        self.diagnostics_table_filter.set_filter_case_sensitivity(CaseSensitivity::CaseSensitive);
        self.diagnostics_table_filter.set_filter_key_column(0);
        self.diagnostics_table_filter.set_filter_reg_exp_q_reg_exp(&pattern);
    }
}
