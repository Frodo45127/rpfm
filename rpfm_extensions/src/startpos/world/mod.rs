//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! World stage of the startpos build: the content of `CAMPAIGN_MODEL/WORLD` (factions, characters, armies,
//! regions, provinces and sea regions) derived from the `start_pos_*` tables, the game tables and the map.
//!
//! The rules were reverse-engineered from game-built startposes of Attila's main campaign. The generators here
//! return keys and table values, not the startpos's runtime ids: ids are global counters assigned when the
//! records are written. Where the game draws from its RNG or stores hash-ordered lists, the generators make a
//! deterministic choice that gives a compatible startpos.
//!
//! # Submodules
//!
//! - [`agents`]: agent abilities, attributes, skill sets and recruitment pools.
//! - [`bodyguards`]: the default general unit of each faction.
//! - [`calendar`]: the campaign's start date and turns per year.
//! - [`characters`]: per-character values (birth year, loyalty, faction leader, map position).
//! - [`diplomacy`]: diplomatic relationships between factions.
//! - [`factions`]: per-faction values (capital, subjects, victory regions, flags and colours...).
//! - [`garrisons`]: the garrison army and navy of each settlement.
//! - [`government`]: government posts and political parties.
//! - [`legacy`]: army legacies (names, emblems, numbering).
//! - [`pools`]: mercenary and levy pools.
//! - [`provinces`]: per-faction province managers and sea regions.
//! - [`rebels`]: the rebel setup of each region.
//! - [`writer`]: the writer of the `WORLD` record, from the generators above and the record layouts.

use getset::{CopyGetters, Getters};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};

use super::tables::{Row, StartposTables};

pub mod agents;
pub mod bodyguards;
pub mod calendar;
pub mod characters;
pub mod diplomacy;
pub mod factions;
pub mod garrisons;
pub mod government;
pub mod legacy;
pub mod pools;
pub mod provinces;
pub mod rebels;
pub mod writer;

#[cfg(test)] mod world_test;

/// Building instance (`building_levels.building_instance_key`) of the primary buildings of major settlements.
const MAJOR_SETTLEMENT_INSTANCE: &str = "att_city_major";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The `start_pos_*` rows of one campaign, with their references resolved to keys.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct CampaignIndex {

    /// Campaign key.
    campaign: String,

    /// Factions of the campaign, in `start_pos_factions` order.
    factions: Vec<FactionEntry>,

    /// Regions of the campaign, in `start_pos_regions` order.
    regions: Vec<RegionEntry>,

    /// Settlements of the campaign's regions, in `start_pos_settlements` order.
    settlements: Vec<SettlementEntry>,

    /// Characters of the campaign's factions, in `start_pos_characters` order.
    characters: Vec<CharacterEntry>,

    /// Positions in `factions`, by faction key.
    #[getset(skip)]
    faction_positions: HashMap<String, usize>,

    /// Positions in `regions`, by region key.
    #[getset(skip)]
    region_positions: HashMap<String, usize>,
}

/// A faction of the campaign (`start_pos_factions` plus its `factions` and `cultures_subcultures` data).
#[derive(Debug, Clone, PartialEq, Default, Getters, CopyGetters)]
pub struct FactionEntry {

    /// Row id in `start_pos_factions`.
    #[getset(get = "pub")]
    id: String,

    /// Faction key.
    #[getset(get = "pub")]
    key: String,

    /// The faction can be picked by players.
    #[getset(get_copy = "pub")]
    playable: bool,

    /// Sorting value of the faction list.
    #[getset(get_copy = "pub")]
    starting_order: i64,

    /// Subculture key.
    #[getset(get = "pub")]
    subculture: String,

    /// Culture key.
    #[getset(get = "pub")]
    culture: String,

    /// Military group, used for unit permissions.
    #[getset(get = "pub")]
    military_group: String,
}

/// A region of the campaign (`start_pos_regions`).
#[derive(Debug, Clone, PartialEq, Default, Getters, CopyGetters)]
pub struct RegionEntry {

    /// Row id in `start_pos_regions`.
    #[getset(get = "pub")]
    id: String,

    /// Region key.
    #[getset(get = "pub")]
    key: String,

    /// Owner faction key. `None` for regions owned by the rebels.
    #[getset(get = "pub")]
    owner: Option<String>,

    /// The region is flagged as a faction capital.
    #[getset(get_copy = "pub")]
    capital_flag: bool,
}

/// A settlement of the campaign (`start_pos_settlements`).
#[derive(Debug, Clone, PartialEq, Default, Getters)]
#[getset(get = "pub")]
pub struct SettlementEntry {

    /// Row id in `start_pos_settlements`.
    id: String,

    /// Region key.
    region: String,

