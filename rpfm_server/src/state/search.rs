//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Global search and replace operations.

use anyhow::Result;

use rpfm_extensions::search::{GlobalSearch, MatchHolder};

use rpfm_ipc::helpers::RFileInfo;

use rpfm_lib::files::{Container, ContainerPath};

use super::{SessionState, loaded_schema};

impl SessionState {

    /// Runs a global search over the open packs and the dependencies.
    ///
    /// # Returns
    ///
    /// The search with its matches, and the info of the files with matches in the open packs.
    pub fn global_search(&mut self, mut search: GlobalSearch) -> Result<(GlobalSearch, Vec<RFileInfo>)> {
        let schema = loaded_schema(&self.schema)?;
        search.search(&self.game, schema, &mut self.packs, &mut self.dependencies, &[]);

        let files_info = RFileInfo::info_from_global_search(&search, &self.packs);
        Ok((search, files_info))
    }

    /// Replaces matches of a global search.
    ///
    /// # Arguments
    ///
    /// * `search` - The search the matches come from.
    /// * `matches` - Matches to replace. If `None`, all the matches of the search are replaced.
    ///
    /// # Returns
    ///
    /// The search, and the info of the edited files.
    pub fn global_search_replace(&mut self, mut search: GlobalSearch, matches: Option<&[MatchHolder]>) -> Result<(GlobalSearch, Vec<RFileInfo>)> {
        let schema = loaded_schema(&self.schema)?;
        let edited_paths = match matches {
            Some(matches) => search.replace(&self.game, schema, &mut self.packs, &mut self.dependencies, matches)?,
            None => search.replace_all(&self.game, schema, &mut self.packs, &mut self.dependencies)?,
        };

        Ok((search, self.files_info_in_all_packs(&edited_paths)))
    }

    /// Returns the info of the files at the provided paths, from every open pack.
    fn files_info_in_all_packs(&self, paths: &[ContainerPath]) -> Vec<RFileInfo> {
        paths.iter()
            .flat_map(|path| self.packs.values().flat_map(move |pack| pack.files_by_path(path, false)))
            .map(RFileInfo::from)
            .collect()
    }
}
