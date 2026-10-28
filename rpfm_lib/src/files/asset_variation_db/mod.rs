//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Asset variation databases (`warscape_asset_variation_db.bin`): which environment file a battle uses.
//!
//! The database has a list of categories (map, season, precipitation...) with their values, and per key
//! (`environment`) a list of files with the flags of the category values each one allows. When a battle starts,
//! the game picks one of the files whose flags allow the battle's values.
//!
//! # File Format
//!
//! FASTBIN0, little endian, with `str` being a u16 size and that many bytes:
//!
//! ```text
//! [8 bytes]  FASTBIN0 signature
//! [u16]      serialise_version (1)
//! [u32]      category count, then per category: u16 version (1), str name, u32 value count, str values
//! [u32]      entry count, then per entry: str key, u16 version (1), u32 variation count,
//!            and per variation: u16 version (1), str filename, u32 flags
//! ```
//!
//! The packs also have the database in XML form, next to the binary one, and the Assembly Kit has its source
//! (`environment_map.xml`). See [`AssetVariationDb::to_xml`], [`AssetVariationDb::from_xml`] and
//! [`AssetVariationDb::from_environment_map`].
//!
//! # Selection
//!
//! As the game does it:
//!
//! - Every value has a bit, counting through the values of all categories in order: `1 << (index & 31)`.
//! - A query ORs the bit of the value chosen for each category. The game finds a value by its name alone, searching
//!   the values of all categories in order, so the first category that has the name gives the bit.
//! - A query with a value missing from every list matches nothing.
//! - A variation matches when its flags are 0, or when they have every bit of the query.
//! - If nothing matches, the game queries again with the map `default_map`, and picks one of the matches at random.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::BTreeMap;

use crate::error::{Result, RLibError};

/// Path of the database in the packs.
pub const PATH: &str = "warscape_asset_variation_db/warscape_asset_variation_db.bin";

/// Path of the XML form of the database in the packs.
pub const PATH_XML: &str = "warscape_asset_variation_db/warscape_asset_variation_db.xml";

/// FASTBIN0 file signature.
pub const SIGNATURE: &[u8; 8] = b"FASTBIN0";

/// Category of the map, and the value the game uses when nothing matches the battle's map.
const MAP_CATEGORY: &str = "map";
const DEFAULT_MAP: &str = "default_map";

/// Amount of bits of the flags: all the values of all categories together can't have more.
const FLAG_BITS: usize = 32;

mod v1;
mod xml;

#[cfg(test)] mod asset_variation_db_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// An asset variation database.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AssetVariationDb {

    /// Version of the format.
    serialise_version: u16,

    /// Categories of the conditions of a battle, with their values.
    categories: Vec<VariationCategory>,

    /// Lists of variations, by key.
    entries: Vec<VariationEntry>,
}

/// A category of the conditions of a battle, like `season`, with its values.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VariationCategory {

    /// Name of the category.
    name: String,

    /// Values of the category, in bit order.
    values: Vec<String>,
}

/// The variations of a key, like `environment`.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VariationEntry {

    /// Key of the entry.
    key: String,

    /// Variations the game picks from.
    variations: Vec<Variation>,
}

/// A file the game may pick, with the values it allows.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Variation {

    /// Path of the file, with backslashes.
    filename: String,

    /// Bits of the values the variation allows. 0 allows everything.
    flags: u32,
}

/// The variations of a key that match a query.
#[derive(Default, PartialEq, Clone, Debug)]
pub struct VariationMatches<'a> {

    /// The matching variations, in database order.
    pub variations: Vec<&'a Variation>,

    /// If nothing matched the map of the query, and the game asked again with the default map.
    pub used_default_map: bool,
}

