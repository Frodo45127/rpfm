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
Module with all the code for managing the PackFile-Specific settings.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QSpinBox;
use qt_widgets::QPlainTextEdit;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

use std::collections::BTreeMap;
use std::sync::Arc;

use rpfm_error::Result;

use rpfm_lib::packfile::PackFileSettings;
use rpfm_lib::packedfile::PackedFileType;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::locale::qtr;
use crate::packedfile_views::PackedFileView;
use super::{ViewType, View};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackFile Settings.
#[derive(Default)]
pub struct PackFileSettingsView {
    settings_text_multi_line: BTreeMap<String, QBox<QPlainTextEdit>>,
    settings_text_single_line: BTreeMap<String, QBox<QLineEdit>>,
    settings_bool: BTreeMap<String, QBox<QCheckBox>>,
    settings_number: BTreeMap<String, QBox<QSpinBox>>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackFileSettingsView`.
impl PackFileSettingsView {

    /// This function creates a new PackFileSettingsView, and sets up his slots and connections.
    ///
    /// The view is loaded dinamically based on the settings we have.
    pub unsafe fn new_view(
        pack_file_view: &mut PackedFileView,
    ) -> Result<()> {

        CENTRAL_COMMAND.send_message_qt(Command::GetPackFileSettings);
        let response = CENTRAL_COMMAND.recv_message_qt();
        let settings = match response {
            Response::PackFileSettings(settings) => settings,
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let mut view = Self::default();
        let layout: QPtr<QGridLayout> = pack_file_view.get_mut_widget().layout().static_downcast();

        let mut row = 0;
        for (key, setting) in &settings.settings_text {
            let label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_label", key)), pack_file_view.get_mut_widget());
            let description_label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_description_label", key)), pack_file_view.get_mut_widget());
            let edit = QPlainTextEdit::from_q_string_q_widget(&QString::from_std_str(&setting), pack_file_view.get_mut_widget());
            description_label.set_word_wrap(true);

            layout.add_widget_5a(&label, row, 0, 1, 1);
            layout.add_widget_5a(&description_label, row + 1, 0, 1, 1);
            layout.add_widget_5a(&edit, row, 1, 2, 1);
            layout.set_row_stretch(row + 1, 100);

            view.settings_text_multi_line.insert(key.to_owned(), edit);
            row += 2;
        }

        for (key, setting) in &settings.settings_string {
            let label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_label", key)), pack_file_view.get_mut_widget());
            let _description_label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_description_label", key)), pack_file_view.get_mut_widget());
            let edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(&setting), pack_file_view.get_mut_widget());

            layout.add_widget_5a(&label, row, 0, 1, 1);
            layout.add_widget_5a(&edit, row, 1, 1, 1);

            view.settings_text_single_line.insert(key.to_owned(), edit);
            row += 1;
        }

        for (key, setting) in &settings.settings_bool {
            let label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_label", key)), pack_file_view.get_mut_widget());
            let _description_label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_description_label", key)), pack_file_view.get_mut_widget());
            let edit = QCheckBox::from_q_widget(pack_file_view.get_mut_widget());
            edit.set_checked(*setting);

            layout.add_widget_5a(&label, row, 0, 1, 1);
            layout.add_widget_5a(&edit, row, 1, 1, 1);

            view.settings_bool.insert(key.to_owned(), edit);
            row += 1;
        }

        for (key, setting) in &settings.settings_number {
            let label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_label", key)), pack_file_view.get_mut_widget());
            let _description_label = QLabel::from_q_string_q_widget(&qtr(&format!("pfs_{}_description_label", key)), pack_file_view.get_mut_widget());
            let edit = QSpinBox::new_1a(pack_file_view.get_mut_widget());
            edit.set_value(*setting);

            layout.add_widget_5a(&label, row, 0, 1, 1);
            layout.add_widget_5a(&edit, row, 1, 1, 1);

            view.settings_number.insert(key.to_owned(), edit);
            row += 1;
        }

        let padding_widget = QWidget::new_1a(pack_file_view.get_mut_widget());
        layout.add_widget_5a(&padding_widget, row, 0, 1, 3);
        layout.set_row_stretch(row, 1000);


        pack_file_view.packed_file_type = PackedFileType::PackFileSettings;
        pack_file_view.view = ViewType::Internal(View::PackFileSettings(Arc::new(view)));

        Ok(())
    }

    /// This function saves a PackFileSettingsView into a PackFileSetting.
    pub unsafe fn save_view(&self) -> PackFileSettings {
        let mut settings = PackFileSettings::default();
        self.settings_text_multi_line.iter().for_each(|(key, widget)| { settings.settings_text.insert(key.to_owned(), widget.to_plain_text().to_std_string()); });
        self.settings_text_single_line.iter().for_each(|(key, widget)| { settings.settings_string.insert(key.to_owned(), widget.text().to_std_string()); });
        self.settings_bool.iter().for_each(|(key, widget)| { settings.settings_bool.insert(key.to_owned(), widget.is_checked()); });
        self.settings_number.iter().for_each(|(key, widget)| { settings.settings_number.insert(key.to_owned(), widget.value()); });

        settings
    }
}
