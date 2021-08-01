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
Module that contains the GameInfo defintion and stuff related with it.

!*/

use std::collections::HashMap;
use std::fs::DirBuilder;
use std::path::PathBuf;

use rpfm_error::{Result, Error, ErrorKind};
use rpfm_macros::*;

use crate::common::get_files_from_subdir;
use crate::config::get_config_path;
use crate::packfile::{Manifest, PFHFileType, PFHVersion};
use crate::SETTINGS;

pub mod supported_games;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the info needed for a game to be "supported" by RPFM.
#[derive(Clone, Debug)]
pub struct GameInfo {

    /// This is the name it'll show up for the user. The *pretty name*. For example, in a dropdown (Warhammer 2).
    display_name: &'static str,

    /// This is the PFHVersion used at the start of every PackFile for that game.
    /// It's in a hashmap of PFHFileType => PFHVersion, so we can have different PackFile versions depending on their type.
    pfh_versions: HashMap<PFHFileType, PFHVersion>,

    /// This is the full name of the schema file used for the game. For example: `schema_wh2.ron`.
    schema_file_name: String,

    /// This is the name of the file containing the dependencies cache for this game.
    depenencies_cache_file_name: String,

    /// This is the **type** of raw files the game uses. -1 is "Don't have Assembly Kit". 0 is Empire/Nappy. 1 is Shogun 2. 2 is anything newer than Shogun 2.
    raw_db_version: i16,

    /// If we can save `PackFile` files for the game.
    supports_editing: bool,

    /// If the db tables should have a GUID in their headers.
    db_tables_have_guid: bool,

    /// Name of the icon used to display the game as `Game Selected`, in an UI.
    game_selected_icon: String,

    /// Name of the big icon used to display the game as `Game Selected`, in an UI.
    game_selected_big_icon: String,

    /// Logic used to name vanilla tables.
    vanilla_db_table_name_logic: VanillaDBTableNameLogic,

    /// Installation-dependant data.
    install_data: HashMap<InstallType, InstallData>
}

/// This enum holds the info about each game approach at naming db tables.
#[derive(Clone, Debug)]
pub enum VanillaDBTableNameLogic {

    /// This variant is for games where the table name is their folder's name.
    FolderName,

    /// This variant is for games where all tables are called the same.
    DefaultName(String),
}

/// This enum represents the different installations of games the game support.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum InstallType {

    /// Windows - Steam variant.
    WinSteam,

    /// Linux - Steam variant.
    LnxSteam,

    /// Windows - Epic Store Variant.
    WinEpic,

    /// Windows - Wargaming Variant.
    WinWargaming,
}

/// This struct contains installation-dependant data about each game.
///
/// NOTE: All PackFile paths contained in this struct are RELATIVE, either to the data folder, or to the game's folder.
#[derive(GetRef, Clone, Debug)]
struct InstallData {

    /// These are the PackFiles that contain db tables. Relative to data_path.
    db_packs: Vec<String>,

    /// These are the PackFiles that contain localization stuff. Relative to data_path.
    loc_packs: Vec<String>,

    /// List of vanilla packs, to be use as reference for knowning what PackFiles are vanilla in games without a manifest file.
    /// Currently only used for Empire and Napoleon. Relative to data_path.
    vanilla_packs: Vec<String>,

    /// If the manifest of the game should be used to get the vanilla PackFile list, or should we use the hardcoded list.
    use_manifest: bool,

    /// StoreID of the game.
    store_id: i64,

    /// Name of the executable of the game, including extension if it has it.
    executable: String,

    /// /data path of the game, or equivalent. Relative to the game's path.
    data_path: String,

    /// Folder where local (your own) mods are stored. Relative to the game's path.
    local_mods_path: String,

    /// Folder where downloaded (other peoples's) mods are stored. Relative to the game's path.
    downloaded_mods_path: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of GameInfo.
impl GameInfo {

    /// This function returns the "Key" name of the Game, meaning in lowercase and without spaces.
    pub fn get_game_key_name(&self) -> String {
        self.display_name.to_lowercase().replace(' ', "_")
    }

    /// This function returns the "Display" name of the Game, meaning properly written.
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    /// This function returns the PFHVersion corresponding to the provided PackFile type. If it's not found, it defaults to the one used by mods.
    pub fn get_pfh_version_by_file_type(&self, pfh_file_type: PFHFileType) -> PFHVersion {
        match self.pfh_versions.get(&pfh_file_type) {
            Some(pfh_version) => *pfh_version,
            None => *self.pfh_versions.get(&PFHFileType::Mod).unwrap(),
        }
    }

