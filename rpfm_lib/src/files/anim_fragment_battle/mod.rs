//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Animation fragment battle file format support.
//!
//! This module handles animation fragment files (`.bin`/`.frg`) which define battle animations
//! for units in Total War games. These files replaced the older text-based animation tables
//! with a more efficient binary format.
//!
//! # File Format
//!
//! Animation fragments use game-specific binary formats with different versions:
//! - **Version 2 (Warhammer 2)**: Basic animation metadata with file references
//! - **Version 2 (Three Kingdoms)**: Enhanced with mount/unmount tables and locomotion graphs
//! - **Version 4 (Warhammer 3)**: Advanced format with animation references and cavalry tech flags
//!
//! # File Extensions
//!
//! - `.bin` - Modern binary animation fragment files (Warhammer 2+)
//! - `.frg` - Legacy fragment file extension (older games)
//!
//! # File Organization
//!
//! Animation fragments are stored in:
//! ```text
//! animations/{skeleton_name}/battle/{animation_type}.bin
//! ```
//!
//! # Supported Games
//!
//! Full support for:
//! - Total War: Warhammer II (version 2)
//! - Total War: Three Kingdoms (version 2, enhanced)
//! - A Total War Saga: Troy (version 2)
//! - Total War: Warhammer III (version 4)
//! - Total War: Pharaoh / Pharaoh Dynasties (version 2)
//!
//! Older games (pre-Warhammer 2) are not supported.
//!
//! # Animation Entry Structure
//!
//! Each entry contains:
//! - Animation ID and metadata (blend time, selection weight)
//! - Weapon bone flags (for weapon attachment points)
//! - File references to animation data and metadata
//! - Optional single-frame variant flag
//!
//! # Table Conversion
//!
//! Animation fragments can be converted to/from [`TableInMemory`]
//! for editing as TSV files.
//!
//! [`TableInMemory`]: crate::files::table::local::TableInMemory
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::anim_fragment_battle::AnimFragmentBattle;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode an animation fragment
//! let fragment = AnimFragmentBattle::decode(&mut data, &Some(extra_data))?;
//!
//! // Access entries
//! for entry in fragment.entries() {
//!     println!("Animation {}: blend={}, weight={}",
//!         entry.animation_id(),
//!         entry.blend_in_time(),
//!         entry.selection_weight()
//!     );
//! }
//!
//! // Convert to table for TSV export
//! let table = fragment.to_table()?;
//! ```

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::Cursor;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::{DecodedData, local::TableInMemory, Table}};
use crate::games::supported_games::{KEY_PHARAOH, KEY_PHARAOH_DYNASTIES, KEY_THREE_KINGDOMS, KEY_TROY, KEY_WARHAMMER_2};
use crate::schema::*;
use crate::utils::check_size_mismatch;

/// Base directory path for animation files.
pub const BASE_PATH: &str = "animations/";

/// Middle path component for battle animations.
pub const MID_PATH: &str = "/battle/";

/// Modern file extension for animation fragment files.
pub const EXTENSION_NEW: &str = ".bin";

/// Legacy file extension for animation fragment files.
pub const EXTENSION_OLD: &str = ".frg";

mod versions;

#[cfg(test)] mod anim_fragment_battle_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a battle animation fragment file.
///
/// Contains animation entries and metadata for a specific skeleton type.
/// The structure varies by game version, with newer games supporting more features.
///
/// # Version Differences
///
/// - **Version 2 (Warhammer 2/Troy/Pharaoh)**: Basic structure with min/max IDs
/// - **Version 2 (Three Kingdoms)**: Adds mount tables and locomotion graph support
/// - **Version 4 (Warhammer 3)**: Enhanced with subversion and cavalry tech flags
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AnimFragmentBattle {
    // Common fields across all versions

    /// File format version (2 or 4).
    version: u32,

    /// List of animation entries in this fragment.
    entries: Vec<Entry>,

    /// Name of the skeleton this animation fragment applies to.
    skeleton_name: String,

    // Warhammer 3 specific (version 4)

    /// Format subversion (version 4 only).
    subversion: u32,

    // Warhammer 3 / Three Kingdoms specific

    /// Name of the animation table.
    table_name: String,

    /// Name of the mount animation table.
    mount_table_name: String,

    /// Name of the unmount animation table.
    unmount_table_name: String,

    /// Locomotion graph identifier.
    locomotion_graph: String,

    /// Whether this uses simple flight mechanics.
    is_simple_flight: bool,

    /// Whether this uses new cavalry technology.
    is_new_cavalry_tech: bool,

    // Warhammer 2 specific (version 2)

    /// Minimum animation ID in this fragment (version 2 only).
    min_id: u32,

    /// Maximum animation ID in this fragment (version 2 only).
    max_id: u32,
}

