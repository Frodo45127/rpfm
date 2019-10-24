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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `PackFileContentsUI`.
!*/

use crate::QString;
use super::PackFileContentsUI;

/// This function sets the status bar tip for all the actions in the provided `PackFileContentsUI`.
pub fn set_tips(ui: &PackFileContentsUI) {

    //---------------------------------------------------//
    // PackFile Contents TreeView's Contextual menu tips.
    //---------------------------------------------------//
    unsafe { ui.context_menu_add_file.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add one or more files to the currently open PackFile. Existing files are not overwriten!")); }
    unsafe { ui.context_menu_add_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add a folder to the currently open PackFile. Existing files are not overwriten!")); }
    unsafe { ui.context_menu_add_from_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add files from another PackFile to the currently open PackFile. Existing files are not overwriten!")); }
    unsafe { ui.context_menu_check_tables.as_mut().unwrap().set_status_tip(&QString::from_std_str("Check all the DB Tables of the currently open PackFile for dependency errors.")); }
    unsafe { ui.context_menu_create_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create an empty folder. Due to how the PackFiles are done, these are NOT KEPT ON SAVING if they stay empty.")); }
    unsafe { ui.context_menu_create_loc.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Loc File (used by the game to store the texts you see ingame) in the selected folder.")); }
    unsafe { ui.context_menu_create_db.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a DB Table (used by the game for... most of the things).")); }
    unsafe { ui.context_menu_create_text.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Plain Text File. It accepts different extensions, like '.xml', '.lua', '.txt',....")); }
    unsafe { ui.context_menu_mass_import_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a bunch of TSV files at the same time. It automatically checks if they are DB Tables, Locs or invalid TSVs, and imports them all at once. Existing files will be overwritten!")); }
    unsafe { ui.context_menu_mass_export_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export every DB Table and Loc PackedFile from this PackFile as TSV files at the same time. Existing files will be overwritten!")); }
    unsafe { ui.context_menu_merge_tables.as_mut().unwrap().set_status_tip(&QString::from_std_str("Merge multple DB Tables/Loc PackedFiles into one.")); }
    unsafe { ui.context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete the selected File/Folder.")); }
    unsafe { ui.context_menu_extract.as_mut().unwrap().set_status_tip(&QString::from_std_str("Extract the selected File/Folder from the PackFile.")); }
    unsafe { ui.context_menu_rename.as_mut().unwrap().set_status_tip(&QString::from_std_str("Rename the selected File/Folder. Remember, whitespaces are NOT ALLOWED and duplicated names in the same folder will NOT BE RENAMED.")); }
    unsafe { ui.context_menu_open_decoder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the selected table in the DB Decoder. To create/update schemas.")); }
    unsafe { ui.context_menu_open_dependency_manager.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the list of PackFiles referenced from this PackFile.")); }
    unsafe { ui.context_menu_open_containing_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the currently open PackFile's location in your default file manager.")); }
    unsafe { ui.context_menu_open_with_external_program.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in an external program.")); }
    unsafe { ui.context_menu_open_in_multi_view.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in a secondary view, without closing the currently open one.")); }
    unsafe { ui.context_menu_open_notes.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackFile's Notes in a secondary view, without closing the currently open PackedFile in the Main View.")); }

    //---------------------------------------------------//
    // PackFile Contents panel tips.
    //---------------------------------------------------//
    unsafe { ui.filter_autoexpand_matches_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Auto-Expand matches. NOTE: Filtering with all matches expanded in a big PackFile (+10k files, like data.pack) can hang the program for a while. You have been warned.")); }
    unsafe { ui.filter_case_sensitive_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Enable/Disable case sensitive filtering for the TreeView.")); }
    unsafe { ui.filter_filter_by_folder_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Set the filter to only filter by folder names and show all the files inside the matched folders.")); }
}
