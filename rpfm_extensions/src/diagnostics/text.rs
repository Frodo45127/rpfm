//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the structs and functions specific for `Text` diagnostics.

use getset::{Getters, MutGetters};
use serde_derive::{Serialize, Deserialize};

use std::collections::{HashMap, HashSet};
use std::{fmt, fmt::Display};

use rpfm_lib::files::{RFile, RFileDecoded};
use rpfm_lib::utils::*;

use crate::dependencies::Dependencies;
use crate::diagnostics::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the results of a Text diagnostic.
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TextDiagnostic {
    path: String,
    pack: String,
    results: Vec<TextDiagnosticReport>
}

/// This struct defines an individual Text diagnostic result.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct TextDiagnosticReport {
    report_type: TextDiagnosticReportType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDiagnosticReportType {
    InvalidKey((u64, u64), (u64, u64), String, String, String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl TextDiagnosticReport {
    pub fn new(report_type: TextDiagnosticReportType) -> Self {
        Self {
            report_type
        }
    }
}

impl DiagnosticReport for TextDiagnosticReport {
    fn message(&self) -> String {
        match &self.report_type {
            TextDiagnosticReportType::InvalidKey(_,_, table, column, key) => "Invalid Key: \"".to_string() + key + "\" is not in table \"" + table + "\", column \"" + column + "\".",
        }
    }

    fn level(&self) -> DiagnosticLevel {
        match self.report_type {
            TextDiagnosticReportType::InvalidKey(_,_,_,_,_) => DiagnosticLevel::Error,
        }
    }
}

impl Display for TextDiagnosticReportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::InvalidKey(_,_,_,_,_) => "InvalidKey",
        }, f)
    }
}

impl TextDiagnostic {
    pub fn new(path: &str, pack: &str) -> Self {
        Self {
            path: path.to_owned(),
            pack: pack.to_owned(),
            results: vec![],
        }
    }

