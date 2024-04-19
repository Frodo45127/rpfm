//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write anim fragment battle files.
//!
//! These are the old anim tables in binary format.
//!
//! Support is complete for all games since Warhammer 2. Older games are not supported.

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{RLibError, Result};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable, table::*};
use crate::games::supported_games::{KEY_PHARAOH, KEY_THREE_KINGDOMS, KEY_TROY, KEY_WARHAMMER_2};
use crate::schema::*;
use crate::utils::check_size_mismatch;

pub const BASE_PATH: &str = "animations/";
pub const MID_PATH: &str = "/battle/";

pub const EXTENSION_NEW: &str = ".bin";
pub const EXTENSION_OLD: &str = ".frg";

mod versions;

#[cfg(test)] mod anim_fragment_battle_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AnimFragmentBattle {

    // Common stuff.
    version: u32,
    entries: Vec<Entry>,
    skeleton_name: String,

    // Wh3 stuff.
    subversion: u32,

    // Wh3/3k stuff.
    table_name: String,
    mount_table_name: String,
    unmount_table_name: String,
    locomotion_graph: String,
    is_simple_flight: bool,
    is_new_cavalry_tech: bool,

    // Wh2 stuff.
    min_id: u32,
    max_id: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Entry {

    // Common stuff.
    animation_id: u32,
    blend_in_time: f32,
    selection_weight: f32,
    weapon_bone: WeaponBone,

    // Wh3 stuff
    anim_refs: Vec<AnimRef>,

    // Wh2 stuff.
    slot_id: u32,
    filename: String,
    metadata: String,
    metadata_sound: String,
    skeleton_type: String,
    uk_3: u32,
    uk_4: String,

    // Wh2/Wh3
    single_frame_variant: bool,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct AnimRef {
    file_path: String,
    meta_file_path: String,
    snd_file_path: String,
}

bitflags! {

    /// This represents the bitmasks of weapon_bone values.
    #[derive(PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct WeaponBone: u32 {
        const WEAPON_BONE_1 = 0b0000_0000_0000_0001;
        const WEAPON_BONE_2 = 0b0000_0000_0000_0010;
        const WEAPON_BONE_3 = 0b0000_0000_0000_0100;
        const WEAPON_BONE_4 = 0b0000_0000_0000_1000;
        const WEAPON_BONE_5 = 0b0000_0000_0001_0000;
        const WEAPON_BONE_6 = 0b0000_0000_0010_0000;
    }
}

//---------------------------------------------------------------------------//
//                      Implementation of AnimFragment
//---------------------------------------------------------------------------//

impl AnimFragmentBattle {

    pub fn definitions() -> (Definition, Definition) {
        let mut anim_refs_definition = Definition::default();
        anim_refs_definition.fields_mut().push(Field::new("file_path".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        anim_refs_definition.fields_mut().push(Field::new("meta_file_path".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        anim_refs_definition.fields_mut().push(Field::new("snd_file_path".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));

        let mut definition = Definition::default();
        definition.fields_mut().push(Field::new("animation_id".to_string(), FieldType::I32, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("blend_in_time".to_string(), FieldType::F32, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("selection_weight".to_string(), FieldType::F32, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("weapon_bone".to_string(), FieldType::I32, false, None, false, None, None, None, String::new(), -1, 6, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("anim_refs".to_string(), FieldType::SequenceU32(Box::new(anim_refs_definition.clone())), false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("slot_id".to_string(), FieldType::I32, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("filename".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("metadata".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("metadata_sound".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("skeleton_type".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("uk_3".to_string(), FieldType::I32, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("uk_4".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("single_frame_variant".to_string(), FieldType::Boolean, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));

        (definition, anim_refs_definition)
    }

    pub fn from_table(table: &Table) -> Result<Vec<Entry>> {
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
                    let data = Table::decode(&mut data, definition, &HashMap::new(), None, false, fields_processed[9].name())?;
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

    pub fn to_table(&self) -> Result<Table> {
        let (definition, anim_refs_definition) = Self::definitions();
        let mut table = Table::new(&definition, None, "");

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

            let mut anim_refs_subtable = Table::new(&anim_refs_definition, None, "anim_refs");
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
            let _ = anim_refs_subtable.encode(&mut writer, &None);

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
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        let version = data.read_u32()?;

        let mut fragment = Self::default();
        fragment.version = version;

        match version {
            2 => match game_key {
                KEY_WARHAMMER_2 | KEY_TROY | KEY_PHARAOH => fragment.read_v2_wh2(data)?,
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
        let game_key = extra_data.game_key.ok_or_else(|| RLibError::DecodingMissingExtraDataField("game_key".to_owned()))?;

        buffer.write_u32(self.version)?;

        match self.version {
            2 => match game_key {
                KEY_WARHAMMER_2 | KEY_TROY | KEY_PHARAOH => self.write_v2_wh2(buffer)?,
                KEY_THREE_KINGDOMS => self.write_v2_3k(buffer)?,
                _ => Err(RLibError::DecodingMatchedCombatUnsupportedVersion(self.version as usize))?,
            },
            4 => self.write_v4(buffer)?,
            _ => Err(RLibError::DecodingAnimFragmentUnsupportedVersion(self.version as usize))?,
        };

        Ok(())
    }
}

