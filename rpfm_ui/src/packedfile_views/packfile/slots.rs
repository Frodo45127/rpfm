//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::widget::Widget;

use qt_core::slots::{SlotBool, SlotModelIndexRef, SlotNoArgs, SlotStringRef};

use rpfm_error::ErrorKind;
use rpfm_lib::packfile::PathType;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::utils::show_dialog;
use super::{PackFileExtraView, PackFileExtraViewRaw};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of the extra PackFile.
pub struct PackFileExtraViewSlots {
    pub import: SlotModelIndexRef<'static>,

    pub filter_change_text: SlotStringRef<'static>,
    pub filter_change_autoexpand_matches: SlotBool<'static>,
    pub filter_change_case_sensitive: SlotBool<'static>,

    pub expand_all: SlotNoArgs<'static>,
    pub collapse_all: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraViewSlots`.
impl PackFileExtraViewSlots {

    /// This function builds the entire slot set for the provided PackFileExtraView.
    pub fn new(app_ui: AppUI, pack_file_contents_view: PackFileContentsUI, global_search_ui: GlobalSearchUI, pack_file_view: PackFileExtraViewRaw) -> Self {

        // When we want to import the selected PackedFile...
        let import = SlotModelIndexRef::new(move |_| {

                // Get the file to get from the TreeView.
                let selection_file_to_move = unsafe { pack_file_view.tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection() };
                if selection_file_to_move.count(()) == 1 {
                    let item_types = pack_file_view.tree_view.get_item_types_from_selection_filtered().iter().map(|x| From::from(x)).collect();

                    // Ask the Background Thread to move the files, and send him the path.
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    CENTRAL_COMMAND.send_message_qt(Command::AddPackedFileFromPackFile(item_types));
                    match CENTRAL_COMMAND.recv_message_qt() {
                        Response::VecPathType(paths_ok) => {

                            // If any of the PackedFiles was already open (and we overwote them) remove his view.
                            for path in &paths_ok {
                                if let PathType::File(path) = path {
                                    app_ui.purge_that_one_specifically(global_search_ui, &path, false);
                                }
                            }

                            // Update the TreeView.
                            let paths_ok = paths_ok.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();
                            pack_file_contents_view.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok));

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
                            unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

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
                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                        _ => panic!(THREADS_COMMUNICATION_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    unsafe { pack_file_view.tree_view.as_mut().unwrap().set_focus(()); }
                }
            }
        );

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotStringRef::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });
        let filter_change_autoexpand_matches = SlotBool::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });
        let filter_change_case_sensitive = SlotBool::new(move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        });

        // Actions without buttons for the TreeView.
        let expand_all = SlotNoArgs::new(move || { unsafe { pack_file_view.tree_view.as_mut().unwrap().expand_all(); }});
        let collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_view.tree_view.as_mut().unwrap().collapse_all(); }});

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
