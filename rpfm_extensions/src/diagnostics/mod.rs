//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Pack validation and diagnostic checking system.
//!
//! This module provides comprehensive validation for Total War mod packs, detecting
//! common errors, potential issues, and best practice violations. Diagnostics help
//! modders identify problems before they cause crashes or unexpected behavior in-game.
//!
//! # Diagnostic Types
//!
//! The system checks multiple aspects of a pack:
//!
//! - **Table Diagnostics** ([`table`]): DB and Loc table validation
//!   - Invalid foreign key references
//!   - Empty required fields (keys, values)
//!   - Duplicate rows
//!   - Orphaned localisation entries
//!
//! - **Pack Diagnostics** ([`pack`]): Pack-level checks
//!   - Files conflicting with vanilla
//!   - Missing declared dependencies
//!
//! - **Dependency Diagnostics** ([`dependency`]): Cross-pack validation
//!   - References to non-existent files
//!   - Circular dependencies
//!
//! - **Portrait Settings Diagnostics** ([`portrait_settings`]): Unit portrait validation
//!   - Invalid art set references
//!   - Missing variant definitions
//!
//! - **Animation Fragment Diagnostics** ([`anim_fragment_battle`]): Animation validation
//!   - Invalid animation references
//!   - Malformed fragment data
//!
//! - **Text Diagnostics** ([`text`]): Script validation
//!
//! - **Config Diagnostics** ([`config`]): Configuration file validation
//!
//! # Diagnostic Levels
//!
//! Each diagnostic has an associated severity level:
//!
//! - **Error**: Critical issues that will likely cause crashes or major problems
//! - **Warning**: Issues that may cause problems or indicate mistakes
//! - **Info**: Suggestions and informational notes
//!
//! # Cell Position Encoding
//!
//! For table diagnostics, the affected cells are encoded as (row, column) pairs:
//!
//! - `(-1, -1)`: Affects the entire table
//! - `(row, -1)`: Affects all columns in a single row
//! - `(-1, column)`: Affects all rows in a single column
//! - `(row, column)`: Affects a specific cell
//!
//! # Filtering
//!
//! Diagnostics can be filtered by:
//!
//! - Ignored folders (skip entire directory trees)
//! - Ignored files (skip specific files)
//! - Ignored fields (skip specific table columns)
//! - Ignored diagnostic types
//!
//! # Usage Example
//!
//! ```ignore
//! use rpfm_extensions::diagnostics::Diagnostics;
//!
//! let mut diagnostics = Diagnostics::default();
//! diagnostics.check(
//!     &mut pack,
//!     &mut dependencies,
//!     &schema,
//!     &game_info,
//!     game_path,
//!     &[],  // Check all paths
//!     false, // Don't check AK-only references
//! );
//!
//! for result in diagnostics.results() {
//!     println!("{}: {}", result.path(), result.message());
//! }
//! ```

use getset::{Getters, MutGetters};
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::cmp::Ordering;
use std::{fmt, fmt::Display};
use std::path::Path;

use rpfm_lib::error::Result;
use rpfm_lib::files::{ContainerPath, Container, DecodeableExtraData, FileType, pack::{DiagnosticIgnoreEntry, Pack}, RFile, RFileDecoded};
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::{FieldType, Schema};

use crate::dependencies::Dependencies;

use self::anim_fragment_battle::*;
use self::config::*;
use self::dependency::*;
use self::pack::*;
use self::portrait_settings::*;
use self::table::*;
use self::text::TextDiagnostic;

pub mod anim_fragment_battle;
pub mod config;
pub mod dependency;
pub mod pack;
pub mod portrait_settings;
pub mod table;
pub mod text;

//-------------------------------------------------------------------------------//
//                              Trait definitions
//-------------------------------------------------------------------------------//

/// Trait for types that can report diagnostic information.
///
/// All diagnostic result types implement this trait to provide a consistent
/// interface for accessing the diagnostic message and severity level.
pub trait DiagnosticReport {

    /// Returns the human-readable message describing this diagnostic.
    ///
    /// The message should clearly explain what the issue is and, where possible,
    /// suggest how to fix it.
    fn message(&self) -> String;

