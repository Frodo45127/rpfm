//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Generation of a campaign's startpos from a pack and the game's files, without launching the game.
//!
//! [`load_tables`] loads the tables (the game's, its Assembly Kit's and the pack's, merged as the game loads them),
//! which [`validation`](super::validation) checks. [`StartposSources`] adds what else the build reads: the campaign
//! map's files, the victory objectives, the characters with legacy override texts and the start season's name. [`GameLayouts`] gets the
//! record layouts from the game's startposes, cached in a layouts folder.
//! [`generate_startpos`] writes the startpos.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{Container, Decodeable, DecodeableExtraData, FileType, RFile, RFileDecoded};
use rpfm_lib::files::db::DB;
use rpfm_lib::files::esf::ESF;
use rpfm_lib::files::pack::Pack;
use rpfm_lib::files::table::DecodedData;
use rpfm_lib::files::trade_routes::TradeRoutes;
use rpfm_lib::games::GameInfo;
use rpfm_lib::games::supported_games::KEY_ATTILA;
use rpfm_lib::schema::Schema;

use crate::dependencies::Dependencies;
use super::layout::{layout_file_name, StartposLayout};
use super::missions::{parse_victory_objectives, VictoryObjectives};
use super::region_map::RegionMap;
use super::settlements::SettlementLayout;
use super::tables::StartposTables;
use super::trade::TradeNetwork;
use super::validation::referenced_tables;
use super::world::CampaignIndex;
use super::world::calendar::Calendar;
use super::world::writer::{WorldInputs, WorldPlan};
use super::writer::{write_startpos, StartposInputs};

#[cfg(test)] #[path = "generator_test.rs"] mod generator_test;

/// Games whose startposes can be generated.
pub const SUPPORTED_GAMES: [&str; 1] = [KEY_ATTILA];

/// Tables the startpos build reads, without the `_tables` suffix.
pub const STARTPOS_TABLES: &[&str] = &[
    "agent_culture_details", "agent_to_agent_abilities", "agent_to_agent_attributes", "agent_uniforms", "agents",
    "armed_citizenry_units_to_unit_groups_junctions", "battlefield_engines", "building_chains", "building_level_armed_citizenry_junctions",
    "building_levels", "campaign_character_art_sets", "campaign_character_art_sets_group_junctions", "campaigns",
    "character_skill_node_sets", "character_skill_nodes", "character_trait_levels", "commander_unit_permissions", "commodities",
    "cultures_subcultures", "faction_political_parties_junctions", "faction_to_mercenary_set_junctions", "factions", "land_units",
    "main_units", "mercenary_pool_to_groups_junctions", "mercenary_unit_groups", "military_force_legacy_emblems",
    "military_force_legacy_names", "ministerial_positions", "ministerial_positions_culture_details", "naval_units",
    "political_parties", "province_to_mercenary_set_junctions", "region_to_province_junctions", "regions", "religions", "resources",
    "start_pos_calendars", "start_pos_character_to_settlements", "start_pos_character_traits", "start_pos_characters",
    "start_pos_diplomacy", "start_pos_family_relationships", "start_pos_factions", "start_pos_horde_details", "start_pos_land_units",
    "start_pos_naval_units", "start_pos_past_events", "start_pos_region_religions", "start_pos_region_slot_templates",
    "start_pos_regions", "start_pos_settlements", "start_pos_technologies", "start_pos_victory_conditions",
    "technology_node_sets", "technology_nodes", "technology_unit_upgrades", "units_to_groupings_military_permissions",
];

/// Path of the victory objectives script.
const VICTORY_OBJECTIVES_PATH: &str = "db/victory_objectives.txt";

/// Localisation key prefix of the `start_pos_characters.legacy_override` texts.
const LEGACY_OVERRIDE_PREFIX: &str = "start_pos_characters_legacy_override_";

/// Localisation key prefix of the season names.
const SEASON_NAME_PREFIX: &str = "seasons_onscreen_";

/// Suffix of the campaign map keys, which the region lookup image's name lacks.
const MAP_KEY_SUFFIX: &str = "_map";

/// Suffix of the region lookup image's name.
const LOOKUP_IMAGE_SUFFIX: &str = "_lookup.tga";

/// TGA image type of colour-mapped images.
const TGA_COLOUR_MAPPED: u8 = 1;

/// Size of a TGA header.
const TGA_HEADER_SIZE: usize = 18;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What a campaign's startpos is built from.
pub struct StartposSources {

