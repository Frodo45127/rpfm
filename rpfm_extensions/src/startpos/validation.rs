//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Validation of the tables a startpos is built from, to report why a build would fail before running it.
//!
//! When the game builds a startpos from broken data it either quits without a startpos, hangs forever, or silently
//! drops rows, and logs nothing. This module finds those problems up front:
//!
//! - Every reference of the `start_pos_*` tables (as the schema defines them) must point to an existing row.
//!   Broken references in [`FATAL_REFERENCES`] make the game quit or hang, so they're errors; the rest make it drop
//!   the row or ignore the value, so they're warnings.
//! - The campaign must exist.
//! - Characters in the generals pool can't also be placed on the map with an army.
//!
//! The outcomes of broken references were measured on Attila, one broken reference at a time.

use std::collections::{HashMap, HashSet};
use std::fmt;

use rpfm_lib::schema::{Definition, Schema};

use super::tables::{Row, StartposTables};

#[cfg(test)] #[path = "validation_test.rs"] mod validation_test;

/// Columns (table without the `_tables` suffix, column) whose broken references make the game quit or hang while
/// building the startpos. `Surname`, `clan_name` and `other_name` are assumed to behave like `Name`.
pub const FATAL_REFERENCES: [(&str, &str); 22] = [
    ("start_pos_characters", "faction"),
    ("start_pos_characters", "Type"),
    ("start_pos_characters", "Name"),
    ("start_pos_characters", "Surname"),
    ("start_pos_characters", "clan_name"),
    ("start_pos_characters", "other_name"),
    ("start_pos_characters", "political_party"),
    ("start_pos_characters", "override_general_unit"),
    ("start_pos_characters", "death_type"),
    ("start_pos_character_traits", "trait_level"),
    ("start_pos_land_units", "unit_type"),
    ("start_pos_naval_units", "unit_type"),
    ("start_pos_horde_details", "primary_building"),
    ("start_pos_factions", "faction"),
    ("start_pos_factions", "government_type"),
    ("start_pos_factions", "state_religion"),
    ("start_pos_factions", "cai_personality_group"),
    ("start_pos_factions", "cdir_military_generator_config"),
    ("start_pos_regions", "region"),
    ("start_pos_regions", "cultural_originator"),
    ("start_pos_settlements", "region"),
    ("start_pos_settlements", "primary_building"),
];

/// Prefix of the tables whose references are checked.
const STARTPOS_TABLE_PREFIX: &str = "start_pos_";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// How bad an issue is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {

    /// The build fails or hangs.
    Error,

    /// The build works, but drops or ignores data.
    Warning,
}

/// A problem in the data a startpos is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartposIssue {

    /// How bad it is.
    pub severity: Severity,

    /// Table, without the `_tables` suffix.
    pub table: String,

    /// Key values of the row, joined by `|`, or the row's position if the table has no keys.
    pub row: String,

    /// Column with the problem, if it's about one.
    pub column: Option<String>,

    /// What's wrong.
    pub message: String,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl fmt::Display for StartposIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
        };
        match &self.column {
            Some(column) => write!(f, "{severity}: {}, row {}, column {column}: {}", self.table, self.row, self.message),
            None => write!(f, "{severity}: {}, row {}: {}", self.table, self.row, self.message),
        }
    }
}

/// Returns the tables the `start_pos_*` tables of a schema reference, without the `_tables` suffix.
pub fn referenced_tables(schema: &Schema) -> HashSet<String> {
    schema.definitions().iter()
        .filter(|(name, _)| name.starts_with(STARTPOS_TABLE_PREFIX))
        .filter_map(|(_, definitions)| latest(definitions))
        .flat_map(|definition| definition.fields_processed().iter().filter_map(|field| field.is_reference(Some(definition.patches()))))
        .map(|(table, _)| table)
        .collect()
}

/// Validates the tables a campaign's startpos is built from.
///
/// # Arguments
///
/// * `tables` - The `start_pos_*` tables, and the tables they reference. References to tables that aren't there
///   aren't checked.
/// * `schema` - Schema with the references of the tables.
/// * `campaign` - Campaign key.
///
/// # Returns
///
/// The issues found, errors first.
pub fn validate(tables: &StartposTables, schema: &Schema, campaign: &str) -> Vec<StartposIssue> {
    let mut issues = vec![];
    validate_campaign(tables, campaign, &mut issues);
    validate_references(tables, schema, &mut issues);
    validate_generals_pool(tables, &mut issues);
    issues.sort_by_key(|issue| issue.severity);
    issues
}

