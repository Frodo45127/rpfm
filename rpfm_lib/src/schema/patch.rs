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
Module with all the code to interact with Schema Patches.

These are manual patches made so the autoimporter doesn't break manual fixes.
!*/

use ron::ser::{to_string_pretty, to_writer_pretty, PrettyConfig};
use ron::de::{from_bytes, from_str};

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::BufWriter;
use std::time::{SystemTime, UNIX_EPOCH};

use rpfm_logging::*;
use rpfm_macros::*;

use super::*;

const SCHEMA_PATCHES_FILE: &str = "patches.ron";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a bunch of Schema Patches in memory.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, GetRef, GetRefMut)]
pub struct SchemaPatches {

    /// It stores the patches split by games.
    patches: HashMap<String, SchemaPatch>
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default, GetRef, GetRefMut)]
pub struct SchemaPatch {

    /// It stores a list of per-table, per-column patches.
    tables: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `SchemaPatches`.
impl SchemaPatches {

    /// This function loads a `SchemaPatches` to memory from a file in the `schemas/` folder.
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(&path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }

    /// This function saves a `SchemaPatches` from memory to a file in the `schemas/` folder.
    pub fn save(&mut self, path: &Path) -> Result<()> {
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(&parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(&path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function imports a schema patch into the currently loaded patchset, using the Game Selected to choose what schema to patch.
    pub fn import(&mut self, game_key: &str, patch: SchemaPatch, path: &Path) -> Result<()> {
        match self.patches.get_mut(game_key) {

            // If we have patches fopr that game.
            Some(patches) => {
                let table_name = patch.tables.keys().next().unwrap();
                match patches.tables.get_mut(table_name) {

                    // If we have patches for that table.
                    Some(table_patch) => {
                        let column_name = patch.tables.get(table_name).unwrap().keys().next().unwrap();
                        table_patch.insert(column_name.to_owned(), patch.tables.get(table_name).unwrap().get(column_name).unwrap().clone());
                    }
                    None => { patches.tables.insert(table_name.to_owned(), patch.tables.get(table_name).unwrap().clone()); }
                }
            }
            None => { self.patches.insert(game_key.to_owned(), patch); },
        }

        self.save(path)
    }

    /// This function retireves a value from a schema patch.
    pub fn get_data(&self, game: &str, table_name: &str, column_name: &str, value: &str) -> Option<String> {
        self.patches.get(game)?.tables.get(table_name)?.get(column_name)?.get(value).cloned()
    }
}

/// Implementation of `SchemaPatch`.
impl SchemaPatch {

    /// This function tries to load a patch from a str.
    pub fn load_from_str(patch: &str) -> Result<Self> {
        from_str(&patch).map_err(From::from)
    }

    /// This function uploads a patch to sentry's service.
    pub fn upload(&self, sentry_guard: &ClientInitGuard, game_name: &str) -> Result<()> {
        let level = Level::Info;
        let message = format!("Summited Schema Patch for: {} - {}.", game_name, SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());
        let config = PrettyConfig::default();
        let mut data = vec![];
        to_writer_pretty(&mut data, &self, config)?;
        let file_name = "patch.txt";

        Logger::send_event(sentry_guard, level, &message, Some((&file_name, &data))).map_err(From::from)
    }
}
