//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Common data structures shared across BMD format files.
//!
//! This module provides reusable geometric and transformation primitives used throughout
//! BMD (Battle Map Definition) files and related formats. These structures are public to
//! allow reuse in other file format modules.
//!
//! # Geometric Primitives
//!
//! ## Points
//! - [`Point2d`] - 2D point (x, y)
//! - [`Point3d`] - 3D point (x, y, z)
//!
//! ## Shapes
//! - [`Rectangle`] - 2D axis-aligned bounding box
//! - [`Cube`] - 3D axis-aligned bounding box
//! - [`Outline2d`] - 2D polyline outline
//! - [`Outline3d`] - 3D polyline outline
//! - [`Polygon2d`] - 2D polygon with arbitrary vertices
//!
//! ## Colors
//! - [`ColourRGB`] - RGB color (floating-point components)
//! - [`ColourRGBA`] - RGBA color (8-bit components)
//!
//! ## Transformations
//! - [`Transform3x4`] - 3x4 transformation matrix (rotation + translation)
//! - [`Transform4x4`] - 4x4 transformation matrix (full affine transform)
//! - [`Quaternion`] - Rotation quaternion (i, j, k, w)
//! - [`Matrix`] - Trait for matrix operations and conversions
//!
//! # Matrix Trait
//!
//! The [`Matrix`] trait provides common operations for transformation matrices:
//! - Element accessors (`m00()`, `m01()`, etc.)
//! - Rotation matrix extraction
//! - Scale extraction and application
//! - Euler angle conversion
//! - Identity matrix creation
//!
//! # Usage
//!
//! ```ignore
//! use rpfm_lib::files::bmd::common::{Point3d, Transform4x4, Matrix};
//!
//! // Create a 3D point
//! let point = Point3d::new(10.0, 20.0, 30.0);
//!
//! // Create an identity transform
//! let transform = Transform4x4::identity();
//!
//! // Extract rotation angles
//! let rotation_matrix = transform.rotation_matrix();
//! let (x, y, z) = Transform4x4::rotation_matrix_to_euler_angles(rotation_matrix, true);
//! ```
//!
//! # Submodules
//!
//! - [`building_link`] - Building linkage data structures
//! - [`building_reference`] - Building reference data structures
//! - [`flags`] - Flag definitions
//! - [`properties`] - Property data structures

use getset::*;
use serde_derive::{Serialize, Deserialize};

use std::ops::Sub;

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

pub mod building_link;
pub mod building_reference;
pub mod flags;
pub mod properties;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// RGB color with floating-point components.
///
/// Used for lighting and material colors in BMD files. Each component is a 32-bit
/// floating-point value typically in the range [0.0, 1.0], though values outside
/// this range are supported for HDR lighting.
///
/// # Fields
///
/// - `r`: Red component
/// - `g`: Green component
/// - `b`: Blue component
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::ColourRGB;
///
/// let mut color = ColourRGB::default();
/// color.set_r(1.0);  // Full red
/// color.set_g(0.5);  // Half green
/// color.set_b(0.0);  // No blue
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ColourRGB {
    /// Red component (typically 0.0-1.0).
    r: f32,

    /// Green component (typically 0.0-1.0).
    g: f32,

    /// Blue component (typically 0.0-1.0).
    b: f32,
}

/// RGBA color with 8-bit components.
///
/// Used for color data requiring alpha (transparency) channel. Each component is
/// an unsigned 8-bit integer in the range [0, 255].
///
/// # Fields
///
/// - `r`: Red component (0-255)
/// - `g`: Green component (0-255)
/// - `b`: Blue component (0-255)
/// - `a`: Alpha (opacity) component (0-255, where 255 is fully opaque)
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::ColourRGBA;
///
/// let mut color = ColourRGBA::default();
/// color.set_r(255);  // Full red
/// color.set_g(128);  // Half green
/// color.set_b(0);    // No blue
/// color.set_a(255);  // Fully opaque
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct ColourRGBA {
    /// Red component (0-255).
    r: u8,

    /// Green component (0-255).
    g: u8,

    /// Blue component (0-255).
    b: u8,

    /// Alpha (opacity) component (0-255, where 255 is fully opaque).
    a: u8,
}

