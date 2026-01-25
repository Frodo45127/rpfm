//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module provides support for reading and writing ESF (Empire Save File) files.
//!
//! # Overview
//!
//! ESF files are hierarchical binary files used extensively in Total War games to store
//! structured game data. They are found in various contexts including:
//!
//! - Campaign save games (`.save`, `.save_multiplayer`)
//! - Campaign startup positions (startpos)
//! - Character save files (`.twc`)
//! - Composite Scene files (`.csc`)
//! - Campaign Effect Object files (`.ccd`)
//! - General ESF data (`.esf`)
//!
//! # File Structure
//!
//! An ESF file consists of:
//!
//! 1. **Header**: Contains the signature, unknown field, creation date, and offset to string tables.
//! 2. **Node Tree**: A recursive tree structure starting from a root record node, containing
//!    all the actual data organized hierarchically.
//! 3. **String Tables**: Three separate tables at the end of the file:
//!    - Record names: Names used by record nodes (e.g., "CAMPAIGN_SAVE_GAME", "FACTION")
//!    - UTF-16 strings: String values referenced by UTF-16 string nodes
//!    - UTF-8/ASCII strings: String values referenced by ASCII string nodes
//!
//! # Supported Signatures
//!
//! ESF files come in several format versions identified by their signature:
//!
//! | Signature | Status        | Notes                                   |
//! |-----------|---------------|-----------------------------------------|
//! | CAAB      | ✅ Supported  | Older format, uses u16 for string sizes |
//! | CBAB      | ✅ Supported  | Newer format, uses u32 for string sizes |
//! | CEAB      | ❌ Unsupported | Rare format                             |
//! | CFAB      | ❌ Unsupported | Rare format                             |
//!
//! # Node Types
//!
//! The ESF format supports a rich set of node types organized into categories:
//!
//! ## Primitive Types
//! - Boolean, signed/unsigned integers (8/16/32/64-bit), floats (32/64-bit)
//! - 2D and 3D coordinates (pairs/triplets of f32)
//! - UTF-16 and ASCII strings (stored as indices into string tables)
//! - Angles (i16)
//!
//! ## Optimized Primitives
//! Many primitive types have optimized encodings for common values to reduce file size:
//! - `BOOL_TRUE`/`BOOL_FALSE`: Single byte instead of marker + value
//! - `U32_ZERO`/`U32_ONE`: Single byte for 0 or 1
//! - `U32_BYTE`/`U32_16BIT`/`U32_24BIT`: Smaller encodings when value fits
//! - Similar optimizations exist for i32 and f32 (zero)
//!
//! ## Arrays
//! All primitive types have corresponding array variants that store multiple values
//! with a length prefix. Arrays also support optimized encodings.
//!
//! ## Record Nodes
//! Record nodes are container nodes that hold other nodes. They have:
//! - A name (from the record names table)
//! - A version number
//! - Flags controlling encoding behavior
//! - Children organized into groups (for nested blocks) or a single list
//!
//! # Compression
//!
//! Large ESF files (particularly campaign startpos files) may contain compressed sections.
//! Nodes with specific names (e.g., `CAMPAIGN_ENV`) are automatically compressed using LZMA1
//! during encoding. The compressed data is stored in special `COMPRESSED_DATA` and
//! `COMPRESSED_DATA_INFO` record nodes.
//!
//! During decoding, these compressed sections are automatically decompressed and the
//! contained ESF data replaces the outer structure.
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::esf::ESF;
//! use rpfm_lib::files::{Decodeable, Encodeable};
//! use std::io::Cursor;
//!
//! // Decode an ESF file
//! let mut reader = Cursor::new(esf_bytes);
//! let esf = ESF::decode(&mut reader, &None)?;
//!
//! // Access the root node and traverse the tree
//! let root = esf.root_node();
//!
//! // Encode back to bytes
//! let mut output = Vec::new();
//! esf.encode(&mut output, &None)?;
//! ```
//!
//! # Internal Submodules
//!
//! - `caab`: CAAB format-specific reading and writing logic
//! - `cbab`: CBAB format-specific reading and writing logic
//! - `utils`: Shared utilities for node reading/writing across formats

use bitflags::bitflags;
use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::{fmt, fmt::Display};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};

