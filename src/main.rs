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
extern crate chrono;

#[macro_use]
extern crate sentry;
extern crate open;
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

#[macro_use]
extern crate lazy_static;
extern crate indexmap;

use qt_widgets::action::Action;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::application::Application;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::file_dialog::{AcceptMode, FileDialog, FileMode};
use qt_widgets::main_window::MainWindow;
use qt_widgets::menu::Menu;
use qt_widgets::message_box::MessageBox;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::splitter::Splitter;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;
use qt_gui::desktop_services::DesktopServices;
use qt_gui::font::Font;
use qt_gui::icon::Icon;
use qt_gui::key_sequence::KeySequence;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::qt::{ContextMenuPolicy, ShortcutContext};
use qt_core::slots::{SlotBool, SlotNoArgs, SlotItemSelectionRefItemSelectionRef};
use cpp_utils::StaticCast;

use std::env::args;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fs::{DirBuilder, copy, remove_file, remove_dir_all};

use indexmap::map::IndexMap;
use chrono::NaiveDateTime;
use sentry::integrations::panic::register_panic_handler;

use common::*;
use common::communications::*;
use error::{ErrorKind, Result};
use packedfile::*;
use packedfile::db::schemas_importer::*;
use settings::*;
use ui::*;
use ui::packedfile_db::*;
use ui::packedfile_loc::*;
use ui::packedfile_text::*;
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

mod background_thread;
mod common;
mod error;
mod packfile;
mod packedfile;
mod settings;
mod updater;
mod ui;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
    /// for "MyMod" folders.
    #[derive(Debug)]
    static ref SUPPORTED_GAMES: IndexMap<&'static str, GameInfo> = {
        let mut map = IndexMap::new();

        // Warhammer 2
        map.insert("warhammer_2", GameInfo {
            display_name: "Warhammer 2".to_owned(),
            id: "PFH5".to_owned(),
            schema: "schema_wh.json".to_owned(),
            db_pack: "data.pack".to_owned(),
            loc_pack: "local_en.pack".to_owned(),
            supports_editing: true,
        });

        // Warhammer
        map.insert("warhammer", GameInfo {
            display_name: "Warhammer".to_owned(),
            id: "PFH4".to_owned(),
            schema: "schema_wh.json".to_owned(),
            db_pack: "data.pack".to_owned(),
            loc_pack: "local_en.pack".to_owned(),
            supports_editing: true,
        });

        // Attila
        map.insert("attila", GameInfo {
            display_name: "Attila".to_owned(),
            id: "PFH4".to_owned(),
            schema: "schema_att.json".to_owned(),
            db_pack: "data.pack".to_owned(),
            loc_pack: "local_en.pack".to_owned(),
            supports_editing: true,
        });

        // Rome 2
        map.insert("rome_2", GameInfo {
            display_name: "Rome 2".to_owned(),
            id: "PFH4".to_owned(),
            schema: "schema_rom2.json".to_owned(),
            db_pack: "data_rome2.pack".to_owned(),
            loc_pack: "local_en.pack".to_owned(),
            supports_editing: true,
        });

        // NOTE: There are things that depend on the order of this list, and this game must ALWAYS be the last one.
        // Otherwise, stuff that uses this list will probably break.
        // Arena
        map.insert("arena", GameInfo {
            display_name: "Arena".to_owned(),
            id: "PFH5".to_owned(),
            schema: "schema_are.json".to_owned(),
            db_pack: "wad.pack".to_owned(),
            loc_pack: "local_ex.pack".to_owned(),
            supports_editing: false,
        });

        map
    };
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the DSN needed for Sentry reports to work. Don't change it.
const SENTRY_DSN: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8@sentry.io/1205298";

/// This constant is used to enable or disable the generation of a new Schema file in compile time.
/// If you don't want to explicity create a new Schema for a game, leave this disabled.
const GENERATE_NEW_SCHEMA: bool = false;

/// Custom type to deal with QStrings more easely.
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

    pub change_packfile_type_data_is_encrypted: *mut Action,
    pub change_packfile_type_index_includes_timestamp: *mut Action,
    pub change_packfile_type_index_is_encrypted: *mut Action,
    pub change_packfile_type_header_is_extended: *mut Action,

    // Action Group for the submenu.
    pub change_packfile_type_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // "Game Selected" menu.
    //-------------------------------------------------------------------------------//

    pub warhammer_2: *mut Action,
    pub warhammer: *mut Action,
    pub attila: *mut Action,
    pub rome_2: *mut Action,
    pub arena: *mut Action,

    pub game_selected_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // "Special Stuff" menu.
    //-------------------------------------------------------------------------------//

    // Warhammer 2's actions.
    pub wh2_patch_siege_ai: *mut Action,
    pub wh2_create_prefab: *mut Action,
    pub wh2_optimize_packfile: *mut Action,

    // Warhammer's actions.
    pub wh_patch_siege_ai: *mut Action,
    pub wh_create_prefab: *mut Action,
    pub wh_optimize_packfile: *mut Action,
    
    // Attila's actions.
    pub att_optimize_packfile: *mut Action,
    
    // Rome 2's actions.
    pub rom2_optimize_packfile: *mut Action,

    //-------------------------------------------------------------------------------//
    // "About" menu.
    //-------------------------------------------------------------------------------//
    pub about_qt: *mut Action,
    pub about_rpfm: *mut Action,
    pub open_manual: *mut Action,
    pub patreon_link: *mut Action,
    pub check_updates: *mut Action,
    pub check_schema_updates: *mut Action,

    //-------------------------------------------------------------------------------//
    // "Contextual" menu for the TreeView.
    //-------------------------------------------------------------------------------//
    pub context_menu_add_file: *mut Action,
    pub context_menu_add_folder: *mut Action,
    pub context_menu_add_from_packfile: *mut Action,
    pub context_menu_create_folder: *mut Action,
    pub context_menu_create_db: *mut Action,
    pub context_menu_create_loc: *mut Action,
    pub context_menu_create_text: *mut Action,
    pub context_menu_mass_import_tsv: *mut Action,
    pub context_menu_mass_export_tsv: *mut Action,
    pub context_menu_delete: *mut Action,
    pub context_menu_extract: *mut Action,
    pub context_menu_rename: *mut Action,
    pub context_menu_open_decoder: *mut Action,
}

