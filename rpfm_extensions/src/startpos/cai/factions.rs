//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI's factions (`CAI_WORLD_FACTIONS`), technology trees (`CAI_WORLD_TECHNOLOGY_TREES`) and
//! technologies (`CAI_WORLD_TECHNOLOGIES`).

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::RecordNode;

use super::map::MapRegion;
use super::plan::CaiPlan;
use super::world_view::WorldView;
use super::{component, CAI_WORLD_PATH};
use crate::startpos::layout::{child_path, StartposLayout};
use crate::startpos::tables::StartposTables;
use crate::startpos::world::CampaignIndex;

#[cfg(test)] #[path = "factions_test.rs"] mod factions_test;

/// World id of the rebels.
const REBELS_WORLD_ID: u32 = 1;

/// Personality group of the rebels.
const REBELS_PERSONALITY: &str = "default";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// What an AI faction refers to.
#[derive(Debug, Clone, Default)]
pub struct FactionLinks {

    /// World id of the faction.
    world_id: u32,

    /// Components of the regions it owns, in map order.
    regions: Vec<u32>,

    /// Connectivity groups it has a presence in.
    groups: Vec<u32>,

    /// Components of its mobiles, in creation order.
    mobiles: Vec<u32>,

    /// Components of its characters, in creation order.
    characters: Vec<u32>,

    /// Component of its capital region (0 for none).
    capital: u32,

    /// Component of its leader (0 for none).
    leader: u32,

    /// Components of its government posts.
    posts: Vec<u32>,

    /// Components of its parts of provinces.
    provinces: Vec<u32>,

    /// Components of its views of the seas.
    seas: Vec<u32>,

    /// Component of its governorship (0 for none).
    governorship: u32,

    /// Its personality group.
    personality: String,

    /// Its starting personality (empty for none).
    starting_personality: String,

    /// Components of the regions of its victory conditions.
    victory_regions: Vec<u32>,

    /// Its horde's mobiles, the first leading (empty for factions with regions).
    horde: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns what each AI faction refers to: the world factions, then the rebels.
///
/// # Arguments
///
/// * `tables` - Tables with `start_pos_factions` and `start_pos_victory_conditions`.
/// * `index` - The campaign's startpos rows.
/// * `world` - The world's objects.
/// * `plan` - Component ids.
/// * `map_regions` - Regions of the map, in map order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn faction_links(tables: &StartposTables, index: &CampaignIndex, world: &WorldView, plan: &CaiPlan, map_regions: &[MapRegion]) -> Result<Vec<FactionLinks>> {
    let region_index = map_regions.iter().enumerate().map(|(position, region)| (region.key().as_str(), position)).collect::<HashMap<_, _>>();
    let region_component = |key: &str| region_index.get(key).map(|position| plan.regions()[*position].id());
    let mut personalities = HashMap::new();
    for row in tables.table("start_pos_factions")?.rows() {
        if let Some(faction) = index.faction_by_id(&row.str("ID")?) {
            personalities.insert(faction.key().to_owned(), (row.str("cai_personality_group")?.into_owned(), row.str("cai_starting_personality")?.into_owned()));
        }
    }
    let mut victory_regions: HashMap<String, Vec<u32>> = HashMap::new();
    for row in tables.table("start_pos_victory_conditions")?.rows() {
        if let (Some(faction), Some(region)) = (index.faction_by_id(&row.str("start_pos_faction")?), region_component(&row.str("region")?)) {
            victory_regions.entry(faction.key().to_owned()).or_default().push(region);
        }
    }

    // Regions and the parts of provinces and seas are listed by owner's world id.
    let owned_regions = |owner: u32| -> Vec<u32> {
        let mut regions = world.regions().iter().filter(|region| region.owner() == owner).filter_map(|region| region_index.get(region.key().as_str()).copied()).collect::<Vec<_>>();
        regions.sort_unstable();
        regions.into_iter().map(|position| plan.regions()[position].id()).collect()
    };
    let parts = |owner: u32| -> (Vec<u32>, Vec<u32>) {
        let provinces = world.provinces().iter().zip(plan.provinces())
            .flat_map(|(province, ids)| province.managers().iter().zip(ids.faction_provinces()))
            .filter(|(part, _)| part.faction() == owner)
            .map(|(_, id)| *id)
            .collect();
        let seas = world.seas().iter().zip(plan.faction_seas())
            .flat_map(|(sea, ids)| sea.managers().iter().zip(ids))
            .filter(|(part, _)| part.faction() == owner)
            .map(|(_, id)| *id)
            .collect();
        (provinces, seas)
    };

