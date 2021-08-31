//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This crate is the old `Error` module of RPFM, who fought in the splitting war and gained independence.

It has been put into his own lib so there is no need to keep a couple of duplicated `Error` modules
for `rpfm-ui` and `rpfm-cli`. As such, **this lib is not intended to be standalone, but a dependency of the `rpfm-lib` crate.

If you need a custom `From` implementation for any error of any lib, add it here.
!*/

use fluent::{FluentError, FluentResource};
use fluent_syntax::parser::ParserError;
use lazy_static::lazy_static;
use log::SetLoggerError;
use sentry::ClientInitGuard;
use serde_json::error::Category;

use std::boxed::Box;
use std::{fmt, fmt::Display};
use std::io;
use std::num::{ParseIntError, ParseFloatError};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use std::result;
use std::string;

pub mod ctd;

/// Alias for handling errors more easely.
pub type Result<T> = result::Result<T, Error>;

/// Current version of the crate.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the DSN needed for Sentry reports to work. Don't change it.
const SENTRY_DSN: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8@sentry.io/1205298";

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    pub static ref SENTRY_GUARD: Arc<RwLock<Option<ClientInitGuard>>> = Arc::new(RwLock::new(None));
}

//---------------------------------------------------------------------------//
//                      Definition of the Types
//---------------------------------------------------------------------------//

/// Custom `Error` Type. One type to hold them all...
///
/// This type implements the `Display` trait to return a meaningful, user-readable error message.
/// Most of the messages contain HTML tags for formatting. If you don't want the HTML tags, use the `Error::to_terminal()` function to remove them.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

