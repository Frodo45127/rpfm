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
    pub toggle_required: QBox<SlotNoArgs>,
}


/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SaveTemplateUI` struct.
///
/// This means everything you can do with the stuff you have in the `SaveTemplateUI` goes here.
pub struct SaveTemplateUISlots {
    pub step_1_slot_add: QBox<SlotNoArgs>,
    pub step_1_slot_remove: QBox<SlotNoArgs>,
    pub step_2_slot_add: QBox<SlotNoArgs>,
    pub step_2_slot_remove: QBox<SlotNoArgs>,
    pub step_3_slot_add: QBox<SlotNoArgs>,
    pub step_3_slot_remove: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TemplateUISlots`.
impl TemplateUISlots {

    /// This function creates a new `TemplateUISlots`.
    pub unsafe fn new(ui: &Rc<TemplateUI>) -> Self {
        let toggle_required = SlotNoArgs::new(&ui.dialog, clone!(
            ui => move || {
            ui.update_template_view();
        }));
        TemplateUISlots {
            toggle_required,
        }
    }
}

/// Implementation of `SaveTemplateUISlots`.
impl SaveTemplateUISlots {

    /// This function creates a new `SaveTemplateUISlots`.
    pub unsafe fn new(ui: &Rc<SaveTemplateUI>) -> Self {

        // Slots for step 1
        let step_1_slot_add = SlotNoArgs::new(&ui.step_1_tableview, clone!(
            ui => move || {
            let qlist_boi = QListOfQStandardItem::new();

            let key = QStandardItem::new();
            let value = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());

            ui.step_1_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
        }));

        let step_1_slot_remove = SlotNoArgs::new(&ui.step_1_tableview, clone!(
            ui => move || {
            let indexes = ui.step_1_tableview.selection_model().selection().indexes();
            let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
            let rows_sorted = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();

            crate::views::table::utils::delete_rows(&ui.step_1_model.static_upcast(), &rows_sorted);
        }));

        // Slots for step 2
        let step_2_slot_add = SlotNoArgs::new(&ui.step_2_tableview, clone!(
            ui => move || {
            let qlist_boi = QListOfQStandardItem::new();

            let key = QStandardItem::new();
            let value = QStandardItem::new();
            let section = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());

            ui.step_2_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
        }));

        let step_2_slot_remove = SlotNoArgs::new(&ui.step_2_tableview, clone!(
            ui => move || {
            let indexes = ui.step_2_tableview.selection_model().selection().indexes();
            let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
            let rows_sorted = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();

            crate::views::table::utils::delete_rows(&ui.step_2_model.static_upcast(), &rows_sorted);
        }));

        // Slots for step 3
        let step_3_slot_add = SlotNoArgs::new(&ui.step_3_tableview, clone!(
            ui => move || {
            let qlist_boi = QListOfQStandardItem::new();

            let key = QStandardItem::new();
            let value = QStandardItem::new();
            let section = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());

            ui.step_3_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
        }));

        let step_3_slot_remove = SlotNoArgs::new(&ui.step_3_tableview, clone!(
            ui => move || {
            let indexes = ui.step_3_tableview.selection_model().selection().indexes();
            let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
            let rows_sorted = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();

            crate::views::table::utils::delete_rows(&ui.step_3_model.static_upcast(), &rows_sorted);
        }));

        SaveTemplateUISlots {
            step_1_slot_add,
            step_1_slot_remove,
            step_2_slot_add,
            step_2_slot_remove,
            step_3_slot_add,
            step_3_slot_remove
        }
    }
}
