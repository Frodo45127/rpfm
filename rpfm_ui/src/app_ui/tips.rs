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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `AppUI`.
!*/

use crate::QString;
use crate::app_ui::AppUI;

/// This function sets the status bar tip for all the actions in the provided `AppUI`.
pub fn set_tips(app_ui: &AppUI) {

    //-----------------------------------------------//
    // `PackFile` menu tips.
    //-----------------------------------------------//
    unsafe { app_ui.packfile_new_packfile.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_new_packfile")); }
    unsafe { app_ui.packfile_open_packfile.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_open_packfile")); }
    unsafe { app_ui.packfile_save_packfile.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_save_packfile")); }
    unsafe { app_ui.packfile_save_packfile_as.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_save_packfile_as")); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_load_all_ca_packfiles")); }
    unsafe { app_ui.packfile_preferences.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_preferences")); }
    unsafe { app_ui.packfile_quit.as_mut().unwrap().set_status_tip(&qtr("tt_packfile_quit")); }

    unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_boot")); }
    unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_release")); }
    unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_patch")); }
    unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_mod")); }
    unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_movie")); }
    unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_other")); }

    unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_data_is_encrypted")); }
    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_status_tip(&qtr("Itt_change_packfile_type_index_includes_timestamp")); }
    unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_index_is_encrypted")); }
    unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_header_is_extended")); }
    unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_status_tip(&qtr("tt_change_packfile_type_data_is_compressed")); }

    //-----------------------------------------------//
    // `Game Selected` menu tips.
    //-----------------------------------------------//
    unsafe { app_ui.game_selected_open_game_data_folder.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_open_game_data_folder")); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_open_game_assembly_kit_folder")); }
    
    unsafe { app_ui.game_selected_three_kingdoms.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_three_kingdoms")); }
    unsafe { app_ui.game_selected_warhammer_2.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_warhammer_2")); }
    unsafe { app_ui.game_selected_warhammer.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_warhammer")); }
    unsafe { app_ui.game_selected_thrones_of_britannia.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_thrones_of_britannia")); }
    unsafe { app_ui.game_selected_attila.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_attila")); }
    unsafe { app_ui.game_selected_rome_2.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_rome_2")); }
    unsafe { app_ui.game_selected_shogun_2.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_shogun_2")); }
    unsafe { app_ui.game_selected_napoleon.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_napoleon")); }
    unsafe { app_ui.game_selected_empire.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_empire")); }
    unsafe { app_ui.game_selected_arena.as_mut().unwrap().set_status_tip(&qtr("tt_game_selected_arena")); }

    //-----------------------------------------------//
    // `Special Stuff` menu tips.
    //-----------------------------------------------//
    let generate_pak_file = qtr("tt_generate_pak_file");
    let optimize_packfile = qtr("tt_optimize_packfile");
    let patch_siege_ai_tip = qtr("tt_patch_siege_ai");
    unsafe { app_ui.special_stuff_three_k_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_three_k_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_wh2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_wh2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
    unsafe { app_ui.special_stuff_wh_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_wh_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
    unsafe { app_ui.special_stuff_tob_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_tob_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_att_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_att_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_rom2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_rom2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_sho2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
    unsafe { app_ui.special_stuff_sho2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_nap_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
    unsafe { app_ui.special_stuff_emp_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }

    //-----------------------------------------------//
    // `About` menu tips.
    //-----------------------------------------------//
    unsafe { app_ui.about_about_qt.as_mut().unwrap().set_status_tip(&qtr("tt_about_about_qt")); }
    unsafe { app_ui.about_about_rpfm.as_mut().unwrap().set_status_tip(&qtr("tt_about_about_rpfm")); }
    unsafe { app_ui.about_open_manual.as_mut().unwrap().set_status_tip(&qtr("tt_about_open_manual")); }
    unsafe { app_ui.about_patreon_link.as_mut().unwrap().set_status_tip(&qtr("tt_about_patreon_link")); }
    unsafe { app_ui.about_check_updates.as_mut().unwrap().set_status_tip(&qtr("tt_about_check_updates")); }
    unsafe { app_ui.about_check_schema_updates.as_mut().unwrap().set_status_tip(&qtr("tt_about_check_schema_updates")); }
}
