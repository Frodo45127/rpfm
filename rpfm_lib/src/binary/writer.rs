//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the [`WriteBytes`] trait, to write bytes from known types to a [`Writer`].
//!
//! [`Writer`]: std::io::Write

use byteorder::{LittleEndian, WriteBytesExt};
use encoding_rs::ISO_8859_15;
use half::f16;
use nalgebra::{Vector2, Vector3, Vector4};

use std::io::Write;

use crate::error::{RLibError, Result};

//---------------------------------------------------------------------------//
//                            Trait Definition
//---------------------------------------------------------------------------//

/// This trait allow us to easily write all kind of data types to something that implements [`Write`].
pub trait WriteBytes: Write {

    /// This function tries to write a bool value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_bool(true).is_ok());
    /// assert_eq!(data, vec![1]);
    /// ```
    fn write_bool(&mut self, boolean: bool) -> Result<()> {
        self.write_u8(u8::from(boolean))
    }

    /// This function tries to write a byte value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_u8(10).is_ok());
    /// assert_eq!(data, vec![10]);
    /// ```
    fn write_u8(&mut self, value: u8) -> Result<()> {
        WriteBytesExt::write_u8(self, value).map_err(From::from)
    }

    /// This function tries to write an u16 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_u16(258).is_ok());
    /// assert_eq!(data, vec![2, 1]);
    /// ```
    fn write_u16(&mut self, integer: u16) -> Result<()> {
        WriteBytesExt::write_u16::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an u24 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_u24(8492696).is_ok());
    /// assert_eq!(data, vec![152, 150, 129]);
    /// ```
    fn write_u24(&mut self, integer: u32) -> Result<()> {
        WriteBytesExt::write_u24::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an u32 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_u32(258).is_ok());
    /// assert_eq!(data, vec![2, 1, 0, 0]);
    /// ```
    fn write_u32(&mut self, integer: u32) -> Result<()> {
        WriteBytesExt::write_u32::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an u64 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_u64(258).is_ok());
    /// assert_eq!(data, vec![2, 1, 0, 0, 0, 0, 0, 0]);
    /// ```
    fn write_u64(&mut self, integer: u64) -> Result<()> {
        WriteBytesExt::write_u64::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an u32 value to `self` as a cauleb128 value.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_cauleb128(10, 0).is_ok());
    /// assert_eq!(data, vec![10]);
    /// ```
    fn write_cauleb128(&mut self, mut integer: u32, padding: usize) -> Result<()> {
        let mut data = vec![];

        loop {

            // Get the byte to encode.
            let byte = integer & 0x7f;

            // If it's not the last one, encode it with the 0x80 bit set,
            // and move the rest of the number to be ready to check the next one.
            data.push(byte as u8 | 0x80);
            if byte != integer {
                integer >>= 7;
            } else {
                break;
            }
        }

        if data.len() < padding {
            data.resize(padding, 0x80);
        }

        data.reverse();
        *data.last_mut().unwrap() &= 0x7f;

        self.write_all(&data).map_err(From::from)
    }

    /// This function tries to write an i8 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_i8(-2).is_ok());
    /// assert_eq!(data, vec![254]);
    /// ```
    fn write_i8(&mut self, integer: i8) -> Result<()> {
        WriteBytesExt::write_i8(self, integer).map_err(From::from)
    }

    /// This function tries to write an i16 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_i16(-258).is_ok());
    /// assert_eq!(data, vec![254, 254]);
    /// ```
    fn write_i16(&mut self, integer: i16) -> Result<()> {
        WriteBytesExt::write_i16::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an i24 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_i24(8_492_696).is_ok());
    /// assert_eq!(data, vec![152, 150, 129]);
    /// ```
    fn write_i24(&mut self, integer: i32) -> Result<()> {
        WriteBytesExt::write_i24::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an i32 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_i32(-258).is_ok());
    /// assert_eq!(data, vec![254, 254, 255, 255]);
    /// ```
    fn write_i32(&mut self, integer: i32) -> Result<()> {
        WriteBytesExt::write_i32::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an i64 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_i64(-258).is_ok());
    /// assert_eq!(data, vec![254, 254, 255, 255, 255, 255, 255, 255]);
    /// ```
    fn write_i64(&mut self, integer: i64) -> Result<()> {
        WriteBytesExt::write_i64::<LittleEndian>(self, integer).map_err(From::from)
    }

