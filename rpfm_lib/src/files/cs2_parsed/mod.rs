//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! CS2 Parsed file format support.
//!
//! CS2 Parsed files (`.cs2.parsed`) define gameplay logic and interaction data for 3D
//! building models in Total War games. These files contain information about unit
//! placement, pathfinding, collision, defense positions, and various building-specific
//! behaviors.
//!
//! # File Format
//!
//! CS2 Parsed files are binary files containing:
//! - Version header
//! - UI flag position for minimap display
//! - Building pieces with destruction states
//! - Gameplay logic: platforms, pipes, gates, EF lines, etc.
//! - Legacy collision data (moved to separate `.cs2.collision` files in newer games)
//! - Projectile emitters (moved to map data in newer games)
//!
//! # Supported Versions
//!
//! - **Version 0**: Legacy format (Empire/Napoleon)
//! - **Version 8**: Legacy format
//! - **Version 9**: Legacy format
//! - **Version 10**: Legacy format
//! - **Version 11**: Legacy format
//! - **Version 12**: Legacy format
//! - **Version 13**: Legacy format
//! - **Version 18**: Older format (Three Kingdoms)
//! - **Version 20**: Older format (Troy/Warhammer II)
//! - **Version 21**: Current format (Warhammer III)
//!
//! # Key Components
//!
//! ## Platforms
//! Define walkable surfaces where units can stand and fight.
//!
//! ## Pipes
//! Define paths for unit movement between platforms (stairs, ladders, doors, etc.).
//!
//! ## EF Lines (Entity Formation Lines)
//! Define positions where units form up for specific actions (firing lines, boarding
//! positions, officer spawn points, etc.).
//!
//! ## Gates
//! Define entrance/exit collision for wall gates.
//!
//! ## Docking Lines
//! Define where siege equipment can attach to walls.
//!
//! # Usage
//!
//! ```rust,ignore
//! use rpfm_lib::files::cs2_parsed::Cs2Parsed;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode from binary data
//! let parsed = Cs2Parsed::decode(&mut data, &None)?;
//!
//! // Access building pieces
//! for piece in parsed.pieces() {
//!     println!("Piece: {}", piece.name());
//!
//!     // Access destructs (damage states)
//!     for destruct in piece.destructs() {
//!         println!("  Destruct: {} ({} platforms, {} pipes)",
//!             destruct.name(),
//!             destruct.platforms().len(),
//!             destruct.pipes().len()
//!         );
//!     }
//! }
//! ```
//!
//! # File Location
//!
//! These files are typically found at:
//! - `rigidmodels/buildings/*/*.cs2.parsed`

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{cs2_collision::Collision3d, DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::games::GameInfo;
use crate::utils::check_size_mismatch;

/// File extension for CS2 Parsed files.
pub const EXTENSION: &str = ".cs2.parsed";

#[cfg(test)] mod cs2_parsed_test;

mod versions;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a CS2 Parsed file decoded in memory.
///
/// Contains all gameplay logic data for a building model, including unit placement,
/// pathfinding, collision, and various building-specific behaviors.
///
/// # Fields
///
/// * `version` - File format version (0, 8-13, 18, 20, or 21)
/// * `ui_flag` - Flag position shown on minimap
/// * `bounding_box` - Overall bounds (not present in v20 onwards)
/// * `int_1` - Unknown field
/// * `pieces` - List of building pieces with destruction states
///
/// # Examples
///
/// ```rust,ignore
/// let parsed = Cs2Parsed::decode(&mut data, &None)?;
/// println!("Version: {}", parsed.version());
/// println!("Pieces: {}", parsed.pieces().len());
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Cs2Parsed {
    /// File format version number.
    version: u32,

    /// Flag position for minimap display.
    ui_flag: UiFlag,

    /// Overall bounding box (not present in v20 onwards).
    bounding_box: Cube,

    /// Unknown field.
    int_1: i32,

    /// List of building pieces.
    pieces: Vec<Piece>,
}

/// A piece of a building model with multiple destruction states.
///
/// Buildings are composed of multiple pieces, each with one or more destruction
/// states (destructs). As a building takes damage, it transitions between these states.
///
/// # Fields
///
/// * `name` - Name identifier for this piece
/// * `node_name` - Scene node name for attachment
/// * `node_transform` - Transformation matrix for positioning
/// * `int_3` - Unknown field
/// * `int_4` - Unknown field (only in v21)
/// * `destructs` - List of destruction states for this piece
/// * `f_6` - Unknown field
///
/// # Examples
///
/// ```rust,ignore
/// for piece in parsed.pieces() {
///     println!("Piece: {} ({} destructs)", piece.name(), piece.destructs().len());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Piece {
    /// Name identifier for this piece.
    name: String,

    /// Scene node name for attachment.
    node_name: String,

    /// Transformation matrix for positioning.
    node_transform: Transform4x4,

    /// Unknown field.
    int_3: i32,

    /// Unknown field (only in v21).
    int_4: i32,

    /// List of destruction states for this piece.
    destructs: Vec<Destruct>,

    /// Unknown field.
    f_6: f32,
}

