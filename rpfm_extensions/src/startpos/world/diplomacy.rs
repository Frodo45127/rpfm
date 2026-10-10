//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Diplomatic relationships between factions (`FACTION/OLD_DIPLOMACY_MANAGER`).
//!
//! Each `start_pos_diplomacy` row gives both factions a relationship: its stance (vassal and client state rows give
//! the overlord the `master` and `protector` stances), trade agreement, non-aggression pact and military access.
//! Vassals join their master's wars. Factions know each other if they have a row, an inherited war, neighbouring
//! regions or a `start_pos_past_events` row. Verified on Attila's main campaign: stances, flags and known factions
//! match for every pair of factions.

use getset::{CopyGetters, Getters};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::Result;

use super::CampaignIndex;
use crate::startpos::region_map::RegionMap;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "diplomacy_test.rs"] mod diplomacy_test;

/// Stance of factions without a relationship row.
pub const NEUTRAL_STANCE: &str = "neutral";

/// Stance of factions at war.
const WAR_STANCE: &str = "war";

/// Value a vassal pair stores on both sides (`#7` on the vassal's, `#8` on the master's). The game stores 20 for the
/// Sassanid vassals and 40 for the Eastern Roman ones; the source isn't known.
const VASSAL_VALUE: i32 = 20;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The relationship of a faction with another one.
#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct Relationship {

    /// Stance: `neutral`, `war`, `military_allies`, `defensive_allies`, `vassal`, `master`, `client_state` or
    /// `protector`.
    #[getset(get = "pub")]
    stance: String,

    /// They have a trade agreement.
    #[getset(get_copy = "pub")]
    trade: bool,

    /// They have a non-aggression pact.
    #[getset(get_copy = "pub")]
    non_aggression: bool,

    /// They grant each other military access.
    #[getset(get_copy = "pub")]
    military_access: bool,

    /// Value of vassal pairs: on the vassal's side and on the master's side.
    #[getset(get_copy = "pub")]
    vassal_values: (i32, i32),

    /// Enemies both factions are at war with, and those enemies' defensive allies (flagged `true`).
    #[getset(get = "pub")]
    allied_against: Vec<(String, bool)>,

    /// The factions know each other.
    #[getset(get_copy = "pub")]
    known: bool,
}

/// The diplomatic relationships of the campaign's factions.
#[derive(Debug, Clone, Default)]
pub struct Diplomacy {

    /// Relationships that aren't the default neutral one, by (faction, other faction) keys.
    relationships: HashMap<(String, String), Relationship>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Default for Relationship {
    fn default() -> Self {
        Self {
            stance: NEUTRAL_STANCE.to_owned(),
            trade: false,
            non_aggression: false,
            military_access: false,
            vassal_values: (0, 0),
            allied_against: vec![],
            known: false,
        }
    }
}

impl Diplomacy {

    /// Builds the relationships of the campaign's factions.
    ///
    /// # Arguments
    ///
    /// * `tables` - Tables with `start_pos_diplomacy` and `start_pos_past_events`.
    /// * `index` - The campaign's startpos rows.
    /// * `map` - Region of each cell of the campaign map, to find the factions with neighbouring regions.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(tables: &StartposTables, index: &CampaignIndex, map: &RegionMap) -> Result<Self> {
        let mut rows = vec![];
        for row in tables.table("start_pos_diplomacy")?.rows() {
            let (Some(first), Some(second)) = (index.faction_by_id(&row.str("faction1")?), index.faction_by_id(&row.str("faction2")?)) else { continue };
            rows.push((first.key().to_owned(), second.key().to_owned(), row.str("stance")?.into_owned(), row.bool("grants_trade_agreement")?, row.bool("non_aggression_pact")?, row.bool("grants_military_access")?));
        }

