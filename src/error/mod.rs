// Here should go all the stuff needed to get the damn error system working, so we can provide precise
// error reports instead what we had before.

use failure::{Backtrace, Context, Fail};
use serde_json::error::Category;
use toml::ser;

use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::result;
use std::io;
use std::string;

pub mod logger;

/// Alias for handling errors more easely.
pub type Result<T> = result::Result<T, Error>;

/// Custom Error Type. Were the magic begins.
#[derive(Debug)]
pub struct Error {
    context: Context<ErrorKind>,
}

/// Custom ErrorKind Type. To be able to differentiate errors. All these errors are supposed to have
/// an String inside, with "user-readable" information about the error in HTML.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {

    // Generic error. For a situation where you just need to throw an error, doesn't matter what kind of error.
    Generic,

    // Error for when someone tries to divide by 0.
    ThereIsAnSpecialPlaceInHellForYou,

    // Error for when serializing to TOML fails.
    TOMLSerializerError,

    //-----------------------------------------------------//
    //                  Network Errors
    //-----------------------------------------------------//

    // Generic network error.
    NetworkGeneric,

    //-----------------------------------------------------//
    //                     IO Errors
    //-----------------------------------------------------//

    // These errors are errors for when dealing with IO problems.
    IOPermissionDenied,
    IOFileNotFound,
    IOGeneric,

    // Error for when copying a file fails.
    IOGenericCopy(PathBuf),

    // Error for when we fail in deleting something from the disk.
    IOGenericDelete(Vec<PathBuf>),

    // Generic error for when we can't write a file to disk.
    IOGenericWrite(Vec<String>),

    // Error for when the Assets folder does not exists and it cannot be created.
    IOCreateAssetFolder,

    // Error for when a folder inside the Assets folder does not exists and it cannot be created.
    IOCreateNestedAssetFolder,

    // Error for IO errors when using "read_dir()".
    IOReadFolder(PathBuf),

    //-----------------------------------------------------//
    //                TSV-related Errors
    //-----------------------------------------------------//

    // These errors are to be used when importing TSV files. The last one is for any other error it can happen not already covered.
    ImportTSVIncorrectRow(usize, usize),
    ImportTSVWrongTypeTable,
    ImportTSVWrongTypeLoc,
    ImportTSVWrongVersion,
    TSVErrorGeneric,

    //-----------------------------------------------------//
    //                 PackFile Errors
    //-----------------------------------------------------//

    // Generic error to hold any other error triggered when opening a PackFile.
    OpenPackFileGeneric(String),

    // Generic error to hold any other error triggered when saving a PackFile.
    SavePackFileGeneric(String),

    // Error for when we try to load an unsupported PackFile.
    PackFileNotSupported,

    // Error for when the PackFile's header can be read but it's not decodeable.
    PackFileHeaderNotComplete,

    // Error for when the PackFile Indexes are incomplete.
    PackFileIndexesNotComplete,

    // Error for when we try to open a PackFile and his extension is not ".pack".
    OpenPackFileInvalidExtension,

    // Error for when trying to save a non-editable PackFile.
    PackFileIsNonEditable,

    // Error for when the PackFile is not a file in the disk.
    PackFileIsNotAFile,

    // Error for when the PackFile is not a valid PackFile.
    PackFileIsNotAPackFile,

    // Error for when the PackFile size doesn't match what we expect.
    PackFileSizeIsNotWhatWeExpect(u64, u64),

    //-----------------------------------------------------//
    //                PackedFile Errors
    //-----------------------------------------------------//

    // Error for when the PackedFile we want to get doesn't exists.
    PackedFileNotFound,

    // Error for when we are trying to do an operation that cannot be done with the PackedFile open.
    PackedFileIsOpen,

    // Error for when we are trying to open a PackedFile in two different views at the same time.
    PackedFileIsOpenInAnotherView,

    // Error for when a load_data or get_data fails.
    PackedFileDataCouldNotBeLoaded,

    // Error for when the PackedFile size doesn't match what we expect.
    PackedFileSizeIsNotWhatWeExpect(usize, usize),

    //--------------------------------//
    // DB Table Errors
    //--------------------------------//

    // Error for when we try to decode something as a DB Table and it fails.
    DBTableIsNotADBTable,

    // Error for when we try to open a table with a List field on it.
    DBTableContainsListField,

    // Error for when we are trying to use "Search&Replace" to place invalid data into a cell.
    DBTableReplaceInvalidData,

    // Error for when we hit an overflow in the "Apply maths" feature.
    DBTableApplyMathsOverflow,

    // Error for when a DB Table fails to decode.
    DBTableDecode(String),

    // Error for when a DB Table is empty and it doesn't have an schema, so it's undecodeable.
    DBTableEmptyWithNoTableDefinition,

    // Error for when we don't have an schema to use.
    SchemaNotFound,

    // Error for when we don't have a table definition for an specific version of a table.
    SchemaTableDefinitionNotFound,

    //--------------------------------//
    // RigidModel Errors
    //--------------------------------//

    // Error for when a RigidModel fails to decode.
    RigidModelDecode(String),

    // Errors for when decoding a RigidModel File.
    RigidModelNotSupportedFile,
    RigidModelNotSupportedType,

    // Error for when the process of patching a RigidModel to Warhammer format fails.
    RigidModelPatchToWarhammer(String),

    // Error for when one of the textures of a rigidmodel represent an unknown mask type.
    RigidModelUnknownMaskTypeFound,

    // Error for when the texture directory hasn't been found while examining a rigidmodel.
    RigidModelTextureDirectoryNotFound,

    // Error for when the texture directory hasn't been found while examining a rigidmodel.
    RigidModelDecalTextureDirectoryNotFound,

    //--------------------------------//
    // Text Errors
    //--------------------------------//

    // Error for when a Text PackedFile fails to decode.
    TextDecode(String),

    // Error for when we try to use Kailua without a types file.
    NoTypesFileFound,

    // Error for when Kailua is not installed.
    KailuaNotFound,

    //--------------------------------//
    // Loc Errors
    //--------------------------------//

    // Error for when a Loc PackedFile fails to decode.
    LocDecode(String),

    // Error for when we try to decode something as a Loc PackedFile and it fails.
    LocPackedFileIsNotALocPackedFile,

    // Error for when we try to decode a Loc PackedFile and fails for corruption.
    LocPackedFileCorrupted,

    //--------------------------------//
    // Image Errors
    //--------------------------------//

    // Error for when an Image fails to decode.
    ImageDecode(String),

    //-----------------------------------------------------//
    //                Decoding Errors
    //-----------------------------------------------------//

    // Error for when we fail to get an UTF-8 string from data.
    StringFromUTF8,

    // This error is to be used when a decoding/encoding operation using the decoding/encoding helpers fails.
    HelperDecodingEncodingError(String),

    //-----------------------------------------------------//
    //                  MyMod Errors
    //-----------------------------------------------------//

    // Error for when we try to uninstall a MyMod that's not currently installed.
    MyModNotInstalled,

    // Error for when the destination folder for installing a MyMod doesn't exists.
    MyModInstallFolderDoesntExists,

    // Error for when the path of a Game is not configured.
    GamePathNotConfigured,

    // Error for when the MyMod path is not configured and it needs it to be.
    MyModPathNotConfigured,

    // Error for when you try to delete a MyMod without having a MyMod selected in the first place.
    MyModDeleteWithoutMyModSelected,

    // Error for when the MyMod PackFile has been deleted, but his folder is nowhere to be found.
    MyModPackFileDeletedFolderNotFound,

    // Error for when trying to remove a non-existant MyMod PackFile.
    MyModPackFileDoesntExist,

    //-----------------------------------------------------//
    //                 Special Errors
    //-----------------------------------------------------//

    // Error for when trying to patch the SiegeAI and there is nothing in the PackFile.
    PatchSiegeAIEmptyPackFile,

    // Error for when trying to patch the SiegeAI and there is no patchable files in the PackFile.
    PatchSiegeAINoPatchableFiles,

    // Error for when you can't do something with a PackedFile open in the right side.
    OperationNotAllowedWithPackedFileOpen,

    //-----------------------------------------------------//
    //                Contextual Errors
    //-----------------------------------------------------//

    // Error for when a name is already in use in a path and is not valid for renaming.
    NameAlreadyInUseInThisPath,

    // Error for when extracting one or more PackedFiles from a PackFile.
    ExtractError(Vec<String>),

    // Errors for when we fail to mass-import/export TSV files.
    MassImport(String),

    // Error for when the introduced input (usually, a name) is empty and it cannot be empty.
    EmptyInput,

    // Error for when the introduced input (usually, a name) has invalid characters, or it's invalid for any other reason.
    InvalidInput,

    // Error for when the introduced input (usually, a name) hasn't changed.
    UnchangedInput,

    // Error for when mass-importing TSV file without selecting any file.
    NoFilesToImport,

    // Error for when the file we are trying to create already exist in the current path.
    FileAlreadyInPackFile,

    // Error for when the folder we are trying to create already exist in the current path.
    FolderAlreadyInPackFile,

    //-----------------------------------------------------//
    //                  Common Errors
    //-----------------------------------------------------//

    // Error for invalid Json syntax.
    JsonErrorSyntax,

    // Error for semantically incorrect Json data.
    JsonErrorData,

    // Error for unexpected EOF.
    JsonErrorEOF,
}

