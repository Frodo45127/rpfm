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
Module with the slots for Table Views.
!*/

use qt_core::QBox;
use qt_core::{SlotOfInt, SlotNoArgs, SlotOfQString};

use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::utils::check_regex;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a table filter.
pub struct FilterViewSlots {
    pub filter_line_edit: QBox<SlotOfQString>,
    pub filter_not_checkbox: QBox<SlotNoArgs>,
    pub filter_match_group_selector: QBox<SlotNoArgs>,
    pub filter_column_selector: QBox<SlotOfInt>,
    pub filter_case_sensitive_button: QBox<SlotNoArgs>,
    pub filter_use_regex_button: QBox<SlotNoArgs>,
    pub filter_show_blank_cells_button: QBox<SlotNoArgs>,
    pub filter_trigger: QBox<SlotNoArgs>,
    pub filter_check_regex: QBox<SlotOfQString>,
    pub filter_add: QBox<SlotNoArgs>,
    pub filter_remove: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `FilterViewSlots`.
impl FilterViewSlots {
    pub unsafe fn new(
        view: &Arc<FilterView>,
        parent_view: &Arc<TableView>,
    ) -> Self {

        // When we want to filter the table...
        let filter_line_edit = SlotOfQString::new(&view.main_widget, clone!(
            view => move |_| {
            FilterView::start_delayed_updates_timer(&view);
        }));

        let filter_match_group_selector = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        let filter_column_selector = SlotOfInt::new(&view.main_widget, clone!(
            parent_view => move |_| {
            parent_view.filter_table();
        }));

        let filter_not_checkbox = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        let filter_case_sensitive_button = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        let filter_use_regex_button = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view,
            view => move || {
                check_regex(&view.filter_line_edit.text().to_std_string(), view.filter_line_edit.static_upcast(), view.use_regex_button().is_checked());

                parent_view.filter_table();
            }
        ));

        let filter_show_blank_cells_button = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        // Function triggered by the filter timer.
        let filter_trigger = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            parent_view.filter_table();
        }));

        // What happens when we trigger the "Check Regex" action.
        let filter_check_regex = SlotOfQString::new(&view.main_widget, clone!(
            view => move |string| {
            check_regex(&string.to_std_string(), view.filter_line_edit.static_upcast(), view.use_regex_button().is_checked());
        }));

        let filter_add = SlotNoArgs::new(&view.main_widget, clone!(
            parent_view => move || {
            match FilterView::new(&parent_view) {
                Ok(_) => FilterView::add_filter_group(&parent_view),
                Err(_) => show_dialog(&parent_view.table_view, "Error while adding new filters. Realistically, this should never happen.", false),
            }
        }));

        let filter_remove = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            parent_view => move || {
            if parent_view.filters().len() > 1 {
                let pos = parent_view.filters().iter().position(|filter_view| view.main_widget.as_ptr().as_raw_ptr() == filter_view.main_widget.as_ptr().as_raw_ptr());
                if let Some(pos) = pos {
                    parent_view.filter_base_widget.layout().remove_widget(view.main_widget.as_ptr());

                    // Make sure to delete the widget so it's not kept in the UI.
                    let filter = parent_view.filters_mut().remove(pos);
                    parent_view.filter_table();

                    filter.main_widget.delete_later();
                }
            }
        }));

        Self {
            filter_line_edit,
            filter_match_group_selector,
            filter_column_selector,
            filter_not_checkbox,
            filter_case_sensitive_button,
            filter_use_regex_button,
            filter_show_blank_cells_button,
            filter_trigger,
            filter_check_regex,
            filter_add,
            filter_remove,
        }
    }
}