        let mut diplomacy = Self::default();
        for (first, second, stance, trade, non_aggression, military_access) in &rows {
            let (first_stance, second_stance) = match stance.as_str() {
                "vassal" => ("vassal", "master"),
                "client_state" => ("client_state", "protector"),
                other => (other, other),
            };
            let vassal = stance == "vassal";
            for (faction, other, stance) in [(first, second, first_stance), (second, first, second_stance)] {
                let relationship = diplomacy.entry(faction, other);
                relationship.stance = stance.to_owned();
                relationship.trade = *trade;
                relationship.non_aggression = *non_aggression;
                relationship.military_access = *military_access;
                relationship.known = true;
            }
            if vassal {
                diplomacy.entry(first, second).vassal_values = (VASSAL_VALUE, 0);
                diplomacy.entry(second, first).vassal_values = (0, VASSAL_VALUE);
            }
        }

        // Vassals join their master's wars. Wars here are the ones from rows, so they don't chain.
        let wars = rows.iter().filter(|row| row.2 == WAR_STANCE).map(|row| (row.0.to_owned(), row.1.to_owned())).collect::<Vec<_>>();
        let enemies_of = |faction: &str| wars.iter()
            .filter_map(|(first, second)| if first == faction { Some(second.to_owned()) } else if second == faction { Some(first.to_owned()) } else { None })
            .collect::<Vec<_>>();
        for (subject, master, ..) in rows.iter().filter(|row| row.2 == "vassal") {
            for enemy in enemies_of(master) {
                if enemy != *subject && !diplomacy.relationships.contains_key(&(subject.to_owned(), enemy.to_owned())) {
                    for (faction, other) in [(subject, &enemy), (&enemy, subject)] {
                        let relationship = diplomacy.entry(faction, other);
                        relationship.stance = WAR_STANCE.to_owned();
                        relationship.known = true;
                    }
                }
            }
        }

        // Allied factions at war with the same enemy fight it together, and against its defensive allies.
        let defensive_allies_of = |faction: &str| rows.iter()
            .filter(|row| row.2 == "defensive_allies")
            .filter_map(|row| if row.0 == faction { Some(row.1.to_owned()) } else if row.1 == faction { Some(row.0.to_owned()) } else { None })
            .collect::<Vec<_>>();
        for (first, second, ..) in rows.iter().filter(|row| matches!(row.2.as_str(), "vassal" | "client_state" | "military_allies" | "defensive_allies")) {
            let second_enemies = enemies_of(second).into_iter().collect::<HashSet<_>>();
            let common = enemies_of(first).into_iter().filter(|enemy| second_enemies.contains(enemy)).collect::<Vec<_>>();
            let mut against = vec![];
            for enemy in common {
                let allies = defensive_allies_of(&enemy);
                against.push((enemy, false));
                against.extend(allies.into_iter().map(|ally| (ally, true)));
            }
            if !against.is_empty() {
                diplomacy.entry(first, second).allied_against = against.clone();
                diplomacy.entry(second, first).allied_against = against;
            }
        }

        // Factions with neighbouring regions know each other.
        let owners = index.regions().iter().filter_map(|region| region.owner().as_ref().map(|owner| (region.key().as_str(), owner.as_str()))).collect::<HashMap<_, _>>();
        for (region, other_region) in map.neighbour_pairs() {
            if let (Some(owner), Some(other_owner)) = (owners.get(region), owners.get(other_region)) {
                if owner != other_owner {
                    diplomacy.entry(owner, other_owner).known = true;
                    diplomacy.entry(other_owner, owner).known = true;
                }
            }
        }

        // Factions with past events between them know each other.
        for row in tables.table("start_pos_past_events")?.rows() {
            let (Some(source), Some(target)) = (index.faction_by_id(&row.str("source")?), index.faction_by_id(&row.str("target")?)) else { continue };
            if source.key() != target.key() {
                diplomacy.entry(source.key(), target.key()).known = true;
                diplomacy.entry(target.key(), source.key()).known = true;
            }
        }

        Ok(diplomacy)
    }

    /// Returns the relationship of a faction with another one: neutral if nothing sets it.
    pub fn relationship(&self, faction: &str, other: &str) -> Relationship {
        self.relationships.get(&(faction.to_owned(), other.to_owned())).cloned().unwrap_or_default()
    }

    /// Returns the relationship of a faction with another one, creating a neutral one if needed.
    fn entry(&mut self, faction: &str, other: &str) -> &mut Relationship {
        self.relationships.entry((faction.to_owned(), other.to_owned())).or_default()
    }
}
