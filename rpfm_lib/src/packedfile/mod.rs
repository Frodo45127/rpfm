//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the Fn, Structs and Impls common to at least 2 PackedFile types.

use serde_derive::{Serialize, Deserialize};

use rpfm_error::{Error, ErrorKind, Result};

use std::{fmt, fmt::Display};
use std::ops::Deref;

use crate::packedfile::table::{db::DB, loc::Loc};
use crate::packfile::packedfile::RawPackedFile;
use crate::schema::{FieldType, Schema};
use crate::SCHEMA;

pub mod rigidmodel;
pub mod table;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This enum represents a ***decoded `PackedFile`***, 
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(PartialEq, Clone, Debug)]
pub enum DecodedPackedFile {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CEO,
    DB(DB),
    Image,
    Loc(Loc),
    MatchedCombat,
    RigidModel,
    StarPos,
    Text,
    Unknown,
}

/// This enum specifies the different types of `PackedFile` we can find in a `PackFile`.
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(Clone, Debug)]
pub enum PackedFileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CEO,
    DB,
    Image,
    Loc,
    MatchedCombat,
    RigidModel,
    StarPos,
    Text,
    Unknown,
}

/// This enum is used to store different types of data in a unified way. Used, for example, to store the data from each field in a DB Table.
///
/// NOTE: `Sequence` it's a recursive type. A Sequence/List means you got a repeated sequence of fields
/// inside a single field. Used, for example, in certain model tables.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DecodedData {
    Boolean(bool),
    Float(f32),
    Integer(i32),
    LongInteger(i64),
    StringU8(String),
    StringU16(String),
    OptionalStringU8(String),
    OptionalStringU16(String),
    Sequence(Vec<Vec<DecodedData>>)
}

