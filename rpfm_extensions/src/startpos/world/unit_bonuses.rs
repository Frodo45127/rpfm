//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Equipment stat bonuses of garrison units from their settlements' buildings
//! (`UNIT/UNIT_STATISTICS_BONUSES/STATS_BLOCK`).
//!
//! The game computes a unit's bonus for each equipment stat (`FUN_10b90310`) as the largest of its sources: the
//! effects with the stat's basic bonus value, the ones for the unit's category and the ones for its class
//! (`FUN_10b86000`). The stats are, by id, armour, shield, weapon, ammunition and horses. A garrison takes the
//! effects of its faction's buildings in the province (`province_to_province_own`,
//! `region_to_region_own_provincewide`) and in its own region (`region_to_region_own`). Verified on Thrones: the 166
//! bonuses of its 83 garrison units with any (forges and military ports: bronze armour and weapons +5).

use std::collections::HashMap;

use rpfm_lib::error::Result;

use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "unit_bonuses_test.rs"] mod unit_bonuses_test;

/// Bonus value ids of the equipment stats, by stat id.
const STAT_BONUS_IDS: [&str; 5] = ["unit_stat_bonus_armour", "unit_stat_bonus_shield", "unit_stat_bonus_weapon", "unit_stat_bonus_ammunition", "unit_stat_bonus_horses"];

/// Effect scopes reaching every region of the province the building is in.
const PROVINCE_SCOPES: [&str; 2] = ["province_to_province_own", "region_to_region_own_provincewide"];

/// Effect scope reaching only the building's region.
const REGION_SCOPE: &str = "region_to_region_own";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Which units a stat effect applies to, by bonus source.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {

    /// Every unit (`effect_bonus_value_basic_junction`).
    Basic,

    /// Units of a category (`effect_bonus_value_unit_category_junction`).
    Category(String),

    /// Units of a class (`effect_bonus_value_unit_class_junction`).
    Class(String),
}

/// A building's effect on an equipment stat.
#[derive(Debug, Clone, PartialEq)]
struct StatEffect {

    /// Stat id (position in [`STAT_BONUS_IDS`]).
    stat: u32,

    /// Units it applies to.
    target: Target,

    /// Bonus.
    value: i64,

    /// It reaches the whole province, not only the building's region.
    provincewide: bool,
}

/// What the stat bonuses of units are worked out from.
#[derive(Debug, Clone, Default)]
pub struct StatBonusRules {

    /// Stat effects of each building level.
    effects: HashMap<String, Vec<StatEffect>>,

    /// Category and class of each unit, by `main_units` key.
    kinds: HashMap<String, (String, String)>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl StatBonusRules {

    /// Collects the stat effects of the buildings and the categories and classes of the units.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(tables: &StartposTables) -> Result<Self> {
        let sources = [
            ("effect_bonus_value_basic_junction", None),
            ("effect_bonus_value_unit_category_junction", Some("unit_category")),
            ("effect_bonus_value_unit_class_junction", Some("unit_class")),
        ];
        let mut targets = HashMap::<String, Vec<(u32, Target)>>::new();
        for (table, column) in sources {
            for row in tables.table(table)?.rows() {
                let bonus = row.str("bonus_value_id")?;
                let Some(stat) = STAT_BONUS_IDS.iter().position(|id| *id == bonus) else { continue };
                let target = match column {
                    Some("unit_category") => Target::Category(row.str("unit_category")?.into_owned()),
                    Some(column) => Target::Class(row.str(column)?.into_owned()),
                    None => Target::Basic,
                };
                targets.entry(row.str("effect")?.into_owned()).or_default().push((stat as u32, target));
            }
        }

        let mut effects = HashMap::<String, Vec<StatEffect>>::new();
        for row in tables.table("building_effects_junction")?.rows() {
            let Some(effect_targets) = targets.get(row.str("effect")?.as_ref()) else { continue };
            let scope = row.str("effect_scope")?;
            let provincewide = PROVINCE_SCOPES.contains(&scope.as_ref());
            if !provincewide && scope != REGION_SCOPE {
                continue;
            }
            let value = row.number("value")? as i64;
            let building = effects.entry(row.str("building")?.into_owned()).or_default();
            building.extend(effect_targets.iter().map(|(stat, target)| StatEffect { stat: *stat, target: target.clone(), value, provincewide }));
        }

        let mut unit_kinds = HashMap::new();
        for table in ["land_units", "naval_units"] {
            for row in tables.table(table)?.rows() {
                unit_kinds.insert(row.str("key")?.into_owned(), (row.str("category")?.into_owned(), row.str("class")?.into_owned()));
            }
        }
        let mut kinds = HashMap::new();
        for row in tables.table("main_units")?.rows() {
            let column = if row.bool("is_naval")? { "naval_unit" } else { "land_unit" };
            if let Some(kind) = unit_kinds.get(row.str(column)?.as_ref()) {
                kinds.insert(row.str("unit")?.into_owned(), kind.clone());
            }
        }

        Ok(Self { effects, kinds })
    }

    /// Returns the stat bonuses of a garrison unit.
    ///
    /// # Arguments
    ///
    /// * `unit` - `main_units` key.
    /// * `buildings` - The faction's buildings in the garrison's province, each with whether it's in the garrison's
    ///   own region.
    ///
    /// # Returns
    ///
    /// The (stat id, bonus) pairs with a bonus, by stat id.
    pub fn bonuses<'b>(&self, unit: &str, buildings: impl IntoIterator<Item = (&'b str, bool)>) -> Vec<(u32, i64)> {
        let Some((category, class)) = self.kinds.get(unit) else { return vec![] };
        let mut sums = HashMap::<(u32, u8), i64>::new();
        for (building, own_region) in buildings {
            for effect in self.effects.get(building).into_iter().flatten().filter(|effect| effect.provincewide || own_region) {
                let source = match &effect.target {
                    Target::Basic => 0,
                    Target::Category(target) if target == category => 1,
                    Target::Class(target) if target == class => 2,
                    _ => continue,
                };
                *sums.entry((effect.stat, source)).or_default() += effect.value;
            }
        }

        (0..STAT_BONUS_IDS.len() as u32)
            .filter_map(|stat| (0..3).filter_map(|source| sums.get(&(stat, source)).copied()).max().filter(|value| *value > 0).map(|value| (stat, value)))
            .collect()
    }
}
