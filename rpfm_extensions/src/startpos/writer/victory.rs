//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the victory missions the faction selection screen shows (`VICTORY_MISSION_OPTIONS`), from the campaign's
//! `victory_objectives.txt`.
//!
//! Each objective is a `CAMPAIGN_MISSION_OBJECTIVE` with its type id and a record of the type's values (none for
//! types without values). The ids come from the game's objective type enum, matched against Attila's startposes and
//! its name table; Thrones and TDD share them, while Rome 2 numbers the types its own way (from its startposes). Vanilla startposes have no layout for the types only mods use (The Dawnless Days' ones): their
//! records are built from the shapes the game writes for them.

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode, RecordNodeFlags};
use rpfm_lib::games::supported_games::KEY_ROME_2;

use super::{bool_node, f32_node, u32_node};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::missions::{MissionObjective, VictoryMission, VictoryObjectives};

#[cfg(test)] #[path = "victory_test.rs"] pub(crate) mod victory_test;

/// Path of the victory missions of each faction.
pub(crate) const OPTIONS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS";

/// Path of a victory mission.
pub(crate) const DEFINITION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION";

/// Path of the objectives and payload of a victory mission.
pub(crate) const OBJECTIVES_AND_PAYLOAD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION/CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD";

/// Path of an objective of a victory mission.
pub(crate) const OBJECTIVE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION/CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD/OBJECTIVES/CAMPAIGN_MISSION_OBJECTIVE";

/// Prefix of the localisation key of a mission's title.
const TITLE_KEY_PREFIX: &str = "missions_localised_title_";

/// Prefix of the localisation key of a mission's description.
const DESCRIPTION_KEY_PREFIX: &str = "missions_localised_description_";

/// Prefix of the record holding an objective's values.
pub(crate) const IMPL_PREFIX: &str = "CAMPAIGN_MISSION_OBJECTIVE_IMPL_";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the victory missions of every faction (`VICTORY_MISSION_OPTIONS`): one entry per faction, with its missions
/// grouped by kind of campaign.
///
/// # Arguments
///
/// * `layout` - Record layouts of the game.
/// * `game` - Key of the game, for its objective type ids.
/// * `objectives` - Victory objectives of the campaign, as `victory_objectives.txt` lists them.
///
/// # Errors
///
/// Returns an error if the layout lacks a record, or the game has no known id for an objective type.
pub fn write_victory_mission_options(layout: &StartposLayout, game: &str, objectives: &[VictoryObjectives]) -> Result<RecordNode> {
    let mut factions = Vec::<&str>::new();
    for objective in objectives {
        if !factions.contains(&objective.faction().as_str()) {
            factions.push(objective.faction());
        }
    }

    let types_path = child_path(OPTIONS_PATH, "VICTORY_MISSIONS_BY_TYPE_BLOCK");
    let missions_path = child_path(&types_path, "VICTORY_MISSIONS_BLOCK");
    let mut options = layout.builder(OPTIONS_PATH)?;
    for faction in factions {
        let mut types = layout.builder(&types_path)?;
        for faction_objectives in objectives.iter().filter(|objective| objective.faction() == faction) {
            let mut missions = layout.builder(&missions_path)?;
            for mission in faction_objectives.missions() {
                missions.push_group()?.set_record("CAMPAIGN_MISSION_DEFINITION", Some(write_mission(layout, game, mission)?))?;
            }
            types.push_group()?
                .set_u32("^+1", faction_objectives.campaign_type().id())?
                .set_record("VICTORY_MISSIONS_BLOCK", Some(missions.build()?))?;
        }
        options.push_group()?
            .set_str("^+1", faction)?
            .set_record("VICTORY_MISSIONS_BY_TYPE_BLOCK", Some(types.build()?))?;
    }
    options.build()
}