/// File extensions associated with ESF files.
///
/// These extensions are used to identify files that should be parsed as ESF format.
pub const EXTENSIONS: [&str; 6] = [
    ".csc",                 // Composite Scene files
    ".ccd",                 // CEO (Campaign Effect Object) files
    ".esf",                 // Generic ESF data files
    ".save",                // Single-player save game files
    ".save_multiplayer",    // Multiplayer save game files
    ".twc",                 // Total War Character save files
];

/// Magic bytes identifying the CAAB ESF format (older format with u16 string sizes).
pub const SIGNATURE_CAAB: &[u8; 4] = &[0xCA, 0xAB, 0x00, 0x00];

/// Magic bytes identifying the CBAB ESF format (newer format with u32 string sizes).
pub const SIGNATURE_CBAB: &[u8; 4] = &[0xCB, 0xAB, 0x00, 0x00];

/// Magic bytes identifying the CEAB ESF format (unsupported).
pub const SIGNATURE_CEAB: &[u8; 4] = &[0xCE, 0xAB, 0x00, 0x00];

/// Magic bytes identifying the CFAB ESF format (unsupported).
pub const SIGNATURE_CFAB: &[u8; 4] = &[0xCF, 0xAB, 0x00, 0x00];

mod caab;
mod cbab;
mod utils;

//#[cfg(test)] mod esf_test;

//---------------------------------------------------------------------------//
//                              Node Type Markers
//---------------------------------------------------------------------------//
//
// These byte markers identify the type of each node in the ESF binary format.
// The marker system was originally documented in ESFEdit.
//
// Markers are organized into ranges:
// - 0x00:        Invalid/reserved
// - 0x01-0x10:   Primitive types (bool, integers, floats, strings, etc.)
// - 0x12-0x1d:   Optimized primitives (compact encodings for common values)
// - 0x21-0x26:   Unknown/undocumented types
// - 0x41-0x50:   Arrays of primitive types
// - 0x52-0x5d:   Arrays with optimized element encodings
// - 0x80+:       Record nodes (high bit set indicates record type)

/// Invalid marker - encountering this during parsing is always an error.
pub const INVALID: u8 = 0x00;

// Primitive type markers (0x01-0x10)
const BOOL: u8 = 0x01;      // Boolean value (1 byte: 0 or 1)
const I8: u8 = 0x02;        // Signed 8-bit integer
const I16: u8 = 0x03;       // Signed 16-bit integer (little-endian)
const I32: u8 = 0x04;       // Signed 32-bit integer (little-endian)
const I64: u8 = 0x05;       // Signed 64-bit integer (little-endian)
const U8: u8 = 0x06;        // Unsigned 8-bit integer
const U16: u8 = 0x07;       // Unsigned 16-bit integer (little-endian)
const U32: u8 = 0x08;       // Unsigned 32-bit integer (little-endian)
const U64: u8 = 0x09;       // Unsigned 64-bit integer (little-endian)
const F32: u8 = 0x0a;       // 32-bit floating point (IEEE 754)
const F64: u8 = 0x0b;       // 64-bit floating point (IEEE 754)
const COORD_2D: u8 = 0x0c;  // 2D coordinate (two f32 values: x, y)
const COORD_3D: u8 = 0x0d;  // 3D coordinate (three f32 values: x, y, z)
const UTF16: u8 = 0x0e;     // UTF-16 string (stored as index into string table)
const ASCII: u8 = 0x0f;     // ASCII/UTF-8 string (stored as index into string table)
const ANGLE: u8 = 0x10;     // Angle value (i16, likely representing degrees or radians scaled)

// Optimized primitive markers (0x12-0x1d)
// These provide compact encodings for common values to reduce file size.
const BOOL_TRUE: u8 = 0x12;     // Boolean true (no additional bytes needed)
const BOOL_FALSE: u8 = 0x13;    // Boolean false (no additional bytes needed)
const U32_ZERO: u8 = 0x14;      // u32 value of 0 (no additional bytes)
const U32_ONE: u8 = 0x15;       // u32 value of 1 (no additional bytes)
const U32_BYTE: u8 = 0x16;      // u32 stored as single byte (0-255)
const U32_16BIT: u8 = 0x17;     // u32 stored as 2 bytes (0-65535)
const U32_24BIT: u8 = 0x18;     // u32 stored as 3 bytes (0-16777215)
const I32_ZERO: u8 = 0x19;      // i32 value of 0 (no additional bytes)
const I32_BYTE: u8 = 0x1a;      // i32 stored as single signed byte (-128 to 127)
const I32_16BIT: u8 = 0x1b;     // i32 stored as 2 bytes (-32768 to 32767)
const I32_24BIT: u8 = 0x1c;     // i32 stored as 3 bytes (-8388608 to 8388607)
const F32_ZERO: u8 = 0x1d;      // f32 value of 0.0 (no additional bytes)

