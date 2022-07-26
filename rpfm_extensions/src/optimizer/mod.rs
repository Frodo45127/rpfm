//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the [Optimizable] and [OptimizableContainer] trait.

use rayon::prelude::*;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use rpfm_lib::error::Result;
use rpfm_lib::files::{Container, db::DB, loc::Loc, pack::Pack, RFileDecoded, table::DecodedData};
use rpfm_lib::games::GameInfo;
use rpfm_lib::schema::patch::SchemaPatches;

use crate::dependencies::Dependencies;

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Optimizable`, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Optimizable {

    /// This function optimizes the provided struct to reduce its size and improve compatibility.
    ///
    /// It returns if the struct has been left in an state where it can be safetly deleted.
    fn optimize(&mut self, game_info: &GameInfo, game_path: &Path, dependencies: &mut Dependencies, schema_patches: Option<&SchemaPatches>) -> bool;
}

/// This trait marks a [Container](rpfm_lib::files::Container) as an `Optimizable` container, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait OptimizableContainer: Container {

    /// This function optimizes the provided [Container](rpfm_lib::files::Container) to reduce its size and improve compatibility.
    ///
    /// It returns the list of files that has been safetly deleted during the optimization process.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> Result<Vec<String>>;
}

//-------------------------------------------------------------------------------//
//                           Trait implementations
//-------------------------------------------------------------------------------//

impl OptimizableContainer for Pack {
    fn optimize(&mut self, _dependencies: &mut Dependencies) -> Result<Vec<String>> {
        todo!()
    }
}

impl Optimizable for DB {
    fn optimize(&mut self, game_info: &GameInfo, game_path: &Path, dependencies: &mut Dependencies, schema_patches: Option<&SchemaPatches>) -> bool {
        match self.data(&None) {
            Ok(entries) => {

                // Get a manipulable copy of all the entries, so we can optimize it.
                let mut entries = entries.to_vec();
                let definition = self.definition();
                let first_key = definition.fields_processed_sorted(true).iter().position(|x| x.is_key()).unwrap_or(0);

                match dependencies.db_data(game_info, game_path, self.table_name(), true, true) {
                    Ok(mut vanilla_tables) => {

                        // First, merge all vanilla and parent db fragments into a single HashSet.
                        let vanilla_table = vanilla_tables.iter_mut()
                            .filter_map(|file| {
                                if let Ok(Some(RFileDecoded::DB(table))) = file.decode(&None, false, true) {
                                    table.data(&None).ok().map(|x| x.to_vec())
                                } else { None }
                            })
                            .flatten()
                            .map(|x| {

                                // We map all floats here to string representations of floats, so we can actually compare them reliably.
                                let json = x.iter().map(|data|
                                    if let DecodedData::F32(value) = data {
                                        DecodedData::StringU8(format!("{:.4}", value))
                                    } else {
                                        data.to_owned()
                                    }
                                ).collect::<Vec<DecodedData>>();
                                serde_json::to_string(&json).unwrap()
                            })
                            .collect::<HashSet<String>>();

                        // Remove ITM and ITNR entries.
                        let new_row = self.new_row(Some(&game_info.game_key_name()), schema_patches).iter().map(|data|
                            if let DecodedData::F32(value) = data {
                                DecodedData::StringU8(format!("{:.4}", value))
                            } else {
                                data.to_owned()
                            }
                        ).collect::<Vec<DecodedData>>();

                        entries.retain(|entry| {
                            let entry_json = entry.iter().map(|data|
                                if let DecodedData::F32(value) = data {
                                    DecodedData::StringU8(format!("{:.4}", value))
                                } else {
                                    data.to_owned()
                                }
                            ).collect::<Vec<DecodedData>>();
                            !vanilla_table.contains(&serde_json::to_string(&entry_json).unwrap()) && entry != &new_row
                        });

                        // Sort the table so it can be dedup. Sorting floats is a pain in the ass.
                        entries.par_sort_by(|a, b| {
                            let ordering = if let DecodedData::F32(x) = a[first_key] {
                                if let DecodedData::F32(y) = b[first_key] {
                                    if float_eq::float_eq!(x, y, abs <= 0.0001) {
                                        Some(Ordering::Equal)
                                    } else { None }
                                } else { None }
                            } else { None };

                            match ordering {
                                Some(ordering) => ordering,
                                None => a[first_key].data_to_string().partial_cmp(&b[first_key].data_to_string()).unwrap_or(Ordering::Equal)
                            }
                        });

                        entries.dedup();

                        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                        //
                        // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                        let _ = self.set_data(None, &entries);
                        self.data(&None).unwrap().is_empty()
                    }
                    Err(_) => false,
                }
            }

            // We don't optimize sql-backed data.
            Err(_) => false,
        }
    }
}

impl Optimizable for Loc {

    /// This function optimizes the provided [Loc](rpfm_lib::files::loc::Loc) file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of duplicated entries.
    /// - Removal of ITM (Identical To Master) entries.
    /// - Removal of ITNR (Identical To New Row) entries.
    ///
    /// It returns if the Loc is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, game_info: &GameInfo, game_path: &Path, dependencies: &mut Dependencies, _schema_patches: Option<&SchemaPatches>) -> bool {
        match self.data(&None) {
            Ok(entries) => {

                // Get a manipulable copy of all the entries, so we can optimize it.
                let mut entries = entries.to_vec();
                match dependencies.loc_data(game_info, game_path, true, true) {
                    Ok(mut vanilla_tables) => {

                        // First, merge all vanilla and parent locs into a single HashMap<key, value>. We don't care about the third column.
                        let vanilla_table = vanilla_tables.iter_mut()
                            .filter_map(|file| {
                                if let Ok(Some(RFileDecoded::Loc(table))) = file.decode(&None, false, true) {
                                    table.data(&None).ok().map(|x| x.to_vec())
                                } else { None }
                            })
                            .map(|data| data.iter()
                                .map(|data| (data[0].data_to_string().to_string(), data[1].data_to_string().to_string()))
                                .collect::<Vec<(String, String)>>())
                            .flatten()
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

                        // Sort the table so it can be dedup.
                        entries.par_sort_by(|a, b| a[0].data_to_string().partial_cmp(&b[0].data_to_string()).unwrap_or(Ordering::Equal));
                        entries.dedup();

                        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                        //
                        // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                        let _ = self.set_data(&entries);
                        self.data(&None).unwrap().is_empty()
                    }
                    Err(_) => false,
                }
            }

            // We don't optimize sql-backed data.
            Err(_) => false,
        }
    }
}
