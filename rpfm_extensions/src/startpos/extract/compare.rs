//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Comparison of the tables extracted from a startpos with the tables it was built from.
//!
//! Every value the extraction gets back differently is something the build lost or changed on its way into the
//! startpos. Rows are matched by what they describe, not by their ids (which the extraction renumbers): ids that
//! reference factions, regions, settlements and characters are replaced by their keys first. Characters are told
//! apart by faction, names and agent type.
//!
//! Only the columns the extraction fills are compared.

use getset::Getters;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

use rpfm_lib::error::Result;
use rpfm_lib::schema::Schema;

use super::ExtractedTables;
use crate::startpos::tables::{Row, StartposTables, Table};
use crate::startpos::world::writer::HORDE_BUILDING_COLUMNS;

#[cfg(test)] #[path = "compare_test.rs"] mod compare_test;

/// Most differences kept as examples per column.
const MAX_EXAMPLES: usize = 5;

/// Relative difference under which two numbers count as equal.
const NUMBER_TOLERANCE: f64 = 1e-3;

/// Stances of `start_pos_diplomacy` that have a direction (the subject first).
const DIRECTED_STANCES: [&str; 2] = ["vassal", "client_state"];

/// Relationship of `start_pos_family_relationships` without direction.
const SPOUSE_RELATIONSHIP: &str = "spouse";

/// How each table is compared.
const TABLE_SPECS: &[TableSpec] = &[
    TableSpec { name: "start_pos_calendars", keys: &["campaign"], refs: &[], renumbered: &[] },
    TableSpec { name: "start_pos_factions", keys: &["faction"], refs: &[], renumbered: &[] },
    TableSpec { name: "start_pos_technologies", keys: &["faction", "technology"], refs: &[("faction", Reference::Faction)], renumbered: &[] },
    TableSpec { name: "start_pos_victory_conditions", keys: &["start_pos_faction", "region", "victory_type"], refs: &[("start_pos_faction", Reference::Faction)], renumbered: &[] },
    TableSpec { name: "start_pos_diplomacy", keys: &["faction1", "faction2"], refs: &[("faction1", Reference::Faction), ("faction2", Reference::Faction)], renumbered: &["key"] },
    TableSpec { name: "start_pos_regions", keys: &["region"], refs: &[("owning_faction", Reference::OptionalFaction)], renumbered: &[] },
    TableSpec { name: "start_pos_settlements", keys: &["region"], refs: &[("region", Reference::Region)], renumbered: &["id"] },
    TableSpec { name: "start_pos_region_religions", keys: &["region", "religion"], refs: &[("region", Reference::Region)], renumbered: &[] },
    TableSpec { name: "start_pos_region_slot_templates", keys: &["region", "slot_type"], refs: &[("region", Reference::RegionKey)], renumbered: &["id"] },
    TableSpec { name: "start_pos_regions_to_unit_resources", keys: &["key", "resource"], refs: &[("key", Reference::Region)], renumbered: &[] },
    TableSpec { name: "start_pos_characters", keys: &["ID"], refs: &[("ID", Reference::Character), ("faction", Reference::Faction)], renumbered: &[] },
    TableSpec { name: "start_pos_character_traits", keys: &["character_id", "trait_level"], refs: &[("character_id", Reference::Character)], renumbered: &["id"] },
    TableSpec { name: "start_pos_character_to_settlements", keys: &["character", "settlement"], refs: &[("character", Reference::Character), ("settlement", Reference::Settlement)], renumbered: &[] },
    TableSpec { name: "start_pos_family_relationships", keys: &["character", "related_to", "relationship"], refs: &[("character", Reference::Character), ("related_to", Reference::Character)], renumbered: &[] },
    TableSpec { name: "start_pos_land_units", keys: &["general", "unit_type"], refs: &[("general", Reference::Character)], renumbered: &["id"] },
    TableSpec { name: "start_pos_naval_units", keys: &["admiral", "unit_type"], refs: &[("admiral", Reference::Character)], renumbered: &["ID"] },
    TableSpec { name: "start_pos_horde_details", keys: &["general"], refs: &[("general", Reference::Character)], renumbered: &[] },
];

/// Columns of `start_pos_regions` the build sums over a province (fertility) or a faction's part of a province.
const PROVINCE_SUMS: [&str; 1] = ["base_fertility"];
const PROVINCE_PART_SUMS: [&str; 2] = ["development_points", "initial_public_order"];

/// Highest public order of a faction's part of a province: the build clamps the sums to it, both ways.
const MAX_PUBLIC_ORDER: i64 = 100;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// How a table is compared.
struct TableSpec {

