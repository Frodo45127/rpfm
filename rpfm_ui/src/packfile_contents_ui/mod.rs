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
Module with all the code related to the main `PackFileContentsUI`.
!*/

use qt_widgets::QAction;
use qt_widgets::QCheckBox;
use qt_widgets::QDialog;
use qt_widgets::{q_dialog_button_box::StandardButton, QDialogButtonBox};
use qt_widgets::QDockWidget;
use qt_widgets::QFileDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::CaseSensitivity;
use qt_core::DockWidgetArea;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QRegExp;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;

use anyhow::Result;
use getset::Getters;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::files::{ContainerPath, pack::RESERVED_NAME_NOTES};

use rpfm_ui_common::locale::qtr;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::packedfile_views::DataSource;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::settings_ui::backend::*;
use crate::utils::*;
use crate::ui_state::OperationalMode;
use crate::UI_STATE;

pub mod connections;
pub mod slots;
pub mod tips;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/filterable_tree_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/filterable_tree_dock_widget.ui";

const RENAME_MOVE_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/rename_move_dialog.ui";
const RENAME_MOVE_VIEW_RELEASE: &str = "ui/rename_move_dialog.ui";

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
    packfile_contents_dock_widget: QPtr<QDockWidget>,
    //packfile_contents_pined_table: Ptr<QTableView>,
    packfile_contents_tree_view: QPtr<QTreeView>,
    packfile_contents_tree_model_filter: QBox<QSortFilterProxyModel>,
    packfile_contents_tree_model: QBox<QStandardItemModel>,
    filter_line_edit: QPtr<QLineEdit>,
    filter_autoexpand_matches_button: QPtr<QToolButton>,
    filter_case_sensitive_button: QPtr<QToolButton>,
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
    context_menu_new_packed_file_portrait_settings: QPtr<QAction>,
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
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let packfile_contents_dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();
        let packfile_contents_dock_inner_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "inner_widget")?;
        let packfile_contents_tree_view_placeholder: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "tree_view")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let filter_autoexpand_matches_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_autoexpand_matches_button")?;
        let filter_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "filter_case_sensitive_button")?;

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        app_ui.main_window().add_dock_widget_2a(DockWidgetArea::LeftDockWidgetArea, &packfile_contents_dock_widget);
        packfile_contents_dock_widget.set_window_title(&qtr("gen_loc_packfile_contents"));
        packfile_contents_dock_widget.set_object_name(&QString::from_std_str("packfile_contents_dock"));

        // Create and configure the `TreeView` itself.
        let packfile_contents_tree_view = new_packed_file_treeview_safe(packfile_contents_dock_inner_widget.static_upcast());
        let packfile_contents_tree_model = new_packed_file_model_safe();
        let packfile_contents_tree_model_filter = new_treeview_filter_safe(packfile_contents_tree_view.static_upcast());
        packfile_contents_tree_model_filter.set_source_model(&packfile_contents_tree_model);
        packfile_contents_tree_model.set_parent(&packfile_contents_tree_view);
        packfile_contents_tree_view.set_model(&packfile_contents_tree_model_filter);

        let layout = packfile_contents_dock_inner_widget.layout().static_downcast::<QGridLayout>();
        layout.replace_widget_2a(packfile_contents_tree_view_placeholder.as_ptr(), packfile_contents_tree_view.as_ptr());

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
        filter_timer_delayed_updates.set_single_shot(true);
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));

        //-------------------------------------------------------------------------------//
        // Contextual menu for the PackFile Contents TreeView.
        //-------------------------------------------------------------------------------//

        // Populate the `Contextual Menu` for the `PackFile` TreeView.
        let packfile_contents_tree_view_context_menu = QMenu::from_q_widget(&packfile_contents_dock_inner_widget);
        let menu_add = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_add"));
        let menu_create = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_create"));
        let menu_open = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_open"));

        let context_menu_add_file = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_file", "context_menu_add_file", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_add_folder = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_folder", "context_menu_add_folder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_add_from_packfile = add_action_to_menu(&menu_add.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "add_from_pack", "context_menu_add_from_packfile", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_folder = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_folder", "context_menu_new_folder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_packed_file_anim_pack = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_animpack", "context_menu_new_packed_file_anim_pack", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_packed_file_db = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_db", "context_menu_new_packed_file_db", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_packed_file_loc = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_loc", "context_menu_new_packed_file_loc", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_packed_file_portrait_settings = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_portrait_settings", "context_menu_new_packed_file_portrait_settings", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_packed_file_text = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_text", "context_menu_new_packed_file_text", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_new_queek_packed_file = add_action_to_menu(&menu_create.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "new_quick_file", "context_menu_new_queek_packed_file", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_rename = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "rename", "context_menu_move", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_delete = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "delete", "context_menu_delete", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_extract = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "extract", "context_menu_extract", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_copy_path = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "copy_path", "context_menu_copy_path", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_decoder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_decoder", "context_menu_open_decoder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_dependency_manager = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_dependency_manager", "context_menu_open_dependency_manager", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_containing_folder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_containing_folder", "context_menu_open_containing_folder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_packfile_settings = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_settings", "context_menu_open_packfile_settings", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_with_external_program = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_external_program", "context_menu_open_with_external_program", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_notes = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_notes", "context_menu_open_notes", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_merge_tables = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "merge_files", "context_menu_merge_tables", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_update_table = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "update_files", "context_menu_update_table", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_generate_missing_loc_data = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "generate_missing_loc_data", "context_menu_generate_missing_loc_data", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));

        let packfile_contents_tree_view_expand_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "expand_all", "treeview_expand_all", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let packfile_contents_tree_view_collapse_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "collapse_all", "treeview_collapse_all", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));

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
        context_menu_new_packed_file_portrait_settings.set_enabled(false);
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
        Ok(Self {

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
            context_menu_new_packed_file_portrait_settings,
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
        })
    }


    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub unsafe fn add_files(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths: &[PathBuf],
        paths_in_container: &[ContainerPath],
        paths_to_ignore: Option<Vec<PathBuf>>,
    ) {
        let window_was_disabled = !app_ui.main_window().is_enabled();
        if !window_was_disabled {
            app_ui.toggle_main_window(false);
        }

        let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFiles(paths.to_vec(), paths_in_container.to_vec(), paths_to_ignore));
        let response1 = CentralCommand::recv(&receiver);
        let response2 = CentralCommand::recv(&receiver);
        match response1 {
            Response::VecContainerPath(paths) => {
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);

                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = UI_STATE.set_open_packedfiles()
                    .iter_mut()
                    .filter(|view| view.data_source() == DataSource::PackFile && (paths.iter().any(|path| path.path_raw() == *view.path_read() || *view.path_read() == RESERVED_NAME_NOTES)))
                    .filter_map(|view| if view.reload(&view.path_copy(), pack_file_contents_ui).is_err() { Some(view.path_copy()) } else { None })
                    .collect::<Vec<_>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response1:?}"),
        }

        match response2 {
            Response::Success => {},
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response2:?}"),
        }

        // Re-enable the Main Window.
        if !window_was_disabled {
            app_ui.toggle_main_window(true);
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
    pub unsafe fn create_rename_dialog(app_ui: &Rc<AppUI>, selected_items: &[ContainerPath]) -> Result<Option<(String, bool)>> {

        // Create and configure the dialog.
        let template_path = if cfg!(debug_assertions) { RENAME_MOVE_VIEW_DEBUG } else { RENAME_MOVE_VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let dialog: QPtr<QDialog> = main_widget.static_downcast();
        dialog.set_window_title(&qtr("rename_move_selection"));

        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let move_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "move_checkbox")?;
        let rewrite_sequence_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        instructions_label.set_text(&qtr("rename_move_instructions"));
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rename_move_selection_placeholder"));
        rewrite_sequence_line_edit.set_focus_0a();
        move_checkbox.set_text(&qtr("rename_move_checkbox"));

        // Remember the last status of the move checkbox.
        if setting_variant_from_q_setting(&settings(), "move_checkbox_status").can_convert(1) {
            move_checkbox.set_checked(setting_bool("move_checkbox_status"));
        }

        match selected_items.len().cmp(&1) {

            // If we only have one selected item, put its path in the line edit.
            Ordering::Equal => if !move_checkbox.is_checked() {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(selected_items[0].path_raw().split('/').last().unwrap()));
            } else {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(selected_items[0].path_raw()));
            }

            // If we have multiple items selected, things get complicated.
            // We need to check if all of them are within the same exact folder to check if we can allow full-path move or not.
            Ordering::Greater => {

                let last_separator = selected_items[0].path_raw().rfind('/');
                let start_path = match last_separator {
                    Some(last_separator) => &selected_items[0].path_raw()[..last_separator + 1],
                    None => ""
                };

                // Branch 1: all items in the same folder. We allow full-path replace, and by default we change the file name to {x}.
                if selected_items.iter().all(|item| item.path_raw().rfind('/') == last_separator && ((!start_path.is_empty() && item.path_raw().starts_with(start_path)) || start_path.is_empty())) {

                    if !move_checkbox.is_checked() {
                        rewrite_sequence_line_edit.set_text(&QString::from_std_str("{x}"));
                    } else {
                        let new_path = format!("{start_path}{{x}}");
                        rewrite_sequence_line_edit.set_text(&QString::from_std_str(new_path));
                    }
                }

                // Branch 2: items are in different folders. We need to disable the checkbox and allow to only replace the name.
                else {
                    move_checkbox.set_enabled(false);
                    rewrite_sequence_line_edit.set_text(&QString::from_std_str("{x}"));
                }
            }
            Ordering::Less => unreachable!("create rename dialog"),
        }

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // TODO: Validator to ensure than on multifile edit {x} is used as long as more than one file shares folder with another one.
        // TODO2: Make sure this can't be triggered if you select a file/folder, and a parent of said file/folder.

        Ok(
            if dialog.exec() == 1 {
                set_setting_bool("move_checkbox_status", move_checkbox.is_checked());

                let new_text = rewrite_sequence_line_edit.text().to_std_string();
                if new_text.is_empty() {
                    None
                } else {
                    Some((rewrite_sequence_line_edit.text().to_std_string(), move_checkbox.is_enabled() && move_checkbox.is_checked())) }
            } else { None }
        )
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
            None => <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui),
        };

        let extraction_path = match UI_STATE.get_operational_mode() {

            // In MyMod mode we extract directly to the folder of the selected MyMod, keeping the folder structure.
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                let mymods_base_path = setting_path(MYMOD_BASE_PATH);
                if mymods_base_path.is_dir() {

                    // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                    let mut mod_name = mod_name.to_owned();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();

                    let mut assets_folder = mymods_base_path;
                    assets_folder.push(game_folder_name);
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
            .filter(|x| x.data_source() == DataSource::PackFile)
            .try_for_each(|packed_file| packed_file.save(app_ui, pack_file_contents_ui)) {
            show_dialog(app_ui.main_window(), error, false);
        }

        else {
            let mut paths_by_source = BTreeMap::new();
            paths_by_source.insert(DataSource::PackFile, items_to_extract);
            let receiver = CENTRAL_COMMAND.send_background(Command::ExtractPackedFiles(paths_by_source, extraction_path, extract_tables_as_tsv));
            app_ui.toggle_main_window(false);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::StringVecPathBuf(result, _) => show_message_info(app_ui.message_widget(), result),
                Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
            app_ui.toggle_main_window(true);
        }
    }

    pub unsafe fn start_delayed_updates_timer(pack_file_contents_ui: &Rc<Self>,) {
        pack_file_contents_ui.filter_timer_delayed_updates.set_interval(500);
        pack_file_contents_ui.filter_timer_delayed_updates.start_0a();
    }
}
