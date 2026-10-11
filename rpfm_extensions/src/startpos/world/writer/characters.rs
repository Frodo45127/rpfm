//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the characters of a faction (`FACTION/CHARACTER_ARRAY`).
//!
//! The game draws some values from its RNG (art sets, birth seasons, colonels' ages and names): this writer picks
//! valid ones deterministically.

use std::collections::HashMap;
use std::f32::consts::PI;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{Coordinates2DNode, NodeType, RecordNode};

use super::forces::{ForcePlan, PlannedCharacter};
use super::government::PostPlan;
use super::sight::{write_line_of_sight, SightPlan};
use super::{child, WorldInputs, WorldPlan};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::coords::HexCell;
use crate::startpos::tables::Row;
use crate::startpos::vision::Vision;
use crate::startpos::world::agents::{agent_abilities, agent_attributes, skill_set, ABILITIES, ATTRIBUTES};
use crate::startpos::world::bodyguards::{bodyguard, Bodyguard};
use crate::startpos::world::characters::{birth_year, loyalty, placement, Placement};
use crate::startpos::world::government::faction_leader;
use crate::startpos::world::{CampaignIndex, CharacterEntry, FactionEntry};

#[path = "family.rs"] mod family;

#[cfg(test)] #[path = "characters_test.rs"] mod characters_test;

/// Path of the characters of a faction.
const CHARACTER_ARRAY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY";

/// Path of a character.
const CHARACTER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER";

/// Path of a character's details.
const DETAILS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER/CHARACTER_DETAILS";


/// Agent type of the generated garrison commanders.
const COLONEL_AGENT: &str = "colonel";

/// Agent type of the generals.
const GENERAL_AGENT: &str = "general";

/// Age of the generated colonels (the game draws it from its RNG).
const COLONEL_AGE: u32 = 30;

/// Localisation key prefix of the character names.
const NAME_KEY_PREFIX: &str = "names_name_";

/// `start_pos_characters` name columns, by the kind the startpos stores with each name.
const NAME_COLUMNS: [&str; 4] = ["Name", "Surname", "clan_name", "other_name"];

/// Suffix the startpos adds to the art set of a character (`CHARACTER_DETAILS` #34).
const ART_SET_SUFFIX: &str = "0";

/// Value of `LINE_OF_SIGHT+18` (`CHARACTER` #28) for faction leaders (meaning unknown).
const LEADER_VALUE: u32 = 4;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Writes the characters of the factions.
pub struct CharacterWriter<'a, 'b> {

    /// Tables, startpos rows and layouts.
    inputs: &'b WorldInputs<'a>,

    /// Characters and forces of the world.
    forces: &'b ForcePlan<'a>,

    /// Government posts of the factions.
    posts: &'b PostPlan,

    /// Vision of the world.
    sight: &'b SightPlan,

    /// `start_pos_characters` rows, by id.
    rows: HashMap<String, Row<'a>>,

    /// Traits of each character (trait key and points of its level), by `start_pos_characters` id.
    traits: HashMap<String, Vec<(String, i64)>>,

    /// Move points and line of sight of each agent type.
    agents: HashMap<String, (u32, f32)>,

    /// Art sets of the campaign.
    art_sets: Vec<ArtSet>,

    /// Default bodyguard of each faction's generals, by faction key.
    bodyguards: HashMap<String, Option<Bodyguard>>,

    /// Family member id of each planned character, by position in the plan.
    family_ids: Vec<u32>,

    /// Family member id of each table character, living or dead, by `start_pos_characters` id.
    entry_family_ids: HashMap<String, u32>,

    /// The campaign's start year.
    start_year: u32,
}

/// An art set a character can use (`campaign_character_art_sets`).
#[derive(Debug, Clone)]
struct ArtSet {
    id: String,
    agent: String,
    is_male: bool,
    faction: String,
    subculture: String,
    culture: String,
}

/// Who a `CHARACTER_DETAILS` record describes: a character of the world, or a dead one of the family tree.
struct DetailsSubject<'r> {

    /// Path of the record.
    path: &'r str,

    /// The character's faction.
    faction: &'r FactionEntry,

    /// The character's table entry, `None` for colonels.
    entry: Option<&'r CharacterEntry>,

    /// Runtime id of the character, 0 for the dead.
    character_id: u32,

    /// Family member id of the character.
    family_id: u32,

    /// Death year and death type code, for the dead.
    death: Option<(u32, i32)>,

    /// Bodyguard of a general without a force.
    bodyguard: Option<Bodyguard>,
}

