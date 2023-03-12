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
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QMenu;
use qt_widgets::QWidget;

use qt_gui::QPixmap;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::SlotOfQString;
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QSignalBlocker;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SortOrder;

use cpp_core::Ref;

use anyhow::Result;
use getset::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::{ContainerPath, FileType, portrait_settings::*, RFile, RFileDecoded};

use rpfm_ui_common::locale::{qtr, tr};

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::*;
use crate::packedfile_views::{FileView, View, ViewType};
use crate::utils::*;

use self::slots::PortraitSettingsSlots;

use super::DataSource;

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
    path: Arc<RwLock<String>>,
    data_source: Arc<RwLock<DataSource>>,

    version: u32,
    detailed_view_widget: QPtr<QWidget>,
    body_camera_settings_groupbox: QPtr<QGroupBox>,

    main_list_view: QPtr<QListView>,
    main_list_filter: QBox<QSortFilterProxyModel>,
    main_list_model: QBox<QStandardItemModel>,
    main_filter_line_edit: QPtr<QLineEdit>,

    head_z_spinbox: QPtr<QDoubleSpinBox>,
    head_y_spinbox: QPtr<QDoubleSpinBox>,
    head_yaw_spinbox: QPtr<QDoubleSpinBox>,
    head_pitch_spinbox: QPtr<QDoubleSpinBox>,
    head_fov_spinbox: QPtr<QDoubleSpinBox>,
    head_skeleton_node_line_edit: QPtr<QLineEdit>,

    body_z_spinbox: QPtr<QDoubleSpinBox>,
    body_y_spinbox: QPtr<QDoubleSpinBox>,
    body_yaw_spinbox: QPtr<QDoubleSpinBox>,
    body_pitch_spinbox: QPtr<QDoubleSpinBox>,
    body_fov_spinbox: QPtr<QDoubleSpinBox>,
    body_skeleton_node_line_edit: QPtr<QLineEdit>,

    variants_widget: QPtr<QWidget>,
    variants_list_view: QPtr<QListView>,
    variants_list_filter: QBox<QSortFilterProxyModel>,
    variants_list_model: QBox<QStandardItemModel>,
    variants_filter_line_edit: QPtr<QLineEdit>,
    file_diffuse_line_edit: QPtr<QLineEdit>,
    file_mask_1_line_edit: QPtr<QLineEdit>,
    file_mask_2_line_edit: QPtr<QLineEdit>,
    file_mask_3_line_edit: QPtr<QLineEdit>,

    diffuse_label: QPtr<QLabel>,
    mask_1_label: QPtr<QLabel>,
    mask_2_label: QPtr<QLabel>,
    mask_3_label: QPtr<QLabel>,

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
    timer_delayed_reload_variant_images: QBox<QTimer>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl PortraitSettingsView {

    /// This function creates a new Portrait Settings View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &mut PortraitSettings,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);

        // ListView and groupboxes.
        let main_list_view: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "main_list_view")?;
        let main_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "main_filter_line_edit")?;
        let detailed_view_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "detailed_view_widget")?;
        let head_camera_settings_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "head_camera_settings_groupbox")?;
        let body_camera_settings_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "body_camera_settings_groupbox")?;
        let variants_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "variants_groupbox")?;
        let variants_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "variants_widget")?;
        head_camera_settings_groupbox.set_title(&qtr("portrait_settings_head_camera_settings_title"));
        body_camera_settings_groupbox.set_title(&qtr("portrait_settings_body_camera_settings_title"));
        variants_groupbox.set_title(&qtr("portrait_settings_variants_title"));
        main_filter_line_edit.set_placeholder_text(&qtr("portrait_settings_filter"));

        // Main camera.
        let head_z_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_z_label")?;
        let head_y_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_y_label")?;
        let head_yaw_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_yaw_label")?;
        let head_pitch_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_pitch_label")?;
        let head_fov_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_fov_label")?;
        let head_skeleton_node_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "head_skeleton_node_label")?;
        head_z_label.set_text(&qtr("portrait_settings_head_z"));
        head_y_label.set_text(&qtr("portrait_settings_head_y"));
        head_yaw_label.set_text(&qtr("portrait_settings_head_yaw"));
        head_pitch_label.set_text(&qtr("portrait_settings_head_pitch"));
        head_fov_label.set_text(&qtr("portrait_settings_head_fov"));
        head_skeleton_node_label.set_text(&qtr("portrait_settings_head_skeleton_node"));

        let head_z_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "head_z_spinbox")?;
        let head_y_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "head_y_spinbox")?;
        let head_yaw_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "head_yaw_spinbox")?;
        let head_pitch_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "head_pitch_spinbox")?;
        let head_fov_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "head_fov_spinbox")?;
        let head_skeleton_node_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "head_skeleton_node_line_edit")?;

        // Body camera
        let body_z_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_z_label")?;
        let body_y_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_y_label")?;
        let body_yaw_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_yaw_label")?;
        let body_pitch_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_pitch_label")?;
        let body_fov_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_fov_label")?;
        let body_skeleton_node_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "body_skeleton_node_label")?;
        body_z_label.set_text(&qtr("portrait_settings_body_z"));
        body_y_label.set_text(&qtr("portrait_settings_body_y"));
        body_yaw_label.set_text(&qtr("portrait_settings_body_yaw"));
        body_pitch_label.set_text(&qtr("portrait_settings_body_pitch"));
        body_fov_label.set_text(&qtr("portrait_settings_body_fov"));
        body_skeleton_node_label.set_text(&qtr("portrait_settings_body_skeleton_node"));

        let body_z_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_z_spinbox")?;
        let body_y_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_y_spinbox")?;
        let body_yaw_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_yaw_spinbox")?;
        let body_pitch_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_pitch_spinbox")?;
        let body_fov_spinbox: QPtr<QDoubleSpinBox> = find_widget(&main_widget.static_upcast(), "body_fov_spinbox")?;
        let body_skeleton_node_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "body_skeleton_node_line_edit")?;

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

        // Placeholders.
        let diffuse_label_placeholder: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "diffuse_label")?;
        let mask_1_label_placeholder: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "mask_1_label")?;
        let mask_2_label_placeholder: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "mask_2_label")?;
        let mask_3_label_placeholder: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "mask_3_label")?;

        let diffuse_label: QPtr<QLabel> = new_resizable_label_safe(&variants_widget.as_ptr(), &QPixmap::new().into_ptr());
        let mask_1_label: QPtr<QLabel> = new_resizable_label_safe(&variants_widget.as_ptr(), &QPixmap::new().into_ptr());
        let mask_2_label: QPtr<QLabel> = new_resizable_label_safe(&variants_widget.as_ptr(), &QPixmap::new().into_ptr());
        let mask_3_label: QPtr<QLabel> = new_resizable_label_safe(&variants_widget.as_ptr(), &QPixmap::new().into_ptr());

        let variants_layout = variants_widget.layout().static_downcast::<QGridLayout>();
        variants_layout.replace_widget_2a(diffuse_label_placeholder, diffuse_label.as_ptr());
        variants_layout.replace_widget_2a(mask_1_label_placeholder, mask_1_label.as_ptr());
        variants_layout.replace_widget_2a(mask_2_label_placeholder, mask_2_label.as_ptr());
        variants_layout.replace_widget_2a(mask_3_label_placeholder, mask_3_label.as_ptr());

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
        let timer_delayed_reload_variant_images = QTimer::new_1a(main_widget.as_ptr());
        timer_delayed_updates_main.set_single_shot(true);
        timer_delayed_updates_variants.set_single_shot(true);
        timer_delayed_reload_variant_images.set_single_shot(true);

        let view = Arc::new(Self{
            path: file_view.path_raw(),
            data_source: Arc::new(RwLock::new(file_view.data_source())),

            version: *data.version(),
            main_list_view,
            main_list_filter,
            main_list_model,
            main_filter_line_edit,
            detailed_view_widget,
            body_camera_settings_groupbox,

            head_z_spinbox,
            head_y_spinbox,
            head_yaw_spinbox,
            head_pitch_spinbox,
            head_fov_spinbox,
            head_skeleton_node_line_edit,

            body_z_spinbox,
            body_y_spinbox,
            body_yaw_spinbox,
            body_pitch_spinbox,
            body_fov_spinbox,
            body_skeleton_node_line_edit,

            variants_widget,
            variants_list_view,
            variants_list_filter,
            variants_list_model,
            variants_filter_line_edit,
            file_diffuse_line_edit,
            file_mask_1_line_edit,
            file_mask_2_line_edit,
            file_mask_3_line_edit,
            diffuse_label,
            mask_1_label,
            mask_2_label,
            mask_3_label,

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
            timer_delayed_reload_variant_images,
        });

        view.load_data(data)?;

        let slots = PortraitSettingsSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        file_view.file_type = FileType::PortraitSettings;
        file_view.view_type = ViewType::Internal(View::PortraitSettings(view));

        Ok(())
    }

    /// Function to clear the full view so it doesn't have data un-linked to any item on the list.
    pub unsafe fn clear_main_view(&self) {
        self.detailed_view_widget.set_enabled(false);

        self.head_z_spinbox.clear();
        self.head_y_spinbox.clear();
        self.head_yaw_spinbox.clear();
        self.head_pitch_spinbox.clear();
        self.head_fov_spinbox.clear();
        self.head_skeleton_node_line_edit.clear();

        self.body_camera_settings_groupbox.set_checked(false);
        self.body_z_spinbox.clear();
        self.body_y_spinbox.clear();
        self.body_yaw_spinbox.clear();
        self.body_pitch_spinbox.clear();
        self.body_fov_spinbox.clear();
        self.body_skeleton_node_line_edit.clear();

        self.variants_list_model.clear();

        self.clear_variants_view();
    }

    /// Function to clear the variants view so it doesn't have data un-linked to any item on the list.
    pub unsafe fn clear_variants_view(&self) {
        self.variants_widget.set_enabled(false);
        self.file_diffuse_line_edit.clear();
        self.file_mask_1_line_edit.clear();
        self.file_mask_2_line_edit.clear();
        self.file_mask_3_line_edit.clear();

        set_pixmap_on_resizable_label_safe(&self.diffuse_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_1_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_2_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_3_label.as_ptr(), &QPixmap::new().into_ptr());
    }

    /// Function to save the view and encode it into a PortraitSettings struct.
    pub unsafe fn save_view(&self) -> PortraitSettings {

        // This saves whatever it's open to its item.
        let selection = self.main_list_view.selection_model().selection();
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());

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

        self.head_z_spinbox.set_value(*data.camera_settings_head().z() as f64);
        self.head_y_spinbox.set_value(*data.camera_settings_head().y() as f64);
        self.head_yaw_spinbox.set_value(*data.camera_settings_head().yaw() as f64);
        self.head_pitch_spinbox.set_value(*data.camera_settings_head().pitch() as f64);
        self.head_fov_spinbox.set_value(*data.camera_settings_head().fov() as f64);
        self.head_skeleton_node_line_edit.set_text(&QString::from_std_str(data.camera_settings_head().skeleton_node()));

        match data.camera_settings_body() {
            Some(data) => {
                self.body_camera_settings_groupbox.set_checked(true);

                self.body_z_spinbox.set_value(*data.z() as f64);
                self.body_y_spinbox.set_value(*data.y() as f64);
                self.body_yaw_spinbox.set_value(*data.yaw() as f64);
                self.body_pitch_spinbox.set_value(*data.pitch() as f64);
                self.body_fov_spinbox.set_value(*data.fov() as f64);
                self.body_skeleton_node_line_edit.set_text(&QString::from_std_str(data.skeleton_node()));
            },
            None => {
                self.body_camera_settings_groupbox.set_checked(false);

                self.body_z_spinbox.clear();
                self.body_y_spinbox.clear();
                self.body_yaw_spinbox.clear();
                self.body_pitch_spinbox.clear();
                self.body_fov_spinbox.clear();
                self.body_skeleton_node_line_edit.clear();
            }
        }

        self.variants_list_model.clear();
        self.variants_widget.set_enabled(false);

        // Disable these on load so they cannot be trigger with no selection.
        self.variants_list_clone.set_enabled(false);
        self.variants_list_delete.set_enabled(false);

        data.variants_mut().sort_by(|a, b| a.filename().cmp(b.filename()));
        for variant in data.variants() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(variant.filename())).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&variant).unwrap())), DATA);
            self.variants_list_model.append_row_q_standard_item(item);
        }

        self.file_diffuse_line_edit.clear();
        self.file_mask_1_line_edit.clear();
        self.file_mask_2_line_edit.clear();
        self.file_mask_3_line_edit.clear();

        set_pixmap_on_resizable_label_safe(&self.diffuse_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_1_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_2_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_3_label.as_ptr(), &QPixmap::new().into_ptr());
    }

    /// This function loads the data of a variant into the variant detailed view.
    pub unsafe fn load_variant_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first item loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.variants_widget.is_enabled() {
            self.variants_widget.set_enabled(true);
        }

        let data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        // Blockers so we don't trigger the image load twice.
        let blocker_file_diffuse = QSignalBlocker::from_q_object(self.file_diffuse_line_edit.static_upcast::<QObject>());
        let blocker_file_mask_1 = QSignalBlocker::from_q_object(self.file_mask_1_line_edit.static_upcast::<QObject>());
        let blocker_file_mask_2 = QSignalBlocker::from_q_object(self.file_mask_2_line_edit.static_upcast::<QObject>());
        let blocker_file_mask_3 = QSignalBlocker::from_q_object(self.file_mask_3_line_edit.static_upcast::<QObject>());

        self.file_diffuse_line_edit.set_text(&QString::from_std_str(data.file_diffuse()));
        self.file_mask_1_line_edit.set_text(&QString::from_std_str(data.file_mask_1()));
        self.file_mask_2_line_edit.set_text(&QString::from_std_str(data.file_mask_2()));
        self.file_mask_3_line_edit.set_text(&QString::from_std_str(data.file_mask_3()));

        blocker_file_diffuse.unblock();
        blocker_file_mask_1.unblock();
        blocker_file_mask_2.unblock();
        blocker_file_mask_3.unblock();

        self.load_variant_images(data.file_diffuse(), data.file_mask_1(), data.file_mask_2(), data.file_mask_3());
    }

    /// This function loads the variants images from the paths in each input to the UI, if found.
    ///
    /// This usually gets triggered "delayed", so except on load, the image change is always about 500ms after the input is changed.
    pub unsafe fn load_variant_images(&self, diffuse: &str, mask_1: &str, mask_2: &str, mask_3: &str) {
        set_pixmap_on_resizable_label_safe(&self.diffuse_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_1_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_2_label.as_ptr(), &QPixmap::new().into_ptr());
        set_pixmap_on_resizable_label_safe(&self.mask_3_label.as_ptr(), &QPixmap::new().into_ptr());

        // Try to get the relevant images and load them to the preview widgets.
        let mut paths = vec![];
        let mut search = false;
        if !diffuse.is_empty() {
            paths.push(ContainerPath::File(diffuse.to_owned()));
            search = true;
        }

        if !mask_1.is_empty() {
            paths.push(ContainerPath::File(mask_1.to_owned()));
            search = true;
        }

        if !mask_2.is_empty() {
            paths.push(ContainerPath::File(mask_2.to_owned()));
            search = true;
        }

        if !mask_3.is_empty() {
            paths.push(ContainerPath::File(mask_3.to_owned()));
            search = true;
        }

        // Do not bother doing this if we have no paths.
        if search {
            let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(paths));
            let response = CENTRAL_COMMAND.recv_try(&receiver);
            match response {
                Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                    Self::load_variant_image_to_label(diffuse, &self.diffuse_label, &mut files);
                    Self::load_variant_image_to_label(mask_1, &self.mask_1_label, &mut files);
                    Self::load_variant_image_to_label(mask_2, &self.mask_2_label, &mut files);
                    Self::load_variant_image_to_label(mask_3, &self.mask_3_label, &mut files);
                },
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        }
    }

    /// This function tries to load the image at the provided path to the provided label.
    ///
    /// Tries.
    pub unsafe fn load_variant_image_to_label(path: &str, label: &QPtr<QLabel>, files: &mut HashMap<DataSource, HashMap<String, RFile>>) {
        if !path.is_empty() {
            let path_we_want = path.to_lowercase();
            for files in files.values_mut() {
                let mut found = false;
                for (path, file) in files {
                    if path.to_lowercase() == path_we_want {
                        if let Ok(Some(RFileDecoded::Image(data))) = file.decode(&None, false, true) {
                            let byte_array = QByteArray::from_slice(data.data()).into_ptr();
                            let image = QPixmap::new();
                            image.load_from_data_q_byte_array(byte_array.as_ref().unwrap());
                            set_pixmap_on_resizable_label_safe(&label.as_ptr(), &image.into_ptr());
                        }
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
    }

    /// This function saves the data of an entry from the detailed view.
    pub unsafe fn save_entry_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: Entry = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        data.camera_settings_head_mut().set_z(self.head_z_spinbox.value() as f32);
        data.camera_settings_head_mut().set_y(self.head_y_spinbox.value() as f32);
        data.camera_settings_head_mut().set_yaw(self.head_yaw_spinbox.value() as f32);
        data.camera_settings_head_mut().set_pitch(self.head_pitch_spinbox.value() as f32);
        data.camera_settings_head_mut().set_fov(self.head_fov_spinbox.value() as f32);
        data.camera_settings_head_mut().set_skeleton_node(self.head_skeleton_node_line_edit.text().to_std_string());

        if self.body_camera_settings_groupbox.is_checked() {
            let mut body_camera_settings = CameraSetting::default();
            body_camera_settings.set_z(self.body_z_spinbox.value() as f32);
            body_camera_settings.set_y(self.body_y_spinbox.value() as f32);
            body_camera_settings.set_yaw(self.body_yaw_spinbox.value() as f32);
            body_camera_settings.set_pitch(self.body_pitch_spinbox.value() as f32);
            body_camera_settings.set_fov(self.body_fov_spinbox.value() as f32);
            body_camera_settings.set_skeleton_node(self.body_skeleton_node_line_edit.text().to_std_string());

            *data.camera_settings_body_mut() = Some(body_camera_settings);
        } else {
            *data.camera_settings_body_mut() = None;
        }

        // This saves whatever it's open in the variant list to its item.
        let selection = self.variants_list_view.selection_model().selection();
        self.variants_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
        self.variants_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
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

    /// Function to add a new empty variant with the provided filename.
    ///
    /// Make sure the filename is valid before calling this.
    pub unsafe fn add_variant(&self, filename: &str) {
        let mut new_variant = Variant::default();
        new_variant.set_filename(filename.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(new_variant.filename())).into_ptr();
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

    /// Function to clone an existing variant with the new one having the provided filename.
    ///
    /// Make sure the filename is valid before calling this.
    pub unsafe fn clone_variant(&self, filename: &str, index: Ref<QModelIndex>) {
        let mut data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
        data.set_filename(filename.to_owned());

        let item = QStandardItem::from_q_string(&QString::from_std_str(data.filename())).into_ptr();
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
