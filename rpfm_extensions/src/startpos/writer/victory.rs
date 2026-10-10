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
//! Each objective is a `CAMPAIGN_MISSION_OBJECTIVE` with its type id and a record of the type's values. The ids come
//! from the game's objective type enum, matched against Attila's startpos: only the types its victory missions use are
//! known.

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{bool_node, u32_node};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::missions::{MissionObjective, VictoryMission, VictoryObjectives};

/// Path of the victory missions of each faction.
const OPTIONS_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS";

/// Path of a victory mission.
const DEFINITION_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION";

/// Path of the objectives and payload of a victory mission.
const OBJECTIVES_AND_PAYLOAD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION/CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD";

/// Path of an objective of a victory mission.
const OBJECTIVE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS/VICTORY_MISSIONS_BY_TYPE_BLOCK/VICTORY_MISSIONS_BLOCK/CAMPAIGN_MISSION_DEFINITION/CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD/OBJECTIVES/CAMPAIGN_MISSION_OBJECTIVE";

/// Prefix of the localisation key of a mission's title.
const TITLE_KEY_PREFIX: &str = "missions_localised_title_";

/// Prefix of the localisation key of a mission's description.
const DESCRIPTION_KEY_PREFIX: &str = "missions_localised_description_";

/// Prefix of the record holding an objective's values.
const IMPL_PREFIX: &str = "CAMPAIGN_MISSION_OBJECTIVE_IMPL_";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Writes the victory missions of every faction (`VICTORY_MISSION_OPTIONS`): one entry per faction, with its missions
/// grouped by kind of campaign.
///
/// # Arguments
///
/// * `layout` - Record layouts of the game.
/// * `objectives` - Victory objectives of the campaign, as `victory_objectives.txt` lists them.
///
/// # Errors
///
/// Returns an error if the layout lacks a record, like the one of an objective type Attila doesn't use.
pub fn write_victory_mission_options(layout: &StartposLayout, objectives: &[VictoryObjectives]) -> Result<RecordNode> {
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
                missions.push_group()?.set_record("CAMPAIGN_MISSION_DEFINITION", Some(write_mission(layout, mission)?))?;
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
fn write_mission(layout: &StartposLayout, mission: &VictoryMission) -> Result<RecordNode> {
    let localisation = |key: String| -> Result<RecordNode> {
        let mut record = layout.builder(&child_path(DEFINITION_PATH, "CAMPAIGN_LOCALISATION"))?;
        record.group()?.set_str("^+1", &key)?;
        record.build()
    };

    let mut objectives = layout.builder(&child_path(OBJECTIVES_AND_PAYLOAD_PATH, "OBJECTIVES"))?;
    for objective in mission.objectives() {
        objectives.push_group()?.set_record("CAMPAIGN_MISSION_OBJECTIVE", Some(write_objective(layout, objective)?))?;
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
fn write_objective(layout: &StartposLayout, objective: &MissionObjective) -> Result<RecordNode> {
    let (id, name) = objective_type(objective);
    let impl_key = format!("{IMPL_PREFIX}{name}");
    let impl_path = child_path(OBJECTIVE_PATH, &impl_key);

    let values = match objective {
        MissionObjective::ConstructNBuildingsIncluding { total, buildings, faction } => {
            let mut children = vec![u32_node(u32::try_from(buildings.len()).unwrap_or(u32::MAX))];
            for building in buildings {
                children.push(NodeType::Ascii(building.to_owned()));
                children.push(bool_node(false));
            }
            children.extend([u32_node(0), u32_node(*total), u32_node(0), NodeType::Ascii(faction.to_owned())]);
            layout.raw_record(&impl_path, children)?
        },
        _ => write_objective_values(layout, &impl_path, objective)?,
    };

    let mut record = layout.builder(OBJECTIVE_PATH)?;
    record.group()?
        .set_u32("^+1", id)?
        .set_record(&impl_key, Some(values))?;
    record.build()
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
        MissionObjective::OwnNUnits { total, naval } => {
            group.set_u32("^+1", *total)?.set_bool("^+2", *naval)?;
        },
        MissionObjective::AtLeastXReligionInAllProvinces { total, religion } => {
            group.set_f32("^+1", *total)?.set_str("^+2", religion)?;
        },

        // The game drops the settlements listed by loot and raze objectives: only their total is kept.
        MissionObjective::ControlNRegionsIncluding { total } |
        MissionObjective::EarnXAmountFromBuildingWealth { total } |
        MissionObjective::EarnXAmountFromRaiding { total } |
        MissionObjective::OwnNProvincesWithCulturalDominance { total } |
        MissionObjective::SubjugateFactions { total } |
        MissionObjective::MaintainNClientStates { total } |
        MissionObjective::LootOrSackNDifferentSettlementsIncluding { total, .. } |
        MissionObjective::RazeOrSackNDifferentSettlementsIncluding { total, .. } => {
            group.set_u32("^+1", *total)?;
        },
        MissionObjective::ConstructNBuildingsIncluding { .. } => {
            return Err(RLibError::StartposLayoutError(format!("{path}: building lists have no fixed layout")));
        },
    }
    record.build()
}

/// Returns the type id of an objective and the name of the record with its values. Naval unit objectives are unit
/// objectives with their naval flag set.
fn objective_type(objective: &MissionObjective) -> (u32, &'static str) {
    match objective {
        MissionObjective::ConstructNBuildingsIncluding { .. } => (4, "CONSTRUCT_N_BUILDINGS_INCLUDING"),
        MissionObjective::ControlNRegionsIncluding { .. } => (29, "CONTROL_N_REGIONS_INCLUDING"),
        MissionObjective::ControlNProvincesIncluding { .. } => (30, "CONTROL_N_PROVINCES_INCLUDING"),
        MissionObjective::OwnNUnits { .. } => (31, "OWN_N_UNITS"),
        MissionObjective::ResearchNTechsOfTypeX { .. } => (32, "RESEARCH_N_TECHS_OF_TYPE_X"),
        MissionObjective::OwnNProvincesWithCulturalDominance { .. } => (33, "OWN_N_PROVINCES_WITH_CULTURAL_DOMINANCE"),
        MissionObjective::MaintainNClientStates { .. } => (49, "MAINTAIN_N_CLIENT_STATES"),
        MissionObjective::SubjugateFactions { .. } => (53, "SUBJUGATE_FACTIONS"),
        MissionObjective::RazeOrSackNDifferentSettlementsIncluding { .. } => (54, "RAZE_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING"),
        MissionObjective::EarnXAmountFromRaiding { .. } => (67, "EARN_X_AMOUNT_FROM_RAIDING"),
        MissionObjective::EarnXAmountFromBuildingWealth { .. } => (68, "EARN_X_AMOUNT_FROM_BUILDING_WEALTH"),
        MissionObjective::AtLeastXReligionInAllProvinces { .. } => (69, "AT_LEAST_X_RELIGION_IN_ALL_PROVINCES"),
        MissionObjective::ReachSpecifiedDate { .. } => (77, "REACH_SPECIFIED_DATE"),
        MissionObjective::LootOrSackNDifferentSettlementsIncluding { .. } => (81, "LOOT_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING"),
    }
}
