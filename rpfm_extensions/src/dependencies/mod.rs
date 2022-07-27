//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains a dependencies system implementation, used to manage dependencies between packs.
/*
use rayon::iter::Either;
use unicase::UniCase;
*/

use getset::*;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::{HashMap, HashSet};

use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{DecodeableExtraData, FileType, RFile, db::DB, pack::Pack};
use rpfm_lib::games::GameInfo;
use rpfm_lib::integrations::table_data::RawTable;
use rpfm_lib::schema::Schema;
use rpfm_lib::utils::{current_time, last_modified_time_from_files};
/*
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
*/
//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents a dependencies manager for all dependencies relevant of a Pack.
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
#[derive(Default, Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
pub struct Dependencies {

    /// Date of the generation of this dependencies cache. For checking if it needs an update.
    build_date: u64,

    //----------------------------------//
    // Cached files.
    //----------------------------------//

    /// Data to quickly load CA dependencies from disk.
    vanilla_files: HashMap<String, RFile>,

    /// Data to quickly load dependencies from parent mods from disk.
    #[serde(skip_serializing, skip_deserializing)]
    parent_files: HashMap<String, RFile>,

/*
    /// Cached data for already checked tables. This is for runtime caching, and it must not be serialized to disk.
    #[serde(skip_serializing, skip_deserializing)]
    cached_data: Arc<RwLock<BTreeMap<String, BTreeMap<i32, DependencyData>>>>,

*/

    /// List of DB tables on the CA files.
    vanilla_tables: HashMap<String, Vec<String>>,

    /// List of DB tables on the parent files.
    parent_tables: HashMap<String, Vec<String>>,

    /// List of Loc tables on the CA files.
    vanilla_locs: HashSet<String>,

    /// List of Loc tables on the parent files.
    parent_locs: HashSet<String>,
/*

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
*/
    /// DB Files only available on the assembly kit. Usable only for references. Do not use them as the base for new tables.
    asskit_only_db_tables: Vec<DB>,
}