    /// Primary building level key.
    primary_building: String,

    /// Port building level key, empty without port.
    port_building: String,

    /// Other building level keys (`building1` to `building5`), without the empty ones.
    buildings: Vec<String>,

    /// Slave points the settlement starts with.
    slave_points: i64,

    /// The primary building is a major settlement one.
    #[getset(skip)]
    major: bool,
}

/// A character of the campaign (`start_pos_characters`).
#[derive(Debug, Clone, PartialEq, Default, Getters, CopyGetters)]
pub struct CharacterEntry {

    /// Row id in `start_pos_characters`.
    #[getset(get = "pub")]
    id: String,

    /// Faction key.
    #[getset(get = "pub")]
    faction: String,

    /// Agent type (`general`, `spy`...).
    #[getset(get = "pub")]
    agent: String,

    /// Age, in years.
    #[getset(get_copy = "pub")]
    age: i64,

    /// Starting cell column (`startx`), 0 if not set.
    #[getset(get_copy = "pub")]
    start_x: u16,

    /// Starting cell row (`starty`), 0 if not set.
    #[getset(get_copy = "pub")]
    start_y: u16,

    /// Ministerial position key, empty if none.
    #[getset(get = "pub")]
    ministerial_position: String,

    /// Loyalty, as written in the table.
    #[getset(get_copy = "pub")]
    loyalty: i64,

    /// Death type, empty for living characters.
    #[getset(get = "pub")]
    death_type: String,

    /// The character starts its faction's family tree.
    #[getset(get_copy = "pub")]
    progenitor: bool,

    /// Unit of the character's bodyguard, overriding the faction's default. Empty for the default.
    #[getset(get = "pub")]
    override_general_unit: String,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CampaignIndex {

    /// Indexes the startpos rows of a campaign.
    ///
    /// # Arguments
    ///
    /// * `tables` - Tables with `start_pos_factions`, `start_pos_regions`, `start_pos_settlements`,
    ///   `start_pos_characters`, `factions` and `cultures_subcultures`.
    /// * `campaign` - Campaign key.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing, or a faction lacks its `factions` row.
    pub fn new(tables: &StartposTables, campaign: &str) -> Result<Self> {
        let cultures = tables.table("cultures_subcultures")?.rows()
            .map(|row| Ok((row.str("subculture")?.into_owned(), row.str("culture")?.into_owned())))
            .collect::<Result<HashMap<_, _>>>()?;
        let game_factions = tables.table("factions")?.rows()
            .map(|row| Ok((row.str("key")?.into_owned(), row)))
            .collect::<Result<HashMap<_, _>>>()?;

        let mut factions = vec![];
        for row in tables.table("start_pos_factions")?.rows() {
            if row.str("campaign")? != campaign {
                continue;
            }
            let key = row.str("faction")?.into_owned();
            let game = game_factions.get(&key).ok_or_else(|| RLibError::StartposInvalidValue("start_pos_factions".to_owned(), "faction".to_owned(), key.to_owned()))?;
            let subculture = game.str("subculture")?.into_owned();
            factions.push(FactionEntry {
                id: row.str("ID")?.into_owned(),
                playable: row.bool("playable")?,
                starting_order: row.int("starting_order")?,
                culture: cultures.get(&subculture).cloned().unwrap_or_default(),
                military_group: game.str("military_group")?.into_owned(),
                subculture,
                key,
            });
        }
        let faction_of_id = factions.iter().map(|faction| (faction.id.to_owned(), faction.key.to_owned())).collect::<HashMap<_, _>>();

        let mut regions = vec![];
        for row in tables.table("start_pos_regions")?.rows() {
            if row.str("campaign")? != campaign {
                continue;
            }
            regions.push(RegionEntry {
                id: row.str("id")?.into_owned(),
                key: row.str("region")?.into_owned(),
                owner: faction_of_id.get(row.str("owning_faction")?.as_ref()).cloned(),
                capital_flag: row.bool("faction_capital")?,
            });
        }
        let region_of_id = regions.iter().map(|region| (region.id.to_owned(), region.key.to_owned())).collect::<HashMap<_, _>>();

        let major_buildings = tables.table("building_levels")?.rows()
            .filter(|row| row.str("building_instance_key").is_ok_and(|instance| instance == MAJOR_SETTLEMENT_INSTANCE))
            .map(|row| row.str("level_name").map(|level| level.into_owned()))
            .collect::<Result<HashSet<_>>>()?;

        let mut settlements = vec![];
        for row in tables.table("start_pos_settlements")?.rows() {
            let Some(region) = region_of_id.get(row.str("region")?.as_ref()) else { continue };
            let mut buildings = vec![];
            for column in ["building1", "building2", "building3", "building4", "building5"] {
                let building = row.str(column)?;
                if !building.is_empty() {
                    buildings.push(building.into_owned());
                }
            }
            let primary_building = row.str("primary_building")?.into_owned();
            settlements.push(SettlementEntry {
                id: row.str("id")?.into_owned(),
                region: region.to_owned(),
                major: major_buildings.contains(&primary_building),
                primary_building,
                port_building: row.str("port_building")?.into_owned(),
                buildings,
                slave_points: row.int("startpos_slave_points")?,
            });
        }

        let mut characters = vec![];
        for row in tables.table("start_pos_characters")?.rows() {
            let Some(faction) = faction_of_id.get(row.str("faction")?.as_ref()) else { continue };
            characters.push(CharacterEntry {
                id: row.str("ID")?.into_owned(),
                faction: faction.to_owned(),
                agent: row.str("Type")?.into_owned(),
                age: row.int("Age")?,
                start_x: cell_coordinate(&row, "startx")?,
                start_y: cell_coordinate(&row, "starty")?,
                ministerial_position: row.str("ministerial_position")?.into_owned(),
                loyalty: row.int("loyalty")?,
                death_type: row.str("death_type")?.into_owned(),
                progenitor: row.bool("progenitor")?,
                override_general_unit: row.str("override_general_unit")?.into_owned(),
            });
        }

        Ok(Self {
            campaign: campaign.to_owned(),
            faction_positions: factions.iter().enumerate().map(|(index, faction)| (faction.key.to_owned(), index)).collect(),
            region_positions: regions.iter().enumerate().map(|(index, region)| (region.key.to_owned(), index)).collect(),
            factions,
            regions,
            settlements,
            characters,
        })
    }

    /// Returns a faction by key.
    pub fn faction(&self, key: &str) -> Option<&FactionEntry> {
        self.faction_positions.get(key).map(|&index| &self.factions[index])
    }

    /// Returns a faction by its `start_pos_factions` row id.
    pub fn faction_by_id(&self, id: &str) -> Option<&FactionEntry> {
        self.factions.iter().find(|faction| faction.id == id)
    }

    /// Returns a region by key.
    pub fn region(&self, key: &str) -> Option<&RegionEntry> {
        self.region_positions.get(key).map(|&index| &self.regions[index])
    }

    /// Returns the regions owned by a faction, in table order.
    pub fn regions_of<'a>(&'a self, faction: &'a str) -> impl Iterator<Item = &'a RegionEntry> + 'a {
        self.regions.iter().filter(move |region| region.owner.as_deref() == Some(faction))
    }

