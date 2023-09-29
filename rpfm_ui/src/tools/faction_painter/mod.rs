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
Module with all the code for managing the Faction Painter tool.

This tool is a simple dialog, where you can choose a faction from a list, and change some of its colours.
!*/

use qt_widgets::QComboBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QPushButton;

use qt_gui::QColor;
use qt_gui::QIcon;
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

use getset::*;
use itertools::Itertools;
use rayon::prelude::*;

use std::collections::HashMap;

use rpfm_lib::files::{ContainerPath, db::DB, RFileDecoded, table::DecodedData};
use rpfm_lib::games::supported_games::*;

use rpfm_ui_common::locale::{tr, qtr};

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;

use self::slots::ToolFactionPainterSlots;
use super::*;
use super::error::ToolsError;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_faction_color_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_faction_color_editor.ui";

/// Role that stores the data corresponding to the faction of each item.
const FACTION_DATA: i32 = 60;

/// Role that stores the icon of the faction represented by each item.
const FACTION_ICON: i32 = 61;

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 10] = [
    KEY_PHARAOH,
    KEY_WARHAMMER_3,
    KEY_TROY,
    KEY_THREE_KINGDOMS,
    KEY_WARHAMMER_2,
    KEY_WARHAMMER,
    KEY_THRONES_OF_BRITANNIA,
    KEY_ATTILA,
    KEY_ROME_2,
    KEY_SHOGUN_2,
];

