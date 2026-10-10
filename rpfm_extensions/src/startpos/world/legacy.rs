//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Army legacies (`MILITARY_FORCE/MILITARY_FORCE_LEGACY`): the name, kind, emblem and number of each force.
//!
//! Garrisons get fixed names. Generals get their `legacy_override` text if the character has one, otherwise a name
//! of their subculture never used before in the faction. Generals also get an emblem. The game draws the names and
//! emblems from its RNG; these functions take the first unused ones in table order. Verified on Attila's main
//! campaign: kinds, override names and numbers match for 343 of 343 forces, and every drawn name and emblem comes
//! from the expected pool without repeats.

use getset::{CopyGetters, Getters};

use std::collections::HashSet;

use rpfm_lib::error::Result;

use super::{CampaignIndex, FactionEntry};
use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "legacy_test.rs"] mod legacy_test;

/// Localisation key prefix of the names of `military_force_legacy_names`.
const NAME_KEY_PREFIX: &str = "military_force_legacy_names_localised_name_";

/// Localisation key prefix of the `start_pos_characters.legacy_override` texts.
const OVERRIDE_KEY_PREFIX: &str = "start_pos_characters_legacy_override_";

/// Name of garrison armies.
const GARRISON_ARMY_NAME: &str = "random_localisation_strings_string_military_force_legacy_name_garrison_army";

/// Name of garrison navies.
const GARRISON_NAVY_NAME: &str = "random_localisation_strings_string_military_force_legacy_name_garrison_navy";

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// The kind of a force, as the legacy stores it (`MILITARY_FORCE_LEGACY` #4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceKind {

    /// A general's army.
    Army = 0,

    /// A general's navy.
    Navy = 1,

    /// A settlement's garrison army.
    GarrisonArmy = 2,

    /// A settlement's garrison navy.
    GarrisonNavy = 3,
}

/// A force of a faction, as needed to name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyForce {

    /// Kind of force.
    pub kind: ForceKind,

    /// `start_pos_characters` id of the commanding general, for armies and navies.
    pub general: Option<String>,
}

/// The legacy of a force.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct Legacy {

    /// Localisation key of the force's name.
    #[getset(get = "pub")]
    name: String,

    /// The name's key again for forces named by a `legacy_override`, empty otherwise (`MILITARY_FORCE_LEGACY` #3).
    #[getset(get = "pub")]
    override_name: String,

    /// Emblem key, empty for garrisons.
    #[getset(get = "pub")]
    emblem: String,

    /// Force number: overridden legacies are numbered 1, 2... in force order; the others are 1.
    #[getset(get_copy = "pub")]
    number: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

/// Returns the legacies of a faction's forces.
///
/// # Arguments
///
/// * `tables` - Tables with `military_force_legacy_names` and `military_force_legacy_emblems`.
/// * `index` - The campaign's startpos rows.
/// * `faction` - The faction.
/// * `forces` - The faction's forces, in startpos order.
/// * `overrides` - `start_pos_characters` ids of the characters with a `legacy_override` text (a localised column,
///   so it comes from the campaign's localisation, not from the table).
///
/// # Returns
///
/// One legacy per force, in the same order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn legacies(tables: &StartposTables, index: &CampaignIndex, faction: &FactionEntry, forces: &[LegacyForce], overrides: &HashSet<String>) -> Result<Vec<Legacy>> {
    let mut army_names = name_pool(tables, index, faction, true)?.into_iter();
    let mut navy_names = name_pool(tables, index, faction, false)?.into_iter();
    let mut army_emblems = emblem_pool(tables, index, faction, true)?.into_iter();
    let mut navy_emblems = emblem_pool(tables, index, faction, false)?.into_iter();
    let mut override_count = 0;

    Ok(forces.iter().map(|force| {
        let overridden = force.general.as_ref().filter(|general| overrides.contains(*general));
        match (force.kind, overridden) {
            (ForceKind::GarrisonArmy, _) => Legacy { name: GARRISON_ARMY_NAME.to_owned(), number: 1, ..Default::default() },
            (ForceKind::GarrisonNavy, _) => Legacy { name: GARRISON_NAVY_NAME.to_owned(), number: 1, ..Default::default() },
            (kind, Some(general)) => {
                override_count += 1;
                let name = format!("{OVERRIDE_KEY_PREFIX}{general}");
                Legacy {
                    override_name: name.to_owned(),
                    name,
                    emblem: next_emblem(kind, &mut army_emblems, &mut navy_emblems),
                    number: override_count,
                }
            },
            (kind, None) => Legacy {
                name: if kind == ForceKind::Navy { navy_names.next() } else { army_names.next() }.unwrap_or_default(),
                emblem: next_emblem(kind, &mut army_emblems, &mut navy_emblems),
                number: 1,
                ..Default::default()
            },
        }
    }).collect())
}

