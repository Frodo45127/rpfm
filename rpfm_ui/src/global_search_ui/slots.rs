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

use qt_gui::color::Color;
use qt_gui::palette::{ColorRole, Palette};

use qt_core::qt::GlobalColor;
use qt_core::slots::{SlotBool, SlotModelIndexRef, SlotNoArgs, SlotStringRef};

use regex::Regex;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::TheOneSlot;
use crate::packfile_contents_ui::PackFileContentsUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the Global Search panel.
pub struct GlobalSearchSlots {
    pub global_search_search: SlotNoArgs<'static>,
    pub global_search_clear: SlotNoArgs<'static>,
    pub global_search_replace_all: SlotNoArgs<'static>,
    pub global_search_check_regex: SlotStringRef<'static>,
    pub global_search_open_match: SlotModelIndexRef<'static>,
    pub global_search_toggle_all: SlotBool<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchSlots`.
impl GlobalSearchSlots {

	/// This function creates an entire `GlobalSearchSlots` struct.
	pub fn new(
        app_ui: AppUI,
        global_search_ui: GlobalSearchUI,
        pack_file_contents_ui: PackFileContentsUI,
        slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>,
    ) -> Self {

        // What happens when we trigger the "Global Search" action.
        let global_search_search = SlotNoArgs::new(clone!(pack_file_contents_ui => move || {
            global_search_ui.search(&pack_file_contents_ui);
        }));

        // What happens when we trigger the "Clear Search" action.
        let global_search_clear = SlotNoArgs::new(move || {
            global_search_ui.clear();
        });

        // What happens when we trigger the "Replace All" action.
        let global_search_replace_all = SlotNoArgs::new(clone!(
            pack_file_contents_ui,
            slot_holder => move || {
            global_search_ui.replace_all(&app_ui, &pack_file_contents_ui, &slot_holder);
        }));


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

        // What happens when we try to open the file corresponding to one of the matches.
        let global_search_open_match = SlotModelIndexRef::new(clone!(slot_holder => move |model_index_filter| {
            GlobalSearchUI::open_match(app_ui, pack_file_contents_ui, global_search_ui, &slot_holder, model_index_filter);
        }));

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
            global_search_clear,
            global_search_replace_all,
            global_search_check_regex,
            global_search_open_match,
            global_search_toggle_all,
		}
	}
}
