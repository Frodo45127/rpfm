//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains a dependencies system implementation, used to manage dependencies between packs.

use getset::{Getters, MutGetters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::{HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::{JoinHandle, spawn};

use rpfm_lib::error::{Result, RLibError};
use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded};
use rpfm_lib::games::GameInfo;
use rpfm_lib::integrations::assembly_kit::table_data::RawTable;
use rpfm_lib::schema::{Definition, Schema};
use rpfm_lib::utils::{current_time, last_modified_time_from_files, starts_with_case_insensitive};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents a dependencies manager for all dependencies relevant of a Pack.
///
/// As even I am getting a bit confused by how this works (and it has caused a few bugs):
/// - First, these ones are serialized to disk and do not change unless we regenerate the dependencies:
///     - asskit_only_db_tables.
///     - vanilla_files.
///     - vanilla_tables.
///     - vanilla_locs.
/// - Then, we have the ones that gets regenerated on rebuild:
///     - parent_files.
///     - parent_tables.
///     - parent_locs.
///     - local_tables_references.
///
/// - Then, on runtime, we add decoded table's reference data to this one, so we don't need to recalculate it again.
///     - local_tables_references,
#[derive(Default, Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Dependencies {

    /// Date of the generation of this dependencies cache. For checking if it needs an update.
    build_date: u64,

    /// Data to quickly load CA dependencies from disk.
    vanilla_files: HashMap<String, RFile>,

    /// Data to quickly load dependencies from parent mods from disk.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_files: HashMap<String, RFile>,

    /// List of DB tables on the CA files.
    vanilla_tables: HashMap<String, Vec<String>>,

    /// List of DB tables on the parent files.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_tables: HashMap<String, Vec<String>>,

    /// List of Loc tables on the CA files.
    vanilla_locs: HashSet<String>,

    /// List of Loc tables on the parent files.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_locs: HashSet<String>,

    /// Data to quickly check if a path exists in the vanilla files
    vanilla_folders: HashSet<String>,

    /// Data to quickly check if a path exists in the parent mod files
    #[serde(skip_serializing, skip_deserializing)]
    parent_folders: HashSet<String>,

    /// Cached data for local tables.
    ///
    /// This is for runtime caching, and it must not be serialized to disk.
    #[serde(skip_serializing, skip_deserializing)]
    local_tables_references: HashMap<String, HashMap<i32, TableReferences>>,

    /// DB Files only available on the assembly kit. Usable only for references. Do not use them as the base for new tables.
    asskit_only_db_tables: HashMap<String, DB>,
}

