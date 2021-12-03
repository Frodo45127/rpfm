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

use qt_widgets::{QCheckBox, QTabWidget};
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::{QDialogButtonBox, q_dialog_button_box, q_dialog_button_box::ButtonRole};
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QSpinBox;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_gui::QGuiApplication;
use qt_gui::{QColor, q_color::NameFormat};
use qt_gui::{QPalette, q_palette::ColorRole};
use qt_gui::QStandardItemModel;

use qt_core::AlignmentFlag;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QString;
use qt_core::QPtr;
use qt_core::QSettings;
use qt_core::QVariant;

use cpp_core::CastInto;
use cpp_core::Ptr;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_error::Result;

use rpfm_lib::SUPPORTED_GAMES;
use rpfm_lib::games::supported_games::*;
use rpfm_lib::settings::{Settings, MYMOD_BASE_PATH, ZIP_PATH};
use rpfm_lib::updater::{BETA, STABLE, get_update_channel, UpdateChannel};

use crate::AppUI;
use crate::{Locale, locale::{qtr, qtre}};
use crate::ffi::*;
use crate::QT_PROGRAM;
use crate::QT_ORG;
use crate::SETTINGS;
use crate::utils::{create_grid_layout, show_dialog};
use self::slots::SettingsUISlots;

mod connections;
mod slots;
mod tips;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the widgets used in the Settings Window.
pub struct SettingsUI {

    //-------------------------------------------------------------------------------//
    // `Dialog` window.
    //-------------------------------------------------------------------------------//
    pub dialog: QBox<QDialog>,
    pub tab_widget: QBox<QTabWidget>,
    pub paths_tab: QBox<QWidget>,
    pub settings_tab: QBox<QWidget>,

    //-------------------------------------------------------------------------------//
    // `Path` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub paths_zip_label: QBox<QLabel>,
    pub paths_zip_line_edit: QBox<QLineEdit>,
    pub paths_zip_button: QBox<QPushButton>,
    pub paths_mymod_label: QBox<QLabel>,
    pub paths_mymod_line_edit: QBox<QLineEdit>,
    pub paths_mymod_button: QBox<QPushButton>,

    pub paths_spoilers: BTreeMap<String, QBox<QWidget>>,

    pub paths_games_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    pub paths_games_buttons: BTreeMap<String, QBox<QPushButton>>,

    pub paths_asskit_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    pub paths_asskit_buttons: BTreeMap<String, QBox<QPushButton>>,

    //-------------------------------------------------------------------------------//
    // `General` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub general_language_label: QBox<QLabel>,
    pub extra_global_default_game_label: QBox<QLabel>,
    pub extra_network_update_channel_label: QBox<QLabel>,
    pub extra_packfile_autosave_interval_label: QBox<QLabel>,
    pub extra_packfile_autosave_amount_label: QBox<QLabel>,
    pub extra_network_check_updates_on_start_label: QBox<QLabel>,
    pub extra_network_check_schema_updates_on_start_label: QBox<QLabel>,
    pub extra_packfile_allow_editing_of_ca_packfiles_label: QBox<QLabel>,
    pub extra_packfile_optimize_not_renamed_packedfiles_label: QBox<QLabel>,
    pub extra_packfile_use_lazy_loading_label: QBox<QLabel>,
    pub extra_packfile_disable_uuid_regeneration_on_db_tables_label: QBox<QLabel>,
    pub extra_packfile_disable_file_previews_label: QBox<QLabel>,
    pub ui_global_use_dark_theme_label: QBox<QLabel>,
    pub ui_window_start_maximized_label: QBox<QLabel>,
    pub ui_window_hide_background_icon_label: QBox<QLabel>,
    pub general_packfile_treeview_resize_to_fit_label: QBox<QLabel>,
    pub general_packfile_treeview_expand_treeview_when_adding_items_label: QBox<QLabel>,

    pub general_language_combobox: QBox<QComboBox>,
    pub extra_global_default_game_combobox: QBox<QComboBox>,
    pub extra_network_update_channel_combobox: QBox<QComboBox>,
    pub extra_packfile_autosave_interval_spinbox: QBox<QSpinBox>,
    pub extra_packfile_autosave_amount_spinbox: QBox<QSpinBox>,
    pub extra_network_check_updates_on_start_checkbox: QBox<QCheckBox>,
    pub extra_network_check_schema_updates_on_start_checkbox: QBox<QCheckBox>,
    pub extra_packfile_allow_editing_of_ca_packfiles_checkbox: QBox<QCheckBox>,
    pub extra_packfile_optimize_not_renamed_packedfiles_checkbox: QBox<QCheckBox>,
    pub extra_packfile_use_lazy_loading_checkbox: QBox<QCheckBox>,
    pub extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox: QBox<QCheckBox>,
    pub extra_packfile_disable_file_previews_checkbox: QBox<QCheckBox>,
    pub ui_global_use_dark_theme_checkbox: QBox<QCheckBox>,
    pub ui_window_start_maximized_checkbox: QBox<QCheckBox>,
    pub ui_window_hide_background_icon_checkbox: QBox<QCheckBox>,
    pub general_packfile_treeview_resize_to_fit_checkbox: QBox<QCheckBox>,
    pub general_packfile_treeview_expand_treeview_when_adding_items_checkbox: QBox<QCheckBox>,

    //-------------------------------------------------------------------------------//
    // `Table` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub ui_table_adjust_columns_to_content_label: QBox<QLabel>,
    pub ui_table_disable_combos_label: QBox<QLabel>,
    pub ui_table_extend_last_column_label: QBox<QLabel>,
    pub ui_table_tight_table_mode_label: QBox<QLabel>,
    pub ui_table_resize_on_edit_label: QBox<QLabel>,
    pub ui_table_use_old_column_order_label: QBox<QLabel>,
    pub ui_table_use_right_size_markers_label: QBox<QLabel>,

    pub ui_table_adjust_columns_to_content_checkbox: QBox<QCheckBox>,
    pub ui_table_disable_combos_checkbox: QBox<QCheckBox>,
    pub ui_table_extend_last_column_checkbox: QBox<QCheckBox>,
    pub ui_table_tight_table_mode_checkbox: QBox<QCheckBox>,
    pub ui_table_resize_on_edit_checkbox: QBox<QCheckBox>,
    pub ui_table_use_old_column_order_checkbox: QBox<QCheckBox>,
    pub ui_table_use_right_size_markers_checkbox: QBox<QCheckBox>,

    pub ui_table_colour_table_added_label: QBox<QLabel>,
    pub ui_table_colour_table_modified_label: QBox<QLabel>,
    pub ui_table_colour_diagnostic_error_label: QBox<QLabel>,
    pub ui_table_colour_diagnostic_warning_label: QBox<QLabel>,
    pub ui_table_colour_diagnostic_info_label: QBox<QLabel>,

    pub ui_table_colour_light_table_added_button: QBox<QPushButton>,
    pub ui_table_colour_light_table_modified_button: QBox<QPushButton>,
    pub ui_table_colour_light_diagnostic_error_button: QBox<QPushButton>,
    pub ui_table_colour_light_diagnostic_warning_button: QBox<QPushButton>,
    pub ui_table_colour_light_diagnostic_info_button: QBox<QPushButton>,
    pub ui_table_colour_dark_table_added_button: QBox<QPushButton>,
    pub ui_table_colour_dark_table_modified_button: QBox<QPushButton>,
    pub ui_table_colour_dark_diagnostic_error_button: QBox<QPushButton>,
    pub ui_table_colour_dark_diagnostic_warning_button: QBox<QPushButton>,
    pub ui_table_colour_dark_diagnostic_info_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // `Debug` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub debug_check_for_missing_table_definitions_label: QBox<QLabel>,
    pub debug_check_for_missing_table_definitions_checkbox: QBox<QCheckBox>,
    pub debug_enable_debug_menu_label: QBox<QLabel>,
    pub debug_enable_debug_menu_checkbox: QBox<QCheckBox>,
    pub debug_spoof_ca_authoring_tool_label: QBox<QLabel>,
    pub debug_spoof_ca_authoring_tool_checkbox: QBox<QCheckBox>,
    pub debug_enable_rigidmodel_editor_label: QBox<QLabel>,
    pub debug_enable_rigidmodel_editor_checkbox: QBox<QCheckBox>,
    pub debug_enable_esf_editor_label: QBox<QLabel>,
    pub debug_enable_esf_editor_checkbox: QBox<QCheckBox>,
    pub debug_enable_unit_editor_label: QBox<QLabel>,
    pub debug_enable_unit_editor_checkbox: QBox<QCheckBox>,

