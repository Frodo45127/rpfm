//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!Module with the slots for RigidModelView Views.

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use getset::Getters;
use qt_core::SlotOfQString;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::integrations::log::info;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::show_dialog;

use super::RigidModelView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a RigidModelView view.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct RigidModelSlots {
    load_data_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    change_version: QBox<SlotOfQString>,
    export_gltf: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl RigidModelSlots {

    /// This function creates a new `RigidModelSlots`.
    pub unsafe fn new(ui: &Arc<RigidModelView>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Self {
        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(&ui.lod_tree_view, clone!(
            app_ui,
            pack_file_contents_ui,
            ui => move |after, _| {
                info!("Triggering 'load_data_to_detailed_view' for Rigid Model view.");

                if after.count_0a() == 1 {
                    let base_index = after.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.lod_tree_filter().map_to_source(filter_index);
                    ui.change_selected_row(Some(index), None, &app_ui, &pack_file_contents_ui);
                } else {
                    ui.change_selected_row(None, None, &app_ui, &pack_file_contents_ui);
                }
            }
        ));

        let change_version = SlotOfQString::new(&ui.lod_tree_view, clone!(
            ui => move |new| {
                info!("Triggering 'change_version' for Rigid Model view.");

                if let Ok(new) = new.to_std_string().parse::<u32>() {
                    if *ui.data.read().unwrap().version() != new {
                        ui.data.write().unwrap().set_version(new);
                    }
                }
            }
        ));

        let export_gltf = SlotNoArgs::new(&ui.lod_tree_view, clone!(
            ui => move || {
                info!("Triggering 'export_gltf' for Rigid Model view.");

                if let Err(error) = ui.export_to_gltf() {
                    show_dialog(ui.detailed_view_groupbox(), error, false);
                }
            }
        ));

        RigidModelSlots {
            load_data_to_detailed_view,
            change_version,
            export_gltf
        }
    }
}
