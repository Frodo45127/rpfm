//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to interact with Schemas.

This module contains all the code related with the schemas used by this lib to decode many PackedFile types.

The basic structure of an `Schema` is:
```ignore
(
    version: 3,
    versioned_files: [
        DB("_kv_battle_ai_ability_usage_variables_tables", [
            (
                version: 0,
                fields: [
                    (
                        name: "key",
                        field_type: StringU8,
                        is_key: true,
                        default_value: None,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
                        is_part_of_colour: None,
                    ),
                    (
                        name: "value",
                        field_type: F32,
                        is_key: false,
                        default_value: None,
                        is_filename: false,
                        filename_relative_path: None,
                        is_reference: None,
                        lookup: None,
                        description: "",
                        ca_order: -1,
                        is_bitwise: 0,
                        enum_values: {},
                        is_part_of_colour: None,
                    ),
                ],
                localised_fields: [],
            ),
        ]),
    ],
)
```

Inside the schema there are `VersionedFile` variants of different types, with a Vec of `Definition`, one for each version of that PackedFile supported.
!*/

use getset::*;
use itertools::Itertools;
use rayon::prelude::*;
use ron::de::{from_bytes, from_str};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Serialize as SerdeSerialize, Serializer};
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::{fmt, fmt::Display};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::localisable_fields::RawLocalisableField;
#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::table_definition::RawDefinition;
#[cfg(feature = "integration_assembly_kit")]use crate::integrations::assembly_kit::table_definition::RawField;
#[cfg(feature = "integration_log")] use crate::integrations::log::*;
#[cfg(feature = "integration_sqlite")] use rusqlite::types::Type;

use crate::error::Result;
use crate::files::table::DecodedData;
use crate::games::supported_games::SupportedGames;

// Legacy Schemas, to keep backwards compatibility during updates.
pub(crate) mod v4;

/// Name of the folder containing all the schemas.
pub const SCHEMA_FOLDER: &str = "schemas";

//const BINARY_EXTENSION: &str = ".bin";

pub const SCHEMA_REPO: &str = "https://github.com/Frodo45127/rpfm-schemas";
pub const SCHEMA_REMOTE: &str = "origin";
pub const SCHEMA_BRANCH: &str = "master";

/// Current structural version of the Schema, for compatibility purposes.
const CURRENT_STRUCTURAL_VERSION: u16 = 5;
const INVALID_VERSION: i32 = -100;

/// Name for unamed colour groups.
pub const MERGE_COLOUR_NO_NAME: &str = "Unnamed Colour Group";

/// Ending for named colour groups.
pub const MERGE_COLOUR_POST: &str = "_hex";

/// Fields that can be ignored in missing field checks.
const IGNORABLE_FIELDS: [&str; 4] = ["s_ColLineage", "s_Generation", "s_GUID", "s_Lineage"];

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This type defines patches for specific table definitions, in a ColumnName -> [key -> value] format.
///
/// Note: for table-wide patches, we use column name "-1".
pub type DefinitionPatch = HashMap<String, HashMap<String, String>>;

/// This struct represents a Schema File in memory, ready to be used to decode versioned PackedFiles.
#[derive(Clone, PartialEq, Eq, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Schema {

    /// It stores the structural version of the Schema.
    version: u16,

    /// It stores the versioned files inside the Schema.
    #[serde(serialize_with = "ordered_map_definitions")]
    definitions: HashMap<String, Vec<Definition>>,

    /// It stores a list of per-table, per-column patches.
    #[serde(serialize_with = "ordered_map_patches")]
    patches: HashMap<String, DefinitionPatch>,
}

/// This struct contains all the data needed to decode a specific version of a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Definition {

    /// The version of the file the definition is for. These versions are:
    /// - `-1`: for fake `Definition`, used for dependency resolving stuff.
    /// - `0`: for unversioned PackedFiles.
    /// - `1+`: for versioned PackedFiles.
    version: i32,

    /// This is a collection of all `Field`s the PackedFile uses, in the order it uses them.
    fields: Vec<Field>,

    /// This is a list of all the fields from this definition that are moved to a Loc PackedFile on exporting.
    localised_fields: Vec<Field>,

    /// This is the order in which key fields are ordered when used as part of a localisation key.
    ///
    /// Note: this order is only applicable to the processed fields, not the raw fields.
    localised_key_order: Vec<u32>,

    /// Patches that apply to this definition.
    ///
    /// Not serialized. Populated when getting the definition.
    #[serde(skip)]
    patches: DefinitionPatch
}

/// This struct holds all the relevant data do properly decode a field from a versioned PackedFile.
#[derive(Clone, PartialEq, Eq, Debug, Setters, Serialize, Deserialize)]
#[getset(set = "pub")]
pub struct Field {

    /// Name of the field. Should contain no spaces, using `_` instead.
    name: String,

    /// Type of the field.
    field_type: FieldType,

    /// `True` if the field is a `Key` field of a table. `False` otherwise.
    is_key: bool,

    /// The default value of the field.
    default_value: Option<String>,

    /// If the field's data corresponds to a filename.
    is_filename: bool,

    /// Path where the file in the data of the field can be, if it's restricted to one path.
    filename_relative_path: Option<String>,

    /// `Some(referenced_table, referenced_column)` if the field is referencing another table/column. `None` otherwise.
    is_reference: Option<(String, String)>,

    /// `Some(referenced_columns)` if the field is using another column/s from the referenced table for lookup values.
    lookup: Option<Vec<String>>,

    /// Aclarative description of what the field is for.
    description: String,

    /// Visual position in CA's Table. `-1` means we don't know its position.
    ca_order: i16,

    /// Variable to tell if this column is a bitwise column (spanned accross multiple columns) or not. Only applicable to numeric fields.
    is_bitwise: i32,

    /// Variable that specifies the "Enum" values for each value in this field.
    enum_values: BTreeMap<i32, String>,

    /// If the field is part of a 3-part RGB column set, and which one (R, G or B) it is.
    is_part_of_colour: Option<u8>,

    /// If the field is not used by the game. This one is set through patches, so we don't serialize it.
    #[serde(skip_serializing, skip_deserializing)]
    unused: bool,
}

