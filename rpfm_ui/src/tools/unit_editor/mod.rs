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
Module with all the code for managing the Unit Editor tool.

This tool is a dialog where you can pick a unit from a list, and edit its values in an easy-to-use way.
!*/

use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QTabWidget;
use qt_widgets::QTextEdit;

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
use qt_widgets::QToolButton;

use std::collections::HashMap;
use std::sync::RwLock;

use rpfm_lib::packfile::PathType;
use rpfm_lib::packfile::packedfile::PackedFile;

use rpfm_macros::*;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::{qtr, tr};
use crate::views::table::utils::{clean_column_names, get_reference_data};
use self::slots::ToolUnitEditorSlots;
use super::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_unit_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_unit_editor.ui";

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 1] = [
    "warhammer_2",
];

/// Default name for files saved with this tool.
const DEFAULT_FILENAME: &str = "unit_edited";

/// Role that stores the data of the unit represented by each item.
const UNIT_DATA: i32 = 60;

/// Path where all the unit info pictures (icons are too small) are located.
const UNIT_INFOPICS_PATH: &str = "ui/units/infopics/";

/// Path where all the unit icon pictures (for backup) are located.
const UNIT_ICONS_PATH: &str = "ui/units/icons/";

/// List of fields tht require special treatment from land_units_tables.
const LAND_UNITS_CUSTOM_FIELDS: [&str; 3] = [
    "short_description_text",
    "historical_description_text",
    "strengths_&_weaknesses_text"
];

/// List of fields tht require special treatment from main_units_tables.
const MAIN_UNITS_CUSTOM_FIELDS: [&str; 3] = [
    "land_unit",
    "naval_unit",
    "caste"
];

/// List of loc keys used by this tool.
///
/// The values are:
/// - table_name_column_name.
/// - table_name_column_name from the column where we can get the "key" of this loc.
const LOC_KEYS: [(&str, &str); 4] = [
    ("land_units_onscreen_name", "land_units_key"),
    ("unit_description_short_texts_text", "unit_description_short_texts_key"),
    ("unit_description_historical_texts_text", "unit_description_historical_texts_key"),
    ("unit_description_strengths_weaknesses_texts_text", "unit_description_strengths_weaknesses_texts_key")
];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the widgets used by the `Unit Editor` Tool, along with some data needed for the view to work.
#[derive(GetRef, GetRefMut)]
pub struct ToolUnitEditor {
    tool: Tool,
    timer_delayed_updates: QBox<QTimer>,

    unit_list_view: QPtr<QListView>,
    unit_list_filter: QBox<QSortFilterProxyModel>,
    unit_list_model: QBox<QStandardItemModel>,
    unit_list_filter_line_edit: QPtr<QLineEdit>,

    detailed_view_tab_widget: QPtr<QTabWidget>,
    unit_icon_label: QPtr<QLabel>,
    unit_icon_key_label: QPtr<QLabel>,

    packed_file_name_label: QPtr<QLabel>,
    packed_file_name_line_edit: QPtr<QLineEdit>,

    unit_caste_previous: Rc<RwLock<String>>,
    unit_type_dependant_widgets: HashMap<String, Vec<QPtr<QWidget>>>,

    unit_icon_key_combobox: QPtr<QComboBox>,
    unit_icon_key_tool_button: QPtr<QToolButton>,

    //-----------------------------------------------------------------------//
    // Main tab groupboxes.
    //-----------------------------------------------------------------------//
    unit_editor_key_loc_data_groupbox: QPtr<QGroupBox>,
    unit_editor_requirements_groupbox: QPtr<QGroupBox>,
    unit_editor_campaign_groupbox: QPtr<QGroupBox>,
    unit_editor_ui_groupbox: QPtr<QGroupBox>,
    unit_editor_audio_groupbox: QPtr<QGroupBox>,

    //-----------------------------------------------------------------------//
    // main_units_tables
    //-----------------------------------------------------------------------//
    main_units_caste_label: QPtr<QLabel>,
    main_units_caste_combobox: QPtr<QComboBox>,
    main_units_unit_label: QPtr<QLabel>,
    main_units_unit_line_edit: QPtr<QLineEdit>,

