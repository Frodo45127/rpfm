//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! What the campaign AI mirrors of a written `WORLD` record: its factions, characters, forces, regions, provinces and
//! seas, with their runtime ids, read through the record layouts.

use getset::{CopyGetters, Getters};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::RecordNode;

use crate::startpos::layout::{child_path, GroupView, StartposLayout};

#[cfg(test)] #[path = "world_view_test.rs"] mod world_view_test;

/// Path of the world.
const WORLD_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The objects of a world, in world order.
#[derive(Debug, Clone, Default, Getters)]
#[getset(get = "pub")]
pub struct WorldView {

    /// Factions, in `FACTION_ARRAY` order.
    factions: Vec<FactionView>,

    /// Regions, in `REGIONS_ARRAY` order.
    regions: Vec<RegionView>,

    /// Provinces, in `PROVINCE_ARRAY` order.
    provinces: Vec<ProvinceView>,

    /// Sea regions, in `SEA_REGIONS_BLOCK` order.
    seas: Vec<SeaView>,
}

/// A faction of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct FactionView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Faction key.
    #[getset(get = "pub")]
    key: String,

    /// Characters, in `CHARACTER_ARRAY` order.
    #[getset(get = "pub")]
    characters: Vec<CharacterView>,

    /// Forces, in `ARMY_ARRAY` order.
    #[getset(get = "pub")]
    forces: Vec<ForceView>,

    /// Government posts, as (runtime id, holder's character id or 0) pairs, in government order.
    #[getset(get = "pub")]
    posts: Vec<(u32, u32)>,
}

/// A character of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct CharacterView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Agent type (`general`, `colonel`, `spy`...).
    #[getset(get = "pub")]
    agent: String,

    /// Cell the character stands on.
    #[getset(get_copy = "pub")]
    cell: (u16, u16),

    /// Runtime id of the force the character commands, 0 for none.
    #[getset(get_copy = "pub")]
    force: u32,

    /// Runtime id of the character's own unit (bodyguard or flagship), 0 for none.
    #[getset(get_copy = "pub")]
    unit: u32,
}

/// A force (army, navy or garrison) of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct ForceView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Runtime id of the commander.
    #[getset(get_copy = "pub")]
    commander: u32,

    /// Runtime ids of the units.
    #[getset(get = "pub")]
    units: Vec<u32>,

    /// Garrison residence of the settlement the force is stationed in, 0 for none.
    #[getset(get_copy = "pub")]
    residence: u32,

    /// Garrison residence the force defends as its garrison army, 0 for none.
    #[getset(get_copy = "pub")]
    garrison_army_of: u32,

    /// Garrison residence the force defends as its garrison navy, 0 for none.
    #[getset(get_copy = "pub")]
    garrison_navy_of: u32,

    /// The force is a navy.
    #[getset(get_copy = "pub")]
    navy: bool,

    /// Runtime ids of a horde army's building slots.
    #[getset(get = "pub")]
    horde_slots: Vec<u32>,
}

/// A region of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct RegionView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Region key.
    #[getset(get = "pub")]
    key: String,

    /// Runtime id of the owner faction.
    #[getset(get_copy = "pub")]
    owner: u32,

    /// Garrison residence of the settlement.
    #[getset(get_copy = "pub")]
    residence: u32,

    /// Runtime ids of the settlement's slots.
    #[getset(get = "pub")]
    slots: Vec<u32>,
}

/// A province of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct ProvinceView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Province key.
    #[getset(get = "pub")]
    key: String,

    /// Each faction's part of the province.
    #[getset(get = "pub")]
    managers: Vec<FactionPartView>,
}

/// A sea region of the world.
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct SeaView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Sea region key.
    #[getset(get = "pub")]
    key: String,

    /// Each faction's view of the sea.
    #[getset(get = "pub")]
    managers: Vec<FactionPartView>,
}

