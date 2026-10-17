//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Extraction of a campaign's `start_pos_*` tables from its startpos: the inverse of the build.
//!
//! The extractor reads a startpos through the record layouts and rebuilds the rows it was built from, so a startpos
//! without its tables can be edited again, and a generated startpos can be checked against the tables it came from
//! (see [`compare`]).
//!
//! The build doesn't keep everything, so the extracted rows are the ones that build the same startpos:
//!
//! - Columns the build ignores (descriptions, multiplayer counts...) keep the table's default value.
//! - A province's fertility, public order and development are sums over its regions: the sum goes to one region.
//! - Horde buildings fill the first building columns, and garrison and bodyguard units aren't rows.
//! - Faction and region ids are kept (the startpos has them in localisation keys). Other row ids are numbered from 1.
//! - Lists the game stores in hash order (units, traits, technologies...) come out in startpos order.
//!
//! # Submodules
//!
//! - [`compare`]: comparison of extracted tables with the tables a startpos was built from.

use getset::Getters;

use std::collections::{BTreeMap, HashMap};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::db::DB;
use rpfm_lib::files::esf::{NodeType, RecordNode, ESF};
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::schema::Schema;

use super::cai::CAI_WORLD_PATH;
use super::layout::{child_path, GroupView, StartposLayout};
use super::tables::StartposTables;
use super::world::writer::WORLD_PATH;

pub mod compare;
mod characters;
mod factions;
mod regions;

#[cfg(test)] pub(crate) mod extract_test;

/// Path of the root record.
const ROOT_PATH: &str = "CAMPAIGN_STARTPOS";

/// Path of the pre-campaign map info.
const PREOPEN_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO";

/// Path of the campaign environment.
const ENV_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV";

/// Path of the campaign model.
const MODEL_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL";

/// Path of a faction.
const FACTION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION";

/// Game tables the extraction reads, besides the startpos, without the `_tables` suffix.
pub const EXTRACTION_TABLES: [&str; 2] = ["character_trait_levels", "seasons"];

/// Localisation key prefix of the faction descriptions, followed by the faction's `start_pos_factions` id.
const DESCRIPTION_KEY_PREFIX: &str = "start_pos_factions_description_";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The `start_pos_*` rows of a campaign, extracted from its startpos.
#[derive(Debug, Clone, Default, Getters)]
pub struct ExtractedTables {

    /// Campaign key.
    #[getset(get = "pub")]
    campaign: String,

    /// Rows of each table, by table name (without the `_tables` suffix).
    tables: BTreeMap<String, TableRows>,
}

/// Rows of a table, with their values as text.
#[derive(Debug, Clone, Default, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct TableRows {

    /// Names of the columns the rows have. Other columns keep their default value.
    columns: Vec<&'static str>,

    /// Rows, with their values in `columns` order.
    rows: Vec<Vec<String>>,
}

/// What the extraction reads from a startpos, read once.
struct Source<'a> {

    /// Record layouts of the startpos.
    layout: &'a StartposLayout,

    /// Campaign key.
    campaign: String,

    /// The calendar (`CAMPAIGN_CALENDAR`).
    calendar: GroupView<'a>,

    /// The calendar's start date.
    start_date: GroupView<'a>,

    /// Factions, in `FACTION_ARRAY` order.
    factions: Vec<FactionSource<'a>>,

    /// Regions (`REGION` groups), in `REGIONS_ARRAY` order.
    regions: Vec<GroupView<'a>>,

    /// Provinces (`PROVINCE` groups), in `PROVINCE_ARRAY` order.
    provinces: Vec<GroupView<'a>>,

    /// The world, for its singletons (family tree, slaves).
    world: GroupView<'a>,

    /// The AI's factions (`CAI_FACTION` groups).
    cai_factions: Vec<GroupView<'a>>,
}

/// A faction of the startpos.
struct FactionSource<'a> {

    /// The `FACTION` group.
    group: GroupView<'a>,

    /// The faction's pre-campaign info (`FACTION_INFOS` group), if it has one.
    info: Option<GroupView<'a>>,

    /// Runtime id.
    id: u32,

    /// Faction key.
    key: String,

    /// Row id in `start_pos_factions`.
    table_id: String,
}

