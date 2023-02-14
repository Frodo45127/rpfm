//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QAction;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QMenu;
use qt_widgets::QPushButton;
use qt_widgets::QTextEdit;
use qt_widgets::QWidget;

use qt_gui::QDesktopServices;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::ContextMenuPolicy;
use qt_core::QBox;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QUrl;

use qt_ui_tools::QUiLoader;

use anyhow::Result;
use getset::Getters;

use std::fs::File;
use std::io::{Read, BufReader};
use std::sync::{Arc, RwLock};

use rpfm_lib::notes::Note;

use crate::ASSETS_PATH;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::new_tips_item_delegate_safe;
use crate::locale::{qtr, tr};
use crate::utils::*;

mod connections;
mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_tip_dialog.ui";
const VIEW_RELEASE: &str = "ui/new_tip_dialog.ui";

/// List of roles for Quick Notes' data.
const ROLE_URL: i32 = 33;
const ROLE_ID: i32 = 34;
const ROLE_PATH: i32 = 35;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct NotesView {
    path: Arc<RwLock<String>>,
    list: QBox<QListView>,
    filter: QBox<QSortFilterProxyModel>,
    model: QBox<QStandardItemModel>,
    new_button: QBox<QPushButton>,

    context_menu: QBox<QMenu>,
    context_menu_edit: QPtr<QAction>,
    context_menu_delete: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl NotesView {

    pub unsafe fn new_view(tips_widget: &Arc<QBox<QWidget>>, path: Arc<RwLock<String>>) -> Arc<Self> {

        let layout: QPtr<QGridLayout> = tips_widget.layout().static_downcast();
        let list = QListView::new_1a(tips_widget.as_ptr());
        let filter = QSortFilterProxyModel::new_1a(tips_widget.as_ptr());
        let model = QStandardItemModel::new_1a(tips_widget.as_ptr());
        let new_button = QPushButton::from_q_string_q_widget(&qtr("new_tip"), tips_widget.as_ptr());
        list.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        list.set_model(&filter);
        filter.set_source_model(&model);

        layout.add_widget_5a(&list, 0, 0, 1, 1);
        layout.add_widget_5a(&new_button, 1, 0, 1, 1);

        new_tips_item_delegate_safe(&list.static_upcast::<QObject>().as_ptr(), true);

        let context_menu = QMenu::from_q_widget(&list);
        let context_menu_edit = context_menu.add_action_q_string(&qtr("tip_edit"));
        let context_menu_delete = context_menu.add_action_q_string(&qtr("tip_delete"));

        context_menu_edit.set_enabled(false);
        context_menu_delete.set_enabled(false);

        let view = Arc::new(Self {
            path,
            list,
            filter,
            model,
            new_button,
            context_menu,
            context_menu_edit,
            context_menu_delete,
        });

        let slots = slots::NotesSlots::new(&view);
        connections::set_connections(&view, &slots);
        view.load_data();

        view
    }

    /// This function loads all notes affecting the path of the view.
    pub unsafe fn load_data(&self) {
        self.model.clear();

        let receiver = CENTRAL_COMMAND.send_background(Command::NotesForPath(self.path.read().unwrap().to_owned()));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::VecNote(mut notes) => {
                if !notes.is_empty() {
                    notes.sort_by_key(|note| *note.id());
                    notes.iter().for_each(|note| self.add_item_to_notes_list(note));

                    let parent = self.list.parent().static_downcast::<QWidget>();
                    let grandparent = parent.parent().static_downcast::<QWidget>();
                    let layout = grandparent.layout().static_downcast::<QGridLayout>();
                    parent.set_visible(true);
                    layout.add_widget_5a(parent, 0, 99, layout.row_count(), 1);
                }
            },
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };
    }

    /// This function saves a note on the currently open Pack.
    pub unsafe fn save_data(&self, note: Note) {

        let receiver = CENTRAL_COMMAND.send_background(Command::AddNote(note));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::Note(note) => self.add_item_to_notes_list(&note),
            Response::Error(error) => show_dialog(&self.list, error, false),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };
    }

    /// This function loads the new note dialog, and returns an error if it fails for any reason.
    pub unsafe fn load_new_note_dialog(&self, edit: bool) -> Result<()> {

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let template_path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), view);
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, &self.list);

        let tip_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "tip_label")?;
        let path_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "path_label")?;
        let link_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "link_label")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let link_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "link_line_edit")?;
        let path_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "path_line_edit")?;
        let tip_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "tip_text_edit")?;

        let dialog = main_widget.static_downcast::<QDialog>();
        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_close());
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Setup data.
        if edit {
            let note = self.note_from_selection();
            link_line_edit.set_text(&QString::from_std_str(note.url().clone().unwrap_or("".to_string())));
            tip_text_edit.set_text(&QString::from_std_str(note.message()));
            path_line_edit.set_text(&QString::from_std_str(note.path()));
        } else {
            path_line_edit.set_text(&QString::from_std_str(self.path.read().unwrap().to_string()));
        }

        // Setup translations.
        dialog.set_window_title(&qtr("new_tip_dialog"));
        tip_label.set_text(&qtr("new_tip_tip"));
        path_label.set_text(&qtr("new_tip_path"));
        link_label.set_text(&qtr("new_tip_link"));

        // If we hit accept, build the tip and save it.
        if dialog.exec() == 1 {
            let mut note = Note::default();
            note.set_message(tip_text_edit.to_plain_text().to_std_string());

            let url = link_line_edit.text().to_std_string();
            if url.is_empty() {
                note.set_url(None);
            } else {
                note.set_url(Some(url));
            }

            note.set_path(path_line_edit.text().to_std_string());

            // If it's an edit, overwrite the default id with the old one.
            if edit {
                let old_note = self.note_from_selection();
                note.set_id(*old_note.id());

                let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
                self.model.remove_row_1a(indexes.at(0).row());
            }

            self.save_data(note);
        }

        Ok(())
    }

    /// This function adds a new note to the notes list.
    unsafe fn add_item_to_notes_list(&self, note: &Note) {
        let item = QStandardItem::new();

        item.set_editable(false);
        item.set_text(&QString::from_std_str(note.message()));
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(note.url().clone().unwrap_or("".to_string()))), ROLE_URL);
        item.set_data_2a(&QVariant::from_u64(*note.id()), ROLE_ID);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(note.path())), ROLE_PATH);

        let mut tooltip = String::new();
        tooltip.push_str(&tr("tip_id"));
        tooltip.push_str(&note.id().to_string());
        tooltip.push('\n');

        if let Some(url) = note.url() {
            tooltip.push_str(&tr("tip_link"));
            tooltip.push_str(url);
            tooltip.push('\n');
        }
        item.set_tool_tip(&QString::from_std_str(&tooltip));

        self.model.append_row_q_standard_item(item.into_raw_ptr())
    }

    /// This function builds a note from the selected item.
    unsafe fn note_from_selection(&self) -> Note {
        let mut note = Note::default();

        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let index = indexes.at(0);
        note.set_id(self.model.data_2a(index, ROLE_ID).to_u_long_long_0a());

        let url = self.model.data_2a(index, ROLE_URL).to_string().to_std_string();
        if url.is_empty() {
            note.set_url(None);
        } else {
            note.set_url(Some(url));
        }

        note.set_path(self.model.data_2a(index, ROLE_PATH).to_string().to_std_string());
        note.set_message(self.model.data_2a(index, 2).to_string().to_std_string());

        note
    }

    unsafe fn context_menu_update(&self) {
        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let enabled = indexes.count_0a() == 1;

        self.context_menu_edit.set_enabled(enabled);
        self.context_menu_delete.set_enabled(enabled);
    }

    /// This function deletes a note from both, the ui and the backend.
    unsafe fn delete_selected_note(&self) {
        let note = self.note_from_selection();

        let _ = CENTRAL_COMMAND.send_background(Command::DeleteNote(note.path().to_owned(), *note.id()));
        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        self.model.remove_row_1a(indexes.at(0).row());
    }

    /// This function tries to open a link associated with the clicked item on the list.
    ///
    /// Does nothing if the note contains no link.
    unsafe fn open_link(&self) {
        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let index = indexes.at(0);
        let link = self.model.data_2a(index, ROLE_URL).to_string();

        if !link.is_empty() {
            QDesktopServices::open_url(&QUrl::new_1a(&link));
        }
    }
}
