//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Extraction of the character tables: characters (living and dead), traits, settlement links, family relationships,
//! the units of their forces and their horde details.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::regions::Regions;
use super::{FactionSource, Ids, Source, TableRows, FACTION_PATH};
use crate::startpos::coords::HexCell;
use crate::startpos::esf_nav::{as_record, as_str, as_u32};
use crate::startpos::layout::{child_path, GroupView};
use crate::startpos::tables::StartposTables;
use crate::startpos::world::writer::characters::family::DEATH_TYPE_CODES;
use crate::startpos::world::NAME_COLUMNS;
use crate::startpos::world::writer::characters::{COLONEL_AGENT, NAME_KEY_PREFIX};
use crate::startpos::world::writer::HORDE_BUILDING_COLUMNS;

#[cfg(test)] #[path = "characters_test.rs"] mod characters_test;

/// Path of a character.
const CHARACTER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/CHARACTER_ARRAY/CHARACTER";

/// Path of a force.
const FORCE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION/ARMY_ARRAY/MILITARY_FORCE";

/// Path of a family member.
const MEMBER_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FAMILY_TREE/FAMILY_MEMBER";

/// Name of the family member records.
const FAMILY_MEMBER: &str = "FAMILY_MEMBER";

/// Ministerial position of faction leaders.
const FACTION_LEADER: &str = "faction_leader";

/// Relationships of `start_pos_family_relationships`.
const CHILD_RELATIONSHIP: &str = "child";
const SPOUSE_RELATIONSHIP: &str = "spouse";

/// Share of a unit's men the tables give in percent.
const FULL_UNIT: u32 = 100;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A family member of the family tree (`WORLD/FAMILY_TREE`).
#[derive(Debug, Clone, Default)]
struct Member<'a> {
    faction: Option<&'a str>,
    details: Option<&'a RecordNode>,
    spouse: u32,
    former_spouses: Vec<u32>,
    children: Vec<u32>,
    adopted_children: Vec<u32>,
    bastard: bool,
    progenitor: bool,
}

/// What the tables keep of a character's details (`CHARACTER_DETAILS`).
struct Details<'a> {
    group: GroupView<'a>,
    names: Vec<String>,
    birth_year: u32,
    death: Option<(u32, i32)>,
    traits: Vec<(String, i32)>,
}

/// A table character found in the startpos, with its new row id.
struct Character<'a> {
    id: String,
    faction: &'a FactionSource<'a>,
    details: Details<'a>,
    family_id: u32,
    group: Option<GroupView<'a>>,
}

/// What tells where characters start: the settlements, and the first army of each faction without capital.
struct Placements<'a, 's> {
    source: &'s Source<'a>,
    regions: &'s Regions,
    army_cells: HashMap<u32, HexCell>,
}

