//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the view for AnimFragment PackedFiles.
!*/

use qt_widgets::QWidget;
use qt_widgets::QLineEdit;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QString;
use qt_core::QSortFilterProxyModel;
use qt_core::QPtr;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_error::{Result, ErrorKind};
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::DecodedPackedFile;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packedfile::table::anim_fragment::AnimFragment;
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::schema::Definition;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::views::table::{TableView, TableType};
use crate::views::table::utils::get_table_from_view;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of an AnimFragment PackedFile.
pub struct PackedFileAnimFragmentView {
    table_view_1: Arc<TableView>,
    table_view_2: Arc<TableView>,
    integer_label_1: QBox<QLabel>,
    integer_label_2: QBox<QLabel>,
    integer_1: QBox<QLineEdit>,
    integer_2: QBox<QLineEdit>,

    definition: Arc<RwLock<Definition>>,
    //packed_file_path: Arc<RwLock<Vec<String>>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileAnimFragmentView`.
impl PackedFileAnimFragmentView {

    /// This function creates a new AnimFraagment View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>
    ) -> Result<PackedFileInfo> {

        // Get the decoded Table.
        if packed_file_view.get_ref_path().is_empty() { CENTRAL_COMMAND.send_message_qt(Command::GetDependencyPackFilesList); }
        else { CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path())); }

        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::AnimFragmentPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        let i1_label = QLabel::from_q_string(&QString::from_std_str(data.get_ref_definition().get_fields_processed()[1].get_name()));
        let i2_label = QLabel::from_q_string(&QString::from_std_str(data.get_ref_definition().get_fields_processed()[2].get_name()));

        let i1_line_edit = QLineEdit::from_q_string(&QString::from_std_str(&data.get_ref_table_data()[0][1].data_to_string()));
        let i2_line_edit = QLineEdit::from_q_string(&QString::from_std_str(&data.get_ref_table_data()[0][2].data_to_string()));

        let table_1 = QWidget::new_0a();
        let table_2 = QWidget::new_0a();
        let layout_1 = QGridLayout::new_0a();
        let layout_2 = QGridLayout::new_0a();
        table_1.set_layout(layout_1.into_ptr());
        table_2.set_layout(layout_2.into_ptr());

        layout.add_widget_5a(&i1_label, 0, 0, 1, 1);
        layout.add_widget_5a(&i2_label, 1, 0, 1, 1);

        layout.add_widget_5a(&i1_line_edit, 0, 1, 1, 1);
        layout.add_widget_5a(&i2_line_edit, 1, 1, 1, 1);

        layout.add_widget_5a(&table_1, 0, 2, 2, 1);
        layout.add_widget_5a(&table_2, 2, 0, 1, 3);

        let table_data = data.get_ref_table_data().get(0).unwrap();
        let table_data_1 = if let Some(data) = table_data.get(0) {
            if let DecodedData::SequenceU32(data) = data { data.clone() } else { unimplemented!() }
        } else { unimplemented!() };

        let table_data_2 = if let Some(data) = table_data.get(3) {
            if let DecodedData::SequenceU32(data) = data { data.clone() } else { unimplemented!() }
        } else { unimplemented!() };

        let table_view_1 = TableView::new_view(
            &table_1,
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            TableType::AnimFragment(From::from(table_data_1)),
            None,
        )?;

        let table_view_2 = TableView::new_view(
            &table_2,
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            TableType::AnimFragment(From::from(table_data_2)),
            None,
        )?;

        let packed_file_table_view = Self {
            table_view_1,
            table_view_2,
            integer_label_1: i1_label,
            integer_label_2: i2_label,
            integer_1: i1_line_edit,
            integer_2: i2_line_edit,

            definition: Arc::new(RwLock::new(data.get_definition())),
        };

        packed_file_view.view = ViewType::Internal(View::AnimFragment(Arc::new(packed_file_table_view)));
        packed_file_view.packed_file_type = PackedFileType::AnimFragment;

        // Return success.
        Ok(packed_file_info)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: AnimFragment) -> Result<()> {

        // Update the stored definition.
        let definition = data.get_definition();
        *self.definition.write().unwrap() = definition;

        // Load the data to the view itself.
        self.load_data(&data)
    }

    /// This function takes care of loading the data into the AnimFragment View.
    pub unsafe fn load_data(&self, original_data: &AnimFragment) -> Result<()> {
        match original_data.get_table_data().get(0) {
            Some(data) => {
                self.integer_label_1.set_text(&QString::from_std_str(original_data.get_ref_definition().get_fields_processed()[1].get_name()));
                self.integer_label_2.set_text(&QString::from_std_str(original_data.get_ref_definition().get_fields_processed()[2].get_name()));

                self.integer_1.set_text(&QString::from_std_str(&data[1].data_to_string()));
                self.integer_2.set_text(&QString::from_std_str(&data[2].data_to_string()));

                // Each table view, we just load them.
                if let Some(data) = data.get(0) {
                    if let DecodedData::SequenceU32(data) = data {
                        self.table_view_1.reload_view(TableType::AnimFragment(From::from(data.clone())));
                    }
                }

                if let Some(data) = data.get(3) {
                    if let DecodedData::SequenceU32(data) = data {
                        self.table_view_2.reload_view(TableType::AnimFragment(From::from(data.clone())));
                    }
                }

                Ok(())
            }
            None => Err(ErrorKind::Generic.into()),
        }
    }

    /// This function takes care of building a DecodedPackedFile from the view's data.
    pub unsafe fn save_data(&self) -> Result<DecodedPackedFile> {
        let mut table = AnimFragment::new(&self.get_definition());
        let mut data = vec![];
        let i1 = DecodedData::I32(self.integer_1.text().to_std_string().parse::<i32>()?);
        let i2 = DecodedData::I32(self.integer_2.text().to_std_string().parse::<i32>()?);

        let filter: QPtr<QSortFilterProxyModel> = self.table_view_1.get_mut_ptr_table_view_primary().model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let data_1 = get_table_from_view(&table_model, &self.table_view_1.get_ref_table_definition())?;

        let filter: QPtr<QSortFilterProxyModel> = self.table_view_2.get_mut_ptr_table_view_primary().model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let data_2 = get_table_from_view(&table_model, &self.table_view_2.get_ref_table_definition())?;

        data.push(DecodedData::SequenceU32(data_1));
        data.push(i1);
        data.push(i2);
        data.push(DecodedData::SequenceU32(data_2));

        let data = vec![data; 1];
        table.set_table_data(&data)?;
        Ok(DecodedPackedFile::AnimFragment(table))
    }

    /// This function returns a copy of the definition of this AnimFragment.
    pub fn get_definition(&self) -> Definition {
        self.definition.read().unwrap().clone()
    }
}