/*
/// This struct contains the minimal data needed (mainly paths), to know what we have loaded in out dependencies.
#[derive(Debug, Clone, Getters, MutGetters)]
pub struct DependenciesInfo {

    /// Full PackedFile-like paths of each asskit-only table.
    pub asskit_tables: Vec<PackedFileInfo>,

    /// Full list of vanilla PackedFile paths.
    pub vanilla_packed_files: Vec<PackedFileInfo>,

    /// Full list of parent PackedFile paths.
    pub parent_packed_files: Vec<PackedFileInfo>,
}

/// This enum is a way to lazy-load parts of the dependencies system just when we need them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LazyLoadedData<T> {
    Loaded(Box<T>),
    NotYetLoaded
}

impl<T> Default for LazyLoadedData<T> {
    fn default() -> Self {
        Self::NotYetLoaded
    }
}*/


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Dependencies {

    //-----------------------------------//
    // Generation and disk IO
    //-----------------------------------//

    /// This function takes care of rebuilding the whole dependencies cache.
    ///
    /// Use it when changing the game selected or opening a new PackFile.
    pub fn rebuild(&mut self, schema: &Schema, pack_names: &[String], file_path: Option<&Path>, game_info: &GameInfo, game_path: &Path) -> Result<()> {

        // If we only want to reload the parent mods, not the full dependencies, we can skip this section.
        if let Some(file_path) = file_path {

            // First, clear the current data, so we're not left with broken data afterwards if the next operations fail.
            *self = Self::default();

            // Try to load the binary file and check if it's even valid.
            let stored_data = Self::load(file_path, schema)?;
            if !stored_data.needs_updating(game_info, game_path)? {
                *self = stored_data;
            }
        }

        // Clear the table's cached data, to ensure it gets rebuild properly when needed.
        //self.cached_data.write().unwrap().clear();

        // Preload parent mods of the currently open PackFile.
        self.load_parent_packs(pack_names, game_info, game_path)?;

        // Build the casing-related HashSets.
        //self.parent_cached_packed_files_paths = LazyLoadedData::NotYetLoaded;
        //self.parent_cached_folders_cased = LazyLoadedData::NotYetLoaded;
        //self.parent_cached_folders_caseless = LazyLoadedData::NotYetLoaded;

        // Pre-decode all tables/locs to memory.
        let mut decode_extra_data = DecodeableExtraData::default();
        decode_extra_data.set_schema(Some(&schema));
        let extra_data = Some(decode_extra_data);

        self.parent_files.par_iter_mut().try_for_each(|(_, file)| {
            match file.file_type() {
                FileType::DB |
                FileType::Loc => file.decode(&extra_data, true, false).map(|_| ()),
                _ => Ok(())
            }
        })?;

        self.parent_files.iter()
            .filter(|(_, file)| matches!(file.file_type(), FileType::DB))
            .for_each(|(path, file)| {
                match file.file_type() {
                    FileType::DB => {
                        let path_split = path.split('/').collect::<Vec<_>>();
                        if path_split.len() == 3 {
                            let table_name = path_split[1].replace("_tables", "");
                            match self.parent_tables.get_mut(&table_name) {
                                Some(table_paths) => table_paths.push(path.to_owned()),
                                None => { self.parent_tables.insert(table_name, vec![path.to_owned()]); },
                            }
                        }
                    }
                    FileType::Loc => {
                        self.parent_locs.insert(path.to_owned());
                    }
                    _ => {}
                }
            }
        );

        Ok(())
    }

    /// This function generates the entire dependency cache for the currently selected game.
    pub fn generate_dependencies_cache(&mut self, game_info: &GameInfo, game_path: &Path, asskit_path: &Option<PathBuf>, version: i16) -> Result<Self> {

        let mut cache = Self::default();
        cache.build_date = current_time()?;
        cache.vanilla_files = Pack::read_and_merge_ca_packs(game_info, game_path)?.files().clone();

        // This one can fail, leaving the dependencies with only game data.
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
    fn generate_asskit_only_db_tables(&mut self, raw_db_path: &Path, version: i16) -> Result<()> {
        let files_to_ignore = self.vanilla_tables.keys().map(|x| &**x).collect::<Vec<&str>>();
        let raw_tables = RawTable::read_all(raw_db_path, version, &files_to_ignore)?;
        self.asskit_only_db_tables = raw_tables.par_iter().map(TryFrom::try_from).collect::<Result<Vec<DB>>>()?;

        Ok(())
    }

    /// This function loads a `Dependencies` to memory from a file in the `dependencies/` folder.
    pub fn load(file_path: &Path, schema: &Schema) -> Result<Self> {
        let mut file = BufReader::new(File::open(&file_path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;

        // Never deserialize directly from the file. It's bloody slow!!!
        let mut dependencies: Self = bincode::deserialize(&data)?;

        // Pre-decode all tables/locs to memory.
        let mut decode_extra_data = DecodeableExtraData::default();
        decode_extra_data.set_schema(Some(&schema));
        let extra_data = Some(decode_extra_data);

        dependencies.vanilla_files.par_iter_mut().try_for_each(|(_, file)| {
            match file.file_type() {
                FileType::DB |
                FileType::Loc => file.decode(&extra_data, true, false).map(|_| ()),
                _ => Ok(())
            }
        })?;

        dependencies.vanilla_files.iter()
            .filter(|(_, file)| matches!(file.file_type(), FileType::DB))
            .for_each(|(path, file)| {
                match file.file_type() {
                    FileType::DB => {
                        let path_split = path.split('/').collect::<Vec<_>>();
                        if path_split.len() == 3 {
                            let table_name = path_split[1].replace("_tables", "");
                            match dependencies.vanilla_tables.get_mut(&table_name) {
                                Some(table_paths) => table_paths.push(path.to_owned()),
                                None => { dependencies.vanilla_tables.insert(table_name, vec![path.to_owned()]); },
                            }
                        }
                    }
                    FileType::Loc => {
                        dependencies.vanilla_locs.insert(path.to_owned());
                    }
                    _ => {}
                }
            }
        );

        // Build the casing-related HashSets.
        //dependencies.vanilla_cached_packed_files_paths = LazyLoadedData::NotYetLoaded;
        //dependencies.vanilla_cached_folders_cased = LazyLoadedData::NotYetLoaded;
        //dependencies.vanilla_cached_folders_caseless = LazyLoadedData::NotYetLoaded;

        Ok(dependencies)
    }

    /// This function saves a `Dependencies` from memory to a file in the `dependencies/` folder.
    pub fn save(&mut self, file_path: &Path) -> Result<()> {
        let mut folder_path = file_path.to_owned();
        folder_path.pop();
        DirBuilder::new().recursive(true).create(&folder_path)?;

        // Never serialize directly into the file. It's bloody slow!!!
        let mut file = File::create(&file_path)?;
        let serialized: Vec<u8> = bincode::serialize(&self)?;
        file.write_all(&serialized).map_err(From::from)
    }

    /// This function is used to check if the files RPFM uses to generate the dependencies cache have changed, requiring an update.
    pub fn needs_updating(&self, game_info: &GameInfo, game_path: &Path) -> Result<bool> {
        let ca_paths = game_info.get_all_ca_packfiles_paths(game_path)?;
        let last_date = last_modified_time_from_files(&ca_paths)?;
        Ok(last_date > self.build_date)
    }


    /// This function loads all the parent [Packs](rpfm_lib::files::pack::Pack) provided as `pack_names` as dependencies,
    /// taking care of also loading all dependencies of all of them, if they're not already loaded.
    fn load_parent_packs(&mut self, pack_names: &[String], game_info: &GameInfo, game_path: &Path) -> Result<()> {
        let data_packs_paths = game_info.get_all_ca_packfiles_paths(game_path)?;
        let content_packs_paths = game_info.get_content_packfiles_paths(game_path);
        let mut loaded_packfiles = vec![];

        pack_names.iter().for_each(|pack_name| self.load_parent_pack(&pack_name, &mut loaded_packfiles, &data_packs_paths, &content_packs_paths));

        Ok(())
    }

    /// This function loads a parent [Pack](rpfm_lib::files::pack::Pack) as a dependency, taking care of also loading all dependencies of it, if they're not already loaded.
    fn load_parent_pack(
        &mut self,
        pack_name: &str,
        already_loaded: &mut Vec<String>,
        data_paths: &[PathBuf],
        external_path: &Option<Vec<PathBuf>>,
    ) {
        // Do not process Packs twice.
        if !already_loaded.contains(&pack_name.to_owned()) {

            // First, if the game has an external path (not in /data) for Packs, we load the external Packs.
            if let Some(ref paths) = external_path {
                if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                    if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false) {
                        already_loaded.push(pack_name.to_owned());
                        pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(&pack_name, already_loaded, data_paths, external_path));
                        self.parent_files.extend(pack.files().clone());
                    }
                }
            }

            // Then we load the Packs from /data, so they take priority over the other ones when overwriting.
            if let Some(path) = data_paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false) {
                    already_loaded.push(pack_name.to_owned());
                    pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(&pack_name, already_loaded, data_paths, external_path));
                    self.parent_files.extend(pack.files().clone());
                }
            }
        }
    }

    //-----------------------------------//
    // Getters
    //-----------------------------------//

    /// This function returns a specific file from the cache, if exists.
    pub fn file(&mut self, game_info: &GameInfo, game_path: &Path, file_path: &str, include_vanilla: bool, include_parent: bool) -> Result<Option<RFile>> {
        if self.needs_updating(game_info, game_path)? {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {

            if include_parent {
                if let Some(file) = self.parent_files.get_mut(file_path) {
                    return Ok(Some(file.clone()));
                }
            }

            if include_vanilla {
                if let Some(file) = self.vanilla_files.get_mut(file_path) {
                    return Ok(Some(file.clone()));
                }
            }

            Ok(None)
        }
    }

    /// This function returns the vanilla/parent locs from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    pub fn loc_data(&mut self, game_info: &GameInfo, game_path: &Path, include_vanilla: bool, include_parent: bool) -> Result<Vec<RFile>> {
        if self.needs_updating(game_info, game_path)? {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];

            if include_vanilla {
                let mut vanilla_locs = self.vanilla_locs.iter().collect::<Vec<_>>();
                vanilla_locs.sort();

                for path in &vanilla_locs {
                    if let Some(file) = self.vanilla_files.get_mut(*path) {
                        cache.push(file.clone());
                    }
                }
            }

            if include_parent {
                let mut parent_locs = self.parent_locs.iter().collect::<Vec<_>>();
                parent_locs.sort();

                for path in &parent_locs {
                    if let Some(file) = self.parent_files.get_mut(*path) {
                        cache.push(file.clone());
                    }
                }
            }

            Ok(cache)
        }
    }

    /// This function returns the vanilla/parent db tables from the cache, according to the params you pass it.
    ///
    /// NOTE: table_name is expected to be the table's folder name, without "_tables" at the end.
    ///
    /// It returns them in the order the game will load them.
    pub fn db_data(&mut self, game_info: &GameInfo, game_path: &Path, table_name: &str, include_vanilla: bool, include_parent: bool) -> Result<Vec<RFile>> {
        if self.needs_updating(game_info, game_path)? {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];

            if include_vanilla {
                if let Some(vanilla_tables) = self.vanilla_tables.get(table_name) {
                    let mut vanilla_tables = vanilla_tables.to_vec();
                    vanilla_tables.sort();

                    for path in &vanilla_tables {
                        if let Some(file) = self.vanilla_files.get_mut(&*path) {
                            cache.push(file.clone());
                        }
                    }
                }
            }

            if include_parent {
                if let Some(parent_tables) = self.parent_tables.get(table_name) {
                    let mut parent_tables = parent_tables.to_vec();
                    parent_tables.sort();

                    for path in &parent_tables {
                        if let Some(file) = self.parent_files.get_mut(&*path) {
                            cache.push(file.clone());
                        }
                    }
                }
            }

            Ok(cache)
        }
    }

    /// This function returns the vanilla/parent DB and Loc tables from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    pub fn db_and_loc_data(&mut self, game_info: &GameInfo, game_path: &Path, include_db: bool, include_loc: bool, include_vanilla: bool, include_parent: bool) -> Result<Vec<RFile>> {
        if self.needs_updating(game_info, game_path)? {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {
            let mut cache = vec![];

            if include_vanilla {
                if include_db {
                    let mut vanilla_tables = self.vanilla_tables.values().flatten().collect::<Vec<_>>();
                    vanilla_tables.sort();

                    for path in &vanilla_tables {
                        if let Some(file) = self.vanilla_files.get_mut(*path) {
                            cache.push(file.clone());
                        }
                    }
                }

                if include_loc {
                    let mut vanilla_locs = self.vanilla_locs.iter().collect::<Vec<_>>();
                    vanilla_locs.sort();

                    for path in &vanilla_locs {
                        if let Some(file) = self.vanilla_files.get_mut(*path) {
                            cache.push(file.clone());
                        }
                    }
                }
            }

            if include_parent {
                if include_db {
                    let mut parent_tables = self.parent_tables.values().flatten().collect::<Vec<_>>();
                    parent_tables.sort();

                    for path in &parent_tables {
                        if let Some(file) = self.parent_files.get_mut(*path) {
                            cache.push(file.clone());
                        }
                    }
                }

                if include_loc {
                    let mut parent_locs = self.parent_locs.iter().collect::<Vec<_>>();
                    parent_locs.sort();

                    for path in &parent_locs {
                        if let Some(file) = self.parent_files.get_mut(*path) {
                            cache.push(file.clone());
                        }
                    }
                }
            }

            Ok(cache)
        }
    }

    //-----------------------------------//
    // Utility functions.
    //-----------------------------------//

    /// This function returns if a specific file exists in the dependencies cache.
    pub fn file_exists(&self, game_info: &GameInfo, game_path: &Path, file_path: &str, include_vanilla: bool, include_parent: bool) -> Result<bool> {
        if self.needs_updating(game_info, game_path)? {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate.into());
        } else {

            if include_parent {
                if self.parent_files.get(file_path).is_some() {
                    return Ok(true)
                }
            }

            if include_vanilla {
                if self.vanilla_files.get(file_path).is_some() {
                    return Ok(true);
                }
            }

            Ok(false)
        }
    }

    /// This function checks if the dependencies cache file exists on disk.
    pub fn are_dependencies_generated(file_path: &Path) -> bool {
        file_path.is_file()
    }

    /// This function checks if there is vanilla data loaded in the provided cache.
    pub fn is_vanilla_data_loaded(&self, include_asskit: bool) -> bool {
        if include_asskit {
            !self.vanilla_files.is_empty() && self.is_asskit_data_loaded()
        } else {
            !self.vanilla_files.is_empty()
        }
    }

    /// This function checks if there is assembly kit data loaded in the provided cache.
    pub fn is_asskit_data_loaded(&self) -> bool {
        !self.asskit_only_db_tables.is_empty()
    }


/*
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
    }*/
}
/*
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
*/
