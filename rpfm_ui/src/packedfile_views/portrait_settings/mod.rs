//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the code to manage a Portrait Settings View.

use qt_widgets::QAction;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QMenu;
use qt_widgets::QSpinBox;
use qt_widgets::QWidget;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::SlotOfQString;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SortOrder;

use cpp_core::Ref;

use anyhow::Result;
use getset::*;
use qt_widgets::q_dialog_button_box::StandardButton;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_lib::files::{FileType, portrait_settings::*};

use crate::app_ui::AppUI;
use crate::ffi::*;
use crate::locale::{qtr, tr};
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::utils::*;
use self::slots::PortraitSettingsSlots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/portrait_settings_editor.ui";
const VIEW_RELEASE: &str = "ui/portrait_settings_editor.ui";

const NAME_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/portrait_settings_name_dialog.ui";
const NAME_VIEW_RELEASE: &str = "ui/portrait_settings_name_dialog.ui";

const DATA: i32 = 20;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a Portrait Setting File.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PortraitSettingsView {
    version: u32,
    detailed_view_widget: QPtr<QWidget>,
    body_camera_settings_groupbox: QPtr<QGroupBox>,

    main_list_view: QPtr<QListView>,
    main_list_filter: QBox<QSortFilterProxyModel>,
    main_list_model: QBox<QStandardItemModel>,
    main_filter_line_edit: QPtr<QLineEdit>,

    main_distance_spinbox: QPtr<QDoubleSpinBox>,
    main_distance_1_spinbox: QPtr<QDoubleSpinBox>,
    main_distance_body_spinbox: QPtr<QSpinBox>,
    main_fov_spinbox: QPtr<QDoubleSpinBox>,
    main_phi_spinbox: QPtr<QDoubleSpinBox>,
    main_theta_spinbox: QPtr<QDoubleSpinBox>,

    body_distance_spinbox: QPtr<QDoubleSpinBox>,
    body_distance_1_spinbox: QPtr<QDoubleSpinBox>,
    body_distance_body_spinbox: QPtr<QSpinBox>,
    body_fov_spinbox: QPtr<QDoubleSpinBox>,
    body_phi_spinbox: QPtr<QDoubleSpinBox>,
    body_theta_spinbox: QPtr<QDoubleSpinBox>,

    variants_widget: QPtr<QWidget>,
    variants_list_view: QPtr<QListView>,
    variants_list_filter: QBox<QSortFilterProxyModel>,
    variants_list_model: QBox<QStandardItemModel>,
    variants_filter_line_edit: QPtr<QLineEdit>,
    file_diffuse_line_edit: QPtr<QLineEdit>,
    file_mask_1_line_edit: QPtr<QLineEdit>,
    file_mask_2_line_edit: QPtr<QLineEdit>,
    file_mask_3_line_edit: QPtr<QLineEdit>,

    main_list_context_menu: QBox<QMenu>,
    variants_list_context_menu: QBox<QMenu>,

    main_list_add: QPtr<QAction>,
    main_list_clone: QPtr<QAction>,
    main_list_delete: QPtr<QAction>,

    variants_list_add: QPtr<QAction>,
    variants_list_clone: QPtr<QAction>,
    variants_list_delete: QPtr<QAction>,

    timer_delayed_updates_main: QBox<QTimer>,
    timer_delayed_updates_variants: QBox<QTimer>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PortraitSettingsView {

    /// This function creates a new Portrait Settings View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut PackedFileView,
        data: &mut PortraitSettings,
        app_ui: &Rc<AppUI>,
    ) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.get_mut_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.get_mut_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        // ListView and groupboxes.
        let main_list_view: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "main_list_view")?;
        let main_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "main_filter_line_edit")?;
        let detailed_view_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "detailed_view_widget")?;
        let main_camera_settings_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "main_camera_settings_groupbox")?;
        let body_camera_settings_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "body_camera_settings_groupbox")?;
        let variants_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "variants_groupbox")?;
        let variants_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "variants_widget")?;
        main_camera_settings_groupbox.set_title(&qtr("portrait_settings_main_camera_settings_title"));
        body_camera_settings_groupbox.set_title(&qtr("portrait_settings_body_camera_settings_title"));
        variants_groupbox.set_title(&qtr("portrait_settings_variants_title"));
        main_filter_line_edit.set_placeholder_text(&qtr("portrait_settings_filter"));

        // Main camera.
        let main_distance_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_distance_label")?;
        let main_distance_1_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_distance_1_label")?;
        let main_distance_body_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_distance_body_label")?;
        let main_fov_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_fov_label")?;
        let main_phi_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_phi_label")?;
        let main_theta_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "main_theta_label")?;
        main_distance_label.set_text(&qtr("portrait_settings_main_distance"));
        main_distance_1_label.set_text(&qtr("portrait_settings_main_distance_1_label"));
        main_distance_body_label.set_text(&qtr("portrait_settings_main_distance_body_label"));
        main_fov_label.set_text(&qtr("portrait_settings_main_fov_label"));
        main_phi_label.set_text(&qtr("portrait_settings_main_phi_label"));
        main_theta_label.set_text(&qtr("portrait_settings_main_theta_label"));

        let main_distance_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "main_distance_spinbox")?;
        let main_distance_1_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "main_distance_1_spinbox")?;
        let main_distance_body_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "main_distance_body_spinbox")?;
        let main_fov_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "main_fov_spinbox")?;
        let main_phi_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "main_phi_spinbox")?;
        let main_theta_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "main_theta_spinbox")?;

        // Body camera
        let body_distance_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_distance_label")?;
        let body_distance_1_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_distance_1_label")?;
        let body_distance_body_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_distance_body_label")?;
        let body_fov_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_fov_label")?;
        let body_phi_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_phi_label")?;
        let body_theta_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_theta_label")?;
        body_distance_label.set_text(&qtr("portrait_settings_body_distance"));
        body_distance_1_label.set_text(&qtr("portrait_settings_body_distance_1_label"));
        body_distance_body_label.set_text(&qtr("portrait_settings_body_distance_body_label"));
        body_fov_label.set_text(&qtr("portrait_settings_body_fov_label"));
        body_phi_label.set_text(&qtr("portrait_settings_body_phi_label"));
        body_theta_label.set_text(&qtr("portrait_settings_body_theta_label"));

        let body_distance_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_distance_spinbox")?;
        let body_distance_1_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_distance_1_spinbox")?;
        let body_distance_body_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "body_distance_body_spinbox")?;
        let body_fov_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_fov_spinbox")?;
        let body_phi_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_phi_spinbox")?;
        let body_theta_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_theta_spinbox")?;

        // Variants
        let variants_list_view: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "variants_list_view")?;
        let variants_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "variants_filter_line_edit")?;
        variants_filter_line_edit.set_placeholder_text(&qtr("portrait_settings_filter"));

        let file_diffuse_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "file_diffuse_label")?;
        let file_mask_1_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "file_mask_1_label")?;
        let file_mask_2_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "file_mask_2_label")?;
        let file_mask_3_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "file_mask_3_label")?;
        file_diffuse_label.set_text(&qtr("portrait_settings_file_diffuse_label"));
        file_mask_1_label.set_text(&qtr("portrait_settings_file_mask_1_label"));
        file_mask_2_label.set_text(&qtr("portrait_settings_file_mask_2_label"));
        file_mask_3_label.set_text(&qtr("portrait_settings_file_mask_3_label"));

        let file_diffuse_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "file_diffuse_line_edit")?;
        let file_mask_1_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "file_mask_1_line_edit")?;
        let file_mask_2_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "file_mask_2_line_edit")?;
        let file_mask_3_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "file_mask_3_line_edit")?;

        // Extra stuff.
        let main_list_filter = QSortFilterProxyModel::new_1a(&main_list_view);
        let main_list_model = QStandardItemModel::new_1a(&main_list_filter);
        main_list_view.set_model(&main_list_filter);
        main_list_filter.set_source_model(&main_list_model);

        let variants_list_filter = QSortFilterProxyModel::new_1a(&variants_list_view);
        let variants_list_model = QStandardItemModel::new_1a(&variants_list_filter);
        variants_list_view.set_model(&variants_list_filter);
        variants_list_filter.set_source_model(&variants_list_model);

        // Context menus.
        let main_list_context_menu = QMenu::from_q_widget(&main_list_view);
        let main_list_add = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "add", "context_menu_add", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        let main_list_clone = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "clone", "context_menu_clone", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        let main_list_delete = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "delete", "context_menu_delete", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        main_list_clone.set_enabled(false);
        main_list_delete.set_enabled(false);

        let variants_list_context_menu = QMenu::from_q_widget(&variants_list_view);
        let variants_list_add = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "add", "context_menu_add", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        let variants_list_clone = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "clone", "context_menu_clone", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        let variants_list_delete = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "portrait_settings", "delete", "context_menu_delete", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        variants_list_clone.set_enabled(false);
        variants_list_delete.set_enabled(false);

        // Filter timer.
        let timer_delayed_updates_main = QTimer::new_1a(main_widget.as_ptr());
        let timer_delayed_updates_variants = QTimer::new_1a(main_widget.as_ptr());
        timer_delayed_updates_main.set_single_shot(true);
        timer_delayed_updates_variants.set_single_shot(true);

        let view = Arc::new(Self{
            version: *data.version(),
            main_list_view,
            main_list_filter,
            main_list_model,
            main_filter_line_edit,
            detailed_view_widget,
            body_camera_settings_groupbox,

            main_distance_spinbox,
            main_distance_1_spinbox,
            main_distance_body_spinbox,
            main_fov_spinbox,
            main_phi_spinbox,
            main_theta_spinbox,

            body_distance_spinbox,
            body_distance_1_spinbox,
            body_distance_body_spinbox,
            body_fov_spinbox,
            body_phi_spinbox,
            body_theta_spinbox,

            variants_widget,
            variants_list_view,
            variants_list_filter,
            variants_list_model,
            variants_filter_line_edit,
            file_diffuse_line_edit,
            file_mask_1_line_edit,
            file_mask_2_line_edit,
            file_mask_3_line_edit,

            main_list_context_menu,
            variants_list_context_menu,

            main_list_add,
            main_list_clone,
            main_list_delete,

            variants_list_add,
            variants_list_clone,
            variants_list_delete,

            timer_delayed_updates_main,
            timer_delayed_updates_variants,
        });

        view.load_data(data)?;

        let slots = PortraitSettingsSlots::new(&view);
        connections::set_connections(&view, &slots);

        file_view.packed_file_type = FileType::PortraitSettings;
        file_view.view = ViewType::Internal(View::PortraitSettings(view));

        Ok(())
    }

    /// Function to save the view and encode it into a PortraitSettings struct.
    pub unsafe fn save_view(&self) -> PortraitSettings {

        // This saves whatever it's open to its item.
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.main_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let mut data = PortraitSettings::default();
        data.set_version(self.version);

        for row in 0..self.main_list_model.row_count_0a() {
            let index = self.main_list_model.index_2a(row, 0);
            let entry = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
            data.entries_mut().push(entry);
        }

        data
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &mut PortraitSettings) -> Result<()> {
        self.load_data(data)
    }

    /// This function loads the data into the view, so it can be accessed in the UI.
    unsafe fn load_data(&self, data: &mut PortraitSettings) -> Result<()> {
        self.main_list_model.clear();

        // Get them sorted so we have them in order for the UI.
        data.entries_mut().sort_by(|a, b| a.id().cmp(b.id()));
        for entry in data.entries() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(entry.id())).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&entry)?)), DATA);
            self.main_list_model.append_row_q_standard_item(item);
        }

        Ok(())
    }

    /// This function loads the data of an entry into the detailed view.
    pub unsafe fn load_entry_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first item loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.detailed_view_widget.is_enabled() {
            self.detailed_view_widget.set_enabled(true);
        }

        let mut data: Entry = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        self.main_distance_spinbox.set_value(*data.camera_settings_head().distance() as f64);
        self.main_distance_1_spinbox.set_value(*data.camera_settings_head().distance_1() as f64);
        self.main_distance_body_spinbox.set_value(*data.camera_settings_head().distance_body() as i32);
        self.main_fov_spinbox.set_value(*data.camera_settings_head().fov() as f64);
        self.main_phi_spinbox.set_value(*data.camera_settings_head().phi() as f64);
        self.main_theta_spinbox.set_value(*data.camera_settings_head().theta() as f64);

        match data.camera_settings_body() {
            Some(data) => {
                self.body_camera_settings_groupbox.set_checked(true);

                self.body_distance_spinbox.set_value(*data.distance() as f64);
                self.body_distance_1_spinbox.set_value(*data.distance_1() as f64);
                self.body_distance_body_spinbox.set_value(*data.distance_body() as i32);
                self.body_fov_spinbox.set_value(*data.fov() as f64);
                self.body_phi_spinbox.set_value(*data.phi() as f64);
                self.body_theta_spinbox.set_value(*data.theta() as f64);
                self.body_theta_spinbox.set_value(*data.theta() as f64);
            },
            None => {
                self.body_camera_settings_groupbox.set_checked(false);

                self.body_distance_spinbox.set_value(0.0);
                self.body_distance_1_spinbox.set_value(0.0);
                self.body_distance_body_spinbox.set_value(0);
                self.body_fov_spinbox.set_value(0.0);
                self.body_phi_spinbox.set_value(0.0);
                self.body_theta_spinbox.set_value(0.0);
                self.body_theta_spinbox.set_value(0.0);
            }
        }

        self.variants_list_model.clear();
        self.variants_widget.set_enabled(false);

        data.variants_mut().sort_by(|a, b| a.id().cmp(b.id()));
        for variant in data.variants() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(variant.id())).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&variant).unwrap())), DATA);
            self.variants_list_model.append_row_q_standard_item(item);
        }

        self.file_diffuse_line_edit.clear();
        self.file_mask_1_line_edit.clear();
        self.file_mask_2_line_edit.clear();
        self.file_mask_3_line_edit.clear();
    }

    /// This function loads the data of a variant into the variant detailed view.
    pub unsafe fn load_variant_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first item loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.variants_widget.is_enabled() {
            self.variants_widget.set_enabled(true);
        }

        let data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        self.file_diffuse_line_edit.set_text(&QString::from_std_str(data.file_diffuse()));
        self.file_mask_1_line_edit.set_text(&QString::from_std_str(data.file_mask_1()));
        self.file_mask_2_line_edit.set_text(&QString::from_std_str(data.file_mask_2()));
        self.file_mask_3_line_edit.set_text(&QString::from_std_str(data.file_mask_3()));
    }

    /// This function saves the data of an entry from the detailed view.
    pub unsafe fn save_entry_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: Entry = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        data.camera_settings_head_mut().set_distance(self.main_distance_spinbox.value() as f32);
        data.camera_settings_head_mut().set_distance_1(self.main_distance_1_spinbox.value() as f32);
        data.camera_settings_head_mut().set_distance_body(self.main_distance_body_spinbox.value() as u16);
        data.camera_settings_head_mut().set_fov(self.main_fov_spinbox.value() as f32);
        data.camera_settings_head_mut().set_phi(self.main_phi_spinbox.value() as f32);
        data.camera_settings_head_mut().set_theta(self.main_theta_spinbox.value() as f32);

        if self.body_camera_settings_groupbox.is_checked() {
            let mut body_camera_settings = CameraSetting::default();
            body_camera_settings.set_distance(self.body_distance_spinbox.value() as f32);
            body_camera_settings.set_distance_1(self.body_distance_1_spinbox.value() as f32);
            body_camera_settings.set_distance_body(self.body_distance_body_spinbox.value() as u16);
            body_camera_settings.set_fov(self.body_fov_spinbox.value() as f32);
            body_camera_settings.set_phi(self.body_phi_spinbox.value() as f32);
            body_camera_settings.set_theta(self.body_theta_spinbox.value() as f32);

            *data.camera_settings_body_mut() = Some(body_camera_settings);
        } else {
            *data.camera_settings_body_mut() = None;
        }

        // This saves whatever it's open in the variant list to its item.
        self.variants_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.variants_list_view.selection_model().selection(), SelectionFlag::Toggle.into());
        data.variants_mut().clear();

        for row in 0..self.variants_list_model.row_count_0a() {
            let index = self.variants_list_model.index_2a(row, 0);
            let variants_data = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
            data.variants_mut().push(variants_data);
        }

        self.main_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
    }

    /// This function saves the data of a variant from the detailed view.
    pub unsafe fn save_variant_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        data.set_file_diffuse(self.file_diffuse_line_edit.text().to_std_string());
        data.set_file_mask_1(self.file_mask_1_line_edit.text().to_std_string());
        data.set_file_mask_2(self.file_mask_2_line_edit.text().to_std_string());
        data.set_file_mask_3(self.file_mask_3_line_edit.text().to_std_string());

        self.variants_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
    }

    /// Function to trigger certain delayed actions, like the filter.
    pub unsafe fn start_delayed_updates_timer(timer: &Ref<QTimer>) {
        timer.set_interval(500);
        timer.start_0a();
    }

    /// Function to filter the faction list.
    pub unsafe fn filter_list(filter: Ref<QSortFilterProxyModel>, line_edit: Ref<QLineEdit>) {
        filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        filter.set_filter_regular_expression_q_string(&line_edit.text());
    }

    /// Function to add a new empty entry with the provided id.
    ///
    /// Make sure the id is valid before calling this.
    pub unsafe fn add_entry(&self, id: &str) {
        let mut new_entry = Entry::default();
        new_entry.set_id(id.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(new_entry.id())).into_ptr();
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&new_entry).unwrap())), DATA);
        self.main_list_model.append_row_q_standard_item(item);

        // Select the new item, clearing out the previous one.
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.main_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let index = self.main_list_model.index_2a(self.main_list_model.row_count_0a() - 1, 0);
        let filter_index = self.main_list_filter.map_from_source(&index);
        if filter_index.is_valid() {
            self.main_list_view.selection_model().select_q_model_index_q_flags_selection_flag(filter_index.as_ref(), SelectionFlag::Select.into())
        }

        self.main_list_filter.sort_2a(0, SortOrder::AscendingOrder);
    }

    /// Function to clone an existing entry with the new one having the provided id.
    ///
    /// Make sure the id is valid before calling this.
    pub unsafe fn clone_entry(&self, id: &str, index: Ref<QModelIndex>) {
        let mut data: Entry = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
        data.set_id(id.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(data.id())).into_ptr();
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.main_list_model.append_row_q_standard_item(item);

        // Select the new item, clearing out the previous one.
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.main_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let index = self.main_list_model.index_2a(self.main_list_model.row_count_0a() - 1, 0);
        let filter_index = self.main_list_filter.map_from_source(&index);
        if filter_index.is_valid() {
            self.main_list_view.selection_model().select_q_model_index_q_flags_selection_flag(filter_index.as_ref(), SelectionFlag::Select.into())
        }

        self.main_list_filter.sort_2a(0, SortOrder::AscendingOrder);
    }

    /// Function to remove an entry from the list.
    pub unsafe fn remove_entry(&self, index: Ref<QModelIndex>) {
        self.main_list_model.remove_row_1a(self.main_list_filter.map_to_source(index).row());
        self.detailed_view_widget.set_enabled(false);
    }

    /// Function to add a new empty variant with the provided id.
    ///
    /// Make sure the id is valid before calling this.
    pub unsafe fn add_variant(&self, id: &str) {
        let mut new_variant = Variant::default();
        new_variant.set_id(id.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(new_variant.id())).into_ptr();
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&new_variant).unwrap())), DATA);
        self.variants_list_model.append_row_q_standard_item(item);

        // Select the new item, clearing out the previous one.
        self.variants_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.variants_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let index = self.variants_list_model.index_2a(self.variants_list_model.row_count_0a() - 1, 0);
        let filter_index = self.variants_list_filter.map_from_source(&index);
        if filter_index.is_valid() {
            self.variants_list_view.selection_model().select_q_model_index_q_flags_selection_flag(filter_index.as_ref(), SelectionFlag::Select.into())
        }

        self.variants_list_filter.sort_2a(0, SortOrder::AscendingOrder);
    }

    /// Function to clone an existing variant with the new one having the provided id.
    ///
    /// Make sure the id is valid before calling this.
    pub unsafe fn clone_variant(&self, id: &str, index: Ref<QModelIndex>) {
        let mut data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
        data.set_id(id.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(data.id())).into_ptr();
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.variants_list_model.append_row_q_standard_item(item);

        // Select the new item, clearing out the previous one.
        self.variants_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.variants_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let index = self.variants_list_model.index_2a(self.variants_list_model.row_count_0a() - 1, 0);
        let filter_index = self.variants_list_filter.map_from_source(&index);
        if filter_index.is_valid() {
            self.variants_list_view.selection_model().select_q_model_index_q_flags_selection_flag(filter_index.as_ref(), SelectionFlag::Select.into())
        }

        self.variants_list_filter.sort_2a(0, SortOrder::AscendingOrder);
    }

    /// Function to remove a variant from the list.
    pub unsafe fn remove_variant(&self, index: Ref<QModelIndex>) {
        self.variants_list_model.remove_row_1a(self.variants_list_filter.map_to_source(index).row());
        self.variants_widget.set_enabled(false);
    }

    /// Function to trigger the dialog that allows you to write a new id, not pressent in banned_list.
    ///
    /// If current_id is passed, it'll be used as default value for the dialog.
    pub unsafe fn id_dialog(&self, current_id: Option<&str>, banned_list: Vec<String>) -> Result<Option<String>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { NAME_VIEW_DEBUG } else { NAME_VIEW_RELEASE };
        let main_widget = load_template(&self.main_list_view, template_path)?;

        let dialog: QPtr<QDialog> = main_widget.static_downcast();
        dialog.set_window_title(&qtr("portrait_settings_id_title"));

        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let message_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "message_widget")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());
        kmessage_widget_close_safe(&message_widget.as_ptr());

        name_line_edit.set_placeholder_text(&qtr("portrait_settings_id"));
        if let Some(id) = current_id {
            name_line_edit.set_text(&QString::from_std_str(id));
            button_box.button(StandardButton::Ok).set_enabled(false);
            show_message_error(&message_widget, tr("portrait_settings_list_id_error"));
        }

        name_line_edit.text_changed().connect(&SlotOfQString::new(&name_line_edit, move |new_name| {
            if banned_list.contains(&new_name.to_std_string()) {
                show_message_error(&message_widget, tr("portrait_settings_list_id_error"));
                button_box.button(StandardButton::Ok).set_enabled(false);
            } else {
                kmessage_widget_close_safe(&message_widget.as_ptr());
                button_box.button(StandardButton::Ok).set_enabled(true);
            }
        }));

        Ok(
            if dialog.exec() == 1 {
                let new_name = name_line_edit.text().to_std_string();
                if new_name.is_empty() {
                    None
                } else {
                    Some(new_name) }
            } else { None }
        )
    }

    /// Function to get the full list of strings from a model.
    pub unsafe fn text_list_from_model(model: &QPtr<QStandardItemModel>) -> Vec<String> {
        (0..model.row_count_0a())
            .map(|row| model.item_1a(row).text().to_std_string())
            .collect::<Vec<_>>()
    }
}
