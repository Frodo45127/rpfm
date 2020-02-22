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
Module with all the code to setup shortcuts for `AppUI`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use super::AppUI;
use crate::locale::qtr;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `AppUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub fn set_shortcuts(app_ui: &mut AppUI) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Menu Bar actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    unsafe { app_ui.packfile_new_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["new_packfile"]))); }
    unsafe { app_ui.packfile_open_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["open_packfile"]))); }
    unsafe { app_ui.packfile_save_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile"]))); }
    unsafe { app_ui.packfile_save_packfile_as.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile_as"]))); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["load_all_ca_packfiles"]))); }
    unsafe { app_ui.packfile_preferences.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["preferences"]))); }
    unsafe { app_ui.packfile_quit.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["quit"]))); }

    unsafe { app_ui.game_selected_open_game_data_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_data_folder"]))); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_assembly_kit_folder"]))); }

    unsafe { app_ui.about_about_qt.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_qt"]))); }
    unsafe { app_ui.about_about_rpfm.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_rpfm"]))); }
    unsafe { app_ui.about_open_manual.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["open_manual"]))); }
    unsafe { app_ui.about_check_updates.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_updates"]))); }
    unsafe { app_ui.about_check_schema_updates.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_schema_updates"]))); }

    // Set the shortcuts to only trigger in the TreeView.
    unsafe { app_ui.packfile_new_packfile.set_shortcut_context(ShortcutContext:: ApplicationShortcut); }
    unsafe { app_ui.packfile_open_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.packfile_save_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.packfile_save_packfile_as.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.packfile_preferences.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.packfile_quit.set_shortcut_context(ShortcutContext::ApplicationShortcut); }

    unsafe { app_ui.game_selected_open_game_data_folder.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.set_shortcut_context(ShortcutContext::ApplicationShortcut); }

    unsafe { app_ui.about_about_qt.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.about_about_rpfm.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.about_open_manual.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.about_check_updates.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.about_check_schema_updates.set_shortcut_context(ShortcutContext::ApplicationShortcut); }

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Command Palette...
    //---------------------------------------------------------------------------------------//

    unsafe { app_ui.command_palette_show.set_shortcut(&QKeySequence::from_q_string(&qtr("shortcut_csp"))); }
    unsafe { app_ui.command_palette_hide.set_shortcut(&QKeySequence::from_q_string(&qtr("shortcut_esc"))); }

    unsafe { app_ui.command_palette_show.set_shortcut_context(ShortcutContext::ApplicationShortcut); }
    unsafe { app_ui.command_palette_hide.set_shortcut_context(ShortcutContext::ApplicationShortcut); }

    unsafe { app_ui.main_window.add_action(app_ui.command_palette_show); }
    unsafe { app_ui.main_window.add_action(app_ui.command_palette_hide); }
}
