//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::utils::check_size_mismatch;

pub const EXTENSION: &str = ".cs2.parsed";

#[cfg(test)] mod cs2_parsed_test;

mod versions;

//PT_WALL_CLIMB,
//PT_DESTROYED_WALL_CLIMB,
//PT_WOOD_WALL_CLIMB,
//PT_DESTROYED_WOOD_WALL_CLIMB,
//PT_CLIMB_LADDER
//PT_SHIP_GRAPPLE
//PT_JUMP_DISEMBARK
//PT_JUMP_ANIM

const SHIP_STAIRCASE: i32 = 1;
const SHIP_WALK: i32 = 2;
const SHIP_LADDER: i32 = 3;
const SIEGE_LADDER1: i32 = 8;
const STAIRS: i32 = 9;
const ROPE: i32 = 10;
const DOOR_NO_TELEPORT: i32 = 13;
const JUMP: i32 = 14;                   // PT_JUMP
const WALL_DOOR_TELEPORT: i32 = 30;     // PT_WALL_DOOR
const JUMP_RAMP: i32 = 32;              // PT_JUMP_RAMP
const LADDER_LEFT: i32 = 33;
const LADDER_RIGHT: i32 = 34;
const SIEGE_LADDER2: i32 = 35;

const LOW_WALL: i32 = 0;
const HIGH_WALL: i32 = 1;
const WINDOW: i32 = 2;
const OVERFLOW: i32 = 3;
const MARINES: i32 = 4;
const SEAMEN: i32 = 5;
const GUNNERS_OVERFLOW: i32 = 6;
const CAPTAIN: i32 = 7;
const OFFICER1: i32 = 8;
const BOARDING: i32 = 9;
const NAVAL_FIRING_POSITION_STAND: i32 = 10;
const NAVAL_FIRING_POSITION_CROUCH: i32 = 11;
const NAVAL_FIRING_POSITION_STAND360: i32 = 12;
const NAVAL_PERIMETER_POSITION: i32 = 13;
const TREE: i32 = 14;
const ENTRANCE_DEFENSE: i32 = 15;
const OFFICER2: i32 = 16;
const OFFICER3: i32 = 17;
const CRENEL_LEFT_OUTER: i32 = 18;
const CRENEL_LEFT_INNER: i32 = 19;
const CRENEL_RIGHT_INNER: i32 = 20;
const CRENEL_RIGHT_OUTER: i32 = 21;
const ENGINE_PLACEMENT: i32 = 22;
const SECONDARY_ENGINE_PLACEMENT: i32 = 23;
const DISEMBARK_LEFT: i32 = 24;
const DISEMBARK_RIGHT: i32 = 25;
const NUM_PURPOSES: i32 = 26;
const INVALID_PURPOSES: i32 = 27;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Cs2Parsed {
    version: u32,
    ui_flag: UiFlag,
    bounding_box: Cube,         // Not present in v20 onwards.
    int_1: i32,
    pieces: Vec<Piece>,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Piece {
    name: String,
    node_name: String,
    node_transform: Transform4x4,
    int_3: i32,
    int_4: i32,                 // Only in v21. Array.
    destructs: Vec<Destruct>,
    f_6: f32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Destruct {
    name: String,
    index: u32,
    collision_outlines: Vec<CollisionOutline>,
    pipes: Vec<Pipe>,
    orange_thingies: Vec<Vec<OrangeThingy>>,
    platforms: Vec<Platform>,
    uk_2: i32,
    bounding_box: Cube,
    uk_3: i32,
    projectile_emitters: Vec<ProjectileEmitter>,
    uk_5: i32,
    soft_collisions: Vec<SoftCollisions>,
    uk_7: i32,
    file_refs: Vec<FileRef>,
    ef_lines: Vec<EFLine>,
    docking_lines: Vec<DockingLine>,
    f_1: f32,                               // Another array
    action_vfx: Vec<Vfx>,
    action_vfx_attachments: Vec<Vfx>,
    bin_data: Vec<Vec<i16>>,                // No idea, but looks like a list of values and the amount correlates with the mount of vfx.
    bin_data_2: Vec<Vec<i16>>,              // And no idea. Present in one destruct in 3k gates.
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct UiFlag {
    name: String,
    transform: Transform4x4,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CollisionOutline {
    name: String,
    vertices: Outline3d,
    uk_1: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct SoftCollisions {
    name: String,
    transform: Transform4x4,
    uk_1: i16,
    point_1: Point2d,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct ProjectileEmitter {
    name: String,
    transform: Transform4x4,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct FileRef {
    key: String,
    name: String,
    transform: Transform4x4,
    uk_1: i16,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct OrangeThingy {
    vertex: Point2d,
    vertex_type: u32,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Platform {
    normal: Point3d,
    vertices: Outline3d,
    flag_1: bool,
    flag_2: bool,
    flag_3: bool,
}

/// Pipes used for moving units between platforms.
///
/// Note that on ships, pipes' vertices must align with a vertex from the decks they're moving units from and into.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Pipe {
    name: String,
    line: Outline3d,
    line_type: PipeType,
}

/// Entity Formation Lines, for units to form for specific actions in specific places.
///
/// Check [EFLineType] documentation to know what types you can use. Some times may not work in certain games, and it's possible the list doesn't match older games.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct EFLine {
    name: String,
    action: EFLineType,
    start: Point3d,
    end: Point3d,
    direction: Point3d,
    parent_index: u32,
}

/// Line where siege equipment will be docked. Needed on walls for siege towers/ladders to be able to dock on them.
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct DockingLine {
    key: String,
    start: Point2d,
    end: Point2d,
    direction: Point2d,
}

#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Vfx {
    key: String,
    matrix_1: Transform4x4,
}

#[derive(PartialEq, Copy, Clone, Debug, Default, Serialize, Deserialize)]
enum PipeType {
    #[default]
    /// For ships. For units to travel through staircases.
    ShipStaircase,
    /// For ships? This is not confirmed.
    ShipWalk,
    /// For ships. For units to travel through ladders.
    ShipLadder,
    /// Ladders used to climb walls in old games (ETW/NTW).
    SiegeLadder1,
    /// Stairs used to climb a specific wall from the inside in 3K.
    Stairs,
    /// Rope used to climb walls in 3K.
    Rope,
    /// Door threshold to enter garrisonable buildings.
    DoorNoTeleport,
    /// Alternative pipe for units to jump into walls.
    Jump,
    /// Teleportation pipes, to teleport from one extreme to the other. Used in Warhammer walls to climb from the inside.
    WallDoorTeleport,
    /// Pipe used for units to jump from siege tower ramps into walls.
    JumpRamp,
    /// Ladder used in siege towers.
    LadderLeft,
    /// Ladder used in siege towers.
    LadderRight,
    /// Ladders used to climb walls in at least the WH games.
    SiegeLadder2,
}

#[derive(PartialEq, Copy, Clone, Debug, Default, Serialize, Deserialize)]
enum EFLineType {
    #[default]
    /// For mid-size walls. Also used for the first row in Warhammer walls.
    LowWall,
    /// For full size walls.
    HighWall,
    /// For windows in garrisonable buildings.
    Window,
    /// For the each row behind the second one. Also used for units standing waiting for the firing line to die and take their place.
    Overflow,
    /// For ships. Marines spawn here, and in gun placements.
    Marines,
    /// For ships. Seamen spawn here.
    Seamen,
    /// No idea.
    GunnersOverflow,
    /// For ships. Spawn point for the captain.
    Captain,
    /// For ships. Spawn point for the first officer.
    Officer1,
    /// For ships. Positions from where ropes will be launched to board other ships, and where other ship's soldiers will enter when boarding. Ships are unboardable without these.
    Boarding,
    /// For ships. Firing position for units standing.
    NavalFiringPositionStand,
    /// For ships. Firing position for units crouching.
    NavalFiringPositionCrouch,
    /// For ships. Firing position for units standing, allowing 360º fire.
    NavalFiringPositionStand360,
    /// For ships? No idea.
    NavalPerimeterPosition,
    /// No idea.
    Tree,
    /// For the entrance to garrisonable buildings. Units will defend this position in melee.
    EntranceDefense,
    /// For ships. Spawn point for the second officer.
    Officer2,
    /// For ships. Spawn point for the third officer.
    Officer3,
    /// No idea. I suspect is for position around crenelations.
    CrenelLeftOuter,
    /// No idea. I suspect is for position around crenelations.
    CrenelLeftInner,
    /// No idea. I suspect is for position around crenelations.
    CrenelRightInner,
    /// No idea. I suspect is for position around crenelations.
    CrenelRightOuter,
    /// No idea.
    EnginePlacement,
    /// No idea.
    SecondaryEnginePlacement,
    /// For ships. Point from where units will jump to land, on the left of the ship.
    DisembarkLeft,
    /// For ships. Point from where units will jump to land, on the right of the ship.
    DisembarkRight,
    /// No idea. Probably invalid.
    NumPurposes,
    /// No idea. Probably invalid.
    InvalidPurposes,
}

//---------------------------------------------------------------------------//
//                           Implementation of Cs2Parsed
//---------------------------------------------------------------------------//

impl Decodeable for Cs2Parsed {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.version = data.read_u32()?;

        match decoded.version {
            21 => decoded.read_v21(data)?,
            20 => decoded.read_v20(data)?,
            18 => decoded.read_v18(data)?,
             _ => return Err(RLibError::DecodingUnsupportedVersion(decoded.version as usize)),
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for Cs2Parsed {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.version)?;

        match self.version {
            21 => self.write_v21(buffer)?,
            20 => self.write_v20(buffer)?,
            18 => self.write_v18(buffer)?,
            _ => unimplemented!()
        }


        Ok(())
    }
}

impl TryFrom<i32> for EFLineType {
    type Error = RLibError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            LOW_WALL => Ok(Self::LowWall),
            HIGH_WALL => Ok(Self::HighWall),
            WINDOW => Ok(Self::Window),
            OVERFLOW => Ok(Self::Overflow),
            MARINES => Ok(Self::Marines),
            SEAMEN => Ok(Self::Seamen),
            GUNNERS_OVERFLOW => Ok(Self::GunnersOverflow),
            CAPTAIN => Ok(Self::Captain),
            OFFICER1 => Ok(Self::Officer1),
            BOARDING => Ok(Self::Boarding),
            NAVAL_FIRING_POSITION_STAND => Ok(Self::NavalFiringPositionStand),
            NAVAL_FIRING_POSITION_CROUCH => Ok(Self::NavalFiringPositionCrouch),
            NAVAL_FIRING_POSITION_STAND360 => Ok(Self::NavalFiringPositionStand360),
            NAVAL_PERIMETER_POSITION => Ok(Self::NavalPerimeterPosition),
            TREE => Ok(Self::Tree),
            ENTRANCE_DEFENSE => Ok(Self::EntranceDefense),
            OFFICER2 => Ok(Self::Officer2),
            OFFICER3 => Ok(Self::Officer3),
            CRENEL_LEFT_OUTER => Ok(Self::CrenelLeftOuter),
            CRENEL_LEFT_INNER => Ok(Self::CrenelLeftInner),
            CRENEL_RIGHT_INNER => Ok(Self::CrenelRightInner),
            CRENEL_RIGHT_OUTER => Ok(Self::CrenelRightOuter),
            ENGINE_PLACEMENT => Ok(Self::EnginePlacement),
            SECONDARY_ENGINE_PLACEMENT => Ok(Self::SecondaryEnginePlacement),
            DISEMBARK_LEFT => Ok(Self::DisembarkLeft),
            DISEMBARK_RIGHT => Ok(Self::DisembarkRight),
            NUM_PURPOSES => Ok(Self::NumPurposes),
            INVALID_PURPOSES => Ok(Self::InvalidPurposes),
            _ => Err(RLibError::UnknownEFLineType(value.to_string())),
        }
    }
}

impl From<EFLineType> for i32 {
    fn from(value: EFLineType) -> Self {
        match value {
            EFLineType::LowWall => LOW_WALL,
            EFLineType::HighWall => HIGH_WALL,
            EFLineType::Window => WINDOW,
            EFLineType::Overflow => OVERFLOW,
            EFLineType::Marines => MARINES,
            EFLineType::Seamen => SEAMEN,
            EFLineType::GunnersOverflow => GUNNERS_OVERFLOW,
            EFLineType::Captain => CAPTAIN,
            EFLineType::Officer1 => OFFICER1,
            EFLineType::Boarding => BOARDING,
            EFLineType::NavalFiringPositionStand => NAVAL_FIRING_POSITION_STAND,
            EFLineType::NavalFiringPositionCrouch => NAVAL_FIRING_POSITION_CROUCH,
            EFLineType::NavalFiringPositionStand360 => NAVAL_FIRING_POSITION_STAND360,
            EFLineType::NavalPerimeterPosition => NAVAL_PERIMETER_POSITION,
            EFLineType::Tree => TREE,
            EFLineType::EntranceDefense => ENTRANCE_DEFENSE,
            EFLineType::Officer2 => OFFICER2,
            EFLineType::Officer3 => OFFICER3,
            EFLineType::CrenelLeftOuter => CRENEL_LEFT_OUTER,
            EFLineType::CrenelLeftInner => CRENEL_LEFT_INNER,
            EFLineType::CrenelRightInner => CRENEL_RIGHT_INNER,
            EFLineType::CrenelRightOuter => CRENEL_RIGHT_OUTER,
            EFLineType::EnginePlacement => ENGINE_PLACEMENT,
            EFLineType::SecondaryEnginePlacement => SECONDARY_ENGINE_PLACEMENT,
            EFLineType::DisembarkLeft => DISEMBARK_LEFT,
            EFLineType::DisembarkRight => DISEMBARK_RIGHT,
            EFLineType::NumPurposes => NUM_PURPOSES,
            EFLineType::InvalidPurposes => INVALID_PURPOSES
        }
    }
}

impl TryFrom<i32> for PipeType {
    type Error = RLibError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            SHIP_STAIRCASE => Ok(Self::ShipStaircase),
            SHIP_WALK => Ok(Self::ShipWalk),
            SHIP_LADDER => Ok(Self::ShipLadder),
            SIEGE_LADDER1 => Ok(Self::SiegeLadder1),
            STAIRS => Ok(Self::Stairs),
            ROPE => Ok(Self::Rope),
            DOOR_NO_TELEPORT => Ok(Self::DoorNoTeleport),
            JUMP => Ok(Self::Jump),
            WALL_DOOR_TELEPORT => Ok(Self::WallDoorTeleport),
            JUMP_RAMP => Ok(Self::JumpRamp),
            LADDER_LEFT => Ok(Self::LadderLeft),
            LADDER_RIGHT => Ok(Self::LadderRight),
            SIEGE_LADDER2 => Ok(Self::SiegeLadder2),
            _ => Err(RLibError::UnknownPipeType(value.to_string())),
        }
    }
}

impl From<PipeType> for i32 {
    fn from(value: PipeType) -> Self {
        match value {
            PipeType::ShipStaircase => SHIP_STAIRCASE,
            PipeType::ShipWalk => SHIP_WALK,
            PipeType::ShipLadder => SHIP_LADDER,
            PipeType::SiegeLadder1 => SIEGE_LADDER1,
            PipeType::Stairs => STAIRS,
            PipeType::Rope => ROPE,
            PipeType::DoorNoTeleport => DOOR_NO_TELEPORT,
            PipeType::Jump => JUMP,
            PipeType::WallDoorTeleport => WALL_DOOR_TELEPORT,
            PipeType::JumpRamp => JUMP_RAMP,
            PipeType::LadderLeft => LADDER_LEFT,
            PipeType::LadderRight => LADDER_RIGHT,
            PipeType::SiegeLadder2 => SIEGE_LADDER2
        }
    }
}