    let mut links = vec![];
    for (position, faction) in world.factions().iter().enumerate() {
        let characters = &plan.characters()[position];
        let regions = owned_regions(faction.id());
        let mobiles = characters.iter().filter_map(|character| character.mobile()).collect::<Vec<_>>();
        let leader_id = faction.posts().iter().map(|(_, holder)| *holder).find(|holder| *holder != 0);
        let leader = leader_id.and_then(|leader| faction.characters().iter().position(|character| character.id() == leader)).map(|position| characters[position].character()).unwrap_or_default();
        let (provinces, seas) = parts(faction.id());
        let (personality, starting_personality) = personalities.get(faction.key()).cloned().unwrap_or_default();
        links.push(FactionLinks {
            world_id: faction.id(),
            groups: if regions.is_empty() { vec![] } else { vec![0] },
            horde: if regions.is_empty() { mobiles.to_vec() } else { vec![] },
            regions,
            characters: characters.iter().map(|character| character.character()).collect(),
            mobiles,
            capital: index.capital_of(faction.key()).and_then(|capital| region_component(capital.key())).unwrap_or_default(),
            leader,
            posts: plan.posts()[position].to_vec(),
            provinces,
            seas,
            governorship: plan.governorships()[position],
            personality,
            starting_personality,
            victory_regions: victory_regions.remove(faction.key()).unwrap_or_default(),
        });
    }

    let regions = owned_regions(REBELS_WORLD_ID);
    let (provinces, seas) = parts(REBELS_WORLD_ID);
    links.push(FactionLinks {
        world_id: REBELS_WORLD_ID,
        groups: if regions.is_empty() { vec![] } else { vec![0] },
        regions,
        provinces,
        seas,
        personality: REBELS_PERSONALITY.to_owned(),
        ..Default::default()
    });
    Ok(links)
}

/// Writes the factions list: every world faction, then the rebels.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `links` - What each faction refers to, in plan order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_factions(layout: &StartposLayout, plan: &CaiPlan, links: &[FactionLinks]) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_FACTIONS");
    let faction_path = child_path(&path, "CAI_FACTION");

    let mut factions = layout.builder(&path)?;
    for (position, (ids, links)) in plan.factions().iter().zip(links).enumerate() {
        let mut personality = layout.builder(&child_path(&faction_path, "CAI_FACTION_PERSONALITY"))?;
        personality.group()?
            .set_u32("^+1", ids.faction())?
            .set_str("^+2", &links.starting_personality)?
            .set_str("^+3", &links.personality)?;

        let mut record = layout.builder(&faction_path)?;
        record.group()?
            .set_u32s("^+1", links.regions.to_vec())?
            .set_u32s("^+2", if links.regions.is_empty() { vec![] } else { vec![plan.theatre()] })?
            .set_u32s("^+3", links.groups.to_vec())?
            .set_u32s("^+4", links.mobiles.to_vec())?
            .set_u32s("^+5", links.characters.to_vec())?
            .set_u32("^+6", links.capital)?
            .set_u32("^+7", links.world_id)?
            .set_u32s("^+8", vec![])?
            .set_u32("^+9", ids.tree().unwrap_or_default())?
            .set_u32s("^+10", if links.governorship == 0 { vec![] } else { vec![links.governorship] })?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &faction_path, ids.cai_faction(), vec![])?))?
            .set_u32("CAI_BDI_NEW_TURN+1", links.capital)?
            .set_u32s("CAI_BDI_NEW_TURN+2", links.victory_regions.to_vec())?
            .set_u32("CAI_BDI_NEW_TURN+3", links.leader)?
            .set_u32s("CAI_BDI_NEW_TURN+4", links.posts.to_vec())?
            .set_u32s("CAI_BDI_NEW_TURN+5", links.provinces.to_vec())?
            .set_record("CAI_FACTION_PERSONALITY", Some(personality.build()?))?
            .set_u32s("CAI_FACTION_PERSONALITY+1", links.seas.to_vec())?
            .set_u32("CAI_FACTION_PERSONALITY+2", (position + 1) as u32)?
            .set_record("CAI_FACTION_HORDE_MOBILE_GROUPS", Some(write_horde_groups(layout, &faction_path, &links.horde)?))?;

        factions.push_group()?
            .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, ids.faction())?))?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, ids.faction(), vec![])?))?
            .set_record("CAI_FACTION", Some(record.build()?))?;
    }
    factions.build()
}

