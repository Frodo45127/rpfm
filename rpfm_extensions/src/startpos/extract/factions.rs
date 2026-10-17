//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Extraction of the campaign-wide and per-faction tables: calendar, factions, technologies, victory conditions and
//! diplomacy.

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::Result;

use super::{FactionSource, Ids, Source, TableRows, FACTION_PATH};
use crate::startpos::layout::child_path;
use crate::startpos::tables::StartposTables;

/// Localisation key prefix of the difficulties in the faction infos.
const DIFFICULTY_KEY_PREFIX: &str = "random_localisation_strings_string_";

/// Season key of the `seasons` row for no season, which shares its order with the first season.
const NO_SEASON: &str = "none";

/// Victory type of the victory conditions the startpos keeps.
const LONG_VICTORY: &str = "long";

/// Path of the AI personality of a faction.
const PERSONALITY_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/CAI_INTERFACE/CAI_WORLD/CAI_WORLD_FACTIONS/CAI_FACTION/CAI_FACTION_PERSONALITY";

/// Stance of factions at war.
const WAR_STANCE: &str = "war";

/// Stance factions have with each other unless something sets another.
const NEUTRAL_STANCE: &str = "neutral";

/// Stance of a vassal towards its master, and the other way around.
const VASSAL_STANCE: &str = "vassal";
const MASTER_STANCE: &str = "master";

/// Stance of a client state towards its protector, and the other way around.
const CLIENT_STANCE: &str = "client_state";
const PROTECTOR_STANCE: &str = "protector";

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Extracts `start_pos_calendars`: the start date and turns per year. The season comes from its `seasons.order`.
pub(super) fn calendar(source: &Source, tables: &StartposTables) -> Result<TableRows> {
    let season_order = source.start_date.u32("^+3").unwrap_or_default();
    let mut season = String::new();
    for row in tables.table("seasons")?.rows() {
        let key = row.str("season")?;
        if key != NO_SEASON && row.int("order")? == i64::from(season_order) {
            season = key.into_owned();
            break;
        }
    }

    let mut rows = TableRows::new(&["campaign", "start_year", "turns_per_year", "start_season", "start_week_of_year"]);
    rows.push(vec![
        source.campaign.to_owned(),
        source.start_year().to_string(),
        source.calendar.u32("^+1").unwrap_or_default().to_string(),
        season,
        source.start_date.u32("^+2").unwrap_or_default().to_string(),
    ]);
    Ok(rows)
}