/// A destruction state of a building piece.
///
/// Each piece can have multiple destructs representing different damage states.
/// Destructs contain all the gameplay logic for that damage state: platforms,
/// pipes, gates, EF lines, collision, etc.
///
/// # Fields
///
/// * `name` - Name identifier for this destruction state
/// * `index` - Index of this destruct
/// * `collision_3d` - Collision mesh (legacy, moved to `.cs2.collision` in newer games)
/// * `collision_outlines` - 2D collision outlines
/// * `windows` - Number of window positions
/// * `doors` - Number of door positions
/// * `gates` - Gate collision data for wall gates
/// * `pipes` - Unit movement paths (stairs, ladders, doors, etc.)
/// * `orange_thingies` - Unknown (possibly no-go zones)
/// * `platforms` - Walkable surfaces for units
/// * `uk_2` - Unknown field
/// * `bounding_box` - Bounding box for this destruct
/// * `cannon_emitters` - Projectile emitter count for cannons (legacy)
/// * `arrow_emitters` - Projectile emitters for arrows (legacy)
/// * `docking_points` - Number of docking points
/// * `soft_collisions` - Soft collision data
/// * `uk_7` - Unknown field
/// * `file_refs` - Attached model references (e.g., torches)
/// * `ef_lines` - Entity Formation lines for unit positioning
/// * `docking_lines` - Lines where siege equipment can attach
/// * `f_1` - Unknown field
/// * `action_vfx` - VFX for actions
/// * `action_vfx_attachments` - VFX attachment points
/// * `bin_data` - Unknown binary data (correlates with VFX count)
/// * `bin_data_2` - Unknown binary data (present in some Three Kingdoms gates)
///
/// # Examples
///
/// ```rust,ignore
/// for destruct in piece.destructs() {
///     println!("Destruct '{}': {} platforms, {} pipes",
///         destruct.name(),
///         destruct.platforms().len(),
///         destruct.pipes().len()
///     );
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Destruct {
    /// Name identifier for this destruction state.
    name: String,

    /// Index of this destruct.
    index: u32,

    /// Collision mesh (legacy, moved to separate `.cs2.collision` files in newer games).
    collision_3d: Collision3d,

    /// 2D collision outlines.
    collision_outlines: Vec<CollisionOutline>,

    /// Number of window positions.
    windows: i32,

    /// Number of door positions.
    doors: i32,

    /// Gate collision data for wall gates.
    gates: Vec<Gate>,

    /// Unit movement paths between platforms.
    pipes: Vec<Pipe>,

    /// Unknown (possibly no-go zones).
    orange_thingies: Vec<Vec<OrangeThingy>>,

    /// Walkable platform surfaces.
    platforms: Vec<Platform>,

    /// Unknown field.
    uk_2: i32,

    /// Bounding box for this destruct.
    bounding_box: Cube,

    /// Projectile emitter count for cannons (legacy, moved to map in newer games).
    cannon_emitters: i32,

    /// Projectile emitters for arrows (legacy, moved to map in newer games).
    arrow_emitters: Vec<ProjectileEmitter>,

    /// Number of docking points.
    docking_points: i32,

    /// Soft collision data.
    soft_collisions: Vec<SoftCollisions>,

    /// Unknown field.
    uk_7: i32,

    /// Attached model references (e.g., torches).
    file_refs: Vec<FileRef>,

    /// Entity Formation lines for unit positioning.
    ef_lines: Vec<EFLine>,

    /// Lines where siege equipment can attach.
    docking_lines: Vec<DockingLine>,

    /// Unknown field.
    f_1: f32,

    /// VFX for actions.
    action_vfx: Vec<Vfx>,

    /// VFX attachment points.
    action_vfx_attachments: Vec<Vfx>,

    /// Unknown binary data (correlates with VFX count).
    bin_data: Vec<Vec<i16>>,

    /// Unknown binary data (present in some Three Kingdoms gates).
    bin_data_2: Vec<Vec<i16>>,
}

