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
use qt_widgets::QFileDialog;
use qt_widgets::q_file_dialog::FileMode;
use qt_widgets::QGroupBox;
use qt_widgets::QToolButton;
use qt_widgets::q_abstract_item_view::{SelectionBehavior, SelectionMode};
use qt_widgets::QGridLayout;
use qt_widgets::QTableView;

use qt_core::CheckState;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QPtr;
use qt_core::QString;

use cpp_core::CppDeletable;
use cpp_core::Ref;

use anyhow::anyhow;
use getset::*;
use rust_translate::translate_from_english;

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

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ToolTranslator {
    tool: Tool,
    pack_tr: Arc<PackTranslation>,
    table: Arc<TableView>,

    // Row being edited. We don't trust the selection as it may be bugged/not work.
    current_row: Arc<RwLock<Option<i32>>>,

    language_combobox: QPtr<QComboBox>,

    action_move_up: QPtr<QAction>,
    action_move_down: QPtr<QAction>,
    action_copy_from_source: QPtr<QAction>,
    action_import_from_translated_pack: QPtr<QAction>,

    move_selection_up: QPtr<QToolButton>,
    move_selection_down: QPtr<QToolButton>,
    copy_from_source: QPtr<QToolButton>,
    import_from_translated_pack: QPtr<QToolButton>,

    original_value_textedit: QPtr<QTextEdit>,
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
        let copy_from_source: QPtr<QToolButton> = tool.find_widget("copy_from_source")?;
        let import_from_translated_pack: QPtr<QToolButton> = tool.find_widget("import_from_translated_pack")?;
        move_selection_up.set_tool_tip(&qtr("translator_move_selection_up"));
        move_selection_down.set_tool_tip(&qtr("translator_move_selection_down"));
        copy_from_source.set_tool_tip(&qtr("translator_copy_from_source"));
        import_from_translated_pack.set_tool_tip(&qtr("translator_import_from_translated_pack"));

        let action_move_up = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_up", Some(table.table_view().static_upcast()));
        let action_move_down = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "move_down", Some(table.table_view().static_upcast()));
        let action_copy_from_source = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "copy_from_source", Some(table.table_view().static_upcast()));
        let action_import_from_translated_pack = add_action_to_widget(app_ui.shortcuts().as_ref(), "translator", "import_from_translated_pack", Some(table.table_view().static_upcast()));

        let original_value_textedit: QPtr<QTextEdit> = tool.find_widget("original_value_textedit")?;
        let translated_value_textedit: QPtr<QTextEdit> = tool.find_widget("translated_value_textedit")?;

        // Build the view itself.
        let view = Rc::new(Self{
            tool,
            pack_tr: Arc::new(data),
            table,
            current_row: Arc::new(RwLock::new(None)),
            language_combobox,
            action_move_up,
            action_move_down,
            action_copy_from_source,
            action_import_from_translated_pack,
            move_selection_up,
            move_selection_down,
            copy_from_source,
            import_from_translated_pack,
            original_value_textedit,
            translated_value_textedit,
        });

        // Build the slots and connect them to the view.
        let slots = ToolTranslatorSlots::new(&view);
        connections::set_connections(&view, &slots);

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
        self.table().table_view().selection_model().select_q_item_selection_q_flags_selection_flag(&self.table().table_view().selection_model().selection(), SelectionFlag::Toggle.into());

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
    pub unsafe fn load_to_detailed_view(&self, index: Ref<QModelIndex>) {
        let original_value_item = self.table.table_model().item_2a(index.row(), 3);
        let translated_value_item = self.table.table_model().item_2a(index.row(), 4);
        let needs_retranslation = self.table.table_model().item_2a(index.row(), 1).check_state() == CheckState::Checked;

        self.original_value_textedit.set_text(&original_value_item.text());
        self.translated_value_textedit.set_text(&translated_value_item.text());

        // Update the row in edition.
        *self.current_row.write().unwrap() = Some(index.row());

        // If the value needs a retrasnlation ask google for one.
        if needs_retranslation {
            let language = self.map_language_to_google();
            if let Ok(tr) = Self::ask_google(&original_value_item.text().to_std_string(), &language) {
                self.translated_value_textedit.set_text(&QString::from_std_str(tr));
            }
        }
    }

    // Selection is EXTREMELY unreliable. We save to the current row instead.
    pub unsafe fn save_from_detailed_view(&self) {
        let current_row = self.current_row.read().unwrap().clone();
        if let Some(current_row) = current_row {

            let old_value_item = self.table.table_model().item_2a(current_row, 4);
            let old_value = old_value_item.text().to_std_string();
            let new_value = self.translated_value_textedit.to_plain_text().to_std_string();

            // If we have a new translation, save it and remove the "needs_retranslation" flag.
            if !new_value.is_empty() && new_value != old_value {

                // If there's any other translation which uses the same value, automatically translate it.
                let original_value_item = self.table.table_model().item_2a(current_row, 3);
                for row in 0..self.table.table_model().row_count_0a() {

                    // Do not apply it to the item we just edited.
                    if current_row != row {
                        let needs_retranslation_item = self.table.table_model().item_2a(row, 1);
                        let needs_retranslation = needs_retranslation_item.check_state() == CheckState::Checked;
                        if needs_retranslation {
                            let og_value_item = self.table.table_model().item_2a(row, 3);
                            if og_value_item.data_1a(2).to_string().to_std_string() == original_value_item.data_1a(2).to_string().to_std_string() {
                                let translated_value_item = self.table.table_model().item_2a(row, 4);
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

    #[tokio::main]
    async fn ask_google(string: &str, language: &str) -> Result<String> {
        if !string.trim().is_empty() {
            let string = string.replace("\\\n", "\n");
            translate_from_english(&string, language).await
                .map(|string|
                    string
                        .replace("\n", "\\\n")          // Fix jump lines.
                        .replace("%20", " ")            // Fix weird spaces.
                )
                .map_err(|err| anyhow!(err.to_string()))
        } else {
            Ok(String::new())
        }
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

            let mut pack = Pack::read_and_merge(&paths, true, false)?;
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
}