    /// Returns the severity level of this diagnostic.
    ///
    /// Used for filtering and prioritizing diagnostic results.
    fn level(&self) -> DiagnosticLevel;
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Container for diagnostic check results and configuration.
///
/// This struct holds both the configuration for which diagnostics to run
/// (via ignore lists) and the results of the diagnostic check.
///
/// # Filtering
///
/// Use the ignore fields to exclude certain items from diagnostic checks:
///
/// - `folders_ignored`: Skip entire folder trees (e.g., "db/deprecated_tables")
/// - `files_ignored`: Skip specific files by path
/// - `fields_ignored`: Skip specific table columns (format: "table_name/field_name")
/// - `diagnostics_ignored`: Skip specific diagnostic types by identifier
#[derive(Debug, Clone, Default, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Diagnostics {

    /// Folder paths to exclude from diagnostic checks.
    ///
    /// Files within these folders (and subfolders) will not be checked.
    folders_ignored: Vec<String>,

    /// File paths to exclude from diagnostic checks.
    files_ignored: Vec<String>,

    /// Table fields to exclude from diagnostic checks.
    ///
    /// Format: "table_name/field_name" (e.g., "units_tables/key")
    fields_ignored: Vec<String>,

    /// Diagnostic type identifiers to skip.
    ///
    /// Use this to disable specific checks that produce false positives
    /// or are not relevant to your mod.
    diagnostics_ignored: Vec<String>,

    /// The diagnostic results from the most recent check.
    results: Vec<DiagnosticType>
}

/// Wrapper enum for all diagnostic result types.
///
/// Each variant corresponds to a different file type or check category,
/// containing the specific diagnostic struct for that type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticType {
    /// Diagnostics for animation fragment battle files.
    AnimFragmentBattle(AnimFragmentBattleDiagnostic),
    /// Diagnostics for configuration files.
    Config(ConfigDiagnostic),
    /// Diagnostics for dependency-related issues.
    Dependency(DependencyDiagnostic),
    /// Diagnostics for DB tables.
    DB(TableDiagnostic),
    /// Diagnostics for Loc (localisation) tables.
    Loc(TableDiagnostic),
    /// Diagnostics for pack-level issues.
    Pack(PackDiagnostic),
    /// Diagnostics for portrait settings files.
    PortraitSettings(PortraitSettingsDiagnostic),
    /// Diagnostics for text/script files.
    Text(TextDiagnostic),
}

/// Severity level of a diagnostic result.
///
/// Used to categorize diagnostics by importance and filter results
/// in the user interface.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    /// Informational message or suggestion.
    ///
    /// These don't indicate errors but may highlight potential improvements
    /// or provide useful information about the mod.
    #[default]
    Info,
    /// Potential issue that may cause problems.
    ///
    /// Warnings indicate things that might be mistakes or could cause
    /// issues in certain circumstances, but aren't definite errors.
    Warning,
    /// Critical issue that will likely cause problems.
    ///
    /// Errors indicate definite problems that should be fixed, such as
    /// invalid references or malformed data that could crash the game.
    Error,
}

/// Per-file ignore state derived from the pack's `diagnostics_files_to_ignore` setting.
///
/// Tuple shape: `(ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields)`.
/// `None` means the whole file is skipped.
pub type FileIgnoreState = (Vec<String>, HashSet<String>, HashMap<String, Vec<String>>);

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Default for DiagnosticType {
    fn default() -> Self {
        Self::Pack(PackDiagnostic::default())
    }
}

impl DiagnosticType {
    pub fn path(&self) -> &str {
        match self {
            Self::AnimFragmentBattle(ref diag) => diag.path(),
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.path(),
            Self::Pack(_) => "",
            Self::PortraitSettings(diag) => diag.path(),
            Self::Text(diag) => diag.path(),
            Self::Dependency(diag) => diag.path(),
            Self::Config(_) => "",
        }
    }

    pub fn pack(&self) -> &str {
        match self {
            Self::AnimFragmentBattle(ref diag) => diag.pack(),
            Self::DB(ref diag) |
            Self::Loc(ref diag) => diag.pack(),
            Self::Pack(diag) => diag.pack(),
            Self::PortraitSettings(diag) => diag.pack(),
            Self::Text(diag) => diag.pack(),
            Self::Dependency(diag) => diag.pack(),
            Self::Config(_) => "",
        }
    }
}

impl Diagnostics {