/// This enum defines every type of field the lib can encode/decode.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    F32,
    F64,
    I16,
    I32,
    I64,
    ColourRGB,
    StringU8,
    StringU16,
    OptionalI16,
    OptionalI32,
    OptionalI64,
    OptionalStringU8,
    OptionalStringU16,
    SequenceU16(Box<Definition>),
    SequenceU32(Box<Definition>)
}

//---------------------------------------------------------------------------//
//                       Enum & Structs Implementations
//---------------------------------------------------------------------------//

/// Implementation of `Schema`.
impl Schema {

    /// This function will save a new patch to the local patches list.
    pub fn new_patch(patches: &HashMap<String, DefinitionPatch>, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        Self::add_patch_to_patch_set(&mut local_patches, patches);

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// This function will remove the local patches for the specified table.
    pub fn remove_patch_for_table(table_name: &str, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        local_patches.remove(table_name);

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// This function will remove the local patches for the specified table and field.
    pub fn remove_patch_for_field(table_name: &str, field_name: &str, path: &Path) -> Result<()> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut local_patches: HashMap<String, DefinitionPatch> = from_bytes(&data)?;

        if let Some(table_patches) = local_patches.get_mut(table_name) {
            table_patches.remove(field_name);
        }

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();
        file.write_all(to_string_pretty(&local_patches, config)?.as_bytes())?;

        Ok(())
    }

    /// This function retrieves a value from a patch for a specific table, column and key.
    pub fn patch_value(&self, table_name: &str, column_name: &str, key: &str) -> Option<&String> {
        self.patches.get(table_name)?.get(column_name)?.get(key)
    }

    /// This function retrieves all patches that affect a specific table.
    pub fn patches_for_table(&self, table_name: &str) -> Option<&DefinitionPatch> {
        self.patches.get(table_name)
    }

    /// This function adds a list of patches into the currently loaded schema.
    ///
    /// Note: if you add a patch, you'll need to re-retrieve any definition you retrieved before in order for them to get patched.
    pub fn add_patch_to_patch_set(patch_set: &mut HashMap<String, DefinitionPatch>, patches: &HashMap<String, DefinitionPatch>) {
        patches.iter().for_each(|(table_name, column_patch)| {
            match patch_set.get_mut(table_name) {
                Some(column_patch_current) => {
                    column_patch.iter().for_each(|(column_name, patch)| {
                        match column_patch_current.get_mut(column_name) {
                            Some(patch_current) => patch_current.extend(patch.clone()),
                            None => {
                                column_patch_current.insert(column_name.to_owned(), patch.clone());
                            }
                        }
                    });
                }
                None => {
                    patch_set.insert(table_name.to_owned(), column_patch.clone());
                }
            }
        });
    }

    /// This function adds a definition for a table into the currently loaded schema.
    pub fn add_definition(&mut self, table_name: &str, definition: &Definition) {
        match self.definitions.get_mut(table_name) {
            Some(definitions) => {
                match definitions.iter_mut().find(|def| def.version() == definition.version()) {
                    Some(def) => *def = definition.to_owned(),
                    None => definitions.push(definition.to_owned()),
                }
            },
            None => { self.definitions.insert(table_name.to_owned(), vec![definition.to_owned()]); },
        }
    }

    /// This function removes a definition for a table from the currently loaded schema.
    pub fn remove_definition(&mut self, table_name: &str, version: i32) {
        if let Some(definitions) = self.definitions.get_mut(table_name) {
            let mut index_to_delete = vec![];
            for (index, definition) in definitions.iter().enumerate() {
                if definition.version == version {
                    index_to_delete.push(index);
                }
            }

            index_to_delete.iter().rev().for_each(|index| { definitions.remove(*index); });
        }
    }

    /// This function returns a copy of a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn definitions_by_table_name_cloned(&self, table_name: &str) -> Option<Vec<Definition>> {
        self.definitions.get(table_name).cloned()
    }

