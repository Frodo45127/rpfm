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
This module defines the code used for thread communication.
!*/

use qt_core::QEventLoop;

use crossbeam::channel::{Receiver, Sender, unbounded};

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::exit;

use rpfm_error::Error;

use rpfm_lib::diagnostics::Diagnostics;
use rpfm_lib::global_search::GlobalSearch;
use rpfm_lib::global_search::MatchHolder;
use rpfm_lib::packedfile::ca_vp8::{CaVp8, SupportedFormats};
use rpfm_lib::packedfile::{DecodedPackedFile, PackedFileType};
use rpfm_lib::packedfile::image::Image;
use rpfm_lib::packedfile::table::{DependencyData, anim_fragment::AnimFragment, animtable::AnimTable, db::{DB, CascadeEdition}, loc::Loc, matched_combat::MatchedCombat};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::packedfile::rigidmodel::RigidModel;
use rpfm_lib::packedfile::uic::UIC;
use rpfm_lib::packfile::{PackFileInfo, PackFileSettings, PathType, PFHFileType};
use rpfm_lib::packfile::packedfile::{PackedFile, PackedFileInfo};
use rpfm_lib::schema::{APIResponseSchema, Definition, Schema};
use rpfm_lib::settings::*;
use rpfm_lib::template::Template;
use rpfm_lib::updater::APIResponse;

use crate::app_ui::NewPackedFile;
use crate::views::table::TableType;
use crate::ui_state::shortcuts::Shortcuts;

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand {
    sender_qt: Sender<Command>,
    sender_rust: Sender<Response>,
    sender_qt_to_network: Sender<Command>,
    sender_network_to_qt: Sender<Response>,
    sender_notification_to_qt: Sender<Notification>,
    sender_diagnostics_to_qt: Sender<Diagnostics>,
    sender_diagnostics_update_to_qt: Sender<(Diagnostics, Vec<PackedFileInfo>)>,
    sender_global_search_update_to_qt: Sender<(GlobalSearch, Vec<PackedFileInfo>)>,
    sender_save_packedfile: Sender<Response>,

    receiver_qt: Receiver<Response>,
    receiver_rust: Receiver<Command>,
    receiver_qt_to_network: Receiver<Command>,
    receiver_network_to_qt: Receiver<Response>,
    receiver_notification_to_qt: Receiver<Notification>,
    receiver_diagnostics_to_qt: Receiver<Diagnostics>,
    receiver_diagnostics_update_to_qt: Receiver<(Diagnostics, Vec<PackedFileInfo>)>,
    receiver_global_search_update_to_qt: Receiver<(GlobalSearch, Vec<PackedFileInfo>)>,
    receiver_save_packedfile: Receiver<Response>,
}

/// This enum defines the commands (messages) you can send to the background thread in order to execute actions.
///
/// Each command should include the data needed for his own execution. For a more detailed explanation, check the
/// docs of each command.
#[derive(Debug)]
pub enum Command {

    /// This command is used when we want to reset the open `PackFile` to his original state.
    ResetPackFile,

    /// This command is used when we want to remove from memory the extra packfile with the provided path.
    RemovePackFileExtra(PathBuf),

    /// This command is used to "clean" a Packfile from corrupted files and save it to disk.
    CleanAndSavePackFileAs(PathBuf),

    /// This command is used when we want to create a new `PackFile`.
    NewPackFile,

    /// This command is used when we want to save our currently open `PackFile`.
    SavePackFile,

    /// This command is used when we want to save our currently open `PackFile` as another `PackFile`.
    SavePackFileAs(PathBuf),

    /// This command is used when we want to save our settings to disk. It requires the settings to save.
    SetSettings(Settings),

    /// This command is used when we want to save our shortcuts to disk. It requires the shortcuts to save.
    SetShortcuts(Shortcuts),

    /// This command is used when we want to get the data used to build the `TreeView`.
    GetPackFileDataForTreeView,

    /// Same as the one before, but for the extra `PackFile`. It requires the pathbuf of the PackFile.
    GetPackFileExtraDataForTreeView(PathBuf),

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

