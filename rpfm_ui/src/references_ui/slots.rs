//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `ReferencesUISlots`.
!*/

use qt_core::QBox;
use qt_core::SlotOfQModelIndex;

use rpfm_lib::integrations::log::*;

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use super::ReferencesUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the references panel.
pub struct ReferencesUISlots {
    pub references_open_result: QBox<SlotOfQModelIndex>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ReferencesUISlots`.
impl ReferencesUISlots {

    /// This function creates an entire `ReferencesUISlots` struct.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Self {

        // What happens when we try to open the file corresponding to one of the matches.
        let references_open_result = SlotOfQModelIndex::new(&references_ui.references_dock_widget, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui => move |model_index_filter| {
                info!("Triggering `Open Reference Match` By Slot");
                ReferencesUI::open_match(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui, model_index_filter.as_ptr());
            }
        ));

        // And here... we return all the slots.
        Self {
            references_open_result,
        }
    }
}
