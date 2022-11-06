//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `Schema` command functions.

use anyhow::Result;

use std::path::Path;

use rpfm_lib::integrations::{git::*, log::*};
use rpfm_lib::schema::*;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							Schema Command Variants
//---------------------------------------------------------------------------//

/// This function downloads the most recent schemas into the provided path.
pub fn update(config: &Config, schema_path: &Path) -> Result<()> {
	if config.verbose {
		info!("Updating schemas…");
	}

    let git_integration = GitIntegration::new(schema_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE);
    git_integration.update_repo()?;

    if config.verbose {
        info!("Schemas updated.");
    }

    Ok(())
}

/*
pub fn to_json(config: &Config) -> Result<()> {
    if config.verbosity_level > 0 {
        info!("Converting schemas to Json…");
    }


    let result = Schema::export_to_json();
    if config.verbosity_level > 0 {
        info!("Schemas converted to Json.");
    }
    result
}

pub fn to_xml(config: &Config) -> Result<()> {
    if config.verbosity_level > 0 {
        info!("Converting schemas to XML…");
    }


    let result = Schema::export_to_xml();
    if config.verbosity_level > 0 {
        info!("Schemas converted to XML.");
    }
    result
}
*/
