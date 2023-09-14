//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters, Setters};
use rpfm_lib::files::{Container, FileType, RFileDecoded};
use serde::{Serialize as SerdeSerialize, Serializer};
use serde_derive::{Serialize, Deserialize};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use rpfm_lib::error::Result;
use rpfm_lib::files::{loc::Loc, pack::Pack, table::*};
use rpfm_lib::schema::*;

pub const TRANSLATED_FILE_NAME: &str = "!!!!!!translated_locs.loc";
pub const TRANSLATED_PATH: &str = "text/!!!!!!translated_locs.loc";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Clone, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct PackTranslation {

    /// Language used for the translations.
    language: String,

    /// The name of the pack these translations were created for.
    pack_name: String,

    /// The translations themselfs.
    #[serde(serialize_with = "ordered_map_translations")]
    translations: HashMap<String, Translation>,
}

#[derive(Debug, Clone, Default, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct Translation {

    /// Loc key of the translated string.
    key: String,

    /// Value of the string to translate, in the base language (usually english).
    value_original: String,

    /// Translated value.
    value_translated: String,

    /// Flag to check if the translation needs to be revised due to the original value changing.
    needs_retranslation: bool,

    /// Flag to mark a translation as removed from the original Pack.
    removed: bool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackTranslation {

    pub fn new(path: &Path, pack: &Pack, game_key: &str, language: &str) -> Result<Self> {
        let mut translations = Self::load(path, pack, game_key, language).unwrap_or_else(|_| {
            let mut tr = Self::default();
            tr.language = language.to_owned();
            tr.pack_name = pack.disk_file_name();
            tr
        });

        // Once we got the previous translation loaded, get the files to translate from the Pack, updating our translation.
        let mut locs = pack.files_by_type(&[FileType::Loc]);

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
            if keys_found.get(&row[0].data_to_string()).is_some() {
                rows_to_delete.push(index);
            } else {
                keys_found.insert(row[0].data_to_string());
            }
        }

        rows_to_delete.reverse();
        for row in &rows_to_delete {
            merged_loc.data_mut().remove(*row);
        }

        // Once we have the clean list of loc entries we have in our Pack, we need to update the translation with it.
        // First we do a pass to mark all removed translations as such. This is separated from the rest because this pass is way slower than the rest.
        for (tr_key, tr) in translations.translations_mut() {
            let mut found = false;
            for row in merged_loc.data().iter() {
                let key = row[0].data_to_string();
                if tr_key == &key {
                    found = true;
                    break;
                }
            }

            tr.removed = !found;
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

        Ok(translations)
    }

    /// This function applies a [PackTranslation] to a Pack.
    pub fn apply(&self, pack: &mut Pack) -> Result<()> {
        todo!()
    }

    /// This function loads a [PackTranslation] to memory from a provided `.json` file.
    ///
    /// If there's no json file, it tries to load from pre-translated files inside the open Pack.
    pub fn load(path: &Path, pack: &Pack, game_key: &str, language: &str) -> Result<Self> {
        let path = path.join(format!("{}/{}/{}.json", game_key, pack.disk_file_name(), language));
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

    pub fn from_table(&mut self, table: &Table) -> Result<()> {
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

    pub fn to_table(&self) -> Result<Table> {
        let definition = Self::definition();
        let mut table = Table::new(&definition, None, "");

        let data = self.translations()
            .iter()
            .map(|(_, tr)| {
            let mut row = Vec::with_capacity(5);
            row.push(DecodedData::StringU8(tr.key().to_owned()));
            row.push(DecodedData::Boolean(*tr.needs_retranslation()));
            row.push(DecodedData::Boolean(*tr.removed()));
            row.push(DecodedData::StringU8(tr.value_original().to_owned()));
            row.push(DecodedData::StringU8(tr.value_translated().to_owned()));
            row
        }).collect::<Vec<_>>();

        table.set_data(&data)?;
        Ok(table)
    }
}

/// Special serializer function to sort the translations HashMap before serializing.
fn ordered_map_translations<S>(value: &HashMap<String, Translation>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer).map_err(From::from)
}
