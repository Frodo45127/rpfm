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
Module with all the code related to the main `DependenciesUI`.
!*/

use qt_widgets::QAction;
use qt_widgets::QDockWidget;
use qt_widgets::QFileDialog;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::{CaseSensitivity, DockWidgetArea};
use qt_core::QBox;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QTimer;
use qt_core::QString;

use anyhow::{anyhow, Result};
use getset::Getters;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_ipc::helpers::DataSource;

use rpfm_lib::files::ContainerPath;

use rpfm_ui_common::utils::{find_widget, load_template};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::UI_STATE;
use crate::utils::*;

pub mod connections;
pub mod slots;
pub mod tips;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/filterable_tree_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_dock_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Dependencies panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct DependenciesUI {

    //-------------------------------------------------------------------------------//
    // `Dependencies` Dock Widget.
    //-------------------------------------------------------------------------------//
    dependencies_dock_widget: QPtr<QDockWidget>,
    //dependencies_pined_table: Ptr<QTableView>,
    dependencies_tree_view: QPtr<QTreeView>,
    dependencies_tree_model_filter: QBox<QSortFilterProxyModel>,
    _dependencies_tree_model: QBox<QStandardItemModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_autoexpand_matches_button: QPtr<QToolButton>,
    filter_case_sensitive_button: QPtr<QToolButton>,
    filter_timer_delayed_updates: QBox<QTimer>,

    //-------------------------------------------------------------------------------//
    // Contextual menu for the Dependencies TreeView.
    //-------------------------------------------------------------------------------//
    dependencies_tree_view_context_menu: QBox<QMenu>,
    context_menu_extract: QPtr<QAction>,
    context_menu_import: QPtr<QAction>,
    context_menu_copy_path: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Actions not in the UI.
    //-------------------------------------------------------------------------------//
    dependencies_tree_view_expand_all: QPtr<QAction>,
    dependencies_tree_view_collapse_all: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DependenciesUI`.
impl DependenciesUI {

    /// This function creates an entire `DependenciesUI` struct.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let dependencies_dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();
        let dependencies_dock_inner_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "inner_widget")?;
        let dependencies_tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'TreeView` Dock Widget and all his contents.
        app_ui.main_window().add_dock_widget_2a(DockWidgetArea::LeftDockWidgetArea, &dependencies_dock_widget);
        dependencies_dock_widget.set_window_title(&qtr("gen_loc_dependencies"));
        dependencies_dock_widget.set_object_name(&QString::from_std_str("dependencies_dock"));

        // Create and configure the `TreeView` itself.
        let dependencies_tree_model = new_packed_file_model_safe();
        let dependencies_tree_model_filter = new_treeview_filter_safe(dependencies_tree_view.static_upcast());
        dependencies_tree_model_filter.set_source_model(&dependencies_tree_model);
        dependencies_tree_model.set_parent(&dependencies_tree_view);
        dependencies_tree_view.set_model(&dependencies_tree_model_filter);

        // Apply the view's delegate.
        new_tree_item_delegate_safe(&dependencies_tree_view.static_upcast::<QObject>().as_ptr(), true);

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_timer_delayed_updates = QTimer::new_1a(&dependencies_dock_widget);
        filter_timer_delayed_updates.set_single_shot(true);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        //-------------------------------------------------------------------------------//
        // Contextual menu for the Dependencies TreeView.
        //-------------------------------------------------------------------------------//

        // Populate the `Contextual Menu` for the `Dependencies` TreeView.
        let dependencies_tree_view_context_menu = QMenu::from_q_widget(&dependencies_dock_inner_widget);

