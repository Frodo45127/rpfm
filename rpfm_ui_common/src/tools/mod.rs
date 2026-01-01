//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::Result;
use getset::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

const TOOLS_FILE: &str = "tools.json";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tools {
    tools: Vec<Tool>,
}

#[derive(Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Tool {
    name: String,
    path: PathBuf,
    games: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Tools {

    pub fn load(custom_config_path: &Option<PathBuf>, fallback_config_path: &Path) -> Result<Self> {
        let path = match custom_config_path {
            Some(path) => path.to_path_buf(),
            None => fallback_config_path.to_path_buf(),
        }.join(TOOLS_FILE);

        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        // Cleanup the loaded order to make sure it's not including not installed packs, or new packs.
        let order: Self = serde_json::from_slice(&data)?;

        Ok(order)
    }

    pub fn save(&self, custom_config_path: &Option<PathBuf>, fallback_config_path: &Path) -> Result<()> {
        let path = match custom_config_path {
            Some(path) => path.to_path_buf(),
            None => fallback_config_path.to_path_buf(),
        }.join(TOOLS_FILE);

        let mut file = BufWriter::new(File::create(path)?);
        file.write_all(to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }
}
