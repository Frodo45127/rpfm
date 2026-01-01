//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

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
    /// This command is used to close a thread.
    Exit,

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

    /// This command is used when we want to get the data used to build the `TreeView`.
    GetPackFileDataForTreeView,

    /// Same as the one before, but for the extra `PackFile`. It requires the pathbuf of the PackFile.
    GetPackFileExtraDataForTreeView(PathBuf),

    /// This command is used to open one or more `PackFiles`. It requires the paths of the `PackFiles`.
    OpenPackFiles(Vec<PathBuf>),

    /// This command is used to open an extra `PackFile`. It requires the path of the `PackFile`.
    OpenPackExtra(PathBuf),

    /// This command is used to open all the CA PackFiles for the game selected as one.
    LoadAllCAPackFiles,

    /// This command is used when we want to get the `RFileInfo` of one or more `PackedFiles`.
    GetPackedFilesInfo(Vec<String>),

    /// This command is used when we want to perform a `Global Search`. It requires the search info.
    GlobalSearch(GlobalSearch),

    /// This command is used when we want to change the `Game Selected`. It contains the name of the game to select, and if we should rebuild the dependencies.
    SetGameSelected(String, bool),

    /// This command is used when we want to change the `Type` of the currently open `PackFile`. It contains the new type.
    SetPackFileType(PFHFileType),

    /// This command is used when we want to generate the dependencies cache for a game. It contains the path of the
    /// source raw db files, the `Raw DB Version` of the currently selected game, and if we should has the files or not.
    GenerateDependenciesCache,

    /// This command is used when we want to update the currently loaded Schema with data from the game selected's Assembly Kit.
    /// It contains the path of the source files, if needed.
    UpdateCurrentSchemaFromAssKit,

    /// This command is used when we want to trigger an optimization pass over the currently open `PackFile`.
    OptimizePackFile(OptimizerOptions),

    /// This command is used to patch the SiegeAI of a Siege Map for warhammer games.
    PatchSiegeAI,

    /// This command is used when we want to change the `Index Includes Timestamp` flag in the currently open `PackFile`
    ChangeIndexIncludesTimestamp(bool),

    /// This command is used when we want to change the `Data is Compressed` flag in the currently open `PackFile`
    ChangeCompressionFormat(CompressionFormat),

    /// This command is used when we want to know the current path of our currently open `PackFile`.
    GetPackFilePath,

    /// This command is used when we want to get the info of the provided `PackedFile`.
    GetRFileInfo(String),

    /// This command is used when we want to check if there is an RPFM update available.
    CheckUpdates,

    /// This command is used when we want to check if there is an Schema update available.
    CheckSchemaUpdates,

    /// This command is used when we want to update our schemas.
    UpdateSchemas,

    /// This command is used when we want to know if there is a Dependency Database loaded in memory.
    ///
    /// Pass true if you want to ensure the dependencies were built with the AssKit.
    IsThereADependencyDatabase(bool),

    /// This command is used when we want to create a new `PackedFile` inside the currently open `PackFile`.
    ///
    /// It requires the path of the new PackedFile, and the `NewPackedFile` with the new PackedFile's info.
    NewPackedFile(String, NewFile),

    /// This command is used when we want to add one or more Files to our currently open `PackFile`.
    ///
    /// It requires the list of filesystem paths to add, and their path once they're inside the `PackFile`.
    AddPackedFiles(Vec<PathBuf>, Vec<ContainerPath>, Option<Vec<PathBuf>>),

    /// This command is used when we want to decode a PackedFile to be shown on the UI. It contains the path of the file, and were it is.
    DecodePackedFile(String, DataSource),

    // This command is used when we want to save an edited `PackedFile` back to the `PackFile`.
    SavePackedFileFromView(String, RFileDecoded),

    // This command is used when we want to add a PackedFile from one PackFile into another.
    AddPackedFilesFromPackFile((PathBuf, Vec<ContainerPath>)),

    // This command is used when we want to add a PackedFile from our PackFile to an Animpack.
    AddPackedFilesFromPackFileToAnimpack(String, Vec<ContainerPath>),

    // This command is used when we want to add a PackedFile from an AnimPack to our PackFile.
    AddPackedFilesFromAnimpack(DataSource, String, Vec<ContainerPath>),

    // This command is used when we want to delete a PackedFile from an AnimPack.
    DeleteFromAnimpack((String, Vec<ContainerPath>)),

    // This command is used when we want to delete one or more PackedFiles from a PackFile. It contains the ContainerPath of each PackedFile to delete.
    DeletePackedFiles(Vec<ContainerPath>),

    // This command is used when we want to extract one or more PackedFiles from a PackFile. It contains the ContainerPaths to extract and the extraction path, and a bool to know if tables must be exported to tsv on extract or not.
    ExtractPackedFiles(BTreeMap<DataSource, Vec<ContainerPath>>, PathBuf, bool),

    // This command is used when we want to rename one or more PackedFiles in a PackFile. It contains a Vec with their original ContainerPath and their new name.
    RenamePackedFiles(Vec<(ContainerPath, ContainerPath)>),

    /// This command is used when we want to know if a folder exists in the currently open PackFile.
    FolderExists(String),

    /// This command is used when we want to know if a PackedFile exists in the currently open PackFile.
    PackedFileExists(String),

    /// This command is used when we want to get the table names (the folder of the tables) of all DB files in our dependency PackFiles.
    GetTableListFromDependencyPackFile,
    GetCustomTableList,
    LocalArtSetIds,
    DependenciesArtSetIds,

    /// This command is used when we want to get the version of the table provided that's compatible with the version of the game we currently have installed.
    GetTableVersionFromDependencyPackFile(String),

    // This command is used when we want to get the definition of the table provided that's compatible with the version of the game we currently have installed.
    GetTableDefinitionFromDependencyPackFile(String),

    /// This command is used when we want to merge multiple compatible tables into one. The contents of this are as follows:
    /// - `Vec<Vec<String>>`: List of paths to merge.
    /// - String: Path of the merged file.
    /// - Bool: Should we delete the source files after merging them?
    MergeFiles(Vec<ContainerPath>, String, bool),

    // This command is used when we want to update a table to a newer version.
    UpdateTable(ContainerPath),

    /// This command is used when we want to replace some specific matches in a Global Search.
    GlobalSearchReplaceMatches(GlobalSearch, Vec<MatchHolder>),

    /// This command is used when we want to replace all matches in a Global Search.
    GlobalSearchReplaceAll(GlobalSearch),

    /// This command is used to decode all tables referenced by columns in the provided definition and return their data.
    /// It requires the table name, the definition of the table to get the reference data from and the list of PackedFiles to ignore.
    GetReferenceDataFromDefinition(String, Definition, bool),

    /// This command is used to get the list of PackFiles that are marked as dependency of our PackFile.
    GetDependencyPackFilesList,

    /// This command is used to set the list of PackFiles that are marked as dependency of our PackFile.
    SetDependencyPackFilesList(Vec<(bool, String)>),

    /// This command is used to get a full list of PackedFile from all known sources to the UI. Requires the path of the PackedFile.
    GetRFilesFromAllSources(Vec<ContainerPath>, bool),

    // This command is used to change the format of a ca_vp8 video packedfile. Requires the path of the PackedFile and the new format.
    SetVideoFormat(String, SupportedFormats),

    // This command is used to save the provided schema to disk.
    SaveSchema(Schema),

    /// This command is used to save to encoded data the cache of the provided paths, and then clean up the cache.
    CleanCache(Vec<ContainerPath>),

    /// This command is used to export a table as TSV. Requires the internal and destination paths for the PackedFile.
    ExportTSV(String, PathBuf, DataSource),

    /// This command is used to import a TSV as a table. Requires the internal and destination paths for the PackedFile.
    ImportTSV(String, PathBuf),

    /// This command is used to open in the defaul file manager the folder of the currently open PackFile.
    OpenContainingFolder,

    /// This command is used to open a PackedFile on a external program. Requires the internal path of the PackedFile.
    OpenPackedFileInExternalProgram(DataSource, ContainerPath),

    /// This command is used to save a PackedFile from an external program. Requires both, internal and external paths of the PackedFile.
    SavePackedFileFromExternalView(String, PathBuf),

    /// This command is used to update the program to the last version available, if possible.
    UpdateMainProgram,

    /// This command is used to trigger an autosave to a backup from time to time.
    TriggerBackupAutosave,

    /// This command is used to trigger a full diagnostics check over the open PackFile.
    DiagnosticsCheck(Vec<String>, bool),

    // This command is used to trigger a partial diagnostics check over the open PackFile.
    DiagnosticsUpdate(Diagnostics, Vec<ContainerPath>, bool),

    /// This command is used to get the settings of the currently open PackFile.
    GetPackSettings,

    // This command is used to set the settings of the currently open PackFile.
    SetPackSettings(PackSettings),

    /// This command is used to trigger the debug missing table definition's code.
    GetMissingDefinitions,

    /// This command is used to rebuild the dependencies of a PackFile. The bool is for rebuilding the whole dependencies, or just the mod-specific ones.
    RebuildDependencies(bool),

    /// This command is used to trigger a cascade edition on all referenced data.
    CascadeEdition(String, Definition, Vec<(Field, String, String)>),

    /// This command is used for the Go To Definition feature. Contains table, column, and values to search.
    GoToDefinition(String, String, Vec<String>),

    /// This command is used to get the source data of a loc key. Contains the loc key to search.
    GetSourceDataFromLocKey(String),

    /// This command is used to get the loc file/column/row of a key. Contains the loc key to search.
    GoToLoc(String),

    /// This command is used for the Find References feature. Contains list of table/columns to search, and value to search.
    SearchReferences(HashMap<String, Vec<String>>, String),

    // This command is used to get the type of a File.
    //GetFileType(String),
    /// This command is used to get the name of the currently open PackFile.
    GetPackFileName,

    /// This command is used to get the raw data of a PackedFile.
    GetPackedFileRawData(String),

    /// This command is used to import files from the dependencies into out PackFile.
    ImportDependenciesToOpenPackFile(BTreeMap<DataSource, Vec<ContainerPath>>),

    /// This command is used to save all provided PackedFiles into the current PackFile, then merge them and optimize them if possible.
    SavePackedFilesToPackFileAndClean(Vec<RFile>, bool),

    /// This command is used to get all the file names under a path in all dependencies.
    GetPackedFilesNamesStartingWitPathFromAllSources(ContainerPath),

    /// This command is used to request all notes under a path, no matter their source.
    NotesForPath(String),

    SaveLocalSchemaPatch(HashMap<String, DefinitionPatch>),
    RemoveLocalSchemaPatchesForTable(String),
    RemoveLocalSchemaPatchesForTableAndField(String, String),

    /// This command is used to import a schema patch in the local schema patches.
    ImportSchemaPatch(HashMap<String, DefinitionPatch>),

    /// This command is used to generate all missing loc entries for the currently open PackFile.
    GenerateMissingLocData,

    /// This command is used to check for updates on the tw_autogen thing.
    CheckLuaAutogenUpdates,

    /// This command is used to update the tw_autogen thing.
    UpdateLuaAutogen,

    /// This command is used to initialize a MyMod Folder.
    InitializeMyModFolder(String, String, bool, bool, Option<String>),

    AddNote(Note),
    DeleteNote(String, u64),
    LiveExport,
    PackMap(Vec<PathBuf>, Vec<(PathBuf, String)>),
    AddLineToPackIgnoredDiagnostics(String),

    CheckEmpireAndNapoleonAKUpdates,
    UpdateEmpireAndNapoleonAK,
    GetPackTranslation(String),
    BuildStarpos(String, bool),
    BuildStarposPost(String, bool),
    BuildStarposCleanup(String, bool),
    BuildStarposGetCampaingIds,
    BuildStarposCheckVictoryConditions,
    UpdateAnimIds(i32, i32),
    GetAnimPathsBySkeletonName(String),
    CheckTranslationsUpdates,
    UpdateTranslations,
    GetTablesFromDependencies(String),
    GetTablesByTableName(String),
    AddKeysToKeyDeletes(String, String, HashSet<String>),
    ExportRigidToGltf(RigidModel, String),

    // Settings commands - for accessing settings from the UI
    SettingsGetBool(String),
    SettingsGetI32(String),
    SettingsGetF32(String),
    SettingsGetString(String),
    SettingsGetPathBuf(String),
    SettingsGetVecString(String),
    SettingsGetVecRaw(String),
    SettingsSetBool(String, bool),
    SettingsSetI32(String, i32),
    SettingsSetF32(String, f32),
    SettingsSetString(String, String),
    SettingsSetPathBuf(String, PathBuf),
    SettingsSetVecString(String, Vec<String>),
    SettingsSetVecRaw(String, Vec<u8>),

    ConfigPath,
    AssemblyKitPath,
    BackupAutosavePath,
    OldAkDataPath,
    SchemasPath,
    TableProfilesPath,
    TranslationsLocalPath,
    DependenciesCachePath,
    SettingsClearPath(PathBuf),
    BackupSettings,
    ClearSettings,
    RestoreBackupSettings,
    OptimizerOptions,
}