/// A faction's part of a province or sea (`FACTION_PROVINCE_MANAGER`, `FACTION_SEA_REGION_MANAGER`).
#[derive(Debug, Clone, Default, Getters, CopyGetters)]
pub struct FactionPartView {

    /// Runtime id.
    #[getset(get_copy = "pub")]
    id: u32,

    /// Runtime id of the faction.
    #[getset(get_copy = "pub")]
    faction: u32,

    /// Runtime ids of the faction's regions in it.
    #[getset(get = "pub")]
    regions: Vec<u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl WorldView {

    /// Reads the objects of a written world.
    ///
    /// # Arguments
    ///
    /// * `layout` - Record layouts the world was written with.
    /// * `world` - The `WORLD` record.
    ///
    /// # Errors
    ///
    /// Returns an error if the world doesn't fit its layouts.
    pub fn new(layout: &StartposLayout, world: &RecordNode) -> Result<Self> {
        let reader = Reader { layout };
        let world_group = reader.single(WORLD_PATH, world)?;

        let faction_array = child_path(WORLD_PATH, "FACTION_ARRAY");
        let factions = reader.items(&faction_array, &world_group, "FACTION_ARRAY", "FACTION")?.into_iter()
            .map(|(_, faction)| reader.faction(&child_path(&faction_array, "FACTION"), &faction))
            .collect::<Result<Vec<_>>>()?;

        let manager_path = child_path(WORLD_PATH, "REGION_MANAGER");
        let manager = reader.single(&manager_path, reader.record(&world_group, "REGION_MANAGER")?)?;
        let region_array = child_path(&manager_path, "REGIONS_ARRAY");
        let regions = reader.items(&region_array, &manager, "REGIONS_ARRAY", "REGION")?.into_iter()
            .map(|(_, region)| reader.region(&child_path(&region_array, "REGION"), &region))
            .collect::<Result<Vec<_>>>()?;

        let manager_path = child_path(WORLD_PATH, "PROVINCE_MANAGER");
        let manager = reader.single(&manager_path, reader.record(&world_group, "PROVINCE_MANAGER")?)?;
        let province_array = child_path(&manager_path, "PROVINCE_ARRAY");
        let province_path = child_path(&province_array, "PROVINCE");
        let provinces = reader.items(&province_array, &manager, "PROVINCE_ARRAY", "PROVINCE")?.into_iter()
            .map(|(entry, province)| Ok(ProvinceView {
                id: province.u32("^+1").unwrap_or_default(),
                key: entry.str("^+1").unwrap_or_default().to_owned(),
                managers: reader.parts(&province_path, &province, "FACTION_PROVINCE_MANAGER_ARRAY", "FACTION_PROVINCE_MANAGER")?,
            }))
            .collect::<Result<Vec<_>>>()?;

        let manager_path = child_path(WORLD_PATH, "SEA_REGION_MANAGER");
        let manager = reader.single(&manager_path, reader.record(&world_group, "SEA_REGION_MANAGER")?)?;
        let sea_block = child_path(&manager_path, "SEA_REGIONS_BLOCK");
        let sea_path = child_path(&sea_block, "SEA_REGION");
        let seas = reader.items(&sea_block, &manager, "SEA_REGIONS_BLOCK", "SEA_REGION")?.into_iter()
            .map(|(entry, sea)| Ok(SeaView {
                id: sea.u32("^+1").unwrap_or_default(),
                key: entry.str("^+1").unwrap_or_default().to_owned(),
                managers: reader.parts(&sea_path, &sea, "FACTION_SEA_REGION_MANAGER_BLOCK", "FACTION_SEA_REGION_MANAGER")?,
            }))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { factions, regions, provinces, seas })
    }
}

/// Reads records through their layouts.
struct Reader<'l> {
    layout: &'l StartposLayout,
}

impl<'l> Reader<'l> {

