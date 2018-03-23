// This is the main file of RPFM. Here is the main loop that builds the UI and controls
// his events.

// Disable these two clippy linters. They throw a lot of false positives, and it's a pain in the ass
// to separate their warnings from the rest.
#![allow(doc_markdown,useless_format)]

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate failure;
extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate gio;
extern crate pango;
extern crate sourceview;
extern crate num;
extern crate restson;
extern crate url;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::{
    File, DirBuilder, copy, remove_file, remove_dir_all
};
use std::io::Write;
use std::env::args;

use failure::Error;
use url::Url;
use restson::RestClient;
use gdk::Gravity;
use gio::prelude::*;
use gio::{
    SimpleAction, Menu, MenuExt, MenuModel
};
use gtk::prelude::*;
use gtk::{
    AboutDialog, Builder, WindowPosition, ApplicationWindow, FileFilter, Grid,
    TreeView, TreeSelection, TreeStore, MessageDialog, ScrolledWindow, Application,
    CellRendererText, TreeViewColumn, Popover, Entry, Button, Image, ListStore, ResponseType,
    ShortcutsWindow, ToVariant, Statusbar, FileChooserNative, FileChooserAction
};
use sourceview::{
    Buffer, BufferExt, View, ViewExt, Language, LanguageManager, LanguageManagerExt
};

use common::coding_helpers;
use common::*;
use packfile::*;
use packfile::packfile::PackFile;
use packedfile::SerializableToCSV;
use packedfile::loc::Loc;
use packedfile::loc::LocData;
use packedfile::db::DB;
use packedfile::db::DBHeader;
use packedfile::db::DBData;
use packedfile::db::schemas::*;
use packedfile::db::schemas_importer::*;
use packedfile::rigidmodel::RigidModel;
use settings::*;
use ui::packedfile_db::*;
use ui::packedfile_loc::*;
use ui::settings::*;
use updater::LastestRelease;

/// This macro is used to clone the variables into the closures without the compiler complaining.
/// This should be BEFORE the `mod xxx` stuff, so submodules can use it too.
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

mod common;
mod ui;
mod packfile;
mod packedfile;
mod settings;
mod updater;


// This constant gets RPFM's version from the "Cargo.toml" file, so we don't have to change it
// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

// This constant is used to enable or disable the generation of a new Schema file in compile time.
// If you don't want to explicity create a new Schema for a game, leave this disabled.
const GENERATE_NEW_SCHEMA: bool = false;

// This enum represent the current "Operational Mode" for RPFM. The allowed modes are:
// - `Normal`: Use the default behavior for everything. This is the Default mode.
// - `MyMod`: Use the `MyMod` specific behavior. This mode is used when you have a "MyMod" selected.
//          This mode holds a tuple `(game_folder_name, mod_name)`. `game_folder_name` is the
//          folder name for that game in "MyMod"'s folder, like "warhammer_2" or "rome_2").
//          `mod_name` is the name of the PackFile with `.pack` at the end.
#[derive(Clone)]
enum Mode {
    MyMod{ game_folder_name: String, mod_name: String },
    Normal,
}

// This struct will store almost the entirety of the UI stuff, so it's not a fucking chaos when
// going inside/outside closures. The exceptions for this struct is stuff generated after RPFM is
// started, like the TreeView for DB PackedFiles or the DB Decoder View.
#[derive(Clone)]
struct AppUI {

    // Main window.
    window: ApplicationWindow,

    // MenuBar at the top of the Window.
    menu_bar: Menu,

    // Section of the "MyMod" menu.
    my_mod_list: Menu,

    // Shortcut window.
    shortcuts_window: ShortcutsWindow,

    // This is the box where all the PackedFile Views are created.
    packed_file_data_display: Grid,

    // "About" window.
    about_window: AboutDialog,

    // Informative dialogs.
    error_dialog: MessageDialog,
    success_dialog: MessageDialog,
    unsaved_dialog: MessageDialog,
    delete_my_mod_dialog: MessageDialog,
    check_updates_dialog: MessageDialog,

    // Popover for renaming PackedFiles and folders.
    rename_popover: Popover,

    // Text entry for the "Rename" Popover.
    rename_popover_text_entry: Entry,

    // Status bar at the bottom of the program. To show informative messages.
    status_bar: Statusbar,

    // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
    folder_tree_view: TreeView,
    folder_tree_store: TreeStore,
    folder_tree_selection: TreeSelection,

    // Context Menu Popover for `folder_tree_view`. It's build from a Model, stored here too.
    folder_tree_view_context_menu: Popover,
    folder_tree_view_context_menu_model: MenuModel,

    // Actions of RPFM's MenuBar.
    menu_bar_new_packfile: SimpleAction,
    menu_bar_open_packfile: SimpleAction,
    menu_bar_save_packfile: SimpleAction,
    menu_bar_save_packfile_as: SimpleAction,
    menu_bar_preferences: SimpleAction,
    menu_bar_quit: SimpleAction,
    menu_bar_generate_dependency_pack_wh2: SimpleAction,
    menu_bar_patch_siege_ai_wh2: SimpleAction,
    menu_bar_generate_dependency_pack_wh: SimpleAction,
    menu_bar_patch_siege_ai_wh: SimpleAction,
    menu_bar_check_updates: SimpleAction,
    menu_bar_about: SimpleAction,
    menu_bar_change_packfile_type: SimpleAction,
    menu_bar_my_mod_new: SimpleAction,
    menu_bar_my_mod_delete: SimpleAction,
    menu_bar_my_mod_install: SimpleAction,
    menu_bar_my_mod_uninstall: SimpleAction,
    menu_bar_change_game_selected: SimpleAction,

    // Actions of the Context Menu for `folder_tree_view`.
    folder_tree_view_add_file: SimpleAction,
    folder_tree_view_add_folder: SimpleAction,
    folder_tree_view_add_from_packfile: SimpleAction,
    folder_tree_view_delete_packedfile: SimpleAction,
    folder_tree_view_extract_packedfile: SimpleAction,
}

