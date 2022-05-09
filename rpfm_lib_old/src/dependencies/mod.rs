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
Module with all the code related to the `Dependencies`.

This module contains the code needed to manage the dependencies of the currently open PackFile.
!*/

use rayon::prelude::*;
use rayon::iter::Either;
use serde_derive::{Serialize, Deserialize};
use unicase::UniCase;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryFrom;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};


use rpfm_common::utils::*;
use rpfm_error::{Result, Error, ErrorKind};
use rpfm_macros::*;

use crate::assembly_kit::table_data::RawTable;
use crate::DB;
use crate::GAME_SELECTED;
use crate::games::VanillaDBTableNameLogic;
use crate::packfile::{PackFile, PathType};
use crate::packfile::packedfile::PackedFile;
use crate::packfile::packedfile::PackedFileInfo;
use crate::packfile::packedfile::CachedPackedFile;
use crate::packedfile::{DecodedPackedFile, PackedFileType};
use crate::packedfile::table::DependencyData;
use crate::SCHEMA;
use crate::settings::get_config_path;

const BINARY_EXTENSION: &str = "pak2";
pub const DEPENDENCIES_FOLDER: &str = "dependencies";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the dependency data for the different features within RPFM.
///
/// As even I am getting a bit confused by how this works (and it has caused a few bugs):
/// - First, these three are loaded with the data to load files from the game and parent files.
///     - parent_cached_packed_files.
///     - vanilla_cached_packed_files.
///     - asskit_only_db_tables.
/// - Then, every table and loc is preloaded here:
///     - parent_packed_files_cache.
///     - vanilla_packed_files_cache.
/// - Then we build the "Path" cache:
///     - vanilla_cached_packed_files_paths.
///     - parent_cached_packed_files_paths.
///     - vanilla_cached_folders_caseless.
///     - parent_cached_folders_caseless.
///     - vanilla_cached_folders_cased.
///     - parent_cached_folders_cased.
///
/// - Then, on runtime, we add decoded table's dependencies data to this one, so we don't need to recalculate it again.
///     - cached_data,
#[derive(Default, Debug, Clone, GetRef, GetRefMut, Serialize, Deserialize)]
pub struct Dependencies {

    /// Date of the generation of this dependencies cache. For checking if it needs an update.
    build_date: i64,

    /// Cached data for already checked tables. This is for runtime caching, and it must not be serialized to disk.
    #[serde(skip_serializing, skip_deserializing)]
    cached_data: Arc<RwLock<BTreeMap<String, BTreeMap<i32, DependencyData>>>>,

    /// Cache for vanilla decoded files, so we don't re-decode them.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_packed_files_cache: Arc<RwLock<HashMap<String, PackedFile>>>,

    /// Cache for parent decoded files, so we don't re-decode them.
    #[serde(skip_serializing, skip_deserializing)]
    parent_packed_files_cache: Arc<RwLock<HashMap<String, PackedFile>>>,

    /// Data to quickly load CA dependencies from disk.
    vanilla_cached_packed_files: HashMap<String, CachedPackedFile>,

    /// Data to quickly load dependencies from parent mods from disk.
    #[serde(skip_serializing, skip_deserializing)]
    parent_cached_packed_files: HashMap<String, CachedPackedFile>,

    /// Data to quickly check if a path exists in the vanilla files as a case insensitive file.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_cached_packed_files_paths: LazyLoadedData<HashSet<UniCase<String>>>,

    /// Data to quickly check if a path exists in the parent mod files as a case insensitive file.
    #[serde(skip_serializing, skip_deserializing)]
    parent_cached_packed_files_paths: LazyLoadedData<HashSet<UniCase<String>>>,

    /// Data to quickly check if a path exists in the vanilla files as a case insensitive folder.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_cached_folders_caseless: LazyLoadedData<HashSet<UniCase<String>>>,

    /// Data to quickly check if a path exists in the parent mod files as a case insensitive folder.
    #[serde(skip_serializing, skip_deserializing)]
    parent_cached_folders_caseless: LazyLoadedData<HashSet<UniCase<String>>>,

    /// Data to quickly check if a path exists in the vanilla files as a case sensitive folder.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_cached_folders_cased: LazyLoadedData<HashSet<String>>,

    /// Data to quickly check if a path exists in the parent mod files as a case sensitive folder.
    #[serde(skip_serializing, skip_deserializing)]
    parent_cached_folders_cased: LazyLoadedData<HashSet<String>>,

