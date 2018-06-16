// This is the main file of RPFM. Here is the main loop that builds the UI and controls
// his events.

// Disable warnings about unknown lints, so we don't have the linter warnings when compiling.
#![allow(unknown_lints)]

// Disable these two clippy linters. They throw a lot of false positives, and it's a pain in the ass
// to separate their warnings from the rest. Also, disable "match_bool" because the methods it suggest
// are harder to read than a match. And "redundant_closure", because the suggerences it gives doesn't work.
#![allow(doc_markdown,useless_format,match_bool,redundant_closure)]

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate failure;
extern crate num;
extern crate url;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;
extern crate restson;

use restson::RestClient;

use qt_widgets::menu::Menu;
use qt_widgets::action::Action;
use qt_widgets::application::Application;
use qt_widgets::widget::Widget;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::splitter::Splitter;
use qt_widgets::tree_view::TreeView;
use qt_widgets::main_window::MainWindow;
use qt_widgets::message_box::MessageBox;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::file_dialog::FileDialog;

use qt_gui::desktop_services::DesktopServices;
use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::icon::Icon;
use qt_core::item_selection_model::ItemSelectionModel;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::flags::Flags;
use qt_core::event_loop::EventLoop;
use qt_core::connection::Signal;
use qt_gui::standard_item::StandardItem;
use qt_core::object::Object;
use qt_core::variant::Variant;
use qt_core::slots::SlotBool;
use qt_core::slots::SlotNoArgs;
use cpp_utils::{CppBox, StaticCast};

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::{
    DirBuilder, copy, remove_file, remove_dir_all
};
use std::env::args;
use failure::Error;
use url::Url;
use common::*;
use packfile::packfile::PackFile;
use packfile::packfile::PackFileExtraData;
use packfile::packfile::PackFileHeader;
use packfile::packfile::PackedFile;
use packedfile::*;
use packedfile::db::schemas::*;
use packedfile::db::schemas_importer::*;
use settings::*;
use updater::*;
use ui::*;
use ui::settings::*;
use ui::updater::*;
/*
use ui::packedfile_db::*;
use ui::packedfile_loc::*;
use ui::packedfile_text::*;
use ui::packedfile_image::*;
use ui::packedfile_rigidmodel::*;
*/
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