    /// This function returns the full list of compatible PFHVersions for this game.
    pub fn get_pfh_versions(&self) -> &HashMap<PFHFileType, PFHVersion> {
        &self.pfh_versions
    }

    /// This function returns this Game's schema file name.
    pub fn get_schema_name(&self) -> &str {
        &self.schema_file_name
    }

    /// This function returns this Game's dependencies cache file name.
    pub fn get_dependencies_cache_file_name(&self) -> &str {
        &self.depenencies_cache_file_name
    }

    /// This function returns this Game's raw_db_version, used to identify how to process AssKit table files for this game.
    pub fn get_raw_db_version(&self) -> i16 {
        self.raw_db_version
    }

    /// This function returns whether this Game supports editing or not.
    pub fn get_supports_editing(&self) -> bool {
        self.supports_editing
    }

    /// This function returns whether this Game's tables should have a GUID in their header or not.
    pub fn get_db_tables_have_guid(&self) -> bool {
        self.db_tables_have_guid
    }

    /// This function returns this Game's icon filename. Normal size.
    pub fn get_game_selected_icon_file_name(&self) -> &str {
        &self.game_selected_icon
    }

    /// This function returns this Game's icon filename. Big size.
    pub fn get_game_selected_icon_big_file_name(&self) -> &str {
        &self.game_selected_big_icon
    }

    /// This function returns this Game's logic for naming db tables.
    pub fn get_vanilla_db_table_name_logic(&self) -> VanillaDBTableNameLogic {
        self.vanilla_db_table_name_logic.clone()
    }

    /// This function gets the `/rpfm_path/pak_files/xxx.pak` path of the Game Selected, if it has one.
    pub fn get_dependencies_cache_file(&self) -> Result<PathBuf> {
        let mut base_path = get_config_path()?;
        base_path.push("pak_files");
        base_path.push(self.get_dependencies_cache_file_name());

        if base_path.is_file() { Ok(base_path) }
        else { Err(ErrorKind::IOFileNotFound.into()) }
    }

    /// This function tries to get the correct InstallType for the currently configured installation of the game.
    pub fn get_install_type(&self) -> Result<InstallType> {
        let base_path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten().ok_or(Error::from(ErrorKind::GamePathNotConfigured))?;

        // Checks to guess what kind of installation we have.
        let base_path_files = get_files_from_subdir(&base_path, false)?;
        let install_type_by_exe = self.install_data.iter().filter_map(|(install_type, install_data)|
            if base_path_files.iter().filter_map(|path| if path.is_file() { path.file_name() } else { None }).any(|filename| filename == &**install_data.get_ref_executable()) {
                Some(install_type)
            } else { None }
        ).collect::<Vec<&InstallType>>();

        // If no compatible install data was found, use the first one we have.
        if install_type_by_exe.is_empty() {
            Ok(self.install_data.keys().next().unwrap().clone())
        }

        // If we only have one install type compatible with the executable we have, return it.
        else if install_type_by_exe.len() == 1 {
            Ok(install_type_by_exe[0].clone())
        }

        // If we have multiple install data compatible, it gets more complex.
        else {

            // First, identify if we have a windows or linux build (mac only exists in your dreams.....).
            // Can't be both because they have different exe names. Unless you're retarded and you merge both, in which case, fuck you.
            let is_windows = install_type_by_exe.iter().any(|install_type| install_type == &&InstallType::WinSteam || install_type == &&InstallType::WinEpic || install_type == &&InstallType::WinWargaming);

            if is_windows {

                // Steam versions of the game have a "steam_api.dll" or "steam_api64.dll" file. Epic has "EOSSDK-Win64-Shipping.dll".
                let has_steam_api_dll = base_path_files.iter().filter_map(|path| path.file_name()).any(|filename| filename == "steam_api.dll" || filename == "steam_api64.dll");
                let has_eos_sdk_dll = base_path_files.iter().filter_map(|path| path.file_name()).any(|filename| filename == "EOSSDK-Win64-Shipping.dll");
                if has_steam_api_dll && install_type_by_exe.contains(&&InstallType::WinSteam) {
                    Ok(InstallType::WinSteam)
                }

                // If not, check wether we have epic libs.
                else if has_eos_sdk_dll && install_type_by_exe.contains(&&InstallType::WinEpic) {
                    Ok(InstallType::WinEpic)
                }

                // If neither of those are true, asume it's wargaming/netease (arena?).
                else {
                    Ok(InstallType::WinWargaming)
                }
            }

            // Otherwise, assume it's linux
            else {
                Ok(InstallType::LnxSteam)
            }
        }
    }