/// Implementation of our custom Error Type.
impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.context.get_context().clone()
    }
}

//------------------------------------------------------------//
//              Implementations of Fail Trait
//------------------------------------------------------------//

/// Implementation of the "Fail" Trait for our custom Error Type.
impl Fail for Error {

    /// Implementation of "cause()" for our custom Error Type.
    fn cause(&self) -> Option<&Fail> {
        self.context.cause()
    }

    /// Implementation of "backtrace()" for our custom Error Type.
    fn backtrace(&self) -> Option<&Backtrace> {
        self.context.backtrace()
    }
}

//------------------------------------------------------------//
//            Extra Implementations for Traits
//------------------------------------------------------------//

/// Implementation of the "Display" Trait for our custom Error Type.
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.context, f)
    }
}

/// Implementation of the "Display" Trait for our custom ErrorKind Type.
/// NOTE: There are so many reasons the decoding/encoding helpers can fail, that we have to group them
/// into one error.
impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::Generic => write!(f, "<p>Generic error. You should never read this.</p>"),
            ErrorKind::ThereIsAnSpecialPlaceInHellForYou => write!(f, "<p>There is an special place in hell for you.</p>"),
            ErrorKind::TOMLSerializerError => write!(f, "<p>This should never happen.</p>"),

            //-----------------------------------------------------//
            //                  Network Errors
            //-----------------------------------------------------//
            ErrorKind::NetworkGeneric => write!(f, "<p>There has been a network-related error. Please, try again later.</p>"),

            //-----------------------------------------------------//
            //                     IO Errors
            //-----------------------------------------------------//
            ErrorKind::IOPermissionDenied => write!(f, "<p>Error while trying to read/write a file from disk. This can be caused by two reasons:</p><ul><li>It's a file in the data folder of Warhammer 2 and you haven't close the Assembly Kit.</li><li>You don't have permission to read/write the file in question.</li></ul>"),
            ErrorKind::IOFileNotFound => write!(f, "<p>Error while trying to use a file from disk:</p><p>The file with the specified path hasn't been found.</p>"),
            ErrorKind::IOGeneric => write!(f, "<p>Error while trying to do an IO operation. This means RPFM failed to read/write something from/to the disk.</p>"),
            ErrorKind::IOGenericCopy(path) => write!(f, "<p>Error while trying to copy one or more files to the following folder:</p><ul>{:#?}</ul>", path),
            ErrorKind::IOGenericDelete(paths) => write!(f, "<p>Error while trying to delete from disk the following files/folders:</p><ul>{:#?}</ul>", paths),
            ErrorKind::IOGenericWrite(paths) => write!(f, "<p>Error while trying to write to disk the following file/s:</p><ul>{:#?}</ul>", paths),
            ErrorKind::IOCreateAssetFolder => write!(f, "<p>The MyMod's asset folder does not exists and it cannot be created.</p>"),
            ErrorKind::IOCreateNestedAssetFolder => write!(f, "<p>The folder does not exists and it cannot be created.</p>"),
            ErrorKind::IOReadFolder(path) => write!(f, "<p>Error while trying to read the following folder:</p><p>{:?}</p>", path),
            //-----------------------------------------------------//
            //                TSV-related Errors
            //-----------------------------------------------------//
            ErrorKind::ImportTSVIncorrectRow(row, column) => write!(f, "<p>This TSV file has an error in the <b>row <i>{}</i></b>, <b>field <i>{}</i></b> (both starting at 1). Please, check it and make sure the value in that field is a valid value for that column.</p>", row + 1, column + 1),
            ErrorKind::ImportTSVWrongTypeTable => write!(f, "<p>This TSV file either belongs to another table, to a localisation PackedFile, it's broken or it's incompatible with RPFM.</p>"),
            ErrorKind::ImportTSVWrongTypeLoc => write!(f, "<p>This TSV file either belongs to a DB table, it's broken or it's incompatible with RPFM.</p>"),
            ErrorKind::ImportTSVWrongVersion => write!(f, "<p>This TSV file belongs to another version of this table. If you want to use it, consider creating a new empty table, fill it with enough empty rows, open this file in a TSV editor, like Excel or LibreOffice, and copy column by column.</p><p>A more automatic solution is on the way, but not yet there.</p>"),
            ErrorKind::TSVErrorGeneric => write!(f, "<p>Error while trying to import/export a TSV file.</p>"),

            //-----------------------------------------------------//
            //                 PackFile Errors
            //-----------------------------------------------------//
            ErrorKind::OpenPackFileGeneric(error) => write!(f, "<p>Error while trying to open a PackFile:</p><p>{}</p>", error),
            ErrorKind::SavePackFileGeneric(error) => write!(f, "<p>Error while trying to save the currently open PackFile:</p><p>{}</p>", error),
            ErrorKind::PackFileNotSupported => write!(f, "
            <p>The file is not a supported PackFile.</p>
            <p>For now, we only support:</p>
            <ul>
            <li>- Warhammer 2.</li>
            <li>- Warhammer.</li>
            <li>- Attila.</li>
            <li>- Rome 2.</li>
            <li>- Arena.</li>
            </ul>"),
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

            //-----------------------------------------------------//
            //                PackedFile Errors
            //-----------------------------------------------------//
            ErrorKind::PackedFileNotFound => write!(f, "<p>This PackedFile no longer exists in the PackFile.</p>"),
            ErrorKind::PackedFileIsOpen => write!(f, "<p>That operation cannot be done while the PackedFile involved on it is open. Please, close it by selecting a Folder/PackFile in the TreeView and try again.</p>"),
            ErrorKind::PackedFileIsOpenInAnotherView => write!(f, "<p>That PackedFile is already open in another view. Opening the same PackedFile in multiple views is not supported.</p>"),
            ErrorKind::PackedFileDataCouldNotBeLoaded => write!(f, "<p>This PackedFile's data could not be loaded. This means RPFM can no longer read the PackFile from the disk.</p>"),
            ErrorKind::PackedFileSizeIsNotWhatWeExpect(reported_size, expected_size) => write!(f, "<p>This PackedFile's reported size is <i><b>{}</b></i> bytes, but we expected it to be <i><b>{}</b></i> bytes. This means that either the decoding logic in RPFM is broken for this PackedFile, or this PackedFile is corrupted.</p>", reported_size, expected_size),

            //--------------------------------//
            // DB Table Errors
            //--------------------------------//
            ErrorKind::DBTableIsNotADBTable => write!(f, "<p>This is either not a DB Table, or it's a DB Table but it's corrupted.</p>"),
            ErrorKind::DBTableContainsListField => write!(f, "<p>This specific table version uses a currently unimplemented type (List), so is undecodeable, for now.</p>"),
            ErrorKind::DBTableReplaceInvalidData => write!(f, "<p>Error while trying to replace the data of a Cell.</p><p>This means you tried to replace a number cell with text, or used a too big, too low or invalid number. Don't do it. It wont end well.</p>"),
            ErrorKind::DBTableApplyMathsOverflow => write!(f, "<p>The operation you just tried to do gives an invalid result. This means that the result value is bigger or lower than the maximum or minimum admited by the type in the cells.</p>"),
            ErrorKind::DBTableDecode(cause) => write!(f, "<p>Error while trying to decode the DB Table:</p><p>{}</p>", cause),
            ErrorKind::DBTableEmptyWithNoTableDefinition => write!(f, "<p>This DB Table is empty and there is not a Table Definition for it. That means is undecodeable.</p>"),
            ErrorKind::SchemaNotFound => write!(f, "<p>There is no Schema for the Game Selected.</p>"),
            ErrorKind::SchemaTableDefinitionNotFound => write!(f, "<p>There is no Table Definition for this specific version of the table in the Schema.</p>"),

            //--------------------------------//
            // RigidModel Errors
            //--------------------------------//
            ErrorKind::RigidModelDecode(cause) => write!(f, "<p>Error while trying to decode the RigidModel PackedFile:</p><p>{}</p>", cause),
            ErrorKind::RigidModelNotSupportedFile => write!(f, "<p>This file is not a Supported RigidModel file.</p>"),
            ErrorKind::RigidModelNotSupportedType => write!(f, "<p>This RigidModel's Type is not currently supported.</p>"),
            ErrorKind::RigidModelPatchToWarhammer(cause) => write!(f, "<p>Error while trying to patch the RigidModel file:</p><p>{}</p>", cause),
            ErrorKind::RigidModelUnknownMaskTypeFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Texture with unknown Mask Type found.</li></ul>"),
            ErrorKind::RigidModelTextureDirectoryNotFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Texture Directories not found.</li></ul>"),
            ErrorKind::RigidModelDecalTextureDirectoryNotFound => write!(f, "<p>Error while trying to decode the RigidModel file:</p><p><ul><li>Decal Texture Directory not found.</li></ul>"),

            //--------------------------------//
            // Text Errors
            //--------------------------------//

            // Error for when a Text PackedFile fails to decode.
            ErrorKind::TextDecode(cause) => write!(f, "<p>Error while trying to decode the Text PackedFile:</p><p>{}</p>", cause),
            ErrorKind::NoTypesFileFound => write!(f, "<p>There is no Types file for the current Game Selected, so you can't use Kailua.</p>"),
            ErrorKind::KailuaNotFound => write!(f, "<p>Kailua executable not found. Install it and try again.</p>"),

            //--------------------------------//
            // Loc Errors
            //--------------------------------//

            // Error for when a Loc PackedFile fails to decode.
            ErrorKind::LocDecode(cause) => write!(f, "<p>Error while trying to decode the Loc PackedFile:</p><p>{}</p>", cause),
            ErrorKind::LocPackedFileIsNotALocPackedFile => write!(f, "<p>This is either not a Loc PackedFile, or it's a Loc PackedFile but it's corrupted.</p>"),
            ErrorKind::LocPackedFileCorrupted => write!(f, "<p>This Loc PackedFile seems to be corrupted.</p>"),

            //--------------------------------//
            // Image Errors
            //--------------------------------//

            // Error for when an Image fails to decode.
            ErrorKind::ImageDecode(cause) => write!(f, "<p>Error while trying to decode the Image PackedFile:</p><p>{}</p>", cause),

            //-----------------------------------------------------//
            //                Decoding Errors
            //-----------------------------------------------------//
            ErrorKind::StringFromUTF8 => write!(f, "<p>Error while converting data to an UTF-8 String.</p>"),
            ErrorKind::HelperDecodingEncodingError(cause) => write!(f, "{}", cause),

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

            //-----------------------------------------------------//
            //                 Special Errors
            //-----------------------------------------------------//
            ErrorKind::PatchSiegeAIEmptyPackFile => write!(f, "<p>This packfile is empty, so we can't patch it.</p>"),
            ErrorKind::PatchSiegeAINoPatchableFiles => write!(f, "<p>There are not files in this Packfile that could be patched/deleted.</p>"),
            ErrorKind::OperationNotAllowedWithPackedFileOpen => write!(f, "<p>This operation cannot be done while there is a PackedFile open. Select a folder or the PackFile to close it and try again.</p>"),

            //-----------------------------------------------------//
            //                Contextual Errors
            //-----------------------------------------------------//
            ErrorKind::NameAlreadyInUseInThisPath => write!(f, "<p>The provided name is already in use in the current path.</p>"),
            ErrorKind::ExtractError(errors) => write!(f, "<p>There has been a problem extracting the following files:</p><ul>{:#?}</ul>", errors),
            ErrorKind::MassImport(errors) => write!(f, "<p>The following files returned error when trying to import them:</p><ul>{}</ul><p>No files have been imported.</p>", errors),
            ErrorKind::EmptyInput => write!(f, "<p>Only my hearth can be empty.</p>"),
            ErrorKind::InvalidInput => write!(f, "<p>There are characters that shall never be used.</p>"),
            ErrorKind::UnchangedInput => write!(f, "<p>Like war, nothing changed.</p>"),
            ErrorKind::NoFilesToImport => write!(f, "<p>It's mathematically impossible to successfully import zero TSV files.</p>"),
            ErrorKind::FileAlreadyInPackFile => write!(f, "<p>The provided file/s already exists in the current path.</p>"),
            ErrorKind::FolderAlreadyInPackFile => write!(f, "<p>That folder already exists in the current path.</p>"),

            //-----------------------------------------------------//
            //                  Common Errors
            //-----------------------------------------------------//
            ErrorKind::JsonErrorSyntax => write!(f, "<p>Error while trying to read JSON data:</p><p>Invalid syntax found.</p>"),
            ErrorKind::JsonErrorData => write!(f, "<p>Error while trying to read JSON data:</p><p>Semantically incorrect data found.</p>"),
            ErrorKind::JsonErrorEOF => write!(f,"<p>Error while trying to read JSON data:</p><p>Unexpected EOF found.</p>"),
        }
    }
}

