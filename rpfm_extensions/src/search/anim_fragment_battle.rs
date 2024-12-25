//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use getset::{Getters, MutGetters};
use serde_derive::{Deserialize, Serialize};

use rpfm_lib::files::anim_fragment_battle::AnimFragmentBattle;

use super::{find_in_string, MatchingMode, replace_match_string, Replaceable, Searchable};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an Anim Fragment Battle File.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentBattleMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<AnimFragmentBattleMatch>,
}

/// This struct represents a match within an Anim Fragment Battle File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct AnimFragmentBattleMatch {

    /// If the match corresponds to a skeleton name value.
    skeleton_name: bool,

    /// If the match corresponds to a table name value.
    table_name: bool,

    /// If the match corresponds to a mount table name value.
    mount_table_name: bool,

    /// If the match corresponds to a unmount table name value.
    unmount_table_name: bool,

    /// If the match corresponds to a locomotion table name value.
    locomotion_graph: bool,

    /// If the match corresponds to an entry in the table view.
    entry: Option<(usize, Option<(usize, bool, bool, bool)>, bool, bool, bool, bool, bool)>,

    /// Byte where the match starts.
    start: usize,

    /// Byte where the match ends.
    end: usize,

    /// The contents of the matched cell.
    text: String,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl Searchable for AnimFragmentBattle {
    type SearchMatches = AnimFragmentBattleMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> AnimFragmentBattleMatches {
        let mut matches = AnimFragmentBattleMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {
                for entry_match in regex.find_iter(self.skeleton_name()) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            true,
                            false,
                            false,
                            false,
                            false,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.skeleton_name().to_owned()
                        )
                    );
                }

                for entry_match in regex.find_iter(self.table_name()) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            true,
                            false,
                            false,
                            false,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.table_name().to_owned()
                        )
                    );
                }

                for entry_match in regex.find_iter(self.mount_table_name()) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            true,
                            false,
                            false,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.mount_table_name().to_owned()
                        )
                    );
                }

                for entry_match in regex.find_iter(self.unmount_table_name()) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.unmount_table_name().to_owned()
                        )
                    );
                }

                for entry_match in regex.find_iter(self.locomotion_graph()) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            false,
                            false,
                            true,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.locomotion_graph().to_owned()
                        )
                    );
                }

                for (row, entry) in self.entries().iter().enumerate() {
                    for (subrow, anim_refs) in entry.anim_refs().iter().enumerate() {
                        for entry_match in regex.find_iter(anim_refs.file_path()) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, true, false, false)), false, false, false, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    anim_refs.file_path().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(anim_refs.meta_file_path()) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, false, true, false)), false, false, false, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    anim_refs.meta_file_path().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(anim_refs.snd_file_path()) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, false, false, true)), false, false, false, false, false)),
                                    entry_match.start(),
                                    entry_match.end(),
                                    anim_refs.snd_file_path().to_owned()
                                )
                            );
                        }
                    }

                    for entry_match in regex.find_iter(entry.filename()) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, true, false, false, false, false)),
                                entry_match.start(),
                                entry_match.end(),
                                entry.filename().to_owned()
                            )
                        );
                    }

                    for entry_match in regex.find_iter(entry.metadata()) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, true, false, false, false)),
                                entry_match.start(),
                                entry_match.end(),
                                entry.metadata().to_owned()
                            )
                        );
                    }

                    for entry_match in regex.find_iter(entry.metadata_sound()) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, true, false, false)),
                                entry_match.start(),
                                entry_match.end(),
                                entry.metadata_sound().to_owned()
                            )
                        );
                    }

                    for entry_match in regex.find_iter(entry.skeleton_type()) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, false, true, false)),
                                entry_match.start(),
                                entry_match.end(),
                                entry.skeleton_type().to_owned()
                            )
                        );
                    }

                    for entry_match in regex.find_iter(entry.uk_4()) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, false, false, true)),
                                entry_match.start(),
                                entry_match.end(),
                                entry.uk_4().to_owned()
                            )
                        );
                    }
                }
            }

            MatchingMode::Pattern(regex) => {
                let pattern = if case_sensitive || regex.is_some() {
                    pattern.to_owned()
                } else {
                    pattern.to_lowercase()
                };

                for (start, end, _) in &find_in_string(self.skeleton_name(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            true,
                            false,
                            false,
                            false,
                            false,
                            None,
                            *start,
                            *end,
                            self.skeleton_name().to_owned()
                        )
                    );
                }

                for (start, end, _) in &find_in_string(self.table_name(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            true,
                            false,
                            false,
                            false,
                            None,
                            *start,
                            *end,
                            self.table_name().to_owned()
                        )
                    );
                }

                for (start, end, _) in &find_in_string(self.mount_table_name(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            true,
                            false,
                            false,
                            None,
                            *start,
                            *end,
                            self.mount_table_name().to_owned()
                        )
                    );
                }

                for (start, end, _) in &find_in_string(self.unmount_table_name(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            false,
                            true,
                            false,
                            None,
                            *start,
                            *end,
                            self.unmount_table_name().to_owned()
                        )
                    );
                }

                for (start, end, _) in &find_in_string(self.locomotion_graph(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        AnimFragmentBattleMatch::new(
                            false,
                            false,
                            false,
                            false,
                            true,
                            None,
                            *start,
                            *end,
                            self.locomotion_graph().to_owned()
                        )
                    );
                }

                for (row, entry) in self.entries().iter().enumerate() {
                    for (subrow, anim_refs) in entry.anim_refs().iter().enumerate() {
                        for (start, end, _) in &find_in_string(anim_refs.file_path(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, true, false, false)), false, false, false, false, false)),
                                    *start,
                                    *end,
                                    anim_refs.file_path().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(anim_refs.meta_file_path(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, false, true, false)), false, false, false, false, false)),
                                    *start,
                                    *end,
                                    anim_refs.meta_file_path().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(anim_refs.snd_file_path(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                AnimFragmentBattleMatch::new(
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((row, Some((subrow, false, false, true)), false, false, false, false, false)),
                                    *start,
                                    *end,
                                    anim_refs.snd_file_path().to_owned()
                                )
                            );
                        }
                    }

                    for (start, end, _) in &find_in_string(entry.filename(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, true, false, false, false, false)),
                                *start,
                                *end,
                                entry.filename().to_owned()
                            )
                        );
                    }

                    for (start, end, _) in &find_in_string(entry.metadata(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, true, false, false, false)),
                                *start,
                                *end,
                                entry.metadata().to_owned()
                            )
                        );
                    }

                    for (start, end, _) in &find_in_string(entry.metadata_sound(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, true, false, false)),
                                *start,
                                *end,
                                entry.metadata_sound().to_owned()
                            )
                        );
                    }

                    for (start, end, _) in &find_in_string(entry.skeleton_type(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, false, true, false)),
                                *start,
                                *end,
                                entry.skeleton_type().to_owned()
                            )
                        );
                    }

                    for (start, end, _) in &find_in_string(entry.uk_4(), &pattern, case_sensitive, regex) {
                        matches.matches.push(
                            AnimFragmentBattleMatch::new(
                                false,
                                false,
                                false,
                                false,
                                false,
                                Some((row, None, false, false, false, false, true)),
                                *start,
                                *end,
                                entry.uk_4().to_owned()
                            )
                        );
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for AnimFragmentBattle {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &AnimFragmentBattleMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self);
        }

        edited
    }
}