/// UI flag position shown on the minimap.
///
/// Defines where the building's flag icon appears on the tactical map.
///
/// # Fields
///
/// * `name` - Name identifier for the flag
/// * `transform` - Transformation matrix for flag position and orientation
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct UiFlag {
    /// Name identifier for the flag.
    name: String,

    /// Transformation matrix for flag position.
    transform: Transform4x4,
}

/// Gate collision data for wall gates.
///
/// Defines the collision geometry for gates where units enter and exit cities.
/// Contains two collision meshes, possibly for open and closed states.
///
/// # Fields
///
/// * `collision_1` - First collision mesh
/// * `collision_2` - Second collision mesh
/// * `uk_1` - Unknown field
/// * `uk_2` - Unknown field
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Gate {
    /// First collision mesh.
    collision_1: Collision3d,

    /// Second collision mesh.
    collision_2: Collision3d,

    /// Unknown field.
    uk_1: u32,

    /// Unknown field.
    uk_2: u32,
}

/// A 3D collision outline.
///
/// Defines a named 3D outline used for collision detection.
///
/// # Fields
///
/// * `name` - Name identifier for this collision outline
/// * `vertices` - 3D polyline defining the outline
/// * `uk_1` - Unknown field
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct CollisionOutline {
    /// Name identifier for this collision outline.
    name: String,

    /// 3D polyline vertices.
    vertices: Outline3d,

    /// Unknown field.
    uk_1: u32,
}

/// Soft collision data.
///
/// Defines soft collision zones with transforms and positions.
///
/// # Fields
///
/// * `name` - Name identifier
/// * `transform` - Transformation matrix
/// * `uk_1` - Unknown field
/// * `point_1` - 2D point position
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct SoftCollisions {
    /// Name identifier.
    name: String,

    /// Transformation matrix.
    transform: Transform4x4,

    /// Unknown field.
    uk_1: i16,

    /// 2D point position.
    point_1: Point2d,
}

/// Projectile emitter position for building-based ranged attacks.
///
/// Defines where projectiles (arrows/cannonballs) are fired from on a building.
/// Legacy feature used in Thrones of Britannia and older games. Newer games
/// moved this logic to the map itself.
///
/// # Fields
///
/// * `name` - Name identifier for this emitter
/// * `transform` - Transformation matrix for emitter position and orientation
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct ProjectileEmitter {
    /// Name identifier for this emitter.
    name: String,

    /// Transformation matrix for position and orientation.
    transform: Transform4x4,
}

