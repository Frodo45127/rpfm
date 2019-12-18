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
This module defines the code used for thread communication.
!*/

use qt_core::event_loop::EventLoop;

use crossbeam::{Receiver, Sender, unbounded};

use std::path::PathBuf;
use std::process::exit;

use rpfm_error::Error;

use rpfm_lib::global_search::GlobalSearch;
use rpfm_lib::packedfile::DecodedPackedFile;
use rpfm_lib::packedfile::table::{db::DB, loc::Loc};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::packedfile::rigidmodel::RigidModel;
use rpfm_lib::packfile::{PackFileInfo, PathType, PFHFileType};
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::APIResponseSchema;
use rpfm_lib::settings::*;

use crate::app_ui::NewPackedFile;
use self::network::*;

pub mod network;

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand {
    sender_qt: Sender<Command>,
    sender_rust: Sender<Response>,
    receiver_qt: Receiver<Response>,
    receiver_rust: Receiver<Command>,
}

/// This enum defines the commands (messages) you can send to the background thread in order to execute actions.
///
/// Each command should include the data needed for his own execution. For a more detailed explanation, check the
/// docs of each command.
#[derive(Debug)]
pub enum Command {

    /// This command is used when we want to reset the open `PackFile` to his original state.
    ResetPackFile,

    /// This command is used when we want to reset the extra `PackFile` (the one used for `Add from PackFile`) to his original state.
    ResetPackFileExtra,

    /// This command is used when we want to create a new `PackFile`.
    NewPackFile,

    /// This command is used when we want to save our currently open `PackFile`.
    SavePackFile,

    /// This command is used when we want to save our currently open `PackFile` as another `PackFile`.
    SavePackFileAs(PathBuf),

    /// This command is used when we want to save our settings to disk. It requires the settings to save.
    SetSettings(Settings),

    /// This command is used when we want to get the data used to build the `TreeView`.
    GetPackFileDataForTreeView,

    /// Same as the one before, but for the extra `PackFile`.
    GetPackFileExtraDataForTreeView,

    /// This command is used to open one or more `PackFiles`. It requires the paths of the `PackFiles`.
    OpenPackFiles(Vec<PathBuf>),

    /// This command is used to open an extra `PackFile`. It requires the path of the `PackFile`.
    OpenPackFileExtra(PathBuf),

    /// This command is used to open all the CA PackFiles for the game selected as one.
    LoadAllCAPackFiles,

    /// This command is used when we want to get the `PackedFileInfo` of one or more `PackedFiles`.
    GetPackedFilesInfo(Vec<Vec<String>>),

    /// This command is used when we want to perform a `Global Search`. It requires the search info.
    GlobalSearch(GlobalSearch),

    /// This command is used when we want to perform an update over a `Global Search`. It requires the search info.
    GlobalSearchUpdate(GlobalSearch, Vec<PathType>),

    /// This command is used when we want to change the `Game Selected`. It contains the name of the game to select.
    SetGameSelected(String),

    /// This command is used when we want to change the `Type` of the currently open `PackFile`. It contains the new type.
    SetPackFileType(PFHFileType),

    /// This command is used when we want to generate a PAK file for the currently selected game. It contains the path of the
    /// source files and the `Raw DB Version` of the currently selected game.
    GeneratePakFile(PathBuf, i16),

    /// This command is used when we want to trigger an optimization pass over the currently open `PackFile`.
    OptimizePackFile,

    /// This command is used to patch the SiegeAI of a Siege Map for warhammer games.
    PatchSiegeAI,

    /// This command is used when we want to change the `Index Includes Timestamp` flag in the currently open `PackFile`
    ChangeIndexIncludesTimestamp(bool),

    /// This command is used when we want to change the `Data is Compressed` flag in the currently open `PackFile`
    ChangeDataIsCompressed(bool),

    /// This command is used when we want to know the current path of our currently open `PackFile`.
    GetPackFilePath,

    /// This command is used when we want to get the info of the provided `PackedFile`.
    GetPackedFileInfo(Vec<String>),

    /// This command is used when we want to check if there is an RPFM update available.
    CheckUpdates,

