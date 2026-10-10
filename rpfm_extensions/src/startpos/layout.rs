//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Layouts of the records of a startpos, used to write new ones.
//!
//! A startpos is a tree of records with dozens of fields each, most of them constant. Instead of building each record
//! by hand, the writer starts from a layout extracted from a game-built startpos: for every record path, its version,
//! its child slots in order (leaves with their most common value, child records always present or optional) and
//! whether its children are an array of groups. Generators then only set the fields that change.
//!
//! Layout files live in the [`STARTPOS_LAYOUT_FOLDER`] folder of RPFM's config folder, one per game and campaign (see
//! [`layout_file_name`]).
//!
//! # Field names
//!
//! Leaves are named after the previous child record that's always present and their position after it, skipping
//! records: `MORGUE+3` is the third leaf after the `MORGUE` record. Leaves before any record use `^` (`^+1`). Optional
//! records don't move the names and optional leaves count even when missing, so names are stable across instances. A record repeated in the same group gets its
//! occurrence appended from the second one on (`FACTION_FLAG_AND_COLOURS@2`).
//!
//! # Paths
//!
//! A record's path is the names of the records leading to it from the root, separated by `/`
//! (`CAMPAIGN_STARTPOS/CAMPAIGN_ENV/CAMPAIGN_MODEL/WORLD/FACTION_ARRAY/FACTION`). Repeated records share the layout of
//! their path, whatever their occurrence (see [`child_path`]).
//!
//! # Lists
//!
//! Some records store variable-length lists in a single group (the family tree, the AI pathfinding and influence maps).
//! Records with more than [`MAX_LAYOUT_SLOTS`] slots are free-form: their layout has no slots and generators push
//! their children directly. Their child records still have layouts.
//!
//! # Defaults
//!
//! A leaf's default is its most common value. Arrays longer than a few dozen elements are generated data (vision bits,
//! pathfinding and AI maps), not constants, so their default is an empty array of the same type.

use serde_derive::{Deserialize, Serialize};

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::mem::{discriminant, Discriminant};
use std::ops::RangeInclusive;
use std::path::Path;

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::esf::{ESF, NodeType, RecordNode, RecordNodeFlags};

#[cfg(test)] #[path = "layout_test.rs"] mod layout_test;

/// Folder, inside RPFM's config folder, where the startpos layout files live.
pub const STARTPOS_LAYOUT_FOLDER: &str = "startpos_layout";

/// Name used for the leaves before the first record of a group.
const START_ANCHOR: &str = "^";

/// Most slots a record layout can have. Records with more are lists, kept free-form.
pub const MAX_LAYOUT_SLOTS: usize = 256;

/// Largest subsequence table (cells) used to align two leaf runs. Bigger runs are aligned in a single pass.
const MAX_ALIGNMENT_CELLS: usize = 1 << 20;

/// Longest array kept as a default value. Longer ones become empty arrays.
const MAX_DEFAULT_ARRAY_LEN: usize = 64;

/// Largest unsigned value with a compact ESF encoding (24 bits).
const MAX_COMPACT_U32: u32 = 0xFF_FFFF;

/// Signed values with a compact ESF encoding (24 bits).
const COMPACT_I32: RangeInclusive<i32> = -0x80_0000..=0x7F_FFFF;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Layouts of every record path of a startpos.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StartposLayout {

    /// Layouts, by record path.
    records: BTreeMap<String, RecordLayout>,
}

/// Layout of the records at a path.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecordLayout {

    /// Record name.
    name: String,

    /// Record version.
    version: u8,

    /// Encoding flags (`RecordNodeFlags` bits).
    flags: u8,

    /// The record's children are an array: any number of groups, each with these slots.
    array: bool,

    /// The record is a list: its groups take any children, pushed by the generators.
    #[serde(default)]
    free_form: bool,

    /// Child slots of each group, in order.
    slots: Vec<Slot>,
}

