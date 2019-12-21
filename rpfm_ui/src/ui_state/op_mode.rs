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
Module with all the code related to the `Operational Mode`.

This module contains the code needed to keep track of the current `Operational Mode`.
!*/

use std::path::PathBuf;

use crate::app_ui::AppUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum represent the current ***Operational Mode*** for RPFM.
#[derive(Debug, Clone)]
pub enum OperationalMode {

    /// MyMod mode enabled. It contains the game folder name (warhammer_2) and the name of the MyMod PackFile.
    MyMod(String, String),

    /// Normal mode enabled.
    Normal,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `OperationalMode`.
impl OperationalMode {

    /// This function sets the current operational mode of the application, depending on the provided MyMod path.
    pub fn set_operational_mode(&mut self, app_ui: &AppUI, mymod_path: Option<&PathBuf>) {
        match mymod_path {

            // If we received a MyMod path, we enable the MyMod mode with that path.
            Some(path) => {

                // Get the `folder_name` and the `mod_name` of our "MyMod".
                let mut path = path.clone();
                let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                path.pop();
                let game_folder_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

                // Set the current mode to `MyMod`.
                *self = OperationalMode::MyMod(game_folder_name, mod_name);

                // Enable all the "MyMod" related actions.
                unsafe { app_ui.mymod_delete_selected.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.mymod_install.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.mymod_uninstall.as_mut().unwrap().set_enabled(true); }
            }

            // If `None` has been provided, we disable the MyMod mode.
            None => {
                *self = OperationalMode::Normal;

                unsafe { app_ui.mymod_delete_selected.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.mymod_install.as_mut().unwrap().set_enabled(false); }
                unsafe { app_ui.mymod_uninstall.as_mut().unwrap().set_enabled(false); }
            }
        }
    }

    /// This function returns a reference to the current `Operational Mode`.
    pub fn get_ref_operational_mode(&self) -> &Self { &self }
}
