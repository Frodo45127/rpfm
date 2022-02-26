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
Module with all the code to connect `PackedFileTextView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileTextView` and `PackedFileTextViewSlots` structs.
!*/

use std::sync::Arc;

use crate::ffi::get_text_changed_dummy_widget_safe;
use super::{PackedFileTextView, slots::PackedFileTextViewSlots};

/// This function connects all the actions from the provided `PackedFileTextView` with their slots in `PackedFileTextViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PackedFileTextView>, slots: &PackedFileTextViewSlots) {
    get_text_changed_dummy_widget_safe(&ui.editor.as_ptr()).text_changed().connect(&slots.modified);
}
