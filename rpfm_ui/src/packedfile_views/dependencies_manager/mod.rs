//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

use anyhow::Result;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_ipc::helpers::RFileInfo;

use rpfm_lib::files::table::DecodedData;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{FileView, View, ViewType};
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
        file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Result<Option<RFileInfo>> {

        // Get the decoded Table.
        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::GetDependencyPackFilesList);
        let response = CentralCommand::recv(&receiver);
        let table_data = match response {
            Response::VecBoolString(table) => TableType::DependencyManager(table.iter()
                .map(|(hard, pack)| vec![DecodedData::Boolean(hard.to_owned()), DecodedData::StringU8(pack.to_owned())])
                .collect::<Vec<Vec<DecodedData>>>()),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        let table_view = TableView::new_view(
            file_view.main_widget(),
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui,
            references_ui,
            table_data,
            Some(file_view.path_raw()),
            file_view.data_source.clone()
        )?;

        let dependencies_manager_view = Self {
            table_view,
        };

        file_view.view_type = ViewType::Internal(View::DependenciesManager(Arc::new(dependencies_manager_view)));

        // Return success.
        Ok(None)
    }

    pub fn get_ref_table(&self) ->&TableView {
        &self.table_view
    }
}
