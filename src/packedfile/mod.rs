// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;
extern crate failure;

use std::path::PathBuf;

use self::failure::Error;


pub mod loc;
pub mod db;
pub mod rigidmodel;

/*
--------------------------------------------------
             Traits for PackedFiles
--------------------------------------------------
*/

/// Trait `SerializableToCSV`: This trait needs to be implemented by all the structs that can be
/// export to and import from a csv file, like `LocData`.
pub trait SerializableToCSV {

    /// `import_csv`: Requires `&mut self` and `&PathBuf` with the path of the csv file, returns
    /// success, or an error.
    fn import_csv(&mut self, csv_file_path: &PathBuf) -> Result<(), Error>;

    /// `export_csv`: Requires `&self` and the destination path for the csv file, returns a success
    /// message, or an error.
    fn export_csv(&self, csv_file_path: &PathBuf) -> Result<String, Error>;
}