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
This module contains the code to build/use the ***Settings*** UI.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::{QDialogButtonBox, q_dialog_button_box, q_dialog_button_box::ButtonRole};
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_gui::QGuiApplication;
use qt_gui::QStandardItemModel;

use qt_core::QFlags;
use qt_core::QString;

use cpp_core::CastInto;
use cpp_core::MutPtr;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::settings::Settings;

use crate::AppUI;
use crate::{Locale, locale::{qtr, qtre}};
use crate::SETTINGS;
use crate::utils::create_grid_layout;
use self::slots::SettingsUISlots;

mod connections;
mod slots;
mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the widgets used in the Settings Window.
#[derive(Clone)]
pub struct SettingsUI {

    //-------------------------------------------------------------------------------//
    // `Dialog` window.
    //-------------------------------------------------------------------------------//
    pub dialog: MutPtr<QDialog>,

    //-------------------------------------------------------------------------------//
    // `Path` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub paths_mymod_label: MutPtr<QLabel>,
    pub paths_mymod_line_edit: MutPtr<QLineEdit>,
    pub paths_mymod_button: MutPtr<QPushButton>,
    pub paths_games_labels: BTreeMap<String, MutPtr<QLabel>>,
    pub paths_games_line_edits: BTreeMap<String, MutPtr<QLineEdit>>,
    pub paths_games_buttons: BTreeMap<String, MutPtr<QPushButton>>,

    //-------------------------------------------------------------------------------//
    // `UI` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub ui_language_label: MutPtr<QLabel>,
    pub ui_global_use_dark_theme_label: MutPtr<QLabel>,
    pub ui_table_adjust_columns_to_content_label: MutPtr<QLabel>,
    pub ui_table_disable_combos_label: MutPtr<QLabel>,
    pub ui_table_extend_last_column_label: MutPtr<QLabel>,
    pub ui_table_tight_table_mode_label: MutPtr<QLabel>,
    pub ui_window_start_maximized_label: MutPtr<QLabel>,
    pub ui_window_hide_background_icon_label: MutPtr<QLabel>,

    pub ui_language_combobox: MutPtr<QComboBox>,
    pub ui_global_use_dark_theme_checkbox: MutPtr<QCheckBox>,
    pub ui_table_adjust_columns_to_content_checkbox: MutPtr<QCheckBox>,
    pub ui_table_disable_combos_checkbox: MutPtr<QCheckBox>,
    pub ui_table_extend_last_column_checkbox: MutPtr<QCheckBox>,
    pub ui_table_tight_table_mode_checkbox: MutPtr<QCheckBox>,
    pub ui_window_start_maximized_checkbox: MutPtr<QCheckBox>,
    pub ui_window_hide_background_icon_checkbox: MutPtr<QCheckBox>,

    //-------------------------------------------------------------------------------//
    // `Extra` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub extra_global_default_game_label: MutPtr<QLabel>,
    pub extra_network_check_updates_on_start_label: MutPtr<QLabel>,
    pub extra_network_check_schema_updates_on_start_label: MutPtr<QLabel>,
    pub extra_packfile_allow_editing_of_ca_packfiles_label: MutPtr<QLabel>,
    pub extra_packfile_optimize_not_renamed_packedfiles_label: MutPtr<QLabel>,
    pub extra_packfile_use_dependency_checker_label: MutPtr<QLabel>,
    pub extra_packfile_use_lazy_loading_label: MutPtr<QLabel>,
    pub extra_disable_uuid_regeneration_on_db_tables_label: MutPtr<QLabel>,

    pub extra_global_default_game_combobox: MutPtr<QComboBox>,
    pub extra_network_check_updates_on_start_checkbox: MutPtr<QCheckBox>,
    pub extra_network_check_schema_updates_on_start_checkbox: MutPtr<QCheckBox>,
    pub extra_packfile_allow_editing_of_ca_packfiles_checkbox: MutPtr<QCheckBox>,
    pub extra_packfile_optimize_not_renamed_packedfiles_checkbox: MutPtr<QCheckBox>,
    pub extra_packfile_use_dependency_checker_checkbox: MutPtr<QCheckBox>,
    pub extra_packfile_use_lazy_loading_checkbox: MutPtr<QCheckBox>,
    pub extra_disable_uuid_regeneration_on_db_tables_checkbox: MutPtr<QCheckBox>,

