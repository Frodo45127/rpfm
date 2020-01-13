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

use crate::QString;
use crate::settings_ui::SettingsUI;

/// This function sets the status bar tip for all the actions in the provided `SettingsUI`.
pub fn set_tips(settings_ui: &SettingsUI) {

    //-----------------------------------------------//
    // `UI` tips.
    //-----------------------------------------------//
    let ui_global_use_dark_theme_tip = qtr("tt_ui_global_use_dark_theme_tip");

    let ui_table_adjust_columns_to_content_tip = qtr("tt_ui_table_adjust_columns_to_content_tip");
    let ui_table_disable_combos_tip = qtr("tt_ui_table_disable_combos_tip");
    let ui_table_extend_last_column_tip = qtr("tt_ui_table_extend_last_column_tip");
    let ui_table_remember_column_sorting_tip = qtr("tt_ui_table_remember_column_sorting_tip");
    let ui_table_remember_column_visual_order_tip = qtr("tt_ui_table_remember_column_visual_order_tip");
    let ui_table_remember_table_state_permanently_tip = qtr("tt_ui_table_remember_table_state_permanently_tip");

    let ui_window_start_maximized_tip = qtr("tt_ui_window_start_maximized_tip");

    unsafe { settings_ui.ui_global_use_dark_theme_label.as_mut().unwrap().set_tool_tip(&ui_global_use_dark_theme_tip); }
    unsafe { settings_ui.ui_global_use_dark_theme_checkbox.as_mut().unwrap().set_tool_tip(&ui_global_use_dark_theme_tip); }
    unsafe { settings_ui.ui_table_adjust_columns_to_content_label.as_mut().unwrap().set_tool_tip(&ui_table_adjust_columns_to_content_tip); }
    unsafe { settings_ui.ui_table_adjust_columns_to_content_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_adjust_columns_to_content_tip); }
    unsafe { settings_ui.ui_table_disable_combos_label.as_mut().unwrap().set_tool_tip(&ui_table_disable_combos_tip); }
    unsafe { settings_ui.ui_table_disable_combos_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_disable_combos_tip); }
    unsafe { settings_ui.ui_table_extend_last_column_label.as_mut().unwrap().set_tool_tip(&ui_table_extend_last_column_tip); }
    unsafe { settings_ui.ui_table_extend_last_column_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_extend_last_column_tip); }
    unsafe { settings_ui.ui_table_remember_column_sorting_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_sorting_tip); }
    unsafe { settings_ui.ui_table_remember_column_sorting_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_sorting_tip); }
    unsafe { settings_ui.ui_table_remember_column_visual_order_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_visual_order_tip); }
    unsafe { settings_ui.ui_table_remember_column_visual_order_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_visual_order_tip); }
    unsafe { settings_ui.ui_table_remember_table_state_permanently_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_table_state_permanently_tip); }
    unsafe { settings_ui.ui_table_remember_table_state_permanently_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_table_state_permanently_tip); }
    unsafe { settings_ui.ui_window_start_maximized_label.as_mut().unwrap().set_tool_tip(&ui_window_start_maximized_tip); }
    unsafe { settings_ui.ui_window_start_maximized_checkbox.as_mut().unwrap().set_tool_tip(&ui_window_start_maximized_tip); }

    //-----------------------------------------------//
    // `Extra` tips.
    //-----------------------------------------------//

    let extra_network_check_updates_on_start_tip = qtr("tt_extra_network_check_updates_on_start_tip");
    let extra_network_check_schema_updates_on_start_tip = qtr("tt_extra_network_check_schema_updates_on_start_tip");
    let extra_packfile_allow_editing_of_ca_packfiles_tip = qtr("tt_extra_packfile_allow_editing_of_ca_packfiles_tip");
    let extra_packfile_optimize_not_renamed_packedfiles_tip = qtr("tt_extra_packfile_optimize_not_renamed_packedfiles_tip");
    let extra_packfile_use_dependency_checker_tip = qtr("tt_extra_packfile_use_dependency_checker_tip");
    let extra_packfile_use_lazy_loading_tip = qtr("tt_extra_packfile_use_lazy_loading_tip");

    unsafe { settings_ui.extra_network_check_updates_on_start_label.as_mut().unwrap().set_tool_tip(&extra_network_check_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_updates_on_start_checkbox.as_mut().unwrap().set_tool_tip(&extra_network_check_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_schema_updates_on_start_label.as_mut().unwrap().set_tool_tip(&extra_network_check_schema_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_schema_updates_on_start_checkbox.as_mut().unwrap().set_tool_tip(&extra_network_check_schema_updates_on_start_tip); }
    unsafe { settings_ui.extra_packfile_allow_editing_of_ca_packfiles_label.as_mut().unwrap().set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip); }
    unsafe { settings_ui.extra_packfile_allow_editing_of_ca_packfiles_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip); }
    unsafe { settings_ui.extra_packfile_optimize_not_renamed_packedfiles_label.as_mut().unwrap().set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip); }
    unsafe { settings_ui.extra_packfile_optimize_not_renamed_packedfiles_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip); }
    unsafe { settings_ui.extra_packfile_use_dependency_checker_label.as_mut().unwrap().set_tool_tip(&extra_packfile_use_dependency_checker_tip); }
    unsafe { settings_ui.extra_packfile_use_dependency_checker_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_use_dependency_checker_tip); }
    unsafe { settings_ui.extra_packfile_use_lazy_loading_label.as_mut().unwrap().set_tool_tip(&extra_packfile_use_lazy_loading_tip); }
    unsafe { settings_ui.extra_packfile_use_lazy_loading_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_use_lazy_loading_tip); }

    //-----------------------------------------------//
    // `Debug` tips.
    //-----------------------------------------------//
    let debug_check_for_missing_table_definitions_tip = qtr("tt_debug_check_for_missing_table_definitions_tip");

    unsafe { settings_ui.debug_check_for_missing_table_definitions_label.as_mut().unwrap().set_tool_tip(&debug_check_for_missing_table_definitions_tip); }
    unsafe { settings_ui.debug_check_for_missing_table_definitions_checkbox.as_mut().unwrap().set_tool_tip(&debug_check_for_missing_table_definitions_tip); }
}
