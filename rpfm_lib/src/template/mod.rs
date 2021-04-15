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

Templates are a way of bootstraping mods in a few clicks. The way this works is:
- Each template has some general data (name, author,...) about the template itself, some parametrizable data, and some hardcoded data.
- When a template is loaded, the user fills the "Options" (sections of the Template to be applied) and "Parameters" (data that gets personalized to the user's need).
- The template then prepares the parametrized data, and applies itself over the open PackFile.
!*/

use git2::{Reference, ReferenceFormat, Repository, Signature, StashFlags, build::CheckoutBuilder};

use serde_json::de::from_reader;
use serde_derive::{Serialize, Deserialize};

use std::fs::{DirBuilder, File};
use std::io::{BufReader, Write};
use std::process::Command as SystemCommand;

use rpfm_macros::GetRef;

use rpfm_error::{ErrorKind, Result};

use crate::common::*;
use crate::dependencies::Dependencies;
use crate::packfile::{PathType, PackFile, packedfile::PackedFile};
use crate::packedfile::PackedFileType;
use crate::packedfile::text::TextType;
use crate::SCHEMA;
use crate::schema::{APIResponseSchema, Definition, Field};
use self::{asset::Asset, template_db::TemplateDB, template_loc::TemplateLoc};

pub const TEMPLATE_FOLDER: &str = "templates";
pub const DEFINITIONS_FOLDER: &str = "definitions";
pub const ASSETS_FOLDER: &str = "assets";
pub const CUSTOM_TEMPLATE_FOLDER: &str = "templates_custom";

pub const TEMPLATE_REPO: &str = "https://github.com/Frodo45127/rpfm-templates";
pub const REMOTE: &str = "origin";
pub const BRANCH: &str = "master";

mod asset;
mod template_db;
mod template_loc;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This struct represents a Template File in memory.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Template {

    /// It stores the structural version of the Table.
    pub version: u16,

    /// Author of the PackFile.
    pub author: String,

    /// Name of the template (his filename.json).
    pub name: String,

    /// Description of what this template does.
    pub description: String,

    /// This message is shown in the UI after the template has been applied.
    pub post_message: String,

    /// Sections this template has.
    pub sections: Vec<TemplateSection>,

    /// List of options this PackFile can have.
    pub options: Vec<TemplateOption>,

    /// List of params this template requires the user to fill.
    pub params: Vec<TemplateParam>,

    /// The list of DB tables that should be created using this template.
    pub dbs: Vec<TemplateDB>,

    /// The list of Loc tables that should be created using this template.
    pub locs: Vec<TemplateLoc>,

    /// The list of binary assets that should be added to the PackFile using this template.
    pub assets: Vec<Asset>,
}

/// This struct is a common field for table templates. It's here so it can be shared between table types.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
struct TemplateField {

    /// Options required for the field to be used in the template.
    required_options: Vec<String>,

    /// Name of the field in the schema (A.K.A column name).
    field_name: String,

    /// Value the field will have.
    field_value: String,
}

/// This struct contains the data for a section that will group items in the view.
#[derive(GetRef, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TemplateSection {

    /// Options required for this section to be enabled.
    required_options: Vec<String>,

    /// Internal key of the section.
    key: String,

    /// Visual name of the section.
    name: String,

    /// Description of what this section is for.
    description: String,
}

/// This struct contains the data of an option to be chosen in a template.
#[derive(GetRef, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TemplateOption {

    /// Options required for this option to be enabled.
    required_options: Vec<String>,

    /// Internal key of the option.
    key: String,

    /// Visual name of the option.
    name: String,

    /// Key of the section where the UI will put the param (for grouping options).
    section: String,
}

/// This struct contains the data of a param to be filled in a Template.
#[derive(GetRef, Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TemplateParam {

    /// Options required for the field to be used in the param.
    required_options: Vec<String>,

    /// Internal key of the param.
    key: String,

    /// Visual name of the param.
    name: String,

    /// Key of the section where the UI will put the param (for grouping params).
    section: String,

    /// Type of the param, so the UI uses one type of input or another.
    param_type: ParamType,

    /// If this field is required to be able to finish the template.
    is_required: bool
}