/// Writes a victory mission (`CAMPAIGN_MISSION_DEFINITION`): its key, issuer, texts and objectives. The payload of
/// victory missions (winning the game) is the layout's.
fn write_mission(layout: &StartposLayout, game: &str, mission: &VictoryMission) -> Result<RecordNode> {
    let localisation = |key: String| -> Result<RecordNode> {
        let mut record = layout.builder(&child_path(DEFINITION_PATH, "CAMPAIGN_LOCALISATION"))?;
        record.group()?.set_str("^+1", &key)?;
        record.build()
    };

    let mut objectives = layout.builder(&child_path(OBJECTIVES_AND_PAYLOAD_PATH, "OBJECTIVES"))?;
    for objective in mission.objectives() {
        objectives.push_group()?.set_record("CAMPAIGN_MISSION_OBJECTIVE", Some(write_objective(layout, game, objective)?))?;
    }
    let mut objectives_and_payload = layout.builder(OBJECTIVES_AND_PAYLOAD_PATH)?;
    objectives_and_payload.group()?.set_record("OBJECTIVES", Some(objectives.build()?))?;

    let mut definition = layout.builder(DEFINITION_PATH)?;
    definition.group()?
        .set_str("^+1", mission.key())?
        .set_str("^+2", mission.issuer())?
        .set_str("^+4", mission.key())?
        .set_record("CAMPAIGN_LOCALISATION", Some(localisation(format!("{TITLE_KEY_PREFIX}{}", mission.key()))?))?
        .set_record("CAMPAIGN_LOCALISATION@2", Some(localisation(format!("{DESCRIPTION_KEY_PREFIX}{}", mission.key()))?))?
        .set_record("CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD", Some(objectives_and_payload.build()?))?
        .set_str("SECONDARY_OBJECTIVES+4", mission.victory_type())?;
    definition.build()
}

/// Writes an objective (`CAMPAIGN_MISSION_OBJECTIVE`): its type id and the record with its values.
fn write_objective(layout: &StartposLayout, game: &str, objective: &MissionObjective) -> Result<RecordNode> {
    let id = objective_id(objective, game).ok_or_else(|| RLibError::StartposLayoutError(format!("no known {game} id for objective type {}", objective.type_key())))?;
    let name = objective_record_name(objective);
    let impl_key = format!("{IMPL_PREFIX}{name}");
    let impl_path = child_path(OBJECTIVE_PATH, &impl_key);

    let values = match objective {
        MissionObjective::StayHorde => None,
        MissionObjective::AttainFameLevel { .. } if layout.record(&impl_path).is_some() => Some(write_objective_values(layout, &impl_path, objective)?),
        _ if builtin_record(objective).is_some() => builtin_record(objective),
        MissionObjective::ConstructNBuildingsIncluding { total, buildings, faction } => {
            let mut children = vec![u32_node(u32::try_from(buildings.len()).unwrap_or(u32::MAX))];
            for building in buildings {
                children.push(NodeType::Ascii(building.to_owned()));
                children.push(bool_node(false));
            }
            children.extend([u32_node(0), u32_node(*total), u32_node(0), NodeType::Ascii(faction.to_owned())]);
            Some(layout.raw_record(&impl_path, children)?)
        },

        // Objectives with lists start with what the game resolves while playing (empty here), then the list's keys.
        MissionObjective::ReclaimNDifferentSettlementsIncluding { total, regions } => Some(layout.raw_record(&impl_path, keyed_list(vec![u32_node(*total)], regions))?),
        MissionObjective::AtLeastXPublicOrderInProvinces { total, provinces } => Some(layout.raw_record(&impl_path, keyed_list(vec![f32_node(*total)], provinces))?),
        MissionObjective::EnsureFactionsHaveNoMilitaryPresence { factions } => Some(layout.raw_record(&impl_path, keyed_list(vec![], factions))?),
        _ => Some(write_objective_values(layout, &impl_path, objective)?),
    };

    // Records the layout has no slot for go right after the type id, where the game puts every one.
    let mut record = layout.builder(OBJECTIVE_PATH)?;
    let group = record.group()?.set_u32("^+1", id)?;
    let has_slot = layout.record(OBJECTIVE_PATH).is_some_and(|objective| objective.has_slot(&impl_key));
    match values {
        Some(values) if has_slot => { group.set_record(&impl_key, Some(values))?; },
        Some(values) => {
            let mut built = record.build()?;
            let mut children = built.children().clone();
            if let Some(first) = children.first_mut() {
                first.insert(1.min(first.len()), NodeType::Record(Box::new(values)));
            }
            built.set_children(children);
            return Ok(built);
        },
        None => {},
    }
    record.build()
}

