//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with utilities to deal with the filesystem restrictions of the Flatpak sandbox.
//!
//! Flatpak's `host` filesystem permission doesn't include the `~/.var/app` folders of other Flatpak
//! apps, yet the file chooser portal happily returns paths inside them: the portal only exports a
//! selection through the document portal when it thinks the app lacks access to it, and having
//! `filesystem=host` is enough for it to assume we already have access, even for paths that
//! permission explicitly excludes.
//!
//! The result is that we get a path we cannot open, and the only way to fix it is for the user to
//! grant us access to it. The functions here detect that situation and build the command that does it.

use std::fs::{File, metadata, read_dir, read_to_string};
use std::path::{Path, PathBuf};

/// File that only exists within a Flatpak sandbox.
const FLATPAK_INFO_PATH: &str = "/.flatpak-info";

/// Env var containing the app id of the Flatpak app we're running as.
const FLATPAK_ID_VAR: &str = "FLATPAK_ID";

/// Folders that, in this order, contain the data folder of each installed Flatpak app.
const FLATPAK_APPS_PATH: [&str; 2] = [".var", "app"];

/// Group of the Flatpak info file containing the data of the app we're running as.
const FLATPAK_INFO_APP_GROUP: &str = "[Application]";

/// Key, within the app group of the Flatpak info file, containing our app id.
const FLATPAK_INFO_NAME_KEY: &str = "name=";

//-------------------------------------------------------------------------------//
//                             Public functions
//-------------------------------------------------------------------------------//

/// This function checks if we're running within a Flatpak sandbox.
///
/// # Returns
///
/// True if we're running within a Flatpak sandbox, false otherwise.
pub fn is_sandboxed() -> bool {
    Path::new(FLATPAK_INFO_PATH).is_file()
}

/// This function returns the app id of the Flatpak app we're running as.
///
/// # Returns
///
/// The app id, or `None` if we're not within a Flatpak sandbox, or if it cannot be retrieved.
pub fn app_id() -> Option<String> {
    if let Ok(app_id) = std::env::var(FLATPAK_ID_VAR) {
        if !app_id.is_empty() {
            return Some(app_id);
        }
    }

    app_id_from_info(&read_to_string(FLATPAK_INFO_PATH).ok()?)
}

/// This function checks if the provided path can be read from within the current sandbox.
///
/// # Arguments
///
/// * `path` - Path to check.
///
/// # Returns
///
/// True if the path exists and we can read it, false otherwise.
pub fn is_path_readable(path: &Path) -> bool {
    match metadata(path) {
        Ok(metadata) => if metadata.is_dir() {
            read_dir(path).is_ok()
        } else {
            File::open(path).is_ok()
        },
        Err(_) => false,
    }
}

/// This function returns the data folder of the Flatpak app the provided path belongs to.
///
/// # Arguments
///
/// * `path` - Path to check.
///
/// # Returns
///
/// The data folder of the app owning the path, or `None` if the path doesn't belong to
/// another Flatpak app's data folder.
pub fn owner_app_data_path(path: &Path) -> Option<PathBuf> {

    // The apps folder is matched anywhere in the path instead of against the home dir, because in
    // some distros the home dir the portal returns is not the one we get from the environment.
    let components = path.components().collect::<Vec<_>>();
    let apps_position = components.windows(2).position(|window|
        window[0].as_os_str() == FLATPAK_APPS_PATH[0] && window[1].as_os_str() == FLATPAK_APPS_PATH[1]
    )?;

    let owner_app_id = components.get(apps_position + 2)?.as_os_str();
    if app_id().is_some_and(|app_id| *owner_app_id == *app_id) {
        return None;
    }

    Some(components[..=apps_position + 2].iter().collect())
}

/// This function returns the command the user has to run to grant us access to the provided path.
///
/// For paths owned by another Flatpak app, the command grants access to that app's full data folder,
/// because games installed there keep their downloaded mods outside their own folder.
///
/// # Arguments
///
/// * `path` - Path we need access to.
///
/// # Returns
///
/// The command to run, or `None` if we're not sandboxed, if we can already read the path, or if
/// our own app id cannot be retrieved.
pub fn missing_permission_command(path: &Path) -> Option<String> {
    if !is_sandboxed() || is_path_readable(path) {
        return None;
    }

    let app_id = app_id()?;
    let path = owner_app_data_path(path).unwrap_or_else(|| path.to_path_buf());

    Some(format!("flatpak override --user --filesystem=\"{}\" {app_id}", path.to_string_lossy()))
}

//-------------------------------------------------------------------------------//
//                             Private functions
//-------------------------------------------------------------------------------//

/// This function parses the app id out of the contents of a Flatpak info file.
pub(crate) fn app_id_from_info(info: &str) -> Option<String> {
    info.lines()
        .skip_while(|line| line.trim() != FLATPAK_INFO_APP_GROUP)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| line.trim().strip_prefix(FLATPAK_INFO_NAME_KEY))
        .map(|app_id| app_id.trim().to_owned())
}
