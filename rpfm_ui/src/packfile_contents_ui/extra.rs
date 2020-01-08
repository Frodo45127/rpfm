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
Module with all the code for utility functions for `PackFileContentsUI`.

This module contains the implementation of custom functions for `PackFileContentsUI`.
The reason they're here and not in the main file is because I don't want to polute
that one, as it's mostly meant for initialization and configuration.
!*/

use qt_widgets::check_box::CheckBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::group_box::GroupBox;
use qt_widgets::label::Label;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::widget::Widget;

use qt_core::connection::Signal;
use qt_core::qt::CaseSensitivity;
use qt_core::reg_exp::RegExp;
use qt_core::slots::SlotNoArgs;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::trigger_treeview_filter;
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::{check_if_path_is_closed, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::utils::{create_grid_layout_unsafe, show_dialog};

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub fn add_packedfiles(&self, app_ui: &AppUI, global_search_ui: &GlobalSearchUI, paths: &[PathBuf], paths_packedfile: &[Vec<String>]) {
        if check_if_path_is_closed(&app_ui, paths_packedfile) {
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

            CENTRAL_COMMAND.send_message_qt(Command::AddPackedFiles((paths.to_vec(), paths_packedfile.to_vec())));
            let response = CENTRAL_COMMAND.recv_message_qt();
            match response {
                Response::Success => {
                    let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                    self.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()));

                    // Update the global search stuff, if needed.
                    global_search_ui.search_on_path(paths.iter().map(From::from).collect());
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
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }

            // Re-enable the Main Window.
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
        }
    }

    /// This function is a helper to add entire folders with subfolders to the UI, keeping the UI updated.
    pub fn add_packed_files_from_folders(&self, app_ui: &AppUI, global_search_ui: &GlobalSearchUI, paths: &[PathBuf], paths_packedfile: &[Vec<String>]) {
        if check_if_path_is_closed(&app_ui, paths_packedfile) {
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
            let paths_to_send = paths.iter().cloned().zip(paths_packedfile.iter().cloned()).collect();
            CENTRAL_COMMAND.send_message_qt(Command::AddPackedFilesFromFolder(paths_to_send));
            let response = CENTRAL_COMMAND.recv_message_qt();
            match response {
                Response::VecPathType(paths_packedfile) => {
                    let paths = paths_packedfile.iter().map(From::from).collect::<Vec<TreePathType>>();
                    self.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()));

                    // Update the global search stuff, if needed.
                    global_search_ui.search_on_path(paths.iter().map(From::from).collect());
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
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }

            // Re-enable the Main Window.
            unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub fn filter_files(&self) {

        // Set the pattern to search.
        let mut pattern = unsafe { RegExp::new(&self.filter_line_edit.as_mut().unwrap().text()) };

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = unsafe { self.filter_case_sensitive_button.as_mut().unwrap().is_checked() };
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }

        // Filter whatever it's in that column by the text we got.
        unsafe { trigger_treeview_filter(self.packfile_contents_tree_model_filter, &mut pattern); }

        // Expand all the matches, if the option for it is enabled.
        if unsafe { self.filter_autoexpand_matches_button.as_ref().unwrap().is_checked() } {
            unsafe { self.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }
        }
    }

    /// This function creates the entire "Rename" dialog.
    ///
    ///It returns the new name of the Item, or `None` if the dialog is canceled or closed.
    pub fn create_rename_dialog(app_ui: &AppUI, selected_items: &[TreePathType]) -> Option<String> {

        // Create and configure the dialog.
        let mut dialog = unsafe { Dialog::new_unsafe(app_ui.main_window as *mut Widget) };
        dialog.set_window_title(&QString::from_std_str("Rename Selection"));
        dialog.set_modal(true);
        dialog.resize((400, 50));
        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);

        // Create a little frame with some instructions.
        let instructions_frame = GroupBox::new(&QString::from_std_str("Instructions"));
        let instructions_grid = create_grid_layout_unsafe(instructions_frame.as_mut_ptr() as *mut Widget);
        let instructions_label = Label::new(&QString::from_std_str(
        "\
    It's easy, but you'll not understand it without an example, so here it's one:
     - Your files/folders says 'you' and 'I'.
     - Write 'whatever {x} want' in the box below.
     - Hit 'Accept'.
     - RPFM will turn that into 'whatever you want' and 'whatever I want' and call your files/folders that.
    And, in case you ask, works with numeric cells too, as long as the resulting text is a valid number.
        "
        ));
        unsafe { instructions_grid.as_mut().unwrap().add_widget((instructions_label.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }

        let mut rewrite_sequence_line_edit = LineEdit::new(());
        rewrite_sequence_line_edit.set_placeholder_text(&QString::from_std_str("Write here whatever you want. {x} it's your current name."));

        // If we only have one selected item, put his name by default in the rename dialog.
        if selected_items.len() == 1 {
            if let TreePathType::File(path) | TreePathType::Folder(path) = &selected_items[0] {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(path.last().unwrap()));
            }
        }
        let accept_button = PushButton::new(&QString::from_std_str("Accept"));

        unsafe { main_grid.as_mut().unwrap().add_widget((instructions_frame.into_raw() as *mut Widget, 0, 0, 1, 2)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((rewrite_sequence_line_edit.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((accept_button.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }

        accept_button.signals().released().connect(&dialog.slots().accept());

        if dialog.exec() == 1 {
            let new_text = rewrite_sequence_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(rewrite_sequence_line_edit.text().to_std_string()) }
        } else { None }
    }

    /// This function creates the "Mass-Import TSV" dialog. Nothing too massive.
    ///
    /// It returns the name of the new imported PackedFiles & their Paths, or None in case of closing the dialog.
    pub fn create_mass_import_tsv_dialog(app_ui: &AppUI) -> Option<(Vec<PathBuf>, Option<String>)> {

        // Create the "Mass-Import TSV" Dialog and configure it.
        let mut dialog = unsafe { Dialog::new_unsafe(app_ui.main_window as *mut Widget) };
        dialog.set_window_title(&QString::from_std_str("Mass-Import TSV Files"));
        dialog.set_modal(true);
        dialog.resize((400, 100));

        // Create the main Grid and his stuff.
        let main_grid = create_grid_layout_unsafe(dialog.as_mut_ptr() as *mut Widget);
        let mut files_to_import_label = Label::new(&QString::from_std_str("Files to import: 0."));
        let select_files_button = PushButton::new(&QString::from_std_str("..."));
        let mut imported_files_name_line_edit = LineEdit::new(());
        let use_original_filenames_label = Label::new(&QString::from_std_str("Use original filename:"));
        let use_original_filenames_checkbox = CheckBox::new(());
        let import_button = PushButton::new(&QString::from_std_str("Import"));

        // Set a dummy name as default.
        imported_files_name_line_edit.set_text(&QString::from_std_str("new_imported_file"));

        // Add all the widgets to the main grid, and the main grid to the dialog.
        unsafe { main_grid.as_mut().unwrap().add_widget((files_to_import_label.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((select_files_button.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((use_original_filenames_label.as_mut_ptr() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((use_original_filenames_checkbox.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((imported_files_name_line_edit.as_mut_ptr() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((import_button.as_mut_ptr() as *mut Widget, 2, 1, 1, 1)); }

        //-------------------------------------------------------------------------------------------//
        // Actions for the Mass-Import TSV Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the list of Paths to import.
        let files_to_import = Rc::new(RefCell::new(vec![]));
        let dialog = dialog.into_raw();

        // What happens when we hit the "..." button.
        let slot_select_files = SlotNoArgs::new(clone!(
            files_to_import => move || {

                // Create the FileDialog to get the TSV files, and add them to the list if we accept.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    dialog as *mut Widget,
                    &QString::from_std_str("Select TSV Files to Import..."),
                )) };

                file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
                file_dialog.set_file_mode(FileMode::ExistingFiles);

                if file_dialog.exec() == 1 {
                    let selected_files = file_dialog.selected_files();
                    files_to_import.borrow_mut().clear();
                    for index in 0..selected_files.count(()) {
                        files_to_import.borrow_mut().push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                    }

                    files_to_import_label.set_text(&QString::from_std_str(&format!("Files to import: {}.", selected_files.count(()))));
                }
            }
        ));

        select_files_button.signals().released().connect(&slot_select_files);
        unsafe { import_button.signals().released().connect(&dialog.as_ref().unwrap().slots().accept()); }

        // If we hit the "Create" button, check if we want to use their native name and send the info back.
        if unsafe { dialog.as_mut().unwrap().exec() } == 1 {
            if use_original_filenames_checkbox.is_checked() {
                Some((files_to_import.borrow().to_vec(), None))
            }
            else {
                let packed_file_name = imported_files_name_line_edit.text().to_std_string();
                Some((files_to_import.borrow().to_vec(), Some(packed_file_name)))
            }
        }

        // In any other case, we return None.
        else { None }
    }
}