    /// This function returns a reference to a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn definitions_by_table_name(&self, table_name: &str) -> Option<&Vec<Definition>>  {
        self.definitions.get(table_name)
    }

    /// This function returns a mutable reference to a specific `VersionedFile` of DB Type from the provided `Schema`.
    pub fn definitions_by_table_name_mut(&mut self, table_name: &str) -> Option<&mut Vec<Definition>>  {
        self.definitions.get_mut(table_name)
    }

    /// This function returns the last compatible definition of a DB Table.
    ///
    /// As we may have versions from other games, we first need to check for the last definition in the dependency database.
    /// If that fails, we try to get it from the schema.
    pub fn definition_newer(&self, table_name: &str, candidates: &[Definition]) -> Option<&Definition> {

        // Version is... complicated. We don't really want the last one, but the last one compatible with our game.
        // So we have to try to get it first from the Dependency Database first. If that fails, we fall back to the schema.
        if let Some(definition) = candidates.iter().max_by(|x, y| x.version().cmp(y.version())) {
            self.definition_by_name_and_version(table_name, *definition.version())
        }

        // If there was no coincidence in the dependency database... we risk ourselves getting the last definition we have for
        // that db from the schema.
        else{
            self.definitions.get(table_name)?.first()
        }
    }

    pub fn definition_by_name_and_version(&self, table_name: &str, table_version: i32) -> Option<&Definition>  {
        self.definitions.get(table_name)?.iter().find(|definition| *definition.version() == table_version)
    }

    pub fn definition_by_name_and_version_mut(&mut self, table_name: &str, table_version: i32) -> Option<&mut Definition>  {
        self.definitions.get_mut(table_name)?.iter_mut().find(|definition| *definition.version() == table_version)
    }

    /// This function loads a [Schema] to memory from a provided `.ron` file.
    pub fn load(path: &Path, local_patches: Option<&Path>) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut schema: Self = from_bytes(&data)?;
        let mut patches = schema.patches().clone();

        // If we got local patches, add them to the patches list.
        //
        // NOTE: we separate the patches from the schemas because otherwise an schema edit will save local patches into the schema,
        // and we want them to remain local.
        if let Some(path) = local_patches {
            if let Ok(file) = File::open(path) {
                let mut file = BufReader::new(file);
                let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
                file.read_to_end(&mut data)?;
                if let Ok(local_patches) = from_bytes::<HashMap<String, DefinitionPatch>>(&data) {
                    Self::add_patch_to_patch_set(&mut patches, &local_patches);
                }
            }
        }

        // Preload all patches to their respective definitions.
        for (table_name, patches) in &patches {
            if let Some(definitions) = schema.definitions_by_table_name_mut(table_name) {
                for definition in definitions {
                    definition.set_patches(patches.clone());
                }
            }
        }

        Ok(schema)
    }

    /// This function loads a [Schema] to memory from a provided `.json` file.
    pub fn load_json(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        let mut schema: Self = serde_json::from_slice(&data)?;

        // Preload all patches to their respective definitions.
        for (table_name, patches) in schema.patches().clone() {
            if let Some(definitions) = schema.definitions_by_table_name_mut(&table_name) {
                for definition in definitions {
                    definition.set_patches(patches.clone());
                }
            }
        }

        Ok(schema)
    }

    /// This function saves a [Schema] from memory to a `.ron` file with the provided path.
    pub fn save(&mut self, path: &Path) -> Result<()> {

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(path)?);
        let config = PrettyConfig::default();

        let mut patches = HashMap::new();

        // Make sure all definitions are properly sorted by version number.
        self.definitions.iter_mut().for_each(|(table_name, definitions)| {
            definitions.sort_by(|a, b| b.version().cmp(a.version()));

            // Fix for empty dependencies, again.
            definitions.iter_mut().for_each(|definition| {
                definition.fields.iter_mut().for_each(|field| {
                    if let Some((ref_table, ref_column)) = field.is_reference(None) {
                        if ref_table.trim().is_empty() || ref_column.trim().is_empty() {
                            field.is_reference = None;
                        }
                    }
                });

                // Move any lookup_hardcoded patches to schema patches.
                if definition.patches.values().any(|x| x.keys().any(|y| y == "lookup_hardcoded")) {
                    let mut def_patches = definition.patches().clone();
                    def_patches.retain(|_, value| {
                        value.retain(|key, _| key == "lookup_hardcoded");
                        !value.is_empty()
                    });
                    patches.insert(table_name.to_owned(), def_patches);
                }

                // Move any unused patches to schema patches.
                if definition.patches.values().any(|x| x.keys().any(|y| y == "unused")) {
                    let mut def_patches = definition.patches().clone();
                    def_patches.retain(|_, value| {
                        value.retain(|key, _| key == "unused");
                        !value.is_empty()
                    });
                    patches.insert(table_name.to_owned(), def_patches);
                }
            })
        });

        Self::add_patch_to_patch_set(self.patches_mut(), &patches);

        file.write_all(to_string_pretty(&self, config)?.as_bytes())?;
        Ok(())
    }

    /// This function saves a [Schema] from memory to a `.json` file with the provided path.
    pub fn save_json(&mut self, path: &Path) -> Result<()> {
        let mut path = path.to_path_buf();
        path.set_extension("json");

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(&path)?);

        // Make sure all definitions are properly sorted by version number.
        self.definitions.iter_mut().for_each(|(_, definitions)| {
            definitions.sort_by(|a, b| b.version().cmp(a.version()));
        });

        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    /// This function exports all the schema files from the provided folder to `.json`.
    ///
    /// For compatibility purposes.
    pub fn export_to_json(schema_folder_path: &Path) -> Result<()> {
        let games = SupportedGames::default();

        games.games_sorted().par_iter().map(|x| x.schema_file_name()).try_for_each(|schema_file| {
            let mut schema_path = schema_folder_path.to_owned();
            schema_path.push(schema_file);

            let mut schema = Schema::load(&schema_path, None)?;
            schema_path.set_extension("json");
            schema.save_json(&schema_path)?;
            Ok(())
        })
    }

    /*
    /// This function exports all the schema files from the `schemas/` folder to `.xml`.
    ///
    /// For compatibility purposes.
    pub fn export_to_xml() -> Result<()> {
        for schema_file in SUPPORTED_GAMES.get_games().iter().map(|x| x.get_schema_name()) {
            let schema = Schema::load(schema_file)?;

            let mut file_path = get_config_path()?.join(SCHEMA_FOLDER);
            file_path.push(schema_file);
            file_path.set_extension("xml");

            let mut file = File::create(&file_path)?;
            file.write_all(quick_xml::se::to_string(&schema)?.as_bytes())?;
        }
        Ok(())
    }
*/

    /// This function allow us to update the provided Schema from a legacy format into the current one.
    pub fn update(schema_path: &Path, schema_patches_path: &Path, game_name: &str) -> Result<()>{
        v4::SchemaV4::update(schema_path, schema_patches_path, game_name)
    }

    /// This function returns all columns that reference the columns on our specific table within the DB Tables of our Schema.
    ///
    /// Returns a list of (local_column_name, vec<(remote_table_name, remote_column_name)>).
    pub fn referencing_columns_for_table(&self, table_name: &str, definition: &Definition) -> HashMap<String, HashMap<String, Vec<String>>> {

        // Iterate over all definitions and find the ones referencing our table/field.
        let fields_processed = definition.fields_processed();
        let definitions = self.definitions();
        let table_name_no_tables = table_name.to_owned().drain(..table_name.len() - 7).collect::<String>();

        fields_processed.iter().filter_map(|field| {

            let references = definitions.par_iter().filter_map(|(ver_name, ver_definitions)| {
                let mut references = ver_definitions.iter().filter_map(|ver_definition| {
                    let ver_patches = Some(ver_definition.patches());
                    let references = ver_definition.fields_processed().iter().filter_map(|ver_field| {
                        if let Some((source_table_name, source_column_name)) = ver_field.is_reference(ver_patches) {
                            if table_name_no_tables == source_table_name && field.name() == source_column_name {
                                Some(ver_field.name().to_owned())
                            } else { None }
                        } else { None }
                    }).collect::<Vec<String>>();
                    if references.is_empty() {
                        None
                    } else {
                        Some(references)
                    }
                }).flatten().collect::<Vec<String>>();
                if references.is_empty() {
                    None
                } else {
                    references.sort();
                    references.dedup();
                    Some((ver_name.to_owned(), references))
                }
            }).collect::<HashMap<String, Vec<String>>>();
            if references.is_empty() {
                None
            } else {
                Some((field.name().to_owned(), references))
            }
        }).collect()
    }

    /// This function returns the list of table/columns that reference the provided columns,
    /// and if there may be a loc entry that changing our column may need a change.
    ///
    /// This supports more than one reference level, except for locs.
    pub fn tables_and_columns_referencing_our_own(
        &self,
        table_name: &str,
        column_name: &str,
        fields: &[Field],
        localised_fields: &[Field]
    ) -> (BTreeMap<String, Vec<String>>, bool) {

        // Make sure the table name is correct.
        let short_table_name = if table_name.ends_with("_tables") { table_name.split_at(table_name.len() - 7).0 } else { table_name };
        let mut tables: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // We get all the db definitions from the schema, then iterate all of them to find what tables/columns reference our own.
        for (ref_table_name, ref_definition) in self.definitions() {
            let mut columns: Vec<String> = vec![];
            for ref_version in ref_definition {
                let ref_fields = ref_version.fields_processed();
                let ref_patches = Some(ref_version.patches());
                let ref_fields_localised = ref_version.localised_fields();
                for ref_field in &ref_fields {
                    if let Some((ref_ref_table, ref_ref_field)) = ref_field.is_reference(ref_patches) {

                        // As this applies to all versions of a table, skip repeated fields.
                        if ref_ref_table == short_table_name && ref_ref_field == column_name && !columns.iter().any(|x| x == ref_field.name()) {
                            columns.push(ref_field.name().to_owned());

                            // If we find a referencing column, get recursion working to check if there is any column referencing this one that needs to be edited.
                            let (ref_of_ref, _) = self.tables_and_columns_referencing_our_own(ref_table_name, ref_field.name(), &ref_fields, ref_fields_localised);
                            for refs in &ref_of_ref {
                                match tables.get_mut(refs.0) {
                                    Some(columns) => for value in refs.1 {
                                        if !columns.contains(value) {
                                            columns.push(value.to_owned());
                                        }
                                    }
                                    None => { tables.insert(refs.0.to_owned(), refs.1.to_vec()); },
                                }
                            }
                        }
                    }
                }
            }

            // Only add them if we actually found columns.
            if !columns.is_empty() {
                tables.insert(ref_table_name.to_owned(), columns);
            }
        }

        // Also, check if we have to be careful about localised fields.
        let patches = self.patches().get(table_name);
        let has_loc_fields = if let Some(field) = fields.iter().find(|x| x.name() == column_name) {
            (field.is_key(patches) || field.name() == "key") && !localised_fields.is_empty()
        } else { false };

        (tables, has_loc_fields)
    }
    /// This function tries to load multiple patches from a str.
    pub fn load_patches_from_str(patch: &str) -> Result<HashMap<String, DefinitionPatch>> {
        from_str(patch).map_err(From::from)
    }

    /// This function tries to load multiple definitions from a str.
    pub fn load_definitions_from_str(definition: &str) -> Result<HashMap<String, Definition>> {
        from_str(definition).map_err(From::from)
    }

    /// This function tries to export a list of patches to a ron string.
    pub fn export_patches_to_str(patches: &HashMap<String, DefinitionPatch>) -> Result<String> {
        let config = PrettyConfig::default();
        ron::ser::to_string_pretty(&patches, config).map_err(From::from)
    }

    /// This function tries to export a list of definitions to a ron string.
    pub fn export_definitions_to_str(definitions: &HashMap<String, Definition>) -> Result<String> {
        let config = PrettyConfig::default();
        ron::ser::to_string_pretty(&definitions, config).map_err(From::from)
    }

    /// This function tries to upload a bunch of [DefinitionPatch] to Sentry's service.
    ///
    /// It requires the **integration_log** feature.
    #[cfg(feature = "integration_log")]
    pub fn upload_patches(sentry_guard: &ClientInitGuard, game_name: &str, patches: HashMap<String, DefinitionPatch>) -> Result<()> {
        let level = Level::Info;
        let message = format!("Schema Patch for: {} - {}.", game_name, crate::utils::current_time()?);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, &patches, config)?;
        let file_name = "patch.txt";

        Logger::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }

    /// This function tries to upload a bunch of [Definition] to Sentry's service.
    ///
    /// It requires the **integration_log** feature.
    #[cfg(feature = "integration_log")]
    pub fn upload_definitions(sentry_guard: &ClientInitGuard, game_name: &str, definitions: HashMap<String, Definition>) -> Result<()> {
        let level = Level::Info;
        let message = format!("Schema Definition for: {} - {}.", game_name, crate::utils::current_time()?);
        let config = PrettyConfig::default();
        let mut data = vec![];
        ron::ser::to_writer_pretty(&mut data, &definitions, config)?;
        let file_name = "definition.txt";

        Logger::send_event(sentry_guard, level, &message, Some((file_name, &data)))
    }
}

