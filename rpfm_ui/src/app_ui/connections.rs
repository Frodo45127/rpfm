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
Module with all the code to connect `AppUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `AppUI` and `AppUISlots` structs.
!*/

use std::rc::Rc;

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
    app_ui.packfile_open_packfile.triggered().connect(&slots.packfile_open_packfile);
    app_ui.packfile_save_packfile.triggered().connect(&slots.packfile_save_packfile);
    app_ui.packfile_save_packfile_as.triggered().connect(&slots.packfile_save_packfile_as);
    app_ui.packfile_install.triggered().connect(&slots.packfile_install);
    app_ui.packfile_uninstall.triggered().connect(&slots.packfile_uninstall);
    app_ui.packfile_load_all_ca_packfiles.triggered().connect(&slots.packfile_load_all_ca_packfiles);

    app_ui.change_packfile_type_boot.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_release.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_patch.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_mod.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_movie.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_other.triggered().connect(&slots.packfile_change_packfile_type);
    app_ui.change_packfile_type_index_includes_timestamp.triggered().connect(&slots.packfile_index_includes_timestamp);
    app_ui.change_packfile_type_data_is_compressed.triggered().connect(&slots.packfile_data_is_compressed);

    app_ui.packfile_preferences.triggered().connect(&slots.packfile_preferences);
    app_ui.packfile_quit.triggered().connect(&slots.packfile_quit);

    //-----------------------------------------------//
    // `MyMod` menu connections.
    //-----------------------------------------------//
    app_ui.menu_bar_mymod.about_to_show().connect(&slots.mymod_open_menu);
    app_ui.mymod_open_mymod_folder.triggered().connect(&slots.mymod_open_mymod_folder);
    app_ui.mymod_new.triggered().connect(&slots.mymod_new);
    app_ui.mymod_delete_selected.triggered().connect(&slots.mymod_delete_selected);
    app_ui.mymod_export.triggered().connect(&slots.mymod_export);
    app_ui.mymod_import.triggered().connect(&slots.mymod_import);

    //-----------------------------------------------//
    // `View` menu connections.
    //-----------------------------------------------//
    app_ui.menu_bar_view.about_to_show().connect(&slots.view_open_menu);

    app_ui.view_toggle_packfile_contents.toggled().connect(&slots.view_toggle_packfile_contents);
    app_ui.view_toggle_global_search_panel.toggled().connect(&slots.view_toggle_global_search_panel);
    app_ui.view_toggle_diagnostics_panel.toggled().connect(&slots.view_toggle_diagnostics_panel);
    app_ui.view_toggle_dependencies_panel.toggled().connect(&slots.view_toggle_dependencies_panel);

    //-----------------------------------------------//
    // `Game Selected` menu connections.
    //-----------------------------------------------//
    app_ui.game_selected_launch_game.triggered().connect(&slots.game_selected_launch_game);

    app_ui.game_selected_open_game_data_folder.triggered().connect(&slots.game_selected_open_game_data_folder);
    app_ui.game_selected_open_game_assembly_kit_folder.triggered().connect(&slots.game_selected_open_game_assembly_kit_folder);
    app_ui.game_selected_open_config_folder.triggered().connect(&slots.game_selected_open_config_folder);

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
    // `Special Stuff` menu connections.
    //-----------------------------------------------//
    app_ui.special_stuff_wh2_patch_siege_ai.triggered().connect(&slots.special_stuff_patch_siege_ai);
    app_ui.special_stuff_wh_patch_siege_ai.triggered().connect(&slots.special_stuff_patch_siege_ai);

    app_ui.special_stuff_wh3_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_troy_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_three_k_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_wh2_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_wh_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_tob_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_att_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_rom2_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_sho2_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_nap_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);
    app_ui.special_stuff_emp_optimize_packfile.triggered().connect(&slots.special_stuff_optimize_packfile);

    app_ui.special_stuff_wh3_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_troy_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_three_k_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_wh2_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_wh_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_tob_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_att_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_rom2_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_sho2_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_nap_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);
    app_ui.special_stuff_emp_generate_dependencies_cache.triggered().connect(&slots.special_stuff_generate_dependencies_cache);

    app_ui.special_stuff_rescue_packfile.triggered().connect(&slots.special_stuff_rescue_packfile);

    //-----------------------------------------------//
    // `Tools` menu connections.
    //-----------------------------------------------//
    app_ui.tools_faction_painter.triggered().connect(&slots.tools_faction_painter);
    app_ui.tools_unit_editor.triggered().connect(&slots.tools_unit_editor);

    //-----------------------------------------------//
    // `About` menu connections.
    //-----------------------------------------------//
    app_ui.about_about_qt.triggered().connect(&slots.about_about_qt);
    app_ui.about_about_rpfm.triggered().connect(&slots.about_about_rpfm);
    app_ui.about_open_manual.triggered().connect(&slots.about_open_manual);
    app_ui.about_patreon_link.triggered().connect(&slots.about_patreon_link);
    app_ui.about_check_updates.triggered().connect(&slots.about_check_updates);
    app_ui.about_check_schema_updates.triggered().connect(&slots.about_check_schema_updates);
    app_ui.about_check_message_updates.triggered().connect(&slots.about_check_message_updates);

    //-----------------------------------------------//
    // `Debug` menu connections.
    //-----------------------------------------------//
    app_ui.debug_update_current_schema_from_asskit.triggered().connect(&slots.debug_update_current_schema_from_asskit);
    app_ui.debug_import_schema_patch.triggered().connect(&slots.debug_import_schema_patch);

    //-----------------------------------------------//
    // `PackedFileView` connections.
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
    app_ui.tab_bar_packed_file_close_all_left.triggered().connect(&slots.tab_bar_packed_file_close_all_left);
    app_ui.tab_bar_packed_file_close_all_right.triggered().connect(&slots.tab_bar_packed_file_close_all_right);
    app_ui.tab_bar_packed_file_prev.triggered().connect(&slots.tab_bar_packed_file_prev);
    app_ui.tab_bar_packed_file_next.triggered().connect(&slots.tab_bar_packed_file_next);
    app_ui.tab_bar_packed_file_import_from_dependencies.triggered().connect(&slots.tab_bar_packed_file_import_from_dependencies);
    app_ui.tab_bar_packed_file_toggle_tips.triggered().connect(&slots.tab_bar_packed_file_toggle_tips);
}
