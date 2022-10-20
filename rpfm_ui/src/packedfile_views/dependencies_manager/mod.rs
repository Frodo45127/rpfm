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
Module with all the code for managing the view for the Dependencies Manager.
!*/

use std::sync::Arc;

use std::rc::Rc;

use anyhow::Result;

use rpfm_lib::files::{FileType, table::DecodedData};
use crate::backend::RFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
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

/// This struct contains pointers to all the widgets needed for the view.
pub struct DependenciesManagerView {
    table_view: Arc<TableView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `DependenciesManagerView`.
impl DependenciesManagerView {

    /// This function creates a new `DependenciesManagerView`, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Result<Option<RFileInfo>> {

        // Get the decoded Table.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetDependencyPackFilesList);
        let response = CentralCommand::recv(&receiver);
        let table_data = match response {
            Response::VecString(table) => TableType::DependencyManager(table.iter().map(|x| vec![DecodedData::StringU8(x.to_owned()); 1]).collect::<Vec<Vec<DecodedData>>>()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
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

        let dependencies_manager_view = Self {
            table_view,
        };

        packed_file_view.view = ViewType::Internal(View::DependenciesManager(Arc::new(dependencies_manager_view)));
        //packed_file_view.packed_file_type = PackedFileType::DependencyPackFilesList;

        // Return success.
        Ok(None)
    }

    pub fn get_ref_table(&self) ->&TableView {
        &self.table_view
    }
}
