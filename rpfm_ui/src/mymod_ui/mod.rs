//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This module contains the code to build/use the ***Settings*** UI.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QGroupBox;
use qt_widgets::{q_dialog_button_box, QDialogButtonBox};
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QTextEdit;
use qt_widgets::QWidget;

use qt_gui::QStandardItemModel;

use qt_core::QString;
use qt_core::QPtr;

use anyhow::Result;
use getset::Getters;

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::ffi::*;
use crate::GAME_SELECTED;
use crate::locale::{qtr, tr};
use crate::settings_ui::backend::*;
use crate::SUPPORTED_GAMES;
use crate::utils::*;

use self::slots::MyModUISlots;

mod connections;
mod slots;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/new_mymod_dialog.ui";
const VIEW_RELEASE: &str = "ui/new_mymod_dialog.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// `This struct holds all the relevant stuff for "MyMod"'s New Mod Window.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct MyModUI {
    dialog: QPtr<QDialog>,
    git_support_groupbox: QPtr<QGroupBox>,
    sublime_support_checkbox: QPtr<QCheckBox>,
    vscode_support_checkbox: QPtr<QCheckBox>,
    gitignore_same_as_files_ignored_on_import_checkbox: QPtr<QCheckBox>,
    gitignore_contents_textedit: QPtr<QTextEdit>,
    pack_import_ignore_contents_textedit: QPtr<QTextEdit>,
    game_combobox: QPtr<QComboBox>,
    name_line_edit: QPtr<QLineEdit>,
    message_widget: QPtr<QWidget>,
    button_box: QPtr<QDialogButtonBox>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `MyModUI`.
impl MyModUI {

