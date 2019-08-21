//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::{langid, LanguageIdentifier};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use rpfm_error::{Error, ErrorKind, Result};
use rpfm_lib::common::get_files_from_subdir;

use crate::LOCALE;
use crate::LOCALE_FALLBACK;

/// Name of the folder containing all the schemas.
const LOCALE_FOLDER: &str = "locale";

/// This function initializes all the translations we're going to use.
pub fn initialize(lang_id: &str) -> Result<Arc<RwLock<FluentBundle<FluentResource>>>> {

    // Get the list of available translations from the locale folder, and load the requested one, if found.
    let locales = get_available_locales();
    let selected_locale = locales.iter().find(|x| x.get_language() == lang_id).ok_or_else(|| return Error::from(ErrorKind::FluentResourceLoadingError))?;
    let locale = format!("{}/{}.ftl", LOCALE_FOLDER, lang_id);

    // If found, load the entire file to a string.
    let mut file = File::open(&locale)?;
    let mut ftl_string = String::new(); 
    file.read_to_string(&mut ftl_string)?;

    // Then to a resource and a bundle.
    let resource = FluentResource::try_new(ftl_string)?;
    let mut bundle = FluentBundle::new(&[selected_locale.clone()]);
    bundle.add_resource(resource)?;

    // If nothing failed, return the new translation.
    Ok(Arc::new(RwLock::new(bundle)))
}

/// This function initializes an empty translation, just in case some idiot deletes the english translation and fails to load it.
pub fn initialize_empty() -> Arc<RwLock<FluentBundle<FluentResource>>> {

    // Create an empty bundle, and return it.
    let ftl_string = String::new(); 
    let resource = FluentResource::try_new(ftl_string).unwrap();
    let mut bundle = FluentBundle::new(&[langid!["en"]]);
    bundle.add_resource(resource).unwrap();
    Arc::new(RwLock::new(bundle))
}

/// This function returns a list of all the languages we have translation files for.
pub fn get_available_locales() -> Vec<LanguageIdentifier> {
    let mut languages = vec![];
    for file in get_files_from_subdir(Path::new("locale")).unwrap() {
        let language = file.file_stem().unwrap().to_string_lossy();
        if let Ok(language_id) = LanguageIdentifier::from_parts(Some(language), None, None, None) {
            languages.push(language_id);
        }
    }
    languages
}

/// This function returns the translation for the key provided in the current language, or a... warning.
///
/// This is a mess, but it works.
pub fn tr(key: &str) -> String {
    let mut _errors = vec![];
    match LOCALE.read().unwrap().get_message(key) {
        Some(message) => match message.value {
            Some(pattern) => LOCALE.read().unwrap().format_pattern(&pattern, None, &mut _errors).to_string(),
            None => tr_fallback(key),
        },
        None => tr_fallback(key),
    }
}

/// This function returns the translation for the key provided in the english language, or a... warning.
///
/// This is a fallback for the `tr` mess.
pub fn tr_fallback(key: &str) -> String {
    let mut _errors = vec![];
    match LOCALE_FALLBACK.read().unwrap().get_message(key) {
        Some(message) => match message.value {
            Some(pattern) => LOCALE_FALLBACK.read().unwrap().format_pattern(&pattern, None, &mut _errors).to_string(),
            None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
        },
        None => "AlL YoUrS TrAnSlAtIoNs ArE BeLoNg To mE.".to_owned(),
    }
}