    /// This command is used when we want to update the currently loaded Schema with data from the game selected's Assembly Kit.
    /// It contains the path of the source files, if needed.
    UpdateCurrentSchemaFromAssKit(Option<PathBuf>),

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
    AddPackedFiles((Vec<PathBuf>, Vec<Vec<String>>, Option<Vec<PathBuf>>)),

    /// This command is used when we want to decode a PackedFile to be shown on the UI.
    DecodePackedFile(Vec<String>),

    /// This command is used when we want to save an edited `PackedFile` back to the `PackFile`.
    SavePackedFileFromView(Vec<String>, DecodedPackedFile),

    /// This command is used when we want to add a PackedFile from one PackFile into another.
    AddPackedFilesFromPackFile((PathBuf, Vec<PathType>)),

    /// This command is used when we want to add a PackedFile from our PackFile to an Animpack.
    AddPackedFilesFromPackFileToAnimpack((Vec<String>, Vec<PathType>)),

    /// This command is used when we want to add a PackedFile from an AnimPack to our PackFile.
    AddPackedFilesFromAnimpack((Vec<String>, Vec<PathType>)),

    /// This command is used when we want to delete a PackedFile from an AnimPack.
    DeleteFromAnimpack((Vec<String>, Vec<PathType>)),

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

    /// This command is used when we want to merge multiple compatible tables into one. The contents of this are as follows:
    /// - Vec<Vec<String>>: List of paths to merge.
    /// - String: Name of the new merged table.
    /// - Bool: Should we delete the source files after merging them?
    MergeTables(Vec<Vec<String>>, String, bool),

    /// This command is used when we want to update a table to a newer version.
    UpdateTable(PathType),

    /// This command is used when we want to replace some specific matches in a Global Search.
    GlobalSearchReplaceMatches(GlobalSearch, Vec<MatchHolder>),

    /// This command is used when we want to replace all matches in a Global Search.
    GlobalSearchReplaceAll(GlobalSearch),

    /// This command is used when we want to add entire folders to the PackFile. The tuples contains their path in disk and their starting path in the PackFile,
    /// and the list of paths to ignore, if any.
    AddPackedFilesFromFolder(Vec<(PathBuf, Vec<String>)>, Option<Vec<PathBuf>>),

    /// This command is used to decode all tables referenced by columns in the provided definition and return their data.
    /// It requires the table name, the definition of the table to get the reference data from and the list of PackedFiles to ignore.
    GetReferenceDataFromDefinition(String, Definition, Vec<Vec<String>>),

    /// This command is used to get the list of PackFiles that are marked as dependency of our PackFile.
    GetDependencyPackFilesList,

    /// This command is used to set the list of PackFiles that are marked as dependency of our PackFile.
    SetDependencyPackFilesList(Vec<String>),

    /// This command is used to get a full PackedFile to the UI. Requires the path of the PackedFile.
    GetPackedFile(Vec<String>),

    /// This command is used to change the format of a ca_vp8 video packedfile. Requires the path of the PackedFile and the new format.
    SetCaVp8Format((Vec<String>, SupportedFormats)),

    /// This command is used to save the provided schema to disk.
    SaveSchema(Schema),

    /// This command is used to save to encoded data the cache of the provided paths, and then clean up the cache.
    CleanCache(Vec<Vec<String>>),

    /// This command is used to export a table as TSV. Requires the internal and destination paths for the PackedFile.
    ExportTSV((Vec<String>, PathBuf)),

    /// This command is used to import a TSV as a table. Requires the internal and destination paths for the PackedFile.
    ImportTSV((Vec<String>, PathBuf)),

    /// This command is used to open in the defaul file manager the folder of the currently open PackFile.
    OpenContainingFolder,

    /// This command is used to open a PackedFile on a external program. Requires the internal path of the PackedFile.
    OpenPackedFileInExternalProgram(Vec<String>),

    /// This command is used to save a PackedFile from an external program. Requires both, internal and external paths of the PackedFile.
    SavePackedFileFromExternalView((Vec<String>, PathBuf)),

