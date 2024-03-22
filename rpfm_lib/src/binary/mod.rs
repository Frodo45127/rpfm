//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
//! | Type          | Bytes                         | Binary Format                                   | Example | Explanation |
//! | ------------- | ----------------------------- | ----------------------------------------------- | ------- | ----------- |
//! | **[bool]**    | 1                             | ```00 or 01```                                  | 0/1     | Boolean value, 0 is false, 1 is true. |
//! | **[u8]**      | 1                             | ```05```                                        | 5       | Unsigned Integer. |
//! | **[u16]**     | 2                             | ```05 00```                                     | 5       | Unsigned Integer. |
//! | **u24**       | 3                             | ```05 00 00```                                  | 5       | Unsigned Integer. |
//! | **[u32]**     | 4                             | ```05 00 00 00```                               | 5       | Unsigned Integer. |
//! | **[u64]**     | 8                             | ```05 00 00 00 00 00 00 00```                   | 5       | Unsigned Integer. |
//! | **[i8]**      | 1                             | ```05```                                        | 5       | Signed Integer. |
//! | **[i16]**     | 2                             | ```05 00```                                     | 5       | Signed Integer. |
//! | **i24**       | 3                             | ```05 00 00```                                  | 5       | Signed Integer. |
//! | **[i32]**     | 4                             | ```05 00 00 00```                               | 5       | Signed Integer. |
//! | **[i64]**     | 8                             | ```05 00 00 00 00 00 00 00```                   | 5       | Signed Integer. |
//! | **[f32]**     | 4                             | ```00 00 80 3F```                               | 1.0     | Floating Point Value. |
//! | **[f64]**     | 8                             | ```00 00 00 00 00 00 F0 3F```                   | 1.0     | Floating Point Value. |
//! | **StringU8**  | 2 (Lenght, u16) + Lenght.     | ```06 00 48 65 6C 6C 6F 77```                   | Hellow  | String encoded in UTF-8, with the first two bytes being the length of the String, in bytes. |
//! | **StringU16** | 2 (Lenght, u16) + Lenght * 2. | ```06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00``` | Hellow  | String encoded in UTF-16, with the first two bytes being the length of the String, in characters. |
//!
//! # Complex types
//!
//! Apart of these, there are a few more complex types supported:
//!
//! | Type                      | Bytes                                     | Binary Format                                         | Example             | Explanation |
//! | ------------------------- | ----------------------------------------- | ----------------------------------------------------- | ------------------- | ----------- |
//! | **Optional i16**          | 3                                         | ```01 05 00```                                        | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **Optional i32**          | 5                                         | ```01 05 00 00 00```                                  | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **Optional i64**          | 9                                         | ```01 05 00 00 00 00 00 00 00```                      | 5                   | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **OptionalStringU8**      | 1 (bool) + 2 (Lenght, u16) + Lenght.      | ```01 06 00 48 65 6C 6C 6F 77```                      | Hellow              | A StringU8 with a boolean before. If the boolean is false, there's no string after it. |
//! | **OptionalStringU16**     | 1 (bool) + 2 (Lenght, u16) + Lenght.      | ```01 06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00```    | Hellow              | A StringU16 with a boolean before. If the boolean is false, there's no string after it. |
//! | **StringU8 0-Terminated** | Variable lenght, until there's a 00 byte. | ```48 65 6C 6C 6F 77 00```                            | Hellow              | A StringU8 without a size before it. The string just continues until a 00 byte is found. |
//! | **StringU8 0-Padded**     | Fixed lenght, depends on the string.      | ```48 65 6C 6C 6F 77 00 00```                         | Hellow (max size 8) | A StringU8 without a size before it. Once the string ends, it continues with 00 until it reaches a specific length (usually X*2). |
//! | **StringU16 0-Padded**    | Fixed lenght, depends on the string.      | ```48 00 65 00 6C 00 6C 00 6F 00 77 00 00 00 00 00``` | Hellow (max size 8) | Same as the StringU8 0-Padded, but each character is 2 bytes. Once the string ends, it continues with 00 00 until it reaches a specific length (usually X*2). |
//! | **RGB**                   | 4                                         | ```33 44 55 00```                                     | #554433             | RGB Colour, written as Hex Values with the format BBGGRR00. |
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

// These constants are needed to work with LEB_128 encoded numbers.
pub const LEB128_CONTROL_BIT: u8 = 0b10000000;
pub const LEB128_SIGNED_MAX: u8 = 0b00111111;
pub const LEB128_UNSIGNED_MAX: u8 = 0b01111111;
pub const U32_BITS: u32 = 32;
