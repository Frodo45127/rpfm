//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `TableView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `TableView` and `TableViewSlots` structs.
!*/

use super::{SearchView, slots::SearchViewSlots};

/// This function connects all the actions from the provided `TableView` with their slots in `TableViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &SearchView, slots: &SearchViewSlots) {
    ui.search_button().released().connect(&slots.search);
    ui.prev_match_button().released().connect(&slots.prev_match);
    ui.next_match_button().released().connect(&slots.next_match);
    ui.replace_button().released().connect(&slots.replace);
    ui.replace_all_button().released().connect(&slots.replace_all);
    ui.close_button().released().connect(&slots.close);
    ui.search_line_edit().text_changed().connect(&slots.check_regex);
    ui.search_line_edit().return_pressed().connect(&slots.search);
}
