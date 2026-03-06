//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Game-specific configuration and metadata for Total War games.
//!
//! This module provides comprehensive information about supported Total War games,
//! including file formats, installation locations, and game-specific behaviors.
//!
//! # Overview
//!
//! RPFM supports multiple Total War games, each with different:
//! - PackFile (PFH) format versions
//! - Installation types (Steam/Epic/Wargaming, Windows/Linux)
//! - File locations (/data, /content, config paths)
//! - Assembly Kit versions and schemas
//! - Localization and language support
//! - Workshop tags and Steam integration
//!
//! # Main Types
//!
//! - [`GameInfo`]: Complete game configuration including paths, versions, and features
//! - [`SupportedGames`]: Registry of all games supported by RPFM
//! - [`InstallType`]: Platform and store variant (Steam/Epic/Wargaming, Windows/Linux)
//! - [`InstallData`]: Installation-specific paths and identifiers
//! - [`Manifest`]: Game manifest file parser for vanilla PackFile lists
//! - [`PFHFileType`]: Type of PackFile (Boot, Release, Patch, Mod, Movie)
//! - [`PFHVersion`]: PackFile format version
//!
//! # Usage Patterns
//!
//! ## Getting Game Information
//!
//! ```ignore
//! use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let supported_games = SupportedGames::default();
//! let game = supported_games.game(&KEY_WARHAMMER_3).unwrap();
//!
//! println!("Game: {}", game.display_name());
//! println!("Schema: {}", game.schema_file_name());
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Game Paths
//!
//! ```ignore
//! # use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//! # use std::path::Path;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let supported_games = SupportedGames::default();
//! # let game = supported_games.game(&KEY_WARHAMMER_3).unwrap();
//! let game_path = Path::new("/path/to/game");
//!
//! // Get various game-specific paths
//! let data_path = game.data_path(game_path)?;
//! let content_path = game.content_path(game_path)?;
//! let local_mods_path = game.local_mods_path(game_path)?;
//!
//! // Get vanilla PackFiles
//! let ca_packs = game.ca_packs_paths(game_path)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Detecting Installation Type
//!
//! ```ignore
//! # use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
//! # use rpfm_lib::games::InstallType;
//! # use std::path::Path;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let supported_games = SupportedGames::default();
//! # let game = supported_games.game(&KEY_WARHAMMER_3).unwrap();
//! # let game_path = Path::new("/path/to/game");
//! let install_type = game.install_type(game_path)?;
//!
//! match install_type {
//!     InstallType::WinSteam => println!("Windows Steam version"),
//!     InstallType::LnxSteam => println!("Linux Steam version"),
//!     InstallType::WinEpic => println!("Windows Epic version"),
//!     InstallType::WinWargaming => println!("Windows Wargaming version"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Submodules
//!
//! - [`supported_games`]: Game registry with all supported Total War titles
//! - [`manifest`]: Manifest file parsing for game PackFile lists
//! - [`pfh_file_type`]: PackFile type classifications
//! - [`pfh_version`]: PackFile format version definitions

use directories::ProjectDirs;
use getset::*;
use log::{info, warn};
use steamlocate::SteamDir;

use std::collections::HashMap;
use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::compression::CompressionFormat;
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

/// Language code: Brazilian Portuguese
pub const BRAZILIAN: &str = "br";
/// Language code: Simplified Chinese
pub const SIMPLIFIED_CHINESE: &str = "cn";
/// Language code: Czech
pub const CZECH: &str = "cz";
/// Language code: English
pub const ENGLISH: &str = "en";
/// Language code: French
pub const FRENCH: &str = "fr";
/// Language code: German
pub const GERMAN: &str = "ge";
/// Language code: Italian
pub const ITALIAN: &str = "it";
/// Language code: Korean
pub const KOREAN: &str = "kr";
/// Language code: Polish
pub const POLISH: &str = "pl";
/// Language code: Russian
pub const RUSSIAN: &str = "ru";
/// Language code: Spanish
pub const SPANISH: &str = "sp";
/// Language code: Turkish
pub const TURKISH: &str = "tr";
/// Language code: Traditional Chinese
pub const TRADITIONAL_CHINESE: &str = "zh";

