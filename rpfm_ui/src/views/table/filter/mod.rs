//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! TableView submodule providing the chip-based filter bar.
//!
//! The filter bar is a single horizontal bar containing zero or more *chips*. Each chip
//! represents one filter predicate (column + value + flags) and is rendered as a compact
//! rounded widget. The bar also exposes an input line edit so the user can type a
//! predicate string and press Enter to spawn a new chip, plus a "Columns" button that
//! opens the column-management popover.

use qt_widgets::QHBoxLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QScrollArea;
use qt_widgets::QToolButton;
use qt_widgets::QVBoxLayout;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QTimer;

use anyhow::{anyhow, Result};
use getset::Getters;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rpfm_ipc::settings_keys::*;
use rpfm_ui_common::utils::{find_widget, load_template};

use crate::ffi::new_responsive_widget_safe;
use crate::settings_ui::backend::settings_bool;
use crate::utils::qtr;
use crate::views::table::clean_column_names;

use self::chip::Chip;
use self::slots::FilterBarSlots;
use super::{FilterChipState, TableView};

pub mod chip;
mod connections;
pub mod slots;

#[cfg(test)] mod test;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/table_filter_groupbox.ui";
const VIEW_RELEASE: &str = "ui/table_filter_groupbox.ui";

const INPUT_DEBOUNCE_MS: i32 = 250;

/// The chips move to their own full-width row as soon as their content no longer fits beside
/// the controls inline (i.e. before a horizontal scrollbar would appear), and fold back inline
/// once there's `STACK_WIDTH_HYSTERESIS` px of slack again. The gap stops a width sitting right
/// on the boundary from thrashing the layout.
const STACK_WIDTH_HYSTERESIS: i32 = 40;

/// Extra px reserved for inter-widget spacing, margins and the separator when checking whether
/// the chips fit inline — so we stack just before, not just after, a scrollbar would show.
const CHIPS_FIT_SLACK: i32 = 40;

