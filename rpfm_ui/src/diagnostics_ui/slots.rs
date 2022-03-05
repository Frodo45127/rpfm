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
Module with all the code related to the main `DiagnosticsUISlots`.
!*/

use qt_core::QBox;
use qt_core::QObject;
use qt_core::QSignalBlocker;
use qt_core::{SlotNoArgs, SlotOfBool, SlotOfQModelIndex};

use log::info;

use std::rc::Rc;

use rpfm_lib::packfile::PathType;

use crate::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the diagnostics panel.
pub struct DiagnosticsUISlots {
    pub diagnostics_check_packfile: QBox<SlotNoArgs>,
    pub diagnostics_check_currently_open_packed_file: QBox<SlotNoArgs>,
    pub diagnostics_open_result: QBox<SlotOfQModelIndex>,
    pub show_hide_extra_filters: QBox<SlotOfBool>,
    pub toggle_filters: QBox<SlotNoArgs>,
    pub toggle_filters_types: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUISlots`.
impl DiagnosticsUISlots {

    /// This function creates an entire `DiagnosticsUISlots` struct.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) -> Self {

        // Checker slots.
        let diagnostics_check_packfile = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move || {
                info!("Triggering `Check PackFile (Diags)` By Slot");

                app_ui.main_window.set_disabled(true);
                DiagnosticsUI::check(&app_ui, &diagnostics_ui);
                app_ui.main_window.set_disabled(false);
            }
        ));

        let diagnostics_check_currently_open_packed_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui => move || {
                info!("Triggering `Check Open PackedFiles (Diag)` By Slot");

                app_ui.main_window.set_disabled(true);
                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);
                let path_types = UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile).map(|x| PathType::File(x.get_ref_path().to_vec())).collect::<Vec<PathType>>();
                DiagnosticsUI::check_on_path(&app_ui, &pack_file_contents_ui, &diagnostics_ui, path_types);
                app_ui.main_window.set_disabled(false);
            }
        ));

        // What happens when we try to open the file corresponding to one of the matches.
        let diagnostics_open_result = SlotOfQModelIndex::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move |model_index_filter| {
                info!("Triggering `Open Diagnostic Match` By Slot");
                DiagnosticsUI::open_match(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, model_index_filter.as_ptr());
            }
        ));

        let show_hide_extra_filters = SlotOfBool::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move |state| {
                if !state { diagnostics_ui.sidebar_scroll_area.hide(); }
                else { diagnostics_ui.sidebar_scroll_area.show();}
            }
        ));

        let toggle_filters = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move || {
            DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
        }));

        let toggle_filters_types = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move || {

                // Lock all signals except the last one, so the filters only trigger once.
                let _blocker_00 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_outdated_table.static_upcast::<QObject>());
                let _blocker_01 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_reference.static_upcast::<QObject>());
                let _blocker_02 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_empty_row.static_upcast::<QObject>());
                let _blocker_03 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_empty_key_field.static_upcast::<QObject>());
                let _blocker_04 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_empty_key_fields.static_upcast::<QObject>());
                let _blocker_05 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_duplicated_combined_keys.static_upcast::<QObject>());
                let _blocker_06 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_no_reference_table_found.static_upcast::<QObject>());
                let _blocker_07 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_no_reference_table_nor_column_found_pak.static_upcast::<QObject>());
                let _blocker_08 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_no_reference_table_nor_column_found_no_pak.static_upcast::<QObject>());
                let _blocker_09 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_escape.static_upcast::<QObject>());
                let _blocker_10 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_duplicated_row.static_upcast::<QObject>());
                let _blocker_11 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_loc_key.static_upcast::<QObject>());
                let _blocker_12 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_dependency_packfile.static_upcast::<QObject>());
                let _blocker_13 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_dependencies_cache_not_generated.static_upcast::<QObject>());
                let _blocker_14 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_packfile_name.static_upcast::<QObject>());
                let _blocker_15 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_table_name_ends_in_number.static_upcast::<QObject>());
                let _blocker_16 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_table_name_has_space.static_upcast::<QObject>());
                let _blocker_17 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_table_is_datacoring.static_upcast::<QObject>());
                let _blocker_18 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_dependencies_cache_outdated.static_upcast::<QObject>());
                let _blocker_19 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_dependencies_cache_could_not_be_loaded.static_upcast::<QObject>());
                let _blocker_20 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_field_with_path_not_found.static_upcast::<QObject>());
                let _blocker_21 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_incorrect_game_path.static_upcast::<QObject>());
                let _blocker_22 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_banned_table.static_upcast::<QObject>());
                let _blocker_23 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_value_cannot_be_empty.static_upcast::<QObject>());

                diagnostics_ui.checkbox_outdated_table.toggle();
                diagnostics_ui.checkbox_invalid_reference.toggle();
                diagnostics_ui.checkbox_empty_row.toggle();
                diagnostics_ui.checkbox_empty_key_field.toggle();
                diagnostics_ui.checkbox_empty_key_fields.toggle();
                diagnostics_ui.checkbox_duplicated_combined_keys.toggle();
                diagnostics_ui.checkbox_no_reference_table_found.toggle();
                diagnostics_ui.checkbox_no_reference_table_nor_column_found_pak.toggle();
                diagnostics_ui.checkbox_no_reference_table_nor_column_found_no_pak.toggle();
                diagnostics_ui.checkbox_invalid_escape.toggle();
                diagnostics_ui.checkbox_duplicated_row.toggle();
                diagnostics_ui.checkbox_invalid_loc_key.toggle();
                diagnostics_ui.checkbox_invalid_dependency_packfile.toggle();
                diagnostics_ui.checkbox_dependencies_cache_not_generated.toggle();
                diagnostics_ui.checkbox_invalid_packfile_name.toggle();
                diagnostics_ui.checkbox_table_name_ends_in_number.toggle();
                diagnostics_ui.checkbox_table_name_has_space.toggle();
                diagnostics_ui.checkbox_table_is_datacoring.toggle();
                diagnostics_ui.checkbox_dependencies_cache_outdated.toggle();
                diagnostics_ui.checkbox_dependencies_cache_could_not_be_loaded.toggle();
                diagnostics_ui.checkbox_field_with_path_not_found.toggle();
                diagnostics_ui.checkbox_incorrect_game_path.toggle();
                diagnostics_ui.checkbox_banned_table.toggle();
                diagnostics_ui.checkbox_value_cannot_be_empty.toggle();

                DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
            }
        ));

        // And here... we return all the slots.
        Self {
            diagnostics_check_packfile,
            diagnostics_check_currently_open_packed_file,
            diagnostics_open_result,
            show_hide_extra_filters,
            toggle_filters,
            toggle_filters_types,
        }
    }
}
