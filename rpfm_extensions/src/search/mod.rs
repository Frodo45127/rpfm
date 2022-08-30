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
Module with all the code related to the `GlobalSearch`.

This module contains the code needed to get a `GlobalSearch` over an entire `PackFile`.
!*/

use regex::{RegexBuilder, Regex};
use rayon::prelude::*;

use rpfm_lib::files::{Container, ContainerPath};
use rpfm_lib::files::{FileType, pack::Pack, RFileDecoded};
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::Schema;

use crate::dependencies::Dependencies;

use self::schema::SchemaMatches;
use self::table::TableMatches;
use self::text::TextMatches;

pub mod schema;
pub mod table;
pub mod text;

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Optimizable`, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Searchable {
    type SearchMatches;

    /// This function optimizes the provided struct to reduce its size and improve compatibility.
    ///
    /// It returns if the struct has been left in an state where it can be safetly deleted.
    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> Self::SearchMatches;
}

/// This trait marks a [Container](rpfm_lib::files::Container) as an `Optimizable` container, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Replaceable: Searchable {

    /// This function optimizes the provided [Container](rpfm_lib::files::Container) to reduce its size and improve compatibility.
    ///
    /// It returns the list of files that has been safetly deleted during the optimization process.
    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &Self::SearchMatches) -> bool;
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the information needed to perform a global search, and the results of said search.
#[derive(Debug, Clone)]
pub struct GlobalSearch {

    /// Pattern to search.
    pub pattern: String,

    /// Pattern to use when replacing. This is a hard pattern, which means regex is not allowed here.
    pub replace_text: String,

    /// Should the global search be *Case Sensitive*?
    pub case_sensitive: bool,

    /// If the search must be done using regex instead basic matching.
    pub use_regex: bool,

    /// Where should we search.
    pub source: SearchSource,

    /// If we should search on DB Tables.
    pub search_on_dbs: bool,

    /// If we should search on Loc Tables.
    pub search_on_locs: bool,

    /// If we should search on Text PackedFiles.
    pub search_on_texts: bool,

    /// If we should search on the currently loaded Schema.
    pub search_on_schema: bool,

    /// Matches on DB Tables.
    pub matches_db: Vec<TableMatches>,

    /// Matches on Loc Tables.
    pub matches_loc: Vec<TableMatches>,

    /// Matches on Text Tables.
    pub matches_text: Vec<TextMatches>,

    /// Matches on Schema definitions.
    pub matches_schema: SchemaMatches,
}

/// This enum defines the matching mode of the search. We use `Pattern` by default, and fall back to it
/// if we try to use `Regex` and the provided regex expression is invalid.
#[derive(Debug, Clone)]
pub enum MatchingMode {
    Regex(Regex),
    Pattern,
}

/// This enum is a way to put together all kind of matches.
#[derive(Debug, Clone)]
pub enum MatchHolder {
    Table(TableMatches),
    Text(TextMatches),
    Schema(SchemaMatches),
}

/// This enum is specifies the source where the search should be performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchSource {
    Pack,
    ParentFiles,
    GameFiles,
    AssKitFiles,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `GlobalSearch`.
impl Default for GlobalSearch {
    fn default() -> Self {
        Self {
            pattern: "".to_owned(),
            replace_text: "".to_owned(),
            case_sensitive: false,
            use_regex: false,
            source: SearchSource::Pack,
            search_on_dbs: true,
            search_on_locs: true,
            search_on_texts: true,
            search_on_schema: false,
            matches_db: vec![],
            matches_loc: vec![],
            matches_text: vec![],
            matches_schema: SchemaMatches::new(),
        }
    }
}

/// Implementation of `GlobalSearch`.
impl GlobalSearch {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn search(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies, update_paths: &[ContainerPath]) {

        // Don't do anything if we have no pattern to search.
        if self.pattern.is_empty() { return }

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // If we're updating, make sure to dedup and get the raw paths of each file to update.
        let update_paths = if !update_paths.is_empty() && self.source == SearchSource::Pack {
            let container_paths = ContainerPath::dedup(&update_paths);
            let raw_paths = container_paths.par_iter()
                .map(|container_path| pack.paths_raw_from_container_path(container_path))
                .flatten()
                .collect::<Vec<_>>();

            for path in raw_paths {
                self.matches_db.retain(|x| x.path() != &path);
                self.matches_loc.retain(|x| x.path() != &path);
                self.matches_text.retain(|x| x.path() != &path);
            }

            container_paths
        }

        // Otherwise, ensure we don't store results from previous searches.
        else {
            self.matches_db = vec![];
            self.matches_loc = vec![];
            self.matches_text = vec![];

            vec![]
        };

        // Schema matches do not support "update search".
        self.matches_schema = SchemaMatches::new();

        match self.source {
            SearchSource::Pack => {

                if self.search_on_dbs {
                    let files = if !update_paths.is_empty() {
                        pack.files_by_type_and_paths(&[FileType::DB], &update_paths)
                    } else {
                        pack.files_by_type(&[FileType::DB])
                    };

                    self.matches_db = files.par_iter()
                        .filter_map(|file| {
                            if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                                let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                if !result.matches().is_empty() {
                                    Some(result)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    ).collect();
                }

                if self.search_on_locs {
                    let files = if !update_paths.is_empty() {
                        pack.files_by_type_and_paths(&[FileType::Loc], &update_paths)
                    } else {
                        pack.files_by_type(&[FileType::Loc])
                    };

                    self.matches_loc = files.par_iter()
                        .filter_map(|file| {
                            if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                                let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                if !result.matches().is_empty() {
                                    Some(result)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    ).collect();
                }

                if self.search_on_texts {
                    let mut files = if !update_paths.is_empty() {
                        pack.files_by_type_and_paths_mut(&[FileType::Loc], &update_paths)
                    } else {
                        pack.files_by_type_mut(&[FileType::Text])
                    };

                    self.matches_text = files.par_iter_mut()
                        .filter_map(|file| {
                            if let Ok(RFileDecoded::Text(table)) = file.decode(&None, false, true).transpose().unwrap() {
                                let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                if !result.matches().is_empty() {
                                    Some(result)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    ).collect();
                }
            }
            SearchSource::ParentFiles => {

                if self.search_on_dbs {
                    if let Ok(files) = dependencies.db_and_loc_data(true, false, false, true) {
                        self.matches_db = files.par_iter()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                                    let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                    if !result.matches().is_empty() {
                                        Some(result)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        ).collect();
                    }
                }

                if self.search_on_locs {
                    if let Ok(files) = dependencies.db_and_loc_data(false, true, false, true) {
                        self.matches_loc = files.par_iter()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                                    let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                    if !result.matches().is_empty() {
                                        Some(result)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        ).collect();
                    }
                }

                if self.search_on_texts {
                    let mut files = dependencies.files_by_types_mut(&[FileType::Text], false, true);
                    self.matches_text = files.par_iter_mut()
                        .filter_map(|(path, file)| {
                            if let Ok(RFileDecoded::Text(text)) = file.decode(&None, false, true).transpose().unwrap() {
                                let result = text.search(path, &self.pattern, self.case_sensitive, &matching_mode);
                                if !result.matches().is_empty() {
                                    Some(result)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    ).collect();
                }
            },
            SearchSource::GameFiles => {

                if self.search_on_dbs {
                    if let Ok(files) = dependencies.db_and_loc_data(true, false, true, false) {
                        self.matches_db = files.par_iter()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                                    let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                    if !result.matches().is_empty() {
                                        Some(result)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        ).collect();
                    }
                }

                if self.search_on_locs {
                    if let Ok(files) = dependencies.db_and_loc_data(false, true, true, false) {
                        self.matches_loc = files.par_iter()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                                    let result = table.search(file.path_in_container_raw(), &self.pattern, self.case_sensitive, &matching_mode);
                                    if !result.matches().is_empty() {
                                        Some(result)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        ).collect();
                    }
                }

                if self.search_on_texts {
                    let mut files = dependencies.files_by_types_mut(&[FileType::Text], true, false);
                    self.matches_text = files.par_iter_mut()
                        .filter_map(|(path, file)| {
                            if let Ok(RFileDecoded::Text(text)) = file.decode(&None, false, true).transpose().unwrap() {
                                let result = text.search(path, &self.pattern, self.case_sensitive, &matching_mode);
                                if !result.matches().is_empty() {
                                    Some(result)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    ).collect();
                }
            },

            // Asskit files are only tables.
            SearchSource::AssKitFiles => {
                if self.search_on_dbs {
                    self.matches_db = dependencies.asskit_only_db_tables()
                        .par_iter()
                        .filter_map(|(table_name, table)| {
                            let file_name = match game_info.vanilla_db_table_name_logic() {
                                VanillaDBTableNameLogic::FolderName => table_name.to_owned(),
                                VanillaDBTableNameLogic::DefaultName(ref default_name) => default_name.to_owned()
                            };

                            let path = format!("db/{}/{}", table_name, file_name);
                            let result = table.search(&path, &self.pattern, self.case_sensitive, &matching_mode);
                            if !result.matches().is_empty() {
                                Some(result)
                            } else {
                                None
                            }
                        }
                    ).collect();
                }
            },
        }

        // Schema searches are a bit independant from the rest, so they're done after the full search.
        if self.search_on_schema {
            self.matches_schema = schema.search("", &self.pattern, self.case_sensitive, &matching_mode);
        }
    }

    /// This function clears the Global Search result's data, and reset the UI for it.
    pub fn clear(&mut self) {
        *self = Self::default();
    }/*


    /// This function returns the PackedFileInfo for all the PackedFiles with the provided paths.
    pub fn get_update_paths_packed_file_info(&self, pack_file: &mut PackFile, paths: &[PathType]) -> Vec<PackedFileInfo> {
        let paths = paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect();
        let packed_files = pack_file.get_ref_packed_files_by_paths(paths);
        packed_files.iter().map(|x| From::from(*x)).collect()
    }*/

    /// This function performs a replace operation over the provided matches.
    ///
    /// NOTE: Schema matches are always ignored.
    pub fn replace(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies, matches: &[MatchHolder]) -> Vec<ContainerPath> {
        let mut edited_paths = vec![];

        // Don't do anything if we have no pattern to search.
        if self.pattern.is_empty() { return edited_paths }

        // This is only useful for Packs, not for dependencies.
        if self.source != SearchSource::Pack { return edited_paths }

        // If we want to use regex and the pattern is invalid, use normal pattern instead of Regex.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // Just replace all the provided matches, one by one.
        for match_file in matches {
            match match_file {
                MatchHolder::Table(search_matches) => {
                    let container_path = ContainerPath::Folder(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path);
                    if let Some(file) = file.get_mut(0) {
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::DB(table) => table.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                RFileDecoded::Loc(table) => table.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                }

                // TODO.
                MatchHolder::Text(_) => {

                }
                MatchHolder::Schema(_) => continue,
            }
        }

        // Update the current search over the edited files.
        self.search(game_info, schema, pack, dependencies, &edited_paths);

        // Return the changed paths.
        edited_paths
    }

    pub fn replace_all(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies) -> Vec<ContainerPath> {
        let mut matches = self.matches_db.iter().map(|x| MatchHolder::Table(x.clone())).collect::<Vec<_>>();
        matches.extend(self.matches_loc.iter().map(|x| MatchHolder::Table(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches_text.iter().map(|x| MatchHolder::Text(x.clone())).collect::<Vec<_>>());

        self.replace(game_info, schema, pack, dependencies, &matches)
    }
}