/// Types of params the templates can use.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ParamType {

    /// Generic checkbox type, for bool fields.
    Checkbox,

    /// Generic integer type.
    Integer,

    /// Generic decimal type.
    Float,

    /// Basic text type. This is used for strings that need no validation.
    Text,

    /// Field type. This is used for params that directly translate into a field in a table, so it can use their validations. It contains it's table name, and the field definition.
    TableField((String, Field)),

    /// Full table type: This is used for params that admit multiple entries, like tables where you add multiple effects to a spell.
    Table(Definition),
}

impl Default for ParamType {
    fn default() -> Self {
        ParamType::Text
    }
}
//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Template`.
impl Template {

    /// This function applyes a `Template` into the currently open PackFile, if there is one open.
    pub fn apply_template(&mut self, options: &[(String, bool)], params: &[(String, String)], pack_file: &mut PackFile, dependencies: &Dependencies, is_custom: bool) -> Result<Vec<Vec<String>>> {

        // "Parse" the options list into keys, so we know what options are enabled.
        let options = self.options.iter().filter_map(|x| {
            options.iter().find_map(|(y, z)| if x.get_ref_key() == y && *z { Some(x.key.to_owned()) } else { None })
        }).collect::<Vec<String>>();

        // If there is no PackFile open, stop.
        if pack_file.get_file_name().is_empty() {
            return Err(ErrorKind::PackFileIsNotAFile.into());
        }

        // First, deal with all the params.
        for param in &self.params {
            let value = params.iter().find_map(|(y, z)| if param.get_ref_key() == y { Some(z.to_owned()) } else { None }).unwrap();
            for db in &mut self.dbs {
                db.replace_params(&param.key, &value);
            }

            for loc in &mut self.locs {
                loc.replace_params(&param.key, &value);
            }

            for asset in &mut self.assets {
                asset.replace_params(&param.key, &value);
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
                    if db.has_required_options(&options) {
                        let packed_file = db.apply_to_packfile(&options, pack_file, schema, dependencies)?;

                        paths.push(packed_file.get_path().to_vec());
                        packed_files.push(packed_file);
                    }
                }

                // Next, the loc tables.
                for loc in &self.locs {
                    if loc.has_required_options(&options) {
                        let packed_file = loc.apply_to_packfile(&options, pack_file, schema)?;

                        paths.push(packed_file.get_path().to_vec());
                        packed_files.push(packed_file);
                    }
                }

                // And finally, the custom assets.
                let mut folder_name = self.name.to_owned();
                folder_name.pop();
                folder_name.pop();
                folder_name.pop();
                folder_name.pop();
                folder_name.pop();
                let assets_folder = if is_custom { get_custom_template_assets_path()?.join(&folder_name) }
                else { get_template_assets_path()?.join(&folder_name) };

                for asset in &self.assets {
                    if asset.has_required_options(&options) {
                        let path = assets_folder.join(&asset.file_path);
                        let packed_file_path = asset.packed_file_path.split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
                        let packed_file = PackedFile::new_from_file(&path, &packed_file_path)?;

                        paths.push(packed_file_path);
                        packed_files.push(packed_file);
                    }
                }

                // Then, if nothing broke, add the new PackedFiles to the PackFile.
                pack_file.add_packed_files(&packed_files.iter().collect::<Vec<&PackedFile>>(), true, true)?;
                Ok(paths)
            }
            None => Err(ErrorKind::SchemaNotFound.into()),
        }
    }

    /// Function to generate a Template from the currently open PackedFile.
    pub fn save_from_packfile(
        &mut self,
        pack_file: &mut PackFile,
    ) -> Result<()> {

        // If we have no PackedFiles, return an error.
        if pack_file.get_packedfiles_list().is_empty() {
            return Err(ErrorKind::Generic.into());
        }

        // DB Importing.
        let tables = pack_file.get_packed_files_by_type(PackedFileType::DB, false);
        self.dbs = tables.iter().map(|table| TemplateDB::new_from_packedfile(&table).unwrap()).collect::<Vec<TemplateDB>>();

        // Loc Importing.
        let tables = pack_file.get_packed_files_by_type(PackedFileType::Loc, false);
        self.locs = tables.iter().map(|table| TemplateLoc::new_from_packedfile(&table).unwrap()).collect::<Vec<TemplateLoc>>();

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

        let assets_path = get_custom_template_assets_path()?.join(&self.name);
        DirBuilder::new().recursive(true).create(&assets_path)?;

        let assets_packed_files = pack_file.get_ref_packed_files_by_types(&raw_types, false);
        let assets_path_types = assets_packed_files.iter().map(|x| PathType::File(x.get_path().to_vec())).collect::<Vec<PathType>>();
        self.assets = assets_packed_files.iter().map(|x| Asset::new_from_packedfile(x)).collect::<Vec<Asset>>();
        if !self.assets.is_empty() {
            pack_file.extract_packed_files_by_type(&assets_path_types, &assets_path)?;
        }

        self.save()
    }

    /// This function returns the list of sections available for the provided Template.
    pub fn get_sections(&self) -> &[TemplateSection] {
        &self.sections
    }

    /// This function returns the list of options available for the provided Template.
    pub fn get_options(&self) -> &[TemplateOption] {
        &self.options
    }

    /// This function returns the list of params available for the provided Template.
    pub fn get_params(&self) -> &[TemplateParam] {
        &self.params
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
    pub fn save(&mut self) -> Result<()> {
        let mut file_path = get_custom_template_definitions_path()?;

        // Make sure the path exists to avoid problems with updating templates.
        DirBuilder::new().recursive(true).create(&file_path)?;

        file_path.push(format!("{}.json", self.name));
        let mut file = File::create(&file_path)?;
        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// This function downloads the latest revision of the template repository.
    pub fn update() -> Result<()> {
        let template_path = get_template_base_path()?;
        let mut repo = match Repository::open(&template_path) {
            Ok(repo) => repo,
            Err(_) => {

                // If it fails to open, it means either we don't have the .git folder, or we don't have a folder at all.
                // In either case, recreate it and redownload the schemas repo. No more steps are needed here.
                // On windows, remove the read-only flags before doing anything else, or this will fail.
                if cfg!(target_os = "windows") {
                    let path = template_path.to_string_lossy().to_string() + "\\*.*";
                    let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
                }
                let _ = std::fs::remove_dir_all(&template_path);
                DirBuilder::new().recursive(true).create(&template_path)?;
                match Repository::clone(TEMPLATE_REPO, &template_path) {
                    Ok(_) => return Ok(()),
                    Err(_) => return Err(ErrorKind::DownloadTemplatesError.into()),
                }
            }
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", BRANCH);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before update from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // If it worked, now we have to do a pull from master. Sadly, git2-rs does not support pull.
        // Instead, we kinda force a fast-forward. Made in StackOverflow.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let (analysis, fetch_commit_id) = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            (repo.merge_analysis(&[&fetch_commit])?, fetch_commit.id())
        };

        // If we're up to date, nothing more is needed.
        if analysis.0.is_up_to_date() {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }
            Err(ErrorKind::AlreadyUpdatedTemplatesError.into())
        }

        // If we can do a fast-forward, we do it. This is the prefered option.
        else if analysis.0.is_fast_forward() {
            let mut reference = repo.find_reference(&master_refname)?;
            reference.set_target(fetch_commit_id, "Fast-Forward")?;
            repo.set_head(&master_refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force())).map_err(From::from)
        }

        // If not, we face multiple problems:
        // - If there are uncommited changes: covered by the stash.
        // - If we're not in the branch: covered by the branch switch.
        // - If the branches diverged: this one... the cleanest way to deal with it should be redownload the repo.
        else if analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {

            // On windows, remove the read-only flags before doing anything else, or this will fail.
            if cfg!(target_os = "windows") {
                let path = template_path.to_string_lossy().to_string() + "\\*.*";
                let _ = SystemCommand::new("attrib").arg("-r").arg(path).arg("/s").output();
            }

            let _ = std::fs::remove_dir_all(&template_path);
            Self::update()
        }
        else {

            // Reset the repo to his original state after the check
            if current_branch_name != master_refname {
                let _ = repo.set_head(&current_branch_name);
            }
            if stash_id.is_ok() {
                let _ = repo.stash_pop(0, None);
            }

            Err(ErrorKind::DownloadTemplatesError.into())
        }
    }

    /// This function checks if there is a new template update in the template repo.
    pub fn check_update() -> Result<APIResponseSchema> {
        let template_path = get_template_base_path()?;
        let mut repo = match Repository::open(&template_path) {
            Ok(repo) => repo,

            // If this fails, it means we either we don´t have the templates downloaded, or we have the old ones downloaded.
            Err(_) => return Ok(APIResponseSchema::NoLocalFiles),
        };

        // Just in case there are loose changes, stash them.
        // Ignore a fail on this, as it's possible we don't have contents to stash.
        let current_branch_name = Reference::normalize_name(repo.head()?.name().unwrap(), ReferenceFormat::ALLOW_ONELEVEL)?.to_lowercase();
        let master_refname = format!("refs/heads/{}", BRANCH);

        let signature = Signature::now("RPFM Updater", "-")?;
        let stash_id = repo.stash_save(&signature, &format!("Stashed changes before checking for updates from branch {}", current_branch_name), Some(StashFlags::INCLUDE_UNTRACKED));

        // In case we're not in master, checkout the master branch.
        if current_branch_name != master_refname {
            repo.set_head(&master_refname)?;
        }

        // Fetch the info of the master branch.
        repo.find_remote(REMOTE)?.fetch(&[BRANCH], None, None)?;
        let analysis = {
            let fetch_head = repo.find_reference("FETCH_HEAD")?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
            repo.merge_analysis(&[&fetch_commit])?
        };

        // Reset the repo to his original state after the check
        if current_branch_name != master_refname {
            let _ = repo.set_head(&current_branch_name);
        }
        if stash_id.is_ok() {
            let _ = repo.stash_pop(0, None);
        }

        if analysis.0.is_up_to_date() {
            Ok(APIResponseSchema::NoUpdate)
        }

        // If the branch is a fast-forward, or has diverged, ask for an update.
        else if analysis.0.is_fast_forward() || analysis.0.is_normal() || analysis.0.is_none() || analysis.0.is_unborn() {
            Ok(APIResponseSchema::NewUpdate)
        }

        // Otherwise, it means the branches diverged. This may be due to local changes or due to me diverging the master branch with a force push.
        else {
            Err(ErrorKind::TemplateUpdateError.into())
        }
    }

    /// This function returns the name of the template.
    pub fn get_ref_name(&self) -> &str {
        &self.name
    }
}

