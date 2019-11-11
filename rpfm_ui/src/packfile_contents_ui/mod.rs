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
Module with all the code related to the main `PackFileContentsUI`.
!*/

use qt_widgets::abstract_item_view::SelectionMode;
use qt_widgets::action::Action;
use qt_widgets::dock_widget::DockWidget;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::menu::Menu;
use qt_widgets::push_button::PushButton;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::object::Object;
use qt_core::qt::{ContextMenuPolicy, DockWidgetArea};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;

use crate::ffi::new_treeview_filter;
use crate::QString;
use crate::utils::create_grid_layout_unsafe;

pub mod connections;
pub mod extra;
pub mod shortcuts;
pub mod slots;
pub mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the PackFile Contents panel.
#[derive(Copy, Clone)]
pub struct PackFileContentsUI {

    //-------------------------------------------------------------------------------//
    // `PackFile Contents` Dock Widget.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_dock_widget: *mut DockWidget,
    //pub packfile_contents_pined_table: *mut TableView,
    pub packfile_contents_tree_view: *mut TreeView,
    pub packfile_contents_tree_model_filter: *mut SortFilterProxyModel,
    pub packfile_contents_tree_model: *mut StandardItemModel,
    pub filter_line_edit: *mut LineEdit,
    pub filter_autoexpand_matches_button: *mut PushButton,
    pub filter_case_sensitive_button: *mut PushButton,

    //-------------------------------------------------------------------------------//
    // Contextual menu for the PackFile Contents TreeView.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_tree_view_context_menu: *mut Menu,
    pub context_menu_add_file: *mut Action,
    pub context_menu_add_folder: *mut Action,
    pub context_menu_add_from_packfile: *mut Action,
    pub context_menu_create_folder: *mut Action,
    pub context_menu_create_db: *mut Action,
    pub context_menu_create_loc: *mut Action,
    pub context_menu_create_text: *mut Action,
    pub context_menu_mass_import_tsv: *mut Action,
    pub context_menu_mass_export_tsv: *mut Action,
    pub context_menu_rename: *mut Action,
    pub context_menu_delete: *mut Action,
    pub context_menu_extract: *mut Action,
    pub context_menu_open_decoder: *mut Action,
    pub context_menu_open_dependency_manager: *mut Action,
    pub context_menu_open_containing_folder: *mut Action,
    pub context_menu_open_with_external_program: *mut Action,
    pub context_menu_open_notes: *mut Action,
    pub context_menu_check_tables: *mut Action,
    pub context_menu_merge_tables: *mut Action,

