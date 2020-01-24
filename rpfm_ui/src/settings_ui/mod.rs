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

use qt_widgets::abstract_button::AbstractButton;
use qt_widgets::check_box::CheckBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::{dialog_button_box, dialog_button_box::{ButtonRole, DialogButtonBox}};
use qt_widgets::file_dialog::{FileDialog, FileMode, Option::ShowDirsOnly};
use qt_widgets::group_box::GroupBox;
use qt_widgets::label::Label;
use qt_widgets::layout::Layout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;

use cpp_utils::StaticCast;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::settings::Settings;

use crate::AppUI;
use crate::{Locale, locale::{qtr, qtre}};
use crate::QString;
use crate::SETTINGS;
use crate::utils::create_grid_layout_safe;
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
    pub dialog: *mut Dialog,

    //-------------------------------------------------------------------------------//
    // `Path` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub paths_mymod_label: *mut Label,
    pub paths_mymod_line_edit: *mut LineEdit,
    pub paths_mymod_button: *mut PushButton,
    pub paths_games_labels: BTreeMap<String, *mut Label>,
    pub paths_games_line_edits: BTreeMap<String, *mut LineEdit>,
    pub paths_games_buttons: BTreeMap<String, *mut PushButton>,

    //-------------------------------------------------------------------------------//
    // `UI` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub ui_language_label: *mut Label,
    pub ui_global_use_dark_theme_label: *mut Label,
    pub ui_table_adjust_columns_to_content_label: *mut Label,
    pub ui_table_disable_combos_label: *mut Label,
    pub ui_table_extend_last_column_label: *mut Label,
    pub ui_table_remember_column_sorting_label: *mut Label,
    pub ui_table_remember_column_visual_order_label: *mut Label,
    pub ui_table_remember_table_state_permanently_label: *mut Label,
    pub ui_window_start_maximized_label: *mut Label,

    pub ui_language_combobox: *mut ComboBox,
    pub ui_global_use_dark_theme_checkbox: *mut CheckBox,
    pub ui_table_adjust_columns_to_content_checkbox: *mut CheckBox,
    pub ui_table_disable_combos_checkbox: *mut CheckBox,
    pub ui_table_extend_last_column_checkbox: *mut CheckBox,
    pub ui_table_remember_column_sorting_checkbox: *mut CheckBox,
    pub ui_table_remember_column_visual_order_checkbox: *mut CheckBox,
    pub ui_table_remember_table_state_permanently_checkbox: *mut CheckBox,
    pub ui_window_start_maximized_checkbox: *mut CheckBox,

    //-------------------------------------------------------------------------------//
    // `Extra` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub extra_global_default_game_label: *mut Label,
    pub extra_network_check_updates_on_start_label: *mut Label,
    pub extra_network_check_schema_updates_on_start_label: *mut Label,
    pub extra_packfile_allow_editing_of_ca_packfiles_label: *mut Label,
    pub extra_packfile_optimize_not_renamed_packedfiles_label: *mut Label,
    pub extra_packfile_use_dependency_checker_label: *mut Label,
    pub extra_packfile_use_lazy_loading_label: *mut Label,

    pub extra_global_default_game_combobox: *mut ComboBox,
    pub extra_network_check_updates_on_start_checkbox: *mut CheckBox,
    pub extra_network_check_schema_updates_on_start_checkbox: *mut CheckBox,
    pub extra_packfile_allow_editing_of_ca_packfiles_checkbox: *mut CheckBox,
    pub extra_packfile_optimize_not_renamed_packedfiles_checkbox: *mut CheckBox,
    pub extra_packfile_use_dependency_checker_checkbox: *mut CheckBox,
    pub extra_packfile_use_lazy_loading_checkbox: *mut CheckBox,

    //-------------------------------------------------------------------------------//
    // `Debug` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub debug_check_for_missing_table_definitions_label: *mut Label,
    pub debug_check_for_missing_table_definitions_checkbox: *mut CheckBox,
    pub debug_enable_debug_menu_label: *mut Label,
    pub debug_enable_debug_menu_checkbox: *mut CheckBox,

    //-------------------------------------------------------------------------------//
    // `ButtonBox` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub button_box_restore_default_button: *mut PushButton,
    pub button_box_text_editor_settings_button: *mut PushButton,
    pub button_box_shortcuts_button: *mut PushButton,
    pub button_box_cancel_button: *mut PushButton,
    pub button_box_accept_button: *mut PushButton,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUI`.
impl SettingsUI {

    /// This function creates a ***Settings*** dialog, execute it, and returns a new `Settings`, or `None` if you close/cancel the dialog.
    pub fn new(app_ui: &AppUI) -> Option<Settings> {
        let mut settings_ui = Self::new_with_parent(app_ui.main_window as *mut Widget);
        let slots = SettingsUISlots::new(&settings_ui);
        connections::set_connections(&settings_ui, &slots);
        tips::set_tips(&settings_ui);
        settings_ui.load(&SETTINGS.lock().unwrap());

        if unsafe { settings_ui.dialog.as_mut().unwrap().exec() == 1 } { Some(settings_ui.save()) }
        else { None }
    }

    /// This function creates a new `SettingsUI` and links it to the provided parent.
    fn new_with_parent(parent: *mut Widget) -> Self {

        // Initialize and configure the settings window.
        let mut dialog = unsafe { Dialog::new_unsafe(parent) };
        dialog.set_window_title(&qtr("settings_title"));
        dialog.set_modal(true);
        dialog.resize((750, 0));

        let mut main_grid = create_grid_layout_safe();
        main_grid.set_contents_margins((4, 0, 4, 4));
        main_grid.set_spacing(4);

        //-----------------------------------------------//
        // `Paths` Frame.
        //-----------------------------------------------//
        let mut paths_frame = GroupBox::new(&qtr("settings_paths_title"));
        let mut paths_grid = create_grid_layout_safe();
        paths_grid.set_contents_margins((4, 0, 4, 0));

        // Create the MyMod's path stuff,
        let paths_mymod_label = Label::new(&qtr("settings_paths_mymod"));
        let mut paths_mymod_line_edit = LineEdit::new(());
        let paths_mymod_button = PushButton::new(&QString::from_std_str("..."));
        paths_mymod_line_edit.set_placeholder_text(&qtr("settings_paths_mymod_ph"));

        unsafe { paths_grid.add_widget((paths_mymod_label.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { paths_grid.add_widget((paths_mymod_line_edit.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { paths_grid.add_widget((paths_mymod_button.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_games_labels = BTreeMap::new();
        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();
        for (index, (folder_name, game_supported)) in SUPPORTED_GAMES.iter().enumerate() {
            let game_label = Label::new(&qtre("settings_game_label", &[&game_supported.display_name]));
            let mut game_line_edit = LineEdit::new(());
            let game_button = PushButton::new(&QString::from_std_str("..."));
            game_line_edit.set_placeholder_text(&qtre("settings_game_line_ph", &[&game_supported.display_name]));

            unsafe { paths_grid.add_widget((game_label.as_mut_ptr() as *mut Widget, (index + 1) as i32, 0, 1, 1)); }
            unsafe { paths_grid.add_widget((game_line_edit.as_mut_ptr() as *mut Widget, (index + 1) as i32, 1, 1, 1)); }
            unsafe { paths_grid.add_widget((game_button.as_mut_ptr() as *mut Widget, (index + 1) as i32, 2, 1, 1)); }

            // Add the LineEdit and Button to the list.
            paths_games_labels.insert((*folder_name).to_string(), game_label.into_raw());
            paths_games_line_edits.insert((*folder_name).to_string(), game_line_edit.into_raw());
            paths_games_buttons.insert((*folder_name).to_string(), game_button.into_raw());
        }

        unsafe { paths_frame.set_layout(paths_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((paths_frame.into_raw() as *mut Widget, 0, 0, 1, 2)); }

        //-----------------------------------------------//
        // `UI` Frame.
        //-----------------------------------------------//
        let mut ui_frame = GroupBox::new(&qtr("settings_ui_title"));
        let mut ui_grid = create_grid_layout_safe();
        ui_grid.set_contents_margins((4, 0, 4, 0));
        ui_grid.set_spacing(4);
        ui_grid.set_row_stretch(99, 10);

        // Create the "UI - TableView" frame and grid.
        let mut ui_table_view_frame = GroupBox::new(&qtr("settings_table_title"));
        let mut ui_table_view_grid = create_grid_layout_safe();
        ui_table_view_grid.set_contents_margins((4, 0, 4, 0));
        ui_table_view_grid.set_spacing(4);
        ui_table_view_grid.set_row_stretch(99, 10);

        let mut ui_language_label = Label::new(&qtr("settings_ui_language"));
        let mut ui_global_use_dark_theme_label = Label::new(&qtr("settings_ui_dark_theme"));
        let mut ui_table_adjust_columns_to_content_label = Label::new(&qtr("settings_ui_table_adjust_columns_to_content"));
        let mut ui_table_disable_combos_label = Label::new(&qtr("settings_ui_table_disable_combos"));
        let mut ui_table_extend_last_column_label = Label::new(&qtr("settings_ui_table_extend_last_column_label"));
        let mut ui_table_remember_column_sorting_label = Label::new(&qtr("settings_ui_table_remember_column_sorting_label"));
        let mut ui_table_remember_column_visual_order_label = Label::new(&qtr("settings_ui_table_remember_column_visual_order_label"));
        let mut ui_table_remember_table_state_permanently_label = Label::new(&qtr("settings_ui_table_remember_table_state_permanently_label"));
        let mut ui_window_start_maximized_label = Label::new(&qtr("settings_ui_window_start_maximized_label"));

        let mut ui_language_combobox = ComboBox::new();
        let mut ui_global_use_dark_theme_checkbox = CheckBox::new(());
        let mut ui_table_adjust_columns_to_content_checkbox = CheckBox::new(());
        let mut ui_table_disable_combos_checkbox = CheckBox::new(());
        let mut ui_table_extend_last_column_checkbox = CheckBox::new(());
        let mut ui_table_remember_column_sorting_checkbox = CheckBox::new(());
        let mut ui_table_remember_column_visual_order_checkbox = CheckBox::new(());
        let mut ui_table_remember_table_state_permanently_checkbox = CheckBox::new(());
        let mut ui_window_start_maximized_checkbox = CheckBox::new(());

        let ui_language_model = StandardItemModel::new(());
        unsafe { ui_language_combobox.set_model(ui_language_model.into_raw() as *mut AbstractItemModel); }
        if let Ok(locales) = Locale::get_available_locales() {
            for (language, _) in locales {
                ui_language_combobox.add_item(&QString::from_std_str(&language));
            }
        }

        // Add all Label/Checkboxes to the grid.
        if cfg!(not(target_os = "linux")) {
            unsafe { ui_grid.add_widget((ui_global_use_dark_theme_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
            unsafe { ui_grid.add_widget((ui_global_use_dark_theme_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
        }

        unsafe { ui_grid.add_widget((ui_window_start_maximized_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_grid.add_widget((ui_window_start_maximized_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { ui_grid.add_widget((ui_language_label.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { ui_grid.add_widget((ui_language_combobox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_adjust_columns_to_content_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_adjust_columns_to_content_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_disable_combos_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_disable_combos_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_extend_last_column_label.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_extend_last_column_checkbox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_remember_column_sorting_label.static_cast_mut() as *mut Widget, 3, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_remember_column_sorting_checkbox.static_cast_mut() as *mut Widget, 3, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_remember_column_visual_order_label.static_cast_mut() as *mut Widget, 4, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_remember_column_visual_order_checkbox.static_cast_mut() as *mut Widget, 4, 1, 1, 1)); }

        unsafe { ui_table_view_grid.add_widget((ui_table_remember_table_state_permanently_label.static_cast_mut() as *mut Widget, 5, 0, 1, 1)); }
        unsafe { ui_table_view_grid.add_widget((ui_table_remember_table_state_permanently_checkbox.static_cast_mut() as *mut Widget, 5, 1, 1, 1)); }

        unsafe { ui_table_view_frame.set_layout(ui_table_view_grid.into_raw() as *mut Layout); }
        unsafe { ui_grid.add_widget((ui_table_view_frame.into_raw() as *mut Widget, 99, 0, 1, 2)); }

        unsafe { ui_frame.set_layout(ui_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((ui_frame.into_raw() as *mut Widget, 1, 0, 2, 1)); }

        //-----------------------------------------------//
        // `Extra` Frame.
        //-----------------------------------------------//
        let mut extra_frame = GroupBox::new(&qtr("settings_extra_title"));
        let mut extra_grid = create_grid_layout_safe();
        extra_grid.set_contents_margins((4, 0, 4, 0));
        extra_grid.set_spacing(4);
        extra_grid.set_row_stretch(99, 10);

        // Create the "Default Game" Label and ComboBox.
        let mut extra_global_default_game_label = Label::new(&qtr("settings_default_game"));
        let mut extra_global_default_game_combobox = ComboBox::new();
        let extra_global_default_game_model = StandardItemModel::new(());
        unsafe { extra_global_default_game_combobox.set_model(extra_global_default_game_model.into_raw() as *mut AbstractItemModel); }
        for (_, game) in SUPPORTED_GAMES.iter() { extra_global_default_game_combobox.add_item(&QString::from_std_str(&game.display_name)); }

        // Create the aditional Labels/CheckBoxes.
        let mut extra_network_check_updates_on_start_label = Label::new(&qtr("settings_check_updates_on_start"));
        let mut extra_network_check_schema_updates_on_start_label = Label::new(&qtr("settings_check_schema_updates_on_start"));
        let mut extra_packfile_allow_editing_of_ca_packfiles_label = Label::new(&qtr("settings_allow_editing_of_ca_packfiles"));
        let mut extra_packfile_optimize_not_renamed_packedfiles_label = Label::new(&qtr("settings_optimize_not_renamed_packedfiles"));
        let mut extra_packfile_use_dependency_checker_label = Label::new(&qtr("settings_use_dependency_checker"));
        let mut extra_packfile_use_lazy_loading_label = Label::new(&qtr("settings_use_lazy_loading"));

        let mut extra_network_check_updates_on_start_checkbox = CheckBox::new(());
        let mut extra_network_check_schema_updates_on_start_checkbox = CheckBox::new(());
        let mut extra_packfile_allow_editing_of_ca_packfiles_checkbox = CheckBox::new(());
        let mut extra_packfile_optimize_not_renamed_packedfiles_checkbox = CheckBox::new(());
        let mut extra_packfile_use_dependency_checker_checkbox = CheckBox::new(());
        let mut extra_packfile_use_lazy_loading_checkbox = CheckBox::new(());

        unsafe { extra_grid.add_widget((extra_global_default_game_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_global_default_game_combobox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_network_check_updates_on_start_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_network_check_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_network_check_schema_updates_on_start_label.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_network_check_schema_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_packfile_allow_editing_of_ca_packfiles_label.static_cast_mut() as *mut Widget, 3, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_packfile_allow_editing_of_ca_packfiles_checkbox.static_cast_mut() as *mut Widget, 3, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_packfile_optimize_not_renamed_packedfiles_label.static_cast_mut() as *mut Widget, 4, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_packfile_optimize_not_renamed_packedfiles_checkbox.static_cast_mut() as *mut Widget, 4, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_packfile_use_dependency_checker_label.static_cast_mut() as *mut Widget, 5, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_packfile_use_dependency_checker_checkbox.static_cast_mut() as *mut Widget, 5, 1, 1, 1)); }

        unsafe { extra_grid.add_widget((extra_packfile_use_lazy_loading_label.static_cast_mut() as *mut Widget, 6, 0, 1, 1)); }
        unsafe { extra_grid.add_widget((extra_packfile_use_lazy_loading_checkbox.static_cast_mut() as *mut Widget, 6, 1, 1, 1)); }

        unsafe { extra_frame.set_layout(extra_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((extra_frame.into_raw() as *mut Widget, 1, 1, 1, 1)); }

        //-----------------------------------------------//
        // `Debug` Frame.
        //-----------------------------------------------//
        let mut debug_frame = GroupBox::new(&qtr("settings_debug_title"));
        let mut debug_grid = create_grid_layout_safe();
        debug_grid.set_contents_margins((4, 0, 4, 0));
        debug_grid.set_spacing(4);
        debug_grid.set_row_stretch(99, 10);

        let mut debug_check_for_missing_table_definitions_label = Label::new(&qtr("settings_debug_missing_table"));
        let mut debug_enable_debug_menu_label = Label::new(&qtr("settings_debug_enable_debug_menu"));

        let mut debug_check_for_missing_table_definitions_checkbox = CheckBox::new(());
        let mut debug_enable_debug_menu_checkbox = CheckBox::new(());

        unsafe { debug_grid.add_widget((debug_check_for_missing_table_definitions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { debug_grid.add_widget((debug_check_for_missing_table_definitions_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { debug_grid.add_widget((debug_enable_debug_menu_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { debug_grid.add_widget((debug_enable_debug_menu_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { debug_frame.set_layout(debug_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((debug_frame.into_raw() as *mut Widget, 2, 1, 1, 1)); }

        //-----------------------------------------------//
        // `ButtonBox` Button Box.
        //-----------------------------------------------//
        let mut button_box = DialogButtonBox::new(());
        let mut button_box_shortcuts_button = PushButton::new(&qtr("shortcut_title"));
        let mut button_box_text_editor_settings_button = PushButton::new(&qtr("settings_text_title"));

        let button_box_restore_default_button = button_box.add_button(dialog_button_box::StandardButton::RestoreDefaults);
        unsafe { button_box.add_button_unsafe(button_box_shortcuts_button.static_cast_mut() as *mut AbstractButton, ButtonRole::ResetRole); }
        unsafe { button_box.add_button_unsafe(button_box_text_editor_settings_button.static_cast_mut() as *mut AbstractButton, ButtonRole::ResetRole); }
        let button_box_cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        let button_box_accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        unsafe { main_grid.add_widget((button_box.into_raw() as *mut Widget, 3, 0, 1, 2)); }
        unsafe { dialog.set_layout(main_grid.into_raw() as *mut Layout); }

        // Now, we build the `SettingsUI` struct and return it.
        Self {

            //-------------------------------------------------------------------------------//
            // `Dialog` window.
            //-------------------------------------------------------------------------------//
            dialog: dialog.into_raw(),

            //-------------------------------------------------------------------------------//
            // `Path` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            paths_mymod_label: paths_mymod_label.into_raw(),
            paths_mymod_line_edit: paths_mymod_line_edit.into_raw(),
            paths_mymod_button: paths_mymod_button.into_raw(),
            paths_games_labels,
            paths_games_line_edits,
            paths_games_buttons,

            //-------------------------------------------------------------------------------//
            // `UI` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            ui_language_label: ui_language_label.into_raw(),
            ui_global_use_dark_theme_label: ui_global_use_dark_theme_label.into_raw(),
            ui_table_adjust_columns_to_content_label: ui_table_adjust_columns_to_content_label.into_raw(),
            ui_table_disable_combos_label: ui_table_disable_combos_label.into_raw(),
            ui_table_extend_last_column_label: ui_table_extend_last_column_label.into_raw(),
            ui_table_remember_column_sorting_label: ui_table_remember_column_sorting_label.into_raw(),
            ui_table_remember_column_visual_order_label: ui_table_remember_column_visual_order_label.into_raw(),
            ui_table_remember_table_state_permanently_label: ui_table_remember_table_state_permanently_label.into_raw(),
            ui_window_start_maximized_label: ui_window_start_maximized_label.into_raw(),

            ui_language_combobox: ui_language_combobox.into_raw(),
            ui_global_use_dark_theme_checkbox: ui_global_use_dark_theme_checkbox.into_raw(),
            ui_table_adjust_columns_to_content_checkbox: ui_table_adjust_columns_to_content_checkbox.into_raw(),
            ui_table_disable_combos_checkbox: ui_table_disable_combos_checkbox.into_raw(),
            ui_table_extend_last_column_checkbox: ui_table_extend_last_column_checkbox.into_raw(),
            ui_table_remember_column_sorting_checkbox: ui_table_remember_column_sorting_checkbox.into_raw(),
            ui_table_remember_column_visual_order_checkbox: ui_table_remember_column_visual_order_checkbox.into_raw(),
            ui_table_remember_table_state_permanently_checkbox: ui_table_remember_table_state_permanently_checkbox.into_raw(),
            ui_window_start_maximized_checkbox: ui_window_start_maximized_checkbox.into_raw(),

            //-------------------------------------------------------------------------------//
            // `Extra` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            extra_global_default_game_label: extra_global_default_game_label.into_raw(),
            extra_network_check_updates_on_start_label: extra_network_check_updates_on_start_label.into_raw(),
            extra_network_check_schema_updates_on_start_label: extra_network_check_schema_updates_on_start_label.into_raw(),
            extra_packfile_allow_editing_of_ca_packfiles_label: extra_packfile_allow_editing_of_ca_packfiles_label.into_raw(),
            extra_packfile_optimize_not_renamed_packedfiles_label: extra_packfile_optimize_not_renamed_packedfiles_label.into_raw(),
            extra_packfile_use_dependency_checker_label: extra_packfile_use_dependency_checker_label.into_raw(),
            extra_packfile_use_lazy_loading_label: extra_packfile_use_lazy_loading_label.into_raw(),

            extra_global_default_game_combobox: extra_global_default_game_combobox.into_raw(),
            extra_network_check_updates_on_start_checkbox: extra_network_check_updates_on_start_checkbox.into_raw(),
            extra_network_check_schema_updates_on_start_checkbox: extra_network_check_schema_updates_on_start_checkbox.into_raw(),
            extra_packfile_allow_editing_of_ca_packfiles_checkbox: extra_packfile_allow_editing_of_ca_packfiles_checkbox.into_raw(),
            extra_packfile_optimize_not_renamed_packedfiles_checkbox: extra_packfile_optimize_not_renamed_packedfiles_checkbox.into_raw(),
            extra_packfile_use_dependency_checker_checkbox: extra_packfile_use_dependency_checker_checkbox.into_raw(),
            extra_packfile_use_lazy_loading_checkbox: extra_packfile_use_lazy_loading_checkbox.into_raw(),

            //-------------------------------------------------------------------------------//
            // `Debug` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            debug_check_for_missing_table_definitions_label: debug_check_for_missing_table_definitions_label.into_raw(),
            debug_check_for_missing_table_definitions_checkbox: debug_check_for_missing_table_definitions_checkbox.into_raw(),
            debug_enable_debug_menu_label: debug_enable_debug_menu_label.into_raw(),
            debug_enable_debug_menu_checkbox: debug_enable_debug_menu_checkbox.into_raw(),
            //-------------------------------------------------------------------------------//
            // `ButtonBox` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            button_box_restore_default_button,
            button_box_text_editor_settings_button: button_box_text_editor_settings_button.into_raw(),
            button_box_shortcuts_button: button_box_shortcuts_button.into_raw(),
            button_box_cancel_button,
            button_box_accept_button,
        }
    }

    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub fn load(&mut self, settings: &Settings) {

        // Load the MyMod Path, if exists.
        unsafe { self.paths_mymod_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(settings.paths["mymods_base_path"].clone().unwrap_or_else(PathBuf::new).to_string_lossy())); }

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter_mut() {
            unsafe { path.as_mut().unwrap().set_text(&QString::from_std_str(&settings.paths[key].clone().unwrap_or_else(PathBuf::new).to_string_lossy())); }
        }

        // Get the default game.
        for (index, (folder_name,_)) in SUPPORTED_GAMES.iter().enumerate() {
            if *folder_name == settings.settings_string["default_game"] {
                unsafe { self.extra_global_default_game_combobox.as_mut().unwrap().set_current_index(index as i32); }
                break;
            }
        }

        // Load the UI Stuff.
        unsafe { self.ui_global_use_dark_theme_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["use_dark_theme"]); }
        unsafe { self.ui_table_adjust_columns_to_content_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["adjust_columns_to_content"]); }
        unsafe { self.ui_table_disable_combos_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["disable_combos_on_tables"]); }
        unsafe { self.ui_table_extend_last_column_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["extend_last_column_on_tables"]); }
        unsafe { self.ui_table_remember_column_sorting_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["remember_column_sorting"]); }
        unsafe { self.ui_table_remember_column_visual_order_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["remember_column_visual_order"]); }
        unsafe { self.ui_table_remember_table_state_permanently_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["remember_table_state_permanently"]); }
        unsafe { self.ui_window_start_maximized_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["start_maximized"]); }

        // Load the Extra Stuff.
        unsafe { self.extra_network_check_updates_on_start_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["check_updates_on_start"]); }
        unsafe { self.extra_network_check_schema_updates_on_start_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["check_schema_updates_on_start"]); }
        unsafe { self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["allow_editing_of_ca_packfiles"]); }
        unsafe { self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["optimize_not_renamed_packedfiles"]); }
        unsafe { self.extra_packfile_use_dependency_checker_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["use_dependency_checker"]); }
        unsafe { self.extra_packfile_use_lazy_loading_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["use_lazy_loading"]); }

        // Load the Debug Stuff.
        unsafe { self.debug_check_for_missing_table_definitions_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["check_for_missing_table_definitions"]); }
        unsafe { self.debug_enable_debug_menu_checkbox.as_mut().unwrap().set_checked(settings.settings_bool["enable_debug_menu"]); }
    }

    /// This function saves the data from our `SettingsUI` into a `Settings` and return it.
    pub fn save(&self) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new();

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let mymod_new_path = unsafe { PathBuf::from(self.paths_mymod_line_edit.as_mut().unwrap().text().to_std_string()) };
        settings.paths.insert("mymods_base_path".to_owned(), if mymod_new_path.is_dir() { Some(mymod_new_path) } else { None });

        // For each entry, we check if it's a valid directory and save it into Settings.
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let new_path = unsafe { PathBuf::from(line_edit.as_mut().unwrap().text().to_std_string()) };
            settings.paths.insert(key.to_owned(), if new_path.is_dir() { Some(new_path) } else { None });
        }

        // We get his game's folder, depending on the selected game.
        let mut game = unsafe { self.extra_global_default_game_combobox.as_mut().unwrap().current_text().to_std_string() };
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        settings.settings_string.insert("default_game".to_owned(), game);

        // Get the UI Settings.
        unsafe { settings.settings_bool.insert("use_dark_theme".to_owned(), self.ui_global_use_dark_theme_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("adjust_columns_to_content".to_owned(), self.ui_table_adjust_columns_to_content_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("disable_combos_on_tables".to_owned(), self.ui_table_disable_combos_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("extend_last_column_on_tables".to_owned(), self.ui_table_extend_last_column_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("remember_column_sorting".to_owned(), self.ui_table_remember_column_sorting_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("remember_column_visual_order".to_owned(), self.ui_table_remember_column_visual_order_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("remember_table_state_permanently".to_owned(), self.ui_table_remember_table_state_permanently_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("start_maximized".to_owned(), self.ui_window_start_maximized_checkbox.as_mut().unwrap().is_checked()); }

        // Get the Extra Settings.
        unsafe { settings.settings_bool.insert("check_updates_on_start".to_owned(), self.extra_network_check_updates_on_start_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("check_schema_updates_on_start".to_owned(), self.extra_network_check_schema_updates_on_start_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("use_dependency_checker".to_owned(), self.extra_packfile_use_dependency_checker_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("use_lazy_loading".to_owned(), self.extra_packfile_use_lazy_loading_checkbox.as_mut().unwrap().is_checked()); }

        // Get the Debug Settings.
        unsafe { settings.settings_bool.insert("check_for_missing_table_definitions".to_owned(), self.debug_check_for_missing_table_definitions_checkbox.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("enable_debug_menu".to_owned(), self.debug_enable_debug_menu_checkbox.as_mut().unwrap().is_checked()); }

        // Return the new Settings.
        settings
    }

    /// This function updates the path you have for the provided game (or mymod, if you pass it `None`)
    /// with the one you select in a `FileDialog`.
    fn update_entry_path(&self, game: Option<&str>) {

        // Create the `FileDialog` and configure it.
        let mut file_dialog = unsafe { FileDialog::new_unsafe((
            self.dialog as *mut Widget,
            &qtr("settings_select_folder"),
        ))};
        file_dialog.set_file_mode(FileMode::Directory);
        file_dialog.set_option(ShowDirsOnly);

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let line_edit = match game {
            Some(game) => *self.paths_games_line_edits.get(game).unwrap(),
            None => self.paths_mymod_line_edit,
        };

        // Get the old Path, if exists.
        let old_path = unsafe { line_edit.as_mut().unwrap().text().to_std_string() };

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            unsafe { file_dialog.set_directory(&line_edit.as_mut().unwrap().text()); }
        }

        // Run it and expect a response (1 => Accept, 0 => Cancel).
        if file_dialog.exec() == 1 {

            // Get the path of the selected file.
            let selected_files = file_dialog.selected_files();
            let path = selected_files.at(0);

            // Add the Path to the LineEdit.
            unsafe { line_edit.as_mut().unwrap().set_text(&path); }
        }
    }
}
