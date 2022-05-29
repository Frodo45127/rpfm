//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains all kind of errors used inside this crate.
//!
//! Not much to say appart of that, really.

use thiserror::Error;

/// Custom `Result` type, to always return our custom error.
pub type Result<T, E = RLibError> = core::result::Result<T, E>;

/// Custom error type for the lib.
#[derive(Error, Debug)]
pub enum RLibError {
    #[error("This is a compressed file and the decompression failed for some reason. This means this File cannot be opened in RPFM.")]
    DataCannotBeDecompressed,

    #[error("The manifest for the Game Selected hasn't been found.")]
    ManifestFileNotFound,

    #[error("Error while parsing the manifest.txt file of the game selected: {0}.")]
    ManifestFileParseError(String),

    #[error("There are no more bytes to decode in the data you provided.")]
    DecodingNotMoreBytesToDecode,

    #[error("Error trying to decode \"{0}\" as boolean: invalid value.")]
    DecodingBoolError(u8),

    #[error("Error trying to decode a byte as a number: No bytes left to decode.")]
    DecodingNoBytesLeftError,

    #[error("Error trying to decode an {0} value: Required bytes: {1}. Provided bytes: {2:?}.")]
    DecodingNotEnoughBytesToDecodeForType(String, usize, Option<usize>),

    #[error(transparent)]
    DecodeIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    DecodeUTF8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    DecodeUTF8StrError(#[from] std::str::Utf8Error),

    #[error(transparent)]
    DecodeUTF16Error(#[from] std::string::FromUtf16Error),

    #[error(transparent)]
    DecodeCharUTF16Error(#[from] std::char::DecodeUtf16Error),

    #[error("Error trying to convert an ISO8859-1 String to an UTF-8 String: {0}.")]
    DecodeUTF8FromISO8859Error(String),

    #[error("Error trying to decode an {0}: Not enough bytes to get his size.")]
    DecodingStringSizeError(String),

    #[error("Error trying to decode an {0}: The first byte is not a boolean.")]
    DecodingOptionalStringBoolError(String),

    #[error("Error trying to read an 00-Terminated String: No byte 00 found.")]
    DecodingString0TeminatedNo0Error,

    #[error("Error trying to convert an UTF-8 String to an ISO8859-1 String: {0}.")]
    EncodeUTF8ToISO8859Error(String),

    #[error("Error trying to encode an {0}: \"{1}\" has a length of {2} chars, but his length should be less or equal than {3}.")]
    EncodingPaddedStringError(String, String, usize, usize),

    #[error("The game with the key \"{0}\" is not supported for the install type \"{1}\".")]
    GameInstallTypeNotSupported(String, String),

    #[error("Launch commands for game \"{0}\", install type \"{1}\" are not currently supported.")]
    GameInstallLaunchNotSupported(String, String),

    #[error("Error trying to convert the following value to a bool: {0}.")]
    ParseBoolError(String),

    #[error("Error while trying to read the following file/folder: {0}. \
        This means that path may not be readable (permissions? other programs locking access to it?) or may not exists at all.")]
    ReadFileFolderError(String),

    #[error("The header of the Pack is incomplete, unsupported or damaged.")]
    PackFileHeaderNotComplete,

    #[error("The indexes of the Pack are incomplete, unsupported or damaged")]
    PackFileIndexesNotComplete,

    #[error("Unknown PFH File Type: {0}")]
    UnknownPFHFileType(String),

    #[error("Unknown PFH Version: {0}")]
    UnknownPFHVersion(String),

    #[error("This is either not a Text File, or a Text File using an unsupported encoding")]
    DecodingTextUnsupportedEncodingOrNotATextFile,

    #[error("This file is neither a CA_VP8 nor an IVF file.")]
    DecodingCAVP8UnsupportedFormat,

    #[error("Incorrect/Unknown Frame size.")]
    DecodingCAVP8IncorrectOrUnknownFrameSize,

    #[error("Unsupported signature: {0:#X}{1:#X}.")]
    DecodingESFUnsupportedSignature(u8, u8),

    #[error("Unsupported data type: {0}.")]
    DecodingESFUnsupportedDataType(u8),

    #[error("Record name not found: {0}.")]
    DecodingESFRecordNameNotFound(u16),

    #[error("String not found: {0}.")]
    DecodingESFStringNotFound(u32),

    #[error("Error decoding combined colour.")]
    DecodingTableCombinedColour,

    #[error("Error trying to decode the Row {0}, Cell {1} as a {2} value: either the value is not a {2}, or there are insufficient bytes left to decode it as a {2} value.")]
    DecodingTableFieldError(u32, u32, String),

    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: invalid ending index {2}.")]
    DecodingTableFieldSequenceIndexError(u32, u32, usize, String),

    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: {2}.")]
    DecodingTableFieldSequenceDataError(u32, u32, String, String),

    #[error("Missing extra data required to decode the file. This means the programmer messed up the code while that tries to decode files.")]
    DecodingMissingExtraData,

    #[error("There are no definitions for this specific version of the table in the Schema and the table is empty. This means this table cannot be open nor decoded.")]
    DecodingDBNoDefinitionsFoundAndEmptyFile,

    #[error("There are no definitions for this specific version of the table in the Schema.")]
    DecodingDBNoDefinitionsFound,

    #[error("This is either not a DB Table, or it's a DB Table but it's corrupted.")]
    DecodingDBNotADBTable,

    #[error("This is either not a Loc Table, or it's a Loc Table but it's corrupted.")]
    DecodingLocNotALocTable,

    #[error("This is either not an Unit Variant, or it's an Unit Variant but it's corrupted.")]
    DecodingUnitVariantNotAUnitVariant,

    #[error("This file's reported size is '{0}' bytes, but we expected it to be '{1}' bytes. This means that the definition of the table is incorrect (only on tables, it's usually this), the decoding logic in RPFM is broken for this file, or this file is corrupted.")]
    DecodingMismatchSizeError(usize, usize),

    #[error("Missing extra data required to encode the file. This means the programmer messed up the code while that tries to decode files.")]
    EncodingMissingExtraData,

    #[error("There was an error while downloading/updating the following git repository: {0}.")]
    GitErrorDownloadFromRepo(String),

    #[error("No updates available for the following git repository: {0}.")]
    GitErrorNoUpdatesAvailable(String),

    #[error("The file's data has been altered on disk by another program since the last time it was accessed by us.")]
    FileSourceChanged,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    RLoggingError(#[from] rpfm_logging::error::RLoggingError),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Represents all other cases of `git2::Error`.
    #[error(transparent)]
    GitError(#[from] git2::Error),

    /// Represents all other cases of `ron::Error`.
    #[error(transparent)]
    RonError(#[from] ron::Error),

    /// Represents all other cases of `csv::Error`.
    #[error(transparent)]
    CSVError(#[from] csv::Error),

    /// Represents all other cases of `serde_json::Error`.
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// Represents all other cases of `std::array::TryFromSliceError`.
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    /// Represents all other cases of `std::time::SystemTimeError`.
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),

    /// Represents all other cases of `r2d2::Error`.
    #[error(transparent)]
    R2D2Error(#[from] r2d2::Error),
}
