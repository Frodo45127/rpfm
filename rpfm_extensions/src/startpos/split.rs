//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Splitting of the `start_pos_*` tables by campaign, as the Assembly Kit's BOB does before building a startpos.
//!
//! The game builds a campaign's startpos from every row of the tables it gets, so tables holding the rows of several
//! campaigns make the build fail. BOB only exports the rows of the campaign it builds:
//!
//! - Rows of tables with a `campaign` column belong to their campaign.
//! - Other rows belong to the campaign if at least one of their references to a `start_pos_*` table points to a row
//!   of the campaign (a character of one of its factions, a link from one of its characters to a settlement of
//!   another campaign...). This runs until no more rows join.
//! - Rows of tables without such references belong to the campaign if one of its rows references them (deal actions
//!   of its diplomatic deals). Not seen in vanilla data, where the tables they're in are empty.
//!
//! Verified on Attila's main campaign: from the Assembly Kit's tables, the rows kept are exactly the ones BOB exports.

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::Result;
use rpfm_lib::files::db::DB;
use rpfm_lib::schema::Schema;

use super::tables::StartposTables;

#[cfg(test)] #[path = "split_test.rs"] mod split_test;

/// Prefix of the tables split by campaign.
const STARTPOS_TABLE_PREFIX: &str = "start_pos_";

/// Column of the tables that say which campaign their rows belong to.
const CAMPAIGN_COLUMN: &str = "campaign";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The rows of the `start_pos_*` tables that belong to a campaign.
#[derive(Debug, Clone, Default)]
pub struct CampaignRows {

    /// Campaign key.
    campaign: String,

    /// References of each table to other `start_pos_*` tables: (column, referenced table, referenced column).
    references: HashMap<String, Vec<(String, String, String)>>,

    /// Values of the referenced columns in the campaign's rows, by (table, column).
    values: HashMap<(String, String), HashSet<String>>,

