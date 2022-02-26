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
Module with all the code to connect `PackFileExtraView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackFileExtraView` and `PackFileExtraViewSlots` structs.
!*/

use std::sync::Arc;

use super::{PackFileExtraView, slots::PackFileExtraViewSlots};

/// This function connects all the actions from the provided `PackFileExtraView` with their slots in `PackFileExtraViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackFileExtraView>, slots: &PackFileExtraViewSlots) {
    ui.get_mut_ptr_tree_view().double_clicked().connect(&slots.import);
    ui.get_mut_ptr_filter_line_edit().text_changed().connect(&slots.filter_change_text);

    ui.get_mut_ptr_autoexpand_matches_button().toggled().connect(&slots.filter_change_autoexpand_matches);
    ui.get_mut_ptr_case_sensitive_button().toggled().connect(&slots.filter_change_case_sensitive);

    ui.get_mut_ptr_expand_all().triggered().connect(&slots.expand_all);
    ui.get_mut_ptr_collapse_all().triggered().connect(&slots.collapse_all);
}