/// One Function to rule them all, One Function to find them,
/// One Function to bring them all and in the darkness bind them.
fn build_ui(application: &Application) {

    // We import the Glade design and get all the UI objects into variables.
    let glade_design = include_str!("gtk/main.glade");
    let help_window = include_str!("gtk/help.ui");
    let menus = include_str!("gtk/menus.ui");
    let builder = Builder::new_from_string(glade_design);

    // We add all the UI onjects to the same builder. You know, one to rule them all.
    builder.add_from_string(help_window).unwrap();
    builder.add_from_string(menus).unwrap();

    // The Context Menu Popover for `folder_tree_view` it's a little tricky to get. We need to
    // get the stuff it's based on and then create it and put it into the AppUI.
    let folder_tree_view = builder.get_object("gtk_folder_tree_view").unwrap();
    let folder_tree_view_context_menu_model = builder.get_object("context_menu_packfile").unwrap();
    let folder_tree_view_context_menu = Popover::new_from_model(Some(&folder_tree_view), &folder_tree_view_context_menu_model);

    // First, create the AppUI to hold all the UI stuff. All the stuff here it's from the executable
    // so we can unwrap it without any problems.
    let app_ui = AppUI {

        // Main window.
        window: builder.get_object("gtk_window").unwrap(),

        // MenuBar at the top of the Window.
        menu_bar: builder.get_object("menubar").unwrap(),

        // Section of the "MyMod" menu.
        my_mod_list: builder.get_object("my-mod-list").unwrap(),

        // Shortcut window.
        shortcuts_window: builder.get_object("shortcuts-main-window").unwrap(),

        // This is the box where all the PackedFile Views are created.
        packed_file_data_display: builder.get_object("gtk_packed_file_data_display").unwrap(),

        // "About" window.
        about_window: builder.get_object("gtk_window_about").unwrap(),

        // Informative dialogs.
        error_dialog: builder.get_object("gtk_error_dialog").unwrap(),
        success_dialog: builder.get_object("gtk_success_dialog").unwrap(),
        unsaved_dialog: builder.get_object("gtk_unsaved_dialog").unwrap(),
        delete_my_mod_dialog: builder.get_object("gtk_delete_my_mod_dialog").unwrap(),
        check_updates_dialog: builder.get_object("gtk_check_updates_dialog").unwrap(),

        // Popover for renaming PackedFiles and folders.
        rename_popover: builder.get_object("gtk_rename_popover").unwrap(),

        // Text entry for the "Rename" Popover.
        rename_popover_text_entry: builder.get_object("gtk_rename_popover_text_entry").unwrap(),

        // Status bar at the bottom of the program. To show informative messages.
        status_bar: builder.get_object("gtk_bottom_status_bar").unwrap(),

        // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
        folder_tree_view,
        folder_tree_store: TreeStore::new(&[String::static_type()]),
        folder_tree_selection: builder.get_object("gtk_folder_tree_view_selection").unwrap(),

        // Context Menu Popover for `folder_tree_view`. It's build from a Model, stored here too.
        folder_tree_view_context_menu,
        folder_tree_view_context_menu_model,

        // Actions of RPFM's MenuBar.
        menu_bar_new_packfile: SimpleAction::new("new-packfile", None),
        menu_bar_open_packfile: SimpleAction::new("open-packfile", None),
        menu_bar_save_packfile: SimpleAction::new("save-packfile", None),
        menu_bar_save_packfile_as: SimpleAction::new("save-packfile-as", None),
        menu_bar_preferences: SimpleAction::new("preferences", None),
        menu_bar_quit: SimpleAction::new("quit", None),
        menu_bar_generate_dependency_pack_wh2: SimpleAction::new("generate-dependency-pack-wh2", None),
        menu_bar_patch_siege_ai_wh2: SimpleAction::new("patch-siege-ai-wh2", None),
        menu_bar_generate_dependency_pack_wh: SimpleAction::new("generate-dependency-pack-wh", None),
        menu_bar_patch_siege_ai_wh: SimpleAction::new("patch-siege-ai-wh", None),
        menu_bar_check_updates: SimpleAction::new("check-updates", None),
        menu_bar_about: SimpleAction::new("about", None),
        menu_bar_change_packfile_type: SimpleAction::new_stateful("change-packfile-type", glib::VariantTy::new("s").ok(), &"mod".to_variant()),
        menu_bar_my_mod_new: SimpleAction::new("my-mod-new", None),
        menu_bar_my_mod_delete: SimpleAction::new("my-mod-delete", None),
        menu_bar_my_mod_install: SimpleAction::new("my-mod-install", None),
        menu_bar_my_mod_uninstall: SimpleAction::new("my-mod-uninstall", None),
        menu_bar_change_game_selected: SimpleAction::new_stateful("change-game-selected", glib::VariantTy::new("s").ok(), &"warhammer_2".to_variant()),

        // Actions of the Context Menu for `folder_tree_view`.
        folder_tree_view_add_file: SimpleAction::new("add-file", None),
        folder_tree_view_add_folder: SimpleAction::new("add-folder", None),
        folder_tree_view_add_from_packfile: SimpleAction::new("add-from-packfile", None),
        folder_tree_view_delete_packedfile: SimpleAction::new("delete-packedfile", None),
        folder_tree_view_extract_packedfile: SimpleAction::new("extract-packedfile", None),
    };

    // Set the main menu bar for the app. This one can appear in all the windows and needs to be
    // enabled or disabled per window.
    application.set_menubar(&app_ui.menu_bar);

    // Config stuff for `app_ui.folder_tree_view`.
    app_ui.folder_tree_view.set_model(Some(&app_ui.folder_tree_store));

    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();
    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);

    app_ui.folder_tree_view.append_column(&column);
    app_ui.folder_tree_view.set_margin_bottom(10);
    app_ui.folder_tree_view.set_enable_search(false);

    // Config stuff for `app_ui.shortcuts_window`.
    app_ui.shortcuts_window.set_title("Shortcuts");
    app_ui.shortcuts_window.set_size_request(600, 400);
    app_ui.window.set_help_overlay(Some(&app_ui.shortcuts_window));

    // Config stuff for `app_ui.about_window`.
    app_ui.about_window.set_program_name("Rusted PackFile Manager");
    app_ui.about_window.set_version(VERSION);
    app_ui.about_window.set_license_type(gtk::License::MitX11);
    app_ui.about_window.set_website("https://github.com/Frodo45127/rpfm");
    app_ui.about_window.set_website_label("Source code and more info here :)");
    app_ui.about_window.set_comments(Some("Made by modders, for modders."));

    app_ui.about_window.add_credit_section("Created and Programmed by", &["Frodo45127"]);
    app_ui.about_window.add_credit_section("Icon by", &["Maruka"]);
    app_ui.about_window.add_credit_section("RigidModel research by", &["Mr.Jox", "Der Spaten", "Maruka", "Frodo45127"]);
    app_ui.about_window.add_credit_section("DB Schemas by", &["PFM team"]);
    app_ui.about_window.add_credit_section("Windows's theme", &["\"Materia for GTK3\" by nana-4"]);
    app_ui.about_window.add_credit_section("Special thanks to", &["- PFM team (for providing the community\n   with awesome modding tools).", "- CA (for being a mod-friendly company)."]);

    // Config stuff for MenuBar Actions.
    application.add_action(&app_ui.menu_bar_new_packfile);
    application.add_action(&app_ui.menu_bar_open_packfile);
    application.add_action(&app_ui.menu_bar_save_packfile);
    application.add_action(&app_ui.menu_bar_save_packfile_as);
    application.add_action(&app_ui.menu_bar_preferences);
    application.add_action(&app_ui.menu_bar_quit);
    application.add_action(&app_ui.menu_bar_generate_dependency_pack_wh2);
    application.add_action(&app_ui.menu_bar_patch_siege_ai_wh2);
    application.add_action(&app_ui.menu_bar_generate_dependency_pack_wh);
    application.add_action(&app_ui.menu_bar_patch_siege_ai_wh);
    application.add_action(&app_ui.menu_bar_about);
    application.add_action(&app_ui.menu_bar_check_updates);
    application.add_action(&app_ui.menu_bar_change_packfile_type);
    application.add_action(&app_ui.menu_bar_my_mod_new);
    application.add_action(&app_ui.menu_bar_my_mod_delete);
    application.add_action(&app_ui.menu_bar_my_mod_install);
    application.add_action(&app_ui.menu_bar_my_mod_uninstall);
    application.add_action(&app_ui.menu_bar_change_game_selected);

    // Config stuff for ´folder_tree_view´ specific Actions.
    application.add_action(&app_ui.folder_tree_view_add_file);
    application.add_action(&app_ui.folder_tree_view_add_folder);
    application.add_action(&app_ui.folder_tree_view_add_from_packfile);
    application.add_action(&app_ui.folder_tree_view_delete_packedfile);
    application.add_action(&app_ui.folder_tree_view_extract_packedfile);

    // Some Accels need to be specified here. Don't know why, but otherwise they do not work.
    application.set_accels_for_action("app.add-file", &["<Primary>a"]);
    application.set_accels_for_action("app.add-folder", &["<Primary>d"]);
    application.set_accels_for_action("app.add-from-packfile", &["<Primary>w"]);
    application.set_accels_for_action("app.delete-packedfile", &["<Primary>Delete"]);
    application.set_accels_for_action("app.extract-packedfile", &["<Primary>e"]);
    application.set_accels_for_action("win.show-help-overlay", &["<Primary><Shift>h"]);

    // We enable D&D PackFiles to `app_ui.folder_tree_view` to open them.
    let targets = vec![gtk::TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP, 0)];
    app_ui.folder_tree_view.drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);

    // Check for updates at the start. Currently this hangs the UI, so do it before showing the UI.
    check_updates(None, &app_ui.status_bar);

    // Then we display the "Tips" text.
    ui::display_help_tips(&app_ui.packed_file_data_display);

    // We link the main ApplicationWindow to the application.
    app_ui.window.set_application(Some(application));

    // We bring up the main window.
    app_ui.window.show_all();

    // We center the window after being loaded, so the load of the display tips don't move it to the left.
    app_ui.window.set_position(WindowPosition::Center);
    app_ui.window.set_gravity(Gravity::Center);

    // This is to get the new schemas. It's controlled by a global const.
    if GENERATE_NEW_SCHEMA {

        // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
        // The second one should contain all the tables of the game, extracted directly from `data.pack`.
        let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_schemas/");
        let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_tables/");
        match import_schema(&assembly_kit_schemas_path, &testing_tables_path) {
            Ok(_) => ui::show_dialog(&app_ui.success_dialog, format!("Success creating a new DB Schema file.")),
            Err(error) => return ui::show_dialog(&app_ui.error_dialog, format!("Error while creating a new DB Schema file:\n{}", error.cause())),
        }
    }

    // This variable is used to "Lock" the "Decode on select" feature of `app_ui.folder_tree_view`.
    // We need it to lock this feature when we open a secondary PackFile and want to import some
    // PackedFiles to our opened PackFile.
    let is_folder_tree_view_locked = Rc::new(RefCell::new(false));

    // Here we define the `Ok` and `Accept` responses for GTK, as it seems Restson causes them to
    // fail to compile if we get them to i32 directly in the `if` statement.
    // NOTE: For some bizarre reason, GTKFileChoosers return `Ok`, while native ones return `Accept`.
    let gtk_response_ok: i32 = ResponseType::Ok.into();
    let gtk_response_accept: i32 = ResponseType::Accept.into();

    // We need two PackFiles:
    // - `pack_file_decoded`: This one will hold our opened PackFile.
    // - `pack_file_decoded_extra`: This one will hold the PackFile opened for `app_ui.add_from_packfile`.
    let pack_file_decoded = Rc::new(RefCell::new(PackFile::new()));
    let pack_file_decoded_extra = Rc::new(RefCell::new(PackFile::new()));

    // We load the settings here, and in case they doesn't exist, we create them.
    let settings = Rc::new(RefCell::new(Settings::load().unwrap_or_else(|_|Settings::new())));

    // We load the list of Supported Games here.
    // TODO: Move this to a const when const fn reach stable in Rust.
    let supported_games = Rc::new(RefCell::new(GameInfo::new()));

    // Load the GTK Settings, like the Theme and Font used.
    load_gtk_settings(&app_ui.window, &settings.borrow());

    // We prepare the schema object to hold an Schema, leaving it as `None` by default.
    let schema: Rc<RefCell<Option<Schema>>> = Rc::new(RefCell::new(None));

    // This specifies the "Operational Mode" RPFM should use. By default it's Normal.
    let mode = Rc::new(RefCell::new(Mode::Normal));

    // And we prepare the stuff for the default game (paths, and those things).
    let game_selected = Rc::new(RefCell::new(GameSelected::new(&settings.borrow())));
    match &*settings.borrow().default_game {
        "warhammer_2" => app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant()),
        "warhammer" => app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant()),
        "attila" => app_ui.menu_bar_change_game_selected.change_state(&"attila".to_variant()),
        "rome_2" => app_ui.menu_bar_change_game_selected.change_state(&"rome_2".to_variant()),
        _ => app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant()),
    }

    // Prepare the "MyMod" menu. This... atrocity needs to be in the following places for MyMod to open PackFiles:
    // - At the start of the program (here).
    // - At the end of MyMod creation.
    // - At the end of MyMod deletion.
    // - At the end of settings update.

    // First, we clear the list.
    app_ui.my_mod_list.remove_all();

    // If we have the "MyMod" path configured...
    if let Some(ref my_mod_base_path) = settings.borrow().paths.my_mods_base_path {

        // And can get without errors the folders in that path...
        if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
            for game_folder in game_folder_list {

                // If the file/folder is valid, we see if it's one of our game's folder.
                if let Ok(game_folder) = game_folder {
                    if game_folder.path().is_dir() &&
                        (
                            game_folder.file_name().to_string_lossy() == "warhammer_2"||
                            game_folder.file_name().to_string_lossy() == "warhammer" ||
                            game_folder.file_name().to_string_lossy() == "attila" ||
                            game_folder.file_name().to_string_lossy() == "rome_2"
                        ) {

                        // We create that game's menu here.
                        let game_submenu: Menu = Menu::new();
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();

                        // If there were no errors while reading the path...
                        if let Ok(game_folder_files) = game_folder.path().read_dir() {

                            // Index to count the valid packfiles.
                            let mut valid_mod_index = 0;

                            // We need to sort these files, so they appear sorted in the menu.
                            // FIXME: remove this unwrap.
                            let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|res| res.unwrap().path()).collect();
                            game_folder_files_sorted.sort();

                            // We get all the stuff in that game's folder...
                            for game_folder_file in game_folder_files_sorted {

                                // And it's a file that ends in .pack...
                                if game_folder_file.is_file() &&
                                    game_folder_file.extension().unwrap_or(OsStr::new("invalid")).to_string_lossy() =="pack" {

                                    // That means our game_folder is a valid folder and it needs to be added to the menu.
                                    let mod_name = game_folder_file.file_name().unwrap_or(OsStr::new("invalid")).to_string_lossy().as_ref().to_owned();
                                    let mod_action = &*format!("my-mod-open-{}-{}", match &*game_folder_name {
                                        "warhammer_2" => "warhammer_2",
                                        "warhammer" => "warhammer",
                                        "attila" => "attila",
                                        "rome_2" => "rome_2",
                                        _ => "if you see this, please report it",
                                    }, valid_mod_index);
                                    game_submenu.append(Some(&*mod_name), Some(&*format!("app.{}", mod_action)));

                                    // We create the action for the new button.
                                    let open_mod = SimpleAction::new(mod_action, None);
                                    application.add_action(&open_mod);

                                    // And when activating the mod button, we open it and set it as selected (chaos incoming).
                                    open_mod.connect_activate(clone!(
                                        app_ui,
                                        settings,
                                        schema,
                                        mode,
                                        game_folder_name,
                                        game_selected,
                                        pack_file_decoded => move |_,_| {
                                            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                                                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                    true
                                                } else {
                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                    false
                                                }
                                            } else { true };

                                            // If we got confirmation...
                                            if lets_do_it {
                                                let pack_file_path = game_folder_file.to_path_buf();
                                                match packfile::open_packfile(pack_file_path) {
                                                    Ok(pack_file_opened) => {
                                                        *pack_file_decoded.borrow_mut() = pack_file_opened;
                                                        ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                                                        set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        // Enable the selected mod.
                                                        *mode.borrow_mut() = Mode::MyMod {
                                                            game_folder_name: game_folder_name.to_owned(),
                                                            mod_name: mod_name.to_owned(),
                                                        };

                                                        // We choose the right option, depending on our PackFile.
                                                        match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                                            0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                                            1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                                            2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                                            3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                                            4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                                            _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                                                        }

                                                        // We deactive these menus, and only activate the one corresponding to our game.
                                                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                                                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                                                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                                                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                                                        // We choose the new GameSelected depending on what the open mod id is.
                                                        match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                                            "PFH5" => {
                                                                game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                                                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                                                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                                            },
                                                            "PFH4" | _ => {
                                                                game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                                                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                                                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                                            },
                                                        }

                                                        app_ui.menu_bar_save_packfile.set_enabled(true);
                                                        app_ui.menu_bar_save_packfile_as.set_enabled(true);
                                                        app_ui.menu_bar_change_packfile_type.set_enabled(true);

                                                        // Enable the controls for "MyMod".
                                                        app_ui.menu_bar_my_mod_delete.set_enabled(true);
                                                        app_ui.menu_bar_my_mod_install.set_enabled(true);
                                                        app_ui.menu_bar_my_mod_uninstall.set_enabled(true);

                                                        // Try to load the Schema for this PackFile's game.
                                                        *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                }
                                            }
                                    }));

                                    valid_mod_index += 1;
                                }
                            }
                        }

                        // Only if the submenu has items, we add it to the big menu.
                        if game_submenu.get_n_items() > 0 {
                            let game_submenu_name = match &*game_folder_name {
                                "warhammer_2" => "Warhammer 2",
                                "warhammer" => "Warhammer",
                                "attila" => "Attila",
                                "rome_2" => "Rome 2",
                                _ => "if you see this, please report it",
                            };
                            app_ui.my_mod_list.append_submenu(game_submenu_name, &game_submenu);
                        }
                    }
                }
            }
        }
    }

    // End of the "Getting Ready" part.
    // From here, it's all event handling.

    // First, we catch the close window event, and close the program when we do it.
    app_ui.window.connect_delete_event(clone!(
        application,
        pack_file_decoded,
        app_ui => move |_,_| {

        // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
        if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
            if app_ui.unsaved_dialog.run() == gtk_response_ok {
                application.quit();
            } else {
                app_ui.unsaved_dialog.hide_on_delete();
            }
        } else {
           application.quit();
        }

        Inhibit(true)
    }));

    //By default, these actions are disabled until a PackFile is created or opened.
    app_ui.menu_bar_save_packfile.set_enabled(false);
    app_ui.menu_bar_save_packfile_as.set_enabled(false);
    app_ui.menu_bar_change_packfile_type.set_enabled(false);

    // We deactive these menus, and only activate the one corresponding to our game.
    app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
    app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
    app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
    app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

    // These needs to be disabled by default at start too.
    app_ui.folder_tree_view_add_file.set_enabled(false);
    app_ui.folder_tree_view_add_folder.set_enabled(false);
    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
    app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
    app_ui.folder_tree_view_extract_packedfile.set_enabled(false);

    // And these three.
    app_ui.menu_bar_my_mod_delete.set_enabled(false);
    app_ui.menu_bar_my_mod_install.set_enabled(false);
    app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

    /*
    --------------------------------------------------------
                     Superior Menu: "File"
    --------------------------------------------------------
    */

    // When we hit the "New PackFile" button.
    app_ui.menu_bar_new_packfile.connect_activate(clone!(
        app_ui,
        schema,
        game_selected,
        mode,
        pack_file_decoded => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                    app_ui.unsaved_dialog.hide_on_delete();
                    true
                } else {
                    app_ui.unsaved_dialog.hide_on_delete();
                    false
                }
            } else { true };

            // If we got confirmation...
            if lets_do_it {

                // We deactive these menus, and only activate the one corresponding to our game.
                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                // We just create a new PackFile with a name, set his type to Mod and update the
                // TreeView to show it.
                let packfile_id = match &*game_selected.borrow().game {
                    "warhammer_2" => {
                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                        "PFH5"
                    },
                    "warhammer" | _ => {
                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                        "PFH4"
                    },
                };

                *pack_file_decoded.borrow_mut() = packfile::new_packfile("unknown.pack".to_string(), packfile_id);
                ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                // Disable selected mod, if we are using it.
                *mode.borrow_mut() = Mode::Normal;

                app_ui.menu_bar_save_packfile.set_enabled(true);
                app_ui.menu_bar_save_packfile_as.set_enabled(true);
                app_ui.menu_bar_change_packfile_type.set_enabled(true);

                // Disable the controls for "MyMod".
                app_ui.menu_bar_my_mod_delete.set_enabled(false);
                app_ui.menu_bar_my_mod_install.set_enabled(false);
                app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

                // Try to load the Schema for this PackFile's game.
                *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
            }
    }));


    // When we hit the "Open PackFile" button.
    app_ui.menu_bar_open_packfile.connect_activate(clone!(
        app_ui,
        game_selected,
        schema,
        settings,
        mode,
        pack_file_decoded => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                    app_ui.unsaved_dialog.hide_on_delete();
                    true
                } else {
                    app_ui.unsaved_dialog.hide_on_delete();
                    false
                }
            } else { true };

            // If we got confirmation...
            if lets_do_it {

                let file_chooser_open_packfile = FileChooserNative::new(
                    "Open PackFile...",
                    &app_ui.window,
                    FileChooserAction::Open,
                    "Accept",
                    "Cancel"
                );

                file_chooser_filter_packfile(&file_chooser_open_packfile, "*.pack");

                // In case we have a default path for the game selected, we use it as base path for opening files.
                if let Some(ref path) = game_selected.borrow().game_data_path {

                    // We check that actually exists before setting it.
                    if path.is_dir() {
                        file_chooser_open_packfile.set_current_folder(&path);
                    }
                }

                // When we select the file to open, we get his path, open it and, if there has been no
                // errors, decode it, update the TreeView to show it and check his type for the Change PackFile
                // Type option in the File menu.
                if file_chooser_open_packfile.run() == gtk_response_accept {
                    let pack_file_path = file_chooser_open_packfile.get_filename().expect("Couldn't open file");
                    match packfile::open_packfile(pack_file_path) {
                        Ok(pack_file_opened) => {
                            *pack_file_decoded.borrow_mut() = pack_file_opened;
                            ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                            set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                            // Disable selected mod, if we are using it.
                            *mode.borrow_mut() = Mode::Normal;

                            // We choose the right option, depending on our PackFile.
                            match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                            }

                            // We deactive these menus, and only activate the one corresponding to our game.
                            app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                            app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                            app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                            app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                            // We choose the new GameSelected depending on what the open mod id is.
                            match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                "PFH5" => {
                                    game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                    app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                    app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                    app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                },
                                "PFH4" | _ => {
                                    game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                    app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                    app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                    app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                },
                            }

                            // Enable the actions for PackFiles.
                            app_ui.menu_bar_save_packfile.set_enabled(true);
                            app_ui.menu_bar_save_packfile_as.set_enabled(true);
                            app_ui.menu_bar_change_packfile_type.set_enabled(true);

                            // Disable the controls for "MyMod".
                            app_ui.menu_bar_my_mod_delete.set_enabled(false);
                            app_ui.menu_bar_my_mod_install.set_enabled(false);
                            app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

                            // Try to load the Schema for this PackFile's game.
                            *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                        }
                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                    }
                }
            }
    }));


    // When we hit the "Save PackFile" button
    app_ui.menu_bar_save_packfile.connect_activate(clone!(
        app_ui,
        game_selected,
        pack_file_decoded => move |_,_| {

        // First, we check if our PackFile has a path. If it doesn't have it, we launch the Save
        // Dialog and set the current name in the entry of the dialog to his name.
        // When we hit "Accept", we get the selected path, encode the PackFile, and save it to that
        // path. After that, we update the TreeView to reflect the name change and hide the dialog.
        let mut pack_file_path: Option<PathBuf> = None;
        if !pack_file_decoded.borrow().pack_file_extra_data.file_path.exists() {
            let file_chooser_save_packfile = FileChooserNative::new(
                "Save PackFile...",
                &app_ui.window,
                FileChooserAction::Save,
                "Save",
                "Cancel"
            );

            file_chooser_save_packfile.set_current_name(&pack_file_decoded.borrow().pack_file_extra_data.file_name);

            // In case we have a default path for the game selected, we use it as base path for saving files.
            if let Some(ref path) = game_selected.borrow().game_data_path {

                // We check it actually exists before setting it.
                if path.is_dir() {
                    file_chooser_save_packfile.set_current_folder(path);
                }
            }

            if file_chooser_save_packfile.run() == gtk_response_accept {
                pack_file_path = Some(file_chooser_save_packfile.get_filename().expect("Couldn't open file"));

                let mut success = false;
                match packfile::save_packfile(&mut *pack_file_decoded.borrow_mut(), pack_file_path) {
                    Ok(result) => {
                        success = true;
                        ui::show_dialog(&app_ui.success_dialog, result);
                    },
                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                }
                if success {
                    // If saved, we reset the title to unmodified.
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    ui::update_tree_view_expand_path(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        &app_ui.folder_tree_view,
                        false
                    );
                }

            }
        }

        // If the PackFile has a path, we just encode it and save it into that path.
        else {
            let mut success = false;
            match packfile::save_packfile(&mut *pack_file_decoded.borrow_mut(), pack_file_path) {
                Ok(result) => {
                    success = true;
                    ui::show_dialog(&app_ui.success_dialog, result);
                },
                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
            }
            if success {
                // If saved, we reset the title to unmodified.
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
            }
        }
    }));


    // When we hit the "Save PackFile as" button.
    app_ui.menu_bar_save_packfile_as.connect_activate(clone!(
        game_selected,
        app_ui,
        mode,
        pack_file_decoded => move |_,_| {

        let file_chooser_save_packfile = FileChooserNative::new(
            "Save PackFile...",
            &app_ui.window,
            FileChooserAction::Save,
            "Save",
            "Cancel"
        );

        // If we are saving an existing PackFile with another name, we start in his current path.
        if pack_file_decoded.borrow().pack_file_extra_data.file_path.exists() {
            file_chooser_save_packfile.set_filename(&pack_file_decoded.borrow().pack_file_extra_data.file_path);
        }

        // In case we have a default path for the game selected, we use it as base path for saving files.
        else if let Some(ref path) = game_selected.borrow().game_data_path {
            file_chooser_save_packfile.set_current_name(&pack_file_decoded.borrow().pack_file_extra_data.file_name);

            // We check it actually exists before setting it.
            if path.is_dir() {
                file_chooser_save_packfile.set_current_folder(path);
            }
        }

        if file_chooser_save_packfile.run() == gtk_response_accept {
            let mut success = false;
            match packfile::save_packfile(
               &mut *pack_file_decoded.borrow_mut(),
               Some(file_chooser_save_packfile.get_filename().expect("Couldn't open file"))) {
                    Ok(result) => {
                        success = true;
                        ui::show_dialog(&app_ui.success_dialog, result);
                    },
                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
            }
            if success {
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                ui::update_tree_view_expand_path(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    &app_ui.folder_tree_view,
                    false
                );

                // If we save the mod as another, we are no longer using "MyMod".
                *mode.borrow_mut() = Mode::Normal;

                // Disable the controls for "MyMod".
                app_ui.menu_bar_my_mod_delete.set_enabled(false);
                app_ui.menu_bar_my_mod_install.set_enabled(false);
                app_ui.menu_bar_my_mod_uninstall.set_enabled(false);
            }
        }
    }));

    // When changing the type of the open PackFile.
    app_ui.menu_bar_change_packfile_type.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |menu_bar_change_packfile_type, selected_type| {
        if let Some(state) = selected_type.clone() {
            let new_state: Option<String> = state.get();
            match &*new_state.unwrap() {
                "boot" => {
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type != 0 {
                        pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 0;
                        menu_bar_change_packfile_type.change_state(&"boot".to_variant());
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    }
                }
                "release" => {
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type != 1 {
                        pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 1;
                        menu_bar_change_packfile_type.change_state(&"release".to_variant());
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    }
                }
                "patch" => {
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type != 2 {
                        pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 2;
                        menu_bar_change_packfile_type.change_state(&"patch".to_variant());
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    }
                }
                "mod" => {
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type != 3 {
                        pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 3;
                        menu_bar_change_packfile_type.change_state(&"mod".to_variant());
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    }
                }
                "movie" => {
                    if pack_file_decoded.borrow().pack_file_header.pack_file_type != 4 {
                        pack_file_decoded.borrow_mut().pack_file_header.pack_file_type = 4;
                        menu_bar_change_packfile_type.change_state(&"movie".to_variant());
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                    }
                }
                _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
            }
        }
    }));

    // When we hit the "Preferences" button.
    app_ui.menu_bar_preferences.connect_activate(clone!(
        app_ui,
        game_selected,
        supported_games,
        pack_file_decoded,
        settings,
        mode,
        application,
        schema => move |_,_| {

        // We disable the button, so we can't start 2 settings windows at the same time.
        app_ui.menu_bar_preferences.set_enabled(false);

        let settings_stuff = Rc::new(RefCell::new(ui::settings::SettingsWindow::create_settings_window(&application, &supported_games.borrow())));
        settings_stuff.borrow().load_to_settings_window(&*settings.borrow());

        // When we press the "Accept" button.
        settings_stuff.borrow().settings_accept.connect_button_release_event(clone!(
            pack_file_decoded,
            app_ui,
            settings_stuff,
            settings,
            game_selected,
            schema,
            mode,
            application => move |_,_| {
            let new_settings = settings_stuff.borrow().save_from_settings_window();
            *settings.borrow_mut() = new_settings;
            if let Err(error) = settings.borrow().save() {
                ui::show_dialog(&app_ui.error_dialog, error.cause());
            }
            settings_stuff.borrow().settings_window.destroy();
            app_ui.menu_bar_preferences.set_enabled(true);

            // If we change any setting, disable the selected mod. We have currently no proper way to check
            // if the "My mod" path has changed, so we disable the selected "My Mod" when changing any setting.
            *mode.borrow_mut() = Mode::Normal;

            // Reset the game selected, just in case we changed it's path.
            match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                "PFH5" => {
                    game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                    app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                }
                "PFH4" | _ => {
                    game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                    app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                }
            }

            // Disable the controls for "MyMod".
            app_ui.menu_bar_my_mod_delete.set_enabled(false);
            app_ui.menu_bar_my_mod_install.set_enabled(false);
            app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

            // Recreate the "MyMod" menu (Atrocity incoming).
            // First, we clear the list.
            app_ui.my_mod_list.remove_all();

            // If we have the "MyMod" path configured...
            if let Some(ref my_mod_base_path) = settings.borrow().paths.my_mods_base_path {

                // And can get without errors the folders in that path...
                if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

                    // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
                    for game_folder in game_folder_list {

                        // If the file/folder is valid, we see if it's one of our game's folder.
                        if let Ok(game_folder) = game_folder {
                            if game_folder.path().is_dir() &&
                                (
                                    game_folder.file_name().to_string_lossy() == "warhammer_2"||
                                    game_folder.file_name().to_string_lossy() == "warhammer" ||
                                    game_folder.file_name().to_string_lossy() == "attila" ||
                                    game_folder.file_name().to_string_lossy() == "rome_2"
                                ) {

                                // We create that game's menu here.
                                let game_submenu: Menu = Menu::new();
                                let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();

                                // If there were no errors while reading the path...
                                if let Ok(game_folder_files) = game_folder.path().read_dir() {

                                    // Index to count the valid packfiles.
                                    let mut valid_mod_index = 0;

                                    // We need to sort these files, so they appear sorted in the menu.
                                    // FIXME: remove this unwrap.
                                    let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|res| res.unwrap().path()).collect();
                                    game_folder_files_sorted.sort();

                                    // We get all the stuff in that game's folder...
                                    for game_folder_file in game_folder_files_sorted {

                                        // And it's a file that ends in .pack...
                                        if game_folder_file.is_file() &&
                                            game_folder_file.extension().unwrap_or(OsStr::new("invalid")).to_string_lossy() =="pack" {

                                            // That means our game_folder is a valid folder and it needs to be added to the menu.
                                            let mod_name = game_folder_file.file_name().unwrap_or(OsStr::new("invalid")).to_string_lossy().as_ref().to_owned();
                                            let mod_action = &*format!("my-mod-open-{}-{}", match &*game_folder_name {
                                                "warhammer_2" => "warhammer_2",
                                                "warhammer" => "warhammer",
                                                "attila" => "attila",
                                                "rome_2" => "rome_2",
                                                _ => "if you see this, please report it",
                                            }, valid_mod_index);
                                            game_submenu.append(Some(&*mod_name), Some(&*format!("app.{}", mod_action)));

                                            // We create the action for the new button.
                                            let open_mod = SimpleAction::new(mod_action, None);
                                            application.add_action(&open_mod);

                                            // And when activating the mod button, we open it and set it as selected (chaos incoming).
                                            open_mod.connect_activate(clone!(
                                                app_ui,
                                                settings,
                                                schema,
                                                mode,
                                                game_folder_name,
                                                game_selected,
                                                pack_file_decoded => move |_,_| {
                                                    // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                                    let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                                                        if app_ui.unsaved_dialog.run() == gtk_response_ok {
                                                            app_ui.unsaved_dialog.hide_on_delete();
                                                            true
                                                        } else {
                                                            app_ui.unsaved_dialog.hide_on_delete();
                                                            false
                                                        }
                                                    } else { true };

                                                    // If we got confirmation...
                                                    if lets_do_it {
                                                        let pack_file_path = game_folder_file.to_path_buf();
                                                        match packfile::open_packfile(pack_file_path) {
                                                            Ok(pack_file_opened) => {
                                                                *pack_file_decoded.borrow_mut() = pack_file_opened;
                                                                ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                                                                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                // Enable the selected mod.
                                                                *mode.borrow_mut() = Mode::MyMod {
                                                                    game_folder_name: game_folder_name.to_owned(),
                                                                    mod_name: mod_name.to_owned(),
                                                                };

                                                                // We choose the right option, depending on our PackFile.
                                                                match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                                                    0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                                                    1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                                                    2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                                                    3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                                                    4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                                                    _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                                                                }

                                                                // We deactive these menus, and only activate the one corresponding to our game.
                                                                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                                                                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                                                                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                                                                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                                                                // We choose the new GameSelected depending on what the open mod id is.
                                                                match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                                                    "PFH5" => {
                                                                        game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                                                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                                                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                                                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                                                    },
                                                                    "PFH4" | _ => {
                                                                        game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                                                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                                                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                                                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                                                    },
                                                                }

                                                                app_ui.menu_bar_save_packfile.set_enabled(true);
                                                                app_ui.menu_bar_save_packfile_as.set_enabled(true);
                                                                app_ui.menu_bar_change_packfile_type.set_enabled(true);

                                                                // Enable the controls for "MyMod".
                                                                app_ui.menu_bar_my_mod_delete.set_enabled(true);
                                                                app_ui.menu_bar_my_mod_install.set_enabled(true);
                                                                app_ui.menu_bar_my_mod_uninstall.set_enabled(true);

                                                                // Try to load the Schema for this PackFile's game.
                                                                *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                                                            }
                                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                        }
                                                    }
                                            }));

                                            valid_mod_index += 1;
                                        }
                                    }
                                }

                                // Only if the submenu has items, we add it to the big menu.
                                if game_submenu.get_n_items() > 0 {
                                    let game_submenu_name = match &*game_folder_name {
                                        "warhammer_2" => "Warhammer 2",
                                        "warhammer" => "Warhammer",
                                        "attila" => "Attila",
                                        "rome_2" => "Rome 2",
                                        _ => "if you see this, please report it",
                                    };
                                    app_ui.my_mod_list.append_submenu(game_submenu_name, &game_submenu);
                                }
                            }
                        }
                    }
                }
            }

            Inhibit(false)
        }));

        // When we press the "Cancel" button, we close the window.
        settings_stuff.borrow().settings_cancel.connect_button_release_event(clone!(
            settings_stuff,
            settings,
            app_ui => move |_,_| {
                settings_stuff.borrow().settings_window.destroy();
                app_ui.menu_bar_preferences.set_enabled(true);

                // Reload the Settings from the file so, if we have changed anything, it's undone.
                *settings.borrow_mut() = Settings::load().unwrap_or_else(|_|Settings::new());
                load_gtk_settings(&app_ui.window, &settings.borrow());

                Inhibit(false)
            }
        ));

        // We catch the destroy event to restore the "Preferences" button.
        settings_stuff.borrow().settings_window.connect_delete_event(clone!(
            settings,
            app_ui => move |settings_window, _| {
                settings_window.destroy();
                app_ui.menu_bar_preferences.set_enabled(true);

                // Reload the Settings from the file so, if we have changed anything, it's undone.
                *settings.borrow_mut() = Settings::load().unwrap_or_else(|_|Settings::new());
                load_gtk_settings(&app_ui.window, &settings.borrow());

                Inhibit(false)
            }
        ));
    }));

    // When we hit the "Quit" button.
    app_ui.menu_bar_quit.connect_activate(clone!(
        application,
        pack_file_decoded,
        app_ui => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                    application.quit();
                } else {
                    app_ui.unsaved_dialog.hide_on_delete();
                }
            } else {
                application.quit();
            }
    }));

    /*
    --------------------------------------------------------
                    Superior Menu: "My Mod"
    --------------------------------------------------------
    */

    // When we hit the "New mod" button.
    app_ui.menu_bar_my_mod_new.connect_activate(clone!(
        app_ui,
        settings,
        application,
        schema,
        game_selected,
        supported_games,
        mode,
        pack_file_decoded => move |_,_| {

        // We disable the button, so we can't open two new mod windows at the same time.
        app_ui.menu_bar_my_mod_new.set_enabled(false);

        // Create the the "New mod" window and put all it's stuff into a variable.
        let new_mod_stuff = Rc::new(RefCell::new(MyModNewWindow::create_my_mod_new_window(&application, &supported_games.borrow(), &game_selected.borrow(), &settings.borrow())));

        // When we press the "Accept" button.
        new_mod_stuff.borrow().my_mod_new_accept.connect_button_release_event(clone!(
            new_mod_stuff,
            application,
            app_ui,
            settings,
            schema,
            mode,
            game_selected,
            pack_file_decoded => move |_,_| {



                // Get the mod name.
                let mod_name = new_mod_stuff.borrow().my_mod_new_name_entry.get_buffer().get_text();

                // Get the PackFile name.
                let full_mod_name = format!("{}.pack", mod_name);

                // We deactive these menus, and only activate the one corresponding to our game.
                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                // We just create a new PackFile with a name, set his type to Mod and update the
                // TreeView to show it.
                let packfile_id = match &*new_mod_stuff.borrow().my_mod_new_game_list_combo.get_active_text().unwrap() {
                    "warhammer_2" => {
                        game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                        "PFH5"
                    },
                    "warhammer" | _ => {
                        game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                        "PFH4"
                    },
                };

                *pack_file_decoded.borrow_mut() = packfile::new_packfile(full_mod_name.to_owned(), packfile_id);
                ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                // Enable the disabled actions...
                app_ui.menu_bar_save_packfile.set_enabled(true);
                app_ui.menu_bar_save_packfile_as.set_enabled(true);
                app_ui.menu_bar_change_packfile_type.set_enabled(true);

                // Get his new path.
                let mut my_mod_path = settings.borrow().paths.my_mods_base_path.clone().unwrap();

                // We get his game's folder, depending on the selected game.
                let selected_game = new_mod_stuff.borrow().my_mod_new_game_list_combo.get_active_text().unwrap();
                let selected_game_folder = match &*selected_game {
                    "Warhammer 2" => "warhammer_2",
                    "Warhammer" => "warhammer",
                    "Attila" => "attila",
                    "Rome 2" => "rome_2",
                    _ => "if_you_see_this_folder_report_it",
                };
                my_mod_path.push(selected_game_folder.to_owned());

                // Just in case the folder doesn't exist, we try to create it. It's save to ignore this result.
                match DirBuilder::new().create(&my_mod_path){
                    Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                };

                // We need to create another folder inside game's folder with the name of the mod, to store extracted files.
                let mut extracted_files_path = my_mod_path.to_path_buf();
                extracted_files_path.push(mod_name.to_owned());
                match DirBuilder::new().create(&extracted_files_path) {
                    Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                };

                // Add the PackFile name to the full path.
                my_mod_path.push(full_mod_name.to_owned());

                // Then we save it.
                if let Err(error) = packfile::save_packfile(&mut pack_file_decoded.borrow_mut(), Some(my_mod_path)) {
                    ui::show_dialog(&app_ui.error_dialog, error.cause());
                }

                // If there was no error while saving, we destroy the window and reenable the "New mod" button.
                else {

                    // Mark it as "selected"
                    *mode.borrow_mut() = Mode::MyMod {
                        game_folder_name: selected_game_folder.to_owned(),
                        mod_name: full_mod_name,
                    };

                    // Enable the controls for "MyMod".
                    app_ui.menu_bar_my_mod_delete.set_enabled(true);
                    app_ui.menu_bar_my_mod_install.set_enabled(true);
                    app_ui.menu_bar_my_mod_uninstall.set_enabled(true);

                    // Recreate the "MyMod" menu (Atrocity incoming).
                    // First, we clear the list.
                    app_ui.my_mod_list.remove_all();

                    // If we have the "MyMod" path configured...
                    if let Some(ref my_mod_base_path) = settings.borrow().paths.my_mods_base_path {

                        // And can get without errors the folders in that path...
                        if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

                            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
                            for game_folder in game_folder_list {

                                // If the file/folder is valid, we see if it's one of our game's folder.
                                if let Ok(game_folder) = game_folder {
                                    if game_folder.path().is_dir() &&
                                        (
                                            game_folder.file_name().to_string_lossy() == "warhammer_2"||
                                            game_folder.file_name().to_string_lossy() == "warhammer" ||
                                            game_folder.file_name().to_string_lossy() == "attila" ||
                                            game_folder.file_name().to_string_lossy() == "rome_2"
                                        ) {

                                        // We create that game's menu here.
                                        let game_submenu: Menu = Menu::new();
                                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();

                                        // If there were no errors while reading the path...
                                        if let Ok(game_folder_files) = game_folder.path().read_dir() {

                                            // Index to count the valid packfiles.
                                            let mut valid_mod_index = 0;

                                            // We need to sort these files, so they appear sorted in the menu.
                                            // FIXME: remove this unwrap.
                                            let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|res| res.unwrap().path()).collect();
                                            game_folder_files_sorted.sort();

                                            // We get all the stuff in that game's folder...
                                            for game_folder_file in game_folder_files_sorted {

                                                // And it's a file that ends in .pack...
                                                if game_folder_file.is_file() &&
                                                    game_folder_file.extension().unwrap_or(OsStr::new("invalid")).to_string_lossy() =="pack" {

                                                    // That means our game_folder is a valid folder and it needs to be added to the menu.
                                                    let mod_name = game_folder_file.file_name().unwrap_or(OsStr::new("invalid")).to_string_lossy().as_ref().to_owned();
                                                    let mod_action = &*format!("my-mod-open-{}-{}", match &*game_folder_name {
                                                        "warhammer_2" => "warhammer_2",
                                                        "warhammer" => "warhammer",
                                                        "attila" => "attila",
                                                        "rome_2" => "rome_2",
                                                        _ => "if you see this, please report it",
                                                    }, valid_mod_index);
                                                    game_submenu.append(Some(&*mod_name), Some(&*format!("app.{}", mod_action)));

                                                    // We create the action for the new button.
                                                    let open_mod = SimpleAction::new(mod_action, None);
                                                    application.add_action(&open_mod);

                                                    // And when activating the mod button, we open it and set it as selected (chaos incoming).
                                                    open_mod.connect_activate(clone!(
                                                        app_ui,
                                                        settings,
                                                        schema,
                                                        mode,
                                                        game_folder_name,
                                                        game_selected,
                                                        pack_file_decoded => move |_,_| {

                                                            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                                            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                                                                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                                    true
                                                                } else {
                                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                                    false
                                                                }
                                                            } else { true };

                                                            // If we got confirmation...
                                                            if lets_do_it {
                                                                let pack_file_path = game_folder_file.to_path_buf();
                                                                match packfile::open_packfile(pack_file_path) {
                                                                    Ok(pack_file_opened) => {
                                                                        *pack_file_decoded.borrow_mut() = pack_file_opened;
                                                                        ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                                                                        set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                        // Enable the selected mod.
                                                                        *mode.borrow_mut() = Mode::MyMod {
                                                                            game_folder_name: game_folder_name.to_owned(),
                                                                            mod_name: mod_name.to_owned(),
                                                                        };

                                                                        // We choose the right option, depending on our PackFile.
                                                                        match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                                                            0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                                                            1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                                                            2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                                                            3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                                                            4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                                                            _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                                                                        }

                                                                        // We deactive these menus, and only activate the one corresponding to our game.
                                                                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                                                                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                                                                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                                                                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                                                                        // We choose the new GameSelected depending on what the open mod id is.
                                                                        match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                                                            "PFH5" => {
                                                                                game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                                                                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                                                                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                                                            },
                                                                            "PFH4" | _ => {
                                                                                game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                                                                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                                                                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                                                            },
                                                                        }

                                                                        app_ui.menu_bar_save_packfile.set_enabled(true);
                                                                        app_ui.menu_bar_save_packfile_as.set_enabled(true);
                                                                        app_ui.menu_bar_change_packfile_type.set_enabled(true);

                                                                        // Enable the controls for "MyMod".
                                                                        app_ui.menu_bar_my_mod_delete.set_enabled(true);
                                                                        app_ui.menu_bar_my_mod_install.set_enabled(true);
                                                                        app_ui.menu_bar_my_mod_uninstall.set_enabled(true);

                                                                        // Try to load the Schema for this PackFile's game.
                                                                        *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                                                                    }
                                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                                }
                                                            }
                                                    }));

                                                    valid_mod_index += 1;
                                                }
                                            }
                                        }

                                        // Only if the submenu has items, we add it to the big menu.
                                        if game_submenu.get_n_items() > 0 {
                                            let game_submenu_name = match &*game_folder_name {
                                                "warhammer_2" => "Warhammer 2",
                                                "warhammer" => "Warhammer",
                                                "attila" => "Attila",
                                                "rome_2" => "Rome 2",
                                                _ => "if you see this, please report it",
                                            };
                                            app_ui.my_mod_list.append_submenu(game_submenu_name, &game_submenu);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // And destroy the window.
                    new_mod_stuff.borrow().my_mod_new_window.destroy();
                    app_ui.menu_bar_my_mod_new.set_enabled(true);
                }
                Inhibit(false)
            }
        ));

        // When we press the "Cancel" button, we close the window and re-enable the "New mod" action.
        new_mod_stuff.borrow().my_mod_new_cancel.connect_button_release_event(clone!(
            new_mod_stuff,
            app_ui => move |_,_| {
            new_mod_stuff.borrow().my_mod_new_window.destroy();
            app_ui.menu_bar_my_mod_new.set_enabled(true);
            Inhibit(false)
        }));

        // We catch the destroy event to restore the "New mod" action.
        new_mod_stuff.borrow().my_mod_new_window.connect_delete_event(clone!(
            app_ui => move |my_mod_new_window, _| {
            my_mod_new_window.destroy();
            app_ui.menu_bar_my_mod_new.set_enabled(true);
            Inhibit(false)
        }));
    }));

    // When we hit the "Delete" button.
    app_ui.menu_bar_my_mod_delete.connect_activate(clone!(
        app_ui,
        application,
        settings,
        schema,
        game_selected,
        mode,
        pack_file_decoded => move |_,_| {

            // This will delete stuff from disk, so we need to be sure we want to do it.
            let lets_do_it = if app_ui.delete_my_mod_dialog.run() == gtk_response_ok {
                app_ui.delete_my_mod_dialog.hide_on_delete();
                true
            } else {
                app_ui.delete_my_mod_dialog.hide_on_delete();
                false
            };

            // If we got confirmation...
            if lets_do_it {

                // We can't change my_mod_selected while it's borrowed, so we need to set this to true
                // if we deleted the current "MyMod", and deal with changing it after ending the borrow.
                let my_mod_selected_deleted;
                let old_mod_name: String;

                // If we have a "MyMod" selected, and the "MyMod" path is configured...
                match *mode.borrow() {
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {
                        if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                            // We get his path.
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(game_folder_name.to_owned());
                            my_mod_path.push(mod_name.to_owned());

                            // We check that path exists.
                            if !my_mod_path.is_file() {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("Source PackFile doesn't exist."));
                            }

                            // And we delete it.
                            if let Err(error) = remove_file(&my_mod_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                            }

                            my_mod_selected_deleted = true;
                            old_mod_name = mod_name.to_owned();

                            // Now we try to delete his asset folder.
                            let mut asset_folder = mod_name.to_owned();
                            asset_folder.pop();
                            asset_folder.pop();
                            asset_folder.pop();
                            asset_folder.pop();
                            asset_folder.pop();
                            my_mod_path.pop();
                            my_mod_path.push(asset_folder);

                            // We check that path exists. This is optional, so it should allow the deletion
                            // process to continue with a warning.
                            if !my_mod_path.is_dir() {
                                ui::show_dialog(&app_ui.error_dialog, format_err!("Mod deleted, but his assets folder hasn't been found."));
                            }

                            // And we delete it if it passed the test before.
                            else if let Err(error) = remove_dir_all(&my_mod_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                            }

                        }
                        else {
                            return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base path not configured."));
                        }
                    }
                    Mode::Normal => return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod not selected.")),
                }

                // If we deleted it, we allow chaos to form below.
                if my_mod_selected_deleted {

                    // Set the selected mod to None.
                    *mode.borrow_mut() = Mode::Normal;

                    // Disable the controls for "MyMod".
                    app_ui.menu_bar_my_mod_delete.set_enabled(false);
                    app_ui.menu_bar_my_mod_install.set_enabled(false);
                    app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

                    // Replace the open PackFile with a dummy one, like during boot.
                    *pack_file_decoded.borrow_mut() = PackFile::new();

                    // Clear the TreeView.
                    app_ui.folder_tree_store.clear();

                    // First, we clear the list.
                    app_ui.my_mod_list.remove_all();

                    // If we have the "MyMod" path configured...
                    if let Some(ref my_mod_base_path) = settings.borrow().paths.my_mods_base_path {

                        // And can get without errors the folders in that path...
                        if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

                            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
                            for game_folder in game_folder_list {

                                // If the file/folder is valid, we see if it's one of our game's folder.
                                if let Ok(game_folder) = game_folder {
                                    if game_folder.path().is_dir() &&
                                        (
                                            game_folder.file_name().to_string_lossy() == "warhammer_2"||
                                            game_folder.file_name().to_string_lossy() == "warhammer" ||
                                            game_folder.file_name().to_string_lossy() == "attila" ||
                                            game_folder.file_name().to_string_lossy() == "rome_2"
                                        ) {

                                        // We create that game's menu here.
                                        let game_submenu: Menu = Menu::new();
                                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();

                                        // If there were no errors while reading the path...
                                        if let Ok(game_folder_files) = game_folder.path().read_dir() {

                                            // Index to count the valid packfiles.
                                            let mut valid_mod_index = 0;

                                            // We need to sort these files, so they appear sorted in the menu.
                                            // FIXME: remove this unwrap.
                                            let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|res| res.unwrap().path()).collect();
                                            game_folder_files_sorted.sort();

                                            // We get all the stuff in that game's folder...
                                            for game_folder_file in game_folder_files_sorted {

                                                // And it's a file that ends in .pack...
                                                if game_folder_file.is_file() &&
                                                    game_folder_file.extension().unwrap_or(OsStr::new("invalid")).to_string_lossy() =="pack" {

                                                    // That means our game_folder is a valid folder and it needs to be added to the menu.
                                                    let mod_name = game_folder_file.file_name().unwrap_or(OsStr::new("invalid")).to_string_lossy().as_ref().to_owned();
                                                    let mod_action = &*format!("my-mod-open-{}-{}", match &*game_folder_name {
                                                        "warhammer_2" => "warhammer_2",
                                                        "warhammer" => "warhammer",
                                                        "attila" => "attila",
                                                        "rome_2" => "rome_2",
                                                        _ => "if you see this, please report it",
                                                    }, valid_mod_index);
                                                    game_submenu.append(Some(&*mod_name), Some(&*format!("app.{}", mod_action)));

                                                    // We create the action for the new button.
                                                    let open_mod = SimpleAction::new(mod_action, None);
                                                    application.add_action(&open_mod);

                                                    // And when activating the mod button, we open it and set it as selected (chaos incoming).
                                                    open_mod.connect_activate(clone!(
                                                        app_ui,
                                                        schema,
                                                        settings,
                                                        mode,
                                                        game_folder_name,
                                                        game_selected,
                                                        pack_file_decoded => move |_,_| {
                                                            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                                            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                                                                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                                    true
                                                                } else {
                                                                    app_ui.unsaved_dialog.hide_on_delete();
                                                                    false
                                                                }
                                                            } else { true };

                                                            // If we got confirmation...
                                                            if lets_do_it {
                                                                let pack_file_path = game_folder_file.to_path_buf();
                                                                match packfile::open_packfile(pack_file_path) {
                                                                    Ok(pack_file_opened) => {
                                                                        *pack_file_decoded.borrow_mut() = pack_file_opened;
                                                                        ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                                                                        set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                        // Enable the selected mod.
                                                                        *mode.borrow_mut() = Mode::MyMod {
                                                                            game_folder_name: game_folder_name.to_owned(),
                                                                            mod_name: mod_name.to_owned(),
                                                                        };

                                                                        // We choose the right option, depending on our PackFile.
                                                                        match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                                                            0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                                                            1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                                                            2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                                                            3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                                                            4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                                                            _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                                                                        }

                                                                        // We deactive these menus, and only activate the one corresponding to our game.
                                                                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                                                                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                                                                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                                                                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                                                                        // We choose the new GameSelected depending on what the open mod id is.
                                                                        match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                                                            "PFH5" => {
                                                                                game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                                                                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                                                                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                                                            },
                                                                            "PFH4" | _ => {
                                                                                game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                                                                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                                                                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                                                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                                                            },
                                                                        }

                                                                        app_ui.menu_bar_save_packfile.set_enabled(true);
                                                                        app_ui.menu_bar_save_packfile_as.set_enabled(true);
                                                                        app_ui.menu_bar_change_packfile_type.set_enabled(true);

                                                                        // Enable the controls for "MyMod".
                                                                        app_ui.menu_bar_my_mod_delete.set_enabled(true);
                                                                        app_ui.menu_bar_my_mod_install.set_enabled(true);
                                                                        app_ui.menu_bar_my_mod_uninstall.set_enabled(true);

                                                                        // Try to load the Schema for this PackFile's game.
                                                                        *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                                                                    }
                                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                                }
                                                            }
                                                    }));

                                                    valid_mod_index += 1;
                                                }
                                            }
                                        }

                                        // Only if the submenu has items, we add it to the big menu.
                                        if game_submenu.get_n_items() > 0 {
                                            let game_submenu_name = match &*game_folder_name {
                                                "warhammer_2" => "Warhammer 2",
                                                "warhammer" => "Warhammer",
                                                "attila" => "Attila",
                                                "rome_2" => "Rome 2",
                                                _ => "if you see this, please report it",
                                            };
                                            app_ui.my_mod_list.append_submenu(game_submenu_name, &game_submenu);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ui::show_dialog(&app_ui.success_dialog, format!("MyMod \"{}\" deleted.", old_mod_name));
                }
            }
        }
    ));

    // When we hit the "Install" button.
    app_ui.menu_bar_my_mod_install.connect_activate(clone!(
        app_ui,
        mode,
        settings => move |_,_| {

            // Depending on our current "Mode", we choose what to do.
            match *mode.borrow() {
                Mode::MyMod {ref game_folder_name, ref mod_name} => {

                    if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                        // Get the game_path for the mod.
                        let game_path = match &**game_folder_name {
                            "warhammer_2" => settings.borrow().paths.warhammer_2.clone(),
                            "warhammer" => settings.borrow().paths.warhammer.clone(),
                            "attila" => settings.borrow().paths.attila.clone(),
                            "rome_2" => settings.borrow().paths.rome_2.clone(),
                            _ => Some(PathBuf::from("error")),
                        };

                        // If the game_path is configured.
                        if let Some(game_path) = game_path {

                            // We get his original path.
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(game_folder_name.to_owned());
                            my_mod_path.push(mod_name.to_owned());

                            // We check that path exists.
                            if !my_mod_path.is_file() {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("Source PackFile doesn't exist."));
                            }

                            // And his destination path.
                            let mut game_path = game_path.to_path_buf();
                            game_path.push("data");

                            // We check that path exists.
                            if !my_mod_path.is_dir() {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("Destination folder (../data) doesn't exist. You sure you configured the right folder for the game?"));
                            }

                            // And his destination file.
                            game_path.push(mod_name.to_owned());

                            // And copy it to the destination.
                            if let Err(error) = copy(my_mod_path, game_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                            }
                        }
                        else {
                            return ui::show_dialog(&app_ui.error_dialog, format_err!("Game folder path not configured."));
                        }
                    }
                    else {
                        ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base path not configured."));
                    }
                }

                // If we have no MyMod selected, return an error.
                Mode::Normal => ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod not selected.")),
            }
        }
    ));

    // When we hit the "Uninstall" button.
    app_ui.menu_bar_my_mod_uninstall.connect_activate(clone!(
        app_ui,
        mode,
        settings => move |_,_| {

            // Depending on our current "Mode", we choose what to do.
            match *mode.borrow() {
                Mode::MyMod {ref game_folder_name, ref mod_name} => {

                    // Get the game_path for the mod.
                    let game_path = match &**game_folder_name {
                        "warhammer_2" => settings.borrow().paths.warhammer_2.clone(),
                        "warhammer" => settings.borrow().paths.warhammer.clone(),
                        "attila" => settings.borrow().paths.attila.clone(),
                        "rome_2" => settings.borrow().paths.rome_2.clone(),
                        _ => Some(PathBuf::from("error")),
                    };

                    // If the game_path is configured.
                    if let Some(game_path) = game_path {

                        // And his destination path.
                        let mut installed_mod_path = game_path.to_path_buf();
                        installed_mod_path.push("data");
                        installed_mod_path.push(mod_name.to_owned());

                        // We check that path exists.
                        if !installed_mod_path.is_file() {
                            return ui::show_dialog(&app_ui.error_dialog, format_err!("The currently selected mod is not installed"));
                        }
                        else {
                            // And remove the mod from the data folder of the game.
                            if let Err(error) = remove_file(installed_mod_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                            }
                        }
                    }
                    else {
                        ui::show_dialog(&app_ui.error_dialog, format_err!("Game folder path not configured."));
                    }
                }
                Mode::Normal => ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod not selected.")),
            }
        }
    ));


    /*
    --------------------------------------------------------
                 Superior Menu: "Game Selected"
    --------------------------------------------------------
    */

    // When changing the selected game.
    app_ui.menu_bar_change_game_selected.connect_activate(clone!(
        settings,
        game_selected => move |menu_bar_change_game_selected, selected| {
        if let Some(state) = selected.clone() {
            let new_state: Option<String> = state.get();
            match &*new_state.unwrap() {
                "warhammer_2" => {
                    game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                    menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                }
                "warhammer" | _ => {
                    game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer);
                    menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                }
                /*
                "attila" => {
                    game_selected.borrow_mut().change_game_selected(&settings.borrow().paths.attila);
                    menu_bar_change_game_selected.change_state(&"attila".to_variant());
                }
                "rome_2" => {
                    game_selected.borrow_mut().change_game_selected(&settings.borrow().paths.rome_2);
                    menu_bar_change_game_selected.change_state(&"rome_2".to_variant());
                }*/
            }
        }
    }));
    /*
    --------------------------------------------------------
                 Superior Menu: "Special Stuff"
    --------------------------------------------------------
    */

    // When we hit the "Patch SiegeAI" button.
    app_ui.menu_bar_patch_siege_ai_wh2.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_| {

        // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
        // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
        let mut sucessful_patching = (false, String::new());
        match packfile::patch_siege_ai(&mut *pack_file_decoded.borrow_mut()) {
            Ok(result) => sucessful_patching = (true, result),
            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
        }
        if sucessful_patching.0 {
            let mut success = false;
            match packfile::save_packfile( &mut *pack_file_decoded.borrow_mut(), None) {
                Ok(result) => {
                    success = true;
                    ui::show_dialog(&app_ui.success_dialog, format!("{}\n\n{}", sucessful_patching.1, result));
                },
                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
            }
            if success {
                ui::update_tree_view_expand_path(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    &app_ui.folder_tree_view,
                    false
                );
            }
        }
    }));

    // When we hit the "Generate Dependency Pack" button.
    app_ui.menu_bar_generate_dependency_pack_wh2.connect_activate(clone!(
        app_ui,
        game_selected => move |_,_| {

            // Get the data folder of game_selected and try to create our dependency PackFile.
            match game_selected.borrow().game_data_path {
                Some(ref path) => {
                    let mut data_pack_path = path.to_path_buf();
                    data_pack_path.push("data.pack");
                    match packfile::open_packfile(data_pack_path) {
                        Ok(ref mut data_packfile) => {
                            data_packfile.pack_file_data.packed_files.retain(|packed_file| packed_file.packed_file_path.starts_with(&["db".to_owned()]));
                            data_packfile.pack_file_header.packed_file_count = data_packfile.pack_file_data.packed_files.len() as u32;

                            // Just in case the folder doesn't exists, we try to create it.
                            match DirBuilder::new().create(PathBuf::from("dependency_packs")) {
                                Ok(_) | Err(_) => {},
                            }

                            let pack_file_path = match &*game_selected.borrow().game {
                                "warhammer_2" => PathBuf::from("dependency_packs/wh2.pack"),
                                "warhammer" | _ => PathBuf::from("dependency_packs/wh.pack")
                            };

                            match packfile::save_packfile(data_packfile, Some(pack_file_path)) {
                                Ok(_) => ui::show_dialog(&app_ui.success_dialog, format_err!("Dependency pack created.")),
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: generated dependency pack couldn't be saved. {:?}", error)),
                            }
                        }
                        Err(_) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: data.pack couldn't be open."))
                    }
                },
                None => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: data path of the game not found."))
            }
        }
    ));

    // When we hit the "Patch SiegeAI" button (Warhammer).
    app_ui.menu_bar_patch_siege_ai_wh.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_| {

        // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
        // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
        let mut sucessful_patching = (false, String::new());
        match packfile::patch_siege_ai(&mut *pack_file_decoded.borrow_mut()) {
            Ok(result) => sucessful_patching = (true, result),
            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
        }
        if sucessful_patching.0 {
            let mut success = false;
            match packfile::save_packfile( &mut *pack_file_decoded.borrow_mut(), None) {
                Ok(result) => {
                    success = true;
                    ui::show_dialog(&app_ui.success_dialog, format!("{}\n\n{}", sucessful_patching.1, result));
                },
                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
            }
            if success {
                ui::update_tree_view_expand_path(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    &app_ui.folder_tree_view,
                    false
                );
            }
        }
    }));

    // When we hit the "Generate Dependency Pack" button (Warhammer).
    app_ui.menu_bar_generate_dependency_pack_wh.connect_activate(clone!(
        game_selected,
        app_ui => move |_,_| {

            // Get the data folder of game_selected and try to create our dependency PackFile.
            match game_selected.borrow().game_data_path {
                Some(ref path) => {
                    let mut data_pack_path = path.to_path_buf();
                    data_pack_path.push("data.pack");
                    match packfile::open_packfile(data_pack_path) {
                        Ok(ref mut data_packfile) => {
                            data_packfile.pack_file_data.packed_files.retain(|packed_file| packed_file.packed_file_path.starts_with(&["db".to_owned()]));
                            data_packfile.pack_file_header.packed_file_count = data_packfile.pack_file_data.packed_files.len() as u32;

                            // Just in case the folder doesn't exists, we try to create it.
                            match DirBuilder::new().create(PathBuf::from("dependency_packs")) {
                                Ok(_) | Err(_) => {},
                            }

                            let pack_file_path = match &*game_selected.borrow().game {
                                "warhammer_2" => PathBuf::from("dependency_packs/wh2.pack"),
                                "warhammer" | _ => PathBuf::from("dependency_packs/wh.pack")
                            };

                            match packfile::save_packfile(data_packfile, Some(pack_file_path)) {
                                Ok(_) => ui::show_dialog(&app_ui.success_dialog, format_err!("Dependency pack created.")),
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: generated dependency pack couldn't be saved. {:?}", error)),
                            }
                        }
                        Err(_) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: data.pack couldn't be open."))
                    }
                },
                None => ui::show_dialog(&app_ui.error_dialog, format_err!("Error: data path of the game not found."))
            }
        }
    ));

    /*
    --------------------------------------------------------
                    Superior Menu: "About"
    --------------------------------------------------------
    */

    // When we hit the "Check Updates" button.
    app_ui.menu_bar_check_updates.connect_activate(clone!(
        app_ui => move |_,_| {
        check_updates(Some(&app_ui.check_updates_dialog), &app_ui.status_bar)
    }));

    // When we hit the "About" button.
    app_ui.menu_bar_about.connect_activate(clone!(
        app_ui => move |_,_| {
        app_ui.about_window.run();
        app_ui.about_window.hide_on_delete();
    }));


    /*
    --------------------------------------------------------
                   Contextual TreeView Popup
    --------------------------------------------------------
    */

    // When we right-click the TreeView, we calculate the position where the popup must aim, and show it.
    //
    // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSE IT WHEN WE HIT HIS BUTTONS.
    app_ui.folder_tree_view.connect_button_release_event(clone!(
        app_ui => move |_,button| {

        if button.get_button() == 3 && app_ui.folder_tree_selection.count_selected_rows() > 0 {
            let rect = ui::get_rect_for_popover(&app_ui.folder_tree_view, Some(button.get_position()));

            app_ui.folder_tree_view_context_menu.set_pointing_to(&rect);
            app_ui.folder_tree_view_context_menu.popup();
        }
        Inhibit(false)
    }));

    // We check every action possible for the selected file when changing the cursor.
    app_ui.folder_tree_view.connect_cursor_changed(clone!(
        pack_file_decoded,
        app_ui => move |_| {

        let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
        for packed_file in &*pack_file_decoded.borrow().pack_file_data.packed_files {

            // If the selection is a file.
            if packed_file.packed_file_path == tree_path {
                app_ui.folder_tree_view_add_file.set_enabled(false);
                app_ui.folder_tree_view_add_folder.set_enabled(false);
                app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
                app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                break;
            }
        }

        // If it's the PackFile.
        if tree_path.is_empty() {
            app_ui.folder_tree_view_add_file.set_enabled(true);
            app_ui.folder_tree_view_add_folder.set_enabled(true);
            app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
            app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
            app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
        }

        // If this is triggered, the selection is a folder.
        else {
            app_ui.folder_tree_view_add_file.set_enabled(true);
            app_ui.folder_tree_view_add_folder.set_enabled(true);
            app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
            app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
            app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
        }
    }));

    // When we hit the "Add file" button.
    app_ui.folder_tree_view_add_file.connect_activate(clone!(
        app_ui,
        settings,
        mode,
        pack_file_decoded => move |_,_| {

        // First, we hide the context menu, then we pick the file selected and add it to the Packfile.
        // After that, we update the TreeView.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {

            let file_chooser_add_file_to_packfile = FileChooserNative::new(
                "Select File...",
                &app_ui.window,
                FileChooserAction::Open,
                "Accept",
                "Cancel"
            );

            match *mode.borrow() {

                // If there is a "MyMod" selected, we need to add whatever we want to add
                // directly to the mod's assets folder.
                Mode::MyMod {ref game_folder_name, ref mod_name} => {
                    // In theory, if we reach this line this should always exist. In theory I should be rich.
                    if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                        // We get his original path.
                        let mut my_mod_path = my_mods_base_path.to_path_buf();
                        my_mod_path.push(game_folder_name.to_owned());

                        // We need his folder, not his PackFile name.
                        let mut folder_name = mod_name.to_owned();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        my_mod_path.push(folder_name);

                        // We check that path exists, and create it if it doesn't.
                        if !my_mod_path.is_dir() {
                            match DirBuilder::new().create(&my_mod_path) {
                                Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                            };
                        }

                        // Then we set that path as current path for the "Add PackedFile" file chooser.
                        file_chooser_add_file_to_packfile.set_current_folder(&my_mod_path);

                        // And run the file_chooser.
                        if file_chooser_add_file_to_packfile.run() == gtk_response_accept {

                            // Get the names of the files to add.
                            let paths = file_chooser_add_file_to_packfile.get_filenames();

                            // For each one of them...
                            for path in &paths {

                                // If we are inside the mod's folder, we need to "emulate" the path to then
                                // file in the TreeView, so we add the file with a custom tree_path.
                                if path.starts_with(&my_mod_path) {

                                    // Remove from their path the base mod path (leaving only their future tree_path).
                                    let mut index = 0;
                                    let mut path_vec = path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                    let mut my_mod_path_vec = my_mod_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                    loop {
                                        if index < path_vec.len() && index < my_mod_path_vec.len() &&
                                            path_vec[index] != my_mod_path_vec[index] {
                                            break;
                                        }
                                        else if index == path_vec.len() || index == my_mod_path_vec.len() {
                                            break;
                                        }
                                        index += 1;
                                    }

                                    let tree_path = path_vec[index..].to_vec();

                                    let mut success = false;
                                    match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path) {
                                        Ok(_) => success = true,
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                    }
                                    if success {
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        ui::update_tree_view_expand_path(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            &app_ui.folder_tree_view,
                                            false
                                        );
                                    }
                                }

                                // If not, we get their tree_path like a normal file.
                                else {

                                    // Get his usual tree_path.
                                    let tree_path = ui::get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);

                                    let mut success = false;
                                    match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path) {
                                        Ok(_) => success = true,
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                    }
                                    if success {
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        ui::update_tree_view_expand_path(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            &app_ui.folder_tree_view,
                                            false
                                        );
                                    }
                                }
                            }
                        }
                    }
                    else {
                        return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base folder not configured."));
                    }
                },

                // If there is no "MyMod" selected, we just keep the normal behavior.
                Mode::Normal => {
                    if file_chooser_add_file_to_packfile.run() == gtk_response_accept {

                        let paths = file_chooser_add_file_to_packfile.get_filenames();
                        for path in &paths {

                            let tree_path = ui::get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);
                            let mut success = false;
                            match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path) {
                                Ok(_) => success = true,
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                            }
                            if success {
                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                ui::update_tree_view_expand_path(
                                    &app_ui.folder_tree_store,
                                    &*pack_file_decoded.borrow(),
                                    &app_ui.folder_tree_selection,
                                    &app_ui.folder_tree_view,
                                    false
                                );
                            }
                        }
                    }
                },
            }
        }
    }));


    // When we hit the "Add folder" button.
    app_ui.folder_tree_view_add_folder.connect_activate(clone!(
        app_ui,
        settings,
        mode,
        pack_file_decoded => move |_,_| {

        // First, we hide the context menu. Then we get the folder selected and we get all the files
        // in him and his subfolders. After that, for every one of those files, we strip his path,
        // leaving then with only the part that will be added to the PackedFile and we add it to the
        // PackFile. After all that, if we added any of the files to the PackFile, we update the
        // TreeView.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {

            let file_chooser_add_folder_to_packfile = FileChooserNative::new(
                "Select Folder...",
                &app_ui.window,
                FileChooserAction::SelectFolder,
                "Accept",
                "Cancel"
            );

            match *mode.borrow() {

                // If there is a "MyMod" selected, we need to add whatever we want to add
                // directly to the mod's assets folder.
                Mode::MyMod {ref game_folder_name, ref mod_name} => {
                    // In theory, if we reach this line this should always exist. In theory I should be rich.
                    if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                        // We get his original path.
                        let mut my_mod_path = my_mods_base_path.to_path_buf();
                        my_mod_path.push(game_folder_name.to_owned());

                        // We need his folder, not his PackFile name.
                        let mut folder_name = mod_name.to_owned();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        folder_name.pop();
                        my_mod_path.push(folder_name);

                        // We check that path exists, and create it if it doesn't.
                        if !my_mod_path.is_dir() {
                            match DirBuilder::new().create(&my_mod_path) {
                                Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                            };
                        }

                        // Then we set that path as current path for the "Add PackedFile" file chooser.
                        file_chooser_add_folder_to_packfile.set_current_folder(&my_mod_path);

                        // Run the file chooser.
                        if file_chooser_add_folder_to_packfile.run() == gtk_response_accept {

                            // Get the folders.
                            let folders = file_chooser_add_folder_to_packfile.get_filenames();

                            // For each folder...
                            for folder in &folders {

                                // If we are inside the mod's folder, we need to "emulate" the path to then
                                // file in the TreeView, so we add the file with a custom tree_path.
                                if folder.starts_with(&my_mod_path) {

                                    // Remove from their path the base mod path (leaving only their future tree_path).
                                    let mut index = 0;
                                    let mut path_vec = folder.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                    let mut my_mod_path_vec = my_mod_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                    loop {
                                        if index < path_vec.len() && index < my_mod_path_vec.len() &&
                                            path_vec[index] != my_mod_path_vec[index] {
                                            break;
                                        }
                                        else if index == path_vec.len() || index == my_mod_path_vec.len() {
                                            break;
                                        }
                                        index += 1;
                                    }

                                    let tree_path = path_vec[index..].to_vec();

                                    // Get the path of the folder without the "final" folder we want to add.
                                    let mut big_parent_prefix = folder.clone();
                                    big_parent_prefix.pop();

                                    // Get all the files from that folder.
                                    match ::common::get_files_from_subdir(folder) {
                                        Ok(file_path_list) => {
                                            let mut file_errors = 0;

                                            // For each file in that folder...
                                            for file in file_path_list {

                                                // Leave them only with the path from the folder we want to add to the end.
                                                match file.strip_prefix(&big_parent_prefix) {
                                                    Ok(filtered_path) => {

                                                        // Then get their unique tree_path, combining our current tree_path
                                                        // with the filtered_path we got for them.
                                                        let mut filtered_path = filtered_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                                        let mut tree_path = tree_path.clone();
                                                        tree_path.pop();
                                                        tree_path.append(&mut filtered_path);

                                                        if packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &file.to_path_buf(), tree_path).is_err() {
                                                            file_errors += 1;
                                                        }
                                                    }
                                                    Err(_) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error adding file/s to the PackFile")),
                                                }
                                            }
                                            if file_errors > 0 {
                                                ui::show_dialog(&app_ui.error_dialog, format!("{} file/s that you wanted to add already exist in the Packfile.", file_errors));
                                            }
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                            ui::update_tree_view_expand_path(
                                                &app_ui.folder_tree_store,
                                                &*pack_file_decoded.borrow(),
                                                &app_ui.folder_tree_selection,
                                                &app_ui.folder_tree_view,
                                                false
                                            );
                                        }
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                    }
                                }

                                // If not, we get their tree_path like a normal folder.
                                else {

                                    // Get the path of the folder without the "final" folder we want to add.
                                    let mut big_parent_prefix = folder.clone();
                                    big_parent_prefix.pop();

                                    // Get all the files from that folder.
                                    match ::common::get_files_from_subdir(folder) {
                                        Ok(file_path_list) => {
                                            let mut file_errors = 0;

                                            // For each file in that folder...
                                            for i in file_path_list {

                                                // Leave them only with the path from the folder we want to add to the end.
                                                match i.strip_prefix(&big_parent_prefix) {
                                                    Ok(filtered_path) => {
                                                        let tree_path = ui::get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &app_ui.folder_tree_selection, false);
                                                        if packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &i.to_path_buf(), tree_path).is_err() {
                                                            file_errors += 1;
                                                        }
                                                    }
                                                    Err(_) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error adding file/s to the PackFile")),
                                                }
                                            }
                                            if file_errors > 0 {
                                                ui::show_dialog(&app_ui.error_dialog, format!("{} file/s that you wanted to add already exist in the Packfile.", file_errors));
                                            }
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                            ui::update_tree_view_expand_path(
                                                &app_ui.folder_tree_store,
                                                &*pack_file_decoded.borrow(),
                                                &app_ui.folder_tree_selection,
                                                &app_ui.folder_tree_view,
                                                false
                                            );
                                        }
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                    }
                                }
                            }
                        }
                    }
                    else {
                        return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base folder not configured."));
                    }
                }

                // If there is no "MyMod" selected, we just keep the normal behavior.
                Mode::Normal => {
                    if file_chooser_add_folder_to_packfile.run() == gtk_response_accept {
                        let folders = file_chooser_add_folder_to_packfile.get_filenames();
                        for folder in &folders {
                            let mut big_parent_prefix = folder.clone();
                            big_parent_prefix.pop();
                            match ::common::get_files_from_subdir(folder) {
                                Ok(file_path_list) => {
                                    let mut file_errors = 0;
                                    for i in file_path_list {
                                        match i.strip_prefix(&big_parent_prefix) {
                                            Ok(filtered_path) => {
                                                let tree_path = ui::get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &app_ui.folder_tree_selection, false);
                                                if packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &i.to_path_buf(), tree_path).is_err() {
                                                    file_errors += 1;
                                                }
                                            }
                                            Err(_) => ui::show_dialog(&app_ui.error_dialog, format_err!("Error adding file/s to the PackFile")),
                                        }
                                    }
                                    if file_errors > 0 {
                                        ui::show_dialog(&app_ui.error_dialog, format!("{} file/s that you wanted to add already exist in the Packfile.", file_errors));
                                    }
                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                    ui::update_tree_view_expand_path(
                                        &app_ui.folder_tree_store,
                                        &*pack_file_decoded.borrow(),
                                        &app_ui.folder_tree_selection,
                                        &app_ui.folder_tree_view,
                                        false
                                    );
                                }
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                            }
                        }
                    }
                }
            }
        }
    }));

    // When we hit the "Add file/folder from PackFile" button.
    app_ui.folder_tree_view_add_from_packfile.connect_activate(clone!(
        app_ui,
        pack_file_decoded,
        pack_file_decoded_extra,
        is_folder_tree_view_locked => move |_,_| {

        // First, we hide the context menu, then we pick the PackFile selected.
        // After that, we update the TreeView.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {

            // Then, we destroy any children that the packed_file_data_display we use may have, cleaning it.
            let childrens_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
            if !childrens_to_utterly_destroy.is_empty() {
                for i in &childrens_to_utterly_destroy {
                    i.destroy();
                }
            }

            let file_chooser_add_from_packfile = FileChooserNative::new(
                "Select PackFile...",
                &app_ui.window,
                FileChooserAction::Open,
                "Accept",
                "Cancel"
            );

            // Set his filter to only admit ".pack" files.
            file_chooser_filter_packfile(&file_chooser_add_from_packfile, "*.pack");

            if file_chooser_add_from_packfile.run() == gtk_response_accept {
                let pack_file_path = file_chooser_add_from_packfile.get_filename().expect("Couldn't open file");
                match packfile::open_packfile(pack_file_path) {

                    // If the extra PackFile is valid, we create a box with a button to exit this mode
                    // and a TreeView of the PackFile data.
                    Ok(pack_file_opened) => {

                        // We put a "Save" button in the top part, and left the lower part for an horizontal
                        // Box with the "Copy" button and the TreeView.
                        let folder_tree_view_extra_exit_button = Button::new_with_label("Exit \"Add file/folder from PackFile\" mode");
                        folder_tree_view_extra_exit_button.set_vexpand(false);
                        app_ui.packed_file_data_display.attach(&folder_tree_view_extra_exit_button, 0, 0, 2, 1);

                        // First, we create the "Copy" Button.
                        let folder_tree_view_extra_copy_button = Button::new_with_label("<=");
                        folder_tree_view_extra_exit_button.set_hexpand(false);
                        app_ui.packed_file_data_display.attach(&folder_tree_view_extra_copy_button, 0, 1, 1, 1);

                        // Second, we create the new TreeView (in a ScrolledWindow) and his TreeStore.
                        let folder_tree_view_extra = TreeView::new();
                        let folder_tree_store_extra = TreeStore::new(&[String::static_type()]);
                        folder_tree_view_extra.set_model(Some(&folder_tree_store_extra));

                        let column_extra = TreeViewColumn::new();
                        let cell_extra = CellRendererText::new();
                        column_extra.pack_start(&cell_extra, true);
                        column_extra.add_attribute(&cell_extra, "text", 0);

                        folder_tree_view_extra.append_column(&column_extra);
                        folder_tree_view_extra.set_enable_tree_lines(true);
                        folder_tree_view_extra.set_enable_search(false);
                        folder_tree_view_extra.set_headers_visible(false);

                        let folder_tree_view_extra_scroll = ScrolledWindow::new(None, None);
                        folder_tree_view_extra_scroll.set_hexpand(true);
                        folder_tree_view_extra_scroll.set_vexpand(true);
                        folder_tree_view_extra_scroll.add(&folder_tree_view_extra);
                        app_ui.packed_file_data_display.attach(&folder_tree_view_extra_scroll, 1, 1, 1, 1);

                        // And show everything and lock the main PackFile's TreeView.
                        app_ui.packed_file_data_display.show_all();
                        *is_folder_tree_view_locked.borrow_mut() = true;

                        *pack_file_decoded_extra.borrow_mut() = pack_file_opened;
                        ui::update_tree_view(&folder_tree_store_extra, &*pack_file_decoded_extra.borrow());

                        // We need to check here if the selected destiny is not a file. Otherwise
                        // we disable the "Copy" button.
                        app_ui.folder_tree_selection.connect_changed(clone!(
                        folder_tree_view_extra_copy_button,
                        pack_file_decoded => move |folder_tree_selection| {
                            let tree_path = ui::get_tree_path_from_selection(folder_tree_selection, true);

                            // Only in case it's not a file, we enable the "Copy" Button.
                            match get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {
                                TreePathType::File(_) => folder_tree_view_extra_copy_button.set_sensitive(false),
                                TreePathType::Folder(_) | TreePathType::PackFile | TreePathType::None => folder_tree_view_extra_copy_button.set_sensitive(true),
                            }
                        }));

                        // When we click in the "Copy" button (<=).
                        folder_tree_view_extra_copy_button.connect_button_release_event(clone!(
                            app_ui,
                            pack_file_decoded,
                            pack_file_decoded_extra,
                            folder_tree_view_extra => move |_,_| {

                            let tree_path_source = ui::get_tree_path_from_selection(&folder_tree_view_extra.get_selection(), true);
                            let tree_path_destination = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
                            let mut packed_file_added = false;
                            match packfile::add_packedfile_to_packfile(
                                &*pack_file_decoded_extra.borrow(),
                                &mut *pack_file_decoded.borrow_mut(),
                                &tree_path_source,
                                &tree_path_destination,
                            ) {
                                Ok(_) => packed_file_added = true,
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                            }
                            if packed_file_added {
                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                ui::update_tree_view_expand_path(
                                    &app_ui.folder_tree_store,
                                    &*pack_file_decoded.borrow(),
                                    &app_ui.folder_tree_selection,
                                    &app_ui.folder_tree_view,
                                    false
                                );
                            }

                            Inhibit(false)
                        }));

                        // When we click in the "Exit "Add file/folder from PackFile" mode" button.
                        folder_tree_view_extra_exit_button.connect_button_release_event(clone!(
                            app_ui,
                            is_folder_tree_view_locked => move |_,_| {
                            *is_folder_tree_view_locked.borrow_mut() = false;

                            // We need to destroy any children that the packed_file_data_display we use may have, cleaning it.
                            let children_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
                            if !children_to_utterly_destroy.is_empty() {
                                for i in &children_to_utterly_destroy {
                                    i.destroy();
                                }
                            }
                            ui::display_help_tips(&app_ui.packed_file_data_display);

                            Inhibit(false)
                        }));

                    }
                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                }
            }
        }
    }));

    // When we hit the "Delete file/folder" button.
    app_ui.folder_tree_view_delete_packedfile.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_|{

        // We hide the context menu, then we get the selected file/folder, delete it and update the
        // TreeView. Pretty simple, actually.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {

            let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
            let mut success = false;
            match packfile::delete_from_packfile(&mut *pack_file_decoded.borrow_mut(), &tree_path) {
                Ok(_) => success = true,
                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
            }
            if success {
                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                ui::update_tree_view_expand_path(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    &app_ui.folder_tree_view,
                    true
                );
            }
        }
    }));


    // When we hit the "Extract file/folder" button.
    app_ui.folder_tree_view_extract_packedfile.connect_activate(clone!(
        app_ui,
        settings,
        mode,
        pack_file_decoded => move |_,_|{

        // First, we hide the context menu.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {
            let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

            // Then, we check with the correlation data if the tree_path is a folder or a file.
            // Both (folder and file) are processed in the same way but we need a different
            // FileChooser for files and folders, so we check first what it's.
            match get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {
                TreePathType::File(_) => {
                    match *mode.borrow() {

                        // If there is a "MyMod" selected, we need to extract whatever we want to extracted
                        // directly to the mod's assets folder.
                        Mode::MyMod {ref game_folder_name, mod_name: _} => {
                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                                // We get his base path (where the PackFile is).
                                let mut my_mod_base_folder = my_mods_base_path.to_path_buf();
                                my_mod_base_folder.push(game_folder_name.to_owned());

                                // Now we create the folder structure of the parents of that PackedFile in the
                                // assets folder, so we have a full structure replicating the PackFile when we
                                // extract stuff from the PackFile.
                                let mut extraction_final_folder = my_mod_base_folder;
                                let mut tree_path = tree_path.to_vec();
                                let tree_path_len = tree_path.len();

                                for (index, folder) in tree_path.iter_mut().enumerate() {

                                    // The PackFile ".pack" extension NEEDS to be removed.
                                    if index == 0 && folder.ends_with(".pack"){

                                        // How to remove the last five characters of a string, lazy way.
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                    }
                                    extraction_final_folder.push(folder);

                                    // The last thing in the path is the new file, so we don't have to
                                    // create a folder for it.
                                    if index < (tree_path_len - 1) {
                                        match DirBuilder::new().create(&extraction_final_folder) {
                                            Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                        };
                                    }
                                }

                                // And finally, we extract our file to the desired destiny.
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &extraction_final_folder
                                ) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                            else {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base path not configured."));
                            }
                        }

                        // If there is no "MyMod" selected, extract normally.
                        Mode::Normal => {

                            let file_chooser_extract_file = FileChooserNative::new(
                                "Select File destination...",
                                &app_ui.window,
                                FileChooserAction::Save,
                                "Extract",
                                "Cancel"
                            );

                            file_chooser_extract_file.set_current_name(&tree_path.last().unwrap());
                            if file_chooser_extract_file.run() == gtk_response_accept {
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &file_chooser_extract_file.get_filename().expect("Couldn't open file")
                                ) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                        }
                    }

                },
                TreePathType::Folder(_) => {

                    match *mode.borrow() {

                        // If there is a "MyMod" selected, we need to extract whatever we want to extracted
                        // directly to the mod's assets folder.
                        Mode::MyMod {ref game_folder_name, mod_name: _} => {

                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                                // We get his base path (where the PackFile is).
                                let mut my_mod_base_folder = my_mods_base_path.to_path_buf();
                                my_mod_base_folder.push(game_folder_name.to_owned());

                                // Now we create the folder structure of the parents of that PackedFile in the
                                // assets folder, so we have a full structure replicating the PackFile when we
                                // extract stuff from the PackFile.
                                let mut extraction_final_folder = my_mod_base_folder;
                                let mut tree_path_tweaked = tree_path.to_vec();

                                // The last folder is the one the extraction function will create, so we
                                // remove it from the path.
                                tree_path_tweaked.pop();

                                for (index, folder) in tree_path_tweaked.iter_mut().enumerate() {

                                    // The PackFile ".pack" extension NEEDS to be removed.
                                    if index == 0 && folder.ends_with(".pack"){

                                        // How to remove the last five characters of a string, lazy way.
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                        folder.pop();
                                    }
                                    extraction_final_folder.push(folder);
                                    match DirBuilder::new().create(&extraction_final_folder) {
                                        Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                    };
                                }

                                // And finally, we extract our file to the desired destiny.
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &extraction_final_folder
                                ) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                            else {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base path not configured."));
                            }
                        }

                        // If there is no "MyMod" selected, extract normally.
                        Mode::Normal => {

                            let file_chooser_extract_folder = FileChooserNative::new(
                                "Select Folder destination...",
                                &app_ui.window,
                                FileChooserAction::CreateFolder,
                                "Extract",
                                "Cancel"
                            );

                            if file_chooser_extract_folder.run() == gtk_response_accept {
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &file_chooser_extract_folder.get_filename().expect("Couldn't open file")) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                        }
                    }
                }
                TreePathType::PackFile => {

                    match *mode.borrow() {

                        // If there is a "MyMod" selected, we need to extract whatever we want to extracted
                        // directly to the mod's assets folder.
                        Mode::MyMod {ref game_folder_name, mod_name: _} => {
                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                                // We get his base path (where the PackFile is).
                                let mut my_mod_base_folder = my_mods_base_path.to_path_buf();
                                my_mod_base_folder.push(game_folder_name.to_owned());

                                // Now we create the folder structure of the parents of that PackedFile in the
                                // assets folder, so we have a full structure replicating the PackFile when we
                                // extract stuff from the PackFile.
                                let mut extraction_final_folder = my_mod_base_folder;
                                let mut pack_file_name = tree_path[0].to_owned();

                                // How to remove the last five characters of a string in a Vec<String>, lazy way.
                                pack_file_name.pop();
                                pack_file_name.pop();
                                pack_file_name.pop();
                                pack_file_name.pop();
                                pack_file_name.pop();

                                extraction_final_folder.push(pack_file_name);
                                match DirBuilder::new().create(&extraction_final_folder) {
                                    Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                };

                                // And finally, we extract our file to the desired destiny.
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &extraction_final_folder
                                ) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                            else {
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("MyMod base path not configured."));
                            }
                        }

                        // If there is no "MyMod" selected, extract normally.
                        Mode::Normal => {

                            let file_chooser_extract_folder = FileChooserNative::new(
                                "Select Folder destination...",
                                &app_ui.window,
                                FileChooserAction::CreateFolder,
                                "Extract",
                                "Cancel"
                            );

                            if file_chooser_extract_folder.run() == gtk_response_accept {
                                match packfile::extract_from_packfile(
                                    &*pack_file_decoded.borrow(),
                                    &tree_path,
                                    &file_chooser_extract_folder.get_filename().expect("Couldn't open file")) {

                                    Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                }
                            }
                        }
                    }
                }
                TreePathType::None => ui::show_dialog(&app_ui.error_dialog, format!("You can't extract non-existent files.")),
            }
        }
    }));

    /*
    --------------------------------------------------------
                        Special Events
    --------------------------------------------------------
    */

    // When we double-click something in the TreeView (or click something already selected).
    app_ui.folder_tree_view.connect_row_activated(clone!(
        app_ui,
        pack_file_decoded => move |_,_,_| {

        // We need to NOT ALLOW to change PackFile names, as it causes problems with "MyMod", and it's
        // actually broken for normal mods.
        let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
        if let TreePathType::PackFile = get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {
            return
        }

        // First, we get the variable for the new name and spawn the popover.
        let new_name: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let rect = ui::get_rect_for_popover(&app_ui.folder_tree_view, None);
        app_ui.rename_popover.set_pointing_to(&rect);
        app_ui.rename_popover_text_entry.get_buffer().set_text(tree_path.last().unwrap());
        app_ui.rename_popover.popup();

        // Now, in the "New Name" popup, we wait until "Enter" (65293) is hit AND released.
        // In that point, we try to rename the file/folder selected. If we success, the TreeView is
        // updated. If not, we get a Dialog saying why.
        app_ui.rename_popover.connect_key_release_event(clone!(
            app_ui,
            pack_file_decoded,
            new_name => move |_, key| {

            // Get his path (so it doesn't remember his old path).
            let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

            // Get the key pressed.
            let key_val = key.get_keyval();
            if key_val == 65293 {
                let mut name_changed = false;
                *new_name.borrow_mut() = app_ui.rename_popover_text_entry.get_buffer().get_text();
                match packfile::rename_packed_file(&mut *pack_file_decoded.borrow_mut(), &tree_path, &*new_name.borrow()) {
                    Ok(_) => {
                        app_ui.rename_popover.popdown();
                        name_changed = true;
                    }
                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                }
                if name_changed {
                    ui::update_tree_view_expand_path(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        &app_ui.folder_tree_view,
                        true
                    );
                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                }
                app_ui.rename_popover_text_entry.get_buffer().set_text("");
            }
            // We need to set this to true to avoid the Enter re-fire this event again and again.
            Inhibit(true)
        }));
    }));


    // When you select a file in the TreeView, decode it with his codec, if it's implemented.
    app_ui.folder_tree_view.connect_cursor_changed(clone!(
        game_selected,
        application,
        schema,
        app_ui,
        pack_file_decoded,
        is_folder_tree_view_locked => move |_| {

        // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't
        // execute anything from here.
        if !(*is_folder_tree_view_locked.borrow()) {

            // First, we destroy any children that the packed_file_data_display we use may have, cleaning it.
            let childrens_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
            if !childrens_to_utterly_destroy.is_empty() {
                for i in &childrens_to_utterly_destroy {
                    i.destroy();
                }
            }

            // Then, we get the tree_path selected, and check if it's a folder or a file.
            let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, false);

            let mut is_a_file = false;
            let mut index: i32 = 0;
            for i in &*pack_file_decoded.borrow().pack_file_data.packed_files {
                if i.packed_file_path == tree_path {
                    is_a_file = true;
                    break;
                }
                index += 1;
            }

            // Only in case it's a file, we do something.
            if is_a_file {

                // First, we get his type to decode it properly
                let mut packed_file_type: &str = "None";
                if tree_path.last().unwrap().ends_with(".loc") {
                    packed_file_type = "LOC";
                }
                else if tree_path.last().unwrap().ends_with(".txt") ||
                        tree_path.last().unwrap().ends_with(".xml") ||
                        tree_path.last().unwrap().ends_with(".csv") ||
                        tree_path.last().unwrap().ends_with(".battle_speech_camera") ||
                        tree_path.last().unwrap().ends_with(".bob") ||
                        tree_path.last().unwrap().ends_with(".xml.shader") ||
                        //tree_path.last().unwrap().ends_with(".benchmark") || // This one needs special decoding/encoding.
                        tree_path.last().unwrap().ends_with(".variantmeshdefinition") ||
                        tree_path.last().unwrap().ends_with(".xml.material") ||
                        tree_path.last().unwrap().ends_with(".environment") ||
                        tree_path.last().unwrap().ends_with(".inl") ||
                        tree_path.last().unwrap().ends_with(".lighting") ||
                        tree_path.last().unwrap().ends_with(".wsmodel") ||
                        tree_path.last().unwrap().ends_with(".lua") {
                    packed_file_type = "TEXT";
                }
                else if tree_path.last().unwrap().ends_with(".rigid_model_v2") {
                    packed_file_type = "RIGIDMODEL"
                }
                else if tree_path.last().unwrap().ends_with(".jpg") ||
                        tree_path.last().unwrap().ends_with(".jpeg") ||
                        tree_path.last().unwrap().ends_with(".tga") ||
                        tree_path.last().unwrap().ends_with(".png") {
                    packed_file_type = "IMAGE"
                }
                else if tree_path[0] == "db" {
                    packed_file_type = "DB";
                }

                // Then, depending of his type we decode it properly (if we have it implemented support
                // for his type).
                match packed_file_type {
                    "LOC" => {

                        // We check if it's decodeable before trying it.
                        let packed_file_data_encoded = &*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data;
                        let packed_file_data_decoded = Loc::read(&packed_file_data_encoded.to_vec());
                        match packed_file_data_decoded {
                            Ok(packed_file_data_decoded) => {

                                let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));
                                // First, we create the new TreeView and all the needed stuff, and prepare it to
                                // display the data from the Loc file.
                                let packed_file_tree_view_stuff = ui::packedfile_loc::PackedFileLocTreeView::create_tree_view(&app_ui.packed_file_data_display);
                                let packed_file_tree_view = packed_file_tree_view_stuff.packed_file_tree_view;
                                let packed_file_list_store = packed_file_tree_view_stuff.packed_file_list_store;
                                let packed_file_tree_view_selection = packed_file_tree_view_stuff.packed_file_tree_view_selection;
                                let packed_file_tree_view_cell_key = packed_file_tree_view_stuff.packed_file_tree_view_cell_key;
                                let packed_file_tree_view_cell_text = packed_file_tree_view_stuff.packed_file_tree_view_cell_text;
                                let packed_file_tree_view_cell_tooltip = packed_file_tree_view_stuff.packed_file_tree_view_cell_tooltip;

                                let context_menu = packed_file_tree_view_stuff.packed_file_popover_menu;
                                let context_menu_add_rows_entry = packed_file_tree_view_stuff.packed_file_popover_menu_add_rows_entry;

                                // We enable "Multiple" selection mode, so we can do multi-row operations.
                                packed_file_tree_view_selection.set_mode(gtk::SelectionMode::Multiple);

                                // Then we populate the TreeView with the entries of the Loc PackedFile.
                                ui::packedfile_loc::PackedFileLocTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_list_store);

                                // Before setting up the actions, we clean the previous ones.
                                remove_temporal_accelerators(&application);

                                // Right-click menu actions.
                                let context_menu_packedfile_loc_add_rows = SimpleAction::new("packedfile_loc_add_rows", None);
                                let context_menu_packedfile_loc_delete_rows = SimpleAction::new("packedfile_loc_delete_rows", None);
                                let context_menu_packedfile_loc_import_csv = SimpleAction::new("packedfile_loc_import_csv", None);
                                let context_menu_packedfile_loc_export_csv = SimpleAction::new("packedfile_loc_export_csv", None);

                                application.add_action(&context_menu_packedfile_loc_add_rows);
                                application.add_action(&context_menu_packedfile_loc_delete_rows);
                                application.add_action(&context_menu_packedfile_loc_import_csv);
                                application.add_action(&context_menu_packedfile_loc_export_csv);

                                // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
                                application.set_accels_for_action("app.packedfile_loc_add_rows", &["<Primary><Shift>a"]);
                                application.set_accels_for_action("app.packedfile_loc_delete_rows", &["<Shift>Delete"]);
                                application.set_accels_for_action("app.packedfile_loc_import_csv", &["<Primary><Shift>i"]);
                                application.set_accels_for_action("app.packedfile_loc_export_csv", &["<Primary><Shift>e"]);

                                // By default, the delete action should be disabled.
                                context_menu_packedfile_loc_delete_rows.set_enabled(false);

                                // Here they come!!! This is what happen when we edit the cells.
                                // This is the key column. Here we need to restrict the String to not having " ",
                                // be empty or repeated.
                                packed_file_tree_view_cell_key.connect_edited(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_,tree_path , new_text|{

                                    // First we need to check if the value has changed. Otherwise we do nothing.
                                    let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                    let edited_cell_column = packed_file_tree_view.get_cursor();
                                    let old_text: String = packed_file_list_store.get_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id()).get().unwrap();

                                    // If the value has changed, then we need to check that the new value is
                                    // valid, as this is a key column.
                                    if old_text != new_text {
                                        let current_line = packed_file_list_store.get_iter_first().unwrap();
                                        let mut key_already_exists = false;
                                        let mut done = false;
                                        while !done {
                                            let key: String = packed_file_list_store.get_value(&current_line, 1).get().unwrap();
                                            if key == new_text {
                                                key_already_exists = true;
                                                break;
                                            }
                                            else if !packed_file_list_store.iter_next(&current_line) {
                                                done = true;
                                            }
                                        }

                                        if new_text.is_empty() {
                                            ui::show_dialog(&app_ui.error_dialog, format!("Only my hearth can be empty."));
                                        }
                                        else if new_text.contains(' ') {
                                            ui::show_dialog(&app_ui.error_dialog, format!("Spaces are not valid characters."));
                                        }
                                        else if key_already_exists {
                                            ui::show_dialog(&app_ui.error_dialog, format!("This key is already in the Loc PackedFile."));
                                        }

                                        // If it has passed all the checks without error, we update the Loc PackedFile
                                        // and save the changes.
                                        else {
                                            let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                            let edited_cell_column = packed_file_tree_view.get_cursor();
                                            packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                                            // Get the data from the table and turn it into a Vec<u8> to write it.
                                            packed_file_data_decoded.borrow_mut().packed_file_data = ui::packedfile_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                                            ::packfile::update_packed_file_data_loc(
                                                &*packed_file_data_decoded.borrow_mut(),
                                                &mut *pack_file_decoded.borrow_mut(),
                                                index as usize);
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        }
                                    }
                                }));


                                packed_file_tree_view_cell_text.connect_edited(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_,tree_path , new_text|{

                                    let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                    let edited_cell_column = packed_file_tree_view.get_cursor();
                                    packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                                    // Get the data from the table and turn it into a Vec<u8> to write it.
                                    packed_file_data_decoded.borrow_mut().packed_file_data = ui::packedfile_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                                    ::packfile::update_packed_file_data_loc(
                                        &*packed_file_data_decoded.borrow_mut(),
                                        &mut *pack_file_decoded.borrow_mut(),
                                        index as usize);
                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                }));


                                packed_file_tree_view_cell_tooltip.connect_toggled(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |cell, tree_path|{

                                    let tree_iter = packed_file_list_store.get_iter(&tree_path).unwrap();
                                    // Get (Option<TreePath>, Option<TreeViewColumn>)
                                    let edited_cell_column: u32 = packed_file_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;
                                    let new_value: bool = packed_file_list_store.get_value(&tree_iter, edited_cell_column as i32).get().unwrap();
                                    let new_value_bool = (!new_value).to_value();
                                    cell.set_active(!new_value);
                                    packed_file_list_store.set_value(&tree_iter, edited_cell_column, &new_value_bool);

                                    // Get the data from the table and turn it into a Vec<u8> to write it.
                                    packed_file_data_decoded.borrow_mut().packed_file_data = ui::packedfile_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                                    ::packfile::update_packed_file_data_loc(
                                        &*packed_file_data_decoded.borrow_mut(),
                                        &mut *pack_file_decoded.borrow_mut(),
                                        index as usize);
                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                }));


                                // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
                                // Then we calculate the position where the popup must aim, and show it.
                                //
                                // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
                                packed_file_tree_view.connect_button_release_event(clone!(
                                    context_menu => move |packed_file_tree_view, button| {

                                    let button_val = button.get_button();
                                    if button_val == 3 {
                                        let rect = ui::get_rect_for_popover(packed_file_tree_view, Some(button.get_position()));

                                        context_menu.set_pointing_to(&rect);
                                        context_menu.popup();
                                    }
                                    Inhibit(false)
                                }));

                                // We check if we can delete something on selection changes.
                                packed_file_tree_view.connect_cursor_changed(clone!(
                                    context_menu_packedfile_loc_delete_rows,
                                    packed_file_tree_view_selection => move |_| {

                                    // If the Loc PackedFile is empty, disable the delete action.
                                    if packed_file_tree_view_selection.count_selected_rows() > 0 {
                                        context_menu_packedfile_loc_delete_rows.set_enabled(true);
                                    }
                                    else {
                                        context_menu_packedfile_loc_delete_rows.set_enabled(false);
                                    }
                                }));

                                // When we hit the "Add row" button.
                                context_menu_packedfile_loc_add_rows.connect_activate(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store,
                                    context_menu_add_rows_entry,
                                    context_menu => move |_,_| {

                                    // We hide the context menu, then we get the selected file/folder, delete it and update the
                                    // TreeView. Pretty simple, actually.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        // First, we check if the input is a valid number, as I'm already seeing people
                                        // trying to add "two" rows.
                                        let number_rows = context_menu_add_rows_entry.get_buffer().get_text();
                                        match number_rows.parse::<u32>() {
                                            Ok(number_rows) => {
                                                // Then we make this the new line's "Key" field unique, so there are no
                                                // duplicate keys in the Loc PackedFile.
                                                for _ in 0..number_rows {
                                                    let mut new_key = String::new();

                                                    // Before checking for duplicates, we need to check if there is at least
                                                    // a row.
                                                    if let Some(mut current_line) = packed_file_list_store.get_iter_first() {
                                                        let mut done = false;
                                                        let mut j = 1;

                                                        while !done {
                                                            let key: String = packed_file_list_store.get_value(&current_line, 1).get().unwrap();

                                                            if key == format!("New_line_{}", j) {
                                                                current_line = packed_file_list_store.get_iter_first().unwrap();
                                                                j += 1;
                                                            }
                                                            else if !packed_file_list_store.iter_next(&current_line) {
                                                                new_key = format!("New_line_{}", j);
                                                                done = true;
                                                            }
                                                        }
                                                    }
                                                    else {
                                                        new_key = format!("New_line_1");
                                                    }

                                                    packed_file_list_store.insert_with_values(None, &[0, 1, 2, 3], &[&"New".to_value(), &new_key.to_value(), &"New_line_text".to_value(), &true.to_value()]);
                                                }

                                                // Get the data from the table and turn it into a Vec<u8> to write it.
                                                packed_file_data_decoded.borrow_mut().packed_file_data = ui::packedfile_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                                                ::packfile::update_packed_file_data_loc(
                                                    &*packed_file_data_decoded.borrow_mut(),
                                                    &mut *pack_file_decoded.borrow_mut(),
                                                    index as usize);
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, format!("You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows? But definetly not \"{}\".", Error::from(error).cause())),
                                        }
                                    }
                                }));

                                // When we hit the "Delete row" button.
                                context_menu_packedfile_loc_delete_rows.connect_activate(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store,
                                    packed_file_tree_view_selection,
                                    context_menu => move |_,_| {

                                    // We hide the context menu, then we get the selected file/folder, delete it and update the
                                    // TreeView. Pretty simple, actually.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        // (Vec<TreePath>, TreeModel)
                                        let mut selected_rows = packed_file_tree_view_selection.get_selected_rows();

                                        // Only in case there is something selected (so we have at least a TreePath)
                                        // we delete rows. We sort the rows selected and reverse them. This is because
                                        // it's the only way I found to always delete the rows in reverse (from last
                                        // to beginning) so we avoid getting missing iters due to the rest of the rows
                                        // repositioning themselves after deleting one of them.
                                        if !selected_rows.0.is_empty() {
                                            selected_rows.0.sort();
                                            for i in (0..selected_rows.0.len()).rev() {
                                                let selected_row_iter = packed_file_list_store.get_iter(&selected_rows.0[i]).unwrap();
                                                packed_file_list_store.remove(&selected_row_iter);
                                            }

                                            // Get the data from the table and turn it into a Vec<u8> to write it.
                                            packed_file_data_decoded.borrow_mut().packed_file_data = ui::packedfile_loc::PackedFileLocTreeView::return_data_from_tree_view(&packed_file_list_store);
                                            ::packfile::update_packed_file_data_loc(
                                                &*packed_file_data_decoded.borrow_mut(),
                                                &mut *pack_file_decoded.borrow_mut(),
                                                index as usize);
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        }
                                    }
                                }));

                                // When we hit the "Import to CSV" button.
                                context_menu_packedfile_loc_import_csv.connect_activate(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store,
                                    context_menu => move |_,_|{

                                    // We hide the context menu first.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        let file_chooser_packedfile_import_csv = FileChooserNative::new(
                                            "Select File to Import...",
                                            &app_ui.window,
                                            FileChooserAction::Open,
                                            "Accept",
                                            "Cancel"
                                        );

                                        file_chooser_filter_packfile(&file_chooser_packedfile_import_csv, "*.csv");

                                        // First we ask for the file to import.
                                        if file_chooser_packedfile_import_csv.run() == gtk_response_accept {

                                            // If there is an error importing, we report it.
                                            if let Err(error) = LocData::import_csv(
                                                &mut packed_file_data_decoded.borrow_mut().packed_file_data,
                                                &file_chooser_packedfile_import_csv.get_filename().expect("Couldn't open file")
                                            ) {
                                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                                            }

                                            // From this point, if the file has been imported properly, we mark the PackFile as "Modified".
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                            // Load the data to the TreeView, and save it to the encoded data too.
                                            PackedFileLocTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_list_store);
                                            update_packed_file_data_loc(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize);
                                        }
                                    }
                                }));

                                // When we hit the "Export to CSV" button.
                                context_menu_packedfile_loc_export_csv.connect_activate(clone!(
                                    app_ui,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    context_menu => move |_,_|{

                                    // We hide the context menu first.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        let file_chooser_packedfile_export_csv = FileChooserNative::new(
                                            "Save CSV File...",
                                            &app_ui.window,
                                            FileChooserAction::Save,
                                            "Save",
                                            "Cancel"
                                        );

                                        let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
                                        file_chooser_packedfile_export_csv.set_current_name(format!("{}.csv",&tree_path.last().unwrap()));

                                        if file_chooser_packedfile_export_csv.run() == gtk_response_accept {
                                            match LocData::export_csv(&packed_file_data_decoded.borrow_mut().packed_file_data, &file_chooser_packedfile_export_csv.get_filename().expect("Couldn't open file")) {
                                                Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause())
                                            }
                                        }
                                    }
                                }));
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                        }

                    }

                    // If it's a DB, we try to decode it
                    "DB" => {

                        // Button for enabling the "Decoding" mode.
                        let packed_file_decode_mode_button = Button::new_with_label("Enter decoding mode");
                        packed_file_decode_mode_button.set_hexpand(true);
                        app_ui.packed_file_data_display.attach(&packed_file_decode_mode_button, 0, 0, 1, 1);
                        app_ui.packed_file_data_display.show_all();

                        let packed_file_data_encoded = Rc::new(RefCell::new(pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data.to_vec()));
                        let packed_file_data_decoded = match *schema.borrow() {
                            Some(ref schema) => DB::read(&packed_file_data_encoded.borrow(), &*tree_path[1], &schema.clone()),
                            None => {
                                packed_file_decode_mode_button.set_sensitive(false);
                                return ui::show_dialog(&app_ui.error_dialog, format_err!("There is no Schema loaded for this game."))
                            },
                        };

                        // From here, we deal we the decoder stuff.
                        packed_file_decode_mode_button.connect_button_release_event(clone!(
                            application,
                            schema,
                            tree_path,
                            app_ui,
                            pack_file_decoded => move |packed_file_decode_mode_button ,_|{

                            // We need to disable the button. Otherwise, things will get weird.
                            packed_file_decode_mode_button.set_sensitive(false);

                            // We destroy the table view if exists, so we don't have to deal with resizing it.
                            let display_last_children = app_ui.packed_file_data_display.get_children();
                            if display_last_children.first().unwrap() != packed_file_decode_mode_button {
                                display_last_children.first().unwrap().destroy();
                            }

                            // Then create the UI..
                            let mut packed_file_decoder = ui::packedfile_db::PackedFileDBDecoder::create_decoder_view(&app_ui.packed_file_data_display);

                            // And only in case the db_header has been decoded, we do the rest.
                            match DBHeader::read(&packed_file_data_encoded.borrow()){
                                Ok(db_header) => {

                                    // We get the initial index to start decoding.
                                    let initial_index = db_header.1;

                                    // We get the Schema for his game, if exists. If we reached this point, the Schema
                                    // should exists. Otherwise, the button for this window will be disabled.
                                    let table_definition = match DB::get_schema(&*tree_path[1], db_header.0.packed_file_header_packed_file_version, &schema.borrow().clone().unwrap()) {
                                        Some(table_definition) => Rc::new(RefCell::new(table_definition)),
                                        None => Rc::new(RefCell::new(TableDefinition::new(db_header.0.packed_file_header_packed_file_version)))
                                    };

                                    // If we managed to load all the static data successfully to the "Decoder" view, we set up all the button's events.
                                    match PackedFileDBDecoder::load_data_to_decoder_view(
                                        &mut packed_file_decoder,
                                        &*tree_path[1],
                                        &packed_file_data_encoded.borrow().to_vec(),
                                        initial_index
                                    ) {
                                        Ok(_) => {

                                            // To keep it simple, we'll use the fields TreeView as "list of fields", and we'll only touch the
                                            // table_definition when getting it or creating it to load the Decoder's View, or saving it.
                                            // Also, when we are loading the data from a definition (first update with existing definition)
                                            // we'll return the index of the byte where the definition ends, so we continue decoding from it.
                                            let index_data = Rc::new(RefCell::new(PackedFileDBDecoder::update_decoder_view(
                                                &packed_file_decoder,
                                                &packed_file_data_encoded.borrow(),
                                                Some(&table_definition.borrow()),
                                                initial_index,
                                            )));

                                            // Update the versions list. Only if we have an schema, we can reach this point, so we just unwrap the schema.
                                            PackedFileDBDecoder::update_versions_list(&packed_file_decoder, &(schema.borrow().clone().unwrap()), &*tree_path[1]);

                                            // Clean the accelerators stuff.
                                            remove_temporal_accelerators(&application);

                                            // Move and delete row actions.
                                            let decoder_move_row_up = SimpleAction::new("move_row_up", None);
                                            let decoder_move_row_down = SimpleAction::new("move_row_down", None);
                                            let decoder_delete_row = SimpleAction::new("delete_row", None);

                                            application.add_action(&decoder_move_row_up);
                                            application.add_action(&decoder_move_row_down);
                                            application.add_action(&decoder_delete_row);

                                            // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
                                            application.set_accels_for_action("app.move_row_up", &["<Shift>Up"]);
                                            application.set_accels_for_action("app.move_row_down", &["<Shift>Down"]);
                                            application.set_accels_for_action("app.delete_row", &["<Shift>Delete"]);

                                            // By default, these two should be disabled.
                                            decoder_move_row_up.set_enabled(false);
                                            decoder_move_row_down.set_enabled(false);

                                            // We check if we can allow actions on selection changes.
                                            packed_file_decoder.fields_tree_view.connect_cursor_changed(clone!(
                                                decoder_move_row_up,
                                                decoder_move_row_down,
                                                decoder_delete_row,
                                                packed_file_decoder => move |_| {

                                                // If the field list is empty, disable all the actions.
                                                if packed_file_decoder.fields_tree_view.get_selection().count_selected_rows() > 0 {
                                                    decoder_move_row_up.set_enabled(true);
                                                    decoder_move_row_down.set_enabled(true);
                                                    decoder_delete_row.set_enabled(true);
                                                }
                                                else {
                                                    decoder_move_row_up.set_enabled(false);
                                                    decoder_move_row_down.set_enabled(false);
                                                    decoder_delete_row.set_enabled(false);
                                                }
                                            }));

                                            // When we press the "Move up" button.
                                            decoder_move_row_up.connect_activate(clone!(
                                                initial_index,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_,_| {

                                                // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                                                // the accels working everywhere.
                                                if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_up_button.has_focus() {

                                                    let current_iter = packed_file_decoder.fields_tree_view.get_selection().get_selected().unwrap().1;
                                                    let new_iter = current_iter.clone();
                                                    if packed_file_decoder.fields_list_store.iter_previous(&new_iter) {
                                                        packed_file_decoder.fields_list_store.move_before(&current_iter, &new_iter);
                                                    }
                                                    *index_data.borrow_mut() = update_first_row_decoded(&packed_file_data_encoded.borrow(), &packed_file_decoder.fields_list_store, &initial_index, &packed_file_decoder);
                                                }
                                            }));

                                            // When we press the "Move down" button.
                                            decoder_move_row_down.connect_activate(clone!(
                                                initial_index,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_,_| {

                                                // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                                                // the accels working everywhere.
                                                if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_down_button.has_focus() {

                                                    let current_iter = packed_file_decoder.fields_tree_view.get_selection().get_selected().unwrap().1;
                                                    let new_iter = current_iter.clone();
                                                    if packed_file_decoder.fields_list_store.iter_next(&new_iter) {
                                                        packed_file_decoder.fields_list_store.move_after(&current_iter, &new_iter);
                                                    }
                                                    *index_data.borrow_mut() = update_first_row_decoded(&packed_file_data_encoded.borrow(), &packed_file_decoder.fields_list_store, &initial_index, &packed_file_decoder);
                                                }
                                            }));

                                            // By default, these buttons are disabled.
                                            packed_file_decoder.all_table_versions_remove_definition.set_sensitive(false);
                                            packed_file_decoder.all_table_versions_load_definition.set_sensitive(false);

                                            // We check if we can allow actions on selection changes.
                                            packed_file_decoder.all_table_versions_tree_view.connect_cursor_changed(clone!(
                                                packed_file_decoder => move |_| {

                                                // If the version list is empty or nothing is selected, disable all the actions.
                                                if packed_file_decoder.all_table_versions_tree_view.get_selection().count_selected_rows() > 0 {
                                                    packed_file_decoder.all_table_versions_remove_definition.set_sensitive(true);
                                                    packed_file_decoder.all_table_versions_load_definition.set_sensitive(true);
                                                }
                                                else {
                                                    packed_file_decoder.all_table_versions_remove_definition.set_sensitive(false);
                                                    packed_file_decoder.all_table_versions_load_definition.set_sensitive(false);
                                                }
                                            }));

                                            // Logic for all the "Use this" buttons. Basically, they just check if it's possible to use their decoder for the bytes we have,
                                            // and advance the index and add their type to the fields view.
                                            packed_file_decoder.use_bool_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::Boolean,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_float_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::Float,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_integer_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::Integer,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_long_integer_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::LongInteger,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));


                                            packed_file_decoder.use_string_u8_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::StringU8,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_string_u16_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::StringU16,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_optional_string_u8_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::OptionalStringU8,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            packed_file_decoder.use_optional_string_u16_button.connect_button_release_event(clone!(
                                                table_definition,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_|{

                                                // We are going to check if this is valid when adding the field to the TreeView, so we just add it.
                                                let index_data_copy = index_data.borrow().clone();
                                                *index_data.borrow_mut() = PackedFileDBDecoder::add_field_to_data_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    &table_definition.borrow(),
                                                    &packed_file_decoder.field_name_entry.get_buffer().get_text(),
                                                    FieldType::OptionalStringU16,
                                                    packed_file_decoder.is_key_field_switch.get_active(),
                                                    &None,
                                                    &String::new(),
                                                    index_data_copy,
                                                    None
                                                );

                                                PackedFileDBDecoder::update_decoder_view(
                                                    &packed_file_decoder,
                                                    &packed_file_data_encoded.borrow(),
                                                    None,
                                                    *index_data.borrow(),
                                                );
                                                packed_file_decoder.delete_all_fields_button.set_sensitive(true);

                                                Inhibit(false)
                                            }));

                                            // When we press the "Delete all fields" button, we remove all fields from the field list,
                                            // we reset the index_data, disable de deletion buttons and update the ui, effectively
                                            // resetting the entire decoder to a blank state.
                                            packed_file_decoder.delete_all_fields_button.connect_button_release_event(clone!(
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |delete_all_fields_button ,_|{
                                                    packed_file_decoder.fields_list_store.clear();
                                                    *index_data.borrow_mut() = initial_index;

                                                    delete_all_fields_button.set_sensitive(false);

                                                    PackedFileDBDecoder::update_decoder_view(
                                                        &packed_file_decoder,
                                                        &packed_file_data_encoded.borrow(),
                                                        None,
                                                        *index_data.borrow(),
                                                    );
                                                Inhibit(false)
                                            }));

                                            // This allow us to remove a field from the list, using the decoder_delete_row action.
                                            decoder_delete_row.connect_activate(clone!(
                                                initial_index,
                                                index_data,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_,_| {

                                                // We only do something in case the focus is in the TreeView or in any of the moving buttons. This should stop problems with
                                                // the accels working everywhere.
                                                if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_up_button.has_focus() || packed_file_decoder.move_down_button.has_focus() {
                                                    if let Some(selection) = packed_file_decoder.fields_tree_view.get_selection().get_selected() {
                                                        packed_file_decoder.fields_list_store.remove(&selection.1);
                                                    }
                                                    *index_data.borrow_mut() = update_first_row_decoded(&packed_file_data_encoded.borrow(), &packed_file_decoder.fields_list_store, &initial_index, &packed_file_decoder);
                                                }
                                            }));

                                            // This allow us to replace the definition we have loaded with one from another version of the table.
                                            packed_file_decoder.all_table_versions_load_definition.connect_button_release_event(clone!(
                                                schema,
                                                tree_path,
                                                app_ui,
                                                packed_file_data_encoded,
                                                packed_file_decoder => move |_ ,_| {

                                                    // Only if we have a version selected, do something.
                                                    if let Some(version_selected) = packed_file_decoder.all_table_versions_tree_view.get_selection().get_selected() {

                                                        // Get the table's name and version selected.
                                                        let table_name = &*tree_path[1];
                                                        let version_to_load: u32 = packed_file_decoder.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                                                        // Check if the Schema actually exists. This should never show up if the schema exists,
                                                        // but the compiler doesn't know it, so we have to check it.
                                                        match *schema.borrow_mut() {
                                                            Some(ref mut schema) => {

                                                                // Get the new definition.
                                                                let table_definition = DB::get_schema(table_name, version_to_load, schema);

                                                                // Remove all the fields of the currently loaded definition.
                                                                packed_file_decoder.fields_list_store.clear();

                                                                // Reload the decoder View with the new definition loaded.
                                                                PackedFileDBDecoder::update_decoder_view(
                                                                    &packed_file_decoder,
                                                                    &packed_file_data_encoded.borrow(),
                                                                    table_definition.as_ref(),
                                                                    initial_index,
                                                                );
                                                            }
                                                            None => ui::show_dialog(&app_ui.error_dialog, format_err!("Cannot load a version of a table from a non-existant Schema."))
                                                        }
                                                    }

                                                Inhibit(false)
                                            }));

                                            // This allow us to remove an entire definition of a table for an specific version.
                                            // Basically, hitting this button deletes the selected definition.
                                            packed_file_decoder.all_table_versions_remove_definition.connect_button_release_event(clone!(
                                                schema,
                                                tree_path,
                                                app_ui,
                                                packed_file_decoder => move |_ ,_| {

                                                    // Only if we have a version selected, do something.
                                                    if let Some(version_selected) = packed_file_decoder.all_table_versions_tree_view.get_selection().get_selected() {

                                                        // Get the table's name and version selected.
                                                        let table_name = &*tree_path[1];
                                                        let version_to_delete: u32 = packed_file_decoder.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                                                        // Check if the Schema actually exists. This should never show up if the schema exists,
                                                        // but the compiler doesn't know it, so we have to check it.
                                                        match *schema.borrow_mut() {
                                                            Some(ref mut schema) => {

                                                                // Try to remove that version form the schema.
                                                                match DB::remove_table_version(table_name, version_to_delete, schema) {

                                                                    // If it worked, update the list.
                                                                    Ok(_) => PackedFileDBDecoder::update_versions_list(&packed_file_decoder, schema, &*tree_path[1]),
                                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                                }
                                                            }
                                                            None => ui::show_dialog(&app_ui.error_dialog, format_err!("Cannot delete a version from a non-existant Schema."))
                                                        }
                                                    }

                                                Inhibit(false)
                                            }));

                                            // This saves the schema to a file. It takes the "table_definition" we had for this version of our table, and put
                                            // in it all the fields we have in the fields tree_view.
                                            packed_file_decoder.save_decoded_schema.connect_button_release_event(clone!(
                                                app_ui,
                                                schema,
                                                table_definition,
                                                tree_path,
                                                pack_file_decoded,
                                                packed_file_decoder => move |_ ,_| {

                                                    // Check if the Schema actually exists. This should never show up if the schema exists,
                                                    // but the compiler doesn't know it, so we have to check it.
                                                    match *schema.borrow_mut() {
                                                        Some(ref mut schema) => {

                                                            // We get the index of our table's definitions. In case we find it, we just return it. If it's not
                                                            // the case, then we create a new table's definitions and return his index. To know if we didn't found
                                                            // an index, we just return -1 as index.
                                                            let mut table_definitions_index = match schema.get_table_definitions(&*tree_path[1]) {
                                                                Some(table_definitions_index) => table_definitions_index as i32,
                                                                None => -1i32,
                                                            };

                                                            if table_definitions_index == -1 {
                                                                schema.add_table_definitions(TableDefinitions::new(&packed_file_decoder.table_type_label.get_text().unwrap()));
                                                                table_definitions_index = schema.get_table_definitions(&*tree_path[1]).unwrap() as i32;
                                                            }
                                                            table_definition.borrow_mut().fields = packed_file_decoder.return_data_from_data_view();
                                                            schema.tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());
                                                            match Schema::save(&schema, &*pack_file_decoded.borrow().pack_file_header.pack_file_id) {
                                                                Ok(_) => ui::show_dialog(&app_ui.success_dialog, format!("Schema saved successfully.")),
                                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                            }

                                                            // After all that, we need to update the version list, as this may have created a new version.
                                                            PackedFileDBDecoder::update_versions_list(&packed_file_decoder, schema, &*tree_path[1]);
                                                        }
                                                        None => ui::show_dialog(&app_ui.error_dialog, format_err!("Cannot save this table's definitions:\nSchemas for this game are not supported yet."))
                                                    }

                                                Inhibit(false)
                                            }));

                                            // This allow us to change a field's data type in the TreeView.
                                            packed_file_decoder.fields_tree_view_cell_combo.connect_edited(clone!(
                                                packed_file_decoder => move |_, tree_path, new_value| {

                                                let tree_iter = packed_file_decoder.fields_list_store.get_iter(&tree_path).unwrap();
                                                packed_file_decoder.fields_list_store.set_value(&tree_iter, 2, &new_value.to_value());

                                            }));

                                            // This allow us to set as "key" a field in the TreeView.
                                            packed_file_decoder.fields_tree_view_cell_bool.connect_toggled(clone!(
                                                packed_file_decoder => move |cell, tree_path| {

                                                let tree_iter = packed_file_decoder.fields_list_store.get_iter(&tree_path).unwrap();
                                                let edited_cell_column = packed_file_decoder.fields_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;
                                                let new_value: bool = packed_file_decoder.fields_list_store.get_value(&tree_iter, edited_cell_column as i32).get().unwrap();
                                                let new_value_bool = (!new_value).to_value();
                                                cell.set_active(!new_value);
                                                packed_file_decoder.fields_list_store.set_value(&tree_iter, edited_cell_column, &new_value_bool);
                                            }));

                                            // This loop takes care of the interaction with string cells.
                                            for edited_cell in &packed_file_decoder.fields_tree_view_cell_string {
                                                edited_cell.connect_edited(clone!(
                                                    packed_file_decoder => move |_ ,tree_path , new_text| {

                                                    let edited_cell = packed_file_decoder.fields_list_store.get_iter(&tree_path);
                                                    let edited_cell_column = packed_file_decoder.fields_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;
                                                    packed_file_decoder.fields_list_store.set_value(&edited_cell.unwrap(), edited_cell_column, &new_text.to_value());
                                                }));
                                            }
                                        }
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                    }
                                },
                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                            }
                            Inhibit(false)
                        }));

                        // If this returns an error, we just leave the button for the decoder.
                        match packed_file_data_decoded {
                            Ok(packed_file_data_decoded) => {

                                // We try to get the "data" database, to check dependencies.
                                let pack_file_path = match &*game_selected.borrow().game {
                                    "warhammer_2" => PathBuf::from("dependency_packs/wh2.pack"),
                                    "warhammer" | _ => PathBuf::from("dependency_packs/wh.pack")
                                };

                                let dependency_database = match packfile::open_packfile(pack_file_path) {
                                    Ok(data) => Some(data.pack_file_data.packed_files.to_vec()),
                                    Err(_) => None,
                                };

                                // ONLY if we get a decoded_db, we set up the TreeView.
                                let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));
                                let table_definition = Rc::new(RefCell::new(packed_file_data_decoded.borrow().packed_file_data.table_definition.clone()));
                                let packed_file_tree_view_stuff = match ui::packedfile_db::PackedFileDBTreeView::create_tree_view(
                                    &app_ui.packed_file_data_display,
                                    &*packed_file_data_decoded.borrow(),
                                    dependency_database,
                                    &pack_file_decoded.borrow().pack_file_data.packed_files,
                                    &schema.borrow().clone().unwrap()
                                ) {
                                    Ok(data) => data,
                                    Err(error) => return ui::show_dialog(&app_ui.error_dialog, error.cause())
                                };

                                let packed_file_tree_view = packed_file_tree_view_stuff.packed_file_tree_view;
                                let packed_file_list_store = packed_file_tree_view_stuff.packed_file_list_store;

                                let packed_file_tree_view_selection = packed_file_tree_view.get_selection();

                                // Here we get our right-click menu.
                                let context_menu = packed_file_tree_view_stuff.packed_file_popover_menu;
                                let context_menu_add_rows_entry = packed_file_tree_view_stuff.packed_file_popover_menu_add_rows_entry;

                                // We enable "Multiple" selection mode, so we can do multi-row operations.
                                packed_file_tree_view_selection.set_mode(gtk::SelectionMode::Multiple);

                                if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view (
                                    &packed_file_data_decoded.borrow().packed_file_data,
                                    &packed_file_list_store
                                ) {
                                    return ui::show_dialog(&app_ui.error_dialog, error.cause());
                                }

                                // Before setting up the actions, we clean the previous ones.
                                remove_temporal_accelerators(&application);

                                // Right-click menu actions.
                                let context_menu_packedfile_db_add_rows = SimpleAction::new("packedfile_db_add_rows", None);
                                let context_menu_packedfile_db_delete_rows = SimpleAction::new("packedfile_db_delete_rows", None);
                                let context_menu_packedfile_db_clone_rows = SimpleAction::new("packedfile_db_clone_rows", None);
                                let context_menu_packedfile_db_import_csv = SimpleAction::new("packedfile_db_import_csv", None);
                                let context_menu_packedfile_db_export_csv = SimpleAction::new("packedfile_db_export_csv", None);

                                application.add_action(&context_menu_packedfile_db_add_rows);
                                application.add_action(&context_menu_packedfile_db_delete_rows);
                                application.add_action(&context_menu_packedfile_db_clone_rows);
                                application.add_action(&context_menu_packedfile_db_import_csv);
                                application.add_action(&context_menu_packedfile_db_export_csv);

                                // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
                                application.set_accels_for_action("app.packedfile_db_add_rows", &["<Primary><Shift>a"]);
                                application.set_accels_for_action("app.packedfile_db_delete_rows", &["<Shift>Delete"]);
                                application.set_accels_for_action("app.packedfile_db_clone_rows", &["<Primary><Shift>d"]);
                                application.set_accels_for_action("app.packedfile_db_import_csv", &["<Primary><Shift>i"]);
                                application.set_accels_for_action("app.packedfile_db_export_csv", &["<Primary><Shift>e"]);

                                // Enable the tooltips for the TreeView.
                                packed_file_tree_view.set_has_tooltip(true);
                                packed_file_tree_view.connect_query_tooltip(clone!(
                                    table_definition => move |tree_view, x, y,_, tooltip| {

                                        // Get the coordinates of the cell under the cursor.
                                        let cell_coords: (i32, i32) = tree_view.convert_widget_to_tree_coords(x, y);

                                        // Get the column in those coordinates, if exists.
                                        let column = tree_view.get_path_at_pos(cell_coords.0, cell_coords.1);
                                        if let Some(column) = column {
                                            if let Some(column) = column.1 {
                                                let column = column.get_sort_column_id();

                                                // We don't want to check the tooltip for the Index column, nor for the fake end column.
                                                if column >= 1 && (column as usize) <= table_definition.borrow().fields.len() {

                                                    // If it's a reference, we put to what cell is referencing in the tooltip.
                                                    let tooltip_text: String = if let Some(ref reference) = table_definition.borrow().fields[column as usize - 1].field_is_reference {
                                                        if !table_definition.borrow().fields[column as usize - 1].field_description.is_empty() {
                                                            format!("{}\n\nThis column is a reference to \"{}/{}\".",
                                                                table_definition.borrow().fields[column as usize - 1].field_description,
                                                                reference.0,
                                                                reference.1
                                                            )
                                                        }
                                                        else {
                                                            format!("This column is a reference to \"{}/{}\".",
                                                                reference.0,
                                                                reference.1
                                                            )
                                                        }

                                                    } else {
                                                        table_definition.borrow().fields[column as usize - 1].field_description.to_owned()
                                                    };

                                                    // If there is a comment for that column, we use it and show the column.
                                                    if !tooltip_text.is_empty() {
                                                        tooltip.set_text(&*tooltip_text);

                                                        // Return true to show the tooltip.
                                                        return true
                                                    }
                                                }
                                            }
                                        }

                                        // In any other case, return false.
                                        false
                                    }
                                ));
                                // These are the events to save edits in cells, one loop for every type of cell.
                                // This loop takes care of reference cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_reference {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text| {

                                        if let Some(tree_iter) = packed_file_list_store.get_iter(&tree_path) {
                                            let edited_cell_column = packed_file_tree_view.get_cursor();
                                            packed_file_list_store.set_value(&tree_iter, edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                                            // Get the data from the table and turn it into a Vec<u8> to write it.
                                            match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                Ok(data) => {
                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                    if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                        ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                    }
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                }
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                            }
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with string cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_string {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text| {

                                        let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                        let edited_cell_column = packed_file_tree_view.get_cursor();
                                        packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                                        // Get the data from the table and turn it into a Vec<u8> to write it.
                                        match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                            Ok(data) => {
                                                packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                    ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                }
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with optional_string cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_optional_string {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text|{

                                        let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                        let edited_cell_column = packed_file_tree_view.get_cursor();
                                        packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_text.to_value());

                                        // Get the data from the table and turn it into a Vec<u8> to write it.
                                        match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                            Ok(data) => {
                                                packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                    ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                }
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with I32 cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_integer {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text|{

                                        match new_text.parse::<i32>() {
                                            Ok(new_number) => {
                                                let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                                let edited_cell_column = packed_file_tree_view.get_cursor();
                                                packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_number.to_value());

                                                // Get the data from the table and turn it into a Vec<u8> to write it.
                                                match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                    Ok(data) => {
                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                        if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                            ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                        }
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                }
                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause()),
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with I64 cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_long_integer {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text|{

                                        match new_text.parse::<i64>() {
                                            Ok(new_number) => {
                                                let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                                let edited_cell_column = packed_file_tree_view.get_cursor();
                                                packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &new_number.to_value());

                                                // Get the data from the table and turn it into a Vec<u8> to write it.
                                                match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                    Ok(data) => {
                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                        if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                            ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                        }
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                }
                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause()),
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with F32 cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_float {
                                    edited_cell.connect_edited(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |_ ,tree_path , new_text|{

                                        match new_text.parse::<f32>() {
                                            Ok(new_number) => {
                                                let edited_cell = packed_file_list_store.get_iter(&tree_path);
                                                let edited_cell_column = packed_file_tree_view.get_cursor();
                                                packed_file_list_store.set_value(&edited_cell.unwrap(), edited_cell_column.1.unwrap().get_sort_column_id() as u32, &format!("{}", new_number).to_value());

                                                // Get the data from the table and turn it into a Vec<u8> to write it.
                                                match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                    Ok(data) => {
                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                        if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                            ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                        }
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                }
                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause()),
                                        }
                                    }));
                                }

                                // This loop takes care of the interaction with bool cells.
                                for edited_cell in &packed_file_tree_view_stuff.packed_file_tree_view_cell_bool {
                                    edited_cell.connect_toggled(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store => move |cell, tree_path|{

                                        let tree_iter = packed_file_list_store.get_iter(&tree_path).unwrap();
                                        // Get (Option<TreePath>, Option<TreeViewColumn>)
                                        let edited_cell_column: u32 = packed_file_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;
                                        let new_value: bool = packed_file_list_store.get_value(&tree_iter, edited_cell_column as i32).get().unwrap();
                                        let new_value_bool = (!new_value).to_value();
                                        cell.set_active(!new_value);
                                        packed_file_list_store.set_value(&tree_iter, edited_cell_column, &new_value_bool);

                                        // Get the data from the table and turn it into a Vec<u8> to write it.
                                        match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                            Ok(data) => {
                                                packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                    ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                }
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                            }
                                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                        }
                                    }));
                                }

                                // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
                                // Then we calculate the position where the popup must aim, and show it.
                                //
                                // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
                                packed_file_tree_view.connect_button_release_event(clone!(
                                    context_menu => move |packed_file_tree_view, button| {

                                    let button_val = button.get_button();
                                    if button_val == 3 {
                                        let rect = ui::get_rect_for_popover(packed_file_tree_view, Some(button.get_position()));

                                        context_menu.set_pointing_to(&rect);
                                        context_menu.popup();
                                    }

                                    Inhibit(false)
                                }));

                                // We check if we can delete something on selection changes.
                                packed_file_tree_view.connect_cursor_changed(clone!(
                                    context_menu_packedfile_db_delete_rows,
                                    context_menu_packedfile_db_clone_rows,
                                    packed_file_tree_view_selection => move |_| {

                                    // If the Loc PackedFile is empty, disable the delete action.
                                    if packed_file_tree_view_selection.count_selected_rows() > 0 {
                                        context_menu_packedfile_db_delete_rows.set_enabled(true);
                                        context_menu_packedfile_db_clone_rows.set_enabled(true);
                                    }
                                    else {
                                        context_menu_packedfile_db_delete_rows.set_enabled(false);
                                        context_menu_packedfile_db_clone_rows.set_enabled(false);
                                    }
                                }));

                                // When we hit the "Add row" button.
                                context_menu_packedfile_db_add_rows.connect_activate(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store,
                                    context_menu_add_rows_entry,
                                    context_menu => move |_,_|{
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        // First, we check if the input is a valid number, as I'm already seeing people
                                        // trying to add "two" rows.
                                        let number_rows = context_menu_add_rows_entry.get_buffer().get_text();
                                        match number_rows.parse::<u32>() {
                                            Ok(number_rows) => {

                                                let column_amount = table_definition.borrow().fields.len() + 1;
                                                for _ in 0..number_rows {

                                                    // Due to issues with types and gtk-rs, we need to create an empty line and then add the
                                                    // values to it, one by one.
                                                    let current_row = packed_file_list_store.append();
                                                    for column in 0..column_amount {

                                                        let gtk_value_field;

                                                        // First column it's always the index.
                                                        if column == 0 {
                                                            gtk_value_field = gtk::ToValue::to_value(&format!("New"));
                                                        }
                                                        else {
                                                            let field_type = &table_definition.borrow().fields[column as usize - 1].field_type;
                                                            match *field_type {
                                                                FieldType::Boolean => {
                                                                    gtk_value_field = gtk::ToValue::to_value(&false);
                                                                }
                                                                FieldType::Float => {
                                                                    gtk_value_field = gtk::ToValue::to_value(&0.0f32.to_string());
                                                                }
                                                                FieldType::Integer | FieldType::LongInteger => {
                                                                    gtk_value_field = gtk::ToValue::to_value(&0);
                                                                }
                                                                FieldType::StringU8 | FieldType::StringU16 | FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                                                                    gtk_value_field = gtk::ToValue::to_value(&String::new());
                                                                }
                                                            }
                                                        }
                                                        packed_file_list_store.set_value(&current_row, column as u32, &gtk_value_field);
                                                    }
                                                }

                                                // Get the data from the table and turn it into a Vec<u8> to write it.
                                                match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                    Ok(data) => {
                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                        if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                            ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                        }
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                                }
                                            }
                                            Err(_) => ui::show_dialog(&app_ui.error_dialog, format!("You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows?")),
                                        }
                                    }
                                }));

                                // When we hit the "Delete row" button.
                                context_menu_packedfile_db_delete_rows.connect_activate(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_tree_view,
                                    packed_file_tree_view_selection,
                                    packed_file_data_decoded,
                                    packed_file_list_store,
                                    context_menu => move |_,_|{
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        // (Vec<TreePath>, TreeModel)
                                        let mut selected_rows = packed_file_tree_view_selection.get_selected_rows();

                                        // Only in case there is something selected (so we have at least a TreePath)
                                        // we delete rows. We sort the rows selected and reverse them. This is because
                                        // it's the only way I found to always delete the rows in reverse (from last
                                        // to beginning) so we avoid getting missing iters due to the rest of the rows
                                        // repositioning themselves after deleting one of them.
                                        if !selected_rows.0.is_empty() {
                                            selected_rows.0.sort();
                                            for i in (0..selected_rows.0.len()).rev() {
                                                let selected_row_iter = packed_file_list_store.get_iter(&selected_rows.0[i]).unwrap();
                                                packed_file_list_store.remove(&selected_row_iter);
                                            }

                                            // Get the data from the table and turn it into a Vec<u8> to write it.
                                            match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                Ok(data) => {
                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                    if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                        ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                    }
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                }
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                            }
                                        }
                                    }
                                }));

                                // When we hit the "Clone row" button.
                                context_menu_packedfile_db_clone_rows.connect_activate(clone!(
                                    table_definition,
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_tree_view_selection,
                                    packed_file_list_store,
                                    context_menu => move |_,_|{
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        // (Vec<TreePath>, TreeModel)
                                        let selected_rows = packed_file_tree_view_selection.get_selected_rows();
                                        let column_amount = table_definition.borrow().fields.len() + 1;

                                        // If we have something selected...
                                        if !selected_rows.0.is_empty() {
                                            for tree_path in &selected_rows.0 {

                                                // We create the new iter, store the old one, and "copy" values from one to the other.
                                                let old_row = packed_file_list_store.get_iter(tree_path).unwrap();
                                                let new_row = packed_file_list_store.append();

                                                for column in 0..column_amount {

                                                    // First column it's always the index.
                                                    if column == 0 {
                                                        packed_file_list_store.set_value(&new_row, column as u32, &gtk::ToValue::to_value(&format!("New")));
                                                    }
                                                    else {
                                                        packed_file_list_store.set_value(&new_row, column as u32, &packed_file_list_store.get_value(&old_row, column as i32));
                                                    }
                                                }
                                            }

                                            // Get the data from the table and turn it into a Vec<u8> to write it.
                                            match ui::packedfile_db::PackedFileDBTreeView::return_data_from_tree_view(&*table_definition.borrow() ,&packed_file_list_store) {
                                                Ok(data) => {
                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;
                                                    if let Err(error) = ::packfile::update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                        ui::show_dialog(&app_ui.error_dialog, error.cause());
                                                    }
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                }
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                            }
                                        }
                                    }
                                }));

                                // When we hit the "Import from CSV" button.
                                context_menu_packedfile_db_import_csv.connect_activate(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    packed_file_list_store,
                                    context_menu => move |_,_|{

                                    // We hide the context menu first.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        let file_chooser_packedfile_import_csv = FileChooserNative::new(
                                            "Select File to Import...",
                                            &app_ui.window,
                                            FileChooserAction::Open,
                                            "Accept",
                                            "Cancel"
                                        );

                                        file_chooser_filter_packfile(&file_chooser_packedfile_import_csv, "*.csv");

                                        // First we ask for the file to import.
                                        if file_chooser_packedfile_import_csv.run() == gtk_response_accept {

                                            // Just in case the import fails after importing (for example, due to importing a CSV from another table,
                                            // or from another version of the table, and it fails while loading to table or saving to PackFile)
                                            // we save a copy of the table, so we can restore it if it fails after we modify it.
                                            let packed_file_data_copy = packed_file_data_decoded.borrow_mut().packed_file_data.clone();
                                            let mut restore_table = (false, format_err!(""));

                                            // If there is an error importing, we report it. This only edits the data after checking
                                            // that it can be decoded properly, so we don't need to restore the table in this case.
                                            if let Err(error) = DBData::import_csv(
                                                &mut packed_file_data_decoded.borrow_mut().packed_file_data,
                                                &file_chooser_packedfile_import_csv.get_filename().expect("Couldn't open file")
                                            ) {
                                                return ui::show_dialog(&app_ui.error_dialog, error.cause());
                                            }

                                            // Here we mark the PackFile as "Modified".
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                            // If there is an error loading the data (wrong table imported?), report it and restore it from the old copy.
                                            if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_list_store) {
                                                restore_table = (true, error);
                                            }

                                            // If the table loaded properly, try to save the data to the encoded file.
                                            if !restore_table.0 {
                                                if let Err(error) = update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                    restore_table = (true, error);
                                                }
                                            }

                                            // If the import broke somewhere along the way, restore the old table and report the error.
                                            if restore_table.0 {
                                                packed_file_data_decoded.borrow_mut().packed_file_data = packed_file_data_copy;
                                                ui::show_dialog(&app_ui.error_dialog, restore_table.1.cause());
                                            }
                                        }
                                    }
                                }));

                                // When we hit the "Export to CSV" button.
                                context_menu_packedfile_db_export_csv.connect_activate(clone!(
                                    app_ui,
                                    packed_file_data_decoded,
                                    packed_file_tree_view,
                                    context_menu => move |_,_|{

                                    // We hide the context menu first.
                                    context_menu.popdown();

                                    // We only do something in case the focus is in the TreeView. This should stop problems with
                                    // the accels working everywhere.
                                    if packed_file_tree_view.has_focus() {

                                        let file_chooser_packedfile_export_csv = FileChooserNative::new(
                                            "Save CSV File...",
                                            &app_ui.window,
                                            FileChooserAction::Save,
                                            "Save",
                                            "Cancel"
                                        );

                                        // Get it's tree_path and it's default name (table-table_name.csv)
                                        let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
                                        file_chooser_packedfile_export_csv.set_current_name(format!("{}-{}.csv", &tree_path[1], &tree_path.last().unwrap()));

                                        // When we select the destination file, export it and report success or error.
                                        if file_chooser_packedfile_export_csv.run() == gtk_response_accept {
                                            match DBData::export_csv(&packed_file_data_decoded.borrow_mut().packed_file_data, &file_chooser_packedfile_export_csv.get_filename().expect("Couldn't open file")) {
                                                Ok(result) => ui::show_dialog(&app_ui.success_dialog, result),
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                            }
                                        }
                                    }
                                }));
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                        }
                    }

                    // If it's a plain text file, we create a source view and try to get highlighting for
                    // his language, if it's an specific language file.
                    "TEXT" => {

                        // Before doing anything, we try to decode the data. Only if we success, we create
                        // the SourceView and add the data to it.
                        let packed_file_data_encoded = &*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data;
                        match coding_helpers::decode_string_u8(packed_file_data_encoded) {
                            Ok(string) => {

                                // First, we create a vertical Box, put a "Save" button in the top part, and left
                                // the lower part for the SourceView.
                                let packed_file_source_view_save_button = Button::new_with_label("Save to PackedFile");
                                app_ui.packed_file_data_display.attach(&packed_file_source_view_save_button, 0, 0, 1, 1);

                                // Second, we create the new SourceView (in a ScrolledWindow) and his buffer,
                                // get his buffer and put the text in it.
                                let packed_file_source_view_scroll = ScrolledWindow::new(None, None);
                                packed_file_source_view_scroll.set_hexpand(true);
                                packed_file_source_view_scroll.set_vexpand(true);
                                app_ui.packed_file_data_display.attach(&packed_file_source_view_scroll, 0, 1, 1, 1);

                                let packed_file_source_view_buffer: Buffer = Buffer::new(None);
                                let packed_file_source_view = View::new_with_buffer(&packed_file_source_view_buffer);

                                // Third, we config the SourceView for our needs.
                                packed_file_source_view.set_tab_width(4);
                                packed_file_source_view.set_show_line_numbers(true);
                                packed_file_source_view.set_indent_on_tab(true);
                                packed_file_source_view.set_highlight_current_line(true);

                                // Then, we get the Language of the file.
                                let language_manager = LanguageManager::get_default().unwrap();
                                let packedfile_language: Option<Language>;
                                if tree_path.last().unwrap().ends_with(".xml") ||
                                    tree_path.last().unwrap().ends_with(".xml.shader") ||
                                    tree_path.last().unwrap().ends_with(".xml.material") ||
                                    tree_path.last().unwrap().ends_with(".variantmeshdefinition") ||
                                    tree_path.last().unwrap().ends_with(".environment") ||
                                    tree_path.last().unwrap().ends_with(".lighting") ||
                                    tree_path.last().unwrap().ends_with(".wsmodel") {
                                    packedfile_language = language_manager.get_language("xml");
                                }
                                else if tree_path.last().unwrap().ends_with(".lua") {
                                    packedfile_language = language_manager.get_language("lua");
                                }
                                else if tree_path.last().unwrap().ends_with(".csv") {
                                    packedfile_language = language_manager.get_language("csv");
                                }
                                else if tree_path.last().unwrap().ends_with(".inl") {
                                    packedfile_language = language_manager.get_language("cpp");
                                }
                                else {
                                    packedfile_language = None;
                                }

                                // Then we set the Language of the file, if it has one.
                                if let Some(language) = packedfile_language {
                                    packed_file_source_view_buffer.set_language(&language);
                                }

                                packed_file_source_view_buffer.set_text(&*string);

                                // And show everything.
                                packed_file_source_view_scroll.add(&packed_file_source_view);
                                app_ui.packed_file_data_display.show_all();

                                // When we click in the "Save to PackedFile" button
                                packed_file_source_view_save_button.connect_button_release_event(clone!(
                                    app_ui,
                                    pack_file_decoded => move |_,_| {
                                    let packed_file_data_decoded = coding_helpers::encode_string_u8(&packed_file_source_view.get_buffer().unwrap().get_slice(
                                        &packed_file_source_view.get_buffer().unwrap().get_start_iter(),
                                        &packed_file_source_view.get_buffer().unwrap().get_end_iter(),
                                        true).unwrap());

                                    ::packfile::update_packed_file_data_text(
                                        &packed_file_data_decoded,
                                        &mut *pack_file_decoded.borrow_mut(),
                                        index as usize);

                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                    Inhibit(false)
                                }));
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                        }
                    }

                    // If it's an image, we just put it in a box and show it. Or... that was the intention.
                    // We can't load them from memory, so we need to create them in the temp folder of the
                    // system and then load them. A mess.
                    "IMAGE" => {
                        let mut temporal_file_path = std::env::temp_dir();
                        temporal_file_path.push(tree_path.last().unwrap());
                        match File::create(&temporal_file_path) {
                            Ok(mut temporal_file) => {
                                if let Err(error) = temporal_file.write_all(&(*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data.to_vec())) {
                                    ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause());
                                }
                                else {
                                    let image = Image::new_from_file(&temporal_file_path);

                                    let packed_file_source_view_scroll = ScrolledWindow::new(None, None);
                                    packed_file_source_view_scroll.add(&image);
                                    packed_file_source_view_scroll.set_hexpand(true);
                                    packed_file_source_view_scroll.set_vexpand(true);
                                    app_ui.packed_file_data_display.attach(&packed_file_source_view_scroll, 0, 0, 1, 1);
                                    app_ui.packed_file_data_display.show_all();
                                }
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause()),
                        }
                    }

                    // If it's a rigidmodel, we decode it and take care of his update events.
                    "RIGIDMODEL" => {
                        let packed_file_data_encoded = &*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data;
                        let packed_file_data_decoded = RigidModel::read(packed_file_data_encoded);
                        match packed_file_data_decoded {
                            Ok(packed_file_data_decoded) => {
                                let packed_file_data_view_stuff = match ui::packedfile_rigidmodel::PackedFileRigidModelDataView::create_data_view(&app_ui.packed_file_data_display, &packed_file_data_decoded){
                                    Ok(result) => result,
                                    Err(error) => return ui::show_dialog(&app_ui.error_dialog, Error::from(error).cause()),
                                };
                                let packed_file_save_button = packed_file_data_view_stuff.packed_file_save_button;
                                let rigid_model_game_patch_button = packed_file_data_view_stuff.rigid_model_game_patch_button;
                                let rigid_model_game_label = packed_file_data_view_stuff.rigid_model_game_label;
                                let packed_file_texture_paths = packed_file_data_view_stuff.packed_file_texture_paths;
                                let packed_file_texture_paths_index = packed_file_data_view_stuff.packed_file_texture_paths_index;
                                let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));

                                // When we hit the "Patch to Warhammer 1&2" button.
                                rigid_model_game_patch_button.connect_button_release_event(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_data_decoded => move |rigid_model_game_patch_button, _| {

                                    let packed_file_data_patch_result = packfile::patch_rigid_model_attila_to_warhammer(&mut *packed_file_data_decoded.borrow_mut());
                                    match packed_file_data_patch_result {
                                        Ok(result) => {
                                            rigid_model_game_patch_button.set_sensitive(false);
                                            rigid_model_game_label.set_text("RigidModel compatible with: \"Warhammer 1&2\".");

                                            let mut success = false;
                                            match ::packfile::update_packed_file_data_rigid(
                                                &*packed_file_data_decoded.borrow(),
                                                &mut *pack_file_decoded.borrow_mut(),
                                                index as usize
                                            ) {
                                                Ok(_) => {
                                                    success = true;
                                                    ui::show_dialog(&app_ui.success_dialog, result);
                                                },
                                                Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                            }
                                            if success {
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                            }
                                        },
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                    }
                                    Inhibit(false)
                                }));

                                // When we hit the "Save to PackFile" button.
                                packed_file_save_button.connect_button_release_event(clone!(
                                    app_ui,
                                    pack_file_decoded,
                                    packed_file_texture_paths,
                                    packed_file_texture_paths_index,
                                    packed_file_data_decoded => move |_ ,_|{

                                    let new_data = match ui::packedfile_rigidmodel::PackedFileRigidModelDataView::return_data_from_data_view(
                                        &packed_file_texture_paths,
                                        &packed_file_texture_paths_index,
                                        &mut (*packed_file_data_decoded.borrow_mut()).packed_file_data.packed_file_data_lods_data.to_vec()
                                    ) {
                                        Ok(new_data) => new_data,
                                        Err(error) => {
                                            ui::show_dialog(&app_ui.error_dialog, error.cause());
                                            return Inhibit(false);
                                        }
                                    };

                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data_lods_data = new_data;

                                    let mut success = false;
                                    match ::packfile::update_packed_file_data_rigid(
                                        &*packed_file_data_decoded.borrow(),
                                        &mut *pack_file_decoded.borrow_mut(),
                                        index as usize
                                    ) {
                                        Ok(result) => {
                                            success = true;
                                            ui::show_dialog(&app_ui.success_dialog, result)
                                        },
                                        Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                                    }
                                    if success {
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                    }
                                    Inhibit(false)
                                }));
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                        }
                    }

                    // If we reach this point, the coding to implement this type of file is not done yet,
                    // so we ignore the file.
                    _ => {
                        ui::display_help_tips(&app_ui.packed_file_data_display);
                    }
                }
            }

            // If it's a folder, then we need to display the Tips.
            else {
                ui::display_help_tips(&app_ui.packed_file_data_display);
            }
        }
    }));

    // This allow us to open a PackFile by "Drag&Drop" it into the folder_tree_view.
    app_ui.folder_tree_view.connect_drag_data_received(clone!(
        app_ui,
        settings,
        schema,
        game_selected,
        mode,
        pack_file_decoded => move |_, _, _, _, selection_data, info, _| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            let lets_do_it = if pack_file_decoded.borrow().pack_file_extra_data.is_modified {
                if app_ui.unsaved_dialog.run() == gtk_response_ok {
                    app_ui.unsaved_dialog.hide_on_delete();
                    true
                } else {
                    app_ui.unsaved_dialog.hide_on_delete();
                    false
                }
            } else { true };

            // If we got confirmation...
            if lets_do_it {
                match info {
                    0 => {
                        let pack_file_path = Url::parse(&selection_data.get_uris()[0]).unwrap().to_file_path().unwrap();
                        match packfile::open_packfile(pack_file_path) {
                            Ok(pack_file_opened) => {

                                *pack_file_decoded.borrow_mut() = pack_file_opened;
                                ui::update_tree_view(&app_ui.folder_tree_store, &*pack_file_decoded.borrow());
                                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                // Disable selected mod, if we are using it.
                                *mode.borrow_mut() = Mode::Normal;

                                // We choose the right option, depending on our PackFile.
                                match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                                    0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                                    1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                                    2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                                    3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                                    4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                                    _ => ui::show_dialog(&app_ui.error_dialog, format_err!("PackFile Type not valid.")),
                                }

                                // We deactive these menus, and only activate the one corresponding to our game.
                                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
                                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);
                                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
                                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);

                                // We choose the new GameSelected depending on what the open mod id is.
                                match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                                    "PFH5" => {
                                        game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.borrow().paths.warhammer_2);
                                        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                                        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
                                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                                    },
                                    "PFH4" | _ => {
                                        game_selected.borrow_mut().change_game_selected("warhammer", &settings.borrow().paths.warhammer_2);
                                        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                                        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
                                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                                    },
                                }

                                app_ui.menu_bar_save_packfile.set_enabled(true);
                                app_ui.menu_bar_save_packfile_as.set_enabled(true);
                                app_ui.menu_bar_change_packfile_type.set_enabled(true);

                                // Disable the controls for "MyMod".
                                app_ui.menu_bar_my_mod_delete.set_enabled(false);
                                app_ui.menu_bar_my_mod_install.set_enabled(false);
                                app_ui.menu_bar_my_mod_uninstall.set_enabled(false);

                                // Try to load the Schema for this PackFile's game.
                                *schema.borrow_mut() = Schema::load(&*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
                            }
                            Err(error) => ui::show_dialog(&app_ui.error_dialog, error.cause()),
                        }
                    }
                    _ => ui::show_dialog(&app_ui.error_dialog, format!("This type of event is not yet used.")),
                }
            }
    }));
}

