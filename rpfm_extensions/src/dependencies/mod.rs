//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::mpsc::channel;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{thread, thread::{spawn, JoinHandle}};
use std::time::Duration;

use rpfm_lib::binary::WriteBytes;
use rpfm_lib::error::{Result, RLibError};
use rpfm_lib::files::{Container, ContainerPath, db::DB, DecodeableExtraData, FileType, pack::Pack, RFile, RFileDecoded, table::Table};
use rpfm_lib::games::{GameInfo, supported_games::*};
use rpfm_lib::integrations::{assembly_kit::table_data::RawTable, log::{info, error}};
use rpfm_lib::schema::{Definition, DefinitionPatch, Field, FieldType, Schema};
use rpfm_lib::utils::{current_time, files_from_subdir, last_modified_time_from_files, starts_with_case_insensitive};

use crate::optimizer::{OptimizableContainer, OptimizerOptions};
use crate::START_POS_WORKAROUND_THREAD;
use crate::VERSION;

pub const KEY_DELETES_TABLE_NAME: &str = "twad_key_deletes_tables";

pub const USER_SCRIPT_FILE_NAME: &str = "user.script.txt";
pub const VICTORY_OBJECTIVES_FILE_NAME: &str = "db/victory_objectives.txt";
pub const VICTORY_OBJECTIVES_EXTRACTED_FILE_NAME: &str = "victory_objectives.txt";

