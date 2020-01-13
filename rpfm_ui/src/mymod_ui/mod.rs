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
This module contains the code to build/use the ***Settings*** UI.
!*/

use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::{dialog_button_box, dialog_button_box::DialogButtonBox};
use qt_widgets::frame::Frame;
use qt_widgets::label::Label;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use cpp_utils::StaticCast;

use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::AppUI;
use crate::QString;
use crate::utils::create_grid_layout_unsafe;
use self::slots::MyModUISlots;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// `This struct holds all the relevant stuff for "MyMod"'s New Mod Window.
#[derive(Copy, Clone, Debug)]
pub struct MyModUI {
    pub mymod_dialog: *mut Dialog,
    pub mymod_game_combobox: *mut ComboBox,
    pub mymod_name_line_edit: *mut LineEdit,
    pub mymod_cancel_button: *mut PushButton,
    pub mymod_accept_button: *mut PushButton,
}


//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `MyModUI`.
impl MyModUI {

    /// This function creates the entire "New Mod" dialog and executes it. It returns
    /// the name of the mod and the folder_name of the game.
    pub fn new(app_ui: &AppUI) -> Option<(String, String)> {

        // Create the "New MyMod" Dialog and configure it.
        let mut dialog = unsafe { Dialog::new_unsafe(app_ui.main_window as *mut Widget) };
        let main_grid = create_grid_layout_unsafe(dialog.static_cast_mut() as *mut Widget);
        dialog.set_window_title(&qtr("mymod_new"));
        dialog.set_modal(true);
        dialog.resize((300, 0));

        // Create the Advices Frame and configure it.
        let advices_frame = Frame::new();
        let advices_grid = create_grid_layout_unsafe(advices_frame.as_mut_ptr() as *mut Widget);
        let advices_label = Label::new(&QString::from_std_str("Things to take into account before creating a new mod:
    - Select the game you'll make the mod for.	
    - Pick an simple name (it shouldn't end in *.pack).	
    - If you want to use multiple words, use \"_\" instead of \" \".	
    - You can't create a mod for a game that has no path set in the settings."));

        unsafe {
            advices_grid.as_mut().unwrap().add_widget((advices_label.into_raw() as *mut Widget, 0, 0, 1, 1));
            main_grid.as_mut().unwrap().add_widget((advices_frame.into_raw() as *mut Widget, 0, 0, 1, 2));
        }

        // Create the "MyMod's Name" Label and LineEdit and configure them.
        let mymod_name_label = Label::new(&qtr("mymod_name"));
        let mut mymod_name_line_edit = LineEdit::new(());
        mymod_name_line_edit.set_placeholder_text(&qtr("mymod_name_default"));

        // Create the "MyMod's Game" Label and ComboBox and configure them.
        let mymod_game_label = Label::new(&qtr("mymod_game"));
        let mut mymod_game_combobox = ComboBox::new();
        let mut mymod_game_model = StandardItemModel::new(());
        unsafe { mymod_game_combobox.set_model(mymod_game_model.static_cast_mut()); }

        // Add the games to the ComboBox.
        for (_, game) in SUPPORTED_GAMES.iter() {
            if game.supports_editing {
                mymod_game_combobox.add_item(&QString::from_std_str(&game.display_name));
            }
        }

        // Add all the widgets to the main grid.
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_label.into_raw() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_line_edit.as_mut_ptr() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_label.into_raw() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_combobox.as_mut_ptr() as *mut Widget, 2, 1, 1, 1)); }

        // Create the bottom ButtonBox and configure it
        let mut button_box = DialogButtonBox::new(());
        let mymod_cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        let mymod_accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.into_raw() as *mut Widget, 3, 0, 1, 2)); }

        // Disable the "Accept" button by default.
        unsafe { mymod_accept_button.as_mut().unwrap().set_enabled(false); }

        // Put all the stuff together and launch the dialog.
        let mymod_ui = Self {
            mymod_dialog: dialog.into_raw(),
            mymod_game_combobox: mymod_game_combobox.into_raw(),
            mymod_name_line_edit: mymod_name_line_edit.into_raw(),
            mymod_cancel_button,
            mymod_accept_button,
        };

        let mymod_slots = MyModUISlots::new(mymod_ui);
        connections::set_connections(&mymod_ui, &mymod_slots);

        // Execute the dialog and return the result if we accepted.
        if unsafe { mymod_ui.mymod_dialog.as_mut().unwrap().exec() } == 1 {
            let mod_name = unsafe { mymod_ui.mymod_name_line_edit.as_mut().unwrap().text().to_std_string() };
            let mut game = unsafe { mymod_ui.mymod_game_combobox.as_mut().unwrap().current_text().to_std_string() };
            if let Some(index) = game.find('&') { game.remove(index); }
            let mod_game = game.replace(' ', "_").to_lowercase();
            Some((mod_name, mod_game))
        }

        // If we cancelled/closed it, return `None`.
        else { None }
    }

    /// This function checks if the MyMod's name is valid or not, disabling or enabling the "Accept" button in response.
    fn check_my_mod_validity(&self) {
        let mod_name = unsafe { self.mymod_name_line_edit.as_mut().unwrap().text().to_std_string() };
        let mut game = unsafe { self.mymod_game_combobox.as_mut().unwrap().current_text().to_std_string() };
        if let Some(index) = game.find('&') { game.remove(index); }
        let mod_game = game.replace(' ', "_").to_lowercase();

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        if let Some(ref mod_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

            // If there is text and it doesn't have whitespaces...
            if !mod_name.is_empty() && !mod_name.contains(' ') {
                let mut mod_path = mod_path.clone();
                mod_path.push(mod_game);
                mod_path.push(format!("{}.pack", mod_name));

                // If a mod with that name for that game already exists, disable the "Accept" button.
                if mod_path.is_file() { unsafe { self.mymod_accept_button.as_mut().unwrap().set_enabled(false); }}

                // If the name is available, enable the `Accept` button.
                else { unsafe { self.mymod_accept_button.as_mut().unwrap().set_enabled(true); } }
            }

            // If name is empty, disable the button.
            else { unsafe { self.mymod_accept_button.as_mut().unwrap().set_enabled(false); } }
        }

        // If there is no "MyMod" path configured, disable the button.
        else { unsafe { self.mymod_accept_button.as_mut().unwrap().set_enabled(false); } }
    }
}
