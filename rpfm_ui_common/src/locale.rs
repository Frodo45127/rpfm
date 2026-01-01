//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use anyhow::{anyhow, Result};
use fluent_bundle::{FluentResource, FluentBundle};
use unic_langid::{langid, LanguageIdentifier, subtags::Language};

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock, RwLockReadGuard};
use std::str::FromStr;

use rpfm_lib::utils::*;

/// Name of the folder containing all the translation files.
const LOCALE_FOLDER: &str = "locale";

/// Replace sequence used to insert data into the translations.
const REPLACE_SEQUENCE: &str = "{}";

/// Include by default the english localisation, to avoid problems with idiots deleting files.
pub static FALLBACK_LOCALE: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(include_str!("../../locale/English_en.ftl").to_owned())));

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains a localisation use in a program using this lib.
#[derive(Clone)]
pub struct Locale(Arc<RwLock<FluentBundle<FluentResource>>>);

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Wacky fix for the "You cannot put a pointer in a static" problem.
unsafe impl Sync for Locale {}
unsafe impl Send for Locale {}

impl Locale {

    /// This function initializes the localisation for the provided language, if exists.
    pub fn initialize(file_name: &str, config_path: &str) -> Result<Self> {

        // Get the list of available translations from the locale folder, and load the requested one, if found.
        let lang_info = file_name.split('_').collect::<Vec<&str>>();
        if lang_info.len() == 2 {
            let lang_id = lang_info[1];
            let locales = Self::get_available_locales(config_path)?;
            let selected_locale = locales.iter()
                .map(|x| x.1.clone())
                .find(|x| x.language == lang_id)
                .ok_or_else(|| anyhow!("Error while trying to load a fluent resource."))?;
            let locale = format!("{}/{}/{}.ftl", config_path, LOCALE_FOLDER, file_name);

            // If found, load the entire file to a string.
            let mut file = File::open(locale)?;
            let mut ftl_string = String::new();
            file.read_to_string(&mut ftl_string)?;

            // Then to a resource and a bundle.
            let resource = FluentResource::try_new(ftl_string).map_err(|_| anyhow!("Failed to initialize fluent main resource."))?;
            let mut bundle = FluentBundle::new([selected_locale].to_vec());
            bundle.add_resource(resource).map_err(|_| anyhow!("Failed to add fluent main resource."))?;

            // If nothing failed, return the new translation.
            Ok(Self(Arc::new(RwLock::new(bundle))))
        }

        else {
            Err(anyhow!("The name '{}' is not a valid localisation file name. It has to have one and only one '_' somewhere and an identifier (en, fr,…) after that.", file_name))
        }
    }

    /// This function initializes the fallback localisation included in the binary.
    pub fn initialize_fallback() -> Result<Self> {
        let resource = FluentResource::try_new(FALLBACK_LOCALE.read().unwrap().to_owned()).map_err(|_| anyhow!("Failed to initialize fluent fallback resource."))?;
        let mut bundle = FluentBundle::new(vec![langid!["en"]]);
        bundle.add_resource(resource).map_err(|_| anyhow!("Failed to add fluent fallback resource."))?;
        Ok(Self(Arc::new(RwLock::new(bundle))))
    }

    /// This function initializes an empty localisation, just in case some idiot deletes the english translation and fails to load it.
    pub fn initialize_empty() -> Self {
        let resource = FluentResource::try_new(String::new()).unwrap();
        let mut bundle = FluentBundle::new(vec![langid!["en"]]);
        bundle.add_resource(resource).unwrap();
        Self(Arc::new(RwLock::new(bundle)))
    }

    /// This function returns a list of all the languages we have translation files for in the `("English", "en")` form.
    pub fn get_available_locales(config_path: &str) -> Result<Vec<(String, LanguageIdentifier)>> {
        let mut languages = vec![];
        for file in files_from_subdir(&PathBuf::from(config_path).join(Path::new(LOCALE_FOLDER)), false)? {
            let language = file.file_stem().unwrap().to_string_lossy().to_string();
            let lang_info = language.split('_').collect::<Vec<&str>>();
            if lang_info.len() == 2 {
                let lang_id = Language::from_str(lang_info[1])?;
                let language_id = LanguageIdentifier::from_parts(lang_id, None, None, &[]);
                languages.push((lang_info[0].to_owned(), language_id));
            }
        }
        Ok(languages)
    }

    /// This function returns the translation for the key provided in the current language.
    ///
    /// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
    pub fn tr(&self, fallback: &Self, key: &str) -> String {
        let mut _errors = vec![];
        let locale = self.get();
        match locale.get_message(key) {
            Some(message) => match message.value() {
                Some(pattern) => locale.format_pattern(pattern, None, &mut _errors).to_string(),
                None => fallback.tr_fallback(key),
            },
            None => fallback.tr_fallback(key),
        }
    }

    /// This function returns the translation for the key provided in the english language, or a... warning.
    fn tr_fallback(&self, key: &str) -> String {
        let mut _errors = vec![];
        let locale = self.get();
        match locale.get_message(key) {
            Some(message) => match message.value() {
                Some(pattern) => locale.format_pattern(pattern, None, &mut _errors).to_string(),
                None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
            },
            None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
        }
    }

    /// This function returns the translation as a `String` for the key provided in the current language,
    /// replacing certain parts of the translation with the replacements provided.
    ///
    /// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
    pub fn tre(&self, fallback: &Self, key: &str, replacements: &[&str]) -> String {
        let mut translation = self.tr(fallback, key);
        replacements.iter().for_each(|x| translation = translation.replacen(REPLACE_SEQUENCE, x, 1));
        translation
    }

    /// This function returns a read-only guard to the provided `Locale`.
    pub fn get(&'_ self) -> RwLockReadGuard<'_, FluentBundle<FluentResource>> {
        self.0.read().unwrap()
    }
}
