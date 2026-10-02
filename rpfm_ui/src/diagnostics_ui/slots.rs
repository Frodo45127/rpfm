//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code related to the main `DiagnosticsUISlots`.

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::QObject;
use qt_core::QSignalBlocker;
use qt_core::{SlotNoArgs, SlotOfBool, SlotOfQModelIndex};

use getset::Getters;

use std::rc::Rc;

use rpfm_ipc::api::diagnostics::IgnoreDiagnostics;
use rpfm_ipc::helpers::DataSource;
use rpfm_lib::files::ContainerPath;
use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::communications::call_api;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::UI_STATE;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                                  Macros
//-------------------------------------------------------------------------------//

macro_rules! diag_blocked {
    ($ui:ident, $check:ident, $toggled:expr) => (
        let _blocker = QSignalBlocker::from_q_object($ui.$check.static_upcast::<QObject>());

        if $toggled {
            $ui.$check.set_checked(true);
        }
    );
}

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
    poll_check: QBox<SlotNoArgs>,
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
                rpfm_telemetry::track_action("Check PackFile (Diags)");

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);
                DiagnosticsUI::check(&diagnostics_ui);
            }
        ));

        let diagnostics_check_currently_open_packed_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Check Open PackedFiles (Diag)");

                let _ = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui);
                let path_types = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).map(|x| ContainerPath::File(x.path_copy())).collect::<Vec<ContainerPath>>();
                DiagnosticsUI::check_on_path(&diagnostics_ui, path_types);
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
                rpfm_telemetry::track_action("Open Diagnostic Match");
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
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Parent Folder");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::ParentFolder, false, false);
            }
        ));

        let ignore_parent_folder_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Parent Folder Field");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::ParentFolder, true, false);
            }
        ));

        let ignore_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore File");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::File, false, false);
            }
        ));

        let ignore_file_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore File Field");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::File, true, false);
            }
        ));

        let ignore_diagnostic_for_parent_folder = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Diagnostic for Parent Folder");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::ParentFolder, false, true);
            }
        ));

        let ignore_diagnostic_for_parent_folder_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Diagnostic for Parent Folder Field");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::ParentFolder, true, true);
            }
        ));

        let ignore_diagnostic_for_file = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Diagnostic for File");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::File, false, true);
            }
        ));

        let ignore_diagnostic_for_file_field = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Diagnostic for File Field");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::File, true, true);
            }
        ));

        let ignore_diagnostic_for_pack = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            pack_file_contents_ui,
            diagnostics_ui => move || {
                rpfm_telemetry::track_action("Diagnostics: Ignore Diagnostic for Pack");
                ignore_selection(&diagnostics_ui, &pack_file_contents_ui, IgnoreScope::Pack, false, true);
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

            diagnostics_ui.save_disabled_diagnostics();
            DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
        }));

        let toggle_filters_all = SlotOfBool::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move |toggled| {

                // Lock all signals except the last one, so the filters only trigger once.
                diag_blocked!(diagnostics_ui, checkbox_outdated_table, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_reference, toggled);
                diag_blocked!(diagnostics_ui, checkbox_empty_row, toggled);
                diag_blocked!(diagnostics_ui, checkbox_empty_key_field, toggled);
                diag_blocked!(diagnostics_ui, checkbox_empty_key_fields, toggled);
                diag_blocked!(diagnostics_ui, checkbox_duplicated_combined_keys, toggled);
                diag_blocked!(diagnostics_ui, checkbox_no_reference_table_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_no_reference_table_nor_column_found_pak, toggled);
                diag_blocked!(diagnostics_ui, checkbox_no_reference_table_nor_column_found_no_pak, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_escape, toggled);
                diag_blocked!(diagnostics_ui, checkbox_duplicated_row, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_loc_key, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_dependency_packfile, toggled);
                diag_blocked!(diagnostics_ui, checkbox_dependencies_cache_not_generated, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_packfile_name, toggled);
                diag_blocked!(diagnostics_ui, checkbox_table_name_ends_in_number, toggled);
                diag_blocked!(diagnostics_ui, checkbox_table_name_has_space, toggled);
                diag_blocked!(diagnostics_ui, checkbox_table_is_datacoring, toggled);
                diag_blocked!(diagnostics_ui, checkbox_dependencies_cache_outdated, toggled);
                diag_blocked!(diagnostics_ui, checkbox_dependencies_cache_could_not_be_loaded, toggled);
                diag_blocked!(diagnostics_ui, checkbox_field_with_path_not_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_incorrect_game_path, toggled);
                diag_blocked!(diagnostics_ui, checkbox_banned_table, toggled);
                diag_blocked!(diagnostics_ui, checkbox_value_cannot_be_empty, toggled);
                diag_blocked!(diagnostics_ui, checkbox_altered_table, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_art_set_id, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_variant_filename, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_diffuse_not_found_for_variant, toggled);
                diag_blocked!(diagnostics_ui, checkbox_datacored_portrait_settings, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_duplicate_formation_name, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_no_absolute_block, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_duplicate_block_id, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_missing_reference, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_forward_reference, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_reference_cycle, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_empty_span, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_invalid_thresholds, toggled);
                diag_blocked!(diagnostics_ui, checkbox_group_formations_no_entity_preferences, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_mask_1_not_found_for_variant, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_mask_2_not_found_for_variant, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_mask_3_not_found_for_variant, toggled);
                diag_blocked!(diagnostics_ui, checkbox_loocomotion_graph_path_not_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_path_not_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_meta_file_path_not_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_snd_file_path_not_found, toggled);
                diag_blocked!(diagnostics_ui, checkbox_lua_invalid_key, toggled);
                diag_blocked!(diagnostics_ui, checkbox_lua_syntax_error, toggled);
                diag_blocked!(diagnostics_ui, checkbox_lua_unknown_method, toggled);
                diag_blocked!(diagnostics_ui, checkbox_lua_wrong_argument_count, toggled);
                diag_blocked!(diagnostics_ui, checkbox_lua_unknown_event, toggled);
                diag_blocked!(diagnostics_ui, checkbox_missing_loc_data_file_detected, toggled);
                diag_blocked!(diagnostics_ui, checkbox_invalid_file_name, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_itm, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_overwrite, toggled);
                diag_blocked!(diagnostics_ui, checkbox_file_duplicated, toggled);

                diagnostics_ui.save_disabled_diagnostics();
                DiagnosticsUI::filter(&app_ui, &diagnostics_ui);
            }
        ));

        let poll_check = SlotNoArgs::new(&diagnostics_ui.diagnostics_dock_widget, clone!(
            app_ui,
            diagnostics_ui => move || {
                DiagnosticsUI::poll_check(&app_ui, &diagnostics_ui);
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
            poll_check,
        }
    }
}