/// 3D axis-aligned bounding box (AABB).
///
/// Represents a rectangular volume aligned with coordinate axes, defined by minimum
/// and maximum corners. Used for spatial bounds, collision volumes, and culling.
///
/// # Fields
///
/// - `min_x`, `min_y`, `min_z`: Minimum corner coordinates
/// - `max_x`, `max_y`, `max_z`: Maximum corner coordinates
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::Cube;
///
/// let mut cube = Cube::default();
/// cube.set_min_x(-10.0);
/// cube.set_max_x(10.0);
/// // Creates a 20x20x20 cube centered at origin
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Cube {
    /// Minimum X coordinate.
    min_x: f32,

    /// Minimum Y coordinate.
    min_y: f32,

    /// Minimum Z coordinate.
    min_z: f32,

    /// Maximum X coordinate.
    max_x: f32,

    /// Maximum Y coordinate.
    max_y: f32,

    /// Maximum Z coordinate.
    max_z: f32,
}

/// 2D polyline outline.
///
/// Represents a sequence of connected 2D points forming an open or closed outline.
/// Used for area boundaries, deployment zones, and other 2D regions.
///
/// # Fields
///
/// - `outline`: Ordered list of 2D points
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Outline2d {
    /// Ordered list of 2D points forming the outline.
    outline: Vec<Point2d>,
}

/// 3D polyline outline.
///
/// Represents a sequence of connected 3D points forming an open or closed outline.
/// Used for 3D boundaries, paths, and spatial regions.
///
/// # Fields
///
/// - `outline`: Ordered list of 3D points
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Outline3d {
    /// Ordered list of 3D points forming the outline.
    outline: Vec<Point3d>,
}

/// 2D point in Cartesian coordinates.
///
/// Represents a position in 2D space. Used for map coordinates, UI positions,
/// and texture coordinates.
///
/// # Fields
///
/// - `x`: Horizontal coordinate
/// - `y`: Vertical coordinate
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Point2d {
    /// X (horizontal) coordinate.
    x: f32,

    /// Y (vertical) coordinate.
    y: f32,
}

/// 3D point in Cartesian coordinates.
///
/// Represents a position in 3D space.
///
/// # Fields
///
/// - `x`: X-axis coordinate
/// - `y`: Y-axis coordinate
/// - `z`: Z-axis coordinate
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::Point3d;
///
/// let p1 = Point3d::new(10.0, 20.0, 30.0);
/// let p2 = Point3d::new(5.0, 5.0, 5.0);
/// let diff = p1 - p2;  // Vector from p2 to p1
/// ```
#[derive(Default, PartialEq, Copy, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Point3d {
    /// X-axis coordinate.
    x: f32,

    /// Y-axis coordinate.
    y: f32,

    /// Z-axis coordinate.
    z: f32,
}

/// 2D polygon with arbitrary vertices.
///
/// Represents a closed 2D polygon defined by an ordered list of vertices.
/// Used for complex area definitions and spatial regions.
///
/// # Fields
///
/// - `points`: Ordered list of polygon vertices
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Polygon2d {
    /// Ordered list of polygon vertices.
    points: Vec<Point2d>
}

/// Rotation quaternion.
///
/// Represents a 3D rotation using quaternion representation (i, j, k, w).
/// Quaternions provide smooth interpolation and avoid gimbal lock.
///
/// # Fields
///
/// - `i`, `j`, `k`: Imaginary components
/// - `w`: Real (scalar) component
///
/// # Quaternion Format
///
/// Standard quaternion format: `q = w + xi + yj + zk`
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::Quaternion;
///
/// let mut quat = Quaternion::default();
/// quat.set_w(1.0);  // Identity rotation
/// ```
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Quaternion {
    /// Imaginary i component.
    i: f32,

    /// Imaginary j component.
    j: f32,

    /// Imaginary k component.
    k: f32,

    /// Real (scalar) w component.
    w: f32,
}

