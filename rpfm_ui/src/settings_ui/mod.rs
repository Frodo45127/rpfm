//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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
use crate::QString;
use crate::SETTINGS;
use crate::settings_ui::slots::SettingsUISlots;
use crate::utils::create_grid_layout_safe;

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
    pub ui_global_use_dark_theme_label: *mut Label,
    pub ui_table_adjust_columns_to_content_label: *mut Label,
    pub ui_table_disable_combos_label: *mut Label,
    pub ui_table_extend_last_column_label: *mut Label,
    pub ui_table_remember_column_sorting_label: *mut Label,
    pub ui_table_remember_column_visual_order_label: *mut Label,
    pub ui_table_remember_table_state_permanently_label: *mut Label,
    pub ui_window_start_maximized_label: *mut Label,

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

    //-------------------------------------------------------------------------------//
    // `ButtonBox` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub button_box_restore_default_button: *mut PushButton,
    pub button_box_shortcuts_button: *mut PushButton,
    pub button_box_cancel_button: *mut PushButton,
    pub button_box_accept_button: *mut PushButton,
}
/*
/// `MyModNewWindow`: This struct holds all the relevant stuff for "My Mod"'s New Mod Window.
#[derive(Clone, Debug)]
pub struct NewMyModDialog {
    pub mymod_game_combobox: *mut ComboBox,
    pub mymod_name_line_edit: *mut LineEdit,
    pub cancel_button: *mut PushButton,
    pub accept_button: *mut PushButton,
}
*/

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
    fn new_with_parent(parent: *mut Widget) -> SettingsUI {

        // Initialize and configure the settings window.
        let mut dialog = unsafe { Dialog::new_unsafe(parent) };
        dialog.set_window_title(&QString::from_std_str("Preferences"));
        dialog.set_modal(true);
        dialog.resize((750, 0));

        let mut main_grid = create_grid_layout_safe();
        main_grid.set_contents_margins((4, 0, 4, 4));
        main_grid.set_spacing(4);

        //-----------------------------------------------//
        // `Paths` Frame.
        //-----------------------------------------------//
        let mut paths_frame = GroupBox::new(&QString::from_std_str("Paths"));
        let mut paths_grid = create_grid_layout_safe();
        paths_grid.set_contents_margins((4, 0, 4, 0));

        // Create the MyMod's path stuff,
        let paths_mymod_label = Label::new(&QString::from_std_str("MyMod's Folder:"));
        let mut paths_mymod_line_edit = LineEdit::new(());
        let paths_mymod_button = PushButton::new(&QString::from_std_str("..."));
        paths_mymod_line_edit.set_placeholder_text(&QString::from_std_str("This is the folder where you want to store all \"MyMod\" related files."));

        unsafe { paths_grid.add_widget((paths_mymod_label.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { paths_grid.add_widget((paths_mymod_line_edit.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { paths_grid.add_widget((paths_mymod_button.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_games_labels = BTreeMap::new();
        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();
        for (index, (folder_name, game_supported)) in SUPPORTED_GAMES.iter().enumerate() {
            let game_label = Label::new(&QString::from_std_str(&format!("TW: {} Folder", game_supported.display_name)));
            let mut game_line_edit = LineEdit::new(());
            let game_button = PushButton::new(&QString::from_std_str("..."));
            game_line_edit.set_placeholder_text(&QString::from_std_str(&*format!("This is the folder where you have {} installed.", game_supported.display_name)));

            unsafe { paths_grid.add_widget((game_label.as_mut_ptr() as *mut Widget, (index + 1) as i32, 0, 1, 1)); }
            unsafe { paths_grid.add_widget((game_line_edit.as_mut_ptr() as *mut Widget, (index + 1) as i32, 1, 1, 1)); }
            unsafe { paths_grid.add_widget((game_button.as_mut_ptr() as *mut Widget, (index + 1) as i32, 2, 1, 1)); }

            // Add the LineEdit and Button to the list.
            paths_games_labels.insert(folder_name.to_string(), game_label.into_raw());
            paths_games_line_edits.insert(folder_name.to_string(), game_line_edit.into_raw());
            paths_games_buttons.insert(folder_name.to_string(), game_button.into_raw());
        }

        unsafe { paths_frame.set_layout(paths_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((paths_frame.into_raw() as *mut Widget, 0, 0, 1, 2)); }

        //-----------------------------------------------//
        // `UI` Frame.
        //-----------------------------------------------//
        let mut ui_frame = GroupBox::new(&QString::from_std_str("UI Settings"));
        let mut ui_grid = create_grid_layout_safe();
        ui_grid.set_contents_margins((4, 0, 4, 0));
        ui_grid.set_spacing(4);
        ui_grid.set_row_stretch(99, 10);

        // Create the "UI - TableView" frame and grid.
        let mut ui_table_view_frame = GroupBox::new(&QString::from_std_str("Table Settings"));
        let mut ui_table_view_grid = create_grid_layout_safe();
        ui_table_view_grid.set_contents_margins((4, 0, 4, 0));
        ui_table_view_grid.set_spacing(4);
        ui_table_view_grid.set_row_stretch(99, 10);

        let mut ui_global_use_dark_theme_label = Label::new(&QString::from_std_str("Use Dark Theme (Requires restart):"));
        let mut ui_table_adjust_columns_to_content_label = Label::new(&QString::from_std_str("Adjust Columns to Content:"));
        let mut ui_table_disable_combos_label = Label::new(&QString::from_std_str("Disable ComboBoxes on Tables:"));
        let mut ui_table_extend_last_column_label = Label::new(&QString::from_std_str("Extend Last Column on Tables:"));
        let mut ui_table_remember_column_sorting_label = Label::new(&QString::from_std_str("Remember Column's Sorting State:"));
        let mut ui_table_remember_column_visual_order_label = Label::new(&QString::from_std_str("Remember Column's Visual Order:"));
        let mut ui_table_remember_table_state_permanently_label = Label::new(&QString::from_std_str("Remember Table State Across PackFiles:"));
        let mut ui_window_start_maximized_label = Label::new(&QString::from_std_str("Start Maximized:"));
        
        let mut ui_global_use_dark_theme_checkbox = CheckBox::new(());
        let mut ui_table_adjust_columns_to_content_checkbox = CheckBox::new(());
        let mut ui_table_disable_combos_checkbox = CheckBox::new(());
        let mut ui_table_extend_last_column_checkbox = CheckBox::new(());
        let mut ui_table_remember_column_sorting_checkbox = CheckBox::new(());
        let mut ui_table_remember_column_visual_order_checkbox = CheckBox::new(());
        let mut ui_table_remember_table_state_permanently_checkbox = CheckBox::new(());
        let mut ui_window_start_maximized_checkbox = CheckBox::new(());

        // Add all Label/Checkboxes to the grid.
        if cfg!(target_os = "windows") {
            unsafe { ui_grid.add_widget((ui_global_use_dark_theme_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
            unsafe { ui_grid.add_widget((ui_global_use_dark_theme_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
        }

        unsafe { ui_grid.add_widget((ui_window_start_maximized_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_grid.add_widget((ui_window_start_maximized_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }
       
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
        let mut extra_frame = GroupBox::new(&QString::from_std_str("Extra Settings"));
        let mut extra_grid = create_grid_layout_safe();
        extra_grid.set_contents_margins((4, 0, 4, 0));
        extra_grid.set_spacing(4);
        extra_grid.set_row_stretch(99, 10);

        // Create the "Default Game" Label and ComboBox.
        let mut extra_global_default_game_label = Label::new(&QString::from_std_str("Default Game:"));
        let mut extra_global_default_game_combobox = ComboBox::new();
        let extra_global_default_game_model = StandardItemModel::new(());
        unsafe { extra_global_default_game_combobox.set_model(extra_global_default_game_model.into_raw() as *mut AbstractItemModel); }
        for (_, game) in SUPPORTED_GAMES.iter() { extra_global_default_game_combobox.add_item(&QString::from_std_str(&game.display_name)); }

        // Create the aditional Labels/CheckBoxes.
        let mut extra_network_check_updates_on_start_label = Label::new(&QString::from_std_str("Check Updates on Start:"));
        let mut extra_network_check_schema_updates_on_start_label = Label::new(&QString::from_std_str("Check Schema Updates on Start:"));
        let mut extra_packfile_allow_editing_of_ca_packfiles_label = Label::new(&QString::from_std_str("Allow Editing of CA PackFiles:"));
        let mut extra_packfile_optimize_not_renamed_packedfiles_label = Label::new(&QString::from_std_str("Optimize Non-Renamed PackedFiles:"));
        let mut extra_packfile_use_dependency_checker_label = Label::new(&QString::from_std_str("Enable Dependency Checker for DB Tables:"));
        let mut extra_packfile_use_lazy_loading_label = Label::new(&QString::from_std_str("Use Lazy-Loading for PackFiles:"));
        
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
        let mut debug_frame = GroupBox::new(&QString::from_std_str("Debug Settings"));
        let mut debug_grid = create_grid_layout_safe();
        debug_grid.set_contents_margins((4, 0, 4, 0));
        debug_grid.set_spacing(4);
        debug_grid.set_row_stretch(99, 10);

        let mut debug_check_for_missing_table_definitions_label = Label::new(&QString::from_std_str("Check for Missing Table Definitions"));
        let mut debug_check_for_missing_table_definitions_checkbox = CheckBox::new(());

        unsafe { debug_grid.add_widget((debug_check_for_missing_table_definitions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { debug_grid.add_widget((debug_check_for_missing_table_definitions_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { debug_frame.set_layout(debug_grid.into_raw() as *mut Layout); }
        unsafe { main_grid.add_widget((debug_frame.into_raw() as *mut Widget, 2, 1, 1, 1)); }

        //-----------------------------------------------//
        // `ButtonBox` Button Box.
        //-----------------------------------------------//
        let mut button_box = DialogButtonBox::new(());
        let mut button_box_shortcuts_button = PushButton::new(&QString::from_std_str("Shortcuts"));

        let button_box_restore_default_button = button_box.add_button(dialog_button_box::StandardButton::RestoreDefaults);
        unsafe { button_box.add_button_unsafe(button_box_shortcuts_button.static_cast_mut() as *mut AbstractButton, ButtonRole::ResetRole); }
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
            paths_games_labels: paths_games_labels,
            paths_games_line_edits: paths_games_line_edits,
            paths_games_buttons: paths_games_buttons,

            //-------------------------------------------------------------------------------//
            // `UI` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            ui_global_use_dark_theme_label: ui_global_use_dark_theme_label.into_raw(),
            ui_table_adjust_columns_to_content_label: ui_table_adjust_columns_to_content_label.into_raw(),
            ui_table_disable_combos_label: ui_table_disable_combos_label.into_raw(),
            ui_table_extend_last_column_label: ui_table_extend_last_column_label.into_raw(),
            ui_table_remember_column_sorting_label: ui_table_remember_column_sorting_label.into_raw(),
            ui_table_remember_column_visual_order_label: ui_table_remember_column_visual_order_label.into_raw(),
            ui_table_remember_table_state_permanently_label: ui_table_remember_table_state_permanently_label.into_raw(),
            ui_window_start_maximized_label: ui_window_start_maximized_label.into_raw(),

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

            //-------------------------------------------------------------------------------//
            // `ButtonBox` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            button_box_restore_default_button: button_box_restore_default_button,
            button_box_shortcuts_button: button_box_shortcuts_button.into_raw(),
            button_box_cancel_button: button_box_cancel_button,
            button_box_accept_button: button_box_accept_button,
        }
    }    

    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub fn load(&mut self, settings: &Settings) {

        // Load the MyMod Path, if exists.
        unsafe { self.paths_mymod_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(settings.paths["mymods_base_path"].clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter_mut() {
            unsafe { path.as_mut().unwrap().set_text(&QString::from_std_str(&settings.paths[key].clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }
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
    }

    /// This function saves the data from our `SettingsUI` into a `Settings` and return it.
    pub fn save(&self) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new();

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let mymod_new_path = unsafe { PathBuf::from(self.paths_mymod_line_edit.as_mut().unwrap().text().to_std_string()) };
        settings.paths.insert("mymods_base_path".to_owned(), match mymod_new_path.is_dir() {
            true => Some(mymod_new_path),
            false => None,
        });

        // For each entry, we check if it's a valid directory and save it into Settings.
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let new_path = unsafe { PathBuf::from(line_edit.as_mut().unwrap().text().to_std_string()) };
            settings.paths.insert(key.to_owned(), match new_path.is_dir() {
                true => Some(new_path),
                false => None,
            });
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

        // Return the new Settings.
        settings
    }

    /// This function updates the path you have for the provided game (or mymod, if you pass it `None`) 
    /// with the one you select in a `FileDialog`.
    fn update_entry_path(&self, game: Option<&str>) {

        // Create the `FileDialog` and configure it.
        let mut file_dialog = unsafe { FileDialog::new_unsafe((
            self.dialog as *mut Widget,
            &QString::from_std_str("Select Folder"),
        ))};
        file_dialog.set_file_mode(FileMode::Directory);
        file_dialog.set_option(ShowDirsOnly);

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let line_edit = match game {
            Some(game) => self.paths_games_line_edits.get(game).unwrap().clone(),
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

/*
/// Implementation of `SettingsUI`.
impl SettingsUI {

    /// This function creates a new `Settings` dialog, returning the new settings, or none if the user closes it.
    pub fn new(app_ui: &AppUI) -> Option<Settings> {

        //-------------------------------------------------------------------------------------------//
        // Creating the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the Preferences Dialog and configure it.
        let dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw() };
        unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Preferences")); }
        unsafe { dialog.as_mut().unwrap().set_modal(true); }
        unsafe { dialog.as_mut().unwrap().resize((750, 0)); }

        // Create the main Grid.
        let main_grid = create_grid_layout_unsafe(dialog as *mut Widget);
        unsafe { main_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 4)); }
        unsafe { main_grid.as_mut().unwrap().set_spacing(4); }

        // Create the Paths Frame.
        let paths_frame = GroupBox::new(&QString::from_std_str("Paths")).into_raw();
        let paths_grid = create_grid_layout_unsafe(paths_frame as *mut Widget);
        unsafe { paths_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 0)); }

        // Create the MyMod's path stuff...
        let mymod_label = Label::new(&QString::from_std_str("MyMod's Folder:")).into_raw();
        let mymod_line_edit = LineEdit::new(()).into_raw();
        let mymod_button = PushButton::new(&QString::from_std_str("...")).into_raw();

        // Configure the MyMod LineEdit.
        unsafe { mymod_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("This is the folder where you want to store all \"MyMod\" related files.")); }

        // Add them to the grid.
        unsafe { paths_grid.as_mut().unwrap().add_widget((mymod_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { paths_grid.as_mut().unwrap().add_widget((mymod_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { paths_grid.as_mut().unwrap().add_widget((mymod_button as *mut Widget, 0, 2, 1, 1)); }

        // For each game supported...
        let mut game_paths = BTreeMap::new();
        let mut game_buttons = BTreeMap::new();
        for (index, (folder_name, game_supported)) in SUPPORTED_GAMES.iter().enumerate() {

            // Create his fields.
            let game_label = Label::new(&QString::from_std_str(&format!("TW: {} Folder", game_supported.display_name))).into_raw();
            let game_line_edit = LineEdit::new(()).into_raw();
            let game_button = PushButton::new(&QString::from_std_str("...")).into_raw();

            // Configure the MyMod LineEdit.
            unsafe { game_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str(&*format!("This is the folder where you have {} installed.", game_supported.display_name))); }

            // And add them to the grid.
            unsafe { paths_grid.as_mut().unwrap().add_widget((game_label as *mut Widget, (index + 1) as i32, 0, 1, 1)); }
            unsafe { paths_grid.as_mut().unwrap().add_widget((game_line_edit as *mut Widget, (index + 1) as i32, 1, 1, 1)); }
            unsafe { paths_grid.as_mut().unwrap().add_widget((game_button as *mut Widget, (index + 1) as i32, 2, 1, 1)); }

            // Add the LineEdit and Button to the list.
            game_paths.insert(folder_name.to_string(), game_line_edit);
            game_buttons.insert(folder_name.to_string(), game_button);
        }

        // Create the "UI Settings" frame and Grid.
        let ui_settings_frame = GroupBox::new(&QString::from_std_str("UI Settings")).into_raw();
        let ui_settings_grid = create_grid_layout_unsafe(ui_settings_frame as *mut Widget);
        unsafe { ui_settings_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 0)); }
        unsafe { ui_settings_grid.as_mut().unwrap().set_spacing(4); }
        unsafe { ui_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

        // Create the "UI TableView Settings" frame and grid.
        let ui_table_view_settings_frame = GroupBox::new(&QString::from_std_str("Table Settings")).into_raw();
        let ui_table_view_settings_grid = create_grid_layout_unsafe(ui_table_view_settings_frame as *mut Widget);
        unsafe { ui_table_view_settings_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 0)); }
        unsafe { ui_table_view_settings_grid.as_mut().unwrap().set_spacing(4); }
        unsafe { ui_table_view_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

        // Create the UI options.
        let mut adjust_columns_to_content_label = Label::new(&QString::from_std_str("Adjust Columns to Content:"));
        let mut extend_last_column_on_tables_label = Label::new(&QString::from_std_str("Extend Last Column on Tables:"));
        let mut disable_combos_on_tables_label = Label::new(&QString::from_std_str("Disable ComboBoxes on Tables:"));
        let mut start_maximized_label = Label::new(&QString::from_std_str("Start Maximized:"));
        let mut remember_table_state_permanently_label = Label::new(&QString::from_std_str("Remember Table State Across PackFiles:"));
        let mut use_dark_theme_label = Label::new(&QString::from_std_str("Use Dark Theme (Requires restart):"));

        let mut remember_column_sorting_label = Label::new(&QString::from_std_str("Remember Column's Sorting State:"));
        let mut remember_column_visual_order_label = Label::new(&QString::from_std_str("Remember Column's Visual Order:"));

        let mut adjust_columns_to_content_checkbox = CheckBox::new(());
        let mut extend_last_column_on_tables_checkbox = CheckBox::new(());
        let mut disable_combos_on_tables_checkbox = CheckBox::new(());
        let mut start_maximized_checkbox = CheckBox::new(());
        let mut remember_table_state_permanently_checkbox = CheckBox::new(());
        let mut use_dark_theme_checkbox = CheckBox::new(());

        let mut remember_column_sorting_checkbox = CheckBox::new(());
        let mut remember_column_visual_order_checkbox = CheckBox::new(());

        // Tips for the UI settings.
        let adjust_columns_to_content_tip = QString::from_std_str("If you enable this, when you open a DB Table or Loc File, all columns will be automatically resized depending on their content's size.\nOtherwise, columns will have a predefined size. Either way, you'll be able to resize them manually after the initial resize.\nNOTE: This can make very big tables take more time to load.");
        let extend_last_column_on_tables_tip = QString::from_std_str("If you enable this, the last column on DB Tables and Loc PackedFiles will extend itself to fill the empty space at his right, if there is any.");
        let disable_combos_on_tables_tip = QString::from_std_str("If you disable this, no more combos will be shown in referenced columns in tables. This means no combos nor autocompletion on DB Tables.\nNow shut up Baldy.");
        let start_maximized_tip = QString::from_std_str("If you enable this, RPFM will start maximized.");
        let remember_table_state_permanently_tip = QString::from_std_str("If you enable this, RPFM will remember the state of a DB Table or Loc PackedFile (filter data, columns moved, what column was sorting the Table,...) even when you close RPFM and open it again. If you don't want this behavior, leave this disabled.");
        let use_dark_theme_tip = QString::from_std_str("<i>Ash nazg durbatulûk, ash nazg gimbatul, ash nazg thrakatulûk, agh burzum-ishi krimpatul</i>");
        
        let remember_column_sorting_tip = QString::from_std_str("Enable this to make RPFM remember for what column was a DB Table/LOC sorted when closing it and opening it again.");
        let remember_column_visual_order_tip = QString::from_std_str("Enable this to make RPFM remember the visual order of the columns of a DB Table/LOC, when closing it and opening it again.");

        adjust_columns_to_content_label.set_tool_tip(&adjust_columns_to_content_tip);
        adjust_columns_to_content_checkbox.set_tool_tip(&adjust_columns_to_content_tip);
        extend_last_column_on_tables_label.set_tool_tip(&extend_last_column_on_tables_tip);
        extend_last_column_on_tables_checkbox.set_tool_tip(&extend_last_column_on_tables_tip);
        disable_combos_on_tables_label.set_tool_tip(&disable_combos_on_tables_tip);
        disable_combos_on_tables_checkbox.set_tool_tip(&disable_combos_on_tables_tip);
        start_maximized_label.set_tool_tip(&start_maximized_tip);
        start_maximized_checkbox.set_tool_tip(&start_maximized_tip);
        remember_table_state_permanently_label.set_tool_tip(&remember_table_state_permanently_tip);
        remember_table_state_permanently_checkbox.set_tool_tip(&remember_table_state_permanently_tip);
        use_dark_theme_label.set_tool_tip(&use_dark_theme_tip);
        use_dark_theme_checkbox.set_tool_tip(&use_dark_theme_tip);

        remember_column_sorting_label.set_tool_tip(&remember_column_sorting_tip);
        remember_column_sorting_checkbox.set_tool_tip(&remember_column_sorting_tip);
        remember_column_visual_order_label.set_tool_tip(&remember_column_visual_order_tip);
        remember_column_visual_order_checkbox.set_tool_tip(&remember_column_visual_order_tip);

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((adjust_columns_to_content_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((adjust_columns_to_content_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((extend_last_column_on_tables_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((extend_last_column_on_tables_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((disable_combos_on_tables_label.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((disable_combos_on_tables_checkbox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((start_maximized_label.static_cast_mut() as *mut Widget, 3, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((start_maximized_checkbox.static_cast_mut() as *mut Widget, 3, 1, 1, 1)); }

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((remember_table_state_permanently_label.static_cast_mut() as *mut Widget, 4, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((remember_table_state_permanently_checkbox.static_cast_mut() as *mut Widget, 4, 1, 1, 1)); }
       
        if cfg!(target_os = "windows") {
            unsafe { ui_settings_grid.as_mut().unwrap().add_widget((use_dark_theme_label.static_cast_mut() as *mut Widget, 5, 0, 1, 1)); }
            unsafe { ui_settings_grid.as_mut().unwrap().add_widget((use_dark_theme_checkbox.static_cast_mut() as *mut Widget, 5, 1, 1, 1)); }
        }
        
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((ui_table_view_settings_frame as *mut Widget, 99, 0, 1, 2)); }

        unsafe { ui_table_view_settings_grid.as_mut().unwrap().add_widget((remember_column_sorting_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { ui_table_view_settings_grid.as_mut().unwrap().add_widget((remember_column_sorting_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { ui_table_view_settings_grid.as_mut().unwrap().add_widget((remember_column_visual_order_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_table_view_settings_grid.as_mut().unwrap().add_widget((remember_column_visual_order_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        // Create the "Extra Settings" frame and Grid.
        let extra_settings_frame = GroupBox::new(&QString::from_std_str("Extra Settings")).into_raw();
        let extra_settings_grid = create_grid_layout_unsafe(extra_settings_frame as *mut Widget);
        unsafe { extra_settings_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 0)); }
        unsafe { extra_settings_grid.as_mut().unwrap().set_spacing(4); }
        unsafe { extra_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

        // Create the "Debug Settings" frame and grid.
        let debug_settings_frame = GroupBox::new(&QString::from_std_str("Debug Settings")).into_raw();
        let debug_settings_grid = create_grid_layout_unsafe(debug_settings_frame as *mut Widget);
        unsafe { debug_settings_grid.as_mut().unwrap().set_contents_margins((4, 0, 4, 0)); }
        unsafe { debug_settings_grid.as_mut().unwrap().set_spacing(4); }
        unsafe { debug_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

        // Create the "Default Game" Label and ComboBox.
        let default_game_label = Label::new(&QString::from_std_str("Default Game:")).into_raw();
        let mut default_game_combobox = ComboBox::new();
        let mut default_game_model = StandardItemModel::new(());
        unsafe { default_game_combobox.set_model(default_game_model.static_cast_mut()); }

        // Add the games to the ComboBox.
        for (_, game) in SUPPORTED_GAMES.iter() { default_game_combobox.add_item(&QString::from_std_str(&game.display_name)); }

        // Create the aditional CheckBoxes.
        let mut allow_editing_of_ca_packfiles_label = Label::new(&QString::from_std_str("Allow Editing of CA PackFiles:"));
        let mut check_updates_on_start_label = Label::new(&QString::from_std_str("Check Updates on Start:"));
        let mut check_schema_updates_on_start_label = Label::new(&QString::from_std_str("Check Schema Updates on Start:"));
        let mut use_dependency_checker_label = Label::new(&QString::from_std_str("Enable Dependency Checker for DB Tables:"));
        let mut use_lazy_loading_label = Label::new(&QString::from_std_str("Use Lazy-Loading for PackFiles:"));
        let mut optimize_not_renamed_packedfiles_label = Label::new(&QString::from_std_str("Optimize Non-Renamed PackedFiles:"));
        
        let mut check_for_missing_table_definitions_label = Label::new(&QString::from_std_str("Check for Missing Table Definitions"));

        let mut allow_editing_of_ca_packfiles_checkbox = CheckBox::new(());
        let mut check_updates_on_start_checkbox = CheckBox::new(());
        let mut check_schema_updates_on_start_checkbox = CheckBox::new(());
        let mut use_dependency_checker_checkbox = CheckBox::new(());
        let mut use_lazy_loading_checkbox = CheckBox::new(());
        let mut optimize_not_renamed_packedfiles_checkbox = CheckBox::new(());

        let mut check_for_missing_table_definitions_checkbox = CheckBox::new(());

        // Tips.
        let allow_editing_of_ca_packfiles_tip = QString::from_std_str("By default, only PackFiles of Type 'Mod' and 'Movie' are editables, as those are the only ones used for modding.\nIf you enable this, you'll be able to edit 'Boot', 'Release' and 'Patch' PackFiles too. Just be careful of not writing over one of the game's original PackFiles!");
        let check_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for updates at the start of the program, and inform you if there is any update available.\nWhether download it or not is up to you.");
        let check_schema_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for schema updates at the start of the program,\nand allow you to automatically download it if there is any update available.");
        let use_dependency_checker_tip = QString::from_std_str("If you enable this, when opening a DB Table RPFM will try to get his dependencies and mark all cells with a reference to another table as 'Not Found In Table' (Red), 'Referenced Table Not Found' (Blue) or 'Correct Reference' (Black). It makes opening a big table a bit slower.");
        let use_lazy_loading_tip = QString::from_std_str("If you enable this, PackFiles will load their data on-demand from the disk instead of loading the entire PackFile to Ram. This reduces Ram usage by a lot, but if something else changes/deletes the PackFile while it's open, the PackFile will likely be unrecoverable and you'll lose whatever is in it.\nIf you mainly mod in Warhammer 2's /data folder LEAVE THIS DISABLED, as a bug in the Assembly Kit causes PackFiles to become broken/be deleted when you have this enabled.");
        let optimize_not_renamed_packedfiles_tip = QString::from_std_str("If you enable this, when running the 'Optimize PackFile' feature RPFM will optimize Tables and Locs that have the same name as their vanilla counterparts.\nUsually, those files are intended to fully override their vanilla counterparts, so by default (this setting off) they are ignored by the optimizer. But it can be useful sometimes to optimize them too (AssKit including too many files), so that's why this setting exists.");
        
        let check_for_missing_table_definitions_tip = QString::from_std_str("If you enable this, RPFM will try to decode EVERY TABLE in the current PackFile when opening it or when changing the Game Selected, and it'll output all the tables without an schema to a \"missing_table_definitions.txt\" file.\nDEBUG FEATURE, VERY SLOW. DON'T ENABLE IT UNLESS YOU REALLY WANT TO USE IT.");

        // Tips for the checkboxes.
        allow_editing_of_ca_packfiles_checkbox.set_tool_tip(&allow_editing_of_ca_packfiles_tip);
        check_updates_on_start_checkbox.set_tool_tip(&check_updates_on_start_tip);
        check_schema_updates_on_start_checkbox.set_tool_tip(&check_schema_updates_on_start_tip);
        use_dependency_checker_checkbox.set_tool_tip(&use_dependency_checker_tip);
        use_lazy_loading_checkbox.set_tool_tip(&use_lazy_loading_tip);
        optimize_not_renamed_packedfiles_checkbox.set_tool_tip(&optimize_not_renamed_packedfiles_tip);

        check_for_missing_table_definitions_checkbox.set_tool_tip(&check_for_missing_table_definitions_tip);

        // Also, for their labels.
        allow_editing_of_ca_packfiles_label.set_tool_tip(&allow_editing_of_ca_packfiles_tip);
        check_updates_on_start_label.set_tool_tip(&check_updates_on_start_tip);
        check_schema_updates_on_start_label.set_tool_tip(&check_schema_updates_on_start_tip);
        use_dependency_checker_label.set_tool_tip(&use_dependency_checker_tip);
        use_lazy_loading_label.set_tool_tip(&use_lazy_loading_tip);
        optimize_not_renamed_packedfiles_label.set_tool_tip(&optimize_not_renamed_packedfiles_tip);

        check_for_missing_table_definitions_label.set_tool_tip(&check_for_missing_table_definitions_tip);

        // Add the "Default Game" stuff to the Grid.
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((default_game_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((default_game_combobox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((allow_editing_of_ca_packfiles_label.into_raw() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((allow_editing_of_ca_packfiles_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_updates_on_start_label.into_raw() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_schema_updates_on_start_label.into_raw() as *mut Widget, 3, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_schema_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 3, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_dependency_checker_label.into_raw() as *mut Widget, 4, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_dependency_checker_checkbox.static_cast_mut() as *mut Widget, 4, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_lazy_loading_label.into_raw() as *mut Widget, 5, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_lazy_loading_checkbox.static_cast_mut() as *mut Widget, 5, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((optimize_not_renamed_packedfiles_label.into_raw() as *mut Widget, 6, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((optimize_not_renamed_packedfiles_checkbox.static_cast_mut() as *mut Widget, 6, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((debug_settings_frame as *mut Widget, 99, 0, 1, 2)); }

        unsafe { debug_settings_grid.as_mut().unwrap().add_widget((check_for_missing_table_definitions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { debug_settings_grid.as_mut().unwrap().add_widget((check_for_missing_table_definitions_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        // Add the Path's grid to his Frame, and his Frame to the Main Grid.
        unsafe { main_grid.as_mut().unwrap().add_widget((paths_frame as *mut Widget, 0, 0, 1, 2)); }

        // Add the Grid to the Frame, and the Frame to the Main Grid.
        unsafe { main_grid.as_mut().unwrap().add_widget((ui_settings_frame as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((extra_settings_frame as *mut Widget, 1, 1, 1, 1)); }

        // Create the bottom ButtonBox.
        let mut button_box = DialogButtonBox::new(());
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.static_cast_mut() as *mut Widget, 2, 0, 1, 2)); }

        // Create the bottom buttons and add them to the dialog.
        let mut shortcuts_button = PushButton::new(&QString::from_std_str("Shortcuts"));
        let restore_default_button = button_box.add_button(dialog_button_box::StandardButton::RestoreDefaults);
        unsafe { button_box.add_button_unsafe(shortcuts_button.static_cast_mut() as *mut AbstractButton, ButtonRole::ResetRole); }
        let cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        let accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        //-------------------------------------------------------------------------------------------//
        // Slots for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for MyMods.
        let slot_select_mymod_path = SlotNoArgs::new(move || {
            update_entry_path(mymod_line_edit, dialog);
        });

        // What happens when we hit any of the "..." buttons for the games.
        let mut slots_select_paths = BTreeMap::new();
        for (key, path) in &game_paths {
            slots_select_paths.insert(key, SlotNoArgs::new(move || {
                update_entry_path(*path, dialog);
            }));
        }

        // What happens when we hit the "Shortcuts" button.
        let slot_shortcuts = SlotNoArgs::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move || {

                // Create the Shortcuts Dialog. If we got new shortcuts...
                if let Some(shortcuts) = ShortcutsDialog::create_shortcuts_dialog(dialog) {

                    // Send the signal to save them.
                    sender_qt.send(Commands::SetShortcuts).unwrap();
                    sender_qt_data.send(Data::Shortcuts(shortcuts)).unwrap();

                    // If there was an error.
                    if let Data::Error(error) = check_message_validity_recv2(&receiver_qt) { 

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If there was and IO error while saving the shortcuts, report it.
                            ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound | ErrorKind::IOGeneric => show_dialog(app_ui.window, false, error.kind()),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }

                    }
                }
            }
        ));

        //-------------------------------------------------------------------------------------------//
        // Actions for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for MyMods.
        unsafe { mymod_button.as_mut().unwrap().signals().released().connect(&slot_select_mymod_path); }

        // What happens when we hit the "..." button for Games.
        for (key, button) in game_buttons.iter() {
            unsafe { button.as_mut().unwrap().signals().released().connect(&slots_select_paths[key]); }
        }

        // What happens when we hit the "Shortcuts" button.
        shortcuts_button.signals().released().connect(&slot_shortcuts);

        // What happens when we hit the "Cancel" button.
        unsafe { cancel_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().accept()); }

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let mut settings_dialog = Self {
            paths_mymod_line_edit: mymod_line_edit,
            paths_games_line_edits: game_paths.clone(),
            ui_adjust_columns_to_content: adjust_columns_to_content_checkbox.into_raw(),
            ui_extend_last_column_on_tables: extend_last_column_on_tables_checkbox.into_raw(),
            ui_disable_combos_on_tables: disable_combos_on_tables_checkbox.into_raw(),
            ui_start_maximized: start_maximized_checkbox.into_raw(),
            ui_remember_table_state_permanently: remember_table_state_permanently_checkbox.into_raw(),
            ui_use_dark_theme: use_dark_theme_checkbox.into_raw(),
            ui_table_view_remember_column_sorting: remember_column_sorting_checkbox.into_raw(),
            ui_table_view_remember_column_visual_order: remember_column_visual_order_checkbox.into_raw(),
            extra_default_game_combobox: default_game_combobox.into_raw(),
            extra_allow_editing_of_ca_packfiles: allow_editing_of_ca_packfiles_checkbox.into_raw(),
            extra_check_updates_on_start: check_updates_on_start_checkbox.into_raw(),
            extra_check_schema_updates_on_start: check_schema_updates_on_start_checkbox.into_raw(),
            extra_use_dependency_checker: use_dependency_checker_checkbox.into_raw(),
            extra_use_lazy_loading_checker: use_lazy_loading_checkbox.into_raw(),
            extra_optimize_not_renamed_packedfiles_checker: optimize_not_renamed_packedfiles_checkbox.into_raw(),
            debug_check_for_missing_table_definitions: check_for_missing_table_definitions_checkbox.into_raw(),
        };

        //-------------------------------------------------------------------------------------------//
        // Loading data to the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Load the MyMod Path, if exists.
        settings_dialog.load_to_settings_dialog(&SETTINGS.lock().unwrap());

        //-------------------------------------------------------------------------------------------//
        // Actions that must exectute at the end...
        //-------------------------------------------------------------------------------------------//
        let settings_dialog = Rc::new(RefCell::new(settings_dialog));

        // What happens when we hit the "Restore Default" action.
        let slot_restore_default = SlotNoArgs::new(clone!(
            settings_dialog => move || {
                (*settings_dialog.borrow_mut()).load_to_settings_dialog(&Settings::new());
            }
        ));

        // What happens when we hit the "Restore Default" button.
        unsafe { restore_default_button.as_mut().unwrap().signals().released().connect(&slot_restore_default); }

        // Show the Dialog, save the current settings, and return them.
        unsafe { if dialog.as_mut().unwrap().exec() == 1 { Some(settings_dialog.borrow().save_from_settings_dialog()) }

        // Otherwise, return None.
        else { None } }
    }*/
/*
    /// This function loads the data from the Settings struct to the Settings Dialog.
    pub fn load_to_settings_dialog(&mut self, settings: &Settings) {

        // Load the MyMod Path, if exists.
        unsafe { self.paths_mymod_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(settings.paths["mymods_base_path"].clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter_mut() {
            unsafe { path.as_mut().unwrap().set_text(&QString::from_std_str(&settings.paths[key].clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }
        }

        // Get the Default Game.
        for (index, (folder_name,_)) in SUPPORTED_GAMES.iter().enumerate() {
            if *folder_name == settings.settings_string["default_game"] {
                unsafe { self.extra_default_game_combobox.as_mut().unwrap().set_current_index(index as i32); }
                break;
            }
        }

        // Load the UI Stuff.
        unsafe { self.ui_adjust_columns_to_content.as_mut().unwrap().set_checked(settings.settings_bool["adjust_columns_to_content"]); }
        unsafe { self.ui_extend_last_column_on_tables.as_mut().unwrap().set_checked(settings.settings_bool["extend_last_column_on_tables"]); }
        unsafe { self.ui_disable_combos_on_tables.as_mut().unwrap().set_checked(settings.settings_bool["disable_combos_on_tables"]); }
        unsafe { self.ui_start_maximized.as_mut().unwrap().set_checked(settings.settings_bool["start_maximized"]); }
        unsafe { self.ui_remember_table_state_permanently.as_mut().unwrap().set_checked(settings.settings_bool["remember_table_state_permanently"]); }
        unsafe { self.ui_use_dark_theme.as_mut().unwrap().set_checked(settings.settings_bool["use_dark_theme"]); }

        // Load the UI TableView Stuff.
        unsafe { self.ui_table_view_remember_column_sorting.as_mut().unwrap().set_checked(settings.settings_bool["remember_column_sorting"]); }
        unsafe { self.ui_table_view_remember_column_visual_order.as_mut().unwrap().set_checked(settings.settings_bool["remember_column_visual_order"]); }

        // Load the Extra Stuff.
        unsafe { self.extra_allow_editing_of_ca_packfiles.as_mut().unwrap().set_checked(settings.settings_bool["allow_editing_of_ca_packfiles"]); }
        unsafe { self.extra_check_updates_on_start.as_mut().unwrap().set_checked(settings.settings_bool["check_updates_on_start"]); }
        unsafe { self.extra_check_schema_updates_on_start.as_mut().unwrap().set_checked(settings.settings_bool["check_schema_updates_on_start"]); }
        unsafe { self.extra_use_dependency_checker.as_mut().unwrap().set_checked(settings.settings_bool["use_dependency_checker"]); }
        unsafe { self.extra_use_lazy_loading_checker.as_mut().unwrap().set_checked(settings.settings_bool["use_lazy_loading"]); }
        unsafe { self.extra_optimize_not_renamed_packedfiles_checker.as_mut().unwrap().set_checked(settings.settings_bool["optimize_not_renamed_packedfiles"]); }

        // Load the Debug Stuff.
        unsafe { self.debug_check_for_missing_table_definitions.as_mut().unwrap().set_checked(settings.settings_bool["check_for_missing_table_definitions"]); }
    }

    /// This function gets the data from the Settings Dialog and returns a Settings struct with that
    /// data in it.
    pub fn save_from_settings_dialog(&self) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new();

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let mymod_new_path = unsafe { PathBuf::from(self.paths_mymod_line_edit.as_mut().unwrap().text().to_std_string()) };
        settings.paths.insert("mymods_base_path".to_owned(), match mymod_new_path.is_dir() {
            true => Some(mymod_new_path),
            false => None,
        });

        // For each entry, we check if it's a valid directory and save it into Settings.
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let new_path = unsafe { PathBuf::from(line_edit.as_mut().unwrap().text().to_std_string()) };
            settings.paths.insert(key.to_owned(), match new_path.is_dir() {
                true => Some(new_path),
                false => None,
            });
        }

        // We get his game's folder, depending on the selected game.
        let mut game = unsafe { self.extra_default_game_combobox.as_mut().unwrap().current_text().to_std_string() };
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        settings.settings_string.insert("default_game".to_owned(), game);

        // Get the UI Settings.
        unsafe { settings.settings_bool.insert("adjust_columns_to_content".to_owned(), self.ui_adjust_columns_to_content.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("extend_last_column_on_tables".to_owned(), self.ui_extend_last_column_on_tables.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("disable_combos_on_tables".to_owned(), self.ui_disable_combos_on_tables.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("start_maximized".to_owned(), self.ui_start_maximized.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("remember_table_state_permanently".to_owned(), self.ui_remember_table_state_permanently.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("use_dark_theme".to_owned(), self.ui_use_dark_theme.as_mut().unwrap().is_checked()); }

        // Get the UI TableView Settings.
        unsafe { settings.settings_bool.insert("remember_column_sorting".to_owned(), self.ui_table_view_remember_column_sorting.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("remember_column_visual_order".to_owned(), self.ui_table_view_remember_column_visual_order.as_mut().unwrap().is_checked()); }

        // Get the Extra Settings.
        unsafe { settings.settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), self.extra_allow_editing_of_ca_packfiles.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("check_updates_on_start".to_owned(), self.extra_check_updates_on_start.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("check_schema_updates_on_start".to_owned(), self.extra_check_schema_updates_on_start.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("use_dependency_checker".to_owned(), self.extra_use_dependency_checker.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("use_lazy_loading".to_owned(), self.extra_use_lazy_loading_checker.as_mut().unwrap().is_checked()); }
        unsafe { settings.settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), self.extra_optimize_not_renamed_packedfiles_checker.as_mut().unwrap().is_checked()); }

        // Get the Debug Settings.
        unsafe { settings.settings_bool.insert("check_for_missing_table_definitions".to_owned(), self.debug_check_for_missing_table_definitions.as_mut().unwrap().is_checked()); }

        // Return the new Settings.
        settings
    }
}*/
/*
/// Implementation of `MyModNewWindow`.
impl NewMyModDialog {

    /// This function creates the entire "New Mod" dialog. It returns the name of the mod and the
    /// folder_name of the game.
    pub fn create_new_mymod_dialog(app_ui: &AppUI) -> Option<(String, String)> {

        //-------------------------------------------------------------------------------------------//
        // Creating the New MyMod Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the "New MyMod" Dialog.
        let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };

        // Change his title.
        dialog.set_window_title(&QString::from_std_str("New MyMod"));

        // Set it Modal, so you can't touch the Main Window with this dialog open.
        dialog.set_modal(true);

        // Resize the Dialog.
        dialog.resize((300, 0));

        // Create the main Grid.
        let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);

        // Create the Advices Frame.
        let advices_frame = Frame::new().into_raw();
        let advices_grid = create_grid_layout_unsafe(advices_frame as *mut Widget);

        // Create the "Advices" Label.
        let advices_label = Label::new(&QString::from_std_str("Things to take into account before creating a new mod:
	- Select the game you'll make the mod for.
	- Pick an simple name (it shouldn't end in *.pack).
	- If you want to use multiple words, use \"_\" instead of \" \".
	- You can't create a mod for a game that has no path set in the settings.")).into_raw();

        unsafe {
            advices_grid.as_mut().unwrap().add_widget((advices_label as *mut Widget, 0, 0, 1, 1));
            main_grid.as_mut().unwrap().add_widget((advices_frame as *mut Widget, 0, 0, 1, 2));
        }

        // Create the "MyMod's Name" Label and LineEdit.
        let mymod_name_label = Label::new(&QString::from_std_str("Name of the Mod:")).into_raw();
        let mymod_name_line_edit = LineEdit::new(()).into_raw();

        // Configure the "MyMod's Name" LineEdit.
        unsafe { mymod_name_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("For example: one_ring_for_me")); }

        // Create the "MyMod's Game" Label and ComboBox.
        let mymod_game_label = Label::new(&QString::from_std_str("Game of the Mod:")).into_raw();
        let mymod_game_combobox = ComboBox::new().into_raw();
        let mut mymod_game_model = StandardItemModel::new(());
        unsafe { mymod_game_combobox.as_mut().unwrap().set_model(mymod_game_model.static_cast_mut()); }

        // Add the games to the ComboBox.
        unsafe { for (_, game) in SUPPORTED_GAMES.iter() { if game.supports_editing { mymod_game_combobox.as_mut().unwrap().add_item(&QString::from_std_str(&game.display_name)); }} }

        // Add all the widgets to the main grid.
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_line_edit as *mut Widget, 1, 1, 1, 1)); }

        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_combobox as *mut Widget, 2, 1, 1, 1)); }

        // Create the bottom ButtonBox.
        let mut button_box = DialogButtonBox::new(());
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.static_cast_mut() as *mut Widget, 3, 0, 1, 2)); }

        // Create the bottom Buttons.
        let cancel_button;
        let accept_button;

        // Add them to the Dialog.
        cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        // Disable the "Accept" button by default.
        unsafe { accept_button.as_mut().unwrap().set_enabled(false); }

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let new_mymod_dialog = Self {
            mymod_game_combobox,
            mymod_name_line_edit,
            cancel_button,
            accept_button,
        };

        //-------------------------------------------------------------------------------------------//
        // Slots for the Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we change the name of the mod.
        let slot_mymod_line_edit_change = SlotNoArgs::new(clone!(
            new_mymod_dialog => move || {
                check_my_mod_validity(&new_mymod_dialog);
            }
        ));

        // What happens when we change the game of the mod.
        let slot_mymod_combobox_change = SlotNoArgs::new(clone!(
            new_mymod_dialog => move || {
                check_my_mod_validity(&new_mymod_dialog);
            }
        ));

        //-------------------------------------------------------------------------------------------//
        // Actions for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we change the name of the mod.
        unsafe { new_mymod_dialog.mymod_name_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_mymod_line_edit_change); }

        // What happens when we change the game of the mod.
        unsafe { new_mymod_dialog.mymod_game_combobox.as_mut().unwrap().signals().current_text_changed().connect(&slot_mymod_combobox_change); }

        // What happens when we hit the "Cancel" button.
        unsafe { new_mymod_dialog.cancel_button.as_mut().unwrap().signals().released().connect(&dialog.slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { new_mymod_dialog.accept_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }


        // Show the Dialog, save the current settings, and return them.
        if dialog.exec() == 1 {

            // Get the text from the LineEdit.
            let mod_name;
            unsafe { mod_name = QString::to_std_string(&new_mymod_dialog.mymod_name_line_edit.as_mut().unwrap().text()); }

            // Get the Game Selected in the ComboBox.
            let mut game;
            unsafe { game = new_mymod_dialog.mymod_game_combobox.as_mut().unwrap().current_text().to_std_string(); }
            if let Some(index) = game.find('&') { game.remove(index); }
            let mod_game = game.replace(' ', "_").to_lowercase();

            // Return it.
            Some((mod_name, mod_game))
        }

        // Otherwise, return None.
        else { None }
    }
}

/// This function takes care of updating the provided LineEdits with the selected path.
fn update_entry_path(
    line_edit: *mut LineEdit,
    dialog: *mut Dialog,
) {

    // Create the FileDialog to get the path.
    let mut file_dialog;
    unsafe {
        file_dialog = FileDialog::new_unsafe((
            dialog as *mut Widget,
            &QString::from_std_str("Select Folder"),
        ));
    }

    // Set it to only search Folders.
    file_dialog.set_file_mode(FileMode::Directory);
    file_dialog.set_option(ShowDirsOnly);

    // Get the old Path, if exists.
    let old_path;
    unsafe { old_path = line_edit.as_mut().unwrap().text().to_std_string(); }

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

/// Check if the new MyMod's name is valid or not, disabling or enabling the "Accept" button in response.
fn check_my_mod_validity(mymod_dialog: &NewMyModDialog) {

    // Get the text from the LineEdit.
    let mod_name;
    unsafe { mod_name = mymod_dialog.mymod_name_line_edit.as_mut().unwrap().text().to_std_string(); }

    // Get the Game Selected in the ComboBox.
    let mut game;
    unsafe { game = mymod_dialog.mymod_game_combobox.as_mut().unwrap().current_text().to_std_string(); }
    if let Some(index) = game.find('&') { game.remove(index); }
    game = game.replace(' ', "_");
    let mod_game = game.to_lowercase();

    // If there is text and it doesn't have whitespaces...
    if !mod_name.is_empty() && !mod_name.contains(' ') {

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        if let Some(ref mod_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {
            let mut mod_path = mod_path.clone();
            mod_path.push(mod_game);
            mod_path.push(format!("{}.pack", mod_name));

            // If a mod with that name for that game already exists, disable the "Accept" button.
            if mod_path.is_file() { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); }}

            // If the name is available, enable the `Accept` button.
            else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(true); } }
        }

        // If there is no "MyMod" path configured, disable the button.
        else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); } }
    }

    // If name is empty, disable the button.
    else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); } }
}
*/