/// A child slot of a record group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Slot {

    /// A leaf value.
    Leaf {

        /// Field name (see the module docs).
        name: String,

        /// Most common value, with its encoding.
        default: NodeType,

        /// The leaf isn't in every instance.
        #[serde(default)]
        optional: bool,

        /// The optional leaf is missing in most instances, so it starts out of a built record until it's set.
        #[serde(default)]
        absent_by_default: bool,
    },

    /// A child record.
    Record {

        /// Key of the child in the path (record name plus occurrence suffix).
        key: String,

        /// The record isn't in every instance.
        optional: bool,
    },
}

/// Builds a record from its layout: every slot starts with its default, and generators replace what they need.
#[derive(Debug, Clone)]
pub struct RecordBuilder<'a> {
    layout: &'a StartposLayout,
    path: String,
    record: &'a RecordLayout,
    groups: Vec<GroupBuilder<'a>>,
}

/// A group of a record being built.
#[derive(Debug, Clone)]
pub struct GroupBuilder<'a> {
    path: String,
    record: &'a RecordLayout,
    values: Vec<Option<NodeType>>,
}

/// A child slot seen while extracting, with what's known so far about it.
#[derive(Debug, Clone)]
enum SlotStats {
    Leaf { kind: Discriminant<NodeType>, values: HashMap<String, (usize, NodeType)> },
    Record { key: String, seen: usize },
}

/// What's known so far about the records of a path.
#[derive(Debug, Clone, Default)]
struct PathStats {
    name: String,
    versions: HashMap<u8, usize>,
    flags: HashMap<u8, usize>,
    array: bool,
    free_form: bool,
    groups: usize,
    slots: Vec<SlotStats>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl StartposLayout {

    /// Extracts the layouts of every record of a startpos.
    ///
    /// # Arguments
    ///
    /// * `startpos` - A game-built startpos.
    ///
    /// # Errors
    ///
    /// Returns an error if the root isn't a record.
    pub fn from_esf(startpos: &ESF) -> Result<Self> {
        let NodeType::Record(root) = startpos.root_node() else {
            return Err(RLibError::StartposLayoutError("the startpos root is not a record".to_owned()));
        };

        let mut stats = HashMap::new();
        collect(root, root.name(), &mut stats);

        let records = stats.into_iter().map(|(path, stats)| (path, stats.into_layout())).collect();
        Ok(Self { records })
    }

    /// Loads a layout file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be read or isn't a valid layout.
    pub fn load(path: &Path) -> Result<Self> {
        let file = BufReader::new(File::open(path)?);
        ron::de::from_reader(file).map_err(|error| RLibError::StartposLayoutError(format!("invalid startpos layout: {error}")))
    }

    /// Saves the layout to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file can't be written.
    pub fn save(&self, path: &Path) -> Result<()> {
        let mut file = BufWriter::new(File::create(path)?);
        let config = ron::ser::PrettyConfig::default();
        let text = ron::ser::to_string_pretty(self, config).map_err(|error| RLibError::StartposLayoutError(format!("can't serialize the startpos layout: {error}")))?;
        file.write_all(text.as_bytes())?;
        Ok(())
    }

    /// Returns the layout of a record path.
    pub fn record(&self, path: &str) -> Option<&RecordLayout> {
        self.records.get(path)
    }

    /// Starts building a record from its layout.
    ///
    /// # Arguments
    ///
    /// * `path` - Path of the record.
    ///
    /// # Returns
    ///
    /// A builder with every slot at its default: one group for normal records, none for arrays.
    ///
    /// # Errors
    ///
    /// Returns an error if the layout has no such path.
    pub fn builder(&self, path: &str) -> Result<RecordBuilder<'_>> {
        let record = self.records.get(path).ok_or_else(|| missing(path))?;
        let mut builder = RecordBuilder { layout: self, path: path.to_owned(), record, groups: vec![] };
        if !record.array {
            builder.push_group()?;
        }
        Ok(builder)
    }

    /// Builds a record with every slot at its default.
    ///
    /// # Errors
    ///
    /// Returns an error if the layout lacks the path or one of its child records.
    pub fn default_record(&self, path: &str) -> Result<RecordNode> {
        self.builder(path)?.build()
    }
}