    /// This function tries to write an Optional i16 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_optional_i16(-258).is_ok());
    /// assert_eq!(data, vec![1, 254, 254]);
    /// ```
    fn write_optional_i16(&mut self, integer: i16) -> Result<()> {
        self.write_bool(true)?;
        Self::write_i16(self, integer)
    }

    /// This function tries to write an Optional i32 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_optional_i32(-258).is_ok());
    /// assert_eq!(data, vec![1, 254, 254, 255, 255]);
    /// ```
    fn write_optional_i32(&mut self, integer: i32) -> Result<()> {
        self.write_bool(true)?;
        Self::write_i32(self, integer)
    }

    /// This function tries to write an Optional i64 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_optional_i64(-258).is_ok());
    /// assert_eq!(data, vec![1, 254, 254, 255, 255, 255, 255, 255, 255]);
    /// ```
    fn write_optional_i64(&mut self, integer: i64) -> Result<()> {
        self.write_bool(true)?;
        Self::write_i64(self, integer)
    }

    /// This function tries to write a f16 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_f16(half::f16::from_f32(-10.2)).is_ok());
    /// assert_eq!(data, vec![26, 201]);
    /// ```
    fn write_f16(&mut self, float: half::f16) -> Result<()> {
        self.write_u16(float.to_bits())
    }

    /// This function tries to write a f32 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_f32(-10.2).is_ok());
    /// assert_eq!(data, vec![51, 51, 35, 193]);
    /// ```
    fn write_f32(&mut self, float: f32) -> Result<()> {
        WriteBytesExt::write_f32::<LittleEndian>(self, float).map_err(From::from)
    }

    /// This function tries to write a normal f32 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_f32_normal_as_u8(0.5).is_ok());
    /// assert_eq!(data, vec![191]);
    /// ```
    fn write_f32_normal_as_u8(&mut self, float: f32) -> Result<()> {
        let value = ((float + 1.0) / 2.0 * 255.0).round() as u8;
        self.write_u8(value)
    }

    /// This function tries to write a f64 value to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_f64(-10.2).is_ok());
    /// assert_eq!(data, vec![102, 102, 102, 102, 102, 102, 36, 192]);
    /// ```
    fn write_f64(&mut self, float: f64) -> Result<()> {
        WriteBytesExt::write_f64::<LittleEndian>(self, float).map_err(From::from)
    }

    /// This function tries to write an UTF-8 String to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u8("Wahahahaha").is_ok());
    /// assert_eq!(data, vec![87, 97, 104, 97, 104, 97, 104, 97, 104, 97]);
    /// ```
    fn write_string_u8(&mut self, string: &str) -> Result<()> {
        self.write_all(string.as_bytes()).map_err(From::from)
    }

    /// This function tries to write an UTF-8 String as an ISO-8859-1 String to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u8_iso_8859_1("Wahaÿhahaha").is_ok());
    /// assert_eq!(data, vec![87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97]);
    /// ```
    fn write_string_u8_iso_8859_1(&mut self, string: &str) -> Result<()> {
        let (string, _, _) = ISO_8859_15.encode(string);
        self.write_all(&string).map_err(From::from)
    }

    /// This function tries to write an UTF-8 String to `self` as a 00-Padded UTF-8 String with a max size of `size`.
    ///
    /// It may fail if `self` cannot be written to. Ìf `crop` is true, in case the string is longer than the size
    /// the string will be cropped to fit in the size we have. If it's false, an error will be returned.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u8_0padded("Waha", 8, false).is_ok());
    /// assert_eq!(data, vec![87, 97, 104, 97, 0, 0, 0, 0]);
    /// ```
    fn write_string_u8_0padded(&mut self, string: &str, size: usize, crop: bool) -> Result<()> {
        if string.len() > size {
            if crop {
                let mut string = string.to_owned();
                string.truncate(size);
                self.write_string_u8(&string)?;
                self.write_all(&vec![0; size - string.len()]).map_err(From::from)
            } else {
               Err(RLibError::EncodingPaddedStringError("UTF-8 0-Padded String".to_owned(), string.to_owned(), string.len(), size))
            }
        }

        else {
            self.write_string_u8(string)?;
            self.write_all(&vec![0; size - string.len()]).map_err(From::from)
        }
    }

