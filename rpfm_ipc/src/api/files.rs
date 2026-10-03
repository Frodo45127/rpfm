//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the files of the open packs and the dependencies.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use serde_json::Value;

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use rpfm_lib::files::{ContainerPath, FileType, RFile, RFileDecoded, text::{Text, TextFormat}};

use crate::helpers::{DataSource, RFileInfo, VideoInfo};

use super::tools::FilesChanged;

use super::{default_true, Done, Request};

/// File name of the Assembly Kit tables, in paths like `db/<table_name>/ak_data`.
pub const ASSEMBLY_KIT_TABLE_FILE_NAME: &str = "ak_data";

/// Default amount of files returned by [`ListFiles`].
pub const DEFAULT_FILES_LIMIT: usize = 500;

/// Where a file is.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileSource {

    /// An open pack, by key.
    Pack(String),

    /// The vanilla files of the selected game.
    GameFiles,

    /// The files of the parent packs of the open packs.
    ParentFiles,

    /// The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`.
    AssemblyKit,
}

/// A file in one of the sources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileRef {

    /// Where the file is.
    pub source: FileSource,

    /// Path of the file in its source.
    pub path: String,
}

/// `files.list`: lists the files of a source, sorted by path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListFiles {

    /// Source to list.
    pub source: FileSource,

    /// Only files whose path starts with this are listed.
    #[serde(default)]
    pub prefix: String,

    /// If files in subfolders are listed. If `false`, the subfolders are listed instead.
    #[serde(default = "default_true")]
    pub recursive: bool,

    /// If set, only files of these types are listed, like `DB`, `Loc`, `Text`, `Image` or `RigidModel`.
    #[serde(default)]
    #[schemars(with = "Option<Vec<String>>")]
    pub file_types: Option<Vec<FileType>>,

    /// Amount of files to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of files to return. Defaults to 500.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// A page of the files of a source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FileList {

    /// The files in the page.
    pub files: Vec<FileEntry>,

    /// Subfolders directly under the prefix, with their full path. Only filled in non-recursive listings.
    pub folders: Vec<String>,

    /// Amount of files matching the request, in all pages.
    pub total: usize,
}

/// A file in a listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FileEntry {

    /// Path of the file in its source.
    pub path: String,

    /// Type of the file, like `DB`, `Loc`, `Text` or `Image`.
    #[schemars(with = "String")]
    pub file_type: FileType,
}

impl Request for ListFiles {
    const METHOD: &'static str = "files.list";
    type Response = FileList;
}

/// `files.create`: creates a new empty file in an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateFile {

    /// Key of the pack.
    pub pack: String,

    /// Path of the new file in the pack, like `db/units_tables/my_mod` or `text/db/my_mod.loc`.
    pub path: String,

    /// Type of the new file.
    pub kind: NewFileKind,
}

/// Type of a new file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum NewFileKind {

    /// An empty DB table.
    Db {

        /// Name of the table, like `units_tables`.
        table_name: String,

        /// Version of the table. If not set, the version the table has in the game files is used,
        /// or the newest one if the game files are not loaded or don't have the table.
        #[serde(default)]
        version: Option<i32>,
    },

    /// An empty Loc table.
    Loc,

    /// An empty text file.
    Text {

        /// Format of the text, like `Plain`, `Lua`, `Xml` or `Json`. Defaults to `Plain`.
        #[serde(default)]
        #[schemars(with = "Option<String>")]
        format: Option<TextFormat>,
    },

    /// An empty AnimPack.
    AnimPack,

    /// A portrait settings file, optionally with entries copied from the vanilla ones.
    PortraitSettings {

        /// Version of the file format.
        version: u32,

        /// Vanilla entries to copy into the new file, with the ID each copy gets.
        #[serde(default)]
        copy_entries: Vec<EntryCopy>,
    },

    /// An empty VMD (variant mesh definition) file.
    Vmd,

    /// An empty WSModel file.
    WsModel,
}

/// An entry copied from a vanilla file, with a new ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntryCopy {

    /// ID of the vanilla entry.
    pub from: String,

    /// ID of the copy.
    pub to: String,
}