/// Where an ignore rule made from the selected diagnostics applies.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IgnoreScope {

    /// The whole pack.
    Pack,

    /// The folder containing the file of each diagnostic.
    ParentFolder,

    /// The file of each diagnostic.
    File,
}

/// Makes the next checks of the selected pack skip the selected diagnostics.
///
/// Diagnostics missing what the rule needs (a path, columns or a type) are skipped.
///
/// # Arguments
///
/// * `diagnostics_ui` - The diagnostics panel, with the selection.
/// * `pack_file_contents_ui` - The pack tree, to know the selected pack.
/// * `scope` - Where the rules apply.
/// * `with_columns` - If the rules only cover the columns of each diagnostic.
/// * `with_type` - If the rules only cover the type of each diagnostic.
unsafe fn ignore_selection(diagnostics_ui: &Rc<DiagnosticsUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, scope: IgnoreScope, with_columns: bool, with_type: bool) {
    let pack = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
    for index in &diagnostics_ui.selection_sorted_and_deduped() {
        let cell = |column| index.model().index_2a(index.row(), column).data_0a().to_string().to_std_string();
        let file_path = cell(3);
        let path = match scope {
            IgnoreScope::Pack => String::new(),
            IgnoreScope::ParentFolder => file_path.rsplit_once('/').map(|(folder, _)| folder.to_owned()).unwrap_or_default(),
            IgnoreScope::File => file_path,
        };

        let columns = if with_columns {
            let columns = cell(6);
            if columns.is_empty() { vec![] } else { serde_json::from_str::<Vec<String>>(&columns).unwrap_or_default() }
        } else {
            vec![]
        };

        let report_types = if with_type { vec![cell(5)].into_iter().filter(|report_type| !report_type.is_empty()).collect() } else { vec![] };
        if (scope != IgnoreScope::Pack && path.is_empty()) || (with_columns && columns.is_empty()) || (with_type && report_types.is_empty()) {
            continue;
        }

        if let Err(error) = call_api(&IgnoreDiagnostics { pack: pack.clone(), path, columns, report_types }) {
            return show_dialog(&diagnostics_ui.diagnostics_dock_widget, error, false);
        }
    }
}
