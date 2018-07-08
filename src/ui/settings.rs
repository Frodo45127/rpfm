// Here it goes all the stuff related with "Settings" and "My Mod" windows.
extern crate url;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

use qt_widgets::layout::Layout;
use qt_widgets::application::Application;
use qt_widgets::widget::Widget;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::splitter::Splitter;
use qt_widgets::tree_view::TreeView;
use qt_widgets::main_window::MainWindow;
use qt_widgets::dialog::Dialog;
use qt_widgets::frame::Frame;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::message_box::MessageBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::check_box::CheckBox;
use qt_widgets::message_box::Icon;
use qt_widgets::message_box;
use qt_widgets::dialog_button_box;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::label::Label;
use qt_widgets::dialog_button_box::DialogButtonBox;
use qt_widgets::file_dialog::FileDialog;
use qt_widgets::file_dialog::FileMode;
use qt_widgets::file_dialog::Option::ShowDirsOnly;
use qt_widgets::group_box::GroupBox;

use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::standard_item::StandardItem;
use qt_core::event_loop::EventLoop;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::slots::SlotBool;
use qt_core::slots::SlotNoArgs;
use qt_core::object::Object;
use qt_core::qt::WidgetAttribute;
use cpp_utils::{CppBox, StaticCast, DynamicCast};

use std::cell::RefCell;
use std::rc::Rc;
use std::path::{
    Path, PathBuf
};
use url::Url;

use packfile::packfile::PackFile;
use settings::Settings;
use settings::GameInfo;
use settings::GameSelected;
//use AppUI;
use super::*;
use packfile;

