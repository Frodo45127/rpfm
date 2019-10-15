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
Module with all the code to connect `AppUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `AppUI` and `AppUISlots` structs.
!*/

use qt_core::connection::Signal;

use super::{AppUI, slots::AppUISlots};

/// This function connects all the actions from the provided `AppUI` with their slots in `AppUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub fn set_connections(app_ui: &AppUI, slots: &AppUISlots) {

	//-----------------------------------------------//
    // Command Palette connections.
    //-----------------------------------------------//
    unsafe { app_ui.command_palette_show.as_ref().unwrap().signals().triggered().connect(&slots.command_palette_show); }
    unsafe { app_ui.command_palette_hide.as_ref().unwrap().signals().triggered().connect(&slots.command_palette_hide); }

    unsafe { app_ui.command_palette_completer.as_ref().unwrap().signals().activated_qt_core_string_ref().connect(&slots.command_palette_trigger); }

    //-----------------------------------------------//
    // `PackFile` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.packfile_new_packfile.as_ref().unwrap().signals().triggered().connect(&slots.packfile_new_packfile); }
    unsafe { app_ui.packfile_open_packfile.as_ref().unwrap().signals().triggered().connect(&slots.packfile_open_packfile); }
    unsafe { app_ui.packfile_save_packfile.as_ref().unwrap().signals().triggered().connect(&slots.packfile_save_packfile); }
    unsafe { app_ui.packfile_save_packfile_as.as_ref().unwrap().signals().triggered().connect(&slots.packfile_save_packfile_as); }
    unsafe { app_ui.packfile_load_all_ca_packfiles.as_ref().unwrap().signals().triggered().connect(&slots.packfile_load_all_ca_packfiles); }

    unsafe { app_ui.change_packfile_type_boot.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_release.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_patch.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_mod.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_movie.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_other.as_ref().unwrap().signals().triggered().connect(&slots.packfile_change_packfile_type); }
    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_ref().unwrap().signals().triggered().connect(&slots.packfile_index_includes_timestamp); }
    unsafe { app_ui.change_packfile_type_data_is_compressed.as_ref().unwrap().signals().triggered().connect(&slots.packfile_data_is_compressed); }

    unsafe { app_ui.packfile_preferences.as_ref().unwrap().signals().triggered().connect(&slots.packfile_preferences); }
    unsafe { app_ui.packfile_quit.as_ref().unwrap().signals().triggered().connect(&slots.packfile_quit); }

    //-----------------------------------------------//
    // `MyMod` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.mymod_new.as_ref().unwrap().signals().triggered().connect(&slots.mymod_new); }
    unsafe { app_ui.mymod_delete_selected.as_ref().unwrap().signals().triggered().connect(&slots.mymod_delete_selected); }
    unsafe { app_ui.mymod_install.as_ref().unwrap().signals().triggered().connect(&slots.mymod_install); }
    unsafe { app_ui.mymod_uninstall.as_ref().unwrap().signals().triggered().connect(&slots.mymod_uninstall); }

    //-----------------------------------------------//
    // `View` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.view_toggle_packfile_contents.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_packfile_contents); }
    unsafe { app_ui.view_toggle_global_search_panel.as_ref().unwrap().signals().triggered().connect(&slots.view_toggle_global_search_panel); }

    //-----------------------------------------------//
    // `Game Selected` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.game_selected_open_game_data_folder.as_ref().unwrap().signals().triggered().connect(&slots.game_selected_open_game_data_folder); }
    unsafe { app_ui.game_selected_open_game_assembly_kit_folder.as_ref().unwrap().signals().triggered().connect(&slots.game_selected_open_game_assembly_kit_folder); }

    unsafe { app_ui.game_selected_three_kingdoms.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_warhammer_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_warhammer.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_thrones_of_britannia.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_attila.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_rome_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_shogun_2.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_napoleon.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_empire.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }
    unsafe { app_ui.game_selected_arena.as_ref().unwrap().signals().triggered().connect(&slots.change_game_selected); }

    //-----------------------------------------------//
    // `Special Stuff` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.special_stuff_wh2_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_patch_siege_ai); }
    unsafe { app_ui.special_stuff_wh_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_patch_siege_ai); }

    unsafe { app_ui.special_stuff_three_k_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_wh2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_wh_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_tob_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_att_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_rom2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_sho2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_nap_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }
    unsafe { app_ui.special_stuff_emp_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_optimize_packfile); }

    unsafe { app_ui.special_stuff_three_k_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_wh2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_wh_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_tob_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_att_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_rom2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }
    unsafe { app_ui.special_stuff_sho2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slots.special_stuff_generate_pak_file); }

    //-----------------------------------------------//
    // `About` menu connections.
    //-----------------------------------------------//
    unsafe { app_ui.about_about_qt.as_ref().unwrap().signals().triggered().connect(&slots.about_about_qt); }
    unsafe { app_ui.about_open_manual.as_ref().unwrap().signals().triggered().connect(&slots.about_open_manual); }
    unsafe { app_ui.about_patreon_link.as_ref().unwrap().signals().triggered().connect(&slots.about_patreon_link); }
    unsafe { app_ui.about_check_updates.as_ref().unwrap().signals().triggered().connect(&slots.about_check_updates); }
    unsafe { app_ui.about_check_schema_updates.as_ref().unwrap().signals().triggered().connect(&slots.about_check_schema_updates); }

    //-----------------------------------------------//
    // `PackedFileView` connections.
    //-----------------------------------------------//
    unsafe { app_ui.tab_bar_packed_file.as_ref().unwrap().signals().tab_close_requested().connect(&slots.packed_file_hide); }
}
