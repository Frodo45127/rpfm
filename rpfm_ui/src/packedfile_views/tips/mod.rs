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
Module with all the code for managing Tips views.
!*/

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

use std::fs::File;
use std::io::{Read, BufReader};
use std::sync::{Arc, RwLock};

use rpfm_error::Result;
use rpfm_lib::tips::*;

use crate::ASSETS_PATH;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::new_tips_item_delegate_safe;
use crate::locale::{qtr, tr};
use crate::utils::*;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_tip_dialog.ui";
const VIEW_RELEASE: &str = "ui/new_tip_dialog.ui";

/// List of roles for tips.
const ROLE_IS_REMOTE: i32 = 30;
const ROLE_USER: i32 = 31;
const ROLE_TIMESTAMP: i32 = 32;
const ROLE_URL: i32 = 33;
const ROLE_ID: i32 = 34;
const ROLE_PATH: i32 = 35;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the Tips panel.
pub struct TipsView {
    path: Arc<RwLock<Vec<String>>>,
    list: QBox<QListView>,
    filter: QBox<QSortFilterProxyModel>,
    model: QBox<QStandardItemModel>,
    new_button: QBox<QPushButton>,

    context_menu: QBox<QMenu>,
    context_menu_edit: QPtr<QAction>,
    context_menu_delete: QPtr<QAction>,
    context_menu_publish: QPtr<QAction>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `TipsView`.
impl TipsView {

    /// This function creates a new TipsView, and sets up his slots and connections.
    pub unsafe fn new_view(tips_widget: &Arc<QBox<QWidget>>, path: &[String]) -> Arc<Self> {

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
        let context_menu_publish = context_menu.add_action_q_string(&qtr("tip_publish"));

        context_menu_edit.set_enabled(false);
        context_menu_delete.set_enabled(false);
        context_menu_publish.set_enabled(false);

        let view = Arc::new(Self {
            path: Arc::new(RwLock::new(path.to_vec())),
            list,
            filter,
            model,
            new_button,
            context_menu,
            context_menu_edit,
            context_menu_delete,
            context_menu_publish
        });

        let slots = slots::TipSlots::new(&view);
        connections::set_connections(&view, &slots);

        if !path.is_empty() {
            view.load_data(path);
        }

        view
    }

    /// This function loads data for the provided path to the view.
    pub unsafe fn load_data(&self, path: &[String]) {

        // For tables, share tips between same-type tables.
        *self.path.write().unwrap() = if path.len() > 2 && path[0].to_lowercase() == "db" { path[0..=1].to_vec() } else { path.to_vec() };
        self.model.clear();

        let receiver = CENTRAL_COMMAND.send_background(Command::GetTipsForPath(self.path.read().unwrap().to_vec()));
        let response = CentralCommand::recv(&receiver);
        let (local_tips, remote_tips) = match response {
            Response::VecTipVecTip(local_tips, remote_tips) => (local_tips, remote_tips),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        if !remote_tips.is_empty() || !local_tips.is_empty() {
            remote_tips.iter().for_each(|tip| self.add_item_to_tip_list(tip, true));
            local_tips.iter().for_each(|tip| self.add_item_to_tip_list(tip, false));

            let parent = self.list.parent().static_downcast::<QWidget>();
            let grandparent = parent.parent().static_downcast::<QWidget>();
            let layout = grandparent.layout().static_downcast::<QGridLayout>();
            parent.set_visible(true);
            layout.add_widget_5a(parent, 0, 99, layout.row_count(), 1);
        }
    }

    /// This function saves a tip to the local tips list for the current path.
    pub unsafe fn save_data(&self, tip: Tip) {
        self.add_item_to_tip_list(&tip, false);

        let receiver = CENTRAL_COMMAND.send_background(Command::AddTipToLocalTips(tip));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::Success => {},
            Response::Error(error) => show_dialog(&self.list, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };
    }

    /// This function loads the new tip dialog, and returns an error if it fails for any reason.
    pub unsafe fn load_new_tip_dialog(&self, edit: bool) -> Result<()> {

        let view = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let template_path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), view);
        let mut data = vec!();
        let mut file = BufReader::new(File::open(template_path)?);
        file.read_to_end(&mut data)?;

        let ui_loader = QUiLoader::new_0a();
        let main_widget = ui_loader.load_bytes_with_parent(&data, &self.list);

