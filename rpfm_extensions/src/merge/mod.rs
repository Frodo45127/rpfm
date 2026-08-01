//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Delta merge: combine same-named DB/Loc tables by row identity instead of blindly
//! concatenating every row.
//!
//! The plain [`DB::merge`](rpfm_lib::files::db::DB::merge)/[`Loc::merge`](rpfm_lib::files::loc::Loc::merge)
//! append every source row verbatim, so two mods editing the same rows produce duplicates instead of a
//! combined result. This module instead matches rows by their schema key column(s)
//! ([`Definition::key_column_positions`]), and for a key shared by more than one source resolves each
//! field individually:
//!
//! - All contributing sources agree on the value: keep it.
//! - They disagree, but only one of them actually diverges from the vanilla/parent baseline: keep the
//!   one that diverges (that source is the only one that touched this field).
//! - Otherwise it's a genuine conflict (including a row that's brand new, with different data, in more
//!   than one source, since there's no baseline to break the tie): reported as a [`MergeConflict`]
//!   unless the caller already supplied a [`MergeResolution`] for it.
//!
//! Tables with no key column can't be matched by row identity at all, so delta merge degrades to plain
//! concatenation for them.

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{db::DB, loc::Loc, RFileDecoded, table::DecodedData};
use rpfm_lib::schema::{Definition, DefinitionPatch};

use crate::dependencies::Dependencies;

#[cfg(test)] mod tests;

//-------------------------------------------------------------------------------//
//                             Options & result types
//-------------------------------------------------------------------------------//

/// Configuration for a table merge.
#[derive(Clone, Debug, Default, Getters, Setters, Deserialize, Serialize)]
#[getset(get = "pub", set = "pub")]
pub struct MergeOptions {

    /// Merge rows by key instead of concatenating them. See the module docs for the resolution rules.
    delta_merge: bool,

    /// Resolutions for conflicts reported by a previous attempt at the same merge.
    resolutions: Vec<MergeResolution>,
}

/// A single field that two or more sources changed in incompatible ways, needing a user decision.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MergeConflict {

    /// Stringified value of each key column, identifying the row this conflict belongs to.
    pub row_key: Vec<String>,

    /// Name of the conflicting column.
    pub field_name: String,

    /// The distinct values proposed for this field, each tagged with the source(s) that proposed it.
    pub candidates: Vec<MergeCandidate>,
}

/// One of the candidate values for a [`MergeConflict`].
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MergeCandidate {

    /// Stringified candidate value.
    pub value: String,

    /// Source table paths that proposed this value.
    pub source_paths: Vec<String>,
}

/// The user's chosen value for one [`MergeConflict`], fed back into a follow-up merge attempt.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MergeResolution {

    /// Must match [`MergeConflict::row_key`] exactly.
    pub row_key: Vec<String>,

    /// Must match [`MergeConflict::field_name`] exactly.
    pub field_name: String,

    /// The candidate value the user picked.
    pub chosen_value: String,
}

//-------------------------------------------------------------------------------//
//                             Public entry points
//-------------------------------------------------------------------------------//

/// Delta-merges multiple DB tables of the same name into one, using `baseline` (the vanilla/parent
/// version of the table, if any) to tell an intentional edit apart from an untouched row.
///
/// Every source is normalized to the first source's definition first, exactly like the plain
/// [`DB::merge`]. Returns the merged table plus any conflicts that need a [`MergeResolution`] before
/// they can be applied; unresolved conflicting fields are left at their baseline (or first-candidate,
/// if there's no baseline) value in the returned table.
pub fn delta_merge_db(sources: &[(&str, &DB)], baseline: Option<&DB>, resolutions: &[MergeResolution]) -> Result<(DB, Vec<MergeConflict>)> {
    if sources.len() < 2 {
        return Err(RLibError::RFileMergeTablesNotEnoughTablesProvided);
    }

    let table_names = sources.iter().map(|(_, db)| db.table_name()).collect::<HashSet<_>>();
    if table_names.len() > 1 {
        return Err(RLibError::RFileMergeTablesDifferentNames);
    }

    let definition = sources[0].1.definition().clone();
    let patches = sources[0].1.patches().clone();
    let table_name = sources[0].1.table_name().to_owned();

    let source_data = sources.iter()
        .map(|(path, db)| {
            let mut db = (*db).clone();
            db.set_definition(&definition);
            (*path, db.data().to_vec())
        })
        .collect::<Vec<_>>();
    let source_rows = source_data.iter().map(|(path, rows)| (*path, rows.as_slice())).collect::<Vec<_>>();

    let baseline_data = baseline.map(|db| {
        let mut db = db.clone();
        db.set_definition(&definition);
        db.data().to_vec()
    });

    let (rows, conflicts) = delta_merge_rows(&source_rows, baseline_data.as_deref(), &definition, Some(&patches), resolutions);

    let mut merged = DB::new(&definition, Some(&patches), &table_name);
    merged.set_data(&rows)?;

    Ok((merged, conflicts))
}