/// Builds the values record of the objective types vanilla Attila layouts don't have, in the version and shape TDD's
/// game-built startpos has them. `None` for the other types.
fn builtin_record(objective: &MissionObjective) -> Option<RecordNode> {
    let ascii = |value: &str| NodeType::Ascii(value.to_owned());
    let (version, children) = match objective {
        MissionObjective::HoldEntiretyOfNProvincesIncluding { total, provinces } => (1, vec![u32_node(*total), NodeType::AsciiArray(provinces.clone())]),
        MissionObjective::ConstructNBuildingsFrom { total, buildings, faction } => {
            let mut children = vec![u32_node(u32::try_from(buildings.len()).unwrap_or(u32::MAX))];
            for building in buildings {
                children.extend([ascii(building), bool_node(false)]);
            }
            children.extend([u32_node(*total), u32_node(0), ascii(faction)]);
            (3, children)
        },
        MissionObjective::ConstructNOfABuilding { total, building, faction } => (2, vec![u32_node(1), ascii(building), u32_node(0), u32_node(*total), ascii(faction)]),
        MissionObjective::HaveAtLeastXMoney { total } => (0, vec![u32_node(*total)]),
        MissionObjective::MaintainTradeWithNFactions { total } => (1, vec![u32_node(*total)]),
        MissionObjective::MakeAlliance { faction } => (3, vec![ascii(faction)]),
        MissionObjective::AttainFameLevel { level, .. } => (1, vec![u32_node(*level)]),
        _ => return None,
    };
    let mut record = RecordNode::default();
    record.set_name(format!("{IMPL_PREFIX}{}", objective_record_name(objective)));
    record.set_version(version);
    record.set_record_flags(RecordNodeFlags::IS_RECORD_NODE);
    record.set_children(vec![children]);
    Some(record)
}

/// Returns the leaves of an objective with a list of keys: its leading values, an empty list of resolved entries, and
/// the keys (count, then each key).
fn keyed_list(mut leading: Vec<NodeType>, keys: &[String]) -> Vec<NodeType> {
    leading.extend([u32_node(0), u32_node(u32::try_from(keys.len()).unwrap_or(u32::MAX))]);
    leading.extend(keys.iter().map(|key| NodeType::Ascii(key.to_owned())));
    leading
}

