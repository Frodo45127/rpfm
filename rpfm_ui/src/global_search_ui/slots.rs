//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use qt_gui::color::Color;
use qt_gui::palette::{ColorRole, Palette};

use qt_core::qt::GlobalColor;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef};

use regex::Regex;

use crate::CENTRAL_COMMAND;
use crate::communications::{THREADS_COMMUNICATION_ERROR, Command, Response};
use crate::global_search_ui::GlobalSearchUI;
use crate::ui_state::global_search::GlobalSearch;
use crate::UI_STATE;


//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the Global Search panel.
pub struct GlobalSearchSlots {
    pub global_search_search: SlotNoArgs<'static>,
    pub global_search_check_regex: SlotStringRef<'static>,

    pub global_search_toggle_all: SlotBool<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchSlots`.
impl GlobalSearchSlots {

	/// This function creates an entire `GlobalSearchSlots` struct.
	pub fn new(global_search_ui: GlobalSearchUI) -> Self {

        // What happens when we trigger the "Global Search" action.
        let global_search_search = SlotNoArgs::new(move || {

            // Create the global search and populate it with all the settings for the search.
            let mut global_search = GlobalSearch::default();
            global_search.pattern = unsafe { global_search_ui.global_search_search_line_edit.as_ref().unwrap().text().to_std_string() };
            global_search.case_sensitive = unsafe { global_search_ui.global_search_case_sensitive_checkbox.as_ref().unwrap().is_checked() };
            global_search.use_regex = unsafe { global_search_ui.global_search_use_regex_checkbox.as_ref().unwrap().is_checked() };

            if unsafe { global_search_ui.global_search_search_on_all_checkbox.as_ref().unwrap().is_checked() } {
                global_search.search_on_dbs = true;
                global_search.search_on_locs = true;
                global_search.search_on_texts = true;
                global_search.search_on_schema = true;
            }
            else {
                global_search.search_on_dbs = unsafe { global_search_ui.global_search_search_on_dbs_checkbox.as_ref().unwrap().is_checked() };
                global_search.search_on_locs = unsafe { global_search_ui.global_search_search_on_locs_checkbox.as_ref().unwrap().is_checked() };
                global_search.search_on_texts = unsafe { global_search_ui.global_search_search_on_texts_checkbox.as_ref().unwrap().is_checked() };
                global_search.search_on_schema = unsafe { global_search_ui.global_search_search_on_schemas_checkbox.as_ref().unwrap().is_checked() };
            }

            let t = std::time::SystemTime::now();
            CENTRAL_COMMAND.send_message_qt(Command::GlobalSearch(global_search));

            // While we wait for an answer, we need to clear the current results panels.
            let tree_view_db = unsafe { global_search_ui.global_search_matches_db_tree_view.as_mut().unwrap() };
            let tree_view_loc = unsafe { global_search_ui.global_search_matches_loc_tree_view.as_mut().unwrap() };
            let tree_view_text = unsafe { global_search_ui.global_search_matches_text_tree_view.as_mut().unwrap() };
            let tree_view_schema = unsafe { global_search_ui.global_search_matches_schema_tree_view.as_mut().unwrap() };

            let model_db = unsafe { global_search_ui.global_search_matches_db_tree_model.as_mut().unwrap() };
            let model_loc = unsafe { global_search_ui.global_search_matches_loc_tree_model.as_mut().unwrap() };
            let model_text = unsafe { global_search_ui.global_search_matches_text_tree_model.as_mut().unwrap() };
            let model_schema = unsafe { global_search_ui.global_search_matches_schema_tree_model.as_mut().unwrap() };

            model_db.clear();
            model_loc.clear();
            model_text.clear();
            model_schema.clear();

            match CENTRAL_COMMAND.recv_message_qt() {
                Response::GlobalSearch(global_search) => {

                    println!("Time to search from click to search complete: {:?}", t.elapsed().unwrap());

                    // Load the results to their respective models. Then, store the GlobalSearch for future checks.
                    GlobalSearch::load_table_matches_to_ui(model_db, tree_view_db, &global_search.matches_db);
                    GlobalSearch::load_table_matches_to_ui(model_loc, tree_view_loc, &global_search.matches_loc);
                    GlobalSearch::load_text_matches_to_ui(model_text, tree_view_text, &global_search.matches_text);
                    GlobalSearch::load_schema_matches_to_ui(model_schema, tree_view_schema, &global_search.matches_schema);
                    //println!("{:?}", global_search);
                    UI_STATE.set_global_search(&global_search);
                }

                // In ANY other situation, it's a message problem.
                _ => panic!(THREADS_COMMUNICATION_ERROR)
            }
        });

        // What happens when we trigger the "Check Regex" action.
        let global_search_check_regex = SlotStringRef::new(move |string| {
            let mut palette = Palette::new(());
            if unsafe { global_search_ui.global_search_use_regex_checkbox.as_ref().unwrap().is_checked() } {
                if Regex::new(&string.to_std_string()).is_ok() {
                    palette.set_color((ColorRole::Base, &Color::new(GlobalColor::DarkGreen)));
                } else {
                    palette.set_color((ColorRole::Base, &Color::new(GlobalColor::DarkRed)));
                }
            }
            else {

                // Not really right but... it does the job for now.
                palette.set_color((ColorRole::Base, &Color::new(GlobalColor::Transparent)));
            }
            unsafe { global_search_ui.global_search_search_line_edit.as_mut().unwrap().set_palette(&palette); }
        });

        // What happens when we toggle the "All" checkbox we have to disable/enable the rest ot the checkboxes..
        let global_search_toggle_all = SlotBool::new(move |state| {
            unsafe { global_search_ui.global_search_search_on_dbs_checkbox.as_mut().unwrap().set_enabled(!state) };
            unsafe { global_search_ui.global_search_search_on_locs_checkbox.as_mut().unwrap().set_enabled(!state) };
            unsafe { global_search_ui.global_search_search_on_texts_checkbox.as_mut().unwrap().set_enabled(!state) };
            unsafe { global_search_ui.global_search_search_on_schemas_checkbox.as_mut().unwrap().set_enabled(!state) };
        });

        // And here... we return all the slots.
		Self {
            global_search_search,
            global_search_check_regex,

            global_search_toggle_all,
		}
	}
}
