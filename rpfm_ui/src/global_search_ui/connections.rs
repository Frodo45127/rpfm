//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use std::rc::Rc;

use super::{GlobalSearchUI, slots::GlobalSearchSlots};

/// This function connects all the actions from the provided `GlobalSearchUI` with their slots in `GlobalSearchSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(global_search_ui: &Rc<GlobalSearchUI>, slots: &GlobalSearchSlots) {
    global_search_ui.search_button.released().connect(&slots.search);
    global_search_ui.clear_button.released().connect(&slots.clear);
    global_search_ui.replace_button.released().connect(&slots.replace_current);
    global_search_ui.replace_all_button.released().connect(&slots.replace_all);
    global_search_ui.search_line_edit.return_pressed().connect(&slots.search);
    global_search_ui.search_line_edit.text_changed().connect(&slots.check_regex);
    global_search_ui.use_regex_checkbox.toggled().connect(&slots.check_regex_clean);

    global_search_ui.matches_table_and_text_tree_view.double_clicked().connect(&slots.open_match);

    global_search_ui.search_on_all_checkbox.toggled().connect(&slots.toggle_all);

    global_search_ui.matches_filter_table_and_text_line_edit.text_changed().connect(&slots.filter_table_and_text);
    global_search_ui.matches_case_sensitive_table_and_text_button.toggled().connect(&slots.filter_table_and_text);
    global_search_ui.matches_column_selector_table_and_text_combobox.current_text_changed().connect(&slots.filter_table_and_text);

    global_search_ui.matches_filter_schema_line_edit.text_changed().connect(&slots.filter_schemas);
    global_search_ui.matches_case_sensitive_schema_button.toggled().connect(&slots.filter_schemas);
    global_search_ui.matches_column_selector_schema_combobox.current_text_changed().connect(&slots.filter_schemas);
}
