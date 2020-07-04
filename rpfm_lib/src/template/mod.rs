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
use crate::packfile::{PackFile, packedfile::PackedFile};
use crate::packedfile::DecodedPackedFile;
use crate::packedfile::table::db::DB;
use crate::packedfile::table::loc::Loc;
use crate::packedfile::table::Table;
use crate::packedfile::table::DecodedData;
use crate::SCHEMA;
use crate::schema::FieldType;

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
    pub description: String,

    /// List of params this template requires the user to fill.
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
    pub default_data: Vec<Vec<(String, String)>>,
}


#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct TemplateLoc {
    pub name: String,
    pub default_data: Vec<Vec<(String, String)>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct Asset {
    pub file_name: String,
    pub packed_file_path: String,
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Template`.
impl Template {

    /// This function applyes a `Template` into the currently open PackFile, if there is one open.
    pub fn apply_template(&mut self, params: &[String], pack_file: &mut PackFile) -> Result<Vec<Vec<String>>> {

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
                    let path = vec!["db".to_owned(), db.table.to_owned() + "_tables", db.name.to_owned()];

                    let mut table = if let Some(packed_file) = pack_file.get_ref_mut_packed_file_by_path(&path) {
                        if let Ok(table) = packed_file.decode_return_ref_no_locks(&schema) {
                            if let DecodedPackedFile::DB(table) = table {
                                table.clone()
                            } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table)?) }
                        } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table)?) }
                    } else { DB::new(&db.name, None, schema.get_ref_last_definition_db(&db.table)?) };

                    let mut data = table.get_table_data();
                    for row in &db.default_data {
                        let mut new_row = Table::get_new_row(table.get_ref_definition());
                        for (index, field) in table.get_ref_definition().get_ref_fields().iter().enumerate() {
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
                        for (index, field) in table.get_ref_definition().get_ref_fields().iter().enumerate() {
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
                let assets_folder = get_template_assets_path()?;
                for asset in &self.assets {
                    let path = assets_folder.join(&asset.file_name);
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

    /// This function loads a `Template` to memory.
    pub fn load(template: &str) -> Result<Self> {
        let mut file_path = get_custom_template_definitions_path()?;
        file_path.push(template);

        if file_path.exists() {
            let file = BufReader::new(File::open(&file_path)?);
            from_reader(file).map_err(From::from)
        }

        else {
            let mut file_path = get_template_definitions_path()?;
            file_path.push(template);

            let file = BufReader::new(File::open(&file_path)?);
            from_reader(file).map_err(From::from)
        }
    }

    /// This function saves a `Template` from memory to a file in the `template/` folder.
    pub fn save(&mut self, template: &str) -> Result<()> {
        let mut file_path = get_template_definitions_path()?;

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
}