impl Definition {

    /// This function creates a new empty `Definition` for the version provided.
    pub fn new(version: i32, schema_patches: Option<&DefinitionPatch>) -> Definition {
        Definition {
            version,
            localised_fields: vec![],
            fields: vec![],
            localised_key_order: vec![],
            patches: schema_patches.cloned().unwrap_or_default(),
        }
    }

    /// This function creates a new empty `Definition` for the version provided, with the fields provided.
    pub fn new_with_fields(version: i32, fields: &[Field], loc_fields: &[Field], schema_patches: Option<&DefinitionPatch>) -> Definition {
        Definition {
            version,
            localised_fields: loc_fields.to_vec(),
            fields: fields.to_vec(),
            localised_key_order: vec![],
            patches: schema_patches.cloned().unwrap_or_default(),
        }
    }

    /// This function returns the reference and lookup data of a definition.
    pub fn reference_data(&self) -> BTreeMap<i32, (String, String, Option<Vec<String>>)> {
        self.fields.iter()
            .enumerate()
            .filter(|x| x.1.is_reference.is_some())
            .map(|x| (x.0 as i32, (x.1.is_reference.clone().unwrap().0, x.1.is_reference.clone().unwrap().1, x.1.lookup.clone())))
            .collect()
    }

    /// This function returns the list of fields a table contains, after it has been expanded/changed due to the attributes of each field.
    pub fn fields_processed(&self) -> Vec<Field> {
        let mut split_colour_fields: BTreeMap<u8, Field> = BTreeMap::new();
        let patches = self.patches();
        let mut fields = self.fields().iter()
            .filter_map(|x|
                if x.is_bitwise() > 1 {
                    let unused = x.unused(Some(patches));
                    let mut fields = vec![x.clone(); x.is_bitwise() as usize];
                    fields.iter_mut().enumerate().for_each(|(index, field)| {
                        field.set_name(format!("{}_{}", field.name(), index + 1));
                        field.set_field_type(FieldType::Boolean);
                        field.set_unused(unused);
                    });
                    Some(fields)
                }

                else if !x.enum_values().is_empty() {
                    let mut field = x.clone();
                    field.set_field_type(FieldType::StringU8);
                    Some(vec![field; 1])
                }

                else if let Some(colour_index) = x.is_part_of_colour() {
                    match split_colour_fields.get_mut(&colour_index) {

                        // If found, add the default value to the other previously known default value.
                        Some(field) => {
                            let default_value = match x.default_value(None) {
                                Some(default_value) => {
                                    if x.name.ends_with("_r") || x.name.ends_with("_red") || x.name == "r" || x.name == "red" {
                                        field.default_value.clone().map(|df| {
                                            format!("{:X}{}", default_value.parse::<i32>().unwrap_or(0), &df[2..])
                                        })
                                    } else if x.name.ends_with("_g") || x.name.ends_with("_green") || x.name == "g" || x.name == "green" {
                                        field.default_value.clone().map(|df| {
                                            format!("{}{:X}{}", &df[..2], default_value.parse::<i32>().unwrap_or(0), &df[4..])
                                        })
                                    } else if x.name.ends_with("_b") || x.name.ends_with("_blue") || x.name == "b" || x.name == "blue" {
                                        field.default_value.clone().map(|df| {
                                            format!("{}{:X}", &df[..4], default_value.parse::<i32>().unwrap_or(0))
                                        })
                                    } else {
                                        Some("000000".to_owned())
                                    }
                                }
                                None => Some("000000".to_owned())
                            };

                            // Update the default value with the one for this colour.
                            field.set_default_value(default_value);

                            if !field.unused(Some(patches)) {
                                field.set_unused(x.unused(Some(patches)));
                            }
                        },
                        None => {
                            let unused = x.unused(Some(patches));
                            let colour_split = x.name().rsplitn(2, '_').collect::<Vec<&str>>();
                            let colour_field_name = if colour_split.len() == 2 {
                                format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST)
                            } else {
                                format!("{}_{}", MERGE_COLOUR_NO_NAME.to_lowercase(), colour_index)
                            };

                            let mut field = x.clone();
                            field.set_name(colour_field_name);
                            field.set_field_type(FieldType::ColourRGB);
                            field.set_unused(unused);

                            // We need to fix the default value so it's a ColourRGB one.
                            let default_value = match field.default_value(None) {
                                Some(default_value) => {
                                    if x.name.ends_with("_r") || x.name.ends_with("_red") || x.name == "r" || x.name == "red" {
                                        Some(format!("{:X}0000", default_value.parse::<i32>().unwrap_or(0)))
                                    } else if x.name.ends_with("_g") || x.name.ends_with("_green") || x.name == "g" || x.name == "green" {
                                        Some(format!("00{:X}00", default_value.parse::<i32>().unwrap_or(0)))
                                    } else if x.name.ends_with("_b") || x.name.ends_with("_blue") || x.name == "b" || x.name == "blue" {
                                        Some(format!("0000{:X}", default_value.parse::<i32>().unwrap_or(0)))
                                    } else {
                                        Some("000000".to_owned())
                                    }
                                }
                                None => Some("000000".to_owned())
                            };

                            field.set_default_value(default_value);

                            split_colour_fields.insert(colour_index, field);
                        }
                    }

                    None
                }

                else {
                    Some(vec![x.clone(); 1])
                }
            )
            .flatten()
            .collect::<Vec<Field>>();