    /// This command is used when we want to check if there is an Schema update available.
    CheckSchemaUpdates,

    /// This command is used when we want to update our schemas.
    UpdateSchemas,

    /// This command is used when we want to know if there is a Dependency Database loaded in memory.
    IsThereADependencyDatabase,

    /// This command is used when we want to know if there is a Schema loaded in memory.
    IsThereASchema,

    /// This command is used when we want to create a new `PackedFile` inside the currently open `PackFile`.
    ///
    /// It requires the path of the new PackedFile, and the `NewPackedFile` with the new PackedFile's info.
    NewPackedFile(Vec<String>, NewPackedFile),

    /// This command is used when we want to add one or more Files to our currently open `PackFile`.
    ///
    /// It requires the list of filesystem paths to add, and their path once they're inside the `PackFile`.
    AddPackedFiles((Vec<PathBuf>, Vec<Vec<String>>)),

    /// This command is used when we want to decode an image to be shown in the UI.
    DecodePackedFileImage(Vec<String>),

    /// This command is used when we want to decode a text PackedFile to be shown in the UI.
    DecodePackedFileText(Vec<String>),

    /// This command is used when we want to decode a rigidmodel PackedFile to be shown in the UI.
    DecodePackedFileRigidModel(Vec<String>),

    /// This command is used when we want to save an edited `PackedFile` back to the `PackFile`.
    SavePackedFileFromView(Vec<String>, DecodedPackedFile),

    /// This command is used when we want to decode a table PackedFile to be shown in the UI.
    DecodePackedFileTable(Vec<String>),

    /// This command is used when we want to add a PackedFile from one PackFile into another.
    AddPackedFileFromPackFile(Vec<PathType>),

    /// This command is used when we want to delete one or more PackedFiles from a PackFile. It contains the PathType of each PackedFile to delete.
    DeletePackedFiles(Vec<PathType>),

    /// This command is used when we want to extract one or more PackedFiles from a PackFile. It contains the PathTypes to extract and the extraction path.
    ExtractPackedFiles(Vec<PathType>, PathBuf),

    /// This command is used when we want to rename one or more PackedFiles in a PackFile. It contains a Vec with their original PathType and their new name.
    RenamePackedFiles(Vec<(PathType, String)>),

    /// This command is used when we want to import a large amount of table-like files from TSV files.
    MassImportTSV(Vec<PathBuf>, Option<String>),

    /// This command is used when we want to export a large amount of table-like files as TSV files.
    MassExportTSV(Vec<PathType>, PathBuf),

    /// This command is used when we want to know if a folder exists in the currently open PackFile.
    FolderExists(Vec<String>),

    /// This command is used when we want to know if a PackedFile exists in the currently open PackFile.
    PackedFileExists(Vec<String>),

    /// This command is used when we want to get the table names (the folder of the tables) of all DB files in our dependency PackFiles.
    GetTableListFromDependencyPackFile,

    /// This command is used when we want to get the version of the table provided that's compatible with the version of the game we currently have installed.
    GetTableVersionFromDependencyPackFile(String),

    /// This command is used when we want to check the integrity of all the DB Tables in the PackFile.
    DBCheckTableIntegrity,

    /// This command is used when we want to merge multiple compatible tables into one. The contents of this are as follows:
    /// - Vec<Vec<String>>: List of paths to merge.
    /// - String: Name of the new merged table.
    /// - Bool: Should we delete the source files after merging them?
    MergeTables(Vec<Vec<String>>, String, bool),

