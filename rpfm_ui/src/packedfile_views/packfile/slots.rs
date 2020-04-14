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
Module with the slots for PackFile Views.
!*/

use qt_core::{SlotOfBool, SlotOfQModelIndex, Slot, SlotOfQString};

use rpfm_lib::packfile::PathType;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::utils::show_dialog;
use super::{PackFileExtraView, PackFileExtraViewRaw};
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of the extra PackFile.
pub struct PackFileExtraViewSlots {
    pub import: SlotOfQModelIndex<'static>,

    pub filter_change_text: SlotOfQString<'static>,
    pub filter_change_autoexpand_matches: SlotOfBool<'static>,
    pub filter_change_case_sensitive: SlotOfBool<'static>,

    pub expand_all: Slot<'static>,
    pub collapse_all: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraViewSlots`.
impl PackFileExtraViewSlots {

    /// This function builds the entire slot set for the provided PackFileExtraView.
    pub unsafe fn new(mut app_ui: AppUI, mut pack_file_contents_ui: PackFileContentsUI, global_search_ui: GlobalSearchUI, mut pack_file_view: PackFileExtraViewRaw) -> Self {

        // When we want to import the selected PackedFile...
        let import = SlotOfQModelIndex::new(move |_| {

                // Get the file to get from the TreeView.
                let selection_file_to_move = pack_file_view.tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = pack_file_view.tree_view.get_item_types_from_selection_filtered().iter().map(From::from).collect();

                    // Ask the Background Thread to move the files, and send him the path.
                    app_ui.main_window.set_enabled(false);
                    CENTRAL_COMMAND.send_message_qt(Command::AddPackedFilesFromPackFile(item_types));
                    let response = CENTRAL_COMMAND.recv_message_qt();
                    match response {
                        Response::VecPathType(paths_ok) => {

                            // If any of the PackedFiles was already open (and we overwote them) remove his view.
                            for path in &paths_ok {
                                if let PathType::File(path) = path {
                                    app_ui.purge_that_one_specifically(global_search_ui, pack_file_contents_ui, &path, false);
                                }
                            }

                            // Update the TreeView.
                            let paths_ok = paths_ok.iter().map(From::from).collect::<Vec<TreePathType>>();
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()));
                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()));
                            UI_STATE.set_is_modified(true, &mut app_ui, &mut pack_file_contents_ui);
/*
                            // Update the global search stuff, if needed.
                            let paths = paths.iter().map(|x|
                                match x {
                                    TreePathType::File(ref path) => path.to_vec(),
                                    TreePathType::Folder(ref path) => path.to_vec(),
                                    TreePathType::PackFile => vec![],
                                    TreePathType::None => unimplemented!(),
                                }
                            ).collect::<Vec<Vec<String>>>();
                            global_search_explicit_paths.borrow_mut().append(&mut paths.to_vec());
                            unsafe { update_global_search_stuff.trigger(); }

                            // For each file added, remove it from the data history if exists.
                            for path in &paths {
                                if table_state_data.borrow().get(path).is_some() {
                                    table_state_data.borrow_mut().remove(path);
                                }

                                // Set it to not remove his color.
                                let data = TableStateData::new_empty();
                                table_state_data.borrow_mut().insert(path.to_vec(), data);
                            }
                            */
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    // Re-enable the Main Window.
                    app_ui.main_window.set_enabled(true);
                    pack_file_view.tree_view.set_focus_0a();
                }
            }
        );

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotOfQString::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });
        let filter_change_autoexpand_matches = SlotOfBool::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });
        let filter_change_case_sensitive = SlotOfBool::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });

        // Actions without buttons for the TreeView.
        let expand_all = Slot::new(move || { pack_file_view.tree_view.expand_all(); });
        let collapse_all = Slot::new(move || { pack_file_view.tree_view.collapse_all(); });

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            import,

            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,

            expand_all,
            collapse_all,
        }
    }
}
