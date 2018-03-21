// Here it goes all the stuff related with "Settings" and "My Mod" windows.
extern crate gtk;
extern crate pango;
extern crate url;

use std::path::{
    Path, PathBuf
};
use url::Url;
use gtk::prelude::*;
use gdk::Gravity;
use gtk::{
    Entry, Box, Button, Frame, ComboBoxText, ApplicationWindow, WindowPosition, Orientation,
    Label, ButtonBox, ButtonBoxStyle, Application, FileChooserNative, ResponseType, FileChooserAction,
    ReliefStyle, StyleContext, CheckButton, Grid
};
use pango::{
    AttrList, Attribute
};
use settings::Settings;

/// `SettingsWindow`: This struct holds all the relevant stuff for the Settings Window.
#[derive(Clone, Debug)]
pub struct SettingsWindow {
    pub settings_window: ApplicationWindow,
    pub settings_path_my_mod_entry: Entry,
    pub settings_path_warhammer_2_entry: Entry,
    pub settings_path_warhammer_entry: Entry,
    pub settings_path_my_mod_button: Button,
    pub settings_path_warhammer_2_button: Button,
    pub settings_path_warhammer_button: Button,
    pub settings_game_list_combo: ComboBoxText,
    pub settings_theme_prefer_dark_theme: CheckButton,
    pub settings_cancel: Button,
    pub settings_accept: Button,
}

/// `MyModNewWindow`: This struct holds all the relevant stuff for "My Mod"'s New Mod Window.
#[derive(Clone, Debug)]
pub struct MyModNewWindow {
    pub my_mod_new_window: ApplicationWindow,
    pub my_mod_new_game_list_combo: ComboBoxText,
    pub my_mod_new_name_entry: Entry,
    pub my_mod_new_name_is_valid_label: Label,
    pub my_mod_new_cancel: Button,
    pub my_mod_new_accept: Button,
}


/// Implementation of `SettingsWindow`.
impl SettingsWindow {

