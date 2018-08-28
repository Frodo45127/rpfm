use std::sync::mpsc::Receiver;
use std::path::PathBuf;

use qt_core::event_loop::EventLoop;
use settings::shortcuts::Shortcuts;
use settings::Settings;
use settings::GameSelected;
use packfile::packfile::PackFileExtraData;
use error;


/// This enum is meant for sending commands from the UI Thread to the Background thread.
/// If you want to know what each command do, check the `background_loop` function.
/// If you need to send data, DO NOT USE THIS. Serialize it to Vec<u8> with serde and send it.
#[derive(Debug)]
pub enum Commands {
    ResetPackFile,
    ResetPackFileExtra,
    NewPackFile,
    OpenPackFile,
    OpenPackFileExtra,
    SavePackFile,
    SavePackFileAs(PathBuf),
    SetPackFileType,
    ChangeIndexIncludesTimestamp,
    GetSchema,
    SaveSchema,
    GetSettings,
    SetSettings,
    GetShortcuts,
    SetShortcuts,
    GetGameSelected,
    SetGameSelected,
    GetPackFileHeader,
    GetPackedFilePath,
    IsThereADependencyDatabase,
    IsThereASchema,
    PatchSiegeAI,
    UpdateSchemas,
    AddPackedFile,
    DeletePackedFile,
    ExtractPackedFile,
    GetTypeOfPath,
    PackedFileExists,
    FolderExists,
    CreatePackedFile,
    CreateFolder,
    UpdateEmptyFolders,
    GetPackFileDataForTreeView,
    GetPackFileExtraDataForTreeView,
    AddPackedFileFromPackFile,
    MassImportTSV,
    MassExportTSV,
    DecodePackedFileLoc,
    EncodePackedFileLoc,
    ImportTSVPackedFileLoc,
    ExportTSVPackedFileLoc,
    DecodePackedFileDB,
    EncodePackedFileDB,
    ImportTSVPackedFileDB,
    ExportTSVPackedFileDB,
    DecodePackedFileText,
    EncodePackedFileText,
    DecodePackedFileRigidModel,
    EncodePackedFileRigidModel,
    PatchAttilaRigidModelToWarhammer,
    DecodePackedFileImage,
    RenamePackedFile,
    GetPackedFile,
    GetTableListFromDependencyPackFile,
    GetTableVersionFromDependencyPackFile,
    OptimizePackFile,
}

#[derive(Debug)]
pub enum BackgroundMessage {
    Dummy,
    GetGameSelectedResponse(GameSelected),
    GetShortcutsResponse(Shortcuts),
    GetSettingsResponse(Settings),
    NewPackFileResponse(u32),
    SavePackFileResponse,
    SavePackFileAsResponse(error::Result<((PackFileExtraData, u32))>),
}

pub fn receive_background_message_responsive(receiver: &Receiver<BackgroundMessage>) -> BackgroundMessage { //TODO make actually responsive
    match receiver.recv() {
        Ok(message) => message,
        Err(e) => {
            eprintln!("receive_background_message_responsive {}", e);
            panic!()
        }
    }
}