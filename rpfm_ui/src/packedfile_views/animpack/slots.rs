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
Module with the slots for AnimPack Views.
!*/

use qt_core::QBox;
use qt_core::SlotNoArgs;

use std::rc::Rc;
use std::sync::Arc;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::animpack::PackedFileAnimPackView;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimPack PackedFile.
pub struct PackedFileAnimPackViewSlots {
    pub unpack: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimPackViewSlots`.
impl PackedFileAnimPackViewSlots {

    /// This function creates the entire slot pack for AnimPack PackedFile Views.
    pub unsafe fn new(
        view: &Arc<PackedFileAnimPackView>,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    )  -> Self {

        // Slot to unpack the entire AnimPack.
        let unpack = SlotNoArgs::new(&view.file_count_data_label, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {

                CENTRAL_COMMAND.send_message_qt(Command::AnimPackUnpack(view.path.read().unwrap().to_vec()));
                let response = CENTRAL_COMMAND.recv_message_qt();
                match response {
                    Response::VecVecString(paths_packedfile) => {
                        let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()));
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths.to_vec()));
                        UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);

                        // Try to reload all open files which data we altered, and close those that failed.
                        let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                        paths_packedfile.iter().for_each(|path| {
                            if let Some(packed_file_view) = open_packedfiles.iter_mut().find(|x| *x.get_ref_path() == *path) {
                                if packed_file_view.reload(path, &pack_file_contents_ui).is_err() {
                                    let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, false);
                                }
                            }
                        });
                    }

                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }
            }
        ));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            unpack,
        }
    }
}