/// 2D axis-aligned rectangle.
///
/// Represents a rectangular area aligned with coordinate axes, defined by
/// minimum and maximum corner coordinates. Used for 2D bounds and regions.
///
/// # Fields
///
/// - `min_x`, `min_y`: Minimum corner coordinates
/// - `max_x`, `max_y`: Maximum corner coordinates
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Rectangle {
    /// Minimum X coordinate.
    min_x: f32,

    /// Minimum Y coordinate.
    min_y: f32,

    /// Maximum X coordinate.
    max_x: f32,

    /// Maximum Y coordinate.
    max_y: f32,
}

/// 3x4 transformation matrix.
///
/// Represents a 3D affine transformation with rotation and translation but no
/// perspective. The matrix is stored in column-major order and contains:
/// - 3x3 rotation/scale submatrix (top-left)
/// - 3x1 translation vector (bottom row)
///
/// # Matrix Layout
///
/// ```text
/// [ m00  m01  m02 ]
/// [ m10  m11  m12 ]
/// [ m20  m21  m22 ]
/// [ m30  m31  m32 ]
/// ```
///
/// # Note
///
/// This struct does not have automatic getters for matrix elements. Use the
/// [`Matrix`] trait methods (m00(), m01(), etc.) to access elements.
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::{Transform3x4, Matrix};
///
/// let transform = Transform3x4::identity();
/// let m00 = transform.m00();  // Access via Matrix trait
/// ```
#[derive(Default, PartialEq, Clone, Debug, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get_mut = "pub", set = "pub")]
pub struct Transform3x4{
    /// Matrix element at row 0, column 0.
    m00: f32,
    /// Matrix element at row 0, column 1.
    m01: f32,
    /// Matrix element at row 0, column 2.
    m02: f32,
    /// Matrix element at row 1, column 0.
    m10: f32,
    /// Matrix element at row 1, column 1.
    m11: f32,
    /// Matrix element at row 1, column 2.
    m12: f32,
    /// Matrix element at row 2, column 0.
    m20: f32,
    /// Matrix element at row 2, column 1.
    m21: f32,
    /// Matrix element at row 2, column 2.
    m22: f32,
    /// Matrix element at row 3, column 0 (translation X).
    m30: f32,
    /// Matrix element at row 3, column 1 (translation Y).
    m31: f32,
    /// Matrix element at row 3, column 2 (translation Z).
    m32: f32,
}

/// 4x4 transformation matrix.
///
/// Represents a full 3D affine transformation including rotation, scale,
/// translation, and perspective. The matrix is stored in column-major order.
///
/// # Matrix Layout
///
/// ```text
/// [ m00  m01  m02  m03 ]
/// [ m10  m11  m12  m13 ]
/// [ m20  m21  m22  m23 ]
/// [ m30  m31  m32  m33 ]
/// ```
///
/// # Note
///
/// This struct does not have automatic getters for matrix elements. Use the
/// [`Matrix`] trait methods (m00(), m01(), etc.) to access elements.
///
/// # Conversions
///
/// - Can be converted from/to [`Cube`] for bounding box storage
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::{Transform4x4, Matrix};
///
/// let transform = Transform4x4::identity();
/// let rotation = transform.rotation_matrix();
/// let (rx, ry, rz) = Transform4x4::rotation_matrix_to_euler_angles(rotation, true);
/// ```
#[derive(Default, PartialEq, Clone, Debug, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get_mut = "pub", set = "pub")]
pub struct Transform4x4 {
    /// Matrix element at row 0, column 0.
    m00: f32,
    /// Matrix element at row 0, column 1.
    m01: f32,
    /// Matrix element at row 0, column 2.
    m02: f32,
    /// Matrix element at row 0, column 3.
    m03: f32,
    /// Matrix element at row 1, column 0.
    m10: f32,
    /// Matrix element at row 1, column 1.
    m11: f32,
    /// Matrix element at row 1, column 2.
    m12: f32,
    /// Matrix element at row 1, column 3.
    m13: f32,
    /// Matrix element at row 2, column 0.
    m20: f32,
    /// Matrix element at row 2, column 1.
    m21: f32,
    /// Matrix element at row 2, column 2.
    m22: f32,
    /// Matrix element at row 2, column 3.
    m23: f32,
    /// Matrix element at row 3, column 0 (translation X).
    m30: f32,
    /// Matrix element at row 3, column 1 (translation Y).
    m31: f32,
    /// Matrix element at row 3, column 2 (translation Z).
    m32: f32,
    /// Matrix element at row 3, column 3 (homogeneous coordinate).
    m33: f32,
}