/// The tables being extracted.
struct CharacterTables {
    characters: TableRows,
    traits: TableRows,
    settlements: TableRows,
    family: TableRows,
    land_units: TableRows,
    naval_units: TableRows,
    hordes: TableRows,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Extracts the character tables.
pub(super) fn extract(source: &Source, tables: &StartposTables, regions: &Regions, ids: &mut Ids) -> Result<Vec<(&'static str, TableRows)>> {
    let mut trait_levels = HashMap::new();
    for row in tables.table("character_trait_levels")?.rows() {
        trait_levels.insert((row.str("trait")?.into_owned(), row.int("threshold_points")?), row.str("key")?.into_owned());
    }
    let members = family_members(source);

    let mut characters = vec![];
    for faction in &source.factions {
        for (_, group) in source.layout.read_items(&child_path(FACTION_PATH, "CHARACTER_ARRAY"), &faction.group, "CHARACTER_ARRAY", "CHARACTER")? {
            if group.str("CHARACTER_DETAILS+1") == Some(COLONEL_AGENT) {
                continue;
            }
            let Some(record) = group.record("CHARACTER_DETAILS") else { continue };
            let details = read_details(source, &child_path(CHARACTER_PATH, "CHARACTER_DETAILS"), record)?;

            // Wives moved to their husband's faction come from their own.
            let origin = details.group.str("GENERAL_BODYGUARD_DETAILS+8").and_then(|key| source.faction_by_key(key)).unwrap_or(faction);
            let family_id = details.group.u32("GENERAL_BODYGUARD_DETAILS+1").unwrap_or_default();
            characters.push(Character { id: ids.next(), faction: origin, details, family_id, group: Some(group) });
        }
    }
    let mut family_ids = members.keys().copied().collect::<Vec<_>>();
    family_ids.sort_unstable();
    for family_id in family_ids {
        let member = &members[&family_id];
        let (Some(faction), Some(record)) = (member.faction.and_then(|key| source.faction_by_key(key)), member.details) else { continue };
        let details = read_details(source, &child_path(MEMBER_PATH, "CHARACTER_DETAILS"), record)?;
        characters.push(Character { id: ids.next(), faction, details, family_id, group: None });
    }

    let mut output = CharacterTables {
        characters: TableRows::new(&[
            "ID", "faction", "Name", "Surname", "clan_name", "other_name", "Age", "Type", "startx", "starty", "ministerial_position",
            "immortal", "override_general_unit", "is_male", "loyalty", "ambition", "political_party", "death_type", "turns_died_before_start",
            "progenitor",
        ]),
        traits: TableRows::new(&["id", "character_id", "trait_level"]),
        settlements: TableRows::new(&["character", "settlement"]),
        family: TableRows::new(&["character", "related_to", "relationship", "bastard", "adopted"]),
        land_units: TableRows::new(&["id", "unit_type", "general", "soldiers"]),
        naval_units: TableRows::new(&["ID", "unit_type", "admiral"]),
        hordes: TableRows::new(&std::iter::once("general").chain(HORDE_BUILDING_COLUMNS).chain(["starting_dev_points"]).collect::<Vec<_>>()),
    };

    let placements = Placements { source, regions, army_cells: first_army_cells(source, &characters)? };
    let by_runtime_id = characters.iter()
        .filter_map(|character| Some((character.group.as_ref()?.u32("LOCOMOTABLE+1")?, character)))
        .collect::<HashMap<_, _>>();
    for character in &characters {
        push_character(&placements, &mut output, character, members.get(&character.family_id))?;
        for (key, points) in &character.details.traits {
            if let Some(level) = trait_levels.get(&(key.to_owned(), i64::from(*points))) {
                output.traits.push(vec![ids.next(), character.id.to_owned(), level.to_owned()]);
            }
        }
    }

    for faction in &source.factions {
        push_forces(source, &mut output, faction, &by_runtime_id, ids)?;
    }

    let by_family_id = characters.iter().map(|character| (character.family_id, character.id.as_str())).collect::<HashMap<_, _>>();
    push_family(&mut output, &members, &by_family_id);

    Ok(vec![
        ("start_pos_characters", output.characters),
        ("start_pos_character_traits", output.traits),
        ("start_pos_character_to_settlements", output.settlements),
        ("start_pos_family_relationships", output.family),
        ("start_pos_land_units", output.land_units),
        ("start_pos_naval_units", output.naval_units),
        ("start_pos_horde_details", output.hordes),
    ])
}

/// Adds the row of a character, and its settlement link if it starts in one.
fn push_character(placements: &Placements, output: &mut CharacterTables, character: &Character, member: Option<&Member>) -> Result<()> {
    let start_year = placements.source.start_year();
    let details = &character.details.group;
    let death_type = character.details.death.and_then(|(_, code)| DEATH_TYPE_CODES.iter().find(|(_, death_code)| *death_code == code)).map(|(death_type, _)| *death_type);
    let (age, died_before) = match character.details.death {
        Some((year, _)) => (i64::from(year) - i64::from(character.details.birth_year), (i64::from(start_year) - i64::from(year)).to_string()),
        None => (i64::from(start_year) - i64::from(character.details.birth_year), String::new()),
    };

    let mut cell = HexCell::new(0, 0);
    let mut ministerial_position = "";
    let mut override_unit = "";
    if let Some(group) = &character.group {
        ministerial_position = if group.u32("CHARACTER_DETAILS+6").unwrap_or_default() != 0 { FACTION_LEADER } else { "" };
        override_unit = group.str("LINE_OF_SIGHT+16").unwrap_or_default();

        cell = character_cell(placements.source, group)?;

        // Characters standing on one of their faction's settlements start there, and the ones of factions without
        // capital standing with their faction's first army start with it. The others start on their cell.
        let commands_force = group.u32("CHARACTER_DETAILS+2").unwrap_or_default() != 0;
        if let Some(settlement) = placements.regions.settlement_at(cell, character.faction.id) {
            output.settlements.push(vec![character.id.to_owned(), settlement.id.to_owned()]);
            cell = HexCell::new(0, 0);
        } else if !commands_force && placements.army_cells.get(&character.faction.id) == Some(&cell) {
            cell = HexCell::new(0, 0);
        }
    }

    let mut row = vec![character.id.to_owned(), character.faction.table_id.to_owned()];
    row.extend(character.details.names.iter().cloned());
    row.extend([
        age.to_string(),
        details.str("AGENT_ATTRIBUTES+6").unwrap_or_default().to_owned(),
        cell.x().to_string(),
        cell.y().to_string(),
        ministerial_position.to_owned(),
        details.bool("GENERAL_BODYGUARD_DETAILS+2").unwrap_or_default().to_string(),
        override_unit.to_owned(),
        details.bool("AGENT_ATTRIBUTES+3").unwrap_or(true).to_string(),
        details.i32("GENERAL_BODYGUARD_DETAILS+4").unwrap_or_default().to_string(),
        details.u32("CAMPAIGN_EFFECT_BUNDLES+3").unwrap_or_default().to_string(),
        details.str("CAMPAIGN_EFFECT_BUNDLES+4").unwrap_or_default().to_owned(),
        death_type.unwrap_or_default().to_owned(),
        died_before,
        member.is_some_and(|member| member.progenitor).to_string(),
    ]);
    output.characters.push(row);
    Ok(())
}

/// Returns the cell a character stands on.
fn character_cell(source: &Source, character: &GroupView) -> Result<HexCell> {
    let Some(locomotable) = character.record("LOCOMOTABLE") else { return Ok(HexCell::new(0, 0)) };
    let locomotable = source.layout.read_single(&child_path(CHARACTER_PATH, "LOCOMOTABLE"), locomotable)?;
    let coordinate = |name: &str| locomotable.u32(name).and_then(|value| u16::try_from(value).ok()).unwrap_or_default();
    Ok(HexCell::new(coordinate("^+2"), coordinate("^+3")))
}

/// Returns the cell of the first general commanding a force of each faction without capital, by faction runtime id.
fn first_army_cells(source: &Source, characters: &[Character]) -> Result<HashMap<u32, HexCell>> {
    let mut cells = HashMap::new();
    for character in characters {
        let Some(group) = &character.group else { continue };
        let faction = character.faction;
        if faction.group.u32("MORGUE+3").unwrap_or_default() != 0 || cells.contains_key(&faction.id) || group.u32("CHARACTER_DETAILS+2").unwrap_or_default() == 0 {
            continue;
        }
        cells.insert(faction.id, character_cell(source, group)?);
    }
    Ok(cells)
}

/// Adds the units and horde details of a faction's general forces. Garrisons and the commander's own unit (its
/// bodyguard or flagship) aren't rows.
fn push_forces(source: &Source, output: &mut CharacterTables, faction: &FactionSource, characters: &HashMap<u32, &Character>, ids: &mut Ids) -> Result<()> {
    let layout = source.layout;
    let container_path = child_path(FORCE_PATH, "UNIT_CONTAINER");
    let unit_path = format!("{container_path}/UNITS_ARRAY/UNIT");
    for (_, force) in layout.read_items(&child_path(FACTION_PATH, "ARMY_ARRAY"), &faction.group, "ARMY_ARRAY", "MILITARY_FORCE")? {
        let is_garrison = force.u32("UNIT_CONTAINER+11").unwrap_or_default() != 0 || force.u32("UNIT_CONTAINER+12").unwrap_or_default() != 0;
        let Some(character) = force.u32("^+2").and_then(|id| characters.get(&id)).filter(|_| !is_garrison) else { continue };
        let own_unit = character.group.as_ref().and_then(|group| group.u32("CHARACTER_DETAILS+3")).unwrap_or_default();
        let navy = force.bool("UNIT_CONTAINER+13").unwrap_or_default();

        let Some(container) = force.record("UNIT_CONTAINER") else { continue };
        let container = layout.read_single(&container_path, container)?;
        for (_, unit) in layout.read_items(&child_path(&container_path, "UNITS_ARRAY"), &container, "UNITS_ARRAY", "UNIT")? {
            if unit.u32("^+1") == Some(own_unit) {
                continue;
            }
            let key = match unit.record("UNIT_RECORD_KEY") {
                Some(record) => layout.read_single(&child_path(&unit_path, "UNIT_RECORD_KEY"), record)?.str("^+1").unwrap_or_default().to_owned(),
                None => continue,
            };
            if navy {
                output.naval_units.push(vec![ids.next(), key, character.id.to_owned()]);
            } else {
                let soldiers = soldiers(unit.u32("COMMANDER_DETAILS+1").unwrap_or_default(), unit.u32("COMMANDER_DETAILS+2").unwrap_or_default());
                output.land_units.push(vec![ids.next(), key, character.id.to_owned(), soldiers.to_string()]);
            }
        }

        if let Some(horde) = force.record("MILITARY_FORCE_HORDE_DETAILS") {
            if let Some(row) = horde_row(source, horde, &character.id)? {
                output.hordes.push(row);
            }
        }
    }
    Ok(())
}

/// Returns the percentage of a unit's men present: the lowest one the build turns into the same number of men.
fn soldiers(men: u32, max_men: u32) -> u32 {
    if max_men == 0 || men >= max_men {
        return FULL_UNIT;
    }
    (men * FULL_UNIT).div_ceil(max_men).max(1)
}

/// Builds the `start_pos_horde_details` row of a horde army: the building of each slot, and its development points
/// (stored plus one). Hordes without buildings nor points don't need a row.
fn horde_row(source: &Source, horde: &RecordNode, general: &str) -> Result<Option<Vec<String>>> {
    let layout = source.layout;
    let horde_path = child_path(FORCE_PATH, "MILITARY_FORCE_HORDE_DETAILS");
    let slot_manager_path = child_path(&horde_path, "MILITARY_FORCE_SLOT_MANAGER");
    let slot_path = format!("{slot_manager_path}/SLOT_BLOCK/MILITARY_FORCE_SLOT");
    let building_manager_path = child_path(&slot_path, "MILITARY_FORCE_BUILDING_MANAGER");
    let horde = layout.read_single(&horde_path, horde)?;

    let mut buildings = vec![String::new(); HORDE_BUILDING_COLUMNS.len()];
    if let Some(manager) = horde.record("MILITARY_FORCE_SLOT_MANAGER") {
        let manager = layout.read_single(&slot_manager_path, manager)?;
        for (index, (_, slot)) in layout.read_items(&child_path(&slot_manager_path, "SLOT_BLOCK"), &manager, "SLOT_BLOCK", "MILITARY_FORCE_SLOT")?.into_iter().enumerate() {
            let Some(building_manager) = slot.record("MILITARY_FORCE_BUILDING_MANAGER") else { continue };
            let building_manager = layout.read_single(&building_manager_path, building_manager)?;
            let Some(base) = building_manager.record("BUILDING_BASE") else { continue };
            let key = layout.read_single(&child_path(&building_manager_path, "BUILDING_BASE"), base)?.str("^+2").unwrap_or_default().to_owned();
            if let Some(column) = buildings.get_mut(index) {
                *column = key;
            }
        }
    }

    let development = match horde.record("MILITARY_FORCE_DEVELOPMENT") {
        Some(record) => layout.read_single(&child_path(&horde_path, "MILITARY_FORCE_DEVELOPMENT"), record)?.u32("^+1").unwrap_or_default().saturating_sub(1),
        None => 0,
    };

    if development == 0 && buildings.iter().all(String::is_empty) {
        return Ok(None);
    }
    let mut row = vec![general.to_owned()];
    row.extend(buildings);
    row.push(development.to_string());
    Ok(Some(row))
}

/// Adds the family relationships between table characters: each parent's children (adopted or not), then each
/// couple, current or former, once.
fn push_family(output: &mut CharacterTables, members: &HashMap<u32, Member>, characters: &HashMap<u32, &str>) {
    let mut family_ids = members.keys().copied().collect::<Vec<_>>();
    family_ids.sort_unstable();

    for &parent_id in &family_ids {
        let (Some(parent), Some(member)) = (characters.get(&parent_id), members.get(&parent_id)) else { continue };
        let children = member.children.iter().map(|child| (child, false)).chain(member.adopted_children.iter().map(|child| (child, true)));
        for (child_id, adopted) in children {
            let Some(child) = characters.get(child_id) else { continue };
            let bastard = members.get(child_id).is_some_and(|child| child.bastard);
            output.family.push(vec![child.to_string(), parent.to_string(), CHILD_RELATIONSHIP.to_owned(), bastard.to_string(), adopted.to_string()]);
        }
    }

    for &member_id in &family_ids {
        let (Some(character), Some(member)) = (characters.get(&member_id), members.get(&member_id)) else { continue };
        for spouse_id in std::iter::once(&member.spouse).chain(&member.former_spouses).filter(|spouse| **spouse > member_id) {
            if let Some(spouse) = characters.get(spouse_id) {
                output.family.push(vec![character.to_string(), spouse.to_string(), SPOUSE_RELATIONSHIP.to_owned(), false.to_string(), false.to_string()]);
            }
        }
    }
}

/// Reads what the tables keep of a character's details.
fn read_details<'a>(source: &Source<'a>, path: &str, record: &'a RecordNode) -> Result<Details<'a>> {
    let layout = source.layout;
    let group = layout.read_single(path, record)?;
    let date_year = |key: &str| -> Result<Option<u32>> {
        group.record(key).map(|date| Ok(layout.read_single(&child_path(path, key), date)?.u32("^+1").unwrap_or_default())).transpose()
    };
    let birth_year = date_year("DATE")?.unwrap_or_default();
    let death_code = group.i32("DATE@2+1").unwrap_or_default();
    let death = if death_code != 0 { Some((date_year("DATE@2")?.unwrap_or_default(), death_code)) } else { None };

    let traits_path = child_path(path, "TRAITS");
    let mut traits = vec![];
    if let Some(record) = group.record("TRAITS") {
        let traits_group = layout.read_single(&traits_path, record)?;
        if let Some(list) = traits_group.record("TRAIT") {
            for entry in layout.read(&child_path(&traits_path, "TRAIT"), list)? {
                traits.push((entry.str("^+1").unwrap_or_default().to_owned(), entry.i32("^+2").unwrap_or_default()));
            }
        }
    }

    Ok(Details { names: names(source, path, &group)?, group, birth_year, death, traits })
}

/// Reads the names of a character, in [`NAME_COLUMNS`] order: each name block holds its kind, the column's position.
fn names(source: &Source, path: &str, details: &GroupView) -> Result<Vec<String>> {
    let layout = source.layout;
    let name_path = child_path(path, "CHARACTER_NAME");
    let block_path = child_path(&name_path, "NAMES_BLOCK");
    let mut names = vec![String::new(); NAME_COLUMNS.len()];
    let Some(record) = details.record("CHARACTER_NAME") else { return Ok(names) };
    let name = layout.read_single(&name_path, record)?;
    let Some(blocks) = name.record("NAMES_BLOCK") else { return Ok(names) };

    for block in layout.read(&block_path, blocks)? {
        let kind = block.i32("CAMPAIGN_LOCALISATION+1").and_then(|kind| usize::try_from(kind).ok()).unwrap_or_default();
        let Some(localisation) = block.record("CAMPAIGN_LOCALISATION") else { continue };
        let key = layout.read_single(&child_path(&block_path, "CAMPAIGN_LOCALISATION"), localisation)?.str("^+1").unwrap_or_default();
        if let (Some(name), Some(slot)) = (key.strip_prefix(NAME_KEY_PREFIX), names.get_mut(kind)) {
            *slot = name.to_owned();
        }
    }
    Ok(names)
}

/// Reads the members of the family tree, by family member id.
///
/// The family tree is a list without layout: its count, then each `FAMILY_MEMBER` with its children in a fixed order
/// (see the family tree writer): id, dead flag, the faction and details of the dead, a 0, mother, father, spouse,
/// marriage date, three counted lists (former spouses, children, adopted children), then flags, among them the
/// bastard (6th) and progenitor (11th) ones.
fn family_members<'a>(source: &Source<'a>) -> HashMap<u32, Member<'a>> {
    let Some(tree) = source.world.record("FAMILY_TREE") else { return HashMap::new() };
    tree.children().iter()
        .flatten()
        .filter_map(as_record)
        .filter(|record| record.name() == FAMILY_MEMBER)
        .filter_map(|record| read_member(&record.children().iter().flatten().collect::<Vec<_>>()))
        .collect()
}