    /// This function creates the entire settings window. It requires the application object to pass
    /// the window to.
    pub fn create_settings_window(application: &Application) -> SettingsWindow {

        let settings_window = ApplicationWindow::new(application);
        settings_window.set_size_request(700, 0);
        settings_window.set_gravity(Gravity::Center);
        settings_window.set_position(WindowPosition::Center);
        settings_window.set_title("Settings");
        settings_window.set_icon_from_file(Path::new("img/rpfm.png")).unwrap();

        // Disable the menubar in this window.
        settings_window.set_show_menubar(false);

        let big_grid = Grid::new();
        big_grid.set_border_width(6);
        big_grid.set_row_spacing(3);
        big_grid.set_column_spacing(3);

        let paths_frame = Frame::new(Some("Paths"));
        let paths_grid = Grid::new();
        paths_frame.set_label_align(0.04, 0.5);
        paths_grid.set_border_width(6);
        paths_grid.set_row_spacing(3);
        paths_grid.set_column_spacing(3);

        let my_mod_label = Label::new(Some("My mod's folder"));
        let warhammer_2_label = Label::new(Some("TW: Warhammer 2 folder"));
        let warhammer_label = Label::new(Some("TW: Warhammer folder"));
        my_mod_label.set_size_request(170, 0);
        my_mod_label.set_xalign(0.0);
        my_mod_label.set_yalign(0.5);
        warhammer_2_label.set_size_request(170, 0);
        warhammer_2_label.set_xalign(0.0);
        warhammer_2_label.set_yalign(0.5);
        warhammer_label.set_size_request(170, 0);
        warhammer_label.set_xalign(0.0);
        warhammer_label.set_yalign(0.5);

        let my_mod_entry = Entry::new();
        let warhammer_2_entry = Entry::new();
        let warhammer_entry = Entry::new();
        my_mod_entry.set_has_frame(false);
        my_mod_entry.set_hexpand(true);
        my_mod_entry.set_placeholder_text("This is the folder where you want to store all \"MyMod\" related files.");
        warhammer_2_entry.set_has_frame(false);
        warhammer_2_entry.set_hexpand(true);
        warhammer_2_entry.set_placeholder_text("This is the folder where you have Warhammer 2 installed.");
        warhammer_entry.set_has_frame(false);
        warhammer_entry.set_hexpand(true);
        warhammer_entry.set_placeholder_text("This is the folder where you have Warhammer installed.");

        let my_mod_button = Button::new_with_label("...");
        let warhammer_2_button = Button::new_with_label("...");
        let warhammer_button = Button::new_with_label("...");

        let theme_frame = Frame::new(Some("Theme Settings"));
        let theme_grid = Grid::new();
        theme_frame.set_label_align(0.04, 0.5);
        theme_grid.set_border_width(6);
        theme_grid.set_row_spacing(3);
        theme_grid.set_column_spacing(3);

        let prefer_dark_theme_label = Label::new(Some("Use Dark Theme:"));
        let prefer_dark_theme_checkbox = CheckButton::new();
        prefer_dark_theme_label.set_size_request(170, 0);
        prefer_dark_theme_label.set_xalign(0.0);
        prefer_dark_theme_label.set_yalign(0.5);
        prefer_dark_theme_checkbox.set_hexpand(true);

        let extra_settings_frame = Frame::new(Some("Extra Settings"));
        let extra_settings_grid = Grid::new();
        extra_settings_frame.set_label_align(0.04, 0.5);
        extra_settings_grid.set_border_width(6);
        extra_settings_grid.set_row_spacing(3);
        extra_settings_grid.set_column_spacing(3);

        let default_game_label = Label::new(Some("Default Game Selected:"));
        let game_list_combo = ComboBoxText::new();
        default_game_label.set_size_request(170, 0);
        default_game_label.set_xalign(0.0);
        default_game_label.set_yalign(0.5);
        game_list_combo.append(None, "Warhammer 2");
        game_list_combo.append(None, "Warhammer");
        //game_list_combo.append(None, "Attila");
        //game_list_combo.append(None, "Rome 2");

        game_list_combo.set_active(0);
        game_list_combo.set_hexpand(true);

        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(10);

        let cancel_button = Button::new_with_label("Cancel");
        let accept_button = Button::new_with_label("Accept");

        // Frame packing stuff...
        paths_grid.attach(&my_mod_label, 0, 0, 1, 1);
        paths_grid.attach(&my_mod_entry, 1, 0, 1, 1);
        paths_grid.attach(&my_mod_button, 2, 0, 1, 1);

        paths_grid.attach(&warhammer_2_label, 0, 1, 1, 1);
        paths_grid.attach(&warhammer_2_entry, 1, 1, 1, 1);
        paths_grid.attach(&warhammer_2_button, 2, 1, 1, 1);

        paths_grid.attach(&warhammer_label, 0, 2, 1, 1);
        paths_grid.attach(&warhammer_entry, 1, 2, 1, 1);
        paths_grid.attach(&warhammer_button, 2, 2, 1, 1);

        paths_frame.add(&paths_grid);

        // Theme Settings packing stuff...
        theme_grid.attach(&prefer_dark_theme_label, 0, 0, 1, 1);
        theme_grid.attach(&prefer_dark_theme_checkbox, 1, 0, 1, 1);

        theme_frame.add(&theme_grid);

        // Extra Settings packing stuff
        extra_settings_grid.attach(&default_game_label, 0, 0, 1, 1);
        extra_settings_grid.attach(&game_list_combo, 1, 0, 1, 1);

        extra_settings_frame.add(&extra_settings_grid);

        // ButtonBox packing stuff...
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&accept_button, false, false, 0);

        // General packing stuff...
        big_grid.attach(&paths_frame, 0, 0, 2, 1);
        big_grid.attach(&theme_frame, 0, 1, 1, 1);
        big_grid.attach(&extra_settings_frame, 1, 1, 1, 1);
        big_grid.attach(&button_box, 0, 2, 2, 1);

        settings_window.add(&big_grid);
        settings_window.show_all();

        // Event to change between "Light/Dark" theme variations.
        prefer_dark_theme_checkbox.connect_toggled(clone!(
            settings_window => move |checkbox| {
                let theme_settings = settings_window.get_settings().unwrap();
                theme_settings.set_property_gtk_application_prefer_dark_theme(checkbox.get_active());
            }
        ));

