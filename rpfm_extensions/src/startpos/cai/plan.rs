//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Component ids of every AI object, in the order the game creates them.
//!
//! The game counts from [`ROOT_ID`]: the interface, the two components of its world mirror, the central pool's
//! desires, the theatre, each map region followed by its HLCIs, the factions (each with its AI faction, technology
//! tree and technologies; the rebels last), the settlements (each after its building slots), the trade route
//! segments, the region boundaries, the characters (each followed by its force: the mobile, a horde's building slots
//! and the units), the provinces (each followed by its factions' parts), the factions' views of the seas, the
//! governorships and the government posts. Beliefs the game creates in between aren't written (the game rebuilds
//! them), so their ids aren't taken.

use getset::{CopyGetters, Getters};

use std::collections::{HashMap, HashSet};

use super::interface::{CENTRAL_DESIRES, DIRECTOR_BELIEFS, DIRECTOR_DESIRES};
use super::managers::ManagerKind;
use super::map::MapRegion;
use super::world_view::WorldView;

#[cfg(test)] #[path = "plan_test.rs"] mod plan_test;

/// Component id of the AI interface itself.
pub const ROOT_ID: u32 = 2;

/// Components of the AI's world mirror (its own and its time's).
const WORLD_COMPONENTS: u32 = 2;

/// Agent types of the characters that only move with a force. Other agents move on their own, as mobiles.
const FORCE_AGENTS: [&str; 2] = ["general", "colonel"];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Component ids of the AI objects.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct CaiPlan {

    /// Next free id.
    #[getset(get_copy = "pub")]
    next_id: u32,

    /// Id of the first central desire.
    #[getset(get_copy = "pub")]
    first_central_desire: u32,

    /// Id of the theatre (the whole map).
    #[getset(get_copy = "pub")]
    theatre: u32,

    /// Ids of each map region, in map order.
    #[getset(get = "pub")]
    regions: Vec<RegionIds>,

    /// Ids of each faction, in world order, then the rebels.
    #[getset(get = "pub")]
    factions: Vec<FactionIds>,

    /// Ids of each land region's settlement, by map region index.
    #[getset(get = "pub")]
    settlements: HashMap<usize, SettlementIds>,

    /// Id of the first trade route segment (one per trade spline, in spline order).
    #[getset(get_copy = "pub")]
    first_trade_segment: u32,

    /// Region boundaries, in creation order.
    #[getset(get = "pub")]
    boundaries: Vec<BoundaryIds>,

    /// Ids of each character, by world faction position, in `CHARACTER_ARRAY` order.
    #[getset(get = "pub")]
    characters: Vec<Vec<CharacterIds>>,

    /// Ids of each province, in world order.
    #[getset(get = "pub")]
    provinces: Vec<ProvinceIds>,

    /// Ids of the factions' views of each sea, by world sea position, in manager order.
    #[getset(get = "pub")]
    faction_seas: Vec<Vec<u32>>,

    /// Id of each faction's governorship, in world order.
    #[getset(get = "pub")]
    governorships: Vec<u32>,

    /// Ids of each faction's government posts, in world order.
    #[getset(get = "pub")]
    posts: Vec<Vec<u32>>,

    /// Ids of the AI managers of the factions owning regions or mobiles, in `factions` order (the rebels always have
    /// one).
    #[getset(get = "pub")]
    managers: Vec<ManagerIds>,

    /// Kind of AI manager of each faction, in `factions` order.
    #[getset(get = "pub")]
    manager_kinds: Vec<ManagerKind>,

    /// Ids of the director pool's beliefs, then its desires, in creation order.
    #[getset(get = "pub")]
    director: Vec<u32>,

    /// Id of the first international trade route (one per route, in route id order).
    #[getset(get_copy = "pub")]
    first_trade_route: u32,
}

/// What the id plan is built from.
#[derive(Debug, Clone, Copy)]
pub struct PlanInputs<'a> {

    /// Regions of the map, in map order.
    pub map_regions: &'a [MapRegion],

    /// The world's objects.
    pub world: &'a WorldView,

    /// Number of technologies of each world faction's tree, in world order.
    pub technology_counts: &'a [usize],

    /// Number of trade splines of the map.
    pub trade_splines: usize,

    /// If players can pick each world faction, in world order.
    pub playable: &'a [bool],

    /// Number of international trade routes of the trade manager.
    pub trade_routes: usize,
}

