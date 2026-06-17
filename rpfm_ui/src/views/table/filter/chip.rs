//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Compact "chip" widget used by the table view's filter bar.
//!
//! A chip represents a single filter predicate: a column, a value, and a set of flags
//! (NOT / regex / case / blank-only / edited-only / variant / group). The chip exposes
//! its state through a handful of small accessors so the parent `FilterBar` can collect
//! it when applying filters.

use qt_widgets::QFrame;
use qt_widgets::QHBoxLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMenu;
use qt_widgets::QSpinBox;
use qt_widgets::QToolButton;
use qt_widgets::QWidgetAction;
use qt_widgets::QWidget;
use qt_widgets::q_frame::Shape;
use qt_widgets::q_tool_button::ToolButtonPopupMode;

use qt_gui::QAction;
use qt_gui::QActionGroup;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QTimer;

use anyhow::Result;
use getset::Getters;

use crate::utils::{qtr, tr};

use super::FilterChipState;

const CHIP_DEBOUNCE_MS: i32 = 400;

/// Stylesheet applied to chip frames. The chip is a compact pill split into three zones
/// with distinct backgrounds so column label, value field, and action buttons read as
/// separate affordances at a glance:
///   - Column label: button-coloured, slight hover; clearly clickable.
///   - Value edit: base-coloured input with focus ring.
///   - Action buttons: flat, hover-tinted.
const CHIP_STYLE: &str = r#"
QFrame#chip_frame {
    border: 1px solid palette(mid);
    border-radius: 8px;
    background: palette(window);
}
QFrame#chip_frame QToolButton#chip_column {
    background: palette(button);
    border: 1px solid palette(mid);
    border-radius: 5px;
    padding: 0px 6px;
    font-weight: 600;
}
QFrame#chip_frame QToolButton#chip_column:hover {
    background: palette(midlight);
}
QFrame#chip_frame QToolButton#chip_column::menu-indicator {
    image: none;
    width: 0;
    height: 0;
}
QFrame#chip_frame QLineEdit#chip_value {
    background: palette(base);
    border: 1px solid palette(mid);
    border-radius: 4px;
    padding: 0px 4px;
}
QFrame#chip_frame QLineEdit#chip_value:focus {
    border-color: palette(highlight);
}
QFrame#chip_frame QToolButton#chip_options,
QFrame#chip_frame QToolButton#chip_remove {
    background: transparent;
    border: none;
    padding: 1px 3px;
}
QFrame#chip_frame QToolButton#chip_options:hover,
QFrame#chip_frame QToolButton#chip_remove:hover {
    background: palette(midlight);
    border-radius: 3px;
}
"#;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Holder for a single chip's widgets + the flag actions that drive its options menu.
///
/// The chip is laid out horizontally:
/// `[ column ▼ ] [ value …       ] [ ⚙ ] [ × ]`
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Chip {
    main_widget: QBox<QFrame>,

    column_button: QPtr<QToolButton>,
    value_edit: QPtr<QLineEdit>,
    #[getset(skip)]
    _options_button: QPtr<QToolButton>,
    remove_button: QPtr<QToolButton>,

    not_action: QPtr<QAction>,
    regex_action: QPtr<QAction>,
    case_action: QPtr<QAction>,
    show_blank_action: QPtr<QAction>,
    show_edited_action: QPtr<QAction>,

    variant_source_action: QPtr<QAction>,
    variant_lookup_action: QPtr<QAction>,
    variant_both_action: QPtr<QAction>,

    group_spinbox: QPtr<QSpinBox>,

    column_menu: QBox<QMenu>,
    #[getset(skip)]
    _options_menu: QBox<QMenu>,

    timer_debounce: QBox<QTimer>,

    #[getset(skip)]
    _columns_list: QBox<QStandardItemModel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Chip {

    /// Build a new chip, parent it to `parent`, and seed it from `state`.
    ///
    /// # Arguments
    ///
    /// * `parent` — the chip will be parented here and inserted into its layout by the caller.
    /// * `column_names` — display names for the column picker, in the user's preferred order
    ///   (typically `fields_processed_sorted`).
    /// * `logical_indices` — parallel to `column_names`; entry `i` is the **logical** model
    ///   column index for the `i`-th display name. This is what gets stored as the chip's
    ///   `column_index` and what the C++ proxy filters by. May be `-1` for "Any".
    /// * `state` — initial chip state. `state.column_index` is interpreted as a logical
    ///   model column index (or -1 for "Any").
    pub unsafe fn new(parent: &QBox<QWidget>, column_names: &[String], logical_indices: &[i32], state: &FilterChipState) -> Result<Self> {
        assert_eq!(column_names.len(), logical_indices.len(), "chip: column_names and logical_indices length mismatch");
        let main_widget = QFrame::new_1a(parent);
        main_widget.set_object_name(&QString::from_std_str("chip_frame"));
        main_widget.set_frame_shape(Shape::NoFrame);
        main_widget.set_style_sheet(&QString::from_std_str(CHIP_STYLE));

        let layout = QHBoxLayout::new_1a(&main_widget);
        layout.set_contents_margins_4a(3, 1, 3, 1);
        layout.set_spacing(3);

        // Column picker. Shows the chosen column name; clicking opens a menu with every
        // column so the user can re-target the chip.
        let column_button = QToolButton::new_1a(&main_widget);
        column_button.set_object_name(&QString::from_std_str("chip_column"));
        column_button.set_popup_mode(ToolButtonPopupMode::InstantPopup);
        column_button.set_auto_raise(false);
        layout.add_widget(&column_button);

        let column_menu = QMenu::from_q_widget(&main_widget);
        let columns_list = QStandardItemModel::new_1a(&column_menu);

        // The "Any" entry maps to column_index = -1, meaning the C++ proxy will scan all columns.
        let any_action = column_menu.add_action_q_string(&qtr("filter_chip_column_any"));
        any_action.set_data(&qt_core::QVariant::from_int(-1));

        column_menu.add_separator();

        // Each menu action carries the LOGICAL column index in its `data`, not the
        // sorted position — that way `current_column_index()` returns something the
        // C++ proxy can use directly.
        for (sorted_idx, name) in column_names.iter().enumerate() {
            let act = column_menu.add_action_q_string(&QString::from_std_str(name));
            act.set_data(&qt_core::QVariant::from_int(logical_indices[sorted_idx]));
        }

        column_button.set_menu(&column_menu);

        // Set the displayed column name based on state. `state.column_index` is a logical
        // index, so we have to find the *sorted* slot whose logical index matches.
        let column_label = if state.column_index < 0 {
            tr("filter_chip_column_any")
        } else {
            logical_indices.iter().position(|i| *i == state.column_index)
                .and_then(|sorted_idx| column_names.get(sorted_idx).cloned())
                .unwrap_or_else(|| tr("filter_chip_column_any"))
        };
        column_button.set_text(&QString::from_std_str(&column_label));

        // Value edit.
        let value_edit = QLineEdit::from_q_widget(&main_widget);
        value_edit.set_object_name(&QString::from_std_str("chip_value"));
        value_edit.set_placeholder_text(&qtr("table_filter"));
        value_edit.set_text(&QString::from_std_str(&state.pattern));
        value_edit.set_clear_button_enabled(false);
        value_edit.set_minimum_width(70);
        layout.add_widget(&value_edit);

        // Options menu, hung off a small "⚙" button.
        let options_button = QToolButton::new_1a(&main_widget);
        options_button.set_object_name(&QString::from_std_str("chip_options"));
        options_button.set_popup_mode(ToolButtonPopupMode::InstantPopup);
        options_button.set_auto_raise(true);
        options_button.set_text(&QString::from_std_str("\u{2699}")); // gear
        options_button.set_tool_tip(&qtr("filter_chip_options"));
        layout.add_widget(&options_button);

        let options_menu = QMenu::from_q_widget(&main_widget);

        let not_action = options_menu.add_action_q_string(&qtr("filter_chip_not"));
        not_action.set_checkable(true);
        not_action.set_checked(state.not);

        let regex_action = options_menu.add_action_q_string(&qtr("filter_chip_regex"));
        regex_action.set_checkable(true);
        regex_action.set_checked(state.regex);

        let case_action = options_menu.add_action_q_string(&qtr("filter_chip_case_sensitive"));
        case_action.set_checkable(true);
        case_action.set_checked(state.case_sensitive);

        options_menu.add_separator();

        let show_blank_action = options_menu.add_action_q_string(&qtr("filter_chip_show_blank"));
        show_blank_action.set_checkable(true);
        show_blank_action.set_checked(state.show_blank);

        let show_edited_action = options_menu.add_action_q_string(&qtr("filter_chip_show_edited"));
        show_edited_action.set_checkable(true);
        show_edited_action.set_checked(state.show_edited);

        options_menu.add_separator();

        // Variant: source / lookup / both, mutually exclusive.
        let variant_group = QActionGroup::new(&options_menu);
        variant_group.set_exclusive(true);

        let variant_source_action = options_menu.add_action_q_string(&qtr("filter_chip_variant_source"));
        variant_source_action.set_checkable(true);
        variant_group.add_action_q_action(&variant_source_action);

        let variant_lookup_action = options_menu.add_action_q_string(&qtr("filter_chip_variant_lookup"));
        variant_lookup_action.set_checkable(true);
        variant_group.add_action_q_action(&variant_lookup_action);

        let variant_both_action = options_menu.add_action_q_string(&qtr("filter_chip_variant_both"));
        variant_both_action.set_checkable(true);
        variant_group.add_action_q_action(&variant_both_action);

        match state.variant {
            1 => variant_lookup_action.set_checked(true),
            2 => variant_both_action.set_checked(true),
            _ => variant_source_action.set_checked(true),
        }

        options_menu.add_separator();

        // Group spinbox lives inside a QWidgetAction so it can be embedded in the menu.
        let group_widget = QWidget::new_1a(&options_menu);
        let group_layout = QHBoxLayout::new_1a(&group_widget);
        group_layout.set_contents_margins_4a(8, 2, 8, 2);
        let group_label = QLabel::from_q_string_q_widget(&qtr("filter_chip_group_label"), &group_widget);
        group_layout.add_widget(&group_label);
        let group_spinbox = QSpinBox::new_1a(&group_widget);
        group_spinbox.set_minimum(0);
        group_spinbox.set_maximum(99);
        group_spinbox.set_value(state.group.max(0));
        group_layout.add_widget(&group_spinbox);
        let group_action = QWidgetAction::new(&options_menu);
        group_action.set_default_widget(&group_widget);
        options_menu.add_action_q_action(&group_action);

        options_button.set_menu(&options_menu);

        // Remove button.
        let remove_button = QToolButton::new_1a(&main_widget);
        remove_button.set_object_name(&QString::from_std_str("chip_remove"));
        remove_button.set_text(&QString::from_std_str("\u{2715}"));
        remove_button.set_tool_tip(&qtr("filter_chip_remove"));
        remove_button.set_auto_raise(true);
        layout.add_widget(&remove_button);

        // Debounce timer for the value edit so we don't refilter on every keystroke.
        let timer_debounce = QTimer::new_1a(&main_widget);
        timer_debounce.set_single_shot(true);
        timer_debounce.set_interval(CHIP_DEBOUNCE_MS);

        // Convert programmatically-built QBoxes into QPtrs so they live as long as their parent
        // (the chip's main_widget). The action handles returned by add_action_q_string are
        // already QPtrs, so no conversion is needed for those.
        let column_button: QPtr<QToolButton> = column_button.into_q_ptr();
        let value_edit: QPtr<QLineEdit> = value_edit.into_q_ptr();
        let options_button: QPtr<QToolButton> = options_button.into_q_ptr();
        let remove_button: QPtr<QToolButton> = remove_button.into_q_ptr();
        let group_spinbox: QPtr<QSpinBox> = group_spinbox.into_q_ptr();

        Ok(Self {
            main_widget,
            column_button,
            value_edit,
            _options_button: options_button,
            remove_button,
            not_action,
            regex_action,
            case_action,
            show_blank_action,
            show_edited_action,
            variant_source_action,
            variant_lookup_action,
            variant_both_action,
            group_spinbox,
            column_menu,
            _options_menu: options_menu,
            timer_debounce,
            _columns_list: columns_list,
        })
    }

    /// Read the current state out of the widgets so it can be applied to the proxy or persisted.
    pub unsafe fn current_state(&self) -> FilterChipState {
        let column_index = self.current_column_index();
        let pattern = self.value_edit.text().to_std_string();
        let not = self.not_action.is_checked();
        let regex = self.regex_action.is_checked();
        let case_sensitive = self.case_action.is_checked();
        let show_blank = self.show_blank_action.is_checked();
        let show_edited = self.show_edited_action.is_checked();
        let variant = if self.variant_lookup_action.is_checked() { 1 }
            else if self.variant_both_action.is_checked() { 2 }
            else { 0 };
        let group = self.group_spinbox.value();

        FilterChipState { column_index, pattern, not, regex, case_sensitive, show_blank, show_edited, variant, group }
    }

    /// Currently-selected column index, or `-1` for "Any".
    pub unsafe fn current_column_index(&self) -> i32 {
        let current_text = self.column_button.text();
        let actions = self.column_menu.actions();
        for i in 0..actions.count() {
            let action = actions.value_1a(i);
            if !action.is_null() && action.text().compare_q_string(&current_text) == 0 {
                let data = action.data();
                if data.is_valid() {
                    return data.to_int_0a();
                }
            }
        }
        -1
    }

    /// Restart the debounce timer (called on every keystroke in the value edit).
    pub unsafe fn restart_debounce(&self) {
        self.timer_debounce.start_0a();
    }

    /// Update the column button text in response to the user picking a column from the menu.
    pub unsafe fn set_column_label(&self, label: &str) {
        self.column_button.set_text(&QString::from_std_str(label));
    }
}
