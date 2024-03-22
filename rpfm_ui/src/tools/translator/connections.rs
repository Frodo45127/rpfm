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
Module with all the code to connect `ToolTranslator` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `ToolTranslator` and `ToolTranslatorSlots` structs.
!*/

use qt_widgets::q_dialog_button_box::StandardButton;

use super::{ToolTranslator, slots::ToolTranslatorSlots};

/// This function connects all the actions from the provided `ToolTranslator` with their slots in `ToolTranslatorSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &ToolTranslator, slots: &ToolTranslatorSlots) {
    ui.table().table_view().selection_model().selection_changed().connect(slots.load_data_to_detailed_view());

    ui.move_selection_up().released().connect(slots.move_selection_up());
    ui.move_selection_down().released().connect(slots.move_selection_down());
    ui.copy_from_source().released().connect(slots.copy_from_source());
    ui.import_from_translated_pack().released().connect(slots.import_from_translated_pack());

    ui.action_move_up().triggered().connect(slots.move_selection_up());
    ui.action_move_down().triggered().connect(slots.move_selection_down());
    ui.action_copy_from_source().triggered().connect(slots.copy_from_source());
    ui.action_import_from_translated_pack().triggered().connect(slots.import_from_translated_pack());

    ui.tool.button_box.button(StandardButton::Cancel).released().connect(ui.tool.get_ref_dialog().slot_close());
    ui.tool.button_box.button(StandardButton::Ok).released().connect(ui.tool.get_ref_dialog().slot_accept());
}
