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
Module with all the code to deal with mod templates.

Templates are a way of bootstraping mods.
!*/

use git2::Repository;

use serde_json::de::from_reader;
use serde_derive::{Serialize, Deserialize};

use std::fs::{DirBuilder, File};
use std::io::{BufReader, Write};

use rpfm_error::{ErrorKind, Result};

use crate::common::*;
use crate::dependencies::Dependencies;
use crate::packfile::{PathType, PackFile, packedfile::PackedFile};
use crate::packedfile::{DecodedPackedFile, PackedFileType};
use crate::packedfile::table::db::DB;
use crate::packedfile::table::loc::Loc;
use crate::packedfile::table::Table;
use crate::packedfile::table::DecodedData;
use crate::packedfile::text::TextType;
use crate::SCHEMA;
use crate::schema::{APIResponseSchema, FieldType};

pub const TEMPLATE_FOLDER: &str = "templates";
pub const DEFINITIONS_FOLDER: &str = "definitions";
pub const ASSETS_FOLDER: &str = "assets";
pub const CUSTOM_TEMPLATE_FOLDER: &str = "templates_custom";

pub const TEMPLATE_REPO: &str = "https://github.com/Frodo45127/rpfm-templates";
pub const REMOTE: &str = "origin";
pub const BRANCH: &str = "master";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Template File in memory.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Template {

    /// It stores the structural version of the Table.
    version: u16,
    pub author: String,
    name: String,
    pub description: String,

    /// List of params this template requires the user to fill.
    ///
    /// This means: (Display Name, Key)
    pub params: Vec<(String, String)>,

    /// The list of tables that should be created using this template.
    dbs: Vec<TemplateDB>,
    locs: Vec<TemplateLoc>,
    assets: Vec<Asset>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct TemplateDB {
    pub name: String,
    pub table: String,

    /// This means: Rows(Values in Row(Field Name, Value)).
    pub default_data: Vec<Vec<(String, String)>>,
}


#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct TemplateLoc {
    pub name: String,

    /// This means: Rows(Values in Row(Field Name, Value)).
    pub default_data: Vec<Vec<(String, String)>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct Asset {
    pub file_path: String,
    pub packed_file_path: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Template`.
impl Template {

    /// This function applyes a `Template` into the currently open PackFile, if there is one open.
    pub fn apply_template(&mut self, params: &[String], pack_file: &mut PackFile, dependencies: &Dependencies, is_custom: bool) -> Result<Vec<Vec<String>>> {

        // If there is no PackFile open, stop.
        if pack_file.get_file_name().is_empty() {
            return Err(ErrorKind::PackFileIsNotAFile.into());
        }

        // First, deal with all the params.
        for (key, value) in self.params.iter().zip(params.iter()) {
            for mut db in &mut self.dbs {
                db.name = db.name.replace(&format!("{{@{}}}", key.1), value);
                db.default_data = db.default_data.iter()
                    .map(|x| x.iter()
                        .map(|y| (y.0.to_owned(), y.1.replace(&format!("{{@{}}}", key.1), value)))
                        .collect()
                    )
                    .collect();
            }

            for mut loc in &mut self.locs {
                loc.name = loc.name.replace(&format!("{{@{}}}", key.1), value);
                loc.default_data = loc.default_data.iter()
                    .map(|x| x.iter()
                        .map(|y| (y.0.to_owned(), y.1.replace(&format!("{{@{}}}", key.1), value)))
                        .collect()
                    )
                    .collect();
            }

            for mut asset in &mut self.assets {
                asset.packed_file_path = asset.packed_file_path.replace(&format!("{{@{}}}", key.1), value);
            }
        }

        // If ANY of the paths has an empty item, stop.
        if self.dbs.iter().any(|x| x.name.is_empty()) ||
            self.locs.iter().any(|x| x.name.is_empty()) ||
            self.assets.iter().any(|x| x.packed_file_path.contains("//") || x.packed_file_path.ends_with('/')) {
            return Err(ErrorKind::InvalidPathsInTemplate.into());
        }


        // Then, just process each section. In case of collision, we try to append the new data at the end of the file.
        match &*SCHEMA.read().unwrap() {
            Some(schema) => {
                let mut paths = vec![];
                let mut packed_files = vec![];

                // First, the db tables.
                for db in &self.dbs {
                    let path = vec!["db".to_owned(), db.table.to_owned(), db.name.to_owned()];

                    let mut table = if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
                        if let Ok(table) = packed_file.decode_return_ref_no_locks(&schema) {
                            if let DecodedPackedFile::DB(table) = table {
                                table.clone()
                            } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table, &dependencies)?) }
                        } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table, &dependencies)?) }
                    } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table, &dependencies)?) };

                    let mut data = table.get_table_data();
                    for row in &db.default_data {
                        let mut new_row = Table::get_new_row(table.get_ref_definition());
                        for (index, field) in table.get_ref_definition().get_fields_processed().iter().enumerate() {
                            if let Some((_, new_data)) = row.iter().find(|x| x.0 == field.get_name()) {
                                new_row[index] = match field.get_ref_field_type() {
                                    FieldType::Boolean => {
                                        let value = new_data.to_lowercase();
                                        if value == "true" || value == "1" { DecodedData::Boolean(true) }
                                        else { DecodedData::Boolean(false) }
                                    }
                                    FieldType::F32 => DecodedData::F32(new_data.parse::<f32>()?),
                                    FieldType::I16 => DecodedData::I16(new_data.parse::<i16>()?),
                                    FieldType::I32 => DecodedData::I32(new_data.parse::<i32>()?),
                                    FieldType::I64 => DecodedData::I64(new_data.parse::<i64>()?),
                                    FieldType::StringU8 => DecodedData::StringU8(new_data.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(new_data.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(new_data.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(new_data.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    FieldType::SequenceU16(_) => unimplemented!(),
                                    FieldType::SequenceU32(_) => unimplemented!()
                                }
                            }
                        }

                        data.push(new_row);
                    }
                    table.set_table_data(&data)?;

                    let packed_file = PackedFile::new_from_decoded(&DecodedPackedFile::DB(table), &path);

                    paths.push(path);
                    packed_files.push(packed_file);
                }

                // Next, the loc tables.
                for loc in &self.locs {
                    let path = vec!["text".to_owned(), "db".to_owned(), loc.name.to_owned()];

                    let mut table = if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
                        if let Ok(table) = packed_file.decode_return_ref_no_locks(&schema) {
                            if let DecodedPackedFile::Loc(table) = table {
                                table.clone()
                            } else { Loc::new(schema.get_ref_last_definition_loc()?) }
                        } else { Loc::new(schema.get_ref_last_definition_loc()?) }
                    } else { Loc::new(schema.get_ref_last_definition_loc()?) };

                    let mut data = table.get_table_data();
                    for row in &loc.default_data {
                        let mut new_row = Table::get_new_row(table.get_ref_definition());
                        for (index, field) in table.get_ref_definition().get_fields_processed().iter().enumerate() {
                            if let Some((_, new_data)) = row.iter().find(|x| x.0 == field.get_name()) {
                                new_row[index] = match field.get_ref_field_type() {
                                    FieldType::Boolean => {
                                        let value = new_data.to_lowercase();
                                        if value == "true" || value == "1" { DecodedData::Boolean(true) }
                                        else { DecodedData::Boolean(false) }
                                    }
                                    FieldType::F32 => DecodedData::F32(new_data.parse::<f32>()?),
                                    FieldType::I16 => DecodedData::I16(new_data.parse::<i16>()?),
                                    FieldType::I32 => DecodedData::I32(new_data.parse::<i32>()?),
                                    FieldType::I64 => DecodedData::I64(new_data.parse::<i64>()?),
                                    FieldType::StringU8 => DecodedData::StringU8(new_data.to_owned()),
                                    FieldType::StringU16 => DecodedData::StringU16(new_data.to_owned()),
                                    FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(new_data.to_owned()),
                                    FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(new_data.to_owned()),

                                    // For now fail on Sequences. These are a bit special and I don't know if the're even possible in TSV.
                                    FieldType::SequenceU16(_) => unimplemented!(),
                                    FieldType::SequenceU32(_) => unimplemented!()
                                }
                            }
                        }

                        data.push(new_row);
                    }
                    table.set_table_data(&data)?;

                    let packed_file = PackedFile::new_from_decoded(&DecodedPackedFile::Loc(table), &path);

                    paths.push(path);
                    packed_files.push(packed_file);
                }

                // And finally, the custom assets.
                let assets_folder = if is_custom { get_template_assets_path()?.join(&self.name) }
                else { get_template_assets_path()?.join(&self.name) };

                for asset in &self.assets {
                    let path = assets_folder.join(&asset.file_path);
                    let packed_file_path = asset.packed_file_path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
                    let packed_file = PackedFile::new_from_file(&path, &packed_file_path)?;

                    paths.push(packed_file_path);
                    packed_files.push(packed_file);
                }

                // Then, if nothing broke, add the new PackedFiles to the PackFile.
                pack_file.add_packed_files(&packed_files.iter().collect::<Vec<&PackedFile>>(), true)?;
                Ok(paths)
            }
            None => Err(ErrorKind::SchemaNotFound.into()),
        }
    }

    /// Function to generate a Template from the currently open PackedFile.
    pub fn save_from_packfile(pack_file: &mut PackFile, template_name: &str, template_author: &str, template_description: &str, params: &[(String, String)]) -> Result<()> {

        // If we have no PackedFiles, return an error.
        if pack_file.get_packedfiles_list().is_empty() {
            return Err(ErrorKind::Generic.into());
        }

        // DB Importing.
        let tables = pack_file.get_packed_files_by_type(PackedFileType::DB, false);
        let dbs = tables.iter().map(|table| TemplateDB::new_from_packedfile(&table).unwrap()).collect::<Vec<TemplateDB>>();

        // Loc Importing.
        let tables = pack_file.get_packed_files_by_type(PackedFileType::Loc, false);
        let locs = tables.iter().map(|table| TemplateLoc::new_from_packedfile(&table).unwrap()).collect::<Vec<TemplateLoc>>();

        // Raw Assets Importing.
        let raw_types = vec![
            PackedFileType::Anim,
            PackedFileType::AnimFragment,
            PackedFileType::AnimPack,
            PackedFileType::AnimTable,
            PackedFileType::CaVp8,
            PackedFileType::CEO,
            PackedFileType::DependencyPackFilesList,
            PackedFileType::Image,
            PackedFileType::GroupFormations,
            PackedFileType::MatchedCombat,
            PackedFileType::RigidModel,
            PackedFileType::StarPos,
            PackedFileType::PackFileSettings,
            PackedFileType::Unknown,
            PackedFileType::Text(TextType::Plain)
        ];

        let assets_path = get_custom_template_assets_path()?.join(template_name);
        DirBuilder::new().recursive(true).create(&assets_path)?;

        let assets = pack_file.get_ref_packed_files_by_types(&raw_types, false);
        let assets_path_types = assets.iter().map(|x| PathType::File(x.get_path().to_vec())).collect::<Vec<PathType>>();

        pack_file.extract_packed_files_by_type(&assets_path_types, &assets_path)?;

        let mut template = Self {
            version: 0,
            author: template_author.to_owned(),
            name: template_name.to_owned(),
            description: template_description.to_owned(),

            params: params.to_vec(),

            dbs,
            locs,
            assets: vec![],
        };

        template.save(template_name)
    }

    /// This function loads a `Template` to memory.
    pub fn load(template: &str, is_custom: bool) -> Result<Self> {
        let mut file_path_official = get_template_definitions_path()?;
        let mut file_path_custom = get_custom_template_definitions_path()?;
        file_path_official.push(template);
        file_path_custom.push(template);

        let file = if is_custom { BufReader::new(File::open(&file_path_custom)?) }
        else { BufReader::new(File::open(&file_path_official)?) };

        let mut template_loaded: Self = from_reader(file)?;
        template_loaded.name = template.to_owned();
        Ok(template_loaded)
    }

    /// This function saves a `Template` from memory to a file in the `template/` folder.
    pub fn save(&mut self, template: &str) -> Result<()> {
        let mut file_path = get_custom_template_definitions_path()?;

        // Make sure the path exists to avoid problems with updating templates.
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(template);
        let mut file = File::create(&file_path)?;
        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// This function downloads the latest revision of the template repository.
    pub fn update() -> Result<()> {
        let template_path = get_template_base_path()?;
        let repo = match Repository::open(&template_path) {
            Ok(repo) => repo,
            Err(_) => {
                DirBuilder::new().recursive(true).create(&template_path)?;
                match Repository::clone(TEMPLATE_REPO, &template_path) {
                    Ok(repo) => repo,
                    Err(_) => return Err(ErrorKind::DownloadTemplatesError.into()),
                }
            }
        };

        // git2-rs does not support pull. Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            Err(ErrorKind::AlreadyUpdatedTemplatesError.into())
        }

        else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", BRANCH);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).map_err(From::from)
        }

        else {
            Err(ErrorKind::DownloadTemplatesError.into())
        }
    }

    /// This function checks if there is a new template update in the template repo.
    pub fn check_update() -> Result<APIResponseSchema> {
        let template_path = get_template_base_path()?;
        let repo = match Repository::open(&template_path) {
            Ok(repo) => repo,

            // If this fails, it means we either we don´t have the templates downloaded, or we have the old ones downloaded.
            Err(_) => return Ok(APIResponseSchema::NoLocalFiles),
        };

        // git2-rs does not support pull. Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_up_to_date() {
            Ok(APIResponseSchema::NoUpdate)
        }

        else if analysis.0.is_fast_forward() {
            Ok(APIResponseSchema::NewUpdate)
        }

        else {
            Err(ErrorKind::TemplateUpdateError.into())
        }
    }
}

impl TemplateDB {
    pub fn new_from_packedfile(packed_file: &PackedFile) -> Result<Self> {
        let mut template = Self::default();
        template.name = packed_file.get_path().last().unwrap().to_owned();
        template.table = match packed_file.get_path().get(1) {
            Some(table) => table.to_owned(),
            None => return Err(ErrorKind::Generic.into()),
        };

        match packed_file.get_decoded_from_memory()? {
            DecodedPackedFile::DB(table) => {
                let definition = table.get_ref_definition();
                for row in table.get_ref_table_data() {
                    let mut row_data = vec![];
                    for (column, field) in row.iter().enumerate() {
                        row_data.push((definition.get_fields_processed().get(column).unwrap().get_name().to_owned(), field.data_to_string()))
                    }
                    template.default_data.push(row_data);
                }
            },

            _ => return Err(ErrorKind::Generic.into()),
        }

        Ok(template)
    }
}

impl TemplateLoc {
    pub fn new_from_packedfile(packed_file: &PackedFile) -> Result<Self> {
        let mut template = Self::default();
        template.name = packed_file.get_path().last().unwrap().to_owned();

        match packed_file.get_decoded_from_memory()? {
            DecodedPackedFile::Loc(table) => {
                let definition = table.get_ref_definition();
                for row in table.get_ref_table_data() {
                    let mut row_data = vec![];
                    for (column, field) in row.iter().enumerate() {
                        row_data.push((definition.get_fields_processed().get(column).unwrap().get_name().to_owned(), field.data_to_string()))
                    }
                    template.default_data.push(row_data);
                }
            },

            _ => return Err(ErrorKind::Generic.into()),
        }

        Ok(template)
    }
}