impl RecordLayout {

    /// Returns the record name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the record version.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns if the record's children are an array of groups.
    pub fn array(&self) -> bool {
        self.array
    }

    /// Returns if the record is a free-form list.
    pub fn free_form(&self) -> bool {
        self.free_form
    }

    /// Returns the child slots of each group.
    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    /// Matches the children of a group of a record at this path with the layout's slots.
    ///
    /// # Arguments
    ///
    /// * `group` - The children of one group of the record.
    ///
    /// # Returns
    ///
    /// The slot of each child, in order, or `None` if some child doesn't fit the layout or the record is free-form.
    pub fn match_children(&self, group: &[NodeType]) -> Option<Vec<&Slot>> {
        if self.free_form {
            return None;
        }
        let tokens = group_tokens(group);
        let left = self.slots.iter().map(Slot::kind).collect::<Vec<_>>();
        let right = tokens.iter().map(token_kind).collect::<Vec<_>>();
        let pairs = align_kinds(&left, &right);
        (pairs.len() == tokens.len()).then(|| pairs.into_iter().map(|(slot, _)| &self.slots[slot]).collect())
    }

    /// Returns the position of a leaf slot.
    fn leaf_position(&self, path: &str, name: &str) -> Result<usize> {
        self.slots.iter()
            .position(|slot| matches!(slot, Slot::Leaf { name: slot_name, .. } if slot_name == name))
            .ok_or_else(|| missing(&format!("{path}:{name}")))
    }

    /// Returns the position of a child record slot.
    fn record_position(&self, path: &str, key: &str) -> Result<usize> {
        self.slots.iter()
            .position(|slot| matches!(slot, Slot::Record { key: slot_key, .. } if slot_key == key))
            .ok_or_else(|| missing(&format!("{path}/{key}")))
    }
}

impl<'a> RecordBuilder<'a> {

    /// Adds a group with every slot at its default.
    ///
    /// # Returns
    ///
    /// The new group, to set its values.
    ///
    /// # Errors
    ///
    /// Returns an error if the layout lacks one of the group's always present child records.
    pub fn push_group(&mut self) -> Result<&mut GroupBuilder<'a>> {
        let mut values = vec![];
        for slot in &self.record.slots {
            values.push(match slot {
                Slot::Leaf { absent_by_default: true, .. } => None,
                Slot::Leaf { default, .. } => Some(default.clone()),
                Slot::Record { optional: true, .. } => None,
                Slot::Record { key, optional: false } => Some(NodeType::Record(Box::new(self.layout.default_record(&child_path(&self.path, key))?))),
            });
        }
        self.groups.push(GroupBuilder { path: self.path.to_owned(), record: self.record, values });
        Ok(self.groups.last_mut().expect("a group was just pushed"))
    }

    /// Returns the first group, where normal (non array) records keep their children.
    ///
    /// # Errors
    ///
    /// Returns an error if the record has no groups (an array without items).
    pub fn group(&mut self) -> Result<&mut GroupBuilder<'a>> {
        let path = self.path.to_owned();
        self.groups.first_mut().ok_or_else(|| missing(&format!("{path}[0]")))
    }

    /// Builds the record.
    ///
    /// # Errors
    ///
    /// This doesn't fail at the moment; it returns a `Result` so validation can be added without changing callers.
    pub fn build(self) -> Result<RecordNode> {
        let mut record = RecordNode::default();
        record.set_name(self.record.name.to_owned());
        record.set_version(self.record.version);
        record.set_record_flags(RecordNodeFlags::from_bits_truncate(self.record.flags));
        record.set_children(self.groups.into_iter().map(|group| group.values.into_iter().flatten().collect()).collect());
        Ok(record)
    }
}