/// Custom `ErrorKind` Type. To be able to return different errors using the same `Error` type.
///
/// This type implements the `Display` trait to return a meaningful, user-readable error message.
/// Most of the messages contain HTML tags for formatting. If you don't want the HTML tags, use the `Error::to_terminal()` function to remove them.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {

    //-----------------------------------------------------//
    //                Ser/Deserializer Errors
    //-----------------------------------------------------//

    /// Error for when serializing to `TOML` fails.
    TOMLSerializerError,

    /// Error for when serializing to `RON` fails.
    RonSerializerError,

    /// Error for when deserializing from `RON` fails.
    RonDeserializerError,

    /// Error for when deserializing from `XML` fails.
    XMLDeserializerError,

    /// Error for when serializing and deserializing to/from `Bincode` fails.
    BincodeSerializerError,

    /// Error for invalid Json syntax.
    JsonErrorSyntax,

    /// Error for semantically incorrect Json data.
    JsonErrorData,

    /// Error for unexpected EOF.
    JsonErrorEOF,

    /// Error for when there is an problem while importing a TSV. It contains the row and column of the problematic field.
    ImportTSVIncorrectRow(usize, usize),

    /// Error for when the first field of a TSV file is incorrect.
    ImportTSVWrongTypeTable,

    /// Error for when the second field of a TSV file is not a valid number.
    ImportTSVInvalidVersion,

    /// Error for when the version of a TSV file is not the one we're trying to import to.
    ImportTSVWrongVersion,

    /// Generic TSV import/export error.
    TSVErrorGeneric,

    /// Generic error for when Fluent fails to parse a sentence.
    FluentParsingError,

    /// Generic error for when Fluent fails to load a resource.
    FluentResourceLoadingError,

    /// Generic error for when parsing a String as a F32 fails.
    ParsingFloatError,

    /// Generic error for when parsing a String as an I32 fails.
    ParsingIntegerError,

    // Generic error for when parsing a String as an I64 fails.
    //ParsingLongIntegerError,

    /// Generic error for when the initialization of a logger has failed.
    InitializingLoggerError,

    /// Generic error for when trying to parse something as a bool.
    NotABooleanValue,

    /// Generic error for when we don't find the dependencies cache or it's out of date.
    DependenciesCacheNotGeneratedorOutOfDate,

    //-----------------------------------------------------//
    //                  Network Errors
    //-----------------------------------------------------//

    /// Generic network error.
    NetworkGeneric,

    //-----------------------------------------------------//
    //                     IO Errors
    //-----------------------------------------------------//

    /// Generic IO Error.
    IOGeneric,

    /// Error for when we received a `PermissionDenied` error from the system.
    IOPermissionDenied,

    /// Error for when the file we tried to access doesn't exist.
    IOFileNotFound,

    /// Error for when copying a file fails. Contains the path of the file.
    IOGenericCopy(PathBuf),

    /// Error for when deleting something from the disk fails. Contains the paths that failed to be deleted.
    IOGenericDelete(Vec<PathBuf>),

    /// Generic error for when we can't write a PackedFile to disk. Contains the path of the PackedFile.
    IOGenericWrite(Vec<String>),

    /// Error for when the Assets folder does not exists and it cannot be created.
    IOCreateAssetFolder,

    /// Error for when a folder inside the Assets folder does not exists and it cannot be created.
    IOCreateNestedAssetFolder,

    /// Error for IO errors when reading files using `read_dir()`. Contains the path of the file.
    IOReadFile(PathBuf),

    /// Error for IO errors when reading folders using `read_dir()`. Contains the path of the folder.
    IOReadFolder(PathBuf),

    /// Error for when a folder cannot be open for whatever reason.
    IOFolderCannotBeOpened,

    //-----------------------------------------------------//
    //                 PackFile Errors
    //-----------------------------------------------------//

    /// Generic error to hold any other error triggered when opening a PackFile. Contains the error message.
    OpenPackFileGeneric(String, String),

    /// Generic error to hold any other error triggered when saving a PackFile. Contains the error message.
    SavePackFileGeneric(String),

    /// Error for when we try to open a PackFile, but don't provide his path.
    PackFileNoPathProvided,

    /// Error for when doing mass-loading and we hit an uknown PFHFileType.
    PackFileTypeUknown,

    // Error for when we try to load an unsupported PackFile.
    //PackFileNotSupported,

    /// Error for when the PackFile's header can be read but it's incomplete.
    PackFileHeaderNotComplete,

    /// Error for when the PackFile Indexes are incomplete.
    PackFileIndexesNotComplete,

    /// Error for when we try to open a PackFile and his extension is not ".pack".
    OpenPackFileInvalidExtension,

    /// Error for when trying to save a non-editable PackFile.
    PackFileIsNonEditable,

    /// Error for when the PackFile is not a file in the disk.
    PackFileIsNotAFile,

    /// Error for when the PackFile is not a valid PackFile.
    PackFileIsNotAPackFile,

    /// Error for when the PackFile size doesn't match what we expect. Contains both, the real size and the expected size.
    PackFileSizeIsNotWhatWeExpect(u64, u64),

    /// Error for when trying to copy a Packfile to the data folder, while it's already in the data folder.
    PackFileIsAlreadyInDataFolder,

    //--------------------------------//
    // Schema Errors
    //--------------------------------//

    /// Error for when we don't have schema files and we couldn't download them.
    SchemaNotFoundAndNotDownloaded,

    /// Error for when we don't have an `Schema` to use.
    SchemaNotFound,

    /// Error for when we don't have a `VersionedFile` for a PackedFile.
    SchemaVersionedFileNotFound,

    /// Error for when we don't have a `Definition` for a specific version of a `VersionedFile`.
    SchemaDefinitionNotFound,

    /// Error for when we don't have schema updates available.
    NoSchemaUpdatesAvailable,

    /// Error for when there was an error while downloading the updated schemas.
    SchemaUpdateError,

    /// Error for when the schema update works, but the dependencies rebuild doesn't.
    SchemaUpdateRebuildError(String),

    //-----------------------------------------------------//
    //                PackedFile Errors
    //-----------------------------------------------------//

    /// Error for when the PackedFile we want to get doesn't exists.
    PackedFileNotFound,

    /// Error for when we are trying to do an operation that cannot be done with the PackedFile open.
    PackedFileIsOpen,

    /// Error for when we are trying to open a PackedFile in two different views at the same time.
    PackedFileIsOpenInAnotherView,

    /// Error for when a load_data or get_data operation fails.
    PackedFileDataCouldNotBeLoaded,

    /// Error for when the PackedFile size doesn't match what we expect. Contains the real size and the expected size.
    PackedFileSizeIsNotWhatWeExpect(usize, usize),

    /// Error for when the compressed PackedFile is either incomplete (<9 bytes) or the decompression failed.
    PackedFileDataCouldNotBeDecompressed,

    /// Error for when we expect data to be in memory, but it isn't.
    PackedFileDataIsNotInMemory,

    /// Error for when we try to open a PackedFile not in the filter from the GlobalSearch.
    PackedFileNotInFilter,

    /// Error for when we try to import a PackedFile from another PackFile and it fails miserably. It contains the paths that failed.
    PackedFileCouldNotBeImported(Vec<String>),

    /// Error for when we fail saving a PackedFile.
    PackedFileSaveError(Vec<String>),

    /// Error for when we cannot open a PackedFile due to not being decodeable on the lib.
    PackedFileTypeUnknown,

    /// Error for when we replace the binary data of a PackedFile with another data that's not decodeable in the same way as the old data.
    NewDataIsNotDecodeableTheSameWayAsOldDAta,

    /// Error for when the checksum of a PackedFile fails.
    PackedFileChecksumFailed,

    /// Error for when we detect the source data of a PackFile changed before reading it to memory.
    PackedFileSourceChanged,

    /// Error for when we get a decoded packedfile in a type we don't expect. Contains both, the received type, and the expected type.
    PackedFileTypeIsNotWhatWeExpected(String, String),

    /// Error for when we're trying to do something we can't or shouldn't to a CA PackFile.
    PackFileIsACAPackFile,

    /// Error for when we decode a PackedFile but it fails the index check.
    PackedFileIncompleteDecoding,

    //--------------------------------//
    // Table Errors
    //--------------------------------//

    /// Error for when a row has not the amount of fields we expected. Contains the amount we expected, and the amount we got.
    TableRowWrongFieldCount(u32, u32),

    /// Error for when a field is not of the type we expected it to be. Contains the type we expected, and the type we got.
    TableWrongFieldType(String, String),

    /// Error for when a Table is empty and it doesn't have an `Definition`, so it's undecodeable.
    TableEmptyWithNoDefinition,

    //--------------------------------//
    // DB Table Errors
    //--------------------------------//

    /// Error for when we try to decode something as a DB Table and it fails.
    DBTableIsNotADBTable,

    /// Error for when we try to open a table with a List field on it.
    DBTableContainsListField,

    /// Error for when we are trying to use "Search&Replace" to place invalid data into a cell.
    DBTableReplaceInvalidData,

    /// Error for when a DB Table fails to decode. Contains the error returned by the decoding process.
    DBTableDecode(String),

    /// Error for when we find missing references when checking a DB Table. Contains a list with the tables with missing references.
    DBMissingReferences(Vec<String>),

    /// Error for when we found no newer version of a table than the one we have.
    NoDefinitionUpdateAvailable,

    /// Error for when we can't find a vanilla version of a table to compare with.
    NoTableInGameFilesToCompare,

    //--------------------------------//
    // RigidModel Errors
    //--------------------------------//

    /// Error for when a RigidModel fails to decode. Contains the error message.
    RigidModelDecode(String),

    /// Error for when QtRME fails to parse a RigidModel.
    RigidModelParseError,

    /// Error for when we try to decode an unsupported RigidModel File.
    RigidModelNotSupportedFile,

    /// Error for when we try to decode a unsupported RigidModel type.
    RigidModelNotSupportedType,

    /// Error for when the process of patching a RigidModel to Warhammer format fails. Contains the error message.
    RigidModelPatchToWarhammer(String),

    /// Error for when one of the textures of a rigidmodel represent an unknown mask type.
    RigidModelUnknownMaskTypeFound,

    /// Error for when the texture directory hasn't been found while examining a rigidmodel.
    RigidModelTextureDirectoryNotFound,

    /// Error for when the decal texture directory hasn't been found while examining a rigidmodel.
    RigidModelDecalTextureDirectoryNotFound,

    //--------------------------------//
    // Text Errors
    //--------------------------------//

    /// Error for when a Text PackedFile fails to decode. Contains the error message.
    TextDecode(String),

    /// Error for when a Text PackedFile fails to decode due to not being a plain text file or having an unsupported encoding.
    TextDecodeWrongEncodingOrNotATextFile,

    /// Error for when we try to use Kailua without a types file.
    NoTypesFileFound,

    /// Error for when Kailua is not installed.
    KailuaNotFound,

    //--------------------------------//
    // Loc Errors
    //--------------------------------//

    /// Error for when a Loc PackedFile fails to decode. Contains the error message.
    LocDecode(String),

    /// Error for when we try to decode something as a Loc PackedFile and it fails.
    LocPackedFileIsNotALocPackedFile,

    /// Error for when we try to decode a Loc PackedFile and fails for corruption.
    LocPackedFileCorrupted,

    //--------------------------------//
    // Image Errors
    //--------------------------------//

    /// Error for when an Image fails to decode. Contains the error message.
    ImageDecode(String),

    //--------------------------------//
    // CA_VP8 Errors
    //--------------------------------//

    /// Error for when a CaVp8 PackedFile fails to decode. Contains the error message.
    CaVp8Decode(String),

    //--------------------------------//
    // AnimPack Errors
    //--------------------------------//

    /// Error for when an AnimPack PackedFile fails to decode. Contains the error message.
    AnimPackDecode(String),

    //--------------------------------//
    // AnimTable Errors
    //--------------------------------//

    /// Error for when an AnimTable PackedFile fails to decode. Contains the error message.
    AnimTableDecode(String),

    //--------------------------------//
    // AnimFragment Errors
    //--------------------------------//

    /// Error for when an AnimFragment PackedFile fails to decode. Contains the error message.
    AnimFragmentDecode(String),

    //--------------------------------//
    // MatchedCombat Errors
    //--------------------------------//

    /// Error for when an MatchedCombat PackedFile fails to decode. Contains the error message.
    MatchedCombatDecode(String),

    //--------------------------------//
    // UIC Errors
    //--------------------------------//

    /// Error for when an UIC PackedFile fails to decode. Contains the error message.
    UICDecode(String),

    //--------------------------------//
    // Unit Variant Errors
    //--------------------------------//

    /// Error for when an Unit Variant PackedFile fails to decode. Contains the error message.
    UnitVariantDecode(String),

    /// Error for when we try to decode a PackedFile that's not an Unit Variant as an Unit Variant.
    PackedFileIsNotUnitVariant,

    //--------------------------------//
    // ESF Errors
    //--------------------------------//

    /// Error for when a ESF PackedFile fails to decode. Contains the error message.
    ESFDecode(String),

    /// Error for when we detect an unsupported data type on a ESF file.
    ESFUnsupportedDataType(String),

    /// Error for when we detect an unsupported signature on a ESF file.
    ESFUnsupportedSignature(String),

    /// Error for when we detect the decoding of a ESF has left bytes untouched.
    ESFIncompleteDecoding,

    /// Error for when we fail ot find an specific record name.
    ESFRecordNameNotFound(u32),

    /// Error for when we fail ot find an specific string.
    ESFStringNotFound(u32),

    //--------------------------------//
    // PAK File Errors
    //--------------------------------//

    /// Error for when we try to get the PAK file of a game for which we have no support for PAK files.
    PAKFileNotSupportedForThisGame,

    //-----------------------------------------------------//
    //                Decoding Errors
    //-----------------------------------------------------//

    /// Error for when we fail to get an UTF-8 string from data.
    StringFromUTF8,

    /// This error is to be used when a decoding/encoding operation using the decoding/encoding helpers fails. Contain the error message.
    HelperDecodingEncodingError(String),

    /// This error is to be used when decoding a table fails, and we want to have the data decoded until it failed.
    TableIncompleteError(String, Vec<u8>),

    //-----------------------------------------------------//
    //                  MyMod Errors
    //-----------------------------------------------------//

    /// Error for when we try to uninstall a MyMod that's not currently installed.
    MyModNotInstalled,

    /// Error for when the destination folder for installing a MyMod doesn't exists.
    MyModInstallFolderDoesntExists,

    /// Error for when the path of a game is not configured.
    GamePathNotConfigured,

    /// Error for when the MyMod path is not configured and it needs it to be.
    MyModPathNotConfigured,

    /// Error for when you try to delete a MyMod without having a MyMod selected in the first place.
    MyModDeleteWithoutMyModSelected,

    /// Error for when the MyMod PackFile has been deleted, but his folder is nowhere to be found.
    MyModPackFileDeletedFolderNotFound,

    /// Error for when trying to remove a non-existant MyMod PackFile.
    MyModPackFileDoesntExist,

    /// Error for when trying to do something only MyMods can do.
    PackFileIsNotAMyMod,

    //-----------------------------------------------------//
    //                 Special Errors
    //-----------------------------------------------------//

    /// Error for when trying to patch the SiegeAI and there is nothing in the PackFile.
    PatchSiegeAIEmptyPackFile,

    /// Error for when trying to patch the SiegeAI and there is no patchable files in the PackFile.
    PatchSiegeAINoPatchableFiles,

    /// Error for when you can't do something with a PackedFile open in the right side.
    OperationNotAllowedWithPackedFileOpen,

    //-----------------------------------------------------//
    //                Contextual Errors
    //-----------------------------------------------------//

    /// Error for when extracting one or more PackedFiles from a PackFile fails. Contains the path of the PackedFiles.
    ExtractError(Vec<String>),

    /// Errors for when we fail to mass-import/export TSV files. Contains the error message.
    MassImport(String),

    /// Error for when the introduced input (usually, a name) is empty and it cannot be empty.
    EmptyInput,

    /// Error for when we're trying to use two paths and both are the same.
    PathsAreEqual,

    /// Error for when mass-importing TSV file without selecting any file.
    NoFilesToImport,

    /// Error for when the file we are trying to create already exist in the current path.
    FileAlreadyInPackFile,

    /// Error for when the folder we are trying to create already exist in the current path.
    FolderAlreadyInPackFile,

    /// Error for when we try to create a Queek PackedFile in a folder that doesn't fit the requirements.
    NoQueekPackedFileHere,

    //-----------------------------------------------------//
    //                Assembly Kit Errors
    //-----------------------------------------------------//

    /// Error for when we fail at finding the `Localisable Fields` file.
    AssemblyKitLocalisableFieldsNotFound,

    /// Error for when we try to do an operation over an assembly kit which version we do not yet support.
    AssemblyKitUnsupportedVersion(i16),

    /// Error for when we try to parse a blacklisted table.
    AssemblyKitTableTableIgnored,

    /// Error for when we try to get the assembly kit, but it's not yet installed.
    AssemblyKitNotFound,

    //-----------------------------------------------------//
    //                  7-Zip Errors
    //-----------------------------------------------------//

    /// Error for when 7-zip is not found in the specified path.
    ZipFolderNotFound,

    //-----------------------------------------------------//
    //                  Common Errors
    //-----------------------------------------------------//

    /// Generic error. For a situation where you just need to throw an error, doesn't matter what kind of error. Try not to use it.
    Generic,

    /// Error to returning non-html errors.
    NoHTMLError(String),

    /// Error for just passing a message along.
    GenericHTMLError(String),

    /// Error for when we're trying add/rename/whatever a file with a reserved path.
    ReservedFiles,

    /// Error for when trying to do something to a file that doesn't exists anymore.
    NonExistantFile,

    /// Error for when we're trying to merge two invalid files.
    InvalidFilesForMerging,

    /// Error for when we're trying to decode more bytes than we have.
    NotEnoughBytesToDecode,

    /// Error for when we try to get the `GameInfo` from an unsupported Game.
    GameNotSupported,

    /// Error for when we have to return an error in any path operation related with the Game Selected's Paths.
    GameSelectedPathNotCorrectlyConfigured,

    /// Error for when we try to load a localisation file with an invalid name.
    InvalidLocalisationFileName(String),

    /// Error for when we try to decode the dependency PackFile List and fail.
    DependencyManagerDecode(String),

    /// Error for when we try to read a PackedFile for the Decoder.
    DecoderDecode(String),

    /// Error for when we try to open in the decoder an incompatible PackedFile.
    PackedFileNotDecodeableWithDecoder,

    /// Error for when importing a definition fails due to missing/broken localisable fields file.
    MissingRawLocalisableFields(String),

    /// Error for when we try to launch a game with no steam ID.
    LaunchNotSupportedForThisGame,

    /// Error for when we cannot open RPFM's config folder.
    ConfigFolderCouldNotBeOpened,

    /// Error for when we have a broken path in a template.
    InvalidPathsInTemplate,

    /// Error for when RPFM fails to download templates.
    DownloadTemplatesError,

    /// Error for when RPFM already has the latest templates downloaded.
    AlreadyUpdatedTemplatesError,

    /// Error for when RPFM cannot find an extra PackFile in memory.
    CannotFindExtraPackFile(PathBuf),

    /// Error for when RPFM cannot find an animtable in the currently open PackFile.
    NoAnimTableInPackFile,

    /// Error for when we fail at finding a download for an architecture when autoupdating.
    NoUpdateForYourArchitecture,

    /// Error for when we fail at extracting the update file.
    ErrorExtractingUpdate,

    /// Error for when the PackedFile has not been decoded into memory.
    PackedFileNotDecoded,

    /// Error for when reading the manifest.txt fails.
    ManifestError,

    /// Error for when there was an error while downloading the updated templates.
    TemplateUpdateError,

    /// Error for when we try to add from the same PackFile we have already open.
    CannotAddFromOpenPackFile,

    /// Error for when we try to decode the PackFile settings and fail. Contains the error message.
    PackFileSettingsDecode(String),

    /// Error for when we have no install type for a game selected.
    NoInstallTypeForGame,

    /// Generic error for when we want to say that a string is too long for the text we want.
    StringTooLong(u32),

    /// Error for when the Assembly Kit path for this game is not yet configured.
    GameAssemblyKitPathNotConfigured,

    /// Error for when we can't find the Manifest where we expect it to be.
    GameManifestNotFound,

    /// Error for when we fail to import a number of paths from the dependencies.
    DependenciesImportFailure(Vec<Vec<String>>),

    /// Error for when the user tries to do a Global Replace over dependencies.
    GlobalReplaceOverDependencies,

    /// Error for when we select an unsupported game for a tool we're trying to use.
    GameSelectedNotSupportedForTool,

    /// Error for when one of the widgets in a Template UI fails to be found.
    TemplateUIWidgetNotFound,

    /// Error for when we try to get an inexistant column from a table. Contains the column name.
    ColumnNotFoundInTable(String)
}

