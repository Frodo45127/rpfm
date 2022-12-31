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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `AppUI`.
!*/

use std::rc::Rc;

use crate::locale::qtr;
use crate::app_ui::AppUI;

/// This function sets the status bar tip for all the actions in the provided `AppUI`.
pub unsafe fn set_tips(app_ui: &Rc<AppUI>) {

    //-----------------------------------------------//
    // `PackFile` menu tips.
    //-----------------------------------------------//
    app_ui.packfile_new_packfile.set_status_tip(&qtr("tt_packfile_new_packfile"));
    app_ui.packfile_open_packfile.set_status_tip(&qtr("tt_packfile_open_packfile"));
    app_ui.packfile_save_packfile.set_status_tip(&qtr("tt_packfile_save_packfile"));
    app_ui.packfile_save_packfile_as.set_status_tip(&qtr("tt_packfile_save_packfile_as"));
    app_ui.packfile_install.set_status_tip(&qtr("tt_packfile_install"));
    app_ui.packfile_uninstall.set_status_tip(&qtr("tt_packfile_uninstall"));
    app_ui.packfile_load_all_ca_packfiles.set_status_tip(&qtr("tt_packfile_load_all_ca_packfiles"));
    app_ui.packfile_preferences.set_status_tip(&qtr("tt_packfile_preferences"));
    app_ui.packfile_quit.set_status_tip(&qtr("tt_packfile_quit"));

    app_ui.change_packfile_type_boot.set_status_tip(&qtr("tt_change_packfile_type_boot"));
    app_ui.change_packfile_type_release.set_status_tip(&qtr("tt_change_packfile_type_release"));
    app_ui.change_packfile_type_patch.set_status_tip(&qtr("tt_change_packfile_type_patch"));
    app_ui.change_packfile_type_mod.set_status_tip(&qtr("tt_change_packfile_type_mod"));
    app_ui.change_packfile_type_movie.set_status_tip(&qtr("tt_change_packfile_type_movie"));
    app_ui.change_packfile_type_other.set_status_tip(&qtr("tt_change_packfile_type_other"));

    app_ui.change_packfile_type_data_is_encrypted.set_status_tip(&qtr("tt_change_packfile_type_data_is_encrypted"));
    app_ui.change_packfile_type_index_includes_timestamp.set_status_tip(&qtr("tt_change_packfile_type_index_includes_timestamp"));
    app_ui.change_packfile_type_index_is_encrypted.set_status_tip(&qtr("tt_change_packfile_type_index_is_encrypted"));
    app_ui.change_packfile_type_header_is_extended.set_status_tip(&qtr("tt_change_packfile_type_header_is_extended"));
    app_ui.change_packfile_type_data_is_compressed.set_status_tip(&qtr("tt_change_packfile_type_data_is_compressed"));

    //-----------------------------------------------//
    // `MyMod` menu tips.
    //-----------------------------------------------//
    app_ui.mymod_new.set_status_tip(&qtr("tt_mymod_new"));
    app_ui.mymod_delete_selected.set_status_tip(&qtr("tt_mymod_delete_selected"));
    app_ui.mymod_import.set_status_tip(&qtr("tt_mymod_import"));
    app_ui.mymod_export.set_status_tip(&qtr("tt_mymod_export"));

    //-----------------------------------------------//
    // `Game Selected` menu tips.
    //-----------------------------------------------//
    app_ui.game_selected_launch_game.set_status_tip(&qtr("tt_game_selected_launch_game"));
    app_ui.game_selected_open_game_data_folder.set_status_tip(&qtr("tt_game_selected_open_game_data_folder"));
    app_ui.game_selected_open_game_assembly_kit_folder.set_status_tip(&qtr("tt_game_selected_open_game_assembly_kit_folder"));
    app_ui.game_selected_open_config_folder.set_status_tip(&qtr("tt_game_selected_open_config_folder"));

    app_ui.game_selected_warhammer_3.set_status_tip(&qtr("tt_game_selected_warhammer_3"));
    app_ui.game_selected_troy.set_status_tip(&qtr("tt_game_selected_troy"));
    app_ui.game_selected_three_kingdoms.set_status_tip(&qtr("tt_game_selected_three_kingdoms"));
    app_ui.game_selected_warhammer_2.set_status_tip(&qtr("tt_game_selected_warhammer_2"));
    app_ui.game_selected_warhammer.set_status_tip(&qtr("tt_game_selected_warhammer"));
    app_ui.game_selected_thrones_of_britannia.set_status_tip(&qtr("tt_game_selected_thrones_of_britannia"));
    app_ui.game_selected_attila.set_status_tip(&qtr("tt_game_selected_attila"));
    app_ui.game_selected_rome_2.set_status_tip(&qtr("tt_game_selected_rome_2"));
    app_ui.game_selected_shogun_2.set_status_tip(&qtr("tt_game_selected_shogun_2"));
    app_ui.game_selected_napoleon.set_status_tip(&qtr("tt_game_selected_napoleon"));
    app_ui.game_selected_empire.set_status_tip(&qtr("tt_game_selected_empire"));
    app_ui.game_selected_arena.set_status_tip(&qtr("tt_game_selected_arena"));

    //-----------------------------------------------//
    // `Special Stuff` menu tips.
    //-----------------------------------------------//
    let generate_dependencies_cache = qtr("tt_generate_dependencies_cache");
    let optimize_packfile = qtr("tt_optimize_packfile");
    let patch_siege_ai_tip = qtr("tt_patch_siege_ai");
    app_ui.special_stuff_wh3_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_wh3_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_troy_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_troy_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_three_k_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_three_k_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_wh2_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_wh2_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_wh2_patch_siege_ai.set_status_tip(&patch_siege_ai_tip);
    app_ui.special_stuff_wh_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_wh_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_wh_patch_siege_ai.set_status_tip(&patch_siege_ai_tip);
    app_ui.special_stuff_tob_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_tob_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_att_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_att_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_rom2_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_rom2_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_sho2_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_sho2_optimize_packfile.set_status_tip(&optimize_packfile);

    app_ui.special_stuff_nap_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_nap_optimize_packfile.set_status_tip(&optimize_packfile);
    app_ui.special_stuff_emp_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);
    app_ui.special_stuff_emp_optimize_packfile.set_status_tip(&optimize_packfile);

    //-----------------------------------------------//
    // `About` menu tips.
    //-----------------------------------------------//
    app_ui.about_about_qt.set_status_tip(&qtr("tt_about_about_qt"));
    app_ui.about_about_rpfm.set_status_tip(&qtr("tt_about_about_rpfm"));
    app_ui.about_open_manual.set_status_tip(&qtr("tt_about_open_manual"));
    app_ui.about_patreon_link.set_status_tip(&qtr("tt_about_patreon_link"));
    app_ui.about_check_updates.set_status_tip(&qtr("tt_about_check_updates"));
    app_ui.about_check_schema_updates.set_status_tip(&qtr("tt_about_check_schema_updates"));
    app_ui.about_check_lua_autogen_updates.set_status_tip(&qtr("tt_about_check_lua_autogen_updates"));
}
