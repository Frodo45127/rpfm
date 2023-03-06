//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to `SettingsUISlots`.
!*/

use qt_widgets::QApplication;
use qt_widgets::QColorDialog;
use qt_widgets::QFontDialog;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_gui::{QPalette, q_palette::ColorRole};
use qt_gui::q_color::NameFormat;

use qt_core::QBox;
use qt_core::QString;
use qt_core::SlotNoArgs;

use std::collections::{BTreeMap, HashMap};
use std::fs::remove_dir_all;
use std::rc::Rc;
use std::process::Command as SystemCommand;

use rpfm_ui_common::locale::tr;

use crate::app_ui::AppUI;
use crate::ffi;
use crate::settings_ui::{backend::*, SettingsUI};
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SettingsUI` struct.
///
/// This means everything you can do with the stuff you have in the `SettingsUI` goes here.
pub struct SettingsUISlots {
    pub restore_default: QBox<SlotNoArgs>,
    pub select_mymod_path: QBox<SlotNoArgs>,
    pub select_zip_path: QBox<SlotNoArgs>,
    pub select_game_paths: BTreeMap<String, QBox<SlotNoArgs>>,
    pub select_asskit_paths: BTreeMap<String, QBox<SlotNoArgs>>,
    pub shortcuts: QBox<SlotNoArgs>,
    pub text_editor: QBox<SlotNoArgs>,
    pub font_settings: QBox<SlotNoArgs>,
    pub clear_dependencies_cache: QBox<SlotNoArgs>,
    pub clear_autosaves: QBox<SlotNoArgs>,
    pub clear_schemas: QBox<SlotNoArgs>,
    pub clear_layout: QBox<SlotNoArgs>,

    pub select_colour_light_table_added: QBox<SlotNoArgs>,
    pub select_colour_light_table_modified: QBox<SlotNoArgs>,
    pub select_colour_light_diagnostic_error: QBox<SlotNoArgs>,
    pub select_colour_light_diagnostic_warning: QBox<SlotNoArgs>,
    pub select_colour_light_diagnostic_info: QBox<SlotNoArgs>,
    pub select_colour_dark_table_added: QBox<SlotNoArgs>,
    pub select_colour_dark_table_modified: QBox<SlotNoArgs>,
    pub select_colour_dark_diagnostic_error: QBox<SlotNoArgs>,
    pub select_colour_dark_diagnostic_warning: QBox<SlotNoArgs>,
    pub select_colour_dark_diagnostic_info: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUISlots`.
impl SettingsUISlots {

