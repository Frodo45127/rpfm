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
Module with all the code to deal with configuration folder stuff.

This module contains all the code related with the configuration folder stuff. This means here is the code to properly initialize and return the config folder.
Many things depend on being able to read and write files in that folder, so always remember to initialize it on start, and stop if the initialization failed.
!*/

use directories::ProjectDirs;

use std::fs::{DirBuilder, File};
use std::path::PathBuf;

use rpfm_error::{ErrorKind, Result};

use crate::SETTINGS;

/// Qualifier for the config folder. Only affects MacOS.
const QUALIFIER: &str = "";

/// Organisation for the config folder. Only affects Windows and MacOS.
const ORGANISATION: &str = "";

/// Name of the config folder.
const PROGRAM_NAME: &str = "rpfm";

/// Function to initialize the config folder, so RPFM can use it to store his stuff.
///
/// This can fail, so if this fails, better stop the program and check why it failed.
#[must_use = "Many things depend on this folder existing. So better check this worked."]
pub fn init_config_path() -> Result<()> {

	let config_path = get_config_path()?;
    let autosaves_path = config_path.join("autosaves");
	let error_path = config_path.join("error");
	let schemas_path = config_path.join("schemas");
    let templates_path = config_path.join("templates");
    let templates_custom_path = config_path.join("templates_custom");

    DirBuilder::new().recursive(true).create(&autosaves_path)?;
    DirBuilder::new().recursive(true).create(&config_path)?;
    DirBuilder::new().recursive(true).create(&error_path)?;
    DirBuilder::new().recursive(true).create(&schemas_path)?;
    DirBuilder::new().recursive(true).create(&templates_path)?;
    DirBuilder::new().recursive(true).create(&templates_custom_path)?;

    // Init autosave files if they're not yet initialized. Minimum 1.
    let mut max_autosaves = SETTINGS.read().unwrap().settings_string["autosave_amount"].parse::<i32>().unwrap_or(10);
    if max_autosaves < 1 { max_autosaves = 1; }
    (1..=max_autosaves).for_each(|x| {
        let path = autosaves_path.join(format!("autosave_{:02?}.pack", x));
        if !path.is_file() {
            let _ = File::create(path);
        }
    });

    Ok(())
}

/// This function returns the current config path, or an error if said path is not available.
///
/// Note: On `Debug´ mode this project is the project from where you execute one of RPFM's programs, which should be the root of the repo.
pub fn get_config_path() -> Result<PathBuf> {
	if cfg!(debug_assertions) { std::env::current_dir().map_err(From::from) } else {
        match ProjectDirs::from(&QUALIFIER, &ORGANISATION, &PROGRAM_NAME) {
    		Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
    		None => Err(ErrorKind::IOFolderCannotBeOpened.into())
    	}
    }
}
