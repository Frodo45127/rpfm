//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the family tree (`WORLD/FAMILY_TREE`): a member per family id, in id order. Characters of the world
//! only hold their links; dead table characters also hold their details.
//!
//! Links come from `start_pos_family_relationships`, in table order: a child points to its father or mother (by the
//! parent's gender) and is listed among the parent's children (adopted ones in their own list); living spouses point
//! to each other, and spouses of the dead list each other as former spouses. Verified on Attila's main campaign:
//! every relationship the game keeps lands where it does in the game's tree (the game drops a few conflicting rows
//! of one family, and orders the lists by its own link order).

use std::collections::HashMap;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{NodeType, RecordNode};

use super::{CharacterInfo, CharacterWriter, DetailsSubject};
use crate::startpos::layout::child_path;
use crate::startpos::world::{CharacterEntry, FactionEntry};
use crate::startpos::writer::{bool_node, u32_node};

#[cfg(test)] #[path = "family_test.rs"] mod family_test;

/// Path of the family tree.
const FAMILY_TREE_PATH: &str = "CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FAMILY_TREE";

/// Age from which living members count as adults.
const ADULT_AGE: i64 = 16;

/// Death type codes of the dead (`CHARACTER_DETAILS` `DATE@2+1`), by `start_pos_characters.death_type`.
const DEATH_TYPE_CODES: [(&str, i32); 4] = [("KILLED_IN_ACTION", 1), ("OLD_AGE", 3), ("EXECUTED", 5), ("DISEASE", 17)];

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Links and flags of a family member.
#[derive(Debug, Clone, Default)]
struct Member {
    mother: u32,
    father: u32,
    spouse: u32,
    former_spouses: Vec<u32>,
    children: Vec<u32>,
    adopted_children: Vec<u32>,
    bastard: bool,
    adopted: bool,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl CharacterWriter<'_, '_> {

    /// Writes the family tree.
    ///
    /// # Errors
    ///
    /// Returns an error if a table, column or layout is missing.
    pub fn write_family_tree(&self) -> Result<RecordNode> {
        let index = self.inputs.index;
        let entries = index.characters().iter().filter_map(|entry| self.entry_family_ids.get(entry.id()).map(|id| (*id, entry))).collect::<HashMap<_, _>>();
        let members = self.family_links(&entries)?;
        let count = self.family_ids.iter().chain(self.entry_family_ids.values()).copied().max().unwrap_or_default();

        let mut children = vec![u32_node(count)];
        for id in 1..=count {
            let member = members.get(&id).cloned().unwrap_or_default();
            children.push(NodeType::Record(Box::new(self.write_member(id, entries.get(&id).copied(), &member)?)));
        }

        // Version 2 trees end with a list of ids (count, then ids), empty at the start.
        children.push(u32_node(0));
        self.inputs.layout.raw_record(FAMILY_TREE_PATH, children)
    }

    /// Works out the links of the members from `start_pos_family_relationships`.
    fn family_links(&self, entries: &HashMap<u32, &CharacterEntry>) -> Result<HashMap<u32, Member>> {
        let mut members = HashMap::<u32, Member>::new();
        let is_dead = |id: u32| entries.get(&id).is_some_and(|entry| !entry.death_type().is_empty());
        for row in self.inputs.tables.table("start_pos_family_relationships")?.rows() {
            let (Some(&character), Some(&related)) = (self.entry_family_ids.get(row.str("character")?.as_ref()), self.entry_family_ids.get(row.str("related_to")?.as_ref())) else { continue };
            match row.str("relationship")?.as_ref() {
                "child" => {
                    let adopted = row.bool("adopted")?;
                    let parent_is_male = entries.get(&related).and_then(|entry| self.rows.get(entry.id())).map(|row| row.bool("is_male")).transpose()?.unwrap_or(true);
                    let member = members.entry(character).or_default();
                    if parent_is_male { member.father = related } else { member.mother = related }
                    member.bastard |= row.bool("bastard")?;
                    member.adopted |= adopted;
                    let parent = members.entry(related).or_default();
                    let list = if adopted { &mut parent.adopted_children } else { &mut parent.children };
                    if !list.contains(&character) {
                        list.push(character);
                    }
                },
                "spouse" if is_dead(character) || is_dead(related) => {
                    for (member, other) in [(character, related), (related, character)] {
                        let former = &mut members.entry(member).or_default().former_spouses;
                        if !former.contains(&other) {
                            former.push(other);
                        }
                    }
                },
                "spouse" => {
                    members.entry(character).or_default().spouse = related;
                    members.entry(related).or_default().spouse = character;
                },
                _ => {},
            }
        }
        Ok(members)
    }

    /// Writes a family member. Dead table characters carry their faction and details; generated characters only
    /// their id.
    fn write_member(&self, id: u32, entry: Option<&CharacterEntry>, member: &Member) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let path = child_path(FAMILY_TREE_PATH, "FAMILY_MEMBER");
        let date = |year: u32| -> Result<NodeType> {
            let mut record = layout.builder(&child_path(&path, "DATE"))?;
            record.group()?.set_u32("^+1", year)?;
            Ok(NodeType::Record(Box::new(record.build()?)))
        };
        let list = |ids: &[u32]| std::iter::once(u32_node(ids.len() as u32)).chain(ids.iter().map(|id| u32_node(*id))).collect::<Vec<_>>();

        let dead = entry.filter(|entry| !entry.death_type().is_empty());
        let mut children = vec![u32_node(id), bool_node(dead.is_some())];
        if let Some(entry) = dead {
            let faction = self.inputs.index.faction(entry.faction()).ok_or_else(|| missing_faction(entry))?;
            children.push(NodeType::Ascii(faction.key().to_owned()));
            children.push(NodeType::Record(Box::new(self.write_dead_details(id, entry, faction, &path)?)));
        }

        let married = member.spouse != 0 || !member.former_spouses.is_empty();
        let adult = entry.filter(|entry| entry.death_type().is_empty()).is_none_or(|entry| entry.age() >= ADULT_AGE);
        children.extend([u32_node(0), u32_node(member.mother), u32_node(member.father), u32_node(member.spouse)]);
        children.push(date(if married { self.start_year } else { 0 })?);
        children.extend(list(&member.former_spouses));
        children.extend(list(&member.children));
        children.extend(list(&member.adopted_children));
        children.extend([bool_node(false), date(0)?, u32_node(0), bool_node(adult), bool_node(false), bool_node(member.bastard), bool_node(false)]);
        children.extend([u32_node(0), u32_node(u32::MAX), bool_node(false), bool_node(entry.is_some_and(|entry| entry.progenitor())), bool_node(member.adopted)]);
        children.push(NodeType::Ascii(String::new()));
        layout.raw_record(&path, children)
    }

