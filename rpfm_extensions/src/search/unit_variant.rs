//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters};

use rpfm_lib::files::unit_variant::UnitVariant;

use super::{find_in_string, MatchingMode, Replaceable, Searchable, replace_match_string};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an UnitVariant File.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct UnitVariantMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<UnitVariantMatch>,
}

/// This struct represents a match within an UnitVariant File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct UnitVariantMatch {

    /// The index of the entry in question in the UnitVariant file. Not sure if the ids are unique, so we use the index.
    entry: usize,

    /// If the match corresponds to the name.
    name: bool,

    /// If the match corresponds to a variant value. We have their index and a bool for each value.
    variant: Option<(usize, bool, bool)>,

    /// Byte where the match starts.
    start: usize,

    /// Byte where the match ends.
    end: usize,

    /// Matched data.
    text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for UnitVariant {
    type SearchMatches = UnitVariantMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> UnitVariantMatches {
        let mut matches = UnitVariantMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (index, data) in self.categories().iter().enumerate() {
                    for entry_match in regex.find_iter(data.name()) {
                        matches.matches.push(
                            UnitVariantMatch::new(
                                index,
                                true,
                                None,
                                entry_match.start(),
                                entry_match.end(),
                                data.name().to_owned()
                            )
                        );
                    }

                    for (vindex, variant) in data.variants().iter().enumerate() {
                        for entry_match in regex.find_iter(variant.mesh_file()) {
                            matches.matches.push(
                                UnitVariantMatch::new(
                                    index,
                                    false,
                                    Some((vindex, true, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.mesh_file().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(variant.texture_folder()) {
                            matches.matches.push(
                                UnitVariantMatch::new(
                                    index,
                                    false,
                                    Some((vindex, false, true)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.texture_folder().to_owned()
                                )
                            );
                        }
                    }
                }
            }

            MatchingMode::Pattern(regex) => {
                for (index, data) in self.categories().iter().enumerate() {
                    for (start, end, _) in &find_in_string(data.name(), pattern, case_sensitive, regex) {
                        matches.matches.push(
                            UnitVariantMatch::new(
                                index,
                                true,
                                None,
                                *start,
                                *end,
                                data.name().to_owned()
                            )
                        );
                    }

                    for (vindex, variant) in data.variants().iter().enumerate() {
                        for (start, end, _) in &find_in_string(variant.mesh_file(), pattern, case_sensitive, regex) {
                            matches.matches.push(
                                UnitVariantMatch::new(
                                    index,
                                    false,
                                    Some((vindex, true, false)),
                                    *start,
                                    *end,
                                    variant.mesh_file().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(variant.texture_folder(), pattern, case_sensitive, regex) {
                            matches.matches.push(
                                UnitVariantMatch::new(
                                    index,
                                    false,
                                    Some((vindex, false, true)),
                                    *start,
                                    *end,
                                    variant.texture_folder().to_owned()
                                )
                            );
                        }
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for UnitVariant {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &UnitVariantMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self);
        }

        edited
    }
}

impl UnitVariantMatches {

    /// This function creates a new `UnitVariantMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl UnitVariantMatch {

    /// This function creates a new `UnitVariantMatch` with the provided data.
    pub fn new(entry: usize, name: bool, variant: Option<(usize, bool, bool)>, start: usize, end: usize, text: String) -> Self {
        Self {
            entry,
            name,
            variant,
            start,
            end,
            text
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut UnitVariant) -> bool {
        let mut edited = false;

        if let Some(entry) = data.categories_mut().get_mut(self.entry) {

            // Get all the previous data and references of data to manipulate here, so we don't duplicate a lot of code per-field in the match mode part.
            let (previous_data, current_data) = {
                if self.name {
                    (entry.name().to_owned(), entry.name_mut())
                } else if let Some((vindex, mesh_file, texture_folder)) = self.variant {
                    match entry.variants_mut().get_mut(vindex) {
                        Some(variant) => {
                            if mesh_file {
                                (variant.mesh_file().to_owned(), variant.mesh_file_mut())
                            } else if texture_folder {
                                (variant.texture_folder().to_owned(), variant.texture_folder_mut())
                            } else {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }

                // This is an error.
                else {
                    return false
                }
            };

            edited = replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, &previous_data, current_data);
        }

        edited
    }
}
