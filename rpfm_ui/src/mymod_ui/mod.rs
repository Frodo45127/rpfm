//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::{q_dialog_button_box, QDialogButtonBox};
use qt_widgets::QFrame;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;

use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QString;
use qt_core::QPtr;

use std::rc::Rc;

use rpfm_lib::GAME_SELECTED;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::AppUI;
use crate::locale::qtr;
use crate::utils::create_grid_layout;
use self::slots::MyModUISlots;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// `This struct holds all the relevant stuff for "MyMod"'s New Mod Window.
pub struct MyModUI {
    pub mymod_dialog: QBox<QDialog>,
    pub mymod_game_combobox: QBox<QComboBox>,
    pub mymod_name_line_edit: QBox<QLineEdit>,
    pub mymod_cancel_button: QPtr<QPushButton>,
    pub mymod_accept_button: QPtr<QPushButton>,
}


//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `MyModUI`.
impl MyModUI {

    /// This function creates the entire "New Mod" dialog and executes it. It returns
    /// the name of the mod and the folder_name of the game.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Option<(String, String)> {

        // Create the "New MyMod" Dialog and configure it.
        let dialog = QDialog::new_1a(&app_ui.main_window);
        let main_grid = create_grid_layout(dialog.static_upcast());
        dialog.set_window_title(&qtr("mymod_new"));
        dialog.set_modal(true);
        dialog.resize_2a(300, 0);

        // Create the Advices Frame and configure it.
        let advices_frame = QFrame::new_0a();
        let advices_grid = create_grid_layout(advices_frame.static_upcast());
        let advices_label = QLabel::from_q_string(&QString::from_std_str("Things to take into account before creating a new mod:
    - Select the game you'll make the mod for.
    - Pick an simple name (it shouldn't end in *.pack).
    - If you want to use multiple words, use \"_\" instead of \" \".
    - You can't create a mod for a game that has no path set in the settings."));

        advices_grid.add_widget_5a(&advices_label, 0, 0, 1, 1);
        main_grid.add_widget_5a(advices_frame.into_ptr(), 0, 0, 1, 2);

        // Create the "MyMod's Name" Label and LineEdit and configure them.
        let mymod_name_label = QLabel::from_q_string(&qtr("mymod_name"));
        let mymod_name_line_edit = QLineEdit::new();
        mymod_name_line_edit.set_placeholder_text(&qtr("mymod_name_default"));

        // Create the "MyMod's Game" Label and ComboBox and configure them.
        let mymod_game_label = QLabel::from_q_string(&qtr("mymod_game"));
        let mymod_game_combobox = QComboBox::new_0a();
        let mymod_game_model = QStandardItemModel::new_0a();
        mymod_game_combobox.set_model(&mymod_game_model);

        // Add the games to the ComboBox.
        let mut selected_index = 0;
        let mut selected_index_counter = 0;
        let game_selected = GAME_SELECTED.read().unwrap().get_game_key_name();
        for game in SUPPORTED_GAMES.get_games() {
            if game.get_supports_editing() {
                mymod_game_combobox.add_item_q_string(&QString::from_std_str(&game.get_display_name()));

                if game.get_game_key_name() == game_selected {
                    selected_index = selected_index_counter
                }
                selected_index_counter += 1;
            }
        }
        mymod_game_combobox.set_current_index(selected_index as i32);

        // Add all the widgets to the main grid.
        main_grid.add_widget_5a(&mymod_name_label, 1, 0, 1, 1);
        main_grid.add_widget_5a(&mymod_name_line_edit, 1, 1, 1, 1);

        main_grid.add_widget_5a(&mymod_game_label, 2, 0, 1, 1);
        main_grid.add_widget_5a(&mymod_game_combobox, 2, 1, 1, 1);

        // Create the bottom ButtonBox and configure it
        let button_box = QDialogButtonBox::new();
        let mymod_cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let mymod_accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);
        main_grid.add_widget_5a(&button_box, 3, 0, 1, 2);

        // Disable the "Accept" button by default.
        mymod_accept_button.set_enabled(false);

        // Put all the stuff together and launch the dialog.
        let mymod_ui = Rc::new(Self {
            mymod_dialog: dialog,
            mymod_game_combobox,
            mymod_name_line_edit,
            mymod_cancel_button,
            mymod_accept_button,
        });

        let mymod_slots = MyModUISlots::new(&mymod_ui);
        connections::set_connections(&mymod_ui, &mymod_slots);

        // Execute the dialog and return the result if we accepted.
        if mymod_ui.mymod_dialog.exec() == 1 {
            let mod_name = mymod_ui.mymod_name_line_edit.text().to_std_string();
            let mut game = mymod_ui.mymod_game_combobox.current_text().to_std_string();
            if let Some(index) = game.find('&') { game.remove(index); }
            let mod_game = game.replace(' ', "_").to_lowercase();
            Some((mod_name, mod_game))
        }

        // If we cancelled/closed it, return `None`.
        else { None }
    }

    /// This function checks if the MyMod's name is valid or not, disabling or enabling the "Accept" button in response.
    unsafe fn check_my_mod_validity(&self) {
        let mod_name = self.mymod_name_line_edit.text().to_std_string();
        let mut game = self.mymod_game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        let mod_game = game.replace(' ', "_").to_lowercase();

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        if let Some(ref mod_path) = SETTINGS.read().unwrap().paths["mymods_base_path"] {

            // If there is text and it doesn't have whitespace...
            if !mod_name.is_empty() && !mod_name.contains(' ') {
                let mut mod_path = mod_path.clone();
                mod_path.push(mod_game);
                mod_path.push(format!("{}.pack", mod_name));

                // If a mod with that name for that game already exists, disable the "Accept" button.
                if mod_path.is_file() { self.mymod_accept_button.set_enabled(false);}

                // If the name is available, enable the `Accept` button.
                else { self.mymod_accept_button.set_enabled(true); }
            }

            // If name is empty, disable the button.
            else { self.mymod_accept_button.set_enabled(false); }
        }

        // If there is no "MyMod" path configured, disable the button.
        else { self.mymod_accept_button.set_enabled(false); }
    }
}
