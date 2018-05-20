// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.
extern crate csv;
extern crate failure;

use failure::Error;
use std::path::PathBuf;

pub mod loc;
pub mod db;
pub mod rigidmodel;

/*
--------------------------------------------------
             Traits for PackedFiles
--------------------------------------------------
*/

/// Trait `SerializableToTSV`: This trait needs to be implemented by all the structs that can be
/// export to and import from a TSV file, like `LocData` and `DBData`.
pub trait SerializableToTSV {

    /// `import_tsv`: Requires `&mut self`, a `&PathBuf` with the path of the TSV file and the name of our PackedFile.
    /// Returns success or an error.
    fn import_tsv(&mut self, tsv_file_path: &PathBuf, packed_file_type: &str) -> Result<(), Error>;

    /// `export_tsv`: Requires `&self`, the destination path for the TSV file and a name and a number (version)
    /// to put in the header of the TSV file. Returns a success message, or an error.
    fn export_tsv(&self, tsv_file_path: &PathBuf, extra_data: (&str, u32)) -> Result<String, Error>;
}
