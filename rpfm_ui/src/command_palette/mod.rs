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
Module with all the code related to the command palette.
!*/

use qt_widgets::action::Action;

use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;

use qt_core::flags::Flags;
use qt_core::qt::CaseSensitivity;

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

/// This is the character we always have to remove from the action names while comparing them.
const THE_UNHOLY_ONE: &str = "&";

/// This function returns the complete list of actions available for the Command Palette.
pub fn get_actions(
	app_ui: &AppUI,
	pack_file_contents_ui: &PackFileContentsUI
) -> Vec<(*mut Action, String)> {

	let mut actions = vec![];
    let shortcuts = UI_STATE.get_shortcuts_no_lock();

	//-------------------------------------------------------------------------------//
    // `PackFile` menu.
    //-------------------------------------------------------------------------------//
	actions.push((app_ui.packfile_new_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.packfile_open_packfile, shortcuts.menu_bar_packfile["open_packfile"].to_owned()));
	actions.push((app_ui.packfile_save_packfile, shortcuts.menu_bar_packfile["save_packfile"].to_owned()));
	actions.push((app_ui.packfile_save_packfile_as, shortcuts.menu_bar_packfile["save_packfile_as"].to_owned()));
	actions.push((app_ui.packfile_load_all_ca_packfiles, shortcuts.menu_bar_packfile["load_all_ca_packfiles"].to_owned()));
	actions.push((app_ui.packfile_preferences, shortcuts.menu_bar_packfile["preferences"].to_owned()));
	actions.push((app_ui.packfile_quit, shortcuts.menu_bar_packfile["quit"].to_owned()));

    //-------------------------------------------------------------------------------//
    // `View` menu.
    //-------------------------------------------------------------------------------//
	//actions.push((app_ui.view_toggle_packfile_contents, shortcuts.menu_bar_packfile["toggle_packfile_contents"].to_owned()));
	//actions.push((app_ui.view_toggle_global_search_panel, shortcuts.menu_bar_packfile["toggle_global_search_panel"].to_owned()));

    //-------------------------------------------------------------------------------//
    // `Game Selected` menu.
    //-------------------------------------------------------------------------------//
	//actions.push((app_ui.game_selected_open_game_data_folder, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	//actions.push((app_ui.game_selected_open_game_assembly_kit_folder, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	//-------------------------------------------------------------------------------//
    // `Special Stuff` menu.
    //-------------------------------------------------------------------------------//
	actions.push((app_ui.special_stuff_three_k_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_three_k_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_wh2_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_wh2_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_wh2_patch_siege_ai, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_wh_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_wh_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_wh_patch_siege_ai, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_tob_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_tob_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_att_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_att_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_rom2_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_rom2_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_sho2_generate_pak_file, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));
	actions.push((app_ui.special_stuff_sho2_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_nap_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

	actions.push((app_ui.special_stuff_emp_optimize_packfile, shortcuts.menu_bar_packfile["new_packfile"].to_owned()));

    //-------------------------------------------------------------------------------//
    // `About` menu.
    //-------------------------------------------------------------------------------//
	actions.push((app_ui.about_about_qt, shortcuts.menu_bar_about["about_qt"].to_owned()));
	actions.push((app_ui.about_about_rpfm, shortcuts.menu_bar_about["about_rpfm"].to_owned()));
	actions.push((app_ui.about_open_manual, shortcuts.menu_bar_about["open_manual"].to_owned()));
	actions.push((app_ui.about_patreon_link, "".to_owned()));
	actions.push((app_ui.about_check_updates, shortcuts.menu_bar_about["check_updates"].to_owned()));
	actions.push((app_ui.about_check_schema_updates, shortcuts.menu_bar_about["check_schema_updates"].to_owned()));

	//-------------------------------------------------------------------------------//
    // Contextual menu for the PackFile Contents TreeView.
    //-------------------------------------------------------------------------------//
	actions.push((pack_file_contents_ui.context_menu_add_file, shortcuts.packfile_contents_tree_view["add_file"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_add_folder, shortcuts.packfile_contents_tree_view["add_folder"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_add_from_packfile, shortcuts.packfile_contents_tree_view["add_from_packfile"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_new_folder, shortcuts.packfile_contents_tree_view["create_folder"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_new_packed_file_db, shortcuts.packfile_contents_tree_view["create_db"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_new_packed_file_loc, shortcuts.packfile_contents_tree_view["create_loc"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_new_packed_file_text, shortcuts.packfile_contents_tree_view["create_text"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_mass_import_tsv, shortcuts.packfile_contents_tree_view["mass_import_tsv"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_mass_export_tsv, shortcuts.packfile_contents_tree_view["mass_export_tsv"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_rename, shortcuts.packfile_contents_tree_view["rename"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_delete, shortcuts.packfile_contents_tree_view["delete"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_extract, shortcuts.packfile_contents_tree_view["extract"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_open_decoder, shortcuts.packfile_contents_tree_view["open_in_decoder"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_open_dependency_manager, shortcuts.packfile_contents_tree_view["open_packfiles_list"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_open_containing_folder, shortcuts.packfile_contents_tree_view["open_containing_folder"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_open_with_external_program, shortcuts.packfile_contents_tree_view["open_with_external_program"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_open_notes, shortcuts.packfile_contents_tree_view["open_notes"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_check_tables, shortcuts.packfile_contents_tree_view["check_tables"].to_owned()));
	actions.push((pack_file_contents_ui.context_menu_merge_tables, shortcuts.packfile_contents_tree_view["merge_tables"].to_owned()));

	actions
}

/// This function loads the entire set of available and enabled actions to the Command Palette.
pub fn load_actions(app_ui: &AppUI, pack_file_contents_ui: &PackFileContentsUI) {
	unsafe { app_ui.command_palette_completer_model.as_mut().unwrap().clear(); }
	let and = QString::from_std_str(THE_UNHOLY_ONE);

	for (mut action_name, action_shortcut) in get_actions(app_ui, pack_file_contents_ui).iter_mut()
		.filter(|x| unsafe { x.0.as_mut().unwrap().is_enabled() })
		.map(|x| (unsafe { x.0.as_mut().unwrap().text() }, x.1.to_owned())) {

		let mut action_data = ListStandardItemMutPtr::new(());
		action_name.remove(&and);

		let mut action_name = StandardItem::new(&action_name);
		action_name.set_text_alignment(Flags::from_int(128));

		let mut action_shortcut = StandardItem::new(&QString::from_std_str(&action_shortcut));
		action_shortcut.set_text_alignment(Flags::from_int(130));

		unsafe { action_data.append_unsafe(&action_name.into_raw()) };
		unsafe { action_data.append_unsafe(&action_shortcut.into_raw()) };
		unsafe { app_ui.command_palette_completer_model.as_mut().unwrap().append_row(&action_data); }
	}

	unsafe { app_ui.command_palette_completer_view.as_mut().unwrap().set_column_width(0, 360); }
}

/// This function executes the action provided (if exists).
pub fn exec_action(app_ui: &AppUI, pack_file_contents_ui: &PackFileContentsUI, action_name: &QString) {
	let and = QString::from_std_str(THE_UNHOLY_ONE);
	for (action, _) in get_actions(app_ui, pack_file_contents_ui) {
		let mut name = unsafe { action.as_ref().unwrap().text() };
		name.remove(&and);
		if QString::compare(&name, action_name) == 0 {
			unsafe { action.as_mut().unwrap().trigger(); }
		}
	}
}
