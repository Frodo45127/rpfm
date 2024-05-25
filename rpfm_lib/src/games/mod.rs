//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module that contains the GameInfo definition and stuff related with it.

use directories::ProjectDirs;
use getset::*;
#[cfg(feature = "integration_log")] use log::{info, warn};
use steamlocate::SteamDir;

use std::collections::HashMap;
use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::{RLibError, Result};
use crate::utils::*;

use self::supported_games::*;
use self::manifest::Manifest;
use self::pfh_file_type::PFHFileType;
use self::pfh_version::PFHVersion;

pub mod supported_games;
pub mod manifest;
pub mod pfh_file_type;
pub mod pfh_version;

const BRAZILIAN: &str = "br";
const SIMPLIFIED_CHINESE: &str = "cn";
const CZECH: &str = "cz";
const ENGLISH: &str = "en";
const FRENCH: &str = "fr";
const GERMAN: &str = "ge";
const ITALIAN: &str = "it";
const KOREAN: &str = "kr";
const POLISH: &str = "pl";
const RUSSIAN: &str = "ru";
const SPANISH: &str = "sp";
const TURKISH: &str = "tr";
const TRADITIONAL_CHINESE: &str = "zh";

pub const LUA_AUTOGEN_FOLDER: &str = "tw_autogen";
pub const LUA_REPO: &str = "https://github.com/chadvandy/tw_autogen";
pub const LUA_REMOTE: &str = "origin";
pub const LUA_BRANCH: &str = "main";

pub const OLD_AK_REPO: &str = "https://github.com/Frodo45127/total_war_ak_files_pre_shogun_2";
pub const OLD_AK_REMOTE: &str = "origin";
pub const OLD_AK_BRANCH: &str = "master";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the info needed for a game to be "supported" by RPFM.
#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct GameInfo {

    /// This is the internal key of the game.
    #[getset(skip)]
    key: &'static str,

    /// This is the name it'll show up for the user. The *pretty name*. For example, in a dropdown (Warhammer 2).
    display_name: &'static str,

    /// This is the PFHVersion used at the start of every PackFile for that game.
    /// It's in a hashmap of PFHFileType => PFHVersion, so we can have different PackFile versions depending on their type.
    pfh_versions: HashMap<PFHFileType, PFHVersion>,

    /// This is the full name of the schema file used for the game. For example: `schema_wh2.ron`.
    schema_file_name: String,

    /// This is the name of the file containing the dependencies cache for this game.
    dependencies_cache_file_name: String,

    /// This is the **type** of raw files the game uses. -1 is "Don't have Assembly Kit". 0 is Empire/Nappy. 1 is Shogun 2. 2 is anything newer than Shogun 2.
    raw_db_version: i16,

    /// This is the version used when generating PortraitSettings files for each game.
    portrait_settings_version: Option<u32>,

    /// If we can save `PackFile` files for the game.
    supports_editing: bool,

    /// If the db tables should have a GUID in their headers.
    db_tables_have_guid: bool,

    /// If the game has locales for all languages, and we only need to load our own locales. Contains the name of the locale file.
    locale_file_name: Option<String>,

    /// List of tables (table_name) which the program should NOT EDIT UNDER ANY CIRCUnSTANCE.
    banned_packedfiles: Vec<String>,

    /// Name of the icon used to display the game as `Game Selected`, in an UI.
    icon_small: String,

    /// Name of the big icon used to display the game as `Game Selected`, in an UI.
    icon_big: String,

    /// Logic used to name vanilla tables.
    vanilla_db_table_name_logic: VanillaDBTableNameLogic,

    /// Installation-dependant data.
    #[getset(skip)]
    install_data: HashMap<InstallType, InstallData>,

    /// Tool-specific vars for each game.
    tool_vars: HashMap<String, String>,

    /// Subfolder under Lua Autogen's folder where the files for this game are, if it's supported.
    lua_autogen_folder: Option<String>,

    /// Table/fields ignored on the assembly kit integration for this game. These are fields that are "lost" when exporting the tables from Dave.
    ak_lost_fields: Vec<String>,

    /// Internal cache to speedup operations related with the install type.
    #[getset(skip)]
    install_type_cache: Arc<RwLock<HashMap<PathBuf, InstallType>>>
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
#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct InstallData {

    /// List of vanilla packs, to be use as reference for knowing what PackFiles are vanilla in games without a manifest file.
    /// Currently only used for Empire and Napoleon. Relative to data_path.
    vanilla_packs: Vec<String>,

    /// If the manifest of the game should be used to get the vanilla PackFile list, or should we use the hardcoded list.
    use_manifest: bool,

    /// StoreID of the game.
    store_id: u64,

    /// StoreID of the AK.
    store_id_ak: u64,

    /// Name of the executable of the game, including extension if it has it.
    executable: String,

    /// /data path of the game, or equivalent. Relative to the game's path.
    data_path: String,

    /// Path where the language.txt file of the game is expected to be. Usually /data, but it's different on linux builds. Relative to the game's path.
    language_path: String,

    /// Folder where local (your own) mods are stored. Relative to the game's path.
    local_mods_path: String,

    /// Folder where downloaded (other peoples's) mods are stored. Relative to the game's path.
    downloaded_mods_path: String,

    /// Name of the folder where the config for this specific game installation are stored.
    config_folder: Option<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Display for InstallType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::WinSteam => "Windows - Steam",
            Self::LnxSteam => "Linux - Steam",
            Self::WinEpic => "Windows - Epic",
            Self::WinWargaming => "Windows - Wargaming",
        }, f)
    }
}