/// Index of the chips scroll area within the inline (controls) row, right after the
/// columns separator. Used to restore its position when folding back inline.
const CHIPS_INLINE_INDEX: i32 = 3;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// The chip-based filter bar widget tree owned by a `TableView`.
///
/// Exactly one `FilterBar` exists per table view; chips are added/removed at runtime
/// into the `chips_container` widget.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct FilterBar {
    /// Resize-aware container holding `main_widget`. This is what gets added to the table
    /// view's layout; its `resized` signal drives the chip reflow.
    root: QBox<QWidget>,

    main_widget: QBox<QWidget>,

    columns_button: QPtr<QToolButton>,
    flagged_rows_button: QPtr<QToolButton>,
    #[getset(skip)]
    _chips_scroll: QPtr<QScrollArea>,
    chips_container: QPtr<QWidget>,
    input_line_edit: QPtr<QLineEdit>,
    add_button: QPtr<QToolButton>,
    help_button: QPtr<QToolButton>,
    row_counter_label: QPtr<qt_widgets::QLabel>,

    timer_input_debounce: QBox<QTimer>,

    /// Cleaned column names in the order shown in chip column pickers (sorted display order).
    column_names: Vec<String>,

    /// Logical column indices corresponding to `column_names`.
    logical_indices: Vec<i32>,

    /// Whether the chips currently live on their own row (true) or inline (false).
    #[getset(skip)]
    chips_stacked: AtomicBool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FilterBar {

    /// Build the filter bar inside `view.filter_base_widget` and seed it with whatever
    /// chips the table's `TableView::initial_filter_chips()` (if any) returns.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; the bar is stored on the view via `view.set_filter_bar`.
    /// `Err(_)` if the .ui template fails to load.
    pub unsafe fn new(view: &Arc<TableView>) -> Result<()> {
        let parent = view.filter_base_widget_ptr();
        let parent_grid: qt_core::QPtr<qt_widgets::QGridLayout> = parent.layout().static_downcast();

        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        // Wrap the loaded bar in a resize-aware container with a vertical layout. The bar's
        // controls row sits on top; when the bar gets too narrow the chips area is moved out
        // of the controls row into a second row of this layout (see `apply_responsive_layout`).
        let root = new_responsive_widget_safe(&view.filter_base_widget_ptr().as_ptr());
        let outer_layout = QVBoxLayout::new_1a(&root);
        outer_layout.set_contents_margins_4a(0, 0, 0, 0);
        outer_layout.set_spacing(2);
        outer_layout.add_widget(&main_widget);

        // load_template puts the widget in a stray layout if the parent already has one;
        // re-attach the container to the real grid so it shows up in the table view's layout.
        parent_grid.add_widget_5a(&root, parent_grid.row_count(), 0, 1, 2);

        let columns_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "columns_button")?;
        let flagged_rows_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "flagged_rows_button")?;
        let chips_scroll: QPtr<QScrollArea> = find_widget(&main_widget.static_upcast(), "chips_scroll")?;
        let chips_container: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "chips_container")?;
        let input_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "input_line_edit")?;
        let add_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "add_button")?;
        let help_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "help_button")?;
        let row_counter_label: QPtr<qt_widgets::QLabel> = find_widget(&main_widget.static_upcast(), "row_counter_label")?;

        input_line_edit.set_placeholder_text(&qtr("filter_bar_placeholder"));
        input_line_edit.set_clear_button_enabled(true);
        add_button.set_tool_tip(&qtr("filter_bar_add_chip"));
        columns_button.set_tool_tip(&qtr("columns_popover_open"));
        help_button.set_tool_tip(&qtr("filter_bar_help"));
        flagged_rows_button.set_tool_tip(&qtr("table_filter_show_flagged_rows_tip"));

        // Force the row counter text from code as well; the .ui sets it but we want
        // to be sure nothing in the load path leaves it empty.
        row_counter_label.set_text(&qt_core::QString::from_std_str("0/0"));

        let timer_input_debounce = QTimer::new_1a(&main_widget);
        timer_input_debounce.set_single_shot(true);
        timer_input_debounce.set_interval(INPUT_DEBOUNCE_MS);

        // Cache cleaned column names plus their logical model indices.
        let (column_names, logical_indices) = {
            let definition = view.table_definition();
            let fields_sorted = definition.fields_processed_sorted(settings_bool(TABLES_USE_OLD_COLUMN_ORDER));
            let fields_processed = definition.fields_processed();
            let mut column_names = Vec::with_capacity(fields_sorted.len());
            let mut logical_indices = Vec::with_capacity(fields_sorted.len());
            for field in &fields_sorted {
                column_names.push(clean_column_names(field.name()));
                let logical = fields_processed.iter().position(|f| f == field).unwrap_or(0) as i32;
                logical_indices.push(logical);
            }
            (column_names, logical_indices)
        };

        let filter_bar = Arc::new(Self {
            root,
            main_widget,
            columns_button,
            flagged_rows_button,
            _chips_scroll: chips_scroll,
            chips_container,
            input_line_edit,
            add_button,
            help_button,
            row_counter_label,
            timer_input_debounce,
            column_names,
            logical_indices,
            chips_stacked: AtomicBool::new(false),
        });

        let slots = FilterBarSlots::new(&filter_bar, view);
        connections::set_connections_filter_bar(&filter_bar, &slots);

        view.set_filter_bar(filter_bar);
        Ok(())
    }

    /// Build a chip from `state`, append it to the bar, wire its signals.
    ///
    /// # Returns
    ///
    /// The new chip on success. Triggers no filter pass on its own — the caller (a slot)
    /// will call `view.filter_table()` once it has installed the chip.
    pub unsafe fn add_chip(&self, view: &Arc<TableView>, state: FilterChipState) -> Result<Arc<Chip>> {
        let chip = Arc::new(Chip::new(&self.main_widget, &self.column_names, &self.logical_indices, &state)?);

        // Insert the chip's widget at the end of the chips layout, before any trailing stretch.
        let layout = self.chips_container.layout();
        if layout.is_null() {
            return Err(anyhow!("chips_container has no layout; .ui template is malformed"));
        }
        let hlayout: QPtr<QHBoxLayout> = layout.static_downcast();
        hlayout.add_widget(chip.main_widget());

        connections::set_connections_chip(&chip, &self.main_widget, view);

        view.filter_chips_mut().push(chip.clone());

        // One more chip raises the width the bar needs; restack if it no longer fits.
        self.apply_responsive_layout(self.root.width(), view.filter_chips().len() as i32);
        Ok(chip)
    }

    /// Remove the chip whose widget pointer matches `chip`, returning whether anything
    /// was removed.
    ///
    /// We intentionally do **not** call `delete_later()` on the chip widget: doing so
    /// from inside the remove button's signal handler races with Qt destroying the slot
    /// wrappers parented to the same widget and crashes the process. Instead we hide
    /// the widget and detach it from the layout, matching the long-standing pattern in
    /// the old `FilterView` code. The widget is reaped when the filter bar itself dies.
    pub unsafe fn remove_chip(&self, view: &Arc<TableView>, chip: &Arc<Chip>) -> bool {
        let mut chips = view.filter_chips_mut();
        let target = chip.main_widget().as_ptr().as_raw_ptr();
        let pos = chips.iter().position(|c| c.main_widget().as_ptr().as_raw_ptr() == target);
        if let Some(pos) = pos {
            let removed = chips.remove(pos);
            self.chips_container.layout().remove_widget(removed.main_widget().as_ptr());
            removed.main_widget().hide();

            // One fewer chip lowers the width the bar needs; fold back inline if it now fits.
            let count = chips.len() as i32;
            drop(chips);
            self.apply_responsive_layout(self.root.width(), count);
            true
        } else {
            false
        }
    }

    /// Restart the debounce timer attached to the input line edit.
    pub unsafe fn restart_input_debounce(&self) {
        self.timer_input_debounce.start_0a();
    }

    /// Reflow the chips depending on the bar's available `width` and how many chips there are.
    ///
    /// The chips share the controls row only while their full content fits beside the controls
    /// (with the input shrunk to its minimum); otherwise they move to their own full-width row
    /// so the filter section never needs a horizontal scrollbar. Hysteresis around the threshold
    /// avoids thrashing on small resizes.
    pub unsafe fn apply_responsive_layout(&self, width: i32, chip_count: i32) {
        let stacked = self.chips_stacked.load(Ordering::Relaxed);

        let should_stack = if chip_count == 0 {
            false
        } else {
            // Width the non-chip controls need, with the input at its minimum, plus the chips'
            // real content width. If that doesn't fit, the chips would otherwise scroll.
            self.chips_container.layout().activate();
            let controls = self.columns_button.size_hint().width()
                + self.flagged_rows_button.size_hint().width()
                + self.input_line_edit.minimum_width()
                + self.add_button.size_hint().width()
                + self.help_button.size_hint().width()
                + self.row_counter_label.size_hint().width();
            let threshold = controls + self.chips_container.size_hint().width() + CHIPS_FIT_SLACK;

            if stacked {
                width < threshold + STACK_WIDTH_HYSTERESIS
            } else {
                width < threshold
            }
        };

        if should_stack == stacked {
            return;
        }

        let chips_widget = self._chips_scroll.static_upcast::<QWidget>();
        let bar_layout: QPtr<QHBoxLayout> = self.main_widget.layout().static_downcast();
        let outer_layout: QPtr<QVBoxLayout> = self.root.layout().static_downcast();

        if should_stack {
            bar_layout.remove_widget(chips_widget.as_ptr());
            outer_layout.add_widget(chips_widget.as_ptr());

            // On its own row the scroll area gets spare vertical space and (being
            // widgetResizable) stretches its content to fill it, making the chips grow
            // taller. Pin it to a single chip-row height so they keep their normal size.
            self._chips_scroll.set_maximum_height(self.chips_container.size_hint().height());
        } else {
            self._chips_scroll.set_maximum_height(16777215);
            outer_layout.remove_widget(chips_widget.as_ptr());
            bar_layout.insert_widget_2a(CHIPS_INLINE_INDEX, chips_widget.as_ptr());
        }

        self.chips_stacked.store(should_stack, Ordering::Relaxed);
    }

    /// Parse the text in the input line edit into a `FilterChipState`. Always succeeds
    /// (even on empty input — caller is expected to discard empty chips), but the parser
    /// is permissive: unknown flags are ignored, malformed `column:value` is treated as
    /// a literal value with no column scope.
    ///
    /// The returned `column_index` is a **logical** model column index (or -1 for "Any").
    ///
    /// Grammar (whitespace-insensitive, all parts optional except value):
    /// `[!] [column:]value [/i] [/r] [/s] [/lookup|/source|/both] [@group]`
    pub fn parse_input(&self, raw: &str) -> FilterChipState {
        parse_predicate(raw, &self.column_names, &self.logical_indices)
    }
}

