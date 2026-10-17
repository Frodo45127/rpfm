//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Extraction of the victory objectives (`victory_objectives.txt`) from the victory missions of the pre-campaign map
//! info (`CAMPAIGN_PREOPEN_MAP_INFO/VICTORY_MISSION_OPTIONS`).

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{extraction_error, Source};
use crate::startpos::esf_nav::{as_str, as_u32};
use crate::startpos::layout::{child_path, GroupView};
use crate::startpos::missions::{CampaignType, MissionObjective, VictoryMission, VictoryObjectives};
use crate::startpos::writer::victory::{objective_id, DEFINITION_PATH, IMPL_PREFIX, OBJECTIVES_AND_PAYLOAD_PATH, OBJECTIVE_PATH, OPTIONS_PATH};

#[cfg(test)] #[path = "missions_test.rs"] mod missions_test;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Extracts the victory objectives of every faction and kind of campaign, in startpos order.
pub(super) fn extract(source: &Source) -> Result<Vec<VictoryObjectives>> {
    let layout = source.layout;
    let types_path = child_path(OPTIONS_PATH, "VICTORY_MISSIONS_BY_TYPE_BLOCK");
    let missions_path = child_path(&types_path, "VICTORY_MISSIONS_BLOCK");
    let Some(options) = source.preopen.record("VICTORY_MISSION_OPTIONS") else { return Ok(vec![]) };

    let mut objectives = vec![];
    for faction in layout.read(OPTIONS_PATH, options)? {
        let faction_key = faction.str("^+1").unwrap_or_default();
        let Some(types) = faction.record("VICTORY_MISSIONS_BY_TYPE_BLOCK") else { continue };
        for campaign_type in layout.read(&types_path, types)? {
            let id = campaign_type.u32("^+1").unwrap_or_default();
            let campaign_type_value = CampaignType::from_id(id).ok_or_else(|| extraction_error(&format!("unknown victory campaign type {id}")))?;
            let mut missions = vec![];
            for (_, definition) in layout.read_items(&missions_path, &campaign_type, "VICTORY_MISSIONS_BLOCK", "CAMPAIGN_MISSION_DEFINITION")? {
                missions.push(mission(source, &definition)?);
            }
            objectives.push(VictoryObjectives::new(faction_key.to_owned(), campaign_type_value, missions));
        }
    }
    Ok(objectives)
}

/// Reads a mission from its definition (`CAMPAIGN_MISSION_DEFINITION`).
fn mission(source: &Source, definition: &GroupView) -> Result<VictoryMission> {
    let layout = source.layout;
    let mut objectives = vec![];
    if let Some(record) = definition.record("CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD") {
        let objectives_and_payload = layout.read_single(&child_path(DEFINITION_PATH, "CAMPAIGN_MISSION_OBJECTIVES_AND_PAYLOAD"), record)?;
        let objectives_path = child_path(OBJECTIVES_AND_PAYLOAD_PATH, "OBJECTIVES");
        for (_, objective) in layout.read_items(&objectives_path, &objectives_and_payload, "OBJECTIVES", "CAMPAIGN_MISSION_OBJECTIVE")? {
            objectives.push(read_objective(source, &objective)?);
        }
    }

    Ok(VictoryMission::new(
        definition.str("^+1").unwrap_or_default().to_owned(),
        definition.str("SECONDARY_OBJECTIVES+4").unwrap_or_default().to_owned(),
        definition.str("^+2").unwrap_or_default().to_owned(),
        objectives,
    ))
}

/// Reads an objective (`CAMPAIGN_MISSION_OBJECTIVE`): its values are in a record named after its type.
fn read_objective(source: &Source, objective: &GroupView) -> Result<MissionObjective> {
    let id = objective.u32("^+1").unwrap_or_default();
    let Some(impl_key) = objective.ids().find(|key| key.starts_with(IMPL_PREFIX)) else {
        return if id == objective_id(&MissionObjective::StayHorde) {
            Ok(MissionObjective::StayHorde)
        } else {
            Err(extraction_error(&format!("victory objective {id} has no values")))
        };
    };
    let name = impl_key.trim_start_matches(IMPL_PREFIX);
    let record = objective.record(impl_key).ok_or_else(|| extraction_error(&format!("{impl_key} isn't a record")))?;

    // Objectives with lists have no fixed layout: their values are read in order.
    match name {
        "CONSTRUCT_N_BUILDINGS_INCLUDING" | "RECLAIM_N_DIFFERENT_SETTLEMENTS_INCLUDING" | "AT_LEAST_X_PUBLIC_ORDER_IN_PROVINCES" | "ENSURE_FACTIONS_HAVE_NO_MILITARY_PRESENCE" => {
            list_objective(name, record).ok_or_else(|| extraction_error(&format!("{impl_key} doesn't have the expected values")))
        },
        _ => {
            let path = child_path(OBJECTIVE_PATH, impl_key);
            let values = source.layout.read_single(&path, record)?;
            fixed_objective(source, name, &path, &values)
        },
    }
}

