//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the view for Table PackedFiles.
!*/

use anyhow::Result;

use std::sync::Arc;
use std::rc::Rc;

use rpfm_lib::files::FileType;

use crate::app_ui::AppUI;
use crate::communications::*;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::references_ui::ReferencesUI;
use crate::views::table::{TableView, TableType};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains pointers to all the widgets in a Table View.
pub struct PackedFileTableView {
    table_view: Arc<TableView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileTableView`.
impl PackedFileTableView {

    /// This function creates a new Table View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        response: Response
    ) -> Result<()> {

        // Get the decoded Table.
        let table_data = match response {
            Response::AnimsTableRFileInfo(table, _) => TableType::AnimsTable(table),
            Response::DBRFileInfo(table, _) => TableType::DB(table),
            Response::LocRFileInfo(table, _) => TableType::Loc(table),
            Response::MatchedCombatRFileInfo(table, _) => TableType::MatchedCombat(table),
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let packed_file_type = match table_data {
            TableType::AnimFragment(_) => FileType::AnimFragment,
            TableType::AnimsTable(_) => FileType::AnimsTable,
            TableType::DB(_) => FileType::DB,
            TableType::Loc(_) => FileType::Loc,
            TableType::MatchedCombat(_) => FileType::MatchedCombat,
            _ => unimplemented!()
        };

        let table_view = TableView::new_view(
            packed_file_view.get_mut_widget(),
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui,
            table_data,
            Some(packed_file_view.get_path_raw()),
            packed_file_view.data_source.clone()
        )?;

        let packed_file_table_view = Self {
            table_view,
        };

        packed_file_view.view = ViewType::Internal(View::Table(Arc::new(packed_file_table_view)));
        packed_file_view.packed_file_type = packed_file_type;

        // Return success.
        Ok(())
    }

    pub fn get_ref_table(&self) ->&TableView {
        &self.table_view
    }
}
