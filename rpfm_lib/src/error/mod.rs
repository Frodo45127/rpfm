//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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

use std::path::PathBuf;

use thiserror::Error;

use crate::files::{FileType, table::Table};

/// Custom `Result` type, to always return our custom error.
pub type Result<T, E = RLibError> = core::result::Result<T, E>;

/// Custom error type for the lib.
#[derive(Error, Debug)]
pub enum RLibError {
    #[error("This file's compression failed for some reason. This means this File cannot be compressed RPFM.")]
    DataCannotBeCompressed,

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
    DecodeFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    DecodeUTF8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    DecodeUTF8StrError(#[from] std::str::Utf8Error),

    #[error(transparent)]
    DecodeUTF16Error(#[from] std::string::FromUtf16Error),

    #[error("Error trying to decode an UTF-16 String. We expected an even amount of bytes, but instead we have {0} bytes.")]
    DecodeUTF16UnevenInputError(usize),

    #[error("Error trying to convert an ISO8859-1 String to an UTF-8 String: {0}.")]
    DecodeUTF8FromISO8859Error(String),

    #[error("Error trying to decode an {0}: Not enough bytes to get his size.")]
    DecodingStringSizeError(String),

    #[error("Error trying to decode an {0}: The first byte is not a boolean.")]
    DecodingOptionalStringBoolError(String),

    #[error("Error trying to read an 00-Terminated String: No byte 00 found.")]
    DecodingString0TeminatedNo0Error,

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
    PackHeaderNotComplete,

    #[error("The subheader of the Pack is incomplete, unsupported or damaged.")]
    PackSubHeaderMissing,

    #[error("The indexes of the Pack are incomplete, unsupported or damaged")]
    PackIndexesNotComplete,

    #[error("Unknown PFH File Type: {0}")]
    UnknownPFHFileType(String),

    #[error("Unknown PFH Version: {0}")]
    UnknownPFHVersion(String),

    #[error("Unknown ESF Signature: {0}")]
    UnknownESFSignature(String),

    #[error("This is either not a Text File, or a Text File using an unsupported encoding")]
    DecodingTextUnsupportedEncodingOrNotATextFile,

    #[error("This file has an unknown/unsupported version: {0}.")]
    DecodingAnimsTableUnknownVersion(i32),

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

    #[error("Unsupported signature: {0}.")]
    EncodingESFUnsupportedSignature(String),

    #[error("Unsupported signature: {0:#X?}.")]
    DecodingFastBinUnsupportedSignature(Vec<u8>),

    #[error("Unsupported version {1} for type {0}.")]
    DecodingFastBinUnsupportedVersion(String, u16),

    #[error("Unsupported version {1} for type {0}.")]
    EncodingFastBinUnsupportedVersion(String, u16),

    #[error("Error decoding combined colour.")]
    DecodingTableCombinedColour,

    #[error("Header section not found. This shouldn't happen.")]
    SoundBankBKHDNotFound,

    #[error("Unsupported section {0} found in SoundBank.")]
    SoundBankUnsupportedSectionFound(String),

    #[error("Unsupported version {0} for object of type {1} found in SoundBank.")]
    SoundBankUnsupportedVersionFound(u32, String),

    #[error("Unsupported language id {0} found in SoundBank.")]
    SoundBankUnsupportedLanguageFound(u32),

    #[error("Unsupported object type {0} found in SoundBank.")]
    SoundBankUnsupportedObjectTypeFound(u8),

    #[error("Error trying to decode the Row {0}, Cell {1} as a {2} value: either the value is not a {2}, or there are insufficient bytes left to decode it as a {2} value.")]
    DecodingTableFieldError(u32, u32, String),

    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: invalid ending index {2}.")]
    DecodingTableFieldSequenceIndexError(u32, u32, usize, String),

    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: {2}.")]
    DecodingTableFieldSequenceDataError(u32, u32, String, String),

    #[error("Error trying to decode a table: {0}. The incomplete table is: {1:#?}.")]
    DecodingTableIncomplete(String, Table),

    #[error("Missing extra data required to decode the file. This means the programmer messed up the code while that tries to decode files.")]
    DecodingMissingExtraData,

    #[error("Missing or invalid extra data provided: \"{0}\"")]
    DecodingMissingExtraDataField(String),

    #[error("Error while trying to save a row from a table: We expected a row with \"{0}\" fields, but we got a row with \"{1}\" fields instead.")]
    TableRowWrongFieldCount(usize, usize),

    #[error("Error while trying to save a row from a table: We expected a field of type \"{0}\", but we got a field of type \"{1}\".")]
    EncodingTableWrongFieldType(String, String),

    #[error("There are no definitions for this specific version of the table in the Schema and the table is empty. This means this table cannot be open nor decoded.")]
    DecodingDBNoDefinitionsFoundAndEmptyFile,

    #[error("There are no definitions for this specific version of the table in the Schema.")]
    DecodingDBNoDefinitionsFound,

    #[error("This is either not a DB Table, or it's a DB Table but it's corrupted.")]
    DecodingDBNotADBTable,

    #[error("This is either not a Loc Table, or it's a Loc Table but it's corrupted.")]
    DecodingLocNotALocTable,

    #[error("This is either not a Matched Combat Table, or it's a Matched Combat Table but it's corrupted.")]
    DecodingMatchedCombatNotAMatchedCombatTable,

    #[error("This is either not an Unit Variant, or it's an Unit Variant but it's corrupted.")]
    DecodingUnitVariantNotAUnitVariant,

    #[error("This file's reported size is '{0}' bytes, but we expected it to be '{1}' bytes. This means that the definition of the table is incorrect (only on tables, it's usually this), the decoding logic in RPFM is broken for this file, or this file is corrupted.")]
    DecodingMismatchSizeError(usize, usize),

    #[error("This file's version ({0}) is not yet supported.")]
    DecodingPortraitSettingUnsupportedVersion(usize),

    #[error("This file's version ({0}) is not yet supported.")]
    DecodingAnimFragmentUnsupportedVersion(usize),

    #[error("This file's version ({0}) is not yet supported.")]
    DecodingMatchedCombatUnsupportedVersion(usize),

    #[error("This file is expected to be of {0} type, but the data provided is of {1} type. If you see this, 99% sure it is a bug.")]
    DecodedDataDoesNotMatchFileType(FileType, FileType),

    #[error("Missing extra data required to encode the file. This means the programmer messed up the code while that tries to decode files.")]
    EncodingMissingExtraData,

    #[error("Invalid state participant value: {0}")]
    InvalidStateParticipantValue(u32),

    #[error("There was an error while downloading/updating the following git repository: {0}.")]
    GitErrorDownloadFromRepo(String),

    #[error("No updates available for the following git repository: {0}.")]
    GitErrorNoUpdatesAvailable(String),

    #[error("The file's data has been altered on disk by another program since the last time it was accessed by us.")]
    FileSourceChanged,

    #[error("At least one of the files (`{3}`) on this {0} is too big for it. The maximum supported size for files is {1}, but your file has {2} bytes.")]
    DataTooBigForContainer(String, u64, usize, String),

    #[error("The following file hasn't been found: {0}.")]
    FileNotFound(String),

    #[error("The following file hasn't yet been decoded: {0}.")]
    FileNotDecoded(String),

    #[error("The following file hasn't yet been cached: {0}.")]
    FileNotCached(String),

    #[error("Operation not allowed: reserved file detected.")]
    ReservedFiles,

    #[error("Operation not allowed: destiny is blank for your file.")]
    EmptyDestiny,

    #[error("No Packs provided.")]
    NoPacksProvided,

    #[error("The SQLite connection pool hasn't been initialized yet.")]
    MissingSQLitePool,

    #[error("The path {0} doesn't have an identifiable filename.")]
    PathMissingFileName(String),

    #[error("The dependencies cache has not been generated or it's outdated and need regenerating.")]
    DependenciesCacheNotGeneratedorOutOfDate,

    #[error("The file with the path {0} hasn't been found in the dependencies cache.")]
    DependenciesCacheFileNotFound(String),

    #[error("This table already has the newer definition available.")]
    NoDefinitionUpdateAvailable,

    #[error("This table cannot be found in the Game Files, so it cannot be automatically updated (yet).")]
    NoTableInGameFilesToCompare,

    #[error("Operations over the Assembly Kit of version {0} are not currently supported.")]
    AssemblyKitUnsupportedVersion(i16),

    #[error("The Assembly Kit Folder could not be read. You may need to install the Assembly Kit.")]
    AssemblyKitNotFound,

    #[error("One of the Assembly Kit Tables you tried to decode has been blacklisted due to issues.")]
    AssemblyKitTableTableIgnored,

    #[error("The `Localisable Fields` file hasn't been found.")]
    AssemblyKitLocalisableFieldsNotFound,

    #[error("The raw table you tried to import is missing a definition.")]
    RawTableMissingDefinition,

    #[error("This TSV file has an error in the row {0}, field {1} (both starting at 0). Please, check it and make sure the value in that field is a valid value for that column.")]
    ImportTSVIncorrectRow(usize, usize),

    #[error("This TSV file either belongs to another table, to a localisation File, it's broken or it's incompatible with RPFM.")]
    ImportTSVWrongTypeTable,

    #[error("This TSV file has an invalid version value at line 1.")]
    ImportTSVInvalidVersion,

    #[error("This TSV file has an invalid or missing file path value at line 1.")]
    ImportTSVInvalidOrMissingPath,

    #[error("You need to pass more than one file to merge.")]
    RFileMergeOnlyOneFileProvided,

    #[error("Merging files of different types is not supported.")]
    RFileMergeDifferentTypes,

    #[error("Merging tables with different table names is not supported.")]
    RFileMergeTablesDifferentNames,

    #[error("Merging files of type {0} is not supported.")]
    RFileMergeNotSupportedForType(String),

    #[error("This Pack is empty, so we can't patch it.")]
    PatchSiegeAIEmptyPack,

    #[error("There are not files in this Pack that could be patched/deleted.")]
    PatchSiegeAINoPatchableFiles,

    #[error("You're trying to perform a Global Replace on a type that doesn't support Regex replacement and requires that both, pattern and replacement have the exact same byte lenght. To avoid breaking files this program doesn't allow you to do that. Either make sure both strings have the exact same byte lenght, don't use regex, or use a hexadecimal editor.")]
    GlobalSearchReplaceRequiresSameLenghtAndNotRegex,

    #[error("Error in path: {1}. {0}")]
    IOErrorPath(Box<Self>, PathBuf),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Represents all other cases of `git2::Error`.
    #[cfg(feature = "integration_git")]
    #[error(transparent)]
    GitError(#[from] git2::Error),

    /// Represents all other cases of `ron::Error`.
    #[error(transparent)]
    RonError(#[from] ron::Error),

    /// Represents all other cases of `ron::error::SpannedError`.
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),

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

    /// Represents all other cases of `std::path::StripPrefixError`.
    #[error(transparent)]
    StripPrefixError(#[from] std::path::StripPrefixError),

    /// Represents all other cases of `toml::ser::Error`.
    #[error(transparent)]
    TomlError(#[from] toml::ser::Error),

    /// Represents all other cases of `bincode::Error`.
    #[cfg(feature = "support_error_bincode")]
    #[error(transparent)]
    BindcodeError(#[from] bincode::Error),

    /// Represents all other cases of `serde_xml_rs::Error`.
    #[cfg(feature = "integration_assembly_kit")]
    #[error(transparent)]
    XmlRsError(#[from] serde_xml_rs::Error),

    /// Represents all other cases of `log::SetLoggerError`.
    #[cfg(feature = "integration_log")]
    #[error(transparent)]
    LogError(#[from] log::SetLoggerError),
}
