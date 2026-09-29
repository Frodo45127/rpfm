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

use rpfm_lib::files::FileType;

use super::{default_true, Request};

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

    /// The tables of the selected game's Assembly Kit. Their paths are `db/<table_name>/ak_data`,
    /// see [`ASSEMBLY_KIT_TABLE_FILE_NAME`].
    AssemblyKit,
}

/// A file in one of the sources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct FileRef {

    /// Where the file is.
    pub source: FileSource,

    /// Path of the file in its source.
    pub path: String,
}

/// `files.list`: lists the files of a source, sorted by path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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

    /// Maximum amount of files to return. Defaults to [`DEFAULT_FILES_LIMIT`].
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