/// Returns the skill set of a faction's armies or navies (`MILITARY_FORCE_LEGACY/CAMPAIGN_SKILLS` #1).
///
/// The key replaces `sub_cult_` in the subculture key with `army_legacy_` or `navy_legacy_`
/// (`att_sub_cult_roman_west` gives `att_army_legacy_roman_west`).
pub fn legacy_skill_set(faction: &FactionEntry, navy: bool) -> String {
    faction.subculture().replace("sub_cult_", if navy { "navy_legacy_" } else { "army_legacy_" })
}

/// Returns the name localisation keys available to a faction's armies or navies.
///
/// # Arguments
///
/// * `tables` - Tables with `military_force_legacy_names`.
/// * `index` - The campaign's startpos rows.
/// * `faction` - The faction.
/// * `for_army` - `true` for army names, `false` for navy names.
///
/// # Returns
///
/// The keys, in table order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn name_pool(tables: &StartposTables, index: &CampaignIndex, faction: &FactionEntry, for_army: bool) -> Result<Vec<String>> {
    let mut names = vec![];
    for row in tables.table("military_force_legacy_names")?.rows() {
        let campaign = row.str("campaign")?;
        if row.str("subculture")? == *faction.subculture() && row.bool("for_army")? == for_army && (campaign.is_empty() || campaign == *index.campaign()) {
            names.push(format!("{NAME_KEY_PREFIX}{}{}", row.str("key")?, faction.subculture()));
        }
    }
    Ok(names)
}

/// Returns the emblems available to a faction's armies or navies.
///
/// They're the rows of the faction, of its subculture (rows without faction) and of its culture (rows without
/// faction and subculture), with no campaign or the current one.
///
/// # Arguments
///
/// * `tables` - Tables with `military_force_legacy_emblems`.
/// * `index` - The campaign's startpos rows.
/// * `faction` - The faction.
/// * `is_army` - `true` for army emblems, `false` for navy emblems.
///
/// # Returns
///
/// The emblem keys, in table order.
///
/// # Errors
///
/// Returns an error if a table or column is missing.
pub fn emblem_pool(tables: &StartposTables, index: &CampaignIndex, faction: &FactionEntry, is_army: bool) -> Result<Vec<String>> {
    let mut emblems = vec![];
    for row in tables.table("military_force_legacy_emblems")?.rows() {
        let campaign = row.str("campaign")?;
        let row_faction = row.str("faction_key")?;
        let row_subculture = row.str("subculture_key")?;
        let matches = row_faction == *faction.key()
            || (row_faction.is_empty() && row_subculture == *faction.subculture())
            || (row_faction.is_empty() && row_subculture.is_empty() && row.str("culure_key")? == *faction.culture());
        if matches && row.bool("is_army")? == is_army && (campaign.is_empty() || campaign == *index.campaign()) {
            emblems.push(row.str("key")?.into_owned());
        }
    }
    Ok(emblems)
}

/// Takes the next unused emblem for a general's army or navy.
fn next_emblem(kind: ForceKind, army: &mut impl Iterator<Item = String>, navy: &mut impl Iterator<Item = String>) -> String {
    if kind == ForceKind::Navy { navy.next() } else { army.next() }.unwrap_or_default()
}
