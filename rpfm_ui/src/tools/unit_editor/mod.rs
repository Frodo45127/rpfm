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
Module with all the code for managing the Unit Editor tool.

This tool is a dialog where you can pick a unit from a list, and edit its values in an easy-to-use way.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::q_dialog_button_box;
use qt_widgets::q_dialog_button_box::ButtonRole;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QPushButton;
use qt_widgets::QTabWidget;
use qt_widgets::QTextEdit;
use qt_widgets::QToolButton;

use qt_gui::QPixmap;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::AlignmentFlag;
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
use qt_core::SortOrder;

use cpp_core::Ref;

use getset::*;
use itertools::Itertools;

use std::collections::HashMap;
use std::sync::RwLock;

use rpfm_lib::files::{ContainerPath, FileType, RFile, text::{Text, TextFormat}};
use rpfm_lib::games::supported_games::*;
use rpfm_lib::integrations::log;

use rpfm_ui_common::locale::{qtr, tr};

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::tools::unit_editor::variant_unit_editor::SubToolVariantUnitEditor;
use crate::views::table::utils::{clean_column_names, get_reference_data};

use self::slots::ToolUnitEditorSlots;

use super::*;

mod connections;
mod slots;
mod variant_unit_editor;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_unit_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_unit_editor.ui";

/// Copy Unit's ui template path.
const COPY_UNIT_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_unit_editor_copy_unit.ui";
const COPY_UNIT_VIEW_RELEASE: &str = "ui/tool_unit_editor_copy_unit.ui";

/// Path and extension of variant meshes.
const VARIANT_MESH_PATH: &str = "variantmeshes/variantmeshdefinitions/";
const VARIANT_MESH_EXTENSION: &str = "variantmeshdefinition";

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 1] = [
    KEY_WARHAMMER_2,
];

/// Default name for files saved with this tool.
const DEFAULT_FILENAME: &str = "unit_edited";

/// Role that stores the data of the unit represented by each item.
const UNIT_DATA: i32 = 60;

/// Key under which variantmeshes data is saved.
const VARIANT_MESH_DATA: &str = "variants_variant_mesh_data";

/// Path where all the unit icon pictures (for backup) are located.
const UNIT_ICONS_PATH: &str = "ui/units/icons/";

/// List of fields tht require special treatment from land_units_tables.
const LAND_UNITS_CUSTOM_FIELDS: [&str; 3] = [
    "short_description_text",
    "historical_description_text",
    "strengths_&_weaknesses_text"
];

/// List of fields tht require special treatment from main_units_tables.
const MAIN_UNITS_CUSTOM_FIELDS: [&str; 1] = [
    "land_unit",
];