        // Second pass to add the combined colour fields.
        fields.append(&mut split_colour_fields.values().cloned().collect::<Vec<Field>>());
        fields
    }

    /// Note, this doesn't work with combined fields.
    pub fn original_field_from_processed(&self, index: usize) -> Field {
        let fields = self.fields();
        let processed = self.fields_processed();

        let field_processed = &processed[index];
        let name = if field_processed.is_bitwise() > 1 {
            let mut name = field_processed.name().to_owned();
            name.drain(..name.rfind('_').unwrap()).collect::<String>()
        }
        else {field_processed.name().to_owned() };

        fields.iter().find(|x| *x.name() == name).unwrap().clone()
    }

    /// This function returns the field list of a definition, properly sorted.
    pub fn fields_processed_sorted(&self, key_first: bool) -> Vec<Field> {
        let mut fields = self.fields_processed();
        let patches = Some(self.patches());
        fields.sort_by(|a, b| {
            if key_first {
                if a.is_key(patches) && b.is_key(patches) { Ordering::Equal }
                else if a.is_key(patches) && !b.is_key(patches) { Ordering::Less }
                else if !a.is_key(patches) && b.is_key(patches) { Ordering::Greater }
                else { Ordering::Equal }
            }
            else if a.ca_order() == -1 || b.ca_order() == -1 { Ordering::Equal }
            else { a.ca_order().cmp(&b.ca_order()) }
        });
        fields
    }

    /// This function returns the position of a column in a definition, or an error if the column is not found.
    pub fn column_position_by_name(&self, column_name: &str) -> Option<usize> {
        self.fields_processed()
            .iter()
            .position(|x| x.name() == column_name)
    }

    /// This function returns the position of all key columns on a table.
    pub fn key_column_positions(&self) -> Vec<usize> {
        self.fields_processed()
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_key(Some(self.patches())))
            .map(|(x, _)| x)
            .collect::<Vec<_>>()
    }

    /// This function maps a table definition to a `CREATE TABLE` SQL Query.
    ///
    /// NOTE: While this function supports creating a table with foreign keys,
    /// said support is disabled because TW tables are not really foreign key-friendly. Specially in mods.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_create_table_string(&self, table_name: &str) -> String {
        let patches = Some(self.patches());
        let fields_sorted = self.fields_processed();
        let fields_query = fields_sorted.iter().map(|field| field.map_to_sql_string(patches)).collect::<Vec<_>>().join(",");

        let local_keys_join = fields_sorted.iter().filter_map(|field| if field.is_key(patches) { Some(format!("\"{}\"", field.name()))} else { None }).collect::<Vec<_>>().join(",");
        let local_keys = format!("CONSTRAINT unique_key PRIMARY KEY (\"pack_name\", \"file_name\", {local_keys_join})");
        //let foreign_keys = fields_sorted.iter()
        //    .filter_map(|field| field.is_reference(patches).clone().map(|(ref_table, ref_column)| (field.name(), ref_table, ref_column)))
        //    .map(|(loc_name, ref_table, ref_field)| format!("CONSTRAINT fk_{table_name} FOREIGN KEY (\"{loc_name}\") REFERENCES {ref_table}(\"{ref_field}\") ON UPDATE CASCADE ON DELETE CASCADE"))
        //    .collect::<Vec<_>>()
        //    .join(",");

        //if foreign_keys.is_empty() {
            if local_keys_join.is_empty() {
                format!("CREATE TABLE \"{}_v{}\" (\"pack_name\" STRING NOT NULL, \"file_name\" STRING NOT NULL, \"is_vanilla\" INTEGER DEFAULT 0, {})",
                    table_name.replace('\"', "'"),
                    self.version(),
                    fields_query
                )
            } else {
                format!("CREATE TABLE \"{}_v{}\" (\"pack_name\" STRING NOT NULL, \"file_name\" STRING NOT NULL, \"is_vanilla\" INTEGER DEFAULT 0, {}, {})",
                    table_name.replace('\"', "'"),
                    self.version(),
                    fields_query,
                    local_keys
                )
            }
        /*} else if local_keys_join.is_empty() {
            format!("CREATE TABLE \"{}_v{}\" (\"table_unique_id\" INTEGER DEFAULT 0, {}, {})",
                table_name.replace('\"', "'"),
                self.version(),
                fields_query,
                foreign_keys
            )
        } else {
            format!("CREATE TABLE \"{}_v{}\" (\"table_unique_id\" INTEGER DEFAULT 0, {}, {}, {})",
                table_name.replace('\"', "'"),
                self.version(),
                fields_query,
                local_keys,
                foreign_keys
            )
        }*/
    }

    /// This function maps a table definition to a `CREATE TABLE` SQL Query.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_insert_into_string(&self) -> String {
        let fields_sorted = self.fields_processed();
        let fields_query = fields_sorted.iter().map(|field| format!("\"{}\"", field.name())).collect::<Vec<_>>().join(",");
        let fields_query = format!("(\"pack_name\", \"file_name\", \"is_vanilla\", {fields_query})");

        fields_query
    }

    /// This function updates the fields in the provided definition with the data in the provided RawDefinition.
    ///
    /// Not all data is updated though, only:
    /// - Is Key.
    /// - Max Length.
    /// - Default Value.
    /// - Filename Relative Path.
    /// - Is Filename.
    /// - Is Reference.
    /// - Lookup.
    /// - CA Order.
    #[cfg(feature = "integration_assembly_kit")]
    pub fn update_from_raw_definition(&mut self, raw_definition: &RawDefinition, unfound_fields: &mut Vec<String>) {
        let raw_table_name = &raw_definition.name.as_ref().unwrap()[..raw_definition.name.as_ref().unwrap().len() - 4];
        let mut combined_fields = BTreeMap::new();
        for (index, raw_field) in raw_definition.fields.iter().enumerate() {

            let mut found = false;
            for field in &mut self.fields {
                if field.name == raw_field.name {
                    if (raw_field.primary_key == "1" && !field.is_key) || (raw_field.primary_key == "0" && field.is_key) {
                        field.is_key = raw_field.primary_key == "1";
                    }

                    if raw_field.default_value.is_some() {
                        field.default_value = raw_field.default_value.clone();
                    }

                    if let Some(ref path) = raw_field.filename_relative_path {
                        let mut new_path = path.to_owned();
                        if path.contains(",") {
                            new_path = path.split(',').map(|x| x.trim()).join(";");
                        }

                        field.filename_relative_path = Some(new_path);
                    }

                    // Some fields are marked as filename, but only have fragment paths, which do not seem to correlate to game file paths.
                    // We need to disable those to avoid false positives on diagnostics.
                    field.is_filename = match raw_field.is_filename {
                        Some(_) => !(raw_field.fragment_path.is_some() && raw_field.filename_relative_path.is_none()),
                        None => false,
                    };

                    // Make sure to cleanup any old invalid definition.
                    if let Some(ref description) = raw_field.field_description {
                        field.description = description.to_owned();
                    } else {
                        field.description = String::new();
                    }

                    // We reset these so we don't inherit wrong references from older tables.
                    field.is_reference = Default::default();
                    field.lookup = Default::default();
                    if let Some(ref table) = raw_field.column_source_table {
                        if let Some(ref columns) = raw_field.column_source_column {
                            if !table.is_empty() && !columns.is_empty() && !columns[0].is_empty() {
                                field.is_reference = Some((table.to_owned(), columns[0].to_owned()));
                                if columns.len() > 1 {
                                    field.lookup = Some(columns[1..].to_vec());
                                }
                            }
                        }
                    }

                    field.ca_order = index as i16;

                    // Detect and group colour fiels.
                    let is_numeric = matches!(field.field_type, FieldType::I16 | FieldType::I32 | FieldType::I64 | FieldType::F32 | FieldType::F64);

                    if is_numeric && (
                        field.name.ends_with("_r") ||
                        field.name.ends_with("_g") ||
                        field.name.ends_with("_b") ||
                        field.name.ends_with("_red") ||
                        field.name.ends_with("_green") ||
                        field.name.ends_with("_blue") ||
                        field.name == "r" ||
                        field.name == "g" ||
                        field.name == "b" ||
                        field.name == "red" ||
                        field.name == "green" ||
                        field.name == "blue"
                    ) {
                        let colour_split = field.name.rsplitn(2, '_').collect::<Vec<&str>>();
                        let colour_field_name = if colour_split.len() == 2 { format!("{}{}", colour_split[1].to_lowercase(), MERGE_COLOUR_POST) } else { MERGE_COLOUR_NO_NAME.to_lowercase() };

                        match combined_fields.get(&colour_field_name) {
                            Some(group_key) => field.is_part_of_colour = Some(*group_key),
                            None => {
                                let group_key = combined_fields.keys().len() as u8 + 1;
                                combined_fields.insert(colour_field_name.to_owned(), group_key);
                                field.is_part_of_colour = Some(group_key);
                            }
                        }
                    }
                    found = true;
                    break;
                }
            }

            if !found {

                // We need to check if it's a loc field before reporting it as unfound.
                for loc_field in self.localised_fields() {
                    if loc_field.name == raw_field.name {
                        found = true;
                        break;
                    }
                }

                // We automatically ignore certain old fields that have nothing to do with the game's data.
                if !found && !IGNORABLE_FIELDS.contains(&&*raw_field.name) {
                    unfound_fields.push(format!("{}/{}", raw_table_name, raw_field.name));
                }
            }
        }
    }

    /// This function populates the `localised_fields` of a definition with data from the assembly kit.
    #[cfg(feature = "integration_assembly_kit")]
    pub fn update_from_raw_localisable_fields(&mut self, raw_definition: &RawDefinition, raw_localisable_fields: &[RawLocalisableField]) {
        let raw_table_name = &raw_definition.name.as_ref().unwrap()[..raw_definition.name.as_ref().unwrap().len() - 4];
        let localisable_fields_names = raw_localisable_fields.iter()
            .filter(|x| x.table_name == raw_table_name)
            .map(|x| &*x.field)
            .collect::<Vec<&str>>();

        if !localisable_fields_names.is_empty() {
            let localisable_fields = raw_definition.fields.iter()
                .filter(|x| localisable_fields_names.contains(&&*x.name))
                .collect::<Vec<&RawField>>();

            self.localised_fields = localisable_fields.iter().map(|x| From::from(*x)).collect();

            // Set their type to StringU8 for consistency.
            self.localised_fields.iter_mut().for_each(|field| field.field_type = FieldType::StringU8);
        }
    }
}