/// This enum defines the responses (messages) you can send to the to the UI thread as result of a command.
///
/// Each response should be named after the types of the items it carries.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Generic response for situations of success.
    Success,

    /// Generic response for situations that returned an error.
    Error(String),

    /// Response to return (bool).
    Bool(bool),

    /// Response to return (i32).
    I32(i32),

    /// Response to return (PathBuf).
    PathBuf(PathBuf),

    /// Response to return (String)
    String(String),
    OptionContainerPath(Option<ContainerPath>),

    // Response to return (ContainerInfo, Vec<RFileInfo>).
    ContainerInfoVecRFileInfo((ContainerInfo, Vec<RFileInfo>)),

    // Response to return (ContainerInfo).
    ContainerInfo(ContainerInfo),

    // Response to return (Option<RFileInfo>).
    OptionRFileInfo(Option<RFileInfo>),

    // Response to return (Vec<Option<RFileInfo>>).
    VecRFileInfo(Vec<RFileInfo>),

    // Response to return (GlobalSearch, Vec<RFileInfo>).
    GlobalSearchVecRFileInfo(Box<GlobalSearch>, Vec<RFileInfo>),

    // Response to return (`Vec<Vec<String>>`).
    //VecVecString(Vec<Vec<String>>),

    // Response to return (Vec<ContainerPath>).
    VecContainerPath(Vec<ContainerPath>),

    // Response to return (Vec<(ContainerPath, Vec<String>)>).
    VecContainerPathContainerPath(Vec<(ContainerPath, ContainerPath)>),

    // Response to return (String, `Vec<Vec<String>>`).
    //StringVecVecString((String, Vec<Vec<String>>)),
    /// Response to return `APIResponse`.
    APIResponse(APIResponse),

    /// Response to return `APIResponseGit`.
    APIResponseGit(GitResponse),

    /// Response to return `(AnimFragmentBattle, RFileInfo)`.
    AnimFragmentBattleRFileInfo(AnimFragmentBattle, RFileInfo),

    AnimPackRFileInfo(Vec<RFileInfo>, RFileInfo),

    /// Response to return `(AnimTable, RFileInfo)`.
    AnimsTableRFileInfo(AnimsTable, RFileInfo),
    AtlasRFileInfo(Atlas, RFileInfo),
    AudioRFileInfo(Audio, RFileInfo),
    UnitVariantRFileInfo(UnitVariant, RFileInfo),

    /// Response to return `(CaVp8, RFileInfo)`.
    VideoInfoRFileInfo(VideoInfo, RFileInfo),

    /// Response to return `(ESF, RFileInfo)`.
    ESFRFileInfo(ESF, RFileInfo),

    #[allow(dead_code)]
    BmdRFileInfo(Box<Bmd>, RFileInfo),

    /// Response to return `(Image, RFileInfo)`.
    ImageRFileInfo(Image, RFileInfo),

    /// Response to return `(Text, RFileInfo)`.
    TextRFileInfo(Text, RFileInfo),
    VMDRFileInfo(Text, RFileInfo),
    WSModelRFileInfo(Text, RFileInfo),

    /// Response to return `(DB, RFileInfo)`.
    DBRFileInfo(DB, RFileInfo),

    /// Response to return `(Loc, RFileInfo)`.
    LocRFileInfo(Loc, RFileInfo),

    /// Response to return `(MatchedCombat, RFileInfo)`.
    MatchedCombatRFileInfo(MatchedCombat, RFileInfo),
    PortraitSettingsRFileInfo(PortraitSettings, RFileInfo),

    /// Response to return `(RigidModel, RFileInfo)`.
    RigidModelRFileInfo(RigidModel, RFileInfo),

    /// Response to return `(UIC, RFileInfo)`.
    UICRFileInfo(UIC, RFileInfo),
    GroupFormationsRFileInfo(GroupFormations, RFileInfo),

    //UnitVariantRFileInfo(UnitVariant, RFileInfo),

    // Response to return `(DecodedPackedFile, RFileInfo)`. For debug views.
    //RFileDecodedRFileInfo(RFileDecoded, RFileInfo),
    /// Response to return `Text`.
    Text(Text),

    /// Response to return `Unknown`.
    Unknown,

    /// Response to return `Vec<String>`.
    VecString(Vec<String>),

    /// Response to return `(i32, i32)`.
    I32I32(i32, i32),

    /// Response to return `BTreeMap<i32, DependencyData>`.
    HashMapI32TableReferences(HashMap<i32, TableReferences>),

    /// Response to return `PackFileSettings`.
    PackSettings(PackSettings),

    /// Response to return `DataSource, Vec<String>, usize, usize`.
    DataSourceStringUsizeUsize(DataSource, String, usize, usize),

    /// Response to return `Vec<(DataSource, Vec<String>, String, usize, usize)>`.
    VecDataSourceStringStringUsizeUsize(Vec<(DataSource, String, String, usize, usize)>),

    /// Response to return `Option<(String, String, Vec<String>)>`.
    OptionStringStringVecString(Option<(String, String, Vec<String>)>),

    /// Response to return `Vec<u8>`.
    VecU8(Vec<u8>),

    /// Response to return `DependenciesInfo`.
    DependenciesInfo(DependenciesInfo),

    RFileDecoded(RFileDecoded),

    /// Response to return `HashMap<DataSource, HashMap<Vec<String>, PackedFile>>`.
    HashMapDataSourceHashMapStringRFile(HashMap<DataSource, HashMap<String, RFile>>),
    Diagnostics(Diagnostics),
    Definition(Definition),
    HashMapDataSourceHashSetContainerPath(HashMap<DataSource, HashSet<ContainerPath>>),
    VecNote(Vec<Note>),
    Note(Note),
    HashSetString(HashSet<String>),
    HashSetStringHashSetString(HashSet<String>, HashSet<String>),
    StringVecContainerPath(String, Vec<ContainerPath>),
    VecContainerPathVecRFileInfo(Vec<ContainerPath>, Vec<RFileInfo>),
    VecContainerPathVecContainerPath(Vec<ContainerPath>, Vec<ContainerPath>),
    StringVecPathBuf(String, Vec<PathBuf>),
    PackTranslation(PackTranslation),
    VecRFile(Vec<RFile>),
    VecBoolString(Vec<(bool, String)>),
    CompressionFormat(CompressionFormat),
    I32I32VecStringVecString(i32, i32, Vec<String>, Vec<String>),
    CompressionFormatDependenciesInfo(CompressionFormat, Option<DependenciesInfo>),

    // Settings responses - for returning settings values to the UI
    SettingsBool(bool),
    SettingsI32(i32),
    SettingsF32(f32),
    SettingsString(String),
    SettingsPathBuf(PathBuf),
    SettingsVecString(Vec<String>),
    SettingsVecRaw(Vec<u8>),

    OptimizerOptions(OptimizerOptions),
}