/*
/// `SettingsWindow`: This struct holds all the relevant stuff for the Settings Window.
#[derive(Clone, Debug)]
pub struct SettingsWindow {
    pub settings_window: ApplicationWindow,
    pub settings_path_my_mod_entry: Entry,
    pub settings_path_my_mod_button: Button,
    pub settings_path_entries: Vec<Entry>,
    pub settings_path_buttons: Vec<Button>,
    pub settings_game_list_combo: ComboBoxText,
    pub settings_extra_allow_edition_of_ca_packfiles: CheckButton,
    pub settings_extra_check_updates_on_start: CheckButton,
    pub settings_extra_check_schema_updates_on_start: CheckButton,
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

/// `NewPrefabWindow`: This struct holds all the relevant stuff for the "New Prefab" window to work.
#[derive(Clone, Debug)]
pub struct NewPrefabWindow {
    pub window: ApplicationWindow,
    pub entries: Vec<Entry>,
    pub accept_button: Button,
    pub cancel_button: Button,
}

/// Implementation of `SettingsWindow`.
impl SettingsWindow {

    /// This function creates the entire settings window. It requires the application object to pass
    /// the window to.
    pub fn create_settings_window(application: &Application, parent: &ApplicationWindow, rpfm_path: &PathBuf, supported_games: &[GameInfo]) -> SettingsWindow {

        let settings_window = ApplicationWindow::new(application);
        settings_window.set_size_request(700, 0);
        settings_window.set_transient_for(parent);
        settings_window.set_position(WindowPosition::CenterOnParent);
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

        let allow_edition_of_ca_packfiles_label = Label::new(Some("Allow edition of CA PackFiles:"));
        let allow_edition_of_ca_packfiles_checkbox = CheckButton::new();
        allow_edition_of_ca_packfiles_label.set_size_request(170, 0);
        allow_edition_of_ca_packfiles_label.set_xalign(0.0);
        allow_edition_of_ca_packfiles_label.set_yalign(0.5);
        allow_edition_of_ca_packfiles_checkbox.set_hexpand(true);

        let check_updates_on_start_label = Label::new(Some("Check Updates on Start:"));
        let check_updates_on_start_checkbox = CheckButton::new();
        check_updates_on_start_label.set_size_request(170, 0);
        check_updates_on_start_label.set_xalign(0.0);
        check_updates_on_start_label.set_yalign(0.5);
        check_updates_on_start_checkbox.set_hexpand(true);

        let check_schema_updates_on_start_label = Label::new(Some("Check Schema Updates on Start:"));
        let check_schema_updates_on_start_checkbox = CheckButton::new();
        check_schema_updates_on_start_label.set_size_request(170, 0);
        check_schema_updates_on_start_label.set_xalign(0.0);
        check_schema_updates_on_start_label.set_yalign(0.5);
        check_schema_updates_on_start_checkbox.set_hexpand(true);

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
        extra_settings_grid.attach(&allow_edition_of_ca_packfiles_label, 0, 1, 1, 1);
        extra_settings_grid.attach(&allow_edition_of_ca_packfiles_checkbox, 1, 1, 1, 1);
        extra_settings_grid.attach(&check_updates_on_start_label, 0, 2, 1, 1);
        extra_settings_grid.attach(&check_updates_on_start_checkbox, 1, 2, 1, 1);
        extra_settings_grid.attach(&check_schema_updates_on_start_label, 0, 3, 1, 1);
        extra_settings_grid.attach(&check_schema_updates_on_start_checkbox, 1, 3, 1, 1);

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
                let new_font = font_settings_button.get_font_name().unwrap_or_else(|| String::from("Segoe UI 9"));
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
                for game in &game_entries {
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
            settings_extra_allow_edition_of_ca_packfiles: allow_edition_of_ca_packfiles_checkbox,
            settings_extra_check_updates_on_start: check_updates_on_start_checkbox,
            settings_extra_check_schema_updates_on_start: check_schema_updates_on_start_checkbox,
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
                window.load_to_settings_window(&default_settings);
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

        // Load the "Allow Edition of CA PackFiles" setting.
        self.settings_extra_allow_edition_of_ca_packfiles.set_active(settings.allow_edition_of_ca_packfiles);

        // Load the "Check Updates on Start" settings.
        self.settings_extra_check_updates_on_start.set_active(settings.check_updates_on_start);
        self.settings_extra_check_schema_updates_on_start.set_active(settings.check_schema_updates_on_start);

        // Load the current Theme prefs.
        self.settings_theme_prefer_dark_theme.set_active(settings.prefer_dark_theme);
        self.settings_theme_font_button.set_font_name(&settings.font);

        // Load the data to the entries.
        self.settings_path_my_mod_entry.get_buffer().set_text(&settings.paths.my_mods_base_path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy());

        // Load the data to the game entries.
        let entries = self.settings_path_entries.iter().zip(self.settings_path_buttons.iter());
        for (index, game) in entries.clone().enumerate() {
            game.0.get_buffer().set_text(&settings.paths.game_paths[index].path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy());
        }

        // Paint the entries and buttons.
        paint_entry(&self.settings_path_my_mod_entry, &self.settings_path_my_mod_button, &self.settings_accept);
        for game in entries {
            paint_entry(game.0, game.1, &self.settings_accept);
        }
    }

    /// This function gets the data from the settings window and returns a Settings object with that
    /// data in it.
    pub fn save_from_settings_window(&self, supported_games: &[GameInfo]) -> Settings {
        let mut settings = Settings::new(supported_games);

        // We get his game's folder, depending on the selected game.
        settings.default_game = self.settings_game_list_combo.get_active_id().unwrap();

        // Get the "Allow Edition of CA PackFiles" setting.
        settings.allow_edition_of_ca_packfiles = self.settings_extra_allow_edition_of_ca_packfiles.get_active();

        // Get the "Check Updates on Start" settings.
        settings.check_updates_on_start = self.settings_extra_check_updates_on_start.get_active();
        settings.check_schema_updates_on_start = self.settings_extra_check_schema_updates_on_start.get_active();

        // Get the Theme and Font settings.
        settings.prefer_dark_theme = self.settings_theme_prefer_dark_theme.get_active();
        settings.font = self.settings_theme_font_button.get_font_name().unwrap_or_else(|| String::from("Segoe UI 9"));

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
        parent: &ApplicationWindow,
        supported_games: &[GameInfo],
        game_selected: &GameSelected,
        settings: &Settings,
        rpfm_path: &PathBuf
    ) -> MyModNewWindow {

        let my_mod_new_window = ApplicationWindow::new(application);
        my_mod_new_window.set_size_request(500, 0);
        my_mod_new_window.set_transient_for(parent);
        my_mod_new_window.set_position(WindowPosition::CenterOnParent);
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

/// Implementation of `NewPrefabWindow`.
impl NewPrefabWindow {

    /// This function creates the window and sets the events needed for everything to work.
    pub fn create_new_prefab_window(
        app_ui: &AppUI,
        application: &Application,
        game_selected: &Rc<RefCell<GameSelected>>,
        pack_file_decoded: &Rc<RefCell<PackFile>>,
        catchment_indexes: &[usize]
    ) {

        // Create the "New Name" window...
        let window = ApplicationWindow::new(application);
        window.set_size_request(500, 0);
        window.set_transient_for(&app_ui.window);
        window.set_position(WindowPosition::CenterOnParent);
        window.set_title("New Prefab");

        // Disable the menubar in this window.
        window.set_show_menubar(false);

        // Create the main `Grid`.
        let grid = Grid::new();
        grid.set_border_width(6);
        grid.set_row_spacing(6);
        grid.set_column_spacing(3);

        // Create the `Frame` for the list of catchments.
        let prefab_frame = Frame::new(Some("Possible Prefabs"));
        prefab_frame.set_label_align(0.04, 0.5);

        // Create the entries `Grid`.
        let entries_grid = Grid::new();
        entries_grid.set_border_width(6);
        entries_grid.set_row_spacing(6);
        entries_grid.set_column_spacing(3);
        prefab_frame.add(&entries_grid);

        // Create the list of entries.
        let mut entries = vec![];

        // For each catchment...
        for (index, catchment_index) in catchment_indexes.iter().enumerate() {

            // Create the label and the entry.
            let label = Label::new(Some(&*format!("Prefab's name for \"{}\\{}\":", pack_file_decoded.borrow().data.packed_files[*catchment_index].path[4], pack_file_decoded.borrow().data.packed_files[*catchment_index].path[5])));
            let entry = Entry::new();
            label.set_xalign(0.0);
            label.set_yalign(0.5);
            entry.set_placeholder_text("For example: one_ring_for_me");
            entry.set_hexpand(true);
            entry.set_has_frame(false);
            entry.set_size_request(200, 0);

            entries_grid.attach(&label, 0, index as i32, 1, 1);
            entries_grid.attach(&entry, 1, index as i32, 1, 1);

            // And push his entry to the list.
            entries.push(entry);
        }

        // Create the buttons.
        let button_box = ButtonBox::new(Orientation::Horizontal);
        button_box.set_layout(ButtonBoxStyle::End);
        button_box.set_spacing(6);

        let accept_button = Button::new_with_label("Accept");
        let cancel_button = Button::new_with_label("Cancel");

        // ButtonBox packing stuff...
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&accept_button, false, false, 0);

        // Grid packing stuff...
        grid.attach(&prefab_frame, 0, 0, 1, 1);
        grid.attach(&button_box, 0, 1, 1, 1);

        window.add(&grid);
        window.show_all();

        // Disable the accept button by default.
        accept_button.set_sensitive(false);

        // Get all the stuff inside one struct, so it's easier to pass it to the closures.
        let new_prefab_stuff = Self {
            window,
            entries,
            accept_button,
            cancel_button,
        };

        // Events for this to work.

        // Every time we change a character in the entry, check if the name is valid, and disable the "Accept"
        // button if it's invalid.
        for entry in &new_prefab_stuff.entries {
            entry.connect_changed(clone!(
                game_selected,
                new_prefab_stuff => move |entry| {

                    // If it's stupid but it works,...
                    new_prefab_stuff.accept_button.set_relief(ReliefStyle::None);
                    new_prefab_stuff.accept_button.set_relief(ReliefStyle::Normal);

                    // If our Game Selected have a path in settings...
                    if let Some(ref game_path) = game_selected.borrow().game_path {

                        // Get the "final" path for that prefab.
                        let prefab_name = entry.get_text().unwrap();
                        let prefab_path = game_path.to_path_buf().join(PathBuf::from(format!("assembly_kit/raw_data/art/prefabs/battle/custom_prefabs/{}.terry", prefab_name)));

                        // Create an attribute list for the entry.
                        let attribute_list = AttrList::new();
                        let invalid_color = Attribute::new_background((214 * 256) - 1, (75 * 256) - 1, (139 * 256) - 1).unwrap();

                        // If it already exist, allow it but mark it, so prefabs don't get overwritten by error.
                        if prefab_path.is_file() { attribute_list.insert(invalid_color); }

                        // Paint it like one of your french girls.
                        entry.set_attributes(&attribute_list);
                    }
                }
            ));
        }

        // If any of the entries has changed, check if we can enable it.
        new_prefab_stuff.accept_button.connect_property_relief_notify(clone!(
            new_prefab_stuff => move |accept_button| {

                // Create the var to check if the name is valid, and the vector to store the names.
                let mut invalid_name = false;
                let mut name_list = vec![];

                // For each entry...
                for entry in &new_prefab_stuff.entries {

                    // Get his text.
                    let name = entry.get_text().unwrap();

                    // If it has spaces, it's empty or it's repeated, it's automatically invalid.
                    if name.contains(' ') || name.is_empty() || name_list.contains(&name) {
                        invalid_name = true;
                        break;
                    }

                    // Otherwise, we add it to the list.
                    else { name_list.push(name); }
                }

                // We enable or disable the button, depending if the name is valid.
                if invalid_name { accept_button.set_sensitive(false); }
                else { accept_button.set_sensitive(true); }
            }
        ));

        // When we press the "Cancel" button, we close the window and re-enable the main window.
        new_prefab_stuff.cancel_button.connect_button_release_event(clone!(
            new_prefab_stuff,
            app_ui => move |_,_| {

                // Destroy the "New Prefab" window,
                new_prefab_stuff.window.destroy();

                // Restore the main window.
                app_ui.window.set_sensitive(true);
                Inhibit(false)
            }
        ));

        // We catch the destroy event of the window.
        new_prefab_stuff.window.connect_delete_event(clone!(
            new_prefab_stuff,
            app_ui => move |_, _| {

                // Destroy the "New Prefab" window,
                new_prefab_stuff.window.destroy();

                // Restore the main window.
                app_ui.window.set_sensitive(true);
                Inhibit(false)
            }
        ));

        // For some reason, the clone! macro is unable to clone this, so we clone it here.
        let catchment_indexes = catchment_indexes.to_vec();

        // If we hit the "Accept" button....
        new_prefab_stuff.accept_button.connect_button_release_event(clone!(
            app_ui,
            pack_file_decoded,
            game_selected,
            new_prefab_stuff => move |_,_| {

                // Get the base path of the game, to put the prefabs in his Assembly Kit directory.
                match game_selected.borrow().game_path {
                    Some(ref game_path) => {

                        // Get the list of all the names in the entries.
                        let name_list = new_prefab_stuff.entries.iter().filter_map(|entry| entry.get_text()).collect::<Vec<String>>();

                        // Try to create the prefabs with the provided names.
                        match packfile::create_prefab_from_catchment(
                            &name_list,
                            &game_path,
                            &catchment_indexes,
                            &pack_file_decoded,
                        ) {
                            Ok(result) => show_dialog(&app_ui.window, true, result),
                            Err(error) => show_dialog(&app_ui.window, false, error),
                        };
                    }

                    // If there is no game_path, stop and report error.
                    None => show_dialog(&app_ui.window, false, "The selected Game Selected doesn't have a path specified in the Settings."),
                }

                // Destroy the "New Prefab" window,
                new_prefab_stuff.window.destroy();

                // Re-enable the main window.
                app_ui.window.set_sensitive(true);
                Inhibit(false)
            }
        ));
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
*/

