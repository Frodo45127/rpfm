//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QColorDialog;
use qt_widgets::QFontDialog;
use qt_widgets::QPushButton;
use qt_widgets::QWidget;

use qt_gui::{QPalette, q_palette::ColorRole};
use qt_gui::q_color::NameFormat;
use qt_gui::QGuiApplication;
use qt_gui::QFontDatabase;
use qt_gui::q_font_database::SystemFont;

use qt_core::QBox;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::SlotNoArgs;

use std::collections::BTreeMap;
use std::fs::remove_dir_all;
use std::rc::Rc;
use std::process::Command as SystemCommand;

use rpfm_lib::common::*;
use rpfm_lib::settings::{init_config_path, Settings, MYMOD_BASE_PATH, ZIP_PATH};

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi;
use crate::locale::tr;
use crate::QT_PROGRAM;
use crate::QT_ORG;
use crate::settings_ui::SettingsUI;
use crate::shortcuts_ui::ShortcutsUI;
use crate::UI_STATE;
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

    pub select_colour_light_local_tip: QBox<SlotNoArgs>,
    pub select_colour_light_remote_tip: QBox<SlotNoArgs>,
    pub select_colour_dark_local_tip: QBox<SlotNoArgs>,
    pub select_colour_dark_remote_tip: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUISlots`.
impl SettingsUISlots {

    /// This function creates a new `SettingsUISlots`.
    pub unsafe fn new(ui: &Rc<SettingsUI>, app_ui: &Rc<AppUI>) -> Self {

        // What happens when we hit thr "Restore Default" button.
        let restore_default = SlotNoArgs::new(&ui.dialog, clone!(
            app_ui,
            ui => move || {

                // Restore RPFM settings.
                if let Err(error) = ui.load(&Settings::new()) {
                    return show_dialog(&ui.dialog, error, false);
                }

                // Restore layout settings.
                let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));
                app_ui.main_window.restore_geometry(&q_settings.value_1a(&QString::from_std_str("originalGeometry")).to_byte_array());
                app_ui.main_window.restore_state_1a(&q_settings.value_1a(&QString::from_std_str("originalWindowState")).to_byte_array());
                q_settings.sync();

                QGuiApplication::set_font(&QFontDatabase::system_font(SystemFont::GeneralFont));
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
        let shortcuts = SlotNoArgs::new(&ui.dialog, clone!(ui => move || {

            // Create the Shortcuts Dialog. If we got new shortcuts, try to save them and report any error.
            if let Some(shortcuts) = ShortcutsUI::new(&ui.dialog) {
                let receiver = CENTRAL_COMMAND.send_background(Command::SetShortcuts(shortcuts.clone()));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::Success => UI_STATE.set_shortcuts(&shortcuts),
                    Response::Error(error) => show_dialog(&ui.dialog, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        }));

        // What happens when we hit the "Text Editor Preferences" button.
        let text_editor = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            ffi::open_text_editor_config_safe(&ui.dialog.static_upcast::<QWidget>().as_ptr());
        }));

        let font_settings = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            let font_changed: *mut bool = &mut false;
            let current_font = QGuiApplication::font();
            let new_font = QFontDialog::get_font_bool_q_font_q_widget(font_changed, current_font.as_ref(), &ui.dialog);
            if *font_changed {
                QGuiApplication::set_font(new_font.as_ref());
            }
        }));

        let clear_dependencies_cache = SlotNoArgs::new(&ui.dialog, clone!(mut ui => move || {
            match get_dependencies_cache_path() {
                Ok(path) => match remove_dir_all(&path) {
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
            match get_backup_autosave_path() {
                Ok(path) => match remove_dir_all(&path) {
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
            match get_schemas_path() {
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
                let q_settings = QSettings::from_2_q_string(&QString::from_std_str(QT_ORG), &QString::from_std_str(QT_PROGRAM));
                app_ui.main_window.restore_geometry(&q_settings.value_1a(&QString::from_std_str("originalGeometry")).to_byte_array());
                app_ui.main_window.restore_state_1a(&q_settings.value_1a(&QString::from_std_str("originalWindowState")).to_byte_array());
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

        let select_colour_light_local_tip = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.debug_colour_light_local_tip_button);
        }));

        let select_colour_light_remote_tip = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.debug_colour_light_remote_tip_button);
        }));

        let select_colour_dark_local_tip = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.debug_colour_dark_local_tip_button);
        }));

        let select_colour_dark_remote_tip = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
                change_colour(&ui.debug_colour_dark_remote_tip_button);
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

            select_colour_light_local_tip,
            select_colour_light_remote_tip,
            select_colour_dark_local_tip,
            select_colour_dark_remote_tip,
		}
	}
}

/// This function updates the colour of a colour button if needed.
unsafe fn change_colour(button: &QBox<QPushButton>) {
    let color = QColorDialog::get_color_1a(button.palette().color_1a(ColorRole::Background));
    if color.is_valid() {
        let palette = QPalette::from_q_color(&color);
        button.set_palette(&palette);

        if cfg!(target_os = "windows") {
            button.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", color.name_1a(NameFormat::HexArgb).to_std_string())));
        }
    }
}