/// Implementation of `Field`.
impl Field {

    /// This function creates a `Field` using the provided data.
    pub fn new(
        name: String,
        field_type: FieldType,
        is_key: bool,
        default_value: Option<String>,
        is_filename: bool,
        filename_relative_path: Option<String>,
        is_reference: Option<(String, String)>,
        lookup: Option<Vec<String>>,
        description: String,
        ca_order: i16,
        is_bitwise: i32,
        enum_values: BTreeMap<i32, String>,
        is_part_of_colour: Option<u8>,
    ) -> Self {
        Self {
            name,
            field_type,
            is_key,
            default_value,
            is_filename,
            filename_relative_path,
            is_reference,
            lookup,
            description,
            ca_order,
            is_bitwise,
            enum_values,
            is_part_of_colour,
            unused: false
        }
    }

    //----------------------------------------------------------------------//
    // Manual getter implementations, because we need to tweak some of them.
    //----------------------------------------------------------------------//
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }
    pub fn is_key(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_key") {
                    return field_patch.parse().unwrap_or(false);
                }
            }
        }

        self.is_key
    }

    pub fn default_value(&self, schema_patches: Option<&DefinitionPatch>) -> Option<String> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("default_value") {
                    return Some(field_patch.to_string());
                }
            }
        }

        self.default_value.clone()
    }

    pub fn is_filename(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_filename") {
                    return field_patch.parse().unwrap_or(false);
                }
            }
        }

        self.is_filename
    }

    pub fn filename_relative_path(&self, schema_patches: Option<&DefinitionPatch>) -> Option<Vec<String>> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("filename_relative_path") {
                    return Some(field_patch.replace('\\', "/").split(';').map(|x| x.to_string()).collect::<Vec<String>>());
                }
            }
        }

        self.filename_relative_path.clone().map(|x| x.replace('\\', "/").split(';').map(|x| x.to_string()).collect::<Vec<String>>())
    }

    pub fn is_reference(&self, schema_patches: Option<&DefinitionPatch>) -> Option<(String,String)> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("is_reference") {
                    let split = field_patch.splitn(2, ';').collect::<Vec<_>>();
                    if split.len() == 2 {
                        return Some((split[0].to_string(), split[1].to_string()));
                    }
                }
            }
        }

        self.is_reference.clone()
    }

    pub fn lookup(&self, schema_patches: Option<&DefinitionPatch>) -> Option<Vec<String>> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("lookup") {
                    return Some(field_patch.split(';').map(|x| x.to_string()).collect());
                }
            }
        }

        self.lookup.clone()
    }

    pub fn lookup_no_patch(&self) -> Option<Vec<String>> {
        self.lookup.clone()
    }

    pub fn lookup_hardcoded(&self, schema_patches: Option<&DefinitionPatch>) -> HashMap<String, String> {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("lookup_hardcoded") {
                    let entries = field_patch.split(":::::").map(|x| x.split(";;;;;").collect::<Vec<_>>()).collect::<Vec<_>>();
                    let mut hashmap = HashMap::new();
                    for entry in entries {
                        hashmap.insert(entry[0].to_owned(), entry[1].to_owned());
                    }
                    return hashmap;
                }
            }
        }

        HashMap::new()
    }

    pub fn description(&self, schema_patches: Option<&DefinitionPatch>) -> String {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(field_patch) = patch.get("description") {
                    return field_patch.to_owned();
                }
            }
        }

        self.description.to_owned()
    }

    pub fn ca_order(&self) ->  i16 {
        self.ca_order
    }
    pub fn is_bitwise(&self) -> i32 {
        self.is_bitwise
    }
    pub fn enum_values(&self) -> &BTreeMap<i32,String> {
        &self.enum_values
    }

    /// Getter for the `enum_values` field, in an option.
    pub fn enum_values_to_option(&self) -> Option<BTreeMap<i32, String>> {
        if self.enum_values.is_empty() { None }
        else { Some(self.enum_values.clone()) }
    }

    /// Getter for the `enum_values` field in a string format.
    pub fn enum_values_to_string(&self) -> String {
        self.enum_values.iter().map(|(x, y)| format!("{x},{y}")).collect::<Vec<String>>().join(";")
    }

    pub fn is_part_of_colour(&self) -> Option<u8>{
        self.is_part_of_colour
    }

    /// Getter for the `cannot_be_empty` field.
    pub fn cannot_be_empty(&self, schema_patches: Option<&DefinitionPatch>) -> bool {
        if let Some(schema_patches) = schema_patches {
            if let Some(patch) = schema_patches.get(self.name()) {
                if let Some(cannot_be_empty) = patch.get("not_empty") {
                    return cannot_be_empty.parse::<bool>().unwrap_or(false);
                }
            }
        }

        false
    }

    /// Getter for the `unused` field.
    pub fn unused(&self, schema_patches: Option<&DefinitionPatch>) -> bool {

        // By default all fields are used, except the ones set through patches. If it's unused, skip patches.
        self.unused || {

            if let Some(schema_patches) = schema_patches {
                if let Some(patch) = schema_patches.get(self.name()) {
                    if let Some(cannot_be_empty) = patch.get("unused") {
                        return cannot_be_empty.parse::<bool>().unwrap_or(false);
                    }
                }
            }

            false
        }
    }

    /// This function maps our field to a String ready to be used in a SQL `CREATE TABLE` command.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_string(&self, schema_patches: Option<&DefinitionPatch>) -> String {
        let mut string = format!(" \"{}\" {:?} ", self.name(), self.field_type().map_to_sql_type());

        if let Some(default_value) = self.default_value(schema_patches) {
            string.push_str(&format!(" DEFAULT \"{}\"", default_value.replace("\"", "\"\"")));
        }

        string
    }
}