        // Events for the `Entries`.
        // Check all the entries. If all are valid, enable the "Accept" button.
        // FIXME: Fix this shit.
        accept_button.connect_property_relief_notify(clone!(
            my_mod_entry,
            warhammer_2_entry,
            warhammer_entry => move |accept_button| {
                if (Path::new(&my_mod_entry.get_buffer().get_text()).is_dir() || my_mod_entry.get_buffer().get_text().is_empty()) &&
                    (Path::new(&warhammer_2_entry.get_buffer().get_text()).is_dir() || warhammer_2_entry.get_buffer().get_text().is_empty()) &&
                    (Path::new(&warhammer_entry.get_buffer().get_text()).is_dir() || warhammer_entry.get_buffer().get_text().is_empty()) {
                    accept_button.set_sensitive(true);
                }
                else {
                    accept_button.set_sensitive(false);
                }
            }
        ));

        // Set their background red while writing in them if their path is not valid.
        my_mod_entry.connect_changed(clone!(
            accept_button,
            my_mod_button => move |text_entry| {
                paint_entry(text_entry, &my_mod_button, &accept_button);
            }
        ));
        warhammer_2_entry.connect_changed(clone!(
            accept_button,
            warhammer_2_button => move |text_entry| {
                paint_entry(text_entry, &warhammer_2_button, &accept_button);
            }
        ));
        warhammer_entry.connect_changed(clone!(
            accept_button,
            warhammer_button => move |text_entry| {
                paint_entry(text_entry, &warhammer_button, &accept_button);
            }
        ));

        // When we press the "..." buttons.
        my_mod_button.connect_button_release_event(clone!(
            my_mod_entry,
            my_mod_button,
            accept_button,
            settings_window => move |_,_| {
                update_entry_path(
                    &my_mod_entry,
                    &my_mod_button,
                    &accept_button,
                    "Select MyMod's Folder",
                    &settings_window,
                );
                Inhibit(false)
            }
        ));

        warhammer_2_button.connect_button_release_event(clone!(
            warhammer_2_entry,
            warhammer_2_button,
            accept_button,
            settings_window => move |_,_| {
                update_entry_path(
                    &warhammer_2_entry,
                    &warhammer_2_button,
                    &accept_button,
                    "Select Warhammer 2 Folder",
                    &settings_window
                );
                Inhibit(false)
            }
        ));

        warhammer_button.connect_button_release_event(clone!(
            warhammer_entry,
            warhammer_button,
            accept_button,
            settings_window => move |_,_| {
                update_entry_path(
                    &warhammer_entry,
                    &warhammer_button,
                    &accept_button,
                    "Select Warhammer Folder",
                    &settings_window
                );
                Inhibit(false)
            }
        ));

        SettingsWindow {
            settings_window,
            settings_path_my_mod_entry: my_mod_entry,
            settings_path_warhammer_2_entry: warhammer_2_entry,
            settings_path_warhammer_entry: warhammer_entry,
            settings_game_list_combo: game_list_combo,
            settings_path_my_mod_button: my_mod_button,
            settings_path_warhammer_2_button: warhammer_2_button,
            settings_path_warhammer_button: warhammer_button,
            settings_theme_prefer_dark_theme: prefer_dark_theme_checkbox,
            settings_cancel: cancel_button,
            settings_accept: accept_button,
        }
    }

    /// This function loads the data from the settings object to the settings window.
    pub fn load_to_settings_window(&self, settings: &Settings) {
        self.settings_game_list_combo.set_active(
            match &*settings.default_game {
                "warhammer_2" => 0,
                "warhammer" => 1,
                "attila" => 2,
                "rome_2" => 3,
                _ => 0,
            }
        );

        // Load the current Theme prefs.
        self.settings_theme_prefer_dark_theme.set_active(settings.prefer_dark_theme);

        // Load the data to the entries.
        if let Some(ref path) = settings.paths.my_mods_base_path {
            self.settings_path_my_mod_entry.get_buffer().set_text(&path.to_string_lossy());
        }
        if let Some(ref path) = settings.paths.warhammer_2 {
            self.settings_path_warhammer_2_entry.get_buffer().set_text(&path.to_string_lossy());
        }
        if let Some(ref path) = settings.paths.warhammer {
            self.settings_path_warhammer_entry.get_buffer().set_text(&path.to_string_lossy());
        }

        // Paint the entries and buttons.
        paint_entry(&self.settings_path_my_mod_entry, &self.settings_path_my_mod_button, &self.settings_accept);
        paint_entry(&self.settings_path_warhammer_2_entry, &self.settings_path_warhammer_2_button, &self.settings_accept);
        paint_entry(&self.settings_path_warhammer_entry, &self.settings_path_warhammer_button, &self.settings_accept);
    }

    /// This function gets the data from the settings window and returns a Settings object with that
    /// data in it.
    pub fn save_from_settings_window(&self) -> Settings {
        let mut settings = Settings::new();

        // We get his game's folder, depending on the selected game.
        let selected_game = self.settings_game_list_combo.get_active_text().unwrap();
        let selected_game_folder = match &*selected_game {
            "Warhammer 2" => "warhammer_2",
            "Warhammer" => "warhammer",
            "Attila" => "attila",
            "Rome 2" => "rome_2",
            _ => "if_you_see_this_folder_report_it",
        };
        settings.default_game = selected_game_folder.to_owned();
        settings.prefer_dark_theme = self.settings_theme_prefer_dark_theme.get_active();

        if Path::new(&self.settings_path_my_mod_entry.get_buffer().get_text()).is_dir() {
            settings.paths.my_mods_base_path = Some(PathBuf::from(&self.settings_path_my_mod_entry.get_buffer().get_text()));
        }
        if Path::new(&self.settings_path_warhammer_2_entry.get_buffer().get_text()).is_dir() {
            settings.paths.warhammer_2 = Some(PathBuf::from(&self.settings_path_warhammer_2_entry.get_buffer().get_text()));
        }
        if Path::new(&self.settings_path_warhammer_entry.get_buffer().get_text()).is_dir() {
            settings.paths.warhammer = Some(PathBuf::from(&self.settings_path_warhammer_entry.get_buffer().get_text()));
        }
        settings
    }
}

