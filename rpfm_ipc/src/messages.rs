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

    /// Closes the open Pack.
    ///
    /// Response: None.
    ClosePack,

    /// Close the extra Pack with the provided path.
    ///
    /// Response: None.
    ClosePackExtra(PathBuf),

    /// Clean the open Pack from corrupted/undecoded files and try to save it to disk.
    ///
    /// Only use this command if your Pack is not save-able otherwise.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    CleanAndSavePackAs(PathBuf),

    /// Creates a new empty Pack.
    ///
    /// Response: None.
    NewPack,

    /// Save the currently open Pack to disk.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    SavePack,

    /// Save the currently open Pack to a new path.
    ///
    /// Response:
    /// - [`Response::ContainerInfo`] on success.
    /// - [`Response::Error`] on failure.
    SavePackAs(PathBuf),

    /// Get the data used to build the `TreeView`].
    ///
    /// Response:
    /// - [`Response::ContainerInfoVecRFileInfo`].
    GetPackFileDataForTreeView,

    /// Get the data for an extra Pack's TreeView. Requires the PathBuf of the PackFile.
    ///
    /// Response:
    /// - [`Response::ContainerInfoVecRFileInfo`] on success.
    /// - [`Response::Error`] on failure.
    GetPackFileExtraDataForTreeView(PathBuf),

    /// Open one or more `PackFiles` and merge them. Requires the paths of the `PackFiles`].
    ///
    /// Response: [`Response::ContainerInfo`] on success, [`Response::Error`] on failure.
    OpenPackFiles(Vec<PathBuf>),

    /// Open an extra Pack for "Add from PackFile" feature. Requires the path of the Pack.
    ///
    /// Response: [`Response::ContainerInfo`] on success, [`Response::Error`] on failure.
    OpenPackExtra(PathBuf),

    /// Open all the CA PackFiles for the selected game as one merged PackFile.
    ///
    /// Response: [`Response::ContainerInfo`] on success, [`Response::Error`] on failure.
    LoadAllCAPackFiles,

    /// Get the `RFileInfo` of one or more `PackedFiles`].
    ///
    /// Response: [`Response::VecRFileInfo`].
    GetPackedFilesInfo(Vec<String>),

    /// Perform a `Global Search`]. Requires the search configuration.
    ///
    /// Response: [`Response::GlobalSearchVecRFileInfo`] on success, [`Response::Error`] if no schema.
    GlobalSearch(GlobalSearch),

    /// Change the `Game Selected`]. Contains the game key and whether to rebuild dependencies.
    ///
    /// Response: [`Response::CompressionFormatDependenciesInfo`] on success, [`Response::Error`] if game not supported.
    SetGameSelected(String, bool),

    /// Change the `Type` of the currently open Pack.
    ///
    /// Response: None.
    SetPackFileType(PFHFileType),

    /// Generate the dependencies cache for the selected game.
    ///
    /// Response: [`Response::DependenciesInfo`] on success, [`Response::Error`] on failure.
    GenerateDependenciesCache,

    /// Update the currently loaded Schema with data from the game's Assembly Kit.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateCurrentSchemaFromAssKit,

    /// Trigger an optimization pass over the currently open Pack.
    ///
    /// Response: [`Response::HashSetStringHashSetString` (deleted paths, added paths) on success, [`Response::Error`] on failure.
    OptimizePackFile(OptimizerOptions),

    /// Patch the SiegeAI of a Siege Map for Warhammer games.
    ///
    /// Response: [`Response::StringVecContainerPath`] on success, [`Response::Error`] on failure.
    PatchSiegeAI,

    /// Change the `Index Includes Timestamp` flag in the currently open Pack.
    ///
    /// Response: None.
    ChangeIndexIncludesTimestamp(bool),

    /// Change the compression format of the currently open Pack.
    ///
    /// Response: [`Response::CompressionFormat` (the actual format set, may differ if unsupported).
    ChangeCompressionFormat(CompressionFormat),

    /// Get the current path of the currently open Pack.
    ///
    /// Response: [`Response::PathBuf`].
    GetPackFilePath,

    /// Get the info of a single `PackedFile`].
    ///
    /// Response: [`Response::OptionRFileInfo`].
    GetRFileInfo(String),

    //-----------------------------------------------------------------------//
    // Update Commands
    //-----------------------------------------------------------------------//

    /// Check if there is an RPFM update available.
    ///
    /// Response: [`Response::APIResponse`] on success, [`Response::Error`] on failure.
    CheckUpdates,

    /// Check if there is a Schema update available.
    ///
    /// Response: [`Response::APIResponseGit`] on success, [`Response::Error`] on failure.
    CheckSchemaUpdates,

    /// Update the schemas from the remote repository.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateSchemas,

    /// Check if there is a Dependency Database loaded in memory.
    /// Pass true to ensure dependencies were built with the AssKit.
    ///
    /// Response: [`Response::Bool`].
    IsThereADependencyDatabase(bool),

    //-----------------------------------------------------------------------//
    // PackedFile Operations
    //-----------------------------------------------------------------------//

    /// Create a new `PackedFile` inside the currently open Pack.
    /// Requires the path and the `NewFile` with the new PackedFile's info.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    NewPackedFile(String, NewFile),

    /// Add one or more Files to the currently open Pack.
    /// Requires: source filesystem paths, destination container paths, optional paths to ignore.
    ///
    /// Response: [`Response::VecContainerPath` (added paths), then `Success` or [`Response::Error`].
    AddPackedFiles(Vec<PathBuf>, Vec<ContainerPath>, Option<Vec<PathBuf>>),

    /// Decode a PackedFile to be shown on the UI.
    /// Contains the path of the file and its data source.
    ///
    /// Response: File-type specific response (e.g., `DBRFileInfo`, `LocRFileInfo`, `TextRFileInfo`,
    /// `ImageRFileInfo`, `RigidModelRFileInfo`, etc.), `Text` for pack notes, `Unknown` for
    /// unsupported types, or [`Response::Error`] on failure.
    DecodePackedFile(String, DataSource),

    /// Save an edited `PackedFile` back to the Pack.
    ///
    /// Response: [`Response::Success`].
    SavePackedFileFromView(String, RFileDecoded),

    /// Add PackedFiles from one PackFile into another.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] if extra PackFile not found.
    AddPackedFilesFromPackFile((PathBuf, Vec<ContainerPath>)),

    /// Add PackedFiles from the main PackFile to an AnimPack.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] on failure.
    AddPackedFilesFromPackFileToAnimpack(String, Vec<ContainerPath>),

    /// Add PackedFiles from an AnimPack to the main PackFile.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] on failure.
    AddPackedFilesFromAnimpack(DataSource, String, Vec<ContainerPath>),

    /// Delete PackedFiles from an AnimPack.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    DeleteFromAnimpack((String, Vec<ContainerPath>)),

    /// Delete one or more PackedFiles from a PackFile.
    ///
    /// Response: [`Response::VecContainerPath` (deleted paths).
    DeletePackedFiles(Vec<ContainerPath>),

    /// Extract one or more PackedFiles from a PackFile.
    /// Contains: paths by data source, extraction path, whether to export tables as TSV.
    ///
    /// Response: [`Response::StringVecPathBuf`] on success, [`Response::Error`] on failure.
    ExtractPackedFiles(BTreeMap<DataSource, Vec<ContainerPath>>, PathBuf, bool),

    /// Rename one or more PackedFiles in a PackFile.
    /// Contains a Vec with original and new ContainerPaths.
    ///
    /// Response: [`Response::VecContainerPathContainerPath`] on success, [`Response::Error`] on failure.
    RenamePackedFiles(Vec<(ContainerPath, ContainerPath)>),

    /// Check if a folder exists in the currently open PackFile.
    ///
    /// Response: [`Response::Bool`].
    FolderExists(String),

    /// Check if a PackedFile exists in the currently open PackFile.
    ///
    /// Response: [`Response::Bool`].
    PackedFileExists(String),

    //-----------------------------------------------------------------------//
    // Dependency Commands
    //-----------------------------------------------------------------------//

    /// Get the table names of all DB files in dependency PackFiles.
    ///
    /// Response: [`Response::VecString`].
    GetTableListFromDependencyPackFile,

    /// Get custom table names (start_pos_, twad_ prefixes) from the schema.
    ///
    /// Response: [`Response::VecString`] on success, [`Response::Error`] if no schema.
    GetCustomTableList,

    /// Get local art set IDs from campaign_character_arts_tables.
    ///
    /// Response: [`Response::HashSetString`].
    LocalArtSetIds,

    /// Get art set IDs from dependencies' campaign_character_arts_tables.
    ///
    /// Response: [`Response::HashSetString`].
    DependenciesArtSetIds,

    /// Get the version of a table from the dependency database.
    ///
    /// Response: [`Response::I32`] on success, [`Response::Error`] if not found or dependencies not loaded.
    GetTableVersionFromDependencyPackFile(String),

    /// Get the definition of a table from the dependency database.
    ///
    /// Response: [`Response::Definition`] on success, [`Response::Error`] if not found.
    GetTableDefinitionFromDependencyPackFile(String),

    /// Merge multiple compatible tables into one.
    /// - `Vec<ContainerPath>`: Paths to merge.
    /// - `String`: Path of the merged file.
    /// - `bool`: Delete source files after merging.
    ///
    /// Response: [`Response::String` (merged path) on success, [`Response::Error`] on failure.
    MergeFiles(Vec<ContainerPath>, String, bool),

    /// Update a table to a newer version.
    ///
    /// Response: [`Response::I32I32VecStringVecString` (old_version, new_version, deleted_fields, added_fields) on success, [`Response::Error`] on failure.
    UpdateTable(ContainerPath),

    //-----------------------------------------------------------------------//
    // Search Commands
    //-----------------------------------------------------------------------//

    /// Replace specific matches in a Global Search.
    ///
    /// Response: [`Response::GlobalSearchVecRFileInfo`] on success, [`Response::Error`] if no schema.
    GlobalSearchReplaceMatches(GlobalSearch, Vec<MatchHolder>),

    /// Replace all matches in a Global Search.
    ///
    /// Response: [`Response::GlobalSearchVecRFileInfo`] on success, [`Response::Error`] if no schema.
    GlobalSearchReplaceAll(GlobalSearch),

    /// Get reference data for columns in a definition.
    /// Requires: table name, definition, force local reference regeneration.
    ///
    /// Response: [`Response::HashMapI32TableReferences`].
    GetReferenceDataFromDefinition(String, Definition, bool),

    /// Get the list of PackFiles marked as dependencies of the current PackFile.
    ///
    /// Response: [`Response::VecBoolString`].
    GetDependencyPackFilesList,

    /// Set the list of PackFiles marked as dependencies of the current PackFile.
    ///
    /// Response: None.
    SetDependencyPackFilesList(Vec<(bool, String)>),

    /// Get PackedFiles from all known sources (PackFile, GameFiles, ParentFiles).
    /// Requires: paths to get, whether to lowercase paths.
    ///
    /// Response: [`Response::HashMapDataSourceHashMapStringRFile`].
    GetRFilesFromAllSources(Vec<ContainerPath>, bool),

    //-----------------------------------------------------------------------//
    // Video Commands
    //-----------------------------------------------------------------------//

    /// Change the format of a ca_vp8 video PackedFile.
    ///
    /// Response: None on success, [`Response::Error`] on failure.
    SetVideoFormat(String, SupportedFormats),

    //-----------------------------------------------------------------------//
    // Schema Commands
    //-----------------------------------------------------------------------//

    /// Save the provided schema to disk.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SaveSchema(Schema),

    /// Encode and clean the cache for the provided paths.
    ///
    /// Response: None.
    CleanCache(Vec<ContainerPath>),

    //-----------------------------------------------------------------------//
    // TSV Commands
    //-----------------------------------------------------------------------//

    /// Export a table as TSV. Requires: internal path, destination path, data source.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    ExportTSV(String, PathBuf, DataSource),

    /// Import a TSV as a table. Requires: internal path, source TSV path.
    ///
    /// Response: [`Response::RFileDecoded`] on success, [`Response::Error`] on failure.
    ImportTSV(String, PathBuf),

    //-----------------------------------------------------------------------//
    // External Program Commands
    //-----------------------------------------------------------------------//

    /// Open the folder containing the currently open PackFile in the file manager.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] if pack doesn't exist on disk.
    OpenContainingFolder,

    /// Open a PackedFile in an external program.
    ///
    /// Response: [`Response::PathBuf` (extracted path) on success, [`Response::Error`] on failure.
    OpenPackedFileInExternalProgram(DataSource, ContainerPath),

    /// Save a PackedFile from an external program. Requires: internal path, external file path.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SavePackedFileFromExternalView(String, PathBuf),

    //-----------------------------------------------------------------------//
    // Program Update Commands
    //-----------------------------------------------------------------------//

    /// Update the program to the latest version available.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateMainProgram,

    /// Trigger an autosave to a backup.
    ///
    /// Response: None.
    TriggerBackupAutosave,

    //-----------------------------------------------------------------------//
    // Diagnostics Commands
    //-----------------------------------------------------------------------//

    /// Trigger a full diagnostics check over the open PackFile.
    /// Requires: ignored diagnostics, check AK-only references.
    ///
    /// Response: [`Response::Diagnostics`].
    DiagnosticsCheck(Vec<String>, bool),

    /// Trigger a partial diagnostics update.
    /// Requires: existing diagnostics, paths to check, check AK-only references.
    ///
    /// Response: [`Response::Diagnostics`].
    DiagnosticsUpdate(Diagnostics, Vec<ContainerPath>, bool),

    //-----------------------------------------------------------------------//
    // Pack Settings Commands
    //-----------------------------------------------------------------------//

    /// Get the settings of the currently open PackFile.
    ///
    /// Response: [`Response::PackSettings`].
    GetPackSettings,

    /// Set the settings of the currently open PackFile.
    ///
    /// Response: None.
    SetPackSettings(PackSettings),

    //-----------------------------------------------------------------------//
    // Debug Commands
    //-----------------------------------------------------------------------//

    /// Export missing table definitions to a file (for debugging).
    ///
    /// Response: None.
    GetMissingDefinitions,

    //-----------------------------------------------------------------------//
    // Dependencies Commands
    //-----------------------------------------------------------------------//

    /// Rebuild the dependencies of a PackFile.
    /// Pass true to rebuild all dependencies, false for mod-specific only.
    ///
    /// Response: [`Response::DependenciesInfo`] on success, [`Response::Error`] if no schema.
    RebuildDependencies(bool),

    //-----------------------------------------------------------------------//
    // Cascade Edition Commands
    //-----------------------------------------------------------------------//

    /// Trigger a cascade edition on all referenced data.
    /// Requires: table name, definition, list of (field, old_value, new_value).
    ///
    /// Response: [`Response::VecContainerPathVecRFileInfo`].
    CascadeEdition(String, Definition, Vec<(Field, String, String)>),

    //-----------------------------------------------------------------------//
    // Navigation Commands
    //-----------------------------------------------------------------------//

    /// Go to the definition of a reference. Contains: table, column, values to search.
    ///
    /// Response: [`Response::DataSourceStringUsizeUsize`] on success, [`Response::Error`] if not found.
    GoToDefinition(String, String, Vec<String>),

    /// Get the source data of a loc key.
    ///
    /// Response: [`Response::OptionStringStringVecString`].
    GetSourceDataFromLocKey(String),

    /// Go to a loc key's location. Contains the loc key to search.
    ///
    /// Response: [`Response::DataSourceStringUsizeUsize`] on success, [`Response::Error`] if not found.
    GoToLoc(String),

    /// Find all references to a value.
    /// Contains: map of table -> columns to search, value to search.
    ///
    /// Response: [`Response::VecDataSourceStringStringUsizeUsize`].
    SearchReferences(HashMap<String, Vec<String>>, String),

    /// Get the name of the currently open PackFile.
    ///
    /// Response: [`Response::String`].
    GetPackFileName,

    /// Get the raw binary data of a PackedFile.
    ///
    /// Response: [`Response::VecU8`] on success, [`Response::Error`] on failure.
    GetPackedFileRawData(String),

    /// Import files from dependencies into the open PackFile.
    ///
    /// Response: [`Response::VecContainerPath` (added paths), then `Success` or `VecString` (failed paths).
    ImportDependenciesToOpenPackFile(BTreeMap<DataSource, Vec<ContainerPath>>),

    /// Save PackedFiles to the current PackFile and optionally optimize.
    /// Requires: files to save, whether to optimize.
    ///
    /// Response: [`Response::VecContainerPathVecContainerPath` (added paths, deleted paths) on success, [`Response::Error`] on failure.
    SavePackedFilesToPackFileAndClean(Vec<RFile>, bool),

    /// Get all file names under a path in all dependencies.
    ///
    /// Response: [`Response::HashMapDataSourceHashSetContainerPath`].
    GetPackedFilesNamesStartingWitPathFromAllSources(ContainerPath),

    //-----------------------------------------------------------------------//
    // Notes Commands
    //-----------------------------------------------------------------------//

    /// Get all notes under a path.
    ///
    /// Response: [`Response::VecNote`].
    NotesForPath(String),

    /// Add a note.
    ///
    /// Response: [`Response::Note`].
    AddNote(Note),

    /// Delete a note.
    ///
    /// Response: None.
    DeleteNote(String, u64),

    //-----------------------------------------------------------------------//
    // Schema Patch Commands
    //-----------------------------------------------------------------------//

    /// Save local schema patches.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SaveLocalSchemaPatch(HashMap<String, DefinitionPatch>),

    /// Remove local schema patches for a table.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    RemoveLocalSchemaPatchesForTable(String),

    /// Remove local schema patches for a specific field in a table.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    RemoveLocalSchemaPatchesForTableAndField(String, String),

    /// Import a schema patch into the local schema patches.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    ImportSchemaPatch(HashMap<String, DefinitionPatch>),

    //-----------------------------------------------------------------------//
    // Loc Generation Commands
    //-----------------------------------------------------------------------//

    /// Generate all missing loc entries for the currently open PackFile.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] on failure.
    GenerateMissingLocData,

    //-----------------------------------------------------------------------//
    // Lua Autogen Commands
    //-----------------------------------------------------------------------//

    /// Check for updates on the tw_autogen repository.
    ///
    /// Response: [`Response::APIResponseGit`] on success, [`Response::Error`] on failure.
    CheckLuaAutogenUpdates,

    /// Update the tw_autogen repository.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateLuaAutogen,

    //-----------------------------------------------------------------------//
    // MyMod Commands
    //-----------------------------------------------------------------------//

    /// Initialize a MyMod folder.
    /// Requires: mod name, game key, sublime support, vscode support, git support (gitignore content).
    ///
    /// Response: [`Response::PathBuf` (path to the new pack) on success, [`Response::Error`] on failure.
    InitializeMyModFolder(String, String, bool, bool, Option<String>),

    /// Live export the PackFile to the game folder.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    LiveExport,

    //-----------------------------------------------------------------------//
    // Map Packing Commands
    //-----------------------------------------------------------------------//

    /// Pack map tiles into the PackFile.
    /// Requires: tile map paths, list of (tile path, name).
    ///
    /// Response: [`Response::VecContainerPathVecContainerPath`] (added paths, deleted paths) on success, [`Response::Error`] on failure.
    PackMap(Vec<PathBuf>, Vec<(PathBuf, String)>),

    //-----------------------------------------------------------------------//
    // Diagnostics Ignore Commands
    //-----------------------------------------------------------------------//

    /// Add a line to the pack's ignored diagnostics.
    ///
    /// Response: None.
    AddLineToPackIgnoredDiagnostics(String),

    //-----------------------------------------------------------------------//
    // Empire/Napoleon AK Commands
    //-----------------------------------------------------------------------//

    /// Check for updates on the old AK files repository.
    ///
    /// Response: [`Response::APIResponseGit`] on success, [`Response::Error`] on failure.
    CheckEmpireAndNapoleonAKUpdates,

    /// Update the old AK files repository.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateEmpireAndNapoleonAK,

    //-----------------------------------------------------------------------//
    // Translation Commands
    //-----------------------------------------------------------------------//

    /// Get pack translation data for a language.
    ///
    /// Response: [`Response::PackTranslation`] on success, [`Response::Error`] on failure.
    GetPackTranslation(String),

    /// Check for translation updates.
    ///
    /// Response: [`Response::APIResponseGit`] on success, [`Response::Error`] on failure.
    CheckTranslationsUpdates,

    /// Update the translations repository.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    UpdateTranslations,

    //-----------------------------------------------------------------------//
    // Starpos Commands
    //-----------------------------------------------------------------------//

    /// Build starpos (pre-processing step).
    /// Requires: campaign ID, process HLP/SPD data.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    BuildStarpos(String, bool),

    /// Build starpos (post-processing step).
    /// Requires: campaign ID, process HLP/SPD data.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] on failure.
    BuildStarposPost(String, bool),

    /// Clean up starpos temporary files.
    /// Requires: campaign ID, process HLP/SPD data.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    BuildStarposCleanup(String, bool),

    /// Get campaign IDs for starpos building.
    ///
    /// Response: [`Response::HashSetString`].
    BuildStarposGetCampaingIds,

    /// Check if victory conditions file exists (required for some games).
    ///
    /// Response: [`Response::Success`] if exists or not needed, [`Response::Error`] if missing.
    BuildStarposCheckVictoryConditions,

    //-----------------------------------------------------------------------//
    // Animation Commands
    //-----------------------------------------------------------------------//

    /// Update animation IDs with offset.
    /// Requires: starting ID, offset.
    ///
    /// Response: [`Response::VecContainerPath`] on success, [`Response::Error`] on failure.
    UpdateAnimIds(i32, i32),

    /// Get animation paths by skeleton name.
    ///
    /// Response: [`Response::HashSetString`].
    GetAnimPathsBySkeletonName(String),

    //-----------------------------------------------------------------------//
    // Table Commands
    //-----------------------------------------------------------------------//

    /// Get tables from dependencies by table name.
    ///
    /// Response: [`Response::VecRFile`] on success, [`Response::Error`] on failure.
    GetTablesFromDependencies(String),

    /// Get table paths by table name from the current PackFile.
    ///
    /// Response: [`Response::VecString`].
    GetTablesByTableName(String),

    /// Add keys to the key_deletes table.
    /// Requires: table file name, key table name, keys to add.
    ///
    /// Response: [`Response::OptionContainerPath`].
    AddKeysToKeyDeletes(String, String, HashSet<String>),

    //-----------------------------------------------------------------------//
    // 3D Export Commands
    //-----------------------------------------------------------------------//

    /// Export a RigidModel to glTF format.
    /// Requires: RigidModel, output path.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
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
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetBool(String, bool),

    /// Set an i32 setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetI32(String, i32),

    /// Set an f32 setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetF32(String, f32),

    /// Set a string setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetString(String, String),

    /// Set a PathBuf setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetPathBuf(String, PathBuf),

    /// Set a `Vec<String>` setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetVecString(String, Vec<String>),

    /// Set raw data setting value.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsSetVecRaw(String, Vec<u8>),

    //-----------------------------------------------------------------------//
    // Path Commands
    //-----------------------------------------------------------------------//

    /// Get the config path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    ConfigPath,

    /// Get the Assembly Kit path for the current game.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    AssemblyKitPath,

    /// Get the backup autosave path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    BackupAutosavePath,

    /// Get the old AK data path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    OldAkDataPath,

    /// Get the schemas path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    SchemasPath,

    /// Get the table profiles path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    TableProfilesPath,

    /// Get the translations local path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    TranslationsLocalPath,

    /// Get the dependencies cache path.
    ///
    /// Response: [`Response::PathBuf`] on success, [`Response::Error`] on failure.
    DependenciesCachePath,

    /// Clear a config path.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    SettingsClearPath(PathBuf),

    //-----------------------------------------------------------------------//
    // Settings Backup Commands
    //-----------------------------------------------------------------------//

    /// Backup the current settings to memory.
    ///
    /// Response: None.
    BackupSettings,

    /// Clear settings and reset to defaults.
    ///
    /// Response: [`Response::Success`] on success, [`Response::Error`] on failure.
    ClearSettings,

    /// Restore settings from the backup.
    ///
    /// Response: None.
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
    /// Response: [`Response::VecDefinition`] on success, [`Response::Error`] if no schema.
    DefinitionsByTableName(String),

    /// Get columns that reference a table's definition.
    ///
    /// Response: [`Response::HashMapStringHashMapStringVecString`] on success, [`Response::Error`] if no schema.
    ReferencingColumnsForDefinition(String, Definition),

    /// Get the current schema.
    ///
    /// Response: [`Response::Schema`] on success, [`Response::Error`] if no schema.
    Schema,

    /// Get a specific definition by table name and version.
    ///
    /// Response: [`Response::Definition`] on success, [`Response::Error`] if not found or no schema.
    DefinitionByTableNameAndVersion(String, i32),

    /// Delete a definition by table name and version.
    ///
    /// Response: None.
    DeleteDefinition(String, i32)
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
    VecContainerPathVecContainerPath(Vec<ContainerPath>, Vec<ContainerPath>),
    VecContainerPathVecRFileInfo(Vec<ContainerPath>, Vec<RFileInfo>),
    VecDataSourceStringStringUsizeUsize(Vec<(DataSource, String, String, usize, usize)>),
    VecDefinition(Vec<Definition>),
    VecNote(Vec<Note>),
    VecRFile(Vec<RFile>),
    VecRFileInfo(Vec<RFileInfo>),
    VecString(Vec<String>),
    VecU8(Vec<u8>),
    VideoInfoRFileInfo(VideoInfo, RFileInfo),
    VMDRFileInfo(Text, RFileInfo),
    WSModelRFileInfo(Text, RFileInfo),

    /// All settings in one response (for batch loading).
    /// Contains: (bool settings, i32 settings, f32 settings, string settings)
    SettingsAll(
        HashMap<String, bool>,
        HashMap<String, i32>,
        HashMap<String, f32>,
        HashMap<String, String>,
    ),
}
