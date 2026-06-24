//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Slots backing the chip-based filter bar.
//!
//! The bar-level slots (FilterBarSlots) are stored on the `TableView` and live as long as
//! the view itself. Each chip carries its own slots in `ChipSlots`, which are stored on
//! the chip so they're dropped when the chip is removed.

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQString;

use qt_gui::SlotOfQAction;

use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::utils::show_dialog;
use crate::views::table::FilterChipState;

use super::chip::Chip;
use super::{FilterBar, TableView};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Slots backing the filter bar itself (its input edit, add button, columns button, debounce timer).
pub struct FilterBarSlots {
    pub input_text_changed: QBox<SlotOfQString>,
    pub input_returned: QBox<SlotNoArgs>,
    pub input_debounce_fired: QBox<SlotNoArgs>,
    pub add_button_clicked: QBox<SlotNoArgs>,
    pub columns_button_clicked: QBox<SlotNoArgs>,
    pub help_button_clicked: QBox<SlotNoArgs>,
}

/// Per-chip slots; owned by the chip so they're released alongside the chip widget.
pub struct ChipSlots {
    pub value_text_changed: QBox<SlotOfQString>,
    pub debounce_fired: QBox<SlotNoArgs>,
    pub column_menu_triggered: QBox<SlotOfQAction>,
    pub options_changed: QBox<SlotNoArgs>,
    pub group_changed: QBox<SlotNoArgs>,
    pub remove_clicked: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FilterBarSlots {
    pub unsafe fn new(bar: &Arc<FilterBar>, view: &Arc<TableView>) -> Self {

        // Typing into the input restarts the debounce timer. We don't refilter live
        // off the input itself — the chip only materialises on Enter or button press.
        let input_text_changed = SlotOfQString::new(bar.main_widget(), clone!(
            bar => move |_| {
                bar.restart_input_debounce();
            }
        ));

        // Enter on the input promotes the typed text into a new chip and refilters.
        let input_returned = SlotNoArgs::new(bar.main_widget(), clone!(
            bar,
            view => move || {
                let raw = bar.input_line_edit().text().to_std_string();
                if raw.trim().is_empty() { return; }
                let state = bar.parse_input(&raw);
                match bar.add_chip(&view, state, false) {
                    Ok(_) => {
                        bar.input_line_edit().clear();
                        view.filter_table();
                    }
                    Err(err) => show_dialog(bar.main_widget(), err.to_string(), false),
                }
            }
        ));

        // Debounce timeout: live preview. We parse the typed text and filter the table
        // through it without materialising a chip, so results update as the user pauses.
        // An empty input reverts to filtering on the existing chips alone.
        let input_debounce_fired = SlotNoArgs::new(bar.main_widget(), clone!(
            bar,
            view => move || {
                let raw = bar.input_line_edit().text().to_std_string();
                if raw.trim().is_empty() {
                    view.filter_table();
                } else {
                    let state = bar.parse_input(&raw);
                    view.filter_table_with_preview(Some(&state));
                }
            }
        ));

        // The "+" button does the same as pressing Enter on the input, but also opens
        // an empty chip when the input is blank so the user can pick a column visually.
        let add_button_clicked = SlotNoArgs::new(bar.main_widget(), clone!(
            bar,
            view => move || {
                rpfm_telemetry::track_action("Table Filter: Add Chip");
                let raw = bar.input_line_edit().text().to_std_string();
                let state = if raw.trim().is_empty() {
                    FilterChipState::default()
                } else {
                    bar.parse_input(&raw)
                };
                match bar.add_chip(&view, state, false) {
                    Ok(_) => {
                        bar.input_line_edit().clear();
                        view.filter_table();
                    }
                    Err(err) => show_dialog(bar.main_widget(), err.to_string(), false),
                }
            }
        ));

        // Open the Columns popover. The popover widget is owned by TableView; it figures
        // out its own anchor from the filter bar's Columns button.
        let columns_button_clicked = SlotNoArgs::new(bar.main_widget(), clone!(
            view => move || {
                view.toggle_columns_popover();
            }
        ));

        // Help button — pop a small dialog with the predicate grammar reference. The body
        // is rich-text so the example snippets render as monospace.
        let help_button_clicked = SlotNoArgs::new(bar.main_widget(), clone!(
            bar => move || {
                use qt_widgets::QMessageBox;
                use qt_widgets::q_message_box::Icon;
                use qt_core::QString;
                use crate::utils::{qtr, tr};

                let box_ = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                    Icon::Information,
                    &qtr("filter_help_title"),
                    &QString::from_std_str(""),
                    qt_widgets::q_message_box::StandardButton::Ok.into(),
                    bar.main_widget(),
                );
                box_.set_text_format(qt_core::TextFormat::RichText);
                box_.set_text(&QString::from_std_str(tr("filter_help_body")));
                box_.exec();
            }
        ));

        Self {
            input_text_changed,
            input_returned,
            input_debounce_fired,
            add_button_clicked,
            columns_button_clicked,
            help_button_clicked,
        }
    }
}

impl ChipSlots {

    /// Build the slot bundle for `chip` and connect it to `view.filter_table()` for any
    /// change that should refilter.
    pub unsafe fn new(chip: &Arc<Chip>, view: &Arc<TableView>) -> Self {

        let value_text_changed = SlotOfQString::new(chip.main_widget(), clone!(
            chip => move |_| {
                chip.restart_debounce();
            }
        ));

        let debounce_fired = SlotNoArgs::new(chip.main_widget(), clone!(
            view => move || {
                view.filter_table();
            }
        ));

        // When the user picks a column from the chip's column menu, copy the action's
        // text onto the button and refilter.
        let column_menu_triggered = SlotOfQAction::new(chip.main_widget(), clone!(
            chip,
            view => move |action| {
                if action.is_null() { return; }
                let label = action.text().to_std_string();
                chip.set_column_label(&label);
                view.filter_table();
            }
        ));

        let options_changed = SlotNoArgs::new(chip.main_widget(), clone!(
            view => move || {
                view.filter_table();
            }
        ));

        let group_changed = SlotNoArgs::new(chip.main_widget(), clone!(
            view => move || {
                view.filter_table();
            }
        ));

        let remove_clicked = SlotNoArgs::new(chip.main_widget(), clone!(
            chip,
            view => move || {
                rpfm_telemetry::track_action("Table Filter: Remove Chip");
                if let Some(bar) = view.filter_bar_arc() {
                    bar.remove_chip(&view, &chip);
                    view.filter_table();
                }
            }
        ));

        Self {
            value_text_changed,
            debounce_fired,
            column_menu_triggered,
            options_changed,
            group_changed,
            remove_clicked,
        }
    }
}
