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
Module with the code to parse `Localisable Fields` files.

The `Localisable Fields` files are files that define which field of which table goes to a .loc file when exported.
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

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit's TExc_LocalisableFields Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to the `entries` field in a `DB` struct. In files, this is the equivalent to the `.xml` file with all the data in the table.
///
/// It contains a vector with all the rows of data in the `.xml` table file.
#[derive(Debug, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawLocalisableFields {

    #[serde(rename = "TExc_LocalisableFields")]
    pub fields: Vec<RawLocalisableField>,
}

/// This is the raw equivalent to a `DecodedData`.
#[derive(Debug, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawLocalisableField {

    #[serde(rename = "$value")]
    pub table_name: String,

    #[serde(rename = "$value")]
    pub field: String,
}
