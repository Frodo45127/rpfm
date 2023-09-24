//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::utils::check_size_mismatch;

use self::actor_mixer::ActorMixer;
use self::attenuation::Attenuation;
use self::audio_bus::AudioBus;
use self::auxiliary_bus::AuxiliaryBus;
use self::blend_container::BlendContainer;
use self::dialogue_event::DialogueEvent;
use self::effect::Effect;
use self::event::Event;
use self::event_action::EventAction;
use self::motion_bus::MotionBus;
use self::motion_fx::MotionFx;
use self::music_playlist_container::MusicPlaylistContainer;
use self::music_segment::MusicSegment;
use self::music_switch_container::MusicSwitchContainer;
use self::music_track::MusicTrack;
use self::random_or_sequence_container::RandomOrSequenceContainer;
use self::settings::Settings;
use self::sound_effect_or_voice::SoundEffectOrVoice;
use self::switch_container::SwitchContainer;
use self::unknown::Unknown;

pub mod actor_mixer;
pub mod attenuation;
pub mod audio_bus;
pub mod auxiliary_bus;
pub mod blend_container;
pub mod dialogue_event;
pub mod effect;
pub mod event;
pub mod event_action;
pub mod motion_bus;
pub mod motion_fx;
pub mod music_playlist_container;
pub mod music_segment;
pub mod music_switch_container;
pub mod music_track;
pub mod random_or_sequence_container;
pub mod settings;
pub mod sound_effect_or_voice;
pub mod switch_container;
pub mod unknown;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Object {
    object_type: ObjectType,
    size: u32,
    data: ObjectData,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ObjectType {
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
pub enum ObjectData {
    Settings(Settings),
    SoundEffectOrVoice(SoundEffectOrVoice),
    EventAction(EventAction),
    Event(Event),
    RandomOrSequenceContainer(RandomOrSequenceContainer),
    SwitchContainer(SwitchContainer),
    ActorMixer(ActorMixer),
    AudioBus(AudioBus),
    BlendContainer(BlendContainer),
    MusicSegment(MusicSegment),
    MusicTrack(MusicTrack),
    MusicSwitchContainer(MusicSwitchContainer),
    MusicPlaylistContainer(MusicPlaylistContainer),
    Attenuation(Attenuation),
    DialogueEvent(DialogueEvent),
    MotionBus(MotionBus),
    MotionFx(MotionFx),
    Effect(Effect),
    Unknown(Unknown),
    AuxiliaryBus(AuxiliaryBus),
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
            _ => Err(RLibError::SoundBankUnsupportedObjectTypeFound(value))?,
        })
    }
}

impl Object {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {

        let object_type = ObjectType::try_from(data.read_u8()?)?;
        let size = data.read_u32()?;

        dbg!(object_type);
        dbg!(size);

        // NOTE: If something needs size, it means it's not decoding the data, just reading it up to memory.
        let pos_pre_data = data.stream_position()?;
        let object_data = match object_type {
            ObjectType::Settings => ObjectData::Settings(Settings::read(data, version)?),
            ObjectType::SoundEffectOrVoice => ObjectData::SoundEffectOrVoice(SoundEffectOrVoice::read(data, version, size as usize)?),
            ObjectType::EventAction => ObjectData::EventAction(EventAction::read(data, version, size as usize)?),
            ObjectType::Event => ObjectData::Event(Event::read(data, version)?),
            ObjectType::RandomOrSequenceContainer => ObjectData::RandomOrSequenceContainer(RandomOrSequenceContainer::read(data, version, size as usize)?),
            ObjectType::SwitchContainer => ObjectData::SwitchContainer(SwitchContainer::read(data, version, size as usize)?),
            ObjectType::ActorMixer => ObjectData::ActorMixer(ActorMixer::read(data, version, size as usize)?),
            ObjectType::AudioBus => ObjectData::AudioBus(AudioBus::read(data, version, size as usize)?),
            ObjectType::BlendContainer => ObjectData::BlendContainer(BlendContainer::read(data, version, size as usize)?),
            ObjectType::MusicSegment => ObjectData::MusicSegment(MusicSegment::read(data, version, size as usize)?),
            ObjectType::MusicTrack => ObjectData::MusicTrack(MusicTrack::read(data, version, size as usize)?),
            ObjectType::MusicSwitchContainer => ObjectData::MusicSwitchContainer(MusicSwitchContainer::read(data, version, size as usize)?),
            ObjectType::MusicPlaylistContainer => ObjectData::MusicPlaylistContainer(MusicPlaylistContainer::read(data, version, size as usize)?),
            ObjectType::Attenuation => ObjectData::Attenuation(Attenuation::read(data, version)?),
            ObjectType::DialogueEvent => ObjectData::DialogueEvent(DialogueEvent::read(data, version, size as usize)?),
            ObjectType::MotionBus => ObjectData::MotionBus(MotionBus::read(data, version, size as usize)?),
            ObjectType::MotionFx => ObjectData::MotionFx(MotionFx::read(data, version, size as usize)?),
            ObjectType::Effect => ObjectData::Effect(Effect::read(data, version, size as usize)?),
            ObjectType::Unknown => ObjectData::Unknown(Unknown::read(data, version, size as usize)?),
            ObjectType::AuxiliaryBus => ObjectData::AuxiliaryBus(AuxiliaryBus::read(data, version, size as usize)?),
        };

        // Check that the node is correctly decoded.
        check_size_mismatch(data.stream_position()? as usize, (pos_pre_data + size as u64) as usize)?;

        Ok(Object {
            object_type,
            size,
            data: object_data,
        })
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        buffer.write_u8(self.object_type as u8)?;

        let mut data = vec![];
        match self.data() {
            ObjectData::Settings(object) => object.write(&mut data, version)?,
            ObjectData::SoundEffectOrVoice(object) => object.write(&mut data, version)?,
            ObjectData::EventAction(object) => object.write(&mut data, version)?,
            ObjectData::Event(object) => object.write(&mut data, version)?,
            ObjectData::RandomOrSequenceContainer(object) => object.write(&mut data, version)?,
            ObjectData::SwitchContainer(object) => object.write(&mut data, version)?,
            ObjectData::ActorMixer(object) => object.write(&mut data, version)?,
            ObjectData::AudioBus(object) => object.write(&mut data, version)?,
            ObjectData::BlendContainer(object) => object.write(&mut data, version)?,
            ObjectData::MusicSegment(object) => object.write(&mut data, version)?,
            ObjectData::MusicTrack(object) => object.write(&mut data, version)?,
            ObjectData::MusicSwitchContainer(object) => object.write(&mut data, version)?,
            ObjectData::MusicPlaylistContainer(object) => object.write(&mut data, version)?,
            ObjectData::Attenuation(object) => object.write(&mut data, version)?,
            ObjectData::DialogueEvent(object) => object.write(&mut data, version)?,
            ObjectData::MotionBus(object) => object.write(&mut data, version)?,
            ObjectData::MotionFx(object) => object.write(&mut data, version)?,
            ObjectData::Effect(object) => object.write(&mut data, version)?,
            ObjectData::Unknown(object) => object.write(&mut data, version)?,
            ObjectData::AuxiliaryBus(object) => object.write(&mut data, version)?,
        }

        buffer.write_u32(data.len() as u32)?;
        buffer.write_all(&data)?;

        Ok(())
    }
}