/// Custom type to deal with QStrirngs more easely.
type QString = qt_core::string::String;

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
/*
/// This struct contains almost the entirety of the UI stuff, so it's not a fucking chaos when
/// going inside/outside closures. The exceptions for this struct is stuff generated after RPFM is
/// started, like the TreeView for DB PackedFiles or the DB Decoder View.
#[derive(Clone)]
pub struct AppUI {

    // Clipboard.
    pub clipboard: Clipboard,

    // Main window.
    pub window: ApplicationWindow,

    // MenuBar at the top of the Window.
    pub menu_bar: Menu,

    // Section of the "MyMod" menu.
    pub my_mod_list: Menu,

    // Shortcut window.
    pub shortcuts_window: ShortcutsWindow,

    // This is the box where all the PackedFile Views are created.
    pub packed_file_data_display: Grid,

    // Status bar at the bottom of the program. To show informative messages.
    pub status_bar: Statusbar,

    // TreeView used to see the PackedFiles, and his TreeStore and TreeSelection.
    pub folder_tree_view: TreeView,
    pub folder_tree_store: TreeStore,
    pub folder_tree_selection: TreeSelection,

    // Column and cells for the `TreeView`.
    pub folder_tree_view_cell: CellRendererText,
    pub folder_tree_view_column: TreeViewColumn,

    // Context Menu Popover for `folder_tree_view`. It's build from a Model, stored here too.
    pub folder_tree_view_context_menu: Popover,
    pub folder_tree_view_context_menu_model: MenuModel,

    // Actions of RPFM's MenuBar.
    pub menu_bar_new_packfile: SimpleAction,
    pub menu_bar_open_packfile: SimpleAction,
    pub menu_bar_save_packfile: SimpleAction,
    pub menu_bar_save_packfile_as: SimpleAction,
    pub menu_bar_preferences: SimpleAction,
    pub menu_bar_quit: SimpleAction,
    pub menu_bar_generate_dependency_pack_wh2: SimpleAction,
    pub menu_bar_patch_siege_ai_wh2: SimpleAction,
    pub menu_bar_create_map_prefab_wh2: SimpleAction,
    pub menu_bar_generate_dependency_pack_wh: SimpleAction,
    pub menu_bar_patch_siege_ai_wh: SimpleAction,
    pub menu_bar_create_map_prefab_wh: SimpleAction,
    pub menu_bar_generate_dependency_pack_att: SimpleAction,
    pub menu_bar_check_updates: SimpleAction,
    pub menu_bar_check_schema_updates: SimpleAction,
    pub menu_bar_open_patreon: SimpleAction,
    pub menu_bar_about: SimpleAction,
    pub menu_bar_change_packfile_type: SimpleAction,
    pub menu_bar_my_mod_new: SimpleAction,
    pub menu_bar_my_mod_delete: SimpleAction,
    pub menu_bar_my_mod_install: SimpleAction,
    pub menu_bar_my_mod_uninstall: SimpleAction,
    pub menu_bar_change_game_selected: SimpleAction,

    // Actions of the Context Menu for `folder_tree_view`.
    pub folder_tree_view_add_file: SimpleAction,
    pub folder_tree_view_add_folder: SimpleAction,
    pub folder_tree_view_add_from_packfile: SimpleAction,
    pub folder_tree_view_rename_packedfile: SimpleAction,
    pub folder_tree_view_delete_packedfile: SimpleAction,
    pub folder_tree_view_extract_packedfile: SimpleAction,
    pub folder_tree_view_create_loc: SimpleAction,
    pub folder_tree_view_create_db: SimpleAction,
    pub folder_tree_view_create_text: SimpleAction,
    pub folder_tree_view_mass_import_tsv_files: SimpleAction,

    // Model for the Context Menu of the DB Decoder (only the model, the menu is created and destroyed with the decoder).
    pub db_decoder_context_menu_model: MenuModel,
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
        menu_bar_create_map_prefab_wh2: SimpleAction::new("create-map-prefab-wh2", None),
        menu_bar_generate_dependency_pack_wh: SimpleAction::new("generate-dependency-pack-wh", None),
        menu_bar_patch_siege_ai_wh: SimpleAction::new("patch-siege-ai-wh", None),
        menu_bar_create_map_prefab_wh: SimpleAction::new("create-map-prefab-wh", None),
        menu_bar_generate_dependency_pack_att: SimpleAction::new("generate-dependency-pack-att", None),
        menu_bar_check_updates: SimpleAction::new("check-updates", None),
        menu_bar_check_schema_updates: SimpleAction::new("check-schema-updates", None),
        menu_bar_open_patreon: SimpleAction::new("open-patreon", None),
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
        folder_tree_view_create_loc: SimpleAction::new("create-loc", None),
        folder_tree_view_create_db: SimpleAction::new("create-db", None),
        folder_tree_view_create_text: SimpleAction::new("create-text", None),
        folder_tree_view_mass_import_tsv_files: SimpleAction::new("mass-import-tsv", None),

        // Model for the Context Menu of the DB Decoder (only the model, the menu is created and destroyed with the decoder).
        db_decoder_context_menu_model: builder.get_object("context_menu_db_decoder").unwrap(),
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
    application.add_action(&app_ui.menu_bar_create_map_prefab_wh2);
    application.add_action(&app_ui.menu_bar_generate_dependency_pack_wh);
    application.add_action(&app_ui.menu_bar_patch_siege_ai_wh);
    application.add_action(&app_ui.menu_bar_create_map_prefab_wh);
    application.add_action(&app_ui.menu_bar_generate_dependency_pack_att);
    application.add_action(&app_ui.menu_bar_open_patreon);
    application.add_action(&app_ui.menu_bar_about);
    application.add_action(&app_ui.menu_bar_check_updates);
    application.add_action(&app_ui.menu_bar_check_schema_updates);
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
    application.add_action(&app_ui.folder_tree_view_create_loc);
    application.add_action(&app_ui.folder_tree_view_create_db);
    application.add_action(&app_ui.folder_tree_view_create_text);
    application.add_action(&app_ui.folder_tree_view_mass_import_tsv_files);

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
    display_help_tips(&app_ui.packed_file_data_display);

    // This is to get the new schemas. It's controlled by a global const.
    if GENERATE_NEW_SCHEMA {

        // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
        // The second one should contain all the tables of the game, extracted directly from `data.pack`.
        let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_schemas/");
        let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_tables/");
        match import_schema(&assembly_kit_schemas_path, &testing_tables_path, &rpfm_path) {
            Ok(_) => show_dialog(&app_ui.window, true, "Schema successfully created."),
            Err(error) => return show_dialog(&app_ui.window, false, format!("Error while creating a new DB Schema file:\n{}", error.cause())),
        }
    }

    // This variable is used to "Lock" the "Decode on select" feature of `app_ui.folder_tree_view`.
    // We need it to lock this feature when we open a secondary PackFile and want to import some
    // PackedFiles to our opened PackFile.
    let is_folder_tree_view_locked = Rc::new(RefCell::new(false));

    // This variable is used to "Lock" the "Delete PackedFile" action. We need this because this is
    // the only action that can change the index of a PackedFile while it's open, causing it to try
    // to save itself in the position of another PackedFile. This can trigger data corruption or an
    // "index out of bounds" CTD in runtime, so we need this variable to check if we can delete a
    // PackedFile before even trying it.
    let is_packedfile_opened = Rc::new(RefCell::new(false));

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

    // Try to open the dependency PackFile of our `game_selected`.
    let dependency_database = match packfile::open_packfile(game_selected.borrow().game_dependency_packfile_path.to_path_buf()) {
        Ok(pack_file) => Rc::new(RefCell::new(Some(pack_file.data.packed_files))),
        Err(_) => Rc::new(RefCell::new(None)),
    };

    // Prepare the "MyMod" menu. This... atrocity needs to be in the following places for MyMod to open PackFiles:
    // - At the start of the program (here).
    // - At the end of MyMod creation.
    // - At the end of MyMod deletion.
    // - At the end of settings update.
    build_my_mod_menu(
        application,
        &app_ui,
        &settings.borrow(),
        &mode,
        &schema,
        &game_selected,
        &supported_games,
        &dependency_database,
        &pack_file_decoded,
        &pack_file_decoded_extra,
        &rpfm_path
    );

    // Check for updates at the start if we have this option enabled. Currently this hangs the UI,
    // so do it before showing the UI.
    if settings.borrow().check_updates_on_start {
        check_updates(VERSION, None, Some(&app_ui.status_bar));
    }

    // Same with schema updates.
    if settings.borrow().check_schema_updates_on_start {
        check_schema_updates(VERSION, &rpfm_path, &supported_games.borrow(), &game_selected, &schema, None, Some(&app_ui.status_bar));
    }

    // Concatenate and push again the last two messages of the Statusbar, to be able to show both message at the same time.
    // FIXME: This is a dirty trick, so it should be fixed in the future.
    concatenate_check_update_messages(&app_ui.status_bar);

    // We bring up the main window.
    app_ui.window.show_all();

    // End of the "Getting Ready" part.
    // From here, it's all event handling.

    // First, we catch the close window event, and close the program when we do it.
    app_ui.window.connect_delete_event(clone!(
        application,
        pack_file_decoded,
        app_ui => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {

                // If we got confirmation...
                application.quit()
            }
            Inhibit(true)
        }
    ));

    // Set the current "Operational Mode" to `Normal`.
    set_my_mod_mode(&app_ui, &mode, None);

    // Disable the "PackFile Management" actions by default.
    enable_packfile_actions(&app_ui, &game_selected, false);

    // Disable all the Contextual Menu actions by default.
    app_ui.folder_tree_view_add_file.set_enabled(false);
    app_ui.folder_tree_view_add_folder.set_enabled(false);
    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
    app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
    app_ui.folder_tree_view_extract_packedfile.set_enabled(false);
    app_ui.folder_tree_view_create_loc.set_enabled(false);
    app_ui.folder_tree_view_create_db.set_enabled(false);
    app_ui.folder_tree_view_create_text.set_enabled(false);
    app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(false);

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
            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {

                // If there is no secondary PackFile opened using the "Data View" at the right side...
                if pack_file_decoded_extra.borrow().extra_data.file_name.is_empty() {

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
                update_treeview(
                    &app_ui.folder_tree_store,
                    &*pack_file_decoded.borrow(),
                    &app_ui.folder_tree_selection,
                    TreeViewOperation::Build,
                    &TreePathType::None,
                );

                // Set the new mod as "Not modified".
                set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                // Enable the actions available for the PackFile from the `MenuBar`.
                enable_packfile_actions(&app_ui, &game_selected, true);

                // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                set_my_mod_mode(&app_ui, &mode, None);

                // Try to load the Schema for this PackFile's game.
                *schema.borrow_mut() = Schema::load(&rpfm_path, &supported_games.borrow().iter().filter(|x| x.folder_name == *game_selected.borrow().game).map(|x| x.schema.to_owned()).collect::<String>()).ok();
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
        dependency_database,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {

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
                        &mode,
                        &schema,
                        &supported_games.borrow(),
                        &game_selected,
                        &dependency_database,
                        &(false, None),
                        &pack_file_decoded,
                        &pack_file_decoded_extra
                    ) { show_dialog(&app_ui.window, false, error.cause()) };
                }
            }
        }
    ));


    // When we hit the "Save PackFile" button
    app_ui.menu_bar_save_packfile.connect_activate(clone!(
        pack_file_decoded,
        settings,
        app_ui => move |_,_| {

            // If our PackFile is editable...
            if pack_file_decoded.borrow().is_editable(&settings.borrow()) {

                // If our PackFile already exists in the filesystem, we save it to that file directly.
                if pack_file_decoded.borrow().extra_data.file_path.is_file() {

                    // We try to save the PackFile at the provided path...
                    let success = match packfile::save_packfile(&mut *pack_file_decoded.borrow_mut(), None) {
                        Ok(_) => {
                            show_dialog(&app_ui.window, true, "PackFile succesfully saved.");
                            true
                        },
                        Err(error) => {
                            show_dialog(&app_ui.window, false, error.cause());
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

            // Otherwise, return a Message specifying the error.
            else { show_dialog(&app_ui.window, false, "This type of PackFile is supported in Read-Only mode.\n\nThis can happen due to:\n - The PackFile's type is 'Boot', 'Release' or 'Patch' and you have 'Allow edition of CA PackFiles' disabled in the settings.\n - The PackFile's type is 'Other'.\n\n If you really want to save it, go to 'PackFile/Change PackFile Type' and change his type to 'Mod' or 'Movie'."); }
        }
    ));


    // When we hit the "Save PackFile as" button.
    app_ui.menu_bar_save_packfile_as.connect_activate(clone!(
        pack_file_decoded,
        game_selected,
        settings,
        app_ui,
        mode => move |_,_| {

            // If our PackFile is editable...
            if pack_file_decoded.borrow().is_editable(&settings.borrow()) {

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
                file_chooser_save_packfile.set_current_name(&pack_file_decoded.borrow().extra_data.file_name);

                // If we are saving an existing PackFile with another name, we start in his current path.
                if pack_file_decoded.borrow().extra_data.file_path.is_file() {
                    file_chooser_save_packfile.set_filename(&pack_file_decoded.borrow().extra_data.file_path);
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
                        Ok(_) => {
                            show_dialog(&app_ui.window, true, "PackFile succesfully saved.");
                            true
                        },
                        Err(error) => {
                            show_dialog(&app_ui.window, false, error.cause());
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
                        update_treeview(
                            &app_ui.folder_tree_store,
                            &*pack_file_decoded.borrow(),
                            &app_ui.folder_tree_selection,
                            TreeViewOperation::Rename(file_path.file_name().unwrap().to_string_lossy().as_ref().to_owned()),
                            &TreePathType::None,
                        );

                        // Set the current "Operational Mode" to Normal, just in case "MyMod" is the current one.
                        set_my_mod_mode(&app_ui, &mode, None);
                    }
                }
            }

            // Otherwise, return a Message specifying the error.
            else { show_dialog(&app_ui.window, false, "This type of PackFile is supported in Read-Only mode.\n\nThis can happen due to:\n - The PackFile's type is 'Boot', 'Release' or 'Patch' and you have 'Allow edition of CA PackFiles' disabled in the settings.\n - The PackFile's type is 'Other'.\n\n If you really want to save it, go to 'PackFile/Change PackFile Type' and change his type to 'Mod' or 'Movie'."); }
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
                        if pack_file_decoded.borrow().header.pack_file_type != 0 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 0;
                            menu_bar_change_packfile_type.change_state(&"boot".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
                    "release" => {
                        if pack_file_decoded.borrow().header.pack_file_type != 1 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 1;
                            menu_bar_change_packfile_type.change_state(&"release".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
                    "patch" => {
                        if pack_file_decoded.borrow().header.pack_file_type != 2 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 2;
                            menu_bar_change_packfile_type.change_state(&"patch".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
                    "mod" => {
                        if pack_file_decoded.borrow().header.pack_file_type != 3 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 3;
                            menu_bar_change_packfile_type.change_state(&"mod".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
                    "movie" => {
                        if pack_file_decoded.borrow().header.pack_file_type != 4 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 4;
                            menu_bar_change_packfile_type.change_state(&"movie".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
                    _ => {
                        if pack_file_decoded.borrow().header.pack_file_type != 9999 {
                            pack_file_decoded.borrow_mut().header.pack_file_type = 9999;
                            menu_bar_change_packfile_type.change_state(&"other".to_variant());
                            set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());
                        }
                    }
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
        dependency_database,
        pack_file_decoded_extra,
        schema => move |_,_| {

            // We disable the action, so we can't start 2 "Settings" windows at the same time.
            app_ui.menu_bar_preferences.set_enabled(false);

            // We create the "Settings Window" and load our current settings to it.
            let settings_stuff = Rc::new(RefCell::new(SettingsWindow::create_settings_window(&application, &app_ui.window, &rpfm_path, &supported_games.borrow())));
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
                dependency_database,
                pack_file_decoded_extra,
                application => move |_,_| {

                    // Save a copy of our old `Settings` to use in the checks below.
                    let old_settings = settings.borrow().clone();

                    // Save the current `Settings` from the "Settings Window" as our new `Settings`.
                    *settings.borrow_mut() = settings_stuff.borrow().save_from_settings_window(&supported_games.borrow());

                    // Save our new `Settings` to a settings file, and report in case of error.
                    if let Err(error) = settings.borrow().save(&rpfm_path) {
                        show_dialog(&app_ui.window, false, error.cause());
                    }

                    // Destroy the "Settings Window".
                    settings_stuff.borrow().settings_window.destroy();

                    // Restore the action, so we can open another "Settings Window" again.
                    app_ui.menu_bar_preferences.set_enabled(true);

                    // If we changed the "MyMod's Folder" path...
                    if settings.borrow().paths.my_mods_base_path != old_settings.paths.my_mods_base_path {

                        // And we have currently opened a "MyMod"...
                        if let Mode::MyMod{..} = *mode.borrow() {

                            // We disable the "MyMod" mode, but leave the PackFile open, so the user doesn't lose any unsaved change.
                            set_my_mod_mode(&app_ui, &mode, None);

                            // Then recreate the "MyMod" submenu.
                            build_my_mod_menu(
                                &application,
                                &app_ui,
                                &settings.borrow(),
                                &mode,
                                &schema,
                                &game_selected,
                                &supported_games,
                                &dependency_database,
                                &pack_file_decoded,
                                &pack_file_decoded_extra,
                                &rpfm_path
                            );
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
            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {
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
        dependency_database,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // We disable the action, so we can't start 2 "New MyMod" windows at the same time.
            app_ui.menu_bar_my_mod_new.set_enabled(false);

            // Create the the "New MyMod" window and put all it's stuff into a variable.
            let new_mod_stuff = Rc::new(RefCell::new(MyModNewWindow::create_my_mod_new_window(&application, &app_ui.window, &supported_games.borrow(), &game_selected.borrow(), &settings.borrow(), &rpfm_path)));

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
                dependency_database,
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
                    update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Build,
                        &TreePathType::None,
                    );

                    // Set the new mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &game_selected, true);

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
                        show_dialog(&app_ui.window, false, error.cause());
                    }

                    // If the new "MyMod" has been saved successfully...
                    else {

                        // Set the current "Operational Mode" to `MyMod`.
                        set_my_mod_mode(&app_ui, &mode, Some(my_mod_path));

                        // Recreate the "MyMod" menu.
                        build_my_mod_menu(
                            &application,
                            &app_ui,
                            &settings.borrow(),
                            &mode,
                            &schema,
                            &game_selected,
                            &supported_games,
                            &dependency_database,
                            &pack_file_decoded,
                            &pack_file_decoded_extra,
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
        dependency_database,
        pack_file_decoded_extra,
        pack_file_decoded => move |_,_| {

            // This will delete stuff from disk, so we pop up the "Are you sure?" message to avoid accidents.
            if are_you_sure(&app_ui.window, true, true) {

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
                                return show_dialog(&app_ui.window, false, "PackFile doesn't exist.");
                            }

                            // And we delete that PackFile.
                            if let Err(error) = remove_file(&my_mod_path).map_err(Error::from) {
                                return show_dialog(&app_ui.window, false, error.cause());
                            }

                            // Now we get his asset folder.
                            let mut my_mod_assets_path = my_mod_path.clone();
                            my_mod_assets_path.pop();
                            my_mod_assets_path.push(&my_mod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists. This is optional, so it should allow the deletion
                            // process to continue with a warning.
                            if !my_mod_assets_path.is_dir() {
                                show_dialog(&app_ui.window, false, "Mod deleted, but his assets folder hasn't been found.");
                            }

                            // If the assets folder exists, we try to delete it.
                            else if let Err(error) = remove_dir_all(&my_mod_assets_path).map_err(Error::from) {
                                return show_dialog(&app_ui.window, false, error.cause());
                            }

                            // We return true, as we have delete the files of the "MyMod".
                            true
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else {
                            return show_dialog(&app_ui.window, false, "MyMod base path not configured.");
                        }
                    }

                    // If we don't have a "MyMod" selected, return an error.
                    Mode::Normal => return show_dialog(&app_ui.window, false, "MyMod not selected."),
                };

                // If we deleted the "MyMod", we allow chaos to form below.
                if mod_deleted {

                    // Set the current "Operational Mode" to `Normal`.
                    set_my_mod_mode(&app_ui, &mode, None);

                    // Replace the open PackFile with a dummy one, like during boot.
                    *pack_file_decoded.borrow_mut() = PackFile::new();

                    // Disable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &game_selected, false);

                    // Set the dummy mod as "Not modified".
                    set_modified(false, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Clear the TreeView.
                    app_ui.folder_tree_store.clear();

                    // Rebuild the "MyMod" menu.
                    build_my_mod_menu(
                        &application,
                        &app_ui,
                        &settings.borrow(),
                        &mode,
                        &schema,
                        &game_selected,
                        &supported_games,
                        &dependency_database,
                        &pack_file_decoded,
                        &pack_file_decoded_extra,
                        &rpfm_path
                    );

                    // Show the "MyMod" deleted Dialog.
                    show_dialog(&app_ui.window, true, format!("MyMod \"{}\" deleted.", old_mod_name));
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
                                return show_dialog(&app_ui.window, false, "PackFile doesn't exist.");
                            }

                            // We check that the destination path exists.
                            if !game_data_path.is_dir() {
                                return show_dialog(&app_ui.window, false, "Destination folder (..xxx/data) doesn't exist. You sure you configured the right folder for the game?");
                            }

                            // Get the destination path for the PackFile with the PackFile included.
                            game_data_path.push(&mod_name);

                            // And copy the PackFile to his destination. If the copy fails, return an error.
                            if let Err(error) = copy(my_mod_path, game_data_path).map_err(Error::from) {
                                return show_dialog(&app_ui.window, false, error.cause());
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else {
                            return show_dialog(&app_ui.window, false, "Game folder path not configured.");
                        }

                    // If the "MyMod" path is not configured, return an error.
                    }
                    else {
                        show_dialog(&app_ui.window, false, "MyMod base path not configured.");
                    }
                }

                // If we have no MyMod selected, return an error.
                Mode::Normal => show_dialog(&app_ui.window, false, "MyMod not selected."),
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
                Mode::MyMod {ref mod_name,..} => {

                    // Get the `game_data_path` of the game.
                    let game_data_path = game_selected.borrow().game_data_path.clone();

                    // If we have a `game_data_path` for the current `GameSelected`...
                    if let Some(mut game_data_path) = game_data_path {

                        // Get the destination path for the PackFile with the PackFile included.
                        game_data_path.push(&mod_name);

                        // We check that the "MyMod" is actually installed in the provided path.
                        if !game_data_path.is_file() {
                            return show_dialog(&app_ui.window, false, "The currently selected \"MyMod\" is not installed.");
                        }

                        // If the "MyMod" is installed, we remove it. If there is a problem deleting it, return an error dialog.
                        else if let Err(error) = remove_file(game_data_path).map_err(Error::from) {
                            return show_dialog(&app_ui.window, false, error.cause());
                        }
                    }

                    // If we don't have a `game_data_path` configured for the current `GameSelected`...
                    else {
                        show_dialog(&app_ui.window, false, "Game folder path not configured.");
                    }
                }

                // If we have no MyMod selected, return an error.
                Mode::Normal => show_dialog(&app_ui.window, false, "MyMod not selected."),
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
        app_ui,
        rpfm_path,
        schema,
        mode,
        settings,
        supported_games,
        pack_file_decoded,
        dependency_database,
        game_selected => move |menu_bar_change_game_selected, selected| {

            // Get the new state of the action.
            if let Some(state) = selected.clone() {
                let new_state: String = state.get().unwrap();

                // Change the state of the action.
                menu_bar_change_game_selected.change_state(&new_state.to_variant());

                // Change the `GameSelected` object.
                game_selected.borrow_mut().change_game_selected(&new_state, &settings.borrow().paths.game_paths.iter().filter(|x| x.game == new_state).map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games.borrow());

                // Change the `Schema` for that game.
                *schema.borrow_mut() = Schema::load(&rpfm_path, &supported_games.borrow().iter().filter(|x| x.folder_name == *game_selected.borrow().game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                // Change the `dependency_database` for that game.
                *dependency_database.borrow_mut() = match packfile::open_packfile(game_selected.borrow().game_dependency_packfile_path.to_path_buf()) {
                    Ok(data) => Some(data.data.packed_files),
                    Err(_) => None,
                };

                // If we have a PackFile opened....
                if !pack_file_decoded.borrow().extra_data.file_name.is_empty() {

                    // Re-enable the "PackFile Management" actions, so the "Special Stuff" menu gets updated properly.
                    enable_packfile_actions(&app_ui, &game_selected, false);
                    enable_packfile_actions(&app_ui, &game_selected, true);

                    // Set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                    set_my_mod_mode(&app_ui, &mode, None);
                }
            }
        }
    ));
    /*
    --------------------------------------------------------
                 Superior Menu: "Special Stuff"
    --------------------------------------------------------
    */

    // When we hit the "Patch SiegeAI" button.
    app_ui.menu_bar_patch_siege_ai_wh2.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_| {
            patch_siege_ai(&app_ui, &pack_file_decoded);
        }
    ));

    // When we hit the "Generate Dependency Pack" button.
    app_ui.menu_bar_generate_dependency_pack_wh2.connect_activate(clone!(
        app_ui,
        rpfm_path,
        game_selected => move |_,_| {
            generate_dependency_pack(&app_ui, &rpfm_path, &game_selected);
        }
    ));

    // When we hit the "Create Map Prefab" button.
    app_ui.menu_bar_create_map_prefab_wh2.connect_activate(clone!(
        application,
        app_ui,
        pack_file_decoded,
        game_selected => move |_,_| {
            create_prefab(&application, &app_ui, &game_selected, &pack_file_decoded);
        }
    ));

    // When we hit the "Patch SiegeAI" button (Warhammer).
    app_ui.menu_bar_patch_siege_ai_wh.connect_activate(clone!(
        app_ui,
        pack_file_decoded => move |_,_| {
            patch_siege_ai(&app_ui, &pack_file_decoded);
        }
    ));

    // When we hit the "Generate Dependency Pack" button (Warhammer).
    app_ui.menu_bar_generate_dependency_pack_wh.connect_activate(clone!(
        game_selected,
        rpfm_path,
        app_ui => move |_,_| {
            generate_dependency_pack(&app_ui, &rpfm_path, &game_selected);
        }
    ));

    // When we hit the "Create Map Prefab" button (Warhammer).
    app_ui.menu_bar_create_map_prefab_wh.connect_activate(clone!(
        application,
        app_ui,
        pack_file_decoded,
        game_selected => move |_,_| {
            create_prefab(&application, &app_ui, &game_selected, &pack_file_decoded);
        }
    ));

    // When we hit the "Generate Dependency Pack" button (Attila).
    app_ui.menu_bar_generate_dependency_pack_att.connect_activate(clone!(
        app_ui,
        rpfm_path,
        game_selected => move |_,_| {
            generate_dependency_pack(&app_ui, &rpfm_path, &game_selected);
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
            check_updates(VERSION, Some(&app_ui.window), None);
        }
    ));

    // When we hit the "Check Schema Updates" button.
    app_ui.menu_bar_check_schema_updates.connect_activate(clone!(
        supported_games,
        game_selected,
        rpfm_path,
        schema,
        app_ui => move |_,_| {
            check_schema_updates(VERSION, &rpfm_path, &supported_games.borrow(), &game_selected, &schema, Some(&app_ui.window), None);
        }
    ));

    // When we hit the "Support me on Patreon" button.
    app_ui.menu_bar_open_patreon.connect_activate(move |_,_| {

        // I doubt GTK allows to put a LinkButton in the Menubar so... time to be creative.
        let link_button = LinkButton::new("https://www.patreon.com/RPFM");
        link_button.emit("activate-link", &[]).unwrap();
    });

    // When we hit the "About" button.
    app_ui.menu_bar_about.connect_activate(clone!(
        rpfm_path,
        app_ui => move |_,_| {
            show_about_window(VERSION, &rpfm_path, &app_ui.window);
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
                let rect = get_rect_for_popover(&app_ui.folder_tree_view, Some(button.get_position()));
                app_ui.folder_tree_view_context_menu.set_pointing_to(&rect);
                app_ui.folder_tree_view_context_menu.popup();
            }
            Inhibit(false)
        }
    ));

    // We check every action possible for the selected file when changing the cursor.
    app_ui.folder_tree_view.connect_cursor_changed(clone!(
        dependency_database,
        pack_file_decoded,
        schema,
        app_ui => move |_| {

            // Get the path of the selected thing.
            let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

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
                    app_ui.folder_tree_view_create_loc.set_enabled(false);
                    app_ui.folder_tree_view_create_db.set_enabled(false);
                    app_ui.folder_tree_view_create_text.set_enabled(false);
                    app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(false);
                },

                // If it's a folder...
                TreePathType::Folder(_) => {
                    app_ui.folder_tree_view_add_file.set_enabled(true);
                    app_ui.folder_tree_view_add_folder.set_enabled(true);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_create_loc.set_enabled(true);
                    app_ui.folder_tree_view_create_db.set_enabled(true);
                    app_ui.folder_tree_view_create_text.set_enabled(true);
                    app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(true);
                },

                // If it's the PackFile...
                TreePathType::PackFile => {
                    app_ui.folder_tree_view_add_file.set_enabled(true);
                    app_ui.folder_tree_view_add_folder.set_enabled(true);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(true);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(true);
                    app_ui.folder_tree_view_create_loc.set_enabled(true);
                    app_ui.folder_tree_view_create_db.set_enabled(true);
                    app_ui.folder_tree_view_create_text.set_enabled(true);
                    app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(true);
                },

                // If there is nothing selected...
                TreePathType::None => {
                    app_ui.folder_tree_view_add_file.set_enabled(false);
                    app_ui.folder_tree_view_add_folder.set_enabled(false);
                    app_ui.folder_tree_view_add_from_packfile.set_enabled(false);
                    app_ui.folder_tree_view_rename_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_delete_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_extract_packedfile.set_enabled(false);
                    app_ui.folder_tree_view_create_loc.set_enabled(false);
                    app_ui.folder_tree_view_create_db.set_enabled(false);
                    app_ui.folder_tree_view_create_text.set_enabled(false);
                    app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(false);
                },
            }

            // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables.
            if dependency_database.borrow().is_none() || schema.borrow().is_none() {
                app_ui.folder_tree_view_create_db.set_enabled(false);
                app_ui.folder_tree_view_mass_import_tsv_files.set_enabled(false);
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

            // Allow to select multiple files at the same time.
            file_chooser_add_file_to_packfile.set_select_multiple(true);

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
                                            show_dialog(&app_ui.window, false, error.cause());
                                            false
                                        }
                                    };

                                    // If we had success adding it...
                                    if success {

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                        // Update the TreeView to show the newly added PackedFile.
                                        update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::Add(tree_path.to_vec()),
                                            &TreePathType::None,
                                        );
                                    }
                                }

                                // If not, we get their tree_path like a normal file.
                                else {

                                    // Get his `TreeView` path.
                                    let tree_path = get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);

                                    // Try to add it to the PackFile.
                                    let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path.to_vec()) {
                                        Ok(_) => true,
                                        Err(error) => {
                                            show_dialog(&app_ui.window, false, error.cause());
                                            false
                                        }
                                    };

                                    // If we had success adding it...
                                    if success {

                                        // Set the mod as "Modified".
                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                        // Update the TreeView to show the newly added PackedFile.
                                        update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::Add(tree_path.to_vec()),
                                            &TreePathType::None,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    else {
                        return show_dialog(&app_ui.window, false, "MyMod base folder not configured.");
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
                            let tree_path = get_tree_path_from_pathbuf(path, &app_ui.folder_tree_selection, true);

                            // Try to add it to the PackFile.
                            let success = match packfile::add_file_to_packfile(&mut *pack_file_decoded.borrow_mut(), path, tree_path.to_vec()) {
                                Ok(_) => true,
                                Err(error) => {
                                    show_dialog(&app_ui.window, false, error.cause());
                                    false
                                }
                            };

                            // If we had success adding it...
                            if success {

                                // Set the mod as "Modified".
                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                // Update the TreeView to show the newly added PackedFile.
                                update_treeview(
                                    &app_ui.folder_tree_store,
                                    &*pack_file_decoded.borrow(),
                                    &app_ui.folder_tree_selection,
                                    TreeViewOperation::Add(tree_path.to_vec()),
                                    &TreePathType::None,
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

                // Allow to select multiple folders at the same time.
                file_chooser_add_folder_to_packfile.set_select_multiple(true);

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
                                                                    show_dialog(&app_ui.window, false, error.cause());
                                                                    false
                                                                }
                                                            };

                                                            // If we had success adding it...
                                                            if success {

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                // Update the TreeView to show the newly added PackedFile.
                                                                update_treeview(
                                                                    &app_ui.folder_tree_store,
                                                                    &*pack_file_decoded.borrow(),
                                                                    &app_ui.folder_tree_selection,
                                                                    TreeViewOperation::Add(tree_path.to_vec()),
                                                                    &TreePathType::None,
                                                                );
                                                            }
                                                        }

                                                        // If there is an error while removing the prefix...
                                                        Err(_) => show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                                    }
                                                }
                                            }

                                            // If there is an error while getting the files to add...
                                            Err(_) => show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
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
                                                                    show_dialog(&app_ui.window, false, error.cause());
                                                                    false
                                                                }
                                                            };

                                                            // If we had success adding it...
                                                            if success {

                                                                // Set the mod as "Modified".
                                                                set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                                // Update the TreeView to show the newly added PackedFile.
                                                                update_treeview(
                                                                    &app_ui.folder_tree_store,
                                                                    &*pack_file_decoded.borrow(),
                                                                    &app_ui.folder_tree_selection,
                                                                    TreeViewOperation::Add(tree_path.to_vec()),
                                                                    &TreePathType::None,
                                                                );
                                                            }
                                                        }

                                                        // If there is an error while removing the prefix...
                                                        Err(_) => show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                                    }
                                                }
                                            }

                                            // If there is an error while getting the files to add...
                                            Err(_) => show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
                                        }
                                    }
                                }
                            }
                        }
                        else {
                            return show_dialog(&app_ui.window, false, "MyMod base folder not configured.");
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
                                                            show_dialog(&app_ui.window, false, error.cause());
                                                            false
                                                        }
                                                    };

                                                    // If we had success adding it...
                                                    if success {

                                                        // Set the mod as "Modified".
                                                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                                                        // Update the TreeView to show the newly added PackedFile.
                                                        update_treeview(
                                                            &app_ui.folder_tree_store,
                                                            &*pack_file_decoded.borrow(),
                                                            &app_ui.folder_tree_selection,
                                                            TreeViewOperation::Add(tree_path.to_vec()),
                                                            &TreePathType::None,
                                                        );
                                                    }
                                                }

                                                // If there is an error while removing the prefix...
                                                Err(_) => show_dialog(&app_ui.window, false, format!("Error adding the following file to the PackFile:\n\n{:?}\n\nThe file's path doesn't start with {:?}", file_path, big_parent_prefix)),
                                            }
                                        }
                                    }

                                    // If there is an error while getting the files to add...
                                    Err(_) => show_dialog(&app_ui.window, false, "Error while getting the files to add to the PackFile."),
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
                    match packfile::open_packfile_with_bufreader(file_chooser_add_from_packfile.get_filename().unwrap()) {

                        // If the extra PackFile is valid...
                        Ok(result) => {

                            // Separate the result.
                            let pack_file_opened = result.0;
                            let mut buffer = Rc::new(RefCell::new(result.1));

                            // We create the "Exit" and "Copy" buttons.
                            let exit_button = Button::new_with_label("Exit \"Add file/folder from PackFile\" mode");
                            let copy_button = Button::new_with_label("<=");
                            exit_button.set_vexpand(false);
                            copy_button.set_hexpand(false);

                            // Paint the fucking button pink, because people keeps complaining they don't see it.
                            StyleContext::add_class(&copy_button.get_style_context().unwrap(), "suggested-action");

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
                            update_treeview(
                                &folder_tree_store_extra,
                                &*pack_file_decoded_extra.borrow(),
                                &folder_tree_view_extra.get_selection(),
                                TreeViewOperation::Build,
                                &TreePathType::None,
                            );

                            // We need to check here if the selected destination is not a file. Otherwise,
                            // we should disable the "Copy" button.
                            app_ui.folder_tree_selection.connect_changed(clone!(
                            copy_button,
                            pack_file_decoded => move |folder_tree_selection| {

                                    // Get his path.
                                    let tree_path = get_tree_path_from_selection(folder_tree_selection, true);

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
                                buffer,
                                pack_file_decoded,
                                pack_file_decoded_extra,
                                folder_tree_view_extra => move |_,_| {

                                    // Get his source & destination paths.
                                    let tree_path_source = get_tree_path_from_selection(&folder_tree_view_extra.get_selection(), true);
                                    let tree_path_destination = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                                    // Get the destination type.
                                    let selection_type = get_type_of_selected_tree_path(&tree_path_destination, &pack_file_decoded.borrow());

                                    // Try to add the PackedFile to the main PackFile.
                                    let success = match packfile::add_packedfile_to_packfile(
                                        &mut buffer.borrow_mut(),
                                        &*pack_file_decoded_extra.borrow(),
                                        &mut *pack_file_decoded.borrow_mut(),
                                        &tree_path_source,
                                        &tree_path_destination,
                                    ) {
                                        Ok(_) => true,
                                        Err(error) => {
                                            show_dialog(&app_ui.window, false, error.cause());
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
                                            .data.packed_files
                                            .iter()
                                            .filter(|x| x.path.starts_with(&source_prefix))
                                            .map(|x| x.path.to_vec())
                                            .collect();

                                        // Update the TreeView to show the newly added PackedFiles.
                                        update_treeview(
                                            &app_ui.folder_tree_store,
                                            &*pack_file_decoded.borrow(),
                                            &app_ui.folder_tree_selection,
                                            TreeViewOperation::AddFromPackFile(source_prefix.to_vec(), destination_prefix.to_vec(), path_list),
                                            &selection_type,
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
                                    display_help_tips(&app_ui.packed_file_data_display);

                                    Inhibit(false)
                                }
                            ));

                        }
                        Err(error) => show_dialog(&app_ui.window, false, error.cause()),
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
                        let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
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
                let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                // Get his type.
                let selection_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

                // And try to rename it.
                let success = match packfile::rename_packed_file(&mut *pack_file_decoded.borrow_mut(), &tree_path, new_name) {
                    Ok(_) => true,
                    Err(error) => {
                        show_dialog(&app_ui.window, false, error.cause());
                        false
                    }
                };

                // If we renamed the file/folder successfully...
                if success {

                    // Set the mod as "Modified".
                    set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                    // Rename whatever is selected (and his childs, if it have any) from the `TreeView`.
                    update_treeview(
                        &app_ui.folder_tree_store,
                        &*pack_file_decoded.borrow(),
                        &app_ui.folder_tree_selection,
                        TreeViewOperation::Rename(new_name.to_owned()),
                        &selection_type,
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

    // When we hit the "Create Loc File" button.
    app_ui.folder_tree_view_create_loc.connect_activate(clone!(
        dependency_database,
        pack_file_decoded,
        application,
        rpfm_path,
        schema,
        app_ui => move |_,_| {

            // We hide the context menu, then we get the selected file/folder, delete it and update the
            // TreeView. Pretty simple, actually.
            app_ui.folder_tree_view_context_menu.popdown();

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Build the "Create Loc File" window.
                show_create_packed_file_window(&application, &app_ui, &rpfm_path, &pack_file_decoded, PackedFileType::Loc, &dependency_database, &schema);
            }
        }
    ));

    // When we hit the "Create DB Table" button.
    app_ui.folder_tree_view_create_db.connect_activate(clone!(
        dependency_database,
        pack_file_decoded,
        application,
        rpfm_path,
        schema,
        app_ui => move |_,_| {

            // We hide the context menu, then we get the selected file/folder, delete it and update the
            // TreeView. Pretty simple, actually.
            app_ui.folder_tree_view_context_menu.popdown();

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Build the "Create DB Table" window.
                show_create_packed_file_window(&application, &app_ui, &rpfm_path, &pack_file_decoded, PackedFileType::DB, &dependency_database, &schema);
            }
        }
    ));

    // When we hit the "Create Text File" button.
    app_ui.folder_tree_view_create_text.connect_activate(clone!(
        dependency_database,
        pack_file_decoded,
        application,
        rpfm_path,
        schema,
        app_ui => move |_,_| {

            // We hide the context menu, then we get the selected file/folder, delete it and update the
            // TreeView. Pretty simple, actually.
            app_ui.folder_tree_view_context_menu.popdown();

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Build the "Create Text File" window.
                show_create_packed_file_window(&application, &app_ui, &rpfm_path, &pack_file_decoded, PackedFileType::Text, &dependency_database, &schema);
            }
        }
    ));

    // When we hit the "Mass-Import TSV Files" button.
    app_ui.folder_tree_view_mass_import_tsv_files.connect_activate(clone!(
        pack_file_decoded,
        application,
        rpfm_path,
        schema,
        app_ui => move |_,_| {

            // We hide the context menu, then we get the selected file/folder, delete it and update the
            // TreeView. Pretty simple, actually.
            app_ui.folder_tree_view_context_menu.popdown();

            // We only do something in case the focus is in the TreeView. This should stop problems with
            // the accels working everywhere.
            if app_ui.folder_tree_view.has_focus() {

                // Build the "Mass-Import TSV Files" window.
                show_tsv_mass_import_window(&application, &app_ui, &rpfm_path, &pack_file_decoded, &schema);
            }
        }
    ));

    // When we hit the "Delete file/folder" button.
    app_ui.folder_tree_view_delete_packedfile.connect_activate(clone!(
        app_ui,
        is_packedfile_opened,
        pack_file_decoded => move |_,_|{

            // We hide the context menu, then we get the selected file/folder, delete it and update the
            // TreeView. Pretty simple, actually.
            app_ui.folder_tree_view_context_menu.popdown();

            // If there is a PackedFile opened, we show a message with the explanation of why we can't
            // delete the selected file/folder.
            if *is_packedfile_opened.borrow() {
                show_dialog(&app_ui.window, false, "You can't delete a PackedFile/Folder while there is a PackedFile opened in the right side. Pls close it by clicking in a Folder/PackFile before trying to delete it again.")
            }

            // Otherwise, we continue the deletion process.
            else {

                // We only do something in case the focus is in the TreeView. This should stop problems with
                // the accels working everywhere.
                if app_ui.folder_tree_view.has_focus() {

                    // Get his `tree_path`.
                    let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

                    // Get his type.
                    let selection_type = get_type_of_selected_tree_path(&tree_path, &pack_file_decoded.borrow());

                    // Try to delete whatever is selected.
                    let success = match packfile::delete_from_packfile(&mut *pack_file_decoded.borrow_mut(), &tree_path) {
                        Ok(_) => true,
                        Err(error) => {
                            show_dialog(&app_ui.window, false, error.cause());
                            false
                        }
                    };

                    // If we succeed...
                    if success {

                        // Set the mod as "Modified".
                        set_modified(true, &app_ui.window, &mut *pack_file_decoded.borrow_mut());

                        // Remove whatever is selected (and his childs, if it have any) from the `TreeView`.
                        update_treeview(
                            &app_ui.folder_tree_store,
                            &*pack_file_decoded.borrow(),
                            &app_ui.folder_tree_selection,
                            TreeViewOperation::Delete,
                            &selection_type,
                        );
                    }
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
                let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);
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
                                Ok(result) => show_dialog(&app_ui.window, true, result),
                                Err(error) => show_dialog(&app_ui.window, false, error.cause())
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else {
                            return show_dialog(&app_ui.window, false, "MyMod base path not configured.");
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
                                return show_dialog(&app_ui.window, false, "You can't extract non-existent files.");
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
                                Ok(result) => show_dialog(&app_ui.window, true, result),
                                Err(error) => show_dialog(&app_ui.window, false, error.cause())
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
                    let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

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
                    let tree_path = get_tree_path_from_selection(&app_ui.folder_tree_selection, true);

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
        supported_games,
        pack_file_decoded,
        dependency_database,
        is_packedfile_opened,
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
                        else if packedfile_name.ends_with(".lua") ||
                                packedfile_name.ends_with(".xml") ||
                                packedfile_name.ends_with(".xml.shader") ||
                                packedfile_name.ends_with(".xml.material") ||
                                packedfile_name.ends_with(".variantmeshdefinition") ||
                                packedfile_name.ends_with(".environment") ||
                                packedfile_name.ends_with(".lighting") ||
                                packedfile_name.ends_with(".wsmodel") ||
                                packedfile_name.ends_with(".csv") ||
                                packedfile_name.ends_with(".tsv") ||
                                packedfile_name.ends_with(".inl") ||
                                packedfile_name.ends_with(".battle_speech_camera") ||
                                packedfile_name.ends_with(".bob") ||
                                //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                                packedfile_name.ends_with(".txt") { "TEXT" }

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
                            if let Err(error) = PackedFileLocTreeView::create_tree_view(
                                &application,
                                &app_ui,
                                &pack_file_decoded,
                                &index,
                                &is_packedfile_opened,
                                &settings.borrow()
                            ) { return show_dialog(&app_ui.window, false, error.cause()) };

                            // Tell the program there is an open PackedFile.
                            *is_packedfile_opened.borrow_mut() = true;
                        }

                        // If the file is a DB PackedFile...
                        "DB" => {
                            if let Err(error) = create_db_view(
                                &application,
                                &app_ui,
                                &rpfm_path,
                                &pack_file_decoded,
                                &index,
                                &is_packedfile_opened,
                                &schema,
                                &dependency_database,
                                &game_selected,
                                &supported_games,
                                &settings.borrow()
                            ) { return show_dialog(&app_ui.window, false, error.cause()) };

                            // Tell the program there is an open PackedFile.
                            *is_packedfile_opened.borrow_mut() = true;
                        }

                        // If it's a plain text file, we create a source view and try to get highlighting for
                        // his language, if it's an specific language file.
                        "TEXT" => {
                            if let Err(error) = create_text_view(
                                &app_ui,
                                &pack_file_decoded,
                                &index,
                                &is_packedfile_opened,
                            ) { return show_dialog(&app_ui.window, false, error.cause()) };

                            // Tell the program there is an open PackedFile.
                            *is_packedfile_opened.borrow_mut() = true;
                        }

                        // If it's an image it doesn't require any extra interaction. Just create the View
                        // and show the Image.
                        "IMAGE" => {
                            if let Err(error) = create_image_view(
                                &app_ui,
                                &pack_file_decoded,
                                &index
                            ) { return show_dialog(&app_ui.window, false, error.cause()) };
                        }

                        // If it's a rigidmodel, we decode it and take care of his update events.
                        "RIGIDMODEL" => {
                            if let Err(error) = PackedFileRigidModelDataView::create_data_view(
                                &app_ui,
                                &pack_file_decoded,
                                &index,
                                &is_packedfile_opened,
                            ) { return show_dialog(&app_ui.window, false, error.cause()) };

                            // Tell the program there is an open PackedFile.
                            *is_packedfile_opened.borrow_mut() = true;
                        }

                        // If we reach this point, the coding to implement this type of file is not done yet,
                        // so we ignore the file.
                        _ => {
                            display_help_tips(&app_ui.packed_file_data_display);
                        }
                    }
                }

                // If it's anything else, then we just show the "Tips" list.
                _ => display_help_tips(&app_ui.packed_file_data_display),
            }
        }
    }));

    // This allow us to open a PackFile by "Drag&Drop" it into the folder_tree_view.
    app_ui.folder_tree_view.connect_drag_data_received(clone!(
        app_ui,
        settings,
        schema,
        rpfm_path,
        mode,
        game_selected,
        supported_games,
        dependency_database,
        pack_file_decoded_extra,
        pack_file_decoded => move |_, _, _, _, selection_data, info, _| {

            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {

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
                            &mode,
                            &schema,
                            &supported_games.borrow(),
                            &game_selected,
                            &dependency_database,
                            &(false, None),
                            &pack_file_decoded,
                            &pack_file_decoded_extra
                        ) { show_dialog(&app_ui.window, false, error.cause()) };
                    }
                    _ => show_dialog(&app_ui.window, false, "This type of event is not yet used."),
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
            &mode,
            &schema,
            &supported_games.borrow(),
            &game_selected,
            &dependency_database,
            &(false, None),
            &pack_file_decoded,
            &pack_file_decoded_extra
        ) { show_dialog(&app_ui.window, false, error.cause()) };
    }
}