    /// This command is used to load a template into the currently open PackFile.
    /// The data it contains is:
    /// - Template.
    /// - Options list.
    /// - Params list.
    /// - Is custom?
    ApplyTemplate(Template, Vec<(String, bool)>, Vec<(String, String)>, bool),

    /// This command is used to save a PackFile into a template.
    SaveTemplate(Template),

    /// This command is used to update the templates.
    UpdateTemplates,

    /// This command is used to update the program to the last version available, if possible.
    UpdateMainProgram,

    /// This command is used to trigger an autosave to a backup from time to time.
    TriggerBackupAutosave,

    /// This command is used to trigger a full diagnostics check over the open PackFile.
    DiagnosticsCheck,

    /// This command is used to trigger a partial diagnostics check over the open PackFile.
    DiagnosticsUpdate((Diagnostics, Vec<PathType>)),

    /// This command is used to check for template updates.
    CheckTemplateUpdates,

    /// This command is used to get the settings of the currently open PackFile.
    GetPackFileSettings,

    /// This command is used to set the settings of the currently open PackFile.
    SetPackFileSettings(PackFileSettings),

    /// This command is used to get the definitions of all the tables in the PackFile.
    GetDefinitionList,

    /// This command is used to trigger the debug missing table definition's code.
    GetMissingDefinitions,

    /// This command is used to rebuild the dependencies of a PackFile.
    RebuildDependencies,

    /// This command is used to trigger a cascade edition on all referenced data.
    CascadeEdition(CascadeEdition),

    /// This command is used for the Go To Definition feature. Contains table, column, and value to search.
    GoToDefinition(String, String, String),

    /// This command is used to get the source data of a loc key. Contains the loc key to search.
    GetSourceDataFromLocKey(String),

    /// This command is used to get the loc file/column/row of a key. Contains the loc key to search.
    GoToLoc(String),

    /// This command is used to get the type of a PackedFile.
    GetPackedFileType(Vec<String>)
}

/// This enum defines the responses (messages) you can send to the to the UI thread as result of a command.
///
/// Each response should be named after the types of the items it carries.
#[derive(Debug)]
pub enum Response {

    /// Generic response for situations of success.
    Success,

    /// Generic response for situations that returned an error.
    Error(Error),

    /// Response to return (bool).
    Bool(bool),

    /// Response to return (i32).
    I32(i32),

    /// Response to return (PathBuf).
    PathBuf(PathBuf),

    /// Response to return (String)
    String(String),

    /// Response to return (PackFileInfo, Vec<PackedFileInfo>).
    PackFileInfoVecPackedFileInfo((PackFileInfo, Vec<PackedFileInfo>)),

    /// Response to return (PackFileInfo).
    PackFileInfo(PackFileInfo),

    /// Response to return (Option<PackedFileInfo>).
    OptionPackedFileInfo(Option<PackedFileInfo>),

    /// Response to return (Vec<Option<PackedFileInfo>>).
    VecOptionPackedFileInfo(Vec<Option<PackedFileInfo>>),

    /// Response to return (GlobalSearch, Vec<PackedFileInfo>).
    GlobalSearchVecPackedFileInfo((GlobalSearch, Vec<PackedFileInfo>)),

    /// Response to return (Vec<Vec<String>>).
    VecVecString(Vec<Vec<String>>),

    /// Response to return (Vec<PathType>).
    VecPathType(Vec<PathType>),

    /// Response to return (Vec<(PathType, Vec<String>)>).
    VecPathTypeVecString(Vec<(PathType, Vec<String>)>),

    /// Response to return (String, Vec<Vec<String>>).
    StringVecVecString((String, Vec<Vec<String>>)),

    /// Response to return `APIResponse`.
    APIResponse(APIResponse),

    /// Response to return `APIResponseSchema`.
    APIResponseSchema(APIResponseSchema),

    /// Response to return `(AnimFragment, PackedFileInfo)`.
    AnimFragmentPackedFileInfo((AnimFragment, PackedFileInfo)),

    /// Response to return `(Vec<String>, PackedFileInfo)`.
    AnimPackPackedFileInfo(((PackFileInfo, Vec<PackedFileInfo>), PackedFileInfo)),

