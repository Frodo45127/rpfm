//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Methods about the translations of the open packs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use rpfm_extensions::translator::DEFAULT_SRC_LANG;

use super::Request;

/// Default amount of entries returned by [`ListTranslations`].
pub const DEFAULT_TRANSLATIONS_LIMIT: usize = 200;

/// `translations.list`: returns the translation of the texts of an open pack to a language.
///
/// Texts already translated in the vanilla game, or in previous translations of the pack, come translated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ListTranslations {

    /// Key of the pack.
    pub pack: String,

    /// Language the texts of the pack are written in, like `EN`.
    #[serde(default = "default_source_language")]
    pub source_language: String,

    /// Language to translate the texts to, like `ES` or `DE`.
    pub language: String,

    /// Which entries to return.
    #[serde(default)]
    pub filter: TranslationFilter,

    /// Amount of matching entries to skip.
    #[serde(default)]
    pub offset: usize,

    /// Maximum amount of entries to return. Defaults to [`DEFAULT_TRANSLATIONS_LIMIT`].
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Which translation entries to return.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TranslationFilter {

    /// All of them.
    #[default]
    All,

    /// Entries without a translation.
    Untranslated,

    /// Entries whose source text changed since they were translated, or that were translated automatically.
    NeedsReview,
}

/// A page of the translation of a pack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Translations {

    /// The entries in the page, sorted by key.
    pub entries: Vec<TranslationEntry>,

    /// Amount of entries matching the request, in all pages.
    pub total: usize,
}

/// The translation of a text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TranslationEntry {

    /// Loc key of the text.
    pub key: String,

    /// Text in the source language.
    pub source: String,

    /// Text in the target language. Empty if it's not translated yet.
    pub translation: String,

    /// If the source text changed since it was translated.
    pub needs_retranslation: bool,

    /// If the translation was made automatically, and still needs a review.
    pub automatic: bool,

    /// If the text no longer exists in the pack.
    pub removed: bool,
}

/// `translations.generate_vanilla`: generates the vanilla texts of a language from the game's locale packs,
/// so translations to and from it can reuse them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GenerateVanillaTexts {

    /// Language of the texts to generate, like `EN`.
    pub language: String,
}

/// If the vanilla texts of a language are available.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VanillaTextsAvailable {

    /// If they're available, generated now or before.
    pub available: bool,
}

impl Request for ListTranslations {
    const METHOD: &'static str = "translations.list";
    type Response = Translations;
}

impl Request for GenerateVanillaTexts {
    const METHOD: &'static str = "translations.generate_vanilla";
    type Response = VanillaTextsAvailable;
}

/// Default source language of translations.
fn default_source_language() -> String {
    DEFAULT_SRC_LANG.to_owned()
}
