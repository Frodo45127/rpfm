//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module for managing user notes attached to PackFile entries.
//!
//! This module provides functionality for creating and managing notes that users can attach
//! to specific files within a PackFile or as global notes for the entire PackFile.
//!
//! # Usage
//!
//! Notes can be:
//! - **Path-specific**: Attached to a particular file path within the PackFile
//! - **Global**: Applied to the entire PackFile (when `path` is empty)
//!
//! Each note has a unique ID, message body, and optional URL for external references.

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a user note attached to a PackFile or a specific file within it.
///
/// Notes provide a way for users to annotate their work, leave reminders, or document
/// specific files or aspects of their PackFile modifications.
///
/// # Examples
///
/// Creating a global note:
/// ```ignore
/// # use rpfm_lib::notes::Note;
/// let note = Note {
///     id: 1,
///     message: "Remember to test this before release".to_string(),
///     url: None,
///     path: String::new(),  // Empty path = global note
/// };
/// ```
///
/// Creating a path-specific note:
/// ```ignore
/// # use rpfm_lib::notes::Note;
/// let note = Note {
///     id: 2,
///     message: "This table needs balancing".to_string(),
///     url: Some("https://wiki.example.com/balance".to_string()),
///     path: "db/units_tables/core_units".to_string(),
/// };
/// ```
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Note {

    /// Unique identifier for this note.
    ///
    /// Used to distinguish between different notes and for referencing specific notes.
    id: u64,

    /// The main content/body of the note.
    ///
    /// Contains the user's message, reminder, or documentation text.
    message: String,

    /// Optional URL associated with the note.
    ///
    /// Can be used to link to external documentation, wiki pages, issue trackers, etc.
    url: Option<String>,

    /// Path within the PackFile where this note applies.
    ///
    /// - Empty string: Global note that applies to the entire PackFile
    /// - Non-empty: Path to a specific file (e.g., `"db/units_tables/core_units"`)
    path: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl Note {}