    //-------------------------------------------------------------------------------//
    // Actions not in the UI.
    //-------------------------------------------------------------------------------//
    pub packfile_contents_tree_view_expand_all: *mut Action,
    pub packfile_contents_tree_view_collapse_all: *mut Action,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function creates an entire `PackFileContentsUI` struct.
    pub fn new(main_window: *mut MainWindow) -> Self {

        //-----------------------------------------------//
        // `PackFile Contents` DockWidget.
        //-----------------------------------------------//

        // Create and configure the 'TreeView` Dock Widget and all his contents.
        let mut packfile_contents_dock_widget = unsafe { DockWidget::new_unsafe(main_window as *mut Widget) };
        let packfile_contents_dock_inner_widget = Widget::new();
        let packfile_contents_dock_layout = create_grid_layout_unsafe(packfile_contents_dock_inner_widget.as_mut_ptr() as *mut Widget);
        unsafe { packfile_contents_dock_widget.set_widget(packfile_contents_dock_inner_widget.into_raw()); }
        unsafe { main_window.as_mut().unwrap().add_dock_widget((DockWidgetArea::LeftDockWidgetArea, packfile_contents_dock_widget.as_mut_ptr())); }
        packfile_contents_dock_widget.set_window_title(&QString::from_std_str("PackFile Contents"));

        // Create and configure the `TreeView` itself.
        let mut packfile_contents_tree_view = TreeView::new();
        let packfile_contents_tree_model = StandardItemModel::new(());
        let packfile_contents_tree_model_filter = unsafe { new_treeview_filter(packfile_contents_dock_widget.as_mut_ptr() as *mut Object) };
        unsafe { packfile_contents_tree_model_filter.as_mut().unwrap().set_source_model(packfile_contents_tree_model.as_mut_ptr() as *mut AbstractItemModel); }
        unsafe { packfile_contents_tree_view.set_model(packfile_contents_tree_model_filter as *mut AbstractItemModel); }
        packfile_contents_tree_view.set_header_hidden(true);
        packfile_contents_tree_view.set_animated(true);
        packfile_contents_tree_view.set_uniform_row_heights(true);
        packfile_contents_tree_view.set_selection_mode(SelectionMode::Extended);
        packfile_contents_tree_view.set_context_menu_policy(ContextMenuPolicy::Custom);
        packfile_contents_tree_view.set_expands_on_double_click(false);

        // Create and configure the widgets to control the `TreeView`s filter.
        let mut filter_line_edit = LineEdit::new(());
        let mut filter_autoexpand_matches_button = PushButton::new(&QString::from_std_str("Auto-Expand Matches"));
        let mut filter_case_sensitive_button = PushButton::new(&QString::from_std_str("AaI"));
        filter_line_edit.set_placeholder_text(&QString::from_std_str("Type here to filter the files in the PackFile. Works with Regex too!"));
        filter_autoexpand_matches_button.set_checkable(true);
        filter_case_sensitive_button.set_checkable(true);

        // Add everything to the `TreeView`s Dock Layout.
        unsafe { packfile_contents_dock_layout.as_mut().unwrap().add_widget((packfile_contents_tree_view.as_mut_ptr() as *mut Widget, 0, 0, 1, 2)); }
        unsafe { packfile_contents_dock_layout.as_mut().unwrap().add_widget((filter_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 2)); }
        unsafe { packfile_contents_dock_layout.as_mut().unwrap().add_widget((filter_autoexpand_matches_button.as_mut_ptr() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { packfile_contents_dock_layout.as_mut().unwrap().add_widget((filter_case_sensitive_button.as_mut_ptr() as *mut Widget, 2, 1, 1, 1)); }

        //-------------------------------------------------------------------------------//
        // Contextual menu for the PackFile Contents TreeView.
        //-------------------------------------------------------------------------------//

        // Populate the `Contextual Menu` for the `PackFile` TreeView.
        let mut packfile_contents_tree_view_context_menu = Menu::new(());
        let menu_add = packfile_contents_tree_view_context_menu.add_menu(&QString::from_std_str("&Add..."));
        let menu_create = packfile_contents_tree_view_context_menu.add_menu(&QString::from_std_str("&Create..."));
        let menu_open = packfile_contents_tree_view_context_menu.add_menu(&QString::from_std_str("&Open..."));

        let menu_add_ref_mut = unsafe { menu_add.as_mut().unwrap() };
        let menu_create_ref_mut = unsafe { menu_create.as_mut().unwrap() };
        let menu_open_ref_mut = unsafe { menu_open.as_mut().unwrap() };
        let context_menu_add_file = menu_add_ref_mut.add_action(&QString::from_std_str("&Add File"));
        let context_menu_add_folder = menu_add_ref_mut.add_action(&QString::from_std_str("Add &Folder"));
        let context_menu_add_from_packfile = menu_add_ref_mut.add_action(&QString::from_std_str("Add from &PackFile"));
        let context_menu_create_folder = menu_create_ref_mut.add_action(&QString::from_std_str("&Create Folder"));
        let context_menu_create_loc = menu_create_ref_mut.add_action(&QString::from_std_str("&Create Loc"));
        let context_menu_create_db = menu_create_ref_mut.add_action(&QString::from_std_str("Create &DB"));
        let context_menu_create_text = menu_create_ref_mut.add_action(&QString::from_std_str("Create &Text"));
        let context_menu_mass_import_tsv = menu_create_ref_mut.add_action(&QString::from_std_str("Mass-Import TSV"));
        let context_menu_mass_export_tsv = menu_create_ref_mut.add_action(&QString::from_std_str("Mass-Export TSV"));
        let context_menu_rename = packfile_contents_tree_view_context_menu.add_action(&QString::from_std_str("&Rename"));
        let context_menu_delete = packfile_contents_tree_view_context_menu.add_action(&QString::from_std_str("&Delete"));
        let context_menu_extract = packfile_contents_tree_view_context_menu.add_action(&QString::from_std_str("&Extract"));
        let context_menu_open_decoder = menu_open_ref_mut.add_action(&QString::from_std_str("&Open with Decoder"));
        let context_menu_open_dependency_manager = menu_open_ref_mut.add_action(&QString::from_std_str("Open &Dependency Manager"));
        let context_menu_open_containing_folder = menu_open_ref_mut.add_action(&QString::from_std_str("Open &Containing Folder"));
        let context_menu_open_with_external_program = menu_open_ref_mut.add_action(&QString::from_std_str("Open with &External Program"));
        let context_menu_open_notes = menu_open_ref_mut.add_action(&QString::from_std_str("Open &Notes"));
        let context_menu_check_tables = packfile_contents_tree_view_context_menu.add_action(&QString::from_std_str("&Check Tables"));
        let context_menu_merge_tables = packfile_contents_tree_view_context_menu.add_action(&QString::from_std_str("&Merge Tables"));
        let packfile_contents_tree_view_expand_all = Action::new(&QString::from_std_str("&Expand All"));
        let packfile_contents_tree_view_collapse_all = Action::new(&QString::from_std_str("&Collapse All"));

        // Configure the `Contextual Menu` for the `PackFile` TreeView.
        unsafe { menu_create_ref_mut.insert_separator(context_menu_mass_import_tsv); }
        unsafe { packfile_contents_tree_view_context_menu.insert_separator(menu_open.as_ref().unwrap().menu_action()); }
        unsafe { packfile_contents_tree_view_context_menu.insert_separator(context_menu_rename); }
        unsafe { packfile_contents_tree_view_context_menu.insert_separator(context_menu_check_tables); }

        // Disable all the Contextual Menu actions by default.
        unsafe {
            context_menu_add_file.as_mut().unwrap().set_enabled(false);
            context_menu_add_folder.as_mut().unwrap().set_enabled(false);
            context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
            context_menu_create_folder.as_mut().unwrap().set_enabled(false);
            context_menu_create_db.as_mut().unwrap().set_enabled(false);
            context_menu_create_loc.as_mut().unwrap().set_enabled(false);
            context_menu_create_text.as_mut().unwrap().set_enabled(false);
            context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
            context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
            context_menu_delete.as_mut().unwrap().set_enabled(false);
            context_menu_extract.as_mut().unwrap().set_enabled(false);
            context_menu_rename.as_mut().unwrap().set_enabled(false);
            context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
            context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
            context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
            context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
            context_menu_open_notes.as_mut().unwrap().set_enabled(false);
        }

        // Create ***Da monsta***.
        Self {

            //-------------------------------------------------------------------------------//
            // `PackFile TreeView` Dock Widget.
            //-------------------------------------------------------------------------------//
            packfile_contents_dock_widget: packfile_contents_dock_widget.into_raw(),
            packfile_contents_tree_view: packfile_contents_tree_view.into_raw(),
            packfile_contents_tree_model_filter,
            packfile_contents_tree_model: packfile_contents_tree_model.into_raw(),
            filter_line_edit: filter_line_edit.into_raw(),
            filter_autoexpand_matches_button: filter_autoexpand_matches_button.into_raw(),
            filter_case_sensitive_button: filter_case_sensitive_button.into_raw(),

            //-------------------------------------------------------------------------------//
            // Contextual menu for the PackFile Contents TreeView.
            //-------------------------------------------------------------------------------//

            packfile_contents_tree_view_context_menu: packfile_contents_tree_view_context_menu.into_raw(),

            context_menu_add_file,
            context_menu_add_folder,
            context_menu_add_from_packfile,

            context_menu_create_folder,
            context_menu_create_loc,
            context_menu_create_db,
            context_menu_create_text,

            context_menu_mass_import_tsv,
            context_menu_mass_export_tsv,

            context_menu_rename,
            context_menu_delete,
            context_menu_extract,

            context_menu_open_decoder,
            context_menu_open_dependency_manager,
            context_menu_open_containing_folder,
            context_menu_open_with_external_program,
            context_menu_open_notes,

            context_menu_check_tables,
            context_menu_merge_tables,

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            packfile_contents_tree_view_expand_all: packfile_contents_tree_view_expand_all.into_raw(),
            packfile_contents_tree_view_collapse_all: packfile_contents_tree_view_collapse_all.into_raw(),
        }
    }
}
