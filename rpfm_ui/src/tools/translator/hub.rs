//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Submission of translations to the Translation Hub, from the translator.

use qt_widgets::QAbstractButton;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton as DialogButton;
use qt_widgets::QHBoxLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QMessageBox;
use qt_widgets::q_message_box::{ButtonRole, Icon, StandardButton};
use qt_widgets::QProgressDialog;
use qt_widgets::QVBoxLayout;

use qt_gui::QDesktopServices;

use qt_core::QFlags;
use qt_core::QString;
use qt_core::QUrl;
use qt_core::TextFormat;
use qt_core::WindowModality;

use anyhow::{anyhow, Result};

use rpfm_extensions::translator::PackTranslation;

use rpfm_ipc::api::translations::{SubmitTranslation, TranslationSubmission};

use crate::GAME_SELECTED;
use crate::communications::call_api_async;
use crate::github_ui;
use crate::settings_ui::backend::translations_local_path;
use crate::utils::{qtr, qtre, tr, tre};

use super::ToolTranslator;

impl ToolTranslator {

    /// Submit the translation to the Translation Hub as a pull request, or update the one already open for it.
    ///
    /// Saves the translation, asks the user to confirm with a summary, signs in to GitHub if needed, and shows
    /// the resulting pull request.
    ///
    /// # Errors
    ///
    /// Returns an error if the translation can't be saved, signing in fails, or the submission fails.
    pub unsafe fn submit_translation(&self) -> Result<()> {
        self.change_selected_row(None, None);
        let mut pack_tr = self.save_translation_file()?;

        let Some(authors) = self.confirm_submission(&pack_tr) else {
            return Ok(());
        };

        if authors != self.authors_line_edit.text().to_std_string() {
            self.authors_line_edit.set_text(&QString::from_std_str(&authors));
            pack_tr = self.save_translation_file()?;
        }

        // A missing or rejected sign-in gets one chance to sign in and retry.
        for _ in 0..2 {
            match self.send_submission(&pack_tr)? {
                TranslationSubmission::Submitted { url, created } => {
                    self.show_submission_result(&url, created);
                    return Ok(());
                },
                TranslationSubmission::SignInRequired => if github_ui::sign_in(self.tool.main_widget())?.is_none() {
                    return Ok(());
                },
            }
        }

        Err(anyhow!(tr("translator_submit_sign_in_failed")))
    }

    /// Save the translation's file with the current state of the translator, without touching the pack.
    ///
    /// # Returns
    ///
    /// The saved translation.
    pub(super) unsafe fn save_translation_file(&self) -> Result<PackTranslation> {
        let mut pack_tr = self.snapshot_pack_translation()?;
        pack_tr.save(&translations_local_path()?, GAME_SELECTED.read().unwrap().key())?;
        Ok(pack_tr)
    }

    /// Show the summary of a submission, so the user can review it and fill in the authors.
    ///
    /// # Returns
    ///
    /// The authors, as typed in the dialog, or `None` if the user cancelled.
    unsafe fn confirm_submission(&self, pack_tr: &PackTranslation) -> Option<String> {
        let stats = pack_tr.stats();
        let dialog = QDialog::new_1a(self.tool.main_widget());
        dialog.set_window_title(&qtr("translator_submit_title"));
        let layout = QVBoxLayout::new_1a(&dialog);

        let intro_label = QLabel::from_q_string_q_widget(&qtr("translator_submit_intro"), &dialog);
        intro_label.set_word_wrap(true);
        layout.add_widget(&intro_label);

        let summary_label = QLabel::from_q_string_q_widget(&qtre("translator_submit_summary", &[
            GAME_SELECTED.read().unwrap().display_name(),
            pack_tr.pack_name(),
            pack_tr.src_lang(),
            pack_tr.language(),
            &pack_tr.version().to_string(),
            &stats.translated().to_string(),
            &stats.total().to_string(),
        ]), &dialog);
        summary_label.set_text_format(TextFormat::RichText);
        layout.add_widget(&summary_label);

        let mut warnings = vec![];
        if *stats.pending() > 0 {
            warnings.push(tre("translator_submit_pending_warning", &[&stats.pending().to_string()]));
        }

        if *stats.auto_translated() > 0 {
            warnings.push(tre("translator_submit_auto_warning", &[&stats.auto_translated().to_string()]));
        }

        if !warnings.is_empty() {
            let warnings_label = QLabel::from_q_string_q_widget(&QString::from_std_str(warnings.join("<br/>")), &dialog);
            warnings_label.set_text_format(TextFormat::RichText);
            warnings_label.set_word_wrap(true);
            layout.add_widget(&warnings_label);
        }

        let authors_layout = QHBoxLayout::new_0a();
        let authors_label = QLabel::from_q_string_q_widget(&qtr("translator_authors"), &dialog);
        let authors_line_edit = QLineEdit::from_q_string_q_widget(&self.authors_line_edit.text(), &dialog);
        authors_line_edit.set_placeholder_text(&qtr("translator_submit_authors_placeholder"));
        authors_layout.add_widget(&authors_label);
        authors_layout.add_widget(&authors_line_edit);
        layout.add_layout_1a(&authors_layout);

        let button_box = QDialogButtonBox::from_q_widget(&dialog);
        let submit_button = button_box.add_button_standard_button(DialogButton::Ok);
        submit_button.set_text(&qtr("translator_submit"));
        button_box.add_button_standard_button(DialogButton::Cancel);
        button_box.accepted().connect(dialog.slot_accept());
        button_box.rejected().connect(dialog.slot_reject());
        layout.add_widget(&button_box);

        if dialog.exec() == 1 {
            Some(authors_line_edit.text().to_std_string())
        } else {
            None
        }
    }

    /// Send a saved translation to the server for submission, showing a progress dialog meanwhile.
    ///
    /// # Returns
    ///
    /// The submission's result.
    unsafe fn send_submission(&self, pack_tr: &PackTranslation) -> Result<TranslationSubmission> {

        // A null cancel text means no cancel button: the server can't stop a submission halfway.
        let progress = QProgressDialog::new_5a(&qtr("translator_submit_progress"), &QString::new(), 0, 0, self.tool.main_widget());
        progress.set_window_title(&qtr("translator_submit_title"));
        progress.set_window_modality(WindowModality::WindowModal);
        progress.set_minimum_duration(0);
        progress.show();

        let request = SubmitTranslation {
            pack_name: pack_tr.pack_name().to_owned(),
            source_language: pack_tr.src_lang().to_owned(),
            language: pack_tr.language().to_owned(),
        };

        let result = call_api_async(&request);

        progress.close();
        result
    }

    /// Tell the user the pull request is ready, offering to open it.
    ///
    /// # Arguments
    ///
    /// * `url` - Web page of the pull request.
    /// * `created` - If the pull request is new, instead of an updated one.
    unsafe fn show_submission_result(&self, url: &str, created: bool) {
        let text = if created { qtr("translator_submit_created") } else { qtr("translator_submit_updated") };
        let message_box = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(Icon::Information, &qtr("translator_submit_title"), &text, QFlags::from(StandardButton::Close), self.tool.main_widget());
        let open_button = message_box.add_button_q_string_button_role(&qtr("translator_submit_open"), ButtonRole::ActionRole);
        message_box.exec();

        if message_box.clicked_button().as_raw_ptr() == open_button.static_upcast::<QAbstractButton>().as_raw_ptr() {
            QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(url)));
        }
    }
}