/// Main function.
fn main() {

    // Initialize sentry, so we can get CTD and thread errors reports.
    let _guard = sentry::init((SENTRY_DSN, sentry::ClientOptions {
        release: sentry_crate_release!(),
        ..Default::default()
    }));

    // If this is a release, register Sentry's Panic Handler, so we get reports on CTD.
    if !cfg!(debug_assertions) { register_panic_handler(); }

    // Create the application...
    Application::create_and_exit(|_app| {

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

        // Create the channels to communicate the threads. The channels are:
        // - `sender_rust, receiver_qt`: used for returning info from the background thread, serialized in Vec<u8>.
        // - `sender_qt, receiver_rust`: used for sending the current action to the background thread.
        // - `sender_qt_data, receiver_rust_data`: used for sending the data to the background thread.
        //   The data sended and received in the last one should be always be serialized into Vec<u8>.
        let (sender_rust, receiver_qt) = channel();
        let (sender_qt, receiver_rust) = channel();
        let (sender_qt_data, receiver_rust_data) = channel();

        // Create the background thread.
        thread::spawn(clone!(rpfm_path => move || { background_thread::background_loop(&rpfm_path, sender_rust, receiver_rust, receiver_rust_data); }));

        //---------------------------------------------------------------------------------------//
        // Creating the UI...
        //---------------------------------------------------------------------------------------//

        // Set the RPFM Icon.
        let icon = Icon::new(&QString::from_std_str(format!("{}/img/rpfm.png", rpfm_path.to_string_lossy())));
        Application::set_window_icon(&icon);

        // Create the main window of the program.
        let mut window = MainWindow::new();
        window.resize((1100, 400));

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
        folder_tree_view.set_animated(true);

        // Create the right-side Grid.
        let mut packed_file_view = Widget::new();
        let mut packed_file_layout = GridLayout::new();
        unsafe { packed_file_view.set_layout(packed_file_layout.static_cast_mut()); }

        // Add the corresponding widgets to the layout.
        unsafe { central_splitter.add_widget(folder_tree_view.static_cast_mut()); }
        unsafe { central_splitter.add_widget(packed_file_view.as_mut_ptr()); }

        // Set the correct proportions for the Splitter.
        let mut clist = qt_core::list::ListCInt::new(());
        clist.append(&300);
        clist.append(&1100);
        central_splitter.set_sizes(&clist);
        central_splitter.set_stretch_factor(0, 0);
        central_splitter.set_stretch_factor(1, 10);

        // MenuBar at the top of the Window.
        let menu_bar = &window.menu_bar();

        // StatusBar at the bottom of the Window.
        let _status_bar = window.status_bar();

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
        let menu_rome_2;
        unsafe { menu_warhammer_2 = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer 2")); }
        unsafe { menu_warhammer = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("War&hammer")); }
        unsafe { menu_attila = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Attila")); }
        unsafe { menu_rome_2 = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Rome 2")); }

        // Contextual Menu for the TreeView.
        let mut folder_tree_view_context_menu = Menu::new(());
        let menu_add = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Add..."));
        let menu_create = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Create..."));
        let menu_open = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Open..."));

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

                change_packfile_type_data_is_encrypted: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Data Is Encrypted")),
                change_packfile_type_index_includes_timestamp: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Index Includes Timestamp")),
                change_packfile_type_index_is_encrypted: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("Index Is &Encrypted")),
                change_packfile_type_header_is_extended: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Header Is Extended")),

                // Action Group for the submenu.
                change_packfile_type_group: ActionGroup::new(menu_change_packfile_type.as_mut().unwrap().static_cast_mut()).into_raw(),

                //-------------------------------------------------------------------------------//
                // "Game Selected" menu.
                //-------------------------------------------------------------------------------//

                warhammer_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Warhammer 2")),
                warhammer: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("War&hammer")),
                attila: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Attila")),
                rome_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("R&ome 2")),
                arena: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("A&rena")),

                game_selected_group: ActionGroup::new(menu_bar_game_seleted.as_mut().unwrap().static_cast_mut()).into_raw(),

                //-------------------------------------------------------------------------------//
                // "Special Stuff" menu.
                //-------------------------------------------------------------------------------//

                // Warhammer 2's actions.
                wh2_patch_siege_ai: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
                wh2_create_prefab: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),
                wh2_optimize_packfile: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

                // Warhammer's actions.
                wh_patch_siege_ai: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
                wh_create_prefab: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),
                wh_optimize_packfile: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),
                
                // Attila's actions.
                att_optimize_packfile: menu_attila.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),
                
                // Rome 2'a actions.
                rom2_optimize_packfile: menu_rome_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

                //-------------------------------------------------------------------------------//
                // "About" menu.
                //-------------------------------------------------------------------------------//
                about_qt: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("About &Qt")),
                about_rpfm: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&About RPFM")),
                open_manual: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&Open Manual")),
                patreon_link: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&Support me on Patreon")),
                check_updates: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("&Check Updates")),
                check_schema_updates: menu_bar_about.as_mut().unwrap().add_action(&QString::from_std_str("Check Schema &Updates")),

                //-------------------------------------------------------------------------------//
                // "Contextual" Menu for the TreeView.
                //-------------------------------------------------------------------------------//

                context_menu_add_file: menu_add.as_mut().unwrap().add_action(&QString::from_std_str("&Add File")),
                context_menu_add_folder: menu_add.as_mut().unwrap().add_action(&QString::from_std_str("Add &Folder")),
                context_menu_add_from_packfile: menu_add.as_mut().unwrap().add_action(&QString::from_std_str("Add from &PackFile")),

                context_menu_create_folder: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("&Create Folder")),
                context_menu_create_loc: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("&Create Loc")),
                context_menu_create_db: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("Create &DB")),
                context_menu_create_text: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("Create &Text")),

                context_menu_mass_import_tsv: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("Mass-Import TSV")),
                context_menu_mass_export_tsv: menu_create.as_mut().unwrap().add_action(&QString::from_std_str("Mass-Export TSV")),

                context_menu_delete: folder_tree_view_context_menu.add_action(&QString::from_std_str("&Delete")),
                context_menu_extract: folder_tree_view_context_menu.add_action(&QString::from_std_str("&Extract")),
                context_menu_rename: folder_tree_view_context_menu.add_action(&QString::from_std_str("&Rename")),

                context_menu_open_decoder: menu_open.as_mut().unwrap().add_action(&QString::from_std_str("&Open with Decoder")),
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

        // These ones are individual, but they need to be checkable and not editable.
        unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checkable(true); }

        unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_enabled(false); }

        // Put separators in the SubMenu.
        unsafe { menu_change_packfile_type.as_mut().unwrap().insert_separator(app_ui.change_packfile_type_other); }
        unsafe { menu_change_packfile_type.as_mut().unwrap().insert_separator(app_ui.change_packfile_type_data_is_encrypted); }

        // The "Game Selected" Menu should be an ActionGroup.
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.attila); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.rome_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.arena); }
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.attila.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.rome_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.arena.as_mut().unwrap().set_checkable(true); }

        // Arena is special, so separate it from the rest.
        unsafe { menu_bar_game_seleted.as_mut().unwrap().insert_separator(app_ui.arena); }

        // Put the Submenus and separators in place.
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_menu(app_ui.preferences, menu_change_packfile_type); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }

        // Put a separator in the "Create" contextual menu.
        unsafe { menu_create.as_mut().unwrap().insert_separator(app_ui.context_menu_mass_import_tsv); }

        // Prepare the TreeView to have a Contextual Menu.
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // This is to get the new schemas. It's controlled by a global const. Don't enable this unless you know what you're doing.
        if GENERATE_NEW_SCHEMA {

            // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
            // The second one should contain all the tables of the game, extracted directly from `data.pack`.
            let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_schemas/");
            let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/schema_stuff/db_tables/");
            match import_schema(&assembly_kit_schemas_path, &testing_tables_path, &rpfm_path) {
                Ok(_) => show_dialog(app_ui.window, true, "Schema successfully created."),
                Err(error) => show_dialog(app_ui.window, false, error),
            }

            // Close the program with code 69
            return 69
        }

        //---------------------------------------------------------------------------------------//
        // Shortcuts for the Menu Bar...
        //---------------------------------------------------------------------------------------//

        // Get the current shortcuts.
        sender_qt.send(Commands::GetShortcuts).unwrap();
        let shortcuts = if let Data::Shortcuts(data) = check_message_validity_recv(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Set the shortcuts for these actions.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("new_packfile").unwrap()))); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("open_packfile").unwrap()))); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("save_packfile").unwrap()))); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("save_packfile_as").unwrap()))); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("preferences").unwrap()))); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("quit").unwrap()))); }

        unsafe { app_ui.about_qt.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_about.get("about_qt").unwrap()))); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_about.get("about_rpfm").unwrap()))); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_about.get("open_manual").unwrap()))); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_about.get("check_updates").unwrap()))); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_about.get("check_schema_updates").unwrap()))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        unsafe { app_ui.about_qt.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        //---------------------------------------------------------------------------------------//
        // Preparing initial state of the Main Window...
        //---------------------------------------------------------------------------------------//

        // Put the stuff we need to move to the slots in Rc<Refcell<>>, so we can clone it without issues.
        let receiver_qt = Rc::new(RefCell::new(receiver_qt));
        let is_modified = Rc::new(RefCell::new(set_modified(false, &app_ui, None)));
        let is_packedfile_opened = Rc::new(RefCell::new(false));
        let is_folder_tree_view_locked = Rc::new(RefCell::new(false));
        let mymod_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let mode = Rc::new(RefCell::new(Mode::Normal));

        // Build the empty structs we need for certain features.
        let add_from_packfile_slots = Rc::new(RefCell::new(AddFromPackFileSlots::new()));
        let db_slots = Rc::new(RefCell::new(PackedFileDBTreeView::new()));
        let loc_slots = Rc::new(RefCell::new(PackedFileLocTreeView::new()));
        let text_slots = Rc::new(RefCell::new(PackedFileTextView::new()));
        let decoder_slots = Rc::new(RefCell::new(PackedFileDBDecoder::new()));
        let rigid_model_slots = Rc::new(RefCell::new(PackedFileRigidModelDataView::new()));

        let monospace_font = Rc::new(RefCell::new(Font::new(&QString::from_std_str("monospace [Consolas]"))));

        // Display the basic tips by default.
        display_help_tips(&app_ui);

        // Build the entire "MyMod" Menu.
        let result = build_my_mod_menu(
            rpfm_path.to_path_buf(),
            sender_qt.clone(),
            &sender_qt_data,
            receiver_qt.clone(),
            app_ui.clone(),
            &menu_bar_mymod,
            is_modified.clone(),
            mode.clone(),
            mymod_menu_needs_rebuild.clone(),
            &is_packedfile_opened
        );

        let mymod_stuff = Rc::new(RefCell::new(result.0));
        let mymod_stuff_slots = Rc::new(RefCell::new(result.1));

        // Disable all the Contextual Menu actions by default.
        unsafe {
            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
        }

        // Set the shortcuts for these actions.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("add_file").unwrap()))); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("add_folder").unwrap()))); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("add_from_packfile").unwrap()))); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("create_folder").unwrap()))); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("create_db").unwrap()))); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("create_loc").unwrap()))); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("create_text").unwrap()))); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("mass_import_tsv").unwrap()))); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("mass_export_tsv").unwrap()))); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("delete").unwrap()))); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("extract").unwrap()))); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("rename").unwrap()))); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("open_in_decoder").unwrap()))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TreeView, so the shortcuts work.
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_file); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_folder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_from_packfile); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_folder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_db); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_loc); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_text); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_mass_import_tsv); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_mass_export_tsv); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_delete); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_extract); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_rename); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_decoder); }

        // Set the current "Operational Mode" to `Normal`.
        set_my_mod_mode(&mymod_stuff, &mode, None);

        //---------------------------------------------------------------------------------------//
        // Action messages in the Status Bar...
        //---------------------------------------------------------------------------------------//

        // Menu bar, PackFile.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Creates a new PackFile and open it. Remember to save it later if you want to keep it!")); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open an existing PackFile.")); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Save the changes made in the currently open PackFile to disk.")); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_status_tip(&QString::from_std_str("Save the currently open PackFile as a new PackFile, instead of overwriting the original one.")); }
        unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Boot. You should never use it.")); }
        unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Release. You should never use it.")); }
        unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Patch. You should never use it.")); }
        unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Mod. You should use this for mods that should show up in the Mod Manager.")); }
        unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Movie. You should use this for mods that'll always be active, and will not show up in the Mod Manager.")); }
        unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Other. This is for PackFiles without write support, so you should never use it.")); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the Preferences/Settings dialog.")); }
        unsafe { app_ui.quit.as_mut().unwrap().set_status_tip(&QString::from_std_str("Exit the Program.")); }

        unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the data of the PackedFiles in this PackFile is encrypted. Saving this kind of PackFiles is NOT SUPPORTED.")); }
        unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the PackedFile Index of this PackFile includes the 'Last Modified' date of every PackedFile. Note that PackFiles with this enabled WILL NOT SHOW UP as mods in the official launcher.")); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the PackedFile Index of this PackFile is encrypted. Saving this kind of PackFiles is NOT SUPPORTED.")); }
        unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the header of this PackFile is extended by 20 bytes. Only seen in Arena PackFiles with encryption. Saving this kind of PackFiles is NOT SUPPORTED.")); }

        // Menu bar, Game Selected.
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer 2' as 'Game Selected'.")); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer' as 'Game Selected'.")); }
        unsafe { app_ui.attila.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Attila' as 'Game Selected'.")); }
        unsafe { app_ui.rome_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Rome 2' as 'Game Selected'.")); }
        unsafe { app_ui.arena.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Arena' as 'Game Selected'.")); }

        // Menu bar, Special Stuff.
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.")); }
        unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_status_tip(&QString::from_std_str("Create prefabs from exported maps. Currently bugged, so don't use it.")); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.")); }
        unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_status_tip(&QString::from_std_str("Create prefabs from exported maps. Currently bugged, so don't use it.")); }

        // Menu bar, About.
        unsafe { app_ui.about_qt.as_mut().unwrap().set_status_tip(&QString::from_std_str("Info about Qt, the UI Toolkit used to make this program.")); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_status_tip(&QString::from_std_str("Info about RPFM.")); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open RPFM's Manual in a PDF Reader.")); }
        unsafe { app_ui.patreon_link.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open RPFM's Patreon page. Even if you are not interested in becoming a Patron, check it out. I post info about the next updates and in-dev features from time to time.")); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_status_tip(&QString::from_std_str("Checks if there is any update available for RPFM.")); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_status_tip(&QString::from_std_str("Checks if there is any update available for the schemas. This is what you have to use after a game's patch.")); }

        // Context Menu.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add one or more files to the currently open PackFile. Existing files are not overwriten!")); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add a folder to the currently open PackFile. Existing files are not overwriten!")); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add files from another PackFile to the currently open PackFile. Existing files are not overwriten!")); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create an empty folder. Due to how the PackFiles are done, these are NOT KEPT ON SAVING if they stay empty.")); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Loc File (used by the game to store the texts you see ingame) in the selected folder.")); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a DB Table (used by the game for... most of the things).")); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Plain Text File. It accepts different extensions, like '.xml', '.lua', '.txt',....")); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a bunch of TSV files at the same time. It automatically checks if they are DB Tables, Locs or invalid TSVs, and imports them all at once. Existing files will be overwritten!")); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export every DB Table and Loc PackedFile from this PackFile as TSV files at the same time. Existing files will be overwritten!")); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete the selected File/Folder.")); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_status_tip(&QString::from_std_str("Extract the selected File/Folder from the PackFile.")); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_status_tip(&QString::from_std_str("Rename a File/Folder. Remember, whitespaces are NOT ALLOWED.")); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the selected table in the DB Decoder. To create/update schemas.")); }

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
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the new Game Selected.
                let mut new_game_selected;
                unsafe { new_game_selected = QString::to_std_string(&app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()); }

                // Remove the '&' from the game's name, and turn it into a `folder_name`.
                if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
                let new_game_selected_folder_name = new_game_selected.replace(' ', "_").to_lowercase();

                // Change the Game Selected in the Background Thread.
                sender_qt.send(Commands::SetGameSelected).unwrap();
                sender_qt_data.send(Data::String(new_game_selected_folder_name)).unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Get the response from the background thread.
                let response = if let Data::GameSelectedBool(data) = check_message_validity_tryrecv(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Get the current settings.
                sender_qt.send(Commands::GetSettings).unwrap();
                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Disable the "PackFile Management" actions.
                enable_packfile_actions(&app_ui, &response.0, &mymod_stuff, settings.clone(), false);

                // If we have a PackFile opened, re-enable the "PackFile Management" actions, so the "Special Stuff" menu gets updated properly.
                if !response.1 { enable_packfile_actions(&app_ui, &response.0, &mymod_stuff, settings, true); }

                // Set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                set_my_mod_mode(&mymod_stuff, &mode, None);

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // "Game Selected" Menu Actions.
        unsafe { app_ui.warhammer_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.warhammer.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.attila.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.rome_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.arena.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }

        // Try to get the Game Selected. This should never fail, so CTD if it does it.
        sender_qt.send(Commands::GetGameSelected).unwrap();
        let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Update the "Game Selected" here, so we can skip some steps when initializing.
        match &*game_selected.game {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
            "arena" => unsafe { app_ui.arena.as_mut().unwrap().trigger(); }
            "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
        }

        //-----------------------------------------------------//
        // "PackFile" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "New PackFile" action.
        let slot_new_packfile = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            mymod_stuff,
            mode,
            is_packedfile_opened,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

                    // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                    purge_them_all(&app_ui, &is_packedfile_opened);

                    // Show the "Tips".
                    display_help_tips(&app_ui);

                    // Tell the Background Thread to create a new PackFile.
                    sender_qt.send(Commands::NewPackFile).unwrap();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Wait until you get the PackFile's type.
                    let pack_file_type = if let Data::U32(data) = check_message_validity_tryrecv(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // We choose the right option, depending on our PackFile (In this case, it's usually mod).
                    match pack_file_type {
                        0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                        1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                        2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                        3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                        4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                        _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                    }

                    // By default, the four bitmask should be false.
                    unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                    // Update the TreeView.
                    update_treeview(
                        &rpfm_path,
                        &sender_qt,
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.window,
                        app_ui.folder_tree_view,
                        app_ui.folder_tree_model,
                        TreeViewOperation::Build(false),
                    );

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Set the new mod as "Not modified".
                    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                    // Try to get the Game Selected. This should never fail, so CTD if it does it.
                    sender_qt.send(Commands::GetGameSelected).unwrap();
                    let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Try to get the settings.
                    sender_qt.send(Commands::GetSettings).unwrap();
                    let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &game_selected, &mymod_stuff, settings, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    set_my_mod_mode(&mymod_stuff, &mode, None);
                }
            }
        ));

        // What happens when we trigger the "Open PackFile" action.
        let slot_open_packfile = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            is_packedfile_opened,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

                    // Create the FileDialog to get the PackFile to open.
                    let mut file_dialog;
                    unsafe { file_dialog = FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Open PackFile"),
                    )); }

                    // Filter it so it only shows PackFiles.
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));

                    // Try to get the Game Selected. This should never fail, so CTD if it does it.
                    sender_qt.send(Commands::GetGameSelected).unwrap();
                    let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // In case we have a default path for the Game Selected, we use it as base path for opening files.
                    if let Some(ref path) = game_selected.game_data_path {

                        // We check that actually exists before setting it.
                        if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(&path.to_string_lossy().as_ref().to_owned())); }
                    }

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Get the path of the selected file and turn it in a Rust's PathBuf.
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &rpfm_path,
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            path,
                            &app_ui,
                            &mymod_stuff,
                            &is_modified,
                            &mode,
                            "",
                            &is_packedfile_opened
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let slot_save_packfile = SlotBool::new(clone!(
            is_modified,
            sender_qt,
            receiver_qt => move |_| {

                // Tell the Background Thread to save the PackFile.
                sender_qt.send(Commands::SavePackFile).unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Check what happened when we tried to save the PackFile.
                match check_message_validity_tryrecv(&receiver_qt) {

                    // If we succeed, we should receive the new "Last Modified Time".
                    Data::U32(date) => {

                        // Set the mod as "Not Modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                        // Update the "Last Modified Date" of the PackFile in the TreeView.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(i64::from(date), 0)))); }
                    }

                    // If it's an error...
                    Data::Error(error) => {

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If the PackFile is not a file, we trigger the "Save Packfile As" action and break the loop.
                            ErrorKind::PackFileIsNotAFile => unsafe { Action::trigger(app_ui.save_packfile_as.as_mut().unwrap()); },

                            // If there was any other error while saving the PackFile, report it and break the loop.
                            ErrorKind::SavePackFileGeneric(_) => show_dialog(app_ui.window, false, error),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR)
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }              
            }
        ));

        // What happens when we trigger the "Save PackFile As" action.
        let slot_save_packfile_as = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Try to get the Game Selected. This should never fail, so CTD if it does it.
                sender_qt.send(Commands::GetGameSelected).unwrap();
                let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Tell the Background Thread that we want to save the PackFile, and wait for confirmation.
                sender_qt.send(Commands::SavePackFileAs).unwrap();

                // Check what response we got.
                match check_message_validity_recv2(&receiver_qt) {

                    // If we got confirmation....
                    Data::PackFileExtraData(extra_data) => {

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
                            let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                            // Pass it to the worker thread.
                            sender_qt_data.send(Data::PathBuf(path.to_path_buf())).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Check what happened when we tried to save the PackFile.
                            match check_message_validity_tryrecv(&receiver_qt) {

                                // If we succeed, we should receive the new "Last Modified Time".
                                Data::U32(date) => {

                                    // Update the "Last Modified Date" of the PackFile in the TreeView.
                                    unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(i64::from(date), 0)))); }

                                    // Get the Selection Model and the Model Index of the PackFile's Cell.
                                    let selection_model;
                                    let model_index;
                                    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                                    unsafe { model_index = app_ui.folder_tree_model.as_ref().unwrap().index((0, 0)); }

                                    // Select the PackFile's Cell with a "Clear & Select".
                                    unsafe { selection_model.as_mut().unwrap().select((&model_index, Flags::from_int(3))); }

                                    // Rename it with the new name.
                                    update_treeview(
                                        &rpfm_path,
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Rename(TreePathType::PackFile, path.file_name().unwrap().to_string_lossy().as_ref().to_owned()),
                                    );

                                    // Set the mod as "Not Modified".
                                    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                                    set_my_mod_mode(&mymod_stuff, &mode, None);

                                    // Report success.
                                    show_dialog(app_ui.window, true, "PackFile successfully saved."); 
                                }

                                // If it's an error...
                                Data::Error(error) => {
                                    match error.kind() {
                                        ErrorKind::SavePackFileGeneric(_) => show_dialog(app_ui.window, false, error),
                                        _ => panic!(THREADS_MESSAGE_ERROR),
                                    }
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR)
                            }

                            // Re-enable the Main Window.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                        }

                        // Otherwise, we take it as we canceled the save in some way, so we tell the
                        // Background Loop to stop waiting.
                        else { sender_qt_data.send(Data::Cancel).unwrap(); }
                    }

                    // If there was an error...
                    Data::Error(error) => {

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If the PackFile is non-editable, we show the error.
                            ErrorKind::PackFileIsNonEditable => show_dialog(app_ui.window, false, error),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR)
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
                sender_qt.send(Commands::SetPackFileType).unwrap();
                sender_qt_data.send(Data::U32(packfile_type)).unwrap();

                // TODO: Make the PackFile become Yellow.
                // Set the mod as "Modified".
                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
            }
        ));

        // What happens when we change the value of "Include Last Modified Date" action.
        let slot_index_includes_timestamp = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data => move |_| {

                // Get the current value of the action.
                let state;
                unsafe { state = app_ui.change_packfile_type_index_includes_timestamp.as_ref().unwrap().is_checked(); }

                // Send the new state to the background thread.
                sender_qt.send(Commands::ChangeIndexIncludesTimestamp).unwrap();
                sender_qt_data.send(Data::Bool(state)).unwrap();
            }
        ));

        // What happens when we trigger the "Preferences" action.
        let slot_preferences = SlotBool::new(clone!(
            mode,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            mymod_stuff,
            mymod_menu_needs_rebuild => move |_| {

                // Try to get the current Settings. This should never fail, so CTD if it does it.
                sender_qt.send(Commands::GetSettings).unwrap();
                let old_settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Create the Settings Dialog. If we got new settings...
                if let Some(settings) = SettingsDialog::create_settings_dialog(&app_ui, &old_settings, &sender_qt, &sender_qt_data, receiver_qt.clone()) {

                    // Send the signal to save them.
                    sender_qt.send(Commands::SetSettings).unwrap();
                    sender_qt_data.send(Data::Settings(settings.clone())).unwrap();

                    // Check what response we got.
                    match check_message_validity_recv2(&receiver_qt) {

                        // If we got confirmation....
                        Data::Success => {

                            // If we changed the "MyMod's Folder" path...
                            if settings.paths.get("mymods_base_path").unwrap() != old_settings.paths.get("mymods_base_path").unwrap() {

                                // We disable the "MyMod" mode, but leave the PackFile open, so the user doesn't lose any unsaved change.
                                set_my_mod_mode(&mymod_stuff, &mode, None);

                                // Then set it to recreate the "MyMod" submenu next time we try to open it.
                                *mymod_menu_needs_rebuild.borrow_mut() = true;
                            }

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // update the current `GameSelected`.
                            let mut games_with_changed_paths = vec![];
                            for (key, value) in settings.paths.iter() {
                                if key != "mymods_base_path" {
                                    if old_settings.paths.get(key).unwrap() != value {
                                        games_with_changed_paths.push(key.to_owned());
                                    }
                                }
                            } 

                            // Get the current GameSelected.
                            sender_qt.send(Commands::GetGameSelected).unwrap();
                            let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // If our current `GameSelected` is in the `games_with_changed_paths` list...
                            if games_with_changed_paths.contains(&game_selected.game) {

                                // Re-select the same game, so `GameSelected` update his paths.
                                unsafe { Action::trigger(app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap()); }
                            }
                        }

                        // If we got an error...
                        Data::Error(error) => {

                            // We must check what kind of error it's.
                            match error.kind() {

                                // If there was and IO error while saving the settings, report it.
                                ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound | ErrorKind::IOGeneric => show_dialog(app_ui.window, false, error.kind()),

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR)
                            }
                        }

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR)
                    }
                }
            }
        ));

        // What happens when we trigger the "Quit" action.
        let slot_quit = SlotBool::new(clone!(
            app_ui,
            is_modified => move |_| {
                if are_you_sure(&app_ui, &is_modified, false) {
                    unsafe { app_ui.window.as_mut().unwrap().close(); }
                }
            }
        ));

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
        unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_ref().unwrap().signals().triggered().connect(&slot_index_includes_timestamp); }

        unsafe { app_ui.preferences.as_ref().unwrap().signals().triggered().connect(&slot_preferences); }
        unsafe { app_ui.quit.as_ref().unwrap().signals().triggered().connect(&slot_quit); }

        //-----------------------------------------------------//
        // "Special Stuff" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "Patch Siege AI" action.
        let slot_patch_siege_ai = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // Ask the background loop to create the Dependency PackFile.
                sender_qt.send(Commands::PatchSiegeAI).unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Get the data from the patching operation...
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::StringVecTreePathType(response) => {

                        // Get the success message and show it.
                        show_dialog(app_ui.window, true, &response.0);

                        // For each file to delete...
                        for item_type in response.1 {

                            // Remove it from the TreeView.
                            update_treeview(
                                &rpfm_path,
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::DeleteUnselected(item_type),
                            );
                        }

                        // Set the mod as "Not Modified", because this action includes saving the PackFile.
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                        // Trigger a save and break the loop.
                        unsafe { Action::trigger(app_ui.save_packfile.as_mut().unwrap()); }
                    }

                    // If it's an error...
                    Data::Error(error) => {

                        // We must check what kind of error it's.
                        match error.kind() {

                            // If the PackFile is empty, report it.
                            ErrorKind::PatchSiegeAIEmptyPackFile => show_dialog(app_ui.window, false, error.kind()),

                            // If no patchable files have been found, report it and break the loop.
                            ErrorKind::PatchSiegeAINoPatchableFiles => show_dialog(app_ui.window, false, error.kind()),

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR)
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Optimize PackFile" action.
        let slot_optimize_packfile = SlotBool::new(clone!(
            rpfm_path,
            is_packedfile_opened,
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // This cannot be done if there is a PackedFile open.
                if *is_packedfile_opened.borrow() { return show_dialog(app_ui.window, false, ErrorKind::OperationNotAllowedWithPackedFileOpen); }
            
                // Ask the background loop to create the Dependency PackFile.
                sender_qt.send(Commands::OptimizePackFile).unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Get the data from the operation...
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::VecTreePathType(response) => {

                        // Get the success message and show it.
                        show_dialog(app_ui.window, true, "PackFile optimized and saved.");

                        // For each file to delete...
                        for item_type in response {

                            // Remove it from the TreeView.
                            update_treeview(
                                &rpfm_path,
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::DeleteUnselected(item_type),
                            );
                        }

                        // Trigger a save and break the loop.
                        unsafe { Action::trigger(app_ui.save_packfile.as_mut().unwrap()); }
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // "Special Stuff" Menu Actions.        
        unsafe { app_ui.wh2_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }
        unsafe { app_ui.wh_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }

        unsafe { app_ui.wh2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.wh_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.att_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.rom2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }

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
                            <li>TW: Arena research and coding: <b>Trolldemorted</b>.</li>
                            <li>TreeView Icons made by <a href=\"https://www.flaticon.com/authors/smashicons\" title=\"Smashicons\">Smashicons</a> from <a href=\"https://www.flaticon.com/\" title=\"Flaticon\">www.flaticon.com</a>. Licensed under <a href=\"http://creativecommons.org/licenses/by/3.0/\" title=\"Creative Commons BY 3.0\" target=\"_blank\">CC 3.0 BY</a>
                        </ul>

                        <h3>Special thanks</h3>
                        <ul style=\"list-style-type: disc\">
                            <li><b>PFM team</b>, for providing the community with awesome modding tools.</li>
                            <li><b>CA</b>, for being a mod-friendly company.</li>
                        </ul>
                        ", &VERSION))
                    );
                }
            }
        );

        // What happens when we trigger the "Open Manual" action.
        let slot_open_manual = SlotBool::new(clone!(
            rpfm_path => move |_| { 
                let mut manual_path = format!("{:?}", rpfm_path.to_path_buf().join(PathBuf::from("rpfm_manual.pdf")));

                // In linux we have to remove the commas.
                if cfg!(target_os = "linux") { 
                    manual_path.remove(0);
                    manual_path.pop();
                }
                
                // No matter how many times I tried, it's IMPOSSIBLE to open a file on windows, so instead we use this magic crate that seems to work everywhere.
                if let Err(error) = open::that(manual_path) {
                    show_dialog(app_ui.window, false, error);
                }
            }
        ));

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
        unsafe { app_ui.open_manual.as_ref().unwrap().signals().triggered().connect(&slot_open_manual); }
        unsafe { app_ui.patreon_link.as_ref().unwrap().signals().triggered().connect(&slot_patreon_link); }
        unsafe { app_ui.check_updates.as_ref().unwrap().signals().triggered().connect(&slot_check_updates); }
        unsafe { app_ui.check_schema_updates.as_ref().unwrap().signals().triggered().connect(&slot_check_schema_updates); }

        //-----------------------------------------------------//
        // TreeView "Contextual" Menu...
        //-----------------------------------------------------//

        // Slot to enable/disable contextual actions depending on the selected item.
        let slot_contextual_menu_enabler = SlotItemSelectionRefItemSelectionRef::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |selection,_| {

                // Get the path of the selected item.
                let path = get_path_from_item_selection(app_ui.folder_tree_model, &selection, true);

                // Try to get the TreePathType. This should never fail, so CTD if it does it.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(path)).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Depending on the type of the selected item, we enable or disable different actions.
                match item_type {

                    // If it's a file...
                    TreePathType::File(data) => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                        }

                        // If it's a DB, we should enable this too.
                        if !data.0.is_empty() && data.0.starts_with(&["db".to_owned()]) && data.0.len() == 3 {
                            unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(true); }
                        }
                    },

                    // If it's a folder...
                    TreePathType::Folder(_) => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // If it's the PackFile...
                    TreePathType::PackFile => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                        }
                    },

                    // If there is nothing selected...
                    TreePathType::None => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                        }
                    },
                }

                // Ask the other thread if there is a Dependency Database loaded.
                sender_qt.send(Commands::IsThereADependencyDatabase).unwrap();
                let is_there_a_dependency_database = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Ask the other thread if there is a Schema loaded.
                sender_qt.send(Commands::IsThereASchema).unwrap();
                let is_there_a_schema = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || !is_there_a_schema {
                    unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false); }
                    unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false); }
                    unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false); }
                }
            }
        ));

        // Slot to show the Contextual Menu for the TreeView.
        let slot_folder_tree_view_context_menu = SlotQtCorePointRef::new(move |_| {
            folder_tree_view_context_menu.exec2(&Cursor::pos());
        });

        // Trigger the "Enable/Disable" slot every time we change the selection in the TreeView.
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slot_contextual_menu_enabler); }

        // Action to show the Contextual Menu for the Treeview.
        unsafe { (app_ui.folder_tree_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slot_folder_tree_view_context_menu); }

        // What happens when we trigger the "Add File/s" action in the Contextual Menu.
        let slot_contextual_menu_add_file = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            rpfm_path => move |_| {

                // Create the FileDialog to get the file/s to add.
                let mut file_dialog;
                unsafe { file_dialog = FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Add File/s"),
                )); }

                // Set it to allow to add multiple files at once.
                file_dialog.set_file_mode(FileMode::ExistingFiles);

                // Depending on the current Operational Mode...
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // Get the settings.
                        sender_qt.send(Commands::GetSettings).unwrap();
                        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref mymods_base_path) = settings.paths.get("mymods_base_path").unwrap() {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() {
                                if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
                                }
                            }

                            // Set the base directory of the File Chooser to be the assets folder of the MyMod.
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the files we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                                let mut paths_packedfile = if paths[0].starts_with(&assets_folder) {

                                    // Get their final paths in the PackFile.
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {

                                        // Get his path, and turn it into a Vec<String>;
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }

                                    // Return the new paths for the TreeView.
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {

                                    // Get their final paths in the PackFile.
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths { paths_packedfile.append(&mut get_path_from_pathbuf(&app_ui, &path, true)); }

                                    // Return the new paths for the TreeView.
                                    paths_packedfile
                                };

                                // Tell the Background Thread to add the files.
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        update_treeview(
                                            &rpfm_path,
                                            &sender_qt,
                                            &sender_qt_data,
                                            receiver_qt.clone(),
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                    }

                                    // If we got an error...
                                    Data::Error(error) => {

                                        // We must check what kind of error it's.
                                        match error.kind() {

                                            // If it's an IO error, report it and break the loop.
                                            ErrorKind::IOGeneric | ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound => {
                                                show_dialog(app_ui.window, false, error);
                                            }

                                            // In ANY other situation, it's a message problem.
                                            _ => panic!(THREADS_MESSAGE_ERROR)
                                        }
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR),
                                }

                                // Re-enable the Main Window.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If it's in "Normal" mode...
                    Mode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the files we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get their final paths in the PackFile.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &paths { paths_packedfile.append(&mut get_path_from_pathbuf(&app_ui, &path, true)); }

                            // Tell the Background Thread to add the files.
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &rpfm_path,
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                }

                                // If we got an error...
                                Data::Error(error) => {

                                    // We must check what kind of error it's.
                                    match error.kind() {

                                        // If it's an IO error, report it and break the loop.
                                        ErrorKind::IOGeneric | ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound => {
                                            show_dialog(app_ui.window, false, error);
                                        }

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR)
                                    }
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }

                            // Re-enable the Main Window.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Add Folder/s" action in the Contextual Menu.
        let slot_contextual_menu_add_folder = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            rpfm_path => move |_| {

                // Create the FileDialog to get the folder/s to add.
                let mut file_dialog;
                unsafe { file_dialog = FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Add Folder/s"),
                )); }

                // TODO: Make this able to select multiple directories at once.
                // Set it to only allow selecting directories.
                file_dialog.set_file_mode(FileMode::Directory);

                // Depending on the current Operational Mode...
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // Get the settings.
                        sender_qt.send(Commands::GetSettings).unwrap();
                        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref mymods_base_path) = settings.paths.get("mymods_base_path").unwrap() {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() {
                                if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
                                }
                            }

                            // Set the base directory of the File Chooser to be the assets folder of the MyMod.
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the folders we want to add.
                                let mut folder_paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Get the Paths of the files inside the folders we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                                // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                                let mut paths_packedfile = if paths[0].starts_with(&assets_folder) {

                                    // Get their final paths in the PackFile.
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {

                                        // Get his path, and turn it into a Vec<String>;
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }

                                    // Return the new paths for the TreeView.
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {

                                    // Get their final paths in the PackFile.
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &folder_paths { paths_packedfile.append(&mut get_path_from_pathbuf(&app_ui, &path, false)); }

                                    // Return the new paths for the TreeView.
                                    paths_packedfile
                                };

                                // Tell the Background Thread to add the files.
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        update_treeview(
                                            &rpfm_path,
                                            &sender_qt,
                                            &sender_qt_data,
                                            receiver_qt.clone(),
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                    }

                                    // If we got an error...
                                    Data::Error(error) => {

                                        // We must check what kind of error it's.
                                        match error.kind() {

                                            // If it's an IO error, report it and break the loop.
                                            ErrorKind::IOGeneric | ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound => {
                                                show_dialog(app_ui.window, false, error);
                                            }

                                            // In ANY other situation, it's a message problem.
                                            _ => panic!(THREADS_MESSAGE_ERROR)
                                        }
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR),
                                }

                                // Re-enable the Main Window.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If it's in "Normal" mode, we just get the paths of the files inside them and add those files.
                    Mode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the folders we want to add.
                            let mut folder_paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get the Paths of the files inside the folders we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                            // Get their final paths in the PackFile.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &folder_paths { paths_packedfile.append(&mut get_path_from_pathbuf(&app_ui, &path, false)); }

                            // Tell the Background Thread to add the files.
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &rpfm_path,
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                }

                                // If we got an error...
                                Data::Error(error) => {

                                    // We must check what kind of error it's.
                                    match error.kind() {

                                        // If it's an IO error, report it and break the loop.
                                        ErrorKind::IOGeneric | ErrorKind::IOPermissionDenied | ErrorKind::IOFileNotFound => {
                                            show_dialog(app_ui.window, false, error);
                                        }

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR)
                                    }
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }

                            // Re-enable the Main Window.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Add from PackFile" action in the Contextual Menu.
        let slot_contextual_menu_add_from_packfile = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_packedfile_opened,
            is_folder_tree_view_locked,
            is_modified,
            add_from_packfile_slots,
            rpfm_path => move |_| {

                // Create the FileDialog to get the PackFile to open.
                let mut file_dialog;
                unsafe { file_dialog = FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Select PackFile"),
                )); }

                // Filter it so it only shows PackFiles.
                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));

                // Run it and expect a response (1 => Accept, 0 => Cancel).
                if file_dialog.exec() == 1 {

                    // Get the path of the selected file and turn it in a Rust's PathBuf.
                    let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                    // Tell the Background Thread to open the secondary PackFile.
                    sender_qt.send(Commands::OpenPackFileExtra).unwrap();
                    sender_qt_data.send(Data::PathBuf(path)).unwrap();

                    // Disable the Main Window (so we can't do other stuff).
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Get the data from the operation...
                    match check_message_validity_tryrecv(&receiver_qt) {
                        
                        // If it's success....
                        Data::Success => {

                            // Block the main `TreeView` from decoding stuff.
                            *is_folder_tree_view_locked.borrow_mut() = true;

                            // Destroy whatever it's in the PackedFile's View.
                            purge_them_all(&app_ui, &is_packedfile_opened);

                            // Build the TreeView to hold all the Extra PackFile's data and save his slots.
                            *add_from_packfile_slots.borrow_mut() = AddFromPackFileSlots::new_with_grid(
                                rpfm_path.to_path_buf(),
                                sender_qt.clone(),
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui,
                                &is_folder_tree_view_locked,
                                &is_modified,
                                &is_packedfile_opened,
                            );
                        }

                        // If we got an error...
                        Data::Error(error) => {

                            // We must check what kind of error it's.
                            match error.kind() {

                                // If it's the "Generic" error, re-enable the main window and return it.
                                ErrorKind::OpenPackFileGeneric(_) => {
                                    show_dialog(app_ui.window, false, error);
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR)
                            }
                        }

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            }
        ));

        // What happens when we trigger the "Create Folder" Action.
        let slot_contextual_menu_create_folder = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Create the "New Folder" dialog and wait for a new name (or a cancelation).
                if let Some(new_folder_name) = create_new_folder_dialog(&app_ui) {

                    // Get his Path, including the name of the PackFile.
                    let mut complete_path = get_path_from_selection(&app_ui, false);

                    // Add the folder's name to the list.
                    complete_path.push(new_folder_name);

                    // Check if the folder exists.
                    sender_qt.send(Commands::FolderExists).unwrap();
                    sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                    let folder_exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // If the folder already exists, return an error.
                    if folder_exists { return show_dialog(app_ui.window, false, ErrorKind::FolderAlreadyInPackFile)}

                    // Add it to the PackFile.
                    sender_qt.send(Commands::CreateFolder).unwrap();
                    sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();

                    // Add the new Folder to the TreeView.
                    update_treeview(
                        &rpfm_path,
                        &sender_qt,
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.window,
                        app_ui.folder_tree_view,
                        app_ui.folder_tree_model,
                        TreeViewOperation::Add(vec![complete_path; 1]),
                    );
                }
            }
        ));

        // What happens when we trigger the "Create DB PackedFile" Action.
        let slot_contextual_menu_create_packed_file_db = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, PackedFileType::DB("".to_owned(), "".to_owned(), 0)) {

                    // Check what we got to create....
                    match packed_file_type {

                        // If we got correct data from the dialog...
                        Ok(packed_file_type) => {

                            // Get the name of the PackedFile.
                            if let PackedFileType::DB(name, table,_) = packed_file_type.clone() {

                                // If the name is not empty...
                                if !name.is_empty() {

                                    // Get his Path, without the name of the PackFile.
                                    let mut complete_path = vec!["db".to_owned(), table, name];

                                    // Check if the PackedFile already exists.
                                    sender_qt.send(Commands::PackedFileExists).unwrap();
                                    sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                                    let exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                    // If the folder already exists, return an error.
                                    if exists { return show_dialog(app_ui.window, false, ErrorKind::FileAlreadyInPackFile)}

                                    // Add it to the PackFile.
                                    sender_qt.send(Commands::CreatePackedFile).unwrap();
                                    sender_qt_data.send(Data::VecStringPackedFileType((complete_path.to_vec(), packed_file_type.clone()))).unwrap();

                                    // Get the response, just in case it failed.
                                    match check_message_validity_recv2(&receiver_qt) {
                                        Data::Success => {
                                            // Add the new Folder to the TreeView.
                                            update_treeview(
                                                &rpfm_path,
                                                &sender_qt,
                                                &sender_qt_data,
                                                receiver_qt.clone(),
                                                app_ui.window,
                                                app_ui.folder_tree_view,
                                                app_ui.folder_tree_model,
                                                TreeViewOperation::Add(vec![complete_path; 1]),
                                            );

                                            // Set it as modified. Exception for the Paint system.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                        }

                                        Data::Error(error) => show_dialog(app_ui.window, false, error),

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR),
                                    }
                                }

                                // Otherwise, the name is invalid.
                                else { show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }
                            }
                        }

                        // If we got an error while trying to prepare the dialog, report it.
                        Err(error) => show_dialog(app_ui.window, false, error),
                    }
                }
            }
        ));

        // What happens when we trigger the "Create Loc PackedFile" Action.
        let slot_contextual_menu_create_packed_file_loc = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // TODO: Replace this with a result.
                // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, PackedFileType::Loc("".to_owned())) {

                    // Check what we got to create....
                    match packed_file_type {

                        // If we got correct data from the dialog...
                        Ok(packed_file_type) => {

                            // Get the name of the PackedFile.
                            if let PackedFileType::Loc(mut name) = packed_file_type.clone() {

                                // If the name is not empty...
                                if !name.is_empty() {

                                    // If the name doesn't end in a ".loc" termination, call it ".loc".
                                    if !name.ends_with(".loc") {
                                        name.push_str(".loc");
                                    }

                                    // Get his Path, without the name of the PackFile.
                                    let mut complete_path = get_path_from_selection(&app_ui, false);

                                    // Add the folder's name to the list.
                                    complete_path.push(name);

                                    // Check if the PackedFile already exists.
                                    sender_qt.send(Commands::PackedFileExists).unwrap();
                                    sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                                    let exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                    // If the folder already exists, return an error.
                                    if exists { return show_dialog(app_ui.window, false, ErrorKind::FileAlreadyInPackFile)}

                                    // Add it to the PackFile.
                                    sender_qt.send(Commands::CreatePackedFile).unwrap();
                                    sender_qt_data.send(Data::VecStringPackedFileType((complete_path.to_vec(), packed_file_type.clone()))).unwrap();

                                    // Get the response, just in case it failed.
                                    match check_message_validity_recv2(&receiver_qt) {
                                        Data::Success => {
                                            // Add the new Folder to the TreeView.
                                            update_treeview(
                                                &rpfm_path,
                                                &sender_qt,
                                                &sender_qt_data,
                                                receiver_qt.clone(),
                                                app_ui.window,
                                                app_ui.folder_tree_view,
                                                app_ui.folder_tree_model,
                                                TreeViewOperation::Add(vec![complete_path; 1]),
                                            );

                                            // Set it as modified. Exception for the Paint system.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                        }

                                        Data::Error(error) => show_dialog(app_ui.window, false, error),

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR),
                                    }
                                }

                                // Otherwise, the name is invalid.
                                else { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }
                            }
                        }

                        // If we got an error while trying to prepare the dialog, report it.
                        Err(error) => show_dialog(app_ui.window, false, error),
                    }
                }
            }
        ));

        // What happens when we trigger the "Create Text PackedFile" Action.
        let slot_contextual_menu_create_packed_file_text = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, PackedFileType::Text("".to_owned())) {

                    // Check what we got to create....
                    match packed_file_type {

                        // If we got correct data from the dialog...
                        Ok(packed_file_type) => {

                            // Get the name of the PackedFile.
                            if let PackedFileType::Text(mut name) = packed_file_type.clone() {

                                // If the name is not empty...
                                if !name.is_empty() {

                                    // If the name doesn't end in a text termination, call it .txt.
                                    if !name.ends_with(".lua") &&
                                        !name.ends_with(".xml") &&
                                        !name.ends_with(".xml.shader") &&
                                        !name.ends_with(".xml.material") &&
                                        !name.ends_with(".variantmeshdefinition") &&
                                        !name.ends_with(".environment") &&
                                        !name.ends_with(".lighting") &&
                                        !name.ends_with(".wsmodel") &&
                                        !name.ends_with(".csv") &&
                                        !name.ends_with(".tsv") &&
                                        !name.ends_with(".inl") &&
                                        !name.ends_with(".battle_speech_camera") &&
                                        !name.ends_with(".bob") &&
                                        !name.ends_with(".cindyscene") &&
                                        !name.ends_with(".cindyscenemanager") &&
                                        !name.ends_with(".txt") {
                                        name.push_str(".txt");
                                    }

                                    // Get his Path, without the name of the PackFile.
                                    let mut complete_path = get_path_from_selection(&app_ui, false);

                                    // Add the folder's name to the list.
                                    complete_path.push(name);

                                    // Check if the PackedFile already exists.
                                    sender_qt.send(Commands::PackedFileExists).unwrap();
                                    sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                                    let exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                    // If the folder already exists, return an error.
                                    if exists { return show_dialog(app_ui.window, false, ErrorKind::FileAlreadyInPackFile)}

                                    // Add it to the PackFile.
                                    sender_qt.send(Commands::CreatePackedFile).unwrap();
                                    sender_qt_data.send(Data::VecStringPackedFileType((complete_path.to_vec(), packed_file_type.clone()))).unwrap();

                                    // Get the response, just in case it failed.
                                    match check_message_validity_recv2(&receiver_qt) {
                                        Data::Success => {
                                            // Add the new Folder to the TreeView.
                                            update_treeview(
                                                &rpfm_path,
                                                &sender_qt,
                                                &sender_qt_data,
                                                receiver_qt.clone(),
                                                app_ui.window,
                                                app_ui.folder_tree_view,
                                                app_ui.folder_tree_model,
                                                TreeViewOperation::Add(vec![complete_path; 1]),
                                            );

                                            // Set it as modified. Exception for the Paint system.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                        }

                                        Data::Error(error) => show_dialog(app_ui.window, false, error),

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR),
                                    }
                                }

                                // Otherwise, the name is invalid.
                                else { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }
                            }
                        }

                        // If we got an error while trying to prepare the dialog, report it.
                        Err(error) => show_dialog(app_ui.window, false, error),
                    }
                }
            }
        ));

        // What happens when we trigger the "Mass-Import TSV" Action.
        let slot_contextual_menu_mass_import_tsv = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Create the "Mass-Import TSV" dialog and wait for his data (or a cancelation).
                if let Some(data) = create_mass_import_tsv_dialog(&app_ui) {

                    // If there is no name...
                    if data.0.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }

                    // If there is no file selected...
                    else if data.1.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::NoFilesToImport) }

                    // Otherwise...
                    else {

                        // Try to import them.
                        sender_qt.send(Commands::MassImportTSV).unwrap();
                        sender_qt_data.send(Data::StringVecPathBuf(data)).unwrap();

                        // Disable the Main Window (so we can't do other stuff).
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                        // Get the data from the operation...
                        match check_message_validity_tryrecv(&receiver_qt) {
                            
                            // If it's success....
                            Data::VecVecStringVecVecString(paths) => {

                                // Get the list of paths to add, removing those we "replaced".
                                let mut paths_to_add = paths.1.to_vec();
                                paths_to_add.retain(|x| !paths.0.contains(&x));

                                // Update the TreeView.
                                update_treeview(
                                    &rpfm_path,
                                    &sender_qt,
                                    &sender_qt_data,
                                    receiver_qt.clone(),
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(paths_to_add),
                                );

                                // Set it as modified. Exception for the paint system.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                            }

                            // If we got an error...
                            Data::Error(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If it's one of the "Mass-Import" specific errors...
                                    ErrorKind::MassImport(_) => {
                                        show_dialog(app_ui.window, true, error);
                                    }

                                    // If one or more files failed to get extracted due to an IO error...
                                    ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => {
                                        show_dialog(app_ui.window, true, error);
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR)
                                }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Mass-Export TSV" Action.
        let slot_contextual_menu_mass_export_tsv = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get a "Folder-only" FileDialog.
                let export_path;
                unsafe {export_path = FileDialog::get_existing_directory_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Select destination folder")
                )); }

                // If we got an export path and it's not empty...
                if !export_path.is_empty() {

                    // Get the Path we choose to export the TSV files in a readable format.
                    let export_path = PathBuf::from(export_path.to_std_string());

                    // If the folder is a valid folder...
                    if export_path.is_dir() {

                        // Tell the Background Thread to export all the tables and loc files there.
                        sender_qt.send(Commands::MassExportTSV).unwrap();
                        sender_qt_data.send(Data::PathBuf(export_path)).unwrap();

                        // Disable the Main Window (so we can't do other stuff).
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                        // Get the data from the operation...
                        match check_message_validity_tryrecv(&receiver_qt) {
                            
                            // If it's success....
                            Data::String(response) => show_dialog(app_ui.window, true, response),

                            // If we got an error...
                            Data::Error(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If one or more files failed to get extracted due to an IO error...
                                    ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => {
                                        show_dialog(app_ui.window, true, error);
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR)
                                }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let slot_contextual_menu_delete = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_packedfile_opened,
            is_modified,
            rpfm_path => move |_| {

                // If there is a PackedFile opened, we show a message with the explanation of why
                // we can't delete the selected file/folder.
                if *is_packedfile_opened.borrow() {
                    show_dialog(app_ui.window, false, ErrorKind::DeletePackedFilesWithPackedFileOpen)
                }

                // Otherwise, we continue the deletion process.
                else {

                    // Get his Path, including the name of the PackFile.
                    let path = get_path_from_selection(&app_ui, true);

                    // In case there is nothing selected, don't try to delete.
                    if path.is_empty() { return }

                    // Tell the Background Thread to delete the selected stuff.
                    sender_qt.send(Commands::DeletePackedFile).unwrap();
                    sender_qt_data.send(Data::VecString(path)).unwrap();

                    // Get the response from the other thread.
                    match check_message_validity_recv2(&receiver_qt) {

                        // Only if the deletion was successful, we update the UI.
                        Data::TreePathType(path_type) => {

                            // Update the TreeView.
                            update_treeview(
                                &rpfm_path,
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::DeleteSelected(path_type),
                            );

                            // Set the mod as "Modified". For now, we don't paint deletions.
                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                        }

                        // This can fail if, for some reason, the command gets resended for one file.
                        Data::Error(error) => {
                            if error.kind() != ErrorKind::Generic { panic!(THREADS_MESSAGE_ERROR); }
                        }
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }
                }
            }
        ));

        // What happens when we trigger the "Extract" action in the Contextual Menu.
        let slot_contextual_menu_extract = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            mode => move |_| {

                // Get his Path, including the name of the PackFile.
                let path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(path.to_vec())).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Get the settings.
                sender_qt.send(Commands::GetSettings).unwrap();
                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Depending on the current Operational Mode...
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref mymods_base_path) = settings.paths.get("mymods_base_path").unwrap() {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() {
                                if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
                                }
                            }

                            // Get the path of the selected item without the PackFile's name.
                            let mut path_without_packfile = path.to_vec();
                            path_without_packfile.reverse();
                            path_without_packfile.pop();
                            path_without_packfile.reverse();

                            // If it's a file or a folder...
                            if item_type == TreePathType::File((vec![String::new()], 1)) || item_type == TreePathType::Folder(vec![String::new()]) {

                                // For each folder in his path...
                                for (index, folder) in path_without_packfile.iter().enumerate() {

                                    // Complete the extracted path.
                                    assets_folder.push(folder);

                                    // The last thing in the path is the new file, so we don't have to create a folder for it.
                                    if index < (path_without_packfile.len() - 1) {
                                        if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                            return show_dialog(app_ui.window, false, ErrorKind::IOCreateNestedAssetFolder);
                                        }
                                    }
                                }
                            }

                            // Tell the Background Thread to delete the selected stuff.
                            sender_qt.send(Commands::ExtractPackedFile).unwrap();
                            sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), assets_folder.to_path_buf()))).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Check what response we got.
                            match check_message_validity_tryrecv(&receiver_qt) {
                            
                                // If it's success....
                                Data::String(response) => show_dialog(app_ui.window, true, response),

                                // If we got an error...
                                Data::Error(error) => {

                                    // We must check what kind of error it's.
                                    match error.kind() {

                                        // TODO: make sure this works properly.
                                        // If one or more files failed to get extracted due to an error...
                                        ErrorKind::ExtractError(_) => {
                                            show_dialog(app_ui.window, true, error);
                                        }

                                        // If one or more files failed to get extracted due to an IO error...
                                        ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => {
                                            show_dialog(app_ui.window, true, error);
                                        }

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR)
                                    }
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }

                            // Re-enable the Main Window.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If we are in "Normal" Mode....
                    Mode::Normal => {

                        // If we want the old PFM behavior (extract full path)...
                        if *settings.settings_bool.get("use_pfm_extracting_behavior").unwrap() {

                            // Get a "Folder-only" FileDialog.
                            let extraction_path;
                            unsafe {extraction_path = FileDialog::get_existing_directory_unsafe((
                                app_ui.window as *mut Widget,
                                &QString::from_std_str("Extract File/Folder")
                            )); }

                            // If we got a path...
                            if !extraction_path.is_empty() {

                                // If we are trying to extract the PackFile...
                                let final_extraction_path =
                                    if let TreePathType::PackFile = item_type {

                                        // Get the Path we choose to save the file/folder and return it.
                                        PathBuf::from(extraction_path.to_std_string())
                                    }

                                    // Otherwise, we use a more complex method.
                                    else {

                                        // Get the Path we choose to save the file/folder.
                                        let mut base_extraction_path = PathBuf::from(extraction_path.to_std_string());

                                        // Add the full path to the extraction path.
                                        let mut addon_path = path.to_vec();
                                        addon_path.reverse();
                                        addon_path.pop();
                                        addon_path.reverse();

                                        // Store the last item.
                                        let final_field = addon_path.pop().unwrap();

                                        // Put together the big path.
                                        let mut final_extraction_path = base_extraction_path.join(addon_path.iter().collect::<PathBuf>());

                                        // Create that directory.
                                        DirBuilder::new().recursive(true).create(&final_extraction_path).unwrap();

                                        // Add back the final item to the path.
                                        final_extraction_path.push(&final_field);

                                        // Return the path.
                                        final_extraction_path
                                };

                                // Tell the Background Thread to delete the selected stuff.
                                sender_qt.send(Commands::ExtractPackedFile).unwrap();
                                sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), final_extraction_path.to_path_buf()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Check what response we got.
                                match check_message_validity_tryrecv(&receiver_qt) {
                                
                                    // If it's success....
                                    Data::String(response) => show_dialog(app_ui.window, true, response),

                                    // If we got an error...
                                    Data::Error(error) => {

                                        // We must check what kind of error it's.
                                        match error.kind() {

                                            // TODO: make sure this works properly.
                                            // If one or more files failed to get extracted due to an error...
                                            ErrorKind::ExtractError(_) => {
                                                show_dialog(app_ui.window, true, error);
                                            }

                                            // If one or more files failed to get extracted due to an IO error...
                                            ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => {
                                                show_dialog(app_ui.window, true, error);
                                            }

                                            // In ANY other situation, it's a message problem.
                                            _ => panic!(THREADS_MESSAGE_ERROR)
                                        }
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR),
                                }

                                // Re-enable the Main Window.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                            }
                        }

                        // Otherwise, get the default FileDialog.
                        else {

                            let mut file_dialog;
                            unsafe { file_dialog = FileDialog::new_unsafe((
                                app_ui.window as *mut Widget,
                                &QString::from_std_str("Extract File/Folder"),
                            )); }

                            // Set it to save mode.
                            file_dialog.set_accept_mode(AcceptMode::Save);

                            // Ask for confirmation in case of overwrite.
                            file_dialog.set_confirm_overwrite(true);

                            // Depending of the item type, change the dialog.
                            match item_type {

                                // If we have selected a file/folder, use his name as default names.
                                TreePathType::File((path,_)) | TreePathType::Folder(path) => file_dialog.select_file(&QString::from_std_str(&path.last().unwrap())),

                                // For the rest, use the name of the PackFile.
                                _ => {

                                    // Get the name of the PackFile and use it as default name.
                                    let model_index;
                                    let name;
                                    unsafe { model_index = app_ui.folder_tree_model.as_ref().unwrap().index((0, 0)); }
                                    name = model_index.data(()).to_string();
                                    file_dialog.select_file(&name);
                                }
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Path we choose to save the file/folder.
                                let mut extraction_path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                                // Tell the Background Thread to delete the selected stuff.
                                sender_qt.send(Commands::ExtractPackedFile).unwrap();
                                sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), extraction_path.to_path_buf()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Check what response we got.
                                match check_message_validity_tryrecv(&receiver_qt) {
                                
                                    // If it's success....
                                    Data::String(response) => show_dialog(app_ui.window, true, response),

                                    // If we got an error...
                                    Data::Error(error) => {

                                        // We must check what kind of error it's.
                                        match error.kind() {

                                            // TODO: make sure this works properly.
                                            // If one or more files failed to get extracted due to an error...
                                            ErrorKind::ExtractError(_) => {
                                                show_dialog(app_ui.window, true, error);
                                            }

                                            // If one or more files failed to get extracted due to an IO error...
                                            ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => {
                                                show_dialog(app_ui.window, true, error);
                                            }

                                            // In ANY other situation, it's a message problem.
                                            _ => panic!(THREADS_MESSAGE_ERROR)
                                        }
                                    }

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR),
                                }

                                // Re-enable the Main Window.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                            }
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Open in decoder" action in the Contextual Menu.
        let slot_contextual_menu_open_decoder = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_packedfile_opened => move |_| {

                // Get his Path, including the name of the PackFile.
                let path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(path)).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // If it's a PackedFile...
                if let TreePathType::File((_, index)) = item_type {

                    // Remove everything from the PackedFile View.
                    purge_them_all(&app_ui, &is_packedfile_opened);

                    // We try to open it in the decoder.
                    if let Ok(result) = PackedFileDBDecoder::create_decoder_view(
                        sender_qt.clone(),
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        &index
                    ) {

                        // Save the monospace font an the slots.
                        *decoder_slots.borrow_mut() = result.0;
                        *monospace_font.borrow_mut() = result.1;
                    }

                    // Disable the "Change game selected" function, so we cannot change the current schema with an open table.
                    unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(false); }
                }
            }
        ));

        // Contextual Menu Actions.
        unsafe { app_ui.context_menu_add_file.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_file); }
        unsafe { app_ui.context_menu_add_folder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_folder); }
        unsafe { app_ui.context_menu_add_from_packfile.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_from_packfile); }
        unsafe { app_ui.context_menu_create_folder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_folder); }
        unsafe { app_ui.context_menu_create_db.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_db); }
        unsafe { app_ui.context_menu_create_loc.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_loc); }
        unsafe { app_ui.context_menu_create_text.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_text); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_mass_import_tsv); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_mass_export_tsv); }
        unsafe { app_ui.context_menu_delete.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_delete); }
        unsafe { app_ui.context_menu_extract.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_extract); }
        unsafe { app_ui.context_menu_open_decoder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_open_decoder); }

        //-----------------------------------------------------------------------------------------//
        // Rename Action. Due to me not understanding how the edition of a TreeView works, we do it
        // in a special way.
        //-----------------------------------------------------------------------------------------//

        // What happens when we trigger the "Rename" Action.
        let slot_contextual_menu_rename = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get his Path, including the name of the PackFile.
                let complete_path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Depending on the type of the selection...
                match item_type.clone() {

                    // If it's a file or a folder...
                    TreePathType::File((path,_)) | TreePathType::Folder(path) => {

                        // Get the name of the selected item.
                        let current_name = path.last().unwrap();

                        // Create the "Rename" dialog and wait for a new name (or a cancelation).
                        if let Some(new_name) = create_rename_dialog(&app_ui, &current_name) {

                            // Send the New Name to the Background Thread, wait for a response.
                            sender_qt.send(Commands::RenamePackedFile).unwrap();
                            sender_qt_data.send(Data::VecStringString((complete_path, new_name.to_owned()))).unwrap();

                            // Check what response we got.
                            match check_message_validity_recv2(&receiver_qt) {
                            
                                // If it's success....
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &rpfm_path,
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Rename(item_type, new_name),
                                    );

                                    // Set the mod as "Modified". This is an exception to the paint system.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
                                }

                                // If we got an error...
                                Data::Error(error) => {

                                    // We must check what kind of error it's.
                                    match error.kind() {

                                        // If the new name is empty, report it.
                                        ErrorKind::EmptyInput => show_dialog(app_ui.window, false, error),

                                        // If the new name contains invalid characters, report it.
                                        ErrorKind::InvalidInput => show_dialog(app_ui.window, false, error),

                                        // If the new name is the same as the old one, report it.
                                        ErrorKind::UnchangedInput => show_dialog(app_ui.window, false, error),

                                        // If the new name is already in use in the path, report it.
                                        ErrorKind::NameAlreadyInUseInThisPath => show_dialog(app_ui.window, false, error),

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!(THREADS_MESSAGE_ERROR)
                                    }
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR),
                            }
                        }
                    }

                    // Otherwise, it's the PackFile or None, and we return, as we can't rename that.
                    _ => return,
                }
            }
        ));

        // Action to start the Renaming Process.
        unsafe { app_ui.context_menu_rename.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_rename); }

        //-----------------------------------------------------//
        // Special Actions, like opening a PackedFile...
        //-----------------------------------------------------//

        // What happens when we try to open a PackedFile...
        let slot_open_packedfile = SlotNoArgs::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            is_folder_tree_view_locked,
            is_packedfile_opened => move || {

                // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here.
                if !(*is_folder_tree_view_locked.borrow()) {

                    // Destroy any children that the PackedFile's View we use may have, cleaning it.
                    purge_them_all(&app_ui, &is_packedfile_opened);

                    // Get the selection to see what we are going to open.
                    let selection;
                    unsafe { selection = app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                    // Get the path of the selected item.
                    let path = get_path_from_item_selection(app_ui.folder_tree_model, &selection, true);

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send(Commands::GetTypeOfPath).unwrap();
                    sender_qt_data.send(Data::VecString(path)).unwrap();
                    let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // We act, depending on his type.
                    match item_type {

                        // Only in case it's a file, we do something.
                        TreePathType::File((tree_path, index)) => {

                            // Get the name of the PackedFile (we are going to use it a lot).
                            let packedfile_name = tree_path.last().unwrap().to_owned();

                            // We get his type to decode it properly
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
                                        packedfile_name.ends_with(".cindyscene") ||
                                        packedfile_name.ends_with(".cindyscenemanager") ||
                                        //packedfile_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                                        packedfile_name.ends_with(".txt") { "TEXT" }

                                // If it ends in any of these, it's an image.
                                else if packedfile_name.ends_with(".jpg") ||
                                        packedfile_name.ends_with(".jpeg") ||
                                        packedfile_name.ends_with(".tga") ||
                                        packedfile_name.ends_with(".dds") ||
                                        packedfile_name.ends_with(".png") { "IMAGE" }

                                // Otherwise, we don't have a decoder for that PackedFile... yet.
                                else { "None" };

                            // Then, depending of his type we decode it properly (if we have it implemented support
                            // for his type).
                            match packed_file_type {
                                // TODO: Fix all these errors.
                                // If the file is a Loc PackedFile...
                                "LOC" => {

                                    // Try to get the view build, or return error.
                                    match PackedFileLocTreeView::create_tree_view(
                                        sender_qt.clone(),
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &is_modified,
                                        &app_ui,
                                        &index
                                    ) {
                                        Ok(new_loc_slots) => *loc_slots.borrow_mut() = new_loc_slots,
                                        Err(error) => return show_dialog(app_ui.window, false, ErrorKind::LocDecode(format!("{}", error))),
                                    }

                                    // Tell the program there is an open PackedFile.
                                    *is_packedfile_opened.borrow_mut() = true;
                                }

                                // If the file is a DB PackedFile...
                                "DB" => {

                                    // Try to get the view build, or return error.
                                    match PackedFileDBTreeView::create_table_view(
                                        sender_qt.clone(),
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &is_modified,
                                        &app_ui,
                                        &index
                                    ) {
                                        Ok(new_db_slots) => *db_slots.borrow_mut() = new_db_slots,
                                        Err(error) => return show_dialog(app_ui.window, false, ErrorKind::DBTableDecode(format!("{}", error))),
                                    }

                                    // Tell the program there is an open PackedFile.
                                    *is_packedfile_opened.borrow_mut() = true;

                                    // Disable the "Change game selected" function, so we cannot change the current schema with an open table.
                                    unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(false); }
                                }

                                // If the file is a Text PackedFile...
                                "TEXT" => {

                                    // Try to get the view build, or return error.
                                    match PackedFileTextView::create_text_view(
                                        sender_qt.clone(),
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &is_modified,
                                        &app_ui,
                                        &index
                                    ) {
                                        Ok(new_text_slots) => *text_slots.borrow_mut() = new_text_slots,
                                        Err(error) => return show_dialog(app_ui.window, false, ErrorKind::TextDecode(format!("{}", error))),
                                    }

                                    // Tell the program there is an open PackedFile.
                                    *is_packedfile_opened.borrow_mut() = true;
                                }

                                // If the file is a Text PackedFile...
                                "RIGIDMODEL" => {

                                    // Try to get the view build, or return error.
                                    match PackedFileRigidModelDataView::create_data_view(
                                        sender_qt.clone(),
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &is_modified,
                                        &app_ui,
                                        &index
                                    ) {
                                        Ok(new_rigid_model_slots) => *rigid_model_slots.borrow_mut() = new_rigid_model_slots,
                                        Err(error) => return show_dialog(app_ui.window, false, ErrorKind::RigidModelDecode(format!("{}", error))),
                                    }

                                    // Tell the program there is an open PackedFile.
                                    *is_packedfile_opened.borrow_mut() = true;
                                }

                                // If the file is a Text PackedFile...
                                "IMAGE" => {

                                    // Try to get the view build, or return error.
                                    if let Err(error) = ui::packedfile_image::create_image_view(
                                        sender_qt.clone(),
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &app_ui,
                                        &index
                                    ) { return show_dialog(app_ui.window, false, ErrorKind::ImageDecode(format!("{}", error))) }
                                }

                                // For any other PackedFile, just restore the display tips.
                                _ => display_help_tips(&app_ui),
                            }
                        }

                        // If it's anything else, then we just show the "Tips" list.
                        _ => display_help_tips(&app_ui),
                    }
                }
            }
        ));

        // Action to try to open a PackedFile.
        unsafe { app_ui.folder_tree_view.as_ref().unwrap().signals().activated().connect(&slot_open_packedfile); }

        // In windows "activated" means double click, so we need to add this action too to compensate it.
        if cfg!(target_os = "windows") {
            unsafe { app_ui.folder_tree_view.as_ref().unwrap().signals().clicked().connect(&slot_open_packedfile); }
        }

        //-----------------------------------------------------//
        // Show the Main Window and start everything...
        //-----------------------------------------------------//

        // We need to rebuild the MyMod menu while opening it if the variable for it is true.
        let slot_rebuild_mymod_menu = SlotNoArgs::new(clone!(
            rpfm_path,
            mymod_stuff,
            mymod_stuff_slots,
            sender_qt,
            is_packedfile_opened,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            mymod_menu_needs_rebuild => move || {

                // If we need to rebuild the "MyMod" menu...
                if *mymod_menu_needs_rebuild.borrow() {

                    // Then rebuild it.
                    let result = build_my_mod_menu(
                        rpfm_path.to_path_buf(),
                        sender_qt.clone(),
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.clone(),
                        &menu_bar_mymod,
                        is_modified.clone(),
                        mode.clone(),
                        mymod_menu_needs_rebuild.clone(),
                        &is_packedfile_opened
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

        // If we have an argument (we open RPFM by clicking in a PackFile directly)...
        if arguments.len() > 1 {

            // Turn the fist argument into a Path.
            let path = PathBuf::from(&arguments[1]);

            // If that argument it's a valid File (not Qt-related)...
            if path.is_file() {

                // Try to open it, and report it case of error.
                if let Err(error) = open_packfile(
                    &rpfm_path,
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    path,
                    &app_ui,
                    &mymod_stuff,
                    &is_modified,
                    &mode,
                    "",
                    &is_packedfile_opened
                ) { show_dialog(app_ui.window, false, error); }
            }
        }

        // Get the settings.
        sender_qt.send(Commands::GetSettings).unwrap();
        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // If we have it enabled in the prefs, check if there are updates.
        if *settings.settings_bool.get("check_updates_on_start").unwrap() { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if *settings.settings_bool.get("check_schema_updates_on_start").unwrap() { check_schema_updates(&app_ui, false, &rpfm_path, &sender_qt, &sender_qt_data, &receiver_qt) };

        // And launch it.
        Application::exec()
    })
}

/// This function enables or disables the actions from the `MenuBar` needed when we open a PackFile.
/// NOTE: To disable the "Special Stuff" actions, we use `enable` => false.
fn enable_packfile_actions(
    app_ui: &AppUI,
    game_selected: &GameSelected,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    settings: Settings,
    enable: bool
) {

    // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
    if game_selected.game == "arena" {

        // Disable the actions that allow to create and save PackFiles.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(false); }

        // This one too, though we had to deal with it specially later on.
        unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }
    }

    // Otherwise...
    else {

        // Enable or disable the actions from "PackFile" Submenu.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(true); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(enable); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(enable); }

        // If there is a "MyMod" path set in the settings...
        if let Some(ref path) = settings.paths.get("mymods_base_path").unwrap() {

            // And it's a valid directory, enable the "New MyMod" button.
            if path.is_dir() { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(true); }}

            // Otherwise, disable it.
            else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
        }

        // Otherwise, disable it.
        else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
    }

    // These actions are common, no matter what game we have.    
    unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().set_enabled(enable); }
    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_enabled(enable); }

    // If we are enabling...
    if enable {

        // Check the Game Selected and enable the actions corresponding to out game.
        match &*game_selected.game {
            "warhammer_2" => {
                unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "warhammer" => {
                unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_enabled(true); }
                unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "attila" => {
                unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "rome_2" => {
                unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            _ => {},
        }
    }

    // If we are disabling...
    else {

        // Disable Warhammer 2 actions...
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Warhammer actions...
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Attila actions...
        unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Rome 2 actions...
        unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(false); }
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

            // Enable all the "MyMod" related actions.
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
/// NOTE: The `game_folder` &str is for when using this function with "MyMods". If you're opening a
/// normal mod, pass an empty &str there.
fn open_packfile(
    rpfm_path: &PathBuf,
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    pack_file_path: PathBuf,
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    is_modified: &Rc<RefCell<bool>>,
    mode: &Rc<RefCell<Mode>>,
    game_folder: &str,
    is_packedfile_opened: &Rc<RefCell<bool>>,
) -> Result<()> {

    // Tell the Background Thread to create a new PackFile.
    sender_qt.send(Commands::OpenPackFile).unwrap();
    sender_qt_data.send(Data::PathBuf(pack_file_path.to_path_buf())).unwrap();

    // Disable the Main Window (so we can't do other stuff).
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

    // Check what response we got.
    match check_message_validity_tryrecv(&receiver_qt) {
    
        // If it's success....
        Data::PackFileHeader(header) => {

            // We choose the right option, depending on our PackFile.
            match header.pack_file_type {
                0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
            }

            // Enable or disable these, depending on what data we have in the header.
            unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(header.data_is_encrypted); }
            unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(header.index_includes_timestamp); }
            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(header.index_is_encrypted); }
            unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(header.header_is_extended); }

            // Update the TreeView.
            update_treeview(
                &rpfm_path,
                sender_qt,
                sender_qt_data,
                receiver_qt.clone(),
                app_ui.window,
                app_ui.folder_tree_view,
                app_ui.folder_tree_model,
                TreeViewOperation::Build(false),
            );

            // Set the new mod as "Not modified".
            *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

            // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
            if !game_folder.is_empty() {

                // NOTE: Arena should never be here.
                // Change the Game Selected in the UI.
                match game_folder {
                    "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
                    "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
                    "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                    "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                }

                // Set the current "Operational Mode" to `MyMod`.
                set_my_mod_mode(&mymod_stuff, mode, Some(pack_file_path));
            }

            // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
            else {

                // Depending on the Id, choose one game or another.
                match &*header.id {

                    // PFH5 is for Warhammer 2/Arena.
                    "PFH5" => {

                        // If the PackFile has the mysterious byte enabled, it's from Arena.
                        if header.header_is_extended { unsafe { app_ui.arena.as_mut().unwrap().trigger(); } }

                        // Otherwise, it's from Warhammer 2.
                        else { unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); } }
                    },

                    // PFH4 is for Warhammer 1/Attila.
                    "PFH4" | _ => {

                        // Get the Game Selected.
                        sender_qt.send(Commands::GetGameSelected).unwrap();
                        let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila.
                        // In any other case, we select Rome 2 by default.
                        match &*game_selected.game {
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                        }
                    },
                }

                // Set the current "Operational Mode" to `Normal`.
                set_my_mod_mode(&mymod_stuff, mode, None);
            }

            // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
            purge_them_all(&app_ui, &is_packedfile_opened);

            // Show the "Tips".
            display_help_tips(&app_ui);

            // Re-enable the Main Window.
            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

        }

        // If we got an error...
        Data::Error(error) => {

            // We must check what kind of error it's.
            match error.kind() {

                // If it's the "Generic" error, re-enable the main window and return it.
                ErrorKind::OpenPackFileGeneric(_) => {
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    return Err(error)
                }

                // In ANY other situation, it's a message problem.
                _ => panic!(THREADS_MESSAGE_ERROR)
            }
        }

        // In ANY other situation, it's a message problem.
        _ => panic!(THREADS_MESSAGE_ERROR),
    }

    // Return success.
    Ok(())
}

