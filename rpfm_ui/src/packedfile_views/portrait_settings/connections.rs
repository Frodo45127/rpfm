//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `PortraitSettingsView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PortraitSettingsView` and `PortraitSettingsSlots` structs.
!*/

use std::sync::Arc;

use super::{PortraitSettingsView, slots::PortraitSettingsSlots};

/// This function connects all the actions from the provided `PortraitSettingsView` with their slots in `PortraitSettingsSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &Arc<PortraitSettingsView>, slots: &PortraitSettingsSlots) {
    ui.main_list_view().selection_model().selection_changed().connect(slots.load_entry_to_detailed_view());
    ui.variants_list_view().selection_model().selection_changed().connect(slots.load_variant_to_detailed_view());

    ui.main_list_model().data_changed().connect(slots.modified());

    ui.timer_delayed_updates_main().timeout().connect(slots.delayed_updates_main());
    ui.timer_delayed_updates_variants().timeout().connect(slots.delayed_updates_variants());

    ui.main_filter_line_edit().text_changed().connect(slots.filter_edited_main());
    ui.variants_filter_line_edit().text_changed().connect(slots.filter_edited_variants());

    ui.main_list_view().custom_context_menu_requested().connect(slots.main_list_context_menu());
    ui.main_list_view().selection_model().selection_changed().connect(slots.main_list_context_menu_enabler());

    ui.variants_list_view().custom_context_menu_requested().connect(slots.variants_list_context_menu());
    ui.variants_list_view().selection_model().selection_changed().connect(slots.variants_list_context_menu_enabler());

    ui.main_list_add().triggered().connect(slots.main_list_add());
    ui.main_list_clone().triggered().connect(slots.main_list_clone());
    ui.main_list_delete().triggered().connect(slots.main_list_delete());

    ui.variants_list_add().triggered().connect(slots.variants_list_add());
    ui.variants_list_clone().triggered().connect(slots.variants_list_clone());
    ui.variants_list_delete().triggered().connect(slots.variants_list_delete());

    ui.file_diffuse_line_edit().text_changed().connect(slots.reload_variant_images());
    ui.file_mask_1_line_edit().text_changed().connect(slots.reload_variant_images());
    ui.file_mask_2_line_edit().text_changed().connect(slots.reload_variant_images());
    ui.file_mask_3_line_edit().text_changed().connect(slots.reload_variant_images());

    ui.timer_delayed_reload_variant_images().timeout().connect(slots.delayed_reload_variant_images());
}
