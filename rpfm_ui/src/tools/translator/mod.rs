//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QButtonGroup;
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::FileMode;
use qt_widgets::QGroupBox;
use qt_widgets::QInputDialog;
use qt_widgets::QMenu;
use qt_widgets::QMessageBox;
use qt_widgets::QProgressDialog;
use qt_widgets::QPushButton;
use qt_widgets::q_message_box::{Icon, StandardButton};
use qt_widgets::QRadioButton;
use qt_widgets::QToolButton;
use qt_widgets::q_abstract_item_view::{SelectionBehavior, SelectionMode};
use qt_widgets::q_tool_button::ToolButtonPopupMode;
use qt_widgets::QGridLayout;
use qt_widgets::QTableView;

use qt_gui::QAction;

use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QEventLoop;
use qt_core::QFlags;
use qt_core::QItemSelection;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QListOfQString;
use qt_core::QPtr;
use qt_core::QSignalBlocker;
use qt_core::QString;
use qt_core::WindowModality;

use cpp_core::CastInto;
use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::Ptr;

use anyhow::anyhow;
use base64::{Engine, engine::general_purpose::STANDARD};
use deepl::{DeepLApi, Error as DeepLError, Lang, ModelType, TagHandling};
use regex::{Captures, Regex};
use serde_json::{json, Value};

use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::{Duration, Instant};

use rpfm_extensions::translator::*;

use rpfm_ipc::settings_keys::*;

use rpfm_lib::files::{Container, ContainerPath, FileType, pack::Pack, RFileDecoded, table::DecodedData};
use rpfm_lib::games::{*, supported_games::*};
use rpfm_lib::integrations::git::GitResponse;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR, send_ipc_command, send_ipc_command_result};
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::{settings_path_buf, settings_set_string, settings_string, translations_local_path};
use crate::views::table::{FilterChipState, TableType, TableView, utils::get_table_from_view};
use crate::utils::show_dialog;

use self::slots::ToolTranslatorSlots;
use super::*;

mod connections;
mod slots;
#[cfg(test)] mod test;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_translator_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_translator_editor.ui";

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

// `LazyLock<Regex>` in a `static` so the compiled regex is shared across calls. Declaring
// these as `const LazyCell<_>` would re-run `Regex::new` every time the constant was named.
static REGEX_COLOR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[col:(.*?)]](.*?)\[\[/col(.*?)]]").unwrap());
static REGEX_RGBA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[rgba:(\d+?):(\d+?):(\d+?):(\d+?)]](.*?)\[\[/rgba(.*?)]]").unwrap());
static REGEX_RGB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[rgba:(\d+?):(\d+?):(\d+?)]](.*?)\[\[/rgba(.*?)]]").unwrap());
static REGEX_IMG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[img:(.+?)]](.*?)\[\[/img(.*?)]]").unwrap());
//static REGEX_TR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{tr:(.+?)}}").unwrap());

// These are all kind of internal links for different types of interactions.
static REGEX_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[url:(.+?)]](.*?)\[\[/url(.*?)]]").unwrap());
static REGEX_SL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[sl:(.+?)]](.*?)\[\[/sl(.*?)]]").unwrap());
static REGEX_SL_TOOLTIP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[sl_tooltip:(.+?)]](.*?)\[\[/sl_tooltip(.*?)]]").unwrap());
static REGEX_TOOLTIP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[tooltip:(.+?)]](.*?)\[\[/tooltip(.*?)]]").unwrap());

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

/// Minimum time between two requests of a batch auto-translation.
const BATCH_MIN_INTERVAL: Duration = Duration::from_millis(250);

/// Maximum time between two requests of a batch auto-translation, once the service starts rate-limiting us.
const BATCH_MAX_INTERVAL: Duration = Duration::from_secs(5);

/// Times a rate-limited request is retried before the line is reported as failed.
const BATCH_MAX_RETRIES: u32 = 5;

/// Maximum amount of texts sent to DeepL in a single batch request.
const DEEPL_GROUP_MAX_TEXTS: usize = 50;

/// Maximum size in bytes of the texts sent to DeepL in a single batch request. DeepL's limit is
/// 128 KiB for the whole request, so this leaves room for the escaping and the rest of the request.
const DEEPL_GROUP_MAX_BYTES: usize = 64 * 1024;

/// Error for requests the translation service rejected for going over its rate limit.
#[derive(Debug, thiserror::Error)]
#[error("The translation service is rate-limiting requests: {0}")]
struct RateLimitedError(String);