/// Represents a single animation entry within a fragment.
///
/// Contains all metadata and file references for one animation. The structure
/// varies between version 2 and version 4 formats.
///
/// # Version Differences
///
/// - **Version 2**: Uses direct filename and metadata strings
/// - **Version 4**: Uses `anim_refs` for multiple animation file references
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Entry {
    // Common fields across all versions

    /// Unique animation identifier.
    animation_id: u32,

    /// Blend-in time in seconds for smooth transitions.
    blend_in_time: f32,

    /// Selection weight for animation variation selection (higher = more likely).
    selection_weight: f32,

    /// Weapon attachment bone flags.
    weapon_bone: WeaponBone,

    // Warhammer 3 specific (version 4)

    /// Animation file references (version 4 only).
    ///
    /// Contains paths to animation data, metadata, and sound files.
    anim_refs: Vec<AnimRef>,

    // Warhammer 2 specific (version 2)

    /// Slot identifier (version 2 only).
    slot_id: u32,

    /// Animation filename (version 2 only).
    filename: String,

    /// Metadata file path (version 2 only).
    metadata: String,

    /// Sound metadata file path (version 2 only).
    metadata_sound: String,

    /// Skeleton type identifier (version 2 only).
    skeleton_type: String,

    /// Unknown field (purpose not identified, version 2 only).
    uk_3: u32,

    /// Unknown field (purpose not identified, version 2 only).
    uk_4: String,

    // Common to version 2 and 4

    /// Whether this is a single-frame animation variant.
    single_frame_variant: bool,
}

/// Animation file reference (version 4 only).
///
/// Contains paths to the three files that make up an animation:
/// - Animation data file (skeletal animation)
/// - Metadata file (timing, events, etc.)
/// - Sound file (audio cues and effects)
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AnimRef {
    /// Path to the animation data file.
    file_path: String,

    /// Path to the animation metadata file.
    meta_file_path: String,

    /// Path to the sound file.
    snd_file_path: String,
}

bitflags! {
    /// Weapon attachment bone flags.
    ///
    /// Defines which bones (attachment points) on the skeleton are used for
    /// weapon positioning during this animation. Multiple bones can be active
    /// simultaneously (e.g., for dual-wielding).
    ///
    /// # Bone Mapping
    ///
    /// Each bit corresponds to a specific weapon attachment point:
    /// - `WEAPON_BONE_1`: Primary weapon hand (typically right hand)
    /// - `WEAPON_BONE_2`: Secondary weapon hand (typically left hand)
    /// - `WEAPON_BONE_3`: Back-mounted weapon (holstered)
    /// - `WEAPON_BONE_4`: Additional attachment point
    /// - `WEAPON_BONE_5`: Additional attachment point
    /// - `WEAPON_BONE_6`: Additional attachment point
    ///
    /// The exact bone mapping depends on the skeleton definition.
    #[derive(PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct WeaponBone: u32 {
        /// Primary weapon bone (bit 0).
        const WEAPON_BONE_1 = 0b0000_0000_0000_0001;
        
        /// Secondary weapon bone (bit 1).
        const WEAPON_BONE_2 = 0b0000_0000_0000_0010;
        
        /// Tertiary weapon bone (bit 2).
        const WEAPON_BONE_3 = 0b0000_0000_0000_0100;
        
        /// Fourth weapon bone (bit 3).
        const WEAPON_BONE_4 = 0b0000_0000_0000_1000;
        
        /// Fifth weapon bone (bit 4).
        const WEAPON_BONE_5 = 0b0000_0000_0001_0000;
        
        /// Sixth weapon bone (bit 5).
        const WEAPON_BONE_6 = 0b0000_0000_0010_0000;
    }
}

//---------------------------------------------------------------------------//
//                      Implementation of AnimFragment
//---------------------------------------------------------------------------//

impl AnimFragmentBattle {