/// Ids of a faction's AI manager.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct ManagerIds {

    /// Position of its faction in the plan's factions.
    #[getset(get_copy = "pub")]
    faction: usize,

    /// Ids of its desires, in creation order (see [`ManagerKind::desires`]).
    #[getset(get = "pub")]
    desires: Vec<u32>,

    /// Id of its faction-wide intention.
    #[getset(get_copy = "pub")]
    intention: u32,

    /// Ids of its financial and strategic context beliefs (the rebels have none).
    #[getset(get_copy = "pub")]
    beliefs: Option<(u32, u32)>,

    /// Ids of the skill management intentions of each of its characters, in character order.
    #[getset(get = "pub")]
    character_intentions: Vec<u32>,

    /// Ids of the skill management intentions of each of its mobiles, in mobile order.
    #[getset(get = "pub")]
    mobile_intentions: Vec<u32>,
}

/// Ids of a faction.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct FactionIds {

    /// The faction's id.
    #[getset(get_copy = "pub")]
    faction: u32,

    /// Id of its AI faction record.
    #[getset(get_copy = "pub")]
    cai_faction: u32,

    /// Id of its technology tree (the rebels have none).
    #[getset(get_copy = "pub")]
    tree: Option<u32>,

    /// Ids of its technologies, in tree order.
    #[getset(get = "pub")]
    technologies: Vec<u32>,
}

/// Ids of a settlement and its building slots.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct SettlementIds {

    /// Ids of the slots, in world order.
    #[getset(get = "pub")]
    slots: Vec<u32>,

    /// The settlement's id.
    #[getset(get_copy = "pub")]
    settlement: u32,
}

/// A boundary between two map regions.
#[derive(Debug, Clone, Default, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct BoundaryIds {

    /// The boundary's id.
    id: u32,

    /// Index of the first region, the one listing the connection first.
    region: usize,

    /// Index of the other region.
    other: usize,

    /// Sum of both regions' connection values.
    value: u32,
}

/// Ids of a character and its force.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct CharacterIds {

    /// The character's id.
    #[getset(get_copy = "pub")]
    character: u32,

    /// Id of the mobile of its force (or of the agent itself), if it has one.
    #[getset(get_copy = "pub")]
    mobile: Option<u32>,

    /// Ids of its horde's building slots.
    #[getset(get = "pub")]
    horde_slots: Vec<u32>,

    /// Ids of its force's units, in world order.
    #[getset(get = "pub")]
    units: Vec<u32>,
}

/// Ids of a province and its factions' parts.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct ProvinceIds {

    /// The province's id.
    #[getset(get_copy = "pub")]
    province: u32,

    /// Ids of each faction's part, in world order.
    #[getset(get = "pub")]
    faction_provinces: Vec<u32>,
}

