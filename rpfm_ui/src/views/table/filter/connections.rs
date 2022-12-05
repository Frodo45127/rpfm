//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use super::{FilterView, slots::FilterViewSlots};

pub unsafe fn set_connections_filter(ui: &FilterView, slots: &FilterViewSlots) {
    ui.filter_line_edit.text_changed().connect(&slots.filter_line_edit);
    ui.filter_line_edit.text_changed().connect(&slots.filter_check_regex);

    ui.group_combobox.current_index_changed().connect(&slots.filter_match_group_selector);
    ui.column_combobox.current_index_changed().connect(&slots.filter_column_selector);
    ui.case_sensitive_button.toggled().connect(&slots.filter_case_sensitive_button);
    ui.show_blank_cells_button.toggled().connect(&slots.filter_show_blank_cells_button);
    ui.timer_delayed_updates.timeout().connect(&slots.filter_trigger);
    ui.add_button.released().connect(&slots.filter_add);
    ui.remove_button.released().connect(&slots.filter_remove);
}
