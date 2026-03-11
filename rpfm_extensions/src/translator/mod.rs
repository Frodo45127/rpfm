//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Mod translation and localization support.
//!
//! This module provides tools for managing translations of mod content, making it
//! easier to localize mods for different languages. It tracks translation status,
//! detects changes in source text, and supports auto-translation from vanilla data.
//!
//! # Overview
//!
//! The translation system works by:
//!
//! 1. Extracting all translatable strings from a pack's Loc files
//! 2. Storing translations in a separate JSON file alongside the pack
//! 3. Tracking which translations need updating when source text changes
//! 4. Auto-translating from vanilla localisation data where possible
//!
//! # Translation Files
//!
//! Translations are stored in separate JSON files. Each file contains all source
//! strings and their translations, along with metadata about translation status.
//!
//! # Auto-Translation
//!
//! The system can automatically translate strings that exist in the game's vanilla
//! localisation files. This is useful for mods that reference vanilla content or
//! use similar terminology.
//!
//! # Workflow
//!
//! 1. Create a [`PackTranslation`] from a pack
//! 2. Export to a translation file for external editing
//! 3. Import completed translations
//! 4. Generate the final translated Loc file for the pack
//!
//! # Output
//!
//! Translated strings are output to a Loc file that overrides the original mod's
//! entries. The filename depends on the game:
//!
//! - **Warhammer 1 and newer** (except Thrones): `!!!!!!translated_locs.loc` - loads
//!   first due to its naming, allowing translations to override the original entries
//! - **Thrones of Britannia and older games**: `localisation.loc`
//!
//! # Example
//!
//! ```ignore
//! use rpfm_extensions::translator::PackTranslation;
//!
//! // Create translation from pack
//! let mut translation = PackTranslation::new(
//!     &[translations_path],
//!     &pack,
//!     "warhammer_3",
//!     "es",  // Spanish
//!     &dependencies,
//!     &english_base,
//!     &local_fixes,
//! )?;
//!
//! // Save translation file
//! translation.save(&output_path)?;
//!
//! // Generate translated Loc file for the pack
//! let loc_file = translation.generate_loc()?;
//! ```

use getset::{Getters, MutGetters, Setters};
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Serialize as SerdeSerialize, Serializer};
use serde_derive::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use rpfm_lib::error::{Result, RLibError};
use rpfm_lib::files::{Container, FileType, loc::Loc, pack::Pack, RFile, RFileDecoded, table::{DecodedData, local::TableInMemory, Table}};
use rpfm_lib::schema::*;

use crate::dependencies::Dependencies;

/// Filename for the generated translated Loc file.
///
/// The leading exclamation marks ensure this file loads before other Loc files,
/// allowing translations to override the original mod's strings.
pub const TRANSLATED_FILE_NAME: &str = "!!!!!!translated_locs.loc";

/// Full path for the translated Loc file within a pack.
pub const TRANSLATED_PATH: &str = "text/!!!!!!translated_locs.loc";

/// Legacy path for translated Loc files (for backwards compatibility).
pub const TRANSLATED_PATH_OLD: &str = "text/localisation.loc";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Translation data for an entire pack.
///
/// Contains all translatable strings from a pack along with their translations
/// and metadata about translation status.
///
/// # Persistence
///
/// This struct is serialized to JSON files for storage and can be loaded back
/// when continuing translation work.
///
/// # Parent Translations
///
/// When a pack has dependencies, translations from parent mods are also loaded
/// and used for auto-translation, ensuring consistent terminology across
/// dependent mods.
#[derive(Debug, Clone, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PackTranslation {

    /// Target language code for translations (e.g., "es", "de", "fr").
    language: String,

    /// Name of the pack these translations belong to.
    pack_name: String,

    /// Map of Loc keys to their translation data.
    ///
    /// Keys are the original Loc entry keys from the pack.
    #[serde(serialize_with = "ordered_map_translations")]
    translations: HashMap<String, Translation>,
}

