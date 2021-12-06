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
Module with all the code for managing the Variant Editor subtool of the Unit Editor tool.

This tool is a dialog where you can configure the variant used by a specific unit.
!*/

use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;

use qt_gui::QPixmap;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::QBox;
use qt_core::QByteArray;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;

use cpp_core::Ref;

use itertools::Itertools;

use std::collections::HashMap;

use rpfm_lib::packfile::PathType;
use rpfm_macros::*;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::tr;
use crate::views::table::utils::get_reference_data;
use self::slots::SubToolVariantUnitEditorSlots;
use super::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_unit_editor_variant_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_unit_editor_variant_editor.ui";

/// List of fields tht require special treatment from unit_variants_colours_tables.
const UNIT_VARIANTS_COLOURS_CUSTOM_FIELDS: [&str; 11] = [
    "key",
    "unit_variant",
    "primary_colour_r",
    "primary_colour_g",
    "primary_colour_b",
    "secondary_colour_r",
    "secondary_colour_g",
    "secondary_colour_b",
    "tertiary_colour_r",
    "tertiary_colour_g",
    "tertiary_colour_b",
];

/// List of fields tht require special treatment from variants_tables.
const VARIANTS_CUSTOM_FIELDS: [&str; 1] = [
    "variant_filename",
];

/// Field that holds the "key" we use to identify each variant. It's "faction" because we can have multiple "faction" variants for each unit variant.
const VARIANT_KEY_VALUE: &str = "unit_variants_faction";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the widgets used by the `Unit Editor` Tool, along with some data needed for the view to work.
#[derive(GetRef, GetRefMut)]
pub struct SubToolVariantUnitEditor {
    tool: Tool,
    timer_delayed_updates: QBox<QTimer>,

    faction_list_view: QPtr<QListView>,
    faction_list_filter: QBox<QSortFilterProxyModel>,
    faction_list_model: QBox<QStandardItemModel>,
    faction_list_filter_line_edit: QPtr<QLineEdit>,

    unit_variants_colours_list_view: QPtr<QListView>,
    unit_variants_colours_list_filter: QBox<QSortFilterProxyModel>,
    unit_variants_colours_list_model: QBox<QStandardItemModel>,

    unit_variants_unit_card_preview_label: QPtr<QLabel>,
    unit_variants_unit_card_label: QPtr<QLabel>,
    unit_variants_unit_card_combobox: QPtr<QComboBox>,

    unit_variants_colours_faction_combobox: QPtr<QComboBox>,
    unit_variants_colours_subculture_combobox: QPtr<QComboBox>,
    unit_variants_colours_soldier_type_combobox: QPtr<QComboBox>,

    unit_variants_colours_primary_colour_combobox: QPtr<QComboBox>,
    unit_variants_colours_secondary_colour_combobox: QPtr<QComboBox>,
    unit_variants_colours_tertiary_colour_combobox: QPtr<QComboBox>,

    variants_mesh_editor_main_widget: QPtr<QWidget>,
    variants_mesh_editor: QBox<QWidget>,
    variants_variant_filename_combobox: QPtr<QComboBox>,

