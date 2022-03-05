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
use ron::de::from_str;
use sentry::Envelope;
use sentry::Level;
use sentry::protocol::{Attachment, EnvelopeItem, Event};

use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;

use rpfm_macros::*;

use crate::GAME_SELECTED;
use crate::SENTRY_GUARD;
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
pub struct SchemaPatch{

    /// It stores a list of per-table, per-column patches.
    tables: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `SchemaPatches`.
impl SchemaPatches {

    /// This function loads a `SchemaPatches` to memory from a file in the `schemas/` folder.
    pub fn load() -> Result<Self> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
        file_path.push(SCHEMA_PATCHES_FILE);

        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        from_bytes(&data).map_err(From::from)
    }

    /// This function saves a `SchemaPatches` from memory to a file in the `schemas/` folder.
    pub fn save(&mut self) -> Result<()> {
        let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(SCHEMA_PATCHES_FILE);
        let mut file = File::create(&file_path)?;
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function imports a schema patch into the currently loaded patchset, using the Game Selected to choose what schema to patch.
    pub fn import(&mut self, patch: SchemaPatch) -> Result<()> {
        let game_selected = GAME_SELECTED.read().unwrap().get_game_key_name();
        match self.patches.get_mut(&game_selected) {

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
            None => { self.patches.insert(game_selected, patch); },
        }

        self.save()
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
    pub fn upload(&self) -> Result<()> {
        if SENTRY_GUARD.read().unwrap().is_enabled() {
            let mut event = Event::new();
            event.level = Level::Info;
            event.message = Some(format!("Summited Schema Patch for: {}.", GAME_SELECTED.read().unwrap().get_display_name()));

            let config = PrettyConfig::default();
            let mut data = vec![];
            to_writer_pretty(&mut data, &self, config)?;

            let mut envelope = Envelope::from(event);
            let attatchment = Attachment {
                buffer: data,
                filename: "patch.ron".to_owned(),
                ty: None
            };

            envelope.add_item(EnvelopeItem::Attachment(attatchment));
            SENTRY_GUARD.read().unwrap().send_envelope(envelope);
        }

        // TODO: Make this fail in case of sentry being not working?
        Ok(())
    }
}