    //-----------------------------------------------------------------------//
    // Loc fields.
    //-----------------------------------------------------------------------//
    loc_land_units_onscreen_name_label: QPtr<QLabel>,
    loc_land_units_onscreen_name_line_edit: QPtr<QLineEdit>,
    loc_unit_description_historical_text_key_label: QPtr<QLabel>,
    loc_unit_description_historical_text_key_ktexteditor: QPtr<QTextEdit>,
    loc_unit_description_short_texts_text_label: QPtr<QLabel>,
    loc_unit_description_short_texts_text_ktexteditor: QPtr<QTextEdit>,
    loc_unit_description_strengths_weaknesses_texts_text_label: QPtr<QLabel>,
    loc_unit_description_strengths_weaknesses_texts_text_ktexteditor: QPtr<QTextEdit>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ToolUnitEditor`.
impl ToolUnitEditor {

    /// This function creates the tool's dialog.
    ///
    /// NOTE: This can fail at runtime if any of the expected widgets is not in the UI's XML.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>
    ) -> Result<()> {

        // Initialize a Tool. This also performs some common checks to ensure we can actually use the tool.
        let paths = vec![
            PathType::Folder(vec!["db".to_owned(), "battle_set_piece_armies_characters_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "land_units_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "main_units_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "unit_description_historical_texts_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "unit_description_short_texts_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "unit_description_strengths_weaknesses_texts_tables".to_owned()]),
            PathType::Folder(vec!["db".to_owned(), "unit_variants_tables".to_owned()]),
            PathType::Folder(vec!["text".to_owned()]),
        ];

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let tool = Tool::new(&app_ui.main_window, &paths, &TOOL_SUPPORTED_GAMES, view)?;
        tool.set_title(&tr("unit_editor_title"));
        tool.backup_used_paths(app_ui, pack_file_contents_ui)?;

        //-----------------------------------------------------------------------//
        // Tool-specific stuff
        //-----------------------------------------------------------------------//

        // ListView.
        let unit_list_view: QPtr<QListView> = tool.find_widget("unit_list_view")?;
        let unit_list_filter = QSortFilterProxyModel::new_1a(&unit_list_view);
        let unit_list_model = QStandardItemModel::new_1a(&unit_list_filter);
        let unit_list_filter_line_edit: QPtr<QLineEdit> = tool.find_widget("unit_list_filter_line_edit")?;
        unit_list_view.set_model(&unit_list_filter);
        unit_list_filter.set_source_model(&unit_list_model);

        // Filter timer.
        let timer_delayed_updates = QTimer::new_1a(tool.get_ref_main_widget());
        timer_delayed_updates.set_single_shot(true);

        // Icon stuff.
        let unit_icon_label: QPtr<QLabel> = tool.find_widget("unit_icon_label")?;
        let unit_icon_key_label: QPtr<QLabel> = tool.find_widget("unit_icon_key_label")?;

        // File name and button box.
        let packed_file_name_label: QPtr<QLabel> = tool.find_widget("packed_file_name_label")?;
        let packed_file_name_line_edit: QPtr<QLineEdit> = tool.find_widget("packed_file_name_line_edit")?;
        packed_file_name_line_edit.set_text(&QString::from_std_str(DEFAULT_FILENAME));

        // Extra stuff.
        let detailed_view_tab_widget: QPtr<QTabWidget> = tool.find_widget("detailed_view_tab_widget")?;
        detailed_view_tab_widget.set_enabled(false);

        //-----------------------------------------------------------------------//
        // Main tab groupboxes.
        //-----------------------------------------------------------------------//
        let unit_editor_key_loc_data_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_key_loc_data_groupbox")?;
        let unit_editor_requirements_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_requirements_groupbox")?;
        let unit_editor_campaign_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_campaign_groupbox")?;
        let unit_editor_ui_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_ui_groupbox")?;
        let unit_editor_audio_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_audio_groupbox")?;

        let unit_icon_key_combobox: QPtr<QComboBox> = tool.find_widget("unit_icon_key_combobox")?;
        let unit_icon_key_tool_button: QPtr<QToolButton> = tool.find_widget("unit_icon_key_tool_button")?;

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//

        let main_units_caste_label: QPtr<QLabel> = tool.find_widget("main_units_caste_label")?;
        let main_units_caste_combobox: QPtr<QComboBox> = tool.find_widget("main_units_caste_combobox")?;
        let main_units_unit_label: QPtr<QLabel> = tool.find_widget("main_units_unit_label")?;
        let main_units_unit_line_edit: QPtr<QLineEdit> = tool.find_widget("main_units_unit_line_edit")?;

        //-----------------------------------------------------------------------//
        // Loc fields.
        //-----------------------------------------------------------------------//

        let loc_land_units_onscreen_name_label: QPtr<QLabel> = tool.find_widget("loc_land_units_onscreen_name_label")?;
        let loc_land_units_onscreen_name_line_edit: QPtr<QLineEdit> = tool.find_widget("loc_land_units_onscreen_name_line_edit")?;

        let loc_unit_description_historical_text_key_label: QPtr<QLabel> = tool.find_widget("loc_unit_description_historical_text_key_label")?;
        let loc_unit_description_historical_text_key_ktexteditor: QPtr<QTextEdit> = tool.find_widget("loc_unit_description_historical_text_key_ktexteditor")?;

        let loc_unit_description_short_texts_text_label: QPtr<QLabel> = tool.find_widget("loc_unit_description_short_texts_text_label")?;
        let loc_unit_description_short_texts_text_ktexteditor: QPtr<QTextEdit> = tool.find_widget("loc_unit_description_short_texts_text_ktexteditor")?;

        let loc_unit_description_strengths_weaknesses_texts_text_label: QPtr<QLabel> = tool.find_widget("loc_unit_description_strengths_weaknesses_texts_text_label")?;
        let loc_unit_description_strengths_weaknesses_texts_text_ktexteditor: QPtr<QTextEdit> = tool.find_widget("loc_unit_description_strengths_weaknesses_texts_text_ktexteditor")?;

        //-----------------------------------------------------------------------//
        // Table-related widgets done.
        //-----------------------------------------------------------------------//

        let unit_caste_previous = Rc::new(RwLock::new("".to_owned()));
        let unit_type_dependant_widgets = HashMap::new();

        // Build the view itself.
        let mut view = Self {
            tool,
            timer_delayed_updates,

            unit_list_view,
            unit_list_filter,
            unit_list_model,
            unit_list_filter_line_edit,

            detailed_view_tab_widget,
            unit_icon_label,
            unit_icon_key_label,

            packed_file_name_label,
            packed_file_name_line_edit,

            unit_caste_previous,
            unit_type_dependant_widgets,

            //-----------------------------------------------------------------------//
            // Main tab groupboxes.
            //-----------------------------------------------------------------------//
            unit_editor_key_loc_data_groupbox,
            unit_editor_requirements_groupbox,
            unit_editor_campaign_groupbox,
            unit_editor_ui_groupbox,
            unit_editor_audio_groupbox,

            unit_icon_key_combobox,
            unit_icon_key_tool_button,

            //-----------------------------------------------------------------------//
            // main_units_tables
            //-----------------------------------------------------------------------//
            main_units_caste_label,
            main_units_caste_combobox,
            main_units_unit_label,
            main_units_unit_line_edit,

            //-----------------------------------------------------------------------//
            // Loc fields.
            //-----------------------------------------------------------------------//
            loc_land_units_onscreen_name_label,
            loc_land_units_onscreen_name_line_edit,
            loc_unit_description_historical_text_key_label,
            loc_unit_description_historical_text_key_ktexteditor,
            loc_unit_description_short_texts_text_label,
            loc_unit_description_short_texts_text_ktexteditor,
            loc_unit_description_strengths_weaknesses_texts_text_label,
            loc_unit_description_strengths_weaknesses_texts_text_ktexteditor,
        };

        // Setup dependant widget relations.
        view.setup_widgets_relations();
        let view = Rc::new(view);

        // Build the slots and connect them to the view.
        let slots = ToolUnitEditorSlots::new(&view);
        connections::set_connections(&view, &slots);

        // Setup text translations.
        view.setup_translations();

        // Load all the data to the view.
        view.load_data()?;

        // If we hit ok, save the data back to the PackFile.
        if view.tool.get_ref_dialog().exec() == 1 {
            view.save_data(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui)?;
        }

        // If nothing failed, it means we have successfully saved the data back to disk, or canceled.
        Ok(())
    }

    /// This function loads the data we need for the faction painter to the view, inside items in the ListView.
    unsafe fn load_data(&self) -> Result<()> {

        // Note: this data is HashMap<DataSource, HashMap<Path, PackedFile>>.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesFromAllSources(self.tool.used_paths.to_vec()));
        let response = CentralCommand::recv(&receiver);
        let mut data = if let Response::HashMapDataSourceHashMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

        let mut processed_data = HashMap::new();

        // Get the table's data.
        get_data_from_all_sources!(self, get_battle_set_piece_armies_characters_data, data, processed_data);
        get_data_from_all_sources!(self, get_main_units_data, data, processed_data);
        get_data_from_all_sources!(self, get_land_units_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_historical_text_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_short_texts_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_strengths_weaknesses_texts_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_variants_data, data, processed_data);
        get_data_from_all_sources!(self, get_loc_data, data, processed_data);

        // Once we got everything processed, build the items for the ListView.
        for (key, data) in processed_data.iter().sorted_by_key(|x| x.0) {
            let item = QStandardItem::from_q_string(&QString::from_std_str(&key)).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(data).unwrap())), UNIT_DATA);
            self.unit_list_model.append_row_q_standard_item(item);
        }

        // Then, build the combos lists with dependencies data.
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

        for icon_key in &icon_keys_sorted {
            let name_without_extension = icon_key.split('.').collect::<Vec<&str>>()[0];
            self.unit_icon_key_combobox.add_item_q_string(&QString::from_std_str(name_without_extension));
        }

        // Store the PackedFiles for use when saving.
        *self.tool.packed_files.borrow_mut() = data;
        Ok(())
    }

    /// This function takes care of saving the data of this Tool into the currently open PackFile, creating a new one if there wasn't one open.
    pub unsafe fn save_data(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>
    ) -> Result<()> {

        // First, save whatever is currently open in the detailed view.
        self.unit_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.unit_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        // Get each faction's data as a HashMap of data/value.
        let data_to_save = (0..self.unit_list_model.row_count_0a())
            .map(|row| serde_json::from_str(
                &self.unit_list_model.data_2a(
                    &self.unit_list_model.index_2a(row, 0),
                    UNIT_DATA
                ).to_string()
            .to_std_string()).unwrap())
            .collect::<Vec<HashMap<String, String>>>();

        // We have to save the data to the last entry of the keys in out list, so if any of the other fields is edited on it, that edition is kept.
        let land_units_packed_file = self.save_land_units_tables_data(&data_to_save)?;
        let main_units_packed_file = self.save_main_units_tables_data(&data_to_save)?;
        let unit_description_historical_texts_packed_file = self.save_unit_description_historical_texts_tables_data(&data_to_save)?;
        let unit_description_short_texts_packed_file = self.save_unit_description_short_texts_tables_data(&data_to_save)?;
        let unit_description_strengths_weaknesses_texts_packed_file = self.save_unit_description_strengths_weaknesses_texts_tables_data(&data_to_save)?;
        let unit_variants_packed_file = self.save_unit_variants_tables_data(&data_to_save)?;

        let loc_packed_file = self.save_loc_data(&data_to_save)?;

        // Once we got the PackedFiles to save properly edited, call the generic tool `save` function to save them to a PackFile.
        self.tool.save(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, &[
            land_units_packed_file,
            main_units_packed_file,
            unit_description_historical_texts_packed_file,
            unit_description_short_texts_packed_file,
            unit_description_strengths_weaknesses_texts_packed_file,
            unit_variants_packed_file,

            loc_packed_file
        ])
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first faction loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.detailed_view_tab_widget.is_enabled() {
            self.detailed_view_tab_widget.set_enabled(true);
        }

        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        //-----------------------------------------------------------------------//
        // land_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "land_units", &LAND_UNITS_CUSTOM_FIELDS) {
            show_message_warning(&self.tool.message_widget, error);
        }

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//
        self.tool.load_field_to_detailed_view_editor_string_short(&data, &self.main_units_unit_line_edit, "main_units_unit");
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.main_units_caste_combobox, "main_units_caste");

        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "main_units", &MAIN_UNITS_CUSTOM_FIELDS) {
            show_message_warning(&self.tool.message_widget, error);
        }

        //-----------------------------------------------------------------------//
        // unit_variants_tables
        //-----------------------------------------------------------------------//
        self.tool.load_field_to_detailed_view_editor_string_combo(&data, &self.unit_icon_key_combobox, "unit_variants_unit_card");

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        self.tool.load_field_to_detailed_view_editor_string_short(&data, &self.loc_land_units_onscreen_name_line_edit, "loc_land_units_onscreen_name");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_historical_text_key_ktexteditor, "loc_unit_description_historical_texts_text");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_short_texts_text_ktexteditor, "loc_unit_description_short_texts_text");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_strengths_weaknesses_texts_text_ktexteditor, "loc_unit_description_strengths_weaknesses_texts_text");

        // The icon needs to be pulled up from the dependencies cache on load.
        self.load_unit_icon(&data, None);
    }

    /// This function loads the unit icon into the tool. If provided with a key, it uses it. If not, it uses whatever key the unit has.
    pub unsafe fn load_unit_icon(&self, data: &HashMap<String, String>, key: Option<String>) {
        let unit_card = if let Some(unit_card) = key { Some(unit_card.to_owned()) }
        else if let Some(unit_card) = data.get("unit_variants_unit_card") { Some(unit_card.to_owned()) }
        else { None };

        // The icon needs to be pulled up from the dependencies cache on load.
        if let Some(unit_card) = unit_card {
            let icon_path_png = format!("{}{}.png", UNIT_INFOPICS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let icon_path_tga = format!("{}{}.tga", UNIT_INFOPICS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let icon_path_png_lowres = format!("{}{}.png", UNIT_ICONS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();
            let icon_path_tga_lowres = format!("{}{}.tga", UNIT_ICONS_PATH.to_owned(), unit_card).split('/').map(|x| x.to_owned()).collect::<Vec<String>>();

            let icon_paths = vec![
                PathType::File(icon_path_png.to_vec()),
                PathType::File(icon_path_tga.to_vec()),
                PathType::File(icon_path_png_lowres.to_vec()),
                PathType::File(icon_path_tga_lowres.to_vec()),
            ];

            let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFilesFromAllSources(icon_paths));
            let response = CentralCommand::recv(&receiver);
            let images_data = if let Response::HashMapDataSourceHashMapVecStringPackedFile(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };
            let image_file = if let Some(image_file) = Tool::get_most_relevant_file(&images_data, &icon_path_png) {
                Some(image_file)
            } else if let Some(image_file) = Tool::get_most_relevant_file(&images_data, &icon_path_tga) {
                Some(image_file)
            } else if let Some(image_file) = Tool::get_most_relevant_file(&images_data, &icon_path_png_lowres) {
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
                self.unit_icon_label.set_pixmap(&image);
            } else {
                self.unit_icon_label.set_text(&QString::from_std_str("No image available"));
            }
        } else {
            self.unit_icon_label.set_text(&QString::from_std_str("No image available"));
        }
    }

    /// This function saves the data of the detailed view to its item in the faction list.
    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();

        //-----------------------------------------------------------------------//
        // land_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "land_units", &LAND_UNITS_CUSTOM_FIELDS) {
            show_message_warning(&self.tool.message_widget, error);
        }

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//
        data.insert("main_units_unit".to_owned(), self.main_units_unit_line_edit.text().to_std_string());
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "main_units", &MAIN_UNITS_CUSTOM_FIELDS) {
            show_message_warning(&self.tool.message_widget, error);
        }

        //-----------------------------------------------------------------------//
        // unit_variants_tables
        //-----------------------------------------------------------------------//
        //data.insert("unit_variants_unit_card".to_owned(), self.unit_icon_key_line_edit.text().to_std_string());

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        data.insert("loc_land_units_onscreen_name".to_owned(), self.loc_land_units_onscreen_name_line_edit.text().to_std_string());
        data.insert("loc_unit_description_historical_texts_text".to_owned(), self.loc_unit_description_historical_text_key_ktexteditor.to_plain_text().to_std_string());
        data.insert("loc_unit_description_short_texts_text".to_owned(), self.loc_unit_description_short_texts_text_ktexteditor.to_plain_text().to_std_string());
        data.insert("loc_unit_description_strengths_weaknesses_texts_text".to_owned(), self.loc_unit_description_strengths_weaknesses_texts_text_ktexteditor.to_plain_text().to_std_string());

        // Update all the referenced keys in our data.
        self.update_keys(&mut data);
        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }
        self.unit_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&data).unwrap())), UNIT_DATA);
    }

    /// Function to trigger certain delayed actions, like the filter.
    pub unsafe fn start_delayed_updates_timer(&self) {
        self.timer_delayed_updates.set_interval(500);
        self.timer_delayed_updates.start_0a();
    }

    /// Function to filter the faction list.
    pub unsafe fn filter_list(&self) {
        self.unit_list_filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        self.unit_list_filter.set_filter_regular_expression_q_string(&self.unit_list_filter_line_edit.text());
    }

    /// Function to setup all the translations of this view.
    pub unsafe fn setup_translations(&self) {
        self.packed_file_name_label.set_text(&qtr("packed_file_name"));

        self.detailed_view_tab_widget.set_tab_text(0, &qtr("tools_unit_editor_main_tab_title"));
        self.detailed_view_tab_widget.set_tab_text(1, &qtr("tools_unit_editor_land_unit_tab_title"));
        self.detailed_view_tab_widget.set_tab_text(2, &qtr("tools_unit_editor_variantmeshes_tab_title"));

        //-----------------------------------------------------------------------//
        // Main tab groupboxes.
        //-----------------------------------------------------------------------//
        self.unit_editor_key_loc_data_groupbox.set_title(&qtr("tools_unit_editor_key_loc_data"));
        self.unit_editor_requirements_groupbox.set_title(&qtr("tools_unit_editor_requirements"));
        self.unit_editor_campaign_groupbox.set_title(&qtr("tools_unit_editor_campaign"));
        self.unit_editor_ui_groupbox.set_title(&qtr("tools_unit_editor_ui"));
        self.unit_editor_audio_groupbox.set_title(&qtr("tools_unit_editor_audio"));

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//
        self.main_units_unit_label.set_text(&QString::from_std_str(&clean_column_names("main_units_unit")));
        self.main_units_caste_label.set_text(&QString::from_std_str(&clean_column_names("main_units_caste")));

        //-----------------------------------------------------------------------//
        // unit_variants_tables
        //-----------------------------------------------------------------------//
        self.unit_icon_key_label.set_text(&QString::from_std_str(clean_column_names("unit_variants_unit_card")));

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        self.loc_land_units_onscreen_name_label.set_text(&QString::from_std_str(&clean_column_names("land_units_onscreen_name")));
        self.loc_unit_description_historical_text_key_label.set_text(&QString::from_std_str(&clean_column_names("unit_description_historical_text_key")));
        self.loc_unit_description_short_texts_text_label.set_text(&QString::from_std_str(&clean_column_names("unit_description_short_texts_text")));
        self.loc_unit_description_strengths_weaknesses_texts_text_label.set_text(&QString::from_std_str(&clean_column_names("unit_description_strengths_weaknesses_texts_text")));
    }

    /// This function gets the data needed for the tool from the land_units table.
    unsafe fn get_battle_set_piece_armies_characters_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "battle_set_piece_armies_characters", "character_name", Some(("main_units".to_owned(), "unit".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the land_units table.
    unsafe fn get_land_units_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "land_units", "key", Some(("main_units".to_owned(), "land_unit".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the main_units table.
    unsafe fn get_main_units_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        if let Some(table) = Tool::get_table_data(data, processed_data, "main_units", "unit", None)? {
            let reference_data = get_reference_data("main_units_tables", table.get_ref_definition())?;

            self.tool.load_reference_data_to_detailed_view_editor_combo(table.get_column_position_by_name("caste")? as i32, &self.main_units_caste_combobox, &reference_data);
        }

        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_historical_text table.
    unsafe fn get_unit_description_historical_text_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_historical_texts", "key", Some(("land_units".to_owned(), "historical_description_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_short_texts table.
    unsafe fn get_unit_description_short_texts_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_short_texts", "key", Some(("land_units".to_owned(), "short_description_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_strengths_weaknesses_texts table.
    unsafe fn get_unit_description_strengths_weaknesses_texts_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_strengths_weaknesses_texts", "key", Some(("land_units".to_owned(), "strengths_&_weaknesses_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_variants table.
    unsafe fn get_unit_variants_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_variants", "name", Some(("land_units".to_owned(), "key".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the locs available.
    unsafe fn get_loc_data(&self, data: &mut HashMap<Vec<String>, PackedFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_loc_data(data, processed_data, &LOC_KEYS)
    }

    /// This function updates the reference keys in all values of an entry.
    unsafe fn update_keys(&self, data: &mut HashMap<String, String>) {
        self.tool.update_keys(data);
    }

    /// This function takes care of saving the land_units related data into a PackedFile.
    unsafe fn save_land_units_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "land_units", &self.get_file_name())
    }

    /// This function takes care of saving the main_units related data into a PackedFile.
    unsafe fn save_main_units_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "main_units", &self.get_file_name())
    }

    /// This function takes care of saving the unit_description_historical_texts related data into a PackedFile.
    unsafe fn save_unit_description_historical_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "unit_description_historical_texts", &self.get_file_name())
    }

    /// This function takes care of saving the unit_description_short_texts related data into a PackedFile.
    unsafe fn save_unit_description_short_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "unit_description_short_texts", &self.get_file_name())
    }

    /// This function takes care of saving the unit_description_strengths_weaknesses_texts related data into a PackedFile.
    unsafe fn save_unit_description_strengths_weaknesses_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "unit_description_strengths_weaknesses_texts", &self.get_file_name())
    }

    /// This function takes care of saving the unit_variants related data into a PackedFile.
    unsafe fn save_unit_variants_tables_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_table_data(data, "unit_variants", &self.get_file_name())
    }

    /// This function takes care of saving all the loc-related data into a PackedFile.
    unsafe fn save_loc_data(&self, data: &[HashMap<String, String>]) -> Result<PackedFile> {
        self.tool.save_loc_data(data, &self.get_file_name(), &LOC_KEYS)
    }

    /// This function returns the file name this tool uses for the PackedFiles, when a PackedFile has no specific name.
    unsafe fn get_file_name(&self) -> String {
        let packed_file_name = self.packed_file_name_line_edit.text();
        if !packed_file_name.is_empty() {
            packed_file_name.to_std_string()
        } else {
            DEFAULT_FILENAME.to_owned()
        }
    }

    /// Function to setup all the relations between widgets.
    pub unsafe fn setup_widgets_relations(&mut self) {
        /*
        let mut widgets_hero = vec![];
        self.unit_type_dependant_widgets.insert("hero", widgets_hero);

        let mut widgets_land_unit = vec![];
        widgets_land_unit.push(self.loc_land_units_onscreen_name_line_edit.static_upcast());
        self.unit_type_dependant_widgets.insert(UnitType::LandUnit, widgets_land_unit);

        let mut widgets_naval_unit = vec![];
        self.unit_type_dependant_widgets.insert(UnitType::NavalUnit, widgets_naval_unit);
        */
    }
}