//-----------------------------------------------------------------------------
// From here, there is code that was in the build_ui function, but it was
// becoming a mess to maintain, and was needed to be split.
//-----------------------------------------------------------------------------

/// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending
/// on the situation.
fn open_packfile(
    pack_file_path: PathBuf,
    rpfm_path: &PathBuf,
    app_ui: &AppUI,
    settings: &Settings,
    mode: &Rc<RefCell<Mode>>,
    schema: &Rc<RefCell<Option<Schema>>>,
    supported_games: &[GameInfo],
    game_selected: &Rc<RefCell<GameSelected>>,
    dependency_database: &Rc<RefCell<Option<Vec<PackedFile>>>>,
    is_my_mod: &(bool, Option<String>),
    pack_file_decoded: &Rc<RefCell<PackFile>>,
    pack_file_decoded_extra: &Rc<RefCell<PackFile>>,
) -> Result<(), Error> {
    match packfile::open_packfile(pack_file_path.to_path_buf()) {
        Ok(pack_file_opened) => {

            // If there is no secondary PackFile opened using the "Data View" at the right side...
            if pack_file_decoded_extra.borrow().extra_data.file_name.is_empty() {

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
            update_treeview(
                &app_ui.folder_tree_store,
                &pack_file_decoded.borrow(),
                &app_ui.folder_tree_selection,
                TreeViewOperation::Build,
                &TreePathType::None,
            );

            // We choose the right option, depending on our PackFile.
            match pack_file_decoded.borrow().header.pack_file_type {
                0 => app_ui.menu_bar_change_packfile_type.change_state(&"boot".to_variant()),
                1 => app_ui.menu_bar_change_packfile_type.change_state(&"release".to_variant()),
                2 => app_ui.menu_bar_change_packfile_type.change_state(&"patch".to_variant()),
                3 => app_ui.menu_bar_change_packfile_type.change_state(&"mod".to_variant()),
                4 => app_ui.menu_bar_change_packfile_type.change_state(&"movie".to_variant()),
                _ => app_ui.menu_bar_change_packfile_type.change_state(&"other".to_variant()),
            }

            // Disable the "PackFile Management" actions.
            enable_packfile_actions(app_ui, game_selected, false);

            // If it's a "MyMod", we choose the game selected depending on his folder's name.
            if is_my_mod.0 {

                // Set `GameSelected` depending on the folder of the "MyMod".
                let game_name = is_my_mod.1.clone().unwrap();
                game_selected.borrow_mut().change_game_selected(&game_name, &settings.paths.game_paths.iter().filter(|x| x.game == game_name).map(|x| x.path.clone()).collect::<Option<PathBuf>>(), supported_games);
                app_ui.menu_bar_change_game_selected.change_state(&game_name.to_variant());

                // Set the current "Operational Mode" to `MyMod`.
                set_my_mod_mode(app_ui, mode, Some(pack_file_path));
            }

            // If it's not a "MyMod", we choose the new GameSelected depending on what the open mod id is.
            else {

                // Set `GameSelected` depending on the ID of the PackFile.
                match &*pack_file_decoded.borrow().header.id {
                    "PFH5" => {
                        game_selected.borrow_mut().change_game_selected("warhammer_2", &settings.paths.game_paths.iter().filter(|x| &x.game == "warhammer_2").map(|x| x.path.clone()).collect::<Option<PathBuf>>(), supported_games);
                        app_ui.menu_bar_change_game_selected.change_state(&"warhammer_2".to_variant());
                    },

                    "PFH4" | _ => {

                        // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila.
                        // In any other case, we select Attila by default.
                        match &*(app_ui.menu_bar_change_game_selected.get_state().unwrap().get::<String>().unwrap()) {
                            "warhammer" => {
                                game_selected.borrow_mut().change_game_selected("warhammer", &settings.paths.game_paths.iter().filter(|x| &x.game == "warhammer").map(|x| x.path.clone()).collect::<Option<PathBuf>>(), supported_games);
                                app_ui.menu_bar_change_game_selected.change_state(&"warhammer".to_variant());
                            }
                            "attila" | _ => {
                                game_selected.borrow_mut().change_game_selected("attila", &settings.paths.game_paths.iter().filter(|x| &x.game == "attila").map(|x| x.path.clone()).collect::<Option<PathBuf>>(), supported_games);
                                app_ui.menu_bar_change_game_selected.change_state(&"attila".to_variant());
                            }
                        }
                    },
                }

                // Change the `dependency_database` for that game.
                *dependency_database.borrow_mut() = match packfile::open_packfile(game_selected.borrow().game_dependency_packfile_path.to_path_buf()) {
                    Ok(data) => Some(data.data.packed_files),
                    Err(_) => None,
                };

                // Set the current "Operational Mode" to `Normal`.
                set_my_mod_mode(app_ui, mode, None);
            }

            // Enable the "PackFile Management" actions.
            enable_packfile_actions(app_ui, game_selected, true);

            // Try to load the Schema for this PackFile's game.
            *schema.borrow_mut() = Schema::load(rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.borrow().game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

            // Test to see if every DB Table can be decoded.
            // let mut counter = 0;
            // for i in pack_file_decoded.borrow().data.packed_files.iter() {
            //     if i.path.starts_with(&["db".to_owned()]) {
            //         if let Some(ref schema) = *schema.borrow() {
            //             if let Err(_) = packedfile::db::DB::read(&i.data, &i.path[1], &schema) {
            //                 match packedfile::db::DBHeader::read(&i.data, &mut 0) {
            //                     Ok(db_header) => {
            //                         if db_header.entry_count > 0 {
            //                             counter += 1;
            //                             println!("{}, {:?}", counter, i.path);
            //                         }
            //                     }
            //                     Err(_) => println!("Error in {:?}", i.path),
            //                 }
            //             }
            //         }
            //     }
            // }

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
    mode: &Rc<RefCell<Mode>>,
    schema: &Rc<RefCell<Option<Schema>>>,
    game_selected: &Rc<RefCell<GameSelected>>,
    supported_games: &Rc<RefCell<Vec<GameInfo>>>,
    dependency_database: &Rc<RefCell<Option<Vec<PackedFile>>>>,
    pack_file_decoded: &Rc<RefCell<PackFile>>,
    pack_file_decoded_extra: &Rc<RefCell<PackFile>>,
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
                                    game_folder_file.extension().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy() =="pack" {

                                    // That means our game_folder is a valid folder and it needs to be added to the menu.
                                    let mod_name = game_folder_file.file_name().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy().as_ref().to_owned();
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
                                        dependency_database,
                                        pack_file_decoded_extra,
                                        pack_file_decoded => move |_,_| {

                                            // If the current PackFile has been changed in any way, we pop up the "Are you sure?" message.
                                            if are_you_sure(&app_ui.window, pack_file_decoded.borrow().extra_data.is_modified, false) {

                                                // If we got confirmation...
                                                let pack_file_path = game_folder_file.to_path_buf();

                                                // Open the PackFile (or die trying it!).
                                                if let Err(error) = open_packfile(
                                                    pack_file_path,
                                                    &rpfm_path,
                                                    &app_ui,
                                                    &settings,
                                                    &mode,
                                                    &schema,
                                                    &supported_games.borrow(),
                                                    &game_selected,
                                                    &dependency_database,
                                                    &(true, Some(game_folder_name.borrow().to_owned())),
                                                    &pack_file_decoded,
                                                    &pack_file_decoded_extra
                                                ) { show_dialog(&app_ui.window, false, error.cause()) };
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
    pack_file_decoded: &Rc<RefCell<PackFile>>,
) {

    // First, we try to patch the PackFile. If there are no errors, we save the result in a tuple.
    // Then we check that tuple and, if it's a success, we save the PackFile and update the TreeView.
    let mut sucessful_patching = (false, String::new());
    match packfile::patch_siege_ai(&mut *pack_file_decoded.borrow_mut()) {
        Ok(result) => sucessful_patching = (true, result),
        Err(error) => show_dialog(&app_ui.window, false, error.cause())
    }
    if sucessful_patching.0 {
        let mut success = false;
        match packfile::save_packfile( &mut *pack_file_decoded.borrow_mut(), None) {
            Ok(_) => {
                success = true;
                show_dialog(&app_ui.window, true, format!("{}\n\n{}", sucessful_patching.1, "PackFile succesfully saved."));
            },
            Err(error) => show_dialog(&app_ui.window, false, error.cause())
        }

        // If it succeed...
        if success {

            // Clear the `TreeView` before updating it (fixes CTD with borrowed PackFile).
            app_ui.folder_tree_store.clear();

            // TODO: Make this update, not rebuild.
            // Rebuild the `TreeView`.
            update_treeview(
                &app_ui.folder_tree_store,
                &*pack_file_decoded.borrow(),
                &app_ui.folder_tree_selection,
                TreeViewOperation::Build,
                &TreePathType::None,
            );
        }
    }
}

/// This function serves as a common function for all the "Generate Dependency Pack" buttons from "Special Stuff".
fn generate_dependency_pack(
    app_ui: &AppUI,
    rpfm_path: &PathBuf,
    game_selected: &Rc<RefCell<GameSelected>>,
) {

    // Get the data folder of game_selected and try to create our dependency PackFile.
    match game_selected.borrow().game_data_path {
        Some(ref path) => {
            let mut data_pack_path = path.to_path_buf();
            data_pack_path.push("data.pack");
            match packfile::open_packfile(data_pack_path) {
                Ok(ref mut data_packfile) => {
                    data_packfile.data.packed_files.retain(|packed_file| packed_file.path.starts_with(&["db".to_owned()]));
                    data_packfile.header.packed_file_count = data_packfile.data.packed_files.len() as u32;

                    // Just in case the folder doesn't exists, we try to create it.
                    let mut dep_packs_path = rpfm_path.clone();
                    dep_packs_path.push("dependency_packs");

                    match DirBuilder::new().create(&dep_packs_path) { Ok(_) | Err(_) => {}, }

                    let pack_file_path = game_selected.borrow().game_dependency_packfile_path.to_path_buf();
                    match packfile::save_packfile(data_packfile, Some(pack_file_path)) {
                        Ok(_) => show_dialog(&app_ui.window, true, "Dependency pack created. Remember to re-create it if you update the game ;)."),
                        Err(error) => show_dialog(&app_ui.window, false, format_err!("Error: generated dependency pack couldn't be saved. {:?}", error)),
                    }
                }
                Err(_) => show_dialog(&app_ui.window, false, "Error: data.pack couldn't be open.")
            }
        },
        None => show_dialog(&app_ui.window, false, "Error: data path of the game not found.")
    }
}

/// This function serves as a common function to all the "Create Prefab" buttons from "Special Stuff".
fn create_prefab(
    application: &Application,
    app_ui: &AppUI,
    game_selected: &Rc<RefCell<GameSelected>>,
    pack_file_decoded: &Rc<RefCell<PackFile>>,
) {
    // Create the list of PackedFiles to "move".
    let mut prefab_catchments: Vec<usize> = vec![];

    // For each PackedFile...
    for (index, packed_file) in pack_file_decoded.borrow().data.packed_files.iter().enumerate() {

        // If it's in the exported map's folder...
        if packed_file.path.starts_with(&["terrain".to_owned(), "tiles".to_owned(), "battle".to_owned(), "_assembly_kit".to_owned()]) {

            // Get his name.
            let packed_file_name = packed_file.path.last().unwrap();

            // If it's one of the exported layers...
            if packed_file_name.starts_with("catchment") && packed_file_name.ends_with(".bin") {

                // Add it to the list.
                prefab_catchments.push(index);
            }
        }
    }

    // If we found at least one catchment PackedFile...
    if !prefab_catchments.is_empty() {

        // Disable the main window, so the user can't do anything until all the prefabs are processed.
        app_ui.window.set_sensitive(false);

        // We create a "New Prefabs" window.
        NewPrefabWindow::create_new_prefab_window(
            &app_ui,
            application,
            game_selected,
            pack_file_decoded,
            &prefab_catchments
        );
    }

    // If there are not suitable PackedFiles...
    else { show_dialog(&app_ui.window, false, "There are no catchment PackedFiles in this PackFile."); }
}

/// This function concatenates the last two messages of the status_bar and shows them like one.
fn concatenate_check_update_messages(status_bar: &Statusbar) {

    // Get the ID of all messages passed to the status_bar with the helper function.
    let context_id = status_bar.get_context_id("Yekaterina");

    // Get the current text, if any.
    let current_text = status_bar.get_message_area().unwrap().get_children()[0].clone().downcast::<Label>().unwrap().get_text().unwrap();

    // Remove it from the status_bar.
    status_bar.pop(context_id);

    // Get the new current text, if any.
    let old_text = status_bar.get_message_area().unwrap().get_children()[0].clone().downcast::<Label>().unwrap().get_text().unwrap();

    // Concatenate both texts and push them.
    let new_text = format!("{} {}", old_text, current_text);
    status_bar.push(context_id, &new_text);
}
*/
/// This struct contains all the "Special Stuff" Actions, so we can pass all of them to functions at once.
#[derive(Copy, Clone)]
pub struct AppUI {

    //-------------------------------------------------------------------------------//
    // Big stuff.
    //-------------------------------------------------------------------------------//
    pub window: *mut MainWindow,
    pub folder_tree_view: *mut TreeView,
    pub folder_tree_model: *mut StandardItemModel,
    pub packed_file_layout: *mut GridLayout,

    //-------------------------------------------------------------------------------//
    // "PackFile" menu.
    //-------------------------------------------------------------------------------//

    // Menus.
    pub new_packfile: *mut Action,
    pub open_packfile: *mut Action,
    pub save_packfile: *mut Action,
    pub save_packfile_as: *mut Action,
    pub preferences: *mut Action,
    pub quit: *mut Action,

    // "Change PackFile Type" submenu.
    pub change_packfile_type_boot: *mut Action,
    pub change_packfile_type_release: *mut Action,
    pub change_packfile_type_patch: *mut Action,
    pub change_packfile_type_mod: *mut Action,
    pub change_packfile_type_movie: *mut Action,
    pub change_packfile_type_other: *mut Action,

    // Action Group for the submenu.
    pub change_packfile_type_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // "Game Selected" menu.
    //-------------------------------------------------------------------------------//

    pub warhammer_2: *mut Action,
    pub warhammer: *mut Action,
    pub attila: *mut Action,

    pub game_selected_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // "Special Stuff" menu.
    //-------------------------------------------------------------------------------//

    // Warhammer 2's actions.
    pub wh2_generate_dependency_pack: *mut Action,
    pub wh2_patch_siege_ai: *mut Action,
    pub wh2_create_prefab: *mut Action,

    // Warhammer's actions.
    pub wh_generate_dependency_pack: *mut Action,
    pub wh_patch_siege_ai: *mut Action,
    pub wh_create_prefab: *mut Action,

    // Attila's actions.
    pub att_generate_dependency_pack: *mut Action,

    //-------------------------------------------------------------------------------//
    // "About" menu.
    //-------------------------------------------------------------------------------//
    pub about_qt: *mut Action,
    pub about_rpfm: *mut Action,
    pub patreon_link: *mut Action,
    pub check_updates: *mut Action,
    pub check_schema_updates: *mut Action,
}

/// This struct will hold all the MyMod-related stuff we have to recreate from time to time.
#[derive(Copy, Clone)]
struct MyModStuff {
    pub new_mymod: *mut Action,
    pub delete_selected_mymod: *mut Action,
    pub install_mymod: *mut Action,
    pub uninstall_mymod: *mut Action,
}

/// This struct holds all the Slots related to the "MyMod" Menu, as otherwise they'll die before we
/// press their buttons and do nothing.
struct MyModSlots {
    pub new_mymod: SlotBool<'static>,
    pub delete_selected_mymod: SlotBool<'static>,
    pub install_mymod: SlotBool<'static>,
    pub uninstall_mymod: SlotBool<'static>,
    pub open_mymod: Vec<SlotBool<'static>>,
}

/// Main function.
fn main() {

    // Create the application...
    Application::create_and_exit(|app| {

        //---------------------------------------------------------------------------------------//
        // Preparing the Program...
        //---------------------------------------------------------------------------------------//

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

        // We load the list of Supported Games here.
        // TODO: Move this to a const when const fn reach stable in Rust.
        let supported_games = GameInfo::new();

        // Create the channels to communicate the threads. The channels are:
        // - `sender_rust, receiver_qt`: used for returning info from the background thread, serialized in Vec<u8>.
        // - `sender_qt, receiver_rust`: used for sending the current action to the background thread.
        // - `sender_qt_data, receiver_rust_data`: used for sending the data to the background thread.
        //   The data sended and received in the last one should be always be serialized into Vec<u8>.
        let (sender_rust, receiver_qt) = channel();
        let (sender_qt, receiver_rust) = channel();
        let (sender_qt_data, receiver_rust_data) = channel();

        // Create the background thread.
        thread::spawn(clone!(rpfm_path => move || { background_loop(&rpfm_path, sender_rust, receiver_rust, receiver_rust_data); }));

        //---------------------------------------------------------------------------------------//
        // Creating the UI...
        //---------------------------------------------------------------------------------------//

        // Set the RPFM Icon.
        let icon = Icon::new(&QString::from_std_str(format!("{}/img/rpfm.png", rpfm_path.to_string_lossy())));
        Application::set_window_icon(&icon);

        // Create the main window of the program.
        let mut window = MainWindow::new();
        window.set_window_title(&QString::from_std_str("Rusted PackFile Manager"));

        // Create a Central Widget and populate it.
        let mut central_widget = Widget::new();
        let mut central_layout = GridLayout::new();

        unsafe { central_widget.set_layout(central_layout.static_cast_mut()); }
        unsafe { window.set_central_widget(central_widget.as_mut_ptr()); }

        // Create the layout for the Central Widget.
        let mut central_splitter = Splitter::new(());
        unsafe { central_layout.add_widget((central_splitter.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }

        // Create the TreeView.
        let mut folder_tree_view = TreeView::new();
        let mut folder_tree_model = StandardItemModel::new(());
        unsafe { folder_tree_view.set_model(folder_tree_model.static_cast_mut()); }
        folder_tree_view.set_header_hidden(true);

        // Create the right-side Grid.
        let mut packed_file_view = Widget::new();
        let mut packed_file_layout = GridLayout::new();
        unsafe { packed_file_view.set_layout(packed_file_layout.static_cast_mut()); }

        // Add the corresponding widgets to the layout.
        unsafe { central_splitter.add_widget(folder_tree_view.static_cast_mut()); }
        unsafe { central_splitter.add_widget(packed_file_view.as_mut_ptr()); }

        // MenuBar at the top of the Window.
        let mut menu_bar = &window.menu_bar();

        // StatusBar at the bottom of the Window.
        let mut status_bar = window.status_bar();

        // Top MenuBar menus.
        let menu_bar_packfile;
        let menu_bar_mymod;
        let menu_bar_game_seleted;
        let menu_bar_special_stuff;
        let menu_bar_about;
        unsafe { menu_bar_packfile = menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&PackFile")); }
        unsafe { menu_bar_mymod = menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&MyMod")); }
        unsafe { menu_bar_game_seleted = menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&Game Selected")); }
        unsafe { menu_bar_special_stuff = menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&Special Stuff")); }
        unsafe { menu_bar_about = menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&About")); }

        // Submenus.
        let menu_change_packfile_type = Menu::new(&QString::from_std_str("&Change PackFile Type")).into_raw();

        let menu_warhammer_2;
        let menu_warhammer;
        let menu_attila;
        unsafe { menu_warhammer_2 = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer 2")); }
        unsafe { menu_warhammer = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer")); }
        unsafe { menu_attila = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Attila")); }

        // Da monsta.
        let app_ui;
        unsafe {
            app_ui = AppUI {

                //-------------------------------------------------------------------------------//
                // Big stuff.
                //-------------------------------------------------------------------------------//
                window: window.into_raw(),
                folder_tree_view: folder_tree_view.into_raw(),
                folder_tree_model: folder_tree_model.into_raw(),
                packed_file_layout: packed_file_layout.into_raw(),

                //-------------------------------------------------------------------------------//
                // "PackFile" menu.
                //-------------------------------------------------------------------------------//

                // Menús.
                new_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&New PackFile")),
                open_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Open PackFile")),
                save_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Save PackFile")),
                save_packfile_as: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("Save PackFile &As...")),
                preferences: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Preferences")),
                quit: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Quit")),

                // "Change PackFile Type" submenu.
                change_packfile_type_boot: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Boot")),
                change_packfile_type_release: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Release")),
                change_packfile_type_patch: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Patch")),
                change_packfile_type_mod: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Mod")),
                change_packfile_type_movie: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("Mo&vie")),
                change_packfile_type_other: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Other")),

                // Action Group for the submenu.
                change_packfile_type_group: ActionGroup::new(menu_change_packfile_type.as_mut().unwrap().static_cast_mut()).into_raw(),

                //-------------------------------------------------------------------------------//
                // "Game Selected" menu.
                //-------------------------------------------------------------------------------//

                warhammer_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Warhammer 2")),
                warhammer: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Warhammer")),
                attila: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Attila")),

                game_selected_group: ActionGroup::new(menu_bar_game_seleted.as_mut().unwrap().static_cast_mut()).into_raw(),

                //-------------------------------------------------------------------------------//
                // "Special Stuff" menu.
                //-------------------------------------------------------------------------------//

                // Warhammer 2's actions.
                wh2_generate_dependency_pack: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Generate Dependency Pack")),
                wh2_patch_siege_ai: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
                wh2_create_prefab: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),

                // Warhammer's actions.
                wh_generate_dependency_pack: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Generate Dependency Pack")),
                wh_patch_siege_ai: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
                wh_create_prefab: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),

                // Attila's actions.
                att_generate_dependency_pack: menu_attila.as_mut().unwrap().add_action(&QString::from_std_str("&Generate Dependency Pack")),

                //-------------------------------------------------------------------------------//
                // "About" menu.
                //-------------------------------------------------------------------------------//
                about_qt: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("About &Qt")),
                about_rpfm: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&About RPFM")),
                patreon_link: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&Support me on Patreon")),
                check_updates: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&Check Updates")),
                check_schema_updates: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("Check Schema &Updates")),
            };
        }

        // The "Change PackFile Type" submenu should be an ActionGroup.
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_boot); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_release); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_patch); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_mod); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_movie); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_other); }
        unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checkable(true); }

        // The "Game Selected" Menu should be an ActionGroup.
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.attila); }
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.attila.as_mut().unwrap().set_checkable(true); }

        // Put the Submenus and separators in place.
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_menu(app_ui.preferences, menu_change_packfile_type); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }

        //---------------------------------------------------------------------------------------//
        // Preparing initial state of the Main Window...
        //---------------------------------------------------------------------------------------//

        // Put the stuff we need to move to the slots in Rc<Refcell<>>, so we can clone it without issues.
        let receiver_qt = Rc::new(RefCell::new(receiver_qt));
        let is_modified = Rc::new(RefCell::new(false));
        let mymod_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let mode = Rc::new(RefCell::new(Mode::Normal));

        // Display the basic tips by default.
        display_help_tips(&app_ui);

        // Get the Game Selected.
        sender_qt.send("get_game_selected").unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

        // Change the Game Selected in the UI.
        match &*game_selected.game {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checked(true); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().set_checked(true); }
            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().set_checked(true); }
        }

        // Build the entire "MyMod" Menu.
        let result = build_my_mod_menu(
            sender_qt.clone(),
            &sender_qt_data,
            receiver_qt.clone(),
            app_ui.clone(),
            &menu_bar_mymod,
            is_modified.clone(),
            mode.clone(),
            supported_games.to_vec(),
            mymod_menu_needs_rebuild.clone(),
        );

        let mymod_stuff = Rc::new(RefCell::new(result.0));
        let mymod_stuff_slots = Rc::new(RefCell::new(result.1));

        // Disable the actions available for the PackFile from the `MenuBar`.
        enable_packfile_actions(&app_ui, &game_selected, false);

        // Set the current "Operational Mode" to `Normal`.
        set_my_mod_mode(&mymod_stuff, &mode, None);

        // Get the settings.
        sender_qt.send("get_settings").unwrap();
        let settings = receiver_qt.borrow().recv().unwrap().unwrap();
        let settings: Settings = serde_json::from_slice(&settings).unwrap();

        // If there is a "MyMod" path set in the settings...
        if let Some(ref path) = settings.paths.my_mods_base_path {

            // And it's a valid directory, enable the "New MyMod" button.
            if path.is_dir() { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(true); }}

            // Otherwise, disable it.
            else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
        }

        // Otherwise, disable it.
        else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}

        //---------------------------------------------------------------------------------------//
        // What should happend when we press buttons and stuff...
        //---------------------------------------------------------------------------------------//

        //-----------------------------------------------------//
        // "Game Selected" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "Change Game Selected" action.
        let slot_change_game_selected = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            supported_games,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the new Game Selected.
                let mut new_game_selected;
                unsafe { new_game_selected = QString::to_std_string(&app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()); }

                // Remove the '&' from the game's name, and get his folder name.
                if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
                let new_game_selected_folder_name = supported_games.iter().filter(|x| x.display_name == new_game_selected).map(|x| x.folder_name.to_owned()).collect::<String>();

                // Change the Game Selected in the Background Thread.
                sender_qt.send("set_game_selected").unwrap();
                sender_qt_data.send(serde_json::to_vec(&new_game_selected_folder_name).map_err(From::from)).unwrap();

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Until we receive a response from the worker thread...
                loop {

                    // When we finally receive the data...
                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                        // Get the (GameSelected, isthereapackfileopen) from the other thread.
                        let response = data.unwrap();
                        let response: (GameSelected, bool) = serde_json::from_slice(&response).unwrap();

                        // If we have a PackFile opened....
                        if !response.1 {

                            // Re-enable the "PackFile Management" actions, so the "Special Stuff" menu gets updated properly.
                            enable_packfile_actions(&app_ui, &response.0, false);
                            enable_packfile_actions(&app_ui, &response.0, true);

                            // Set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                            set_my_mod_mode(&mymod_stuff, &mode, None);
                        }

                        // Stop the loop.
                        break;
                    }

                    // Keep the UI responsive.
                    event_loop.process_events(());

                    // Wait a bit to not saturate a CPU core.
                    thread::sleep(Duration::from_millis(50));
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // "Game Selected" Menu Actions.
        unsafe { app_ui.warhammer_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.warhammer.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.attila.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }

        //-----------------------------------------------------//
        // "PackFile" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "New PackFile" action.
        let slot_new_packfile = SlotBool::new(clone!(
            is_modified,
            mymod_stuff,
            mode,
            sender_qt,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&is_modified, false) {

                    // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                    purge_them_all(&app_ui);

                    // Show the "Tips".
                    display_help_tips(&app_ui);

                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                    let mut event_loop = EventLoop::new();

                    // Tell the Background Thread to create a new PackFile.
                    sender_qt.send("new_packfile").unwrap();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Until we receive a response from the worker thread...
                    loop {

                        // When we finally receive the data of the PackFile...
                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                            // Unwrap the data.
                            let data = data.unwrap();

                            // Deserialize it (name of the packfile, paths of the PackedFiles).
                            let data: (&str, Vec<Vec<String>>, u32) = serde_json::from_slice(&data).unwrap();

                            // Update the TreeView.
                            update_treeview(
                                &app_ui,
                                (&data.0, data.1.to_vec()),
                                TreeViewOperation::Build,
                            );

                            // We choose the right option, depending on our PackFile (In this case, it's usually mod).
                            match data.2 {
                                0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                                1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                                2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                                3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                                4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                                _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                            }

                            // Stop the loop.
                            break;
                        }

                        // Keep the UI responsive.
                        event_loop.process_events(());

                        // Wait a bit to not saturate a CPU core.
                        thread::sleep(Duration::from_millis(50));
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Set the new mod as "Not modified".
                    *is_modified.borrow_mut() = set_modified(false, &app_ui);

                    // Get the Game Selected.
                    sender_qt.send("get_game_selected").unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let game_selected = serde_json::from_slice(&response).unwrap();

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &game_selected, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    set_my_mod_mode(&mymod_stuff, &mode, None);
                }
            }
        ));

        // What happens when we trigger the "Open PackFile" action.
        let slot_open_packfile = SlotBool::new(clone!(
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&is_modified, false) {

                    // Create the FileDialog to get the PackFile to open.
                    let mut file_dialog;
                    unsafe { file_dialog = FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Open PackFile"),
                    )); }

                    // Filter it so it only shows PackFiles.
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));

                    // Get the Game Selected.
                    sender_qt.send("get_game_selected").unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                    // In case we have a default path for the Game Selected, we use it as base path for opening files.
                    if let Some(ref path) = game_selected.game_data_path {

                        // We check that actually exists before setting it.
                        if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(&path.to_string_lossy().as_ref().to_owned())); }
                    }

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Get the path of the selected file and turn it in a Rust's PathBuf.
                        let mut path: PathBuf = PathBuf::new();
                        let path_qt = file_dialog.selected_files();
                        for index in 0..path_qt.size() { path.push(path_qt.at(index).to_std_string()); }

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            path,
                            &app_ui,
                            &mymod_stuff,
                            &is_modified,
                            &mode,
                            "",
                        ) {
                            show_dialog(&app_ui, false, format!("Error while opening the PackFile:\n\n{}", error.cause()));
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let slot_save_packfile = SlotBool::new(clone!(
            is_modified,
            mode,
            sender_qt,
            receiver_qt => move |_| {

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Tell the Background Thread to create a new PackFile.
                sender_qt.send("save_packfile").unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Until we receive a response from the worker thread...
                loop {

                    // When we finally receive the data...
                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                        // Check what the result of the saving process was.
                        match data {

                            // In case of success, show a dialog saying it, and set the mod as "Not Modified".
                            Ok(_) => {
                                *is_modified.borrow_mut() = set_modified(false, &app_ui);
                                show_dialog(&app_ui, true, "PackFile successfully saved.");
                            }

                            // In case of error, we can have two results.
                            Err(error) => {

                                // If the error message is empty, we have no original file, so we trigger a "Save PackFile As" action.
                                if error.cause().to_string().is_empty() { unsafe { Action::trigger(app_ui.save_packfile_as.as_mut().unwrap()); } }

                                // Otherwise, it's an error, so we report it.
                                else { show_dialog(&app_ui, false, format!("Error while saving the PackFile:\n\n{}", error.cause())); }
                            }
                        }

                        // Stop the loop.
                        break;
                    }

                    // Keep the UI responsive.
                    event_loop.process_events(());

                    // Wait a bit to not saturate a CPU core.
                    thread::sleep(Duration::from_millis(50));
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Save PackFile As" action.
        let slot_save_packfile_as = SlotBool::new(clone!(
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Get the Game Selected.
                sender_qt.send("get_game_selected").unwrap();
                let response = receiver_qt.borrow().recv().unwrap().unwrap();
                let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                // Tell the Background Thread to create a new PackFile.
                sender_qt.send("save_packfile_as").unwrap();

                // Get the confirmation that is editable, or an error.
                let confirmation = receiver_qt.borrow().recv().unwrap();
                match confirmation {

                    // If we got confirmation, we ask for a Path to write it.
                    Ok(extra_data) => {

                        // Get the extra data of the PackFile.
                        let extra_data: PackFileExtraData = serde_json::from_slice(&extra_data).unwrap();

                        // Create the FileDialog to get the PackFile to open.
                        let mut file_dialog;
                        unsafe { file_dialog = FileDialog::new_unsafe((
                            app_ui.window as *mut Widget,
                            &QString::from_std_str("Save PackFile"),
                        )); }

                        // Set it to save mode.
                        file_dialog.set_accept_mode(qt_widgets::file_dialog::AcceptMode::Save);

                        // Filter it so it only shows PackFiles.
                        file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));

                        // Ask for confirmation in case of overwrite.
                        file_dialog.set_confirm_overwrite(true);

                        // Set the default suffix to ".pack", in case we forgot to write it.
                        file_dialog.set_default_suffix(&QString::from_std_str("pack"));

                        // Set the current name of the PackFile as default name.
                        file_dialog.select_file(&QString::from_std_str(&extra_data.file_name));

                        // If we are saving an existing PackFile with another name, we start in his current path.
                        if extra_data.file_path.is_file() {
                            let mut path = extra_data.file_path.to_path_buf();
                            path.pop();
                            file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
                        }

                        // In case we have a default path for the Game Selected and that path is valid,
                        // we use his data folder as base path for saving our PackFile.
                        else if let Some(ref path) = game_selected.game_data_path {

                            // We check it actually exists before setting it.
                            if path.is_dir() {
                                file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
                            }
                        }

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Path we choose to save the file.
                            let mut path: PathBuf = PathBuf::new();
                            let path_qt = file_dialog.selected_files();
                            for index in 0..path_qt.size() { path.push(path_qt.at(index).to_std_string()); }

                            // Pass it to the worker thread.
                            sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Until we receive a response from the worker thread...
                            loop {

                                // When we finally receive the data...
                                if let Ok(data) = receiver_qt.borrow().try_recv() {

                                    // Check what the result of the saving process was.
                                    match data {

                                        // In case of success...
                                        Ok(_) => {

                                            // Set the mod as "Not Modified".
                                            *is_modified.borrow_mut() = set_modified(false, &app_ui);

                                            // Get the Selection Model and the Model Index of the PackFile's Cell.
                                            let selection_model;
                                            let model_index;
                                            unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                                            unsafe { model_index = app_ui.folder_tree_model.as_ref().unwrap().index((0, 0)); }

                                            // Select the PackFile's Cell with a "Clear & Select".
                                            unsafe { selection_model.as_mut().unwrap().select((&model_index, Flags::from_int(3))); }

                                            // Rename it with the new name.
                                            update_treeview(
                                                &app_ui,
                                                ("", vec![]),
                                                TreeViewOperation::Rename(path.file_name().unwrap().to_string_lossy().as_ref().to_owned()),
                                            );

                                            // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                                            set_my_mod_mode(&mymod_stuff, &mode, None);

                                            // Report success.
                                            show_dialog(&app_ui, true, "PackFile successfully saved.");
                                        }

                                        // In case of error, we report it.
                                        Err(error) => show_dialog(&app_ui, false, format!("Error while saving the PackFile:\n\n{}", error.cause())),
                                    }

                                    // Stop the loop.
                                    break;
                                }

                                // Keep the UI responsive.
                                event_loop.process_events(());

                                // Wait a bit to not saturate a CPU core.
                                thread::sleep(Duration::from_millis(50));
                            }

                            // Re-enable the Main Window.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                        }

                        // Otherwise, we take it as we canceled the save in some way, so we tell the
                        // Background Loop to stop waiting.
                        else { sender_qt_data.send(Err(format_err!(""))).unwrap(); }
                    }

                    // If we got an error, this is not an editable file, so we report it.
                    Err(error) => show_dialog(&app_ui, false, error.cause()),
                }
            }
        ));

        // What happens when we trigger the "Change PackFile Type" action.
        let slot_change_packfile_type = SlotBool::new(clone!(
            is_modified,
            sender_qt,
            sender_qt_data => move |_| {

                // Get the currently selected PackFile's Type.
                let packfile_type;
                unsafe { packfile_type = match &*QString::to_std_string(&app_ui.change_packfile_type_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()) {
                    "&Boot" => 0,
                    "&Release" => 1,
                    "&Patch" => 2,
                    "&Mod" => 3,
                    "Mo&vie" => 4,
                    _ => 99,
                }; }

                // Send the type to the Background Thread.
                sender_qt.send("set_packfile_type").unwrap();
                sender_qt_data.send(serde_json::to_vec(&packfile_type).map_err(From::from)).unwrap();

                // Set the mod as "Modified".
                *is_modified.borrow_mut() = set_modified(true, &app_ui);
            }
        ));

        // What happens when we trigger the "Preferences" action.
        let slot_preferences = SlotBool::new(clone!(
            mode,
            supported_games,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            mymod_stuff,
            mymod_stuff_slots,
            mymod_menu_needs_rebuild,
            is_modified,
            rpfm_path => move |_| {

                // Request the current Settings.
                sender_qt.send("get_settings").unwrap();

                let settings_encoded = receiver_qt.borrow().recv().unwrap().unwrap();
                let old_settings: Settings = serde_json::from_slice(&settings_encoded).unwrap();

                // Create the Settings Dialog. If we got new settings...
                if let Some(settings) = SettingsDialog::create_settings_dialog(&app_ui, &rpfm_path, old_settings.clone(), &supported_games) {

                    // Send the signal to save them.
                    sender_qt.send("set_settings").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&settings).map_err(From::from)).unwrap();

                    // Check if we were able to save them,
                    match receiver_qt.borrow().recv().unwrap() {

                        // If we were successful..
                        Ok(_) => {

                            // If we changed the "MyMod's Folder" path...
                            if settings.paths.my_mods_base_path != old_settings.paths.my_mods_base_path {

                                // We disable the "MyMod" mode, but leave the PackFile open, so the user doesn't lose any unsaved change.
                                set_my_mod_mode(&mymod_stuff, &mode, None);

                                // Then set it to recreate the "MyMod" submenu next time we try to open it.
                                *mymod_menu_needs_rebuild.borrow_mut() = true;
                            }

                            // If there is a "MyMod" path set in the settings...
                            if let Some(ref path) = settings.paths.my_mods_base_path {

                                // And it's a valid directory, enable the "New MyMod" button.
                                if path.is_dir() { unsafe { mymod_stuff.borrow_mut().new_mymod.as_mut().unwrap().set_enabled(true); }}

                                // Otherwise, disable it.
                                else { unsafe { mymod_stuff.borrow_mut().new_mymod.as_mut().unwrap().set_enabled(false); }}
                            }

                            // Otherwise, disable it.
                            else { unsafe { mymod_stuff.borrow_mut().new_mymod.as_mut().unwrap().set_enabled(false); }}

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // update the current `GameSelected`.
                            let new_game_paths = settings.paths.game_paths.clone();
                            let game_paths = new_game_paths.iter().zip(old_settings.paths.game_paths.iter());
                            let games_with_changed_paths = game_paths.filter(|x| x.0.path != x.1.path).map(|x| x.0.game.to_owned()).collect::<Vec<String>>();

                            // If our current `GameSelected` is in the `games_with_changed_paths` list...
                            if games_with_changed_paths.contains(&game_selected.game) {

                                // Re-select the same game, so `GameSelected` update his paths.
                                unsafe { Action::trigger(app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap()); }
                            }
                        }

                        // If there was an error while saving them, report it.
                        Err(error) => show_dialog(&app_ui, false, format!("Error while saving the Settings:\n\n{}", error.cause())),
                    }
                }
            }
        ));

        // What happens when we trigger the "Quit" action.
        let slot_quit = SlotBool::new( |_| { unsafe { app_ui.window.as_mut().unwrap().close(); }});

        // "PackFile" Menu Actions.
        unsafe { app_ui.new_packfile.as_ref().unwrap().signals().triggered().connect(&slot_new_packfile); }
        unsafe { app_ui.open_packfile.as_ref().unwrap().signals().triggered().connect(&slot_open_packfile); }
        unsafe { app_ui.save_packfile.as_ref().unwrap().signals().triggered().connect(&slot_save_packfile); }
        unsafe { app_ui.save_packfile_as.as_ref().unwrap().signals().triggered().connect(&slot_save_packfile_as); }

        unsafe { app_ui.change_packfile_type_boot.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_release.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_patch.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_mod.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_movie.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_other.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }

        unsafe { app_ui.preferences.as_ref().unwrap().signals().triggered().connect(&slot_preferences); }
        unsafe { app_ui.quit.as_ref().unwrap().signals().triggered().connect(&slot_quit); }

        //-----------------------------------------------------//
        // "Special Stuff" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "Generate Dependency Pack" action.
        let slot_generate_dependency_pack = SlotBool::new(clone!(
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // Ask the background loop to create the Dependency PackFile.
                sender_qt.send("create_dependency_database").unwrap();

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Until we receive a response from the worker thread...
                loop {

                    // When we finally receive a response...
                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                        // Check what the result of the creation process was.
                        match data {

                            // In case of success.....
                            Ok(data) => {

                                // Get the success message and show it.
                                let message: &str = serde_json::from_slice(&data).unwrap();
                                show_dialog(&app_ui, true, message);

                                // Reload the Dependency PackFile for our Game Selected.
                                sender_qt.send("set_dependency_database").unwrap();
                            }

                            // In case of error, report the error.
                            Err(error) => show_dialog(&app_ui, false, error.cause()),
                        }

                        // Stop the loop.
                        break;
                    }

                    // Keep the UI responsive.
                    event_loop.process_events(());

                    // Wait a bit to not saturate a CPU core.
                    thread::sleep(Duration::from_millis(50));
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Patch Siege AI" action.
        let slot_patch_siege_ai = SlotBool::new(clone!(
            is_modified,
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // Ask the background loop to create the Dependency PackFile.
                sender_qt.send("patch_siege_ai").unwrap();

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Until we receive a response from the worker thread...
                loop {

                    // When we finally receive a response...
                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                        // Check what the result of the patching process was.
                        match data {

                            // In case of success.....
                            Ok(data) => {

                                // Get the success message and show it.
                                let message: String = serde_json::from_slice(&data).unwrap();
                                show_dialog(&app_ui, true, &message);

                                // Set the mod as "Not Modified", because this action includes saving the PackFile.
                                *is_modified.borrow_mut() = set_modified(false, &app_ui);

                                // Get the data we need to update the UI...
                                let data = receiver_qt.borrow().recv().unwrap().unwrap();

                                // Deserialize it (name of the packfile, paths of the PackedFiles).
                                let data: (&str, Vec<Vec<String>>, u32) = serde_json::from_slice(&data).unwrap();

                                // Update the TreeView.
                                update_treeview(
                                    &app_ui,
                                    (&data.0, data.1.to_vec()),
                                    TreeViewOperation::Build,
                                );
                            }

                            // In case of error, report the error.
                            Err(error) => show_dialog(&app_ui, false, error.cause()),
                        }

                        // Stop the loop.
                        break;
                    }

                    // Keep the UI responsive.
                    event_loop.process_events(());

                    // Wait a bit to not saturate a CPU core.
                    thread::sleep(Duration::from_millis(50));
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // "Special Stuff" Menu Actions.
        unsafe { app_ui.wh2_generate_dependency_pack.as_ref().unwrap().signals().triggered().connect(&slot_generate_dependency_pack); }
        unsafe { app_ui.wh_generate_dependency_pack.as_ref().unwrap().signals().triggered().connect(&slot_generate_dependency_pack); }
        unsafe { app_ui.att_generate_dependency_pack.as_ref().unwrap().signals().triggered().connect(&slot_generate_dependency_pack); }

        unsafe { app_ui.wh2_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }
        unsafe { app_ui.wh_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }

        //-----------------------------------------------------//
        // "About" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "About Qt" action.
        let slot_about_qt = SlotBool::new(|_| { unsafe { MessageBox::about_qt(app_ui.window as *mut Widget); }});

        // What happens when we trigger the "About RPFM" action.
        let slot_about_rpfm = SlotBool::new(|_| {
            unsafe {
                MessageBox::about(
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("About RPFM"),
                    &QString::from_std_str(format!(
                        "<table>
                            <tr>
                                <td><h2><b>Rusted PackFile Manager</b></h2></td>
                                <td>{}</td>
                            </tr>
                        </table>

                        <p><b>Rusted PackFile Manager</b> (a.k.a. RPFM) is a modding tool for modern Total War Games, made by modders, for modders.</p>
                        <p>This program is <b>open-source</b>, under MIT License. You can always get the last version (or collaborate) here:</p>
                        <a href=\"https://github.com/Frodo45127/rpfm\">https://github.com/Frodo45127/rpfm</a>
                        <p>This program is also <b>free</b> (if you paid for this, sorry, but you got scammed), but if you want to help with money, here is <b>RPFM's Patreon</b>:</p>
                        <a href=\"https://www.patreon.com/RPFM\">https://www.patreon.com/RPFM</a>

                        <h3>Credits</h3>
                        <ul style=\"list-style-type: disc\">
                            <li>Created and Programmed by: <b>Frodo45127</b>.</li>
                            <li>Icon by: <b>Maruka</b>.</li>
                            <li>RigidModel research by: <b>Mr.Jox</b>, <b>Der Spaten</b>, <b>Maruka</b> and <b>Frodo45127</b>.</li>
                            <li>LUA functions by: <b>Aexrael Dex</b>.</li>
                        </ul>

                        <h3>Special thanks</h3>
                        <ul style=\"list-style-type: disc\">
                            <li><b>PFM team</b>, for providing the community with awesome modding tools.</li>
                            <li><b>CA</b>, for being a mod-friendly company.</li>
                        </ul>
                        ", &VERSION
                    ))
                ); }
            }
        );

        // What happens when we trigger the "Support me on Patreon" action.
        let slot_patreon_link = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str("https://www.patreon.com/RPFM"))); });

        // What happens when we trigger the "Check Updates" action.
        let slot_check_updates = SlotBool::new(move |_| { check_updates(&app_ui, true); });

        // What happens when we trigger the "Check Schema Updates" action.
        let slot_check_schema_updates = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            rpfm_path => move |_| { check_schema_updates(&app_ui, true, &rpfm_path, &sender_qt, &sender_qt_data, &receiver_qt) }));

        // "About" Menu Actions.
        unsafe { app_ui.about_qt.as_ref().unwrap().signals().triggered().connect(&slot_about_qt); }
        unsafe { app_ui.about_rpfm.as_ref().unwrap().signals().triggered().connect(&slot_about_rpfm); }
        unsafe { app_ui.patreon_link.as_ref().unwrap().signals().triggered().connect(&slot_patreon_link); }
        unsafe { app_ui.check_updates.as_ref().unwrap().signals().triggered().connect(&slot_check_updates); }
        unsafe { app_ui.check_schema_updates.as_ref().unwrap().signals().triggered().connect(&slot_check_schema_updates); }

        //-----------------------------------------------------//
        // Show the Main Window and start everything...
        //-----------------------------------------------------//

        // We need to rebuild the MyMod menu while opening it if the variable for it is true.
        let slot_rebuild_mymod_menu = SlotNoArgs::new(clone!(
            mymod_stuff,
            mymod_stuff_slots,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            mymod_menu_needs_rebuild => move || {
                if *mymod_menu_needs_rebuild.borrow() {

                    // Then recreate the "MyMod" submenu.
                    let result = build_my_mod_menu(
                        sender_qt.clone(),
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.clone(),
                        &menu_bar_mymod,
                        is_modified.clone(),
                        mode.clone(),
                        supported_games.to_vec(),
                        mymod_menu_needs_rebuild.clone(),
                    );

                    // And store the new values.
                    *mymod_stuff.borrow_mut() = result.0;
                    *mymod_stuff_slots.borrow_mut() = result.1;

                    // Disable the rebuild for the next time.
                    *mymod_menu_needs_rebuild.borrow_mut() = false;
                }
            }
        ));
        unsafe { menu_bar_mymod.as_ref().unwrap().signals().about_to_show().connect(&slot_rebuild_mymod_menu); }

        // Show the Main Window...
        unsafe { app_ui.window.as_mut().unwrap().show(); }

        // If we have it enabled in the prefs, check if there are updates.
        if settings.check_updates_on_start { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if settings.check_schema_updates_on_start { check_schema_updates(&app_ui, false, &rpfm_path, &sender_qt, &sender_qt_data, &receiver_qt) };

        // And launch it.
        Application::exec()
    })
}

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI stuff here.
/// The sender is to send stuff back (Result with something encoded or error) to the UI.
/// The receiver is to receive orders to execute from the loop.
/// The receiver_data is to receive data (whatever data is needed) encoded with serde from the UI Thread.
fn background_loop(
    rpfm_path: &PathBuf,
    sender: Sender<Result<Vec<u8>, Error>>,
    receiver: Receiver<&str>,
    receiver_data: Receiver<Result<Vec<u8>, Error>>
) {

    //---------------------------------------------------------------------------------------//
    // Initializing stuff...
    //---------------------------------------------------------------------------------------//

    // We need two PackFiles:
    // - `pack_file_decoded`: This one will hold our opened PackFile.
    // - `pack_file_decoded_extra`: This one will hold the PackFile opened for the `add_from_packfile` feature.
    let mut pack_file_decoded = PackFile::new();
    let mut pack_file_decoded_extra = PackFile::new();

    // We load the list of Supported Games here.
    // TODO: Move this to a const when const fn reach stable in Rust.
    let supported_games = GameInfo::new();

    // We load the settings here, and in case they doesn't exist, we create them.
    let mut settings = Settings::load(&rpfm_path, &supported_games).unwrap_or_else(|_|Settings::new(&supported_games));

    // We prepare the schema object to hold an Schema, leaving it as `None` by default.
    let mut schema: Option<Schema> = None;

    // And we prepare the stuff for the default game (paths, and those things).
    let mut game_selected = GameSelected::new(&settings, &rpfm_path, &supported_games);

    // Try to open the dependency PackFile of our `game_selected`.
    let mut dependency_database = match packfile::open_packfile(game_selected.game_dependency_packfile_path.to_path_buf()) {
        Ok(pack_file) => Some(pack_file.data.packed_files),
        Err(_) => None,
    };

    //---------------------------------------------------------------------------------------//
    // Looping forever and ever...
    //---------------------------------------------------------------------------------------//

    // Start the main loop.
    loop {

        // Wait until you get something through the channel.
        match receiver.recv() {

            // If you got a message...
            Ok(data) => {

                // Act depending on what that message is.
                match data {

                    // In case we want to reset the PackFile to his original state (dummy)...
                    "reset_packfile" => {

                        // Create the new PackFile.
                        pack_file_decoded = PackFile::new();
                    }

                    // In case we want to create a "New PackFile"...
                    "new_packfile" => {

                        // Get the ID for the new PackFile.
                        let pack_file_id = supported_games.iter().filter(|x| x.folder_name == game_selected.game).map(|x| x.id.to_owned()).collect::<String>();

                        // Create the new PackFile.
                        pack_file_decoded = packfile::new_packfile("unknown.pack".to_string(), &pack_file_id);

                        // Try to load the Schema for this PackFile's game.
                        schema = Schema::load(&rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                        // Get the data we must return to the UI thread and serialize it.
                        let data = serde_json::to_vec(&(&pack_file_decoded.extra_data.file_name, pack_file_decoded.data.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(), pack_file_decoded.header.pack_file_type)).map_err(From::from);

                        // Send a response to the UI thread.
                        sender.send(data).unwrap();
                    }

                    // In case we want to create a "New PackFile"...
                    "open_packfile" => {

                        // Get the path to the PackFile.
                        let path = receiver_data.recv().unwrap().unwrap();

                        // Try to deserialize it as a path.
                        let path = serde_json::from_slice(&path).unwrap();

                        // Open the PackFile (Or die trying it).
                        match packfile::open_packfile(path) {
                            Ok(pack_file) => {

                                // Get the decoded PackFile.
                                pack_file_decoded = pack_file;

                                // Try to load the Schema for this PackFile's game.
                                schema = Schema::load(&rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                                // Get the data we must return to the UI thread and serialize it.
                                let data = serde_json::to_vec(&(
                                    &pack_file_decoded.extra_data.file_name,
                                    pack_file_decoded.data.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                                    pack_file_decoded.header.pack_file_type
                                )).map_err(From::from);

                                // Send a response to the UI thread.
                                sender.send(data).unwrap();

                                //Test to see if every DB Table can be decoded.
                                // let mut counter = 0;
                                // for i in pack_file_decoded.data.packed_files.iter() {
                                //     if i.path.starts_with(&["db".to_owned()]) {
                                //         if let Some(ref schema) = schema {
                                //             if let Err(_) = packedfile::db::DB::read(&i.data, &i.path[1], &schema) {
                                //                 match packedfile::db::DBHeader::read(&i.data, &mut 0) {
                                //                     Ok(db_header) => {
                                //                         if db_header.entry_count > 0 {
                                //                             counter += 1;
                                //                             println!("{}, {:?}", counter, i.path);
                                //                         }
                                //                     }
                                //                     Err(_) => println!("Error in {:?}", i.path),
                                //                 }
                                //             }
                                //         }
                                //     }
                                // }
                            }

                            // If there is an error, send it back to the UI.
                            Err(error) => sender.send(Err(error)).unwrap(),
                        }
                    }

                    // When we want to "Save a PackFile"...
                    "save_packfile" => {

                        // Check if it's editable.
                        if pack_file_decoded.is_editable(&settings) {

                            // Check if it already exist in the disk.
                            if pack_file_decoded.extra_data.file_path.is_file() {

                                // If it passed all the checks, then try to save it and return the result.
                                match packfile::save_packfile(&mut pack_file_decoded, None) {
                                    Ok(_) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                    Err(error) => sender.send(Err(format_err!("Error while trying to save the PackFile:\n\n{}", error.cause()))).unwrap(),
                                }
                            }

                            // Otherwise, we default to the "Save PackFile As" action sending an empty error as response.
                            else { sender.send(Err(format_err!(""))).unwrap(); }
                        }

                        // Otherwise, return an error.
                        else { sender.send(Err(format_err!("This type of PackFile is supported in Read-Only mode.\n\nThis can happen due to:\n - The PackFile's type is 'Boot', 'Release' or 'Patch' and you have 'Allow edition of CA PackFiles' disabled in the settings.\n - The PackFile's type is 'Other'.\n\n If you really want to save it, go to 'PackFile/Change PackFile Type' and change his type to 'Mod' or 'Movie'."))).unwrap(); }
                    }

                    // When we want to "Save a PackFile As"...
                    "save_packfile_as" => {

                        // Check if it's editable.
                        if pack_file_decoded.is_editable(&settings) {

                            // If it's editable, we tell the UI to ask for a Path to save it and pass it the extra data.
                            sender.send(serde_json::to_vec(&pack_file_decoded.extra_data).map_err(From::from)).unwrap();

                            // Wait until you get a path to save it, or an error to cancel the save operation.
                            let path = receiver_data.recv().unwrap();

                            // If it's a path...
                            if let Ok(path) = path {

                                // Deserialize it.
                                let path: PathBuf = serde_json::from_slice(&path).unwrap();

                                // Try to save the PackFile and return the results.
                                match packfile::save_packfile(&mut pack_file_decoded, Some(path.to_path_buf())) {
                                    Ok(_) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                    Err(error) => sender.send(Err(format_err!("Error while trying to save the PackFile:\n\n{}", error.cause()))).unwrap(),
                                }
                            }
                        }

                        // Otherwise, return an error.
                        else { sender.send(Err(format_err!("This type of PackFile is supported in Read-Only mode.\n\nThis can happen due to:\n - The PackFile's type is 'Boot', 'Release' or 'Patch' and you have 'Allow edition of CA PackFiles' disabled in the settings.\n - The PackFile's type is 'Other'.\n\n If you really want to save it, go to 'PackFile/Change PackFile Type' and change his type to 'Mod' or 'Movie'."))).unwrap(); }
                    }

                    // When we change the PackFile's Type...
                    "set_packfile_type" => {

                        // Get the new Type.
                        let new_type = receiver_data.recv().unwrap().unwrap();
                        let new_type = serde_json::from_slice(&new_type).unwrap();

                        // Change it.
                        pack_file_decoded.header.pack_file_type = new_type;
                    }

                    // When we want to know what game is selected...
                    "get_settings" => {

                        // Send the current Game Selected back to the UI thread.
                        sender.send(serde_json::to_vec(&settings).map_err(From::from)).unwrap();
                    }

                    // When we change the Settings...
                    "set_settings" => {

                        // Get the new Settings, and set it.
                        let new_settings = receiver_data.recv().unwrap().unwrap();
                        settings = serde_json::from_slice(&new_settings).unwrap();

                        // Save our new `Settings` to a settings file, and report in case of error.
                        match settings.save(&rpfm_path) {
                            Ok(()) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                            Err(error) => sender.send(Err(error)).unwrap(),
                        }
                    }

                    // When we want to know what game is selected...
                    "get_game_selected" => {

                        // Send the current Game Selected back to the UI thread.
                        sender.send(serde_json::to_vec(&game_selected).map_err(From::from)).unwrap();
                    }

                    // When we change the Game Selected...
                    "set_game_selected" => {

                        // Get the new Game Selected, and set it.
                        let game_name = receiver_data.recv().unwrap().unwrap();
                        let game_name: &str = serde_json::from_slice(&game_name).unwrap();
                        game_selected.change_game_selected(&game_name, &settings.paths.game_paths.iter().filter(|x| x.game == game_name).map(|x| x.path.clone()).collect::<Option<PathBuf>>(), &supported_games);

                        // Try to load the Schema for this game.
                        schema = Schema::load(&rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                        // Change the `dependency_database` for that game.
                        dependency_database = match packfile::open_packfile(game_selected.game_dependency_packfile_path.to_path_buf()) {
                            Ok(data) => Some(data.data.packed_files),
                            Err(_) => None,
                        };

                        // Send back the new Game Selected, and a bool indicating if there is a PackFile open.
                        sender.send(serde_json::to_vec(&(game_selected.clone(), pack_file_decoded.extra_data.file_name.is_empty())).map_err(From::from)).unwrap();
                    }

                    // When we want to know what the PackFile's header is...
                    "get_packfile_id" => {

                        // Send the header of the currently open PackFile.
                        sender.send(serde_json::to_vec(&pack_file_decoded.header.id).map_err(From::from)).unwrap();
                    }

                    // When we want to change the dependency_database for an specific PackFile...
                    "set_dependency_database" => {

                        // Change the `dependency_database` for that game.
                        dependency_database = match packfile::open_packfile(game_selected.game_dependency_packfile_path.to_path_buf()) {
                            Ok(data) => Some(data.data.packed_files),
                            Err(_) => None,
                        };
                    }

                    // When we want to create a new Dependency Database PackFile...
                    "create_dependency_database" => {

                        // Get the data folder of game_selected.
                        match game_selected.game_data_path {

                            // If we got it...
                            Some(ref path) => {

                                // Get the path of the data.pack PackFile.
                                let mut data_pack_path = path.to_path_buf();
                                data_pack_path.push("data.pack");

                                // Try to open it...
                                match packfile::open_packfile(data_pack_path) {

                                    // If we could open it...
                                    Ok(ref mut data_packfile) => {

                                        // Get all the PackedFiles from the db folder (all the tables).
                                        data_packfile.data.packed_files.retain(|packed_file| packed_file.path.starts_with(&["db".to_owned()]));
                                        data_packfile.header.packed_file_count = data_packfile.data.packed_files.len() as u32;

                                        // Get the path of the Dependency PackFiles.
                                        let mut dep_packs_path = game_selected.game_dependency_packfile_path.to_path_buf();
                                        dep_packs_path.pop();

                                        // Create it if it doesn't exist yet (or report error if you can't).
                                        if let Err(error) = DirBuilder::new().recursive(true).create(&dep_packs_path) {
                                            return sender.send(Err(format_err!("Error while trying to create the dependency folder:\n{}", Error::from(error).cause()))).unwrap();
                                        }

                                        // Try to save the new PackFile, and report the result.
                                        match packfile::save_packfile(data_packfile, Some(game_selected.game_dependency_packfile_path.to_path_buf())) {
                                            Ok(_) => sender.send(serde_json::to_vec("Dependency PackFile created. Remember to re-create it if you update the game ;).").map_err(From::from)).unwrap(),
                                            Err(error) => sender.send(Err(format_err!("Generated Dependency PackFile couldn't be saved:\n{}", error.cause()))).unwrap(),
                                        }
                                    }

                                    // If we couldn't open it, report the error.
                                    Err(_) => sender.send(Err(format_err!("Error: data.pack couldn't be open."))).unwrap()
                                }
                            },

                            // If we couldn't found the data folder, report it.
                            None => sender.send(Err(format_err!("Error: data folder of the game not found."))).unwrap()
                        }
                    }

                    // When we want to patch a PackFile...
                    "patch_siege_ai" => {

                        // First, we try to patch the PackFile.
                        match packfile::patch_siege_ai(&mut pack_file_decoded) {

                            // If we succeed....
                            Ok(result) => {

                                // Then we try to save the Patched PackFile.
                                match packfile::save_packfile(&mut pack_file_decoded, None) {

                                    // If we succeed...
                                    Ok(_) => {

                                        // Report it.
                                        sender.send(serde_json::to_vec(&result).map_err(From::from)).unwrap();

                                        // Get the data we must return to the UI thread and serialize it.
                                        let data = serde_json::to_vec(&(
                                            &pack_file_decoded.extra_data.file_name,
                                            pack_file_decoded.data.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                                            pack_file_decoded.header.pack_file_type
                                        )).map_err(From::from);

                                        // Send a response to the UI thread.
                                        sender.send(data).unwrap();
                                    }

                                    // If there is an error, report it.
                                    Err(error) => sender.send(Err(format_err!("Error while trying to save the PackFile:\n{}", error.cause()))).unwrap()
                                }
                            }

                            // Otherwise, return an error.
                            Err(error) => sender.send(Err(format_err!("Error while trying to patch the PackFile:\n{}", error.cause()))).unwrap()
                        }
                    }

                    // When we want to update our Schemas...
                    "update_schemas" => {

                        // Get the extra data needed to update the schemas.
                        let data = receiver_data.recv().unwrap().unwrap();
                        let data: (Versions, Versions) = serde_json::from_slice(&data).unwrap();

                        // Try to update the schemas...
                        match update_schemas(data.0, data.1, rpfm_path) {

                            // If there is success...
                            Ok(_) => {

                                // Reload the currently loaded schema, just in case it was updated.
                                schema = Schema::load(rpfm_path, &supported_games.iter().filter(|x| x.folder_name == game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                                // Return success.
                                sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap();
                            }

                            // If there is an error while updating, report it.
                            Err(error) => sender.send(Err(error)).unwrap(),
                        }
                    }

                    _ => println!("Error while receiving message, \"{}\" is not a valid message.", data),
                }
            }

            // If you got an error, it means the main UI Thread is dead.
            Err(_) => {

                // Print a message in case we got a terminal to show it.
                println!("Main UI Thread dead. Exiting...");

                // Break the loop, effectively terminating the thread.
                break;
            },
        }
    }
}

/// This function enables or disables the actions from the `MenuBar` needed when we open a PackFile.
/// NOTE: To disable the "Special Stuff" actions, we use `enable` => false.
fn enable_packfile_actions(
    app_ui: &AppUI,
    game_selected: &GameSelected,
    enable: bool
) {

    // Enable or disable the actions from "PackFile" Submenu.
    unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(enable); }
    unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(enable); }
    unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().set_enabled(enable); }

    // If we are enabling...
    if enable {

        // Check the Game Selected and enable the actions corresponding to out game.
        match &*game_selected.game {
            "warhammer_2" => {
                unsafe { app_ui.wh2_generate_dependency_pack.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_enabled(true); }
            },
            "warhammer" => {
                unsafe { app_ui.wh_generate_dependency_pack.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_enabled(true); }
            },
            "attila" => {
                unsafe { app_ui.att_generate_dependency_pack.as_mut().unwrap().set_enabled(true); }
            },
            _ => {},
        }
    }

    // If we are disabling...
    else {
        // Disable Warhammer 2 actions...
        unsafe { app_ui.wh2_generate_dependency_pack.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_enabled(false); }

        // Disable Warhammer actions...
        unsafe { app_ui.wh_generate_dependency_pack.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_enabled(false); }

        // Disable Attila actions...
        unsafe { app_ui.att_generate_dependency_pack.as_mut().unwrap().set_enabled(false); }
    }
}

/// This function is used to set the current "Operational Mode". It not only sets the "Operational Mode",
/// but it also takes care of disabling or enabling all the signals related with the "MyMod" Mode.
/// If `my_mod_path` is None, we want to set the `Normal` mode. Otherwise set the `MyMod` mode.
fn set_my_mod_mode(
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    mode: &Rc<RefCell<Mode>>,
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
            unsafe { mymod_stuff.borrow_mut().delete_selected_mymod.as_mut().unwrap().set_enabled(true); }
            unsafe { mymod_stuff.borrow_mut().install_mymod.as_mut().unwrap().set_enabled(true); }
            unsafe { mymod_stuff.borrow_mut().uninstall_mymod.as_mut().unwrap().set_enabled(true); }
        }

        // If `None` has been provided...
        None => {

            // Set the current mode to `Normal`.
            *mode.borrow_mut() = Mode::Normal;

            // Disable all "MyMod" related actions, except "New MyMod".
            unsafe { mymod_stuff.borrow_mut().delete_selected_mymod.as_mut().unwrap().set_enabled(false); }
            unsafe { mymod_stuff.borrow_mut().install_mymod.as_mut().unwrap().set_enabled(false); }
            unsafe { mymod_stuff.borrow_mut().uninstall_mymod.as_mut().unwrap().set_enabled(false); }
        }
    }
}

