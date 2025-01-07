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
Module with all the code related to the main `DiagnosticsUISlots`.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::QObject;
use qt_core::QSignalBlocker;
use qt_core::{SlotNoArgs, SlotOfBool, SlotOfQModelIndex};

use getset::Getters;

use std::rc::Rc;

use rpfm_lib::integrations::log::*;
use rpfm_lib::files::ContainerPath;
use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::Command;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the diagnostics panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct DiagnosticsUISlots {
    diagnostics_check_packfile: QBox<SlotNoArgs>,
    diagnostics_check_currently_open_packed_file: QBox<SlotNoArgs>,
    diagnostics_open_result: QBox<SlotOfQModelIndex>,
    contextual_menu: QBox<SlotOfQPoint>,
    contextual_menu_enabler: QBox<SlotNoArgs>,
    ignore_parent_folder: QBox<SlotNoArgs>,
    ignore_parent_folder_field: QBox<SlotNoArgs>,
    ignore_file: QBox<SlotNoArgs>,
    ignore_file_field: QBox<SlotNoArgs>,
    ignore_diagnostic_for_parent_folder: QBox<SlotNoArgs>,
    ignore_diagnostic_for_parent_folder_field: QBox<SlotNoArgs>,
    ignore_diagnostic_for_file: QBox<SlotNoArgs>,
    ignore_diagnostic_for_file_field: QBox<SlotNoArgs>,
    ignore_diagnostic_for_pack: QBox<SlotNoArgs>,
    show_hide_extra_filters: QBox<SlotOfBool>,
    toggle_filters: QBox<SlotOfBool>,
    toggle_filters_all: QBox<SlotOfBool>,
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
        references_ui: &Rc<ReferencesUI>,
    ) -> Self {

        // Checker slots.
        let diagnostics_check_packfile = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui => move || {
                info!("Triggering `Check PackFile (Diags)` By Slot");

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);
                DiagnosticsUI::check(&app_ui, &diagnostics_ui);
            }
        ));

        let diagnostics_check_currently_open_packed_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui => move || {
                info!("Triggering `Check Open PackedFiles (Diag)` By Slot");

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);
                let path_types = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| ContainerPath::File(x.path_copy())).collect::<Vec<ContainerPath>>();
                DiagnosticsUI::check_on_path(&app_ui, &diagnostics_ui, path_types);
            }
        ));

        // What happens when we try to open the file corresponding to one of the matches.
        let diagnostics_open_result = SlotOfQModelIndex::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |model_index_filter| {
                info!("Triggering `Open Diagnostic Match` By Slot");
                DiagnosticsUI::open_match(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, model_index_filter.as_ptr());
            }
        ));

        let contextual_menu = SlotOfQPoint::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move |_| {
            diagnostics_ui.diagnostics_table_view_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let contextual_menu_enabler = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();

                // Parent folder diagnostics need to have a parent folder to be enabled.
                let has_path = selection.iter().all(|index| !index.model().index_2a(index.row(), 3).data_0a().to_string().is_empty());
                let has_parents = selection.iter().all(|index| index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string().contains('/'));
                let has_fields = selection.iter().all(|index| !index.model().index_2a(index.row(), 6).data_0a().to_string().is_empty());

                let non_ignorable_fields = [
                    "InvalidDependencyPackName",
                    "DependenciesCacheNotGenerated",
                    "DependenciesCacheOutdated",
                    "DependenciesCacheCouldNotBeLoaded",
                    "IncorrectGamePath",
                    "InvalidPackName"
                ];

                let can_be_ignored = selection.iter().all(|index| !non_ignorable_fields.contains(&&*index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string()));

                diagnostics_ui.ignore_parent_folder.set_enabled(!selection.is_empty() && has_parents);
                diagnostics_ui.ignore_parent_folder_field.set_enabled(!selection.is_empty() && has_parents && has_fields);

                diagnostics_ui.ignore_file.set_enabled(!selection.is_empty() && has_path);
                diagnostics_ui.ignore_file_field.set_enabled(!selection.is_empty() && has_path && has_fields);

                diagnostics_ui.ignore_diagnostic_for_parent_folder.set_enabled(!selection.is_empty() && can_be_ignored && has_parents);
                diagnostics_ui.ignore_diagnostic_for_parent_folder_field.set_enabled(!selection.is_empty() && can_be_ignored && has_parents && has_fields);

                diagnostics_ui.ignore_diagnostic_for_file.set_enabled(!selection.is_empty() && can_be_ignored && has_path);
                diagnostics_ui.ignore_diagnostic_for_file_field.set_enabled(!selection.is_empty() && can_be_ignored && has_path && has_fields);

                // This one is enabled as long as there is a selection.
                diagnostics_ui.ignore_diagnostic_for_pack.set_enabled(!selection.is_empty() && can_be_ignored);
            }
        ));

        let ignore_parent_folder = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let (path, _) = path.rsplit_once('/').unwrap();
                    if !path.is_empty() {
                        string.push_str(path);
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_parent_folder_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let (path, _) = path.rsplit_once('/').unwrap();
                    let fields = index.model().index_2a(index.row(), 6).data_0a().to_string().to_std_string();
                    let fields: Vec<String> = if fields.is_empty() {
                        vec![]
                    } else {
                        serde_json::from_str(&fields).unwrap()
                    };

                    if !path.is_empty() && !fields.is_empty() {
                        string.push_str(&format!("{path};{}", fields.join(",")));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    if !path.is_empty() {
                        string.push_str(&path);
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_file_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let fields = index.model().index_2a(index.row(), 6).data_0a().to_string().to_std_string();
                    let fields: Vec<String> = if fields.is_empty() {
                        vec![]
                    } else {
                        serde_json::from_str(&fields).unwrap()
                    };

                    if !path.is_empty() && !fields.is_empty() {
                        string.push_str(&format!("{path};{}", fields.join(",")));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_diagnostic_for_parent_folder = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let (path, _) = path.rsplit_once('/').unwrap();
                    let diagnostic = index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string();

                    if !path.is_empty() && !diagnostic.is_empty() {
                        string.push_str(&format!("{path};;{diagnostic}"));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_diagnostic_for_parent_folder_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let (path, _) = path.rsplit_once('/').unwrap();
                    let diagnostic = index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string();
                    let fields = index.model().index_2a(index.row(), 6).data_0a().to_string().to_std_string();
                    let fields: Vec<String> = if fields.is_empty() {
                        vec![]
                    } else {
                        serde_json::from_str(&fields).unwrap()
                    };

                    if !path.is_empty() && !fields.is_empty() && !diagnostic.is_empty() {
                        string.push_str(&format!("{path};{};{diagnostic}", fields.join(",")));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_diagnostic_for_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let diagnostic = index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string();
                    if !path.is_empty() && !diagnostic.is_empty() {
                        string.push_str(&format!("{path};;{diagnostic}"));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_diagnostic_for_file_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let path = index.model().index_2a(index.row(), 3).data_0a().to_string().to_std_string();
                    let diagnostic = index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string();
                    let fields = index.model().index_2a(index.row(), 6).data_0a().to_string().to_std_string();
                    let fields: Vec<String> = if fields.is_empty() {
                        vec![]
                    } else {
                        serde_json::from_str(&fields).unwrap()
                    };

                    if !path.is_empty() && !fields.is_empty() && !diagnostic.is_empty() {
                        string.push_str(&format!("{path};{};{diagnostic}", fields.join(",")));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let ignore_diagnostic_for_pack = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move || {
                let selection = diagnostics_ui.selection_sorted_and_deduped();
                let mut string = String::new();

                for index in &selection {
                    let diagnostic = index.model().index_2a(index.row(), 5).data_0a().to_string().to_std_string();
                    if !diagnostic.is_empty() {
                        string.push_str(&format!(";;{diagnostic}"));
                        string.push('\n');
                    }
                }

                if !string.is_empty() {
                    CENTRAL_COMMAND.send_background(Command::AddLineToPackIgnoredDiagnostics(format!("\n{string}")));
                }
            }
        ));

        let show_hide_extra_filters = SlotOfBool::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            diagnostics_ui => move |state| {
                if !state { diagnostics_ui.sidebar_scroll_area.hide(); }
                else { diagnostics_ui.sidebar_scroll_area.show();}
            }
        ));

        let toggle_filters = SlotOfBool::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move |toggled| {

            // Uncheck all if it's checked.
            if !toggled && diagnostics_ui.checkbox_all.is_checked() {
                diagnostics_ui.checkbox_all.block_signals(true);
                diagnostics_ui.checkbox_all.set_checked(false);
                diagnostics_ui.checkbox_all.block_signals(false);
            }

            DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
        }));

        let toggle_filters_all = SlotOfBool::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move |toggled| {

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
                let _blocker_24 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_art_set_id.static_upcast::<QObject>());
                let _blocker_25 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_invalid_variant_filename.static_upcast::<QObject>());
                let _blocker_26 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_file_diffuse_not_found_for_variant.static_upcast::<QObject>());
                let _blocker_27 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_datacored_portrait_settings.static_upcast::<QObject>());
                let _blocker_28 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_file_mask_1_not_found_for_variant.static_upcast::<QObject>());
                let _blocker_29 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_file_mask_2_not_found_for_variant.static_upcast::<QObject>());
                let _blocker_30 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_file_mask_3_not_found_for_variant.static_upcast::<QObject>());
                let _blocker_31 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_loocomotion_graph_path_not_found.static_upcast::<QObject>());
                let _blocker_32 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_file_path_not_found.static_upcast::<QObject>());
                let _blocker_33 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_meta_file_path_not_found.static_upcast::<QObject>());
                let _blocker_34 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_snd_file_path_not_found.static_upcast::<QObject>());
                let _blocker_35 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_lua_invalid_key.static_upcast::<QObject>());
                let _blocker_36 = QSignalBlocker::from_q_object(diagnostics_ui.checkbox_missing_loc_data_file_detected.static_upcast::<QObject>());

                if toggled {
                    diagnostics_ui.checkbox_outdated_table.set_checked(true);
                    diagnostics_ui.checkbox_invalid_reference.set_checked(true);
                    diagnostics_ui.checkbox_empty_row.set_checked(true);
                    diagnostics_ui.checkbox_empty_key_field.set_checked(true);
                    diagnostics_ui.checkbox_empty_key_fields.set_checked(true);
                    diagnostics_ui.checkbox_duplicated_combined_keys.set_checked(true);
                    diagnostics_ui.checkbox_no_reference_table_found.set_checked(true);
                    diagnostics_ui.checkbox_no_reference_table_nor_column_found_pak.set_checked(true);
                    diagnostics_ui.checkbox_no_reference_table_nor_column_found_no_pak.set_checked(true);
                    diagnostics_ui.checkbox_invalid_escape.set_checked(true);
                    diagnostics_ui.checkbox_duplicated_row.set_checked(true);
                    diagnostics_ui.checkbox_invalid_loc_key.set_checked(true);
                    diagnostics_ui.checkbox_invalid_dependency_packfile.set_checked(true);
                    diagnostics_ui.checkbox_dependencies_cache_not_generated.set_checked(true);
                    diagnostics_ui.checkbox_invalid_packfile_name.set_checked(true);
                    diagnostics_ui.checkbox_table_name_ends_in_number.set_checked(true);
                    diagnostics_ui.checkbox_table_name_has_space.set_checked(true);
                    diagnostics_ui.checkbox_table_is_datacoring.set_checked(true);
                    diagnostics_ui.checkbox_dependencies_cache_outdated.set_checked(true);
                    diagnostics_ui.checkbox_dependencies_cache_could_not_be_loaded.set_checked(true);
                    diagnostics_ui.checkbox_field_with_path_not_found.set_checked(true);
                    diagnostics_ui.checkbox_incorrect_game_path.set_checked(true);
                    diagnostics_ui.checkbox_banned_table.set_checked(true);
                    diagnostics_ui.checkbox_value_cannot_be_empty.set_checked(true);
                    diagnostics_ui.checkbox_invalid_art_set_id.set_checked(true);
                    diagnostics_ui.checkbox_invalid_variant_filename.set_checked(true);
                    diagnostics_ui.checkbox_file_diffuse_not_found_for_variant.set_checked(true);
                    diagnostics_ui.checkbox_file_mask_1_not_found_for_variant.set_checked(true);
                    diagnostics_ui.checkbox_file_mask_2_not_found_for_variant.set_checked(true);
                    diagnostics_ui.checkbox_file_mask_3_not_found_for_variant.set_checked(true);
                    diagnostics_ui.checkbox_datacored_portrait_settings.set_checked(true);
                    diagnostics_ui.checkbox_loocomotion_graph_path_not_found.set_checked(true);
                    diagnostics_ui.checkbox_file_path_not_found.set_checked(true);
                    diagnostics_ui.checkbox_meta_file_path_not_found.set_checked(true);
                    diagnostics_ui.checkbox_snd_file_path_not_found.set_checked(true);
                    diagnostics_ui.checkbox_lua_invalid_key.set_checked(true);
                    diagnostics_ui.checkbox_missing_loc_data_file_detected.set_checked(true);
                }

                DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
            }
        ));

        // And here... we return all the slots.
        Self {
            diagnostics_check_packfile,
            diagnostics_check_currently_open_packed_file,
            diagnostics_open_result,
            contextual_menu,
            contextual_menu_enabler,
            ignore_parent_folder,
            ignore_parent_folder_field,
            ignore_file,
            ignore_file_field,
            ignore_diagnostic_for_parent_folder,
            ignore_diagnostic_for_parent_folder_field,
            ignore_diagnostic_for_file,
            ignore_diagnostic_for_file_field,
            ignore_diagnostic_for_pack,
            show_hide_extra_filters,
            toggle_filters,
            toggle_filters_all,
        }
    }
}
