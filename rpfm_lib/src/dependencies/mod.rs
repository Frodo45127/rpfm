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
Module with all the code related to the `Dependencies`.

This module contains the code needed to manage the dependencies of the currently open PackFile.
!*/

use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rpfm_macros::*;
use rpfm_error::{Result, Error};

use crate::assembly_kit::table_data::RawTable;
use crate::common::*;
use crate::config::get_config_path;
use crate::DB;
use crate::GAME_SELECTED;
use crate::packfile::PackFile;
use crate::packfile::packedfile::PackedFile;
use crate::packfile::packedfile::CachedPackedFile;
use crate::packedfile::{DecodedPackedFile, PackedFileType};
use crate::packedfile::table::DependencyData;
use crate::SCHEMA;
use crate::SUPPORTED_GAMES;

const BINARY_EXTENSION: &str = ".pak2";
const DEPENDENCIES_FOLDER: &str = "dependencies";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the dependency data for the different features within RPFM.
#[derive(Default, Debug, Clone, GetRef, GetRefMut, Serialize, Deserialize)]
pub struct Dependencies {

    /// Date of the generation of this dependencies cache. For checking if it needs an update.
    build_date: i64,

    /// Cached data for already checked tables. This is for runtime caching, and it must not be serialized to disk.
    #[serde(skip_serializing, skip_deserializing)]
    cached_data: Arc<RwLock<BTreeMap<String, BTreeMap<i32, DependencyData>>>>,

    /// Cache for vanilla decoded files, so we don't re-decode them.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_packed_files_cache: Arc<RwLock<BTreeMap<Vec<String>, PackedFile>>>,

    /// Cache for parent decoded files, so we don't re-decode them.
    #[serde(skip_serializing, skip_deserializing)]
    parent_packed_files_cache: Arc<RwLock<BTreeMap<Vec<String>, PackedFile>>>,

    /// Data to quickly load CA dependencies from disk.
    vanilla_cached_packed_files: Vec<CachedPackedFile>,

    /// Data to quickly load dependencies from parent mods from disk.
    #[serde(skip_serializing, skip_deserializing)]
    parent_cached_packed_files: Vec<CachedPackedFile>,

