//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Submission of translations to the Translation Hub.
//!
//! Builds, from a saved [`PackTranslation`], everything needed to submit it to the hub as a pull
//! request: where its file goes, which outdated file it replaces, the branch it's pushed to, and the
//! texts of the commit and the pull request. The GitHub requests themselves are done elsewhere.

use getset::Getters;
use serde_derive::{Deserialize, Serialize};

use super::{DEFAULT_SRC_LANG, PackTranslation};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// Line counts of a translation, ignoring lines removed from the pack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TranslationStats {

    /// Lines in the pack.
    total: usize,

    /// Lines with an up-to-date translation.
    translated: usize,

    /// Lines that still need translating, because they're new or their source text changed.
    pending: usize,

    /// Up-to-date lines translated automatically and not reviewed yet.
    auto_translated: usize,
}

/// Everything needed to submit a translation to the Translation Hub as a pull request.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct HubSubmission {

    /// Path of the translation's file in the hub.
    file_path: String,

    /// Path of the same translation in the other format version, to delete from the hub if it's there.
    replaced_path: Option<String>,

    /// Branch the submission is pushed to. It's the same for every submission of the same translation.
    branch: String,

    /// Message of the submission's commit.
    commit_message: String,

    /// Title of the pull request.
    title: String,

    /// Description of the pull request, in Markdown.
    body: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PackTranslation {

    /// Path of this translation's file, relative to a translations folder or the hub's root.
    ///
    /// # Arguments
    ///
    /// * `game_key` - Key of the game the translation belongs to.
    ///
    /// # Returns
    ///
    /// `{game_key}/{pack_name}/{file_name}`, with `/` separators so it's also valid as a repository path.
    pub fn relative_path(&self, game_key: &str) -> String {
        format!("{}/{}/{}", game_key, self.pack_name, self.file_name())
    }

    /// Line counts of this translation, ignoring lines removed from the pack.
    pub fn stats(&self) -> TranslationStats {
        self.translations.values()
            .filter(|tr| !tr.rem)
            .fold(TranslationStats::default(), |mut stats, tr| {
                stats.total += 1;
                if tr.retr {
                    stats.pending += 1;
                } else {
                    stats.translated += 1;
                    if tr.aut {
                        stats.auto_translated += 1;
                    }
                }

                stats
            })
    }

    /// Build the submission of this translation to the Translation Hub.
    ///
    /// # Arguments
    ///
    /// * `game_key` - Key of the game the translation belongs to.
    ///
    /// # Returns
    ///
    /// The paths, branch and texts of the submission.
    pub fn hub_submission(&self, game_key: &str) -> HubSubmission {
        let folder = format!("{}/{}", game_key, self.pack_name);

        // Mirrors `save`: only EN-sourced translations have files in both formats, and `{language}.json` is always the EN one.
        let replaced_path = if self.src_lang.eq_ignore_ascii_case(DEFAULT_SRC_LANG) {
            let other_file_name = if self.version == 0 {
                format!("{}-{}.json", self.src_lang, self.language)
            } else {
                format!("{}.json", self.language)
            };

            Some(format!("{folder}/{other_file_name}"))
        } else {
            None
        };

        let branch = ["rpfm", game_key, &self.pack_name, &format!("{}-{}", self.src_lang, self.language)]
            .iter()
            .map(|segment| branch_segment(segment))
            .collect::<Vec<_>>()
            .join("/");

        let languages = format!("{} -> {}", self.src_lang, self.language);
        let stats = self.stats();
        let authors = if self.authors.is_empty() {
            "Not specified".to_owned()
        } else {
            self.authors.join(", ")
        };

        let body = format!("Translation submitted from RPFM's Translator.

- Game: `{game_key}`
- Pack: `{pack}`
- Languages: `{languages}`
- Format version: `{version}`
- Authors: {authors}
- Lines: {translated} of {total} translated, {pending} pending, {auto} of the translated ones auto-translated and not reviewed.
",
            pack = self.pack_name,
            version = self.version,
            translated = stats.translated,
            total = stats.total,
            pending = stats.pending,
            auto = stats.auto_translated,
        );

        HubSubmission {
            file_path: self.relative_path(game_key),
            replaced_path,
            branch,
            commit_message: format!("Update {} translation ({languages}) [{game_key}]", self.pack_name),
            title: format!("[{game_key}] Translation for {} ({languages})", self.pack_name),
            body,
        }
    }
}

/// Turn a text into a valid segment of a git branch name.
///
/// Characters other than ASCII letters, digits, `.`, `_` and `-` become `-`, and the result avoids the
/// sequences git forbids in reference names: `..`, a leading or trailing `.`, and a trailing `.lock`.
fn branch_segment(text: &str) -> String {
    let mut segment = String::with_capacity(text.len());
    for character in text.chars() {
        let character = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') { character } else { '-' };
        let previous = segment.chars().last();

        // Collapse runs of dashes, and break up `..` so it can't appear.
        if (character == '-' && previous == Some('-')) || (character == '.' && previous == Some('.')) {
            continue;
        }

        segment.push(character);
    }

    let mut segment = segment.trim_matches(|character| character == '-' || character == '.').to_owned();
    if segment.ends_with(".lock") {
        segment.push('_');
    }

    if segment.is_empty() {
        segment.push('_');
    }

    segment
}
