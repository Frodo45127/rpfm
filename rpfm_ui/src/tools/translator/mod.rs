//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

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

use getset::*;

use std::sync::{Arc, RwLock};

use rpfm_extensions::translator::*;

use rpfm_lib::files::{ContainerPath, RFileDecoded, table::DecodedData};

use rpfm_ui_common::locale::tr;

use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::translations_local_path;
use crate::views::table::{filter::*, TableType, TableView, utils::get_table_from_view};

use self::slots::ToolTranslatorSlots;
use super::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/tool_translator_editor.ui";
const VIEW_RELEASE: &str = "ui/tool_translator_editor.ui";

/// List of games this tool supports.
const TOOL_SUPPORTED_GAMES: [&str; 1] = [
    "warhammer_3",
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
        let table_view: QPtr<QTableView> = find_widget(&tool.main_widget().static_upcast(), "table_view")?;
        let table_view_container: QPtr<QWidget> = find_widget(&tool.main_widget().static_upcast(), "table_view_container")?;
        let table_view_container = table_view_container.into_q_box();

        // Unlike other tools, data is loaded here, because we need it to generate the table widget.
        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackTranslation);
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
        table.table_view().sort_by_column_1a(0);

        if let Some(filter_removed) = table.filters().get(0) {
            filter_removed.filter_line_edit().set_text(&QString::from_std_str("false"));
            filter_removed.column_combobox().set_current_index(2);
            filter_removed.use_regex_button().set_checked(false);
        }

        FilterView::new(&table)?;

        if let Some(filter_retranslation) = table.filters().get(1) {
            filter_retranslation.filter_line_edit().set_text(&QString::from_std_str("true"));
            filter_retranslation.column_combobox().set_current_index(1);
            filter_retranslation.use_regex_button().set_checked(false);
        }

        let original_value_textedit: QPtr<QTextEdit> = tool.find_widget("original_value_textedit")?;
        let translated_value_textedit: QPtr<QTextEdit> = tool.find_widget("translated_value_textedit")?;

        // Build the view itself.
        let view = Rc::new(Self{
            tool,
            pack_tr: Arc::new(data),
            table,
            original_value_textedit,
            translated_value_textedit,
        });

        // Build the slots and connect them to the view.
        let slots = ToolTranslatorSlots::new(&view);
        connections::set_connections(&view, &slots);

        // Setup text translations.
        view.setup_translations();

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

        self.original_value_textedit.set_text(&original_value_item.text());
        self.translated_value_textedit.set_text(&translated_value_item.text());
    }

    pub unsafe fn save_from_detailed_view(&self, index: Ref<QModelIndex>) {
        let old_value_item = self.table.table_model().item_2a(index.row(), 4);
        let old_value = old_value_item.text().to_std_string();
        let new_value = self.translated_value_textedit.to_plain_text().to_std_string();

        // If we have a new translation, save it and remove the "needs_retranslation" flag.
        if !new_value.is_empty() && new_value != old_value {
            old_value_item.set_text(&QString::from_std_str(new_value));
            self.table.table_model().item_2a(index.row(), 0).set_check_state(CheckState::Unchecked);
        }
    }

    /// Function to setup all the translations of this view.
    pub unsafe fn setup_translations(&self) {
        /*
        self.banner_groupbox.set_title(&qtr("banner"));
        self.uniform_groupbox.set_title(&qtr("uniform"));
        */
    }
}