    /// This function gets the `/data` path or equivalent of the game selected, if said game it's configured in the settings
    pub fn get_data_path(&self) -> Result<PathBuf> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten().ok_or(Error::from(ErrorKind::GamePathNotConfigured))?;
        let install_type = self.get_install_type()?;
        let install_data = self.install_data.get(&install_type).ok_or(Error::from(ErrorKind::GameNotSupported))?;
        Ok(path.join(PathBuf::from(install_data.get_ref_data_path())))
    }

    /// This function gets the `/data` path or equivalent (the folder local mods are installed during development) of the game selected, if said game it's configured in the settings
    pub fn get_local_mods_path(&self) -> Result<PathBuf> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten().ok_or(Error::from(ErrorKind::GamePathNotConfigured))?;
        let install_type = self.get_install_type()?;
        let install_data = self.install_data.get(&install_type).ok_or(Error::from(ErrorKind::GameNotSupported))?;
        Ok(path.join(PathBuf::from(install_data.get_ref_local_mods_path())))
    }

    /// This function gets the `/assembly_kit` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn get_assembly_kit_path(&self) -> Result<PathBuf> {
        SETTINGS.read().unwrap().paths.get(&(self.get_game_key_name() + "_assembly_kit")).cloned().flatten().ok_or(Error::from(ErrorKind::GameAssemblyKitPathNotConfigured))
    }

    /// This function returns the assembly kit raw data path, or an error if this game doesn't have a known path.
    pub fn get_assembly_kit_db_tables_path(&self) -> Result<PathBuf> {
        let mut base_path = self.get_assembly_kit_path()?;
        let version = self.get_raw_db_version();
        match version {

            // Post-Shogun 2 games.
            2 | 1 => {
                base_path.push("raw_data/db");
                Ok(base_path)
            }

            // Shogun 2/Older games
            _ => Err(ErrorKind::AssemblyKitUnsupportedVersion(version).into())
        }
    }

    /// This function gets the `/mods` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn get_content_packfiles_paths(&self) -> Option<Vec<PathBuf>> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten()?;
        let install_type = self.get_install_type().ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let downloaded_mods_path = install_data.get_ref_downloaded_mods_path();

        // If the path is empty, it means this game does not support downloaded mods.
        if downloaded_mods_path.is_empty() {
            return None;
        }

        let path = path.join(PathBuf::from(downloaded_mods_path));
        let mut paths = vec![];

        for path in get_files_from_subdir(&path, true).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
                None => continue,
            }
        }

        paths.sort();
        Some(paths)
    }

    /// This function gets the `/data` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn get_data_packfiles_paths(&self) -> Option<Vec<PathBuf>> {
        let game_path = self.get_data_path().ok()?;
        let mut paths = vec![];

        for path in get_files_from_subdir(&game_path, false).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
                None => continue,
            }
        }

        paths.sort();
        Some(paths)
    }


    /// This function gets the destination folder for MyMod packs.
    pub fn get_mymod_install_path(&self) -> Option<PathBuf> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten()?;
        let install_type = self.get_install_type().ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let path = path.join(PathBuf::from(install_data.get_ref_local_mods_path()));

        // Make sure the folder exists.
        DirBuilder::new().recursive(true).create(&path).ok()?;

        Some(path)
    }

    /// This function gets the `/data/xxx.pack` or equivalent paths with db tables of the game selected, if said game it's configured in the settings.
    pub fn get_db_packs_paths(&self) -> Option<Vec<PathBuf>> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten()?;
        let install_type = self.get_install_type().ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let data_path = path.join(PathBuf::from(install_data.get_ref_data_path()));
        let db_packs_paths = install_data.get_ref_db_packs();

        let mut full_paths = vec![];
        for pack_path in db_packs_paths {
            let mut path = data_path.to_path_buf();
            path.push(pack_path);
            full_paths.push(path);
        }

        Some(full_paths)
    }

    /// This function gets the `/data/xxx.pack` or equivalent paths with loc tables of the game selected, if said game it's configured in the settings.
    pub fn get_loc_packs_paths(&self) -> Option<Vec<PathBuf>> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten()?;
        let install_type = self.get_install_type().ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let data_path = path.join(PathBuf::from(install_data.get_ref_data_path()));
        let loc_packs_paths = install_data.get_ref_loc_packs();

        let mut full_paths = vec![];
        for pack_path in loc_packs_paths {
            let mut path = data_path.to_path_buf();
            path.push(pack_path);
            full_paths.push(path);
        }

        Some(full_paths)
    }

    /// This function returns if we should use the manifest of the game (if found) to get the vanilla PackFiles, or if we should get them from out hardcoded list.
    pub fn use_manifest(&self) -> Result<bool> {
        let install_type = self.get_install_type()?;
        let install_data = self.install_data.get(&install_type).ok_or(ErrorKind::GameNotSupported)?;

        // If the install_type is linux, or we actually have a hardcoded list, ignore all Manifests.
        Ok(*install_data.get_ref_use_manifest())
    }

    /// This function is used to get the paths of all CA PackFiles on the data folder of the game selected.
    ///
    /// If it fails to find a manifest, it falls back to all non-mod files!
    pub fn get_all_ca_packfiles_paths(&self) -> Result<Vec<PathBuf>> {

        // Check if we can use the manifest for this.
        if !self.use_manifest()? {
            self.get_all_ca_packfiles_paths_no_manifest()
        } else {

            // Try to get the manifest, if exists.
            match Manifest::read_from_game_selected() {
                Ok(manifest) => {
                    let pack_file_names = manifest.0.iter().filter_map(|x|
                        if x.get_ref_relative_path().ends_with(".pack") {
                            Some(x.get_ref_relative_path().to_owned())
                        } else { None }
                        ).collect::<Vec<String>>();

                    let data_path = self.get_data_path()?;
                    Ok(pack_file_names.iter().map(|x| {
                        let mut pack_file_path = data_path.to_path_buf();
                        pack_file_path.push(x);
                        pack_file_path
                    }).collect::<Vec<PathBuf>>())
                }

                // If there is no manifest, use the hardcoded file list for the game, if it has one.
                Err(_) => self.get_all_ca_packfiles_paths_no_manifest()
            }
        }
    }

    /// This function tries to get the ca PackFiles without depending on a Manifest. For internal use only.
    fn get_all_ca_packfiles_paths_no_manifest(&self) -> Result<Vec<PathBuf>> {
        let data_path = self.get_data_path()?;
        let install_type = self.get_install_type()?;
        let vanilla_packs = &self.install_data.get(&install_type).ok_or(ErrorKind::GameNotSupported)?.vanilla_packs;
        if !vanilla_packs.is_empty() {
            Ok(vanilla_packs.iter().map(|x| {
                let mut pack_file_path = data_path.to_path_buf();
                pack_file_path.push(x);
                pack_file_path
            }).collect::<Vec<PathBuf>>())
        }

        // If there is no hardcoded list, get every path.
        else {
            Ok(get_files_from_subdir(&data_path, false)?.iter()
                .filter_map(|x| if let Some(extension) = x.extension() {
                    if extension.to_string_lossy().to_lowercase() == "pack" {
                        Some(x.to_owned())
                    } else { None }
                } else { None }).collect::<Vec<PathBuf>>()
            )
        }
    }

    /// This command returns the "launch" command for executing this game's installation.
    pub fn get_game_launch_command(&self) -> Result<String> {
        let install_type = self.get_install_type()?;
        match install_type {
            InstallType::LnxSteam |
            InstallType::WinSteam => Ok(format!("steam://rungameid/{}", self.install_data.get(&install_type).ok_or(ErrorKind::GameSelectedPathNotCorrectlyConfigured)?.get_ref_store_id())),
            _ => todo!()
        }
    }

    /// This command returns the "Executable" path for the game's installation.
    pub fn get_executable_path(&self) -> Option<PathBuf> {
        let path = SETTINGS.read().unwrap().paths.get(&self.get_game_key_name()).cloned().flatten()?;
        let install_type = self.get_install_type().ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let executable_path = path.join(PathBuf::from(install_data.get_ref_executable()));

        Some(executable_path)
    }
}