impl GroupBuilder<'_> {

    /// Replaces a leaf, keeping the encoding of its default.
    ///
    /// # Arguments
    ///
    /// * `name` - Field name.
    /// * `value` - New value. It must be of the same type as the default.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or the value has another type.
    pub fn set(&mut self, name: &str, value: NodeType) -> Result<&mut Self> {
        let position = self.record.leaf_position(&self.path, name)?;
        let Some(Slot::Leaf { default, .. }) = self.record.slots.get(position) else { unreachable!("leaf_position returns leaf slots") };
        if discriminant(default) != discriminant(&value) {
            return Err(RLibError::StartposLayoutError(format!("{}:{name}: expected a value like {default:?}, got {value:?}", self.path)));
        }
        self.values[position] = Some(value);
        Ok(self)
    }

    /// Replaces an unsigned leaf, keeping its type.
    ///
    /// Games write integers and floats with their compact encoding when the value has one, so the leaf gets the
    /// `optimized` flag a decoded startpos would have.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't an unsigned integer.
    pub fn set_u32(&mut self, name: &str, value: u32) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::U32(node) => { node.set_value(value).set_optimized(value <= MAX_COMPACT_U32); },
            NodeType::U16(node) => { *node = u16::try_from(value).map_err(|_| self.wrong_type(name, &value))?; },
            NodeType::U8(node) => { *node = u8::try_from(value).map_err(|_| self.wrong_type(name, &value))?; },
            _ => return Err(self.wrong_type(name, &value)),
        }
        self.set(name, node)
    }

    /// Replaces a signed leaf, keeping its type. See [`Self::set_u32`] for its `optimized` flag.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a signed integer.
    pub fn set_i32(&mut self, name: &str, value: i32) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::I32(node) => { node.set_value(value).set_optimized(COMPACT_I32.contains(&value)); },
            NodeType::I16(node) => { *node = i16::try_from(value).map_err(|_| self.wrong_type(name, &value))?; },
            NodeType::I8(node) => { *node = i8::try_from(value).map_err(|_| self.wrong_type(name, &value))?; },
            _ => return Err(self.wrong_type(name, &value)),
        }
        self.set(name, node)
    }

    /// Replaces a float leaf. Only zero has a compact encoding: see [`Self::set_u32`] for its `optimized` flag.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a float.
    pub fn set_f32(&mut self, name: &str, value: f32) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::F32(node) => { node.set_value(value).set_optimized(value == 0.0); },
            _ => return Err(self.wrong_type(name, &value)),
        }
        self.set(name, node)
    }

    /// Replaces a boolean leaf, keeping its encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a boolean.
    pub fn set_bool(&mut self, name: &str, value: bool) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::Bool(node) => { node.set_value(value); },
            _ => return Err(self.wrong_type(name, &value)),
        }
        self.set(name, node)
    }

    /// Replaces a string leaf (ASCII or UTF-16).
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a string.
    pub fn set_str(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::Ascii(node) | NodeType::Utf16(node) => { *node = value.to_owned(); },
            _ => return Err(self.wrong_type(name, &value)),
        }
        self.set(name, node)
    }

    /// Adds a child to a free-form record.
    ///
    /// # Errors
    ///
    /// Returns an error if the record isn't free-form.
    pub fn push_child(&mut self, node: NodeType) -> Result<&mut Self> {
        if !self.record.free_form {
            return Err(RLibError::StartposLayoutError(format!("{} has a fixed layout", self.path)));
        }
        self.values.push(Some(node));
        Ok(self)
    }

    /// Replaces a string array leaf (ASCII or UTF-16).
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a string array.
    pub fn set_strings(&mut self, name: &str, values: Vec<String>) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::AsciiArray(node) | NodeType::Utf16Array(node) => { *node = values; },
            _ => return Err(self.wrong_type(name, &values)),
        }
        self.set(name, node)
    }

    /// Replaces a byte array leaf (`U8Array` or `I8Array`).
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't a byte array.
    pub fn set_bytes(&mut self, name: &str, values: Vec<u8>) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::U8Array(node) | NodeType::I8Array(node) => { *node = values; },
            _ => return Err(self.wrong_type(name, &values)),
        }
        self.set(name, node)
    }

    /// Replaces an unsigned array leaf, keeping its encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't an unsigned array.
    pub fn set_u32s(&mut self, name: &str, values: Vec<u32>) -> Result<&mut Self> {
        let mut node = self.default_of(name)?;
        match &mut node {
            NodeType::U32Array(node) => { node.set_value(values); },
            _ => return Err(self.wrong_type(name, &values)),
        }
        self.set(name, node)
    }

    /// Leaves an optional leaf out of the record.
    ///
    /// # Errors
    ///
    /// Returns an error if the field doesn't exist or isn't optional.
    pub fn unset(&mut self, name: &str) -> Result<&mut Self> {
        let position = self.record.leaf_position(&self.path, name)?;
        if !matches!(self.record.slots[position], Slot::Leaf { optional: true, .. }) {
            return Err(RLibError::StartposLayoutError(format!("{}:{name} is always present", self.path)));
        }
        self.values[position] = None;
        Ok(self)
    }

    /// Replaces a child record.
    ///
    /// # Arguments
    ///
    /// * `key` - Child key (record name plus occurrence suffix).
    /// * `record` - The new record, or `None` to leave an optional record out.
    ///
    /// # Errors
    ///
    /// Returns an error if the group has no such child record, or `None` is given for a record that's always present.
    pub fn set_record(&mut self, key: &str, record: Option<RecordNode>) -> Result<&mut Self> {
        let position = self.record.record_position(&self.path, key)?;
        if record.is_none() && matches!(self.record.slots[position], Slot::Record { optional: false, .. }) {
            return Err(RLibError::StartposLayoutError(format!("{}/{key} is always present", self.path)));
        }
        self.values[position] = record.map(|record| NodeType::Record(Box::new(record)));
        Ok(self)
    }

    /// Returns the default value of a leaf.
    fn default_of(&self, name: &str) -> Result<NodeType> {
        match &self.record.slots[self.record.leaf_position(&self.path, name)?] {
            Slot::Leaf { default, .. } => Ok(default.clone()),
            Slot::Record { .. } => unreachable!("leaf_position returns leaf slots"),
        }
    }

    /// Builds the error for a value that doesn't fit a leaf.
    fn wrong_type(&self, name: &str, value: &dyn Debug) -> RLibError {
        RLibError::StartposLayoutError(format!("{}:{name}: {value:?} doesn't fit the field", self.path))
    }
}

