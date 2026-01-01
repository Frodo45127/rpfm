//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! TableView submodule to provide Filter functionality.

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLineEdit;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QTimer;

use anyhow::Result;
use getset::{Getters, MutGetters};

use std::sync::Arc;

use rpfm_ui_common::utils::*;

use crate::settings_helpers::settings_bool;
use crate::utils::{qtr, tr};
use crate::views::table::clean_column_names;

use self::slots::FilterViewSlots;
use super::TableView;

mod connections;
mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/table_filter_groupbox.ui";
const VIEW_RELEASE: &str = "ui/table_filter_groupbox.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the stuff needed for a filter row.
#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct FilterView {
    main_widget: QBox<QWidget>,
    not_checkbox: QPtr<QCheckBox>,
    filter_line_edit: QPtr<QLineEdit>,
    case_sensitive_button: QPtr<QToolButton>,
    use_regex_button: QPtr<QToolButton>,
    show_blank_cells_button: QPtr<QToolButton>,
    show_edited_cells_button: QPtr<QToolButton>,
    group_combobox: QPtr<QComboBox>,
    column_combobox: QPtr<QComboBox>,
    variant_combobox: QPtr<QComboBox>,
    timer_delayed_updates: QBox<QTimer>,
    add_button: QPtr<QToolButton>,
    remove_button: QPtr<QToolButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FilterView {

    pub unsafe fn new(view: &Arc<TableView>) -> Result<()> {
        let parent = view.filter_base_widget_ptr();
        let parent_grid: QPtr<QGridLayout> = parent.layout().static_downcast();

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(parent, template_path)?;

        let not_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "not_checkbox")?;
        let filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "filter_line_edit")?;
        let case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "case_sensitive_button")?;
        let use_regex_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "use_regex")?;
        let show_blank_cells_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "show_blank_cells_button")?;
        let show_edited_cells_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "show_edited_cells_button")?;
        let group_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "group_combobox")?;
        let column_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "column_combobox")?;
        let variant_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "variant_combobox")?;
        let timer_delayed_updates = QTimer::new_1a(&main_widget);
        let add_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "add_button")?;
        let remove_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "remove_button")?;

        filter_line_edit.set_placeholder_text(&qtr("table_filter"));
        filter_line_edit.set_clear_button_enabled(true);
        use_regex_button.set_tool_tip(&qtr("table_filter_use_regex"));
        use_regex_button.set_checked(true);
        show_blank_cells_button.set_tool_tip(&qtr("table_filter_show_blank_cells"));
        show_edited_cells_button.set_tool_tip(&qtr("table_filter_show_edited_cells"));
        case_sensitive_button.set_tool_tip(&qtr("table_filter_case_sensitive"));
        timer_delayed_updates.set_single_shot(true);

        // The first filter must never be deleted.
        if !view.filters().is_empty() {
            remove_button.set_enabled(true);
        }

        // Reuse the models from the first filterview, as that one will never get destroyed.
        if let Some(first_filter) = view.filters().first() {
            variant_combobox.set_model(first_filter.variant_combobox.model());
            column_combobox.set_model(first_filter.column_combobox.model());
            group_combobox.set_model(first_filter.group_combobox.model());
        }

        else {
            let filter_match_group_list = QStandardItemModel::new_1a(&group_combobox);
            let filter_column_list = QStandardItemModel::new_1a(&column_combobox);
            let filter_variant_list = QStandardItemModel::new_1a(&variant_combobox);

            variant_combobox.set_model(&filter_variant_list);
            column_combobox.set_model(&filter_column_list);
            group_combobox.set_model(&filter_match_group_list);

            let fields = view.table_definition().fields_processed_sorted(settings_bool("tables_use_old_column_order"));
            for field in &fields {
                let name = clean_column_names(field.name());
                column_combobox.add_item_q_string(&QString::from_std_str(name));
            }

            group_combobox.add_item_q_string(&QString::from_std_str(format!("{} {}", tr("filter_group"), 1)));

            variant_combobox.add_item_q_string(&qtr("filter_variant_source"));
            variant_combobox.add_item_q_string(&qtr("filter_variant_lookup"));
            variant_combobox.add_item_q_string(&qtr("filter_variant_both"));
        }

        // Add the new filter at the bottom of the window.
        parent_grid.add_widget_5a(&main_widget, parent_grid.row_count(), 0, 1, 2);

        let filter = Arc::new(Self {
            main_widget,
            not_checkbox,
            filter_line_edit,
            case_sensitive_button,
            use_regex_button,
            show_blank_cells_button,
            show_edited_cells_button,
            group_combobox,
            column_combobox,
            variant_combobox,
            timer_delayed_updates,
            add_button,
            remove_button,
        });

        let slots = FilterViewSlots::new(&filter, view);
        connections::set_connections_filter(&filter, &slots);

        view.filters_mut().push(filter);
        Ok(())
    }

    pub unsafe fn start_delayed_updates_timer(&self) {
        self.timer_delayed_updates.set_interval(500);
        self.timer_delayed_updates.start_0a();
    }

    pub unsafe fn add_filter_group(view: &TableView) {
        if view.filters()[0].group_combobox.count() < view.filters().len() as i32 {
            let name = QString::from_std_str(format!("{} {}", tr("filter_group"), view.filters()[0].group_combobox.count() + 1));
            view.filters()[0].group_combobox.add_item_q_string(&name);
        }
    }
}