    detailed_view_widget: QPtr<QWidget>,
    unit_variants_colours_widget: QPtr<QWidget>
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SubToolVariantUnitEditor`.
impl SubToolVariantUnitEditor {

    /// This function creates the tool's dialog.
    ///
    /// NOTE: This can fail at runtime if any of the expected widgets is not in the UI's XML.
    pub unsafe fn new(parent: Ref<QWidget>, data: &HashMap<String, String>) -> Result<Option<HashMap<String, String>>> {

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let tool = Tool::new(parent, &[], &TOOL_SUPPORTED_GAMES, view)?;

        tool.set_title(&tr("variant_editor_title"));

        // ListView.
        let faction_list_view: QPtr<QListView> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(), "faction_list_view")?;
        let faction_list_filter = QSortFilterProxyModel::new_1a(&faction_list_view);
        let faction_list_model = QStandardItemModel::new_1a(&faction_list_filter);
        let faction_list_filter_line_edit: QPtr<QLineEdit> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(), "faction_list_filter_line_edit")?;
        faction_list_view.set_model(&faction_list_filter);
        faction_list_filter.set_source_model(&faction_list_model);

        let unit_variants_colours_list_view: QPtr<QListView> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(), "unit_variants_colours_list_view")?;
        let unit_variants_colours_list_filter = QSortFilterProxyModel::new_1a(&unit_variants_colours_list_view);
        let unit_variants_colours_list_model = QStandardItemModel::new_1a(&unit_variants_colours_list_filter);
        unit_variants_colours_list_view.set_model(&unit_variants_colours_list_filter);
        unit_variants_colours_list_filter.set_source_model(&unit_variants_colours_list_model);

        // Filter timer.
        let timer_delayed_updates = QTimer::new_1a(tool.get_ref_main_widget());
        timer_delayed_updates.set_single_shot(true);

        // Icon stuff.
        let unit_variants_unit_card_preview_label: QPtr<QLabel> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(),"unit_variants_unit_card_preview_label")?;
        let unit_variants_unit_card_label: QPtr<QLabel> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(),"unit_variants_unit_card_label")?;
        let unit_variants_unit_card_combobox: QPtr<QComboBox> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(),"unit_variants_unit_card_combobox")?;

        let unit_variants_colours_faction_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_faction_combobox")?;
        let unit_variants_colours_subculture_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_subculture_combobox")?;
        let unit_variants_colours_soldier_type_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_soldier_type_combobox")?;

        let unit_variants_colours_primary_colour_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_primary_colour_combobox")?;
        let unit_variants_colours_secondary_colour_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_secondary_colour_combobox")?;
        let unit_variants_colours_tertiary_colour_combobox: QPtr<QComboBox> = tool.find_widget("unit_variants_colours_tertiary_colour_combobox")?;

        let variants_mesh_editor_main_widget: QPtr<QWidget> = tool.find_widget("variants_mesh_editor_main_widget")?;
        let variants_mesh_editor_placeholder: QPtr<QWidget> = tool.find_widget("variants_mesh_editor")?;
        let variants_mesh_editor: QBox<QWidget> = new_text_editor_safe(&variants_mesh_editor_main_widget);
        let variants_variant_filename_combobox: QPtr<QComboBox> = tool.find_widget("variants_variant_filename_combobox")?;
        variants_mesh_editor_placeholder.set_visible(false);
        variants_mesh_editor_main_widget.layout().replace_widget_2a(variants_mesh_editor_placeholder.as_ptr(), variants_mesh_editor.as_ptr());

        // Detailed view widget.
        let detailed_view_widget: QPtr<QWidget> = Tool::find_widget_no_tool(&tool.get_ref_main_widget().static_upcast(),"detailed_view_widget")?;
        let unit_variants_colours_widget: QPtr<QWidget> = tool.find_widget("unit_variants_colours_widget")?;

        // Build the view itself.
        let view = Self {
            tool,
            timer_delayed_updates,

            faction_list_view,
            faction_list_filter,
            faction_list_model,
            faction_list_filter_line_edit,

            unit_variants_colours_list_view,
            unit_variants_colours_list_filter,
            unit_variants_colours_list_model,

            unit_variants_unit_card_preview_label,
            unit_variants_unit_card_label,
            unit_variants_unit_card_combobox,

            unit_variants_colours_faction_combobox,
            unit_variants_colours_subculture_combobox,
            unit_variants_colours_soldier_type_combobox,

            unit_variants_colours_primary_colour_combobox,
            unit_variants_colours_secondary_colour_combobox,
            unit_variants_colours_tertiary_colour_combobox,

            variants_mesh_editor_main_widget,
            variants_mesh_editor,
            variants_variant_filename_combobox,

            detailed_view_widget,
            unit_variants_colours_widget,
        };

        // Build the slots and connect them to the view.
        let view = Rc::new(view);
        let slots = SubToolVariantUnitEditorSlots::new(&view);
        connections::set_connections(&view, &slots)?;

        // Setup text translations.
        view.setup_translations()?;

        // Load all the data to the view.
        view.load_data(&data)?;

        // If we hit ok, save the data back to the parent tool.
        if view.tool.get_ref_main_widget().static_downcast::<QDialog>().exec() == 1 {
            Ok(Some(view.save_data()?))
        }

        // If nothing failed, but we cancelled, exit with no data.
        else {
            Ok(None)
        }
    }

    /// This function loads the data we need for the editor to the view, inside items in the ListView.
    ///
    /// As key we use the faction column, using * as a replacement for empty faction.
    unsafe fn load_data(&self, data: &HashMap<String, String>) -> Result<()> {

        // The listView of the variant editor is supposed to represent all the different faction versions of each variant.
        for (_, value) in data.iter()
            .sorted_by_key(|x| x.0)
            .filter(|x| x.0.starts_with(VARIANT_KEY_VALUE)) {

            // If no variant is set, use a * to identify it.
            let value = if value.is_empty() { "*" } else { value };
            let item = QStandardItem::from_q_string(&QString::from_std_str(&value)).into_ptr();

            // Search on the unit data for this faction's data, and for the definitions.
            let faction_data = data.iter()
                .filter(|x| x.0.ends_with(value) || x.0.ends_with("_definition"))
                .map(|x| {

                    // If a definition is found, insert it.
                    if x.0.ends_with("_definition") {
                        (x.0.to_owned(), x.1.to_owned())
                    }

                    // If a field belonging to our faction (value) is found, format it so we get its individual column name, without extra data.
                    else {
                        let mut clean_key = x.0.to_owned();
                        if let Some(index) = clean_key.find(value) {
                            clean_key = clean_key.split_at(index - 1).0.to_owned();
                        }
                        (clean_key, x.1.to_owned())
                    }
                })
                .collect::<HashMap<String, String>>();

            // Then, load the faction data to the UI.
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&faction_data).unwrap())), UNIT_DATA);
            self.faction_list_model.append_row_q_standard_item(item);
        }

        self.get_unit_variants_colours_data(data)?;

        // Load icon and variantmesh paths.
        self.load_icon_paths()?;
        self.load_variant_mesh_paths()?;

        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_variants_colours table.
    unsafe fn get_unit_variants_colours_data(&self, data: &HashMap<String, String>) -> Result<()> {
        if let Some(definition) = data.get("unit_variants_colours_definition") {
            let definition = serde_json::from_str(definition).unwrap();
            let reference_data = get_reference_data("unit_variants_colours_tables", &definition)?;

            self.tool.load_reference_data_to_detailed_view_editor_combo(definition.get_column_position_by_name("faction")? as i32, &self.tool.find_widget("unit_variants_colours_faction_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(definition.get_column_position_by_name("soldier_type")? as i32, &self.tool.find_widget("unit_variants_colours_soldier_type_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(definition.get_column_position_by_name("subculture")? as i32, &self.tool.find_widget("unit_variants_colours_subculture_combobox")?, &reference_data);
        }

        Ok(())
    }

    /// This function loads all available icon paths to the UI.
    unsafe fn load_icon_paths(&self) -> Result<()> {
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesNamesStartingWitPathFromAllSources(PathType::Folder(UNIT_ICONS_PATH.split("/").map(|x| x.to_owned()).collect())));
        let response = CentralCommand::recv(&receiver);
        let icon_keys = if let Response::HashMapDataSourceHashSetVecString(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
        let icon_keys_sorted = icon_keys.values()
            .map(|paths|
                paths.par_iter()
                .map(|path| path.join("/"))
                .collect::<Vec<String>>()
            )
            .flatten()
            .sorted()
            .collect::<Vec<String>>();

        self.unit_variants_unit_card_combobox.add_item_q_string(&QString::from_std_str(""));
        for icon_key in &icon_keys_sorted {
            let name_without_extension = icon_key.split('.').collect::<Vec<&str>>()[0];
            self.unit_variants_unit_card_combobox.add_item_q_string(&QString::from_std_str(name_without_extension));
        }

        Ok(())
    }

    /// This function loads all available variantmesh paths to the UI.
    unsafe fn load_variant_mesh_paths(&self) -> Result<()> {
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesNamesStartingWitPathFromAllSources(PathType::Folder(VARIANT_MESH_PATH.split("/").map(|x| x.to_owned()).collect())));
        let response = CentralCommand::recv(&receiver);
        let variant_keys = if let Response::HashMapDataSourceHashSetVecString(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
        let variant_keys_sorted = variant_keys.values()
            .map(|paths|
                paths.par_iter()
                .map(|path| path.join("/"))
                .collect::<Vec<String>>()
            )
            .flatten()
            .sorted()
            .collect::<Vec<String>>();

        self.variants_variant_filename_combobox.add_item_q_string(&QString::from_std_str(""));
        for variant_key in &variant_keys_sorted {
            let name_without_extension = variant_key.split('.').collect::<Vec<&str>>()[0];
            self.variants_variant_filename_combobox.add_item_q_string(&QString::from_std_str(name_without_extension));
        }

        Ok(())
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first faction loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.detailed_view_widget.is_enabled() {
            self.detailed_view_widget.set_enabled(true);
        }

        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors: Vec<String> = vec![];

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "unit_variants", &UNIT_VARIANTS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "variants", &VARIANTS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        // Load custom entries from unit_variants.
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.unit_variants_unit_card_combobox, "unit_variants_unit_card");
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.variants_variant_filename_combobox, "variants_variant_filename");

        // The icon needs to be pulled up from the dependencies cache on load.
        self.load_unit_icon(&data, None);
        self.load_variant_mesh(&data, None);

        // Colours must be loaded into a list, with the same logic as the main faction list.
        self.load_unit_variants_colours(&data);

        // Once everything is loaded, disable the colours section until a colour variant is selected.
        self.unit_variants_colours_widget.set_enabled(false);

        // If we have any errors, show them here.
        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_unit_variants_colours_to_detailed_view(&self, index: Ref<QModelIndex>) {
        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors: Vec<String> = vec![];

        if !self.unit_variants_colours_widget.is_enabled() {
            self.unit_variants_colours_widget.set_enabled(true);
        }

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "unit_variants_colours", &UNIT_VARIANTS_COLOURS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.unit_variants_colours_faction_combobox, "unit_variants_colours_faction");
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.unit_variants_colours_subculture_combobox, "unit_variants_colours_subculture");
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.unit_variants_colours_soldier_type_combobox, "unit_variants_colours_soldier_type");

        self.tool.load_fields_to_detailed_view_editor_combo_color_split(&data, &self.unit_variants_colours_primary_colour_combobox, "unit_variants_colours_primary_colour_r", "unit_variants_colours_primary_colour_g", "unit_variants_colours_primary_colour_b");
        self.tool.load_fields_to_detailed_view_editor_combo_color_split(&data, &self.unit_variants_colours_secondary_colour_combobox, "unit_variants_colours_secondary_colour_r", "unit_variants_colours_secondary_colour_g", "unit_variants_colours_secondary_colour_b");
        self.tool.load_fields_to_detailed_view_editor_combo_color_split(&data, &self.unit_variants_colours_tertiary_colour_combobox, "unit_variants_colours_tertiary_colour_r", "unit_variants_colours_tertiary_colour_g", "unit_variants_colours_tertiary_colour_b");

        // If we have any errors, show them here.
        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }
    }

    /// This function loads the unit_variants_colours entries into a listview,
    pub unsafe fn load_unit_variants_colours(&self, data: &HashMap<String, String>) {
        data.iter()
            .sorted_by_key(|x| x.0)
            .filter(|x| x.0.starts_with("unit_variants_colours_key"))
            .for_each(|(_, value)| {

            let value = if value.is_empty() { "*" } else { value };
            let item = QStandardItem::from_q_string(&QString::from_std_str(&value)).into_ptr();
            let unit_variants_colours = data.iter()
                .filter(|x| x.0.ends_with(value) || x.0 == "unit_variants_colours_definition")
                .map(|x| {
                    if x.0.ends_with("_definition") {
                        (x.0.to_owned(), x.1.to_owned())
                    } else {
                        let mut clean_key = x.0.to_owned();
                        if let Some(index) = clean_key.find(value) {
                            clean_key = clean_key.split_at(index - 1).0.to_owned();
                        }
                        (clean_key, x.1.to_owned())
                    }
                })
                .collect::<HashMap<String, String>>();

            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&unit_variants_colours).unwrap())), UNIT_DATA);
            self.unit_variants_colours_list_model.append_row_q_standard_item(item);
        });
    }

    /// This function loads the unit icon into the tool. If provided with a key, it uses it. If not, it uses whatever key the unit has.
    pub unsafe fn load_unit_icon(&self, data: &HashMap<String, String>, key: Option<String>) {
        let unit_card = if let Some(unit_card) = key { Some(unit_card.to_owned()) }
        else if let Some(unit_card) = data.get("unit_variants_unit_card") { Some(unit_card.to_owned()) }
        else { None };

        // The icon needs to be pulled up from the dependencies cache on load.
        if let Some(unit_card) = unit_card {
            let icon_path_png_lowres = format!("{}{}.png", UNIT_ICONS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let icon_path_tga_lowres = format!("{}{}.tga", UNIT_ICONS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();

            let icon_paths = vec![
                PathType::File(icon_path_png_lowres.to_vec()),
                PathType::File(icon_path_tga_lowres.to_vec()),
            ];

            let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesFromAllSources(icon_paths));
            let response = CentralCommand::recv(&receiver);
            let images_data = if let Response::HashMapDataSourceHashMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
            let image_file = if let Some(image_file) = Tool::get_most_relevant_file(&images_data, &icon_path_png_lowres) {
                Some(image_file)
            } else if let Some(image_file) = Tool::get_most_relevant_file(&images_data, &icon_path_tga_lowres) {
                Some(image_file)
            } else {
                None
            };

            if let Some(image_file) = image_file {
                let image_data = image_file.get_raw_data().unwrap();
                let byte_array = QByteArray::from_slice(&image_data);
                let image = QPixmap::new();
                image.load_from_data_q_byte_array(&byte_array);
                self.unit_variants_unit_card_preview_label.set_pixmap(&image);
            } else {
                self.unit_variants_unit_card_preview_label.set_text(&QString::from_std_str("No image available"));
            }
        } else {
            self.unit_variants_unit_card_preview_label.set_text(&QString::from_std_str("No image available"));
        }
    }

    /// This function loads the variantmesh of a unit into the tool. If provided with a key, it uses it. If not, it uses whatever key the unit has.
    pub unsafe fn load_variant_mesh(&self, data: &HashMap<String, String>, key: Option<String>) {

        // If it's the initial load to detailed view (not edition of path), check if we have edited data already.
        if key.is_none() {
            if let Some(data) = data.get(VARIANT_MESH_DATA) {
                if !data.is_empty() {
                    set_text_safe(&self.variants_mesh_editor.static_upcast(), &QString::from_std_str(data).into_ptr(), &QString::from_std_str("XML").as_ptr());
                    return;
                }
            }
        }

        // If not, load it from the backend.
        let variant = if let Some(variant) = key { Some(variant.to_owned()) }
        else if let Some(variant) = data.get("variants_variant_filename") { Some(variant.to_owned()) }
        else { None };

        // The variant needs to be pulled up from the dependencies cache on load.
        if let Some(variant) = variant {
            let variant_path = format!("{}{}.{}", VARIANT_MESH_PATH, variant, VARIANT_MESH_EXTENSION).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let variant_paths = vec![
                PathType::File(variant_path.to_vec()),
            ];

            let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesFromAllSources(variant_paths));
            let response = CentralCommand::recv(&receiver);
            let variant_data = if let Response::HashMapDataSourceHashMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
            let file = if let Some(file) = Tool::get_most_relevant_file(&variant_data, &variant_path) {
                Some(file)
            } else {
                None
            };

            if let Some(mut file) = file {
                if let Ok(DecodedPackedFile::Text(data)) = file.decode_return_ref() {
                    set_text_safe(&self.variants_mesh_editor.static_upcast(), &QString::from_std_str(data.get_ref_contents()).as_ptr(), &QString::from_std_str("XML").as_ptr());
                } else {
                    set_text_safe(&self.variants_mesh_editor.static_upcast(), &QString::from_std_str("").as_ptr(), &QString::from_std_str("XML").as_ptr());
                }
            } else {
                set_text_safe(&self.variants_mesh_editor.static_upcast(), &QString::from_std_str("").as_ptr(), &QString::from_std_str("XML").as_ptr());
            }
        } else {
            set_text_safe(&self.variants_mesh_editor.static_upcast(), &QString::from_std_str("").as_ptr(), &QString::from_std_str("XML").as_ptr());
        }
    }

    /// This function takes care of saving the data of this Tool into the currently open PackFile, creating a new one if there wasn't one open.
    pub unsafe fn save_data(&self) -> Result<HashMap<String, String>> {

        // First, save whatever is currently open in the detailed view.
        self.faction_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.faction_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        // Get each faction's data as a HashMap of data/value.
        let data_to_save = (0..self.faction_list_model.row_count_0a())
            .map(|row| {
                let index = self.faction_list_model.index_2a(row, 0);
                let data = self.faction_list_model.data_2a(&index, UNIT_DATA);
                let data: HashMap<String, String> = serde_json::from_str(&data.to_string().to_std_string()).unwrap();

                let faction_key = self.faction_list_model.data_1a(&index).to_string().to_std_string();
                let data = data.iter().map(|(key, value)| {
                    let key = if !key.ends_with("_definition") { format!("{}|{}", key, &faction_key) } else { key.to_owned() };
                    (key, value.to_owned())
                }).collect::<HashMap<String, String>>();

                data
            })
            .flatten()
            .collect::<HashMap<String, String>>();

       Ok(data_to_save)
    }

    /// This function saves the data of the detailed view to its item in the faction list.
    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors: Vec<String> = vec![];

        //-----------------------------------------------------------------------//
        // unit_variants_colours_tables
        //
        // This one may contain multiple entries for each faction, so we have to
        // parse them before anything else, and map them to `key|faction` (* for
        // empty faction).
        //-----------------------------------------------------------------------//
        self.save_unit_variants_colours(&mut data);

        //-----------------------------------------------------------------------//
        // variants_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "variants", &VARIANTS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        //-----------------------------------------------------------------------//
        // unit_variants_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "unit_variants", &UNIT_VARIANTS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        self.tool.save_field_from_detailed_view_editor_combo(&mut data, &self.unit_variants_unit_card_combobox, "unit_variants_unit_card");
        self.tool.save_field_from_detailed_view_editor_combo(&mut data, &self.variants_variant_filename_combobox, "variants_variant_filename");

        // Save the variantmesh data, in case we edited.
        self.save_variant_mesh(&mut data);

        // Update all the referenced keys in our data.
        self.update_keys(&mut data);

        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }

        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }
        self.faction_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&data).unwrap())), UNIT_DATA);
    }

    /// This function saves the variant mesh contents to a role in the Faction ListView, so we can save it to a file later.
    pub unsafe fn save_variant_mesh(&self, data: &mut HashMap<String, String>) {
        let string = get_text_safe(&self.variants_mesh_editor).to_std_string();
        data.insert(VARIANT_MESH_DATA.to_owned(), string);
    }

    /// This function saves the unit_variants_colours entries from a listview into our faction entries.
    pub unsafe fn save_unit_variants_colours(&self, data: &mut HashMap<String, String>) {

        // Save the currently selected colour variant.
        self.unit_variants_colours_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.unit_variants_colours_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        // Remove all old entries here, before we pull them from the UI.
        data.retain(|key, _| !key.starts_with("unit_variants_colours_"));

        // Get the new entries from the ListView.
        let mut new_entries = HashMap::new();

        for index in 0..self.unit_variants_colours_list_model.row_count_0a() {
            let index = self.unit_variants_colours_list_model.index_2a(index, 0);
            let mut entry_data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
            let item_name = index.data_0a().to_string().to_std_string();
            let mapped_entry = entry_data.iter_mut().map(|(key, value)| {
                if key.ends_with("_definition") {
                    (key.to_owned(), value.to_owned())
                } else {
                    (format!("{}|{}", key, item_name), value.to_owned())
                }
            }).collect::<HashMap<String, String>>();
            new_entries.extend(mapped_entry);
        }

        data.extend(new_entries);

        // Clear the color variant list.
        self.unit_variants_colours_list_model.clear();
    }

    /// This function saves the unit_variants_colours entries from the UI to a listview.
    unsafe fn save_unit_variants_colours_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors: Vec<String> = vec![];

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "unit_variants_colours", &UNIT_VARIANTS_COLOURS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        self.tool.save_field_from_detailed_view_editor_combo(&mut data, &self.unit_variants_colours_faction_combobox, "unit_variants_colours_faction");
        self.tool.save_field_from_detailed_view_editor_combo(&mut data, &self.unit_variants_colours_subculture_combobox, "unit_variants_colours_subculture");
        self.tool.save_field_from_detailed_view_editor_combo(&mut data, &self.unit_variants_colours_soldier_type_combobox, "unit_variants_colours_soldier_type");

        self.tool.save_fields_from_detailed_view_editor_combo_color_split(&mut data, &self.unit_variants_colours_primary_colour_combobox, "unit_variants_colours_primary_colour_r", "unit_variants_colours_primary_colour_g", "unit_variants_colours_primary_colour_b");
        self.tool.save_fields_from_detailed_view_editor_combo_color_split(&mut data, &self.unit_variants_colours_secondary_colour_combobox, "unit_variants_colours_secondary_colour_r", "unit_variants_colours_secondary_colour_g", "unit_variants_colours_secondary_colour_b");
        self.tool.save_fields_from_detailed_view_editor_combo_color_split(&mut data, &self.unit_variants_colours_tertiary_colour_combobox, "unit_variants_colours_tertiary_colour_r", "unit_variants_colours_tertiary_colour_g", "unit_variants_colours_tertiary_colour_b");

        // If we have any errors, show them here.
        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }

        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        self.unit_variants_colours_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&data).unwrap())), UNIT_DATA);
    }

    /// Function to trigger certain delayed actions, like the filter.
    pub unsafe fn start_delayed_updates_timer(&self) {
        self.timer_delayed_updates.set_interval(500);
        self.timer_delayed_updates.start_0a();
    }

    /// Function to filter the faction list.
    pub unsafe fn filter_list(&self) {
        self.faction_list_filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        self.faction_list_filter.set_filter_regular_expression_q_string(&self.faction_list_filter_line_edit.text());
    }

    /// This function updates the reference keys in all values of an entry.
    unsafe fn update_keys(&self, data: &mut HashMap<String, String>) {
        self.tool.update_keys(data);
    }

    /// Function to setup all the translations of this view.
    pub unsafe fn setup_translations(&self) -> Result<()> {
        self.tool.find_widget::<QGroupBox>("variants_mesh_editor_groupbox")?.set_title(&qtr("variants_mesh_editor_title"));
        self.tool.find_widget::<QGroupBox>("unit_variants_colours_groupbox")?.set_title(&qtr("unit_variants_colours_title"));
        self.tool.find_widget::<QLabel>("unit_variants_unit_card_label")?.set_text(&qtr("unit_variants_unit_card"));
        self.tool.find_widget::<QLabel>("variants_variant_filename_label")?.set_text(&qtr("variants_variant_filename"));

        self.tool.find_widget::<QLabel>("unit_variants_colours_primary_colour_label")?.set_text(&qtr("unit_variants_colours_primary_colour"));
        self.tool.find_widget::<QLabel>("unit_variants_colours_secondary_colour_label")?.set_text(&qtr("unit_variants_colours_secondary_colour"));
        self.tool.find_widget::<QLabel>("unit_variants_colours_tertiary_colour_label")?.set_text(&qtr("unit_variants_colours_tertiary_colour"));

        self.tool.find_widget::<QLabel>("faction_list_label")?.set_text(&qtr("faction_list_title"));
        self.tool.find_widget::<QLabel>("unit_variants_colours_list_label")?.set_text(&qtr("unit_variants_colours_list_title"));

        Ok(())
    }
}
