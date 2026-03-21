//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QActionGroup;
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

use rpfm_ipc::MYMOD_BASE_PATH;
use rpfm_ipc::helpers::DataSource;

use rpfm_lib::files::{ContainerPath, pack::RESERVED_NAME_NOTES};

use rpfm_ui_common::utils::{find_widget, load_template};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::settings_ui::backend::{settings_bool, settings_path_buf, settings_set_bool};
use crate::ui_state::OperationalMode;
use crate::UI_STATE;
use crate::utils::{add_action_to_menu, qtr, show_dialog, show_message_info};

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
    context_menu_copy: QPtr<QAction>,
    context_menu_cut: QPtr<QAction>,
    context_menu_paste: QPtr<QAction>,
    context_menu_duplicate: QPtr<QAction>,
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
    // Pack-level actions (shown when pack root is right-clicked).
    //-------------------------------------------------------------------------------//
    context_menu_install: QPtr<QAction>,
    context_menu_uninstall: QPtr<QAction>,

    context_menu_packfile_type_menu: QPtr<QMenu>,
    context_menu_packfile_type_group: QBox<QActionGroup>,
    context_menu_packfile_type_boot: QPtr<QAction>,
    context_menu_packfile_type_release: QPtr<QAction>,
    context_menu_packfile_type_patch: QPtr<QAction>,
    context_menu_packfile_type_mod: QPtr<QAction>,
    context_menu_packfile_type_movie: QPtr<QAction>,
    context_menu_index_includes_timestamp: QPtr<QAction>,
    context_menu_header_is_extended: QPtr<QAction>,
    context_menu_index_is_encrypted: QPtr<QAction>,
    context_menu_data_is_encrypted: QPtr<QAction>,

    context_menu_compression_menu: QPtr<QMenu>,
    context_menu_compression_group: QBox<QActionGroup>,
    context_menu_compression_none: QPtr<QAction>,
    context_menu_compression_lzma1: QPtr<QAction>,
    context_menu_compression_lz4: QPtr<QAction>,
    context_menu_compression_zstd: QPtr<QAction>,

    context_menu_optimize_packfile: QPtr<QAction>,
    context_menu_rescue_packfile: QPtr<QAction>,
    context_menu_build_starpos: QPtr<QAction>,
    context_menu_patch_siege_ai: QPtr<QAction>,
    context_menu_live_export: QPtr<QAction>,
    context_menu_pack_map: QPtr<QAction>,
    context_menu_update_anim_ids: QPtr<QAction>,

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
        if settings_bool("packfile_treeview_resize_to_fit") {
            //packfile_contents_tree_view.set_size_adjust_policy(qt_widgets::q_abstract_scroll_area::SizeAdjustPolicy::AdjustToContents);
            //packfile_contents_tree_view.horizontal_scroll_bar().set_disabled(true);
            //packfile_contents_tree_view.set_horizontal_scroll_bar_policy(qt_core::ScrollBarPolicy::ScrollBarAlwaysOff);
            //packfile_contents_tree_view.header().set_section_resize_mode_1a(ResizeMode::ResizeToContents);
            //packfile_contents_tree_view.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Maximum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_inner_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Minimum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::MinimumExpanding, qt_widgets::q_size_policy::Policy::Maximum);

        }

        packfile_contents_tree_view.set_drag_enabled(settings_bool("enable_pack_contents_drag_and_drop"));

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
        let context_menu_copy = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "copy", "context_menu_copy", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_cut = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "cut", "context_menu_cut", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_paste = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "paste", "context_menu_paste", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_duplicate = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "duplicate", "context_menu_duplicate", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_decoder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_decoder", "context_menu_open_decoder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_dependency_manager = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_dependency_manager", "context_menu_open_dependency_manager", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_containing_folder = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_containing_folder", "context_menu_open_containing_folder", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_packfile_settings = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_settings", "context_menu_open_packfile_settings", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_with_external_program = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_in_external_program", "context_menu_open_with_external_program", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_open_notes = add_action_to_menu(&menu_open.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "open_pack_notes", "context_menu_open_notes", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_merge_tables = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "merge_files", "context_menu_merge_tables", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_update_table = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "update_files", "context_menu_update_table", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let context_menu_generate_missing_loc_data = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "generate_missing_loc_data", "context_menu_generate_missing_loc_data", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));

        //-------------------------------------------------------------------------------//
        // Pack-level actions (shown when pack root is right-clicked).
        //-------------------------------------------------------------------------------//

        packfile_contents_tree_view_context_menu.add_separator();

        let context_menu_install = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("packfile_install"));
        let context_menu_uninstall = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("packfile_uninstall"));

        packfile_contents_tree_view_context_menu.add_separator();

        // Pack Type submenu.
        let context_menu_packfile_type_menu = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("change_packfile_type"));
        let menu_packfile_type = &context_menu_packfile_type_menu;
        let context_menu_packfile_type_boot = menu_packfile_type.add_action_q_string(&qtr("packfile_type_boot"));
        let context_menu_packfile_type_release = menu_packfile_type.add_action_q_string(&qtr("packfile_type_release"));
        let context_menu_packfile_type_patch = menu_packfile_type.add_action_q_string(&qtr("packfile_type_patch"));
        let context_menu_packfile_type_mod = menu_packfile_type.add_action_q_string(&qtr("packfile_type_mod"));
        let context_menu_packfile_type_movie = menu_packfile_type.add_action_q_string(&qtr("packfile_type_movie"));

        let context_menu_packfile_type_group = QActionGroup::new(menu_packfile_type);
        context_menu_packfile_type_group.add_action_q_action(&context_menu_packfile_type_boot);
        context_menu_packfile_type_group.add_action_q_action(&context_menu_packfile_type_release);
        context_menu_packfile_type_group.add_action_q_action(&context_menu_packfile_type_patch);
        context_menu_packfile_type_group.add_action_q_action(&context_menu_packfile_type_mod);
        context_menu_packfile_type_group.add_action_q_action(&context_menu_packfile_type_movie);
        context_menu_packfile_type_boot.set_checkable(true);
        context_menu_packfile_type_release.set_checkable(true);
        context_menu_packfile_type_patch.set_checkable(true);
        context_menu_packfile_type_mod.set_checkable(true);
        context_menu_packfile_type_movie.set_checkable(true);

        // Header flags (individual checkable items).
        menu_packfile_type.add_separator();
        let context_menu_index_includes_timestamp = menu_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_includes_timestamp"));
        let context_menu_header_is_extended = menu_packfile_type.add_action_q_string(&qtr("change_packfile_type_header_is_extended"));
        let context_menu_index_is_encrypted = menu_packfile_type.add_action_q_string(&qtr("change_packfile_type_index_is_encrypted"));
        let context_menu_data_is_encrypted = menu_packfile_type.add_action_q_string(&qtr("change_packfile_type_data_is_encrypted"));
        context_menu_index_includes_timestamp.set_checkable(true);
        context_menu_header_is_extended.set_checkable(true);
        context_menu_index_is_encrypted.set_checkable(true);
        context_menu_data_is_encrypted.set_checkable(true);
        context_menu_header_is_extended.set_enabled(false);
        context_menu_index_is_encrypted.set_enabled(false);
        context_menu_data_is_encrypted.set_enabled(false);

        // Compression Format submenu.
        let context_menu_compression_menu = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("compression_format"));
        let menu_compression_format = &context_menu_compression_menu;
        let context_menu_compression_none = menu_compression_format.add_action_q_string(&qtr("compression_format_none"));
        let context_menu_compression_lzma1 = menu_compression_format.add_action_q_string(&qtr("compression_format_lzma1"));
        let context_menu_compression_lz4 = menu_compression_format.add_action_q_string(&qtr("compression_format_lz4"));
        let context_menu_compression_zstd = menu_compression_format.add_action_q_string(&qtr("compression_format_zstd"));

        let context_menu_compression_group = QActionGroup::new(menu_compression_format);
        context_menu_compression_group.add_action_q_action(&context_menu_compression_none);
        context_menu_compression_group.add_action_q_action(&context_menu_compression_lzma1);
        context_menu_compression_group.add_action_q_action(&context_menu_compression_lz4);
        context_menu_compression_group.add_action_q_action(&context_menu_compression_zstd);
        context_menu_compression_none.set_checkable(true);
        context_menu_compression_lzma1.set_checkable(true);
        context_menu_compression_lz4.set_checkable(true);
        context_menu_compression_zstd.set_checkable(true);

        packfile_contents_tree_view_context_menu.add_separator();

        // Special stuff actions (pack-level operations).
        let context_menu_optimize_packfile = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_optimize_packfile"));
        let context_menu_rescue_packfile = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_rescue_packfile"));
        let context_menu_build_starpos = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_build_starpos"));
        let context_menu_patch_siege_ai = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_patch_siege_ai"));
        let context_menu_live_export = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_live_export"));
        let context_menu_pack_map = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_pack_map"));
        let context_menu_update_anim_ids = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("special_stuff_update_anim_ids"));

        packfile_contents_tree_view_context_menu.add_separator();

        let packfile_contents_tree_view_expand_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "expand_all", "treeview_expand_all", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));
        let packfile_contents_tree_view_collapse_all = add_action_to_menu(&packfile_contents_tree_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "pack_tree_context_menu", "collapse_all", "treeview_collapse_all", Some(packfile_contents_tree_view.static_upcast::<qt_widgets::QWidget>()));

        // Configure the `Contextual Menu` for the `PackFile` TreeView.
        packfile_contents_tree_view_context_menu.insert_separator(menu_open.menu_action());
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_rename);
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_copy);
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
        context_menu_copy.set_enabled(false);
        context_menu_cut.set_enabled(false);
        context_menu_paste.set_enabled(false);
        context_menu_duplicate.set_enabled(false);
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
            context_menu_copy,
            context_menu_cut,
            context_menu_paste,
            context_menu_duplicate,

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
            // Pack-level actions.
            //-------------------------------------------------------------------------------//
            context_menu_install,
            context_menu_uninstall,

            context_menu_packfile_type_menu,
            context_menu_packfile_type_group,
            context_menu_packfile_type_boot,
            context_menu_packfile_type_release,
            context_menu_packfile_type_patch,
            context_menu_packfile_type_mod,
            context_menu_packfile_type_movie,
            context_menu_index_includes_timestamp,
            context_menu_header_is_extended,
            context_menu_index_is_encrypted,
            context_menu_data_is_encrypted,

            context_menu_compression_menu,
            context_menu_compression_group,
            context_menu_compression_none,
            context_menu_compression_lzma1,
            context_menu_compression_lz4,
            context_menu_compression_zstd,

            context_menu_optimize_packfile,
            context_menu_rescue_packfile,
            context_menu_build_starpos,
            context_menu_patch_siege_ai,
            context_menu_live_export,
            context_menu_pack_map,
            context_menu_update_anim_ids,

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

        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::AddPackedFiles(pack_key.clone(), paths.to_vec(), paths_in_container.to_vec(), paths_to_ignore));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::VecContainerPathOptionString(paths, error) => {
                if !paths.is_empty() {
                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile, &pack_key);

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

                if let Some(error) = error {
                    show_dialog(app_ui.main_window(), error, false);
                }
            }

            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        move_checkbox.set_checked(settings_bool("move_checkbox_status"));

        match selected_items.len().cmp(&1) {

            // If we only have one selected item, put its path in the line edit.
            Ordering::Equal => if !move_checkbox.is_checked() {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(selected_items[0].path_raw().split('/').next_back().unwrap()));
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
                let _ = settings_set_bool("move_checkbox_status", move_checkbox.is_checked());

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
                let mymods_base_path = settings_path_buf(MYMOD_BASE_PATH);
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
            let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
            let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::ExtractPackedFiles(pack_key, paths_by_source, extraction_path, extract_tables_as_tsv));
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

    pub unsafe fn start_delayed_updates_timer(pack_file_contents_ui: &Rc<Self>,) {
        pack_file_contents_ui.filter_timer_delayed_updates.set_interval(500);
        pack_file_contents_ui.filter_timer_delayed_updates.start_0a();
    }

    /// Get the pack key from the current tree view selection, or fall back to the first editable pack root.
    ///
    /// This is the primary way UI code obtains the pack key to pass to server commands.
    /// When multiple packs are open, it derives the key from the selected item's root node.
    /// If nothing is selected, it returns the first editable pack root's key.
    pub unsafe fn pack_key_from_selection_or_first(&self) -> Option<String> {

        // First, try to get it from the current tree selection.
        let selection = self.packfile_contents_tree_view.selection_model().selection().indexes();
        if selection.count_0a() > 0 {
            let index = selection.at(0);
            let filter: qt_core::QPtr<qt_core::QSortFilterProxyModel> = self.packfile_contents_tree_view.model().static_downcast();
            let source_index = filter.map_to_source(index);

            if let Some(key) = self.packfile_contents_tree_view.get_pack_key_from_index(source_index) {
                return Some(key);
            }
        }

        // Fallback: return the first editable pack root's key.
        for row in 0..self.packfile_contents_tree_model.row_count_0a() {
            let item = self.packfile_contents_tree_model.item_1a(row);
            let root_type = item.data_1a(rpfm_ui_common::ROOT_NODE_TYPE).to_int_0a();
            if root_type == rpfm_ui_common::ROOT_NODE_TYPE_EDITABLE_PACKFILE {
                let variant = item.data_1a(rpfm_ui_common::ITEM_PACK_KEY);
                if variant.is_valid() && !variant.is_null() {
                    let key = variant.to_string().to_std_string();
                    if !key.is_empty() {
                        return Some(key);
                    }
                }
            }
        }

        None
    }

    /// Returns the selected items grouped by their pack key.
    ///
    /// Each selected item is resolved to its root pack node to determine the pack key,
    /// then grouped into a `BTreeMap<String, Vec<ContainerPath>>`.
    pub unsafe fn selected_items_grouped_by_pack_key(&self) -> BTreeMap<String, Vec<ContainerPath>> {
        let tree_view = &self.packfile_contents_tree_view;
        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<qt_gui::QStandardItemModel> = filter.source_model().static_downcast();

        let indexes_visual = tree_view.selection_model().selection().indexes();
        let mut result: BTreeMap<String, Vec<ContainerPath>> = BTreeMap::new();

        for i in 0..indexes_visual.count_0a() {
            let source_index = filter.map_to_source(indexes_visual.at(i));
            let item = model.item_from_index(&source_index);
            if item.is_null() {
                continue;
            }

            let container_path = <QPtr<QTreeView> as PackTree>::get_type_from_item(item, &model);
            let pack_key = tree_view.get_pack_key_from_index(model.index_from_item(item)).unwrap_or_default();
            result.entry(pack_key).or_default().push(container_path);
        }

        result
    }
}
