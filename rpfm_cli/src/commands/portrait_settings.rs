//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `PortraitSettings` command functions.

use anyhow::Result;

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Write};
use std::path::Path;

use rpfm_lib::files::{Decodeable, Encodeable, portrait_settings::PortraitSettings};
use rpfm_lib::integrations::log::*;

use crate::config::Config;

//---------------------------------------------------------------------------//
//                     PortraitSettings Command Variants
//---------------------------------------------------------------------------//

pub fn to_json(config: &Config, bin_path: &Path, json_path: &Path) -> Result<()> {
    if config.verbose {
        info!("Converting PortraitSettings to Json…");
    }

    let mut bin_file = BufReader::new(File::open(bin_path)?);
    let mut bin_data = vec![];
    bin_file.read_to_end(&mut bin_data)?;

    let mut reader = Cursor::new(bin_data);
    let data = PortraitSettings::decode(&mut reader, &None)?;
    let json_data = data.to_json()?;

    let mut json_file = BufWriter::new(File::create(json_path)?);
    json_file.write_all(json_data.as_bytes())?;

    if config.verbose {
        info!("PortraitSettings converted to Json.");
    }

    Ok(())
}

pub fn from_json(config: &Config, json_path: &Path, bin_path: &Path) -> Result<()> {
    if config.verbose {
        info!("Converting Json to PortraitSettings…");
    }

    let mut str_file = BufReader::new(File::open(json_path)?);
    let mut str_data = String::new();
    str_file.read_to_string(&mut str_data)?;

    let mut data = PortraitSettings::from_json(&str_data)?;
    let mut bin_data = vec![];
    data.encode(&mut bin_data, &None)?;

    let mut bin_file = BufWriter::new(File::create(bin_path)?);
    bin_file.write_all(&bin_data)?;

    if config.verbose {
        info!("Json converted to PortraitSettings.");
    }

    Ok(())
}
