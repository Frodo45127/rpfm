//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Module with the slots for UnitVariant Views.

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use getset::Getters;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::integrations::log::info;

use rpfm_ui_common::clone;
use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::packedfile_views::{DataSource, utils::set_modified};
use crate::packfile_contents_ui::PackFileContentsUI;

use super::UnitVariantView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a UnitVariant view.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct UnitVariantSlots {
    modified: QBox<SlotNoArgs>,

    delayed_updates_main: QBox<SlotNoArgs>,
    delayed_updates_variants: QBox<SlotNoArgs>,
    filter_edited_main: QBox<SlotNoArgs>,
    filter_edited_variants: QBox<SlotNoArgs>,
    load_entry_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    load_variant_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    main_list_context_menu: QBox<SlotOfQPoint>,
    main_list_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    variants_list_context_menu: QBox<SlotOfQPoint>,
    variants_list_context_menu_enabler: QBox<SlotOfQItemSelectionQItemSelection>,
    main_list_add: QBox<SlotNoArgs>,
    main_list_clone: QBox<SlotNoArgs>,
    main_list_delete: QBox<SlotNoArgs>,
    variants_list_add: QBox<SlotNoArgs>,
    variants_list_clone: QBox<SlotNoArgs>,
    variants_list_delete: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl UnitVariantSlots {
    pub unsafe fn new(view: &Arc<UnitVariantView>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>)  -> Self {
        let modified = SlotNoArgs::new(view.main_list_view(), clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                info!("Triggering `Modified Unit Variant File` By Slot");

                if let DataSource::PackFile = *view.data_source.read().unwrap() {
                    set_modified(true, &view.path.read().unwrap(), &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        let delayed_updates_main = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
                UnitVariantView::filter_list(view.main_list_filter.as_ref().unwrap(), view.main_filter_line_edit.as_ref().unwrap());
            }
        ));

        let delayed_updates_variants = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
                UnitVariantView::filter_list(view.variants_list_filter.as_ref().unwrap(), view.variants_filter_line_edit.as_ref().unwrap());
            }
        ));

        let filter_edited_main = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
                UnitVariantView::start_delayed_updates_timer(&view.timer_delayed_updates_main.as_ref().unwrap());
            }
        ));

        let filter_edited_variants = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
                UnitVariantView::start_delayed_updates_timer(&view.timer_delayed_updates_variants.as_ref().unwrap());
            }
        ));

        let load_entry_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(view.main_list_view(), clone!(
            view => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_indexes = before.at(0).indexes();
                    let filter_index = filter_indexes.at(0);
                    let index = view.main_list_filter().map_to_source(filter_index);
                    view.save_entry_from_detailed_view(index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_indexes = after.at(0).indexes();
                    let filter_index = filter_indexes.at(0);
                    let index = view.main_list_filter().map_to_source(filter_index);
                    view.load_entry_to_detailed_view(index.as_ref());
                }

                // If nothing is loaded, means we're selecting multiple things, or none.
                // We need to clear the view to ensure no weird shenaningans happen.
                else {
                    view.clear_main_view();
                }
            }
        ));

        let load_variant_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(view.main_list_view(), clone!(
            view => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_indexes = before.at(0).indexes();
                    let filter_index = filter_indexes.at(0);
                    let index = view.variants_list_filter().map_to_source(filter_index);
                    view.save_variant_from_detailed_view(index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_indexes = after.at(0).indexes();
                    let filter_index = filter_indexes.at(0);
                    let index = view.variants_list_filter().map_to_source(filter_index);
                    view.load_variant_to_detailed_view(index.as_ref());
                }

                // If nothing is loaded, means we're selecting multiple things, or none.
                // We need to clear the view to ensure no weird shenaningans happen.
                else {
                    view.clear_variants_view();
                }
            }
        ));

        let main_list_context_menu = SlotOfQPoint::new(view.main_list_view(), clone!(
            view => move |_| {
            view.main_list_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));
        let main_list_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(view.main_list_view(), clone!(
            view => move |after, _| {
                let enabled = after.count_0a() == 1;
                view.main_list_clone.set_enabled(enabled);
                view.main_list_delete.set_enabled(enabled);
            }
        ));

        let variants_list_context_menu = SlotOfQPoint::new(view.main_list_view(), clone!(
            view => move |_| {
            view.variants_list_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));
        let variants_list_context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(view.main_list_view(), clone!(
            view => move |after, _| {
                let enabled = after.count_0a() == 1;
                view.variants_list_clone.set_enabled(enabled);
                view.variants_list_delete.set_enabled(enabled);
            }
        ));

        let main_list_add = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
            let current_values = UnitVariantView::value_list_from_model(&view.main_list_model().static_upcast());

            match view.id_dialog(None, current_values) {
                Ok(new_id) => if let Some(new_id) = new_id {
                    view.add_category(new_id);
                },
                Err(error) => show_dialog(view.main_list_view(), error, false),
            }
        }));
        let main_list_clone = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {

            let selection = view.main_list_view.selection_model().selected_indexes();
            let index = selection.at(0);
            if index.is_valid() {
                let current_values = UnitVariantView::value_list_from_model(&view.main_list_model().static_upcast());

                let source_index = view.main_list_filter.map_to_source(index);
                let item = view.main_list_model.item_from_index(&source_index);
                let data = item.data_1a(2).to_long_long_0a();
                match view.id_dialog(Some(data), current_values) {
                    Ok(new_id) => if let Some(new_id) = new_id {
                        view.clone_category(new_id, index);
                    },
                    Err(error) => show_dialog(view.main_list_view(), error, false),
                }
            }
        }));
        let main_list_delete = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
            view.remove_category(view.main_list_view.selection_model().selected_indexes().at(0))
        }));

        let variants_list_add = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
            view.add_variant();
        }));
        let variants_list_clone = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {

            let selection = view.variants_list_view.selection_model().selected_indexes();
            let index = selection.at(0);
            if index.is_valid() {
                view.clone_variant(index);
            }
        }));
        let variants_list_delete = SlotNoArgs::new(view.main_list_view(), clone!(
            view => move || {
            view.remove_variant(view.variants_list_view.selection_model().selected_indexes().at(0))
        }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            modified,
            delayed_updates_main,
            delayed_updates_variants,
            filter_edited_main,
            filter_edited_variants,
            load_entry_to_detailed_view,
            load_variant_to_detailed_view,
            main_list_context_menu,
            main_list_context_menu_enabler,
            variants_list_context_menu,
            variants_list_context_menu_enabler,
            main_list_add,
            main_list_clone,
            main_list_delete,
            variants_list_add,
            variants_list_clone,
            variants_list_delete,
        }
    }
}
