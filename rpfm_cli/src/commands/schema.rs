//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use log::info;

use rpfm_error::Result;
use rpfm_lib::schema::Schema;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							Schema Command Variants
//---------------------------------------------------------------------------//

pub fn update(config: &Config) -> Result<()> {
	if config.verbosity_level > 0 {
		info!("Updating schemas…");
	}

	let result = Schema::update_schema_repo();
    if config.verbosity_level > 0 {
        info!("Schemas updated.");
    }
    result
}

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