//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(data: &RawPackedFile) -> Result<Self> {
        let schema = SCHEMA.lock().unwrap();
        match PackedFileType::get_packed_file_type(data.get_path()) {
            PackedFileType::DB => {
                match schema.deref() {
                    Some(schema) => {
                        let name = data.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let data = data.get_data()?;
                        let packed_file = DB::read(&data, name, &schema)?;
                        Ok(DecodedPackedFile::DB(packed_file))
                    }
                    None => Ok(DecodedPackedFile::Unknown),
                }
            }

            PackedFileType::Loc => {
                match schema.deref() {
                    Some(schema) => {
                        let data = data.get_data()?;
                        let packed_file = Loc::read(&data, &schema)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Ok(DecodedPackedFile::Unknown),
                }
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(data: &RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(data.get_path()) {
            PackedFileType::DB => {
                let name = data.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let data = data.get_data()?;
                let packed_file = DB::read(&data, name, &schema)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Loc => {
                let data = data.get_data()?;
                let packed_file = Loc::read(&data, &schema)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    pub fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DecodedPackedFile::DB(data) => data.save(),
            DecodedPackedFile::Loc(data) => data.save(),
            _=> unimplemented!(),
        }
    }
}

//----------------------------------------------------------------//
// Implementations for `PackedFileType`.
//----------------------------------------------------------------//

/// Display implementation of `PackedFileType`.
impl Display for PackedFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackedFileType::Anim => write!(f, "Anim"),
            PackedFileType::AnimFragment => write!(f, "AnimFragment"),
            PackedFileType::AnimPack => write!(f, "AnimPack"),
            PackedFileType::AnimTable => write!(f, "AnimTable"),
            PackedFileType::CEO => write!(f, "CEO"),
            PackedFileType::DB => write!(f, "DB Table"),
            PackedFileType::Image => write!(f, "Image"),
            PackedFileType::Loc => write!(f, "Loc Table"),
            PackedFileType::MatchedCombat => write!(f, "Matched Combat"),
            PackedFileType::RigidModel => write!(f, "RigidModel"),
            PackedFileType::StarPos => write!(f, "StartPos"),
            PackedFileType::Text => write!(f, "Text"),
            PackedFileType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Implementation of `PackedFileType`.
impl PackedFileType {

    /// This function returns the type of the `PackedFile` at the provided path.
    pub fn get_packed_file_type(path: &[String]) -> Self {
        if let Some(packedfile_name) = path.last() {

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            if path[0] == "db" { PackedFileType::DB }

            // If it ends in ".loc", it's a localisation PackedFile.
            else if packedfile_name.ends_with(".loc") { PackedFileType::Loc }

            // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
            else if packedfile_name.ends_with(".rigid_model_v2") { PackedFileType::RigidModel }

            // If it ends in any of these, it's a plain text PackedFile.
            else if packedfile_name.ends_with(".lua") ||
                    packedfile_name.ends_with(".xml") ||
                    packedfile_name.ends_with(".xml.shader") ||
                    packedfile_name.ends_with(".xml.material") ||
                    packedfile_name.ends_with(".variantmeshdefinition") ||
                    packedfile_name.ends_with(".environment") ||
                    packedfile_name.ends_with(".lighting") ||
                    packedfile_name.ends_with(".wsmodel") ||
                    packedfile_name.ends_with(".csv") ||
                    packedfile_name.ends_with(".tsv") ||
                    packedfile_name.ends_with(".inl") ||
                    packedfile_name.ends_with(".battle_speech_camera") ||
                    packedfile_name.ends_with(".bob") ||
                    packedfile_name.ends_with(".cindyscene") ||
                    packedfile_name.ends_with(".cindyscenemanager") ||
                    //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                    packedfile_name.ends_with(".txt") { PackedFileType::Text }

            // If it ends in any of these, it's an image.
            else if packedfile_name.ends_with(".jpg") ||
                    packedfile_name.ends_with(".jpeg") ||
                    packedfile_name.ends_with(".tga") ||
                    packedfile_name.ends_with(".dds") ||
                    packedfile_name.ends_with(".png") { PackedFileType::Image }

            // Otherwise, we don't have a decoder for that PackedFile... yet.
            else { PackedFileType::Unknown }
        }

        // If we didn't got a name, it means something broke. Return none.
        else { PackedFileType::Unknown }
    }
}

//----------------------------------------------------------------//
// Implementations for `DecodedData`.
//----------------------------------------------------------------//

/// Display implementation of `DecodedData`.
impl Display for DecodedData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodedData::Boolean(_) => write!(f, "Boolean"),
            DecodedData::Float(_) => write!(f, "Float"),
            DecodedData::Integer(_) => write!(f, "Integer"),
            DecodedData::LongInteger(_) => write!(f, "LongInteger"),
            DecodedData::StringU8(_) => write!(f, "StringU8"),
            DecodedData::StringU16(_) => write!(f, "StringU16"),
            DecodedData::OptionalStringU8(_) => write!(f, "OptionalStringU8"),
            DecodedData::OptionalStringU16(_) => write!(f, "OptionalStringU16"),
            DecodedData::Sequence(_) => write!(f, "Sequence"),
        }
    }
}

/// PartialEq implementation of `DecodedData`. We need this implementation due to the float comparison being... special.
impl PartialEq for DecodedData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DecodedData::Boolean(x), DecodedData::Boolean(y)) => x == y,
            (DecodedData::Float(x), DecodedData::Float(y)) => ((x * 1_000_000f32).round() / 1_000_000f32) == ((y * 1_000_000f32).round() / 1_000_000f32),
            (DecodedData::Integer(x), DecodedData::Integer(y)) => x == y,
            (DecodedData::LongInteger(x), DecodedData::LongInteger(y)) => x == y,
            (DecodedData::StringU8(x), DecodedData::StringU8(y)) => x == y,
            (DecodedData::StringU16(x), DecodedData::StringU16(y)) => x == y,
            (DecodedData::OptionalStringU8(x), DecodedData::OptionalStringU8(y)) => x == y,
            (DecodedData::OptionalStringU16(x), DecodedData::OptionalStringU16(y)) => x == y,
            (DecodedData::Sequence(x), DecodedData::Sequence(y)) => x == y,
            _ => false
        }
    }
}

/// Implementation of `DecodedData`.
impl DecodedData {

    /// Default implementation of `DecodedData`.
    pub fn default(field_type: &FieldType) -> Self {
        match field_type {
            FieldType::Boolean => DecodedData::Boolean(false),
            FieldType::Float => DecodedData::Float(0.0),
            FieldType::Integer => DecodedData::Integer(0),
            FieldType::LongInteger => DecodedData::LongInteger(0),
            FieldType::StringU8 => DecodedData::StringU8("".to_owned()),
            FieldType::StringU16 => DecodedData::StringU16("".to_owned()),
            FieldType::OptionalStringU8 => DecodedData::OptionalStringU8("".to_owned()),
            FieldType::OptionalStringU16 => DecodedData::OptionalStringU16("".to_owned()),
            FieldType::Sequence(fields) => DecodedData::Sequence(vec![fields.iter().map(|x| Self::default(&x.field_type)).collect::<Vec<DecodedData>>()]),
        }
    }

