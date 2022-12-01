//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_core::QBox;
use qt_core::{SlotOfBool, SlotOfQModelIndex, SlotNoArgs, SlotOfQString};

use rpfm_lib::integrations::log::*;

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::utils::check_regex as check_regex_string;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the Global Search panel.
pub struct GlobalSearchSlots {
    pub search: QBox<SlotNoArgs>,
    pub clear: QBox<SlotNoArgs>,
    pub replace_current: QBox<SlotNoArgs>,
    pub replace_all: QBox<SlotNoArgs>,
    pub check_regex: QBox<SlotOfQString>,
    pub check_regex_clean: QBox<SlotOfBool>,
    pub open_match: QBox<SlotOfQModelIndex>,
    pub toggle_all: QBox<SlotOfBool>,
    pub filter_dbs: QBox<SlotNoArgs>,
    pub filter_locs: QBox<SlotNoArgs>,
    pub filter_texts: QBox<SlotNoArgs>,
    pub filter_schemas: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchSlots`.
impl GlobalSearchSlots {

	/// This function creates an entire `GlobalSearchSlots` struct.
	pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Self {

        // What happens when we trigger the "Global Search" action.
        let search = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
            pack_file_contents_ui,
            global_search_ui => move || {
            info!("Triggering `Global Search` By Slot");
            global_search_ui.search(&pack_file_contents_ui);
        }));

        // What happens when we trigger the "Clear Search" action.
        let clear = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
            global_search_ui => move || {
            global_search_ui.clear();
        }));

        // What happens when we trigger the "Replace Current" action.
        let replace_current = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move || {
            info!("Triggering `Global Replace (current)` By Slot");
            GlobalSearchUI::replace_current(&app_ui, &pack_file_contents_ui, &global_search_ui);
        }));

        // What happens when we trigger the "Replace All" action.
        let replace_all = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move || {
            info!("Triggering `Global Replace (all)` By Slot");
            GlobalSearchUI::replace_all(&app_ui, &pack_file_contents_ui, &global_search_ui);
        }));

        // What happens when we trigger the "Check Regex" action.
        let check_regex = SlotOfQString::new(&global_search_ui.dock_widget, clone!(
            global_search_ui => move |string| {
            if global_search_ui.use_regex_checkbox.is_checked() {
                check_regex_string(&string.to_std_string(), global_search_ui.search_combobox.static_upcast());
            }
        }));

        // What happens when we toggle the "Use Regex" checkbox.
        let check_regex_clean = SlotOfBool::new(&global_search_ui.dock_widget, clone!(
            global_search_ui => move |is_checked| {
            if is_checked {
                check_regex_string(&global_search_ui.search_combobox.current_text().to_std_string(), global_search_ui.search_combobox.static_upcast());
            } else {
                check_regex_string("", global_search_ui.search_combobox.static_upcast());
            }
        }));

        // What happens when we try to open the file corresponding to one of the matches.
        let open_match = SlotOfQModelIndex::new(&global_search_ui.dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |model_index_filter| {
            info!("Triggering `Open Global Search Match` By Slot");
            GlobalSearchUI::open_match(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, model_index_filter.as_ptr());
        }));

        // What happens when we toggle the "All" checkbox we have to disable/enable the rest ot the checkboxes..
        let toggle_all = SlotOfBool::new(&global_search_ui.dock_widget, clone!(
        global_search_ui => move |state| {
            global_search_ui.search_on_dbs_checkbox.set_enabled(!state);
            global_search_ui.search_on_locs_checkbox.set_enabled(!state);
            global_search_ui.search_on_texts_checkbox.set_enabled(!state);
            global_search_ui.search_on_schemas_checkbox.set_enabled(!state);
        }));

        // What happens when we filter the different result TreeViews
        let filter_dbs = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
        global_search_ui => move || {
            GlobalSearchUI::filter_results(
                &global_search_ui.matches_db_tree_view,
                &global_search_ui.matches_filter_db_line_edit,
                &global_search_ui.matches_column_selector_db_combobox,
                &global_search_ui.matches_case_sensitive_db_button,
            );
        }));

        let filter_locs = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
        global_search_ui => move || {
            GlobalSearchUI::filter_results(
                &global_search_ui.matches_loc_tree_view,
                &global_search_ui.matches_filter_loc_line_edit,
                &global_search_ui.matches_column_selector_loc_combobox,
                &global_search_ui.matches_case_sensitive_loc_button,
            );
        }));

        let filter_texts = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
        global_search_ui => move || {
            GlobalSearchUI::filter_results(
                &global_search_ui.matches_text_tree_view,
                &global_search_ui.matches_filter_text_line_edit,
                &global_search_ui.matches_column_selector_text_combobox,
                &global_search_ui.matches_case_sensitive_text_button,
            );
        }));

        let filter_schemas = SlotNoArgs::new(&global_search_ui.dock_widget, clone!(
        global_search_ui => move || {
            GlobalSearchUI::filter_results(
                &global_search_ui.matches_schema_tree_view,
                &global_search_ui.matches_filter_schema_line_edit,
                &global_search_ui.matches_column_selector_schema_combobox,
                &global_search_ui.matches_case_sensitive_schema_button,
            );
        }));

        // And here... we return all the slots.
		Self {
            search,
            clear,
            replace_current,
            replace_all,
            check_regex,
            check_regex_clean,
            open_match,
            toggle_all,
            filter_dbs,
            filter_locs,
            filter_texts,
            filter_schemas,
		}
	}
}
