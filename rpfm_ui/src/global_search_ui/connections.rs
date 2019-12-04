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
Module with all the code to connect `GlobalSearchUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `GlobalSearchUI` and `GlobalSearchSlots` structs.
!*/

use qt_core::connection::Signal;

use super::{GlobalSearchUI, slots::GlobalSearchSlots};

/// This function connects all the actions from the provided `GlobalSearchUI` with their slots in `GlobalSearchSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub fn set_connections(global_search_ui: &GlobalSearchUI, slots: &GlobalSearchSlots) {
    unsafe { global_search_ui.global_search_search_button.as_ref().unwrap().signals().released().connect(&slots.global_search_search); }
    unsafe { global_search_ui.global_search_search_line_edit.as_ref().unwrap().signals().return_pressed().connect(&slots.global_search_search); }
    unsafe { global_search_ui.global_search_search_line_edit.as_ref().unwrap().signals().text_changed().connect(&slots.global_search_check_regex); }

    unsafe { global_search_ui.global_search_matches_db_tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.global_search_open_match); }
    unsafe { global_search_ui.global_search_matches_loc_tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.global_search_open_match); }
    unsafe { global_search_ui.global_search_matches_text_tree_view.as_ref().unwrap().signals().double_clicked().connect(&slots.global_search_open_match); }

    unsafe { global_search_ui.global_search_search_on_all_checkbox.as_ref().unwrap().signals().toggled().connect(&slots.global_search_toggle_all); }
}