impl FieldType {

    /// This function maps our type to a SQLite Type.
    #[cfg(feature = "integration_sqlite")]
    pub fn map_to_sql_type(&self) -> Type {
        match self {
            FieldType::Boolean => Type::Integer,
            FieldType::F32 => Type::Real,
            FieldType::F64 => Type::Real,
            FieldType::I16 => Type::Integer,
            FieldType::I32 => Type::Integer,
            FieldType::I64 => Type::Integer,
            FieldType::ColourRGB => Type::Text,
            FieldType::StringU8 => Type::Text,
            FieldType::StringU16 => Type::Text,
            FieldType::OptionalI16 => Type::Integer,
            FieldType::OptionalI32 => Type::Integer,
            FieldType::OptionalI64 => Type::Integer,
            FieldType::OptionalStringU8 => Type::Text,
            FieldType::OptionalStringU16 => Type::Text,
            FieldType::SequenceU16(_) => Type::Blob,
            FieldType::SequenceU32(_) => Type::Blob,
        }
    }
}
//---------------------------------------------------------------------------//
//                         Extra Implementations
//---------------------------------------------------------------------------//

/// Default implementation of `Schema`.
impl Default for Schema {
    fn default() -> Self {
        Self {
            version: CURRENT_STRUCTURAL_VERSION,
            definitions: HashMap::new(),
            patches: HashMap::new()
        }
    }
}

