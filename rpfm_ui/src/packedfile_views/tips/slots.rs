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
Module with the slots for Tips Views.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::sync::Arc;

use crate::locale::tr;
use crate::utils::show_dialog;

use super::TipsView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a Tips View.
pub struct TipSlots {
    pub context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    pub context_menu: QBox<SlotOfQPoint>,
    pub new_tip: QBox<SlotNoArgs>,
    pub edit_tip: QBox<SlotNoArgs>,
    pub delete_tip: QBox<SlotNoArgs>,
    pub publish_tip: QBox<SlotNoArgs>,
    pub open_link: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `TipSlots`.
impl TipSlots {

    /// This function creates the entire slot pack for Tip Views.
    pub unsafe fn new(view: &Arc<TipsView>)  -> Self {

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
            if let Err(error) = view.load_new_tip_dialog(false) {
                show_dialog(&view.list, error, false);
            }
        }));

        let edit_tip = SlotNoArgs::new(&view.list, clone!(view => move || {
            if let Err(error) = view.load_new_tip_dialog(true) {
                show_dialog(&view.list, error, false);
            }
        }));

        let publish_tip = SlotNoArgs::new(&view.list, clone!(view => move || {
            match view.publish_tip() {
                Ok(_) => show_dialog(&view.list, tr("message_uploaded_correctly"), true),
                Err(error) => show_dialog(&view.list, error, false),
            }
        }));

        let delete_tip = SlotNoArgs::new(&view.list, clone!(view => move || {
            view.delete_selected_tip();
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
            publish_tip,
            open_link,
        }
    }
}