//---------------------------------------------------------------------------//
//                           Implementations
//---------------------------------------------------------------------------//

impl Decodeable for ColourRGB {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            r: data.read_f32()?,
            g: data.read_f32()?,
            b: data.read_f32()?,
        })
    }
}

impl Encodeable for ColourRGB {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.r)?;
        buffer.write_f32(self.g)?;
        buffer.write_f32(self.b)?;

        Ok(())
    }
}

impl Decodeable for ColourRGBA {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            r: data.read_u8()?,
            g: data.read_u8()?,
            b: data.read_u8()?,
            a: data.read_u8()?,
        })
    }
}

impl Encodeable for ColourRGBA {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u8(self.r)?;
        buffer.write_u8(self.g)?;
        buffer.write_u8(self.b)?;
        buffer.write_u8(self.a)?;

        Ok(())
    }
}

impl Decodeable for Cube {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            min_x: data.read_f32()?,
            min_y: data.read_f32()?,
            min_z: data.read_f32()?,
            max_x: data.read_f32()?,
            max_y: data.read_f32()?,
            max_z: data.read_f32()?,
        })
    }
}

impl Encodeable for Cube {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.min_x)?;
        buffer.write_f32(self.min_y)?;
        buffer.write_f32(self.min_z)?;
        buffer.write_f32(self.max_x)?;
        buffer.write_f32(self.max_y)?;
        buffer.write_f32(self.max_z)?;

        Ok(())
    }
}

impl Decodeable for Outline2d {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.outline.push(Point2d::decode(data, extra_data)?);
        }

        Ok(decoded)
    }
}

impl Encodeable for Outline2d {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.outline.len() as u32)?;

        for point in &mut self.outline {
            point.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}

impl Decodeable for Outline3d {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.outline.push(Point3d::decode(data, extra_data)?);
        }

        Ok(decoded)
    }
}

impl Encodeable for Outline3d {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.outline.len() as u32)?;

        for point in &mut self.outline {
            point.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}

impl Decodeable for Point2d {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            x: data.read_f32()?,
            y: data.read_f32()?,
        })
    }
}

impl Encodeable for Point2d {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.x)?;
        buffer.write_f32(self.y)?;

        Ok(())
    }
}

impl Decodeable for Point3d {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            x: data.read_f32()?,
            y: data.read_f32()?,
            z: data.read_f32()?,
        })
    }
}

impl Encodeable for Point3d {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.x)?;
        buffer.write_f32(self.y)?;
        buffer.write_f32(self.z)?;

        Ok(())
    }
}

impl Decodeable for Polygon2d {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.points.push(Point2d::decode(data, extra_data)?);
        }

        Ok(decoded)
    }
}

impl Encodeable for Polygon2d {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_u32(self.points.len() as u32)?;
        for point in &mut self.points {
            point.encode(buffer, extra_data)?;
        }

        Ok(())
    }
}

impl Decodeable for Quaternion {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            i: data.read_f32()?,
            j: data.read_f32()?,
            k: data.read_f32()?,
            w: data.read_f32()?,
        })
    }
}