/// Default name for files saved with this tool.
const DEFAULT_FILENAME: &str = "faction_painter_edited";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the widgets used by the `Faction Painter` Tool, along with some data needed for the view to work.
#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ToolFactionPainter {
    tool: Tool,
    timer_delayed_updates: QBox<QTimer>,
    faction_list_view: QPtr<QListView>,
    faction_list_filter: QBox<QSortFilterProxyModel>,
    faction_list_model: QBox<QStandardItemModel>,
    faction_list_filter_line_edit: QPtr<QLineEdit>,
    faction_name_label: QPtr<QLabel>,
    faction_icon_label: QPtr<QLabel>,
    banner_groupbox: QPtr<QGroupBox>,
    banner_colour_primary_label: QPtr<QLabel>,
    banner_colour_secondary_label: QPtr<QLabel>,
    banner_colour_tertiary_label: QPtr<QLabel>,
    banner_colour_primary: QPtr<QComboBox>,
    banner_colour_secondary: QPtr<QComboBox>,
    banner_colour_tertiary: QPtr<QComboBox>,
    banner_restore_initial_values_button: QPtr<QPushButton>,
    banner_restore_vanilla_values_button: QPtr<QPushButton>,
    uniform_groupbox: QPtr<QGroupBox>,
    uniform_colour_primary_label: QPtr<QLabel>,
    uniform_colour_secondary_label: QPtr<QLabel>,
    uniform_colour_tertiary_label: QPtr<QLabel>,
    uniform_colour_primary: QPtr<QComboBox>,
    uniform_colour_secondary: QPtr<QComboBox>,
    uniform_colour_tertiary: QPtr<QComboBox>,
    uniform_restore_initial_values_button: QPtr<QPushButton>,
    uniform_restore_vanilla_values_button: QPtr<QPushButton>,
    packed_file_name_label: QPtr<QLabel>,
    packed_file_name_line_edit: QPtr<QLineEdit>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ToolFactionPainter`.
impl ToolFactionPainter {

    /// This function creates the tool's dialog.
    ///
    /// NOTE: This can fail at runtime if any of the expected widgets is not in the UI's XML.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) -> Result<()> {

        // Initialize a Tool. This also performs some common checks to ensure we can actually use the tool.
        // TODO: Move this to a tool var.
        let paths = match &*GAME_SELECTED.read().unwrap().key() {
            KEY_WARHAMMER_3 => vec![
                ContainerPath::Folder("db/factions_tables".to_owned()),
                ContainerPath::Folder("text".to_owned()),
            ],
            _ => vec![
                ContainerPath::Folder("db/factions_tables".to_owned()),
                ContainerPath::Folder("db/faction_banners_tables".to_owned()),
                ContainerPath::Folder("db/faction_uniform_colours_tables".to_owned()),
                ContainerPath::Folder("text".to_owned()),
            ]
        };

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let tool = Tool::new(app_ui.main_window(), &paths, &TOOL_SUPPORTED_GAMES, view)?;
        tool.set_title(&tr("faction_painter_title"));
        tool.backup_used_paths(app_ui, pack_file_contents_ui)?;

        // ListView.
        let faction_list_view: QPtr<QListView> = tool.find_widget("faction_list_view")?;
        let faction_list_filter_line_edit: QPtr<QLineEdit> = tool.find_widget("faction_list_filter_line_edit")?;

        // Details view.
        let faction_icon_label: QPtr<QLabel> = tool.find_widget("faction_icon_label")?;
        let faction_name_label: QPtr<QLabel> = tool.find_widget("faction_name_label")?;

        // Banner GroupBox.
        let banner_groupbox: QPtr<QGroupBox> = tool.find_widget("banner_groupbox")?;
        let banner_colour_primary_label: QPtr<QLabel> = tool.find_widget("banner_colour_primary_label")?;
        let banner_colour_secondary_label: QPtr<QLabel> = tool.find_widget("banner_colour_secondary_label")?;
        let banner_colour_tertiary_label: QPtr<QLabel> = tool.find_widget("banner_colour_tertiary_label")?;
        let banner_colour_primary: QPtr<QComboBox> = tool.find_widget("banner_colour_primary")?;
        let banner_colour_secondary: QPtr<QComboBox> = tool.find_widget("banner_colour_secondary")?;
        let banner_colour_tertiary: QPtr<QComboBox> = tool.find_widget("banner_colour_tertiary")?;
        let banner_restore_initial_values_button: QPtr<QPushButton> = tool.find_widget("banner_restore_initial_values_button")?;
        let banner_restore_vanilla_values_button: QPtr<QPushButton> = tool.find_widget("banner_restore_vanilla_values_button")?;

        // Uniform GroupBox.
        let uniform_groupbox: QPtr<QGroupBox> = tool.find_widget("uniform_groupbox")?;
        let uniform_colour_primary_label: QPtr<QLabel> = tool.find_widget("uniform_colour_primary_label")?;
        let uniform_colour_secondary_label: QPtr<QLabel> = tool.find_widget("uniform_colour_secondary_label")?;
        let uniform_colour_tertiary_label: QPtr<QLabel> = tool.find_widget("uniform_colour_tertiary_label")?;
        let uniform_colour_primary: QPtr<QComboBox> = tool.find_widget("uniform_colour_primary")?;
        let uniform_colour_secondary: QPtr<QComboBox> = tool.find_widget("uniform_colour_secondary")?;
        let uniform_colour_tertiary: QPtr<QComboBox> = tool.find_widget("uniform_colour_tertiary")?;
        let uniform_restore_initial_values_button: QPtr<QPushButton> = tool.find_widget("uniform_restore_initial_values_button")?;
        let uniform_restore_vanilla_values_button: QPtr<QPushButton> = tool.find_widget("uniform_restore_vanilla_values_button")?;

        let packed_file_name_label: QPtr<QLabel> = tool.find_widget("packed_file_name_label")?;
        let packed_file_name_line_edit: QPtr<QLineEdit> = tool.find_widget("packed_file_name_line_edit")?;
        packed_file_name_line_edit.set_text(&QString::from_std_str(DEFAULT_FILENAME));

        // Extra stuff.
        let faction_list_filter = QSortFilterProxyModel::new_1a(&faction_list_view);
        let faction_list_model = QStandardItemModel::new_1a(&faction_list_filter);
        faction_list_view.set_model(&faction_list_filter);
        faction_list_filter.set_source_model(&faction_list_model);

        // Filter timer.
        let timer_delayed_updates = QTimer::new_1a(tool.main_widget());
        timer_delayed_updates.set_single_shot(true);

        // Build the view itself.
        let view = Rc::new(Self{
            tool,
            timer_delayed_updates,
            faction_list_view,
            faction_list_filter,
            faction_list_model,
            faction_list_filter_line_edit,
            faction_icon_label,
            faction_name_label,
            banner_groupbox,
            banner_colour_primary_label,
            banner_colour_secondary_label,
            banner_colour_tertiary_label,
            banner_colour_primary,
            banner_colour_secondary,
            banner_colour_tertiary,
            banner_restore_initial_values_button,
            banner_restore_vanilla_values_button,
            uniform_groupbox,
            uniform_colour_primary_label,
            uniform_colour_secondary_label,
            uniform_colour_tertiary_label,
            uniform_colour_primary,
            uniform_colour_secondary,
            uniform_colour_tertiary,
            uniform_restore_initial_values_button,
            uniform_restore_vanilla_values_button,
            packed_file_name_label,
            packed_file_name_line_edit,
        });

        // Build the slots and connect them to the view.
        let slots = ToolFactionPainterSlots::new(&view);
        connections::set_connections(&view, &slots);

        // Setup text translations.
        view.setup_translations();

        // Load all the data to the view.
        view.load_data()?;

        // If we hit ok, save the data back to the Pack.
        if view.tool.get_ref_dialog().exec() == 1 {
            view.save_data(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui)?;
        }

        // If nothing failed, it means we have successfully saved the data back to disk, or canceled.
        Ok(())
    }

    /// This function loads the data we need for the faction painter to the view, inside items in the ListView.
    unsafe fn load_data(&self) -> Result<()> {

        // Note: this data is HashMap<DataSource, HashMap<Path, RFile>>.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(self.tool.used_paths.to_vec(), false));
        let response = CentralCommand::recv(&receiver);
        let mut data = if let Response::HashMapDataSourceHashMapStringRFile(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

        let mut processed_data = HashMap::new();

        // Get the table's data.
        get_data_from_all_sources!(self, get_faction_data, data, processed_data, true);
        get_data_from_all_sources!(self, get_faction_loc_data, data, processed_data, true);

        if self.tool.used_paths().contains(&ContainerPath::Folder("db/faction_banners_tables".to_owned())) {
            get_data_from_all_sources!(self, get_faction_banner_data, data, processed_data, true);
        }
        if self.tool.used_paths().contains(&ContainerPath::Folder("db/faction_uniform_colours_tables".to_owned())) {
            get_data_from_all_sources!(self, get_faction_uniform_data, data, processed_data, true);
        }

        // Finally, grab the flag files. For that, get the paths from each faction's data, and request the flag icons.
        // These flag paths are already pre-processed to contain their full icon path, and a common slash format.
        let paths_to_use = processed_data.values()
            .filter_map(|x| x.get("flags_path"))
            .filter_map(|x| if !x.is_empty() { Some(ContainerPath::File(x.to_owned())) } else { None })
            .collect::<Vec<ContainerPath>>();

        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(paths_to_use, false));
        let response = CentralCommand::recv(&receiver);
        let images_data = if let Response::HashMapDataSourceHashMapStringRFile(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

        // Map the paths to be a single string, lowercase. That should speed-up things.
        let mut images_data: HashMap<DataSource, HashMap<String, RFile>> = images_data.iter().map(|(x, y)| (*x, y.par_iter().map(|(path, z)| (path.to_lowercase(), z.clone())).collect())).collect();

        // Once we got everything processed, build the items for the ListView.
        for (key, data) in processed_data.iter_mut().sorted_by_key(|x| x.0) {
            let item = QStandardItem::from_q_string(&QString::from_std_str(format!("{} - {}", data.get("screen_name").unwrap(), key))).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(data).unwrap())), FACTION_DATA);

            // Image paths, we may or may not have them, so only try to load them if we actually have a path for them.
            if let Some(image_path) = data.get("flags_path") {
                let image_path_lowercase = image_path.to_lowercase();
                let mut image_data = None;

                if let Some(data) = images_data.get_mut(&DataSource::PackFile) {
                    if let Some(image_packed_file) = data.get_mut(&image_path_lowercase) {
                        if let Some(RFileDecoded::Image(decoded)) = image_packed_file.decode(&None, false, true)? {
                            image_data = Some(decoded.data().to_vec());
                        }
                    }
                }
                if image_data.is_none() {
                    if let Some(data) = images_data.get_mut(&DataSource::ParentFiles) {
                        if let Some(image_packed_file) = data.get_mut(&image_path_lowercase) {
                            if let Some(RFileDecoded::Image(decoded)) = image_packed_file.decode(&None, false, true)? {
                                image_data = Some(decoded.data().to_vec());
                            }
                        }
                    }
                }
                if image_data.is_none() {
                    if let Some(data) = images_data.get_mut(&DataSource::GameFiles) {
                        if let Some(image_packed_file) = data.get_mut(&image_path_lowercase) {
                            if let Some(RFileDecoded::Image(decoded)) = image_packed_file.decode(&None, false, true)? {
                                image_data = Some(decoded.data().to_vec());
                            }
                        }
                    }
                }

                // If we got an image, load it into an icon, and load its raw data into the item, for future use.
                if let Some(image_data) = image_data {
                    let byte_array = QByteArray::from_slice(&image_data);
                    let image = QPixmap::new();
                    image.load_from_data_q_byte_array(&byte_array);
                    item.set_icon(&QIcon::from_q_pixmap(&image));

                    // Store the icon for future use.
                    item.set_data_2a(&QVariant::from_q_byte_array(&byte_array), FACTION_ICON);
                }
            }

            // Finally, add the item to the list.
            self.faction_list_model.append_row_q_standard_item(item);
        }

        self.faction_list_filter.sort_1a(0);

        // Store the RFiles for use when saving.
        *self.tool.packed_files.borrow_mut() = data;
        Ok(())
    }

    /// This function takes care of saving the data of this Tool into the currently open Pack, creating a new one if there wasn't one open.
    pub unsafe fn save_data(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>
    ) -> Result<()> {

        // First, save whatever is currently open in the detailed view.
        self.faction_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.faction_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        // Get each faction's data as a HashMap of data/value.
        let data_to_save = (0..self.faction_list_model.row_count_0a())
            .map(|row|
                serde_json::from_str(
                    &self.faction_list_model.data_2a(
                        &self.faction_list_model.index_2a(row, 0),
                        FACTION_DATA
                    ).to_string().to_std_string()
                ).unwrap())
            .collect::<Vec<HashMap<String, String>>>();

        // We have to save the data to the last entry of the keys in out list, so if any of the other fields is edited on it, that edition is kept.
        let mut files_to_save = vec![];
        match &*GAME_SELECTED.read().unwrap().key() {
            KEY_WARHAMMER_3 => {
                files_to_save.push(self.save_factions_data(&data_to_save)?);
            }
            _ => {
                files_to_save.push(self.save_faction_banner_data(&data_to_save)?);
                files_to_save.push(self.save_faction_uniform_data(&data_to_save)?);
            }
        };

        // Once we got the RFiles to save properly edited, call the generic tool `save` function to save them to a Pack.
        self.tool.save(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, &files_to_save)
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first faction loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.banner_groupbox.is_enabled() {
            self.banner_groupbox.set_enabled(true);
        }
        if !self.uniform_groupbox.is_enabled() {
            self.uniform_groupbox.set_enabled(true);
        }

        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();
        self.tool.load_field_to_detailed_view_editor_string_label(&data, self.faction_name_label(), "screen_name");

        let image = QPixmap::new();
        image.load_from_data_q_byte_array(&index.data_1a(FACTION_ICON).to_byte_array());
        self.faction_icon_label().set_pixmap(&image);

        // From here, everything can not exits, depending on our tables.
        let mut missing_fields = vec![];

        if data.get("banner_primary").is_some() {
            self.banner_groupbox.set_checked(true);

            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.banner_colour_primary(), "banner_primary") {
                missing_fields.push(field);
            }
            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.banner_colour_secondary(), "banner_secondary") {
                missing_fields.push(field);
            }
            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.banner_colour_tertiary(), "banner_tertiary") {
                missing_fields.push(field);
            }
        } else {
            self.banner_groupbox.set_checked(false);
        }

        if data.get("uniform_primary").is_some() {
            self.uniform_groupbox.set_checked(true);

            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.uniform_colour_primary(), "uniform_primary") {
                missing_fields.push(field);
            }
            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.uniform_colour_secondary(), "uniform_secondary") {
                missing_fields.push(field);
            }
            if let Some(field) = self.tool.load_fields_to_detailed_view_editor_combo_color(&data, self.uniform_colour_tertiary(), "uniform_tertiary") {
                missing_fields.push(field);
            }
        } else {
            self.uniform_groupbox.set_checked(false);
        }

        // If any of the fields failed, report it.
        if !missing_fields.is_empty() {
            show_message_warning(&self.tool.message_widget, ToolsError::ToolEntryDataNotFound(missing_fields.join(", ")));
        }
    }

    /// This function saves the data of the detailed view to its item in the faction list.
    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        // Only save if checked. If not, remove the data we have, if we have any.
        if self.banner_groupbox.is_checked() {
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.banner_colour_primary(), "banner_primary");
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.banner_colour_secondary(), "banner_secondary");
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.banner_colour_tertiary(), "banner_tertiary");
        } else {
            data.remove("banner_primary");
            data.remove("banner_secondary");
            data.remove("banner_tertiary");
        }

        if self.uniform_groupbox.is_checked() {
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.uniform_colour_primary(), "uniform_primary");
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.uniform_colour_secondary(), "uniform_secondary");
            self.tool.save_fields_from_detailed_view_editor_combo_color(&mut data, self.uniform_colour_tertiary(), "uniform_tertiary");
        } else {
            data.remove("uniform_primary");
            data.remove("uniform_secondary");
            data.remove("uniform_tertiary");
        }

        self.faction_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), FACTION_DATA);
    }

    /// This function restores the banner colours to its initial values when we opened the tool.
    pub unsafe fn banner_restore_initial_values(&self) {
        let index = self.faction_list_filter.map_to_source(&self.faction_list_view.selection_model().current_index());
        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        if let Some(banner_primary) = data.get("banner_initial_primary") {
            set_color_safe(&self.banner_colour_primary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_primary))).as_ptr());
        }
        if let Some(banner_secondary) = data.get("banner_initial_secondary") {
            set_color_safe(&self.banner_colour_secondary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_secondary))).as_ptr());
        }
        if let Some(banner_tertiary) = data.get("banner_initial_tertiary") {
            set_color_safe(&self.banner_colour_tertiary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_tertiary))).as_ptr());
        }
    }

    /// This function restores the banner colours to its vanilla values when we opened the tool.
    ///
    /// Note: This one can fail if the faction is custom and not in the game files. The button should already be disabled
    /// in those cases, but we also control it here, just in case.
    pub unsafe fn banner_restore_vanilla_values(&self) {
        let index = self.faction_list_filter.map_to_source(&self.faction_list_view.selection_model().current_index());
        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        if let Some(banner_primary) = data.get("banner_vanilla_primary") {
            set_color_safe(&self.banner_colour_primary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_primary))).as_ptr());
        }
        if let Some(banner_secondary) = data.get("banner_vanilla_secondary") {
            set_color_safe(&self.banner_colour_secondary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_secondary))).as_ptr());
        }
        if let Some(banner_tertiary) = data.get("banner_vanilla_tertiary") {
            set_color_safe(&self.banner_colour_tertiary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", banner_tertiary))).as_ptr());
        }
    }

    /// This function restores the uniform colours to its initial values when we opened the tool.
    pub unsafe fn uniform_restore_initial_values(&self) {
        let index = self.faction_list_filter.map_to_source(&self.faction_list_view.selection_model().current_index());
        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        if let Some(uniform_primary) = data.get("uniform_initial_primary") {
            set_color_safe(&self.uniform_colour_primary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_primary))).as_ptr());
        }
        if let Some(uniform_secondary) = data.get("uniform_initial_secondary") {
            set_color_safe(&self.uniform_colour_secondary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_secondary))).as_ptr());
        }
        if let Some(uniform_tertiary) = data.get("uniform_initial_tertiary") {
            set_color_safe(&self.uniform_colour_tertiary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_tertiary))).as_ptr());
        }
    }

    /// This function restores the uniform colours to its vanilla values when we opened the tool.
    ///
    /// Note: This one can fail if the faction is custom and not in the game files. The button should already be disabled
    /// in those cases, but we also control it here, just in case.
    pub unsafe fn uniform_restore_vanilla_values(&self) {
        let index = self.faction_list_filter.map_to_source(&self.faction_list_view.selection_model().current_index());
        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        if let Some(uniform_primary) = data.get("uniform_vanilla_primary") {
            set_color_safe(&self.uniform_colour_primary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_primary))).as_ptr());
        }
        if let Some(uniform_secondary) = data.get("uniform_vanilla_secondary") {
            set_color_safe(&self.uniform_colour_secondary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_secondary))).as_ptr());
        }
        if let Some(uniform_tertiary) = data.get("uniform_vanilla_tertiary") {
            set_color_safe(&self.uniform_colour_tertiary().as_ptr().static_upcast(), &QColor::from_q_string(&QString::from_std_str(format!("#{}", uniform_tertiary))).as_ptr());
        }
    }

    /// Function to trigger certain delayed actions, like the filter.
    pub unsafe fn start_delayed_updates_timer(&self) {
        self.timer_delayed_updates.set_interval(500);
        self.timer_delayed_updates.start_0a();
    }

    /// Function to filter the faction list.
    pub unsafe fn filter_list(&self) {

        // So, funny bug: if we "hide" with a filter the open item, the entire thing crashes. Not sure why.
        // So we have to "unselect" it, filter, then check if it's still visible, and select it again.
        self.faction_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.faction_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        self.faction_list_filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        self.faction_list_filter.set_filter_regular_expression_q_string(&self.faction_list_filter_line_edit.text());

        self.faction_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.faction_list_view.selection_model().selection(), SelectionFlag::Toggle.into());
    }

    /// Function to setup all the translations of this view.
    pub unsafe fn setup_translations(&self) {
        self.banner_groupbox.set_title(&qtr("banner"));
        self.uniform_groupbox.set_title(&qtr("uniform"));

        self.banner_colour_primary_label.set_text(&qtr("primary"));
        self.banner_colour_secondary_label.set_text(&qtr("secondary"));
        self.banner_colour_tertiary_label.set_text(&qtr("tertiary"));

        self.uniform_colour_primary_label.set_text(&qtr("primary"));
        self.uniform_colour_secondary_label.set_text(&qtr("secondary"));
        self.uniform_colour_tertiary_label.set_text(&qtr("tertiary"));

        self.banner_restore_initial_values_button.set_text(&qtr("restore_initial_values"));
        self.banner_restore_vanilla_values_button.set_text(&qtr("restore_vanilla_values"));

        self.uniform_restore_initial_values_button.set_text(&qtr("restore_initial_values"));
        self.uniform_restore_vanilla_values_button.set_text(&qtr("restore_vanilla_values"));

        self.packed_file_name_label.set_text(&qtr("packed_file_name"));
    }

    /// This function gets the data needed for the tool from the factions table.
    unsafe fn get_faction_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>, data_source: DataSource) -> Result<()> {
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_factions_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_factions_row_key".to_owned()))?;

        let banner_primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_primary_colour_column_name".to_owned()))?;
        let banner_secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_secondary_colour_column_name".to_owned()))?;
        let banner_tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_tertiary_colour_column_name".to_owned()))?;

        let uniform_primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_primary_colour_column_name".to_owned()))?;
        let uniform_secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_secondary_colour_column_name".to_owned()))?;
        let uniform_tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_tertiary_colour_column_name".to_owned()))?;

        // First, get the keys, names and flags from the factions tables.
        for (path, packed_file) in data.iter_mut() {
            if path.to_lowercase().starts_with("db/factions_tables/") {
                if let Ok(RFileDecoded::DB(table)) = packed_file.decoded() {

                    // We need multiple column's data for this to work.
                    let key_column = table.column_position_by_name("key").ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), "key".to_string()))?;
                    let flag_path_column = table.column_position_by_name("flags_path").ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), "flags_path".to_string()))?;

                    // Only used for WH3.
                    let banner_primary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(banner_primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_primary_colour_column_name.to_string()))? } else { 0 };
                    let banner_secondary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(banner_secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_secondary_colour_column_name.to_string()))? } else { 0 };
                    let banner_tertiary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(banner_tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_tertiary_colour_column_name.to_string()))? } else { 0 };

                    let uniform_primary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(uniform_primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_primary_colour_column_name.to_string()))? } else { 0 };
                    let uniform_secondary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(uniform_secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_secondary_colour_column_name.to_string()))? } else { 0 };
                    let uniform_tertiary_colour_column = if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 { table.column_position_by_name(uniform_tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_tertiary_colour_column_name.to_string()))? } else { 0 };

                    let definition = serde_json::to_string(table.definition())?;
                    for row in table.data().iter() {
                        let mut data = HashMap::new();

                        match Tool::get_row_by_column_index(row, flag_path_column)? {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => {

                                // This cleans up the \ slashes some paths use, and removes the "data\" prefix some games use in these paths.
                                let mut value = value.to_owned().replace('\\', "/") + "/mon_64.png";
                                if value.starts_with("data/") {
                                    value = value[5..].to_owned();
                                }
                                data.insert("flags_path".to_owned(), value);
                            }
                            _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                        }

                        match Tool::get_row_by_column_index(row, key_column)? {
                            DecodedData::StringU8(ref key) |
                            DecodedData::StringU16(ref key) |
                            DecodedData::OptionalStringU8(ref key) |
                            DecodedData::OptionalStringU16(ref key) => {
                                data.insert("key".to_owned(), key.to_owned());
                            }
                            _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                        }

                        // In WH3 the 3 tables were merged into factions, so we have to check here for their data
                        if GAME_SELECTED.read().unwrap().key() == KEY_WARHAMMER_3 {
                            let banner_primary_row_by_column = Tool::get_row_by_column_index(row, banner_primary_colour_column)?;
                            let banner_secondary_row_by_column = Tool::get_row_by_column_index(row, banner_secondary_colour_column)?;
                            let banner_tertiary_row_by_column = Tool::get_row_by_column_index(row, banner_tertiary_colour_column)?;

                            let uniform_primary_row_by_column = Tool::get_row_by_column_index(row, uniform_primary_colour_column)?;
                            let uniform_secondary_row_by_column = Tool::get_row_by_column_index(row, uniform_secondary_colour_column)?;
                            let uniform_tertiary_row_by_column = Tool::get_row_by_column_index(row, uniform_tertiary_colour_column)?;

                            let banner_primary_colour = match banner_primary_row_by_column {
                                DecodedData::ColourRGB(_) => banner_primary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let banner_secondary_colour = match banner_secondary_row_by_column {
                                DecodedData::ColourRGB(_) => banner_secondary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let banner_tertiary_colour = match banner_tertiary_row_by_column {
                                DecodedData::ColourRGB(_) => banner_tertiary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };

                            let uniform_primary_colour = match uniform_primary_row_by_column {
                                DecodedData::ColourRGB(_) => uniform_primary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let uniform_secondary_colour = match uniform_secondary_row_by_column {
                                DecodedData::ColourRGB(_) => uniform_secondary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let uniform_tertiary_colour = match uniform_tertiary_row_by_column {
                                DecodedData::ColourRGB(_) => uniform_tertiary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };

                            // If we're processing the game files, set the vanilla values.
                            if let DataSource::GameFiles = data_source {
                                data.insert("banner_vanilla_primary".to_owned(), banner_primary_colour.to_string());
                                data.insert("banner_vanilla_secondary".to_owned(), banner_secondary_colour.to_string());
                                data.insert("banner_vanilla_tertiary".to_owned(), banner_tertiary_colour.to_string());

                                data.insert("uniform_vanilla_primary".to_owned(), uniform_primary_colour.to_string());
                                data.insert("uniform_vanilla_secondary".to_owned(), uniform_secondary_colour.to_string());
                                data.insert("uniform_vanilla_tertiary".to_owned(), uniform_tertiary_colour.to_string());
                            }

                            // Set the initial values. The last value inputted is the initial one due to how we load the data.
                            data.insert("banner_initial_primary".to_owned(), banner_primary_colour.to_string());
                            data.insert("banner_initial_secondary".to_owned(), banner_secondary_colour.to_string());
                            data.insert("banner_initial_tertiary".to_owned(), banner_tertiary_colour.to_string());
                            data.insert("banner_primary".to_owned(), banner_primary_colour.to_string());
                            data.insert("banner_secondary".to_owned(), banner_secondary_colour.to_string());
                            data.insert("banner_tertiary".to_owned(), banner_tertiary_colour.to_string());

                            data.insert("uniform_initial_primary".to_owned(), uniform_primary_colour.to_string());
                            data.insert("uniform_initial_secondary".to_owned(), uniform_secondary_colour.to_string());
                            data.insert("uniform_initial_tertiary".to_owned(), uniform_tertiary_colour.to_string());
                            data.insert("uniform_primary".to_owned(), uniform_primary_colour.to_string());
                            data.insert("uniform_secondary".to_owned(), uniform_secondary_colour.to_string());
                            data.insert("uniform_tertiary".to_owned(), uniform_tertiary_colour.to_string());
                        }

                        // Also save the full row, so we can easely edit it and put it into a file later on.
                        if data.get(row_key).is_none() {
                            data.insert(row_key.to_owned(), serde_json::to_string(row)?);
                        }

                        // Store the definition, so we can re-use it later to recreate the table.
                        // TODO: change this so it's not done on EVERY SINGLE ROW.
                        if data.get("factions_definition").is_none() {
                            data.insert("factions_definition".to_owned(), definition.to_owned());
                        }

                        if let Some(key) = data.get("key") {
                            processed_data.insert(key.to_owned(), data);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// This function gets the data needed for the tool from the factions table.
    unsafe fn get_faction_loc_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>, _data_source: DataSource) -> Result<()> {
        for (path, packed_file) in data.iter_mut() {
            if path.to_lowercase().ends_with(".loc") {
                if let Ok(RFileDecoded::Loc(table)) = packed_file.decoded() {
                    let base_name = "factions_screen_name_".to_owned();
                    let table_data = table.data();

                    processed_data.iter_mut().for_each(|(key, values)| {
                        let key = format!("{}{}", base_name, key);
                        let screen_name = table_data.iter().find_map(|row| {
                            if row[0].data_to_string() == key {
                                Some(row[1].data_to_string())
                            } else {
                                None
                            }
                        });

                        if let Some(screen_name) = screen_name {
                            values.insert("screen_name".to_string(), screen_name.to_string());
                        }
                    });
                }
            }
        }
        Ok(())
    }

    /// This function gets the data needed for the tool from the faction_banners table.
    unsafe fn get_faction_banner_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>, data_source: DataSource) -> Result<()> {

        let table_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_table_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_table_name".to_owned()))?;
        let table_definition_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_table_definition").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_table_definition".to_owned()))?;
        let key_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_key_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_key_column_name".to_owned()))?;
        let primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_primary_colour_column_name".to_owned()))?;
        let secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_secondary_colour_column_name".to_owned()))?;
        let tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_tertiary_colour_column_name".to_owned()))?;
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_row_key".to_owned()))?;

        for (path, packed_file) in data.iter_mut() {
            if path.to_lowercase().starts_with(&format!("db/{}/", table_name)) {

                if let Ok(RFileDecoded::DB(table)) = packed_file.decoded() {

                    // We need multiple column's data for this to work.
                    let key_column = table.column_position_by_name(key_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), key_column_name.to_string()))?;

                    let primary_colour_column = table.column_position_by_name(primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), primary_colour_column_name.to_string()))?;
                    let secondary_colour_column = table.column_position_by_name(secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), secondary_colour_column_name.to_string()))?;
                    let tertiary_colour_column = table.column_position_by_name(tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), tertiary_colour_column_name.to_string()))?;

                    for row in table.data().iter() {
                        let key = match Tool::get_row_by_column_index(row, key_column)? {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                        };

                        if let Some(faction_data) = processed_data.get_mut(key) {
                            let primary_row_by_column = Tool::get_row_by_column_index(row, primary_colour_column)?;
                            let secondary_row_by_column = Tool::get_row_by_column_index(row, secondary_colour_column)?;
                            let tertiary_row_by_column = Tool::get_row_by_column_index(row, tertiary_colour_column)?;

                            let primary_colour = match primary_row_by_column {
                                DecodedData::ColourRGB(_) => primary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let secondary_colour = match secondary_row_by_column {
                                DecodedData::ColourRGB(_) => secondary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let tertiary_colour = match tertiary_row_by_column {
                                DecodedData::ColourRGB(_) => tertiary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };

                            // If we're processing the game files, set the vanilla values.
                            if let DataSource::GameFiles = data_source {
                                faction_data.insert("banner_vanilla_primary".to_owned(), primary_colour.to_string());
                                faction_data.insert("banner_vanilla_secondary".to_owned(), secondary_colour.to_string());
                                faction_data.insert("banner_vanilla_tertiary".to_owned(), tertiary_colour.to_string());
                            }

                            // Set the initial values. The last value inputted is the initial one due to how we load the data.
                            faction_data.insert("banner_initial_primary".to_owned(), primary_colour.to_string());
                            faction_data.insert("banner_initial_secondary".to_owned(), secondary_colour.to_string());
                            faction_data.insert("banner_initial_tertiary".to_owned(), tertiary_colour.to_string());
                            faction_data.insert("banner_primary".to_owned(), primary_colour.to_string());
                            faction_data.insert("banner_secondary".to_owned(), secondary_colour.to_string());
                            faction_data.insert("banner_tertiary".to_owned(), tertiary_colour.to_string());

                            // Also save the full row, so we can easely edit it and put it into a file later on.
                            if faction_data.get(row_key).is_none() {
                                faction_data.insert(row_key.to_owned(), serde_json::to_string(row)?);
                            }

                            // Store the definition, so we can re-use it later to recreate the table.
                            if faction_data.get(table_definition_name).is_none() {
                                let definition = serde_json::to_string(table.definition())?;
                                faction_data.insert(table_definition_name.to_owned(), definition);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// This function gets the data needed for the tool from the faction_uniform_colours table.
    unsafe fn get_faction_uniform_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>, data_source: DataSource) -> Result<()> {

        let table_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_table_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_table_name".to_owned()))?;
        let table_definition_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_table_definition").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_table_definition".to_owned()))?;
        let key_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_key_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_key_column_name".to_owned()))?;
        let primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_primary_colour_column_name".to_owned()))?;
        let secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_secondary_colour_column_name".to_owned()))?;
        let tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_tertiary_colour_column_name".to_owned()))?;
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_row_key".to_owned()))?;

        for (path, packed_file) in data.iter_mut() {
            if path.to_lowercase().starts_with(&format!("db/{}/", table_name)) {

                if let Ok(RFileDecoded::DB(table)) = packed_file.decoded() {

                    // We need multiple column's data for this to work.
                    let key_column = table.column_position_by_name(key_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), key_column_name.to_string()))?;

                    let primary_colour_column = table.column_position_by_name(primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), primary_colour_column_name.to_string()))?;
                    let secondary_colour_column = table.column_position_by_name(secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), secondary_colour_column_name.to_string()))?;
                    let tertiary_colour_column = table.column_position_by_name(tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), tertiary_colour_column_name.to_string()))?;

                    for row in table.data().iter() {
                        let key = match Tool::get_row_by_column_index(row, key_column)? {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                        };

                        if let Some(faction_data) = processed_data.get_mut(key) {
                            let primary_row_by_column = Tool::get_row_by_column_index(row, primary_colour_column)?;
                            let secondary_row_by_column = Tool::get_row_by_column_index(row, secondary_colour_column)?;
                            let tertiary_row_by_column = Tool::get_row_by_column_index(row, tertiary_colour_column)?;

                            let primary_colour = match primary_row_by_column {
                                DecodedData::ColourRGB(_) => primary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let secondary_colour = match secondary_row_by_column {
                                DecodedData::ColourRGB(_) => secondary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };
                            let tertiary_colour = match tertiary_row_by_column {
                                DecodedData::ColourRGB(_) => tertiary_row_by_column.data_to_string(),
                                _ => return Err(ToolsError::ToolTableColumnNotOfTypeWeExpected.into()),
                            };

                            // If we're processing the game files, set the vanilla values.
                            if let DataSource::GameFiles = data_source {
                                faction_data.insert("uniform_vanilla_primary".to_owned(), primary_colour.to_string());
                                faction_data.insert("uniform_vanilla_secondary".to_owned(), secondary_colour.to_string());
                                faction_data.insert("uniform_vanilla_tertiary".to_owned(), tertiary_colour.to_string());
                            }

                            // Set the initial values. The last value inputted is the initial one due to how we load the data.
                            faction_data.insert("uniform_initial_primary".to_owned(), primary_colour.to_string());
                            faction_data.insert("uniform_initial_secondary".to_owned(), secondary_colour.to_string());
                            faction_data.insert("uniform_initial_tertiary".to_owned(), tertiary_colour.to_string());
                            faction_data.insert("uniform_primary".to_owned(), primary_colour.to_string());
                            faction_data.insert("uniform_secondary".to_owned(), secondary_colour.to_string());
                            faction_data.insert("uniform_tertiary".to_owned(), tertiary_colour.to_string());

                            // Also save the full row, so we can easely edit it and put it into a file later on.
                            if faction_data.get(row_key).is_none() {
                                faction_data.insert(row_key.to_owned(), serde_json::to_string(row)?);
                            }

                            // Store the definition, so we can re-use it later to recreate the table.
                            if faction_data.get(table_definition_name).is_none() {
                                let definition = serde_json::to_string(table.definition())?;
                                faction_data.insert(table_definition_name.to_owned(), definition);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// This function takes care of saving the factions's data into a RFile.
    unsafe fn save_factions_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        let table_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_factions_table_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_factions_table_name".to_owned()))?;
        let table_definition_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_factions_table_definition").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_factions_table_definition".to_owned()))?;
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_factions_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_factions_row_key".to_owned()))?;

        let banner_primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_primary_colour_column_name".to_owned()))?;
        let banner_secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_secondary_colour_column_name".to_owned()))?;
        let banner_tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_tertiary_colour_column_name".to_owned()))?;

        let uniform_primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_primary_colour_column_name".to_owned()))?;
        let uniform_secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_secondary_colour_column_name".to_owned()))?;
        let uniform_tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_tertiary_colour_column_name".to_owned()))?;

        if let Some(first) = data.iter().next() {
            if let Some(definition) = first.get(table_definition_name) {
                let mut table = DB::new(&serde_json::from_str(definition)?, None, table_name);

                let banner_primary_colour_column = table.column_position_by_name(banner_primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_primary_colour_column_name.to_string()))?;
                let banner_secondary_colour_column = table.column_position_by_name(banner_secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_secondary_colour_column_name.to_string()))?;
                let banner_tertiary_colour_column = table.column_position_by_name(banner_tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), banner_tertiary_colour_column_name.to_string()))?;

                let uniform_primary_colour_column = table.column_position_by_name(uniform_primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_primary_colour_column_name.to_string()))?;
                let uniform_secondary_colour_column = table.column_position_by_name(uniform_secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_secondary_colour_column_name.to_string()))?;
                let uniform_tertiary_colour_column = table.column_position_by_name(uniform_tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), uniform_tertiary_colour_column_name.to_string()))?;

                let table_data = data.par_iter()
                    .filter_map(|row_data| {
                        let row = row_data.get(row_key)?;
                        let mut row: Vec<DecodedData> = serde_json::from_str(row).ok()?;
                        let mut save_row = false;
                        if row_data.get("banner_primary").is_some() {
                            let banner_primary = row_data.get("banner_primary")?.to_owned();
                            let banner_secondary = row_data.get("banner_secondary")?.to_owned();
                            let banner_tertiary = row_data.get("banner_tertiary")?.to_owned();

                            row[banner_primary_colour_column] = DecodedData::ColourRGB(banner_primary);
                            row[banner_secondary_colour_column] = DecodedData::ColourRGB(banner_secondary);
                            row[banner_tertiary_colour_column] = DecodedData::ColourRGB(banner_tertiary);

                            save_row = true;
                        }
                        if row_data.get("uniform_primary").is_some() {
                            let uniform_primary = row_data.get("uniform_primary")?.to_owned();
                            let uniform_secondary = row_data.get("uniform_secondary")?.to_owned();
                            let uniform_tertiary = row_data.get("uniform_tertiary")?.to_owned();

                            row[uniform_primary_colour_column] = DecodedData::ColourRGB(uniform_primary);
                            row[uniform_secondary_colour_column] = DecodedData::ColourRGB(uniform_secondary);
                            row[uniform_tertiary_colour_column] = DecodedData::ColourRGB(uniform_tertiary);

                            save_row = true;
                        }

                        if save_row {
                            Some(row)
                        } else {
                            None
                        }
                    }).collect::<Vec<Vec<DecodedData>>>();

                table.set_data(&table_data)?;
                let path = format!("db/{}/{}", table_name, self.get_file_name());
                Ok(RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path))
            } else { Err(ToolsError::Impossibru.into()) }
        } else { Err(ToolsError::Impossibru.into()) }
    }

    /// This function takes care of saving the banner's data into a RFile.
    unsafe fn save_faction_banner_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {

        let table_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_table_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_table_name".to_owned()))?;
        let table_definition_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_table_definition").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_table_definition".to_owned()))?;
        let primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_primary_colour_column_name".to_owned()))?;
        let secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_secondary_colour_column_name".to_owned()))?;
        let tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_tertiary_colour_column_name".to_owned()))?;
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_row_key".to_owned()))?;

        let key_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_banner_key_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_banner_key_column_name".to_owned()))?;

        if let Some(first) = data.iter().next() {
            if let Some(definition) = first.get(table_definition_name) {
                let definition = serde_json::from_str(definition)?;
                let mut table = DB::new(&definition, None, table_name);

                let fields_processed = definition.fields_processed();
                let key_column = table.column_position_by_name(key_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), key_column_name.to_string()))?;

                let primary_colour_column = table.column_position_by_name(primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), primary_colour_column_name.to_string()))?;
                let secondary_colour_column = table.column_position_by_name(secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), secondary_colour_column_name.to_string()))?;
                let tertiary_colour_column = table.column_position_by_name(tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), tertiary_colour_column_name.to_string()))?;

                let table_data = data.par_iter()
                    .filter_map(|row_data| {

                        // Check if the original line is found, use it. If not, make a new line.
                        let mut row = if let Some(row) = row_data.get(row_key) {
                            serde_json::from_str(row).ok()?
                        } else {
                            let key = row_data.get("key")?;
                            let mut row = table.new_row();
                            row[key_column] = match fields_processed[key_column].field_type() {
                                FieldType::StringU8 => DecodedData::StringU8(key.to_owned()),
                                FieldType::StringU16 => DecodedData::StringU16(key.to_owned()),
                                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(key.to_owned()),
                                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(key.to_owned()),
                                _ => unimplemented!()
                            };
                            row
                        };

                        if row_data.get("banner_primary").is_some() {
                            let primary = row_data.get("banner_primary")?.to_owned();
                            let secondary = row_data.get("banner_secondary")?.to_owned();
                            let tertiary = row_data.get("banner_tertiary")?.to_owned();

                            row[primary_colour_column] = DecodedData::ColourRGB(primary);
                            row[secondary_colour_column] = DecodedData::ColourRGB(secondary);
                            row[tertiary_colour_column] = DecodedData::ColourRGB(tertiary);

                            Some(row)
                        } else {
                            None
                        }
                    }).collect::<Vec<Vec<DecodedData>>>();

                table.set_data(&table_data)?;
                let path = format!("db/{}/{}", table_name, self.get_file_name());
                Ok(RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path))
            } else { Err(ToolsError::Impossibru.into()) }
        } else { Err(ToolsError::Impossibru.into()) }
    }

    /// This function takes care of saving the banner's data into a RFile.
    unsafe fn save_faction_uniform_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {

        let table_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_table_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_table_name".to_owned()))?;
        let table_definition_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_table_definition").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_table_definition".to_owned()))?;
        let primary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_primary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_primary_colour_column_name".to_owned()))?;
        let secondary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_secondary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_secondary_colour_column_name".to_owned()))?;
        let tertiary_colour_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_tertiary_colour_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_tertiary_colour_column_name".to_owned()))?;
        let row_key = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_row_key").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_row_key".to_owned()))?;

        let key_column_name = GAME_SELECTED.read().unwrap().tool_var("faction_painter_uniform_key_column_name").ok_or_else(|| ToolsError::ToolVarNotFoundForGame("faction_painter_uniform_key_column_name".to_owned()))?;

        if let Some(first) = data.iter().next() {
            if let Some(definition) = first.get(table_definition_name) {
                let definition = serde_json::from_str(definition)?;
                let mut table = DB::new(&definition, None, table_name);

                let fields_processed = definition.fields_processed();
                let key_column = table.column_position_by_name(key_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), key_column_name.to_string()))?;

                let primary_colour_column = table.column_position_by_name(primary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), primary_colour_column_name.to_string()))?;
                let secondary_colour_column = table.column_position_by_name(secondary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), secondary_colour_column_name.to_string()))?;
                let tertiary_colour_column = table.column_position_by_name(tertiary_colour_column_name).ok_or_else(|| ToolsError::MissingColumnInTable(table.table_name().to_string(), tertiary_colour_column_name.to_string()))?;

                let table_data = data.par_iter()
                    .filter_map(|row_data| {

                        // Check if the original line is found, use it. If not, make a new line.
                        let mut row = if let Some(row) = row_data.get(row_key) {
                            serde_json::from_str(row).ok()?
                        } else {
                            let key = row_data.get("key")?;
                            let mut row = table.new_row();
                            row[key_column] = match fields_processed[key_column].field_type() {
                                FieldType::StringU8 => DecodedData::StringU8(key.to_owned()),
                                FieldType::StringU16 => DecodedData::StringU16(key.to_owned()),
                                FieldType::OptionalStringU8 => DecodedData::OptionalStringU8(key.to_owned()),
                                FieldType::OptionalStringU16 => DecodedData::OptionalStringU16(key.to_owned()),
                                _ => unimplemented!()
                            };
                            row
                        };

                        // If this is missing, the checkbox for this field is disabled.s
                        if row_data.get("uniform_primary").is_some() {
                            let primary = row_data.get("uniform_primary")?.to_owned();
                            let secondary = row_data.get("uniform_secondary")?.to_owned();
                            let tertiary = row_data.get("uniform_tertiary")?.to_owned();

                            row[primary_colour_column] = DecodedData::ColourRGB(primary);
                            row[secondary_colour_column] = DecodedData::ColourRGB(secondary);
                            row[tertiary_colour_column] = DecodedData::ColourRGB(tertiary);

                            Some(row)
                        } else {
                            None
                        }
                    }).collect::<Vec<Vec<DecodedData>>>();

                table.set_data(&table_data)?;
                let path = format!("db/{}/{}", table_name, self.get_file_name());
                Ok(RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path))
            } else { Err(ToolsError::Impossibru.into()) }
        } else { Err(ToolsError::Impossibru.into()) }
    }

    /// This function returns the file name this tool uses for the RFiles, when a RFile has no specific name.
    unsafe fn get_file_name(&self) -> String {
        let packed_file_name = self.packed_file_name_line_edit.text();
        if !packed_file_name.is_empty() {
            packed_file_name.to_std_string()
        } else {
            DEFAULT_FILENAME.to_owned()
        }
    }
}