    /// Campaign key.
    campaign: String,

    /// Campaign map key.
    map: String,

    /// Localised name of the season the campaign starts in, empty if it has none.
    season_name: String,

    /// RGB colours of the campaign map's region lookup image, by palette index. Empty without the image.
    region_palette: Vec<[u8; 3]>,

    /// Tables the build reads.
    tables: StartposTables,

    /// The campaign map's `map_data.esf`.
    map_data: ESF,

    /// The campaign map's `hlp_data.esf`.
    hlp_data: ESF,

    /// The campaign map's `dynamic_resources.esf`.
    dynamic_resources: ESF,

    /// The campaign map's `trade_routes.ptd`.
    trade_routes: TradeRoutes,

    /// Victory objectives of the campaigns (`db/victory_objectives.txt`).
    victory_objectives: Vec<VictoryObjectives>,

    /// `start_pos_characters` ids of the characters with a legacy override text.
    legacy_overrides: HashSet<String>,
}

/// Where a file is looked for: the pack first, then the game's files.
struct FileSource<'a> {
    pack: &'a Pack,
    dependencies: &'a Dependencies,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl StartposSources {

    /// Collects the sources of a campaign's startpos from its tables (see [`load_tables`]), a pack and the game's
    /// files. Files come from the pack if it has them, otherwise from the game.
    ///
    /// # Arguments
    ///
    /// * `tables` - The tables of the build.
    /// * `dependencies` - The game's files.
    /// * `pack` - Pack with the campaign's startpos tables.
    /// * `campaign` - Campaign key.
    ///
    /// # Errors
    ///
    /// Returns an error if a file is missing or can't be decoded.
    pub fn from_tables(tables: StartposTables, dependencies: &Dependencies, pack: &Pack, campaign: &str) -> Result<Self> {
        let map = tables.table("campaigns")?.rows()
            .find_map(|row| (row.str("campaign_name").ok()? == campaign).then(|| row.str("map_name").ok()).flatten())
            .ok_or_else(|| RLibError::StartposInvalidValue("campaigns".to_owned(), "campaign_name".to_owned(), campaign.to_owned()))?
            .into_owned();

        let files = FileSource { pack, dependencies };
        let map_file = |name: &str| files.bytes(&format!("campaign_maps/{map}/{name}"));
        let esf = |name: &str| -> Result<ESF> { ESF::decode(&mut Cursor::new(map_file(name)?), &None) };
        let victory_text = files.bytes(VICTORY_OBJECTIVES_PATH)?;
        let lookup_image = format!("{}{LOOKUP_IMAGE_SUFFIX}", map.strip_suffix(MAP_KEY_SUFFIX).unwrap_or(&map));
        let region_palette = match map_file(&lookup_image) {
            Ok(bytes) => tga_palette(&bytes)?,
            Err(_) => vec![],
        };
        let season = Calendar::new(&tables, campaign)?.start_season().to_owned();

        Ok(Self {
            campaign: campaign.to_owned(),
            season_name: loc_text(dependencies, pack, &format!("{SEASON_NAME_PREFIX}{season}")).unwrap_or_default(),
            region_palette,
            map_data: esf("map_data.esf")?,
            hlp_data: esf("hlp_data.esf")?,
            dynamic_resources: esf("dynamic_resources.esf")?,
            trade_routes: TradeRoutes::decode(&mut Cursor::new(map_file("trade_routes.ptd")?), &None)?,
            victory_objectives: parse_victory_objectives(&String::from_utf8_lossy(&victory_text))?,
            legacy_overrides: legacy_overrides(dependencies, pack),
            map,
            tables,
        })
    }
}

