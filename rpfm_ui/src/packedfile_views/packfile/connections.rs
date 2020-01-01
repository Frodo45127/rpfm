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
Module with all the code to connect `PackFileExtraView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackFileExtraView` and `PackFileExtraView` structs.
!*/

use qt_core::connection::Signal;

use super::{PackFileExtraView, slots::PackFileExtraViewSlots};

/// This function connects all the actions from the provided `PackFileExtraView` with their slots in `PackFileExtraView`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub fn set_connections(ui: &PackFileExtraView, slots: &PackFileExtraViewSlots) {
    ui.get_ref_mut_tree_view().signals().double_clicked().connect(&slots.import);
    ui.get_ref_mut_filter_line_edit().signals().text_changed().connect(&slots.filter_change_text);

    ui.get_ref_mut_autoexpand_matches_button().signals().toggled().connect(&slots.filter_change_autoexpand_matches);
    ui.get_ref_mut_case_sensitive_button().signals().toggled().connect(&slots.filter_change_case_sensitive);

    ui.get_ref_mut_expand_all().signals().triggered().connect(&slots.expand_all);
    ui.get_ref_mut_collapse_all().signals().triggered().connect(&slots.collapse_all);
}
