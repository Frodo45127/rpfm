//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `PackedFileAnimPackView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileAnimPackView` and `PackedFileAnimPackViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackedFileAnimPackView, slots::PackedFileAnimPackViewSlots};

/// This function connects all the actions from the provided `PackedFileAnimPackView` with their slots in `PackedFileAnimPackViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileAnimPackView>, slots: &PackedFileAnimPackViewSlots) {

    ui.pack_tree_view().double_clicked().connect(slots.copy_in());
    ui.anim_pack_tree_view().double_clicked().connect(slots.copy_out());

    ui.pack_filter_line_edit().text_changed().connect(slots.pack_filter_change_text());
    ui.pack_filter_autoexpand_matches_button().toggled().connect(slots.pack_filter_change_autoexpand_matches());
    ui.pack_filter_case_sensitive_button().toggled().connect(slots.pack_filter_change_case_sensitive());

    ui.anim_pack_filter_line_edit().text_changed().connect(slots.anim_pack_filter_change_text());
    ui.anim_pack_filter_autoexpand_matches_button().toggled().connect(slots.anim_pack_filter_change_autoexpand_matches());
    ui.anim_pack_filter_case_sensitive_button().toggled().connect(slots.anim_pack_filter_change_case_sensitive());

    ui.pack_expand_all().triggered().connect(slots.pack_expand_all());
    ui.pack_collapse_all().triggered().connect(slots.pack_collapse_all());
    ui.anim_pack_expand_all().triggered().connect(slots.anim_pack_expand_all());
    ui.anim_pack_collapse_all().triggered().connect(slots.anim_pack_collapse_all());
    ui.anim_pack_delete().triggered().connect(slots.delete());
}