// Unknown/undocumented type markers (0x21-0x26)
// These types exist in some ESF files but their exact purpose is unclear.
const UNKNOWN_21: u8 = 0x21;    // Unknown type, stores u32
const UNKNOWN_23: u8 = 0x23;    // Unknown type, stores u8
const UNKNOWN_24: u8 = 0x24;    // Unknown type, stores u16
const UNKNOWN_25: u8 = 0x25;    // Unknown type, stores u32

// Three Kingdoms DLC "Eight Princes" introduced this type
const UNKNOWN_26: u8 = 0x26;    // Variable-length unknown type with special encoding

// Array type markers (0x41-0x50)
// Arrays store multiple values of the same type with a CAULEB128-encoded length prefix.
const BOOL_ARRAY: u8 = 0x41;
const I8_ARRAY: u8 = 0x42;
const I16_ARRAY: u8 = 0x43;
const I32_ARRAY: u8 = 0x44;
const I64_ARRAY: u8 = 0x45;
const U8_ARRAY: u8 = 0x46;
const U16_ARRAY: u8 = 0x47;
const U32_ARRAY: u8 = 0x48;
const U64_ARRAY: u8 = 0x49;
const F32_ARRAY: u8 = 0x4a;
const F64_ARRAY: u8 = 0x4b;
const COORD_2D_ARRAY: u8 = 0x4c;
const COORD_3D_ARRAY: u8 = 0x4d;
const UTF16_ARRAY: u8 = 0x4e;
const ASCII_ARRAY: u8 = 0x4f;
const ANGLE_ARRAY: u8 = 0x50;

// Optimized array markers (0x52-0x5d)
// Arrays where elements use compact encodings. Some markers exist in the format
// but don't make practical sense (e.g., array of all zeros/ones).
const BOOL_TRUE_ARRAY: u8 = 0x52;   // Unused - array of all true makes no sense
const BOOL_FALSE_ARRAY: u8 = 0x53;  // Unused - array of all false makes no sense
const U32_ZERO_ARRAY: u8 = 0x54;    // Unused - array of all zeros makes no sense
const U32_ONE_ARRAY: u8 = 0x55;     // Unused - array of all ones makes no sense
const U32_BYTE_ARRAY: u8 = 0x56;    // u32 array with each element stored as 1 byte
const U32_16BIT_ARRAY: u8 = 0x57;   // u32 array with each element stored as 2 bytes
const U32_24BIT_ARRAY: u8 = 0x58;   // u32 array with each element stored as 3 bytes
const I32_ZERO_ARRAY: u8 = 0x59;    // Unused - array of all zeros makes no sense
const I32_BYTE_ARRAY: u8 = 0x5a;    // i32 array with each element stored as 1 byte
const I32_16BIT_ARRAY: u8 = 0x5b;   // i32 array with each element stored as 2 bytes
const I32_24BIT_ARRAY: u8 = 0x5c;   // i32 array with each element stored as 3 bytes
const F32_ZERO_ARRAY: u8 = 0x5d;    // Unused - array of all zeros makes no sense

// Compression-related constants
// Large nodes (e.g., campaign environment data) may be LZMA1-compressed.
/// Record names that trigger automatic compression during encoding.
const COMPRESSED_TAGS: [&str; 1] = ["CAMPAIGN_ENV"];
/// Name of the record node containing compressed data bytes.
const COMPRESSED_DATA_TAG: &str = "COMPRESSED_DATA";
/// Name of the record node containing compression metadata (uncompressed size + LZMA header).
const COMPRESSED_DATA_INFO_TAG: &str = "COMPRESSED_DATA_INFO";

