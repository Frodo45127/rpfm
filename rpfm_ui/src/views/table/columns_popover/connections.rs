//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Signal/slot wiring for the Columns popover.

use std::sync::Arc;

use super::ColumnsPopover;
use super::slots::ColumnsPopoverSlots;

/// Connect the popover's search field, master buttons, and per-row move buttons to slots.
///
/// The per-row Hide/Freeze checkbox connections are made in
/// `views::table::connections::set_connections` since those slots live on `TableView` and
/// have always done so — moving them would create needless churn.
pub unsafe fn set_connections_columns_popover(popover: &Arc<ColumnsPopover>, slots: &ColumnsPopoverSlots) {
    popover.search_edit().text_changed().connect(&slots.search_text_changed);
    popover.hide_all_checkbox().state_changed().connect(&slots.hide_all_state_changed);
    popover.freeze_all_checkbox().state_changed().connect(&slots.freeze_all_state_changed);

    for (row, slot) in popover.rows().iter().zip(slots.move_up.iter()) {
        row.move_up_button().released().connect(slot);
    }
    for (row, slot) in popover.rows().iter().zip(slots.move_down.iter()) {
        row.move_down_button().released().connect(slot);
    }
}