/// Reference to an attached building model.
///
/// Used to attach additional models to buildings (e.g., torches on Attila's walls,
/// decorative elements).
///
/// # Fields
///
/// * `key` - Path to the model file to attach
/// * `name` - Name identifier for this attachment
/// * `transform` - Transformation matrix for attachment position
/// * `uk_1` - Unknown field (possibly unique ID within the file)
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct FileRef {
    /// Path to the model file to attach.
    key: String,

    /// Name identifier for this attachment.
    name: String,

    /// Transformation matrix for attachment position.
    transform: Transform4x4,

    /// Unknown field (possibly unique ID within the file).
    uk_1: i16,
}

/// Unknown vertex data (possibly no-go zones).
///
/// # Fields
///
/// * `vertex` - 2D vertex position
/// * `vertex_type` - Type identifier for this vertex
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct OrangeThingy {
    /// 2D vertex position.
    vertex: Point2d,

    /// Type identifier for this vertex.
    vertex_type: u32,
}

/// A walkable platform surface where units can stand and fight.
///
/// Platforms define the areas where units can be placed within a building.
/// Different flags control how the pathfinder treats the platform.
///
/// # Fields
///
/// * `normal` - Surface normal vector
/// * `vertices` - 3D outline defining the platform boundary
/// * `flag_1` - Unknown behavior flag
/// * `flag_2` - Treats platform as ground if true (units can walk freely)
/// * `flag_3` - Unknown flag (set in siege tower ramp platforms)
///
/// # Examples
///
/// ```rust,ignore
/// for platform in destruct.platforms() {
///     if *platform.flag_2() {
///         println!("Platform '{}' is treated as ground", /* no name field */);
///     }
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Platform {
    /// Surface normal vector.
    normal: Point3d,

    /// 3D polyline defining platform boundary.
    vertices: Outline3d,

    /// Unknown behavior flag.
    flag_1: bool,

    /// Treats platform as ground if true (units can walk freely).
    flag_2: bool,

    /// Unknown flag (set in siege tower ramp platforms).
    flag_3: bool,
}

/// A path for unit movement between platforms.
///
/// Pipes define how units move between different platform levels (stairs, ladders,
/// doors, teleports, etc.). On ships, pipe vertices must align with deck vertices.
///
/// # Fields
///
/// * `name` - Name identifier for this pipe
/// * `line` - 3D polyline defining the movement path
/// * `line_type` - Type of pipe (stairs, ladder, door, etc.)
///
/// # Examples
///
/// ```rust,ignore
/// for pipe in destruct.pipes() {
///     println!("Pipe '{}': {:?}", pipe.name(), pipe.line_type());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Pipe {
    /// Name identifier for this pipe.
    name: String,

    /// 3D polyline defining the movement path.
    line: Outline3d,

    /// Type of pipe (stairs, ladder, door, etc.).
    line_type: PipeType,
}

/// Entity Formation line for unit positioning and actions.
///
/// EF Lines define where units should form up for specific actions like firing,
/// boarding, defending, or spawning. Different types serve different purposes.
/// See `EFLineType` for available types and their uses.
///
/// # Fields
///
/// * `name` - Name identifier for this line
/// * `action` - Type of action/formation for this line
/// * `start` - Starting point of the line
/// * `end` - Ending point of the line
/// * `direction` - Direction vector for unit orientation
/// * `parent_index` - Parent index for hierarchical relationships
///
/// # Examples
///
/// ```rust,ignore
/// for ef_line in destruct.ef_lines() {
///     println!("EF Line '{}': {:?}", ef_line.name(), ef_line.action());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct EFLine {
    /// Name identifier for this line.
    name: String,

    /// Type of action/formation for this line.
    action: EFLineType,

    /// Starting point of the line.
    start: Point3d,

    /// Ending point of the line.
    end: Point3d,

    /// Direction vector for unit orientation.
    direction: Point3d,

    /// Parent index for hierarchical relationships.
    parent_index: u32,
}

