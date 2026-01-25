//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the traits [`ReadBytes`] and [`WriteBytes`], used to read binary data
//! into known types and write them back to binary.
//!
//! These traits are automatically implemented for anything that implements [`Read`] + [`Seek`]
//! (for [`ReadBytes`]) or [`Write`] (for [`WriteBytes`]).
//!
//! # Examples
//!
//! To read data using the [`ReadBytes`] trait, we can do this:
//!
//! ```rust
//! use std::io::Cursor;
//!
//! use rpfm_lib::binary::ReadBytes;
//!
//! let data = vec![10, 0, 87, 97, 104, 97, 104, 97, 104, 97, 104, 97];
//! let mut cursor = Cursor::new(data);
//! let string = ReadBytes::read_sized_string_u8(&mut cursor).unwrap();
//! println!("{}", string);
//! ```
//!
//! And for writing data using the [`WriteBytes`] trait, we can do this:
//!
//! ```rust
//! use rpfm_lib::binary::WriteBytes;
//!
//! let mut data = vec![];
//! data.write_string_u8("Wahahahaha").unwrap();
//! println!("{:?}", data);
//! ```
//!
//! Check each trait docs for more info about all the types that can be read/written.
//!
//! # Simple types
//!
//! The most simple and common readable/writable types (always using LittleEndian) are:
//!
//! | Type            | Bytes                         | Binary Format                                   | Example | Explanation |
//! | --------------- | ----------------------------- | ----------------------------------------------- | ------- | ----------- |
//! | **[`bool`]**    | 1                             | ```00 or 01```                                  | 0/1     | Boolean value, 0 is false, 1 is true. Read with [`ReadBytes::read_bool`], write with [`WriteBytes::write_bool`]. |
//! | **[`u8`]**      | 1                             | ```05```                                        | 5       | Unsigned Integer. Read with [`ReadBytes::read_u8`], write with [`WriteBytes::write_u8`]. |
//! | **[`u16`]**     | 2                             | ```05 00```                                     | 5       | Unsigned Integer. Read with [`ReadBytes::read_u16`], write with [`WriteBytes::write_u16`]. |
//! | **u24**         | 3                             | ```05 00 00```                                  | 5       | Unsigned Integer. Read with [`ReadBytes::read_u24`], write with [`WriteBytes::write_u24`]. |
//! | **[`u32`]**     | 4                             | ```05 00 00 00```                               | 5       | Unsigned Integer. Read with [`ReadBytes::read_u32`], write with [`WriteBytes::write_u32`]. |
//! | **[`u64`]**     | 8                             | ```05 00 00 00 00 00 00 00```                   | 5       | Unsigned Integer. Read with [`ReadBytes::read_u64`], write with [`WriteBytes::write_u64`]. |
//! | **[`i8`]**      | 1                             | ```05```                                        | 5       | Signed Integer. Read with [`ReadBytes::read_i8`], write with [`WriteBytes::write_i8`]. |
//! | **[`i16`]**     | 2                             | ```05 00```                                     | 5       | Signed Integer. Read with [`ReadBytes::read_i16`], write with [`WriteBytes::write_i16`]. |
//! | **i24**         | 3                             | ```05 00 00```                                  | 5       | Signed Integer. Read with [`ReadBytes::read_i24`], write with [`WriteBytes::write_i24`]. |
//! | **[`i32`]**     | 4                             | ```05 00 00 00```                               | 5       | Signed Integer. Read with [`ReadBytes::read_i32`], write with [`WriteBytes::write_i32`]. |
//! | **[`i64`]**     | 8                             | ```05 00 00 00 00 00 00 00```                   | 5       | Signed Integer. Read with [`ReadBytes::read_i64`], write with [`WriteBytes::write_i64`]. |
//! | **[`f32`]**     | 4                             | ```00 00 80 3F```                               | 1.0     | Floating Point Value. Read with [`ReadBytes::read_f32`], write with [`WriteBytes::write_f32`]. |
//! | **[`f64`]**     | 8                             | ```00 00 00 00 00 00 F0 3F```                   | 1.0     | Floating Point Value. Read with [`ReadBytes::read_f64`], write with [`WriteBytes::write_f64`]. |
//! | **StringU8**    | 2 (Length, u16) + Length.     | ```06 00 48 65 6C 6C 6F 77```                   | Hellow  | String encoded in UTF-8, with the first two bytes being the length of the String, in bytes. Read with [`ReadBytes::read_sized_string_u8`], write with [`WriteBytes::write_sized_string_u8`]. |
//! | **StringU16**   | 2 (Length, u16) + Length * 2. | ```06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00``` | Hellow  | String encoded in UTF-16, with the first two bytes being the length of the String, in characters. Read with [`ReadBytes::read_sized_string_u16`], write with [`WriteBytes::write_sized_string_u16`]. |
//!
//! # Complex types
//!
//! Apart of these, there are a few more complex types supported:
//!
//! | Type                      | Bytes                                     | Binary Format                                         | Example             | Explanation |
//! | ------------------------- | ----------------------------------------- | ----------------------------------------------------- | ------------------- | ----------- |
//! | **Optional i16**          | 3                                         | ```01 05 00```                                        | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. Read with [`ReadBytes::read_optional_i16`], write with [`WriteBytes::write_optional_i16`]. |
//! | **Optional i32**          | 5                                         | ```01 05 00 00 00```                                  | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. Read with [`ReadBytes::read_optional_i32`], write with [`WriteBytes::write_optional_i32`]. |
//! | **Optional i64**          | 9                                         | ```01 05 00 00 00 00 00 00 00```                      | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. Read with [`ReadBytes::read_optional_i64`], write with [`WriteBytes::write_optional_i64`]. |
//! | **OptionalStringU8**      | 1 (bool) + 2 (Length, u16) + Length.      | ```01 06 00 48 65 6C 6C 6F 77```                      | Hellow              | A StringU8 with a boolean before. If the boolean is false, there's no string after it. Read with [`ReadBytes::read_optional_string_u8`], write with [`WriteBytes::write_optional_string_u8`]. |
//! | **OptionalStringU16**     | 1 (bool) + 2 (Length, u16) + Length.      | ```01 06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00```    | Hellow              | A StringU16 with a boolean before. If the boolean is false, there's no string after it. Read with [`ReadBytes::read_optional_string_u16`], write with [`WriteBytes::write_optional_string_u16`]. |
//! | **StringU8 0-Terminated** | Variable length, until there's a 00 byte. | ```48 65 6C 6C 6F 77 00```                            | Hellow              | A StringU8 without a size before it. The string just continues until a 00 byte is found. Read with [`ReadBytes::read_string_u8_0terminated`], write with [`WriteBytes::write_string_u8_0terminated`]. |
//! | **StringU8 0-Padded**     | Fixed length, depends on the string.      | ```48 65 6C 6C 6F 77 00 00```                         | Hellow (max size 8) | A StringU8 without a size before it. Once the string ends, it continues with 00 until it reaches a specific length (usually X*2). Read with [`ReadBytes::read_string_u8_0padded`], write with [`WriteBytes::write_string_u8_0padded`]. |
//! | **StringU16 0-Padded**    | Fixed length, depends on the string.      | ```48 00 65 00 6C 00 6C 00 6F 00 77 00 00 00 00 00``` | Hellow (max size 8) | Same as the StringU8 0-Padded, but each character is 2 bytes. Once the string ends, it continues with 00 00 until it reaches a specific length (usually X*2). Read with [`ReadBytes::read_string_u16_0padded`], write with [`WriteBytes::write_string_u16_0padded`]. |
//! | **RGB**                   | 4                                         | ```33 44 55 00```                                     | #554433             | RGB Colour, written as Hex Values with the format BBGGRR00. Read with [`ReadBytes::read_string_colour_rgb`], write with [`WriteBytes::write_string_colour_rgb`]. |
//!
//! There are some even more complex types used by specific files, like ISO-8859-1 Strings or Number Packs.
//! Those are explained in their respective file's documentation.
//!
//! [`ReadBytes`]: reader::ReadBytes
//! [`WriteBytes`]: writer::WriteBytes
//! [`Cursor`]: std::io::Cursor
//! [`Read`]: std::io::Read
//! [`Seek`]: std::io::Seek
//! [`Write`]: std::io::Write
//! [`File`]: std::fs::File

mod reader;
mod writer;

pub use self::reader::ReadBytes;
pub use self::writer::WriteBytes;

#[cfg(test)] mod reader_test;
#[cfg(test)] mod writer_test;
