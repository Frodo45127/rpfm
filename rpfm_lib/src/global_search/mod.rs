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
Module with all the code related to the `GlobalSearch`.

This module contains the code needed to get a `GlobalSearch` over an entire `PackFile`.
!*/

use regex::{RegexBuilder, Regex};
use rayon::prelude::*;

use rpfm_error::{ErrorKind, Result};

use crate::dependencies::Dependencies;
use crate::games::VanillaDBTableNameLogic;
use crate::GAME_SELECTED;
use crate::packfile::{PackFile, PathType};
use crate::packfile::packedfile::PackedFileInfo;
use crate::packedfile::{DecodedPackedFile, PackedFileType};
use crate::packedfile::table::{DecodedData, db::DB, loc::Loc};
use crate::packedfile::text::{Text, TextType};
use crate::schema::{Definition, Schema, VersionedFile};
use crate::SCHEMA;

use self::schema::{SchemaMatches, SchemaMatch};
use self::table::{TableMatches, TableMatch};
use self::text::{TextMatches, TextMatch};

pub mod schema;
pub mod table;
pub mod text;

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
    pub matches_schema: Vec<SchemaMatches>,
}

/// This enum defines the matching mode of the search. We use `Pattern` by default, and fall back to it
/// if we try to use `Regex` and the provided regex expression is invalid.
#[derive(Debug, Clone)]
enum MatchingMode {
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
    PackFile,
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
            source: SearchSource::PackFile,
            search_on_dbs: true,
            search_on_locs: true,
            search_on_texts: true,
            search_on_schema: false,
            matches_db: vec![],
            matches_loc: vec![],
            matches_text: vec![],
            matches_schema: vec![],
        }
    }
}

/// Implementation of `GlobalSearch`.
impl GlobalSearch {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn search(&mut self, pack_file: &mut PackFile, dependencies: &Dependencies) {

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // Ensure we don't store results from previous searches.
        self.matches_db = vec![];
        self.matches_loc = vec![];
        self.matches_text = vec![];
        self.matches_schema = vec![];

