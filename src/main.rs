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
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;
extern crate cpp_utils;

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
use qt_core::event_loop::EventLoop;
use qt_core::flags::Flags;
use qt_core::qt::{ContextMenuPolicy, ShortcutContext};
use qt_core::slots::{SlotBool, SlotNoArgs, SlotItemSelectionRefItemSelectionRef};
use cpp_utils::StaticCast;

use std::env::{args, temp_dir};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fs::{File, DirBuilder, copy, remove_file, remove_dir_all};
use std::io::{BufReader, Seek, SeekFrom, Read, Write};

use failure::Error;
use common::*;
use common::coding_helpers::*;
use packfile::packfile::{PackFile, PackFileExtraData, PackFileHeader};
use packedfile::*;
use packedfile::loc::*;
use packedfile::db::*;
use packedfile::db::schemas::*;
use packedfile::db::schemas_importer::*;
use packedfile::rigidmodel::*;
use settings::*;
use updater::*;
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

mod common;
mod ui;
mod packfile;
mod packedfile;
mod settings;
mod updater;

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This const is the standard message in case of message deserializing error.
const THREADS_MESSAGE_ERROR: &str = "<p>Error in thread communication system. If you see this, it's a bug. Please report it (so it can be fixed), including what you did just before this message pop up.</p>";

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
    pub change_packfile_type_music: *mut Action,
    pub change_packfile_type_other: *mut Action,

    pub change_packfile_type_index_has_extra_u32: *mut Action,
    pub change_packfile_type_index_is_encrypted: *mut Action,
    pub change_packfile_type_mysterious_byte: *mut Action,

    // Action Group for the submenu.
    pub change_packfile_type_group: *mut ActionGroup,

    //-------------------------------------------------------------------------------//
    // "Game Selected" menu.
    //-------------------------------------------------------------------------------//

    pub warhammer_2: *mut Action,
    pub warhammer: *mut Action,
    pub attila: *mut Action,
    pub arena: *mut Action,

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
    pub context_menu_delete: *mut Action,
    pub context_menu_extract: *mut Action,
    pub context_menu_rename: *mut Action,
    pub context_menu_open_decoder: *mut Action,
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
        // TODO: Make the size of the TreeView consistent.
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
        unsafe { menu_warhammer_2 = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer 2")); }
        unsafe { menu_warhammer = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer")); }
        unsafe { menu_attila = menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Attila")); }

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
                change_packfile_type_music: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("M&usic")),
                change_packfile_type_other: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Other")),

                change_packfile_type_index_has_extra_u32: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Index Has Extra U32")),
                change_packfile_type_index_is_encrypted: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("Index Is &Encrypted")),
                change_packfile_type_mysterious_byte: menu_change_packfile_type.as_mut().unwrap().add_action(&QString::from_std_str("&Has Mysterious Byte")),

                // Action Group for the submenu.
                change_packfile_type_group: ActionGroup::new(menu_change_packfile_type.as_mut().unwrap().static_cast_mut()).into_raw(),

                //-------------------------------------------------------------------------------//
                // "Game Selected" menu.
                //-------------------------------------------------------------------------------//

                warhammer_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Warhammer 2")),
                warhammer: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("War&hammer")),
                attila: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Attila")),
                arena: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("A&rena")),

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
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_music); }
        unsafe { app_ui.change_packfile_type_group.as_mut().unwrap().add_action_unsafe(app_ui.change_packfile_type_other); }
        unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_music.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checkable(true); }

        // These ones are individual, but they need to be checkable and not editable.
        unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_checkable(true); }

        unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_enabled(false); }
        unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_enabled(false); }

        // Put separators in the SubMenu.
        unsafe { menu_change_packfile_type.as_mut().unwrap().insert_separator(app_ui.change_packfile_type_other); }
        unsafe { menu_change_packfile_type.as_mut().unwrap().insert_separator(app_ui.change_packfile_type_index_has_extra_u32); }

        // The "Game Selected" Menu should be an ActionGroup.
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.warhammer); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.attila); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.arena); }
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.attila.as_mut().unwrap().set_checkable(true); }
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
                Err(error) => show_dialog(app_ui.window, false, format!("Error while creating a new DB Schema file:\n{}", error.cause())),
            }

            // Close the program with code 69
            return 69
        }

        //---------------------------------------------------------------------------------------//
        // Shortcuts for the Menu Bar...
        //---------------------------------------------------------------------------------------//

        // Set the shortcuts for these actions.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+n"))); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+o"))); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+s"))); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+shift+s"))); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+p"))); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+q"))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        //---------------------------------------------------------------------------------------//
        // Preparing initial state of the Main Window...
        //---------------------------------------------------------------------------------------//

        // Put the stuff we need to move to the slots in Rc<Refcell<>>, so we can clone it without issues.
        let receiver_qt = Rc::new(RefCell::new(receiver_qt));
        let is_modified = Rc::new(RefCell::new(false));
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

        // Get the Game Selected.
        sender_qt.send("get_game_selected").unwrap();
        let response = receiver_qt.borrow().recv().unwrap().unwrap();
        let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

        // Change the Game Selected in the UI.
        match &*game_selected.game {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checked(true); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().set_checked(true); }
            "arena" => unsafe { app_ui.arena.as_mut().unwrap().set_checked(true); }
            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().set_checked(true); }
        }

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
            supported_games.to_vec(),
            mymod_menu_needs_rebuild.clone(),
            &is_packedfile_opened
        );

        let mymod_stuff = Rc::new(RefCell::new(result.0));
        let mymod_stuff_slots = Rc::new(RefCell::new(result.1));

        // Disable the actions available for the PackFile from the `MenuBar`.
        enable_packfile_actions(&app_ui, &game_selected, false);

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
            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
        }

        // Set the shortcuts for these actions.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+shift+a"))); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+shift+f"))); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+shift+p"))); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+l"))); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+b"))); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+n"))); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+m"))); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+."))); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+del"))); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+e"))); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+r"))); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str("ctrl+j"))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
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
        unsafe { app_ui.change_packfile_type_music.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Music. Don't know much about this type, other than it's used in Attila and Arena for Music PackFiles.")); }
        unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_status_tip(&QString::from_std_str("Changes the PackFile's Type to Other. This is for PackFiles without write support, so you should never use it.")); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the Preferences/Settings dialog.")); }
        unsafe { app_ui.quit.as_mut().unwrap().set_status_tip(&QString::from_std_str("Exit the Program.")); }

        unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the PackedFile Index of this PackFile has an 4 random bytes before each PackedFile's Path. For now, saving this kind of PackFiles is NOT SUPPORTED.")); }
        unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the PackedFile Index of this PackFile is encrypted. Saving this kind of PackFiles is NOT SUPPORTED.")); }
        unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, this PackFile has a mysterious byte in the header. Only seen in Arena PackFiles. Saving this kind of PackFiles is NOT SUPPORTED.")); }

        // Menu bar, Game Selected.
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer 2' as 'Game Selected'.")); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer' as 'Game Selected'.")); }
        unsafe { app_ui.attila.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Attila' as 'Game Selected'.")); }
        unsafe { app_ui.arena.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Arena' as 'Game Selected'.")); }

        // Menu bar, Special Stuff.
        unsafe { app_ui.wh2_generate_dependency_pack.as_mut().unwrap().set_status_tip(&QString::from_std_str("Generate a 'Dependency' PackFile for 'TW:Warhammer 2'. Needed for some features. You have to do this again after every game's patch! Remember it!")); }
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.")); }
        unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_status_tip(&QString::from_std_str("Create prefabs from exported maps. Currently bugged, so don't use it.")); }
        unsafe { app_ui.wh_generate_dependency_pack.as_mut().unwrap().set_status_tip(&QString::from_std_str("Generate a 'Dependency' PackFile for 'TW:Warhammer'. Needed for some features. You have to do this again after every game's patch! Remember it!")); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.")); }
        unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_status_tip(&QString::from_std_str("Create prefabs from exported maps. Currently bugged, so don't use it.")); }
        unsafe { app_ui.att_generate_dependency_pack.as_mut().unwrap().set_status_tip(&QString::from_std_str("Generate a 'Dependency' PackFile for 'TW:Attila'. Needed for some features. You have to do this again after every game's patch! Remember it!")); }

        // Menu bar, About.
        unsafe { app_ui.about_qt.as_mut().unwrap().set_status_tip(&QString::from_std_str("Info about Qt, the UI Toolkit used to make this program.")); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_status_tip(&QString::from_std_str("Info about RPFM.")); }
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
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a bunch of TSV files at the same time. It automatically checks if they are DB Tables, Locs or invalid TSVs, and imports them all at once. Existing files are overwriten!")); }
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

                // When we finally receive the data...
                if let Ok(response) = receiver_qt.borrow().recv().unwrap() {

                    // Try to deserialize it.
                    match serde_json::from_slice(&response) {

                        // If it can be deserialized as a (GameSelected, bool)...
                        Ok(response) => {

                            // Redundant, but needed for the deserializer to know the type.
                            let response: (GameSelected, bool) = response;

                            // If the Game Selected is Arena, block any attempt of creating or saving a PackFile.
                            if response.0.game == "arena" {

                                // Disable the actions that allow to create and save PackFiles.
                                unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(false); }
                                unsafe { app_ui.save_packfile.as_mut().unwrap().set_enabled(false); }
                                unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_enabled(false); }

                                // This one too, though we had to deal with it specially later on.
                                unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }
                            }

                            // Otherwise, enable them.
                            else {

                                // Disable the actions that allow to create and save PackFiles.
                                unsafe { app_ui.new_packfile.as_mut().unwrap().set_enabled(true); }

                                // Disable the "PackFile Management" actions.
                                enable_packfile_actions(&app_ui, &response.0, false);

                                // If we have a PackFile opened, re-enable the "PackFile Management" actions, so the "Special Stuff" menu gets updated properly.
                                if !response.1 { enable_packfile_actions(&app_ui, &response.0, true); }

                                // Get the current settings.
                                sender_qt.send("get_settings").unwrap();

                                // Wait until you get something from the background thread.
                                if let Ok(settings) = receiver_qt.borrow().recv().unwrap() {

                                    // Try to deserialize it.
                                    match serde_json::from_slice(&settings) {

                                        // If it can be deserialized as a GameSelected...
                                        Ok(settings) => {

                                            // Redundant, but needed for the deserializer to know the type.
                                            let settings: Settings = settings;

                                            // If there is a "MyMod" path set in the settings...
                                            if let Some(ref path) = settings.paths.my_mods_base_path {

                                                // And it's a valid directory, enable the "New MyMod" button.
                                                if path.is_dir() { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(true); }}

                                                // Otherwise, disable it.
                                                else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
                                            }

                                            // Otherwise, disable it.
                                            else { unsafe { mymod_stuff.borrow().new_mymod.as_mut().unwrap().set_enabled(false); }}
                                        },

                                        // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                        Err(_) => return show_dialog(app_ui.window, false, THREADS_MESSAGE_ERROR),
                                    }
                                }
                            }

                            // Set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                            set_my_mod_mode(&mymod_stuff, &mode, None);
                        },

                        // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                        Err(_) => return show_dialog(app_ui.window, false, THREADS_MESSAGE_ERROR),
                    }
                }
            }
        ));

        // "Game Selected" Menu Actions.
        unsafe { app_ui.warhammer_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.warhammer.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.attila.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.arena.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }

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
                            let packed_file_type: u32 = serde_json::from_slice(&data).unwrap();

                            // We choose the right option, depending on our PackFile (In this case, it's usually mod).
                            match packed_file_type {
                                0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                                1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                                2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                                3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                                4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                                17 => unsafe { app_ui.change_packfile_type_music.as_mut().unwrap().set_checked(true); }
                                _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                            }

                            // By default, the three bitmask should be false.
                            unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_checked(false); }

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
                    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

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
                        ) {
                            show_dialog(app_ui.window, false, format!("Error while opening the PackFile:\n\n{}", error.cause()));
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let slot_save_packfile = SlotBool::new(clone!(
            is_modified,
            sender_qt,
            receiver_qt => move |_| {

                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                let mut event_loop = EventLoop::new();

                // Tell the Background Thread to save the PackFile.
                sender_qt.send("save_packfile").unwrap();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Until we receive a response from the worker thread...
                loop {

                    // When we finally receive the data...
                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                        // Check what the result of the saving process was.
                        match data {

                            // In case of success, set the mod as "Not Modified".
                            Ok(_) => *is_modified.borrow_mut() = set_modified(false, &app_ui, None),

                            // In case of error, we can have two results.
                            Err(error) => {

                                // If the error message is empty, we have no original file, so we trigger a "Save PackFile As" action.
                                if error.cause().to_string().is_empty() { unsafe { Action::trigger(app_ui.save_packfile_as.as_mut().unwrap()); } }

                                // Otherwise, it's an error, so we report it.
                                else { show_dialog(app_ui.window, false, format!("<p>Error while saving the PackFile:</p><p>{}</p>", error.cause())); }
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
            rpfm_path,
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
                                            *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

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

                                            // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                                            set_my_mod_mode(&mymod_stuff, &mode, None);

                                            // Report success.
                                            show_dialog(app_ui.window, true, "PackFile successfully saved.");
                                        }

                                        // In case of error, we report it.
                                        Err(error) => show_dialog(app_ui.window, false, format!("Error while saving the PackFile:\n\n{}", error.cause())),
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
                    Err(error) => show_dialog(app_ui.window, false, error.cause()),
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
                    "M&usic" => 17,
                    _ => 99,
                }; }

                // Send the type to the Background Thread.
                sender_qt.send("set_packfile_type").unwrap();
                sender_qt_data.send(serde_json::to_vec(&packfile_type).map_err(From::from)).unwrap();

                // TODO: Make the PackFile become Yellow.
                // Set the mod as "Modified".
                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);
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
            mymod_menu_needs_rebuild => move |_| {

                // Request the current Settings.
                sender_qt.send("get_settings").unwrap();

                let settings_encoded = receiver_qt.borrow().recv().unwrap().unwrap();
                let old_settings: Settings = serde_json::from_slice(&settings_encoded).unwrap();

                // Create the Settings Dialog. If we got new settings...
                if let Some(settings) = SettingsDialog::create_settings_dialog(&app_ui, &old_settings, &supported_games) {

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
                        Err(error) => show_dialog(app_ui.window, false, format!("Error while saving the Settings:\n\n{}", error.cause())),
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
        unsafe { app_ui.change_packfile_type_music.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_other.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }

        unsafe { app_ui.preferences.as_ref().unwrap().signals().triggered().connect(&slot_preferences); }
        unsafe { app_ui.quit.as_ref().unwrap().signals().triggered().connect(&slot_quit); }

        //-----------------------------------------------------//
        // "Special Stuff" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "Generate Dependency Pack" action.
        let slot_generate_dependency_pack = SlotBool::new(clone!(
            receiver_qt,
            sender_qt => move |_| {

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
                                show_dialog(app_ui.window, true, message);

                                // Reload the Dependency PackFile for our Game Selected.
                                sender_qt.send("set_dependency_database").unwrap();
                            }

                            // In case of error, report the error.
                            Err(error) => show_dialog(app_ui.window, false, error.cause()),
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
            rpfm_path,
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
                                let result: (String, Vec<TreePathType>) = serde_json::from_slice(&data).unwrap();
                                show_dialog(app_ui.window, true, &result.0);

                                // Set the mod as "Not Modified", because this action includes saving the PackFile.
                                *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                                // For each file to delete...
                                for item_type in result.1 {

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
                            }

                            // In case of error, report the error.
                            Err(error) => show_dialog(app_ui.window, false, error.cause()),
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
                            <li>TreeView Icons made by <a href=\"https://www.flaticon.com/authors/smashicons\" title=\"Smashicons\">Smashicons</a> from <a href=\"https://www.flaticon.com/\" title=\"Flaticon\">www.flaticon.com</a>. Licensed under <a href=\"http://creativecommons.org/licenses/by/3.0/\" title=\"Creative Commons BY 3.0\" target=\"_blank\">CC 3.0 BY</a>
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
        // TreeView "Contextual" Menu...
        //-----------------------------------------------------//

        // Slot to enable/disable contextual actions depending on the selected item.
        let slot_contextual_menu_enabler = SlotItemSelectionRefItemSelectionRef::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |selection,_| {

                // Get the path of the selected item.
                let path = get_path_from_item_selection(app_ui.folder_tree_model, &selection, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send("get_type_of_path").unwrap();
                sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                // If we trigger certain actions quick enough, the message system will mess up and this can be err.
                if let Ok(response) = receiver_qt.borrow().recv().unwrap() {

                    // Due to syncronisation issues, this can be error. So check it properly.
                    let item_type: TreePathType = match serde_json::from_slice(&response) {
                        Ok(item_type) => item_type,
                        Err(_) => return
                    };

                    // Depending on the type of the selected item, we enable or disable different actions.
                    match item_type {

                        // If it's a file...
                        TreePathType::File(data) => {
                            unsafe {
                                app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                                app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                                app_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            }

                            // If it's a DB, we should enable this too.
                            if !data.0.is_empty() && data.0.starts_with(&["db".to_owned()]) {
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
                                app_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                                app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            }
                        },
                    }

                    // Ask the other thread if there is a Dependency Database loaded.
                    sender_qt.send("is_there_a_dependency_database").unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let is_there_a_dependency_database: bool = serde_json::from_slice(&response).unwrap();

                    // Ask the other thread if there is a Schema loaded.
                    sender_qt.send("is_there_a_schema").unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let is_there_a_schema: bool = serde_json::from_slice(&response).unwrap();

                    // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables.
                    if !is_there_a_dependency_database || !is_there_a_schema {
                        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false); }
                        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false); }
                    }
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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

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
                            sender_qt.send("get_settings").unwrap();
                            let settings = receiver_qt.borrow().recv().unwrap().unwrap();
                            let settings: Settings = serde_json::from_slice(&settings).unwrap();

                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.paths.my_mods_base_path {

                                // We get the assets folder of our mod (without .pack extension).
                                let mut assets_folder = my_mods_base_path.to_path_buf();
                                assets_folder.push(&game_folder_name);
                                assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists, and create it if it doesn't.
                                if !assets_folder.is_dir() {
                                    if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                        return show_dialog(app_ui.window, false, "Error while adding files:\n The MyMod's asset folder does not exists and it cannot be created.");
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
                                    sender_qt.send("add_packedfile").unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&paths).map_err(From::from)).unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&paths_packedfile).map_err(From::from)).unwrap();

                                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                                    let mut event_loop = EventLoop::new();

                                    // Disable the Main Window (so we can't do other stuff).
                                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                    // Until we receive a response from the worker thread...
                                    loop {

                                        // When we finally receive a response...
                                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                                            // If we got a response...
                                            if let Ok(data) = data {

                                                // Get the list of errors.
                                                let error_list: Vec<Vec<String>> = serde_json::from_slice(&data).unwrap();

                                                // If there is any error, report it. Otherwise, it's a success.
                                                let error_message = error_list.iter().map(|x| format!("<li>{:?}</li>", x.iter().collect::<PathBuf>())).collect::<String>();
                                                if !error_list.is_empty() { show_dialog(app_ui.window, false, format!("<p>The following files failed to be imported:</p> <ul>{}</ul>", error_message)); }

                                                // Set it as modified. Exception for the Paint System.
                                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                                // Take out of the path list the ones that failed.
                                                paths_packedfile.retain(|x| !error_list.contains(x));

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
                            }

                            // If there is no "MyMod" path configured, report it.
                            else { return show_dialog(app_ui.window, false, "Error while adding files:\n MyMod Path not configured."); }
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
                                sender_qt.send("add_packedfile").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&paths).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&paths_packedfile).map_err(From::from)).unwrap();

                                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                                let mut event_loop = EventLoop::new();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Until we receive a response from the worker thread...
                                loop {

                                    // When we finally receive a response...
                                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                                        // If we got a response...
                                        if let Ok(data) = data {

                                            // Get the list of errors.
                                            let error_list: Vec<Vec<String>> = serde_json::from_slice(&data).unwrap();

                                            // If there is any error, report it. Otherwise, it's a success.
                                            let error_message = error_list.iter().map(|x| format!("<li>{:?}</li>", x.iter().collect::<PathBuf>())).collect::<String>();
                                            if !error_list.is_empty() { show_dialog(app_ui.window, false, format!("<p>The following files failed to be imported:</p> <ul>{}</ul>", error_message)); }

                                            // Set it as modified. Exception for the Paint System.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                            // Take out of the path list the ones that failed.
                                            paths_packedfile.retain(|x| !error_list.contains(x));

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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

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
                            sender_qt.send("get_settings").unwrap();
                            let settings = receiver_qt.borrow().recv().unwrap().unwrap();
                            let settings: Settings = serde_json::from_slice(&settings).unwrap();

                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.paths.my_mods_base_path {

                                // We get the assets folder of our mod (without .pack extension).
                                let mut assets_folder = my_mods_base_path.to_path_buf();
                                assets_folder.push(&game_folder_name);
                                assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists, and create it if it doesn't.
                                if !assets_folder.is_dir() {
                                    if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                        return show_dialog(app_ui.window, false, "Error while adding files:\n The MyMod's asset folder does not exists and it cannot be created.");
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
                                    sender_qt.send("add_packedfile").unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&paths).map_err(From::from)).unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&paths_packedfile).map_err(From::from)).unwrap();

                                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                                    let mut event_loop = EventLoop::new();

                                    // Disable the Main Window (so we can't do other stuff).
                                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                    // Until we receive a response from the worker thread...
                                    loop {

                                        // When we finally receive a response...
                                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                                            // If we got a response...
                                            if let Ok(data) = data {

                                                // Get the list of errors.
                                                let error_list: Vec<Vec<String>> = serde_json::from_slice(&data).unwrap();

                                                // If there is any error, report it. Otherwise, it's a success.
                                                let error_message = error_list.iter().map(|x| format!("<li>{:?}</li>", x.iter().collect::<PathBuf>())).collect::<String>();
                                                if !error_list.is_empty() { show_dialog(app_ui.window, false, format!("<p>The following files failed to be imported:</p> <ul>{}</ul>", error_message)); }

                                                // Set it as modified. Exception for the Paint System.
                                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                                // Take out of the path list the ones that failed.
                                                paths_packedfile.retain(|x| !error_list.contains(x));

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
                            }

                            // If there is no "MyMod" path configured, report it.
                            else { return show_dialog(app_ui.window, false, "Error while adding files:\n MyMod Path not configured."); }
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
                                sender_qt.send("add_packedfile").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&paths).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&paths_packedfile).map_err(From::from)).unwrap();

                                // Prepare the event loop, so we don't hang the UI while the background thread is working.
                                let mut event_loop = EventLoop::new();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Until we receive a response from the worker thread...
                                loop {

                                    // When we finally receive a response...
                                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                                        // If we got a response...
                                        if let Ok(data) = data {

                                            // Get the list of errors.
                                            let error_list: Vec<Vec<String>> = serde_json::from_slice(&data).unwrap();

                                            // If there is any error, report it. Otherwise, it's a success.
                                            let error_message = error_list.iter().map(|x| format!("<li>{:?}</li>", x.iter().collect::<PathBuf>())).collect::<String>();
                                            if !error_list.is_empty() { show_dialog(app_ui.window, false, format!("<p>The following files failed to be imported:</p> <ul>{}</ul>", error_message)); }

                                            // Set it as modified. Exception for the Paint System.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                            // Take out of the path list the ones that failed.
                                            paths_packedfile.retain(|x| !error_list.contains(x));

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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

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
                        let mut path: PathBuf = PathBuf::new();
                        let path_qt = file_dialog.selected_files();
                        for index in 0..path_qt.size() { path.push(path_qt.at(index).to_std_string()); }

                        // Tell the Background Thread to open the secondary PackFile.
                        sender_qt.send("open_packfile_extra").unwrap();

                        // Send the path to the Background Thread.
                        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

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

                                    // If it was it, stop the loop and continue.
                                    Ok(_) => break,

                                    // Otherwise, return an error.
                                    Err(error) => return show_dialog(app_ui.window, false, format!("<p>Error while opening the secondary PackFile:</p> <p>{}</p>", error)),
                                }
                            }

                            // Keep the UI responsive.
                            event_loop.process_events(());

                            // Wait a bit to not saturate a CPU core.
                            thread::sleep(Duration::from_millis(50));
                        }

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

                        // Re-enable the Main Window.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Create Folder" Action.
        let slot_contextual_menu_create_folder = SlotBool::new(clone!(
            rpfm_path,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Create the "New Folder" dialog and wait for a new name (or a cancelation).
                    if let Some(new_folder_name) = create_new_folder_dialog(&app_ui) {

                        // Get his Path, including the name of the PackFile.
                        let mut complete_path = get_path_from_selection(&app_ui, false);

                        // Add the folder's name to the list.
                        complete_path.push(new_folder_name);

                        // Check if the folder exists.
                        sender_qt.send("folder_exists").unwrap();
                        sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let folder_exists: bool = serde_json::from_slice(&response).unwrap();

                        // If the folder already exists, return an error.
                        if folder_exists { return show_dialog(app_ui.window, false, "Error: this folder already exists in this Path.")}

                        // Add it to the PackFile.
                        sender_qt.send("create_folder").unwrap();
                        sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();

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
            }
        ));

        // What happens when we trigger the "Create DB PackedFile" Action.
        let slot_contextual_menu_create_packed_file_db = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                    if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &receiver_qt, PackedFileType::DB("".to_owned(), "".to_owned(), 0)) {

                        // Get the name of the PackedFile.
                        if let PackedFileType::DB(name, table,_) = packed_file_type.clone() {

                            // If the name is not empty...
                            if !name.is_empty() {

                                // Get his Path, without the name of the PackFile.
                                let mut complete_path = vec!["db".to_owned(), table, name];

                                // Check if the folder exists.
                                sender_qt.send("packed_file_exists").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                let response = receiver_qt.borrow().recv().unwrap().unwrap();
                                let exists: bool = serde_json::from_slice(&response).unwrap();

                                // If the folder already exists, return an error.
                                if exists { return show_dialog(app_ui.window, false, "Error: there is already a File with this name in this Path.")}

                                // Add it to the PackFile.
                                sender_qt.send("create_packed_file").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&packed_file_type).map_err(From::from)).unwrap();

                                // Get the response, just in case it failed.
                                let response = receiver_qt.borrow().recv().unwrap();
                                if let Err(error) = response { return show_dialog(app_ui.window, false, format_err!("<p>Error while creating the new PackedFile:</p><p>{}</p>", error.cause())) }

                                // Set it as modified. Exception for the Paint system.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

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

                            // Otherwise, the name is invalid.
                            else { return show_dialog(app_ui.window, false, "Error: only my heart can be empty.") }
                        }
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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                    if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &receiver_qt, PackedFileType::Loc("".to_owned())) {

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

                                // Check if the folder exists.
                                sender_qt.send("packed_file_exists").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                let response = receiver_qt.borrow().recv().unwrap().unwrap();
                                let exists: bool = serde_json::from_slice(&response).unwrap();

                                // If the folder already exists, return an error.
                                if exists { return show_dialog(app_ui.window, false, "Error: there is already a File with this name in this Path.")}

                                // Add it to the PackFile.
                                sender_qt.send("create_packed_file").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&packed_file_type).map_err(From::from)).unwrap();

                                // Get the response, just in case it failed.
                                let response = receiver_qt.borrow().recv().unwrap();
                                if let Err(error) = response { return show_dialog(app_ui.window, false, format_err!("<p>Error while creating the new PackedFile:</p><p>{}</p>", error.cause())) }

                                // Set it as modified. Exception for the Paint System.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

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

                            // Otherwise, the name is invalid.
                            else { return show_dialog(app_ui.window, false, "Error: only my heart can be empty.") }
                        }
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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Create the "New PackedFile" dialog and wait for his data (or a cancelation).
                    if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &receiver_qt, PackedFileType::Text("".to_owned())) {

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

                                // Check if the folder exists.
                                sender_qt.send("packed_file_exists").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                let response = receiver_qt.borrow().recv().unwrap().unwrap();
                                let exists: bool = serde_json::from_slice(&response).unwrap();

                                // If the folder already exists, return an error.
                                if exists { return show_dialog(app_ui.window, false, "Error: there is already a File with this name in this Path.")}

                                // Add it to the PackFile.
                                sender_qt.send("create_packed_file").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&packed_file_type).map_err(From::from)).unwrap();

                                // Get the response, just in case it failed.
                                let response = receiver_qt.borrow().recv().unwrap();
                                if let Err(error) = response { return show_dialog(app_ui.window, false, format_err!("<p>Error while creating the new PackedFile:</p><p>{}</p>", error.cause())) }

                                // Set it as modified. Exception for the Paint System.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

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

                            // Otherwise, the name is invalid.
                            else { return show_dialog(app_ui.window, false, "Error: only my heart can be empty.") }
                        }
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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Create the "Mass-Import TSV" dialog and wait for his data (or a cancelation).
                    if let Some(data) = create_mass_import_tsv_dialog(&app_ui) {

                        // If there is no name...
                        if data.0.is_empty() { return show_dialog(app_ui.window, false, "Error: You need a name in order to import the files.") }

                        // If there is no file selected...
                        else if data.1.is_empty() { return show_dialog(app_ui.window, false, "Error: You didn't selected any file to import.") }

                        // Otherwise...
                        else {

                            // Try to import them.
                            sender_qt.send("mass_import_tsv").unwrap();
                            sender_qt_data.send(serde_json::to_vec(&data).map_err(From::from)).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Prepare the event loop, so we don't hang the UI while the background thread is working.
                            let mut event_loop = EventLoop::new();

                            // Until we receive a response from the worker thread...
                            loop {

                                // When we finally receive the data...
                                if let Ok(data) = receiver_qt.borrow().try_recv() {

                                    // Check what the result of the importing process was.
                                    match data {

                                        // In case of success...
                                        Ok(response) => {

                                            // Get the list of paths to replace, and to add to the view.
                                            let mut paths: (Vec<Vec<String>>, Vec<Vec<String>>) = serde_json::from_slice(&response).unwrap();

                                            // Set it as modified. Exception for the paint system.
                                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

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
                                        }

                                        // In case of error, show the dialog with the error.
                                        Err(error) => show_dialog(app_ui.window, false, format!("<p>Error while importing TSV Files:</p><p>{}</p>", error.cause())),
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
                    show_dialog(app_ui.window, false, "You can't delete a PackedFile/Folder while there is a PackedFile opened in the right side. Pls, close it by clicking in a Folder/PackFile before trying to delete it again.")
                }

                // Otherwise, we continue the deletion process.
                else {

                    // We only do something in case the focus is in the TreeView. This should stop
                    // problems with the accels working everywhere.
                    let has_focus;
                    unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                    if has_focus {

                        // Get his Path, including the name of the PackFile.
                        let path = get_path_from_selection(&app_ui, true);

                        // Tell the Background Thread to delete the selected stuff.
                        sender_qt.send("delete_packedfile").unwrap();
                        sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();

                        // When we finally receive the data...
                        if let Ok(data) = receiver_qt.borrow().recv() {

                            // If we had success...
                            if let Ok(response) = data {

                                // Set the mod as "Modified". For now, we don't paint deletions.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                // Get the type of the selection.
                                let path_type: TreePathType = serde_json::from_slice(&response).unwrap();

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
                            }
                        }
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

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Prepare the event loop, so we don't hang the UI while the background thread is working.
                    let mut event_loop = EventLoop::new();

                    // Get his Path, including the name of the PackFile.
                    let path = get_path_from_selection(&app_ui, true);

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send("get_type_of_path").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

                    // Get the settings.
                    sender_qt.send("get_settings").unwrap();
                    let settings = receiver_qt.borrow().recv().unwrap().unwrap();
                    let settings: Settings = serde_json::from_slice(&settings).unwrap();

                    // Depending on the current Operational Mode...
                    match *mode.borrow() {

                        // If we have a "MyMod" selected...
                        Mode::MyMod {ref game_folder_name, ref mod_name} => {

                            // In theory, if we reach this line this should always exist. In theory I should be rich.
                            if let Some(ref my_mods_base_path) = settings.paths.my_mods_base_path {

                                // We get the assets folder of our mod (without .pack extension).
                                let mut assets_folder = my_mods_base_path.to_path_buf();
                                assets_folder.push(&game_folder_name);
                                assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists, and create it if it doesn't.
                                if !assets_folder.is_dir() {
                                    if let Err(_) = DirBuilder::new().recursive(true).create(&assets_folder) {
                                        return show_dialog(app_ui.window, false, "Error while extracting files:\n The MyMod's asset folder does not exists and it cannot be created.");
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
                                                return show_dialog(app_ui.window, false, "Error while extracting files:\n The extracted folder couldn't be created.");
                                            }
                                        }
                                    }
                                }

                                // Tell the Background Thread to delete the selected stuff.
                                sender_qt.send("extract_packedfile").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                                sender_qt_data.send(serde_json::to_vec(&assets_folder).map_err(From::from)).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Until we receive a response from the worker thread...
                                loop {

                                    // When we finally receive the data...
                                    if let Ok(data) = receiver_qt.borrow().try_recv() {

                                        // Check what the result of the deletion process was.
                                        match data {

                                            // In case of success...
                                            Ok(response) => {

                                                // Get the result, and show it.
                                                let result: String = serde_json::from_slice(&response).unwrap();
                                                show_dialog(app_ui.window, true, result);
                                            },

                                            // In case of error, show the dialog with the error.
                                            Err(error) => show_dialog(app_ui.window, false, error.cause()),
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

                            // If there is no "MyMod" path configured, report it.
                            else { return show_dialog(app_ui.window, false, "Error while extracting files:\n MyMod Path not configured."); }
                        }

                        // If we are in "Normal" Mode....
                        Mode::Normal => {

                            // If we want the old PFM behavior (extract full path)...
                            if settings.use_pfm_extracting_behavior {

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
                                    sender_qt.send("extract_packedfile").unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&final_extraction_path).map_err(From::from)).unwrap();

                                    // Disable the Main Window (so we can't do other stuff).
                                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                    // Until we receive a response from the worker thread...
                                    loop {

                                        // When we finally receive the data...
                                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                                            // Check what the result of the deletion process was.
                                            match data {

                                                // In case of success...
                                                Ok(response) => {

                                                    // Get the result, and show it.
                                                    let result: String = serde_json::from_slice(&response).unwrap();
                                                    show_dialog(app_ui.window, true, result);
                                                },

                                                // In case of error, show the dialog with the error.
                                                Err(error) => show_dialog(app_ui.window, false, error.cause()),
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
                                    sender_qt.send("extract_packedfile").unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                                    sender_qt_data.send(serde_json::to_vec(&extraction_path).map_err(From::from)).unwrap();

                                    // Disable the Main Window (so we can't do other stuff).
                                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                    // Until we receive a response from the worker thread...
                                    loop {

                                        // When we finally receive the data...
                                        if let Ok(data) = receiver_qt.borrow().try_recv() {

                                            // Check what the result of the deletion process was.
                                            match data {

                                                // In case of success...
                                                Ok(response) => {

                                                    // Get the result, and show it.
                                                    let result: String = serde_json::from_slice(&response).unwrap();
                                                    show_dialog(app_ui.window, true, result);
                                                },

                                                // In case of error, show the dialog with the error.
                                                Err(error) => show_dialog(app_ui.window, false, error.cause()),
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
                            }
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Open in decoder" action in the Contextual Menu.
        let slot_contextual_menu_open_decoder = SlotBool::new(clone!(
            supported_games,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_packedfile_opened,
            rpfm_path => move |_| {

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Get his Path, including the name of the PackFile.
                    let path = get_path_from_selection(&app_ui, true);

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send("get_type_of_path").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

                    // If it's a PackedFile...
                    if let TreePathType::File((_, index)) = item_type {

                        // Remove everything from the PackedFile View.
                        purge_them_all(&app_ui, &is_packedfile_opened);

                        // We try to open it in the decoder.
                        if let Ok(result) = PackedFileDBDecoder::create_decoder_view(
                            &rpfm_path,
                            supported_games.to_vec(),
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
        unsafe { app_ui.context_menu_delete.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_delete); }
        unsafe { app_ui.context_menu_extract.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_extract); }
        unsafe { app_ui.context_menu_open_decoder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_open_decoder); }

        //-----------------------------------------------------------------------------------------//
        // Rename Action. Due to me not understanding how the edition of a TreeView works, we do it
        // in a special way. So TODO: Fix this shit.
        //-----------------------------------------------------------------------------------------//

        // What happens when we trigger the "Rename" Action.
        let slot_contextual_menu_rename = SlotBool::new(clone!(
            rpfm_path,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // We only do something in case the focus is in the TreeView. This should stop
                // problems with the accels working everywhere.
                let has_focus;
                unsafe { has_focus = app_ui.folder_tree_view.as_mut().unwrap().has_focus() };
                if has_focus {

                    // Get his Path, including the name of the PackFile.
                    let complete_path = get_path_from_selection(&app_ui, true);

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send("get_type_of_path").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&complete_path).map_err(From::from)).unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

                    // Depending on the type of the selection...
                    match item_type.clone() {

                        // If it's a file or a folder...
                        TreePathType::File((path,_)) | TreePathType::Folder(path) => {

                            // Get the name of the selected item.
                            let current_name = path.last().unwrap();

                            // Create the "Rename" dialog and wait for a new name (or a cancelation).
                            if let Some(new_name) = create_rename_dialog(&app_ui, &current_name) {

                                // Send the New Name to the Background Thread, wait for a response.
                                sender_qt.send("rename_packed_file").unwrap();
                                sender_qt_data.send(serde_json::to_vec(&(complete_path, &new_name)).map_err(From::from)).unwrap();
                                let response = receiver_qt.borrow().recv().unwrap();

                                // Depending on the result, we act...
                                match response {

                                    // If the new name was valid...
                                    Ok(_) => {

                                        // Set the mod as "Modified". This is an exception to the paint system.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

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
                                    }

                                    // If the new name was invalid...
                                    Err(error) => show_dialog(app_ui.window, false, error.cause()),
                                }
                            }
                        }

                        // Otherwise, it's the PackFile or None, and we return, as we can't rename that.
                        _ => return,
                    }

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
                    sender_qt.send("get_type_of_path").unwrap();
                    sender_qt_data.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                    let item_type: TreePathType = serde_json::from_slice(&response).unwrap();

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
                                        Err(error) => return show_dialog(app_ui.window, false, format!("<p>Error while opening a Loc PackedFile:</p> <p>{}</p>", error.cause())),
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
                                        Err(error) => return show_dialog(app_ui.window, false, format!("<p>Error while opening a DB PackedFile:</p> <p>{}</p>", error.cause())),
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
                                        Err(error) => return show_dialog(app_ui.window, false, format!("<p>Error while opening a Text PackedFile:</p> <p>{}</p>", error.cause())),
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
                                        Err(error) => return show_dialog(app_ui.window, false, format!("<p>Error while opening a RigidModel PackedFile:</p> <p>{}</p>", error.cause())),
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
                                    ) { return show_dialog(app_ui.window, false, error.cause()); }
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
                if *mymod_menu_needs_rebuild.borrow() {

                    // Then recreate the "MyMod" submenu.
                    let result = build_my_mod_menu(
                        rpfm_path.to_path_buf(),
                        sender_qt.clone(),
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.clone(),
                        &menu_bar_mymod,
                        is_modified.clone(),
                        mode.clone(),
                        supported_games.to_vec(),
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

        // Get the settings.
        sender_qt.send("get_settings").unwrap();
        let settings = receiver_qt.borrow().recv().unwrap().unwrap();
        let settings: Settings = serde_json::from_slice(&settings).unwrap();

        // If we have it enabled in the prefs, check if there are updates.
        if settings.check_updates_on_start { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if settings.check_schema_updates_on_start { check_schema_updates(&app_ui, false, &rpfm_path, &sender_qt, &sender_qt_data, &receiver_qt) };

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
                ) { show_dialog(app_ui.window, false, format!("<p>Error while opening the PackFile:</p><p>{}</p>", error.cause())) }
            }
        }

        // And launch it.
        Application::exec()
    })
}

/// This is the background loop that's going to be executed in a parallel thread to the UI. No UI or "Unsafe" stuff here.
/// The sender is to send stuff back (Result with something encoded with serde_json or error) to the UI.
/// The receiver is to receive orders to execute from the loop.
/// The receiver_data is to receive data (whatever data is needed) encoded with serde_json from the UI Thread.
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

    // TODO: This crashes if LICENSE get's deleted. Fix this shit.
    // The extra PackFile needs to keep a BufReader to not destroy the Ram.
    let mut pack_file_decoded_extra_buffer = BufReader::new(File::open(rpfm_path.join(PathBuf::from("LICENSE"))).unwrap());

    // These are a list of empty PackedFiles, used to store data of the open PackedFile.
    let mut packed_file_loc = Loc::new();
    let mut packed_file_db = DB::new("", 0, TableDefinition::new(0));
    let mut packed_file_rigid_model = RigidModel::new();

    // We load the list of Supported Games here.
    // TODO: Move this to a const when const fn reach stable in Rust.
    let supported_games = GameInfo::new();

    // We load the settings here, and in case they doesn't exist or they are not valid, we create them.
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

        // Wait until you get something through the channel. This hangs the thread until we got something,
        // so it doesn't use processing power until we send it a message.
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

                    // In case we want to reset the Secondary PackFile to his original state (dummy)...
                    "reset_packfile_extra" => {

                        // Create the new PackFile.
                        pack_file_decoded_extra = PackFile::new();
                    }

                    // In case we want to create a "New PackFile"...
                    "new_packfile" => {

                        // Get the ID for the new PackFile.
                        let pack_file_id = supported_games.iter().filter(|x| x.folder_name == game_selected.game).map(|x| x.id.to_owned()).collect::<String>();

                        // Create the new PackFile.
                        pack_file_decoded = packfile::new_packfile("unknown.pack".to_string(), &pack_file_id);

                        // Try to load the Schema for this PackFile's game.
                        schema = Schema::load(&rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                        // Get the PackFile's Type we must return to the UI thread and serialize it.
                        let data = serde_json::to_vec(&pack_file_decoded.header.pack_file_type).map_err(From::from);

                        // Send a response to the UI thread.
                        sender.send(data).unwrap();
                    }

                    // In case we want to "Open a PackFile"...
                    "open_packfile" => {

                        // Get the path to the PackFile.
                        let path = receiver_data.recv().unwrap().unwrap();

                        // Try to deserialize it as a path.
                        let path = serde_json::from_slice(&path).unwrap();

                        // Open the PackFile (Or die trying it).
                        match packfile::open_packfile(path) {

                            // If we succeed at opening the PackFile...
                            Ok(pack_file) => {

                                // Get the decoded PackFile.
                                pack_file_decoded = pack_file;

                                // Try to load the Schema for this PackFile's game.
                                schema = Schema::load(&rpfm_path, &supported_games.iter().filter(|x| x.folder_name == *game_selected.game).map(|x| x.schema.to_owned()).collect::<String>()).ok();

                                // Get the PackFile's Header we must return to the UI thread and serialize it.
                                let data = serde_json::to_vec(&pack_file_decoded.header).map_err(From::from);

                                // Send a response to the UI thread.
                                sender.send(data).unwrap();

                                // Test to see if every DB Table can be decoded. This is slow and only useful when
                                // a new patch lands and you want to know what tables you need to decode. So, unless that,
                                // leave this code commented.
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

                    // In case we want to "Open an Extra PackFile" (for "Add from PackFile")...
                    "open_packfile_extra" => {

                        // Get the path to the PackFile.
                        let path = receiver_data.recv().unwrap().unwrap();

                        // Try to deserialize it as a path.
                        let path = serde_json::from_slice(&path).unwrap();

                        // Open the PackFile as Read-Only (Or die trying it).
                        match packfile::open_packfile_with_bufreader(path) {

                            // If we managed to open it...
                            Ok(result) => {

                                // Get the PackFile and the Buffer.
                                pack_file_decoded_extra = result.0;
                                pack_file_decoded_extra_buffer = result.1;

                                // Encode a success to send it to the UI thread.
                                let data = serde_json::to_vec(&()).map_err(From::from);

                                // Send a response to the UI thread.
                                sender.send(data).unwrap();
                            }

                            // If there is an error, send it back to the UI.
                            Err(error) => sender.send(Err(error)).unwrap(),
                        }
                    }

                    // In case we want to "Save a PackFile"...
                    "save_packfile" => {

                        // If it's of a type we can edit...
                        if pack_file_decoded.is_editable(&settings) {

                            // Check if it already exist in the disk.
                            if pack_file_decoded.extra_data.file_path.is_file() {

                                // If it passed all the checks, then try to save it and return the result.
                                match packfile::save_packfile(&mut pack_file_decoded, None) {
                                    Ok(_) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                    Err(error) => sender.send(Err(format_err!("<p>Error while trying to save the PackFile:</p><p>{}</p>", error.cause()))).unwrap(),
                                }
                            }

                            // Otherwise, we default to the "Save PackFile As" action sending an empty error as response.
                            else { sender.send(Err(format_err!(""))).unwrap(); }
                        }

                        // Otherwise, return an error.
                        else {
                            sender.send(Err(format_err!("
                                <p>This type of PackFile is supported in Read-Only mode.</p>
                                <p>This can happen due to:</p>
                                <ul>
                                    <li>The PackFile's type is <i>'Boot'</i>, <i>'Release'</i>, <i>'Patch'</i> or <i>'Music'</i> and you have <i>'Allow edition of CA PackFiles'</i> disabled in the settings.</li>
                                    <li>The PackFile's type is <i>'Other'</i>.</li>
                                    <li>One of the checkboxes under <i>'PackFile/Change PackFile Type'</i> is checked.</li>
                                </ul>
                                <p>If you really want to save it, go to <i>'PackFile/Change PackFile Type'</i> and change his type to 'Mod' or 'Movie'. Note that if the cause it's the third on the list, there is no way to save the PackFile, yet.</p>"
                            ))).unwrap();
                        }
                    }

                    // In case we want to "Save a PackFile As"...
                    "save_packfile_as" => {

                        // If it's of a type we can edit...
                        if pack_file_decoded.is_editable(&settings) {

                            // If it's editable, we send the UI the "Extra data" of the PackFile, as the UI needs it for some stuff.
                            sender.send(serde_json::to_vec(&pack_file_decoded.extra_data).map_err(From::from)).unwrap();

                            // Wait until you get a path from the UI.
                            if let Ok(path) = receiver_data.recv().unwrap() {

                                // Try to deserialize it.
                                match serde_json::from_slice(&path) {

                                    // If it can be deserialized as a PathBuf...
                                    Ok(path) => {

                                        // Redundant, but needed for the deserializer to know the type.
                                        let path: PathBuf = path;

                                        // Try to save the PackFile and return the results.
                                        match packfile::save_packfile(&mut pack_file_decoded, Some(path.to_path_buf())) {
                                            Ok(_) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                            Err(error) => sender.send(Err(format_err!("<p>Error while trying to save the PackFile:</p><p>{}</p>", error.cause()))).unwrap(),
                                        }
                                    },

                                    // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                    Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                                }
                            }
                        }

                        // Otherwise, return an error.
                        else {
                            sender.send(Err(format_err!("
                                <p>This type of PackFile is supported in Read-Only mode.</p>
                                <p>This can happen due to:</p>
                                <ul>
                                    <li>The PackFile's type is <i>'Boot'</i>, <i>'Release'</i>, <i>'Patch'</i> or <i>'Music'</i> and you have <i>'Allow edition of CA PackFiles'</i> disabled in the settings.</li>
                                    <li>The PackFile's type is <i>'Other'</i>.</li>
                                    <li>One of the checkboxes under <i>'PackFile/Change PackFile Type'</i> is checked.</li>
                                </ul>
                                <p>If you really want to save it, go to <i>'PackFile/Change PackFile Type'</i> and change his type to 'Mod' or 'Movie'. Note that if the cause it's the third on the list, there is no way to save the PackFile, yet.</p>"
                            ))).unwrap();
                        }
                    }

                    // In case we want to change the PackFile's Type...
                    "set_packfile_type" => {

                        // Wait until you get something from the UI.
                        if let Ok(new_type) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&new_type) {

                                // If it can be deserialized as a U32...
                                Ok(new_type) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let new_type: u32 = new_type;

                                    // Change the type of the PackFile.
                                    pack_file_decoded.header.pack_file_type = new_type;
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to get the currently loaded Schema...
                    "get_schema" => {

                        // Send the schema back to the UI thread.
                        sender.send(serde_json::to_vec(&schema).map_err(From::from)).unwrap();
                    }

                    // In case we want to get the current settings...
                    "get_settings" => {

                        // Send the current settings back to the UI thread.
                        sender.send(serde_json::to_vec(&settings).map_err(From::from)).unwrap();
                    }

                    // In case we want to change the current settings...
                    "set_settings" => {

                        // Wait until you get something from the UI.
                        if let Ok(new_settings) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&new_settings) {

                                // If it can be deserialized as a Settings struct...
                                Ok(new_settings) => {

                                    // Update our current settings with the ones we received from the UI.
                                    settings = new_settings;

                                    // Save our Settings to a settings file, and report in case of error.
                                    match settings.save(&rpfm_path) {
                                        Ok(()) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want get our current Game Selected...
                    "get_game_selected" => {

                        // Send the current Game Selected back to the UI thread.
                        sender.send(serde_json::to_vec(&game_selected).map_err(From::from)).unwrap();
                    }

                    // In case we want to change the current Game Selected...
                    "set_game_selected" => {

                        // Wait until you get something from the UI.
                        if let Ok(game_name) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&game_name) {

                                // If it can be deserialized as a &str...
                                Ok(game_name) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let game_name: &str = game_name;

                                    // Get the new Game Selected, and set it.
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
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to get the current PackFile's Header...
                    "get_packfile_header" => {

                        // Send the header of the currently open PackFile.
                        sender.send(serde_json::to_vec(&pack_file_decoded.header).map_err(From::from)).unwrap();
                    }

                    // In case we want to get the path of a PackedFile...
                    "get_packed_file_path" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // Get a reference to the path of the PackedFile.
                                    let path = &pack_file_decoded.data.packed_files[index].path;

                                    // Serialize and send the path of the PackedFile.
                                    sender.send(serde_json::to_vec(&path).map_err(From::from)).unwrap();
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to change the current Dependency Database...
                    "set_dependency_database" => {

                        // Change the `dependency_database` for that game.
                        dependency_database = match packfile::open_packfile(game_selected.game_dependency_packfile_path.to_path_buf()) {
                            Ok(data) => Some(data.data.packed_files),
                            Err(_) => None,
                        };
                    }

                    // In case we want to check if there is a current Dependency Database loaded...
                    "is_there_a_dependency_database" => {
                        match dependency_database {
                            Some(_) => sender.send(serde_json::to_vec(&true).map_err(From::from)).unwrap(),
                            None => sender.send(serde_json::to_vec(&false).map_err(From::from)).unwrap(),
                        }
                    }

                    // In case we want to check if there is an Schema loaded...
                    "is_there_a_schema" => {
                        match schema {
                            Some(_) => sender.send(serde_json::to_vec(&true).map_err(From::from)).unwrap(),
                            None => sender.send(serde_json::to_vec(&false).map_err(From::from)).unwrap(),
                        }
                    }

                    // TODO: This is not revised, because it's going to be deleted.
                    // In case we want to create a new Dependency Database...
                    "create_dependency_database" => {

                        // Get the data folder of game_selected.
                        match game_selected.game_data_path {

                            // If we got it...
                            Some(ref path) => {

                                // Get the path of the data.pack PackFile.
                                let mut data_pack_path = path.to_path_buf();
                                data_pack_path.push("data.pack");

                                // Try to open it...
                                match packfile::open_packfile_with_bufreader(data_pack_path) {

                                    // If we could open it...
                                    Ok(data_packfile) => {

                                        // Create the DB PackFile.
                                        let mut pack_file_db = PackFile::new();

                                        // Get the PackFile and the BufReader.
                                        let pack_file = data_packfile.0;
                                        let mut reader = data_packfile.1;

                                        // List of PackedFiles to add.
                                        let mut packed_files = vec![];

                                        // For each PackFile in the data.pack...
                                        for (index, packed_file) in pack_file.data.packed_files.iter().enumerate() {

                                            // If it's a DB file...
                                            if packed_file.path.starts_with(&["db".to_owned()]) {

                                                // Clone the PackedFile.
                                                let mut packed_file = packed_file.clone();

                                                // Read it.
                                                packed_file.data = vec![0; packed_file.size as usize];
                                                reader.seek(SeekFrom::Start(pack_file.packed_file_indexes[index])).unwrap();
                                                reader.read_exact(&mut packed_file.data).unwrap();

                                                // Add it to the PackedFiles List.
                                                packed_files.push(packed_file);
                                            }
                                        }

                                        // Add them to the new PackFile.
                                        pack_file_db.add_packedfiles(packed_files);

                                        // Get the path of the Dependency PackFiles.
                                        let mut dep_packs_path = game_selected.game_dependency_packfile_path.to_path_buf();
                                        dep_packs_path.pop();

                                        // Create it if it doesn't exist yet (or report error if you can't).
                                        if let Err(error) = DirBuilder::new().recursive(true).create(&dep_packs_path) {
                                            return sender.send(Err(format_err!("Error while trying to create the dependency folder:\n{}", Error::from(error).cause()))).unwrap();
                                        }

                                        // Try to save the new PackFile, and report the result.
                                        match packfile::save_packfile(&mut pack_file_db, Some(game_selected.game_dependency_packfile_path.to_path_buf())) {
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

                    // In case we want to Patch the SiegeAI of a PackFile...
                    "patch_siege_ai" => {

                        // First, we try to patch the PackFile.
                        match packfile::patch_siege_ai(&mut pack_file_decoded) {

                            // If we succeed....
                            Ok(result) => {

                                // Then we try to save the Patched PackFile.
                                match packfile::save_packfile(&mut pack_file_decoded, None) {

                                    // If we succeed, send back the result.
                                    Ok(_) => sender.send(serde_json::to_vec(&result).map_err(From::from)).unwrap(),

                                    // If there is an error, report it.
                                    Err(error) => sender.send(Err(format_err!("<p>The PackFile has been patched, but there was an error while trying to save it:</p><p>{}</p>", error.cause()))).unwrap()
                                }
                            }

                            // Otherwise, return an error.
                            Err(error) => sender.send(Err(format_err!("<p>Error while trying to patch the PackFile:</p><p>{}</p>", error.cause()))).unwrap()
                        }
                    }

                    // In case we want to update our Schemas...
                    "update_schemas" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple of Versions...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (Versions, Versions) = data;

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
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to add PackedFiles into a PackFile...
                    "add_packedfile" => {

                        // Wait until you get something from the UI.
                        if let Ok(paths) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&paths) {

                                // If it can be deserialized as a Vector of PathBufs...
                                Ok(paths) => {

                                    // Wait until you get something from the UI.
                                    if let Ok(paths_packedfile) = receiver_data.recv().unwrap() {

                                        // Try to deserialize it.
                                        match serde_json::from_slice(&paths_packedfile) {

                                            // If it can be deserialized as a Vector of Vectors of Strings...
                                            Ok(paths_packedfile) => {

                                                // Redundant, but needed for the deserializer to know the types.
                                                let paths: Vec<PathBuf> = paths;
                                                let paths_packedfile: Vec<Vec<String>> = paths_packedfile;

                                                // Get a list of the PackedFiles that failed to be added, for one reason or another.
                                                let mut errors = vec![];

                                                // For each file...
                                                for index in 0..paths.len() {

                                                    // Try to add it to the PackFile. If it fails, add it to the error list.
                                                    if let Err(_) = packfile::add_file_to_packfile(&mut pack_file_decoded, &paths[index], paths_packedfile[index].to_vec()) {
                                                        errors.push(paths_packedfile[index].to_vec());
                                                    }
                                                }

                                                // Send back the list of files that failed.
                                                sender.send(serde_json::to_vec(&errors).map_err(From::from)).unwrap();
                                            },

                                            // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                            Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                                        }
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to delete PackedFiles from a PackFile...
                    "delete_packedfile" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: Vec<String> = path;

                                    // Get the type of the Path we want to delete.
                                    let path_type = get_type_of_selected_path(&path, &pack_file_decoded);

                                    // Try to delete the PackedFiles from the PackFile, changing his return in case of success.
                                    match packfile::delete_from_packfile(&mut pack_file_decoded, &path) {

                                        // In case of success...
                                        Ok(_) => sender.send(serde_json::to_vec(&path_type).map_err(From::from)).unwrap(),

                                        // In case of error, send the error back.
                                        Err(error) => sender.send(Err(error)).unwrap()
                                    };
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to extract PackedFiles from a PackFile...
                    "extract_packedfile" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Wait until you get something from the UI.
                                    if let Ok(extraction_path) = receiver_data.recv().unwrap() {

                                        // Try to deserialize it.
                                        match serde_json::from_slice(&extraction_path) {

                                            // If it can be deserialized as a PathBuf...
                                            Ok(extraction_path) => {

                                                // Redundant, but needed for the deserializer to know the type.
                                                let path: Vec<String> = path;
                                                let extraction_path: PathBuf = extraction_path;

                                                // Try to extract the PackFile.
                                                match packfile::extract_from_packfile(
                                                    &pack_file_decoded,
                                                    &path,
                                                    &extraction_path
                                                ) {
                                                    Ok(result) => sender.send(serde_json::to_vec(&result).map_err(From::from)).unwrap(),
                                                    Err(error) => sender.send(Err(error)).unwrap(),
                                                }
                                            },

                                            // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                            Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                                        }
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to get the type of an item in the TreeView, from his path...
                    "get_type_of_path" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: Vec<String> = path;

                                    // Get the type of the selected item.
                                    let path_type = get_type_of_selected_path(&path, &pack_file_decoded);

                                    // Send the type back.
                                    sender.send(serde_json::to_vec(&path_type).map_err(From::from)).unwrap();
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to know if a PackedFile exists, knowing his path...
                    "packed_file_exists" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: Vec<String> = path;

                                    // Check if the path exists as a PackedFile.
                                    let exists = pack_file_decoded.data.packedfile_exists(&path);

                                    // Send the result back.
                                    sender.send(serde_json::to_vec(&exists).map_err(From::from)).unwrap();
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to know if a Folder exists, knowing his path...
                    "folder_exists" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: Vec<String> = path;

                                    // Check if the path exists as a folder.
                                    let exists = pack_file_decoded.data.folder_exists(&path);

                                    // Send the result back.
                                    sender.send(serde_json::to_vec(&exists).map_err(From::from)).unwrap();
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to create a PackedFile from scratch...
                    "create_packed_file" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Wait until you get something from the UI.
                                    if let Ok(packed_file_type) = receiver_data.recv().unwrap() {

                                        // Try to deserialize it.
                                        match serde_json::from_slice(&packed_file_type) {

                                            // If it can be deserialized as a PackedFileType...
                                            Ok(packed_file_type) => {

                                                // Redundant, but needed for the deserializer to know the types.
                                                let path: Vec<String> = path;
                                                let packed_file_type: PackedFileType = packed_file_type;

                                                // Create the PackedFile.
                                                match create_packed_file(
                                                    &mut pack_file_decoded,
                                                    packed_file_type,
                                                    path,
                                                    &schema,
                                                ) {
                                                    // Send the result back.
                                                    Ok(_) => sender.send(serde_json::to_vec(&()).map_err(From::from)).unwrap(),
                                                    Err(error) => sender.send(Err(error)).unwrap(),
                                                }
                                            },

                                            // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                            Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                                        }
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // TODO: Move checkings here, from the UI.
                    // In case we want to create an empty folder...
                    "create_folder" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a Vector of Strings...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: Vec<String> = path;

                                    // Check if the path exists as a folder.
                                    pack_file_decoded.data.empty_folders.push(path);
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to update the empty folder list...
                    "update_empty_folders" => {

                        // Update the empty folder list, if needed.
                        pack_file_decoded.data.update_empty_folders();
                    }

                    // In case we want to get the data of a PackFile needed to form the TreeView...
                    "get_packfile_data_for_treeview" => {

                        // Get the name and the PackedFile list, and serialize it.
                        let data = serde_json::to_vec(&(
                            &pack_file_decoded.extra_data.file_name,
                            pack_file_decoded.data.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                        )).map_err(From::from);

                        // Send the data to the UI thread.
                        sender.send(data).unwrap();
                    }

                    // In case we want to get the data of a Secondary PackFile needed to form the TreeView...
                    "get_packfile_extra_data_for_treeview" => {

                        // Get the name and the PackedFile list, and serialize it.
                        let data = serde_json::to_vec(&(
                            &pack_file_decoded_extra.extra_data.file_name,
                            pack_file_decoded_extra.data.packed_files.iter().map(|x| x.path.to_vec()).collect::<Vec<Vec<String>>>(),
                        )).map_err(From::from);

                        // Send the data to the UI thread.
                        sender.send(data).unwrap();
                    }

                    // In case we want to move stuff from one PackFile to another...
                    "add_packedfile_from_packfile" => {

                        // Wait until you get something from the UI.
                        if let Ok(paths) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&paths) {

                                // If it can be deserialized as a tuple of Vectors of Strings...
                                Ok(paths) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let paths: (Vec<String>, Vec<String>) = paths;

                                    // Try to add the PackedFile to the main PackFile.
                                    match packfile::add_packedfile_to_packfile(
                                        &mut pack_file_decoded_extra_buffer,
                                        &pack_file_decoded_extra,
                                        &mut pack_file_decoded,
                                        &paths.0,
                                        &paths.1,
                                    ) {

                                        // In case of success, get the list of copied PackedFiles and send it back.
                                        Ok(_) => {

                                            // Get the new "Prefix" for the PackedFiles.
                                            let mut source_prefix = paths.0;

                                            // Remove the PackFile's name from it.
                                            source_prefix.reverse();
                                            source_prefix.pop();
                                            source_prefix.reverse();

                                            // Get the new "Prefix" for the Destination PackedFiles.
                                            let mut destination_prefix = paths.1;

                                            // Remove the PackFile's name from it.
                                            destination_prefix.reverse();
                                            destination_prefix.pop();
                                            destination_prefix.reverse();

                                            // Get all the PackedFiles to copy.
                                            let path_list: Vec<Vec<String>> = pack_file_decoded_extra
                                                .data.packed_files
                                                .iter()
                                                .filter(|x| x.path.starts_with(&source_prefix))
                                                .map(|x| x.path.to_vec())
                                                .collect();

                                            // Send all of it back.
                                            sender.send(serde_json::to_vec(&(source_prefix, destination_prefix, path_list)).map_err(From::from)).unwrap();
                                        }

                                        // In case of error, report it.
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to Mass-Import TSV Files...
                    "mass_import_tsv" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple with a String and a Vector of PathBufs...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (String, Vec<PathBuf>) = data;

                                    // Try to import the files.
                                    match packedfile::tsv_mass_import(&data.1, &data.0, &schema, &mut pack_file_decoded) {
                                        Ok(result) => sender.send(serde_json::to_vec(&result).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to decode a Loc PackedFile...
                    "decode_packed_file_loc" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // We try to decode it as a Loc PackedFile.
                                    match Loc::read(&pack_file_decoded.data.packed_files[index].data) {

                                        // If we succeed, store it and send it back.
                                        Ok(packed_file_decoded) => {
                                            packed_file_loc = packed_file_decoded;
                                            sender.send(serde_json::to_vec(&packed_file_loc.data).map_err(From::from)).unwrap();
                                        }

                                        // In case of error, report it.
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to encode a Loc PackedFile...
                    "encode_packed_file_loc" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple with a LocData and an index...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (LocData, usize) = data;

                                    // Replace the old encoded data with the new one.
                                    packed_file_loc.data = data.0;

                                    // Update the PackFile to reflect the changes.
                                    packfile::update_packed_file_data_loc(
                                        &packed_file_loc,
                                        &mut pack_file_decoded,
                                        data.1
                                    );
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to import a TSV file into a Loc PackedFile...
                    "import_tsv_packed_file_loc" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a PathBuf...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: PathBuf = path;

                                    // Try to import the TSV into the open Loc PackedFile, or die trying.
                                    match packed_file_loc.data.import_tsv(&path, "Loc PackedFile") {
                                        Ok(_) => sender.send(serde_json::to_vec(&packed_file_loc.data).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to export a Loc PackedFile into a TSV file...
                    "export_tsv_packed_file_loc" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a PathBuf...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: PathBuf = path;

                                    // Try to export the TSV from the open Loc PackedFile, or die trying.
                                    match packed_file_loc.data.export_tsv(&path, ("Loc PackedFile", 9001)) {
                                        Ok(success) => sender.send(serde_json::to_vec(&success).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to decode a DB PackedFile...
                    "decode_packed_file_db" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // Depending if there is an Schema for this game or not...
                                    match schema {

                                        // If there is an Schema loaded for this game...
                                        Some(ref schema) => {

                                            // We try to decode it as a DB PackedFile.
                                            match DB::read(
                                                &pack_file_decoded.data.packed_files[index].data,
                                                &pack_file_decoded.data.packed_files[index].path[1],
                                                schema,
                                            ) {

                                                // If we succeed, store it and send it back.
                                                Ok(packed_file_decoded) => {
                                                    packed_file_db = packed_file_decoded;
                                                    sender.send(serde_json::to_vec(&packed_file_db.data).map_err(From::from)).unwrap();
                                                }

                                                // In case of error, report it.
                                                Err(error) => sender.send(Err(error)).unwrap(),
                                            }
                                        }

                                        // If there is no schema, return an error.
                                        None => sender.send(Err(format_err!("<p>Error while trying to open a DB Table:</p><p>There is no Schema loaded for this Game.</p>"))).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to encode a DB PackedFile...
                    "encode_packed_file_db" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple with a DBData and an index...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (DBData, usize) = data;

                                    // Replace the old encoded data with the new one.
                                    packed_file_db.data = data.0;

                                    // Update the PackFile to reflect the changes.
                                    packfile::update_packed_file_data_db(
                                        &packed_file_db,
                                        &mut pack_file_decoded,
                                        data.1
                                    );
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to import a TSV file into a DB PackedFile...
                    "import_tsv_packed_file_db" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a PathBuf...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: PathBuf = path;

                                    // Get his name.
                                    let name = &packed_file_db.db_type;

                                    // Try to import the TSV into the open DB PackedFile, or die trying.
                                    match packed_file_db.data.import_tsv(&path, name) {
                                        Ok(_) => sender.send(serde_json::to_vec(&packed_file_db.data).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to export a DB PackedFile into a TSV file...
                    "export_tsv_packed_file_db" => {

                        // Wait until you get something from the UI.
                        if let Ok(path) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&path) {

                                // If it can be deserialized as a PathBuf...
                                Ok(path) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let path: PathBuf = path;

                                    // Try to export the TSV into the open DB PackedFile, or die trying.
                                    match packed_file_db.data.export_tsv(&path, (&packed_file_db.db_type, packed_file_db.header.version)) {
                                        Ok(success) => sender.send(serde_json::to_vec(&success).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to decode a Plain Text PackedFile...
                    "decode_packed_file_text" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // NOTE: This only works for UTF-8 and ISO_8859_1 encoded files. Check their encoding before adding them here to be decoded.
                                    // Try to decode the PackedFile as a normal UTF-8 string.
                                    let mut decoded_string = decode_string_u8(&pack_file_decoded.data.packed_files[index].data);

                                    // If there is an error, try again as ISO_8859_1, as there are some text files using that encoding.
                                    if decoded_string.is_err() {
                                        if let Ok(string) = decode_string_u8_iso_8859_1(&pack_file_decoded.data.packed_files[index].data) {
                                            decoded_string = Ok(string);
                                        }
                                    }

                                    // Depending if the decoding worked or not, send back the text file or an error.
                                    match decoded_string {
                                        Ok(text) => sender.send(serde_json::to_vec(&text).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to encode a Text PackedFile...
                    "encode_packed_file_text" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple with a String and an index...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (String, usize) = data;

                                    // Encode the text.
                                    let encoded_text = encode_string_u8(&data.0);

                                    // Update the PackFile to reflect the changes.
                                    packfile::update_packed_file_data_text(
                                        &encoded_text,
                                        &mut pack_file_decoded,
                                        data.1
                                    );
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to decode a RigidModel...
                    "decode_packed_file_rigid_model" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // We try to decode it as a RigidModel.
                                    match RigidModel::read(&pack_file_decoded.data.packed_files[index].data) {

                                        // If we succeed, store it and send it back.
                                        Ok(packed_file_decoded) => {
                                            packed_file_rigid_model = packed_file_decoded;
                                            sender.send(serde_json::to_vec(&packed_file_rigid_model).map_err(From::from)).unwrap();
                                        }

                                        // In case of error, report it.
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to encode a RigidModel...
                    "encode_packed_file_rigid_model" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple with a RigidModel and an index...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (RigidModel, usize) = data;

                                    // Replace the old encoded data with the new one.
                                    packed_file_rigid_model = data.0;

                                    // Update the PackFile to reflect the changes.
                                    packfile::update_packed_file_data_rigid(
                                        &packed_file_rigid_model,
                                        &mut pack_file_decoded,
                                        data.1
                                    );
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to patch a decoded RigidModel from Attila to Warhammer...
                    "patch_rigid_model_attila_to_warhammer" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // We try to patch the RigidModel.
                                    match packfile::patch_rigid_model_attila_to_warhammer(&mut packed_file_rigid_model) {

                                        // If we succeed...
                                        Ok(_) => {

                                            // Update the PackFile to reflect the changes.
                                            packfile::update_packed_file_data_rigid(
                                                &packed_file_rigid_model,
                                                &mut pack_file_decoded,
                                                index
                                            );

                                            // Send back the patched PackedFile.
                                            sender.send(serde_json::to_vec(&packed_file_rigid_model).map_err(From::from)).unwrap()
                                        }

                                        // In case of error, report it.
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to decode an Image...
                    "decode_packed_file_image" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // Get the data of the image we want to open, and his name.
                                    let image_data = &pack_file_decoded.data.packed_files[index].data;
                                    let image_name = &pack_file_decoded.data.packed_files[index].path.last().unwrap().to_owned();

                                    // Create a temporal file for the image in the TEMP directory of the filesystem.
                                    let mut temporal_file_path = temp_dir();
                                    temporal_file_path.push(image_name);
                                    match File::create(&temporal_file_path) {
                                        Ok(mut temporal_file) => {

                                            // If there is an error while trying to write the image to the TEMP folder, report it.
                                            if temporal_file.write_all(image_data).is_err() {
                                                sender.send(Err(format_err!("<p>Error while trying to open the following image:\"{}\".</p>", image_name))).unwrap();
                                            }

                                            // If it worked, create an Image with the new file and show it inside a ScrolledWindow.
                                            else { sender.send(serde_json::to_vec(&temporal_file_path).map_err(From::from)).unwrap(); }
                                        }

                                        // If there is an error when trying to create the file into the TEMP folder, report it.
                                        Err(_) => sender.send(Err(format_err!("<p>Error while trying to create a file in the TEMP folder for the following image:\"{}\".</p>", image_name))).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to "Rename a PackedFile"...
                    "rename_packed_file" => {

                        // Wait until you get something from the UI.
                        if let Ok(data) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&data) {

                                // If it can be deserialized as a tuple of a Vector of Strings and a &str...
                                Ok(data) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let data: (Vec<String>, &str) = data;

                                    // Try to rename it and report the result.
                                    match packfile::rename_packed_file(&mut pack_file_decoded, &data.0, data.1) {
                                        Ok(success) => sender.send(serde_json::to_vec(&success).map_err(From::from)).unwrap(),
                                        Err(error) => sender.send(Err(error)).unwrap(),
                                    }
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case we want to get a PackedFile's data...
                    "get_packed_file" => {

                        // Wait until you get something from the UI.
                        if let Ok(index) = receiver_data.recv().unwrap() {

                            // Try to deserialize it.
                            match serde_json::from_slice(&index) {

                                // If it can be deserialized as an usize...
                                Ok(index) => {

                                    // Redundant, but needed for the deserializer to know the type.
                                    let index: usize = index;

                                    // Send back the PackedFile.
                                    sender.send(serde_json::to_vec(&pack_file_decoded.data.packed_files[index]).map_err(From::from)).unwrap();
                                },

                                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                                Err(_) => sender.send(Err(format_err!("{}", THREADS_MESSAGE_ERROR))).unwrap(),
                            }
                        }
                    }

                    // In case the message received doesn't exists, show it in the terminal.
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
    sender_qt: &Sender<&str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
    pack_file_path: PathBuf,
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    is_modified: &Rc<RefCell<bool>>,
    mode: &Rc<RefCell<Mode>>,
    game_folder: &str,
    is_packedfile_opened: &Rc<RefCell<bool>>,
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
                    let header: PackFileHeader = serde_json::from_slice(&data).unwrap();

                    // We choose the right option, depending on our PackFile.
                    match header.pack_file_type {
                        0 => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                        1 => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                        2 => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                        3 => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                        4 => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                        17 => unsafe { app_ui.change_packfile_type_music.as_mut().unwrap().set_checked(true); }
                        _ => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                    }

                    // Enable or disable these, depending on what data we have in the header.
                    unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_checked(header.index_has_extra_u32); }
                    unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(header.index_is_encrypted); }
                    unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_checked(header.mysterious_mask); }

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

                    // Stop the loop.
                    break;
                }

                // Otherwise...
                Err(error) => {

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Return an error.
                    return Err(error)
                }
            }
        }

        // Keep the UI responsive.
        event_loop.process_events(());

        // Wait a bit to not saturate a CPU core.
        thread::sleep(Duration::from_millis(10));
    }

    // Set the new mod as "Not modified".
    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

    // If it's a "MyMod" (game_folder_name is not empty), we choose the Game selected Depending on it.
    if !game_folder.is_empty() {

        // NOTE: Arena should never be here.
        // Change the Game Selected in the UI.
        match game_folder {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
        }

        // Set the current "Operational Mode" to `MyMod`.
        set_my_mod_mode(&mymod_stuff, mode, Some(pack_file_path));
    }

    // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
    else {

        // Get the PackFile's Header.
        sender_qt.send("get_packfile_header").unwrap();

        // Wait until you get something from the background thread.
        if let Ok(header) = receiver_qt.borrow().recv().unwrap() {

            // Try to deserialize it.
            match serde_json::from_slice(&header) {

                // If it can be deserialized as an PackFileHeader...
                Ok(header) => {

                    // Redundant, but needed for the deserializer to know the type.
                    let header: PackFileHeader = header;

                    // Depending on the Id, choose one game or another.
                    match &*header.id {

                        // PFH5 is for Warhammer 2/Arena.
                        "PFH5" => {

                            // If the PackFile has the mysterious byte enabled, it's from Arena.
                            if header.mysterious_mask { unsafe { app_ui.arena.as_mut().unwrap().trigger(); } }

                            // Otherwise, it's from Warhammer 2.
                            else { unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); } }
                        },

                        // PFH4 is for Warhammer 1/Attila.
                        "PFH4" | _ => {

                            // Get the Game Selected.
                            sender_qt.send("get_game_selected").unwrap();
                            let response = receiver_qt.borrow().recv().unwrap().unwrap();
                            let game_selected: GameSelected = serde_json::from_slice(&response).unwrap();

                            // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila.
                            // In any other case, we select Attila by default.
                            match &*game_selected.game {
                                "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                                "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            }
                        },
                    }
                },

                // If error, there are problems with the messages between threads. Give a warning and ask for a report.
                Err(_) => show_dialog(app_ui.window, false, THREADS_MESSAGE_ERROR),
            }
        }

        // Set the current "Operational Mode" to `Normal`.
        set_my_mod_mode(&mymod_stuff, mode, None);
    }

    // Re-enable the Main Window.
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

    // Change the Dependency Database used for our PackFile in the other Thread.
    sender_qt.send("set_dependency_database").unwrap();

    // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
    purge_them_all(&app_ui, &is_packedfile_opened);

    // Show the "Tips".
    display_help_tips(&app_ui);

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
    sender_qt: Sender<&'static str>,
    sender_qt_data: &Sender<Result<Vec<u8>, Error>>,
    receiver_qt: Rc<RefCell<Receiver<Result<Vec<u8>, Error>>>>,
    app_ui: AppUI,
    menu_bar_mymod: &*mut Menu,
    is_modified: Rc<RefCell<bool>>,
    mode: Rc<RefCell<Mode>>,
    supported_games: Vec<GameInfo>,
    needs_rebuild: Rc<RefCell<bool>>,
    is_packedfile_opened: &Rc<RefCell<bool>>
) -> (MyModStuff, MyModSlots) {

    // Get the current Settings, as we are going to need them later.
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
            needs_rebuild,
            supported_games => move |_| {

                // Create the "New MyMod" Dialog, and get the result.
                match NewMyModDialog::create_new_mymod_dialog(&app_ui, &supported_games, &settings) {

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
                            "attila" | _ => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                        }

                        // Get his new path from the base "MyMod" path + `mod_game`.
                        let mut mymod_path = settings.paths.my_mods_base_path.clone().unwrap();
                        mymod_path.push(&mod_game);

                        // Just in case the folder doesn't exist, we try to create it.
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path) {
                            return show_dialog(app_ui.window, false, format!("Error while creating the folder \"{}\" to store the MyMods.", mod_game));
                        }

                        // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
                        let mut mymod_path_private = mymod_path.to_path_buf();
                        mymod_path_private.push(&mod_name);
                        if let Err(_) = DirBuilder::new().recursive(true).create(&mymod_path_private) {
                            return show_dialog(app_ui.window, false, format!("Error while creating the folder \"{}\" to store the MyMod's files.", mod_name));
                        };

                        // Add the PackFile's name to the full path.
                        mymod_path.push(&full_mod_name);

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send("new_packfile").unwrap();

                        // We ignore the "New PackFile" returning confirmation, as we don't need it.
                        let _confirmation = receiver_qt.borrow().recv().unwrap();

                        // Tell the Background Thread to create a new PackFile.
                        sender_qt.send("save_packfile_as").unwrap();

                        // We ignore the returning confirmation, as we are going to provide the saving path ourselfs.
                        let _confirmation = receiver_qt.borrow().recv().unwrap();

                        // Pass the new PackFile's Path to the worker thread.
                        sender_qt_data.send(serde_json::to_vec(&mymod_path).map_err(From::from)).unwrap();

                        // When we finally receive the data...
                        if let Ok(data) = receiver_qt.borrow().recv() {

                            // Check what the result of the saving process was.
                            match data {

                                // In case of success...
                                Ok(_) => {

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

                                    // By default, the three bitmask should be false.
                                    unsafe { app_ui.change_packfile_type_index_has_extra_u32.as_mut().unwrap().set_checked(false); }
                                    unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                                    unsafe { app_ui.change_packfile_type_mysterious_byte.as_mut().unwrap().set_checked(false); }

                                    // Set the new "MyMod" as "Not modified".
                                    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                                    // Get the Game Selected.
                                    sender_qt.send("get_game_selected").unwrap();
                                    let response = receiver_qt.borrow().recv().unwrap().unwrap();
                                    let game_selected = serde_json::from_slice(&response).unwrap();

                                    // Enable the actions available for the PackFile from the `MenuBar`.
                                    enable_packfile_actions(&app_ui, &game_selected, true);

                                    // Set the current "Operational Mode" to `MyMod`.
                                    set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, Some(mymod_path));

                                    // Set it to rebuild next time we try to open the "MyMod" Menu.
                                    *needs_rebuild.borrow_mut() = true;
                                }

                                // In case of error, report it.
                                Err(error) => show_dialog(app_ui.window, false, format!("<p>Error while saving the PackFile:</p><p>{}</p>", error.cause())),
                            }
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
                            if let Some(ref mymods_base_path) = settings.paths.my_mods_base_path {

                                // We get his path.
                                let mut mymod_path = mymods_base_path.to_path_buf();
                                mymod_path.push(&game_folder_name);
                                mymod_path.push(&mod_name);

                                // If the mod doesn't exist, return error.
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, "Error: PackFile doesn't exist, so it can't be deleted."); }

                                // And we try to delete his PackFile. If it fails, return error.
                                if let Err(error) = remove_file(&mymod_path).map_err(Error::from) {
                                    return show_dialog(app_ui.window, false, format!("<p>Error while deleting the PackFile from disk:</p><p>{}</p>", error.cause()));
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.to_path_buf();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(&mymod_path.file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(app_ui.window, false, "Mod deleted, but his assets folder haven't been found.");
                                }

                                // If the assets folder exists, we try to delete it. Again, this is optional, so it should not stop the deleting process.
                                else if let Err(error) = remove_dir_all(&mymod_assets_path).map_err(Error::from) {
                                    show_dialog(app_ui.window, false, format!("<p>Error while deleting the Assets Folder of the MyMod from disk:</p><p>{}</p>", error.cause()));
                                }

                                // We return true, as we have delete the files of the "MyMod".
                                true
                            }

                            // If the "MyMod" path is not configured, return an error.
                            else { return show_dialog(app_ui.window, false, "MyMod base path not configured, so the MyMod couldn't be deleted."); }
                        }

                        // If we don't have a "MyMod" selected, return an error.
                        Mode::Normal => return show_dialog(app_ui.window, false, "You can't delete the selected MyMod if there is no MyMod selected."),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {

                        // Set the current "Operational Mode" to `Normal`.
                        set_my_mod_mode(&Rc::new(RefCell::new(mymod_stuff.clone())), &mode, None);

                        // Create a "dummy" PackFile, effectively closing the currently open PackFile.
                        sender_qt.send("reset_packfile").unwrap();

                        // Set the dummy mod as "Not modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                        // Get the Game Selected.
                        sender_qt.send("get_game_selected").unwrap();
                        let response = receiver_qt.borrow().recv().unwrap().unwrap();
                        let game_selected = serde_json::from_slice(&response).unwrap();

                        // Disable the actions available for the PackFile from the `MenuBar`.
                        enable_packfile_actions(&app_ui, &game_selected, false);

                        // Clear the TreeView.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().clear(); }

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
                                if !mymod_path.is_file() { return show_dialog(app_ui.window, false, "Error: PackFile doesn't exist, so it can't be deleted."); }

                                // We check that the destination path exists.
                                if !game_data_path.is_dir() {
                                    return show_dialog(app_ui.window, false, "Destination folder (..xxx/data) doesn't exist. You sure you configured the right folder for the game?");
                                }

                                // Get the destination path for the PackFile with the PackFile name included.
                                game_data_path.push(&mod_name);

                                // And copy the PackFile to his destination. If the copy fails, return an error.
                                if let Err(error) = copy(mymod_path, game_data_path).map_err(Error::from) {
                                    return show_dialog(app_ui.window, false, format!("<p>Error while copying the PackFile to the Data folder:</p><p>{}</p>", error.cause()));
                                }
                            }

                            // If we don't have a `game_data_path` configured for the current `GameSelected`...
                            else { return show_dialog(app_ui.window, false, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it."); }
                        }

                        // If the "MyMod" path is not configured, return an error.
                        else { show_dialog(app_ui.window, false, "MyMod base path not configured, so the MyMod couldn't be installed."); }
                    }

                    // If we have no "MyMod" selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, "You can't install the selected MyMod if there is no MyMod selected."),
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
                            if !game_data_path.is_file() { return show_dialog(app_ui.window, false, "The currently selected MyMod is not installed."); }

                            // If the "MyMod" is installed, we remove it. If there is a problem deleting it, return an error dialog.
                            else if let Err(error) = remove_file(game_data_path).map_err(Error::from) {
                                return show_dialog(app_ui.window, false, format!("<p>Error uninstalling the MyMod:</p><p>{}</p>", error.cause()));
                            }
                        }

                        // If we don't have a `game_data_path` configured for the current `GameSelected`...
                        else { show_dialog(app_ui.window, false, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it."); }
                    }

                    // If we have no MyMod selected, return an error.
                    Mode::Normal => show_dialog(app_ui.window, false, "You can't uninstall the selected MyMod if there is no MyMod selected."),
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

                    // Get the list of supported games folders.
                    let supported_folders = supported_games.iter().map(|x| x.folder_name.to_owned()).collect::<Vec<String>>();

                    // If it's a valid folder, and it's in our supported games list...
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
                                                ) { show_dialog(app_ui.window, false, format!("<p>Error while opening the PackFile:</p><p>{}</p>", error.cause())) }
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
    if let Some(ref path) = settings.paths.my_mods_base_path {

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