    /// Response to return `(AnimTable, PackedFileInfo)`.
    AnimTablePackedFileInfo((AnimTable, PackedFileInfo)),

    /// Response to return `(CaVp8, PackedFileInfo)`.
    CaVp8PackedFileInfo((CaVp8, PackedFileInfo)),

    /// Response to return `(Image, PackedFileInfo)`.
    ImagePackedFileInfo((Image, PackedFileInfo)),

    /// Response to return `(Text, PackedFileInfo)`.
    TextPackedFileInfo((Text, PackedFileInfo)),

    /// Response to return `(DB, PackedFileInfo)`.
    DBPackedFileInfo((DB, PackedFileInfo)),

    /// Response to return `(Loc, PackedFileInfo)`.
    LocPackedFileInfo((Loc, PackedFileInfo)),

    /// Response to return `(MatchedCombat, PackedFileInfo)`.
    MatchedCombatPackedFileInfo((MatchedCombat, PackedFileInfo)),

    /// Response to return `(RigidModel, PackedFileInfo)`.
    RigidModelPackedFileInfo((RigidModel, PackedFileInfo)),

    /// Response to return `(UIC, PackedFileInfo)`.
    UICPackedFileInfo((UIC, PackedFileInfo)),

    /// Response to return `Text`.
    Text(Text),

    /// Response to return `Unknown`.
    Unknown,

    /// Response to return `(Vec<Vec<String>>, Vec<Vec<String>>)`.
    VecVecStringVecVecString((Vec<Vec<String>>, Vec<Vec<String>>)),

    /// Response to return `Vec<String>`.
    VecString(Vec<String>),

    /// Response to return `(i32, i32)`.
    I32I32((i32, i32)),

    /// Response to return `BTreeMap<i32, DependencyData>`.
    BTreeMapI32DependencyData(BTreeMap<i32, DependencyData>),

    /// Response to return `Option<PackedFile>`.
    OptionPackedFile(Option<PackedFile>),

    /// Response to return `TableType`.
    TableType(TableType),

    /// Response to return `PackFileSettings`.
    PackFileSettings(PackFileSettings),

    /// Response to return `Vec<(String, Definition)>`.
    VecStringDefinition(Vec<(String, Definition)>),

    /// Response to return `Vec<Vec<String>>, Vec<PackedFileInfo>`.
    VecVecStringVecPackedFileInfo(Vec<Vec<String>>, Vec<PackedFileInfo>),

    /// Response to return `Vec<String>, usize, usize`.
    VecStringUsizeUsize(Vec<String>, usize, usize),

    /// Response to return `Option<(String, String, String)>`.
    OptionStringStringString(Option<(String, String, String)>),

    /// Response to return `PackedFileType`.
    PackedFileType(PackedFileType),
}

#[derive(Debug)]
pub enum Notification {
    Error(Error),
    Done,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl Default for CentralCommand {
    fn default() -> Self {
        let command_channel = unbounded();
        let response_channel = unbounded();
        let network_command_channel = unbounded();
        let network_response_channel = unbounded();
        let notification_response_channel = unbounded();
        let diagnostics_response_channel = unbounded();
        let diagnostics_update_response_channel = unbounded();
        let global_search_update_response_channel = unbounded();
        let save_packedfile_response_channel = unbounded();
        Self {
            sender_qt: command_channel.0,
            sender_rust: response_channel.0,
            sender_qt_to_network: network_command_channel.0,
            sender_network_to_qt: network_response_channel.0,
            sender_notification_to_qt: notification_response_channel.0,
            sender_diagnostics_to_qt: diagnostics_response_channel.0,
            sender_diagnostics_update_to_qt: diagnostics_update_response_channel.0,
            sender_global_search_update_to_qt: global_search_update_response_channel.0,
            sender_save_packedfile: save_packedfile_response_channel.0,
            receiver_qt: response_channel.1,
            receiver_rust: command_channel.1,
            receiver_qt_to_network: network_command_channel.1,
            receiver_network_to_qt: network_response_channel.1,
            receiver_notification_to_qt: notification_response_channel.1,
            receiver_diagnostics_to_qt: diagnostics_response_channel.1,
            receiver_diagnostics_update_to_qt: diagnostics_update_response_channel.1,
            receiver_global_search_update_to_qt: global_search_update_response_channel.1,
            receiver_save_packedfile: save_packedfile_response_channel.1,
        }
    }
}

/// Implementation of `CentralCommand`.
impl CentralCommand {

