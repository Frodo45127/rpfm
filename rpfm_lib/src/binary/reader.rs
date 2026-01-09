//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the [`ReadBytes`] trait, to read bytes to known types.

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::{ISO_8859_15, UTF_16LE};
use itertools::Itertools;
use nalgebra::{Vector2, Vector4};

use std::io::{Read, Seek, SeekFrom};

use crate::error::{Result, RLibError};

//---------------------------------------------------------------------------//
//                            Trait Definition
//---------------------------------------------------------------------------//

/// This trait allow us to easily read all kind of data from a source that implements [`Read`] + [`Seek`].
pub trait ReadBytes: Read + Seek {

    /// This function returns the lenght of the data we're reading.
    ///
    /// Extracted from the nightly std.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let mut cursor = Cursor::new(data);
    /// let len = cursor.len().unwrap();
    /// assert_eq!(len, 4);
    /// ```
    fn len(&mut self) -> Result<u64> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;
        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }
        Ok(len)
    }

    /// This function returns if the data is empty.
    ///
    /// It's slightly faster than checking for len == 0.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![];
    /// let mut cursor = Cursor::new(data);
    /// assert!(ReadBytes::is_empty(&mut cursor).unwrap());
    /// ```
    fn is_empty(&mut self) -> Result<bool> {
        self.len().map(|len| len == 0)
    }

    /// This function returns the amount of bytes specified in the `size` argument as a [`Vec<u8>`].
    ///
    /// If `rewind` is true, the cursor will be reset to its original position once the data is returned.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 2, 3, 4];
    /// let mut cursor = Cursor::new(data.to_vec());
    /// let data_read = cursor.read_slice(4, false).unwrap();
    /// assert_eq!(data, data_read);
    ///
    /// # assert_eq!(ReadBytes::read_slice(&mut Cursor::new([1, 2, 3, 4]), 4, false).unwrap(), vec![1, 2, 3, 4]);
    /// # assert_eq!(ReadBytes::read_slice(&mut Cursor::new(vec![0u8; 0]), 0, false).unwrap(), vec![0u8; 0]);
    /// # assert_eq!(ReadBytes::read_slice(&mut Cursor::new([]), 4, false).is_err(), true);
    /// ```
    fn read_slice(&mut self, size: usize, rewind: bool) -> Result<Vec<u8>> {
        let mut data = vec![0; size];

        // If len is 0, just return.
        if size == 0 {
            return Ok(data)
        }

        self.read_exact(&mut data)?;

        if rewind {
            self.seek(SeekFrom::Current(-(size as i64)))?;
        }

        Ok(data)
    }

    /// This function tries to read a bool value from `self`.
    ///
    /// This is simple: 0 is false, 1 is true. Anything else is an error.
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0, 1, 2];
    /// let mut cursor = Cursor::new(data);
    ///
    /// let first = cursor.read_bool();
    /// let second = cursor.read_bool();
    /// let third = cursor.read_bool();
    ///
    /// assert_eq!(first.unwrap(), false);
    /// assert_eq!(second.unwrap(), true);
    /// assert!(third.is_err());
    /// ```
    fn read_bool(&mut self) -> Result<bool> {
        let value = self.read_u8()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(RLibError::DecodingBoolError(value)),
        }
    }

    /// This function tries to read an unsigned byte value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_u8().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_u8().is_err(), true);
    /// ```
    fn read_u8(&mut self) -> Result<u8> {
        ReadBytesExt::read_u8(self).map_err(From::from)
    }

    /// This function tries to read an u16 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 10];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_u16().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_u16().is_err(), true);
    /// ```
    fn read_u16(&mut self) -> Result<u16> {
        ReadBytesExt::read_u16::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an u24 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![152, 150, 129, 152, 150];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_u24().unwrap();
    ///
    /// assert_eq!(data, 84_926_96);
    /// assert_eq!(cursor.read_u24().is_err(), true);
    /// ```
    fn read_u24(&mut self) -> Result<u32> {
        ReadBytesExt::read_u24::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an u32 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 0, 0, 10, 0, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_u32().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_u32().is_err(), true);
    /// ```
    fn read_u32(&mut self) -> Result<u32> {
        ReadBytesExt::read_u32::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an u64 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 0, 0, 0, 0, 0, 0, 10, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_u64().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_u64().is_err(), true);
    /// ```
    fn read_u64(&mut self) -> Result<u64> {
        ReadBytesExt::read_u64::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read CA's own take (or I greatly misunderstood this) on ULEB_128 values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x80, 10];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_cauleb128().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_cauleb128().is_err(), true);
    /// ```
    fn read_cauleb128(&mut self) -> Result<u32> {
        let mut value: u32 = 0;
        let mut byte = self.read_u8()?;

        while(byte & 0x80) != 0 {
            value = (value << 7) | (byte & 0x7f) as u32;

            // Check the new byte is even valid before continuing.
            byte = self.read_u8()?;
        }

        value = (value << 7) | (byte & 0x7f) as u32;
        Ok(value)
    }

    /// This function tries to read a signed byte value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![254];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_i8().unwrap();
    ///
    /// assert_eq!(data, -2);
    /// assert_eq!(cursor.read_i8().is_err(), true);
    /// ```
    fn read_i8(&mut self) -> Result<i8> {
        ReadBytesExt::read_i8(self).map_err(From::from)
    }

    /// This function tries to read an i16 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![254, 254, 10];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_i16().unwrap();
    ///
    /// assert_eq!(data, -258);
    /// assert_eq!(cursor.read_i16().is_err(), true);
    /// ```
    fn read_i16(&mut self) -> Result<i16> {
        ReadBytesExt::read_i16::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an i24 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![152, 150, 129, 152, 150];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_i24().unwrap();
    ///
    /// assert_eq!(data, -8_284_520);
    /// assert_eq!(cursor.read_i24().is_err(), true);
    /// ```
    fn read_i24(&mut self) -> Result<i32> {
        ReadBytesExt::read_i24::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an i32 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 0, 0, 10, 0, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_i32().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_i32().is_err(), true);
    /// ```
    fn read_i32(&mut self) -> Result<i32> {
        ReadBytesExt::read_i32::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an i64 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 0, 0, 0, 0, 0, 0, 10, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_i64().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_i64().is_err(), true);
    /// ```
    fn read_i64(&mut self) -> Result<i64> {
        ReadBytesExt::read_i64::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an optional i16 value from `self`.
    ///
    /// The value is preceeded by a bool. If the bool is true, we expect a value after it.
    /// If its false, we expect a sentinel value (0) after it.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 254, 254, 2];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_optional_i16().unwrap();
    ///
    /// assert_eq!(data, -258);
    /// assert_eq!(cursor.read_optional_i16().is_err(), true);
    ///
    /// # assert_eq!(ReadBytes::read_optional_i16(&mut Cursor::new([1, 10])).is_err(), true);
    /// ```
    fn read_optional_i16(&mut self) -> Result<i16> {
        let _ = self.read_bool()?;
        self.read_i16()
    }

    /// This function tries to read an optional i32 value from `self`.
    ///
    /// The value is preceeded by a bool. If the bool is true, we expect a value after it.
    /// If its false, we expect a sentinel value (0) after it.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 10, 0, 0, 0, 2];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_optional_i32().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_optional_i32().is_err(), true);
    ///
    /// # assert_eq!(ReadBytes::read_optional_i32(&mut Cursor::new([1, 10])).is_err(), true);
    /// ```
    fn read_optional_i32(&mut self) -> Result<i32> {
        let _ = self.read_bool()?;
        self.read_i32()
    }

    /// This function tries to read an optional i64 value from `self`.
    ///
    /// The value is preceeded by a bool. If the bool is true, we expect a value after it.
    /// If its false, we expect a sentinel value (0) after it.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 10, 0, 0, 0, 0, 0, 0, 0, 2];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_optional_i64().unwrap();
    ///
    /// assert_eq!(data, 10);
    /// assert_eq!(cursor.read_optional_i64().is_err(), true);
    ///
    /// # assert_eq!(ReadBytes::read_optional_i64(&mut Cursor::new([1, 10])).is_err(), true);
    /// ```
    fn read_optional_i64(&mut self) -> Result<i64> {
        let _ = self.read_bool()?;
        self.read_i64()
    }

    /// This function tries to read an f16 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![32, 65];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_f16().unwrap();
    ///
    /// assert_eq!(data, half::f16::from_f32(2.5625));
    /// assert_eq!(cursor.read_f16().is_err(), true);
    /// ```
    fn read_f16(&mut self) -> Result<half::f16> {
        Ok(half::f16::from_bits(self.read_u16()?))
    }

    /// This function tries to read an f32 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0, 0, 32, 65];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_f32().unwrap();
    ///
    /// assert_eq!(data, 10.0);
    /// assert_eq!(cursor.read_f32().is_err(), true);
    /// ```
    fn read_f32(&mut self) -> Result<f32> {
        ReadBytesExt::read_f32::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an f32 value encoded in a single byte from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![32];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_f32_normal_from_u8().unwrap();
    ///
    /// assert_eq!(data, -0.7490196);
    /// assert_eq!(cursor.read_f32_normal_from_u8().is_err(), true);
    /// ```
    fn read_f32_normal_from_u8(&mut self) -> Result<f32> {
        Ok(self.read_u8()? as f32 / 255.0 * 2.0 - 1.0)
    }

    /// This function tries to read an f64 value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0, 0, 0, 0, 0, 0, 36, 64];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_f64().unwrap();
    ///
    /// assert_eq!(data, 10.0);
    /// assert_eq!(cursor.read_f64().is_err(), true);
    /// ```
    fn read_f64(&mut self) -> Result<f64> {
        ReadBytesExt::read_f64::<LittleEndian>(self).map_err(From::from)
    }

    /// This function tries to read an UTF-8 String value of the provided `size` from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-8 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 97, 104, 97, 104, 97, 104, 97, 104, 97];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u8(10).unwrap();
    ///
    /// assert_eq!(data, "Wahahahaha");
    /// assert_eq!(cursor.read_string_u8(10).is_err(), true);
    /// ```
    fn read_string_u8(&mut self, size: usize) -> Result<String> {
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;
        String::from_utf8(data).map_err(From::from)
    }

    /// This function tries to read an ISO-8859-15 String value of the provided `size` from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 97, 104, 97, 255, 104, 97, 104, 97, 104, 97];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u8_iso_8859_15(11).unwrap();
    ///
    /// assert_eq!(data, "Wahaÿhahaha");
    /// assert_eq!(cursor.read_string_u8_iso_8859_15(10).is_err(), true);
    /// ```
    fn read_string_u8_iso_8859_15(&mut self, size: usize) -> Result<String> {
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;

        Ok(ISO_8859_15.decode(&data).0.to_string())
    }

    /// This function tries to read a 00-Padded UTF-8 String value of the provided `size` from `self`.
    ///
    /// Note that `size` here is the full lenght of the String, including the 00 bytes that act as padding.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-8 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 97, 104, 97, 104, 97, 0, 0, 0, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u8_0padded(10).unwrap();
    ///
    /// assert_eq!(data, "Wahaha");
    /// assert_eq!(cursor.read_string_u8_0padded(10).is_err(), true);
    /// ```
    fn read_string_u8_0padded(&mut self, size: usize) -> Result<String> {
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;

        let size_no_zeros = data.iter().position(|x| *x == 0).map_or(size, |x| x);
        String::from_utf8(data[..size_no_zeros].to_vec()).map_err(From::from)
    }

    /// This function tries to read a 00-Terminated (or NULL-Terminated) UTF-8 String value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-8 String, the last value is not 00 or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 97, 104, 97, 104, 97, 104, 97, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u8_0terminated().unwrap();
    ///
    /// assert_eq!(data, "Wahahaha");
    /// assert_eq!(cursor.read_string_u8_0terminated().is_err(), true);
    /// ```
    fn read_string_u8_0terminated(&mut self) -> Result<String> {

        // So, reads are expensive, so instead of reading byte by byte, we read a bunch of them
        // and start searching with memchr. If we can't find anything, read another bunch and try again.
        let mut buf = [0; 512];
        let mut data = vec![];
        let mut curr_pos = 0u64;
        let mut end_pos = 0u64;
        let mut found = false;

        loop {
            let read = self.read(&mut buf);
            match read {
                Ok(0) => break,
                Ok(read_bytes) => {
                    if let Some(pos) = memchr::memchr(0, &buf[..read_bytes]) {

                        // If we found a 00, get the final "read" position, the final position of the 00 byte,
                        // and mark the byte as found.
                        end_pos = curr_pos + read_bytes as u64;
                        curr_pos += pos as u64;
                        data.extend_from_slice(&buf[..pos]);
                        found = true;
                        break;
                    } else {
                        curr_pos += read_bytes as u64;
                        data.extend_from_slice(&buf);
                    }
                }

                // If there is any error, just return it.
                Err(error) => return Err(error)?,
            }
        }

        // If we exited without finding the 00 byte, return an error.
        if !found {
            return Err(RLibError::DecodingString0TeminatedNo0Error);
        }

        // Move the cursor to the end of the value, so we can continue reading.
        // -1 because we need to end after the 00 byte.
        let new_pos = (end_pos - curr_pos - 1) as i64;
        self.seek(SeekFrom::Current(-new_pos))?;

        // Get a String from it. Lossy because older games have packs with broken symbols in their paths.
        Ok(String::from_utf8_lossy(&data).to_string())
    }

    /// This function tries to read a Sized UTF-8 String value from `self`.
    ///
    /// In Sized Strings, the first two values of the data are the size in Characters of the string,
    /// followed by the String itself.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-8 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_sized_string_u8().unwrap();
    ///
    /// assert_eq!(data, "Wahahahaha");
    /// assert_eq!(cursor.read_sized_string_u8().is_err(), true);
    /// ```
    fn read_sized_string_u8(&mut self) -> Result<String> {
        if let Ok(size) = self.read_u16() {
            // TODO: check if we have to restore cursor pos on failure.
            self.read_string_u8(size as usize)
        }
        else {
            Err(RLibError::DecodingStringSizeError("UTF-8 String".to_owned()))
        }
    }

    /// This function tries to read a Sized UTF-8 String value from `self`.
    ///
    /// In these particular Sized Strings, the first four values of the data are the size in Characters of the string,
    /// followed by the String itself.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-8 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![10, 0, 0, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_sized_string_u8_u32().unwrap();
    ///
    /// assert_eq!(data, "Wahahahaha");
    /// assert_eq!(cursor.read_sized_string_u8_u32().is_err(), true);
    /// ```
    fn read_sized_string_u8_u32(&mut self) -> Result<String> {
        if let Ok(size) = self.read_u32() {
            // TODO: check if we have to restore cursor pos on failure.
            self.read_string_u8(size as usize)
        }
        else {
            Err(RLibError::DecodingStringSizeError("UTF-8 String".to_owned()))
        }
    }

    /// This function tries to read an Optional UTF-8 String value from `self`.
    ///
    /// In Optional Strings, the first byte is a boolean. If true, it's followed by a Sized String.
    /// If false, then there is no more data after the boolean.
    ///
    /// It may fail if there are not enough bytes to read the value, the first value is not a boolean,
    /// the value contains invalid, characters for an UTF-8 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_optional_string_u8().unwrap();
    ///
    /// assert_eq!(data, "Wahahahaha");
    /// assert_eq!(cursor.read_optional_string_u8().is_err(), true);
    /// ```
    fn read_optional_string_u8(&mut self) -> Result<String> {
        let is = self.read_bool()
            .map_err(|_| RLibError::DecodingOptionalStringBoolError("UTF-8 Optional String".to_owned()))?;

        if is {
            self.read_sized_string_u8()
        } else {
            Ok(String::new())
        }
    }

    /// This function tries to read an UTF-16 String value of the provided `size` (in bytes) from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-16 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u16(12).unwrap();
    ///
    /// assert_eq!(data, "Wahaha");
    /// assert_eq!(cursor.read_string_u16(12).is_err(), true);
    /// ```
    fn read_string_u16(&mut self, size: usize) -> Result<String> {
        if size % 2 == 1 {
            return Err(RLibError::DecodeUTF16UnevenInputError(size));
        }
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;

        Ok(UTF_16LE.decode(&data).0.to_string())
    }

    /// This function tries to read a 00-Padded UTF-16 String value of the provided `size` from `self`.
    ///
    /// Note that `size` here is the full lenght of the String in bytes, including the 00 bytes that act as padding.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-16 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 0, 97, 0, 104, 0, 97, 0, 104, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u16_0padded(20).unwrap();
    ///
    /// assert_eq!(data, "Wahaha");
    /// assert_eq!(cursor.read_string_u16_0padded(20).is_err(), true);
    /// ```
    fn read_string_u16_0padded(&mut self, size: usize) -> Result<String> {
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;

        let size_no_zeros = (0..size.wrapping_div(2)).position(|x| data[x * 2] == 0).map_or(size.wrapping_div(2), |x| x);
        Ok(UTF_16LE.decode(&data[..size_no_zeros * 2]).0.to_string())
    }

    /// This function tries to read a 00-Terminated (or NULL-Terminated) UTF-16 String value from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-16 String, the last value is not 00 or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![87, 00, 97, 00, 104, 00, 97, 00, 104, 00, 97, 00, 104, 00, 97, 00, 00, 00];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_u16_0terminated().unwrap();
    ///
    /// assert_eq!(data, "Wahahaha");
    /// assert_eq!(cursor.read_string_u16_0terminated().is_err(), true);
    /// ```
    fn read_string_u16_0terminated(&mut self) -> Result<String> {

        // So, reads are expensive, so instead of reading byte by byte, we read a bunch of them
        // and start searching with a chunk iterator. If we can't find anything, read another bunch and try again.
        let mut buf = [0; 512];
        let mut data = vec![];
        let mut curr_pos = 0u64;
        let mut end_pos = 0u64;
        let mut found = false;

        loop {
            let read = self.read(&mut buf);
            match read {
                Ok(0) => break,
                Ok(read_bytes) => {

                    if let Some(pos) = buf[..read_bytes].iter().chunks(2).into_iter().position(|chunk| {
                        let chunk = chunk.collect::<Vec<_>>();
                        chunk.len() == 2 && *chunk[0] == 0 && *chunk[1] == 0
                    }) {

                        // If we found a 00 00, get the final "read" position, the final position of the 00 byte,
                        // and mark the byte as found.
                        end_pos = curr_pos + read_bytes as u64;
                        curr_pos += pos as u64 * 2;
                        data.extend_from_slice(&buf[..pos * 2]);
                        found = true;
                        break;
                    } else {
                        curr_pos += read_bytes as u64;
                        data.extend_from_slice(&buf);
                    }
                }

                // If there is any error, just return it.
                Err(error) => return Err(error)?,
            }
        }

        // If we exited without finding the 00 byte, return an error.
        if !found {
            return Err(RLibError::DecodingString0TeminatedNo0Error);
        }

        // Move the cursor to the end of the value, so we can continue reading.
        // -2 because we need to end after the last 00 byte.
        let new_pos = end_pos as i64 - curr_pos as i64 - 2;
        self.seek(SeekFrom::Current(-new_pos))?;

        // Get a String from it. Lossy because older games have packs with broken symbols in their paths.
        Ok(UTF_16LE.decode(&data).0.to_string())
    }

    /// This function tries to read a Sized UTF-16 String value from `self`.
    ///
    /// In Sized Strings, the first two values of the data are the size in Characters of the string,
    /// followed by the String itself.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-16 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![4, 0, 87, 0, 97, 0, 104, 0, 97, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_sized_string_u16().unwrap();
    ///
    /// assert_eq!(data, "Waha");
    /// assert_eq!(cursor.read_sized_string_u16().is_err(), true);
    /// ```
    fn read_sized_string_u16(&mut self) -> Result<String> {
        if let Ok(size) = self.read_u16() {
            // TODO: check if we have to restore cursor pos on failure.
            self.read_string_u16(size.wrapping_mul(2) as usize)
        }
        else {
            Err(RLibError::DecodingStringSizeError("UTF-16 String".to_owned()))
        }
    }

    /// This function tries to read a Sized UTF-16 String value from `self`.
    ///
    /// In these particular Sized Strings, the first four values of the data are the size in Characters of the string,
    /// followed by the String itself.
    ///
    /// It may fail if there are not enough bytes to read the value, the value contains invalid
    /// characters for an UTF-16 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![4, 0, 0, 0, 87, 0, 97, 0, 104, 0, 97, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_sized_string_u16_u32().unwrap();
    ///
    /// assert_eq!(data, "Waha");
    /// assert_eq!(cursor.read_sized_string_u16_u32().is_err(), true);
    /// ```
    fn read_sized_string_u16_u32(&mut self) -> Result<String> {
        if let Ok(size) = self.read_u32() {
            // TODO: check if we have to restore cursor pos on failure.
            self.read_string_u16(size.wrapping_mul(2) as usize)
        }
        else {
            Err(RLibError::DecodingStringSizeError("UTF-16 String".to_owned()))
        }
    }

    /// This function tries to read an Optional UTF-16 String value from `self`.
    ///
    /// In Optional Strings, the first byte is a boolean. If true, it's followed by a Sized String.
    /// If false, then there is no more data after the boolean.
    ///
    /// It may fail if there are not enough bytes to read the value, the first value is not a boolean,
    /// the value contains invalid, characters for an UTF-16 String, or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![1, 4, 0, 87, 0, 97, 0, 104, 0, 97, 0];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_optional_string_u16().unwrap();
    ///
    /// assert_eq!(data, "Waha");
    /// assert_eq!(cursor.read_optional_string_u16().is_err(), true);
    /// ```
    fn read_optional_string_u16(&mut self) -> Result<String> {
        let is = self.read_bool()
            .map_err(|_| RLibError::DecodingOptionalStringBoolError("UTF-16 Optional String".to_owned()))?;

        if is {
            self.read_sized_string_u16()
        } else {
            Ok(String::new())
        }
    }

    /// This function tries to read a Hex-Encoded RGB Colour from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0xFF, 0x04, 0x05, 0x00];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_string_colour_rgb().unwrap();
    ///
    /// assert_eq!(data, "0504FF");
    /// assert_eq!(cursor.read_string_colour_rgb().is_err(), true);
    /// ```
    fn read_string_colour_rgb(&mut self) -> Result<String> {
        let value = self.read_u32()?;

        // Padding to 8 zeros so we don't lose the first one, then remove the last two zeros (alpha?).
        // REMEMBER, FORMAT ENCODED IS BBGGRR00.
        Ok(format!("{value:06X?}"))
    }

    /// This function tries to read a Vector of 2 u8 values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_2_u8().unwrap();
    ///
    /// assert_eq!(data, Vector2::new(10, 10));
    /// assert_eq!(cursor.read_vector_2_u8().is_err(), true);
    /// ```
    fn read_vector_2_u8(&mut self) -> Result<Vector2<u8>> {
        let x = self.read_u8()?;
        let y = self.read_u8()?;

        Ok(Vector2::new(x, y))
    }

    /// This function tries to read a Vector of 2 f32 percentage values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_2_f32_pct_from_vector_2_u8().unwrap();
    ///
    /// assert_eq!(data, Vector2::new(0.039215688, 0.039215688));
    /// assert_eq!(cursor.read_vector_2_f32_pct_from_vector_2_u8().is_err(), true);
    /// ```
    fn read_vector_2_f32_pct_from_vector_2_u8(&mut self) -> Result<Vector2<f32>> {
        let x = self.read_u8()? as f32;
        let y = self.read_u8()? as f32;

        Ok(Vector2::new(x / 255.0, y / 255.0))
    }

    /// This function tries to read a Vector of 2 f32 values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector2;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A, 0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_2_f32_from_vector_2_f16().unwrap();
    ///
    /// assert_eq!(data, Vector2::new(0.00018429756, 0.00018429756));
    /// assert_eq!(cursor.read_vector_2_f32_from_vector_2_f16().is_err(), true);
    /// ```
    fn read_vector_2_f32_from_vector_2_f16(&mut self) -> Result<Vector2<f32>> {
        let x = self.read_f16()?.to_f32();
        let y = self.read_f16()?.to_f32();

        Ok(Vector2::new(x, y))
    }

    /// This function tries to read a Vector of 4 u8 values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A, 0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_u8().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(10, 10, 10, 10));
    /// assert_eq!(cursor.read_vector_4_u8().is_err(), true);
    /// ```
    fn read_vector_4_u8(&mut self) -> Result<Vector4<u8>> {
        let x = self.read_u8()?;
        let y = self.read_u8()?;
        let z = self.read_u8()?;
        let w = self.read_u8()?;

        Ok(Vector4::new(x, y, z, w))
    }

    /// This function tries to read a Vector of 4 f32 values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![
    ///     0x00, 0x00, 0xFF, 0x3F,
    ///     0x00, 0x00, 0xFF, 0x3F,
    ///     0x00, 0x00, 0xFF, 0x3F,
    ///     0x00, 0x00, 0xFF, 0x3F
    /// ];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_f32().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(1.9921875, 1.9921875, 1.9921875, 1.9921875));
    /// assert_eq!(cursor.read_vector_4_f32().is_err(), true);
    /// ```
    fn read_vector_4_f32(&mut self) -> Result<Vector4<f32>> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        let w = self.read_f32()?;

        Ok(Vector4::new(x, y, z, w))
    }

    /// This function tries to read a Vector of 4 f32 values from a Vector of 3 f32 values from`self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![
    ///     0x00, 0x00, 0xFF, 0x3F,
    ///     0x00, 0x00, 0xFF, 0x3F,
    ///     0x00, 0x00, 0xFF, 0x3F
    /// ];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_f32_from_vec_3_f32().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(1.9921875, 1.9921875, 1.9921875, 0.0));
    /// assert_eq!(cursor.read_vector_4_f32_from_vec_3_f32().is_err(), true);
    /// ```
    fn read_vector_4_f32_from_vec_3_f32(&mut self) -> Result<Vector4<f32>> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;

        Ok(Vector4::new(x, y, z, 0.0))
    }

    /// This function tries to read a Vector of 4 f32 normalized values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A, 0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_f32_normal_from_vector_4_u8().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(-0.92156863, -0.92156863, -0.92156863, -0.92156863));
    /// assert_eq!(cursor.read_vector_4_f32_normal_from_vector_4_u8().is_err(), true);
    /// ```
    fn read_vector_4_f32_normal_from_vector_4_u8(&mut self) -> Result<Vector4<f32>> {
        let mut x = self.read_f32_normal_from_u8()?;
        let mut y = self.read_f32_normal_from_u8()?;
        let mut z = self.read_f32_normal_from_u8()?;
        let w = self.read_f32_normal_from_u8()?;

        if w > 0.0 {
            x *= w;
            y *= w;
            z *= w;
        }

        Ok(Vector4::new(x, y, z, w))
    }

    /// This function tries to read a Vector of 4 f32 percentage values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![0x0A, 0x0A, 0x0A, 0x0A];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_f32_pct_from_vector_4_u8().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(0.039215688, 0.039215688, 0.039215688, 0.039215688));
    /// assert_eq!(cursor.read_vector_4_f32_pct_from_vector_4_u8().is_err(), true);
    /// ```
    fn read_vector_4_f32_pct_from_vector_4_u8(&mut self) -> Result<Vector4<f32>> {
        let x = self.read_u8()? as f32;
        let y = self.read_u8()? as f32;
        let z = self.read_u8()? as f32;
        let w = self.read_u8()? as f32;

        Ok(Vector4::new(x / 255.0, y / 255.0, z / 255.0, w / 255.0))
    }

    /// This function tries to read a Vector of 4 f32 normalized values from `self`.
    ///
    /// It may fail if there are not enough bytes to read the value or `self` cannot be read.
    ///
    /// ```rust
    /// use nalgebra::Vector4;
    /// use std::io::Cursor;
    ///
    /// use rpfm_lib::binary::ReadBytes;
    ///
    /// let data = vec![
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F,
    ///     0x0A, 0x3F
    /// ];
    /// let mut cursor = Cursor::new(data);
    /// let data = cursor.read_vector_4_f32_normal_from_vector_4_f16().unwrap();
    ///
    /// assert_eq!(data, Vector4::new(3.096775, 3.096775, 3.096775, 1.7597656));
    /// assert_eq!(cursor.read_vector_4_f32_normal_from_vector_4_f16().is_err(), true);
    /// ```
    fn read_vector_4_f32_normal_from_vector_4_f16(&mut self) -> Result<Vector4<f32>> {
        let mut x = self.read_f16()?.to_f32();
        let mut y = self.read_f16()?.to_f32();
        let mut z = self.read_f16()?.to_f32();
        let w = self.read_f16()?.to_f32();

        if w != 0.0 {
            x *= w;
            y *= w;
            z *= w;
        }

        Ok(Vector4::new(x, y, z, w))
    }
}

// Automatic implementation for everything that implements `Read + Seek`.
impl<R: Read + Seek> ReadBytes for R {}