/// `files.add_from_disk`: adds files and folders from disk to an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddFilesFromDisk {

    /// Key of the pack.
    pub pack: String,

    /// Paths on disk of the files and folders to add.
    pub paths: Vec<PathBuf>,

    /// Folder of the pack to add them to. Empty for the root of the pack.
    #[serde(default)]
    pub destination: String,

    /// Path in the pack of each entry of `paths`, in the same order, instead of putting them all in `destination`.
    /// For files, it's their new path. For folders, it's the folder their contents go to.
    #[serde(default)]
    pub destinations: Option<Vec<String>>,

    /// If added folders keep their own name in the pack, instead of adding only their contents. Defaults to the server's setting.
    #[serde(default)]
    pub include_base_folder: Option<bool>,

    /// Files and folders under any of these paths are skipped.
    #[serde(default)]
    pub ignore: Vec<PathBuf>,
}

/// `files.copy`: copies files and folders from any source into an open pack, keeping their paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopyFiles {

    /// Where to copy the files from.
    pub source: FileSource,

    /// Paths of the files and folders to copy.
    pub paths: Vec<String>,

    /// Key of the pack to copy them to.
    pub pack: String,
}

/// Files added to a pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesAdded {

    /// Paths in the pack of the added files.
    pub added: Vec<String>,

    /// Paths of the files that couldn't be added.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_added: Vec<String>,

    /// Last error found while adding the files, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `files.delete`: deletes files and folders from an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteFiles {

    /// Key of the pack.
    pub pack: String,

    /// Paths of the files and folders to delete.
    pub paths: Vec<String>,
}

/// Files deleted from a pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesDeleted {

    /// Paths of the deleted files.
    pub deleted: Vec<String>,
}

/// `files.rename`: renames or moves files and folders of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenameFiles {

    /// Key of the pack.
    pub pack: String,

    /// Renames to apply.
    pub renames: Vec<FileRename>,
}

/// A rename of a file or folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileRename {

    /// Current path.
    pub from: String,

    /// New path.
    pub to: String,
}

/// Files renamed in a pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesRenamed {

    /// Old and new path of each renamed file.
    pub renamed: Vec<FileRename>,
}

/// `files.duplicate`: copies files of an open pack in the same pack, adding a number to their names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DuplicateFiles {

    /// Key of the pack.
    pub pack: String,

    /// Paths of the files and folders to duplicate.
    pub paths: Vec<String>,
}

/// `files.extract`: extracts files and folders of an open pack, the game files or the parent packs to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtractFiles {

    /// Where the files are. The Assembly Kit tables can't be extracted.
    pub source: FileSource,

    /// Paths of the files and folders to extract.
    pub paths: Vec<String>,

    /// Folder on disk to extract them to. Their paths in the source are kept under it.
    pub destination: PathBuf,

    /// If tables are extracted as TSV files instead of binary ones.
    #[serde(default)]
    pub as_tsv: bool,

    /// If TSV files use the old column order, with keys first. Defaults to the server's setting.
    #[serde(default)]
    pub tsv_keys_first: Option<bool>,
}

/// Files extracted to disk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FilesExtracted {

    /// Paths on disk of the extracted files.
    pub extracted: Vec<PathBuf>,
}

impl Request for CreateFile {
    const METHOD: &'static str = "files.create";
    type Response = FileEntry;
}

impl Request for AddFilesFromDisk {
    const METHOD: &'static str = "files.add_from_disk";
    type Response = FilesAdded;
}

impl Request for CopyFiles {
    const METHOD: &'static str = "files.copy";
    type Response = FilesAdded;
}

impl Request for DeleteFiles {
    const METHOD: &'static str = "files.delete";
    type Response = FilesDeleted;
}

impl Request for RenameFiles {
    const METHOD: &'static str = "files.rename";
    type Response = FilesRenamed;
}

impl Request for DuplicateFiles {
    const METHOD: &'static str = "files.duplicate";
    type Response = FilesAdded;
}

impl Request for ExtractFiles {
    const METHOD: &'static str = "files.extract";
    type Response = FilesExtracted;
}

