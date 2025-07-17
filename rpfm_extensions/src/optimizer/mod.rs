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

use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{Container, ContainerPath, db::DB, FileType, loc::Loc, pack::Pack, portrait_settings::PortraitSettings, RFileDecoded, table::DecodedData, text::TextFormat};
use rpfm_lib::schema::Schema;

use crate::dependencies::Dependencies;

const EMPTY_MASK_PATH_END: &str = "empty_mask.png";

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
        options: &OptimizerOptions,
    ) -> Result<HashSet<String>>;
}

/// Struct containing the configurable options for the optimizer.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OptimizerOptions {

    /// Allow the optimizer to optimize datacored tables. THIS IS NOT RECOMMENDED, as datacored tables usually are they way they are for a reason.
    optimize_datacored_tables: bool,

    /// Allow the optimizer to remove unused art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused art sets are actually unused and not caused by a typo.
    remove_unused_art_sets: bool,

    /// Allow the optimizer to remove unused variants from art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused variants are actually unused and not caused by a typo.
    remove_unused_variants: bool,

    /// Allow the optimizer to remove empty masks in Portrait Settings file, reducing their side.
    ///
    /// Ingame there's no difference between an empty mask and an invalid one, so it's better to remove them to reduce their size.
    remove_empty_masks: bool,
}

impl OptimizerOptions {

    /// Allow the optimizer to optimize datacored tables.
    ///
    /// THIS IS NOT RECOMMENDED, as datacored tables usually are they way they are for a reason.
    pub fn set_optimize_datacored_tables(&mut self, enable: bool) -> &mut Self {
        self.optimize_datacored_tables = enable;
        self
    }

    /// Allow the optimizer to remove unused art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused art sets are actually unused and not caused by a typo.
    pub fn set_remove_unused_art_sets(&mut self, enable: bool) -> &mut Self {
        self.remove_unused_art_sets = enable;
        self
    }

    /// Allow the optimizer to remove unused variants from art sets in Portrait Settings files.
    ///
    /// Only use this after you have confirmed the unused variants are actually unused and not caused by a typo.
    pub fn set_remove_unused_variants(&mut self, enable: bool) -> &mut Self {
        self.remove_unused_variants = enable;
        self
    }

