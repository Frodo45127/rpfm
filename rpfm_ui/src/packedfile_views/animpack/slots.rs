//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use rpfm_lib::files::ContainerPath;

use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::DataSource;
use crate::packedfile_views::animpack::PackedFileAnimPackView;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::setting_bool;
use crate::UI_STATE;
use crate::utils::show_dialog;
use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimPack PackedFile.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackedFileAnimPackViewSlots {
    copy_in: QBox<SlotOfQModelIndex>,
    copy_out: QBox<SlotOfQModelIndex>,
    delete: QBox<SlotNoArgs>,

    pack_filter_change_text: QBox<SlotOfQString>,
    pack_filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pack_filter_change_case_sensitive: QBox<SlotOfBool>,
    anim_pack_filter_change_text: QBox<SlotOfQString>,
    anim_pack_filter_change_autoexpand_matches: QBox<SlotOfBool>,
    anim_pack_filter_change_case_sensitive: QBox<SlotOfBool>,

    pack_expand_all: QBox<SlotNoArgs>,
    pack_collapse_all: QBox<SlotNoArgs>,
    anim_pack_expand_all: QBox<SlotNoArgs>,
    anim_pack_collapse_all: QBox<SlotNoArgs>,
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

                // Do not add files to the animpack if its not in our own Pack.
                if *view.data_source.read().unwrap() != DataSource::PackFile {
                    return;
                }

                // Get the file to get from the TreeView.
                let selection_file_to_move = view.pack_tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = view.pack_tree_view.get_item_types_from_selection_filtered();

                    // Save the files in question to the background, to ensure we have all their data updated.
                    for file_view in UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile) {
                        let _ = file_view.save(&app_ui, &pack_file_contents_ui);
                    }

                    // Ask the Background Thread to copy the files, and send him the path.
                    app_ui.toggle_main_window(false);
                    let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFilesFromPackFileToAnimpack(view.path().read().unwrap().to_owned(), item_types));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecContainerPath(paths_ok) => {

                            // Update the AnimPack TreeView with the new files.
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile);
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile);

                            // Mark the AnimPack in the PackFile as modified.
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::File(view.path().read().unwrap().to_owned()); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }

                    // Re-enable and re-focus the Main Window.
                    app_ui.toggle_main_window(true);
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

                // Do not import if we don't have an open pack.
                if view.pack_tree_model_filter().source_model().row_count_0a() == 0 {
                    return;
                }

                // Get the file to get from the TreeView.
                let selection_file_to_move = view.anim_pack_tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = view.anim_pack_tree_view.get_item_types_from_selection_filtered();

                    // Ask the Background Thread to copy the files, and send him the path.
                    app_ui.toggle_main_window(false);
                    let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFilesFromAnimpack(*view.data_source.read().unwrap(), view.path().read().unwrap().to_owned(), item_types));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecContainerPath(paths_ok) => {

                            // Update the AnimPack TreeView with the new files.
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile);
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);

                            // Reload all the views belonging to overwritten files.
                            for file_view in UI_STATE.set_open_packedfiles().iter_mut() {
                                for path_ok in &paths_ok {
                                    if path_ok.path_raw() == file_view.path_copy() && file_view.data_source() == DataSource::PackFile {
                                        let _ = file_view.reload(path_ok.path_raw(), &pack_file_contents_ui);
                                    }
                                }
                            }
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }

                    // Re-enable and re-focus the Main Window.
                    app_ui.toggle_main_window(true);
                    view.pack_tree_view.clear_focus();
                    view.pack_tree_view.set_focus_0a();

                    PackFileContentsUI::start_delayed_updates_timer(&pack_file_contents_ui);
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
                    let item_types = view.anim_pack_tree_view.get_item_types_from_selection_filtered();

                    // Ask the backend to delete them.
                    let receiver = CENTRAL_COMMAND.send_background(Command::DeleteFromAnimpack((view.path().read().unwrap().to_owned(), item_types.clone())));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::Success => {

                            // If it works, remove them from the view.
                            view.anim_pack_tree_view.update_treeview(true, TreeViewOperation::Delete(item_types, setting_bool("delete_empty_folders_on_delete")), DataSource::PackFile);

                            // Mark the AnimPack in the PackFile as modified.
                            view.pack_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::File(view.path().read().unwrap().to_owned()); 1]), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        }

                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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