/// Line where siege equipment can attach to walls.
///
/// Docking lines are required on walls for siege towers and ladders to attach.
/// Without these, siege equipment cannot dock to the wall.
///
/// # Fields
///
/// * `key` - Key identifier for this docking line
/// * `start` - Starting point of the line
/// * `end` - Ending point of the line
/// * `direction` - Direction vector for docking orientation
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct DockingLine {
    /// Key identifier for this docking line.
    key: String,

    /// Starting point of the line.
    start: Point2d,

    /// Ending point of the line.
    end: Point2d,

    /// Direction vector for docking orientation.
    direction: Point2d,
}

/// Visual effects attachment point.
///
/// Defines where VFX (particle effects, animations, etc.) are attached to the building.
///
/// # Fields
///
/// * `key` - Path to the VFX resource
/// * `matrix_1` - Transformation matrix for VFX position and orientation
#[derive(PartialEq, Clone, Debug, Default, Getters, Setters, Serialize, Deserialize)]
pub struct Vfx {
    /// Path to the VFX resource.
    key: String,

    /// Transformation matrix for VFX position and orientation.
    matrix_1: Transform4x4,
}

/// Type of pipe for unit movement.
///
/// Defines the movement behavior for a pipe. Different types handle different
/// movement scenarios like stairs, ladders, doors, teleportation, etc.
///
/// # Variants by Use Case
///
/// ## Naval Movement (Ships)
/// - [`ShipStaircase`](Self::ShipStaircase) - Movement through ship staircases
/// - [`ShipWalk`](Self::ShipWalk) - Walking movement on ships (unconfirmed)
/// - [`ShipLadder`](Self::ShipLadder) - Climbing ship ladders
///
/// ## Wall Climbing
/// - [`SiegeLadder1`](Self::SiegeLadder1) - Wall ladders in Empire/Napoleon
/// - [`SiegeLadder2`](Self::SiegeLadder2) - Wall ladders in Warhammer games
/// - [`Rope`](Self::Rope) - Rope climbing in Three Kingdoms
///
/// ## Siege Equipment
/// - [`LadderLeft`](Self::LadderLeft) - Left ladder in siege towers
/// - [`LadderRight`](Self::LadderRight) - Right ladder in siege towers
/// - [`JumpRamp`](Self::JumpRamp) - Jumping from siege tower ramps
///
/// ## Doors and Entry
/// - [`DoorNoTeleport`](Self::DoorNoTeleport) - Door threshold for garrisonable buildings
/// - [`WallDoorTeleport`](Self::WallDoorTeleport) - Teleport between ends (Warhammer walls)
/// - [`GroundTeleport`](Self::GroundTeleport) - Barricade teleportation (Warhammer III)
///
/// ## Other
/// - [`Stairs`](Self::Stairs) - Interior wall stairs (Three Kingdoms)
/// - [`Jump`](Self::Jump) - Jumping onto walls
/// - [`UnknownSambucaPipe`](Self::UnknownSambucaPipe) - Sambuca-related (Thrones of Britannia)
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(i32)]
enum PipeType {
    /// Ship staircase movement.
    ShipStaircase = 1,
    /// Ship walking movement (unconfirmed).
    ShipWalk = 2,
    /// Ship ladder climbing.
    ShipLadder = 3,
    /// Wall climbing ladders (Empire/Napoleon).
    SiegeLadder1 = 8,
    /// Interior wall stairs (Three Kingdoms).
    Stairs = 9,
    /// Rope wall climbing (Three Kingdoms).
    Rope = 10,
    /// Sambuca pipe (Thrones of Britannia).
    UnknownSambucaPipe = 11,
    /// Door entry threshold for garrisonable buildings.
    DoorNoTeleport = 13,
    /// Jumping onto walls (PT_JUMP).
    Jump = 14,
    /// Teleportation between pipe ends (Warhammer walls, PT_WALL_DOOR).
    WallDoorTeleport = 30,
    /// Jumping from siege tower ramps to walls (PT_JUMP_RAMP).
    JumpRamp = 32,
    /// Left ladder in siege towers.
    LadderLeft = 33,
    /// Right ladder in siege towers.
    LadderRight = 34,
    /// Wall climbing ladders (Warhammer).
    SiegeLadder2 = 35,
    /// Barricade teleportation (Warhammer III).
    GroundTeleport = 38,
}

