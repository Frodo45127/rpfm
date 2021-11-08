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
Module with all the code to connect `ToolUnitEditor` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ToolUnitEditor` and `ToolUnitEditorSlots` structs.
!*/

use qt_widgets::q_dialog_button_box::StandardButton;

use super::{ToolUnitEditor, slots::ToolUnitEditorSlots};

/// This function connects all the actions from the provided `ToolUnitEditor` with their slots in `ToolUnitEditorSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ToolUnitEditor, slots: &ToolUnitEditorSlots) {
    ui.unit_list_view.selection_model().selection_changed().connect(&slots.load_data_to_detailed_view);
    ui.unit_list_filter_line_edit.text_changed().connect(&slots.filter_edited);
    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);
    ui.main_units_caste_combobox.current_index_changed().connect(&slots.change_caste);
    ui.unit_icon_key_combobox.current_index_changed().connect(&slots.change_icon);

    ui.tool.button_box.button(StandardButton::Cancel).released().connect(ui.tool.get_ref_dialog().slot_close());
    ui.tool.button_box.button(StandardButton::Ok).released().connect(ui.tool.get_ref_dialog().slot_accept());
}
