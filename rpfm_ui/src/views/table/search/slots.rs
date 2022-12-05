//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::{SlotNoArgs, SlotOfQString};

use rpfm_lib::integrations::log::*;

use crate::utils::check_regex as check_regex_string;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

pub struct SearchViewSlots {
    pub search: QBox<SlotNoArgs>,
    pub prev_match: QBox<SlotNoArgs>,
    pub next_match: QBox<SlotNoArgs>,
    pub replace: QBox<SlotNoArgs>,
    pub replace_all: QBox<SlotNoArgs>,
    pub close: QBox<SlotNoArgs>,
    pub check_regex: QBox<SlotOfQString>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl SearchViewSlots {

    pub unsafe fn new(view: &Arc<SearchView>, table_view: &Arc<TableView>) -> Self {

        let search = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                info!("Triggering `Local Search` By Slot");
                view.search(&table_view);
            }
        ));

        let prev_match = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                info!("Triggering `Local Prev Match` By Slot");
                view.prev_match(&table_view);
            }
        ));

        let next_match = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                info!("Triggering `Local Next Match` By Slot");
                view.next_match(&table_view);
            }
        ));

        let replace = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                info!("Triggering `Local Replace Current` By Slot");
                view.replace_current(&table_view);
            }
        ));

        let replace_all = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                info!("Triggering `Local Replace All` By Slot");
                view.replace_all(&table_view);
            }
        ));

        let close = SlotNoArgs::new(&view.main_widget, clone!(
            view,
            table_view => move || {
                view.main_widget.hide();
                table_view.table_view.set_focus_0a();
            }
        ));

        // What happens when we trigger the "Check Regex" action.
        let check_regex = SlotOfQString::new(&view.main_widget, clone!(
            mut view => move |string| {
            check_regex_string(&string.to_std_string(), view.search_line_edit.static_upcast());
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            search,
            prev_match,
            next_match,
            replace,
            replace_all,
            close,
            check_regex,
        }
    }
}

