//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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


use anyhow::Result;
use rayon::prelude::*;

use std::{collections::HashMap, fmt::Debug};

use rpfm_common::{games::pfh_version::PFHVersion, schema::Schema};
use rpfm_macros::*;

/*
use std::convert::TryFrom;
use std::{fmt, fmt::Display};
use std::ops::Deref;
*/
//use crate::{dependencies::Dependencies, packfile::{RESERVED_NAME_EXTRA_PACKFILE, RESERVED_NAME_NOTES}};
//use crate::packedfile::animpack::AnimPack;
//use crate::packedfile::ca_vp8::CaVp8;
//use crate::packedfile::esf::ESF;
//use crate::packedfile::image::Image;
//use crate::packedfile::table::{anim_fragment::AnimFragment, animtable::AnimTable, db::DB, loc::Loc, matched_combat::MatchedCombat};
use crate::text::TextType;
//use crate::packedfile::rigidmodel::RigidModel;
//use crate::packedfile::uic::UIC;
//use crate::packedfile::unit_variant::UnitVariant;
//use crate::packfile::packedfile::{CachedPackedFile, PackedFile, RawPackedFile};
//use crate::schema::Schema;
//use crate::SCHEMA;
//use crate::SETTINGS;

pub mod animpack;
pub mod ca_vp8;
pub mod db;
pub mod esf;
pub mod image;
pub mod loc;
pub mod pack;
pub mod rigidmodel;
pub mod table;
pub mod text;
pub mod uic;
pub mod unit_variant;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Clone, Debug, PartialEq)]
pub struct RFile<T: Decodeable> {
    path: String,
    timestamp: Option<i64>,
    data: RFileInnerData<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RFileInnerData<T: Decodeable> {
    Decoded(T),
    Catched(Vec<u8>),
    OnDisk(OnDisk)

}

/// This struct contains the stuff needed to read the data of a particular PackedFile from disk.
#[derive(Clone, Debug, PartialEq, GetRef)]
pub struct OnDisk {

    /// Reader over the PackFile containing the PackedFile.
    path: String,
    start: u64,
    size: u32,
    is_compressed: bool,
    is_encrypted: Option<PFHVersion>,

    /// Last Modified Date on disk of the PackFile containing this PackedFile.
    last_modified_date_pack: i64,
}




/*
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
    CEO(ESF),
    DB(DB),
    ESF(ESF),
    Image(Image),
    GroupFormations,
    Loc(Loc),
    MatchedCombat(MatchedCombat),
    RigidModel(RigidModel),
    Text(Text),
    UIC(UIC),
    UnitVariant(UnitVariant),
    Unknown,
}
*/
/// This enum specifies the different types of `PackedFile` we can find in a `PackFile`.
///
/// Keep in mind that, despite we having logic to recognize them, we can't decode many of them yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileType {
    Anim,
    AnimFragment,
    AnimPack,
    AnimTable,
    CaVp8,
    CEO,
    DB,
    ESF,
    Image,
    GroupFormations,
    Loc,
    MatchedCombat,
    RigidModel,
    UIC,
    UnitVariant,

    /// This one is an exception, as it contains the TextType of the Text PackedFile, so we can do things depending on the type.
    Text(TextType),

    /// This one is special. It's used just in case we want to open the Dependency PackFile List as a PackedFile.
    DependencyPackFilesList,

    /// To identify PackFiles in a PackedFile context.
    PackFile,
    PackFileSettings,
    Unknown,
}

pub enum ContainerPath {
    File(String),
    Folder(String),
    FullContainer,
}

//---------------------------------------------------------------------------//
//                           Trait Definitions
//---------------------------------------------------------------------------//

pub trait Decodeable: Send + Sync {
    fn file_type(&self) -> FileType;
    fn decode(data: &[u8], extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> where Self: Sized;
}

pub trait Encodeable {
    fn encode(&self) -> Vec<u8>;
}

pub trait Container<T: Decodeable> {
    /*
    fn insert(&mut self, file: RFile<T>) -> ContainerPath;
    fn remove(&mut self, path: &ContainerPath) -> Vec<ContainerPath>;
    fn files(&self) -> &HashMap<String, RFile<T>>;
    fn files_mut(&mut self) -> &mut HashMap<String, RFile<T>>;
    fn files_by_path(&self, path: &ContainerPath) -> Vec<&RFile<T>>;
    fn paths(&self) -> Vec<ContainerPath>;
    fn paths_raw(&self) -> Vec<&str>;*/

    fn insert(&mut self, file: RFile<T>) -> ContainerPath {
        let path = file.path();
        let path_raw = file.path_raw();
        self.files_mut().insert(path_raw.to_owned(), file);
        path
    }