/// Implementation of `MyModNewWindow`.
impl MyModNewWindow {

    /// This function creates the entire "New mod" window. It requires the application object to pass
    /// the window to.
    pub fn create_my_mod_new_window(application: &Application) -> MyModNewWindow {

        let my_mod_new_window = ApplicationWindow::new(application);
        my_mod_new_window.set_size_request(500, 0);
        my_mod_new_window.set_gravity(Gravity::Center);
        my_mod_new_window.set_position(WindowPosition::Center);
        my_mod_new_window.set_title("New mod");
        my_mod_new_window.set_icon_from_file(Path::new("img/rpfm.png")).unwrap();

        // Disable the menubar in this window.
        my_mod_new_window.set_show_menubar(false);

        // One box to hold them all.
        let big_boxx = Box::new(Orientation::Vertical, 0);
        big_boxx.set_border_width(7);

        let advices_frame = Frame::new(Some("Advices"));
        advices_frame.set_label_align(0.04, 0.5);

        let advices_label = Label::new(Some("Things to take into account before creating a new mod:
	- Select the game you'll make the mod for.
	- Pick an simple name (it shouldn't end in *.pack).
	- If you want to use multiple words, use \"_\" instead of \" \".
	- You can't create a mod for a game that has no path set in the settings."));
        advices_label.set_size_request(170, 0);
        advices_label.set_xalign(0.5);
        advices_label.set_yalign(0.5);

        let selected_game_box = Box::new(Orientation::Horizontal, 0);
        selected_game_box.set_border_width(4);

        let selected_game_label = Label::new(Some("Game of the Mod:"));
        selected_game_label.set_size_request(125, 0);
        selected_game_label.set_xalign(0.0);
        selected_game_label.set_yalign(0.5);

        let selected_game_list_combo = ComboBoxText::new();
        selected_game_list_combo.append(None, "Warhammer 2");
        selected_game_list_combo.append(None, "Warhammer");
        //selected_game_list_combo.append(None, "Attila");
        //selected_game_list_combo.append(None, "Rome 2");
        selected_game_list_combo.set_active(0);

        let mod_name_box = Box::new(Orientation::Horizontal, 0);
        mod_name_box.set_border_width(4);

        let mod_name_label = Label::new(Some("Name of the Mod:"));
        mod_name_label.set_size_request(125, 0);
        mod_name_label.set_xalign(0.0);
        mod_name_label.set_yalign(0.5);

        let mod_name_entry = Entry::new();
        mod_name_entry.set_placeholder_text("For example: one_ring_for_me");

        let is_name_valid_label = Label::new(Some("Invalid"));
        is_name_valid_label.set_size_request(125, 0);
        is_name_valid_label.set_xalign(0.5);
        is_name_valid_label.set_yalign(0.5);

        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(10);

        let cancel_button = Button::new_with_label("Cancel");
        let accept_button = Button::new_with_label("Accept");

        // Frame packing stuff...
        advices_frame.add(&advices_label);

        // Input packing stuff...
        selected_game_box.pack_start(&selected_game_label, false, false, 0);
        selected_game_box.pack_end(&selected_game_list_combo, true, true, 0);

        mod_name_box.pack_start(&mod_name_label, false, false, 0);
        mod_name_box.pack_start(&mod_name_entry, true, true, 0);
        mod_name_box.pack_end(&is_name_valid_label, false, false, 0);

        // ButtonBox packing stuff...
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&accept_button, false, false, 0);

        // General packing stuff...
        big_boxx.pack_start(&advices_frame, false, false, 0);
        big_boxx.pack_start(&selected_game_box, false, false, 0);
        big_boxx.pack_start(&mod_name_box, false, false, 0);
        big_boxx.pack_end(&button_box, false, false, 5);

        my_mod_new_window.add(&big_boxx);
        my_mod_new_window.show_all();

        MyModNewWindow {
            my_mod_new_window,
            my_mod_new_game_list_combo: selected_game_list_combo,
            my_mod_new_name_entry: mod_name_entry,
            my_mod_new_name_is_valid_label: is_name_valid_label,
            my_mod_new_cancel: cancel_button,
            my_mod_new_accept: accept_button,
        }
    }
}

/// This function paints the provided `Entry` depending if the text inside the `Entry` is a valid
/// `Path` or not. It also set the button as "destructive-action" if there is no path set or it's
/// invalid. And, If any of the paths is invalid, it disables the "Accept" button.
fn paint_entry(text_entry: &Entry, text_button: &Button, accept_button: &Button) {
    let text = text_entry.get_buffer().get_text();
    let attribute_list = AttrList::new();
    let style_context = text_button.get_style_context().unwrap();

    // Set `text_button` as "Normal" by default.
    text_button.set_relief(ReliefStyle::None);
    StyleContext::remove_class(&style_context, "suggested-action");
    StyleContext::remove_class(&style_context, "destructive-action");

    // If there is text and it's an invalid path, we paint in red. We clear the background otherwise.
    if !text.is_empty() {
        if !Path::new(&text).is_dir() {
            let red = Attribute::new_background(65535, 35000, 35000).unwrap();
            attribute_list.insert(red);

            text_button.set_relief(ReliefStyle::Normal);
            StyleContext::add_class(&style_context, "destructive-action");
        }
    }

    // If the `Entry` is empty, we mark his button red.
    else {
        text_button.set_relief(ReliefStyle::Normal);
        StyleContext::add_class(&style_context, "suggested-action");
    }

    text_entry.set_attributes(&attribute_list);

    // Trigger the "check all the paths" signal. This is extremely wonky, but until I find a better
    // way to do it.... It works.
    // FIXME: Fix this shit.
    accept_button.set_relief(ReliefStyle::None);
    accept_button.set_relief(ReliefStyle::Normal);
}

/// This function gets a Folder from a Native FileChooser and put his path into the provided `Entry`.
fn update_entry_path(
    text_entry: &Entry,
    text_button: &Button,
    accept_button: &Button,
    file_chooser_title: &str,
    file_chooser_parent: &ApplicationWindow) {

    let file_chooser_select_folder = FileChooserNative::new(
        file_chooser_title,
        file_chooser_parent,
        FileChooserAction::SelectFolder,
        "Accept",
        "Cancel"
    );

    // If we already have a Path inside the `text_entry` (and it's not empty or an invalid folder),
    // we set it as "starting" path for the FileChooser.
    if let Some(current_path) = text_entry.get_text() {
        if !current_path.is_empty() && PathBuf::from(&current_path).is_dir() {
            file_chooser_select_folder.set_current_folder(PathBuf::from(&current_path));
        }
    }

    // Then run the created FileChooser and update the `text_entry` only if we received `Accept`.
    // We get his `URI`, translate it into `PathBuf`, and then to `&str` to put it into `text_entry`.
    if file_chooser_select_folder.run() == Into::<i32>::into(ResponseType::Accept) {
        if let Some(new_folder) = file_chooser_select_folder.get_uri() {
            let path = Url::parse(&new_folder).unwrap().to_file_path().unwrap();
            text_entry.set_text(&path.to_string_lossy());
            paint_entry(text_entry, text_button, accept_button);
        }
    }
}
