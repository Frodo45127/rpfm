//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_core::QBox;
use qt_core::QEventLoop;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQItemSelectionQItemSelection;

use std::rc::Rc;

use rpfm_lib::integrations::log::*;

use rpfm_ui_common::clone;

use crate::utils::show_dialog;

use super::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct ToolTranslatorSlots {
    load_data_to_detailed_view: QBox<SlotOfQItemSelectionQItemSelection>,
    move_selection_up: QBox<SlotNoArgs>,
    move_selection_down: QBox<SlotNoArgs>,
    translate_with_deepl: QBox<SlotNoArgs>,
    translate_with_chatgpt: QBox<SlotNoArgs>,
    translate_with_google: QBox<SlotNoArgs>,
    copy_from_source: QBox<SlotNoArgs>,
    import_from_translated_pack: QBox<SlotNoArgs>,
    update_preview_original: QBox<SlotNoArgs>,
    update_preview_translated: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl ToolTranslatorSlots {

    /// This function creates a new `ToolTranslatorSlots`.
    pub unsafe fn new(ui: &Rc<ToolTranslator>) -> Self {

        let load_data_to_detailed_view = SlotOfQItemSelectionQItemSelection::new(ui.tool.main_widget(), clone!(
            ui => move |after, _| {
                info!("Triggering 'load_data_to_detailed_view' for Translator.");

                if after.count_0a() == 1 {
                    let base_index = after.at(0);
                    let indexes = base_index.indexes();
                    let filter_index = indexes.at(0);
                    let index = ui.table().table_filter().map_to_source(filter_index);
                    ui.change_selected_row(Some(index), None);
                } else {
                    ui.change_selected_row(None, None);
                }
            }
        ));

        let move_selection_up = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'move_selection_up' for Translator.");

                ui.change_selected_row(None, Some(false));
            }
        ));

        let move_selection_down = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'move_selection_down' for Translator.");

                ui.change_selected_row(None, Some(true));
            }
        ));

        let translate_with_deepl = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'translate_with_deepl' for Translator.");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events_0a();

                let source_text = ui.original_value_textedit().to_plain_text().to_std_string();
                let language = ui.map_language_to_deepl();
                let result = ToolTranslator::ask_deepl(&source_text, language);
                if let Ok(tr) = result {
                    ui.translated_value_textedit.set_text(&QString::from_std_str(tr));
                }

                ui.translated_value_textedit().set_enabled(true);
            }
        ));

        let translate_with_chatgpt = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'translate_with_chatgpt' for Translator.");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events_0a();

                let source_text = ui.original_value_textedit().to_plain_text().to_std_string();
                let language = ui.map_language_to_natural();
                let context = ui.context_text_edit().to_plain_text().to_std_string();
                let result = ToolTranslator::ask_chat_gpt(&source_text, &language, &context);
                if let Ok(tr) = result {
                    ui.translated_value_textedit.set_text(&QString::from_std_str(tr));
                }

                ui.translated_value_textedit().set_enabled(true);
            }
        ));

        let translate_with_google = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'translate_with_google' for Translator.");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events_0a();

                let source_text = ui.original_value_textedit().to_plain_text().to_std_string();
                let language = ui.map_language_to_google();
                let result = ToolTranslator::ask_google(&source_text, &language);
                if let Ok(tr) = result {
                    ui.translated_value_textedit.set_text(&QString::from_std_str(tr));
                }

                ui.translated_value_textedit().set_enabled(true);
            }
        ));

        let copy_from_source = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'copy_from_source' for Translator.");

                let source_text = ui.original_value_textedit().to_plain_text();
                ui.translated_value_textedit().set_text(&source_text);
            }
        ));

        let import_from_translated_pack = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'import_from_translated_pack' for Translator.");

                if let Err(error) = ui.import_from_another_pack() {
                    show_dialog(ui.tool.main_widget(), error, false);
                }
            }
        ));

        let update_preview_original = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'update_preview_original' for Translator.");

                ui.original_value_html().clear();
                ui.original_value_html().set_text(&QString::from_std_str(ui.to_html(&ui.original_value_textedit().to_plain_text().to_std_string())));
            }
        ));

        let update_preview_translated = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                info!("Triggering 'update_preview_translated' for Translator.");

                ui.translated_value_html().clear();
                ui.translated_value_html().set_text(&QString::from_std_str(ui.to_html(&ui.translated_value_textedit().to_plain_text().to_std_string())));
            }
        ));

        ToolTranslatorSlots {
            load_data_to_detailed_view,
            move_selection_up,
            move_selection_down,
            translate_with_deepl,
            translate_with_chatgpt,
            translate_with_google,
            copy_from_source,
            import_from_translated_pack,
            update_preview_original,
            update_preview_translated,
        }
    }
}
