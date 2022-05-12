//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the traits [`Decoder`] and [`Encoder`], used to decode binary data
//! into usable data and encode it back to binary.
//!
//! # Simple types
//!
//! The most simple and common decodeable/encodeable types (always using LittleEndian) are:
//!
//! | Type | Bytes | Binary Format | Example | Explanation |
//! | ---- | ----- | ------------- | ------- | ----------- |
//! | **[bool]** | 1   | ```00 or 01```      | 0/1     | Boolean value, 0 is false, 1 is true. |
//! | **[u8]**  | 1    | ```05```            | 5       | Unsigned Integer. |
//! | **[u16]** | 2    | ```05 00```         | 5       | Unsigned Integer. |
//! | **u24**  | 3    | ```05 00 00```      | 5       | Unsigned Integer. |
//! | **[u32]** | 4    | ```05 00 00 00```   | 5       | Unsigned Integer. |
//! | **[u64]** | 8    | ```05 00 00 00 00 00 00 00``` | 5 | Unsigned Integer. |
//! | **[i8]**  | 1    | ```05```            | 5       | Signed Integer. |
//! | **[i16]** | 2    | ```05 00```         | 5       | Signed Integer. |
//! | **i24**  | 3    | ```05 00 00```      | 5       | Signed Integer. |
//! | **[i32]** | 4    | ```05 00 00 00```   | 5       | Signed Integer. |
//! | **[i64]** | 8    | ```05 00 00 00 00 00 00 00``` | 5 | Signed Integer. |
//! | **[f32]** | 4    | ```00 00 80 3F```   | 1.0       | Floating Point Value. |
//! | **[f64]** | 8    | ```00 00 00 00 00 00 F0 3F``` | 1.0 | Floating Point Value. |
//! | **StringU8** | 2 (Lenght, u16) + Lenght. | ```06 00 48 65 6C 6C 6F 77``` | Hellow | String encoded in UTF-8, with the first two bytes being the length of the String, in bytes. |
//! | **StringU16** | 2 (Lenght, u16) + Lenght * 2. | ```06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00``` | Hellow | String encoded in UTF-16, with the first two bytes being the length of the String, in characters. |
//!
//! # Complex types
//!
//! Apart of these, there are a few more complex types supported:
//!
//! | Type | Bytes | Binary Format | Example | Explanation |
//! | ---- | ----- | ------------- | ------- | ----------- |
//! | **Optional i16** | 3    | ```01 05 00```         | 5       | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **Optional i32** | 5    | ```01 05 00 00 00```   | 5       | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **Optional i64** | 9    | ```01 05 00 00 00 00 00 00 00``` | 5 | Signed Integer with a boolean before. If the boolean is false, the value following it is always 0. |
//! | **RGB**  | 4     | ```33 44 55 00```   | #554433 | RGB Colour, encoded as Hex Values with the format BBGGRR00. |
//! | **OptionalStringU8** | 1 (bool) + 2 (Lenght, u16) + Lenght. | ```01 06 00 48 65 6C 6C 6F 77``` | Hellow | A StringU8 with a boolean before. If the boolean is false, there's no string after it. |
//! | **OptionalStringU16** | 1 (bool) + 2 (Lenght, u16) + Lenght. | ```01 06 00 48 00 65 00 6C 00 6C 00 6F 00 77 00``` | Hellow | A StringU16 with a boolean before. If the boolean is false, there's no string after it. |
//! | **StringU8 0-Padded** | Fixed lenght, depends on the string | ```48 65 6C 6C 6F 77 00 00``` | Hellow (max size 8) | A StringU8 without a size before it. Once the string ends, it continues with 00 until it reaches a specificic length (usually X*2). |
//! | **StringU8 0-Terminated** | Variable lenght, until there's a 00 byte. | ```48 65 6C 6C 6F 77 00``` | Hellow | A StringU8 without a size before it. The string just continues until a 00 byte is found. |
//! | **StringU16 0-Padded** | Fixed lenght, depends on the string | ```48 00 65 00 6C 00 6C 00 6F 00 77 00 00 00 00 00``` | Hellow (max size 8) | Same as the StringU8 0-Padded, but each character is 2 bytes. Once the string ends, it continues with 00 00 until it reaches a specificic length (usually X*2). |
//!
//! There are some even more complex types used by specific files, like ISO-8859-1 Strings or Number Packs.
//! Those are explained in their respective file's documentation.
//!
//! [`Decoder`]: decoder::Decoder
//! [`Encoder`]: encoder::Encoder

pub mod decoder;
pub mod encoder;

#[cfg(test)] mod decoder_test;
#[cfg(test)] mod encoder_test;