/// `file.read`: returns the contents of a file of any source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadFile {

    /// The file.
    pub file: FileRef,

    /// How to return the contents. Defaults to `decoded`.
    #[serde(default)]
    pub format: ReadFormat,
}

/// How to return the contents of a file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadFormat {

    /// The decoded file, as JSON. For DB and Loc tables, `table.rows` returns only the rows you need,
    /// and for images and other binary files `raw` is smaller.
    #[default]
    Decoded,

    /// The text, for text files like scripts, XML or JSON files.
    Text,

    /// The bytes of the file, encoded in base64.
    Raw,
}

/// Contents of a file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FileData {

    /// The text of a text file.
    Text {

        /// The text.
        text: String,
    },

    /// The decoded file, as JSON, in the format `file.read` returns it.
    Decoded {

        /// The decoded file.
        data: Value,
    },

    /// The bytes of the file.
    Raw {

        /// The bytes, encoded in base64.
        base64: String,
    },
}

/// Contents of a file, with its type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FileContents {

    /// Type of the file, like `Text`, `Image` or `RigidModel`.
    #[schemars(with = "String")]
    pub file_type: FileType,

    /// The contents.
    pub contents: FileData,
}

/// `file.write`: replaces the contents of a file of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WriteFile {

    /// Key of the pack.
    pub pack: String,

    /// Path of the file. With raw contents, the file is created if it doesn't exist.
    pub path: String,

    /// The new contents. Text contents only work on files that are text files.
    pub contents: FileData,
}

/// `animpack.list`: lists the files inside an AnimPack of any source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListAnimPack {

    /// The AnimPack.
    pub file: FileRef,
}

/// `animpack.add`: copies files of an open pack into an AnimPack of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddToAnimPack {

    /// Key of the pack with the AnimPack.
    pub pack: String,

    /// Path of the AnimPack.
    pub animpack: String,

    /// Key of the pack to copy the files from.
    pub from_pack: String,

    /// Paths of the files and folders to copy.
    pub paths: Vec<String>,
}

/// `animpack.extract`: copies files of an AnimPack of any source into an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtractFromAnimPack {

    /// The AnimPack.
    pub file: FileRef,

    /// Paths inside the AnimPack of the files and folders to copy.
    pub paths: Vec<String>,

    /// Key of the pack to copy them to.
    pub to_pack: String,
}

/// `animpack.delete`: deletes files from an AnimPack of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteFromAnimPack {

    /// Key of the pack with the AnimPack.
    pub pack: String,

    /// Path of the AnimPack.
    pub animpack: String,

    /// Paths inside the AnimPack of the files and folders to delete.
    pub paths: Vec<String>,
}

impl Request for ReadFile {
    const METHOD: &'static str = "file.read";
    type Response = FileContents;
}

impl Request for WriteFile {
    const METHOD: &'static str = "file.write";
    type Response = Done;
}

impl Request for ListAnimPack {
    const METHOD: &'static str = "animpack.list";
    type Response = FileList;
}

impl Request for AddToAnimPack {
    const METHOD: &'static str = "animpack.add";
    type Response = FilesAdded;
}

impl Request for ExtractFromAnimPack {
    const METHOD: &'static str = "animpack.extract";
    type Response = FilesAdded;
}

impl Request for DeleteFromAnimPack {
    const METHOD: &'static str = "animpack.delete";
    type Response = Done;
}

/// `file.view_data`: returns a file decoded as a client opening it in a view needs it.
///
/// Meant for clients with views for each type of file. Others should use `file.read`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetViewData {

    /// The file.
    pub file: FileRef,
}

/// A file decoded for a view.
//
// The decoded file is not boxed so clients can match the type of file in the same pattern as the variant.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewData {

    /// The decoded file, and its info.
    Decoded(RFileDecoded, RFileInfo),

    /// The info of the files of an AnimPack, and of the AnimPack itself.
    AnimPack(Vec<RFileInfo>, RFileInfo),

    /// The info of a video, without its frames, and of the file.
    Video(VideoInfo, RFileInfo),

    /// The notes of a pack, as markdown.
    Notes(Text),

    /// The file is of a type that is not decoded.
    Unsupported,
}

