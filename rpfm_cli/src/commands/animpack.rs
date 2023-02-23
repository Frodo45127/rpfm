//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the `AnimPack` command functions.

use anyhow::Result;

use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::{Path, PathBuf};

use rpfm_lib::binary::ReadBytes;
use rpfm_lib::files::{animpack::AnimPack, ContainerPath, Container, Decodeable, DecodeableExtraData, Encodeable, EncodeableExtraData};
use rpfm_lib::integrations::log::*;
use rpfm_lib::utils::last_modified_time_from_file;

use crate::config::Config;

//---------------------------------------------------------------------------//
//                          AnimPack Command Variants
//---------------------------------------------------------------------------//

/// This function list the contents of the provided AnimPack.
pub fn list(config: &Config, path: &Path) -> Result<()> {

    if config.verbose {
        info!("Listing AnimPack Contents.");
    }

    let mut reader = BufReader::new(File::open(path)?);
    let path_str = path.to_str().unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_disk_file_path(Some(path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let pack = AnimPack::decode(&mut reader, &Some(extra_data))?;
    let files: BTreeMap<_, _> = pack.files().iter().collect();
    for (path, _) in files {
        println!("{path}");
    }

    Ok(())
}

/// This function creates a new empty AnimPack with the provided path.
pub fn create(config: &Config, path: &Path) -> Result<()> {
    if config.verbose {
        info!("Creating new empty AnimPack at {}.", path.to_string_lossy().to_string());
    }

    let mut file = BufWriter::new(File::create(path)?);
    let mut pack = AnimPack::default();
    pack.encode(&mut file, &None).map_err(From::from)
}

/// This function adds the provided files/folders to the provided AnimPack.
pub fn add(config: &Config, pack_path: &Path, file_path: &[(PathBuf, String)], folder_path: &[(PathBuf, String)]) -> Result<()> {
    if config.verbose {
        info!("Adding files/folders to a AnimPack at {}.", pack_path.to_string_lossy().to_string());
    }

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = AnimPack::decode(&mut reader, &Some(extra_data))?;

    for (folder_path, container_path) in folder_path {
        pack.insert_folder(folder_path, container_path, &None, &None)?;
    }

    for (file_path, container_path) in file_path {
        pack.insert_file(file_path, container_path, &None)?;
    }

    pack.preload()?;

    let mut writer = BufWriter::new(File::create(pack_path)?);
    pack.encode(&mut writer, &None)?;

    if config.verbose {
        info!("Files/folders added.");
    }

    Ok(())
}

/// This function deletes the provided files/folders from the provided AnimPack.
pub fn delete(config: &Config, pack_path: &Path, file_path: &[String], folder_path: &[String]) -> Result<()> {
    if config.verbose {
        info!("Delete files/folders from a AnimPack at {}.", pack_path.to_string_lossy().to_string());
    }

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = AnimPack::decode(&mut reader, &Some(extra_data))?;

    let mut container_paths = folder_path.iter().map(|x| ContainerPath::Folder(x.to_string())).collect::<Vec<_>>();
    container_paths.append(&mut file_path.iter().map(|x| ContainerPath::File(x.to_string())).collect::<Vec<_>>());
    let container_paths = ContainerPath::dedup(&container_paths);

    for container_path in container_paths {
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

/// This function extracts the provided files/folders from the provided AnimPack, keeping their folder structure.
pub fn extract(config: &Config, pack_path: &Path, file_path: &[(String, PathBuf)], folder_path: &[(String, PathBuf)]) -> Result<()> {
    if config.verbose {
        info!("Extracting files/folders from a AnimPack at {}.", pack_path.to_string_lossy().to_string());
    }

    let pack_path_str = pack_path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(pack_path)?);
    let mut extra_data = DecodeableExtraData::default();

    extra_data.set_disk_file_path(Some(&pack_path_str));
    extra_data.set_timestamp(last_modified_time_from_file(reader.get_ref())?);
    extra_data.set_data_size(reader.len()?);

    let mut pack = AnimPack::decode(&mut reader, &Some(extra_data))?;
    let mut extra_data = EncodeableExtraData::default();
    if let Some(game) = &config.game {
        extra_data.set_game_key(Some(game.game_key_name()));
    }

    let extra_data = Some(extra_data);

    for (container_path, folder_path) in folder_path {
        let container_path = ContainerPath::Folder(container_path.to_owned());
        pack.extract(container_path, folder_path, true, &None, false, &extra_data)?;
    }

    for (container_path, file_path) in file_path {
        let container_path = ContainerPath::File(container_path.to_owned());
        pack.extract(container_path, file_path, true, &None, false, &extra_data)?;
    }

    if config.verbose {
        info!("Files/folders extracted.");
    }

    Ok(())
}

