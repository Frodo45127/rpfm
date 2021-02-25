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
Module with all the code related to `TemplateUISlots`.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `TemplateUI` struct.
///
/// This means everything you can do with the stuff you have in the `TemplateUI` goes here.
pub struct TemplateUISlots {
    //pub toggle_required: QBox<SlotNoArgs>,
}


/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SaveTemplateUI` struct.
///
/// This means everything you can do with the stuff you have in the `SaveTemplateUI` goes here.
pub struct SaveTemplateUISlots {
    pub sections_slot_add: QBox<SlotNoArgs>,
    pub sections_slot_remove: QBox<SlotNoArgs>,
    pub options_slot_add: QBox<SlotNoArgs>,
    pub options_slot_remove: QBox<SlotNoArgs>,
    pub params_slot_add: QBox<SlotNoArgs>,
    pub params_slot_remove: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TemplateUISlots`.
impl TemplateUISlots {

    /// This function creates a new `TemplateUISlots`.
    pub unsafe fn new(ui: &Rc<TemplateUI>) -> Self {
        //let toggle_required = SlotNoArgs::new(&ui.dialog, clone!(
        //    ui => move || {
        //    ui.update_template_view();
        //}));
        TemplateUISlots {
            //toggle_required,
        }
    }
}

/// Implementation of `SaveTemplateUISlots`.
impl SaveTemplateUISlots {

    /// This function creates a new `SaveTemplateUISlots`.
    pub unsafe fn new(ui: &Rc<SaveTemplateUI>) -> Self {

        // Slots for step 1
        let sections_slot_add = SlotNoArgs::new(&ui.sections_tableview, clone!(
            ui => move || {
            SaveTemplateUI::add_empty_row(&ui.sections_model);
        }));

        let sections_slot_remove = SlotNoArgs::new(&ui.sections_tableview, clone!(
            ui => move || {
            SaveTemplateUI::remove_rows(&ui.sections_model, &ui.sections_tableview);
        }));

        // Slots for step 2
        let options_slot_add = SlotNoArgs::new(&ui.options_tableview, clone!(
            ui => move || {
            SaveTemplateUI::add_empty_row(&ui.options_model);
        }));

        let options_slot_remove = SlotNoArgs::new(&ui.options_tableview, clone!(
            ui => move || {
            SaveTemplateUI::remove_rows(&ui.options_model, &ui.options_tableview);
        }));

        // Slots for step 3
        let params_slot_add = SlotNoArgs::new(&ui.params_tableview, clone!(
            ui => move || {
            SaveTemplateUI::add_empty_row(&ui.params_model);
        }));

        let params_slot_remove = SlotNoArgs::new(&ui.params_tableview, clone!(
            ui => move || {
            SaveTemplateUI::remove_rows(&ui.params_model, &ui.params_tableview);
        }));

        SaveTemplateUISlots {
            sections_slot_add,
            sections_slot_remove,
            options_slot_add,
            options_slot_remove,
            params_slot_add,
            params_slot_remove
        }
    }
}