/// Delta-merges multiple Loc tables into one. See [`delta_merge_db`] for the general behavior; Loc
/// tables always key on their single `key` column.
pub fn delta_merge_loc(sources: &[(&str, &Loc)], baseline: Option<&Loc>, resolutions: &[MergeResolution]) -> Result<(Loc, Vec<MergeConflict>)> {
    if sources.len() < 2 {
        return Err(RLibError::RFileMergeTablesNotEnoughTablesProvided);
    }

    let definition = sources[0].1.definition().clone();

    let source_data = sources.iter()
        .map(|(path, loc)| {
            let mut loc = (*loc).clone();
            loc.set_definition(&definition);
            (*path, loc.data().to_vec())
        })
        .collect::<Vec<_>>();
    let source_rows = source_data.iter().map(|(path, rows)| (*path, rows.as_slice())).collect::<Vec<_>>();

    let baseline_data = baseline.map(|loc| {
        let mut loc = loc.clone();
        loc.set_definition(&definition);
        loc.data().to_vec()
    });

    let (rows, conflicts) = delta_merge_rows(&source_rows, baseline_data.as_deref(), &definition, None, resolutions);

    let mut merged = Loc::new();
    merged.set_definition(&definition);
    merged.set_data(&rows)?;

    Ok((merged, conflicts))
}

/// Flattens the vanilla/parent versions of a DB table (in load order) into a single baseline table,
/// for use as the `baseline` argument of [`delta_merge_db`]. Returns `None` if the table isn't loaded.
pub fn db_baseline(dependencies: &Dependencies, table_name: &str) -> Option<DB> {
    let files = dependencies.db_data(table_name, true, true).ok()?;

    let mut baseline: Option<DB> = None;
    for file in files {
        if let Ok(RFileDecoded::DB(table)) = file.decoded() {
            match baseline {
                None => baseline = Some(table.clone()),
                Some(ref mut merged) => merged.data_mut().extend(table.data().iter().cloned()),
            }
        }
    }

    baseline
}

/// Flattens the vanilla/parent Loc tables (in load order) into a single baseline table, for use as the
/// `baseline` argument of [`delta_merge_loc`]. Returns `None` if no Loc data is loaded.
pub fn loc_baseline(dependencies: &Dependencies) -> Option<Loc> {
    let files = dependencies.loc_data(true, true).ok()?;

    let mut baseline: Option<Loc> = None;
    for file in files {
        if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
            match baseline {
                None => baseline = Some(table.clone()),
                Some(ref mut merged) => merged.data_mut().extend(table.data().iter().cloned()),
            }
        }
    }

    baseline
}

//-------------------------------------------------------------------------------//
//                             Core algorithm
//-------------------------------------------------------------------------------//