    /// Table name, without the `_tables` suffix.
    name: &'static str,

    /// Columns that identify a row, after resolving references. Rows with the same values are told apart by order.
    keys: &'static [&'static str],

    /// Columns with ids of other tables' rows, replaced by keys before comparing.
    refs: &'static [(&'static str, Reference)],

    /// Row id columns the extraction numbers anew, left out of the comparison.
    renumbered: &'static [&'static str],
}

/// The kind of row an id column references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reference {

    /// A `start_pos_factions` row of the campaign. Rows referencing other factions aren't the campaign's.
    Faction,

    /// A `start_pos_factions` row of the campaign, or anything else for none (like rebel-owned regions).
    OptionalFaction,

    /// A `start_pos_regions` row of the campaign.
    Region,

    /// A region key of the campaign (not an id).
    RegionKey,

    /// A `start_pos_settlements` row of the campaign's regions.
    Settlement,

    /// A `start_pos_characters` row of the campaign's factions.
    Character,
}

/// The keys of the rows of one table set that other rows reference, by row id.
#[derive(Debug, Default)]
struct References {
    factions: HashMap<String, String>,
    regions: HashMap<String, String>,
    region_keys: HashSet<String>,
    settlements: HashMap<String, String>,
    characters: HashMap<String, String>,
}

/// How the rows of one table of the extraction compare with the original ones.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct TableComparison {

    /// Table name, without the `_tables` suffix.
    table: String,

    /// Rows found in both.
    matched_rows: usize,

    /// Keys of the original rows the extraction lacks.
    missing_rows: Vec<String>,

    /// Keys of the extracted rows the original tables lack.
    extra_rows: Vec<String>,

    /// How each compared column matches, in table order.
    columns: Vec<ColumnComparison>,
}

/// How the values of one column compare in the matched rows.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct ColumnComparison {

    /// Column name.
    column: String,

    /// Matched rows with the same value.
    equal: usize,

    /// Matched rows with a different value.
    different: usize,

    /// Some of the differences: the row's key, the original value and the extracted one.
    examples: Vec<(String, String, String)>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Compares the tables extracted from a startpos with the tables it was built from.
///
/// # Arguments
///
/// * `original` - The tables the startpos was built from. Rows of other campaigns are left out.
/// * `extracted` - The tables extracted from the startpos.
/// * `schema` - Schema of the game, to type the extracted values.
///
/// # Returns
///
/// The comparison of each extracted table.
///
/// # Errors
///
/// Returns an error if the extracted values don't fit the schema, or a table lacks a column.
pub fn compare_tables(original: &StartposTables, extracted: &ExtractedTables, schema: &Schema) -> Result<Vec<TableComparison>> {
    let campaign = extracted.campaign();
    let extracted_tables = extracted.to_tables(schema)?;
    let original_references = References::new(original, campaign)?;
    let extracted_references = References::new(&extracted_tables, campaign)?;

    let mut comparisons = vec![];
    for spec in TABLE_SPECS {
        let Some(rows) = extracted.table(spec.name) else { continue };
        let columns = rows.columns().iter().copied().filter(|column| !spec.keys.contains(column) && !spec.renumbered.contains(column)).collect::<Vec<_>>();
        let mut ours = canonical_rows(original.table(spec.name).ok(), spec, &original_references, campaign)?;
        let mut theirs = canonical_rows(extracted_tables.table(spec.name).ok(), spec, &extracted_references, campaign)?;
        if spec.name == "start_pos_regions" {
            let provinces = region_provinces(original)?;
            sum_by_province(&mut ours, &provinces);
            sum_by_province(&mut theirs, &provinces);
        }

        let mut comparison = TableComparison {
            table: spec.name.to_owned(),
            columns: columns.iter().map(|column| ColumnComparison { column: column.to_string(), ..Default::default() }).collect(),
            ..Default::default()
        };
        for (key, original_row) in &ours {
            let Some(extracted_row) = theirs.get(key) else {
                comparison.missing_rows.push(key.to_owned());
                continue;
            };
            comparison.matched_rows += 1;
            for column in &mut comparison.columns {
                let original_value = original_row.get(&column.column).cloned().unwrap_or_default();
                let extracted_value = extracted_row.get(&column.column).cloned().unwrap_or_default();
                if same_value(&original_value, &extracted_value) {
                    column.equal += 1;
                } else {
                    column.different += 1;
                    if column.examples.len() < MAX_EXAMPLES {
                        column.examples.push((key.to_owned(), original_value, extracted_value));
                    }
                }
            }
        }
        comparison.extra_rows = theirs.keys().filter(|key| !ours.contains_key(*key)).cloned().collect();
        comparisons.push(comparison);
    }
    Ok(comparisons)
}

