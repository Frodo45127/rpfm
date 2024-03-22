//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `Pack` command functions.

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::diagnostics::Diagnostics;

use rpfm_lib::binary::ReadBytes;
use rpfm_lib::files::{ContainerPath, Container, Decodeable, DecodeableExtraData, Encodeable, EncodeableExtraData, FileType, pack::Pack};
use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;
use rpfm_lib::utils::last_modified_time_from_file;

use crate::config::Config;

//---------------------------------------------------------------------------//
// 							Pack Command Variants
//---------------------------------------------------------------------------//

/// This function changes the Pack Type of the provided Pack.
pub fn set_pack_type(config: &Config, pack_path: &Path, pfh_file_type: PFHFileType) -> Result<()> {
    if config.verbose {
        info!("Changing PFH Type for Pack at {}.", pack_path.to_string_lossy().to_string());
    }

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = Pack::decode(&mut reader, &Some(extra_data))?;
    pack.preload()?;
    pack.set_pfh_file_type(pfh_file_type);

    let mut writer = BufWriter::new(File::create(pack_path)?);
    pack.encode(&mut writer, &None)?;

    if config.verbose {
        info!("Changed PFH Type to {}.", pfh_file_type);
    }

    Ok(())
}

/// This function list the contents of the provided Pack.
pub fn list(config: &Config, path: &Path) -> Result<()> {

    if config.verbose {
		info!("Listing Pack Contents.");
	}

    let mut reader = BufReader::new(File::open(path)?);
    let path_str = path.to_str().unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_disk_file_path(Some(path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let pack = Pack::decode(&mut reader, &Some(extra_data))?;
    let files: BTreeMap<_, _> = pack.files().iter().collect();
    for (path, _) in files {
        println!("{path}");
    }

	Ok(())
}

/// This function creates a new empty Pack with the provided path.
pub fn create(config: &Config, path: &Path) -> Result<()> {
    if config.verbose {
        info!("Creating new empty Mod Pack at {}.", path.to_string_lossy().to_string());
    }

    match &config.game {
        Some(game) => {
            let mut file = BufWriter::new(File::create(path)?);
            let mut pack = Pack::new_with_version(game.pfh_version_by_file_type(PFHFileType::Mod));
            pack.encode(&mut file, &None)?;
            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}

/// This function adds the provided files/folders to the provided Pack.
pub fn add(config: &Config, schema_path: &Option<PathBuf>, pack_path: &Path, file_path: &[(PathBuf, String)], folder_path: &[(PathBuf, String)]) -> Result<()> {
    if config.verbose {
        info!("Adding files/folders to a Pack at {}.", pack_path.to_string_lossy().to_string());
        info!("Tsv to Binary is: {}.", schema_path.is_some());
    }

    // Load the schema if we try to import tsv files.
    let schema = if let Some(schema_path) = schema_path {
        if schema_path.is_file() {
            Some(Schema::load(schema_path, None)?)
        } else {
            warn!("Schema path provided, but it doesn't point to a valid schema. Disabling `TSV to Binary`.");
            None
        }
    } else { None };

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = Pack::decode(&mut reader, &Some(extra_data))?;

    for (folder_path, container_path) in folder_path {
        if config.verbose {
            info!("Adding folder: {}", container_path);
        }

        pack.insert_folder(folder_path, container_path, &None, &schema, false)?;
    }

    for (file_path, container_path) in file_path {
        if config.verbose {
            info!("Adding file: {}", container_path);
        }

        pack.insert_file(file_path, container_path, &schema)?;
    }

    pack.preload()?;

    let mut writer = BufWriter::new(File::create(pack_path)?);
    pack.encode(&mut writer, &None)?;

    if config.verbose {
        info!("Files/folders added.");
    }

    Ok(())
}

/// This function deletes the provided files/folders from the provided Pack.
pub fn delete(config: &Config, pack_path: &Path, file_path: &[String], folder_path: &[String]) -> Result<()> {
    if config.verbose {
        info!("Delete files/folders from a Pack at {}.", pack_path.to_string_lossy().to_string());
    }

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = Pack::decode(&mut reader, &Some(extra_data))?;

    let mut container_paths = folder_path.iter().map(|x| ContainerPath::Folder(x.to_string())).collect::<Vec<_>>();
    container_paths.append(&mut file_path.iter().map(|x| ContainerPath::File(x.to_string())).collect::<Vec<_>>());
    let container_paths = ContainerPath::dedup(&container_paths);

    for container_path in container_paths {
        if config.verbose {
            info!("Deleting path: {}", container_path.path_raw());
        }

        pack.remove(&container_path);
    }

    pack.preload()?;

    let mut writer = BufWriter::new(File::create(pack_path)?);
    pack.encode(&mut writer, &None)?;

    if config.verbose {
        info!("Files/folders deleted.");
    }

    Ok(())
}

/// This function extracts the provided files/folders from the provided Pack, keeping their folder structure.
pub fn extract(config: &Config, schema_path: &Option<PathBuf>, pack_path: &Path, file_path: &[(String, PathBuf)], folder_path: &[(String, PathBuf)]) -> Result<()> {
    if config.verbose {
        info!("Extracting files/folders from a Pack at {}.", pack_path.to_string_lossy().to_string());
        info!("Tables as Tsv is: {}.", schema_path.is_some());
    }

    // Load the schema if we try to import tsv files.
    let schema = if let Some(schema_path) = schema_path {
        if schema_path.is_file() {
            Some(Schema::load(schema_path, None)?)
        } else {
            warn!("Schema path provided, but it doesn't point to a valid schema. Disabling `Table as TSV`.");
            None
        }
    } else { None };

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = Pack::decode(&mut reader, &Some(extra_data))?;
    let mut extra_data = EncodeableExtraData::default();
    if let Some(game) = &config.game {
        extra_data = EncodeableExtraData::new_from_game_info(game);
    }

    let extra_data = Some(extra_data);

    for (container_path, folder_path) in folder_path {
        if config.verbose {
            info!("Extracting folder: {}", container_path);
        }

        let container_path = ContainerPath::Folder(container_path.to_owned());
        pack.extract(container_path, folder_path, true, &schema, false, true, &extra_data)?;
    }

    for (container_path, file_path) in file_path {
        if config.verbose {
            info!("Extracting file: {}", container_path);
        }

        let container_path = ContainerPath::File(container_path.to_owned());
        pack.extract(container_path, file_path, true, &schema, false, true, &extra_data)?;
    }

    if config.verbose {
        info!("Files/folders extracted.");
    }

    Ok(())
}


/// This function diagnose problems in the provided Packs.
pub fn diagnose(config: &Config, game_path: &Path, pak_path: &Path, schema_path: &Path, pack_paths: &[PathBuf]) -> Result<()> {
    if config.verbose {
        info!("Diagnosing problems in the following Packs:");
        for pack_path in pack_paths {
            info!(" - {}", pack_path.to_string_lossy().to_string());
        }
    }

    // Load both, the schema and the Packs to memory.
    let schema = Schema::load(schema_path, None)?;
    let mut pack = Pack::read_and_merge(pack_paths, true, false)?;

    // Prepare the table's extra data,
    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_schema(Some(&schema));
    let table_extra_data = Some(extra_data);

    // Decode the tables and locs from the packs, and return a list of the decoded tables for later.
    let tables = pack.files_by_type_mut(&[FileType::DB, FileType::Loc])
        .par_iter_mut()
        .filter_map(|file| {
            let _ = file.decode(&table_extra_data, true, false);
            if file.file_type() == FileType::DB {
                Some(file.path_in_container_split()[1].to_owned())
            } else {
                None
            }
        }).collect::<Vec<String>>();

    match &config.game {
        Some(game_info) => {

            // Build the dependencies cache for the game and generate the references for our specific Pack.
            let mut dependencies = Dependencies::default();
            dependencies.rebuild(&Some(schema), pack.dependencies(), Some(pak_path), game_info, game_path)?;
            dependencies.generate_local_db_references(&pack, &tables);

            // Trigger a diagnostics check.
            let mut diagnostics = Diagnostics::default();
            diagnostics.check(&mut pack, &mut dependencies, game_info, game_path, &[], false);

            if config.verbose {
                info!("Diagnosed problems in the following Packs:");
                for pack_path in pack_paths {
                    info!(" - {}", pack_path.to_string_lossy().to_string());
                }
                println!("Verbose mode detected. Marking beginning: ----------------------------");
            }

            println!("{}", diagnostics.json()?);

            if config.verbose {
                println!("----------------------------");
            }

            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}

/// This function merges the provided Packs into a new one, and saves it to the provided save path.
pub fn merge(config: &Config, save_pack_path: &Path, source_pack_paths: &[PathBuf]) -> Result<()> {
    if config.verbose {
        info!("Creating new Merged Mod Pack at {}.", save_pack_path.to_string_lossy().to_string());
        info!("Packs ready to be merged:");
        for source_pack_path in source_pack_paths {
            info!(" - {}", source_pack_path.to_string_lossy().to_string());
        }
    }

    match &config.game {
        Some(game) => {
            let mut pack = Pack::read_and_merge(source_pack_paths, true, false)?;
            pack.save(Some(save_pack_path), game, &None)?;
            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}

/// This function adds a dependency to the provided Pack.
pub fn add_dependency(config: &Config, pack_path: &Path, dependency: &str) -> Result<()> {
    if config.verbose {
        info!("Adding a dependency ({}) to the following Pack: {}", dependency, pack_path.file_name().unwrap().to_string_lossy());
    }

    match &config.game {
        Some(game) => {
            let mut pack = Pack::read_and_merge(&[pack_path.to_path_buf()], true, false)?;
            pack.dependencies_mut().push(dependency.to_owned());
            pack.save(None, game, &None)?;
            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}

/// This function removes a dependency from the provided Pack.
pub fn remove_dependency(config: &Config, pack_path: &Path, dependency: &str) -> Result<()> {
    if config.verbose {
        info!("Removing a dependency ({}) from the following Pack: {}", dependency, pack_path.file_name().unwrap().to_string_lossy());
    }

    match &config.game {
        Some(game) => {
            let mut pack = Pack::read_and_merge(&[pack_path.to_path_buf()], true, false)?;
            if let Some(pos) = pack.dependencies().iter().position(|x| x == dependency) {
                pack.dependencies_mut().remove(pos);
            }
            pack.save(None, game, &None)?;
            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}

/// This function removes all dependencies from the provided Pack.
pub fn remove_all_dependencies(config: &Config, pack_path: &Path) -> Result<()> {
    if config.verbose {
        info!("Removing all dependencies from the following Pack: {}", pack_path.file_name().unwrap().to_string_lossy());
    }

    match &config.game {
        Some(game) => {
            let mut pack = Pack::read_and_merge(&[pack_path.to_path_buf()], true, false)?;
            pack.dependencies_mut().clear();
            pack.save(None, game, &None)?;
            Ok(())
        }
        None => Err(anyhow!("No Game provided.")),
    }
}
