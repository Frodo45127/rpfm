//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `PackFileContentsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackFileContentsUI` and `PackFileContentsSlots` structs.
!*/

use qt_widgets::widget::Widget;
use qt_core::connection::Signal;

use super::{PackFileContentsUI, slots::PackFileContentsSlots};

/// This function connects all the actions from the provided `PackFileContentsUI` with their slots in `PackFileContentsSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub fn set_connections(ui: &PackFileContentsUI, slots: &PackFileContentsSlots) {

    unsafe { (ui.packfile_contents_tree_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.contextual_menu); }
    unsafe { ui.packfile_contents_tree_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.contextual_menu_enabler); }

    unsafe { ui.packfile_contents_tree_view_expand_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_expand_all); }
    unsafe { ui.packfile_contents_tree_view_collapse_all.as_ref().unwrap().signals().triggered().connect(&slots.packfile_contents_tree_view_collapse_all); }
}
