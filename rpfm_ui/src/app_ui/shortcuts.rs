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
Module with all the code to setup shortcuts for `AppUI`.
!*/

use qt_gui::key_sequence::KeySequence;

use qt_core::qt::ShortcutContext;

use super::AppUI;
use crate::QString;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `AppUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub fn set_shortcuts(app_ui: &AppUI) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Menu Bar actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    unsafe { app_ui.packfile_new_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["new_packfile"]))); }
    unsafe { app_ui.packfile_open_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["open_packfile"]))); }
    unsafe { app_ui.packfile_save_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile"]))); }
    unsafe { app_ui.packfile_save_packfile_as.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile_as"]))); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["load_all_ca_packfiles"]))); }
    unsafe { app_ui.packfile_preferences.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["preferences"]))); }
    unsafe { app_ui.packfile_quit.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["quit"]))); }

    unsafe { app_ui.game_selected_open_game_data_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_data_folder"]))); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_assembly_kit_folder"]))); }

    unsafe { app_ui.about_about_qt.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_qt"]))); }
    unsafe { app_ui.about_about_rpfm.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_rpfm"]))); }
    unsafe { app_ui.about_open_manual.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_about["open_manual"]))); }
    unsafe { app_ui.about_check_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_updates"]))); }
    unsafe { app_ui.about_check_schema_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_schema_updates"]))); }

    // Set the shortcuts to only trigger in the TreeView.
    unsafe { app_ui.packfile_new_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_open_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_save_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_save_packfile_as.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_preferences.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.packfile_quit.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

    unsafe { app_ui.game_selected_open_game_data_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

    unsafe { app_ui.about_about_qt.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.about_about_rpfm.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.about_open_manual.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.about_check_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.about_check_schema_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Command Palette...
    //---------------------------------------------------------------------------------------//

    unsafe { app_ui.command_palette_show.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("Ctrl+Shift+P"))); }
    unsafe { app_ui.command_palette_hide.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("Esc"))); }

    unsafe { app_ui.command_palette_show.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
    unsafe { app_ui.command_palette_hide.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

    unsafe { app_ui.main_window.as_mut().unwrap().add_action(app_ui.command_palette_show); }
    unsafe { app_ui.main_window.as_mut().unwrap().add_action(app_ui.command_palette_hide); }
}
