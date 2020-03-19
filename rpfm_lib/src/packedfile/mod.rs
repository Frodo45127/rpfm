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
use crate::packedfile::ca_vp8::CaVp8;
use crate::packedfile::image::Image;
use crate::packedfile::table::{db::DB, loc::Loc};
use crate::packedfile::text::{Text, TextType};
use crate::packedfile::rigidmodel::RigidModel;
use crate::packfile::packedfile::{PackedFile, RawPackedFile};
use crate::schema::Schema;
use crate::SCHEMA;

pub mod ca_vp8;
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
    CaVp8(CaVp8),
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackedFileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CaVp8,
    CEO,
    DB,
    Image,
    Loc,
    MatchedCombat,
    RigidModel,
    StarPos,

    /// This one is an exception, as it contains the MimeType of the Text PackedFile, so we can do things depending on the type.
    Text(TextType),

    /// This one is special. It's used just in case we want to open the Dependency PackFile List as a PackedFile.
    DependencyPackFilesList,
    Unknown,
}

//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(raw_packed_file: &RawPackedFile) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file.get_path()) {

            PackedFileType::CaVp8 => {
                let data = raw_packed_file.get_data()?;
                let packed_file = CaVp8::read(data)?;
                Ok(DecodedPackedFile::CaVp8(packed_file))
            }

            PackedFileType::DB => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let data = raw_packed_file.get_data()?;
                        let packed_file = DB::read(&data, name, &schema, false)?;
                        Ok(DecodedPackedFile::DB(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::Image => {
                let data = raw_packed_file.get_data()?;
                let packed_file = Image::read(&data)?;
                Ok(DecodedPackedFile::Image(packed_file))
            }

            PackedFileType::Loc => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data()?;
                        let packed_file = Loc::read(&data, &schema, false)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::Text(_) => {
                let data = raw_packed_file.get_data()?;
                let mut packed_file = Text::read(&data)?;
                let packed_file_type = PackedFileType::get_packed_file_type(raw_packed_file.get_path());
                if let PackedFileType::Text(text_type) = packed_file_type {
                    packed_file.set_text_type(text_type);
                }
                Ok(DecodedPackedFile::Text(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(raw_packed_file: &RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file.get_path()) {
            PackedFileType::CaVp8 => {
                let data = raw_packed_file.get_data()?;
                let packed_file = CaVp8::read(data)?;
                Ok(DecodedPackedFile::CaVp8(packed_file))
            }

            PackedFileType::DB => {
                let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let data = raw_packed_file.get_data()?;
                let packed_file = DB::read(&data, name, &schema, false)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Image => {
                let data = raw_packed_file.get_data()?;
                let packed_file = Text::read(&data)?;
                Ok(DecodedPackedFile::Text(packed_file))
            }

            PackedFileType::Loc => {
                let data = raw_packed_file.get_data()?;
                let packed_file = Loc::read(&data, &schema, false)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::Text(_) => {
                let data = raw_packed_file.get_data()?;
                let mut packed_file = Text::read(&data)?;
                let packed_file_type = PackedFileType::get_packed_file_type(raw_packed_file.get_path());
                if let PackedFileType::Text(text_type) = packed_file_type {
                    packed_file.set_text_type(text_type);
                }
                Ok(DecodedPackedFile::Text(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    ///
    /// Keep in mind this should only work for PackedFiles with saving support.
    pub fn encode(&self) -> Option<Result<Vec<u8>>> {
        match self {
            DecodedPackedFile::CaVp8(data) => Some(data.save()),
            DecodedPackedFile::DB(data) => Some(data.save()),
            DecodedPackedFile::Loc(data) => Some(data.save()),
            DecodedPackedFile::Text(data) => Some(data.save()),
            _=> None,
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
            PackedFileType::CaVp8 => write!(f, "CA_VP8"),
            PackedFileType::CEO => write!(f, "CEO"),
            PackedFileType::DB => write!(f, "DB Table"),
            PackedFileType::DependencyPackFilesList => write!(f, "Dependency PackFile List"),
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

    /// This function returns the type of the `PackedFile` at the provided path based on the path itself.
    pub fn get_packed_file_type(path: &[String]) -> Self {
        if let Some(packedfile_name) = path.last() {
            if packedfile_name.ends_with(table::loc::EXTENSION) { PackedFileType::Loc }
            else if packedfile_name.ends_with(rigidmodel::EXTENSION) { PackedFileType::RigidModel }
            else if packedfile_name.ends_with(ca_vp8::EXTENSION) { PackedFileType::CaVp8 }
            else if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| packedfile_name.ends_with(x)) {
                PackedFileType::Text(*text_type)
            }

            else if image::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(x)) {
                PackedFileType::Image
            }

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            else if path[0].to_lowercase() == "db" { PackedFileType::DB }

            // Otherwise, we don't have a decoder for that PackedFile... yet.
            else { PackedFileType::Unknown }
        }

        // If we didn't got a name, it means something broke. Return none.
        else { PackedFileType::Unknown }
    }

    /// This function returns the type of the provided `PackedFile` based on the data it contains.
    pub fn get_packed_file_type_by_data(packed_file: &PackedFile) -> Self {
        match packed_file.get_raw_data() {
            Ok(data) => {
                if let Some(packedfile_name) = packed_file.get_path().last() {
                    if packedfile_name.ends_with(rigidmodel::EXTENSION) {
                        return Self::RigidModel
                    }
                    else if image::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(x)) {
                        return Self::Image
                    }
                    else if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| packedfile_name.ends_with(x)) {
                        if Text::read(&data).is_ok() {
                            return Self::Text(*text_type)
                        }

                    }

                    if Loc::is_loc(&data) { Self::Loc }
                    else if DB::read_header(&data).is_ok() { Self::DB }
                    else if CaVp8::is_video(&data) { Self::CaVp8 }
                    else { Self::Unknown }
                }

                else { Self::Unknown }
            }
            Err(_) => Self::Unknown,
        }
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
            Self::CaVp8 |
            Self::CEO |
            Self::DB |
            Self::DependencyPackFilesList |
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
            Self::CaVp8 |
            Self::CEO |
            Self::DB |
            Self::DependencyPackFilesList |
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
            DecodedPackedFile::CaVp8(_) => PackedFileType::CaVp8,
            DecodedPackedFile::CEO => PackedFileType::CEO,
            DecodedPackedFile::DB(_) => PackedFileType::DB,
            DecodedPackedFile::Image(_) => PackedFileType::Image,
            DecodedPackedFile::Loc(_) => PackedFileType::Loc,
            DecodedPackedFile::MatchedCombat => PackedFileType::MatchedCombat,
            DecodedPackedFile::RigidModel(_) => PackedFileType::RigidModel,
            DecodedPackedFile::StarPos => PackedFileType::StarPos,
            DecodedPackedFile::Text(text) => PackedFileType::Text(text.get_text_type()),
            DecodedPackedFile::Unknown => PackedFileType::Unknown,
        }
    }
}