/// Writes the record with the values of an objective of a fixed shape.
fn write_objective_values(layout: &StartposLayout, path: &str, objective: &MissionObjective) -> Result<RecordNode> {
    let mut record = layout.builder(path)?;
    let group = record.group()?;
    match objective {
        MissionObjective::ReachSpecifiedDate { year, week } => {
            let mut date = layout.builder(&child_path(path, "DATE"))?;
            date.group()?.set_u32("^+1", *year)?.set_u32("^+2", *week)?;
            group.set_record("DATE", Some(date.build()?))?;
        },
        MissionObjective::ControlNProvincesIncluding { total, provinces } => {
            group.set_u32("^+1", *total)?.set_strings("^+2", provinces.clone())?;
        },
        MissionObjective::ResearchNTechsOfTypeX { total, categories, text } => {
            let mut localisation = layout.builder(&child_path(path, "CAMPAIGN_LOCALISATION"))?;
            localisation.group()?.set_str("^+1", text.as_deref().unwrap_or_default())?;
            group.set_u32("^+1", *total)?
                .set_strings("^+2", categories.clone())?
                .set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?;
        },
        MissionObjective::OwnNUnits { total, naval, units } => {
            group.set_u32("^+1", *total)?.set_bool("^+2", *naval)?.set_strings("^+5", units.clone())?;
        },
        MissionObjective::SubjugateFactions { total, faction } => {
            group.set_u32("^+1", *total)?.set_str("^+2", faction.as_deref().unwrap_or_default())?;
        },
        MissionObjective::AtLeastXReligionInAllProvinces { total, religion } => {
            group.set_f32("^+1", *total)?.set_str("^+2", religion)?;
        },
        MissionObjective::ControlNRegionsIncluding { total, regions } => {
            group.set_u32("^+1", *total)?.set_strings("^+2", regions.clone())?;
        },
        MissionObjective::IncomeAtLeastX { income } => {
            group.set_u32("^+1", *income)?;
        },
        MissionObjective::EliminateCharacterInBattle { faction } => {
            group.set_str("^+2", faction)?;
        },
        // Attila's fame objectives have no completion status.
        MissionObjective::AttainFameLevel { level, status } => {
            group.set_u32("^+1", *level)?;
            if layout.record(path).is_some_and(|values| values.has_slot("^+2")) {
                group.set_str("^+2", status.as_deref().unwrap_or_default())?;
            }
        },
        MissionObjective::Scripted { script_key, text } => {
            let mut localisation = layout.builder(&child_path(path, "CAMPAIGN_LOCALISATION"))?;
            localisation.group()?.set_str("^+1", text.as_deref().unwrap_or_default())?;
            group.set_record("CAMPAIGN_LOCALISATION", Some(localisation.build()?))?.set_str("CAMPAIGN_LOCALISATION+1", script_key)?;
        },
        MissionObjective::ControlNPortsIncluding { total, regions } => {
            group.set_u32("^+1", *total)?.set_strings("^+2", regions.clone())?;
        },
        MissionObjective::OwnAPortAdjoiningSeaRegionsIncluding { regions } => {
            group.set_strings("^+1", regions.clone())?;
        },

        // The game drops the settlements listed by loot and raze objectives: only their total is kept.
        MissionObjective::EarnXAmountFromBuildingWealth { total } |
        MissionObjective::MaintainNAlliances { total } |
        MissionObjective::ConfederateFactions { total } |
        MissionObjective::EarnXAmountFromRaiding { total } |
        MissionObjective::OwnNProvincesWithCulturalDominance { total } |
        MissionObjective::MaintainNClientStates { total } |
        MissionObjective::LootOrSackNDifferentSettlementsIncluding { total, .. } |
        MissionObjective::RazeOrSackNDifferentSettlementsIncluding { total, .. } => {
            group.set_u32("^+1", *total)?;
        },
        MissionObjective::HoldEntiretyOfNProvincesIncluding { .. } |
        MissionObjective::ConstructNBuildingsFrom { .. } |
        MissionObjective::ConstructNOfABuilding { .. } |
        MissionObjective::HaveAtLeastXMoney { .. } |
        MissionObjective::MaintainTradeWithNFactions { .. } |
        MissionObjective::MakeAlliance { .. } => {
            return Err(RLibError::StartposLayoutError(format!("{path}: built without a layout")));
        },
        MissionObjective::ConstructNBuildingsIncluding { .. } |
        MissionObjective::ReclaimNDifferentSettlementsIncluding { .. } |
        MissionObjective::AtLeastXPublicOrderInProvinces { .. } |
        MissionObjective::EnsureFactionsHaveNoMilitaryPresence { .. } |
        MissionObjective::StayHorde => {
            return Err(RLibError::StartposLayoutError(format!("{path}: lists have no fixed layout")));
        },
    }
    record.build()
}