impl PathStats {

    /// Adds a group of a record instance, aligning it with the slots seen so far.
    fn add_group(&mut self, group: &[(Option<String>, &NodeType)]) {
        self.groups += 1;
        if self.free_form {
            return;
        }

        // Fast path: same shape as the slots seen so far.
        let same_shape = group.len() == self.slots.len() && group.iter().zip(&self.slots).all(|(token, slot)| token_kind(token) == slot.kind());
        let alignment = if same_shape {
            (0..group.len()).map(|index| (index, index)).collect()
        } else {
            align(&self.slots, group)
        };

        // Insert the tokens that didn't align, keeping the order of both sequences.
        let mut merged = Vec::with_capacity(self.slots.len().max(group.len()));
        let mut slot_index = 0;
        let mut group_index = 0;
        let mut pairs = alignment.into_iter().peekable();
        while slot_index < self.slots.len() || group_index < group.len() {
            match pairs.peek().copied() {
                Some((slot, token)) if slot == slot_index && token == group_index => {
                    let mut stats = self.slots[slot].clone();
                    stats.record(group[token].1);
                    merged.push(stats);
                    slot_index += 1;
                    group_index += 1;
                    pairs.next();
                },
                Some((slot, _)) if slot_index < slot => {
                    merged.push(self.slots[slot_index].clone());
                    slot_index += 1;
                },
                Some(_) => {
                    merged.push(SlotStats::new(&group[group_index]));
                    group_index += 1;
                },
                None if slot_index < self.slots.len() => {
                    merged.push(self.slots[slot_index].clone());
                    slot_index += 1;
                },
                None => {
                    merged.push(SlotStats::new(&group[group_index]));
                    group_index += 1;
                },
            }
        }
        self.slots = merged;
        if self.slots.len() > MAX_LAYOUT_SLOTS {
            self.free_form = true;
            self.slots.clear();
        }
    }