/// List of fields tht require special treatment from unit_variants_tables.
const UNIT_VARIANTS_CUSTOM_FIELDS: [&str; 4] = [
    "unit_card",
    "faction",
    "unit",
    "variant",
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
#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ToolUnitEditor {
    tool: Tool,
    timer_delayed_updates: QBox<QTimer>,

    unit_list_view: QPtr<QListView>,
    unit_list_filter: QBox<QSortFilterProxyModel>,
    unit_list_model: QBox<QStandardItemModel>,
    unit_list_filter_line_edit: QPtr<QLineEdit>,

    detailed_view_tab_widget: QPtr<QTabWidget>,

    unit_icon_label_preview_widget: QPtr<QWidget>,
    variant_editor_tool_button: QPtr<QToolButton>,

    packed_file_name_label: QPtr<QLabel>,
    packed_file_name_line_edit: QPtr<QLineEdit>,
    copy_button: QPtr<QPushButton>,

    unit_caste_previous: Rc<RwLock<String>>,
    unit_type_dependant_widgets: HashMap<String, Vec<QPtr<QWidget>>>,

    //-----------------------------------------------------------------------//
    // Copy unit dialog.
    //-----------------------------------------------------------------------//
    copy_unit_widget: QBox<QWidget>,
    copy_unit_button_box: QPtr<QDialogButtonBox>,
    copy_unit_instructions_label: QPtr<QLabel>,
    copy_unit_new_unit_name_label: QPtr<QLabel>,
    copy_unit_new_unit_name_combobox: QPtr<QComboBox>,
    copy_unit_new_unit_name_combobox_model: QBox<QStandardItemModel>,

    //-----------------------------------------------------------------------//
    // Main tab groupboxes.
    //-----------------------------------------------------------------------//
    unit_editor_key_loc_data_groupbox: QPtr<QGroupBox>,
    unit_editor_requirements_groupbox: QPtr<QGroupBox>,
    unit_editor_campaign_groupbox: QPtr<QGroupBox>,
    unit_editor_ui_groupbox: QPtr<QGroupBox>,
    unit_editor_audio_groupbox: QPtr<QGroupBox>,
    unit_editor_battle_visibility_groupbox: QPtr<QGroupBox>,

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
            //ContainerPath::Folder(vec!["db".to_owned(), "battle_set_piece_armies_characters_tables".to_owned()]),
            ContainerPath::Folder("db/land_units_tables".to_owned()),
            ContainerPath::Folder("db/main_units_tables".to_owned()),
            ContainerPath::Folder("db/unit_description_historical_texts_tables".to_owned()),
            ContainerPath::Folder("db/unit_description_short_texts_tables".to_owned()),
            ContainerPath::Folder("db/unit_description_strengths_weaknesses_texts_tables".to_owned()),
            ContainerPath::Folder("db/unit_variants_colours_tables".to_owned()),
            ContainerPath::Folder("db/unit_variants_tables".to_owned()),
            ContainerPath::Folder("db/variants_tables".to_owned()),
            ContainerPath::Folder("text".to_owned()),
        ];

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let tool = Tool::new(app_ui.main_window(), &paths, &TOOL_SUPPORTED_GAMES, view)?;
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
        let timer_delayed_updates = QTimer::new_1a(tool.main_widget());
        timer_delayed_updates.set_single_shot(true);

        // Icon stuff.
        let unit_icon_label_preview_widget: QPtr<QWidget> = tool.find_widget("unit_icon_label_preview_widget")?;
        let variant_editor_tool_button: QPtr<QToolButton> = tool.find_widget("variant_editor_tool_button")?;
        create_grid_layout(unit_icon_label_preview_widget.static_upcast());

        // File name and button box.
        let packed_file_name_label: QPtr<QLabel> = tool.find_widget("packed_file_name_label")?;
        let packed_file_name_line_edit: QPtr<QLineEdit> = tool.find_widget("packed_file_name_line_edit")?;
        packed_file_name_line_edit.set_text(&QString::from_std_str(DEFAULT_FILENAME));

        let copy_button = tool.find_widget::<QDialogButtonBox>("button_box")?.add_button_q_string_button_role(&qtr("copy_unit"), ButtonRole::ActionRole);
        copy_button.set_enabled(false);

        // Extra stuff.
        let detailed_view_tab_widget: QPtr<QTabWidget> = tool.find_widget("detailed_view_tab_widget")?;
        detailed_view_tab_widget.set_enabled(false);

        //-----------------------------------------------------------------------//
        // Copy unit dialog.
        //-----------------------------------------------------------------------//
        let copy_unit_view = if cfg!(debug_assertions) { COPY_UNIT_VIEW_DEBUG } else { COPY_UNIT_VIEW_RELEASE };
        let copy_unit_widget = crate::utils::load_template(&tool.main_widget, copy_unit_view)?;

        let copy_unit_button_box: QPtr<QDialogButtonBox> = tool.find_widget("copy_unit_button_box")?;
        let copy_unit_instructions_label: QPtr<QLabel> = tool.find_widget("copy_unit_instructions_label")?;
        let copy_unit_new_unit_name_label: QPtr<QLabel> = tool.find_widget("copy_unit_new_unit_name_label")?;
        let copy_unit_new_unit_name_combobox: QPtr<QComboBox> = tool.find_widget("copy_unit_new_unit_name_combobox")?;
        let copy_unit_new_unit_name_combobox_model = QStandardItemModel::new_1a(&copy_unit_new_unit_name_combobox);
        copy_unit_new_unit_name_combobox.set_model(&copy_unit_new_unit_name_combobox_model);

        //-----------------------------------------------------------------------//
        // Main tab groupboxes.
        //-----------------------------------------------------------------------//
        let unit_editor_key_loc_data_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_key_loc_data_groupbox")?;
        let unit_editor_requirements_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_requirements_groupbox")?;
        let unit_editor_campaign_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_campaign_groupbox")?;
        let unit_editor_ui_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_ui_groupbox")?;
        let unit_editor_audio_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_audio_groupbox")?;
        let unit_editor_battle_visibility_groupbox: QPtr<QGroupBox> = tool.find_widget("unit_battle_visibility_groupbox")?;

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

            unit_icon_label_preview_widget,
            variant_editor_tool_button,

            packed_file_name_label,
            packed_file_name_line_edit,
            copy_button,

            unit_caste_previous,
            unit_type_dependant_widgets,

            //-----------------------------------------------------------------------//
            // Copy unit dialog.
            //-----------------------------------------------------------------------//
            copy_unit_widget,
            copy_unit_button_box,
            copy_unit_instructions_label,
            copy_unit_new_unit_name_label,
            copy_unit_new_unit_name_combobox,
            copy_unit_new_unit_name_combobox_model,

            //-----------------------------------------------------------------------//
            // Main tab groupboxes.
            //-----------------------------------------------------------------------//
            unit_editor_key_loc_data_groupbox,
            unit_editor_requirements_groupbox,
            unit_editor_campaign_groupbox,
            unit_editor_ui_groupbox,
            unit_editor_audio_groupbox,
            unit_editor_battle_visibility_groupbox,

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
        connections::set_connections(&view, &slots)?;

        // Setup text translations.
        view.setup_translations();

        // Load all the data to the view.
        view.load_data()?;

        // If we hit ok, save the data back to the PackFile.
        if view.tool.get_ref_dialog().exec() == 1 {
            view.save_data(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui)?;
        }

        // If nothing failed, it means we have successfully saved the data back to disk, or canceled.wh_main_teb_cha_captain_0
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
        // NOTE: Order matters here.
        get_data_from_all_sources!(self, get_main_units_data, data, processed_data);
        get_data_from_all_sources!(self, get_land_units_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_historical_text_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_short_texts_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_description_strengths_weaknesses_texts_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_variants_data, data, processed_data);
        get_data_from_all_sources!(self, get_unit_variants_colours_data, data, processed_data);
        get_data_from_all_sources!(self, get_variants_data, data, processed_data);
        get_data_from_all_sources!(self, get_loc_data, data, processed_data);

        // Once we got everything processed, build the items for the ListView.
        for (key, data) in processed_data.iter().sorted_by_key(|x| x.0) {
            let item = QStandardItem::from_q_string(&QString::from_std_str(key)).into_ptr();
            item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(data)?)), UNIT_DATA);
            self.unit_list_model.append_row_q_standard_item(item);
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
        let unit_variants_colours_packed_file = self.save_unit_variants_colours_tables_data(&data_to_save)?;
        let unit_variants_packed_file = self.save_unit_variants_tables_data(&data_to_save)?;
        let variants_packed_file = self.save_variants_tables_data(&data_to_save)?;

        let loc_packed_file = self.save_loc_data(&data_to_save)?;

        let mut variant_meshes_packed_files = self.save_variant_meshes_data(&data_to_save)?;

        // Join all PackedFiles together to pass them to the save function.
        let mut packed_files = vec![
            land_units_packed_file,
            main_units_packed_file,
            unit_description_historical_texts_packed_file,
            unit_description_short_texts_packed_file,
            unit_description_strengths_weaknesses_texts_packed_file,
            unit_variants_colours_packed_file,
            unit_variants_packed_file,
            variants_packed_file,

            loc_packed_file
        ];

        // Also add the edited variant_meshes.
        packed_files.append(&mut variant_meshes_packed_files);

        // Once we got the PackedFiles to save properly edited, call the generic tool `save` function to save them to a PackFile.
        self.tool.save(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, &packed_files)
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {

        // If it's the first faction loaded into the detailed view, enable the groupboxes so they can be edited.
        if !self.detailed_view_tab_widget.is_enabled() {
            self.detailed_view_tab_widget.set_enabled(true);
        }

        let data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors = vec![];

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        //-----------------------------------------------------------------------//
        // land_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "land_units", &LAND_UNITS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.load_definition_to_detailed_view_editor(&data, "main_units", &MAIN_UNITS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        self.tool.load_field_to_detailed_view_editor_string_short(&data, &self.loc_land_units_onscreen_name_line_edit, "loc_land_units_onscreen_name");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_historical_text_key_ktexteditor, "loc_unit_description_historical_texts_text");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_short_texts_text_ktexteditor, "loc_unit_description_short_texts_text");
        self.tool.load_field_to_detailed_view_editor_string_long(&data, &self.loc_unit_description_strengths_weaknesses_texts_text_ktexteditor, "loc_unit_description_strengths_weaknesses_texts_text");

        // The icon needs to be pulled up from the dependencies cache on load.
        if let Err(error) = self.load_unit_icons(&data) {
            errors.push(error.to_string());
        }

        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }
    }

    /// This function loads the unit icon into the tool. If provided with a key, it uses it. If not, it uses whatever key the unit has.
    pub unsafe fn load_unit_icons(&self, data: &HashMap<String, String>) -> Result<()> {

        // Clear the current layout.
        clear_layout(&self.unit_icon_label_preview_widget);
        let layout: QPtr<QGridLayout> = self.unit_icon_label_preview_widget.layout().static_downcast();

        let unit_cards = data.iter().filter_map(|(key, value)| if key.starts_with("unit_variants_unit_card") { Some(value) } else { None }).collect::<Vec<&String>>();

        // The icons needs to be pulled up from the dependencies cache on load.
        if !unit_cards.is_empty() {
            let mut icon_paths = vec![];
            for unit_card in unit_cards {
                let icon_path_png = ContainerPath::File(format!("{}{}.png", UNIT_ICONS_PATH.to_owned(), unit_card));
                let icon_path_tga = ContainerPath::File(format!("{}{}.tga", UNIT_ICONS_PATH.to_owned(), unit_card));

                if !icon_paths.contains(&icon_path_png) {
                    icon_paths.push(icon_path_png);
                }

                if !icon_paths.contains(&icon_path_tga) {
                    icon_paths.push(icon_path_tga);
                }
            }

            let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(icon_paths.to_vec(), false));
            let response = CentralCommand::recv(&receiver);
            let images_data = if let Response::HashMapDataSourceHashMapStringRFile(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

            let mut images_files = icon_paths.iter().filter_map(|path_type| {
                if let ContainerPath::File(path) = path_type {
                    Tool::get_most_relevant_file(&images_data, path)
                } else { None }
            }).collect::<Vec<RFile>>();

            for (column, images_file) in images_files.iter_mut().enumerate() {
                if let Some(RFileDecoded::Image(decoded)) = images_file.decode(&None, false, true)? {
                    let image_data = decoded.data();
                    let byte_array = QByteArray::from_slice(image_data);
                    let image = QPixmap::new();
                    let label = QLabel::from_q_widget(&self.unit_icon_label_preview_widget);
                    image.load_from_data_q_byte_array(&byte_array);
                    label.set_alignment(AlignmentFlag::AlignHCenter | AlignmentFlag::AlignVCenter);
                    label.set_pixmap(&image);
                    layout.add_widget_3a(&label, 0, column as i32);
                }
            }
        } else {
            let label = QLabel::from_q_widget(&self.unit_icon_label_preview_widget);
            label.set_alignment(AlignmentFlag::AlignHCenter | AlignmentFlag::AlignVCenter);
            label.set_text(&QString::from_std_str("No image available"));
            layout.add_widget_3a(&label, 0, 0);
        }

        Ok(())
    }

    /// This function saves the data of the detailed view to its item in the faction list.
    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let mut errors = vec![];

        //-----------------------------------------------------------------------//
        // land_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "land_units", &LAND_UNITS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        //-----------------------------------------------------------------------//
        // main_units_tables
        //-----------------------------------------------------------------------//
        if let Err(error) = self.tool.save_definition_from_detailed_view_editor(&mut data, "main_units", &MAIN_UNITS_CUSTOM_FIELDS) {
            errors.push(error.to_string());
        }

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        data.insert("loc_land_units_onscreen_name".to_owned(), self.loc_land_units_onscreen_name_line_edit.text().to_std_string());
        data.insert("loc_unit_description_historical_texts_text".to_owned(), self.loc_unit_description_historical_text_key_ktexteditor.to_plain_text().to_std_string());
        data.insert("loc_unit_description_short_texts_text".to_owned(), self.loc_unit_description_short_texts_text_ktexteditor.to_plain_text().to_std_string());
        data.insert("loc_unit_description_strengths_weaknesses_texts_text".to_owned(), self.loc_unit_description_strengths_weaknesses_texts_text_ktexteditor.to_plain_text().to_std_string());

        // Update all the referenced keys in our data.
        self.update_keys(&mut data);

        if !errors.is_empty() {
            show_message_warning(&self.tool.message_widget, errors.join("\n"));
        }

        if cfg!(debug_assertions) {
            log::info!("{:#?}", data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }
        self.unit_list_model.item_from_index(index).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data).unwrap())), UNIT_DATA);
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
        self.unit_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.unit_list_view.selection_model().selection(), SelectionFlag::Toggle.into());

        self.unit_list_filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        self.unit_list_filter.set_filter_regular_expression_q_string(&self.unit_list_filter_line_edit.text());

        self.unit_list_view.selection_model().select_q_item_selection_q_flags_selection_flag(&self.unit_list_view.selection_model().selection(), SelectionFlag::Toggle.into());
    }

    /// Function to setup all the translations of this view.
    pub unsafe fn setup_translations(&self) {
        self.packed_file_name_label.set_text(&qtr("packed_file_name"));

        self.detailed_view_tab_widget.set_tab_text(0, &qtr("tools_unit_editor_main_tab_title"));
        self.detailed_view_tab_widget.set_tab_text(1, &qtr("tools_unit_editor_land_unit_tab_title"));
        self.detailed_view_tab_widget.set_tab_text(2, &qtr("tools_unit_editor_variantmeshes_tab_title"));

        //-----------------------------------------------------------------------//
        // Copy unit dialog.
        //-----------------------------------------------------------------------//
        self.copy_unit_instructions_label.set_text(&qtr("copy_unit_instructions"));
        self.copy_unit_new_unit_name_label.set_text(&qtr("copy_unit_new_unit_name"));

        //-----------------------------------------------------------------------//
        // Main tab groupboxes.
        //-----------------------------------------------------------------------//
        self.unit_editor_key_loc_data_groupbox.set_title(&qtr("tools_unit_editor_key_loc_data"));
        self.unit_editor_requirements_groupbox.set_title(&qtr("tools_unit_editor_requirements"));
        self.unit_editor_campaign_groupbox.set_title(&qtr("tools_unit_editor_campaign"));
        self.unit_editor_ui_groupbox.set_title(&qtr("tools_unit_editor_ui"));
        self.unit_editor_audio_groupbox.set_title(&qtr("tools_unit_editor_audio"));
        self.unit_editor_battle_visibility_groupbox.set_title(&qtr("tools_unit_battle_visibility"));

        //-----------------------------------------------------------------------//
        // Loc data
        //-----------------------------------------------------------------------//
        self.loc_land_units_onscreen_name_label.set_text(&QString::from_std_str(clean_column_names("land_units_onscreen_name")));
        self.loc_unit_description_historical_text_key_label.set_text(&QString::from_std_str(clean_column_names("unit_description_historical_text_key")));
        self.loc_unit_description_short_texts_text_label.set_text(&QString::from_std_str(clean_column_names("unit_description_short_texts_text")));
        self.loc_unit_description_strengths_weaknesses_texts_text_label.set_text(&QString::from_std_str(clean_column_names("unit_description_strengths_weaknesses_texts_text")));
    }

    // This function gets the data needed for the tool from the land_units table.
    //unsafe fn get_battle_set_piece_armies_characters_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
    //    Tool::get_table_data(data, processed_data, "battle_set_piece_armies_characters", &["character_name"], Some(("main_units".to_owned(), "unit".to_owned())))?;
    //    Ok(())
    //}

    /// This function gets the data needed for the tool from the land_units table.
    unsafe fn get_land_units_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        if let Some(table) = Tool::get_table_data(data, processed_data, "land_units", &["key"], Some(("main_units".to_owned(), "land_unit".to_owned())))? {
            let reference_data = get_reference_data(FileType::DB, "land_units_tables", table.definition())?;

            let column_ai_usage_group = table.column_position_by_name("ai_usage_group").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "ai_usage_group".to_string()))? as i32;
            let column_animal = table.column_position_by_name("animal").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "animal".to_string()))? as i32;
            let column_armour = table.column_position_by_name("armour").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "armour".to_string()))? as i32;
            let column_articulated_record = table.column_position_by_name("articulated_record").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "articulated_record".to_string()))? as i32;
            let column_attribute_group = table.column_position_by_name("attribute_group").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "attribute_group".to_string()))? as i32;
            let column_category = table.column_position_by_name("category").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "category".to_string()))? as i32;
            let column_class = table.column_position_by_name("class").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "class".to_string()))? as i32;
            let column_engine = table.column_position_by_name("engine").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "engine".to_string()))? as i32;
            let column_ground_stat_effect_group = table.column_position_by_name("ground_stat_effect_group").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "ground_stat_effect_group".to_string()))? as i32;
            let column_man_animation = table.column_position_by_name("man_animation").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "man_animation".to_string()))? as i32;
            let column_man_entity = table.column_position_by_name("man_entity").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "man_entity".to_string()))? as i32;
            let column_mount = table.column_position_by_name("mount").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "mount".to_string()))? as i32;
            let column_officers = table.column_position_by_name("officers").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "officers".to_string()))? as i32;
            let column_primary_melee_weapon = table.column_position_by_name("primary_melee_weapon").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "primary_melee_weapon".to_string()))? as i32;
            let column_primary_missile_weapon = table.column_position_by_name("primary_missile_weapon").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "primary_missile_weapon".to_string()))? as i32;
            let column_selection_vo = table.column_position_by_name("selection_vo").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "selection_vo".to_string()))? as i32;
            let column_selected_vo_secondary = table.column_position_by_name("selected_vo_secondary").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "selected_vo_secondary".to_string()))? as i32;
            let column_selected_vo_tertiary = table.column_position_by_name("selected_vo_tertiary").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "selected_vo_tertiary".to_string()))? as i32;
            let column_shield = table.column_position_by_name("shield").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "shield".to_string()))? as i32;
            let column_spacing = table.column_position_by_name("spacing").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "spacing".to_string()))? as i32;
            let column_training_level = table.column_position_by_name("training_level").ok_or_else(|| ToolsError::MissingColumnInTable("land_units".to_string(), "training_level".to_string()))? as i32;

            self.tool.load_reference_data_to_detailed_view_editor_combo(column_ai_usage_group, &self.tool.find_widget("land_units_ai_usage_group_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_animal, &self.tool.find_widget("land_units_animal_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_armour, &self.tool.find_widget("land_units_armour_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_articulated_record, &self.tool.find_widget("land_units_articulated_record_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_attribute_group, &self.tool.find_widget("land_units_attribute_group_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_category, &self.tool.find_widget("land_units_category_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_class, &self.tool.find_widget("land_units_class_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_engine, &self.tool.find_widget("land_units_engine_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_ground_stat_effect_group, &self.tool.find_widget("land_units_ground_stat_effect_group_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_man_animation, &self.tool.find_widget("land_units_man_animation_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_man_entity, &self.tool.find_widget("land_units_man_entity_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_mount, &self.tool.find_widget("land_units_mount_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_officers, &self.tool.find_widget("land_units_officers_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_primary_melee_weapon, &self.tool.find_widget("land_units_primary_melee_weapon_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_primary_missile_weapon, &self.tool.find_widget("land_units_primary_missile_weapon_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_selection_vo, &self.tool.find_widget("land_units_selection_vo_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_selected_vo_secondary, &self.tool.find_widget("land_units_selected_vo_secondary_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_selected_vo_tertiary, &self.tool.find_widget("land_units_selected_vo_tertiary_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_shield, &self.tool.find_widget("land_units_shield_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_spacing, &self.tool.find_widget("land_units_spacing_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_training_level, &self.tool.find_widget("land_units_training_level_combobox")?, &reference_data);
        }
        Ok(())
    }

    /// This function gets the data needed for the tool from the main_units table.
    unsafe fn get_main_units_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        if let Some(table) = Tool::get_table_data(data, processed_data, "main_units", &["unit"], None)? {
            let reference_data = get_reference_data(FileType::DB, "main_units_tables", table.definition())?;

            let column_additional_building_requirement = table.column_position_by_name("additional_building_requirement").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "additional_building_requirement".to_string()))? as i32;
            let column_audio_voiceover_actor_group = table.column_position_by_name("audio_voiceover_actor_group").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "audio_voiceover_actor_group".to_string()))? as i32;
            let column_audio_voiceover_culture = table.column_position_by_name("audio_voiceover_culture").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "audio_voiceover_culture".to_string()))? as i32;
            let column_audio_voiceover_culture_override = table.column_position_by_name("audio_voiceover_culture_override").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "audio_voiceover_culture_override".to_string()))? as i32;
            let column_caste = table.column_position_by_name("caste").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "caste".to_string()))? as i32;
            let column_mount = table.column_position_by_name("mount").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "mount".to_string()))? as i32;
            let column_naval_unit = table.column_position_by_name("naval_unit").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "naval_unit".to_string()))? as i32;
            let column_porthole_camera = table.column_position_by_name("porthole_camera").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "porthole_camera".to_string()))? as i32;
            let column_region_unit_resource_requirement = table.column_position_by_name("region_unit_resource_requirement").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "region_unit_resource_requirement".to_string()))? as i32;
            let column_religion_requirement = table.column_position_by_name("religion_requirement").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "religion_requirement".to_string()))? as i32;
            let column_resource_requirement = table.column_position_by_name("resource_requirement").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "resource_requirement".to_string()))? as i32;
            let column_ui_unit_group_land = table.column_position_by_name("ui_unit_group_land").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "ui_unit_group_land".to_string()))? as i32;
            let column_ui_unit_group_naval = table.column_position_by_name("ui_unit_group_naval").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "ui_unit_group_naval".to_string()))? as i32;
            let column_weight = table.column_position_by_name("weight").ok_or_else(|| ToolsError::MissingColumnInTable("main_units".to_string(), "weight".to_string()))? as i32;

            self.tool.load_reference_data_to_detailed_view_editor_combo(column_additional_building_requirement, &self.tool.find_widget("main_units_additional_building_requirement_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_audio_voiceover_actor_group, &self.tool.find_widget("main_units_audio_voiceover_actor_group_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_audio_voiceover_culture, &self.tool.find_widget("main_units_audio_voiceover_culture_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_audio_voiceover_culture_override, &self.tool.find_widget("main_units_audio_voiceover_culture_override_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_caste, &self.tool.find_widget("main_units_caste_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_mount, &self.tool.find_widget("main_units_mount_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_naval_unit, &self.tool.find_widget("main_units_naval_unit_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_porthole_camera, &self.tool.find_widget("main_units_porthole_camera_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_region_unit_resource_requirement, &self.tool.find_widget("main_units_region_unit_resource_requirement_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_religion_requirement, &self.tool.find_widget("main_units_religion_requirement_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_resource_requirement, &self.tool.find_widget("main_units_resource_requirement_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_ui_unit_group_land, &self.tool.find_widget("main_units_ui_unit_group_land_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_ui_unit_group_naval, &self.tool.find_widget("main_units_ui_unit_group_naval_combobox")?, &reference_data);
            self.tool.load_reference_data_to_detailed_view_editor_combo(column_weight, &self.tool.find_widget("main_units_weight_combobox")?, &reference_data);
        }

        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_historical_text table.
    unsafe fn get_unit_description_historical_text_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_historical_texts", &["key"], Some(("land_units".to_owned(), "historical_description_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_short_texts table.
    unsafe fn get_unit_description_short_texts_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_short_texts", &["key"], Some(("land_units".to_owned(), "short_description_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_description_strengths_weaknesses_texts table.
    unsafe fn get_unit_description_strengths_weaknesses_texts_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_description_strengths_weaknesses_texts", &["key"], Some(("land_units".to_owned(), "strengths_&_weaknesses_text".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_variants_colours table.
    unsafe fn get_unit_variants_colours_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_variants_colours", &["unit_variant", "key"], Some(("unit_variants".to_owned(), "name".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the unit_variants table.
    unsafe fn get_unit_variants_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "unit_variants", &["unit", "faction"], Some(("land_units".to_owned(), "key".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the variants table.
    unsafe fn get_variants_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_table_data(data, processed_data, "variants", &["variant_name"], Some(("unit_variants".to_owned(), "variant".to_owned())))?;
        Ok(())
    }

    /// This function gets the data needed for the tool from the locs available.
    unsafe fn get_loc_data(&self, data: &mut HashMap<String, RFile>, processed_data: &mut HashMap<String, HashMap<String, String>>) -> Result<()> {
        Tool::get_loc_data(data, processed_data, &LOC_KEYS)
    }

    /// This function updates the reference keys in all values of an entry.
    unsafe fn update_keys(&self, data: &mut HashMap<String, String>) {
        self.tool.update_keys(data);
    }

    /// This function takes care of saving the land_units related data into a RFile.
    unsafe fn save_land_units_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "land_units", &self.get_file_name(), &["key"])
    }

    /// This function takes care of saving the main_units related data into a RFile.
    unsafe fn save_main_units_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "main_units", &self.get_file_name(), &["unit"])
    }

    /// This function takes care of saving the unit_description_historical_texts related data into a RFile.
    unsafe fn save_unit_description_historical_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "unit_description_historical_texts", &self.get_file_name(), &["key"])
    }

    /// This function takes care of saving the unit_description_short_texts related data into a RFile.
    unsafe fn save_unit_description_short_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "unit_description_short_texts", &self.get_file_name(), &["key"])
    }

    /// This function takes care of saving the unit_description_strengths_weaknesses_texts related data into a RFile.
    unsafe fn save_unit_description_strengths_weaknesses_texts_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "unit_description_strengths_weaknesses_texts", &self.get_file_name(), &["key"])
    }

    /// This function takes care of saving the unit_variants_colours related data into a RFile.
    unsafe fn save_unit_variants_colours_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "unit_variants_colours", &self.get_file_name(), &["key"])
    }

    /// This function takes care of saving the unit_variants related data into a RFile.
    unsafe fn save_unit_variants_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "unit_variants", &self.get_file_name(), &["faction"])
    }

    /// This function takes care of saving the variants related data into a RFile.
    unsafe fn save_variants_tables_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_table_data(data, "variants", &self.get_file_name(), &["variant_name"])
    }

    /// This function takes care of saving all the loc-related data into a RFile.
    unsafe fn save_loc_data(&self, data: &[HashMap<String, String>]) -> Result<RFile> {
        self.tool.save_loc_data(data, &self.get_file_name(), &LOC_KEYS)
    }

    /// This function takes care of saving all the edited variant meshes into PackedFiles.
    unsafe fn save_variant_meshes_data(&self, data: &[HashMap<String, String>]) -> Result<Vec<RFile>> {
        let mut packed_files = vec![];
        for unit_data in data {
            let mut variant_meshes = unit_data.iter()
                .filter(|(key, _)| key.starts_with(VARIANT_MESH_DATA))
                .filter_map(|(key, value)| {
                    let subkeys = key.split('|').collect::<Vec<&str>>();
                    let file_name = if subkeys.len() > 1 {
                        unit_data.get(&format!("variants_variant_filename|{}", subkeys[1..].join("|")))?
                    } else {
                        unit_data.get(key)?
                    };

                    let mut text = Text::default();
                    text.set_contents(value.to_string());
                    text.set_format(TextFormat::Xml);

                    let path = format!("{}{}.{}", VARIANT_MESH_PATH, file_name, VARIANT_MESH_EXTENSION);
                    let packed_file = RFile::new_from_decoded(&RFileDecoded::Text(text), 0, &path);
                    Some(packed_file)
                })
                .collect::<Vec<RFile>>();
            packed_files.append(&mut variant_meshes);
        }

        Ok(packed_files)
    }

    /// This function returns the file name this tool uses for the PackedFiles, when a RFile has no specific name.
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

    /// Function to load the `Copy Unit` dialog.
    pub unsafe fn load_copy_unit_dialog(&self) -> Result<()> {
        let source_unit = self.unit_list_view.selection_model().selection();
        if source_unit.count_0a() != 1 {
            return Err(ToolsError::GenericError("No unit selected".to_string()).into());
        }

        let source_unit_real = self.unit_list_filter().map_to_source(&source_unit.take_at(0).indexes().take_at(0));
        let source_unit_name = self.unit_list_model().item_from_index(&source_unit_real).text();
        self.copy_unit_button_box.button(q_dialog_button_box::StandardButton::Ok).set_enabled(false);

        // Copy the model of the unit list and sort it, because here we don't use an intermediate filter.
        self.copy_unit_new_unit_name_combobox_model.clear();

        for row in 0..self.unit_list_model.row_count_0a() {
            let item = QStandardItem::from_q_string(&self.unit_list_model.item_1a(row).text());
            self.copy_unit_new_unit_name_combobox_model.append_row_q_standard_item(item.into_ptr());
        }

        self.copy_unit_new_unit_name_combobox_model.sort_2a(0, SortOrder::AscendingOrder);
        self.copy_unit_new_unit_name_combobox.set_current_text(&source_unit_name);

        let dialog: QPtr<QDialog> = self.copy_unit_widget.static_downcast();
        if dialog.exec() == 1 {

            // Save the source unit.
            self.save_from_detailed_view(source_unit_real.as_ref());

            // Clone the source unit, updating its relevant keys in the process.
            let new_item = (*self.unit_list_model.item_from_index(&source_unit_real)).clone();
            let new_unit_name = self.copy_unit_new_unit_name_combobox.current_text();
            new_item.set_text(&new_unit_name);

            let mut data: HashMap<String, String> = serde_json::from_str(&new_item.data_1a(UNIT_DATA).to_string().to_std_string())?;
            data.insert("main_units_unit".to_owned(), new_unit_name.to_std_string());
            data.insert("land_units_key".to_owned(), new_unit_name.to_std_string());
            self.update_keys(&mut data);
            new_item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data)?)), UNIT_DATA);

            self.unit_list_model.append_row_q_standard_item(new_item);
            let new_index = self.unit_list_model.index_from_item(new_item);

            // Clear the filters (just in case) and open the new unit.
            self.unit_list_filter_line_edit().clear();
            self.unit_list_filter().sort_2a(0, SortOrder::AscendingOrder);
            self.unit_list_view().set_current_index(&self.unit_list_filter().map_from_source(&new_index));
        }

        Ok(())
    }

    /// Function to load the `Variant Editor` dialog.
    pub unsafe fn open_variant_editor(&self) -> Result<()> {
        let selection = self.unit_list_view.selection_model().selection();
        let filter_index = selection.take_at(0).indexes().take_at(0);
        let index = self.unit_list_filter().map_to_source(filter_index.as_ref());

        let mut data: HashMap<String, String> = serde_json::from_str(&index.data_1a(UNIT_DATA).to_string().to_std_string()).unwrap();
        let variant_data: HashMap<String, String> = data.iter().filter_map(|(key, val)|
            if key.starts_with("unit_variants_colours") ||
                key.starts_with("unit_variants") ||
                key.starts_with("variants") {
                Some((key.to_owned(), val.to_owned()))
            } else { None }
        ).collect();

        // Log in debug mode, for debugging.
        if cfg!(debug_assertions) {
            log::info!("{:#?}", variant_data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
        }

        let new_data = SubToolVariantUnitEditor::new(self.tool.main_widget().as_ref().unwrap(), &variant_data)?;
        if let Some(new_data) = new_data {
            if cfg!(debug_assertions) {
                log::info!("{:#?}", new_data.iter().sorted_by_key(|x| x.0).collect::<std::collections::BTreeMap<&String, &String>>());
            }

            // Delete old variant entries before re-adding them.
            data = data.iter().filter_map(|(key, val)|
                if !key.starts_with("unit_variants_colours") &&
                    !key.starts_with("unit_variants") &&
                    !key.starts_with("variants") {
                    Some((key.to_owned(), val.to_owned()))
                } else { None }
            ).collect();

            data.extend(new_data);
            self.unit_list_model.item_from_index(index.as_ref()).set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&data)?)), UNIT_DATA);
        }

        Ok(())
    }
}