/// Backend selector for the batch auto-translation menu.
#[derive(Debug, Clone, Copy)]
pub enum BatchTranslateMethod {
    Deepl,
    Ai,
    Google,
}

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ToolTranslator {
    tool: Tool,
    pack_tr: Arc<PackTranslation>,
    table: Arc<TableView>,

    // Item with the key being edited.
    current_key: Arc<RwLock<Option<CppBox<QModelIndex>>>>,

    // Text auto-translation pushed into the textedit by load_to_detailed_view, when applicable.
    // Used so save_from_detailed_view can mark the row as `aut` when the user accepted the
    // auto-translation untouched, and clear `aut` when the user edited it.
    current_auto_translation: Arc<RwLock<Option<String>>>,

    colors: HashMap<String, String>,
    tagged_images: HashMap<String, String>,
    language_combobox: QPtr<QComboBox>,
    version_combobox: QPtr<QComboBox>,

    // Source language is picked from the pre-dialog before the translator opens, so inside the
    // translator it lives as a read-only label rather than an editable selector.
    source_language_value: QPtr<QLabel>,

    // Widgets that we hide by default to keep the dialog compact. The toggle buttons next to
    // them flip their visibility so the controls are still discoverable.
    info_label: QPtr<QLabel>,
    behavior_groupbox: QPtr<QGroupBox>,
    help_toggle: QPtr<QPushButton>,
    preview_toggle: QPtr<QPushButton>,
    behavior_toggle: QPtr<QPushButton>,

    deepl_radio_button: QPtr<QRadioButton>,
    ai_radio_button: QPtr<QRadioButton>,
    google_translate_radio_button: QPtr<QRadioButton>,
    copy_source_radio_button: QPtr<QRadioButton>,

    context_text_edit: QPtr<QTextEdit>,
    edit_all_same_values_radio_button: QPtr<QRadioButton>,

    key_line_edit: QPtr<QLineEdit>,

    action_move_up: QPtr<QAction>,
    action_move_down: QPtr<QAction>,
    action_copy_from_source: QPtr<QAction>,
    action_clear_translation: QPtr<QAction>,
    action_import_from_translated_pack: QPtr<QAction>,

    move_selection_up: QPtr<QToolButton>,
    move_selection_down: QPtr<QToolButton>,
    translate_with_deepl: QPtr<QToolButton>,
    translate_with_ai: QPtr<QToolButton>,
    translate_with_google: QPtr<QToolButton>,
    copy_from_source: QPtr<QToolButton>,
    clear_translation: QPtr<QToolButton>,
    import_from_translated_pack: QPtr<QToolButton>,

    // The batch translate button shows a popup menu. We keep the menu alive on the struct so
    // it isn't dropped while still attached to the button.
    #[allow(dead_code)] batch_translate_menu: QBox<QMenu>,
    batch_translate_deepl_action: QPtr<QAction>,
    batch_translate_ai_action: QPtr<QAction>,
    batch_translate_google_action: QPtr<QAction>,
    batch_translate_overwrite_action: QPtr<QAction>,

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
        let tool = Tool::new(app_ui.main_window(), &paths, &TOOL_SUPPORTED_GAMES, view, false)?;
        tool.set_title(&tr("translator_title"));
        tool.backup_used_paths(app_ui, pack_file_contents_ui)?;

        // Translations list.
        let table_view: QPtr<QTableView> = tool.find_widget("table_view")?;
        let table_view_container: QPtr<QWidget> = tool.find_widget("table_view_container")?;
        let table_view_container = table_view_container.into_q_box();

        let language_label: QPtr<QLabel> = tool.find_widget("language_label")?;
        let language_combobox: QPtr<QComboBox> = tool.find_widget("language_combobox")?;
        let source_language_label: QPtr<QLabel> = tool.find_widget("source_language_label")?;
        let source_language_value: QPtr<QLabel> = tool.find_widget("source_language_value")?;
        language_label.set_text(&qtr("translator_language"));
        source_language_label.set_text(&qtr("translator_source_language"));

        let version_label: QPtr<QLabel> = tool.find_widget("version_label")?;
        let version_combobox: QPtr<QComboBox> = tool.find_widget("version_combobox")?;
        version_label.set_text(&qtr("translator_version"));
        version_combobox.set_tool_tip(&qtr("translator_version_tooltip"));
        // Populated in display order: index 0 = v0 (legacy), index 1 = v1 (current). We use the
        // combobox index as the version number, so keep these in sync with the file format.
        version_combobox.add_item_q_string(&qtr("translator_version_0"));
        version_combobox.add_item_q_string(&qtr("translator_version_1"));

        let behavior_groupbox: QPtr<QGroupBox> = tool.find_widget("behavior_groupbox")?;
        let behavior_label: QPtr<QLabel> = tool.find_widget("behavior_label")?;
        let context_label: QPtr<QLabel> = tool.find_widget("context_label")?;
        let context_text_edit: QPtr<QTextEdit> = tool.find_widget("context_text_edit")?;
        let deepl_radio_button: QPtr<QRadioButton> = tool.find_widget("deepl_radio")?;
        let ai_radio_button: QPtr<QRadioButton> = tool.find_widget("ai_radio")?;
        let google_translate_radio_button: QPtr<QRadioButton> = tool.find_widget("google_translate_radio")?;
        let copy_source_radio_button: QPtr<QRadioButton> = tool.find_widget("copy_source_radio")?;
        let empty_radio_button: QPtr<QRadioButton> = tool.find_widget("empty_radio")?;
        context_label.set_text(&qtr("context"));
        behavior_groupbox.set_title(&qtr("behavior_title"));
        behavior_label.set_text(&qtr("behavior_info"));
        deepl_radio_button.set_text(&qtr("behavior_deepl"));
        ai_radio_button.set_text(&qtr("behavior_ai"));
        google_translate_radio_button.set_text(&qtr("behavior_google_translate"));
        copy_source_radio_button.set_text(&qtr("behavior_copy_source"));
        empty_radio_button.set_text(&qtr("behavior_empty"));

        let behavior_group = QButtonGroup::new_1a(&behavior_groupbox);
        behavior_group.add_button_1a(&deepl_radio_button);
        behavior_group.add_button_1a(&ai_radio_button);
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
        let game_path = settings_path_buf(game.key());

        // Target candidates come from the game's installed locale packs. EN is always included as a
        // fallback because most modders write in English and the Translation Hub's vanilla data is keyed on EN.
        let mut candidates = vec!["EN".to_owned()];
        if let Ok(ca_packs) = game.ca_packs_paths(&game_path) {
            for code in ca_packs.iter()
                .filter_map(|path| path.file_stem())
                .filter(|name| name.to_string_lossy().starts_with("local_"))
                .map(|name| name.to_string_lossy().split_at(6).1.to_uppercase())
            {
                if code.chars().count() == 2 && !candidates.contains(&code) {
                    candidates.push(code);
                }
            }
        }
        candidates.sort();

        // Source candidates include languages not installed, as their vanilla texts may have been generated earlier.
        let mut src_candidates = [BRAZILIAN, SIMPLIFIED_CHINESE, CZECH, ENGLISH, FRENCH, GERMAN, ITALIAN, KOREAN, POLISH, RUSSIAN, SPANISH, TURKISH, TRADITIONAL_CHINESE]
            .iter()
            .map(|code| code.to_uppercase())
            .collect::<Vec<_>>();
        src_candidates.sort();

        // Source language pre-dialog. The full translator dialog isn't visible yet (we haven't
        // called exec() on it), so this small picker shows over the main window. Cancelling
        // aborts the tool — the user can re-open it to pick again.
        let src_items = QListOfQString::new_0a();
        for code in &src_candidates {
            src_items.append_q_string(&QString::from_std_str(code));
        }
        let last_src_lang = settings_string(TRANSLATOR_SOURCE_LANGUAGE);
        let default_src_idx = src_candidates.iter()
            .position(|c| c.eq_ignore_ascii_case(&last_src_lang))
            .or_else(|| src_candidates.iter().position(|c| c == DEFAULT_SRC_LANG))
            .unwrap_or(0) as i32;
        let mut src_ok = false;
        let src_chosen = QInputDialog::get_item_7a(
            app_ui.main_window(),
            &qtr("translator_source_language_dialog_title"),
            &qtr("translator_source_language_dialog_label"),
            &src_items,
            default_src_idx,
            false,
            &mut src_ok as *mut bool,
        );
        if !src_ok {
            return Ok(());
        }
        let src_lang = src_chosen.to_std_string();
        source_language_value.set_text(&QString::from_std_str(&src_lang));
        let _ = settings_set_string(TRANSLATOR_SOURCE_LANGUAGE, &src_lang);

        // The Translation Hub only has English vanilla texts, so other languages have to come from the game files.
        let is_default_src_lang = src_lang.eq_ignore_ascii_case(DEFAULT_SRC_LANG);
        if !is_default_src_lang {
            let available = send_ipc_command_result(Command::GenerateVanillaTranslationSource(src_lang.clone()), response_extractor!(Response::Bool))?;
            if !available {
                show_message(app_ui.main_window(), Icon::Warning, &qtr("translator_vanilla_source_title"), &qtre("translator_vanilla_source_missing", &[&src_lang, &src_lang, &src_lang]));
            }
        }

        // Target language: use the game's configured locale when present, otherwise let the
        // user pick from the same candidate list we built above.
        let locale = game.game_locale_from_file(&game_path)?;
        let language = match locale {
            Some(locale) => {
                let language = locale.to_uppercase();
                language_combobox.insert_item_int_q_string(0, &QString::from_std_str(&language));
                language_combobox.set_current_index(0);
                language
            },
            None => {
                // Drop the locked-in source so the target list doesn't offer translating to itself.
                let mut targets: Vec<_> = candidates.iter().filter(|c| **c != src_lang).cloned().collect();
                if targets.is_empty() {
                    return Err(anyhow!("The translator couldn't figure out what languages you have for the game."));
                }
                targets.sort();
                for (index, lang) in targets.iter().enumerate() {
                    language_combobox.insert_item_int_q_string(index as i32, &QString::from_std_str(lang));
                }
                if targets.len() > 1 {
                    language_combobox.set_enabled(true);
                }
                language_combobox.set_current_index(0);
                targets[0].to_owned()
            }
        };

        // A game running in the source language means the user switched it just to get its vanilla texts, which are now saved.
        if !is_default_src_lang && language.eq_ignore_ascii_case(&src_lang) {
            show_message(app_ui.main_window(), Icon::Information, &qtr("translator_vanilla_source_title"), &qtre("translator_vanilla_source_generated", &[&src_lang]));
            return Ok(());
        }

        // Get the list of colours supported by the game. They're in the ui_colours table in the modern games.
        let mut colors = HashMap::new();
        let mut files = send_ipc_command(Command::GetRFilesFromAllSources(vec![ContainerPath::Folder("db/ui_colours_tables".to_owned())], false), response_extractor!(Response::HashMapDataSourceHashMapStringRFile));
        {
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
        }

        // Get the list of tagged images from the dbs.
        let mut tagged_images = HashMap::new();
        let mut files = send_ipc_command(Command::GetRFilesFromAllSources(vec![ContainerPath::Folder("db/ui_tagged_images_tables".to_owned())], false), response_extractor!(Response::HashMapDataSourceHashMapStringRFile));
        {
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
        }

        // Check if the repo needs updating, and update it if so.
        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::CheckTranslationsUpdates);
        let response_thread = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
        match response_thread {
            Response::APIResponseGit(ref response) => {
                match response {
                    GitResponse::NewUpdate |
                    GitResponse::NoLocalFiles |
                    GitResponse::Diverged => {
                        let receiver = CENTRAL_COMMAND.read().unwrap().send(Command::UpdateTranslations);
                        let response_thread = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);

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
        let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
        let data = send_ipc_command_result(Command::GetPackTranslation(pack_key.clone(), src_lang.clone(), language), response_extractor!(Response::PackTranslation))?;

        let table_data = TableType::TranslatorTable(data.to_table()?);
        let table = TableView::new_view(&table_view_container, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile)), Arc::new(RwLock::new(pack_key)))?;

        let layout = tool.main_widget().layout().static_downcast::<QGridLayout>();
        layout.replace_widget_2a(table_view.as_ptr(), table.table_view().as_ptr());
        table_view.delete();

        // The translation list need special configuration.
        table.table_view().set_selection_mode(SelectionMode::SingleSelection);
        table.table_view().set_selection_behavior(SelectionBehavior::SelectRows);
        table.table_view().set_column_width(0, 300);
        table.table_view().set_column_width(1, 50);
        table.table_view().set_column_width(2, 50);
        table.table_view().set_column_width(3, 50);
        table.table_view().set_column_width(4, 400);
        table.table_view().set_column_width(5, 400);
        //table.table_view().sort_by_column_1a(0);
        //table.table_view().sort_by_column_1a(1);

        // Pre-seed the chip bar with a "show only untranslated rows" filter (column 2 = false).
        if let Some(bar) = table.filter_bar_arc() {
            let state = FilterChipState {
                column_index: 2,
                pattern: "false".to_string(),
                regex: false,
                ..FilterChipState::default()
            };
            let _ = bar.add_chip(&table, state, false);
            table.filter_table();
        }
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

        let help_toggle: QPtr<QPushButton> = tool.find_widget("help_toggle")?;
        let preview_toggle: QPtr<QPushButton> = tool.find_widget("preview_toggle")?;
        let behavior_toggle: QPtr<QPushButton> = tool.find_widget("behavior_toggle")?;
        help_toggle.set_text(&qtr("translator_help_toggle"));
        help_toggle.set_tool_tip(&qtr("translator_help_toggle_tooltip"));
        preview_toggle.set_text(&qtr("translator_preview_toggle"));
        preview_toggle.set_tool_tip(&qtr("translator_preview_toggle_tooltip"));
        behavior_toggle.set_text(&qtr("translator_behavior_toggle"));
        behavior_toggle.set_tool_tip(&qtr("translator_behavior_toggle_tooltip"));

        let move_selection_up: QPtr<QToolButton> = tool.find_widget("move_selection_up")?;
        let move_selection_down: QPtr<QToolButton> = tool.find_widget("move_selection_down")?;
        let translate_with_deepl: QPtr<QToolButton> = tool.find_widget("translate_with_deepl")?;
        let translate_with_ai: QPtr<QToolButton> = tool.find_widget("translate_with_ai")?;
        let translate_with_google: QPtr<QToolButton> = tool.find_widget("translate_with_google")?;
        let copy_from_source: QPtr<QToolButton> = tool.find_widget("copy_from_source")?;
        let clear_translation: QPtr<QToolButton> = tool.find_widget("clear_translation")?;
        let import_from_translated_pack: QPtr<QToolButton> = tool.find_widget("import_from_translated_pack")?;
        let batch_translate: QPtr<QToolButton> = tool.find_widget("batch_translate")?;
        move_selection_up.set_tool_tip(&qtr("translator_move_selection_up"));
        move_selection_down.set_tool_tip(&qtr("translator_move_selection_down"));
        translate_with_deepl.set_tool_tip(&qtr("translator_translate_with_deepl"));
        translate_with_ai.set_tool_tip(&qtr("translator_translate_with_ai"));
        translate_with_google.set_tool_tip(&qtr("translator_translate_with_google"));
        copy_from_source.set_tool_tip(&qtr("translator_copy_from_source"));
        clear_translation.set_tool_tip(&qtr("translator_clear_translation"));
        import_from_translated_pack.set_tool_tip(&qtr("translator_import_from_translated_pack"));
        batch_translate.set_tool_tip(&qtr("translator_batch_translate"));
        batch_translate.set_popup_mode(ToolButtonPopupMode::InstantPopup);

        let batch_translate_menu = QMenu::from_q_widget(&batch_translate);
        let batch_translate_deepl_action = batch_translate_menu.add_action_q_string(&qtr("translator_batch_translate_deepl"));
        let batch_translate_ai_action = batch_translate_menu.add_action_q_string(&qtr("translator_batch_translate_ai"));
        let batch_translate_google_action = batch_translate_menu.add_action_q_string(&qtr("translator_batch_translate_google"));
        batch_translate_menu.add_separator();
        let batch_translate_overwrite_action = batch_translate_menu.add_action_q_string(&qtr("translator_batch_translate_overwrite"));
        batch_translate_overwrite_action.set_checkable(true);
        batch_translate.set_menu(&batch_translate_menu);

        // Only allow AI translation if we have both a key and an endpoint URL configured.
        // The provider can be anything that speaks the OpenAI chat-completions wire format.
        if settings_string(AI_API_KEY).is_empty() || settings_string(AI_API_URL).is_empty() || settings_string(AI_MODEL).is_empty() {
            ai_radio_button.set_enabled(false);
            context_text_edit.set_enabled(false);
            translate_with_ai.set_enabled(false);
            batch_translate_ai_action.set_enabled(false);
        } else {
            ai_radio_button.set_checked(true);
        }

        if settings_string(DEEPL_API_KEY).is_empty() {
            deepl_radio_button.set_enabled(false);
            translate_with_deepl.set_enabled(false);
            batch_translate_deepl_action.set_enabled(false);
        } else {
            deepl_radio_button.set_checked(true);
        }

        let action_move_up = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_up", Some(table.table_view().static_upcast()));
        let action_move_down = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_down", Some(table.table_view().static_upcast()));
        let action_copy_from_source = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "copy_from_source", Some(table.table_view().static_upcast()));
        let action_clear_translation = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "clear_translation", Some(table.table_view().static_upcast()));
        let action_import_from_translated_pack = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "import_from_translated_pack", Some(table.table_view().static_upcast()));

        let original_value_html: QPtr<QTextEdit> = tool.find_widget("original_value_html")?;
        let original_value_textedit: QPtr<QTextEdit> = tool.find_widget("original_value_textedit")?;
        let translated_value_html: QPtr<QTextEdit> = tool.find_widget("translated_value_html")?;
        let translated_value_textedit: QPtr<QTextEdit> = tool.find_widget("translated_value_textedit")?;
        original_value_html.document().set_default_style_sheet(&QString::from_std_str(CSS_STYLE));
        translated_value_html.document().set_default_style_sheet(&QString::from_std_str(CSS_STYLE));

        // Collapse the help, the auto-translation settings and the previews by default, so the initial view
        // is just the metadata, the source and the translation editor. Each one has a toggle in the info strip.
        info_label.set_visible(false);
        behavior_groupbox.set_visible(false);
        original_value_html.set_visible(false);
        translated_value_html.set_visible(false);

        // Select the version that matches what's stored on disk. Items 0 and 1 line up with the
        // version numbers; anything else (unexpected) falls back to the current v1 entry.
        let version_index = (*data.version()).min(1) as i32;
        version_combobox.set_current_index(version_index);

        // v0 has no source language, so it's only valid for EN-sourced translations.
        if !is_default_src_lang {
            version_combobox.set_current_index(CURRENT_VERSION as i32);
            version_combobox.set_enabled(false);
        }

        // Build the view itself.
        let view = Rc::new(Self {
            tool,
            pack_tr: Arc::new(data),
            table,
            current_key: Arc::new(RwLock::new(None)),
            current_auto_translation: Arc::new(RwLock::new(None)),
            colors,
            tagged_images,
            language_combobox,
            version_combobox,
            source_language_value,
            info_label,
            behavior_groupbox,
            help_toggle,
            preview_toggle,
            behavior_toggle,
            context_text_edit,
            deepl_radio_button,
            ai_radio_button,
            google_translate_radio_button,
            copy_source_radio_button,
            edit_all_same_values_radio_button,
            key_line_edit,
            action_move_up,
            action_move_down,
            action_copy_from_source,
            action_clear_translation,
            action_import_from_translated_pack,
            move_selection_up,
            move_selection_down,
            translate_with_deepl,
            translate_with_ai,
            translate_with_google,
            copy_from_source,
            clear_translation,
            import_from_translated_pack,
            batch_translate_menu,
            batch_translate_deepl_action,
            batch_translate_ai_action,
            batch_translate_google_action,
            batch_translate_overwrite_action,
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
            view.save_data(app_ui, pack_file_contents_ui, global_search_ui, dependencies_ui)?;
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
        dependencies_ui: &Rc<DependenciesUI>
    ) -> Result<()> {

        // First, save whatever is currently open in the detailed view.
        self.change_selected_row(None, None);

        // Then save both, the updated translations to disk, and the translated locs to the pack.
        let mut pack_tr = self.snapshot_pack_translation()?;
        pack_tr.save(&translations_local_path()?, GAME_SELECTED.read().unwrap().key())?;

        let mut loc_file = Loc::new();
        let mut loc_data = vec![];
        for (key, tr) in pack_tr.translations() {
            if !*tr.rem() {
                loc_data.push(vec![
                    DecodedData::StringU16(key.to_owned()),
                    DecodedData::StringU16(if !tr.dst().is_empty() && !*tr.retr() { tr.dst().to_owned() } else { tr.src().to_owned() }),
                    DecodedData::Boolean(false),
                ]);
            }
        }

        loc_file.set_data(&loc_data)?;
        let loc = RFile::new_from_decoded(&RFileDecoded::Loc(loc_file), 0, TRANSLATED_PATH);

        // TODO: Old games need to overwrite the localisation.loc file instead of using a custom loc file.
        let files_to_save = vec![loc];

        // Once we got the RFiles to save properly edited, call the generic tool `save` function to save them to a Pack.
        self.tool.save(app_ui, pack_file_contents_ui, global_search_ui, dependencies_ui, &files_to_save)
    }

    /// Build a fresh [`PackTranslation`] from the live UI state.
    ///
    /// Callers should call [`change_selected_row`](Self::change_selected_row) first to flush
    /// the per-row editor's in-flight value into the model — this method only reads the model.
    unsafe fn snapshot_pack_translation(&self) -> Result<PackTranslation> {
        let table = get_table_from_view(&self.table().table_model_ptr().static_upcast(), &self.table().table_definition())?;
        let mut pack_tr = (**self.pack_tr()).clone();
        pack_tr.from_table(&table)?;
        pack_tr.set_language(self.language_combobox.current_text().to_std_string());

        // Combobox index maps directly to the file format version (0 / 1). Anything negative
        // would mean nothing is selected, which shouldn't be possible — fall back to v1 then.
        let version = self.version_combobox.current_index().max(0) as u32;
        pack_tr.set_version(version);

        Ok(pack_tr)
    }

    /// This function loads the data of a faction into the detailed view.
    pub unsafe fn load_to_detailed_view(&self, index: &CppBox<QModelIndex>) {
        let key_item = self.table.table_model().item_from_index(index);
        let original_value_item = self.table.table_model().item_from_index(&index.sibling_at_column(4));
        let translated_value_item = self.table.table_model().item_from_index(&index.sibling_at_column(5));
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

        // Start with no pending auto-translation; the branches below set it if they fire.
        *self.current_auto_translation.write().unwrap() = None;

        // If the value needs a retrasnlation decide what to do depending on the behavior group.
        // Only do it if the text is empty. If there's a previous translation, keep it so it can be fixed.
        if needs_retranslation && self.translated_value_textedit().to_plain_text().is_empty() {
            let auto_result = if self.deepl_radio_button().is_checked() {
                let source_language = self.map_source_language_to_deepl();
                let language = self.map_language_to_deepl();
                Self::ask_deepl(&source_text, source_language, language).ok()
            } else if self.ai_radio_button().is_checked() {
                let language = self.map_language_to_natural();
                let context = self.context_text_edit().to_plain_text().to_std_string();
                Self::ask_ai(&source_text, &language, &context).ok()
            } else if self.google_translate_radio_button().is_checked() {
                let language = self.map_language_to_google();
                Self::ask_google(&source_text, &language).ok()
            } else if self.copy_source_radio_button().is_checked() {
                Some(self.original_value_textedit().to_plain_text().to_std_string())
            } else {
                None
            };

            if let Some(tr) = auto_result {
                self.translated_value_textedit.set_plain_text(&QString::from_std_str(&tr));
                // Track the auto-filled text so save_from_detailed_view can mark the row as
                // `aut` when the user accepts it untouched, and clear `aut` when the user edits it.
                *self.current_auto_translation.write().unwrap() = Some(tr);
            }
        }

        // Re-enable this, as it's disabled on changing row.
        self.translated_value_textedit.set_enabled(true);
    }

    // Selection is EXTREMELY unreliable. We save to the current row instead.
    pub unsafe fn save_from_detailed_view(&self, old_key_index: &CppBox<QModelIndex>) {
        let current_row = old_key_index.row();

        let old_value_item = self.table.table_model().item_2a(current_row, 5);
        let old_value = old_value_item.text().to_std_string();
        let mut new_value = self.translated_value_textedit.to_plain_text().to_std_string();

        // If the textedit value still matches the auto-translation pushed by load_to_detailed_view,
        // the user accepted it without touching it → flag the row as `aut`. If the user edited it,
        // the values won't match → clear `aut`.
        let from_auto = self.current_auto_translation.read().unwrap()
            .as_ref()
            .is_some_and(|auto| auto == &new_value);

        // If we have a new translation, save it and update the retr/aut flags accordingly.
        if !new_value.is_empty() && new_value != old_value {
            new_value = new_value.replace("\n||\n", "||");
            new_value = new_value.replace("\n", "\\\\n");

            let aut_state = if from_auto { CheckState::Checked } else { CheckState::Unchecked };

            // If there's any other translation which uses the same value, automatically translate it.
            let original_value_item = self.table.table_model().item_2a(current_row, 4);
            let original_value_item_qstr = original_value_item.data_1a(2).to_string();
            for row in 0..self.table.table_model().row_count_0a() {

                // Do not apply it to the item we just edited.
                if current_row != row {
                    let needs_retranslation_item = self.table.table_model().item_2a(row, 1);
                    let needs_retranslation = needs_retranslation_item.check_state() == CheckState::Checked;
                    if needs_retranslation || self.edit_all_same_values_radio_button().is_checked() {
                        let og_value_item = self.table.table_model().item_2a(row, 4);
                        if og_value_item.data_1a(2).to_string().compare_q_string(&original_value_item_qstr) == 0 {
                            let translated_value_item = self.table.table_model().item_2a(row, 5);
                            translated_value_item.set_text(&QString::from_std_str(&new_value));

                            // Propagated edits inherit the same aut state as the edited row.
                            needs_retranslation_item.set_check_state(CheckState::Unchecked);
                            self.table.table_model().item_2a(row, 3).set_check_state(aut_state);
                        }
                    }
                }
            }

            old_value_item.set_text(&QString::from_std_str(&new_value));
            self.table.table_model().item_2a(current_row, 1).set_check_state(CheckState::Unchecked);
            self.table.table_model().item_2a(current_row, 3).set_check_state(aut_state);
        }

        // Clear the tracker now that we've consumed it for this row.
        *self.current_auto_translation.write().unwrap() = None;
    }

    unsafe fn change_selected_row(&self, new_index: Option<CppBox<QModelIndex>>, sibling_mode: Option<bool>) {
        let is_generic_sel_change = new_index.is_some();
        self.translated_value_textedit().set_enabled(false);

        let event_loop = QEventLoop::new_0a();
        event_loop.process_events();

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

        self.table().filter_table();
    }

    /// Reset the currently-selected row back to an "untranslated" state: empty `dst`,
    /// `retr` checked, `aut` cleared. We write straight to the model rather than going
    /// through `save_from_detailed_view`, which short-circuits on empty values.
    pub unsafe fn clear_selected_translation(&self) {
        let current_index = self.current_key.read().unwrap().as_ref().map(|i| i.row());
        let Some(current_row) = current_index else { return };

        let translated_value_item = self.table.table_model().item_2a(current_row, 5);
        translated_value_item.set_text(&QString::new());

        let needs_retranslation_item = self.table.table_model().item_2a(current_row, 1);
        needs_retranslation_item.set_check_state(CheckState::Checked);

        let auto_translated_item = self.table.table_model().item_2a(current_row, 3);
        auto_translated_item.set_check_state(CheckState::Unchecked);

        // Mirror the change in the detail pane and forget any pending auto-translation
        // tracker so the next save doesn't re-flag the cleared row as `aut`.
        self.translated_value_textedit.clear();
        self.translated_value_html.clear();
        *self.current_auto_translation.write().unwrap() = None;
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
        Self::game_language_to_deepl(&self.language_combobox().current_text().to_std_string())
    }

    /// DeepL only accepts base language codes as source, so regional variants are collapsed.
    unsafe fn map_source_language_to_deepl(&self) -> Lang {
        match Self::game_language_to_deepl(&self.source_language_value().text().to_std_string()) {
            Lang::PT_BR => Lang::PT,
            Lang::ZH_HANS | Lang::ZH_HANT => Lang::ZH,
            Lang::EN_GB => Lang::EN,
            lang => lang,
        }
    }

    fn game_language_to_deepl(lang: &str) -> Lang {
        match &*lang.to_lowercase() {
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
            let response = reqwest::get(&url).await?;
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(anyhow!(RateLimitedError(response.text().await.unwrap_or_default())));
            }

            let response = response.text().await?;
            let translated_text: String = if let Some(data) = serde_json::from_str::<Value>(&response)?[0].as_array() {
                let mut string = String::new();
                for item in data {
                    string.push_str(item[0].as_str().unwrap());
                }

                string.replace("%20", " ")            // Fix weird spaces.
            } else {
                return Err(anyhow!("Error retrieving google translation."));
            };

            Ok(translated_text)
        } else {
            Ok(String::new())
        }
    }

    /// Translate `string` into `language` through any AI provider that exposes the OpenAI
    /// chat-completions wire format.
    ///
    /// The endpoint URL, API key, and model are read from the user's settings, so the same code
    /// path works against OpenAI, Anthropic's OpenAI-compat endpoint, Gemini's
    /// `/v1beta/openai/`, OpenRouter, Ollama, vLLM, LM Studio, etc.
    #[tokio::main]
    async fn ask_ai(string: &str, language: &str, context: &str) -> Result<String> {
        let api_url = settings_string(AI_API_URL);
        let api_key = settings_string(AI_API_KEY);
        let model = settings_string(AI_MODEL);

        if api_url.is_empty() {
            return Err(anyhow!("Missing AI API URL. Set it in Preferences > AI Settings."));
        }
        if api_key.is_empty() {
            return Err(anyhow!("Missing AI API Key. Set it in Preferences > AI Settings."));
        }
        if model.is_empty() {
            return Err(anyhow!("Missing AI model name. Set it in Preferences > AI Settings."));
        }

        let mut prompt = format!("Translate the sentence after #### to {language}, keeping the translation as close to the original in tone and style as you can.");
        prompt.push_str(" Preserve the following parts of the text in the translation: any text delimited with '[[' and ']]', '||', jumplines and tabulations. ");
        if !context.is_empty() {
            prompt.push_str(&format!(" For context, use the following info: {context}. #### "));
        }
        prompt.push_str(string);

        // Tokens are roughly 3/4 of a word; we use a generous approximation and double it to
        // account for the completion side. Some providers reject `max_tokens` that exceed their
        // context window, but for translation snippets this is well within bounds.
        let max_tokens = (prompt.len() / 4) as u32 * 2;
        let body = json!({
            "model": model,
            "temperature": 0.2,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "user", "content": prompt }
            ],
        });

        let response = reqwest::Client::new()
            .post(&api_url)
            .bearer_auth(&api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(anyhow!(RateLimitedError(text)));
            }

            return Err(anyhow!("AI request failed ({status}): {text}"));
        }

        let payload: Value = response.json().await?;
        let mut response_text = payload["choices"]
            .get(0)
            .and_then(|choice| choice["message"]["content"].as_str())
            .ok_or_else(|| anyhow!("Unexpected AI response shape: {payload}"))?
            .to_owned();

        // Some providers prepend stray newlines to chat completions; strip them so the textedit
        // stays clean.
        while response_text.starts_with('\n') {
            response_text.remove(0);
        }

        Ok(response_text)
    }

    fn ask_deepl(string: &str, source_language: Lang, language: Lang) -> Result<String> {
        Self::ask_deepl_many(vec![string.to_owned()], source_language, language)?
            .pop()
            .ok_or_else(|| anyhow!("DeepL returned no translation."))
    }

    /// Translate several texts with DeepL in a single request.
    ///
    /// # Arguments
    ///
    /// * `strings` - Texts to translate. DeepL translates each one independently.
    /// * `source_language` - Language of the texts.
    /// * `language` - Language to translate them to.
    ///
    /// # Returns
    ///
    /// The translations, in the same order as `strings`.
    ///
    /// # Errors
    ///
    /// Returns a [`RateLimitedError`] if DeepL rate-limits the request, or a generic error if the request
    /// fails or doesn't return one translation per text.
    #[tokio::main]
    async fn ask_deepl_many(strings: Vec<String>, source_language: Lang, language: Lang) -> Result<Vec<String>> {
        let api_key = settings_string(DEEPL_API_KEY);
        if api_key.is_empty() {
            return Err(anyhow!("Missing DeepL API Key."))
        };

        let api = DeepLApi::with(&api_key).new();

        let count = strings.len();
        let strings = strings.iter()
            .map(|string| string
                .replace("[[", "<[[")
                .replace("<[[/", "</[[")
                .replace("]]", "]]>"))
            .collect::<Vec<_>>();

        let translated = api.translate_text(strings, language)
            .source_lang(source_language)
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
            .await
            .map_err(|error| match error {
                DeepLError::Network { status, message } if status.as_u16() == 429 => anyhow!(RateLimitedError(message)),
                error => anyhow!(error),
            })?;

        if translated.translations.len() != count {
            return Err(anyhow!("DeepL returned {} translations for {} texts.", translated.translations.len(), count));
        }

        let translated_texts = translated.translations.iter()
            .map(|x| x.text
                .replace("</[[", "<[[/")
                .replace("<[[", "[[")
                .replace("]]>", "]]"))
            .collect();

        Ok(translated_texts)
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
            for index in 0..file_dialog.selected_files().count() {
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

            // Block signals to avoid slow per-line updates.
            let _blocker = QSignalBlocker::from_q_object(self.table().table_model());

            for data in merged_loc.data().iter() {
                let key = data[0].data_to_string();
                let value = data[1].data_to_string();

                // We check against the original pack_tr because it's faster than just searching on the table.
                if let Some(tr) = self.pack_tr.translations().get(&*key) {
                    if tr.src() != &value && tr.dst() != &value {
                        for row in 0..self.table().table_model().row_count_0a() {
                            let key_item = self.table().table_model().item_1a(row);
                            if key_item.text().to_std_string() == key {
                                let needs_retranslation_item = self.table().table_model().item_2a(row, 1);
                                let aut_item = self.table().table_model().item_2a(row, 3);
                                let value_translated_item = self.table().table_model().item_2a(row, 5);

                                needs_retranslation_item.set_check_state(CheckState::Unchecked);
                                aut_item.set_check_state(CheckState::Unchecked);
                                value_translated_item.set_text(&QString::from_std_str(&value));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Auto-translate every row currently flagged as needing re-translation using the chosen backend.
    ///
    /// Rows with an outdated translation are skipped unless the overwrite menu option is checked. Requests
    /// are throttled, and retried with backoff when the service rate-limits them. Successful translations
    /// clear `retr` (column 1) and set `aut` (column 3) so the user can review them. Failed rows are left
    /// untouched, and listed in the results dialog shown at the end.
    pub unsafe fn batch_translate_all(&self, method: BatchTranslateMethod) {
        rpfm_telemetry::track_action("Translator: batch_translate_all");

        // Take a snapshot of the radio button state in case we need it (AI needs the context).
        let source_language_deepl = self.map_source_language_to_deepl();
        let language_deepl = self.map_language_to_deepl();
        let language_natural = self.map_language_to_natural();
        let language_google = self.map_language_to_google();
        let context = self.context_text_edit().to_plain_text().to_std_string();
        let overwrite_outdated = self.batch_translate_overwrite_action().is_checked();

        // Outdated rows keep their previous translation so it can be fixed by hand, unless told otherwise.
        let model = self.table().table_model();
        let rows = (0..model.row_count_0a())
            .filter(|row| model.item_2a(*row, 1).check_state() == CheckState::Checked)
            .filter(|row| overwrite_outdated || model.item_2a(*row, 5).text().trimmed().is_empty())
            .collect::<Vec<_>>();

        let progress = QProgressDialog::new_5a(&QString::new(), &qtr("translator_batch_cancel"), 0, rows.len() as i32, self.tool.main_widget());
        progress.set_window_title(&qtr("translator_batch_translate"));
        progress.set_window_modality(WindowModality::WindowModal);
        progress.set_minimum_duration(0);
        progress.set_auto_close(false);
        progress.set_auto_reset(false);

        // Translation backends operate on plain text — undo the on-disk escaping
        // (||/\\n) before sending, then re-apply after we get the result back.
        let sources = rows.iter()
            .map(|row| model.item_2a(*row, 4).text().to_std_string()
                .replace("||", "\n||\n")
                .replace("\\\\n", "\n"))
            .collect::<Vec<_>>();

        let event_loop = QEventLoop::new_0a();
        let mut interval = BATCH_MIN_INTERVAL;
        let mut last_request: Option<Instant> = None;
        let mut processed = 0;
        let mut translated = 0;
        let mut failed = vec![];

        'groups: for group in Self::batch_groups(&sources, method) {
            progress.set_label_text(&qtre("translator_batch_progress", &[&processed.to_string(), &rows.len().to_string()]));
            progress.set_value(processed as i32);

            let mut retries = 0;
            let result = loop {
                if let Some(last_request) = last_request {
                    let wait = (last_request + interval).saturating_duration_since(Instant::now());
                    if !Self::wait_for_batch(&progress, &event_loop, wait) {
                        break 'groups;
                    }
                }

                last_request = Some(Instant::now());
                let result = match method {
                    BatchTranslateMethod::Deepl => Self::ask_deepl_many(sources[group.clone()].to_vec(), source_language_deepl.clone(), language_deepl.clone()),
                    BatchTranslateMethod::Ai => Self::ask_ai(&sources[group.start], &language_natural, &context).map(|translation| vec![translation]),
                    BatchTranslateMethod::Google => Self::ask_google(&sources[group.start], &language_google).map(|translation| vec![translation]),
                };

                // Being rate-limited means we're going too fast: slow down for the rest of the batch, and back off before retrying.
                match result {
                    Err(error) if error.is::<RateLimitedError>() && retries < BATCH_MAX_RETRIES => {
                        retries += 1;
                        interval = (interval * 2).min(BATCH_MAX_INTERVAL);
                        if !Self::wait_for_batch(&progress, &event_loop, Duration::from_secs(1 << retries)) {
                            break 'groups;
                        }
                    }
                    result => break result,
                }
            };

            processed += group.len();
            let group_rows = &rows[group];
            match result {
                Ok(translations) => {
                    for (row, translation) in group_rows.iter().zip(translations) {
                        let stored = translation
                            .replace("\n||\n", "||")
                            .replace("\n", "\\\\n");
                        model.item_2a(*row, 5).set_text(&QString::from_std_str(&stored));
                        model.item_2a(*row, 1).set_check_state(CheckState::Unchecked);
                        model.item_2a(*row, 3).set_check_state(CheckState::Checked);
                        translated += 1;
                    }
                }

                // A failed request fails every line in its group.
                Err(error) => failed.extend(group_rows.iter().map(|row| format!("{}: {error}", model.item_2a(*row, 0).text().to_std_string()))),
            }
        }

        progress.close();

        let not_processed = rows.len() - processed;
        let mut text = tre("translator_batch_results", &[&translated.to_string(), &failed.len().to_string(), &not_processed.to_string()]);
        if !failed.is_empty() {
            text.push_str(&tr("translator_batch_results_failed"));
        }

        let icon = if failed.is_empty() && not_processed == 0 { Icon::Information } else { Icon::Warning };
        let message_box = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(icon, &qtr("translator_batch_results_title"), &QString::from_std_str(text), QFlags::from(StandardButton::Ok), self.tool.main_widget());
        if !failed.is_empty() {
            message_box.set_detailed_text(&QString::from_std_str(failed.join("\n")));
        }
        message_box.exec();
    }

    /// Split the texts of a batch auto-translation into the groups sent in each request.
    ///
    /// DeepL accepts several texts per request, so its groups are as big as its limits allow. The other
    /// backends take one text per request.
    ///
    /// # Arguments
    ///
    /// * `sources` - Texts to translate.
    /// * `method` - Backend the texts are sent to.
    ///
    /// # Returns
    ///
    /// Consecutive, non-empty ranges of `sources` covering all of it.
    fn batch_groups(sources: &[String], method: BatchTranslateMethod) -> Vec<Range<usize>> {
        let (max_texts, max_bytes) = match method {
            BatchTranslateMethod::Deepl => (DEEPL_GROUP_MAX_TEXTS, DEEPL_GROUP_MAX_BYTES),
            BatchTranslateMethod::Ai | BatchTranslateMethod::Google => (1, usize::MAX),
        };

        let mut groups = vec![];
        let mut start = 0;
        let mut bytes = 0;
        for (index, source) in sources.iter().enumerate() {

            // A text bigger than the byte limit still goes alone in its own group.
            if index > start && (index - start == max_texts || bytes + source.len() > max_bytes) {
                groups.push(start..index);
                start = index;
                bytes = 0;
            }

            bytes += source.len();
        }

        if start < sources.len() {
            groups.push(start..sources.len());
        }

        groups
    }

    /// Wait while keeping the UI responsive, so the batch progress dialog can still be cancelled.
    ///
    /// # Returns
    ///
    /// `false` if the user cancelled the batch while waiting, `true` otherwise.
    unsafe fn wait_for_batch(progress: &QBox<QProgressDialog>, event_loop: &QBox<QEventLoop>, duration: Duration) -> bool {
        let deadline = Instant::now() + duration;
        loop {
            event_loop.process_events();
            if progress.was_canceled() {
                return false;
            }

            let now = Instant::now();
            if now >= deadline {
                return true;
            }

            std::thread::sleep((deadline - now).min(Duration::from_millis(50)));
        }
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
                let mut files = send_ipc_command(Command::GetRFilesFromAllSources(vec![ContainerPath::File(path.to_owned())], false), response_extractor!(Response::HashMapDataSourceHashMapStringRFile));
                {
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
                }

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

/// Show a message box with a custom icon and title.
unsafe fn show_message(parent: impl CastInto<Ptr<QWidget>>, icon: Icon, title: &CppBox<QString>, text: &CppBox<QString>) {
    let message_box = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(icon, title, text, QFlags::from(StandardButton::Ok), parent);
    message_box.exec();
}