/// Ids of a map region and its HLCIs.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct RegionIds {

    /// The region's id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Its HLCIs' ids, in the region's HLCI order.
    #[getset(get = "pub")]
    hlcis: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CaiPlan {

    /// Plans the ids of a campaign's AI objects.
    pub fn new(inputs: &PlanInputs) -> Self {
        let world = inputs.world;
        let mut plan = Self { next_id: ROOT_ID + 1 + WORLD_COMPONENTS, ..Default::default() };
        plan.first_central_desire = plan.take(CENTRAL_DESIRES.len() as u32);
        plan.theatre = plan.take(1);
        plan.regions = inputs.map_regions.iter().map(|region| {
            let id = plan.take(1);
            let hlcis = (0..region.hlcis().len()).map(|_| plan.take(1)).collect();
            RegionIds { id, hlcis }
        }).collect();

        for count in inputs.technology_counts.iter().take(world.factions().len()) {
            let faction = plan.take(1);
            let cai_faction = plan.take(1);
            let tree = Some(plan.take(1));
            let technologies = (0..*count).map(|_| plan.take(1)).collect();
            plan.factions.push(FactionIds { faction, cai_faction, tree, technologies });
        }
        let faction = plan.take(1);
        let cai_faction = plan.take(1);
        plan.factions.push(FactionIds { faction, cai_faction, tree: None, technologies: vec![] });

        for (index, map_region) in inputs.map_regions.iter().enumerate() {
            if let Some(region) = world.regions().iter().find(|region| region.key() == map_region.key()) {
                let slots = region.slots().iter().map(|_| plan.take(1)).collect();
                let settlement = plan.take(1);
                plan.settlements.insert(index, SettlementIds { slots, settlement });
            }
        }

        plan.first_trade_segment = plan.take(inputs.trade_splines as u32);
        plan.boundaries = boundaries(inputs.map_regions).into_iter()
            .map(|(region, other, value)| BoundaryIds { id: plan.take(1), region, other, value })
            .collect();

        for faction in world.factions() {
            let characters = faction.characters().iter().map(|character| {
                let id = plan.take(1);
                let force = faction.forces().iter().find(|force| character.force() != 0 && force.id() == character.force());
                let (mobile, horde_slots, units) = match force {
                    Some(force) => (
                        Some(plan.take(1)),
                        force.horde_slots().iter().map(|_| plan.take(1)).collect(),
                        force.units().iter().map(|_| plan.take(1)).collect(),
                    ),
                    None if !FORCE_AGENTS.contains(&character.agent().as_str()) => (Some(plan.take(1)), vec![], vec![]),
                    None => (None, vec![], vec![]),
                };
                CharacterIds { character: id, mobile, horde_slots, units }
            }).collect();
            plan.characters.push(characters);
        }

        plan.provinces = world.provinces().iter().map(|province| {
            let id = plan.take(1);
            let faction_provinces = province.managers().iter().map(|_| plan.take(1)).collect();
            ProvinceIds { province: id, faction_provinces }
        }).collect();
        plan.faction_seas = world.seas().iter().map(|sea| sea.managers().iter().map(|_| plan.take(1)).collect()).collect();
        plan.governorships = world.factions().iter().map(|_| plan.take(1)).collect();
        plan.posts = world.factions().iter().map(|faction| faction.posts().iter().map(|_| plan.take(1)).collect()).collect();

        plan.manager_kinds = (0..world.factions().len())
            .map(|position| if inputs.playable.get(position).copied().unwrap_or_default() { ManagerKind::Human } else { ManagerKind::Ai })
            .chain(std::iter::once(ManagerKind::Rebels))
            .collect();

        let owners = world.regions().iter().map(|region| region.owner()).collect::<HashSet<_>>();
        let counts = plan.characters.iter()
            .map(|characters| (characters.len(), characters.iter().filter(|character| character.mobile().is_some()).count()))
            .collect::<Vec<_>>();
        for (position, (faction, (characters, mobiles))) in world.factions().iter().zip(counts).enumerate() {
            if !owners.contains(&faction.id()) && mobiles == 0 {
                continue;
            }

            // Only AI managers have intentions for their characters and mobiles.
            let kind = plan.manager_kinds[position];
            let (characters, mobiles) = if kind == ManagerKind::Ai { (characters, mobiles) } else { (0, 0) };
            let desires = kind.desires().iter().map(|_| plan.take(1)).collect();
            let intention = plan.take(1);
            let beliefs = Some((plan.take(1), plan.take(1)));
            let character_intentions = (0..characters).map(|_| plan.take(1)).collect();
            let mobile_intentions = (0..mobiles).map(|_| plan.take(1)).collect();
            plan.managers.push(ManagerIds { faction: position, desires, intention, beliefs, character_intentions, mobile_intentions });
        }
        let desires = ManagerKind::Rebels.desires().iter().map(|_| plan.take(1)).collect();
        let intention = plan.take(1);
        plan.managers.push(ManagerIds { faction: plan.factions.len() - 1, desires, intention, ..Default::default() });

        plan.director = (0..DIRECTOR_BELIEFS.len() + DIRECTOR_DESIRES.len()).map(|_| plan.take(1)).collect();
        plan.first_trade_route = plan.take(inputs.trade_routes as u32);
        plan
    }

    /// Takes the next ids.
    ///
    /// # Returns
    ///
    /// The first of them.
    fn take(&mut self, count: u32) -> u32 {
        let first = self.next_id;
        self.next_id += count;
        first
    }
}

/// Returns the boundaries between map regions: walking the regions in map order and each one's connections in order,
/// one boundary per pair of regions at its first appearance, with the sum of both directions' values.
fn boundaries(map_regions: &[MapRegion]) -> Vec<(usize, usize, u32)> {
    let mut values = HashMap::new();
    for (index, region) in map_regions.iter().enumerate() {
        for (other, value) in region.connections() {
            *values.entry((index.min(*other), index.max(*other))).or_insert(0) += value;
        }
    }

    let mut boundaries = vec![];
    let mut seen = HashSet::new();
    for (index, region) in map_regions.iter().enumerate() {
        for (other, _) in region.connections() {
            let pair = (index.min(*other), index.max(*other));
            if seen.insert(pair) {
                boundaries.push((index, *other, values[&pair]));
            }
        }
    }
    boundaries
}