//-----------------------------------------------------------------------------
// From here, there is code that was in the build_ui function, but it was
// becoming a mess to maintain, and was needed to be split.
//-----------------------------------------------------------------------------

/// This function sets the currently open PackFile as "modified" or unmodified, both in the PackFile
/// and in the title bar, depending on the value of the "is_modified" boolean.
fn set_modified(
    is_modified: bool,
    window: &ApplicationWindow,
    pack_file_decoded: &mut PackFile,
) {
    if is_modified {
        pack_file_decoded.pack_file_extra_data.is_modified = true;
        window.set_title(&format!("Rusted PackFile Manager -> {}(modified)", pack_file_decoded.pack_file_extra_data.file_name));
    }
    else {
        pack_file_decoded.pack_file_extra_data.is_modified = false;
        window.set_title(&format!("Rusted PackFile Manager -> {}", pack_file_decoded.pack_file_extra_data.file_name));
    }
}

/// This function cleans the accelerators and actions created by the PackedFile Views, so they can be
/// reused in another View.
fn remove_temporal_accelerators(application: &Application) {

    // Remove stuff of Loc View.
    application.set_accels_for_action("packedfile_loc_add_rows", &[]);
    application.set_accels_for_action("packedfile_loc_delete_rows", &[]);
    application.set_accels_for_action("packedfile_loc_import_csv", &[]);
    application.set_accels_for_action("packedfile_loc_export_csv", &[]);
    application.remove_action("packedfile_loc_add_rows");
    application.remove_action("packedfile_loc_delete_rows");
    application.remove_action("packedfile_loc_import_csv");
    application.remove_action("packedfile_loc_export_csv");

    // Remove stuff of DB View.
    application.set_accels_for_action("packedfile_db_add_rows", &[]);
    application.set_accels_for_action("packedfile_db_delete_rows", &[]);
    application.set_accels_for_action("packedfile_db_clone_rows", &[]);
    application.set_accels_for_action("packedfile_db_import_csv", &[]);
    application.set_accels_for_action("packedfile_db_export_csv", &[]);
    application.remove_action("packedfile_db_add_rows");
    application.remove_action("packedfile_db_delete_rows");
    application.remove_action("packedfile_db_clone_rows");
    application.remove_action("packedfile_db_import_csv");
    application.remove_action("packedfile_db_export_csv");

    // Remove stuff of DB decoder View.
    application.set_accels_for_action("move_row_up", &[]);
    application.set_accels_for_action("move_row_down", &[]);
    application.set_accels_for_action("delete_row", &[]);
    application.remove_action("move_row_up");
    application.remove_action("move_row_down");
    application.remove_action("delete_row");
}

