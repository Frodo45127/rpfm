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
Module with all the code to connect `ToolFactionPainter` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ToolFactionPainter` and `ToolFactionPainterSlots` structs.
!*/

use qt_widgets::q_dialog_button_box::StandardButton;

use super::{ToolFactionPainter, slots::ToolFactionPainterSlots};

/// This function connects all the actions from the provided `ToolFactionPainter` with their slots in `ToolFactionPainterSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ToolFactionPainter, slots: &ToolFactionPainterSlots) {
    ui.faction_list_view.selection_model().selection_changed().connect(&slots.load_data_to_detailed_view);
    ui.faction_list_filter_line_edit.text_changed().connect(&slots.filter_edited);
    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);

    ui.banner_restore_initial_values_button.released().connect(&slots.banner_restore_initial_values);
    ui.banner_restore_vanilla_values_button.released().connect(&slots.banner_restore_vanilla_values);
    ui.uniform_restore_initial_values_button.released().connect(&slots.uniform_restore_initial_values);
    ui.uniform_restore_vanilla_values_button.released().connect(&slots.uniform_restore_vanilla_values);

    ui.button_box.button(StandardButton::Cancel).released().connect(ui.dialog.slot_close());
    ui.button_box.button(StandardButton::Ok).released().connect(ui.dialog.slot_accept());
}
