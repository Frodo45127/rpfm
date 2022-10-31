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
Module with all the code related to the main `PackFileContentsUI`.
!*/

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QAction;
use qt_widgets::QCheckBox;
use qt_widgets::QDialog;
use qt_widgets::QDockWidget;
use qt_widgets::{QFileDialog, q_file_dialog::FileMode};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::CaseSensitivity;
use qt_core::{ContextMenuPolicy, DockWidgetArea};
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::SlotNoArgs;

use getset::Getters;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::files::ContainerPath;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::locale::{qtr, qtre};
use crate::packedfile_views::DataSource;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::settings_ui::backend::*;
use crate::utils::*;
use crate::ui_state::OperationalMode;
use crate::UI_STATE;

pub mod connections;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the PackFile Contents panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackFileContentsUI {

    //-------------------------------------------------------------------------------//
    // `PackFile Contents` Dock Widget.
    //-------------------------------------------------------------------------------//
    packfile_contents_dock_widget: QBox<QDockWidget>,
    //packfile_contents_pined_table: Ptr<QTableView>,
    packfile_contents_tree_view: QBox<QTreeView>,
    packfile_contents_tree_model_filter: QBox<QSortFilterProxyModel>,
    packfile_contents_tree_model: QBox<QStandardItemModel>,
    filter_line_edit: QBox<QLineEdit>,
    filter_autoexpand_matches_button: QBox<QPushButton>,
    filter_case_sensitive_button: QBox<QPushButton>,
    filter_timer_delayed_updates: QBox<QTimer>,

