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
Module with all the code to connect `PackedFileDecoderView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackedFileDecoderView` and `PackedFileDecoderViewSlots` structs.
!*/

use super::{PackedFileDecoderView, slots::PackedFileDecoderViewSlots};

/// This function connects all the actions from the provided `PackedFileDecoderView` with their slots in `PackedFileDecoderViewSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackedFileDecoderView, slots: &PackedFileDecoderViewSlots) {

    // Sync the scroll bars of the three hex data views.
    ui.get_mut_ptr_hex_view_index().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);
    ui.get_mut_ptr_hex_view_raw().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);
    ui.get_mut_ptr_hex_view_decoded().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);

    // Signal to sync the selection between both HexViews.
    ui.get_mut_ptr_hex_view_raw().selection_changed().connect(&slots.hex_view_selection_raw_sync);
    ui.get_mut_ptr_hex_view_decoded().selection_changed().connect(&slots.hex_view_selection_decoded_sync);

    ui.get_mut_ptr_table_view().custom_context_menu_requested().connect(&slots.table_view_context_menu);
    ui.get_mut_ptr_table_view_old_versions().custom_context_menu_requested().connect(&slots.table_view_versions_context_menu);


}