/// Standalone parser kept outside `impl` so it can be unit-tested without a Qt context.
///
/// `column_names` and `logical_indices` are parallel slices; entry `i` in `logical_indices`
/// is the logical model column index for the display name at `column_names[i]`.
fn parse_predicate(raw: &str, column_names: &[String], logical_indices: &[i32]) -> FilterChipState {
    let mut state = FilterChipState {
        column_index: -1,
        pattern: String::new(),
        not: false,
        regex: true,           // matches the legacy default of "regex on"
        case_sensitive: false,
        show_blank: false,
        show_edited: false,
        variant: 0,
        group: 0,
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return state;
    }

    let mut remaining = trimmed.to_string();

    if let Some(stripped) = remaining.strip_prefix('!') {
        state.not = true;
        remaining = stripped.trim_start().to_string();
    }

    // Pull off trailing flag tokens (anything starting with '/' or '@') from the right.
    // We walk tokens from the right; everything that isn't a flag joins back into the value.
    let mut tokens: Vec<&str> = remaining.split_whitespace().collect();
    while let Some(last) = tokens.last().copied() {
        if let Some(rest) = last.strip_prefix('/') {
            match rest {
                "i" | "I" => state.case_sensitive = false,
                "s" | "S" => state.case_sensitive = true,
                "r" | "R" => state.regex = true,
                "n" | "N" => state.regex = false,
                "source" => state.variant = 0,
                "lookup" => state.variant = 1,
                "both" => state.variant = 2,
                "blank" | "empty" => state.show_blank = true,
                "edited" => state.show_edited = true,
                _ => break,
            }
            tokens.pop();
            continue;
        }
        if let Some(rest) = last.strip_prefix('@') {
            if let Ok(n) = rest.parse::<i32>() {
                state.group = n;
                tokens.pop();
                continue;
            }
        }
        break;
    }
    remaining = tokens.join(" ");

    // Split on the first ':' for column scoping. If the prefix doesn't match a column,
    // treat the whole thing as the value (so URLs etc. work).
    if let Some(colon_pos) = remaining.find(':') {
        let (col_part, value_part) = remaining.split_at(colon_pos);
        let col_part = col_part.trim();
        let value_part = value_part[1..].trim_start();
        if !col_part.is_empty() {
            if let Some(sorted_idx) = column_names.iter().position(|c| c.eq_ignore_ascii_case(col_part)) {
                // Translate display position to logical model column index.
                state.column_index = logical_indices.get(sorted_idx).copied().unwrap_or(-1);
                state.pattern = value_part.to_string();
                return state;
            }
        }
    }

    state.pattern = remaining;
    state
}