/// This holds the reference data for a table's column.
#[derive(Eq, PartialEq, Clone, Default, Debug, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TableReferences {

    /// Name of the column these references are for. Only for debugging, do not rely on it for anything.
    field_name: String,

    /// If the table is only present in the Ak. Useful to identify unused tables on diagnostics checks.
    referenced_table_is_ak_only: bool,

    /// If the referenced column has been moved into a loc file while exporting it from Dave.
    referenced_column_is_localised: bool,

    /// The data itself, as in "key, lookup" format.
    data: HashMap<String, String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Dependencies {

    //-----------------------------------//
    // Generation and disk IO
    //-----------------------------------//

    /// This function takes care of rebuilding the whole dependencies cache to be used with a new Pack.
    ///
    /// If a file path is passed, the dependencies cache at that path will be used, replacing the currently loaded dependencies cache.
    /// If a schema is not passed, no tables/locs will be pre-decoded. Make sure to decode them later with [decode_tables].
    pub fn rebuild(&mut self, schema: &Option<Schema>, parent_pack_names: &[String], file_path: Option<&Path>, game_info: &GameInfo, game_path: &Path) -> Result<()> {

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
        self.local_tables_references.clear();

        // Preload parent mods of the currently loaded Pack.
        self.load_parent_packs(parent_pack_names, game_info, game_path)?;
        self.parent_files.par_iter_mut().map(|(_, file)| file.guess_file_type()).collect::<Result<()>>()?;

        // Then build the table/loc lists, for easy access.
        self.parent_files.iter()
            .filter(|(_, file)| matches!(file.file_type(), FileType::DB))
            .for_each(|(path, file)| {
                match file.file_type() {
                    FileType::DB => {
                        if let Some(table_name) = file.db_table_name_from_path() {
                            match self.parent_tables.get_mut(table_name) {
                                Some(table_paths) => table_paths.push(path.to_owned()),
                                None => { self.parent_tables.insert(table_name.to_owned(), vec![path.to_owned()]); },
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

        // Build the folder list.
        self.parent_folders = self.parent_files.par_iter().filter_map(|(path, _)| {
            let file_path_split = path.split('/').collect::<Vec<&str>>();
            let folder_path_len = file_path_split.len() - 1;
            if folder_path_len == 0 {
                None
            } else {

                let mut paths = Vec::with_capacity(folder_path_len);

                for (index, folder) in file_path_split.iter().enumerate() {
                    if index < path.len() - 1 && !folder.is_empty() {
                        paths.push(file_path_split[0..=index].join("/"))
                    }
                }

                Some(paths)
            }
        }).flatten().collect::<HashSet<String>>();

        // Only decode the tables if we passed a schema. If not, it's responsability of the user to decode them later.
        if let Some(schema) = schema {
            let mut decode_extra_data = DecodeableExtraData::default();
            decode_extra_data.set_schema(Some(schema));
            let extra_data = Some(decode_extra_data);

            let mut files = self.parent_locs.iter().chain(self.parent_tables.values().flatten()).filter_map(|path| {
                self.parent_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.parent_files.par_extend(files);
        }

        Ok(())
    }

    /// This function generates the dependencies cache for the game provided and returns it.
    pub fn generate_dependencies_cache(game_info: &GameInfo, game_path: &Path, asskit_path: &Option<PathBuf>) -> Result<Self> {
        let mut cache = Self::default();
        cache.build_date = current_time()?;
        cache.vanilla_files = Pack::read_and_merge_ca_packs(game_info, game_path)?.files().clone();

        let cacheable = cache.vanilla_files.par_iter_mut()
            .filter_map(|(_, file)| {
                let _ = file.guess_file_type();

                match file.file_type() {
                    FileType::DB |
                    FileType::Loc => Some(file),
                    _ => None,
                }
            })
            .collect::<Vec<&mut RFile>>();

        cacheable.iter()
            .for_each(|file| {
                match file.file_type() {
                    FileType::DB => {
                        if let Some(table_name) = file.db_table_name_from_path() {
                            match cache.vanilla_tables.get_mut(table_name) {
                                Some(table_paths) => table_paths.push(file.path_in_container_raw().to_owned()),
                                None => { cache.vanilla_tables.insert(table_name.to_owned(), vec![file.path_in_container_raw().to_owned()]); },
                            }
                        }
                    }
                    FileType::Loc => {
                        cache.vanilla_locs.insert(file.path_in_container_raw().to_owned());
                    }
                    _ => {}
                }
            }
        );

        cache.vanilla_folders = cache.vanilla_files.par_iter().filter_map(|(path, _)| {
            let file_path_split = path.split('/').collect::<Vec<&str>>();
            let folder_path_len = file_path_split.len() - 1;
            if folder_path_len == 0 {
                None
            } else {

                let mut paths = Vec::with_capacity(folder_path_len);

                for (index, folder) in file_path_split.iter().enumerate() {
                    if index < path.len() - 1 && !folder.is_empty() {
                        paths.push(file_path_split[0..=index].join("/"))
                    }
                }

                Some(paths)
            }
        }).flatten().collect::<HashSet<String>>();

        // This one can fail, leaving the dependencies with only game data.
        if let Some(path) = asskit_path {
            let _ = cache.generate_asskit_only_db_tables(path, game_info.raw_db_version());
        }

        Ok(cache)
    }

    /// This function generates a "fake" table list with tables only present in the Assembly Kit.
    ///
    /// This works by processing all the tables from the game's raw table folder and turning them into fake decoded tables,
    /// with version -1. That will allow us to use them for dependency checking and for populating combos.
    ///
    /// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK2 file.
    fn generate_asskit_only_db_tables(&mut self, raw_db_path: &Path, version: i16) -> Result<()> {
        let files_to_ignore = self.vanilla_tables.keys().map(|table_name| &table_name[..table_name.len() - 7]).collect::<Vec<_>>();
        let raw_tables = RawTable::read_all(raw_db_path, version, &files_to_ignore)?;
        let asskit_only_db_tables = raw_tables.par_iter().map(TryFrom::try_from).collect::<Result<Vec<DB>>>()?;
        self.asskit_only_db_tables = asskit_only_db_tables.par_iter().map(|table| (table.table_name().to_owned(), table.clone())).collect::<HashMap<String, DB>>();

        Ok(())
    }

    /// This function builds the local db references data for the tables you pass to it from the Pack provided.
    ///
    /// Table names must be provided as full names (with *_tables* at the end).
    ///
    /// NOTE: This function, like many others, assumes the tables are already decoded in the Pack. If they're not, they'll be ignored.
    pub fn generate_local_db_references(&mut self, pack: &Pack, table_names: &[String]) {

        let local_tables_references = pack.files_by_type(&[FileType::DB]).par_iter().filter_map(|file| {
            if let Ok(RFileDecoded::DB(db)) = file.decoded() {

                // Only generate references for the tables you pass it, or for all if we pass the list of tables empty.
                if table_names.is_empty() || table_names.iter().any(|x| x == db.table_name()) {
                     Some((db.table_name().to_owned(), self.generate_references(db.definition())))
                } else { None }
            } else { None }
        }).collect::<HashMap<_, _>>();

        self.local_tables_references.extend(local_tables_references);
    }

    /// This function builds the local db references data for the table with the definition you pass to and stores it in the cache.
    pub fn generate_local_definition_references(&mut self, table_name: &str, definition: &Definition) {
        self.local_tables_references.insert(table_name.to_owned(), self.generate_references(definition));
    }

    /// This function builds the local db references data for the table with the definition you pass to, and returns it.
    pub fn generate_references(&self, definition: &Definition) -> HashMap<i32, TableReferences> {
        definition.fields_processed().into_iter().enumerate().filter_map(|(column, field)| {
            if let Some((ref ref_table, ref ref_column)) = field.is_reference() {
                if !ref_table.is_empty() && !ref_column.is_empty() {
                    let ref_table = format!("{}_tables", ref_table);

                    // Get his lookup data if it has it.
                    let lookup_data = if let Some(ref data) = field.lookup() { data.to_vec() } else { Vec::with_capacity(0) };
                    let mut references = TableReferences::default();
                    *references.field_name_mut() = field.name().to_owned();

                    let fake_found = self.db_reference_data_from_asskit_tables(&mut references, (&ref_table, ref_column, &lookup_data));
                    let real_found = self.db_reference_data_from_from_vanilla_and_modded_tables(&mut references, (&ref_table, ref_column, &lookup_data));

                    if fake_found && real_found.is_none() {
                        references.referenced_table_is_ak_only = true;
                    }

                    if let Some(ref_definition) = real_found {
                        if ref_definition.localised_fields().iter().any(|x| x.name() == ref_column) {
                            references.referenced_column_is_localised = true;
                        }
                    }

                    Some((column as i32, references))
                } else { None }
            } else { None }
        }).collect::<HashMap<_, _>>()
    }

    /// This function tries to load dependencies from the path provided.
    pub fn load(file_path: &Path, schema: &Option<Schema>) -> Result<Self> {

        // Optimization: Instead of a big file, we split the dependencies in 3 files. Why?
        // Because bincode is not multithreaded and, while reading 3 medium files is slower than a big one,
        // deserializing 3 medium files in 3 separate threads is way faster than 1 big file in 1 thread.
        let mut file_path_1 = file_path.to_path_buf();
        let handle_1: JoinHandle<Result<(u64, Vec<RFile>)>> = spawn(move || {
            file_path_1.set_extension("pak1");
            let mut file = BufReader::new(File::open(&file_path_1)?);
            let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
            file.read_to_end(&mut data)?;

            // Never deserialize directly from the file. It's bloody slow!!!
            bincode::deserialize(&data).map_err(From::from)
        });

        let mut file_path_2 = file_path.to_path_buf();
        let handle_2: JoinHandle<Result<Vec<RFile>>> = spawn(move || {
            file_path_2.set_extension("pak2");
            let mut file = BufReader::new(File::open(&file_path_2)?);
            let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
            file.read_to_end(&mut data)?;

            // Never deserialize directly from the file. It's bloody slow!!!
            bincode::deserialize(&data).map_err(From::from)
        });

        let mut file_path_3 = file_path.to_path_buf();
        let handle_3: JoinHandle<Result<(HashMap<String, Vec<String>>, HashSet<String>, HashSet<String>, HashMap<String, DB>)>> = spawn(move || {
            file_path_3.set_extension("pak3");
            let mut file = BufReader::new(File::open(&file_path_3)?);
            let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
            file.read_to_end(&mut data)?;

            // Never deserialize directly from the file. It's bloody slow!!!
            bincode::deserialize(&data).map_err(From::from)
        });

        // Get the thread's data in reverse, as 1 and 2 are actually the slower to process.
        let mut dependencies = Self::default();
        let data_3 = handle_3.join().unwrap()?;
        let data_2 = handle_2.join().unwrap()?;
        let data_1 = handle_1.join().unwrap()?;

        // The vanilla file list is stored in a Vec format instead of a hashmap, because a vec can be splited,
        // and that list is more than 100mb long in some games. Here we turn it back to HashMap and merge it.
        let mut vanilla_files: HashMap<_,_> = data_1.1.into_par_iter().map(|file| (file.path_in_container_raw().to_owned(), file)).collect();
        vanilla_files.par_extend(data_2.into_par_iter().map(|file| (file.path_in_container_raw().to_owned(), file)));

        dependencies.build_date = data_1.0;
        dependencies.vanilla_files = vanilla_files;
        dependencies.vanilla_tables = data_3.0;
        dependencies.vanilla_locs = data_3.1;
        dependencies.vanilla_folders = data_3.2;
        dependencies.asskit_only_db_tables = data_3.3;

        // Only decode the tables if we passed a schema. If not, it's responsability of the user to decode them later.
        if let Some(schema) = schema {
            let mut decode_extra_data = DecodeableExtraData::default();
            decode_extra_data.set_schema(Some(schema));
            let extra_data = Some(decode_extra_data);

            let mut files = dependencies.vanilla_locs.iter().chain(dependencies.vanilla_tables.values().flatten()).filter_map(|path| {
                dependencies.vanilla_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            dependencies.vanilla_files.par_extend(files);
        }

        Ok(dependencies)
    }

    /// This function saves a dependencies cache to the provided path.
    pub fn save(&mut self, file_path: &Path) -> Result<()> {
        let mut folder_path = file_path.to_owned();
        folder_path.pop();
        DirBuilder::new().recursive(true).create(&folder_path)?;

        let mut file_path_1 = file_path.to_path_buf();
        let mut file_path_2 = file_path.to_path_buf();
        let mut file_path_3 = file_path.to_path_buf();

        file_path_1.set_extension("pak1");
        file_path_2.set_extension("pak2");
        file_path_3.set_extension("pak3");

        let mut file_1 = File::create(&file_path_1)?;
        let mut file_2 = File::create(&file_path_2)?;
        let mut file_3 = File::create(&file_path_3)?;

        // Split the vanilla file's list in half and turn it into a vec, so it's faster when loading.
        // NOTE: While the HashMap -> Vec conversion only keeping values thing is slower to read
        // than serializing/loading the keys directly, it saves about 40mb of data on disk.
        let mut vanilla_files_1 = self.vanilla_files.par_iter().map(|(_, b)| b.clone()).collect::<Vec<RFile>>();
        let vanilla_files_2 = vanilla_files_1.split_off(self.vanilla_files.len() / 2);

        // Never serialize directly into the file. It's bloody slow!!!
        let serialized_1: Vec<u8> = bincode::serialize(&(&self.build_date, &vanilla_files_1))?;
        let serialized_2: Vec<u8> = bincode::serialize(&vanilla_files_2)?;
        let serialized_3: Vec<u8> = bincode::serialize(&(&self.vanilla_tables, &self.vanilla_locs, &self.vanilla_folders, &self.asskit_only_db_tables))?;

        file_1.write_all(&serialized_1).map_err(RLibError::from)?;
        file_2.write_all(&serialized_2).map_err(RLibError::from)?;
        file_3.write_all(&serialized_3).map_err(From::from)
    }

    /// This function is used to check if the game files used to generate the dependencies cache have changed, requiring an update.
    pub fn needs_updating(&self, game_info: &GameInfo, game_path: &Path) -> Result<bool> {
        let ca_paths = game_info.ca_packs_paths(game_path)?;
        let last_date = last_modified_time_from_files(&ca_paths)?;
        Ok(last_date > self.build_date)
    }


    /// This function loads all the parent [Packs](rpfm_lib::files::pack::Pack) provided as `parent_pack_names` as dependencies,
    /// taking care of also loading all dependencies of all of them, if they're not already loaded.
    fn load_parent_packs(&mut self, parent_pack_names: &[String], game_info: &GameInfo, game_path: &Path) -> Result<()> {
        let data_packs_paths = game_info.ca_packs_paths(game_path)?;
        let content_packs_paths = game_info.content_packs_paths(game_path);
        let mut loaded_packfiles = vec![];

        parent_pack_names.iter().for_each(|pack_name| self.load_parent_pack(pack_name, &mut loaded_packfiles, &data_packs_paths, &content_packs_paths));

        Ok(())
    }

    /// This function loads a parent [Pack](rpfm_lib::files::pack::Pack) as a dependency,
    /// taking care of also loading all dependencies of it, if they're not already loaded.
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
                        pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(pack_name, already_loaded, data_paths, external_path));
                        self.parent_files.extend(pack.files().clone());
                    }
                }
            }

            // Then we load the Packs from /data, so they take priority over the other ones when overwriting.
            if let Some(path) = data_paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false) {
                    already_loaded.push(pack_name.to_owned());
                    pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(pack_name, already_loaded, data_paths, external_path));
                    self.parent_files.extend(pack.files().clone());
                }
            }
        }
    }

    /// Function to force-decode all tables/locs in the dependencies.
    ///
    /// Many operations require them to be decoded, so if you did not decoded them on load, make sure to call this to decode them after load.
    pub fn decode_tables(&mut self, schema: &Option<Schema>) {
        if let Some(schema) = schema {

            let mut decode_extra_data = DecodeableExtraData::default();
            decode_extra_data.set_schema(Some(schema));
            let extra_data = Some(decode_extra_data);

            // Vanilla files.
            let mut files = self.vanilla_locs.iter().chain(self.vanilla_tables.values().flatten()).filter_map(|path| {
                self.vanilla_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.vanilla_files.par_extend(files);

            // Parent files.
            let mut files = self.parent_locs.iter().chain(self.parent_tables.values().flatten()).filter_map(|path| {
                self.parent_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.parent_files.par_extend(files);
        }
    }

    //-----------------------------------//
    // Getters
    //-----------------------------------//

    /// This function returns a reference to a specific file from the cache, if exists.
    pub fn file(&self, file_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> Result<&RFile> {
        if include_parent {
            if case_insensitive {
                if let Some(file) = self.parent_files.par_iter().find_map_first(|(path, file)| if caseless::canonical_caseless_match_str(path, file_path) { Some(file) } else { None }) {
                    return Ok(file);
                }
            } else if let Some(file) = self.parent_files.get(file_path) {
                return Ok(file);
            }
        }

        if include_vanilla {
            if case_insensitive {
                if let Some(file) = self.vanilla_files.par_iter().find_map_first(|(path, file)| if caseless::canonical_caseless_match_str(path, file_path) { Some(file) } else { None }) {
                    return Ok(file);
                }
            } else if let Some(file) = self.vanilla_files.get(file_path) {
                return Ok(file);
            }
        }

        Err(RLibError::DependenciesCacheFileNotFound(file_path.to_owned()))
    }

    /// This function returns a mutable reference to a specific file from the cache, if exists.
    pub fn file_mut(&mut self, file_path: &str, include_vanilla: bool, include_parent: bool) -> Result<&mut RFile> {
        if include_parent {
            if let Some(file) = self.parent_files.get_mut(file_path) {
                return Ok(file);
            }
        }

        if include_vanilla {
            if let Some(file) = self.vanilla_files.get_mut(file_path) {
                return Ok(file);
            }
        }

        Err(RLibError::DependenciesCacheFileNotFound(file_path.to_owned()))
    }

    /// This function returns a reference to all files corresponding to the provided paths.
    pub fn files_by_path(&self, file_paths: &[ContainerPath], include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> HashMap<String, &RFile> {
        file_paths.into_par_iter().flat_map(|file_path| {
            let mut files = HashMap::new();
            match file_path {
                ContainerPath::Folder(folder_path) => {

                    if include_vanilla {
                        if folder_path.is_empty() {
                            files.extend(self.vanilla_files.par_iter()
                                .map(|(path, file)| (path.to_owned(), file))
                                .collect::<HashMap<_,_>>());
                        } else {
                            files.extend(self.vanilla_files.par_iter()
                                .filter(|(path, _)| {
                                    if case_insensitive {
                                        starts_with_case_insensitive(path, folder_path)
                                    } else {
                                        path.starts_with(folder_path)
                                    }
                                })
                                .map(|(path, file)| (path.to_owned(), file))
                                .collect::<HashMap<_,_>>());
                        }
                    }

                    if include_parent {
                        files = self.parent_files.par_iter()
                            .filter(|(path, _)| {
                                if case_insensitive {
                                    starts_with_case_insensitive(path, folder_path)
                                } else {
                                    path.starts_with(folder_path)
                                }
                            })
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect();
                    }
                }
                ContainerPath::File(file_path) => {
                    if let Ok(file) = self.file(file_path, include_vanilla, include_parent, case_insensitive) {
                        files.insert(file_path.to_string(), file);
                    }
                }
            }
            files
        }).collect()
    }

    /// This function returns a reference to all files of the specified FileTypes from the cache, if any, along with their path.
    pub fn files_by_types(&self, file_types: &[FileType], include_vanilla: bool, include_parent: bool) -> HashMap<String, &RFile> {
        let mut files = HashMap::new();

        // Vanilla first, so if parent files are found, they overwrite vanilla files.
        if include_vanilla {
            files.extend(self.vanilla_files.par_iter()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        if include_parent {
            files = self.parent_files.par_iter()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect();
        }

        files
    }

    /// This function returns a mutable reference to all files of the specified FileTypes from the cache, if any, along with their path.
    pub fn files_by_types_mut(&mut self, file_types: &[FileType], include_vanilla: bool, include_parent: bool) -> HashMap<String, &mut RFile> {
        let mut files = HashMap::new();

        // Vanilla first, so if parent files are found, they overwrite vanilla files.
        if include_vanilla {
            files.extend(self.vanilla_files.par_iter_mut()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        if include_parent {
            files = self.parent_files.par_iter_mut()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect();
        }

        files
    }

    /// This function returns the vanilla/parent locs from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    pub fn loc_data(&self, include_vanilla: bool, include_parent: bool) -> Result<Vec<&RFile>> {
        let mut cache = vec![];

        if include_vanilla {
            let mut vanilla_locs = self.vanilla_locs.iter().collect::<Vec<_>>();
            vanilla_locs.sort();

            for path in &vanilla_locs {
                if let Some(file) = self.vanilla_files.get(*path) {
                    cache.push(file);
                }
            }
        }

        if include_parent {
            let mut parent_locs = self.parent_locs.iter().collect::<Vec<_>>();
            parent_locs.sort();

            for path in &parent_locs {
                if let Some(file) = self.parent_files.get(*path) {
                    cache.push(file);
                }
            }
        }

        Ok(cache)
    }

    /// This function returns the vanilla/parent db tables from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    ///
    /// NOTE: table_name is expected to be the table's folder name, with "_tables" at the end.
    pub fn db_data(&self, table_name: &str, include_vanilla: bool, include_parent: bool) -> Result<Vec<&RFile>> {
        let mut cache = vec![];

        if include_vanilla {
            if let Some(vanilla_tables) = self.vanilla_tables.get(table_name) {
                let mut vanilla_tables = vanilla_tables.to_vec();
                vanilla_tables.sort();

                for path in &vanilla_tables {
                    if let Some(file) = self.vanilla_files.get(path) {
                        cache.push(file);
                    }
                }
            }
        }

        if include_parent {
            if let Some(parent_tables) = self.parent_tables.get(table_name) {
                let mut parent_tables = parent_tables.to_vec();
                parent_tables.sort();

                for path in &parent_tables {
                    if let Some(file) = self.parent_files.get(path) {
                        cache.push(file);
                    }
                }
            }
        }

        Ok(cache)
    }

    /// This function returns the vanilla/parent DB and Loc tables from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    pub fn db_and_loc_data(&self, include_db: bool, include_loc: bool, include_vanilla: bool, include_parent: bool) -> Result<Vec<&RFile>> {
        let mut cache = vec![];

        if include_vanilla {
            if include_db {
                let mut vanilla_tables = self.vanilla_tables.values().flatten().collect::<Vec<_>>();
                vanilla_tables.sort();

                for path in &vanilla_tables {
                    if let Some(file) = self.vanilla_files.get(*path) {
                        cache.push(file);
                    }
                }
            }

            if include_loc {
                let mut vanilla_locs = self.vanilla_locs.iter().collect::<Vec<_>>();
                vanilla_locs.sort();

                for path in &vanilla_locs {
                    if let Some(file) = self.vanilla_files.get(*path) {
                        cache.push(file);
                    }
                }
            }
        }

        if include_parent {
            if include_db {
                let mut parent_tables = self.parent_tables.values().flatten().collect::<Vec<_>>();
                parent_tables.sort();

                for path in &parent_tables {
                    if let Some(file) = self.parent_files.get(*path) {
                        cache.push(file);
                    }
                }
            }

            if include_loc {
                let mut parent_locs = self.parent_locs.iter().collect::<Vec<_>>();
                parent_locs.sort();

                for path in &parent_locs {
                    if let Some(file) = self.parent_files.get(*path) {
                        cache.push(file);
                    }
                }
            }
        }

        Ok(cache)
    }

    //-----------------------------------//
    // Advanced Getters.
    //-----------------------------------//

    /// This function returns the reference/lookup data of all relevant columns of a DB Table.
    ///
    /// NOTE: This assumes you've populated the runtime references before this. If not, it'll fail.
    pub fn db_reference_data(&self, pack: &Pack, table_name: &str, definition: &Definition) -> HashMap<i32, TableReferences> {

        // First check if the data is already cached, to speed up things.
        let mut vanilla_references = match self.local_tables_references.get(table_name) {
            Some(cached_data) => cached_data.clone(),
            None => panic!("To be fixed: If you see this, you forgot to call generate_local_db_references before this."),
        };

        let local_references = definition.fields_processed().into_par_iter().enumerate().filter_map(|(column, field)| {
            if let Some((ref ref_table, ref ref_column)) = field.is_reference() {
                if !ref_table.is_empty() && !ref_column.is_empty() {

                    // Get his lookup data if it has it.
                    let lookup_data = if let Some(ref data) = field.lookup() { data.to_vec() } else { Vec::with_capacity(0) };
                    let mut references = TableReferences::default();
                    *references.field_name_mut() = field.name().to_owned();

                    let _local_found = Self::db_reference_data_from_local_pack(&mut references, (ref_table, ref_column, &lookup_data), pack);

                    Some((column as i32, references))
                } else { None }
            } else { None }
        }).collect::<HashMap<_, _>>();

        vanilla_references.par_iter_mut().for_each(|(key, value)|
            if let Some(local_value) = local_references.get(key) {
                value.data.extend(local_value.data.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        );

        vanilla_references
    }

    /// This function returns the reference/lookup data of all relevant columns of a DB Table from the vanilla/parent data.
    ///
    /// If reference data was found, the most recent definition of said data is returned.
    fn db_reference_data_from_from_vanilla_and_modded_tables(&self, references: &mut TableReferences, reference_info: (&str, &str, &[String])) -> Option<Definition> {
        let mut data_found: Option<Definition> = None;
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        if let Ok(files) = self.db_data(ref_table, true, true) {
            files.iter().for_each(|file| {
                if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                    let fields_processed = db.definition().fields_processed();
                    let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);
                    let ref_lookup_columns_index = ref_lookup_columns.iter().map(|column| fields_processed.iter().position(|x| x.name() == column)).collect::<Vec<_>>();

                    if let Ok(data) = db.data(&None) {

                        for row in &*data {
                            let mut reference_data = String::new();
                            let mut lookup_data = vec![];

                            // First, we get the reference data.
                            if let Some(index) = ref_column_index {
                                reference_data = row[index].data_to_string().to_string();
                            }

                            // Then, we get the lookup data.
                            for column in ref_lookup_columns_index.iter().flatten() {
                                lookup_data.push(row[*column].data_to_string());
                            }

                            references.data.insert(reference_data, lookup_data.join(" "));
                        }

                        // Once done with the table, check if we should return its definition.
                        match data_found {
                            Some(ref definition) => {
                                if db.definition().version() > definition.version() {
                                    data_found = Some(db.definition().clone());
                                }
                            }

                            None => data_found = Some(db.definition().clone()),
                        }
                    }
                }
            });
        }
        data_found
    }

    /// This function returns the reference/lookup data of all relevant columns of a DB Table from the assembly kit data.
    ///
    /// It returns true if data is found, otherwise it returns false.
    fn db_reference_data_from_asskit_tables(&self, references: &mut TableReferences, reference_info: (&str, &str, &[String])) -> bool {
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        match self.asskit_only_db_tables.get(ref_table) {
            Some(table) => {
                if let Ok(data) = table.data(&None) {
                    let fields_processed = table.definition().fields_processed();
                    let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);
                    let ref_lookup_columns_index = ref_lookup_columns.iter().map(|column| fields_processed.iter().position(|x| x.name() == column)).collect::<Vec<_>>();

                    for row in &*data {
                        let mut reference_data = String::new();
                        let mut lookup_data = vec![];

                        // First, we get the reference data.
                        if let Some(index) = ref_column_index {
                            reference_data = row[index].data_to_string().to_string();
                        }

                        // Then, we get the lookup data.
                        for column in ref_lookup_columns_index.iter().flatten() {
                            lookup_data.push(row[*column].data_to_string());
                        }

                        references.data.insert(reference_data, lookup_data.join(" "));
                    }
                }
                true
            },
            None => false,
        }
    }

    /// This function returns the reference/lookup data of all relevant columns of a DB Table from the provided Pack.
    fn db_reference_data_from_local_pack(references: &mut TableReferences, reference_info: (&str, &str, &[String]), pack: &Pack) -> bool {

        let mut data_found = false;
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        pack.files_by_path(&ContainerPath::Folder(format!("db/{}_tables", ref_table)), false).iter()
            .for_each(|file| {
            if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                let fields_processed = db.definition().fields_processed();
                let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);
                let ref_lookup_columns_index = ref_lookup_columns.iter().map(|column| fields_processed.iter().position(|x| x.name() == column)).collect::<Vec<_>>();

                if let Ok(data) = db.data(&None) {

                    for row in &*data {
                        let mut reference_data = String::new();
                        let mut lookup_data = vec![];

                        // First, we get the reference data.
                        if let Some(index) = ref_column_index {
                            reference_data = row[index].data_to_string().to_string();
                        }

                        // Then, we get the lookup data.
                        for column in ref_lookup_columns_index.iter().flatten() {
                            lookup_data.push(row[*column].data_to_string());
                        }

                        references.data.insert(reference_data, lookup_data.join(" "));

                    }

                    if !&data.is_empty() && !data_found {
                        data_found = true;
                    }
                }
            }
        });
        data_found
    }

    /// This function returns the table/column/key from the provided loc key.
    ///
    /// We return the table without "_tables". Keep that in mind if you use this.
    pub fn loc_key_source(&self, key: &str) -> Option<(String, String, String)> {
        let key_split = key.split('_').collect::<Vec<_>>();
        let mut table_name = String::new();

        // We don't know how much of the string the key the table is, so we try adding parts until we find a table that matches.
        for (index, value) in key_split.iter().enumerate() {
            table_name.push_str(value);

            let full_table_name = format!("{}_tables", table_name);

            if let Ok(rfiles) = self.db_data(&full_table_name, true, false) {
                for rfile in rfiles {
                    if let Ok(RFileDecoded::DB(table)) = rfile.decoded() {
                        let definition = table.definition();
                        let localised_fields = definition.localised_fields();
                        if !localised_fields.is_empty() && key_split.len() > index + 2 {
                            let mut field = String::new();
                            let fields_processed = definition.fields_processed();

                            // Loop to get the column.
                            for (second_index, value) in key_split[index + 1..].iter().enumerate() {
                                field.push_str(value);
                                if localised_fields.iter().any(|x| x.name() == field) {

                                    // If we reached this, the rest is the value.
                                    let key_field = &key_split[index + second_index + 2..].join("_");
                                    if let Some(field) = fields_processed.iter().find(|x| (x.name() == "key" || x.name() == "id") && x.is_key()) {
                                        return Some((table_name, field.name().to_string(), key_field.to_owned()));
                                    }
                                }
                                field.push('_');
                            }
                        }
                    }
                }
            }

            // Add an underscore before adding the next part of the table name in the next loop.
            table_name.push('_');
        }

        None
    }

    //-----------------------------------//
    // Utility functions.
    //-----------------------------------//

    /// This function returns if a specific file exists in the dependencies cache.
    pub fn file_exists(&self, file_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> bool {
        if include_parent {
            if case_insensitive {
                if self.parent_files.par_iter().any(|(path, _)| caseless::canonical_caseless_match_str(path, file_path)) {
                    return true
                }
            } else if self.parent_files.get(file_path).is_some() {
                return true
            }
        }

        if include_vanilla {
            if case_insensitive {
                if self.vanilla_files.par_iter().any(|(path, _)| caseless::canonical_caseless_match_str(path, file_path)) {
                    return true
                }
            } else if self.vanilla_files.get(file_path).is_some() {
                return true
            }
        }

        false
    }

    /// This function returns if a specific folder exists in the dependencies cache.
    pub fn folder_exists(&self, folder_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> bool {
        if include_parent {
            if case_insensitive {
                if self.parent_folders.par_iter().any(|path| caseless::canonical_caseless_match_str(path, folder_path)) {
                    return true
                }
            } else if self.parent_folders.get(folder_path).is_some() {
                return true
            }
        }

        if include_vanilla {
            if case_insensitive {
                if self.vanilla_folders.par_iter().any(|path| caseless::canonical_caseless_match_str(path, folder_path)) {
                    return true
                }
            } else if self.vanilla_folders.get(folder_path).is_some() {
                return true
            }
        }

        false
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

    /// This function is used to check if a table is outdated or not.
    pub fn is_db_outdated(&self, rfile: &RFileDecoded) -> bool {
        if let RFileDecoded::DB(data) = rfile {
            let dep_db_undecoded = if let Ok(undecoded) = self.db_data(data.table_name(), true, false) { undecoded } else { return false };
            let dep_db_decoded = dep_db_undecoded.iter().filter_map(|x| if let Ok(RFileDecoded::DB(decoded)) = x.decoded() { Some(decoded) } else { None }).collect::<Vec<_>>();

            if let Some(vanilla_db) = dep_db_decoded.iter().max_by(|x, y| x.definition().version().cmp(y.definition().version())) {
                if vanilla_db.definition().version() > data.definition().version() {
                    return true;
                }
            }
        }

        false
    }

    /// This function is used to get the version of a table in the game files, if said table is in the game files.
    pub fn db_version(&self, table_name: &str) -> Option<i32> {
        let tables = self.vanilla_tables.get(table_name)?;
        for table_path in tables {

            let table = self.vanilla_files.get(table_path)?;
            if let RFileDecoded::DB(table) = table.decoded().ok()? {
                return Some(*table.definition().version());
            }
        }

        None
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the vanilla files.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_db(&mut self, rfile: &mut RFileDecoded) -> Result<(i32, i32)> {
        match rfile {
            RFileDecoded::DB(data) => {
                let dep_db_undecoded = self.db_data(data.table_name(), true, false)?;
                let dep_db_decoded = dep_db_undecoded.iter().filter_map(|x| if let Ok(RFileDecoded::DB(decoded)) = x.decoded() { Some(decoded) } else { None }).collect::<Vec<_>>();

                if let Some(vanilla_db) = dep_db_decoded.iter().max_by(|x, y| x.definition().version().cmp(y.definition().version())) {

                    let definition_new = vanilla_db.definition();
                    let definition_old = data.definition().clone();
                    if definition_old != *definition_new {
                        data.set_definition(definition_new);
                        Ok((*definition_old.version(), *definition_new.version()))
                    }
                    else {
                        Err(RLibError::NoDefinitionUpdateAvailable)
                    }
                }
                else { Err(RLibError::NoTableInGameFilesToCompare) }
            }
            _ => Err(RLibError::DecodingDBNotADBTable),
        }
    }
}

