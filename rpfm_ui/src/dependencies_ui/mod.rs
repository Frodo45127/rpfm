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
Module with all the code related to the main `DependenciesUI`.
!*/

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QAction;
use qt_widgets::QDockWidget;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::{CaseSensitivity, ContextMenuPolicy, DockWidgetArea};
use qt_core::QBox;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QTimer;
use qt_core::QString;

use std::collections::BTreeMap;
use std::rc::Rc;

use rpfm_error::ErrorKind;
use rpfm_lib::packfile::PathType;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::locale::qtr;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::UI_STATE;
use crate::utils::*;

pub mod connections;
pub mod shortcuts;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Dependencies panel.
pub struct DependenciesUI {

    //-------------------------------------------------------------------------------//
    // `Dependencies` Dock Widget.
    //-------------------------------------------------------------------------------//
    pub dependencies_dock_widget: QBox<QDockWidget>,
    //pub dependencies_pined_table: Ptr<QTableView>,
    pub dependencies_tree_view: QBox<QTreeView>,
    pub dependencies_tree_model_filter: QBox<QSortFilterProxyModel>,
    pub dependencies_tree_model: QBox<QStandardItemModel>,
    pub filter_line_edit: QBox<QLineEdit>,
    pub filter_autoexpand_matches_button: QBox<QPushButton>,
    pub filter_case_sensitive_button: QBox<QPushButton>,
    pub filter_timer_delayed_updates: QBox<QTimer>,