    //-------------------------------------------------------------------------------//
    // `Debug` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub debug_check_for_missing_table_definitions_label: MutPtr<QLabel>,
    pub debug_check_for_missing_table_definitions_checkbox: MutPtr<QCheckBox>,
    pub debug_enable_debug_menu_label: MutPtr<QLabel>,
    pub debug_enable_debug_menu_checkbox: MutPtr<QCheckBox>,

    //-------------------------------------------------------------------------------//
    // `ButtonBox` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub button_box_restore_default_button: MutPtr<QPushButton>,
    pub button_box_text_editor_settings_button: MutPtr<QPushButton>,
    pub button_box_shortcuts_button: MutPtr<QPushButton>,
    pub button_box_font_settings_button: MutPtr<QPushButton>,
    pub button_box_cancel_button: MutPtr<QPushButton>,
    pub button_box_accept_button: MutPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUI`.
impl SettingsUI {

    /// This function creates a ***Settings*** dialog, execute it, and returns a new `Settings`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(app_ui: &mut AppUI) -> Option<Settings> {
        let mut settings_ui = Self::new_with_parent(app_ui.main_window);
        let slots = SettingsUISlots::new(&mut settings_ui);

        connections::set_connections(&settings_ui, &slots);
        tips::set_tips(&mut settings_ui);
        settings_ui.load(&SETTINGS.read().unwrap());

        if settings_ui.dialog.exec() == 1 { Some(settings_ui.save()) }
        else { None }
    }

    /// This function creates a new `SettingsUI` and links it to the provided parent.
    unsafe fn new_with_parent(parent: impl CastInto<MutPtr<QWidget>>) -> Self {

        // Initialize and configure the settings window.
        let mut dialog = QDialog::new_1a(parent).into_ptr();
        dialog.set_window_title(&qtr("settings_title"));
        dialog.set_modal(true);
        dialog.resize_2a(750, 0);

        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        main_grid.set_contents_margins_4a(4, 0, 4, 4);
        main_grid.set_spacing(4);

        //-----------------------------------------------//
        // `Paths` Frame.
        //-----------------------------------------------//
        let paths_frame = QGroupBox::from_q_string(&qtr("settings_paths_title")).into_ptr();
        let mut paths_grid = create_grid_layout(paths_frame.static_upcast_mut());
        paths_grid.set_contents_margins_4a(4, 0, 4, 0);

        // Create the MyMod's path stuff,
        let mut paths_mymod_label = QLabel::from_q_string(&qtr("settings_paths_mymod"));
        let mut paths_mymod_line_edit = QLineEdit::new();
        let mut paths_mymod_button = QPushButton::from_q_string(&QString::from_std_str("..."));
        paths_mymod_line_edit.set_placeholder_text(&qtr("settings_paths_mymod_ph"));

        paths_grid.add_widget_5a(&mut paths_mymod_label, 0, 0, 1, 1);
        paths_grid.add_widget_5a(&mut paths_mymod_line_edit, 0, 1, 1, 1);
        paths_grid.add_widget_5a(&mut paths_mymod_button, 0, 2, 1, 1);

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_games_labels = BTreeMap::new();
        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();
        for (index, (folder_name, game_supported)) in SUPPORTED_GAMES.iter().enumerate() {
            let mut game_label = QLabel::from_q_string(&qtre("settings_game_label", &[&game_supported.display_name]));
            let mut game_line_edit = QLineEdit::new();
            let mut game_button = QPushButton::from_q_string(&QString::from_std_str("..."));
            game_line_edit.set_placeholder_text(&qtre("settings_game_line_ph", &[&game_supported.display_name]));

            paths_grid.add_widget_5a(&mut game_label, (index + 1) as i32, 0, 1, 1);
            paths_grid.add_widget_5a(&mut game_line_edit, (index + 1) as i32, 1, 1, 1);
            paths_grid.add_widget_5a(&mut game_button, (index + 1) as i32, 2, 1, 1);

            // Add the LineEdit and Button to the list.
            paths_games_labels.insert((*folder_name).to_string(), game_label.into_ptr());
            paths_games_line_edits.insert((*folder_name).to_string(), game_line_edit.into_ptr());
            paths_games_buttons.insert((*folder_name).to_string(), game_button.into_ptr());
        }

        main_grid.add_widget_5a(paths_frame, 0, 0, 1, 2);

        //-----------------------------------------------//
        // `UI` Frame.
        //-----------------------------------------------//
        let ui_frame = QGroupBox::from_q_string(&qtr("settings_ui_title")).into_ptr();
        let mut ui_grid = create_grid_layout(ui_frame.static_upcast_mut());
        ui_grid.set_contents_margins_4a(4, 0, 4, 0);
        ui_grid.set_spacing(4);
        ui_grid.set_row_stretch(99, 10);

        // Create the "UI - TableView" frame and grid.
        let ui_table_view_frame = QGroupBox::from_q_string(&qtr("settings_table_title")).into_ptr();
        let mut ui_table_view_grid = create_grid_layout(ui_table_view_frame.static_upcast_mut());
        ui_table_view_grid.set_contents_margins_4a(4, 0, 4, 0);
        ui_table_view_grid.set_spacing(4);
        ui_table_view_grid.set_row_stretch(99, 10);

        let mut ui_language_label = QLabel::from_q_string(&qtr("settings_ui_language"));
        let mut ui_global_use_dark_theme_label = QLabel::from_q_string(&qtr("settings_ui_dark_theme"));
        let mut ui_table_adjust_columns_to_content_label = QLabel::from_q_string(&qtr("settings_ui_table_adjust_columns_to_content"));
        let mut ui_table_disable_combos_label = QLabel::from_q_string(&qtr("settings_ui_table_disable_combos"));
        let mut ui_table_extend_last_column_label = QLabel::from_q_string(&qtr("settings_ui_table_extend_last_column_label"));
        let mut ui_table_tight_table_mode_label = QLabel::from_q_string(&qtr("settings_ui_table_tight_table_mode_label"));
        let mut ui_window_start_maximized_label = QLabel::from_q_string(&qtr("settings_ui_window_start_maximized_label"));
        let mut ui_window_hide_background_icon_label = QLabel::from_q_string(&qtr("settings_ui_window_hide_background_icon"));

        let mut ui_language_combobox = QComboBox::new_0a();
        let mut ui_global_use_dark_theme_checkbox = QCheckBox::new();
        let mut ui_table_adjust_columns_to_content_checkbox = QCheckBox::new();
        let mut ui_table_disable_combos_checkbox = QCheckBox::new();
        let mut ui_table_extend_last_column_checkbox = QCheckBox::new();
        let mut ui_table_tight_table_mode_checkbox = QCheckBox::new();
        let mut ui_window_start_maximized_checkbox = QCheckBox::new();
        let mut ui_window_hide_background_icon_checkbox = QCheckBox::new();

        let ui_language_model = QStandardItemModel::new_0a().into_ptr();
        ui_language_combobox.set_model(ui_language_model);
        if let Ok(locales) = Locale::get_available_locales() {
            for (language, _) in locales {
                ui_language_combobox.add_item_q_string(&QString::from_std_str(&language));
            }
        }

        // Add all Label/Checkboxes to the grid.
        if cfg!(not(target_os = "linux")) {
            ui_grid.add_widget_5a(&mut ui_global_use_dark_theme_label, 0, 0, 1, 1);
            ui_grid.add_widget_5a(&mut ui_global_use_dark_theme_checkbox, 0, 1, 1, 1);
        }

        ui_grid.add_widget_5a(&mut ui_window_start_maximized_label, 1, 0, 1, 1);
        ui_grid.add_widget_5a(&mut ui_window_start_maximized_checkbox, 1, 1, 1, 1);

        ui_grid.add_widget_5a(&mut ui_window_hide_background_icon_label, 2, 0, 1, 1);
        ui_grid.add_widget_5a(&mut ui_window_hide_background_icon_checkbox, 2, 1, 1, 1);

        ui_grid.add_widget_5a(&mut ui_language_label, 3, 0, 1, 1);
        ui_grid.add_widget_5a(&mut ui_language_combobox, 3, 1, 1, 1);

        ui_table_view_grid.add_widget_5a(&mut ui_table_adjust_columns_to_content_label, 0, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&mut ui_table_adjust_columns_to_content_checkbox, 0, 1, 1, 1);

        ui_table_view_grid.add_widget_5a(&mut ui_table_disable_combos_label, 1, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&mut ui_table_disable_combos_checkbox, 1, 1, 1, 1);

        ui_table_view_grid.add_widget_5a(&mut ui_table_extend_last_column_label, 2, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&mut ui_table_extend_last_column_checkbox, 2, 1, 1, 1);

        ui_table_view_grid.add_widget_5a(&mut ui_table_tight_table_mode_label, 3, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&mut ui_table_tight_table_mode_checkbox, 3, 1, 1, 1);

        ui_grid.add_widget_5a(ui_table_view_frame, 99, 0, 1, 2);
        main_grid.add_widget_5a(ui_frame, 1, 0, 2, 1);

        //-----------------------------------------------//
        // `Extra` Frame.
        //-----------------------------------------------//
        let extra_frame = QGroupBox::from_q_string(&qtr("settings_extra_title")).into_ptr();
        let mut extra_grid = create_grid_layout(extra_frame.static_upcast_mut());
        extra_grid.set_contents_margins_4a(4, 0, 4, 0);
        extra_grid.set_spacing(4);
        extra_grid.set_row_stretch(99, 10);

        // Create the "Default Game" Label and ComboBox.
        let mut extra_global_default_game_label = QLabel::from_q_string(&qtr("settings_default_game"));
        let mut extra_global_default_game_combobox = QComboBox::new_0a();
        let extra_global_default_game_model = QStandardItemModel::new_0a().into_ptr();
        extra_global_default_game_combobox.set_model(extra_global_default_game_model);
        for (_, game) in SUPPORTED_GAMES.iter() { extra_global_default_game_combobox.add_item_q_string(&QString::from_std_str(&game.display_name)); }

        // Create the aditional Labels/CheckBoxes.
        let mut extra_network_check_updates_on_start_label = QLabel::from_q_string(&qtr("settings_check_updates_on_start"));
        let mut extra_network_check_schema_updates_on_start_label = QLabel::from_q_string(&qtr("settings_check_schema_updates_on_start"));
        let mut extra_packfile_allow_editing_of_ca_packfiles_label = QLabel::from_q_string(&qtr("settings_allow_editing_of_ca_packfiles"));
        let mut extra_packfile_optimize_not_renamed_packedfiles_label = QLabel::from_q_string(&qtr("settings_optimize_not_renamed_packedfiles"));
        let mut extra_packfile_use_dependency_checker_label = QLabel::from_q_string(&qtr("settings_use_dependency_checker"));
        let mut extra_packfile_use_lazy_loading_label = QLabel::from_q_string(&qtr("settings_use_lazy_loading"));
        let mut extra_disable_uuid_regeneration_on_db_tables_label = QLabel::from_q_string(&qtr("settings_disable_uuid_regeneration_tables"));

        let mut extra_network_check_updates_on_start_checkbox = QCheckBox::new();
        let mut extra_network_check_schema_updates_on_start_checkbox = QCheckBox::new();
        let mut extra_packfile_allow_editing_of_ca_packfiles_checkbox = QCheckBox::new();
        let mut extra_packfile_optimize_not_renamed_packedfiles_checkbox = QCheckBox::new();
        let mut extra_packfile_use_dependency_checker_checkbox = QCheckBox::new();
        let mut extra_packfile_use_lazy_loading_checkbox = QCheckBox::new();
        let mut extra_disable_uuid_regeneration_on_db_tables_checkbox = QCheckBox::new();

        extra_grid.add_widget_5a(&mut extra_global_default_game_label, 0, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_global_default_game_combobox, 0, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_network_check_updates_on_start_label, 1, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_network_check_updates_on_start_checkbox, 1, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_network_check_schema_updates_on_start_label, 2, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_network_check_schema_updates_on_start_checkbox, 2, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_packfile_allow_editing_of_ca_packfiles_label, 3, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_packfile_allow_editing_of_ca_packfiles_checkbox, 3, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_packfile_optimize_not_renamed_packedfiles_label, 4, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_packfile_optimize_not_renamed_packedfiles_checkbox, 4, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_packfile_use_dependency_checker_label, 5, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_packfile_use_dependency_checker_checkbox, 5, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_packfile_use_lazy_loading_label, 6, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_packfile_use_lazy_loading_checkbox, 6, 1, 1, 1);

        extra_grid.add_widget_5a(&mut extra_disable_uuid_regeneration_on_db_tables_label, 7, 0, 1, 1);
        extra_grid.add_widget_5a(&mut extra_disable_uuid_regeneration_on_db_tables_checkbox, 7, 1, 1, 1);

        main_grid.add_widget_5a(extra_frame, 1, 1, 1, 1);

        //-----------------------------------------------//
        // `Debug` Frame.
        //-----------------------------------------------//
        let debug_frame = QGroupBox::from_q_string(&qtr("settings_debug_title")).into_ptr();
        let mut debug_grid = create_grid_layout(debug_frame.static_upcast_mut());
        debug_grid.set_contents_margins_4a(4, 0, 4, 0);
        debug_grid.set_spacing(4);
        debug_grid.set_row_stretch(99, 10);

        let mut debug_check_for_missing_table_definitions_label = QLabel::from_q_string(&qtr("settings_debug_missing_table"));
        let mut debug_enable_debug_menu_label = QLabel::from_q_string(&qtr("settings_debug_enable_debug_menu"));

        let mut debug_check_for_missing_table_definitions_checkbox = QCheckBox::new();
        let mut debug_enable_debug_menu_checkbox = QCheckBox::new();

        debug_grid.add_widget_5a(&mut debug_check_for_missing_table_definitions_label, 0, 0, 1, 1);
        debug_grid.add_widget_5a(&mut debug_check_for_missing_table_definitions_checkbox, 0, 1, 1, 1);

        debug_grid.add_widget_5a(&mut debug_enable_debug_menu_label, 1, 0, 1, 1);
        debug_grid.add_widget_5a(&mut debug_enable_debug_menu_checkbox, 1, 1, 1, 1);

        main_grid.add_widget_5a(debug_frame, 2, 1, 1, 1);

        //-----------------------------------------------//
        // `ButtonBox` Button Box.
        //-----------------------------------------------//
        let mut button_box = QDialogButtonBox::new();
        let mut button_box_shortcuts_button = QPushButton::from_q_string(&qtr("shortcut_title"));
        let mut button_box_text_editor_settings_button = QPushButton::from_q_string(&qtr("settings_text_title"));
        let mut button_box_font_settings_button = QPushButton::from_q_string(&qtr("settings_font_title"));

        let button_box_restore_default_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::RestoreDefaults);
        button_box.add_button_q_abstract_button_button_role(&mut button_box_shortcuts_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&mut button_box_text_editor_settings_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&mut button_box_font_settings_button, ButtonRole::ResetRole);
        let button_box_cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let button_box_accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);

        main_grid.add_widget_5a(button_box.into_ptr(), 3, 0, 1, 2);

        // Now, we build the `SettingsUI` struct and return it.
        Self {

            //-------------------------------------------------------------------------------//
            // `Dialog` window.
            //-------------------------------------------------------------------------------//
            dialog,

            //-------------------------------------------------------------------------------//
            // `Path` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            paths_mymod_label: paths_mymod_label.into_ptr(),
            paths_mymod_line_edit: paths_mymod_line_edit.into_ptr(),
            paths_mymod_button: paths_mymod_button.into_ptr(),
            paths_games_labels,
            paths_games_line_edits,
            paths_games_buttons,

            //-------------------------------------------------------------------------------//
            // `UI` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            ui_language_label: ui_language_label.into_ptr(),
            ui_global_use_dark_theme_label: ui_global_use_dark_theme_label.into_ptr(),
            ui_table_adjust_columns_to_content_label: ui_table_adjust_columns_to_content_label.into_ptr(),
            ui_table_disable_combos_label: ui_table_disable_combos_label.into_ptr(),
            ui_table_extend_last_column_label: ui_table_extend_last_column_label.into_ptr(),
            ui_table_tight_table_mode_label: ui_table_tight_table_mode_label.into_ptr(),
            ui_window_start_maximized_label: ui_window_start_maximized_label.into_ptr(),
            ui_window_hide_background_icon_label: ui_window_hide_background_icon_label.into_ptr(),

            ui_language_combobox: ui_language_combobox.into_ptr(),
            ui_global_use_dark_theme_checkbox: ui_global_use_dark_theme_checkbox.into_ptr(),
            ui_table_adjust_columns_to_content_checkbox: ui_table_adjust_columns_to_content_checkbox.into_ptr(),
            ui_table_disable_combos_checkbox: ui_table_disable_combos_checkbox.into_ptr(),
            ui_table_extend_last_column_checkbox: ui_table_extend_last_column_checkbox.into_ptr(),
            ui_table_tight_table_mode_checkbox: ui_table_tight_table_mode_checkbox.into_ptr(),
            ui_window_start_maximized_checkbox: ui_window_start_maximized_checkbox.into_ptr(),
            ui_window_hide_background_icon_checkbox: ui_window_hide_background_icon_checkbox.into_ptr(),

            //-------------------------------------------------------------------------------//
            // `Extra` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            extra_global_default_game_label: extra_global_default_game_label.into_ptr(),
            extra_network_check_updates_on_start_label: extra_network_check_updates_on_start_label.into_ptr(),
            extra_network_check_schema_updates_on_start_label: extra_network_check_schema_updates_on_start_label.into_ptr(),
            extra_packfile_allow_editing_of_ca_packfiles_label: extra_packfile_allow_editing_of_ca_packfiles_label.into_ptr(),
            extra_packfile_optimize_not_renamed_packedfiles_label: extra_packfile_optimize_not_renamed_packedfiles_label.into_ptr(),
            extra_packfile_use_dependency_checker_label: extra_packfile_use_dependency_checker_label.into_ptr(),
            extra_packfile_use_lazy_loading_label: extra_packfile_use_lazy_loading_label.into_ptr(),
            extra_disable_uuid_regeneration_on_db_tables_label: extra_disable_uuid_regeneration_on_db_tables_label.into_ptr(),

            extra_global_default_game_combobox: extra_global_default_game_combobox.into_ptr(),
            extra_network_check_updates_on_start_checkbox: extra_network_check_updates_on_start_checkbox.into_ptr(),
            extra_network_check_schema_updates_on_start_checkbox: extra_network_check_schema_updates_on_start_checkbox.into_ptr(),
            extra_packfile_allow_editing_of_ca_packfiles_checkbox: extra_packfile_allow_editing_of_ca_packfiles_checkbox.into_ptr(),
            extra_packfile_optimize_not_renamed_packedfiles_checkbox: extra_packfile_optimize_not_renamed_packedfiles_checkbox.into_ptr(),
            extra_packfile_use_dependency_checker_checkbox: extra_packfile_use_dependency_checker_checkbox.into_ptr(),
            extra_packfile_use_lazy_loading_checkbox: extra_packfile_use_lazy_loading_checkbox.into_ptr(),
            extra_disable_uuid_regeneration_on_db_tables_checkbox: extra_disable_uuid_regeneration_on_db_tables_checkbox.into_ptr(),

            //-------------------------------------------------------------------------------//
            // `Debug` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            debug_check_for_missing_table_definitions_label: debug_check_for_missing_table_definitions_label.into_ptr(),
            debug_check_for_missing_table_definitions_checkbox: debug_check_for_missing_table_definitions_checkbox.into_ptr(),
            debug_enable_debug_menu_label: debug_enable_debug_menu_label.into_ptr(),
            debug_enable_debug_menu_checkbox: debug_enable_debug_menu_checkbox.into_ptr(),
            //-------------------------------------------------------------------------------//
            // `ButtonBox` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            button_box_restore_default_button,
            button_box_text_editor_settings_button: button_box_text_editor_settings_button.into_ptr(),
            button_box_shortcuts_button: button_box_shortcuts_button.into_ptr(),
            button_box_font_settings_button: button_box_font_settings_button.into_ptr(),
            button_box_cancel_button,
            button_box_accept_button,
        }
    }

    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub unsafe fn load(&mut self, settings: &Settings) {

        // Load the MyMod Path, if exists.
        self.paths_mymod_line_edit.set_text(&QString::from_std_str(settings.paths["mymods_base_path"].clone().unwrap_or_else(PathBuf::new).to_string_lossy()));

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter_mut() {
            path.set_text(&QString::from_std_str(&settings.paths[key].clone().unwrap_or_else(PathBuf::new).to_string_lossy()));
        }

        // Get the default game.
        for (index, (folder_name,_)) in SUPPORTED_GAMES.iter().enumerate() {
            if *folder_name == settings.settings_string["default_game"] {
                self.extra_global_default_game_combobox.set_current_index(index as i32);
                break;
            }
        }

        let language_selected = settings.settings_string["language"].split('_').collect::<Vec<&str>>()[0];
        for (index, (language,_)) in Locale::get_available_locales().unwrap().iter().enumerate() {
            if *language == language_selected {
                self.ui_language_combobox.set_current_index(index as i32);
                break;
            }
        }

        // Load the UI Stuff.
        self.ui_global_use_dark_theme_checkbox.set_checked(settings.settings_bool["use_dark_theme"]);
        self.ui_table_adjust_columns_to_content_checkbox.set_checked(settings.settings_bool["adjust_columns_to_content"]);
        self.ui_table_disable_combos_checkbox.set_checked(settings.settings_bool["disable_combos_on_tables"]);
        self.ui_table_extend_last_column_checkbox.set_checked(settings.settings_bool["extend_last_column_on_tables"]);
        self.ui_table_tight_table_mode_checkbox.set_checked(settings.settings_bool["tight_table_mode"]);
        self.ui_window_start_maximized_checkbox.set_checked(settings.settings_bool["start_maximized"]);
        self.ui_window_hide_background_icon_checkbox.set_checked(settings.settings_bool["hide_background_icon"]);

        // Load the Extra Stuff.
        self.extra_network_check_updates_on_start_checkbox.set_checked(settings.settings_bool["check_updates_on_start"]);
        self.extra_network_check_schema_updates_on_start_checkbox.set_checked(settings.settings_bool["check_schema_updates_on_start"]);
        self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.set_checked(settings.settings_bool["allow_editing_of_ca_packfiles"]);
        self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.set_checked(settings.settings_bool["optimize_not_renamed_packedfiles"]);
        self.extra_packfile_use_dependency_checker_checkbox.set_checked(settings.settings_bool["use_dependency_checker"]);
        self.extra_packfile_use_lazy_loading_checkbox.set_checked(settings.settings_bool["use_lazy_loading"]);
        self.extra_disable_uuid_regeneration_on_db_tables_checkbox.set_checked(settings.settings_bool["disable_uuid_regeneration_on_db_tables"]);

        // Load the Debug Stuff.
        self.debug_check_for_missing_table_definitions_checkbox.set_checked(settings.settings_bool["check_for_missing_table_definitions"]);
        self.debug_enable_debug_menu_checkbox.set_checked(settings.settings_bool["enable_debug_menu"]);
    }

    /// This function saves the data from our `SettingsUI` into a `Settings` and return it.
    pub unsafe fn save(&self) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new();

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let mymod_new_path = PathBuf::from(self.paths_mymod_line_edit.text().to_std_string());
        settings.paths.insert("mymods_base_path".to_owned(), if mymod_new_path.is_dir() { Some(mymod_new_path) } else { None });

        // For each entry, we check if it's a valid directory and save it into Settings.
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let new_path = PathBuf::from(line_edit.text().to_std_string());
            settings.paths.insert(key.to_owned(), if new_path.is_dir() { Some(new_path) } else { None });
        }

        // We get his game's folder, depending on the selected game.
        let mut game = self.extra_global_default_game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        settings.settings_string.insert("default_game".to_owned(), game);

        // We need to store the full locale filename, not just the visible name!
        let mut language = self.ui_language_combobox.current_text().to_std_string();
        if let Some(index) = language.find('&') { language.remove(index); }
        if let Some((_, locale)) = Locale::get_available_locales().unwrap().iter().find(|(x, _)| &language == x) {
            let file_name = format!("{}_{}", language, locale.language);
            settings.settings_string.insert("language".to_owned(), file_name);
        }

        let current_font = QGuiApplication::font();
        settings.settings_string.insert("font_name".to_owned(), current_font.family().to_std_string());
        settings.settings_string.insert("font_size".to_owned(), current_font.point_size().to_string());

        // Get the UI Settings.
        settings.settings_bool.insert("use_dark_theme".to_owned(), self.ui_global_use_dark_theme_checkbox.is_checked());
        settings.settings_bool.insert("adjust_columns_to_content".to_owned(), self.ui_table_adjust_columns_to_content_checkbox.is_checked());
        settings.settings_bool.insert("disable_combos_on_tables".to_owned(), self.ui_table_disable_combos_checkbox.is_checked());
        settings.settings_bool.insert("extend_last_column_on_tables".to_owned(), self.ui_table_extend_last_column_checkbox.is_checked());
        settings.settings_bool.insert("tight_table_mode".to_owned(), self.ui_table_tight_table_mode_checkbox.is_checked());
        settings.settings_bool.insert("start_maximized".to_owned(), self.ui_window_start_maximized_checkbox.is_checked());
        settings.settings_bool.insert("hide_background_icon".to_owned(), self.ui_window_hide_background_icon_checkbox.is_checked());

        // Get the Extra Settings.
        settings.settings_bool.insert("check_updates_on_start".to_owned(), self.extra_network_check_updates_on_start_checkbox.is_checked());
        settings.settings_bool.insert("check_schema_updates_on_start".to_owned(), self.extra_network_check_schema_updates_on_start_checkbox.is_checked());
        settings.settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.is_checked());
        settings.settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.is_checked());
        settings.settings_bool.insert("use_dependency_checker".to_owned(), self.extra_packfile_use_dependency_checker_checkbox.is_checked());
        settings.settings_bool.insert("use_lazy_loading".to_owned(), self.extra_packfile_use_lazy_loading_checkbox.is_checked());
        settings.settings_bool.insert("disable_uuid_regeneration_on_db_tables".to_owned(), self.extra_disable_uuid_regeneration_on_db_tables_checkbox.is_checked());

        // Get the Debug Settings.
        settings.settings_bool.insert("check_for_missing_table_definitions".to_owned(), self.debug_check_for_missing_table_definitions_checkbox.is_checked());
        settings.settings_bool.insert("enable_debug_menu".to_owned(), self.debug_enable_debug_menu_checkbox.is_checked());

        // Return the new Settings.
        settings
    }

    /// This function updates the path you have for the provided game (or mymod, if you pass it `None`)
    /// with the one you select in a `FileDialog`.
    unsafe fn update_entry_path(&self, game: Option<&str>) {

        // Create the `FileDialog` and configure it.
        let mut file_dialog = QFileDialog::from_q_widget_q_string(
            self.dialog,
            &qtr("settings_select_folder"),
        );
        file_dialog.set_file_mode(FileMode::Directory);
        file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let mut line_edit = match game {
            Some(game) => *self.paths_games_line_edits.get(game).unwrap(),
            None => self.paths_mymod_line_edit,
        };

        // Get the old Path, if exists.
        let old_path = line_edit.text().to_std_string();

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            file_dialog.set_directory_q_string(&line_edit.text());
        }

        // Run it and expect a response (1 => Accept, 0 => Cancel).
        if file_dialog.exec() == 1 {

            // Get the path of the selected file.
            let selected_files = file_dialog.selected_files();
            let path = selected_files.at(0);

            // Add the Path to the LineEdit.
            line_edit.set_text(path);
        }
    }
}
