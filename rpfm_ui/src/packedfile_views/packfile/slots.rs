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
Module with the slots for PackFile Views.
!*/

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::{SlotOfBool, SlotOfQModelIndex, SlotNoArgs, SlotOfQString};
use qt_core::QBox;

use std::sync::Arc;
use std::rc::Rc;

use rpfm_lib::files::ContainerPath;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::utils::show_dialog;
use super::PackFileExtraView;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of the extra PackFile.
pub struct PackFileExtraViewSlots {
    pub import: QBox<SlotOfQModelIndex>,

    pub filter_change_text: QBox<SlotOfQString>,
    pub filter_change_autoexpand_matches: QBox<SlotOfBool>,
    pub filter_change_case_sensitive: QBox<SlotOfBool>,

    pub context_menu: QBox<SlotOfQPoint>,
    pub expand_all: QBox<SlotNoArgs>,
    pub collapse_all: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileExtraViewSlots`.
impl PackFileExtraViewSlots {

    /// This function builds the entire slot set for the provided PackFileExtraView.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        pack_file_view: &Arc<PackFileExtraView>
    ) -> Self {

        // When we want to import the selected PackedFile...
        let import = SlotOfQModelIndex::new(&pack_file_view.tree_view, clone!(
            app_ui,
            pack_file_contents_ui,
            pack_file_view => move |_| {

                // Get the file to get from the TreeView.
                let selection_file_to_move = pack_file_view.tree_view.selection_model().selection();
                if selection_file_to_move.count_0a() == 1 {
                    let item_types = pack_file_view.tree_view.get_item_types_from_selection_filtered();

                    // Ask the Background Thread to move the files, and send him the path.
                    app_ui.toggle_main_window(false);
                    let receiver = CENTRAL_COMMAND.send_background(Command::AddPackedFilesFromPackFile((pack_file_view.pack_file_path.read().unwrap().to_path_buf(), item_types)));
                    let response = CentralCommand::recv(&receiver);
                    match response {
                        Response::VecContainerPath(paths_ok) => {

                            // If any of the PackedFiles was already open (and we overwrote them) remove his view.
                            for path in &paths_ok {
                                if let ContainerPath::File(path) = path {
                                    let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                                    if let Some(packed_file_view) = open_packedfiles.iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                                        if packed_file_view.reload(path, &pack_file_contents_ui).is_err() {
                                            let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, path, DataSource::PackFile, false);
                                        }
                                    }
                                }
                            }

                            // Update the TreeView.
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths_ok.to_vec()), DataSource::PackFile);
                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_ok.to_vec()), DataSource::PackFile);
                            UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                    }

                    // Re-enable the Main Window.
                    app_ui.toggle_main_window(true);
                    pack_file_view.tree_view.set_focus_0a();
                }
            }
        ));

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotOfQString::new(&pack_file_view.tree_view, clone!(pack_file_view => move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        }));
        let filter_change_autoexpand_matches = SlotOfBool::new(&pack_file_view.tree_view, clone!(pack_file_view => move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        }));
        let filter_change_case_sensitive = SlotOfBool::new(&pack_file_view.tree_view, clone!(pack_file_view => move |_| {
            PackFileExtraView::filter_files(&pack_file_view);
        }));

        // Slot to show the Contextual Menu for the TreeView.
        let context_menu = SlotOfQPoint::new(&pack_file_view.tree_view, clone!(
            pack_file_view => move |_| {
            pack_file_view.context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // Actions without buttons for the TreeView.
        let expand_all = SlotNoArgs::new(&pack_file_view.tree_view, clone!(pack_file_view => move || { pack_file_view.tree_view.expand_all(); }));
        let collapse_all = SlotNoArgs::new(&pack_file_view.tree_view, clone!(pack_file_view => move || { pack_file_view.tree_view.collapse_all(); }));

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            import,

            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,

            context_menu,
            expand_all,
            collapse_all,
        }
    }
}
