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
Module with all the code for utility functions for `PackFileContentsUI`.

This module contains the implementation of custom functions for `PackFileContentsUI`.
The reason they're here and not in the main file is because I don't want to polute
that one, as it's mostly meant for initialization and configuration.
!*/

use qt_widgets::widget::Widget;

use qt_core::qt::CaseSensitivity;
use qt_core::reg_exp::RegExp;

use std::path::PathBuf;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::trigger_treeview_filter;
use crate::pack_tree::{check_if_path_is_closed, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::show_dialog;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub fn add_packedfiles(&self, app_ui: &AppUI, paths: &[PathBuf], paths_packedfile: &[Vec<String>]) {
        if check_if_path_is_closed(&app_ui, paths_packedfile) {
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

            CENTRAL_COMMAND.send_message_qt(Command::AddPackedFiles((paths.to_vec(), paths_packedfile.to_vec())));
            match CENTRAL_COMMAND.recv_message_qt() {
                Response::Success => {
                    let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                    self.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths));

                    // Update the global search stuff, if needed.
                    //global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                    //unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                    // For each file added, remove it from the data history if exists.
                    //for path in &paths_packedfile {
                        //if table_state_data.borrow().get(path).is_some() {
                            //table_state_data.borrow_mut().remove(path);
                        //}
                        //let data = TableStateData::new_empty();
                        //table_state_data.borrow_mut().insert(path.to_vec(), data);
                    //}
                }

                Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                _ => panic!(THREADS_COMMUNICATION_ERROR),
            }

            // Re-enable the Main Window.
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub fn filter_files(&self) {

        // Set the pattern to search.
        let mut pattern = unsafe { RegExp::new(&self.filter_line_edit.as_mut().unwrap().text()) };

        // Check if the filter should be "Case Sensitive" and if it should "Filter By Folders".
        let filter_by_folder = unsafe { self.filter_filter_by_folder_button.as_mut().unwrap().is_checked() };
        let case_sensitive = unsafe { self.filter_case_sensitive_button.as_mut().unwrap().is_checked() };
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }

        // Filter whatever it's in that column by the text we got.
        unsafe { trigger_treeview_filter(self.packfile_contents_tree_model_filter, &mut pattern, filter_by_folder); }

        // Expand all the matches, if the option for it is enabled.
        if unsafe { self.filter_autoexpand_matches_button.as_ref().unwrap().is_checked() } {
            unsafe { self.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }
        }
    }
}
