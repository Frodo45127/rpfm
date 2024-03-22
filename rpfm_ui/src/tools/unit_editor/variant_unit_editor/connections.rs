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
Module with all the code to connect `SubToolVariantUnitEditor` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `SubToolVariantUnitEditor` and `SubToolVariantUnitEditorSlots` structs.
!*/

use super::{SubToolVariantUnitEditor, slots::SubToolVariantUnitEditorSlots};

/// This function connects all the actions from the provided `SubToolVariantUnitEditor` with their slots in `SubToolVariantUnitEditorSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &SubToolVariantUnitEditor, slots: &SubToolVariantUnitEditorSlots) {
    ui.faction_list_view.custom_context_menu_requested().connect(&slots.faction_list_context_menu);
    ui.faction_list_view.selection_model().selection_changed().connect(&slots.faction_list_context_menu_enabler);
    ui.faction_list_view.selection_model().selection_changed().connect(&slots.load_data_to_detailed_view);

    ui.faction_list_add_faction.triggered().connect(&slots.faction_list_add_faction);
    ui.faction_list_clone_faction.triggered().connect(&slots.faction_list_clone_faction);
    ui.faction_list_delete_faction.triggered().connect(&slots.faction_list_delete_faction);

    ui.faction_list_filter_line_edit.text_changed().connect(&slots.filter_edited);
    ui.timer_delayed_updates.timeout().connect(&slots.delayed_updates);
    ui.unit_variants_unit_card_combobox.current_index_changed().connect(&slots.change_icon);
    ui.variants_variant_filename_combobox.current_index_changed().connect(&slots.change_variant_mesh);

    ui.unit_variants_colours_list_view.custom_context_menu_requested().connect(&slots.unit_variants_colours_list_context_menu);
    ui.unit_variants_colours_list_view.selection_model().selection_changed().connect(&slots.unit_variants_colours_list_context_menu_enabler);
    ui.unit_variants_colours_list_view.selection_model().selection_changed().connect(&slots.load_unit_variants_colours_to_detailed_view);

    ui.unit_variants_colours_list_add_colour_variant.triggered().connect(&slots.unit_variants_colours_list_add_colour_variant);
    ui.unit_variants_colours_list_clone_colour_variant.triggered().connect(&slots.unit_variants_colours_list_clone_colour_variant);
    ui.unit_variants_colours_list_delete_colour_variant.triggered().connect(&slots.unit_variants_colours_list_delete_colour_variant);

    ui.new_faction_name_combobox.current_text_changed().connect(&slots.add_faction_check);
    ui.new_colour_variant_name_combobox.current_text_changed().connect(&slots.add_colour_variant_check);
}
