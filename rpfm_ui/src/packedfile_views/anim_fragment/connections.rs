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
Module with all the code to connect `PackedFileAnimFragmentView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileAnimFragmentView` and `PackedFileAnimFragmentViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackedFileAnimFragmentView, slots::PackedFileAnimFragmentViewSlots};

/// This function connects all the actions from the provided `PackedFileAnimFragmentView` with their slots in `PackedFileAnimFragmentViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileAnimFragmentView>, slots: &PackedFileAnimFragmentViewSlots) {
    ui.get_ref_table_view_2().timer_delayed_updates.timeout().connect(&slots.delayed_updates);
}

