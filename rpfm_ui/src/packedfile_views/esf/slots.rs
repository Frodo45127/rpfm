//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the slots for ESF Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQString;
use qt_core::SlotOfBool;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::esf::PackedFileESFView;
use crate::packedfile_views::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::utils::check_regex;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a ESF PackedFile.
pub struct PackedFileESFViewSlots {
    pub filter_trigger: QBox<SlotNoArgs>,
    pub filter_change_text: QBox<SlotOfQString>,
    pub filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub filter_change_case_sensitive: QBox<SlotOfBool>,
    pub filter_check_regex: QBox<SlotOfQString>,

    pub open_node: QBox<SlotOfQItemSelectionQItemSelection>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileESFViewSlots`.
impl PackedFileESFViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileESFView>,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    )  -> Self {

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotOfQString::new(&view.tree_view, clone!(
            view => move |_| {
                PackedFileESFView::start_delayed_updates_timer(&view);
            }
        ));
        let filter_change_autoexpand_matches = SlotOfBool::new(&view.tree_view, clone!(
            view => move |_| {
                PackedFileESFView::filter_files(&view);
            }
        ));
        let filter_change_case_sensitive = SlotOfBool::new(&view.tree_view, clone!(
            view => move |_| {
                PackedFileESFView::filter_files(&view);
            }
        ));

        // Function triggered by the filter timer.
        let filter_trigger = SlotNoArgs::new(&view.tree_view, clone!(
            view => move || {
                PackedFileESFView::filter_files(&view);
            }
        ));


        // What happens when we trigger the "Check Regex" action.
        let filter_check_regex = SlotOfQString::new(&view.tree_view, clone!(
            view => move |string| {
                check_regex(&string.to_std_string(), view.filter_line_edit.static_upcast(), true);
            }
        ));

        // Slot to change the format of the video to CAMV.
        let open_node = SlotOfQItemSelectionQItemSelection::new(&view.tree_view, clone!(
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui,
            view => move |after, before| {

                // Save the previous data if needed.
                if before.count_0a() == 1 {
                    let filter_index = before.take_at(0).indexes().take_at(0);
                    let index = view.tree_filter.map_to_source(filter_index.as_ref());
                    view.detailed_view.write().unwrap().save_from_detailed_view(&view.tree_view, index.as_ref());
                }

                // Load the new data.
                if after.count_0a() == 1 {
                    let filter_index = after.take_at(0).indexes().take_at(0);
                    let index = view.tree_filter.map_to_source(filter_index.as_ref());
                    view.detailed_view.write().unwrap().load_to_detailed_view(
                        &view.tree_view,
                        index.as_ref(),
                        &app_ui,
                        &global_search_ui,
                        &pack_file_contents_ui,
                        &diagnostics_ui,
                        &dependencies_ui,
                        &references_ui
                    );
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            filter_trigger,
            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,
            filter_check_regex,

            open_node,
        }
    }
}