/// This function takes care of the re-creation of the "MyMod" list in the following moments:
/// - At the start of the program.
/// - At the end of MyMod deletion.
/// - At the end of MyMod creation.
/// - At the end of settings update.
/// We need to return a tuple with the actions (for further manipulation) and the slots (to keep them alive).
fn build_my_mod_menu(
    rpfm_path: PathBuf,
    sender_qt: Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: Rc<RefCell<Receiver<Data>>>,
    app_ui: AppUI,
    menu_bar_mymod: &*mut Menu,
    is_modified: Rc<RefCell<bool>>,
    mode: Rc<RefCell<Mode>>,
    needs_rebuild: Rc<RefCell<bool>>,
    is_packedfile_opened: &Rc<RefCell<bool>>
) -> (MyModStuff, MyModSlots) {

    // Get the current Settings, as we are going to need them later.
    sender_qt.send(Commands::GetSettings).unwrap();
    let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

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

    // And we create the slots.
    let mut mymod_slots = MyModSlots {

        // This slot is used for the "New MyMod" action.
        new_mymod: SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_packedfile_opened,
            app_ui,
            mode,
            settings,
            is_modified,
            needs_rebuild => move |_| {

                // Create the "New MyMod" Dialog, and get the result.
                match NewMyModDialog::create_new_mymod_dialog(&app_ui, &settings) {

                    // If we accepted...
                    Some(data) => {

                        // Get the info about the new MyMod.
                        let mod_name = data.0;
                        let mod_game = data.1;

                        // Get the PackFile's name.
                        let full_mod_name = format!("{}.pack", mod_name);

                        // Change the Game Selected to match the one we chose for the new "MyMod".
                        // NOTE: Arena should not be on this list.
                        match &*mod_game {
                            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                        }

                        // Get his new path from the base "MyMod" path + `mod_game`.
                        let mut mymod_path = settings.paths.get("mymods_base_path").unwrap().clone().unwrap();
                        mymod_path.push(&mod_game);

                        // Just in case the folder doesn't exist, we try to create it.
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path) {
                            return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
                        }

                        // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                        let mut mymod_path_private = mymod_path.to_path_buf();
                        mymod_path_private.push(&mod_name);
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path_private) {
                            return show_dialog(app_ui.window, false, ErrorKind::IOCreateNestedAssetFolder);
                        };

                        // Add the PackFile's name to the full path.
                        mymod_path.push(&full_mod_name);

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send(Commands::NewPackFile).unwrap();
                        let _ = if let Data::U32(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send(Commands::SavePackFileAs).unwrap();
                        let _ = if let Data::PackFileExtraData(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Pass the new PackFile's Path to the worker thread.
                        sender_qt_data.send(Data::PathBuf(mymod_path.to_path_buf())).unwrap();

                        // Check what response we got.
                        match check_message_validity_tryrecv(&receiver_qt) {
                        
                            // If it's success....
                            Data::U32(_) => {

                                // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                                purge_them_all(&app_ui, &is_packedfile_opened);

                                // Show the "Tips".
                                display_help_tips(&app_ui);

                                // Update the TreeView.
                                update_treeview(
                                    &rpfm_path,
                                    &sender_qt,
                                    &sender_qt_data,
                                    receiver_qt.clone(),
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Build(false),
                                );

                                // Mark it as "Mod" in the UI.
                                unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }

                                // By default, the four bitmask should be false.
                                unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                                unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                                // Set the new "MyMod" as "Not modified".
                                *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                                // Get the Game Selected.
                                sender_qt.send(Commands::GetGameSelected).unwrap();
                                let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                // Try to get the settings.
                                sender_qt.send(Commands::GetSettings).unwrap();
                                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                // Enable the actions available for the PackFile from the `MenuBar`.
                                enable_packfile_actions(&app_ui, &game_selected, &Rc::new(RefCell::new(mymod_stuff.clone())), settings, true);

                                // Set the current "Operational Mode" to `MyMod`.
                                set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, Some(mymod_path));

                                // Set it to rebuild next time we try to open the "MyMod" Menu.
                                *needs_rebuild.borrow_mut() = true;
                            },

                            // If we got an error...
                            Data::Error(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If there was any other error while saving the PackFile, report it and break the loop.
                                    ErrorKind::SavePackFileGeneric(_) => show_dialog(app_ui.window, false, error),

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR)
                                }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }

                    // If we canceled the creation of a "MyMod", just return.
                    None => return,
                }
            }
        )),

        // This slot is used for the "Delete Selected MyMod" action.
        delete_selected_mymod: SlotBool::new(clone!(
            sender_qt,
            receiver_qt,
            settings,
            mode,
            is_modified,
            app_ui => move |_| {

                // Ask before doing it, as this will permanently delete the mod from the Disk.
                if are_you_sure(&app_ui, &is_modified, true) {

                    // We want to keep our "MyMod" name for the success message, so we store it here.
                    let old_mod_name: String;

                    // Try to delete the "MyMod" and his folder.
                    let mod_deleted = match *mode.borrow() {

                        // If we have a "MyMod" selected...
                        Mode::MyMod {ref game_folder_name, ref mod_name} => {

                            // We save the name of the PackFile for later use.
                            old_mod_name = mod_name.to_owned();

                            // And the "MyMod" path is configured...
                            if let Some(ref mymods_base_path) = settings.paths.get("mymods_base_path").unwrap() {

                                // We get his path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // If the mod doesn't exist, return error.
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDoesntExist); }

                                // And we try to delete his PackFile. If it fails, return error.
                                if let Err(_) = remove_file(&mymod_path) {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![mymod_path; 1]));
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.to_path_buf();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(&mymod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDeletedFolderNotFound);
                                }

                                // If the assets folder exists, we try to delete it. Again, this is optional, so it should not stop the deleting process.
                                else if let Err(_) = remove_dir_all(&mymod_assets_path) {
                                    show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![mymod_assets_path; 1]));
                                }

                                // We return true, as we have delete the files of the "MyMod".
                                true
                            }

                            // If the "MyMod" path is not configured, return an error.
                            else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                        }

                        // If we don't have a "MyMod" selected, return an error.
                        Mode::Normal => return show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {

                        // Set the current "Operational Mode" to `Normal`.
                        set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, None);

                        // Create a "dummy" PackFile, effectively closing the currently open PackFile.
                        sender_qt.send(Commands::ResetPackFile).unwrap();

                        // Get the Game Selected.
                        sender_qt.send(Commands::GetGameSelected).unwrap();
                        let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Try to get the settings.
                        sender_qt.send(Commands::GetSettings).unwrap();
                        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Disable the actions available for the PackFile from the `MenuBar`.
                        enable_packfile_actions(&app_ui, &game_selected, &Rc::new(RefCell::new(mymod_stuff.clone())), settings, false);

                        // Clear the TreeView.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().clear(); }

                        // Set the dummy mod as "Not modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                        // Set it to rebuild next time we try to open the MyMod Menu.
                        *needs_rebuild.borrow_mut() = true;

                        // Show the "MyMod" deleted Dialog.
                        show_dialog(app_ui.window, true, format!("MyMod successfully deleted: \"{}\".", old_mod_name));
                    }
                }
            }
        )),

        // This slot is used for the "Install MyMod" action.
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
                        if let Some(ref mymods_base_path) = settings.paths.get("mymods_base_path").unwrap() {

                            // Get the Game Selected.
                            sender_qt.send(Commands::GetGameSelected).unwrap();
                            let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // Get the `game_data_path` of the game.
                            let game_data_path = game_selected.game_data_path.clone();

                            // If we have a `game_data_path` for the current `GameSelected`...
                            if let Some(mut game_data_path) = game_data_path {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // We check that the "MyMod"s PackFile exists.
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModPackFileDoesntExist); }

                                // We check that the destination path exists.
                                if !game_data_path.is_dir() {
                                    return show_dialog(app_ui.window, false, ErrorKind::MyModInstallFolderDoesntExists);
                                }

                                // Get the destination path for the PackFile with the PackFile name included.
                                game_data_path.push(&mod_name);

                                // And copy the PackFile to his destination. If the copy fails, return an error.
                                if let Err(_) = copy(mymod_path, game_data_path.to_path_buf()) {
                                    return show_dialog(app_ui.window, false, ErrorKind::IOGenericCopy(game_data_path));
                                }
                            }

                            // If we don't have a `game_data_path` configured for the current `GameSelected`...
                            else { return show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else { show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If we have no "MyMod" selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                }

            }
        )),

        // This slot is used for the "Uninstall MyMod" action.
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
                        sender_qt.send(Commands::GetGameSelected).unwrap();
                        let game_selected = if let Data::GameSelected(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Get the `game_data_path` of the game.
                        let game_data_path = game_selected.game_data_path.clone();

                        // If we have a `game_data_path` for the current `GameSelected`...
                        if let Some(mut game_data_path) = game_data_path {

                            // Get the destination path for the PackFile with the PackFile included.
                            game_data_path.push(&mod_name);

                            // We check that the "MyMod" is actually installed in the provided path.
                            if !game_data_path.is_file() { return show_dialog(app_ui.window, false, ErrorKind::MyModNotInstalled); }

                            // If the "MyMod" is installed, we remove it. If there is a problem deleting it, return an error dialog.
                            else if let Err(_) = remove_file(game_data_path.to_path_buf()) {
                                return show_dialog(app_ui.window, false, ErrorKind::IOGenericDelete(vec![game_data_path; 1]));
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else { show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
                    }

                    // If we have no MyMod selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, ErrorKind::MyModDeleteWithoutMyModSelected),
                }
            }
        )),

        // This is an empty list to populate later with the slots used to open every "MyMod" we have.
        open_mymod: vec![],
    };

    // "About" Menu Actions.
    unsafe { mymod_stuff.new_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.new_mymod); }
    unsafe { mymod_stuff.delete_selected_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.delete_selected_mymod); }
    unsafe { mymod_stuff.install_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.install_mymod); }
    unsafe { mymod_stuff.uninstall_mymod.as_ref().unwrap().signals().triggered().connect(&mymod_slots.uninstall_mymod); }

    // Status bar tips.
    unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a new MyMod.")); }
    unsafe { mymod_stuff.delete_selected_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete the currently selected MyMod.")); }
    unsafe { mymod_stuff.install_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Copy the currently selected MyMod into the data folder of the GameSelected.")); }
    unsafe { mymod_stuff.uninstall_mymod.as_mut().unwrap().set_status_tip(&QString::from_std_str("Removes the currently selected MyMod from the data folder of the GameSelected.")); }

    //---------------------------------------------------------------------------------------//
    // Build the "Dynamic" part of the menu...
    //---------------------------------------------------------------------------------------//

    // Add a separator for this section.
    unsafe { menu_bar_mymod.as_mut().unwrap().add_separator(); }

    // If we have the "MyMod" path configured...
    if let Some(ref mymod_base_path) = settings.paths.get("mymods_base_path").unwrap() {

        // And can get without errors the folders in that path...
        if let Ok(game_folder_list) = mymod_base_path.read_dir() {

            // We get all the games that have mods created (Folder exists and has at least a *.pack file inside).
            for game_folder in game_folder_list {

                // If the file/folder is valid...
                if let Ok(game_folder) = game_folder {

                    // Get the list of supported games folders.
                    let supported_folders = SUPPORTED_GAMES.iter().filter(|(_, x)| x.supports_editing == true).map(|(folder_name,_)| folder_name.to_string()).collect::<Vec<String>>();

                    // If it's a valid folder, and it's in our supported games list...
                    if game_folder.path().is_dir() && supported_folders.contains(&game_folder.file_name().to_string_lossy().as_ref().to_owned()) {

                        // We create that game's menu here.
                        let game_folder_name = game_folder.file_name().to_string_lossy().as_ref().to_owned();
                        let game_display_name = &SUPPORTED_GAMES.get(&*game_folder_name).unwrap().display_name;

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

                                    // Get this into an Rc so we can pass it to the "Open PackFile" closure.
                                    let mymod_stuff = Rc::new(RefCell::new(mymod_stuff.clone()));

                                    // Create the slot for that action.
                                    let slot_open_mod = SlotBool::new(clone!(
                                        rpfm_path,
                                        game_folder_name,
                                        is_modified,
                                        mode,
                                        mymod_stuff,
                                        pack_file,
                                        is_packedfile_opened,
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt => move |_| {

                                            // Check first if there has been changes in the PackFile.
                                            if are_you_sure(&app_ui, &is_modified, false) {

                                                // Open the PackFile (or die trying it!).
                                                if let Err(error) = open_packfile(
                                                    &rpfm_path,
                                                    &sender_qt,
                                                    &sender_qt_data,
                                                    &receiver_qt,
                                                    pack_file.to_path_buf(),
                                                    &app_ui,
                                                    &mymod_stuff,
                                                    &is_modified,
                                                    &mode,
                                                    &game_folder_name,
                                                    &is_packedfile_opened,
                                                ) { show_dialog(app_ui.window, false, error) }
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

    // If there is a "MyMod" path set in the settings...
    if let Some(ref path) = settings.paths.get("mymods_base_path").unwrap() {

        // And it's a valid directory, enable the "New MyMod" button.
        if path.is_dir() { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(true); }}

        // Otherwise, disable it.
        else { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(false); }}
    }

    // Otherwise, disable it.
    else { unsafe { mymod_stuff.new_mymod.as_mut().unwrap().set_enabled(false); }}

    // Disable by default the rest of the actions.
    unsafe { mymod_stuff.delete_selected_mymod.as_mut().unwrap().set_enabled(false); }
    unsafe { mymod_stuff.install_mymod.as_mut().unwrap().set_enabled(false); }
    unsafe { mymod_stuff.uninstall_mymod.as_mut().unwrap().set_enabled(false); }

    // Return the MyModStuff with all the new actions.
    (mymod_stuff, mymod_slots)
}