impl Default for PipeType {
    fn default() -> Self {
        Self::ShipStaircase
    }
}

/// Type of Entity Formation line.
///
/// Defines the purpose of an EF line - where units should position themselves
/// for specific actions like firing, boarding, defending, or spawning.
///
/// # Variants by Use Case
///
/// ## Wall Defense
/// - [`LowWall`](Self::LowWall) - Mid-size walls, first row in Warhammer
/// - [`HighWall`](Self::HighWall) - Full-size walls
/// - [`Overflow`](Self::Overflow) - Rows behind the second, waiting positions
/// - [`CrenelLeftOuter`](Self::CrenelLeftOuter) - Crenellation positions (likely)
/// - [`CrenelLeftInner`](Self::CrenelLeftInner) - Crenellation positions (likely)
/// - [`CrenelRightInner`](Self::CrenelRightInner) - Crenellation positions (likely)
/// - [`CrenelRightOuter`](Self::CrenelRightOuter) - Crenellation positions (likely)
///
/// ## Building Defense
/// - [`Window`](Self::Window) - Window firing positions
/// - [`EntranceDefense`](Self::EntranceDefense) - Melee defense at building entrances
///
/// ## Naval - Spawn Points
/// - [`Marines`](Self::Marines) - Marine spawn locations and gun placements
/// - [`Seamen`](Self::Seamen) - Seamen spawn locations
/// - [`Captain`](Self::Captain) - Captain spawn point
/// - [`Officer1`](Self::Officer1) - First officer spawn point
/// - [`Officer2`](Self::Officer2) - Second officer spawn point
/// - [`Officer3`](Self::Officer3) - Third officer spawn point
///
/// ## Naval - Combat Positions
/// - [`NavalFiringPositionStand`](Self::NavalFiringPositionStand) - Standing firing positions
/// - [`NavalFiringPositionCrouch`](Self::NavalFiringPositionCrouch) - Crouching firing positions
/// - [`NavalFiringPositionStand360`](Self::NavalFiringPositionStand360) - 360° standing fire
/// - [`NavalPerimeterPosition`](Self::NavalPerimeterPosition) - Perimeter positions (unknown)
///
/// ## Naval - Boarding and Disembark
/// - [`Boarding`](Self::Boarding) - Boarding rope launch/entry points (required for boarding)
/// - [`DisembarkLeft`](Self::DisembarkLeft) - Left-side disembarkation point
/// - [`DisembarkRight`](Self::DisembarkRight) - Right-side disembarkation point
///
/// ## Equipment and Other
/// - [`EnginePlacement`](Self::EnginePlacement) - Engine placement (unknown)
/// - [`SecondaryEnginePlacement`](Self::SecondaryEnginePlacement) - Secondary engine (unknown)
/// - [`GunnersOverflow`](Self::GunnersOverflow) - Gunner overflow (unknown)
/// - [`Tree`](Self::Tree) - Unknown purpose
///
/// ## Invalid/Reserved
/// - [`NumPurposes`](Self::NumPurposes) - Probably invalid
/// - [`InvalidPurposes`](Self::InvalidPurposes) - Probably invalid
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[repr(i32)]
enum EFLineType {
    /// Mid-size walls, first row in Warhammer walls.
    LowWall = 0,
    /// Full-size walls.
    HighWall = 1,
    /// Window firing positions in garrisonable buildings.
    Window = 2,
    /// Rows behind the second one, waiting positions.
    Overflow = 3,
    /// Naval: Marine spawn locations and gun placements.
    Marines = 4,
    /// Naval: Seamen spawn locations.
    Seamen = 5,
    /// Naval: Gunner overflow (unknown).
    GunnersOverflow = 6,
    /// Naval: Captain spawn point.
    Captain = 7,
    /// Naval: First officer spawn point.
    Officer1 = 8,
    /// Naval: Boarding rope launch/entry points (required for boarding).
    Boarding = 9,
    /// Naval: Standing firing positions.
    NavalFiringPositionStand = 10,
    /// Naval: Crouching firing positions.
    NavalFiringPositionCrouch = 11,
    /// Naval: 360° standing firing positions.
    NavalFiringPositionStand360 = 12,
    /// Naval: Perimeter positions (unknown).
    NavalPerimeterPosition = 13,
    /// Unknown purpose.
    Tree = 14,
    /// Melee defense at building entrances.
    EntranceDefense = 15,
    /// Naval: Second officer spawn point.
    Officer2 = 16,
    /// Naval: Third officer spawn point.
    Officer3 = 17,
    /// Crenellation positions (likely).
    CrenelLeftOuter = 18,
    /// Crenellation positions (likely).
    CrenelLeftInner = 19,
    /// Crenellation positions (likely).
    CrenelRightInner = 20,
    /// Crenellation positions (likely).
    CrenelRightOuter = 21,
    /// Engine placement (unknown).
    EnginePlacement = 22,
    /// Secondary engine placement (unknown).
    SecondaryEnginePlacement = 23,
    /// Naval: Left-side disembarkation point.
    DisembarkLeft = 24,
    /// Naval: Right-side disembarkation point.
    DisembarkRight = 25,
    /// Probably invalid.
    NumPurposes = 26,
    /// Probably invalid.
    InvalidPurposes = 27,
}

