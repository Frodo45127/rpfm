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
Module with all the code to interact with the Schema Versions File.

This module contains all the code related with the Schema Versions File, used to keep track of Schema updates.
!*/

use reqwest::blocking;
use ron::de::{from_str, from_reader};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use rpfm_error::Result;

use crate::config::get_config_path;
use crate::schema::*;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Versions File in memory, keeping track of the local version of each schema.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct VersionsFile(BTreeMap<String, u32>);

/// This enum controls the possible responses from the server when asking if there is a new Schema update.
///
/// The (Versions, Versions) is local, current.
#[derive(Debug, Serialize, Deserialize)]
pub enum APIResponseSchema {
    SuccessNewUpdate(VersionsFile, VersionsFile),
    SuccessNoLocalUpdate,
    SuccessNoUpdate,
    Error,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `VersionsFile`.
impl VersionsFile {

    /// This function returns a reference to the internal `BTreeMap` of the provided `VersionsFile`.
    pub fn get(&self) -> &BTreeMap<String, u32> {
        &self.0
    }

    /// This function loads the local `VersionsFile` to memory.
    pub fn load() -> Result<Self> {
        let file_path = get_config_path()?.join(SCHEMA_FOLDER).join(SCHEMA_VERSIONS_FILE);
        let file = BufReader::new(File::open(&file_path)?);
        from_reader(file).map_err(From::from)
    }

    /// This function saves the provided `VersionsFile` to disk.
    fn save(&self) -> Result<()> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);

        // Make sure the path exists to avoid problems with updating schemas.
        DirBuilder::new().recursive(true).create(&file_path)?;
        file_path.push(SCHEMA_VERSIONS_FILE);

        let mut file = BufWriter::new(File::create(file_path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function match your local schemas against the remote ones, and downloads any updated ones.
    ///
    /// If no local `VersionsFile` is found, it downloads it from the repo, along with all the schema files.
    pub fn check_update() -> Result<APIResponseSchema> {

        // If there is a local schema, match it against the remote one, to check if there is an update or not.
        let versions_file_url = format!("{}{}", SCHEMA_UPDATE_URL_DEVELOP, SCHEMA_VERSIONS_FILE);
        match Self::load() {
            Ok(local) => {
                let remote: Self = if let Ok(string) = blocking::get(&versions_file_url) {
                    if let Ok(string) = string.text() {
                        if let Ok(remote) = from_str::<Self>(&string) {
                            remote
                        } else { return Ok(APIResponseSchema::Error); }
                    } else { return Ok(APIResponseSchema::Error); }
                } else { return Ok(APIResponseSchema::Error); };

                if local == remote { return Ok(APIResponseSchema::SuccessNoUpdate); }

                for (remote_file_name, remote_version) in &remote.0 {
                    if let Some(local_version) = local.0.get(remote_file_name) {
                        if remote_version > local_version { return Ok(APIResponseSchema::SuccessNewUpdate(local, remote)); }
                    }
                }
            },

            // If there is no local `VersionsFile`, check if we can get them from the repo.
            Err(_) => {
                if let Ok(string) = blocking::get(&versions_file_url) {
                    if let Ok(string) = string.text() {
                        if from_str::<Self>(&string).is_ok() {
                            return Ok(APIResponseSchema::SuccessNoLocalUpdate);
                        }
                    }
                }
                return Ok(APIResponseSchema::Error);
            }
        }

        // If we reached this place, return a "no update found" response.
        Ok(APIResponseSchema::SuccessNoUpdate)
    }

    /// This function match your local schemas against the remote ones, and downloads any updated ones.
    ///
    /// If no local `VersionsFile` is found, it downloads it from the repo, along with all the schema files.
    pub fn update() -> Result<()> {

        // If there is a local schema, match it against the remote one, download the different schemas,
        // then update our local schema with the remote one's data.
        let versions_file_url = format!("{}{}", SCHEMA_UPDATE_URL_DEVELOP, SCHEMA_VERSIONS_FILE);
        match Self::load() {
            Ok(local) => {
                let remote: Self = from_str(&blocking::get(&versions_file_url)?.text()?)?;
                for (remote_file_name, remote_version) in &remote.0 {
                    let schema_url = format!("{}{}", SCHEMA_UPDATE_URL_DEVELOP, remote_file_name);
                    match local.0.get(remote_file_name) {
                        Some(local_version) => {

                            // If it's an update over our own schema, we download it and overwrite the current schema.
                            // NOTE: Github's API has a limit of 1MB per file, so we take it directly from raw.githubusercontent.com instead.
                            if remote_version > local_version {
                                let mut schema: Schema = from_str(&blocking::get(&schema_url)?.text()?)?;
                                schema.save(remote_file_name)?;
                            }
                        }
                        None => {
                            let mut schema: Schema = from_str(&blocking::get(&schema_url)?.text()?)?;
                            schema.save(remote_file_name)?;
                        }
                    }
                }

                local.save()
            },

            // If there is no local `VersionsFile`, download all the schemas, then save the new local `VersionsFile`.
            Err(_) => {
                let local: Self = from_str(&blocking::get(&versions_file_url)?.text()?)?;
                for file_name in local.0.keys() {
                    let mut schema: Schema = from_str(&blocking::get(&format!("{}{}", SCHEMA_UPDATE_URL_DEVELOP, file_name))?.text()?)?;
                    schema.save(file_name)?;
                }
                local.save()
            }
        }
    }
}