    /// Converts the stats into a layout.
    fn into_layout(self) -> RecordLayout {
        let mut slots = vec![];
        let mut anchor = START_ANCHOR.to_owned();
        let mut offset = 0;
        for slot in self.slots {
            match slot {
                SlotStats::Record { key, seen } => {
                    let optional = seen < self.groups;
                    if !optional {
                        anchor = key.to_owned();
                        offset = 0;
                    }
                    slots.push(Slot::Record { key, optional });
                },
                SlotStats::Leaf { values, .. } => {
                    let seen = values.values().map(|(count, _)| count).sum::<usize>();
                    let optional = seen < self.groups;
                    let absent_by_default = seen * 2 < self.groups;
                    offset += 1;

                    // The most common value is the default; ties go to the smallest text form, to stay deterministic.
                    let (_, (_, mut default)) = values.into_iter().max_by(|(text_a, (count_a, _)), (text_b, (count_b, _))| count_a.cmp(count_b).then_with(|| text_b.cmp(text_a))).expect("leaves have at least one value");
                    trim_default(&mut default);
                    slots.push(Slot::Leaf { name: format!("{anchor}+{offset}"), default, optional, absent_by_default });
                },
            }
        }

        let most_common = |counts: HashMap<u8, usize>| counts.into_iter().max_by_key(|&(value, count)| (count, Reverse(value))).map(|(value, _)| value).unwrap_or_default();
        RecordLayout {
            name: self.name,
            version: most_common(self.versions),
            flags: most_common(self.flags),
            array: self.array,
            free_form: self.free_form,
            slots,
        }
    }
}

impl SlotStats {

    /// Creates the stats of a slot from its first token.
    fn new(token: &(Option<String>, &NodeType)) -> Self {
        match token {
            (Some(key), _) => Self::Record { key: key.to_owned(), seen: 1 },
            (None, node) => {
                let mut values = HashMap::new();
                values.insert(format!("{node:?}"), (1, (*node).clone()));
                Self::Leaf { kind: discriminant(*node), values }
            },
        }
    }

    /// Records another instance of the slot.
    fn record(&mut self, node: &NodeType) {
        match self {
            Self::Record { seen, .. } => *seen += 1,
            Self::Leaf { values, .. } => values.entry(format!("{node:?}")).or_insert_with(|| (0, node.clone())).0 += 1,
        }
    }
}

/// Returns the file name of the layout of a game's campaign.
///
/// # Arguments
///
/// * `game_key` - Key of the game (`attila`).
/// * `campaign` - Campaign key (`main_attila`).
pub fn layout_file_name(game_key: &str, campaign: &str) -> String {
    format!("{game_key}_{campaign}.ron")
}

/// Returns the path of a child record from its parent's path and its key: the occurrence suffix is dropped, so every
/// occurrence shares a layout.
pub fn child_path(parent: &str, key: &str) -> String {
    let name = key.split_once('@').map(|(name, _)| name).unwrap_or(key);
    format!("{parent}/{name}")
}

/// Empties array defaults longer than [`MAX_DEFAULT_ARRAY_LEN`].
fn trim_default(node: &mut NodeType) {
    let len = match node {
        NodeType::Unknown26(values) | NodeType::I8Array(values) | NodeType::U8Array(values) => values.len(),
        NodeType::BoolArray(values) => values.len(),
        NodeType::I16Array(values) | NodeType::AngleArray(values) => values.len(),
        NodeType::I32Array(values) => values.value().len(),
        NodeType::I64Array(values) => values.len(),
        NodeType::U16Array(values) => values.len(),
        NodeType::U32Array(values) => values.value().len(),
        NodeType::U64Array(values) => values.len(),
        NodeType::F32Array(values) => values.len(),
        NodeType::F64Array(values) => values.len(),
        NodeType::Coord2dArray(values) => values.len(),
        NodeType::Coord3dArray(values) => values.len(),
        NodeType::Utf16Array(values) | NodeType::AsciiArray(values) => values.len(),
        _ => return,
    };
    if len <= MAX_DEFAULT_ARRAY_LEN {
        return;
    }

    match node {
        NodeType::Unknown26(values) | NodeType::I8Array(values) | NodeType::U8Array(values) => values.clear(),
        NodeType::BoolArray(values) => values.clear(),
        NodeType::I16Array(values) | NodeType::AngleArray(values) => values.clear(),
        NodeType::I32Array(values) => values.value_mut().clear(),
        NodeType::I64Array(values) => values.clear(),
        NodeType::U16Array(values) => values.clear(),
        NodeType::U32Array(values) => values.value_mut().clear(),
        NodeType::U64Array(values) => values.clear(),
        NodeType::F32Array(values) => values.clear(),
        NodeType::F64Array(values) => values.clear(),
        NodeType::Coord2dArray(values) => values.clear(),
        NodeType::Coord3dArray(values) => values.clear(),
        NodeType::Utf16Array(values) | NodeType::AsciiArray(values) => values.clear(),
        _ => {},
    }
}

