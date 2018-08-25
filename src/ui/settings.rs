// Here it goes all the stuff related with "Settings" and "My Mod" windows.
extern crate serde_json;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

use qt_widgets::check_box::CheckBox;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::dialog::Dialog;
use qt_widgets::{dialog_button_box, dialog_button_box::DialogButtonBox};
use qt_widgets::file_dialog::{FileDialog, FileMode, Option::ShowDirsOnly};
use qt_widgets::frame::Frame;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
use qt_widgets::label::Label;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::layout::Layout;
use qt_widgets::push_button::PushButton;
use qt_widgets::widget::Widget;

use qt_gui::standard_item_model::StandardItemModel;

use qt_core::connection::Signal;
use qt_core::slots::SlotNoArgs;
use cpp_utils::StaticCast;

use std::sync::mpsc::{Sender, Receiver};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::{Path, PathBuf};

use AppUI;
use Commands;
use QString;
use common::*;
use error::{ErrorKind, Result};
use settings::GameInfo;
use settings::Settings;
use settings::shortcuts::Shortcuts;
use super::shortcuts::ShortcutsDialog;
use super::show_dialog;

/// `SettingsDialog`: This struct holds all the relevant stuff for the Settings Dialog.
pub struct SettingsDialog {
    pub paths_mymod_line_edit: *mut LineEdit,
    pub paths_games_line_edits: Vec<*mut LineEdit>,
    pub ui_adjust_columns_to_content: *mut CheckBox,
    pub extra_default_game_combobox: *mut ComboBox,
    pub extra_allow_editing_of_ca_packfiles: *mut CheckBox,
    pub extra_check_updates_on_start: *mut CheckBox,
    pub extra_check_schema_updates_on_start: *mut CheckBox,
    pub extra_use_pfm_extracting_behavior: *mut CheckBox,
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
        supported_games: &[GameInfo],
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Result<Vec<u8>>>,
        receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>>>>>, 
    ) -> Option<Settings> {

        //-------------------------------------------------------------------------------------------//
        // Creating the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // Create the Preferences Dialog.
        let dialog;
        unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw(); }

        // Change his title.
        unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Preferences")); }

        // Set it Modal, so you can't touch the Main Window with this dialog open.
        unsafe { dialog.as_mut().unwrap().set_modal(true); }

        // Resize the Dialog.
        unsafe { dialog.as_mut().unwrap().resize((750, 0)); }

        // Create the main Grid.
        let main_grid = GridLayout::new().into_raw();
        unsafe { dialog.as_mut().unwrap().set_layout(main_grid as *mut Layout); }

        // Create the Paths Frame.
        let paths_frame = GroupBox::new(&QString::from_std_str("Paths")).into_raw();
        let mut paths_grid = GridLayout::new();

        // Create the MyMod's path stuff...
        let mymod_label = Label::new(&QString::from_std_str("MyMod's Path:")).into_raw();
        let mymod_line_edit = LineEdit::new(()).into_raw();
        let mymod_button = PushButton::new(&QString::from_std_str("...")).into_raw();

        // Configure the MyMod LineEdit.
        unsafe { mymod_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("This is the folder where you want to store all \"MyMod\" related files.")); }

        // Add them to the grid.
        unsafe { paths_grid.add_widget((mymod_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { paths_grid.add_widget((mymod_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { paths_grid.add_widget((mymod_button as *mut Widget, 0, 2, 1, 1)); }

        // For each game supported...
        let mut game_paths = vec![];
        let mut game_buttons = vec![];
        for (index, game_supported) in supported_games.iter().enumerate() {

            // Create his fields.
            let game_label = Label::new(&QString::from_std_str(&format!("TW: {} folder", game_supported.display_name))).into_raw();
            let game_line_edit = LineEdit::new(()).into_raw();
            let game_button = PushButton::new(&QString::from_std_str("...")).into_raw();

            // Configure the MyMod LineEdit.
            unsafe { game_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str(&*format!("This is the folder where you have {} installed.", game_supported.display_name))); }

            // And add them to the grid.
            unsafe { paths_grid.add_widget((game_label as *mut Widget, (index + 1) as i32, 0, 1, 1)); }
            unsafe { paths_grid.add_widget((game_line_edit as *mut Widget, (index + 1) as i32, 1, 1, 1)); }
            unsafe { paths_grid.add_widget((game_button as *mut Widget, (index + 1) as i32, 2, 1, 1)); }

            // Add the LineEdit and Button to the list.
            game_paths.push(game_line_edit);
            game_buttons.push(game_button);
        }

        // Create the "UI Settings" frame and Grid.
        let ui_settings_frame = GroupBox::new(&QString::from_std_str("UI Settings")).into_raw();
        let ui_settings_grid = GridLayout::new().into_raw();
        unsafe { ui_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

        // Create the UI options.
        let mut adjust_columns_to_content_label = Label::new(&QString::from_std_str("Adjust Columns to Content:"));
        let mut adjust_columns_to_content_checkbox = CheckBox::new(());

        let mut shortcuts_label = Label::new(&QString::from_std_str("See/Change Shortcuts:"));
        let mut shortcuts_button = PushButton::new(&QString::from_std_str("Shortcuts"));

        // Tips for the UI settings.
        let adjust_columns_to_content_tip = QString::from_std_str("If you enable this, when you open a DB Table or Loc File, all columns will be automatically resized depending on their content's size.\nOtherwise, columns will have a predefined size. Either way, you'll be able to resize them manually after the initial resize.\nNOTE: This KILLS PERFORMANCE in very big tables.");
        let shortcuts_tip = QString::from_std_str("See/change the shortcuts from here if you don't like them. Changes are applied on restart of the program.");
        adjust_columns_to_content_label.set_tool_tip(&adjust_columns_to_content_tip);
        adjust_columns_to_content_checkbox.set_tool_tip(&adjust_columns_to_content_tip);
        shortcuts_label.set_tool_tip(&shortcuts_tip);
        shortcuts_button.set_tool_tip(&shortcuts_tip);

        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((adjust_columns_to_content_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((adjust_columns_to_content_checkbox.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((shortcuts_label.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
        unsafe { ui_settings_grid.as_mut().unwrap().add_widget((shortcuts_button.static_cast_mut() as *mut Widget, 1, 1, 1, 1)); }

        // Create the "Extra Settings" frame and Grid.
        let extra_settings_frame = GroupBox::new(&QString::from_std_str("Extra Settings")).into_raw();
        let extra_settings_grid = GridLayout::new().into_raw();
        unsafe { extra_settings_grid.as_mut().unwrap().set_row_stretch(99, 10); }

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

        // Tips.
        let allow_editing_of_ca_packfiles_tip = QString::from_std_str("By default, only PackFiles of Type 'Mod' and 'Movie' are editables, as those are the only ones used for modding.\nIf you enable this, you'll be able to edit 'Boot', 'Release' and 'Patch' PackFiles too. Just be careful of not writing over one of the game's original PackFiles!");
        let check_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for updates at the start of the program, and inform you if there is any update available.\nWhether download it or not is up to you.");
        let check_schema_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for schema updates at the start of the program,\nand allow you to automatically download it if there is any update available.");
        let use_pfm_extracting_behavior_tip = QString::from_std_str("By default, extracting a file/folder extracts just the file to wherever you want.\nIf you enable this, the file/folder will be extracted wherever you want UNDER HIS ENTIRE PATH.\nThat means that extracting a table go from 'myfolder/table_file' to 'myfolder/db/main_units_tables/table_file'.");

        // Tips for the checkboxes.
        allow_editing_of_ca_packfiles_checkbox.set_tool_tip(&allow_editing_of_ca_packfiles_tip);
        check_updates_on_start_checkbox.set_tool_tip(&check_updates_on_start_tip);
        check_schema_updates_on_start_checkbox.set_tool_tip(&check_schema_updates_on_start_tip);
        use_pfm_extracting_behavior_checkbox.set_tool_tip(&use_pfm_extracting_behavior_tip);

        // Also, for their labels.
        allow_editing_of_ca_packfiles_label.set_tool_tip(&allow_editing_of_ca_packfiles_tip);
        check_updates_on_start_label.set_tool_tip(&check_updates_on_start_tip);
        check_schema_updates_on_start_label.set_tool_tip(&check_schema_updates_on_start_tip);
        use_pfm_extracting_behavior_label.set_tool_tip(&use_pfm_extracting_behavior_tip);

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

        // Add the Path's grid to his Frame, and his Frame to the Main Grid.
        unsafe { paths_frame.as_mut().unwrap().set_layout(paths_grid.static_cast_mut() as *mut Layout); }
        unsafe { main_grid.as_mut().unwrap().add_widget((paths_frame as *mut Widget, 0, 0, 1, 2)); }

        // Add the Grid to the Frame, and the Frame to the Main Grid.
        unsafe { ui_settings_frame.as_mut().unwrap().set_layout(ui_settings_grid as *mut Layout); }
        unsafe { extra_settings_frame.as_mut().unwrap().set_layout(extra_settings_grid as *mut Layout); }
        unsafe { main_grid.as_mut().unwrap().add_widget((ui_settings_frame as *mut Widget, 1, 0, 1, 1)); }
        unsafe { main_grid.as_mut().unwrap().add_widget((extra_settings_frame as *mut Widget, 1, 1, 1, 1)); }

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
        // Slots for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for MyMods.
        let slot_select_mymod_path = SlotNoArgs::new(move || {
            update_entry_path(mymod_line_edit, dialog);
        });

        // What happens when we hit any of the "..." buttons for the games.
        let mut slots_select_paths = vec![];
        for path in &game_paths {
            slots_select_paths.push(SlotNoArgs::new(move || {
                update_entry_path(*path, dialog);
            }));
        }

        // What happens when we hit the "Shortcuts" button.
        let slot_shortcuts = SlotNoArgs::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move || {

                // Try to get the current Shortcuts.
                sender_qt.send(Commands::GetShortcuts).unwrap();
                let old_shortcuts: Shortcuts = match check_message_validity_recv(&receiver_qt) {
                    Ok(data) => data,
                    Err(_) => panic!(THREADS_MESSAGE_ERROR)
                };

                // Create the Shortcuts Dialog. If we got new shortcuts...
                if let Some(shortcuts) = ShortcutsDialog::create_shortcuts_dialog(dialog, &old_shortcuts) {

                    // Send the signal to save them.
                    sender_qt.send(Commands::SetShortcuts).unwrap();
                    sender_qt_data.send(serde_json::to_vec(&shortcuts).map_err(From::from)).unwrap();

                    // Wait until you got a response.
                    let response: Result<()> = check_message_validity_recv(&receiver_qt);

                    // If we got an error...
                    if let Err(error) = response {

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If there was and IO error while saving the settings, report it.
                            ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound | ErrorKind::IOGeneric => show_dialog(app_ui.window, false, error.kind()),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }
                }
            }
        ));

        //-------------------------------------------------------------------------------------------//
        // Actions for the Settings Dialog...
        //-------------------------------------------------------------------------------------------//

        // What happens when we hit the "..." button for MyMods.
        unsafe { mymod_button.as_mut().unwrap().signals().released().connect(&slot_select_mymod_path); }

        // What happens when we hit the "..." button for Games.
        for (index, button) in game_buttons.iter().enumerate() {
            unsafe { button.as_mut().unwrap().signals().released().connect(&slots_select_paths[index]); }
        }

        // What happens when we hit the "Shortcuts" button.
        shortcuts_button.signals().released().connect(&slot_shortcuts);

        // What happens when we hit the "Cancel" button.
        unsafe { cancel_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().close()); }

        // What happens when we hit the "Accept" button.
        unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().accept()); }

        //-------------------------------------------------------------------------------------------//
        // Put all the important things together...
        //-------------------------------------------------------------------------------------------//

        let mut settings_dialog = Self {
            paths_mymod_line_edit: mymod_line_edit,
            paths_games_line_edits: game_paths.to_vec(),
            ui_adjust_columns_to_content: adjust_columns_to_content_checkbox.into_raw(),
            extra_default_game_combobox: default_game_combobox.into_raw(),
            extra_allow_editing_of_ca_packfiles: allow_editing_of_ca_packfiles_checkbox.into_raw(),
            extra_check_updates_on_start: check_updates_on_start_checkbox.into_raw(),
            extra_check_schema_updates_on_start: check_schema_updates_on_start_checkbox.into_raw(),
            extra_use_pfm_extracting_behavior: use_pfm_extracting_behavior_checkbox.into_raw(),
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
        unsafe { if dialog.as_mut().unwrap().exec() == 1 { Some(settings_dialog.borrow().save_from_settings_dialog(supported_games)) }

        // Otherwise, return None.
        else { None } }
    }

    /// This function loads the data from the Settings struct to the Settings Dialog.
    pub fn load_to_settings_dialog(
        &mut self,
        settings: &Settings,
        supported_games: &[GameInfo]
    ) {

        // Load the MyMod Path, if exists.
        unsafe { self.paths_mymod_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&settings.paths.my_mods_base_path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }

        // Load the Game Paths, if they exists.
        for (index, path) in self.paths_games_line_edits.iter_mut().enumerate() {
            unsafe { path.as_mut().unwrap().set_text(&QString::from_std_str(&settings.paths.game_paths[index].path.clone().unwrap_or_else(||PathBuf::new()).to_string_lossy())); }
        }

        // Get the Default Game.
        for (index, game) in supported_games.iter().enumerate() {
            if game.folder_name == settings.default_game {
                unsafe { self.extra_default_game_combobox.as_mut().unwrap().set_current_index(index as i32); }
                break;
            }
        }

        // Load the UI Stuff.
        unsafe { self.ui_adjust_columns_to_content.as_mut().unwrap().set_checked(settings.adjust_columns_to_content); }

        // Load the Extra Stuff.
        unsafe { self.extra_allow_editing_of_ca_packfiles.as_mut().unwrap().set_checked(settings.allow_editing_of_ca_packfiles); }
        unsafe { self.extra_check_updates_on_start.as_mut().unwrap().set_checked(settings.check_updates_on_start); }
        unsafe { self.extra_check_schema_updates_on_start.as_mut().unwrap().set_checked(settings.check_schema_updates_on_start); }
        unsafe { self.extra_use_pfm_extracting_behavior.as_mut().unwrap().set_checked(settings.use_pfm_extracting_behavior); }
    }

    /// This function gets the data from the Settings Dialog and returns a Settings struct with that
    /// data in it.
    pub fn save_from_settings_dialog(&self, supported_games: &[GameInfo]) -> Settings {

        // Create a new Settings.
        let mut settings = Settings::new(supported_games);

        // Only if we have a valid directory, we save it. Otherwise we wipe it out.
        let my_mod_new_path;
        unsafe { my_mod_new_path = PathBuf::from(self.paths_mymod_line_edit.as_mut().unwrap().text().to_std_string()); }
        settings.paths.my_mods_base_path = match my_mod_new_path.is_dir() {
            true => Some(my_mod_new_path),
            false => None,
        };

        // For each entry, we get check if it's a valid directory and save it into Settings.
        for (index, game) in self.paths_games_line_edits.iter().enumerate() {
            let new_path;
            unsafe { new_path = PathBuf::from(game.as_mut().unwrap().text().to_std_string()); }
            settings.paths.game_paths[index].path = match new_path.is_dir() {
                true => Some(new_path),
                false => None,
            };
        }

        // We get his game's folder, depending on the selected game.
        let index;
        unsafe { index = self.extra_default_game_combobox.as_mut().unwrap().current_index() as usize; }
        settings.default_game = supported_games[index].folder_name.to_owned();

        // Get the UI Settings.
        unsafe { settings.adjust_columns_to_content = self.ui_adjust_columns_to_content.as_mut().unwrap().is_checked(); }

        // Get the Extra Settings.
        unsafe { settings.allow_editing_of_ca_packfiles = self.extra_allow_editing_of_ca_packfiles.as_mut().unwrap().is_checked(); }
        unsafe { settings.check_updates_on_start = self.extra_check_updates_on_start.as_mut().unwrap().is_checked(); }
        unsafe { settings.check_schema_updates_on_start = self.extra_check_schema_updates_on_start.as_mut().unwrap().is_checked(); }
        unsafe { settings.use_pfm_extracting_behavior = self.extra_use_pfm_extracting_behavior.as_mut().unwrap().is_checked(); }

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
        unsafe { for game in supported_games { if game.display_name != "Arena" { mymod_game_combobox.as_mut().unwrap().add_item(&QString::from_std_str(&game.display_name)); }} }

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

/// This function takes care of updating the provided LineEdits with the selected path.
fn update_entry_path(
    line_edit: *mut LineEdit,
    dialog: *mut Dialog,
) {

    // Create the FileDialog to get the path.
    let mut file_dialog;
    unsafe {
        file_dialog = FileDialog::new_unsafe((
            dialog as *mut Widget,
            &QString::from_std_str("Select Folder"),
        ));
    }

    // Set it to only search Folders.
    file_dialog.set_file_mode(FileMode::Directory);
    file_dialog.set_option(ShowDirsOnly);

    // Get the old Path, if exists.
    let old_path;
    unsafe { old_path = line_edit.as_mut().unwrap().text().to_std_string(); }

    // If said path is not empty, and is a dir, set it as the initial directory.
    if !old_path.is_empty() && Path::new(&old_path).is_dir() {
        unsafe { file_dialog.set_directory(&line_edit.as_mut().unwrap().text()); }
    }

    // Run it and expect a response (1 => Accept, 0 => Cancel).
    if file_dialog.exec() == 1 {

        // Get the path of the selected file.
        let selected_files = file_dialog.selected_files();
        let path = selected_files.at(0);

        // Add the Path to the LineEdit.
        unsafe { line_edit.as_mut().unwrap().set_text(&path); }
    }
}

/// Check if the new MyMod's name is valid or not, disabling or enabling the "Accept" button in response.
fn check_my_mod_validity(
    mymod_dialog: &NewMyModDialog,
    settings: &Settings,
    supported_games: &[GameInfo],
) {

    // Get the text from the LineEdit.
    let mod_name;
    unsafe { mod_name = mymod_dialog.mymod_name_line_edit.as_mut().unwrap().text().to_std_string(); }

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

/*

/// `NewPrefabWindow`: This struct holds all the relevant stuff for the "New Prefab" window to work.
#[derive(Clone, Debug)]
pub struct NewPrefabWindow {
    pub window: ApplicationWindow,
    pub entries: Vec<Entry>,
    pub accept_button: Button,
    pub cancel_button: Button,
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
                            Ok(result) => show_dialog(app_ui.window, true, result),
                            Err(error) => show_dialog(app_ui.window, false, error),
                        };
                    }

                    // If there is no game_path, stop and report error.
                    None => show_dialog(app_ui.window, false, "The selected Game Selected doesn't have a path specified in the Settings."),
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

*/
