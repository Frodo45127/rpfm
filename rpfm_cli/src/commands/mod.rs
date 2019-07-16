//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use clap::ArgMatches;

use rpfm_error::{ErrorKind, Result};

use crate::config::Config;

mod table;
mod packfile;

//---------------------------------------------------------------------------//
// 								Command Variants
//---------------------------------------------------------------------------//

/// This function triggers functions that require the `PackFile` command.
pub fn command_packfile(config: &Config, matches: &ArgMatches) -> Result<()> { 
    match matches.value_of("packfile") {
        Some(packfile_path) => {
		    if matches.is_present("add") {
				match matches.values_of("add") {
					Some(mut values) => packfile::add_to_packfile(&config, packfile_path, values.nth(0).unwrap(), values.nth(0)), 
					None => Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))?
				}
		    }

		    else if matches.is_present("delete") {
				match matches.values_of("delete") {
					Some(mut values) => packfile::delete_file_from_packfile(&config, packfile_path, values.nth(0).unwrap()), 
					None => Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))?
				}
		    }

			else if matches.is_present("list") { packfile::list_packfile_contents(&config, packfile_path) }
			else { Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))? }
        },
        None => Err(ErrorKind::NoHTMLError("No PackFile provided.".to_owned()))?,
    }
}

/// This function triggers functions that require the `Table` command.
pub fn command_table(config: &Config, matches: &ArgMatches) -> Result<()> { 
    if matches.is_present("import") {
		match matches.values_of("import") {
			Some(mut values) => table::import_tsv(&config, values.nth(0).unwrap(), values.nth(0)), 
			None => Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))?
		}
    }

    else if matches.is_present("export") {
		match matches.values_of("export") {
			Some(mut values) => table::export_tsv(&config, values.nth(0).unwrap(), values.nth(0)), 
			None => Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))?
		}
    }
	
	//else if matches.is_present("export") { packfile::list_packfile_contents(config, packfile_path) }
	else { Err(ErrorKind::NoHTMLError("No valid argument provided.".to_owned()))? }
}