/// Extracts `start_pos_factions`, in startpos order.
pub(super) fn factions(source: &Source) -> Result<TableRows> {
    let personalities = source.cai_factions.iter()
        .filter_map(|faction| Some((faction.u32("^+7")?, faction.record("CAI_FACTION_PERSONALITY")?)))
        .map(|(id, record)| {
            let personality = source.layout.read_single(PERSONALITY_PATH, record)?;
            Ok((id, (personality.str("^+3").unwrap_or_default().to_owned(), personality.str("^+2").unwrap_or_default().to_owned())))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    let mut rows = TableRows::new(&[
        "ID", "faction", "campaign", "playable", "treasury", "starting_order", "government_type", "state_religion", "is_major", "difficulty",
        "ai_manager", "can_ever_convert_religion", "cdir_military_generator_config", "cai_personality_group", "cai_starting_personality",
    ]);
    for faction in &source.factions {
        let group = &faction.group;
        let economics = source.faction_record(faction, "FACTION_ECONOMICS")?;
        let government = source.faction_record(faction, "GOVERNMENT")?;
        let religion = source.faction_record(faction, "FACTION_RELIGION_MANAGER")?;
        let info = faction.info.as_ref();
        let (personality, starting_personality) = personalities.get(&faction.id).cloned().unwrap_or_default();

        rows.push(vec![
            faction.table_id.to_owned(),
            faction.key.to_owned(),
            source.campaign.to_owned(),
            group.bool("MORGUE+1").unwrap_or_default().to_string(),
            economics.and_then(|economics| economics.i32("history+1")).unwrap_or_default().to_string(),
            info.and_then(|info| info.i32("^+4")).unwrap_or_default().to_string(),
            government.as_ref().and_then(|government| government.str("^+2")).unwrap_or_default().to_owned(),
            religion.as_ref().and_then(|religion| religion.str("^+1")).unwrap_or_default().to_owned(),
            group.bool("GOVERNMENT+3").unwrap_or_default().to_string(),
            info.and_then(|info| info.str("^+8")).map(|difficulty| difficulty.trim_start_matches(DIFFICULTY_KEY_PREFIX)).unwrap_or_default().to_owned(),
            group.str("PRESTIGE+4").unwrap_or_default().to_owned(),
            religion.as_ref().and_then(|religion| religion.bool("^+2")).unwrap_or_default().to_string(),
            group.str("PRESTIGE+6").unwrap_or_default().to_owned(),
            personality,
            starting_personality,
        ]);
    }
    Ok(rows)
}

/// Extracts `start_pos_technologies`: the researched technologies of each faction.
pub(super) fn technologies(source: &Source) -> Result<TableRows> {
    let manager_path = child_path(FACTION_PATH, "FACTION_TECHNOLOGY_MANAGER");
    let researched_path = child_path(&manager_path, "RESEARCHED_TECHS");
    let mut rows = TableRows::new(&["faction", "technology"]);
    for faction in &source.factions {
        let Some(manager) = source.faction_record(faction, "FACTION_TECHNOLOGY_MANAGER")? else { continue };
        let Some(researched) = manager.record("RESEARCHED_TECHS") else { continue };
        for technology in source.layout.read(&researched_path, researched)? {
            if technology.bool("^+2").unwrap_or_default() {
                rows.push(vec![faction.table_id.to_owned(), technology.str("^+1").unwrap_or_default().to_owned()]);
            }
        }
    }
    Ok(rows)
}

/// Extracts `start_pos_victory_conditions`: the victory regions of each faction. The startpos only keeps the long
/// victory ones.
pub(super) fn victory_conditions(source: &Source) -> TableRows {
    let mut rows = TableRows::new(&["start_pos_faction", "region", "victory_type"]);
    for faction in &source.factions {
        for region in faction.group.strs("MORGUE+2").unwrap_or_default() {
            rows.push(vec![faction.table_id.to_owned(), region.to_owned(), LONG_VICTORY.to_owned()]);
        }
    }
    rows
}

/// Extracts `start_pos_diplomacy`: one row per pair of factions whose relationship isn't the default one, from the
/// side of the faction that comes first. Wars of vassals against their master's enemies are left out, as the build
/// adds them.
pub(super) fn diplomacy(source: &Source, ids: &mut Ids) -> Result<TableRows> {
    let manager_path = child_path(FACTION_PATH, "OLD_DIPLOMACY_MANAGER");
    let array_path = child_path(&manager_path, "DIPLOMACY_RELATIONSHIPS_ARRAY");
    let positions = source.factions.iter().enumerate().map(|(position, faction)| (faction.id, position)).collect::<HashMap<_, _>>();

    // Rows as (first faction, second faction, stance, military access, trade, non-aggression).
    let mut relationships = vec![];
    for (position, faction) in source.factions.iter().enumerate() {
        let Some(manager) = source.faction_record(faction, "OLD_DIPLOMACY_MANAGER")? else { continue };
        for (_, relationship) in source.layout.read_items(&array_path, &manager, "DIPLOMACY_RELATIONSHIPS_ARRAY", "OLD_DIPLOMACY_RELATIONSHIP")? {
            let Some(other) = relationship.u32("^+1").and_then(|id| positions.get(&id)).map(|&other| &source.factions[other]) else { continue };
            if positions[&other.id] <= position {
                continue;
            }

            let stance = relationship.str("^+4").unwrap_or(NEUTRAL_STANCE);
            let trade = relationship.bool("^+2").unwrap_or_default();
            let non_aggression = relationship.bool("HOSTAGES+3").unwrap_or_default();
            let military_access = relationship.i32("HOSTAGES+4").unwrap_or_default() != 0;
            if stance == NEUTRAL_STANCE && !trade && !non_aggression && !military_access {
                continue;
            }

            // Rows have the subject first, so the master's and protector's sides swap the pair.
            let (first, second, stance) = match stance {
                MASTER_STANCE => (other, faction, VASSAL_STANCE),
                PROTECTOR_STANCE => (other, faction, CLIENT_STANCE),
                stance => (faction, other, stance),
            };
            relationships.push((first, second, stance, military_access, trade, non_aggression));
        }
    }

    let masters = relationships.iter()
        .filter(|relationship| relationship.2 == VASSAL_STANCE)
        .map(|relationship| (relationship.0.id, relationship.1.id))
        .collect::<HashMap<_, _>>();
    let wars = relationships.iter()
        .filter(|relationship| relationship.2 == WAR_STANCE)
        .flat_map(|relationship| [(relationship.0.id, relationship.1.id), (relationship.1.id, relationship.0.id)])
        .collect::<HashSet<_>>();
    let joined_war = |subject: &FactionSource, enemy: &FactionSource| masters.get(&subject.id).is_some_and(|master| wars.contains(&(*master, enemy.id)));

    let mut rows = TableRows::new(&["key", "faction1", "faction2", "stance", "grants_military_access", "grants_trade_agreement", "non_aggression_pact"]);
    for (first, second, stance, military_access, trade, non_aggression) in relationships {
        if stance == WAR_STANCE && (joined_war(first, second) || joined_war(second, first)) {
            continue;
        }
        rows.push(vec![
            ids.next(),
            first.table_id.to_owned(),
            second.table_id.to_owned(),
            stance.to_owned(),
            military_access.to_string(),
            trade.to_string(),
            non_aggression.to_string(),
        ]);
    }
    Ok(rows)
}
