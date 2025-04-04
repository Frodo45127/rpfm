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
Module with all the code to connect `DiagnosticsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `DiagnosticsUI` and `DiagnosticsUISlots` structs.
!*/

use super::{DiagnosticsUI, slots::DiagnosticsUISlots};

/// This function connects all the actions from the provided `DiagnosticsUI` with their slots in `DiagnosticsUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &DiagnosticsUI, slots: &DiagnosticsUISlots) {
    ui.diagnostics_table_view.double_clicked().connect(slots.diagnostics_open_result());

    ui.diagnostics_button_check_packfile.released().connect(slots.diagnostics_check_packfile());
    ui.diagnostics_button_check_current_packed_file.released().connect(slots.diagnostics_check_currently_open_packed_file());

    ui.diagnostics_button_info.toggled().connect(slots.toggle_filters());
    ui.diagnostics_button_warning.toggled().connect(slots.toggle_filters());
    ui.diagnostics_button_error.toggled().connect(slots.toggle_filters());
    ui.diagnostics_button_only_current_packed_file.toggled().connect(slots.toggle_filters());

    ui.diagnostics_button_show_more_filters.toggled().connect(slots.show_hide_extra_filters());

    ui.diagnostics_table_view.custom_context_menu_requested().connect(slots.contextual_menu());
    ui.diagnostics_table_view.selection_model().selection_changed().connect(slots.contextual_menu_enabler());
    ui.diagnostics_table_view_context_menu.about_to_show().connect(slots.contextual_menu_enabler());

    ui.ignore_parent_folder.triggered().connect(slots.ignore_parent_folder());
    ui.ignore_parent_folder_field.triggered().connect(slots.ignore_parent_folder_field());
    ui.ignore_file.triggered().connect(slots.ignore_file());
    ui.ignore_file_field.triggered().connect(slots.ignore_file_field());
    ui.ignore_diagnostic_for_parent_folder.triggered().connect(slots.ignore_diagnostic_for_parent_folder());
    ui.ignore_diagnostic_for_parent_folder_field.triggered().connect(slots.ignore_diagnostic_for_parent_folder_field());
    ui.ignore_diagnostic_for_file.triggered().connect(slots.ignore_diagnostic_for_file());
    ui.ignore_diagnostic_for_file_field.triggered().connect(slots.ignore_diagnostic_for_file_field());
    ui.ignore_diagnostic_for_pack.triggered().connect(slots.ignore_diagnostic_for_pack());

    ui.checkbox_all.toggled().connect(slots.toggle_filters_all());
    ui.checkbox_outdated_table.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_reference.toggled().connect(slots.toggle_filters());
    ui.checkbox_empty_row.toggled().connect(slots.toggle_filters());
    ui.checkbox_empty_key_field.toggled().connect(slots.toggle_filters());
    ui.checkbox_empty_key_fields.toggled().connect(slots.toggle_filters());
    ui.checkbox_duplicated_combined_keys.toggled().connect(slots.toggle_filters());
    ui.checkbox_no_reference_table_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_no_reference_table_nor_column_found_pak.toggled().connect(slots.toggle_filters());
    ui.checkbox_no_reference_table_nor_column_found_no_pak.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_escape.toggled().connect(slots.toggle_filters());
    ui.checkbox_duplicated_row.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_dependency_packfile.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_loc_key.toggled().connect(slots.toggle_filters());
    ui.checkbox_dependencies_cache_not_generated.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_packfile_name.toggled().connect(slots.toggle_filters());
    ui.checkbox_table_name_ends_in_number.toggled().connect(slots.toggle_filters());
    ui.checkbox_table_name_has_space.toggled().connect(slots.toggle_filters());
    ui.checkbox_table_is_datacoring.toggled().connect(slots.toggle_filters());
    ui.checkbox_dependencies_cache_outdated.toggled().connect(slots.toggle_filters());
    ui.checkbox_dependencies_cache_could_not_be_loaded.toggled().connect(slots.toggle_filters());
    ui.checkbox_field_with_path_not_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_incorrect_game_path.toggled().connect(slots.toggle_filters());
    ui.checkbox_banned_table.toggled().connect(slots.toggle_filters());
    ui.checkbox_value_cannot_be_empty.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_art_set_id.toggled().connect(slots.toggle_filters());
    ui.checkbox_invalid_variant_filename.toggled().connect(slots.toggle_filters());
    ui.checkbox_file_diffuse_not_found_for_variant.toggled().connect(slots.toggle_filters());
    ui.checkbox_file_mask_1_not_found_for_variant.toggled().connect(slots.toggle_filters());
    ui.checkbox_file_mask_2_not_found_for_variant.toggled().connect(slots.toggle_filters());
    ui.checkbox_file_mask_3_not_found_for_variant.toggled().connect(slots.toggle_filters());
    ui.checkbox_datacored_portrait_settings.toggled().connect(slots.toggle_filters());
    ui.checkbox_loocomotion_graph_path_not_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_file_path_not_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_meta_file_path_not_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_snd_file_path_not_found.toggled().connect(slots.toggle_filters());
    ui.checkbox_lua_invalid_key.toggled().connect(slots.toggle_filters());
    ui.checkbox_missing_loc_data_file_detected.toggled().connect(slots.toggle_filters());
    ui.checkbox_uppercase_script_or_table_file_name.toggled().connect(slots.toggle_filters());
}