    pub debug_clear_autosave_folder_button: QBox<QPushButton>,
    pub debug_clear_schema_folder_button: QBox<QPushButton>,
    pub debug_clear_layout_settings_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // `Diagnostics` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub diagnostics_diagnostics_trigger_on_open_label: QBox<QLabel>,
    pub diagnostics_diagnostics_trigger_on_table_edit_label: QBox<QLabel>,

    pub diagnostics_diagnostics_trigger_on_open_checkbox: QBox<QCheckBox>,
    pub diagnostics_diagnostics_trigger_on_table_edit_checkbox: QBox<QCheckBox>,

    //-------------------------------------------------------------------------------//
    // `Warning` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub warning_message: QBox<QLabel>,

    //-------------------------------------------------------------------------------//
    // `ButtonBox` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    pub button_box_restore_default_button: QPtr<QPushButton>,
    pub button_box_text_editor_settings_button: QBox<QPushButton>,
    pub button_box_shortcuts_button: QBox<QPushButton>,
    pub button_box_font_settings_button: QBox<QPushButton>,
    pub button_box_cancel_button: QPtr<QPushButton>,
    pub button_box_accept_button: QPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUI`.
impl SettingsUI {

    /// This function creates a ***Settings*** dialog, execute it, and returns a new `Settings`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Option<Settings> {
        let settings_ui = Rc::new(Self::new_with_parent(&app_ui.main_window));
        let slots = SettingsUISlots::new(&settings_ui, app_ui);

        connections::set_connections(&settings_ui, &slots);
        tips::set_tips(&settings_ui);

        // If load fails due to missing locale folder, show the error and cancel the settings edition.
        if let Err(error) = settings_ui.load(&SETTINGS.read().unwrap()) {
            show_dialog(&app_ui.main_window, error, false);
            return None;
        }

        if settings_ui.dialog.exec() == 1 { Some(settings_ui.save()) }
        else { None }
    }

    /// This function creates a new `SettingsUI` and links it to the provided parent.
    unsafe fn new_with_parent(parent: impl CastInto<Ptr<QWidget>>) -> Self {

        // Initialize and configure the settings window.
        let dialog = QDialog::new_1a(parent);
        dialog.set_window_title(&qtr("settings_title"));
        dialog.set_modal(true);
        dialog.resize_2a(750, 0);

        let main_grid = create_grid_layout(dialog.static_upcast());
        main_grid.set_contents_margins_4a(4, 0, 4, 4);
        main_grid.set_spacing(4);

        let tab_widget = QTabWidget::new_1a(&dialog);
        let paths_tab = QWidget::new_1a(&tab_widget);
        let settings_tab = QWidget::new_1a(&tab_widget);

        let paths_grid = create_grid_layout(paths_tab.static_upcast());
        paths_grid.set_contents_margins_4a(4, 0, 4, 4);
        paths_grid.set_spacing(4);

        let settings_grid = create_grid_layout(settings_tab.static_upcast());
        settings_grid.set_contents_margins_4a(4, 0, 4, 4);
        settings_grid.set_spacing(4);

        tab_widget.add_tab_2a(&paths_tab, &qtr("settings_tab_paths"));
        tab_widget.add_tab_2a(&settings_tab, &qtr("settings_tab_settings"));

        main_grid.add_widget_5a(&tab_widget, 0, 0, 1, 3);
        //-----------------------------------------------//
        // `Game Paths` Frame.
        //-----------------------------------------------//
        let paths_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_game_paths_title"), &dialog);
        let main_paths_grid = create_grid_layout(paths_frame.static_upcast());
        main_paths_grid.set_contents_margins_4a(4, 0, 4, 0);

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_spoilers = BTreeMap::new();

        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();

        let mut paths_asskit_line_edits = BTreeMap::new();
        let mut paths_asskit_buttons = BTreeMap::new();

        for (index, game_supported) in SUPPORTED_GAMES.get_games().iter().enumerate() {
            let spoiler = new_spoiler_safe(&QString::from_std_str(game_supported.get_display_name()).as_ptr(), 200, &paths_frame.as_ptr().static_upcast());

            // Note: ignore the warnings caused by this. They're harmless.
            let game_path_layout = create_grid_layout(spoiler.static_upcast());

            let game_key = game_supported.get_game_key_name();
            let game_label = QLabel::from_q_string_q_widget(&qtr("settings_game_label"), &spoiler);
            let game_line_edit = QLineEdit::from_q_widget(&spoiler);
            let game_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), &spoiler);
            game_line_edit.set_placeholder_text(&qtre("settings_game_line_ph", &[game_supported.get_display_name()]));

            game_path_layout.add_widget_5a(&game_label, 0, 0, 1, 1);
            game_path_layout.add_widget_5a(&game_line_edit, 0, 1, 1, 1);
            game_path_layout.add_widget_5a(&game_button, 0, 2, 1, 1);

            // Add the LineEdit and Button to the list.
            paths_games_line_edits.insert(game_key.to_owned(), game_line_edit);
            paths_games_buttons.insert(game_key.to_owned(), game_button);

            if game_key != KEY_EMPIRE &&
                game_key != KEY_NAPOLEON &&
                game_key != KEY_ARENA {

                let asskit_label = QLabel::from_q_string_q_widget(&qtr("settings_asskit_label"), &spoiler);
                let asskit_line_edit = QLineEdit::from_q_widget(&spoiler);
                let asskit_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), &spoiler);
                asskit_line_edit.set_placeholder_text(&qtre("settings_asskit_line_ph", &[game_supported.get_display_name()]));

                game_path_layout.add_widget_5a(&asskit_label, 1, 0, 1, 1);
                game_path_layout.add_widget_5a(&asskit_line_edit, 1, 1, 1, 1);
                game_path_layout.add_widget_5a(&asskit_button, 1, 2, 1, 1);

                // Add the LineEdit and Button to the list.
                paths_asskit_line_edits.insert(game_key.to_owned(), asskit_line_edit);
                paths_asskit_buttons.insert(game_key.to_owned(), asskit_button);
            }

            set_spoiler_layout_safe(&spoiler.as_ptr(), &game_path_layout.as_ptr().static_upcast());
            main_paths_grid.add_widget_5a(&spoiler, index as i32 + 1, 0, 1, 1);
            paths_spoilers.insert(game_key.to_owned(), spoiler);
        }

        paths_grid.add_widget_5a(&paths_frame, 0, 0, 1, 3);

        //-----------------------------------------------//
        // `Extra Paths` Frame.
        //-----------------------------------------------//

        let extra_paths_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_extra_paths_title"), &dialog);
        let extra_paths_grid = create_grid_layout(extra_paths_frame.static_upcast());
        extra_paths_grid.set_contents_margins_4a(4, 0, 4, 0);

        // Create the MyMod's path stuff.
        let paths_mymod_label = QLabel::from_q_string_q_widget(&qtr("settings_paths_mymod"), &extra_paths_frame);
        let paths_mymod_line_edit = QLineEdit::from_q_widget(&extra_paths_frame);
        let paths_mymod_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), &extra_paths_frame);
        paths_mymod_line_edit.set_placeholder_text(&qtr("settings_paths_mymod_ph"));

        extra_paths_grid.add_widget_5a(&paths_mymod_label, 0, 0, 1, 1);
        extra_paths_grid.add_widget_5a(&paths_mymod_line_edit, 0, 1, 1, 1);
        extra_paths_grid.add_widget_5a(&paths_mymod_button, 0, 2, 1, 1);

        // Create the 7Zip path stuff.
        let paths_zip_label = QLabel::from_q_string_q_widget(&qtr("settings_paths_zip"), &extra_paths_frame);
        let paths_zip_line_edit = QLineEdit::from_q_widget(&extra_paths_frame);
        let paths_zip_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), &extra_paths_frame);
        paths_zip_line_edit.set_placeholder_text(&qtr("settings_paths_zip_ph"));

        extra_paths_grid.add_widget_5a(&paths_zip_label, 1, 0, 1, 1);
        extra_paths_grid.add_widget_5a(&paths_zip_line_edit, 1, 1, 1, 1);
        extra_paths_grid.add_widget_5a(&paths_zip_button, 1, 2, 1, 1);

        paths_grid.add_widget_5a(&extra_paths_frame, 1, 0, 1, 3);

        //-----------------------------------------------//
        // `General` Frame.
        //-----------------------------------------------//
        let general_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_ui_title"), &dialog);
        let general_grid = create_grid_layout(general_frame.static_upcast());
        general_grid.set_contents_margins_4a(4, 0, 4, 0);
        general_grid.set_spacing(4);

        // Language combo.
        let general_language_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_language"), &general_frame);
        let general_language_combobox = QComboBox::new_1a(&general_frame);

        let general_language_model = QStandardItemModel::new_1a(&general_language_combobox);
        general_language_combobox.set_model(&general_language_model);
        if let Ok(locales) = Locale::get_available_locales() {
            for (language, _) in locales {
                general_language_combobox.add_item_q_string(&QString::from_std_str(&language));
            }
        }

        // Default game combo.
        let extra_global_default_game_label = QLabel::from_q_string_q_widget(&qtr("settings_default_game"), &general_frame);
        let extra_global_default_game_combobox = QComboBox::new_1a(&general_frame);

        let extra_global_default_game_model = QStandardItemModel::new_1a(&extra_global_default_game_combobox);
        extra_global_default_game_combobox.set_model(&extra_global_default_game_model);
        for game in SUPPORTED_GAMES.get_games().iter() {
            extra_global_default_game_combobox.add_item_q_string(&QString::from_std_str(&game.get_display_name()));
        }

        // Update channel combo.
        let extra_network_update_channel_label = QLabel::from_q_string_q_widget(&qtr("settings_update_channel"), &general_frame);
        let extra_network_update_channel_combobox = QComboBox::new_1a(&general_frame);
        extra_network_update_channel_combobox.add_item_q_string(&QString::from_std_str(STABLE));
        extra_network_update_channel_combobox.add_item_q_string(&QString::from_std_str(BETA));

        // Autosave stuff
        let extra_packfile_autosave_interval_label = QLabel::from_q_string_q_widget(&qtr("settings_autosave_interval"), &general_frame);
        let extra_packfile_autosave_amount_label = QLabel::from_q_string_q_widget(&qtr("settings_autosave_amount"), &general_frame);
        let extra_packfile_autosave_interval_spinbox = QSpinBox::new_1a(&general_frame);
        let extra_packfile_autosave_amount_spinbox = QSpinBox::new_1a(&general_frame);

        // Update checkers.
        let extra_network_check_updates_on_start_label = QLabel::from_q_string_q_widget(&qtr("settings_check_updates_on_start"), &general_frame);
        let extra_network_check_schema_updates_on_start_label = QLabel::from_q_string_q_widget(&qtr("settings_check_schema_updates_on_start"), &general_frame);
        let extra_network_check_updates_on_start_checkbox = QCheckBox::from_q_widget(&general_frame);
        let extra_network_check_schema_updates_on_start_checkbox = QCheckBox::from_q_widget(&general_frame);

        // Behavior settings.
        let extra_packfile_allow_editing_of_ca_packfiles_label = QLabel::from_q_string_q_widget(&qtr("settings_allow_editing_of_ca_packfiles"), &general_frame);
        let extra_packfile_allow_editing_of_ca_packfiles_checkbox = QCheckBox::from_q_widget(&general_frame);

        let extra_packfile_optimize_not_renamed_packedfiles_label = QLabel::from_q_string_q_widget(&qtr("settings_optimize_not_renamed_packedfiles"), &general_frame);
        let extra_packfile_optimize_not_renamed_packedfiles_checkbox = QCheckBox::from_q_widget(&general_frame);

        let extra_packfile_disable_file_previews_label = QLabel::from_q_string_q_widget(&qtr("settings_disable_file_previews"), &general_frame);
        let extra_packfile_disable_file_previews_checkbox = QCheckBox::from_q_widget(&general_frame);

        let ui_global_use_dark_theme_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_dark_theme"), &general_frame);
        let ui_global_use_dark_theme_checkbox = QCheckBox::from_q_widget(&general_frame);

        let ui_window_start_maximized_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_window_start_maximized_label"), &general_frame);
        let ui_window_start_maximized_checkbox = QCheckBox::from_q_widget(&general_frame);

        let ui_window_hide_background_icon_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_window_hide_background_icon"), &general_frame);
        let ui_window_hide_background_icon_checkbox = QCheckBox::from_q_widget(&general_frame);

        let general_packfile_treeview_resize_to_fit_label = QLabel::from_q_string_q_widget(&qtr("settings_packfile_treeview_resize_to_fit"), &general_frame);
        let general_packfile_treeview_resize_to_fit_checkbox = QCheckBox::from_q_widget(&general_frame);
        general_packfile_treeview_resize_to_fit_label.set_visible(false);
        general_packfile_treeview_resize_to_fit_checkbox.set_visible(false);

        let general_packfile_treeview_expand_treeview_when_adding_items_label = QLabel::from_q_string_q_widget(&qtr("settings_expand_treeview_when_adding_items"), &general_frame);
        let general_packfile_treeview_expand_treeview_when_adding_items_checkbox = QCheckBox::from_q_widget(&general_frame);

        // Adding to the grid.
        general_grid.add_widget_5a(&general_language_label, 0, 0, 1, 1);
        general_grid.add_widget_5a(&general_language_combobox, 0, 1, 1, 1);

        general_grid.add_widget_5a(&extra_global_default_game_label, 1, 0, 1, 1);
        general_grid.add_widget_5a(&extra_global_default_game_combobox, 1, 1, 1, 1);

        general_grid.add_widget_5a(&extra_network_update_channel_label, 2, 0, 1, 1);
        general_grid.add_widget_5a(&extra_network_update_channel_combobox, 2, 1, 1, 1);

        general_grid.add_widget_5a(&extra_packfile_autosave_amount_label, 3, 0, 1, 1);
        general_grid.add_widget_5a(&extra_packfile_autosave_amount_spinbox, 3, 1, 1, 1);

        general_grid.add_widget_5a(&extra_packfile_autosave_interval_label, 4, 0, 1, 1);
        general_grid.add_widget_5a(&extra_packfile_autosave_interval_spinbox, 4, 1, 1, 1);

        general_grid.add_widget_5a(&extra_network_check_updates_on_start_label, 5, 0, 1, 1);
        general_grid.add_widget_5a(&extra_network_check_updates_on_start_checkbox, 5, 1, 1, 1);

        general_grid.add_widget_5a(&extra_network_check_schema_updates_on_start_label, 6, 0, 1, 1);
        general_grid.add_widget_5a(&extra_network_check_schema_updates_on_start_checkbox, 6, 1, 1, 1);

        general_grid.add_widget_5a(&extra_packfile_allow_editing_of_ca_packfiles_label, 8, 0, 1, 1);
        general_grid.add_widget_5a(&extra_packfile_allow_editing_of_ca_packfiles_checkbox, 8, 1, 1, 1);

        general_grid.add_widget_5a(&extra_packfile_optimize_not_renamed_packedfiles_label, 9, 0, 1, 1);
        general_grid.add_widget_5a(&extra_packfile_optimize_not_renamed_packedfiles_checkbox, 9, 1, 1, 1);

        general_grid.add_widget_5a(&extra_packfile_disable_file_previews_label, 10, 0, 1, 1);
        general_grid.add_widget_5a(&extra_packfile_disable_file_previews_checkbox, 10, 1, 1, 1);

        general_grid.add_widget_5a(&ui_global_use_dark_theme_label, 13, 0, 1, 1);
        general_grid.add_widget_5a(&ui_global_use_dark_theme_checkbox, 13, 1, 1, 1);

        general_grid.add_widget_5a(&ui_window_start_maximized_label, 14, 0, 1, 1);
        general_grid.add_widget_5a(&ui_window_start_maximized_checkbox, 14, 1, 1, 1);

        general_grid.add_widget_5a(&ui_window_hide_background_icon_label, 15, 0, 1, 1);
        general_grid.add_widget_5a(&ui_window_hide_background_icon_checkbox, 15, 1, 1, 1);

        //general_grid.add_widget_5a(&general_packfile_treeview_resize_to_fit_label, 14, 0, 1, 1);
        //general_grid.add_widget_5a(&general_packfile_treeview_resize_to_fit_checkbox, 14, 1, 1, 1);

        general_grid.add_widget_5a(&general_packfile_treeview_expand_treeview_when_adding_items_label, 16, 0, 1, 1);
        general_grid.add_widget_5a(&general_packfile_treeview_expand_treeview_when_adding_items_checkbox, 16, 1, 1, 1);

        settings_grid.add_widget_5a(&general_frame, 2, 0, 2, 1);

        //-----------------------------------------------//
        // `Table` Frame.
        //-----------------------------------------------//

        let ui_table_view_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_table_title"), &dialog);
        let ui_table_view_grid = create_grid_layout(ui_table_view_frame.static_upcast());
        ui_table_view_grid.set_contents_margins_4a(4, 0, 4, 0);
        ui_table_view_grid.set_spacing(4);
        ui_table_view_grid.set_row_stretch(99, 10);

        let ui_table_adjust_columns_to_content_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_adjust_columns_to_content"), &ui_table_view_frame);
        let ui_table_adjust_columns_to_content_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_disable_combos_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_disable_combos"), &ui_table_view_frame);
        let ui_table_disable_combos_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_extend_last_column_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_extend_last_column_label"), &ui_table_view_frame);
        let ui_table_extend_last_column_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_tight_table_mode_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_tight_table_mode_label"), &ui_table_view_frame);
        let ui_table_tight_table_mode_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_resize_on_edit_label = QLabel::from_q_string_q_widget(&qtr("settings_table_resize_on_edit"), &ui_table_view_frame);
        let ui_table_resize_on_edit_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_use_old_column_order_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_use_old_column_order_label"), &ui_table_view_frame);
        let ui_table_use_old_column_order_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let extra_packfile_disable_uuid_regeneration_on_db_tables_label = QLabel::from_q_string_q_widget(&qtr("settings_disable_uuid_regeneration_tables"), &ui_table_view_frame);
        let extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        let ui_table_use_right_size_markers_label = QLabel::from_q_string_q_widget(&qtr("settings_use_right_side_markers"), &ui_table_view_frame);
        let ui_table_use_right_size_markers_checkbox = QCheckBox::from_q_widget(&ui_table_view_frame);

        ui_table_view_grid.add_widget_5a(&ui_table_adjust_columns_to_content_label, 0, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_adjust_columns_to_content_checkbox, 0, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_disable_combos_label, 1, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_disable_combos_checkbox, 1, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_extend_last_column_label, 2, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_extend_last_column_checkbox, 2, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_tight_table_mode_label, 3, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_tight_table_mode_checkbox, 3, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_resize_on_edit_label, 4, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_resize_on_edit_checkbox, 4, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_use_old_column_order_label, 5, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_use_old_column_order_checkbox, 5, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&extra_packfile_disable_uuid_regeneration_on_db_tables_label, 6, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox, 6, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_use_right_size_markers_label, 7, 0, 1, 2);
        ui_table_view_grid.add_widget_5a(&ui_table_use_right_size_markers_checkbox, 7, 2, 1, 1);

        let settings_ui_table_colour_light_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_light_label"), &ui_table_view_frame);
        let settings_ui_table_colour_dark_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_dark_label"), &ui_table_view_frame);

        let ui_table_colour_table_added_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_table_added_label"), &ui_table_view_frame);
        let ui_table_colour_table_modified_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_table_modified_label"), &ui_table_view_frame);
        let ui_table_colour_diagnostic_error_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_diagnostic_error_label"), &ui_table_view_frame);
        let ui_table_colour_diagnostic_warning_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_diagnostic_warning_label"), &ui_table_view_frame);
        let ui_table_colour_diagnostic_info_label = QLabel::from_q_string_q_widget(&qtr("settings_ui_table_colour_diagnostic_info_label"), &ui_table_view_frame);
        ui_table_colour_table_added_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        ui_table_colour_table_modified_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        ui_table_colour_diagnostic_error_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        ui_table_colour_diagnostic_warning_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        ui_table_colour_diagnostic_info_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));

        let ui_table_colour_light_table_added_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_light_table_modified_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_light_diagnostic_error_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_light_diagnostic_warning_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_light_diagnostic_info_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_dark_table_added_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_dark_table_modified_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_dark_diagnostic_error_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_dark_diagnostic_warning_button = QPushButton::from_q_widget(&ui_table_view_frame);
        let ui_table_colour_dark_diagnostic_info_button = QPushButton::from_q_widget(&ui_table_view_frame);

        ui_table_colour_light_table_added_button.set_auto_fill_background(true);
        ui_table_colour_light_table_modified_button.set_auto_fill_background(true);
        ui_table_colour_light_diagnostic_error_button.set_auto_fill_background(true);
        ui_table_colour_light_diagnostic_warning_button.set_auto_fill_background(true);
        ui_table_colour_light_diagnostic_info_button.set_auto_fill_background(true);
        ui_table_colour_dark_table_added_button.set_auto_fill_background(true);
        ui_table_colour_dark_table_modified_button.set_auto_fill_background(true);
        ui_table_colour_dark_diagnostic_error_button.set_auto_fill_background(true);
        ui_table_colour_dark_diagnostic_warning_button.set_auto_fill_background(true);
        ui_table_colour_dark_diagnostic_info_button.set_auto_fill_background(true);

        ui_table_view_grid.add_widget_5a(&settings_ui_table_colour_light_label, 90, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&settings_ui_table_colour_dark_label, 90, 2, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_colour_table_added_label, 92, 1, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_table_modified_label, 93, 1, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_diagnostic_error_label, 95, 1, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_diagnostic_warning_label, 96, 1, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_diagnostic_info_label, 97, 1, 1, 1);

        ui_table_view_grid.add_widget_5a(&ui_table_colour_light_table_added_button, 92, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_light_table_modified_button, 93, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_light_diagnostic_error_button, 95, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_light_diagnostic_warning_button, 96, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_light_diagnostic_info_button, 97, 0, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_dark_table_added_button, 92, 2, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_dark_table_modified_button, 93, 2, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_dark_diagnostic_error_button, 95, 2, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_dark_diagnostic_warning_button, 96, 2, 1, 1);
        ui_table_view_grid.add_widget_5a(&ui_table_colour_dark_diagnostic_info_button, 97, 2, 1, 1);

        settings_grid.add_widget_5a(&ui_table_view_frame, 2, 1, 1, 1);

        //-----------------------------------------------//
        // `Debug` Frame.
        //-----------------------------------------------//
        let debug_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_debug_title"), &dialog);
        let debug_grid = create_grid_layout(debug_frame.static_upcast());
        debug_grid.set_contents_margins_4a(4, 0, 4, 0);
        debug_grid.set_spacing(4);
        debug_grid.set_row_stretch(80, 10);

        let debug_check_for_missing_table_definitions_label = QLabel::from_q_string_q_widget(&qtr("settings_debug_missing_table"), &debug_frame);
        let debug_enable_debug_menu_label = QLabel::from_q_string_q_widget(&qtr("settings_debug_enable_debug_menu"), &debug_frame);
        let debug_spoof_ca_authoring_tool_label = QLabel::from_q_string_q_widget(&qtr("settings_debug_spoof_ca_authoring_tool"), &debug_frame);
        let debug_enable_rigidmodel_editor_label = QLabel::from_q_string_q_widget(&qtr("settings_enable_rigidmodel_editor"), &debug_frame);
        let debug_enable_esf_editor_label = QLabel::from_q_string_q_widget(&qtr("settings_enable_esf_editor"), &debug_frame);
        let debug_enable_unit_editor_label = QLabel::from_q_string_q_widget(&qtr("settings_enable_unit_editor"), &debug_frame);

        let debug_check_for_missing_table_definitions_checkbox = QCheckBox::from_q_widget(&debug_frame);
        let debug_enable_debug_menu_checkbox = QCheckBox::from_q_widget(&debug_frame);
        let debug_spoof_ca_authoring_tool_checkbox = QCheckBox::from_q_widget(&debug_frame);
        let debug_enable_rigidmodel_editor_checkbox = QCheckBox::from_q_widget(&debug_frame);
        let debug_enable_esf_editor_checkbox = QCheckBox::from_q_widget(&debug_frame);
        let debug_enable_unit_editor_checkbox = QCheckBox::from_q_widget(&debug_frame);

        let extra_packfile_use_lazy_loading_label = QLabel::from_q_string_q_widget(&qtr("settings_use_lazy_loading"), &debug_frame);
        let extra_packfile_use_lazy_loading_checkbox = QCheckBox::from_q_widget(&debug_frame);

        let debug_clear_autosave_folder_button = QPushButton::from_q_string_q_widget(&qtr("settings_debug_clear_autosave_folder"), &debug_frame);
        let debug_clear_schema_folder_button = QPushButton::from_q_string_q_widget(&qtr("settings_debug_clear_schema_folder"), &debug_frame);
        let debug_clear_layout_settings_button = QPushButton::from_q_string_q_widget(&qtr("settings_debug_clear_layout_settings"), &debug_frame);

        debug_grid.add_widget_5a(&debug_check_for_missing_table_definitions_label, 0, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_check_for_missing_table_definitions_checkbox, 0, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_enable_debug_menu_label, 1, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_enable_debug_menu_checkbox, 1, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_spoof_ca_authoring_tool_label, 2, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_spoof_ca_authoring_tool_checkbox, 2, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_enable_rigidmodel_editor_label, 3, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_enable_rigidmodel_editor_checkbox, 3, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_enable_esf_editor_label, 4, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_enable_esf_editor_checkbox, 4, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_enable_unit_editor_label, 5, 0, 1, 1);
        debug_grid.add_widget_5a(&debug_enable_unit_editor_checkbox, 5, 1, 1, 1);

        debug_grid.add_widget_5a(&extra_packfile_use_lazy_loading_label, 11, 0, 1, 1);
        debug_grid.add_widget_5a(&extra_packfile_use_lazy_loading_checkbox, 11, 1, 1, 1);

        debug_grid.add_widget_5a(&debug_clear_autosave_folder_button, 85, 0, 1, 2);
        debug_grid.add_widget_5a(&debug_clear_schema_folder_button, 86, 0, 1, 2);
        debug_grid.add_widget_5a(&debug_clear_layout_settings_button, 87, 0, 1, 2);

        settings_grid.add_widget_5a(&debug_frame, 2, 2, 1, 1);

        //-----------------------------------------------//
        // `Diagnostics` Frame.
        //-----------------------------------------------//
        let diagnostics_frame = QGroupBox::from_q_string_q_widget(&qtr("settings_diagnostics_title"), &dialog);
        let diagnostics_grid = create_grid_layout(diagnostics_frame.static_upcast());
        diagnostics_grid.set_contents_margins_4a(4, 0, 4, 0);
        diagnostics_grid.set_spacing(4);
        diagnostics_grid.set_row_stretch(80, 10);

        let diagnostics_diagnostics_trigger_on_open_label = QLabel::from_q_string_q_widget(&qtr("settings_diagnostics_trigger_on_open"), &diagnostics_frame);
        let diagnostics_diagnostics_trigger_on_table_edit_label = QLabel::from_q_string_q_widget(&qtr("settings_diagnostics_trigger_on_edit"), &diagnostics_frame);

        let diagnostics_diagnostics_trigger_on_open_checkbox = QCheckBox::from_q_widget(&diagnostics_frame);
        let diagnostics_diagnostics_trigger_on_table_edit_checkbox = QCheckBox::from_q_widget(&diagnostics_frame);

        diagnostics_grid.add_widget_5a(&diagnostics_diagnostics_trigger_on_open_label, 1, 0, 1, 1);
        diagnostics_grid.add_widget_5a(&diagnostics_diagnostics_trigger_on_open_checkbox, 1, 1, 1, 1);

        diagnostics_grid.add_widget_5a(&diagnostics_diagnostics_trigger_on_table_edit_label, 2, 0, 1, 1);
        diagnostics_grid.add_widget_5a(&diagnostics_diagnostics_trigger_on_table_edit_checkbox, 2, 1, 1, 1);

        settings_grid.add_widget_5a(&diagnostics_frame, 3, 2, 1, 1);

        //-----------------------------------------------//
        // `Warning` section.
        //-----------------------------------------------//
        let warning_frame = QGroupBox::from_q_widget(&dialog);
        let warning_grid = create_grid_layout(warning_frame.static_upcast());
        let warning_message = QLabel::from_q_string_q_widget(&qtr("settings_warning_message"), &warning_frame);
        warning_message.set_word_wrap(true);
        warning_message.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));

        warning_grid.add_widget_5a(&warning_message, 0, 0, 1, 1);
        settings_grid.add_widget_5a(&warning_frame, 3, 1, 1, 1);

        //-----------------------------------------------//
        // `ButtonBox` Button Box.
        //-----------------------------------------------//
        let button_box = QDialogButtonBox::from_q_widget(&dialog);
        let button_box_shortcuts_button = QPushButton::from_q_string_q_widget(&qtr("shortcut_title"), &button_box);
        let button_box_text_editor_settings_button = QPushButton::from_q_string_q_widget(&qtr("settings_text_title"), &button_box);
        let button_box_font_settings_button = QPushButton::from_q_string_q_widget(&qtr("settings_font_title"), &button_box);

        let button_box_restore_default_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::RestoreDefaults);
        button_box.add_button_q_abstract_button_button_role(&button_box_shortcuts_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&button_box_text_editor_settings_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&button_box_font_settings_button, ButtonRole::ResetRole);
        let button_box_cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let button_box_accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);

        main_grid.add_widget_5a(&button_box, 4, 0, 1, 3);

        // Now, we build the `SettingsUI` struct and return it.
        Self {

            //-------------------------------------------------------------------------------//
            // `Dialog` window.
            //-------------------------------------------------------------------------------//
            dialog,

            tab_widget,
            paths_tab,
            settings_tab,

            //-------------------------------------------------------------------------------//
            // `Path` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            paths_zip_label,
            paths_zip_line_edit,
            paths_zip_button,
            paths_mymod_label,
            paths_mymod_line_edit,
            paths_mymod_button,
            paths_spoilers,
            paths_games_line_edits,
            paths_games_buttons,
            paths_asskit_line_edits,
            paths_asskit_buttons,

            //-------------------------------------------------------------------------------//
            // `General` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            general_language_label,
            extra_global_default_game_label,
            extra_network_update_channel_label,
            extra_packfile_autosave_amount_label,
            extra_packfile_autosave_interval_label,
            extra_network_check_updates_on_start_label,
            extra_network_check_schema_updates_on_start_label,
            extra_packfile_allow_editing_of_ca_packfiles_label,
            extra_packfile_optimize_not_renamed_packedfiles_label,
            extra_packfile_use_lazy_loading_label,
            extra_packfile_disable_uuid_regeneration_on_db_tables_label,
            extra_packfile_disable_file_previews_label,
            ui_global_use_dark_theme_label,
            ui_window_start_maximized_label,
            ui_window_hide_background_icon_label,
            general_packfile_treeview_resize_to_fit_label,
            general_packfile_treeview_expand_treeview_when_adding_items_label,

            general_language_combobox,
            extra_global_default_game_combobox,
            extra_network_update_channel_combobox,
            extra_packfile_autosave_amount_spinbox,
            extra_packfile_autosave_interval_spinbox,
            extra_network_check_updates_on_start_checkbox,
            extra_network_check_schema_updates_on_start_checkbox,
            extra_packfile_allow_editing_of_ca_packfiles_checkbox,
            extra_packfile_optimize_not_renamed_packedfiles_checkbox,
            extra_packfile_use_lazy_loading_checkbox,
            extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox,
            extra_packfile_disable_file_previews_checkbox,
            ui_global_use_dark_theme_checkbox,
            ui_window_start_maximized_checkbox,
            ui_window_hide_background_icon_checkbox,
            general_packfile_treeview_resize_to_fit_checkbox,
            general_packfile_treeview_expand_treeview_when_adding_items_checkbox,

            //-------------------------------------------------------------------------------//
            // `Table` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            ui_table_adjust_columns_to_content_label,
            ui_table_disable_combos_label,
            ui_table_extend_last_column_label,
            ui_table_tight_table_mode_label,
            ui_table_resize_on_edit_label,
            ui_table_use_old_column_order_label,
            ui_table_use_right_size_markers_label,

            ui_table_adjust_columns_to_content_checkbox,
            ui_table_disable_combos_checkbox,
            ui_table_extend_last_column_checkbox,
            ui_table_tight_table_mode_checkbox,
            ui_table_resize_on_edit_checkbox,
            ui_table_use_old_column_order_checkbox,
            ui_table_use_right_size_markers_checkbox,

            ui_table_colour_table_added_label,
            ui_table_colour_table_modified_label,
            ui_table_colour_diagnostic_error_label,
            ui_table_colour_diagnostic_warning_label,
            ui_table_colour_diagnostic_info_label,

            ui_table_colour_light_table_added_button,
            ui_table_colour_light_table_modified_button,
            ui_table_colour_light_diagnostic_error_button,
            ui_table_colour_light_diagnostic_warning_button,
            ui_table_colour_light_diagnostic_info_button,
            ui_table_colour_dark_table_added_button,
            ui_table_colour_dark_table_modified_button,
            ui_table_colour_dark_diagnostic_error_button,
            ui_table_colour_dark_diagnostic_warning_button,
            ui_table_colour_dark_diagnostic_info_button,

            //-------------------------------------------------------------------------------//
            // `Debug` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            debug_check_for_missing_table_definitions_label,
            debug_check_for_missing_table_definitions_checkbox,
            debug_enable_debug_menu_label,
            debug_enable_debug_menu_checkbox,
            debug_spoof_ca_authoring_tool_label,
            debug_spoof_ca_authoring_tool_checkbox,
            debug_enable_rigidmodel_editor_label,
            debug_enable_rigidmodel_editor_checkbox,
            debug_enable_esf_editor_label,
            debug_enable_esf_editor_checkbox,
            debug_enable_unit_editor_label,
            debug_enable_unit_editor_checkbox,

            debug_clear_autosave_folder_button,
            debug_clear_schema_folder_button,
            debug_clear_layout_settings_button,

            //-------------------------------------------------------------------------------//
            // `Diagnostics` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            diagnostics_diagnostics_trigger_on_open_label,
            diagnostics_diagnostics_trigger_on_table_edit_label,

            diagnostics_diagnostics_trigger_on_open_checkbox,
            diagnostics_diagnostics_trigger_on_table_edit_checkbox,

            //-------------------------------------------------------------------------------//
            // `Warning` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            warning_message,

            //-------------------------------------------------------------------------------//
            // `ButtonBox` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            button_box_restore_default_button,
            button_box_text_editor_settings_button,
            button_box_shortcuts_button,
            button_box_font_settings_button,
            button_box_cancel_button,
            button_box_accept_button,
        }
    }

    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub unsafe fn load(&self, settings: &Settings) -> Result<()> {

        // Load the MyMod and 7Zip paths, if exists.
        self.paths_mymod_line_edit.set_text(&QString::from_std_str(settings.paths[MYMOD_BASE_PATH].clone().unwrap_or_else(PathBuf::new).to_string_lossy()));
        self.paths_zip_line_edit.set_text(&QString::from_std_str(settings.paths[ZIP_PATH].clone().unwrap_or_else(PathBuf::new).to_string_lossy()));

        // Load the Game Paths, if they exists.
        for (key, path) in self.paths_games_line_edits.iter() {
            if let Some(ref path_data) = settings.paths[key] {
                if let Some(spoiler) = self.paths_spoilers.get(key) {
                    path.set_text(&QString::from_std_str(&path_data.to_string_lossy()));
                    toggle_animated_safe(&spoiler.as_ptr());
                }
            }
        }

        for (key, path) in self.paths_asskit_line_edits.iter() {
            if let Some(ref path_data) = settings.paths[&(key.to_owned() + "_assembly_kit")] {
                path.set_text(&QString::from_std_str(&path_data.to_string_lossy()));
            }
        }

        // Get the default game.
        for (index, game) in SUPPORTED_GAMES.get_games().iter().enumerate() {
            if game.get_game_key_name() == settings.settings_string["default_game"] {
                self.extra_global_default_game_combobox.set_current_index(index as i32);
                break;
            }
        }

        let language_selected = settings.settings_string["language"].split('_').collect::<Vec<&str>>()[0];
        for (index, (language,_)) in Locale::get_available_locales()?.iter().enumerate() {
            if *language == language_selected {
                self.general_language_combobox.set_current_index(index as i32);
                break;
            }
        }

        for (index, update_channel_name) in [UpdateChannel::Stable, UpdateChannel::Beta].iter().enumerate() {
            if update_channel_name == &get_update_channel() {
                self.extra_network_update_channel_combobox.set_current_index(index as i32);
                break;
            }
        }

        // Load the General Stuff.
        self.extra_packfile_autosave_amount_spinbox.set_value(settings.settings_string["autosave_amount"].parse::<i32>().unwrap_or(10));
        self.extra_packfile_autosave_interval_spinbox.set_value(settings.settings_string["autosave_interval"].parse::<i32>().unwrap_or(10));
        self.ui_global_use_dark_theme_checkbox.set_checked(settings.settings_bool["use_dark_theme"]);
        self.ui_window_start_maximized_checkbox.set_checked(settings.settings_bool["start_maximized"]);
        self.ui_window_hide_background_icon_checkbox.set_checked(settings.settings_bool["hide_background_icon"]);
        self.extra_network_check_updates_on_start_checkbox.set_checked(settings.settings_bool["check_updates_on_start"]);
        self.extra_network_check_schema_updates_on_start_checkbox.set_checked(settings.settings_bool["check_schema_updates_on_start"]);
        self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.set_checked(settings.settings_bool["allow_editing_of_ca_packfiles"]);
        self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.set_checked(settings.settings_bool["optimize_not_renamed_packedfiles"]);
        self.extra_packfile_use_lazy_loading_checkbox.set_checked(settings.settings_bool["use_lazy_loading"]);
        self.extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox.set_checked(settings.settings_bool["disable_uuid_regeneration_on_db_tables"]);
        self.extra_packfile_disable_file_previews_checkbox.set_checked(settings.settings_bool["disable_file_previews"]);
        self.general_packfile_treeview_resize_to_fit_checkbox.set_checked(settings.settings_bool["packfile_treeview_resize_to_fit"]);
        self.general_packfile_treeview_expand_treeview_when_adding_items_checkbox.set_checked(settings.settings_bool["expand_treeview_when_adding_items"]);

        // Load the Table Stuff.
        self.ui_table_adjust_columns_to_content_checkbox.set_checked(settings.settings_bool["adjust_columns_to_content"]);
        self.ui_table_disable_combos_checkbox.set_checked(settings.settings_bool["disable_combos_on_tables"]);
        self.ui_table_extend_last_column_checkbox.set_checked(settings.settings_bool["extend_last_column_on_tables"]);
        self.ui_table_tight_table_mode_checkbox.set_checked(settings.settings_bool["tight_table_mode"]);
        self.ui_table_resize_on_edit_checkbox.set_checked(settings.settings_bool["table_resize_on_edit"]);
        self.ui_table_use_old_column_order_checkbox.set_checked(settings.settings_bool["tables_use_old_column_order"]);
        self.ui_table_use_right_size_markers_checkbox.set_checked(settings.settings_bool["use_right_size_markers"]);

        // Load colours.
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));

        let colour_light_table_added = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_table_added")).to_string());
        let colour_light_table_modified = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_table_modified")).to_string());
        let colour_light_diagnostic_error = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_error")).to_string());
        let colour_light_diagnostic_warning = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_warning")).to_string());
        let colour_light_diagnostic_info = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_light_diagnostic_info")).to_string());
        let colour_dark_table_added = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_table_added")).to_string());
        let colour_dark_table_modified = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_table_modified")).to_string());
        let colour_dark_diagnostic_error = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_error")).to_string());
        let colour_dark_diagnostic_warning = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_warning")).to_string());
        let colour_dark_diagnostic_info = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str("colour_dark_diagnostic_info")).to_string());

        self.ui_table_colour_light_table_added_button.set_palette(&QPalette::from_q_color(&colour_light_table_added));
        self.ui_table_colour_light_table_modified_button.set_palette(&QPalette::from_q_color(&colour_light_table_modified));
        self.ui_table_colour_light_diagnostic_error_button.set_palette(&QPalette::from_q_color(&colour_light_diagnostic_error));
        self.ui_table_colour_light_diagnostic_warning_button.set_palette(&QPalette::from_q_color(&colour_light_diagnostic_warning));
        self.ui_table_colour_light_diagnostic_info_button.set_palette(&QPalette::from_q_color(&colour_light_diagnostic_info));
        self.ui_table_colour_dark_table_added_button.set_palette(&QPalette::from_q_color(&colour_dark_table_added));
        self.ui_table_colour_dark_table_modified_button.set_palette(&QPalette::from_q_color(&colour_dark_table_modified));
        self.ui_table_colour_dark_diagnostic_error_button.set_palette(&QPalette::from_q_color(&colour_dark_diagnostic_error));
        self.ui_table_colour_dark_diagnostic_warning_button.set_palette(&QPalette::from_q_color(&colour_dark_diagnostic_warning));
        self.ui_table_colour_dark_diagnostic_info_button.set_palette(&QPalette::from_q_color(&colour_dark_diagnostic_info));

        // So, windows is fucking annoying when it wants, and here's an example. QPalette doesn't change the visual colour of buttons, only on windows.
        // The colour is there, but the button color will not change. So we have to set it, AGAIN, with stylesheets, only in fucking windows.
        if cfg!(target_os = "windows") {
            self.ui_table_colour_light_table_added_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_light_table_added.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_light_table_modified_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_light_table_modified.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_light_diagnostic_error_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_light_diagnostic_error.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_light_diagnostic_warning_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_light_diagnostic_warning.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_light_diagnostic_info_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_light_diagnostic_info.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_dark_table_added_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_dark_table_added.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_dark_table_modified_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_dark_table_modified.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_dark_diagnostic_error_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_dark_diagnostic_error.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_dark_diagnostic_warning_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_dark_diagnostic_warning.name_1a(NameFormat::HexArgb).to_std_string())));
            self.ui_table_colour_dark_diagnostic_info_button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", colour_dark_diagnostic_info.name_1a(NameFormat::HexArgb).to_std_string())));
        }

        // Load the Debug Stuff.
        self.debug_check_for_missing_table_definitions_checkbox.set_checked(settings.settings_bool["check_for_missing_table_definitions"]);
        self.debug_enable_debug_menu_checkbox.set_checked(settings.settings_bool["enable_debug_menu"]);
        self.debug_spoof_ca_authoring_tool_checkbox.set_checked(settings.settings_bool["spoof_ca_authoring_tool"]);
        self.debug_enable_rigidmodel_editor_checkbox.set_checked(settings.settings_bool["enable_rigidmodel_editor"]);
        self.debug_enable_esf_editor_checkbox.set_checked(settings.settings_bool["enable_esf_editor"]);
        self.debug_enable_unit_editor_checkbox.set_checked(settings.settings_bool["enable_unit_editor"]);

        // Load the Diagnostics Stuff.
        self.diagnostics_diagnostics_trigger_on_open_checkbox.set_checked(settings.settings_bool["diagnostics_trigger_on_open"]);
        self.diagnostics_diagnostics_trigger_on_table_edit_checkbox.set_checked(settings.settings_bool["diagnostics_trigger_on_table_edit"]);

        Ok(())
    }

    /// This function saves the data from our `SettingsUI` into a `Settings` and return it.
    pub unsafe fn save(&self) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new();

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let mymod_new_path = PathBuf::from(self.paths_mymod_line_edit.text().to_std_string());
        settings.paths.insert(MYMOD_BASE_PATH.to_owned(), if mymod_new_path.is_dir() { Some(mymod_new_path) } else { None });

        let zip_new_path = PathBuf::from(self.paths_zip_line_edit.text().to_std_string());
        settings.paths.insert(ZIP_PATH.to_owned(), if zip_new_path.is_file() { Some(zip_new_path) } else { None });

        // For each entry, we check if it's a valid directory and save it into Settings.
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let new_path = PathBuf::from(line_edit.text().to_std_string());
            settings.paths.insert(key.to_owned(), if new_path.is_dir() { Some(new_path) } else { None });
        }

        for (key, line_edit) in self.paths_asskit_line_edits.iter() {
            let new_path = PathBuf::from(line_edit.text().to_std_string());
            settings.paths.insert(key.to_owned() + "_assembly_kit", if new_path.is_dir() { Some(new_path) } else { None });
        }

        // We get his game's folder, depending on the selected game.
        let mut game = self.extra_global_default_game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        settings.settings_string.insert("default_game".to_owned(), game);

        // We need to store the full locale filename, not just the visible name!
        let mut language = self.general_language_combobox.current_text().to_std_string();
        if let Some(index) = language.find('&') { language.remove(index); }
        if let Some((_, locale)) = Locale::get_available_locales().unwrap().iter().find(|(x, _)| &language == x) {
            let file_name = format!("{}_{}", language, locale.language);
            settings.settings_string.insert("language".to_owned(), file_name);
        }

        let update_channel = self.extra_network_update_channel_combobox.current_text().to_std_string();
        settings.settings_string.insert("update_channel".to_owned(), update_channel);

        let current_font = QGuiApplication::font();
        settings.settings_string.insert("font_name".to_owned(), current_font.family().to_std_string());
        settings.settings_string.insert("font_size".to_owned(), current_font.point_size().to_string());

        // Get the General Settings.
        settings.settings_string.insert("autosave_amount".to_owned(), self.extra_packfile_autosave_amount_spinbox.value().to_string());
        settings.settings_string.insert("autosave_interval".to_owned(), self.extra_packfile_autosave_interval_spinbox.value().to_string());
        settings.settings_bool.insert("use_dark_theme".to_owned(), self.ui_global_use_dark_theme_checkbox.is_checked());
        settings.settings_bool.insert("start_maximized".to_owned(), self.ui_window_start_maximized_checkbox.is_checked());
        settings.settings_bool.insert("hide_background_icon".to_owned(), self.ui_window_hide_background_icon_checkbox.is_checked());
        settings.settings_bool.insert("check_updates_on_start".to_owned(), self.extra_network_check_updates_on_start_checkbox.is_checked());
        settings.settings_bool.insert("check_schema_updates_on_start".to_owned(), self.extra_network_check_schema_updates_on_start_checkbox.is_checked());
        settings.settings_bool.insert("allow_editing_of_ca_packfiles".to_owned(), self.extra_packfile_allow_editing_of_ca_packfiles_checkbox.is_checked());
        settings.settings_bool.insert("optimize_not_renamed_packedfiles".to_owned(), self.extra_packfile_optimize_not_renamed_packedfiles_checkbox.is_checked());
        settings.settings_bool.insert("use_lazy_loading".to_owned(), self.extra_packfile_use_lazy_loading_checkbox.is_checked());
        settings.settings_bool.insert("disable_uuid_regeneration_on_db_tables".to_owned(), self.extra_packfile_disable_uuid_regeneration_on_db_tables_checkbox.is_checked());
        settings.settings_bool.insert("disable_file_previews".to_owned(), self.extra_packfile_disable_file_previews_checkbox.is_checked());
        settings.settings_bool.insert("packfile_treeview_resize_to_fit".to_owned(), self.general_packfile_treeview_resize_to_fit_checkbox.is_checked());
        settings.settings_bool.insert("expand_treeview_when_adding_items".to_owned(), self.general_packfile_treeview_expand_treeview_when_adding_items_checkbox.is_checked());

        // Get the Table Settings.
        settings.settings_bool.insert("adjust_columns_to_content".to_owned(), self.ui_table_adjust_columns_to_content_checkbox.is_checked());
        settings.settings_bool.insert("disable_combos_on_tables".to_owned(), self.ui_table_disable_combos_checkbox.is_checked());
        settings.settings_bool.insert("extend_last_column_on_tables".to_owned(), self.ui_table_extend_last_column_checkbox.is_checked());
        settings.settings_bool.insert("tight_table_mode".to_owned(), self.ui_table_tight_table_mode_checkbox.is_checked());
        settings.settings_bool.insert("table_resize_on_edit".to_owned(), self.ui_table_resize_on_edit_checkbox.is_checked());
        settings.settings_bool.insert("tables_use_old_column_order".to_owned(), self.ui_table_use_old_column_order_checkbox.is_checked());
        settings.settings_bool.insert("use_right_size_markers".to_owned(), self.ui_table_use_right_size_markers_checkbox.is_checked());

        // Get the colours high.
        let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));

        q_settings.set_value(&QString::from_std_str("colour_light_table_added"), &QVariant::from_q_string(&self.ui_table_colour_light_table_added_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_light_table_modified"), &QVariant::from_q_string(&self.ui_table_colour_light_table_modified_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_error"), &QVariant::from_q_string(&self.ui_table_colour_light_diagnostic_error_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_warning"), &QVariant::from_q_string(&self.ui_table_colour_light_diagnostic_warning_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_light_diagnostic_info"), &QVariant::from_q_string(&self.ui_table_colour_light_diagnostic_info_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_dark_table_added"), &QVariant::from_q_string(&self.ui_table_colour_dark_table_added_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_dark_table_modified"), &QVariant::from_q_string(&self.ui_table_colour_dark_table_modified_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_error"), &QVariant::from_q_string(&self.ui_table_colour_dark_diagnostic_error_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_warning"), &QVariant::from_q_string(&self.ui_table_colour_dark_diagnostic_warning_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));
        q_settings.set_value(&QString::from_std_str("colour_dark_diagnostic_info"), &QVariant::from_q_string(&self.ui_table_colour_dark_diagnostic_info_button.palette().color_1a(ColorRole::Background).name_1a(NameFormat::HexArgb)));

        q_settings.sync();

        // Get the Debug Settings.
        settings.settings_bool.insert("check_for_missing_table_definitions".to_owned(), self.debug_check_for_missing_table_definitions_checkbox.is_checked());
        settings.settings_bool.insert("enable_debug_menu".to_owned(), self.debug_enable_debug_menu_checkbox.is_checked());
        settings.settings_bool.insert("spoof_ca_authoring_tool".to_owned(), self.debug_spoof_ca_authoring_tool_checkbox.is_checked());
        settings.settings_bool.insert("enable_rigidmodel_editor".to_owned(), self.debug_enable_rigidmodel_editor_checkbox.is_checked());
        settings.settings_bool.insert("enable_esf_editor".to_owned(), self.debug_enable_esf_editor_checkbox.is_checked());
        settings.settings_bool.insert("enable_unit_editor".to_owned(), self.debug_enable_unit_editor_checkbox.is_checked());

        // Get the Diagnostics Settings.
        settings.settings_bool.insert("diagnostics_trigger_on_open".to_owned(), self.diagnostics_diagnostics_trigger_on_open_checkbox.is_checked());
        settings.settings_bool.insert("diagnostics_trigger_on_table_edit".to_owned(), self.diagnostics_diagnostics_trigger_on_table_edit_checkbox.is_checked());

        settings
    }

    /// This function updates the path you have for the provided game (or mymod, if you pass it `None`)
    /// with the one you select in a `FileDialog`.
    unsafe fn update_entry_path(&self, game: &str, is_asskit_path: bool) {

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let (line_edit, is_file) = if is_asskit_path {
            match self.paths_asskit_line_edits.get(game) {
                Some(line_edit) => (line_edit, false),
                None => match game {
                    MYMOD_BASE_PATH => (&self.paths_mymod_line_edit, false),
                    ZIP_PATH => (&self.paths_zip_line_edit, true),
                    _ => return,
                }
            }
        } else {
            match self.paths_games_line_edits.get(game) {
                Some(line_edit) => (line_edit, false),
                None => match game {
                    MYMOD_BASE_PATH => (&self.paths_mymod_line_edit, false),
                    ZIP_PATH => (&self.paths_zip_line_edit, true),
                    _ => return,
                }
            }
        };

        // Create the `FileDialog` and configure it.
        let title = if is_file { qtr("settings_select_file") } else { qtr("settings_select_folder") };
        let file_dialog = QFileDialog::from_q_widget_q_string(
            &self.dialog,
            &title,
        );

        if !is_file {
            file_dialog.set_file_mode(FileMode::Directory);
            file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));
        }

        // Get the old Path, if exists.
        let old_path = line_edit.text().to_std_string();

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            file_dialog.set_directory_q_string(&line_edit.text());
        }

        // If the game path is set and the assembly kit path it isn't, use the game path to begin with.
        else if is_asskit_path {
            if let Some(line_edit) = self.paths_games_line_edits.get(game) {
                let old_game_path = line_edit.text().to_std_string();
                if !old_game_path.is_empty() && Path::new(&old_game_path).is_dir() {
                    file_dialog.set_directory_q_string(&line_edit.text());
                }
            }
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