//------------------------------------------------------------//
//         Extra Implementations for the From Trait
//------------------------------------------------------------//

/// Implementation to create a custom error from an ErrorKind.
impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { context: Context::new(kind) }
    }
}

/// Implementation to create a custom error from an Context.
impl From<Context<ErrorKind>> for Error {
    fn from(context: Context<ErrorKind>) -> Error {
        Error { context }
    }
}

/// Implementation to create a custom error from a serde_json::Error. Based on the "From" used to convert it to std::io::Error.
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {

        // Get his category, and create an error based on that.
        match error.classify() {
            Category::Io => Error::from(ErrorKind::IOGeneric),
            Category::Syntax => Error::from(ErrorKind::JsonErrorSyntax),
            Category::Data => Error::from(ErrorKind::JsonErrorData),
            Category::Eof => Error::from(ErrorKind::JsonErrorEOF),
        }
    }
}

/// Implementation to create a custom error from a csv::Error. Based on the "From" used to convert it to std::io::Error.
impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Error {

        // Get his category, and create an error based on that.
        match error.kind() {
            csv::ErrorKind::Io(_) => Error::from(ErrorKind::IOGeneric),
            _ => Error::from(ErrorKind::TSVErrorGeneric)
        }
    }
}

/// Implementation to create a custom error from a FromUTF8Error.
impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Error {
        Error::from(ErrorKind::StringFromUTF8)
    }
}

/// Implementation to create a custom error from a Reqwest Error.
impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Error {
        Error::from(ErrorKind::NetworkGeneric)
    }
}

/// Implementation to create a custom error from a Toml Error.
impl From<ser::Error> for Error {
    fn from(_: ser::Error) -> Error {
        Error::from(ErrorKind::TOMLSerializerError)
    }
}

/// Implementation to create a custom error from a std::io::Error. Based on the "From" used to convert it to std::io::Error.
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {

        // Get his category, and create an error based on that.
        match error.kind() {
            io::ErrorKind::NotFound => Error::from(ErrorKind::IOFileNotFound),
            io::ErrorKind::PermissionDenied => Error::from(ErrorKind::IOPermissionDenied),
            _ => Error::from(ErrorKind::IOGeneric),
        }
    }
}
