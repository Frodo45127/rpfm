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

use qt_widgets::QTreeView;
use qt_widgets::QCheckBox;
use qt_widgets::QDialog;
use qt_widgets::{QFileDialog, q_file_dialog::FileMode};
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QRegExp;
use qt_core::QString;
use qt_core::SlotNoArgs;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_error::ErrorKind;

use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::trigger_treeview_filter_safe;
use crate::locale::{qtr, qtre};
use crate::packedfile_views::DataSource;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::{create_grid_layout, show_dialog};
use crate::ui_state::OperationalMode;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsUI`.
impl PackFileContentsUI {

    /// This function is a helper to add PackedFiles to the UI, keeping the UI updated.
    pub unsafe fn add_packedfiles(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths: &[PathBuf],
        paths_packedfile: &[Vec<String>],
        paths_to_ignore: Option<Vec<PathBuf>>
    ) {
        app_ui.main_window.set_enabled(false);

        CENTRAL_COMMAND.send_message_qt(Command::AddPackedFiles((paths.to_vec(), paths_packedfile.to_vec(), paths_to_ignore)));
        let response1 = CENTRAL_COMMAND.recv_message_qt();
        let response2 = CENTRAL_COMMAND.recv_message_qt();
        match response1 {
            Response::VecPathType(paths) => {
                let paths = paths.iter().map(From::from).collect::<Vec<TreePathType>>();
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths.to_vec()), DataSource::PackFile);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = paths_packedfile.iter().filter_map(|path| {
                    if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                        if packed_file_view.reload(path, pack_file_contents_ui).is_err() {
                            Some(path.to_vec())
                        } else { None }
                    } else { None }
                }).collect::<Vec<Vec<String>>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response1),
        }

        match response2 {
            Response::Success => {},
            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response2),
        }

        // Re-enable the Main Window.
        app_ui.main_window.set_enabled(true);
    }

    /// This function is a helper to add entire folders with subfolders to the UI, keeping the UI updated.
    pub unsafe fn add_packed_files_from_folders(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths: &[PathBuf],
        paths_packedfile: &[Vec<String>],
        paths_to_ignore: Option<Vec<PathBuf>>
    ) {
        app_ui.main_window.set_enabled(false);
        let paths_to_send = paths.iter().cloned().zip(paths_packedfile.iter().cloned()).collect();
        CENTRAL_COMMAND.send_message_qt(Command::AddPackedFilesFromFolder(paths_to_send, paths_to_ignore));
        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::VecPathType(paths_packedfile) => {
                let paths = paths_packedfile.iter().map(From::from).collect::<Vec<TreePathType>>();
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths.to_vec()), DataSource::PackFile);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths.to_vec()), DataSource::PackFile);
                UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);

                // Try to reload all open files which data we altered, and close those that failed.
                let failed_paths = paths_packedfile.iter().filter_map(|path| {
                    if let PathType::File(path) = path {
                        if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                            if packed_file_view.reload(path, pack_file_contents_ui).is_err() {
                                Some(path.to_vec())
                            } else { None }
                        } else { None }
                    } else { None }
                }).collect::<Vec<Vec<String>>>();

                for path in &failed_paths {
                    let _ = AppUI::purge_that_one_specifically(app_ui, pack_file_contents_ui, path, DataSource::PackFile, false);
                }
            }

            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Re-enable the Main Window.
        app_ui.main_window.set_enabled(true);
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_files(pack_file_contents_ui: &Rc<Self>) {

        // Set the pattern to search.
        let pattern = QRegExp::new_1a(&pack_file_contents_ui.filter_line_edit.text());

        // Check if the filter should be "Case Sensitive".
        let case_sensitive = pack_file_contents_ui.filter_case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        // Filter whatever it's in that column by the text we got.
        trigger_treeview_filter_safe(&pack_file_contents_ui.packfile_contents_tree_model_filter, &pattern.as_ptr());

        // Expand all the matches, if the option for it is enabled.
        if pack_file_contents_ui.filter_autoexpand_matches_button.is_checked() {
            pack_file_contents_ui.packfile_contents_tree_view.expand_all();
        }
    }

    /// This function creates the entire "Rename" dialog.
    ///
    ///It returns the new name of the Item, or `None` if the dialog is canceled or closed.
    pub unsafe fn create_rename_dialog(app_ui: &Rc<AppUI>, selected_items: &[TreePathType]) -> Option<String> {

        // Create and configure the dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("rename_selection"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 50);
        let main_grid = create_grid_layout(dialog.static_upcast());

        // Create a little frame with some instructions.
        let instructions_frame = QGroupBox::from_q_string(&qtr("rename_selection_instructions"));
        let instructions_grid = create_grid_layout(instructions_frame.static_upcast());
        let instructions_label = QLabel::from_q_string(&qtr("rename_instructions"));
        instructions_grid.add_widget_5a(&instructions_label, 0, 0, 1, 1);

        let rewrite_sequence_line_edit = QLineEdit::new();
        rewrite_sequence_line_edit.set_placeholder_text(&qtr("rename_selection_placeholder"));

        // If we only have one selected item, put his name by default in the rename dialog.
        if selected_items.len() == 1 {
            if let TreePathType::File(path) | TreePathType::Folder(path) = &selected_items[0] {
                rewrite_sequence_line_edit.set_text(&QString::from_std_str(path.last().unwrap()));
            }
        }
        let accept_button = QPushButton::from_q_string(&qtr("gen_loc_accept"));

        main_grid.add_widget_5a(&instructions_frame, 0, 0, 1, 2);
        main_grid.add_widget_5a(&rewrite_sequence_line_edit, 1, 0, 1, 1);
        main_grid.add_widget_5a(&accept_button, 1, 1, 1, 1);

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
        let dialog = Rc::new(QDialog::new_1a(&app_ui.main_window)    );
        dialog.set_window_title(&qtr("mass_import_tsv"));
        dialog.set_modal(true);
        dialog.resize_2a(400, 100);

        // Create the main Grid and his stuff.
        let main_grid = create_grid_layout(dialog.static_upcast());
        let files_to_import_label = QLabel::from_q_string(&qtr("mass_import_num_to_import"));
        let select_files_button = QPushButton::from_q_string(&QString::from_std_str("..."));
        let imported_files_name_line_edit = QLineEdit::new();
        let use_original_filenames_label = QLabel::from_q_string(&qtr("mass_import_use_original_filename"));
        let use_original_filenames_checkbox = QCheckBox::new();
        let import_button = QPushButton::from_q_string(&qtr("mass_import_import"));

        // Set a dummy name as default.
        imported_files_name_line_edit.set_text(&qtr("mass_import_default_name"));

        // Add all the widgets to the main grid, and the main grid to the dialog.
        main_grid.add_widget_5a(& files_to_import_label, 0, 0, 1, 1);
        main_grid.add_widget_5a(& select_files_button, 0, 1, 1, 1);
        main_grid.add_widget_5a(& use_original_filenames_label, 1, 0, 1, 1);
        main_grid.add_widget_5a(& use_original_filenames_checkbox, 1, 1, 1, 1);
        main_grid.add_widget_5a(& imported_files_name_line_edit, 2, 0, 1, 1);
        main_grid.add_widget_5a(& import_button, 2, 1, 1, 1);

        //-------------------------------------------------------------------------------------------//
        // Actions for the Mass-Import TSV Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the list of Paths to import.
        let files_to_import = Rc::new(RefCell::new(vec![]));

        // What happens when we hit the "..." button.
        let slot_select_files = SlotNoArgs::new(&*dialog, clone!(
            dialog,
            files_to_import => move || {

                // Create the FileDialog to get the TSV files, and add them to the list if we accept.
                let file_dialog = QFileDialog::from_q_widget_q_string(
                    &*dialog,
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

    pub unsafe fn extract_packed_files(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<Self>,
        paths_to_extract: Option<Vec<PathType>>
    ) {

        // Get the currently selected paths (and visible) paths, or the ones received from the function.
        let items_to_extract = match paths_to_extract {
            Some(paths) => paths,
            None => {
                let selected_items = <QBox<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
                selected_items.iter().map(From::from).collect::<Vec<PathType>>()
            }
        };

        let extraction_path = match UI_STATE.get_operational_mode() {

            // In MyMod mode we extract directly to the folder of the selected MyMod, keeping the folder structure.
            OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                if let Some(ref mymods_base_path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {

                    // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                    let mut mod_name = mod_name.to_owned();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();
                    mod_name.pop();

                    let mut assets_folder = mymods_base_path.to_path_buf();
                    assets_folder.push(&game_folder_name);
                    assets_folder.push(&mod_name);
                    assets_folder
                }

                // If there is no MyMod path configured, report it.
                else { return show_dialog(&app_ui.main_window, ErrorKind::MyModPathNotConfigured, true); }
            }

            // In normal mode, we ask the user to provide us with a path.
            OperationalMode::Normal => {
                let extraction_path = QFileDialog::get_existing_directory_2a(
                    &app_ui.main_window,
                    &qtr("context_menu_extract_packfile"),
                );

                if !extraction_path.is_empty() { PathBuf::from(extraction_path.to_std_string()) }
                else { return }
            }
        };

        // We have to save our data from cache to the backend before extracting it. Otherwise we would extract outdated data.
        // TODO: Make this more... optimal.
        if let Err(error) = UI_STATE.get_open_packedfiles()
            .iter()
            .filter(|x| x.get_data_source() == DataSource::PackFile)
            .try_for_each(|packed_file| packed_file.save(app_ui, pack_file_contents_ui)) {
            show_dialog(&app_ui.main_window, error, false);
        }

        else {
            CENTRAL_COMMAND.send_message_qt(Command::ExtractPackedFiles(items_to_extract, extraction_path));
            app_ui.main_window.set_enabled(false);
            let response = CENTRAL_COMMAND.recv_message_qt_try();
            match response {
                Response::String(result) => show_dialog(&app_ui.main_window, result, true),
                Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
            }
            app_ui.main_window.set_enabled(true);
        }
    }

    pub unsafe fn start_delayed_updates_timer(pack_file_contents_ui: &Rc<Self>,) {
        pack_file_contents_ui.filter_timer_delayed_updates.set_interval(500);
        pack_file_contents_ui.filter_timer_delayed_updates.start_0a();
    }
}