    //-------------------------------------------------------------------------------//
    // Contextual menu for the PackFile Contents TreeView.
    //-------------------------------------------------------------------------------//
    packfile_contents_tree_view_context_menu: QBox<QMenu>,
    context_menu_add_file: QPtr<QAction>,
    context_menu_add_folder: QPtr<QAction>,
    context_menu_add_from_packfile: QPtr<QAction>,
    context_menu_new_folder: QPtr<QAction>,
    context_menu_new_packed_file_anim_pack: QPtr<QAction>,
    context_menu_new_packed_file_db: QPtr<QAction>,
    context_menu_new_packed_file_loc: QPtr<QAction>,
    context_menu_new_packed_file_text: QPtr<QAction>,
    context_menu_new_queek_packed_file: QPtr<QAction>,
    context_menu_rename: QPtr<QAction>,
    context_menu_delete: QPtr<QAction>,
    context_menu_extract: QPtr<QAction>,
    context_menu_copy_path: QPtr<QAction>,
    context_menu_open_decoder: QPtr<QAction>,
    context_menu_open_dependency_manager: QPtr<QAction>,
    context_menu_open_containing_folder: QPtr<QAction>,
    context_menu_open_packfile_settings: QPtr<QAction>,
    context_menu_open_with_external_program: QPtr<QAction>,
    context_menu_open_notes: QPtr<QAction>,
    context_menu_merge_tables: QPtr<QAction>,
    context_menu_update_table: QPtr<QAction>,
    context_menu_generate_missing_loc_data: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Actions not in the UI.
    //-------------------------------------------------------------------------------//
    packfile_contents_tree_view_expand_all: QPtr<QAction>,
    packfile_contents_tree_view_collapse_all: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function creates an entire `PackFileContentsUI` struct.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Self {

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'TreeView` Dock Widget and all his contents.
        let packfile_contents_dock_widget = QDockWidget::from_q_widget(app_ui.main_window());
        let packfile_contents_dock_inner_widget = QWidget::new_1a(&packfile_contents_dock_widget);
        let packfile_contents_dock_layout = create_grid_layout(packfile_contents_dock_inner_widget.static_upcast());
        packfile_contents_dock_widget.set_widget(&packfile_contents_dock_inner_widget);
        app_ui.main_window().add_dock_widget_2a(DockWidgetArea::LeftDockWidgetArea, &packfile_contents_dock_widget);
        packfile_contents_dock_widget.set_window_title(&qtr("gen_loc_packfile_contents"));
        packfile_contents_dock_widget.set_object_name(&QString::from_std_str("packfile_contents_dock"));

        // Create and configure the `TreeView` itself.
        let packfile_contents_tree_view = QTreeView::new_1a(&packfile_contents_dock_inner_widget);
        let packfile_contents_tree_model = new_packed_file_model_safe();
        let packfile_contents_tree_model_filter = new_treeview_filter_safe(packfile_contents_tree_view.static_upcast());
        packfile_contents_tree_model_filter.set_source_model(&packfile_contents_tree_model);
        packfile_contents_tree_model.set_parent(&packfile_contents_tree_view);
        packfile_contents_tree_view.set_model(&packfile_contents_tree_model_filter);
        packfile_contents_tree_view.set_header_hidden(true);
        packfile_contents_tree_view.set_animated(true);
        packfile_contents_tree_view.set_uniform_row_heights(true);
        packfile_contents_tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        packfile_contents_tree_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        packfile_contents_tree_view.set_expands_on_double_click(true);
        packfile_contents_tree_view.header().set_stretch_last_section(false);

        // Apply the view's delegate.
        new_tree_item_delegate_safe(&packfile_contents_tree_view.static_upcast::<QObject>().as_ptr(), true);

        // Not yet working.
        if setting_bool("packfile_treeview_resize_to_fit") {
            //packfile_contents_tree_view.set_size_adjust_policy(qt_widgets::q_abstract_scroll_area::SizeAdjustPolicy::AdjustToContents);
            //packfile_contents_tree_view.horizontal_scroll_bar().set_disabled(true);
            //packfile_contents_tree_view.set_horizontal_scroll_bar_policy(qt_core::ScrollBarPolicy::ScrollBarAlwaysOff);
            //packfile_contents_tree_view.header().set_section_resize_mode_1a(ResizeMode::ResizeToContents);
            //packfile_contents_tree_view.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Maximum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_inner_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Minimum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::MinimumExpanding, qt_widgets::q_size_policy::Policy::Maximum);

        }

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_timer_delayed_updates = QTimer::new_1a(&packfile_contents_dock_widget);
        let filter_line_edit = QLineEdit::from_q_widget(&packfile_contents_dock_inner_widget);
        let filter_autoexpand_matches_button = QPushButton::from_q_string_q_widget(&qtr("treeview_autoexpand"), &packfile_contents_dock_inner_widget);
        let filter_case_sensitive_button = QPushButton::from_q_string_q_widget(&qtr("treeview_aai"), &packfile_contents_dock_inner_widget);
        filter_timer_delayed_updates.set_single_shot(true);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Add everything to the `TreeView`s Dock Layout.
        packfile_contents_dock_layout.add_widget_5a(&packfile_contents_tree_view, 0, 0, 1, 2);
        packfile_contents_dock_layout.add_widget_5a(&filter_line_edit, 1, 0, 1, 2);
        packfile_contents_dock_layout.add_widget_5a(&filter_autoexpand_matches_button, 2, 0, 1, 1);
        packfile_contents_dock_layout.add_widget_5a(&filter_case_sensitive_button, 2, 1, 1, 1);

        //-------------------------------------------------------------------------------//
        // Contextual menu for the PackFile Contents TreeView.
        //-------------------------------------------------------------------------------//

        // Populate the `Contextual Menu` for the `PackFile` TreeView.
        let packfile_contents_tree_view_context_menu = QMenu::from_q_widget(&packfile_contents_dock_inner_widget);
        let menu_add = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_add"));
        let menu_create = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_create"));
        let menu_open = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_open"));

        let context_menu_add_file = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_file", "context_menu_add_file");
        let context_menu_add_folder = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_folder", "context_menu_add_folder");
        let context_menu_add_from_packfile = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_from_pack", "context_menu_add_from_packfile");
        let context_menu_new_folder = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_folder", "context_menu_new_folder");
        let context_menu_new_packed_file_anim_pack = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_animpack", "context_menu_new_packed_file_anim_pack");
        let context_menu_new_packed_file_db = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_db", "context_menu_new_packed_file_db");
        let context_menu_new_packed_file_loc = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_loc", "context_menu_new_packed_file_loc");
        let context_menu_new_packed_file_text = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_text", "context_menu_new_packed_file_text");
        let context_menu_new_queek_packed_file = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_quick_file", "context_menu_new_queek_packed_file");
        let context_menu_rename = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "rename", "context_menu_rename");
        let context_menu_delete = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "delete", "context_menu_delete");
        let context_menu_extract = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "extract", "context_menu_extract");
        let context_menu_copy_path = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "copy_path", "context_menu_copy_path");
        let context_menu_open_decoder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_decoder", "context_menu_open_decoder");
        let context_menu_open_dependency_manager = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_dependency_manager", "context_menu_open_dependency_manager");
        let context_menu_open_containing_folder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_containing_folder", "context_menu_open_containing_folder");
        let context_menu_open_packfile_settings = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_settings", "context_menu_open_packfile_settings");
        let context_menu_open_with_external_program = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_external_program", "context_menu_open_with_external_program");
        let context_menu_open_notes = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_notes", "context_menu_open_notes");
        let context_menu_merge_tables = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "merge_files", "context_menu_merge_tables");
        let context_menu_update_table = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "update_files", "context_menu_update_table");
        let context_menu_generate_missing_loc_data = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "generate_missing_loc_data", "context_menu_generate_missing_loc_data");

        let packfile_contents_tree_view_expand_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "expand_all", "treeview_expand_all");
        let packfile_contents_tree_view_collapse_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "collapse_all", "treeview_collapse_all");

        shortcut_associate_action_group_to_widget_safe(app_ui.shortcuts().as_ptr(), QString::from_std_str("pack_tree_context_menu").into_ptr(), packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>().as_ptr());

        // Configure the `Contextual Menu` for the `PackFile` TreeView.
        packfile_contents_tree_view_context_menu.insert_separator(&menu_open.menu_action());
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_rename);
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_merge_tables);

        // Disable all the Contextual Menu actions by default.
        context_menu_add_file.set_enabled(false);
        context_menu_add_folder.set_enabled(false);
        context_menu_add_from_packfile.set_enabled(false);
        context_menu_new_folder.set_enabled(false);
        context_menu_new_packed_file_anim_pack.set_enabled(false);
        context_menu_new_packed_file_db.set_enabled(false);
        context_menu_new_packed_file_loc.set_enabled(false);
        context_menu_new_packed_file_text.set_enabled(false);
        context_menu_new_queek_packed_file.set_enabled(false);
        context_menu_delete.set_enabled(false);
        context_menu_rename.set_enabled(false);
        context_menu_extract.set_enabled(false);
        context_menu_copy_path.set_enabled(false);
        context_menu_open_decoder.set_enabled(false);
        context_menu_open_dependency_manager.set_enabled(false);
        context_menu_open_containing_folder.set_enabled(false);
        context_menu_open_packfile_settings.set_enabled(false);
        context_menu_open_with_external_program.set_enabled(false);
        context_menu_open_notes.set_enabled(false);

        // Create ***Da monsta***.
        Self {

            //-------------------------------------------------------------------------------//
            // `PackFile TreeView` Dock Widget.
            //-------------------------------------------------------------------------------//
            packfile_contents_dock_widget,
            packfile_contents_tree_view,
            packfile_contents_tree_model_filter,
            packfile_contents_tree_model,
            filter_line_edit,
            filter_autoexpand_matches_button,
            filter_case_sensitive_button,
            filter_timer_delayed_updates,

            //-------------------------------------------------------------------------------//
            // Contextual menu for the PackFile Contents TreeView.
            //-------------------------------------------------------------------------------//

            packfile_contents_tree_view_context_menu,

            context_menu_add_file,
            context_menu_add_folder,
            context_menu_add_from_packfile,

            context_menu_new_folder,
            context_menu_new_packed_file_anim_pack,
            context_menu_new_packed_file_db,
            context_menu_new_packed_file_loc,
            context_menu_new_packed_file_text,
            context_menu_new_queek_packed_file,

            context_menu_rename,
            context_menu_delete,
            context_menu_extract,
            context_menu_copy_path,

            context_menu_open_decoder,
            context_menu_open_dependency_manager,
            context_menu_open_containing_folder,
            context_menu_open_packfile_settings,
            context_menu_open_with_external_program,
            context_menu_open_notes,

            context_menu_merge_tables,
            context_menu_update_table,
            context_menu_generate_missing_loc_data,

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
        }
    }


    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub unsafe fn add_packedfiles(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths: &[PathBuf],
        paths_packedfile: &[String],
        paths_to_ignore: Option<Vec<PathBuf>>,
        import_tables_from_tsv: bool
    ) {
        let window_was_disabled = !app_ui.main_window().is_enabled();
        if !window_was_disabled {
            app_ui.main_window().set_enabled(false);
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFiles(paths.to_vec(), paths_packedfile.to_vec(), paths_to_ignore));
        let response1 = CentralCommand::recv(&receiver);
        let response2 = CentralCommand::recv(&receiver);
        match response1 {
            Response::VecContainerPath(paths) => {
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);

                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = paths_packedfile.iter().filter_map(|path| {
                    if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                        if packed_file_view.reload(path, pack_file_contents_ui).is_err() {
                            Some(path.to_owned())
                        } else { None }
                    } else { None }
                }).collect::<Vec<String>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response1),
        }

        match response2 {
            Response::Success => {},
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response2),
        }

        // Re-enable the Main Window.
        if !window_was_disabled {
            app_ui.main_window().set_enabled(true);
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_files(pack_file_contents_ui: &Rc<Self>) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&pack_file_contents_ui.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = pack_file_contents_ui.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&pack_file_contents_ui.packfile_contents_tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if pack_file_contents_ui.filter_autoexpand_matches_button.is_checked() {
            pack_file_contents_ui.packfile_contents_tree_view.expand_all();
        }
    }

    /// This function creates the entire "Rename" dialog.
    ///
    ///It returns the new name of the Item, or `None` if the dialog is canceled or closed.
    pub unsafe fn create_rename_dialog(app_ui: &Rc<AppUI>, selected_items: &[ContainerPath]) -> Option<String> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(app_ui.main_window());
        dialog.set_window_title(&qtr("rename_selection"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string(&qtr("rename_selection_instructions"));
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string(&qtr("rename_instructions"));
        instructions_grid.add_widget_5a(&instructions_label, 0, 0, 1, 1);

        let rewrite_sequence_line_edit = QLineEdit::new();
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rename_selection_placeholder"));

        // If we only have one selected item, put his name by default in the rename dialog.
        if selected_items.len() == 1 {
            rewrite_sequence_line_edit.set_text(&QString::from_std_str(selected_items[0].path_raw()));
        }
        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        main_grid.add_widget_5a(&instructions_frame, 0, 0, 1, 2);
        main_grid.add_widget_5a(&rewrite_sequence_line_edit, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 1, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = rewrite_sequence_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(rewrite_sequence_line_edit.text().to_std_string()) }
        } else { None }
    }

    pub unsafe fn extract_packed_files(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths_to_extract: Option<Vec<ContainerPath>>,
        extract_tables_as_tsv: bool,
    ) {

        // Get the currently selected paths (and visible) paths, or the ones received from the function.
        let items_to_extract = match paths_to_extract {
            Some(paths) => paths,
            None => <QBox<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui),
        };

        let extraction_path = match UI_STATE.get_operational_mode() {

            // In MyMod mode we extract directly to the folder of the selected MyMod, keeping the folder structure.
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                let mymods_base_path = setting_path("mymods_base_path");
                if mymods_base_path.is_dir() {

                    // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                    let mut mod_name = mod_name.to_owned();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();

                    let mut assets_folder = mymods_base_path.to_path_buf();
                    assets_folder.push(&game_folder_name);
                    assets_folder.push(&mod_name);
                    assets_folder
                }

                // If there is no MyMod path configured, report it.
                else {
                    return show_dialog(app_ui.main_window(), "MyMod path is not configured. Configure it in the settings and try again.", true);
                }
            }

            // In normal mode, we ask the user to provide us with a path.
            OperationalMode::Normal => {
                let extraction_path = QFileDialog::get_existing_directory_2a(
                    app_ui.main_window(),
                    &qtr("context_menu_extract_packfile"),
                );

                if !extraction_path.is_empty() { PathBuf::from(extraction_path.to_std_string()) }
                else { return }
            }
        };

        // We have to save our data from cache to the backend before extracting it. Otherwise we would extract outdated data.
        if let Err(error) = UI_STATE.get_open_packedfiles()
            .iter()
            .filter(|x| x.get_data_source() == DataSource::PackFile)
            .try_for_each(|packed_file| packed_file.save(&app_ui, &pack_file_contents_ui)) {
            show_dialog(app_ui.main_window(), error, false);
        }

        else {
            let receiver = CENTRAL_COMMAND.send_background(Command::ExtractPackedFiles(items_to_extract, extraction_path, extract_tables_as_tsv));
            app_ui.main_window().set_enabled(false);
            let response = CentralCommand::recv_try(&receiver);
            match response {
                Response::String(result) => show_dialog(app_ui.main_window(), result, true),
                Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
            app_ui.main_window().set_enabled(true);
        }
    }

    pub unsafe fn start_delayed_updates_timer(pack_file_contents_ui: &Rc<Self>,) {
        pack_file_contents_ui.filter_timer_delayed_updates.set_interval(500);
        pack_file_contents_ui.filter_timer_delayed_updates.start_0a();
    }
}
