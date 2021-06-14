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
Module with the slots for ESF Views.
!*/

use qt_core::QBox;
use qt_core::QString;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQString;
use qt_core::SlotOfBool;

use std::sync::Arc;

use crate::packedfile_views::esf::{esftree::ESFTree, PackedFileESFView};
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

    pub open_node: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileESFViewSlots`.
impl PackedFileESFViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileESFView>,
        //app_ui: &Rc<AppUI>,
        //pack_file_contents_ui: &Rc<PackFileContentsUI>,
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
                check_regex(&string.to_std_string(), view.filter_line_edit.static_upcast());
            }
        ));

        // Slot to change the format of the video to CAMV.
        let open_node = SlotNoArgs::new(&view.tree_view, clone!(
            //app_ui,
            //pack_file_contents_ui,
            view => move || {
                let items = view.tree_view.get_items_from_selection(true);
                if items.len() == 1 {
                    let data = items[0].data_1a(40).to_string();
                    crate::ffi::set_text_safe(&view.editor, &data.as_ptr(), &QString::from_std_str("json").as_ptr());
                    //dbg!(data);
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

