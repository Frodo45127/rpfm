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
Module with all the code related to the `Diagnostics`.

This module contains the code needed to get a `Diagnostics` over an entire `PackFile`.
!*/

use rayon::prelude::*;
use fancy_regex::Regex;

use crate::DB;
use crate::DEPENDENCY_DATABASE;
use crate::FAKE_DEPENDENCY_DATABASE;
use crate::packfile::{PackFile, PathType};
use crate::packedfile::{table::DecodedData, DecodedPackedFile, PackedFileType};
use crate::packfile::packedfile::PackedFileInfo;
use crate::PackedFile;
use crate::schema::FieldType;

use self::packfile::PackFileDiagnostic;
use self::table::{TableDiagnostic, TableDiagnosticReport};

pub mod packfile;
pub mod table;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a diagnostics check over multiple PackedFiles.
#[derive(Debug, Clone)]
pub struct Diagnostics(Vec<DiagnosticType>);

#[derive(Debug, Clone)]
pub enum DiagnosticType {
    DB(TableDiagnostic),
    Loc(TableDiagnostic),
    PackFile(PackFileDiagnostic),
}

/// This enum defines the possible results for a result of a diagnostic check.
#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `Default` for `Diagnostics`.
impl Default for Diagnostics {
    fn default() -> Self {
        Self(vec![])
    }
}

impl DiagnosticType {
    pub fn get_path(&self) -> &[String] {
        match self {
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.get_path(),
            Self::PackFile(ref diag) => diag.get_path(),
        }
    }
}

/// Implementation of `Diagnostics`.
impl Diagnostics {

    pub fn get_ref_diagnostics(&self) -> &[DiagnosticType] {
        &self.0
    }

    pub fn get_ref_mut_diagnostics(&mut self) -> &mut Vec<DiagnosticType> {
        &mut self.0
    }

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn check(&mut self, pack_file: &PackFile) {
        let real_dep_db = DEPENDENCY_DATABASE.read().unwrap();
        let fake_dep_db = FAKE_DEPENDENCY_DATABASE.read().unwrap();
        self.0 = pack_file.get_ref_packed_files_by_types(&[PackedFileType::DB, PackedFileType::Loc], false).par_iter().filter_map(|packed_file| {
            match packed_file.get_packed_file_type_by_path() {
                PackedFileType::DB => Self::check_db(pack_file, packed_file.get_ref_decoded(), packed_file.get_path(), &real_dep_db, &fake_dep_db),
                PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path()),
                _ => None,
            }
        }).collect();

