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
Module with all the code related to `SubToolVariantUnitEditorSlots`.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::rc::Rc;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action in the `SubToolVariantUnitEditor` struct.
///
/// This means everything you can do with the stuff you have in the `SubToolVariantUnitEditor` goes here.
pub struct SubToolVariantUnitEditorSlots {
    pub delayed_updates: QBox<SlotNoArgs>,
    pub filter_edited: QBox<SlotNoArgs>,
    pub load_data_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    pub load_unit_variants_colours_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,

    pub change_icon: QBox<SlotNoArgs>,
    pub change_variant_mesh: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SubToolVariantUnitEditorSlots`.
impl SubToolVariantUnitEditorSlots {

    /// This function creates a new `SubToolVariantUnitEditorSlots`.
    pub unsafe fn new(ui: &Rc<SubToolVariantUnitEditor>) -> Self {

        let delayed_updates = SlotNoArgs::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move || {
                ui.filter_list();
            }
        ));

        let filter_edited = SlotNoArgs::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move || {
                ui.start_delayed_updates_timer();
            }
        ));

        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_index = before.take_at(0).indexes().take_at(0);
                    let index = ui.get_ref_faction_list_filter().map_to_source(filter_index.as_ref());
                    ui.save_from_detailed_view(index.as_ref());
                    ui.detailed_view_widget.set_enabled(false);
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_index = after.take_at(0).indexes().take_at(0);
                    let index = ui.get_ref_faction_list_filter().map_to_source(filter_index.as_ref());
                    ui.load_to_detailed_view(index.as_ref());
                    ui.detailed_view_widget.set_enabled(true);
                }
            }
        ));

        let load_unit_variants_colours_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_index = before.take_at(0).indexes().take_at(0);
                    let index = ui.get_ref_unit_variants_colours_list_filter().map_to_source(filter_index.as_ref());
                    ui.save_unit_variants_colours_from_detailed_view(index.as_ref());
                    ui.unit_variants_colours_widget.set_enabled(false);
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_index = after.take_at(0).indexes().take_at(0);
                    let index = ui.get_ref_unit_variants_colours_list_filter().map_to_source(filter_index.as_ref());
                    ui.load_unit_variants_colours_to_detailed_view(index.as_ref());
                    ui.unit_variants_colours_widget.set_enabled(true);
                }
            }
        ));

        let change_icon = SlotNoArgs::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move || {
                let key = ui.unit_variants_unit_card_combobox.current_text().to_std_string();
                ui.load_unit_icon(&HashMap::new(), Some(key));
            }
        ));

        let change_variant_mesh = SlotNoArgs::new(ui.tool.get_ref_main_widget(), clone!(
            ui => move || {
                let key = ui.variants_variant_filename_combobox.current_text().to_std_string();
                ui.load_variant_mesh(&HashMap::new(), Some(key));
            }
        ));

        SubToolVariantUnitEditorSlots {
            delayed_updates,
            filter_edited,
            load_data_to_detailed_view,
            load_unit_variants_colours_to_detailed_view,

            change_icon,
            change_variant_mesh,
        }
    }
}
