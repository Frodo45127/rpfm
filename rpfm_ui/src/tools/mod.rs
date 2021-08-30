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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QWidget;

use qt_core::QBox;

use qt_ui_tools::QUiLoader;

use cpp_core::{CastInto, Ptr};

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Read, BufReader};
use std::rc::Rc;

use rpfm_error::{ErrorKind, Result};
use rpfm_macros::*;

use rpfm_lib::GAME_SELECTED;
use rpfm_lib::packfile::PathType;
use rpfm_lib::packfile::packedfile::PackedFile;
use rpfm_lib::SCHEMA;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::packedfile_views::DataSource;
use crate::pack_tree::{PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;

pub mod faction_painter;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the common content and behavior shared across Tools.
#[derive(GetRef, GetRefMut)]
pub struct Tool {

    /// Main widget of the tool, built from a Template.
    main_widget: QBox<QWidget>,

    /// Paths which the tool requires data from.
    used_paths: Vec<PathType>,

    /// Stored PackedFiles, for quickly pulling data from them if needed.
    packed_files: Rc<RefCell<HashMap<DataSource, BTreeMap<Vec<String>, PackedFile>>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Tool`.
impl Tool {

    /// This function creates a Tool with the data it needs.
    pub unsafe fn new(parent: impl CastInto<Ptr<QWidget>>, paths: &[PathType], tool_supported_games: &[&str], template_path: &str) -> Result<Self> {

        // First, some checks to ensure we can actually open a tool.
        // The requeriments for all tools are:
        // - Game Selected supported by the specific tool we want to open.
        // - Schema for the Game Selected.
        // - Dependencies cache generated and up-to-date.
        //
        // These requeriments are common for all tools, so they're checked here.
        if tool_supported_games.iter().all(|x| *x != &GAME_SELECTED.read().unwrap().get_game_key_name()) {
            return Err(ErrorKind::GameSelectedNotSupportedForTool.into());
        }

        if SCHEMA.read().unwrap().is_none() {
            return Err(ErrorKind::SchemaNotFound.into());
        }

        CENTRAL_COMMAND.send_message_qt(Command::IsThereADependencyDatabase);
        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::Bool(it_is) => if !it_is { return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into()); },
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }

        // Load the UI Template.
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, parent);

        // Dedup the paths.
        let used_paths = PathType::dedup(&paths);

        // Then, build the tool.
        Ok(Self{
            main_widget,
            used_paths,
            packed_files: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    /// This function saves the tools data to the PackFile, in a common way across all tools, and triggers the relevant UI updates.
    pub unsafe fn save(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        // First, check if we actually have an open PackFile.


        // Then, save the PackedFiles to the PackFile and update the view.
        CENTRAL_COMMAND.send_message_qt(Command::SavePackedFilesToPackFileAndClean(self.packed_files.borrow().clone()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        match response {
            Response::VecVecStringVecVecString((paths_to_add, paths_to_delete)) => {

                // Get the list of paths to add, removing those we "replaced".
                let paths_to_add = paths_to_add.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                let paths_to_delete = paths_to_delete.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();

                // Update the TreeView.
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Add(paths_to_add.to_vec()), DataSource::PackFile);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(paths_to_add), DataSource::PackFile);
                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(paths_to_delete), DataSource::PackFile);
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }

            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }

        // Reload the paths edited by the tool whose views are open.
        self.reload_used_paths(app_ui, pack_file_contents_ui);

        // Re-enable the Main Window.
        app_ui.main_window.set_enabled(true);
        Ok(())
    }

    /// This function takes care of backing up the open files we need for the tool, so we always have their latest data.
    ///
    /// Really... we backup everything. To be optimized in the future for backing up only specific PathTypes.
    pub unsafe fn backup_used_paths(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui)
    }

    /// This function takes care of reloading open files we have edited with the tool.
    ///
    /// If a view fails to reload, it just closes it. No view should ever fail, but... we're not in a sunshine and rainbow's world.
    pub unsafe fn reload_used_paths(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let mut paths_to_purge = vec![];
        for path_type in &self.used_paths {
            match path_type {
                PathType::File(ref path) => {
                    if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| *x.get_ref_path() == *path && x.get_data_source() == DataSource::PackFile) {
                        if packed_file_view.reload(path, &pack_file_contents_ui).is_err() {
                            paths_to_purge.push(path.to_vec());
                        }
                    }
                },
                PathType::Folder(ref path) => {
                    for packed_file_view in UI_STATE.set_open_packedfiles().iter_mut().filter(|x| x.get_ref_path().starts_with(path) && x.get_ref_path().len() > path.len() && x.get_data_source() == DataSource::PackFile) {
                        if packed_file_view.reload(path, &pack_file_contents_ui).is_err() {
                            paths_to_purge.push(path.to_vec());
                        }
                    }
                },
                PathType::PackFile => {
                    for packed_file_view in &mut *UI_STATE.set_open_packedfiles() {
                        if packed_file_view.reload(&packed_file_view.get_path(), &pack_file_contents_ui).is_err() {
                            paths_to_purge.push(packed_file_view.get_path().to_vec());
                        }
                    }
                },
                PathType::None => unimplemented!(),
            }
        }

        for path in &paths_to_purge {
            let _ = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, &path, DataSource::PackFile, false);
        }
    }
}