        if let Some(diagnostics) = Self::check_packfile() {
            self.0.push(diagnostics);
        }
    }

    /// This function takes care of checking the db tables of your mod for errors.
    fn check_db(
        pack_file: &PackFile,
        packed_file: &DecodedPackedFile,
        path: &[String],
        real_dep_db: &[PackedFile],
        fake_dep_db: &[DB],
    ) ->Option<DiagnosticType> {
        if let DecodedPackedFile::DB(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);
            let dependency_data = DB::get_dependency_data(
                &pack_file,
                table.get_ref_definition(),
                real_dep_db,
                fake_dep_db,
                &[],
            );

            // Check all the columns with reference data.
            let mut columns_without_reference_table = vec![];
            let mut columns_with_reference_table_and_no_column = vec![];
            let mut keys = vec![];

            // Before anything else, check if the table is outdated.
            if table.is_outdated() {
                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                    column_number: 0,
                    row_number: -1,
                    message: "Possibly outdated table.".to_owned(),
                    level: DiagnosticLevel::Error,
                });
            }

            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let mut row_is_empty = true;
                let mut row_keys_are_empty = true;
                let mut local_keys = vec![];
                for (column, field) in table.get_ref_definition().get_fields_processed().iter().enumerate() {
                    let cell_data = cells[column].data_to_string();

                    // First, check if we have dependency data for that column.
                    if field.get_is_reference().is_some() {
                        match dependency_data.get(&(column as i32)) {
                            Some(ref_data) => {

                                // Blue cell check. Only one for each column, so we don't fill the diagnostics with this.
                                if ref_data.is_empty() {
                                    if !columns_with_reference_table_and_no_column.contains(&column) {
                                        columns_with_reference_table_and_no_column.push(column);
                                    }
                                }

                                // Check for non-empty cells with reference data, but the data in the cell is not in the reference data list.
                                else if !cell_data.is_empty() && !ref_data.contains_key(&cell_data) {
                                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                                        column_number: column as u32,
                                        row_number: row as i64,
                                        message: format!("Invalid reference \"{}\" in column \"{}\".", &cell_data, table.get_ref_definition().get_fields_processed()[column].get_name()),
                                        level: DiagnosticLevel::Error,
                                    });
                                }
                            }
                            None => {
                                if !columns_without_reference_table.contains(&column) {
                                    columns_without_reference_table.push(column);
                                }
                            }
                        }
                    }

                    // Check for empty keys/rows.
                    if row_is_empty && (!cell_data.is_empty() && cell_data != "false") {
                        row_is_empty = false;
                    }

                    if row_keys_are_empty && field.get_is_key() && (!cell_data.is_empty() && cell_data != "false") {
                        row_keys_are_empty = false;
                    }

                    if field.get_is_key() && field.get_field_type() != FieldType::OptionalStringU8 && field.get_field_type() != FieldType::Boolean && (cell_data.is_empty() || cell_data == "false") {
                        diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                            column_number: column as u32,
                            row_number: row as i64,
                            message: format!("Empty key for column \"{}\".", table.get_ref_definition().get_fields_processed()[column].get_name()),
                            level: DiagnosticLevel::Warning,
                        });
                    }

                    if field.get_is_key() {
                        local_keys.push(cell_data);
                    }
                }

                if row_is_empty {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Empty row.".to_string(),
                        level: DiagnosticLevel::Error,
                    });
                }

                if row_keys_are_empty {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Empty key fields.".to_string(),
                        level: DiagnosticLevel::Warning,
                    });
                }

                if local_keys.len() > 1 && keys.contains(&local_keys) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Duplicated combined keys.".to_string(),
                        level: DiagnosticLevel::Error,
                    });
                }
                else {
                    keys.push(local_keys);
                }
            }

            // Checks that only need to be done once per table.
            for column in &columns_without_reference_table {
                diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                    column_number: *column as u32,
                    row_number: -1,
                    message: format!("No reference table found for column \"{}\".", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                    level: DiagnosticLevel::Info,
                });
            }

            for column in &columns_with_reference_table_and_no_column {
                if fake_dep_db.is_empty() {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: *column as u32,
                        row_number: -1,
                        message: format!("No reference column found in referenced table for column \"{}\". Did you forgot to generate the PAK file for this game?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                        level: DiagnosticLevel::Warning,
                    });
                }
                else {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: *column as u32,
                        row_number: -1,
                        message: format!("No reference column found in referenced table for column \"{}\". Maybe a problem with the schema?", table.get_ref_definition().get_fields_processed()[*column as usize].get_name()),
                        level: DiagnosticLevel::Info,
                    });
                }
            }

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::DB(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking the loc tables of your mod for errors.
    fn check_loc(packed_file: &DecodedPackedFile, path: &[String]) ->Option<DiagnosticType> {
        if let DecodedPackedFile::Loc(table) = packed_file {
            let mut diagnostic = TableDiagnostic::new(path);

            // Check all the columns with reference data.
            let mut keys = vec![];
            for (row, cells) in table.get_ref_table_data().iter().enumerate() {
                let key = if let DecodedData::StringU16(ref data) = cells[0] { data } else { unimplemented!() };
                let data = if let DecodedData::StringU16(ref data) = cells[1] { data } else { unimplemented!() };

                if key.is_empty() && data.is_empty() {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Empty row.".to_string(),
                        level: DiagnosticLevel::Warning,
                    });
                }

                if key.is_empty() && !data.is_empty() {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Empty key.".to_string(),
                        level: DiagnosticLevel::Warning,
                    });
                }

                // Magic Regex. It works. Don't ask why.
                if !data.is_empty() && Regex::new(r"(?<!\\)\\n|(?<!\\)\\t").unwrap().is_match(data).unwrap() {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 1,
                        row_number: row as i64,
                        message: "Invalid line jump/tabulation detected in loc entry. Use \\\\n or \\\\t instead.".to_string(),
                        level: DiagnosticLevel::Warning,
                    });
                }

                let local_keys = vec![key, data];
                if keys.contains(&local_keys) {
                    diagnostic.get_ref_mut_result().push(TableDiagnosticReport {
                        column_number: 0,
                        row_number: row as i64,
                        message: "Duplicated row.".to_string(),
                        level: DiagnosticLevel::Warning,
                    });
                }
                else {
                    keys.push(local_keys);
                }
            }

            if !diagnostic.get_ref_result().is_empty() {
                Some(DiagnosticType::Loc(diagnostic))
            } else { None }
        } else { None }
    }

    /// This function takes care of checking for PackFile-Related for errors.
    fn check_packfile() ->Option<DiagnosticType> {
        let diagnostic = PackFileDiagnostic::new();
        if !diagnostic.get_ref_result().is_empty() {
            Some(DiagnosticType::PackFile(diagnostic))
        } else { None }
    }

    /// This function performs a limited diagnostic check on the `PackedFiles` in the provided paths, and updates the `Diagnostic` with the results.
    ///
    /// This means that, as long as you change any `PackedFile` in the `PackFile`, you should trigger this. That way, the `Diagnostics`
    /// will always be up-to-date in an efficient way.
    ///
    /// If you passed the entire `PackFile` to this and it crashed, it's not an error. I forced that crash. If you want to do that,
    /// use the normal check function, because it's a lot more efficient than this one.
    ///
    /// NOTE: The schema search is not updated on schema change. Remember that.
    pub fn update(&mut self, pack_file: &PackFile, updated_paths: &[PathType]) {

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
            self.get_ref_mut_diagnostics().retain(|x| &x.get_path() != path);
        }

        // If we got no schema, don't even decode.
        let real_dep_db = DEPENDENCY_DATABASE.read().unwrap();
        let fake_dep_db = FAKE_DEPENDENCY_DATABASE.read().unwrap();
        for packed_file in pack_file.get_ref_packed_files_by_paths(paths.iter().map(|x| x.as_ref()).collect()) {
            let diagnostic = match packed_file.get_packed_file_type_by_path() {
                PackedFileType::DB => Self::check_db(pack_file, packed_file.get_ref_decoded(), packed_file.get_path(), &real_dep_db, &fake_dep_db),
                PackedFileType::Loc => Self::check_loc(packed_file.get_ref_decoded(), packed_file.get_path()),
                _ => None,
            };

            if let Some(diagnostic) = diagnostic {
                self.get_ref_mut_diagnostics().push(diagnostic);
            }
        }

        self.get_ref_mut_diagnostics().sort_by(|a, b| a.get_path().cmp(b.get_path()));
    }

    /// This function returns the PackedFileInfo for all the PackedFiles with the provided paths.
    pub fn get_update_paths_packed_file_info(&self, pack_file: &PackFile, paths: &[PathType]) -> Vec<PackedFileInfo> {
        let paths = paths.iter().filter_map(|x| if let PathType::File(path) = x { Some(&**path) } else { None }).collect();
        let packed_files = pack_file.get_ref_packed_files_by_paths(paths);
        packed_files.iter().map(|x| From::from(*x)).collect()
    }
}
