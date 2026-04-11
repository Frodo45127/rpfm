//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! # IPC Messages Module
//!
//! This module defines the core IPC protocol structures used for communication between the RPFM
//! frontend and backend server.
//!
//! ## Overview
//!
//! The protocol is built around three main types:
//!
//! - [`Message<T>`]: A generic wrapper that adds request-response correlation via unique IDs.
//! - [`Command`]: An enum defining all actions the frontend can request from the server.
//! - [`Response`]: An enum defining all possible results the server can return.
//!
//! ## Message Correlation
//!
//! Every message includes a unique `id` field that allows the frontend to match responses to their
//! original requests. This enables:
//!
//! - **Asynchronous communication**: Multiple requests can be in flight simultaneously.
//! - **Non-blocking UI**: The frontend doesn't need to wait for responses before sending new requests.
//! - **Error handling**: Responses can be matched back to the context that initiated them.
//!
//! ## Command Categories
//!
//! Commands are organized into logical groups:
//!
//! - **PackFile Operations**: Open, save, close, and modify PackFiles.
//! - **PackedFile Operations**: Create, delete, extract, rename, and decode individual files.
//! - **Dependency Operations**: Query and manage game dependencies.
//! - **Search Operations**: Global search and reference lookups.
//! - **Schema Operations**: Load, save, and update table schemas.
//! - **Settings Operations**: Get and set application settings.
//! - **Update Operations**: Check for and apply updates to schemas, translations, etc.
//! - **Diagnostics**: Run diagnostic checks on PackFiles.
//! - **Navigation**: Go-to-definition and reference search features.
//!
//! ## Response Types
//!
//! Responses are typically named after the types they contain (e.g., `Response::Bool(bool)`,
//! `Response::String(String)`). For complex operations, specialized responses like
//! `Response::DBRFileInfo` or `Response::ContainerInfoVecRFileInfo` carry domain-specific data.
//!
//! Each [`Command`] variant's documentation specifies which [`Response`] variant(s) it returns.

use serde::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;

use rpfm_extensions::dependencies::TableReferences;
use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::optimizer::OptimizerOptions;
use rpfm_extensions::search::{GlobalSearch, MatchHolder};
use rpfm_extensions::translator::PackTranslation;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{
    anim_fragment_battle::AnimFragmentBattle, anims_table::AnimsTable, atlas::Atlas, audio::Audio,
    bmd::Bmd, db::DB, esf::ESF, group_formations::GroupFormations, image::Image, loc::Loc,
    matched_combat::MatchedCombat, pack::PackSettings, portrait_settings::PortraitSettings,
    rigidmodel::RigidModel, text::Text, uic::UIC, unit_variant::UnitVariant,
    video::SupportedFormats, ContainerPath, RFile, RFileDecoded,
};
use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::integrations::git::GitResponse;
use rpfm_lib::notes::Note;
use rpfm_lib::schema::{Definition, DefinitionPatch, Field, Schema};

use crate::helpers::*;
use crate::settings_keys::SettingsSnapshot;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct is a wrapper for all messages (commands and responses) sent between the UI and the server.
///
/// It includes a unique ID to correlate responses with their original requests.
#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T: Debug> {
    pub id: u64,
    pub data: T,
}

/// This enum represents the current operational mode for a pack.
///
/// A pack can either be in normal mode or in MyMod mode, which links it to
/// a specific game folder and mod name for import/export operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalMode {

    /// MyMod mode enabled. Contains the game folder name (e.g. "warhammer_2") and the MyMod pack name.
    MyMod(String, String),

    /// Normal mode - no MyMod association.
    Normal,
}

