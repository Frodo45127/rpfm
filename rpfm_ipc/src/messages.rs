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


use rpfm_lib::files::{
    ContainerPath, RFile,
};
use rpfm_lib::schema::{Definition, Field};

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

/// This enum represents the current operational mode for a pack.
///
/// A pack can either be in normal mode or in MyMod mode, which links it to
/// a specific game folder and mod name for import/export operations.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalMode {

    /// MyMod mode enabled. Contains the game folder name (e.g. "warhammer_2") and the MyMod pack name.
    MyMod(String, String),

    /// Normal mode - no MyMod association.
    #[default]
    Normal,
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

    /// Get the data used to build the `TreeView` for a specific pack.
    /// The field is the pack key.
    ///
    /// Response:
    /// - [`Response::ContainerInfoVecRFileInfo`].
    GetPackFileDataForTreeView(String),

    /// Get the `RFileInfo` of one or more `PackedFiles` from a specific pack.
    /// First field is the pack key, second is the list of file paths.
    ///
    /// Response: [`Response::VecRFileInfo`].
    GetPackedFilesInfo(String, Vec<String>),

    /// Get the info of a single `PackedFile` from a specific pack.
    /// First field is the pack key, second is the file path.
    ///
    /// Response: [`Response::OptionRFileInfo`].
    GetRFileInfo(String, String),

    //-----------------------------------------------------------------------//
    // Update Commands
    //-----------------------------------------------------------------------//

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

    /// Add PackedFiles from one open pack into another.
    /// First field is the target pack key, second is the source pack key, third is the paths to copy.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] if source pack not found.
    AddPackedFilesFromPackFile(String, String, Vec<ContainerPath>),

    /// Add PackedFiles from a specific pack to an AnimPack, which may live in a different pack.
    /// Fields are the source pack key, the pack key that owns the AnimPack, the animpack path,
    /// and the container paths to copy.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    AddPackedFilesFromPackFileToAnimpack(String, String, String, Vec<ContainerPath>),

    /// Add PackedFiles from an AnimPack to a specific pack, which may differ from the AnimPack's own.
    /// Fields are the pack key that owns the AnimPack (only used when the data source is a PackFile),
    /// the destination pack key, the data source, the animpack path, and the container paths.
    ///
    /// Response:
    /// - [`Response::VecContainerPath`] on success.
    /// - [`Response::Error`] on failure.
    AddPackedFilesFromAnimpack(String, String, DataSource, String, Vec<ContainerPath>),

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

    //-----------------------------------------------------------------------//
    // Search Commands
    //-----------------------------------------------------------------------//

    /// Get PackedFiles from all known sources (PackFile, GameFiles, ParentFiles).
    /// Requires: paths to get, whether to lowercase paths.
    ///
    /// Response: [`Response::HashMapDataSourceHashMapStringRFile`].
    GetRFilesFromAllSources(Vec<ContainerPath>, bool),

    //-----------------------------------------------------------------------//
    // Video Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Schema Commands
    //-----------------------------------------------------------------------//

    /// Encode and clean the cache for the provided paths in a specific pack.
    /// First field is the pack key, second is the paths to clean.
    ///
    /// Response: [`Response::Success`].
    CleanCache(String, Vec<ContainerPath>),

    //-----------------------------------------------------------------------//
    // TSV Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // External Program Commands
    //-----------------------------------------------------------------------//

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

    //-----------------------------------------------------------------------//
    // Diagnostics Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Pack Settings Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Debug Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Dependencies Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Cascade Edition Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Navigation Commands
    //-----------------------------------------------------------------------//

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

    //-----------------------------------------------------------------------//
    // Schema Patch Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Loc Generation Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Lua Autogen Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // MyMod Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Map Packing Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Diagnostics Ignore Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Empire/Napoleon AK Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Translation Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Translation Hub Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Starpos Commands
    //-----------------------------------------------------------------------//


    //-----------------------------------------------------------------------//
    // CEO Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Animation Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Table Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // 3D Export Commands
    //-----------------------------------------------------------------------//

    //-----------------------------------------------------------------------//
    // Schema Query Commands
    //-----------------------------------------------------------------------//

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

    Bool(bool),
    ContainerInfoVecRFileInfo((ContainerInfo, Vec<RFileInfo>)),
    HashMapDataSourceHashMapStringRFile(HashMap<DataSource, HashMap<String, RFile>>),
    HashMapDataSourceHashSetContainerPath(HashMap<DataSource, HashSet<ContainerPath>>),
    OptionRFileInfo(Option<RFileInfo>),
    PathBuf(PathBuf),
    StringVecPathBuf(String, Vec<PathBuf>),
    VecContainerPath(Vec<ContainerPath>),
    VecContainerPathContainerPath(Vec<(ContainerPath, ContainerPath)>),
    VecContainerPathOptionString(Vec<ContainerPath>, Option<String>),
    VecContainerPathVecContainerPath(Vec<ContainerPath>, Vec<ContainerPath>),
    VecContainerPathBTreeMapStringVecContainerPath(Vec<ContainerPath>, BTreeMap<String, Vec<ContainerPath>>),
    VecContainerPathVecString(Vec<ContainerPath>, Vec<String>),
    VecField(Vec<Field>),
    VecRFileInfo(Vec<RFileInfo>),
}
