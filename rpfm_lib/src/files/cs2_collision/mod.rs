//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! CS2 Collision file format support.
//!
//! CS2 Collision files (`.cs2.collision`) define collision meshes for 3D models in
//! Total War games. These files contain triangular mesh data used for physics
//! collision detection and pathfinding.
//!
//! # File Format
//!
//! CS2 Collision files are binary files containing:
//! - Magic number and version header
//! - Overall bounding box
//! - One or more named collision meshes with vertices and triangles
//! - Triangle adjacency information for efficient collision queries
//!
//! # Supported Versions
//!
//! - **Version 0**
//! - **Version 11**
//! - **Version 13**
//! - **Version 20**
//! - **Version 21**
//!
//! # File Contents
//!
//! - **Bounding Box**: Overall bounds containing all collision meshes
//! - **Collision3d Objects**: Named collision meshes with triangle data
//! - **Triangle Adjacency**: Edge connectivity for fast neighbor queries
//!
//! # Usage
//!
//! ```rust,ignore
//! use rpfm_lib::files::cs2_collision::Cs2Collision;
//! use rpfm_lib::files::Decodeable;
//!
//! // Decode from binary data
//! let collision = Cs2Collision::decode(&mut data, &None)?;
//!
//! // Access collision meshes
//! for mesh in collision.collisions_3d() {
//!     println!("Mesh: {} ({} vertices, {} triangles)",
//!         mesh.name(),
//!         mesh.vertices().len(),
//!         mesh.triangles().len()
//!     );
//! }
//! ```
//!
//! # File Location
//!
//! These files are typically found at:
//! - `rigidmodels/buildings/*/*.cs2.collision`

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::error::{Result, RLibError};
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::files::bmd::common::*;
use crate::utils::check_size_mismatch;

/// File extension for CS2 Collision files.
pub const EXTENSION: &str = ".cs2.collision";

#[cfg(test)] mod cs2_collision_test;

mod versions;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Represents a CS2 Collision file decoded in memory.
///
/// Contains all collision mesh data for a 3D model, including vertices,
/// triangles, and adjacency information for efficient collision detection.
///
/// # Fields
///
/// * `magic_number` - File format identifier
/// * `version` - File format version (0, 11, 13, 20, or 21)
/// * `bounding_box` - Overall bounds containing all collision meshes
/// * `collisions_3d` - List of named collision meshes
///
/// # Examples
///
/// ```rust,ignore
/// let collision = Cs2Collision::decode(&mut data, &None)?;
/// println!("Version: {}", collision.version());
/// println!("Meshes: {}", collision.collisions_3d().len());
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct Cs2Collision {
    /// File format identifier.
    magic_number: u32,

    /// File format version number.
    version: u32,

    /// Bounding box containing all collision meshes.
    bounding_box: Cube,

    /// List of collision meshes in this file.
    collisions_3d: Vec<Collision3d>,
}

/// A named 3D collision mesh with vertices and triangles.
///
/// Represents a single collision mesh within a CS2 Collision file. Each mesh
/// has a name, vertex list, and triangle list with adjacency information.
///
/// # Fields
///
/// * `name` - Name identifier for this collision mesh (UTF-8)
/// * `uk_1` - Unknown field (possibly an ID)
/// * `uk_2` - Unknown field (appears to be 0 or 1)
/// * `vertices` - List of 3D vertex positions
/// * `triangles` - List of triangles with adjacency data
/// * `zero_4` - Reserved field (always 0)
/// * `bounding_box` - Bounding box for this mesh
///
/// # Examples
///
/// ```rust,ignore
/// for mesh in collision.collisions_3d() {
///     println!("Mesh '{}' has {} triangles", mesh.name(), mesh.triangles().len());
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct Collision3d {
    /// Name identifier for this collision mesh.
    name: String,

    /// Unknown field (possibly an ID).
    uk_1: i32,

    /// Unknown field (appears to be 0 or 1).
    uk_2: i32,

    /// List of vertex positions for this mesh.
    vertices: Vec<Point3d>,

    /// List of triangles with adjacency information.
    triangles: Vec<CollisionTriangle>,

    /// Reserved field (always 0).
    zero_4: i32,

    /// Bounding box for this collision mesh.
    bounding_box: Cube,
}