impl AnimFragmentBattleMatches {

    /// This function creates a new `AnimFragmentBattleMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl AnimFragmentBattleMatch {

    /// This function creates a new `AnimFragmentBattleMatch` with the provided data.
    pub fn new(skeleton_name: bool, table_name: bool, mount_table_name: bool, unmount_table_name: bool, locomotion_graph: bool, entry: Option<(usize, Option<(usize, bool, bool, bool)>, bool, bool, bool, bool, bool)>, start: usize, end: usize, text: String) -> Self {
        Self {
            skeleton_name,
            table_name,
            mount_table_name,
            unmount_table_name,
            locomotion_graph,
            entry,
            start,
            end,
            text,
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut AnimFragmentBattle) -> bool {

        // Get all the previous data and references of data to manipulate here, so we don't duplicate a lot of code per-field in the match mode part.
        let (previous_data, current_data) = {
            if self.skeleton_name {
                (data.skeleton_name().to_owned(), data.skeleton_name_mut())
            } else if self.table_name {
                (data.table_name().to_owned(), data.table_name_mut())
            } else if self.mount_table_name {
                (data.mount_table_name().to_owned(), data.mount_table_name_mut())
            } else if self.unmount_table_name {
                (data.unmount_table_name().to_owned(), data.unmount_table_name_mut())
            } else if self.locomotion_graph {
                (data.locomotion_graph().to_owned(), data.locomotion_graph_mut())
            } else if let Some((row, anim_ref, filename, metadata, metadata_sound, skeleton_type, uk_4)) = self.entry {
                match data.entries_mut().get_mut(row) {
                    Some(entry) => {
                        if let Some((subrow, file_path, meta_file_path, snd_file_path)) = anim_ref {
                             match entry.anim_refs_mut().get_mut(subrow) {
                                Some(subentry) => {
                                    if file_path {
                                        (subentry.file_path().to_owned(), subentry.file_path_mut())
                                    } else if meta_file_path {
                                        (subentry.meta_file_path().to_owned(), subentry.meta_file_path_mut())
                                    } else if snd_file_path {
                                        (subentry.snd_file_path().to_owned(), subentry.snd_file_path_mut())
                                    } else {
                                        return false;
                                    }
                                }
                                None => return false,
                            }
                        } else if filename {
                            (entry.filename().to_owned(), entry.filename_mut())
                        } else if metadata {
                            (entry.metadata().to_owned(), entry.metadata_mut())
                        } else if metadata_sound {
                            (entry.metadata_sound().to_owned(), entry.metadata_sound_mut())
                        } else if skeleton_type {
                            (entry.skeleton_type().to_owned(), entry.skeleton_type_mut())
                        } else if uk_4 {
                            (entry.uk_4().to_owned(), entry.uk_4_mut())
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

        replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, &previous_data, current_data)
    }
}