/// Writes the technology trees list, one per faction with a tree.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_technology_trees(layout: &StartposLayout, plan: &CaiPlan) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_TECHNOLOGY_TREES");
    let mut trees = layout.builder(&path)?;
    for ids in plan.factions() {
        let Some(tree_id) = ids.tree() else { continue };
        let mut tree = layout.builder(&child_path(&path, "CAI_TECHNOLOGY_TREE"))?;
        tree.group()?.set_u32s("^+1", ids.technologies().to_vec())?;

        trees.push_group()?
            .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, tree_id, vec![])?))?
            .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, ids.faction())?))?
            .set_record("CAI_TECHNOLOGY_TREE", Some(tree.build()?))?;
    }
    trees.build()
}

/// Writes the technologies list: each faction's technologies, faction by faction.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `plan` - Component ids.
/// * `technologies` - Technology keys of each world faction's tree, in plan order.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub fn write_technologies(layout: &StartposLayout, plan: &CaiPlan, technologies: &[Vec<String>]) -> Result<RecordNode> {
    let path = child_path(CAI_WORLD_PATH, "CAI_WORLD_TECHNOLOGIES");
    let mut list = layout.builder(&path)?;
    for (ids, keys) in plan.factions().iter().zip(technologies) {
        let tree_id = ids.tree().unwrap_or_default();
        for (id, key) in ids.technologies().iter().zip(keys) {
            let mut technology = layout.builder(&child_path(&path, "CAI_TECHNOLOGY"))?;
            technology.group()?
                .set_u32("^+1", tree_id)?
                .set_str("^+2", key)?;

            list.push_group()?
                .set_record("CAI_BDI_COMPONENT", Some(component(layout, &path, *id, vec![])?))?
                .set_record("OWNED_DIRECT", Some(owned_direct(layout, &path, ids.faction())?))?
                .set_record("CAI_TECHNOLOGY", Some(technology.build()?))?;
        }
    }
    list.build()
}

/// Writes the horde groups of a faction: none, or one with its mobiles led by the first.
fn write_horde_groups(layout: &StartposLayout, faction_path: &str, horde: &[u32]) -> Result<RecordNode> {
    let path = child_path(faction_path, "CAI_FACTION_HORDE_MOBILE_GROUPS");
    let mut groups = layout.builder(&path)?;
    groups.group()?.set_u32("^+1", u32::from(!horde.is_empty()))?;
    if let Some(leader) = horde.first() {
        let mut group = layout.builder(&child_path(&path, "CAI_FACTION_HORDE_MOBILE_GROUP"))?;
        group.group()?
            .set_u32s("^+1", horde.to_vec())?
            .set_u32("^+2", *leader)?;
        groups.group()?.set_record("CAI_FACTION_HORDE_MOBILE_GROUP", Some(group.build()?))?;
    }
    groups.build()
}

/// Writes the owner record (`OWNED_DIRECT`) of an object of a list.
pub(crate) fn owned_direct(layout: &StartposLayout, list_path: &str, owner: u32) -> Result<RecordNode> {
    let mut record = layout.builder(&child_path(list_path, "OWNED_DIRECT"))?;
    record.group()?.set_u32("^+1", owner)?;
    record.build()
}
