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

use rpfm_lib::files::portrait_settings::PortraitSettings;

use super::{find_in_string, MatchingMode, Replaceable, Searchable, replace_match_string};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an PortraitSettings File.
#[derive(Debug, Clone, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PortraitSettingsMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<PortraitSettingsMatch>,
}

/// This struct represents a match within an PortraitSettings File.
#[derive(Debug, Default, Clone, Eq, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PortraitSettingsMatch {

    /// The index of the entry in question in the PortraitSettings file. Not sure if the ids are unique, so we use the index.
    entry: usize,

    /// If the match corresponds to the id.
    id: bool,

    /// If the match corresponds to a camera settings head (skeleton node) value.
    camera_settings_head: bool,

    /// If the match corresponds to a camera settings body (skeleton node) value.
    camera_settings_body: bool,

    /// If the match corresponds to a variant value. We have their index and a bool for each value.
    variant: Option<(usize, bool, bool, bool, bool, bool)>,

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

impl Searchable for PortraitSettings {
    type SearchMatches = PortraitSettingsMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> PortraitSettingsMatches {
        let mut matches = PortraitSettingsMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {
                for (index, data) in self.entries().iter().enumerate() {
                    for entry_match in regex.find_iter(data.id()) {
                        matches.matches.push(
                            PortraitSettingsMatch::new(
                                index,
                                true,
                                false,
                                false,
                                None,
                                entry_match.start(),
                                entry_match.end(),
                                data.id().to_owned()
                            )
                        );
                    }

                    for entry_match in regex.find_iter(data.camera_settings_head().skeleton_node()) {
                        matches.matches.push(
                            PortraitSettingsMatch::new(
                                index,
                                false,
                                true,
                                false,
                                None,
                                entry_match.start(),
                                entry_match.end(),
                                data.camera_settings_head().skeleton_node().to_owned()
                            )
                        );
                    }

                    if let Some(camera_body) = data.camera_settings_body() {
                        for entry_match in regex.find_iter(camera_body.skeleton_node()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    true,
                                    None,
                                    entry_match.start(),
                                    entry_match.end(),
                                    camera_body.skeleton_node().to_owned()
                                )
                            );
                        }
                    }

                    for (vindex, variant) in data.variants().iter().enumerate() {
                        for entry_match in regex.find_iter(variant.filename()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, true, false, false, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.filename().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(variant.file_diffuse()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, true, false, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.file_diffuse().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(variant.file_mask_1()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, true, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.file_mask_1().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(variant.file_mask_2()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, false, true, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.file_mask_2().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(variant.file_mask_3()) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, false, false, true)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    variant.file_mask_3().to_owned()
                                )
                            );
                        }
                    }
                }
            }

            MatchingMode::Pattern(regex) => {
                let pattern = if case_sensitive || regex.is_some() {
                    pattern.to_owned()
                } else {
                    pattern.to_lowercase()
                };

                for (index, data) in self.entries().iter().enumerate() {

                    for (start, end, _) in &find_in_string(data.id(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            PortraitSettingsMatch::new(
                                index,
                                true,
                                false,
                                false,
                                None,
                                *start,
                                *end,
                                data.id().to_owned()
                            )
                        );

                    }

                    for (start, end, _) in &find_in_string(data.camera_settings_head().skeleton_node(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            PortraitSettingsMatch::new(
                                index,
                                false,
                                true,
                                false,
                                None,
                                *start,
                                *end,
                                data.camera_settings_head().skeleton_node().to_owned()
                            )
                        );
                    }

                    if let Some(camera_body) = data.camera_settings_body() {
                        for (start, end, _) in &find_in_string(camera_body.skeleton_node(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    true,
                                    None,
                                    *start,
                                    *end,
                                    camera_body.skeleton_node().to_owned()
                                )
                            );
                        }
                    }

                    for (vindex, variant) in data.variants().iter().enumerate() {
                        for (start, end, _) in &find_in_string(variant.filename(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, true, false, false, false, false)),
                                    *start,
                                    *end,
                                    variant.filename().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(variant.file_diffuse(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, true, false, false, false)),
                                    *start,
                                    *end,
                                    variant.file_diffuse().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(variant.file_mask_1(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, true, false, false)),
                                    *start,
                                    *end,
                                    variant.file_mask_1().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(variant.file_mask_2(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, false, true, false)),
                                    *start,
                                    *end,
                                    variant.file_mask_2().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(variant.file_mask_3(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                PortraitSettingsMatch::new(
                                    index,
                                    false,
                                    false,
                                    false,
                                    Some((vindex, false, false, false, false, true)),
                                    *start,
                                    *end,
                                    variant.file_mask_3().to_owned()
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

impl Replaceable for PortraitSettings {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &PortraitSettingsMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self);
        }

        edited
    }
}

impl PortraitSettingsMatches {

    /// This function creates a new `PortraitSettingsMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl PortraitSettingsMatch {

    /// This function creates a new `PortraitSettingsMatch` with the provided data.
    pub fn new(entry: usize, id: bool, camera_settings_head: bool, camera_settings_body: bool, variant: Option<(usize, bool, bool, bool, bool, bool)>, start: usize, end: usize, data: String) -> Self {
        Self {
            entry,
            id,
            camera_settings_head,
            camera_settings_body,
            variant,
            start,
            end,
            text: data
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut PortraitSettings) -> bool {
        let mut edited = false;

        if let Some(entry) = data.entries_mut().get_mut(self.entry) {

            // Get all the previous data and references of data to manipulate here, so we don't duplicate a lot of code per-field in the match mode part.
            let (previous_data, current_data) = {
                if self.id {
                    (entry.id().to_owned(), entry.id_mut())
                } else if self.camera_settings_head {
                    (entry.camera_settings_head().skeleton_node().to_owned(), entry.camera_settings_head_mut().skeleton_node_mut())
                } else if self.camera_settings_body {
                    match entry.camera_settings_body_mut() {
                        Some(body) => (body.skeleton_node().to_owned(), body.skeleton_node_mut()),
                        None => return false,
                    }
                } else if let Some((vindex, filename, file_diffuse, file_mask_1, file_mask_2, file_mask_3)) = self.variant {
                    match entry.variants_mut().get_mut(vindex) {
                        Some(variant) => {
                            if filename {
                                (variant.filename().to_owned(), variant.filename_mut())
                            } else if file_diffuse {
                                (variant.file_diffuse().to_owned(), variant.file_diffuse_mut())
                            } else if file_mask_1 {
                                (variant.file_mask_1().to_owned(), variant.file_mask_1_mut())
                            } else if file_mask_2 {
                                (variant.file_mask_2().to_owned(), variant.file_mask_2_mut())
                            } else if file_mask_3 {
                                (variant.file_mask_3().to_owned(), variant.file_mask_3_mut())
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
