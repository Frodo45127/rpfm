//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the code to manage a Unit Variant View.

use qt_widgets::QAction;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
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
use qt_core::QModelIndex;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;
use qt_core::SortOrder;

use cpp_core::Ref;

use anyhow::Result;
use getset::*;

use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::{FileType, RFileDecoded, unit_variant::*};

use rpfm_ui_common::locale::{qtr, tr};
use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::ffi::*;
use crate::packedfile_views::{FileView, View, ViewType};
use crate::utils::*;
use crate::views::debug::DebugView;

use self::slots::UnitVariantSlots;

use super::DataSource;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/unit_variant_editor.ui";
const VIEW_RELEASE: &str = "ui/unit_variant_editor.ui";

const NAME_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/unit_variant_name_dialog.ui";
const NAME_VIEW_RELEASE: &str = "ui/unit_variant_name_dialog.ui";

const DATA: i32 = 20;
const NAME: i32 = 40;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a Unit Variant File.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct UnitVariantView {
    path: Arc<RwLock<String>>,
    data_source: Arc<RwLock<DataSource>>,

    version: u32,
    detailed_view_widget: QPtr<QWidget>,

    main_list_view: QPtr<QListView>,
    main_list_filter: QBox<QSortFilterProxyModel>,
    main_list_model: QBox<QStandardItemModel>,
    main_filter_line_edit: QPtr<QLineEdit>,

    name_lineedit: QPtr<QLineEdit>,

    variants_widget: QPtr<QWidget>,
    variants_list_view: QPtr<QListView>,
    variants_list_filter: QBox<QSortFilterProxyModel>,
    variants_list_model: QBox<QStandardItemModel>,
    variants_filter_line_edit: QPtr<QLineEdit>,
    mesh_file_line_edit: QPtr<QLineEdit>,
    texture_folder_line_edit: QPtr<QLineEdit>,
    unknown_value_spinbox: QPtr<QSpinBox>,

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


pub struct UnitVariantDebugView {
    debug_view: Arc<DebugView>,
}


//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl UnitVariantView {

    /// This function creates a new Unit Variant View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &mut UnitVariant,
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
        let unit_variant_details_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "details_groupbox")?;
        let variants_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "variants_groupbox")?;
        let variants_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "variants_widget")?;
        unit_variant_details_groupbox.set_title(&qtr("unit_variant_details_title"));
        variants_groupbox.set_title(&qtr("unit_variant_variants_title"));
        main_filter_line_edit.set_placeholder_text(&qtr("unit_variant_filter"));

        // Unit Variant data.
        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let name_lineedit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_lineedit")?;
        name_label.set_text(&qtr("unit_variant_name"));

        // Variants
        let variants_list_view: QPtr<QListView> = find_widget(&main_widget.static_upcast(), "variants_list_view")?;
        let variants_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "variants_filter_line_edit")?;
        variants_filter_line_edit.set_placeholder_text(&qtr("unit_variant_filter"));

        let mesh_file_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "mesh_file_label")?;
        let texture_folder_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "texture_folder_label")?;
        let unknown_value_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "unknown_value_label")?;
        mesh_file_label.set_text(&qtr("unit_variant_mesh_file"));
        texture_folder_label.set_text(&qtr("unit_variant_texture_folder"));
        unknown_value_label.set_text(&qtr("unit_variant_unknown_value"));

        let mesh_file_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "mesh_file_line_edit")?;
        let texture_folder_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "texture_folder_line_edit")?;
        let unknown_value_spinbox: QPtr<QSpinBox> = find_widget(&main_widget.static_upcast(), "unknown_value_spinbox")?;

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
        let main_list_add = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "add", "context_menu_add", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        let main_list_clone = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "clone", "context_menu_clone", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        let main_list_delete = add_action_to_menu(&main_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "delete", "context_menu_delete", Some(main_list_view.static_upcast::<qt_widgets::QWidget>()));
        main_list_clone.set_enabled(false);
        main_list_delete.set_enabled(false);

        let variants_list_context_menu = QMenu::from_q_widget(&variants_list_view);
        let variants_list_add = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "add", "context_menu_add", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        let variants_list_clone = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "clone", "context_menu_clone", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        let variants_list_delete = add_action_to_menu(&variants_list_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "unit_variant", "delete", "context_menu_delete", Some(variants_list_view.static_upcast::<qt_widgets::QWidget>()));
        variants_list_clone.set_enabled(false);
        variants_list_delete.set_enabled(false);

        // Filter timer.
        let timer_delayed_updates_main = QTimer::new_1a(main_widget.as_ptr());
        let timer_delayed_updates_variants = QTimer::new_1a(main_widget.as_ptr());
        timer_delayed_updates_main.set_single_shot(true);
        timer_delayed_updates_variants.set_single_shot(true);

        let view = Arc::new(Self{
            path: file_view.path_raw(),
            data_source: Arc::new(RwLock::new(file_view.data_source())),

            version: *data.version(),
            main_list_view,
            main_list_filter,
            main_list_model,
            main_filter_line_edit,
            detailed_view_widget,

            name_lineedit,

            variants_widget,
            variants_list_view,
            variants_list_filter,
            variants_list_model,
            variants_filter_line_edit,
            mesh_file_line_edit,
            texture_folder_line_edit,
            unknown_value_spinbox,

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

        let slots = UnitVariantSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        file_view.file_type = FileType::UnitVariant;
        file_view.view_type = ViewType::Internal(View::UnitVariant(view));

        Ok(())
    }

    /// Function to clear the full view so it doesn't have data un-linked to any item on the list.
    pub unsafe fn clear_main_view(&self) {
        self.detailed_view_widget.set_enabled(false);

        self.name_lineedit.clear();
        self.variants_list_model.clear();

        self.clear_variants_view();
    }

    /// Function to clear the variants view so it doesn't have data un-linked to any item on the list.
    pub unsafe fn clear_variants_view(&self) {
        self.variants_widget.set_enabled(false);

        self.mesh_file_line_edit.clear();
        self.texture_folder_line_edit.clear();
        self.unknown_value_spinbox.clear();
    }

    /// Function to save the view and encode it into a UnitVariant struct.
    pub unsafe fn save_view(&self) -> UnitVariant {

        // This saves whatever it's open to its item.
        let selection = self.main_list_view.selection_model().selection();
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
        self.main_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());

        let mut data = UnitVariant::default();
        data.set_version(self.version);

        for row in 0..self.main_list_model.row_count_0a() {
            let index = self.main_list_model.index_2a(row, 0);
            let mut category: Category = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

            // Update the category id.
            category.set_id(index.data_1a(2).to_long_long_0a() as u64);

            data.categories_mut().push(category);
        }

        data
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &mut UnitVariant) -> Result<()> {

        // Clear ALL the fields before reloading.
        self.clear_main_view();
        self.load_data(data)
    }

    /// This function loads the data into the view, so it can be accessed in the UI.
    unsafe fn load_data(&self, data: &mut UnitVariant) -> Result<()> {
        self.main_list_model.clear();

        // Get them sorted so we have them in order for the UI.
        data.categories_mut().sort_by(|a, b| a.id().cmp(b.id()));
        for entry in data.categories() {
            let item = QStandardItem::new();

            item.set_data_2a(&QVariant::from_u64(*entry.id()), 2);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(entry.name())), NAME);
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&entry)?)), DATA);

            self.main_list_model.append_row_q_standard_item(item.into_ptr());
        }

        new_unit_variant_item_delegate_safe(&self.main_list_view.static_upcast::<QObject>().as_ptr(), 0);

        Ok(())
    }

    /// This function loads the data of an entry into the detailed view.
    pub unsafe fn load_entry_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first item loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.detailed_view_widget.is_enabled() {
            self.detailed_view_widget.set_enabled(true);
        }

        let data: Category = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        self.name_lineedit.set_text(&QString::from_std_str(data.name()));
        self.variants_list_model.clear();
        self.variants_widget.set_enabled(false);

        // Disable these on load so they cannot be trigger with no selection.
        self.variants_list_clone.set_enabled(false);
        self.variants_list_delete.set_enabled(false);

        for (row, variant) in data.variants().iter().enumerate() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(row.to_string())).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&variant).unwrap())), DATA);
            self.variants_list_model.append_row_q_standard_item(item);
        }

        new_unit_variant_item_delegate_safe(&self.variants_list_view.static_upcast::<QObject>().as_ptr(), 0);

        self.mesh_file_line_edit.clear();
        self.texture_folder_line_edit.clear();
        self.unknown_value_spinbox.clear();
    }

    /// This function loads the data of a variant into the variant detailed view.
    pub unsafe fn load_variant_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first item loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.variants_widget.is_enabled() {
            self.variants_widget.set_enabled(true);
        }

        let data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        self.mesh_file_line_edit.set_text(&QString::from_std_str(data.mesh_file()));
        self.texture_folder_line_edit.set_text(&QString::from_std_str(data.texture_folder()));
        self.unknown_value_spinbox.set_value(*data.unknown_value() as i32);
    }

    /// This function saves the data of an entry from the detailed view.
    pub unsafe fn save_entry_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let item = self.main_list_model.item_from_index(index);
        let mut data: Category = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        // Update the name in the item too.
        data.set_name(self.name_lineedit.text().to_std_string());
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(data.name())), NAME);

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

        data.set_mesh_file(self.mesh_file_line_edit.text().to_std_string());
        data.set_texture_folder(self.texture_folder_line_edit.text().to_std_string());
        data.set_unknown_value(self.unknown_value_spinbox.value() as u16);

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
    pub unsafe fn add_category(&self, id: i64) {
        let mut data = Category::default();
        data.set_id(id as u64);

        let item = QStandardItem::new();

        item.set_data_2a(&QVariant::from_u64(*data.id()), 2);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(data.name())), NAME);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.main_list_model.append_row_q_standard_item(item.into_ptr());

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
    pub unsafe fn clone_category(&self, id: i64, index: Ref<QModelIndex>) {
        let mut data: Category = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();
        data.set_id(id as u64);

        let item = QStandardItem::new();

        item.set_data_2a(&QVariant::from_u64(*data.id()), 2);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(data.name())), NAME);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.main_list_model.append_row_q_standard_item(item.into_ptr());

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
    pub unsafe fn remove_category(&self, index: Ref<QModelIndex>) {
        self.main_list_model.remove_row_1a(self.main_list_filter.map_to_source(index).row());
        self.detailed_view_widget.set_enabled(false);
    }

    /// Function to add a new empty variant with the provided filename.
    ///
    /// Make sure the filename is valid before calling this.
    pub unsafe fn add_variant(&self) {
        let data = Variant::default();
        let item = QStandardItem::new();

        item.set_data_2a(&QVariant::from_int(self.variants_list_model.row_count_0a()), 2);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.variants_list_model.append_row_q_standard_item(item.into_ptr());

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
    pub unsafe fn clone_variant(&self, index: Ref<QModelIndex>) {
        let data: Variant = serde_json::from_str(&index.data_1a(DATA).to_string().to_std_string()).unwrap();

        let item = QStandardItem::new();

        item.set_data_2a(&QVariant::from_int(self.variants_list_model.row_count_0a()), 2);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), DATA);
        self.variants_list_model.append_row_q_standard_item(item.into_ptr());

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

        // Renumber all the rows.
        (0..self.variants_list_model.row_count_0a()).for_each(|row| {
            let item = self.variants_list_model.item_1a(row);
            item.set_data_2a(&QVariant::from_int(row), 2)
        });

        self.variants_widget.set_enabled(false);
    }

    /// Function to trigger the dialog that allows you to write a new id, not present in banned_list.
    ///
    /// If current_id is passed, it'll be used as default value for the dialog.
    pub unsafe fn id_dialog(&self, current_id: Option<i64>, banned_list: Vec<i64>) -> Result<Option<i64>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { NAME_VIEW_DEBUG } else { NAME_VIEW_RELEASE };
        let main_widget = load_template(&self.main_list_view, template_path)?;

        let dialog: QPtr<QDialog> = main_widget.static_downcast();
        dialog.set_window_title(&qtr("unit_variant_new_category_title"));

        let id_spinbox = new_q_spinbox_i64_safe(&main_widget.static_upcast());
        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let layout = main_widget.layout().static_downcast::<QGridLayout>();
        layout.replace_widget_2a(name_line_edit.as_ptr(), id_spinbox.as_ptr());
        name_line_edit.delete_later();

        let message_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "message_widget")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());
        kmessage_widget_close_safe(&message_widget.as_ptr());

        if let Some(id) = current_id {
            set_value_q_spinbox_i64_safe(&id_spinbox, id);
            button_box.button(StandardButton::Ok).set_enabled(false);
            show_message_error(&message_widget, tr("portrait_settings_list_id_error"));
        }

        //value_changed_q_spinbox_i64(&id_spinbox).connect(&SlotOfI64::new(&id_spinbox, move |new_id| {
        //    dbg!(&new_id);
        //    if banned_list.contains(&new_id) {
        //        show_message_error(&message_widget, tr("portrait_settings_list_id_error"));
        //        button_box.button(StandardButton::Ok).set_enabled(false);
        //    } else {
        //        kmessage_widget_close_safe(&message_widget.as_ptr());
        //        button_box.button(StandardButton::Ok).set_enabled(true);
        //    }
        //}));

        Ok(
            if dialog.exec() == 1 {
                let new_id = id_spinbox.text();
                if new_id.is_empty() || banned_list.contains(&new_id.to_long_long_0a()) {
                    None
                } else {
                    Some(new_id.to_long_long_0a()) }
            } else { None }
        )
    }

    /// Function to get the full list of strings from a model.
    pub unsafe fn value_list_from_model(model: &QPtr<QStandardItemModel>) -> Vec<i64> {
        (0..model.row_count_0a())
            .map(|row| model.item_1a(row).data_1a(2).to_long_long_0a())
            .collect::<Vec<_>>()
    }
}

impl UnitVariantDebugView {

    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: UnitVariant
    ) -> Result<()> {

        // For now just build a debug view.
        let debug_view = DebugView::new_view(
            file_view.main_widget(),
            RFileDecoded::UnitVariant(data),
            file_view.path_raw(),
        )?;

        let view = Self {
            debug_view,
        };

        file_view.view_type = ViewType::Internal(View::UnitVariantDebug(Arc::new(view)));
        file_view.file_type = FileType::MatchedCombat;

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: UnitVariant) -> Result<()> {
        self.debug_view.reload_view(&serde_json::to_string_pretty(&data)?);

        Ok(())
    }
}