/// Translation entry for a single localizable string.
///
/// Tracks both the original and translated text, along with status flags
/// indicating whether the translation is up-to-date.
#[derive(Debug, Clone, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Translation {

    /// The Loc key identifying this string.
    key: String,

    /// Original text in the base language (typically English).
    ///
    /// This is used to detect when the source text changes, requiring
    /// re-translation.
    value_original: String,

    /// Translated text in the target language.
    ///
    /// May be empty if not yet translated.
    value_translated: String,

    /// Whether this translation needs review.
    ///
    /// Set to `true` when the original text changes after translation,
    /// indicating the translation may be outdated.
    needs_retranslation: bool,

    /// Whether this string has been removed from the source pack.
    ///
    /// Translations for removed strings are kept for reference but marked
    /// as removed. If the string reappears, it will be flagged for re-translation.
    removed: bool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackTranslation {

    pub fn new(paths: &[PathBuf], pack: &Pack, game_key: &str, language: &str, dependencies: &Dependencies, base_english: &HashMap<String, String>, base_local_fixes: &HashMap<String, String>) -> Result<Self> {
        let mut translations = Self::load(paths, &pack.disk_file_name(), game_key, language).unwrap_or_else(|_| {
            let mut tr = Self::default();
            tr.language = language.to_owned();
            tr.pack_name = pack.disk_file_name();
            tr
        });

        // If the pack has dependencies, we have to try to load their translations too, then patch the live dependencies with them.
        // Otherwise, we'll have a situation where data is compared and imported from the wrong language.
        let mut parent_tr = vec![];
        for (_, pack_name) in pack.dependencies() {
            if let Ok(ptr) = Self::load(paths, pack_name, game_key, language) {
                parent_tr.push(ptr);
            }
        }

        // Once we got the previous translation loaded, get the files to translate from the Pack, updating our translation.
        let mut locs = pack.files_by_type(&[FileType::Loc]);
        let merged_loc = Self::sort_and_merge_locs_for_translation(&mut locs)?;
        let merged_loc_data = merged_loc.data();
        let merged_loc_hash = merged_loc_data
            .par_iter()
            .map(|x| (x[0].data_to_string(), x[1].data_to_string()))
            .collect::<HashMap<_,_>>();

        // Once we have the clean list of loc entries we have in our Pack, we need to update the translation with it.
        // First we do a pass to mark all removed translations as such. This is separated from the rest because this pass is way slower than the rest.
        for (tr_key, tr) in translations.translations_mut() {
            let was_removed = tr.removed;
            tr.removed = !merged_loc_hash.contains_key(&**tr_key);

            // If the line has been removed, unmark it for translation.
            // If the line has been re-added, only flag for retranslation if the original value changed or there's no translation yet.
            if tr.removed {
                tr.needs_retranslation = false;
            } else if was_removed {
                if let Some(current_value) = merged_loc_hash.get(&**tr_key) {
                    tr.needs_retranslation = tr.value_translated.is_empty() || *current_value != tr.value_original;
                }
            }
        }

        // Next, we update the translations data with the loc data of the merged loc.
        for row in merged_loc.data().iter() {
            let key = row[0].data_to_string();
            let value = row[1].data_to_string();

            match translations.translations.get_mut(&*key) {
                Some(tr) => {
                    if value != tr.value_original {
                        tr.value_original = value.to_string();
                        tr.needs_retranslation = true;
                    }
                },

                None => {
                    let tr = Translation {
                        key: key.to_string(),
                        value_original: value.to_string(),
                        value_translated: String::new(),
                        needs_retranslation: true,
                        removed: false,
                    };

                    translations.translations.insert(key.to_string(), tr);
                }
            }
        }

        // Lastly, we do an auto-translation pass. We have two copies of base local: one normal and one patched with parent translations.
        // This is needed because the base localisation data doesn't have the translation data for parent mods included.
        let mut base_local_tr = dependencies.localisation_data().clone();
        for ptr in parent_tr {
            for (key, val) in ptr.translations() {
                if !*val.needs_retranslation() && !val.value_translated().is_empty() {
                    if let Some(ptr_val) = base_local_tr.get_mut(key) {
                        *ptr_val = val.value_translated().to_string();
                    }
                }
            }
        }

        let tr_copy = translations.translations().clone();
        translations.translations_mut().par_iter_mut().for_each(|(tr_key, tr)| {
            if !tr.removed {

                // Mark empty lines as translated.
                if tr.value_original().trim().is_empty() && tr.value_translated().trim().is_empty() {
                    tr.value_translated = tr.value_original.to_owned();
                    tr.needs_retranslation = false;
                }

                // If the value is unchanged from english, just copy the vanilla translation.
                //
                // NOTE: This is really a patch for packs not using optimizing pass, because the optimizer actually removes these entries.
                else if let Some(vanilla_data) = base_english.get(tr_key) {
                    if tr.value_original() == vanilla_data {
                        if let Some(vanilla_data) = base_local_fixes.get(tr_key) {
                            tr.value_translated = vanilla_data.to_owned();
                            tr.needs_retranslation = false;
                        } else if let Some(vanilla_data) = base_local_tr.get(tr_key) {
                            tr.value_translated = vanilla_data.to_owned();
                            tr.needs_retranslation = false;
                        }
                    }
                }

                // If the value is equal to another value in the english translation (but with a different key), we may be able to reuse it.
                //
                // Note that this is prone to give wrong translations as it doesn't have any context, so we only do it for lines that are not yet translated.
                else if tr.value_translated().trim().is_empty() || *tr.needs_retranslation() {
                    if let Some((key, _)) = base_english.iter().find(|(_, value)| *value == tr.value_original()) {
                        if let Some(value_tr) = base_local_fixes.get(key) {
                            tr.value_translated = value_tr.to_owned();
                            tr.needs_retranslation = false;
                        } else if let Some(value_tr) = base_local_tr.get(key) {
                            tr.value_translated = value_tr.to_owned();
                            tr.needs_retranslation = false;
                        }
                    } else if let Some((_, value_tr)) = tr_copy.iter()
                        .find(|(_, tr_copy)| *tr_copy.value_original() == *tr.value_original() && !*tr_copy.needs_retranslation() && *tr.needs_retranslation()) {
                        tr.value_translated = value_tr.value_translated().to_owned();
                        tr.needs_retranslation = false;
                    }
                }
            }
        });

        Ok(translations)
    }

    // TODO: Move this to the normal merge functions.
    pub fn sort_and_merge_locs_for_translation(locs: &mut [&RFile]) -> Result<Loc> {

        // We need them in a specific order so the file priority removes unused loc entries from the translation.
        locs.sort_by(|a, b| a.path_in_container_raw().cmp(b.path_in_container_raw()));
        let locs = locs.iter()
            .filter(|file| {
                if let Some(name) = file.file_name() {
                    !name.is_empty() && name != TRANSLATED_FILE_NAME
                } else {
                    false
                }
            })
            .filter_map(|file| if let Ok(RFileDecoded::Loc(loc)) = file.decoded() { Some(loc) } else { None })
            .collect::<Vec<_>>();

        // Once we merge all the locs in the correct order, remove duplicated keys except the first one.
        let mut merged_loc = Loc::merge(&locs)?;
        let mut keys_found = HashSet::new();
        let mut rows_to_delete = vec![];
        for (index, row) in merged_loc.data().iter().enumerate() {
            if keys_found.contains(&row[0].data_to_string()) {
                rows_to_delete.push(index);
            } else {
                keys_found.insert(row[0].data_to_string());
            }
        }

        rows_to_delete.reverse();
        for row in &rows_to_delete {
            merged_loc.data_mut().remove(*row);
        }

        Ok(merged_loc)
    }

    /// This function applies a [PackTranslation] to a Pack.
    pub fn apply(&self, _pack: &mut Pack) -> Result<()> {
        todo!()
    }

    /// This function loads a [PackTranslation] to memory from either a local json file, or a remote one.
    pub fn load(paths: &[PathBuf], pack_name: &str, game_key: &str, language: &str) -> Result<Self> {
        for path in paths {
            match Self::load_json(path, pack_name, game_key, language) {
                Ok(mut tr) => return {
                    for trad in tr.translations_mut() {
                        trad.1.value_translated = trad.1.value_translated.replace("\n||\n", "||");
                        trad.1.value_translated = trad.1.value_translated.replace("\r", "\\\\r");
                        trad.1.value_translated = trad.1.value_translated.replace("\n", "\\\\n");
                        trad.1.value_translated = trad.1.value_translated.replace("\t", "\\\\t");
                    }
                    Ok(tr)
                },
                Err(_) => continue,
            }
        }

        Err(RLibError::TranslatorCouldNotLoadTranslation)
    }

    fn load_json(path: &Path, pack_name: &str, game_key: &str, language: &str) -> Result<Self> {
        let path = path.join(format!("{game_key}/{pack_name}/{language}.json"));
        let mut file = BufReader::new(File::open(path)?);
        let mut data = Vec::with_capacity(file.get_ref().metadata()?.len() as usize);
        file.read_to_end(&mut data)?;
        serde_json::from_slice(&data).map_err(From::from)
    }

    /// This function saves a [PackTranslation] from memory to a `.json` file with the provided path.
    pub fn save(&mut self, path: &Path, game_key: &str) -> Result<()> {
        let path = path.join(format!("{}/{}/{}.json", game_key, self.pack_name, self.language));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let mut file = BufWriter::new(File::create(&path)?);
        file.write_all(serde_json::to_string_pretty(&self)?.as_bytes())?;
        Ok(())
    }

    pub fn definition() -> Definition {
        let mut definition = Definition::default();

        // We put the booleans first because they may act as a kind of filter.
        definition.fields_mut().push(Field::new("key".to_string(), FieldType::StringU8, true, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("needs_retranslation".to_string(), FieldType::Boolean, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("removed".to_string(), FieldType::Boolean, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("value_original".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));
        definition.fields_mut().push(Field::new("value_translated".to_string(), FieldType::StringU8, false, None, false, None, None, None, String::new(), -1, 0, BTreeMap::new(), None));

        definition
    }

    pub fn from_table(&mut self, table: &TableInMemory) -> Result<()> {
        self.translations_mut().clear();

        for row in table.data().iter() {
            let mut tr = Translation::default();

            if let DecodedData::StringU8(ref data) = row[0] {
                tr.set_key(data.to_owned());
            }

            if let DecodedData::Boolean(data) = row[1] {
                tr.set_needs_retranslation(data);
            }

            if let DecodedData::Boolean(data) = row[2] {
                tr.set_removed(data);
            }

            if let DecodedData::StringU8(ref data) = row[3] {
                tr.set_value_original(data.to_owned());
            }

            if let DecodedData::StringU8(ref data) = row[4] {
                tr.set_value_translated(data.to_owned());
            }

            self.translations_mut().insert(tr.key.to_owned(), tr);
        }

        Ok(())
    }

    pub fn to_table(&self) -> Result<TableInMemory> {
        let definition = Self::definition();
        let mut table = TableInMemory::new(&definition, None, "");

        // Due to bugs in the table filters, we pre-sort the data by putting stuff that needs to be retranslated at the start.
        let data = self.translations()
            .iter()
            .sorted_by(|(_, tr1), (_, tr2)| Ord::cmp(tr1.key(), tr2.key()))
            .sorted_by(|(_, tr1), (_, tr2)| Ord::cmp(tr2.needs_retranslation(), tr1.needs_retranslation()))
            .map(|(_, tr)| vec![
                DecodedData::StringU8(tr.key().to_owned()),
                DecodedData::Boolean(*tr.needs_retranslation()),
                DecodedData::Boolean(*tr.removed()),
                DecodedData::StringU8(tr.value_original().to_owned()),
                DecodedData::StringU8(tr.value_translated().to_owned()),
            ]).collect::<Vec<_>>();

        table.set_data(&data)?;
        Ok(table)
    }
}

/// Special serializer function to sort the translations HashMap before serializing.
fn ordered_map_translations<S>(value: &HashMap<String, Translation>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