impl Default for OperationalMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// This enum defines the commands (messages) you can send to the background thread in order to execute actions.
///
/// Each command should include the data needed for his own execution. For a more detailed explanation, check the
/// docs of each command.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {

    /// Close the background thread. Do not use this command directly.
    ///
    /// Response: None (breaks the loop).
    Exit,

    /// Signal that the client is intentionally disconnecting.
    ///
    /// This allows the server to immediately clean up the session's resources instead of
    /// waiting for the timeout. If this was the last active session, the server will also
    /// shut down.
    ///
    /// Response: [`Response::Success`] (sent before cleanup begins).
    ClientDisconnecting,

    //-----------------------------------------------------------------------//
    // PackFile Operations
    //-----------------------------------------------------------------------//

    /// Closes a specific open Pack identified by its pack key.
    ///
    /// Response: [`Response::Success`].
    ClosePack(String),

    /// Closes all currently open Packs.
    ///
    /// Response: [`Response::Success`].
    CloseAllPacks,

    /// Clean a specific open Pack from corrupted/undecoded files and try to save it to disk.
    /// First field is the pack key, second is the destination path.
    ///
    /// Only use this command if your Pack is not save-able otherwise.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    CleanAndSavePackAs(String, PathBuf),

    /// List all currently open packs with their keys and metadata.
    ///
    /// Response: [`Response::VecStringContainerInfo`].
    ListOpenPacks,

    /// Creates a new empty Pack.
    ///
    /// Response: [`Response::String`] with the assigned pack key.
    NewPack,

    /// Save a specific open Pack to disk. The field is the pack key.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    SavePack(String),

    /// Save a specific open Pack to a new path.
    /// First field is the pack key, second is the destination path.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    SavePackAs(String, PathBuf),

    /// Get the data used to build the `TreeView` for a specific pack.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::ContainerInfoVecRFileInfo`].
    GetPackFileDataForTreeView(String),

    /// Open one or more `PackFiles` and merge them. Requires the paths of the `PackFiles`].
    ///
    /// Response:
    /// - [`Response::StringContainerInfo`] (pack_key, info) on success.
    /// - [`Response::Error`] on failure.
    OpenPackFiles(Vec<PathBuf>),

    /// Open all the CA PackFiles for the selected game as one merged PackFile.
    ///
    /// Response:
    /// - [`Response::StringContainerInfo`] (pack_key, info) on success.
    /// - [`Response::Error`] on failure.
    LoadAllCAPackFiles,

    /// Get the `RFileInfo` of one or more `PackedFiles` from a specific pack.
    /// First field is the pack key, second is the list of file paths.
    ///
    /// Response: [`Response::VecRFileInfo`].
    GetPackedFilesInfo(String, Vec<String>),

    /// Perform a `Global Search` on a specific pack. Requires the pack key and search configuration.
    ///
    /// Response:
    /// - [`Response::GlobalSearchVecRFileInfo`] on success.
    /// - [`Response::Error`] if no schema.
    GlobalSearch(String, GlobalSearch),

    /// Change the `Game Selected`]. Contains the game key and whether to rebuild dependencies.
    ///
    /// Response:
    /// - [`Response::CompressionFormatDependenciesInfo`] on success.
    /// - [`Response::Error`] if game not supported.
    SetGameSelected(String, bool),

    /// Get the currently selected game key.
    ///
    /// Response: [`Response::String`].
    GetGameSelected,

    /// Change the `Type` of a specific open Pack.
    /// First field is the pack key, second is the new type.
    ///
    /// Response: [`Response::Success`].
    SetPackFileType(String, PFHFileType),

    /// Generate the dependencies cache for the selected game.
    ///
    /// Response:
    /// - [`Response::DependenciesInfo`] on success.
    /// - [`Response::Error`] on failure.
    GenerateDependenciesCache,

    /// Update the currently loaded Schema with data from the game's Assembly Kit.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateCurrentSchemaFromAssKit,

    /// Trigger an optimization pass over a specific open Pack.
    /// First field is the pack key, second is the optimizer options.
    ///
    /// Response:
    /// - [`Response::HashSetStringHashSetString`] (deleted paths, added paths) on success.
    /// - [`Response::Error`] on failure.
    OptimizePackFile(String, OptimizerOptions),

    /// Patch the SiegeAI of a Siege Map for Warhammer games in a specific pack.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::StringVecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    PatchSiegeAI(String),

    /// Change the `Index Includes Timestamp` flag in a specific open Pack.
    /// First field is the pack key, second is the flag value.
    ///
    /// Response: [`Response::Success`].
    ChangeIndexIncludesTimestamp(String, bool),

    /// Change the compression format of a specific open Pack.
    /// First field is the pack key, second is the compression format.
    ///
    /// Response:
    /// - [`Response::CompressionFormat`] (the actual format set, may differ if unsupported).
    ChangeCompressionFormat(String, CompressionFormat),

    /// Get the current path of a specific open Pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::PathBuf`].
    GetPackFilePath(String),

    /// Get the info of a single `PackedFile` from a specific pack.
    /// First field is the pack key, second is the file path.
    ///
    /// Response: [`Response::OptionRFileInfo`].
    GetRFileInfo(String, String),

    //-----------------------------------------------------------------------//
    // Update Commands
    //-----------------------------------------------------------------------//

    /// Check if there is an RPFM update available.
    ///
    /// Response:
    /// - [`Response::APIResponse`] on success.
    /// - [`Response::Error`] on failure.
    CheckUpdates,

    /// Check if there is a Schema update available.
    ///
    /// Response:
    /// - [`Response::APIResponseGit`] on success.
    /// - [`Response::Error`] on failure.
    CheckSchemaUpdates,

    /// Update the schemas from the remote repository.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateSchemas,

    /// Check if there is a Dependency Database loaded in memory.
    /// Pass true to ensure dependencies were built with the AssKit.
    ///
    /// Response: [`Response::Bool`].
    IsThereADependencyDatabase(bool),

    //-----------------------------------------------------------------------//
    // PackedFile Operations
    //-----------------------------------------------------------------------//

    /// Create a new `PackedFile` inside a specific open Pack.
    /// First field is the pack key, then path and NewFile info.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    NewPackedFile(String, String, NewFile),

    /// Add one or more Files to a specific open Pack.
    /// First field is the pack key, then source filesystem paths, destination container paths, optional paths to ignore.
    ///
    /// Response:
    /// - [`Response::VecContainerPathOptionString`] (added paths, optional error message).
    AddPackedFiles(String, Vec<PathBuf>, Vec<ContainerPath>, Option<Vec<PathBuf>>),

    /// Decode a PackedFile to be shown on the UI.
    /// First field is the pack key, then the path of the file and its data source.
    ///
    /// Response:
    /// - [`Response::AnimFragmentBattleRFileInfo`] for AnimFragmentBattle files.
    /// - [`Response::AnimPackRFileInfo`] for AnimPack files.
    /// - [`Response::AnimsTableRFileInfo`] for AnimsTable files.
    /// - [`Response::AtlasRFileInfo`] for Atlas files.
    /// - [`Response::AudioRFileInfo`] for Audio files.
    /// - [`Response::BmdRFileInfo`] for BMD files.
    /// - [`Response::DBRFileInfo`] for DB table files.
    /// - [`Response::ESFRFileInfo`] for ESF files.
    /// - [`Response::GroupFormationsRFileInfo`] for GroupFormations files.
    /// - [`Response::ImageRFileInfo`] for Image files.
    /// - [`Response::LocRFileInfo`] for Loc files.
    /// - [`Response::MatchedCombatRFileInfo`] for MatchedCombat files.
    /// - [`Response::PortraitSettingsRFileInfo`] for PortraitSettings files.
    /// - [`Response::RigidModelRFileInfo`] for RigidModel files.
    /// - [`Response::TextRFileInfo`] for Text files.
    /// - [`Response::UICRFileInfo`] for UIC files.
    /// - [`Response::UnitVariantRFileInfo`] for UnitVariant files.
    /// - [`Response::VideoInfoRFileInfo`] for Video files.
    /// - [`Response::VMDRFileInfo`] for VMD files.
    /// - [`Response::WSModelRFileInfo`] for WSModel files.
    /// - [`Response::Text`] for pack notes.
    /// - [`Response::Unknown`] for unsupported types.
    /// - [`Response::Error`] on failure.
    DecodePackedFile(String, String, DataSource),

    /// Save an edited `PackedFile` back to a specific Pack.
    /// First field is the pack key, then path and decoded file data.
    ///
    /// Response: [`Response::Success`].
    SavePackedFileFromView(String, String, RFileDecoded),

    /// Add PackedFiles from one open pack into another.
    /// First field is the target pack key, second is the source pack key, third is the paths to copy.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] if source pack not found.
    AddPackedFilesFromPackFile(String, String, Vec<ContainerPath>),

    /// Add PackedFiles from a specific pack to an AnimPack.
    /// First field is the pack key, then animpack path and container paths.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    AddPackedFilesFromPackFileToAnimpack(String, String, Vec<ContainerPath>),

    /// Add PackedFiles from an AnimPack to a specific pack.
    /// First field is the pack key, then data source, animpack path, and container paths.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    AddPackedFilesFromAnimpack(String, DataSource, String, Vec<ContainerPath>),

    /// Delete PackedFiles from an AnimPack in a specific pack.
    /// First field is the pack key, then animpack path and container paths.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    DeleteFromAnimpack(String, String, Vec<ContainerPath>),

    /// Delete one or more PackedFiles from a specific pack.
    /// First field is the pack key, second is the paths to delete.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] (deleted paths).
    DeletePackedFiles(String, Vec<ContainerPath>),

    /// Copy one or more PackedFiles to the internal clipboard.
    /// The field is a map of pack key to the paths to copy from that pack.
    /// This stores path references in a server-side clipboard for later pasting.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    CopyPackedFiles(BTreeMap<String, Vec<ContainerPath>>),

    /// Cut one or more PackedFiles to the internal clipboard.
    /// Same as copy, but the files will be removed from the source pack on paste.
    /// The field is a map of pack key to the paths to cut from that pack.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    CutPackedFiles(BTreeMap<String, Vec<ContainerPath>>),

    /// Paste PackedFiles from the internal clipboard into a pack.
    /// First field is the target pack key, second is the destination folder path.
    ///
    /// Response:
    /// - [`Response::VecContainerPathVecContainerPathString`] (added paths, cut-deleted paths, source pack key) on success.
    /// - [`Response::Error`] on failure.
    PastePackedFiles(String, String),

    /// Duplicate one or more PackedFiles in-place within the same pack.
    /// First field is the pack key, second is the paths to duplicate.
    /// Files are cloned with a numeric suffix added to avoid name collisions.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] (new duplicated paths) on success.
    /// - [`Response::Error`] on failure.
    DuplicatePackedFiles(String, Vec<ContainerPath>),

    /// Extract one or more PackedFiles from a pack.
    /// First field is the pack key, then paths by data source, extraction path, whether to export tables as TSV.
    ///
    /// Response:
    /// - [`Response::StringVecPathBuf`] on success.
    /// - [`Response::Error`] on failure.
    ExtractPackedFiles(String, BTreeMap<DataSource, Vec<ContainerPath>>, PathBuf, bool),

    /// Rename one or more PackedFiles in a specific pack.
    /// First field is the pack key, second is a Vec with original and new ContainerPaths.
    ///
    /// Response:
    /// - [`Response::VecContainerPathContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    RenamePackedFiles(String, Vec<(ContainerPath, ContainerPath)>),

    /// Check if a folder exists in a specific open PackFile.
    /// First field is the pack key, second is the folder path.
    ///
    /// Response: [`Response::Bool`].
    FolderExists(String, String),

    /// Check if a PackedFile exists in a specific open PackFile.
    /// First field is the pack key, second is the file path.
    ///
    /// Response: [`Response::Bool`].
    PackedFileExists(String, String),

    //-----------------------------------------------------------------------//
    // Dependency Commands
    //-----------------------------------------------------------------------//

    /// Get the table names of all DB files in dependency PackFiles.
    ///
    /// Response: [`Response::VecString`].
    GetTableListFromDependencyPackFile,

    /// Get custom table names (start_pos_, twad_ prefixes) from the schema.
    ///
    /// Response:
    /// - [`Response::VecString`] on success.
    /// - [`Response::Error`] if no schema.
    GetCustomTableList,

    /// Get local art set IDs from campaign_character_arts_tables in a specific pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::HashSetString`].
    LocalArtSetIds(String),

    /// Get art set IDs from dependencies' campaign_character_arts_tables.
    ///
    /// Response: [`Response::HashSetString`].
    DependenciesArtSetIds,

    /// Get the version of a table from the dependency database.
    ///
    /// Response:
    /// - [`Response::I32`] on success.
    /// - [`Response::Error`] if not found or dependencies not loaded.
    GetTableVersionFromDependencyPackFile(String),

    /// Get the definition of a table from the dependency database.
    ///
    /// Response:
    /// - [`Response::Definition`] on success.
    /// - [`Response::Error`] if not found.
    GetTableDefinitionFromDependencyPackFile(String),

    /// Merge multiple compatible tables into one in a specific pack.
    /// First field is the pack key, then paths to merge, merged file path, delete source flag.
    ///
    /// Response:
    /// - [`Response::String`] (merged path) on success.
    /// - [`Response::Error`] on failure.
    MergeFiles(String, Vec<ContainerPath>, String, bool),

    /// Update a table to a newer version in a specific pack.
    /// First field is the pack key, second is the container path.
    ///
    /// Response:
    /// - [`Response::I32I32VecStringVecString`] (old_version, new_version, deleted_fields, added_fields) on success.
    /// - [`Response::Error`] on failure.
    UpdateTable(String, ContainerPath),

    //-----------------------------------------------------------------------//
    // Search Commands
    //-----------------------------------------------------------------------//

    /// Replace specific matches in a Global Search on a specific pack.
    /// First field is the pack key, then search config and match holders.
    ///
    /// Response:
    /// - [`Response::GlobalSearchVecRFileInfo`] on success.
    /// - [`Response::Error`] if no schema.
    GlobalSearchReplaceMatches(String, GlobalSearch, Vec<MatchHolder>),

    /// Replace all matches in a Global Search on a specific pack.
    /// First field is the pack key, second is the search config.
    ///
    /// Response:
    /// - [`Response::GlobalSearchVecRFileInfo`] on success.
    /// - [`Response::Error`] if no schema.
    GlobalSearchReplaceAll(String, GlobalSearch),

    /// Get reference data for columns in a definition from a specific pack.
    /// First field is the pack key, then table name, definition, force flag.
    ///
    /// Response: [`Response::HashMapI32TableReferences`].
    GetReferenceDataFromDefinition(String, String, Definition, bool),

    /// Get the list of PackFiles marked as dependencies of a specific pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::VecBoolString`].
    GetDependencyPackFilesList(String),

    /// Set the list of PackFiles marked as dependencies of a specific pack.
    /// First field is the pack key, second is the dependency list.
    ///
    /// Response: [`Response::Success`].
    SetDependencyPackFilesList(String, Vec<(bool, String)>),

    /// Get PackedFiles from all known sources (PackFile, GameFiles, ParentFiles).
    /// Requires: paths to get, whether to lowercase paths.
    ///
    /// Response: [`Response::HashMapDataSourceHashMapStringRFile`].
    GetRFilesFromAllSources(Vec<ContainerPath>, bool),

    //-----------------------------------------------------------------------//
    // Video Commands
    //-----------------------------------------------------------------------//

    /// Change the format of a ca_vp8 video PackedFile in a specific pack.
    /// First field is the pack key, then file path and format.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SetVideoFormat(String, String, SupportedFormats),

    //-----------------------------------------------------------------------//
    // Schema Commands
    //-----------------------------------------------------------------------//

    /// Save the provided schema to disk.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SaveSchema(Schema),

    /// Encode and clean the cache for the provided paths in a specific pack.
    /// First field is the pack key, second is the paths to clean.
    ///
    /// Response: [`Response::Success`].
    CleanCache(String, Vec<ContainerPath>),

    //-----------------------------------------------------------------------//
    // TSV Commands
    //-----------------------------------------------------------------------//

    /// Export a table as TSV from a specific pack.
    /// First field is the pack key, then internal path, destination path, data source.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    ExportTSV(String, String, PathBuf, DataSource),

    /// Import a TSV as a table into a specific pack.
    /// First field is the pack key, then internal path, source TSV path.
    ///
    /// Response:
    /// - [`Response::RFileDecoded`] on success.
    /// - [`Response::Error`] on failure.
    ImportTSV(String, String, PathBuf),

    //-----------------------------------------------------------------------//
    // External Program Commands
    //-----------------------------------------------------------------------//

    /// Open the folder containing a specific open PackFile in the file manager.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] if pack doesn't exist on disk.
    OpenContainingFolder(String),

    /// Open a PackedFile in an external program.
    /// First field is the pack key, then data source and container path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] (extracted path) on success.
    /// - [`Response::Error`] on failure.
    OpenPackedFileInExternalProgram(String, DataSource, ContainerPath),

    /// Save a PackedFile from an external program to a specific pack.
    /// First field is the pack key, then internal path, external file path.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SavePackedFileFromExternalView(String, String, PathBuf),

    //-----------------------------------------------------------------------//
    // Program Update Commands
    //-----------------------------------------------------------------------//

    /// Update the program to the latest version available.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateMainProgram,

    /// Trigger an autosave to a backup for a specific pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::Success`].
    TriggerBackupAutosave(String),

    //-----------------------------------------------------------------------//
    // Diagnostics Commands
    //-----------------------------------------------------------------------//

    /// Trigger a full diagnostics check over all open Packs.
    /// First field is ignored diagnostics, then check AK-only references.
    ///
    /// Response: [`Response::Diagnostics`].
    DiagnosticsCheck(Vec<String>, bool),

    /// Trigger a partial diagnostics update over all open packs.
    /// First field is existing diagnostics, then paths to check, check AK-only references.
    ///
    /// Response: [`Response::Diagnostics`].
    DiagnosticsUpdate(Diagnostics, Vec<ContainerPath>, bool),

    //-----------------------------------------------------------------------//
    // Pack Settings Commands
    //-----------------------------------------------------------------------//

    /// Get the settings of a specific open PackFile.
    /// The field is the pack key.
    ///
    /// Response: [`Response::PackSettings`].
    GetPackSettings(String),

    /// Set the settings of a specific open PackFile.
    /// First field is the pack key, second is the settings.
    ///
    /// Response: [`Response::Success`].
    SetPackSettings(String, PackSettings),

    //-----------------------------------------------------------------------//
    // Debug Commands
    //-----------------------------------------------------------------------//

    /// Export missing table definitions from a specific pack to a file (for debugging).
    /// The field is the pack key.
    ///
    /// Response: [`Response::Success`].
    GetMissingDefinitions(String),

    //-----------------------------------------------------------------------//
    // Dependencies Commands
    //-----------------------------------------------------------------------//

    /// Rebuild the dependencies.
    /// Pass true to rebuild all dependencies, false for mod-specific only.
    ///
    /// Response:
    /// - [`Response::DependenciesInfo`] on success.
    /// - [`Response::Error`] if no schema.
    RebuildDependencies(bool),

    //-----------------------------------------------------------------------//
    // Cascade Edition Commands
    //-----------------------------------------------------------------------//

    /// Trigger a cascade edition on all referenced data in a specific pack.
    /// First field is the pack key, then table name, definition, list of (field, old_value, new_value).
    ///
    /// Response: [`Response::VecContainerPathVecRFileInfo`].
    CascadeEdition(String, String, Definition, Vec<(Field, String, String)>),

    //-----------------------------------------------------------------------//
    // Navigation Commands
    //-----------------------------------------------------------------------//

    /// Go to the definition of a reference in a specific pack.
    /// First field is the pack key, then table, column, values to search.
    ///
    /// Response:
    /// - [`Response::DataSourceStringUsizeUsize`] on success.
    /// - [`Response::Error`] if not found.
    GoToDefinition(String, String, String, Vec<String>),

    /// Get the source data of a loc key from a specific pack.
    /// First field is the pack key, second is the loc key.
    ///
    /// Response: [`Response::OptionStringStringVecString`].
    GetSourceDataFromLocKey(String, String),

    /// Go to a loc key's location in a specific pack.
    /// First field is the pack key, second is the loc key to search.
    ///
    /// Response:
    /// - [`Response::DataSourceStringUsizeUsize`] on success.
    /// - [`Response::Error`] if not found.
    GoToLoc(String, String),

    /// Find all references to a value in a specific pack.
    /// First field is the pack key, then map of table -> columns to search, value to search.
    ///
    /// Response: [`Response::VecDataSourceStringStringUsizeUsize`].
    SearchReferences(String, HashMap<String, Vec<String>>, String),

    /// Get the name of a specific open PackFile.
    /// The field is the pack key.
    ///
    /// Response: [`Response::String`].
    GetPackFileName(String),

    /// Get the raw binary data of a PackedFile from a specific pack.
    /// First field is the pack key, second is the file path.
    ///
    /// Response:
    /// - [`Response::VecU8`] on success.
    /// - [`Response::Error`] on failure.
    GetPackedFileRawData(String, String),

    /// Import files from dependencies into a specific open PackFile.
    /// First field is the pack key, second is the paths by data source.
    ///
    /// Response:
    /// - [`Response::VecContainerPathVecString`] (added paths, failed paths).
    /// - [`Response::Error`] on failure.
    ImportDependenciesToOpenPackFile(String, BTreeMap<DataSource, Vec<ContainerPath>>),

    /// Save PackedFiles to a specific PackFile and optionally optimize.
    /// First field is the pack key, then files to save, whether to optimize.
    ///
    /// Response:
    /// - [`Response::VecContainerPathVecContainerPath`] (added paths, deleted paths) on success.
    /// - [`Response::Error`] on failure.
    SavePackedFilesToPackFileAndClean(String, Vec<RFile>, bool),

    /// Get all file names under a path in all dependencies.
    ///
    /// Response: [`Response::HashMapDataSourceHashSetContainerPath`].
    GetPackedFilesNamesStartingWitPathFromAllSources(ContainerPath),

    //-----------------------------------------------------------------------//
    // Notes Commands
    //-----------------------------------------------------------------------//

    /// Get all notes under a path in a specific pack.
    /// First field is the pack key, second is the path.
    ///
    /// Response: [`Response::VecNote`].
    NotesForPath(String, String),

    /// Add a note to a specific pack.
    /// First field is the pack key, second is the note.
    ///
    /// Response: [`Response::Note`].
    AddNote(String, Note),

    /// Delete a note from a specific pack.
    /// First field is the pack key, then path and note ID.
    ///
    /// Response: [`Response::Success`].
    DeleteNote(String, String, u64),

    //-----------------------------------------------------------------------//
    // Schema Patch Commands
    //-----------------------------------------------------------------------//

    /// Save local schema patches.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SaveLocalSchemaPatch(HashMap<String, DefinitionPatch>),

    /// Remove local schema patches for a table.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    RemoveLocalSchemaPatchesForTable(String),

    /// Remove local schema patches for a specific field in a table.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    RemoveLocalSchemaPatchesForTableAndField(String, String),

    /// Import a schema patch into the local schema patches.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    ImportSchemaPatch(HashMap<String, DefinitionPatch>),

    //-----------------------------------------------------------------------//
    // Loc Generation Commands
    //-----------------------------------------------------------------------//

    /// Generate all missing loc entries for a specific open PackFile.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    GenerateMissingLocData(String),

    //-----------------------------------------------------------------------//
    // Lua Autogen Commands
    //-----------------------------------------------------------------------//

    /// Check for updates on the tw_autogen repository.
    ///
    /// Response:
    /// - [`Response::APIResponseGit`] on success.
    /// - [`Response::Error`] on failure.
    CheckLuaAutogenUpdates,

    /// Update the tw_autogen repository.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateLuaAutogen,

    //-----------------------------------------------------------------------//
    // MyMod Commands
    //-----------------------------------------------------------------------//

    /// Initialize a MyMod folder.
    /// Requires: mod name, game key, sublime support, vscode support, git support (gitignore content).
    ///
    /// Response:
    /// - [`Response::PathBuf`] (path to the new pack) on success.
    /// - [`Response::Error`] on failure.
    InitializeMyModFolder(String, String, bool, bool, Option<String>),

    /// Live export a specific PackFile to the game folder.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    LiveExport(String),

    /// Set the operational mode for a specific pack.
    /// First field is the pack key, second is the new operational mode.
    ///
    /// Response: [`Response::Success`].
    SetPackOperationalMode(String, OperationalMode),

    /// Get the operational mode for a specific pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::OperationalMode`].
    GetPackOperationalMode(String),

    //-----------------------------------------------------------------------//
    // Map Packing Commands
    //-----------------------------------------------------------------------//

    /// Pack map tiles into a specific PackFile.
    /// First field is the pack key, then tile map paths, list of (tile path, name).
    ///
    /// Response:
    /// - [`Response::VecContainerPathVecContainerPath`] (added paths, deleted paths) on success.
    /// - [`Response::Error`] on failure.
    PackMap(String, Vec<PathBuf>, Vec<(PathBuf, String)>),

    //-----------------------------------------------------------------------//
    // Diagnostics Ignore Commands
    //-----------------------------------------------------------------------//

    /// Add a line to a specific pack's ignored diagnostics.
    /// First field is the pack key, second is the diagnostic line.
    ///
    /// Response: [`Response::Success`].
    AddLineToPackIgnoredDiagnostics(String, String),

    //-----------------------------------------------------------------------//
    // Empire/Napoleon AK Commands
    //-----------------------------------------------------------------------//

    /// Check for updates on the old AK files repository.
    ///
    /// Response:
    /// - [`Response::APIResponseGit`] on success.
    /// - [`Response::Error`] on failure.
    CheckEmpireAndNapoleonAKUpdates,

    /// Update the old AK files repository.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateEmpireAndNapoleonAK,

    //-----------------------------------------------------------------------//
    // Translation Commands
    //-----------------------------------------------------------------------//

    /// Get pack translation data for a language from a specific pack.
    /// First field is the pack key, second is the language.
    ///
    /// Response:
    /// - [`Response::PackTranslation`] on success.
    /// - [`Response::Error`] on failure.
    GetPackTranslation(String, String),

    /// Check for translation updates.
    ///
    /// Response:
    /// - [`Response::APIResponseGit`] on success.
    /// - [`Response::Error`] on failure.
    CheckTranslationsUpdates,

    /// Update the translations repository.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    UpdateTranslations,

    //-----------------------------------------------------------------------//
    // Starpos Commands
    //-----------------------------------------------------------------------//

    /// Build starpos (pre-processing step) for a specific pack.
    /// First field is the pack key, then campaign ID, process HLP/SPD data.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    BuildStarpos(String, String, bool),

    /// Build starpos (post-processing step) for a specific pack.
    /// First field is the pack key, then campaign ID, process HLP/SPD data.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    BuildStarposPost(String, String, bool),

    /// Clean up starpos temporary files for a specific pack.
    /// First field is the pack key, then campaign ID, process HLP/SPD data.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    BuildStarposCleanup(String, String, bool),

    /// Get campaign IDs for starpos building from a specific pack.
    /// The field is the pack key.
    ///
    /// Response: [`Response::HashSetString`].
    BuildStarposGetCampaingIds(String),

    /// Check if victory conditions file exists in a specific pack (required for some games).
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::Success`] if exists or not needed.
    /// - [`Response::Error`] if missing.
    BuildStarposCheckVictoryConditions(String),

    //-----------------------------------------------------------------------//
    // Animation Commands
    //-----------------------------------------------------------------------//

    /// Update animation IDs with offset in a specific pack.
    /// First field is the pack key, then starting ID, offset.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    UpdateAnimIds(String, i32, i32),

    /// Get animation paths by skeleton name.
    ///
    /// Response: [`Response::HashSetString`].
    GetAnimPathsBySkeletonName(String),

    //-----------------------------------------------------------------------//
    // Table Commands
    //-----------------------------------------------------------------------//

    /// Get tables from dependencies by table name.
    ///
    /// Response:
    /// - [`Response::VecRFile`] on success.
    /// - [`Response::Error`] on failure.
    GetTablesFromDependencies(String),

    /// Get table paths by table name from a specific PackFile.
    /// First field is the pack key, second is the table name.
    ///
    /// Response: [`Response::VecString`].
    GetTablesByTableName(String, String),

    /// Add keys to the key_deletes table in a specific pack.
    /// First field is the pack key, then table file name, key table name, keys to add.
    ///
    /// Response: [`Response::OptionContainerPath`].
    AddKeysToKeyDeletes(String, String, String, HashSet<String>),

    //-----------------------------------------------------------------------//
    // 3D Export Commands
    //-----------------------------------------------------------------------//

    /// Export a RigidModel to glTF format.
    /// Requires: RigidModel, output path.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    ExportRigidToGltf(RigidModel, String),

    //-----------------------------------------------------------------------//
    // Settings Getter Commands
    //-----------------------------------------------------------------------//

    /// Get a boolean setting value.
    ///
    /// Response: [`Response::Bool`].
    SettingsGetBool(String),

    /// Get an i32 setting value.
    ///
    /// Response: [`Response::I32`].
    SettingsGetI32(String),

    /// Get an f32 setting value.
    ///
    /// Response: [`Response::F32`].
    SettingsGetF32(String),

    /// Get a string setting value.
    ///
    /// Response: [`Response::String`].
    SettingsGetString(String),

    /// Get a PathBuf setting value.
    ///
    /// Response: [`Response::PathBuf`].
    SettingsGetPathBuf(String),

    /// Get a `Vec<String>` setting value.
    ///
    /// Response: [`Response::VecString`].
    SettingsGetVecString(String),

    /// Get raw data setting value.
    ///
    /// Response: [`Response::VecU8`].
    SettingsGetVecRaw(String),

    /// Get all settings at once (for batch loading).
    ///
    /// This is much more efficient than calling individual SettingsGet* commands
    /// when you need multiple settings, as it requires only one IPC round-trip.
    ///
    /// Response: [`Response::SettingsAll`].
    SettingsGetAll,

    //-----------------------------------------------------------------------//
    // Settings Setter Commands
    //-----------------------------------------------------------------------//

    /// Set a boolean setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetBool(String, bool),

    /// Set an i32 setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetI32(String, i32),

    /// Set an f32 setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetF32(String, f32),

    /// Set a string setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetString(String, String),

    /// Set a PathBuf setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetPathBuf(String, PathBuf),

    /// Set a `Vec<String>` setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetVecString(String, Vec<String>),

    /// Set raw data setting value.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsSetVecRaw(String, Vec<u8>),

    //-----------------------------------------------------------------------//
    // Path Commands
    //-----------------------------------------------------------------------//

    /// Get the config path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    ConfigPath,

    /// Get the Assembly Kit path for the current game.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    AssemblyKitPath,

    /// Get the backup autosave path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    BackupAutosavePath,

    /// Get the old AK data path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    OldAkDataPath,

    /// Get the schemas path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    SchemasPath,

    /// Get the table profiles path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    TableProfilesPath,

    /// Get the translations local path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    TranslationsLocalPath,

    /// Get the dependencies cache path.
    ///
    /// Response:
    /// - [`Response::PathBuf`] on success.
    /// - [`Response::Error`] on failure.
    DependenciesCachePath,

    /// Clear a config path.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    SettingsClearPath(PathBuf),

    //-----------------------------------------------------------------------//
    // Settings Backup Commands
    //-----------------------------------------------------------------------//

    /// Backup the current settings to memory.
    ///
    /// Response: [`Response::Success`].
    BackupSettings,

    /// Clear settings and reset to defaults.
    ///
    /// Response:
    /// - [`Response::Success`] on success.
    /// - [`Response::Error`] on failure.
    ClearSettings,

    /// Restore settings from the backup.
    ///
    /// Response: [`Response::Success`].
    RestoreBackupSettings,

    /// Get the optimizer options.
    ///
    /// Response: [`Response::OptimizerOptions`].
    OptimizerOptions,

    //-----------------------------------------------------------------------//
    // Schema Query Commands
    //-----------------------------------------------------------------------//

    /// Check if a schema is loaded.
    ///
    /// Response: [`Response::Bool`].
    IsSchemaLoaded,

    /// Get all definitions for a table name.
    ///
    /// Response:
    /// - [`Response::VecDefinition`] on success.
    /// - [`Response::Error`] if no schema.
    DefinitionsByTableName(String),

    /// Get columns that reference a table's definition.
    ///
    /// Response:
    /// - [`Response::HashMapStringHashMapStringVecString`] on success.
    /// - [`Response::Error`] if no schema.
    ReferencingColumnsForDefinition(String, Definition),

    /// Get the current schema.
    ///
    /// Response:
    /// - [`Response::Schema`] on success.
    /// - [`Response::Error`] if no schema.
    Schema,

    /// Get a specific definition by table name and version.
    ///
    /// Response:
    /// - [`Response::Definition`] on success.
    /// - [`Response::Error`] if not found or no schema.
    DefinitionByTableNameAndVersion(String, i32),

    /// Delete a definition by table name and version.
    ///
    /// Response: [`Response::Success`].
    DeleteDefinition(String, i32),

    /// Get the processed fields from a definition (bitwise expansion, enum conversion, colour merging applied).
    ///
    /// Response: [`Response::VecField`].
    FieldsProcessed(Definition),
}

/// This enum defines the responses (messages) you can send to the UI thread as result of a command.
///
/// Each response is named after the types of the items it carries, making them self-documenting.
/// For example, `VecString` returns a `Vec<String>`, and `DBRFileInfo` returns a `(DB, RFileInfo)` tuple.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Generic response for situations of success where no data needs to be returned.
    Success,

    /// Generic response for situations that returned an error, containing the error message.
    Error(String),

    /// Response sent by the server immediately after a WebSocket connection is established.
    /// Contains the session ID that the client is connected to.
    SessionConnected(u64),

    #[allow(dead_code)]BmdRFileInfo(Box<Bmd>, RFileInfo),
    AnimFragmentBattleRFileInfo(AnimFragmentBattle, RFileInfo),
    AnimPackRFileInfo(Vec<RFileInfo>, RFileInfo),
    AnimsTableRFileInfo(AnimsTable, RFileInfo),
    APIResponse(APIResponse),
    APIResponseGit(GitResponse),
    AtlasRFileInfo(Atlas, RFileInfo),
    AudioRFileInfo(Audio, RFileInfo),
    Bool(bool),
    CompressionFormat(CompressionFormat),
    CompressionFormatDependenciesInfo(CompressionFormat, Option<DependenciesInfo>),
    ContainerInfo(ContainerInfo),
    ContainerInfoVecRFileInfo((ContainerInfo, Vec<RFileInfo>)),
    StringContainerInfo(String, ContainerInfo),
    DataSourceStringUsizeUsize(DataSource, String, usize, usize),
    DBRFileInfo(DB, RFileInfo),
    Definition(Definition),
    DependenciesInfo(DependenciesInfo),
    Diagnostics(Diagnostics),
    ESFRFileInfo(ESF, RFileInfo),
    F32(f32),
    GlobalSearchVecRFileInfo(Box<GlobalSearch>, Vec<RFileInfo>),
    GroupFormationsRFileInfo(GroupFormations, RFileInfo),
    HashMapDataSourceHashMapStringRFile(HashMap<DataSource, HashMap<String, RFile>>),
    HashMapDataSourceHashSetContainerPath(HashMap<DataSource, HashSet<ContainerPath>>),
    HashMapI32TableReferences(HashMap<i32, TableReferences>),
    HashMapStringHashMapStringVecString(HashMap<String, HashMap<String, Vec<String>>>),
    HashSetString(HashSet<String>),
    HashSetStringHashSetString(HashSet<String>, HashSet<String>),
    I32(i32),
    I32I32(i32, i32),
    I32I32VecStringVecString(i32, i32, Vec<String>, Vec<String>),
    ImageRFileInfo(Image, RFileInfo),
    LocRFileInfo(Loc, RFileInfo),
    MatchedCombatRFileInfo(MatchedCombat, RFileInfo),
    Note(Note),
    OperationalMode(OperationalMode),
    OptimizerOptions(OptimizerOptions),
    OptionContainerPath(Option<ContainerPath>),
    OptionRFileInfo(Option<RFileInfo>),
    OptionStringStringVecString(Option<(String, String, Vec<String>)>),
    PackSettings(PackSettings),
    PackTranslation(PackTranslation),
    PathBuf(PathBuf),
    PortraitSettingsRFileInfo(PortraitSettings, RFileInfo),
    RFileDecoded(RFileDecoded),
    RigidModelRFileInfo(RigidModel, RFileInfo),
    Schema(Schema),
    String(String),
    StringVecContainerPath(String, Vec<ContainerPath>),
    StringVecPathBuf(String, Vec<PathBuf>),
    Text(Text),
    TextRFileInfo(Text, RFileInfo),
    UICRFileInfo(UIC, RFileInfo),
    UnitVariantRFileInfo(UnitVariant, RFileInfo),
    Unknown,
    VecBoolString(Vec<(bool, String)>),
    VecContainerPath(Vec<ContainerPath>),
    VecContainerPathContainerPath(Vec<(ContainerPath, ContainerPath)>),
    VecContainerPathOptionString(Vec<ContainerPath>, Option<String>),
    VecContainerPathVecContainerPath(Vec<ContainerPath>, Vec<ContainerPath>),
    VecContainerPathBTreeMapStringVecContainerPath(Vec<ContainerPath>, BTreeMap<String, Vec<ContainerPath>>),
    VecContainerPathVecContainerPathString(Vec<ContainerPath>, Vec<ContainerPath>, String),
    VecContainerPathVecRFileInfo(Vec<ContainerPath>, Vec<RFileInfo>),
    VecContainerPathVecString(Vec<ContainerPath>, Vec<String>),
    VecDataSourceStringStringUsizeUsize(Vec<(DataSource, String, String, usize, usize)>),
    VecDefinition(Vec<Definition>),
    VecField(Vec<Field>),
    VecNote(Vec<Note>),
    VecRFile(Vec<RFile>),
    VecRFileInfo(Vec<RFileInfo>),
    VecString(Vec<String>),
    VecStringContainerInfo(Vec<(String, ContainerInfo)>),
    VecU8(Vec<u8>),
    VideoInfoRFileInfo(VideoInfo, RFileInfo),
    VMDRFileInfo(Text, RFileInfo),
    WSModelRFileInfo(Text, RFileInfo),

    /// All settings in one response (for batch loading).
    SettingsAll(SettingsSnapshot),
}