    /// Values the campaign's rows reference in tables without references of their own, by (table, column).
    referenced: HashMap<(String, String), HashSet<String>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CampaignRows {

    /// Finds the rows of the `start_pos_*` tables that belong to a campaign.
    ///
    /// # Arguments
    ///
    /// * `tables` - Every row the build sees: the game's and the pack's.
    /// * `schema` - Schema with the references between the tables.
    /// * `campaign` - Campaign key.
    ///
    /// # Errors
    ///
    /// Returns an error if a table lacks a column its definition has.
    pub fn new(tables: &StartposTables, schema: &Schema, campaign: &str) -> Result<Self> {
        let mut rows = Self { campaign: campaign.to_owned(), references: references(schema), ..Default::default() };
        let referenced = rows.references.values().flatten().map(|(_, table, column)| (table.to_owned(), column.to_owned())).collect::<HashSet<_>>();

        // Rows join as the rows they reference do, so this repeats until none joins.
        loop {
            let mut values = HashMap::<(String, String), HashSet<String>>::new();
            for name in rows.references.keys() {
                let Ok(table) = tables.table(name) else { continue };
                for row in table.rows() {
                    let value = |column: &str| row.str(column).ok().map(|value| value.into_owned());
                    if !rows.belongs(name, value) {
                        continue;
                    }
                    for (table_name, column) in referenced.iter().filter(|(table_name, _)| table_name == name) {
                        if let Ok(value) = row.str(column) {
                            values.entry((table_name.to_owned(), column.to_owned())).or_default().insert(value.into_owned());
                        }
                    }
                }
            }

            let joined = values.values().map(HashSet::len).sum::<usize>() > rows.values.values().map(HashSet::len).sum::<usize>();
            rows.values = values;
            if !joined {
                break;
            }
        }

        rows.add_referenced_rows(tables)?;
        Ok(rows)
    }

    /// Returns if a row of a table belongs to the campaign.
    ///
    /// # Arguments
    ///
    /// * `table` - Table name, without the `_tables` suffix.
    /// * `value` - Returns the value of a column of the row, or `None` if the table has no such column.
    pub fn belongs(&self, table: &str, value: impl Fn(&str) -> Option<String>) -> bool {
        if let Some(campaign) = value(CAMPAIGN_COLUMN) {
            return campaign == self.campaign;
        }

        let references = self.references.get(table).map(Vec::as_slice).unwrap_or_default();
        if references.is_empty() {
            return self.referenced.iter()
                .filter(|((referenced_table, _), _)| referenced_table == table)
                .any(|((_, column), values)| value(column).is_some_and(|value| values.contains(&value)));
        }

        references.iter().any(|(column, target_table, target_column)| {
            value(column)
                .filter(|value| !value.is_empty())
                .is_some_and(|value| self.values.get(&(target_table.to_owned(), target_column.to_owned())).is_some_and(|values| values.contains(&value)))
        })
    }

    /// Removes the rows of a `start_pos_*` table that don't belong to the campaign.
    ///
    /// # Arguments
    ///
    /// * `db` - The table. Tables that aren't `start_pos_*` ones are left as they are.
    ///
    /// # Returns
    ///
    /// The number of rows removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the rows can't be set back.
    pub fn retain(&self, db: &mut DB) -> Result<usize> {
        let name = db.table_name_without_tables();
        if !name.starts_with(STARTPOS_TABLE_PREFIX) {
            return Ok(0);
        }

        let positions = db.definition().fields_processed().iter().enumerate()
            .map(|(position, field)| (field.name().to_owned(), position))
            .collect::<HashMap<_, _>>();
        let rows = db.data().to_vec();
        let kept = rows.iter()
            .filter(|row| self.belongs(&name, |column| positions.get(column).map(|position| row[*position].data_to_string().into_owned())))
            .cloned()
            .collect::<Vec<_>>();

        let removed = rows.len() - kept.len();
        if removed > 0 {
            db.set_data(&kept)?;
        }
        Ok(removed)
    }

    /// Adds the values of the rows of tables without references that the campaign's rows reference.
    fn add_referenced_rows(&mut self, tables: &StartposTables) -> Result<()> {
        let mut added = HashMap::<(String, String), HashSet<String>>::new();
        for (name, references) in &self.references {
            let Ok(table) = tables.table(name) else { continue };
            let targets = references.iter().filter(|(_, target, _)| self.references.get(target).is_none_or(Vec::is_empty)).collect::<Vec<_>>();
            if targets.is_empty() {
                continue;
            }
            for row in table.rows() {
                if !self.belongs(name, |column| row.str(column).ok().map(|value| value.into_owned())) {
                    continue;
                }
                for (column, target_table, target_column) in &targets {
                    let value = row.str(column)?;
                    if !value.is_empty() {
                        added.entry((target_table.to_owned(), target_column.to_owned())).or_default().insert(value.into_owned());
                    }
                }
            }
        }
        self.referenced = added;
        Ok(())
    }
}

/// Returns the references of every `start_pos_*` table of a schema to other `start_pos_*` tables, by table name
/// (without the `_tables` suffix), from the newest definition of each table.
fn references(schema: &Schema) -> HashMap<String, Vec<(String, String, String)>> {
    schema.definitions().iter()
        .filter(|(name, _)| name.starts_with(STARTPOS_TABLE_PREFIX))
        .filter_map(|(name, definitions)| Some((name, definitions.iter().max_by_key(|definition| *definition.version())?)))
        .map(|(name, definition)| {
            let references = definition.fields_processed().iter()
                .filter_map(|field| {
                    let (table, column) = field.is_reference(Some(definition.patches()))?;
                    table.starts_with(STARTPOS_TABLE_PREFIX).then(|| (field.name().to_owned(), table, column))
                })
                .collect();
            (name.trim_end_matches("_tables").to_owned(), references)
        })
        .collect()
}