    //-------------------------------------------------------------------------------//
    // Contextual menu for the Dependencies TreeView.
    //-------------------------------------------------------------------------------//
    pub dependencies_tree_view_context_menu: QBox<QMenu>,
    pub context_menu_import: QPtr<QAction>,
    pub context_menu_copy_path: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Actions not in the UI.
    //-------------------------------------------------------------------------------//
    pub dependencies_tree_view_expand_all: QBox<QAction>,
    pub dependencies_tree_view_collapse_all: QBox<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DependenciesUI`.
impl DependenciesUI {

    /// This function creates an entire `DependenciesUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Self {

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'TreeView` Dock Widget and all his contents.
        let dependencies_dock_widget = QDockWidget::from_q_widget(main_window);
        let dependencies_dock_inner_widget = QWidget::new_1a(&dependencies_dock_widget);
        let dependencies_dock_layout = create_grid_layout(dependencies_dock_inner_widget.static_upcast());
        dependencies_dock_widget.set_widget(&dependencies_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::LeftDockWidgetArea, &dependencies_dock_widget);
        dependencies_dock_widget.set_window_title(&qtr("gen_loc_dependencies"));
        dependencies_dock_widget.set_object_name(&QString::from_std_str("dependencies_dock"));

        // Create and configure the `TreeView` itself.
        let dependencies_tree_view = QTreeView::new_1a(&dependencies_dock_inner_widget);
        let dependencies_tree_model = new_packed_file_model_safe();
        let dependencies_tree_model_filter = new_treeview_filter_safe(dependencies_tree_view.static_upcast());
        dependencies_tree_model_filter.set_source_model(&dependencies_tree_model);
        dependencies_tree_model.set_parent(&dependencies_tree_view);
        dependencies_tree_view.set_model(&dependencies_tree_model_filter);
        dependencies_tree_view.set_header_hidden(true);
        dependencies_tree_view.set_animated(true);
        dependencies_tree_view.set_uniform_row_heights(true);
        dependencies_tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        dependencies_tree_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        dependencies_tree_view.set_expands_on_double_click(true);
        dependencies_tree_view.header().set_stretch_last_section(false);

        // Apply the view's delegate.
        new_tree_item_delegate_safe(&dependencies_tree_view.static_upcast::<QObject>().as_ptr(), true);

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_timer_delayed_updates = QTimer::new_1a(&dependencies_dock_widget);
        let filter_line_edit = QLineEdit::from_q_widget(&dependencies_dock_inner_widget);
        let filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), &dependencies_dock_inner_widget);
        let filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), &dependencies_dock_inner_widget);
        filter_timer_delayed_updates.set_single_shot(true);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Add everything to the `TreeView`s Dock Layout.
        dependencies_dock_layout.add_widget_5a(&dependencies_tree_view, 0, 0, 1, 2);
        dependencies_dock_layout.add_widget_5a(&filter_line_edit, 1, 0, 1, 2);
        dependencies_dock_layout.add_widget_5a(&filter_autoexpand_matches_button, 2, 0, 1, 1);
        dependencies_dock_layout.add_widget_5a(&filter_case_sensitive_button, 2, 1, 1, 1);

        //-------------------------------------------------------------------------------//
        // Contextual menu for the Dependencies TreeView.
        //-------------------------------------------------------------------------------//

        // Populate the `Contextual Menu` for the `Dependencies` TreeView.
        let dependencies_tree_view_context_menu = QMenu::from_q_widget(&dependencies_dock_inner_widget);

        let context_menu_import = dependencies_tree_view_context_menu.add_action_q_string(&qtr("context_menu_import"));
        let context_menu_copy_path = dependencies_tree_view_context_menu.add_action_q_string(&qtr("context_menu_copy_path"));

        let dependencies_tree_view_expand_all = QAction::from_q_string(&qtr("treeview_expand_all"));
        let dependencies_tree_view_collapse_all = QAction::from_q_string(&qtr("treeview_collapse_all"));

        // Create ***Da monsta***.
        Self {

            //-------------------------------------------------------------------------------//
            // `Dependencies` Dock Widget.
            //-------------------------------------------------------------------------------//
            dependencies_dock_widget,
            dependencies_tree_view,
            dependencies_tree_model_filter,
            dependencies_tree_model,
            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
            filter_timer_delayed_updates,

            //-------------------------------------------------------------------------------//
            // Contextual menu for the Dependencies TreeView.
            //-------------------------------------------------------------------------------//

            dependencies_tree_view_context_menu,

            context_menu_import,
            context_menu_copy_path,

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            dependencies_tree_view_expand_all,
            dependencies_tree_view_collapse_all,
        }
    }

    /// Function to filter the Dependencies TreeView.
    pub unsafe fn filter_files(&self) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&self.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&self.dependencies_tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if self.filter_autoexpand_matches_button.is_checked() {
            self.dependencies_tree_view.expand_all();
        }
    }

    pub unsafe fn start_delayed_updates_timer(&self) {
        self.filter_timer_delayed_updates.set_interval(500);
        self.filter_timer_delayed_updates.start_0a();
    }

    /// This function is used to import dependencies into our own PackFile.
    pub unsafe fn import_dependencies(&self, paths_by_source: BTreeMap<DataSource, Vec<PathType>>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        app_ui.main_window.set_enabled(false);
        let receiver = CENTRAL_COMMAND.send_background(Command::ImportDependenciesToOpenPackFile(paths_by_source));
        let response1 = CentralCommand::recv(&receiver);
        let response2 = CentralCommand::recv(&receiver);
        match response1 {
            Response::VecPathType(added_paths) => {
                let paths = added_paths.iter().map(From::from).collect::<Vec<TreePathType>>();
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths.to_vec()), DataSource::PackFile);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = added_paths.iter().filter_map(|path| {
                    if let PathType::File(ref path) = path {
                        if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                            if packed_file_view.reload(path, pack_file_contents_ui).is_err() {
                                Some(path.to_vec())
                            } else { None }
                        } else { None }
                    } else { None }
                }).collect::<Vec<Vec<String>>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response1),
        }

        match response2 {
            Response::Success => {},
            Response::VecVecString(error_paths) => show_dialog(&app_ui.main_window, ErrorKind::DependenciesImportFailure(error_paths), false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response2),
        }

        // Re-enable the Main Window.
        app_ui.main_window.set_enabled(true);
    }
}
