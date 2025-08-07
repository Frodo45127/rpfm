//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QButtonGroup;
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::FileMode;
use qt_widgets::QGroupBox;
use qt_widgets::QRadioButton;
use qt_widgets::QToolButton;
use qt_widgets::q_abstract_item_view::{SelectionBehavior, SelectionMode};
use qt_widgets::QGridLayout;
use qt_widgets::QTableView;

use qt_core::CheckState;
use qt_core::QEventLoop;
use qt_core::QItemSelection;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QString;

use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::Ref;

use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::STANDARD};
use chat_gpt_lib_rs::api_resources::{completions::{create_completion, CreateCompletionRequest, PromptInput}, models::Model};
use chat_gpt_lib_rs::OpenAIClient;
use deepl::{DeepLApi, Lang, ModelType, TagHandling};
use getset::*;
use regex::{Captures, Regex};
use serde_json::Value;

use std::cell::LazyCell;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rpfm_extensions::translator::*;

use rpfm_lib::files::{Container, ContainerPath, FileType, pack::Pack, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::integrations::git::GitResponse;

use rpfm_ui_common::locale::{tr, tre, qtr};
use rpfm_ui_common::settings::*;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::translations_local_path;
use crate::views::table::{TableType, TableView, utils::get_table_from_view};

use self::slots::ToolTranslatorSlots;
use super::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_translator_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_translator_editor.ui";

pub const VANILLA_LOC_NAME: &str = "vanilla_english.tsv";
pub const VANILLA_FIXES_NAME: &str = "vanilla_fixes_";

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 13] = [
    KEY_PHARAOH_DYNASTIES,
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
    KEY_NAPOLEON,
    KEY_EMPIRE,
];

const REGEX_COLOR: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[col:(.*?)]](.*?)\[\[/col(.*?)]]").unwrap());
const REGEX_RGBA: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[rgba:(\d+?):(\d+?):(\d+?):(\d+?)]](.*?)\[\[/rgba(.*?)]]").unwrap());
const REGEX_RGB: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[rgba:(\d+?):(\d+?):(\d+?)]](.*?)\[\[/rgba(.*?)]]").unwrap());
const REGEX_IMG: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[img:(.+?)]](.*?)\[\[/img(.*?)]]").unwrap());
//const REGEX_TR: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\{\{tr:(.+?)}}").unwrap());

// These are all kind of internal links for different types of interactions.
const REGEX_URL: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[url:(.+?)]](.*?)\[\[/url(.*?)]]").unwrap());
const REGEX_SL: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[sl:(.+?)]](.*?)\[\[/sl(.*?)]]").unwrap());
const REGEX_SL_TOOLTIP: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[sl_tooltip:(.+?)]](.*?)\[\[/sl_tooltip(.*?)]]").unwrap());
const REGEX_TOOLTIP: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"\[\[tooltip:(.+?)]](.*?)\[\[/tooltip(.*?)]]").unwrap());

// While QTextDoc doesn't support this full css, we still have it just in case in the future there's a way to use it.
const CSS_STYLE: &str = "
.tooltip {
  position: relative;
  display: inline-block;
  border-bottom: 1px dotted black;
}

.tooltip .tooltiptext {
  visibility: hidden;
  width: 120px;
  background-color: black;
  color: #fff;
  text-align: center;
  border-radius: 6px;
  padding: 5px 0;

  /* Position the tooltip */
  position: absolute;
  z-index: 1;
}

