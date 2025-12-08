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

use rpfm_lib::files::rigidmodel::RigidModel;

use super::{find_in_string, MatchingMode, Replaceable, Searchable, replace_match_string};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents all the matches of the global search within an RigidModel File.
#[derive(Debug, Clone, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatches {

    /// The path of the file.
    path: String,

    /// The list of matches within the file.
    matches: Vec<RigidModelMatch>,
}

/// This struct represents a match within an RigidModel File.
#[derive(Debug, Clone, Eq, PartialEq, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RigidModelMatch {

    /// If the match is in the skeleton id of the rigid.
    skeleton_id: bool,

    /// If the match is inside a mesh. The values are
    /// - Lod index.
    /// - Mesh index.
    mesh_value: Option<(i32, i32)>,

    /// If the match is in the name of a mesh.
    mesh_name: bool,

    /// If the match is in the name of the material.
    mesh_mat_name: bool,

    /// If the match is in the texture directory of a mesh.
    mesh_textute_directory: bool,

    /// If the match is in the filters of a mesh.
    mesh_filters: bool,

    /// If the match is in the name of one of the attachment points of a mesh.
    mesh_att_point_name: Option<i32>,

    /// If the match is in the name of one of the texture paths of a mesh.
    mesh_texture_path: Option<i32>,

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

impl Searchable for RigidModel {
    type SearchMatches = RigidModelMatches;

    fn search(&self, file_path: &str, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> RigidModelMatches {
        let mut matches = RigidModelMatches::new(file_path);

        match matching_mode {
            MatchingMode::Regex(regex) => {

                for entry_match in regex.find_iter(self.skeleton_id()) {
                    matches.matches.push(
                        RigidModelMatch::new(
                            true,
                            None,
                            false,
                            false,
                            false,
                            false,
                            None,
                            None,
                            entry_match.start(),
                            entry_match.end(),
                            self.skeleton_id().to_owned()
                        )
                    );
                }

                for (l_index, lod) in self.lods().iter().enumerate() {
                    for (m_index, mesh) in lod.mesh_blocks().iter().enumerate() {
                        for entry_match in regex.find_iter(mesh.mesh().name()) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    true,
                                    false,
                                    false,
                                    false,
                                    None,
                                    None,
                                    entry_match.start(),
                                    entry_match.end(),
                                    mesh.mesh().name().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(mesh.material().name()) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    true,
                                    false,
                                    false,
                                    None,
                                    None,
                                    entry_match.start(),
                                    entry_match.end(),
                                    mesh.material().name().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(mesh.material().texture_directory()) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    false,
                                    true,
                                    false,
                                    None,
                                    None,
                                    entry_match.start(),
                                    entry_match.end(),
                                    mesh.material().texture_directory().to_owned()
                                )
                            );
                        }

                        for entry_match in regex.find_iter(mesh.material().filters()) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    false,
                                    false,
                                    true,
                                    None,
                                    None,
                                    entry_match.start(),
                                    entry_match.end(),
                                    mesh.material().filters().to_owned()
                                )
                            );
                        }

                        for (a_index, attachment) in mesh.material().attachment_points().iter().enumerate() {
                            for entry_match in regex.find_iter(attachment.name()) {
                                matches.matches.push(
                                    RigidModelMatch::new(
                                        false,
                                        Some((l_index as i32, m_index as i32)),
                                        false,
                                        false,
                                        false,
                                        false,
                                        Some(a_index as i32),
                                        None,
                                        entry_match.start(),
                                        entry_match.end(),
                                        attachment.name().to_owned()
                                    )
                                );
                            }
                        }

