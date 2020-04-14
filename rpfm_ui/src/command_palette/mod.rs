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

use qt_widgets::QAction;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;

use qt_core::QFlags;
use qt_core::AlignmentFlag;

use cpp_core::MutPtr;
use cpp_core::Ref;

use crate::app_ui::AppUI;
use crate::ffi::add_to_q_list_safe;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

/// This is the character we always have to remove from the action names while comparing them.
const THE_UNHOLY_ONE: &str = "&";

/// This function returns the complete list of actions available for the Command Palette.
pub unsafe fn get_actions(
	app_ui: &AppUI,
	pack_file_contents_ui: &PackFileContentsUI
) -> Vec<(MutPtr<QAction>, String)> {

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
pub unsafe fn load_actions(app_ui: &mut AppUI, pack_file_contents_ui: &PackFileContentsUI) {
	app_ui.command_palette_completer_model.clear();
	let and = QString::from_std_str(THE_UNHOLY_ONE);

	for (mut action_name, action_shortcut) in get_actions(app_ui, pack_file_contents_ui).iter_mut()
		.filter(|x| x.0.is_enabled())
		.map(|x| (x.0.text(), x.1.to_owned())) {

		let action_data = QListOfQStandardItem::new().into_ptr();
		action_name.remove_q_string(&and);

		let mut action_name = QStandardItem::from_q_string(&action_name).into_ptr();
		action_name.set_text_alignment(QFlags::from(AlignmentFlag::AlignVCenter));

		let mut action_shortcut = QStandardItem::from_q_string(&QString::from_std_str(&action_shortcut)).into_ptr();
		action_shortcut.set_text_alignment(AlignmentFlag::AlignVCenter | AlignmentFlag::AlignRight);

		add_to_q_list_safe(action_data, action_name);
		add_to_q_list_safe(action_data, action_shortcut);
		app_ui.command_palette_completer_model.append_row_q_list_of_q_standard_item(action_data.as_ref().unwrap());
	}

	app_ui.command_palette_completer_view.set_column_width(0, 360);
}

/// This function executes the action provided (if exists).
pub unsafe fn exec_action(app_ui: &AppUI, pack_file_contents_ui: &PackFileContentsUI, action_name: Ref<QString>) {
	let and = QString::from_std_str(THE_UNHOLY_ONE);
	for (mut action, _) in get_actions(app_ui, pack_file_contents_ui) {
		let mut name = action.text();
		name.remove_q_string(&and);
		if QString::compare_2_q_string(name.as_ref(), action_name) == 0 {
			action.trigger();
		}
	}
}