    /// DB Files only available on the assembly kit. Usable only for references. Do not use them as the base for new tables.
    asskit_only_db_tables: Vec<DB>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Dependencies`.
impl Dependencies {

    /// This function takes care of rebuilding the whole dependencies cache.
    ///
    /// Use it when changing the game selected or opening a new PackFile.
    pub fn rebuild(&mut self, packfile_list: &[String], only_parent_mods: bool) -> Result<()> {
        if let Ok(needs_updating) = self.needs_updating() {
            if !needs_updating {
                if !only_parent_mods {

                    // Clear the dependencies. This is needed because, if we don't clear them here, then overwrite them,
                    // the bastart triggers a memory leak in the next step. Not sure why.
                    self.vanilla_cached_packed_files.clear();
                    self.asskit_only_db_tables.clear();
                    self.cached_data.write().unwrap().clear();
                    self.vanilla_packed_files_cache.write().unwrap().clear();

                    self.vanilla_cached_packed_files = vec![];
                    self.asskit_only_db_tables = vec![];
                    *self.cached_data.write().unwrap() = BTreeMap::new();
                    *self.vanilla_packed_files_cache.write().unwrap() = BTreeMap::new();

                    // Preload the data from the game that only changes on updates.
                    let stored_data = Self::load_from_binary()?;
                    self.vanilla_cached_packed_files = stored_data.vanilla_cached_packed_files;
                    self.vanilla_packed_files_cache = stored_data.vanilla_packed_files_cache;
                    self.asskit_only_db_tables = stored_data.asskit_only_db_tables;

                    // Pre-decode all tables/locs to memory.
                    if let Some(ref schema) = *SCHEMA.read().unwrap() {
                        self.vanilla_packed_files_cache.write().unwrap().par_iter_mut().for_each(|x| {
                            let _ = x.1.decode_no_locks(schema);
                        });
                    }
                }

                // Preload parent mods of the currently open PackFile.
                self.parent_cached_packed_files.clear();
                self.parent_cached_packed_files = vec![];

                self.parent_packed_files_cache.write().unwrap().clear();
                *self.parent_packed_files_cache.write().unwrap() = BTreeMap::new();

                PackFile::load_custom_dependency_packfiles(&mut self.parent_packed_files_cache.write().unwrap(), &mut self.parent_cached_packed_files, packfile_list);

                // Pre-decode all tables/locs to memory.
                if let Some(ref schema) = *SCHEMA.read().unwrap() {
                    self.parent_packed_files_cache.write().unwrap().par_iter_mut().for_each(|x| {
                        let _ = x.1.decode_no_locks(schema);
                    });
                };
            }
        }

        Ok(())
    }

    /// This function generates the entire dependency cache for the currently selected game.
    pub fn generate_dependencies_cache(&mut self, path: &PathBuf, version: i16) -> Result<()> {

        self.build_date = get_current_time();

        if let Ok(pack_file) = PackFile::open_all_ca_packfiles() {
            self.vanilla_cached_packed_files = pack_file.get_ref_packed_files_all().par_iter()
                .filter_map(|x| if let Ok(data) = CachedPackedFile::try_from(*x) { Some(data) } else { None })
                .collect::<Vec<CachedPackedFile>>();

            // Preload all tables/locs to cache.
            if let Some(ref schema) = *SCHEMA.read().unwrap() {
                self.vanilla_packed_files_cache.write().unwrap().append(&mut self.vanilla_cached_packed_files.par_iter()
                    .filter_map(|cached_packed_file| {
                        let packed_file_type = PackedFileType::get_cached_packed_file_type(cached_packed_file, false);
                        if packed_file_type.eq_non_strict_slice(&[PackedFileType::DB, PackedFileType::Loc]) {
                            if let Ok(mut packed_file) = PackedFile::try_from(cached_packed_file) {
                                let _ = packed_file.decode_no_locks(schema);
                                Some((packed_file.get_path().to_vec(), packed_file))
                            } else { None }
                        } else { None }
                    }).collect());
            }
        }

        self.generate_asskit_only_db_tables(&path, version)?;

        Ok(())
    }

    /// This function generates a "fake" table list with tables only present in the Assembly Kit.
    ///
    /// This works by processing all the tables from the game's raw table folder and turning them into fake decoded tables,
    /// with version -1. That will allow us to use them for dependency checking and for populating combos.
    ///
    /// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK file.
    fn generate_asskit_only_db_tables(
        &mut self,
        raw_db_path: &PathBuf,
        version: i16,
    ) -> Result<()> {
        let (raw_tables, _) = RawTable::read_all(raw_db_path, version, true, self)?;
        self.asskit_only_db_tables = raw_tables.par_iter().map(From::from).collect::<Vec<DB>>();

        Ok(())
    }

    /// This function returns the db/locs from the cache, according to the params you pass it.
    pub fn get_db_and_loc_tables_from_cache(&self, include_db: bool, include_loc: bool, include_vanilla: bool, include_modded: bool) -> Vec<PackedFile> {
        let mut cache = vec![];

        if include_vanilla {
            cache.append(&mut self.vanilla_packed_files_cache.read().unwrap().par_iter().filter_map(|(_, packed_file)| {
                let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                if include_db && packed_file_type == PackedFileType::DB {
                    Some(packed_file.clone())
                } else if include_loc && packed_file_type == PackedFileType::Loc {
                    Some(packed_file.clone())
                } else {
                    None
                }
            }).collect())
        }

        if include_modded {
            cache.append(&mut self.parent_packed_files_cache.read().unwrap().par_iter().filter_map(|(_, packed_file)| {
                let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                if include_db && packed_file_type == PackedFileType::DB {
                    Some(packed_file.clone())
                } else if include_loc && packed_file_type == PackedFileType::Loc {
                    Some(packed_file.clone())
                } else {
                    None
                }
            }).collect())
        }

        cache
    }

    /// This function returns the provided dbs from the cache, according to the params you pass it. Table name must end in _tables.
    pub fn get_db_tables_from_cache(&self, table_name: &str, include_vanilla: bool, include_modded: bool) -> Vec<DB> {
        let mut cache = vec![];
        let table_name = table_name.to_lowercase();

        if include_vanilla {
            cache.append(&mut self.vanilla_packed_files_cache.read().unwrap().par_iter().filter_map(|(path, packed_file)| {
                let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                if packed_file_type == PackedFileType::DB && path[1].to_lowercase() == table_name {
                    if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() {
                        Some(db.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect())
        }

        if include_modded {
            cache.append(&mut self.parent_packed_files_cache.read().unwrap().par_iter().filter_map(|(path, packed_file)| {
                let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                if packed_file_type == PackedFileType::DB && path[1].to_lowercase() == table_name {
                    if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() {
                        Some(db.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect())
        }

        cache
    }

    /// This function checks if the current Game Selected has a dependencies file created.
    pub fn game_has_dependencies_generated(&self) -> bool {
        let mut file_path = get_config_path().unwrap().join(DEPENDENCIES_FOLDER);
        file_path.push(SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().pak_file.clone().unwrap());
        file_path.set_extension(BINARY_EXTENSION);

        file_path.is_file()
    }

    /// This function checks if the current Game Selected has the vanilla data loaded in the dependencies.
    pub fn game_has_vanilla_data_loaded(&self) -> bool {
        !self.vanilla_packed_files_cache.read().unwrap().is_empty()
    }

    /// This function checks if the current Game Selected has the asskit data loaded in the dependencies.
    pub fn game_has_asskit_data_loaded(&self) -> bool {
        !self.asskit_only_db_tables.is_empty()
    }

    /// This function loads a `Dependencies` to memory from a file in the `dependencies/` folder.
    pub fn load_from_binary() -> Result<Self> {
        let mut file_path = get_config_path()?.join(DEPENDENCIES_FOLDER);
        file_path.push(SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().pak_file.clone().unwrap());
        file_path.set_extension(BINARY_EXTENSION);

        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        // Never deserialize directly from the file. It's bloody slow!!!
        let dependencies: Self = bincode::deserialize(&data).map_err(|x| Error::from(x))?;

        // Preload all tables/locs to cache.
        dependencies.vanilla_packed_files_cache.write().unwrap().append(&mut dependencies.vanilla_cached_packed_files.par_iter()
            .filter_map(|cached_packed_file| {
                let packed_file_type = PackedFileType::get_cached_packed_file_type(cached_packed_file, false);
                if packed_file_type.eq_non_strict_slice(&[PackedFileType::DB, PackedFileType::Loc]) {
                    if let Ok(packed_file) = PackedFile::try_from(cached_packed_file) {
                        Some((packed_file.get_path().to_vec(), packed_file))
                    } else { None }
                } else { None }
            }).collect());

        Ok(dependencies)
    }

    /// This function saves a `Dependencies` from memory to a file in the `dependencies/` folder.
    pub fn save_to_binary(&mut self) -> Result<()> {
        let mut file_path = get_config_path()?.join(DEPENDENCIES_FOLDER);
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(SUPPORTED_GAMES.get(&**GAME_SELECTED.read().unwrap()).unwrap().pak_file.clone().unwrap());
        file_path.set_extension(BINARY_EXTENSION);
        let mut file = File::create(&file_path)?;

        // Never serialize directly into the file. It's bloody slow!!!
        let serialized: Vec<u8> = bincode::serialize(&self)?;
        file.write_all(&serialized).map_err(From::from)
    }

    /// This function is used to check if the files RPFM uses to generate the dependencies cache have changed, requiring an update.
    pub fn needs_updating(&self) -> Result<bool> {
        let ca_paths = get_all_ca_packfiles_paths()?;
        let last_date = get_last_modified_time_from_files(&ca_paths)?;
        Ok(last_date > self.build_date)
    }
}