                        for (t_index, texture) in mesh.material().textures().iter().enumerate() {
                            for entry_match in regex.find_iter(texture.path()) {
                                matches.matches.push(
                                    RigidModelMatch::new(
                                        false,
                                        Some((l_index as i32, m_index as i32)),
                                        false,
                                        false,
                                        false,
                                        false,
                                        None,
                                        Some(t_index as i32),
                                        entry_match.start(),
                                        entry_match.end(),
                                        texture.path().to_owned()
                                    )
                                );
                            }
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

                for (start, end, _) in &find_in_string(self.skeleton_id(), &pattern, case_sensitive, regex) {
                    matches.matches.push(
                        RigidModelMatch::new(
                            true,
                            None,
                            false,
                            false,
                            false,
                            false,
                            None,
                            None,
                            *start,
                            *end,
                            self.skeleton_id().to_owned()
                        )
                    );
                }

                for (l_index, lod) in self.lods().iter().enumerate() {
                    for (m_index, mesh) in lod.mesh_blocks().iter().enumerate() {
                        for (start, end, _) in &find_in_string(mesh.mesh().name(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    true,
                                    false,
                                    false,
                                    false,
                                    None,
                                    None,
                                    *start,
                                    *end,
                                    mesh.mesh().name().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(mesh.material().name(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    true,
                                    false,
                                    false,
                                    None,
                                    None,
                                    *start,
                                    *end,
                                    mesh.material().name().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(mesh.material().texture_directory(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    false,
                                    true,
                                    false,
                                    None,
                                    None,
                                    *start,
                                    *end,
                                    mesh.material().texture_directory().to_owned()
                                )
                            );
                        }

                        for (start, end, _) in &find_in_string(mesh.material().filters(), &pattern, case_sensitive, regex) {
                            matches.matches.push(
                                RigidModelMatch::new(
                                    false,
                                    Some((l_index as i32, m_index as i32)),
                                    false,
                                    false,
                                    false,
                                    true,
                                    None,
                                    None,
                                    *start,
                                    *end,
                                    mesh.material().filters().to_owned()
                                )
                            );
                        }

                        for (a_index, attachment) in mesh.material().attachment_points().iter().enumerate() {
                            for (start, end, _) in &find_in_string(attachment.name(), &pattern, case_sensitive, regex) {
                                matches.matches.push(
                                    RigidModelMatch::new(
                                        false,
                                        Some((l_index as i32, m_index as i32)),
                                        false,
                                        false,
                                        false,
                                        false,
                                        Some(a_index as i32),
                                        None,
                                        *start,
                                        *end,
                                        attachment.name().to_owned()
                                    )
                                );
                            }
                        }

                        for (t_index, texture) in mesh.material().textures().iter().enumerate() {
                            for (start, end, _) in &find_in_string(texture.path(), &pattern, case_sensitive, regex) {
                                matches.matches.push(
                                    RigidModelMatch::new(
                                        false,
                                        Some((l_index as i32, m_index as i32)),
                                        false,
                                        false,
                                        false,
                                        false,
                                        None,
                                        Some(t_index as i32),
                                        *start,
                                        *end,
                                        texture.path().to_owned()
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }

        matches
    }
}

impl Replaceable for RigidModel {

    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &RigidModelMatches) -> bool {
        let mut edited = false;

        // NOTE: Due to changes in index positions, we need to do this in reverse.
        // Otherwise we may cause one edit to generate invalid indexes for the next matches.
        for search_match in search_matches.matches().iter().rev() {
            edited |= search_match.replace(pattern, replace_pattern, case_sensitive, matching_mode, self);
        }

        edited
    }
}

impl RigidModelMatches {

    /// This function creates a new `RigidModelMatches` for the provided path.
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            matches: vec![],
        }
    }
}

impl RigidModelMatch {

    /// This function creates a new `RigidModelMatch` with the provided data.
    pub fn new(
        skeleton_id: bool,
        mesh_value: Option<(i32, i32)>,
        mesh_name: bool,
        mesh_mat_name: bool,
        mesh_textute_directory: bool,
        mesh_filters: bool,
        mesh_att_point_name: Option<i32>,
        mesh_texture_path: Option<i32>,
        start: usize,
        end: usize,
        data: String
    ) -> Self {
        Self {
            skeleton_id,
            mesh_value,
            mesh_name,
            mesh_mat_name,
            mesh_textute_directory,
            mesh_filters,
            mesh_att_point_name,
            mesh_texture_path,
            start,
            end,
            text: data,
        }
    }

    /// This function replaces all the matches in the provided data.
    fn replace(&self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, data: &mut RigidModel) -> bool {

        // Get all the previous data and references of data to manipulate here, so we don't duplicate a lot of code per-field in the match mode part.
        let (previous_data, current_data) = {
            if self.skeleton_id {
                (data.skeleton_id().to_owned(), data.skeleton_id_mut())
            } else if let Some((l_index, m_index)) = self.mesh_value {
                if let Some(lod) = data.lods_mut().get_mut(l_index as usize) {
                    if let Some(mesh) = lod.mesh_blocks_mut().get_mut(m_index as usize) {
                        if self.mesh_name {
                            (mesh.mesh().name().to_owned(), mesh.mesh_mut().name_mut())
                        } else if self.mesh_mat_name {
                            (mesh.material().name().to_owned(), mesh.material_mut().name_mut())
                        } else if self.mesh_textute_directory {
                            (mesh.material().texture_directory().to_owned(), mesh.material_mut().texture_directory_mut())
                        } else if self.mesh_filters {
                            (mesh.material().filters().to_owned(), mesh.material_mut().filters_mut())
                        } else if let Some(a_index) = self.mesh_att_point_name {
                            if let Some(att) = mesh.material_mut().attachment_points_mut().get_mut(a_index as usize) {
                                (att.name().to_owned(), att.name_mut())
                            } else {
                                return false
                            }
                        } else if let Some(t_index) = self.mesh_texture_path {
                            if let Some(tex) = mesh.material_mut().textures_mut().get_mut(t_index as usize) {
                                (tex.path().to_owned(), tex.path_mut())
                            } else {
                                return false
                            }
                        } else {
                            return false
                        }
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            } else {
                return false
            }
        };

        replace_match_string(pattern, replace_pattern, case_sensitive, matching_mode, self.start, self.end, &previous_data, current_data)
    }
}
