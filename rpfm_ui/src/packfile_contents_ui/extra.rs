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

use qt_widgets::QCheckBox;
use qt_widgets::QDialog;
use qt_widgets::{QFileDialog, q_file_dialog::FileMode};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;

use qt_core::CaseSensitivity;
use qt_core::QRegExp;
use qt_core::QString;
use qt_core::Slot;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::packfile::PathType;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::trigger_treeview_filter_safe;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, qtre};
use crate::pack_tree::{check_if_path_is_closed, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::{create_grid_layout, show_dialog};

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub unsafe fn add_packedfiles(
        &mut self,
        app_ui: &mut AppUI,
        global_search_ui: &mut GlobalSearchUI,
        paths: &[PathBuf],
        paths_packedfile: &[Vec<String>]
    ) {
        if check_if_path_is_closed(&app_ui, paths_packedfile) {
            app_ui.main_window.set_enabled(false);


            CENTRAL_COMMAND.send_message_qt(Command::AddPackedFiles((paths.to_vec(), paths_packedfile.to_vec())));
            let response = CENTRAL_COMMAND.recv_message_qt();
            match response {
                Response::Success => {

                    // Clear the preview cache before adding stuff!!!
                    paths_packedfile.iter().for_each(|path| {
                        app_ui.purge_that_one_specifically(*global_search_ui, *self, path, false);
                    });

                    let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                    self.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()));

                    // Update the global search stuff, if needed.
                    global_search_ui.search_on_path(self, paths.iter().map(From::from).collect());
                }

                Response::Error(error) => show_dialog(app_ui.main_window, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }

            // Re-enable the Main Window.
            app_ui.main_window.set_enabled(true);
        }
    }

    /// This function is a helper to add entire folders with subfolders to the UI, keeping the UI updated.
    pub unsafe fn add_packed_files_from_folders(&mut self, app_ui: &mut AppUI, global_search_ui: &mut GlobalSearchUI, paths: &[PathBuf], paths_packedfile: &[Vec<String>]) {
        if check_if_path_is_closed(&app_ui, paths_packedfile) {
            app_ui.main_window.set_enabled(false);
            let paths_to_send = paths.iter().cloned().zip(paths_packedfile.iter().cloned()).collect();
            CENTRAL_COMMAND.send_message_qt(Command::AddPackedFilesFromFolder(paths_to_send));
            let response = CENTRAL_COMMAND.recv_message_qt();
            match response {
                Response::VecPathType(paths_packedfile) => {

                    // Clear the preview cache before adding stuff!!!
                    paths_packedfile.iter().for_each(|path| {
                        if let PathType::File(path) = path {
                            app_ui.purge_that_one_specifically(*global_search_ui, *self, path, false);
                        }
                    });

                    let paths = paths_packedfile.iter().map(From::from).collect::<Vec<TreePathType>>();
                    self.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()));

                    // Update the global search stuff, if needed.
                    global_search_ui.search_on_path(self, paths.iter().map(From::from).collect());
                }

                Response::Error(error) => show_dialog(app_ui.main_window, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }

            // Re-enable the Main Window.
            app_ui.main_window.set_enabled(true);
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_files(&mut self) {

        // Set the pattern to search.
        let mut pattern = QRegExp::new_1a(&self.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = self.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&mut self.packfile_contents_tree_model_filter, &mut pattern);

        // Expand all the matches, if the option for it is enabled.
        if self.filter_autoexpand_matches_button.is_checked() {
            self.packfile_contents_tree_view.expand_all();
        }
    }

    /// This function creates the entire "Rename" dialog.
    ///
    ///It returns the new name of the Item, or `None` if the dialog is canceled or closed.
    pub unsafe fn create_rename_dialog(app_ui: &mut AppUI, selected_items: &[TreePathType]) -> Option<String> {

        // Create and configure the dialog.
        let mut dialog = QDialog::new_1a(app_ui.main_window).into_ptr();
        dialog.set_window_title(&qtr("rename_selection"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string(&qtr("rename_selection_instructions")).into_ptr();
        let mut instructions_grid = create_grid_layout(instructions_frame.static_upcast_mut());
        let instructions_label = QLabel::from_q_string(&QString::from_std_str(
            "\
    It's easy, but you'll not understand it without an example, so here it's one:
     - Your files/folders says 'you' and 'I'.
     - Write 'whatever {x} want' in the box below.
     - Hit 'Accept'.
     - RPFM will turn that into 'whatever you want' and 'whatever I want' and call your files/folders that.
    And, in case you ask, works with numeric cells too, as long as the resulting text is a valid number."));
        instructions_grid.add_widget_5a(instructions_label.into_ptr(), 0, 0, 1, 1);

        let mut rewrite_sequence_line_edit = QLineEdit::new();
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rename_selection_placeholder"));

        // If we only have one selected item, put his name by default in the rename dialog.
        if selected_items.len() == 1 {
            if let TreePathType::File(path) | TreePathType::Folder(path) = &selected_items[0] {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(path.last().unwrap()));
            }
        }
        let mut accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        main_grid.add_widget_5a(instructions_frame, 0, 0, 1, 2);
        main_grid.add_widget_5a(&mut rewrite_sequence_line_edit, 1, 0, 1, 1);
        main_grid.add_widget_5a(&mut accept_button, 1, 1, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        if dialog.exec() == 1 {
            let new_text = rewrite_sequence_line_edit.text().to_std_string();
            if new_text.is_empty() { None } else { Some(rewrite_sequence_line_edit.text().to_std_string()) }
        } else { None }
    }

    /// This function creates the "Mass-Import TSV" dialog. Nothing too massive.
    ///
    /// It returns the name of the new imported PackedFiles & their Paths, or None in case of closing the dialog.
    pub unsafe fn create_mass_import_tsv_dialog(app_ui: &AppUI) -> Option<(Vec<PathBuf>, Option<String>)> {

        // Create the "Mass-Import TSV" Dialog and configure it.
        let mut dialog = QDialog::new_1a(app_ui.main_window).into_ptr();
        dialog.set_window_title(&qtr("mass_import_tsv"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 100);

        // Create the main Grid and his stuff.
        let mut main_grid = create_grid_layout(dialog.static_upcast_mut());
        let mut files_to_import_label = QLabel::from_q_string(&qtr("mass_import_num_to_import"));
        let mut select_files_button = QPushButton::from_q_string(&QString::from_std_str("..."));
        let mut imported_files_name_line_edit = QLineEdit::new();
        let mut use_original_filenames_label = QLabel::from_q_string(&qtr("mass_import_use_original_filename"));
        let mut use_original_filenames_checkbox = QCheckBox::new();
        let mut import_button = QPushButton::from_q_string(&qtr("mass_import_import"));

        // Set a dummy name as default.
        imported_files_name_line_edit.set_text(&qtr("mass_import_default_name"));

        // Add all the widgets to the main grid, and the main grid to the dialog.
        main_grid.add_widget_5a(&mut files_to_import_label, 0, 0, 1, 1);
        main_grid.add_widget_5a(&mut select_files_button, 0, 1, 1, 1);
        main_grid.add_widget_5a(&mut use_original_filenames_label, 1, 0, 1, 1);
        main_grid.add_widget_5a(&mut use_original_filenames_checkbox, 1, 1, 1, 1);
        main_grid.add_widget_5a(&mut imported_files_name_line_edit, 2, 0, 1, 1);
        main_grid.add_widget_5a(&mut import_button, 2, 1, 1, 1);

        //-------------------------------------------------------------------------------------------//
        // Actions for the Mass-Import TSV Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the list of Paths to import.
        let files_to_import = Rc::new(RefCell::new(vec![]));

        // What happens when we hit the "..." button.
        let slot_select_files = Slot::new(clone!(
            files_to_import => move || {

                // Create the FileDialog to get the TSV files, and add them to the list if we accept.
                let mut file_dialog = QFileDialog::from_q_widget_q_string(
                    dialog,
                    &qtr("mass_import_select"),
                );

                file_dialog.set_name_filter(&QString::from_std_str("TSV Files (*.tsv)"));
                file_dialog.set_file_mode(FileMode::ExistingFiles);

                if file_dialog.exec() == 1 {
                    let selected_files = file_dialog.selected_files();
                    files_to_import.borrow_mut().clear();
                    for index in 0..selected_files.count_0a() {
                        files_to_import.borrow_mut().push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                    }

                    files_to_import_label.set_text(&qtre("files_to_import", &[&selected_files.count_0a().to_string()]));
                }
            }
        ));

        select_files_button.released().connect(&slot_select_files);
        import_button.released().connect(dialog.slot_accept());

        // If we hit the "Create" button, check if we want to use their native name and send the info back.
        if dialog.exec() == 1 {
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
