//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!
//! Module with all the code for managing the view for RigidModel files.
//!
//! This module loads the rigidmodel editor in a editable view. No 3d modeling here.

use qt_widgets::q_abstract_item_view::{SelectionBehavior, SelectionMode};
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QSpinBox;
use qt_widgets::QGridLayout;
use qt_widgets::QSplitter;
use qt_widgets::QTreeView;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QBox;
use qt_core::QEventLoop;
use qt_core::QItemSelection;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;

use anyhow::Result;
use getset::*;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::FileType;
use rpfm_lib::files::rigidmodel::RigidModel;

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::{find_widget, load_template};
#[cfg(feature = "support_model_renderer")] use rpfm_ui_common::settings::setting_bool;
#[cfg(feature = "support_model_renderer")] use rpfm_ui_common::utils::show_dialog;

use crate::packedfile_views::{AppUI, DataSource, FileView, PackFileContentsUI, utils::set_modified, View, ViewType};

use self::slots::RigidModelSlots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/rigid_model_editor.ui";
const VIEW_RELEASE: &str = "ui/rigid_model_editor.ui";

const DATA_INDEX: i32 = 20;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a RigidModel PackedFile.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct RigidModelView {
    path: Arc<RwLock<String>>,
    data_source: Arc<RwLock<DataSource>>,
    data: Arc<RwLock<RigidModel>>,

    current_key: Arc<RwLock<Option<CppBox<QModelIndex>>>>,

    detailed_view_groupbox: QPtr<QGroupBox>,
    mesh_block_groupbox: QPtr<QGroupBox>,
    material_data_groupbox: QPtr<QGroupBox>,
    mesh_data_groupbox: QPtr<QGroupBox>,

    lod_tree_view: QPtr<QTreeView>,
    lod_tree_filter: QBox<QSortFilterProxyModel>,
    lod_tree_model: QBox<QStandardItemModel>,

    visibility_spinbox: QPtr<QDoubleSpinBox>,
    lod_number_spinbox: QPtr<QSpinBox>,
    quality_level_spinbox: QPtr<QSpinBox>,

    #[cfg(feature = "support_model_renderer")] renderer: QBox<QWidget>,
    #[cfg(feature = "support_model_renderer")] renderer_enabled: bool,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `RigidModelView`.
impl RigidModelView {

    /// This function creates a new RigidModel View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &RigidModel,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        let splitter = QSplitter::from_q_widget(file_view.main_widget());
        layout.add_widget_5a(&splitter, 0, 0, 1, 1);
        splitter.add_widget(&main_widget);

        #[cfg(feature = "support_model_renderer")] let mut renderer_enabled = false;

        // TreeView and groupboxes.
        let lod_tree_view: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "lod_tree_view")?;
        let detailed_view_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "detailed_view_groupbox")?;
        let mesh_block_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "mesh_block_groupbox")?;
        let material_data_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "material_data_groupbox")?;
        let mesh_data_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "mesh_data_groupbox")?;
        detailed_view_groupbox.set_title(&qtr("rigid_model_editor_detailed_view_title"));
        mesh_block_groupbox.set_title(&qtr("rigid_model_editor_mesh_block_title"));
        material_data_groupbox.set_title(&qtr("rigid_model_editor_material_data_title"));
        mesh_data_groupbox.set_title(&qtr("rigid_model_editor_mesh_data_title"));

        // Lod data.
        let visibility_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "visibility_label")?;
        let lod_number_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "lod_number_label")?;
        let quality_level_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "quality_level_label")?;
        let visibility_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "visibility_spinbox")?;
        let lod_number_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "lod_number_spinbox")?;
        let quality_level_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "quality_level_spinbox")?;
        visibility_label.set_text(&qtr("rigid_model_editor_visibility"));
        lod_number_label.set_text(&qtr("rigid_model_editor_lod_number"));
        quality_level_label.set_text(&qtr("rigid_model_editor_quality_level"));

        // Extra stuff.
        let lod_tree_filter = QSortFilterProxyModel::new_1a(&lod_tree_view);
        let lod_tree_model = QStandardItemModel::new_1a(&lod_tree_filter);
        lod_tree_view.set_model(&lod_tree_filter);
        lod_tree_filter.set_source_model(&lod_tree_model);

        lod_tree_view.set_selection_mode(SelectionMode::SingleSelection);
        lod_tree_view.set_selection_behavior(SelectionBehavior::SelectRows);
        lod_tree_view.set_header_hidden(true);

        detailed_view_groupbox.set_enabled(false);
        mesh_block_groupbox.set_enabled(false);
        material_data_groupbox.set_enabled(false);
        mesh_data_groupbox.set_enabled(false);

        let view = Arc::new(Self{
            path: file_view.path_raw(),
            data_source: Arc::new(RwLock::new(file_view.data_source())),
            data: Arc::new(RwLock::new(data.clone())),

            current_key: Arc::new(RwLock::new(None)),

            detailed_view_groupbox,
            mesh_block_groupbox,
            material_data_groupbox,
            mesh_data_groupbox,

            lod_tree_view,
            lod_tree_filter,
            lod_tree_model,

            visibility_spinbox,
            lod_number_spinbox,
            quality_level_spinbox,

            #[cfg(feature = "support_model_renderer")] renderer: {
                if setting_bool("enable_renderer") {
                    match create_q_rendering_widget(&mut file_view.main_widget().as_ptr()) {
                        Ok(renderer) => {

                            // We need to manually pause the renderer or it'll keep lagging the UI.
                            let mut e_data = vec![];
                            data.clone().encode(e_data, None);
                            if let Err(error) = add_new_primary_asset(&renderer.as_ptr(), &file_view.path().read().unwrap(), e_data) {
                                show_dialog(file_view.main_widget(), error, false);
                                pause_rendering(&renderer.as_ptr());
                            }

                            renderer_enabled = true;
                            renderer.size_policy().set_horizontal_stretch(1);
                            splitter.add_widget(&renderer);
                            renderer
                        }
                        Err(error) => {
                            show_dialog(file_view.main_widget(), error, false);
                            QWidget::new_1a(file_view.main_widget())
                        }
                    }
                } else {
                    QWidget::new_1a(file_view.main_widget())
                }
            },
            #[cfg(feature = "support_model_renderer")] renderer_enabled,
        });

        view.load_data()?;

        let slots = RigidModelSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        file_view.file_type = FileType::RigidModel;
        file_view.view_type = ViewType::Internal(View::RigidModel(view));

        Ok(())
    }

    /// This function loads the data into the view, so it can be accessed in the UI.
    unsafe fn load_data(&self) -> Result<()> {
        self.lod_tree_model.clear();

        let data = self.data().read().unwrap();
        for (index, lod) in data.lods().iter().enumerate() {
            let item = QStandardItem::from_q_string(&QString::from_std_str("Lod ".to_string() + &index.to_string())).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(format!("{index}"))), DATA_INDEX);
            item.set_editable(false);

            for (subindex, _) in lod.mesh_blocks().iter().enumerate() {
                let sub_item = QStandardItem::from_q_string(&QString::from_std_str("Mesh Block ".to_string() + &subindex.to_string())).into_ptr();
                sub_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(format!("{index}-{subindex}"))), DATA_INDEX);
                sub_item.set_editable(false);
                item.append_row_q_standard_item(sub_item);
            }

            self.lod_tree_model.append_row_q_standard_item(item);
        }

        self.lod_tree_view().expand_all();

        Ok(())
    }

    /// Function to save the view and encode it into a RigidModel struct.
    pub unsafe fn save_view(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<RigidModel> {
        self.change_selected_row(None, None, app_ui, pack_file_contents_ui);

        let data = self.data.read().unwrap().clone();
        Ok(data)
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &RigidModel) -> Result<()> {
        self.clear_selected_field_data();

        self.lod_tree_model.clear();
        *self.data.write().unwrap() = data.clone();

        self.load_data()?;

        #[cfg(feature = "support_model_renderer")] {
            if let Some(ref path) = self.path {
                if self.renderer_enabled {
                    let mut e_data = vec![];
                    self.data.read().unwrap().clone().encode(e_data, None);
                    let _ = add_new_primary_asset(&self.renderer.as_ptr(), &path.read().unwrap(), e_data);
                }
            }
        }

        Ok(())
    }

    /// This function loads the data of a lod into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: &CppBox<QModelIndex>) {
        let key_item = self.lod_tree_model().item_from_index(index);
        let index_str = key_item.data_1a(DATA_INDEX).to_string().to_std_string();

        let data = self.data().read().unwrap();
        let (lod, mesh) = if index_str.contains("-") {
            let indexes = index_str.split("-").collect::<Vec<_>>();
            let lod_index = indexes[0].parse::<usize>().unwrap();
            let mesh_index = indexes[1].parse::<usize>().unwrap();

            let lod = &data.lods()[lod_index];
            let mesh = &lod.mesh_blocks()[mesh_index];
            (lod, Some(mesh))
        } else {
            let lod_index = index_str.parse::<usize>().unwrap();
            let lod = &data.lods()[lod_index];
            (lod, None)
        };

        self.visibility_spinbox.set_value(*lod.visibility_distance() as f64);
        self.lod_number_spinbox.set_value(*lod.authored_lod_number() as i32);
        self.quality_level_spinbox.set_value(*lod.quality_level() as i32);

        if let Some(mesh) = mesh {
            //TODO
        }

        // Re-enable this, as it's disabled on changing row.
        self.detailed_view_groupbox.set_enabled(true);
    }

    /// This function saves the data from the detailed view to the backend object.
    pub unsafe fn save_from_detailed_view(&self, old_key_index: &CppBox<QModelIndex>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if let DataSource::PackFile = *self.data_source.read().unwrap() {
            let key_item = self.lod_tree_model().item_from_index(old_key_index);
            let index_str = key_item.data_1a(DATA_INDEX).to_string().to_std_string();

            let mut data = self.data().write().unwrap();
            let (lod, mesh_index) = if index_str.contains("-") {
                let indexes = index_str.split("-").collect::<Vec<_>>();
                let lod_index = indexes[0].parse::<usize>().unwrap();
                let mesh_index = indexes[1].parse::<usize>().unwrap();

                (&mut data.lods_mut()[lod_index], Some(mesh_index))
            } else {
                let lod_index = index_str.parse::<usize>().unwrap();
                (&mut data.lods_mut()[lod_index], None)
            };

            lod.set_visibility_distance(self.visibility_spinbox().value() as f32);
            lod.set_authored_lod_number(self.lod_number_spinbox().value() as u32);
            lod.set_quality_level(self.quality_level_spinbox().value() as u32);

            if let Some(mesh_index) = mesh_index {
                // TODO
            }

            // As we don't use the list itself to store the data, we use this instead of a modified slot to mark the file as modified.
            set_modified(true, &self.path.read().unwrap(), app_ui, pack_file_contents_ui);
        }
    }

    pub unsafe fn change_selected_row(&self, new_index: Option<CppBox<QModelIndex>>, sibling_mode: Option<bool>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let is_generic_sel_change = new_index.is_some();
        self.detailed_view_groupbox().set_enabled(false);

        let event_loop = QEventLoop::new_0a();
        event_loop.process_events_0a();

        // If we have items in the table, try to figure the next one. If we don't have the current one visible,
        // default to the first/last item, depending on the direction we're moving.
        if self.lod_tree_filter().row_count_0a() > 0 {
            let mut current_index = self.current_key.write().unwrap();
            let new_index = if new_index.is_some() {
                new_index
            } else if let Some(next) = sibling_mode {
                match *current_index {
                    Some(ref index) => {
                        let current_index_filtered = self.lod_tree_filter().map_from_source(index);
                        if current_index_filtered.is_valid() {
                            let new_row = if next {
                                current_index_filtered.row() + 1
                            } else {
                                current_index_filtered.row() - 1
                            };

                            let new_index_filtered = current_index_filtered.sibling_at_row(new_row);
                            if new_index_filtered.is_valid() {
                                let new_index = self.lod_tree_filter().map_to_source(&new_index_filtered);
                                Some(new_index)
                            } else {
                                None
                            }
                        } else {
                            let new_index_filtered = if next {
                                self.lod_tree_filter().index_2a(0, 0)
                            } else {
                                self.lod_tree_filter().index_2a(self.lod_tree_filter().row_count_0a() - 1, 0)
                            };

                            let new_index = self.lod_tree_filter().map_to_source(&new_index_filtered);
                            Some(new_index)
                        }
                    }

                    None => {
                        let new_index_filtered = if next {
                            self.lod_tree_filter().index_2a(0, 0)
                        } else {
                            self.lod_tree_filter().index_2a(self.lod_tree_filter().row_count_0a() - 1, 0)
                        };

                        let new_index = self.lod_tree_filter().map_to_source(&new_index_filtered);
                        Some(new_index)
                    }
                }
            } else {
                None
            };

            // Handle the selection change.
            match *current_index {
                Some(ref current_index) => self.save_from_detailed_view(current_index, app_ui, pack_file_contents_ui),
                None => self.clear_selected_field_data(),
            }

            match new_index {
                Some(ref new_index) => self.load_to_detailed_view(new_index),
                None => self.clear_selected_field_data(),
            }

            *current_index = new_index;

            // If we're not changing the index due to a selection change, manually move the selected line.
            if !is_generic_sel_change {

                // Make sure to block the signals before switching the selection, or it'll trigger this twice.
                self.lod_tree_view().selection_model().block_signals(true);
                let sel_model = self.lod_tree_view().selection_model();
                sel_model.clear();

                if let Some(ref index) = *current_index {
                    let filter_index = self.lod_tree_filter().map_from_source(index);
                    if filter_index.is_valid() {
                        let col_count = self.lod_tree_model().column_count_0a();
                        let end_index = filter_index.sibling_at_column(col_count - 1);
                        let new_selection = QItemSelection::new_2a(&filter_index, &end_index);

                        // This triggers a save of the editing item.
                        sel_model.select_q_item_selection_q_flags_selection_flag(&new_selection, SelectionFlag::Toggle.into());
                    }
                }

                self.lod_tree_view().selection_model().block_signals(false);
                self.lod_tree_view().viewport().update();
            }
        }

        else {
            let mut current_index = self.current_key.write().unwrap();
            match *current_index {
                Some(ref current_index) => self.save_from_detailed_view(current_index, app_ui, pack_file_contents_ui),
                None => self.clear_selected_field_data(),
            }

            *current_index = None;
        }
    }

    unsafe fn clear_selected_field_data(&self) {
        self.visibility_spinbox.clear();
        self.lod_number_spinbox.clear();
        self.quality_level_spinbox.clear();
    }
}