impl Default for EFLineType {
    fn default() -> Self {
        Self::LowWall
    }
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
            13 => decoded.read_v13(data)?,
            12 => decoded.read_v12(data)?,
            11 => decoded.read_v11(data)?,
            10 => decoded.read_v10(data)?,
            9 => decoded.read_v9(data)?,
            8 => decoded.read_v8(data)?,
            0 => decoded.read_v0(data)?,
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
            13 => self.write_v13(buffer)?,
            12 => self.write_v12(buffer)?,
            11 => self.write_v11(buffer)?,
            10 => self.write_v10(buffer)?,
            9 => self.write_v9(buffer)?,
            8 => self.write_v8(buffer)?,
            0 => self.write_v0(buffer)?,
            _ => unimplemented!()
        }

        Ok(())
    }
}

impl TryFrom<i32> for EFLineType {
    type Error = RLibError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::LowWall as i32 => Ok(Self::LowWall),
            _ if value == Self::HighWall as i32 => Ok(Self::HighWall),
            _ if value == Self::Window as i32 => Ok(Self::Window),
            _ if value == Self::Overflow as i32 => Ok(Self::Overflow),
            _ if value == Self::Marines as i32 => Ok(Self::Marines),
            _ if value == Self::Seamen as i32 => Ok(Self::Seamen),
            _ if value == Self::GunnersOverflow as i32 => Ok(Self::GunnersOverflow),
            _ if value == Self::Captain as i32 => Ok(Self::Captain),
            _ if value == Self::Officer1 as i32 => Ok(Self::Officer1),
            _ if value == Self::Boarding as i32 => Ok(Self::Boarding),
            _ if value == Self::NavalFiringPositionStand as i32 => Ok(Self::NavalFiringPositionStand),
            _ if value == Self::NavalFiringPositionCrouch as i32 => Ok(Self::NavalFiringPositionCrouch),
            _ if value == Self::NavalFiringPositionStand360 as i32 => Ok(Self::NavalFiringPositionStand360),
            _ if value == Self::NavalPerimeterPosition as i32 => Ok(Self::NavalPerimeterPosition),
            _ if value == Self::Tree as i32 => Ok(Self::Tree),
            _ if value == Self::EntranceDefense as i32 => Ok(Self::EntranceDefense),
            _ if value == Self::Officer2 as i32 => Ok(Self::Officer2),
            _ if value == Self::Officer3 as i32 => Ok(Self::Officer3),
            _ if value == Self::CrenelLeftOuter as i32 => Ok(Self::CrenelLeftOuter),
            _ if value == Self::CrenelLeftInner as i32 => Ok(Self::CrenelLeftInner),
            _ if value == Self::CrenelRightInner as i32 => Ok(Self::CrenelRightInner),
            _ if value == Self::CrenelRightOuter as i32 => Ok(Self::CrenelRightOuter),
            _ if value == Self::EnginePlacement as i32 => Ok(Self::EnginePlacement),
            _ if value == Self::SecondaryEnginePlacement as i32 => Ok(Self::SecondaryEnginePlacement),
            _ if value == Self::DisembarkLeft as i32 => Ok(Self::DisembarkLeft),
            _ if value == Self::DisembarkRight as i32 => Ok(Self::DisembarkRight),
            _ if value == Self::NumPurposes as i32 => Ok(Self::NumPurposes),
            _ if value == Self::InvalidPurposes as i32 => Ok(Self::InvalidPurposes),
            _ => Err(RLibError::UnknownEFLineType(value.to_string())),
        }
    }
}

