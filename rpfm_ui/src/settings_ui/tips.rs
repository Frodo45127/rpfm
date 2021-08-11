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
Module with all the code to setup the tips (as tooltips) for the actions in `SettingsUI`.
!*/

use std::rc::Rc;

use crate::locale::qtr;
use crate::settings_ui::SettingsUI;

/// This function sets the status bar tip for all the actions in the provided `SettingsUI`.
pub unsafe fn set_tips(settings_ui: &Rc<SettingsUI>) {

    //-----------------------------------------------//
    // `UI` tips.
    //-----------------------------------------------//
    let ui_global_use_dark_theme_tip = qtr("tt_ui_global_use_dark_theme_tip");

    let ui_table_adjust_columns_to_content_tip = qtr("tt_ui_table_adjust_columns_to_content_tip");
    let ui_table_disable_combos_tip = qtr("tt_ui_table_disable_combos_tip");
    let ui_table_extend_last_column_tip = qtr("tt_ui_table_extend_last_column_tip");
    let ui_table_tight_table_mode_tip = qtr("tt_ui_table_tight_table_mode_tip");
    let ui_table_use_right_size_markers_tip = qtr("tt_ui_table_use_right_side_markers_tip");

    let ui_window_start_maximized_tip = qtr("tt_ui_window_start_maximized_tip");
    let settings_expand_treeview_when_adding_items_tip = qtr("settings_expand_treeview_when_adding_items_tip");

    settings_ui.ui_global_use_dark_theme_label.set_tool_tip(&ui_global_use_dark_theme_tip);
    settings_ui.ui_global_use_dark_theme_checkbox.set_tool_tip(&ui_global_use_dark_theme_tip);
    settings_ui.ui_table_adjust_columns_to_content_label.set_tool_tip(&ui_table_adjust_columns_to_content_tip);
    settings_ui.ui_table_adjust_columns_to_content_checkbox.set_tool_tip(&ui_table_adjust_columns_to_content_tip);
    settings_ui.ui_table_disable_combos_label.set_tool_tip(&ui_table_disable_combos_tip);
    settings_ui.ui_table_disable_combos_checkbox.set_tool_tip(&ui_table_disable_combos_tip);
    settings_ui.ui_table_extend_last_column_label.set_tool_tip(&ui_table_extend_last_column_tip);
    settings_ui.ui_table_extend_last_column_checkbox.set_tool_tip(&ui_table_extend_last_column_tip);
    settings_ui.ui_table_tight_table_mode_label.set_tool_tip(&ui_table_tight_table_mode_tip);
    settings_ui.ui_table_tight_table_mode_checkbox.set_tool_tip(&ui_table_tight_table_mode_tip);
    settings_ui.ui_table_use_right_size_markers_label.set_tool_tip(&ui_table_use_right_size_markers_tip);
    settings_ui.ui_table_use_right_size_markers_checkbox.set_tool_tip(&ui_table_use_right_size_markers_tip);
    settings_ui.ui_window_start_maximized_label.set_tool_tip(&ui_window_start_maximized_tip);
    settings_ui.ui_window_start_maximized_checkbox.set_tool_tip(&ui_window_start_maximized_tip);
    settings_ui.general_packfile_treeview_expand_treeview_when_adding_items_label.set_tool_tip(&settings_expand_treeview_when_adding_items_tip);
    settings_ui.general_packfile_treeview_expand_treeview_when_adding_items_checkbox.set_tool_tip(&settings_expand_treeview_when_adding_items_tip);

    //-----------------------------------------------//
    // `Extra` tips.
    //-----------------------------------------------//

    let autosave_amount_tip = qtr("tt_settings_autosave_amount");
    let extra_network_check_updates_on_start_tip = qtr("tt_extra_network_check_updates_on_start_tip");
    let extra_network_check_schema_updates_on_start_tip = qtr("tt_extra_network_check_schema_updates_on_start_tip");
    let extra_packfile_allow_editing_of_ca_packfiles_tip = qtr("tt_extra_packfile_allow_editing_of_ca_packfiles_tip");
    let extra_packfile_optimize_not_renamed_packedfiles_tip = qtr("tt_extra_packfile_optimize_not_renamed_packedfiles_tip");
    let extra_packfile_use_lazy_loading_tip = qtr("tt_extra_packfile_use_lazy_loading_tip");
    let extra_disable_uuid_regeneration_on_db_tables_label_tip = qtr("tt_extra_disable_uuid_regeneration_on_db_tables_label_tip");

    settings_ui.extra_packfile_autosave_amount_label.set_tool_tip(&autosave_amount_tip);
    settings_ui.extra_packfile_autosave_amount_spinbox.set_tool_tip(&autosave_amount_tip);

    settings_ui.extra_network_check_updates_on_start_label.set_tool_tip(&extra_network_check_updates_on_start_tip);
    settings_ui.extra_network_check_updates_on_start_checkbox.set_tool_tip(&extra_network_check_updates_on_start_tip);
    settings_ui.extra_network_check_schema_updates_on_start_label.set_tool_tip(&extra_network_check_schema_updates_on_start_tip);
    settings_ui.extra_network_check_schema_updates_on_start_checkbox.set_tool_tip(&extra_network_check_schema_updates_on_start_tip);
    settings_ui.extra_packfile_allow_editing_of_ca_packfiles_label.set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip);
    settings_ui.extra_packfile_allow_editing_of_ca_packfiles_checkbox.set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip);
    settings_ui.extra_packfile_optimize_not_renamed_packedfiles_label.set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip);
    settings_ui.extra_packfile_optimize_not_renamed_packedfiles_checkbox.set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip);
    settings_ui.extra_packfile_use_lazy_loading_label.set_tool_tip(&extra_packfile_use_lazy_loading_tip);
    settings_ui.extra_packfile_use_lazy_loading_checkbox.set_tool_tip(&extra_packfile_use_lazy_loading_tip);
    settings_ui.extra_packfile_disable_uuid_regeneration_on_db_tables_label.set_tool_tip(&extra_disable_uuid_regeneration_on_db_tables_label_tip);
    settings_ui.extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox.set_tool_tip(&extra_disable_uuid_regeneration_on_db_tables_label_tip);

    //-----------------------------------------------//
    // `Debug` tips.
    //-----------------------------------------------//
    let debug_check_for_missing_table_definitions_tip = qtr("tt_debug_check_for_missing_table_definitions_tip");
    let debug_spoof_ca_authoring_tool_tip = qtr("tt_settings_debug_spoof_ca_authoring_tool");
    let debug_enable_rigidmodel_editor = qtr("tt_settings_debug_enable_rigidmodel_editor");
    let debug_enable_esf_editor = qtr("tt_settings_debug_enable_esf_editor");

    let debug_clear_autosaves = qtr("tt_settings_debug_clear_autosave_folder");
    let debug_clear_schemas = qtr("tt_settings_debug_clear_schema_folder");
    let debug_clear_layout = qtr("tt_settings_debug_clear_layout_settings");

    settings_ui.debug_check_for_missing_table_definitions_label.set_tool_tip(&debug_check_for_missing_table_definitions_tip);
    settings_ui.debug_check_for_missing_table_definitions_checkbox.set_tool_tip(&debug_check_for_missing_table_definitions_tip);
    settings_ui.debug_spoof_ca_authoring_tool_label.set_tool_tip(&debug_spoof_ca_authoring_tool_tip);
    settings_ui.debug_spoof_ca_authoring_tool_checkbox.set_tool_tip(&debug_spoof_ca_authoring_tool_tip);
    settings_ui.debug_enable_rigidmodel_editor_label.set_tool_tip(&debug_enable_rigidmodel_editor);
    settings_ui.debug_enable_rigidmodel_editor_checkbox.set_tool_tip(&debug_enable_rigidmodel_editor);
    settings_ui.debug_enable_esf_editor_label.set_tool_tip(&debug_enable_esf_editor);
    settings_ui.debug_enable_esf_editor_checkbox.set_tool_tip(&debug_enable_esf_editor);

    settings_ui.debug_clear_autosave_folder_button.set_tool_tip(&debug_clear_autosaves);
    settings_ui.debug_clear_schema_folder_button.set_tool_tip(&debug_clear_schemas);
    settings_ui.debug_clear_layout_settings_button.set_tool_tip(&debug_clear_layout);

    //-----------------------------------------------//
    // `Diagnostics` tips.
    //-----------------------------------------------//
    let diagnostics_trigger_diagnostics_on_open_tip = qtr("tt_diagnostics_trigger_diagnostics_on_open_tip");
    let diagnostics_trigger_diagnostics_on_table_edit_tip = qtr("tt_diagnostics_trigger_diagnostics_on_table_edit_tip");

    settings_ui.diagnostics_diagnostics_trigger_on_open_label.set_tool_tip(&diagnostics_trigger_diagnostics_on_open_tip);
    settings_ui.diagnostics_diagnostics_trigger_on_open_checkbox.set_tool_tip(&diagnostics_trigger_diagnostics_on_open_tip);
    settings_ui.diagnostics_diagnostics_trigger_on_table_edit_label.set_tool_tip(&diagnostics_trigger_diagnostics_on_table_edit_tip);
    settings_ui.diagnostics_diagnostics_trigger_on_table_edit_checkbox.set_tool_tip(&diagnostics_trigger_diagnostics_on_table_edit_tip);
}
