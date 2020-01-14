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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `PackFileContentsUI`.
!*/

use crate::locale::qtr;
use super::PackFileContentsUI;

/// This function sets the status bar tip for all the actions in the provided `PackFileContentsUI`.
pub fn set_tips(ui: &PackFileContentsUI) {

    //---------------------------------------------------//
    // PackFile Contents TreeView's Contextual menu tips.
    //---------------------------------------------------//
    unsafe { ui.context_menu_add_file.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_add_file")); }
    unsafe { ui.context_menu_add_folder.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_add_folder")); }
    unsafe { ui.context_menu_add_from_packfile.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_add_from_packfile")); }
    unsafe { ui.context_menu_check_tables.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_check_tables")); }
    unsafe { ui.context_menu_new_folder.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_new_folder")); }
    unsafe { ui.context_menu_new_packed_file_db.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_new_packed_file_db")); }
    unsafe { ui.context_menu_new_packed_file_loc.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_new_packed_file_loc")); }
    unsafe { ui.context_menu_new_packed_file_text.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_new_packed_file_text")); }
    unsafe { ui.context_menu_mass_import_tsv.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_mass_import_tsv")); }
    unsafe { ui.context_menu_mass_export_tsv.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_mass_export_tsv")); }
    unsafe { ui.context_menu_merge_tables.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_merge_tables")); }
    unsafe { ui.context_menu_delete.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_delete")); }
    unsafe { ui.context_menu_extract.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_extract")); }
    unsafe { ui.context_menu_rename.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_rename")); }
    unsafe { ui.context_menu_open_decoder.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_open_decoder")); }
    unsafe { ui.context_menu_open_dependency_manager.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_open_dependency_manager")); }
    unsafe { ui.context_menu_open_containing_folder.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_open_containing_folder")); }
    unsafe { ui.context_menu_open_with_external_program.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_open_with_external_program")); }
    unsafe { ui.context_menu_open_notes.as_mut().unwrap().set_status_tip(&qtr("tt_context_menu_open_notes")); }

    //---------------------------------------------------//
    // PackFile Contents panel tips.
    //---------------------------------------------------//
    unsafe { ui.filter_autoexpand_matches_button.as_mut().unwrap().set_status_tip(&qtr("tt_filter_autoexpand_matches_button")); }
    unsafe { ui.filter_case_sensitive_button.as_mut().unwrap().set_status_tip(&qtr("tt_filter_case_sensitive_button")); }
}