/// Numbers the rows whose ids the startpos doesn't keep.
#[derive(Debug, Default)]
struct Ids {
    last: u64,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Extracts the `start_pos_*` tables of a campaign from its startpos.
///
/// # Arguments
///
/// * `startpos` - The startpos.
/// * `layout` - Record layouts of the game (the ones the startpos is written with).
/// * `tables` - Game tables with the [`EXTRACTION_TABLES`].
///
/// # Returns
///
/// The rows of every table the startpos keeps something of.
///
/// # Errors
///
/// Returns an error if the startpos doesn't fit the layouts, or a table is missing.
pub fn extract_tables(startpos: &ESF, layout: &StartposLayout, tables: &StartposTables) -> Result<ExtractedTables> {
    let source = Source::read(startpos, layout)?;
    let mut ids = Ids::default();
    let mut extracted = ExtractedTables { campaign: source.campaign.to_owned(), tables: BTreeMap::new() };

    extracted.insert("start_pos_calendars", factions::calendar(&source, tables)?);
    extracted.insert("start_pos_factions", factions::factions(&source)?);
    extracted.insert("start_pos_technologies", factions::technologies(&source)?);
    extracted.insert("start_pos_victory_conditions", factions::victory_conditions(&source));
    extracted.insert("start_pos_diplomacy", factions::diplomacy(&source, &mut ids)?);

    let regions = regions::extract(&source, &mut ids)?;
    let characters = characters::extract(&source, tables, &regions, &mut ids)?;
    for (name, rows) in regions.tables.into_iter().chain(characters) {
        extracted.insert(name, rows);
    }
    Ok(extracted)
}

impl ExtractedTables {

    /// Returns the rows of a table, by name (without the `_tables` suffix).
    pub fn table(&self, name: &str) -> Option<&TableRows> {
        self.tables.get(name)
    }

    /// Returns the tables, by name (without the `_tables` suffix).
    pub fn tables(&self) -> impl Iterator<Item = (&str, &TableRows)> {
        self.tables.iter().map(|(name, rows)| (name.as_str(), rows))
    }

    /// Builds a DB file per table, with the newest definition of the schema. Columns the extraction doesn't fill keep
    /// their default value, and columns the definition lacks are left out.
    ///
    /// # Arguments
    ///
    /// * `schema` - Schema of the game.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema has no definition for a table, or a value doesn't fit its column's type.
    pub fn to_dbs(&self, schema: &Schema) -> Result<Vec<DB>> {
        self.tables.iter().map(|(name, rows)| {
            let table_name = format!("{name}_tables");
            let definition = schema.definitions_by_table_name(&table_name)
                .and_then(|definitions| definitions.iter().max_by_key(|definition| *definition.version()))
                .ok_or_else(|| extraction_error(&format!("the schema has no definition for {table_name}")))?;

            let mut db = DB::new(definition, None, &table_name);
            let fields = definition.fields_processed();
            let positions = rows.columns.iter()
                .map(|column| fields.iter().position(|field| field.name() == *column))
                .collect::<Vec<_>>();

            let mut data = Vec::with_capacity(rows.rows.len());
            for row in &rows.rows {
                let mut new_row = db.new_row();
                for (position, value) in positions.iter().zip(row) {
                    if let Some(position) = *position {
                        new_row[position] = DecodedData::new_from_type_and_string(fields[position].field_type(), value)?;
                    }
                }
                data.push(new_row);
            }
            db.set_data(&data)?;
            Ok(db)
        }).collect()
    }

    /// Builds a table set from the extracted rows, typed as the schema's newest definitions (see [`Self::to_dbs`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the schema has no definition for a table, or a value doesn't fit its column's type.
    pub fn to_tables(&self, schema: &Schema) -> Result<StartposTables> {
        let mut tables = StartposTables::new();
        for db in self.to_dbs(schema)? {
            tables.add_db(&db)?;
        }
        Ok(tables)
    }

    /// Adds the rows of a table.
    fn insert(&mut self, name: &str, rows: TableRows) {
        self.tables.insert(name.to_owned(), rows);
    }
}

impl TableRows {

    /// Creates an empty table with the given columns.
    fn new(columns: &[&'static str]) -> Self {
        Self { columns: columns.to_vec(), rows: vec![] }
    }

    /// Adds a row, with its values in column order.
    fn push(&mut self, row: Vec<String>) {
        debug_assert_eq!(row.len(), self.columns.len(), "a row of the extracted tables doesn't fit its columns");
        self.rows.push(row);
    }
}

impl<'a> Source<'a> {