/// What the writer needs to know about a character, from its table row or as a generated colonel.
struct CharacterInfo<'r> {
    agent: String,
    is_male: bool,
    birth_year: u32,
    row: Option<&'r Row<'r>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl<'a, 'b> CharacterWriter<'a, 'b> {

    /// Prepares the writer of the characters.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Tables, startpos rows and layouts.
    /// * `plan` - Order of the factions and regions.
    /// * `forces` - Characters and forces of the world.
    /// * `posts` - Government posts of the factions.
    /// * `sight` - Vision of the world.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(inputs: &'b WorldInputs<'a>, plan: &WorldPlan<'a>, forces: &'b ForcePlan<'a>, posts: &'b PostPlan, sight: &'b SightPlan) -> Result<Self> {
        let tables = inputs.tables;
        let mut rows = HashMap::new();
        for row in tables.table("start_pos_characters")?.rows() {
            rows.insert(row.str("ID")?.into_owned(), row);
        }

        let mut levels = HashMap::new();
        for row in tables.table("character_trait_levels")?.rows() {
            levels.insert(row.str("key")?.into_owned(), (row.str("trait")?.into_owned(), row.int("threshold_points")?));
        }
        let mut traits = HashMap::<String, Vec<(String, i64)>>::new();
        for row in tables.table("start_pos_character_traits")?.rows() {
            if let Some(level) = levels.get(row.str("trait_level")?.as_ref()) {
                traits.entry(row.str("character_id")?.into_owned()).or_default().push(level.clone());
            }
        }

        let mut agents = HashMap::new();
        for row in tables.table("agents")?.rows() {
            agents.insert(row.str("key")?.into_owned(), (u32::try_from(row.int("move_points")?).unwrap_or_default(), row.number("line_of_sight")? as f32));
        }

        let mut groups = vec![];
        for row in tables.table("campaign_character_art_sets_group_junctions")?.rows() {
            if row.str("campaign")? == *inputs.index.campaign() {
                groups.push(row.str("group")?.into_owned());
            }
        }
        let mut art_sets = vec![];
        for row in tables.table("campaign_character_art_sets")?.rows() {
            if groups.contains(&row.str("group")?.into_owned()) {
                art_sets.push(ArtSet {
                    id: row.str("art_set_id")?.into_owned(),
                    agent: row.str("agent_type")?.into_owned(),
                    is_male: row.bool("is_male")?,
                    faction: row.str("faction")?.into_owned(),
                    subculture: row.str("subculture")?.into_owned(),
                    culture: row.str("culture")?.into_owned(),
                });
            }
        }

        let mut bodyguards = HashMap::new();
        for faction in plan.factions() {
            bodyguards.insert(faction.key().to_owned(), bodyguard(tables, faction, false)?);
        }

        let (family_ids, entry_family_ids) = family_ids(inputs.index, plan, forces);
        Ok(Self {
            family_ids,
            entry_family_ids,
            start_year: super::armies::start_year(inputs)?,
            inputs,
            forces,
            posts,
            sight,
            rows,
            traits,
            agents,
            art_sets,
            bodyguards,
        })
    }

    /// Writes the characters of a faction.
    ///
    /// # Errors
    ///
    /// Returns an error if a table, column or layout is missing.
    pub fn write_character_array(&self, faction: &FactionEntry) -> Result<RecordNode> {
        let mut array = self.inputs.layout.builder(CHARACTER_ARRAY_PATH)?;
        for (position, character) in self.forces.characters().iter().enumerate() {
            if character.faction().key() == faction.key() {
                array.push_group()?.set_record("CHARACTER", Some(self.write_character(position, character)?))?;
            }
        }
        array.build()
    }

    /// Writes a character: its position, ids, details and the post it holds.
    fn write_character(&self, position: usize, character: &PlannedCharacter) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let faction = character.faction();
        let info = self.info(character)?;
        let (move_points, line_of_sight) = self.agents.get(&info.agent).copied().unwrap_or_default();
        let (world, cell) = self.position(character)?;
        let force = character.force().map(|force| &self.forces.forces()[force]);
        let is_leader = character.entry().is_some_and(|entry| faction_leader(self.inputs.index, faction.key()) == Some(entry));

        let mut coordinates = Coordinates2DNode::default();
        coordinates.set_x(world.0);
        coordinates.set_y(world.1);
        let mut locomotable = child(layout, CHARACTER_PATH, "LOCOMOTABLE")?;
        locomotable.group()?
            .set("^+1", NodeType::Coord2d(coordinates))?
            .set_u32("^+2", u32::from(cell.x()))?
            .set_u32("^+3", u32::from(cell.y()))?
            .set_f32("^+4", PI)?
            .set_u32("^+6", move_points)?
            .set_u32("^+7", move_points)?;

        let override_unit = character.entry().map(|entry| entry.override_general_unit().as_str()).unwrap_or_default();
        let mut record = layout.builder(CHARACTER_PATH)?;
        record.group()?
            .set_record("LOCOMOTABLE", Some(locomotable.build()?))?
            .set_u32("LOCOMOTABLE+1", character.id())?
            .set_record("CHARACTER_DETAILS", Some(self.write_details(&self.subject(position, character, &info), &info)?))?
            .set_str("CHARACTER_DETAILS+1", &info.agent)?
            .set_u32("CHARACTER_DETAILS+2", force.map(|force| force.id()).unwrap_or_default())?
            .set_u32("CHARACTER_DETAILS+3", force.and_then(|force| force.commander_unit_id()).unwrap_or_default())?
            .set_u32("CHARACTER_DETAILS+6", if is_leader { self.posts.leader_post(faction.key()).unwrap_or_default() } else { 0 })?
            .set_f32("LINE_OF_SIGHT+4", line_of_sight)?
            .set_str("LINE_OF_SIGHT+16", override_unit)?
            .set_u32("LINE_OF_SIGHT+18", if is_leader { LEADER_VALUE } else { 0 })?
            .set_bool("LINE_OF_SIGHT+19", is_leader)?
            .set_record("LINE_OF_SIGHT", Some(write_line_of_sight(layout, CHARACTER_PATH, self.sight.grid(), self.vision(character)?.as_ref())?))?;
        record.build()
    }

    /// Returns what the details of a character of the world are written from.
    fn subject<'r>(&'r self, position: usize, character: &'r PlannedCharacter, info: &CharacterInfo) -> DetailsSubject<'r> {
        let faction = character.faction();
        let bodyguard = self.bodyguards.get(faction.key()).cloned().flatten().filter(|_| info.agent != COLONEL_AGENT && character.force().is_none());
        DetailsSubject {
            path: DETAILS_PATH,
            faction,
            entry: character.entry(),
            character_id: character.id(),
            family_id: self.family_ids[position],
            death: None,
            bodyguard,
        }
    }

    /// Writes the details of a character: names, birth (and death), traits, abilities, skills, art set and bodyguard.
    fn write_details(&self, subject: &DetailsSubject, info: &CharacterInfo) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let (path, faction) = (subject.path, subject.faction);
        let row = info.row;

        let mut birth = child(layout, path, "DATE")?;
        birth.group()?.set_u32("^+1", info.birth_year)?;

        let mut traits = child(layout, &format!("{path}/TRAITS"), "TRAIT")?;
        let character_traits = subject.entry.and_then(|entry| self.traits.get(entry.id())).map(Vec::as_slice).unwrap_or_default();
        for (key, points) in character_traits {
            traits.push_group()?
                .set_str("^+1", key)?
                .set_i32("^+2", i32::try_from(*points).unwrap_or_default())?;
        }
        let mut traits_record = child(layout, path, "TRAITS")?;
        traits_record.group()?.set_record("TRAIT", Some(traits.build()?))?;

        let mut abilities = child(layout, path, "AGENT_ABILITIES")?;
        for (ability, enabled) in ABILITIES.iter().zip(agent_abilities(self.inputs.tables, &info.agent)?) {
            abilities.push_group()?.set_str("^+1", ability)?.set_bool("^+2", enabled)?;
        }
        let mut attributes = child(layout, path, "AGENT_ATTRIBUTES")?;
        for (attribute, value) in ATTRIBUTES.iter().zip(agent_attributes(self.inputs.tables, &info.agent)?) {
            attributes.push_group()?.set_str("^+1", attribute)?.set_i32("^+2", i32::try_from(value).unwrap_or_default())?;
        }

        let mut skills = child(layout, path, "CAMPAIGN_SKILLS")?;
        skills.group()?
            .set_str("^+2", &skill_set(self.inputs.tables, self.inputs.index, &info.agent, faction)?.unwrap_or_default())?
            .set_bool("^+4", info.is_male)?;

        let seed = if subject.character_id != 0 { subject.character_id } else { subject.family_id };
        let art_set = self.art_set(faction, info, seed);
        let mut art_set_info = child(layout, path, "CHARACTER_DETAILS_ART_SET_INFO")?;
        art_set_info.group()?
            .set_str("^+1", faction.key())?
            .set_str("^+2", faction.culture())?
            .set_str("^+3", faction.subculture())?
            .set_str("^+4", art_set_agent(&info.agent))?
            .set_str("^+5", art_set)?;

        let mut record = layout.builder(path)?;
        let group = record.group()?;
        if let Some((year, code)) = subject.death {
            let mut death = child(layout, path, "DATE@2")?;
            death.group()?.set_u32("^+1", year)?;
            group.set_record("DATE@2", Some(death.build()?))?.set_i32("DATE@2+1", code)?;
        }
        group.set_record("TRAITS", Some(traits_record.build()?))?
            .set_record("CHARACTER_NAME", Some(self.write_name(path, row)?))?
            .set_record("DATE", Some(birth.build()?))?
            .set_str("DATE@2+2", faction.key())?
            .set_u32("DATE@2+3", subject.character_id)?
            .set_record("AGENT_ABILITIES", Some(abilities.build()?))?
            .set_record("AGENT_ATTRIBUTES", Some(attributes.build()?))?
            .set_bool("AGENT_ATTRIBUTES+3", info.is_male)?
            .set_str("AGENT_ATTRIBUTES+6", &info.agent)?
            .set_record("CAMPAIGN_SKILLS", Some(skills.build()?))?
            .set_record("CHARACTER_DETAILS_ART_SET_INFO", Some(art_set_info.build()?))?
            .set_record("GENERAL_BODYGUARD_DETAILS", Some(self.write_bodyguard(path, subject.bodyguard.as_ref())?))?
            .set_u32("GENERAL_BODYGUARD_DETAILS+1", subject.family_id)?
            .set_str("GENERAL_BODYGUARD_DETAILS+6", &format!("{art_set}{ART_SET_SUFFIX}"))?
            .set_str("GENERAL_BODYGUARD_DETAILS+8", faction.key())?;

        if let (Some(row), Some(entry)) = (row, subject.entry) {
            let ambition = if subject.death.is_some() { 0 } else { u32::try_from(row.int("ambition")?).unwrap_or_default() };
            record.group()?
                .set_u32("CAMPAIGN_EFFECT_BUNDLES+3", ambition)?
                .set_str("CAMPAIGN_EFFECT_BUNDLES+4", &row.str("political_party")?)?
                .set_bool("GENERAL_BODYGUARD_DETAILS+2", row.bool("immortal")?)?
                .set_i32("GENERAL_BODYGUARD_DETAILS+4", i32::try_from(loyalty(entry)).unwrap_or_default())?;
        }
        record.build()
    }

    /// Writes the names of a character (`CHARACTER_NAME`) under a parent record. See [`write_character_name`].
    pub(super) fn write_name(&self, parent: &str, row: Option<&Row>) -> Result<RecordNode> {
        write_character_name(self.inputs.layout, parent, row)
    }

    /// Writes the bodyguard of a character: the faction's default one for generals without a force, empty otherwise
    /// (a general's force holds its bodyguard as a unit).
    ///
    /// # Arguments
    ///
    /// * `details_path` - Path of the character's details.
    /// * `default` - The bodyguard, if the character keeps one.
    fn write_bodyguard(&self, details_path: &str, default: Option<&Bodyguard>) -> Result<RecordNode> {
        let path = format!("{details_path}/GENERAL_BODYGUARD_DETAILS");
        let mut record = self.inputs.layout.builder(&path)?;
        let Some(default) = default else {
            record.group()?.set_str("^+1", "")?;
            return record.build();
        };

        // The unit's details are only there when the character keeps its bodyguard.
        let bonuses = self.inputs.layout.default_record(&child_path(&path, "UNIT_STATISTICS_BONUSES"))?;
        record.group()?
            .set_str("^+1", default.unit())?
            .set_u32("^+2", default.men())?
            .set_u32("^+3", default.men())?
            .set_u32("^+4", 0)?
            .set_u32("^+5", 0)?
            .set_f32("^+6", 0.0)?
            .set_record("UNIT_STATISTICS_BONUSES", Some(bonuses))?;
        record.build()
    }

    /// Returns the family member id of a faction's progenitor (`start_pos_characters.progenitor`, living or dead), or
    /// 0 if it has none.
    pub fn progenitor_family_id(&self, faction: &FactionEntry) -> u32 {
        self.inputs.index.characters_of(faction.key())
            .find(|entry| entry.progenitor())
            .and_then(|entry| self.entry_family_ids.get(entry.id()))
            .copied()
            .unwrap_or_default()
    }

    /// Returns what a character sees: a disk of its agent's sight radius around it. Generals without a force don't
    /// see anything.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn vision(&self, character: &PlannedCharacter) -> Result<Option<Vision>> {
        let agent = self.info(character)?.agent;
        if agent == GENERAL_AGENT && character.force().is_none() {
            return Ok(None);
        }
        let (_, line_of_sight) = self.agents.get(&agent).copied().unwrap_or_default();
        let (position, _) = self.position(character)?;
        Ok(Some(self.sight.grid().disk(position, line_of_sight)))
    }

    /// Returns the planned characters of a faction.
    pub fn characters_of<'s>(&'s self, faction: &FactionEntry) -> impl Iterator<Item = &'s PlannedCharacter<'a>> + 's {
        self.forces.characters_of(faction.key())
    }

    /// Returns the runtime id of a faction's leader, if it has one.
    pub fn leader_id(&self, faction: &FactionEntry) -> Option<u32> {
        let leader = faction_leader(self.inputs.index, faction.key())?;
        self.forces.characters_of(faction.key()).find(|character| character.entry() == Some(leader)).map(|character| character.id())
    }

    /// Returns the `start_pos_characters` row of a character, `None` for colonels.
    pub(super) fn row(&self, character: &PlannedCharacter) -> Option<&Row<'a>> {
        character.entry().and_then(|entry| self.rows.get(entry.id()))
    }

    /// Returns the agent type, gender, birth year and table row of a character.
    fn info<'r>(&'r self, character: &PlannedCharacter) -> Result<CharacterInfo<'r>> {
        let Some(entry) = character.entry() else {
            return Ok(CharacterInfo { agent: COLONEL_AGENT.to_owned(), is_male: true, birth_year: self.start_year.saturating_sub(COLONEL_AGE), row: None });
        };
        let row = self.rows.get(entry.id());
        Ok(CharacterInfo {
            agent: entry.agent().to_owned(),
            is_male: row.map(|row| row.bool("is_male")).transpose()?.unwrap_or(true),
            birth_year: u32::try_from(birth_year(i64::from(self.start_year), entry)).unwrap_or_default(),
            row,
        })
    }

    /// Returns the art set of a character: one of the campaign's art sets for its agent type and gender, of its
    /// faction, subculture (sets without faction) or culture (sets without faction and subculture). The game draws
    /// one at random; this takes them in turns by `seed` (the character's runtime id, or family id for the dead).
    fn art_set(&self, faction: &FactionEntry, info: &CharacterInfo, seed: u32) -> &str {
        let candidates = self.art_sets.iter()
            .filter(|art_set| art_set.agent == art_set_agent(&info.agent) && art_set.is_male == info.is_male)
            .filter(|art_set| art_set.faction == *faction.key()
                || (art_set.faction.is_empty() && art_set.subculture == *faction.subculture())
                || (art_set.faction.is_empty() && art_set.subculture.is_empty() && art_set.culture == *faction.culture()))
            .collect::<Vec<_>>();
        candidates.get(seed as usize % candidates.len().max(1)).map(|art_set| art_set.id.as_str()).unwrap_or_default()
    }

    /// Returns the world position and cell of a character.
    ///
    /// Colonels stand on their settlement. Table characters stand on their starting cell, their linked settlement,
    /// their faction's first general (factions without capital), or off the map.
    pub(super) fn position(&self, character: &PlannedCharacter) -> Result<((f32, f32), HexCell)> {
        let settlement_cell = |region: &str| self.inputs.settlements.iter()
            .find(|settlement| settlement.region() == region)
            .and_then(|settlement| settlement.slots().first())
            .and_then(|slot| slot.position());
        let at = |cell: Option<HexCell>| cell.map(|cell| (cell.world_position(), cell)).unwrap_or_default();

        let Some(entry) = character.entry() else {
            let region = character.force().and_then(|force| self.forces.forces()[force].region().clone()).unwrap_or_default();
            return Ok(at(settlement_cell(&region)));
        };

        Ok(match placement(self.inputs.tables, self.inputs.index, entry)? {
            Placement::Cell(cell) => at(Some(cell)),
            Placement::Settlement(region) => at(settlement_cell(&region)),
            Placement::FirstArmy => {
                let first_general = self.forces.characters_of(character.faction().key())
                    .find(|other| other.force().is_some() && other.entry().is_some_and(|other| other.id() != entry.id()));
                match first_general {
                    Some(general) => self.position(general)?,
                    None => Default::default(),
                }
            },
            Placement::OffMap => Default::default(),
        })
    }
}