impl Encodeable for Quaternion {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.i)?;
        buffer.write_f32(self.j)?;
        buffer.write_f32(self.k)?;
        buffer.write_f32(self.w)?;

        Ok(())
    }
}

impl Decodeable for Rectangle {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            min_x: data.read_f32()?,
            min_y: data.read_f32()?,
            max_x: data.read_f32()?,
            max_y: data.read_f32()?,
        })
    }
}

impl Encodeable for Rectangle {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.min_x)?;
        buffer.write_f32(self.min_y)?;
        buffer.write_f32(self.max_x)?;
        buffer.write_f32(self.max_y)?;

        Ok(())
    }
}

impl Decodeable for Transform3x4 {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            m00: data.read_f32()?,
            m01: data.read_f32()?,
            m02: data.read_f32()?,
            m10: data.read_f32()?,
            m11: data.read_f32()?,
            m12: data.read_f32()?,
            m20: data.read_f32()?,
            m21: data.read_f32()?,
            m22: data.read_f32()?,
            m30: data.read_f32()?,
            m31: data.read_f32()?,
            m32: data.read_f32()?,
        })
    }
}

impl Encodeable for Transform3x4 {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.m00)?;
        buffer.write_f32(self.m01)?;
        buffer.write_f32(self.m02)?;
        buffer.write_f32(self.m10)?;
        buffer.write_f32(self.m11)?;
        buffer.write_f32(self.m12)?;
        buffer.write_f32(self.m20)?;
        buffer.write_f32(self.m21)?;
        buffer.write_f32(self.m22)?;
        buffer.write_f32(self.m30)?;
        buffer.write_f32(self.m31)?;
        buffer.write_f32(self.m32)?;

        Ok(())
    }
}

impl Decodeable for Transform4x4 {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            m00: data.read_f32()?,
            m01: data.read_f32()?,
            m02: data.read_f32()?,
            m03: data.read_f32()?,
            m10: data.read_f32()?,
            m11: data.read_f32()?,
            m12: data.read_f32()?,
            m13: data.read_f32()?,
            m20: data.read_f32()?,
            m21: data.read_f32()?,
            m22: data.read_f32()?,
            m23: data.read_f32()?,
            m30: data.read_f32()?,
            m31: data.read_f32()?,
            m32: data.read_f32()?,
            m33: data.read_f32()?,
        })
    }
}

impl Encodeable for Transform4x4 {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.m00)?;
        buffer.write_f32(self.m01)?;
        buffer.write_f32(self.m02)?;
        buffer.write_f32(self.m03)?;
        buffer.write_f32(self.m10)?;
        buffer.write_f32(self.m11)?;
        buffer.write_f32(self.m12)?;
        buffer.write_f32(self.m13)?;
        buffer.write_f32(self.m20)?;
        buffer.write_f32(self.m21)?;
        buffer.write_f32(self.m22)?;
        buffer.write_f32(self.m23)?;
        buffer.write_f32(self.m30)?;
        buffer.write_f32(self.m31)?;
        buffer.write_f32(self.m32)?;
        buffer.write_f32(self.m33)?;

        Ok(())
    }
}