        let context_menu_extract = add_action_to_menu(&dependencies_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "dependencies_context_menu", "extract_from_dependencies", "context_menu_extract", Some(dependencies_dock_widget.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_import = add_action_to_menu(&dependencies_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "dependencies_context_menu", "import_from_dependencies", "context_menu_import", Some(dependencies_dock_widget.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_copy_path = add_action_to_menu(&dependencies_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "dependencies_context_menu", "copy_path", "context_menu_copy_path", Some(dependencies_dock_widget.static_upcast::<qt_widgets::QWidget>()));
        let dependencies_tree_view_expand_all = add_action_to_menu(&dependencies_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "dependencies_context_menu", "expand_all", "treeview_expand_all", Some(dependencies_dock_widget.static_upcast::<qt_widgets::QWidget>()));
        let dependencies_tree_view_collapse_all = add_action_to_menu(&dependencies_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "dependencies_context_menu", "collapsse_all", "treeview_collapse_all", Some(dependencies_dock_widget.static_upcast::<qt_widgets::QWidget>()));

        context_menu_extract.set_enabled(false);
        context_menu_import.set_enabled(false);

        // Create ***Da monsta***.
        Ok(Self {

            //-------------------------------------------------------------------------------//
            // `Dependencies` Dock Widget.
            //-------------------------------------------------------------------------------//
            dependencies_dock_widget,
            dependencies_tree_view,
            dependencies_tree_model_filter,
            _dependencies_tree_model: dependencies_tree_model,
            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
            filter_timer_delayed_updates,

            //-------------------------------------------------------------------------------//
            // Contextual menu for the Dependencies TreeView.
            //-------------------------------------------------------------------------------//
            dependencies_tree_view_context_menu,
            context_menu_extract,
            context_menu_import,
            context_menu_copy_path,

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            dependencies_tree_view_expand_all,
            dependencies_tree_view_collapse_all,
        })
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
    pub unsafe fn import_dependencies(&self, paths_by_source: BTreeMap<DataSource, Vec<ContainerPath>>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        app_ui.toggle_main_window(false);

        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::ImportDependenciesToOpenPackFile(paths_by_source));
        let response1 = CentralCommand::recv(&receiver);
        let response2 = CentralCommand::recv(&receiver);
        match response1 {
            Response::VecContainerPath(paths) => {
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);

                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = paths.iter().filter_map(|path| {
                    if let ContainerPath::File(ref path) = path {
                        if let Some(file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.path_read() == *path && x.data_source() == DataSource::PackFile) {
                            if file_view.reload(path, pack_file_contents_ui).is_err() {
                                Some(path.to_owned())
                            } else { None }
                        } else { None }
                    } else { None }
                }).collect::<Vec<String>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response1:?}"),
        }

        match response2 {
            Response::Success => {},
            Response::VecString(error_paths) => show_dialog(app_ui.main_window(), anyhow!("<p>There was an error importing the following files:</p> <ul>{}</ul>", error_paths.iter().map(|x| "<li>".to_owned() + x + "</li>").collect::<String>()), false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response2:?}"),
        }

        // Re-enable the Main Window.
        app_ui.toggle_main_window(true);
    }

    pub unsafe fn extract(&self, app_ui: &Rc<AppUI>) {
        let paths = self.dependencies_tree_view.get_item_types_and_data_source_from_selection(true);
        let parent_paths = paths.iter().filter_map(|(path, source)| if let DataSource::ParentFiles = source { Some(path.to_owned()) } else { None }).collect::<Vec<ContainerPath>>();
        let game_paths = paths.iter().filter_map(|(path, source)| if let DataSource::GameFiles = source { Some(path.to_owned()) } else { None }).collect::<Vec<ContainerPath>>();

        let mut paths_by_source = BTreeMap::new();
        if !parent_paths.is_empty() {
            paths_by_source.insert(DataSource::ParentFiles, parent_paths);
        }

        if !game_paths.is_empty() {
            paths_by_source.insert(DataSource::GameFiles, game_paths);
        }

        let extraction_path = QFileDialog::get_existing_directory_2a(
            app_ui.main_window(),
            &qtr("context_menu_extract_packfile"),
        );

        let extraction_path = if !extraction_path.is_empty() {
            PathBuf::from(extraction_path.to_std_string())
        } else { return };

        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::ExtractPackedFiles(paths_by_source, extraction_path, true));
        app_ui.toggle_main_window(false);
        let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
        match response {
            Response::StringVecPathBuf(result, _) => show_message_info(app_ui.message_widget(), result),
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }
        app_ui.toggle_main_window(true);
    }
}