    /// Writes the details of a dead table character: born and died the given number of years before the start.
    fn write_dead_details(&self, family_id: u32, entry: &CharacterEntry, faction: &FactionEntry, member_path: &str) -> Result<RecordNode> {
        let row = self.rows.get(entry.id());
        // The column is an optional string.
        let died_before = row.map(|row| row.str("turns_died_before_start")).transpose()?.and_then(|value| value.parse::<i64>().ok()).unwrap_or_default();
        let death_year = u32::try_from(i64::from(self.start_year) - died_before).unwrap_or_default();
        let info = CharacterInfo {
            agent: entry.agent().to_owned(),
            is_male: row.map(|row| row.bool("is_male")).transpose()?.unwrap_or(true),
            birth_year: u32::try_from(i64::from(death_year) - entry.age()).unwrap_or_default(),
            row,
        };
        let code = DEATH_TYPE_CODES.iter().find(|(death_type, _)| *death_type == entry.death_type()).map(|(_, code)| *code).unwrap_or_default();
        let path = child_path(member_path, "CHARACTER_DETAILS");
        let subject = DetailsSubject { path: &path, faction, entry: Some(entry), character_id: 0, family_id, death: Some((death_year, code)), bodyguard: None };
        self.write_details(&subject, &info)
    }
}

/// Builds the error for a character whose faction isn't in the campaign.
fn missing_faction(entry: &CharacterEntry) -> RLibError {
    RLibError::StartposInvalidValue("start_pos_characters".to_owned(), "faction".to_owned(), entry.faction().to_owned())
}
