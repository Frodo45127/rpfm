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

    ui.get_mut_ptr_bool_button().released().connect(&slots.use_this_bool);
    ui.get_mut_ptr_f32_button().released().connect(&slots.use_this_f32);
    ui.get_mut_ptr_f64_button().released().connect(&slots.use_this_f64);
    ui.get_mut_ptr_i16_button().released().connect(&slots.use_this_i16);
    ui.get_mut_ptr_i32_button().released().connect(&slots.use_this_i32);
    ui.get_mut_ptr_i64_button().released().connect(&slots.use_this_i64);
    ui.get_mut_ptr_colour_rgb_button().released().connect(&slots.use_this_colour_rgb);
    ui.get_mut_ptr_string_u8_button().released().connect(&slots.use_this_string_u8);
    ui.get_mut_ptr_string_u16_button().released().connect(&slots.use_this_string_u16);
    ui.get_mut_ptr_optional_string_u8_button().released().connect(&slots.use_this_optional_string_u8);
    ui.get_mut_ptr_optional_string_u16_button().released().connect(&slots.use_this_optional_string_u16);
    ui.get_mut_ptr_sequence_u32_button().released().connect(&slots.use_this_sequence_u32);

    // Signal to sync the selection between both HexViews.
    ui.get_mut_ptr_hex_view_raw().selection_changed().connect(&slots.hex_view_selection_raw_sync);
    ui.get_mut_ptr_hex_view_decoded().selection_changed().connect(&slots.hex_view_selection_decoded_sync);

    ui.get_mut_ptr_table_model().data_changed().connect(&slots.table_change_field_type);

    ui.get_mut_ptr_table_view_context_menu_move_up().triggered().connect(&slots.table_view_context_menu_move_up);
    ui.get_mut_ptr_table_view_context_menu_move_down().triggered().connect(&slots.table_view_context_menu_move_down);
    ui.get_mut_ptr_table_view_context_menu_move_left().triggered().connect(&slots.table_view_context_menu_move_left);
    ui.get_mut_ptr_table_view_context_menu_move_rigth().triggered().connect(&slots.table_view_context_menu_move_right);
    ui.get_mut_ptr_table_view_context_menu_delete().triggered().connect(&slots.table_view_context_menu_delete);

    ui.get_mut_ptr_table_view().custom_context_menu_requested().connect(&slots.table_view_context_menu);
    ui.get_mut_ptr_table_view().selection_model().selection_changed().connect(&slots.table_view_context_menu_enabler);

    ui.get_mut_ptr_table_view_old_versions().custom_context_menu_requested().connect(&slots.table_view_versions_context_menu);
    ui.get_mut_ptr_table_view_old_versions().selection_model().selection_changed().connect(&slots.table_view_versions_context_menu_enabler);

    ui.get_mut_ptr_table_view_old_versions_context_menu_load().triggered().connect(&slots.table_view_old_versions_context_menu_load);
    ui.get_mut_ptr_table_view_old_versions_context_menu_delete().triggered().connect(&slots.table_view_old_versions_context_menu_delete);

    ui.get_mut_ptr_import_from_assembly_kit_button().released().connect(&slots.import_from_assembly_kit);
    ui.get_mut_ptr_test_definition_button().released().connect(&slots.test_definition);
    ui.get_mut_ptr_clear_definition_button().released().connect(&slots.remove_all_fields);
    ui.get_mut_ptr_save_button().released().connect(&slots.save_definition);
}