/// Implementation of `Error`.
impl Error {

    /// This function returns the `ErrorKind` of the provided `Error`.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// This function removes the HTML tags from the error messages, to make them *"Terminal Friendly"*.
    pub fn to_terminal(&self) -> String {
        format!("{}", self)
            .replace("<p>", "")         // Remove start of paragraph.
            .replace("</p>", "\n")      // Replace end of paragraph with a jump line.
            .replace("<ul>", "\n")      // Replace start of list with a jump line.
            .replace("</ul>", "\n")     // Replace end of list with a jump line.
            .replace("<li>", "")        // Remove start of list entry.
            .replace("</li>", "\n")     // Replace end of list entry with a jump line.
            .replace("<i>", "")         // Replace start of italics.
            .replace("</i>", "")        // Replace end of italics.
    }
}

//------------------------------------------------------------//
//            Extra Implementations for Traits
//------------------------------------------------------------//

/// Implementation of the `Display` Trait for our `Error`.
///
/// This allow us to directly show the error message corresponding to the underlying `ErrorKind`, instead of returning `ErrorKind` to show the message.
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

/// Implementation of the `Display` Trait for our `ErrorKind`.
impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {

            //-----------------------------------------------------//
            //                Ser/Deserializer Errors
            //-----------------------------------------------------//
            ErrorKind::TOMLSerializerError => write!(f, "<p>This should never happen.</p>"),
            ErrorKind::RonSerializerError => write!(f, "<p>This should never happen.</p>"),
            ErrorKind::RonDeserializerError => write!(f, "<p>This should never happen.</p>"),
            ErrorKind::XMLDeserializerError => write!(f, "<p>This should never happen.</p>"),
            ErrorKind::BincodeSerializerError => write!(f, "<p>This should never happen.</p>"),
            ErrorKind::JsonErrorSyntax => write!(f, "<p>Error while trying to read JSON data:</p><p>Invalid syntax found.</p>"),
            ErrorKind::JsonErrorData => write!(f, "<p>Error while trying to read JSON data:</p><p>Semantically incorrect data found.</p>"),
            ErrorKind::JsonErrorEOF => write!(f,"<p>Error while trying to read JSON data:</p><p>Unexpected EOF found.</p>"),
            ErrorKind::ImportTSVIncorrectRow(row, column) => write!(f, "<p>This TSV file has an error in the <b>row <i>{}</i></b>, <b>field <i>{}</i></b> (both starting at 1). Please, check it and make sure the value in that field is a valid value for that column.</p>", row + 1, column + 1),
            ErrorKind::ImportTSVWrongTypeTable => write!(f, "<p>This TSV file either belongs to another table, to a localisation PackedFile, it's broken or it's incompatible with RPFM.</p>"),
            ErrorKind::ImportTSVWrongVersion => write!(f, "<p>This TSV file belongs to another version of this table. If you want to use it, consider creating a new empty table, fill it with enough empty rows, open this file in a TSV editor, like Excel or LibreOffice, and copy column by column.</p><p>A more automatic solution is on the way, but not yet there.</p>"),
            ErrorKind::ImportTSVInvalidVersion => write!(f, "<p>This TSV file has an invalid version value at line 1.</p>"),
            ErrorKind::TSVErrorGeneric => write!(f, "<p>Error while trying to import/export a TSV file.</p>"),
            ErrorKind::FluentParsingError => write!(f, "<p>Error while trying to parse a fluent sentence.</p>"),
            ErrorKind::FluentResourceLoadingError => write!(f, "<p>Error while trying to load a fluent resource.</p>"),
            ErrorKind::ParsingFloatError => write!(f, "<p>Error while trying to parse a String as a Float.</p>"),
            ErrorKind::ParsingIntegerError => write!(f, "<p>Error while trying to parse a String as an Integer.</p>"),
            ErrorKind::InitializingLoggerError => write!(f, "<p>Error while trying to initialize the logger.</p>"),
            //ErrorKind::ParsingLongIntegerError => write!(f, "<p>Error while trying to parse a String as a Long Integer.</p>"),
            ErrorKind::NotABooleanValue => write!(f, "<p>Error while trying to parse something as a bool.</p>"),
            ErrorKind::DependenciesCacheNotGeneratedorOutOfDate => write!(f, "<p>The dependencies cache for the Game Selected is either missing or outdated. Please, re-generate it and try again.</p>"),

            //-----------------------------------------------------//
            //                  Network Errors
            //-----------------------------------------------------//
            ErrorKind::NetworkGeneric => write!(f, "<p>There has been a network-related error. Please, try again later.</p>"),

            //-----------------------------------------------------//
            //                     IO Errors
            //-----------------------------------------------------//
            ErrorKind::IOGeneric => write!(f, "<p>Error while trying to do an IO operation. This means RPFM failed to read/write something from/to the disk.</p>"),
            ErrorKind::IOPermissionDenied => write!(f, "<p>Error while trying to read/write a file from disk. This can be caused by two reasons:</p><ul><li>It's a file in the data folder of Warhammer 2 and you haven't close the Assembly Kit.</li><li>You don't have permission to read/write the file in question.</li></ul>"),
            ErrorKind::IOFileNotFound => write!(f, "<p>Error while trying to use a file from disk:</p><p>The file with the specified path hasn't been found.</p>"),
            ErrorKind::IOGenericCopy(path) => write!(f, "<p>Error while trying to copy one or more files to the following folder:</p><ul>{:#?}</ul>", path),
            ErrorKind::IOGenericDelete(paths) => write!(f, "<p>Error while trying to delete from disk the following files/folders:</p><ul>{:#?}</ul>", paths),
            ErrorKind::IOGenericWrite(paths) => write!(f, "<p>Error while trying to write to disk the following file/s:</p><ul>{:#?}</ul>", paths),
            ErrorKind::IOCreateAssetFolder => write!(f, "<p>The MyMod's asset folder does not exists and it cannot be created.</p>"),
            ErrorKind::IOCreateNestedAssetFolder => write!(f, "<p>The folder does not exists and it cannot be created.</p>"),
            ErrorKind::IOReadFolder(path) => write!(f, "<p>Error while trying to read the following folder:</p><p>{:?}</p><p>This means that path may not be readable by RPFM (permissions? other programs locking access to it?) or may not exists at all.", path),
            ErrorKind::IOReadFile(path) => write!(f, "<p>Error while trying to read the following file:</p><p>{:?}</p><p>This means that path may not be readable by RPFM (permissions? other programs locking access to it?) or may not exists at all.", path),
            ErrorKind::IOFolderCannotBeOpened => write!(f, "<p>The folder couldn't be opened. This means either it doesn't exist, or RPFM has no access to it.</p>"),

            //-----------------------------------------------------//
            //                 PackFile Errors
            //-----------------------------------------------------//
            ErrorKind::OpenPackFileGeneric(name, error) => write!(f, "<p>Error while trying to open the PackFile \"{}\":</p><p>{}</p>", name, error),
            ErrorKind::SavePackFileGeneric(error) => write!(f, "<p>Error while trying to save the currently open PackFile:</p><p>{}</p>", error),
            ErrorKind::PackFileNoPathProvided => write!(f, "<p>No PackFile's path was provided.</p>"),
            ErrorKind::PackFileTypeUknown => write!(f, "<p>The provided PackFile has an Unkwnon PackFile type, which means it cannot be loaded with others. Open it alone if you want to see his contents.</p>"),
            /*ErrorKind::PackFileNotSupported => write!(f, "
            <p>The file is not a supported PackFile.</p>
            <p>For now, we only support:</p>
            <ul>
            <li>- Warhammer 2.</li>
            <li>- Warhammer.</li>
            <li>- Attila.</li>
            <li>- Rome 2.</li>
            <li>- Arena.</li>
            </ul>"),*/
            ErrorKind::PackFileHeaderNotComplete => write!(f, "<p>The header of the PackFile is incomplete, unsupported or damaged.</p>"),
            ErrorKind::PackFileIndexesNotComplete => write!(f, "<p>The indexes of this of the PackFile are incomplete, unsupported or damaged.</p>"),
            ErrorKind::OpenPackFileInvalidExtension => write!(f, "<p>RPFM can only open packfiles whose name ends in <i>'.pack'</i></p>"),
            ErrorKind::PackFileIsNonEditable => write!(f, "
            <p>This type of PackFile is supported in Read-Only mode.</p>
            <p>This can happen due to:</p>
            <ul>
            <li>The PackFile's type is <i>'Boot'</i>, <i>'Release'</i>, <i>'Patch'</i> or <i>'Music'</i> and you have <i>'Allow edition of CA PackFiles'</i> disabled in the settings.</li>
            <li>The PackFile's type is <i>'Other'</i>.</li>
            <li>One of the greyed checkboxes under <i>'PackFile/Change PackFile Type'</i> is checked.</li>
            </ul>
            <p>If you really want to save it, go to <i>'PackFile/Change PackFile Type'</i> and change his type to 'Mod' or 'Movie'. Note that if the cause it's the third on the list, there is no way to save the PackFile, yet.</p>
            <p><b>NOTE</b>: If you created this PackFile using the <i>'Load All CA PackedFiles'</i> feature, NEVER try to save it unless you have 64GB of ram or more. Otherwise it may hang your entire computer to dead.</p>"),
            ErrorKind::PackFileIsNotAPackFile => write!(f, "<p>This file is not a valid PackFile.</p>"),
            ErrorKind::PackFileIsNotAFile => write!(f, "<p>This PackFile doesn't exists as a file in the disk.</p>"),
            ErrorKind::PackFileSizeIsNotWhatWeExpect(reported_size, expected_size) => write!(f, "<p>This PackFile's reported size is <i><b>{}</b></i> bytes, but we expected it to be <i><b>{}</b></i> bytes. This means that either the decoding logic in RPFM is broken for this PackFile, or this PackFile is corrupted.</p>", reported_size, expected_size),
            ErrorKind::NewDataIsNotDecodeableTheSameWayAsOldDAta => write!(f, "<p>The PackedFile you added is not the same type as the one you had before. So... the view showing it will get closed.</p>"),
            ErrorKind::PackFileIsAlreadyInDataFolder => write!(f, "<p>This PackFile is already being edited from the data folder of the game. You cannot install/uninstall it.</p>"),

            //-----------------------------------------------------//
            //                Schema Errors
            //-----------------------------------------------------//
            ErrorKind::SchemaNotFoundAndNotDownloaded => write!(f, "<p>There is no Schema file to load on the disk, and the tries to download one have failed.</p>"),
            ErrorKind::SchemaNotFound => write!(f, "<p>There is no Schema for the Game Selected.</p>"),
            ErrorKind::SchemaVersionedFileNotFound => write!(f, "<p>There is no Definition of the table in the Schema.</p>"),
            ErrorKind::SchemaDefinitionNotFound => write!(f, "<p>There is no Definition for this specific version of the table in the Schema.</p>"),
            ErrorKind::NoSchemaUpdatesAvailable => write!(f, "<p>No schema updates available</p>"),
            ErrorKind::SchemaUpdateError => write!(f, "<p>There was an error while downloading the schemas. Please, try again later.</p><p>If the problem persists (like that time I force-pushed to the repo breaking the updater, good old times) go to <b><i>Preferences/Clear Schema folder</i></b>, and try again.</p>"),
            ErrorKind::SchemaUpdateRebuildError(error) => write!(f, "<p>The schemas were updated successfully, but reloading the dependencies cache for the current Game Selected failed for the following reason:</p> {}", error),

            //-----------------------------------------------------//
            //                PackedFile Errors
            //-----------------------------------------------------//
            ErrorKind::PackedFileNotFound => write!(f, "<p>This PackedFile no longer exists in the PackFile.</p>"),
            ErrorKind::PackedFileIsOpen => write!(f, "<p>That operation cannot be done while the PackedFile involved on it is open. Please, close it by selecting a Folder/PackFile in the TreeView and try again.</p>"),
            ErrorKind::PackedFileIsOpenInAnotherView => write!(f, "<p>That PackedFile is already open in another view. Opening the same PackedFile in multiple views is not supported.</p>"),
            ErrorKind::PackedFileDataCouldNotBeLoaded => write!(f, "<p>This PackedFile's data could not be loaded. This means RPFM can no longer read the PackFile from the disk.</p>"),
            ErrorKind::PackedFileSizeIsNotWhatWeExpect(reported_size, expected_size) => write!(f, "<p>This PackedFile's reported size is <i><b>{}</b></i> bytes, but we expected it to be <i><b>{}</b></i> bytes. This means that either the decoding logic in RPFM is broken for this PackedFile, or this PackedFile is corrupted.</p>", reported_size, expected_size),
            ErrorKind::PackedFileDataCouldNotBeDecompressed => write!(f, "<p>This is a compressed file and the decompresion failed for some reason. This means this PackedFile cannot be opened in RPFM.</p>"),
            ErrorKind::PackedFileDataIsNotInMemory => write!(f, "<p>This PackedFile's data is not in memory. If you see this, report it, as it's a bug.</p>"),
            ErrorKind::PackedFileNotInFilter => write!(f, "<p>This PackedFile is not in the current TreeView filter. If you want to open it, remove the filter.</p>"),
            ErrorKind::PackedFileCouldNotBeImported(paths) => write!(f, "<p>The following failed to be imported:<ul>{}</ul></p>", paths.iter().map(|x| format!("<li>{}<li>", x)).collect::<String>()),
            ErrorKind::PackedFileSaveError(path) => write!(f, "<p>The following PackedFile failed to be saved: {}</p>", path.join("/")),
            ErrorKind::PackedFileTypeUnknown => write!(f, "<p>The PackedFile could not be opened.</p>"),
            ErrorKind::PackedFileChecksumFailed => write!(f, "<p>The PackedFile checksum failed. If you see this, please report it with the actions you did in RPFM before this happened.</p>"),
            ErrorKind::PackedFileSourceChanged => write!(f, "<p>The PackFile's data has been altered on disk outside RPFM. If you see this, it means you're using lazy-loading and another program has altered the data on disk before RPFM could load it to memory.</p><p><b>Basically, this means your PackFile got partially corrupted.</b></p><p>Luckily, not all is lost. Your original PackFile on disk should still be safe, and RPFM can recover part of the files inside the open PackFile: DB tables, Locs and any PackedFile open before this message appeared. To do that, go to Special Stuff and hit '<b><i>Rescue PackFile</i></b>', then choose a folder to save the clean PackFile.</p><p>That will create a PackFile with only the files that were confirmed as non-corrupted, so at least you can recover their data.</p><p>And some final words: If you intentionally opened the same PackFile in two instances of RPFM and tried to save on both, that was the cause of this. No, it's not a bug in RPFM. No, I can't magically fix it. It's how lazy-loading data from disk works. If you don't like it, you can disable lazy-loading in the settings. You'll be resistant to PackFile corruption, but RPFM will use a ton more RAM. So... choose your poison.</p><p><b>Note:</b> If this message appeared while adding files from a PackFile, you're save. Just close the 'Add From PackFile' tab and open it again.</p>"),
            ErrorKind::PackedFileTypeIsNotWhatWeExpected(found_type, expected_type) => write!(f, "<p>The PackedFile's type is not what we expected. We expected {} but found {}. This is either a bug or you did weird things with the game selected.</p>", expected_type, found_type),
            ErrorKind::PackFileIsACAPackFile => write!(f, "<p>You can't do that to a CA PackFile, you monster!</p>"),
            ErrorKind::PackedFileIncompleteDecoding => write!(f, "<p>The PackedFile has been decoded, but there's still part of it undecoded. This means the decoder logic is incomplete for this PackedFile.</p>"),

            //--------------------------------//
            // Table Errors
            //--------------------------------//
            ErrorKind::TableRowWrongFieldCount(expected, real) => write!(f, "<p>Error while trying to save a row from a table:</p><p>We expected a row with \"{}\" fields, but we got a row with \"{}\" fields instead.</p>", expected, real),
            ErrorKind::TableWrongFieldType(expected, real) => write!(f, "<p>Error while trying to save a row from a table:</p><p>We expected a field of type \"{}\", but we got a field of type \"{}\".</p>", expected, real),
            ErrorKind::TableEmptyWithNoDefinition => write!(f, "<p>This table is empty and there is not a Definition for it. That means is undecodeable.</p>"),

            //--------------------------------//
            // DB Table Errors
            //--------------------------------//
            ErrorKind::DBTableIsNotADBTable => write!(f, "<p>This is either not a DB Table, or it's a DB Table but it's corrupted.</p>"),
            ErrorKind::DBTableContainsListField => write!(f, "<p>This specific table version uses a currently unimplemented type (List), so is undecodeable, for now.</p>"),
            ErrorKind::DBTableReplaceInvalidData => write!(f, "<p>Error while trying to replace the data of a Cell.</p><p>This means you tried to replace a number cell with text, or used a too big, too low or invalid number. Don't do it. It wont end well.</p>"),
            ErrorKind::DBTableDecode(cause) => write!(f, "<p>Error while trying to decode the DB Table:</p><p>{}</p><p>Before anything else, please check your game selected is really the one this PackFile is for! If it isn't, change your game selected and try again.</p>", cause),
            ErrorKind::DBMissingReferences(references) => write!(f, "<p>The currently open PackFile has reference errors in the following tables:<ul>{}</ul></p>", references.iter().map(|x| format!("<li>{}<li>", x)).collect::<String>()),
            ErrorKind::NoDefinitionUpdateAvailable => write!(f, "<p>This table already has the newer definition available.</p>"),
            ErrorKind::NoTableInGameFilesToCompare => write!(f, "<p>This table cannot be found in the Game Files, so it cannot be automatically updated (yet).</p>"),

            //--------------------------------//
            // RigidModel Errors
            //--------------------------------//
            ErrorKind::RigidModelDecode(cause) => write!(f, "<p>Error while trying to decode the RigidModel PackedFile:</p><p>{}</p>", cause),
            ErrorKind::RigidModelParseError => write!(f, "<p>This Rigidmodel cannot be correctly parsed yet.</p>"),
            ErrorKind::RigidModelNotSupportedFile => write!(f, "<p>This file is not a Supported RigidModel file.</p>"),
            ErrorKind::RigidModelNotSupportedType => write!(f, "<p>This RigidModel's Type is not currently supported.</p>"),
            ErrorKind::RigidModelPatchToWarhammer(cause) => write!(f, "<p>Error while trying to patch the RigidModel file:</p><p>{}</p>", cause),
            ErrorKind::RigidModelUnknownMaskTypeFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Texture with unknown Mask Type found.</li></ul>"),
            ErrorKind::RigidModelTextureDirectoryNotFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Texture Directories not found.</li></ul>"),
            ErrorKind::RigidModelDecalTextureDirectoryNotFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Decal Texture Directory not found.</li></ul>"),

            //--------------------------------//
            // Text Errors
            //--------------------------------//
            ErrorKind::TextDecode(cause) => write!(f, "<p>Error while trying to decode the Text PackedFile:</p><p>{}</p>", cause),
            ErrorKind::TextDecodeWrongEncodingOrNotATextFile => write!(f, "<p>This is either not a Text PackedFile, or a Text PackedFile using an unsupported encoding</p>"),
            ErrorKind::NoTypesFileFound => write!(f, "<p>There is no Types file for the current Game Selected, so you can't use Kailua.</p>"),
            ErrorKind::KailuaNotFound => write!(f, "<p>Kailua executable not found. Install it and try again.</p>"),

            //--------------------------------//
            // Loc Errors
            //--------------------------------//
            ErrorKind::LocDecode(cause) => write!(f, "<p>Error while trying to decode the Loc PackedFile:</p><p>{}</p>", cause),
            ErrorKind::LocPackedFileIsNotALocPackedFile => write!(f, "<p>This is either not a Loc PackedFile, or it's a Loc PackedFile but it's corrupted.</p>"),
            ErrorKind::LocPackedFileCorrupted => write!(f, "<p>This Loc PackedFile seems to be corrupted.</p>"),

            //--------------------------------//
            // Image Errors
            //--------------------------------//
            ErrorKind::ImageDecode(cause) => write!(f, "<p>Error while trying to decode the Image PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // CA_VP8 Errors
            //--------------------------------//
            ErrorKind::CaVp8Decode(cause) => write!(f, "<p>Error while trying to decode the CaVp8 PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // AnimPack Errors
            //--------------------------------//
            ErrorKind::AnimPackDecode(cause) => write!(f, "<p>Error while trying to decode the AnimPack PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // AnimTable Errors
            //--------------------------------//
            ErrorKind::AnimTableDecode(cause) => write!(f, "<p>Error while trying to decode the AnimTable PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // AnimFragment Errors
            //--------------------------------//
            ErrorKind::AnimFragmentDecode(cause) => write!(f, "<p>Error while trying to decode the AnimFragment PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // MatchedCombat Errors
            //--------------------------------//
            ErrorKind::MatchedCombatDecode(cause) => write!(f, "<p>Error while trying to decode the MatchedCombat PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // UIC Errors
            //--------------------------------//
            ErrorKind::UICDecode(cause) => write!(f, "<p>Error while trying to decode the UIC PackedFile:</p><p>{}</p>", cause),

            //--------------------------------//
            // ESF Errors
            //--------------------------------//
            ErrorKind::ESFDecode(cause) => write!(f, "<p>Error while trying to decode the ESF PackedFile:</p><p>{}</p>", cause),
            ErrorKind::ESFUnsupportedDataType(data_type) => write!(f, "<p>Unsupported data type: {}</p>", data_type),
            ErrorKind::ESFUnsupportedSignature(signature) => write!(f, "<p>Unsupported signature: {}</p>", signature),
            ErrorKind::ESFIncompleteDecoding => write!(f, "<p>There are bytes still to decode, but the decoding process has finished. This means RPFM cannot yet decode this file correctly</p><p>If you see this message, please report it to RPFM's author so support for the file that caused the error can be implemented.</p>"),
            ErrorKind::ESFStringNotFound(index) => write!(f, "<p>String not found: {}</p>", index),
            ErrorKind::ESFRecordNameNotFound(index) => write!(f, "<p>Record name not found: {}</p>", index),

            //--------------------------------//
            // UnitVariant Errors
            //--------------------------------//
            ErrorKind::UnitVariantDecode(cause) => write!(f, "<p>Error while trying to decode the Unit Variant PackedFile:</p><p>{}</p>", cause),
            ErrorKind::PackedFileIsNotUnitVariant => write!(f, "<p>This PackedFile is not an Unit Variant</p>"),

            //--------------------------------//
            // PAK File Errors
            //--------------------------------//

            // Error for when we try to get the PAK file of a game for which we have no support for PAK files.
            ErrorKind::PAKFileNotSupportedForThisGame => write!(f, "<p>The currently selected game doesn't have support for PAK files.</p>"),

            //-----------------------------------------------------//
            //                Decoding Errors
            //-----------------------------------------------------//
            ErrorKind::StringFromUTF8 => write!(f, "<p>Error while converting data to an UTF-8 String.</p>"),
            ErrorKind::HelperDecodingEncodingError(cause) => write!(f, "{}", cause),
            ErrorKind::TableIncompleteError(cause, _) => write!(f, "{}", cause),

            //-----------------------------------------------------//
            //                  MyMod Errors
            //-----------------------------------------------------//
            ErrorKind::MyModNotInstalled => write!(f, "<p>The currently selected MyMod is not installed.</p>"),
            ErrorKind::MyModInstallFolderDoesntExists => write!(f, "<p>Destination folder (..xxx/data) doesn't exist. You sure you configured the right folder for the game?</p>"),
            ErrorKind::GamePathNotConfigured => write!(f, "<p>Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.</p>"),
            ErrorKind::MyModPathNotConfigured => write!(f, "<p>MyMod path is not configured. Configure it in the settings and try again.</p>"),
            ErrorKind::MyModDeleteWithoutMyModSelected => write!(f, "<p>You can't delete the selected MyMod if there is no MyMod selected.</p>"),
            ErrorKind::MyModPackFileDeletedFolderNotFound => write!(f, "<p>The Mod's PackFile has been deleted, but his assets folder is nowhere to be found.</p>"),
            ErrorKind::MyModPackFileDoesntExist => write!(f, "<p>The PackFile of the selected MyMod doesn't exists, so it can't be installed or removed.</p>"),
            ErrorKind::PackFileIsNotAMyMod => write!(f, "<p>This action is only available for MyMods.</p>"),

            //-----------------------------------------------------//
            //                 Special Errors
            //-----------------------------------------------------//
            ErrorKind::PatchSiegeAIEmptyPackFile => write!(f, "<p>This packfile is empty, so we can't patch it.</p>"),
            ErrorKind::PatchSiegeAINoPatchableFiles => write!(f, "<p>There are not files in this Packfile that could be patched/deleted.</p>"),
            ErrorKind::OperationNotAllowedWithPackedFileOpen => write!(f, "<p>This operation cannot be done while there is a PackedFile open. Select a folder or the PackFile to close it and try again.</p>"),

            //-----------------------------------------------------//
            //                Contextual Errors
            //-----------------------------------------------------//
            ErrorKind::ExtractError(errors) => write!(f, "<p>There has been a problem extracting the following files:</p><ul>{:#?}</ul>", errors),
            ErrorKind::MassImport(errors) => write!(f, "<p>The following files returned error when trying to import them:</p><ul>{}</ul><p>No files have been imported.</p>", errors),
            ErrorKind::EmptyInput => write!(f, "<p>Only my hearth can be empty.</p>"),
            ErrorKind::PathsAreEqual => write!(f, "<p>Both paths (source and destination) are the same.</p>"),
            ErrorKind::NoFilesToImport => write!(f, "<p>It's mathematically impossible to successfully import zero TSV files.</p>"),
            ErrorKind::FileAlreadyInPackFile => write!(f, "<p>The provided file/s already exists in the current path.</p>"),
            ErrorKind::FolderAlreadyInPackFile => write!(f, "<p>That folder already exists in the current path.</p>"),
            ErrorKind::NoQueekPackedFileHere => write!(f, "<p>I don't know what type of file goes in that folder, boi.</p>"),

            //-----------------------------------------------------//
            //                Assembly Kit Errors
            //-----------------------------------------------------//
            ErrorKind::AssemblyKitLocalisableFieldsNotFound => write!(f, "<p>The `Localisable Fields` file hasn't been found.</p>"),
            ErrorKind::AssemblyKitUnsupportedVersion(version) => write!(f, "<p>Operations over the Assembly Kit of version {} are not currently supported.</p>", version),
            ErrorKind::AssemblyKitTableTableIgnored => write!(f, "<p>One of the Assembly Kit Tables you tried to decode has been blacklisted due to issues.</p>"),
            ErrorKind::AssemblyKitNotFound => write!(f, "<p>The Assembly Kit Folder could not be read. You may need to install the Assembly Kit for the current game selected.</p>"),

            //-----------------------------------------------------//
            //                  7-Zip Errors
            //-----------------------------------------------------//
            ErrorKind::ZipFolderNotFound => write!(f, "<p>7Zip path not found, or the 7Zip path you put in the settings is wrong.</p>"),

            //-----------------------------------------------------//
            //                  Common Errors
            //-----------------------------------------------------//
            ErrorKind::Generic => write!(f, "<p>Generic error. You should never read this.</p>"),
            ErrorKind::NoHTMLError(error) => write!(f,"{}", error),
            ErrorKind::GenericHTMLError(error) => write!(f,"{}", error),
            ErrorKind::ReservedFiles => write!(f, "<p>One or more of the files you're trying to add/create/rename to have a reserved name. Those names are reserved for internal use in RPFM. Please, try again with another name.</p>"),
            ErrorKind::NonExistantFile => write!(f, "<p>The file you tried to... use doesn't exist. This is a bug, because if everything worked propetly, you'll never see this message.</p>"),
            ErrorKind::InvalidFilesForMerging => write!(f, "<p>The files you selected are not all LOCs, neither DB Tables of the same type and version.</p>"),
            ErrorKind::NotEnoughBytesToDecode => write!(f, "<p>There are not enough bytes to decode in the data you provided.</p>"),
            ErrorKind::GameNotSupported => write!(f, "<p>The game you tried to get the info is not supported.</p>"),
            ErrorKind::GameSelectedPathNotCorrectlyConfigured => write!(f, "<p>The Game Selected's Path is not properly configured.</p>"),
            ErrorKind::InvalidLocalisationFileName(name) => write!(f, "<p>The name '{}' is not a valid localisation file name. It has to have one and only one '_' somewhere and an identifier (en, fr,...) after that.</p>", name),
            ErrorKind::DependencyManagerDecode(cause) => write!(f, "<p>Error while trying to decode the Dependency PackFile List:</p><p>{}</p>", cause),
            ErrorKind::DecoderDecode(cause) => write!(f, "<p>Error while trying to load the following PackedFile to the decoder:</p><p>{}</p>", cause),
            ErrorKind::PackedFileNotDecodeableWithDecoder => write!(f, "<p>This PackedFile cannot be decoded using the PackedFile Decoder.</p>"),
            ErrorKind::MissingRawLocalisableFields(cause) => write!(f, "<p>{}</p><p>This happens when the TExc_LocalisableFields.xml file is missing/empty/broken in raw_data/db. If it's so, copy it from another game and try again.</p>", cause),
            ErrorKind::LaunchNotSupportedForThisGame => write!(f, "<p>The currently selected game cannot be launched from Steam.</p>"),
            ErrorKind::ConfigFolderCouldNotBeOpened => write!(f, "<p>RPFM's config folder couldn't be open (maybe it doesn't exists?).</p>"),
            ErrorKind::InvalidPathsInTemplate => write!(f, "<p>An empty/invalid path has been detected when processing the template. This can be caused by a bad template or by an empty parameter.<p>"),
            ErrorKind::DownloadTemplatesError => write!(f, "<p>Failed to download the latest templates.<p>"),
            ErrorKind::AlreadyUpdatedTemplatesError => write!(f, "<p>Templates already up-to-date.<p>"),
            ErrorKind::CannotFindExtraPackFile(path) => write!(f, "<p>Cannot find extra PackFile with path: {:?}.<p>", path),
            ErrorKind::NoAnimTableInPackFile => write!(f, "<p>No AnimTable found in the PackFile.<p>"),
            ErrorKind::NoUpdateForYourArchitecture => write!(f, "<p>No download available for your architecture.<p>"),
            ErrorKind::ErrorExtractingUpdate => write!(f, "<p>There was an error while extracting the update. This means either I uploaded a broken file, or your download was incomplete. In any case, no changes have been done so... try again later.<p>"),
            ErrorKind::PackedFileNotDecoded => write!(f, "<p>Undecoded PackedFile. If you see this, it's a bug, so please report it.<p>"),
            ErrorKind::ManifestError => write!(f, "<p>Error while parsing the manifest.txt file of the game selected.<p>"),
            ErrorKind::TemplateUpdateError => write!(f, "<p>There was an error while downloading the templates. Please, try again later.</p>"),
            ErrorKind::CannotAddFromOpenPackFile => write!(f, "<p>You cannot add PackedFile to the same PackFile you're adding from. It's like putting a bag of holding into a bag of holding.</p>"),
            ErrorKind::PackFileSettingsDecode(cause) => write!(f, "<p>Error while trying to decode the PackFile-Specific Settings:</p><p>{}</p>", cause),
            ErrorKind::NoInstallTypeForGame => write!(f, "<p>The currently selected game doesn't have an Install Type. If this pops up and the Game is not Arena, please report it.</p>"),
            ErrorKind::StringTooLong(size) => write!(f, "<p>The string is too long. The MAX limit is {}.</p>", size),
            ErrorKind::GameAssemblyKitPathNotConfigured => write!(f, "<p>The Assembly Kit path is not yet configured for the game selected.</p>"),
            ErrorKind::GameManifestNotFound => write!(f, "<p>The manifest for the Game Selected hasn't been found.</p>"),
            ErrorKind::DependenciesImportFailure(paths) => write!(f, "<p>There was an error importing the following files:</p> <ul>{}</ul>", paths.iter().map(|x| "<li>".to_owned() + &x.join("/") + "</li>").collect::<String>()),
            ErrorKind::GlobalReplaceOverDependencies => write!(f, "<p>The dependencies are read-only. You cannot do a Global Replace over them.</p>"),
            ErrorKind::GameSelectedNotSupportedForTool => write!(f, "<p>This tool is not supported for the currently selected game.</p>"),
            ErrorKind::TemplateUIWidgetNotFound => write!(f, "<p>One of the widgets of this view has not been found in the UI Template. This means either the code is wrong, or the template is incomplete/outdated.</p>"),
            ErrorKind::ColumnNotFoundInTable(column_name) => write!(f, "<p>The column '{}' is missing from the table we expected to contain it.</p>", column_name),
        }
    }
}

//------------------------------------------------------------//
//   Implementations for internal types for the From Trait
//------------------------------------------------------------//

/// Implementation to create an `Error` from a `String`.
impl From<String> for Error {
    fn from(error: String) -> Self {
        Self { kind: ErrorKind::NoHTMLError(error) }
    }
}

/// Implementation to create an `Error` from an `ErrorKind`.
impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

//------------------------------------------------------------//
//      Implementations for std types for the From Trait
//------------------------------------------------------------//

/// Implementation to create an `Error` from a `FromUTF8Error`.
impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Self {
        Self::from(ErrorKind::StringFromUTF8)
    }
}

/// Implementation to create an `Error` from a `std::io::Error`.
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {

        // Get his category, and create an error based on that.
        match error.kind() {
            io::ErrorKind::NotFound => Self::from(ErrorKind::IOFileNotFound),
            io::ErrorKind::PermissionDenied => Self::from(ErrorKind::IOPermissionDenied),
            _ => Self::from(ErrorKind::IOGeneric),
        }
    }
}

//------------------------------------------------------------//
//   Implementations for external types for the From Trait
//------------------------------------------------------------//

/// Implementation to create an `Error` from a `serde_json::Error`.
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {

        // Get his category, and create an error based on that.
        match error.classify() {
            Category::Io => Self::from(ErrorKind::IOGeneric),
            Category::Syntax => Self::from(ErrorKind::JsonErrorSyntax),
            Category::Data => Self::from(ErrorKind::JsonErrorData),
            Category::Eof => Self::from(ErrorKind::JsonErrorEOF),
        }
    }
}

/// Implementation to create an `Error` from a `csv::Error`.
impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {

        // Get his category, and create an error based on that.
        match error.kind() {
            csv::ErrorKind::Io(_) => Self::from(ErrorKind::IOGeneric),
            _ => Self::from(ErrorKind::TSVErrorGeneric)
        }
    }
}

/// Implementation to create an `Error` from a `toml::ser::Error`.
impl From<toml::ser::Error> for Error {
    fn from(_: toml::ser::Error) -> Self {
        Self::from(ErrorKind::TOMLSerializerError)
    }
}

/// Implementation to create an `Error` from a `serde_xml_rs::Error`.
impl From<serde_xml_rs::Error> for Error {
    fn from(_: serde_xml_rs::Error) -> Self {
        Self::from(ErrorKind::XMLDeserializerError)
    }
}

/// Implementation to create an `Error` from a `Box<bincode::ErrorKind>`.
impl From<Box<bincode::ErrorKind>> for Error {
    fn from(_: Box<bincode::ErrorKind>) -> Self {
        Self::from(ErrorKind::BincodeSerializerError)
    }
}

/// Implementation to create an `Error` from a `ron::error::Error`.
impl From<ron::error::Error> for Error {
    fn from(_: ron::error::Error) -> Self {
        Self::from(ErrorKind::RonDeserializerError)
    }
}

/// Implementation to create an `Error` from a `(FluentResource, Vec<ParserError>)`. Because for fluent, single errors are hard.
impl From<(FluentResource, Vec<ParserError>)> for Error {
    fn from(_: (FluentResource, Vec<ParserError>)) -> Self {
        Self::from(ErrorKind::FluentParsingError)
    }
}

/// Implementation to create an `Error` from a `Vec<FluentError>`. Because for fluent, single errors are hard.
impl From<Vec<FluentError>> for Error {
    fn from(_: Vec<FluentError>) -> Self {
        Self::from(ErrorKind::FluentResourceLoadingError)
    }
}

/// Implementation to create an `Error` from a `ParseFloatError`.
impl From<ParseFloatError> for Error {
    fn from(_: ParseFloatError) -> Self {
        Self::from(ErrorKind::ParsingFloatError)
    }
}

/// Implementation to create an `Error` from a `ParseIntegerError`.
impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::from(ErrorKind::ParsingIntegerError)
    }
}

/// Implementation to create an `Error` from a `SetLoggerError`.
impl From<SetLoggerError> for Error {
    fn from(_: SetLoggerError) -> Self {
        Self::from(ErrorKind::InitializingLoggerError)
    }
}

/// Implementation to create an `Error` from a `git2::Error`.
impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self::from(ErrorKind::GenericHTMLError(error.message().to_string()))
    }
}

/// Implementation to create an `Error` from a `self_update::errors::Error`.
impl From<self_update::errors::Error> for Error {
    fn from(error: self_update::errors::Error) -> Self {
        Self::from(ErrorKind::GenericHTMLError(error.to_string()))
    }
}
