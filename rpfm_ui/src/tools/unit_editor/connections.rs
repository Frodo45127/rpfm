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
Module with all the code to connect `ToolUnitEditor` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ToolUnitEditor` and `ToolUnitEditorSlots` structs.
!*/

use qt_widgets::{QComboBox, q_dialog_button_box::StandardButton};

use rpfm_error::Result;

use super::{ToolUnitEditor, slots::ToolUnitEditorSlots};

/// This function connects all the actions from the provided `ToolUnitEditor` with their slots in `ToolUnitEditorSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ToolUnitEditor, slots: &ToolUnitEditorSlots) -> Result<()> {

    ui.unit_list_view.selection_model().selection_changed().connect(&slots.load_data_to_detailed_view);
    ui.unit_list_filter_line_edit.text_changed().connect(&slots.filter_edited);
    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);
    ui.tool.find_widget::<QComboBox>("main_units_caste_combobox")?.current_index_changed().connect(&slots.change_caste);

    ui.copy_unit_new_unit_name_combobox.current_text_changed().connect(&slots.copy_unit_check);

    ui.variant_editor_tool_button.released().connect(&slots.open_variant_editor);

    ui.tool.button_box.button(StandardButton::Cancel).released().connect(ui.tool.get_ref_dialog().slot_close());
    ui.tool.button_box.button(StandardButton::Ok).released().connect(ui.tool.get_ref_dialog().slot_accept());
    ui.copy_button.released().connect(&slots.copy_unit);

    Ok(())
}