/// Common operations for transformation matrices.
///
/// This trait abstracts behavior shared between [`Transform3x4`] and [`Transform4x4`],
/// providing matrix element access and transformation utilities.
///
/// # Provided Methods
///
/// - **Element Access**: m00() through m33() - Access individual matrix elements
/// - **Rotation**: `rotation_matrix()` - Extract 3x3 rotation submatrix
/// - **Scaling**: `extract_scales()`, `apply_scales()`, `normalize_rotation_matrix()`
/// - **Euler Angles**: `rotation_matrix_to_euler_angles()`, `euler_angles_to_rotation_matrix()`
/// - **Identity**: `identity()` - Create identity transform
///
/// # Rotation Order
///
/// Euler angle conversions use 'xyz' extrinsic rotation order (roll-pitch-yaw).
///
/// # Example
///
/// ```ignore
/// use rpfm_lib::files::bmd::common::{Transform4x4, Matrix};
///
/// let transform = Transform4x4::identity();
///
/// // Extract rotation
/// let rotation = transform.rotation_matrix();
/// let scales = Transform4x4::extract_scales(rotation);
///
/// // Convert to Euler angles (in degrees)
/// let (x, y, z) = Transform4x4::rotation_matrix_to_euler_angles(rotation, true);
/// println!("Rotation: X={}, Y={}, Z={}", x, y, z);
/// ```
pub trait Matrix {
    /// Returns matrix element at row 0, column 0.
    fn m00(&self) -> f32;
    /// Returns matrix element at row 0, column 1.
    fn m01(&self) -> f32;
    /// Returns matrix element at row 0, column 2.
    fn m02(&self) -> f32;
    /// Returns matrix element at row 0, column 3 (0.0 for 3x4 matrices).
    fn m03(&self) -> f32;
    /// Returns matrix element at row 1, column 0.
    fn m10(&self) -> f32;
    /// Returns matrix element at row 1, column 1.
    fn m11(&self) -> f32;
    /// Returns matrix element at row 1, column 2.
    fn m12(&self) -> f32;
    /// Returns matrix element at row 1, column 3 (0.0 for 3x4 matrices).
    fn m13(&self) -> f32;
    /// Returns matrix element at row 2, column 0.
    fn m20(&self) -> f32;
    /// Returns matrix element at row 2, column 1.
    fn m21(&self) -> f32;
    /// Returns matrix element at row 2, column 2.
    fn m22(&self) -> f32;
    /// Returns matrix element at row 2, column 3 (0.0 for 3x4 matrices).
    fn m23(&self) -> f32;
    /// Returns matrix element at row 3, column 0 (translation X).
    fn m30(&self) -> f32;
    /// Returns matrix element at row 3, column 1 (translation Y).
    fn m31(&self) -> f32;
    /// Returns matrix element at row 3, column 2 (translation Z).
    fn m32(&self) -> f32;
    /// Returns matrix element at row 3, column 3 (1.0 for 3x4 matrices).
    fn m33(&self) -> f32;

    /// Extracts the 3x3 rotation submatrix.
    ///
    /// Converts from CA's column-major serialization to standard row-major
    /// rotation matrix representation.
    ///
    /// # Returns
    ///
    /// 3x3 rotation matrix as nalgebra `Matrix3<f64>`.
    ///
    /// # Reference
    ///
    /// See: <https://developer.unigine.com/forum/uploads/monthly_2020_05/image.png.674c8b961433f2a7a62c54bc55cb599c.png>
    fn rotation_matrix(&self) -> Matrix3<f64> {

        // Fix order of the elements here
        Matrix3::new(
            self.m00() as f64, self.m10() as f64, self.m20() as f64,
            self.m01() as f64, self.m11() as f64, self.m21() as f64,
            self.m02() as f64, self.m12() as f64, self.m22() as f64
        )
    }

    /// Extracts scale factors from a rotation matrix.
    ///
    /// Computes the scale of each axis by taking the norm of each column vector.
    ///
    /// # Parameters
    ///
    /// - `matrix`: 3x3 rotation/scale matrix
    ///
    /// # Returns
    ///
    /// Tuple of (scale_x, scale_y, scale_z)
    ///
    /// # Note
    ///
    /// **Does not support negative scales.** Negative scales will be treated as positive.
    ///
    /// # Reference
    ///
    /// See: <https://math.stackexchange.com/a/1463487>
    fn extract_scales(matrix: Matrix3<f64>) -> (f64, f64, f64) {
        let scale = (
            matrix.column(0).norm(),
            matrix.column(1).norm(),
            matrix.column(2).norm()
        );
        scale
    }

