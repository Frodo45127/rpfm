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
Module with all the code related to the main `PackFileContentsUI`.
!*/

use qt_widgets::q_abstract_item_view::SelectionMode;
use qt_widgets::QAction;
use qt_widgets::QDockWidget;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::{ContextMenuPolicy, DockWidgetArea};
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;

use cpp_core::Ptr;

use rpfm_lib::SETTINGS;

use crate::ffi::{new_packed_file_model_safe, new_treeview_filter_safe};
use crate::locale::qtr;
use crate::utils::create_grid_layout;

pub mod connections;
pub mod extra;
pub mod shortcuts;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the PackFile Contents panel.
pub struct PackFileContentsUI {

    //-------------------------------------------------------------------------------//
    // `PackFile Contents` Dock Widget.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_dock_widget: QBox<QDockWidget>,
    //pub packfile_contents_pined_table: Ptr<QTableView>,
    pub packfile_contents_tree_view: QBox<QTreeView>,
    pub packfile_contents_tree_model_filter: QBox<QSortFilterProxyModel>,
    pub packfile_contents_tree_model: QBox<QStandardItemModel>,
    pub filter_line_edit: QBox<QLineEdit>,
    pub filter_autoexpand_matches_button: QBox<QPushButton>,
    pub filter_case_sensitive_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // Contextual menu for the PackFile Contents TreeView.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_tree_view_context_menu: QBox<QMenu>,
    pub context_menu_add_file: QPtr<QAction>,
    pub context_menu_add_folder: QPtr<QAction>,
    pub context_menu_add_from_packfile: QPtr<QAction>,
    pub context_menu_new_folder: QPtr<QAction>,
    pub context_menu_new_packed_file_db: QPtr<QAction>,
    pub context_menu_new_packed_file_loc: QPtr<QAction>,
    pub context_menu_new_packed_file_text: QPtr<QAction>,
    pub context_menu_new_queek_packed_file: QPtr<QAction>,
    pub context_menu_mass_import_tsv: QPtr<QAction>,
    pub context_menu_mass_export_tsv: QPtr<QAction>,
    pub context_menu_rename: QPtr<QAction>,
    pub context_menu_delete: QPtr<QAction>,
    pub context_menu_extract: QPtr<QAction>,
    pub context_menu_copy_path: QPtr<QAction>,
    pub context_menu_open_decoder: QPtr<QAction>,
    pub context_menu_open_dependency_manager: QPtr<QAction>,
    pub context_menu_open_containing_folder: QPtr<QAction>,
    pub context_menu_open_with_external_program: QPtr<QAction>,
    pub context_menu_open_notes: QPtr<QAction>,
    pub context_menu_merge_tables: QPtr<QAction>,
    pub context_menu_update_table: QPtr<QAction>,