    /// Reads the only group of a record.
    fn single<'a>(&self, path: &str, record: &'a RecordNode) -> Result<GroupView<'a>> where 'l: 'a {
        self.layout.read(path, record)?.into_iter().next().ok_or_else(|| world_error(&format!("{path} has no group")))
    }

    /// Returns a child record of a group.
    fn record<'a>(&self, group: &GroupView<'a>, key: &str) -> Result<&'a RecordNode> {
        group.record(key).ok_or_else(|| world_error(&format!("missing {key}")))
    }

    /// Reads the items of an array record child of a group: each array group, with its item record's group.
    fn items<'a>(&self, array_path: &str, parent: &GroupView<'a>, array_key: &str, item_key: &str) -> Result<Vec<(GroupView<'a>, GroupView<'a>)>> where 'l: 'a {
        let Some(array) = parent.record(array_key) else { return Ok(vec![]) };
        let item_path = child_path(array_path, item_key);
        self.layout.read(array_path, array)?.into_iter()
            .map(|entry| {
                let item = self.record(&entry, item_key)?;
                Ok((entry.clone(), self.single(&item_path, item)?))
            })
            .collect()
    }

    /// Reads a faction.
    fn faction(&self, path: &str, faction: &GroupView) -> Result<FactionView> {
        let character_array = child_path(path, "CHARACTER_ARRAY");
        let character_path = child_path(&character_array, "CHARACTER");
        let characters = self.items(&character_array, faction, "CHARACTER_ARRAY", "CHARACTER")?.into_iter()
            .map(|(_, character)| {
                let cell = match character.record("LOCOMOTABLE") {
                    Some(record) => {
                        let locomotable = self.single(&child_path(&character_path, "LOCOMOTABLE"), record)?;
                        (locomotable.u32("^+2").unwrap_or_default() as u16, locomotable.u32("^+3").unwrap_or_default() as u16)
                    },
                    None => (0, 0),
                };
                Ok(CharacterView {
                    id: character.u32("LOCOMOTABLE+1").unwrap_or_default(),
                    agent: character.str("CHARACTER_DETAILS+1").unwrap_or_default().to_owned(),
                    cell,
                    force: character.u32("CHARACTER_DETAILS+2").unwrap_or_default(),
                    unit: character.u32("CHARACTER_DETAILS+3").unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let army_array = child_path(path, "ARMY_ARRAY");
        let force_path = child_path(&army_array, "MILITARY_FORCE");
        let forces = self.items(&army_array, faction, "ARMY_ARRAY", "MILITARY_FORCE")?.into_iter()
            .map(|(_, force)| self.force(&force_path, &force))
            .collect::<Result<Vec<_>>>()?;

        let government_path = child_path(path, "GOVERNMENT");
        let mut posts = vec![];
        if let Some(government) = faction.record("GOVERNMENT") {
            let government = self.single(&government_path, government)?;
            let posts_path = child_path(&government_path, "CHARACTER_POSTS");
            let post_path = child_path(&posts_path, "CHARACTER_POST");
            if let Some(array) = government.record("CHARACTER_POSTS") {
                for entry in self.layout.read(&posts_path, array)? {
                    for index in 1.. {
                        let key = if index == 1 { "CHARACTER_POST".to_owned() } else { format!("CHARACTER_POST@{index}") };
                        let Some(post) = entry.record(&key) else { break };
                        let post = self.single(&post_path, post)?;
                        posts.push((post.u32("^+1").unwrap_or_default(), post.u32("^+3").unwrap_or_default()));
                    }
                }
            }
        }

        Ok(FactionView {
            id: faction.u32("^+1").unwrap_or_default(),
            key: faction.str("^+2").unwrap_or_default().to_owned(),
            characters,
            forces,
            posts,
        })
    }

    /// Reads a force.
    fn force(&self, path: &str, force: &GroupView) -> Result<ForceView> {
        let container_path = child_path(path, "UNIT_CONTAINER");
        let units_path = child_path(&container_path, "UNITS_ARRAY");
        let mut units = vec![];
        if let Some(container) = force.record("UNIT_CONTAINER") {
            let container = self.single(&container_path, container)?;
            for (_, unit) in self.items(&units_path, &container, "UNITS_ARRAY", "UNIT")? {
                units.push(unit.u32("^+1").unwrap_or_default());
            }
        }

        let horde_path = child_path(path, "MILITARY_FORCE_HORDE_DETAILS");
        let slot_manager_path = child_path(&horde_path, "MILITARY_FORCE_SLOT_MANAGER");
        let slot_block_path = child_path(&slot_manager_path, "SLOT_BLOCK");
        let mut horde_slots = vec![];
        if let Some(horde) = force.record("MILITARY_FORCE_HORDE_DETAILS") {
            let horde = self.single(&horde_path, horde)?;
            if let Some(manager) = horde.record("MILITARY_FORCE_SLOT_MANAGER") {
                let manager = self.single(&slot_manager_path, manager)?;
                for (_, slot) in self.items(&slot_block_path, &manager, "SLOT_BLOCK", "MILITARY_FORCE_SLOT")? {
                    horde_slots.push(slot.u32("^+1").unwrap_or_default());
                }
            }
        }

        Ok(ForceView {
            id: force.u32("^+1").unwrap_or_default(),
            commander: force.u32("^+2").unwrap_or_default(),
            units,
            residence: force.u32("UNIT_CONTAINER+1").unwrap_or_default(),
            garrison_army_of: force.u32("UNIT_CONTAINER+11").unwrap_or_default(),
            garrison_navy_of: force.u32("UNIT_CONTAINER+12").unwrap_or_default(),
            navy: force.bool("UNIT_CONTAINER+13").unwrap_or_default(),
            horde_slots,
        })
    }

    /// Reads a region.
    fn region(&self, path: &str, region: &GroupView) -> Result<RegionView> {
        let settlement_path = child_path(path, "SETTLEMENT");
        let mut residence = 0;
        if let Some(settlement) = region.record("SETTLEMENT") {
            let settlement = self.single(&settlement_path, settlement)?;
            if let Some(record) = settlement.record("GARRISON_RESIDENCE") {
                residence = self.single(&child_path(&settlement_path, "GARRISON_RESIDENCE"), record)?.u32("^+1").unwrap_or_default();
            }
        }

        let slot_manager_path = child_path(path, "REGION_SLOT_MANAGER");
        let slot_array_path = child_path(&slot_manager_path, "REGION_SLOT_ARRAY");
        let mut slots = vec![];
        if let Some(manager) = region.record("REGION_SLOT_MANAGER") {
            let manager = self.single(&slot_manager_path, manager)?;
            for (_, slot) in self.items(&slot_array_path, &manager, "REGION_SLOT_ARRAY", "REGION_SLOT")? {
                slots.push(slot.u32("^+1").unwrap_or_default());
            }
        }

        Ok(RegionView {
            id: region.u32("^+1").unwrap_or_default(),
            key: region.str("^+2").unwrap_or_default().to_owned(),
            owner: region.u32("SETTLEMENT+9").unwrap_or_default(),
            residence,
            slots,
        })
    }

    /// Reads the faction parts of a province or sea.
    fn parts(&self, path: &str, parent: &GroupView, array_key: &str, item_key: &str) -> Result<Vec<FactionPartView>> {
        let array_path = child_path(path, array_key);
        self.items(&array_path, parent, array_key, item_key)?.into_iter()
            .map(|(_, part)| Ok(FactionPartView {
                id: part.u32("^+1").unwrap_or_default(),
                faction: part.u32("^+2").unwrap_or_default(),
                regions: part.u32s("^+3").map(<[u32]>::to_vec).unwrap_or_default(),
            }))
            .collect()
    }
}

/// Builds the error for a world that doesn't fit its layouts.
fn world_error(message: &str) -> RLibError {
    RLibError::StartposLayoutError(format!("reading the world: {message}"))
}