    /// Applies scale factors to a rotation matrix.
    ///
    /// Scales each column of the matrix by the corresponding scale factor.
    ///
    /// # Parameters
    ///
    /// - `matrix`: 3x3 rotation matrix (should be normalized)
    /// - `scales`: Tuple of (scale_x, scale_y, scale_z)
    ///
    /// # Returns
    ///
    /// Scaled rotation matrix
    fn apply_scales(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
        Matrix3::new(
            matrix[(0, 0)] * scales.0, matrix[(0, 1)] * scales.1, matrix[(0, 2)] * scales.2,
            matrix[(1, 0)] * scales.0, matrix[(1, 1)] * scales.1, matrix[(1, 2)] * scales.2,
            matrix[(2, 0)] * scales.0, matrix[(2, 1)] * scales.1, matrix[(2, 2)] * scales.2,
        )
    }

    /// Normalizes a rotation matrix by removing scale factors.
    ///
    /// Divides each column by the corresponding scale factor to produce a pure
    /// rotation matrix.
    ///
    /// # Parameters
    ///
    /// - `matrix`: 3x3 rotation/scale matrix
    /// - `scales`: Tuple of (scale_x, scale_y, scale_z) to remove
    ///
    /// # Returns
    ///
    /// Normalized rotation matrix (orthonormal)
    fn normalize_rotation_matrix(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
        Matrix3::new(
            matrix[(0, 0)] / scales.0, matrix[(0, 1)] / scales.1, matrix[(0, 2)] / scales.2,
            matrix[(1, 0)] / scales.0, matrix[(1, 1)] / scales.1, matrix[(1, 2)] / scales.2,
            matrix[(2, 0)] / scales.0, matrix[(2, 1)] / scales.1, matrix[(2, 2)] / scales.2,
        )
    }

    /// Converts a rotation matrix to Euler angles.
    ///
    /// Uses 'xyz' extrinsic rotation order (roll-pitch-yaw).
    ///
    /// # Parameters
    ///
    /// - `matrix`: 3x3 rotation matrix
    /// - `degrees`: If true, return angles in degrees; if false, in radians
    ///
    /// # Returns
    ///
    /// Tuple of (x_rotation, y_rotation, z_rotation) in specified units
    ///
    /// # Example (Python equivalent using scipy)
    ///
    /// ```python
    /// from scipy.spatial.transform import Rotation as R
    /// r = R.from_euler("xyz", [-130.0, 80.0, -30.0], degrees=True)
    /// m = r.as_matrix()
    /// r = R.from_matrix(m)
    /// angles = r.as_euler("xyz", degrees=True)
    /// ```
    fn rotation_matrix_to_euler_angles(matrix: Matrix3<f64>, degrees: bool) -> (f64, f64, f64) {
        let rotation = Rotation3::from_matrix_unchecked(matrix);
        let euler = rotation.euler_angles();
        if degrees {
            (
                euler.0.to_degrees(),
                euler.1.to_degrees(),
                euler.2.to_degrees(),
            )
        } else {
           (euler.0, euler.1, euler.2)
        }
    }

    /// Converts Euler angles to a rotation matrix.
    ///
    /// Uses 'xyz' extrinsic rotation order (roll-pitch-yaw).
    ///
    /// # Parameters
    ///
    /// - `angles`: Tuple of (x_rotation, y_rotation, z_rotation)
    /// - `degrees`: If true, angles are in degrees; if false, in radians
    ///
    /// # Returns
    ///
    /// 3x3 rotation matrix with values near zero cleaned up (< 1e-5 set to 0.0)
    fn euler_angles_to_rotation_matrix(angles: (f64, f64, f64), degrees: bool) -> Matrix3<f64> {
        let _angles = if degrees {
            (
                angles.0.to_radians(),
                angles.1.to_radians(),
                angles.2.to_radians(),
            )
        } else {
            angles
        };
        let rotation = Rotation3::from_euler_angles(_angles.0, _angles.1, _angles.2);
        let mut matrix : Matrix3<f64> = rotation.into();

        // Clean up near-zero values for prettier output
        matrix.iter_mut().for_each(|element| {
            if element.abs() < 1e-5 {
                *element = 0.0;
            }
        });
        matrix
    }

