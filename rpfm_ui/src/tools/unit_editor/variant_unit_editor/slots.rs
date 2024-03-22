//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;
use qt_core::SlotOfQString;

use std::rc::Rc;

use rpfm_ui_common::clone;

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

    pub faction_list_context_menu: QBox<SlotOfQPoint>,
    pub faction_list_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    pub faction_list_add_faction: QBox<SlotNoArgs>,
    pub faction_list_clone_faction: QBox<SlotNoArgs>,
    pub faction_list_delete_faction: QBox<SlotNoArgs>,

    pub unit_variants_colours_list_context_menu: QBox<SlotOfQPoint>,
    pub unit_variants_colours_list_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    pub unit_variants_colours_list_add_colour_variant: QBox<SlotNoArgs>,
    pub unit_variants_colours_list_clone_colour_variant: QBox<SlotNoArgs>,
    pub unit_variants_colours_list_delete_colour_variant: QBox<SlotNoArgs>,

    pub change_icon: QBox<SlotNoArgs>,
    pub change_variant_mesh: QBox<SlotNoArgs>,

    pub add_faction_check: QBox<SlotOfQString>,
    pub add_colour_variant_check: QBox<SlotOfQString>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SubToolVariantUnitEditorSlots`.
impl SubToolVariantUnitEditorSlots {

    /// This function creates a new `SubToolVariantUnitEditorSlots`.
    pub unsafe fn new(ui: &Rc<SubToolVariantUnitEditor>) -> Self {

        let delayed_updates = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.filter_list();
            }
        ));

        let filter_edited = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.start_delayed_updates_timer();
            }
        ));

        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_index = before.at(0).indexes().take_at(0);
                    let index = ui.faction_list_filter().map_to_source(filter_index.as_ref());
                    ui.save_from_detailed_view(index.as_ref());
                    ui.detailed_view_widget.set_enabled(false);
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_index = after.at(0).indexes().take_at(0);
                    let index = ui.faction_list_filter().map_to_source(filter_index.as_ref());
                    ui.load_to_detailed_view(index.as_ref());
                    ui.detailed_view_widget.set_enabled(true);
                }
            }
        ));

        let load_unit_variants_colours_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_index = before.at(0).indexes().take_at(0);
                    let index = ui.unit_variants_colours_list_filter().map_to_source(filter_index.as_ref());
                    ui.save_unit_variants_colours_from_detailed_view(index.as_ref());
                    ui.unit_variants_colours_widget.set_enabled(false);
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_index = after.at(0).indexes().take_at(0);
                    let index = ui.unit_variants_colours_list_filter().map_to_source(filter_index.as_ref());
                    ui.load_unit_variants_colours_to_detailed_view(index.as_ref());
                    ui.unit_variants_colours_widget.set_enabled(true);
                }
            }
        ));

        let faction_list_context_menu = SlotOfQPoint::new(ui.tool.main_widget(), clone!(
            ui => move |_| {
            ui.faction_list_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let faction_list_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, _| {
                let enabled = after.count_0a() == 1;
                ui.faction_list_clone_faction.set_enabled(enabled);

                if enabled && after.at(0).indexes().take_at(0).data_0a().to_string().to_std_string() == "*" {
                    ui.faction_list_delete_faction.set_enabled(false);
                } else {
                    ui.faction_list_delete_faction.set_enabled(enabled);
                }
            }
        ));

        let faction_list_add_faction = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.load_add_faction_dialog() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let faction_list_clone_faction = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.load_clone_faction_dialog() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let faction_list_delete_faction = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.delete_faction() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let unit_variants_colours_list_context_menu = SlotOfQPoint::new(ui.tool.main_widget(), clone!(
            ui => move |_| {
            ui.unit_variants_colours_list_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let unit_variants_colours_list_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, _| {
                let enabled = after.count_0a() == 1;
                ui.unit_variants_colours_list_clone_colour_variant.set_enabled(enabled);
                ui.unit_variants_colours_list_delete_colour_variant.set_enabled(enabled);
            }
        ));

        let unit_variants_colours_list_add_colour_variant = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.load_add_colour_variant_dialog() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let unit_variants_colours_list_clone_colour_variant = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.load_clone_colour_variant_dialog() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let unit_variants_colours_list_delete_colour_variant = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if let Err(error) = ui.delete_colour_variant() {
                    show_message_warning(&ui.tool.message_widget, error);
                }
            }
        ));

        let change_icon = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                let key = ui.unit_variants_unit_card_combobox.current_text().to_std_string();
                let _ = ui.load_unit_icon(&HashMap::new(), Some(key));
            }
        ));

        let change_variant_mesh = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                let key = ui.variants_variant_filename_combobox.current_text().to_std_string();
                ui.load_variant_mesh(&HashMap::new(), Some(key));
            }
        ));

        let add_faction_check = SlotOfQString::new(ui.tool.main_widget(), clone!(
            ui => move |value| {
                let ok_button = ui.new_faction_button_box.button(q_dialog_button_box::StandardButton::Ok);
                ok_button.set_enabled(ui.faction_list_model.find_items_1a(value).is_empty());
            }
        ));

        let add_colour_variant_check = SlotOfQString::new(ui.tool.main_widget(), clone!(
            ui => move |value| {

                // TODO: Make this check against the full key list, not just a subset.
                let enabled = value.to_std_string().parse::<i32>().is_ok() && ui.unit_variants_colours_list_model.find_items_1a(value).is_empty();
                let ok_button = ui.new_colour_variant_button_box.button(q_dialog_button_box::StandardButton::Ok);
                ok_button.set_enabled(enabled);
            }
        ));

        SubToolVariantUnitEditorSlots {
            delayed_updates,
            filter_edited,
            load_data_to_detailed_view,
            load_unit_variants_colours_to_detailed_view,

            faction_list_context_menu,
            faction_list_context_menu_enabler,
            faction_list_add_faction,
            faction_list_clone_faction,
            faction_list_delete_faction,

            unit_variants_colours_list_context_menu,
            unit_variants_colours_list_context_menu_enabler,
            unit_variants_colours_list_add_colour_variant,
            unit_variants_colours_list_clone_colour_variant,
            unit_variants_colours_list_delete_colour_variant,

            change_icon,
            change_variant_mesh,

            add_faction_check,
            add_colour_variant_check,
        }
    }
}
