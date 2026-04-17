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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `AppUI`.
!*/

use std::rc::Rc;

use crate::{app_ui::AppUI, utils::qtr};

/// This function sets the status bar tip for all the actions in the provided `AppUI`.
pub unsafe fn set_tips(app_ui: &Rc<AppUI>) {

    //-----------------------------------------------//
    // `PackFile` menu tips.
    //-----------------------------------------------//
    app_ui.packfile_new_packfile.set_status_tip(&qtr("tt_packfile_new_packfile"));
    app_ui.packfile_load_all_ca_packfiles.set_status_tip(&qtr("tt_packfile_load_all_ca_packfiles"));
    app_ui.packfile_settings.set_status_tip(&qtr("tt_packfile_settings"));
    app_ui.packfile_quit.set_status_tip(&qtr("tt_packfile_quit"));

    //-----------------------------------------------//
    // `MyMod` menu tips.
    //-----------------------------------------------//
    app_ui.mymod_new.set_status_tip(&qtr("tt_mymod_new"));
    app_ui.mymod_import_all.set_status_tip(&qtr("tt_mymod_import_all"));
    app_ui.mymod_export_all.set_status_tip(&qtr("tt_mymod_export_all"));

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
    // `Game Selected` menu tips.
    //-----------------------------------------------//
    let generate_dependencies_cache = qtr("tt_generate_dependencies_cache");
    app_ui.game_selected_generate_dependencies_cache.set_status_tip(&generate_dependencies_cache);

    //-----------------------------------------------//
    // `About` menu tips.
    //-----------------------------------------------//
    app_ui.about_about_qt.set_status_tip(&qtr("tt_about_about_qt"));
    app_ui.about_about_rpfm.set_status_tip(&qtr("tt_about_about_rpfm"));
    app_ui.about_check_updates.set_status_tip(&qtr("tt_about_check_updates"));
}