    /// Creates an identity transformation matrix.
    ///
    /// # Returns
    ///
    /// Identity matrix (no rotation, no translation, unit scale)
    fn identity() -> Self;
}

impl Matrix for Transform3x4 {
    fn m00(&self) -> f32 {
        self.m00
    }
    fn m01(&self) -> f32 {
        self.m01
    }
    fn m02(&self) -> f32 {
        self.m02
    }
    fn m03(&self) -> f32 {
        0.0
    }
    fn m10(&self) -> f32 {
        self.m10
    }
    fn m11(&self) -> f32 {
        self.m11
    }
    fn m12(&self) -> f32 {
        self.m12
    }
    fn m13(&self) -> f32 {
        0.0
    }
    fn m20(&self) -> f32 {
        self.m20
    }
    fn m21(&self) -> f32 {
        self.m21
    }
    fn m22(&self) -> f32 {
        self.m22
    }
    fn m23(&self) -> f32 {
        0.0
    }
    fn m30(&self) -> f32 {
        self.m30
    }
    fn m31(&self) -> f32 {
        self.m31
    }
    fn m32(&self) -> f32 {
        self.m32
    }
    fn m33(&self) -> f32 {
        1.0
    }

    fn identity() -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
        }
    }
}

impl Matrix for Transform4x4 {
    fn m00(&self) -> f32 {
        self.m00
    }
    fn m01(&self) -> f32 {
        self.m01
    }
    fn m02(&self) -> f32 {
        self.m02
    }
    fn m03(&self) -> f32 {
        self.m03
    }
    fn m10(&self) -> f32 {
        self.m10
    }
    fn m11(&self) -> f32 {
        self.m11
    }
    fn m12(&self) -> f32 {
        self.m12
    }
    fn m13(&self) -> f32 {
        self.m13
    }
    fn m20(&self) -> f32 {
        self.m20
    }
    fn m21(&self) -> f32 {
        self.m21
    }
    fn m22(&self) -> f32 {
        self.m22
    }
    fn m23(&self) -> f32 {
        self.m23
    }
    fn m30(&self) -> f32 {
        self.m30
    }
    fn m31(&self) -> f32 {
        self.m31
    }
    fn m32(&self) -> f32 {
        self.m32
    }
    fn m33(&self) -> f32 {
        self.m33
    }

    fn identity() -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m03: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m20: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m30: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }
}

impl Point3d {
    /// Creates a new 3D point with the specified coordinates.
    ///
    /// # Parameters
    ///
    /// - `x`: X-axis coordinate
    /// - `y`: Y-axis coordinate
    /// - `z`: Z-axis coordinate
    ///
    /// # Returns
    ///
    /// New `Point3d` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::files::bmd::common::Point3d;
    ///
    /// let point = Point3d::new(10.0, 20.0, 30.0);
    /// assert_eq!(*point.x(), 10.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Sub for Point3d {
    type Output = Self;

    /// Subtracts two 3D points to produce a vector.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rpfm_lib::files::bmd::common::Point3d;
    ///
    /// let p1 = Point3d::new(10.0, 20.0, 30.0);
    /// let p2 = Point3d::new(5.0, 10.0, 15.0);
    /// let diff = p1 - p2;  // Results in (5.0, 10.0, 15.0)
    /// ```
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl From<Cube> for Transform4x4 {
    fn from(value: Cube) -> Self {
        Self {
            m00: value.min_x,
            m01: value.min_y,
            m02: value.min_z,
            m10: value.max_x,
            m11: value.max_y,
            m12: value.max_z,
            ..Default::default()
        }
    }
}

impl From<Transform4x4> for Cube {
    fn from(value: Transform4x4) -> Self {
        Self {
            min_x: value.m00,
            min_y: value.m01,
            min_z: value.m02,
            max_x: value.m10,
            max_y: value.m11,
            max_z: value.m12
        }
    }
}