    /// Returns the capital of a faction: the first of its regions flagged as capital, in table order.
    pub fn capital_of<'a>(&'a self, faction: &'a str) -> Option<&'a RegionEntry> {
        self.regions_of(faction).find(|region| region.capital_flag)
    }

    /// Returns the settlement of a region.
    pub fn settlement_of(&self, region: &str) -> Option<&SettlementEntry> {
        self.settlements.iter().find(|settlement| settlement.region == region)
    }

    /// Returns the characters of a faction, in table order.
    pub fn characters_of<'a>(&'a self, faction: &'a str) -> impl Iterator<Item = &'a CharacterEntry> + 'a {
        self.characters.iter().filter(move |character| character.faction == faction)
    }

    /// Returns the factions in startpos order: playable factions first, each group by `starting_order`.
    ///
    /// The game sorts with an unstable sort, so factions with the same starting order may come out in another
    /// order in game-built startposes.
    pub fn factions_in_startpos_order(&self) -> Vec<&FactionEntry> {
        let mut factions = self.factions.iter().collect::<Vec<_>>();
        factions.sort_by_key(|faction| (!faction.playable, faction.starting_order));
        factions
    }
}

impl SettlementEntry {

    /// Returns if the settlement is a major one (its primary building is a [`MAJOR_SETTLEMENT_INSTANCE`]). Major
    /// settlements have walls and are the main region of their province.
    pub fn is_major(&self) -> bool {
        self.major
    }

    /// Returns every building level of the settlement: primary, port (if any) and the others, in that order.
    pub fn all_buildings(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.primary_building.as_str())
            .chain(std::iter::once(self.port_building.as_str()))
            .chain(self.buildings.iter().map(String::as_str))
            .filter(|building| !building.is_empty())
    }
}

/// Reads a starting cell coordinate, stored as a float in `start_pos_characters`.
fn cell_coordinate(row: &Row, column: &str) -> Result<u16> {
    let value = row.f32(column)?;
    if !(0.0..=f32::from(u16::MAX)).contains(&value) {
        return Err(RLibError::StartposInvalidValue("start_pos_characters".to_owned(), column.to_owned(), value.to_string()));
    }
    Ok(value as u16)
}
