//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use std::sync::Arc;

use std::rc::Rc;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, View, ViewType};

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
    ) -> Result<Option<PackedFileInfo>> {

        // Get the decoded Table.
        let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));

        let response = CentralCommand::recv(&receiver);
        let (table_data, packed_file_info) = match response {
            Response::AnimTablePackedFileInfo((table, packed_file_info)) => (TableType::AnimTable(table), Some(packed_file_info)),
            Response::DBPackedFileInfo((table, packed_file_info)) => (TableType::DB(table), Some(packed_file_info)),
            Response::LocPackedFileInfo((table, packed_file_info)) => (TableType::Loc(table), Some(packed_file_info)),
            Response::MatchedCombatPackedFileInfo((table, packed_file_info)) => (TableType::MatchedCombat(table), Some(packed_file_info)),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let packed_file_type = match table_data {

            // This one should never happen.
            TableType::AnimFragment(_) => PackedFileType::AnimFragment,
            TableType::AnimTable(_) => PackedFileType::AnimTable,
            TableType::DB(_) => PackedFileType::DB,
            TableType::Loc(_) => PackedFileType::Loc,
            TableType::MatchedCombat(_) => PackedFileType::MatchedCombat,
            _ => unimplemented!()
        };

        let table_view = TableView::new_view(
            packed_file_view.get_mut_widget(),
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui,
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
        Ok(packed_file_info)
    }

    pub fn get_ref_table(&self) ->&TableView {
        &self.table_view
    }
}
