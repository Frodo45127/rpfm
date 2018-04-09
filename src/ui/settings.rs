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
    Entry, Button, Frame, ComboBoxText, ApplicationWindow, WindowPosition, Orientation,
    Label, ButtonBox, ButtonBoxStyle, Application, FileChooserNative, ResponseType, FileChooserAction,
    ReliefStyle, StyleContext, CheckButton, Grid, FontButton
};
use pango::{
    AttrList, Attribute
};
use settings::Settings;
use settings::GameInfo;
use settings::GameSelected;

/// `SettingsWindow`: This struct holds all the relevant stuff for the Settings Window.
#[derive(Clone, Debug)]
pub struct SettingsWindow {
    pub settings_window: ApplicationWindow,
    pub settings_path_my_mod_entry: Entry,
    pub settings_path_my_mod_button: Button,
    pub settings_path_entries: Vec<Entry>,
    pub settings_path_buttons: Vec<Button>,
    pub settings_game_list_combo: ComboBoxText,
    pub settings_extra_check_updates_on_start: CheckButton,
    pub settings_theme_prefer_dark_theme: CheckButton,
    pub settings_theme_font_button: FontButton,
    pub settings_cancel: Button,
    pub settings_accept: Button,
}

/// `MyModNewWindow`: This struct holds all the relevant stuff for "My Mod"'s New Mod Window.
#[derive(Clone, Debug)]
pub struct MyModNewWindow {
    pub my_mod_new_window: ApplicationWindow,
    pub my_mod_new_game_list_combo: ComboBoxText,
    pub my_mod_new_name_entry: Entry,
    pub my_mod_new_cancel: Button,
    pub my_mod_new_accept: Button,
}


/// Implementation of `SettingsWindow`.
impl SettingsWindow {

    /// This function creates the entire settings window. It requires the application object to pass
    /// the window to.
    pub fn create_settings_window(application: &Application, rpfm_path: &PathBuf, supported_games: &[GameInfo]) -> SettingsWindow {

        let settings_window = ApplicationWindow::new(application);
        settings_window.set_size_request(700, 0);
        settings_window.set_gravity(Gravity::Center);
        settings_window.set_position(WindowPosition::Center);
        settings_window.set_title("Settings");

        // Config the icon for the Settings Window. If this fails, something went wrong when setting the paths,
        // so crash the program, as we don't know what more is broken.
        settings_window.set_icon_from_file(&Path::new(&format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()))).unwrap();

        // Disable the menubar in this window.
        settings_window.set_show_menubar(false);

        // Get the current GTK settings. This unwrap is not expected to fail anytime, so we unwrap it.
        let gtk_settings = &settings_window.get_settings().unwrap();

        // Stuff of the Settings window.
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

        // The "MyMod" entry is created here.
        let my_mod_label = Label::new(Some("My mod's folder"));
        let my_mod_entry = Entry::new();
        let my_mod_button = Button::new_with_label("...");
        my_mod_label.set_size_request(170, 0);
        my_mod_label.set_xalign(0.0);
        my_mod_label.set_yalign(0.5);
        my_mod_entry.set_has_frame(false);
        my_mod_entry.set_hexpand(true);
        my_mod_entry.set_placeholder_text("This is the folder where you want to store all \"MyMod\" related files.");

        // All the "Game" entries are created dinamically here, depending on the supported games.
        let mut game_entries: Vec<Entry> = vec![];
        let mut game_buttons: Vec<Button> = vec![];

        for (index, game) in supported_games.iter().enumerate() {
            let label = Label::new(Some(&*format!("TW: {} folder", game.display_name)));
            let entry = Entry::new();
            let button = Button::new_with_label("...");
            label.set_size_request(170, 0);
            label.set_xalign(0.0);
            label.set_yalign(0.5);
            entry.set_has_frame(false);
            entry.set_hexpand(true);
            entry.set_placeholder_text(&*format!("This is the folder where you have {} installed.", game.display_name));

            paths_grid.attach(&label, 0, (index + 1) as i32, 1, 1);
            paths_grid.attach(&entry, 1, (index + 1) as i32, 1, 1);
            paths_grid.attach(&button, 2, (index + 1) as i32, 1, 1);

            game_entries.push(entry);
            game_buttons.push(button);
        }