    fn remove(&mut self, path: &ContainerPath) -> Vec<ContainerPath> {
        match path {
            ContainerPath::File(path) => {
                self.files_mut().remove(path);
                return vec![ContainerPath::File(path.to_owned())];
            },
            ContainerPath::Folder(path) => {
                let paths_to_remove = self.files().par_iter()
                    .filter_map(|(key, _)| if key.starts_with(path) { Some(key.to_owned()) } else { None }).collect::<Vec<String>>();

                paths_to_remove.iter().for_each(|path| {
                    self.files_mut().remove(path);
                });
                return paths_to_remove.par_iter().map(|path| ContainerPath::File(path.to_string())).collect();
            },
            ContainerPath::FullContainer => {
                self.files_mut().clear();
                return vec![ContainerPath::FullContainer];
            },
        }
    }

    fn files(&self) -> &HashMap<std::string::String, RFile<T>>;
    fn files_mut(&mut self) -> &mut HashMap<std::string::String, RFile<T>>;

    fn files_by_path(&self, path: &ContainerPath) -> Vec<&RFile<T>> {
        match path {
            ContainerPath::File(path) => {
                match self.files().get(path) {
                    Some(file) => vec![file],
                    None => vec![],
                }
            },
            ContainerPath::Folder(path) => {
                self.files().par_iter()
                    .filter_map(|(key, file)|
                        if key.starts_with(path) { Some(file) } else { None }
                    ).collect::<Vec<&RFile<T>>>()
            },
            ContainerPath::FullContainer => {
                self.files().values().collect()
            },
        }
    }

    fn paths(&self) -> Vec<ContainerPath> {
        self.files().par_iter().map(|(path, _)| ContainerPath::File(path.to_owned())).collect()
    }