/// Implementation of TemplateField.
impl TemplateField {

    /// This function builds a new TemplateField from the data provided.
    pub fn new(required_options: &[String], field_name: &str, field_value: &str) -> Self {
        Self {
            required_options: required_options.to_vec(),
            field_name: field_name.to_owned(),
            field_value: field_value.to_owned(),
        }
    }

    /// This function returns the column name for this field.
    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// This function returns the value for this field.
    pub fn get_field_value(&self) -> &str {
        &self.field_value
    }

    /// This function returns the value for this field.
    pub fn get_ref_mut_field_value(&mut self) -> &mut String {
        &mut self.field_value
    }

    /// This function is used to check if we have all the options required to use this field in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }
}

impl TemplateSection {

    pub fn new_from_key_name_required_options_description(key: &str, name: &str, required_options: &[String], description: &str) -> Self {
        Self {
            required_options: required_options.to_vec(),
            key: key.to_owned(),
            name: name.to_owned(),
            description: description.to_owned(),
        }
    }

    /// This function is used to check if we have all the options required to use this section in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }
}

impl TemplateOption {

    pub fn new_from_key_name_section(key: &str, name: &str, section: &str) -> Self {
        Self {
            required_options: vec![],
            key: key.to_owned(),
            name: name.to_owned(),
            section: section.to_owned(),
        }
    }

    /// This function is used to check if we have all the options required to use this field in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }
}

impl TemplateParam {

    pub fn new_from_key_name_section_param_type_check_state(key: &str, name: &str, section: &str, param_type: &str, is_required: bool) -> Self {
        Self {
            required_options: vec![],
            key: key.to_owned(),
            name: name.to_owned(),
            section: section.to_owned(),
            param_type: serde_json::from_str(param_type).unwrap_or(ParamType::Text),
            is_required
        }
    }

    /// This function is used to check if we have all the options required to use this field in the template.
    pub fn has_required_options(&self, options: &[String]) -> bool {
        self.required_options.is_empty() || self.required_options.iter().all(|x| options.contains(x))
    }
}