/// Returns the agent type whose art sets a character uses: colonels look like generals.
fn art_set_agent(agent: &str) -> &str {
    if agent == COLONEL_AGENT { GENERAL_AGENT } else { agent }
}

/// Returns the family member ids: of every planned character (by position in the plan), and of every table
/// character, living or dead (by `start_pos_characters` id). The counter numbers each faction's table characters
/// (the living ones, then the dead ones, which only appear in the family tree), then the colonels.
fn family_ids(index: &CampaignIndex, plan: &WorldPlan, forces: &ForcePlan) -> (Vec<u32>, HashMap<String, u32>) {
    let mut ids = vec![0; forces.characters().len()];
    let mut by_entry = HashMap::new();
    let mut next_id = 1;
    for faction in plan.factions() {
        for (position, character) in forces.characters().iter().enumerate() {
            if let Some(entry) = character.entry().filter(|_| character.faction().key() == faction.key()) {
                ids[position] = next_id;
                by_entry.insert(entry.id().to_owned(), next_id);
                next_id += 1;
            }
        }
        for entry in index.characters_of(faction.key()).filter(|entry| !entry.death_type().is_empty()) {
            by_entry.insert(entry.id().to_owned(), next_id);
            next_id += 1;
        }
    }
    for (position, character) in forces.characters().iter().enumerate() {
        if character.entry().is_none() {
            ids[position] = next_id;
            next_id += 1;
        }
    }
    (ids, by_entry)
}