/// This function updates the "First row decoded" column in the Decoder View, the current index and
/// the decoded entries. This should be called in row changes (deletion and moving, not adding).
fn update_first_row_decoded(packedfile: &[u8], list_store: &ListStore, index: &usize, decoder: &PackedFileDBDecoder) -> usize {
    let iter = list_store.get_iter_first();
    let mut index = *index;
    if let Some(current_iter) = iter {
        loop {
            // Get the type from the column...
            let field_type = match list_store.get_value(&current_iter, 2).get().unwrap() {
                "Bool"=> FieldType::Boolean,
                "Float" => FieldType::Float,
                "Integer" => FieldType::Integer,
                "LongInteger" => FieldType::LongInteger,
                "StringU8" => FieldType::StringU8,
                "StringU16" => FieldType::StringU16,
                "OptionalStringU8" => FieldType::OptionalStringU8,
                "OptionalStringU16" | _ => FieldType::OptionalStringU16,
            };

            // Get the decoded data using it's type...
            let decoded_data = decode_data_by_fieldtype(
                packedfile,
                &field_type,
                index
            );

            // Update it's index for the next field.
            index = decoded_data.1;

            // Set the new values.
            list_store.set_value(&current_iter, 6, &gtk::ToValue::to_value(&decoded_data.0));

            // Break the loop once you run out of rows.
            if !list_store.iter_next(&current_iter) {
                break;
            }
        }
    }
    PackedFileDBDecoder::update_decoder_view(
        decoder,
        packedfile,
        None,
        index,
    );
    index
}