/// Returns the type id of an objective in a game, if known.
pub(crate) fn objective_id(objective: &MissionObjective, game: &str) -> Option<u32> {
    if game == KEY_ROME_2 {
        return rome_2_objective_id(objective);
    }
    Some(match objective {
        MissionObjective::ConstructNBuildingsIncluding { .. } => 4,
        MissionObjective::ControlNRegionsIncluding { .. } => 29,
        MissionObjective::ControlNProvincesIncluding { .. } => 30,
        MissionObjective::OwnNUnits { .. } => 31,
        MissionObjective::ResearchNTechsOfTypeX { .. } => 32,
        MissionObjective::OwnNProvincesWithCulturalDominance { .. } => 33,
        MissionObjective::MaintainNClientStates { .. } => 49,
        MissionObjective::SubjugateFactions { .. } => 53,
        MissionObjective::RazeOrSackNDifferentSettlementsIncluding { .. } => 54,
        MissionObjective::EarnXAmountFromRaiding { .. } => 67,
        MissionObjective::EarnXAmountFromBuildingWealth { .. } => 68,
        MissionObjective::AtLeastXReligionInAllProvinces { .. } => 69,
        MissionObjective::ReachSpecifiedDate { .. } => 77,
        MissionObjective::LootOrSackNDifferentSettlementsIncluding { .. } => 81,
        MissionObjective::EnsureFactionsHaveNoMilitaryPresence { .. } => 7,
        MissionObjective::EliminateCharacterInBattle { .. } => 14,
        MissionObjective::IncomeAtLeastX { .. } => 36,
        MissionObjective::MaintainNAlliances { .. } => 51,
        MissionObjective::ConfederateFactions { .. } => 52,
        MissionObjective::StayHorde => 65,
        MissionObjective::AtLeastXPublicOrderInProvinces { .. } => 76,
        MissionObjective::ReclaimNDifferentSettlementsIncluding { .. } => 86,
        MissionObjective::AttainFameLevel { .. } => 89,
        MissionObjective::ControlNPortsIncluding { .. } => 45,
        MissionObjective::OwnAPortAdjoiningSeaRegionsIncluding { .. } => 48,
        MissionObjective::Scripted { .. } => 90,
        MissionObjective::ConstructNBuildingsFrom { .. } => 3,
        MissionObjective::MakeAlliance { .. } => 9,
        MissionObjective::MaintainTradeWithNFactions { .. } => 34,
        MissionObjective::HoldEntiretyOfNProvincesIncluding { .. } => 38,
        MissionObjective::HaveAtLeastXMoney { .. } => 46,
        MissionObjective::ConstructNOfABuilding { .. } => 75,
    })
}

/// Returns the type id of an objective in Rome 2, if its startposes show it.
fn rome_2_objective_id(objective: &MissionObjective) -> Option<u32> {
    Some(match objective {
        MissionObjective::ControlNRegionsIncluding { .. } => 23,
        MissionObjective::ControlNProvincesIncluding { .. } => 24,
        MissionObjective::OwnNUnits { .. } => 25,
        MissionObjective::ResearchNTechsOfTypeX { .. } => 26,
        MissionObjective::OwnNProvincesWithCulturalDominance { .. } => 27,
        MissionObjective::MaintainTradeWithNFactions { .. } => 28,
        MissionObjective::IncomeAtLeastX { .. } => 30,
        MissionObjective::ControlNPortsIncluding { .. } => 39,
        MissionObjective::HaveAtLeastXMoney { .. } => 40,
        MissionObjective::RazeOrSackNDifferentSettlementsIncluding { .. } => 48,
        _ => return None,
    })
}

/// Returns the name of the record with an objective's values: its type key, except for naval unit objectives, which
/// are unit objectives with their naval flag set, and alliances, kept in a diplomacy record.
pub(crate) fn objective_record_name(objective: &MissionObjective) -> &'static str {
    match objective {
        MissionObjective::OwnNUnits { .. } => "OWN_N_UNITS",
        MissionObjective::MakeAlliance { .. } => "DIPLOMACY",
        _ => objective.type_key(),
    }
}