        let theme_frame = Frame::new(Some("Theme Settings"));
        let theme_grid = Grid::new();
        theme_frame.set_label_align(0.04, 0.5);
        theme_grid.set_border_width(6);
        theme_grid.set_row_spacing(3);
        theme_grid.set_column_spacing(3);

        let prefer_dark_theme_label = Label::new(Some("Use Dark Theme:"));
        let prefer_dark_theme_checkbox = CheckButton::new();
        let font_settings_label = Label::new(Some("Font Settings:"));
        let font_settings_button = FontButton::new();
        prefer_dark_theme_label.set_size_request(170, 0);
        prefer_dark_theme_label.set_xalign(0.0);
        prefer_dark_theme_label.set_yalign(0.5);
        prefer_dark_theme_checkbox.set_hexpand(true);
        font_settings_label.set_size_request(170, 0);
        font_settings_label.set_xalign(0.0);
        font_settings_label.set_yalign(0.5);
        font_settings_button.set_hexpand(true);

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
        for game in supported_games.iter() {
            game_list_combo.append(Some(&*game.folder_name), &game.display_name);
        }

        game_list_combo.set_active(0);
        game_list_combo.set_hexpand(true);

        let check_updates_on_start_label = Label::new(Some("Check Updates on Start:"));
        let check_updates_on_start_checkbox = CheckButton::new();
        check_updates_on_start_label.set_size_request(170, 0);
        check_updates_on_start_label.set_xalign(0.0);
        check_updates_on_start_label.set_yalign(0.5);
        check_updates_on_start_checkbox.set_hexpand(true);

        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(10);

        let restore_default_button = Button::new_with_label("Restore Default");
        let cancel_button = Button::new_with_label("Cancel");
        let accept_button = Button::new_with_label("Accept");

        // Frame packing stuff...
        paths_grid.attach(&my_mod_label, 0, 0, 1, 1);
        paths_grid.attach(&my_mod_entry, 1, 0, 1, 1);
        paths_grid.attach(&my_mod_button, 2, 0, 1, 1);

        paths_frame.add(&paths_grid);

        // Theme Settings packing stuff...
        theme_grid.attach(&prefer_dark_theme_label, 0, 0, 1, 1);
        theme_grid.attach(&prefer_dark_theme_checkbox, 1, 0, 1, 1);
        theme_grid.attach(&font_settings_label, 0, 1, 1, 1);
        theme_grid.attach(&font_settings_button, 1, 1, 1, 1);

        theme_frame.add(&theme_grid);

        // Extra Settings packing stuff
        extra_settings_grid.attach(&default_game_label, 0, 0, 1, 1);
        extra_settings_grid.attach(&game_list_combo, 1, 0, 1, 1);
        extra_settings_grid.attach(&check_updates_on_start_label, 0, 1, 1, 1);
        extra_settings_grid.attach(&check_updates_on_start_checkbox, 1, 1, 1, 1);

        extra_settings_frame.add(&extra_settings_grid);

