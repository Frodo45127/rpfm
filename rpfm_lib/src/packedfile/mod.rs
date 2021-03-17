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

use crate::dependencies::Dependencies;
use crate::packedfile::animpack::AnimPack;
use crate::packedfile::ca_vp8::CaVp8;
use crate::packedfile::image::Image;
use crate::packedfile::table::{anim_fragment::AnimFragment, animtable::AnimTable, db::DB, loc::Loc, matched_combat::MatchedCombat};
use crate::packedfile::text::{Text, TextType};
use crate::packedfile::rigidmodel::RigidModel;
use crate::packedfile::uic::UIC;
use crate::packfile::packedfile::RawPackedFile;
use crate::schema::Schema;
use crate::SCHEMA;

pub mod animpack;
pub mod ca_vp8;
pub mod image;
pub mod rigidmodel;
pub mod table;
pub mod text;
pub mod uic;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This enum represents a ***decoded `PackedFile`***,
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(PartialEq, Clone, Debug)]
pub enum DecodedPackedFile {
    Anim,
    AnimFragment(AnimFragment),
    AnimPack(AnimPack),
    AnimTable(AnimTable),
    CaVp8(CaVp8),
    CEO,
    DB(DB),
    Image(Image),
    GroupFormations,
    Loc(Loc),
    MatchedCombat(MatchedCombat),
    RigidModel(RigidModel),
    StarPos,
    Text(Text),
    UIC(UIC),
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
    GroupFormations,
    Loc,
    MatchedCombat,
    RigidModel,
    StarPos,
    UIC,

    /// This one is an exception, as it contains the MimeType of the Text PackedFile, so we can do things depending on the type.
    Text(TextType),

    /// This one is special. It's used just in case we want to open the Dependency PackFile List as a PackedFile.
    DependencyPackFilesList,
    PackFileSettings,
    Unknown,
}

//----------------------------------------------------------------//
// Implementations for `DecodedPackedFile`.
//----------------------------------------------------------------//

