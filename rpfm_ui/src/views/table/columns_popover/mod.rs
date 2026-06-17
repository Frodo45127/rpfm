//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Popover-style replacement for the table view's column sidebar.
//!
//! Shown by clicking the "Columns" button on the filter bar; closes when the user clicks
//! outside (via the `Qt::Popup` window flag). Contains a search field at the top and one
//! row per column with: name label, Hide checkbox, Freeze checkbox, move-up/move-down
//! buttons. The hide/freeze checkboxes are the *same* widgets as the legacy sidebar so
//! the existing slot bodies in `TableView::slots` keep working unchanged.

use qt_widgets::QCheckBox;
use qt_widgets::QFrame;
use qt_widgets::QHBoxLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QScrollArea;
use qt_widgets::QToolButton;
use qt_widgets::QVBoxLayout;
use qt_widgets::QWidget;
use qt_widgets::q_frame::Shape;

use qt_core::AlignmentFlag;
use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::QFlags;
use qt_core::QPoint;
use qt_core::QPropertyAnimation;
use qt_core::QPtr;
use qt_core::q_abstract_animation::DeletionPolicy;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::ScrollBarPolicy;
use qt_core::WidgetAttribute;
use qt_core::WindowType;

use anyhow::Result;
use getset::Getters;

use crate::utils::qtr;

pub mod slots;
pub mod connections;

const POPOVER_MIN_WIDTH: i32 = 360;
const POPOVER_MIN_HEIGHT: i32 = 240;
const POPOVER_MAX_HEIGHT: i32 = 480;

/// Vertical gap between the anchor button and the popover's bottom edge — just enough
/// breathing room so the card doesn't visually touch the button.
const POPOVER_ANCHOR_GAP: i32 = 8;

/// Duration of the slide-up animation in milliseconds.
const POPOVER_ANIM_MS: i32 = 160;

/// Card stylesheet — rounded background + 1 px border.
const CARD_STYLE: &str = r#"
QFrame#popover_card {
    background: palette(window);
    border: 1px solid palette(mid);
    border-radius: 8px;
}
"#;

/// Fixed width of the per-row hide/freeze checkbox cells. Wide enough to fit the legend
/// labels "Hide" / "Freeze" so titles can be centered over the checkbox columns.
const CHECKBOX_CELL_WIDTH: i32 = 56;

/// Fixed width of the move-up / move-down columns.
const MOVE_BUTTON_WIDTH: i32 = 26;

/// Approximate width of a Qt scroll bar in the default style. The scroll area inside
/// the card reserves this much vertical real estate for its scrollbar; the legend /
/// master rows above (which live directly on the card) need the same right inset so
/// their checkbox columns line up with the scroll-area rows' checkbox columns.
const SCROLLBAR_RESERVE_PX: i32 = 17;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// All widgets belonging to the Columns popover.
///
/// The popover is a top-level `QWidget` with `Qt::Popup` window flag: it auto-closes when
/// the user clicks outside its bounds.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct ColumnsPopover {
    /// Top-level frameless host. Translucent background — only the inner card and the
    /// arrow draw any visible pixels.
    main_widget: QBox<QWidget>,

    /// Styled card that holds all popover content. Visually the "popover" to the user.
    /// Held for ownership; readers go through the inner widgets directly.
    #[getset(skip)]
    _card: QBox<QFrame>,

    search_edit: QPtr<QLineEdit>,

    hide_all_checkbox: QBox<QCheckBox>,
    freeze_all_checkbox: QBox<QCheckBox>,

    /// Layout that hosts the column rows.
    rows_layout: QPtr<QVBoxLayout>,

    /// Per-column widgets, indexed by the same order as the column-picker fields.
    rows: Vec<ColumnRow>,
}