/// Loads the tables of a startpos build: the ones the build reads, and every `start_pos_*` table of the schema with
/// the tables it references, for validation.
///
/// Each table merges the game's fragments and then the pack's, in load order: a row replaces an earlier row with the
/// same key. Tables neither has come from the game's Assembly Kit.
///
/// # Arguments
///
/// * `dependencies` - The game's files.
/// * `pack` - Pack with the campaign's startpos tables.
/// * `schema` - Schema of the game, to decode the tables.
///
/// # Errors
///
/// Returns an error if a table can't be decoded, or its fragments don't share their columns.
pub fn load_tables(dependencies: &Dependencies, pack: &Pack, schema: &Schema) -> Result<StartposTables> {
    let mut extra = DecodeableExtraData::default();
    extra.set_schema(Some(schema));
    let extra = Some(extra);

    let mut names = STARTPOS_TABLES.iter().map(|name| name.to_string()).collect::<BTreeSet<_>>();
    names.extend(referenced_tables(schema));
    names.extend(schema.definitions().keys().filter(|name| name.starts_with("start_pos_")).map(|name| name.trim_end_matches("_tables").to_owned()));

    let mut tables = StartposTables::new();
    for table in &names {
        let folder = format!("db/{table}_tables/");
        let mut fragments = vec![];
        for file in dependencies.db_data(&format!("{table}_tables"), true, true)? {
            fragments.extend(decoded_db(file, &extra)?);
        }

        let mut pack_files = pack.files().iter().filter(|(path, _)| path.starts_with(&folder)).collect::<Vec<_>>();
        pack_files.sort_by_key(|(path, _)| *path);
        let mut pack_fragments = vec![];
        for (_, file) in pack_files {
            pack_fragments.extend(decoded_db(file, &extra)?);
        }

        if fragments.is_empty() && pack_fragments.is_empty() {
            fragments.extend(dependencies.asskit_only_db_tables().get(&format!("{table}_tables")).cloned());
        }
        fragments.extend(pack_fragments);
        add_merged(&mut tables, table, &fragments)?;
    }
    Ok(tables)
}

impl FileSource<'_> {

    /// Returns the bytes of a file, from the pack if it has it, otherwise from the game's files.
    fn bytes(&self, path: &str) -> Result<Vec<u8>> {
        let file = match self.pack.file(path, true) {
            Some(file) => file,
            None => self.dependencies.file(path, true, true, true)?,
        };

        // Files are cloned so the pack's and the cache's copies keep their state.
        file.clone().encode(&None, false, false, true)?.ok_or_else(|| RLibError::FileNotFound(path.to_owned()))
    }
}

/// Generates the startpos of a campaign.
///
/// # Arguments
///
/// * `sources` - What the startpos is built from.
/// * `layout` - Record layouts of the game.
///
/// # Errors
///
/// Returns an error if the sources lack data the build needs.
pub fn generate_startpos(sources: &StartposSources, layout: &StartposLayout) -> Result<ESF> {
    generate_startpos_with_plan(sources, layout, |index| WorldPlan::new(index))
}

/// Generates the startpos of a campaign with the orders and ids of its world given by a plan.
pub(crate) fn generate_startpos_with_plan(sources: &StartposSources, layout: &StartposLayout, plan: impl for<'i> FnOnce(&'i CampaignIndex) -> WorldPlan<'i>) -> Result<ESF> {
    let index = CampaignIndex::new(&sources.tables, &sources.campaign)?;
    let settlements = SettlementLayout::all_from_map_data(&sources.map_data, |region| index.settlement_of(region).is_some_and(|settlement| settlement.is_major()))?;
    let region_map = RegionMap::from_map_data(&sources.map_data)?;
    let world = WorldInputs {
        tables: &sources.tables,
        index: &index,
        layout,
        settlements: &settlements,
        region_map: &region_map,
        legacy_overrides: &sources.legacy_overrides,
    };

    let trade_network = TradeNetwork::from_trade_routes(&sources.trade_routes, &region_map.scale());
    let inputs = StartposInputs {
        world: &world,
        trade_network: &trade_network,
        victory_objectives: &sources.victory_objectives,
        map_data: &sources.map_data,
        hlp_data: &sources.hlp_data,
        dynamic_resources: &sources.dynamic_resources,
        campaign: &sources.campaign,
        map: &sources.map,
        season_name: &sources.season_name,
        region_palette: &sources.region_palette,
    };
    write_startpos(&inputs, &plan(&index))
}

/// Record layouts of a game: one per campaign the game has a startpos for, and one merged from all of them.
#[derive(Debug, Clone)]
pub struct GameLayouts {

    /// Layout merged from the startposes of all the game's campaigns.
    merged: StartposLayout,

    /// Layout of each of the game's campaigns, by campaign key.
    campaigns: BTreeMap<String, StartposLayout>,
}

impl GameLayouts {