    /// This function takes care of checking for Text-Related for errors.
    pub fn check(
        file: &RFile,
        pack: &Pack,
        dependencies: &Dependencies,
        global_ignored_diagnostics: &[String],
        ignored_fields: &[String],
        ignored_diagnostics: &HashSet<String>,
        ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>,
    ) -> Option<DiagnosticType> {


        if let Ok(RFileDecoded::Text(text)) = file.decoded() {
            let mut diagnostic = Self::new(file.path_in_container_raw(), file.container_name().as_deref().unwrap_or(""));

            let text = text.contents();
            let mut start_pos = 0;

            // We're only interested in tables marked with "--@db".
            while let Some(pos) = text[start_pos..].find("--@db ") {
                if let Some(end_line) = text[start_pos + pos..].find('\n') {

                    // We only support single-line comments.
                    let table_data = text[start_pos + pos + 6..start_pos + pos + end_line].split(' ').collect::<Vec<_>>();

                    // We expect table name and column.
                    if table_data.len() >= 2 {
                        let table_name = if table_data[0].ends_with("_tables") { table_data[0].to_owned() } else { table_data[0].to_owned() + "_tables" };
                        let table_column = if table_data[1].ends_with("\r") {
                            &table_data[1][..table_data[1].len() - 1]
                        } else {
                            table_data[1]
                        };

                        let index_to_check = if let Some(indexes) = table_data.get(2) {
                            indexes.split(",")
                                .filter_map(|x| x.parse::<usize>().ok())
                                .collect()
                        } else {
                            vec![]
                        };

                        // We need to make sure we only check the next line for the start, or we may end up checking the wrong vars.
                        let (next_line_start, next_line_end) = match text[start_pos + pos + 6..].find('\n') {
                            Some(nls) => if text.as_bytes().get(start_pos + pos + 6 + nls + 1).is_some() {
                                match text[start_pos + pos + 6 + nls + 1..].find('\n') {
                                    Some(nle) => (start_pos + pos + 6 + nls + 1, start_pos + pos + 6 + nls + 1 + nle),
                                    None => break,
                                }
                            } else {
                                break;
                            }

                            None => break,
                        };

                        // Formats supported:
                        // - Single line, single value:
                        //      hb = "key"
                        //
                        // - Single line, single table:
                        //      hb = { "a", "b" }
                        //
                        // - Single line, keyed table:
                        //      hb = { "a" = "b", "c" = "d" }
                        //
                        // - Multiple lines, single table:
                        //      hb = {
                        //          "a",
                        //          "b"
                        //      }
                        //
                        // - Multiple lines, keyed table (support for key and value:
                        //      hb = {
                        //          "a" = "b"
                        //          "c" = "d"
                        //      }

                        // Data to search are strings in commas between {}.
                        let (keys, data_start, data_end) = {
                            let mut vals = (vec![], 0, 0);

                            if let Some(data_start) = text[next_line_start..next_line_end].find('{') {
                                if let Some(data_end) = text[next_line_start + data_start..].find('}') {

                                    // +1 to not include the { at the start.
                                    let data_to_search = &text[next_line_start + data_start + 1..next_line_start + data_start + data_end];

                                    // Multi-line table.
                                    if data_to_search.contains('\n') || data_to_search.contains('\r') {

                                        // Keyed table.
                                        if data_to_search.contains('=') {
                                            let data_split = data_to_search.split('\n')
                                                .filter_map(|x| {
                                                    let spl = x.split('=')
                                                        .map(|y| y.split('\"').collect::<Vec<_>>());

                                                    let mut keys = vec![];
                                                    for (i, data) in spl.enumerate() {
                                                        if index_to_check.contains(&i) && data.len() == 3 {
                                                            keys.push(data[1].to_owned());
                                                        }
                                                    }

                                                    if !keys.is_empty() {
                                                        Some(keys)
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .flatten()
                                                .collect::<Vec<_>>();

                                            vals = (data_split, data_start, data_end)
                                        }

                                        // Non-keyed/single value table.
                                        else {
                                            let data_split = data_to_search.split('\n')
                                                .filter_map(|x| {

                                                    // On each line, we want the data between commas.
                                                    let spl = x.split('\"').collect::<Vec<_>>();
                                                    if spl.len() == 3 {
                                                        Some(spl[1].to_owned())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect::<Vec<_>>();

                                            vals = (data_split, data_start, data_end)
                                        }
                                    }

                                    // Single line keyed table.
                                    else if data_to_search.contains('=') {
                                        let data_split = data_to_search.split(',')
                                            .filter_map(|x| {
                                                let spl = x.split('=')
                                                    .map(|y| y.split('\"').collect::<Vec<_>>());

                                                let mut keys = vec![];
                                                for (i, data) in spl.enumerate() {
                                                    if index_to_check.contains(&i) && data.len() == 3 {
                                                        keys.push(data[1].to_owned());
                                                    }
                                                }

                                                if !keys.is_empty() {
                                                    Some(keys)
                                                } else {
                                                    None
                                                }
                                            })
                                            .flatten()
                                            .collect::<Vec<_>>();

                                        vals = (data_split, data_start, data_end)
                                    }

                                    // Single line non-keyed table.
                                    else {
                                        let data_split = data_to_search.split(',')
                                            .filter_map(|x| {

                                                // On each line, we want the data between commas.
                                                let spl = x.split('\"').collect::<Vec<_>>();
                                                if spl.len() == 3 {
                                                    Some(spl[1].to_owned())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect::<Vec<_>>();

                                        vals = (data_split, data_start, data_end)
                                    }
                                }
                            }

                            // No { means it's single line, single value.
                            else if let Some(data_start) = text[next_line_start..next_line_end].find('\"') {
                                // +1 to skip the starting comma.
                                if text.as_bytes().get(next_line_start + data_start + 1).is_some() {
                                    if let Some(data_end) = text[next_line_start + data_start + 1..].find('\"') {
                                       if text.as_bytes().get(next_line_start + data_start + 1 + data_end).is_some() {
                                            let data_to_search = &text[next_line_start + data_start + 1..next_line_start + data_start + 1 + data_end];
                                            vals = (vec![data_to_search.to_string()], data_start, data_end)
                                        }
                                    }
                                }
                            }

                            vals
                        };

                        let mut not_found = HashMap::new();

                        // Add the files from the dependencies, then the files from the pack, then reverse the list so we process first the pack ones.
                        if let Ok(mut tables) = dependencies.db_data(&table_name, true, true) {
                            tables.append(&mut pack.files_by_path(&ContainerPath::Folder("db/".to_string() + &table_name + "/"), true));
                            tables.reverse();

                            // If there are no tables that match out name, ignore it.
                            if tables.is_empty() {
                                start_pos = next_line_start + data_start + data_end;
                                continue;
                            }

                            for key in &keys {
                                let key_to_check = key.trim();

                                // Calculate the row, column_start and column_end of the data.
                                let start_cursor = line_column_from_string_pos(text, (next_line_start + data_start + 1) as u64);
                                let end_cursor = line_column_from_string_pos(text, (next_line_start + data_start + 1 + data_end) as u64);

                                let mut found = false;
                                for table in &tables {
                                    if let Ok(RFileDecoded::DB(table)) = table.decoded() {
                                        let definition = table.definition();
                                        if let Some(column) = definition.column_position_by_name(table_column) {
                                            for row in table.data().iter() {
                                                if row[column].data_to_string() == *key_to_check {
                                                    found = true;
                                                    break;
                                                }
                                            }

                                            if found {
                                                break;
                                            }
                                        }
                                    }
                                }

                                if !found {
                                    not_found.insert(key_to_check, (start_cursor, end_cursor));
                                }
                            }

                            for (key, (start, end)) in &not_found {
                                if !Diagnostics::ignore_diagnostic(global_ignored_diagnostics, None, Some("InvalidKey"), ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields)  {
                                    let result = TextDiagnosticReport::new(TextDiagnosticReportType::InvalidKey(*start, *end, table_name.to_string(), table_column.to_string(), key.to_string()));
                                    diagnostic.results_mut().push(result);
                                }
                            }
                        }

                        start_pos = next_line_start + data_start + data_end;
                    }
                }
            }

            if !diagnostic.results().is_empty() {
                Some(DiagnosticType::Text(diagnostic))
            } else { None }
        } else { None }
    }
}