    fn paths_raw(&self) -> Vec<String> {
        self.files()
            .par_iter()
            .map(|(path, _)| path.to_owned())
            .collect()
    }
}


impl<T: Decodeable> RFile<T> {
    pub fn data(&self) -> &[u8] {
        match &self.data {
            RFileInnerData::Decoded(_) => todo!(),
            RFileInnerData::Catched(data) => data,
            RFileInnerData::OnDisk(_) => todo!(),
        }
    }
    pub fn path(&self) -> ContainerPath {
        ContainerPath::File(self.path.to_owned())
    }
    pub fn path_raw(&self) -> &str {
        &self.path
    }
}
/*
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
                        let packed_file = AnimFragment::read(&data, schema, false)?;
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
                        let packed_file = AnimTable::read(&data, schema, false)?;
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

            PackedFileType::ESF => {
                let data = raw_packed_file.get_data()?;
                let packed_file = ESF::read(&data)?;
                Ok(DecodedPackedFile::ESF(packed_file))
            }

            PackedFileType::DB => {
                let schema = SCHEMA.read().unwrap();
                match schema.deref() {
                    Some(schema) => {
                        let data = raw_packed_file.get_data_and_keep_it()?;
                        let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                        let packed_file = DB::read(&data, name, schema, false)?;
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
                        let packed_file = Loc::read(&data, schema, false)?;
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
                        let packed_file = MatchedCombat::read(&data, schema, false)?;
                        Ok(DecodedPackedFile::MatchedCombat(packed_file))
                    }
                    None => Err(ErrorKind::SchemaNotFound.into()),
                }
            }

            #[cfg(feature = "support_rigidmodel")]
            PackedFileType::RigidModel => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = RigidModel::read(&data);
                Ok(DecodedPackedFile::RigidModel(packed_file))
            }

            PackedFileType::Text(text_type) => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let mut packed_file = Text::read(&data)?;
                packed_file.set_text_type(text_type);
                Ok(DecodedPackedFile::Text(packed_file))
            }

            #[cfg(feature = "support_uic")]
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

            PackedFileType::UnitVariant => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UnitVariant::read(&data)?;
                Ok(DecodedPackedFile::UnitVariant(packed_file))
            }

            _=> Ok(DecodedPackedFile::Unknown)
        }
    }

    /// This function decodes a `RawPackedFile` into a `DecodedPackedFile`, returning it.
    pub fn decode_no_locks(raw_packed_file: &mut RawPackedFile, schema: &Schema) -> Result<Self> {
        match PackedFileType::get_packed_file_type(raw_packed_file, true) {

            PackedFileType::AnimFragment => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimFragment::read(&data, schema, false)?;
                Ok(DecodedPackedFile::AnimFragment(packed_file))
            }

            PackedFileType::AnimPack => Self::decode(raw_packed_file),

            PackedFileType::AnimTable => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = AnimTable::read(&data, schema, false)?;
                Ok(DecodedPackedFile::AnimTable(packed_file))
            }

            PackedFileType::CaVp8 => Self::decode(raw_packed_file),
            PackedFileType::ESF => Self::decode(raw_packed_file),

            PackedFileType::DB => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let name = raw_packed_file.get_path().get(1).ok_or_else(|| Error::from(ErrorKind::DBTableIsNotADBTable))?;
                let packed_file = DB::read(&data, name, schema, false)?;
                Ok(DecodedPackedFile::DB(packed_file))
            }

            PackedFileType::Image => Self::decode(raw_packed_file),

            PackedFileType::Loc => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = Loc::read(&data, schema, false)?;
                Ok(DecodedPackedFile::Loc(packed_file))
            }

            PackedFileType::MatchedCombat => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = MatchedCombat::read(&data, schema, false)?;
                Ok(DecodedPackedFile::MatchedCombat(packed_file))
            }

            #[cfg(feature = "support_rigidmodel")]
            PackedFileType::RigidModel => Self::decode(raw_packed_file),

            PackedFileType::Text(_) => Self::decode(raw_packed_file),

            #[cfg(feature = "support_uic")]
            PackedFileType::UIC => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UIC::read(&data, &schema)?;
                Ok(DecodedPackedFile::UIC(packed_file))
            }

            PackedFileType::UnitVariant => {
                let data = raw_packed_file.get_data_and_keep_it()?;
                let packed_file = UnitVariant::read(&data)?;
                Ok(DecodedPackedFile::UnitVariant(packed_file))
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
            DecodedPackedFile::ESF(data) => Some(Ok(data.save())),
            DecodedPackedFile::Loc(data) => Some(data.save()),
            DecodedPackedFile::MatchedCombat(data) => Some(data.save()),

            #[cfg(feature = "support_rigidmodel")]
            DecodedPackedFile::RigidModel(data) => Some(Ok(data.save())),

            DecodedPackedFile::Text(data) => Some(data.save()),

            #[cfg(feature = "support_uic")]
            DecodedPackedFile::UIC(data) => Some(Ok(data.save())),

            DecodedPackedFile::UnitVariant(data) => Some(data.save()),
            _=> None,
        }
    }

    /// This function updates a DB Table to its latest valid version, being the latest valid version the one in the data.pack or equivalent of the game.
    ///
    /// It returns both, old and new versions, or an error.
    pub fn update_table(&mut self, dependencies: &Dependencies) -> Result<(i32, i32)> {
        match self {
            DecodedPackedFile::DB(data) => {
                let dep_db = dependencies.get_db_tables_from_cache(data.get_ref_table_name(), true, false)?;
                if let Some(vanilla_db) = dep_db.iter()
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
            PackedFileType::ESF => write!(f, "ESF"),
            PackedFileType::Image => write!(f, "Image"),
            PackedFileType::GroupFormations => write!(f, "Group Formations"),
            PackedFileType::Loc => write!(f, "Loc Table"),
            PackedFileType::MatchedCombat => write!(f, "Matched Combat"),
            PackedFileType::PackFile => write!(f, "PackFile"),
            PackedFileType::RigidModel => write!(f, "RigidModel"),
            PackedFileType::UIC => write!(f, "UI Component"),
            PackedFileType::UnitVariant => write!(f, "Unit Variant"),
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

        // Reserved PackedFiles.
        if path == [RESERVED_NAME_NOTES] {
            return Self::Text(TextType::Markdown);
        }

        if !path.is_empty() && path.starts_with(&[RESERVED_NAME_EXTRA_PACKFILE.to_owned()]) {
            return Self::PackFile;
        }

        if let Some(packedfile_name) = path.last() {
            let packedfile_name = packedfile_name.to_lowercase();

            if packedfile_name.ends_with(table::loc::EXTENSION) {
                return Self::Loc;
            }

            if packedfile_name.ends_with(rigidmodel::EXTENSION) {
                return Self::RigidModel
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

            if packedfile_name.ends_with(unit_variant::EXTENSION) {
                return Self::UnitVariant
            }

            if esf::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(*x)) && SETTINGS.read().unwrap().settings_bool["enable_esf_editor"] {
                return Self::ESF;
            }

            // If that failed, try types that need to be in a specific path.
            let path_str = path.iter().map(String::as_str).collect::<Vec<&str>>();
            if path_str.starts_with(&table::matched_combat::BASE_PATH) && packedfile_name.ends_with(table::matched_combat::EXTENSION) {
                return Self::MatchedCombat;
            }

            if path_str.starts_with(&table::animtable::BASE_PATH) && packedfile_name.ends_with(table::animtable::EXTENSION) {
                return Self::AnimTable;
            }

            if path_str.starts_with(&table::anim_fragment::BASE_PATH) && table::anim_fragment::EXTENSIONS.iter().any(|x| packedfile_name.ends_with(*x)) {
                return Self::AnimFragment;
            }

            // If that failed, check if it's in a folder which is known to only have specific files.
            if let Some(folder) = path.get(0) {
                let base_folder = folder.to_lowercase();
                if &base_folder == "db" {
                    return Self::DB;
                }

                if &base_folder == "ui" && (!packedfile_name.contains('.') || packedfile_name.ends_with(uic::EXTENSION)) {
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

    /// This function returns the type of the provided `CachedPackedFile` based on the info about them (path, name, extension,...).
    ///
    /// Strict mode also performs a search by checking the data directly if no type was found, but that's very slow. Think twice before using it.
    pub fn get_cached_packed_file_type(packed_file: &CachedPackedFile, strict_mode: bool) -> Self {

        // First, try with extensions.
        let path = packed_file.get_ref_packed_file_path().to_lowercase();
        if path.ends_with(table::loc::EXTENSION) {
            return Self::Loc;
        }

        if path.ends_with(rigidmodel::EXTENSION) {
            return Self::RigidModel
        }

        if path.ends_with(animpack::EXTENSION) {
            return Self::AnimPack
        }

        if path.ends_with(ca_vp8::EXTENSION) {
            return Self::CaVp8;
        }

        if image::EXTENSIONS.iter().any(|x| path.ends_with(x)) {
            return Self::Image;
        }

        if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(x, _)| path.ends_with(x)) {
            return Self::Text(*text_type);
        }

        if path.ends_with(unit_variant::EXTENSION) {
            return Self::UnitVariant
        }

        if esf::EXTENSIONS.iter().any(|x| path.ends_with(*x)) && SETTINGS.read().unwrap().settings_bool["enable_esf_editor"] {
            return Self::ESF;
        }

        // If that failed, try types that need to be in a specific path.
        let path_str = path.split('/').collect::<Vec<&str>>();
        if path.ends_with(table::matched_combat::EXTENSION) && path_str.starts_with(&table::matched_combat::BASE_PATH) {
            return Self::MatchedCombat;
        }

        if path.ends_with(table::animtable::EXTENSION) && path_str.starts_with(&table::animtable::BASE_PATH) {
            return Self::AnimTable;
        }

        if path_str.starts_with(&table::anim_fragment::BASE_PATH) && table::anim_fragment::EXTENSIONS.iter().any(|x| path.ends_with(*x)) {
            return Self::AnimFragment;
        }

        // If that failed, check if it's in a folder which is known to only have specific files.
        if let Some(folder) = path_str.get(0) {
            if *folder == "db" {
                return Self::DB;
            }

            if *folder == "ui" && (!path.contains('.') || path.ends_with(uic::EXTENSION)) {
                return Self::UIC;
            }
        }

        // If nothing worked, turn it into a proper PackedFile and try to get the type that way.
        // NOTE: EXTREMELY SLOW!!!!!!
        if strict_mode {
            Self::get_packed_file_type(PackedFile::try_from(packed_file).unwrap().get_ref_raw(), strict_mode);
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
            Self::ESF |
            Self::Image |
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::PackFile |
            Self::RigidModel |
            Self::PackFileSettings |
            Self::UIC |
            Self::UnitVariant |
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
            Self::ESF |
            Self::Image |
            Self::GroupFormations |
            Self::Loc |
            Self::MatchedCombat |
            Self::PackFile |
            Self::RigidModel |
            Self::PackFileSettings |
            Self::UIC |
            Self::UnitVariant |
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
            DecodedPackedFile::CEO(_) => PackedFileType::CEO,
            DecodedPackedFile::DB(_) => PackedFileType::DB,
            DecodedPackedFile::Image(_) => PackedFileType::Image,
            DecodedPackedFile::GroupFormations => PackedFileType::GroupFormations,
            DecodedPackedFile::Loc(_) => PackedFileType::Loc,
            DecodedPackedFile::MatchedCombat(_) => PackedFileType::MatchedCombat,
            DecodedPackedFile::RigidModel(_) => PackedFileType::RigidModel,
            DecodedPackedFile::ESF(_) => PackedFileType::ESF,
            DecodedPackedFile::Text(text) => PackedFileType::Text(text.get_text_type()),
            DecodedPackedFile::UIC(_) => PackedFileType::UIC,
            DecodedPackedFile::UnitVariant(_) => PackedFileType::UnitVariant,
            DecodedPackedFile::Unknown => PackedFileType::Unknown,
        }
    }
}
*/