impl TableComparison {

    /// Returns the share of the original values the extraction got back: values of matched rows that are equal, over
    /// the values of every original row.
    pub fn match_ratio(&self) -> f64 {
        let equal = self.columns.iter().map(|column| column.equal).sum::<usize>() + self.matched_rows;
        let total = (self.matched_rows + self.missing_rows.len()) * (self.columns.len() + 1);
        if total == 0 { 1.0 } else { equal as f64 / total as f64 }
    }
}

impl fmt::Display for TableComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {} rows matched, {} missing, {} extra ({:.4} of the values)", self.table, self.matched_rows, self.missing_rows.len(), self.extra_rows.len(), self.match_ratio())?;
        for key in self.missing_rows.iter().take(MAX_EXAMPLES) {
            writeln!(f, "    missing row {key}")?;
        }
        for key in self.extra_rows.iter().take(MAX_EXAMPLES) {
            writeln!(f, "    extra row {key}")?;
        }
        for column in self.columns.iter().filter(|column| column.different > 0) {
            writeln!(f, "    {}: {} equal, {} different", column.column, column.equal, column.different)?;
            for (key, original, extracted) in &column.examples {
                writeln!(f, "        {key}: {original:?} -> {extracted:?}")?;
            }
        }
        Ok(())
    }
}

impl References {

    /// Collects the keys of the campaign's factions, regions, settlements and characters in a table set.
    fn new(tables: &StartposTables, campaign: &str) -> Result<Self> {
        let mut references = Self::default();
        if let Ok(factions) = tables.table("start_pos_factions") {
            for row in factions.rows().filter(|row| in_campaign(row, campaign)) {
                references.factions.insert(row.str("ID")?.into_owned(), row.str("faction")?.into_owned());
            }
        }
        if let Ok(regions) = tables.table("start_pos_regions") {
            for row in regions.rows().filter(|row| in_campaign(row, campaign)) {
                references.regions.insert(row.str("id")?.into_owned(), row.str("region")?.into_owned());
                references.region_keys.insert(row.str("region")?.into_owned());
            }
        }
        if let Ok(settlements) = tables.table("start_pos_settlements") {
            for row in settlements.rows() {
                if let Some(region) = references.regions.get(row.str("region")?.as_ref()) {
                    references.settlements.insert(row.str("id")?.into_owned(), region.to_owned());
                }
            }
        }

        // Characters with the same faction, first name, age and agent type are told apart by their order. Other
        // names aren't part of it, as the game drops the ones that don't fit the faction.
        if let Ok(characters) = tables.table("start_pos_characters") {
            let mut seen = HashMap::<String, usize>::new();
            for row in characters.rows() {
                let Some(faction) = references.factions.get(row.str("faction")?.as_ref()) else { continue };
                let signature = format!("{faction}/{}/{}/{}", row.str("Name")?, row.str("Age")?, row.str("Type")?);
                let occurrence = seen.entry(signature.to_owned()).or_default();
                *occurrence += 1;
                references.characters.insert(row.str("ID")?.into_owned(), format!("{signature}#{occurrence}"));
            }
        }
        Ok(references)
    }

    /// Returns the key of a referenced row, if it's one of the campaign's.
    fn resolve(&self, reference: Reference, id: &str) -> Option<String> {
        let map = match reference {
            Reference::Faction | Reference::OptionalFaction => &self.factions,
            Reference::Region => &self.regions,
            Reference::RegionKey => return self.region_keys.get(id).map(|_| id.to_owned()),
            Reference::Settlement => &self.settlements,
            Reference::Character => &self.characters,
        };
        map.get(id).cloned()
    }
}

