//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the [Optimizable] and [OptimizableContainer] trait.

use getset::{Getters, Setters};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{Container, ContainerPath, db::DB, EncodeableExtraData, FileType, loc::Loc, pack::Pack, portrait_settings::PortraitSettings, RFile, RFileDecoded, table::DecodedData, text::TextFormat};
use rpfm_lib::games::{GameInfo, supported_games::KEY_WARHAMMER_3};
use rpfm_lib::schema::Schema;

use crate::dependencies::{Dependencies, KEY_DELETES_TABLE_NAME};

const EMPTY_MASK_PATH_END: &str = "empty_mask.png";
const DEFAULT_KEY_DELETES_FILE: &str = "db/twad_key_deletes_tables/generated_deletes";

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Optimizable`, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Optimizable {

    /// This function optimizes the provided struct to reduce its size and improve compatibility.
    ///
    /// It returns if the struct has been left in an state where it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies, container: Option<&mut Pack>, options: &OptimizerOptions) -> bool;
}

/// This trait marks a [Container] as an `Optimizable` container, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait OptimizableContainer: Container {

    /// This function optimizes the provided [Container] to reduce its size and improve compatibility.
    ///
    /// It returns the list of files that has been safetly deleted during the optimization process.
    fn optimize(&mut self,
        paths_to_optimize: Option<Vec<ContainerPath>>,
        dependencies: &mut Dependencies,
        schema: &Schema,
        game: &GameInfo,
        options: &OptimizerOptions,
    ) -> Result<(HashSet<String>, HashSet<String>)>;
}

/// Struct containing the configurable options for the optimizer.
#[derive(Clone, Debug, Getters, Setters, Deserialize, Serialize)]
#[getset(get = "pub", set = "pub")]
pub struct OptimizerOptions {

    /// Allow the optimizer to remove files unchanged from vanilla, reducing the pack size.
    pack_remove_itm_files: bool,

    /// Allows the optimizer to update the twad_key_deletes table using the data cored tables in your pack to guess the keys.
    ///
    /// IT DOESN'T DELETE THE DATACORED TABLES.
    db_import_datacores_into_twad_key_deletes: bool,

    /// Allow the optimizer to optimize datacored tables. THIS IS NOT RECOMMENDED, as datacored tables usually are they way they are for a reason.
    ///
    /// THIS IS NOT RECOMMENDED, as datacored tables usually are the way they are for a reason.
    db_optimize_datacored_tables: bool,

    /// Allows the optimizer to remove duplicated rows from db and loc files.
    table_remove_duplicated_entries: bool,

    /// Allows the optimizer to remove ITM (Identical To Master) rows from db and loc files.
    table_remove_itm_entries: bool,

    /// Allows the optimizer to remove ITNR (Identical To New Row) rows from db and loc files.
    table_remove_itnr_entries: bool,

    /// Allows the optimizer to remove empty db and loc files.
    table_remove_empty_file: bool,

    /// Allows the optimizer to remove unused xml files in map folders.
    text_remove_unused_xml_map_folders: bool,

    /// Allows the optimizer to remove unused xml files in the prefab folder.
    text_remove_unused_xml_prefab_folder: bool,

    /// Allows the optimizer to remove unused agf files.
    text_remove_agf_files: bool,

    /// Allows the optimizer to remove unused model_statistics files.
    text_remove_model_statistics_files: bool,

    /// Allow the optimizer to remove unused art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused art sets are actually unused and not caused by a typo.
    pts_remove_unused_art_sets: bool,

    /// Allow the optimizer to remove unused variants from art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused variants are actually unused and not caused by a typo.
    pts_remove_unused_variants: bool,

    /// Allow the optimizer to remove empty masks in Portrait Settings file, reducing their side.
    ///
    /// Ingame there's no difference between an empty mask and an invalid one, so it's better to remove them to reduce their size.
    pts_remove_empty_masks: bool,

    /// Allows the optimizer to remove empty Portrait Settings files.
    pts_remove_empty_file: bool,
}

//-------------------------------------------------------------------------------//
//                           Trait implementations
//-------------------------------------------------------------------------------//

impl Default for OptimizerOptions {
    fn default() -> Self {
        Self {
            pack_remove_itm_files: true,
            db_import_datacores_into_twad_key_deletes: false,
            db_optimize_datacored_tables: false,
            table_remove_duplicated_entries: true,
            table_remove_itm_entries: true,
            table_remove_itnr_entries: true,
            table_remove_empty_file: true,
            text_remove_unused_xml_map_folders: true,
            text_remove_unused_xml_prefab_folder: true,
            text_remove_agf_files: true,
            text_remove_model_statistics_files: true,
            pts_remove_unused_art_sets: false,
            pts_remove_unused_variants: false,
            pts_remove_empty_masks: false,
            pts_remove_empty_file: true,
        }
    }
}

impl OptimizableContainer for Pack {

    /// This function optimizes the provided [Pack] file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - DB/Loc tables (except if the table has the same name as his vanilla/parent counterpart and `optimize_datacored_tables` is false):
    ///     - Removal of duplicated entries.
    ///     - Removal of ITM (Identical To Master) entries.
    ///     - Removal of ITNR (Identical To New Row) entries.
    ///     - Removal of empty tables.
    ///     - Conversion of datacores into twad_key_deletes_entries.
    /// - Text files:
    ///     - Removal of XML files in map folders (extra files resulting of Terry export process).
    ///     - Removal of XML files in prefabs folder (extra files resulting of Terry export process).
    ///     - Removal of .agf files (byproduct of bob exporting models).
    ///     - Removal of .model_statistics files (byproduct of bob exporting models).
    /// - Portrait Settings files:
    ///     - Removal of variants not present in the variants table (unused data).
    ///     - Removal of art sets not present in the campaign_character_arts table (unused data).
    ///     - Removal of empty masks.
    ///     - Removal of empty Portrait Settings files.
    /// - Pack:
    ///     - Remove files identical to parent/vanilla.
    fn optimize(&mut self,
        paths_to_optimize: Option<Vec<ContainerPath>>,
        dependencies: &mut Dependencies,
        schema: &Schema,
        game: &GameInfo,
        options: &OptimizerOptions
    ) -> Result<(HashSet<String>, HashSet<String>)> {
        let mut files_to_add: HashSet<String> = HashSet::new();
        let mut files_to_delete: HashSet<String> = HashSet::new();

        // We can only optimize if we have vanilla data available.
        if !dependencies.is_vanilla_data_loaded(false) {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate);
        }

        // If we're importing the datacored deletions, create the file for them if it doesn't exist.
        if options.db_import_datacores_into_twad_key_deletes && game.key() == KEY_WARHAMMER_3 {
            if let Some(def) = schema.definitions_by_table_name(KEY_DELETES_TABLE_NAME) {
                if def.len() >= 1 {
                    let table = DB::new(&def[0], None, KEY_DELETES_TABLE_NAME);
                    let _ = self.insert(RFile::new_from_decoded(&RFileDecoded::DB(table), 0, DEFAULT_KEY_DELETES_FILE));
                    files_to_add.insert(DEFAULT_KEY_DELETES_FILE.to_owned());
                }
            }
        }

        // Cache the pack paths for the text file checks.
        let pack_paths = self.paths().keys().map(|x| x.to_owned()).collect::<HashSet<String>>();
        let mut self_copy = self.clone();

        // List of files to optimize.
        let mut files_to_optimize = match paths_to_optimize {
            Some(paths) => self.files_by_paths_mut(&paths, false),
            None => self.files_mut().values_mut().collect::<Vec<_>>(),
        };


        // Import into twad_key_deletes is only supported in wh3, as that table is only in that game... for now.
        if options.db_import_datacores_into_twad_key_deletes && game.key() == KEY_WARHAMMER_3 {
            let mut generated_rows = vec![];
            let datacores = files_to_optimize.iter()
                .filter(|x| x.file_type() == FileType::DB && dependencies.file_exists(x.path_in_container_raw(), true, true, true))
                .collect::<Vec<_>>();

            for datacore in datacores {
                if let Ok(dep_file) = dependencies.file(datacore.path_in_container_raw(), true, true, true) {
                    if let Ok(RFileDecoded::DB(dep_table)) = dep_file.decoded() {
                        if let Ok(RFileDecoded::DB(datacore_table)) = datacore.decoded() {
                            let mut datacore_keys: HashSet<String> = HashSet::new();
                            let key_cols = datacore_table.definition().key_column_positions();
                            datacore_keys.extend(datacore_table.data()
                                .iter()
                                .map(|x| key_cols.iter()
                                    .map(|y| x[*y].data_to_string())
                                    .join("")
                                )
                                .collect::<Vec<_>>()
                            );

                            let mut dep_keys = HashSet::new();
                            let key_cols = dep_table.definition().key_column_positions();
                            dep_keys.extend(dep_table.data()
                                .iter()
                                .map(|x| key_cols.iter()
                                    .map(|y| x[*y].data_to_string())
                                    .join("")
                                )
                                .collect::<Vec<_>>()
                            );

                            let table_name_dec_data = DecodedData::StringU8(datacore_table.table_name_without_tables().to_owned());
                            for key in dep_keys {
                                if !datacore_keys.contains(&key) {
                                    generated_rows.push(vec![table_name_dec_data.clone(), DecodedData::StringU8(key.to_owned())]);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(file) = files_to_optimize.iter_mut().find(|x| x.path_in_container_raw() == DEFAULT_KEY_DELETES_FILE) {
                if let Ok(RFileDecoded::DB(db)) = file.decoded_mut() {
                    let _ = db.set_data(&generated_rows);
                }
            }
        }

        // Pass to identify and remove itms.
        if options.pack_remove_itm_files {
            let extra_data = Some(EncodeableExtraData::new_from_game_info(game));
            for rfile in &mut files_to_optimize {
                if let Ok(dep_file) = dependencies.file_mut(rfile.path_in_container_raw(), true, true) {
                    if let Ok(local_hash) = rfile.data_hash(&extra_data) {
                        if let Ok(dependency_hash) = dep_file.data_hash(&extra_data) {
                            if local_hash == dependency_hash {
                                files_to_delete.insert(rfile.path_in_container_raw().to_string());
                            }
                        }
                    }
                }
            }
        }

        // Then, do a second pass, this time over the decodeable files that we can optimize.
        files_to_delete.extend(files_to_optimize.iter_mut().filter_map(|rfile| {

            // Only check it if it's not already marked for deletion.
            let path = rfile.path_in_container_raw().to_owned();
            if !files_to_delete.contains(&path) {

                match rfile.file_type() {
                    FileType::DB => {

                        // Unless we specifically wanted to, ignore the same-name-as-vanilla-or-parent files,
                        // as those are probably intended to overwrite vanilla files, not to be optimized.
                        if options.db_optimize_datacored_tables || !dependencies.file_exists(&path, true, true, true) {
                            if let Ok(RFileDecoded::DB(db)) = rfile.decoded_mut() {
                                if db.optimize(dependencies, Some(&mut self_copy), options) {
                                    if options.table_remove_empty_file {
                                        return Some(path);
                                    }
                                }
                            }
                        }
                    }

                    FileType::Loc => {

                        // Same as with tables, don't optimize them if they're overwriting.
                        if options.db_optimize_datacored_tables || !dependencies.file_exists(&path, true, true, true) {
                            if let Ok(RFileDecoded::Loc(loc)) = rfile.decoded_mut() {
                                if loc.optimize(dependencies, Some(&mut self_copy), options) {
                                    if options.table_remove_empty_file {
                                        return Some(path);
                                    }
                                }
                            }
                        }
                    }

                    FileType::Text => {

                        // agf and model_statistics are debug files outputed by bob in older games.
                        if (options.text_remove_agf_files && path.ends_with(".agf")) ||
                            (options.text_remove_model_statistics_files && path.ends_with(".model_statistics")) {
                            if let Ok(Some(RFileDecoded::Text(_))) = rfile.decode(&None, false, true) {
                                return Some(path);
                            }
                        }

                        else if !path.is_empty() && (
                                (options.text_remove_unused_xml_prefab_folder && path.starts_with("prefabs/")) ||
                                (options.text_remove_unused_xml_map_folders && (
                                    path.starts_with("terrain/battles/") ||
                                    path.starts_with("terrain/tiles/battle/")
                                ))
                            )
                            && !path.ends_with(".wsmodel")
                            && !path.ends_with(".environment")
                            && !path.ends_with(".environment_group")
                            && !path.ends_with(".environment_group.override")

                            // Delete all xml files that match a bin file.
                            && (
                                path.ends_with(".xml") && (
                                    pack_paths.contains(&path[..path.len() - 4].to_lowercase()) ||
                                    pack_paths.contains(&(path[..path.len() - 4].to_lowercase() + ".bin"))
                                )
                            )
                         {
                            if let Ok(Some(RFileDecoded::Text(text))) = rfile.decode(&None, false, true) {
                                if *text.format() == TextFormat::Xml {
                                    return Some(path);

                                }
                            }
                        }
                    }

                    FileType::PortraitSettings => {

                        // In portrait settings file we look to cleanup variants and art sets that are not referenced by the game tables.
                        // Meaning they are not used by the game.
                        if let Ok(RFileDecoded::PortraitSettings(ps)) = rfile.decoded_mut() {
                            if ps.optimize(dependencies, Some(&mut self_copy), options) {
                                if options.pts_remove_empty_file {
                                    return Some(path);
                                }
                            }
                        }
                    }

                    // Ignore the rest.
                    _ => {}
                }
            }

            None
        }).collect::<Vec<String>>());

        // Delete all the files marked for deletion.
        files_to_delete.iter().for_each(|x| { self.remove(&ContainerPath::File(x.to_owned())); });

        // Return the deleted files, so the caller can know what got removed.
        Ok((files_to_delete, files_to_add))
    }
}

impl Optimizable for DB {

    /// This function optimizes the provided [DB] file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of duplicated entries.
    /// - Removal of ITM (Identical To Master) entries.
    /// - Removal of ITNR (Identical To New Row) entries.
    ///
    /// It returns if the DB is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies, container: Option<&mut Pack>, options: &OptimizerOptions) -> bool {
        let container = match container {
            Some(container) => container,
            None => return false,
        };

        // Get a manipulable copy of all the entries, so we can optimize it.
        let mut entries = self.data().to_vec();

        match dependencies.db_data_datacored(self.table_name(), container, true, true) {
            Ok(mut vanilla_tables) => {

                // First, merge all vanilla and parent db fragments into a single HashSet.
                let vanilla_table = vanilla_tables.iter_mut()
                    .filter_map(|file| {
                        if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                            Some(table.data().to_vec())
                        } else { None }
                    })
                    .flatten()
                    .map(|x| {

                        // We map all floats here to string representations of floats, so we can actually compare them reliably.
                        let json = x.iter().map(|data|
                            if let DecodedData::F32(value) = data {
                                DecodedData::StringU8(format!("{value:.4}"))
                            } else if let DecodedData::F64(value) = data {
                                DecodedData::StringU8(format!("{value:.4}"))
                            } else {
                                data.to_owned()
                            }
                        ).collect::<Vec<DecodedData>>();
                        serde_json::to_string(&json).unwrap()
                    })
                    .collect::<HashSet<String>>();

                // Remove ITM and ITNR entries.
                let new_row = self.new_row().iter().map(|data|
                    if let DecodedData::F32(value) = data {
                        DecodedData::StringU8(format!("{value:.4}"))
                    } else if let DecodedData::F64(value) = data {
                        DecodedData::StringU8(format!("{value:.4}"))
                    } else {
                        data.to_owned()
                    }
                ).collect::<Vec<DecodedData>>();

                entries.retain(|entry| {
                    let entry_json = entry.iter().map(|data|
                        if let DecodedData::F32(value) = data {
                            DecodedData::StringU8(format!("{value:.4}"))
                        } else if let DecodedData::F64(value) = data {
                            DecodedData::StringU8(format!("{value:.4}"))
                        } else {
                            data.to_owned()
                        }
                    ).collect::<Vec<DecodedData>>();

                    (!options.table_remove_itm_entries || (
                        options.table_remove_itm_entries &&
                        !vanilla_table.contains(&serde_json::to_string(&entry_json).unwrap()))
                    ) &&
                    (!options.table_remove_itnr_entries || (
                        options.table_remove_itnr_entries &&
                        entry != &new_row)
                    )
                });

                // Dedupper. This is slower than a normal dedup, but it doesn't reorder rows.
                if options.table_remove_duplicated_entries {
                    let mut dummy_set = HashSet::new();
                    entries.retain(|x| dummy_set.insert(x.clone()));
                }

                // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                //
                // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                let _ = self.set_data(&entries);
                self.data().is_empty()
            }
            Err(_) => false,
        }
    }
}

impl Optimizable for Loc {

    /// This function optimizes the provided [Loc] file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of duplicated entries.
    /// - Removal of ITM (Identical To Master) entries.
    /// - Removal of ITNR (Identical To New Row) entries.
    ///
    /// It returns if the Loc is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies, _container: Option<&mut Pack>, options: &OptimizerOptions) -> bool {

        // Get a manipulable copy of all the entries, so we can optimize it.
        let mut entries = self.data().to_vec();
        match dependencies.loc_data(true, true) {
            Ok(mut vanilla_tables) => {

                // First, merge all vanilla and parent locs into a single HashMap<key, value>. We don't care about the third column.
                let vanilla_table = vanilla_tables.iter_mut()
                    .filter_map(|file| {
                        if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                            Some(table.data().to_vec())
                        } else { None }
                    })
                    .flat_map(|data| data.iter()
                        .map(|data| (data[0].data_to_string().to_string(), data[1].data_to_string().to_string()))
                        .collect::<Vec<(String, String)>>())
                    .collect::<HashMap<String, String>>();

                // Remove ITM and ITNR entries.
                let new_row = self.new_row();
                entries.retain(|entry| {
                    if options.table_remove_itnr_entries && entry == &new_row {
                        return false;
                    }

                    if options.table_remove_itm_entries {
                        match vanilla_table.get(&*entry[0].data_to_string()) {
                            Some(vanilla_value) => return &*entry[1].data_to_string() != vanilla_value,
                            None => return true
                        }
                    }

                    true
                });

                // Dedupper. This is slower than a normal dedup, but it doesn't reorder rows.
                if options.table_remove_duplicated_entries {
                    let mut dummy_set = HashSet::new();
                    entries.retain(|x| dummy_set.insert(x.clone()));
                }

                // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                //
                // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                let _ = self.set_data(&entries);
                self.data().is_empty()
            }
            Err(_) => false,
        }
    }
}

impl Optimizable for PortraitSettings {

    /// This function optimizes the provided [PortraitSettings] file in order to make it smaller.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of variants not present in the variants table (unused data).
    /// - Removal of art sets not present in the campaign_character_arts table (unused data).
    ///
    /// It returns if the PortraitSettings is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies, container: Option<&mut Pack>, options: &OptimizerOptions) -> bool {

        // Get a manipulable copy of all the entries, so we can optimize it.
        let mut entries = self.entries().to_vec();

        // Get the list of art set ids and variant filenames to check against.
        let art_set_ids = dependencies.db_values_from_table_name_and_column_name(container.as_deref(), "campaign_character_arts_tables", "art_set_id", true, true);
        let mut variant_filenames = dependencies.db_values_from_table_name_and_column_name(container.as_deref(), "variants_tables", "variant_filename", true, true);
        if variant_filenames.is_empty() {
            variant_filenames = dependencies.db_values_from_table_name_and_column_name(container.as_deref(), "variants_tables", "variant_name", true, true);
        }

        // Do not do anything if we don't have ids and variants.
        if art_set_ids.is_empty() || variant_filenames.is_empty() {
            return false;
        }

        entries.retain_mut(|entry| {
            entry.variants_mut().retain_mut(|variant| {
                if options.pts_remove_empty_masks {
                    if variant.file_mask_1().ends_with(EMPTY_MASK_PATH_END) {
                        variant.file_mask_1_mut().clear();
                    }
                    if variant.file_mask_2().ends_with(EMPTY_MASK_PATH_END) {
                        variant.file_mask_2_mut().clear();
                    }
                    if variant.file_mask_3().ends_with(EMPTY_MASK_PATH_END) {
                        variant.file_mask_3_mut().clear();
                    }
                }

                if options.pts_remove_unused_variants {
                    variant_filenames.contains(variant.filename())
                } else {
                    true
                }
            });

            if options.pts_remove_unused_art_sets {
                art_set_ids.contains(entry.id())
            } else {
                true
            }
        });

        self.set_entries(entries);
        self.entries().is_empty()
    }
}
