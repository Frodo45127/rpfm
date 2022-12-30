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
Module with all the code related to the `ReferencesUI`.
!*/

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::QDockWidget;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QMainWindow;
use qt_widgets::QTableView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::{DockWidgetArea, Orientation, SortOrder};
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QSignalBlocker;
use qt_core::QVariant;
use qt_core::q_item_selection_model::SelectionFlag;

use cpp_core::Ptr;

use anyhow::Result;
use getset::Getters;

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::new_tableview_filter_safe;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::pack_tree::PackTree;
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::*;
use crate::utils::*;
use crate::UI_STATE;

pub mod connections;
pub mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/references_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/references_dock_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the References panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct ReferencesUI {

    //-------------------------------------------------------------------------------//
    // `References` Dock Widget.
    //-------------------------------------------------------------------------------//
    references_dock_widget: QPtr<QDockWidget>,
    references_table_view: QPtr<QTableView>,
    references_table_model: QBox<QStandardItemModel>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ReferencesUI {

    /// This function creates an entire `ReferencesUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let references_dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();
        let references_dock_inner_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "inner_widget")?;
        let references_table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "results_table_view")?;

        main_window.add_dock_widget_2a(DockWidgetArea::BottomDockWidgetArea, references_dock_widget.as_ptr());
        references_dock_widget.set_window_title(&qtr("gen_loc_references"));
        references_dock_widget.set_object_name(&QString::from_std_str("references_dock"));

        let references_table_filter = new_tableview_filter_safe(references_dock_inner_widget.static_upcast());
        let references_table_model = QStandardItemModel::new_1a(&references_dock_inner_widget);
        references_table_filter.set_source_model(&references_table_model);
        references_table_view.set_model(&references_table_filter);

        if setting_bool("tight_table_mode") {
            references_table_view.vertical_header().set_minimum_section_size(22);
            references_table_view.vertical_header().set_maximum_section_size(22);
            references_table_view.vertical_header().set_default_section_size(22);
        }

        Ok(Self {

            //-------------------------------------------------------------------------------//
            // `References` Dock Widget.
            //-------------------------------------------------------------------------------//
            references_dock_widget,
            references_table_view,
            references_table_model,
        })
    }

    /// This function takes care of loading the results of a reference search into the table.
    pub unsafe fn load_references_to_ui(&self, references: Vec<(DataSource, String, String, usize, usize)>) {

        // First, clean the current diagnostics.
        self.references_table_model.clear();

        if !references.is_empty() {
            let blocker = QSignalBlocker::from_q_object(&self.references_table_model);
            for (index, (data_source, path, column_name, column_number, row_number)) in references.iter().enumerate() {

                // Unlock in the last step.
                if index == references.len() - 1 {
                    blocker.unblock();
                }

                let qlist_boi = QListOfQStandardItem::new();

                // Create an empty row.
                let data_source_item = QStandardItem::new();
                let path_item = QStandardItem::new();
                let column_name_item = QStandardItem::new();
                let column_number_item = QStandardItem::new();
                let row_number_item = QStandardItem::new();

                data_source_item.set_text(&QString::from_std_str(format!("{}", data_source)));
                path_item.set_text(&QString::from_std_str(path));
                column_name_item.set_text(&QString::from_std_str(column_name));
                column_number_item.set_data_2a(&QVariant::from_int(*column_number as i32), 2);
                column_number_item.set_data_1a(&QVariant::from_int(*column_number as i32));
                row_number_item.set_data_2a(&QVariant::from_int(*row_number as i32), 2);
                row_number_item.set_data_1a(&QVariant::from_int(*row_number as i32));

                data_source_item.set_editable(false);
                path_item.set_editable(false);
                column_name_item.set_editable(false);
                column_number_item.set_editable(false);
                row_number_item.set_editable(false);

                // Add an empty row to the list.
                qlist_boi.append_q_standard_item(&data_source_item.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&path_item.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&column_name_item.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&column_number_item.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&row_number_item.into_ptr().as_mut_raw_ptr());

                // Append the new row.
                self.references_table_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
            }

            self.references_table_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("reference_search_data_source")));
            self.references_table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("reference_search_path")));
            self.references_table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("reference_search_column_name")));
            self.references_table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("reference_search_column_number")));
            self.references_table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("reference_search_row_number")));

            // Hide the column number column for tables.
            self.references_table_view.hide_column(3);
            self.references_table_view.sort_by_column_2a(1, SortOrder::AscendingOrder);

            self.references_table_view.horizontal_header().set_stretch_last_section(true);
            self.references_table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    pub unsafe fn open_match(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<Self>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());
        let row = model_index.row();

        let reference_data_source = DataSource::from(&*model.item_2a(row, 0).text().to_std_string());
        let reference_path = model.item_2a(row, 1).text().to_std_string();
        let reference_column_number = model.item_2a(row, 3).data_0a().to_int_0a();
        let reference_row_number = model.item_2a(row, 4).data_0a().to_int_0a();

        match reference_data_source {
            DataSource::PackFile => {
                let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&reference_path, reference_data_source);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(pack_file_contents_ui.packfile_contents_tree_view().static_upcast::<QObject>());
                        pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
            },
            DataSource::ParentFiles |
            DataSource::AssKitFiles |
            DataSource::GameFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&reference_path, reference_data_source);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
            },
            DataSource::ExternalFile => {},
        }

        // Open the table and select the cell.
        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui,Some(reference_path.to_owned()), true, false, reference_data_source);
        if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == reference_path && x.get_data_source() == reference_data_source) {
            if let ViewType::Internal(View::Table(view)) = packed_file_view.get_view() {
                let table_view = view.get_ref_table();
                let table_view = table_view.table_view_ptr();
                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                let table_selection_model = table_view.selection_model();

                let table_model_index = table_model.index_2a(reference_row_number, reference_column_number);
                let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                if table_model_index_filtered.is_valid() {
                    table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                    table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                }
            }
        }
    }
}