    /// This function creates a new `SettingsUISlots`.
    pub unsafe fn new(ui: &Rc<SettingsUI>, app_ui: &Rc<AppUI>) -> Self {

        // What happens when we hit the "Restore Default" button.
        let restore_default = SlotNoArgs::new(&ui.dialog, clone!(
            app_ui,
            ui => move || {

                // Restore RPFM settings and reload the view, WITHOUT SAVING THE SETTINGS.
                // An exception are the original states. We need to keep those.
                let q_settings = settings();
                let keys = q_settings.all_keys();

                let mut old_settings = HashMap::new();
                for i in 0..keys.count_0a() {
                    old_settings.insert(keys.at(i).to_std_string(), setting_from_q_setting_variant(&q_settings, &keys.at(i).to_std_string()));
                }

                // Fonts are a bit special. Init picks them up from the running app, not from a fixed value,
                // so we need to manually overwrite them here before init_settings gets triggered.
                let original_font_name = setting_string("original_font_name");
                let original_font_size = setting_int("original_font_size");

                q_settings.clear();

                set_setting_string_to_q_setting(&q_settings, "font_name", &original_font_name);
                set_setting_int_to_q_setting(&q_settings, "font_size", original_font_size);

                q_settings.sync();

                init_settings(&app_ui.main_window().static_upcast());
                if let Err(error) = ui.load() {
                    return show_dialog(&ui.dialog, error, false);
                }

                // Once the original settings are reloaded, wipe them out from the backend again and put the old ones in.
                // That way, if the user cancels, we still have the old settings.
                q_settings.clear();
                q_settings.sync();

                for (key, value) in &old_settings {
                    set_setting_variant_to_q_setting(&q_settings, key, value.as_ref());
                }

                // Set this value to indicate future operations that a reset has taken place.
                set_setting_bool_to_q_setting(&q_settings, "factoryReset", true);

                // Save the backend settings again.
                q_settings.sync();
            }
        ));

        // What happens when we hit the "..." button for MyMods.
        let select_mymod_path = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                ui.update_entry_path(MYMOD_BASE_PATH, false);
            }
        ));

        // What happens when we hit the "..." button for 7Zip.
        let select_zip_path = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
            ui.update_entry_path(ZIP_PATH, false);
        }));

        // What happens when we hit any of the "..." buttons for the games.
        let mut select_game_paths = BTreeMap::new();
        for key in ui.paths_games_line_edits.keys() {
            select_game_paths.insert(
                key.to_owned(),
                SlotNoArgs::new(&ui.dialog, clone!(
                    key,
                    ui => move || {
                    ui.update_entry_path(&key, false);
                }))
            );
        }

        // What happens when we hit any of the "..." buttons for the asskits.
        let mut select_asskit_paths = BTreeMap::new();
        for key in ui.paths_asskit_line_edits.keys() {
            select_asskit_paths.insert(
                key.to_owned(),
                SlotNoArgs::new(&ui.dialog, clone!(
                    key,
                    ui => move || {
                    ui.update_entry_path(&key, true);
                }))
            );
        }

        // What happens when we hit the "Shortcuts" button.
        let shortcuts = SlotNoArgs::new(&ui.dialog, clone!(
            app_ui,
            ui => move || {
                crate::ffi::kshortcut_dialog_init_safe(&ui.dialog.static_upcast::<QWidget>().as_ptr(), app_ui.shortcuts().as_ptr());
        }));

        // What happens when we hit the "Text Editor Preferences" button.
        let text_editor = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            ffi::open_text_editor_config_safe(&ui.dialog.static_upcast::<QWidget>().as_ptr());
        }));

        let font_settings = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            let font_changed: *mut bool = &mut false;
            let current_font = QApplication::font();
            let new_font = QFontDialog::get_font_bool_q_font_q_widget(font_changed, current_font.as_ref(), &ui.dialog);
            if *font_changed {
                *ui.font_data.borrow_mut() = (new_font.family().to_std_string(), new_font.point_size());
            }
        }));

        let clear_dependencies_cache = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            match dependencies_cache_path() {
                Ok(path) => match remove_dir_all(path) {
                    Ok(_) => {
                        let _ = init_config_path();
                        show_dialog(&ui.dialog, tr("dependencies_cache_cleared"), true);
                    }
                    Err(error) => show_dialog(&ui.dialog, error, false),
                }
                Err(error) => show_dialog(&ui.dialog, error, false)
            }
        }));

        let clear_autosaves = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            match backup_autosave_path() {
                Ok(path) => match remove_dir_all(path) {
                    Ok(_) => {
                        let _ = init_config_path();
                        show_dialog(&ui.dialog, tr("autosaves_cleared"), true);
                    }
                    Err(error) => show_dialog(&ui.dialog, error, false),
                }
                Err(error) => show_dialog(&ui.dialog, error, false)
            }
        }));

        let clear_schemas = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            match schemas_path() {
                Ok(path) => {

                    // On windows, remove the read-only flags before doing anything else, or this will fail.
                    if cfg!(target_os = "windows") {
                        let path = path.to_string_lossy().to_string() + "\\*.*";
                        let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                    }
                    match remove_dir_all(&path) {
                        Ok(_) => {
                            let _ = init_config_path();
                            show_dialog(&ui.dialog, tr("schemas_cleared"), true);
                        }
                        Err(error) => show_dialog(&ui.dialog, error, false),
                    }
                }
                Err(error) => show_dialog(&ui.dialog, error, false)
            }
        }));

        let clear_layout = SlotNoArgs::new(&ui.dialog, clone!(
            app_ui => move || {
                let q_settings = settings();
                app_ui.main_window().restore_geometry(&q_settings.value_1a(&QString::from_std_str("originalGeometry")).to_byte_array());
                app_ui.main_window().restore_state_1a(&q_settings.value_1a(&QString::from_std_str("originalWindowState")).to_byte_array());
                q_settings.sync();
        }));

        let select_colour_light_table_added = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_light_table_added_button);
        }));

        let select_colour_light_table_modified = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_light_table_modified_button);
        }));

        let select_colour_light_diagnostic_error = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_light_diagnostic_error_button);
        }));

        let select_colour_light_diagnostic_warning = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_light_diagnostic_warning_button);
        }));

        let select_colour_light_diagnostic_info = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_light_diagnostic_info_button);
        }));

        let select_colour_dark_table_added = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_dark_table_added_button);
        }));

        let select_colour_dark_table_modified = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_dark_table_modified_button);
        }));

        let select_colour_dark_diagnostic_error = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_dark_diagnostic_error_button);
        }));

        let select_colour_dark_diagnostic_warning = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_dark_diagnostic_warning_button);
        }));

        let select_colour_dark_diagnostic_info = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.ui_table_colour_dark_diagnostic_info_button);
        }));

        // And here... we return all the slots.
		Self {
            restore_default,
            select_mymod_path,
            select_zip_path,
            select_game_paths,
            select_asskit_paths,
            shortcuts,
            text_editor,
            font_settings,
            clear_dependencies_cache,
            clear_autosaves,
            clear_schemas,
            clear_layout,
            select_colour_light_table_added,
            select_colour_light_table_modified,
            select_colour_light_diagnostic_error,
            select_colour_light_diagnostic_warning,
            select_colour_light_diagnostic_info,
            select_colour_dark_table_added,
            select_colour_dark_table_modified,
            select_colour_dark_diagnostic_error,
            select_colour_dark_diagnostic_warning,
            select_colour_dark_diagnostic_info,
		}
	}
}

/// This function updates the colour of a colour button if needed.
unsafe fn change_colour(button: &QBox<QPushButton>) {
    let color = QColorDialog::get_color_1a(button.palette().color_1a(ColorRole::Background));
    if color.is_valid() {
        let palette = QPalette::from_q_color(&color);
        button.set_palette(&palette);
        button.set_style_sheet(&QString::from_std_str(format!("background-color: {}", color.name_1a(NameFormat::HexArgb).to_std_string())));
    }
}
