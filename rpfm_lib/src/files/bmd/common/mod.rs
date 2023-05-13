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
use crate::error::Result;
use crate::files::{Decodeable, EncodeableExtraData, Encodeable};

use super::*;

pub mod building_link;
pub mod flags;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Colour {
    r: f32,
    g: f32,
    b: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Cube {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Outline {
    outline: Vec<Point2d>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Point2d {
    x: f32,
    y: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Point3d {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Polygon2d {
    points: Vec<Point2d>
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Quaternion {
    i: f32,
    j: f32,
    k: f32,
    w: f32,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Rectangle {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

// NOTE: These two don't have automatic getters. They get their getters from the Matrix trait.
#[derive(Default, PartialEq, Clone, Debug, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get_mut = "pub", set = "pub")]
pub struct Transform3x4{
    m00: f32,
    m01: f32,
    m02: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m20: f32,
    m21: f32,
    m22: f32,
    m30: f32,
    m31: f32,
    m32: f32,
}

#[derive(Default, PartialEq, Clone, Debug, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get_mut = "pub", set = "pub")]
pub struct Transform4x4 {
    m00: f32,
    m01: f32,
    m02: f32,
    m03: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m13: f32,
    m20: f32,
    m21: f32,
    m22: f32,
    m23: f32,
    m30: f32,
    m31: f32,
    m32: f32,
    m33: f32,
}

//---------------------------------------------------------------------------//
//                           Implementations
//---------------------------------------------------------------------------//

impl Decodeable for Colour {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        Ok(Self {
            r: data.read_f32()?,
            g: data.read_f32()?,
            b: data.read_f32()?,
        })
    }
}

impl Encodeable for Colour {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_f32(self.r)?;
        buffer.write_f32(self.g)?;
        buffer.write_f32(self.b)?;

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

impl Decodeable for Outline {

    fn decode<R: ReadBytes>(data: &mut R, extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let mut decoded = Self::default();

        for _ in 0..data.read_u32()? {
            decoded.outline.push(Point2d::decode(data, extra_data)?);
        }

        Ok(decoded)
    }
}

impl Encodeable for Outline {

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

/// This trait abstract common behavior between the known transforms.
pub trait Matrix {
    fn m00(&self) -> f32;
    fn m01(&self) -> f32;
    fn m02(&self) -> f32;
    fn m03(&self) -> f32;
    fn m10(&self) -> f32;
    fn m11(&self) -> f32;
    fn m12(&self) -> f32;
    fn m13(&self) -> f32;
    fn m20(&self) -> f32;
    fn m21(&self) -> f32;
    fn m22(&self) -> f32;
    fn m23(&self) -> f32;
    fn m30(&self) -> f32;
    fn m31(&self) -> f32;
    fn m32(&self) -> f32;
    fn m33(&self) -> f32;

    /// Restores normal Rotation matrix representation as on the picture:
    /// https://developer.unigine.com/forum/uploads/monthly_2020_05/image.png.674c8b961433f2a7a62c54bc55cb599c.png
    /// pic note: (it seems like R should be applies to each column)
    /// from CA's column-first serialization
    fn rotation_matrix(&self) -> Matrix3<f64> {

        // Fix order of the elements here
        let matrix = Matrix3::new(
            self.m00() as f64, self.m10() as f64, self.m20() as f64,
            self.m01() as f64, self.m11() as f64, self.m21() as f64,
            self.m02() as f64, self.m12() as f64, self.m22() as f64
        );
        matrix
    }

    ///Extracts scales as described here:
    ///https://math.stackexchange.com/a/1463487
    ///DOES NOT SUPPORT NEGATIVE SCALES
    fn extract_scales(matrix: Matrix3<f64>) -> (f64, f64, f64) {
        let scale = (
            matrix.column(0).norm(),
            matrix.column(1).norm(),
            matrix.column(2).norm()
        );
        scale
    }

    fn apply_scales(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
        let scaled_matrix = Matrix3::new(
            matrix[(0, 0)] * scales.0, matrix[(0, 1)] * scales.1, matrix[(0, 2)] * scales.2,
            matrix[(1, 0)] * scales.0, matrix[(1, 1)] * scales.1, matrix[(1, 2)] * scales.2,
            matrix[(2, 0)] * scales.0, matrix[(2, 1)] * scales.1, matrix[(2, 2)] * scales.2,
        );
        scaled_matrix
    }

    fn normalize_rotation_matrix(matrix: Matrix3<f64>, scales: (f64, f64, f64)) -> Matrix3<f64> {
        let normalized_matrix = Matrix3::new(
            matrix[(0, 0)] / scales.0, matrix[(0, 1)] / scales.1, matrix[(0, 2)] / scales.2,
            matrix[(1, 0)] / scales.0, matrix[(1, 1)] / scales.1, matrix[(1, 2)] / scales.2,
            matrix[(2, 0)] / scales.0, matrix[(2, 1)] / scales.1, matrix[(2, 2)] / scales.2,
        );
        normalized_matrix
    }

    /*
    As I understand it uses 'xyz' extrinsic rotations order
    Python code using scipy lib:
        r = Rotation.from_euler("xyz", [-130.00000555832042, 80.00000457701574, -29.999991697018082], degrees=True)
        m = r.as_matrix()
        r = Rotation.from_matrix(m)
        angles = r.as_euler("xyz", degrees=True)
     */
    fn rotation_matrix_to_euler_angles(matrix: Matrix3<f64>, degrees: bool) -> (f64, f64, f64) {
        let rotation = Rotation3::from_matrix_unchecked(matrix);
        let euler = rotation.euler_angles();
        let euler_angles = if degrees {
            (
                euler.0.to_degrees(),
                euler.1.to_degrees(),
                euler.2.to_degrees(),
            )
        } else {
           (euler.0, euler.1, euler.2)
        };
        euler_angles
    }

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

        //prettify
        matrix.iter_mut().for_each(|element| {
            if element.abs() < 1e-5 {
                *element = 0.0;
            }
        });
        matrix
    }
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
}