/// One row inside the popover, representing a single column.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct ColumnRow {
    row_widget: QBox<QWidget>,
    name_label: QPtr<QLabel>,
    hide_checkbox: QBox<QCheckBox>,
    freeze_checkbox: QBox<QCheckBox>,
    move_up_button: QPtr<QToolButton>,
    move_down_button: QPtr<QToolButton>,

    /// Logical column index this row corresponds to (matches `fields_processed` order).
    logical_index: i32,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ColumnsPopover {

    /// Build the popover widget tree.
    ///
    /// # Arguments
    ///
    /// * `parent` — parent widget, used only for ownership; the popover is a top-level
    ///   window regardless of who owns it.
    /// * `column_names` — display names in the order they should appear in the popover.
    /// * `logical_indexes` — for each entry in `column_names`, the logical column index
    ///   the underlying table view uses (so master checks / move actions hit the right
    ///   column).
    ///
    /// # Returns
    ///
    /// The fully-built popover. Caller is responsible for connecting its slots via
    /// `connections::set_connections_columns_popover`.
    pub unsafe fn new(parent: &QBox<QWidget>, column_names: &[String], logical_indexes: &[i32]) -> Result<Self> {
        assert_eq!(column_names.len(), logical_indexes.len(), "columns_popover: column_names and logical_indexes length mismatch");

        // Top-level frameless host. We draw nothing on it directly; the visible "card"
        // is a styled QFrame placed inside, and the arrow protrudes below the card into
        // a translucent strip at the bottom.
        let main_widget = QWidget::new_1a(parent);
        let flags = QFlags::from(WindowType::Popup).to_int() | WindowType::FramelessWindowHint.to_int();
        main_widget.set_window_flags(QFlags::from(flags));
        main_widget.set_attribute_1a(WidgetAttribute::WATranslucentBackground);
        main_widget.set_minimum_width(POPOVER_MIN_WIDTH);
        main_widget.set_minimum_height(POPOVER_MIN_HEIGHT);
        main_widget.set_maximum_height(POPOVER_MAX_HEIGHT);
        main_widget.set_window_title(&qtr("columns_popover_title"));
        main_widget.hide();

        // Outer vertical layout on main_widget — just the card.
        let main_layout = QVBoxLayout::new_1a(&main_widget);
        main_layout.set_contents_margins_4a(0, 0, 0, 0);
        main_layout.set_spacing(0);

        // The card itself — this is what the user perceives as the popover.
        let card = QFrame::new_1a(&main_widget);
        card.set_object_name(&QString::from_std_str("popover_card"));
        card.set_style_sheet(&QString::from_std_str(CARD_STYLE));
        main_layout.add_widget(&card);

        // Inner content layout — every piece of UI lives inside the card.
        let outer = QVBoxLayout::new_1a(&card);
        outer.set_contents_margins_4a(8, 8, 8, 8);
        outer.set_spacing(4);

        // Title.
        let title_label = QLabel::from_q_string_q_widget(&qtr("columns_popover_title"), &card);
        title_label.set_style_sheet(&QString::from_std_str("font-weight: bold;"));
        outer.add_widget(&title_label);

        // Search.
        let search_edit = QLineEdit::from_q_widget(&card);
        search_edit.set_placeholder_text(&qtr("columns_popover_search"));
        search_edit.set_clear_button_enabled(true);
        outer.add_widget(&search_edit);

        // Visual separator.
        let separator = QFrame::new_1a(&card);
        separator.set_frame_shape(Shape::HLine);
        outer.add_widget(&separator);

        // Column-headers legend row. Tells the user what each checkbox column means.
        // The widgets to the right of the name use fixed-width cells (matching every
        // per-row entry below) so titles and checkboxes line up column-for-column.
        let legend_widget = QWidget::new_1a(&card);
        let legend_layout = QHBoxLayout::new_1a(&legend_widget);
        // Right margin matches the always-on scrollbar reserved inside the scroll area
        // below, so the legend's right-aligned cells line up with the rows' cells.
        legend_layout.set_contents_margins_4a(2, 0, SCROLLBAR_RESERVE_PX, 0);
        legend_layout.set_spacing(4);

        let column_header = QLabel::from_q_string_q_widget(&qtr("header_column"), &legend_widget);
        column_header.set_text_format(qt_core::TextFormat::RichText);
        legend_layout.add_widget(&column_header);
        legend_layout.add_stretch_1a(1);

        let hide_header = QLabel::from_q_string_q_widget(&qtr("columns_popover_hide"), &legend_widget);
        hide_header.set_style_sheet(&QString::from_std_str("font-weight: 600;"));
        hide_header.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        let hide_header_cell = wrap_in_cell(&legend_widget, &hide_header, CHECKBOX_CELL_WIDTH);
        legend_layout.add_widget(&hide_header_cell);

        let freeze_header = QLabel::from_q_string_q_widget(&qtr("columns_popover_freeze"), &legend_widget);
        freeze_header.set_style_sheet(&QString::from_std_str("font-weight: 600;"));
        freeze_header.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        let freeze_header_cell = wrap_in_cell(&legend_widget, &freeze_header, CHECKBOX_CELL_WIDTH);
        legend_layout.add_widget(&freeze_header_cell);

        // Empty placeholders for the two move-button columns so the legend's checkbox
        // headers align with the per-row checkboxes (which have move buttons to their right).
        let legend_up_spacer = empty_cell(&legend_widget, MOVE_BUTTON_WIDTH);
        legend_layout.add_widget(&legend_up_spacer);
        let legend_down_spacer = empty_cell(&legend_widget, MOVE_BUTTON_WIDTH);
        legend_layout.add_widget(&legend_down_spacer);

        outer.add_widget(&legend_widget);

        // Master toggles row.
        let master_widget = QWidget::new_1a(&card);
        let master_grid = QHBoxLayout::new_1a(&master_widget);
        // Same scrollbar reservation as the legend so the master checkboxes align with
        // their column counterparts in the scrolled list.
        master_grid.set_contents_margins_4a(2, 2, SCROLLBAR_RESERVE_PX, 2);
        master_grid.set_spacing(4);

        let master_label = QLabel::from_q_string_q_widget(&qtr("all"), &master_widget);
        master_label.set_style_sheet(&QString::from_std_str("font-style: italic; color: palette(mid);"));
        master_grid.add_widget(&master_label);
        master_grid.add_stretch_1a(1);

        let hide_all_checkbox = QCheckBox::from_q_widget(&master_widget);
        hide_all_checkbox.set_tool_tip(&qtr("columns_popover_hide"));
        let hide_all_cell = wrap_in_cell(&master_widget, &hide_all_checkbox, CHECKBOX_CELL_WIDTH);
        master_grid.add_widget(&hide_all_cell);

        let freeze_all_checkbox = QCheckBox::from_q_widget(&master_widget);
        freeze_all_checkbox.set_tool_tip(&qtr("columns_popover_freeze"));
        let freeze_all_cell = wrap_in_cell(&master_widget, &freeze_all_checkbox, CHECKBOX_CELL_WIDTH);
        master_grid.add_widget(&freeze_all_cell);

        let master_up_spacer = empty_cell(&master_widget, MOVE_BUTTON_WIDTH);
        master_grid.add_widget(&master_up_spacer);
        let master_down_spacer = empty_cell(&master_widget, MOVE_BUTTON_WIDTH);
        master_grid.add_widget(&master_down_spacer);

        outer.add_widget(&master_widget);

        // Visual separator between the master row and the per-column rows.
        let separator2 = QFrame::new_1a(&card);
        separator2.set_frame_shape(Shape::HLine);
        outer.add_widget(&separator2);

        // Scrollable column list. Vertical scrollbar is forced always-on so the inner
        // viewport's width stays constant — otherwise it shrinks by ~15 px when the
        // scrollbar appears, and the per-column rows misalign with the legend / master
        // rows above (which live directly on the card and don't see the scrollbar).
        let scroll_area = QScrollArea::new_1a(&card);
        scroll_area.set_widget_resizable(true);
        scroll_area.set_frame_shape(Shape::NoFrame);
        scroll_area.set_vertical_scroll_bar_policy(ScrollBarPolicy::ScrollBarAlwaysOn);
        let scroll_inner = QWidget::new_1a(&scroll_area);
        let inner_layout = QVBoxLayout::new_1a(&scroll_inner);
        inner_layout.set_contents_margins_4a(0, 0, 0, 0);
        inner_layout.set_spacing(2);

        let mut rows = Vec::with_capacity(column_names.len());
        for (display_index, (name, logical_index)) in column_names.iter().zip(logical_indexes.iter()).enumerate() {
            let row = build_column_row(&scroll_inner, name, *logical_index, display_index, column_names.len())?;
            inner_layout.add_widget(row.row_widget());
            rows.push(row);
        }
        inner_layout.add_stretch_1a(1);

        scroll_area.set_widget(&scroll_inner);
        outer.add_widget(&scroll_area);

        let search_edit: QPtr<QLineEdit> = search_edit.into_q_ptr();
        let rows_layout: QPtr<QVBoxLayout> = QPtr::new(inner_layout.as_ptr());

        Ok(Self {
            main_widget,
            _card: card,
            search_edit,
            hide_all_checkbox,
            freeze_all_checkbox,
            rows_layout,
            rows,
        })
    }

    /// Position and show the popover **above** `anchor` with a slide-up animation.
    pub unsafe fn pop_above(&self, anchor: &QPtr<QToolButton>) {
        if anchor.is_null() {
            self.main_widget.show();
            return;
        }
        let anchor_widget: QPtr<QWidget> = anchor.static_upcast();

        // Force a layout pass so we know the popover's actual size before positioning.
        self.main_widget.adjust_size();
        let popup_width = self.main_widget.width().max(POPOVER_MIN_WIDTH);
        let popup_height = self.main_widget.height().max(POPOVER_MIN_HEIGHT);

        let anchor_top_global = anchor_widget.map_to_global_q_point(&QPoint::new_2a(0, 0));
        let anchor_width = anchor_widget.width();
        let final_x = anchor_top_global.x() + (anchor_width / 2) - (popup_width / 2);
        let final_y = anchor_top_global.y() - POPOVER_ANCHOR_GAP - popup_height;

        // Start position is a few pixels below the final position so the popup slides up.
        let start_y = final_y + 16;
        self.main_widget.move_2a(final_x, start_y);
        self.main_widget.show();
        self.main_widget.raise();
        self.main_widget.activate_window();

        // Parent the animation to the popover so the returned QBox doesn't delete it when
        // this function returns (a parent-less QBox would, killing the slide before it runs).
        // DeleteWhenStopped lets Qt reap it once the animation finishes.
        let anim = QPropertyAnimation::new_3a(&self.main_widget, &QByteArray::from_slice(b"pos"), &self.main_widget);
        anim.set_duration(POPOVER_ANIM_MS);
        anim.set_start_value(&QVariant::from_q_point(&QPoint::new_2a(final_x, start_y)));
        anim.set_end_value(&QVariant::from_q_point(&QPoint::new_2a(final_x, final_y)));
        anim.start_1a(DeletionPolicy::DeleteWhenStopped);
    }

    /// Toggle visibility. Used by the Columns button slot.
    pub unsafe fn toggle(&self, anchor: &QPtr<QToolButton>) {
        if self.main_widget.is_visible() {
            self.main_widget.hide();
        } else {
            self.pop_above(anchor);
        }
    }

    /// Move the given row up or down by `delta` positions in the visible list. Caller
    /// is responsible for moving the underlying table column; this only touches the
    /// popover's row layout and the per-row up/down enabled state.
    pub unsafe fn move_row_in_layout(&self, row_widget: &QBox<QWidget>, delta: i32) {
        if self.rows_layout.is_null() { return; }
        let current = self.rows_layout.index_of_q_widget(row_widget.as_ptr());
        if current < 0 { return; }
        let target = current + delta;
        if target < 0 || target >= self.rows_layout.count() { return; }

        self.rows_layout.remove_widget(row_widget.as_ptr());
        self.rows_layout.insert_widget_2a(target, row_widget.as_ptr());

        // Re-evaluate up/down button enabled state across all rows so the new first/last
        // rows get their buttons disabled correctly.
        self.refresh_move_button_state();
    }

    /// Disable the up button on the topmost row and the down button on the bottom row.
    pub unsafe fn refresh_move_button_state(&self) {
        let total = self.rows_layout.count();
        if total <= 0 { return; }
        for row in &self.rows {
            let pos = self.rows_layout.index_of_q_widget(row.row_widget().as_ptr());
            if pos < 0 { continue; }
            row.move_up_button().set_enabled(pos > 0);
            // The last layout slot is the stretch we added in `new()`, so the last *row*
            // sits at index `total - 2`.
            row.move_down_button().set_enabled(pos < total - 2);
        }
    }

    /// Apply a search-filter string: rows whose displayed name doesn't contain `needle`
    /// (case-insensitive) are hidden. Empty needle shows all rows.
    pub unsafe fn apply_search_filter(&self, needle: &str) {
        let needle_lower = needle.to_lowercase();
        for row in self.rows() {
            if needle_lower.is_empty() {
                row.row_widget().show();
            } else {
                let name = row.name_label().text().to_std_string().to_lowercase();
                if name.contains(&needle_lower) {
                    row.row_widget().show();
                } else {
                    row.row_widget().hide();
                }
            }
        }
    }
}

