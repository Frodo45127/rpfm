//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Wars and stance re-evaluation marks the AI starts with.
//!
//! While setting up the starting diplomacy, the game goes through each faction's relationships (factions and their
//! relationships in startpos order) and fires the event of each `start_pos_diplomacy` row. The AI's diplomacy listener
//! reacts to those events by marking, in some factions' strategic stance managers, other factions whose stance they
//! must re-evaluate (`CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION`):
//!
//! - War rows (on the declaring `faction1`'s turn): the target marks the declarer, and so do the target's relatives.
//!   Then each established vassal of either side joins the war against the other side, with the same marks.
//! - Vassal rows (on the vassal's turn): the vassal first declares war on its master's enemies; it and its military
//!   access partners mark each other (the access is cancelled); it becomes established; then it marks its master,
//!   its relatives mark the master and the master's relatives mark it. Client state rows: the same marks, no wars.
//! - Alliance rows: no marks, but the processing faction adds the ally to its relatives.
//!
//! A faction's relatives are its vassals, overlord, client states, protector and allies (the alliance row's
//! `faction2` holds `faction1` from the start). Verified on Attila's main campaign: 172/172 marks.

use std::collections::{BTreeSet, HashMap, HashSet};

use rpfm_lib::error::Result;

use crate::startpos::tables::StartposTables;
use crate::startpos::world::CampaignIndex;

#[cfg(test)] #[path = "stances_test.rs"] mod stances_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Wars and stance re-evaluation marks after the starting diplomacy.
#[derive(Debug, Clone, Default)]
pub struct StartingStances {

    /// Factions each faction marked, in marking order, by faction index.
    marks: Vec<Vec<usize>>,

    /// Pairs of faction indexes at war, smallest first.
    wars: HashSet<(usize, usize)>,
}

/// State of the starting diplomacy while its events fire.
struct Simulation {
    stances: StartingStances,

    /// Relatives of each faction (see the module docs), by faction index.
    relatives: Vec<BTreeSet<usize>>,

    /// Vassals each faction has once their row was processed.
    established: Vec<BTreeSet<usize>>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl StartingStances {

    /// Simulates the starting diplomacy.
    ///
    /// # Arguments
    ///
    /// * `tables` - Tables with `start_pos_diplomacy`.
    /// * `index` - The campaign's startpos rows.
    /// * `factions` - Keys of the world's factions, in startpos order.
    ///
    /// # Errors
    ///
    /// Returns an error if the table or one of its columns is missing.
    pub fn new(tables: &StartposTables, index: &CampaignIndex, factions: &[&str]) -> Result<Self> {
        let position = factions.iter().enumerate().map(|(position, key)| (*key, position)).collect::<HashMap<_, _>>();
        let mut stances = HashMap::new();
        let mut access = vec![BTreeSet::new(); factions.len()];
        for row in tables.table("start_pos_diplomacy")?.rows() {
            let (Some(first), Some(second)) = (index.faction_by_id(&row.str("faction1")?), index.faction_by_id(&row.str("faction2")?)) else { continue };
            let (Some(&first), Some(&second)) = (position.get(first.key().as_str()), position.get(second.key().as_str())) else { continue };
            stances.insert((first, second), row.str("stance")?.into_owned());
            if row.bool("grants_military_access")? {
                access[first].insert(second);
                access[second].insert(first);
            }
        }

        let mut simulation = Simulation {
            stances: Self { marks: vec![vec![]; factions.len()], wars: HashSet::new() },
            relatives: vec![BTreeSet::new(); factions.len()],
            established: vec![BTreeSet::new(); factions.len()],
        };
        for ((first, second), stance) in &stances {
            if matches!(stance.as_str(), "vassal" | "client_state" | "military_allies" | "defensive_allies") {
                simulation.relatives[*second].insert(*first);
            }
            if matches!(stance.as_str(), "vassal" | "client_state") {
                simulation.relatives[*first].insert(*second);
            }
        }

        for (faction, partners) in access.iter().enumerate() {
            for other in (0..factions.len()).filter(|other| *other != faction) {
                match stances.get(&(faction, other)).map(String::as_str) {
                    Some("war") => if !simulation.stances.at_war(faction, other) {
                        simulation.declare_war(faction, other);
                    },
                    Some("vassal") => {
                        for enemy in 0..factions.len() {
                            if enemy != faction && enemy != other && simulation.stances.at_war(other, enemy) && !simulation.stances.at_war(faction, enemy) {
                                simulation.declare_war(faction, enemy);
                            }
                        }
                        for &partner in partners.iter().filter(|partner| **partner != other) {
                            simulation.stances.mark(faction, partner);
                            simulation.stances.mark(partner, faction);
                        }
                        simulation.established[other].insert(faction);
                        simulation.mark_subject(faction, other);
                    },
                    Some("client_state") => simulation.mark_subject(faction, other),
                    Some("military_allies" | "defensive_allies") => { simulation.relatives[faction].insert(other); },
                    _ => if matches!(stances.get(&(other, faction)).map(String::as_str), Some("military_allies" | "defensive_allies")) {
                        simulation.relatives[faction].insert(other);
                    },
                }
            }
        }
        Ok(simulation.stances)
    }

    /// Returns the factions a faction marked for re-evaluation, in marking order.
    ///
    /// # Arguments
    ///
    /// * `faction` - Index of the faction in startpos order.
    pub fn marks(&self, faction: usize) -> &[usize] {
        self.marks.get(faction).map(Vec::as_slice).unwrap_or_default()
    }

    /// Returns if two factions are at war.
    ///
    /// # Arguments
    ///
    /// * `faction` - Index of a faction in startpos order.
    /// * `other` - Index of the other faction.
    pub fn at_war(&self, faction: usize, other: usize) -> bool {
        self.wars.contains(&(faction.min(other), faction.max(other)))
    }

    /// Marks a faction in another one's stance manager, once.
    fn mark(&mut self, faction: usize, target: usize) {
        if !self.marks[faction].contains(&target) {
            self.marks[faction].push(target);
        }
    }
}

impl Simulation {

    /// Declares a war: the target and its relatives mark the declarer, then established vassals join in.
    fn declare_war(&mut self, declarer: usize, target: usize) {
        self.stances.wars.insert((declarer.min(target), declarer.max(target)));
        self.stances.mark(target, declarer);
        self.notify_relatives(target, declarer);

        let joining = self.established[declarer].iter().map(|vassal| (*vassal, target))
            .chain(self.established[target].iter().map(|vassal| (*vassal, declarer)))
            .collect::<Vec<_>>();
        for (vassal, enemy) in joining {
            if vassal != enemy && !self.stances.at_war(vassal, enemy) {
                self.declare_war(vassal, enemy);
            }
        }
    }

    /// Makes the marks of a vassal or client state row.
    fn mark_subject(&mut self, subject: usize, overlord: usize) {
        self.stances.mark(subject, overlord);
        self.notify_relatives(subject, overlord);
        self.notify_relatives(overlord, subject);
    }

    /// Makes every relative of a faction (but the target) mark the target.
    fn notify_relatives(&mut self, faction: usize, target: usize) {
        let relatives = self.relatives[faction].iter().copied().filter(|relative| *relative != target).collect::<Vec<_>>();
        for relative in relatives {
            self.stances.mark(relative, target);
        }
    }
}
