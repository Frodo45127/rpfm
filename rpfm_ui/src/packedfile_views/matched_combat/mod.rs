//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the view for Matched Combat file.
//!
//! NOTE: For now we use a debug view, as no UI has been done yet.

use anyhow::Result;

use std::sync::Arc;

use rpfm_lib::files::{FileType, matched_combat::MatchedCombat, RFileDecoded};

use crate::packedfile_views::{FileView, View, ViewType};
use crate::views::debug::DebugView;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

pub struct FileMatchedCombatDebugView {
    debug_view: Arc<DebugView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FileMatchedCombatDebugView {

    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: MatchedCombat
    ) -> Result<()> {

        // For now just build a debug view.
        let debug_view = DebugView::new_view(
            file_view.main_widget(),
            RFileDecoded::MatchedCombat(data),
            file_view.path_raw(),
        )?;

        let view = Self {
            debug_view,
        };

        file_view.view_type = ViewType::Internal(View::MatchedCombatDebug(Arc::new(view)));
        file_view.file_type = FileType::MatchedCombat;

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: MatchedCombat) -> Result<()> {
        self.debug_view.reload_view(&serde_json::to_string_pretty(&data)?);

        Ok(())
    }
}
