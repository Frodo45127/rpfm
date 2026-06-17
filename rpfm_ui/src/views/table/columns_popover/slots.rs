//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Slots for the Columns popover (search field, master buttons, per-row move-up/down).

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfInt;
use qt_core::SlotOfQString;

use std::sync::Arc;

use rpfm_ui_common::clone;

use super::ColumnsPopover;
use crate::views::table::TableView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

pub struct ColumnsPopoverSlots {
    pub search_text_changed: QBox<SlotOfQString>,
    pub hide_all_state_changed: QBox<SlotOfInt>,
    pub freeze_all_state_changed: QBox<SlotOfInt>,
    pub move_up: Vec<QBox<SlotNoArgs>>,
    pub move_down: Vec<QBox<SlotNoArgs>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ColumnsPopoverSlots {

    /// Build all popover-level slots.
    ///
    /// Per-row hide/freeze checkbox slots are *not* built here — those live in
    /// `TableViewSlots` (as `hide_show_columns` / `freeze_columns`) and are connected to
    /// the checkboxes regardless of where they live in the widget tree. Only the search,
    /// master buttons, and move buttons are new behavior owned by the popover.
    pub unsafe fn new(popover: &Arc<ColumnsPopover>, view: &Arc<TableView>) -> Self {

        let search_text_changed = SlotOfQString::new(popover.main_widget(), clone!(
            popover => move |text| {
                let needle = text.to_std_string();
                popover.apply_search_filter(&needle);
            }
        ));

        let hide_all_state_changed = SlotOfInt::new(popover.main_widget(), clone!(
            popover => move |state| {
                let checked = state == 2;
                for row in popover.rows() {
                    row.hide_checkbox().set_checked(checked);
                }
            }
        ));

        let freeze_all_state_changed = SlotOfInt::new(popover.main_widget(), clone!(
            popover => move |state| {
                let checked = state == 2;
                for row in popover.rows() {
                    row.freeze_checkbox().set_checked(checked);
                }
            }
        ));

        // Build per-row move-up / move-down slots. Each closure captures the row's logical
        // index (snapshotted at popover-build time) and asks the table view's horizontal
        // header to slide the column by one visual position. It also calls back into the
        // popover to reorder the visible row list so the popover stays in sync with the
        // table.
        let mut move_up = Vec::with_capacity(popover.rows().len());
        let mut move_down = Vec::with_capacity(popover.rows().len());

        for (row_index, row) in popover.rows().iter().enumerate() {
            let logical_index = *row.logical_index();

            let up_slot = SlotNoArgs::new(popover.main_widget(), clone!(
                view,
                popover => move || {
                    move_column_by(&view, &popover, row_index, logical_index, -1);
                }
            ));
            move_up.push(up_slot);

            let down_slot = SlotNoArgs::new(popover.main_widget(), clone!(
                view,
                popover => move || {
                    move_column_by(&view, &popover, row_index, logical_index, 1);
                }
            ));
            move_down.push(down_slot);
        }

        Self {
            search_text_changed,
            hide_all_state_changed,
            freeze_all_state_changed,
            move_up,
            move_down,
        }
    }
}

/// Slide `logical_index` by `delta` visual positions on the underlying table header and
/// keep the popover's visible row list in sync.
unsafe fn move_column_by(view: &Arc<TableView>, popover: &Arc<super::ColumnsPopover>, row_index: usize, logical_index: i32, delta: i32) {
    let header = view.table_view().horizontal_header();
    let visual = header.visual_index(logical_index);
    let target = visual + delta;
    if target < 0 || target >= header.count() { return; }
    header.move_section(visual, target);

    // Mirror the move in the popover row layout so the visible list matches.
    if let Some(row) = popover.rows().get(row_index) {
        popover.move_row_in_layout(row.row_widget(), delta);
    }
}