/// `SettingsDialog`: This struct holds all the relevant stuff for the Settings Dialog.
pub struct SettingsDialog {
    pub path_mymod_line_edit: Rc<RefCell<CppBox<LineEdit>>>,
    pub path_mymod_button: Rc<RefCell<CppBox<PushButton>>>,
    pub path_line_edits: Vec<Rc<RefCell<CppBox<LineEdit>>>>,
    pub path_buttons: Vec<Rc<RefCell<CppBox<PushButton>>>>,
    pub extra_default_game_combobox: CppBox<ComboBox>,
    pub extra_allow_editing_of_ca_packfiles: CppBox<CheckBox>,
    pub extra_check_updates_on_start: CppBox<CheckBox>,
    pub extra_check_schema_updates_on_start: CppBox<CheckBox>,
    pub extra_use_pfm_extracting_behavior: CppBox<CheckBox>,
    //pub theme_prefer_dark_theme: CheckButton,
    //pub theme_font_button: FontButton,
    pub cancel_button: *mut PushButton,
    pub accept_button: Rc<RefCell<*mut PushButton>>,
}

/// `MyModNewWindow`: This struct holds all the relevant stuff for "My Mod"'s New Mod Window.
#[derive(Clone, Debug)]
pub struct NewMyModDialog {
    pub mymod_game_combobox: *mut ComboBox,
    pub mymod_name_line_edit: *mut LineEdit,
    pub cancel_button: *mut PushButton,
    pub accept_button: *mut PushButton,
}

