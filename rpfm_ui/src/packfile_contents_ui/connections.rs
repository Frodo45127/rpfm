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
Module with all the code to connect `PackFileContentsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackFileContentsUI` and `PackFileContentsSlots` structs.
!*/

use qt_widgets::widget::Widget;
use qt_core::connection::Signal;

use super::{PackFileContentsUI, slots::PackFileContentsSlots};

/// This function connects all the actions from the provided `PackFileContentsUI` with their slots in `PackFileContentsSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub fn set_connections(ui: &PackFileContentsUI, slots: &PackFileContentsSlots) {
    unsafe { ui.packfile_contents_tree_view.as_ref().unwrap().signals().clicked().connect(&slots.open_packedfile_full); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.open_packedfile_preview); }
    //unsafe { ui.packfile_contents_tree_view.as_ref().unwrap().signals().activated().connect(&slots.open_packedfile_full); }
    //unsafe { ui.packfile_contents_tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.open_packedfile_full); }

    // Trigger the filter whenever the "filtered" text or any of his settings changes.
    unsafe { ui.filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slots.filter_change_text); }
    unsafe { ui.filter_autoexpand_matches_button.as_mut().unwrap().signals().toggled().connect(&slots.filter_change_autoexpand_matches); }
    unsafe { ui.filter_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slots.filter_change_case_sensitive); }

    unsafe { (ui.packfile_contents_tree_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.contextual_menu); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.contextual_menu_enabler); }
    unsafe { ui.packfile_contents_tree_view_context_menu.as_mut().unwrap().signals().about_to_show().connect(&slots.contextual_menu_enabler); }

    unsafe { ui.context_menu_add_file.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_add_file); }
    unsafe { ui.context_menu_add_folder.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_add_folder); }
    unsafe { ui.context_menu_add_from_packfile.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_add_from_packfile); }
    unsafe { ui.context_menu_delete.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_delete); }
    unsafe { ui.context_menu_extract.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_extract); }
    unsafe { ui.context_menu_rename.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_rename); }

    unsafe { ui.context_menu_new_folder.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_new_folder); }
    unsafe { ui.context_menu_new_packed_file_db.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_new_packed_file_db); }
    unsafe { ui.context_menu_new_packed_file_loc.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_new_packed_file_loc); }
    unsafe { ui.context_menu_new_packed_file_text.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_new_packed_file_text); }
    unsafe { ui.context_menu_new_queek_packed_file.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_new_queek_packed_file); }
    unsafe { ui.context_menu_check_tables.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_tables_check_integrity); }
    unsafe { ui.context_menu_merge_tables.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_tables_merge_tables); }
    unsafe { ui.context_menu_update_table.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_tables_update_table); }

    unsafe { ui.context_menu_mass_import_tsv.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_mass_import_tsv); }
    unsafe { ui.context_menu_mass_export_tsv.as_ref().unwrap().signals().triggered().connect(&slots.contextual_menu_mass_export_tsv); }

    unsafe { ui.packfile_contents_tree_view_expand_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_expand_all); }
    unsafe { ui.packfile_contents_tree_view_collapse_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_collapse_all); }
}
