//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `AppUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `AppUI` and `AppUISlots` structs.
!*/

use std::rc::Rc;

use crate::ffi::main_window_drop_pack_signal;
use super::{AppUI, slots::AppUISlots};

/// This function connects all the actions from the provided `AppUI` with their slots in `AppUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(app_ui: &Rc<AppUI>, slots: &AppUISlots) {

    //-----------------------------------------------//
    // `PackFile` menu connections.
    //-----------------------------------------------//
    app_ui.menu_bar_packfile.about_to_show().connect(&slots.packfile_open_menu);

    app_ui.packfile_new_packfile.triggered().connect(&slots.packfile_new_packfile);
    app_ui.packfile_open_packfiles.triggered().connect(&slots.packfile_open_packfiles);
    app_ui.packfile_open_and_merge_packs.triggered().connect(&slots.packfile_open_and_merge_packs);
    app_ui.packfile_load_all_ca_packfiles.triggered().connect(&slots.packfile_load_all_ca_packfiles);
    app_ui.packfile_save_all.triggered().connect(&slots.packfile_save_all);

    app_ui.packfile_select_session.triggered().connect(&slots.packfile_select_session);
    app_ui.packfile_settings.triggered().connect(&slots.packfile_settings);
    app_ui.packfile_quit.triggered().connect(&slots.packfile_quit);

    //-----------------------------------------------//
    // `MyMod` menu connections.
    //-----------------------------------------------//
    app_ui.menu_bar_mymod.about_to_show().connect(&slots.mymod_open_menu);
    app_ui.mymod_open_mymod_folder.triggered().connect(&slots.mymod_open_mymod_folder);
    app_ui.mymod_new.triggered().connect(&slots.mymod_new);

    //-----------------------------------------------//
    // `View` menu connections.
    //-----------------------------------------------//
    app_ui.menu_bar_view.about_to_show().connect(&slots.view_open_menu);

    app_ui.view_toggle_packfile_contents.toggled().connect(&slots.view_toggle_packfile_contents);
    app_ui.view_toggle_global_search_panel.toggled().connect(&slots.view_toggle_global_search_panel);
    app_ui.view_toggle_diagnostics_panel.toggled().connect(&slots.view_toggle_diagnostics_panel);
    app_ui.view_toggle_dependencies_panel.toggled().connect(&slots.view_toggle_dependencies_panel);
    app_ui.view_toggle_references_panel.toggled().connect(&slots.view_toggle_references_panel);

    //-----------------------------------------------//
    // `Game Selected` menu connections.
    //-----------------------------------------------//
    app_ui.game_selected_launch_game.triggered().connect(&slots.game_selected_launch_game);

    app_ui.game_selected_open_game_data_folder.triggered().connect(&slots.game_selected_open_game_data_folder);
    app_ui.game_selected_open_game_assembly_kit_folder.triggered().connect(&slots.game_selected_open_game_assembly_kit_folder);
    app_ui.game_selected_open_config_folder.triggered().connect(&slots.game_selected_open_config_folder);

    app_ui.game_selected_pharaoh_dynasties.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_pharaoh.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_warhammer_3.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_troy.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_three_kingdoms.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_warhammer_2.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_warhammer.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_thrones_of_britannia.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_attila.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_rome_2.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_shogun_2.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_napoleon.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_empire.triggered().connect(&slots.change_game_selected);
    app_ui.game_selected_arena.triggered().connect(&slots.change_game_selected);

    //-----------------------------------------------//
    // `Game Selected` menu connections.
    //-----------------------------------------------//
    app_ui.game_selected_generate_dependencies_cache.triggered().connect(&slots.game_selected_generate_dependencies_cache);

    //-----------------------------------------------//
    // `Tools` menu connections.
    //-----------------------------------------------//
    app_ui.tools_faction_painter.triggered().connect(&slots.tools_faction_painter);
    app_ui.tools_unit_editor.triggered().connect(&slots.tools_unit_editor);
    app_ui.tools_translator.triggered().connect(&slots.tools_translator);

    //-----------------------------------------------//
    // `About` menu connections.
    //-----------------------------------------------//
    app_ui.about_about_qt.triggered().connect(&slots.about_about_qt);
    app_ui.about_about_rpfm.triggered().connect(&slots.about_about_rpfm);
    app_ui.about_check_updates.triggered().connect(&slots.about_check_updates);

    //-----------------------------------------------//
    // `Debug` menu connections.
    //-----------------------------------------------//
    app_ui.debug_update_current_schema_from_asskit.triggered().connect(&slots.debug_update_current_schema_from_asskit);
    app_ui.debug_import_schema_patch.triggered().connect(&slots.debug_import_schema_patch);
    app_ui.debug_reload_style_sheet.triggered().connect(&slots.debug_reload_style_sheet);

    //-----------------------------------------------//
    // `FileView` connections.
    //-----------------------------------------------//
    app_ui.tab_bar_packed_file.tab_close_requested().connect(&slots.packed_file_hide);
    app_ui.tab_bar_packed_file.current_changed().connect(&slots.packed_file_update);
    app_ui.tab_bar_packed_file.tab_bar_double_clicked().connect(&slots.packed_file_unpreview);

    //-----------------------------------------------//
    // `Generic` connections.
    //-----------------------------------------------//
    app_ui.timer_backup_autosave.timeout().connect(&slots.pack_file_backup_autosave);

    app_ui.tab_bar_packed_file.custom_context_menu_requested().connect(&slots.tab_bar_packed_file_context_menu_show);
    app_ui.tab_bar_packed_file_close.triggered().connect(&slots.tab_bar_packed_file_close);
    app_ui.tab_bar_packed_file_close_all.triggered().connect(&slots.tab_bar_packed_file_close_all);
    app_ui.tab_bar_packed_file_close_all_other.triggered().connect(&slots.tab_bar_packed_file_close_all_other);
    app_ui.tab_bar_packed_file_close_all_left.triggered().connect(&slots.tab_bar_packed_file_close_all_left);
    app_ui.tab_bar_packed_file_close_all_right.triggered().connect(&slots.tab_bar_packed_file_close_all_right);
    app_ui.tab_bar_packed_file_prev.triggered().connect(&slots.tab_bar_packed_file_prev);
    app_ui.tab_bar_packed_file_next.triggered().connect(&slots.tab_bar_packed_file_next);
    app_ui.tab_bar_packed_file_import_from_dependencies.triggered().connect(&slots.tab_bar_packed_file_import_from_dependencies);
    app_ui.tab_bar_packed_file_toggle_quick_notes.triggered().connect(&slots.tab_bar_packed_file_toggle_quick_notes);

    main_window_drop_pack_signal(app_ui.main_window.static_upcast()).connect(&slots.open_pack_drop);
    //-----------------------------------------------//
    // `StatusBar` connections.
    //-----------------------------------------------//
    app_ui.discord_button.released().connect(&slots.discord_link);
    app_ui.github_button.released().connect(&slots.github_link);
    app_ui.patreon_button.released().connect(&slots.patreon_link);
    app_ui.manual_button.released().connect(&slots.manual_link);
}
