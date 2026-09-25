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

#[cfg(test)] mod test;

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
/// # Persistence and Format Versioning
///
/// This struct is serialized to JSON files for storage. There are currently two
/// on-disk formats, picked by the [`version`](Self::version) field:
///
/// - `0` — legacy format. Per-entry `key`/`value_original`/`value_translated`/
///   `needs_retranslation`/`removed`; no `aut` and no `version` at the root.
///   The user can opt back into this format from the translator UI to share
///   translations with older tooling.
/// - `1` — current format. Adds `version` and `aut`, and uses the shorter
///   per-entry field names (`src`/`dst`/`retr`/`rem`/`aut`).
///
/// On load, files without a `version` field are treated as v0 — i.e. existing
/// translation hubs and legacy local files keep their format unless the user
/// explicitly upgrades them.
///
/// # Parent Translations
///
/// When a pack has dependencies, translations from parent mods are also loaded
/// and used for auto-translation, ensuring consistent terminology across
/// dependent mods.
#[derive(Debug, Clone, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PackTranslation {

    /// On-disk format version. See the struct-level docs for the meaning of
    /// each value. New in-memory translations default to v1; legacy files
    /// without this field are treated as v0.
    #[serde(default)]
    version: u32,

    /// Target language code for translations (e.g., "ES", "DE", "FR").
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
/// Tracks both the source and translated text, along with status flags
/// indicating whether the translation is up-to-date, was generated
/// automatically, or refers to a string no longer present in the pack.
///
/// Field names were shortened in the new format; serde aliases keep
/// backwards-compatible deserialization of the old field names. The old
/// per-entry `key` field is no longer stored — the map key in
/// [`PackTranslation::translations`] is the authoritative identifier — and
/// extra fields in legacy files are silently ignored.
#[derive(Debug, Clone, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Translation {

    /// Source text in the base language (typically English).
    ///
    /// Used to detect when the source text changes, requiring re-translation.
    #[serde(alias = "value_original")]
    src: String,

    /// Translated text in the target language.
    ///
    /// May be empty if not yet translated.
    #[serde(alias = "value_translated")]
    dst: String,

    /// Whether this translation needs review because the source text changed
    /// after the previous translation.
    #[serde(alias = "needs_retranslation")]
    retr: bool,

    /// Whether this string has been removed from the source pack.
    ///
    /// Translations for removed strings are kept for reference but marked
    /// as removed. If the string reappears, it will be flagged for re-translation.
    #[serde(alias = "removed")]
    rem: bool,

    /// Whether this translation was generated automatically (e.g. from vanilla
    /// data) and still needs manual review.
    ///
    /// Missing from the old format; defaults to `false` when loading legacy files.
    #[serde(default)]
    aut: bool,
}

/// Current on-disk format version. New translations default to this; legacy
/// files without a `version` field are still loaded as v0 and saved back as v0
/// unless the user opts in to the new format from the UI.
pub const CURRENT_VERSION: u32 = 1;

/// Wire-format counterparts used when saving as legacy v0. We can't reuse the
/// canonical structs because field names changed and the per-entry `key` field
/// is no longer stored. Building these on save lets us write the old shape
/// without polluting [`PackTranslation`] with version-specific serde glue.
#[derive(Serialize)]
struct PackTranslationV0Wire<'a> {
    language: &'a str,
    pack_name: &'a str,
    translations: BTreeMap<String, TranslationV0Wire<'a>>,
}