impl From<EFLineType> for i32 {
    fn from(value: EFLineType) -> Self {
        value as i32
    }
}

impl TryFrom<i32> for PipeType {
    type Error = RLibError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            _ if value == Self::ShipStaircase as i32 => Ok(Self::ShipStaircase),
            _ if value == Self::ShipWalk as i32 => Ok(Self::ShipWalk),
            _ if value == Self::ShipLadder as i32 => Ok(Self::ShipLadder),
            _ if value == Self::SiegeLadder1 as i32 => Ok(Self::SiegeLadder1),
            _ if value == Self::Stairs as i32 => Ok(Self::Stairs),
            _ if value == Self::Rope as i32 => Ok(Self::Rope),
            _ if value == Self::UnknownSambucaPipe as i32 => Ok(Self::UnknownSambucaPipe),
            _ if value == Self::DoorNoTeleport as i32 => Ok(Self::DoorNoTeleport),
            _ if value == Self::Jump as i32 => Ok(Self::Jump),
            _ if value == Self::WallDoorTeleport as i32 => Ok(Self::WallDoorTeleport),
            _ if value == Self::JumpRamp as i32 => Ok(Self::JumpRamp),
            _ if value == Self::LadderLeft as i32 => Ok(Self::LadderLeft),
            _ if value == Self::LadderRight as i32 => Ok(Self::LadderRight),
            _ if value == Self::SiegeLadder2 as i32 => Ok(Self::SiegeLadder2),
            _ if value == Self::GroundTeleport as i32 => Ok(Self::GroundTeleport),
            _ => Err(RLibError::UnknownPipeType(value.to_string())),
        }
    }
}

impl From<PipeType> for i32 {
    fn from(value: PipeType) -> Self {
        value as i32
    }
}

impl Cs2Parsed {

    /// Migrates this CS2 Parsed file to be compatible with a specific game.
    ///
    /// Converts the file version to the maximum version supported by the target game.
    /// Games generally support all previous format versions, so migration only occurs
    /// if the current file version is newer than the game's maximum supported version.
    ///
    /// # Arguments
    ///
    /// * `game` - Target game information containing maximum supported version
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Migration successful or not needed
    /// * `Err(RLibError::GameDoesntSupportCs2Migration)` - Game doesn't support CS2 files
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut parsed = Cs2Parsed::decode(&mut data, &None)?;
    /// parsed.migrate_game(&game_info)?;
    /// // File is now compatible with the target game
    /// ```
    pub fn migrate_game(&mut self, game: &GameInfo) -> Result<()> {

        if *game.max_cs2_parsed_version() == 0 {
            return Err(RLibError::GameDoesntSupportCs2Migration)
        }

        // Games (at least until thrones) seem to support all previous formats.
        // So we only perform a migration if the format of the file is newer than the latest supported one by the game.
        if self.version > *game.max_cs2_parsed_version() {
            self.version = *game.max_cs2_parsed_version();
        }

        Ok(())
    }
}
