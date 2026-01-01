//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use getset::Getters;

use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::utils::show_dialog;

use super::NotesView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct NotesSlots {
    context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    context_menu: QBox<SlotOfQPoint>,
    new_tip: QBox<SlotNoArgs>,
    edit_tip: QBox<SlotNoArgs>,
    delete_tip: QBox<SlotNoArgs>,
    open_link: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl NotesSlots {

    pub unsafe fn new(view: &Arc<NotesView>)  -> Self {

        let context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(&view.list, clone!(
            view => move |_, _| {
                view.context_menu_update();
            }
        ));

        let context_menu = SlotOfQPoint::new(&view.list, clone!(
            view => move |_| {
            view.context_menu_update();
            view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let new_tip = SlotNoArgs::new(&view.new_button, clone!(view => move || {
            if let Err(error) = view.load_new_note_dialog(false) {
                show_dialog(&view.list, error, false);
            }
        }));

        let edit_tip = SlotNoArgs::new(&view.list, clone!(view => move || {
            if let Err(error) = view.load_new_note_dialog(true) {
                show_dialog(&view.list, error, false);
            }
        }));

        let delete_tip = SlotNoArgs::new(&view.list, clone!(view => move || {
            view.delete_selected_note();
        }));

        let open_link = SlotNoArgs::new(&view.list, clone!(view => move || {
            view.open_link();
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            context_menu_enabler,
            context_menu,
            new_tip,
            edit_tip,
            delete_tip,
            open_link,
        }
    }
}
