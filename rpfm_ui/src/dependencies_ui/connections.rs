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
Module with all the code to connect `DependenciesUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `DependenciesUI` and `DependenciesUISlots` structs.
!*/

use super::{DependenciesUI, slots::DependenciesUISlots};

/// This function connects all the actions from the provided `DependenciesUI` with their slots in `DependenciesUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &DependenciesUI, slots: &DependenciesUISlots) {
    ui.dependencies_tree_view.selection_model().selection_changed().connect(&slots.open_packedfile_preview);
    ui.dependencies_tree_view.double_clicked().connect(&slots.open_packedfile_full);

    // Trigger the filter whenever the "filtered" text or any of his settings changes.
    ui.filter_timer_delayed_updates.timeout().connect(&slots.filter_trigger);
    ui.filter_line_edit.text_changed().connect(&slots.filter_change_text);
    ui.filter_autoexpand_matches_button.toggled().connect(&slots.filter_change_autoexpand_matches);
    ui.filter_case_sensitive_button.toggled().connect(&slots.filter_change_case_sensitive);
    ui.filter_line_edit.text_changed().connect(&slots.filter_check_regex);

    ui.dependencies_tree_view.custom_context_menu_requested().connect(&slots.contextual_menu);
    ui.dependencies_tree_view.selection_model().selection_changed().connect(&slots.contextual_menu_enabler);
    ui.dependencies_tree_view_context_menu.about_to_show().connect(&slots.contextual_menu_enabler);

    ui.context_menu_extract.triggered().connect(&slots.contextual_menu_extract);
    ui.context_menu_import.triggered().connect(&slots.contextual_menu_import);
    ui.context_menu_copy_path.triggered().connect(&slots.contextual_menu_copy_path);

    ui.dependencies_tree_view_expand_all.triggered().connect(&slots.dependencies_tree_view_expand_all);
    ui.dependencies_tree_view_collapse_all.triggered().connect(&slots.dependencies_tree_view_collapse_all);
}