/// Returns the campaign's rows of a table, with their references resolved, by key.
fn canonical_rows(table: Option<&Table>, spec: &TableSpec, references: &References, campaign: &str) -> Result<BTreeMap<String, HashMap<String, String>>> {
    let mut rows = BTreeMap::new();
    let Some(table) = table else { return Ok(rows) };
    let has_campaign = table.column("campaign").is_ok();
    let mut seen = HashMap::<String, usize>::new();

    'rows: for row in table.rows() {
        if has_campaign && !in_campaign(&row, campaign) {
            continue;
        }

        let mut values = HashMap::new();
        for column in table.column_names() {
            values.insert(column.to_owned(), row.str(column)?.into_owned());
        }
        for (column, reference) in spec.refs {
            let id = values.get(*column).cloned().unwrap_or_default();
            match (references.resolve(*reference, &id), reference) {
                (Some(key), _) => { values.insert(column.to_string(), key); },
                (None, Reference::OptionalFaction) => { values.insert(column.to_string(), String::new()); },
                (None, _) => continue 'rows,
            }
        }
        normalise(spec.name, &mut values);

        let key = spec.keys.iter().map(|column| values.get(*column).map(String::as_str).unwrap_or_default()).collect::<Vec<_>>().join("|");
        let occurrence = seen.entry(key.to_owned()).or_default();
        *occurrence += 1;
        let key = if *occurrence == 1 { key } else { format!("{key}#{occurrence}") };
        rows.insert(key, values);
    }
    Ok(rows)
}

/// Puts the pairs without direction (diplomatic relationships between equals, spouses) in a fixed order, and packs
/// horde buildings in the first columns, as the build does.
fn normalise(table: &str, values: &mut HashMap<String, String>) {
    if table == "start_pos_horde_details" {
        let buildings = HORDE_BUILDING_COLUMNS.iter().filter_map(|column| values.get(*column).filter(|building| !building.is_empty()).cloned()).collect::<Vec<_>>();
        for (index, column) in HORDE_BUILDING_COLUMNS.iter().enumerate() {
            values.insert(column.to_string(), buildings.get(index).cloned().unwrap_or_default());
        }
        return;
    }

    let (first, second) = match table {
        "start_pos_diplomacy" if !values.get("stance").is_some_and(|stance| DIRECTED_STANCES.contains(&stance.as_str())) => ("faction1", "faction2"),
        "start_pos_family_relationships" if values.get("relationship").is_some_and(|relationship| relationship == SPOUSE_RELATIONSHIP) => ("character", "related_to"),
        _ => return,
    };
    if values.get(first) > values.get(second) {
        let first_value = values.remove(first).unwrap_or_default();
        let second_value = values.remove(second).unwrap_or_default();
        values.insert(first.to_owned(), second_value);
        values.insert(second.to_owned(), first_value);
    }
}

/// Returns the province of each region, by region key.
fn region_provinces(tables: &StartposTables) -> Result<HashMap<String, String>> {
    tables.table("region_to_province_junctions")?.rows()
        .map(|row| Ok((row.str("region")?.into_owned(), row.str("province")?.into_owned())))
        .collect()
}

/// Replaces the values the build sums over a province, or over a faction's part of it, with their sum, in every row
/// of the group: the build only keeps the sums.
fn sum_by_province(rows: &mut BTreeMap<String, HashMap<String, String>>, provinces: &HashMap<String, String>) {
    let groups = |values: &HashMap<String, String>, column: &str| {
        let province = values.get("region").and_then(|region| provinces.get(region)).cloned().unwrap_or_default();
        if PROVINCE_SUMS.contains(&column) { province } else { format!("{province}|{}", values.get("owning_faction").cloned().unwrap_or_default()) }
    };

    for column in PROVINCE_SUMS.iter().chain(&PROVINCE_PART_SUMS) {
        let mut sums = HashMap::<String, i64>::new();
        for values in rows.values() {
            let value = values.get(*column).and_then(|value| value.parse::<i64>().ok()).unwrap_or_default();
            *sums.entry(groups(values, column)).or_default() += value;
        }
        for values in rows.values_mut() {
            let mut sum = sums.get(&groups(values, column)).copied().unwrap_or_default();
            if *column == "initial_public_order" {
                sum = sum.clamp(-MAX_PUBLIC_ORDER, MAX_PUBLIC_ORDER);
            }
            values.insert(column.to_string(), sum.to_string());
        }
    }
}

/// Returns if a row with a `campaign` column belongs to the campaign. Rows without the column belong to every one.
fn in_campaign(row: &Row, campaign: &str) -> bool {
    row.str("campaign").map(|value| value == campaign).unwrap_or(true)
}

/// Returns if two values are the same: equal as text, or as numbers within [`NUMBER_TOLERANCE`].
fn same_value(original: &str, extracted: &str) -> bool {
    if original == extracted {
        return true;
    }
    match (original.parse::<f64>(), extracted.parse::<f64>()) {
        (Ok(original), Ok(extracted)) => (original - extracted).abs() <= NUMBER_TOLERANCE * original.abs().max(1.0),
        _ => false,
    }
}