/// Reads an objective with fixed values, by the name of its values record.
fn fixed_objective(source: &Source, name: &str, path: &str, values: &GroupView) -> Result<MissionObjective> {
    let total = values.u32("^+1").unwrap_or_default();
    let strings = || values.strs("^+2").map(<[String]>::to_vec).unwrap_or_default();
    Ok(match name {
        "REACH_SPECIFIED_DATE" => {
            let date = match values.record("DATE") {
                Some(record) => Some(source.layout.read_single(&child_path(path, "DATE"), record)?),
                None => None,
            };
            let date_value = |name: &str| date.as_ref().and_then(|date| date.u32(name)).unwrap_or_default();
            MissionObjective::ReachSpecifiedDate { year: date_value("^+1"), week: date_value("^+2") }
        },
        "CONTROL_N_REGIONS_INCLUDING" => MissionObjective::ControlNRegionsIncluding { total, regions: strings() },
        "CONTROL_N_PROVINCES_INCLUDING" => MissionObjective::ControlNProvincesIncluding { total, provinces: strings() },
        "RESEARCH_N_TECHS_OF_TYPE_X" => {
            let text = match values.record("CAMPAIGN_LOCALISATION") {
                Some(record) => source.layout.read_single(&child_path(path, "CAMPAIGN_LOCALISATION"), record)?.str("^+1").filter(|text| !text.is_empty()).map(str::to_owned),
                None => None,
            };
            MissionObjective::ResearchNTechsOfTypeX { total, categories: strings(), text }
        },
        "OWN_N_UNITS" => MissionObjective::OwnNUnits { total, naval: values.bool("^+2").unwrap_or_default() },
        "AT_LEAST_X_RELIGION_IN_ALL_PROVINCES" => MissionObjective::AtLeastXReligionInAllProvinces {
            total: values.f32("^+1").unwrap_or_default(),
            religion: values.str("^+2").unwrap_or_default().to_owned(),
        },
        "INCOME_AT_LEAST_X" => MissionObjective::IncomeAtLeastX { income: total },
        "ELIMINATE_CHARACTER_IN_BATTLE" => MissionObjective::EliminateCharacterInBattle { faction: values.str("^+2").unwrap_or_default().to_owned() },
        "ATTAIN_FAME_LEVEL" => MissionObjective::AttainFameLevel { level: total },
        "EARN_X_AMOUNT_FROM_BUILDING_WEALTH" => MissionObjective::EarnXAmountFromBuildingWealth { total },
        "EARN_X_AMOUNT_FROM_RAIDING" => MissionObjective::EarnXAmountFromRaiding { total },
        "OWN_N_PROVINCES_WITH_CULTURAL_DOMINANCE" => MissionObjective::OwnNProvincesWithCulturalDominance { total },
        "SUBJUGATE_FACTIONS" => MissionObjective::SubjugateFactions { total },
        "MAINTAIN_N_CLIENT_STATES" => MissionObjective::MaintainNClientStates { total },
        "MAINTAIN_N_ALLIANCES" => MissionObjective::MaintainNAlliances { total },
        "CONFEDERATE_FACTIONS" => MissionObjective::ConfederateFactions { total },

        // The game doesn't keep the settlements these objectives list.
        "LOOT_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING" => MissionObjective::LootOrSackNDifferentSettlementsIncluding { total, regions: vec![] },
        "RAZE_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING" => MissionObjective::RazeOrSackNDifferentSettlementsIncluding { total, regions: vec![] },
        other => return Err(extraction_error(&format!("unsupported victory objective {other}"))),
    })
}

/// Reads an objective with a list, from the values of its record in order (see the victory missions writer).
fn list_objective(name: &str, record: &RecordNode) -> Option<MissionObjective> {
    let mut values = record.children().first()?.iter();
    Some(match name {
        "CONSTRUCT_N_BUILDINGS_INCLUDING" => {
            let count = as_u32(values.next()?)?;
            let buildings = (0..count).map(|_| {
                let building = values.next().and_then(as_str).map(str::to_owned);
                values.next();
                building
            }).collect::<Option<Vec<_>>>()?;
            values.next()?;
            let total = as_u32(values.next()?)?;
            values.next()?;
            MissionObjective::ConstructNBuildingsIncluding { total, buildings, faction: as_str(values.next()?)?.to_owned() }
        },
        "RECLAIM_N_DIFFERENT_SETTLEMENTS_INCLUDING" => {
            let total = as_u32(values.next()?)?;
            MissionObjective::ReclaimNDifferentSettlementsIncluding { total, regions: keys(&mut values)? }
        },
        "AT_LEAST_X_PUBLIC_ORDER_IN_PROVINCES" => {
            let total = match values.next()? {
                NodeType::F32(node) => *node.value(),
                _ => return None,
            };
            MissionObjective::AtLeastXPublicOrderInProvinces { total, provinces: keys(&mut values)? }
        },
        _ => MissionObjective::EnsureFactionsHaveNoMilitaryPresence { factions: keys(&mut values)? },
    })
}

/// Reads the keys at the end of a list objective's values: a 0 for what the game resolves while playing, their count,
/// then each key.
fn keys(values: &mut std::slice::Iter<NodeType>) -> Option<Vec<String>> {
    values.next()?;
    let count = as_u32(values.next()?)?;
    (0..count).map(|_| values.next().and_then(as_str).map(str::to_owned)).collect()
}
