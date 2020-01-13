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
Module with all the code to decode/encode/interact with the different type of `PackedFiles`.

This module contains all the code related with interacting with the different type of `PackedFiles`
you can find in a `PackFile`. Here, you can find some generic enums used by the different `PackedFiles`.

For encoding/decoding/proper manipulation of the data in each type of `PackedFile`, check their respective submodules
!*/

use rayon::prelude::*;

use std::{fmt, fmt::Display};
use std::ops::Deref;

use rpfm_error::{Error, ErrorKind, Result};

use crate::DEPENDENCY_DATABASE;
use crate::packedfile::image::Image;
use crate::packedfile::table::{db::DB, loc::Loc};
use crate::packedfile::text::{Text, TextType};
use crate::packedfile::rigidmodel::RigidModel;
use crate::packfile::packedfile::RawPackedFile;
use crate::schema::Schema;
use crate::SCHEMA;

pub mod image;
pub mod rigidmodel;
pub mod table;
pub mod text;

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
    Image(Image),
    Loc(Loc),
    MatchedCombat,
    RigidModel(RigidModel),
    StarPos,
    Text(Text),
    Unknown,
}

/// This enum specifies the different types of `PackedFile` we can find in a `PackFile`.
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(Clone, Debug, Eq, PartialEq)]
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

    /// This one is an exception, as it contains the MimeType of the Text PackedFile, so we can do things depending on the type..
    Text(TextType),
    Unknown,
}

//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(data: &RawPackedFile) -> Result<Self> {
        match PackedFileType::get_packed_file_type(data.get_path()) {
            PackedFileType::DB => {
                let schema = SCHEMA.read().unwrap();
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

            PackedFileType::Image => {
                let data = data.get_data()?;
                let packed_file = Image::read(&data)?;
                Ok(DecodedPackedFile::Image(packed_file))
            }

            PackedFileType::Loc => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = data.get_data()?;
                        let packed_file = Loc::read(&data, &schema)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Ok(DecodedPackedFile::Unknown),
                }
            }

            PackedFileType::Text(_) => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
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

            PackedFileType::Image => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }

            PackedFileType::Loc => {
                let data = data.get_data()?;
                let packed_file = Loc::read(&data, &schema)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::Text(_) => {
                let data = data.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    pub fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DecodedPackedFile::DB(data) => data.save(),
            DecodedPackedFile::Image(_) => unimplemented!(),
            DecodedPackedFile::Loc(data) => data.save(),
            DecodedPackedFile::Text(data) => data.save(),
            _=> unimplemented!(),
        }
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the data.pack or equivalent of the game.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_table(&mut self) -> Result<(i32, i32)> {
        match self {
            DecodedPackedFile::DB(data) => {
                let mut dep_db = DEPENDENCY_DATABASE.lock().unwrap();
                if let Some(schema) = &*SCHEMA.read().unwrap() {
                    if let Some(vanilla_db) = dep_db.par_iter_mut()
                        .filter_map(|x| x.decode_return_ref_no_locks(&schema).ok())
                        .filter_map(|x| if let DecodedPackedFile::DB(y) = x { Some(y) } else { None })
                        .filter(|x| x.name == data.name)
                        .max_by(|x, y| x.get_definition().version.cmp(&y.get_definition().version)) {

                        let definition_new = vanilla_db.get_definition();
                        let definition_old = data.get_definition();
                        if definition_old != definition_new {
                            data.set_definition(&definition_new);
                            Ok((definition_old.version, definition_new.version))
                        }
                        else {
                            Err(ErrorKind::NoDefinitionUpdateAvailable.into())
                        }
                    }
                    else { Err(ErrorKind::NoTableInGameFilesToCompare.into()) }
                }
                else { Err(ErrorKind::SchemaNotFound.into()) }

            }
            _ => Err(ErrorKind::DBTableIsNotADBTable.into()),
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
            PackedFileType::Text(text_type) => write!(f, "Text, type: {:?}", text_type),
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
            else if packedfile_name.ends_with(".lua") { PackedFileType::Text(TextType::Lua) }
            else if packedfile_name.ends_with(".xml") ||
                    packedfile_name.ends_with(".xml.shader") ||
                    packedfile_name.ends_with(".xml.material") ||
                    packedfile_name.ends_with(".variantmeshdefinition") ||
                    packedfile_name.ends_with(".environment") ||
                    packedfile_name.ends_with(".lighting") ||
                    packedfile_name.ends_with(".wsmodel") { PackedFileType::Text(TextType::Xml) }

            else if packedfile_name.ends_with(".csv") ||
                    packedfile_name.ends_with(".tsv") ||
                    packedfile_name.ends_with(".inl") ||
                    packedfile_name.ends_with(".battle_speech_camera") ||
                    packedfile_name.ends_with(".bob") ||
                    packedfile_name.ends_with(".cindyscene") ||
                    packedfile_name.ends_with(".cindyscenemanager") ||
                    //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                    packedfile_name.ends_with(".txt") { PackedFileType::Text(TextType::Plain) }

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

    /// This function is a less strict version of the one implemented with the `Eq` trait.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict(&self, other: &Self) -> bool {
        match self {
            Self::Anim |
            Self::AnimFragment |
            Self::AnimPack |
            Self::AnimTable |
            Self::CEO |
            Self::DB |
            Self::Image |
            Self::Loc |
            Self::MatchedCombat |
            Self::RigidModel |
            Self::StarPos |
            Self::Unknown => self == other,
            Self::Text(_) => if let Self::Text(_) = other { true } else { false },
        }
    }

    /// This function is a less strict version of the one implemented with the `Eq` trait, adapted to work with slices of types instead of singular types.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict_slice(&self, others: &[Self]) -> bool {
        match self {
            Self::Anim |
            Self::AnimFragment |
            Self::AnimPack |
            Self::AnimTable |
            Self::CEO |
            Self::DB |
            Self::Image |
            Self::Loc |
            Self::MatchedCombat |
            Self::RigidModel |
            Self::StarPos |
            Self::Unknown => others.contains(&self),
            Self::Text(_) => others.iter().any(|x| if let Self::Text(_) = x { true } else { false }),
        }
    }
}

/// From implementation to get the type from a DecodedPackedFile.
impl From<&DecodedPackedFile> for PackedFileType {
    fn from(packed_file: &DecodedPackedFile) -> Self {
        match packed_file {
            DecodedPackedFile::Anim => PackedFileType::Anim,
            DecodedPackedFile::AnimFragment => PackedFileType::AnimFragment,
            DecodedPackedFile::AnimPack => PackedFileType::AnimPack,
            DecodedPackedFile::AnimTable => PackedFileType::AnimTable,
            DecodedPackedFile::CEO => PackedFileType::CEO,
            DecodedPackedFile::DB(_) => PackedFileType::DB,
            DecodedPackedFile::Image(_) => PackedFileType::Image,
            DecodedPackedFile::Loc(_) => PackedFileType::Loc,
            DecodedPackedFile::MatchedCombat => PackedFileType::MatchedCombat,
            DecodedPackedFile::RigidModel(_) => PackedFileType::RigidModel,
            DecodedPackedFile::StarPos => PackedFileType::StarPos,
            DecodedPackedFile::Text(_) => PackedFileType::Text(TextType::Plain),
            DecodedPackedFile::Unknown => PackedFileType::Unknown,
        }
    }
}