    /// Returns table schema definitions for animation fragments.
    ///
    /// Provides two [`Definition`]s:
    /// 1. Main entry definition with all animation entry fields
    /// 2. Animation reference sub-definition (for version 4 `anim_refs` field)
    ///
    /// These definitions are used when converting animation fragments to/from
    /// [`TableInMemory`] for TSV export/import.
    ///
    /// [`TableInMemory`]: crate::files::table::local::TableInMemory
    ///
    /// # Returns
    ///
    /// A tuple of `(entry_definition, anim_ref_definition)`.
    pub fn definitions() -> (Definition, Definition) {
        let mut anim_refs_definition = Definition::default();
        anim_refs_definition.fields_mut().push(Field { name: "file_path".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        anim_refs_definition.fields_mut().push(Field { name: "meta_file_path".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        anim_refs_definition.fields_mut().push(Field { name: "snd_file_path".to_string(), field_type: FieldType::StringU8, ..Default::default() });

        let mut definition = Definition::default();
        definition.fields_mut().push(Field { name: "animation_id".to_string(), field_type: FieldType::I32, is_key: true, ..Default::default() });
        definition.fields_mut().push(Field { name: "blend_in_time".to_string(), field_type: FieldType::F32, ..Default::default() });
        definition.fields_mut().push(Field { name: "selection_weight".to_string(), field_type: FieldType::F32, ..Default::default() });
        definition.fields_mut().push(Field { name: "weapon_bone".to_string(), field_type: FieldType::I32, is_bitwise: 6, ..Default::default() });
        definition.fields_mut().push(Field { name: "anim_refs".to_string(), field_type: FieldType::SequenceU32(Box::new(anim_refs_definition.clone())), ..Default::default() });
        definition.fields_mut().push(Field { name: "slot_id".to_string(), field_type: FieldType::I32, is_key: true, ..Default::default() });
        definition.fields_mut().push(Field { name: "filename".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "metadata".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "metadata_sound".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "skeleton_type".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "uk_3".to_string(), field_type: FieldType::I32, ..Default::default() });
        definition.fields_mut().push(Field { name: "uk_4".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "single_frame_variant".to_string(), field_type: FieldType::Boolean, ..Default::default() });

        (definition, anim_refs_definition)
    }

    /// Converts a table to a list of animation entries.
    ///
    /// Parses a [`TableInMemory`] (typically loaded from TSV)
    /// and extracts animation entry data.
    ///
    /// [`TableInMemory`]: crate::files::table::local::TableInMemory
    ///
    /// # Parameters
    ///
    /// - `table`: The table containing animation entry data
    ///
    /// # Returns
    ///
    /// A vector of [`Entry`] structs parsed from the table rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the table structure doesn't match the expected schema
    /// or if data conversion fails.
    pub fn from_table(table: &TableInMemory) -> Result<Vec<Entry>> {
        let mut entries = vec![];

        let definition = table.definition();
        let fields_processed = definition.fields_processed();

        for row in table.data().iter() {
            let mut entry = Entry::default();

            if let DecodedData::I32(data) = row[0] {
                entry.set_animation_id(data as u32);
            }

            if let DecodedData::F32(data) = row[1] {
                entry.set_blend_in_time(data);
            }

            if let DecodedData::F32(data) = row[2] {
                entry.set_selection_weight(data);
            }

            if let DecodedData::Boolean(data_1) = row[3] {
                if let DecodedData::Boolean(data_2) = row[4] {
                    if let DecodedData::Boolean(data_3) = row[5] {
                        if let DecodedData::Boolean(data_4) = row[6] {
                            if let DecodedData::Boolean(data_5) = row[7] {
                                if let DecodedData::Boolean(data_6) = row[8] {
                                    let mut bits = WeaponBone::empty();

                                    if data_1 { bits |= WeaponBone::WEAPON_BONE_1; }
                                    if data_2 { bits |= WeaponBone::WEAPON_BONE_2; }
                                    if data_3 { bits |= WeaponBone::WEAPON_BONE_3; }
                                    if data_4 { bits |= WeaponBone::WEAPON_BONE_4; }
                                    if data_5 { bits |= WeaponBone::WEAPON_BONE_5; }
                                    if data_6 { bits |= WeaponBone::WEAPON_BONE_6; }

                                    entry.set_weapon_bone(bits);
                                }
                            }
                        }
                    }
                }
            }

            if let DecodedData::SequenceU32(ref data) = row[9] {
                if let FieldType::SequenceU32(ref definition) = fields_processed[9].field_type() {
                    let mut data = Cursor::new(data);
                    let data = TableInMemory::decode(&mut data, definition, &HashMap::new(), None, false, fields_processed[9].name())?;
                    let mut entries = vec![];

                    for row in data.data().iter() {
                        let mut entry = AnimRef::default();

                        if let DecodedData::StringU8(ref data) = row[0] {
                            entry.set_file_path(data.to_string());
                        }

                        if let DecodedData::StringU8(ref data) = row[1] {
                            entry.set_meta_file_path(data.to_string());
                        }

                        if let DecodedData::StringU8(ref data) = row[2] {
                            entry.set_snd_file_path(data.to_string());
                        }

                        entries.push(entry);
                    }

                    entry.set_anim_refs(entries);
                }
            }

            if let DecodedData::I32(data) = row[10] {
                entry.set_slot_id(data as u32);
            }

            if let DecodedData::StringU8(ref data) = row[11] {
                entry.set_filename(data.to_string());
            }

            if let DecodedData::StringU8(ref data) = row[12] {
                entry.set_metadata(data.to_string());
            }

            if let DecodedData::StringU8(ref data) = row[13] {
                entry.set_metadata_sound(data.to_string());
            }

            if let DecodedData::StringU8(ref data) = row[14] {
                entry.set_skeleton_type(data.to_string());
            }

            if let DecodedData::I32(data) = row[15] {
                entry.set_uk_3(data as u32);
            }

            if let DecodedData::StringU8(ref data) = row[16] {
                entry.set_uk_4(data.to_string());
            }

            if let DecodedData::Boolean(data) = row[17] {
                entry.set_single_frame_variant(data);
            }

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Converts this animation fragment to a table.
    ///
    /// Creates a [`TableInMemory`] from the animation entries, which can then be
    /// exported as TSV for editing.
    ///
    /// [`TableInMemory`]: crate::files::table::local::TableInMemory
    ///
    /// # Returns
    ///
    /// A table containing all animation entries with the appropriate schema.
    ///
    /// # Errors
    ///
    /// Returns an error if table construction or data conversion fails.
    pub fn to_table(&self) -> Result<TableInMemory> {
        let (definition, anim_refs_definition) = Self::definitions();
        let mut table = TableInMemory::new(&definition, None, "");

        let data = self.entries()
            .iter()
            .map(|entry| {
            let mut row = Vec::with_capacity(19);
            row.push(DecodedData::I32(*entry.animation_id() as i32));
            row.push(DecodedData::F32(*entry.blend_in_time()));
            row.push(DecodedData::F32(*entry.selection_weight()));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_1)));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_2)));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_3)));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_4)));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_5)));
            row.push(DecodedData::Boolean(entry.weapon_bone().contains(WeaponBone::WEAPON_BONE_6)));

            let mut anim_refs_subtable = TableInMemory::new(&anim_refs_definition, None, "anim_refs");
            let mut anim_ref_rows = Vec::with_capacity(entry.anim_refs().len());
            for anim_ref in entry.anim_refs() {
                let anim_ref_row = vec![
                    DecodedData::StringU8(anim_ref.file_path().to_string()),
                    DecodedData::StringU8(anim_ref.meta_file_path().to_string()),
                    DecodedData::StringU8(anim_ref.snd_file_path().to_string()),
                ];
                anim_ref_rows.push(anim_ref_row)
            }
            anim_refs_subtable.set_data(&anim_ref_rows).unwrap();
            let mut writer = vec![];
            writer.write_u32(anim_ref_rows.len() as u32).unwrap();
            let _ = anim_refs_subtable.encode(&mut writer);

            row.push(DecodedData::SequenceU32(writer));

            row.push(DecodedData::I32(*entry.slot_id() as i32));
            row.push(DecodedData::StringU8(entry.filename().to_string()));
            row.push(DecodedData::StringU8(entry.metadata().to_string()));
            row.push(DecodedData::StringU8(entry.metadata_sound().to_string()));
            row.push(DecodedData::StringU8(entry.skeleton_type().to_string()));
            row.push(DecodedData::I32(*entry.uk_3() as i32));
            row.push(DecodedData::StringU8(entry.uk_4().to_string()));
            row.push(DecodedData::Boolean(*entry.single_frame_variant()));

            row
        }).collect::<Vec<_>>();

        table.set_data(&data)?;
        Ok(table)
    }
}

