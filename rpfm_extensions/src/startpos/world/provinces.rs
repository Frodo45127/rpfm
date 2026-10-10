//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-faction managers of provinces (`PROVINCE/FACTION_PROVINCE_MANAGER_ARRAY`) and sea regions
//! (`WORLD/SEA_REGION_MANAGER`).
//!
//! Verified on Attila's main campaign: 94 of 94 province managers match in their regions and 93 in their main region,
//! and 25 of 25 sea regions match. The game lists managers and regions in orders derived from its hash-ordered region
//! array; these functions use table and map order instead, which changes the main region of provinces without capital
//! or major settlement (one rebel province in Attila).

use getset::Getters;

use std::collections::{BTreeMap, HashMap};

use rpfm_lib::error::Result;

use super::CampaignIndex;
use crate::startpos::region_map::RegionMap;
use crate::startpos::settlements::SettlementLayout;
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "provinces_test.rs"] mod provinces_test;

/// Position of the port slot in a settlement layout.
const PORT_SLOT: usize = 1;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A faction's holdings in a province.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters)]
#[getset(get = "pub")]
pub struct ProvinceManager {

    /// Province key.
    province: String,

    /// Faction key, `None` for the rebels.
    faction: Option<String>,

    /// Keys of the faction's regions in the province, in `region_to_province_junctions` order.
    regions: Vec<String>,

    /// The faction's main region in the province: its capital if there, else the major settlement, else the first in
    /// table order.
    main_region: String,
}

/// A sea region and the factions with ports on it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters)]
#[getset(get = "pub")]
pub struct SeaRegion {

    /// Sea region key.
    key: String,

    /// Port regions on the sea, grouped by faction (`None` for the rebels), sorted by region key.
    ports: Vec<(Option<String>, Vec<String>)>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the province managers: one per faction (or the rebels) owning regions in a province.
///
/// # Arguments
///
/// * `tables` - Tables with `region_to_province_junctions`.
/// * `index` - The campaign's startpos rows.
///
/// # Returns
///
/// The managers, by province in table order, then by faction in order of their first region.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn province_managers(tables: &StartposTables, index: &CampaignIndex) -> Result<Vec<ProvinceManager>> {
    let mut managers = Vec::<ProvinceManager>::new();
    for row in tables.table("region_to_province_junctions")?.rows() {
        let Some(region) = index.region(&row.str("region")?) else { continue };
        let province = row.str("province")?;
        let faction = region.owner().to_owned();

        match managers.iter_mut().find(|manager| manager.province == province && manager.faction == faction) {
            Some(manager) => manager.regions.push(region.key().to_owned()),
            None => managers.push(ProvinceManager {
                province: province.into_owned(),
                faction,
                regions: vec![region.key().to_owned()],
                main_region: String::new(),
            }),
        }
    }

    for manager in &mut managers {
        let capital = manager.faction.as_deref().and_then(|faction| index.capital_of(faction)).map(|capital| capital.key().as_str());
        let is_major = |region: &str| index.settlement_of(region).is_some_and(|settlement| settlement.is_major());
        manager.main_region = manager.regions.iter()
            .find(|region| Some(region.as_str()) == capital)
            .or_else(|| manager.regions.iter().find(|region| is_major(region)))
            .or_else(|| manager.regions.first())
            .cloned()
            .unwrap_or_default();
    }

    Ok(managers)
}

/// Returns the sea regions with ports: every sea a port zone cell lies in, with the port regions of each faction.
///
/// # Arguments
///
/// * `map` - Region map of the campaign map.
/// * `layouts` - Settlement layouts of the campaign map (their port slot zones).
/// * `index` - The campaign's startpos rows.
///
/// # Returns
///
/// The seas in map order. Seas without ports (rivers, lakes) are left out.
pub fn sea_regions(map: &RegionMap, layouts: &[SettlementLayout], index: &CampaignIndex) -> Vec<SeaRegion> {
    let mut seas = BTreeMap::<usize, BTreeMap<String, Option<String>>>::new();
    let map_position = map.keys().iter().enumerate().map(|(position, key)| (key.as_str(), position)).collect::<HashMap<_, _>>();

    for layout in layouts {
        let Some(region) = index.region(layout.region()) else { continue };
        for cell in layout.slots().get(PORT_SLOT).map(|slot| slot.zone().as_slice()).unwrap_or_default() {
            let Some(sea) = map.region_at(*cell) else { continue };
            if index.region(sea).is_some() {
                continue;
            }
            if let Some(&position) = map_position.get(sea) {
                seas.entry(position).or_default().insert(region.key().to_owned(), region.owner().to_owned());
            }
        }
    }

    seas.into_iter().map(|(position, ports)| {
        let mut by_faction = Vec::<(Option<String>, Vec<String>)>::new();
        for (region, faction) in ports {
            match by_faction.iter_mut().find(|(owner, _)| *owner == faction) {
                Some((_, regions)) => regions.push(region),
                None => by_faction.push((faction, vec![region])),
            }
        }
        SeaRegion { key: map.keys()[position].to_owned(), ports: by_faction }
    }).collect()
}
