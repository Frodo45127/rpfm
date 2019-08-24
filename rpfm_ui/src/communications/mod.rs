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

use crossbeam::unbounded;
use crossbeam::Sender;
use crossbeam::Receiver;

use rpfm_error::Error;
use rpfm_lib::settings::*;

use std::process::exit;

/// This const is the standard message in case of message communication error. If this happens, crash the program and send a report to Sentry.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers neccesary to communicate both, backend and frontend threads.
pub struct CentralCommand {
    sender_qt: Sender<Command>,
    sender_rust: Sender<Response>,
    receiver_qt: Receiver<Response>,
    receiver_rust: Receiver<Command>,
}

/// This enum is meant for sending commands from the UI Thread to the Background thread.
/// If you want to know what each command do, check the `background_loop` function.
/// If you need to send data, DO NOT USE THIS. Use the `Data` enum.
#[derive(Debug)]
pub enum Command {
    ResetPackFile,
    ResetPackFileExtra,
    NewPackFile,
    SetSettings(Settings),
    /*
    OpenPackFiles,
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
    GetPackFileDataForTreeView,
    GetPackFileExtraDataForTreeView,
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

/// This enum is meant to send data back and forward between threads. Variants here are 
/// defined by type. For example, if you want to send two different datas of the same type, 
// you use the same variant. It's like that because otherwise this'll be a variant chaos.
#[derive(Debug)]
pub enum Response {
    Success,
    Cancel,
    Error(Error),
    U32(u32),
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
    StringI64VecVecString((String, i64, Vec<Vec<String>>)),
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