/// Default implementation of `FieldType`.
impl Default for Field {
    fn default() -> Self {
        Self {
            name: String::from("new_field"),
            field_type: FieldType::StringU8,
            is_key: false,
            default_value: None,
            is_filename: false,
            filename_relative_path: None,
            is_reference: None,
            lookup: None,
            description: String::from(""),
            ca_order: -1,
            is_bitwise: 0,
            enum_values: BTreeMap::new(),
            is_part_of_colour: None,
            unused: false,
        }
    }
}

/// Display implementation of `FieldType`.
impl Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::F32 => write!(f, "F32"),
            FieldType::F64 => write!(f, "F64"),
            FieldType::I16 => write!(f, "I16"),
            FieldType::I32 => write!(f, "I32"),
            FieldType::I64 => write!(f, "I64"),
            FieldType::ColourRGB => write!(f, "ColourRGB"),
            FieldType::StringU8 => write!(f, "StringU8"),
            FieldType::StringU16 => write!(f, "StringU16"),
            FieldType::OptionalI16 => write!(f, "OptionalI16"),
            FieldType::OptionalI32 => write!(f, "OptionalI32"),
            FieldType::OptionalI64 => write!(f, "OptionalI64"),
            FieldType::OptionalStringU8 => write!(f, "OptionalStringU8"),
            FieldType::OptionalStringU16 => write!(f, "OptionalStringU16"),
            FieldType::SequenceU16(_) => write!(f, "SequenceU16"),
            FieldType::SequenceU32(_) => write!(f, "SequenceU32"),
        }
    }
}

/// Implementation of `From<&RawDefinition>` for `Definition.
impl From<&DecodedData> for FieldType {
    fn from(data: &DecodedData) -> Self {
        match data {
            DecodedData::Boolean(_) => FieldType::Boolean,
            DecodedData::F32(_) => FieldType::F32,
            DecodedData::F64(_) => FieldType::F64,
            DecodedData::I16(_) => FieldType::I16,
            DecodedData::I32(_) => FieldType::I32,
            DecodedData::I64(_) => FieldType::I64,
            DecodedData::ColourRGB(_) => FieldType::ColourRGB,
            DecodedData::StringU8(_) => FieldType::StringU8,
            DecodedData::StringU16(_) => FieldType::StringU16,
            DecodedData::OptionalI16(_) => FieldType::OptionalI16,
            DecodedData::OptionalI32(_) => FieldType::OptionalI32,
            DecodedData::OptionalI64(_) => FieldType::OptionalI64,
            DecodedData::OptionalStringU8(_) => FieldType::OptionalStringU8,
            DecodedData::OptionalStringU16(_) => FieldType::OptionalStringU16,
            DecodedData::SequenceU16(_) => FieldType::SequenceU16(Box::new(Definition::new(INVALID_VERSION, None))),
            DecodedData::SequenceU32(_) => FieldType::SequenceU32(Box::new(Definition::new(INVALID_VERSION, None))),
        }
    }
}

/// Special serializer function to sort the definitions HashMap before serializing.
fn ordered_map_definitions<S>(value: &HashMap<String, Vec<Definition>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

/// Special serializer function to sort the patches HashMap before serializing.
fn ordered_map_patches<S>(value: &HashMap<String, HashMap<String, HashMap<String, String>>>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, BTreeMap<_, BTreeMap<_, _>>> = value.iter().map(|(a, x)| (a, x.iter().map(|(b, y)| (b, y.iter().collect())).collect())).collect();
    ordered.serialize(serializer)
}