/// A collision triangle with vertex indices and edge adjacency information.
///
/// Represents a single triangle in the collision mesh along with its three edges
/// and information about adjacent triangles. This adjacency data enables efficient
/// collision detection and mesh traversal.
///
/// # Triangle Structure
///
/// Each triangle has:
/// - 3 vertex indices defining the triangle face
/// - 3 edges, each with adjacency to neighboring triangles
/// - Face index for identification
///
/// # Edge Adjacency
///
/// For each edge (1, 2, 3):
/// - Two vertex indices defining the edge
/// - Face index (redundant, same as main face_index)
/// - Across face index (index of triangle sharing this edge, or -1 if boundary)
///
/// # Fields
///
/// ## Triangle Face
/// * `face_index` - Index identifier for this triangle
/// * `padding` - Alignment padding (always 0)
/// * `vertex_1`, `vertex_2`, `vertex_3` - Indices into the vertex array
///
/// ## Edge 1
/// * `edge_1_vertex_1`, `edge_1_vertex_2` - Vertex indices for edge 1
/// * `face_index_1` - Face index (same as face_index)
/// * `zero_1` - Reserved (always 0)
/// * `across_face_index_1` - Index of adjacent triangle across edge 1 (-1 if none)
///
/// ## Edge 2
/// * `edge_2_vertex_1`, `edge_2_vertex_2` - Vertex indices for edge 2
/// * `face_index_2` - Face index (same as face_index)
/// * `zero_2` - Reserved (always 0)
/// * `across_face_index_2` - Index of adjacent triangle across edge 2 (-1 if none)
///
/// ## Edge 3
/// * `edge_3_vertex_1`, `edge_3_vertex_2` - Vertex indices for edge 3
/// * `face_index_3` - Face index (same as face_index)
/// * `zero_3` - Reserved (always 0)
/// * `across_face_index_3` - Index of adjacent triangle across edge 3 (-1 if none)
///
/// ## Terminator
/// * `zero_4` - Reserved field (always 0)
///
/// # Examples
///
/// ```rust,ignore
/// for triangle in mesh.triangles() {
///     // Check if triangle has neighbors on all edges
///     let has_neighbor_1 = triangle.across_face_index_1() >= 0;
///     let has_neighbor_2 = triangle.across_face_index_2() >= 0;
///     let has_neighbor_3 = triangle.across_face_index_3() >= 0;
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub(crate)")]
pub struct CollisionTriangle {
    /// Index identifier for this triangle.
    face_index: i32,

    /// Alignment padding (always 0).
    padding: i8,

    /// First vertex index.
    vertex_1: i32,

    /// Second vertex index.
    vertex_2: i32,

    /// Third vertex index.
    vertex_3: i32,

    /// First vertex of edge 1.
    edge_1_vertex_1: i32,

    /// Second vertex of edge 1.
    edge_1_vertex_2: i32,

    /// Face index for edge 1 (same as face_index).
    face_index_1: i32,

    /// Reserved (always 0).
    zero_1: i32,

    /// Index of triangle adjacent across edge 1 (-1 if none).
    across_face_index_1: i32,

    /// First vertex of edge 2.
    edge_2_vertex_1: i32,

    /// Second vertex of edge 2.
    edge_2_vertex_2: i32,

    /// Face index for edge 2 (same as face_index).
    face_index_2: i32,

    /// Reserved (always 0).
    zero_2: i32,

    /// Index of triangle adjacent across edge 2 (-1 if none).
    across_face_index_2: i32,

    /// First vertex of edge 3.
    edge_3_vertex_1: i32,

    /// Second vertex of edge 3.
    edge_3_vertex_2: i32,

    /// Face index for edge 3 (same as face_index).
    face_index_3: i32,

    /// Reserved (always 0).
    zero_3: i32,

    /// Index of triangle adjacent across edge 3 (-1 if none).
    across_face_index_3: i32,

    /// Reserved terminator field (always 0).
    zero_4: i32,
}

//---------------------------------------------------------------------------//
//                           Implementation of Cs2Collision
//---------------------------------------------------------------------------//

impl Decodeable for Cs2Collision {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();
        decoded.magic_number = data.read_u32()?;
        decoded.version = data.read_u32()?;

        match decoded.version {
            21 => decoded.read_v21(data)?,
            20 => decoded.read_v20(data)?,
             _ => return Err(RLibError::DecodingUnsupportedVersion(decoded.version as usize)),
        }

        // Trigger an error if there's left data on the source.
        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(decoded)
    }
}

impl Encodeable for Cs2Collision {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.magic_number)?;
        buffer.write_u32(self.version)?;

        match self.version {
            21 => self.write_v21(buffer)?,
            20 => self.write_v20(buffer)?,
            _ => unimplemented!()
        }


        Ok(())
    }
}
