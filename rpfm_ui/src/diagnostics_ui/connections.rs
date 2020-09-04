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
Module with all the code to connect `DiagnosticsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `DiagnosticsUI` and `DiagnosticsUISlots` structs.
!*/

use super::{DiagnosticsUI, slots::DiagnosticsUISlots};

/// This function connects all the actions from the provided `DiagnosticsUI` with their slots in `DiagnosticsUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &DiagnosticsUI, slots: &DiagnosticsUISlots) {
    ui.diagnostics_table_view.double_clicked().connect(&slots.diagnostics_open_result);

    ui.diagnostics_button_info.toggled().connect(&slots.toggle_filters_by_level);
    ui.diagnostics_button_warning.toggled().connect(&slots.toggle_filters_by_level);
    ui.diagnostics_button_error.toggled().connect(&slots.toggle_filters_by_level);
}