/// Implementation of GameInfo.
impl GameInfo {

    //---------------------------------------------------------------------------//
    // Getters.
    //---------------------------------------------------------------------------//

    /// This function returns the "Key" name of the Game, meaning in lowercase and without spaces.
    pub fn key(&self) -> &str {
        self.key
    }

    /// This function returns the PFHVersion corresponding to the provided PackFile type. If it's not found, it defaults to the one used by mods.
    pub fn pfh_version_by_file_type(&self, pfh_file_type: PFHFileType) -> PFHVersion {
        match self.pfh_versions.get(&pfh_file_type) {
            Some(pfh_version) => *pfh_version,
            None => *self.pfh_versions.get(&PFHFileType::Mod).unwrap(),
        }
    }

    //---------------------------------------------------------------------------//
    // Advanced getters.
    //---------------------------------------------------------------------------//

    /// This function tries to get the correct InstallType for the currently configured installation of the game.
    pub fn install_type(&self, game_path: &Path) -> Result<InstallType> {

        // This function takes 10ms to execute. In a few places, it's executed 2-5 times, and quickly adds up.
        // So before executing it, check the cache to see if it has been executed before.
        if let Some(install_type) = self.install_type_cache.read().unwrap().get(game_path) {
            return Ok(install_type.clone());
        }

        // Checks to guess what kind of installation we have.
        let base_path_files = files_from_subdir(game_path, false)?;
        let install_type_by_exe = self.install_data.iter().filter_map(|(install_type, install_data)|
            if base_path_files.iter().filter_map(|path| if path.is_file() { path.file_name() } else { None }).any(|filename| filename == &**install_data.executable()) {
                Some(install_type)
            } else { None }
        ).collect::<Vec<&InstallType>>();

        // If no compatible install data was found, use the first one we have.
        if install_type_by_exe.is_empty() {
            let install_type = self.install_data.keys().next().unwrap();
            self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), install_type.clone());
            Ok(install_type.clone())
        }

        // If we only have one install type compatible with the executable we have, return it.
        else if install_type_by_exe.len() == 1 {
            self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), install_type_by_exe[0].clone());
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
                    self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), InstallType::WinSteam);
                    Ok(InstallType::WinSteam)
                }

                // If not, check wether we have epic libs.
                else if has_eos_sdk_dll && install_type_by_exe.contains(&&InstallType::WinEpic) {
                    self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), InstallType::WinEpic);
                    Ok(InstallType::WinEpic)
                }

                // If neither of those are true, assume it's wargaming/netease (arena?).
                else {
                    self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), InstallType::WinWargaming);
                    Ok(InstallType::WinWargaming)
                }
            }

            // Otherwise, assume it's linux
            else {
                self.install_type_cache.write().unwrap().insert(game_path.to_path_buf(), InstallType::LnxSteam);
                Ok(InstallType::LnxSteam)
            }
        }
    }

    /// This function gets the install data for the game, if it's a supported installation.
    pub fn install_data(&self, game_path: &Path) -> Result<&InstallData> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(install_data)
    }

    /// This function gets the `/data` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn data_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.data_path()))
    }

    /// This function gets the `/contents` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn content_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.downloaded_mods_path()))
    }

    /// This function gets the `language.txt` path of the game selected, if said game uses it and it's configured in the settings.
    pub fn language_path(&self, game_path: &Path) -> Result<PathBuf> {

        // For games that don't support
        let language_file_name = self.locale_file_name().clone().unwrap_or_else(|| "language.txt".to_owned());

        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        let base_path = game_path.join(install_data.language_path());

        // The language files are either in this folder, or in a folder with the locale value inside this folder.
        let path_with_file = base_path.join(language_file_name);
        if path_with_file.is_file() {
            Ok(base_path)
        } else {

            // Yes, this is ugly. But I'm not the retarded idiot that decided to put the file that sets the language used inside a folder specific of the language used.
            let path = base_path.join(BRAZILIAN);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(SIMPLIFIED_CHINESE);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(CZECH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(ENGLISH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(FRENCH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(GERMAN);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(ITALIAN);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(KOREAN);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(POLISH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(RUSSIAN);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(SPANISH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(TURKISH);
            if path.is_dir() {
                return Ok(path);
            }
            let path = base_path.join(TRADITIONAL_CHINESE);
            if path.is_dir() {
                return Ok(path);
            }

            // If no path exists, we just return the base path.
            Ok(base_path)
        }
    }

    /// This function gets the `/data` path or equivalent (the folder local mods are installed during development) of the game selected, if said game it's configured in the settings
    pub fn local_mods_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.local_mods_path()))
    }

    /// This function gets the `/mods` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn content_packs_paths(&self, game_path: &Path) -> Option<Vec<PathBuf>> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let downloaded_mods_path = install_data.downloaded_mods_path();

        // If the path is empty, it means this game does not support downloaded mods.
        if downloaded_mods_path.is_empty() {
            return None;
        }

        let path = std::fs::canonicalize(game_path.join(downloaded_mods_path)).ok()?;
        let mut paths = vec![];

        for path in files_from_subdir(&path, true).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" || extension == "bin" { paths.push(path.to_path_buf()); }
                None => continue,
            }
        }

        paths.sort();
        Some(paths)
    }

    /// This function gets the paths of the Packs from the `/secondary` path or equivalent of the game selected, if it's configured in the settings.
    ///
    /// Secondary path must be absolute.
    pub fn secondary_packs_paths(&self, secondary_path: &Path) -> Option<Vec<PathBuf>> {
        if !secondary_path.is_dir() || !secondary_path.exists() || !secondary_path.is_absolute() {
            return None;
        }

        let game_path = secondary_path.join(self.key());
        if !game_path.is_dir() || !game_path.exists() {
            return None;
        }

        let mut paths = vec![];

        for path in files_from_subdir(&game_path, false).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" {
                    paths.push(path.to_path_buf());
                }
                None => continue,
            }
        }

        paths.sort();
        Some(paths)
    }

    /// This function gets the `/data` path or equivalent of the game selected, if said game it's configured in the settings.
    pub fn data_packs_paths(&self, game_path: &Path) -> Option<Vec<PathBuf>> {
        let game_path = self.data_path(game_path).ok()?;
        let mut paths = vec![];

        for path in files_from_subdir(&game_path, false).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
                None => continue,
            }
        }

        paths.sort();
        Some(paths)
    }


    /// This function gets the destination folder for MyMod packs.
    pub fn mymod_install_path(&self, game_path: &Path) -> Option<PathBuf> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let path = game_path.join(PathBuf::from(install_data.local_mods_path()));

        // Make sure the folder exists.
        DirBuilder::new().recursive(true).create(&path).ok()?;

        Some(path)
    }

    /// This function returns if we should use the manifest of the game (if found) to get the vanilla PackFiles, or if we should get them from out hardcoded list.
    pub fn use_manifest(&self, game_path: &Path) -> Result<bool> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;

        // If the install_type is linux, or we actually have a hardcoded list, ignore all Manifests.
        Ok(*install_data.use_manifest())
    }

    /// This function returns the steam id for a specific game installation.
    pub fn steam_id(&self, game_path: &Path) -> Result<u64> {
        let install_type = self.install_type(game_path)?;
        let install_data = match install_type {
            InstallType::WinSteam |
            InstallType::LnxSteam => self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?,
            _ => return Err(RLibError::ReservedFiles)
        };

        Ok(*install_data.store_id())
    }

    /// This function is used to get the paths of all CA PackFiles on the data folder of the game selected.
    ///
    /// If it fails to find a manifest, it falls back to all non-mod files!
    ///
    /// NOTE: For WH3, this is language-sensitive. Meaning, if you have the game on spanish, it'll try to load the spanish localization files ONLY.
    pub fn ca_packs_paths(&self, game_path: &Path) -> Result<Vec<PathBuf>> {

        // Check if we have to filter by language, to avoid overwriting our language with another one.
        let language = self.game_locale_from_file(game_path)?;

        // Check if we can use the manifest for this.
        if !self.use_manifest(game_path)? {
            self.ca_packs_paths_no_manifest(game_path, &language)
        } else {

            // Try to get the manifest, if exists.
            match Manifest::read_from_game_path(self, game_path) {
                Ok(manifest) => {
                    let data_path = self.data_path(game_path)?;
                    let mut paths = manifest.0.iter().filter_map(|entry|
                        if entry.relative_path().ends_with(".pack") {

                            let mut pack_file_path = data_path.to_path_buf();
                            pack_file_path.push(entry.relative_path());
                            match &language {
                                Some(language) => {

                                    // Filter out other language's packfiles.
                                    if entry.relative_path().contains("local_") {
                                        let language = "local_".to_owned() + language;
                                        if entry.relative_path().contains(&language) {
                                            entry.path_from_manifest_entry(pack_file_path)
                                        } else {
                                            None
                                        }
                                    } else {
                                        entry.path_from_manifest_entry(pack_file_path)
                                    }
                                }
                                None => entry.path_from_manifest_entry(pack_file_path),
                            }
                        } else { None }
                        ).collect::<Vec<PathBuf>>();

                    paths.sort();
                    Ok(paths)
                }

                // If there is no manifest, use the hardcoded file list for the game, if it has one.
                Err(_) => self.ca_packs_paths_no_manifest(game_path, &language)
            }
        }
    }

    /// This function tries to get the ca PackFiles without depending on a Manifest. For internal use only.
    fn ca_packs_paths_no_manifest(&self, game_path: &Path, language: &Option<String>) -> Result<Vec<PathBuf>> {
        let data_path = self.data_path(game_path)?;
        let install_type = self.install_type(game_path)?;
        let vanilla_packs = &self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?.vanilla_packs;
        let language_pack = language.clone().map(|lang| format!("local_{lang}"));
        if !vanilla_packs.is_empty() {
            Ok(vanilla_packs.iter().filter_map(|pack_name| {

                let mut pack_file_path = data_path.to_path_buf();
                pack_file_path.push(pack_name);
                match language_pack {
                    Some(ref language_pack) => {

                        // Filter out other language's packfiles.
                        if !pack_name.is_empty() && pack_name.starts_with("local_") {
                            if pack_name.starts_with(language_pack) {
                                std::fs::canonicalize(pack_file_path).ok()
                            } else {
                                None
                            }
                        } else {
                            std::fs::canonicalize(pack_file_path).ok()
                        }
                    }
                    None => std::fs::canonicalize(pack_file_path).ok(),
                }
            }).collect::<Vec<PathBuf>>())
        }

        // If there is no hardcoded list, get every path.
        else {
            Ok(files_from_subdir(&data_path, false)?.iter()
                .filter_map(|x| if let Some(extension) = x.extension() {
                    if extension.to_string_lossy().to_lowercase() == "pack" {
                        Some(x.to_owned())
                    } else { None }
                } else { None }).collect::<Vec<PathBuf>>()
            )
        }
    }

    /// This command returns the "launch" command for executing this game's installation.
    pub fn game_launch_command(&self, game_path: &Path) -> Result<String> {
        let install_type = self.install_type(game_path)?;

        match install_type {
            InstallType::LnxSteam |
            InstallType::WinSteam => {
                let store_id = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?.store_id();
                Ok(format!("steam://rungameid/{store_id}"))
            },
            _ => Err(RLibError::GameInstallLaunchNotSupported(self.display_name.to_string(), install_type.to_string())),
        }
    }

    /// This command returns the "Executable" path for the game's installation.
    pub fn executable_path(&self, game_path: &Path) -> Option<PathBuf> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let executable_path = game_path.join(install_data.executable());

        Some(executable_path)
    }

    /// This command returns the "config" path for the game's installation.
    pub fn config_path(&self, game_path: &Path) -> Option<PathBuf> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let config_folder = install_data.config_folder.as_ref()?;

        ProjectDirs::from("com", "The Creative Assembly", config_folder).map(|dir| {
            let mut dir = dir.config_dir().to_path_buf();
            dir.pop();
            dir
        })
    }

    /// Check if a specific file is banned.
    pub fn is_file_banned(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.banned_packedfiles.iter().any(|x| path.starts_with(x))
    }

    /// Tries to retrieve a tool var for the game.
    pub fn tool_var(&self, var: &str) -> Option<&String> {
        self.tool_vars.get(var)
    }

    /// This function tries to get the language of the game. Defaults to english if not found.
    pub fn game_locale_from_file(&self, game_path: &Path) -> Result<Option<String>> {
        match self.locale_file_name() {
            Some(locale_file) => {
                let language_path = self.language_path(game_path)?;
                let locale_path = language_path.join(locale_file);
                let mut language = String::new();
                if let Ok(mut file) = File::open(locale_path) {
                    file.read_to_string(&mut language)?;

                    let language = match &*language {
                        "BR" => BRAZILIAN.to_owned(),
                        "CN" => SIMPLIFIED_CHINESE.to_owned(),
                        "CZ" => CZECH.to_owned(),
                        "EN" => ENGLISH.to_owned(),
                        "FR" => FRENCH.to_owned(),
                        "DE" => GERMAN.to_owned(),
                        "IT" => ITALIAN.to_owned(),
                        "KR" => KOREAN.to_owned(),
                        "PO" => POLISH.to_owned(),
                        "RU" => RUSSIAN.to_owned(),
                        "ES" => SPANISH.to_owned(),
                        "TR" => TURKISH.to_owned(),
                        "ZH" => TRADITIONAL_CHINESE.to_owned(),

                        // Default to english if we can't find the proper one.
                        _ => ENGLISH.to_owned(),
                    };

                    #[cfg(feature = "integration_log")] {
                        info!("Language file found, using {} language.", language);
                    }

                    Ok(Some(language))
                } else {
                    #[cfg(feature = "integration_log")] {
                        warn!("Missing or unreadable language file under {}. Using english language.", game_path.to_string_lossy());
                    }
                    Ok(Some(ENGLISH.to_owned()))
                }
            }
            None => Ok(None),
        }
    }

    /// This function gets the version number of the exe for the current GameSelected, if it exists.
    pub fn game_version_number(&self, game_path: &Path) -> Option<u32> {
        match self.key() {
            KEY_TROY => {
                let exe_path = self.executable_path(game_path)?;
                if exe_path.is_file() {
                    let mut data = vec![];
                    let mut file = BufReader::new(File::open(exe_path).ok()?);
                    file.read_to_end(&mut data).ok()?;

                    let version_info = pe_version_info(&data).ok()?;
                    let version_info = version_info.fixed()?;
                    let mut version: u32 = 0;

                    // The CA format is limited so these can only be u8 when encoded, so we can safetly convert them.
                    let major = version_info.dwFileVersion.Major as u32;
                    let minor = version_info.dwFileVersion.Minor as u32;
                    let patch = version_info.dwFileVersion.Patch as u32;
                    let build = version_info.dwFileVersion.Build as u32;

                    version += major << 24;
                    version += minor << 16;
                    version += patch << 8;
                    version += build;
                    Some(version)
                }

                // If we have no exe, return a default value.
                else {
                    None
                }

            }

            _ => None,
        }
    }

    /// This function searches for installed total war games.
    ///
    /// NOTE: Only works for steam-installed games.
    pub fn find_game_install_location(&self) -> Result<Option<PathBuf>> {

        // Steam install data. We don't care if it's windows or linux, as the data we want is the same in both.
        let install_data = if let Some(install_data) = self.install_data.get(&InstallType::WinSteam) {
            install_data
        } else if let Some(install_data) = self.install_data.get(&InstallType::LnxSteam) {
            install_data
        } else {
            return Ok(None);
        };

        if install_data.store_id() > &0 {
            if let Ok(steamdir) = SteamDir::locate() {
                return match steamdir.find_app(*install_data.store_id() as u32) {
                    Ok(Some((app, lib))) => {
                        let app_path = lib.resolve_app_dir(&app);
                        if app_path.is_dir() {
                            Ok(Some(app_path.to_path_buf()))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None)
                }
            }
        }

        Ok(None)
    }

    /// This function searches for installed total war Assembly Kits.
    ///
    /// NOTE: Only works for steam-installed games.
    pub fn find_assembly_kit_install_location(&self) -> Result<Option<PathBuf>> {

        // Steam install data. We don't care if it's windows or linux, as the data we want is the same in both.
        let install_data = if let Some(install_data) = self.install_data.get(&InstallType::WinSteam) {
            install_data
        } else if let Some(install_data) = self.install_data.get(&InstallType::LnxSteam) {
            install_data
        } else {
            return Ok(None);
        };

        if install_data.store_id_ak() > &0 {
            if let Ok(steamdir) = SteamDir::locate() {
                return match steamdir.find_app(*install_data.store_id_ak() as u32) {
                    Ok(Some((app, lib))) => {
                        let app_path = lib.resolve_app_dir(&app);
                        if app_path.is_dir() {
                            Ok(Some(app_path.to_path_buf()))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None)
                }
            }
        }

        Ok(None)
    }

    /// This function returns the list of public tags available in the workshop for each game.
    pub fn steam_workshop_tags(&self) -> Result<Vec<String>> {
        Ok(match self.key() {
            KEY_PHARAOH => vec![
                String::from("mod"),
                String::from("graphical"),
                String::from("campaign"),
                String::from("ui"),
                String::from("battle"),
                String::from("overhaul"),
                String::from("units"),
            ],
            KEY_WARHAMMER_3 => vec![
                String::from("graphical"),
                String::from("campaign"),
                String::from("units"),
                String::from("battle"),
                String::from("ui"),
                String::from("maps"),
                String::from("overhaul"),
                String::from("compilation"),
                String::from("cheat"),
            ],
            KEY_TROY => vec![
                String::from("mod"),
                String::from("ui"),
                String::from("graphical"),
                String::from("units"),
                String::from("battle"),
                String::from("campaign"),
                String::from("overhaul"),
                String::from("compilation"),
            ],
            KEY_THREE_KINGDOMS => vec![
                String::from("mod"),
                String::from("graphical"),
                String::from("overhaul"),
                String::from("ui"),
                String::from("battle"),
                String::from("campaign"),
                String::from("maps"),
                String::from("units"),
                String::from("compilation"),
            ],
            KEY_WARHAMMER_2 => vec![
                String::from("mod"),
                String::from("Units"),
                String::from("Battle"),
                String::from("Graphical"),
                String::from("UI"),
                String::from("Campaign"),
                String::from("Maps"),
                String::from("Overhaul"),
                String::from("Compilation"),
                String::from("Mod Manager"),
                String::from("Skills"),
                String::from("map"),
            ],
            KEY_WARHAMMER => vec![
                String::from("mod"),
                String::from("UI"),
                String::from("Graphical"),
                String::from("Overhaul"),
                String::from("Battle"),
                String::from("Campaign"),
                String::from("Compilation"),
                String::from("Units"),
                String::from("Maps"),
                String::from("Spanish"),
                String::from("English"),
                String::from("undefined"),
                String::from("map"),
            ],
            KEY_THRONES_OF_BRITANNIA => vec![
                String::from("mod"),
                String::from("ui"),
                String::from("battle"),
                String::from("campaign"),
                String::from("units"),
                String::from("compilation"),
                String::from("graphical"),
                String::from("overhaul"),
                String::from("maps"),
            ],
            KEY_ATTILA => vec![
                String::from("mod"),
                String::from("UI"),
                String::from("Graphical"),
                String::from("Battle"),
                String::from("Campaign"),
                String::from("Units"),
                String::from("Overhaul"),
                String::from("Compilation"),
                String::from("Maps"),
                String::from("version_2"),
                String::from("Czech"),
                String::from("Danish"),
                String::from("English"),
                String::from("Finnish"),
                String::from("French"),
                String::from("German"),
                String::from("Hungarian"),
                String::from("Italian"),
                String::from("Japanese"),
                String::from("Korean"),
                String::from("Norwegian"),
                String::from("Romanian"),
                String::from("Russian"),
                String::from("Spanish"),
                String::from("Swedish"),
                String::from("Thai"),
                String::from("Turkish"),
            ],
            KEY_ROME_2 => vec![
                String::from("mod"),
                String::from("Units"),
                String::from("Battle"),
                String::from("Overhaul"),
                String::from("Compilation"),
                String::from("Campaign"),
                String::from("Graphical"),
                String::from("UI"),
                String::from("Maps"),
                String::from("version_2"),
                String::from("English"),
                String::from("gribble"),
                String::from("tribble"),
            ],
            KEY_SHOGUN_2 => vec![
                String::from("map"),
                String::from("historical"),
                String::from("multiplayer"),
                String::from("mod"),
                String::from("version_2"),
                String::from("English"),
                String::from("ui"),
                String::from("graphical"),
                String::from("overhaul"),
                String::from("units"),
                String::from("campaign"),
                String::from("battle"),
            ],
            _ => return Err(RLibError::GameDoesntSupportWorkshop(self.key().to_owned()))
        })
    }

    /// This function returns the game that corresponds to the provided Steam ID, if any.
    pub fn game_by_steam_id(steam_id: u64) -> Result<Self> {
        let games = SupportedGames::default();
        for game in games.games() {

            // No need to check LnxSteam, as they share the same id.
            match game.install_data.get(&InstallType::WinSteam) {
                Some(install_data) => if install_data.store_id == steam_id {
                    return Ok(game.clone());
                } else {
                    continue;
                }
                None => continue,
            }
        }

        Err(RLibError::SteamIDDoesntBelongToKnownGame(steam_id))
    }
}