pub const GAMES_NEEDING_VICTORY_OBJECTIVES: [&str; 9] = [
    KEY_PHARAOH_DYNASTIES,
    KEY_PHARAOH,
    KEY_TROY,
    KEY_THREE_KINGDOMS,
    KEY_WARHAMMER_2,
    KEY_WARHAMMER,
    KEY_THRONES_OF_BRITANNIA,
    KEY_ATTILA,
    KEY_ROME_2
];

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
    pub fn rebuild(&mut self, schema: &Option<Schema>, parent_pack_names: &[String], file_path: Option<&Path>, game_info: &GameInfo, game_path: &Path, secondary_path: &Path) -> Result<()> {

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
        self.load_parent_files(schema, parent_pack_names, game_info, game_path, secondary_path)?;

        // Populate the localisation data.
        let loc_files = self.loc_data(true, true).unwrap_or_default();
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
    pub fn generate_dependencies_cache(schema: &Option<Schema>, game_info: &GameInfo, game_path: &Path, asskit_path: &Option<PathBuf>, ignore_game_files_in_ak: bool) -> Result<Self> {
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
            let _ = cache.generate_asskit_only_db_tables(schema, path, *game_info.raw_db_version(), ignore_game_files_in_ak);
        }

        Ok(cache)
    }

    /// This function generates a "fake" table list with tables only present in the Assembly Kit.
    ///
    /// This works by processing all the tables from the game's raw table folder and turning them into fake decoded tables,
    /// with version -1. That will allow us to use them for dependency checking and for populating combos.
    ///
    /// To keep things fast, only undecoded or missing (from the game files) tables will be included into the PAK2 file.
    fn generate_asskit_only_db_tables(&mut self, schema: &Option<Schema>, raw_db_path: &Path, version: i16, ignore_game_files: bool) -> Result<()> {
        let files_to_ignore = if ignore_game_files {
            self.vanilla_tables.keys().map(|table_name| &table_name[..table_name.len() - 7]).collect::<Vec<_>>()
        } else {
            vec![]
        };
        let raw_tables = RawTable::read_all(raw_db_path, version, &files_to_ignore)?;
        let asskit_only_db_tables = raw_tables.par_iter()
            .map(|x| match schema {
                Some(schema) => {
                    let mut table_name = x.definition.clone().unwrap().name.unwrap().to_owned();
                    table_name.pop();
                    table_name.pop();
                    table_name.pop();
                    table_name.pop();

                    table_name = format!("{table_name}_tables");

                    let definition = schema.definitions().get(&table_name).and_then(|x| x.first());

                    x.to_db(definition)
                }
                None => x.to_db(None),
            })
            .collect::<Result<Vec<DB>>>()?;

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

        self.bruteforce_loc_key_order(&mut Schema::default(), None, None, Some(&mut asskit_only_db_tables))?;
        self.asskit_only_db_tables = asskit_only_db_tables;

        Ok(())
    }

    /// This function builds the local db references data for the tables you pass to it from the Pack provided.
    ///
    /// Table names must be provided as full names (with *_tables* at the end).
    ///
    /// NOTE: This function, like many others, assumes the tables are already decoded in the Pack. If they're not, they'll be ignored.
    pub fn generate_local_db_references(&mut self, schema: &Schema, pack: &Pack, table_names: &[String]) {

        let local_tables_references = pack.files_by_type(&[FileType::DB]).par_iter().filter_map(|file| {
            if let Ok(RFileDecoded::DB(db)) = file.decoded() {

                // Only generate references for the tables you pass it, or for all if we pass the list of tables empty.
                if table_names.is_empty() || table_names.iter().any(|x| x == db.table_name()) {
                    Some((db.table_name().to_owned(), self.generate_references(schema, db.table_name(), db.definition())))
                } else { None }
            } else { None }
        }).collect::<HashMap<_, _>>();

        self.local_tables_references.extend(local_tables_references);
    }

    /// This function builds the local db references data for the table with the definition you pass to and stores it in the cache.
    pub fn generate_local_definition_references(&mut self, schema: &Schema, table_name: &str, definition: &Definition) {
        self.local_tables_references.insert(table_name.to_owned(), self.generate_references(schema, table_name, definition));
    }

    /// This function builds the local db references data for the table with the definition you pass to, and returns it.
    pub fn generate_references(&self, schema: &Schema, local_table_name: &str, definition: &Definition) -> HashMap<i32, TableReferences> {

        // Trick: before doing this, we modify the definition to include any lookup from any reference,
        // so we are actually able to catch recursive-like lookups without reading multiple tables.
        let mut definition = definition.clone();
        self.add_recursive_lookups_to_definition(schema, &mut definition, local_table_name);

        let patches = Some(definition.patches());
        let fields_processed = definition.fields_processed();

        // Key deletes works in a different way. For it we have to get the names of all the tables,
        // then we retrieve the keys data dinamically when selecting in the ui.
        if local_table_name == KEY_DELETES_TABLE_NAME {
            let mut hashmap = HashMap::new();
            let mut references = TableReferences::default();
            *references.field_name_mut() = "table_name".to_owned();

            for key in schema.definitions().keys() {
                if key.len() > 7 {
                    let table_name = key.to_owned().drain(..key.len() - 7).collect::<String>();
                    references.data.insert(table_name, String::new());
                }
            }

            hashmap.insert(1, references);
            return hashmap;
        }

        fields_processed.par_iter().enumerate().filter_map(|(column, field)| {
            match field.is_reference(patches) {
                Some((ref ref_table, ref ref_column)) => {
                    if !ref_table.is_empty() && !ref_column.is_empty() {
                        let ref_table = format!("{ref_table}_tables");

                        // Get his lookup data if it has it.
                        let lookup_data = if let Some(ref data) = field.lookup_no_patch() { data.to_vec() } else { Vec::with_capacity(0) };
                        let mut references = TableReferences::default();
                        *references.field_name_mut() = field.name().to_owned();

                        let fake_found = self.db_reference_data_from_asskit_tables(&mut references, (&ref_table, ref_column, &lookup_data));
                        let real_found = self.db_reference_data_from_vanilla_and_modded_tables(&mut references, (&ref_table, ref_column, &lookup_data));

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
                    if let Some(ref lookup_data) = field.lookup_no_patch() {

                        // Only single-keyed tables can have lookups.
                        if field.is_key(patches) && fields_processed.iter().filter(|x| x.is_key(patches)).count() == 1 {
                            let ref_table = local_table_name;
                            let ref_column = field.name();

                            // Get his lookup data if it has it.
                            let mut references = TableReferences::default();
                            *references.field_name_mut() = field.name().to_owned();

                            let fake_found = self.db_reference_data_from_asskit_tables(&mut references, (ref_table, ref_column, lookup_data));
                            let real_found = self.db_reference_data_from_vanilla_and_modded_tables(&mut references, (ref_table, ref_column, lookup_data));

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
        let game_data_path_str = game_data_path.to_string_lossy().replace('\\', "/");

        self.vanilla_loose_files = files_from_subdir(&game_data_path, true)?
            .into_par_iter()
            .filter_map(|path| {
                let mut path = path.to_string_lossy().replace('\\', "/");
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
    fn load_parent_files(&mut self, schema: &Option<Schema>, parent_pack_names: &[String], game_info: &GameInfo, game_path: &Path, secondary_path: &Path) -> Result<()> {
        self.parent_files.clear();
        self.parent_tables.clear();
        self.parent_locs.clear();
        self.parent_folders.clear();
        self.parent_paths.clear();

        // Preload parent mods of the currently loaded Pack.
        self.load_parent_packs(parent_pack_names, game_info, game_path, secondary_path)?;
        self.parent_files.par_iter_mut().map(|(_, file)| file.guess_file_type()).collect::<Result<()>>()?;

        // Then build the table/loc lists, for easy access.
        self.parent_files.iter()
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

            let mut files = self.parent_tables.values().flatten().filter_map(|path| {
                self.parent_files.remove(path).map(|file| (path.to_owned(), file))
            }).collect::<Vec<_>>();

            files.par_iter_mut().for_each(|(_, file)| {
                let _ = file.decode(&extra_data, true, false);
            });

            self.parent_files.par_extend(files);
        }

        // Also decode the locs. They don't need an schema.
        let mut files = self.parent_locs.iter().filter_map(|path| {
            self.parent_files.remove(path).map(|file| (path.to_owned(), file))
        }).collect::<Vec<_>>();

        files.par_iter_mut().for_each(|(_, file)| {
            let _ = file.decode(&None, true, false);
        });

        self.parent_files.par_extend(files);

        Ok(())
    }

    /// This function loads all the parent [Packs](rpfm_lib::files::pack::Pack) provided as `parent_pack_names` as dependencies,
    /// taking care of also loading all dependencies of all of them, if they're not already loaded.
    fn load_parent_packs(&mut self, parent_pack_names: &[String], game_info: &GameInfo, game_path: &Path, secondary_path: &Path) -> Result<()> {
        let data_packs_paths = game_info.data_packs_paths(game_path).unwrap_or_default();
        let secondary_packs_paths = game_info.secondary_packs_paths(secondary_path);
        let content_packs_paths = game_info.content_packs_paths(game_path);
        let mut loaded_packfiles = vec![];

        parent_pack_names.iter().for_each(|pack_name| self.load_parent_pack(pack_name, &mut loaded_packfiles, &data_packs_paths, &secondary_packs_paths, &content_packs_paths, game_info));

        Ok(())
    }

    /// This function loads a parent [Pack](rpfm_lib::files::pack::Pack) as a dependency,
    /// taking care of also loading all dependencies of it, if they're not already loaded.
    fn load_parent_pack(
        &mut self,
        pack_name: &str,
        already_loaded: &mut Vec<String>,
        data_paths: &[PathBuf],
        secondary_paths: &Option<Vec<PathBuf>>,
        content_paths: &Option<Vec<PathBuf>>,
        game_info: &GameInfo
    ) {
        // Do not process Packs twice.
        if !already_loaded.contains(&pack_name.to_owned()) {

            // First check in /data. If we have packs there, do not bother checking for external Packs.
            if let Some(path) = data_paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], game_info, true, false, false) {
                    already_loaded.push(pack_name.to_owned());
                    pack.dependencies().iter().for_each(|(_, pack_name)| self.load_parent_pack(pack_name, already_loaded, data_paths, secondary_paths, content_paths, game_info));
                    self.parent_files.extend(pack.files().clone());

                    return;
                }
            }

            // Then check in /secondary. If we have packs there, do not bother checking for content Packs.
            if let Some(ref paths) = secondary_paths {
                if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                    if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], game_info, true, false, false) {
                        already_loaded.push(pack_name.to_owned());
                        pack.dependencies().iter().for_each(|(_, pack_name)| self.load_parent_pack(pack_name, already_loaded, data_paths, secondary_paths, content_paths, game_info));
                        self.parent_files.extend(pack.files().clone());

                        return;
                    }
                }
            }

            // If nothing else works, check in content.
            if let Some(ref paths) = content_paths {
                if let Some(path) = paths.iter().find(|x| x.file_name().unwrap().to_string_lossy() == pack_name) {
                    if let Ok(pack) = Pack::read_and_merge(&[path.to_path_buf()], game_info, true, false, false) {
                        already_loaded.push(pack_name.to_owned());
                        pack.dependencies().iter().for_each(|(_, pack_name)| self.load_parent_pack(pack_name, already_loaded, data_paths, secondary_paths, content_paths, game_info));
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
        let file_path = if let Some(file_path) = file_path.strip_prefix('/') {
            file_path
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
                if let Some(file) = self.parent_paths.get(&lower).and_then(|paths| self.parent_files.get(&paths[0])) {
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
                if let Some(file) = self.vanilla_paths.get(&lower).and_then(|paths| self.vanilla_files.get(&paths[0])) {
                    return Ok(file);
                }

            }

            // Same check for loose paths.
            if let Some(file) = self.vanilla_loose_files.get(file_path) {
                return Ok(file);
            }

            if case_insensitive {
                let lower = file_path.to_lowercase();
                if let Some(file) = self.vanilla_loose_paths.get(&lower).and_then(|paths| self.vanilla_loose_files.get(&paths[0])) {
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
            hashmap.extend(file_paths.par_iter()
                .filter_map(|file_path| self.file(file_path, include_vanilla, include_parent, case_insensitive)
                    .ok()
                    .map(|file| (file_path.to_owned(), file)))
                .collect::<Vec<(_,_)>>()
            );
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

    /// This function returns the vanilla/parent db tables from the cache, according to the params you pass it,
    /// applying to them any datacore from the provided Pack.
    ///
    /// It returns them in the order the game will load them.
    ///
    /// NOTE: table_name is expected to be the table's folder name, with "_tables" at the end.
    pub fn db_data_datacored<'a>(&'a self, table_name: &str, pack: &'a Pack, include_vanilla: bool, include_parent: bool) -> Result<Vec<&'a RFile>> {
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

        let paths = cache.iter()
            .map(|x| x.path_in_container())
            .collect::<Vec<_>>();

        for pack_file in pack.files_by_paths(&paths, true) {
            for cache_file in &mut cache {
                if cache_file.path_in_container() == pack_file.path_in_container() {
                    *cache_file = pack_file;
                    break;
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
    pub fn db_reference_data(&self, schema: &Schema, pack: &Pack, table_name: &str, definition: &Definition, loc_data: &Option<HashMap<Cow<str>, Cow<str>>>) -> HashMap<i32, TableReferences> {

        // First check if the data is already cached, to speed up things.
        //
        // NOTE: The None branch should only trigger in cases were there's a bug. We just let it pass without reference instead of crashing.
        let mut vanilla_references = match self.local_tables_references.get(table_name) {
            Some(cached_data) => cached_data.clone(),
            None => HashMap::new(),
        };

        // If we receive premade loc data (because this may trigger on many files at the same time), don't calculate it here.
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

        // Trick: before doing this, we modify the definition to include any lookup from any reference,
        // so we are actually able to catch recursive-like lookups without reading multiple tables.
        let mut definition = definition.clone();
        self.add_recursive_lookups_to_definition(schema, &mut definition, table_name);

        let patches = Some(definition.patches());
        let fields_processed = definition.fields_processed();
        let local_references = fields_processed.par_iter().enumerate().filter_map(|(column, field)| {
            match field.is_reference(patches) {
                Some((ref ref_table, ref ref_column)) => {
                    if !ref_table.is_empty() && !ref_column.is_empty() {

                        // Get his lookup data if it has it.
                        let lookup_data = if let Some(ref data) = field.lookup_no_patch() { data.to_vec() } else { Vec::with_capacity(0) };
                        let mut references = TableReferences::default();
                        *references.field_name_mut() = field.name().to_owned();

                        let _local_found = self.db_reference_data_from_local_pack(&mut references, (ref_table, ref_column, &lookup_data), pack, loc_data);

                        Some((column as i32, references))
                    } else { None }
                }

                // In the fallback case (no references) we still need to check for lookup data within our table and the locs.
                None => {
                    if let Some(ref lookup_data) = field.lookup_no_patch() {

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

                            let _local_found = self.db_reference_data_from_local_pack(&mut references, (&ref_table, ref_column, lookup_data), pack, loc_data);

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

        for (index, field) in fields_processed.iter().enumerate() {
            match vanilla_references.get_mut(&(index as i32)) {
                Some(references) => {
                    let hardcoded_lookup = field.lookup_hardcoded(patches);
                    if !hardcoded_lookup.is_empty() {
                        references.data.extend(hardcoded_lookup);
                    }
                },
                None => {
                    let mut references = TableReferences::default();
                    *references.field_name_mut() = field.name().to_owned();
                    let hardcoded_lookup = field.lookup_hardcoded(patches);
                    if !hardcoded_lookup.is_empty() {
                        references.data.extend(hardcoded_lookup);
                        vanilla_references.insert(index as i32, references);
                    }
                },
            }
        }

        vanilla_references
    }

    /// This function returns the reference/lookup data of all relevant columns of a DB Table from the vanilla/parent data.
    ///
    /// If reference data was found, the most recent definition of said data is returned.
    fn db_reference_data_from_vanilla_and_modded_tables(&self, references: &mut TableReferences, reference_info: (&str, &str, &[String])) -> Option<Definition> {
        self.db_reference_data_generic(references, reference_info, None, &HashMap::new())
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
    fn db_reference_data_from_local_pack(&self, references: &mut TableReferences, reference_info: (&str, &str, &[String]), pack: &Pack, loc_data: &HashMap<Cow<str>, Cow<str>>) -> Option<Definition> {
        self.db_reference_data_generic(references, reference_info, Some(pack), loc_data)
    }

    fn db_reference_data_generic(&self, references: &mut TableReferences, reference_info: (&str, &str, &[String]), pack: Option<&Pack>, loc_data: &HashMap<Cow<str>, Cow<str>>) -> Option<Definition> {
        let mut data_found: Option<Definition> = None;

        let ref_table = reference_info.0;
        let ref_column = reference_info.1;
        let ref_lookup_columns = reference_info.2;

        let mut cache = HashMap::new();

        // Input is not guaranteed to be in one or another format, so sanitize them here.
        let ref_table_full = if ref_table.ends_with("_tables") {
            ref_table.to_owned()
        } else {
            ref_table.to_owned() + "_tables"
        };

        let files = match pack {
            Some(pack) => {
                let mut files = pack.files_by_path(&ContainerPath::Folder(format!("db/{ref_table_full}")), true);
                files.append(&mut self.db_data(&ref_table_full, true, true).unwrap_or_else(|_| vec![]));
                files
            },
            None => self.db_data(&ref_table_full, true, true).unwrap_or_else(|_| vec![]),
        };

        let mut table_data_cache: HashMap<String, HashMap<String, String>> = HashMap::new();

        files.iter().for_each(|file| {
            if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                let definition = db.definition();
                let fields_processed = definition.fields_processed();

                // Only continue if the column we're referencing actually exists.
                if let Some(ref_column_index) = fields_processed.iter().position(|x| x.name() == ref_column) {

                    // Here we analyze the lookups to build their table cache.
                    let lookups_analyzed = ref_lookup_columns.iter().map(|ref_lookup_path| {
                        let ref_lookup_steps = ref_lookup_path.split(':').map(|x| x.split('#').collect::<Vec<_>>()).collect::<Vec<_>>();
                        let mut is_loc = false;
                        let mut col_pos = 0;

                        for (index, ref_lookup_step) in ref_lookup_steps.iter().enumerate() {
                            if ref_lookup_step.len() == 3 {
                                let lookup_ref_table = ref_lookup_step[0];
                                let lookup_ref_key = ref_lookup_step[1];
                                let lookup_ref_lookup = ref_lookup_step[2];
                                let lookup_ref_table_long = lookup_ref_table.to_owned() + "_tables";

                                // Build the cache for the tables we need to check.
                                if !cache.contains_key(lookup_ref_table) {
                                    let mut files = vec![];

                                    if let Some(pack) = pack {
                                        files.append(&mut pack.files_by_path(&ContainerPath::Folder(format!("db/{lookup_ref_table_long}")), true));
                                    }

                                    // Only add to the cache the files not already there due to being in the pack.
                                    for file in self.db_data(&lookup_ref_table_long, true, true).unwrap_or_else(|_| vec![]) {
                                        if files.iter().all(|x| x.path_in_container_raw() != file.path_in_container_raw()) {
                                            files.push(file);
                                        }
                                    }

                                    if !files.is_empty() {

                                        // Make sure they're in order so if the lookup is in a mod, we have to do less iterations to find it.
                                        files.sort_by(|a, b| a.path_in_container_raw().cmp(b.path_in_container_raw()));
                                        cache.insert(lookup_ref_table.to_owned(), files);
                                    }
                                }

                                // If it's the last step, check if it's a loc, or a table column.
                                if index == ref_lookup_steps.len() - 1 {
                                    if let Some(file) = cache.get(lookup_ref_table) {
                                        if let Some(file) = file.first() {
                                            if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                                                let definition = db.definition();
                                                let fields_processed = definition.fields_processed();
                                                let localised_fields = definition.localised_fields();

                                                match localised_fields.iter().position(|x| x.name() == lookup_ref_lookup) {
                                                    Some(loc_pos) => {
                                                        is_loc = true;
                                                        col_pos = loc_pos;
                                                    },
                                                    None => match fields_processed.iter().position(|x| x.name() == lookup_ref_lookup) {
                                                        Some(pos) => {
                                                            is_loc = false;
                                                            col_pos = pos;
                                                        },
                                                        None => {
                                                            //error!("Missing column for lookup. This is a bug.");
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Build the hashed cache for lookups, so we don't need to iterate again and again for each row.
                                if let Some(files) = cache.get(lookup_ref_table) {
                                    for file in files {
                                        let table_data_column_cache_key = file.path_in_container_raw().to_owned() + &ref_lookup_step.join("++");
                                        if !table_data_cache.contains_key(&table_data_column_cache_key) {
                                            if let Ok(RFileDecoded::DB(db)) = file.decoded() {
                                                let definition = db.definition();
                                                let fields_processed = definition.fields_processed();
                                                let localised_fields = definition.localised_fields();
                                                let localised_order = definition.localised_key_order();

                                                let loc_key = if is_loc {
                                                    if let Some(loc_field) = localised_fields.get(col_pos) {
                                                        let mut loc_key = String::with_capacity(2 + lookup_ref_table.len() + loc_field.name().len());
                                                        loc_key.push_str(lookup_ref_table);
                                                        loc_key.push('_');
                                                        loc_key.push_str(loc_field.name());
                                                        loc_key.push('_');
                                                        loc_key
                                                    } else {
                                                        String::new()
                                                    }
                                                } else {
                                                    String::new()
                                                };

                                                if let Some(source_key_column) = fields_processed.iter().position(|x| x.name() == lookup_ref_key) {

                                                    // Intermediate step cache.
                                                    if index < ref_lookup_steps.len() - 1 {
                                                        if let Some(source_lookup_column) = fields_processed.iter().position(|x| x.name() == lookup_ref_lookup) {
                                                            let cache = db.data().iter()
                                                                .map(|row| (row[source_key_column].data_to_string().to_string(), row[source_lookup_column].data_to_string().to_string()))
                                                                .collect::<HashMap<_,_>>();

                                                            table_data_cache.insert(table_data_column_cache_key.clone(), cache);
                                                        }
                                                    }

                                                    // Locs are already pre-cached. We only need the final part of their key.
                                                    else if is_loc {
                                                        let cache = db.data().iter()
                                                            .map(|row| {
                                                                let mut loc_key = loc_key.to_owned();
                                                                loc_key.push_str(&localised_order.iter().map(|pos| row[*pos as usize].data_to_string()).join(""));
                                                                (row[source_key_column].data_to_string().to_string(), loc_key)
                                                            })
                                                            .collect::<HashMap<_,_>>();
                                                        table_data_cache.insert(table_data_column_cache_key.clone(), cache);
                                                    }

                                                    else {
                                                        let cache = db.data().iter()
                                                            .map(|row| (row[source_key_column].data_to_string().to_string(), row[col_pos].data_to_string().to_string()))
                                                            .collect::<HashMap<_,_>>();

                                                        table_data_cache.insert(table_data_column_cache_key.clone(), cache);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                error!("Badly built lookup. This is a bug.");
                            }
                        }

                        (ref_lookup_steps, is_loc)

                    }).collect::<Vec<_>>();

                    let data = db.data();
                    for row in &*data {
                        let mut lookup_data = Vec::with_capacity(lookups_analyzed.len());

                        // First, we get the reference data.
                        let reference_data = row[ref_column_index].data_to_string();

                        // Then, we get the lookup data. Only calculate it for non-empty keys.
                        for (lookup_steps, is_loc) in lookups_analyzed.iter() {
                            if !reference_data.is_empty() {

                                if let Some(lookup) = self.db_reference_data_generic_lookup(&cache, loc_data, &reference_data, lookup_steps, *is_loc, &table_data_cache) {
                                    lookup_data.push(lookup);
                                }
                            }
                        }

                        references.data.insert(reference_data.to_string(), lookup_data.into_iter().join(":"));
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

        data_found
    }

    fn db_reference_data_generic_lookup(
        &self,
        cache: &HashMap<String, Vec<&RFile>>,
        loc_data: &HashMap<Cow<str>, Cow<str>>,
        lookup_key: &str,
        lookup_steps: &[Vec<&str>],
        is_loc: bool,
        table_data_cache: &HashMap<String, HashMap<String, String>>
    ) -> Option<String> {
        let mut data_found: Option<String> = None;

        if lookup_steps.is_empty() {
            return None;
        }

        let current_step = &lookup_steps[0];
        let source_table = current_step[0];

        if let Some(files) = cache.get(source_table) {
            for file in files {
                let table_data_column_cache_key = file.path_in_container_raw().to_owned() + &current_step.join("++");
                if let Some(table_data_column_cache) = table_data_cache.get(&table_data_column_cache_key) {

                    if let Some(lookup_value) = table_data_column_cache.get(lookup_key) {

                        // If we're not yet in the last step, reduce the steps and repeat.
                        if lookup_steps.len() > 1 {
                            if !lookup_value.is_empty() {
                                data_found = self.db_reference_data_generic_lookup(cache, loc_data, lookup_value, &lookup_steps[1..], is_loc, table_data_cache);
                            }
                        }

                        // If we're on the last step, properly get the lookup data. Locs first.
                        else if is_loc {

                            if let Some(data) = loc_data.get(&**lookup_value) {
                                data_found = Some(data.to_string());
                            } else if let Some(data) = self.localisation_data.get(&**lookup_value) {
                                data_found = Some(data.to_string());
                            } else {
                                data_found = Some(lookup_value.to_string())
                            }
                        }

                        // Then table columns.
                        else {
                            data_found = Some(lookup_value.to_owned());
                        }

                        // If we find a match, don't bother with the rest of the files.
                        break;
                    }
                }
            }
        }

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
            if self.parent_files.contains_key(file_path) {
                return true
            } else if case_insensitive {
                let lower = file_path.to_lowercase();
                if self.parent_paths.contains_key(&lower) {
                    return true
                }
            }
        }

        if include_vanilla {

            if self.vanilla_files.contains_key(file_path) || self.vanilla_loose_files.contains_key(file_path) {
                return true
            } else if case_insensitive {
                let lower = file_path.to_lowercase();
                if self.vanilla_paths.contains_key(&lower) || self.vanilla_loose_paths.contains_key(&lower) {
                    return true
                }
            }
        }

        false
    }

    /// This function returns if a specific folder exists in the dependencies cache.
    pub fn folder_exists(&self, folder_path: &str, include_vanilla: bool, include_parent: bool, case_insensitive: bool) -> bool {
        if include_parent && (
            self.parent_folders.contains(folder_path) ||
            (case_insensitive && self.parent_folders.par_iter().any(|path| caseless::canonical_caseless_match_str(path, folder_path)))
        ) {
            return true
        }

        if include_vanilla && (
            (self.vanilla_folders.contains(folder_path) || self.vanilla_loose_folders.contains(folder_path)) ||
            (case_insensitive && self.vanilla_folders.par_iter().chain(self.vanilla_loose_folders.par_iter()).any(|path| caseless::canonical_caseless_match_str(path, folder_path)))
        ) {
            return true
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
                    table.definition().column_position_by_name(column_name).map(|column| table.data().par_iter().map(|row| row[column].data_to_string().to_string()).collect::<Vec<_>>())
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        if let Some(pack) = pack {
            let files = pack.files_by_path(&ContainerPath::Folder(format!("db/{table_name}")), true);
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    table.definition().column_position_by_name(column_name).map(|column| table.data().par_iter().map(|row| row[column].data_to_string().to_string()).collect::<Vec<_>>())
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
                        table.definition().column_position_by_name(desired_column_name).map(|desired_column| table.data().par_iter().map(|row| (row[column].data_to_string().to_string(), row[desired_column].data_to_string().to_string())).collect::<Vec<_>>())
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        if let Some(pack) = pack {
            let files = pack.files_by_path(&ContainerPath::Folder(format!("db/{table_name}")), true);
            values.extend(files.par_iter().filter_map(|file| {
                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                    if let Some(column) = table.definition().column_position_by_name(key_column_name) {
                        table.definition().column_position_by_name(desired_column_name).map(|desired_column| table.data().par_iter().map(|row| (row[column].data_to_string().to_string(), row[desired_column].data_to_string().to_string())).collect::<Vec<_>>())
                    } else { None }
                } else { None }
            }).flatten().collect::<Vec<_>>());
        }

        values
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the vanilla files.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_db(&mut self, rfile: &mut RFileDecoded) -> Result<(i32, i32, Vec<String>, Vec<String>)> {
        match rfile {
            RFileDecoded::DB(data) => {
                let dep_db_undecoded = self.db_data(data.table_name(), true, false)?;
                let dep_db_decoded = dep_db_undecoded.iter().filter_map(|x| if let Ok(RFileDecoded::DB(decoded)) = x.decoded() { Some(decoded) } else { None }).collect::<Vec<_>>();

                if let Some(vanilla_db) = dep_db_decoded.iter().max_by(|x, y| x.definition().version().cmp(y.definition().version())) {

                    let definition_new = vanilla_db.definition();
                    let definition_old = data.definition().clone();
                    if definition_old != *definition_new {
                        data.set_definition(definition_new);

                        // Get the info about the definition differences.
                        let fields_old = definition_old.fields_processed();
                        let fields_new = definition_new.fields_processed();
                        let fields_deleted = fields_old.iter()
                            .filter(|x| fields_new.iter().all(|y| y.name() != x.name()))
                            .map(|x| x.name().to_owned())
                            .collect::<Vec<_>>();
                        let fields_added = fields_new.iter()
                            .filter(|x| fields_old.iter().all(|y| y.name() != x.name()))
                            .map(|x| x.name().to_owned())
                            .collect::<Vec<_>>();

                        Ok((*definition_old.version(), *definition_new.version(), fields_deleted, fields_added))
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

    /// Function to generate the missing loc entries in a pack.
    pub fn generate_missing_loc_data(&self, pack: &mut Pack) -> Result<Vec<ContainerPath>> {
        let loc_data = self.loc_data(true, true)?;
        let mut existing_locs = HashMap::new();

        for loc in &loc_data {
            if let Ok(RFileDecoded::Loc(ref data)) = loc.decoded() {
                existing_locs.extend(data.table().data().iter().map(|x| (x[0].data_to_string().to_string(), x[1].data_to_string().to_string())));
            }
        }

        pack.generate_missing_loc_data(&existing_locs).map_err(From::from)
    }

    /// This function bruteforces the order in which multikeyed tables get their keys together for loc entries.
    pub fn bruteforce_loc_key_order(&self, schema: &mut Schema, locs: Option<HashMap<String, Vec<String>>>, local_files: Option<&Pack>, mut ak_files: Option<&mut HashMap<String, DB>>) -> Result<()> {
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

        // This is to fix bruteforcing not working on tables like campaigns.
        let local_files = match local_files {
            Some(pack) => pack.files_by_type(&[FileType::DB])
                .iter()
                .filter_map(|x| match x.decoded() {
                    Ok(RFileDecoded::DB(db)) => Some(db),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        // Get all the tables so we don't need to re-fetch each table individually.
        let mut db_tables = if ak_files.is_some() {
            ak_tables.values().collect::<Vec<_>>()
        } else {
            self.db_and_loc_data(true, false, true, false)?
                .iter()
                .filter_map(|file| if let Ok(RFileDecoded::DB(table)) = file.decoded() { Some(table) } else { None })
                .collect::<Vec<_>>()
        };

        db_tables.extend_from_slice(&local_files);

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
        info!("Bruteforce: fields still not found :{fields_still_not_found:#?}");

        // Once everything is done, run a check on the loc keys to see if any of them still doesn't match any table/field combo.
        // This will fail if called on cache generation. Only execute it when updating the schema.
        if ak_files.is_none() {
            for key in loc_table.keys().sorted() {
                match self.loc_key_source(key) {
                    Some((_, _, _)) => {},
                    None => info!("-- Bruteforce: cannot find source for loc key {key}."),
                }
            }
        }

        Ok(())
    }

    /// This function generates automatic schema patches based mainly on bruteforcing and some clever logic.
    #[allow(clippy::if_same_then_else)]
    pub fn generate_automatic_patches(&self, schema: &mut Schema, pack: &Pack) -> Result<()> {
        let mut db_tables = self.db_and_loc_data(true, false, true, false)?
            .iter()
            .filter_map(|file| if let Ok(RFileDecoded::DB(table)) = file.decoded() { Some(table) } else { None })
            .collect::<Vec<_>>();

        db_tables.extend_from_slice(&pack.files_by_type(&[FileType::DB])
            .iter()
            .filter_map(|x| if let Ok(RFileDecoded::DB(db)) = x.decoded() {
                Some(db)
            } else {
                None
            })
            .collect::<Vec<_>>()
        );

        let current_patches = schema.patches_mut();
        let mut new_patches: HashMap<String, DefinitionPatch> = HashMap::new();

        // Cache all image and video paths.
        let image_paths = self.vanilla_files()
            .keys()
            .filter(|x| x.ends_with(".png") || x.ends_with(".tga"))
            .collect::<Vec<_>>();

        let video_paths = self.vanilla_files()
            .keys()
            .filter(|x| x.ends_with(".ca_vp8"))
            .collect::<Vec<_>>();

        for table in &db_tables {
            let definition = table.definition();
            let fields = definition.fields_processed();
            for (column, field) in fields.iter().enumerate() {
                match field.field_type() {
                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => {

                        // Icons can be found by:
                        // - Checking if the data contains ".png" or ".tga".
                        // - Checking if the data contains "Icon" or "Image" in the name.
                        //
                        // Note that if the field contains incomplete/relative paths, this will guess and try to find unique files that match the path.
                        let mut possible_icon = false;
                        let low_name = field.name().to_lowercase();
                        if (low_name.contains("icon") || low_name.contains("image")) &&

                            // Attila. This doesn't match with anything that makes sense.
                            !(table.table_name() == "building_sets_tables" && field.name() == "icon") &&

                            // This really should be called category. It's wrong in the ak.
                            !(table.table_name() == "character_traits_tables" && field.name() == "icon") {
                            possible_icon = true;
                        }

                        // Use hashset for uniqueness and ram usage.
                        let mut possible_relative_paths = table.data().par_iter()
                            .filter_map(|row| {

                                // Only check fields that are not already marked, or are marked but without path (like override_icon in incidents).
                                if !field.is_filename(None) || (
                                        field.is_filename(None) && (
                                            field.filename_relative_path(None).is_none() ||
                                            field.filename_relative_path(None).unwrap().is_empty()
                                        )
                                    ) || (

                                        // This table has an incorrect path by default.
                                        (table.table_name() == "advisors_tables" && field.name() == "advisor_icon_path") ||

                                        // This one is missing subpaths.
                                        (table.table_name() == "campaign_post_battle_captive_options_tables" && field.name() == "icon_path") ||

                                        // This one for some reason points to "working_data" and has no replacement bit.
                                        (table.table_name() == "narrative_viewer_tabs_tables" && field.name() == "image_path") ||

                                        // This one has a path missing the replacement bits.
                                        (table.table_name() == "technology_ui_groups_tables" && field.name() == "optional_background_image")
                                    ) {

                                    // These checks filter out certain problematic cell values:
                                    // - .: means empty in some image fields.
                                    // - x: means empty in some image fields.
                                    // - placeholder: because it's in multiple places and generates false positives.
                                    let mut data = row[column].data_to_string().to_lowercase().replace("\\", "/");

                                    // Fix formatting for cells which start with / or \\.
                                    if data.starts_with("/") {
                                        if data.len() > 1 {
                                            data = data[1..].to_owned();
                                        } else {
                                            data = String::new();
                                        }
                                    }

                                    if !data.is_empty() && !data.ends_with("/") &&
                                        data != "." &&
                                        data != "x" &&
                                        data != "false" &&
                                        data != "building_placeholder" &&
                                        data != "placehoder.png" &&
                                        data != "placeholder" &&
                                        data != "placeholder.tga" &&
                                        data != "placeholder.png" && (
                                            possible_icon ||
                                            data.ends_with(".png") || data.ends_with(".tga")
                                        ) {

                                        let possible_paths = image_paths.iter()

                                            // Manual filters for some fields that are known to trigger hard-to-fix false positives.
                                            .filter(|x| {
                                                if table.table_name() == "aide_de_camp_speeches_tables" && field.name() == "icon_name" {
                                                    x.starts_with("ui/battle ui/adc_icons/")
                                                } else if table.table_name() == "agent_string_subculture_overrides_tables" && field.name() == "icon_path" {
                                                    x.starts_with("ui/campaign ui/agents/icons/")
                                                } else if table.table_name() == "ancillary_types_tables" && field.name() == "ui_icon" {
                                                    x.starts_with("ui/portraits/ancillaries/")
                                                } else if table.table_name() == "battlefield_building_categories_tables" && field.name() == "icon_path" {
                                                    x.starts_with("ui/battle ui/building icons/")
                                                } else if table.table_name() == "bonus_value_uis_tables" && field.name() == "icon" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "building_culture_variants_tables" && field.name() == "icon" {
                                                    x.starts_with("ui/buildings/icons/")
                                                } else if table.table_name() == "campaign_payload_ui_details_tables" && field.name() == "icon" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "campaign_post_battle_captive_options_tables" && field.name() == "icon_path" {
                                                    x.starts_with("ui/campaign ui/captive_option_icons/")
                                                } else if table.table_name() == "capture_point_types_tables" && field.name() == "icon_name" {
                                                    x.starts_with("ui/battle ui/capture_point_icons/")
                                                } else if table.table_name() == "character_skills_tables" && field.name() == "image_path" {
                                                    x.starts_with("ui/campaign ui/skills/")
                                                } else if table.table_name() == "character_traits_tables" && field.name() == "icon_custom" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")

                                                // This is to fix issues with incomplete cursor paths.
                                                } else if table.table_name() == "cursors_tables" && field.name() == "image" {
                                                    !x.starts_with(&(data.to_owned() + "_"))
                                                } else if table.table_name() == "dilemmas_tables" && field.name() == "ui_image" {
                                                    x.starts_with("ui/eventpics/")
                                                } else if table.table_name() == "effect_bundles_tables" && field.name() == "ui_icon" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "effects_tables" && (field.name() == "icon" || field.name() == "icon_negative") {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "faction_groups_tables" && field.name() == "ui_icon" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "incidents_tables" && field.name() == "ui_image" {
                                                    x.starts_with("ui/eventpics/")
                                                } else if table.table_name() == "message_event_strings_tables" && field.name() == "image" {
                                                    x.starts_with("ui/eventpics/")
                                                } else if table.table_name() == "missions_tables" && field.name() == "ui_icon" {
                                                    x.starts_with("ui/campaign ui/message_icons/")

                                                // This is to fix false positives in sequencial missions in Pharaoh.
                                                } else if table.table_name() == "missions_tables" && field.name() == "ui_image" {
                                                    x.starts_with("ui/eventpics/") && x.ends_with(&(data.to_owned() + ".png"))
                                                } else if table.table_name() == "pooled_resources_tables" && field.name() == "optional_icon_path" {
                                                    x.starts_with("ui/skins/")
                                                } else if table.table_name() == "projectile_shot_type_enum_tables" && field.name() == "icon_name" {
                                                    x.starts_with("ui/battle ui/ability_icons/")
                                                } else if table.table_name() == "religions_tables" && field.name() == "ui_icon_path" {
                                                    x.starts_with("ui/campaign ui/religion_icons/")
                                                } else if table.table_name() == "special_ability_phases_tables" && field.name() == "ticker_icon" {
                                                    x.starts_with("ui/battle ui/ability_icons/")
                                                } else if table.table_name() == "technologies_tables" && field.name() == "icon_name" {
                                                    x.starts_with("ui/campaign ui/technologies/")
                                                } else if table.table_name() == "technologies_tables" && field.name() == "info_pic" {
                                                    x.starts_with("ui/eventpics/")
                                                } else if table.table_name() == "trait_categories_tables" && field.name() == "icon_path" {
                                                    x.starts_with("ui/campaign ui/effect_bundles/")
                                                } else if table.table_name() == "ui_unit_groupings_tables" && field.name() == "icon" {
                                                    x.starts_with("ui/common ui/unit_category_icons/")
                                                } else if table.table_name() == "victory_types_tables" && field.name() == "icon" {
                                                    x.starts_with("ui/campaign ui/victory_type_icons/")

                                                // For some reason, some brilliant mind at CA decided to end a video name with ".png". So we need to filter this here.
                                                } else if table.table_name() == "videos_tables" && field.name() == "video_name" {
                                                    x.starts_with("movies/")
                                                } else {
                                                    true
                                                }
                                            })

                                            // This filter is for reducing false positives in these cases:
                                            // - "default" or generic data.
                                            // - "x" value for invalid paths
                                            // - Entries that end in "_", which is used for some button path entries.
                                            .filter(|x| if !data.ends_with('_') {
                                                if !data.contains("/") {
                                                    if !data.contains('.') {
                                                        x.contains(&("/".to_owned() + &data + "."))
                                                    } else {
                                                        x.contains(&("/".to_owned() + &data))
                                                    }
                                                } else {
                                                    x.contains(&data)
                                                }
                                            } else {
                                                false
                                            })

                                            // Replace only the last instance, to avoid weird folder-replacing bugs.
                                            .filter_map(|x| x.rfind(&data).map(|pos| (x, pos)))
                                            .map(|(x, pos)| x[..pos].to_owned() + &x[pos..].replacen(&data, "%", 1))
                                            .collect::<Vec<_>>();


                                        if !possible_paths.is_empty() {
                                            return Some(possible_paths)
                                        }
                                    }
                                }

                                None
                            })
                            .flatten()
                            .collect::<HashSet<String>>();

                        // Video files can be found by:
                        // - Checking if the data contains ".ca_vp8".
                        // - Checking if the data contains "video" in the name.
                        //
                        // Note that if the field contains incomplete/relative paths, this will guess and try to find unique files that match the path.
                        let mut possible_video = false;
                        if low_name.contains("video") {
                            possible_video = true;
                        }

                        possible_relative_paths.extend(
                            table.data().par_iter().filter_map(|row| {

                                // Only check fields that are not already marked, or are marked but without path (like override_icon in incidents).
                                if !field.is_filename(None) || (
                                        field.is_filename(None) && (
                                            field.filename_relative_path(None).is_none() ||
                                            field.filename_relative_path(None).unwrap().is_empty()
                                        )
                                    ) || (

                                        // This table is missing the subpaths (which are valid) by default.
                                        table.table_name() == "videos_tables" && field.name() == "video_name"
                                    ) {

                                    let mut data = row[column].data_to_string().to_lowercase().replace("\\", "/");

                                    // Fix formatting for cells which start with / or \\.
                                    if data.starts_with("/") {
                                        if data.len() > 1 {
                                            data = data[1..].to_owned();
                                        } else {
                                            data = String::new();
                                        }
                                    }

                                    if !data.is_empty() && (
                                            possible_video ||
                                            data.ends_with(".ca_vp8")
                                        ) {

                                        let possible_paths = video_paths.iter()
                                            .filter(|x| {
                                                if table.table_name() == "videos_tables" && field.name() == "video_name" {
                                                    x.starts_with("movies/")
                                                } else {
                                                    true
                                                }
                                            })
                                            // This filter is for reducing false positives in these cases:
                                            // - "%_something", which is used for sequential videos.
                                            // - Faction-specific videos.
                                            .filter(|x| if !data.contains('.') {
                                                    x.contains(&("/".to_owned() + &data + "."))
                                                } else {
                                                    x.contains(&("/".to_owned() + &data))
                                                })

                                            // Replace only the last instance, to avoid weird folder-replacing bugs.
                                            .filter_map(|x| x.rfind(&data).map(|pos| (x, pos)))
                                            .map(|(x, pos)| x[..pos].to_owned() + &x[pos..].replacen(&data, "%", 1))
                                            .collect::<Vec<_>>();


                                        if !possible_paths.is_empty() {
                                            return Some(possible_paths)
                                        }
                                    }
                                }

                                None
                            })
                            .flatten()
                            .collect::<HashSet<String>>()
                        );

                        // Debug message.
                        if !possible_relative_paths.is_empty() && (possible_relative_paths.len() > 1 || (possible_relative_paths.len() == 1 && possible_relative_paths.iter().collect::<Vec<_>>()[0] != "%")) {
                            info!("Checking table {}, field {} ...", table.table_name(), field.name());
                            dbg!(&possible_relative_paths);
                        }

                        // This one has an incorrect relative path value that needs to be patched out.
                        //
                        // This is due to we assigning a name to this column which matches a different column in the AK.
                        if (table.table_name() == "models_building_tables" && field.name() == "logic_file") ||
                            (table.table_name() == "models_sieges_tables" && (field.name() == "model_file" || field.name() == "logic_file" || field.name() == "collision_file")) ||
                            (table.table_name() == "models_deployables_tables" && (field.name() == "model_file" || field.name() == "logic_file" || field.name() == "collision_file")) {
                            possible_relative_paths.clear();
                            possible_relative_paths.insert("%".to_owned());
                        }

                        // These columns have incomplete paths or are incorrectly marked as files. Do not treat them as file paths.
                        if (table.table_name() == "ui_mercenary_recruitment_infos_tables" && field.name() == "hire_button_icon_path") ||
                            (table.table_name() == "battles_tables" && (field.name() == "specification" || field.name() == "battle_environment_audio")) ||
                            (table.table_name() == "factions_tables" && field.name() == "key") ||
                            (table.table_name() == "frontend_faction_leaders_tables" && field.name() == "key") {
                            let mut patch = HashMap::new();
                            patch.insert("is_filename".to_owned(), "false".to_owned());

                            match new_patches.get_mut(table.table_name()) {
                                Some(patches) => match patches.get_mut(field.name()) {
                                    Some(patches) => patches.extend(patch),
                                    None => { patches.insert(field.name().to_owned(), patch); }
                                },
                                None => {
                                    let mut table_patch = HashMap::new();
                                    table_patch.insert(field.name().to_owned(), patch);
                                    new_patches.insert(table.table_name().to_string(), table_patch);
                                }
                            }
                        }

                        // Only make patches for fields we manage to pinpoint to a file.
                        if !possible_relative_paths.is_empty() {
                            let mut possible_relative_paths = possible_relative_paths.iter().collect::<Vec<_>>();
                            possible_relative_paths.sort();

                            let mut patch = HashMap::new();
                            if !field.is_filename(None) {
                                patch.insert("is_filename".to_owned(), "true".to_owned());
                            }

                            // Only add paths if we're not dealing with single paths with full replacement, or we're force-replacing a path (advisors table).
                            if possible_relative_paths.len() > 1 || (
                                (
                                    possible_relative_paths.len() == 1 &&
                                    possible_relative_paths[0].contains('%') &&
                                    possible_relative_paths[0] != "%"
                                ) || (
                                    possible_relative_paths[0] == "%" &&
                                    field.filename_relative_path(None).is_some() &&
                                    !field.filename_relative_path(None).unwrap().is_empty()
                                )
                            ) {
                                patch.insert("filename_relative_path".to_owned(), possible_relative_paths.into_iter().join(";"));
                            }

                            // Do not bother with empty patches.
                            if !patch.is_empty() {
                                match new_patches.get_mut(table.table_name()) {
                                    Some(patches) => match patches.get_mut(field.name()) {
                                        Some(patches) => patches.extend(patch),
                                        None => { patches.insert(field.name().to_owned(), patch); }
                                    },
                                    None => {
                                        let mut table_patch = HashMap::new();
                                        table_patch.insert(field.name().to_owned(), patch);
                                        new_patches.insert(table.table_name().to_string(), table_patch);
                                    }
                                }
                            }
                        }
                        /*
                        if (low_name == "key" || low_name == "id") && table.data().par_iter().all(|x| x[column].data_to_string().parse::<i32>().is_ok()) {
                            let mut patch = HashMap::new();
                            patch.insert("is_numeric".to_owned(), "true".to_owned());

                            match new_patches.get_mut(table.table_name()) {
                                Some(patches) => match patches.get_mut(field.name()) {
                                    Some(patches) => patches.extend(patch),
                                    None => { patches.insert(field.name().to_owned(), patch); }
                                },
                                None => {
                                    let mut table_patch = HashMap::new();
                                    table_patch.insert(field.name().to_owned(), patch);
                                    new_patches.insert(table.table_name().to_string(), table_patch);
                                }
                            }
                        }*/
                    }
                    FieldType::I64 |
                    FieldType::OptionalI64 => {
                        /*let low_name = field.name().to_lowercase();
                        if (low_name == "key" || low_name == "id") && table.data().par_iter().all(|x| x[column].data_to_string().parse::<i32>().is_ok()) {
                            let mut patch = HashMap::new();
                            patch.insert("is_numeric".to_owned(), "true".to_owned());

                            match new_patches.get_mut(table.table_name()) {
                                Some(patches) => match patches.get_mut(field.name()) {
                                    Some(patches) => patches.extend(patch),
                                    None => { patches.insert(field.name().to_owned(), patch); }
                                },
                                None => {
                                    let mut table_patch = HashMap::new();
                                    table_patch.insert(field.name().to_owned(), patch);
                                    new_patches.insert(table.table_name().to_string(), table_patch);
                                }
                            }
                        }*/
                    }
                    _ => continue
                }
            }
        }

        Schema::add_patch_to_patch_set(current_patches, &new_patches);

        Ok(())
    }

    /// Function to add tiles and tile maps to the provided pack.
    ///
    /// Only for Warhammer 3.
    pub fn add_tile_maps_and_tiles(&mut self, pack: &mut Pack, game: &GameInfo, schema: &Schema, options: OptimizerOptions, tile_maps: Vec<PathBuf>, tiles: Vec<(PathBuf, String)>) -> Result<(Vec<ContainerPath>, Vec<ContainerPath>)> {
        let mut added_paths = vec![];

        // Tile Maps are from assembly_kit/working_data/terrain/battles/.
        for tile_map in &tile_maps {
            added_paths.append(&mut pack.insert_folder(tile_map, "terrain/battles", &None, &None, true)?);
        }

        // Tiles are from assembly_kit/working_data/terrain/tiles/battle/, and can be in a subfolder if they're part of a tileset.
        for (tile, subpath) in &tiles {

            let (internal_path, needs_tile_database) = if subpath.is_empty() {
                ("terrain/tiles/battle".to_owned(), false)
            } else {
                (format!("terrain/tiles/battle/{}", subpath.replace('\\', "/")), true)
            };
            added_paths.append(&mut pack.insert_folder(tile, &internal_path, &None, &None, true)?);

            // If it's part of a tile set, we need to add the relevant tile database file for the tileset or the map will load as blank ingame.
            if needs_tile_database {

                // We only need the database for out map, not the full database folder.
                let subpath_len = subpath.replace('\\', "/").split('/').count();
                let mut tile_database = tile.to_path_buf();

                (0..=subpath_len).for_each(|_| {
                    tile_database.pop();
                });

                let file_name = format!("{}_{}.bin", subpath.replace('/', "_"), tile.file_name().unwrap().to_string_lossy());
                tile_database.push(format!("_tile_database/TILES/{file_name}"));
                let tile_database_path = format!("terrain/tiles/battle/_tile_database/TILES/{file_name}");

                added_paths.push(pack.insert_file(&tile_database, &tile_database_path, &None)?.unwrap());
            }
        }

        let (paths_to_delete, paths_to_add) = pack.optimize(Some(added_paths.clone()), self, schema, &game, &options)?;

        let paths_to_delete = paths_to_delete.iter()
            .map(|path| ContainerPath::File(path.to_string()))
            .collect::<Vec<_>>();

        added_paths.extend(paths_to_add.into_iter()
            .map(|path| ContainerPath::File(path.to_string()))
            .collect::<Vec<_>>());

        Ok((added_paths, paths_to_delete))
    }

    // Function to trigger an startpos build.
    //
    // After this ends, remember to call the post one!
    pub fn build_starpos_pre(&self, pack_file: &mut Pack, game: &GameInfo, game_path: &Path, campaign_id: &str, process_hlp_spd_data: bool, sub_start_pos: &str) -> Result<()> {
        let pack_name = pack_file.disk_file_name();
        if pack_name.is_empty() {
            return Err(RLibError::BuildStartposError("The Pack needs to be saved to disk in order to build a startpos. Save it and try again.".to_owned()));
        }

        if campaign_id.is_empty() {
            return Err(RLibError::BuildStartposError("campaign_id not provided.".to_owned()));
        }

        let process_hlp_spd_data_string = if process_hlp_spd_data {
            String::from("process_campaign_ai_map_data;")
        } else {
            String::new()
        };

        // Note: 3K uses 2 passes per campaign, each one with a different startpos, but both share the hlp/spd process, so that only needs to be generated once.
        // Also, extra folders is to fix a bug in Rome 2, Attila and possibly Thrones where objectives are not processed if certain folders are missing.
        let extra_folders = "add_working_directory assembly_kit\\working_data;";
        let mut user_script_contents = if game.key() == KEY_ATTILA || game.key() == KEY_THRONES_OF_BRITANNIA { extra_folders.to_owned() } else { String::new() };

        user_script_contents.push_str(&format!("
    mod {pack_name};
    process_campaign_startpos {campaign_id} {sub_start_pos};
    {process_hlp_spd_data_string}
    quit_after_campaign_processing;"
        ));

        // Games may fail to launch if we don't have this path created, which is done the first time we start the game.
        let game_data_path = game.data_path(&game_path)?;
        if !game_path.is_dir() {
            return Err(RLibError::BuildStartposError("Game path incorrect. Fix it in the settings and try again.".to_owned()));
        }

        if !PathBuf::from(pack_file.disk_file_path()).starts_with(&game_data_path) {
            return Err(RLibError::BuildStartposError("The Pack needs to be in /data. Install it there and try again.".to_owned()));
        }

        // We need to extract the victory_objectives.txt file to "data/campaign_id/". Warhammer 3 doesn't use this file.
        if GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) {
            let mut game_campaign_path = game_data_path.to_path_buf();
            game_campaign_path.push(campaign_id);
            DirBuilder::new().recursive(true).create(&game_campaign_path)?;

            game_campaign_path.push(VICTORY_OBJECTIVES_EXTRACTED_FILE_NAME);
            pack_file.extract(ContainerPath::File(VICTORY_OBJECTIVES_FILE_NAME.to_owned()), &game_campaign_path, false, &None, true, false, &None, true)?;
        }

        let config_path = game.config_path(&game_path).ok_or(RLibError::BuildStartposError("Error getting the game's config path.".to_owned()))?;
        let scripts_path = config_path.join("scripts");
        DirBuilder::new().recursive(true).create(&scripts_path)?;

        // Rome 2 is bugged when generating startpos using the userscript. We need to pass it to the game through args in a cmd terminal instead of by file.
        //
        // So don't do any userscript change for Rome 2.
        if game.key() != KEY_ROME_2 {

            // Make a backup before editing the script, so we can restore it later.
            let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
            let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");

            if uspa.is_file() {
                std::fs::copy(&uspa, uspb)?;
            }

            let mut file = BufWriter::new(File::create(uspa)?);

            // Napoleon, Empire and Shogun 2 require the user.script.txt or mod list file (for Shogun's latest update) to be in UTF-16 LE. What the actual fuck.
            if *game.raw_db_version() < 2 {
                file.write_string_u16(&user_script_contents)?;
            } else {
                file.write_all(user_script_contents.as_bytes())?;
            }

            file.flush()?;
        }

        // Due to how the starpos is generated, if we generate it on vanilla campaigns it'll overwrite existing files if it's generated on /data.
        // So we must backup the vanilla files, then restore them after.
        //
        // Only needed from Warhammer 1 onwards, and in Rome 2 due to how is generated there.
        if game.key() != KEY_THRONES_OF_BRITANNIA &&
            game.key() != KEY_ATTILA &&
            game.key() != KEY_SHOGUN_2 {

            let sub_start_pos_suffix = if sub_start_pos.is_empty() {
                String::new()
            } else {
                format!("_{sub_start_pos}")
            };

            let starpos_path = game_data_path.join(format!("campaigns/{campaign_id}/startpos{sub_start_pos_suffix}.esf"));
            if starpos_path.is_file() {
                let starpos_path_bak = game_data_path.join(format!("campaigns/{campaign_id}/startpos{sub_start_pos_suffix}.esf.bak"));
                std::fs::copy(&starpos_path, starpos_path_bak)?;
                std::fs::remove_file(starpos_path)?;
            }
        }

        // Same for the other two files, if we're generating them. We need to get the campaign name from the campaigns table first, then get the files generated.
        if process_hlp_spd_data {
            let map_names = self.db_values_from_table_name_and_column_name_for_value(Some(pack_file), "campaigns_tables", "campaign_name", "map_name", true, true);
            if let Some(map_name) = map_names.get(campaign_id) {
                match game.key() {

                    // For generating the hlp data, from Warhammer 1 onwards the game outputs it to /data, which may not exists and may conflict with existing files.
                    //
                    // Create the folder just in case, and back any file found.
                    KEY_PHARAOH_DYNASTIES |
                    KEY_PHARAOH |
                    KEY_WARHAMMER_3 |
                    KEY_TROY |
                    KEY_THREE_KINGDOMS |
                    KEY_WARHAMMER_2 |
                    KEY_WARHAMMER => {
                        let hlp_folder_path = game_data_path.join(format!("campaign_maps/{map_name}"));
                        if !hlp_folder_path.is_dir() {
                            DirBuilder::new().recursive(true).create(&hlp_folder_path)?;
                        }

                        let hlp_path = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf"));
                        if hlp_path.is_file() {
                            let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));
                            std::fs::copy(&hlp_path, hlp_path_bak)?;
                            std::fs::remove_file(hlp_path)?;
                        }
                    },

                    // For Thrones and Attila is more tricky, because the game itself is bugged when processing this file.
                    //
                    // It's generated in the game's config folder, but we need to manually keep recreating the folder for a while because the game deletes it
                    // in the middle of the process and causes an error when trying to write the file. The way we do it is with a background thread
                    // that keeps recreating it every 100ms if it ever detects it's gone.
                    //
                    // Keep in mind this thread is kept alive for as long as the program runs unless it's intentionally stopped. So remember to stop it.
                    KEY_THRONES_OF_BRITANNIA |
                    KEY_ATTILA => {
                        let folder_path = config_path.join(format!("maps/campaign_maps/{map_name}"));

                        let (sender, receiver) = channel::<bool>();
                        let join = thread::spawn(move || {
                            loop {
                                match receiver.try_recv() {
                                    Ok(stop) => if stop {
                                        break;
                                    }
                                    Err(_) => {
                                        if !folder_path.is_dir() {
                                            let _ = DirBuilder::new().recursive(true).create(&folder_path);
                                        }

                                        thread::sleep(Duration::from_millis(100));
                                    }
                                }
                            }
                        });

                         *START_POS_WORKAROUND_THREAD.write().unwrap() = Some(vec![(sender, join)]);
                    },

                    // For rome 2 is a weird one. It generates the file in config (like Attila), but them moves it to /data (like Warhammer).
                    //
                    // So we need to first, ensure the config folder is created (it may not exists, but it's not deleted mid-process like in Attile)
                    // and it's empty, and then backup the hlp file, if exists, from /data.
                    KEY_ROME_2 => {
                        let hlp_folder = game_data_path.join(format!("campaign_maps/{map_name}/"));
                        if hlp_folder.is_dir() {
                            let _ = DirBuilder::new().recursive(true).create(&hlp_folder);
                        }

                        let hlp_path = hlp_folder.join("hlp_data.esf");
                        if hlp_path.is_file() {
                            let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));
                            std::fs::copy(&hlp_path, hlp_path_bak)?;
                            std::fs::remove_file(hlp_path)?;
                        }

                    }
                    KEY_SHOGUN_2 => return Err(RLibError::BuildStartposError("Unsupported... yet. If you want to test support for this game, let me know.".to_owned())),
                    KEY_NAPOLEON => return Err(RLibError::BuildStartposError("Unsupported... yet. If you want to test support for this game, let me know.".to_owned())),
                    KEY_EMPIRE => return Err(RLibError::BuildStartposError("Unsupported... yet. If you want to test support for this game, let me know.".to_owned())),
                    _ => return Err(RLibError::BuildStartposError("How the fuck did you trigger this?".to_owned())),
                }

                // This file is only from Warhammer 1 onwards. No need to check if the path exists because the hlp process should have created the folder.
                if game.key() != KEY_THRONES_OF_BRITANNIA &&
                    game.key() != KEY_ATTILA &&
                    game.key() != KEY_ROME_2 &&
                    game.key() != KEY_SHOGUN_2 &&
                    game.key() != KEY_NAPOLEON &&
                    game.key() != KEY_EMPIRE {

                    let spd_path = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf"));
                    if spd_path.is_file() {
                        let spd_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf.bak"));
                        std::fs::copy(&spd_path, spd_path_bak)?;
                        std::fs::remove_file(spd_path)?;
                    }
                }
            }
        }

        // Then launch the game. 3K needs to be launched manually and in a blocking manner to make sure it does each pass it has to do correctly.
        if game.key() == KEY_THREE_KINGDOMS {
            let exe_path = game.executable_path(&game_path).ok_or_else(|| RLibError::BuildStartposError("Game exe path not found.".to_owned()))?;
            let exe_name = exe_path.file_name().ok_or_else(|| RLibError::BuildStartposError("Game exe name not found.".to_owned()))?.to_string_lossy();

            // NOTE: This uses a non-existant load order file on purpouse, so no mod in the load order interferes with generating the startpos.
            let mut command = Command::new("cmd");
            command.arg("/C");
            command.arg("start");
            command.arg("/wait");
            command.arg("/d");
            command.arg(game_path.to_string_lossy().replace('\\', "/"));
            command.arg(exe_name.to_string());
            command.arg("temp_file.txt;");

            let _ = command.output()?;

            // In multipass, we need to clean the user script after each pass.
            let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
            let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");
            if uspb.is_file() {
                std::fs::copy(uspb, uspa)?;
            }

            // If there's no backup, means there was no file to begin with, so we delete the custom file.
            else if uspa.is_file() {
                std::fs::remove_file(uspa)?;
            }

        // Rome 2 needs to be launched manually through the cmd with params. The rest can be launched through their regular launcher.
        } else if game.key() == KEY_ROME_2 {
            let exe_path = game.executable_path(&game_path).ok_or_else(|| RLibError::BuildStartposError("Game exe path not found.".to_owned()))?;
            let exe_name = exe_path.file_name().ok_or_else(|| RLibError::BuildStartposError("Game exe name not found.".to_owned()))?.to_string_lossy();

            // NOTE: This uses a non-existant load order file on purpouse, so no mod in the load order interferes with generating the startpos.
            let mut command = Command::new("cmd");
            command.arg("/C");
            command.arg("start");
            command.arg("/d");
            command.arg(game_path.to_string_lossy().replace('\\', "/"));
            command.arg(exe_name.to_string());
            command.arg("temp_file.txt;");

            // We need to turn the user script contents into a oneliner or the command will ignore it.
            #[cfg(target_os = "windows")] {
                use std::os::windows::process::CommandExt;

                // Rome 2 needs the working_data folder in order to throw the startpos file there.
                command.raw_arg(extra_folders);
                command.raw_arg(user_script_contents.replace("\n", " "));
            }

            command.spawn()?;
        } else {
            match game.game_launch_command(&game_path) {
                Ok(command) => { let _ = open::that(command); },
                _ => return Err(RLibError::BuildStartposError("The currently selected game cannot be launched from Steam.".to_owned())),
            }
        }

        Ok(())
    }

    /// Function to trigger the second part of the startpos build process, which involves importing the startpos file
    /// into the provided pack.
    ///
    /// Call this when the game closes after the pre function launched it.
    ///
    /// NOTE: The assembly kit path is only needed for Rome 2.
    pub fn build_starpos_post(&self, pack_file: &mut Pack, game: &GameInfo, game_path: &Path, asskit_path: Option<PathBuf>,campaign_id: &str, process_hlp_spd_data: bool, cleanup_mode: bool, sub_start_pos: &[String]) -> Result<Vec<ContainerPath>> {

        let mut startpos_failed = false;
        let mut sub_startpos_failed = vec![];
        let mut hlp_failed = false;
        let mut spd_failed = false;

        // Before anything else, close the workaround thread.
        if let Some(data) = START_POS_WORKAROUND_THREAD.write().unwrap().as_mut() {
            let (sender, handle) = data.remove(0);
            let _ = sender.send(true);
            let _ = handle.join();
        }

        *START_POS_WORKAROUND_THREAD.write().unwrap() = None;

        if !game_path.is_dir() {
            return Err(RLibError::BuildStartposError("Game path incorrect. Fix it in the settings and try again.".to_owned()));
        }

        let game_data_path = game.data_path(&game_path)?;

        // Warhammer 3 doesn't use this folder.
        if GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&game.key()) {

            // We need to delete the "data/campaign_id/" folder.
            let mut game_campaign_path = game_data_path.to_path_buf();
            game_campaign_path.push(campaign_id);
            if game_campaign_path.is_dir() {
                let _ = std::fs::remove_dir_all(game_campaign_path);
            }
        }

        let config_path = game.config_path(&game_path).ok_or(RLibError::BuildStartposError("Error getting the game's config path.".to_owned()))?;
        let scripts_path = config_path.join("scripts");
        if !scripts_path.is_dir() {
            DirBuilder::new().recursive(true).create(&scripts_path)?;
        }

        // Restore the userscript backup, if any.
        let uspa = scripts_path.join(USER_SCRIPT_FILE_NAME);
        let uspb = scripts_path.join(USER_SCRIPT_FILE_NAME.to_owned() + ".bak");
        if uspb.is_file() {
            std::fs::copy(uspb, uspa)?;
        }

        // If there's no backup, means there was no file to begin with, so we delete the custom file.
        else if uspa.is_file() {
            std::fs::remove_file(uspa)?;
        }

        let mut added_paths = vec![];

        // Add the starpos file. As some games have multiple startpos per campaign (3K) we return a vector with all the paths we have to generate.
        let starpos_paths = match game.key() {
            KEY_PHARAOH_DYNASTIES |
            KEY_PHARAOH |
            KEY_WARHAMMER_3 |
            KEY_TROY |
            KEY_THREE_KINGDOMS |
            KEY_WARHAMMER_2 |
            KEY_WARHAMMER => {
                if sub_start_pos.is_empty() {
                    vec![game_data_path.join(format!("campaigns/{campaign_id}/startpos.esf"))]
                } else {
                    let mut paths = vec![];
                    for sub in sub_start_pos {
                        paths.push(game_data_path.join(format!("campaigns/{campaign_id}/startpos_{sub}.esf")));

                    }
                    paths
                }
            }
            KEY_THRONES_OF_BRITANNIA |
            KEY_ATTILA => vec![config_path.join(format!("maps/campaigns/{campaign_id}/startpos.esf"))],

            // Rome 2 outputs the startpos in the assembly kit folder.
            KEY_ROME_2 => {
                match asskit_path {
                    Some(asskit_path) => {
                        if !asskit_path.is_dir() {
                            return Err(RLibError::BuildStartposError("Assembly Kit path is not a valid folder.".to_owned()));
                        }

                        vec![asskit_path.join(format!("working_data/campaigns/{campaign_id}/startpos.esf"))]
                    },
                    None => return Err(RLibError::BuildStartposError("Assembly Kit path not provided.".to_owned())),
                }
            },

            // Shogun 2 outputs to data, but unlike modern names, vanilla startpos are packed, so there's no rist of overwrite.
            // We still need to clean it up later though. Napoleon and Empire override vanilla files, so those are backed.
            KEY_SHOGUN_2 |
            KEY_NAPOLEON |
            KEY_EMPIRE => vec![game_data_path.join(format!("campaigns/{campaign_id}/startpos.esf"))],
            _ => return Err(RLibError::BuildStartposError("How the fuck did you trigger this?".to_owned())),
        };

        let starpos_paths_pack = if sub_start_pos.is_empty() {
            vec![format!("campaigns/{}/startpos.esf", campaign_id)]
        } else {
            let mut paths = vec![];
            for sub in sub_start_pos {
                paths.push(format!("campaigns/{campaign_id}/startpos_{sub}.esf"));
            }
            paths
        };

        if !cleanup_mode {
            for (index, starpos_path) in starpos_paths.iter().enumerate() {
                if !starpos_path.is_file() {
                    if sub_start_pos.is_empty() {
                        startpos_failed = true;
                    } else {
                        sub_startpos_failed.push(sub_start_pos[index].to_owned());
                    }
                } else {

                    let mut rfile = RFile::new_from_file_path(starpos_path)?;
                    rfile.set_path_in_container_raw(&starpos_paths_pack[index]);
                    rfile.load()?;
                    rfile.guess_file_type()?;

                    added_paths.push(pack_file.insert(rfile).map(|x| x.unwrap())?);
                }
            }
        }

        // Restore the old starpos if there was one, and delete the new one if it has already been added.
        //
        // Only needed from Warhammer 1 onwards, and for Rome 2, Napoleon and Empire. Other games generate the startpos outside that folder.
        //
        // 3K uses 2 startpos, so we need to restore them both.
        if game.key() != KEY_THRONES_OF_BRITANNIA &&
            game.key() != KEY_ATTILA &&
            game.key() != KEY_SHOGUN_2 {

            for starpos_path in &starpos_paths {
                let file_name = starpos_path.file_name().unwrap().to_string_lossy().to_string();
                let file_name_bak = file_name + ".bak";

                let mut starpos_path_bak = starpos_path.to_path_buf();
                starpos_path_bak.set_file_name(file_name_bak);

                if starpos_path_bak.is_file() {
                    std::fs::copy(&starpos_path_bak, starpos_path)?;
                    std::fs::remove_file(starpos_path_bak)?;
                }
            }
        }

        // In Shogun 2, we need to cleanup the generated file as to not interfere with the packed one.
        if game.key() == KEY_SHOGUN_2 {
            for starpos_path in &starpos_paths {
                if starpos_path.is_file() {
                    std::fs::remove_file(starpos_path)?;
                }
            }
        }

        // Same with the other two files.
        if process_hlp_spd_data {
            let map_names = self.db_values_from_table_name_and_column_name_for_value(Some(pack_file), "campaigns_tables", "campaign_name", "map_name", true, true);
            if let Some(map_name) = map_names.get(campaign_id) {

                // Same as with startpos. It's different depending on the game.
                let hlp_path = match game.key() {
                    KEY_PHARAOH_DYNASTIES |
                    KEY_PHARAOH |
                    KEY_WARHAMMER_3 |
                    KEY_TROY |
                    KEY_THREE_KINGDOMS |
                    KEY_WARHAMMER_2 |
                    KEY_WARHAMMER => game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf")),
                    KEY_THRONES_OF_BRITANNIA |
                    KEY_ATTILA => config_path.join(format!("maps/campaign_maps/{map_name}/hlp_data.esf")),
                    KEY_ROME_2 => game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf")),
                    _ => return Err(RLibError::BuildStartposError("How the fuck did you trigger this?".to_owned())),
                };

                let hlp_path_pack = format!("campaign_maps/{map_name}/hlp_data.esf");

                if !cleanup_mode {

                    if !hlp_path.is_file() {
                        hlp_failed = true;
                    } else {

                        let mut rfile_hlp = RFile::new_from_file_path(&hlp_path)?;
                        rfile_hlp.set_path_in_container_raw(&hlp_path_pack);
                        rfile_hlp.load()?;
                        rfile_hlp.guess_file_type()?;

                        added_paths.push(pack_file.insert(rfile_hlp).map(|x| x.unwrap())?);
                    }
                }

                // Only needed from Warhammer 1 onwards, and in Rome 2. Other games generate the hlp file outside that folder.
                if game.key() != KEY_THRONES_OF_BRITANNIA &&
                    game.key() != KEY_ATTILA {

                    let hlp_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/hlp_data.esf.bak"));

                    if hlp_path_bak.is_file() {
                        std::fs::copy(&hlp_path_bak, hlp_path)?;
                        std::fs::remove_file(hlp_path_bak)?;
                    }
                }

                // The spd file was introduced in Warhammer 1. Don't expect it on older games.
                if game.key() != KEY_THRONES_OF_BRITANNIA &&
                    game.key() != KEY_ATTILA &&
                    game.key() != KEY_ROME_2 {

                    let spd_path = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf"));
                    let spd_path_pack = format!("campaign_maps/{map_name}/spd_data.esf");

                    if !cleanup_mode {

                        if !spd_path.is_file() {
                            spd_failed = true;
                        } else {

                            let mut rfile_spd = RFile::new_from_file_path(&spd_path)?;
                            rfile_spd.set_path_in_container_raw(&spd_path_pack);
                            rfile_spd.load()?;
                            rfile_spd.guess_file_type()?;

                            added_paths.push(pack_file.insert(rfile_spd).map(|x| x.unwrap())?);
                        }
                    }

                    let spd_path_bak = game_data_path.join(format!("campaign_maps/{map_name}/spd_data.esf.bak"));
                    if spd_path_bak.is_file() {
                        std::fs::copy(&spd_path_bak, spd_path)?;
                        std::fs::remove_file(spd_path_bak)?;
                    }
                }
            }
        }

        let mut error = String::new();
        if startpos_failed || (!sub_start_pos.is_empty() && !sub_startpos_failed.is_empty()) || hlp_failed || spd_failed {
            error.push_str("<p>One or more files failed to generate:</p><ul>")
        }
        if startpos_failed {
            error.push_str("<li>Startpos file failed to generate.</li>");
        }

        for sub_failed in &sub_startpos_failed {
            error.push_str(&format!("<li>\"{sub_failed}\" Startpos file failed to generate.</li>"));
        }

        if hlp_failed {
            error.push_str("<li>HLP file failed to generate.</li>");
        }

        if spd_failed {
            error.push_str("<li>SPD file failed to generate.</li>");
        }

        if startpos_failed || hlp_failed || spd_failed {
            error.push_str("</ul><p>No files were added and the related files were restored to their pre-build state. Check your tables are correct before trying to generate them again.</p>")
        }

        if error.is_empty() {
            Ok(added_paths)
        } else {
            Err(RLibError::BuildStartposError(error))
        }
    }

    /// This function imports a specific table from the data it has in the AK.
    ///
    /// Tables generated with this are VALID.
    pub fn import_from_ak(&self, table_name: &str, schema: &Schema) -> Result<DB> {
        let definition = if let Some(definitions) = schema.definitions_by_table_name_cloned(table_name) {
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
            Err(RLibError::AssemblyKitTableNotFound(table_name.to_owned()))
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

    /// This function manipulates a definition to recursively add reference lookups if found.
    ///
    /// THIS IS DANGEROUS IF WE FIND A CYCLIC DEPENDENCY.
    pub fn add_recursive_lookups_to_definition(&self, schema: &Schema, definition: &mut Definition, table_name: &str) {
        let schema_patches = definition.patches().clone();

        for field in definition.fields_mut().iter_mut() {

            // First check lookups on the local table.
            if let Some(lookup_data_old) = field.lookup(Some(&schema_patches)) {
                let mut lookup_data = vec![];

                // Check first for local lookups.
                if !lookup_data_old.is_empty() {

                    let table_name = if let Some(table_name) = table_name.strip_suffix("_tables") {
                        table_name.to_owned()
                    } else {
                        table_name.to_owned()
                    };

                    for lookup_data_old in &lookup_data_old {
                        let lookup_string = format!("{}#{}#{}", table_name, field.name(), lookup_data_old);
                        self.add_recursive_lookups(schema, &schema_patches, lookup_data_old, &mut lookup_data, &lookup_string, &table_name);
                    }

                }

                // If our field is a reference, do recursive checks to find out all the lookup data of a specific field.
                if let Some((ref_table_name, ref_column)) = field.is_reference(Some(&schema_patches)) {
                    for lookup_data_old in &lookup_data_old {
                        let lookup_string = format!("{ref_table_name}#{ref_column}#{lookup_data_old}");
                        self.add_recursive_lookups(schema, &schema_patches, lookup_data_old, &mut lookup_data, &lookup_string, &ref_table_name);
                    }
                }

                if !lookup_data.is_empty() {
                    field.set_lookup(Some(lookup_data));
                } else {
                    field.set_lookup(None);
                }
            }
        }
    }

    fn add_recursive_lookups(&self,
        schema: &Schema,
        schema_patches: &HashMap<String, HashMap<String, String>>,
        lookup: &str,
        lookup_data: &mut Vec<String>,
        lookup_string: &str,
        table_name: &str
    ) {
        let mut finish_lookup = false;
        let table_name = table_name.to_string() + "_tables";
        if let Ok(ref_tables) = self.db_data(&table_name, true, true) {
            let candidates = ref_tables.iter()
                .filter_map(|rfile| rfile.decoded().ok())
                .filter_map(|decoded| if let RFileDecoded::DB(db) = decoded {
                    Some(db.definition().clone())
                } else {
                    None
                })
                .collect::<Vec<_>>();

            if let Some(definition) = schema.definition_newer(&table_name, &candidates) {

                // If this fails, it may be a loc.
                if let Some(pos) = definition.column_position_by_name(lookup) {
                    if let Some(field) = definition.fields_processed().get(pos) {

                        // If our field is a reference, we need to go one level deeper to find the lookup.
                        if let Some((ref_table_name, ref_column)) = field.is_reference(Some(schema_patches)) {
                            if let Some(lookups) = field.lookup(Some(schema_patches)) {
                                for lookup in &lookups {
                                    let lookup_string = format!("{lookup_string}:{ref_table_name}#{ref_column}#{lookup}");

                                    self.add_recursive_lookups(schema, schema_patches, lookup, lookup_data, &lookup_string, &ref_table_name);
                                }
                            } else {
                                finish_lookup = true;
                            }
                        } else {
                            finish_lookup = true;
                        }
                    } else {
                        finish_lookup = true;
                    }
                }

                else if definition.localised_fields().iter().any(|x| x.name() == lookup) {
                    finish_lookup = true;
                }
            } else {
                finish_lookup = true;
            }
        }

        if finish_lookup && !lookup_data.iter().any(|x| x == lookup_string) {
            lookup_data.push(lookup_string.to_owned());
        }
    }
}
