//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code to deal with the settings used to configure this program.

use qt_core::QBox;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;

use crate::app_ui::AppUI;
use crate::settings_helpers::*;

//-------------------------------------------------------------------------------//
//                         Setting-related functions
//-------------------------------------------------------------------------------//

pub unsafe fn set_setting_if_new_string(q_settings: &QBox<QSettings>, setting: &str, value: &str) {
    if !q_settings.value_1a(&QString::from_std_str(setting)).is_valid() {
        q_settings.set_value(&QString::from_std_str(setting), &QVariant::from_q_string(&QString::from_std_str(value)));
    }
}

pub unsafe fn init_app_exclusive_settings(app_ui: &AppUI) {

    // Colours.
    let q_settings = qt_core::QSettings::new();
    set_setting_if_new_string(&q_settings, "colour_light_table_added", "#87ca00");
    set_setting_if_new_string(&q_settings, "colour_light_table_modified", "#e67e22");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_error", "#ff0000");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_warning", "#bebe00");
    set_setting_if_new_string(&q_settings, "colour_light_diagnostic_info", "#55aaff");
    set_setting_if_new_string(&q_settings, "colour_dark_table_added", "#00ff00");
    set_setting_if_new_string(&q_settings, "colour_dark_table_modified", "#e67e22");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_error", "#ff0000");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_warning", "#cece67");
    set_setting_if_new_string(&q_settings, "colour_dark_diagnostic_info", "#55aaff");
    q_settings.sync();

    // These settings need to use QSettings because they're read in the C++ side.
    settings_set_raw_data("originalGeometry", &app_ui.main_window().save_geometry().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());
    settings_set_raw_data("originalWindowState", &app_ui.main_window().save_state_0a().as_slice().iter().map(|x| *x as u8).collect::<Vec<_>>());

    // This one needs to be checked here, due to how the ui works.
    app_ui.menu_bar_debug().menu_action().set_visible(settings_bool("enable_debug_menu"));
}
