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
Module with all the code to connect `PackedFileESFView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileESFView` and `PackedFileESFViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackedFileESFView, slots::PackedFileESFViewSlots};

/// This function connects all the actions from the provided `PackedFileESFView` with their slots in `PackedFileESFViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileESFView>, slots: &PackedFileESFViewSlots) {

    // Trigger the filter whenever the "filtered" text or any of his settings changes.
    ui.filter_timer_delayed_updates.timeout().connect(&slots.filter_trigger);
    ui.filter_line_edit.text_changed().connect(&slots.filter_change_text);
    ui.filter_autoexpand_matches_button.toggled().connect(&slots.filter_change_autoexpand_matches);
    ui.filter_case_sensitive_button.toggled().connect(&slots.filter_change_case_sensitive);
    ui.filter_line_edit.text_changed().connect(&slots.filter_check_regex);

     ui.tree_view.selection_model().selection_changed().connect(&slots.open_node);
}
