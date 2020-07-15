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
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `TableView`.
!*/

use crate::locale::qtr;
use super::TableView;

/// This function sets the status bar tip for all the actions in the provided `TableView`.
pub unsafe fn set_tips(ui: &mut TableView) {

    // Status Tips for the actions.
    ui.get_mut_ptr_context_menu_add_rows().set_status_tip(&qtr("Add an empty row at the end of the table."));
    ui.get_mut_ptr_context_menu_insert_rows().set_status_tip(&qtr("Insert an empty row just above the one selected."));
    ui.get_mut_ptr_context_menu_delete_rows().set_status_tip(&qtr("Delete all the selected rows."));
    //ui.get_mut_ptr_context_menu_apply_maths_to_selection().set_status_tip(&qtr("Apply a simple mathematical operation to every cell in the selected cells."));
    //ui.get_mut_ptr_context_menu_rewrite_selection().set_status_tip(&qtr("Rewrite the selected cells using a pattern."));
    ui.get_mut_ptr_context_menu_clone_and_append().set_status_tip(&qtr("Duplicate the selected rows and append the new rows at the end of the table."));
    ui.get_mut_ptr_context_menu_clone_and_insert().set_status_tip(&qtr("Duplicate the selected rows and insert the new rows under the original ones."));
    ui.get_mut_ptr_context_menu_copy().set_status_tip(&qtr("Copy whatever is selected to the Clipboard."));
    ui.get_mut_ptr_context_menu_copy_as_lua_table().set_status_tip(&qtr("Turns the entire DB Table into a LUA Table and copies it to the clipboard."));
    ui.get_mut_ptr_context_menu_paste().set_status_tip(&qtr("Try to paste whatever is in the Clipboard. If the data of a cell is incompatible with the content to paste, the cell is ignored."));
    //ui.get_mut_ptr_context_menu_paste_as_new_lines().set_status_tip(&qtr("Try to paste whatever is in the Clipboard as new lines at the end of the table. Does nothing if the data is not compatible with the cell."));
    //ui.get_mut_ptr_context_menu_paste_to_fill_selection().set_status_tip(&qtr("Try to paste whatever is in the Clipboard in EVERY CELL selected. Does nothing if the data is not compatible with the cell."));
    //ui.get_mut_ptr_context_menu_selection_invert().set_status_tip(&qtr("Inverts the current selection."));
    //ui.get_mut_ptr_context_menu_search().set_status_tip(&qtr("Search what you want in the table. Also allows you to replace coincidences."));
    //ui.get_mut_ptr_context_menu_sidebar().set_status_tip(&qtr("Open/Close the sidebar with the controls to hide/show/freeze columns."));
    ui.get_mut_ptr_context_menu_import_tsv().set_status_tip(&qtr("Import a TSV file into this table, replacing all the data."));
    ui.get_mut_ptr_context_menu_export_tsv().set_status_tip(&qtr("Export this table's data into a TSV file."));
    ui.get_mut_ptr_context_menu_undo().set_status_tip(&qtr("A classic."));
    ui.get_mut_ptr_context_menu_redo().set_status_tip(&qtr("Another classic."));
}