    /// This command is used when we want to update a table to a newer version.
    UpdateTable(PathType),
    /*
    OpenPackFileExtra,
    SavePackFile,
    SavePackFileAs,
    LoadAllCAPackFiles,
    SetPackFileType,
    ChangeIndexIncludesTimestamp,
    ChangeDataIsCompressed,
    SaveSchema,
    SetSettings,
    SetShortcuts,
    SetGameSelected,
    IsThereADependencyDatabase,
    IsThereASchema,
    PatchSiegeAI,
    UpdateSchemas,
    AddPackedFile,
    DeletePackedFile,
    ExtractPackedFile,
    PackedFileExists,
    FolderExists,
    CreatePackedFile,
    AddPackedFileFromPackFile,
    MassImportTSV,
    MassExportTSV,
    DecodePackedFileLoc,
    EncodePackedFileLoc,
    DecodePackedFileDB,
    EncodePackedFileDB,
    DecodePackedFileText,
    EncodePackedFileText,
    DecodePackedFileRigidModel,
    EncodePackedFileRigidModel,
    PatchAttilaRigidModelToWarhammer,
    DecodePackedFileImage,
    RenamePackedFiles,
    GetPackedFile,
    GetTableListFromDependencyPackFile,
    GetTableVersionFromDependencyPackFile,
    OptimizePackFile,
    GeneratePakFile,
    GetPackFilesList,
    SetPackFilesList,
    DecodeDependencyDB,
    CheckScriptWithKailua,
    GlobalSearch,
    UpdateGlobalSearchData,
    OpenWithExternalProgram,
    OpenContainingFolder,
    ImportTSVPackedFile,
    ExportTSVPackedFile,
    CheckTables,
    MergeTables,
    GenerateSchemaDiff,
    GetNotes,
    SetNotes,*/
}

/// This enum defines the responses (messages) you can send to the to the UI thread as result of a command.
///
/// Each response should be named after the types of the items it carries.
#[derive(Debug)]
pub enum Response {

    /// Generic response for situations of success.
    Success,

    /// Generic response for situations where the action was cancelled.
    Cancel,

    /// Generic response for situations that returned an error.
    Error(Error),

    /// Response to return (bool).
    Bool(bool),

    /// Response to return (u32).
    U32(u32),

    /// Response to return (i32).
    I32(i32),

    /// Response to return (i64).
    I64(i64),

    /// Response to return (PathBuf).
    PathBuf(PathBuf),

    /// Response to return (String)
    String(String),

    /// Response to return (String, i64, Vec<Vec<String>>).
    StringI64VecVecString((String, i64, Vec<Vec<String>>)),

    /// Response to return (PackFileInfo, Vec<PackedFileInfo>).
    PackFileInfoVecPackedFileInfo((PackFileInfo, Vec<PackedFileInfo>)),

    /// Response to return (PackFileInfo).
    PackFileInfo(PackFileInfo),

    /// Response to return (Option<PackedFileInfo>).
    OptionPackedFileInfo(Option<PackedFileInfo>),

    /// Response to return (Vec<Option<PackedFileInfo>>).
    VecOptionPackedFileInfo(Vec<Option<PackedFileInfo>>),

    /// Response to return (GlobalSearch).
    GlobalSearch(GlobalSearch),

    /// Response to return (Vec<Vec<String>>).
    VecVecString(Vec<Vec<String>>),

    /// Response to return (Vec<PathType>).
    VecPathType(Vec<PathType>),

    /// Response to return (Vec<(PathType, String)>).
    VecPathTypeString(Vec<(PathType, String)>),

    /// Response to return (Vec<PathType>),
    VecPathTypeVecPathType((Vec<PathType>, Vec<PathType>)),

    /// Response to return (String, Vec<Vec<String>>).
    StringVecVecString((String, Vec<Vec<String>>)),

    /// Response to return `APIResponse`.
    APIResponse(APIResponse),

    /// Response to return `APIResponseSchema`.
    APIResponseSchema(APIResponseSchema),

    /// Response to return `Text`.
    Text(Text),

    /// Response to return `DB`.
    DB(DB),

    /// Response to return `Loc`.
    Loc(Loc),

    /// Response to return `RigidModel`.
    RigidModel(RigidModel),

    /// Response to return `Unknown`.
    Unknown,

    /// Response to return `(Vec<Vec<String>>, Vec<Vec<String>>)`.
    VecVecStringVecVecString((Vec<Vec<String>>, Vec<Vec<String>>)),

    /// Response to return `Vec<String>`.
    VecString(Vec<String>),

