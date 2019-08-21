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
Module with all the code related to the command palette.
!*/

use qt_widgets::action::Action;

use qt_core::string_list::StringList;

use crate::QString;
use crate::app_ui::AppUI;

/// This is the character we always have to remove from the action names while comparing them.
const THE_UNHOLY_ONE: &str = "&";

/// This function returns the complete list of actions available for the Command Palette.
pub fn get_actions(app_ui: &AppUI) -> Vec<*mut Action> {
	let mut actions = vec![];

	//-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.packfile_new_packfile);
	actions.push(app_ui.packfile_open_packfile);
	actions.push(app_ui.packfile_save_packfile);
	actions.push(app_ui.packfile_save_packfile_as);
	actions.push(app_ui.packfile_load_all_ca_packfiles);
	actions.push(app_ui.packfile_preferences);
	actions.push(app_ui.packfile_quit);

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.view_toggle_packfile_contents);
	actions.push(app_ui.view_toggle_global_search_panel);

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.game_selected_open_game_data_folder);
	actions.push(app_ui.game_selected_open_game_assembly_kit_folder);

	//-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.special_stuff_three_k_generate_pak_file);
	actions.push(app_ui.special_stuff_three_k_optimize_packfile);

	actions.push(app_ui.special_stuff_wh2_generate_pak_file);
	actions.push(app_ui.special_stuff_wh2_optimize_packfile);
	actions.push(app_ui.special_stuff_wh2_patch_siege_ai);

	actions.push(app_ui.special_stuff_wh_generate_pak_file);
	actions.push(app_ui.special_stuff_wh_optimize_packfile);
	actions.push(app_ui.special_stuff_wh_patch_siege_ai);

	actions.push(app_ui.special_stuff_tob_generate_pak_file);
	actions.push(app_ui.special_stuff_tob_optimize_packfile);

	actions.push(app_ui.special_stuff_att_generate_pak_file);
	actions.push(app_ui.special_stuff_att_optimize_packfile);

	actions.push(app_ui.special_stuff_rom2_generate_pak_file);
	actions.push(app_ui.special_stuff_rom2_optimize_packfile);

	actions.push(app_ui.special_stuff_sho2_generate_pak_file);
	actions.push(app_ui.special_stuff_sho2_optimize_packfile);

	actions.push(app_ui.special_stuff_nap_optimize_packfile);
	
	actions.push(app_ui.special_stuff_emp_optimize_packfile);

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.about_about_qt);
	actions.push(app_ui.about_about_rpfm);
	actions.push(app_ui.about_open_manual);
	actions.push(app_ui.about_patreon_link);
	actions.push(app_ui.about_check_updates);
	actions.push(app_ui.about_check_schema_updates);

	//-------------------------------------------------------------------------------//
    // Contextual menu for the PackFile Contents TreeView.
    //-------------------------------------------------------------------------------//
	actions.push(app_ui.context_menu_add_file);
	actions.push(app_ui.context_menu_add_folder);
	actions.push(app_ui.context_menu_add_from_packfile);
	actions.push(app_ui.context_menu_create_folder);
	actions.push(app_ui.context_menu_create_db);
	actions.push(app_ui.context_menu_create_loc);
	actions.push(app_ui.context_menu_create_text);
	actions.push(app_ui.context_menu_mass_import_tsv);
	actions.push(app_ui.context_menu_mass_export_tsv);
	actions.push(app_ui.context_menu_rename);
	actions.push(app_ui.context_menu_delete);
	actions.push(app_ui.context_menu_extract);
	actions.push(app_ui.context_menu_open_decoder);
	actions.push(app_ui.context_menu_open_dependency_manager);
	actions.push(app_ui.context_menu_open_containing_folder);
	actions.push(app_ui.context_menu_open_with_external_program);
	actions.push(app_ui.context_menu_open_in_multi_view);
	actions.push(app_ui.context_menu_open_notes);
	actions.push(app_ui.context_menu_check_tables);
	actions.push(app_ui.context_menu_merge_tables);
	actions.push(app_ui.context_menu_global_search);

	actions
}

/// This function loads the entire set of available and enabled actions to the Command Palette.
pub fn load_actions(app_ui: &AppUI) {
	let and = QString::from_std_str(THE_UNHOLY_ONE);
	let mut string_list = StringList::new(());
	for mut action_name in unsafe { get_actions(app_ui).iter_mut().filter(|x| x.as_mut().unwrap().is_enabled()).map(|x| x.as_mut().unwrap().text()) } {
		action_name.remove(&and);
		string_list.append(&action_name);
	}
	unsafe { app_ui.command_palette_completer_model.as_mut().unwrap().set_string_list(&string_list); }
}

/// This function executes the action provided (if exists).
pub fn exec_action(app_ui: &AppUI, action_name: &QString) {
	let and = QString::from_std_str(THE_UNHOLY_ONE);
	for action in get_actions(app_ui) {
		let mut name = unsafe { action.as_ref().unwrap().text() };
		name.remove(&and);
		if QString::compare(&name, action_name) == 0 {
			unsafe { action.as_mut().unwrap().trigger(); }
		}
	}
}