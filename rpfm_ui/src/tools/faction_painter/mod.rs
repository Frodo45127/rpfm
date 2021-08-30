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
Module with all the code for managing the Faction Painter tool.

This tool is a simple dialog, where you can choose a faction from a list, and change some of its colours.
!*/

use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
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

use unicase::UniCase;

use std::collections::{BTreeMap, HashMap};

use rpfm_lib::packfile::PathType;
use rpfm_lib::packfile::packedfile::PackedFile;
use rpfm_lib::packedfile::DecodedPackedFile;
use rpfm_lib::packedfile::table::DecodedData;

use rpfm_macros::*;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::locale::qtr;
use self::slots::ToolFactionPainterSlots;

use super::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW: &'static str = "rpfm_ui/ui_templates/tool_faction_color_editor.ui";

/// Role that stores the data corresponding to the faction of each item.
const FACTION_DATA: i32 = 60;

/// Role that stores the icon of the faction represented by each item.
const FACTION_ICON: i32 = 61;

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 1] = ["warhammer_2"];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the widgets used by the `Faction Painter` Tool, along with some data needed for the view to work.
#[derive(GetRef, GetRefMut)]
pub struct ToolFactionPainter {
    tool: Tool,
    dialog: QBox<QDialog>,
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
    button_box: QPtr<QDialogButtonBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ToolFactionPainter`.
impl ToolFactionPainter {

    /// This function creates the tool's dialog. NOTE: This can fail at runtime if any of the expected widgets is not in the UI's XML.
    pub unsafe fn new(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

        // Build the tool's dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("faction_painter_title"));
        dialog.set_modal(true);

        // Initialize a Tool. This also performs some common checks to ensure we can actually use the tool.
        let paths = vec![PathType::Folder(vec!["db".to_owned()])];
        let tool = Tool::new(&dialog, &paths, &TOOL_SUPPORTED_GAMES, VIEW)?;
        tool.backup_used_paths(app_ui, pack_file_contents_ui)?;

        // ListView.
        let faction_list_view: QPtr<QListView> = tool.get_ref_main_widget().find_child("faction_list_view").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let faction_list_filter_line_edit: QPtr<QLineEdit> = tool.get_ref_main_widget().find_child("faction_list_filter_line_edit").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;

        // Details view.
        let faction_icon_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("faction_icon_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let faction_name_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("faction_name_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;

        // Banner GroupBox.
        let banner_groupbox: QPtr<QGroupBox> = tool.get_ref_main_widget().find_child("banner_groupbox").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_primary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("banner_colour_primary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_secondary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("banner_colour_secondary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_tertiary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("banner_colour_tertiary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_primary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("banner_colour_primary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_secondary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("banner_colour_secondary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_colour_tertiary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("banner_colour_tertiary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_restore_initial_values_button: QPtr<QPushButton> = tool.get_ref_main_widget().find_child("banner_restore_initial_values_button").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let banner_restore_vanilla_values_button: QPtr<QPushButton> = tool.get_ref_main_widget().find_child("banner_restore_vanilla_values_button").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;

        // Uniform GroupBox.
        let uniform_groupbox: QPtr<QGroupBox> = tool.get_ref_main_widget().find_child("uniform_groupbox").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_primary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("uniform_colour_primary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_secondary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("uniform_colour_secondary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_tertiary_label: QPtr<QLabel> = tool.get_ref_main_widget().find_child("uniform_colour_tertiary_label").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_primary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("uniform_colour_primary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_secondary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("uniform_colour_secondary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_colour_tertiary: QPtr<QComboBox> = tool.get_ref_main_widget().find_child("uniform_colour_tertiary").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_restore_initial_values_button: QPtr<QPushButton> = tool.get_ref_main_widget().find_child("uniform_restore_initial_values_button").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;
        let uniform_restore_vanilla_values_button: QPtr<QPushButton> = tool.get_ref_main_widget().find_child("uniform_restore_vanilla_values_button").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;

        // Button Box.
        let button_box: QPtr<QDialogButtonBox> = tool.get_ref_main_widget().find_child("button_box").map_err(|_| ErrorKind::TemplateUIWidgetNotFound)?;

        // Extra stuff.
        let faction_list_filter = QSortFilterProxyModel::new_1a(&faction_list_view);
        let faction_list_model = QStandardItemModel::new_1a(&faction_list_filter);
        faction_list_view.set_model(&faction_list_filter);
        faction_list_filter.set_source_model(&faction_list_model);

        // Filter timer.
        let timer_delayed_updates = QTimer::new_1a(&dialog);
        timer_delayed_updates.set_single_shot(true);

        // Build the view itself.
        let view = Rc::new(Self{
            tool,
            dialog,
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
            button_box,
        });

        // Build the slots and connect them to the view.
        let slots = ToolFactionPainterSlots::new(&view);
        connections::set_connections(&view, &slots);

        // Setup text translations.
        view.setup_translations();

        // Load all the data to the view.
        view.load_data()?;

        // If we hit ok, save the data back to the PackFile.
        if view.dialog.exec() == 1 {
            view.save_data(app_ui, pack_file_contents_ui)?;
        }

        // If nothing failed, it means we have successfully saved the data back to disk, or canceled.
        Ok(())
    }

    /// This function loads the data we need for the faction painter to the view, inside items in the ListView.
    unsafe fn load_data(&self) -> Result<()> {
        let paths_to_use = vec![
            PathType::Folder(vec!["db".to_owned(), "factions_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "faction_banners_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "faction_uniform_colours_tables".to_owned()]),
        ];

        // Note: this data is HashMap<DataSource, BTreeMap<Path, PackedFile>>.
        CENTRAL_COMMAND.send_message_qt(Command::GetPackedFilesFromAllSources(paths_to_use));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let mut data = if let Response::HashMapDataSourceBTreeMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

        let mut processed_data = BTreeMap::new();

        // First, get the faction's data.
        if let Some(data) = data.get_mut(&DataSource::GameFiles) {
            Self::get_faction_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::ParentFiles) {
            Self::get_faction_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::PackFile) {
            Self::get_faction_data(data, &mut processed_data);
        }

        // Then, get the banner colours.
        if let Some(data) = data.get_mut(&DataSource::GameFiles) {
            Self::get_faction_banner_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::ParentFiles) {
            Self::get_faction_banner_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::PackFile) {
            Self::get_faction_banner_data(data, &mut processed_data);
        }

        // Then, get the uniform colours.
        if let Some(data) = data.get_mut(&DataSource::GameFiles) {
            Self::get_faction_uniform_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::ParentFiles) {
            Self::get_faction_uniform_data(data, &mut processed_data);
        }
        if let Some(data) = data.get_mut(&DataSource::PackFile) {
            Self::get_faction_uniform_data(data, &mut processed_data);
        }

        // Finally, grab the flag files. For that, get the paths from each faction's data, and request the flag icons.
        // These flag paths are already pre-processed to contain their full icon path, and a common slash format.
        let paths_to_use = processed_data.values()
            .map(|x| x.get("flags_path").unwrap()
                .split("/")
                .map(|x| x.to_owned())
                .collect::<Vec<String>>()
            )
            .filter_map(|x| if !x.is_empty() { Some(PathType::File(x.to_vec())) } else { None })
            .collect::<Vec<PathType>>();

        CENTRAL_COMMAND.send_message_qt(Command::GetPackedFilesFromAllSources(paths_to_use));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let images_data = if let Response::HashMapDataSourceBTreeMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

        // Prepare the image paths in unicased format, so we can get them despite what weird casing the paths have.
        let images_paths_unicased = processed_data.iter().map(|(x, y)|
            (x.to_owned(), UniCase::new(y.get("flags_path").unwrap().to_owned()))
        ).collect::<HashMap<String, UniCase<String>>>();

        // Once we got everything processed, build the items for the ListView.
        for (key, data) in &processed_data {

            let item = QStandardItem::from_q_string(&QString::from_std_str(&format!("{} - {}", data.get("screen_name").unwrap(), key))).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(data).unwrap())), FACTION_DATA);

            // Image paths, we may or may not have them, so only try to load them if we actually have a path for them.
            if let Some(image_path_unicased) = images_paths_unicased.get(key) {
                let mut image_data = None;

                if let Some(data) = images_data.get(&DataSource::PackFile) {
                    if let Some(image_packed_file) = data.iter().find_map(|(path, packed_file)| if &UniCase::new(path.join("/")) == image_path_unicased { Some(packed_file) } else { None }) {
                        image_data = Some(image_packed_file.get_raw_data().unwrap());
                    }
                }
                if image_data.is_none() {
                    if let Some(data) = images_data.get(&DataSource::ParentFiles) {
                        if let Some(image_packed_file) = data.iter().find_map(|(path, packed_file)| if &UniCase::new(path.join("/")) == image_path_unicased { Some(packed_file) } else { None }) {
                            image_data = Some(image_packed_file.get_raw_data().unwrap());
                        }
                    }
                }
                if image_data.is_none() {
                    if let Some(data) = images_data.get(&DataSource::GameFiles) {
                        if let Some(image_packed_file) = data.iter().find_map(|(path, packed_file)| if &UniCase::new(path.join("/")) == image_path_unicased { Some(packed_file) } else { None }) {
                            image_data = Some(image_packed_file.get_raw_data().unwrap());
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

        // Store the PackedFiles for use when saving.
        *self.tool.packed_files.borrow_mut() = data;
        Ok(())
    }

    /// This function takes care of saving the data of this Tool into the currently open PackFile, creating a new one if there wasn't one open.
    pub unsafe fn save_data(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

        // First, save whatever is currently open in the detailed view.
        self.faction_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.faction_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        let data_to_save = (0..self.faction_list_model.row_count_0a())
            .map(|row| serde_json::from_str(
                &self.faction_list_model.data_2a(
                    &self.faction_list_model.index_2a(row, 0),
                    FACTION_DATA
                ).to_string()
            .to_std_string()).unwrap())
            .collect::<Vec<BTreeMap<String, String>>>();

        // We have to save the data to the last entry of the keys in out list, so if any of the other fields is edited on it, that edition is kept.
        for data in &data_to_save {
            let mut found_banner = false;
            if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::PackFile) {
                found_banner |= self.save_faction_banner_data(data, packed_files).is_some();
            }

            if !found_banner {
                if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::ParentFiles) {
                    found_banner |= self.save_faction_banner_data(data, packed_files).is_some();
                }
            }

            if !found_banner {
                if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::GameFiles) {
                    self.save_faction_banner_data(data, packed_files);
                }
            }

            let mut found_uniform = false;
            if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::PackFile) {
                found_uniform |= self.save_faction_uniform_data(data, packed_files).is_some();
            }

            if !found_uniform {
                if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::ParentFiles) {
                    found_uniform |= self.save_faction_uniform_data(data, packed_files).is_some();
                }
            }

            if !found_uniform {
                if let Some(packed_files) = self.tool.packed_files.borrow_mut().get_mut(&DataSource::GameFiles) {
                    self.save_faction_uniform_data(data, packed_files);
                }
            }
        }

        // Once we got the PackedFiles to save properly edited, call the generic tool `save` function to save them to a PackFile.
        self.tool.save(app_ui, pack_file_contents_ui)
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {
        let data: BTreeMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();
        let screen_name = data.get("screen_name").unwrap();
        self.get_ref_faction_name_label().set_text(&QString::from_std_str(screen_name));

        let image = QPixmap::new();
        image.load_from_data_q_byte_array(&index.data_1a(61).to_byte_array());
        self.get_ref_faction_icon_label().set_pixmap(&image);

        let banner_primary = data.get("banner_primary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
        let banner_secondary = data.get("banner_secondary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
        let banner_tertiary = data.get("banner_tertiary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();

        let uniform_primary = data.get("uniform_primary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
        let uniform_secondary = data.get("uniform_secondary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
        let uniform_tertiary = data.get("uniform_tertiary").unwrap().split(',').map(|x| x.parse().unwrap()).collect::<Vec<i32>>();

        set_color_safe(&self.get_ref_banner_colour_primary().as_ptr().static_upcast(), &QColor::from_rgb_3a(banner_primary[0], banner_primary[1], banner_primary[2]).as_ptr());
        set_color_safe(&self.get_ref_banner_colour_secondary().as_ptr().static_upcast(), &QColor::from_rgb_3a(banner_secondary[0], banner_secondary[1], banner_secondary[2]).as_ptr());
        set_color_safe(&self.get_ref_banner_colour_tertiary().as_ptr().static_upcast(), &QColor::from_rgb_3a(banner_tertiary[0], banner_tertiary[1], banner_tertiary[2]).as_ptr());

        set_color_safe(&self.get_ref_uniform_colour_primary().as_ptr().static_upcast(), &QColor::from_rgb_3a(uniform_primary[0], uniform_primary[1], uniform_primary[2]).as_ptr());
        set_color_safe(&self.get_ref_uniform_colour_secondary().as_ptr().static_upcast(), &QColor::from_rgb_3a(uniform_secondary[0], uniform_secondary[1], uniform_secondary[2]).as_ptr());
        set_color_safe(&self.get_ref_uniform_colour_tertiary().as_ptr().static_upcast(), &QColor::from_rgb_3a(uniform_tertiary[0], uniform_tertiary[1], uniform_tertiary[2]).as_ptr());
    }

    /// This function saves the data of the detailed view to its item in the faction list.
    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: BTreeMap<String, String> = serde_json::from_str(&index.data_1a(FACTION_DATA).to_string().to_std_string()).unwrap();

        let banner_primary = get_color_safe(&self.get_ref_banner_colour_primary().as_ptr().static_upcast());
        let banner_secondary = get_color_safe(&self.get_ref_banner_colour_secondary().as_ptr().static_upcast());
        let banner_tertiary = get_color_safe(&self.get_ref_banner_colour_tertiary().as_ptr().static_upcast());

        let uniform_primary = get_color_safe(&self.get_ref_uniform_colour_primary().as_ptr().static_upcast());
        let uniform_secondary = get_color_safe(&self.get_ref_uniform_colour_secondary().as_ptr().static_upcast());
        let uniform_tertiary = get_color_safe(&self.get_ref_uniform_colour_tertiary().as_ptr().static_upcast());

        *data.get_mut("banner_primary").unwrap() = format!("{},{},{}", banner_primary.red(), banner_primary.green(), banner_primary.blue());
        *data.get_mut("banner_secondary").unwrap() = format!("{},{},{}", banner_secondary.red(), banner_secondary.green(), banner_secondary.blue());
        *data.get_mut("banner_tertiary").unwrap() = format!("{},{},{}", banner_tertiary.red(), banner_tertiary.green(), banner_tertiary.blue());

        *data.get_mut("uniform_primary").unwrap() = format!("{},{},{}", uniform_primary.red(), uniform_primary.green(), uniform_primary.blue());
        *data.get_mut("uniform_secondary").unwrap() = format!("{},{},{}", uniform_secondary.red(), uniform_secondary.green(), uniform_secondary.blue());
        *data.get_mut("uniform_tertiary").unwrap() = format!("{},{},{}", uniform_tertiary.red(), uniform_tertiary.green(), uniform_tertiary.blue());

        self.faction_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&data).unwrap())), FACTION_DATA);
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
    }

    unsafe fn get_faction_data(data: &mut BTreeMap<Vec<String>, PackedFile>, processed_data: &mut BTreeMap<String, BTreeMap<String, String>>) {

        // First, get the keys, names and flags from the factions tables.
        for (path, packed_file) in data.iter_mut() {
            if path[1] == "factions_tables" {

                let decoded = packed_file.decode_return_ref().unwrap();
                if let DecodedPackedFile::DB(table) = decoded {

                    // We need multiple column's data for this to work.
                    let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "key").unwrap();
                    let name_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "screen_name").unwrap();
                    let flag_path_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "flags_path").unwrap();

                    for row in table.get_ref_table_data() {
                        let mut data = BTreeMap::new();

                        match row[name_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => {
                                data.insert("screen_name".to_owned(), value.to_owned());
                            }
                            _ => unimplemented!(),
                        }

                        match row[flag_path_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => {
                                data.insert("flags_path".to_owned(), value.to_owned().replace("\\", "/") + "/mon_64.png");
                            }
                            _ => unimplemented!(),
                        }

                        match row[key_column] {
                            DecodedData::StringU8(ref key) |
                            DecodedData::StringU16(ref key) |
                            DecodedData::OptionalStringU8(ref key) |
                            DecodedData::OptionalStringU16(ref key) => {
                                data.insert("key".to_owned(), key.to_owned());
                                processed_data.insert(key.to_owned(), data);
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }
        }
    }

    unsafe fn get_faction_banner_data(data: &mut BTreeMap<Vec<String>, PackedFile>, processed_data: &mut BTreeMap<String, BTreeMap<String, String>>) {

        for (path, packed_file) in data.iter_mut() {
            if path[1] == "faction_banners_tables" {

                let decoded = packed_file.decode_return_ref().unwrap();
                if let DecodedPackedFile::DB(table) = decoded {

                    // We need multiple column's data for this to work.
                    let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "key").unwrap();

                    let primary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_red").unwrap();
                    let primary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_green").unwrap();
                    let primary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_blue").unwrap();

                    let secondary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_red").unwrap();
                    let secondary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_green").unwrap();
                    let secondary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_blue").unwrap();

                    let tertiary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_red").unwrap();
                    let tertiary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_green").unwrap();
                    let tertiary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_blue").unwrap();
                    dbg!(table.get_ref_table_data());

                    for row in table.get_ref_table_data() {
                        let key = match row[key_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => unimplemented!(),
                        };

                        if let Some(faction_data) = processed_data.get_mut(key) {
                            let primary_r = match row[primary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let primary_g = match row[primary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let primary_b = match row[primary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let secondary_r = match row[secondary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let secondary_g = match row[secondary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let secondary_b = match row[secondary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let tertiary_r = match row[tertiary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let tertiary_g = match row[tertiary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let tertiary_b = match row[tertiary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let primary = format!("{},{},{}", primary_r, primary_g, primary_b);
                            let secondary = format!("{},{},{}", secondary_r, secondary_g, secondary_b);
                            let tertiary = format!("{},{},{}", tertiary_r, tertiary_g, tertiary_b);

                            faction_data.insert("banner_primary".to_owned(), primary);
                            faction_data.insert("banner_secondary".to_owned(), secondary);
                            faction_data.insert("banner_tertiary".to_owned(), tertiary);
                        }
                    }
                }
            }
        }
    }

    unsafe fn get_faction_uniform_data(data: &mut BTreeMap<Vec<String>, PackedFile>, processed_data: &mut BTreeMap<String, BTreeMap<String, String>>) {

        for (path, packed_file) in data.iter_mut() {
            if path[1] == "faction_uniform_colours_tables" {

                let decoded = packed_file.decode_return_ref().unwrap();
                if let DecodedPackedFile::DB(table) = decoded {

                    // We need multiple column's data for this to work.
                    let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "faction_name").unwrap();

                    let primary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_r").unwrap();
                    let primary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_g").unwrap();
                    let primary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_b").unwrap();

                    let secondary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_r").unwrap();
                    let secondary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_g").unwrap();
                    let secondary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_b").unwrap();

                    let tertiary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_r").unwrap();
                    let tertiary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_g").unwrap();
                    let tertiary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_b").unwrap();

                    for row in table.get_ref_table_data() {
                        let key = match row[key_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => unimplemented!(),
                        };

                        if let Some(faction_data) = processed_data.get_mut(key) {
                            let primary_r = match row[primary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let primary_g = match row[primary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let primary_b = match row[primary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let secondary_r = match row[secondary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let secondary_g = match row[secondary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let secondary_b = match row[secondary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let tertiary_r = match row[tertiary_colour_r_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let tertiary_g = match row[tertiary_colour_g_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };
                            let tertiary_b = match row[tertiary_colour_b_column] {
                                DecodedData::I32(ref value) => value,
                                _ => unimplemented!(),
                            };

                            let primary = format!("{},{},{}", primary_r, primary_g, primary_b);
                            let secondary = format!("{},{},{}", secondary_r, secondary_g, secondary_b);
                            let tertiary = format!("{},{},{}", tertiary_r, tertiary_g, tertiary_b);

                            faction_data.insert("uniform_primary".to_owned(), primary);
                            faction_data.insert("uniform_secondary".to_owned(), secondary);
                            faction_data.insert("uniform_tertiary".to_owned(), tertiary);
                        }
                    }
                }
            }
        }
    }

    unsafe fn save_faction_banner_data(&self, data: &BTreeMap<String, String>, packed_files: &mut BTreeMap<Vec<String>, PackedFile>) -> Option<()> {
        for (path, packed_file) in packed_files.iter_mut().rev() {
            if path[1] == "faction_banners_tables" {
                let decoded = packed_file.decode_return_ref_mut().unwrap();
                if let DecodedPackedFile::DB(table) = decoded {

                    // We need multiple column's data for this to work.
                    let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "key").unwrap();

                    let primary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_red").unwrap();
                    let primary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_green").unwrap();
                    let primary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_blue").unwrap();

                    let secondary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_red").unwrap();
                    let secondary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_green").unwrap();
                    let secondary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_blue").unwrap();

                    let tertiary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_red").unwrap();
                    let tertiary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_green").unwrap();
                    let tertiary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_blue").unwrap();

                    let mut table_data = table.get_table_data();
                    for row in &mut table_data {
                        let key = match row[key_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => unimplemented!(),
                        };

                        if let Some(faction_key) = data.get("key") {
                            if faction_key == key {
                                let primary = data.get("banner_primary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
                                let secondary = data.get("banner_secondary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
                                let tertiary = data.get("banner_tertiary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();

                                row[primary_colour_r_column] = DecodedData::I32(primary[0]);
                                row[primary_colour_g_column] = DecodedData::I32(primary[1]);
                                row[primary_colour_b_column] = DecodedData::I32(primary[2]);

                                row[secondary_colour_r_column] = DecodedData::I32(secondary[0]);
                                row[secondary_colour_g_column] = DecodedData::I32(secondary[1]);
                                row[secondary_colour_b_column] = DecodedData::I32(secondary[2]);

                                row[tertiary_colour_r_column] = DecodedData::I32(tertiary[0]);
                                row[tertiary_colour_g_column] = DecodedData::I32(tertiary[1]);
                                row[tertiary_colour_b_column] = DecodedData::I32(tertiary[2]);

                                table.set_table_data(&table_data);
                                return Some(());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    unsafe fn save_faction_uniform_data(&self, data: &BTreeMap<String, String>, packed_files: &mut BTreeMap<Vec<String>, PackedFile>) -> Option<()> {
        for (path, packed_file) in packed_files.iter_mut().rev() {
            if path[1] == "faction_uniform_colours_tables" {
                let decoded = packed_file.decode_return_ref_mut().unwrap();
                if let DecodedPackedFile::DB(table) = decoded {

                    // We need multiple column's data for this to work.
                    let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "faction_name").unwrap();

                    let primary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_r").unwrap();
                    let primary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_g").unwrap();
                    let primary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "primary_colour_b").unwrap();

                    let secondary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_r").unwrap();
                    let secondary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_g").unwrap();
                    let secondary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "secondary_colour_b").unwrap();

                    let tertiary_colour_r_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_r").unwrap();
                    let tertiary_colour_g_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_g").unwrap();
                    let tertiary_colour_b_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "tertiary_colour_b").unwrap();

                    let mut table_data = table.get_table_data();
                    for row in &mut table_data {
                        let key = match row[key_column] {
                            DecodedData::StringU8(ref value) |
                            DecodedData::StringU16(ref value) |
                            DecodedData::OptionalStringU8(ref value) |
                            DecodedData::OptionalStringU16(ref value) => value,
                            _ => unimplemented!(),
                        };

                        if let Some(faction_key) = data.get("key") {
                            if faction_key == key {
                                let primary = data.get("uniform_primary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
                                let secondary = data.get("uniform_secondary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();
                                let tertiary = data.get("uniform_tertiary").unwrap().split(",").map(|x| x.parse().unwrap()).collect::<Vec<i32>>();

                                row[primary_colour_r_column] = DecodedData::I32(primary[0]);
                                row[primary_colour_g_column] = DecodedData::I32(primary[1]);
                                row[primary_colour_b_column] = DecodedData::I32(primary[2]);

                                row[secondary_colour_r_column] = DecodedData::I32(secondary[0]);
                                row[secondary_colour_g_column] = DecodedData::I32(secondary[1]);
                                row[secondary_colour_b_column] = DecodedData::I32(secondary[2]);

                                row[tertiary_colour_r_column] = DecodedData::I32(tertiary[0]);
                                row[tertiary_colour_g_column] = DecodedData::I32(tertiary[1]);
                                row[tertiary_colour_b_column] = DecodedData::I32(tertiary[2]);

                                table.set_table_data(&table_data);
                                return Some(());
                            }
                        }
                    }
                }
            }
        }

        None
    }
}
