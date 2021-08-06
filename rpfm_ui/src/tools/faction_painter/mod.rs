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
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QPushButton;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;

use qt_ui_tools::QUiLoader;

use std::fs::File;
use std::io::Read;

use rpfm_lib::packfile::PathType;
use rpfm_lib::packedfile::DecodedPackedFile;
use rpfm_lib::packedfile::table::DecodedData;

use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::locale::qtr;
use crate::utils::show_dialog;
use super::*;

mod connections;
mod slots;

const VIEW: &'static str = "rpfm_ui/ui_templates/tool_faction_color_editor.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct represents the common content and behavior shared across Tools.
pub struct ToolFactionPainter {
    tool: Tool,
    dialog: QBox<QDialog>,
    faction_list_view: QPtr<QListView>,
    faction_list_filter: QBox<QSortFilterProxyModel>,
    faction_list_model: QBox<QStandardItemModel>,
    faction_list_filter_line_edit: QPtr<QLineEdit>,
    faction_name_label: QPtr<QLabel>,
    banner_groupbox: QPtr<QGroupBox>,
    banner_colour_primary_label: QPtr<QLabel>,
    banner_colour_secondary_label: QPtr<QLabel>,
    banner_colour_tertiary_label: QPtr<QLabel>,
    banner_colour_primary: QPtr<QComboBox>,
    banner_colour_secondary: QPtr<QComboBox>,
    banner_colour_tertiary: QPtr<QComboBox>,
    banner_restore_initial_values_button: QPtr<QPushButton>,
    banner_restore_vanilla_values_button: QPtr<QPushButton>,
    uniform_groupbox: QPtr<QGroupBox>,
    uniform_colour_primary_label: QPtr<QLabel>,
    uniform_colour_secondary_label: QPtr<QLabel>,
    uniform_colour_tertiary_label: QPtr<QLabel>,
    uniform_colour_primary: QPtr<QComboBox>,
    uniform_colour_secondary: QPtr<QComboBox>,
    uniform_colour_tertiary: QPtr<QComboBox>,
    uniform_restore_initial_values_button: QPtr<QPushButton>,
    uniform_restore_vanilla_values_button: QPtr<QPushButton>,
    button_box: QPtr<QDialogButtonBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `ToolFactionPainter`.
impl ToolFactionPainter {