/// Implementation of `DecodedPackedFile`.
impl DecodedPackedFile {

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode(raw_packed_file: &mut RawPackedFile) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file, true) {

            PackedFileType::AnimFragment => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = AnimFragment::read(&data, &schema, false)?;
                        Ok(DecodedPackedFile::AnimFragment(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::AnimPack => {
                let data = raw_packed_file.get_data()?;
                let packed_file = AnimPack::read(&data)?;
                Ok(DecodedPackedFile::AnimPack(packed_file))
            }

            PackedFileType::AnimTable => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = AnimTable::read(&data, &schema, false)?;
                        Ok(DecodedPackedFile::AnimTable(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::CaVp8 => {
                let data = raw_packed_file.get_data()?;
                let packed_file = CaVp8::read(data)?;
                Ok(DecodedPackedFile::CaVp8(packed_file))
            }

            PackedFileType::DB => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let packed_file = DB::read(&data, &name, &schema, false)?;
                        Ok(DecodedPackedFile::DB(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::Image => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = Image::read(&data)?;
                Ok(DecodedPackedFile::Image(packed_file))
            }

            PackedFileType::Loc => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = Loc::read(&data, &schema, false)?;
                        Ok(DecodedPackedFile::Loc(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            PackedFileType::MatchedCombat => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = MatchedCombat::read(&data, &schema, false)?;
                        Ok(DecodedPackedFile::MatchedCombat(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }
            PackedFileType::Text(text_type) => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let mut packed_file = Text::read(&data)?;
                packed_file.set_text_type(text_type);
                Ok(DecodedPackedFile::Text(packed_file))
            }

            PackedFileType::UIC => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let packed_file = UIC::read(&data, &schema)?;
                        Ok(DecodedPackedFile::UIC(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(raw_packed_file: &mut RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file, true) {

            PackedFileType::AnimFragment => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimFragment::read(&data, &schema, false)?;
                Ok(DecodedPackedFile::AnimFragment(packed_file))
            }

            PackedFileType::AnimPack => Self::decode(raw_packed_file),

            PackedFileType::AnimTable => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimTable::read(&data, &schema, false)?;
                Ok(DecodedPackedFile::AnimTable(packed_file))
            }

            PackedFileType::CaVp8 => Self::decode(raw_packed_file),

            PackedFileType::DB => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let packed_file = DB::read(&data, &name, &schema, false)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Image => Self::decode(raw_packed_file),

            PackedFileType::Loc => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = Loc::read(&data, &schema, false)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::MatchedCombat => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = MatchedCombat::read(&data, &schema, false)?;
                Ok(DecodedPackedFile::MatchedCombat(packed_file))
            }

            PackedFileType::Text(_) => Self::decode(raw_packed_file),

            PackedFileType::UIC => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UIC::read(&data, &schema)?;
                Ok(DecodedPackedFile::UIC(packed_file))
            }
            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function encodes a `DecodedPackedFile` into a `Vec<u8>`, returning it.
    ///
    /// Keep in mind this should only work for PackedFiles with saving support.
    pub fn encode(&self) -> Option<Result<Vec<u8>>> {
        match self {
            DecodedPackedFile::AnimFragment(data) => Some(data.save()),
            DecodedPackedFile::AnimPack(data) => Some(Ok(data.save())),
            DecodedPackedFile::AnimTable(data) => Some(data.save()),
            DecodedPackedFile::CaVp8(data) => Some(Ok(data.save())),
            DecodedPackedFile::DB(data) => Some(data.save()),
            DecodedPackedFile::Loc(data) => Some(data.save()),
            DecodedPackedFile::MatchedCombat(data) => Some(data.save()),
            DecodedPackedFile::Text(data) => Some(data.save()),
            DecodedPackedFile::UIC(data) => Some(Ok(data.save())),
            _=> None,
        }
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the data.pack or equivalent of the game.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_table(&mut self, dependencies: &Dependencies) -> Result<(i32, i32)> {
        match self {
            DecodedPackedFile::DB(data) => {
                let dep_db = dependencies.get_ref_dependency_database();
                if let Some(vanilla_db) = dep_db.par_iter()
                    .filter_map(|x| x.get_decoded_from_memory().ok())
                    .filter_map(|x| if let DecodedPackedFile::DB(y) = x { Some(y) } else { None })
                    .filter(|x| x.name == data.name)
                    .max_by(|x, y| x.get_ref_definition().get_version().cmp(&y.get_ref_definition().get_version())) {

                    let definition_new = vanilla_db.get_definition();
                    let definition_old = data.get_definition();
                    if definition_old != definition_new {
                        data.set_definition(&definition_new);
                        Ok((definition_old.get_version(), definition_new.get_version()))
                    }
                    else {
                        Err(ErrorKind::NoDefinitionUpdateAvailable.into())
                    }
                }
                else { Err(ErrorKind::NoTableInGameFilesToCompare.into()) }
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
            PackedFileType::GroupFormations => write!(f, "Group Formations"),
            PackedFileType::Loc => write!(f, "Loc Table"),
            PackedFileType::MatchedCombat => write!(f, "Matched Combat"),
            PackedFileType::RigidModel => write!(f, "RigidModel"),
            PackedFileType::StarPos => write!(f, "StartPos"),
            PackedFileType::UIC => write!(f, "UI Component"),
            PackedFileType::Text(text_type) => write!(f, "Text, type: {:?}", text_type),
            PackedFileType::PackFileSettings => write!(f, "PackFile Settings"),
            PackedFileType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Implementation of `PackedFileType`.
impl PackedFileType {

    /// This function returns the type of the provided `PackedFile` based on the info about them (path, name, extension,...).
    ///
    /// Strict mode also performs a search by checking the data directly if no type was found, but that's very slow. Think twice before using it.
    pub fn get_packed_file_type(packed_file: &RawPackedFile, strict_mode: bool) -> Self {

        // First, try with extensions.
        let path = packed_file.get_path();
        if let Some(packedfile_name) = path.last() {
            let packedfile_name = packedfile_name.to_lowercase();

            if packedfile_name.ends_with(table::loc::EXTENSION) {
                return Self::Loc;
            }

            if packedfile_name.ends_with(rigidmodel::EXTENSION) {
                return Self::RigidModel
            }

            if packedfile_name.ends_with(table::anim_fragment::EXTENSION) {
                return Self::AnimFragment;
            }

            if packedfile_name.ends_with(animpack::EXTENSION) {
                return Self::AnimPack
            }

            if packedfile_name.ends_with(ca_vp8::EXTENSION) {
                return Self::CaVp8;
            }

            if image::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(x)) {
                return Self::Image;
            }

            if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| packedfile_name.ends_with(x)) {
                return Self::Text(*text_type);
            }

            // If that failed, try types that need to be in a specific path.
            let path_str = path.iter().map(String::as_str).collect::<Vec<&str>>();
            if packedfile_name.ends_with(table::matched_combat::EXTENSION) && path_str.starts_with(&table::matched_combat::BASE_PATH) {
                return Self::MatchedCombat;
            }

            if packedfile_name.ends_with(table::animtable::EXTENSION) && path_str.starts_with(&table::animtable::BASE_PATH) {
                return Self::AnimTable;
            }

            // If that failed, check if it's in a folder which is known to only have specific files.
            if let Some(folder) = path.get(0) {
                let base_folder = folder.to_lowercase();
                if &base_folder == "db" {
                    return Self::DB;
                }

                if &base_folder == "ui" && (!packedfile_name.contains(".") || packedfile_name.ends_with(uic::EXTENSION)) {
                    return Self::UIC;
                }
            }

            // If nothing worked, then it's simple: if we enabled strict mode, check the data. If not, we don't know.
            // This is very slow when done over a lot of files, so be careful with it.
            if strict_mode {
                let data = packed_file.get_data().unwrap();

                if Text::read(&data).is_ok() {
                    return Self::Text(TextType::Plain);
                }

                if Loc::is_loc(&data) {
                    return Self::Loc;
                }

                if DB::read_header(&data).is_ok() {
                    return Self::DB;
                }

                if CaVp8::is_video(&data) {
                    return Self::CaVp8;
                }

                if UIC::is_ui_component(&data) {
                    return Self::UIC;
                }
            }
        }

        // If we reach this... we're clueless.
        Self::Unknown
    }

    /// This function is a less strict version of the one implemented with the `Eq` trait.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict(self, other: Self) -> bool {
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
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::RigidModel |
            Self::StarPos |
            Self::PackFileSettings |
            Self::UIC |
            Self::Unknown => self == other,
            Self::Text(_) => matches!(other, Self::Text(_)),
        }
    }

    /// This function is a less strict version of the one implemented with the `Eq` trait, adapted to work with slices of types instead of singular types.
    ///
    /// It performs an equality check between both provided types, ignoring the subtypes. This means,
    /// a Text PackedFile with subtype XML and one with subtype LUA will return true, because both are Text PackedFiles.
    pub fn eq_non_strict_slice(self, others: &[Self]) -> bool {
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
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::RigidModel |
            Self::StarPos |
            Self::PackFileSettings |
            Self::UIC |
            Self::Unknown => others.contains(&self),
            Self::Text(_) => others.iter().any(|x| matches!(x, Self::Text(_))),
        }
    }
}

/// From implementation to get the type from a DecodedPackedFile.
impl From<&DecodedPackedFile> for PackedFileType {
    fn from(packed_file: &DecodedPackedFile) -> Self {
        match packed_file {
            DecodedPackedFile::Anim => PackedFileType::Anim,
            DecodedPackedFile::AnimFragment(_) => PackedFileType::AnimFragment,
            DecodedPackedFile::AnimPack(_) => PackedFileType::AnimPack,
            DecodedPackedFile::AnimTable(_) => PackedFileType::AnimTable,
            DecodedPackedFile::CaVp8(_) => PackedFileType::CaVp8,
            DecodedPackedFile::CEO => PackedFileType::CEO,
            DecodedPackedFile::DB(_) => PackedFileType::DB,
            DecodedPackedFile::Image(_) => PackedFileType::Image,
            DecodedPackedFile::GroupFormations => PackedFileType::GroupFormations,
            DecodedPackedFile::Loc(_) => PackedFileType::Loc,
            DecodedPackedFile::MatchedCombat(_) => PackedFileType::MatchedCombat,
            DecodedPackedFile::RigidModel(_) => PackedFileType::RigidModel,
            DecodedPackedFile::StarPos => PackedFileType::StarPos,
            DecodedPackedFile::Text(text) => PackedFileType::Text(text.get_text_type()),
            DecodedPackedFile::UIC(_) => PackedFileType::UIC,
            DecodedPackedFile::Unknown => PackedFileType::Unknown,
        }
    }
}