// Record nodes use the high bit of their type byte to indicate they are records,
// with additional flag bits controlling encoding behavior.
bitflags! {

    /// Flags that control how a record node is encoded in the ESF binary format.
    ///
    /// These flags are stored in the high bits of the type byte for record nodes.
    /// The combination of flags determines how the node header and children are encoded.
    ///
    /// # Binary Layout
    ///
    /// For a record node's first byte:
    /// - Bit 7 (0x80): Always set for record nodes (`IS_RECORD_NODE`)
    /// - Bit 6 (0x40): Set if node has nested blocks (`HAS_NESTED_BLOCKS`)
    /// - Bit 5 (0x20): Set if using non-optimized header format (`HAS_NON_OPTIMIZED_INFO`)
    /// - Bits 0-4: Used for optimized encoding of version/name when bit 5 is clear
    #[derive(PartialEq, Clone, Copy, Default, Debug, Serialize, Deserialize)]
    pub struct RecordNodeFlags: u8 {

        /// Indicates this is a record node (container with children).
        /// This flag is always set for record nodes and distinguishes them from primitive nodes.
        const IS_RECORD_NODE            = 0b1000_0000;

        /// Indicates the record contains multiple groups of children (nested blocks).
        /// When set, children are organized into separate groups, each with its own size prefix.
        /// When clear, all children are in a single flat list.
        const HAS_NESTED_BLOCKS         = 0b0100_0000;

        /// Indicates the node uses the full 3-byte header format (u16 name index + u8 version).
        /// When clear, the header is compressed into 2 bytes using bitwise encoding:
        /// - Bits 1-4: Version (4 bits, max 15)
        /// - Bit 0 + next byte: Name index (9 bits, max 511)
        const HAS_NON_OPTIMIZED_INFO    = 0b0010_0000;
    }
}

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a complete ESF file decoded in memory.
///
/// An ESF file contains a tree of nodes starting from a single root node. The root node
/// is always a record node that contains all other data in the file.
///
/// # Fields
///
/// - `signature`: Identifies the format version (CAAB, CBAB, etc.)
/// - `unknown_1`: Purpose unknown, typically 0
/// - `creation_date`: Unix timestamp or similar date value
/// - `root_node`: The top-level record node containing all file data
#[derive(Getters, Setters, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[getset(get = "pub", set = "pub")]
pub struct ESF {

    /// Format signature identifying the ESF version (CAAB, CBAB, etc.).
    signature: ESFSignature,

    /// Unknown header field, typically 0. May be reserved for future use.
    unknown_1: u32,

    /// Creation timestamp of the ESF file.
    creation_date: u32,

    /// Root node of the node tree containing all ESF data.
    /// This is always a record node in valid ESF files.
    root_node: NodeType,
}

/// Identifies the format version of an ESF file.
///
/// Different signatures indicate different encoding rules, particularly for
/// string size prefixes and potentially other format variations.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum ESFSignature {
    /// CAAB format - older version using u16 for string size prefixes.
    #[default]
    CAAB,
    /// CBAB format - newer version using u32 for string size prefixes.
    CBAB,
    /// CEAB format - unsupported, rarely encountered.
    CEAB,
    /// CFAB format - unsupported, rarely encountered.
    CFAB
}

