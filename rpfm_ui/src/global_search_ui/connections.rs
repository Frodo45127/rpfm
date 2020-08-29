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
Module with all the code to connect `GlobalSearchUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `GlobalSearchUI` and `GlobalSearchSlots` structs.
!*/

use super::{GlobalSearchUI, slots::GlobalSearchSlots};

/// This function connects all the actions from the provided `GlobalSearchUI` with their slots in `GlobalSearchSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(global_search_ui: &GlobalSearchUI, slots: &GlobalSearchSlots) {
    global_search_ui.global_search_search_button.released().connect(&slots.global_search_search);
    global_search_ui.global_search_clear_button.released().connect(&slots.global_search_clear);
    global_search_ui.global_search_replace_button.released().connect(&slots.global_search_replace_current);
    global_search_ui.global_search_replace_all_button.released().connect(&slots.global_search_replace_all);
    global_search_ui.global_search_search_line_edit.return_pressed().connect(&slots.global_search_search);
    global_search_ui.global_search_search_line_edit.text_changed().connect(&slots.global_search_check_regex);
    global_search_ui.global_search_use_regex_checkbox.toggled().connect(&slots.global_search_check_regex_clean);

    global_search_ui.global_search_matches_db_tree_view.double_clicked().connect(&slots.global_search_open_match);
    global_search_ui.global_search_matches_loc_tree_view.double_clicked().connect(&slots.global_search_open_match);
    global_search_ui.global_search_matches_text_tree_view.double_clicked().connect(&slots.global_search_open_match);

    global_search_ui.global_search_search_on_all_checkbox.toggled().connect(&slots.global_search_toggle_all);

    global_search_ui.global_search_matches_filter_db_line_edit.text_changed().connect(&slots.global_search_filter_dbs);
    global_search_ui.global_search_matches_case_sensitive_db_button.toggled().connect(&slots.global_search_filter_dbs);
    global_search_ui.global_search_matches_column_selector_db_combobox.current_text_changed().connect(&slots.global_search_filter_dbs);

    global_search_ui.global_search_matches_filter_loc_line_edit.text_changed().connect(&slots.global_search_filter_locs);
    global_search_ui.global_search_matches_case_sensitive_loc_button.toggled().connect(&slots.global_search_filter_locs);
    global_search_ui.global_search_matches_column_selector_loc_combobox.current_text_changed().connect(&slots.global_search_filter_locs);

    global_search_ui.global_search_matches_filter_text_line_edit.text_changed().connect(&slots.global_search_filter_texts);
    global_search_ui.global_search_matches_case_sensitive_text_button.toggled().connect(&slots.global_search_filter_texts);
    global_search_ui.global_search_matches_column_selector_text_combobox.current_text_changed().connect(&slots.global_search_filter_texts);

    global_search_ui.global_search_matches_filter_schema_line_edit.text_changed().connect(&slots.global_search_filter_schemas);
    global_search_ui.global_search_matches_case_sensitive_schema_button.toggled().connect(&slots.global_search_filter_schemas);
    global_search_ui.global_search_matches_column_selector_schema_combobox.current_text_changed().connect(&slots.global_search_filter_schemas);
}