/// Generic row-diff/merge core shared by [`delta_merge_db`] and [`delta_merge_loc`]. Operates purely on
/// rows and a [`Definition`], so it doesn't care whether it's merging DB or Loc data.
///
/// All of `sources`, `baseline`, and the rows they contain are expected to already share `definition`'s
/// column layout (the two public wrappers normalize sources to a common definition before calling this).
fn delta_merge_rows(
    sources: &[(&str, &[Vec<DecodedData>])],
    baseline: Option<&[Vec<DecodedData>]>,
    definition: &Definition,
    patches: Option<&DefinitionPatch>,
    resolutions: &[MergeResolution],
) -> (Vec<Vec<DecodedData>>, Vec<MergeConflict>) {
    let fields = definition.fields_processed();
    let key_positions = fields.iter().enumerate().filter(|(_, field)| field.is_key(patches)).map(|(position, _)| position).collect::<Vec<_>>();

    // No key columns to match rows by: fall back to the same behavior as the plain merge.
    if key_positions.is_empty() {
        let rows = sources.iter().flat_map(|(_, rows)| rows.iter().cloned()).collect();
        return (rows, vec![]);
    }

    let row_key = |row: &[DecodedData]| key_positions.iter().map(|&position| row[position].clone()).collect::<Vec<DecodedData>>();
    let row_key_strings = |row: &[DecodedData]| key_positions.iter().map(|&position| row[position].data_to_string().to_string()).collect::<Vec<String>>();

    let baseline_index = baseline.map(|rows| rows.iter().map(|row| (row_key(row), row)).collect::<HashMap<_, _>>()).unwrap_or_default();

    // Group rows by key, preserving the order keys are first seen in across all sources.
    let mut key_order = Vec::new();
    let mut rows_by_key: HashMap<Vec<DecodedData>, Vec<(&str, &Vec<DecodedData>)>> = HashMap::new();
    for (source_path, rows) in sources {
        for row in rows.iter() {
            let key = row_key(row);
            if !rows_by_key.contains_key(&key) {
                key_order.push(key.clone());
            }
            rows_by_key.entry(key).or_default().push((source_path, row));
        }
    }

    let mut merged_rows = Vec::with_capacity(key_order.len());
    let mut conflicts = Vec::new();

    for key in key_order {
        let candidates = &rows_by_key[&key];

        // Only one source touched this key: nothing to reconcile, take it as-is.
        if let [(_, row)] = candidates[..] {
            merged_rows.push(row.clone());
            continue;
        }

        let baseline_row = baseline_index.get(&key).copied();
        let key_strings = row_key_strings(candidates[0].1);
        let mut merged_row = baseline_row.cloned().unwrap_or_else(|| candidates[0].1.clone());

        for (column, field) in fields.iter().enumerate() {
            let mut distinct_values: Vec<(DecodedData, Vec<String>)> = Vec::new();
            for (source_path, row) in candidates {
                let value = &row[column];
                match distinct_values.iter_mut().find(|(existing, _)| existing == value) {
                    Some((_, source_paths)) => source_paths.push((*source_path).to_owned()),
                    None => distinct_values.push((value.clone(), vec![(*source_path).to_owned()])),
                }
            }

            // Every source agrees: use that value, whether or not it matches the baseline.
            if let [(value, _)] = &distinct_values[..] {
                merged_row[column] = value.clone();
                continue;
            }

            // They disagree, but only one source actually diverges from the baseline: that's the edit.
            if let Some(baseline_row) = baseline_row {
                let mut diverging = distinct_values.iter().filter(|(value, _)| value != &baseline_row[column]);
                if let (Some((value, _)), None) = (diverging.next(), diverging.next()) {
                    merged_row[column] = value.clone();
                    continue;
                }
            }

            let resolution = resolutions.iter().find(|resolution| resolution.row_key == key_strings && resolution.field_name == field.name());
            match resolution.and_then(|resolution| DecodedData::new_from_type_and_string(field.field_type(), &resolution.chosen_value).ok()) {
                Some(value) => merged_row[column] = value,
                None => conflicts.push(MergeConflict {
                    row_key: key_strings.clone(),
                    field_name: field.name().to_owned(),
                    candidates: distinct_values.into_iter().map(|(value, source_paths)| MergeCandidate { value: value.data_to_string().to_string(), source_paths }).collect(),
                }),
            }
        }

        merged_rows.push(merged_row);
    }

    (merged_rows, conflicts)
}
