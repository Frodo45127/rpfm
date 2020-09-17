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
Module with all the code to connect `PackedFileCaVp8View` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileCaVp8View` and `PackedFileCaVp8ViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackedFileCaVp8View, slots::PackedFileCaVp8ViewSlots};

/// This function connects all the actions from the provided `PackedFileCaVp8View` with their slots in `PackedFileCaVp8ViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileCaVp8View>, slots: &PackedFileCaVp8ViewSlots) {
    ui.get_mut_ptr_convert_to_camv_button().released().connect(&slots.convert_to_camv);
    ui.get_mut_ptr_convert_to_ivf_button().released().connect(&slots.convert_to_ivf);
}