    /// This function tries to write an UTF-8 String to `self` as a 00-Terminated (or NULL-Terminated) UTF-8 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u8_0terminated("Wahahaha").is_ok());
    /// assert_eq!(data, vec![87, 97, 104, 97, 104, 97, 104, 97, 0]);
    /// ```
    fn write_string_u8_0terminated(&mut self, string: &str) -> Result<()> {
        self.write_string_u8(string)?;
        Self::write_u8(self, 0)
    }

    /// This function tries to write an UTF-8 String to `self` as a Sized UTF-8 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_sized_string_u8("Wahaha").is_ok());
    /// assert_eq!(data, vec![6, 0, 87, 97, 104, 97, 104, 97]);
    /// ```
    fn write_sized_string_u8(&mut self, string: &str) -> Result<()> {
        self.write_u16(string.len() as u16)?;
        self.write_string_u8(string)
    }

    /// This function tries to write an UTF-8 String to `self` as a Sized UTF-8 String, with a 4 byte size.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_sized_string_u8_u32("Wahaha").is_ok());
    /// assert_eq!(data, vec![6, 0, 0, 0, 87, 97, 104, 97, 104, 97]);
    /// ```
    fn write_sized_string_u8_u32(&mut self, string: &str) -> Result<()> {
        self.write_u32(string.len() as u32)?;
        self.write_string_u8(string)
    }

    /// This function tries to write an UTF-8 String to `self` as an Optional UTF-8 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_optional_string_u8("Wahaha").is_ok());
    /// assert_eq!(data, vec![1, 6, 0, 87, 97, 104, 97, 104, 97]);
    /// ```
    fn write_optional_string_u8(&mut self, string: &str) -> Result<()> {
        if string.is_empty() {
            self.write_bool(false)
        }
        else {
            self.write_bool(true)?;
            self.write_u16(string.len() as u16)?;
            self.write_string_u8(string)
        }
    }

    /// This function tries to write an UTF-8 String to `self` as an UTF-16 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u16("Wahaha").is_ok());
    /// assert_eq!(data, vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0]);
    /// ```
    fn write_string_u16(&mut self, string: &str) -> Result<()> {
        string.encode_utf16().try_for_each(|character| self.write_u16(character))
    }

    /// This function tries to write an UTF-8 String to `self` as a 00-Padded UTF-16 String with a max size of `size`.
    ///
    /// It may fail if `self` cannot be written to. Ìf `crop` is true, in case the string is longer than the size
    /// the string will be cropped to fit in the size we have. If it's false, an error will be returned.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_u16_0padded("Waha", 16, false).is_ok());
    /// assert_eq!(data, vec![87, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// ```
    fn write_string_u16_0padded(&mut self, string: &str, size: usize, crop: bool) -> Result<()> {
        if string.len() * 2 > size {
            if crop {
                let mut string = string.to_owned();
                string.truncate(size);
                self.write_string_u16(&string)?;
                self.write_all(&vec![0; size - (string.len() * 2)]).map_err(From::from)
            } else {
                Err(RLibError::EncodingPaddedStringError("UTF-16 0-Padded String".to_owned(), string.to_string(), string.len(), size))
            }
        }

        else {
            self.write_string_u16(string)?;
            self.write_all(&vec![0; size - (string.len() * 2)]).map_err(From::from)
        }
    }

    /// This function tries to write an UTF-8 String to `self` as a Sized UTF-16 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_sized_string_u16("¡Bebes mejor de lo que luchas, Zhang Fei!").is_ok());
    /// assert_eq!(data, vec![0x29, 0x00, 0xA1, 0x00, 0x42, 0x00, 0x65, 0x00, 0x62, 0x00, 0x65, 0x00, 0x73, 0x00, 0x20, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6A, 0x00, 0x6F, 0x00, 0x72, 0x00, 0x20, 0x00, 0x64, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x71, 0x00, 0x75, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x75, 0x00, 0x63, 0x00, 0x68, 0x00, 0x61, 0x00, 0x73, 0x00, 0x2C, 0x00, 0x20, 0x00, 0x5A, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00, 0x46, 0x00, 0x65, 0x00, 0x69, 0x00, 0x21, 0x00]);
    /// ```
    fn write_sized_string_u16(&mut self, string: &str) -> Result<()> {
        self.write_u16(string.encode_utf16().count() as u16)?;
        self.write_string_u16(string)
    }