.tooltip:hover .tooltiptext {
  visibility: visible;
}";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ToolTranslator {
    tool: Tool,
    pack_tr: Arc<PackTranslation>,
    table: Arc<TableView>,

    // Item with the key being edited.
    current_key: Arc<RwLock<Option<CppBox<QModelIndex>>>>,

    colors: HashMap<String, String>,
    tagged_images: HashMap<String, String>,
    language_combobox: QPtr<QComboBox>,

    deepl_radio_button: QPtr<QRadioButton>,
    chatgpt_radio_button: QPtr<QRadioButton>,
    google_translate_radio_button: QPtr<QRadioButton>,
    copy_source_radio_button: QPtr<QRadioButton>,

    context_text_edit: QPtr<QTextEdit>,
    edit_all_same_values_radio_button: QPtr<QRadioButton>,

    key_line_edit: QPtr<QLineEdit>,

    action_move_up: QPtr<QAction>,
    action_move_down: QPtr<QAction>,
    action_copy_from_source: QPtr<QAction>,
    action_import_from_translated_pack: QPtr<QAction>,

    move_selection_up: QPtr<QToolButton>,
    move_selection_down: QPtr<QToolButton>,
    translate_with_deepl: QPtr<QToolButton>,
    translate_with_chatgpt: QPtr<QToolButton>,
    translate_with_google: QPtr<QToolButton>,
    copy_from_source: QPtr<QToolButton>,
    import_from_translated_pack: QPtr<QToolButton>,

    original_value_html: QPtr<QTextEdit>,
    original_value_textedit: QPtr<QTextEdit>,
    translated_value_html: QPtr<QTextEdit>,
    translated_value_textedit: QPtr<QTextEdit>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ToolTranslator {

    /// This function creates the tool's dialog.
    ///
    /// NOTE: This can fail at runtime if any of the expected widgets is not in the UI's XML.
    pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Result<()> {

        let paths = vec![ContainerPath::File(TRANSLATED_PATH.to_owned())];

        // Initialize a Tool. This also performs some common checks to ensure we can actually use the tool.
        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let tool = Tool::new(app_ui.main_window(), &paths, &TOOL_SUPPORTED_GAMES, view)?;
        tool.set_title(&tr("translator_title"));
        tool.backup_used_paths(app_ui, pack_file_contents_ui)?;

        // Translations list.
        let table_view: QPtr<QTableView> = tool.find_widget("table_view")?;
        let table_view_container: QPtr<QWidget> = tool.find_widget("table_view_container")?;
        let table_view_container = table_view_container.into_q_box();

        let language_label: QPtr<QLabel> = tool.find_widget("language_label")?;
        let language_combobox: QPtr<QComboBox> = tool.find_widget("language_combobox")?;
        language_label.set_text(&qtr("translator_language"));

        let behavior_groupbox: QPtr<QGroupBox> = tool.find_widget("behavior_groupbox")?;
        let behavior_label: QPtr<QLabel> = tool.find_widget("behavior_label")?;
        let context_label: QPtr<QLabel> = tool.find_widget("context_label")?;
        let context_text_edit: QPtr<QTextEdit> = tool.find_widget("context_text_edit")?;
        let deepl_radio_button: QPtr<QRadioButton> = tool.find_widget("deepl_radio")?;
        let chatgpt_radio_button: QPtr<QRadioButton> = tool.find_widget("chatgpt_radio")?;
        let google_translate_radio_button: QPtr<QRadioButton> = tool.find_widget("google_translate_radio")?;
        let copy_source_radio_button: QPtr<QRadioButton> = tool.find_widget("copy_source_radio")?;
        let empty_radio_button: QPtr<QRadioButton> = tool.find_widget("empty_radio")?;
        context_label.set_text(&qtr("context"));
        behavior_groupbox.set_title(&qtr("behavior_title"));
        behavior_label.set_text(&qtr("behavior_info"));
        deepl_radio_button.set_text(&qtr("behavior_deepl"));
        chatgpt_radio_button.set_text(&qtr("behavior_chatgpt"));
        google_translate_radio_button.set_text(&qtr("behavior_google_translate"));
        copy_source_radio_button.set_text(&qtr("behavior_copy_source"));
        empty_radio_button.set_text(&qtr("behavior_empty"));

        let behavior_group = QButtonGroup::new_1a(&behavior_groupbox);
        behavior_group.add_button_1a(&deepl_radio_button);
        behavior_group.add_button_1a(&chatgpt_radio_button);
        behavior_group.add_button_1a(&google_translate_radio_button);
        behavior_group.add_button_1a(&copy_source_radio_button);
        behavior_group.add_button_1a(&empty_radio_button);
        behavior_group.set_exclusive(true);
        google_translate_radio_button.set_checked(true);

        let behavior_edit_label: QPtr<QLabel> = tool.find_widget("behavior_edit_label")?;
        let edit_all_same_values_radio_button: QPtr<QRadioButton> = tool.find_widget("edit_all_same_values_radio")?;
        let edit_only_this_value_radio_button: QPtr<QRadioButton> = tool.find_widget("edit_only_this_value_radio")?;
        behavior_edit_label.set_text(&qtr("behavior_edit_info"));
        edit_all_same_values_radio_button.set_text(&qtr("behavior_edit_all_same_values"));
        edit_only_this_value_radio_button.set_text(&qtr("behavior_edit_only_this_value"));

        let behavior_edit_group = QButtonGroup::new_1a(&behavior_groupbox);
        behavior_edit_group.add_button_1a(&edit_all_same_values_radio_button);
        behavior_edit_group.add_button_1a(&edit_only_this_value_radio_button);
        behavior_edit_group.set_exclusive(true);
        edit_all_same_values_radio_button.set_checked(true);

        // For language, we try to get it from the game folder. If we can't, we fallback to whatever local files we have.
        let game = GAME_SELECTED.read().unwrap().clone();
        let game_path = setting_path(game.key());
        let locale = game.game_locale_from_file(&game_path)?;
        let language = match locale {
            Some(locale) => {
                let language = locale.to_uppercase();
                language_combobox.insert_item_int_q_string(0, &QString::from_std_str(&language));
                language_combobox.set_current_index(0);
                language
            },
            None => {
                if let Ok(ca_packs) = game.ca_packs_paths(&game_path) {
                    let mut languages = ca_packs.iter()
                        .filter_map(|path| path.file_stem())
                        .filter(|name| name.to_string_lossy().starts_with("local_"))
                        .map(|name| name.to_string_lossy().split_at(6).1.to_uppercase())
                        .collect::<Vec<_>>();

                    // Sort, and remove anything longer than 2 characters to avoid duplicates.
                    languages.retain(|lang| lang.chars().count() == 2);
                    languages.sort();

                    for (index, language) in languages.iter().enumerate() {
                        language_combobox.insert_item_int_q_string(index as i32, &QString::from_std_str(language));
                    }

                    // If there's more than 1 possible language, allow to alter the language.
                    if languages.len() > 1 {
                        language_combobox.set_enabled(true);
                    }

                    language_combobox.set_current_index(0);
                    languages[0].to_owned()
                } else {
                    return Err(anyhow!("The translator couldn't figure out what languages you have for the game."));
                }
            }
        };

        // Get the list of colours supported by the game. They're in the ui_colours table in the modern games.
        let mut colors = HashMap::new();
        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(vec![ContainerPath::Folder("db/ui_colours_tables".to_owned())], false));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                let mut files_merge = HashMap::new();
                if let Some(files) = files.remove(&DataSource::GameFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::ParentFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::PackFile) {
                    files_merge.extend(files);
                }

                for (_, rfile) in files_merge {
                    if let Ok(RFileDecoded::DB(table)) = rfile.decoded() {
                        if let Some(key_col) = table.column_position_by_name("key") {
                            if let Some(col_col) = table.column_position_by_name("unnamed colour group_1") {
                                for row in table.data().iter() {
                                    colors.insert(row[key_col].data_to_string().to_string(), row[col_col].data_to_string().to_string());
                                }
                            }
                        }
                    }
                }
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        // Get the list of tagged images from the dbs.
        let mut tagged_images = HashMap::new();
        let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(vec![ContainerPath::Folder("db/ui_tagged_images_tables".to_owned())], false));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                let mut files_merge = HashMap::new();
                if let Some(files) = files.remove(&DataSource::GameFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::ParentFiles) {
                    files_merge.extend(files);
                }

                if let Some(files) = files.remove(&DataSource::PackFile) {
                    files_merge.extend(files);
                }

                for (_, rfile) in files_merge {
                    if let Ok(RFileDecoded::DB(table)) = rfile.decoded() {
                        if let Some(key_col) = table.column_position_by_name("key") {
                            if let Some(path_col) = table.column_position_by_name("image_path") {
                                for row in table.data().iter() {
                                    tagged_images.insert(row[key_col].data_to_string().to_string(), row[path_col].data_to_string().to_string());
                                }
                            }
                        }
                    }
                }
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        // Check if the repo needs updating, and update it if so.
        let receiver = CENTRAL_COMMAND.send_network(Command::CheckTranslationsUpdates);
        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);
        match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::NoLocalFiles |
                    GitResponse::Diverged => {
                        let receiver = CENTRAL_COMMAND.send_background(Command::UpdateTranslations);
                        let response_thread = CENTRAL_COMMAND.recv_try(&receiver);

                        // Show the error, but continue anyway.
                        if let Response::Error(error) = response_thread {
                            show_dialog(app_ui.main_window(), tre("translation_download_error", &[&error.to_string()]), false);
                        }
                    }
                    GitResponse::NoUpdate => {}
                }
            }

            Response::Error(error) => {
                show_dialog(app_ui.main_window(), tre("translation_download_error", &[&error.to_string()]), false);
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response_thread:?}"),
        }

        // Unlike other tools, data is loaded here, because we need it to generate the table widget.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackTranslation(language));
        let response = CentralCommand::recv(&receiver);
        let data = if let Response::PackTranslation(data) = response { data } else { panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"); };

        let table_data = TableType::TranslatorTable(data.to_table()?);
        let table = TableView::new_view(&table_view_container, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile)))?;

        let layout = tool.main_widget().layout().static_downcast::<QGridLayout>();
        layout.replace_widget_2a(table_view.as_ptr(), table.table_view().as_ptr());
        table_view.delete();

        // The translation list need special configuration.
        table.table_view().set_selection_mode(SelectionMode::SingleSelection);
        table.table_view().set_selection_behavior(SelectionBehavior::SelectRows);
        table.table_view().set_column_width(0, 300);
        table.table_view().set_column_width(1, 50);
        table.table_view().set_column_width(2, 50);
        table.table_view().set_column_width(3, 400);
        table.table_view().set_column_width(4, 400);
        //table.table_view().sort_by_column_1a(0);
        //table.table_view().sort_by_column_1a(1);

        if let Some(filter_removed) = table.filters().first() {
            filter_removed.filter_line_edit().set_text(&QString::from_std_str("false"));
            filter_removed.column_combobox().set_current_index(2);
            filter_removed.use_regex_button().set_checked(false);
        }

        //FilterView::new(&table)?;
        //if let Some(filter_retranslation) = table.filters().get(1) {
        //    filter_retranslation.filter_line_edit().set_text(&QString::from_std_str("true"));
        //    filter_retranslation.column_combobox().set_current_index(1);
        //    filter_retranslation.use_regex_button().set_checked(false);
        //}
        let key_label: QPtr<QLabel> = tool.find_widget("key_label")?;
        let key_line_edit: QPtr<QLineEdit> = tool.find_widget("key_line_edit")?;
        key_label.set_text(&qtr("translator_key"));
        key_line_edit.set_enabled(false);

        let info_groupbox: QPtr<QGroupBox> = tool.find_widget("info_groupbox")?;
        let original_value_groupbox: QPtr<QGroupBox> = tool.find_widget("original_value_groupbox")?;
        let translated_value_groupbox: QPtr<QGroupBox> = tool.find_widget("translated_value_groupbox")?;
        info_groupbox.set_title(&qtr("translator_info_title"));
        original_value_groupbox.set_title(&qtr("translator_original_value_title"));
        translated_value_groupbox.set_title(&qtr("translator_translated_value_title"));

        let info_label: QPtr<QLabel> = tool.find_widget("info_label")?;
        info_label.set_text(&qtr("translator_info"));
        info_label.set_open_external_links(true);

        let move_selection_up: QPtr<QToolButton> = tool.find_widget("move_selection_up")?;
        let move_selection_down: QPtr<QToolButton> = tool.find_widget("move_selection_down")?;
        let translate_with_deepl: QPtr<QToolButton> = tool.find_widget("translate_with_deepl")?;
        let translate_with_chatgpt: QPtr<QToolButton> = tool.find_widget("translate_with_chatgpt")?;
        let translate_with_google: QPtr<QToolButton> = tool.find_widget("translate_with_google")?;
        let copy_from_source: QPtr<QToolButton> = tool.find_widget("copy_from_source")?;
        let import_from_translated_pack: QPtr<QToolButton> = tool.find_widget("import_from_translated_pack")?;
        move_selection_up.set_tool_tip(&qtr("translator_move_selection_up"));
        move_selection_down.set_tool_tip(&qtr("translator_move_selection_down"));
        translate_with_deepl.set_tool_tip(&qtr("translator_translate_with_deepl"));
        translate_with_chatgpt.set_tool_tip(&qtr("translator_translate_with_chatgpt"));
        translate_with_google.set_tool_tip(&qtr("translator_translate_with_google"));
        copy_from_source.set_tool_tip(&qtr("translator_copy_from_source"));
        import_from_translated_pack.set_tool_tip(&qtr("translator_import_from_translated_pack"));

        // Only allow AI translation if we have a key in settings. Ignore keys in env.
        if setting_string("ai_openai_api_key").is_empty() {
            chatgpt_radio_button.set_enabled(false);
            context_text_edit.set_enabled(false);
            translate_with_chatgpt.set_enabled(false);
        } else {
            chatgpt_radio_button.set_checked(true);
        }

        if setting_string("deepl_api_key").is_empty() {
            deepl_radio_button.set_enabled(false);
            translate_with_deepl.set_enabled(false);
        } else {
            deepl_radio_button.set_checked(true);
        }

        let action_move_up = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_up", Some(table.table_view().static_upcast()));
        let action_move_down = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_down", Some(table.table_view().static_upcast()));
        let action_copy_from_source = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "copy_from_source", Some(table.table_view().static_upcast()));
        let action_import_from_translated_pack = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "import_from_translated_pack", Some(table.table_view().static_upcast()));

        let original_value_html: QPtr<QTextEdit> = tool.find_widget("original_value_html")?;
        let original_value_textedit: QPtr<QTextEdit> = tool.find_widget("original_value_textedit")?;
        let translated_value_html: QPtr<QTextEdit> = tool.find_widget("translated_value_html")?;
        let translated_value_textedit: QPtr<QTextEdit> = tool.find_widget("translated_value_textedit")?;
        original_value_html.document().set_default_style_sheet(&QString::from_std_str(CSS_STYLE));
        translated_value_html.document().set_default_style_sheet(&QString::from_std_str(CSS_STYLE));

        // Build the view itself.
        let view = Rc::new(Self {
            tool,
            pack_tr: Arc::new(data),
            table,
            current_key: Arc::new(RwLock::new(None)),
            colors,
            tagged_images,
            language_combobox,
            context_text_edit,
            deepl_radio_button,
            chatgpt_radio_button,
            google_translate_radio_button,
            copy_source_radio_button,
            edit_all_same_values_radio_button,
            key_line_edit,
            action_move_up,
            action_move_down,
            action_copy_from_source,
            action_import_from_translated_pack,
            move_selection_up,
            move_selection_down,
            translate_with_deepl,
            translate_with_chatgpt,
            translate_with_google,
            copy_from_source,
            import_from_translated_pack,
            original_value_html,
            original_value_textedit,
            translated_value_html,
            translated_value_textedit,
        });

        // Build the slots and connect them to the view.
        let slots = ToolTranslatorSlots::new(&view);
        connections::set_connections(&view, &slots);
        view.tool.get_ref_dialog().resize_2a(1800, 800);

        // If we hit ok, save the data back to the Pack.
        if view.tool.get_ref_dialog().exec() == 1 {
            view.save_data(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui)?;
        }

        // If nothing failed, it means we have successfully saved the data back to disk, or canceled.
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
        self.change_selected_row(None, None);

        // Then save both, the updated translations to disk, and the translated locs to the pack.
        let table = get_table_from_view(&self.table().table_model_ptr().static_upcast(), &self.table().table_definition())?;
        let mut pack_tr = (**self.pack_tr()).clone();
        pack_tr.from_table(&table)?;
        pack_tr.set_language(self.language_combobox.current_text().to_std_string());
        pack_tr.save(&translations_local_path()?, GAME_SELECTED.read().unwrap().key())?;

        let mut loc_file = Loc::new();
        let mut loc_data = vec![];
        for (key, tr) in pack_tr.translations() {
            if !*tr.removed() {
                loc_data.push(vec![
                    DecodedData::StringU16(key.to_owned()),
                    DecodedData::StringU16(if !tr.value_translated().is_empty() && !*tr.needs_retranslation() { tr.value_translated().to_owned() } else { tr.value_original().to_owned() }),
                    DecodedData::Boolean(false),
                ]);
            }
        }

        loc_file.set_data(&loc_data)?;
        let loc = RFile::new_from_decoded(&RFileDecoded::Loc(loc_file), 0, TRANSLATED_PATH);

        // TODO: Old games need to overwrite the localisation.loc file instead of using a custom loc file.
        let files_to_save = vec![loc];

        // Once we got the RFiles to save properly edited, call the generic tool `save` function to save them to a Pack.
        self.tool.save(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, &files_to_save)
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: &CppBox<QModelIndex>) {
        let key_item = self.table.table_model().item_from_index(index);
        let original_value_item = self.table.table_model().item_from_index(&index.sibling_at_column(3));
        let translated_value_item = self.table.table_model().item_from_index(&index.sibling_at_column(4));
        let needs_retranslation = self.table.table_model().item_from_index(&index.sibling_at_column(1)).check_state() == CheckState::Checked;

        let mut source_text = original_value_item.text().to_std_string();
        source_text = source_text.replace("||", "\n||\n");
        source_text = source_text.replace("\\\\n", "\n");

        let mut translated_text = translated_value_item.text().to_std_string();
        translated_text = translated_text.replace("||", "\n||\n");
        translated_text = translated_text.replace("\\\\n", "\n");

        self.key_line_edit.set_text(&key_item.text());
        self.original_value_textedit.set_plain_text(&QString::from_std_str(&source_text));
        self.translated_value_textedit.set_plain_text(&QString::from_std_str(&translated_text));

        // If the value needs a retrasnlation decide what to do depending on the behavior group.
        // Only do it if the text is empty. If there's a previous translation, keep it so it can be fixed.
        if needs_retranslation && self.translated_value_textedit().to_plain_text().is_empty() {
            if self.deepl_radio_button().is_checked() {
                let language = self.map_language_to_deepl();
                let result = Self::ask_deepl(&source_text, language);
                if let Ok(tr) = result {
                    self.translated_value_textedit.set_plain_text(&QString::from_std_str(tr));
                }
            } else if self.chatgpt_radio_button().is_checked() {
                let language = self.map_language_to_natural();
                let context = self.context_text_edit().to_plain_text().to_std_string();
                let result = Self::ask_chat_gpt(&source_text, &language, &context);
                if let Ok(tr) = result {
                    self.translated_value_textedit.set_plain_text(&QString::from_std_str(tr));
                }
            } else if self.google_translate_radio_button().is_checked() {
                let language = self.map_language_to_google();
                let result = Self::ask_google(&source_text, &language);
                if let Ok(tr) = result {
                    self.translated_value_textedit.set_plain_text(&QString::from_std_str(tr));
                }
            } else if self.copy_source_radio_button().is_checked() {
                self.translated_value_textedit.set_plain_text(&self.original_value_textedit().to_plain_text());
            }
        }

        // Re-enable this, as it's disabled on changing row.
        self.translated_value_textedit.set_enabled(true);
    }

    // Selection is EXTREMELY unreliable. We save to the current row instead.
    pub unsafe fn save_from_detailed_view(&self, old_key_index: &CppBox<QModelIndex>) {
        let current_row = old_key_index.row();

        let old_value_item = self.table.table_model().item_2a(current_row, 4);
        let old_value = old_value_item.text().to_std_string();
        let mut new_value = self.translated_value_textedit.to_plain_text().to_std_string();

        // If we have a new translation, save it and remove the "needs_retranslation" flag.
        if !new_value.is_empty() && new_value != old_value {

            // If there's any other translation which uses the same value, automatically translate it.
            let original_value_item = self.table.table_model().item_2a(current_row, 3);
            let original_value_item_qstr = original_value_item.data_1a(2).to_string();
            for row in 0..self.table.table_model().row_count_0a() {

                // Do not apply it to the item we just edited.
                if current_row != row {
                    let needs_retranslation_item = self.table.table_model().item_2a(row, 1);
                    let needs_retranslation = needs_retranslation_item.check_state() == CheckState::Checked;
                    if needs_retranslation || self.edit_all_same_values_radio_button().is_checked() {
                        let og_value_item = self.table.table_model().item_2a(row, 3);
                        if og_value_item.data_1a(2).to_string().compare_q_string(&original_value_item_qstr) == 0 {
                            let translated_value_item = self.table.table_model().item_2a(row, 4);

                            new_value = new_value.replace("\n||\n", "||");
                            new_value = new_value.replace("\n", "\\\\n");

                            translated_value_item.set_text(&QString::from_std_str(&new_value));

                            // Unmark it from retranslations.
                            needs_retranslation_item.set_check_state(CheckState::Unchecked);
                        }
                    }
                }
            }

            old_value_item.set_text(&QString::from_std_str(&new_value));
            self.table.table_model().item_2a(current_row, 1).set_check_state(CheckState::Unchecked);
        }
    }

    unsafe fn change_selected_row(&self, new_index: Option<CppBox<QModelIndex>>, sibling_mode: Option<bool>) {
        let is_generic_sel_change = new_index.is_some();
        self.translated_value_textedit().set_enabled(false);

        let event_loop = QEventLoop::new_0a();
        event_loop.process_events_0a();

        // If we have items in the table, try to figure the next one. If we don't have the current one visible,
        // default to the first/last item, depending on the direction we're moving.
        if self.table().table_filter().row_count_0a() > 0 {
            let mut current_index = self.current_key.write().unwrap();
            let new_index = if new_index.is_some() {
                new_index
            } else if let Some(next) = sibling_mode {
                match *current_index {
                    Some(ref index) => {
                        let current_index_filtered = self.table().table_filter().map_from_source(index);
                        if current_index_filtered.is_valid() {
                            let new_row = if next {
                                current_index_filtered.row() + 1
                            } else {
                                current_index_filtered.row() - 1
                            };

                            let new_index_filtered = current_index_filtered.sibling_at_row(new_row);
                            if new_index_filtered.is_valid() {
                                let new_index = self.table().table_filter().map_to_source(&new_index_filtered);
                                Some(new_index)
                            } else {
                                None
                            }
                        } else {
                            let new_index_filtered = if next {
                                self.table().table_filter().index_2a(0, 0)
                            } else {
                                self.table().table_filter().index_2a(self.table().table_filter().row_count_0a() - 1, 0)
                            };

                            let new_index = self.table().table_filter().map_to_source(&new_index_filtered);
                            Some(new_index)
                        }
                    }

                    None => {
                        let new_index_filtered = if next {
                            self.table().table_filter().index_2a(0, 0)
                        } else {
                            self.table().table_filter().index_2a(self.table().table_filter().row_count_0a() - 1, 0)
                        };

                        let new_index = self.table().table_filter().map_to_source(&new_index_filtered);
                        Some(new_index)
                    }
                }
            } else {
                None
            };

            // Handle the selection change.
            match *current_index {
                Some(ref current_index) => self.save_from_detailed_view(current_index),
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
                self.table().table_view().selection_model().block_signals(true);
                let sel_model = self.table().table_view().selection_model();
                sel_model.clear();

                if let Some(ref index) = *current_index {
                    let filter_index = self.table().table_filter().map_from_source(index);
                    if filter_index.is_valid() {
                        let col_count = self.table().table_model().column_count_0a();
                        let end_index = filter_index.sibling_at_column(col_count - 1);
                        let new_selection = QItemSelection::new_2a(&filter_index, &end_index);

                        // This triggers a save of the editing item.
                        sel_model.select_q_item_selection_q_flags_selection_flag(&new_selection, SelectionFlag::Toggle.into());
                    }
                }

                self.table().table_view().selection_model().block_signals(false);
                self.table().table_view().viewport().update();
            }
        }

        // If the table is empty, it means we don't have neither out current item visible, nor the next one.
        // So we just save the current item (if there is one), and clear the view.
        else {
            let mut current_index = self.current_key.write().unwrap();
            match *current_index {
                Some(ref current_index) => self.save_from_detailed_view(current_index),
                None => self.clear_selected_field_data(),
            }

            *current_index = None;
        }

        self.table().filters()[0].start_delayed_updates_timer();
    }

    unsafe fn clear_selected_field_data(&self) {
        self.key_line_edit.clear();
        self.original_value_textedit.clear();
        self.original_value_html.clear();
        self.translated_value_textedit.clear();
        self.translated_value_html.clear();
        self.translated_value_textedit.set_enabled(false);
    }

    unsafe fn map_language_to_google(&self) -> String {
        let lang = self.language_combobox().current_text().to_std_string().to_lowercase();
        match &*lang {
            BRAZILIAN => "pt".to_owned(),
            SIMPLIFIED_CHINESE => "zh".to_owned(),
            CZECH => "cs".to_owned(),
            ENGLISH => "en".to_owned(),
            FRENCH => "fr".to_owned(),
            GERMAN => "de".to_owned(),
            ITALIAN => "it".to_owned(),
            KOREAN => "ko".to_owned(),
            POLISH => "pl".to_owned(),
            RUSSIAN => "ru".to_owned(),
            SPANISH => "es".to_owned(),
            TURKISH => "tr".to_owned(),
            TRADITIONAL_CHINESE => "zh-TW".to_owned(),
            _ => "en".to_owned(),
        }
    }

    unsafe fn map_language_to_natural(&self) -> String {
        let lang = self.language_combobox().current_text().to_std_string().to_lowercase();
        match &*lang {
            BRAZILIAN => "Portuguese".to_owned(),
            SIMPLIFIED_CHINESE => "Simplified Chinese".to_owned(),
            CZECH => "Czech".to_owned(),
            ENGLISH => "English".to_owned(),
            FRENCH => "French".to_owned(),
            GERMAN => "German".to_owned(),
            ITALIAN => "Italian".to_owned(),
            KOREAN => "Korean".to_owned(),
            POLISH => "Polish".to_owned(),
            RUSSIAN => "Russian".to_owned(),
            SPANISH => "Spanish".to_owned(),
            TURKISH => "Turkish".to_owned(),
            TRADITIONAL_CHINESE => "Traditional Chinese".to_owned(),
            _ => "English".to_owned(),
        }
    }

    unsafe fn map_language_to_deepl(&self) -> Lang {
        let lang = self.language_combobox().current_text().to_std_string().to_lowercase();
        match &*lang {
            BRAZILIAN => Lang::PT_BR,
            SIMPLIFIED_CHINESE => Lang::ZH_HANS,
            CZECH => Lang::CS,
            ENGLISH => Lang::EN_GB,
            FRENCH => Lang::FR,
            GERMAN => Lang::DE,
            ITALIAN => Lang::IT,
            KOREAN => Lang::KO,
            POLISH => Lang::PL,
            RUSSIAN => Lang::RU,
            SPANISH => Lang::ES,
            TURKISH => Lang::TR,
            TRADITIONAL_CHINESE => Lang::ZH_HANT,
            _ => Lang::EN_GB,
        }
    }

    #[tokio::main]
    async fn ask_google(string: &str, language: &str) -> Result<String> {
        if !string.trim().is_empty() {
            let string = string
                .replace('\n', "%0A")
                .replace('"', "%22")
                .replace("#", "%23")
                .replace("&", "%26")
                .replace('\'', "%27")
                .replace("<", "%3C")
                .replace(">", "%3E");

            let url = format!("https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={language}&dt=t&q={string}");
            let response = reqwest::get(&url).await?.text().await?;
            let translated_text: String = if let Some(data) = serde_json::from_str::<Value>(&response)?[0].as_array() {
                let mut string = String::new();
                for item in data {
                    string.push_str(item[0].as_str().unwrap());
                }

                string.replace("%20", " ")            // Fix weird spaces.
            } else {
                return Err(anyhow!("Error retrieving google translation.").into());
            };

            Ok(translated_text)
        } else {
            Ok(String::new())
        }
    }

    #[tokio::main]
    async fn ask_chat_gpt(string: &str, language: &str, context: &str) -> Result<String> {

        // Get the API key from the settings. If no API key is provided, it will use the OPENAI_API_KEY env variable.
        let api_key = {
            let key = setting_string("ai_openai_api_key");
            if key.is_empty() {
                None
            } else {
                Some(key)
            }
        };
        let client = OpenAIClient::new(api_key)?;

        // Prepare a request to generate a text completion.
        let mut prompt = format!("Translate the sentence after #### to {language}, keeping the translation as close to the original in tone and style as you can.");
        prompt.push_str(" Preserve the following parts of the text in the translation: any text delimited with '[[' and ']]', '||', jumplines and tabulations. ");
        if !context.is_empty() {
            prompt.push_str(&format!(" For context, use the following info: {context}. #### "));
        }
        prompt.push_str(&string);

        // According to OpenAI's docs, tokens is more or less 3/4 of a word. We don't have a way to easily count words, so we do a generous approximation.
        // Then we duplicate it taking into account the completion tokens.
        let max_tokens = Some((prompt.len() / 4) as u32 * 2u32);
        let request = CreateCompletionRequest {
            model: Model::Gpt3_5TurboInstruct,
            prompt: Some(PromptInput::String(prompt)),
            max_tokens,
            temperature: Some(0.2),
            ..Default::default()
        };

        // Responses sometimes start with jumplines, and we need them clean.
        let response = create_completion(&client, &request).await?;
        let mut response_text = response.choices.first().map(|x| x.text.clone()).unwrap_or_default();
        if response_text.starts_with("\n\n") {
            response_text = response_text[2..].to_owned();
        } else if response_text.starts_with("\n") {
            response_text = response_text[1..].to_owned();
        }

        Ok(response_text)
    }

    #[tokio::main]
    async fn ask_deepl(string: &str, language: Lang) -> Result<String> {
        let api_key = setting_string("deepl_api_key");
        if api_key.is_empty() {
            return Err(anyhow!("Missing DeepL API Key."))
        };

        let api = DeepLApi::with(&api_key).new();

        let string = string
            .replace("[[", "<[[")
            .replace("<[[/", "</[[")
            .replace("]]", "]]>");

        let translated = api.translate_text(string, language)
            .source_lang(Lang::EN)
            .model_type(ModelType::PreferQualityOptimized)
            .ignore_tags(vec![
                "rgba".to_owned(),
                "col".to_owned(),
                "img".to_owned(),
                "url".to_owned(),
                "sl".to_owned(),
                "sl_tooltip".to_owned(),
                "tooltip".to_owned(),
            ])
            .tag_handling(TagHandling::Xml)
            .await?;

        let translated_text = translated.translations.iter()
            .map(|x| &x.text)
            .join("\n")
            .replace("</[[", "<[[/")
            .replace("<[[", "[[")
            .replace("]]>", "]]");

        Ok(translated_text)
    }

    pub unsafe fn import_from_another_pack(&self) -> Result<()> {
        let file_dialog = QFileDialog::from_q_widget_q_string(
            self.tool.main_widget(),
            &qtr("open_packfiles"),
        );
        file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
        file_dialog.set_file_mode(FileMode::ExistingFiles);

        if file_dialog.exec() == 1 {

            let mut paths = vec![];
            for index in 0..file_dialog.selected_files().count_0a() {
                paths.push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
            }

            let mut pack = Pack::read_and_merge(&paths, &GAME_SELECTED.read().unwrap().clone(), true, false, false)?;
            {
                let mut locs = pack.files_by_type_mut(&[FileType::Loc]);
                locs.par_iter_mut().for_each(|file| {
                    let _ = file.decode(&None, true, false);
                });
            }
            let mut locs = pack.files_by_type(&[FileType::Loc]);

            let merged_loc = PackTranslation::sort_and_merge_locs_for_translation(&mut locs)?;
            for data in merged_loc.data().iter() {
                let key = data[0].data_to_string();
                let value = data[1].data_to_string();

                // We check against the original pack_tr because it's faster than just searching on the table.
                if let Some(tr) = self.pack_tr.translations().get(&*key) {
                    if tr.value_original() != &value && tr.value_translated() != &value {
                        for row in 0..self.table().table_model().row_count_0a() {
                            let key_item = self.table().table_model().item_1a(row);
                            if key_item.text().to_std_string() == key {
                                let needs_retranslation_item = self.table().table_model().item_2a(row, 1);
                                let value_translated_item = self.table().table_model().item_2a(row, 4);

                                needs_retranslation_item.set_check_state(CheckState::Unchecked);
                                value_translated_item.set_text(&QString::from_std_str(&value));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Util to format a value into an html string we can use in the translator's UI.
    fn to_html(&self, str: &str) -> String {
        let mut html = str
            .replace("||", "<br/>")
            .replace("\n", "<br/>")
            .replace("\\\\t", "\t");

        // In older games there's no colours table, so we use the colour value directly.
        html = REGEX_COLOR.replace_all(&html, |caps: &Captures| {
            let color = self.colors().get(&caps[1])
                .map(|x| format!("#{x}"))
                .unwrap_or(caps[1].to_string());

            format!("<span style='color:{color};'>{}</span>", &caps[2])
        }).to_string();

        // Trs are translation replacers. We just need to replace the string with the value of the tr key.
        /*html = REGEX_TR.replace_all(&html, |caps: &Captures| {
            let color = self.colors().get(&caps[1])
                .map_or(&caps[1], |v| v);

            format!("<span style='color:#{color};'>{}</span>", &caps[2])
        }).to_string();*/

        html = REGEX_IMG.replace_all(&html, |caps: &Captures| {
            let path = self.tagged_images().get(&caps[1]).cloned().unwrap_or(caps[1].to_string());

            // Get the list of tagged images from the dbs.
            let image_data = STANDARD.encode({
                let mut d = vec![];
                let receiver = CENTRAL_COMMAND.send_background(Command::GetRFilesFromAllSources(vec![ContainerPath::File(path.to_owned())], false));
                let response = CentralCommand::recv(&receiver);
                match response {
                    Response::HashMapDataSourceHashMapStringRFile(mut files) => {
                        let mut files_merge = HashMap::new();
                        if let Some(files) = files.remove(&DataSource::GameFiles) {
                            files_merge.extend(files);
                        }

                        if let Some(files) = files.remove(&DataSource::ParentFiles) {
                            files_merge.extend(files);
                        }

                        if let Some(files) = files.remove(&DataSource::PackFile) {
                            files_merge.extend(files);
                        }

                        for (_, mut rfile) in files_merge {
                            if let Ok(Some(RFileDecoded::Image(data))) = rfile.decode(&None, false, true) {
                                d = data.data().to_vec();
                                break;
                            }
                        }
                    },
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                };

                d
            });

            // NOTE: QTextEdit doesn't seem to be able to resize images.
            format!("<img src='data:image/jpeg;base64,{image_data}'>{}</img>", &caps[2])
        }).to_string();

        // NOTE: Some of these point to help pages made from lua scripts pointing, mainly the ones used for tooltips.
        //
        // We ignore those, as it falls quite out of scope to support them.
        html = REGEX_URL.replace_all(&html, "<a href='$1'>$2 (link: $1)</a>").to_string();
        html = REGEX_SL.replace_all(&html, "<a href='$1'>$2 (link: $1)</a>").to_string();
        html = REGEX_SL_TOOLTIP.replace_all(&html, "<a href='$1'>$2 (link: $1)</a>").to_string();

        // Tooltips don't work in QTextDocument, but we still format it in the usual way, so it looks different.
        html = REGEX_TOOLTIP.replace_all(&html, "<span class='tooltip'>$2<span class='tooltiptext'>$1</span></span>").to_string();

        // Limit alpha to 0.25, because otherwise we get invisible text that's visible in the game.
        html = REGEX_RGBA.replace_all(&html, |caps: &Captures| {
            let limit_alpha = if let Some(val) = caps.get(4) {
                if let Ok(val) = val.as_str().parse::<f32>() {
                    val < 0.25
                } else {
                    false
                }
            } else {
                false
            };

            if limit_alpha {
                format!("<span style='color:rgba({},{},{},0.25);'>{}</span>", &caps[1], &caps[2], &caps[3], &caps[5])
            } else {
                format!("<span style='color:rgba({},{},{},{});'>{}</span>", &caps[1], &caps[2], &caps[3], &caps[4], &caps[5])
            }
        }).to_string();

        html = REGEX_RGB.replace_all(&html, "<span style='color:rgb($1,$2,$3);'>$4</span>").to_string();

        html
    }
}