    //-------------------------------------------------------------------------------//
    // Actions not in the UI.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_tree_view_expand_all: QBox<QAction>,
    pub packfile_contents_tree_view_collapse_all: QBox<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function creates an entire `PackFileContentsUI` struct.
    pub unsafe fn new(main_window: Ptr<QMainWindow>) -> Self {

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'TreeView` Dock Widget and all his contents.
        let packfile_contents_dock_widget = QDockWidget::from_q_widget(main_window);
        let packfile_contents_dock_inner_widget = QWidget::new_0a();
        let packfile_contents_dock_layout = create_grid_layout(packfile_contents_dock_inner_widget.static_upcast());
        packfile_contents_dock_widget.set_widget(&packfile_contents_dock_inner_widget);
        main_window.add_dock_widget_2a(DockWidgetArea::LeftDockWidgetArea, &packfile_contents_dock_widget);
        packfile_contents_dock_widget.set_window_title(&qtr("gen_loc_packfile_contents"));

        // Create and configure the `TreeView` itself.
        let packfile_contents_tree_view = QTreeView::new_0a();
        let packfile_contents_tree_model = new_packed_file_model_safe();
        let packfile_contents_tree_model_filter = new_treeview_filter_safe(packfile_contents_dock_widget.static_upcast());
        packfile_contents_tree_model_filter.set_source_model(&packfile_contents_tree_model);
        packfile_contents_tree_view.set_model(&packfile_contents_tree_model_filter);
        packfile_contents_tree_view.set_header_hidden(true);
        packfile_contents_tree_view.set_animated(true);
        packfile_contents_tree_view.set_uniform_row_heights(true);
        packfile_contents_tree_view.set_selection_mode(SelectionMode::ExtendedSelection);
        packfile_contents_tree_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        packfile_contents_tree_view.set_expands_on_double_click(true);
        packfile_contents_tree_view.header().set_stretch_last_section(false);

        // Not yet working.
        if SETTINGS.read().unwrap().settings_bool["packfile_treeview_resize_to_fit"] {
            //packfile_contents_tree_view.set_size_adjust_policy(qt_widgets::q_abstract_scroll_area::SizeAdjustPolicy::AdjustToContents);
            //packfile_contents_tree_view.horizontal_scroll_bar().set_disabled(true);
            //packfile_contents_tree_view.set_horizontal_scroll_bar_policy(qt_core::ScrollBarPolicy::ScrollBarAlwaysOff);
            //packfile_contents_tree_view.header().set_section_resize_mode_1a(ResizeMode::ResizeToContents);
            //packfile_contents_tree_view.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Maximum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_inner_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::Minimum, qt_widgets::q_size_policy::Policy::Maximum);
            //packfile_contents_dock_widget.set_size_policy_2a(qt_widgets::q_size_policy::Policy::MinimumExpanding, qt_widgets::q_size_policy::Policy::Maximum);

        }

        // Create and configure the widgets to control the `TreeView`s filter.
        let filter_line_edit = QLineEdit::new();
        let filter_autoexpand_matches_button = QPushButton::from_q_string(&qtr("treeview_autoexpand"));
        let filter_case_sensitive_button = QPushButton::from_q_string(&qtr("treeview_aai"));
        filter_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
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
        let packfile_contents_tree_view_context_menu = QMenu::new();
        let menu_add = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_add"));
        let menu_create = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_create"));
        let menu_open = packfile_contents_tree_view_context_menu.add_menu_q_string(&qtr("context_menu_open"));

        let context_menu_add_file = menu_add.add_action_q_string(&qtr("context_menu_add_file"));
        let context_menu_add_folder = menu_add.add_action_q_string(&qtr("context_menu_add_folder"));
        let context_menu_add_from_packfile = menu_add.add_action_q_string(&qtr("context_menu_add_from_packfile"));
        let context_menu_new_folder = menu_create.add_action_q_string(&qtr("context_menu_new_folder"));
        let context_menu_new_packed_file_db = menu_create.add_action_q_string(&qtr("context_menu_new_packed_file_db"));
        let context_menu_new_packed_file_loc = menu_create.add_action_q_string(&qtr("context_menu_new_packed_file_loc"));
        let context_menu_new_packed_file_text = menu_create.add_action_q_string(&qtr("context_menu_new_packed_file_text"));
        let context_menu_new_queek_packed_file = menu_create.add_action_q_string(&qtr("context_menu_new_queek_packed_file"));
        let context_menu_mass_import_tsv = menu_create.add_action_q_string(&qtr("context_menu_mass_import_tsv"));
        let context_menu_mass_export_tsv = menu_create.add_action_q_string(&qtr("context_menu_mass_export_tsv"));
        let context_menu_rename = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_rename"));
        let context_menu_delete = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_delete"));
        let context_menu_extract = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_extract"));
        let context_menu_copy_path = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_copy_path"));
        let context_menu_open_decoder = menu_open.add_action_q_string(&qtr("context_menu_open_decoder"));
        let context_menu_open_dependency_manager = menu_open.add_action_q_string(&qtr("context_menu_open_dependency_manager"));
        let context_menu_open_containing_folder = menu_open.add_action_q_string(&qtr("context_menu_open_containing_folder"));
        let context_menu_open_with_external_program = menu_open.add_action_q_string(&qtr("context_menu_open_with_external_program"));
        let context_menu_open_notes = menu_open.add_action_q_string(&qtr("context_menu_open_notes"));
        let context_menu_merge_tables = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_merge_tables"));
        let context_menu_update_table = packfile_contents_tree_view_context_menu.add_action_q_string(&qtr("context_menu_update_table"));
        let packfile_contents_tree_view_expand_all = QAction::from_q_string(&qtr("treeview_expand_all"));
        let packfile_contents_tree_view_collapse_all = QAction::from_q_string(&qtr("treeview_collapse_all"));

        // Configure the `Contextual Menu` for the `PackFile` TreeView.
        packfile_contents_tree_view_context_menu.insert_separator(&menu_open.menu_action());
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_rename);
        packfile_contents_tree_view_context_menu.insert_separator(&context_menu_merge_tables);

        // Disable all the Contextual Menu actions by default.
        context_menu_add_file.set_enabled(false);
        context_menu_add_folder.set_enabled(false);
        context_menu_add_from_packfile.set_enabled(false);
        context_menu_new_folder.set_enabled(false);
        context_menu_new_packed_file_db.set_enabled(false);
        context_menu_new_packed_file_loc.set_enabled(false);
        context_menu_new_packed_file_text.set_enabled(false);
        context_menu_new_queek_packed_file.set_enabled(false);
        context_menu_mass_import_tsv.set_enabled(false);
        context_menu_mass_export_tsv.set_enabled(false);
        context_menu_delete.set_enabled(false);
        context_menu_rename.set_enabled(false);
        context_menu_extract.set_enabled(false);
        context_menu_copy_path.set_enabled(false);
        context_menu_open_decoder.set_enabled(false);
        context_menu_open_dependency_manager.set_enabled(false);
        context_menu_open_containing_folder.set_enabled(false);
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

            //-------------------------------------------------------------------------------//
            // Contextual menu for the PackFile Contents TreeView.
            //-------------------------------------------------------------------------------//

            packfile_contents_tree_view_context_menu,

            context_menu_add_file,
            context_menu_add_folder,
            context_menu_add_from_packfile,

            context_menu_new_folder,
            context_menu_new_packed_file_loc,
            context_menu_new_packed_file_db,
            context_menu_new_packed_file_text,
            context_menu_new_queek_packed_file,

            context_menu_mass_import_tsv,
            context_menu_mass_export_tsv,

            context_menu_rename,
            context_menu_delete,
            context_menu_extract,
            context_menu_copy_path,

            context_menu_open_decoder,
            context_menu_open_dependency_manager,
            context_menu_open_containing_folder,
            context_menu_open_with_external_program,
            context_menu_open_notes,

            context_menu_merge_tables,
            context_menu_update_table,

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
        }
    }
}