/// This function opens the PackFile at the provided Path, and sets all the stuff needed, depending
/// on the situation.
/// NOTE: The `game_folder` &str is for when using this function with "MyMods".
fn open_packfile(
    sender_qt: &Sender<&str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
    pack_file_path: PathBuf,
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    is_modified: &Rc<RefCell<bool>>,
    mode: &Rc<RefCell<Mode>>,
    game_folder: &str,
) -> Result<(), Error> {

    // Tell the Background Thread to create a new PackFile.
    sender_qt.send("open_packfile").unwrap();

    // Send the path to the Background Thread.
    sender_qt_data.send(serde_json::to_vec(&pack_file_path).map_err(From::from)).unwrap();

    // Prepare the event loop, so we don't hang the UI while the background thread is working.
    let mut event_loop = EventLoop::new();

    // Disable the Main Window (so we can't do other stuff).
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

    // Until we receive a response from the worker thread...
    loop {

        // When we finally receive the data of the PackFile...
        if let Ok(data) = receiver_qt.borrow().try_recv() {

            // Check if the PackFile was succesfully decoded or not.
            match data {

                // If it was it...
                Ok(data) => {

                    // Deserialize it (name of the packfile, paths of the PackedFiles, type of the PackFile).
                    let data: (&str, Vec<Vec<String>>, u32) = serde_json::from_slice(&data).unwrap();

                    // Update the TreeView.
                    update_treeview(
                        &app_ui,
                        (&data.0, data.1.to_vec()),
                        TreeViewOperation::Build,
                    );

                    // We choose the right option, depending on our PackFile.
                    match data.2 {
                        0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                        1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                        2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                        3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                        4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                        _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                    }

                    // Stop the loop.
                    break;
                }

                // Otherwise, return an error.
                Err(error) => return Err(error)
            }
        }

        // Keep the UI responsive.
        event_loop.process_events(());

        // Wait a bit to not saturate a CPU core.
        thread::sleep(Duration::from_millis(50));
    }

    // Set the new mod as "Not modified".
    *is_modified.borrow_mut() = set_modified(false, &app_ui);

    // Get the Game Selected.
    sender_qt.send("get_game_selected").unwrap();
    let response = receiver_qt.borrow().recv().unwrap().unwrap();
    let game_selected = serde_json::from_slice(&response).unwrap();

    // Enable the actions available for the PackFile from the `MenuBar`.
    enable_packfile_actions(&app_ui, &game_selected, false);

    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
    set_my_mod_mode(&mymod_stuff, &mode, None);

    // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
    if !game_folder.is_empty() {

        // Change the Game Selected in the UI.
        match game_folder {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checked(true); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().set_checked(true); }
            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().set_checked(true); }
        }

        // Change the Game Selected in the other Thread.
        sender_qt.send("set_game_selected").unwrap();
        sender_qt_data.send(serde_json::to_vec(game_folder).map_err(From::from)).unwrap();

        // Set the current "Operational Mode" to `MyMod`.
        set_my_mod_mode(&mymod_stuff, mode, Some(pack_file_path));

        // Receive the return from `set_game_selected`, so it doesn't mess up the channels.
        receiver_qt.borrow().recv().unwrap();
    }

    // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
    else {

        // Get the PackFile Header.
        sender_qt.send("get_packfile_id").unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        let id: &str = serde_json::from_slice(&response).unwrap();

        // Depending on the Id, choose one game or another.
        match &*id {

            // PFH5 is for Warhammer 2/Arena, but Arena is not yet supported.
            "PFH5" => {

                // Change the Game Selected in the UI.
                unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checked(true); }

                // Change the Game Selected in the other Thread.
                sender_qt.send("set_game_selected").unwrap();
                sender_qt_data.send(serde_json::to_vec("warhammer_2").map_err(From::from)).unwrap();

                // Receive the return from `set_game_selected`, so it doesn't mess up the channels.
                receiver_qt.borrow().recv().unwrap();
            },

            // PFH4 is for Warhammer 1/Attila.
            "PFH4" | _ => {

                // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila.
                // In any other case, we select Attila by default.
                match &*game_selected.game {
                    "warhammer" => {

                        // Change the Game Selected in the UI.
                        unsafe { app_ui.warhammer.as_mut().unwrap().set_checked(true); }

                        // Change the Game Selected in the other Thread.
                        sender_qt.send("set_game_selected").unwrap();
                        sender_qt_data.send(serde_json::to_vec("warhammer").map_err(From::from)).unwrap();

                        // Receive the return from `set_game_selected`, so it doesn't mess up the channels.
                        receiver_qt.borrow().recv().unwrap();
                    }
                    "attila" | _ => {

                        // Change the Game Selected in the UI.
                        unsafe { app_ui.attila.as_mut().unwrap().set_checked(true); }

                        // Change the Game Selected in the other Thread.
                        sender_qt.send("set_game_selected").unwrap();
                        sender_qt_data.send(serde_json::to_vec("attila").map_err(From::from)).unwrap();

                        // Receive the return from `set_game_selected`, so it doesn't mess up the channels.
                        receiver_qt.borrow().recv().unwrap();
                    }
                }
            },
        }

        // Set the current "Operational Mode" to `Normal`.
        set_my_mod_mode(&mymod_stuff, mode, None);
    }

    // Re-enable the Main Window.
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

    // Change the Dependency Database used for our PackFile in the other Thread.
    sender_qt.send("set_dependency_database").unwrap();

    // Enable the actions available for the PackFile from the `MenuBar`.
    enable_packfile_actions(&app_ui, &game_selected, true);

    // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
    purge_them_all(&app_ui);

    // Show the "Tips".
    display_help_tips(&app_ui);

    // Return success.
    Ok(())
}

