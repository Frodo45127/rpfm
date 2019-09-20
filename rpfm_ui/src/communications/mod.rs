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
use rpfm_lib::settings::*;
use rpfm_lib::packfile::{PackFileInfo, PathType, PFHFileType};
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::ui_state::global_search::GlobalSearch;

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers neccesary to communicate both, backend and frontend threads.
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

    /// This command is used when we want to save our settings to disk. It requires the settings to save.
    SetSettings(Settings),

    /// This command is used when we want to get the data used to build the `TreeView`.
    GetPackFileDataForTreeView,

    /// Same as the one before, but for the extra `PackFile`.
    GetPackFileExtraDataForTreeView,

    /// This command is used to open one or more `PackFiles`. It requires the paths of the `PackFiles`.
    OpenPackFiles(Vec<PathBuf>),

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

    /// Respone to return (u32).
    U32(u32),

    /// Respone to return (String, i64, Vec<Vec<String>>).
    StringI64VecVecString((String, i64, Vec<Vec<String>>)),

    /// Respone to return (PackFileInfo, Vec<PackedFileInfo>).
    PackFileInfoVecPackedFileInfo((PackFileInfo, Vec<PackedFileInfo>)),

    /// Respone to return (PackFileInfo).
    PackFileInfo(PackFileInfo),

    /// Response to return (Vec<Option<PackedFileInfo>>).
    VecOptionPackedFileInfo(Vec<Option<PackedFileInfo>>),

    /// Response to return (GlobalSearch).
    GlobalSearch(GlobalSearch),

    /// Response to return (Vec<Vec<String>>).
    VecVecString(Vec<Vec<String>>),

    /// Response to return (String, Vec<Vec<String>>).
    StringVecVecString((String, Vec<Vec<String>>))
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
