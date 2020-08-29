//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `GlobalSearchSlots`.
!*/

use qt_core::{SlotOfBool, SlotOfQModelIndex, Slot, SlotOfQString};

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::check_regex;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the Global Search panel.
pub struct GlobalSearchSlots {
    pub global_search_search: Slot<'static>,
    pub global_search_clear: Slot<'static>,
    pub global_search_replace_current: Slot<'static>,
    pub global_search_replace_all: Slot<'static>,
    pub global_search_check_regex: SlotOfQString<'static>,
    pub global_search_check_regex_clean: SlotOfBool<'static>,
    pub global_search_open_match: SlotOfQModelIndex<'static>,
    pub global_search_toggle_all: SlotOfBool<'static>,
    pub global_search_filter_dbs: Slot<'static>,
    pub global_search_filter_locs: Slot<'static>,
    pub global_search_filter_texts: Slot<'static>,
    pub global_search_filter_schemas: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchSlots`.
impl GlobalSearchSlots {

	/// This function creates an entire `GlobalSearchSlots` struct.
	pub unsafe fn new(
        mut app_ui: AppUI,
        mut global_search_ui: GlobalSearchUI,
        pack_file_contents_ui: PackFileContentsUI
    ) -> Self {

        // What happens when we trigger the "Global Search" action.
        let global_search_search = Slot::new(clone!(mut pack_file_contents_ui => move || {
            global_search_ui.search(&mut pack_file_contents_ui);
        }));

        // What happens when we trigger the "Clear Search" action.
        let global_search_clear = Slot::new(move || {
            global_search_ui.clear();
        });

        // What happens when we trigger the "Replace Current" action.
        let global_search_replace_current = Slot::new(clone!(
            mut pack_file_contents_ui => move || {
            global_search_ui.replace_current(&mut app_ui, &mut pack_file_contents_ui);
        }));

        // What happens when we trigger the "Replace All" action.
        let global_search_replace_all = Slot::new(clone!(
            mut pack_file_contents_ui => move || {
            global_search_ui.replace_all(&mut app_ui, &mut pack_file_contents_ui);
        }));

        // What happens when we trigger the "Check Regex" action.
        let global_search_check_regex = SlotOfQString::new(move |string| {
            if global_search_ui.global_search_use_regex_checkbox.is_checked() {
                check_regex(&string.to_std_string(), global_search_ui.global_search_search_line_edit.static_upcast_mut());
            }
        });

        // What happens when we toggle the "Use Regex" checkbox.
        let global_search_check_regex_clean = SlotOfBool::new(move |is_checked| {
            if is_checked {
                check_regex(&global_search_ui.global_search_search_line_edit.text().to_std_string(), global_search_ui.global_search_search_line_edit.static_upcast_mut());
            } else {
                check_regex("", global_search_ui.global_search_search_line_edit.static_upcast_mut());
            }
        });

        // What happens when we try to open the file corresponding to one of the matches.
        let global_search_open_match = SlotOfQModelIndex::new(move |model_index_filter| {
            GlobalSearchUI::open_match(app_ui, pack_file_contents_ui, model_index_filter.as_ptr());
        });

        // What happens when we toggle the "All" checkbox we have to disable/enable the rest ot the checkboxes..
        let global_search_toggle_all = SlotOfBool::new(move |state| {
            global_search_ui.global_search_search_on_dbs_checkbox.set_enabled(!state);
            global_search_ui.global_search_search_on_locs_checkbox.set_enabled(!state);
            global_search_ui.global_search_search_on_texts_checkbox.set_enabled(!state);
            global_search_ui.global_search_search_on_schemas_checkbox.set_enabled(!state);
        });

        // What happens when we filter the different result TreeViews
        let global_search_filter_dbs = Slot::new(move || {
            GlobalSearchUI::filter_results(
                global_search_ui.global_search_matches_db_tree_view,
                global_search_ui.global_search_matches_filter_db_line_edit,
                global_search_ui.global_search_matches_column_selector_db_combobox,
                global_search_ui.global_search_matches_case_sensitive_db_button,
            );
        });

        let global_search_filter_locs = Slot::new(move || {
            GlobalSearchUI::filter_results(
                global_search_ui.global_search_matches_loc_tree_view,
                global_search_ui.global_search_matches_filter_loc_line_edit,
                global_search_ui.global_search_matches_column_selector_loc_combobox,
                global_search_ui.global_search_matches_case_sensitive_loc_button,
            );
        });

        let global_search_filter_texts = Slot::new(move || {
            GlobalSearchUI::filter_results(
                global_search_ui.global_search_matches_text_tree_view,
                global_search_ui.global_search_matches_filter_text_line_edit,
                global_search_ui.global_search_matches_column_selector_text_combobox,
                global_search_ui.global_search_matches_case_sensitive_text_button,
            );
        });

        let global_search_filter_schemas = Slot::new(move || {
            GlobalSearchUI::filter_results(
                global_search_ui.global_search_matches_schema_tree_view,
                global_search_ui.global_search_matches_filter_schema_line_edit,
                global_search_ui.global_search_matches_column_selector_schema_combobox,
                global_search_ui.global_search_matches_case_sensitive_schema_button,
            );
        });

        // And here... we return all the slots.
		Self {
            global_search_search,
            global_search_clear,
            global_search_replace_current,
            global_search_replace_all,
            global_search_check_regex,
            global_search_check_regex_clean,
            global_search_open_match,
            global_search_toggle_all,
            global_search_filter_dbs,
            global_search_filter_locs,
            global_search_filter_texts,
            global_search_filter_schemas,
		}
	}
}