    /// Loads the layouts of a game, extracting them from the game's startposes if they aren't cached or one of the
    /// startposes is newer than the cache (like after a patch, or a new campaign).
    ///
    /// # Arguments
    ///
    /// * `layouts_folder` - Folder with the cached layouts.
    /// * `game` - The game.
    /// * `game_data_path` - The game's data folder, with `campaigns/<campaign>/startpos.esf`.
    ///
    /// # Errors
    ///
    /// Returns an error if the game has no startpos to take the layouts from, or one can't be decoded or cached.
    pub fn load_or_extract(layouts_folder: &Path, game: &GameInfo, game_data_path: &Path) -> Result<Self> {
        let startposes = game_startposes(game_data_path);
        if startposes.is_empty() {
            return Err(RLibError::StartposLayoutError(format!("the game has no startpos in {} to take the record layouts from", game_data_path.join("campaigns").display())));
        }

        let merged_path = layouts_folder.join(layout_file_name(game.key(), None));
        let campaign_path = |campaign: &str| layouts_folder.join(layout_file_name(game.key(), Some(campaign)));
        let modified = |path: &Path| path.metadata().and_then(|metadata| metadata.modified()).ok();
        let newest_startpos = startposes.values().filter_map(|path| modified(path)).max();
        let is_current = |path: &Path| modified(path).is_some_and(|cached| newest_startpos.is_some_and(|newest| cached >= newest));

        if is_current(&merged_path) && startposes.keys().all(|campaign| is_current(&campaign_path(campaign))) {
            let cached = StartposLayout::load(&merged_path).and_then(|merged| {
                let campaigns = startposes.keys().map(|campaign| Ok((campaign.to_owned(), StartposLayout::load(&campaign_path(campaign))?))).collect::<Result<_>>()?;
                Ok(Self { merged, campaigns })
            });
            if let Ok(layouts) = cached {
                return Ok(layouts);
            }
        }

        // Each startpos is decoded once, for its own layout and for the merged one.
        let mut campaigns = BTreeMap::new();
        let merged = StartposLayout::from_esfs(startposes.iter().map(|(campaign, path)| {
            let startpos = ESF::decode(&mut BufReader::new(File::open(path)?), &None)?;
            campaigns.insert(campaign.to_owned(), StartposLayout::from_esf(&startpos)?);
            Ok(startpos)
        }))?;

        std::fs::create_dir_all(layouts_folder)?;
        for (campaign, layout) in &campaigns {
            layout.save(&campaign_path(campaign))?;
        }
        merged.save(&merged_path)?;
        Ok(Self { merged, campaigns })
    }

    /// Returns the layout to write a campaign's startpos with: the campaign's own if the game has its startpos,
    /// otherwise the one of a game campaign on the same map, otherwise the merged one. The campaign's or map's layout
    /// takes the records it lacks from the merged one.
    ///
    /// # Arguments
    ///
    /// * `tables` - Tables with `campaigns`, to find the campaigns on the same map.
    /// * `campaign` - Campaign key.
    pub fn for_campaign(&self, tables: &StartposTables, campaign: &str) -> StartposLayout {
        let maps = tables.table("campaigns").map(|table| table.rows()
            .filter_map(|row| Some((row.str("campaign_name").ok()?.into_owned(), row.str("map_name").ok()?.into_owned())))
            .collect::<HashMap<_, _>>())
            .unwrap_or_default();
        let map = maps.get(campaign);

        let primary = self.campaigns.get(campaign)
            .or_else(|| self.campaigns.iter().find(|(other, _)| map.is_some() && maps.get(other.as_str()) == map).map(|(_, layout)| layout));
        match primary {
            Some(layout) => layout.clone().with_fallback(&self.merged),
            None => self.merged.clone(),
        }
    }
}

/// Returns the paths of the game's startposes, by campaign key.
fn game_startposes(game_data_path: &Path) -> BTreeMap<String, PathBuf> {
    std::fs::read_dir(game_data_path.join("campaigns"))
        .map(|entries| entries.flatten()
            .filter_map(|entry| Some((entry.file_name().into_string().ok()?, entry.path().join("startpos.esf"))))
            .filter(|(_, path)| path.is_file())
            .collect())
        .unwrap_or_default()
}

/// Returns a decoded DB from a file, decoding a copy of it if needed. Empty files of tables without definitions
/// hold no rows, so they're skipped.
fn decoded_db(file: &RFile, extra: &Option<DecodeableExtraData>) -> Result<Option<DB>> {
    if let Ok(RFileDecoded::DB(db)) = file.decoded() {
        return Ok(Some(db.clone()));
    }
    match file.clone().decode(extra, false, true) {
        Ok(Some(RFileDecoded::DB(db))) => Ok(Some(db)),
        Err(RLibError::DecodingDBNoDefinitionsFoundAndEmptyFile) => Ok(None),
        Err(error) => Err(error),
        Ok(_) => Err(RLibError::DecodingDBNotADBTable),
    }
}

