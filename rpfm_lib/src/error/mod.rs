//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Error types and result handling for the RPFM library.
//!
//! This module defines [`RLibError`], a comprehensive error type that covers all possible
//! error conditions that can occur when working with Total War PackFiles and related files.
//!
//! # Error Categories
//!
//! The errors are organized into several categories:
//! - **Compression/Decompression**: Errors related to file compression operations
//! - **Encoding/Decoding**: Errors when reading or writing various file formats
//! - **File I/O**: General file reading and writing errors
//! - **Schema/Definition**: Errors related to missing or invalid schemas and definitions
//! - **Game-Specific**: Errors specific to certain Total War games or features
//! - **External Libraries**: Wrapped errors from third-party dependencies
//!
//! # Usage
//!
//! This module provides a custom [`Result`] type alias that uses [`RLibError`] as the default error type:
//!
//! ```ignore
//! use rpfm_lib::error::Result;
//!
//! fn do_something() -> Result<String> {
//!     // Returns Result<String, RLibError>
//!     Ok("success".to_string())
//! }
//! ```

use std::path::PathBuf;

use thiserror::Error;

use crate::files::{FileType, table::local::TableInMemory};

/// Custom [`Result`] type alias that uses [`RLibError`] as the default error type.
///
/// This is a convenience type alias that allows functions to return `Result<T>` instead
/// of `Result<T, RLibError>`, making function signatures cleaner throughout the codebase.
///
/// [`Result`]: std::result::Result
pub type Result<T, E = RLibError> = core::result::Result<T, E>;

/// Comprehensive error type for all RPFM library operations.
///
/// This enum covers all possible error conditions that can occur when working with
/// Total War PackFiles and related file formats. Each variant includes descriptive
/// error messages via the `#[error]` attribute from the `thiserror` crate.
///
/// # Error Display
///
/// All errors implement [`std::fmt::Display`] and provide user-friendly error messages.
/// The error messages are automatically generated from the `#[error(...)]` attributes.
#[derive(Error, Debug)]
pub enum RLibError {
    // Compression/Decompression Errors
    
    /// Error when data compression fails.
    #[error("This file's compression failed for some reason. This means this File cannot be compressed RPFM.")]
    DataCannotBeCompressed,

    /// Error when data decompression fails.
    #[error("This is a compressed file and the decompression failed for some reason. This means this File cannot be opened in RPFM.")]
    DataCannotBeDecompressed,

    // Manifest Errors

    /// Manifest file for selected game not found.
    #[error("The manifest for the Game Selected hasn't been found.")]
    ManifestFileNotFound,

    /// Error parsing game's manifest.txt file.
    #[error("Error while parsing the manifest.txt file of the game selected: {0}.")]
    ManifestFileParseError(String),

    // Binary Decoding Errors

    /// No more bytes available for decoding.
    #[error("There are no more bytes to decode in the data you provided.")]
    DecodingNotMoreBytesToDecode,

    /// Invalid byte value for boolean decoding.
    #[error("Error trying to decode \"{0}\" as boolean: invalid value.")]
    DecodingBoolError(u8),

    /// No bytes remaining for number decoding.
    #[error("Error trying to decode a byte as a number: No bytes left to decode.")]
    DecodingNoBytesLeftError,

    /// Insufficient bytes for type decoding.
    #[error("Error trying to decode an {0} value: Required bytes: {1}. Provided bytes: {2:?}.")]
    DecodingNotEnoughBytesToDecodeForType(String, usize, Option<usize>),

