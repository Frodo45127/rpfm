//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, Setters};
use serde_derive::{Serialize, Deserialize};

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Individual note.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Note {

    /// Unique identifier of the Note.
    id: u64,

    /// Note's main body.
    message: String,

    /// URL associated withe the note.
    url: Option<String>,

    /// Path where this note applies. Empty for global notes.
    path: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

impl Note {}