    /// Allow the optimizer to remove empty masks in Portrait Settings file, reducing their side.
    ///
    /// Ingame there's no difference between an empty mask and an invalid one, so it's better to remove them to reduce their size.
    pub fn set_remove_empty_masks(&mut self, enable: bool) -> &mut Self {
        self.remove_empty_masks = enable;
        self
    }
}
//-------------------------------------------------------------------------------//
//                           Trait implementations
//-------------------------------------------------------------------------------//

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
    ///
    /// NOTE: due to a consequence of the optimization, all tables are also sorted by their first key.
    ///
    /// Not yet working:
    /// - Remove files identical to Parent/Vanilla files (if is identical to vanilla, but a parent mod overwrites it, it ignores it).
    fn optimize(&mut self,
        paths_to_optimize: Option<Vec<ContainerPath>>,
        dependencies: &mut Dependencies,
        _schema: &Schema,
        options: &OptimizerOptions
    ) -> Result<HashSet<String>> {

        // We can only optimize if we have vanilla data available.
        if !dependencies.is_vanilla_data_loaded(false) {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate);
        }

        // Cache the pack paths for the text file checks.
        let pack_paths = self.paths().keys().map(|x| x.to_owned()).collect::<HashSet<String>>();
        let mut self_copy = self.clone();

        // List of files to optimize.
        let mut files_to_optimize = match paths_to_optimize {
            Some(paths) => self.files_by_paths_mut(&paths, false),
            None => self.files_mut().values_mut().collect::<Vec<_>>(),
        };

        // List of files to delete.
        let mut files_to_delete: HashSet<String> = HashSet::new();
        /*
        // First, do a hash pass over all the files, and mark for removal those that match by path and hash with vanilla/parent ones.
        let packedfiles_paths = self.get_ref_packed_files_all_paths().iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>();
        let mut dependencies_overwritten_files = dependencies.get_most_relevant_files_by_paths(&packedfiles_paths);
        files_to_delete.append(&mut dependencies_overwritten_files.iter_mut().filter_map(|dep_packed_file| {
            if let Some(packed_file) = self.get_ref_mut_packed_file_by_path(dep_packed_file.get_path()) {
                if let Ok(local_hash) = packed_file.get_hash_from_data() {
                    if let Ok(dependency_hash) = dep_packed_file.get_hash_from_data() {
                        if local_hash == dependency_hash {
                            Some(packed_file.get_path().to_vec())
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        }).collect());
        */

        //let mut extra_data = DecodeableExtraData::default();
        //extra_data.set_schema(Some(schema));
        //let extra_data = Some(extra_data);

        // Then, do a second pass, this time over the decodeable files that we can optimize.
        files_to_delete.extend(files_to_optimize.iter_mut().filter_map(|rfile| {

            // Only check it if it's not already marked for deletion.
            let path = rfile.path_in_container_raw().to_owned();
            if !files_to_delete.contains(&path) {

                match rfile.file_type() {
                    FileType::DB => {

                        // Unless we specifically wanted to, ignore the same-name-as-vanilla-or-parent files,
                        // as those are probably intended to overwrite vanilla files, not to be optimized.
                        if options.optimize_datacored_tables || !dependencies.file_exists(&path, true, true, true) {
                            if let Ok(RFileDecoded::DB(db)) = rfile.decoded_mut() {
                                if db.optimize(dependencies, Some(&mut self_copy), options) {
                                    return Some(path);
                                }
                            }
                        }
                    }

                    FileType::Loc => {

                        // Same as with tables, don't optimize them if they're overwriting.
                        if options.optimize_datacored_tables || !dependencies.file_exists(&path, true, true, true) {
                            if let Ok(RFileDecoded::Loc(loc)) = rfile.decoded_mut() {
                                if loc.optimize(dependencies, Some(&mut self_copy), options) {
                                    return Some(path);
                                }
                            }
                        }
                    }

                    FileType::Text => {

                        // agf and model_statistics are debug files outputed by bob in older games.
                        if path.ends_with(".agf") || path.ends_with(".model_statistics") {
                            if let Ok(Some(RFileDecoded::Text(_))) = rfile.decode(&None, false, true) {
                                return Some(path);
                            }
                        }

                        else if !path.is_empty() && (
                                path.starts_with("prefabs/") ||
                                path.starts_with("terrain/battles/") ||
                                path.starts_with("terrain/tiles/battle/")
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
                                return Some(path);
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
        Ok(files_to_delete)
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
    fn optimize(&mut self, dependencies: &mut Dependencies, _container: Option<&mut Pack>, _options: &OptimizerOptions) -> bool {

        // Get a manipulable copy of all the entries, so we can optimize it.
        let mut entries = self.data().to_vec();

        match dependencies.db_data(self.table_name(), true, true) {
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
                    !vanilla_table.contains(&serde_json::to_string(&entry_json).unwrap()) && entry != &new_row
                });

                // Dedupper. This is slower than a normal dedup, but it doesn't reorder rows.
                let mut dummy_set = HashSet::new();
                entries.retain(|x| dummy_set.insert(x.clone()));

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
    fn optimize(&mut self, dependencies: &mut Dependencies, _container: Option<&mut Pack>, _options: &OptimizerOptions) -> bool {

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
                    if entry == &new_row {
                        return false;
                    }

                    match vanilla_table.get(&*entry[0].data_to_string()) {
                        Some(vanilla_value) => &*entry[1].data_to_string() != vanilla_value,
                        None => true
                    }
                });

                // Dedupper. This is slower than a normal dedup, but it doesn't reorder rows.
                let mut dummy_set = HashSet::new();
                entries.retain(|x| dummy_set.insert(x.clone()));

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
                if options.remove_empty_masks {
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

                if options.remove_unused_variants {
                    variant_filenames.contains(variant.filename())
                } else {
                    true
                }
            });

            if options.remove_unused_art_sets {
                art_set_ids.contains(entry.id())
            } else {
                true
            }
        });

        self.set_entries(entries);
        self.entries().is_empty()
    }
}
