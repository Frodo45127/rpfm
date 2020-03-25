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
Module with all the code to connect `PackedFileExternalView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileExternalView` and `PackedFileExternalViewSlots` structs.
!*/

use super::{PackedFileExternalView, slots::PackedFileExternalViewSlots};

/// This function connects all the actions from the provided `PackedFileExternalView` with their slots in `PackedFileExternalViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackedFileExternalView, slots: &PackedFileExternalViewSlots) {
    ui.get_mut_ptr_stop_watching_button().released().connect(&slots.stop_watching);
    ui.get_mut_ptr_open_folder_button().released().connect(&slots.open_folder);
}