    /// This function creates the entire "New Mod" dialog and executes it. It returns
    /// the name of the mod and the folder_name of the game.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<Option<(String, String, bool, bool, String, Option<String>)>> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        // Get the common widgets for all tools.
        let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
        let lua_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "lua_groupbox")?;
        let sublime_support_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "sublime_support_checkbox")?;
        let vscode_support_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "vscode_support_checkbox")?;
        let git_support_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "git_support_groupbox")?;
        let gitignore_same_as_files_ignored_on_import_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "gitignore_same_as_files_ignored_on_import_checkbox")?;
        let gitignore_contents_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "gitignore_contents_label")?;
        let gitignore_contents_textedit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "gitignore_contents_textedit")?;
        let pack_import_ignore_contents_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "pack_import_ignore_contents_label")?;
        let pack_import_ignore_contents_textedit: QPtr<QTextEdit> = find_widget(&main_widget.static_upcast(), "pack_import_ignore_contents_textedit")?;

        let game_combobox: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "game_combobox")?;
        let name_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "name_label")?;
        let name_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
        let message_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "message_widget")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;

        // Close the message widget, as by default is open.
        kmessage_widget_close_safe(&message_widget.as_ptr());

        // Configure the dialog.
        let dialog: QPtr<QDialog> = main_widget.static_downcast();
        dialog.set_window_title(&qtr("mymod_new"));
        //dialog.set_attribute_1a(WidgetAttribute::WADeleteOnClose);        // This crashes the program if we try to grab data from the dialog after the Execute.

        instructions_label.set_text(&qtr("new_mymod_instructions"));
        lua_groupbox.set_title(&qtr("new_mymod_lua_support"));
        git_support_groupbox.set_title(&qtr("new_mymod_git_support"));
        sublime_support_checkbox.set_text(&qtr("new_mymod_sublime_support"));
        vscode_support_checkbox.set_text(&qtr("new_mymod_vscode_support"));
        gitignore_contents_label.set_text(&qtr("new_mymod_gitignore_contents"));
        gitignore_same_as_files_ignored_on_import_checkbox.set_text(&qtr("new_mymod_gitignore_same_as_files_ignored_on_import"));
        pack_import_ignore_contents_label.set_text(&qtr("new_mymod_pack_import_ignore_contents"));
        name_line_edit.set_placeholder_text(&qtr("mymod_name_default"));
        name_label.set_text(&qtr("mymod_name"));
        pack_import_ignore_contents_textedit.set_placeholder_text(&qtr("new_mymod_pack_import_ignore_contents_placeholder"));
        gitignore_contents_textedit.set_placeholder_text(&qtr("new_mymod_gitignore_contents_placeholder"));

        let game_model = QStandardItemModel::new_0a();
        game_combobox.set_model(&game_model);

        // Add the games to the ComboBox.
        let mut selected_index = 0;
        let mut selected_index_counter = 0;
        let game_selected = GAME_SELECTED.read().unwrap().game_key_name();
        for game in SUPPORTED_GAMES.games_sorted() {
            if game.supports_editing() {
                game_combobox.add_item_q_string(&QString::from_std_str(game.display_name()));

                if game.game_key_name() == game_selected {
                    selected_index = selected_index_counter
                }
                selected_index_counter += 1;
            }
        }
        game_combobox.set_current_index(selected_index);

        // Disable the "Accept" button by default.
        button_box.button(q_dialog_button_box::StandardButton::Ok).set_enabled(false);

        // Put all the stuff together and launch the dialog.
        let mymod_ui = Rc::new(Self {
            dialog,
            git_support_groupbox,
            sublime_support_checkbox,
            vscode_support_checkbox,
            gitignore_same_as_files_ignored_on_import_checkbox,
            gitignore_contents_textedit,
            pack_import_ignore_contents_textedit,
            game_combobox,
            name_line_edit,
            message_widget,
            button_box,
        });

        let mymod_slots = MyModUISlots::new(&mymod_ui);
        connections::set_connections(&mymod_ui, &mymod_slots);

        // Execute the dialog and return the result if we accepted.
        let result = mymod_ui.dialog.exec();
        if result == 1 {
            let mut game = mymod_ui.game_combobox.current_text().to_std_string();
            if let Some(index) = game.find('&') { game.remove(index); }
            let mod_game = game.replace(' ', "_").to_lowercase();
            let mod_name = mymod_ui.name_line_edit.text().to_std_string();

            let mut autoignored_paths = String::new();

            // Lua stuff
            if mymod_ui.sublime_support_checkbox.is_checked() {
                autoignored_paths.push_str(&format!("\n{}.sublime-project\n{}.sublime-workspace", mod_name, mod_name));
            }
            if mymod_ui.vscode_support_checkbox.is_checked() {
                autoignored_paths.push_str("\n.vscode");
            }
            if mymod_ui.sublime_support_checkbox.is_checked() || mymod_ui.vscode_support_checkbox.is_checked() {
                autoignored_paths.push_str("\n.luarc.json");
            }

            // Git stuff.
            if mymod_ui.git_support_groupbox.is_checked() {
                autoignored_paths.push_str("\n.git");
            }

            let mut pack_import_ignore_paths = mymod_ui.pack_import_ignore_contents_textedit().to_plain_text().to_std_string();
            pack_import_ignore_paths.push_str(&autoignored_paths);

            let gitignore_contents = if mymod_ui.git_support_groupbox.is_checked() {
                if mymod_ui.gitignore_same_as_files_ignored_on_import_checkbox.is_checked() {
                    Some(pack_import_ignore_paths.to_string())
                } else {
                    let mut gitignore = mymod_ui.gitignore_contents_textedit().to_plain_text().to_std_string();
                    gitignore.push_str(&autoignored_paths);
                    Some(gitignore)
                }
            } else { None };

            Ok(Some((mod_name, mod_game, mymod_ui.sublime_support_checkbox.is_checked(), mymod_ui.vscode_support_checkbox.is_checked(), pack_import_ignore_paths, gitignore_contents)))
        }

        // If we cancelled/closed it, return `None`.
        else { Ok(None) }
    }

    /// Function to update the dialog depending on options selected.
    unsafe fn update_dialog(&self) {

        // Code to disable the gitignore textedit if we checked the "Same as import" checkbox.
        self.gitignore_contents_textedit.set_enabled(!self.gitignore_same_as_files_ignored_on_import_checkbox.is_checked());

        // Code to enable/disable the accept button depending on if we inputted a valid name or not.
        let mod_name = self.name_line_edit.text().to_std_string();
        let mut game = self.game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        let mod_game = game.replace(' ', "_").to_lowercase();

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        let mut mod_path = setting_path(MYMOD_BASE_PATH);
        if mod_path.is_dir() {

            if mod_name.contains(' ') {
                if kmessage_widget_is_closed_safe(&self.message_widget().as_ptr()) {
                    show_message_error(self.message_widget(), tr("mymod_error_spaces_on_name"));
                }
            } else {
                kmessage_widget_close_safe(&self.message_widget().as_ptr());
            }

            // If there is text and it doesn't have whitespace...
            if !mod_name.is_empty() && !mod_name.contains(' ') {
                mod_path.push(mod_game);
                mod_path.push(format!("{}.pack", mod_name));

                if !mod_path.is_file() { self.button_box().button(q_dialog_button_box::StandardButton::Ok).set_enabled(true);}
                else { self.button_box().button(q_dialog_button_box::StandardButton::Ok).set_enabled(false); }
            }

            // If name is empty or contains spaces, disable the button. Also, if it contains spaces throw a warning.
            else {
                self.button_box().button(q_dialog_button_box::StandardButton::Ok).set_enabled(false);
            }
        }

        // If there is no "MyMod" path configured, disable the button.
        else {
            self.button_box().button(q_dialog_button_box::StandardButton::Ok).set_enabled(false);
        }
    }
}
