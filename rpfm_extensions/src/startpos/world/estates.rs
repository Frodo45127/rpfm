//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Estates of the region slots (`REGION_SLOT/REGION_BUILDING_MANAGER/ESTATES`, Thrones of Britannia only).
//!
//! Each building gives its slot the estates of its `amount` effects (`building_effects_junction` effects that
//! `effect_bonus_value_estate_junctions` maps to an estate type with the `amount` bonus value), as many of each
//! type as the effect's value. The `start_pos_character_estates` rows give the nth estate of a type in a region to
//! a character; the others belong to the faction leader of the building's owner. Verified on Thrones' campaign:
//! the estates of 598 of 598 slots, and the owners of 97 of 97 estates (77 leaders, 20 from the table).

use std::collections::HashMap;

use rpfm_lib::error::Result;

use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "estates_test.rs"] mod estates_test;

/// Bonus value of the effects that give estates.
const AMOUNT_BONUS_VALUE: &str = "amount";

/// What the startpos tables say about estates.
#[derive(Debug, Clone, Default)]
pub struct EstateRules {

    /// Estate types and amounts each building level gives, in table order.
    provided: HashMap<String, Vec<(String, u32)>>,

    /// `start_pos_characters` id of the owner of the nth estate of a type in a region, by (region, type, n).
    owners: HashMap<(String, String, u32), String>,
}

impl EstateRules {

    /// Reads the rules from the tables. Games without estates get none.
    ///
    /// # Errors
    ///
    /// Returns an error if a table or column is missing.
    pub fn new(tables: &StartposTables) -> Result<Self> {
        let mut types = HashMap::new();
        for row in tables.table("effect_bonus_value_estate_junctions")?.rows() {
            if row.str("bonus_value_id")? == AMOUNT_BONUS_VALUE {
                types.insert(row.str("effect")?.into_owned(), row.str("estate")?.into_owned());
            }
        }

        let mut provided = HashMap::<String, Vec<(String, u32)>>::new();
        if !types.is_empty() {
            for row in tables.table("building_effects_junction")?.rows() {
                if let Some(estate) = types.get(row.str("effect")?.as_ref()) {
                    let amount = row.number("value")?.max(0.0) as u32;
                    provided.entry(row.str("building")?.into_owned()).or_default().push((estate.to_owned(), amount));
                }
            }
        }

        let mut owners = HashMap::new();
        for row in tables.table("start_pos_character_estates")?.rows() {
            let number = u32::try_from(row.int("number")?).unwrap_or_default();
            owners.insert((row.str("region")?.into_owned(), row.str("estate_type")?.into_owned(), number), row.str("character")?.into_owned());
        }
        Ok(Self { provided, owners })
    }

    /// Returns the estate types a building level gives, one entry per estate, in table order.
    pub fn estates_of(&self, building: &str) -> Vec<&str> {
        self.provided.get(building).into_iter().flatten()
            .flat_map(|(estate, amount)| std::iter::repeat_n(estate.as_str(), *amount as usize))
            .collect()
    }

    /// Returns the `start_pos_characters` id of the owner the table gives the nth estate of a type in a region.
    pub fn table_owner(&self, region: &str, estate: &str, number: u32) -> Option<&str> {
        self.owners.get(&(region.to_owned(), estate.to_owned(), number)).map(String::as_str)
    }
}