impl Decodeable for AnimFragmentBattle {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        let version = data.read_u32()?;

        let mut fragment = Self::default();
        fragment.version = version;

        match version {
            2 => match game_info.key() {
                KEY_WARHAMMER_2 | KEY_TROY | KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => fragment.read_v2_wh2(data)?,
                KEY_THREE_KINGDOMS => fragment.read_v2_3k(data)?,
                _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(fragment.version as usize))?,
            },
            4 => fragment.read_v4(data)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(version as usize))?,
        }

        // If we are not in the last byte, it means we didn't parse the entire file, which means this file is corrupt.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(fragment)
    }
}

impl Encodeable for AnimFragmentBattle {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let extra_data = extra_data.as_ref().ok_or(RLibError::DecodingMissingExtraData)?;
        let game_info = extra_data.game_info.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_info".to_owned()))?;

        buffer.write_u32(self.version)?;

        match self.version {
            2 => match game_info.key() {
                KEY_WARHAMMER_2 | KEY_TROY | KEY_PHARAOH | KEY_PHARAOH_DYNASTIES => self.write_v2_wh2(buffer)?,
                KEY_THREE_KINGDOMS => self.write_v2_3k(buffer)?,
                _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(self.version as usize))?,
            },
            4 => self.write_v4(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}