    /// This function creates a Tool with the data it needs.
    pub unsafe fn new(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Prepare the backend data.
        let paths = vec![PathType::Folder(vec!["db".to_owned()])];
        let tool = Tool::new(&paths);
        if let Err(error) = tool.backup_used_paths(app_ui, pack_file_contents_ui) {
            return show_dialog(&app_ui.main_window, error, false);
        }

        // Build the dialog.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("faction_painter_title"));
        dialog.set_modal(true);

        let mut data = vec!();
        let mut file = File::open(VIEW).unwrap();
        if let Err(error) = file.read_to_end(&mut data) {
            return show_dialog(&app_ui.main_window, error, false);
        }

        let ui_loader = QUiLoader::new_0a();
        let widget = ui_loader.load_bytes_with_parent(&data, &dialog);

        // Get the widgets from the view.

        // ListView.
        let faction_list_view: QPtr<QListView> = widget.find_child("faction_list_view").unwrap();
        let faction_list_filter_line_edit: QPtr<QLineEdit> = widget.find_child("faction_list_filter_line_edit").unwrap();

        // Details view.
        let faction_name_label: QPtr<QLabel> = widget.find_child("faction_name_label").unwrap();

        // Banner GroupBox.
        let banner_groupbox: QPtr<QGroupBox> = widget.find_child("banner_groupbox").unwrap();

        let banner_colour_primary_label: QPtr<QLabel> = widget.find_child("banner_colour_primary_label").unwrap();
        let banner_colour_secondary_label: QPtr<QLabel> = widget.find_child("banner_colour_secondary_label").unwrap();
        let banner_colour_tertiary_label: QPtr<QLabel> = widget.find_child("banner_colour_tertiary_label").unwrap();

        let banner_colour_primary: QPtr<QComboBox> = widget.find_child("banner_colour_primary").unwrap();
        let banner_colour_secondary: QPtr<QComboBox> = widget.find_child("banner_colour_secondary").unwrap();
        let banner_colour_tertiary: QPtr<QComboBox> = widget.find_child("banner_colour_tertiary").unwrap();

        let banner_restore_initial_values_button: QPtr<QPushButton> = widget.find_child("banner_restore_initial_values_button").unwrap();
        let banner_restore_vanilla_values_button: QPtr<QPushButton> = widget.find_child("banner_restore_vanilla_values_button").unwrap();

        // Uniform GroupBox.
        let uniform_groupbox: QPtr<QGroupBox> = widget.find_child("uniform_groupbox").unwrap();

        let uniform_colour_primary_label: QPtr<QLabel> = widget.find_child("uniform_colour_primary_label").unwrap();
        let uniform_colour_secondary_label: QPtr<QLabel> = widget.find_child("uniform_colour_secondary_label").unwrap();
        let uniform_colour_tertiary_label: QPtr<QLabel> = widget.find_child("uniform_colour_tertiary_label").unwrap();

        let uniform_colour_primary: QPtr<QComboBox> = widget.find_child("uniform_colour_primary").unwrap();
        let uniform_colour_secondary: QPtr<QComboBox> = widget.find_child("uniform_colour_secondary").unwrap();
        let uniform_colour_tertiary: QPtr<QComboBox> = widget.find_child("uniform_colour_tertiary").unwrap();

        let uniform_restore_initial_values_button: QPtr<QPushButton> = widget.find_child("uniform_restore_initial_values_button").unwrap();
        let uniform_restore_vanilla_values_button: QPtr<QPushButton> = widget.find_child("uniform_restore_vanilla_values_button").unwrap();

        // Button Box.
        let button_box: QPtr<QDialogButtonBox> = widget.find_child("button_box").unwrap();

        // Extra stuff.
        let faction_list_filter = QSortFilterProxyModel::new_1a(&faction_list_view);
        let faction_list_model = QStandardItemModel::new_1a(&faction_list_filter);
        faction_list_view.set_model(&faction_list_filter);
        faction_list_filter.set_source_model(&faction_list_model);

        // build the view.
        let view = Rc::new(Self{
            tool,
            dialog,
            faction_list_view,
            faction_list_filter,
            faction_list_model,
            faction_list_filter_line_edit,
            faction_name_label,
            banner_groupbox,
            banner_colour_primary_label,
            banner_colour_secondary_label,
            banner_colour_tertiary_label,
            banner_colour_primary,
            banner_colour_secondary,
            banner_colour_tertiary,
            banner_restore_initial_values_button,
            banner_restore_vanilla_values_button,
            uniform_groupbox,
            uniform_colour_primary_label,
            uniform_colour_secondary_label,
            uniform_colour_tertiary_label,
            uniform_colour_primary,
            uniform_colour_secondary,
            uniform_colour_tertiary,
            uniform_restore_initial_values_button,
            uniform_restore_vanilla_values_button,
            button_box,
        });

        let slots = slots::ToolFactionPainterSlots::new(app_ui, pack_file_contents_ui, &view);

        connections::set_connections(&view, &slots);
        //shortcuts::set_shortcuts(&view);

        CENTRAL_COMMAND.send_message_qt(Command::GetPackedFilesFromAllSources(vec!["db".to_owned(), "factions_tables".to_owned(), "data__".to_owned()]));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let mut data = if let Response::VecPackedFileDataSource(data) = response { data } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response); };

        //dbg!(&data);
        for (packed_file, source) in &mut data {
            let decoded = packed_file.decode_return_ref().unwrap();
            if let DecodedPackedFile::DB(table) = decoded {
                let key_column = table.get_ref_definition().get_fields_processed().iter().position(|x| x.get_name() == "key").unwrap();
                for row in table.get_ref_table_data() {
                    if let DecodedData::StringU8(key) = &row[key_column] {
                        let item = QStandardItem::from_q_string(&QString::from_std_str(&key)).into_ptr();
                        view.faction_list_model.append_row_q_standard_item(item)
                    }
                }
            }
        }

        view.faction_name_label.set_text(&qt_core::QString::from_std_str("test"));

        if view.dialog.exec() == 1 { } else { }
    }
}
