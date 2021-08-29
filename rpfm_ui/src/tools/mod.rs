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

use std::rc::Rc;

use rpfm_error::Result;

use rpfm_lib::packfile::PathType;

use crate::AppUI;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::UI_STATE;

pub mod faction_painter;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the common content and behavior shared across Tools.
pub struct Tool {
    used_paths: Vec<PathType>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Tool`.
impl Tool {

    /// This function creates a Tool with the data it needs.
    pub fn new(paths: &[PathType]) -> Self {
        let used_paths = PathType::dedup(&paths);
        Self{
            used_paths,
        }
    }

    /// This function takes care of backing up the open files we need for the tool, so we always have their latest data.
    pub unsafe fn backup_used_paths(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {
        AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui)
    }

    /// This function takes care of reloading open files we have edited with the tool.
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
