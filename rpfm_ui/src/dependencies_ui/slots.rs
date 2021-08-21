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
Module with all the code related to the main `DependenciesUISlots`.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;
use qt_gui::QGuiApplication;

use qt_core::QBox;
use qt_core::{SlotOfBool, SlotNoArgs, SlotOfQString};

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::PackTree;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::DataSource;
use crate::QString;
use crate::utils::*;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the Dependencies panel.
pub struct DependenciesUISlots {
    pub open_packedfile_preview: QBox<SlotNoArgs>,
    pub open_packedfile_full: QBox<SlotNoArgs>,

    pub filter_trigger: QBox<SlotNoArgs>,
    pub filter_change_text: QBox<SlotOfQString>,
    pub filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub filter_change_case_sensitive: QBox<SlotOfBool>,
    pub filter_check_regex: QBox<SlotOfQString>,

    pub contextual_menu: QBox<SlotOfQPoint>,
    pub contextual_menu_enabler: QBox<SlotNoArgs>,
    pub contextual_menu_import: QBox<SlotOfBool>,
    pub contextual_menu_copy_path: QBox<SlotOfBool>,

    pub dependencies_tree_view_expand_all: QBox<SlotNoArgs>,
    pub dependencies_tree_view_collapse_all: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DependenciesUISlots`.
impl DependenciesUISlots {

	/// This function creates an entire `DependenciesUISlots` struct.
	pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) -> Self {

        // Slot to open the selected PackedFile as a preview.
        let open_packedfile_preview = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move || {
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, None, true, false, DataSource::GameFiles);
        }));

        // Slot to open the selected PackedFile as a permanent view.
        let open_packedfile_full = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move || {
            AppUI::open_packedfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, None, false, false, DataSource::GameFiles);
        }));

        // What happens when we trigger one of the filter events for the Dependencies TreeView.
        let filter_change_text = SlotOfQString::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |_| {
                DependenciesUI::start_delayed_updates_timer(&dependencies_ui);
            }
        ));
        let filter_change_autoexpand_matches = SlotOfBool::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |_| {
                DependenciesUI::filter_files(&dependencies_ui);
            }
        ));
        let filter_change_case_sensitive = SlotOfBool::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |_| {
                DependenciesUI::filter_files(&dependencies_ui);
            }
        ));

        // Function triggered by the filter timer.
        let filter_trigger = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move || {
                DependenciesUI::filter_files(&dependencies_ui);
            }
        ));

        // What happens when we trigger the "Check Regex" action.
        let filter_check_regex = SlotOfQString::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |string| {
                check_regex(&string.to_std_string(), dependencies_ui.filter_line_edit.static_upcast());
            }
        ));

        // Slot to show the Contextual Menu for the TreeView.
        let contextual_menu = SlotOfQPoint::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |_| {
            dependencies_ui.dependencies_tree_view_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));


        // Slot to enable/disable contextual actions depending on the selected item.
        let contextual_menu_enabler = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            pack_file_contents_ui,
            dependencies_ui => move || {
                if let Some(root_node_type) = dependencies_ui.dependencies_tree_view.get_root_source_type_from_selection(true) {

                    match root_node_type {
                        DataSource::PackFile => {
                            dependencies_ui.context_menu_import.set_enabled(false);
                        },
                        DataSource::ParentFiles => {
                            if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() > 0 {
                                dependencies_ui.context_menu_import.set_enabled(true);
                            } else {
                                dependencies_ui.context_menu_import.set_enabled(false);
                            }
                        },
                        DataSource::GameFiles => {
                            if pack_file_contents_ui.packfile_contents_tree_model.row_count_0a() > 0 {
                                dependencies_ui.context_menu_import.set_enabled(true);
                            } else {
                                dependencies_ui.context_menu_import.set_enabled(false);
                            }
                        },
                        DataSource::AssKitFiles => {
                            dependencies_ui.context_menu_import.set_enabled(false);
                        },
                        DataSource::ExternalFile => {
                            dependencies_ui.context_menu_import.set_enabled(false);
                        },
                    }
                }
            }
        ));

        // What happens when we trigger the "Import" action in the Contextual Menu.
        let contextual_menu_import = SlotOfBool::new(&dependencies_ui.dependencies_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            dependencies_ui => move |_| {
                dependencies_ui.import_dependencies(&app_ui, &pack_file_contents_ui);
            }
        ));

        let contextual_menu_copy_path = SlotOfBool::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move |_| {
            let selected_paths = dependencies_ui.dependencies_tree_view.get_path_from_selection();
            if selected_paths.len() == 1 && !selected_paths[0].is_empty() {
                QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(selected_paths[0].join("/")));
            }
        }));

        let dependencies_tree_view_expand_all = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move || {
                dependencies_ui.dependencies_tree_view.expand_all();
            }
        ));
        let dependencies_tree_view_collapse_all = SlotNoArgs::new(&dependencies_ui.dependencies_dock_widget, clone!(
            dependencies_ui => move || {
                dependencies_ui.dependencies_tree_view.collapse_all();
            }
        ));

        // And here... we return all the slots.
		Self {
            open_packedfile_preview,
            open_packedfile_full,

            filter_trigger,
            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,
            filter_check_regex,

            contextual_menu,
            contextual_menu_enabler,
            contextual_menu_import,
            contextual_menu_copy_path,

            dependencies_tree_view_expand_all,
            dependencies_tree_view_collapse_all,
        }
	}
}