/// Represents all possible node types in an ESF file.
///
/// ESF files use a tagged union approach where each node in the binary format
/// starts with a type marker byte that identifies what kind of data follows.
/// This enum mirrors that structure.
///
/// # Categories
///
/// - **Primitive nodes**: Single values (bool, integers, floats, strings)
/// - **Optimized nodes**: Primitives with compact encoding tracking (`BoolNode`, `I32Node`, etc.)
/// - **Complex nodes**: Structured data like coordinates
/// - **Array nodes**: Collections of same-typed values
/// - **Record nodes**: Container nodes with named children
/// - **Unknown nodes**: Undocumented types preserved for round-trip fidelity
///
/// Note: Some type information was originally extracted from ESFEdit.
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub enum NodeType {

    /// Invalid/uninitialized node. Used as a placeholder; encountering this during
    /// parsing or encoding indicates an error.
    #[default]
    Invalid,

    // Primitive nodes
    /// Boolean value with optimization tracking.
    Bool(BoolNode),
    /// Signed 8-bit integer.
    I8(i8),
    /// Signed 16-bit integer.
    I16(i16),
    /// Signed 32-bit integer with optimization tracking.
    I32(I32Node),
    /// Signed 64-bit integer.
    I64(i64),
    /// Unsigned 8-bit integer.
    U8(u8),
    /// Unsigned 16-bit integer.
    U16(u16),
    /// Unsigned 32-bit integer with optimization tracking.
    U32(U32Node),
    /// Unsigned 64-bit integer.
    U64(u64),
    /// 32-bit floating point with optimization tracking.
    F32(F32Node),
    /// 64-bit floating point.
    F64(f64),
    /// 2D coordinate (x, y as f32).
    Coord2d(Coordinates2DNode),
    /// 3D coordinate (x, y, z as f32).
    Coord3d(Coordinates3DNode),
    /// UTF-16 encoded string (stored as index, resolved during decode).
    Utf16(String),
    /// ASCII/UTF-8 encoded string (stored as index, resolved during decode).
    Ascii(String),
    /// Angle value stored as i16.
    Angle(i16),

    // Unknown/undocumented types - preserved for round-trip encoding fidelity
    /// Unknown type 0x21 storing a u32 value.
    Unknown21(u32),
    /// Unknown type 0x23 storing a u8 value.
    Unknown23(u8),
    /// Unknown type 0x24 storing a u16 value.
    Unknown24(u16),
    /// Unknown type 0x25 storing a u32 value.
    Unknown25(u32),
    /// Unknown type 0x26 with variable-length data (Three Kingdoms DLC).
    Unknown26(Vec<u8>),

    // Array types
    /// Array of boolean values.
    BoolArray(Vec<bool>),
    /// Array of i8 values (stored as u8 for efficiency).
    I8Array(Vec<u8>),
    /// Array of i16 values.
    I16Array(Vec<i16>),
    /// Array of i32 values with optimization tracking.
    I32Array(VecI32Node),
    /// Array of i64 values.
    I64Array(Vec<i64>),
    /// Array of u8 values (raw bytes).
    U8Array(Vec<u8>),
    /// Array of u16 values.
    U16Array(Vec<u16>),
    /// Array of u32 values with optimization tracking.
    U32Array(VecU32Node),
    /// Array of u64 values.
    U64Array(Vec<u64>),
    /// Array of f32 values.
    F32Array(Vec<f32>),
    /// Array of f64 values.
    F64Array(Vec<f64>),
    /// Array of 2D coordinates.
    Coord2dArray(Vec<Coordinates2DNode>),
    /// Array of 3D coordinates.
    Coord3dArray(Vec<Coordinates3DNode>),
    /// Array of UTF-16 strings.
    Utf16Array(Vec<String>),
    /// Array of ASCII strings.
    AsciiArray(Vec<String>),
    /// Array of angle values.
    AngleArray(Vec<i16>),

    /// Record node - a named container holding child nodes.
    /// Records form the hierarchical structure of ESF files.
    Record(Box<RecordNode>),
}

/// Boolean node with optimization tracking.
///
/// When `optimized` is true, the value is encoded using `BOOL_TRUE` (0x12) or
/// `BOOL_FALSE` (0x13) markers which require no additional bytes. When false,
/// uses the standard `BOOL` (0x01) marker followed by a byte.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct BoolNode {
    /// The boolean value.
    value: bool,
    /// Whether to use optimized encoding (single marker byte, no value byte).
    optimized: bool,
}

/// Signed 32-bit integer node with optimization tracking.
///
/// When `optimized` is true, the encoder selects the smallest encoding that fits:
/// - `I32_ZERO` (0x19): Value is 0, no additional bytes
/// - `I32_BYTE` (0x1a): Value fits in i8 (-128 to 127), 1 byte
/// - `I32_16BIT` (0x1b): Value fits in i16 (-32768 to 32767), 2 bytes
/// - `I32_24BIT` (0x1c): Value fits in 24 bits, 3 bytes
/// - `I32` (0x04): Full 4-byte encoding
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct I32Node {
    /// The signed 32-bit integer value.
    value: i32,
    /// Whether to use optimized (variable-width) encoding.
    optimized: bool,
}