/// Implementation of `SettingsDialog`.
impl SettingsDialog {

    /// This function creates the entire settings window. It requires the application object to pass
    /// the window to. It returns the new Settings, or None if we are cancelling.
    pub fn create_settings_dialog(
        app_ui: &AppUI,
        settings: &Settings,
        supported_games: &[GameInfo]
    ) -> Option<Settings> {

        //-------------------------------------------------------------------------------------------//
        // Creating the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the Preferences Dialog.
        let mut dialog;
        unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

        // Change his title.
        dialog.set_window_title(&QString::from_std_str("Preferences"));

        // Set it Modal, so you can't touch the Main Window with this dialog open.
        dialog.set_modal(true);

        // Resize the Dialog.
        dialog.resize((750, 0));

        // Create the main Grid.
        let main_grid = GridLayout::new().into_raw();

        // Create the Paths Frame.
        let paths_frame = GroupBox::new(&QString::from_std_str("Paths")).into_raw();
        let mut paths_grid = GridLayout::new();

        // Create the MyMod's path stuff...
        let mymod_label = Label::new(&QString::from_std_str("MyMod's Path:")).into_raw();
        let mut mymod_line_edit = LineEdit::new(());
        let mut mymod_button = PushButton::new(&QString::from_std_str("..."));

        unsafe {

            // Configure the MyMod LineEdit.
            mymod_line_edit.set_placeholder_text(&QString::from_std_str("This is the folder where you want to store all \"MyMod\" related files."));

            // Add them to the grid.
            paths_grid.add_widget((mymod_label as *mut Widget, 0, 0, 1, 1));
            paths_grid.add_widget((mymod_line_edit.static_cast_mut() as *mut Widget, 0, 1, 1, 1));
            paths_grid.add_widget((mymod_button.static_cast_mut() as *mut Widget, 0, 2, 1, 1));
        }

        // For each game supported...
        let mut game_paths = vec![];
        let mut game_buttons = vec![];
        for (index, game_supported) in supported_games.iter().enumerate() {

            // Create his fields.
            let game_label = Label::new(&QString::from_std_str(&format!("TW: {} folder", game_supported.display_name))).into_raw();
            let mut game_line_edit = LineEdit::new(());
            let mut game_button = PushButton::new(&QString::from_std_str("..."));

            unsafe {

                // Configure the MyMod LineEdit.
                game_line_edit.set_placeholder_text(&QString::from_std_str(&*format!("This is the folder where you have {} installed.", game_supported.display_name)));

                // And add them to the grid.
                paths_grid.add_widget((game_label as *mut Widget, (index + 1) as i32, 0, 1, 1));
                paths_grid.add_widget((game_line_edit.static_cast_mut() as *mut Widget, (index + 1) as i32, 1, 1, 1));
                paths_grid.add_widget((game_button.static_cast_mut() as *mut Widget, (index + 1) as i32, 2, 1, 1));
            }

            // Add the LineEdit and Button to the list.
            game_paths.push(Rc::new(RefCell::new(game_line_edit)));
            game_buttons.push(Rc::new(RefCell::new(game_button)));
        }

        unsafe {

            // Add the Grid to the Frame, and the Frame to the Main Grid.
            paths_frame.as_mut().unwrap().set_layout(paths_grid.static_cast_mut() as *mut Layout);
            main_grid.as_mut().unwrap().add_widget((paths_frame as *mut Widget, 0, 0, 1, 2));

            // And the Main Grid to the Dialog...
            dialog.set_layout(main_grid as *mut Layout);
        }

        // Create the "Extra Settings" frame and Grid.
        let extra_settings_frame = GroupBox::new(&QString::from_std_str("Extra Settings")).into_raw();
        let extra_settings_grid = GridLayout::new().into_raw();

        // Create the "Default Game" Label and ComboBox.
        let default_game_label = Label::new(&QString::from_std_str("Default Game:")).into_raw();
        let mut default_game_combobox = ComboBox::new();
        let mut default_game_model = StandardItemModel::new(());
        unsafe { default_game_combobox.set_model(default_game_model.static_cast_mut()); }

        // Add the games to the ComboBox.
        for game in supported_games { default_game_combobox.add_item(&QString::from_std_str(&game.display_name)); }

        // Create the aditional CheckBoxes.
        let mut allow_editing_of_ca_packfiles_label = Label::new(&QString::from_std_str("Allow Editing of CA PackFiles:"));
        let mut check_updates_on_start_label = Label::new(&QString::from_std_str("Check Updates on Start:"));
        let mut check_schema_updates_on_start_label = Label::new(&QString::from_std_str("Check Schema Updates on Start:"));
        let mut use_pfm_extracting_behavior_label = Label::new(&QString::from_std_str("Use PFM Extracting Behavior:"));

        let mut allow_editing_of_ca_packfiles_checkbox = CheckBox::new(());
        let mut check_updates_on_start_checkbox = CheckBox::new(());
        let mut check_schema_updates_on_start_checkbox = CheckBox::new(());
        let mut use_pfm_extracting_behavior_checkbox = CheckBox::new(());

        // Tips for the checkboxes.
        allow_editing_of_ca_packfiles_checkbox.set_tool_tip(&QString::from_std_str("By default, only PackFiles of Type 'Mod' and 'Movie' are editables, as are the only ones used for modding.\nIf you enable this, you'll be able to edit 'Boot', 'Release' and 'Patch' PackFiles too. Just be careful of not writing over one of the game's original PackFiles!"));
        check_updates_on_start_checkbox.set_tool_tip(&QString::from_std_str("If you enable this, RPFM will check for updates at the start of the Program, and inform you if there is any update available.\nWhether download it or not is up to you."));
        check_schema_updates_on_start_checkbox.set_tool_tip(&QString::from_std_str("If you enable this, RPFM will check for Schema updates at the start of the Program,\nand allow you to automatically download it if there is any update available."));
        use_pfm_extracting_behavior_checkbox.set_tool_tip(&QString::from_std_str("By default, extracting a file/folder extracts just the file to wherever you want.\nIf you enable this, the file/folder will be extracted wherever you want UNDER HIS ENTIRE PATH.\nThat means that extracting a table go from 'myfolder/table_file' to 'myfolder/db/main_units_tables/table_file'."));

        // Also, for their labels.
        allow_editing_of_ca_packfiles_label.set_tool_tip(&QString::from_std_str("By default, only PackFiles of Type 'Mod' and 'Movie' are editables, as are the only ones used for modding.\nIf you enable this, you'll be able to edit 'Boot', 'Release' and 'Patch' PackFiles too. Just be careful of not writing over one of the game's original PackFiles!"));
        check_updates_on_start_label.set_tool_tip(&QString::from_std_str("If you enable this, RPFM will check for updates at the start of the Program, and inform you if there is any update available.\nWhether download it or not is up to you."));
        check_schema_updates_on_start_label.set_tool_tip(&QString::from_std_str("If you enable this, RPFM will check for Schema updates at the start of the Program,\nand allow you to automatically download it if there is any update available."));
        use_pfm_extracting_behavior_label.set_tool_tip(&QString::from_std_str("By default, extracting a file/folder extracts just the file to wherever you want.\nIf you enable this, the file/folder will be extracted wherever you want UNDER HIS ENTIRE PATH.\nThat means that extracting a table go from 'myfolder/table_file' to 'myfolder/db/main_units_tables/table_file'."));

        // Add the "Default Game" stuff to the Grid.
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((default_game_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((default_game_combobox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((allow_editing_of_ca_packfiles_label.into_raw() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((allow_editing_of_ca_packfiles_checkbox.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_updates_on_start_label.into_raw() as *mut Widget, 2, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 2, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_schema_updates_on_start_label.into_raw() as *mut Widget, 3, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((check_schema_updates_on_start_checkbox.static_cast_mut() as *mut Widget, 3, 1, 1, 1)); }

        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_pfm_extracting_behavior_label.into_raw() as *mut Widget, 4, 0, 1, 1)); }
        unsafe { extra_settings_grid.as_mut().unwrap().add_widget((use_pfm_extracting_behavior_checkbox.static_cast_mut() as *mut Widget, 4, 1, 1, 1)); }

        unsafe {

            // Add the Grid to the Frame, and the Frame to the Main Grid.
            extra_settings_frame.as_mut().unwrap().set_layout(extra_settings_grid as *mut Layout);
            main_grid.as_mut().unwrap().add_widget((extra_settings_frame as *mut Widget, 1, 1, 1, 1));
        }

        // Create the bottom ButtonBox.
        let mut button_box = DialogButtonBox::new(());
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.static_cast_mut() as *mut Widget, 2, 0, 1, 2)); }

        // Create the bottom Buttons.
        let restore_default_button;
        let cancel_button;
        let accept_button;

        // Add them to the Dialog.
        restore_default_button = button_box.add_button(dialog_button_box::StandardButton::RestoreDefaults);
        cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        //-------------------------------------------------------------------------------------------//
        // Preparations...
        //-------------------------------------------------------------------------------------------//
        let mymod_line_edit = Rc::new(RefCell::new(mymod_line_edit));
        let mymod_button = Rc::new(RefCell::new(mymod_button));
        let accept_button = Rc::new(RefCell::new(accept_button));

        //-------------------------------------------------------------------------------------------//
        // Slots for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for MyMods.
        let slot_select_mymod_path = SlotNoArgs::new(clone!(
            mymod_line_edit,
            mymod_button,
            accept_button => move || {
                update_entry_path(&mymod_line_edit, &mymod_button, &accept_button);
            }
        ));

        let mut slots_select_paths = vec![];
        let gp = game_paths.to_vec();
        let gb = game_buttons.to_vec();
        let paths_and_buttons = gp.iter().cloned().zip(gb.iter().cloned());
        for path in paths_and_buttons {
            slots_select_paths.push(SlotNoArgs::new(clone!(
                path,
                accept_button => move || {
                    update_entry_path(&path.0, &path.1, &accept_button);
                }
            )));
        }

        //-------------------------------------------------------------------------------------------//
        // Actions for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for Games.
        for (index, button) in game_buttons.iter().enumerate() {
            button.borrow().signals().released().connect(&slots_select_paths[index]);
        }

        // What happens when we hit the "..." button for MyMods.
        mymod_button.borrow().signals().released().connect(&slot_select_mymod_path);

        // What happens when we hit the "Cancel" button.
        unsafe { cancel_button.as_mut().unwrap().signals().released().connect(&dialog.slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { accept_button.borrow().as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let mut settings_dialog = Self {
            path_mymod_line_edit: mymod_line_edit,
            path_mymod_button: mymod_button,
            path_line_edits: game_paths,
            path_buttons: game_buttons,
            extra_default_game_combobox: default_game_combobox,
            extra_allow_editing_of_ca_packfiles: allow_editing_of_ca_packfiles_checkbox,
            extra_check_updates_on_start: check_updates_on_start_checkbox,
            extra_check_schema_updates_on_start: check_schema_updates_on_start_checkbox,
            extra_use_pfm_extracting_behavior: use_pfm_extracting_behavior_checkbox,
            cancel_button,
            accept_button,
        };

        //-------------------------------------------------------------------------------------------//
        // Loading data to the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Load the MyMod Path, if exists.
        settings_dialog.load_to_settings_dialog(&settings, supported_games);

        //-------------------------------------------------------------------------------------------//
        // Actions that must exectute at the end...
        //-------------------------------------------------------------------------------------------//
        let settings_dialog = Rc::new(RefCell::new(settings_dialog));

        // What happens when we hit the "Restore Default" action.
        let slot_restore_default = SlotNoArgs::new(clone!(
            settings_dialog => move || {

                let new_settings = Settings::new(supported_games);
                (*settings_dialog.borrow_mut()).load_to_settings_dialog(&new_settings, supported_games)
            }
        ));

        // What happens when we hit the "Restore Default" button.
        unsafe { restore_default_button.as_mut().unwrap().signals().released().connect(&slot_restore_default); }


        // Show the Dialog, save the current settings, and return them.
        if dialog.exec() == 1 { Some(settings_dialog.borrow().save_from_settings_dialog(supported_games)) }

        // Otherwise, return None.
        else { None }
    }

    /// This function loads the data from the Settings struct to the Settings Dialog.
    pub fn load_to_settings_dialog(
        &mut self,
        settings: &Settings,
        supported_games: &[GameInfo]
    ) {

        // Load the MyMod Path, if exists.
        self.path_mymod_line_edit.borrow_mut().set_text(&QString::from_std_str(&settings.paths.my_mods_base_path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy()));

        // Load the Game Paths, if they exists.
        for (index, path) in self.path_line_edits.iter_mut().enumerate() {
            path.borrow_mut().set_text(&QString::from_std_str(&settings.paths.game_paths[index].path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy()));
        }

        // Get the Default Game.
        for (index, game) in supported_games.iter().enumerate() {
            if game.folder_name == settings.default_game {
                self.extra_default_game_combobox.set_current_index(index as i32);
                break;
            }
        }

        // Load the Extra Stuff.
        self.extra_allow_editing_of_ca_packfiles.set_checked(settings.allow_editing_of_ca_packfiles);
        self.extra_check_updates_on_start.set_checked(settings.check_updates_on_start);
        self.extra_check_schema_updates_on_start.set_checked(settings.check_schema_updates_on_start);
        self.extra_use_pfm_extracting_behavior.set_checked(settings.use_pfm_extracting_behavior);
    }

    /// This function gets the data from the Settings Dialog and returns a Settings struct with that
    /// data in it.
    pub fn save_from_settings_dialog(&self, supported_games: &[GameInfo]) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new(supported_games);

        // Only if we have valid directories, we save them. Otherwise we wipe them out.
        settings.paths.my_mods_base_path = match Path::new(&self.path_mymod_line_edit.borrow().text().to_std_string()).is_dir() {
            true => Some(PathBuf::from(&self.path_mymod_line_edit.borrow().text().to_std_string())),
            false => None,
        };

        // For each entry, we get check if it's a valid directory and save it into `settings`.
        for (index, game) in self.path_line_edits.iter().enumerate() {
            settings.paths.game_paths[index].path = match Path::new(&game.borrow().text().to_std_string()).is_dir() {
                true => Some(PathBuf::from(&game.borrow().text().to_std_string())),
                false => None,
            };
        }

        // We get his game's folder, depending on the selected game.
        let index = self.extra_default_game_combobox.current_index();
        settings.default_game = supported_games[index as usize].folder_name.to_owned();

        // Get the Extra Settings.
        settings.allow_editing_of_ca_packfiles = self.extra_allow_editing_of_ca_packfiles.is_checked();
        settings.check_updates_on_start = self.extra_check_updates_on_start.is_checked();
        settings.check_schema_updates_on_start = self.extra_check_schema_updates_on_start.is_checked();
        settings.use_pfm_extracting_behavior = self.extra_use_pfm_extracting_behavior.is_checked();

        // Return the new Settings.
        settings
    }
}

/// Implementation of `MyModNewWindow`.
impl NewMyModDialog {

    /// This function creates the entire "New Mod" dialog. It returns the name of the mod and the
    /// folder_name of the game.
    pub fn create_new_mymod_dialog(
        app_ui: &AppUI,
        supported_games: &[GameInfo],
        settings: &Settings,
    ) -> Option<(String, String)> {

        //-------------------------------------------------------------------------------------------//
        // Creating the New MyMod Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the "New MyMod" Dialog.
        let mut dialog;
        unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

        // Change his title.
        dialog.set_window_title(&QString::from_std_str("New MyMod"));

        // Set it Modal, so you can't touch the Main Window with this dialog open.
        dialog.set_modal(true);

        // Resize the Dialog.
        dialog.resize((300, 0));

        // Create the main Grid.
        let main_grid = GridLayout::new().into_raw();

        // Create the Advices Frame.
        let advices_frame = Frame::new().into_raw();
        let mut advices_grid = GridLayout::new();

        // Create the "Advices" Label.
        let advices_label = Label::new(&QString::from_std_str("Things to take into account before creating a new mod:
	- Select the game you'll make the mod for.
	- Pick an simple name (it shouldn't end in *.pack).
	- If you want to use multiple words, use \"_\" instead of \" \".
	- You can't create a mod for a game that has no path set in the settings.")).into_raw();

        unsafe {

            // Add it to his frame.
            advices_grid.add_widget((advices_label as *mut Widget, 0, 0, 1, 1));

            // Add the Grid to the Frame, and the Frame to the Main Grid.
            advices_frame.as_mut().unwrap().set_layout(advices_grid.static_cast_mut() as *mut Layout);
            main_grid.as_mut().unwrap().add_widget((advices_frame as *mut Widget, 0, 0, 1, 2));

            // And the Main Grid to the Dialog...
            dialog.set_layout(main_grid as *mut Layout);
        }

        // Create the "MyMod's Name" Label and LineEdit.
        let mymod_name_label = Label::new(&QString::from_std_str("Name of the Mod:")).into_raw();
        let mymod_name_line_edit = LineEdit::new(()).into_raw();

        // Configure the "MyMod's Name" LineEdit.
        unsafe { mymod_name_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("For example: one_ring_for_me")); }

        // Create the "MyMod's Game" Label and ComboBox.
        let mymod_game_label = Label::new(&QString::from_std_str("Game of the Mod:")).into_raw();
        let mymod_game_combobox = ComboBox::new().into_raw();
        let mut mymod_game_model = StandardItemModel::new(());
        unsafe { mymod_game_combobox.as_mut().unwrap().set_model(mymod_game_model.static_cast_mut()); }

        // Add the games to the ComboBox.
        unsafe { for game in supported_games { mymod_game_combobox.as_mut().unwrap().add_item(&QString::from_std_str(&game.display_name)); } }

        // Add all the widgets to the main grid.
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_name_line_edit as *mut Widget, 1, 1, 1, 1)); }

        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((mymod_game_combobox as *mut Widget, 2, 1, 1, 1)); }

        // Create the bottom ButtonBox.
        let mut button_box = DialogButtonBox::new(());
        unsafe { main_grid.as_mut().unwrap().add_widget((button_box.static_cast_mut() as *mut Widget, 3, 0, 1, 2)); }

        // Create the bottom Buttons.
        let cancel_button;
        let accept_button;

        // Add them to the Dialog.
        cancel_button = button_box.add_button(dialog_button_box::StandardButton::Cancel);
        accept_button = button_box.add_button(dialog_button_box::StandardButton::Save);

        // Disable the "Accept" button by default.
        unsafe { accept_button.as_mut().unwrap().set_enabled(false); }

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let new_mymod_dialog = Self {
            mymod_game_combobox,
            mymod_name_line_edit,
            cancel_button,
            accept_button,
        };

        //-------------------------------------------------------------------------------------------//
        // Slots for the Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we change the name of the mod.
        let slot_mymod_line_edit_change = SlotNoArgs::new(clone!(
            new_mymod_dialog => move || {
                check_my_mod_validity(&new_mymod_dialog, &settings, &supported_games);
            }
        ));

        // What happens when we change the game of the mod.
        let slot_mymod_combobox_change = SlotNoArgs::new(clone!(
            new_mymod_dialog => move || {
                check_my_mod_validity(&new_mymod_dialog, &settings, &supported_games);
            }
        ));

        //-------------------------------------------------------------------------------------------//
        // Actions for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we change the name of the mod.
        unsafe { new_mymod_dialog.mymod_name_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_mymod_line_edit_change); }

        // What happens when we change the game of the mod.
        unsafe { new_mymod_dialog.mymod_game_combobox.as_mut().unwrap().signals().current_text_changed().connect(&slot_mymod_combobox_change); }

        // What happens when we hit the "Cancel" button.
        unsafe { new_mymod_dialog.cancel_button.as_mut().unwrap().signals().released().connect(&dialog.slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { new_mymod_dialog.accept_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }


        // Show the Dialog, save the current settings, and return them.
        if dialog.exec() == 1 {

            // Get the text from the LineEdit.
            let mod_name;
            unsafe { mod_name = QString::to_std_string(&new_mymod_dialog.mymod_name_line_edit.as_mut().unwrap().text()); }

            // Get the Game Selected in the ComboBox.
            let index;
            unsafe { index = new_mymod_dialog.mymod_game_combobox.as_mut().unwrap().current_index(); }
            let mod_game = supported_games[index as usize].folder_name.to_owned();

            // Return it.
            Some((mod_name, mod_game))
        }

        // Otherwise, return None.
        else { None }
    }
}

/// This function creates the entire "Rename" dialog. It returns the new name of the PackedFile, or
/// None if the dialog is canceled or closed.
pub fn create_rename_dialog(
    app_ui: &AppUI,
    name: &str,
) -> Option<String> {

    //-------------------------------------------------------------------------------------------//
    // Creating the Rename Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "New MyMod" Dialog.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

    // Change his title.
    dialog.set_window_title(&QString::from_std_str("Rename"));

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Resize the Dialog.
    dialog.resize((300, 0));

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "New Name" LineEdit.
    let mut new_name_line_edit = LineEdit::new(());

    // Set the current name as default.
    new_name_line_edit.set_text(&QString::from_std_str(name));

    // Create the "Rename" button.
    let rename_button = PushButton::new(&QString::from_std_str("Rename")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_name_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((rename_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the Rename Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "Rename" button.
    unsafe { rename_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "Rename" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let mod_name = QString::to_std_string(&new_name_line_edit.text());

        // Return the new name.
        Some(mod_name)
    }

    // Otherwise, return None.
    else { None }
}

/// This function creates the entire "New Folder" dialog. It returns the new name of the Folder, or
/// None if the dialog is canceled or closed.
pub fn create_new_folder_dialog(
    app_ui: &AppUI,
) -> Option<String> {

    //-------------------------------------------------------------------------------------------//
    // Creating the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // Create the "New Folder" Dialog.
    let mut dialog;
    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget); }

    // Change his title.
    dialog.set_window_title(&QString::from_std_str("New Folder"));

    // Set it Modal, so you can't touch the Main Window with this dialog open.
    dialog.set_modal(true);

    // Resize the Dialog.
    dialog.resize((300, 0));

    // Create the main Grid.
    let main_grid = GridLayout::new().into_raw();

    // Create the "New Folder" LineEdit.
    let mut new_folder_line_edit = LineEdit::new(());

    // Set the current name as default.
    new_folder_line_edit.set_text(&QString::from_std_str("new_folder"));

    // Create the "New Folder" button.
    let new_folder_button = PushButton::new(&QString::from_std_str("New Folder")).into_raw();

    // Add all the widgets to the main grid.
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_line_edit.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((new_folder_button as *mut Widget, 0, 1, 1, 1)); }

    // And the Main Grid to the Dialog...
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    //-------------------------------------------------------------------------------------------//
    // Actions for the New Folder Dialog...
    //-------------------------------------------------------------------------------------------//

    // What happens when we hit the "Rename" button.
    unsafe { new_folder_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    // Show the Dialog and, if we hit the "Rename" button...
    if dialog.exec() == 1 {

        // Get the text from the LineEdit.
        let mod_name = QString::to_std_string(&new_folder_line_edit.text());

        // Return the new name.
        Some(mod_name)
    }

    // Otherwise, return None.
    else { None }
}

/// This function takes care of updating the provided LineEdits with the selected path.
fn update_entry_path(
    line_edit: &Rc<RefCell<CppBox<LineEdit>>>,
    line_button: &Rc<RefCell<CppBox<PushButton>>>,
    accept_button: &Rc<RefCell<*mut PushButton>>,
) {

    // Create a parent, as we don't really care right now for it.
    let mut parent = Widget::new();

    // Create the FileDialog to get the path.
    let mut file_dialog;
    unsafe {
        file_dialog = FileDialog::new_unsafe((
            parent.into_raw(),
            &QString::from_std_str("Select Folder"),
        ));
    }

    // Set it to only search Folders.
    file_dialog.set_file_mode(FileMode::Directory);
    file_dialog.set_option(ShowDirsOnly);

    // Get the old Path, if exists.
    let old_path = QString::to_std_string(&line_edit.borrow().text());
    if !old_path.is_empty() && Path::new(&old_path).is_dir() {
        file_dialog.set_directory(&line_edit.borrow().text());
    }

    // Run it and expect a response (1 => Accept, 0 => Cancel).
    if file_dialog.exec() == 1 {

        // Get the path of the selected file and turn it in a Rust's PathBuf.
        let mut path: PathBuf = PathBuf::new();
        let path_qt = file_dialog.selected_files();
        for index in 0..path_qt.size() { path.push(path_qt.at(index).to_std_string()); }

        // Add the Path to the LineEdit.
        line_edit.borrow_mut().set_text(&QString::from_std_str(path.to_string_lossy()));
    }
}

/// Modification of `paint_entry`. In this one, the button painted red is the `Accept` button.
fn check_my_mod_validity(
    mymod_dialog: &NewMyModDialog,
    settings: &Settings,
    supported_games: &[GameInfo],
) {

    // Get the text from the LineEdit.
    let mod_name;
    unsafe { mod_name = QString::to_std_string(&mymod_dialog.mymod_name_line_edit.as_mut().unwrap().text()); }

    // Get the Game Selected in the ComboBox.
    let index;
    unsafe { index = mymod_dialog.mymod_game_combobox.as_mut().unwrap().current_index(); }
    let mod_game = supported_games[index as usize].folder_name.to_owned();

    // If there is text and it doesn't have whitespaces...
    if !mod_name.is_empty() && !mod_name.contains(' ') {

        // If we have "MyMod" path configured (we SHOULD have it to access this window, but just in case...).
        if let Some(ref mod_path) = settings.paths.my_mods_base_path {
            let mut mod_path = mod_path.clone();
            mod_path.push(mod_game);
            mod_path.push(format!("{}.pack", mod_name));

            // If a mod with that name for that game already exists, disable the "Accept" button.
            if mod_path.is_file() { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); }}

            // If the name is available, enable the `Accept` button.
            else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(true); } }
        }

        // If there is no "MyMod" path configured, disable the button.
        else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); } }
    }

    // If name is empty, disable the button.
    else { unsafe { mymod_dialog.accept_button.as_mut().unwrap().set_enabled(false); } }
}