impl Request for GetViewData {
    const METHOD: &'static str = "file.view_data";
    type Response = ViewData;
}

/// `files.info`: returns the info of files of an open pack, like their original pack and last modification time.
///
/// Meant for clients showing the files of a pack as a tree. Others should use `files.list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetFilesInfo {

    /// Key of the pack.
    pub pack: String,

    /// Paths of the files. If not set, the info of every file of the pack is returned. Paths without a file are skipped.
    #[serde(default)]
    pub paths: Option<Vec<String>>,
}

/// Info of files of a pack.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesInfo {

    /// The info of each file.
    pub files: Vec<RFileInfo>,
}

impl Request for GetFilesInfo {
    const METHOD: &'static str = "files.info";
    type Response = FilesInfo;
}

/// `files.paste`: copies or moves files and folders of open packs into a folder of an open pack.
///
/// Each selected file or folder is pasted with its own name under the destination folder, not with its full path.
/// Meant for clients with a clipboard. Others should use `files.copy` and `files.rename`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasteFiles {

    /// Paths of the files and folders to paste, by key of their pack.
    pub sources: BTreeMap<String, Vec<String>>,

    /// If the files are removed from their packs, instead of copied.
    pub cut: bool,

    /// Key of the pack to paste them into.
    pub to_pack: String,

    /// Folder of the pack to paste them into. Empty for the root of the pack.
    pub destination: String,
}

/// Result of pasting files.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesPasted {

    /// Paths of the files added to the target pack.
    pub added: Vec<String>,

    /// Paths of the files removed from each source pack, by pack key, if they were cut.
    pub deleted: BTreeMap<String, Vec<String>>,
}

impl Request for PasteFiles {
    const METHOD: &'static str = "files.paste";
    type Response = FilesPasted;
}

/// `files.from_all_sources`: returns files and folders found in the open packs, the parent packs and the game files.
///
/// Meant for clients editing files of several sources at once, like the UI's tools. Others should use `file.read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetFilesFromAllSources {

    /// Paths of the files and folders.
    pub paths: Vec<ContainerPath>,

    /// If the paths of the returned files are lowercased.
    #[serde(default)]
    pub lowercase_paths: bool,
}

/// Files of every source.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesFromAllSources {

    /// The files found, by path, for each source.
    pub files: HashMap<DataSource, HashMap<String, RFile>>,
}

impl Request for GetFilesFromAllSources {
    const METHOD: &'static str = "files.from_all_sources";
    type Response = FilesFromAllSources;
}

/// `files.save_files`: adds files to an open pack, replacing the ones with the same path, optionally optimizing the pack after it.
///
/// Meant for clients editing files outside of the server, like the UI's tools. Others should use `file.write`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveFiles {

    /// Key of the pack.
    pub pack: String,

    /// The files, named with their path in the pack.
    pub files: Vec<RFile>,

    /// If the pack is optimized with the optimizer options of the settings after adding the files.
    #[serde(default)]
    pub optimize: bool,
}

impl Request for SaveFiles {
    const METHOD: &'static str = "files.save_files";
    type Response = FilesChanged;
}

/// `file.open_external`: extracts a file of an open pack to a temporary folder, and opens it in the system's default program for its type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenInExternalProgram {

    /// Key of the pack.
    pub pack: String,

    /// Path of the file.
    pub path: String,
}

/// A file extracted to edit it in an external program.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalFile {

    /// Path of the extracted file on disk.
    pub path: PathBuf,
}

impl Request for OpenInExternalProgram {
    const METHOD: &'static str = "file.open_external";
    type Response = ExternalFile;
}

/// `file.save_external`: replaces a file of an open pack with the file on disk an external program edited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveExternalFile {

    /// Key of the pack.
    pub pack: String,

    /// Path of the file in the pack.
    pub path: String,

    /// Path of the edited file on disk.
    pub external_path: PathBuf,
}

impl Request for SaveExternalFile {
    const METHOD: &'static str = "file.save_external";
    type Response = Done;
}
