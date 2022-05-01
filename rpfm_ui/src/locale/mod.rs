//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the localisation of rpfm_ui.

This module contains all the code needed to initialize/localize the entire UI, or at least the strings
on this program (the ones from the rpfm_lib/error are another story.
!*/

use qt_core::QString;

use cpp_core::CppBox;

use fluent_bundle::{FluentResource, FluentBundle};
use unic_langid::{langid, LanguageIdentifier, subtags::Language};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::str::FromStr;

use rpfm_common::utils::*;
use rpfm_error::{Error, ErrorKind, Result};

use crate::ASSETS_PATH;
use crate::LOCALE;
use crate::LOCALE_FALLBACK;

/// Name of the folder containing all the schemas.
const LOCALE_FOLDER: &str = "locale";

/// Replace sequence used to insert data into the translations.
const REPLACE_SEQUENCE: &str = "{}";

/// Include by default the english localisation, to avoid problems with idiots deleting files.
const FALLBACK_LOCALE: &str = include_str!("../../../locale/English_en.ftl");

/// This struct contains a localisation use in RPFM.
#[derive(Clone)]
pub struct Locale(Arc<RwLock<FluentBundle<FluentResource>>>);

/// Wacky fix for the "You cannot put a pointer in a static" problem.
unsafe impl Sync for Locale {}

/// Implementation of `Locale`.
impl Locale {

    /// This function initializes the localisation for the provided language, if exists.
    pub fn initialize(file_name: &str) -> Result<Self> {

        // Get the list of available translations from the locale folder, and load the requested one, if found.
        let lang_info = file_name.split('_').collect::<Vec<&str>>();
        if lang_info.len() == 2 {
            let lang_id = lang_info[1];
            let locales = Self::get_available_locales()?;
            let selected_locale = locales.iter().map(|x| x.1.clone()).find(|x| x.language == lang_id).ok_or_else(|| Error::from(ErrorKind::FluentResourceLoadingError))?;
            let locale = format!("{}/{}/{}.ftl", ASSETS_PATH.to_string_lossy(), LOCALE_FOLDER, file_name);

            // If found, load the entire file to a string.
            let mut file = File::open(&locale)?;
            let mut ftl_string = String::new();
            file.read_to_string(&mut ftl_string)?;

            // Then to a resource and a bundle.
            let resource = FluentResource::try_new(ftl_string)?;
            let mut bundle = FluentBundle::new([selected_locale].to_vec());
            bundle.add_resource(resource)?;

            // If nothing failed, return the new translation.
            Ok(Self(Arc::new(RwLock::new(bundle))))
        }

        else {
            Err(ErrorKind::InvalidLocalisationFileName(file_name.to_string()).into())
        }
    }

   /// This function initializes the fallback localisation included in the binary.
    pub fn initialize_fallback() -> Result<Self> {
        let resource = FluentResource::try_new(FALLBACK_LOCALE.to_owned())?;
        let mut bundle = FluentBundle::new(vec![langid!["en"]]);
        bundle.add_resource(resource)?;
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
    pub fn get_available_locales() -> Result<Vec<(String, LanguageIdentifier)>> {
        let mut languages = vec![];
        for file in get_files_from_subdir(&ASSETS_PATH.to_path_buf().join(Path::new("locale")), false)? {
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
    fn tr(key: &str) -> String {
        let mut _errors = vec![];
        match LOCALE.get().get_message(key) {
            Some(message) => match message.value() {
                Some(pattern) => LOCALE.get().format_pattern(pattern, None, &mut _errors).to_string(),
                None => Self::tr_fallback(key),
            },
            None => Self::tr_fallback(key),
        }
    }

    /// This function returns the translation for the key provided in the english language, or a... warning.
    fn tr_fallback(key: &str) -> String {
        let mut _errors = vec![];
        match LOCALE_FALLBACK.get().get_message(key) {
            Some(message) => match message.value() {
                Some(pattern) => LOCALE_FALLBACK.get().format_pattern(pattern, None, &mut _errors).to_string(),
                None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
            },
            None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
        }
    }

    /// This function returns a read-only guard to the provided `Locale`.
    pub fn get(&self) -> RwLockReadGuard<FluentBundle<FluentResource>> {
        self.0.read().unwrap()
    }
}

/// This function returns the translation as a `String` for the key provided in the current language.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn tr(key: &str) -> String {
    Locale::tr(key)
}

/// This function returns the translation as a `String` for the key provided in the current language,
/// replacing certain parts of the translation with the replacements provided.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
#[allow(unused)]
pub fn tre(key: &str, replacements: &[&str]) -> String {
    let mut translation = Locale::tr(key);
    replacements.iter().for_each(|x| translation = translation.replacen(REPLACE_SEQUENCE, x, 1));
    translation
}

/// This function returns the translation as a `QString` for the key provided in the current language.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn qtr(key: &str) -> CppBox<QString> {
    QString::from_std_str(Locale::tr(key))
}

/// This function returns the translation as a `QString` for the key provided in the current language,
/// replacing certain parts of the translation with the replacements provided.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn qtre(key: &str, replacements: &[&str]) -> CppBox<QString> {
    let mut translation = Locale::tr(key);
    replacements.iter().for_each(|x| translation = translation.replacen(REPLACE_SEQUENCE, x, 1));
    QString::from_std_str(translation)
}