    /// Reads the parts of a startpos the extraction needs.
    fn read(startpos: &'a ESF, layout: &'a StartposLayout) -> Result<Self> {
        let NodeType::Record(root) = startpos.root_node() else { return Err(extraction_error("the startpos has no root record")) };
        let root = layout.read_single(ROOT_PATH, root)?;
        let preopen = layout.read_single(PREOPEN_PATH, required(&root, "CAMPAIGN_PREOPEN_MAP_INFO")?)?;
        let env = layout.read_single(ENV_PATH, required(&root, "CAMPAIGN_ENV")?)?;
        let model = layout.read_single(MODEL_PATH, required(&env, "CAMPAIGN_MODEL")?)?;
        let world = layout.read_single(WORLD_PATH, required(&model, "WORLD")?)?;

        let calendar_path = child_path(MODEL_PATH, "CAMPAIGN_CALENDAR");
        let calendar = layout.read_single(&calendar_path, required(&model, "CAMPAIGN_CALENDAR")?)?;
        let start_date = layout.read_single(&child_path(&calendar_path, "DATE"), required(&calendar, "DATE")?)?;

        let infos_path = child_path(PREOPEN_PATH, "FACTION_INFOS");
        let infos = match preopen.record("FACTION_INFOS") {
            Some(record) => layout.read(&infos_path, record)?,
            None => vec![],
        };
        let mut infos = infos.into_iter()
            .filter_map(|info| Some((info.str("^+1")?.to_owned(), info)))
            .collect::<HashMap<_, _>>();

        let factions = layout.read_items(&child_path(WORLD_PATH, "FACTION_ARRAY"), &world, "FACTION_ARRAY", "FACTION")?.into_iter()
            .map(|(_, group)| {
                let key = group.str("^+2").unwrap_or_default().to_owned();
                let info = infos.remove(&key);
                let table_id = info.as_ref()
                    .and_then(|info| info.str("^+5"))
                    .and_then(|description| description.strip_prefix(DESCRIPTION_KEY_PREFIX))
                    .unwrap_or_default()
                    .to_owned();
                FactionSource { id: group.u32("^+1").unwrap_or_default(), group, info, key, table_id }
            })
            .collect();

        let region_manager_path = child_path(WORLD_PATH, "REGION_MANAGER");
        let region_manager = layout.read_single(&region_manager_path, required(&world, "REGION_MANAGER")?)?;
        let regions = layout.read_items(&child_path(&region_manager_path, "REGIONS_ARRAY"), &region_manager, "REGIONS_ARRAY", "REGION")?.into_iter()
            .map(|(_, region)| region)
            .collect();

        let province_manager_path = child_path(WORLD_PATH, "PROVINCE_MANAGER");
        let province_manager = layout.read_single(&province_manager_path, required(&world, "PROVINCE_MANAGER")?)?;
        let provinces = layout.read_items(&child_path(&province_manager_path, "PROVINCE_ARRAY"), &province_manager, "PROVINCE_ARRAY", "PROVINCE")?.into_iter()
            .map(|(_, province)| province)
            .collect();

        let cai_interface_path = child_path(MODEL_PATH, "CAI_INTERFACE");
        let mut cai_factions = vec![];
        if let Some(interface) = model.record("CAI_INTERFACE") {
            let interface = layout.read_single(&cai_interface_path, interface)?;
            if let Some(cai_world) = interface.record("CAI_WORLD") {
                let cai_world = layout.read_single(CAI_WORLD_PATH, cai_world)?;
                let factions_path = child_path(CAI_WORLD_PATH, "CAI_WORLD_FACTIONS");
                cai_factions = layout.read_items(&factions_path, &cai_world, "CAI_WORLD_FACTIONS", "CAI_FACTION")?.into_iter()
                    .map(|(_, faction)| faction)
                    .collect();
            }
        }

        Ok(Self {
            layout,
            campaign: preopen.str("DATE+1").unwrap_or_default().to_owned(),
            calendar,
            start_date,
            factions,
            regions,
            provinces,
            world,
            cai_factions,
        })
    }

    /// Returns the start year.
    fn start_year(&self) -> u32 {
        self.start_date.u32("^+1").unwrap_or_default()
    }

    /// Returns a faction by runtime id.
    fn faction(&self, id: u32) -> Option<&FactionSource<'a>> {
        self.factions.iter().find(|faction| faction.id == id)
    }

    /// Returns a faction by key.
    fn faction_by_key(&self, key: &str) -> Option<&FactionSource<'a>> {
        self.factions.iter().find(|faction| faction.key == key)
    }

    /// Reads the only group of a child record of a faction.
    fn faction_record(&self, faction: &FactionSource<'a>, key: &str) -> Result<Option<GroupView<'a>>> {
        faction.group.record(key)
            .map(|record| self.layout.read_single(&child_path(FACTION_PATH, key), record))
            .transpose()
    }
}

impl Ids {

    /// Returns a new id.
    fn next(&mut self) -> String {
        self.last += 1;
        self.last.to_string()
    }
}

/// Returns a child record of a group, or an error naming it if the group lacks it.
fn required<'a>(group: &GroupView<'a>, key: &str) -> Result<&'a RecordNode> {
    group.record(key).ok_or_else(|| extraction_error(&format!("missing {key}")))
}

/// Builds the error for a startpos that lacks something.
fn extraction_error(message: &str) -> RLibError {
    RLibError::StartposExtractionError(message.to_owned())
}