/// Reads a family member from its children, or `None` if they don't have the expected shape.
fn read_member<'a>(nodes: &[&'a NodeType]) -> Option<(u32, Member<'a>)> {
    let mut nodes = nodes.iter().copied();
    let id = as_u32(nodes.next()?)?;
    let mut member = Member::default();
    if as_bool(nodes.next()?)? {
        member.faction = Some(as_str(nodes.next()?)?);
        member.details = Some(as_record(nodes.next()?)?);
    }

    // The 0 before the parents, the mother and the father, which the children lists give from the other side.
    nodes.nth(2)?;
    member.spouse = as_u32(nodes.next()?)?;
    nodes.next()?;
    member.former_spouses = read_list(&mut nodes)?;
    member.children = read_list(&mut nodes)?;
    member.adopted_children = read_list(&mut nodes)?;

    let flags = nodes.collect::<Vec<_>>();
    member.bastard = as_bool(flags.get(5)?)?;
    member.progenitor = as_bool(flags.get(10)?)?;
    Some((id, member))
}

/// Reads a counted list of ids: its length, then each id.
fn read_list<'a>(nodes: &mut impl Iterator<Item = &'a NodeType>) -> Option<Vec<u32>> {
    let count = as_u32(nodes.next()?)?;
    (0..count).map(|_| nodes.next().and_then(as_u32)).collect()
}

/// Returns the value of a boolean leaf.
fn as_bool(node: &NodeType) -> Option<bool> {
    match node {
        NodeType::Bool(node) => Some(*node.value()),
        _ => None,
    }
}