        let user_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "user_label")?;
        let tip_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "tip_label")?;
        let path_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "path_label")?;
        let link_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "link_label")?;

        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let user_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "user_line_edit")?;
        let link_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "link_line_edit")?;
        let path_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "path_line_edit")?;
        let tip_text_edit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "tip_text_edit")?;

        let dialog = main_widget.static_downcast::<QDialog>();
        button_box.button(StandardButton::Cancel).released().connect(dialog.slot_close());
        button_box.button(StandardButton::Ok).released().connect(dialog.slot_accept());

        // Setup data.
        if edit {
            let tip = self.get_tip_from_selection();
            user_line_edit.set_text(&QString::from_std_str(tip.get_ref_user()));
            link_line_edit.set_text(&QString::from_std_str(tip.get_ref_url()));
            tip_text_edit.set_text(&QString::from_std_str(tip.get_ref_message()));
            path_line_edit.set_text(&QString::from_std_str(tip.get_ref_path()));
        } else {
            path_line_edit.set_text(&QString::from_std_str(self.path.read().unwrap().join("/")));
            user_line_edit.set_text(&QString::from_std_str("Daniel, the Demon Prince"));
        }

        // Setup translations.
        dialog.set_window_title(&qtr("new_tip_dialog"));
        user_label.set_text(&qtr("new_tip_user"));
        tip_label.set_text(&qtr("new_tip_tip"));
        path_label.set_text(&qtr("new_tip_path"));
        link_label.set_text(&qtr("new_tip_link"));

        // If we hit accept, build the tip and save it.
        if dialog.exec() == 1 {
            let mut tip = Tip::default();
            tip.set_user(user_line_edit.text().to_std_string());
            tip.set_message(tip_text_edit.to_plain_text().to_std_string());
            tip.set_url(link_line_edit.text().to_std_string());
            tip.set_path(path_line_edit.text().to_std_string());

            // If it's an edit, overwrite the default id with the old one.
            if edit {
                let old_tip = self.get_tip_from_selection();
                tip.set_id(*old_tip.get_ref_id());

                let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
                self.model.remove_row_1a(indexes.at(0).row());
            }

            self.save_data(tip);
        }

        Ok(())
    }

    /// This function adds a new tip to the tip list.
    unsafe fn add_item_to_tip_list(&self, tip: &Tip, is_remote: bool) {
        let item = QStandardItem::new();

        item.set_editable(false);
        item.set_text(&QString::from_std_str(tip.get_ref_message()));
        item.set_data_2a(&QVariant::from_bool(is_remote), ROLE_IS_REMOTE);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(tip.get_ref_user())), ROLE_USER);
        item.set_data_2a(&QVariant::from_u64(*tip.get_ref_timestamp() as u64), ROLE_TIMESTAMP);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(tip.get_ref_url())), ROLE_URL);
        item.set_data_2a(&QVariant::from_u64(*tip.get_ref_id()), ROLE_ID);
        item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(tip.get_ref_path())), ROLE_PATH);

        let mut tooltip = String::new();
        tooltip.push_str(&tr("tip_id"));
        tooltip.push_str(&tip.get_ref_id().to_string());
        tooltip.push('\n');

        if !tip.get_ref_user().is_empty() {
            tooltip.push_str(&tr("tip_author"));
            tooltip.push_str(tip.get_ref_user());
            tooltip.push('\n');
        }
        if !tip.get_ref_url().is_empty() {
            tooltip.push_str(&tr("tip_link"));
            tooltip.push_str(tip.get_ref_url());
            tooltip.push('\n');
        }
        item.set_tool_tip(&QString::from_std_str(&tooltip));

        self.model.append_row_q_standard_item(item.into_raw_ptr())
    }

    /// This function builds a tip from the selected item.
    unsafe fn get_tip_from_selection(&self) -> Tip {
        let mut tip = Tip::default();

        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let index = indexes.at(0);
        tip.set_id(self.model.data_2a(index, ROLE_ID).to_u_long_long_0a());
        tip.set_user(self.model.data_2a(index, ROLE_USER).to_string().to_std_string());
        tip.set_url(self.model.data_2a(index, ROLE_URL).to_string().to_std_string());
        tip.set_path(self.model.data_2a(index, ROLE_PATH).to_string().to_std_string());
        tip.set_message(self.model.data_2a(index, 2).to_string().to_std_string());
        tip.set_timestamp(self.model.data_2a(index, ROLE_TIMESTAMP).to_u_long_long_0a() as u128);

        tip
    }

    unsafe fn context_menu_update(&self) {

        // TODO: This fails a lot. Fix it.
        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let enabled = if indexes.count_0a() == 1 {
            let index = indexes.take_at(0);
            let is_remote = self.model.data_2a(&index, ROLE_IS_REMOTE).to_bool();
            if !is_remote {
                true
            } else {
                false
            }
        } else {
            false
        };

        self.context_menu_edit.set_enabled(enabled);
        self.context_menu_delete.set_enabled(enabled);
        self.context_menu_publish.set_enabled(enabled);
    }

    /// This function deletes a tip from both, the ui and the backend.
    unsafe fn delete_selected_tip(&self) {
        let tip = self.get_tip_from_selection();

        let receiver = CENTRAL_COMMAND.send_background(Command::DeleteTipById(*tip.get_ref_id()));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::Success => {},
            Response::Error(error) => show_dialog(&self.list, error, false),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        self.model.remove_row_1a(indexes.at(0).row());
    }

    /// This function tries to open a link associated with the clicked item on the list.
    unsafe fn open_link(&self) {
        let indexes = self.filter.map_selection_to_source(&self.list.selection_model().selection()).indexes();
        let index = indexes.at(0);
        let link = self.model.data_2a(index, ROLE_URL).to_string();

        if !link.is_empty() {
            QDesktopServices::open_url(&QUrl::new_1a(&link));
        }
    }

    /// This function takes care of pulishing the selected tip to github.
    unsafe fn publish_tip(&self) -> Result<()> {
        let tip = self.get_tip_from_selection();

        let receiver = CENTRAL_COMMAND.send_background(Command::PublishTipById(*tip.get_ref_id()));
        let response = CentralCommand::recv(&receiver);
        match response {
            Response::Success => Ok(()),
            Response::Error(error) => Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        }
    }
}