/// Writes the names of a character (`CHARACTER_NAME`) under a parent record: one block per name kind, with the name's
/// localisation key if it has one. Nameless characters get the same blocks, all of kind 0.
///
/// # Arguments
///
/// * `layout` - Record layouts of the game.
/// * `parent` - Path of the record holding the names.
/// * `row` - The character's `start_pos_characters` row, or `None` for a nameless character.
///
/// # Errors
///
/// Returns an error if the layout lacks the records or the row a name column.
pub(crate) fn write_character_name(layout: &StartposLayout, parent: &str, row: Option<&Row>) -> Result<RecordNode> {
    let names_path = format!("{parent}/CHARACTER_NAME/NAMES_BLOCK");
    let mut blocks = child(layout, &format!("{parent}/CHARACTER_NAME"), "NAMES_BLOCK")?;
    for (kind, column) in NAME_COLUMNS.iter().enumerate() {
        let name = row.map(|row| row.str(column)).transpose()?.unwrap_or_default();
        let mut localisation = child(layout, &names_path, "CAMPAIGN_LOCALISATION")?;
        if name.is_empty() {
            localisation.group()?.set_str("^+1", "")?;
        } else {
            localisation.group()?.set_str("^+1", &format!("{NAME_KEY_PREFIX}{name}"))?.unset("^+2")?;
        }
        blocks.push_group()?
            .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?
            .set_i32("CAMPAIGN_LOCALISATION+1", if row.is_some() { i32::try_from(kind).unwrap_or_default() } else { 0 })?;
    }

    let mut record = child(layout, parent, "CHARACTER_NAME")?;
    record.group()?.set_record("NAMES_BLOCK", Some(blocks.build()?))?;
    record.build()
}
