//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the notes of the open packs: comments attached to their files and folders.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Done, Request};

/// A note of a pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NoteEntry {

    /// ID of the note, unique within its path.
    pub id: u64,

    /// Path of the file or folder the note is attached to. Empty for the whole pack.
    pub path: String,

    /// Text of the note.
    pub message: String,

    /// Link attached to the note, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// `notes.list`: returns the notes of an open pack for a path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListNotes {

    /// Key of the pack.
    pub pack: String,

    /// Path of the file or folder whose notes to return, with the notes of the folders containing it.
    /// Empty for all the notes of the pack.
    #[serde(default)]
    pub path: String,
}

/// Notes of a pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NoteList {

    /// The notes.
    pub notes: Vec<NoteEntry>,
}

/// `notes.add`: attaches a note to a file or folder of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddNote {

    /// Key of the pack.
    pub pack: String,

    /// Path of the file or folder to attach the note to. Empty for the whole pack.
    #[serde(default)]
    pub path: String,

    /// Text of the note.
    pub message: String,

    /// Link to attach to the note.
    #[serde(default)]
    pub url: Option<String>,

    /// ID of a note attached to the same path, to replace it instead of adding a new one.
    #[serde(default)]
    pub id: Option<u64>,
}

/// `notes.delete`: deletes a note of an open pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteNote {

    /// Key of the pack.
    pub pack: String,

    /// Path the note is attached to.
    pub path: String,

    /// ID of the note.
    pub id: u64,
}

impl Request for ListNotes {
    const METHOD: &'static str = "notes.list";
    type Response = NoteList;
}

impl Request for AddNote {
    const METHOD: &'static str = "notes.add";
    type Response = NoteEntry;
}

impl Request for DeleteNote {
    const METHOD: &'static str = "notes.delete";
    type Response = Done;
}