#[derive(Serialize)]
struct TranslationV0Wire<'a> {
    key: String,
    value_original: &'a str,
    value_translated: &'a str,
    needs_retranslation: bool,
    removed: bool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackTranslation {

    pub fn new(paths: &[PathBuf], pack: &Pack, game_key: &str, language: &str, dependencies: &Dependencies, base_english: &HashMap<String, String>, base_local_fixes: &HashMap<String, String>) -> Result<Self> {
        let mut translations = Self::load(paths, &pack.disk_file_name(), game_key, language).unwrap_or_else(|_| {
            // No existing file → new translation. `Default::default()` already gives us
            // version=1; we stamp the language and pack name on top.
            Self {
                language: language.to_owned(),
                pack_name: pack.disk_file_name(),
                ..Default::default()
            }
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
            let was_removed = tr.rem;
            tr.rem = !merged_loc_hash.contains_key(&**tr_key);

            // If the line has been removed, unmark it for translation.
            // If the line has been re-added, only flag for retranslation if the original value changed or there's no translation yet.
            if tr.rem {
                tr.retr = false;
            } else if was_removed {
                if let Some(current_value) = merged_loc_hash.get(&**tr_key) {
                    tr.retr = tr.dst.is_empty() || *current_value != tr.src;
                }
            }
        }

        // Next, we update the translations data with the loc data of the merged loc.
        for row in merged_loc.data().iter() {
            let key = row[0].data_to_string();
            let value = row[1].data_to_string();

            match translations.translations.get_mut(&*key) {
                Some(tr) => {
                    if value != tr.src {
                        tr.src = value.to_string();
                        tr.retr = true;
                        // Source changed — any prior auto-translation is no longer trustworthy.
                        tr.aut = false;
                    }
                },

                None => {
                    let tr = Translation {
                        src: value.to_string(),
                        dst: String::new(),
                        retr: true,
                        rem: false,
                        aut: false,
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
                if !*val.retr() && !val.dst().is_empty() {
                    if let Some(ptr_val) = base_local_tr.get_mut(key) {
                        *ptr_val = val.dst().to_string();
                    }
                }
            }
        }

        let tr_copy = translations.translations().clone();
        translations.translations_mut().par_iter_mut().for_each(|(tr_key, tr)| {
            if !tr.rem {

                // Fix incorrectly translated lines.
                if !tr.src().trim().is_empty() && tr.dst().trim().is_empty() && !tr.retr() {
                    tr.retr = true;
                }

                // Empty source/empty translation: trivially "translate" by copying. Not flagged
                // as auto because it requires no manual review.
                else if tr.src().trim().is_empty() && tr.dst().trim().is_empty() {
                    tr.dst = tr.src.to_owned();
                    tr.retr = false;
                    tr.aut = false;
                }

                // If the value is unchanged from english, just copy the vanilla translation.
                //
                // NOTE: This is really a patch for packs not using optimizing pass, because the optimizer actually removes these entries.
                else if let Some(vanilla_data) = base_english.get(tr_key) {
                    if tr.src() == vanilla_data {
                        if let Some(vanilla_data) = base_local_fixes.get(tr_key).filter(|v| !v.trim().is_empty()) {
                            tr.dst = vanilla_data.to_owned();
                            tr.retr = false;
                            tr.aut = true;
                        } else if let Some(vanilla_data) = base_local_tr.get(tr_key).filter(|v| !v.trim().is_empty()) {
                            tr.dst = vanilla_data.to_owned();
                            tr.retr = false;
                            tr.aut = true;
                        }
                    }
                }

                // If the value is equal to another value in the english translation (but with a different key), we may be able to reuse it.
                //
                // Note that this is prone to give wrong translations as it doesn't have any context, so we only do it for lines that are not yet translated.
                else if tr.dst().trim().is_empty() || *tr.retr() {
                    if let Some((key, _)) = base_english.iter().find(|(_, value)| *value == tr.src()) {
                        if let Some(value_tr) = base_local_fixes.get(key).filter(|v| !v.trim().is_empty()) {
                            tr.dst = value_tr.to_owned();
                            tr.retr = false;
                            tr.aut = true;
                        } else if let Some(value_tr) = base_local_tr.get(key).filter(|v| !v.trim().is_empty()) {
                            tr.dst = value_tr.to_owned();
                            tr.retr = false;
                            tr.aut = true;
                        }
                    } else if let Some((_, value_tr)) = tr_copy.iter()
                        .find(|(_, tr_copy)| *tr_copy.src() == *tr.src() && !*tr_copy.retr() && *tr.retr() && !tr_copy.dst().trim().is_empty()) {
                        tr.dst = value_tr.dst().to_owned();
                        tr.retr = false;
                        tr.aut = true;
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
    ///
    /// Files written in the old format (with `key`, `value_original`, `value_translated`,
    /// `needs_retranslation`, `removed` and without `aut`) are accepted transparently via
    /// serde aliases and field defaults.
    pub fn load(paths: &[PathBuf], pack_name: &str, game_key: &str, language: &str) -> Result<Self> {
        for path in paths {
            match Self::load_json(path, pack_name, game_key, language) {
                Ok(mut tr) => return {
                    for trad in tr.translations_mut() {
                        trad.1.dst = trad.1.dst.replace("\n||\n", "||");
                        trad.1.dst = trad.1.dst.replace("\r", "\\\\r");
                        trad.1.dst = trad.1.dst.replace("\n", "\\\\n");
                        trad.1.dst = trad.1.dst.replace("\t", "\\\\t");
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

        // Peek at the JSON to determine the version before deserialising. We need a different
        // default for "missing version field in file" (= v0, legacy) versus "freshly created
        // in-memory PackTranslation" (= v1, current). Serde defaults can only express one of
        // those, so we resolve it manually here.
        let value: serde_json::Value = serde_json::from_slice(&data)?;
        let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let mut pack_tr: Self = serde_json::from_value(value)?;
        pack_tr.version = version;
        Ok(pack_tr)
    }

    /// This function saves a [PackTranslation] from memory to a `.json` file with the provided path.
    ///
    /// The on-disk format depends on [`Self::version`]: 0 writes the legacy shape, 1 (or higher)
    /// writes the current shape. Downgrading v1 → v0 drops the per-entry `aut` flag.
    pub fn save(&mut self, path: &Path, game_key: &str) -> Result<()> {
        let path = path.join(format!("{}/{}/{}.json", game_key, self.pack_name, self.language));

        // Make sure the path exists to avoid problems with updating schemas.
        if let Some(parent_folder) = path.parent() {
            DirBuilder::new().recursive(true).create(parent_folder)?;
        }

        let json = if self.version == 0 {
            // Materialize the v0 wire shape from current data. The per-entry `aut` flag is
            // dropped silently — there's no corresponding concept in v0.
            let translations = self.translations.iter()
                .map(|(key, tr)| (key.clone(), TranslationV0Wire {
                    key: key.clone(),
                    value_original: tr.src(),
                    value_translated: tr.dst(),
                    needs_retranslation: *tr.retr(),
                    removed: *tr.rem(),
                }))
                .collect::<BTreeMap<_, _>>();

            let wire = PackTranslationV0Wire {
                language: &self.language,
                pack_name: &self.pack_name,
                translations,
            };

            serde_json::to_string_pretty(&wire)?
        } else {
            serde_json::to_string_pretty(&self)?
        };

        let mut file = BufWriter::new(File::create(&path)?);
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn definition() -> Definition {
        let mut definition = Definition::default();

        // We put the booleans first because they may act as a kind of filter.
        definition.fields_mut().push(Field { name: "key".to_string(), field_type: FieldType::StringU8, is_key: true, ..Default::default() });
        definition.fields_mut().push(Field { name: "retr".to_string(), field_type: FieldType::Boolean, ..Default::default() });
        definition.fields_mut().push(Field { name: "rem".to_string(), field_type: FieldType::Boolean, ..Default::default() });
        definition.fields_mut().push(Field { name: "aut".to_string(), field_type: FieldType::Boolean, ..Default::default() });
        definition.fields_mut().push(Field { name: "src".to_string(), field_type: FieldType::StringU8, ..Default::default() });
        definition.fields_mut().push(Field { name: "dst".to_string(), field_type: FieldType::StringU8, ..Default::default() });

        definition
    }

    pub fn from_table(&mut self, table: &TableInMemory) -> Result<()> {
        self.translations_mut().clear();

        for row in table.data().iter() {
            let mut tr = Translation::default();
            let mut key = String::new();

            if let DecodedData::StringU8(ref data) = row[0] {
                key = data.to_owned();
            }

            if let DecodedData::Boolean(data) = row[1] {
                tr.set_retr(data);
            }

            if let DecodedData::Boolean(data) = row[2] {
                tr.set_rem(data);
            }

            if let DecodedData::Boolean(data) = row[3] {
                tr.set_aut(data);
            }

            if let DecodedData::StringU8(ref data) = row[4] {
                tr.set_src(data.to_owned());
            }

            if let DecodedData::StringU8(ref data) = row[5] {
                tr.set_dst(data.to_owned());
            }

            self.translations_mut().insert(key, tr);
        }

        Ok(())
    }

    pub fn to_table(&self) -> Result<TableInMemory> {
        let definition = Self::definition();
        let mut table = TableInMemory::new(&definition, None, "");

        // Due to bugs in the table filters, we pre-sort the data by putting stuff that needs to be retranslated at the start.
        let data = self.translations()
            .iter()
            .sorted_by(|(k1, _), (k2, _)| Ord::cmp(k1, k2))
            .sorted_by(|(_, tr1), (_, tr2)| Ord::cmp(tr2.retr(), tr1.retr()))
            .map(|(key, tr)| vec![
                DecodedData::StringU8(key.to_owned()),
                DecodedData::Boolean(*tr.retr()),
                DecodedData::Boolean(*tr.rem()),
                DecodedData::Boolean(*tr.aut()),
                DecodedData::StringU8(tr.src().to_owned()),
                DecodedData::StringU8(tr.dst().to_owned()),
            ]).collect::<Vec<_>>();

        table.set_data(&data)?;
        Ok(table)
    }
}

impl Default for PackTranslation {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            language: String::new(),
            pack_name: String::new(),
            translations: HashMap::new(),
        }
    }
}

/// Special serializer function to sort the translations HashMap before serializing.
fn ordered_map_translations<S>(value: &HashMap<String, Translation>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