/// Collects the stats of a record and everything under it.
fn collect(record: &RecordNode, path: &str, stats: &mut HashMap<String, PathStats>) {
    let groups = record.children().iter().map(|group| group_tokens(group)).collect::<Vec<_>>();

    let entry = stats.entry(path.to_owned()).or_insert_with(|| PathStats { name: record.name().to_owned(), ..Default::default() });
    *entry.versions.entry(*record.version()).or_default() += 1;
    *entry.flags.entry(record.record_flags().bits()).or_default() += 1;
    entry.array |= record.record_flags().contains(RecordNodeFlags::HAS_NESTED_BLOCKS);
    for tokens in &groups {
        entry.add_group(tokens);
    }

    for tokens in &groups {
        for (key, node) in tokens {
            if let (Some(key), NodeType::Record(child)) = (key, node) {
                collect(child, &child_path(path, key), stats);
            }
        }
    }
}

/// Returns the children of a group as tokens: records get their key (name plus occurrence suffix), leaves `None`.
fn group_tokens(group: &[NodeType]) -> Vec<(Option<String>, &NodeType)> {
    let mut seen = HashMap::<&str, usize>::new();
    group.iter().map(|node| match node {
        NodeType::Record(record) => {
            let count = seen.entry(record.name()).or_default();
            *count += 1;
            let key = if *count == 1 { record.name().to_owned() } else { format!("{}@{count}", record.name()) };
            (Some(key), node)
        },
        _ => (None, node),
    }).collect()
}

/// What a child or slot is, for alignment: a record by key, or a leaf by type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind<'a> {
    Record(&'a str),
    Leaf(Discriminant<NodeType>),
}

impl SlotStats {

    /// Returns the kind of the slot.
    fn kind(&self) -> Kind<'_> {
        match self {
            Self::Record { key, .. } => Kind::Record(key),
            Self::Leaf { kind, .. } => Kind::Leaf(*kind),
        }
    }
}

impl Slot {

    /// Returns the kind of the slot.
    fn kind(&self) -> Kind<'_> {
        match self {
            Self::Record { key, .. } => Kind::Record(key),
            Self::Leaf { default, .. } => Kind::Leaf(discriminant(default)),
        }
    }
}

/// Returns the kind of a group token.
fn token_kind<'a>(token: &'a (Option<String>, &NodeType)) -> Kind<'a> {
    match token {
        (Some(key), _) => Kind::Record(key),
        (None, node) => Kind::Leaf(discriminant(*node)),
    }
}

/// Aligns a group with the slots seen so far.
///
/// # Returns
///
/// The (slot index, token index) pairs that match, in order.
fn align(slots: &[SlotStats], group: &[(Option<String>, &NodeType)]) -> Vec<(usize, usize)> {
    let left = slots.iter().map(SlotStats::kind).collect::<Vec<_>>();
    let right = group.iter().map(token_kind).collect::<Vec<_>>();
    align_kinds(&left, &right)
}

