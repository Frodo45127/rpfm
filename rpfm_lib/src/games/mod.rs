//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module that contains the GameInfo definition and stuff related with it.

!*/


use std::collections::HashMap;
use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use getset::*;
#[cfg(feature = "integration_log")] use log::{info, warn};

use crate::error::{RLibError, Result};
use crate::utils::*;

use self::supported_games::KEY_TROY;
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

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the info needed for a game to be "supported" by RPFM.
#[derive(Clone, Debug)]
pub struct GameInfo {

    /// This is the internal key of the game.
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
    game_selected_icon: String,

    /// Name of the big icon used to display the game as `Game Selected`, in an UI.
    game_selected_big_icon: String,

    /// Logic used to name vanilla tables.
    vanilla_db_table_name_logic: VanillaDBTableNameLogic,

    /// Installation-dependant data.
    install_data: HashMap<InstallType, InstallData>,

    /// Tool-specific vars for each game.
    tool_vars: HashMap<String, String>,

    /// Subfolder under Lua Autogen's folder where the files for this game are, if it's supported.
    lua_autogen_folder: Option<String>
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
struct InstallData {

    /// List of vanilla packs, to be use as reference for knowing what PackFiles are vanilla in games without a manifest file.
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

    /// Path where the language.txt file of the game is expected to be. Usually /data, but it's different on linux builds. Relative to the game's path.
    language_path: String,

    /// Folder where local (your own) mods are stored. Relative to the game's path.
    local_mods_path: String,

    /// Folder where downloaded (other peoples's) mods are stored. Relative to the game's path.
    downloaded_mods_path: String,
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
    pub fn game_key_name(&self) -> &str {
        self.key
    }

    /// This function returns the "Display" name of the Game, meaning properly written.
    pub fn display_name(&self) -> &str {
        self.display_name
    }

    /// This function returns the PFHVersion corresponding to the provided PackFile type. If it's not found, it defaults to the one used by mods.
    pub fn pfh_version_by_file_type(&self, pfh_file_type: PFHFileType) -> PFHVersion {
        match self.pfh_versions.get(&pfh_file_type) {
            Some(pfh_version) => *pfh_version,
            None => *self.pfh_versions.get(&PFHFileType::Mod).unwrap(),
        }
    }

    /// This function returns the full list of compatible PFHVersions for this game.
    pub fn pfh_versions(&self) -> &HashMap<PFHFileType, PFHVersion> {
        &self.pfh_versions
    }

    /// This function returns this Game's schema file name.
    pub fn schema_file_name(&self) -> &str {
        &self.schema_file_name
    }

    /// This function returns this Game's dependencies cache file name.
    pub fn dependencies_cache_file_name(&self) -> &str {
        &self.dependencies_cache_file_name
    }

    /// This function returns this Game's raw_db_version, used to identify how to process AssKit table files for this game.
    pub fn raw_db_version(&self) -> i16 {
        self.raw_db_version
    }

    /// This function returns this Game's PortraitSettings version, if any.
    pub fn portrait_settings_version(&self) -> Option<u32> {
        self.portrait_settings_version
    }

    /// This function returns whether this Game supports editing or not.
    pub fn supports_editing(&self) -> bool {
        self.supports_editing
    }

    /// This function returns whether this Game's tables should have a GUID in their header or not.
    pub fn db_tables_have_guid(&self) -> bool {
        self.db_tables_have_guid
    }

    /// This function returns the file with the language of the game, if any.
    pub fn locale_file_name(&self) -> &Option<String> {
        &self.locale_file_name
    }

    /// This function returns this Game's icon filename. Normal size.
    pub fn icon_file_name(&self) -> &str {
        &self.game_selected_icon
    }

    /// This function returns this Game's icon filename. Big size.
    pub fn icon_big_file_name(&self) -> &str {
        &self.game_selected_big_icon
    }

    /// This function returns this Game's logic for naming db tables.
    pub fn vanilla_db_table_name_logic(&self) -> &VanillaDBTableNameLogic {
        &self.vanilla_db_table_name_logic
    }

    /// This function returns this Game's logic for naming db tables.
    pub fn lua_autogen_folder(&self) -> Option<&str> {
        self.lua_autogen_folder.as_deref()
    }

    //---------------------------------------------------------------------------//
    // Advanced getters.
    //---------------------------------------------------------------------------//

    /// This function tries to get the correct InstallType for the currently configured installation of the game.
    pub fn install_type(&self, game_path: &Path) -> Result<InstallType> {

        // Checks to guess what kind of installation we have.
        let base_path_files = files_from_subdir(game_path, false)?;
        let install_type_by_exe = self.install_data.iter().filter_map(|(install_type, install_data)|
            if base_path_files.iter().filter_map(|path| if path.is_file() { path.file_name() } else { None }).any(|filename| filename == &**install_data.executable()) {
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

                // If neither of those are true, assume it's wargaming/netease (arena?).
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

        let path = game_path.join(downloaded_mods_path);
        let mut paths = vec![];

        for path in files_from_subdir(&path, true).ok()?.iter() {
            match path.extension() {
                Some(extension) => if extension == "pack" { paths.push(path.to_path_buf()); }
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
                                        let language = format!("local_{language}");
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
                                Some(pack_file_path)
                            } else {
                                None
                            }
                        } else {
                            Some(pack_file_path)
                        }
                    }
                    None => Some(pack_file_path),
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
    fn game_locale_from_file(&self, game_path: &Path) -> Result<Option<String>> {
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
        match self.game_key_name() {
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
}