/// Unsigned 32-bit integer node with optimization tracking.
///
/// When `optimized` is true, the encoder selects the smallest encoding that fits:
/// - `U32_ZERO` (0x14): Value is 0, no additional bytes
/// - `U32_ONE` (0x15): Value is 1, no additional bytes
/// - `U32_BYTE` (0x16): Value fits in u8 (0-255), 1 byte
/// - `U32_16BIT` (0x17): Value fits in u16 (0-65535), 2 bytes
/// - `U32_24BIT` (0x18): Value fits in 24 bits (0-16777215), 3 bytes
/// - `U32` (0x08): Full 4-byte encoding
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct U32Node {
    /// The unsigned 32-bit integer value.
    value: u32,
    /// Whether to use optimized (variable-width) encoding.
    optimized: bool,
}

/// 32-bit floating point node with optimization tracking.
///
/// When `optimized` is true and the value is exactly 0.0, uses `F32_ZERO` (0x1d)
/// which requires no additional bytes. Otherwise uses standard `F32` (0x0a) encoding.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct F32Node {
    /// The 32-bit floating point value.
    value: f32,
    /// Whether to use optimized encoding (zero detection).
    optimized: bool,
}

/// Array of signed 32-bit integers with optimization tracking.
///
/// When `optimized` is true, the encoder analyzes all values to find the smallest
/// encoding that fits all elements, using byte/16-bit/24-bit arrays when possible.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VecI32Node {
    /// The array of i32 values.
    value: Vec<i32>,
    /// Whether to use optimized (smaller element size) encoding when possible.
    optimized: bool,
}

/// Array of unsigned 32-bit integers with optimization tracking.
///
/// When `optimized` is true, the encoder analyzes all values to find the smallest
/// encoding that fits all elements, using byte/16-bit/24-bit arrays when possible.
#[derive(Getters, MutGetters, Setters, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct VecU32Node {
    /// The array of u32 values.
    value: Vec<u32>,
    /// Whether to use optimized (smaller element size) encoding when possible.
    optimized: bool,
}

/// 2D coordinate node storing X and Y position values.
///
/// Commonly used for map positions, UI coordinates, and other 2D spatial data.
/// Each coordinate is stored as a 32-bit float.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Coordinates2DNode {
    /// X coordinate.
    x: f32,
    /// Y coordinate.
    y: f32,
}

/// 3D coordinate node storing X, Y, and Z position values.
///
/// Commonly used for world positions, unit locations, and other 3D spatial data.
/// Each coordinate is stored as a 32-bit float.
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Coordinates3DNode {
    /// X coordinate.
    x: f32,
    /// Y coordinate.
    y: f32,
    /// Z coordinate.
    z: f32,
}

/// A record node that contains other nodes as children.
///
/// Record nodes are the structural backbone of ESF files, organizing data into
/// a hierarchical tree. Each record has a name (referencing the string table),
/// a version number, and zero or more child nodes.
///
/// # Children Organization
///
/// Children can be organized in two ways depending on the `HAS_NESTED_BLOCKS` flag:
/// - **Without nested blocks**: Single flat list of children (`children[0]`)
/// - **With nested blocks**: Multiple groups of children, each group representing
///   a logical grouping (e.g., multiple instances of the same record type)
///
/// # Example Structure
///
/// A typical ESF might have a structure like:
/// ```text
/// ROOT
/// ├── CAMPAIGN_SAVE_GAME (record)
/// │   ├── FACTION (record, nested blocks for multiple factions)
/// │   │   ├── [Group 0: Faction 1 data...]
/// │   │   └── [Group 1: Faction 2 data...]
/// │   └── DATE (record)
/// │       ├── year (u32)
/// │       └── turn (u32)
/// ```
#[derive(Getters, MutGetters, Setters, PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RecordNode {

    /// Encoding flags controlling how this record is serialized.
    record_flags: RecordNodeFlags,

    /// Version number of this record's schema (0-255).
    /// Used by the game to handle format changes across patches.
    version: u8,

    /// Name of the record (e.g., "FACTION", "CAMPAIGN_SAVE_GAME").
    /// This is resolved from the record names string table during decode.
    name: String,

    /// Child nodes organized into groups.
    /// - Without `HAS_NESTED_BLOCKS`: Single group at index 0
    /// - With `HAS_NESTED_BLOCKS`: Multiple groups, each a separate logical unit
    children: Vec<Vec<NodeType>>
}