    /// DB Files only available on the assembly kit. Usable only for references. Do not use them as the base for new tables.
    asskit_only_db_tables: Vec<DB>,
}

/// This struct contains the minimal data needed (mainly paths), to know what we have loaded in out dependencies.
#[derive(Debug, Clone, GetRef, GetRefMut)]
pub struct DependenciesInfo {

    /// Full PackedFile-like paths of each asskit-only table.
    pub asskit_tables: Vec<PackedFileInfo>,

    /// Full list of vanilla PackedFile paths.
    pub vanilla_packed_files: Vec<PackedFileInfo>,

    /// Full list of parent PackedFile paths.
    pub parent_packed_files: Vec<PackedFileInfo>,
}

/// This enum is a way to lazy-load parts of the dependencies system just when we need them.
#[derive(Debug, Clone, GetRef, GetRefMut, Serialize, Deserialize)]
pub enum LazyLoadedData<T> {
    Loaded(Box<T>),
    NotYetLoaded
}

impl<T> Default for LazyLoadedData<T> {
    fn default() -> Self {
        Self::NotYetLoaded
    }
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

        // If we only want to reload the parent mods, not the full dependencies, we can skip this section.
        if !only_parent_mods {

            // First, clear the current data, so we're not left with broken data afterwards if the next operations fail.
            *self = Self::default();

            // Try to load the binary file and check if it's even valid.
            let stored_data = Self::load_from_binary()?;
            if !stored_data.needs_updating()? {
                *self = stored_data;
            }
        }

        // Clear the table's cached data, to ensure it gets rebuild properly when needed.
        self.cached_data.write().unwrap().clear();

        // Preload parent mods of the currently open PackFile.
        PackFile::load_custom_dependency_packfiles(&mut self.parent_packed_files_cache.write().unwrap(), &mut self.parent_cached_packed_files, packfile_list);

        // Build the casing-related HashSets.
        self.parent_cached_packed_files_paths = LazyLoadedData::NotYetLoaded;
        self.parent_cached_folders_cased = LazyLoadedData::NotYetLoaded;
        self.parent_cached_folders_caseless = LazyLoadedData::NotYetLoaded;