    /// This function performs a search over the parts of the provided Packs, storing his results.
    #[allow(clippy::too_many_arguments)]
    pub fn check(&mut self, packs: &mut BTreeMap<String, Pack>, dependencies: &mut Dependencies, schema: &Schema, game_info: &GameInfo, game_path: &Path, paths_to_check: &[ContainerPath], check_ak_only_refs: bool) {

        // Clear the diagnostics first if we're doing a full check, or only the config ones and the ones for the path to update if we're doing a partial check.
        if paths_to_check.is_empty() {
            self.results.clear();
        } else {
            self.results.retain(|diagnostic| !paths_to_check.contains(&ContainerPath::File(diagnostic.path().to_string())));
            self.results.iter_mut().for_each(|x| {
                if let DiagnosticType::Config(config) = x {
                    config.results_mut().retain(|x|
                        match x.report_type() {
                            ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                            ConfigDiagnosticReportType::DependenciesCacheOutdated |
                            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_) |
                            ConfigDiagnosticReportType::IncorrectGamePath => false,
                        }
                    );
                }
            });
        }

        // First, check for config issues, as some of them may stop the checking prematurely.
        if let Some(diagnostics) = ConfigDiagnostic::check(dependencies, game_info, game_path) {
            let is_diagnostic_blocking = if let DiagnosticType::Config(ref diagnostic) = diagnostics {
                diagnostic.results().iter().any(|diagnostic| matches!(diagnostic.report_type(),
                    ConfigDiagnosticReportType::IncorrectGamePath |
                    ConfigDiagnosticReportType::DependenciesCacheNotGenerated |
                    ConfigDiagnosticReportType::DependenciesCacheOutdated |
                    ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(_)))
            } else { false };

            // If we have one of the blocking diagnostics, report it and return.
            self.results.push(diagnostics);
            if is_diagnostic_blocking {
                return;
            }
        }

        // TODO: Check if we should split this so each pack is only affected by their own ignored files.
        let files_to_ignore = packs.values().find_map(|pack| pack.settings().diagnostics_files_to_ignore());

        // To make sure we can read any non-db and non-loc file, we need to pre-decode them here.
        {
            // Extra data to decode animfragmentbattle files.
            let mut extra_data = DecodeableExtraData::default();
            extra_data.set_game_info(Some(game_info));
            let extra_data = Some(extra_data);

            for pack in packs.values_mut() {
                pack.files_by_type_mut(&[FileType::AnimFragmentBattle, FileType::Text, FileType::PortraitSettings])
                    .par_iter_mut()
                    .for_each(|file| { let _ = file.decode(&extra_data, true, false); });
            }
        }

        // Logic here: we want to process the tables on batches containing all the tables of the same type, so we can check duplicates in different tables.
        // To do that, we have to sort/split the file list, the process that.
        let files: Vec<(&str, &RFile)> = if paths_to_check.is_empty() {
            packs.iter().flat_map(|(key, pack)| pack.files_by_type(&[FileType::AnimFragmentBattle, FileType::DB, FileType::Loc, FileType::Text, FileType::PortraitSettings]).into_iter().map(move |file| (key.as_str(), file))).collect()
        } else {
            packs.iter().flat_map(|(key, pack)| pack.files_by_type_and_paths(&[FileType::AnimFragmentBattle, FileType::DB, FileType::Loc, FileType::Text, FileType::PortraitSettings], paths_to_check, false).into_iter().map(move |file| (key.as_str(), file))).collect()
        };

        let mut files_split: HashMap<&str, Vec<(&str, &RFile)>> = HashMap::new();
        let mut we_need_loc_data = false;
        for (pack_key, file) in &files {
            match file.file_type() {
                FileType::AnimFragmentBattle => {
                    if let Some(table_set) = files_split.get_mut("anim_fragment_battle") {
                        table_set.push((pack_key, file));
                    } else {
                        files_split.insert("anim_fragment_battle", vec![(pack_key, file)]);
                    }
                },
                FileType::DB => {
                    we_need_loc_data = true;

                    let path_split = file.path_in_container_split();
                    if path_split.len() > 2 {
                        if let Some(table_set) = files_split.get_mut(path_split[1]) {
                            table_set.push((pack_key, file));
                        } else {
                            files_split.insert(path_split[1], vec![(pack_key, file)]);
                        }
                    }
                },
                FileType::Loc => {
                    if let Some(table_set) = files_split.get_mut("locs") {
                        table_set.push((pack_key, file));
                    } else {
                        files_split.insert("locs", vec![(pack_key, file)]);
                    }
                },
                FileType::Text => {
                    if let Some(name) = file.file_name() {
                        if name.ends_with(".lua") {
                            if let Some(table_set) = files_split.get_mut("lua") {
                                table_set.push((pack_key, file));
                            } else {
                                files_split.insert("lua", vec![(pack_key, file)]);
                            }
                        }
                    }
                },
                FileType::PortraitSettings => {
                    if let Some(table_set) = files_split.get_mut("portrait_settings") {
                        table_set.push((pack_key, file));
                    } else {
                        files_split.insert("portrait_settings", vec![(pack_key, file)]);
                    }
                },
                _ => {},
            }
        }