/// This function takes care of the re-creation of the "MyMod" list in the following moments:
/// - At the start of the program (here).
/// - At the end of MyMod deletion.
/// - At the end of MyMod creation.
/// - At the end of settings update.
/// We need to return the struct for further manipulation of his actions.
fn build_my_mod_menu(
    sender_qt: Sender<&'static str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
    app_ui: AppUI,
    menu_bar_mymod: &*mut Menu,
    is_modified: Rc<RefCell<bool>>,
    mode: Rc<RefCell<Mode>>,
    supported_games: Vec<GameInfo>,
    needs_rebuild: Rc<RefCell<bool>>,
) -> (MyModStuff, MyModSlots) {

    // Get the current Settings.
    sender_qt.send("get_settings").unwrap();
    let settings_encoded = receiver_qt.borrow().recv().unwrap().unwrap();
    let settings: Settings = serde_json::from_slice(&settings_encoded).unwrap();

    //---------------------------------------------------------------------------------------//
    // Build the "Static" part of the menu...
    //---------------------------------------------------------------------------------------//

    // First, we clear the list, just in case this is a "Rebuild" of the menu.
    unsafe { menu_bar_mymod.as_mut().unwrap().clear(); }

    // Then, we create the actions again.
    let mymod_stuff;
    unsafe {
        mymod_stuff = MyModStuff {
            new_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&New MyMod")),
            delete_selected_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Delete Selected MyMod")),
            install_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Install")),
            uninstall_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Uninstall")),
        }
    }

    // Add a separator in the middle of the menu.
    unsafe { menu_bar_mymod.as_mut().unwrap().insert_separator(mymod_stuff.install_mymod); }

    // And we create the empty slots, to be replaced later with the real ones from the Main function.
    let mut mymod_slots = MyModSlots {
        new_mymod: SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            app_ui,
            mode,
            settings,
            is_modified,
            needs_rebuild,
            supported_games => move |_| {

                // Create the "New MyMod" Dialog, and get the result.
                match NewMyModDialog::create_new_mymod_dialog(&app_ui, &supported_games, &settings) {

                    // If we accepted...
                    Some(data) => {

                        // Get the info about the new MyMod.
                        let mod_name = data.0;
                        let mod_game = data.1;

                        // Get the PackFile name.
                        let full_mod_name = format!("{}.pack", mod_name);

                        // Change the Game Selected.
                        match &*mod_game {
                            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
                            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                        }

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send("new_packfile").unwrap();

                        // Prepare the event loop, so we don't hang the UI while the background thread is working.
                        let mut event_loop = EventLoop::new();

                        // Disable the Main Window (so we can't do other stuff).
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                        // Until we receive a response from the worker thread...
                        loop {

                            // When we finally receive the data of the PackFile...
                            if let Ok(data) = receiver_qt.borrow().try_recv() {

                                // Unwrap the data.
                                let data = data.unwrap();

                                // Deserialize it (name of the packfile, paths of the PackedFiles).
                                let data: (&str, Vec<Vec<String>>, u32) = serde_json::from_slice(&data).unwrap();

                                // Update the TreeView.
                                update_treeview(
                                    &app_ui,
                                    (&data.0, data.1.to_vec()),
                                    TreeViewOperation::Build,
                                );

                                // Mark it as "Mod" in the UI.
                                unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }

                                // Stop the loop.
                                break;
                            }

                            // Keep the UI responsive.
                            event_loop.process_events(());

                            // Wait a bit to not saturate a CPU core.
                            thread::sleep(Duration::from_millis(50));
                        }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                        // Set the new mod as "Not modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui);

                        // Get the Game Selected.
                        sender_qt.send("get_game_selected").unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let game_selected = serde_json::from_slice(&response).unwrap();

                        // Enable the actions available for the PackFile from the `MenuBar`.
                        enable_packfile_actions(&app_ui, &game_selected, true);

                        // Get his new path from the base "MyMod" path + `mod_game`.
                        let mut mymod_path = settings.paths.my_mods_base_path.clone().unwrap();
                        mymod_path.push(&mod_game);

                        // Just in case the folder doesn't exist, we try to create it.
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path) {
                            return show_dialog(&app_ui, false, format!("Error while creating the folder {} to store the MyMods.", mod_game));
                        }

                        // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                        let mut mymod_path_private = mymod_path.to_path_buf();
                        mymod_path_private.push(&mod_name);
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path_private) {
                            return show_dialog(&app_ui, false, format!("Error while creating the folder {} to store the MyMod's files.", mod_name));
                        };

                        // Add the PackFile name to the full path.
                        mymod_path.push(&full_mod_name);

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send("save_packfile_as").unwrap();

                        // We ignore the returning confirmation.
                        let _confirmation = receiver_qt.borrow().recv().unwrap();

                        // Pass it to the worker thread.
                        sender_qt_data.send(serde_json::to_vec(&mymod_path).map_err(From::from)).unwrap();

                        // Disable the Main Window (so we can't do other stuff).
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                        // Until we receive a response from the worker thread...
                        loop {

                            // When we finally receive the data...
                            if let Ok(data) = receiver_qt.borrow().try_recv() {

                                // Check what the result of the saving process was.
                                match data {

                                    // In case of success...
                                    Ok(_) => {

                                        // Set the current "Operational Mode" to `MyMod`.
                                        set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, Some(mymod_path));

                                        // Set it to rebuild next time we try to open the MyMod Menu.
                                        *needs_rebuild.borrow_mut() = true;

                                        // Get the Selection Model and the Model Index of the PackFile's Cell.
                                        let selection_model;
                                        let model_index;
                                        unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                                        unsafe { model_index = app_ui.folder_tree_model.as_ref().unwrap().index((0, 0)); }

                                        // Select the PackFile's Cell with a "Clear & Select".
                                        unsafe { selection_model.as_mut().unwrap().select((&model_index, Flags::from_int(3))); }

                                        // Rename the Unknown PackFile to his final name.
                                        update_treeview(
                                            &app_ui,
                                            ("", vec![]),
                                            TreeViewOperation::Rename(full_mod_name),
                                        );
                                    }

                                    // In case of error, we can have two results.
                                    Err(error) => show_dialog(&app_ui, false, format!("Error while saving the PackFile:\n\n{}", error.cause())),
                                }

                                // Stop the loop.
                                break;
                            }

                            // Keep the UI responsive.
                            event_loop.process_events(());

                            // Wait a bit to not saturate a CPU core.
                            thread::sleep(Duration::from_millis(50));
                        }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }

                    // If we canceled the creation of a "MyMod", just return.
                    None => return,
                }
            }
        )),
        delete_selected_mymod: SlotBool::new(clone!(
            sender_qt,
            receiver_qt,
            settings,
            mode,
            is_modified,
            app_ui => move |_| {

                // Ask before doing it, as this will permanently delete the mod from the Disk.
                if are_you_sure(&is_modified, true) {

                    // We want to keep our "MyMod" name for the success message, so we store it here.
                    let old_mod_name: String;

                    // If we have a "MyMod" selected...
                    let mod_deleted = match *mode.borrow() {
                        Mode::MyMod {ref game_folder_name, ref mod_name} => {

                            // We save the name of the PackFile for later use.
                            old_mod_name = mod_name.to_owned();

                            // And the "MyMod" path is configured...
                            if let Some(ref mymods_base_path) = settings.paths.my_mods_base_path {

                                // We get his path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // If the mod doesn't exist, return error.
                                if !mymod_path.is_file() { return show_dialog(&app_ui, false, "Error: PackFile doesn't exist, so it can't be deleted."); }

                                // And we delete that PackFile.
                                if let Err(error) = remove_file(&mymod_path).map_err(Error::from) {
                                    return show_dialog(&app_ui, false, format!("Error while deleting the PackFile from disk:\n{}", error.cause()));
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.to_path_buf();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(&mymod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(&app_ui, false, "Mod deleted, but his assets folder haven't been found.");
                                }

                                // If the assets folder exists, we try to delete it.
                                else if let Err(error) = remove_dir_all(&mymod_assets_path).map_err(Error::from) {
                                    return show_dialog(&app_ui, false, format!("Error while deleting the Assets Folder of the MyMod from disk:\n{}", error.cause()));
                                }

                                // We return true, as we have delete the files of the "MyMod".
                                true
                            }

                            // If the "MyMod" path is not configured, return an error.
                            else { return show_dialog(&app_ui, false, "MyMod base path not configured, so the MyMod couldn't be deleted."); }
                        }

                        // If we don't have a "MyMod" selected, return an error.
                        Mode::Normal => return show_dialog(&app_ui, false, "You can't delete the selected MyMod if there is no MyMod selected."),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {


                        // Set the current "Operational Mode" to `Normal`.
                        set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, None);

                        // Create a "dummy" PackFile, effectively closing the currently open PackFile.
                        sender_qt.send("reset_packfile").unwrap();

                        // Set the dummy mod as "Not modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui);

                        // Get the Game Selected.
                        sender_qt.send("get_game_selected").unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let game_selected = serde_json::from_slice(&response).unwrap();

                        // Enable the actions available for the PackFile from the `MenuBar`.
                        enable_packfile_actions(&app_ui, &game_selected, false);

                        // Clear the TreeView.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().clear(); }

                        // Set it to rebuild next time we try to open the MyMod Menu.
                        *needs_rebuild.borrow_mut() = true;

                        // Show the "MyMod" deleted Dialog.
                        show_dialog(&app_ui, true, format!("MyMod successfully deleted: \"{}\".", old_mod_name));
                    }
                }
            }
        )),
        install_mymod: SlotBool::new(clone!(
            sender_qt,
            receiver_qt,
            settings,
            mode,
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // And the "MyMod" path is configured...
                        if let Some(ref mymods_base_path) = settings.paths.my_mods_base_path {

                            // Get the Game Selected.
                            sender_qt.send("get_game_selected").unwrap();
                            let response = receiver_qt.borrow().recv().unwrap().unwrap();
                            let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                            // Get the `game_data_path` of the game.
                            let game_data_path = game_selected.game_data_path.clone();

                            // If we have a `game_data_path` for the current `GameSelected`...
                            if let Some(mut game_data_path) = game_data_path {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // We check that the "MyMod"s PackFile exists.
                                if !mymod_path.is_file() { return show_dialog(&app_ui, false, "Error: PackFile doesn't exist, so it can't be deleted."); }

                                // We check that the destination path exists.
                                if !game_data_path.is_dir() {
                                    return show_dialog(&app_ui, false, "Destination folder (..xxx/data) doesn't exist. You sure you configured the right folder for the game?");
                                }

                                // Get the destination path for the PackFile with the PackFile included.
                                game_data_path.push(&mod_name);

                                // And copy the PackFile to his destination. If the copy fails, return an error.
                                if let Err(error) = copy(mymod_path, game_data_path).map_err(Error::from) {
                                    return show_dialog(&app_ui, false, format!("Error while copying the PackFile to the Data folder:\n{}", error.cause()));
                                }
                            }

                            // If we don't have a `game_data_path` configured for the current `GameSelected`...
                            else { return show_dialog(&app_ui, false, "Game Path not configured. Go to 'PackFile/Preferences' and configure it."); }
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else { show_dialog(&app_ui, false, "MyMod base path not configured, so the MyMod couldn't be installed."); }
                    }

                    // If we have no MyMod selected, return an error.
                    Mode::Normal => show_dialog(&app_ui, false, "You can't install the selected MyMod if there is no MyMod selected."),
                }

            }
        )),
        uninstall_mymod: SlotBool::new(clone!(
            sender_qt,
            receiver_qt,
            mode,
            app_ui => move |_| {

                // Depending on our current "Mode", we choose what to do.
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref mod_name,..} => {

                        // Get the Game Selected.
                        sender_qt.send("get_game_selected").unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                        // Get the `game_data_path` of the game.
                        let game_data_path = game_selected.game_data_path.clone();

                        // If we have a `game_data_path` for the current `GameSelected`...
                        if let Some(mut game_data_path) = game_data_path {

                            // Get the destination path for the PackFile with the PackFile included.
                            game_data_path.push(&mod_name);

                            // We check that the "MyMod" is actually installed in the provided path.
                            if !game_data_path.is_file() {
                                return show_dialog(&app_ui, false, "The currently selected MyMod is not installed.");
                            }

                            // If the "MyMod" is installed, we remove it. If there is a problem deleting it, return an error dialog.
                            else if let Err(error) = remove_file(game_data_path).map_err(Error::from) {
                                return show_dialog(&app_ui, false, format!("Error uninstalling the MyMod:\n{}", error.cause()));
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else {
                            show_dialog(&app_ui, false, "Game Path not configured. Go to 'PackFile/Preferences' and configure it.");
                        }
                    }

                    // If we have no MyMod selected, return an error.
                    Mode::Normal => show_dialog(&app_ui, false, "You can't uninstall the selected MyMod if there is no MyMod selected."),
                }
            }
        )),
        open_mymod: vec![],
    };

    // "About" Menu Actions.
    unsafe { mymod_stuff.new_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.new_mymod); }
    unsafe { mymod_stuff.delete_selected_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.delete_selected_mymod); }
    unsafe { mymod_stuff.install_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.install_mymod); }
    unsafe { mymod_stuff.uninstall_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.uninstall_mymod); }

    //---------------------------------------------------------------------------------------//
    // Build the "Dynamic" part of the menu...
    //---------------------------------------------------------------------------------------//

    // Add a separator for this section.
    unsafe { menu_bar_mymod.as_mut().unwrap().add_separator(); }

    // Get the current settings.
    sender_qt.send("get_settings").unwrap();
    let response = receiver_qt.borrow().recv().unwrap().unwrap();
    let settings: Settings = serde_json::from_slice(&response).unwrap();

    // If we have the "MyMod" path configured...
    if let Some(ref my_mod_base_path) = settings.paths.my_mods_base_path {

        // And can get without errors the folders in that path...
        if let Ok(game_folder_list) = my_mod_base_path.read_dir() {

            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
            for game_folder in game_folder_list {

                // If the file/folder is valid...
                if let Ok(game_folder) = game_folder {

                    // If it's a valid folder, and it's in our supported games list...
                    let supported_folders = supported_games.iter().map(|x| x.folder_name.to_owned()).collect::<Vec<String>>();
                    if game_folder.path().is_dir() && supported_folders.contains(&game_folder.file_name().to_string_lossy().as_ref().to_owned()) {

                        // We create that game's menu here.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        let game_display_name = supported_games.iter().filter(|x| x.folder_name == game_folder_name).map(|x| x.display_name.to_owned()).collect::<String>();
                        let mut game_submenu = Menu::new(&QString::from_std_str(&game_display_name));

                        // If there were no errors while reading the path...
                        if let Ok(game_folder_files) = game_folder.path().read_dir() {

                            // We need to sort these files, so they appear sorted in the menu.
                            let mut game_folder_files_sorted: Vec<_> = game_folder_files.map(|x| x.unwrap().path()).collect();
                            game_folder_files_sorted.sort();

                            // We get all the stuff in that game's folder...
                            for pack_file in &game_folder_files_sorted {

                                // And it's a file that ends in .pack...
                                if pack_file.is_file() && pack_file.extension().unwrap_or_else(||OsStr::new("invalid")).to_string_lossy() == "pack" {

                                    // That means our file is a valid PackFile and it needs to be added to the menu.
                                    let mod_name = pack_file.file_name().unwrap().to_string_lossy().as_ref().to_owned();

                                    // Create the action for it.
                                    let open_mod_action = game_submenu.add_action(&QString::from_std_str(mod_name));

                                    // Get this into an Rc so we can pass it to the Open closure.
                                    let mymod_stuff = Rc::new(RefCell::new(mymod_stuff.clone()));

                                    // Create the slot for that action.
                                    let slot_open_mod = SlotBool::new(clone!(
                                        game_folder_name,
                                        is_modified,
                                        mode,
                                        mymod_stuff,
                                        pack_file,
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt => move |_| {

                                            // Check first if there has been changes in the PackFile.
                                            if are_you_sure(&is_modified, false) {

                                                // Open the PackFile (or die trying it!).
                                                if let Err(error) = open_packfile(
                                                    &sender_qt,
                                                    &sender_qt_data,
                                                    &receiver_qt,
                                                    pack_file.to_path_buf(),
                                                    &app_ui,
                                                    &mymod_stuff,
                                                    &is_modified,
                                                    &mode,
                                                    &game_folder_name,
                                                ) { show_dialog(&app_ui, false, format!("Error while opening the PackFile:\n\n{}", error.cause())) }
                                            }
                                        }
                                    ));

                                    // Add the slot to the list.
                                    mymod_slots.open_mymod.push(slot_open_mod);

                                    // Connect the action to the slot.
                                    unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(mymod_slots.open_mymod.last().unwrap()); }
                                }
                            }
                        }

                        // Only if the submenu has items, we add it to the big menu.
                        if game_submenu.actions().count() > 0 {
                            unsafe { menu_bar_mymod.as_mut().unwrap().add_menu_unsafe(game_submenu.into_raw()); }
                        }
                    }
                }
            }
        }
    }

    // Return the MyModStuff with all the new actions.
    (mymod_stuff, mymod_slots)
}