    /// This function tries to write an UTF-8 String to `self` as a Sized UTF-16 String, with a four-byte size.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_sized_string_u16_u32("¡Bebes mejor de lo que luchas, Zhang Fei!").is_ok());
    /// assert_eq!(data, vec![0x29, 0x00, 0x00, 0x00, 0xA1, 0x00, 0x42, 0x00, 0x65, 0x00, 0x62, 0x00, 0x65, 0x00, 0x73, 0x00, 0x20, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x6A, 0x00, 0x6F, 0x00, 0x72, 0x00, 0x20, 0x00, 0x64, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x20, 0x00, 0x71, 0x00, 0x75, 0x00, 0x65, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x75, 0x00, 0x63, 0x00, 0x68, 0x00, 0x61, 0x00, 0x73, 0x00, 0x2C, 0x00, 0x20, 0x00, 0x5A, 0x00, 0x68, 0x00, 0x61, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00, 0x46, 0x00, 0x65, 0x00, 0x69, 0x00, 0x21, 0x00]);
    /// ```
    fn write_sized_string_u16_u32(&mut self, string: &str) -> Result<()> {
        self.write_u32(string.encode_utf16().count() as u32)?;
        self.write_string_u16(string)
    }

    /// This function tries to write an UTF-8 String to `self` as an Optional UTF-16 String.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_optional_string_u16("Waha").is_ok());
    /// assert_eq!(data, vec![1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0]);
    /// ```
    fn write_optional_string_u16(&mut self, string: &str) -> Result<()> {
        if string.is_empty() {
            self.write_bool(false)
        }
        else {
            self.write_bool(true)?;
            self.write_u16(string.encode_utf16().count() as u16)?;
            self.write_string_u16(string)
        }
    }

    /// This function tries to write an UTF-8 String representing a Hex-Encoded RGB Colour to `self`.
    ///
    /// It may fail if `self` cannot be written to or if the string is not a valid Hex-Encoded RGB Colour.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_string_colour_rgb("0504FF").is_ok());
    /// assert_eq!(data, vec![0xFF, 0x04, 0x05, 0x00]);
    /// ```
    fn write_string_colour_rgb(&mut self, value: &str) -> Result<()> {
        let value = u32::from_str_radix(value, 16)?;
        self.write_u32(value)
    }

    /// This function tries to write an Vector of 2 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_2_u8(Vector2::new(10, 10)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A]);
    /// ```
    fn write_vector_2_u8(&mut self, value: Vector2<u8>) -> Result<()> {
        self.write_u8(value.x)?;
        self.write_u8(value.y)?;

        Ok(())
    }

    /// This function tries to write an Vector of 2 f32 converted to 2 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_2_f32_pct_as_vector_2_u8(Vector2::new(0.039215688, 0.039215688)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A]);
    /// ```
    fn write_vector_2_f32_pct_as_vector_2_u8(&mut self, value: Vector2<f32>) -> Result<()> {
        self.write_u8((value.x * 255.0) as u8)?;
        self.write_u8((value.y * 255.0) as u8)?;

        Ok(())
    }

    /// This function tries to write an Vector of 2 f32 converted to 2 f16 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_2_f32_as_vector_2_f16(Vector2::new(0.00018429756, 0.00018429756)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
    /// ```
    fn write_vector_2_f32_as_vector_2_f16(&mut self, value: Vector2<f32>) -> Result<()> {
        self.write_f16(f16::from_f32(value.x))?;
        self.write_f16(f16::from_f32(value.y))?;

        Ok(())
    }