unsafe fn build_column_row(parent: &QBox<QWidget>, name: &str, logical_index: i32, display_index: usize, total: usize) -> Result<ColumnRow> {
    let row_widget = QWidget::new_1a(parent);
    let row_layout = QHBoxLayout::new_1a(&row_widget);
    row_layout.set_contents_margins_4a(2, 0, 2, 0);
    row_layout.set_spacing(4);

    // Name on the left, stretch in the middle, then the fixed-width hide/freeze/move
    // cells on the right. The stretch makes the row tolerant to long names — they push
    // into the stretch zone but the right-hand columns stay anchored.
    let name_label = QLabel::from_q_string_q_widget(&QString::from_std_str(name), &row_widget);
    row_layout.add_widget(&name_label);
    row_layout.add_stretch_1a(1);

    let hide_checkbox = QCheckBox::from_q_widget(&row_widget);
    hide_checkbox.set_tool_tip(&qtr("columns_popover_hide"));
    let hide_cell = wrap_in_cell(&row_widget, &hide_checkbox, CHECKBOX_CELL_WIDTH);
    row_layout.add_widget(&hide_cell);

    let freeze_checkbox = QCheckBox::from_q_widget(&row_widget);
    freeze_checkbox.set_tool_tip(&qtr("columns_popover_freeze"));
    let freeze_cell = wrap_in_cell(&row_widget, &freeze_checkbox, CHECKBOX_CELL_WIDTH);
    row_layout.add_widget(&freeze_cell);

    let move_up_button = QToolButton::new_1a(&row_widget);
    move_up_button.set_text(&QString::from_std_str("\u{25b2}"));
    move_up_button.set_tool_tip(&qtr("columns_popover_move_up"));
    move_up_button.set_auto_raise(true);
    move_up_button.set_enabled(display_index > 0);
    move_up_button.set_fixed_width(MOVE_BUTTON_WIDTH);
    row_layout.add_widget(&move_up_button);

    let move_down_button = QToolButton::new_1a(&row_widget);
    move_down_button.set_text(&QString::from_std_str("\u{25bc}"));
    move_down_button.set_tool_tip(&qtr("columns_popover_move_down"));
    move_down_button.set_auto_raise(true);
    move_down_button.set_enabled(display_index + 1 < total);
    move_down_button.set_fixed_width(MOVE_BUTTON_WIDTH);
    row_layout.add_widget(&move_down_button);

    let name_label: QPtr<QLabel> = name_label.into_q_ptr();
    let move_up_button: QPtr<QToolButton> = move_up_button.into_q_ptr();
    let move_down_button: QPtr<QToolButton> = move_down_button.into_q_ptr();

    Ok(ColumnRow {
        row_widget,
        name_label,
        hide_checkbox,
        freeze_checkbox,
        move_up_button,
        move_down_button,
        logical_index,
    })
}

/// Wrap `child` in a fixed-width container so it lines up across rows regardless of the
/// child's natural size. The child is centered within the container.
unsafe fn wrap_in_cell<T>(parent: &QBox<QWidget>, child: &QBox<T>, width: i32) -> QBox<QWidget>
where
    T: cpp_core::CppDeletable + cpp_core::StaticUpcast<qt_core::QObject> + cpp_core::StaticUpcast<QWidget>,
{
    let cell = QWidget::new_1a(parent);
    cell.set_fixed_width(width);
    let layout = QHBoxLayout::new_1a(&cell);
    layout.set_contents_margins_4a(0, 0, 0, 0);
    layout.set_spacing(0);
    layout.add_widget(child);
    layout.set_alignment_q_widget_q_flags_alignment_flag(child, QFlags::from(AlignmentFlag::AlignCenter));
    cell
}

/// Empty placeholder cell of fixed width — used in legend / master rows where the
/// move-button columns have no content but still need to occupy their column slot.
unsafe fn empty_cell(parent: &QBox<QWidget>, width: i32) -> QBox<QWidget> {
    let cell = QWidget::new_1a(parent);
    cell.set_fixed_width(width);
    cell
}
