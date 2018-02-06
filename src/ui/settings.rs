// Here it goes all the stuff related with the settings window.
extern crate gtk;

use std::path::{
    Path, PathBuf
};
use gtk::prelude::*;
use gdk::Gravity;
use gtk::{
    Entry, Box, Button, Frame, ComboBoxText, ApplicationWindow, WindowPosition, Orientation,
    Label, ButtonBox, ButtonBoxStyle, Application
};
use settings::Settings;

/// This struct holds all the relevant stuff for the Settings Window.
#[derive(Clone, Debug)]
pub struct SettingsWindow {
    pub settings_window: ApplicationWindow,
    pub settings_path_my_mod_entry: Entry,
    pub settings_path_my_mod_button: Button,
    pub settings_path_warhammer_2_entry: Entry,
    pub settings_path_warhammer_2_button: Button,
    pub settings_game_list_combo: ComboBoxText,
    pub settings_cancel: Button,
    pub settings_accept: Button,
}

/// Implementation of SettingsWindow.
impl SettingsWindow {

    /// This function creates the entire settings window. It requires the application object to pass
    /// the window to.
    pub fn create_settings_window(application: &Application) -> SettingsWindow {

        let settings_window = ApplicationWindow::new(application);
        settings_window.set_size_request(550, 0);
        settings_window.set_gravity(Gravity::Center);
        settings_window.set_position(WindowPosition::Center);
        settings_window.set_title("Settings");
        settings_window.set_icon_from_file(Path::new("img/rpfm.png")).unwrap();

        let big_boxx = Box::new(Orientation::Vertical, 0);
        big_boxx.set_border_width(7);

        let paths_frame = Frame::new(Some("Paths"));
        paths_frame.set_label_align(0.04, 0.5);

        let paths_big_boxx = Box::new(Orientation::Vertical, 0);
        let path_my_mod_box = Box::new(Orientation::Horizontal, 0);
        let path_warhammer_2_box = Box::new(Orientation::Horizontal, 0);
        path_my_mod_box.set_border_width(4);
        path_warhammer_2_box.set_border_width(4);

        let my_mod_label = Label::new(Some("My mod's folder"));
        let warhammer_2_label = Label::new(Some("TW: Warhammer 2 folder"));
        my_mod_label.set_size_request(170, 0);
        my_mod_label.set_alignment(0.0, 0.5);
        warhammer_2_label.set_size_request(170, 0);
        warhammer_2_label.set_alignment(0.0, 0.5);

        let my_mod_entry = Entry::new();
        let warhammer_2_entry = Entry::new();

        let my_mod_button = Button::new_with_label("...");
        let warhammer_2_button = Button::new_with_label("...");

        let settings_big_boxx = Box::new(Orientation::Vertical, 0);
        let default_game_box = Box::new(Orientation::Horizontal, 0);
        default_game_box.set_border_width(4);

        let default_game_label = Label::new(Some("Default Game Selected:"));
        let game_list_combo = ComboBoxText::new();
        game_list_combo.append(None, "Warhammer 2");
        //game_list_combo.append(None, "Warhammer");
        //game_list_combo.append(None, "Attila");
        //game_list_combo.append(None, "Rome 2");

        game_list_combo.set_active(0);
        game_list_combo.set_size_request(250, 0);

        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(10);

        let cancel_button = Button::new_from_stock("gtk-cancel");
        let accept_button = Button::new_from_stock("gtk-ok");

        // Frame packing stuff...
        path_my_mod_box.pack_start(&my_mod_label, false, false, 0);
        path_my_mod_box.pack_start(&my_mod_entry, true, true, 0);
        path_my_mod_box.pack_end(&my_mod_button, false, false, 0);

        path_warhammer_2_box.pack_start(&warhammer_2_label, false, false, 0);
        path_warhammer_2_box.pack_start(&warhammer_2_entry, true, true, 0);
        path_warhammer_2_box.pack_end(&warhammer_2_button, false, false, 0);

        paths_big_boxx.pack_start(&path_my_mod_box, false, false, 0);
        paths_big_boxx.pack_start(&path_warhammer_2_box, false, false, 0);

        paths_frame.add(&paths_big_boxx);

        // Settings packing stuff...
        default_game_box.pack_start(&default_game_label, false, false, 0);
        default_game_box.pack_end(&game_list_combo, false, false, 0);

        settings_big_boxx.pack_start(&default_game_box, false, false, 0);

        // ButtonBox packing stuff...
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&accept_button, false, false, 0);


        // General packing stuff...
        big_boxx.pack_start(&paths_frame, false, false, 0);
        big_boxx.pack_start(&settings_big_boxx, false, false, 0);
        big_boxx.pack_end(&button_box, false, false, 5);

        settings_window.add(&big_boxx);
        settings_window.show_all();

        SettingsWindow {
            settings_window,
            settings_path_my_mod_entry: my_mod_entry,
            settings_path_my_mod_button: my_mod_button,
            settings_path_warhammer_2_entry: warhammer_2_entry,
            settings_path_warhammer_2_button: warhammer_2_button,
            settings_game_list_combo: game_list_combo,
            settings_cancel: cancel_button,
            settings_accept: accept_button,
        }
    }

    /// This function loads the data from the settings object to the settings window.
    pub fn load_to_settings_window(&self, settings: &Settings) {
        self.settings_game_list_combo.set_active(settings.default_game);
        if let Some(ref path) = settings.paths.my_mods_base_path {
            self.settings_path_my_mod_entry.get_buffer().set_text(&path.to_string_lossy());
        }
        if let Some(ref path) = settings.paths.warhammer_2 {
            self.settings_path_warhammer_2_entry.get_buffer().set_text(&path.to_string_lossy());
        }
    }

    /// This function gets the data from the settings window and returns a Settings object with that
    /// data in it.
    pub fn save_from_settings_window(&self) -> Settings {
        let mut settings = Settings::new();
        settings.default_game = self.settings_game_list_combo.get_active();
        if Path::new(&self.settings_path_my_mod_entry.get_buffer().get_text()).is_dir() {
            settings.paths.my_mods_base_path = Some(PathBuf::from(&self.settings_path_my_mod_entry.get_buffer().get_text()));
        }
        if Path::new(&self.settings_path_warhammer_2_entry.get_buffer().get_text()).is_dir() {
            settings.paths.warhammer_2 = Some(PathBuf::from(&self.settings_path_warhammer_2_entry.get_buffer().get_text()));
        }
        settings
    }
}
