//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::rc::Rc;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct ToolTranslatorSlots {
    load_data_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ToolTranslatorSlots {

    /// This function creates a new `ToolTranslatorSlots`.
    pub unsafe fn new(ui: &Rc<ToolTranslator>) -> Self {

        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, before| {
                info!("Triggering 'load_data_to_detailed_view' for Translator.");

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let base_index = before.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.save_from_detailed_view(index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let base_index = after.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.load_to_detailed_view(index.as_ref());
                }
            }
        ));

        ToolTranslatorSlots {
            load_data_to_detailed_view,
        }
    }
}