        // Pre-decode all tables/locs to memory.
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            self.parent_packed_files_cache.write().unwrap().par_iter_mut().for_each(|x| {
                let _ = x.1.decode_no_locks(schema);
            });
        };

        Ok(())
    }

    /// This function generates the entire dependency cache for the currently selected game.
    pub fn generate_dependencies_cache(&mut self, asskit_path: &Option<PathBuf>, version: i16) -> Result<Self> {

        let mut cache = Self::default();
        cache.build_date = current_time();
        cache.vanilla_cached_packed_files = PackFile::open_all_ca_packfiles()?.get_ref_packed_files_all()
            .par_iter()
            .filter_map(|x| CachedPackedFile::new_from_packed_file(*x).ok())
            .map(|x| (x.packed_file_path().to_owned(), x))
            .collect::<HashMap<String, CachedPackedFile>>();

        // This one can fail, leaving the dependencies with only game data.
        // This is needed to support table creation on Empire and Napoleon.
        if let Some(path) = asskit_path {
            let _ = cache.generate_asskit_only_db_tables(path, version);
        }

        Ok(cache)
    }

    /// This function generates a "fake" table list with tables only present in the Assembly Kit.
    ///
    /// This works by processing all the tables from the game's raw table folder and turning them into fake decoded tables,
    /// with version -1. That will allow us to use them for dependency checking and for populating combos.
    ///
    /// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK file.
    fn generate_asskit_only_db_tables(
        &mut self,
        raw_db_path: &Path,
        version: i16,
    ) -> Result<()> {
        let (raw_tables, _) = RawTable::read_all(raw_db_path, version, true, self)?;
        self.asskit_only_db_tables = raw_tables.par_iter().map(From::from).collect::<Vec<DB>>();

        Ok(())
    }

    /// This function returns the db/locs from the cache, according to the params you pass it.
    pub fn get_db_and_loc_tables_from_cache(&self, include_db: bool, include_loc: bool, include_vanilla: bool, include_modded: bool) -> Result<Vec<PackedFile>> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];

            if include_vanilla {
                cache.append(&mut self.vanilla_packed_files_cache.read().unwrap().par_iter().filter_map(|(_, packed_file)| {
                    let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                    if (include_db && packed_file_type == PackedFileType::DB) ||
                        (include_loc && packed_file_type == PackedFileType::Loc) {
                        Some(packed_file.clone())
                    } else {
                        None
                    }
                }).collect())
            }

            if include_modded {
                cache.append(&mut self.parent_packed_files_cache.read().unwrap().par_iter().filter_map(|(_, packed_file)| {
                    let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                    if (include_db && packed_file_type == PackedFileType::DB) ||
                        (include_loc && packed_file_type == PackedFileType::Loc) {
                        Some(packed_file.clone())
                    } else {
                        None
                    }
                }).collect())
            }

            Ok(cache)
        }
    }

    /// This function returns the provided dbs from the cache, according to the params you pass it. Table name must end in _tables.
    pub fn get_db_tables_from_cache(&self, table_name: &str, include_vanilla: bool, include_modded: bool) -> Result<Vec<DB>> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];
            let mut table_folder = "db/".to_owned();
            table_folder.push_str(&table_name.to_lowercase());

            if include_vanilla {
                cache.append(&mut self.vanilla_packed_files_cache.read().unwrap().par_iter().filter_map(|(path, packed_file)| {
                    let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                    if packed_file_type == PackedFileType::DB && path.to_lowercase().starts_with(&table_folder) {
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
                    if packed_file_type == PackedFileType::DB && path.to_lowercase().starts_with(&table_folder) {
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

            Ok(cache)
        }
    }

    /// This function returns the provided dbs from the cache with their full path, according to the params you pass it. Table name must end in _tables.
    pub fn get_db_tables_with_path_from_cache(&self, table_name: &str, include_vanilla: bool, include_modded: bool) -> Result<Vec<(String, DB)>> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];
            let mut table_folder = "db/".to_owned();
            table_folder.push_str(&table_name.to_lowercase());

            if include_vanilla {
                cache.append(&mut self.vanilla_packed_files_cache.read().unwrap().par_iter().filter_map(|(path, packed_file)| {
                    let packed_file_type = PackedFileType::get_packed_file_type(packed_file.get_ref_raw(), false);
                    if packed_file_type == PackedFileType::DB && path.to_lowercase().starts_with(&table_folder) {
                        if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() {
                            Some((path.to_owned(), db.clone()))
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
                    if packed_file_type == PackedFileType::DB && path.to_lowercase().starts_with(&table_folder) {
                        if let Ok(DecodedPackedFile::DB(db)) = packed_file.get_decoded_from_memory() {
                            Some((path.to_owned(), db.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).collect())
            }

            Ok(cache)
        }
    }

    /// This function checks if the current Game Selected has a dependencies file created.
    pub fn game_has_dependencies_generated(&self) -> bool {
        let mut file_path = get_config_path().unwrap().join(DEPENDENCIES_FOLDER);
        file_path.push(GAME_SELECTED.read().unwrap().get_dependencies_cache_file_name());
        file_path.set_extension(BINARY_EXTENSION);

        file_path.is_file()
    }

    /// This function checks if the current Game Selected has the vanilla data loaded in the dependencies.
    ///
    /// TODO: rework this to use the build date, so it's more accurate.
    pub fn game_has_vanilla_data_loaded(&self, include_asskit: bool) -> bool {
        if include_asskit {
            !self.vanilla_packed_files_cache.read().unwrap().is_empty() && self.game_has_asskit_data_loaded()
        } else {
            !self.vanilla_packed_files_cache.read().unwrap().is_empty()
        }
    }

    /// This function checks if the current Game Selected has the asskit data loaded in the dependencies.
    pub fn game_has_asskit_data_loaded(&self) -> bool {
        !self.asskit_only_db_tables.is_empty()
    }

    /// This function loads a `Dependencies` to memory from a file in the `dependencies/` folder.
    pub fn load_from_binary() -> Result<Self> {
        let mut file_path = get_config_path()?.join(DEPENDENCIES_FOLDER);
        file_path.push(GAME_SELECTED.read().unwrap().get_dependencies_cache_file_name());
        file_path.set_extension(BINARY_EXTENSION);

        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        // Never deserialize directly from the file. It's bloody slow!!!
        let mut dependencies: Self = bincode::deserialize(&data).map_err(Error::from)?;

        // Preload all tables/locs to cache.
        if let Some(schema) = &*SCHEMA.read().unwrap() {
            dependencies.vanilla_packed_files_cache.write().unwrap().extend(dependencies.vanilla_cached_packed_files.par_iter()
                .filter_map(|(path, cached_packed_file)| {
                    let packed_file_type = PackedFileType::get_cached_packed_file_type(cached_packed_file, false);
                    if packed_file_type.eq_non_strict_slice(&[PackedFileType::DB, PackedFileType::Loc]) {
                        if let Ok(mut packed_file) = PackedFile::try_from(cached_packed_file) {

                            // Only allow files that actually decode.
                            if packed_file.decode_no_locks(&schema).is_ok() {
                                Some((path.to_owned(), packed_file))
                            } else { None }
                        } else { None }
                    } else { None }
                }).collect::<HashMap<String, PackedFile>>());
        }

        // Build the casing-related HashSets.
        dependencies.vanilla_cached_packed_files_paths = LazyLoadedData::NotYetLoaded;
        dependencies.vanilla_cached_folders_cased = LazyLoadedData::NotYetLoaded;
        dependencies.vanilla_cached_folders_caseless = LazyLoadedData::NotYetLoaded;

        Ok(dependencies)
    }

    /// This function saves a `Dependencies` from memory to a file in the `dependencies/` folder.
    pub fn save_to_binary(&mut self) -> Result<()> {
        let mut file_path = get_config_path()?.join(DEPENDENCIES_FOLDER);
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(GAME_SELECTED.read().unwrap().get_dependencies_cache_file_name());
        file_path.set_extension(BINARY_EXTENSION);
        let mut file = File::create(&file_path)?;

        // Never serialize directly into the file. It's bloody slow!!!
        let serialized: Vec<u8> = bincode::serialize(&self)?;
        file.write_all(&serialized).map_err(From::from)
    }

    /// This function is used to kinda-lazily initialize the vanilla paths from the dependencies checks. This speeds up reloads at the cost of a slight delay later.
    pub fn initialize_vanilla_paths(&mut self) {

        // Build the casing-related HashSets if needed.
        if let LazyLoadedData::NotYetLoaded = self.vanilla_cached_packed_files_paths {
            self.vanilla_cached_packed_files_paths = LazyLoadedData::Loaded(Box::new(self.vanilla_cached_packed_files.par_iter().map(|(key, _)| UniCase::new(key.to_owned())).collect::<HashSet<UniCase<String>>>()));
        }

        if let LazyLoadedData::NotYetLoaded = self.vanilla_cached_folders_cased {
            if let LazyLoadedData::Loaded(vanilla_cached_packed_files_paths) = &mut self.vanilla_cached_packed_files_paths {
                self.vanilla_cached_folders_cased = LazyLoadedData::Loaded(Box::new(vanilla_cached_packed_files_paths.par_iter().map(|x| {
                    let path = x.split('/').collect::<Vec<&str>>();
                    let mut paths = Vec::with_capacity(path.len() - 1);

                    for (index, folder) in path.iter().enumerate() {
                        if index < path.len() - 1 && !folder.is_empty() {
                            paths.push(path[0..=index].join("/"))
                        }
                    }

                    paths
                }).flatten().collect::<HashSet<String>>()));
            }
        }

        if let LazyLoadedData::NotYetLoaded = self.vanilla_cached_folders_caseless {
            if let LazyLoadedData::Loaded(vanilla_cached_folders_cased) = &mut self.vanilla_cached_folders_cased {
                self.vanilla_cached_folders_caseless = LazyLoadedData::Loaded(Box::new(vanilla_cached_folders_cased.par_iter().map(|x| UniCase::new(x.to_owned())).collect::<HashSet<UniCase<String>>>()));
            }
        }
    }

    /// This function is used to kinda-lazily initialize the parent paths from the dependencies checks. This speeds up reloads at the cost of a slight delay later.
    pub fn initialize_parent_paths(&mut self) {

        // Build the casing-related HashSets.
        if let LazyLoadedData::NotYetLoaded = self.parent_cached_packed_files_paths {
            self.parent_cached_packed_files_paths = LazyLoadedData::Loaded(Box::new(self.parent_cached_packed_files.keys().map(|x| UniCase::new(x.to_owned())).collect::<HashSet<UniCase<String>>>()));
        }

        if let LazyLoadedData::NotYetLoaded = self.parent_cached_folders_cased {
            if let LazyLoadedData::Loaded(parent_cached_packed_files_paths) = &mut self.parent_cached_packed_files_paths {
                self.parent_cached_folders_cased = LazyLoadedData::Loaded(Box::new(parent_cached_packed_files_paths.par_iter().map(|x| {
                    let path = x.split('/').collect::<Vec<&str>>();
                    let mut paths = Vec::with_capacity(path.len() - 1);

                    for (index, folder) in path.iter().enumerate() {
                        if index < path.len() - 1 && !folder.is_empty() {
                            paths.push(path[0..=index].join("/"))
                        }
                    }

                    paths
                }).flatten().collect::<HashSet<String>>()));
            }
        }

        if let LazyLoadedData::NotYetLoaded = self.parent_cached_folders_caseless {
            if let LazyLoadedData::Loaded(parent_cached_folders_cased) = &mut self.parent_cached_folders_cased {
                self.parent_cached_folders_caseless = LazyLoadedData::Loaded(Box::new(parent_cached_folders_cased.par_iter().map(|x| UniCase::new(x.to_owned())).collect::<HashSet<UniCase<String>>>()));
            }
        }
    }

    /// This function is used to check if the files RPFM uses to generate the dependencies cache have changed, requiring an update.
    pub fn needs_updating(&self) -> Result<bool> {
        let ca_paths = GAME_SELECTED.read().unwrap().get_all_ca_packfiles_paths()?;
        let last_date = last_modified_time_from_files(&ca_paths)?;
        Ok(last_date > self.build_date)
    }

    /// This function returns the provided file, if exists, or an error if not, from the game files.
    pub fn get_packedfile_from_game_files(&self, path: &[String]) -> Result<PackedFile> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let path = path.join("/");
        let packed_file = self.vanilla_packed_files_cache.read().unwrap().par_iter()
            .find_map_any(|(cached_path, packed_file)| if cached_path == &path { Some(packed_file.clone()) } else { None })
            .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound));

        // If we found it in the cache, return it.
        if packed_file.is_ok() {
            packed_file
        }

        // If not, check on the big list.
        else {
            let packed_file = self.vanilla_cached_packed_files.par_iter()
                .find_map_any(|(cached_path, cache_packed_file)| if cached_path == &path {
                    Some(PackedFile::try_from(cache_packed_file))
                } else { None })
                .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))??;

            // If we found one, add it to the cache to reduce load times later on.
            self.vanilla_packed_files_cache.write().unwrap().insert(path, packed_file.clone());

            Ok(packed_file)
        }
    }

    /// This function returns the provided file, if exists, or an error if not, from the game files. Unicased version.
    pub fn get_packedfile_from_game_files_unicased(&self, path: &[String]) -> Result<PackedFile> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let path = UniCase::new(path.join("/"));
        let packed_file = self.vanilla_packed_files_cache.read().unwrap().par_iter()
            .find_map_any(|(cached_path, packed_file)| if UniCase::new(cached_path) == path { Some(packed_file.clone()) } else { None })
            .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound));

        // If we found it in the cache, return it.
        if packed_file.is_ok() {
            packed_file
        }

        // If not, check on the big list.
        else {
            let packed_file = self.vanilla_cached_packed_files.par_iter()
                .find_map_any(|(cached_path, cache_packed_file)| if UniCase::new(cached_path) == path {
                    Some(PackedFile::try_from(cache_packed_file))
                } else { None })
                .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))??;

            // If we found one, add it to the cache to reduce load times later on.
            self.vanilla_packed_files_cache.write().unwrap().insert(path.to_string(), packed_file.clone());

            Ok(packed_file)
        }
    }

    /// This function returns the provided files, if exist, or an error if not, from the game files.
    pub fn get_packedfiles_from_game_files(&self, paths: &[PathType]) -> Result<(Vec<PackedFile>, Vec<Vec<String>>)> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let mut packed_files = vec![];
        let mut errors = vec![];
        let paths = PathType::dedup(paths);

        for path in paths {
            match path {
                PathType::File(path) => match self.get_packedfile_from_game_files(&path) {
                    Ok(packed_file) => packed_files.push(packed_file),
                    Err(_) => errors.push(path),
                },
                PathType::Folder(base_path) => {
                    let base_path = base_path.join("/");
                    let (mut folder_packed_files, mut error_paths) = self.vanilla_cached_packed_files.par_iter()
                        .filter(|(path, _)| path.starts_with(&base_path) && path.len() > base_path.len())
                        .partition_map(|(path, cached_packed_file)|
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        );

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);

                },
                PathType::PackFile => {
                    let (mut folder_packed_files, mut error_paths) = self.vanilla_cached_packed_files.par_iter()
                        .partition_map(|(path, cached_packed_file)| {
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        });

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);
                },
                PathType::None => unimplemented!(),
            }
        }

        Ok((packed_files, errors))
    }

    /// This function returns the provided files, if exist, or an error if not, from the game files. Unicased version.
    pub fn get_packedfiles_from_game_files_unicased(&self, paths: &[PathType]) -> Result<(Vec<PackedFile>, Vec<Vec<String>>)> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let mut packed_files = vec![];
        let mut errors = vec![];
        let paths = PathType::dedup(paths);

        for path in paths {
            match path {
                PathType::File(path) => match self.get_packedfile_from_game_files_unicased(&path) {
                    Ok(packed_file) => packed_files.push(packed_file),
                    Err(_) => errors.push(path),
                },
                PathType::Folder(base_path) => {
                    let base_path = base_path.join("/");
                    let base_char_len = base_path.chars().count();
                    let base_path_unicased = UniCase::new(&base_path);

                    let (mut folder_packed_files, mut error_paths) = self.vanilla_cached_packed_files.par_iter()
                        .filter(|(path, _)| {
                            if path.len() > base_path.len() {
                                let path_reduced = UniCase::new(path.chars().enumerate().take_while(|(index, _)| index < &base_char_len).map(|(_, c)| c).collect::<String>());
                                path_reduced == base_path_unicased
                            } else { false }
                        })
                        .partition_map(|(path, cached_packed_file)|
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        );

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);

                },
                PathType::PackFile => {
                    let (mut folder_packed_files, mut error_paths) = self.vanilla_cached_packed_files.par_iter()
                        .partition_map(|(path, cached_packed_file)| {
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        });

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);
                },
                PathType::None => unimplemented!(),
            }
        }

        Ok((packed_files, errors))
    }

    /// This function returns all the PackedFiles in the game files of the provided types.
    pub fn get_packedfiles_from_game_files_by_types(&self, packed_file_types: &[PackedFileType], strict_match_mode: bool) -> Result<Vec<PackedFile>> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        Ok(self.vanilla_cached_packed_files.par_iter()
            .filter_map(|(_, cached_packed_file)| {
                let y = PackedFileType::get_cached_packed_file_type(cached_packed_file, false);
                if strict_match_mode {
                    if packed_file_types.contains(&y) {
                        PackedFile::try_from(cached_packed_file).ok()
                    } else {
                        None
                    }
                } else if y.eq_non_strict_slice(packed_file_types) {
                    PackedFile::try_from(cached_packed_file).ok()
                } else {
                    None
                }
            }).collect())
    }

    /// This function returns the provided file, if exists, or an error if not, from the parent mod files.
    pub fn get_packedfile_from_parent_files(&self, path: &[String]) -> Result<PackedFile> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let path = path.join("/");
        let packed_file = self.parent_packed_files_cache.read().unwrap().par_iter()
            .find_map_any(|(cached_path, packed_file)| if cached_path == &path { Some(packed_file.clone()) } else { None })
            .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound));

        // If we found it in the cache, return it.
        if packed_file.is_ok() {
            packed_file
        }

        // If not, check on the big list.
        else {
            let packed_file = self.parent_cached_packed_files.par_iter()
                .find_map_any(|(cached_path, cache_packed_file)| if cached_path == &path {
                    Some(PackedFile::try_from(cache_packed_file))
                } else { None })
                .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))??;

            // If we found one, add it to the cache to reduce load times later on.
            self.parent_packed_files_cache.write().unwrap().insert(path, packed_file.clone());

            Ok(packed_file)
        }
    }

    /// This function returns the provided file, if exists, or an error if not, from the parent mod files. Unicased version.
    pub fn get_packedfile_from_parent_files_unicased(&self, path: &[String]) -> Result<PackedFile> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let path = UniCase::new(path.join("/"));
        let packed_file = self.parent_packed_files_cache.read().unwrap().par_iter()
            .find_map_any(|(cached_path, packed_file)| if UniCase::new(cached_path) == path { Some(packed_file.clone()) } else { None })
            .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound));

        // If we found it in the cache, return it.
        if packed_file.is_ok() {
            packed_file
        }

        // If not, check on the big list.
        else {
            let packed_file = self.parent_cached_packed_files.par_iter()
                .find_map_any(|(cached_path, cache_packed_file)| if UniCase::new(cached_path) == path {
                    Some(PackedFile::try_from(cache_packed_file))
                } else { None })
                .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))??;

            // If we found one, add it to the cache to reduce load times later on.
            self.parent_packed_files_cache.write().unwrap().insert(path.to_string(), packed_file.clone());

            Ok(packed_file)
        }
    }

    /// This function returns the provided files, if exist, or an error if not, from the parent files.
    pub fn get_packedfiles_from_parent_files(&self, paths: &[PathType]) -> Result<(Vec<PackedFile>, Vec<Vec<String>>)> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let mut packed_files = vec![];
        let mut errors = vec![];
        let paths = PathType::dedup(paths);

        for path in paths {
            match path {
                PathType::File(path) => match self.get_packedfile_from_parent_files(&path) {
                    Ok(packed_file) => packed_files.push(packed_file),
                    Err(_) => errors.push(path),
                },
                PathType::Folder(base_path) => {
                    let base_path = base_path.join("/");
                    let (mut folder_packed_files, mut error_paths) =self.parent_cached_packed_files.par_iter()
                        .filter(|(path, _)| path.starts_with(&base_path) && path.len() > base_path.len())
                        .partition_map(|(path, cached_packed_file)|
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        );

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);

                },
                PathType::PackFile => {
                    let (mut folder_packed_files, mut error_paths) = self.parent_cached_packed_files.par_iter()
                        .partition_map(|(path, cached_packed_file)| {
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        });

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);
                },
                PathType::None => unimplemented!(),
            }
        }

        Ok((packed_files, errors))
    }

    /// This function returns the provided files, if exist, or an error if not, from the parent files. Unicased version.
    pub fn get_packedfiles_from_parent_files_unicased(&self, paths: &[PathType]) -> Result<(Vec<PackedFile>, Vec<Vec<String>>)> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        let mut packed_files = vec![];
        let mut errors = vec![];
        let paths = PathType::dedup(paths);

        for path in paths {
            match path {
                PathType::File(path) => match self.get_packedfile_from_parent_files_unicased(&path) {
                    Ok(packed_file) => packed_files.push(packed_file),
                    Err(_) => errors.push(path),
                },
                PathType::Folder(base_path) => {
                    let base_path = base_path.join("/");
                    let base_char_len = base_path.chars().count();
                    let base_path_unicased = UniCase::new(&base_path);

                    let (mut folder_packed_files, mut error_paths) =self.parent_cached_packed_files.par_iter()
                        .filter(|(path, _)| {
                            if path.len() > base_path.len() {
                                let path_reduced = UniCase::new(path.chars().enumerate().take_while(|(index, _)| index < &base_char_len).map(|(_, c)| c).collect::<String>());
                                path_reduced == base_path_unicased
                            } else { false }
                        })
                        .partition_map(|(path, cached_packed_file)|
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        );

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);

                },
                PathType::PackFile => {
                    let (mut folder_packed_files, mut error_paths) = self.parent_cached_packed_files.par_iter()
                        .partition_map(|(path, cached_packed_file)| {
                            match PackedFile::try_from(cached_packed_file) {
                                Ok(packed_file) => Either::Left(packed_file),
                                Err(_) => Either::Right(path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>()),
                            }
                        });

                    packed_files.append(&mut folder_packed_files);
                    errors.append(&mut error_paths);
                },
                PathType::None => unimplemented!(),
            }
        }

        Ok((packed_files, errors))
    }

    /// This function returns all the PackedFiles in the parent files of the provided types.
    pub fn get_packedfiles_from_parent_files_by_types(&self, packed_file_types: &[PackedFileType], strict_match_mode: bool) -> Result<Vec<PackedFile>> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        Ok(self.parent_cached_packed_files.par_iter()
            .filter_map(|(_, cached_packed_file)| {
                let y = PackedFileType::get_cached_packed_file_type(cached_packed_file, false);
                if strict_match_mode {
                    if packed_file_types.contains(&y) {
                        PackedFile::try_from(cached_packed_file).ok()
                    } else {
                        None
                    }
                } else if y.eq_non_strict_slice(packed_file_types) {
                    PackedFile::try_from(cached_packed_file).ok()
                } else {
                    None
                }
            }).collect())
    }

    /// This function returns the provided file, if exists, or an error if not, from the asskit files.
    pub fn get_packedfile_from_asskit_files(&self, path: &[String]) -> Result<DB> {
        if self.needs_updating()? {
            return Err(ErrorKind::DependenciesCacheNotGeneratedorOutOfDate.into());
        }

        // From the asskit we only have tables, so no need for the full path.
        if let Some(table_name) = path.get(1) {
            self.asskit_only_db_tables.par_iter()
            .find_map_any(|x| if x.get_table_name() == *table_name { Some(x.clone()) } else { None })
            .ok_or_else(|| Error::from(ErrorKind::PackedFileNotFound))
        } else { Err(ErrorKind::PackedFileNotFound.into()) }
    }

    /// This function returns the provided file exists on the game files.
    pub fn file_exists_on_game_files(&self, path: &UniCase<String>, case_insensitive: bool) -> bool {
        if case_insensitive {
            if let LazyLoadedData::Loaded(vanilla_cached_packed_files_paths) = &self.vanilla_cached_packed_files_paths {
                vanilla_cached_packed_files_paths.contains(path)
            } else {
                false
            }
        } else {
            self.vanilla_cached_packed_files.contains_key(&**path)
        }
    }

    /// This function returns the provided file exists on the parent mod files.
    pub fn file_exists_on_parent_files(&self, path: &UniCase<String>, case_insensitive: bool) -> bool {
        if case_insensitive {
            if let LazyLoadedData::Loaded(parent_cached_packed_files_paths) = &self.parent_cached_packed_files_paths {
                parent_cached_packed_files_paths.contains(path)
            } else {
                false
            }
        } else {
            self.parent_cached_packed_files.contains_key(&**path)
        }
    }

    /// This function returns the provided folder exists on the game files.
    pub fn folder_exists_on_game_files(&self, path: &UniCase<String>, case_insensitive: bool) -> bool {
        if case_insensitive {
            if let LazyLoadedData::Loaded(vanilla_cached_folders_caseless) = &self.vanilla_cached_folders_caseless {
                vanilla_cached_folders_caseless.contains(path)
            } else {
                false
            }
        } else {
            if let LazyLoadedData::Loaded(vanilla_cached_folders_cased) = &self.vanilla_cached_folders_cased {
                vanilla_cached_folders_cased.contains(&**path)
            } else {
                false
            }
        }
    }

    /// This function returns the provided folder exists on the parent mod files.
    pub fn folder_exists_on_parent_files(&self, path: &UniCase<String>, case_insensitive: bool) -> bool {
        if case_insensitive {
            if let LazyLoadedData::Loaded(parent_cached_folders_caseless) = &self.parent_cached_folders_caseless {
                parent_cached_folders_caseless.contains(path)
            } else {
                false
            }
        } else {
            if let LazyLoadedData::Loaded(parent_cached_folders_cased) = &self.parent_cached_folders_cased {
                parent_cached_folders_cased.contains(&**path)
            } else {
                false
            }
        }
    }

    pub fn get_most_relevant_files_by_paths(&self, paths: &[PathType]) -> Vec<PackedFile> {
        let mut packed_files = vec![];

        for path in paths {
            if let PathType::File(path) = path {

                if let Ok(packed_file) = self.get_packedfile_from_parent_files(&path) {
                    packed_files.push(packed_file);
                }
                else if let Ok(packed_file) = self.get_packedfile_from_game_files(&path) {
                    packed_files.push(packed_file);
                }
            }
        }

        packed_files
    }
}

impl From<&Dependencies> for DependenciesInfo {
    fn from(dependencies: &Dependencies) -> Self {
        let table_name_logic = GAME_SELECTED.read().unwrap().get_vanilla_db_table_name_logic();

        Self {
            asskit_tables: dependencies.asskit_only_db_tables().par_iter().map(|table| {
                let table_name = match table_name_logic {
                    VanillaDBTableNameLogic::DefaultName(ref name) => name.to_owned(),
                    VanillaDBTableNameLogic::FolderName => table.get_table_name(),
                };

                PackedFileInfo::from(&PackedFile::new_from_decoded(&DecodedPackedFile::DB(table.clone()), &["db".to_owned(), table.get_table_name(), table_name]))
            }).collect(),
            vanilla_packed_files: dependencies.vanilla_cached_packed_files().par_iter().map(|(_, cached_packed_file)| PackedFileInfo::from(cached_packed_file)).collect(),
            parent_packed_files:dependencies.parent_cached_packed_files().par_iter().map(|(_, cached_packed_file)| PackedFileInfo::from(cached_packed_file)).collect(),
        }
    }
}
