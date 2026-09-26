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
Module with all the code to connect `GroupFormationsView` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `GroupFormationsView` and `GroupFormationsSlots` structs.
!*/

use std::sync::Arc;

use super::{GroupFormationsView, ItemList, slots::GroupFormationsSlots};

use qt_core::QBox;
use qt_core::SlotNoArgs;

/// This function connects all the actions from the provided `GroupFormationsView` with their slots in `GroupFormationsSlots`.
pub unsafe fn set_connections(ui: &Arc<GroupFormationsView>, slots: &GroupFormationsSlots) {
    ui.formations_filter_line_edit().text_changed().connect(slots.filter_formations());
    ui.formations_list_view().selection_model().selection_changed().connect(slots.formation_selected());
    ui.formations_list_view().custom_context_menu_requested().connect(slots.formations_context_menu());
    ui.add_formation().triggered().connect(slots.add_formation());
    ui.clone_formation().triggered().connect(slots.clone_formation());
    ui.delete_formation().triggered().connect(slots.delete_formation());

    ui.blocks_tree_view().selection_model().selection_changed().connect(slots.blocks_selected());
    ui.blocks_tree_view().custom_context_menu_requested().connect(slots.blocks_context_menu());
    ui.add_absolute().triggered().connect(slots.add_absolute());
    ui.add_relative().triggered().connect(slots.add_relative());
    ui.add_span().triggered().connect(slots.add_span());
    ui.delete_block().triggered().connect(slots.delete_block());
    ui.delete_subtree().triggered().connect(slots.delete_subtree());
    ui.undo().triggered().connect(slots.undo());
    ui.redo().triggered().connect(slots.redo());

    ui.name_line_edit().editing_finished().connect(slots.formation_fields_changed());
    ui.ai_priority_spinbox().value_changed().connect(slots.formation_fields_changed());
    ui.uk_2_spinbox().value_changed().connect(slots.formation_fields_changed());
    ui.ai_purpose_model().item_changed().connect(slots.ai_purpose_changed());
    connect_list(ui.min_unit_category(), slots.min_unit_category_changed(), slots.add_min_unit_category(), slots.remove_min_unit_category());
    connect_list(ui.subcultures(), slots.subcultures_changed(), slots.add_subculture(), slots.remove_subculture());
    connect_list(ui.factions(), slots.factions_changed(), slots.add_faction(), slots.remove_faction());

    ui.parent_combobox().activated().connect(slots.parent_changed());
    ui.position_x_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.position_y_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.block_priority_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.arrangement_combobox().activated().connect(slots.container_fields_changed());
    ui.spacing_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.crescent_y_offset_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.minimum_threshold_spinbox().value_changed().connect(slots.container_fields_changed());
    ui.maximum_threshold_spinbox().value_changed().connect(slots.container_fields_changed());
    connect_list(ui.entity_preferences(), slots.entity_preferences_changed(), slots.add_entity_preference(), slots.remove_entity_preference());
    ui.span_members_model().item_changed().connect(slots.span_members_changed());

    ui.issues_list_view().clicked().connect(slots.issue_clicked());
}

/// Connects an inspector list to the slot saving it, and its buttons to the slots adding and removing rows.
///
/// Rows added by the view are ignored by the saving slot while it's loading, so only edits by the user save the list.
unsafe fn connect_list(list: &ItemList, changed: &QBox<SlotNoArgs>, add: &QBox<SlotNoArgs>, remove: &QBox<SlotNoArgs>) {
    list.model().data_changed().connect(changed);
    list.model().rows_inserted().connect(changed);
    list.model().rows_removed().connect(changed);
    list.add_button().released().connect(add);
    list.remove_button().released().connect(remove);
}
