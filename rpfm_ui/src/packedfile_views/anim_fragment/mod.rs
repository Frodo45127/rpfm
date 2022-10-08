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

use anyhow::{anyhow, Result};

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::{anim_fragment::AnimFragment, FileType, RFileDecoded, table::*};
use rpfm_lib::games::supported_games::KEY_WARHAMMER_2;
use rpfm_lib::schema::Definition;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::GAME_SELECTED;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::{DataSource, PackedFileView, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::views::debug::DebugView;
use crate::views::table::{TableView, TableType};
use crate::views::table::utils::get_table_from_view;

use self::slots::PackedFileAnimFragmentViewSlots;

mod connections;
mod slots;

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
    packed_file_path: Arc<RwLock<String>>,
    data_source: Arc<RwLock<DataSource>>,
}

/// This struct contains the debug view of an AnimFragment PackedFile.
pub struct PackedFileAnimFragmentDebugView {
    debug_view: Arc<DebugView>,
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
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        data: AnimFragment
    ) -> Result<()> {

        // For any other game, use the debug view.
        if GAME_SELECTED.read().unwrap().game_key_name() != KEY_WARHAMMER_2 {

            // For now just build a debug view.
            let debug_view = DebugView::new_view(
                packed_file_view.get_mut_widget(),
                RFileDecoded::AnimFragment(data),
                packed_file_view.get_path_raw(),
            )?;

            let packed_file_debug_view = PackedFileAnimFragmentDebugView {
                debug_view,
            };

            packed_file_view.view = ViewType::Internal(View::AnimFragmentDebug(Arc::new(packed_file_debug_view)));
            packed_file_view.packed_file_type = FileType::AnimFragment;

            // Return success.
            Ok(())
        }

        // For Wh2, use the fancy view.
        else {
            let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

            let i1_label = QLabel::from_q_string_q_widget(&QString::from_std_str(data.definition().fields_processed()[1].name()), packed_file_view.get_mut_widget());
            let i2_label = QLabel::from_q_string_q_widget(&QString::from_std_str(data.definition().fields_processed()[2].name()), packed_file_view.get_mut_widget());

            let i1_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(&data.data()?[0][1].data_to_string()), packed_file_view.get_mut_widget());
            let i2_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(&data.data()?[0][2].data_to_string()), packed_file_view.get_mut_widget());

            let table_1 = QWidget::new_1a(packed_file_view.get_mut_widget());
            let table_2 = QWidget::new_1a(packed_file_view.get_mut_widget());
            let layout_1 = QGridLayout::new_1a(&table_1);
            let layout_2 = QGridLayout::new_1a(&table_2);
            table_1.set_layout(&layout_1);
            table_2.set_layout(&layout_2);

            layout.add_widget_5a(&i1_label, 0, 0, 1, 1);
            layout.add_widget_5a(&i2_label, 1, 0, 1, 1);

            layout.add_widget_5a(&i1_line_edit, 0, 1, 1, 1);
            layout.add_widget_5a(&i2_line_edit, 1, 1, 1, 1);

            layout.add_widget_5a(&table_1, 0, 2, 2, 1);
            layout.add_widget_5a(&table_2, 2, 0, 1, 3);

            return Err(anyhow!("to fix later"));

            /*
            let table_data = data.data()?.get(0).unwrap();
            let table_data_1 = if let Some(DecodedData::SequenceU32(data)) = table_data.get(0) {
                Table::new(/* &rpfm_lib::schema::Definition */, /* &str */, /* bool */)//data.clone()
            } else { unimplemented!() };

            let table_data_2 = if let Some(DecodedData::SequenceU32(data)) = table_data.get(3) {
                Table::new(/* &rpfm_lib::schema::Definition */, /* &str */, /* bool */)//data.clone()
            } else { unimplemented!() };

            let table_view_1 = TableView::new_view(
                &table_1,
                app_ui,
                global_search_ui,
                pack_file_contents_ui,
                diagnostics_ui,
                dependencies_ui,
                references_ui,
                TableType::AnimFragment(From::from(table_data_1)),
                None,
                packed_file_view.data_source.clone()
            )?;

            let table_view_2 = TableView::new_view(
                &table_2,
                app_ui,
                global_search_ui,
                pack_file_contents_ui,
                diagnostics_ui,
                dependencies_ui,
                references_ui,
                TableType::AnimFragment(From::from(table_data_2)),
                None,
                packed_file_view.data_source.clone()
            )?;

            let packed_file_table_view = Arc::new(Self {
                table_view_1,
                table_view_2,
                integer_label_1: i1_label,
                integer_label_2: i2_label,
                integer_1: i1_line_edit,
                integer_2: i2_line_edit,

                definition: Arc::new(RwLock::new(data.definition().clone())),
                packed_file_path: packed_file_view.get_path_raw(),
                data_source: packed_file_view.data_source.clone(),
            });

            let packed_file_anim_fragment_view_slots = PackedFileAnimFragmentViewSlots::new(
                &packed_file_table_view,
                app_ui,
                pack_file_contents_ui,
                diagnostics_ui
            );

            connections::set_connections(&packed_file_table_view, &packed_file_anim_fragment_view_slots);
            packed_file_view.view = ViewType::Internal(View::AnimFragment(packed_file_table_view));
            packed_file_view.packed_file_type = FileType::AnimFragment;

            // Return success.
            Ok(())*/
        }
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: AnimFragment) -> Result<()> {

        // Update the stored definition.
        let definition = data.definition();
        *self.definition.write().unwrap() = definition.clone();

        // Load the data to the view itself.
        self.load_data(&data)
    }

    /// This function takes care of loading the data into the AnimFragment View.
    pub unsafe fn load_data(&self, original_data: &AnimFragment) -> Result<()> {
        match original_data.data()?.get(0) {
            Some(data) => {
                self.integer_label_1.set_text(&QString::from_std_str(original_data.definition().fields_processed()[1].name()));
                self.integer_label_2.set_text(&QString::from_std_str(original_data.definition().fields_processed()[2].name()));

                self.integer_1.set_text(&QString::from_std_str(&data[1].data_to_string()));
                self.integer_2.set_text(&QString::from_std_str(&data[2].data_to_string()));

                // Each table view, we just load them.
                //if let Some(DecodedData::SequenceU32(data)) = data.get(0) {
                //    self.table_view_1.reload_view(TableType::AnimFragment(From::from(data.clone())));
                //}

                //if let Some(DecodedData::SequenceU32(data)) = data.get(3) {
                //    self.table_view_2.reload_view(TableType::AnimFragment(From::from(data.clone())));
                //}

                Ok(())
            }
            None => Err(anyhow!("WTF did you do? Things broke.")),
        }
    }

    /// This function takes care of building a RFileDecoded from the view's data.
    pub unsafe fn save_data(&self) -> Result<RFileDecoded> {
        let mut table = AnimFragment::new(&self.get_definition());
        let mut data = vec![];
        let i1 = DecodedData::I32(self.integer_1.text().to_std_string().parse::<i32>()?);
        let i2 = DecodedData::I32(self.integer_2.text().to_std_string().parse::<i32>()?);

        let filter: QPtr<QSortFilterProxyModel> = self.table_view_1.table_view_primary_ptr().model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let data_1 = get_table_from_view(&table_model, &self.table_view_1.table_definition())?;

        let filter: QPtr<QSortFilterProxyModel> = self.table_view_2.table_view_primary_ptr().model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let data_2 = get_table_from_view(&table_model, &self.table_view_2.table_definition())?;

        //data.push(DecodedData::SequenceU32(Box::new(data_1)));
        data.push(i1);
        data.push(i2);
        //data.push(DecodedData::SequenceU32(Box::new(data_2)));

        let data = vec![data; 1];
        //table.set_table_data(&data)?;
        Ok(RFileDecoded::AnimFragment(table))
    }

    /// This function returns a copy of the definition of this AnimFragment.
    pub fn get_definition(&self) -> Definition {
        self.definition.read().unwrap().clone()
    }

    pub fn get_ref_table_view_2(&self) -> &TableView {
        &self.table_view_2
    }

    /// This function returns a copy of the datasource of this table.
    pub fn get_data_source(&self) -> DataSource {
        self.data_source.read().unwrap().clone()
    }
}