        // ButtonBox packing stuff...
        button_box.pack_start(&restore_default_button, false, false, 0);
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
            gtk_settings => move |checkbox| {
                gtk_settings.set_property_gtk_application_prefer_dark_theme(checkbox.get_active());
            }
        ));

        // Event to change the Font used.
        font_settings_button.connect_font_set(clone!(
            gtk_settings => move |font_settings_button| {
                let new_font = font_settings_button.get_font_name().unwrap_or("Segoe UI 9".to_owned());
                gtk_settings.set_property_gtk_font_name(Some(&new_font));
            }
        ));

        // Events for the `Entries`.
        // Check all the entries. If all are valid, enable the "Accept" button.
        // FIXME: Fix this shit.
        accept_button.connect_property_relief_notify(clone!(
            game_entries,
            my_mod_entry => move |accept_button| {
                let mut invalid_path = false;
                for game in game_entries.iter() {
                    if Path::new(&game.get_buffer().get_text()).is_dir() || game.get_buffer().get_text().is_empty() {
                        invalid_path = false;
                    }
                    else {
                        invalid_path = true;
                        break;
                    }
                }

                if (Path::new(&my_mod_entry.get_buffer().get_text()).is_dir() || my_mod_entry.get_buffer().get_text().is_empty()) && !invalid_path {
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

        // Create an iterator chaining every entry with his button.
        let game_entries_cloned = game_entries.clone();
        let game_buttons_cloned = game_buttons.clone();
        let entries = game_entries_cloned.iter().cloned().zip(game_buttons_cloned.iter().cloned());
        for (index, game) in entries.enumerate() {

            // When we change the path in the game's entry.
            game.0.connect_changed(clone!(
                accept_button,
                game => move |text_entry| {
                    paint_entry(text_entry, &game.1, &accept_button);
                }
            ));

            // When we press the "..." buttons.
            let supported_games = supported_games.to_vec();
            game.1.connect_button_release_event(clone!(
                game,
                accept_button,
                supported_games,
                settings_window => move |_,_| {
                    update_entry_path(
                        &game.0,
                        &game.1,
                        &accept_button,
                        &format!("Select {} Folder", &supported_games[index].display_name),
                        &settings_window,
                    );
                    Inhibit(false)
                }
            ));
        }

        // Create the SettingsWindow object and store it (We need it for the "Restore Default" button).
        let window = SettingsWindow {
            settings_window,
            settings_path_my_mod_entry: my_mod_entry,
            settings_path_my_mod_button: my_mod_button,
            settings_path_entries: game_entries,
            settings_path_buttons: game_buttons,
            settings_game_list_combo: game_list_combo,
            settings_extra_check_updates_on_start: check_updates_on_start_checkbox,
            settings_theme_prefer_dark_theme: prefer_dark_theme_checkbox,
            settings_theme_font_button: font_settings_button,
            settings_cancel: cancel_button,
            settings_accept: accept_button,
        };

        // When we press the "Restore Default" button, we restore the settings to their "Default" values.
        let supported_games = supported_games.to_vec();
        restore_default_button.connect_button_release_event(clone!(
            window => move |_,_| {
                let default_settings = Settings::new(&supported_games);
                &window.load_to_settings_window(&default_settings);
                load_gtk_settings(&window.settings_window, &default_settings);
                Inhibit(false)
            }
        ));

        // Now, return the window.
        window
    }

    /// This function loads the data from the settings object to the settings window.
    pub fn load_to_settings_window(&self, settings: &Settings) {

        // Load the "Default Game".
        self.settings_game_list_combo.set_active_id(Some(&*settings.default_game));

        // Load the "Check Updates on Start" setting.
        self.settings_extra_check_updates_on_start.set_active(settings.check_updates_on_start);

        // Load the current Theme prefs.
        self.settings_theme_prefer_dark_theme.set_active(settings.prefer_dark_theme);
        self.settings_theme_font_button.set_font_name(&settings.font);

        // Load the data to the entries.
        self.settings_path_my_mod_entry.get_buffer().set_text(&settings.paths.my_mods_base_path.clone().unwrap_or(PathBuf::from("")).to_string_lossy());

        // Load the data to the game entries.
        let entries = self.settings_path_entries.iter().zip(self.settings_path_buttons.iter());
        for (index, game) in entries.clone().enumerate() {
            game.0.get_buffer().set_text(&settings.paths.game_paths[index].path.clone().unwrap_or(PathBuf::from("")).to_string_lossy());
        }

        // Paint the entries and buttons.
        paint_entry(&self.settings_path_my_mod_entry, &self.settings_path_my_mod_button, &self.settings_accept);
        for game in entries {
            paint_entry(&game.0, &game.1, &self.settings_accept);
        }
    }

    /// This function gets the data from the settings window and returns a Settings object with that
    /// data in it.
    pub fn save_from_settings_window(&self, supported_games: &[GameInfo]) -> Settings {
        let mut settings = Settings::new(supported_games);

        // We get his game's folder, depending on the selected game.
        settings.default_game = self.settings_game_list_combo.get_active_id().unwrap();

        // Get the "Check Updates on Start" setting.
        settings.check_updates_on_start = self.settings_extra_check_updates_on_start.get_active();

        // Get the Theme and Font settings.
        settings.prefer_dark_theme = self.settings_theme_prefer_dark_theme.get_active();
        settings.font = self.settings_theme_font_button.get_font_name().unwrap_or("Segoe UI 9".to_owned());

        // Only if we have valid directories, we save them. Otherwise we wipe them out.
        settings.paths.my_mods_base_path = match Path::new(&self.settings_path_my_mod_entry.get_buffer().get_text()).is_dir() {
            true => Some(PathBuf::from(&self.settings_path_my_mod_entry.get_buffer().get_text())),
            false => None,
        };

        // For each entry, we get check if it's a valid directory and save it into `settings`.
        let entries = self.settings_path_entries.iter().zip(self.settings_path_buttons.iter());
        for (index, game) in entries.enumerate() {
            settings.paths.game_paths[index].path = match Path::new(&game.0.get_buffer().get_text()).is_dir() {
                true => Some(PathBuf::from(&game.0.get_buffer().get_text())),
                false => None,
            }
        }

        settings
    }
}

/// Implementation of `MyModNewWindow`.
impl MyModNewWindow {

    /// This function creates the entire "New Mod" window. It requires the application object to pass
    /// the window to.
    pub fn create_my_mod_new_window(
        application: &Application,
        supported_games: &[GameInfo],
        game_selected: &GameSelected,
        settings: &Settings,
        rpfm_path: &PathBuf
    ) -> MyModNewWindow {

        let my_mod_new_window = ApplicationWindow::new(application);
        my_mod_new_window.set_size_request(500, 0);
        my_mod_new_window.set_gravity(Gravity::Center);
        my_mod_new_window.set_position(WindowPosition::Center);
        my_mod_new_window.set_title("New MyMod");

        // Config the icon for the New "MyMod" Window. If this fails, something went wrong when setting the paths,
        // so crash the program, as we don't know what more is broken.
        my_mod_new_window.set_icon_from_file(&Path::new(&format!("{}/img/rpfm.png", rpfm_path.to_string_lossy()))).unwrap();

        // Disable the menubar in this window.
        my_mod_new_window.set_show_menubar(false);

        // Stuff of the "New Mod" window.
        let big_grid = Grid::new();
        big_grid.set_border_width(6);
        big_grid.set_row_spacing(6);
        big_grid.set_column_spacing(3);

        let advices_frame = Frame::new(Some("Advices"));
        advices_frame.set_label_align(0.04, 0.5);

        let advices_label = Label::new(Some("Things to take into account before creating a new mod:
	- Select the game you'll make the mod for.
	- Pick an simple name (it shouldn't end in *.pack).
	- If you want to use multiple words, use \"_\" instead of \" \".
	- You can't create a mod for a game that has no path set in the settings."));
        advices_label.set_size_request(-1, 0);
        advices_label.set_xalign(0.5);
        advices_label.set_yalign(0.5);

        let mod_name_label = Label::new(Some("Name of the Mod:"));
        mod_name_label.set_size_request(120, 0);
        mod_name_label.set_xalign(0.0);
        mod_name_label.set_yalign(0.5);

        let mod_name_entry = Entry::new();
        mod_name_entry.set_placeholder_text("For example: one_ring_for_me");
        mod_name_entry.set_hexpand(true);
        mod_name_entry.set_has_frame(false);

        let selected_game_label = Label::new(Some("Game of the Mod:"));
        selected_game_label.set_size_request(120, 0);
        selected_game_label.set_xalign(0.0);
        selected_game_label.set_yalign(0.5);

        let selected_game_list_combo = ComboBoxText::new();
        for game in supported_games.iter() {
            selected_game_list_combo.append(Some(&*game.folder_name), &game.display_name);
        }
        selected_game_list_combo.set_active_id(Some(&*game_selected.game));
        selected_game_list_combo.set_hexpand(true);

        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(6);

        let cancel_button = Button::new_with_label("Cancel");
        let accept_button = Button::new_with_label("Accept");

        // Frame packing stuff...
        advices_frame.add(&advices_label);

        // ButtonBox packing stuff...
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&accept_button, false, false, 0);

        // General packing stuff...
        big_grid.attach(&advices_frame, 0, 0, 2, 1);
        big_grid.attach(&mod_name_label, 0, 1, 1, 1);
        big_grid.attach(&mod_name_entry, 1, 1, 1, 1);
        big_grid.attach(&selected_game_label, 0, 2, 1, 1);
        big_grid.attach(&selected_game_list_combo, 1, 2, 1, 1);
        big_grid.attach(&button_box, 0, 3, 2, 1);

        my_mod_new_window.add(&big_grid);
        my_mod_new_window.show_all();

        // By default, the `mod_name_entry` will be empty, so let the ´Accept´ button disabled.
        accept_button.set_sensitive(false);

        // Events to check the Mod's Name is valid and available. This should be done while writing
        // in `mod_name_entry` and when changing the selected game.
        mod_name_entry.connect_changed(clone!(
            settings,
            selected_game_list_combo,
            accept_button => move |text_entry| {
                let selected_game = selected_game_list_combo.get_active_id().unwrap();
                check_my_mod_validity(text_entry, selected_game, &settings, &accept_button);
            }
        ));

        selected_game_list_combo.connect_changed(clone!(
            mod_name_entry,
            settings,
            accept_button => move |selected_game_list_combo| {
                let selected_game = selected_game_list_combo.get_active_id().unwrap();
                check_my_mod_validity(&mod_name_entry, selected_game, &settings, &accept_button);
            }
        ));

        MyModNewWindow {
            my_mod_new_window,
            my_mod_new_game_list_combo: selected_game_list_combo,
            my_mod_new_name_entry: mod_name_entry,
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

/// Modification of `paint_entry`. In this one, the button painted red is the `Accept` button.
fn check_my_mod_validity(
    text_entry: &Entry,
    selected_game: String,
    settings: &Settings,
    accept_button: &Button
) {
    let text = text_entry.get_buffer().get_text();
    let attribute_list = AttrList::new();
    let red = Attribute::new_background(65535, 35000, 35000).unwrap();

    // If there is text and it doesn't have whitespaces...
    if !text.is_empty() && !text.contains(' ') {

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        if let Some(ref mod_path) = settings.paths.my_mods_base_path {
            let mut mod_path = mod_path.clone();
            mod_path.push(selected_game);
            mod_path.push(format!("{}.pack", text));

            // If a mod with that name for that game already exists, disable the "Accept" button.
            if mod_path.is_file() {

                attribute_list.insert(red);
                accept_button.set_sensitive(false);
            }

            // If the name is available, enable the `Accept` button.
            else {
                accept_button.set_sensitive(true);
            }
        }

        // If there is no "MyMod" path configured, disable the button.
        else {
            attribute_list.insert(red);
            accept_button.set_sensitive(false);
        }
    }

    // If name is empty, disable the button but don't make the text red.
    else {
        accept_button.set_sensitive(false);
    }

    text_entry.set_attributes(&attribute_list);
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

/// This function loads the Theme and Font settings we have in our `Setting` object to GTK.
pub fn load_gtk_settings(window: &ApplicationWindow, settings: &Settings) {

    // Depending on our settings, load the GTK Theme we want to use.
    let gtk_settings = window.get_settings().unwrap();
    gtk_settings.set_property_gtk_application_prefer_dark_theme(settings.prefer_dark_theme);
    gtk_settings.set_property_gtk_font_name(Some(&settings.font));
}