/// Local folder name for Lua autogen files
pub const LUA_AUTOGEN_FOLDER: &str = "tw_autogen";
/// Git repository URL for Lua autogen type definitions
pub const LUA_REPO: &str = "https://github.com/chadvandy/tw_autogen";
/// Git remote name for Lua autogen repository
pub const LUA_REMOTE: &str = "origin";
/// Git branch name for Lua autogen repository
pub const LUA_BRANCH: &str = "main";

/// Git repository URL for old (pre-Shogun 2) Assembly Kit files
pub const OLD_AK_REPO: &str = "https://github.com/Frodo45127/total_war_ak_files_pre_shogun_2";
/// Git remote name for old Assembly Kit repository
pub const OLD_AK_REMOTE: &str = "origin";
/// Git branch name for old Assembly Kit repository
pub const OLD_AK_BRANCH: &str = "master";

/// Git repository URL for community translation hub
pub const TRANSLATIONS_REPO: &str = "https://github.com/Frodo45127/total_war_translation_hub";
/// Git remote name for translations repository
pub const TRANSLATIONS_REMOTE: &str = "origin";
/// Git branch name for translations repository
pub const TRANSLATIONS_BRANCH: &str = "master";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Complete configuration and metadata for a supported Total War game.
///
/// This struct contains all information needed for RPFM to work with a specific
/// Total War game, including file formats, paths, features, and game-specific behaviors.
///
/// # Organization
///
/// The struct can be organized into several logical groups:
/// - **Identity**: Key, display name
/// - **File Formats**: PFH versions, schema files, compression formats
/// - **Features**: Editing support, GUID requirements, portrait settings
/// - **Assembly Kit**: Raw DB version, lost fields list
/// - **Paths & Installation**: Install data per platform/store
/// - **Localization**: Language file, locale support
/// - **Tools**: Lua autogen, tool variables
/// - **Restrictions**: Banned files, validation logic
///
/// # Access Patterns
///
/// Most fields are accessed through getters provided by the `Getters` derive macro.
/// Some methods provide computed values based on installation detection:
/// - [`GameInfo::install_type()`] - Detects the installation variant
/// - [`GameInfo::data_path()`] - Resolves game-specific /data path
/// - [`GameInfo::ca_packs_paths()`] - Lists vanilla PackFiles
///
/// # Installation Detection
///
/// The struct supports multiple installation types per game and automatically
/// detects which one is present by examining executables and DLL files in the
/// game directory. See [`GameInfo::install_type()`] for details.
#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct GameInfo {

    /// Internal game identifier key (e.g., `"warhammer_3"`).
    ///
    /// Used for directory names, file lookups, and programmatic identification.
    #[getset(skip)]
    key: &'static str,

    /// User-friendly display name (e.g., `"Warhammer 3"`).
    ///
    /// Shown in UI dropdowns and messages.
    display_name: &'static str,

    /// PackFile header versions by file type.
    ///
    /// Maps [`PFHFileType`] (Boot, Release, Patch, Mod, Movie) to the appropriate
    /// [`PFHVersion`] for this game. If a type isn't in the map, defaults to Mod type.
    pfh_versions: HashMap<PFHFileType, PFHVersion>,

    /// Schema file name for this game (e.g., `"schema_wh3.ron"`).
    ///
    /// Used to load table definitions for decoding DB files.
    schema_file_name: String,

    /// Dependencies cache file name for this game.
    ///
    /// Stores cached dependency tree for faster pack loading.
    dependencies_cache_file_name: String,

    /// Assembly Kit version for raw database files.
    ///
    /// - `-1`: No Assembly Kit available
    /// - `0`: Empire/Napoleon format
    /// - `1`: Shogun 2 format
    /// - `2`: Rome 2 and later format
    raw_db_version: i16,

    /// Portrait settings file version for this game.
    ///
    /// `None` if the game doesn't use portrait settings files.
    portrait_settings_version: Option<u32>,

    /// Whether PackFiles can be saved for this game.
    ///
    /// Some very old games are read-only.
    supports_editing: bool,

    /// Whether DB table headers include GUIDs.
    ///
    /// Newer games include a GUID in the table header for identification.
    db_tables_have_guid: bool,

    /// Language/locale file name (e.g., `"language.txt"`).
    ///
    /// `None` if the game doesn't use a language file, or if all locales are loaded.
    locale_file_name: Option<String>,

    /// Paths to files that RPFM should never edit.
    ///
    /// Contains table names or file paths that are protected by the game's integrity
    /// checks to prevent bypassing DLC ownership validation.
    banned_packedfiles: Vec<String>,

    /// Small icon file name for UI display.
    icon_small: String,

    /// Large icon file name for UI display.
    icon_big: String,

    /// Logic for naming vanilla DB table files.
    ///
    /// Some games name tables after their folder, others use a default name.
    vanilla_db_table_name_logic: VanillaDBTableNameLogic,

    /// Installation data per platform/store combination.
    ///
    /// Contains paths, executables, and store IDs for different installation types.
    /// Not exposed by getters - use [`GameInfo::install_data()`] instead.
    #[getset(skip)]
    install_data: HashMap<InstallType, InstallData>,

    /// Game-specific tool variables.
    ///
    /// Key-value pairs for tool-specific configuration.
    tool_vars: HashMap<String, String>,

    /// Subdirectory name in Lua autogen repository for this game.
    ///
    /// `None` if Lua autogen doesn't support this game.
    lua_autogen_folder: Option<String>,

    /// Assembly Kit fields that are lost during export.
    ///
    /// List of `table_name.field_name` entries that exist in vanilla data
    /// but don't appear in Assembly Kit exports because they are either unused
    /// or separated from the tables on export.
    ak_lost_fields: Vec<String>,

    /// Internal cache for install type detection.
    ///
    /// Speeds up repeated calls to [`GameInfo::install_type()`].
    #[getset(skip)]
    install_type_cache: Arc<RwLock<HashMap<PathBuf, InstallType>>>,

    /// Supported compression formats, newest to oldest.
    ///
    /// Used to determine which compression to use when saving files.
    compression_formats_supported: Vec<CompressionFormat>,

    /// Maximum CS2.parsed format version supported.
    ///
    /// Used for cross-game model conversion compatibility.
    max_cs2_parsed_version: u32,
}

