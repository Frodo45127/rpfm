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
Module with all the code to setup shortcuts for `AppUI`.
!*/

use qt_gui::QKeySequence;

use qt_core::ShortcutContext;
use qt_core::QString;

use std::rc::Rc;

use super::AppUI;
use crate::UI_STATE;

/// This function setup all the shortcuts used by the actions in the provided `AppUI` .
///
/// This function is just glue to trigger after initializing the actions. It's here to not fill the other module with a ton of shortcuts.
pub unsafe fn set_shortcuts(app_ui: &Rc<AppUI>) {
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the Menu Bar actions...
    //---------------------------------------------------------------------------------------//

    // Set the shortcuts for these actions.
    app_ui.packfile_new_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["new_packfile"])));
    app_ui.packfile_open_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["open_packfile"])));
    app_ui.packfile_save_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile"])));
    app_ui.packfile_save_packfile_as.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["save_packfile_as"])));
    app_ui.packfile_install.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["packfile_install"])));
    app_ui.packfile_uninstall.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["packfile_uninstall"])));
    app_ui.packfile_load_all_ca_packfiles.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["load_all_ca_packfiles"])));
    app_ui.packfile_preferences.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["preferences"])));
    app_ui.packfile_quit.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_packfile["quit"])));

    app_ui.mymod_new.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_mymod["mymod_new"])));
    app_ui.mymod_delete_selected.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_mymod["mymod_delete_selected"])));
    app_ui.mymod_import.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_mymod["mymod_import"])));
    app_ui.mymod_export.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_mymod["mymod_export"])));

    app_ui.view_toggle_packfile_contents.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_view["view_toggle_packfile_contents"])));
    app_ui.view_toggle_global_search_panel.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_view["view_toggle_global_search_panel"])));
    app_ui.view_toggle_diagnostics_panel.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_view["view_toggle_diagnostics_panel"])));
    app_ui.view_toggle_dependencies_panel.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_view["view_toggle_dependencies_panel"])));
    app_ui.view_toggle_references_panel.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_view["view_toggle_references_panel"])));

    app_ui.game_selected_launch_game.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["launch_game"])));
    app_ui.game_selected_open_game_data_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_data_folder"])));
    app_ui.game_selected_open_game_assembly_kit_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_game_assembly_kit_folder"])));
    app_ui.game_selected_open_config_folder.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_game_selected["open_config_folder"])));

    app_ui.special_stuff_three_k_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_three_k_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_wh2_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_wh2_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));
    app_ui.special_stuff_wh2_patch_siege_ai.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["patch_siege_ai"])));

    app_ui.special_stuff_wh_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_wh_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));
    app_ui.special_stuff_wh_patch_siege_ai.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["patch_siege_ai"])));

    app_ui.special_stuff_tob_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_tob_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_att_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_att_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_rom2_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_rom2_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_sho2_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_sho2_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_nap_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_nap_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.special_stuff_emp_generate_dependencies_cache.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["generate_dependencies_cache"])));
    app_ui.special_stuff_emp_optimize_packfile.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_special_stuff["optimize_packfile"])));

    app_ui.about_about_qt.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_qt"])));
    app_ui.about_about_rpfm.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["about_rpfm"])));
    app_ui.about_open_manual.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["open_manual"])));
    app_ui.about_patreon_link.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["support_me_on_patreon"])));
    app_ui.about_check_updates.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_updates"])));
    app_ui.about_check_schema_updates.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.menu_bar_about["check_schema_updates"])));

    app_ui.packfile_new_packfile.set_shortcut_context(ShortcutContext:: ApplicationShortcut);
    app_ui.packfile_open_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_save_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_save_packfile_as.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_install.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_uninstall.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_load_all_ca_packfiles.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_preferences.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.packfile_quit.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.mymod_new.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.mymod_delete_selected.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.mymod_import.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.mymod_export.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.view_toggle_packfile_contents.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.view_toggle_global_search_panel.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.view_toggle_diagnostics_panel.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.view_toggle_dependencies_panel.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.view_toggle_references_panel.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.game_selected_launch_game.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.game_selected_open_game_data_folder.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.game_selected_open_game_assembly_kit_folder.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.game_selected_open_config_folder.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_three_k_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_three_k_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_wh2_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_wh2_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_wh2_patch_siege_ai.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_wh_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_wh_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_wh_patch_siege_ai.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_tob_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_tob_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_att_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_att_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_rom2_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_rom2_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_sho2_generate_dependencies_cache.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_sho2_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.special_stuff_nap_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.special_stuff_emp_optimize_packfile.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.about_about_qt.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.about_about_rpfm.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.about_open_manual.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.about_patreon_link.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.about_check_updates.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.about_check_schema_updates.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    //---------------------------------------------------------------------------------------//
    // Shortcuts for the PackedFile Views...
    //---------------------------------------------------------------------------------------//
    app_ui.tab_bar_packed_file_close.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_close_tab"])));
    app_ui.tab_bar_packed_file_close_all.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_close_tab_all"])));
    app_ui.tab_bar_packed_file_close_all_left.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_close_tab_all_left"])));
    app_ui.tab_bar_packed_file_close_all_right.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_close_tab_all_right"])));
    app_ui.tab_bar_packed_file_prev.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_tab_prev"])));
    app_ui.tab_bar_packed_file_next.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_tab_next"])));
    app_ui.tab_bar_packed_file_import_from_dependencies.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_import_from_dependencies"])));
    app_ui.tab_bar_packed_file_toggle_tips.set_shortcut(&QKeySequence::from_q_string(&QString::from_std_str(&shortcuts.packed_file_table["shortcut_toggle_tips"])));

    app_ui.tab_bar_packed_file_close.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_close_all.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_close_all_left.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_close_all_right.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_prev.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_next.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_import_from_dependencies.set_shortcut_context(ShortcutContext::ApplicationShortcut);
    app_ui.tab_bar_packed_file_toggle_tips.set_shortcut_context(ShortcutContext::ApplicationShortcut);

    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_close);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_close_all);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_close_all_left);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_close_all_right);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_prev);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_next);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_import_from_dependencies);
    app_ui.tab_bar_packed_file.add_action(&app_ui.tab_bar_packed_file_toggle_tips);
}
