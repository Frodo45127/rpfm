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
Module with the code to parse `Localisable Fields` files.

The `Localisable Fields` files are files that define which field of which table goes to a .loc file when exported.
!*/

use serde_derive::Deserialize;
use serde_xml_rs::from_reader;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rpfm_error::{Result, ErrorKind};

use super::*;

//---------------------------------------------------------------------------//
// Types for parsing the Assembly Kit's TExc_LocalisableFields Files into.
//---------------------------------------------------------------------------//

/// This is the raw equivalent to the `entries` field in a `DB` struct. In files, this is the equivalent to the `.xml` file with all the data in the table.
///
/// It contains a vector with all the rows of data in the `.xml` table file.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "dataroot")]
pub struct RawLocalisableFields {

    #[serde(rename = "TExc_LocalisableFields")]
    pub fields: Vec<RawLocalisableField>,
}

/// This is the raw equivalent to a `DecodedData`.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "datafield")]
pub struct RawLocalisableField {
    pub table_name: String,
    pub field: String,
}

//---------------------------------------------------------------------------//
// Implementations
//---------------------------------------------------------------------------//

/// Implementation of `RawLocalisableFields`.
impl RawLocalisableFields {

    /// This function tries to parse a Raw Assembly Kit Localisable Fields Table to memory.
    pub fn read(raw_data_path: &Path, version: i16) -> Result<Self> {
        match version {
            2 | 1 => {
                let localisable_fields_path = get_raw_localisable_fields_path(raw_data_path, version)?;
                let localisable_fields_file = BufReader::new(File::open(&localisable_fields_path)?);
                from_reader(localisable_fields_file).map_err(From::from)
            }
            _ => Err(ErrorKind::AssemblyKitUnsupportedVersion(version).into())
        }
    }
}
