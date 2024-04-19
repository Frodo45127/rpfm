//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains a dependencies system implementation, used to manage dependencies between packs.

use getset::{Getters, MutGetters};
use itertools::{Either, Itertools};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::{JoinHandle, spawn};

use rpfm_lib::error::{Result, RLibError};
use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded};
use rpfm_lib::games::GameInfo;
use rpfm_lib::integrations::{assembly_kit::table_data::RawTable, log::info};
use rpfm_lib::schema::{Definition, Field, FieldType, Schema};
use rpfm_lib::utils::{current_time, files_from_subdir, last_modified_time_from_files, starts_with_case_insensitive};

use crate::VERSION;

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
///     - vanilla_loose_files.
///     - vanilla_loose_tables.
///     - vanilla_loose_locs.
///     - vanilla_loose_folders.
///     - vanilla_loose_paths.
///     - parent_files.
///     - parent_tables.
///     - parent_locs.
///     - parent_folders.
///     - parent_paths.
///     - local_tables_references.
///
/// - Then, on runtime, we add decoded table's reference data to this one, so we don't need to recalculate it again.
///     - local_tables_references,
#[derive(Default, Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Dependencies {

    /// Date of the generation of this dependencies cache. For checking if it needs an update.
    build_date: u64,

    /// Version of the program used to generate the dependencies, so they're properly invalidated on update.
    version: String,

    /// Data to quickly load loose files as part of the dependencies.
    ///
    /// Not serialized, regenerated on rebuild because these can frequently change.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_loose_files: HashMap<String, RFile>,

    /// Data to quickly load CA dependencies from disk.
    vanilla_files: HashMap<String, RFile>,

    /// Data to quickly load dependencies from parent mods from disk.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_files: HashMap<String, RFile>,

    /// List of DB tables on the CA loose files. Not really used, but just in case.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_loose_tables: HashMap<String, Vec<String>>,

    /// List of DB tables on the CA files.
    vanilla_tables: HashMap<String, Vec<String>>,

    /// List of DB tables on the parent files.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_tables: HashMap<String, Vec<String>>,

    /// List of Loc tables on the CA loose files. Not really used, but just in case.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_loose_locs: HashSet<String>,

    /// List of Loc tables on the CA files.
    vanilla_locs: HashSet<String>,

    /// List of Loc tables on the parent files.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_locs: HashSet<String>,

    /// Data to quickly check if a path exists in the vanilla loose files.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_loose_folders: HashSet<String>,

    /// Data to quickly check if a path exists in the vanilla files.
    vanilla_folders: HashSet<String>,

    /// Data to quickly check if a path exists in the parent mod files.
    #[serde(skip_serializing, skip_deserializing)]
    parent_folders: HashSet<String>,

    /// List of vanilla loose paths lowercased, with their casing counterparts. To quickly find files.
    #[serde(skip_serializing, skip_deserializing)]
    vanilla_loose_paths: HashMap<String, Vec<String>>,

    /// List of vanilla paths lowercased, with their casing counterparts. To quickly find files.
    vanilla_paths: HashMap<String, Vec<String>>,

    /// List of parent paths lowercased, with their casing counterparts. To quickly find files.
    ///
    /// Not serialized, regenerated from parent Packs on rebuild.
    #[serde(skip_serializing, skip_deserializing)]
    parent_paths: HashMap<String, Vec<String>>,

    /// Cached data for local tables.
    ///
    /// This is for runtime caching, and it must not be serialized to disk.
    #[serde(skip_serializing, skip_deserializing)]
    local_tables_references: HashMap<String, HashMap<i32, TableReferences>>,

    /// Data from all the locs, so we can quickly search for a loc entry.
    #[serde(skip_serializing, skip_deserializing)]
    localisation_data: HashMap<String, String>,

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
    /// If a schema is not passed, no tables/locs will be pre-decoded. Make sure to decode them later with [Dependencies::decode_tables].
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

        // Load vanilla loose files (from /data).
        self.load_loose_files(schema, game_info, game_path)?;

        // Load parent mods of the currently loaded Pack.
        self.load_parent_files(schema, parent_pack_names, game_info, game_path)?;

        // Populate the localisation data.
        let loc_files = self.loc_data(true, true).unwrap_or(vec![]);
        let loc_decoded = loc_files.iter()
            .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
            .map(|file| file.data())
            .collect::<Vec<_>>();

        self.localisation_data = loc_decoded.par_iter()
            .flat_map(|data| data.par_iter()
                .map(|entry| (entry[0].data_to_string().to_string(), entry[1].data_to_string().to_string()))
                .collect::<Vec<(_,_)>>()
            ).collect::<HashMap<_,_>>();

        Ok(())
    }

    /// This function generates the dependencies cache for the game provided and returns it.
    pub fn generate_dependencies_cache(game_info: &GameInfo, game_path: &Path, asskit_path: &Option<PathBuf>, ignore_game_files_in_ak: bool) -> Result<Self> {
        let mut cache = Self::default();
        cache.build_date = current_time()?;
        cache.version = VERSION.to_owned();
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

        cache.vanilla_files.keys().for_each(|path| {
            let lower = path.to_lowercase();
            match cache.vanilla_paths.get_mut(&lower) {
                Some(paths) => paths.push(path.to_owned()),
                None => { cache.vanilla_paths.insert(lower, vec![path.to_owned()]); },
            }
        });

        // Load vanilla loose files before processing the AK files, so the AK process has these files available.
        //
        // It's mainly so any loose loc is used in the bruteforcing process.
        cache.load_loose_files(&None, game_info, game_path)?;

        // This one can fail, leaving the dependencies with only game data.
        if let Some(path) = asskit_path {
            let _ = cache.generate_asskit_only_db_tables(path, *game_info.raw_db_version(), ignore_game_files_in_ak);
        }

        Ok(cache)
    }

    /// This function generates a "fake" table list with tables only present in the Assembly Kit.
    ///
    /// This works by processing all the tables from the game's raw table folder and turning them into fake decoded tables,
    /// with version -1. That will allow us to use them for dependency checking and for populating combos.
    ///
    /// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK2 file.
    fn generate_asskit_only_db_tables(&mut self, raw_db_path: &Path, version: i16, ignore_game_files: bool) -> Result<()> {
        let files_to_ignore = if ignore_game_files {
            self.vanilla_tables.keys().map(|table_name| &table_name[..table_name.len() - 7]).collect::<Vec<_>>()
        } else {
            vec![]
        };
        let raw_tables = RawTable::read_all(raw_db_path, version, &files_to_ignore)?;
        let asskit_only_db_tables = raw_tables.par_iter().map(TryFrom::try_from).collect::<Result<Vec<DB>>>()?;

        // We need to bruteforce loc keys for ak tables here, so locs relations are setup correctly for ak tables.
        let mut asskit_only_db_tables = asskit_only_db_tables.par_iter().map(|table| (table.table_name().to_owned(), table.clone())).collect::<HashMap<String, DB>>();

        let decode_extra_data = DecodeableExtraData::default();
        let extra_data = Some(decode_extra_data);

        // Vanilla loose files.
        let mut files = self.vanilla_loose_locs.iter().filter_map(|path| {
            self.vanilla_loose_files.remove(path).map(|file| (path.to_owned(), file))
        }).collect::<Vec<_>>();

        files.par_iter_mut().for_each(|(_, file)| {
            let _ = file.decode(&extra_data, true, false);
        });

        self.vanilla_loose_files.par_extend(files);

        // Vanilla files.
        let mut files = self.vanilla_locs.iter().filter_map(|path| {
            self.vanilla_files.remove(path).map(|file| (path.to_owned(), file))
        }).collect::<Vec<_>>();

        files.par_iter_mut().for_each(|(_, file)| {
            let _ = file.decode(&extra_data, true, false);
        });

        self.vanilla_files.par_extend(files);

        self.bruteforce_loc_key_order(&mut Schema::default(), None, Some(&mut asskit_only_db_tables))?;
        self.asskit_only_db_tables = asskit_only_db_tables;

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
                     Some((db.table_name().to_owned(), self.generate_references(db.table_name(), db.definition())))
                } else { None }
            } else { None }
        }).collect::<HashMap<_, _>>();

        self.local_tables_references.extend(local_tables_references);
    }

    /// This function builds the local db references data for the table with the definition you pass to and stores it in the cache.
    pub fn generate_local_definition_references(&mut self, table_name: &str, definition: &Definition) {
        self.local_tables_references.insert(table_name.to_owned(), self.generate_references(table_name, definition));
    }

    /// This function builds the local db references data for the table with the definition you pass to, and returns it.
    pub fn generate_references(&self, local_table_name: &str, definition: &Definition) -> HashMap<i32, TableReferences> {
        let patches = Some(definition.patches());
        let fields_processed = definition.fields_processed();

        fields_processed.iter().enumerate().filter_map(|(column, field)| {
            match field.is_reference(patches) {
                Some((ref ref_table, ref ref_column)) => {
                    if !ref_table.is_empty() && !ref_column.is_empty() {
                        let ref_table = format!("{ref_table}_tables");

                        // Get his lookup data if it has it.
                        let lookup_data = if let Some(ref data) = field.lookup(patches) { data.to_vec() } else { Vec::with_capacity(0) };
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
                },

                // In the fallback case (no references) we still need to check for lookup data within our table and the locs.
                None => {
                    if let Some(ref lookup_data) = field.lookup(patches) {

                        // Only single-keyed tables can have lookups.
                        if field.is_key(patches) && fields_processed.iter().filter(|x| x.is_key(patches)).count() == 1 {
                            let ref_table = local_table_name;
                            let ref_column = field.name();

                            // Get his lookup data if it has it.
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
                },
            }
        }).collect::<HashMap<_, _>>()
    }

    /// This function tries to load dependencies from the path provided.
    pub fn load(file_path: &Path, schema: &Option<Schema>) -> Result<Self> {

        // Optimization: Instead of a big file, we split the dependencies in 3 files. Why?
        // Because bincode is not multithreaded and, while reading 3 medium files is slower than a big one,
        // deserializing 3 medium files in 3 separate threads is way faster than 1 big file in 1 thread.
        let mut file_path_1 = file_path.to_path_buf();
        let handle_1: JoinHandle<Result<(u64, String, Vec<RFile>)>> = spawn(move || {
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
        let handle_3: JoinHandle<Result<(HashMap<String, Vec<String>>, HashSet<String>, HashSet<String>, HashMap<String, Vec<String>>, HashMap<String, DB>)>> = spawn(move || {
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
        let mut vanilla_files: HashMap<_,_> = data_1.2.into_par_iter().map(|file| (file.path_in_container_raw().to_owned(), file)).collect();
        vanilla_files.par_extend(data_2.into_par_iter().map(|file| (file.path_in_container_raw().to_owned(), file)));

        dependencies.build_date = data_1.0;
        dependencies.version = data_1.1;
        dependencies.vanilla_files = vanilla_files;
        dependencies.vanilla_tables = data_3.0;
        dependencies.vanilla_locs = data_3.1;
        dependencies.vanilla_folders = data_3.2;
        dependencies.vanilla_paths = data_3.3;
        dependencies.asskit_only_db_tables = data_3.4;

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
        let serialized_1: Vec<u8> = bincode::serialize(&(&self.build_date, &self.version, &vanilla_files_1))?;
        let serialized_2: Vec<u8> = bincode::serialize(&vanilla_files_2)?;
        let serialized_3: Vec<u8> = bincode::serialize(&(&self.vanilla_tables, &self.vanilla_locs, &self.vanilla_folders, &self.vanilla_paths, &self.asskit_only_db_tables))?;

        file_1.write_all(&serialized_1).map_err(RLibError::from)?;
        file_2.write_all(&serialized_2).map_err(RLibError::from)?;
        file_3.write_all(&serialized_3).map_err(From::from)
    }

    /// This function is used to check if the game files used to generate the dependencies cache have changed, requiring an update.
    pub fn needs_updating(&self, game_info: &GameInfo, game_path: &Path) -> Result<bool> {
        let ca_paths = game_info.ca_packs_paths(game_path)?;
        let last_date = last_modified_time_from_files(&ca_paths)?;
        Ok(last_date > self.build_date || self.version != VERSION)
    }

    /// This function loads all the loose files within the game's /data folder.
    fn load_loose_files(&mut self, schema: &Option<Schema>, game_info: &GameInfo, game_path: &Path) -> Result<()> {
        self.vanilla_loose_files.clear();
        self.vanilla_loose_tables.clear();
        self.vanilla_loose_locs.clear();
        self.vanilla_loose_folders.clear();
        self.vanilla_loose_paths.clear();

        let game_data_path = game_info.data_path(game_path)?;
        let game_data_path_str = game_data_path.to_string_lossy().replace("\\", "/");

        self.vanilla_loose_files = files_from_subdir(&game_data_path, true)?
            .into_par_iter()
            .filter_map(|path| {
                let mut path = path.to_string_lossy().replace("\\", "/");
                if !path.ends_with(".pack") {
                    if let Ok(mut rfile) = RFile::new_from_file(&path) {
                        let subpath = path.split_off(game_data_path_str.len() + 1);
                        rfile.set_path_in_container_raw(&subpath);
                        let _ = rfile.guess_file_type();
                        Some((subpath, rfile))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<HashMap<String, RFile>>();

        let cacheable = self.vanilla_loose_files.par_iter_mut()
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
                            match self.vanilla_loose_tables.get_mut(table_name) {
                                Some(table_paths) => table_paths.push(file.path_in_container_raw().to_owned()),
                                None => { self.vanilla_loose_tables.insert(table_name.to_owned(), vec![file.path_in_container_raw().to_owned()]); },
                            }
                        }
                    }
                    FileType::Loc => {
                        self.vanilla_loose_locs.insert(file.path_in_container_raw().to_owned());
                    }
                    _ => {}
                }
            }
        );

        self.vanilla_loose_folders = self.vanilla_loose_files.par_iter().filter_map(|(path, _)| {
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

        self.vanilla_loose_files.keys().for_each(|path| {
            let lower = path.to_lowercase();
            match self.vanilla_loose_paths.get_mut(&lower) {
                Some(paths) => paths.push(path.to_owned()),
                None => { self.vanilla_loose_paths.insert(lower, vec![path.to_owned()]); },
            }
        });

        // Only decode the tables if we passed a schema. If not, it's responsability of the user to decode them later.
        if let Some(schema) = schema {
            let mut decode_extra_data = DecodeableExtraData::default();
            decode_extra_data.set_schema(Some(schema));
            let extra_data = Some(decode_extra_data);

            let mut files = self.vanilla_loose_locs.iter().chain(self.vanilla_loose_tables.values().flatten()).filter_map(|path| {
                self.vanilla_loose_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.vanilla_loose_files.par_extend(files);
        }

        Ok(())
    }


    /// This function loads all the loose files within the game's /data folder.
    fn load_parent_files(&mut self, schema: &Option<Schema>, parent_pack_names: &[String], game_info: &GameInfo, game_path: &Path) -> Result<()> {
        self.parent_files.clear();
        self.parent_tables.clear();
        self.parent_locs.clear();
        self.parent_folders.clear();
        self.parent_paths.clear();

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

        self.parent_files.keys().for_each(|path| {
            let lower = path.to_lowercase();
            match self.parent_paths.get_mut(&lower) {
                Some(paths) => paths.push(path.to_owned()),
                None => { self.parent_paths.insert(lower, vec![path.to_owned()]); },
            }
        });

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

    /// This function loads all the parent [Packs](rpfm_lib::files::pack::Pack) provided as `parent_pack_names` as dependencies,
    /// taking care of also loading all dependencies of all of them, if they're not already loaded.
    fn load_parent_packs(&mut self, parent_pack_names: &[String], game_info: &GameInfo, game_path: &Path) -> Result<()> {
        let data_packs_paths = game_info.data_packs_paths(game_path).unwrap_or(vec![]);
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

            // First check in /data. If we have packs there, do not bother checking for external Packs.
            if let Some(path) = data_paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false) {
                    already_loaded.push(pack_name.to_owned());
                    pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(pack_name, already_loaded, data_paths, external_path));
                    self.parent_files.extend(pack.files().clone());
                }
            }

            // If the Packs are not found in data, check in content.
            else if let Some(ref paths) = external_path {
                if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                    if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], true, false) {
                        already_loaded.push(pack_name.to_owned());
                        pack.dependencies().iter().for_each(|pack_name| self.load_parent_pack(pack_name, already_loaded, data_paths, external_path));
                        self.parent_files.extend(pack.files().clone());
                    }
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

            // Vanilla loose files.
            let mut files = self.vanilla_loose_locs.iter().chain(self.vanilla_loose_tables.values().flatten()).filter_map(|path| {
                self.vanilla_loose_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.vanilla_loose_files.par_extend(files);

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
        let file_path = if file_path.starts_with('/') {
            &file_path[1..]
        } else {
            file_path
        };

        if include_parent {

            // Even on case-insensitive searches, try to use get first. We may get lucky.
            if let Some(file) = self.parent_files.get(file_path) {
                return Ok(file);
            }

            if case_insensitive {
                let lower = file_path.to_lowercase();
                if let Some(file) = self.parent_paths.get(&lower).map(|paths| self.parent_files.get(&paths[0])).flatten() {
                    return Ok(file);
                }
            }
        }

        if include_vanilla {

            // Even on case-insensitive searches, try to use get first. We may get lucky.
            if let Some(file) = self.vanilla_files.get(file_path) {
                return Ok(file);
            }

            if case_insensitive {
                let lower = file_path.to_lowercase();
                if let Some(file) = self.vanilla_paths.get(&lower).map(|paths| self.vanilla_files.get(&paths[0])).flatten() {
                    return Ok(file);
                }

            }

            // Same check for loose paths.
            if let Some(file) = self.vanilla_loose_files.get(file_path) {
                return Ok(file);
            }

            if case_insensitive {
                let lower = file_path.to_lowercase();
                if let Some(file) = self.vanilla_loose_paths.get(&lower).map(|paths| self.vanilla_loose_files.get(&paths[0])).flatten() {
                    return Ok(file);
                }
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

            if let Some(file) = self.vanilla_loose_files.get_mut(file_path) {
                return Ok(file);
            }
        }

        Err(RLibError::DependenciesCacheFileNotFound(file_path.to_owned()))
    }

    /// This function returns a reference to all files corresponding to the provided paths.
    pub fn files_by_path(&self, file_paths: &[ContainerPath], include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> HashMap<String, &RFile> {
        let (file_paths, folder_paths): (Vec<_>, Vec<_>) = file_paths.iter().partition_map(|file_path| match file_path {
            ContainerPath::File(file_path) => Either::Left(file_path.to_owned()),
            ContainerPath::Folder(file_path) => Either::Right(file_path.to_owned()),
        });

        let mut hashmap = HashMap::new();

        // File check.
        if !file_paths.is_empty() {
            hashmap.extend(file_paths.par_iter().filter_map(|file_path| self.file(file_path, include_vanilla, include_parent, case_insensitive).ok().map(|file| (file_path.to_owned(), file))).collect::<Vec<(_,_)>>());
        }

        // Folder check.
        if !folder_paths.is_empty() {
            hashmap.extend(folder_paths.into_par_iter().flat_map(|folder_path| {
                let mut folder = vec![];
                let folder_path = folder_path.to_owned() + "/";
                if include_vanilla {

                    if folder_path == "/" {
                        folder.extend(self.vanilla_loose_files.par_iter()
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());

                        folder.extend(self.vanilla_files.par_iter()
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());

                    } else {
                        folder.extend(self.vanilla_loose_files.par_iter()
                            .filter(|(path, _)| {
                                if case_insensitive {
                                    starts_with_case_insensitive(path, &folder_path)
                                } else {
                                    path.starts_with(&folder_path)
                                }
                            })
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());

                        folder.extend(self.vanilla_files.par_iter()
                            .filter(|(path, _)| {
                                if case_insensitive {
                                    starts_with_case_insensitive(path, &folder_path)
                                } else {
                                    path.starts_with(&folder_path)
                                }
                            })
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());
                    }
                }

                if include_parent {
                    if folder_path == "/" {
                        folder.extend(self.parent_files.par_iter()
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());

                    } else {
                        folder.extend(self.parent_files.par_iter()
                            .filter(|(path, _)| {
                                if case_insensitive {
                                    starts_with_case_insensitive(path, &folder_path)
                                } else {
                                    path.starts_with(&folder_path)
                                }
                            })
                            .map(|(path, file)| (path.to_owned(), file))
                            .collect::<Vec<(_,_)>>());
                    }
                }
                folder
            }).collect::<Vec<(_,_)>>());
        }

        hashmap
    }

    /// This function returns a reference to all files of the specified FileTypes from the cache, if any, along with their path.
    pub fn files_by_types(&self, file_types: &[FileType], include_vanilla: bool, include_parent: bool) -> HashMap<String, &RFile> {
        let mut files = HashMap::new();

        // Vanilla first, so if parent files are found, they overwrite vanilla files.
        if include_vanilla {
            files.extend(self.vanilla_loose_files.par_iter().chain(self.vanilla_files.par_iter())
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        if include_parent {
            files.extend(self.parent_files.par_iter()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        files
    }

    /// This function returns a mutable reference to all files of the specified FileTypes from the cache, if any, along with their path.
    pub fn files_by_types_mut(&mut self, file_types: &[FileType], include_vanilla: bool, include_parent: bool) -> HashMap<String, &mut RFile> {
        let mut files = HashMap::new();

        // Vanilla first, so if parent files are found, they overwrite vanilla files.
        if include_vanilla {
            files.extend(self.vanilla_loose_files.par_iter_mut().chain(self.vanilla_files.par_iter_mut())
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        if include_parent {
            files.extend(self.parent_files.par_iter_mut()
                .filter(|(_, file)| file_types.contains(&file.file_type()))
                .map(|(path, file)| (path.to_owned(), file))
                .collect::<HashMap<_,_>>());
        }

        files
    }

    /// This function returns the vanilla/parent locs from the cache, according to the params you pass it.
    ///
    /// It returns them in the order the game will load them.
    pub fn loc_data(&self, include_vanilla: bool, include_parent: bool) -> Result<Vec<&RFile>> {
        let mut cache = vec![];

        if include_vanilla {
            let mut vanilla_loose_locs = self.vanilla_loose_locs.iter().collect::<Vec<_>>();
            vanilla_loose_locs.sort();

            for path in &vanilla_loose_locs {
                if let Some(file) = self.vanilla_loose_files.get(*path) {
                    cache.push(file);
                }
            }

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
            if let Some(vanilla_loose_tables) = self.vanilla_loose_tables.get(table_name) {
                let mut vanilla_loose_tables = vanilla_loose_tables.to_vec();
                vanilla_loose_tables.sort();

                for path in &vanilla_loose_tables {
                    if let Some(file) = self.vanilla_loose_files.get(path) {
                        cache.push(file);
                    }
                }
            }

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
                let mut vanilla_loose_tables = self.vanilla_loose_tables.values().flatten().collect::<Vec<_>>();
                vanilla_loose_tables.sort();

                for path in &vanilla_loose_tables {
                    if let Some(file) = self.vanilla_loose_files.get(*path) {
                        cache.push(file);
                    }
                }

                let mut vanilla_tables = self.vanilla_tables.values().flatten().collect::<Vec<_>>();
                vanilla_tables.sort();

                for path in &vanilla_tables {
                    if let Some(file) = self.vanilla_files.get(*path) {
                        cache.push(file);
                    }
                }
            }

            if include_loc {
                let mut vanilla_loose_locs = self.vanilla_loose_locs.iter().collect::<Vec<_>>();
                vanilla_loose_locs.sort();

                for path in &vanilla_loose_locs {
                    if let Some(file) = self.vanilla_loose_files.get(*path) {
                        cache.push(file);
                    }
                }

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
    pub fn db_reference_data(&self, pack: &Pack, table_name: &str, definition: &Definition, loc_data: &Option<HashMap<Cow<str>, Cow<str>>>) -> HashMap<i32, TableReferences> {

        // First check if the data is already cached, to speed up things.
        let mut vanilla_references = match self.local_tables_references.get(table_name) {
            Some(cached_data) => cached_data.clone(),
            None => panic!("To be fixed: If you see this, you forgot to call generate_local_db_references before this."),
        };

        // If we receive prenade loc data (because this may trigger on many files at the same time), don't calculate it here.
        let (_loc_files, loc_decoded) = if loc_data.is_some() {
            (vec![], vec![])
        } else {
            let loc_files = pack.files_by_type(&[FileType::Loc]);
            let loc_decoded = loc_files.iter()
                .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
                .map(|file| file.data())
                .collect::<Vec<_>>();
            (loc_files, loc_decoded)
        };

        let mut _loc_data_dummy = HashMap::new();
        let loc_data = if let Some(ref loc_data) = loc_data {
            loc_data
        } else {
            _loc_data_dummy = loc_decoded.par_iter()
                .flat_map(|data| data.par_iter()
                    .map(|entry| (entry[0].data_to_string(), entry[1].data_to_string()))
                    .collect::<Vec<(_,_)>>()
                ).collect::<HashMap<_,_>>();
            &_loc_data_dummy
        };

        let patches = Some(definition.patches());
        let fields_processed = definition.fields_processed();
        let local_references = fields_processed.par_iter().enumerate().filter_map(|(column, field)| {
            match field.is_reference(patches) {
                Some((ref ref_table, ref ref_column)) => {
                    if !ref_table.is_empty() && !ref_column.is_empty() {

                        // Get his lookup data if it has it.
                        let lookup_data = if let Some(ref data) = field.lookup(patches) { data.to_vec() } else { Vec::with_capacity(0) };
                        let mut references = TableReferences::default();
                        *references.field_name_mut() = field.name().to_owned();

                        let _local_found = Self::db_reference_data_from_local_pack(&mut references, (ref_table, ref_column, &lookup_data), pack, &loc_data);

                        Some((column as i32, references))
                    } else { None }
                }

                // In the fallback case (no references) we still need to check for lookup data within our table and the locs.
                None => {
                    if let Some(ref lookup_data) = field.lookup(patches) {

                        // Only single-keyed tables can have lookups.
                        if field.is_key(patches) && fields_processed.iter().filter(|x| x.is_key(patches)).count() == 1 {

                            // The fallback here is to avoid crashes on packs that have renamed folders.
                            let ref_table = if table_name.ends_with("_tables") && table_name.len() > 7 {
                                table_name.to_owned().drain(..table_name.len() - 7).collect()
                            } else {
                                table_name.to_owned()
                            };

                            let ref_column = field.name();

                            // Get his lookup data if it has it.
                            let mut references = TableReferences::default();
                            *references.field_name_mut() = field.name().to_owned();

                            let _local_found = Self::db_reference_data_from_local_pack(&mut references, (&ref_table, ref_column, &lookup_data), pack, &loc_data);

                            Some((column as i32, references))
                        } else { None }
                    } else { None }
                }
            }
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
                    let definition = db.definition();
                    let fields_processed = definition.fields_processed();
                    let localised_fields = definition.localised_fields();
                    let localised_order = definition.localised_key_order();
                    let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);

                    // Due to how it works, if we don't have reference data, we CANNOT have lookup data, as we'll save it in a hashmap and we'll need the ref data to use it as key.
                    if let Some(ref_column_index) = ref_column_index {

                        // This one is over localised first, then over normal fields.
                        let ref_lookup_columns_index = ref_lookup_columns.iter().flat_map(|column| {
                            match localised_fields.iter().position(|x| x.name() == column) {
                                Some(index) => Some((true, index)),
                                None => fields_processed.iter().position(|x| x.name() == column).map(|index| (false, index))
                            }
                        }).collect::<Vec<_>>();

                        let name_short = db.table_name_without_tables();
                        let data = db.data().par_iter().map(|row| {
                            let mut lookup_data = Vec::with_capacity(ref_lookup_columns_index.len());

                            // First, we get the reference data.
                            let reference_data = row[ref_column_index].data_to_string().to_string();

                            // Then, we get the lookup data.
                            for (is_loc, column) in ref_lookup_columns_index.iter() {
                                if *is_loc {

                                    // Optimisation: This is way faster than format when done in-mass.
                                    let loc_values = localised_order.iter().map(|pos| row[*pos as usize].data_to_string()).join("");
                                    let mut loc_key = String::with_capacity(2 + name_short.len() + localised_fields[*column].name().len() + loc_values.len());
                                    loc_key.push_str(&name_short);
                                    loc_key.push('_');
                                    loc_key.push_str(localised_fields[*column].name());
                                    loc_key.push('_');
                                    loc_key.push_str(&loc_values);

                                    if let Some(data) = self.localisation_data.get(&loc_key) {
                                        lookup_data.push(Cow::from(data));
                                    }
                                }

                                else {
                                    lookup_data.push(row[*column].data_to_string());
                                }
                            }
                            (reference_data, lookup_data.join(":"))
                        }).collect::<Vec<(_,_)>>();

                        references.data.extend(data);
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
                let fields_processed = table.definition().fields_processed();
                let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);
                let ref_lookup_columns_index = ref_lookup_columns.iter().map(|column| fields_processed.iter().position(|x| x.name() == column)).collect::<Vec<_>>();

                for row in &*table.data() {
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
                true
            },
            None => false,
        }
    }

    /// This function returns the reference/lookup data of all relevant columns of a DB Table from the provided Pack.
    fn db_reference_data_from_local_pack(references: &mut TableReferences, reference_info: (&str, &str, &[String]), pack: &Pack, loc_data: &HashMap<Cow<str>, Cow<str>>) -> bool {

        let mut data_found = false;
        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        pack.files_by_path(&ContainerPath::Folder(format!("db/{ref_table}_tables")), true).iter()
            .for_each(|file| {
            if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                let definition = db.definition();
                let fields_processed = definition.fields_processed();
                let localised_fields = definition.localised_fields();
                let localised_order = definition.localised_key_order();
                let ref_column_index = fields_processed.iter().position(|x| x.name() == ref_column);

                // This one is over localised first, then over normal fields.
                let ref_lookup_columns_index = ref_lookup_columns.iter().flat_map(|column| {
                    match localised_fields.iter().position(|x| x.name() == column) {
                        Some(index) => Some((true, index)),
                        None => fields_processed.iter().position(|x| x.name() == column).map(|index| (false, index))
                    }
                }).collect::<Vec<_>>();

                let data = db.data();
                for row in &*db.data() {
                    let mut reference_data = String::new();
                    let mut lookup_data = Vec::with_capacity(ref_lookup_columns_index.len());

                    // First, we get the reference data.
                    if let Some(index) = ref_column_index {
                        reference_data = row[index].data_to_string().to_string();
                    }

                    // Then, we get the lookup data.
                    for (is_loc, column) in ref_lookup_columns_index.iter() {
                        if *is_loc {
                            let loc_values = localised_order.iter().map(|pos| row[*pos as usize].data_to_string()).join("");
                            let mut loc_key = String::with_capacity(2 + ref_table.len() + localised_fields[*column].name().len() + loc_values.len());
                            loc_key.push_str(&ref_table);
                            loc_key.push('_');
                            loc_key.push_str(localised_fields[*column].name());
                            loc_key.push('_');
                            loc_key.push_str(&loc_values);

                            if let Some(data) = loc_data.get(&*loc_key) {
                                lookup_data.push(data.clone());
                            }
                        }

                        else {
                            lookup_data.push(row[*column].data_to_string());
                        }
                    }

                    references.data.insert(reference_data, lookup_data.into_iter().join(":"));
                }

                if !&data.is_empty() && !data_found {
                    data_found = true;
                }
            }
        });
        data_found
    }

    /// This function returns the table/column/key from the provided loc key.
    ///
    /// We return the table without "_tables". Keep that in mind if you use this.
    pub fn loc_key_source(&self, key: &str) -> Option<(String, String, Vec<String>)> {
        let key_split = key.split('_').collect::<Vec<_>>();

        // We don't know how much of the string the key the table is, so we try removing parts until we find a table that matches.
        // in reverse so longer table names have priority in case of collision.
        for (index, _) in key_split.iter().enumerate().rev() {

            // Index 0 would mean empty table name.
            if index >= 1 {

                let mut table_name = key_split[..index].join("_");
                let full_table_name = format!("{table_name}_tables");

                if let Ok(rfiles) = self.db_data(&full_table_name, true, false) {
                    let mut decoded = rfiles.iter()
                        .filter_map(|x| if let Ok(RFileDecoded::DB(table)) = x.decoded() {
                            Some(table)
                        } else {
                            None
                        }).collect::<Vec<_>>();

                    // Also add the ak files if present.
                    if let Some(ak_file) = self.asskit_only_db_tables().get(&full_table_name) {
                        decoded.push(ak_file);
                    }

                    for table in decoded {
                        let definition = table.definition();
                        let localised_fields = definition.localised_fields();
                        let localised_key_order = definition.localised_key_order();
                        if !localised_fields.is_empty() {
                            let mut field = String::new();

                            // Loop to get the column.
                            for (second_index, value) in key_split[index..].iter().enumerate() {
                                field.push_str(value);

                                if localised_fields.iter().any(|x| x.name() == field) {

                                    // If we reached this, the rest is the value.
                                    let key_data = &key_split[index + second_index + 1..].join("_");

                                    // Once we get the key, we need to use the stored loc order to find out to what specific line it belongs.
                                    // And yes, this means checking every single fucking line in every single table.
                                    let data = table.data();
                                    for row in data.iter() {
                                        let generated_key_split = localised_key_order.iter().map(|col| row[*col as usize].data_to_string()).collect::<Vec<_>>();
                                        let generated_key = generated_key_split.join("");
                                        if &generated_key == key_data {
                                            return Some((table_name, field, generated_key_split.iter().map(|x| x.to_string()).collect()));
                                        }
                                    }
                                }

                                field.push('_');
                            }
                        }
                    }
                }

                // Add an underscore before adding the next part of the table name in the next loop.
                table_name.push('_');
            }
        }

        None
    }

    //-----------------------------------//
    // Utility functions.
    //-----------------------------------//

    /// This function returns if a specific file exists in the dependencies cache.
    pub fn file_exists(&self, file_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> bool {
        if include_parent {
            if self.parent_files.get(file_path).is_some() {
                return true
            } else if case_insensitive {
                let lower = file_path.to_lowercase();
                if self.parent_paths.get(&lower).is_some() {
                    return true
                }
            }
        }

        if include_vanilla {

            if self.vanilla_files.get(file_path).is_some() || self.vanilla_loose_files.get(file_path).is_some() {
                return true
            } else if case_insensitive {
                let lower = file_path.to_lowercase();
                if self.vanilla_paths.get(&lower).is_some() || self.vanilla_loose_paths.get(&lower).is_some() {
                    return true
                }
            }
        }

        false
    }

    /// This function returns if a specific folder exists in the dependencies cache.
    pub fn folder_exists(&self, folder_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> bool {
        if include_parent {
            if self.parent_folders.get(folder_path).is_some() {
                return true
            } else if case_insensitive && self.parent_folders.par_iter().any(|path| caseless::canonical_caseless_match_str(path, folder_path)) {
                return true
            }
        }

        if include_vanilla {
            if self.vanilla_folders.get(folder_path).is_some() || self.vanilla_loose_folders.get(folder_path).is_some() {
                return true
            } else if case_insensitive && self.vanilla_folders.par_iter().chain(self.vanilla_loose_folders.par_iter()).any(|path| caseless::canonical_caseless_match_str(path, folder_path)) {
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

            let table = self.vanilla_loose_files.get(table_path)?;
            if let RFileDecoded::DB(table) = table.decoded().ok()? {
                return Some(*table.definition().version());
            }
        }

        None
    }

    /// This function returns the list of values a column of a table has, across all instances of said table in the dependencies and the provided Pack.
    pub fn db_values_from_table_name_and_column_name(&self, pack: Option<&Pack>, table_name: &str, column_name: &str, include_vanilla: bool, include_parent: bool) -> HashSet<String> {
        let mut values = HashSet::new();

        if let Ok(files) = self.db_data(table_name, include_vanilla, include_parent) {
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    if let Some(column) = table.definition().column_position_by_name(column_name) {
                        Some(table.data().par_iter().map(|row| row[column].data_to_string().to_string()).collect::<Vec<_>>())
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        if let Some(pack) = pack {
            let files = pack.files_by_path(&ContainerPath::Folder(format!("db/{table_name}")), true);
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    if let Some(column) = table.definition().column_position_by_name(column_name) {
                        Some(table.data().par_iter().map(|row| row[column].data_to_string().to_string()).collect::<Vec<_>>())
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        values
    }

    /// This function returns the value a table has in the row it has a specific value in a specific column.
    pub fn db_values_from_table_name_and_column_name_for_value(&self, pack: Option<&Pack>, table_name: &str, key_column_name: &str, desired_column_name: &str, include_vanilla: bool, include_parent: bool) -> HashMap<String, String> {
        let mut values = HashMap::new();

        if let Ok(files) = self.db_data(table_name, include_vanilla, include_parent) {
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    if let Some(column) = table.definition().column_position_by_name(key_column_name) {
                        if let Some(desired_column) = table.definition().column_position_by_name(desired_column_name) {
                            Some(table.data().par_iter().map(|row| (row[column].data_to_string().to_string(), row[desired_column].data_to_string().to_string())).collect::<Vec<_>>())
                        } else { None }
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        if let Some(pack) = pack {
            let files = pack.files_by_path(&ContainerPath::Folder(format!("db/{table_name}")), true);
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    if let Some(column) = table.definition().column_position_by_name(key_column_name) {
                        if let Some(desired_column) = table.definition().column_position_by_name(desired_column_name) {
                            Some(table.data().par_iter().map(|row| (row[column].data_to_string().to_string(), row[desired_column].data_to_string().to_string())).collect::<Vec<_>>())
                        } else { None }
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        values
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

    /// This function bruteforces the order in which multikeyed tables get their keys together for loc entries.
    pub fn bruteforce_loc_key_order(&self, schema: &mut Schema, locs: Option<HashMap<String, Vec<String>>>, mut ak_files: Option<&mut HashMap<String, DB>>) -> Result<()> {
        let mut fields_still_not_found = vec![];

        // Get all vanilla loc keys into a big hashmap so we can check them fast.
        let loc_files = self.loc_data(true, false)?;
        let loc_table = loc_files.iter()
            .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
            .flat_map(|file| file.data().to_vec())
            .map(|entry| (entry[0].data_to_string().to_string(), entry[1].data_to_string().to_string()))
            .collect::<HashMap<_,_>>();

        let ak_tables = match ak_files {
            Some(ref tables) => (**tables).clone(),
            None => HashMap::new(),
        };

        // Get all the tables so we don't need to re-fetch each table individually.
        let db_tables = if ak_files.is_some() {
            ak_tables.values().collect::<Vec<_>>()
        } else {
            self.db_and_loc_data(true, false, true, false)?
                .iter()
                .filter_map(|file| if let Ok(RFileDecoded::DB(table)) = file.decoded() { Some(table) } else { None })
                .collect::<Vec<_>>()
        };

        // Merge tables of the same name and version, so we got more chances of loc data being found.
        let mut db_tables_dedup: Vec<DB> = vec![];
        for table in &db_tables {
            match db_tables_dedup.iter_mut().find(|x| x.table_name() == table.table_name() && x.definition().version() == table.definition().version()) {
                Some(db_source) => *db_source = DB::merge(&[db_source, table])?,
                None => db_tables_dedup.push((*table).clone()),
            }
        }

        for table in &db_tables_dedup {
            let definition = table.definition();
            let mut loc_fields = definition.localised_fields().to_vec();

            // We assume the fields that came with the table are correct, as they probably come from the normal procedure to get these.
            //let mut loc_fields_final: Vec<Field> = vec![];
            let mut loc_fields_final = loc_fields.to_vec();

            // If we received possible loc info, add the one we received.
            if let Some(ref loc_fields_info) = locs {
                loc_fields.clear();

                if let Some(loc_names) = loc_fields_info.get(&table.table_name_without_tables()) {
                    for name in loc_names {
                        if loc_fields.iter().all(|x| x.name() != name) {

                            let mut field = Field::default();
                            field.set_name(name.to_string());
                            field.set_field_type(FieldType::StringU8);

                            loc_fields.push(field);
                        }
                    }
                }
            }

            let fields = definition.fields_processed();
            let key_fields = fields.iter()
                .enumerate()
                .filter(|(_, field)| field.is_key(None))
                .collect::<Vec<_>>();

            // Check which fields from the missing field list are actually loc fields.
            let short_table_name = table.table_name_without_tables();
            for localised_field in &loc_fields {
                let localised_key = format!("{}_{}_", short_table_name, localised_field.name());

                // Note: the second check is to avoid a weird bug I'm still not sure why it happens where loc fields get duplicated.
                if loc_table.keys().any(|x| x.starts_with(&localised_key)) && loc_fields_final.iter().all(|x| x.name() != localised_field.name()) {
                    loc_fields_final.push(localised_field.clone());
                }
            }

            // Some fields fail the previous check because the table contains a field with the same name. So we must repeat it with the table fields.
            // There is a weird corner case here where a localised field may start like the name of another table field. We need to avoid that.
            for table_field in &fields {
                if loc_fields_final.iter().all(|x| !x.name().starts_with(table_field.name())) {
                    let localised_key = format!("{}_{}_", short_table_name, table_field.name());
                    if loc_table.keys().any(|x| x.starts_with(&localised_key)) && loc_fields_final.iter().all(|x| x.name() != table_field.name()) {
                        loc_fields_final.push(table_field.clone());
                    }
                }
            }

            for loc_field in &loc_fields {
                if loc_fields_final.iter().all(|x| x.name() != loc_field.name()) {
                    fields_still_not_found.push(format!("{}/{}", table.table_name_without_tables(), loc_field.name()));
                }
            }

            // Save the loc fields.
            if let Some(ak_files) = &mut ak_files {
                let ak_table = ak_files.get_mut(table.table_name()).unwrap();
                let mut definition = ak_table.definition().clone();
                definition.set_localised_fields(loc_fields_final.to_vec());
                ak_table.set_definition(&definition);

            } else if let Some(schema_definition) = schema.definition_by_name_and_version_mut(table.table_name(), *definition.version()) {
                schema_definition.set_localised_fields(loc_fields_final.to_vec());
            }

            // If after updating the loc data we have loc fields, try to find the key order for them.
            if !loc_fields_final.is_empty() {

                // If we only have one key field, don't bother searching.
                let order = if key_fields.len() == 1 {
                    vec![key_fields[0].0 as u32]
                }

                // If we have multiple key fields, we need to test for combinations.
                else {
                    let mut order = Vec::with_capacity(key_fields.len());
                    let combos = key_fields.iter().permutations(key_fields.len());
                    let table_data = table.data();
                    for combo in combos {

                        // Many multikeyed tables admit empty values as part of the key. We need rows with no empty values.
                        // NOTE: While we just need one line to get the order, we check every line to avoid wrong orders due to first line sharing fields.
                        let mut combo_is_valid = true;
                        for row in table_data.iter() {
                            //for (index, _) in &combo {
                            //    if row[*index].data_to_string().is_empty() {
                            //        fail_due_to_empty_keys_in_combos = true;
                            //        //break;
                            //    }
                            //}

                            let mut combined_key = String::new();
                            for (index, _) in &combo {
                                combined_key.push_str(&row[*index].data_to_string());
                            }

                            for localised_field in &loc_fields_final {
                                let localised_key = format!("{}_{}_{}", short_table_name, localised_field.name(), combined_key);
                                match loc_table.get(&localised_key) {
                                    Some(_) => {
                                        if order.is_empty() {
                                            order = combo.iter().map(|(index, _)| *index as u32).collect();
                                        }
                                    }
                                    None => {
                                        combo_is_valid = false;
                                        break;
                                    }
                                }
                            }

                            // If the combo was not valid for a loc field on a line, stop.
                            if !combo_is_valid {
                                break;
                            }
                        }

                        // If the combo is not valid, reset the order and try the next one.
                        if !combo_is_valid {
                            order = vec![];
                            continue;
                        }

                        if !order.is_empty() {
                            break;
                        }
                    }

                    order
                };

                if !order.is_empty() && !loc_fields_final.is_empty() {
                    info!("Bruteforce: loc key order found for table {}, version {}.", table.table_name(), definition.version());
                    if let Some(ak_files) = &mut ak_files {
                        let ak_table = ak_files.get_mut(table.table_name()).unwrap();
                        let mut definition = ak_table.definition().clone();
                        definition.set_localised_key_order(order);
                        ak_table.set_definition(&definition);
                    } else if let Some(schema_definition) = schema.definition_by_name_and_version_mut(table.table_name(), *definition.version()) {
                        schema_definition.set_localised_key_order(order);
                    }
                } else {
                    info!("Bruteforce: loc key order found (but may be incorrect) for table {}, version {}.", table.table_name(), definition.version());

                    // If we don't have locs, make sure to delete any order we had.
                    if loc_fields_final.is_empty() {
                        if let Some(ak_files) = &mut ak_files {
                            let ak_table = ak_files.get_mut(table.table_name()).unwrap();
                            let mut definition = ak_table.definition().clone();
                            definition.set_localised_key_order(vec![]);
                            ak_table.set_definition(&definition);
                        } else if let Some(schema_definition) = schema.definition_by_name_and_version_mut(table.table_name(), *definition.version()) {
                            schema_definition.set_localised_key_order(vec![]);
                        }
                    }
                }
            }

            // Make sure to cleanup any past mess here.
            else if let Some(ak_files) = &mut ak_files {
                let ak_table = ak_files.get_mut(table.table_name()).unwrap();
                let mut definition = ak_table.definition().clone();
                definition.set_localised_key_order(vec![]);
                ak_table.set_definition(&definition);
            } else if let Some(schema_definition) = schema.definition_by_name_and_version_mut(table.table_name(), *definition.version()) {
                schema_definition.set_localised_key_order(vec![]);
            }
        }

        // Dedup this list, because if the game had multiple table files, we'll get duplicated fields.
        fields_still_not_found.sort();
        fields_still_not_found.dedup();
        info!("Bruteforce: fields still not found :{:#?}", fields_still_not_found);

        // Once everything is done, run a check on the loc keys to see if any of them still doesn't match any table/field combo.
        // This will fail if called on cache generation. Only execute it when updating the schema.
        if ak_files.is_none() {
            for key in loc_table.keys().sorted() {
                match self.loc_key_source(key) {
                    Some((_, _, _)) => {},
                    None => info!("-- Bruteforce: cannot find source for loc key {}.", key),
                }
            }
        }

        Ok(())
    }

    /// This function imports a specific table from the data it has in the AK.
    ///
    /// Tables generated with this are VALID.
    pub fn import_from_ak(&self, table_name: &str, schema: &Schema) -> Result<DB> {
        let definition = if let Some(definitions) = schema.definitions_by_table_name_cloned(&table_name) {
            if !definitions.is_empty() {
                definitions[0].clone()
            } else {
                return Err(RLibError::DecodingDBNoDefinitionsFound)
            }
        } else {
            return Err(RLibError::DecodingDBNoDefinitionsFound)
        };

        // Create the new table according to the schema, and import its data from the AK.
        if let Some(ak_file) = self.asskit_only_db_tables().get(table_name) {
            let mut real_table = ak_file.clone();
            real_table.set_definition(&definition);
            Ok(real_table)
        } else {
            return Err(RLibError::AssemblyKitTableNotFound(table_name.to_owned()))
        }
    }

    //-----------------------------------//
    // Dangerous functions.
    //-----------------------------------//

    /// This function manually inserts a loc file from this into the dependencies as a vanilla loc.
    ///
    /// THIS IS DANGEROUS. DO NOT USE IT UNLESS YOU KNOW WHAT YOU'RE DOING.
    pub fn insert_loc_as_vanilla_loc(&mut self, rfile: RFile) {
        let path = rfile.path_in_container_raw().to_owned();
        self.vanilla_files.insert(path.to_owned(), rfile);
        self.vanilla_locs.insert(path);
    }
}