/// A query: the value chosen for each category. Categories left out, or with an empty value, add no bit.
pub type VariationQuery = BTreeMap<String, String>;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl AssetVariationDb {

    /// Amount of values of all the categories together.
    pub fn value_count(&self) -> usize {
        self.categories.iter().map(|category| category.values.len()).sum()
    }

    /// Returns the category with the provided name, if any.
    pub fn category(&self, name: &str) -> Option<&VariationCategory> {
        self.categories.iter().find(|category| category.name == name)
    }

    /// Returns the entry with the provided key, if any.
    pub fn entry(&self, key: &str) -> Option<&VariationEntry> {
        self.entries.iter().find(|entry| entry.key == key)
    }

    /// Index of the first bit of a category, or None if there's no category with that name.
    fn first_bit(&self, name: &str) -> Option<usize> {
        let mut index = 0;
        for category in &self.categories {
            if category.name == name {
                return Some(index);
            }

            index += category.values.len();
        }

        None
    }

    /// Bit of a value of a category, or 0 if the category or the value don't exist.
    pub fn bit(&self, category: &str, value: &str) -> u32 {
        match (self.category(category), self.first_bit(category)) {
            (Some(category), Some(first)) => category.values.iter()
                .position(|known| known == value)
                .map_or(0, |index| 1 << ((first + index) % FLAG_BITS)),
            _ => 0,
        }
    }

    /// Bit the game gives a value: that of the first value with this name in any category, or None if no category has it.
    pub fn value_bit(&self, value: &str) -> Option<u32> {
        self.categories.iter()
            .flat_map(|category| &category.values)
            .position(|known| known == value)
            .map(|index| 1 << (index % FLAG_BITS))
    }

    /// Bits of a query, or None if one of its values is in no category, which makes the game match nothing.
    pub fn query_mask(&self, query: &VariationQuery) -> Option<u32> {
        query.values()
            .filter(|value| !value.is_empty())
            .try_fold(0, |mask, value| self.value_bit(value).map(|bit| mask | bit))
    }

    /// Returns the variations of a key that match a query, as the game finds them.
    ///
    /// # Arguments
    ///
    /// * `key` - Key of the entry, like `environment`.
    /// * `query` - Value chosen for each category.
    ///
    /// # Returns
    ///
    /// The matches. If nothing matches, they're the ones of the same query with the map `default_map`.
    pub fn matches(&self, key: &str, query: &VariationQuery) -> VariationMatches<'_> {
        let mut matches = VariationMatches::default();
        let Some(entry) = self.entry(key) else { return matches };

        let find = |query: &VariationQuery| match self.query_mask(query) {
            Some(mask) => entry.variations.iter()
                .filter(|variation| variation.flags == 0 || variation.flags & mask == mask)
                .collect::<Vec<_>>(),
            None => vec![],
        };

        matches.variations = find(query);
        if matches.variations.is_empty() {
            let mut query = query.clone();
            query.insert(MAP_CATEGORY.to_owned(), DEFAULT_MAP.to_owned());
            matches.variations = find(&query);
            matches.used_default_map = true;
        }

        matches
    }

    /// Returns, per category, the values allowed by some flags. Flags 0 allow every value.
    pub fn allowed_values(&self, flags: u32) -> Vec<(&str, Vec<&str>)> {
        let mut index = 0;
        self.categories.iter().map(|category| {
            let allowed = category.values.iter().enumerate()
                .filter(|(value, _)| flags == 0 || flags >> ((index + value) % FLAG_BITS) & 1 == 1)
                .map(|(_, value)| value.as_str())
                .collect();

            index += category.values.len();
            (category.name.as_str(), allowed)
        }).collect()
    }

    /// Returns the flags that allow exactly the provided values of each category. Categories left out allow all their values.
    pub fn flags_for(&self, allowed: &BTreeMap<String, Vec<String>>) -> u32 {
        let mut flags = 0;
        let mut index = 0;
        for category in &self.categories {
            let allowed = allowed.get(&category.name);
            for (value_index, value) in category.values.iter().enumerate() {
                if allowed.is_none_or(|allowed| allowed.contains(value)) {
                    flags |= 1 << ((index + value_index) % FLAG_BITS);
                }
            }

            index += category.values.len();
        }

        flags
    }

    /// Checks the database for what makes it wrong for the game.
    ///
    /// # Returns
    ///
    /// A description of each problem: more values than bits, repeated names or values, no `default_map`, flags with bits
    /// of no value, and variations that can never match.
    pub fn validate(&self) -> Vec<String> {
        let mut problems = vec![];
        let total = self.value_count();
        if total > FLAG_BITS {
            problems.push(format!("{total} values in all categories: the flags have {FLAG_BITS} bits, so bits would collide"));
        }

        for (index, category) in self.categories.iter().enumerate() {
            if self.categories[index + 1..].iter().any(|other| other.name == category.name) {
                problems.push(format!("category '{}' appears twice", category.name));
            }

            for (value_index, value) in category.values.iter().enumerate() {
                if category.values[value_index + 1..].contains(value) {
                    problems.push(format!("value '{value}' appears twice in '{}'", category.name));
                }

                if let Some(first) = self.categories[..index].iter().find(|other| other.values.contains(value)) {
                    problems.push(format!(
                        "value '{value}' is in '{}' and '{}': the game finds a value by its name in all categories, so the one in '{}' is never used",
                        first.name, category.name, category.name
                    ));
                }
            }
        }

        if self.category(MAP_CATEGORY).is_some_and(|map| !map.values.iter().any(|value| value == DEFAULT_MAP)) {
            problems.push(format!("category '{MAP_CATEGORY}' has no value '{DEFAULT_MAP}': the game asks again with it when nothing matched"));
        }

        let used = if total >= FLAG_BITS { u32::MAX } else { (1 << total) - 1 };
        for variation in self.entries.iter().flat_map(|entry| &entry.variations) {
            if variation.flags & !used != 0 {
                problems.push(format!("{}: flags have bits that belong to no value", variation.filename));
            }

            // Flags 0 allow everything.
            if variation.flags != 0 {
                if let Some((category, _)) = self.allowed_values(variation.flags).into_iter().find(|(_, allowed)| allowed.is_empty()) {
                    problems.push(format!("{}: no {category} allowed, it can never match", variation.filename));
                }
            }
        }

        problems
    }

    /// Adds a value to a category, moving the bits of the flags of every variation so they keep allowing the same values.
    ///
    /// A variation that allowed every value of the category, without being unconstrained, also allows the new one.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the category doesn't exist, the value is empty or already
    /// exists in any category (the game finds values by name alone), or every bit of the flags is in use.
    pub fn add_value(&mut self, category: &str, value: &str) -> Result<()> {
        let Some(category_index) = self.categories.iter().position(|known| known.name == category) else {
            return Err(refused(format!("no category '{category}'")));
        };

        let values = &self.categories[category_index].values;
        if value.is_empty() {
            return Err(refused("the value is empty".to_owned()));
        }

        if let Some(owner) = self.category_of(value) {
            return Err(refused(format!("'{value}' already exists in '{owner}'")));
        }

        if self.value_count() >= FLAG_BITS {
            return Err(refused(format!("all {FLAG_BITS} flag bits are in use: remove a value of any category first")));
        }

        let first = self.first_bit(category).unwrap_or_default();
        let count = values.len();
        let new_bit = first + count;
        let moved = (0..self.value_count()).map(|bit| Some(if bit < new_bit { bit } else { bit + 1 })).collect::<Vec<_>>();
        let category_bits = (first..first + count).fold(0u32, |bits, bit| bits | 1 << bit);

        for variation in self.entries.iter_mut().flat_map(|entry| &mut entry.variations) {
            let allowed_all = variation.flags != 0 && count > 0 && variation.flags & category_bits == category_bits;
            variation.flags = move_bits(variation.flags, &moved);
            if allowed_all {
                variation.flags |= 1 << new_bit;
            }
        }

        self.categories[category_index].values.push(value.to_owned());
        Ok(())
    }

    /// Removes a value from a category, moving the bits of the flags of every variation so they keep allowing the same values.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the category or the value don't exist, if the value is the
    /// `default_map` map, or if a variation would allow nothing else, as its flags would become 0, which allows everything.
    pub fn remove_value(&mut self, category: &str, value: &str) -> Result<()> {
        let Some(category_index) = self.categories.iter().position(|known| known.name == category) else {
            return Err(refused(format!("no category '{category}'")));
        };

        let Some(value_index) = self.categories[category_index].values.iter().position(|known| known == value) else {
            return Err(refused(format!("no value '{value}' in '{category}'")));
        };

        if category == MAP_CATEGORY && value == DEFAULT_MAP {
            return Err(refused(format!("the game asks again with map '{DEFAULT_MAP}' when nothing matched, so that value must stay")));
        }

        let removed_bit = self.first_bit(category).unwrap_or_default() + value_index;
        let moved = (0..self.value_count()).map(|bit| match bit.cmp(&removed_bit) {
            std::cmp::Ordering::Less => Some(bit),
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => Some(bit - 1),
        }).collect::<Vec<_>>();

        let variations = self.entries.iter().flat_map(|entry| &entry.variations);
        if let Some(variation) = variations.clone().find(|variation| variation.flags != 0 && move_bits(variation.flags, &moved) == 0) {
            return Err(refused(format!("{} allows nothing else, removing '{value}' would turn it into an unconstrained variation (flags 0)", variation.filename)));
        }

        for variation in self.entries.iter_mut().flat_map(|entry| &mut entry.variations) {
            variation.flags = move_bits(variation.flags, &moved);
        }

        self.categories[category_index].values.remove(value_index);
        Ok(())
    }

    /// Renames a value of a category. The flags don't change.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the category or the value don't exist, or the new name is
    /// empty or already exists in any category.
    pub fn rename_value(&mut self, category: &str, from: &str, to: &str) -> Result<()> {
        if to.is_empty() {
            return Err(refused("the new name is empty".to_owned()));
        }

        if let Some(owner) = self.category_of(to).filter(|_| from != to) {
            return Err(refused(format!("'{to}' already exists in '{owner}'")));
        }

        let Some(category) = self.categories.iter_mut().find(|known| known.name == category) else {
            return Err(refused(format!("no category '{category}'")));
        };

        let Some(value) = category.values.iter_mut().find(|known| *known == from) else {
            return Err(refused(format!("no value '{from}' in '{}'", category.name)));
        };

        *value = to.to_owned();
        Ok(())
    }

    /// Name of the first category that has a value, if any.
    pub fn category_of(&self, value: &str) -> Option<&str> {
        self.categories.iter().find(|category| category.values.iter().any(|known| known == value)).map(|category| category.name.as_str())
    }

    /// Adds a value to a category that every variation allows exactly when it allows another value of the same category,
    /// like a new climate with the weather of an existing one.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if `from` isn't a value of the category, or for the reasons of
    /// [`AssetVariationDb::add_value`].
    pub fn clone_value(&mut self, category: &str, from: &str, value: &str) -> Result<()> {
        if self.category(category).is_none_or(|known| !known.values.iter().any(|known| known == from)) {
            return Err(refused(format!("no value '{from}' in '{category}'")));
        }

        self.add_value(category, value)?;
        let from_bit = self.bit(category, from);
        let new_bit = self.bit(category, value);
        for variation in self.entries.iter_mut().flat_map(|entry| &mut entry.variations).filter(|variation| variation.flags != 0) {
            if variation.flags & from_bit != 0 {
                variation.flags |= new_bit;
            } else {
                variation.flags &= !new_bit;
            }
        }

        Ok(())
    }

    /// Adds an empty category after the others. It takes no bit until it has values.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the name is empty or already used.
    pub fn add_category(&mut self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(refused("the category name is empty".to_owned()));
        }

        if self.category(name).is_some() {
            return Err(refused(format!("category '{name}' already exists")));
        }

        let mut category = VariationCategory::default();
        category.name = name.to_owned();
        self.categories.push(category);
        Ok(())
    }

    /// Removes a category with its values, moving the bits of the flags of every variation so they keep allowing the
    /// same values of the other categories.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the category doesn't exist, or if a variation only
    /// restricts this category, as its flags would become 0, which allows everything.
    pub fn remove_category(&mut self, name: &str) -> Result<()> {
        let Some(category_index) = self.categories.iter().position(|known| known.name == name) else {
            return Err(refused(format!("no category '{name}'")));
        };

        let first = self.first_bit(name).unwrap_or_default();
        let count = self.categories[category_index].values.len();
        let moved = (0..self.value_count()).map(|bit| match bit {
            bit if bit < first => Some(bit),
            bit if bit < first + count => None,
            bit => Some(bit - count),
        }).collect::<Vec<_>>();

        let variations = self.entries.iter().flat_map(|entry| &entry.variations);
        if let Some(variation) = variations.clone().find(|variation| variation.flags != 0 && move_bits(variation.flags, &moved) == 0) {
            return Err(refused(format!("{} only has bits in '{name}', removing it would turn it into an unconstrained variation (flags 0)", variation.filename)));
        }

        for variation in self.entries.iter_mut().flat_map(|entry| &mut entry.variations) {
            variation.flags = move_bits(variation.flags, &moved);
        }

        self.categories.remove(category_index);
        Ok(())
    }

    /// Renames a category. The flags don't change.
    ///
    /// # Errors
    ///
    /// Returns [`RLibError::AssetVariationDbEditRefused`] if the category doesn't exist, or the new name is empty or already used.
    pub fn rename_category(&mut self, from: &str, to: &str) -> Result<()> {
        if to.is_empty() {
            return Err(refused("the new name is empty".to_owned()));
        }

        if from != to && self.category(to).is_some() {
            return Err(refused(format!("category '{to}' already exists")));
        }

        let Some(category) = self.categories.iter_mut().find(|known| known.name == from) else {
            return Err(refused(format!("no category '{from}'")));
        };

        category.name = to.to_owned();
        Ok(())
    }
}

/// Moves every bit of some flags to the index `moved` has for it, dropping the ones it has None for. Flags 0 stay 0.
fn move_bits(flags: u32, moved: &[Option<usize>]) -> u32 {
    if flags == 0 {
        return 0;
    }

    moved.iter().take(FLAG_BITS).enumerate()
        .filter(|(bit, _)| flags >> bit & 1 == 1)
        .filter_map(|(_, to)| *to)
        .fold(0, |out, to| out | 1 << to)
}

/// Error of a refused edit.
fn refused(reason: String) -> RLibError {
    RLibError::AssetVariationDbEditRefused(reason)
}
