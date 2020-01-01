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

This module contains the code needed to get a `GlobalSeach` over an entire `PackFile`.
!*/

use regex::Regex;
use rayon::prelude::*;

use rpfm_error::{ErrorKind, Result};

use crate::packfile::{PackFile, PathType};
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
/// if we try to use `Regex` and the provided regex expresion is invalid.
#[derive(Debug, Clone)]
enum MatchingMode {
    Regex(Regex),
    Pattern,
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
    pub fn search(&mut self, pack_file: &mut PackFile) {

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
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
            if self.search_on_dbs {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::DB);
                self.matches_db = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::DB(data) = decoded_packed_file {
                            Some(self.search_on_db(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
            }

            if self.search_on_locs {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::Loc);
                self.matches_loc = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::Loc(data) = decoded_packed_file {
                            Some(self.search_on_loc(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
            }

            if self.search_on_texts {
                let mut packed_files = pack_file.get_ref_mut_packed_files_by_type(&PackedFileType::Text(TextType::Plain));
                self.matches_text = packed_files.par_iter_mut().filter_map(|packed_file| {
                    let path = packed_file.get_path().to_vec();
                    if let Ok(decoded_packed_file) = packed_file.decode_return_ref_no_locks(&schema) {
                        if let DecodedPackedFile::Text(data) = decoded_packed_file {
                            Some(self.search_on_text(&path, &data, &matching_mode))
                        } else { None }
                    } else { None }
                }).collect();
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
        if &self.pattern == "" { return }

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
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
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
                    match packed_file.decode_return_ref_no_locks(&schema).unwrap_or_else(|_| &DecodedPackedFile::Unknown) {
                        DecodedPackedFile::DB(data) => {
                            if self.search_on_dbs {
                                self.search_on_db(&path, data, &matching_mode);
                            }
                        }
                        DecodedPackedFile::Loc(data) => {
                            if self.search_on_locs {
                                self.search_on_loc(&path, data, &matching_mode);
                            }
                        }
                        DecodedPackedFile::Text(data) => {
                            if self.search_on_texts {
                                self.search_on_text(&path, data, &matching_mode);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    /// This function clears the Global Search resutl's data, and reset the UI for it.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// This function performs a replace operation over the entire match set, except schemas..
    pub fn replace_all(&mut self, pack_file: &mut PackFile) -> Vec<Vec<String>> {
        let mut errors = vec![];

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            if let Ok(regex) = Regex::new(&self.pattern) {
                MatchingMode::Regex(regex)
            }
            else { MatchingMode::Pattern }
        } else { MatchingMode::Pattern };
        let schema = &*SCHEMA.read().unwrap();
        if let Some(ref schema) = schema {
            let mut changed_files = vec![];
            for match_table in &self.matches_db {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(packed_file) = packed_file.decode_return_ref_mut_no_locks(&schema) {
                        if let DecodedPackedFile::DB(ref mut table) = packed_file {
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
            }

            for match_table in &self.matches_loc {
                if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&match_table.path) {
                    if let Ok(packed_file) = packed_file.decode_return_ref_mut_no_locks(&schema) {
                        if let DecodedPackedFile::Loc(ref mut table) = packed_file {
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
                    DecodedData::Float(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<f32>()?;
                    }
                    DecodedData::Integer(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i32>()?;
                    }
                    DecodedData::LongInteger(ref mut field) => {
                        let mut string = field.to_string();
                        self.replace_match(&mut string, matching_mode);
                        *field = string.parse::<i64>()?;
                    }
                    DecodedData::StringU8(ref mut field) |
                    DecodedData::StringU16(ref mut field) |
                    DecodedData::OptionalStringU8(ref mut field) |
                    DecodedData::OptionalStringU16(ref mut field) => self.replace_match(field, matching_mode),
                    DecodedData::Sequence(_) => return Err(ErrorKind::Generic.into()),
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
                if regex.is_match(&text) {
                    *text = regex.replace_all(&text, &*self.replace_text).to_string();
                }
            }
            MatchingMode::Pattern => {
                while let Some(start) = text.find(&self.pattern) {
                    let end = start + self.pattern.len();
                    text.replace_range(start..end, &self.replace_text);
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
                    DecodedData::Float(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Integer(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::LongInteger(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Sequence(_) => continue,
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
                    DecodedData::Float(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Integer(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::LongInteger(ref data) => self.match_decoded_data(&data.to_string(), matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),

                    DecodedData::StringU8(ref data) |
                    DecodedData::StringU16(ref data) |
                    DecodedData::OptionalStringU8(ref data) |
                    DecodedData::OptionalStringU16(ref data) => self.match_decoded_data(data, matching_mode, &mut matches.matches, table_data.get_ref_definition(), column_number as u32, row_number as i64),
                    DecodedData::Sequence(_) => continue,
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
                let lenght = self.pattern.chars().count();
                let mut column = 0;

                for (row, data) in data.get_ref_contents().lines().enumerate() {
                    while let Some(text) = data.get(column..) {
                        match text.find(&self.pattern) {
                            Some(position) => {
                                matches.matches.push(TextMatch::new(position as u64, row as u64, lenght as i64, data.to_owned()));
                                column += position + lenght;
                            }
                            None => break,
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
                VersionedFile::DB(_, definitions) |
                VersionedFile::Loc(definitions) |
                VersionedFile::DepManager(definitions)  => {

                    match matching_mode {
                        MatchingMode::Regex(regex) => {
                            for definition in definitions {
                                for (index, field) in definition.fields.iter().enumerate() {
                                    if regex.is_match(&field.name) {
                                        matches.push(SchemaMatch::new(
                                            definition.version,
                                            index as u32,
                                            field.name.to_owned()
                                        ));
                                    }
                                }
                            }
                        }

                        // If we're searching a pattern, we just check every text PackedFile, line by line.
                        MatchingMode::Pattern => {
                            for definition in definitions {
                                for (index, field) in definition.fields.iter().enumerate() {
                                    if field.name.contains(&self.pattern) {
                                        matches.push(SchemaMatch::new(
                                            definition.version,
                                            index as u32,
                                            field.name.to_owned()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !matches.is_empty() {
                let (versioned_file_type, versioned_file_name) = match versioned_file {
                    VersionedFile::DB(name, _) => ("DB".to_owned(), Some(name.to_owned())),
                    VersionedFile::Loc(_) => ("Loc".to_owned(), None),
                    VersionedFile::DepManager(_) => ("Dependency Manager".to_owned(), None),
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
                if regex.is_match(&text) {
                    let column_name = &definition.fields[column_number as usize].name;
                    matches.push(TableMatch::new(&column_name, column_number, row_number, text));
                }
            }

            MatchingMode::Pattern => {
                if text.contains(&self.pattern) {
                    let column_name = &definition.fields[column_number as usize].name;
                    matches.push(TableMatch::new(column_name, column_number, row_number, text));
                }
            }
        }
    }
}
