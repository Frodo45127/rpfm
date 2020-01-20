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
Module with all the code to interact with the Assembly Kit's DB Files.

This module contains all the code needed to parse Assembly Kit's DB files to a format we can understand.
!*/

use rayon::prelude::*;
use regex::Regex;
use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::borrow::BorrowMut;
use std::fs::{File, DirBuilder, read_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use rpfm_error::{Result, Error, ErrorKind};

use crate::{DEPENDENCY_DATABASE, GAME_SELECTED, SCHEMA, SUPPORTED_GAMES};
use crate::common::*;
use crate::packfile::PackFile;
use crate::packedfile::table::DecodedData;
use crate::packedfile::table::db::DB;
use crate::schema::*;

const LOCALISABLE_FILES_FILE_NAME_V2: &str = "TExc_LocalisableFields";

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit DB Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to the `entries` field in a `DB` struct. In files, this is the equivalent to the `.xml` file with all the data in the table.
///
/// It contains a vector with all the rows of data in the `.xml` table file.
#[derive(Debug, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawTable {
    pub rows: Vec<RawTableRow>,
}

/// This is the raw equivalent to a row of data from a DB file.
#[derive(Debug, Deserialize)]
#[serde(rename = "datarow")]
pub struct RawTableRow {

    #[serde(rename = "datafield")]
    pub fields: Vec<RawTableField>,
}

/// This is the raw equivalent to a `DecodedData`.
#[derive(Debug, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawTableField {
    pub field_name: String,

    #[serde(rename = "$value")]
    pub field_data: String,
}