/// This function checks if there is any newer version of RPFM released. If the dialog is None, it'll
/// show the result in the status bar.
fn check_updates(check_updates_dialog: Option<&MessageDialog>, status_bar: &Statusbar) {

    // Create new client with API base URL
    let mut client = RestClient::new("https://api.github.com").unwrap();
    client.set_header_raw("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:59.0) Gecko/20100101 Firefox/59.0");

    // If we got a dialog, use it.
    if let Some(check_updates_dialog) = check_updates_dialog {

        // GET http://httpbin.org/anything and deserialize the result automatically
        match client.get(()) {
            Ok(last_release) => {

                let last_release: LastestRelease = last_release;

                // All this depends on the fact that the releases are called "vX.X.X". We only compare
                // numbers here, so we remove everything else.
                let mut last_version = last_release.name.to_owned();
                last_version.remove(0);
                last_version.split_off(5);
                if last_version != VERSION {
                    check_updates_dialog.set_property_text(Some(&format!("New update found: {}", last_release.name)));
                    check_updates_dialog.set_property_secondary_text(Some(&format!("Download available here:\n<a href=\"{}\">{}</a>\n\nChanges:\n{}", last_release.html_url, last_release.html_url, last_release.body)));
                }
                else {
                    check_updates_dialog.set_property_text(Some(&format!("No new updates available")));
                    check_updates_dialog.set_property_secondary_text(Some(&format!("More luck next time :)")));
                }
            }
            Err(_) => {
                check_updates_dialog.set_property_text(Some(&format!("Error checking new updates")));
                check_updates_dialog.set_property_secondary_text(Some(&format!("Mmmm if this happends, there has been a problem with your connection to the Github.com server.\n\nPlease, make sure you can access to <a href=\"https:\\\\api.github.com\">https:\\\\api.github.com</a>")));
            }
        }
        check_updates_dialog.run();
        check_updates_dialog.hide_on_delete();
    }

    // If there is no dialog, use the status bar to display the result.
    else {

        // GET http://httpbin.org/anything and deserialize the result automatically
        match client.get(()) {
            Ok(last_release) => {

                let last_release: LastestRelease = last_release;

                // All this depends on the fact that the releases are called "vX.X.X". We only compare
                // numbers here, so we remove everything else.
                let mut last_version = last_release.name.to_owned();
                last_version.remove(0);
                last_version.split_off(5);
                if last_version != VERSION {
                    let message = &*format!("New update found. Check \"About/About\" for a link to the download.");
                    let context_id = status_bar.get_context_id(message);
                    status_bar.push(context_id, message);
                }
                else {
                    let message = &*format!("No new updates available.");
                    let context_id = status_bar.get_context_id(message);
                    status_bar.push(context_id, message);
                }
            }
            Err(_) => {
                let message = &*format!("Error checking new updates.");
                let context_id = status_bar.get_context_id(message);
                status_bar.push(context_id, message);
            }
        }
    }
}

/// This function adds a Filter to the provided FileChooser, using the `pattern` &str.
fn file_chooser_filter_packfile(file_chooser: &FileChooserNative, pattern: &str) {
    let filter = FileFilter::new();
    filter.add_pattern(pattern);
    file_chooser.add_filter(&filter);
}

/// Main function.
fn main() {

    // We create the application.
    let application = Application::new("com.github.frodo45127.rpfm", gio::ApplicationFlags::NON_UNIQUE).expect("Initialization failed...");

    // We initialize it.
    application.connect_startup(move |app| {
        build_ui(app);
    });

    // We start GTK. Yay.
    application.connect_activate(|_| {});

    // And we run for our lives before it explodes.
    application.run(&args().collect::<Vec<_>>());
}