    /// This function serves to send message from the main thread to the background thread.
    #[allow(dead_code)]
    pub fn send_message_qt(&self, data: Command) {
        if self.sender_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_rust(&self, data: Response) {
        if self.sender_rust.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send message from the main thread to the network thread.
    #[allow(dead_code)]
    pub fn send_message_qt_to_network(&self, data: Command) {
        if self.sender_qt_to_network.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send message from the main thread to the network thread.
    #[allow(dead_code)]
    pub fn send_message_network_to_qt(&self, data: Response) {
        if self.sender_network_to_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_notification_to_qt(&self, data: Notification) {
        if self.sender_notification_to_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send diagnostics message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_diagnostics_to_qt(&self, data: Diagnostics) {
        if self.sender_diagnostics_to_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send diagnostics message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_diagnostics_update_to_qt(&self, data: (Diagnostics, Vec<PackedFileInfo>)) {
        if self.sender_diagnostics_update_to_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send a global search message from the background thread to the main thread.
    #[allow(dead_code)]
    pub fn send_message_global_search_update_to_qt(&self, data: (GlobalSearch, Vec<PackedFileInfo>)) {
        if self.sender_global_search_update_to_qt.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
        }
    }

    /// This function serves to send message from the background thread to the main thread when a PackedFile is saved.
    #[allow(dead_code)]
    pub fn send_message_save_packedfile(&self, data: Response) {
        if self.sender_save_packedfile.send(data).is_err() {
            panic!(THREADS_SENDER_ERROR);
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

    /// This functions serves to receive messages from the main thread into the network thread.
    #[allow(dead_code)]
    pub fn recv_message_qt_to_network(&self) -> Command {
        match self.receiver_qt_to_network.recv() {
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
        let response = self.receiver_qt.recv() ;
        match response {
            Ok(data) => data,
            Err(_) => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This functions serves to receive messages from the network thread into the main thread.
    ///
    /// This function does only try once, and it locks the thread. Use it only in small stuff.
    #[allow(dead_code)]
    pub fn recv_message_network_to_qt(&self) -> Response {
        let response = self.receiver_network_to_qt.recv() ;
        match response {
            Ok(data) => data,
            Err(_) => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_diagnostics_to_qt_try(&self) -> Diagnostics {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_diagnostics_to_qt.try_recv();
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() {
                    panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_diagnostics_update_to_qt_try(&self) -> (Diagnostics, Vec<PackedFileInfo>) {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_diagnostics_update_to_qt.try_recv();
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() {
                    panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_global_search_update_to_qt_try(&self) -> (GlobalSearch, Vec<PackedFileInfo>) {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_global_search_update_to_qt.try_recv();
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() {
                    panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_notification_to_qt_try(&self) -> Response {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_notification_to_qt.try_recv();
            match response {
                Ok(data) => match data{
                    Notification::Done => return Response::Success,
                    Notification::Error(error) => return Response::Error(error),
                }
                Err(error) => if error.is_disconnected() {
                    panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }

    /// This functions serves to receive messages from the background thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_qt_try(&self) -> Response {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_qt.try_recv() ;
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) }
            }
            unsafe { event_loop.process_events_0a() };
        }
    }

    /// This functions serves to receive messages from the network thread into the main thread.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_network_to_qt_try(&self) -> Response {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_network_to_qt.try_recv() ;
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }

    /// This functions serves to receive messages from the network thread into the main thread when a PackedFile is saved.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    #[allow(dead_code)]
    pub fn recv_message_save_packedfile_try(&self) -> Response {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = self.receiver_save_packedfile.try_recv() ;
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }
}
