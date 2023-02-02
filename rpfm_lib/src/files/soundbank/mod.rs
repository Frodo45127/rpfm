//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!
//!
//!
//! For more info about all this stuff, check https://github.com/bnnm/wwiser/.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::collections::HashMap;
use std::io::Write;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

use super::DecodeableExtraData;

/// Extension used by soundbank files.
pub const EXTENSION: &str = ".bnk";

/// Hash Types.
const FNV_NO: &str = "none";// #special value, no hashname allowed
const FNV_BNK: &str = "bank";
const FNV_LNG: &str = "language";
const FNV_EVT: &str = "event";
const FNV_BUS: &str = "bus";
const FNV_SFX: &str = "sfx";
const FNV_TRG: &str = "trigger";
const FNV_GME: &str = "rtpc/game-variable";
const FNV_VAR: &str = "variable";// #switches/states names
const FNV_VAL: &str = "value";// #switches/states values
const FNV_UNK: &str = "???";

#[cfg(test)] mod test_soundbank;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire `SoundBank` file decoded in memory.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct SoundBank {
    sections: Vec<Section>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Section {
    BKHD(BKHD),
    HIRC(HIRC),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct BKHD {
    version: u32,
    id: u32,
    language: Language,
    feedback_in_bank: u32,
    project_id: u32,
    padding: Vec<u8>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct HIRC {
    objects: Vec<Object>,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
enum ObjectType {
    Settings = 1,
    SoundEffectOrVoice = 2,
    EventAction = 3,
    Event = 4,
    RandomOrSequenceContainer = 5,
    SwitchContainer = 6,
    ActorMixer = 7,
    AudioBus = 8,
    BlendContainer = 9,
    MusicSegment = 10,
    MusicTrack = 11,
    MusicSwitchContainer = 12,
    MusicPlaylistContainer = 13,
    Attenuation = 14,
    DialogueEvent = 15,
    MotionBus = 16,
    MotionFx = 17,
    Effect = 18,
    Unknown = 19,
    AuxiliaryBus = 20
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Object {
    object_type: ObjectType,
    size: u32,
    id: u32,
    data: ObjectData,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
enum ObjectData {
    Settings(ObjectDataSettings),
    Event(ObjectDataEvent),
    Raw(Vec<u8>),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct ObjectDataSettings {
    settings: HashMap<u8, f32>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct ObjectDataEvent {
    values: Vec<u32>,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
enum Language {
    Sfx = 0x00,
    Arabic = 0x01,
    Bulgarian = 0x02,
    Chinese1 = 0x03, //(HK),
    Chinese2 = 0x04, //(PRC),
    Chinese3 = 0x05, //(Taiwan),
    Czech = 0x06,
    Danish = 0x07,
    Dutch = 0x08,
    English1 = 0x09, //(Australia),
    English2 = 0x0A, //(India),
    English3 = 0x0B, //(UK),
    English4 = 0x0C, //(US),
    Finnish = 0x0D,
    French1 = 0x0E, //(Canada),
    French2 = 0x0F, //(France),
    German = 0x10,
    Greek = 0x11,
    Hebrew = 0x12,
    Hungarian = 0x13,
    Indonesian = 0x14,
    Italian = 0x15,
    Japanese = 0x16,
    Korean = 0x17,
    Latin = 0x18,
    Norwegian = 0x19,
    Polish = 0x1A,
    Portuguese1 = 0x1B, //(Brazil),
    Portuguese2 = 0x1C, //(Portugal),
    Romanian = 0x1D,
    Russian = 0x1E,
    Slovenian = 0x1F,
    Spanish1 = 0x20, //(Mexico),
    Spanish2 = 0x21, //(Spain),
    Spanish3 = 0x22, //(US),
    Swedish = 0x23,
    Turkish = 0x24,
    Ukrainian = 0x25,
    Vietnamese = 0x26,
}

//---------------------------------------------------------------------------//
//                        Implementation of SoundBank
//---------------------------------------------------------------------------//

impl TryFrom<u8> for ObjectType {
    type Error = RLibError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Settings,
            2 => Self::SoundEffectOrVoice,
            3 => Self::EventAction,
            4 => Self::Event,
            5 => Self::RandomOrSequenceContainer,
            6 => Self::SwitchContainer,
            7 => Self::ActorMixer,
            8 => Self::AudioBus,
            9 => Self::BlendContainer,
            10 => Self::MusicSegment,
            11 => Self::MusicTrack,
            12 => Self::MusicSwitchContainer,
            13 => Self::MusicPlaylistContainer,
            14 => Self::Attenuation,
            15 => Self::DialogueEvent,
            16 => Self::MotionBus,
            17 => Self::MotionFx,
            18 => Self::Effect,
            19 => Self::Unknown,
            20 => Self::AuxiliaryBus,
            _ => unimplemented!("{}", value),
        })
    }
}

impl TryFrom<u32> for Language {
    type Error = RLibError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Sfx,
            0x01 => Self::Arabic,
            0x02 => Self::Bulgarian,
            0x03 => Self::Chinese1,
            0x04 => Self::Chinese2,
            0x05 => Self::Chinese3,
            0x06 => Self::Czech,
            0x07 => Self::Danish,
            0x08 => Self::Dutch,
            0x09 => Self::English1,
            0x0A => Self::English2,
            0x0B => Self::English3,
            0x0C => Self::English4,
            0x0D => Self::Finnish,
            0x0E => Self::French1,
            0x0F => Self::French2,
            0x10 => Self::German,
            0x11 => Self::Greek,
            0x12 => Self::Hebrew,
            0x13 => Self::Hungarian,
            0x14 => Self::Indonesian,
            0x15 => Self::Italian,
            0x16 => Self::Japanese,
            0x17 => Self::Korean,
            0x18 => Self::Latin,
            0x19 => Self::Norwegian,
            0x1A => Self::Polish,
            0x1B => Self::Portuguese1,
            0x1C => Self::Portuguese2,
            0x1D => Self::Romanian,
            0x1E => Self::Russian,
            0x1F => Self::Slovenian,
            0x20 => Self::Spanish1,
            0x21 => Self::Spanish2,
            0x22 => Self::Spanish3,
            0x23 => Self::Swedish,
            0x24 => Self::Turkish,
            0x25 => Self::Ukrainian,
            0x26 => Self::Vietnamese,
            _ => unimplemented!("{}", value),
        })
    }
}

impl Decodeable for SoundBank {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        let data_len = data.len()?;

        while let Ok(section_signature) = data.read_string_u8(4) {
            let section_size = data.read_u32()? as u64;
            let section_start = data.stream_position()?;
            let section_end = section_start + section_size;

            dbg!(&section_signature);
            dbg!(&section_size);
            match &*section_signature {
                "BKHD" => {

                    // CA seems to put an extra byte here for some reason. Remove it to get the proper version.
                    let version = data.read_u32()? ^ 0x80_00_00_00;dbg!(version);
                    let id = data.read_u32()?;dbg!(id);
                    let language = Language::try_from(data.read_u32()?)?;dbg!(&language);
                    let feedback_in_bank = data.read_u32()?;dbg!(feedback_in_bank);
                    let project_id = data.read_u32()?;dbg!(project_id);
                    let curr_pos = data.stream_position()?;
                    let padding = data.read_slice((section_end - curr_pos) as usize, false)?;

                    decoded.sections.push(Section::BKHD(BKHD {
                        version,
                        id,
                        language,
                        feedback_in_bank,
                        project_id,
                        padding,
                    }));
                }

                "HIRC" => {
                    let count = data.read_u32()?;dbg!(count);
                    let mut objects = vec![];
                    for _ in 0..count {
                        //dbg!(i);
                        //dbg!(data.stream_position()?);
                        let object_type = ObjectType::try_from(data.read_u8()?)?;
                        let size = data.read_u32()?;

                        let object_start = data.stream_position()?;
                        let object_end = object_start + size as u64;

                        let id = data.read_u32()?;

                                dbg!(object_type);
                                dbg!(id);
                        let data = match object_type {
                            ObjectType::Settings => {
                                dbg!(data.stream_position()?);
                                let element_count = data.read_u8()?;dbg!(element_count);
                                let mut settings = HashMap::new();
                                for _ in 0..element_count {
                                    let index = data.read_u8()?;
                                    let data = data.read_f32()?;
                                    settings.insert(index, data);
                                }
                                ObjectData::Settings(ObjectDataSettings {
                                    settings
                                })
                            },
                            /*
                            ObjectType::SoundEffectOrVoice => todo!(),
                            */
                            ObjectType::EventAction => {
                                let curr_pos = data.stream_position()?;
                                let data = data.read_slice((object_end - curr_pos) as usize, false)?;
                                let mut data_2 = std::io::Cursor::new(data.to_vec());
                                dbg!(data_2.read_u8()?);
                                dbg!(data_2.read_u8()?);
                                dbg!(data_2.read_u32()?);
                                ObjectData::Raw(data)
                            },
                            ObjectType::Event => {

                                dbg!(data.stream_position()?);
                                let element_count = data.read_u32()?;dbg!(element_count);
                                let mut values = vec![];
                                for _ in 0..element_count {
                                    let data = data.read_u32()?;
                                    values.push(data);
                                }
                                dbg!(&values);
                                ObjectData::Event(ObjectDataEvent {
                                    values
                                })
                            },
                            /*ObjectType::RandomOrSequenceContainer => todo!(),
                            ObjectType::SwitchContainer => todo!(),
                            ObjectType::ActorMixer => todo!(),
                            ObjectType::AudioBus => todo!(),
                            ObjectType::BlendContainer => todo!(),
                            ObjectType::MusicSegment => todo!(),
                            ObjectType::MusicTrack => todo!(),
                            ObjectType::MusicSwitchContainer => todo!(),
                            ObjectType::MusicPlaylistContainer => todo!(),
                            ObjectType::Attenuation => todo!(),
                            ObjectType::DialogueEvent => todo!(),
                            ObjectType::MotionBus => todo!(),
                            ObjectType::MotionFx => todo!(),
                            ObjectType::Effect => todo!(),
                            ObjectType::Unknown => todo!(),
                            ObjectType::AuxiliaryBus => todo!(),*/
                            _ => {
                                let curr_pos = data.stream_position()?;
                                ObjectData::Raw(data.read_slice((object_end - curr_pos) as usize, false)?)
                            },
                        };

                        let object = Object {
                            object_type,
                            size,
                            id,
                            data
                        };

                        objects.push(object);
                    }

                    decoded.sections.push(Section::HIRC(HIRC {
                        objects
                    }));
                },
                _ => todo!(),
            }
        }

        check_size_mismatch(data.stream_position()? as usize, data_len as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for SoundBank {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        for section in self.sections() {
            match section {
                Section::BKHD(data) => {
                    let mut encoded_data = vec![];
                    encoded_data.write_u32(data.version | 0x80_00_00_00)?;
                    encoded_data.write_u32(data.id)?;
                    encoded_data.write_u32(data.language as u32)?;
                    encoded_data.write_u32(data.feedback_in_bank)?;
                    encoded_data.write_u32(data.project_id)?;
                    encoded_data.write_all(&data.padding)?;

                    buffer.write_string_u8("BKHD")?;
                    buffer.write_u32(encoded_data.len() as u32)?;
                    buffer.write_all(&encoded_data)?;
                }
                Section::HIRC(data) => {
                    let mut encoded_data = vec![];
                    encoded_data.write_u32(data.objects.len() as u32)?;
                    for object in &data.objects {
                        encoded_data.write_u8(object.object_type as u8)?;

                        match &object.data {
                            ObjectData::Settings(data) => {
                                let mut encoded_object_data = vec![];
                                encoded_object_data.write_u32(object.id)?;
                                encoded_object_data.write_u8(data.settings.len() as u8)?;

                                for (index, value) in &data.settings {
                                    encoded_object_data.write_u8(*index)?;
                                    encoded_object_data.write_f32(*value)?;
                                }

                                encoded_data.write_u32(encoded_object_data.len() as u32)?;
                                encoded_data.write_all(&encoded_object_data)?;
                            },
                            ObjectData::Event(data) => {
                                let mut encoded_object_data = vec![];
                                encoded_object_data.write_u32(object.id)?;
                                encoded_object_data.write_u32(data.values.len() as u32)?;

                                for value in &data.values {
                                    encoded_object_data.write_u32(*value)?;
                                }

                                encoded_data.write_u32(encoded_object_data.len() as u32)?;
                                encoded_data.write_all(&encoded_object_data)?;
                            },
                            ObjectData::Raw(data) => {
                                encoded_data.write_u32(data.len() as u32 + 4)?;
                                encoded_data.write_u32(object.id)?;
                                encoded_data.write_all(data)?;
                            }
                        }
                    }

                    buffer.write_string_u8("HIRC")?;
                    buffer.write_u32(encoded_data.len() as u32)?;
                    buffer.write_all(&encoded_data)?;
                }
            }
        }

        Ok(())
    }
}
