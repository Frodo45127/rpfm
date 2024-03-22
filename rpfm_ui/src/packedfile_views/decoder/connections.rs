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
    ui.hex_view_index().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);
    ui.hex_view_raw().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);
    ui.hex_view_decoded().vertical_scroll_bar().value_changed().connect(&slots.hex_view_scroll_sync);

    ui.bool_button().released().connect(&slots.use_this_bool);
    ui.f32_button().released().connect(&slots.use_this_f32);
    ui.f64_button().released().connect(&slots.use_this_f64);
    ui.i16_button().released().connect(&slots.use_this_i16);
    ui.i32_button().released().connect(&slots.use_this_i32);
    ui.i64_button().released().connect(&slots.use_this_i64);
    ui.optional_i16_button().released().connect(&slots.use_this_optional_i16);
    ui.optional_i32_button().released().connect(&slots.use_this_optional_i32);
    ui.optional_i64_button().released().connect(&slots.use_this_optional_i64);
    ui.colour_rgb_button().released().connect(&slots.use_this_colour_rgb);
    ui.string_u8_button().released().connect(&slots.use_this_string_u8);
    ui.string_u16_button().released().connect(&slots.use_this_string_u16);
    ui.optional_string_u8_button().released().connect(&slots.use_this_optional_string_u8);
    ui.optional_string_u16_button().released().connect(&slots.use_this_optional_string_u16);
    ui.sequence_u32_button().released().connect(&slots.use_this_sequence_u32);

    // Signal to sync the selection between both HexViews.
    ui.hex_view_raw().selection_changed().connect(&slots.hex_view_selection_raw_sync);
    ui.hex_view_decoded().selection_changed().connect(&slots.hex_view_selection_decoded_sync);

    ui.table_model().data_changed().connect(&slots.table_change_field_type);

    ui.table_view_context_menu_move_up().triggered().connect(&slots.table_view_context_menu_move_up);
    ui.table_view_context_menu_move_down().triggered().connect(&slots.table_view_context_menu_move_down);
    ui.table_view_context_menu_move_left().triggered().connect(&slots.table_view_context_menu_move_left);
    ui.table_view_context_menu_move_right().triggered().connect(&slots.table_view_context_menu_move_right);
    ui.table_view_context_menu_delete().triggered().connect(&slots.table_view_context_menu_delete);

    ui.table_view().custom_context_menu_requested().connect(&slots.table_view_context_menu);
    ui.table_view().selection_model().selection_changed().connect(&slots.table_view_context_menu_enabler);

    ui.table_view_old_versions().custom_context_menu_requested().connect(&slots.table_view_versions_context_menu);
    ui.table_view_old_versions().selection_model().selection_changed().connect(&slots.table_view_versions_context_menu_enabler);

    ui.table_view_old_versions_context_menu_load().triggered().connect(&slots.table_view_old_versions_context_menu_load);
    ui.table_view_old_versions_context_menu_delete().triggered().connect(&slots.table_view_old_versions_context_menu_delete);

    ui.import_from_assembly_kit_button().released().connect(&slots.import_from_assembly_kit);
    ui.test_definition_button().released().connect(&slots.test_definition);
    ui.clear_definition_button().released().connect(&slots.remove_all_fields);
    ui.save_button().released().connect(&slots.save_definition);
}
