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
use qt_core::SlotOfBool;
use qt_core::SlotOfInt;
use qt_core::SlotOfQItemSelectionQItemSelection;
use qt_core::SlotOfQVariant;

use std::rc::Rc;


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
    translate_with_ai: QBox<SlotNoArgs>,
    translate_with_google: QBox<SlotNoArgs>,
    copy_from_source: QBox<SlotNoArgs>,
    clear_translation: QBox<SlotNoArgs>,
    import_from_translated_pack: QBox<SlotNoArgs>,
    batch_translate_deepl: QBox<SlotNoArgs>,
    batch_translate_ai: QBox<SlotNoArgs>,
    batch_translate_google: QBox<SlotNoArgs>,
    toggle_glossary: QBox<SlotNoArgs>,
    glossary_animation_step: QBox<SlotOfQVariant>,
    glossary_animation_finished: QBox<SlotNoArgs>,
    version_changed: QBox<SlotOfInt>,
    use_deepl_glossary_toggled: QBox<SlotOfBool>,
    toggle_help: QBox<SlotNoArgs>,
    toggle_preview: QBox<SlotNoArgs>,
    toggle_behavior: QBox<SlotNoArgs>,
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
                rpfm_telemetry::track_action("Translator: load_data_to_detailed_view");

                if after.count() == 1 {
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
                rpfm_telemetry::track_action("Translator: move_selection_up");

                ui.change_selected_row(None, Some(false));
            }
        ));

        let move_selection_down = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: move_selection_down");

                ui.change_selected_row(None, Some(true));
            }
        ));

        let translate_with_deepl = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: translate_with_deepl");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events();

                let source_text = ui.original_value_textedit().to_plain_text().to_std_string();
                let source_language = ui.map_source_language_to_deepl();
                let language = ui.map_language_to_deepl();
                let glossary = ui.glossary_snapshot();
                let (deepl_glossary, _) = ui.deepl_glossary(&glossary, &source_language, &language);
                let result = ToolTranslator::ask_deepl(&source_text, source_language, language, &deepl_glossary);
                if let Ok(tr) = result {
                    ui.translated_value_textedit.set_text(&QString::from_std_str(tr));
                }

                ui.translated_value_textedit().set_enabled(true);
            }
        ));

        let translate_with_ai = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: translate_with_ai");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events();

                let source_text = ui.original_value_textedit().to_plain_text().to_std_string();
                let language = ui.map_language_to_natural();
                let context = ui.context_text_edit().to_plain_text().to_std_string();
                let glossary = ui.glossary_snapshot();
                let result = ToolTranslator::ask_ai(&source_text, &language, &context, &glossary);
                if let Ok(tr) = result {
                    ui.translated_value_textedit.set_text(&QString::from_std_str(tr));
                }

                ui.translated_value_textedit().set_enabled(true);
            }
        ));

        let translate_with_google = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: translate_with_google");

                ui.translated_value_textedit().set_enabled(false);
                let event_loop = QEventLoop::new_0a();
                event_loop.process_events();

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
                rpfm_telemetry::track_action("Translator: copy_from_source");

                let source_text = ui.original_value_textedit().to_plain_text();
                ui.translated_value_textedit().set_text(&source_text);
            }
        ));

        let clear_translation = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: clear_translation");
                ui.clear_selected_translation();
            }
        ));

        let import_from_translated_pack = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: import_from_translated_pack");

                if let Err(error) = ui.import_from_another_pack() {
                    show_dialog(ui.tool.main_widget(), error, false);
                }
            }
        ));

        let batch_translate_deepl = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.batch_translate_all(BatchTranslateMethod::Deepl);
            }
        ));

        let batch_translate_ai = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.batch_translate_all(BatchTranslateMethod::Ai);
            }
        ));

        let batch_translate_google = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.batch_translate_all(BatchTranslateMethod::Google);
            }
        ));

        let toggle_glossary = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: toggle_glossary");
                ui.toggle_glossary_pane();
            }
        ));

        let glossary_animation_step = SlotOfQVariant::new(ui.tool.main_widget(), clone!(
            ui => move |width| {
                ui.glossary_pane().set_fixed_width(width.to_int_0a());
            }
        ));

        // Once closed, hide the pane so it doesn't keep a zero-width widget around taking focus.
        let glossary_animation_finished = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                if !*ui.glossary_visible().read().unwrap() {
                    ui.glossary_pane().hide();
                }
            }
        ));

        let version_changed = SlotOfInt::new(ui.tool.main_widget(), clone!(
            ui => move |_| {
                ui.update_v1_only_widgets();
            }
        ));

        let use_deepl_glossary_toggled = SlotOfBool::new(ui.tool.main_widget(), move |enabled| {
            let _ = settings_set_bool(TRANSLATOR_USE_DEEPL_GLOSSARY, enabled);
        });

        // Visibility toggles for the collapsible sections. The QPushButtons are checkable, so
        // by the time `released` fires Qt has already flipped their checked state.
        let toggle_help = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.info_label().set_visible(ui.help_toggle().is_checked());
            }
        ));

        let toggle_preview = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                let visible = ui.preview_toggle().is_checked();
                ui.original_value_html().set_visible(visible);
                ui.translated_value_html().set_visible(visible);
            }
        ));

        let toggle_behavior = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                ui.behavior_groupbox().set_visible(ui.behavior_toggle().is_checked());
            }
        ));

        let update_preview_original = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: update_preview_original");

                ui.original_value_html().clear();
                ui.original_value_html().set_text(&QString::from_std_str(ui.to_html(&ui.original_value_textedit().to_plain_text().to_std_string())));
            }
        ));

        let update_preview_translated = SlotNoArgs::new(ui.tool.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Translator: update_preview_translated");

                ui.translated_value_html().clear();
                ui.translated_value_html().set_text(&QString::from_std_str(ui.to_html(&ui.translated_value_textedit().to_plain_text().to_std_string())));
            }
        ));

        ToolTranslatorSlots {
            load_data_to_detailed_view,
            move_selection_up,
            move_selection_down,
            translate_with_deepl,
            translate_with_ai,
            translate_with_google,
            copy_from_source,
            clear_translation,
            import_from_translated_pack,
            batch_translate_deepl,
            batch_translate_ai,
            batch_translate_google,
            toggle_glossary,
            glossary_animation_step,
            glossary_animation_finished,
            version_changed,
            use_deepl_glossary_toggled,
            toggle_help,
            toggle_preview,
            toggle_behavior,
            update_preview_original,
            update_preview_translated,
        }
    }
}