/// Aligns two sequences of kinds, as matched index pairs in order.
///
/// Records are aligned first (they're few, and keyed), then the leaves between each pair of aligned records: one to
/// one if both runs have the same types, by longest common subsequence otherwise. Groups can have thousands of
/// leaves, so a single subsequence table over whole groups would be too big, and runs too long for a table (variable
/// length data like the AI influence maps) are aligned in a single pass.
fn align_kinds(left: &[Kind], right: &[Kind]) -> Vec<(usize, usize)> {
    let records = |kinds: &[Kind]| kinds.iter().enumerate().filter(|(_, kind)| matches!(kind, Kind::Record(_))).map(|(index, _)| index).collect::<Vec<_>>();
    let (left_records, right_records) = (records(left), records(right));
    let record_pairs = longest_common_subsequence(&left_records, &right_records, |&l, &r| left[l] == right[r])
        .into_iter()
        .map(|(l, r)| (left_records[l], right_records[r]))
        .collect::<Vec<_>>();

    // Leaf runs between consecutive aligned records (and before the first and after the last).
    let mut pairs = vec![];
    let mut starts = (0, 0);
    for &(l, r) in record_pairs.iter().chain(std::iter::once(&(left.len(), right.len()))) {
        let leaves = |kinds: &[Kind], start: usize, end: usize| (start..end).filter(|&index| matches!(kinds[index], Kind::Leaf(_))).collect::<Vec<_>>();
        let (left_leaves, right_leaves) = (leaves(left, starts.0, l), leaves(right, starts.1, r));
        let same_types = left_leaves.len() == right_leaves.len() && left_leaves.iter().zip(&right_leaves).all(|(&a, &b)| left[a] == right[b]);
        if same_types {
            pairs.extend(left_leaves.iter().copied().zip(right_leaves.iter().copied()));
        } else if left_leaves.len().saturating_mul(right_leaves.len()) > MAX_ALIGNMENT_CELLS {
            pairs.extend(align_in_one_pass(&left_leaves, &right_leaves, |&a, &b| left[a] == right[b]));
        } else {
            pairs.extend(longest_common_subsequence(&left_leaves, &right_leaves, |&a, &b| left[a] == right[b]).into_iter().map(|(a, b)| (left_leaves[a], right_leaves[b])));
        }
        if l < left.len() {
            pairs.push((l, r));
        }
        starts = (l + 1, r + 1);
    }
    pairs
}

/// Aligns two sequences in a single pass: matching elements are paired in order, and on a mismatch the element of the
/// longer remaining sequence is skipped.
fn align_in_one_pass<A, B>(left: &[A], right: &[B], matches: impl Fn(&A, &B) -> bool) -> Vec<(usize, usize)> {
    let mut pairs = vec![];
    let (mut l, mut r) = (0, 0);
    while l < left.len() && r < right.len() {
        if matches(&left[l], &right[r]) {
            pairs.push((l, r));
            l += 1;
            r += 1;
        } else if left.len() - l >= right.len() - r {
            l += 1;
        } else {
            r += 1;
        }
    }
    pairs
}

/// Returns the longest common subsequence of two sequences under a matching rule, as matched index pairs in order.
fn longest_common_subsequence<A, B>(left: &[A], right: &[B], matches: impl Fn(&A, &B) -> bool) -> Vec<(usize, usize)> {
    let mut lengths = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for l in (0..left.len()).rev() {
        for r in (0..right.len()).rev() {
            lengths[l][r] = if matches(&left[l], &right[r]) {
                lengths[l + 1][r + 1] + 1
            } else {
                lengths[l + 1][r].max(lengths[l][r + 1])
            };
        }
    }

    let mut pairs = vec![];
    let (mut l, mut r) = (0, 0);
    while l < left.len() && r < right.len() {
        if matches(&left[l], &right[r]) {
            pairs.push((l, r));
            l += 1;
            r += 1;
        } else if lengths[l + 1][r] >= lengths[l][r + 1] {
            l += 1;
        } else {
            r += 1;
        }
    }
    pairs
}

/// Builds the error for a path or field missing from the layout.
fn missing(path: &str) -> RLibError {
    RLibError::StartposLayoutError(format!("the startpos layout has no {path}"))
}