/// Adds a table merged from its fragments, in load order: a row replaces an earlier row with the same key, in place.
fn add_merged(tables: &mut StartposTables, table: &str, fragments: &[DB]) -> Result<()> {
    let Some(first) = fragments.first() else { return Ok(()) };
    let columns = first.definition().fields_processed().iter().map(|field| field.name().to_owned()).collect::<Vec<_>>();

    let mut rows: Vec<Vec<DecodedData>> = vec![];
    let mut positions = HashMap::new();
    for fragment in fragments {
        let fragment_columns = fragment.definition().fields_processed().iter().map(|field| field.name().to_owned()).collect::<Vec<_>>();
        let order = columns.iter()
            .map(|column| fragment_columns.iter().position(|other| other == column).ok_or_else(|| RLibError::StartposMissingColumn(table.to_owned(), column.to_owned())))
            .collect::<Result<Vec<_>>>()?;
        let keys = fragment.definition().key_column_positions();

        for row in fragment.data().iter() {
            let key = keys.iter().map(|&column| row[column].data_to_string().into_owned()).collect::<Vec<_>>();
            let row = order.iter().map(|&source| row[source].clone()).collect::<Vec<_>>();
            match (keys.is_empty(), positions.get(&key)) {
                (false, Some(&position)) => rows[position] = row,
                (false, None) => {
                    positions.insert(key, rows.len());
                    rows.push(row);
                },
                (true, _) => rows.push(row),
            }
        }
    }
    tables.add_rows(table, &columns, &rows)
}

/// Returns the `start_pos_characters` ids with a legacy override text, in the pack's or the game's localisation.
fn legacy_overrides(dependencies: &Dependencies, pack: &Pack) -> HashSet<String> {
    let game_texts = dependencies.localisation_data().iter().map(|(key, text)| (key.to_owned(), text.to_owned()));
    let pack_texts = pack.files_by_type(&[FileType::Loc]).into_iter().flat_map(loc_texts);
    game_texts.chain(pack_texts)
        .filter(|(_, text)| !text.is_empty())
        .filter_map(|(key, _)| key.strip_prefix(LEGACY_OVERRIDE_PREFIX).map(str::to_owned))
        .collect()
}

/// Returns the text of a localisation key, from the pack if it has it, else from the game.
fn loc_text(dependencies: &Dependencies, pack: &Pack, key: &str) -> Option<String> {
    pack.files_by_type(&[FileType::Loc]).into_iter()
        .flat_map(loc_texts)
        .find_map(|(loc_key, text)| (loc_key == key).then_some(text))
        .or_else(|| dependencies.localisation_data().get(key).cloned())
}

/// Returns the RGB colours of a colour-mapped TGA image's palette, by palette index.
///
/// # Errors
///
/// Returns an error if the image isn't colour-mapped, or is cut short.
pub(crate) fn tga_palette(bytes: &[u8]) -> Result<Vec<[u8; 3]>> {
    let invalid = || RLibError::CampaignMapDataError("the region lookup image isn't a colour-mapped TGA".to_owned());
    let header = bytes.get(..TGA_HEADER_SIZE).ok_or_else(invalid)?;
    if header[2] != TGA_COLOUR_MAPPED {
        return Err(invalid());
    }

    // Header bytes: 0 id length, 3-4 first palette index, 5-6 palette length, 7 bits per palette entry (BGR order).
    let start = TGA_HEADER_SIZE + usize::from(header[0]);
    let length = usize::from(u16::from_le_bytes([header[5], header[6]]));
    let entry_size = usize::from(header[7]) / 8;
    let palette = bytes.get(start..start + length * entry_size).ok_or_else(invalid)?;
    if entry_size < 3 {
        return Err(invalid());
    }
    Ok(palette.chunks_exact(entry_size).map(|entry| [entry[2], entry[1], entry[0]]).collect())
}

/// Returns the (key, text) pairs of a localisation file, or none if it can't be decoded.
fn loc_texts(file: &RFile) -> Vec<(String, String)> {
    let decoded = match file.decoded() {
        Ok(decoded) => Some(decoded.clone()),
        Err(_) => file.clone().decode(&None, false, true).ok().flatten(),
    };
    match decoded {
        Some(RFileDecoded::Loc(loc)) => loc.data().iter()
            .filter_map(|row| Some((row.first()?.data_to_string().into_owned(), row.get(1)?.data_to_string().into_owned())))
            .collect(),
        _ => vec![],
    }
}