        // Getting this here speeds up a lot path-checking later.
        let mut local_file_path_list = HashMap::new();
        for pack in packs.values() {
            local_file_path_list.extend(pack.paths_cache().iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        let local_file_path_list = &local_file_path_list;

        let loc_files: Vec<&RFile> = packs.values().flat_map(|pack| pack.files_by_type(&[FileType::Loc])).collect();
        let loc_decoded = loc_files.iter()
            .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
            .map(|file| file.data())
            .collect::<Vec<_>>();

        // Loc data takes a few ms to get, and it's only needed if we're going to check on tables, for the lookup data. So only get it if we really need it.
        let loc_data = if we_need_loc_data {
            Some(loc_decoded.par_iter()
            .flat_map(|data| data.par_iter()
                .map(|entry| (entry[0].data_to_string(), entry[1].data_to_string()))
                .collect::<Vec<(_,_)>>()
            ).collect::<HashMap<_,_>>())
        } else {
            None
        };

        // That way we can get it fast on the first try, and skip.
        let table_names = files_split.iter().filter(|(key, _)| **key != "anim_fragment_battle" && **key != "locs" && **key != "lua" && **key != "portrait_settings").map(|(key, _)| key.to_string()).collect::<Vec<_>>();

        // If table names is empty this triggers a full regeneration, which is slow as fuck. So make sure to avoid that if we're only doing a partial check.
        if !table_names.is_empty() || (table_names.is_empty() && paths_to_check.is_empty()) {
            dependencies.generate_local_db_references(schema, packs, &table_names);
        }

        // Caches for Portrait Settings diagnostics. There are some alt lookups for tables with differently named columns between games.
        let art_set_ids = dependencies.db_values_from_table_name_and_column_name(Some(packs), "campaign_character_arts_tables", "art_set_id", true, true);
        let mut variant_filenames = dependencies.db_values_from_table_name_and_column_name(Some(packs), "variants_tables", "variant_filename", true, true);
        if variant_filenames.is_empty() {
            variant_filenames = dependencies.db_values_from_table_name_and_column_name(Some(packs), "variants_tables", "variant_name", true, true);
        }

        // Process the files in batches.
        self.results.append(&mut files_split.par_iter().filter_map(|(_, files)| {
            let mut diagnostics = Vec::with_capacity(files.len());

            // Ignore empty groups, which should never happen, but just in case.
            if let Some(file_type) = files.first().map(|(_, x)| x.file_type()) {

                // DB groups are processed as a group, not per file, so we are able to detect duplicated lines between files.
                // Same for locs.
                match file_type {
                    FileType::DB => {
                        diagnostics.extend_from_slice(&TableDiagnostic::check_db(
                            files,
                            dependencies,
                            &self.diagnostics_ignored,
                            game_info,
                            local_file_path_list,
                            check_ak_only_refs,
                            &files_to_ignore,
                            packs,
                            schema,
                            &loc_data
                        ));
                    },
                    FileType::Loc => {
                        diagnostics.extend_from_slice(&TableDiagnostic::check_loc(
                            files,
                            &self.diagnostics_ignored,
                            &files_to_ignore,
                        ));
                    }
                    _ => {
                        for (pack_key, file) in files {
                            let (ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields) = Self::ignore_data_for_file(file, &files_to_ignore)?;

                            let diagnostic = match file.file_type() {
                                FileType::AnimFragmentBattle => AnimFragmentBattleDiagnostic::check(
                                    pack_key,
                                    file,
                                    dependencies,
                                    &self.diagnostics_ignored,
                                    &ignored_fields,
                                    &ignored_diagnostics,
                                    &ignored_diagnostics_for_fields,
                                    local_file_path_list,
                                ),

                                FileType::Text => TextDiagnostic::check(pack_key, file, packs, dependencies, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields),
                                FileType::PortraitSettings => PortraitSettingsDiagnostic::check(pack_key, file, &art_set_ids, &variant_filenames, dependencies, &self.diagnostics_ignored, &ignored_fields, &ignored_diagnostics, &ignored_diagnostics_for_fields, local_file_path_list),
                                _ => None,
                            };

                            if let Some(diagnostic) = diagnostic {
                                diagnostics.push(diagnostic);
                            }
                        }
                    }
                }
            }

            Some(diagnostics)
        }).flatten().collect());

        // These two are global, so do not execute on file-specific runs.
        if paths_to_check.is_empty() {
            self.results_mut().extend(DependencyDiagnostic::check(packs));
            self.results_mut().extend(PackDiagnostic::check(packs, dependencies, game_info));
        }

        self.results_mut().sort_by(|a, b| {
            if !a.path().is_empty() && !b.path().is_empty() {
                a.path().cmp(b.path())
            } else if a.path().is_empty() && !b.path().is_empty() {
                Ordering::Greater
            } else if !a.path().is_empty() && b.path().is_empty() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
    }

    /// Function to know if an specific field/diagnostic must be ignored.
    fn ignore_diagnostic(global_ignored_diagnostics: &[String], field_name: Option<&str>, diagnostic: Option<&str>, ignored_fields: &[String], ignored_diagnostics: &HashSet<String>, ignored_diagnostics_for_fields: &HashMap<String, Vec<String>>) -> bool {
        let mut ignore_diagnostic = false;

        if let Some(diagnostic) = diagnostic {
            return global_ignored_diagnostics.iter().any(|x| x == diagnostic);
        }

        // If we have a field, and it's in the ignored list, ignore it.
        if let Some(field_name) = field_name {
            ignore_diagnostic = ignored_fields.iter().any(|x| x == field_name);
        }

        // If we have a diagnostic, and it's in the ignored list, ignore it.
        else if let Some(diagnostic) = diagnostic {
            ignore_diagnostic = ignored_diagnostics.get(diagnostic).is_some();
        }

        // If we have not yet being ignored, check for specific diagnostics for specific fields.
        if !ignore_diagnostic {
            if let Some(field_name) = field_name {
                if let Some(diagnostic) = diagnostic {
                    if let Some(diags) = ignored_diagnostics_for_fields.get(field_name) {
                        ignore_diagnostic = diags.iter().any(|x| x == diagnostic);
                    }
                }
            }
        }

        ignore_diagnostic
    }

    /// Ignore entire tables if their path starts with the one we have (so we can do mass ignores) and we didn't specified a field to ignore.
    fn ignore_data_for_file(file: &RFile, files_to_ignore: &Option<Vec<DiagnosticIgnoreEntry>>) -> Option<FileIgnoreState> {
        let mut ignored_fields = vec![];
        let mut ignored_diagnostics = HashSet::new();
        let mut ignored_diagnostics_for_fields: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(ref files_to_ignore) = files_to_ignore {
            for (path_to_ignore, fields, diags_to_ignore) in files_to_ignore {

                // If the rule doesn't affect this PackedFile, ignore it.
                if !path_to_ignore.is_empty() && file.path_in_container_raw().starts_with(path_to_ignore) {

                    // If we don't have either fields or diags specified, we ignore the entire file.
                    if fields.is_empty() && diags_to_ignore.is_empty() {
                        return None;
                    }

                    // If we have both, fields and diags, disable only those diags for those fields.
                    if !fields.is_empty() && !diags_to_ignore.is_empty() {
                        for field in fields {
                            match ignored_diagnostics_for_fields.get_mut(field) {
                                Some(diagnostics) => diagnostics.append(&mut diags_to_ignore.to_vec()),
                                None => { ignored_diagnostics_for_fields.insert(field.to_owned(), diags_to_ignore.to_vec()); },
                            }
                        }
                    }

                    // Otherwise, check if we only have fields or diags, and put them separately.
                    else if !fields.is_empty() {
                        ignored_fields.append(&mut fields.to_vec());
                    }

                    else if !diags_to_ignore.is_empty() {
                        ignored_diagnostics.extend(diags_to_ignore.to_vec());
                    }
                }
            }
        }
        Some((ignored_fields, ignored_diagnostics, ignored_diagnostics_for_fields))
    }

    /// This function converts an entire diagnostics struct into a JSon string.
    pub fn json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(From::from)
    }
}

impl Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::AnimFragmentBattle(_) => "AnimFragmentBattle",
            Self::Config(_) => "Config",
            Self::DB(_) => "DB",
            Self::Loc(_) => "Loc",
            Self::Pack(_) => "Packfile",
            Self::PortraitSettings(_) => "PortraitSettings",
            Self::Text(_) => "Text",
            Self::Dependency(_) => "DependencyManager",
        }, f)
    }
}
