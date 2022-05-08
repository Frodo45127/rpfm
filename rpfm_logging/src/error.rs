//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with the `Decoder` trait, to decode bytes to readable data.

This module contains the `Decoder` trait and his implementation for `&[u8]`. This trait allow us
to safely (yes, it covers your `index-out-of-bounds` bugs) decode any type of data contained within
a PackFile/PackedFile.

Note: If you change anything from here, remember to update the `decoder_test.rs` file for it.
!*/

use thiserror::Error;

//---------------------------------------------------------------------------//
//                      `Decoder` Trait Definition
//---------------------------------------------------------------------------//

pub type Result<T, E = RLoggingError> = core::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum RLoggingError {

    /// Represents all other cases of `std::time::SystemTimeError`.
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    LogError(#[from] log::SetLoggerError),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    TomlError(#[from] toml::ser::Error),
}
