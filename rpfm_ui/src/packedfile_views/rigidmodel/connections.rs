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
Module with all the code to connect `RigidModelView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `RigidModelView` and `RigidModelSlots` structs.
!*/

use std::sync::Arc;

use super::{RigidModelView, slots::RigidModelSlots};

/// This function connects all the actions from the provided `RigidModelView` with their slots in `RigidModelSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<RigidModelView>, slots: &RigidModelSlots) {
    ui.lod_tree_view().selection_model().selection_changed().connect(slots.load_data_to_detailed_view());
    ui.version_combobox().current_text_changed().connect(slots.change_version());
    ui.export_gltf_button().released().connect(slots.export_gltf());
}
