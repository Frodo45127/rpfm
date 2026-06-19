//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Signal/slot wiring for the filter bar and individual chips.

use qt_core::QBox;

use std::sync::Arc;

use qt_widgets::QWidget;

use crate::ffi::responsive_widget_resized_signal;

use super::FilterBar;
use super::TableView;
use super::chip::Chip;
use super::slots::{ChipSlots, FilterBarSlots};

/// Wire the filter bar's input + buttons + debounce timer.
pub unsafe fn set_connections_filter_bar(bar: &Arc<FilterBar>, slots: &FilterBarSlots) {
    bar.input_line_edit().text_changed().connect(&slots.input_text_changed);
    bar.input_line_edit().return_pressed().connect(&slots.input_returned);
    bar.timer_input_debounce().timeout().connect(&slots.input_debounce_fired);
    bar.add_button().released().connect(&slots.add_button_clicked);
    bar.columns_button().released().connect(&slots.columns_button_clicked);
    bar.help_button().released().connect(&slots.help_button_clicked);
    responsive_widget_resized_signal(bar.root().static_upcast()).connect(&slots.bar_resized);
}

/// Wire one chip's widgets and stash its slot bundle on the view so the slots survive.
///
/// `_bar_widget` is unused but kept in the signature for parity with how other widgets
/// receive their owner pointer — Qt parents handle lifetime.
pub unsafe fn set_connections_chip(chip: &Arc<Chip>, _bar_widget: &QBox<QWidget>, view: &Arc<TableView>) {
    let slots = ChipSlots::new(chip, view);

    chip.value_edit().text_changed().connect(&slots.value_text_changed);
    chip.timer_debounce().timeout().connect(&slots.debounce_fired);

    chip.column_menu().triggered().connect(&slots.column_menu_triggered);

    chip.not_action().toggled().connect(&slots.options_changed);
    chip.regex_action().toggled().connect(&slots.options_changed);
    chip.case_action().toggled().connect(&slots.options_changed);
    chip.show_blank_action().toggled().connect(&slots.options_changed);
    chip.show_edited_action().toggled().connect(&slots.options_changed);
    chip.variant_source_action().toggled().connect(&slots.options_changed);
    chip.variant_lookup_action().toggled().connect(&slots.options_changed);
    chip.variant_both_action().toggled().connect(&slots.options_changed);

    chip.group_spinbox().value_changed().connect(&slots.group_changed);

    chip.remove_button().released().connect(&slots.remove_clicked);

    // Stash the slot bundle on the view so it lives as long as the chip does.
    view.push_chip_slots(slots);
}
