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
Module with all the code for managing the view for AnimFragment PackedFiles.
!*/

use getset::Getters;
use qt_widgets::QSpinBox;
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
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackedFileAnimFragmentView {
    table_view: Arc<TableView>,
    integer_label_1: QBox<QLabel>,
    integer_label_2: QBox<QLabel>,
    integer_1: QBox<QSpinBox>,
    integer_2: QBox<QSpinBox>,

    packed_file_path: Arc<RwLock<String>>,

    #[getset(skip)]
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

            let i1_label = QLabel::from_q_string_q_widget(&QString::from_std_str(data.skeleton_1()), packed_file_view.get_mut_widget());
            let i2_label = QLabel::from_q_string_q_widget(&QString::from_std_str(data.skeleton_2()), packed_file_view.get_mut_widget());

            let integer_1 = QSpinBox::new_1a(packed_file_view.get_mut_widget());
            let integer_2 = QSpinBox::new_1a(packed_file_view.get_mut_widget());
            integer_1.set_value(*data.min_id());
            integer_2.set_value(*data.max_id());

            let table = QWidget::new_1a(packed_file_view.get_mut_widget());
            let layout_1 = QGridLayout::new_1a(&table);
            table.set_layout(&layout_1);

            layout.add_widget_5a(&i1_label, 0, 0, 1, 1);
            layout.add_widget_5a(&i2_label, 1, 0, 1, 1);

            layout.add_widget_5a(&integer_1, 0, 1, 1, 1);
            layout.add_widget_5a(&integer_2, 1, 1, 1, 1);

            layout.add_widget_5a(&table, 0, 2, 2, 1);

            let table_data = data.data()?.get(0).unwrap();
            let table_view = TableView::new_view(
                &table,
                app_ui,
                global_search_ui,
                pack_file_contents_ui,
                diagnostics_ui,
                dependencies_ui,
                references_ui,
                TableType::AnimFragment(data),
                None,
                packed_file_view.data_source.clone()
            )?;

            let packed_file_table_view = Arc::new(Self {
                table_view,
                integer_label_1: i1_label,
                integer_label_2: i2_label,
                integer_1,
                integer_2,

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
            Ok(())
        }
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: AnimFragment) -> Result<()> {

        // Update the stored definition.
        //let definition = data.definition();
        //*self.definition.write().unwrap() = definition.clone();

        // Load the data to the view itself.
        self.load_data(&data)
    }

    /// This function takes care of loading the data into the AnimFragment View.
    pub unsafe fn load_data(&self, original_data: &AnimFragment) -> Result<()> {
        self.integer_label_1.set_text(&QString::from_std_str(original_data.skeleton_1()));
        self.integer_label_2.set_text(&QString::from_std_str(original_data.skeleton_2()));

        self.integer_1.set_value(*original_data.min_id());
        self.integer_2.set_value(*original_data.max_id());

        // Each table view, we just load them.
        //if let Some(DecodedData::SequenceU32(data)) = data.get(0) {
        //    self.table_view_1.reload_view(TableType::AnimFragment(From::from(data.clone())));
        //}

        //if let Some(DecodedData::SequenceU32(data)) = data.get(3) {
        //    self.table_view_2.reload_view(TableType::AnimFragment(From::from(data.clone())));
        //}

        Ok(())
    }

    /// This function takes care of building a RFileDecoded from the view's data.
    pub unsafe fn save_data(&self) -> Result<RFileDecoded> {
        let mut table = AnimFragment::new(&self.definition());
        let mut data = vec![];
        let i1 = DecodedData::I32(self.integer_1.text().to_std_string().parse::<i32>()?);
        let i2 = DecodedData::I32(self.integer_2.text().to_std_string().parse::<i32>()?);

        let filter: QPtr<QSortFilterProxyModel> = self.table_view.table_view_ptr().model().static_downcast();
        let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let data_1 = get_table_from_view(&table_model, &self.table_view.table_definition())?;

        //data.push(DecodedData::SequenceU32(Box::new(data_1)));
        data.push(i1);
        data.push(i2);
        //data.push(DecodedData::SequenceU32(Box::new(data_2)));

        let data = vec![data; 1];
        //table.set_table_data(&data)?;
        Ok(RFileDecoded::AnimFragment(table))
    }

    /// This function returns a copy of the definition of this AnimFragment.
    pub fn definition(&self) -> Definition {
        self.table_view().table_definition().clone()
    }

    /// This function returns a copy of the datasource of this table.
    pub fn data_source(&self) -> DataSource {
        self.data_source.read().unwrap().clone()
    }
}