        // If we got no schema, don't even decode.
        if let Some(ref schema) = *SCHEMA.read().unwrap() {

            match self.source {
                SearchSource::PackFile => {
                    if self.search_on_dbs {
                        let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(PackedFileType::DB, false);
                        self.matches_db = packed_files.par_iter_mut().filter_map(|packed_file| {
                            let path = packed_file.get_path().to_vec();
                            if let Ok(DecodedPackedFile::DB(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                Some(self.search_on_db(&path, data, &matching_mode))
                            } else { None }
                        }).collect();
                    }

                    if self.search_on_locs {
                        let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(PackedFileType::Loc, false);
                        self.matches_loc = packed_files.par_iter_mut().filter_map(|packed_file| {
                            let path = packed_file.get_path().to_vec();
                            if let Ok(DecodedPackedFile::Loc(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                Some(self.search_on_loc(&path, data, &matching_mode))
                            } else { None }
                        }).collect();
                    }

                    if self.search_on_texts {
                        let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(PackedFileType::Text(TextType::Plain), false);
                        self.matches_text = packed_files.par_iter_mut().filter_map(|packed_file| {
                            let path = packed_file.get_path().to_vec();
                            if let Ok(DecodedPackedFile::Text(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                Some(self.search_on_text(&path, data, &matching_mode))
                            } else { None }
                        }).collect();
                    }
                }
                SearchSource::ParentFiles => {
                    if self.search_on_dbs {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_parent_files_by_types(&[PackedFileType::DB], false) {
                            self.matches_db = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::DB(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_db(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }

                    if self.search_on_locs {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_parent_files_by_types(&[PackedFileType::Loc], false) {
                            self.matches_loc = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::Loc(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_loc(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }

                    if self.search_on_texts {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_parent_files_by_types(&[PackedFileType::Text(TextType::Plain)], false) {
                            self.matches_text = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::Text(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_text(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }
                },
                SearchSource::GameFiles => {
                    if self.search_on_dbs {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_game_files_by_types(&[PackedFileType::DB], false) {
                            self.matches_db = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::DB(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_db(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }

                    if self.search_on_locs {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_game_files_by_types(&[PackedFileType::Loc], false) {
                            self.matches_loc = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::Loc(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_loc(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }

                    if self.search_on_texts {
                        if let Ok(mut packed_files) = dependencies.get_packedfiles_from_game_files_by_types(&[PackedFileType::Text(TextType::Plain)], false) {
                            self.matches_text = packed_files.par_iter_mut().filter_map(|packed_file| {
                                let path = packed_file.get_path().to_vec();
                                if let Ok(DecodedPackedFile::Text(data)) = packed_file.decode_return_ref_no_locks(schema) {
                                    Some(self.search_on_text(&path, data, &matching_mode))
                                } else { None }
                            }).collect();
                        }
                    }
                },

                // Asskit files are only tables.
                SearchSource::AssKitFiles => {
                    if self.search_on_dbs {
                        let game_selected = GAME_SELECTED.read().unwrap();
                        let tables = dependencies.get_ref_asskit_only_db_tables();
                        self.matches_db = tables.par_iter().filter_map(|table| {
                            let table_name = match game_selected.get_vanilla_db_table_name_logic() {
                                VanillaDBTableNameLogic::FolderName => table.get_table_name(),
                                VanillaDBTableNameLogic::DefaultName(ref default_name) => default_name.to_owned()
                            };

                            let path = vec!["db".to_owned(), table.get_table_name(), table_name];
                            Some(self.search_on_db(&path, table, &matching_mode))
                        }).collect();
                    }
                },
            }

            if self.search_on_schema {
                self.search_on_schema(schema, &matching_mode);
            }
        }
    }

    /// This function performs a limited search on the `PackedFiles` in the provided paths, and updates the `GlobalSearch` with the results.
    ///
    /// This means that, as long as you change any `PackedFile` in the `PackFile`, you should trigger this. That way, the `GlobalSearch`
    /// will always be up-to-date in an efficient way.
    ///
    /// If you passed the entire `PackFile` to this and it crashed, it's not an error. I forced that crash. If you want to do that,
    /// use the normal search function, because it's a lot more efficient than this one.
    ///
    /// NOTE: The schema search is not updated on schema change. Remember that.
    pub fn update(&mut self, pack_file: &mut PackFile, updated_paths: &[PathType]) {

        // Don't do anything if we have no pattern to search.
        if self.pattern.is_empty() { return }

        // This is only useful for PackFiles, not for dependencies..
        if self.source != SearchSource::PackFile { return }

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };

        // Turn all our updated packs into `PackedFile` paths, and get them.
        let mut paths = vec![];
        for path_type in updated_paths {
            match path_type {
                PathType::File(path) => paths.push(path.to_vec()),
                PathType::Folder(path) => paths.append(&mut pack_file.get_ref_packed_files_by_path_start(path).iter().map(|x| x.get_path().to_vec()).collect()),
                _ => unimplemented!()
            }
        }

        // We remove the added/edited/deleted files from all the search.
        for path in &paths {
            self.matches_db.retain(|x| &x.path != path);
            self.matches_loc.retain(|x| &x.path != path);
            self.matches_text.retain(|x| &x.path != path);
        }

        // If we got no schema, don't even decode.
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            for path in &paths {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(path) {
                    match packed_file.decode_return_ref_no_locks(schema).unwrap_or(&DecodedPackedFile::Unknown) {
                        DecodedPackedFile::DB(data) => {
                            if self.search_on_dbs {
                                self.matches_db.push(self.search_on_db(path, data, &matching_mode));
                            }
                        }
                        DecodedPackedFile::Loc(data) => {
                            if self.search_on_locs {
                                self.matches_loc.push(self.search_on_loc(path, data, &matching_mode));
                            }
                        }
                        DecodedPackedFile::Text(data) => {
                            if self.search_on_texts {
                                self.matches_text.push(self.search_on_text(path, data, &matching_mode));
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    /// This function clears the Global Search result's data, and reset the UI for it.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// This function returns the PackedFileInfo for all the PackedFiles the current search has searched on.
    pub fn get_results_packed_file_info(&self, pack_file: &mut PackFile) -> Vec<PackedFileInfo> {
        let mut types = vec![];
        if self.search_on_dbs { types.push(PackedFileType::DB); }
        if self.search_on_locs { types.push(PackedFileType::Loc); }
        if self.search_on_texts { types.push(PackedFileType::Text(TextType::Plain)); }
        let packed_files = pack_file.get_ref_packed_files_by_types(&types, false);
        packed_files.iter().map(|x| From::from(*x)).collect()
    }

    /// This function returns the PackedFileInfo for all the PackedFiles with the provided paths.
    pub fn get_update_paths_packed_file_info(&self, pack_file: &mut PackFile, paths: &[PathType]) -> Vec<PackedFileInfo> {
        let paths = paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect();
        let packed_files = pack_file.get_ref_packed_files_by_paths(paths);
        packed_files.iter().map(|x| From::from(*x)).collect()
    }

    /// This function performs a replace operation over the provided matches.
    ///
    /// NOTE: Schema matches are always ignored.
    pub fn replace_matches(&mut self, pack_file: &mut PackFile, matches: &[MatchHolder]) -> Vec<Vec<String>>{
        let mut errors = vec![];

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };
        let schema = &*SCHEMA.read().unwrap();
        if let Some(ref schema) = schema {
            let mut changed_files = vec![];
            for match_file in matches {
                match match_file {
                    MatchHolder::Table(match_table) => {
                        if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                            if let Ok(packed_file) = packed_file.decode_return_ref_mut_no_locks(schema) {
                                match packed_file {
                                    DecodedPackedFile::DB(ref mut table) => {
                                        let mut data = table.get_table_data();
                                        for match_data in &match_table.matches {

                                            // If any replace in the table fails, forget about this one and try the next one.
                                            if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                                changed_files.retain(|x| x != &match_table.path);
                                                errors.push(match_table.path.to_vec());
                                                break;
                                            }
                                        }

                                        if changed_files.contains(&match_table.path) && table.set_table_data(&data).is_err() {
                                            changed_files.retain(|x| x != &match_table.path);
                                            errors.push(match_table.path.to_vec());
                                        }
                                    }
                                    DecodedPackedFile::Loc(ref mut table)=> {
                                        let mut data = table.get_table_data();
                                        for match_data in &match_table.matches {

                                            // If any replace in the table fails, forget about this one and try the next one.
                                            if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                                changed_files.retain(|x| x != &match_table.path);
                                                errors.push(match_table.path.to_vec());
                                                break;
                                            }
                                        }

                                        if changed_files.contains(&match_table.path) && table.set_table_data(&data).is_err() {
                                            changed_files.retain(|x| x != &match_table.path);
                                            errors.push(match_table.path.to_vec());
                                        }
                                    }
                                    _ => unimplemented!()
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

            let changed_files = changed_files.iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>();
            self.update(pack_file, &changed_files);
        }
        errors
    }

    /// This function performs a replace operation over the entire match set, except schemas..
    pub fn replace_all(&mut self, pack_file: &mut PackFile) -> Vec<Vec<String>> {
        let mut errors = vec![];

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };
        let schema = &*SCHEMA.read().unwrap();
        if let Some(ref schema) = schema {
            let mut changed_files = vec![];
            for match_table in &self.matches_db {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(DecodedPackedFile::DB(ref mut table)) = packed_file.decode_return_ref_mut_no_locks(schema) {
                        let mut data = table.get_table_data();
                        for match_data in &match_table.matches {

                            // If any replace in the table fails, forget about this one and try the next one.
                            if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                changed_files.retain(|x| x != &match_table.path);
                                errors.push(match_table.path.to_vec());
                                break;
                            }
                        }

                        if changed_files.contains(&match_table.path) && table.set_table_data(&data).is_err() {
                            changed_files.retain(|x| x != &match_table.path);
                            errors.push(match_table.path.to_vec());
                        }
                    }
                }
            }

            for match_table in &self.matches_loc {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(DecodedPackedFile::Loc(ref mut table)) = packed_file.decode_return_ref_mut_no_locks(schema) {
                        let mut data = table.get_table_data();
                        for match_data in &match_table.matches {

                            // If any replace in the table fails, forget about this one and try the next one.
                            if self.replace_match_table(&mut data, &mut changed_files, match_table, match_data, &matching_mode).is_err() {
                                changed_files.retain(|x| x != &match_table.path);
                                errors.push(match_table.path.to_vec());
                                break;
                            }
                        }

                        if changed_files.contains(&match_table.path) && table.set_table_data(&data).is_err() {
                            changed_files.retain(|x| x != &match_table.path);
                            errors.push(match_table.path.to_vec());
                        }
                    }
                }
            }

            let changed_files = changed_files.iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>();
            self.update(pack_file, &changed_files);
        }

        errors
    }

    /// This function tries to replace data in a Table PackedFile. It fails if the data is not suitable for that column.
    fn replace_match_table(
        &self,
        data: &mut Vec<Vec<DecodedData>>,
        changed_files: &mut Vec<Vec<String>>,
        match_table: &TableMatches,
        match_data: &TableMatch,
        matching_mode: &MatchingMode,
    ) -> Result<()> {
        if let Some(row) = data.get_mut((match_data.row_number) as usize) {
            if let Some(field) = row.get_mut(match_data.column_number as usize) {
                match field {
                    DecodedData::Boolean(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = &string == "true";
                    }
                    DecodedData::F32(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<f32>()?;
                    }
                    DecodedData::I16(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i16>()?;
                    }
                    DecodedData::I32(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i32>()?;
                    }
                    DecodedData::I64(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i64>()?;
                    }
                    DecodedData::StringU8(ref mut field) |
                    DecodedData::StringU16(ref mut field) |
                    DecodedData::OptionalStringU8(ref mut field) |
                    DecodedData::OptionalStringU16(ref mut field) => self.replace_match(field, matching_mode),
                    DecodedData::SequenceU16(_) | DecodedData::SequenceU32(_) => return Err(ErrorKind::Generic.into()),
                }

                if !changed_files.contains(&match_table.path) {
                    changed_files.push(match_table.path.to_vec());
                }
            }
        }
        Ok(())
    }

    /// This function replaces all the matches in the provided text.
    fn replace_match(&self, text: &mut String, matching_mode: &MatchingMode) {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if regex.is_match(text) {
                    *text = regex.replace_all(text, &*self.replace_text).to_string();
                }
            }
            MatchingMode::Pattern => {
                let mut index = 0;
                while let Some(start) = text.find(&self.pattern) {

                    // Advance the index so we don't get trapped in an infinite loop... again.
                    if start >= index {
                        let end = start + self.pattern.len();
                        text.replace_range(start..end, &self.replace_text);
                        index = end;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// This function performs a search over the provided DB Table.
    fn search_on_db(&self, path: &[String], table_data: &DB, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(path);

        for (row_number, row) in table_data.get_ref_table_data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                match cell {
                    DecodedData::Boolean(ref data) => {
                        let text = if *data { "true" } else { "false" };
                        self.match_decoded_data(text, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64);
                    }
                    DecodedData::F32(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I16(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I32(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I64(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::SequenceU16(_) | DecodedData::SequenceU32(_) => continue,
                }
            }
        }

        matches
    }

    /// This function performs a search over the provided Loc Table.
    fn search_on_loc(&self, path: &[String], table_data: &Loc, matching_mode: &MatchingMode) -> TableMatches {
        let mut matches = TableMatches::new(path);

        for (row_number, row) in table_data.get_ref_table_data().iter().enumerate() {
            for (column_number, cell) in row.iter().enumerate() {
                match cell {
                    DecodedData::Boolean(ref data) => {
                        let text = if *data { "true" } else { "false" };
                        self.match_decoded_data(text, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64);
                    }
                    DecodedData::F32(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I16(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I32(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::I64(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::SequenceU16(_) | DecodedData::SequenceU32(_) => continue,
                }
            }
        }

        matches
    }

    /// This function performs a search over the provided Text PackedFile.
    fn search_on_text(&self, path: &[String], data: &Text, matching_mode: &MatchingMode) -> TextMatches {
        let mut matches = TextMatches::new(path);
        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (row, data) in data.get_ref_contents().lines().enumerate() {
                    for match_data in regex.find_iter(data) {
                        matches.matches.push(
                            TextMatch::new(
                                match_data.start() as u64,
                                row as u64,
                                (match_data.end() - match_data.start()) as i64,
                                data.to_owned()
                            )
                        );
                    }
                }
            }

            // If we're searching a pattern, we just check every text PackedFile, line by line.
            MatchingMode::Pattern => {
                let pattern = if self.case_sensitive { self.pattern.to_owned() } else { self.pattern.to_lowercase() };
                let length = self.pattern.chars().count();
                let mut column = 0;

                for (row, data) in data.get_ref_contents().lines().enumerate() {
                    while let Some(text) = data.get(column..) {
                        if self.case_sensitive {
                            match text.find(&pattern) {
                                Some(position) => {
                                    matches.matches.push(TextMatch::new(position as u64, row as u64, length as i64, data.to_owned()));
                                    column += position + length;
                                }
                                None => break,
                            }
                        }
                        else {
                            let text = text.to_lowercase();
                            match text.find(&pattern) {
                                Some(position) => {
                                    matches.matches.push(TextMatch::new(position as u64, row as u64, length as i64, data.to_owned()));
                                    column += position + length;
                                }
                                None => break,
                            }
                        }
                    }

                    column = 0;
                }
            }
        }

        matches
    }


    /// This function performs a search over the provided Text PackedFile.
    fn search_on_schema(&mut self, schema: &Schema, matching_mode: &MatchingMode) {
        for versioned_file in schema.get_ref_versioned_file_all() {
            let mut matches = vec![];
            match versioned_file {
                VersionedFile::AnimFragment(definitions) |
                VersionedFile::AnimTable(definitions) |
                VersionedFile::DB(_, definitions) |
                VersionedFile::DepManager(definitions) |
                VersionedFile::Loc(definitions) |
                VersionedFile::MatchedCombat(definitions) => {

                    match matching_mode {
                        MatchingMode::Regex(regex) => {
                            for definition in definitions {
                                for (index, field) in definition.get_fields_processed().iter().enumerate() {
                                    if regex.is_match(field.get_name()) {
                                        matches.push(SchemaMatch::new(
                                            definition.get_version(),
                                            index as u32,
                                            field.get_name().to_owned()
                                        ));
                                    }
                                }
                            }
                        }

                        // If we're searching a pattern, we just check every text PackedFile, line by line.
                        MatchingMode::Pattern => {
                            let pattern = if self.case_sensitive { self.pattern.to_owned() } else { self.pattern.to_lowercase() };
                            for definition in definitions {
                                for (index, field) in definition.get_fields_processed().iter().enumerate() {
                                    if self.case_sensitive {
                                        if field.get_name().contains(&pattern) {
                                            matches.push(SchemaMatch::new(
                                                definition.get_version(),
                                                index as u32,
                                                field.get_name().to_owned()
                                            ));
                                        }
                                    }
                                    else {
                                        let name = field.get_name().to_lowercase();
                                        if name.contains(&pattern) {
                                            matches.push(SchemaMatch::new(
                                                definition.get_version(),
                                                index as u32,
                                                field.get_name().to_owned()
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !matches.is_empty() {
                let (versioned_file_type, versioned_file_name) = match versioned_file {
                    VersionedFile::AnimFragment(_) => ("AnimFragment".to_owned(), None),
                    VersionedFile::AnimTable(_) => ("AnimTable".to_owned(), None),
                    VersionedFile::DB(name, _) => ("DB".to_owned(), Some(name.to_owned())),
                    VersionedFile::DepManager(_) => ("Dependency Manager".to_owned(), None),
                    VersionedFile::Loc(_) => ("Loc".to_owned(), None),
                    VersionedFile::MatchedCombat(_) => ("MatchedCombat".to_owned(), None),
                };
                let mut schema_matches = SchemaMatches::new(versioned_file_type, versioned_file_name);
                schema_matches.matches = matches;
                self.matches_schema.push(schema_matches);
            }
        }
    }


    /// This function check if the provided `&str` matches our search.
    fn match_decoded_data(
        &self,
        text: &str,
        matching_mode: &MatchingMode,
        matches: &mut Vec<TableMatch>,
        definition: &Definition,
        column_number: u32,
        row_number: i64,
    ) {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if regex.is_match(text) {
                    let column_name = &definition.get_fields_processed()[column_number as usize].get_name().to_owned();
                    matches.push(TableMatch::new(column_name, column_number, row_number, text));
                }
            }

            MatchingMode::Pattern => {
                if self.case_sensitive {
                    if text.contains(&self.pattern) {
                        let column_name = &definition.get_fields_processed()[column_number as usize].get_name().to_owned();
                        matches.push(TableMatch::new(column_name, column_number, row_number, text));
                    }
                }
                else {
                    let pattern = self.pattern.to_lowercase();
                    let text = text.to_lowercase();
                    if text.contains(&pattern) {
                        let column_name = &definition.get_fields_processed()[column_number as usize].get_name().to_owned();
                        matches.push(TableMatch::new(column_name, column_number, row_number, &text));
                    }
                }
            }
        }
    }
}