    /// Wrapper for integer parsing errors.
    #[error(transparent)]
    DecodeIntError(#[from] std::num::ParseIntError),

    /// Wrapper for float parsing errors.
    #[error(transparent)]
    DecodeFloatError(#[from] std::num::ParseFloatError),

    /// Wrapper for UTF-8 string conversion errors.
    #[error(transparent)]
    DecodeUTF8Error(#[from] std::string::FromUtf8Error),

    /// Wrapper for UTF-8 string slice validation errors.
    #[error(transparent)]
    DecodeUTF8StrError(#[from] std::str::Utf8Error),

    /// Wrapper for UTF-16 string conversion errors.
    #[error(transparent)]
    DecodeUTF16Error(#[from] std::string::FromUtf16Error),

    /// UTF-16 string has odd byte count.
    #[error("Error trying to decode an UTF-16 String. We expected an even amount of bytes, but instead we have {0} bytes.")]
    DecodeUTF16UnevenInputError(usize),

    /// ISO-8859-1 to UTF-8 conversion failed.
    #[error("Error trying to convert an ISO8859-1 String to an UTF-8 String: {0}.")]
    DecodeUTF8FromISO8859Error(String),

    /// String size cannot be determined.
    #[error("Error trying to decode an {0}: Not enough bytes to get his size.")]
    DecodingStringSizeError(String),

    /// Optional string missing boolean prefix.
    #[error("Error trying to decode an {0}: The first byte is not a boolean.")]
    DecodingOptionalStringBoolError(String),

    /// Null-terminated string missing terminator.
    #[error("Error trying to read an 00-Terminated String: No byte 00 found.")]
    DecodingString0TeminatedNo0Error,

    /// Padded string exceeds maximum length.
    #[error("Error trying to encode an {0}: \"{1}\" has a length of {2} chars, but his length should be less or equal than {3}.")]
    EncodingPaddedStringError(String, String, usize, usize),

    // Game Installation Errors

    /// Game install type not supported.
    #[error("The game with the key \"{0}\" is not supported for the install type \"{1}\".")]
    GameInstallTypeNotSupported(String, String),

    /// Launch command not supported for game install type.
    #[error("Launch commands for game \"{0}\", install type \"{1}\" are not currently supported.")]
    GameInstallLaunchNotSupported(String, String),

    /// Invalid boolean value string.
    #[error("Error trying to convert the following value to a bool: {0}.")]
    ParseBoolError(String),

    /// File or folder cannot be read.
    #[error("Error while trying to read the following file/folder: {0}. \
        This means that path may not be readable (permissions? other programs locking access to it?) or may not exists at all.")]
    ReadFileFolderError(String),

    // PackFile Structure Errors

    /// PackFile header corrupted or unsupported.
    #[error("The header of the Pack is incomplete, unsupported or damaged.")]
    PackHeaderNotComplete,

    /// PackFile subheader missing or corrupted.
    #[error("The subheader of the Pack is incomplete, unsupported or damaged.")]
    PackSubHeaderMissing,

    /// PackFile indexes corrupted or incomplete.
    #[error("The indexes of the Pack are incomplete, unsupported or damaged")]
    PackIndexesNotComplete,

    /// Unknown PFH file type.
    #[error("Unknown PFH File Type: {0}")]
    UnknownPFHFileType(String),

    /// Unknown PFH version.
    #[error("Unknown PFH Version: {0}")]
    UnknownPFHVersion(String),

    // File Format Errors

    /// Unknown ESF signature string.
    #[error("Unknown ESF Signature: {0}")]
    UnknownESFSignature(String),

    /// Unknown ESF signature bytes.
    #[error("Unknown ESF Signature: {0:#X} {1:#X}")]
    UnknownESFSignatureBytes(u8, u8),

    /// Unknown Empire File line type.
    #[error("Unknown EF Line Type: {0}")]
    UnknownEFLineType(String),

    /// Unknown pipe type.
    #[error("Unknown Pipe Type: {0}")]
    UnknownPipeType(String),

    /// CS2 migration not supported for this game.
    #[error("Migration to this game is not yet supported for cs2.parsed files.")]
    GameDoesntSupportCs2Migration,

    /// Text file encoding unsupported or not a text file.
    #[error("This is either not a Text File, or a Text File using an unsupported encoding")]
    DecodingTextUnsupportedEncodingOrNotATextFile,

    /// Unknown anim table version.
    #[error("This file has an unknown/unsupported version: {0}.")]
    DecodingAnimsTableUnknownVersion(i32),

    /// File is not CA_VP8 or IVF format.
    #[error("This file is neither a CA_VP8 nor an IVF file.")]
    DecodingCAVP8UnsupportedFormat,

    /// CA_VP8 frame size invalid.
    #[error("Incorrect/Unknown Frame size.")]
    DecodingCAVP8IncorrectOrUnknownFrameSize,

    // ESF Errors

    /// ESF signature not supported.
    #[error("Unsupported signature: {0:#X} {1:#X}.")]
    DecodingESFUnsupportedSignature(u8, u8),

    /// ESF data type not supported.
    #[error("Unsupported data type: {0}.")]
    DecodingESFUnsupportedDataType(u8),

    /// ESF record name not in string table.
    #[error("Record name not found: {0}.")]
    DecodingESFRecordNameNotFound(u16),

    /// ESF string not in string table.
    #[error("String not found: {0}.")]
    DecodingESFStringNotFound(u32),

    /// ESF encoding signature not supported.
    #[error("Unsupported signature: {0}.")]
    EncodingESFUnsupportedSignature(String),

    // Font File Errors

    /// Font file signature not supported.
    #[error("Unsupported signature: {0:#X?}.")]
    DecodingFontUnsupportedSignature(Vec<u8>),

    // FastBin Errors

    /// FastBin signature not supported.
    #[error("Unsupported signature: {0:#X?}.")]
    DecodingFastBinUnsupportedSignature(Vec<u8>),

    /// FastBin version not supported for decoding.
    #[error("Unsupported version {1} for type {0}.")]
    DecodingFastBinUnsupportedVersion(String, u16),

    /// FastBin version not supported for encoding.
    #[error("Unsupported version {1} for type {0}.")]
    EncodingFastBinUnsupportedVersion(String, u16),

    // RigidModel Errors

    /// RigidModel signature not supported.
    #[error("Unsupported signature: {0:#X?}.")]
    DecodingRigidModelUnsupportedSignature(Vec<u8>),

    /// RigidModel version not supported.
    #[error("Unknown rigid model version: {0}")]
    DecodingRigidModelUnsupportedVersion(u32),

    /// RigidModel material type not supported.
    #[error("Unsupported material type: {0}.")]
    DecodingRigidModelUnsupportedMaterialType(u16),

    /// RigidModel texture type unknown.
    #[error("Unsupported texture type: {0}.")]
    DecodingRigidModelUnknownTextureType(i32),

    /// RigidModel vertex format unknown.
    #[error("Unsupported vertex format: {0}.")]
    DecodingRigidModelUnknownVertexFormat(u16),

    /// RigidModel vertex format incompatible with material.
    #[error("Unsupported vertex format {0} for material {1}.")]
    DecodingRigidModelUnsupportedVertexFormatForMaterial(u16, u16),

    /// Group Formations unknown enum value.
    #[error("Unknown group formations {0} value: {1}.")]
    DecodingGroupFormationsUnknownEnumValue(String, u32),

    // Table Decoding Errors

    /// Combined colour field decoding failed.
    #[error("Error decoding combined colour.")]
    DecodingTableCombinedColour,

    // SoundBank Errors

    /// SoundBank BKHD header section missing.
    #[error("Header section not found. This shouldn't happen.")]
    SoundBankBKHDNotFound,

    /// SoundBank section type not supported.
    #[error("Unsupported section {0} found in SoundBank.")]
    SoundBankUnsupportedSectionFound(String),

    /// SoundBank object version not supported.
    #[error("Unsupported version {0} for object of type {1} found in SoundBank.")]
    SoundBankUnsupportedVersionFound(u32, String),

    /// SoundBank language ID not supported.
    #[error("Unsupported language id {0} found in SoundBank.")]
    SoundBankUnsupportedLanguageFound(u32),

    /// SoundBank object type not supported.
    #[error("Unsupported object type {0} found in SoundBank.")]
    SoundBankUnsupportedObjectTypeFound(u8),

    // Table Field Errors

    /// Table field decoding failed.
    #[error("Error trying to decode the Row {0}, Cell {1} as a {2} value: either the value is not a {2}, or there are insufficient bytes left to decode it as a {2} value.")]
    DecodingTableFieldError(u32, u32, String),

    /// Table sequence field index out of bounds.
    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: invalid ending index {2}.")]
    DecodingTableFieldSequenceIndexError(u32, u32, usize, String),

    /// Table sequence field data invalid.
    #[error("Error trying to get the data for a {3} on Row {0}, Cell {1}: {2}.")]
    DecodingTableFieldSequenceDataError(u32, u32, String, String),

    /// Table decoding incomplete with partial data.
    #[error("Error trying to decode a table: {0}. The incomplete table is: {1:#?}.")]
    DecodingTableIncomplete(String, Box<TableInMemory>),

    // Extra Data Errors

    /// Required extra decoding data missing.
    #[error("Missing extra data required to decode the file. This means the programmer messed up the code while that tries to decode files.")]
    DecodingMissingExtraData,

    /// Extra data field missing or invalid.
    #[error("Missing or invalid extra data provided: \"{0}\"")]
    DecodingMissingExtraDataField(String),

    /// File decoding not supported for selected game.
    #[error("Decoding of this file is unsupported for game: \"{0}\"")]
    DecodingUnsupportedGameSelected(String),

    // Table Encoding Errors

    /// Table row field count mismatch.
    #[error("Error while trying to save a row from a table: We expected a row with \"{0}\" fields, but we got a row with \"{1}\" fields instead.")]
    TableRowWrongFieldCount(usize, usize),

    /// Table field type mismatch.
    #[error("Error while trying to save a row from a table: We expected a field of type \"{0}\", but we got a field of type \"{1}\".")]
    EncodingTableWrongFieldType(String, String),

    // Schema/Definition Errors

    /// Table definition missing and file empty.
    #[error("There are no definitions for this specific version of the table in the Schema and the table is empty. This means this table cannot be open nor decoded.")]
    DecodingDBNoDefinitionsFoundAndEmptyFile,

    /// Table definition missing from schema.
    #[error("There are no definitions for this specific version of the table in the Schema.")]
    DecodingDBNoDefinitionsFound,

    /// File not a valid DB table.
    #[error("This is either not a DB Table, or it's a DB Table but it's corrupted.")]
    DecodingDBNotADBTable,

    /// File not a valid Loc table.
    #[error("This is either not a Loc Table, or it's a Loc Table but it's corrupted.")]
    DecodingLocNotALocTable,

    /// File not a valid Matched Combat table.
    #[error("This is either not a Matched Combat Table, or it's a Matched Combat Table but it's corrupted.")]
    DecodingMatchedCombatNotAMatchedCombatTable,

    /// File not a valid Unit Variant.
    #[error("This is either not an Unit Variant, or it's an Unit Variant but it's corrupted.")]
    DecodingUnitVariantNotAUnitVariant,

    /// File size mismatch with expected size.
    #[error("This file's reported size is '{0}' bytes, but we expected it to be '{1}' bytes. This means that the definition of the table is incorrect (only on tables, it's usually this), the decoding logic in RPFM is broken for this file, or this file is corrupted.")]
    DecodingMismatchSizeError(usize, usize),

    // Version Errors

    /// Portrait Settings version not supported.
    #[error("This file's version ({0}) is not yet supported.")]
    DecodingPortraitSettingUnsupportedVersion(usize),

    /// Generic unsupported version error.
    #[error("This file's version ({0}) is not yet supported.")]
    DecodingUnsupportedVersion(usize),

    /// Anim Fragment version not supported.
    #[error("This file's version ({0}) is not yet supported.")]
    DecodingAnimFragmentUnsupportedVersion(usize),

    /// Matched Combat version not supported.
    #[error("This file's version ({0}) is not yet supported.")]
    DecodingMatchedCombatUnsupportedVersion(usize),

    // File Type Errors

    /// Decoded data type doesn't match expected file type.
    #[error("This file is expected to be of {0} type, but the data provided is of {1} type. If you see this, 99% sure it is a bug.")]
    DecodedDataDoesNotMatchFileType(FileType, FileType),

    /// SoundPacked decoding not supported for game.
    #[error("Decoding of SoundPacked files is not supported for this game: {0}.")]
    DecodingSoundPackedUnsupportedGame(String),

    /// SoundPacked encoding not supported for game.
    #[error("Encoding of SoundPacked files is not supported for this game: {0}.")]
    EncodingSoundPackedUnsupportedGame(String),

    /// Required extra encoding data missing.
    #[error("Missing extra data required to encode the file. This means the programmer messed up the code while that tries to decode files.")]
    EncodingMissingExtraData,

    /// Invalid state participant value.
    #[error("Invalid state participant value: {0}")]
    InvalidStateParticipantValue(u32),

    // Git Repository Errors

    /// Git repository download or update failed.
    #[error("There was an error while downloading/updating the following git repository: {0}.")]
    GitErrorDownloadFromRepo(String),

    /// No updates available for git repository.
    #[error("No updates available for the following git repository: {0}.")]
    GitErrorNoUpdatesAvailable(String),

    // Lazy Loading Errors

    /// File data changed on disk during lazy loading.
    #[error("The file's data for file ({0}) has been altered on disk by another program since the last time it was accessed by us. If you see this, it means you're using lazy-loading and another program has altered the data on disk before this program loaded it to memory.

        Basically, this means your Pack got partially corrupted.

        If you see this message in a program that's not RPFM,... ask its author what to do.

        If you see this message in RPFM, your original Pack on disk should still be safe, and RPFM can recover part of the files inside the open PackFile: DB tables, Locs and any PackedFile open before this message appeared. To do that, go to 'Special Stuff' and hit 'Rescue PackFile', then choose a folder to save the clean Pack.

        That will create a Pack with only the files that were confirmed as non-corrupted, so at least you can recover their data.

        And some final words: if you intentionally opened the same Pack in two instances of RPFM and tried to save on both, that was the cause of this. No, it's not a bug in RPFM. No, I can't magically fix it. It's how lazy-loading data from disk works. If you don't like it, you can disable lazy-loading in the settings. You'll be resistant to Pack corruption, but RPFM will use a ton more RAM. So... choose your poison.

        Note: if this message appeared while adding files from a Pack, you're save. Just close the 'Add From PackFile' tab and open it again.")]
    FileSourceChanged(String),

    // Size/Limit Errors

    /// File too large for container.
    #[error("At least one of the files (`{3}`) on this {0} is too big for it. The maximum supported size for files is {1}, but your file has {2} bytes.")]
    DataTooBigForContainer(String, u64, usize, String),

    // File Operation Errors

    /// File not found in pack.
    #[error("The following file hasn't been found: {0}.")]
    FileNotFound(String),

    /// File not yet decoded.
    #[error("The following file hasn't yet been decoded: {0}.")]
    FileNotDecoded(String),

    /// File not yet cached.
    #[error("The following file hasn't yet been cached: {0}.")]
    FileNotCached(String),

    /// Reserved file operation blocked.
    #[error("Operation not allowed: reserved file detected.")]
    ReservedFiles,

    /// Empty destination path.
    #[error("Operation not allowed: destiny is blank for your file.")]
    EmptyDestiny,

    /// No packs provided to operation.
    #[error("No Packs provided.")]
    NoPacksProvided,

    /// Live export has no files to export.
    #[error("No files to export.")]
    LiveExportNoFilesToExport,

    // Build/Update Errors

    /// Startpos build error.
    #[error("{0}")]
    BuildStartposError(String),

    /// Animation IDs update error.
    #[error("{0}")]
    UpdateAnimIdsError(String),

    // SQLite Errors

    /// SQLite connection pool not initialized.
    #[error("The SQLite connection pool hasn't been initialized yet.")]
    MissingSQLitePool,

    /// Path missing filename component.
    #[error("The path {0} doesn't have an identifiable filename.")]
    PathMissingFileName(String),

    // Dependencies Cache Errors

    /// Dependencies cache not generated or outdated.
    #[error("The dependencies cache has not been generated or it's outdated and need regenerating.")]
    DependenciesCacheNotGeneratedorOutOfDate,

    /// File not found in dependencies cache.
    #[error("The file with the path {0} hasn't been found in the dependencies cache.")]
    DependenciesCacheFileNotFound(String),

    // Definition Update Errors

    /// Table already has latest definition.
    #[error("This table already has the newer definition available.")]
    NoDefinitionUpdateAvailable,

    /// Table not found in game files for comparison.
    #[error("This table cannot be found in the Game Files, so it cannot be automatically updated (yet).")]
    NoTableInGameFilesToCompare,

    // Assembly Kit Errors

    /// Assembly Kit version not supported.
    #[error("Operations over the Assembly Kit of version {0} are not currently supported.")]
    AssemblyKitUnsupportedVersion(i16),

    /// Assembly Kit folder not found or readable.
    #[error("The Assembly Kit Folder could not be read. You may need to install the Assembly Kit.")]
    AssemblyKitNotFound,

    /// Table not found in Assembly Kit.
    #[error("The table {0} was not found in the Assembly Kit.")]
    AssemblyKitTableNotFound(String),

    /// Assembly Kit table blacklisted.
    #[error("One of the Assembly Kit Tables you tried to decode has been blacklisted due to issues.")]
    AssemblyKitTableTableIgnored,

    /// Localisable fields file not found.
    #[error("The `Localisable Fields` file hasn't been found.")]
    AssemblyKitLocalisableFieldsNotFound,

    /// Relationships file not found.
    #[error("The relationships file hasn't been found.")]
    AssemblyKitExtraRelationshipsNotFound,

    /// Raw table import missing definition.
    #[error("The raw table you tried to import is missing a definition.")]
    RawTableMissingDefinition,

    // TSV Import Errors

    /// TSV import row/field error.
    #[error("This TSV file has an error in the row {0}, field {1} (both starting at 0). Please, check it and make sure the value in that field is a valid value for that column.")]
    ImportTSVIncorrectRow(usize, usize),

    /// TSV file incompatible or wrong type.
    #[error("This TSV file either belongs to another table, to a localization File, it's broken or it's incompatible with RPFM.")]
    ImportTSVWrongTypeTable,

    /// TSV file has invalid version.
    #[error("This TSV file has an invalid version value at line 1.")]
    ImportTSVInvalidVersion,

    /// TSV file missing or invalid path.
    #[error("This TSV file has an invalid or missing file path value at line 1.")]
    ImportTSVInvalidOrMissingPath,

    // File Merge Errors

    /// Merge requires multiple files.
    #[error("You need to pass more than one file to merge.")]
    RFileMergeOnlyOneFileProvided,

    /// Cannot merge files of different types.
    #[error("Merging files of different types is not supported.")]
    RFileMergeDifferentTypes,

    /// Cannot merge tables with different names.
    #[error("Merging tables with different table names is not supported.")]
    RFileMergeTablesDifferentNames,

    /// Table merge requires at least two tables.
    #[error("Merging tables needs at least two tables.")]
    RFileMergeTablesNotEnoughTablesProvided,

    /// File type doesn't support merging.
    #[error("Merging files of type {0} is not supported.")]
    RFileMergeNotSupportedForType(String),

    // Group Formation Errors

    /// Group formation block type unknown.
    #[error("Block Type {0} is not supported.")]
    GroupFormationUnknownBlockType(u32),

    // Patch Errors

    /// Cannot patch empty pack.
    #[error("This Pack is empty, so we can't patch it.")]
    PatchSiegeAIEmptyPack,

    /// No patchable files in pack.
    #[error("There are not files in this Pack that could be patched/deleted.")]
    PatchSiegeAINoPatchableFiles,

    /// No schema provided when required.
    #[error("No Schema provided.")]
    SchemaNotProvided,

    // IVF Errors

    /// Invalid subtraction during IVF processing.
    #[error("Invalid subtraction when processing an IVF file. This means the something went wrong when saving the IVF file.")]
    IVFInvalidSubstraction,

    // Steam Workshop Errors

    /// Game doesn't support Steam Workshop.
    #[error("The game {0} doesn't support the Steam Workshop.")]
    GameDoesntSupportWorkshop(String),

    /// SteamID not recognized as Total War game.
    #[error("The SteamID {0} doesn't belong to any known Total War game.")]
    SteamIDDoesntBelongToKnownGame(u64),

    // Global Search/Replace Errors

    /// Global replace requires same byte length without regex.
    #[error("You're trying to perform a Global Replace on a type that doesn't support Regex replacement and requires that both, pattern and replacement have the exact same byte length. To avoid breaking files this program doesn't allow you to do that. Either make sure both strings have the exact same byte length, don't use regex, or use a hexadecimal editor.")]
    GlobalSearchReplaceRequiresSameLengthAndNotRegex,

    /// IO error with associated path.
    #[error("Error in path: {1}. {0}")]
    IOErrorPath(Box<Self>, PathBuf),

    // Translation Errors

    /// No translation found.
    #[error("No translation could be found.")]
    TranslatorCouldNotLoadTranslation,

    // GameInfo Errors

    /// GameInfo missing from pack-reading function.
    #[error("GameInfo has not been provided to the pack-reading function when reading the pack.")]
    GameInfoMissingFromDecodingFunction,

    /// GameInfo missing from pack-saving function.
    #[error("GameInfo has not been provided to the pack-saving function when saving the pack.")]
    GameInfoMissingFromEncodingFunction,

    // External Library Error Wrappers

    /// Wrapper for [`std::io::Error`] errors.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// Wrapper for [`git2::Error`] errors.
    #[cfg(feature = "integration_git")]
    #[error(transparent)]
    GitError(#[from] git2::Error),

    /// Wrapper for [`ron::Error`] errors.
    #[error(transparent)]
    RonError(#[from] ron::Error),

    /// Wrapper for [`ron::error::SpannedError`] errors.
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),

    /// Wrapper for [`csv::Error`] errors.
    #[error(transparent)]
    CSVError(#[from] csv::Error),

    /// Wrapper for [`serde_json::Error`] errors.
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// Wrapper for [`std::array::TryFromSliceError`] errors.
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    /// Wrapper for [`std::time::SystemTimeError`] errors.
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),

    /// Wrapper for [`std::path::StripPrefixError`] errors.
    #[error(transparent)]
    StripPrefixError(#[from] std::path::StripPrefixError),

    /// Wrapper for [`r2d2::Error`] errors.
    #[cfg(feature = "integration_sqlite")]
    #[error(transparent)]
    R2D2Error(#[from] r2d2::Error),

    /// Wrapper for [`rusqlite::Error`] errors.
    #[cfg(feature = "integration_sqlite")]
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),

    /// Wrapper for [`toml::ser::Error`] errors.
    #[error(transparent)]
    TomlError(#[from] toml::ser::Error),

    /// Wrapper for [`bitcode::Error`] errors.
    #[cfg(feature = "support_error_bitcode")]
    #[error(transparent)]
    BitcodeError(#[from] bitcode::Error),

    /// Wrapper for [`serde_xml_rs::Error`] errors.
    #[cfg(feature = "integration_assembly_kit")]
    #[error(transparent)]
    XmlRsError(#[from] serde_xml_rs::Error),

    /// Wrapper for [`log::SetLoggerError`] errors.
    #[error(transparent)]
    LogError(#[from] log::SetLoggerError),

    /// Wrapper for [`lz4_flex::frame::Error`] errors.
    #[error(transparent)]
    Lz4Error(#[from] lz4_flex::frame::Error),

    /// Wrapper for [`lzma_rs::error::Error`] errors.
    #[error(transparent)]
    LzmaError(#[from] lzma_rs::error::Error),

    /// Wrapper for [`image::ImageError`] errors.
    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    /// Wrapper for [`dds::DecodingError`] errors.
    #[error(transparent)]
    DDSDecError(#[from] dds::DecodingError),

    /// Wrapper for [`dds::EncodingError`] errors.
    #[error(transparent)]
    DDSEncError(#[from] dds::EncodingError),

    /// DDS colour format not supported.
    #[error("Unsupported colour format for DDS files.")]
    DecodingDDSColourFormatUnsupported,
}