    /// This functions checks if the type of an specific `DecodedData` is the one it should have, according to the provided `FieldType`.
    pub fn is_field_type_correct(decoded_data: &DecodedData, field_type: FieldType) -> bool {
        match decoded_data {
            DecodedData::Boolean(_) => field_type == FieldType::Boolean,
            DecodedData::Float(_) => field_type == FieldType::Float,
            DecodedData::Integer(_) => field_type == FieldType::Integer,
            DecodedData::LongInteger(_) => field_type == FieldType::LongInteger,
            DecodedData::StringU8(_) => field_type == FieldType::StringU8,
            DecodedData::StringU16(_) => field_type == FieldType::StringU16,
            DecodedData::OptionalStringU8(_) => field_type == FieldType::OptionalStringU8,
            DecodedData::OptionalStringU16(_) => field_type == FieldType::OptionalStringU16,
            DecodedData::Sequence(_) => if let FieldType::Sequence(_) = field_type { true } else { false },
        }
    }
}

//----------------------------------------------------------------//
// Generic Functions for PackedFiles.
//----------------------------------------------------------------//































































/*


/// This function merges (if it's possible) the provided DB and LOC tables into one with the name and, if asked,
/// it deletes the source files. Table_type means true: DB, false: LOC.
pub fn merge_tables( 
    pack_file: &mut PackFile,
    source_paths: &[Vec<String>],
    name: &str,
    delete_source_paths: bool,
    table_type: bool,
) -> Result<(Vec<String>, Vec<PathType>)> {
    
    let mut db_files = vec![];
    let mut loc_files = vec![];

    // Decode them depending on their type.
    for path in source_paths {
        if let Some(packed_file) = pack_file.get_ref_packed_file_by_path(path) {
            let packed_file_data = packed_file.get_data()?;
            
            if table_type { 
                if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    db_files.push(DB::read(&packed_file_data, &path[1], &schema)?); 
                }
                else { return Err(ErrorKind::SchemaNotFound)? }
            }
            else { loc_files.push(Loc::read(&packed_file_data)?); }
        }
    }

    // Merge them all into one, and return error if any problem arise.
    let packed_file_data = if table_type {
        let mut final_entries_list = vec![];
        let mut version = -2;
        let mut table_definition = Definition::new(0);

        for table in &mut db_files {
            if version == -2 { 
                version = table.definition.version; 
                table_definition = table.definition.clone();
            }
            else if table.definition.version != version { return Err(ErrorKind::InvalidFilesForMerging)? }

            final_entries_list.append(&mut table.entries);
        }

        let mut new_table = DB::new(&db_files[0].name, &table_definition);
        new_table.entries = final_entries_list;
        new_table.save()
    }

    else {
        let mut final_entries_list = vec![];
        for table in &mut loc_files {
            final_entries_list.append(&mut table.entries);
        }

        let mut new_table = Loc::new();
        new_table.entries = final_entries_list;
        new_table.save()
    };

    // And then, we reach the part where we have to do the "saving to PackFile" stuff.
    let mut path = source_paths[0].to_vec();
    path.pop();
    path.push(name.to_owned());
    let packed_file = PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, packed_file_data);

    // If we want to remove the source files, this is the moment.
    let mut deleted_paths = vec![];
    if delete_source_paths {
        for path in source_paths {
            pack_file.remove_packed_file_by_path(path);
            deleted_paths.push(path);
        }
    }

    // Prepare the paths to return.
    let added_path = pack_file.add_packed_files(&[packed_file], true)?.get(0).ok_or_else(|| Error::from(ErrorKind::ReservedFiles))?.to_vec();
    deleted_paths.retain(|x| x != &&added_path);

    let mut tree_paths = vec![];
    for path in &deleted_paths {
        tree_paths.push(PathType::File(path.to_vec()));
    }
    Ok((added_path, tree_paths))
}

//----------------------------------------------------------------//
// Mass-TSV Functions for PackedFiles.
//----------------------------------------------------------------//
/*
/// This function is used to Mass-Import TSV files into a PackFile. Note that this will OVERWRITE any
/// existing PackedFile that has a name conflict with the TSV files provided.
pub fn tsv_mass_import(
    tsv_paths: &[PathBuf],
    name: Option<String>,
    pack_file: &mut PackFile
) -> Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {

    // Create a list of PackedFiles succesfully imported, and another for the ones that didn't work.
    // The a third one to return the PackedFiles that were overwritten, so the UI can have an easy time updating his TreeView.
    let mut packed_files: Vec<PackedFile> = vec![];
    let mut packed_files_to_remove = vec![];
    let mut error_files = vec![];

    for path in tsv_paths {

        // We open it and read it to a string. We use the first row to check what kind of TSV is, and the second one we ignore it.
        let mut tsv = String::new();
        BufReader::new(File::open(&path)?).read_to_string(&mut tsv)?;

        // We get his first line, if it have it. Otherwise, we return an error in this file.
        if let Some(line) = tsv.lines().next() {

            // Split the first line by \t so we can get the info of the table. Only if we have 2 items, continue.
            let tsv_info = line.split('\t').collect::<Vec<&str>>();
            if tsv_info.len() == 2 {

                // Get the type and the version of the table, and with that, get his definition.
                let table_type = tsv_info[0];
                let table_version = match tsv_info[1].parse::<i32>() {
                    Ok(version) => version,
                    Err(_) => {
                        error_files.push(path.to_string_lossy().to_string()); 
                        continue
                    }
                };
                
                let table_definition = if let Some(ref schema) = *SCHEMA.lock().unwrap() {
                    schema.get_versioned_file_db(&table_type)?.get_version(table_version)?.clone()
                } else { error_files.push(path.to_string_lossy().to_string()); continue };

                // Then, import whatever we have and, depending on what we have, save it.
                match import_tsv(&table_definition, &path, &table_type, table_version) {
                    Ok(data) => {
                        match table_type {

                            // Loc Tables.
                            "Loc PackedFile" => {
                                let mut loc = Loc::new();
                                loc.entries = data;
                                let raw_data = loc.save();

                                // Depending on the name received, call it one thing or another.
                                let name = match name {
                                    Some(ref name) => name.to_string(),
                                    None => path.file_stem().unwrap().to_str().unwrap().to_string(),
                                };

                                let mut path = vec!["text".to_owned(), "db".to_owned(), format!("{}.loc", name)];

                                // If that path already exists in the list of new PackedFiles to add, change it using the index.
                                let mut index = 1;
                                while packed_files.iter().any(|x| x.get_path() == &*path) {
                                    path[2] = format!("{}_{}.loc", name, index);
                                    index += 1;
                                }

                                // If that path already exist in the PackFile, add it to the "remove" list.
                                if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                                // Create and add the new PackedFile to the list of PackedFiles to add.
                                packed_files.push(PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, raw_data));
                            }
        
                            // DB Tables.
                            _ => {
                                let mut db = DB::new(table_type, &table_definition);
                                db.entries = data;
                                let raw_data = db.save();

                                // Depending on the name received, call it one thing or another.
                                let name = match name {
                                    Some(ref name) => name.to_string(),
                                    None => path.file_stem().unwrap().to_str().unwrap().to_string(),
                                };

                                let mut path = vec!["db".to_owned(), table_type.to_owned(), name.to_owned()];
                        
                                // If that path already exists in the list of new PackedFiles to add, change it using the index.
                                let mut index = 1;
                                while packed_files.iter().any(|x| x.get_path() == &*path) {
                                    path[2] = format!("{}_{}", name, index);
                                    index += 1;
                                }
                                
                                // If that path already exists in the PackFile, add it to the "remove" list.
                                if pack_file.packedfile_exists(&path) { packed_files_to_remove.push(path.to_vec()) }

                                // Create and add the new PackedFile to the list of PackedFiles to add.
                                packed_files.push(PackedFile::read_from_vec(path, pack_file.get_file_name(), get_current_time(), false, raw_data));
                            }
                        }
                    }
                    Err(_) => error_files.push(path.to_string_lossy().to_string()),
                }
            }
            else { error_files.push(path.to_string_lossy().to_string()) }
        }
        else { error_files.push(path.to_string_lossy().to_string()) }
    }

    // If any of the files returned error, return error.
    if !error_files.is_empty() {
        let error_files_string = error_files.iter().map(|x| format!("<li>{}</li>", x)).collect::<String>();
        return Err(ErrorKind::MassImport(error_files_string))?
    }

    // Get the "TreePath" of the new PackFiles to return them.
    let tree_path = packed_files.iter().map(|x| x.get_path().to_vec()).collect::<Vec<Vec<String>>>();

    // Remove all the "conflicting" PackedFiles from the PackFile, before adding the new ones.
    for packed_file_to_remove in &packed_files_to_remove {
        pack_file.remove_packed_file_by_path(packed_file_to_remove);
    }

    // We add all the files to the PackFile, and return success.
    pack_file.add_packed_files(&packed_files, true)?;
    Ok((packed_files_to_remove, tree_path))
}
*/
/// This function is used to Mass-Export TSV files from a PackFile. Note that this will OVERWRITE any
/// existing file that has a name conflict with the TSV files provided.
pub fn tsv_mass_export(
    export_path: &PathBuf,
    pack_file: &mut PackFile
) -> Result<String> {

    // Lists of PackedFiles that couldn't be exported for one thing or another and exported PackedFile names,
    // so we make sure we don't overwrite those with the following ones.
    let mut error_list = vec![];
    let mut exported_files = vec![];

    // If the PackedFile is a DB Table and we have an schema, try to decode it and export it.
    match *SCHEMA.lock().unwrap() {
        Some(ref schema) => {

            let mut packed_files = pack_file.get_ref_packed_files_by_path_start(&["db".to_owned()]);
            packed_files.append(&mut pack_file.get_ref_packed_files_by_path_end(&[".loc".to_owned()]));
            for packed_file in &mut packed_files {

                // We check if his path is empty first to avoid false positives related with "starts_with" function.
                if !packed_file.get_path().is_empty() {

                    if packed_file.get_path().starts_with(&["db".to_owned()]) && packed_file.get_path().len() == 3 {
                        match DB::read(&(packed_file.get_data()?), &packed_file.get_path()[1], &schema) {
                            Ok(db) => {

                                // His name will be "db_name_file_name.tsv". If that's taken, we'll add an index until we find one available.
                                let mut name = format!("{}_{}.tsv", packed_file.get_path()[1], packed_file.get_path().last().unwrap().to_owned());
                                let mut export_path = export_path.to_path_buf();

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                let mut index = 1;
                                while exported_files.contains(&name) {
                                    name = format!("{}_{}_{}.tsv", packed_file.get_path()[1], packed_file.get_path().last().unwrap().to_owned(), index);
                                    index += 1;
                                }

                                export_path.push(name.to_owned());
                                let headers = db.definition.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>();
                                match export_tsv(&db.entries, &export_path, &headers, (&packed_file.get_path()[1], db.definition.version)) {
                                    Ok(_) => exported_files.push(name.to_owned()),
                                    Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                                }
                            }
                            Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                        }
                    }
                    
                    // Otherwise, we check if it's a Loc PackedFile, and try to decode it and export it.
                    else if packed_file.get_path().last().unwrap().ends_with(".loc") {
                        match Loc::read(&(packed_file.get_data()?)) {
                            Ok(loc) => {

                                // His name will be "file_name.tsv". If that's taken, we'll add an index until we find one available.
                                let mut name = format!("{}.tsv", packed_file.get_path().last().unwrap().to_owned());
                                let mut export_path = export_path.to_path_buf();

                                // Checks to avoid overwriting exported files go here, in an infinite loop of life and death.
                                let mut index = 1;
                                while exported_files.contains(&name) {
                                    name = format!("{}_{}.tsv", packed_file.get_path().last().unwrap().to_owned(), index);
                                    index += 1;
                                }

                                export_path.push(name.to_owned());
                                let headers = schema.get_versioned_file_loc()?.get_version(1)?.fields.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>();
                                match export_tsv(&loc.entries, &export_path, &headers, ("Loc PackedFile", 1)) {
                                    Ok(_) => exported_files.push(name.to_owned()),
                                    Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                                }
                            }
                            Err(error) => error_list.push((packed_file.get_path().to_vec().join("\\"), error)),
                        }
                    }
                }
            }
        }
        None => error_list.push(("".to_string(), Error::from(ErrorKind::SchemaNotFound))),
    }

    // If there has been errors, return ok with the list of errors.
    if !error_list.is_empty() {
        let error_files_string = error_list.iter().map(|x| format!("<li>{}</li>", x.0)).collect::<String>();
        Ok(format!("<p>All exportable files have been exported, except the following ones:</p><ul>{}</ul>", error_files_string))
    }

    // Otherwise, just return success and an empty error list.
    else { Ok("<p>All exportable files have been exported.</p>".to_owned()) }
}

*/