//---------------------------------------------------------------------------//
//                           Implementation of ESF
//---------------------------------------------------------------------------//

impl ESF {

    /// Creates a shallow copy of this ESF with the root node replaced by `Invalid`.
    ///
    /// This is useful when you need to preserve the header metadata (signature,
    /// creation date, etc.) but want to rebuild or replace the node tree.
    pub fn clone_without_root_node(&self) -> Self {
        Self {
            signature: self.signature,
            unknown_1: self.unknown_1,
            creation_date: self.creation_date,
            root_node: NodeType::Invalid,
        }
    }
}

impl NodeType {

    /// Creates a copy of a node without its children (for record nodes).
    ///
    /// For record nodes, this creates a new record with the same name, flags,
    /// and version but with an empty children list. For all other node types,
    /// this is equivalent to `clone()`.
    ///
    /// Useful for building modified node trees while preserving the structure.
    pub fn clone_without_children(&self) -> Self {
        match self {
            Self::Record(node) => {
                let mut new_node = RecordNode::default();
                new_node.set_name(node.name().to_owned());
                new_node.set_record_flags(*node.record_flags());
                new_node.set_version(*node.version());

                Self::Record(Box::new(new_node))
            }

            _ => self.clone()
        }
    }
}

impl Display for ESFSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::CAAB => "CAAB",
            Self::CBAB => "CBAB",
            Self::CEAB => "CEAB",
            Self::CFAB => "CFAB",
        }, f)
    }
}

impl TryFrom<&str> for ESFSignature {
    type Error = RLibError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "CAAB" => Ok(Self::CAAB),
            "CBAB" => Ok(Self::CBAB),
            "CEAB" => Ok(Self::CEAB),
            "CFAB" => Ok(Self::CFAB),
            _ => Err(RLibError::UnknownESFSignature(value.to_string())),
        }
    }
}

impl TryFrom<Vec<u8>> for ESFSignature {
    type Error = RLibError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match value.as_slice().try_into()? {
            SIGNATURE_CAAB => Ok(Self::CAAB),
            SIGNATURE_CBAB => Ok(Self::CBAB),
            SIGNATURE_CEAB => Ok(Self::CEAB),
            SIGNATURE_CFAB => Ok(Self::CFAB),
            _ => Err(RLibError::UnknownESFSignatureBytes(value[0], value[1])),
        }
    }
}

impl From<ESFSignature> for Vec<u8> {
    fn from(value: ESFSignature) -> Self {
        match value {
            ESFSignature::CAAB => SIGNATURE_CAAB.to_vec(),
            ESFSignature::CBAB => SIGNATURE_CBAB.to_vec(),
            ESFSignature::CEAB => SIGNATURE_CEAB.to_vec(),
            ESFSignature::CFAB => SIGNATURE_CFAB.to_vec(),
        }
    }
}

impl Decodeable for ESF {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut esf = Self::default();

        let sig_bytes = data.read_slice(4, false)?;
        esf.signature = ESFSignature::try_from(sig_bytes.to_vec())?;

        match esf.signature {
            ESFSignature::CAAB => Self::read_caab(&mut esf, data)?,
            ESFSignature::CBAB => Self::read_cbab(&mut esf, data)?,
            _ => return Err(RLibError::DecodingESFUnsupportedSignature(sig_bytes[0], sig_bytes[1])),
        };

        // Debugging code.
        //use std::io::Write;
        //let mut x = std::fs::File::create("ceo.json")?;
        //x.write_all(&serde_json::to_string_pretty(&esf).unwrap().as_bytes())?;

        Ok(esf)
    }
}

impl Encodeable for ESF {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        let sig_bytes: Vec<u8> = Vec::from(self.signature);
        buffer.write_all(&sig_bytes)?;

        match self.signature {
            ESFSignature::CAAB => self.save_caab(buffer, extra_data),
            ESFSignature::CBAB => self.save_cbab(buffer, extra_data),
            _ => Err(RLibError::EncodingESFUnsupportedSignature(self.signature.to_string())),
        }
    }
}
