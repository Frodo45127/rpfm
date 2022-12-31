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
Module with all the code to connect `PackedFileVideoView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileVideoView` and `PackedFileVideoViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackedFileVideoView, slots::PackedFileVideoViewSlots};

/// This function connects all the actions from the provided `PackedFileVideoView` with their slots in `PackedFileVideoViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileVideoView>, slots: &PackedFileVideoViewSlots) {
    ui.get_mut_ptr_convert_to_camv_button().released().connect(&slots.convert_to_camv);
    ui.get_mut_ptr_convert_to_ivf_button().released().connect(&slots.convert_to_ivf);
}