/// Checks the campaign is in the `campaigns` table.
fn validate_campaign(tables: &StartposTables, campaign: &str, issues: &mut Vec<StartposIssue>) {
    let exists = tables.table("campaigns").is_ok_and(|table| table.rows().any(|row| row.str("campaign_name").is_ok_and(|name| name == campaign)));
    if !exists {
        issues.push(StartposIssue {
            severity: Severity::Error,
            table: "campaigns".to_owned(),
            row: campaign.to_owned(),
            column: None,
            message: format!("the campaign {campaign} isn't in the table, so the game skips the build."),
        });
    }
}

/// Checks the references of every `start_pos_*` table.
fn validate_references(tables: &StartposTables, schema: &Schema, issues: &mut Vec<StartposIssue>) {
    let mut values = HashMap::<(String, String), Option<HashSet<String>>>::new();
    let mut names = schema.definitions().keys().filter(|name| name.starts_with(STARTPOS_TABLE_PREFIX)).collect::<Vec<_>>();
    names.sort();

    for name in names {
        let table_name = name.trim_end_matches("_tables");
        let (Ok(table), Some(definition)) = (tables.table(table_name), latest(&schema.definitions()[name])) else { continue };
        let fields = definition.fields_processed();
        let keys = fields.iter().filter(|field| field.is_key(Some(definition.patches()))).map(|field| field.name().to_owned()).collect::<Vec<_>>();

        for field in fields.iter() {
            let Some((target_table, target_column)) = field.is_reference(Some(definition.patches())) else { continue };
            if table.column(field.name()).is_err() {
                continue;
            }
            let targets = values.entry((target_table.to_owned(), target_column.to_owned()))
                .or_insert_with(|| column_values(tables, &target_table, &target_column));
            let Some(targets) = targets else { continue };

            let fatal = FATAL_REFERENCES.contains(&(table_name, field.name()));
            for (position, row) in table.rows().enumerate() {
                let Ok(value) = row.str(field.name()) else { continue };
                if value.is_empty() || targets.contains(value.as_ref()) {
                    continue;
                }

                issues.push(StartposIssue {
                    severity: if fatal { Severity::Error } else { Severity::Warning },
                    table: table_name.to_owned(),
                    row: row_name(&row, &keys, position),
                    column: Some(field.name().to_owned()),
                    message: format!("{value} isn't in {target_table}.{target_column}. {}", if fatal {
                        "The game quits or hangs while building the startpos."
                    } else {
                        "The game drops the row or ignores the value."
                    }),
                });
            }
        }
    }
}

/// Checks no character in the generals pool is also placed on the map with an army.
fn validate_generals_pool(tables: &StartposTables, issues: &mut Vec<StartposIssue>) {
    let Ok(characters) = tables.table("start_pos_characters") else { return };
    let placed = column_values(tables, "start_pos_character_to_settlements", "character").unwrap_or_default();
    let generals = column_values(tables, "start_pos_land_units", "general").unwrap_or_default();

    for (position, row) in characters.rows().enumerate() {
        let (Ok(id), Ok(true)) = (row.str("ID"), row.bool("is_in_generals_pool")) else { continue };
        if placed.contains(id.as_ref()) && generals.contains(id.as_ref()) {
            issues.push(StartposIssue {
                severity: Severity::Error,
                table: "start_pos_characters".to_owned(),
                row: row_name(&row, &["ID".to_owned()], position),
                column: Some("is_in_generals_pool".to_owned()),
                message: "the character is in the generals pool, but also placed on the map with an army. The game quits while building the startpos.".to_owned(),
            });
        }
    }
}

/// Returns the values of a column, or `None` if the table or column isn't there.
fn column_values(tables: &StartposTables, table: &str, column: &str) -> Option<HashSet<String>> {
    let table = tables.table(table).ok()?;
    table.column(column).ok()?;
    Some(table.rows().filter_map(|row| row.str(column).ok().map(|value| value.into_owned())).collect())
}

/// Returns the name of a row: its key values joined by `|`, or its position if it has no keys.
fn row_name(row: &Row, keys: &[String], position: usize) -> String {
    let values = keys.iter().filter_map(|key| row.str(key).ok().map(|value| value.into_owned())).collect::<Vec<_>>();
    if values.is_empty() { format!("#{}", position + 1) } else { values.join("|") }
}

/// Returns the latest definition of a table.
fn latest(definitions: &[Definition]) -> Option<&Definition> {
    definitions.iter().max_by_key(|definition| *definition.version())
}