/// Strategy for naming vanilla DB table files.
///
/// Different Total War games use different conventions for naming their
/// database table files in vanilla PackFiles.
#[derive(Clone, Debug)]
pub enum VanillaDBTableNameLogic {

    /// Table files are named after their containing folder.
    ///
    /// Example: `db/units_tables/` contains file named `units_tables`
    FolderName,

    /// All table files use the same default name.
    ///
    /// Example: All tables are named `data` or similar
    DefaultName(String),
}

/// Game installation platform and store variant.
///
/// Represents the different ways a Total War game can be installed,
/// which affects executable names, DLL dependencies, and paths.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum InstallType {

    /// Windows installation from Steam.
    ///
    /// Identified by presence of `steam_api.dll` or `steam_api64.dll`.
    WinSteam,

    /// Linux installation from Steam.
    ///
    /// Identified by Linux executable names.
    LnxSteam,

    /// Windows installation from Epic Games Store.
    ///
    /// Identified by presence of `EOSSDK-Win64-Shipping.dll`.
    WinEpic,

    /// Windows installation from Wargaming/Netease platform.
    ///
    /// Used for Arena and similar special distributions.
    WinWargaming,
}

/// Installation-specific paths and identifiers.
///
/// Contains all the data that varies between different installation types of
/// the same game (Steam vs Epic, Windows vs Linux, etc.).
///
/// # Path Relativity
///
/// **Important**: All paths in this struct are RELATIVE paths, either to:
/// - The game's root directory (most paths)
/// - The data directory (`vanilla_packs`)
///
/// This allows the same configuration to work across different installation
/// locations by combining with the actual game path at runtime.
#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct InstallData {

    /// Vanilla PackFile names (without paths).
    ///
    /// Used as fallback when no manifest file exists (Empire, Napoleon).
    /// Paths are relative to the `data_path`.
    vanilla_packs: Vec<String>,

    /// Whether to use the game's manifest file for vanilla PackFile discovery.
    ///
    /// `true`: Read manifest file (most games)
    /// `false`: Use hardcoded `vanilla_packs` list
    use_manifest: bool,

    /// Steam/store ID for the game.
    ///
    /// Used for Steam integration and auto-detection.
    store_id: u64,

    /// Steam/store ID for the game's Assembly Kit.
    ///
    /// `0` if no Assembly Kit is available on Steam.
    store_id_ak: u64,

    /// Game executable file name (with extension).
    ///
    /// Used to detect installation type and for launching the game.
    /// Examples: `"Warhammer3.exe"`, `"Shogun2.exe"`
    executable: String,

    /// Data directory path relative to game root.
    ///
    /// Where PackFiles are stored. Usually `"data"` but varies.
    data_path: String,

    /// Language file directory path relative to game root.
    ///
    /// Where `language.txt` or equivalent is located.
    /// May be different from `data_path` on Linux builds.
    language_path: String,

    /// Local mods directory path relative to game root.
    ///
    /// Where the game loads locally-installed mods from.
    local_mods_path: String,

    /// Downloaded mods directory path relative to game root.
    ///
    /// Where Steam Workshop and other downloaded mods are stored.
    /// Empty string if the game doesn't support downloadable mods.
    downloaded_mods_path: String,

    /// Config directory name (not full path).
    ///
    /// Used with platform-specific config locations (AppData on Windows,
    /// .config on Linux). `None` if game doesn't store config externally.
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

    /// Returns the game's unique identifier key.
    ///
    /// The key is the game name in lowercase without spaces (e.g., `"warhammer_3"`, `"troy"`).
    /// This is used for configuration files, file paths, and internal identification.
    ///
    /// # Returns
    ///
    /// A static string slice containing the game's key identifier.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
    ///
    /// let supported_games = SupportedGames::default();
    /// let game_info = supported_games.game(&KEY_WARHAMMER_3).unwrap();
    /// assert_eq!(game_info.key(), KEY_WARHAMMER_3);
    /// ```
    pub fn key(&self) -> &str {
        self.key
    }

    /// Returns the PackFile format version for a specific file type.
    ///
    /// Different PackFile types (Boot, Release, Patch, Mod, Movie) may use different format
    /// versions within the same game. This method looks up the appropriate [`PFHVersion`]
    /// for the given [`PFHFileType`].
    ///
    /// # Arguments
    ///
    /// * `pfh_file_type` - The type of PackFile to look up
    ///
    /// # Returns
    ///
    /// The [`PFHVersion`] used for the specified file type. If no specific version is
    /// configured for the file type, returns the version used for Mod PackFiles.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
    /// use rpfm_lib::games::pfh_file_type::PFHFileType;
    ///
    /// let supported_games = SupportedGames::default();
    /// let game_info = supported_games.game(&KEY_WARHAMMER_3).unwrap();
    /// let mod_version = game_info.pfh_version_by_file_type(PFHFileType::Mod);
    /// ```
    pub fn pfh_version_by_file_type(&self, pfh_file_type: PFHFileType) -> PFHVersion {
        match self.pfh_versions.get(&pfh_file_type) {
            Some(pfh_version) => *pfh_version,
            None => *self.pfh_versions.get(&PFHFileType::Mod).unwrap(),
        }
    }

    //---------------------------------------------------------------------------//
    // Advanced getters.
    //---------------------------------------------------------------------------//

    /// Detects the installation type (Steam, Epic, Wargaming, etc.) for a game installation.
    ///
    /// This method analyzes the game's directory structure and files to determine which
    /// platform or distribution the game was installed from. The result is cached to avoid
    /// repeated filesystem scans.
    ///
    /// # Detection Strategy
    ///
    /// 1. Checks for platform-specific executable names
    /// 2. For Windows installations with multiple possible types:
    ///    - Looks for `steam_api.dll` or `steam_api64.dll` for Steam
    ///    - Looks for `EOSSDK-Win64-Shipping.dll` for Epic Games Store
    ///    - Falls back to Wargaming/Netease if neither found
    /// 3. Assumes Linux Steam for Linux installations
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Returns the detected [`InstallType`], or an error if the path is invalid.
    ///
    /// # Performance
    ///
    /// Results are cached internally. First call takes ~10ms, subsequent calls are instant.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::games::supported_games::{SupportedGames, KEY_WARHAMMER_3};
    /// use std::path::Path;
    ///
    /// let supported_games = SupportedGames::default();
    /// let game_info = supported_games.game(&KEY_WARHAMMER_3).unwrap();
    /// let game_path = Path::new("/path/to/game");
    ///
    /// match game_info.install_type(game_path) {
    ///     Ok(install_type) => println!("Detected: {:?}", install_type),
    ///     Err(e) => eprintln!("Detection failed: {}", e),
    /// }
    /// ```
    pub fn install_type(&self, game_path: &Path) -> Result<InstallType> {

        // This function takes 10ms to execute. In a few places, it's executed 2-5 times, and quickly adds up.
        // So before executing it, check the cache to see if it has been executed before.
        if let Some(install_type) = self.install_type_cache.read().unwrap().get(game_path) {
            return Ok(install_type.clone());
        }

        // Checks to guess what kind of installation we have.
        let base_path_files = files_from_subdir(game_path, false)?;
        let install_type_by_exe = self.install_data.iter().filter_map(|(install_type, install_data)|
            if base_path_files.iter().filter_map(|path| if path.is_file() { path.file_name() } else { None }).any(|filename| filename.to_ascii_lowercase() == *install_data.executable().to_lowercase()) {
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

    /// Returns the installation-specific data for a game.
    ///
    /// After detecting the installation type, this method retrieves the corresponding
    /// configuration data (executable names, paths, Steam IDs, etc.) for that installation.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Returns a reference to the [`InstallData`] for the detected installation type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The installation type cannot be detected
    /// - The detected installation type is not supported for this game
    pub fn install_data(&self, game_path: &Path) -> Result<&InstallData> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(install_data)
    }

    /// Returns the path to the game's `/data` directory.
    ///
    /// The `/data` directory contains the game's vanilla PackFiles and is the primary
    /// location for game content. The exact directory name may vary by game and platform.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the `/data` directory (or platform-specific equivalent).
    ///
    /// # Errors
    ///
    /// Returns an error if the installation type cannot be detected or is not supported.
    pub fn data_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.data_path()))
    }

    /// Returns the path to the downloaded mods directory.
    ///
    /// This is the directory where Steam Workshop or other platform mods are downloaded to.
    /// Not all games support downloaded mods through official platforms.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the downloaded mods directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the installation type cannot be detected or is not supported.
    pub fn content_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.downloaded_mods_path()))
    }

    /// Returns the directory containing the game's language configuration file.
    ///
    /// The language configuration file (typically `language.txt` or similar) stores the
    /// player's selected interface language. The file location varies by game and may be
    /// nested inside a language-specific subdirectory.
    ///
    /// # Behavior
    ///
    /// If the language file is in the base directory, returns that directory. Otherwise,
    /// searches through language-specific subdirectories (brazilian, chinese, english, etc.)
    /// and returns the first one found.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the directory containing the language file.
    ///
    /// # Errors
    ///
    /// Returns an error if the installation type cannot be detected or is not supported.
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

    /// Returns the path to the local mods directory.
    ///
    /// This is where locally-installed mods are loaded from by the game. The location
    /// varies by game:
    /// - **Troy**: A separate directory from `/data` to avoid polluting the data folder
    /// - **Other games**: Points to the `/data` directory
    ///
    /// Mods placed in this directory are loaded by the game without requiring workshop
    /// or platform distribution.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the local mods directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the installation type cannot be detected or is not supported.
    pub fn local_mods_path(&self, game_path: &Path) -> Result<PathBuf> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;
        Ok(game_path.join(install_data.local_mods_path()))
    }

    /// Returns paths to all PackFiles in the downloaded mods directory.
    ///
    /// Recursively scans the downloaded mods directory (Steam Workshop, etc.) and returns
    /// paths to all `.pack` and `.bin` files found. Returns `None` if the game doesn't
    /// support downloaded mods or the directory doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// A sorted vector of absolute paths to PackFiles, or `None` if not applicable.
    ///
    /// # File Extensions
    ///
    /// Searches for both `.pack` and `.bin` extensions as some games use `.bin` for
    /// certain mod types.
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

    /// Returns paths to all PackFiles in a secondary mods directory.
    ///
    /// Some users keep additional mod collections in custom directories outside the game
    /// installation. This method scans a user-specified secondary path for PackFiles.
    ///
    /// The secondary path should contain a subdirectory named after the game's key
    /// (e.g., `secondary_path/warhammer_3/`), which is then scanned for `.pack` files.
    ///
    /// # Arguments
    ///
    /// * `secondary_path` - Absolute path to the base secondary mods directory
    ///
    /// # Returns
    ///
    /// A sorted vector of absolute paths to PackFiles, or `None` if:
    /// - The path is not absolute, doesn't exist, or isn't a directory
    /// - The game-specific subdirectory doesn't exist
    /// - No `.pack` files are found
    ///
    /// # Path Structure
    ///
    /// Expected structure: `secondary_path/{game_key}/*.pack`
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

    /// Returns paths to all PackFiles in the game's `/data` directory.
    ///
    /// Scans the game's main data directory (non-recursively) for all `.pack` files.
    /// This typically includes vanilla game PackFiles and any mods installed directly
    /// in the data directory.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// A sorted vector of absolute paths to PackFiles, or `None` if:
    /// - The data directory cannot be determined
    /// - The directory doesn't exist or cannot be read
    /// - No `.pack` files are found
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


    /// Returns the installation path for "MyMod" PackFiles.
    ///
    /// Returns the directory where mods created with RPFM's "MyMod" feature should be
    /// installed. This is typically the local mods directory. Creates the directory
    /// if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the MyMod installation directory, or `None` if:
    /// - The installation type cannot be detected
    /// - The directory cannot be created
    pub fn mymod_install_path(&self, game_path: &Path) -> Option<PathBuf> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let path = game_path.join(PathBuf::from(install_data.local_mods_path()));

        // Make sure the folder exists.
        DirBuilder::new().recursive(true).create(&path).ok()?;

        Some(path)
    }

    /// Returns whether to use the game's manifest file for discovering vanilla PackFiles.
    ///
    /// Some games have a `manifest.txt` file that lists all official PackFiles. This method
    /// determines whether RPFM should use that manifest or fall back to a hardcoded list
    /// of PackFile names.
    ///
    /// # Decision Logic
    ///
    /// Returns `false` (don't use manifest) if:
    /// - The installation is Linux (manifests may be unreliable)
    /// - A hardcoded PackFile list exists for this game/install type
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// `true` if the manifest should be used, `false` if the hardcoded list should be used.
    ///
    /// # Errors
    ///
    /// Returns an error if the installation type cannot be detected or is not supported.
    pub fn use_manifest(&self, game_path: &Path) -> Result<bool> {
        let install_type = self.install_type(game_path)?;
        let install_data = self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?;

        // If the install_type is linux, or we actually have a hardcoded list, ignore all Manifests.
        Ok(*install_data.use_manifest())
    }

    /// Returns the Steam App ID for the game installation.
    ///
    /// The Steam App ID is used for launching games via Steam, checking workshop content,
    /// and other Steam-specific integrations.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// The Steam App ID as a 64-bit unsigned integer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The installation type cannot be detected
    /// - The installation is not a Steam installation (Windows or Linux)
    /// - The installation type is not supported for this game
    pub fn steam_id(&self, game_path: &Path) -> Result<u64> {
        let install_type = self.install_type(game_path)?;
        let install_data = match install_type {
            InstallType::WinSteam |
            InstallType::LnxSteam => self.install_data.get(&install_type).ok_or_else(|| RLibError::GameInstallTypeNotSupported(self.display_name.to_string(), install_type.to_string()))?,
            _ => return Err(RLibError::ReservedFiles)
        };

        Ok(*install_data.store_id())
    }

    /// Returns paths to all Creative Assembly (vanilla) PackFiles.
    ///
    /// Discovers all official game PackFiles in the data directory. Uses the game's manifest
    /// file if available and configured, otherwise falls back to a hardcoded list or scanning
    /// all PackFiles.
    ///
    /// # Language Filtering
    ///
    /// For games with multiple language packs (e.g., Warhammer 3), only returns PackFiles
    /// matching the configured game language. This prevents loading multiple language
    /// localizations simultaneously.
    ///
    /// Language-specific PackFiles typically have `local_{language}` in their names
    /// (e.g., `local_en.pack`, `local_es.pack`).
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// A sorted vector of absolute paths to vanilla PackFiles.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The game language cannot be determined
    /// - The data directory cannot be accessed
    /// - The installation type is not supported
    ///
    /// # Fallback Behavior
    ///
    /// If manifest reading fails, automatically falls back to `ca_packs_paths_no_manifest()`.
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

    /// Returns vanilla PackFiles without using a manifest (internal fallback).
    ///
    /// This is an internal method used by [`ca_packs_paths`] when no manifest is available
    /// or manifest reading fails. Uses a hardcoded list of PackFile names if available,
    /// otherwise returns all `.pack` files in the data directory.
    ///
    /// # Language Filtering
    ///
    /// Like [`ca_packs_paths`], filters language-specific PackFiles to only include the
    /// configured game language.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    /// * `language` - Optional language code to filter localization PackFiles
    ///
    /// # Returns
    ///
    /// A vector of absolute paths to vanilla PackFiles.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data directory cannot be accessed
    /// - The installation type is not supported
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

    /// Returns the launch URI for starting the game.
    ///
    /// Generates a platform-specific URI or command that can be used to launch the game
    /// from external applications or scripts.
    ///
    /// # Platform Support
    ///
    /// Currently only supports Steam installations (Windows and Linux), which use Steam URIs
    /// in the format `steam://rungameid/{app_id}`.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// A string containing the launch URI or command.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The installation type cannot be detected
    /// - The installation platform doesn't support programmatic launching (e.g., Epic, Wargaming)
    /// - The installation type is not supported for this game
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

    /// Returns the path to the game's executable file.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the game executable, or `None` if:
    /// - The installation type cannot be detected
    /// - The installation type is not supported
    pub fn executable_path(&self, game_path: &Path) -> Option<PathBuf> {
        let install_type = self.install_type(game_path).ok()?;
        let install_data = self.install_data.get(&install_type)?;
        let executable_path = game_path.join(install_data.executable());

        Some(executable_path)
    }

    /// Returns the path to the game's configuration directory.
    ///
    /// Total War games store user configuration, preferences, and save files in a
    /// platform-specific configuration directory (e.g., AppData on Windows, ~/.config on Linux).
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// Absolute path to the game's configuration directory, or `None` if:
    /// - The installation type cannot be detected
    /// - The game doesn't have a defined configuration folder
    /// - The platform-specific configuration path cannot be determined
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

    /// Checks if a file path is banned from modification.
    ///
    /// Some game files are protected by integrity checks to prevent bypassing DLC
    /// ownership validation. This method checks if a file path matches any banned
    /// path prefixes.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to check (typically a PackFile-relative path)
    ///
    /// # Returns
    ///
    /// `true` if the file is banned and should not be modified, `false` otherwise.
    ///
    /// # Comparison
    ///
    /// The comparison is case-insensitive and uses prefix matching.
    pub fn is_file_banned(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.banned_packedfiles.iter().any(|x| path.starts_with(x))
    }

    /// Retrieves a game-specific tool variable.
    ///
    /// Tool variables are key-value pairs used to configure tool behavior for specific
    /// games. Examples might include special file paths, version numbers, or feature flags.
    ///
    /// # Arguments
    ///
    /// * `var` - The variable name to look up
    ///
    /// # Returns
    ///
    /// The variable value if found, or `None` if the variable is not defined for this game.
    pub fn tool_var(&self, var: &str) -> Option<&String> {
        self.tool_vars.get(var)
    }

    /// Reads the game's configured language from its configuration file.
    ///
    /// Attempts to read the game's language setting from its `language.txt` or equivalent
    /// configuration file. This determines which localization PackFiles should be loaded.
    ///
    /// # Language Codes
    ///
    /// The file typically contains a 2-letter code (e.g., "EN", "ES", "DE") which is
    /// mapped to the full language name used in PackFile names.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// - `Ok(Some(language))` - Language successfully read and mapped
    /// - `Ok(Some("english"))` - File missing/unreadable, defaulted to English
    /// - `Ok(None)` - Game doesn't use a language configuration file
    ///
    /// # Errors
    ///
    /// Returns an error if the language file path cannot be determined due to
    /// installation type detection failures.
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
                    info!("Language file found, using {language} language.");
                    Ok(Some(language))
                } else {
                    warn!("Missing or unreadable language file under {}. Using english language.", game_path.to_string_lossy());
                    Ok(Some(ENGLISH.to_owned()))
                }
            }
            None => Ok(None),
        }
    }

    /// Extracts the version number from the game's executable.
    ///
    /// Reads version information embedded in the game's executable file. Currently only
    /// implemented for Troy; returns `None` for other games.
    ///
    /// # Version Encoding
    ///
    /// The version is encoded as a 32-bit integer:
    /// - Bits 24-31: Major version
    /// - Bits 16-23: Minor version
    /// - Bits 8-15: Patch version
    /// - Bits 0-7: Build number
    ///
    /// For example, version 1.3.0.5 would be encoded as `0x01030005`.
    ///
    /// # Arguments
    ///
    /// * `game_path` - Absolute path to the game's installation directory
    ///
    /// # Returns
    ///
    /// The encoded version number, or `None` if:
    /// - Version extraction is not implemented for this game
    /// - The executable doesn't exist
    /// - The executable version info cannot be read
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

    /// Automatically discovers the game's installation directory.
    ///
    /// Searches for the game installation using platform-specific methods. Currently only
    /// supports Steam installations via the Steam library folders system.
    ///
    /// # Platform Support
    ///
    /// - **Windows Steam**: Searches via Steam library folders
    /// - **Linux Steam**: Searches via Steam library folders
    /// - **Other platforms**: Not supported (returns `Ok(None)`)
    ///
    /// # Arguments
    ///
    /// None - uses the game's configured Steam App ID
    ///
    /// # Returns
    ///
    /// - `Ok(Some(path))` - Game installation found at the returned path
    /// - `Ok(None)` - Game not found or platform not supported
    ///
    /// # Errors
    ///
    /// Returns an error if Steam library folder parsing fails.
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

    /// Automatically discovers the Assembly Kit installation directory.
    ///
    /// Assembly Kits are official modding tools distributed separately from the games.
    /// This method searches for the Assembly Kit using platform-specific methods,
    /// currently only supporting Steam installations.
    ///
    /// # Platform Support
    ///
    /// - **Windows Steam**: Searches via Steam library folders
    /// - **Linux Steam**: Searches via Steam library folders
    /// - **Other platforms**: Not supported (returns `Ok(None)`)
    ///
    /// # Arguments
    ///
    /// None - uses the game's configured Assembly Kit Steam App ID
    ///
    /// # Returns
    ///
    /// - `Ok(Some(path))` - Assembly Kit found at the returned path
    /// - `Ok(None)` - Assembly Kit not found, not available for this game, or platform not supported
    ///
    /// # Errors
    ///
    /// Returns an error if Steam library folder parsing fails.
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

    /// Returns the list of Steam Workshop tags available for this game.
    ///
    /// Steam Workshop allows mod creators to tag their mods with categories like "graphical",
    /// "campaign", "units", etc. This method returns the official list of tags recognized
    /// by the Steam Workshop for this specific game.
    ///
    /// # Tag Categories
    ///
    /// Common tags across games include:
    /// - Content types: "graphical", "campaign", "units", "battle"
    /// - Scope: "overhaul", "ui", "maps"
    /// - Collections: "compilation", "mod manager"
    /// - Languages: "English", "Spanish", etc. (in some games)
    ///
    /// # Returns
    ///
    /// A vector of tag strings recognized by Steam Workshop for this game.
    ///
    /// # Errors
    ///
    /// Returns an error if the game doesn't support Steam Workshop.
    pub fn steam_workshop_tags(&self) -> Result<Vec<String>> {
        Ok(match self.key() {
            KEY_PHARAOH_DYNASTIES => vec![
                String::from("mod"),
                String::from("graphical"),
                String::from("campaign"),
                String::from("ui"),
                String::from("battle"),
                String::from("overhaul"),
                String::from("units"),
            ],
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

    /// Looks up a game by its Steam App ID.
    ///
    /// Given a Steam App ID, searches through all supported games to find the matching game.
    /// This is useful when you have a Steam App ID (e.g., from Steam library or launch parameters)
    /// and need to identify which Total War game it corresponds to.
    ///
    /// # Arguments
    ///
    /// * `steam_id` - The Steam App ID to search for
    ///
    /// # Returns
    ///
    /// The [`GameInfo`] for the matching game.
    ///
    /// # Errors
    ///
    /// Returns an error if no known game matches the provided Steam App ID.
    ///
    /// # Example
    ///
    /// ```
    /// use rpfm_lib::games::GameInfo;
    ///
    /// // Look up Warhammer 3 by its Steam App ID
    /// let game_info = GameInfo::game_by_steam_id(1142710).unwrap();
    /// assert_eq!(game_info.key(), "warhammer_3");
    /// ```
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