    /// This function tries to write an Vector of 3 normalized f32 converted to 4 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector3;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_3_f32_normal_as_vector_4_u8(Vector3::new(-0.92156863, -0.92156863, -0.92156863)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x00]);
    /// ```
    fn write_vector_3_f32_normal_as_vector_4_u8(&mut self, value: Vector3<f32>) -> Result<()> {
        self.write_f32_normal_as_u8(value.x)?;
        self.write_f32_normal_as_u8(value.y)?;
        self.write_f32_normal_as_u8(value.z)?;
        self.write_f32_normal_as_u8(-1.0)?;

        Ok(())
    }

    /// This function tries to write an Vector of 4 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_u8(Vector4::new(10, 10, 10, 10)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
    /// ```
    fn write_vector_4_u8(&mut self, value: Vector4<u8>) -> Result<()> {
        self.write_u8(value.x)?;
        self.write_u8(value.y)?;
        self.write_u8(value.z)?;
        self.write_u8(value.w)?;

        Ok(())
    }

    /// This function tries to write an Vector of 4 f32 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_f32(Vector4::new(10.0, 10.0, 10.0, 10.0)).is_ok());
    /// assert_eq!(data, vec![0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65]);
    /// ```
    fn write_vector_4_f32(&mut self, value: Vector4<f32>) -> Result<()> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.z)?;
        self.write_f32(value.w)?;

        Ok(())
    }

    /// This function tries to write an Vector of 4 f32 to a Vector of 3 f32 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_f32_to_vector_3_f32(Vector4::new(10.0, 10.0, 10.0, 0.0)).is_ok());
    /// assert_eq!(data, vec![0, 0, 32, 65, 0, 0, 32, 65, 0, 0, 32, 65]);
    /// ```
    fn write_vector_4_f32_to_vector_3_f32(&mut self, value: Vector4<f32>) -> Result<()> {
        self.write_f32(value.x)?;
        self.write_f32(value.y)?;
        self.write_f32(value.z)?;

        Ok(())
    }

    /// This function tries to write an Vector of 4 f32 as percentage converted to 4 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_f32_pct_as_vector_4_u8(Vector4::new(0.039215688, 0.039215688, 0.039215688, 0.039215688)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
    /// ```
    fn write_vector_4_f32_pct_as_vector_4_u8(&mut self, value: Vector4<f32>) -> Result<()> {
        self.write_u8((value.x * 255.0) as u8)?;
        self.write_u8((value.y * 255.0) as u8)?;
        self.write_u8((value.z * 255.0) as u8)?;
        self.write_u8((value.w * 255.0) as u8)?;

        Ok(())
    }


    /// This function tries to write an Vector of 4 normalized f32 converted to 4 u8 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_f32_normal_as_vector_4_u8(Vector4::new(-0.92156863, -0.92156863, -0.92156863, -0.92156863)).is_ok());
    /// assert_eq!(data, vec![0x0A, 0x0A, 0x0A, 0x0A]);
    /// ```
    fn write_vector_4_f32_normal_as_vector_4_u8(&mut self, value: Vector4<f32>) -> Result<()> {
        self.write_f32_normal_as_u8(value.x)?;
        self.write_f32_normal_as_u8(value.y)?;
        self.write_f32_normal_as_u8(value.z)?;
        self.write_f32_normal_as_u8(value.w)?;

        Ok(())
    }

    /// This function tries to write an Vector of 4 normalized f32 converted to 4 f16 to `self`.
    ///
    /// It may fail if `self` cannot be written to.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::WriteBytes;
    ///
    /// let mut data = vec![];
    /// assert!(data.write_vector_4_f32_normal_as_vector_4_f16(Vector4::new(3.096775, 3.096775, 3.096775, 1.7597656)).is_ok());
    /// assert_eq!(data, vec![
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F
    /// ]);
    /// ```
    fn write_vector_4_f32_normal_as_vector_4_f16(&mut self, value: Vector4<f32>) -> Result<()> {
        let mut x = value.x;
        let mut y = value.y;
        let mut z = value.z;
        let w = value.w;

        if w != 0.0 {
            x /= w;
            y /= w;
            z /= w;
        }

        self.write_f16(f16::from_f32(x))?;
        self.write_f16(f16::from_f32(y))?;
        self.write_f16(f16::from_f32(z))?;
        self.write_f16(f16::from_f32(w))?;

        Ok(())
    }
}

// Automatic implementation for everything that implements `Write`.
impl<W: Write> WriteBytes for W {}