    /// Response to return `(Vec<String>, Vec<PathType>)`.
    VecStringVecPathType((Vec<String>, Vec<PathType>)),

    /// Response to return `(i32, i32)`.
    I32I32((i32, i32))
/*
    Bool(bool),
    I32(i32),
    I64(i64),

    String(String),
    StringVecString((String, Vec<String>)),
    StringVecVecString((String, Vec<Vec<String>>)),
    PathBuf(PathBuf),
    PathBufI16((PathBuf, i16)),

    Settings(Settings),
    Shortcuts(Shortcuts),
    Schema(Schema),

    PFHFileType(PFHFileType),
    PackFileUIData(PackFileInfo),

    PackedFile(PackedFile),
    DefinitionPathBufStringI32((Definition, PathBuf, String, i32)),
    VecVecDecodedData((Vec<Vec<DecodedData>>)),
    VecVecDecodedDataPathBufVecStringTupleStrI32((Vec<Vec<DecodedData>>, PathBuf, Vec<String>, (String, i32))),

    Loc(Loc),
    LocVecString((Loc, Vec<String>)),

    DB(DB),
    DBVecString((DB, Vec<String>)),

    RigidModel(RigidModel),
    RigidModelVecString((RigidModel, Vec<String>)),

    PathType(PathType),

    OptionStringVecPathBuf((Option<String>, Vec<PathBuf>)),

    StringVecPathType((String, Vec<PathType>)),
    VecPathBufVecVecString((Vec<PathBuf>, Vec<Vec<String>>)),
    VecString(Vec<String>),
    VecStringPackedFileType((Vec<String>, PackedFileType)),
    VecVecStringStringBoolBool((Vec<Vec<String>>, String, bool, bool)),
    VecVecStringVecVecString((Vec<Vec<String>>, Vec<Vec<String>>)),
    VecGlobalMatch(Vec<GlobalMatch>),
    VersionsVersions((Versions, Versions)),
    VecPathTypeString(Vec<(PathType, String)>),
    VecPathType(Vec<PathType>),
    VecStringVecPathType((Vec<String>, Vec<PathType>)),
    VecPathTypePathBuf((Vec<PathType>, PathBuf)),
    VecPathBuf(Vec<PathBuf>),
    Definition(Definition),
    BTreeMapI32VecString(BTreeMap<i32, Vec<String>>),*/
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl Default for CentralCommand {
    fn default() -> Self {
        let command_channel = unbounded();
        let response_channel = unbounded();
        Self {
            sender_qt: command_channel.0,
            sender_rust: response_channel.0,
            receiver_qt: response_channel.1,
            receiver_rust: command_channel.1,
        }
    }
}

/// Implementation of `CentralCommand`.
impl CentralCommand {

    /// This function serves to send message from the main thread to the background thread.
    #[allow(dead_code)]
    pub fn send_message_qt(&self, data: Command) {
        if self.sender_qt.send(data).is_err() {
            panic!(THREADS_COMMUNICATION_ERROR)
        }
    }

    /// This function serves to send message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_rust(&self, data: Response) {
        if self.sender_rust.send(data).is_err() {
            panic!(THREADS_COMMUNICATION_ERROR)
        }
    }

    /// This functions serves to receive messages from the main thread into the background thread.
    #[allow(dead_code)]
    pub fn recv_message_rust(&self) -> Command {
        match self.receiver_rust.recv() {
            Ok(data) => data,

            // If we hit an error here, it means the main thread is dead. So... report it and exit.
            Err(_) => {
                println!("Main UI Thread dead. Exiting...");
                exit(0);
            }
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function does only try once, and it locks the thread. Use it only in small stuff.
    #[allow(dead_code)]
    pub fn recv_message_qt(&self) -> Response {
        match self.receiver_qt.recv() {
            Ok(data) => data,
            Err(_) => panic!(THREADS_COMMUNICATION_ERROR)
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_qt_try(&self) -> Response {
        let mut event_loop = EventLoop::new();
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            match self.receiver_qt.try_recv() {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() { panic!(THREADS_COMMUNICATION_ERROR) }
            }
            event_loop.process_events(());
        }
    }
}
