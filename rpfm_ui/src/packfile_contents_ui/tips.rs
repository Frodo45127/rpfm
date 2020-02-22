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
pub fn set_tips(ui: &mut PackFileContentsUI) {

    //---------------------------------------------------//
    // PackFile Contents TreeView's Contextual menu tips.
    //---------------------------------------------------//
    unsafe { ui.context_menu_add_file.set_status_tip(&qtr("tt_context_menu_add_file")); }
    unsafe { ui.context_menu_add_folder.set_status_tip(&qtr("tt_context_menu_add_folder")); }
    unsafe { ui.context_menu_add_from_packfile.set_status_tip(&qtr("tt_context_menu_add_from_packfile")); }
    unsafe { ui.context_menu_check_tables.set_status_tip(&qtr("tt_context_menu_check_tables")); }
    unsafe { ui.context_menu_new_folder.set_status_tip(&qtr("tt_context_menu_new_folder")); }
    unsafe { ui.context_menu_new_packed_file_db.set_status_tip(&qtr("tt_context_menu_new_packed_file_db")); }
    unsafe { ui.context_menu_new_packed_file_loc.set_status_tip(&qtr("tt_context_menu_new_packed_file_loc")); }
    unsafe { ui.context_menu_new_packed_file_text.set_status_tip(&qtr("tt_context_menu_new_packed_file_text")); }
    unsafe { ui.context_menu_mass_import_tsv.set_status_tip(&qtr("tt_context_menu_mass_import_tsv")); }
    unsafe { ui.context_menu_mass_export_tsv.set_status_tip(&qtr("tt_context_menu_mass_export_tsv")); }
    unsafe { ui.context_menu_merge_tables.set_status_tip(&qtr("tt_context_menu_merge_tables")); }
    unsafe { ui.context_menu_delete.set_status_tip(&qtr("tt_context_menu_delete")); }
    unsafe { ui.context_menu_extract.set_status_tip(&qtr("tt_context_menu_extract")); }
    unsafe { ui.context_menu_rename.set_status_tip(&qtr("tt_context_menu_rename")); }
    unsafe { ui.context_menu_open_decoder.set_status_tip(&qtr("tt_context_menu_open_decoder")); }
    unsafe { ui.context_menu_open_dependency_manager.set_status_tip(&qtr("tt_context_menu_open_dependency_manager")); }
    unsafe { ui.context_menu_open_containing_folder.set_status_tip(&qtr("tt_context_menu_open_containing_folder")); }
    unsafe { ui.context_menu_open_with_external_program.set_status_tip(&qtr("tt_context_menu_open_with_external_program")); }
    unsafe { ui.context_menu_open_notes.set_status_tip(&qtr("tt_context_menu_open_notes")); }

    //---------------------------------------------------//
    // PackFile Contents panel tips.
    //---------------------------------------------------//
    unsafe { ui.filter_autoexpand_matches_button.set_status_tip(&qtr("tt_filter_autoexpand_matches_button")); }
    unsafe { ui.filter_case_sensitive_button.set_status_tip(&qtr("tt_filter_case_sensitive_button")); }
}
