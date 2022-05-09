//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `TextMatches`.

This module contains the code needed to get text matches from a `GlobalSearch`.
!*/

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within a text PackedFile.
#[derive(Debug, Clone)]
pub struct TextMatches {

    /// The path of the file.
    pub path: Vec<String>,

    /// The list of matches within the file.
    pub matches: Vec<TextMatch>,
}

/// This struct represents a match on a piece of text within a Text PackedFile.
#[derive(Debug, Clone)]
pub struct TextMatch {

    // Column of the first character of the match.
    pub column: u64,

    // Row of the first character of the match.
    pub row: u64,

    // Length of the matched pattern.
    pub len: i64,

    // Line of text containing the match.
    pub text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TextMatches`.
impl TextMatches {

    /// This function creates a new `TextMatches` for the provided path.
    pub fn new(path: &[String]) -> Self {
        Self {
            path: path.to_vec(),
            matches: vec![],
        }
    }
}

/// Implementation of `TextMatch`.
impl TextMatch {

    /// This function creates a new `TextMatch` with the provided data.
    pub fn new(column: u64, row: u64, len: i64, text: String) -> Self {
        Self {
            column,
            row,
            len,
            text,
        }
    }
}
