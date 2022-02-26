//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
use qt_core::{SlotOfBool, SlotOfQString, SlotNoArgs, SlotOfQModelIndex};

use std::rc::Rc;
use std::sync::Arc;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::DataSource;
use crate::packedfile_views::animpack::PackedFileAnimPackView;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;
use crate::utils::show_dialog;
use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimPack PackedFile.
pub struct PackedFileAnimPackViewSlots {
    pub copy_in: QBox<SlotOfQModelIndex>,
    pub copy_out: QBox<SlotOfQModelIndex>,
    pub delete: QBox<SlotNoArgs>,

    pub pack_filter_change_text: QBox<SlotOfQString>,
    pub pack_filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub pack_filter_change_case_sensitive: QBox<SlotOfBool>,
    pub anim_pack_filter_change_text: QBox<SlotOfQString>,
    pub anim_pack_filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub anim_pack_filter_change_case_sensitive: QBox<SlotOfBool>,

    pub pack_expand_all: QBox<SlotNoArgs>,
    pub pack_collapse_all: QBox<SlotNoArgs>,
    pub anim_pack_expand_all: QBox<SlotNoArgs>,
    pub anim_pack_collapse_all: QBox<SlotNoArgs>,
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

        // Slot to copy stuff from the PackFile into the AnimPack.
        let copy_in = SlotOfQModelIndex::new(&view.pack_tree_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |_| {

                // Get the file to get from the TreeView.
                let selection_file_to_move = view.pack_tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = view.pack_tree_view.get_item_types_from_selection_filtered().iter().map(From::from).collect();

                    // Save the files in question to the background, to ensure we have all their data updated.
                    for packed_file_view in UI_STATE.get_open_packedfiles().iter().filter(|x| x.get_data_source() == DataSource::PackFile) {
                        let _ = packed_file_view.save(&app_ui, &pack_file_contents_ui);
                    }

                    // Ask the Background Thread to copy the files, and send him the path.
                    app_ui.main_window.set_enabled(false);
                    let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFilesFromPackFileToAnimpack((view.get_ref_path().read().unwrap().to_vec(), item_types)));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecPathType(paths_ok) => {

                            // Update the AnimPack TreeView with the new files.
                            let paths_ok = paths_ok.iter().map(From::from).collect::<Vec<TreePathType>>();
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile);
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile);

                            // Mark the AnimPack in the PackFile as modified.
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::File(view.get_ref_path().read().unwrap().to_vec()); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        },
                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    // Re-enable and re-focus the Main Window.
                    app_ui.main_window.set_enabled(true);
                    view.pack_tree_view.clear_focus();
                    view.pack_tree_view.set_focus_0a();
                }
            }
        ));

        // Slot to copy stuff from our AnimPack to the open PackFile.
        let copy_out = SlotOfQModelIndex::new(&view.pack_tree_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move |_| {

                // Get the file to get from the TreeView.
                let selection_file_to_move = view.anim_pack_tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = view.anim_pack_tree_view.get_item_types_from_selection_filtered().iter().map(From::from).collect();

                    // Ask the Background Thread to copy the files, and send him the path.
                    app_ui.main_window.set_enabled(false);
                    let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFilesFromAnimpack((view.get_ref_path().read().unwrap().to_vec(), item_types)));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecPathType(paths_ok) => {

                            // Update the AnimPack TreeView with the new files.
                            let paths_ok = paths_ok.iter().map(From::from).collect::<Vec<TreePathType>>();
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile);
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);

                            // Reload all the views belonging to overwritten files.
                            for packed_file_view in UI_STATE.set_open_packedfiles().iter_mut() {
                                for path_ok in &paths_ok {
                                    if let TreePathType::File(path) = path_ok {
                                        if path == &packed_file_view.get_path() && packed_file_view.get_data_source() == DataSource::PackFile {
                                            let _ = packed_file_view.reload(path, &pack_file_contents_ui);
                                        }
                                    }
                                }
                            }
                        },
                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    // Re-enable and re-focus the Main Window.
                    app_ui.main_window.set_enabled(true);
                    view.pack_tree_view.clear_focus();
                    view.pack_tree_view.set_focus_0a();
                }
            }
        ));

        // Slot to delele files from our AnimPack.
        let delete = SlotNoArgs::new(&view.pack_tree_view, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {

                // Get the file to delete from the TreeView.
                let selection_file_to_move = view.anim_pack_tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let tree_item_types = view.anim_pack_tree_view.get_item_types_from_selection_filtered();
                    let item_types = tree_item_types.iter().map(From::from).collect();

                    // Ask the backend to delete them.
                    let receiver = CENTRAL_COMMAND.send_background(Command::DeleteFromAnimpack((view.path.read().unwrap().to_vec(), item_types)));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::Success => {

                            // If it works, remove them from the view.
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::Delete(tree_item_types), DataSource::PackFile);

                            // Mark the AnimPack in the PackFile as modified.
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![TreePathType::File(view.get_ref_path().read().unwrap().to_vec()); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }
                }
            }
        ));

        let pack_filter_change_text = SlotOfQString::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, false);
            }
        ));
        let pack_filter_change_autoexpand_matches = SlotOfBool::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, false);
            }
        ));
        let pack_filter_change_case_sensitive = SlotOfBool::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, false);
            }
        ));

        let anim_pack_filter_change_text = SlotOfQString::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, true);
            }
        ));
        let anim_pack_filter_change_autoexpand_matches = SlotOfBool::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, true);
            }
        ));
        let anim_pack_filter_change_case_sensitive = SlotOfBool::new(&view.pack_tree_view.parent(), clone!(
            view => move |_| {
                PackedFileAnimPackView::filter_files(&view, true);
            }
        ));

        // Actions without buttons for the TreeView.
        let pack_expand_all = SlotNoArgs::new(&view.pack_tree_view.parent(), clone!(view => move || { view.pack_tree_view.expand_all(); }));
        let pack_collapse_all = SlotNoArgs::new(&view.pack_tree_view.parent(), clone!(view => move || { view.pack_tree_view.collapse_all(); }));
        let anim_pack_expand_all = SlotNoArgs::new(&view.pack_tree_view.parent(), clone!(view => move || { view.anim_pack_tree_view.expand_all(); }));
        let anim_pack_collapse_all = SlotNoArgs::new(&view.pack_tree_view.parent(), clone!(view => move || { view.anim_pack_tree_view.collapse_all(); }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            copy_in,
            copy_out,
            delete,
            pack_filter_change_text,
            pack_filter_change_autoexpand_matches,
            pack_filter_change_case_sensitive,
            anim_pack_filter_change_text,
            anim_pack_filter_change_autoexpand_matches,
            anim_pack_filter_change_case_sensitive,
            pack_expand_all,
            pack_collapse_all,
            anim_pack_expand_all,
            anim_pack_collapse_all,
        }
    }
}

