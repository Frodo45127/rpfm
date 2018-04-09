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
extern crate url;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::{
    DirBuilder, copy, remove_file, remove_dir_all
};
use std::env::args;

use failure::Error;
use url::Url;
use gio::prelude::*;
use gio::{
    SimpleAction, Menu, MenuExt, MenuModel
};
use gdk::Atom;
use gtk::prelude::*;
use gtk::{
    Builder, WindowPosition, ApplicationWindow, FileFilter, Grid, TreePath, Clipboard,
    TreeView, TreeSelection, TreeStore, ScrolledWindow, Application, CellRendererMode, TreeIter,
    CellRendererText, TreeViewColumn, Popover, Button, ListStore, ResponseType,
    ShortcutsWindow, ToVariant, Statusbar, FileChooserNative, FileChooserAction, ToValue
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
use ui::*;
use ui::packedfile_db::*;
use ui::packedfile_loc::*;
use ui::packedfile_text::*;
use ui::packedfile_image::*;
use ui::packedfile_rigidmodel::*;
use ui::settings::*;
use ui::updater::*;

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


/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This constant is used to enable or disable the generation of a new Schema file in compile time.
/// If you don't want to explicity create a new Schema for a game, leave this disabled.
const GENERATE_NEW_SCHEMA: bool = false;

/// This enum represent the current "Operational Mode" for RPFM. The allowed modes are:
/// - `Normal`: Use the default behavior for everything. This is the Default mode.
/// - `MyMod`: Use the `MyMod` specific behavior. This mode is used when you have a "MyMod" selected.
///   This mode holds a tuple `(game_folder_name, mod_name)`:
///  - `game_folder_name` is the folder name for that game in "MyMod"s folder, like `warhammer_2` or `rome_2`).
///  - `mod_name` is the name of the PackFile with `.pack` at the end.
#[derive(Clone)]
enum Mode {
    MyMod{ game_folder_name: String, mod_name: String },
    Normal,
}

/// This struct contains almost the entirety of the UI stuff, so it's not a fucking chaos when
/// going inside/outside closures. The exceptions for this struct is stuff generated after RPFM is
/// started, like the TreeView for DB PackedFiles or the DB Decoder View.
#[derive(Clone)]
struct AppUI {

    // Clipboard.
    clipboard: Clipboard,

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

    // Status bar at the bottom of the program. To show informative messages.
    status_bar: Statusbar,

    // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
    folder_tree_view: TreeView,
    folder_tree_store: TreeStore,
    folder_tree_selection: TreeSelection,

    // Column and cells for the `TreeView`.
    folder_tree_view_cell: CellRendererText,
    folder_tree_view_column: TreeViewColumn,

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
    folder_tree_view_rename_packedfile: SimpleAction,
    folder_tree_view_delete_packedfile: SimpleAction,
    folder_tree_view_extract_packedfile: SimpleAction,
}

/// One Function to rule them all, One Function to find them,
/// One Function to bring them all and in the darkness bind them.
fn build_ui(application: &Application) {

    // We get all the Arguments provided when starting RPFM. Why? If we are opening a PackFile by
    // double-clicking on it (for example, with file asociation in windows) our current dir is the
    // one where the PackFile is, not where the `rpfm-code.exe` is. So RPFM gets confused and it
    // doesn't find his settings, his schemas,... To fix this, we need to get the folder where the
    // executable is and use it as a base for all the path stuff. Note that this should only work on
    // release, as the way it works it's used by cargo to run the debug builds.
    let arguments = args().collect::<Vec<String>>();

    // In debug mode, we just take the current path (so we don't break debug builds). In Release mode,
    // we take the `.exe` path. We use unwrap here because in case of fail, we want to crash RPFM.
    let rpfm_path: PathBuf = if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    };

    // We create the `Clipboard`.
    let clipboard_atom = Atom::intern("CLIPBOARD");
    let clipboard = Clipboard::get(&clipboard_atom);

    // We import the Glade design and get all the UI objects into variables.
    let help_window = include_str!("gtk/help.ui");
    let menus = include_str!("gtk/menus.ui");
    let builder = Builder::new_from_string(help_window);

    // We add all the UI onjects to the same builder. You know, one to rule them all.
    builder.add_from_string(menus).unwrap();

    // Create the main window.
    let main_window = MainWindow::create_main_window(application, &rpfm_path);

    // The Context Menu Popover for `folder_tree_view` it's a little tricky to get. We need to
    // get the stuff it's based on and then create it and put it into the AppUI.
    let folder_tree_view_context_menu_model = builder.get_object("context_menu_packfile").unwrap();
    let folder_tree_view_context_menu = Popover::new_from_model(Some(&main_window.folder_tree_view), &folder_tree_view_context_menu_model);

    // First, create the AppUI to hold all the UI stuff. All the stuff here it's from the executable
    // so we can unwrap it without any problems.
    let app_ui = AppUI {

        // Clipboard.
        clipboard,

        // Main window.
        window: main_window.window,

        // MenuBar at the top of the Window.
        menu_bar: builder.get_object("menubar").unwrap(),

        // Section of the "MyMod" menu.
        my_mod_list: builder.get_object("my-mod-list").unwrap(),

        // Shortcut window.
        shortcuts_window: builder.get_object("shortcuts-main-window").unwrap(),

        // This is the box where all the PackedFile Views are created.
        packed_file_data_display: main_window.packed_file_data_display,

        // Status bar at the bottom of the program. To show informative messages.
        status_bar: main_window.status_bar,

        // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
        folder_tree_view: main_window.folder_tree_view,
        folder_tree_store: main_window.folder_tree_store,
        folder_tree_selection: main_window.folder_tree_selection,

        // Column and cells for the `TreeView`.
        folder_tree_view_cell: main_window.folder_tree_view_cell,
        folder_tree_view_column: main_window.folder_tree_view_column,

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
        folder_tree_view_rename_packedfile: SimpleAction::new("rename-packedfile", None),
        folder_tree_view_delete_packedfile: SimpleAction::new("delete-packedfile", None),
        folder_tree_view_extract_packedfile: SimpleAction::new("extract-packedfile", None),
    };

    // Set the main menu bar for the app. This one can appear in all the windows and needs to be
    // enabled or disabled per window.
    application.set_menubar(&app_ui.menu_bar);

    // Config stuff for `app_ui.shortcuts_window`.
    app_ui.shortcuts_window.set_title("Shortcuts");
    app_ui.shortcuts_window.set_size_request(600, 400);
    app_ui.window.set_help_overlay(Some(&app_ui.shortcuts_window));

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
    application.add_action(&app_ui.folder_tree_view_rename_packedfile);
    application.add_action(&app_ui.folder_tree_view_delete_packedfile);
    application.add_action(&app_ui.folder_tree_view_extract_packedfile);

    // Some Accels need to be specified here. Don't know why, but otherwise they do not work.
    application.set_accels_for_action("app.add-file", &["<Primary>a"]);
    application.set_accels_for_action("app.add-folder", &["<Primary>d"]);
    application.set_accels_for_action("app.add-from-packfile", &["<Primary>w"]);
    application.set_accels_for_action("app.rename-packedfile", &["<Primary>r"]);
    application.set_accels_for_action("app.delete-packedfile", &["<Primary>Delete"]);
    application.set_accels_for_action("app.extract-packedfile", &["<Primary>e"]);
    application.set_accels_for_action("win.show-help-overlay", &["<Primary><Shift>h"]);

    // We enable D&D PackFiles to `app_ui.folder_tree_view` to open them.
    let targets = vec![gtk::TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP, 0)];
    app_ui.folder_tree_view.drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);

    // Then we display the "Tips" text.
    ui::display_help_tips(&app_ui.packed_file_data_display);

    // This is to get the new schemas. It's controlled by a global const.
    if GENERATE_NEW_SCHEMA {

        // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
        // The second one should contain all the tables of the game, extracted directly from `data.pack`.
        let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_schemas/");
        let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_tables/");
        match import_schema(&assembly_kit_schemas_path, &testing_tables_path, &rpfm_path) {
            Ok(_) => ui::show_dialog(&app_ui.window, true, "Schema successfully created."),
            Err(error) => return ui::show_dialog(&app_ui.window, false, format!("Error while creating a new DB Schema file:\n{}", error.cause())),
        }
    }

    // This variable is used to "Lock" the "Decode on select" feature of `app_ui.folder_tree_view`.
    // We need it to lock this feature when we open a secondary PackFile and want to import some
    // PackedFiles to our opened PackFile.
    let is_folder_tree_view_locked = Rc::new(RefCell::new(false));

    // Here we define the `Accept` response for GTK, as it seems Restson causes it to fail to compile
    // if we get them to i32 directly in the `if` statement.
    // NOTE: For some bizarre reason, GTKFileChoosers return `Ok`, while native ones return `Accept`.
    let gtk_response_accept: i32 = ResponseType::Accept.into();

    // We need two PackFiles:
    // - `pack_file_decoded`: This one will hold our opened PackFile.
    // - `pack_file_decoded_extra`: This one will hold the PackFile opened for `app_ui.add_from_packfile`.
    let pack_file_decoded = Rc::new(RefCell::new(PackFile::new()));
    let pack_file_decoded_extra = Rc::new(RefCell::new(PackFile::new()));

    // We load the list of Supported Games here.
    // TODO: Move this to a const when const fn reach stable in Rust.
    let supported_games = Rc::new(RefCell::new(GameInfo::new()));

    // We load the settings here, and in case they doesn't exist, we create them.
    let settings = Rc::new(RefCell::new(Settings::load(&rpfm_path, &supported_games.borrow()).unwrap_or_else(|_|Settings::new(&supported_games.borrow()))));

    // Load the GTK Settings, like the Theme and Font used.
    load_gtk_settings(&app_ui.window, &settings.borrow());

    // We prepare the schema object to hold an Schema, leaving it as `None` by default.
    let schema: Rc<RefCell<Option<Schema>>> = Rc::new(RefCell::new(None));

    // This specifies the "Operational Mode" RPFM should use. By default it's Normal.
    let mode = Rc::new(RefCell::new(Mode::Normal));

    // And we prepare the stuff for the default game (paths, and those things).
    let game_selected = Rc::new(RefCell::new(GameSelected::new(&settings.borrow(), &rpfm_path, &supported_games.borrow())));

    // Set the default game as selected game.
    app_ui.menu_bar_change_game_selected.change_state(&(&settings.borrow().default_game).to_variant());

    // Prepare the "MyMod" menu. This... atrocity needs to be in the following places for MyMod to open PackFiles:
    // - At the start of the program (here).
    // - At the end of MyMod creation.
    // - At the end of MyMod deletion.
    // - At the end of settings update.
    build_my_mod_menu(
        application,
        &app_ui,
        &settings.borrow(),
        mode.clone(),
        schema.clone(),
        game_selected.clone(),
        supported_games.clone(),
        pack_file_decoded.clone(),
        pack_file_decoded_extra.clone(),
        &rpfm_path
    );

    // Check for updates at the start if we have this option enabled. Currently this hangs the UI,
    // so do it before showing the UI.
    if settings.borrow().check_updates_on_start {
        check_updates(&VERSION, None, Some(&app_ui.status_bar));
    }

    // We bring up the main window.
    app_ui.window.set_position(WindowPosition::Center);
    app_ui.window.show_all();

    // End of the "Getting Ready" part.
    // From here, it's all event handling.

    // First, we catch the close window event, and close the program when we do it.
    app_ui.window.connect_delete_event(clone!(
        application,
        pack_file_decoded,
        app_ui => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {

                // If we got confirmation...
                application.quit()
            }
            Inhibit(true)
        }
    ));

    // Set the current "Operational Mode" to `Normal`.
    set_my_mod_mode(&app_ui, mode.clone(), None);

    // Disable the "PackFile Management" actions by default.
    enable_packfile_actions(&app_ui, game_selected.clone(), false);

    // Disable all the Contextual Menu actions by default.
    app_ui.folder_tree_view_add_file.set_enabled(false);
    app_ui.folder_tree_view_add_folder.set_enabled(false);
    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
    app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
    app_ui.folder_tree_view_extract_packedfile.set_enabled(false);

    // If there is a "MyMod" path set in the settings...
    if let Some(ref path) = settings.borrow().paths.my_mods_base_path {

        // And it's a valid directory, enable the "New MyMod" button.
        if path.is_dir() { app_ui.menu_bar_my_mod_new.set_enabled(true); }

        // Otherwise, disable it.
        else { app_ui.menu_bar_my_mod_new.set_enabled(false); }
    }

    // Otherwise, disable it.
    else { app_ui.menu_bar_my_mod_new.set_enabled(false); }

    /*
    --------------------------------------------------------
                     Superior Menu: "File"
    --------------------------------------------------------
    */

    // When we hit the "New PackFile" button or use his shortcut.
    app_ui.menu_bar_new_packfile.connect_activate(clone!(
        app_ui,
        schema,
        game_selected,
        supported_games,
        rpfm_path,
        mode,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {

                // If there is no secondary PackFile opened using the "Data View" at the right side...
                if pack_file_decoded_extra.borrow().pack_file_extra_data.file_name.is_empty() {

                    // We need to destroy any children that the packed_file_data_display we use may have, cleaning it.
                    let children_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
                    if !children_to_utterly_destroy.is_empty() {
                        for i in &children_to_utterly_destroy {
                            i.destroy();
                        }
                    }

                    // Show the "Tips".
                    display_help_tips(&app_ui.packed_file_data_display);
                }

                // Get the ID for the new PackFile.
                let pack_file_id = supported_games.borrow().iter().filter(|x| x.folder_name == game_selected.borrow().game).map(|x| x.id.to_owned()).collect::<String>();

                // Create the new PackFile.
                *pack_file_decoded.borrow_mut() = packfile::new_packfile("unknown.pack".to_string(), &pack_file_id);

                // Clear the `TreeView` before updating it (fixes CTD with borrowed PackFile).
                app_ui.folder_tree_store.clear();

                // Build the `TreeView`.
                ui::update_treeview(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    TreeViewOperation::Build,
                    TreePathType::None,
                );

                // Set the new mod as "Not modified".
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                // Enable the actions available for the PackFile from the `MenuBar`.
                enable_packfile_actions(&app_ui, game_selected.clone(), true);

                // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                set_my_mod_mode(&app_ui, mode.clone(), None);

                // Try to load the Schema for this PackFile's game.
                *schema.borrow_mut() = Schema::load(&rpfm_path, &*pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();
            }
        }
    ));


    // When we hit the "Open PackFile" button.
    app_ui.menu_bar_open_packfile.connect_activate(clone!(
        app_ui,
        game_selected,
        rpfm_path,
        schema,
        settings,
        mode,
        supported_games,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {

                // If we got confirmation...
                let file_chooser_open_packfile = FileChooserNative::new(
                    "Open PackFile...",
                    &app_ui.window,
                    FileChooserAction::Open,
                    "Accept",
                    "Cancel"
                );

                // We only want to open PackFiles, so only show them.
                file_chooser_filter_packfile(&file_chooser_open_packfile, "*.pack");

                // In case we have a default path for the game selected, we use it as base path for opening files.
                if let Some(ref path) = game_selected.borrow().game_data_path {

                    // We check that actually exists before setting it.
                    if path.is_dir() {
                        file_chooser_open_packfile.set_current_folder(&path);
                    }
                }

                // If we hit "Accept"...
                if file_chooser_open_packfile.run() == gtk_response_accept {

                    // Open the PackFile (or die trying it!).
                    if let Err(error) = open_packfile(
                        file_chooser_open_packfile.get_filename().unwrap(),
                        &rpfm_path,
                        &app_ui,
                        &settings.borrow(),
                        mode.clone(),
                        &mut schema.borrow_mut(),
                        &supported_games.borrow(),
                        game_selected.clone(),
                        (false, None),
                        pack_file_decoded.clone(),
                        pack_file_decoded_extra.clone()
                    ) { ui::show_dialog(&app_ui.window, false, error.cause()) };
                }
            }
        }
    ));


    // When we hit the "Save PackFile" button
    app_ui.menu_bar_save_packfile.connect_activate(clone!(
        pack_file_decoded,
        app_ui => move |_,_| {

            // If our PackFile already exists in the filesystem, we save it to that file directly.
            if pack_file_decoded.borrow().pack_file_extra_data.file_path.is_file() {

                // We try to save the PackFile at the provided path...
                let success = match packfile::save_packfile(&mut *pack_file_decoded.borrow_mut(), None) {
                    Ok(result) => {
                        ui::show_dialog(&app_ui.window, true, result);
                        true
                    },
                    Err(error) => {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                        false
                    }
                };

                // If we succeed...
                if success {

                    // Set the mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                }
            }

            // If our PackFile doesn't exist in the filesystem (it's new, or the base PackFile has been deleted),
            // we trigger the "Save as" dialog.
            else { app_ui.menu_bar_save_packfile_as.activate(None); }
        }
    ));


    // When we hit the "Save PackFile as" button.
    app_ui.menu_bar_save_packfile_as.connect_activate(clone!(
        pack_file_decoded,
        game_selected,
        app_ui,
        mode => move |_,_| {

            // Create the FileChooserNative.
            let file_chooser_save_packfile = FileChooserNative::new(
                "Save PackFile as...",
                &app_ui.window,
                FileChooserAction::Save,
                "Save",
                "Cancel"
            );

            // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
            file_chooser_save_packfile.set_do_overwrite_confirmation(true);

            // We are only interested in seeing ".pack" files.
            file_chooser_filter_packfile(&file_chooser_save_packfile, "*.pack");

            // We put the current name of the file as "Suggested" name.
            file_chooser_save_packfile.set_current_name(&pack_file_decoded.borrow().pack_file_extra_data.file_name);

            // If we are saving an existing PackFile with another name, we start in his current path.
            if pack_file_decoded.borrow().pack_file_extra_data.file_path.is_file() {
                file_chooser_save_packfile.set_filename(&pack_file_decoded.borrow().pack_file_extra_data.file_path);
            }

            // In case we have a default path for the game selected and that path is valid, we use it as base path for saving our PackFile.
            else if let Some(ref path) = game_selected.borrow().game_data_path {

                // We check it actually exists before setting it.
                if path.is_dir() {
                    file_chooser_save_packfile.set_current_folder(path);
                }
            }

            // If we hit "Accept" (and "Accept" again if we are overwriting a PackFile)...
            if file_chooser_save_packfile.run() == gtk_response_accept {

                // Get the new PackFile's path.
                let mut file_path = file_chooser_save_packfile.get_filename().unwrap();

                // If the new PackFile's name doesn't end in ".pack", we add it at the end.
                if !file_path.ends_with(".pack") { file_path.set_extension("pack"); }

                // We try to save the PackFile at the provided path...
                let success = match packfile::save_packfile(&mut *pack_file_decoded.borrow_mut(), Some(file_path.to_path_buf())) {
                    Ok(result) => {
                        ui::show_dialog(&app_ui.window, true, result);
                        true
                    },
                    Err(error) => {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                        false
                    }
                };

                // If we succeed...
                if success {

                    // Set the mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Select the first `TreeIter`, so the rename works.
                    app_ui.folder_tree_selection.select_iter(&app_ui.folder_tree_store.get_iter_first().unwrap());

                    // Update the TreeView to reflect the possible PackFile name change.
                    ui::update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Rename(file_path.file_name().unwrap().to_string_lossy().as_ref().to_owned()),
                        TreePathType::None,
                    );

                    // Set the current "Operational Mode" to Normal, just in case "MyMod" is the current one.
                    set_my_mod_mode(&app_ui, mode.clone(), None);
                }
            }
        }
    ));

    // When changing the type of the opened PackFile.
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
                    _ => ui::show_dialog(&app_ui.window, false, "PackFile Type not valid."),
                }
            }
        }
    ));

    // When we hit the "Preferences" button.
    app_ui.menu_bar_preferences.connect_activate(clone!(
        app_ui,
        game_selected,
        supported_games,
        pack_file_decoded,
        settings,
        rpfm_path,
        mode,
        application,
        pack_file_decoded_extra,
        schema => move |_,_| {

            // We disable the action, so we can't start 2 "Settings" windows at the same time.
            app_ui.menu_bar_preferences.set_enabled(false);

            // We create the "Settings Window" and load our current settings to it.
            let settings_stuff = Rc::new(RefCell::new(SettingsWindow::create_settings_window(&application, &rpfm_path, &supported_games.borrow())));
            settings_stuff.borrow().load_to_settings_window(&*settings.borrow());

            // When we press the "Accept" button.
            settings_stuff.borrow().settings_accept.connect_button_release_event(clone!(
                pack_file_decoded,
                app_ui,
                settings_stuff,
                settings,
                game_selected,
                supported_games,
                rpfm_path,
                schema,
                mode,
                pack_file_decoded_extra,
                application => move |_,_| {

                    // Save a copy of our old `Settings` to use in the checks below.
                    let old_settings = settings.borrow().clone();

                    // Save the current `Settings` from the "Settings Window" as our new `Settings`.
                    *settings.borrow_mut() = settings_stuff.borrow().save_from_settings_window(&supported_games.borrow());

                    // Save our new `Settings` to a settings file, and report in case of error.
                    if let Err(error) = settings.borrow().save(&rpfm_path) {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                    }

                    // Destroy the "Settings Window".
                    settings_stuff.borrow().settings_window.destroy();

                    // Restore the action, so we can open another "Settings Window" again.
                    app_ui.menu_bar_preferences.set_enabled(true);

                    // If we changed the "MyMod's Folder" path...
                    if settings.borrow().paths.my_mods_base_path != old_settings.paths.my_mods_base_path {

                        // And we have currently opened a "MyMod"...
                        match *mode.borrow() {
                            Mode::MyMod {mod_name: _, game_folder_name: _} => {

                                // We disable the "MyMod" mode, but leave the PackFile open, so the user doesn't lose any unsaved change.
                                set_my_mod_mode(&app_ui, mode.clone(), None);

                                // Then recreate the "MyMod" submenu.
                                build_my_mod_menu(
                                    &application,
                                    &app_ui,
                                    &settings.borrow(),
                                    mode.clone(),
                                    schema.clone(),
                                    game_selected.clone(),
                                    supported_games.clone(),
                                    pack_file_decoded.clone(),
                                    pack_file_decoded_extra.clone(),
                                    &rpfm_path
                                );
                            }
                            _ => {}
                        }
                    }

                    // If there is a "MyMod" path set in the settings...
                    if let Some(ref path) = settings.borrow().paths.my_mods_base_path {

                        // And it's a valid directory, enable the "New MyMod" button.
                        if path.is_dir() { app_ui.menu_bar_my_mod_new.set_enabled(true); }

                        // Otherwise, disable it.
                        else { app_ui.menu_bar_my_mod_new.set_enabled(false); }
                    }

                    // Otherwise, disable it.
                    else { app_ui.menu_bar_my_mod_new.set_enabled(false); }

                    // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                    // update the current `GameSelected`.
                    let new_game_paths = settings.borrow().paths.game_paths.clone();
                    let game_paths = new_game_paths.iter().zip(old_settings.paths.game_paths.iter());
                    let changed_paths_games = game_paths.filter(|x| x.0.path != x.1.path).map(|x| x.0.game.to_owned()).collect::<Vec<String>>();

                    // If our current `GameSelected` is in the `changed_paths_games` list...
                    if changed_paths_games.contains(&game_selected.borrow().game) {

                        // Re-select the same game, so `GameSelected` update his paths.
                        let new_game_selected = game_selected.borrow().game.to_owned();
                        app_ui.menu_bar_change_game_selected.activate(Some(&new_game_selected.to_variant()));
                    }
                    Inhibit(false)
                }
            ));

            // When we press the "Cancel" button, we close the window.
            settings_stuff.borrow().settings_cancel.connect_button_release_event(clone!(
                settings_stuff,
                settings,
                rpfm_path,
                supported_games,
                app_ui => move |_,_| {

                    // Destroy the "Settings Window".
                    settings_stuff.borrow().settings_window.destroy();

                    // Restore the action, so we can open another "Settings Window" again.
                    app_ui.menu_bar_preferences.set_enabled(true);

                    // Reload the old `Settings` from the "Settings File" so, if we have changed anything, it's undone.
                    *settings.borrow_mut() = Settings::load(&rpfm_path, &supported_games.borrow()).unwrap_or_else(|_|Settings::new(&supported_games.borrow()));

                    // Reload the GTK-Related settings.
                    load_gtk_settings(&app_ui.window, &settings.borrow());

                    Inhibit(false)
                }
            ));

            // We catch the destroy event to restore the "Preferences" button.
            settings_stuff.borrow().settings_window.connect_delete_event(clone!(
                settings,
                rpfm_path,
                supported_games,
                app_ui => move |settings_window, _| {

                    // Destroy the "Settings Window".
                    settings_window.destroy();

                    // Restore the action, so we can open another "Settings Window" again.
                    app_ui.menu_bar_preferences.set_enabled(true);

                    // Reload the old `Settings` from the "Settings File" so, if we have changed anything, it's undone.
                    *settings.borrow_mut() = Settings::load(&rpfm_path, &supported_games.borrow()).unwrap_or_else(|_|Settings::new(&supported_games.borrow()));

                    // Reload the GTK-Related settings.
                    load_gtk_settings(&app_ui.window, &settings.borrow());

                    Inhibit(false)
                }
            ));
        }
    ));

    // When we hit the "Quit" button.
    app_ui.menu_bar_quit.connect_activate(clone!(
        application,
        pack_file_decoded,
        app_ui => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {
                application.quit();
            }
        }
    ));

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
        rpfm_path,
        mode,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // We disable the action, so we can't start 2 "New MyMod" windows at the same time.
            app_ui.menu_bar_my_mod_new.set_enabled(false);

            // Create the the "New MyMod" window and put all it's stuff into a variable.
            let new_mod_stuff = Rc::new(RefCell::new(MyModNewWindow::create_my_mod_new_window(&application, &supported_games.borrow(), &game_selected.borrow(), &settings.borrow(), &rpfm_path)));

            // When we press the "Accept" button.
            new_mod_stuff.borrow().my_mod_new_accept.connect_button_release_event(clone!(
                new_mod_stuff,
                application,
                app_ui,
                settings,
                schema,
                mode,
                supported_games,
                rpfm_path,
                game_selected,
                pack_file_decoded_extra,
                pack_file_decoded => move |_,_| {

                    // Get the mod name.
                    let mod_name = new_mod_stuff.borrow().my_mod_new_name_entry.get_buffer().get_text();

                    // Get the PackFile name.
                    let full_mod_name = format!("{}.pack", mod_name);

                    // Change the `GameSelected` with the one we have chosen for the new "MyMod".
                    let new_mod_game = &*new_mod_stuff.borrow().my_mod_new_game_list_combo.get_active_id().unwrap().to_owned();
                    app_ui.menu_bar_change_game_selected.activate(Some(&new_mod_game.to_variant()));

                    // Get the ID for the new PackFile.
                    let pack_file_id = supported_games.borrow().iter().filter(|x| x.folder_name == game_selected.borrow().game).map(|x| x.id.to_owned()).collect::<String>();

                    // Create the new PackFile.
                    *pack_file_decoded.borrow_mut() = packfile::new_packfile(full_mod_name.to_owned(), &pack_file_id);

                    // Clear the `TreeView` before updating it (fixes CTD with borrowed PackFile).
                    app_ui.folder_tree_store.clear();

                    // Build the `TreeView`.
                    ui::update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Build,
                        TreePathType::None,
                    );

                    // Set the new mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, game_selected.clone(), true);

                    // Get his new path from the base "MyMod" path + `new_mod_game`.
                    let mut my_mod_path = settings.borrow().paths.my_mods_base_path.clone().unwrap();
                    my_mod_path.push(&new_mod_game);

                    // Just in case the folder doesn't exist, we try to create it. It's save to ignore this result.
                    match DirBuilder::new().create(&my_mod_path){
                        Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                    };

                    // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                    let mut my_mod_private_folder = my_mod_path.to_path_buf();
                    my_mod_private_folder.push(mod_name.to_owned());
                    match DirBuilder::new().create(&my_mod_private_folder) {
                        Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                    };

                    // Add the PackFile name to the full path.
                    my_mod_path.push(full_mod_name.to_owned());

                    // Then we try to save the new "MyMod"s PackFile, and show a message in case of error.
                    if let Err(error) = packfile::save_packfile(&mut pack_file_decoded.borrow_mut(), Some(my_mod_path.to_owned())) {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                    }

                    // If the new "MyMod" has been saved successfully...
                    else {

                        // Set the current "Operational Mode" to `MyMod`.
                        set_my_mod_mode(&app_ui, mode.clone(), Some(my_mod_path));

                        // Recreate the "MyMod" menu.
                        build_my_mod_menu(
                            &application,
                            &app_ui,
                            &settings.borrow(),
                            mode.clone(),
                            schema.clone(),
                            game_selected.clone(),
                            supported_games.clone(),
                            pack_file_decoded.clone(),
                            pack_file_decoded_extra.clone(),
                            &rpfm_path
                        );

                        // Destroy the "New MyMod" window,
                        new_mod_stuff.borrow().my_mod_new_window.destroy();

                        // Restore the action, so we can open another "New MyMod" window again.
                        app_ui.menu_bar_my_mod_new.set_enabled(true);
                    }
                    Inhibit(false)
                }
            ));

            // When we press the "Cancel" button, we close the window and re-enable the "New mod" action.
            new_mod_stuff.borrow().my_mod_new_cancel.connect_button_release_event(clone!(
                new_mod_stuff,
                app_ui => move |_,_| {

                    // Destroy the "New MyMod" window,
                    new_mod_stuff.borrow().my_mod_new_window.destroy();

                    // Restore the action, so we can open another "New MyMod" window again.
                    app_ui.menu_bar_my_mod_new.set_enabled(true);
                    Inhibit(false)
                }
            ));

            // We catch the destroy event to restore the "New mod" action.
            new_mod_stuff.borrow().my_mod_new_window.connect_delete_event(clone!(
                app_ui => move |my_mod_new_window, _| {

                    // Destroy the "New MyMod" window,
                    my_mod_new_window.destroy();

                    // Restore the action, so we can open another "New MyMod" window again.
                    app_ui.menu_bar_my_mod_new.set_enabled(true);
                    Inhibit(false)
                }
            ));
        }
    ));

    // When we hit the "Delete" button.
    app_ui.menu_bar_my_mod_delete.connect_activate(clone!(
        app_ui,
        application,
        settings,
        schema,
        game_selected,
        rpfm_path,
        mode,
        supported_games,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // This will delete stuff from disk, so we pop up the "Are you sure?" message to avoid accidents.
            if ui::are_you_sure(&app_ui.window, true, true) {

                // We want to keep our "MyMod" name for the success message, so we store it here.
                let old_mod_name: String;

                // If we have a "MyMod" selected...
                let mod_deleted = match *mode.borrow() {
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // We save the name of the PackFile for later use.
                        old_mod_name = mod_name.to_owned();

                        // And the "MyMod" path is configured...
                        if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                            // We get his path.
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(&game_folder_name);
                            my_mod_path.push(&mod_name);

                            // We check that path exists.
                            if !my_mod_path.is_file() {
                                return ui::show_dialog(&app_ui.window, false, "PackFile doesn't exist.");
                            }

                            // And we delete that PackFile.
                            if let Err(error) = remove_file(&my_mod_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.window, false, error.cause());
                            }

                            // Now we get his asset folder.
                            let mut my_mod_assets_path = my_mod_path.clone();
                            my_mod_assets_path.pop();
                            my_mod_assets_path.push(&my_mod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists. This is optional, so it should allow the deletion
                            // process to continue with a warning.
                            if !my_mod_assets_path.is_dir() {
                                ui::show_dialog(&app_ui.window, false, "Mod deleted, but his assets folder hasn't been found.");
                            }

                            // If the assets folder exists, we try to delete it.
                            else if let Err(error) = remove_dir_all(&my_mod_assets_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.window, false, error.cause());
                            }

                            // We return true, as we have delete the files of the "MyMod".
                            true
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else {
                            return ui::show_dialog(&app_ui.window, false, "MyMod base path not configured.");
                        }
                    }

                    // If we don't have a "MyMod" selected, return an error.
                    Mode::Normal => return ui::show_dialog(&app_ui.window, false, "MyMod not selected."),
                };

                // If we deleted the "MyMod", we allow chaos to form below.
                if mod_deleted {

                    // Set the current "Operational Mode" to `Normal`.
                    set_my_mod_mode(&app_ui, mode.clone(), None);

                    // Replace the open PackFile with a dummy one, like during boot.
                    *pack_file_decoded.borrow_mut() = PackFile::new();

                    // Disable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, game_selected.clone(), false);

                    // Set the dummy mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Clear the TreeView.
                    app_ui.folder_tree_store.clear();

                    // Rebuild the "MyMod" menu.
                    build_my_mod_menu(
                        &application,
                        &app_ui,
                        &settings.borrow(),
                        mode.clone(),
                        schema.clone(),
                        game_selected.clone(),
                        supported_games.clone(),
                        pack_file_decoded.clone(),
                        pack_file_decoded_extra.clone(),
                        &rpfm_path
                    );

                    // Show the "MyMod" deleted Dialog.
                    ui::show_dialog(&app_ui.window, true, format!("MyMod \"{}\" deleted.", old_mod_name));
                }
            }
        }
    ));

    // When we hit the "Install" button.
    app_ui.menu_bar_my_mod_install.connect_activate(clone!(
        app_ui,
        mode,
        game_selected,
        settings => move |_,_| {

            // Depending on our current "Mode", we choose what to do.
            match *mode.borrow() {

                // If we have a "MyMod" selected...
                Mode::MyMod {ref game_folder_name, ref mod_name} => {

                    // And the "MyMod" path is configured...
                    if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                        // Get the `game_data_path` of the game.
                        let game_data_path = game_selected.borrow().game_data_path.clone();

                        // If we have a `game_data_path` for the current `GameSelected`...
                        if let Some(mut game_data_path) = game_data_path {

                            // We get the "MyMod"s PackFile path.
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(&game_folder_name);
                            my_mod_path.push(&mod_name);

                            // We check that the "MyMod"s PackFile exists.
                            if !my_mod_path.is_file() {
                                return ui::show_dialog(&app_ui.window, false, "PackFile doesn't exist.");
                            }

                            // We check that the destination path exists.
                            if !game_data_path.is_dir() {
                                return ui::show_dialog(&app_ui.window, false, "Destination folder (..xxx/data) doesn't exist. You sure you configured the right folder for the game?");
                            }

                            // Get the destination path for the PackFile with the PackFile included.
                            game_data_path.push(&mod_name);

                            // And copy the PackFile to his destination. If the copy fails, return an error.
                            if let Err(error) = copy(my_mod_path, game_data_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.window, false, error.cause());
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else {
                            return ui::show_dialog(&app_ui.window, false, "Game folder path not configured.");
                        }

                    // If the "MyMod" path is not configured, return an error.
                    }
                    else {
                        ui::show_dialog(&app_ui.window, false, "MyMod base path not configured.");
                    }
                }

                // If we have no MyMod selected, return an error.
                Mode::Normal => ui::show_dialog(&app_ui.window, false, "MyMod not selected."),
            }
        }
    ));

    // When we hit the "Uninstall" button.
    app_ui.menu_bar_my_mod_uninstall.connect_activate(clone!(
        app_ui,
        mode,
        game_selected => move |_,_| {

            // Depending on our current "Mode", we choose what to do.
            match *mode.borrow() {

                // If we have a "MyMod" selected...
                Mode::MyMod {game_folder_name: _, ref mod_name} => {

                    // Get the `game_data_path` of the game.
                    let game_data_path = game_selected.borrow().game_data_path.clone();

                    // If we have a `game_data_path` for the current `GameSelected`...
                    if let Some(mut game_data_path) = game_data_path {

                        // Get the destination path for the PackFile with the PackFile included.
                        game_data_path.push(&mod_name);

                        // We check that the "MyMod" is actually installed in the provided path.
                        if !game_data_path.is_file() {
                            return ui::show_dialog(&app_ui.window, false, "The currently selected \"MyMod\" is not installed.");
                        }

                        // If the "MyMod" is installed...
                        else {

                            // We remove it. If there is a problem deleting it, return an error dialog.
                            if let Err(error) = remove_file(game_data_path).map_err(|error| Error::from(error)) {
                                return ui::show_dialog(&app_ui.window, false, error.cause());
                            }
                        }
                    }

                    // If we don't have a `game_data_path` configured for the current `GameSelected`...
                    else {
                        ui::show_dialog(&app_ui.window, false, "Game folder path not configured.");
                    }
                }

                // If we have no MyMod selected, return an error.
                Mode::Normal => ui::show_dialog(&app_ui.window, false, "MyMod not selected."),
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
        supported_games,
        game_selected => move |menu_bar_change_game_selected, selected| {

        // Get the new state of the action.
        if let Some(state) = selected.clone() {
            let new_state: String = state.get().unwrap();

            // Change the state of the action.
            menu_bar_change_game_selected.change_state(&new_state.to_variant());

            // Change the `GameSelected` object.
            game_selected.borrow_mut().change_game_selected(&new_state, &settings.borrow().paths.game_paths.iter().filter(|x| x.game == new_state).map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games.borrow());
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
            patch_siege_ai(&app_ui, pack_file_decoded.clone());
        }
    ));

    // When we hit the "Generate Dependency Pack" button.
    app_ui.menu_bar_generate_dependency_pack_wh2.connect_activate(clone!(
        app_ui,
        rpfm_path,
        game_selected => move |_,_| {
            generate_dependency_pack(&app_ui, &rpfm_path, game_selected.clone());
        }
    ));

    // When we hit the "Patch SiegeAI" button (Warhammer).
    app_ui.menu_bar_patch_siege_ai_wh.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_| {
            patch_siege_ai(&app_ui, pack_file_decoded.clone());
        }
    ));

    // When we hit the "Generate Dependency Pack" button (Warhammer).
    app_ui.menu_bar_generate_dependency_pack_wh.connect_activate(clone!(
        game_selected,
        rpfm_path,
        app_ui => move |_,_| {
            generate_dependency_pack(&app_ui, &rpfm_path, game_selected.clone());
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
            check_updates(&VERSION, Some(&app_ui.window), None);
        }
    ));

    // When we hit the "About" button.
    app_ui.menu_bar_about.connect_activate(clone!(
        rpfm_path,
        app_ui => move |_,_| {
            ui::show_about_window(VERSION, &rpfm_path, &app_ui.window);
        }
    ));

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

            // If we Right-Click and there is something selected...
            if button.get_button() == 3 && app_ui.folder_tree_selection.count_selected_rows() > 0 {

                // Get a Rectangle over the selected line, and popup the Contextual Menu.
                let rect = ui::get_rect_for_popover(&app_ui.folder_tree_view, Some(button.get_position()));
                app_ui.folder_tree_view_context_menu.set_pointing_to(&rect);
                app_ui.folder_tree_view_context_menu.popup();
            }
            Inhibit(false)
        }
    ));

    // We check every action possible for the selected file when changing the cursor.
    app_ui.folder_tree_view.connect_cursor_changed(clone!(
        pack_file_decoded,
        app_ui => move |_| {

            // Get the path of the selected thing.
            let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

            // Get the type of the selected thing.
            let selection_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

            // Depending on the type of the selected thing, we enable or disable different actions.
            match selection_type {

                // If it's a file...
                TreePathType::File(_) => {
                    app_ui.folder_tree_view_add_file.set_enabled(false);
                    app_ui.folder_tree_view_add_folder.set_enabled(false);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                },

                // If it's a folder...
                TreePathType::Folder(_) => {
                    app_ui.folder_tree_view_add_file.set_enabled(true);
                    app_ui.folder_tree_view_add_folder.set_enabled(true);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                },

                // If it's the PackFile...
                TreePathType::PackFile => {
                    app_ui.folder_tree_view_add_file.set_enabled(true);
                    app_ui.folder_tree_view_add_folder.set_enabled(true);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                },

                // If there is nothing selected...
                TreePathType::None => {
                    app_ui.folder_tree_view_add_file.set_enabled(false);
                    app_ui.folder_tree_view_add_folder.set_enabled(false);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(false);
                },
            }
        }
    ));

    // When we hit the "Add file" button.
    app_ui.folder_tree_view_add_file.connect_activate(clone!(
        app_ui,
        settings,
        mode,
        pack_file_decoded => move |_,_| {

        // First, we hide the context menu.
        app_ui.folder_tree_view_context_menu.popdown();

        // We only do something in case the focus is in the TreeView. This should stop problems with
        // the accels working everywhere.
        if app_ui.folder_tree_view.has_focus() {

            // Create our `FileChooser` to select the files to add.
            let file_chooser_add_file_to_packfile = FileChooserNative::new(
                "Select File...",
                &app_ui.window,
                FileChooserAction::Open,
                "Accept",
                "Cancel"
            );

            // Check the current "Operational Mode".
            match *mode.borrow() {

                // If we are in "MyMod" mode...
                Mode::MyMod {ref game_folder_name, ref mod_name} => {

                    // In theory, if we reach this line this should always exist. In theory I should be rich.
                    if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                        // We get the assets path for the selected "MyMod".
                        let mut my_mod_path = my_mods_base_path.to_path_buf();
                        my_mod_path.push(&game_folder_name);
                        my_mod_path.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                        // We check that path exists, and create it if it doesn't.
                        if !my_mod_path.is_dir() {
                            match DirBuilder::new().create(&my_mod_path) {
                                Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                            };
                        }

                        // Then we set that path as current path for the "Add PackedFile" file chooser.
                        file_chooser_add_file_to_packfile.set_current_folder(&my_mod_path);

                        // If we hit "Accept"...
                        if file_chooser_add_file_to_packfile.run() == gtk_response_accept {

                            // Get the names of the files to add.
                            let paths = file_chooser_add_file_to_packfile.get_filenames();

                            // For each one of them...
                            for path in &paths {

                                // If we are inside the mod's folder, we need to "emulate" the path to then
                                // file in the TreeView, so we add the file with a custom tree_path.
                                if path.starts_with(&my_mod_path) {

                                    // Turn both paths into `Vec<String>`, so we can compare them better.
                                    let path_vec = path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                    let my_mod_path_vec = my_mod_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();

                                    // Get the index from where his future tree_path starts.
                                    let index = my_mod_path_vec.len();

                                    // Get his `TreeView` tree_path.
                                    let tree_path = path_vec[index..].to_vec();

                                    // Try to add it to the PackFile.
                                    let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path.to_vec()) {
                                        Ok(_) => true,
                                        Err(error) => {
                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                            false
                                        }
                                    };

                                    // If we had success adding it...
                                    if success {

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                        // Update the TreeView to show the newly added PackedFile.
                                        ui::update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::Add(tree_path.to_vec()),
                                            TreePathType::None,
                                        );
                                    }
                                }

                                // If not, we get their tree_path like a normal file.
                                else {

                                    // Get his `TreeView` path.
                                    let tree_path = ui::get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);

                                    // Try to add it to the PackFile.
                                    let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path.to_vec()) {
                                        Ok(_) => true,
                                        Err(error) => {
                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                            false
                                        }
                                    };

                                    // If we had success adding it...
                                    if success {

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                        // Update the TreeView to show the newly added PackedFile.
                                        ui::update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::Add(tree_path.to_vec()),
                                            TreePathType::None,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    else {
                        return ui::show_dialog(&app_ui.window, false, "MyMod base folder not configured.");
                    }
                },

                // If there is no "MyMod" selected, we just keep the normal behavior.
                Mode::Normal => {

                    // If we hit the "Accept" button...
                    if file_chooser_add_file_to_packfile.run() == gtk_response_accept {

                        // Get all the files selected.
                        let paths = file_chooser_add_file_to_packfile.get_filenames();

                        // For each file to add...
                        for path in &paths {

                            // Get his `TreeView` path.
                            let tree_path = ui::get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);

                            // Try to add it to the PackFile.
                            let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path.to_vec()) {
                                Ok(_) => true,
                                Err(error) => {
                                    ui::show_dialog(&app_ui.window, false, error.cause());
                                    false
                                }
                            };

                            // If we had success adding it...
                            if success {

                                // Set the mod as "Modified".
                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                // Update the TreeView to show the newly added PackedFile.
                                ui::update_treeview(
                                    &app_ui.folder_tree_store,
                                    &*pack_file_decoded.borrow(),
                                    &app_ui.folder_tree_selection,
                                    TreeViewOperation::Add(tree_path.to_vec()),
                                    TreePathType::None,
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

            // First, we hide the context menu.
            app_ui.folder_tree_view_context_menu.popdown();

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Create the `FileChooser`.
                let file_chooser_add_folder_to_packfile = FileChooserNative::new(
                    "Select Folder...",
                    &app_ui.window,
                    FileChooserAction::SelectFolder,
                    "Accept",
                    "Cancel"
                );

                // Check the current "Operational Mode".
                match *mode.borrow() {

                    // If the current mode is "MyMod"...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                            // We get the assets path for the selected "MyMod".
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(&game_folder_name);
                            my_mod_path.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !my_mod_path.is_dir() {
                                match DirBuilder::new().create(&my_mod_path) {
                                    Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                };
                            }

                            // Then we set that path as current path for the "Add PackedFile" file chooser.
                            file_chooser_add_folder_to_packfile.set_current_folder(&my_mod_path);

                            // If we hit "Accept"...
                            if file_chooser_add_folder_to_packfile.run() == gtk_response_accept {

                                // Get the selected folders.
                                let folders = file_chooser_add_folder_to_packfile.get_filenames();

                                // For each folder...
                                for folder in &folders {

                                    // If we are inside the mod's folder, we need to "emulate" the path to then
                                    // file in the TreeView, so we add the file with a custom tree_path.
                                    if folder.starts_with(&my_mod_path) {

                                        // Turn both paths into `Vec<String>`, so we can compare them better.
                                        let path_vec = folder.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                        let my_mod_path_vec = my_mod_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();

                                        // Get the index from where his future tree_path starts.
                                        let index = my_mod_path_vec.len();

                                        // Get his `TreeView` tree_path.
                                        let tree_path = path_vec[index..].to_vec();

                                        // Get the "Prefix" of the folder.
                                        let mut big_parent_prefix = folder.clone();
                                        big_parent_prefix.pop();

                                        // Get all the files inside that folder recursively.
                                        match get_files_from_subdir(folder) {

                                            // If we succeed...
                                            Ok(file_path_list) => {

                                                // For each file...
                                                for file_path in file_path_list {

                                                    // Remove his prefix, leaving only the path from the folder onwards.
                                                    match file_path.strip_prefix(&big_parent_prefix) {

                                                        // If there is no problem...
                                                        Ok(filtered_path) => {

                                                            // Then get their unique tree_path, combining our current tree_path
                                                            // with the filtered_path we got for them.
                                                            let mut filtered_path = filtered_path.iter().map(|t| t.to_str().unwrap().to_string()).collect::<Vec<String>>();
                                                            let mut tree_path = tree_path.clone();
                                                            tree_path.pop();
                                                            tree_path.append(&mut filtered_path);

                                                            // Try to add it to the PackFile.
                                                            let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &file_path.to_path_buf(), tree_path.to_vec()) {
                                                                Ok(_) => true,
                                                                Err(error) => {
                                                                    ui::show_dialog(&app_ui.window, false, error.cause());
                                                                    false
                                                                }
                                                            };

                                                            // If we had success adding it...
                                                            if success {

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                // Update the TreeView to show the newly added PackedFile.
                                                                ui::update_treeview(
                                                                    &app_ui.folder_tree_store,
                                                                    &*pack_file_decoded.borrow(),
                                                                    &app_ui.folder_tree_selection,
                                                                    TreeViewOperation::Add(tree_path.to_vec()),
                                                                    TreePathType::None,
                                                                );
                                                            }
                                                        }

                                                        // If there is an error while removing the prefix...
                                                        Err(_) => ui::show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                                    }
                                                }
                                            }

                                            // If there is an error while getting the files to add...
                                            Err(_) => ui::show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
                                        }
                                    }

                                    // If not, we get their tree_path like a normal folder.
                                    else {

                                        // Get the "Prefix" of the folder.
                                        let mut big_parent_prefix = folder.clone();
                                        big_parent_prefix.pop();

                                        // Get all the files inside that folder recursively.
                                        match get_files_from_subdir(folder) {

                                            // If we succeed...
                                            Ok(file_path_list) => {

                                                // For each file...
                                                for file_path in file_path_list {

                                                    // Remove his prefix, leaving only the path from the folder onwards.
                                                    match file_path.strip_prefix(&big_parent_prefix) {

                                                        // If there is no problem...
                                                        Ok(filtered_path) => {

                                                            // Get his `tree_path`.
                                                            let tree_path = get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &app_ui.folder_tree_selection, false);

                                                            // Try to add it to the PackFile.
                                                            let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &file_path.to_path_buf(), tree_path.to_vec()) {
                                                                Ok(_) => true,
                                                                Err(error) => {
                                                                    ui::show_dialog(&app_ui.window, false, error.cause());
                                                                    false
                                                                }
                                                            };

                                                            // If we had success adding it...
                                                            if success {

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                // Update the TreeView to show the newly added PackedFile.
                                                                ui::update_treeview(
                                                                    &app_ui.folder_tree_store,
                                                                    &*pack_file_decoded.borrow(),
                                                                    &app_ui.folder_tree_selection,
                                                                    TreeViewOperation::Add(tree_path.to_vec()),
                                                                    TreePathType::None,
                                                                );
                                                            }
                                                        }

                                                        // If there is an error while removing the prefix...
                                                        Err(_) => ui::show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                                    }
                                                }
                                            }

                                            // If there is an error while getting the files to add...
                                            Err(_) => ui::show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
                                        }
                                    }
                                }
                            }
                        }
                        else {
                            return ui::show_dialog(&app_ui.window, false, "MyMod base folder not configured.");
                        }
                    }

                    // If there is no "MyMod" selected, we just keep the normal behavior.
                    Mode::Normal => {

                        // If we hit "Accept"...
                        if file_chooser_add_folder_to_packfile.run() == gtk_response_accept {

                            // Get the folders we want to add.
                            let folders = file_chooser_add_folder_to_packfile.get_filenames();

                            // For each folder...
                            for folder in &folders {

                                // Get the "Prefix" of the folder.
                                let mut big_parent_prefix = folder.clone();
                                big_parent_prefix.pop();

                                // Get all the files inside that folder recursively.
                                match get_files_from_subdir(folder) {

                                    // If we succeed...
                                    Ok(file_path_list) => {

                                        // For each file...
                                        for file_path in file_path_list {

                                            // Remove his prefix, leaving only the path from the folder onwards.
                                            match file_path.strip_prefix(&big_parent_prefix) {

                                                // If there is no problem...
                                                Ok(filtered_path) => {

                                                    // Get his `tree_path`.
                                                    let tree_path = get_tree_path_from_pathbuf(&filtered_path.to_path_buf(), &app_ui.folder_tree_selection, false);

                                                    // Try to add it to the PackFile.
                                                    let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), &file_path.to_path_buf(), tree_path.to_vec()) {
                                                        Ok(_) => true,
                                                        Err(error) => {
                                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                                            false
                                                        }
                                                    };

                                                    // If we had success adding it...
                                                    if success {

                                                        // Set the mod as "Modified".
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        // Update the TreeView to show the newly added PackedFile.
                                                        ui::update_treeview(
                                                            &app_ui.folder_tree_store,
                                                            &*pack_file_decoded.borrow(),
                                                            &app_ui.folder_tree_selection,
                                                            TreeViewOperation::Add(tree_path.to_vec()),
                                                            TreePathType::None,
                                                        );
                                                    }
                                                }

                                                // If there is an error while removing the prefix...
                                                Err(_) => ui::show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                            }
                                        }
                                    }

                                    // If there is an error while getting the files to add...
                                    Err(_) => ui::show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
                                }
                            }
                        }
                    }
                }
            }
        }
    ));

    // When we hit the "Add file/folder from PackFile" button.
    app_ui.folder_tree_view_add_from_packfile.connect_activate(clone!(
        app_ui,
        pack_file_decoded,
        pack_file_decoded_extra,
        is_folder_tree_view_locked => move |_,_| {

            // First, we hide the context menu.
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

                // Create the `FileChooser`.
                let file_chooser_add_from_packfile = FileChooserNative::new(
                    "Select PackFile...",
                    &app_ui.window,
                    FileChooserAction::Open,
                    "Accept",
                    "Cancel"
                );

                // Set his filter to only admit ".pack" files.
                file_chooser_filter_packfile(&file_chooser_add_from_packfile, "*.pack");

                // If we hit "Accept"...
                if file_chooser_add_from_packfile.run() == gtk_response_accept {

                    // Try to open the selected PackFile.
                    match packfile::open_packfile(file_chooser_add_from_packfile.get_filename().unwrap()) {

                        // If the extra PackFile is valid...
                        Ok(pack_file_opened) => {

                            // We create the "Exit" and "Copy" buttons.
                            let exit_button = Button::new_with_label("Exit \"Add file/folder from PackFile\" mode");
                            let copy_button = Button::new_with_label("<=");
                            exit_button.set_vexpand(false);
                            copy_button.set_hexpand(false);

                            // We attach them to the main grid.
                            app_ui.packed_file_data_display.attach(&exit_button, 0, 0, 2, 1);
                            app_ui.packed_file_data_display.attach(&copy_button, 0, 1, 1, 1);

                            // We create the new TreeView (in a ScrolledWindow) and his TreeStore.
                            let folder_tree_view_extra = TreeView::new();
                            let folder_tree_store_extra = TreeStore::new(&[String::static_type()]);
                            folder_tree_view_extra.set_model(Some(&folder_tree_store_extra));

                            // We create his column.
                            let column_extra = TreeViewColumn::new();
                            let cell_extra = CellRendererText::new();
                            column_extra.pack_start(&cell_extra, true);
                            column_extra.add_attribute(&cell_extra, "text", 0);

                            // Configuration for the `TreeView`.
                            folder_tree_view_extra.append_column(&column_extra);
                            folder_tree_view_extra.set_enable_tree_lines(true);
                            folder_tree_view_extra.set_enable_search(false);
                            folder_tree_view_extra.set_headers_visible(false);

                            // We create an `ScrolledWindow` for the `TreeView`.
                            let folder_tree_view_extra_scroll = ScrolledWindow::new(None, None);
                            folder_tree_view_extra_scroll.set_hexpand(true);
                            folder_tree_view_extra_scroll.set_vexpand(true);
                            folder_tree_view_extra_scroll.add(&folder_tree_view_extra);
                            app_ui.packed_file_data_display.attach(&folder_tree_view_extra_scroll, 1, 1, 1, 1);

                            // Show everything.
                            app_ui.packed_file_data_display.show_all();

                            // Block the main `TreeView` from decoding stuff.
                            *is_folder_tree_view_locked.borrow_mut() = true;

                            // Store the second PackFile's data.
                            *pack_file_decoded_extra.borrow_mut() = pack_file_opened;

                            // Build the second `TreeView`.
                            ui::update_treeview(
                                &folder_tree_store_extra,
                                &*pack_file_decoded_extra.borrow(),
                                &folder_tree_view_extra.get_selection(),
                                TreeViewOperation::Build,
                                TreePathType::None,
                            );

                            // We need to check here if the selected destination is not a file. Otherwise,
                            // we should disable the "Copy" button.
                            app_ui.folder_tree_selection.connect_changed(clone!(
                            copy_button,
                            pack_file_decoded => move |folder_tree_selection| {

                                    // Get his path.
                                    let tree_path = ui::get_tree_path_from_selection(folder_tree_selection, true);

                                    // Only in case it's not a file, we enable the "Copy" Button.
                                    match get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {
                                        TreePathType::File(_) => copy_button.set_sensitive(false),
                                        _ => copy_button.set_sensitive(true),
                                    }
                                }
                            ));

                            // When we click in the "Copy" button (<=).
                            copy_button.connect_button_release_event(clone!(
                                app_ui,
                                pack_file_decoded,
                                pack_file_decoded_extra,
                                folder_tree_view_extra => move |_,_| {

                                    // Get his source & destination paths.
                                    let tree_path_source = ui::get_tree_path_from_selection(&folder_tree_view_extra.get_selection(), true);
                                    let tree_path_destination = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                                    // Get the source & destination types.
                                    let selection_type = get_type_of_selected_tree_path(&tree_path_destination, &pack_file_decoded.borrow());

                                    // Try to add the PackedFile to the main PackFile.
                                    let success = match packfile::add_packedfile_to_packfile(
                                        &*pack_file_decoded_extra.borrow(),
                                        &mut *pack_file_decoded.borrow_mut(),
                                        &tree_path_source,
                                        &tree_path_destination,
                                    ) {
                                        Ok(_) => true,
                                        Err(error) => {
                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                            false
                                        }
                                    };

                                    // If it succeed...
                                    if success {

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                        // Get the new "Prefix" for the PackedFiles.
                                        let mut source_prefix = tree_path_source;

                                        // Remove the PackFile's name from it.
                                        source_prefix.reverse();
                                        source_prefix.pop();
                                        source_prefix.reverse();

                                        // Get the new "Prefix" for the Destination PackedFiles.
                                        let mut destination_prefix = tree_path_destination;

                                        // Remove the PackFile's name from it.
                                        destination_prefix.reverse();
                                        destination_prefix.pop();
                                        destination_prefix.reverse();

                                        // Get all the PackedFiles to copy.
                                        let path_list: Vec<Vec<String>> = pack_file_decoded_extra.borrow()
                                            .pack_file_data.packed_files
                                            .iter()
                                            .filter(|x| x.packed_file_path.starts_with(&source_prefix))
                                            .map(|x| x.packed_file_path.to_vec())
                                            .collect();

                                        // Update the TreeView to show the newly added PackedFiles.
                                        ui::update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::AddFromPackFile(source_prefix.to_vec(), destination_prefix.to_vec(), path_list),
                                            selection_type,
                                        );
                                    }

                                    Inhibit(false)
                                }
                            ));

                            // When we click in the "Exit "Add file/folder from PackFile" mode" button.
                            exit_button.connect_button_release_event(clone!(
                                app_ui,
                                pack_file_decoded_extra,
                                is_folder_tree_view_locked => move |_,_| {

                                    // Remove the `pack_file_decoded_extra` from memory.
                                    *pack_file_decoded_extra.borrow_mut() = PackFile::new();

                                    // Unlock the `TreeView`.
                                    *is_folder_tree_view_locked.borrow_mut() = false;

                                    // We need to destroy any children that the packed_file_data_display we use may have, cleaning it.
                                    let children_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
                                    if !children_to_utterly_destroy.is_empty() {
                                        for i in &children_to_utterly_destroy {
                                            i.destroy();
                                        }
                                    }

                                    // Show the "Tips".
                                    ui::display_help_tips(&app_ui.packed_file_data_display);

                                    Inhibit(false)
                                }
                            ));

                        }
                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                    }
                }
            }
        }
    ));

    // The "Rename" action requires multiple things to happend, so we group them together.
    {
        let old_snake = Rc::new(RefCell::new(String::new()));

        // When we hit the "Rename file/folder" button, we start editing the file we want to rename.
        app_ui.folder_tree_view_rename_packedfile.connect_activate(clone!(
            app_ui,
            old_snake,
            pack_file_decoded => move |_,_|{

                // We hide the context menu.
                app_ui.folder_tree_view_context_menu.popdown();

                // We only do something in case the focus is in the TreeView. This should stop problems with
                // the accels working everywhere.
                if app_ui.folder_tree_view.has_focus() {

                    // If we have at least one file selected...
                    if app_ui.folder_tree_selection.get_selected_rows().0.len() >= 1 {

                        // If the selected file/folder turns out to be the PackFile, stop right there, criminal scum.
                        let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
                        if let TreePathType::PackFile = get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {
                            return
                        }

                        // Set the cells to "Editable" mode, so we can edit them.
                        app_ui.folder_tree_view_cell.set_property_mode(CellRendererMode::Editable);

                        // Get the `TreePath` of what we want to rename.
                        let tree_path: TreePath = app_ui.folder_tree_selection.get_selected_rows().0[0].clone();

                        // Get the old name of the file/folder, for restoring purpouses.
                        let tree_iter = app_ui.folder_tree_store.get_iter(&tree_path).unwrap();
                        *old_snake.borrow_mut() = app_ui.folder_tree_store.get_value(&tree_iter, 0).get().unwrap();

                        // Start editing the name at the selected `TreePath`.
                        app_ui.folder_tree_view.set_cursor(&tree_path, Some(&app_ui.folder_tree_view_column), true);
                    }
                }
            }
        ));

        // When the edition is finished...
        app_ui.folder_tree_view_cell.connect_edited(clone!(
            pack_file_decoded,
            old_snake,
            app_ui => move |cell,_, new_name| {

                // Get the `tree_path` of the selected file/folder...
                let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                // Get his type.
                let selection_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

                // And try to rename it.
                let success = match packfile::rename_packed_file(&mut *pack_file_decoded.borrow_mut(), &tree_path, &new_name) {
                    Ok(_) => true,
                    Err(error) => {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                        false
                    }
                };

                // If we renamed the file/folder successfully...
                if success {

                    // Set the mod as "Modified".
                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Rename whatever is selected (and his childs, if it have any) from the `TreeView`.
                    ui::update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Rename(new_name.to_owned()),
                        selection_type,
                    );
                }

                // If we didn't rename the file, restore his old name.
                else {
                    cell.set_property_text(Some(&old_snake.borrow()));
                }

                // Set the cells back to "Activatable" mode.
                cell.set_property_mode(CellRendererMode::Activatable);
            }
        ));

        // When the edition is canceled...
        app_ui.folder_tree_view_cell.connect_editing_canceled(move |cell| {

                // Set the cells back to "Activatable" mode.
                cell.set_property_mode(CellRendererMode::Activatable);
            }
        );
    }

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

                // Get his `tree_path`.
                let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                // Get his type.
                let selection_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

                // Try to delete whatever is selected.
                let success = match packfile::delete_from_packfile(&mut *pack_file_decoded.borrow_mut(), &tree_path) {
                    Ok(_) => true,
                    Err(error) => {
                        ui::show_dialog(&app_ui.window, false, error.cause());
                        false
                    }
                };

                // If we succeed...
                if success {

                    // Set the mod as "Modified".
                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Remove whatever is selected (and his childs, if it have any) from the `TreeView`.
                    ui::update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Delete,
                        selection_type,
                    );
                }
            }
        }
    ));

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

                // Get the selected path, both in complete and incomplete forms.
                let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
                let mut tree_path_incomplete = tree_path.to_vec();
                tree_path_incomplete.reverse();
                tree_path_incomplete.pop();
                tree_path_incomplete.reverse();

                // Get the type of the selection.
                let selection_type = get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow());

                // Check the current "Operational Mode".
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref my_mods_base_path) = settings.borrow().paths.my_mods_base_path {

                            // We get the assets folder of our mod.
                            let mut my_mod_path = my_mods_base_path.to_path_buf();
                            my_mod_path.push(&game_folder_name);
                            my_mod_path.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !my_mod_path.is_dir() {
                                match DirBuilder::new().create(&my_mod_path) {
                                    Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                };
                            }

                            // Create the path for the final destination of the file.
                            let mut extraction_final_folder = my_mod_path.to_path_buf();

                            // If it's a file or a folder...
                            if selection_type == TreePathType::File((vec![String::new()], 1)) || selection_type == TreePathType::Folder(vec![String::new()]) {

                                // If it's a folder, remove the last directory, as that one will be created when extracting.
                                if selection_type == TreePathType::Folder(vec![String::new()]) { tree_path_incomplete.pop(); }

                                // For each folder in his path...
                                for (index, folder) in tree_path_incomplete.iter().enumerate() {

                                    // Complete the extracted path.
                                    extraction_final_folder.push(folder);

                                    // The last thing in the path is the new file, so we don't have to
                                    // create a folder for it.
                                    if index < (tree_path_incomplete.len() - 1) {
                                        match DirBuilder::new().create(&extraction_final_folder) {
                                            Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                        };
                                    }
                                }
                            }

                            // And finally, we extract our file to the desired destiny.
                            match packfile::extract_from_packfile(
                                &*pack_file_decoded.borrow(),
                                &tree_path,
                                &extraction_final_folder
                            ) {
                                Ok(result) => ui::show_dialog(&app_ui.window, true, result),
                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause())
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else {
                            return ui::show_dialog(&app_ui.window, false, "MyMod base path not configured.");
                        }
                    }

                    // If there is no "MyMod" selected, extract normally.
                    Mode::Normal => {

                        // Create the `FileChooser`.
                        let file_chooser_extract =

                            // If we have selected a file...
                            if selection_type == TreePathType::File((vec![String::new()], 1)) {

                                // Create a `FileChooser` to extract files.
                                let file_chooser = FileChooserNative::new(
                                    "Select File destination...",
                                    &app_ui.window,
                                    FileChooserAction::Save,
                                    "Extract",
                                    "Cancel"
                                );

                                // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                                file_chooser.set_do_overwrite_confirmation(true);

                                // Return it.
                                file_chooser
                            }

                            // If we have selected a folder or the PackFile...
                            else if selection_type == TreePathType::Folder(vec![String::new()]) ||
                                 selection_type == TreePathType::PackFile {

                                // Create a `FileChooser` to extract folders.
                                FileChooserNative::new(
                                    "Select Folder destination...",
                                    &app_ui.window,
                                    FileChooserAction::CreateFolder,
                                    "Extract",
                                    "Cancel"
                                )
                            }

                            // Otherwise, return an error.
                            else {
                                return ui::show_dialog(&app_ui.window, false, "You can't extract non-existent files.");
                            };

                        // If we have selected a file...
                        if selection_type == TreePathType::File((vec![String::new()], 1)) {

                            // Set the `FileChooser` current name to the PackFile's name.
                            file_chooser_extract.set_current_name(&tree_path.last().unwrap());
                        }

                        // If we hit "Extract"...
                        if file_chooser_extract.run() == gtk_response_accept {

                            // Get the extraction path.
                            let mut extraction_path = file_chooser_extract.get_filename().unwrap();

                            // If we have selected the PackFile...
                            if selection_type == TreePathType::PackFile {

                                // Add the PackFile's name to the path.
                                extraction_path.push(&app_ui.folder_tree_store.get_value(&app_ui.folder_tree_store.get_iter_first().unwrap(), 0).get::<String>().unwrap());

                                // We check that path exists, and create it if it doesn't.
                                if !extraction_path.is_dir() {
                                    match DirBuilder::new().create(&extraction_path) {
                                        Ok(_) | Err(_) => { /* This returns ok if it created the folder and err if it already exist. */ }
                                    };
                                }
                            }

                            // Try to extract the PackFile.
                            match packfile::extract_from_packfile(
                                &*pack_file_decoded.borrow(),
                                &tree_path,
                                &extraction_path
                            ) {
                                Ok(result) => ui::show_dialog(&app_ui.window, true, result),
                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause())
                            }
                        }
                    }
                }
            }
        }
    ));

    /*
    --------------------------------------------------------
                        Special Events
    --------------------------------------------------------
    */

    // When we press "->", we expand the selected folder (if it's a folder). We do the oposite thing with "<-".
    app_ui.folder_tree_view.connect_key_release_event(clone!(
        pack_file_decoded,
        app_ui => move |_, key| {

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Get the pressed key.
                let key_val = key.get_keyval();

                // If we press "->"...
                if key_val == 65363 {

                    // We get whatever is selected.
                    let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                    // We get the type of the selected thing.
                    match get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {

                        // If the selected thing it's `PackFile` or `Folder`...
                        TreePathType::PackFile | TreePathType::Folder(_) => {

                            // Get his `TreePath`.
                            let tree_path: TreePath = app_ui.folder_tree_selection.get_selected_rows().0[0].clone();

                            // And expand it.
                            app_ui.folder_tree_view.expand_row(&tree_path, false);
                        },
                        _ => {},
                    }
                }

                // If we press "<-"...
                else if key_val == 65361 {

                    // We get whatever is selected.
                    let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                    // We get the type of the selected thing.
                    match get_type_of_selected_tree_path(&tree_path, &*pack_file_decoded.borrow()) {

                        // If the selected thing it's `PackFile` or `Folder`...
                        TreePathType::PackFile | TreePathType::Folder(_) => {

                            // Get his `TreePath`.
                            let tree_path: TreePath = app_ui.folder_tree_selection.get_selected_rows().0[0].clone();

                            // And collapse it.
                            app_ui.folder_tree_view.collapse_row(&tree_path);
                        },
                        _ => {},
                    }
                }
            }

            Inhibit(false)
        }
    ));

    // When we double-click a file in the `TreeView`, try to decode it with his codec, if it's implemented.
    app_ui.folder_tree_view.connect_row_activated(clone!(
        game_selected,
        application,
        schema,
        app_ui,
        settings,
        rpfm_path,
        pack_file_decoded,
        is_folder_tree_view_locked => move |_,_,_| {

        // Before anything else, we need to check if the `TreeView` is unlocked. Otherwise we don't do anything from here.
        if !(*is_folder_tree_view_locked.borrow()) {

            // First, we destroy any children that the `packed_file_data_display` we use may have, cleaning it.
            let childrens_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
            if !childrens_to_utterly_destroy.is_empty() {
                for i in &childrens_to_utterly_destroy {
                    i.destroy();
                }
            }

            // Then, we get the `tree_path` selected, and check what it is.
            let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
            let path_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

            // We act, depending on his type.
            match path_type {

                // Only in case it's a file, we do something.
                TreePathType::File((tree_path, index)) => {

                    // Get the name of the PackedFile (we are going to use it a lot).
                    let packedfile_name = tree_path.last().unwrap().clone();

                    // First, we get his type to decode it properly
                    let mut packed_file_type: &str =

                        // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
                        if tree_path[0] == "db" { "DB" }

                        // If it ends in ".loc", it's a localisation PackedFile.
                        else if packedfile_name.ends_with(".loc") { "LOC" }

                        // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
                        else if packedfile_name.ends_with(".rigid_model_v2") { "RIGIDMODEL" }

                        // If it ends in any of these, it's a plain text PackedFile.
                        else if packedfile_name.ends_with(".txt") ||
                                packedfile_name.ends_with(".xml") ||
                                packedfile_name.ends_with(".csv") ||
                                packedfile_name.ends_with(".battle_speech_camera") ||
                                packedfile_name.ends_with(".bob") ||
                                packedfile_name.ends_with(".xml.shader") ||
                                //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                                packedfile_name.ends_with(".variantmeshdefinition") ||
                                packedfile_name.ends_with(".xml.material") ||
                                packedfile_name.ends_with(".environment") ||
                                packedfile_name.ends_with(".inl") ||
                                packedfile_name.ends_with(".lighting") ||
                                packedfile_name.ends_with(".wsmodel") ||
                                packedfile_name.ends_with(".lua") { "TEXT" }

                        // If it ends in any of these, it's an image.
                        else if packedfile_name.ends_with(".jpg") ||
                                packedfile_name.ends_with(".jpeg") ||
                                packedfile_name.ends_with(".tga") ||
                                packedfile_name.ends_with(".png") { "IMAGE" }

                        // Otherwise, we don't have a decoder for that PackedFile... yet.
                        else { "None" };

                    // Then, depending of his type we decode it properly (if we have it implemented support
                    // for his type).
                    match packed_file_type {

                        // If the file is a Loc PackedFile...
                        "LOC" => {

                            // We try to decode it as a Loc PackedFile.
                            match Loc::read(&*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data) {

                                // If we succeed...
                                Ok(packed_file_data_decoded) => {

                                    // Store the decoded file in a Rc<RefCell<data>> so we can pass it to closures.
                                    let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));

                                    // Create the `TreeView` for it.
                                    let packed_file_stuff = PackedFileLocTreeView::create_tree_view(&app_ui.packed_file_data_display, &settings.borrow());

                                    // We enable "Multiple" selection mode, so we can do multi-row operations.
                                    packed_file_stuff.tree_view.get_selection().set_mode(gtk::SelectionMode::Multiple);

                                    // Then we populate the TreeView with the entries of the Loc PackedFile.
                                    PackedFileLocTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_stuff.list_store);

                                    // Before setting up the actions, we clean the previous ones.
                                    remove_temporal_accelerators(&application);

                                    // Right-click menu actions.
                                    let context_menu_packedfile_loc_add_rows = SimpleAction::new("packedfile_loc_add_rows", None);
                                    let context_menu_packedfile_loc_delete_rows = SimpleAction::new("packedfile_loc_delete_rows", None);
                                    let context_menu_packedfile_loc_copy_cell = SimpleAction::new("packedfile_loc_copy_cell", None);
                                    let context_menu_packedfile_loc_paste_cell = SimpleAction::new("packedfile_loc_paste_cell", None);
                                    let context_menu_packedfile_loc_copy_rows = SimpleAction::new("packedfile_loc_copy_rows", None);
                                    let context_menu_packedfile_loc_paste_rows = SimpleAction::new("packedfile_loc_paste_rows", None);
                                    let context_menu_packedfile_loc_import_csv = SimpleAction::new("packedfile_loc_import_csv", None);
                                    let context_menu_packedfile_loc_export_csv = SimpleAction::new("packedfile_loc_export_csv", None);

                                    application.add_action(&context_menu_packedfile_loc_add_rows);
                                    application.add_action(&context_menu_packedfile_loc_delete_rows);
                                    application.add_action(&context_menu_packedfile_loc_copy_cell);
                                    application.add_action(&context_menu_packedfile_loc_paste_cell);
                                    application.add_action(&context_menu_packedfile_loc_copy_rows);
                                    application.add_action(&context_menu_packedfile_loc_paste_rows);
                                    application.add_action(&context_menu_packedfile_loc_import_csv);
                                    application.add_action(&context_menu_packedfile_loc_export_csv);

                                    // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
                                    application.set_accels_for_action("app.packedfile_loc_add_rows", &["<Primary><Shift>a"]);
                                    application.set_accels_for_action("app.packedfile_loc_delete_rows", &["<Shift>Delete"]);
                                    application.set_accels_for_action("app.packedfile_loc_copy_cell", &["<Primary>c"]);
                                    application.set_accels_for_action("app.packedfile_loc_paste_cell", &["<Primary>v"]);
                                    application.set_accels_for_action("app.packedfile_loc_copy_rows", &["<Primary>z"]);
                                    application.set_accels_for_action("app.packedfile_loc_paste_rows", &["<Primary>x"]);
                                    application.set_accels_for_action("app.packedfile_loc_import_csv", &["<Primary><Shift>i"]);
                                    application.set_accels_for_action("app.packedfile_loc_export_csv", &["<Primary><Shift>e"]);

                                    // By default, the delete action should be disabled.
                                    context_menu_packedfile_loc_delete_rows.set_enabled(false);

                                    // Here they come!!! This is what happen when we edit the cells.
                                    // This is the key column. Here we need to restrict the String to not having " ", be empty or repeated.
                                    packed_file_stuff.cell_key.connect_edited(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_, tree_path, new_text|{

                                            // Get the cell's old text, to check for changes.
                                            let tree_iter = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();
                                            let old_text: String = packed_file_stuff.list_store.get_value(&tree_iter, 1).get().unwrap();

                                            // If the text has changed we need to check that the new text is valid, as this is a key column.
                                            // Otherwise, we do nothing.
                                            if old_text != new_text && !new_text.is_empty() && !new_text.contains(' ') {

                                                // Get the first row's `TreeIter`.
                                                let current_line = packed_file_stuff.list_store.get_iter_first().unwrap();

                                                // Loop to search for coincidences.
                                                let mut key_already_exists = false;
                                                loop {

                                                    //  If we found a coincidence, break the loop.
                                                    if packed_file_stuff.list_store.get_value(&current_line, 1).get::<String>().unwrap() == new_text {
                                                        key_already_exists = true;
                                                        break;
                                                    }

                                                    // If we reached the end of the `ListStore`, we break the loop.
                                                    else if !packed_file_stuff.list_store.iter_next(&current_line) { break; }
                                                }

                                                // If there is a coincidence with another key...
                                                if key_already_exists {
                                                    ui::show_dialog(&app_ui.window, false, "This key is already in the Loc PackedFile.");
                                                }

                                                // If it has passed all the checks without error...
                                                else {

                                                    // Change the value in the cell.
                                                    packed_file_stuff.list_store.set_value(&tree_iter, 1, &new_text.to_value());

                                                    // Replace the old encoded data with the new one.
                                                    packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                    // Update the PackFile to reflect the changes.
                                                    update_packed_file_data_loc(
                                                        &*packed_file_data_decoded.borrow_mut(),
                                                        &mut *pack_file_decoded.borrow_mut(),
                                                        index as usize
                                                    );

                                                    // Set the mod as "Modified".
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                }
                                            }

                                            // If the field is empty,
                                            else if new_text.is_empty() {
                                                ui::show_dialog(&app_ui.window, false, "Only my hearth can be empty.");
                                            }

                                            // If the field contains spaces.
                                            else if new_text.contains(' ') {
                                                ui::show_dialog(&app_ui.window, false, "Spaces are not valid characters.");
                                            }
                                        }
                                    ));

                                    // When we change the text of the "Text" cell.
                                    packed_file_stuff.cell_text.connect_edited(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_, tree_path, new_text| {

                                            // Get the cell's old text, to check for changes.
                                            let tree_iter = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();
                                            let old_text: String = packed_file_stuff.list_store.get_value(&tree_iter, 2).get().unwrap();

                                            // If it has changed...
                                            if old_text != new_text {

                                                // Change the value in the cell.
                                                packed_file_stuff.list_store.set_value(&tree_iter, 2, &new_text.to_value());

                                                // Replace the old encoded data with the new one.
                                                packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                // Update the PackFile to reflect the changes.
                                                update_packed_file_data_loc(
                                                    &*packed_file_data_decoded.borrow_mut(),
                                                    &mut *pack_file_decoded.borrow_mut(),
                                                    index as usize
                                                );

                                                // Set the mod as "Modified".
                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                            }
                                        }
                                    ));

                                    // When we change the state (true/false) of the "Tooltip" cell.
                                    packed_file_stuff.cell_tooltip.connect_toggled(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |cell, tree_path|{

                                            // Get his `TreeIter` and his column.
                                            let tree_iter = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();
                                            let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                            // Get his new state.
                                            let state = !cell.get_active();

                                            // Change it in the `ListStore`.
                                            packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &state.to_value());

                                            // Change his state.
                                            cell.set_active(state);

                                            // Replace the old encoded data with the new one.
                                            packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                            // Update the PackFile to reflect the changes.
                                            update_packed_file_data_loc(
                                                &*packed_file_data_decoded.borrow_mut(),
                                                &mut *pack_file_decoded.borrow_mut(),
                                                index as usize
                                            );

                                            // Set the mod as "Modified".
                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        }
                                    ));

                                    // We check if we can delete something on selection changes.
                                    packed_file_stuff.tree_view.connect_cursor_changed(clone!(
                                        context_menu_packedfile_loc_copy_cell,
                                        context_menu_packedfile_loc_copy_rows,
                                        context_menu_packedfile_loc_delete_rows => move |tree_view| {

                                            // If we have something selected, enable these actions.
                                            if tree_view.get_selection().count_selected_rows() > 0 {
                                                context_menu_packedfile_loc_copy_cell.set_enabled(true);
                                                context_menu_packedfile_loc_copy_rows.set_enabled(true);
                                                context_menu_packedfile_loc_delete_rows.set_enabled(true);
                                            }

                                            // Otherwise, disable them.
                                            else {
                                                context_menu_packedfile_loc_copy_cell.set_enabled(false);
                                                context_menu_packedfile_loc_copy_rows.set_enabled(false);
                                                context_menu_packedfile_loc_delete_rows.set_enabled(false);
                                            }
                                        }
                                    ));

                                    // When we hit the "Add row" button.
                                    context_menu_packedfile_loc_add_rows.connect_activate(clone!(
                                        app_ui,
                                        packed_file_stuff => move |_,_| {

                                            // We hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // First, we check if the input is a valid number, as I'm already seeing people
                                                // trying to add "two" rows.
                                                match packed_file_stuff.add_rows_entry.get_buffer().get_text().parse::<u32>() {

                                                    // If the number is valid...
                                                    Ok(number_rows) => {

                                                        // For each new row we want...
                                                        for _ in 0..number_rows {

                                                            // Add a new empty line.
                                                            packed_file_stuff.list_store.insert_with_values(None, &[0, 1, 2, 3], &[&"New".to_value(), &"".to_value(), &"".to_value(), &true.to_value()]);
                                                        }
                                                    }
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, format!("You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows? But definetly not \"{}\".", Error::from(error).cause())),
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Delete row" button.
                                    context_menu_packedfile_loc_delete_rows.connect_activate(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // We hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the selected row's `TreePath`.
                                                let selected_rows = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                // If we have any row selected...
                                                if !selected_rows.is_empty() {

                                                    // For each row (in reverse)...
                                                    for row in (0..selected_rows.len()).rev() {

                                                        // Remove it.
                                                        packed_file_stuff.list_store.remove(&packed_file_stuff.list_store.get_iter(&selected_rows[row]).unwrap());
                                                    }

                                                    // Replace the old encoded data with the new one.
                                                    packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                    // Update the PackFile to reflect the changes.
                                                    update_packed_file_data_loc(
                                                        &*packed_file_data_decoded.borrow_mut(),
                                                        &mut *pack_file_decoded.borrow_mut(),
                                                        index as usize
                                                    );

                                                    // Set the mod as "Modified".
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Copy cell" button.
                                    context_menu_packedfile_loc_copy_cell.connect_activate(clone!(
                                        app_ui,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the the focused cell.
                                                let focused_cell = packed_file_stuff.tree_view.get_cursor();

                                                // If there is a focused `TreePath`...
                                                if let Some(tree_path) = focused_cell.0 {

                                                    // And a focused `TreeViewColumn`...
                                                    if let Some(column) = focused_cell.1 {

                                                        // If the cell is the index...
                                                        if column.get_sort_column_id() == 0 {

                                                            // Get his value and put it into the `Clipboard`.
                                                            app_ui.clipboard.set_text(&packed_file_stuff.list_store.get_value(&packed_file_stuff.list_store.get_iter(&tree_path).unwrap(), 0).get::<String>().unwrap(),);
                                                        }

                                                        // If we are trying to copy the "tooltip" column...
                                                        else if column.get_sort_column_id() == 3 {

                                                            // Get the state of the toggle into an `&str`.
                                                            let state = if packed_file_stuff.list_store.get_value(&packed_file_stuff.list_store.get_iter(&tree_path).unwrap(), 3).get().unwrap() { "true" } else { "false" };

                                                            // Put the state of the toggle into the `Clipboard`.
                                                            app_ui.clipboard.set_text(state);
                                                        }

                                                        // Otherwise...
                                                        else {

                                                            // Get the text from the focused cell and put it into the `Clipboard`.
                                                            app_ui.clipboard.set_text(
                                                                &packed_file_stuff.list_store.get_value(
                                                                    &packed_file_stuff.list_store.get_iter(&tree_path).unwrap(),
                                                                    column.get_sort_column_id(),
                                                                ).get::<&str>().unwrap()
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Paste cell" button.
                                    context_menu_packedfile_loc_paste_cell.connect_activate(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the the focused cell.
                                                let focused_cell = packed_file_stuff.tree_view.get_cursor();

                                                // If there is a focused `TreePath`...
                                                if let Some(tree_path) = focused_cell.0 {

                                                    // And a focused `TreeViewColumn`...
                                                    if let Some(column) = focused_cell.1 {

                                                        // If the cell is the index...
                                                        if column.get_sort_column_id() == 0 {

                                                            // Don't do anything.
                                                            return
                                                        }

                                                        // If we are trying to paste the "tooltip" column...
                                                        else if column.get_sort_column_id() == 3 {

                                                            // If we got the state of the toggle from the `Clipboard`...
                                                            if let Some(data) = app_ui.clipboard.wait_for_text() {

                                                                // Get the state of the toggle into an `&str`.
                                                                let state = if data == "true" { true } else if data == "false" { false } else { return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a Loc PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".") };

                                                                // Set the state of the toggle of the cell.
                                                                packed_file_stuff.list_store.set_value(&packed_file_stuff.list_store.get_iter(&tree_path).unwrap(), 3, &state.to_value());

                                                                // Replace the old encoded data with the new one.
                                                                packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                                // Update the PackFile to reflect the changes.
                                                                update_packed_file_data_loc(
                                                                    &*packed_file_data_decoded.borrow_mut(),
                                                                    &mut *pack_file_decoded.borrow_mut(),
                                                                    index as usize
                                                                );

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                            }
                                                        }

                                                        // Otherwise...
                                                        else {

                                                            // If we got the state of the toggle from the `Clipboard`...
                                                            if let Some(data) = app_ui.clipboard.wait_for_text() {

                                                                // Update his value.
                                                                packed_file_stuff.list_store.set_value(&packed_file_stuff.list_store.get_iter(&tree_path).unwrap(), column.get_sort_column_id() as u32, &data.to_value());

                                                                // Replace the old encoded data with the new one.
                                                                packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                                // Update the PackFile to reflect the changes.
                                                                update_packed_file_data_loc(
                                                                    &*packed_file_data_decoded.borrow_mut(),
                                                                    &mut *pack_file_decoded.borrow_mut(),
                                                                    index as usize
                                                                );

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Copy row" button.
                                    context_menu_packedfile_loc_copy_rows.connect_activate(clone!(
                                        app_ui,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the selected rows.
                                                let selected_rows = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                // If there is something selected...
                                                if !selected_rows.is_empty() {

                                                    // Get the list of `TreeIter`s we want to copy.
                                                    let tree_iter_list = selected_rows.iter().map(|row| packed_file_stuff.list_store.get_iter(row).unwrap()).collect::<Vec<TreeIter>>();

                                                    // Create the `String` that will copy the row that will bring that shit of TLJ down.
                                                    let mut copy_string = String::new();

                                                    // For each row...
                                                    for row in &tree_iter_list {

                                                        // Get the data from the three columns, and push it to our copy `String`. The format is:
                                                        // - Everything between "".
                                                        // - A comma between columns.
                                                        // - A \n at the end of the row.
                                                        copy_string.push_str("\"");
                                                        copy_string.push_str(packed_file_stuff.list_store.get_value(&row, 1).get::<&str>().unwrap());
                                                        copy_string.push_str("\",\"");
                                                        copy_string.push_str(packed_file_stuff.list_store.get_value(&row, 2).get::<&str>().unwrap());
                                                        copy_string.push_str("\",\"");
                                                        copy_string.push_str(
                                                            match packed_file_stuff.list_store.get_value(&row, 3).get::<bool>().unwrap() {
                                                                true => "true",
                                                                false => "false",
                                                            }
                                                        );
                                                        copy_string.push_str("\"\n");
                                                    }

                                                    // Pass all the copied rows to the clipboard.
                                                    app_ui.clipboard.set_text(&copy_string);
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Paste row" button.
                                    context_menu_packedfile_loc_paste_rows.connect_activate(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // When it gets the data from the `Clipboard`....
                                                if let Some(data) = app_ui.clipboard.wait_for_text() {

                                                    // Store here all the decoded fields.
                                                    let mut fields_data = vec![];

                                                    // Get the type of the data copied. If it's in CSV format...
                                                    if let Some(_) = data.find("\",\"") {

                                                        // For each row in the data we received...
                                                        for row in data.lines() {

                                                            // Remove the "" at the beginning and at the end.
                                                            let mut row = row.to_owned();
                                                            row.pop();
                                                            row.remove(0);

                                                            // Get all the data from his fields.
                                                            fields_data.push(row.split("\",\"").map(|x| x.to_owned()).collect::<Vec<String>>());
                                                        }
                                                    }

                                                    // Otherwise, we asume it's a TSV copy from excel.
                                                    // TODO: Check this with other possible sources.
                                                    else {

                                                        // For each row in the data we received...
                                                        for row in data.lines() {

                                                            // Get all the data from his fields.
                                                            fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
                                                        }
                                                    }

                                                    // Get the selected row, if there is any.
                                                    let selected_row = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                    // If there is at least one line selected, use it as "base" to paste.
                                                    let mut tree_iter = if !selected_row.is_empty() {
                                                        packed_file_stuff.list_store.get_iter(&selected_row[0]).unwrap()
                                                    }

                                                    // Otherwise, append a new `TreeIter` to the `TreeView`, and use it.
                                                    else { packed_file_stuff.list_store.append() };

                                                    // If we have enough fields in our data to fill the row...
                                                    if fields_data.len() >= 3 {

                                                        // For each row in our fields_data vec...
                                                        for (row_index, row) in fields_data.iter().enumerate() {

                                                            // Fill the "Index" column with "New".
                                                            packed_file_stuff.list_store.set_value(&tree_iter, 0, &"New".to_value());

                                                            // Fill the "key" and "text" columns.
                                                            packed_file_stuff.list_store.set_value(&tree_iter, 1, &row[0].to_value());
                                                            packed_file_stuff.list_store.set_value(&tree_iter, 2, &row[1].to_value());

                                                            // Fill the "tooltip" column too, with a little trick. Return an error in case of wrong format.
                                                            packed_file_stuff.list_store.set_value(&tree_iter, 3, &(
                                                                if row[2] == "true" { true }
                                                                else if row[2] == "false" { false }
                                                                else { return ui::show_dialog(&app_ui.window, false, format!("Error while trying to paste a row to a Loc PackedFile:\n\nThe third field of row {} is neither \"true\" nor \"false\".", row_index + 1)) }
                                                            ).to_value());

                                                            // Move to the next row. If it doesn't exist and it's not the last loop....
                                                            if !packed_file_stuff.list_store.iter_next(&tree_iter) && row_index < (fields_data.len() - 1) {

                                                                // Create it.
                                                                tree_iter = packed_file_stuff.list_store.append();
                                                            }
                                                        }

                                                        // Replace the old encoded data with the new one.
                                                        packed_file_data_decoded.borrow_mut().packed_file_data = PackedFileLocTreeView::return_data_from_tree_view(&packed_file_stuff.list_store);

                                                        // Update the PackFile to reflect the changes.
                                                        update_packed_file_data_loc(
                                                            &*packed_file_data_decoded.borrow_mut(),
                                                            &mut *pack_file_decoded.borrow_mut(),
                                                            index as usize
                                                        );

                                                        // Set the mod as "Modified".
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                    }
                                                };
                                            }
                                        }
                                    ));

                                    // When we hit the "Import to CSV" button.
                                    context_menu_packedfile_loc_import_csv.connect_activate(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_|{

                                            // We hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Create the `FileChooser`.
                                                let file_chooser = FileChooserNative::new(
                                                    "Select CSV File to Import...",
                                                    &app_ui.window,
                                                    FileChooserAction::Open,
                                                    "Import",
                                                    "Cancel"
                                                );

                                                // Enable the CSV filter for the `FileChooser`.
                                                file_chooser_filter_packfile(&file_chooser, "*.csv");

                                                // If we have selected a file to import...
                                                if file_chooser.run() == gtk_response_accept {

                                                    // If there is an error while importing the CSV file, we report it.
                                                    if let Err(error) = LocData::import_csv(
                                                        &mut packed_file_data_decoded.borrow_mut().packed_file_data,
                                                        &file_chooser.get_filename().unwrap()
                                                    ) {
                                                        ui::show_dialog(&app_ui.window, false, error.cause());
                                                    }

                                                    // Otherwise...
                                                    else {

                                                        // Load the new data to the TreeView.
                                                        PackedFileLocTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_stuff.list_store);

                                                        // Update the PackFile to reflect the changes.
                                                        update_packed_file_data_loc(
                                                            &*packed_file_data_decoded.borrow_mut(),
                                                            &mut *pack_file_decoded.borrow_mut(),
                                                            index as usize
                                                        );

                                                        // Set the mod as "Modified".
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Export to CSV" button.
                                    context_menu_packedfile_loc_export_csv.connect_activate(clone!(
                                        app_ui,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_|{

                                            // We hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Create the `FileChooser`.
                                                let file_chooser = FileChooserNative::new(
                                                    "Export CSV File...",
                                                    &app_ui.window,
                                                    FileChooserAction::Save,
                                                    "Save",
                                                    "Cancel"
                                                );

                                                // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                                                file_chooser.set_do_overwrite_confirmation(true);

                                                // Set the name of the Loc PackedFile as the default new name.
                                                file_chooser.set_current_name(format!("{}.csv", &packedfile_name));

                                                // If we hit "Save"...
                                                if file_chooser.run() == gtk_response_accept {

                                                    // Try to export the CSV.
                                                    match LocData::export_csv(&packed_file_data_decoded.borrow_mut().packed_file_data, &file_chooser.get_filename().unwrap()) {
                                                        Ok(result) => ui::show_dialog(&app_ui.window, true, result),
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause())
                                                    }
                                                }
                                            }
                                        }
                                    ));
                                }
                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }

                        // If the file is a DB PackedFile...
                        "DB" => {

                            let packed_file_data_encoded = &(pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data);
                            let packed_file_data_decoded = match *schema.borrow() {
                                Some(ref schema) => DB::read(&packed_file_data_encoded, &*tree_path[1], &schema),
                                None => return ui::show_dialog(&app_ui.window, false, "There is no Schema loaded for this game."),
                            };

                            // We create the button to enable the "Decoding" mode.
                            let decode_mode_button = Button::new_with_label("Enter decoding mode");
                            decode_mode_button.set_hexpand(true);
                            app_ui.packed_file_data_display.attach(&decode_mode_button, 0, 0, 1, 1);
                            app_ui.packed_file_data_display.show_all();

                            // From here, we deal we the decoder stuff.
                            decode_mode_button.connect_button_release_event(clone!(
                                application,
                                schema,
                                tree_path,
                                rpfm_path,
                                app_ui,
                                packed_file_data_encoded,
                                pack_file_decoded => move |decode_mode_button ,_| {

                                    // We need to disable the button. Otherwise, things will get weird.
                                    decode_mode_button.set_sensitive(false);

                                    // We destroy the table view if exists, so we don't have to deal with resizing it.
                                    let display_last_children = app_ui.packed_file_data_display.get_children();
                                    if display_last_children.first().unwrap() != decode_mode_button {
                                        display_last_children.first().unwrap().destroy();
                                    }

                                    // Then create the UI..
                                    let mut packed_file_decoder = PackedFileDBDecoder::create_decoder_view(&app_ui.packed_file_data_display);

                                    // And only in case the db_header has been decoded, we do the rest.
                                    match DBHeader::read(&packed_file_data_encoded){
                                        Ok(db_header) => {

                                            // We get the initial index to start decoding.
                                            let initial_index = db_header.1;

                                            // We get the Schema for his game, if exists. If we reached this point, the Schema
                                            // should exists. Otherwise, the button for this window will not exist.
                                            let table_definition = match DB::get_schema(&tree_path[1], db_header.0.packed_file_header_packed_file_version, &schema.borrow().clone().unwrap()) {
                                                Some(table_definition) => Rc::new(RefCell::new(table_definition)),
                                                None => Rc::new(RefCell::new(TableDefinition::new(db_header.0.packed_file_header_packed_file_version)))
                                            };

                                            // We try to load the static data from the encoded PackedFile into the "Decoder" view.
                                            match PackedFileDBDecoder::load_data_to_decoder_view(
                                                &mut packed_file_decoder,
                                                &*tree_path[1],
                                                &packed_file_data_encoded,
                                                initial_index
                                            ) {

                                                // If we succeed...
                                                Ok(_) => {

                                                    // Update the "Decoder" View dinamyc data (entries, treeview,...) and get the
                                                    // current "index_data" (position in the vector we are decoding).
                                                    let index_data = Rc::new(RefCell::new(PackedFileDBDecoder::update_decoder_view(
                                                        &packed_file_decoder,
                                                        &packed_file_data_encoded,
                                                        Some(&table_definition.borrow()),
                                                        initial_index,
                                                    )));

                                                    // Update the versions list. Only if we have an schema, we can reach this point, so we just unwrap the schema.
                                                    PackedFileDBDecoder::update_versions_list(&packed_file_decoder, &schema.borrow().clone().unwrap(), &*tree_path[1]);

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

                                                    // By default, these buttons are disabled.
                                                    packed_file_decoder.all_table_versions_remove_definition.set_sensitive(false);
                                                    packed_file_decoder.all_table_versions_load_definition.set_sensitive(false);

                                                    // We check if we can allow actions on selection changes.
                                                    packed_file_decoder.fields_tree_view.connect_cursor_changed(clone!(
                                                        decoder_move_row_up,
                                                        decoder_move_row_down,
                                                        decoder_delete_row => move |tree_view| {

                                                            // If nothing is selected, disable all the actions.
                                                            if tree_view.get_selection().count_selected_rows() > 0 {
                                                                decoder_move_row_up.set_enabled(true);
                                                                decoder_move_row_down.set_enabled(true);
                                                                decoder_delete_row.set_enabled(true);
                                                            }

                                                            // Otherwise, enable them.
                                                            else {
                                                                decoder_move_row_up.set_enabled(false);
                                                                decoder_move_row_down.set_enabled(false);
                                                                decoder_delete_row.set_enabled(false);
                                                            }
                                                        }
                                                    ));

                                                    // We check if we can allow actions on selection changes.
                                                    packed_file_decoder.all_table_versions_tree_view.connect_cursor_changed(clone!(
                                                        packed_file_decoder => move |tree_view| {

                                                            // If nothing is selected, enable all the actions.
                                                            if tree_view.get_selection().count_selected_rows() > 0 {
                                                                packed_file_decoder.all_table_versions_remove_definition.set_sensitive(true);
                                                                packed_file_decoder.all_table_versions_load_definition.set_sensitive(true);
                                                            }

                                                            // Otherwise, disable them.
                                                            else {
                                                                packed_file_decoder.all_table_versions_remove_definition.set_sensitive(false);
                                                                packed_file_decoder.all_table_versions_load_definition.set_sensitive(false);
                                                            }
                                                        }
                                                    ));

                                                    // When we press the "Move up" button.
                                                    decoder_move_row_up.connect_activate(clone!(
                                                        initial_index,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_,_| {

                                                            // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                                                            // the accels working everywhere.
                                                            if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_up_button.has_focus() {

                                                                // Get the current iter.
                                                                let current_iter = packed_file_decoder.fields_tree_view.get_selection().get_selected().unwrap().1;
                                                                let new_iter = current_iter.clone();

                                                                // If there is a previous iter, swap them.
                                                                if packed_file_decoder.fields_list_store.iter_previous(&new_iter) {
                                                                    packed_file_decoder.fields_list_store.move_before(&current_iter, &new_iter);

                                                                    // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                                                                    *index_data.borrow_mut() = update_first_row_decoded(
                                                                        &packed_file_data_encoded,
                                                                        &packed_file_decoder.fields_list_store,
                                                                        &initial_index,
                                                                        &packed_file_decoder
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    ));

                                                    // When we press the "Move down" button.
                                                    decoder_move_row_down.connect_activate(clone!(
                                                        initial_index,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_,_| {

                                                            // We only do something in case the focus is in the TreeView or in it's button. This should stop problems with
                                                            // the accels working everywhere.
                                                            if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_down_button.has_focus() {

                                                                // Get the current iter.
                                                                let current_iter = packed_file_decoder.fields_tree_view.get_selection().get_selected().unwrap().1;
                                                                let new_iter = current_iter.clone();

                                                                // If there is a next iter, swap them.
                                                                if packed_file_decoder.fields_list_store.iter_next(&new_iter) {
                                                                    packed_file_decoder.fields_list_store.move_after(&current_iter, &new_iter);

                                                                    // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                                                                    *index_data.borrow_mut() = update_first_row_decoded(
                                                                        &packed_file_data_encoded,
                                                                        &packed_file_decoder.fields_list_store,
                                                                        &initial_index,
                                                                        &packed_file_decoder
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    ));

                                                    // This allow us to remove a field from the list, using the decoder_delete_row action.
                                                    decoder_delete_row.connect_activate(clone!(
                                                        initial_index,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_,_| {

                                                            // We only do something in case the focus is in the TreeView or in any of the moving buttons. This should stop problems with
                                                            // the accels working everywhere.
                                                            if packed_file_decoder.fields_tree_view.has_focus() || packed_file_decoder.move_up_button.has_focus() || packed_file_decoder.move_down_button.has_focus() {

                                                                // If there is something selected, delete it.
                                                                if let Some(selection) = packed_file_decoder.fields_tree_view.get_selection().get_selected() {
                                                                    packed_file_decoder.fields_list_store.remove(&selection.1);
                                                                }

                                                                // Update the "First row decoded" column, and get the new "index_data" to continue decoding.
                                                                *index_data.borrow_mut() = update_first_row_decoded(
                                                                    &packed_file_data_encoded,
                                                                    &packed_file_decoder.fields_list_store,
                                                                    &initial_index,
                                                                    &packed_file_decoder
                                                                );
                                                            }
                                                        }
                                                    ));

                                                    // Logic for all the "Use this" buttons. Basically, they just check if it's possible to use their decoder
                                                    // for the bytes we have, and advance the index and add their type to the fields view.

                                                    // When we hit the "Use this" button for boolean fields.
                                                    packed_file_decoder.use_bool_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::Boolean,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for float fields.
                                                    packed_file_decoder.use_float_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::Float,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for integer fields.
                                                    packed_file_decoder.use_integer_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::Integer,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for long integer fields.
                                                    packed_file_decoder.use_long_integer_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::LongInteger,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for string U8 fields.
                                                    packed_file_decoder.use_string_u8_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::StringU8,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for string u16 fields.
                                                    packed_file_decoder.use_string_u16_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::StringU16,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for optional string u8 fields.
                                                    packed_file_decoder.use_optional_string_u8_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::OptionalStringU8,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we hit the "Use this" button for optional string u16 fields.
                                                    packed_file_decoder.use_optional_string_u16_button.connect_button_release_event(clone!(
                                                        table_definition,
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_|{

                                                            // Get a copy of our current index.
                                                            let index_data_copy = index_data.borrow().clone();

                                                            // Add the field to the table, update it, and get the new "index_data".
                                                            *index_data.borrow_mut() = PackedFileDBDecoder::use_this(
                                                                &packed_file_decoder,
                                                                &table_definition,
                                                                index_data_copy,
                                                                &packed_file_data_encoded,
                                                                FieldType::OptionalStringU16,
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we press the "Delete all fields" button.
                                                    packed_file_decoder.delete_all_fields_button.connect_button_release_event(clone!(
                                                        index_data,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |delete_all_fields_button,_| {

                                                            // Clear the `TreeView`.
                                                            packed_file_decoder.fields_list_store.clear();

                                                            // Reset the "index_data".
                                                            *index_data.borrow_mut() = initial_index;

                                                            // Disable this button.
                                                            delete_all_fields_button.set_sensitive(false);

                                                            // Re-update the "Decoder" View.
                                                            PackedFileDBDecoder::update_decoder_view(
                                                                &packed_file_decoder,
                                                                &packed_file_data_encoded,
                                                                None,
                                                                *index_data.borrow(),
                                                            );

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // This allow us to replace the definition we have loaded with one from another version of the table.
                                                    packed_file_decoder.all_table_versions_load_definition.connect_button_release_event(clone!(
                                                        schema,
                                                        tree_path,
                                                        app_ui,
                                                        packed_file_data_encoded,
                                                        packed_file_decoder => move |_ ,_| {

                                                            // Only if we have a version selected, do something.
                                                            if let Some(version_selected) = packed_file_decoder.all_table_versions_tree_view.get_selection().get_selected() {

                                                                // Get the version selected.
                                                                let version_to_load: u32 = packed_file_decoder.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                                                                // Check if the Schema actually exists. This should never show up if the schema exists,
                                                                // but the compiler doesn't know it, so we have to check it.
                                                                match *schema.borrow_mut() {
                                                                    Some(ref mut schema) => {

                                                                        // Get the new definition.
                                                                        let table_definition = DB::get_schema(&tree_path[1], version_to_load, schema);

                                                                        // Remove all the fields of the currently loaded definition.
                                                                        packed_file_decoder.fields_list_store.clear();

                                                                        // Reload the decoder View with the new definition loaded.
                                                                        PackedFileDBDecoder::update_decoder_view(
                                                                            &packed_file_decoder,
                                                                            &packed_file_data_encoded,
                                                                            table_definition.as_ref(),
                                                                            initial_index,
                                                                        );
                                                                    }
                                                                    None => ui::show_dialog(&app_ui.window, false, "Cannot load a version of a table from a non-existant Schema.")
                                                                }
                                                            }

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // This allow us to remove an entire definition of a table for an specific version.
                                                    // Basically, hitting this button deletes the selected definition.
                                                    packed_file_decoder.all_table_versions_remove_definition.connect_button_release_event(clone!(
                                                        schema,
                                                        tree_path,
                                                        app_ui,
                                                        packed_file_decoder => move |_ ,_| {

                                                            // Only if we have a version selected, do something.
                                                            if let Some(version_selected) = packed_file_decoder.all_table_versions_tree_view.get_selection().get_selected() {

                                                                // Get the version selected.
                                                                let version_to_delete: u32 = packed_file_decoder.all_table_versions_list_store.get_value(&version_selected.1, 0).get().unwrap();

                                                                // Check if the Schema actually exists. This should never show up if the schema exists,
                                                                // but the compiler doesn't know it, so we have to check it.
                                                                match *schema.borrow_mut() {
                                                                    Some(ref mut schema) => {

                                                                        // Try to remove that version form the schema.
                                                                        match DB::remove_table_version(&tree_path[1], version_to_delete, schema) {

                                                                            // If it worked, update the list.
                                                                            Ok(_) => PackedFileDBDecoder::update_versions_list(&packed_file_decoder, schema, &tree_path[1]),
                                                                            Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                                        }
                                                                    }
                                                                    None => ui::show_dialog(&app_ui.window, false, "Cannot delete a version from a non-existant Schema.")
                                                                }
                                                            }

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // When we press the "Finish it!" button.
                                                    packed_file_decoder.save_decoded_schema.connect_button_release_event(clone!(
                                                        app_ui,
                                                        schema,
                                                        table_definition,
                                                        tree_path,
                                                        rpfm_path,
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

                                                                    // If we didn't found a table definition for our table...
                                                                    if table_definitions_index == -1 {

                                                                        // We create one.
                                                                        schema.add_table_definitions(TableDefinitions::new(&packed_file_decoder.table_type_label.get_text().unwrap()));

                                                                        // And get his index.
                                                                        table_definitions_index = schema.get_table_definitions(&*tree_path[1]).unwrap() as i32;
                                                                    }

                                                                    // We replace his fields with the ones from the `TreeView`.
                                                                    table_definition.borrow_mut().fields = packed_file_decoder.return_data_from_data_view();

                                                                    // We add our `TableDefinition` to the main `Schema`.
                                                                    schema.tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());

                                                                    // And try to save the main `Schema`.
                                                                    match Schema::save(&schema, &rpfm_path, &*pack_file_decoded.borrow().pack_file_header.pack_file_id) {
                                                                        Ok(_) => ui::show_dialog(&app_ui.window, true, "Schema successfully saved."),
                                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                                    }

                                                                    // After all that, we need to update the version list, as this may have created a new version.
                                                                    PackedFileDBDecoder::update_versions_list(&packed_file_decoder, schema, &*tree_path[1]);
                                                                }
                                                                None => ui::show_dialog(&app_ui.window, false, "Cannot save this table's definitions:\nSchemas for this game are not supported, yet.")
                                                            }

                                                            Inhibit(false)
                                                        }
                                                    ));

                                                    // This allow us to change a field's data type in the TreeView.
                                                    packed_file_decoder.fields_tree_view_cell_combo.connect_edited(clone!(
                                                        packed_file_decoder => move |_, tree_path, new_value| {

                                                            // Get his iter and change it. Not to hard.
                                                            let tree_iter = &packed_file_decoder.fields_list_store.get_iter(&tree_path).unwrap();
                                                            packed_file_decoder.fields_list_store.set_value(tree_iter, 2, &new_value.to_value());
                                                        }
                                                    ));

                                                    // This allow us to set as "key" a field in the TreeView.
                                                    packed_file_decoder.fields_tree_view_cell_bool.connect_toggled(clone!(
                                                        packed_file_decoder => move |cell, tree_path| {

                                                            // Get his `TreeIter`.
                                                            let tree_iter = packed_file_decoder.fields_list_store.get_iter(&tree_path).unwrap();

                                                            // Get his new state.
                                                            let state = !cell.get_active();

                                                            // Change it in the `ListStore`.
                                                            packed_file_decoder.fields_list_store.set_value(&tree_iter, 3, &state.to_value());

                                                            // Change his state.
                                                            cell.set_active(state);
                                                        }
                                                    ));

                                                    // This loop takes care of the interaction with string cells.
                                                    for edited_cell in &packed_file_decoder.fields_tree_view_cell_string {
                                                        edited_cell.connect_edited(clone!(
                                                            packed_file_decoder => move |_ ,tree_path , new_text| {

                                                                // Get his iter.
                                                                let tree_iter = packed_file_decoder.fields_list_store.get_iter(&tree_path).unwrap();

                                                                // Get his column.
                                                                let edited_cell_column = packed_file_decoder.fields_tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                                // Set his new value.
                                                                packed_file_decoder.fields_list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());
                                                            }
                                                        ));
                                                    }
                                                }
                                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                            }
                                        },
                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                    }
                                    Inhibit(false)
                                }
                            ));

                            // If this returns an error, we just leave the button for the decoder.
                            match packed_file_data_decoded {
                                Ok(packed_file_data_decoded) => {

                                    // Try to open the dependency PackFile of our game.
                                    let dependency_database = match packfile::open_packfile(game_selected.borrow().game_dependency_packfile_path.to_path_buf()) {
                                        Ok(data) => Some(data.pack_file_data.packed_files),
                                        Err(_) => None,
                                    };

                                    // Get the decoded PackedFile in a `Rc<RefCell<>>` so we can pass it to the closures.
                                    let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));

                                    // Get a reference to the `TableDefinition` of our table.
                                    let table_definition = &(packed_file_data_decoded.borrow().packed_file_data.table_definition);

                                    // Try to create the `TreeView`.
                                    let packed_file_stuff = match ui::packedfile_db::PackedFileDBTreeView::create_tree_view(
                                        &app_ui.packed_file_data_display,
                                        &*packed_file_data_decoded.borrow(),
                                        dependency_database,
                                        &pack_file_decoded.borrow().pack_file_data.packed_files,
                                        &schema.borrow().clone().unwrap(),
                                        &settings.borrow(),
                                    ) {
                                        Ok(data) => data,
                                        Err(error) => return ui::show_dialog(&app_ui.window, false, error.cause())
                                    };

                                    // We enable "Multiple" selection mode, so we can do multi-row operations.
                                    packed_file_stuff.tree_view.get_selection().set_mode(gtk::SelectionMode::Multiple);

                                    // Try to load the data from the table to the `TreeView`.
                                    if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view (
                                        &packed_file_data_decoded.borrow().packed_file_data,
                                        &packed_file_stuff.list_store
                                    ) {
                                        return ui::show_dialog(&app_ui.window, false, error.cause());
                                    }

                                    // Before setting up the actions, we clean the previous ones.
                                    remove_temporal_accelerators(&application);

                                    // Right-click menu actions.
                                    let context_menu_packedfile_db_add_rows = SimpleAction::new("packedfile_db_add_rows", None);
                                    let context_menu_packedfile_db_delete_rows = SimpleAction::new("packedfile_db_delete_rows", None);
                                    let context_menu_packedfile_db_copy_cell = SimpleAction::new("packedfile_db_copy_cell", None);
                                    let context_menu_packedfile_db_paste_cell = SimpleAction::new("packedfile_db_paste_cell", None);
                                    let context_menu_packedfile_db_clone_rows = SimpleAction::new("packedfile_db_clone_rows", None);
                                    let context_menu_packedfile_db_copy_rows = SimpleAction::new("packedfile_db_copy_rows", None);
                                    let context_menu_packedfile_db_paste_rows = SimpleAction::new("packedfile_db_paste_rows", None);
                                    let context_menu_packedfile_db_import_csv = SimpleAction::new("packedfile_db_import_csv", None);
                                    let context_menu_packedfile_db_export_csv = SimpleAction::new("packedfile_db_export_csv", None);

                                    application.add_action(&context_menu_packedfile_db_add_rows);
                                    application.add_action(&context_menu_packedfile_db_delete_rows);
                                    application.add_action(&context_menu_packedfile_db_copy_cell);
                                    application.add_action(&context_menu_packedfile_db_paste_cell);
                                    application.add_action(&context_menu_packedfile_db_clone_rows);
                                    application.add_action(&context_menu_packedfile_db_copy_rows);
                                    application.add_action(&context_menu_packedfile_db_paste_rows);
                                    application.add_action(&context_menu_packedfile_db_import_csv);
                                    application.add_action(&context_menu_packedfile_db_export_csv);

                                    // Accels for popovers need to be specified here. Don't know why, but otherwise they do not work.
                                    application.set_accels_for_action("app.packedfile_db_add_rows", &["<Primary><Shift>a"]);
                                    application.set_accels_for_action("app.packedfile_db_delete_rows", &["<Shift>Delete"]);
                                    application.set_accels_for_action("app.packedfile_db_copy_cell", &["<Primary>c"]);
                                    application.set_accels_for_action("app.packedfile_db_paste_cell", &["<Primary>v"]);
                                    application.set_accels_for_action("app.packedfile_db_clone_rows", &["<Primary><Shift>d"]);
                                    application.set_accels_for_action("app.packedfile_db_copy_rows", &["<Primary>z"]);
                                    application.set_accels_for_action("app.packedfile_db_paste_rows", &["<Primary>x"]);
                                    application.set_accels_for_action("app.packedfile_db_import_csv", &["<Primary><Shift>i"]);
                                    application.set_accels_for_action("app.packedfile_db_export_csv", &["<Primary><Shift>e"]);

                                    // When a tooltip gets triggered...
                                    packed_file_stuff.tree_view.connect_query_tooltip(clone!(
                                        table_definition => move |tree_view, x, y,_, tooltip| {

                                            // Get the coordinates of the cell under the cursor.
                                            let cell_coords: (i32, i32) = tree_view.convert_widget_to_tree_coords(x, y);

                                            // If we got a column...
                                            if let Some(position) = tree_view.get_path_at_pos(cell_coords.0, cell_coords.1) {
                                                if let Some(column) = position.1 {

                                                    // Get his ID.
                                                    let column = column.get_sort_column_id();

                                                    // We don't want to check the tooltip for the Index column, nor for the fake end column.
                                                    if column >= 1 && (column as usize) <= table_definition.fields.len() {

                                                        // If it's a reference, we put to what cell is referencing in the tooltip.
                                                        let tooltip_text: String =

                                                            if let Some(ref reference) = table_definition.fields[column as usize - 1].field_is_reference {
                                                                if !table_definition.fields[column as usize - 1].field_description.is_empty() {
                                                                    format!("{}\n\nThis column is a reference to \"{}/{}\".",
                                                                        table_definition.fields[column as usize - 1].field_description,
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

                                                            }

                                                            // Otherwise, use the text from the description of that field.
                                                            else { table_definition.fields[column as usize - 1].field_description.to_owned() };

                                                        // If we got text to display, use it.
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
                                    for edited_cell in &packed_file_stuff.list_cell_reference {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text| {

                                                // If we got a cell...
                                                if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                    // Get his column.
                                                    let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                    // Change his value in the `TreeView`.
                                                    packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with string cells.
                                    for edited_cell in &packed_file_stuff.list_cell_string {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text| {

                                                // If we got a cell...
                                                if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                    // Get his column.
                                                    let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                    // Change his value in the `TreeView`.
                                                    packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with optional_string cells.
                                    for edited_cell in &packed_file_stuff.list_cell_optional_string {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text|{

                                                // If we got a cell...
                                                if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                    // Get his column.
                                                    let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                    // Change his value in the `TreeView`.
                                                    packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &new_text.to_value());

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with I32 cells.
                                    for edited_cell in &packed_file_stuff.list_cell_integer {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text|{

                                                // Check if what we got is a valid i32 number.
                                                match new_text.parse::<i32>() {

                                                    // If it's a valid i32 number...
                                                    Ok(new_number) => {

                                                        // If we got a cell...
                                                        if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                            // Get his column.
                                                            let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                            // Change his value in the `TreeView`.
                                                            packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &new_number.to_value());

                                                            // Try to save the new data from the `TreeView`.
                                                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                                // If we succeed...
                                                                Ok(data) => {

                                                                    // Replace our current decoded data with the new one.
                                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                                                    if let Err(error) = update_packed_file_data_db(
                                                                        &*packed_file_data_decoded.borrow_mut(),
                                                                        &mut *pack_file_decoded.borrow_mut(),
                                                                        index as usize
                                                                    ) {
                                                                        ui::show_dialog(&app_ui.window, false, error.cause());
                                                                    }

                                                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                }

                                                                // If there is an error, report it.
                                                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                            }
                                                        }
                                                    }

                                                    // If it isn't a valid i32 number, report it.
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, Error::from(error).cause()),
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with I64 cells.
                                    for edited_cell in &packed_file_stuff.list_cell_long_integer {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text|{

                                                // Check if what we got is a valid i64 number.
                                                match new_text.parse::<i64>() {

                                                    // If it's a valid i64 number...
                                                    Ok(new_number) => {

                                                        // If we got a cell...
                                                        if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                            // Get his column.
                                                            let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                            // Change his value in the `TreeView`.
                                                            packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &new_number.to_value());

                                                            // Try to save the new data from the `TreeView`.
                                                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                                // If we succeed...
                                                                Ok(data) => {

                                                                    // Replace our current decoded data with the new one.
                                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                                                    if let Err(error) = update_packed_file_data_db(
                                                                        &*packed_file_data_decoded.borrow_mut(),
                                                                        &mut *pack_file_decoded.borrow_mut(),
                                                                        index as usize
                                                                    ) {
                                                                        ui::show_dialog(&app_ui.window, false, error.cause());
                                                                    }

                                                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                }

                                                                // If there is an error, report it.
                                                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                            }
                                                        }
                                                    }

                                                    // If it isn't a valid i32 number, report it.
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, Error::from(error).cause()),
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with F32 cells.
                                    for edited_cell in &packed_file_stuff.list_cell_float {
                                        edited_cell.connect_edited(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |_ ,tree_path , new_text|{

                                                // Check if what we got is a valid f32 number.
                                                match new_text.parse::<f32>() {

                                                    // If it's a valid f32 number...
                                                    Ok(new_number) => {

                                                        // If we got a cell...
                                                        if let Some(tree_iter) = packed_file_stuff.list_store.get_iter(&tree_path) {

                                                            // Get his column.
                                                            let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                            // Change his value in the `TreeView`.
                                                            packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &format!("{}", new_number).to_value());

                                                            // Try to save the new data from the `TreeView`.
                                                            match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                                // If we succeed...
                                                                Ok(data) => {

                                                                    // Replace our current decoded data with the new one.
                                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                                    // Try to save the changes to the PackFile. If there is an error, report it.
                                                                    if let Err(error) = update_packed_file_data_db(
                                                                        &*packed_file_data_decoded.borrow_mut(),
                                                                        &mut *pack_file_decoded.borrow_mut(),
                                                                        index as usize
                                                                    ) {
                                                                        ui::show_dialog(&app_ui.window, false, error.cause());
                                                                    }

                                                                    // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                }

                                                                // If there is an error, report it.
                                                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                            }
                                                        }
                                                    }

                                                    // If it isn't a valid i32 number, report it.
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, Error::from(error).cause()),
                                                }
                                            }
                                        ));
                                    }

                                    // This loop takes care of the interaction with bool cells.
                                    for edited_cell in &packed_file_stuff.list_cell_bool {
                                        edited_cell.connect_toggled(clone!(
                                            table_definition,
                                            app_ui,
                                            pack_file_decoded,
                                            packed_file_data_decoded,
                                            packed_file_stuff => move |cell, tree_path| {

                                                // Get his `TreeIter` and his column.
                                                let tree_iter = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();
                                                let edited_cell_column = packed_file_stuff.tree_view.get_cursor().1.unwrap().get_sort_column_id() as u32;

                                                // Get his new state.
                                                let state = !cell.get_active();

                                                // Change it in the `ListStore`.
                                                packed_file_stuff.list_store.set_value(&tree_iter, edited_cell_column, &state.to_value());

                                                // Change his state.
                                                cell.set_active(state);

                                                // Try to save the new data from the `TreeView`.
                                                match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                    // If we succeed...
                                                    Ok(data) => {

                                                        // Replace our current decoded data with the new one.
                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                        // Try to save the changes to the PackFile. If there is an error, report it.
                                                        if let Err(error) = update_packed_file_data_db(
                                                            &*packed_file_data_decoded.borrow_mut(),
                                                            &mut *pack_file_decoded.borrow_mut(),
                                                            index as usize
                                                        ) {
                                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                                        }

                                                        // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                    }

                                                    // If there is an error, report it.
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                }
                                            }
                                        ));
                                    }

                                    // When we right-click the TreeView, we check if we need to enable or disable his buttons first.
                                    // Then we calculate the position where the popup must aim, and show it.
                                    //
                                    // NOTE: REMEMBER, WE OPEN THE POPUP HERE, BUT WE NEED TO CLOSED IT WHEN WE HIT HIS BUTTONS.
                                    packed_file_stuff.tree_view.connect_button_release_event(clone!(
                                        packed_file_stuff => move |tree_view, button| {

                                            // If we clicked the right mouse button...
                                            if button.get_button() == 3 {

                                                packed_file_stuff.context_menu.set_pointing_to(&get_rect_for_popover(tree_view, Some(button.get_position())));
                                                packed_file_stuff.context_menu.popup();
                                            }

                                            Inhibit(false)
                                        }
                                    ));

                                    // We check if we can delete something on selection changes.
                                    packed_file_stuff.tree_view.connect_cursor_changed(clone!(
                                        context_menu_packedfile_db_copy_cell,
                                        context_menu_packedfile_db_copy_rows,
                                        context_menu_packedfile_db_clone_rows,
                                        context_menu_packedfile_db_delete_rows => move |tree_view| {

                                            // If we have something selected...
                                            if tree_view.get_selection().count_selected_rows() > 0 {

                                                // Allow to delete, clone and copy.
                                                context_menu_packedfile_db_copy_cell.set_enabled(true);
                                                context_menu_packedfile_db_copy_rows.set_enabled(true);
                                                context_menu_packedfile_db_clone_rows.set_enabled(true);
                                                context_menu_packedfile_db_delete_rows.set_enabled(true);
                                            }

                                            // Otherwise, disable them.
                                            else {
                                                context_menu_packedfile_db_copy_cell.set_enabled(false);
                                                context_menu_packedfile_db_copy_rows.set_enabled(false);
                                                context_menu_packedfile_db_clone_rows.set_enabled(false);
                                                context_menu_packedfile_db_delete_rows.set_enabled(false);
                                            }
                                        }
                                    ));

                                    // When we hit the "Add row" button.
                                    context_menu_packedfile_db_add_rows.connect_activate(clone!(
                                        table_definition,
                                        app_ui,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // First, we check if the input is a valid number, as I'm already seeing people
                                                // trying to add "two" rows.
                                                match packed_file_stuff.add_rows_entry.get_buffer().get_text().parse::<u32>() {

                                                    // If the number is valid...
                                                    Ok(number_rows) => {

                                                        // For each row...
                                                        for _ in 0..number_rows {

                                                            // Add an empty row at the end of the `TreeView`, filling his index.
                                                            let new_row = packed_file_stuff.list_store.append();
                                                            packed_file_stuff.list_store.set_value(&new_row, 0, &"New".to_value());

                                                            // For each column we have...
                                                            for column in 1..(table_definition.fields.len() + 1) {

                                                                match table_definition.fields[column - 1].field_type {
                                                                    FieldType::Boolean => packed_file_stuff.list_store.set_value(&new_row, column as u32, &false.to_value()),
                                                                    FieldType::Float => packed_file_stuff.list_store.set_value(&new_row, column as u32, &0.0f32.to_string().to_value()),
                                                                    FieldType::Integer | FieldType::LongInteger => packed_file_stuff.list_store.set_value(&new_row, column as u32, &0.to_value()),
                                                                    FieldType::StringU8 | FieldType::StringU16 | FieldType::OptionalStringU8 | FieldType::OptionalStringU16 => {
                                                                        packed_file_stuff.list_store.set_value(&new_row, column as u32, &String::new().to_value());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // If it's not a valid number, report it.
                                                    Err(_) => ui::show_dialog(&app_ui.window, false, "You can only add an \"ENTIRE NUMBER\" of rows. Like 4, or 6. Maybe 5, who knows?"),
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Delete row" button.
                                    context_menu_packedfile_db_delete_rows.connect_activate(clone!(
                                        table_definition,
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the selected row's `TreePath`.
                                                let selected_rows = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                // If we have any row selected...
                                                if !selected_rows.is_empty() {

                                                    // For each row (in reverse)...
                                                    for row in (0..selected_rows.len()).rev() {

                                                        // Remove it.
                                                        packed_file_stuff.list_store.remove(&packed_file_stuff.list_store.get_iter(&selected_rows[row]).unwrap());
                                                    }

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Copy cell" button.
                                    context_menu_packedfile_db_copy_cell.connect_activate(clone!(
                                        app_ui,
                                        table_definition,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the the focused cell.
                                                let focused_cell = packed_file_stuff.tree_view.get_cursor();

                                                // If there is a focused `TreePath`...
                                                if let Some(tree_path) = focused_cell.0 {

                                                    // And a focused `TreeViewColumn`...
                                                    if let Some(column) = focused_cell.1 {

                                                        // Get his `TreeIter`.
                                                        let row = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();

                                                        // Get his column ID.
                                                        let column = column.get_sort_column_id();

                                                        // If the cell is the index...
                                                        if column == 0 {

                                                            // Get his value and put it into the `Clipboard`.
                                                            app_ui.clipboard.set_text(&packed_file_stuff.list_store.get_value(&row, 0).get::<String>().unwrap(),);
                                                        }

                                                        // Otherwise...
                                                        else {

                                                            // Check his `field_type`...
                                                            let data = match table_definition.fields[column as usize - 1].field_type {

                                                                // If it's a boolean, get "true" or "false".
                                                                FieldType::Boolean => {
                                                                    match packed_file_stuff.list_store.get_value(&row, column).get::<bool>().unwrap() {
                                                                        true => "true".to_owned(),
                                                                        false => "false".to_owned(),
                                                                    }
                                                                }

                                                                // If it's an Integer or a Long Integer, turn it into a `String`. Don't know why, but otherwise integer columns crash the program.
                                                                FieldType::Integer => format!("{}", packed_file_stuff.list_store.get_value(&row, column).get::<i32>().unwrap()),
                                                                FieldType::LongInteger => format!("{}", packed_file_stuff.list_store.get_value(&row, column).get::<i64>().unwrap()),

                                                                // If it's any other type, just decode it as `String`.
                                                                _ => packed_file_stuff.list_store.get_value(&row, column).get::<String>().unwrap(),
                                                            };

                                                            // Put the data into the `Clipboard`.
                                                            app_ui.clipboard.set_text(&data);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Paste cell" button.
                                    context_menu_packedfile_db_paste_cell.connect_activate(clone!(
                                        app_ui,
                                        table_definition,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the the focused cell.
                                                let focused_cell = packed_file_stuff.tree_view.get_cursor();

                                                // If there is a focused `TreePath`...
                                                if let Some(tree_path) = focused_cell.0 {

                                                    // And a focused `TreeViewColumn`...
                                                    if let Some(column) = focused_cell.1 {

                                                        // If we got text from the `Clipboard`...
                                                        if let Some(data) = app_ui.clipboard.wait_for_text() {

                                                            // Get his `TreeIter`.
                                                            let row = packed_file_stuff.list_store.get_iter(&tree_path).unwrap();

                                                            // Get his column ID.
                                                            let column = column.get_sort_column_id() as u32;

                                                            // If the cell is the index...
                                                            if column == 0 {

                                                                // Don't do anything.
                                                                return
                                                            }

                                                            // Otherwise...
                                                            else {

                                                                // Check his `field_type`...
                                                                match table_definition.fields[column as usize - 1].field_type {

                                                                    // If it's a boolean, get "true" or "false".
                                                                    FieldType::Boolean => {
                                                                        let state = if data == "true" { true } else if data == "false" { false } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".")
                                                                        };
                                                                        packed_file_stuff.list_store.set_value(&row, column, &state.to_value());
                                                                    }
                                                                    FieldType::Integer => {
                                                                        if let Ok(data) = data.parse::<i32>() {
                                                                            packed_file_stuff.list_store.set_value(&row, column, &data.to_value());
                                                                        } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I32.")
                                                                        };
                                                                    },
                                                                    FieldType::LongInteger => {
                                                                        if let Ok(data) = data.parse::<i64>() {
                                                                            packed_file_stuff.list_store.set_value(&row, column, &data.to_value());
                                                                        } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I64.")
                                                                        };
                                                                    },
                                                                    FieldType::Float => {
                                                                        if let Ok(_) = data.parse::<f32>() {
                                                                            packed_file_stuff.list_store.set_value(&row, column, &data.to_value());
                                                                        } else { return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid F32.") }
                                                                    },

                                                                    // All these are Strings, so it can be together,
                                                                    FieldType::StringU8 |
                                                                    FieldType::StringU16 |
                                                                    FieldType::OptionalStringU8 |
                                                                    FieldType::OptionalStringU16 => packed_file_stuff.list_store.set_value(&row, column, &data.to_value()),
                                                                };

                                                                // Try to save the new data from the `TreeView`.
                                                                match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                                    // If we succeed...
                                                                    Ok(data) => {

                                                                        // Replace our current decoded data with the new one.
                                                                        packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                                        // Try to save the changes to the PackFile. If there is an error, report it.
                                                                        if let Err(error) = update_packed_file_data_db(
                                                                            &*packed_file_data_decoded.borrow_mut(),
                                                                            &mut *pack_file_decoded.borrow_mut(),
                                                                            index as usize
                                                                        ) {
                                                                            ui::show_dialog(&app_ui.window, false, error.cause());
                                                                        }

                                                                        // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                                    }

                                                                    // If there is an error, report it.
                                                                    Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Clone row" button.
                                    context_menu_packedfile_db_clone_rows.connect_activate(clone!(
                                        table_definition,
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the selected row's `TreePath`.
                                                let selected_rows = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                // If we have any row selected...
                                                if !selected_rows.is_empty() {

                                                    // For each selected row...
                                                    for tree_path in &selected_rows {

                                                        // We get the old `TreeIter` and create a new one.
                                                        let old_row = packed_file_stuff.list_store.get_iter(tree_path).unwrap();
                                                        let new_row = packed_file_stuff.list_store.append();

                                                        // For each column...
                                                        for column in 0..(table_definition.fields.len() + 1) {

                                                            // First column it's always the index. Any other column, just copy the values from one `TreeIter` to the other.
                                                            match column {
                                                                0 => packed_file_stuff.list_store.set_value(&new_row, column as u32, &gtk::ToValue::to_value(&format!("New"))),
                                                                _ => packed_file_stuff.list_store.set_value(&new_row, column as u32, &packed_file_stuff.list_store.get_value(&old_row, column as i32)),
                                                            }
                                                        }
                                                    }

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Copy row" button.
                                    context_menu_packedfile_db_copy_rows.connect_activate(clone!(
                                        table_definition,
                                        app_ui,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Get the selected rows.
                                                let selected_rows = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                // Get the list of `TreeIter`s we want to copy.
                                                let tree_iter_list = selected_rows.iter().map(|row| packed_file_stuff.list_store.get_iter(row).unwrap()).collect::<Vec<TreeIter>>();

                                                // Create the `String` that will copy the row that will bring that shit of TLJ down.
                                                let mut copy_string = String::new();

                                                // For each row...
                                                for row in &tree_iter_list {

                                                    // Create the `String` to hold the data from the string.
                                                    let mut row_text = String::new();

                                                    // For each column...
                                                    for column in 1..(table_definition.fields.len() + 1) {

                                                        // Check his `field_type`...
                                                        let data = match table_definition.fields[column as usize - 1].field_type {

                                                            // If it's a boolean, get "true" or "false".
                                                            FieldType::Boolean => {
                                                                match packed_file_stuff.list_store.get_value(&row, column as i32).get::<bool>().unwrap() {
                                                                    true => "true".to_owned(),
                                                                    false => "false".to_owned(),
                                                                }
                                                            }

                                                            // If it's an Integer or a Long Integer, turn it into a `String`. Don't know why, but otherwise integer columns crash the program.
                                                            FieldType::Integer => format!("{}", packed_file_stuff.list_store.get_value(&row, column as i32).get::<i32>().unwrap()),
                                                            FieldType::LongInteger => format!("{}", packed_file_stuff.list_store.get_value(&row, column as i32).get::<i64>().unwrap()),

                                                            // If it's any other type, just decode it as `String`.
                                                            _ => packed_file_stuff.list_store.get_value(&row, column as i32).get::<String>().unwrap(),
                                                        };

                                                        // Add the text to the copied row.
                                                        row_text.push_str(&format!("\"{}\"", data));

                                                        // If it's not the last column...
                                                        if column < table_definition.fields.len() {

                                                            // Put a comma between fields, so excel understand them.
                                                            row_text.push_str(",");
                                                        }
                                                    }

                                                    // Add the copied row to the list.
                                                    copy_string.push_str(&format!("{}\n", row_text));
                                                }

                                                // Pass all the copied rows to the clipboard.
                                                app_ui.clipboard.set_text(&copy_string);
                                            }
                                        }
                                    ));

                                    // When we hit the "Paste row" button.
                                    context_menu_packedfile_db_paste_rows.connect_activate(clone!(
                                        table_definition,
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // Hide the context menu.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // When it gets the data from the `Clipboard`....
                                                if let Some(data) = app_ui.clipboard.wait_for_text() {

                                                    // Get the definitions for this table.
                                                    let fields_type = table_definition.fields.iter().map(|x| x.field_type).collect::<Vec<FieldType>>();

                                                    // Store here all the decoded fields.
                                                    let mut fields_data = vec![];

                                                    // Get the type of the data copied. If it's in CSV format...
                                                    if let Some(_) = data.find("\",\"") {

                                                        // For each row in the data we received...
                                                        for row in data.lines() {

                                                            // Remove the "" at the beginning and at the end.
                                                            let mut row = row.to_owned();
                                                            row.pop();
                                                            row.remove(0);

                                                            // Get all the data from his fields.
                                                            fields_data.push(row.split("\",\"").map(|x| x.to_owned()).collect::<Vec<String>>());
                                                        }
                                                    }

                                                    // Otherwise, we asume it's a TSV copy from excel.
                                                    // TODO: Check this with other possible sources.
                                                    else {

                                                        // For each row in the data we received...
                                                        for row in data.lines() {

                                                            // Get all the data from his fields.
                                                            fields_data.push(row.split('\t').map(|x| x.to_owned()).collect::<Vec<String>>());
                                                        }
                                                    }

                                                    // Get the selected row, if there is any.
                                                    let selected_row = packed_file_stuff.tree_view.get_selection().get_selected_rows().0;

                                                    // If there is at least one line selected, use it as "base" to paste.
                                                    let mut tree_iter = if !selected_row.is_empty() {
                                                        packed_file_stuff.list_store.get_iter(&selected_row[0]).unwrap()
                                                    }

                                                    // Otherwise, append a new `TreeIter` to the `TreeView`, and use it.
                                                    else { packed_file_stuff.list_store.append() };

                                                    // For each row in our fields_data list...
                                                    for (row_index, row) in fields_data.iter().enumerate() {

                                                        // Fill the "Index" column with "New".
                                                        packed_file_stuff.list_store.set_value(&tree_iter, 0, &"New".to_value());

                                                        // For each field in a row...
                                                        for (index, field) in row.iter().enumerate() {

                                                            // Check if that field exists in the table.
                                                            let field_type = fields_type.get(index);

                                                            // If it exists...
                                                            if let Some(field_type) = field_type {

                                                                // Check his `field_type`...
                                                                match *field_type {

                                                                    // If it's a boolean, get "true" or "false".
                                                                    FieldType::Boolean => {
                                                                        let state = if field == "true" { true } else if field == "false" { false } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is neither \"true\" nor \"false\".")
                                                                        };
                                                                        packed_file_stuff.list_store.set_value(&tree_iter, (index + 1) as u32, &state.to_value());
                                                                    }
                                                                    FieldType::Integer => {
                                                                        if let Ok(field) = field.parse::<i32>() {
                                                                            packed_file_stuff.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                                        } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I32.")
                                                                        };
                                                                    },
                                                                    FieldType::LongInteger => {
                                                                        if let Ok(field) = field.parse::<i64>() {
                                                                            packed_file_stuff.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                                        } else {
                                                                            return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid I64.")
                                                                        };
                                                                    },
                                                                    FieldType::Float => {
                                                                        if let Ok(_) = field.parse::<f32>() {
                                                                            packed_file_stuff.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value());
                                                                        } else { return ui::show_dialog(&app_ui.window, false, "Error while trying to paste a cell to a DB PackedFile:\n\nThe value provided is not a valid F32.") }
                                                                    },

                                                                    // All these are Strings, so it can be together,
                                                                    FieldType::StringU8 |
                                                                    FieldType::StringU16 |
                                                                    FieldType::OptionalStringU8 |
                                                                    FieldType::OptionalStringU16 => packed_file_stuff.list_store.set_value(&tree_iter, (index + 1) as u32, &field.to_value()),
                                                                };
                                                            }

                                                            // If the field doesn't exists, return.
                                                            else { return }
                                                        }

                                                        // Move to the next row. If it doesn't exist and it's not the last loop....
                                                        if !packed_file_stuff.list_store.iter_next(&tree_iter) && row_index < (fields_data.len() - 1) {

                                                            // Create it.
                                                            tree_iter = packed_file_stuff.list_store.append();
                                                        }
                                                    }

                                                    // Try to save the new data from the `TreeView`.
                                                    match PackedFileDBTreeView::return_data_from_tree_view(&table_definition, &packed_file_stuff.list_store) {

                                                        // If we succeed...
                                                        Ok(data) => {

                                                            // Replace our current decoded data with the new one.
                                                            packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data = data;

                                                            // Try to save the changes to the PackFile. If there is an error, report it.
                                                            if let Err(error) = update_packed_file_data_db(
                                                                &*packed_file_data_decoded.borrow_mut(),
                                                                &mut *pack_file_decoded.borrow_mut(),
                                                                index as usize
                                                            ) {
                                                                ui::show_dialog(&app_ui.window, false, error.cause());
                                                            }

                                                            // Set the mod as "modified", regardless if we succeed at saving the data or not.
                                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                        }

                                                        // If there is an error, report it.
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                };
                                            }
                                        }
                                    ));

                                    // When we hit the "Import from CSV" button.
                                    context_menu_packedfile_db_import_csv.connect_activate(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // We hide the context menu first.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                // Create the `FileChooser`.
                                                let file_chooser = FileChooserNative::new(
                                                    "Select CSV File to Import...",
                                                    &app_ui.window,
                                                    FileChooserAction::Open,
                                                    "Import",
                                                    "Cancel"
                                                );

                                                // Enable the CSV filter for the `FileChooser`.
                                                file_chooser_filter_packfile(&file_chooser, "*.csv");

                                                // If we have selected a file to import...
                                                if file_chooser.run() == gtk_response_accept {

                                                    // Just in case the import fails after importing (for example, due to importing a CSV from another table,
                                                    // or from another version of the table, and it fails while loading to table or saving to PackFile)
                                                    // we save a copy of the table, so we can restore it if it fails after we modify it.
                                                    let packed_file_data_copy = packed_file_data_decoded.borrow_mut().packed_file_data.clone();
                                                    let mut restore_table = (false, format_err!(""));

                                                    // If there is an error importing, we report it. This only edits the data after checking
                                                    // that it can be decoded properly, so we don't need to restore the table in this case.
                                                    if let Err(error) = DBData::import_csv(
                                                        &mut packed_file_data_decoded.borrow_mut().packed_file_data,
                                                        &file_chooser.get_filename().unwrap()
                                                    ) {
                                                        return ui::show_dialog(&app_ui.window, false, error.cause());
                                                    }

                                                    // If there is an error loading the data (wrong table imported?), report it and restore it from the old copy.
                                                    if let Err(error) = PackedFileDBTreeView::load_data_to_tree_view(&packed_file_data_decoded.borrow().packed_file_data, &packed_file_stuff.list_store) {
                                                        restore_table = (true, error);
                                                    }

                                                    // If the table loaded properly, try to save the data to the encoded file.
                                                    if !restore_table.0 {
                                                        if let Err(error) = update_packed_file_data_db(&*packed_file_data_decoded.borrow_mut(), &mut *pack_file_decoded.borrow_mut(), index as usize) {
                                                            restore_table = (true, error);
                                                        }
                                                    }

                                                    // If the import broke somewhere along the way.
                                                    if restore_table.0 {

                                                        // Restore the old copy.
                                                        packed_file_data_decoded.borrow_mut().packed_file_data = packed_file_data_copy;

                                                        // Report the error.
                                                        ui::show_dialog(&app_ui.window, false, restore_table.1.cause());
                                                    }

                                                    // If there hasn't been any error.
                                                    else {

                                                        // Here we mark the PackFile as "Modified".
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                    }
                                                }
                                            }
                                        }
                                    ));

                                    // When we hit the "Export to CSV" button.
                                    context_menu_packedfile_db_export_csv.connect_activate(clone!(
                                        app_ui,
                                        packed_file_data_decoded,
                                        packed_file_stuff => move |_,_| {

                                            // We hide the context menu first.
                                            packed_file_stuff.context_menu.popdown();

                                            // We only do something in case the focus is in the TreeView. This should stop problems with
                                            // the accels working everywhere.
                                            if packed_file_stuff.tree_view.has_focus() {

                                                let file_chooser = FileChooserNative::new(
                                                    "Export CSV File...",
                                                    &app_ui.window,
                                                    FileChooserAction::Save,
                                                    "Save",
                                                    "Cancel"
                                                );

                                                // We want to ask before overwriting files. Just in case. Otherwise, there can be an accident.
                                                file_chooser.set_do_overwrite_confirmation(true);

                                                // Get it's tree_path and it's default name (table-table_name.csv)
                                                let tree_path = ui::get_tree_path_from_selection(&app_ui.folder_tree_selection, false);
                                                file_chooser.set_current_name(format!("{}-{}.csv", &tree_path[1], &tree_path.last().unwrap()));

                                                // If we hit "Save"...
                                                if file_chooser.run() == gtk_response_accept {

                                                    // Try to export the CSV.
                                                    match DBData::export_csv(&packed_file_data_decoded.borrow_mut().packed_file_data, &file_chooser.get_filename().unwrap()) {
                                                        Ok(result) => ui::show_dialog(&app_ui.window, true, result),
                                                        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                    }
                                                }
                                            }
                                        }
                                    ));
                                }

                                // If we receive an error while decoding, report it.
                                Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                            }
                        }

                        // If it's a plain text file, we create a source view and try to get highlighting for
                        // his language, if it's an specific language file.
                        "TEXT" => {

                            let source_view_buffer = create_text_view(
                                &app_ui.packed_file_data_display,
                                &app_ui.status_bar,
                                &tree_path.last().unwrap(),
                                &pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data
                            );

                            // If we got the SourceView done, we save his buffer on change.
                            match source_view_buffer {
                                Some(source_view_buffer) => {
                                    source_view_buffer.connect_changed(clone!(
                                        app_ui,
                                        pack_file_decoded => move |source_view_buffer| {
                                            let packed_file_data = coding_helpers::encode_string_u8(&source_view_buffer.get_slice(
                                                &source_view_buffer.get_start_iter(),
                                                &source_view_buffer.get_end_iter(),
                                                true
                                            ).unwrap());

                                            update_packed_file_data_text(
                                                &packed_file_data,
                                                &mut pack_file_decoded.borrow_mut(),
                                                index as usize
                                            );

                                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                        }
                                    ));
                                }

                                // If none has been returned, there has been an error while decoding.
                                None => {
                                    let message = "Error while trying to decode a Text PackedFile.";
                                    ui::show_message_in_statusbar(&app_ui.status_bar, message);
                                }
                            }
                        }

                        // If it's an image it doesn't require any extra interaction. Just create the View
                        // and show the Image.
                        "IMAGE" => {
                            create_image_view(
                                &app_ui.packed_file_data_display,
                                &app_ui.status_bar,
                                &tree_path.last().unwrap(),
                                &pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data
                            );
                        }

                        // If it's a rigidmodel, we decode it and take care of his update events.
                        "RIGIDMODEL" => {
                            let packed_file_data_encoded = &*pack_file_decoded.borrow().pack_file_data.packed_files[index as usize].packed_file_data;
                            let packed_file_data_decoded = RigidModel::read(packed_file_data_encoded);
                            match packed_file_data_decoded {
                                Ok(packed_file_data_decoded) => {
                                    let packed_file_data_view_stuff = match ui::packedfile_rigidmodel::PackedFileRigidModelDataView::create_data_view(&app_ui.packed_file_data_display, &packed_file_data_decoded){
                                        Ok(result) => result,
                                        Err(error) => {
                                            let message = format_err!("Error while trying to decode a RigidModel: {}", Error::from(error).cause());
                                            return ui::show_message_in_statusbar(&app_ui.status_bar, message)
                                        },
                                    };
                                    let patch_button = packed_file_data_view_stuff.rigid_model_game_patch_button;
                                    let game_label = packed_file_data_view_stuff.rigid_model_game_label;
                                    let texture_paths = packed_file_data_view_stuff.packed_file_texture_paths;
                                    let texture_paths_index = packed_file_data_view_stuff.packed_file_texture_paths_index;
                                    let packed_file_data_decoded = Rc::new(RefCell::new(packed_file_data_decoded));

                                    // When we hit the "Patch to Warhammer 1&2" button.
                                    patch_button.connect_button_release_event(clone!(
                                        app_ui,
                                        pack_file_decoded,
                                        packed_file_data_decoded => move |patch_button, _| {

                                        // Patch the RigidModel...
                                        let packed_file_data_patch_result = packfile::patch_rigid_model_attila_to_warhammer(&mut *packed_file_data_decoded.borrow_mut());
                                        match packed_file_data_patch_result {
                                            Ok(result) => {

                                                // Disable the button and change his game...
                                                patch_button.set_sensitive(false);
                                                game_label.set_text("Warhammer 1&2");

                                                // Save the changes to the PackFile....
                                                let mut success = false;
                                                match update_packed_file_data_rigid(
                                                    &*packed_file_data_decoded.borrow(),
                                                    &mut *pack_file_decoded.borrow_mut(),
                                                    index as usize
                                                ) {
                                                    Ok(_) => {
                                                        success = true;
                                                        ui::show_dialog(&app_ui.window, true, result);
                                                    },
                                                    Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                                }

                                                // If it works, set it as modified.
                                                if success {
                                                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                }
                                            },
                                            Err(error) => ui::show_dialog(&app_ui.window, false, error.cause()),
                                        }
                                        Inhibit(false)
                                    }));

                                    // When we change any of the Paths...
                                    // TODO: It's extremely slow with big models. Need to find a way to fix it.
                                    for lod in &texture_paths {
                                        for texture_path in lod {
                                            texture_path.connect_changed(clone!(
                                                pack_file_decoded,
                                                packed_file_data_decoded,
                                                texture_paths,
                                                texture_paths_index,
                                                app_ui => move |_| {

                                                    // Get the data from the View...
                                                    let new_data = match PackedFileRigidModelDataView::return_data_from_data_view(
                                                        &texture_paths,
                                                        &texture_paths_index,
                                                        &mut (*packed_file_data_decoded.borrow_mut()).packed_file_data.packed_file_data_lods_data.to_vec()
                                                    ) {
                                                        Ok(new_data) => new_data,
                                                        Err(error) => {
                                                            let message = format_err!("Error while trying to save changes to a RigidModel: {}", Error::from(error).cause());
                                                            return ui::show_message_in_statusbar(&app_ui.status_bar, message)
                                                        }
                                                    };

                                                    // Save it encoded into the opened RigidModel...
                                                    packed_file_data_decoded.borrow_mut().packed_file_data.packed_file_data_lods_data = new_data;

                                                    // And then into the PackFile.
                                                    let success;
                                                    match update_packed_file_data_rigid(
                                                        &*packed_file_data_decoded.borrow(),
                                                        &mut *pack_file_decoded.borrow_mut(),
                                                        index as usize
                                                    ) {
                                                        Ok(_) => { success = true },
                                                        Err(error) => {
                                                            let message = format_err!("Error while trying to save changes to a RigidModel: {}", Error::from(error).cause());
                                                            return ui::show_message_in_statusbar(&app_ui.status_bar, message)
                                                        }
                                                    }

                                                    // If it works, set it as modified.
                                                    if success {
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                                                    }
                                                }
                                            ));
                                        }
                                    }
                                }
                                Err(error) => {
                                    let message = format_err!("Error while trying to decoded a RigidModel: {}", Error::from(error).cause());
                                    return ui::show_message_in_statusbar(&app_ui.status_bar, message)
                                }
                            }
                        }

                        // If we reach this point, the coding to implement this type of file is not done yet,
                        // so we ignore the file.
                        _ => {
                            ui::display_help_tips(&app_ui.packed_file_data_display);
                        }
                    }
                }

                // If it's anything else, then we just show the "Tips" list.
                _ => ui::display_help_tips(&app_ui.packed_file_data_display),
            }
        }
    }));

    // This allow us to open a PackFile by "Drag&Drop" it into the folder_tree_view.
    app_ui.folder_tree_view.connect_drag_data_received(clone!(
        app_ui,
        settings,
        schema,
        game_selected,
        rpfm_path,
        mode,
        supported_games,
        pack_file_decoded_extra,
        pack_file_decoded => move |_, _, _, _, selection_data, info, _| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {

                // If we got confirmation...
                match info {
                    0 => {
                        let pack_file_path = Url::parse(&selection_data.get_uris()[0]).unwrap().to_file_path().unwrap();

                        // Open the PackFile (or die trying it!).
                        if let Err(error) = open_packfile(
                            pack_file_path,
                            &rpfm_path,
                            &app_ui,
                            &settings.borrow(),
                            mode.clone(),
                            &mut schema.borrow_mut(),
                            &supported_games.borrow(),
                            game_selected.clone(),
                            (false, None),
                            pack_file_decoded.clone(),
                            pack_file_decoded_extra.clone()
                        ) { ui::show_dialog(&app_ui.window, false, error.cause()) };
                    }
                    _ => ui::show_dialog(&app_ui.window, false, "This type of event is not yet used."),
                }
            }
        }
    ));

    // If we have an argument (we open RPFM by clicking in a PackFile directly)...
    if arguments.len() > 1 {

        // Get the PackFile's path and...
        let pack_file_path = PathBuf::from(&arguments[1]);

        // Open the PackFile (or die trying it!).
        if let Err(error) = open_packfile(
            pack_file_path,
            &rpfm_path,
            &app_ui,
            &settings.borrow(),
            mode,
            &mut schema.borrow_mut(),
            &supported_games.borrow(),
            game_selected,
            (false, None),
            pack_file_decoded.clone(),
            pack_file_decoded_extra.clone()
        ) { ui::show_dialog(&app_ui.window, false, error.cause()) };
    }
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
    application.set_accels_for_action("packedfile_loc_copy_cell", &[]);
    application.set_accels_for_action("packedfile_loc_paste_cell", &[]);
    application.set_accels_for_action("packedfile_loc_copy_rows", &[]);
    application.set_accels_for_action("packedfile_loc_paste_rows", &[]);
    application.set_accels_for_action("packedfile_loc_import_csv", &[]);
    application.set_accels_for_action("packedfile_loc_export_csv", &[]);
    application.remove_action("packedfile_loc_add_rows");
    application.remove_action("packedfile_loc_delete_rows");
    application.remove_action("packedfile_loc_copy_cell");
    application.remove_action("packedfile_loc_paste_cell");
    application.remove_action("packedfile_loc_copy_rows");
    application.remove_action("packedfile_loc_paste_rows");
    application.remove_action("packedfile_loc_import_csv");
    application.remove_action("packedfile_loc_export_csv");

    // Remove stuff of DB View.
    application.set_accels_for_action("packedfile_db_add_rows", &[]);
    application.set_accels_for_action("packedfile_db_delete_rows", &[]);
    application.set_accels_for_action("packedfile_db_copy_cell", &[]);
    application.set_accels_for_action("packedfile_db_paste_cell", &[]);
    application.set_accels_for_action("packedfile_db_clone_rows", &[]);
    application.set_accels_for_action("packedfile_db_copy_rows", &[]);
    application.set_accels_for_action("packedfile_db_paste_rows", &[]);
    application.set_accels_for_action("packedfile_db_import_csv", &[]);
    application.set_accels_for_action("packedfile_db_export_csv", &[]);
    application.remove_action("packedfile_db_add_rows");
    application.remove_action("packedfile_db_delete_rows");
    application.remove_action("packedfile_db_copy_cell");
    application.remove_action("packedfile_db_paste_cell");
    application.remove_action("packedfile_db_clone_rows");
    application.remove_action("packedfile_db_copy_rows");
    application.remove_action("packedfile_db_paste_rows");
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

/// This function adds a Filter to the provided FileChooser, using the `pattern` &str.
fn file_chooser_filter_packfile(file_chooser: &FileChooserNative, pattern: &str) {
    let filter = FileFilter::new();
    filter.add_pattern(pattern);
    file_chooser.add_filter(&filter);
}

/// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending
/// on the situation.
fn open_packfile(
    pack_file_path: PathBuf,
    rpfm_path: &PathBuf,
    app_ui: &AppUI,
    settings: &Settings,
    mode: Rc<RefCell<Mode>>,
    schema: &mut Option<Schema>,
    supported_games: &[GameInfo],
    game_selected: Rc<RefCell<GameSelected>>,
    is_my_mod: (bool, Option<String>),
    pack_file_decoded: Rc<RefCell<PackFile>>,
    pack_file_decoded_extra: Rc<RefCell<PackFile>>,
) -> Result<(), Error> {
    match packfile::open_packfile(pack_file_path.to_path_buf()) {
        Ok(pack_file_opened) => {

            // If there is no secondary PackFile opened using the "Data View" at the right side...
            if pack_file_decoded_extra.borrow().pack_file_extra_data.file_name.is_empty() {

                // We need to destroy any children that the packed_file_data_display we use may have, cleaning it.
                let children_to_utterly_destroy = app_ui.packed_file_data_display.get_children();
                if !children_to_utterly_destroy.is_empty() {
                    for i in &children_to_utterly_destroy {
                        i.destroy();
                    }
                }

                // Show the "Tips".
                display_help_tips(&app_ui.packed_file_data_display);
            }

            // Get the PackFile into our main PackFile...
            *pack_file_decoded.borrow_mut() = pack_file_opened;

            // Update the Window and the TreeView with his data...
            set_modified(false, &app_ui.window, &mut pack_file_decoded.borrow_mut());

            // Clear the `TreeView` before updating it (fixes CTD with borrowed PackFile).
            app_ui.folder_tree_store.clear();

            // Build the `TreeView`.
            ui::update_treeview(
                &app_ui.folder_tree_store,
                &pack_file_decoded.borrow(),
                &app_ui.folder_tree_selection,
                TreeViewOperation::Build,
                TreePathType::None,
            );

            // We choose the right option, depending on our PackFile.
            match pack_file_decoded.borrow().pack_file_header.pack_file_type {
                0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                _ => ui::show_dialog(&app_ui.window, false, "PackFile Type not valid."),
            }

            // Disable the "PackFile Management" actions.
            enable_packfile_actions(&app_ui, game_selected.clone(), false);

            // If it's a "MyMod", we choose the game selected depending on his folder's name.
            if is_my_mod.0 {

                // Set `GameSelected` depending on the folder of the "MyMod".
                let game_name = is_my_mod.1.clone().unwrap();
                game_selected.borrow_mut().change_game_selected(&game_name, &settings.paths.game_paths.iter().filter(|x| &x.game == &game_name).map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games);
                app_ui.menu_bar_change_game_selected.change_state(&game_name.to_variant());

                // Set the current "Operational Mode" to `MyMod`.
                set_my_mod_mode(&app_ui, mode.clone(), Some(pack_file_path));
            }

            // If it's not a "MyMod", we choose the new GameSelected depending on what the open mod id is.
            else {

                // Set `GameSelected` depending on the ID of the PackFile.
                match &*pack_file_decoded.borrow().pack_file_header.pack_file_id {
                    "PFH5" => {
                        game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.paths.game_paths.iter().filter(|x| &x.game == "warhammer_2").map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games);
                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                    },
                    "PFH4" | _ => {
                        game_selected.borrow_mut().change_game_selected("warhammer", &settings.paths.game_paths.iter().filter(|x| &x.game == "warhammer").map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games);
                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                    },
                }

                // Set the current "Operational Mode" to `Normal`.
                set_my_mod_mode(&app_ui, mode.clone(), None);
            }

            // Enable the "PackFile Management" actions.
            enable_packfile_actions(&app_ui, game_selected.clone(), true);

            // Try to load the Schema for this PackFile's game.
            *schema = Schema::load(&rpfm_path, &pack_file_decoded.borrow().pack_file_header.pack_file_id).ok();

            // Return success.
            Ok(())
        }

        // In case of error while opening the PackFile, return the error.
        Err(error) => Err(error),
    }
}

/// This function takes care of the re-creation of the "MyMod" list in the following moments:
/// - At the start of the program (here).
/// - At the end of MyMod deletion.
/// - At the end of MyMod creation.
/// - At the end of settings update.
fn build_my_mod_menu(
    application: &Application,
    app_ui: &AppUI,
    settings: &Settings,
    mode: Rc<RefCell<Mode>>,
    schema: Rc<RefCell<Option<Schema>>>,
    game_selected: Rc<RefCell<GameSelected>>,
    supported_games: Rc<RefCell<Vec<GameInfo>>>,
    pack_file_decoded: Rc<RefCell<PackFile>>,
    pack_file_decoded_extra: Rc<RefCell<PackFile>>,
    rpfm_path: &PathBuf,
) {
    // First, we clear the list.
    app_ui.my_mod_list.remove_all();

    // If we have the "MyMod" path configured...
    if let Some(ref my_mod_base_path) = settings.paths.my_mods_base_path {

        // And can get without errors the folders in that path...
        if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
            for game_folder in game_folder_list {

                // If the file/folder is valid, we see if it's one of our supported game's folder.
                if let Ok(game_folder) = game_folder {

                    let supported_folders = supported_games.borrow().iter().map(|x| x.folder_name.to_owned()).collect::<Vec<String>>();
                    if game_folder.path().is_dir() && supported_folders.contains(&game_folder.file_name().to_string_lossy().as_ref().to_owned()) {

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
                                    let mod_action = &*format!("my-mod-open-{}-{}", game_folder_name, valid_mod_index);

                                    // GTK have... behavior that needs to be changed when showing "_".
                                    let mod_name_visual = mod_name.replace('_', "__");
                                    game_submenu.append(Some(&*mod_name_visual), Some(&*format!("app.{}", mod_action)));

                                    // We create the action for the new button.
                                    let open_mod = SimpleAction::new(mod_action, None);
                                    application.add_action(&open_mod);

                                    // And when activating the mod button, we open it and set it as selected (chaos incoming).
                                    let game_folder_name = Rc::new(RefCell::new(game_folder_name.clone()));

                                    open_mod.connect_activate(clone!(
                                        app_ui,
                                        settings,
                                        schema,
                                        mode,
                                        game_folder_name,
                                        rpfm_path,
                                        supported_games,
                                        game_selected,
                                        pack_file_decoded_extra,
                                        pack_file_decoded => move |_,_| {

                                            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                            if ui::are_you_sure(&app_ui.window, pack_file_decoded.borrow().pack_file_extra_data.is_modified, false) {

                                                // If we got confirmation...
                                                let pack_file_path = game_folder_file.to_path_buf();

                                                // Open the PackFile (or die trying it!).
                                                if let Err(error) = open_packfile(
                                                    pack_file_path,
                                                    &rpfm_path,
                                                    &app_ui,
                                                    &settings,
                                                    mode.clone(),
                                                    &mut schema.borrow_mut(),
                                                    &supported_games.borrow(),
                                                    game_selected.clone(),
                                                    (true, Some(game_folder_name.borrow().to_owned())),
                                                    pack_file_decoded.clone(),
                                                    pack_file_decoded_extra.clone()
                                                ) { ui::show_dialog(&app_ui.window, false, error.cause()) };
                                            }
                                        }
                                    ));

                                    valid_mod_index += 1;
                                }
                            }
                        }

                        // Only if the submenu has items, we add it to the big menu.
                        if game_submenu.get_n_items() > 0 {
                            let game_submenu_name = supported_games.borrow().iter().filter(|x| game_folder_name == x.folder_name).map(|x| x.display_name.to_owned()).collect::<String>();
                            app_ui.my_mod_list.append_submenu(Some(&*format!("{}", game_submenu_name)), &game_submenu);
                        }
                    }
                }
            }
        }
    }
}

/// This function serves as a common function for all the "Patch SiegeAI" buttons from "Special Stuff".
fn patch_siege_ai(
    app_ui: &AppUI,
    pack_file_decoded: Rc<RefCell<PackFile>>,
) {

    // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
    // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
    let mut sucessful_patching = (false, String::new());
    match packfile::patch_siege_ai(&mut *pack_file_decoded.borrow_mut()) {
        Ok(result) => sucessful_patching = (true, result),
        Err(error) => ui::show_dialog(&app_ui.window, false, error.cause())
    }
    if sucessful_patching.0 {
        let mut success = false;
        match packfile::save_packfile( &mut *pack_file_decoded.borrow_mut(), None) {
            Ok(result) => {
                success = true;
                ui::show_dialog(&app_ui.window, true, format!("{}\n\n{}", sucessful_patching.1, result));
            },
            Err(error) => ui::show_dialog(&app_ui.window, false, error.cause())
        }

        // If it succeed...
        if success {

            // Clear the `TreeView` before updating it (fixes CTD with borrowed PackFile).
            app_ui.folder_tree_store.clear();

            // TODO: Make this update, not rebuild.
            // Rebuild the `TreeView`.
            ui::update_treeview(
                &app_ui.folder_tree_store,
                &*pack_file_decoded.borrow(),
                &app_ui.folder_tree_selection,
                TreeViewOperation::Build,
                TreePathType::None,
            );
        }
    }
}

/// This function serves as a common function for all the "Generate Dependency Pack" buttons from "Special Stuff".
fn generate_dependency_pack(
    app_ui: &AppUI,
    rpfm_path: &PathBuf,
    game_selected: Rc<RefCell<GameSelected>>,
) {

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
                    let mut dep_packs_path = rpfm_path.clone();
                    dep_packs_path.push("dependency_packs");

                    match DirBuilder::new().create(&dep_packs_path) {
                        Ok(_) | Err(_) => {},
                    }

                    let pack_file_path = match &*game_selected.borrow().game {
                        "warhammer_2" => PathBuf::from(format!("{}/wh2.pack", dep_packs_path.to_string_lossy())),
                        "warhammer" | _ => PathBuf::from(format!("{}/wh.pack", dep_packs_path.to_string_lossy())),
                    };

                    match packfile::save_packfile(data_packfile, Some(pack_file_path)) {
                        Ok(_) => ui::show_dialog(&app_ui.window, true, "Dependency pack created. Remember to re-create it if you update the game ;)."),
                        Err(error) => ui::show_dialog(&app_ui.window, false, format_err!("Error: generated dependency pack couldn't be saved. {:?}", error)),
                    }
                }
                Err(_) => ui::show_dialog(&app_ui.window, false, "Error: data.pack couldn't be open.")
            }
        },
        None => ui::show_dialog(&app_ui.window, false, "Error: data path of the game not found.")
    }
}

/// This function is used to set the current "Operational Mode". It not only sets the "Operational Mode",
/// but it also takes care of disabling or enabling all the signals related with the "MyMod" Mode.
/// If `my_mod_path` is None, we want to set the `Normal` mode. Otherwise set the `MyMod` mode.
fn set_my_mod_mode(
    app_ui: &AppUI,
    mode: Rc<RefCell<Mode>>,
    my_mod_path: Option<PathBuf>,
) {
    // Check if we provided a "my_mod_path".
    match my_mod_path {

        // If we have a `my_mod_path`...
        Some(my_mod_path) => {

            // Get the `folder_name` and the `mod_name` of our "MyMod".
            let mut path = my_mod_path.clone();
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
            path.pop();
            let game_folder_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Set the current mode to `MyMod`.
            *mode.borrow_mut() = Mode::MyMod {
                game_folder_name,
                mod_name,
            };

            // Enable the controls for "MyMod".
            app_ui.menu_bar_my_mod_delete.set_enabled(true);
            app_ui.menu_bar_my_mod_install.set_enabled(true);
            app_ui.menu_bar_my_mod_uninstall.set_enabled(true);
        }

        // If `None` has been provided...
        None => {

            // Set the current mode to `Normal`.
            *mode.borrow_mut() = Mode::Normal;

            // Disable all "MyMod" related actions, except "New MyMod".
            app_ui.menu_bar_my_mod_delete.set_enabled(false);
            app_ui.menu_bar_my_mod_install.set_enabled(false);
            app_ui.menu_bar_my_mod_uninstall.set_enabled(false);
        }
    }
}

/// This function enables or disables the actions from the `MenuBar` needed when we open a PackFile.
/// NOTE: To disable the "Special Stuff" actions, we use `disable`
fn enable_packfile_actions(app_ui: &AppUI, game_selected: Rc<RefCell<GameSelected>>, enable: bool) {

    // Enable or disable the actions from "PackFile" Submenu.
    app_ui.menu_bar_save_packfile.set_enabled(enable);
    app_ui.menu_bar_save_packfile_as.set_enabled(enable);
    app_ui.menu_bar_change_packfile_type.set_enabled(enable);

    // Only if we are enabling...
    if enable {

        // Enable the actions from the "Special Stuff" Submenu.
        match &*game_selected.borrow().game {
            "warhammer_2" => {
                app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(true);
                app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(true);
            },
            "warhammer" => {
                app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(true);
                app_ui.menu_bar_patch_siege_ai_wh.set_enabled(true);
            },
            _ => {},
        }
    }

    // If we are disabling...
    else {
        // Disable Warhammer 2 actions...
        app_ui.menu_bar_generate_dependency_pack_wh2.set_enabled(false);
        app_ui.menu_bar_patch_siege_ai_wh2.set_enabled(false);

        // Disable actions...
        app_ui.menu_bar_generate_dependency_pack_wh.set_enabled(false);
        app_ui.menu_bar_patch_siege_ai_wh.set_enabled(false);
    }
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
