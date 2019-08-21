//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is the main file of RPFM. Here is the main loop that builds the UI and controls his events.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::cast_lossless,                  // Disabled due to useless warnings.
    clippy::cognitive_complexity,           // Disabled due to useless warnings.
    clippy::cyclomatic_complexity,          // Disabled due to useless warnings.
    clippy::doc_markdown,                   // Disabled due to false positives on things that shouldn't be formated in the docs as it says.
    clippy::if_same_then_else,              // Disabled because some of the solutions it provides are freaking hard to read.
    clippy::match_bool,                     // Disabled because the solutions it provides are harder to read than the current code.
    clippy::module_inception,               // Disabled because it's quite useless.
    clippy::needless_bool,                  // Disabled because the solutions it provides are harder to read than the current code.
    clippy::new_ret_no_self,                // Disabled because the reported situations are special cases. So no, I'm not going to rewrite them.
    clippy::redundant_closure,              // Disabled because the solutions it provides doesn't even work.             
    clippy::suspicious_else_formatting,     // Disabled because the errors it gives are actually false positives due to comments.
    clippy::too_many_arguments,             // Disabled because you never have enough arguments.
    clippy::type_complexity,                // Disabled temporarily because there are other things to do before rewriting the types it warns about.
    clippy::useless_format,                 // Disabled due to false positives.
    clippy::match_wild_err_arm              // Disabled because, despite being a bad practice, it's the intended behavior in the code it warns about.
)]

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

use rpfm_lib::SETTINGS;
use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::langid;
use std::sync::RwLock;
// Uses for everything we need. It's a looooong list.
use qt_widgets::abstract_item_view::{SelectionMode, ScrollMode};
use qt_widgets::action::Action;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::application::Application;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::file_dialog::{FileDialog, FileMode, Option::ShowDirsOnly};
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::main_window::MainWindow;
use qt_widgets::menu::Menu;
use qt_widgets::message_box;
use qt_widgets::message_box::MessageBox;
use qt_widgets::push_button::PushButton;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::splitter::Splitter;
use qt_widgets::table_view::TableView;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::color::Color;
use qt_gui::cursor::Cursor;
use qt_gui::desktop_services::DesktopServices;
use qt_gui::font::Font;
use qt_gui::icon::Icon;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::palette::{Palette, ColorGroup, ColorRole};
use qt_gui::slots::SlotStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::item_selection_model::SelectionFlag;
use qt_core::object::Object;
use qt_core::qt::{CaseSensitivity, ContextMenuPolicy, Orientation, ShortcutContext, SortOrder, WindowState};
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef, SlotCInt, SlotModelIndexRef, SlotItemSelectionRefItemSelectionRef};
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::reg_exp::RegExp;
use qt_core::variant::Variant;
use cpp_utils::StaticCast;

use std::env::args;
use std::collections::BTreeMap;
use std::ops::DerefMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::OsStr;
use std::panic;
use std::path::{Path, PathBuf};
use std::fs::{DirBuilder, copy, remove_file, remove_dir_all};

use chrono::NaiveDateTime;
use lazy_static::lazy_static;

use rpfm_lib::SUPPORTED_GAMES;
use crate::shortcuts::Shortcuts;
use rpfm_error::{ErrorKind, Result};/*
use rpfm_lib::packfile::{CompressionState, PathType};
use rpfm_lib::packfile::{PFHVersion, PFHFileType, PFHFlags};

use rpfm_lib::packedfile::*;

use rpfm_lib::schema::assembly_kit::*;
use rpfm_lib::schema::Schema;
use rpfm_lib::config::init_config_path;

use rpfm_lib::common::*;
use rpfm_lib::SETTINGS;
use rpfm_lib::DOCS_BASE_URL;
use rpfm_lib::PATREON_URL;
use rpfm_lib::GAME_SELECTED;

use crate::communications::*;
use crate::main_extra::*;
use crate::ui::*;
use crate::ui::packedfile_table::db_decoder::*;
use crate::ui::packedfile_table::dependency_manager::*;
use crate::ui::packedfile_table::packedfile_db::*;
use crate::ui::packedfile_table::packedfile_loc::*;
use crate::ui::packedfile_text::packedfile_text::*;
use crate::ui::packedfile_text::packfile_notes::*;
use crate::ui::packedfile_rigidmodel::*;
use crate::ui::packfile_treeview::*;
use crate::ui::qt_custom_stuff::*;
use crate::ui::settings::*;
use crate::ui::table_state::*;
use crate::ui::updater::*;
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
/*
mod communications;
mod main_extra;
mod ui;
*/
mod app_ui;
mod command_palette;
mod background_thread;
mod ffi;
mod locale;
mod shortcuts;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// Path were the stuff used by RPFM (settings, schemas,...) is. In debug mode, we just take the current path
    /// (so we don't break debug builds). In Release mode, we take the `.exe` path.
    #[derive(Debug)]
    static ref RPFM_PATH: PathBuf = if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    };

    /// Icons for the PackFile TreeView.
    //static ref TREEVIEW_ICONS: Icons = Icons::new();

    /// Bright and dark palettes of colours for Windows.
    /// The dark one is taken from here: https://gist.github.com/QuantumCD/6245215
    static ref LIGHT_PALETTE: Palette = Palette::new(());
    static ref DARK_PALETTE: Palette = {
        let mut palette = Palette::new(());

        // Base config.
        palette.set_color((ColorRole::Window, &Color::new((51, 51, 51))));
        palette.set_color((ColorRole::WindowText, &Color::new((187, 187, 187))));
        palette.set_color((ColorRole::Base, &Color::new((34, 34, 34))));
        palette.set_color((ColorRole::AlternateBase, &Color::new((51, 51, 51))));
        palette.set_color((ColorRole::ToolTipBase, &Color::new((187, 187, 187))));
        palette.set_color((ColorRole::ToolTipText, &Color::new((187, 187, 187))));
        palette.set_color((ColorRole::Text, &Color::new((187, 187, 187))));
        palette.set_color((ColorRole::Button, &Color::new((51, 51, 51))));
        palette.set_color((ColorRole::ButtonText, &Color::new((187, 187, 187))));
        palette.set_color((ColorRole::BrightText, &Color::new((255, 0, 0))));
        palette.set_color((ColorRole::Link, &Color::new((42, 130, 218))));

        palette.set_color((ColorRole::Highlight, &Color::new((42, 130, 218))));
        palette.set_color((ColorRole::HighlightedText, &Color::new((204, 204, 204))));

        // Disabled config.
        palette.set_color((ColorGroup::Disabled, ColorRole::Window, &Color::new((34, 34, 34))));
        palette.set_color((ColorGroup::Disabled, ColorRole::WindowText, &Color::new((85, 85, 85))));
        palette.set_color((ColorGroup::Disabled, ColorRole::Base, &Color::new((34, 34, 34))));
        palette.set_color((ColorGroup::Disabled, ColorRole::AlternateBase, &Color::new((34, 34, 34))));
        palette.set_color((ColorGroup::Disabled, ColorRole::ToolTipBase, &Color::new((85, 85, 85))));
        palette.set_color((ColorGroup::Disabled, ColorRole::ToolTipText, &Color::new((85, 85, 85))));
        palette.set_color((ColorGroup::Disabled, ColorRole::Text, &Color::new((85, 85, 85))));
        palette.set_color((ColorGroup::Disabled, ColorRole::Button, &Color::new((34, 34, 34))));
        palette.set_color((ColorGroup::Disabled, ColorRole::ButtonText, &Color::new((85, 85, 85))));
        palette.set_color((ColorGroup::Disabled, ColorRole::BrightText, &Color::new((170, 0, 0))));
        palette.set_color((ColorGroup::Disabled, ColorRole::Link, &Color::new((42, 130, 218))));

        palette.set_color((ColorGroup::Disabled, ColorRole::Highlight, &Color::new((42, 130, 218))));
        palette.set_color((ColorGroup::Disabled, ColorRole::HighlightedText, &Color::new((85, 85, 85))));

        palette
    };

    /// Stylesheet used by the dark theme in Windows.
    //static ref DARK_STYLESHEET: String = create_dark_theme_stylesheet();

    // Colors used all over the program for theming and stuff.
    static ref MEDIUM_DARK_GREY: &'static str = "333333";            // Medium-Dark Grey. The color of the background of the Main Window.
    static ref MEDIUM_DARKER_GREY: &'static str = "262626";          // Medium-Darker Grey.
    static ref DARK_GREY: &'static str = "181818";                   // Dark Grey. The color of the background of the Main TreeView.
    static ref SLIGHTLY_DARKER_GREY: &'static str = "101010";        // A Bit Darker Grey.
    static ref KINDA_WHITY_GREY: &'static str = "BBBBBB";            // Light Grey. The color of the normal Text.
    static ref KINDA_MORE_WHITY_GREY: &'static str = "CCCCCC";       // Lighter Grey. The color of the highlighted Text.
    static ref EVEN_MORE_WHITY_GREY: &'static str = "FAFAFA";        // Even Lighter Grey.
    static ref BRIGHT_RED: &'static str = "FF0000";                  // Bright Red, as our Lord.
    static ref DARK_RED: &'static str = "FF0000";                    // Dark Red, as our face after facing our enemies.
    static ref LINK_BLUE: &'static str = "2A82DA";                   // Blue, used for Zeldas.
    static ref ORANGE: &'static str = "E67E22";                      // Orange, used for borders.
    static ref MEDIUM_GREY: &'static str = "555555";

    /// The current Settings and Shortcuts. To avoid reference and lock issues, this should be edited ONLY in the background thread.
    static ref SHORTCUTS: Arc<Mutex<Shortcuts>> = Arc::new(Mutex::new(Shortcuts::load().unwrap_or_else(|_|Shortcuts::new())));

    /// Variable to keep track of the state of the PackFile.
    static ref IS_MODIFIED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    /// History for the filters, search, columns...., so table and loc filters are remembered when zapping files, and cleared when the open PackFile changes.
    /// NOTE: This affects both DB Tables and Loc PackedFiles.
    //static ref TABLE_STATES_UI: Mutex<BTreeMap<Vec<String>, TableStateUI>> = Mutex::new(TableStateUI::load().unwrap_or_else(|_| TableStateUI::new()));

    /// Variable to lock/unlock certain actions of the Folder TreeView.
    static ref IS_FOLDER_TREE_VIEW_LOCKED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    
    /// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
    static ref LOCALE_FALLBACK: Arc<RwLock<FluentBundle<FluentResource>>> = {
        match locale::initialize("en") {
            Ok(locale) => locale,
            Err(_) => locale::initialize_empty(),
        }
    };
    
    /// Variable to keep the locale data used by the UI loaded and available. If we fail to load the selected locale data, copy the english one instead.
    static ref LOCALE: Arc<RwLock<FluentBundle<FluentResource>>> = {
        match SETTINGS.lock().unwrap().settings_string.get("language") {
            Some(language) => locale::initialize(language).unwrap_or_else(|_| LOCALE_FALLBACK.clone()),
            None => LOCALE_FALLBACK.clone(),
        }
    };
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
#[derive(Debug, Clone)]
pub enum Mode {
    MyMod{ game_folder_name: String, mod_name: String },
    Normal,
}

/// This enum represents a match when using the "Global Search" feature.
///  - `DB`: (path, Vec(column_name, column_number, row_number, text).
///  - `Loc`: (path, Vec(column_name, row_number, text)
#[derive(Debug, Clone)]
pub enum GlobalMatch {
    DB((Vec<String>, Vec<(String, i32, i64, String)>)),
    Loc((Vec<String>, Vec<(String, i32, i64, String)>)),
}


/// Main function.
fn main() {
    
    // Log the crashes so the user can send them himself.
    //if !cfg!(debug_assertions) { panic::set_hook(Box::new(move |info: &panic::PanicInfo| { Report::new(info, VERSION).save().unwrap(); })); }

    // If the config folder doesn't exist, and we failed to initialize it, force a crash.
    // If this fails, half the program will be broken in one way or another, so better save than sorry.
    //if let Err(error) = init_config_path() { panic!(error); }

    //---------------------------------------------------------------------------------------//
    // Preparing the Program...
    //---------------------------------------------------------------------------------------//

    // Create the channels to communicate the threads. The channels are:
    // - `sender_rust, receiver_qt`: used for returning info from the background thread, serialized in Vec<u8>.
    // - `sender_qt, receiver_rust`: used for sending the current action to the background thread.
    // - `sender_qt_data, receiver_rust_data`: used for sending the data to the background thread.
    //   The data sended and received in the last one should be always be serialized into Vec<u8>.
    //let (sender_rust, receiver_qt) = channel();
    //let (sender_qt, receiver_rust) = channel();
    //let (sender_qt_data, receiver_rust_data) = channel();

    // Create the background thread.
    thread::spawn(move || { background_thread::background_loop(/*&sender_rust, &receiver_rust, &receiver_rust_data*/); });

    // Create the application...
    Application::create_and_exit(|app| {
        use crate::app_ui::AppUI;
        use crate::app_ui::slots::AppUISlots;

        let (app_ui, slots) = AppUI::new();







/*
        //---------------------------------------------------------------------------------------//
        // Preparing the Program...
        //---------------------------------------------------------------------------------------//

        // Create the channels to communicate the threads. The channels are:
        // - `sender_rust, receiver_qt`: used for returning info from the background thread, serialized in Vec<u8>.
        // - `sender_qt, receiver_rust`: used for sending the current action to the background thread.
        // - `sender_qt_data, receiver_rust_data`: used for sending the data to the background thread.
        //   The data sended and received in the last one should be always be serialized into Vec<u8>.
        let (sender_rust, receiver_qt) = channel();
        let (sender_qt, receiver_rust) = channel();
        let (sender_qt_data, receiver_rust_data) = channel();

        // Create the background thread.
        thread::spawn(move || { background_thread::background_loop(&sender_rust, &receiver_rust, &receiver_rust_data); });


        //---------------------------------------------------------------------------------------//
        // Shortcuts for the Menu Bar...
        //---------------------------------------------------------------------------------------//

        // Set the shortcuts for these actions.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["new_packfile"]))); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["open_packfile"]))); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["save_packfile"]))); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["save_packfile_as"]))); }
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["load_all_ca_packfiles"]))); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["preferences"]))); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["quit"]))); }

        unsafe { app_ui.open_game_data_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_game_selected["open_game_data_folder"]))); }
        unsafe { app_ui.open_game_assembly_kit_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_game_selected["open_game_assembly_kit_folder"]))); }
        
        unsafe { app_ui.about_qt.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_about["about_qt"]))); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_about["about_rpfm"]))); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_about["open_manual"]))); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_about["check_updates"]))); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_about["check_schema_updates"]))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        unsafe { app_ui.open_game_data_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_game_assembly_kit_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        
        unsafe { app_ui.about_qt.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        //---------------------------------------------------------------------------------------//
        // Preparing initial state of the Main Window...
        //---------------------------------------------------------------------------------------//
        *IS_MODIFIED.lock().unwrap() = update_packfile_state(None, &app_ui);

        // This cannot go into lazy_static because StandardItem is not send.
        let table_state_data = Rc::new(RefCell::new(TableStateData::new()));

        // Put the stuff we need to move to the slots in Rc<Refcell<>>, so we can clone it without issues.
        let receiver_qt = Rc::new(RefCell::new(receiver_qt));
        let packedfiles_open_in_packedfile_view = Rc::new(RefCell::new(BTreeMap::new()));
        let mymod_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let open_from_submenu_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let mode = Rc::new(RefCell::new(Mode::Normal));

        // Build the empty structs we need for certain features.
        let slots = Rc::new(RefCell::new(vec![]));
        let monospace_font = Rc::new(RefCell::new(Font::new(&QString::from_std_str("monospace [Consolas]"))));

        // Here we store the pattern for the global search, and paths whose files have been changed/are new and need to be checked.
        let global_search_pattern = Rc::new(RefCell::new(None));
        let global_search_explicit_paths = Rc::new(RefCell::new(vec![]));

        // Signal to save the tables states to disk when we're about to close RPFM. We ignore the error here, as at this point we cannot report it to the user.
        let slot_save_states = SlotNoArgs::new(move || {
            let _y = TableStateUI::save();
        });
        app.deref_mut().signals().about_to_quit().connect(&slot_save_states);

        // Display the basic tips by default.
        display_help_tips(&app_ui);

        // Build the entire "MyMod" Menu.
        let result = build_my_mod_menu(
            &sender_qt,
            &sender_qt_data,
            &receiver_qt,
            app_ui,
            menu_bar_mymod,
            &mode,
            mymod_menu_needs_rebuild.clone(),
            &packedfiles_open_in_packedfile_view,
            close_global_search_action,
            &table_state_data
        );

        let mymod_stuff = Rc::new(RefCell::new(result.0));
        let mymod_stuff_slots = Rc::new(RefCell::new(result.1));

        // Build the "Open From Content" and "Open From Data" submenus.
        let open_from_slots = Rc::new(RefCell::new(vec![]));

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
            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(false);
        }

        // Set the shortcuts for these actions.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_file"]))); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_folder"]))); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_from_packfile"]))); }
        unsafe { app_ui.context_menu_check_tables.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["check_tables"]))); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_folder"]))); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_db"]))); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_loc"]))); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_text"]))); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["mass_import_tsv"]))); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["mass_export_tsv"]))); }
        unsafe { app_ui.context_menu_merge_tables.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["merge_tables"]))); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["delete"]))); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["extract"]))); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["rename"]))); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_in_decoder"]))); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_packfiles_list"]))); }
        unsafe { app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_containing_folder"]))); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_with_external_program"]))); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_in_multi_view"]))); }
        unsafe { app_ui.context_menu_open_notes.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_notes"]))); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["global_search"]))); }
        unsafe { app_ui.tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["expand_all"]))); }
        unsafe { app_ui.tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["collapse_all"]))); }

        // Set the shortcuts to only trigger in the TreeView.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_check_tables.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_merge_tables.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_notes.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.tree_view_expand_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.tree_view_collapse_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add the actions to the TreeView, so the shortcuts work.
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_file); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_folder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_add_from_packfile); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_check_tables); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_folder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_db); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_loc); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_create_text); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_mass_import_tsv); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_mass_export_tsv); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_merge_tables); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_delete); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_extract); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_rename); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_decoder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_dependency_manager); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_containing_folder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_with_external_program); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_in_multi_view); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_notes); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_global_search); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.tree_view_expand_all); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.tree_view_collapse_all); }

        // Set the current "Operational Mode" to `Normal`.
        set_my_mod_mode(&mymod_stuff, &mode, None);

        //---------------------------------------------------------------------------------------//
        // Action messages in the Status Bar...
        //---------------------------------------------------------------------------------------//

        // Menu bar, PackFile.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Creates a new PackFile and open it. Remember to save it later if you want to keep it!")); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open an existing PackFile, or multiple existing PackFiles into one.")); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Save the changes made in the currently open PackFile to disk.")); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_status_tip(&QString::from_std_str("Save the currently open PackFile as a new PackFile, instead of overwriting the original one.")); }
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_status_tip(&QString::from_std_str("Try to load every PackedFile from every vanilla PackFile of the selected game into RPFM at the same time, using lazy-loading to load the PackedFiles. Keep in mind that if you try to save it, your PC may die.")); }
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
        
        unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_status_tip(&QString::from_std_str("If checked, the data of each PackedFile in the open PackFile will be compressed on save. If you want to decompress a PackFile, disable this, then save it.")); }

        // Menu bar, Game Selected.
        unsafe { app_ui.open_game_data_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Tries to open the currently selected game's Data folder (if exists) in the default file manager.")); }
        unsafe { app_ui.open_game_assembly_kit_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Tries to open the currently selected game's Assembly Kit folder (if exists) in the default file manager.")); }
        
        unsafe { app_ui.three_kingdoms.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Three Kingdoms' as 'Game Selected'.")); }
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer 2' as 'Game Selected'.")); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer' as 'Game Selected'.")); }
        unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW: Thrones of Britannia' as 'Game Selected'.")); }
        unsafe { app_ui.attila.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Attila' as 'Game Selected'.")); }
        unsafe { app_ui.rome_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Rome 2' as 'Game Selected'.")); }
        unsafe { app_ui.shogun_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Shogun 2' as 'Game Selected'.")); }
        unsafe { app_ui.napoleon.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Napoleon' as 'Game Selected'.")); }
        unsafe { app_ui.empire.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Empire' as 'Game Selected'.")); }
        unsafe { app_ui.arena.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Arena' as 'Game Selected'.")); }

        // Menu bar, Special Stuff.
        let patch_siege_ai_tip = QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.");
        let optimize_packfile = QString::from_std_str("Check and remove any data in DB Tables and Locs (Locs only for english users) that is unchanged from the base game. That means your mod will only contain the stuff you change, avoiding incompatibilities with other mods.");
        let generate_pak_file = QString::from_std_str("Generates a PAK File (Processed Assembly Kit File) for the game selected, to help with dependency checking.");
        unsafe { app_ui.three_k_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.three_k_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
        unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.wh2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
        unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.wh_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.tob_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.tob_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.att_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.rom2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.sho2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.sho2_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.nap_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.nap_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }
        unsafe { app_ui.emp_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.emp_generate_pak_file.as_mut().unwrap().set_status_tip(&generate_pak_file); }

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
        unsafe { app_ui.context_menu_check_tables.as_mut().unwrap().set_status_tip(&QString::from_std_str("Check all the DB Tables of the currently open PackFile for dependency errors.")); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create an empty folder. Due to how the PackFiles are done, these are NOT KEPT ON SAVING if they stay empty.")); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Loc File (used by the game to store the texts you see ingame) in the selected folder.")); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a DB Table (used by the game for... most of the things).")); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the dialog to create a Plain Text File. It accepts different extensions, like '.xml', '.lua', '.txt',....")); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Import a bunch of TSV files at the same time. It automatically checks if they are DB Tables, Locs or invalid TSVs, and imports them all at once. Existing files will be overwritten!")); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_status_tip(&QString::from_std_str("Export every DB Table and Loc PackedFile from this PackFile as TSV files at the same time. Existing files will be overwritten!")); }
        unsafe { app_ui.context_menu_merge_tables.as_mut().unwrap().set_status_tip(&QString::from_std_str("Merge multple DB Tables/Loc PackedFiles into one.")); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_status_tip(&QString::from_std_str("Delete the selected File/Folder.")); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_status_tip(&QString::from_std_str("Extract the selected File/Folder from the PackFile.")); }
        unsafe { app_ui.context_menu_rename.as_mut().unwrap().set_status_tip(&QString::from_std_str("Rename the selected File/Folder. Remember, whitespaces are NOT ALLOWED and duplicated names in the same folder will NOT BE RENAMED.")); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the selected table in the DB Decoder. To create/update schemas.")); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the list of PackFiles referenced from this PackFile.")); }
        unsafe { app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the currently open PackFile's location in your default file manager.")); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in an external program.")); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in a secondary view, without closing the currently open one.")); }
        unsafe { app_ui.context_menu_open_notes.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackFile's Notes in a secondary view, without closing the currently open PackedFile in the Main View.")); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_status_tip(&QString::from_std_str("Performs a search over every DB Table, Loc PackedFile and Text File in the PackFile.")); }
        
        // TreeView Filter buttons.
        unsafe { app_ui.folder_tree_filter_autoexpand_matches_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Auto-Expand matches. NOTE: Filtering with all matches expanded in a big PackFile (+10k files, like data.pack) can hang the program for a while. You have been warned.")); }
        unsafe { app_ui.folder_tree_filter_case_sensitive_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Enable/Disable case sensitive filtering for the TreeView.")); }
        unsafe { app_ui.folder_tree_filter_filter_by_folder_button.as_mut().unwrap().set_status_tip(&QString::from_std_str("Set the filter to only filter by folder names and show all the files inside the matched folders.")); }

        //---------------------------------------------------------------------------------------//
        // What should happend when we press buttons and stuff...
        //---------------------------------------------------------------------------------------//

        // Actions without buttons for the TreeView.
        let slot_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { app_ui.folder_tree_view.as_mut().unwrap().expand_all(); }});
        let slot_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { app_ui.folder_tree_view.as_mut().unwrap().collapse_all(); }});
        unsafe { app_ui.tree_view_expand_all.as_ref().unwrap().signals().triggered().connect(&slot_tree_view_expand_all); }
        unsafe { app_ui.tree_view_collapse_all.as_ref().unwrap().signals().triggered().connect(&slot_tree_view_collapse_all); }

        // What happens when we want to hide the "Global Search" view.
        let slot_close_global_search = SlotNoArgs::new(clone!(
            global_search_pattern => move || {
                unsafe { global_search_widget.as_mut().unwrap().hide(); }
                *global_search_pattern.borrow_mut() = None;
            }
        ));
        unsafe { close_global_search_action.as_ref().unwrap().signals().triggered().connect(&slot_close_global_search); }

        //-----------------------------------------------------//
        // "Game Selected" Menu...
        //-----------------------------------------------------//

        // The list of icons for representing the current "Game Selected" in the UI.
        let game_selected_icons: BTreeMap<String, Icon> = {
            let mut map = BTreeMap::new();

            for (key, game) in SUPPORTED_GAMES.iter() {
                let mut path = RPFM_PATH.to_path_buf().join("img");
                path.push(game.game_selected_icon.to_owned());
                let image = Icon::new(&QString::from_std_str(path.to_str().unwrap()));
                map.insert((*key).to_owned(), image);
            }
            map
        };

        // What happens when we trigger the "Change Game Selected" action.
        let slot_change_game_selected = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            open_from_submenu_menu_needs_rebuild,
            mode,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the new Game Selected.
                let mut new_game_selected = unsafe { QString::to_std_string(&app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()) };

                // Remove the '&' from the game's name, and turn it into a `folder_name`.
                if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
                let new_game_selected_folder_name = new_game_selected.replace(' ', "_").to_lowercase();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Change the Game Selected in the Background Thread.
                sender_qt.send(Commands::SetGameSelected).unwrap();
                sender_qt_data.send(Data::String(new_game_selected_folder_name.to_owned())).unwrap();

                // Prepare to rebuild the submenu next time we try to open the PackFile menu.
                *open_from_submenu_menu_needs_rebuild.borrow_mut() = true;

                // Get the response from the background thread.
                let is_a_packfile_open = if let Data::Bool(data) = check_message_validity_tryrecv(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Disable the "PackFile Management" actions.
                enable_packfile_actions(&app_ui, &mymod_stuff, false);

                // If we have a PackFile opened, re-enable the "PackFile Management" actions, so the "Special Stuff" menu gets updated properly.
                if is_a_packfile_open { enable_packfile_actions(&app_ui, &mymod_stuff, true); }

                // Set the current "Operational Mode" to `Normal` (In case we were in `MyMod` mode).
                set_my_mod_mode(&mymod_stuff, &mode, None);

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                // Change the GameSelected Icon. Disabled until we find better icons.
                let image = game_selected_icons.get(&**GAME_SELECTED.lock().unwrap()).unwrap();
                unsafe { app_ui.window.as_mut().unwrap().set_window_icon(&image); }
            }
        ));

        // "Game Selected" Menu Actions.
        unsafe { app_ui.three_kingdoms.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.warhammer_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.warhammer.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.thrones_of_britannia.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.attila.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.rome_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.shogun_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.napoleon.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.empire.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.arena.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }

        // Update the "Game Selected" here, so we can skip some steps when initializing.
        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
        match &*game_selected {
            "three_kingdoms" => unsafe { app_ui.three_kingdoms.as_mut().unwrap().trigger(); }
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
            "arena" => unsafe { app_ui.arena.as_mut().unwrap().trigger(); }
            "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
            "shogun_2" => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
            "napoleon" => unsafe { app_ui.napoleon.as_mut().unwrap().trigger(); }
            "empire" | _ => unsafe { app_ui.empire.as_mut().unwrap().trigger(); }
        }

        //-----------------------------------------------------//
        // "PackFile" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "New PackFile" action.
        let slot_new_packfile = SlotBool::new(clone!(
            mymod_stuff,
            mode,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, false) {

                    // Destroy whatever it's in the PackedFile's view, to avoid data corruption. Also hide the Global Search stuff.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                    // Close the Global Search stuff and reset the filter's history.
                    unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                    if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                    // Show the "Tips".
                    display_help_tips(&app_ui);

                    // Tell the Background Thread to create a new PackFile.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    sender_qt.send(Commands::NewPackFile).unwrap();

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

                    // We also disable compression by default.
                    unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_checked(false); }

                    // Update the TreeView.
                    update_treeview(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        app_ui.folder_tree_view,
                        Some(app_ui.folder_tree_filter),
                        app_ui.folder_tree_model,
                        TreeViewOperation::Build(false),
                    );

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &mymod_stuff, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    set_my_mod_mode(&mymod_stuff, &mode, None);

                    // Clean the TableStateData.
                    *table_state_data.borrow_mut() = TableStateData::new(); 
                }
            }
        ));

        // What happens when we trigger the "Open PackFile" action.
        let slot_open_packfile = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            table_state_data,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, false) {

                    // Create the FileDialog to get the PackFile to open and configure it.
                    let mut file_dialog = unsafe { FileDialog::new_unsafe((
                        app_ui.window as *mut Widget,
                        &QString::from_std_str("Open PackFiles"),
                    )) };
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                    file_dialog.set_file_mode(FileMode::ExistingFiles);

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Now the fun thing. We have to get all the selected files, and then open them one by one.
                        // For that we use the same logic as for the "Load All CA PackFiles" feature.
                        let mut paths = vec![];
                        for index in 0..file_dialog.selected_files().count(()) {
                            paths.push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                        }

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &paths,
                            &app_ui,
                            &mymod_stuff,
                            &mode,
                            "",
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &table_state_data,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let slot_save_packfile = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            table_state_data,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {
                if let Err(error) = save_packfile(
                    false,
                    &app_ui,
                    &mode,
                    &mymod_stuff,
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &table_state_data,
                    &packedfiles_open_in_packedfile_view
                ) { show_dialog(app_ui.window, false, error); }
            }
        ));

        // What happens when we trigger the "Save PackFile As" action.
        let slot_save_packfile_as = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            table_state_data,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {
                if let Err(error) = save_packfile(
                    true,
                    &app_ui,
                    &mode,
                    &mymod_stuff,
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &table_state_data,
                    &packedfiles_open_in_packedfile_view
                ) { show_dialog(app_ui.window, false, error); }   
            }
        ));

        // What happens when we trigger the "Load All CA PackFiles" action.
        let slot_load_all_ca_packfiles = SlotBool::new(clone!(
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, false) {

                    // Tell the Background Thread to try to load the PackFiles.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    sender_qt.send(Commands::LoadAllCAPackFiles).unwrap();
                    match check_message_validity_tryrecv(&receiver_qt) {
                    
                        // If it's success....
                        Data::PackFileUIData(data) => {

                            // This PackFile is a special one. It'll always be type "Other(200)" with every special stuff as false.
                            // TODO: Encrypted PackedFiles haven't been tested with this.
                            unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                            unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                            // Set the compression level correctly, because otherwise we may fuckup some files.
                            let compression_state = match data.compression_state {
                                CompressionState::Enabled => true,
                                CompressionState::Partial | CompressionState::Disabled => false,
                            };
                            unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_checked(compression_state); }

                            // Update the TreeView.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Build(false),
                            );

                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                            match &*game_selected {
                                "three_kingdoms" => unsafe { app_ui.three_kingdoms.as_mut().unwrap().trigger(); },
                                "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); },
                                "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                                "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                                "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                                "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                                "shogun_2" => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                                "napoleon" => unsafe { app_ui.napoleon.as_mut().unwrap().trigger(); }
                                "empire" => unsafe { app_ui.empire.as_mut().unwrap().trigger(); }
                                "arena" => unsafe { app_ui.arena.as_mut().unwrap().trigger(); },
                                _ => unreachable!()
                            }

                            // Set the current "Operational Mode" to `Normal`.
                            set_my_mod_mode(&mymod_stuff, &mode, None);

                            // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                            // Close the Global Search stuff and reset the filter's history.
                            unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                            if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                            // Show the "Tips".
                            display_help_tips(&app_ui);

                            // Clean the TableStateData.
                            *table_state_data.borrow_mut() = TableStateData::new(); 
                        }

                        // If we got an error...
                        Data::Error(error) => show_dialog(app_ui.window, false, error),

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            }
        ));

        // What happens when we trigger the "Change PackFile Type" action.
        let slot_change_packfile_type = SlotBool::new(clone!(
            app_ui,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the currently selected PackFile's Type.
                let packfile_type = unsafe { match &*(app_ui.change_packfile_type_group.as_mut().unwrap().checked_action().as_mut().unwrap().text().to_std_string()) {
                    "&Boot" => PFHFileType::Boot,
                    "&Release" => PFHFileType::Release,
                    "&Patch" => PFHFileType::Patch,
                    "&Mod" => PFHFileType::Mod,
                    "Mo&vie" => PFHFileType::Movie,
                    _ => PFHFileType::Other(99),
                } };

                // Send the type to the Background Thread.
                sender_qt.send(Commands::SetPackFileType).unwrap();
                sender_qt_data.send(Data::PFHFileType(packfile_type)).unwrap();

                // Modify the PackFile.
                update_treeview(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    app_ui.folder_tree_view,
                    Some(app_ui.folder_tree_filter),
                    app_ui.folder_tree_model,
                    TreeViewOperation::Modify(vec![TreePathType::PackFile]),
                );
            }
        ));

        // What happens when we change the value of "Include Last Modified Date" action.
        let slot_index_includes_timestamp = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data => move |_| {

                // Get the current value of the action.
                let state = unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_ref().unwrap().is_checked() };

                // Send the new state to the background thread.
                sender_qt.send(Commands::ChangeIndexIncludesTimestamp).unwrap();
                sender_qt_data.send(Data::Bool(state)).unwrap();
            }
        ));

        // What happens when we enable/disable compression on the current PackFile.
        let slot_data_is_compressed = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data => move |_| {

                let state = unsafe { app_ui.change_packfile_type_data_is_compressed.as_ref().unwrap().is_checked() };
                sender_qt.send(Commands::ChangeDataIsCompressed).unwrap();
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


                // Create the Settings Dialog. If we got new settings...
                let old_settings = SETTINGS.lock().unwrap().clone();
                if let Some(settings) = SettingsDialog::create_settings_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt) {

                    // Send the signal to save them.
                    sender_qt.send(Commands::SetSettings).unwrap();
                    sender_qt_data.send(Data::Settings(settings.clone())).unwrap();

                    // Check what response we got.
                    match check_message_validity_recv2(&receiver_qt) {

                        // If we got confirmation....
                        Data::Success => {

                            // If we changed the "MyMod's Folder" path...
                            if settings.paths["mymods_base_path"] != old_settings.paths["mymods_base_path"] {

                                // We disable the "MyMod" mode, but leave the PackFile open, so the user doesn't lose any unsaved change.
                                set_my_mod_mode(&mymod_stuff, &mode, None);

                                // Then set it to recreate the "MyMod" submenu next time we try to open it.
                                *mymod_menu_needs_rebuild.borrow_mut() = true;
                            }

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // update the current `GameSelected`.
                            let mut games_with_changed_paths = vec![];
                            for (key, value) in settings.paths.iter() {
                                if key != "mymods_base_path" && &old_settings.paths[key] != value {
                                    games_with_changed_paths.push(key.to_owned());
                                }
                            } 

                            // If our current `GameSelected` is in the `games_with_changed_paths` list...
                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
                            if games_with_changed_paths.contains(&game_selected) {

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
            app_ui => move |_| {
                if are_you_sure(&app_ui, false) {
                    unsafe { app_ui.window.as_mut().unwrap().close(); }
                }
            }
        ));

        // "PackFile" Menu Actions.
        unsafe { app_ui.new_packfile.as_ref().unwrap().signals().triggered().connect(&slot_new_packfile); }
        unsafe { app_ui.open_packfile.as_ref().unwrap().signals().triggered().connect(&slot_open_packfile); }
        unsafe { app_ui.save_packfile.as_ref().unwrap().signals().triggered().connect(&slot_save_packfile); }
        unsafe { app_ui.save_packfile_as.as_ref().unwrap().signals().triggered().connect(&slot_save_packfile_as); }
        unsafe { app_ui.load_all_ca_packfiles.as_ref().unwrap().signals().triggered().connect(&slot_load_all_ca_packfiles); }

        unsafe { app_ui.change_packfile_type_boot.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_release.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_patch.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_mod.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_movie.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_other.as_ref().unwrap().signals().triggered().connect(&slot_change_packfile_type); }
        unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_ref().unwrap().signals().triggered().connect(&slot_index_includes_timestamp); }
        unsafe { app_ui.change_packfile_type_data_is_compressed.as_ref().unwrap().signals().triggered().connect(&slot_data_is_compressed); }

        unsafe { app_ui.preferences.as_ref().unwrap().signals().triggered().connect(&slot_preferences); }
        unsafe { app_ui.quit.as_ref().unwrap().signals().triggered().connect(&slot_quit); }

        //-----------------------------------------------------//
        // "Game Selected" Menu (only actions)...
        //-----------------------------------------------------//

        // What happens when we trigger the "Open Game's Data Folder" action.
        let slot_open_game_data_folder = SlotBool::new(move |_| {
            if let Some(path) = get_game_selected_data_path(&*GAME_SELECTED.lock().unwrap()) {
                if open::that(&path).is_err() { show_dialog(app_ui.window, false, ErrorKind::IOFolderCannotBeOpened); };
            }
            else { show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
        });

        // What happens when we trigger the "Open Game's Assembly Kit Folder" action.
        let slot_open_game_assembly_kit_folder = SlotBool::new(move |_| {
            if let Some(path) = get_game_selected_assembly_kit_path(&*GAME_SELECTED.lock().unwrap()) {
                if open::that(&path).is_err() { show_dialog(app_ui.window, false, ErrorKind::IOFolderCannotBeOpened)};
            }
            else { show_dialog(app_ui.window, false, ErrorKind::GamePathNotConfigured); }
        });

        // "Game Selected" menu actions.
        unsafe { app_ui.open_game_data_folder.as_ref().unwrap().signals().triggered().connect(&slot_open_game_data_folder); }
        unsafe { app_ui.open_game_assembly_kit_folder.as_ref().unwrap().signals().triggered().connect(&slot_open_game_assembly_kit_folder); }

        //-----------------------------------------------------//
        // "Special Stuff" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "Patch Siege AI" action.
        let slot_patch_siege_ai = SlotBool::new(clone!(
            receiver_qt,
            mode,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            mymod_stuff,
            sender_qt,
            sender_qt_data => move |_| {

                // Ask the background loop to patch the PackFile, and wait for a response.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                sender_qt.send(Commands::PatchSiegeAI).unwrap();
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::StringVecPathType(response) => {
                        let response = (response.0, response.1.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>());
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            app_ui.folder_tree_view,
                            Some(app_ui.folder_tree_filter),
                            app_ui.folder_tree_model,
                            TreeViewOperation::Delete(response.1),
                        );

                        if let Err(error) = save_packfile(
                            false,
                            &app_ui,
                            &mode,
                            &mymod_stuff,
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &table_state_data,
                            &packedfiles_open_in_packedfile_view
                        ) { show_dialog(app_ui.window, false, error); }
                        else { show_dialog(app_ui.window, true, &response.0); }
                    }

                    // If the PackFile is empty or is not patchable, report it. Otherwise, praise the nine divines.
                    Data::Error(error) => {
                        match error.kind() {
                            ErrorKind::PatchSiegeAIEmptyPackFile |
                            ErrorKind::PatchSiegeAINoPatchableFiles => show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }
                    _ => panic!(THREADS_MESSAGE_ERROR)
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Optimize PackFile" action.
        let slot_optimize_packfile = SlotBool::new(clone!(
            packedfiles_open_in_packedfile_view,
            mode,
            mymod_stuff,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // This cannot be done if there is a PackedFile open. Well, can be done, but it's a pain in the ass to do it.
                if !packedfiles_open_in_packedfile_view.borrow().is_empty() { return show_dialog(app_ui.window, false, ErrorKind::OperationNotAllowedWithPackedFileOpen); }
            
                // If there is no problem, ere we go.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                sender_qt.send(Commands::OptimizePackFile).unwrap();
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::VecPathType(response) => {
                        let response = response.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            app_ui.folder_tree_view,
                            Some(app_ui.folder_tree_filter),
                            app_ui.folder_tree_model,
                            TreeViewOperation::Delete(response),
                        );

                        if let Err(error) = save_packfile(
                            false,
                            &app_ui,
                            &mode,
                            &mymod_stuff,
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &table_state_data,
                            &packedfiles_open_in_packedfile_view
                        ) { show_dialog(app_ui.window, false, error); }
                        else { show_dialog(app_ui.window, true, "PackFile optimized and saved."); }

                        // Update the global search stuff, if needed.
                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                    }

                    // If there was an error while optimizing... we got the wrong side of the coin.
                    Data::Error(error) => show_dialog(app_ui.window, false, error),
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Generate Pak File" action.
        let slot_generate_pak_file = SlotBool::new(clone!(
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // If there is no problem, ere we go.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // For Rome 2+, we need the game path set. For other games, we have to ask for a path.
                let version = SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().raw_db_version;
                let path = match version {

                    // Post-Shogun 2 games.
                    2 => {
                        let mut path = SETTINGS.lock().unwrap().paths[&**GAME_SELECTED.lock().unwrap()].clone().unwrap();
                        path.push("assembly_kit");
                        path.push("raw_data");
                        path.push("db");
                        path
                    }

                    // Shogun 2.
                    1 => {

                        // Create the FileDialog to get the path of the Assembly Kit.
                        let mut file_dialog = unsafe { FileDialog::new_unsafe((
                            app_ui.window as *mut Widget,
                            &QString::from_std_str("Select Assembly Kit's Folder"),
                        )) };

                        // Set it to only search Folders.
                        file_dialog.set_file_mode(FileMode::Directory);
                        file_dialog.set_option(ShowDirsOnly);

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        let mut path = if file_dialog.exec() == 1 { PathBuf::from(file_dialog.selected_files().at(0).to_std_string()) 
                        } else { PathBuf::from("") };
                        path.push("raw_data");
                        path.push("db");
                        path
                    }

                    // Empire and Napoleon.
                    0 => {

                        // Create the FileDialog to get the path of the Assembly Kit.
                        let mut file_dialog = unsafe { FileDialog::new_unsafe((
                            app_ui.window as *mut Widget,
                            &QString::from_std_str("Select Raw DB Folder"),
                        )) };

                        // Set it to only search Folders.
                        file_dialog.set_file_mode(FileMode::Directory);
                        file_dialog.set_option(ShowDirsOnly);

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 { PathBuf::from(file_dialog.selected_files().at(0).to_std_string()) 
                        } else { PathBuf::from("") }
                    }

                    // For any other game, return an empty path.
                    _ => PathBuf::new(),
                };

                if path.file_name().is_some() {
                    sender_qt.send(Commands::GeneratePakFile).unwrap();
                    sender_qt_data.send(Data::PathBufI16((path, version))).unwrap();
                    match check_message_validity_tryrecv(&receiver_qt) {
                        Data::Success => show_dialog(app_ui.window, true, "PAK File succesfully created and reloaded."),
                        Data::Error(error) => show_dialog(app_ui.window, false, error),
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }
                }
                else {
                    show_dialog(app_ui.window, false, "This operation is not supported for the Game Selected.");
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // "Special Stuff" Menu Actions.        
        unsafe { app_ui.wh2_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }
        unsafe { app_ui.wh_patch_siege_ai.as_ref().unwrap().signals().triggered().connect(&slot_patch_siege_ai); }

        unsafe { app_ui.three_k_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.wh2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.wh_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.tob_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.att_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.rom2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.sho2_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.nap_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }
        unsafe { app_ui.emp_optimize_packfile.as_ref().unwrap().signals().triggered().connect(&slot_optimize_packfile); }

        unsafe { app_ui.three_k_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.wh2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.wh_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.tob_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.att_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.rom2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.sho2_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.nap_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }
        unsafe { app_ui.emp_generate_pak_file.as_ref().unwrap().signals().triggered().connect(&slot_generate_pak_file); }

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
                            <li>LUA Types for Kailua: <b>DrunkFlamingo</b>.</li>
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
        let slot_open_manual = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str(*DOCS_BASE_URL))); });

        // What happens when we trigger the "Support me on Patreon" action.
        let slot_patreon_link = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str(*PATREON_URL))); });

        // What happens when we trigger the "Check Updates" action.
        let slot_check_updates = SlotBool::new(move |_| { check_updates(&app_ui, true); });

        // What happens when we trigger the "Check Schema Updates" action.
        let slot_check_schema_updates = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| { check_schema_updates(&app_ui, true, &sender_qt, &sender_qt_data, &receiver_qt) }));

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
            receiver_qt => move |_,_| {

                // Get the currently selected paths, and get how many we have of each type.
                let selected_items = get_item_types_from_main_treeview_selection(&app_ui);
                let (mut file, mut folder, mut packfile, mut none) = (0, 0, 0, 0);
                let mut item_types = vec![];
                for item_type in &selected_items {
                    match item_type {
                        TreePathType::File(_) => file += 1,
                        TreePathType::Folder(_) => folder += 1,
                        TreePathType::PackFile => packfile += 1,
                        TreePathType::None => none += 1,
                    }
                    item_types.push(item_type);
                }

                // Now we do some bitwise magic to get what type of selection combination we have.
                let mut contents: u8 = 0;
                if file != 0 { contents |= 1; } 
                if folder != 0 { contents |= 2; } 
                if packfile != 0 { contents |= 4; } 
                if none != 0 { contents |= 8; } 
                match contents {

                    // Only one or more files selected.
                    1 => {

                        // These options are valid for 1 or more files.
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 file selected, and should not be usable if multiple files
                        // are selected.
                        let enabled = if file == 1 { true } else { false };
                        unsafe {
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(enabled);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(enabled);
                        }

                        // Only if we have multiple files selected, we give the option to merge. Further checkings are done when clicked.
                        let enabled = if file > 1 { true } else { false };
                        unsafe { app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(enabled); }

                        // If we only have selected one file and it's a DB, we should enable this too.
                        let mut enable_db_decoder = false;
                        if file == 1 {
                            if let TreePathType::File(data) = &item_types[0] {                                
                                if !data.is_empty() && data.starts_with(&["db".to_owned()]) && data.len() == 3 {
                                    enable_db_decoder = true;
                                }
                            }
                        }
                        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(enable_db_decoder); }
                    },

                    // Only one or more folders selected.
                    2 => {

                        // These options are valid for 1 or more folders.
                        unsafe {
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 folder selected.
                        let enabled = if folder == 1 { true } else { false };
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(enabled);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(enabled);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(enabled);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(enabled);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(enabled);
                        }
                    },

                    // One or more files and one or more folders selected.
                    3 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // One PackFile (you cannot have two in the same TreeView) selected.
                    4 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more files selected.
                    5 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more folders selected.
                    6 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile, one or more files, and one or more folders selected.
                    7 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value. 
                    0 | 8..=255 => {
                        unsafe {
                            app_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(false);
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
                    unsafe { app_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false); }
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
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            mode => move |_| {

                // Create the FileDialog to get the file/s to add and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Add File/s"),
                )) };
                file_dialog.set_file_mode(FileMode::ExistingFiles);

                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let settings = SETTINGS.lock().unwrap().clone();
                        let mymods_base_path = &settings.paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
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
                                let paths_packedfile = if paths[0].starts_with(&assets_folder) {

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

                                // If we have a PackedFile open and it's on the adding list, ask the user to be sure. Do it in rev, otherwise it has problems.
                                let mut views = vec![];
                                for (view, packed_file) in packedfiles_open_in_packedfile_view.borrow().iter().rev() {
                                    if paths_packedfile.contains(&packed_file.borrow()) { views.push(*view); }
                                }
                                if !views.is_empty() {
                                    let mut dialog = unsafe { MessageBox::new_unsafe((
                                        message_box::Icon::Information,
                                        &QString::from_std_str("One or more of the PackedFiles you want to replace is open."),
                                        &QString::from_std_str("Are you sure you want to replace it? Hitting yes will close it."),
                                        Flags::from_int(16384) | Flags::from_int(65536),
                                        app_ui.window as *mut Widget,
                                    )) };

                                    // 16384 means yes.
                                    if dialog.exec() != 16384 { return }
                                    else { 
                                        for view in &views {
                                            purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view); 
                                            let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                            let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                            if visible_widgets == 0 { display_help_tips(&app_ui); }
                                        }
                                    }
                                }

                                // Tell the Background Thread to add the files.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            &receiver_qt,
                                            &app_ui,
                                            app_ui.folder_tree_view,
                                            Some(app_ui.folder_tree_filter),
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths),
                                        );

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                        // For each file added, remove it from the data history if exists.
                                        for path in &paths_packedfile {
                                            if table_state_data.borrow().get(path).is_some() {
                                                table_state_data.borrow_mut().remove(path);
                                            }
                                            let data = TableStateData::new_empty();
                                            table_state_data.borrow_mut().insert(path.to_vec(), data);
                                        }
                                    }

                                    // If we got an error, just show it.
                                    Data::Error(error) => show_dialog(app_ui.window, false, error),

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

                            // If we have a PackedFile open and it's on the adding list, ask the user to be sure. Do it in rev, otherwise it has problems.
                            let mut views = vec![];
                            for (view, packed_file) in packedfiles_open_in_packedfile_view.borrow().iter().rev() {
                                if paths_packedfile.contains(&packed_file.borrow()) { views.push(*view); }
                            }
                            if !views.is_empty() {
                                let mut dialog = unsafe { MessageBox::new_unsafe((
                                    message_box::Icon::Information,
                                    &QString::from_std_str("One or more of the PackedFiles you want to replace is open."),
                                    &QString::from_std_str("Are you sure you want to replace it? Hitting yes will close it."),
                                    Flags::from_int(16384) | Flags::from_int(65536),
                                    app_ui.window as *mut Widget,
                                )) };

                                // 16384 means yes.
                                if dialog.exec() != 16384 { return }
                                else { 
                                    for view in &views {
                                        purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view); 
                                        let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                        let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                        if visible_widgets == 0 { display_help_tips(&app_ui); }
                                    }
                                }
                            }

                            // Tell the Background Thread to add the files.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &app_ui,
                                        app_ui.folder_tree_view,
                                        Some(app_ui.folder_tree_filter),
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths),
                                    );

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                    // For each file added, remove it from the data history if exists.
                                    for path in &paths_packedfile {
                                        if table_state_data.borrow().get(path).is_some() {
                                            table_state_data.borrow_mut().remove(path);
                                        }
                                        let data = TableStateData::new_empty();
                                        table_state_data.borrow_mut().insert(path.to_vec(), data);
                                    }
                                }

                                // If we got an error, just show it.
                                Data::Error(error) => show_dialog(app_ui.window, false, error),

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
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            mode => move |_| {

                // Create the FileDialog to get the folder/s to add.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Add Folder/s"),
                )) };

                // TODO: Make this able to select multiple directories at once.
                file_dialog.set_file_mode(FileMode::Directory);

                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let settings = SETTINGS.lock().unwrap().clone();
                        let mymods_base_path = &settings.paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
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
                                let paths_packedfile = if paths[0].starts_with(&assets_folder) {

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

                                // If we have a PackedFile open and it's on the adding list, ask the user to be sure. Do it in rev, otherwise it has problems.
                                let mut views = vec![];
                                for (view, packed_file) in packedfiles_open_in_packedfile_view.borrow().iter().rev() {
                                    if paths_packedfile.contains(&packed_file.borrow()) { views.push(*view); }
                                }
                                if !views.is_empty() {
                                    let mut dialog = unsafe { MessageBox::new_unsafe((
                                        message_box::Icon::Information,
                                        &QString::from_std_str("One or more of the PackedFiles you want to replace is open."),
                                        &QString::from_std_str("Are you sure you want to replace it? Hitting yes will close it."),
                                        Flags::from_int(16384) | Flags::from_int(65536),
                                        app_ui.window as *mut Widget,
                                    )) };

                                    // 16384 means yes.
                                    if dialog.exec() != 16384 { return }
                                    else { 
                                        for view in &views {
                                            purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view); 
                                            let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                            let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                            if visible_widgets == 0 { display_help_tips(&app_ui); }
                                        }
                                    }
                                }

                                // Tell the Background Thread to add the files.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            &receiver_qt,
                                            &app_ui,
                                            app_ui.folder_tree_view,
                                            Some(app_ui.folder_tree_filter),
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths),
                                        );

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                        // For each file added, remove it from the data history if exists.
                                        for path in &paths_packedfile {
                                            if table_state_data.borrow().get(path).is_some() {
                                                table_state_data.borrow_mut().remove(path);
                                            }

                                            let data = TableStateData::new_empty();
                                            table_state_data.borrow_mut().insert(path.to_vec(), data);
                                        }
                                    }

                                    // If we got an error, just show it.
                                    Data::Error(error) => show_dialog(app_ui.window, false, error),

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

                            // If we have a PackedFile open and it's on the adding list, ask the user to be sure. Do it in rev, otherwise it has problems.
                            let mut views = vec![];
                            for (view, packed_file) in packedfiles_open_in_packedfile_view.borrow().iter().rev() {
                                if paths_packedfile.contains(&packed_file.borrow()) { views.push(*view); }
                            }
                            if !views.is_empty() {
                                let mut dialog = unsafe { MessageBox::new_unsafe((
                                    message_box::Icon::Information,
                                    &QString::from_std_str("One or more of the PackedFiles you want to replace is open."),
                                    &QString::from_std_str("Are you sure you want to replace it? Hitting yes will close it."),
                                    Flags::from_int(16384) | Flags::from_int(65536),
                                    app_ui.window as *mut Widget,
                                )) };

                                // 16384 means yes.
                                if dialog.exec() != 16384 { return }
                                else { 
                                    for view in &views {
                                        purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view); 
                                        let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                        let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                        if visible_widgets == 0 { display_help_tips(&app_ui); }
                                    }
                                }
                            }

                            // Tell the Background Thread to add the files.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    let paths = paths_packedfile.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        &receiver_qt,
                                        &app_ui,
                                        app_ui.folder_tree_view,
                                        Some(app_ui.folder_tree_filter),
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths),
                                    );

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                    // For each file added, remove it from the data history if exists.
                                    for path in &paths_packedfile {
                                        if table_state_data.borrow().get(path).is_some() {
                                            table_state_data.borrow_mut().remove(path);
                                        }
                                        let data = TableStateData::new_empty();
                                        table_state_data.borrow_mut().insert(path.to_vec(), data);
                                    }
                                }

                                // If we got an error, just show it.
                                Data::Error(error) => show_dialog(app_ui.window, false, error),

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
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            slots => move |_| {

                // Create the FileDialog to get the PackFile to open.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Select PackFile"),
                )) };

                // Filter it so it only shows PackFiles.
                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));

                // Run it and expect a response (1 => Accept, 0 => Cancel).
                if file_dialog.exec() == 1 {

                    // Get the path of the selected file and turn it in a Rust's PathBuf.
                    let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                    // Tell the Background Thread to open the secondary PackFile.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    sender_qt.send(Commands::OpenPackFileExtra).unwrap();
                    sender_qt_data.send(Data::PathBuf(path)).unwrap();

                    // Get the data from the operation...
                    match check_message_validity_tryrecv(&receiver_qt) {
                        
                        // If it's success....
                        Data::Success => {

                            // Destroy whatever it's in the PackedFile's View.
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                            // Block the main `TreeView` from decoding stuff.
                            *IS_FOLDER_TREE_VIEW_LOCKED.lock().unwrap() = true;

                            // Build the TreeView to hold all the Extra PackFile's data and save his slots.
                            slots.borrow_mut().push(TheOneSlot::TreeView(AddFromPackFileSlots::new_with_grid(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui,
                                &packedfiles_open_in_packedfile_view,
                                &global_search_explicit_paths,
                                update_global_search_stuff,
                                &table_state_data
                            )));
                        }

                        Data::Error(error) => {
                            match error.kind() {
                                ErrorKind::OpenPackFileGeneric(_) => show_dialog(app_ui.window, false, error),
                                _ => panic!(THREADS_MESSAGE_ERROR)
                            }
                        }
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            }
        ));

        // What happens when we trigger the "Create Folder" Action.
        let slot_contextual_menu_create_folder = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Create the "New Folder" dialog and wait for a new name (or a cancelation).
                if let Some(new_folder_name) = create_new_folder_dialog(&app_ui) {

                    // Get the currently selected paths, and only continue if there is only one.
                    let selected_paths = get_path_from_main_treeview_selection(&app_ui);
                    if selected_paths.len() == 1 {

                        // Add the folder's name to the list.
                        let mut complete_path = selected_paths[0].to_vec();
                        complete_path.append(&mut (new_folder_name.split("/").map(|x| x.to_owned()).filter(|x| !x.is_empty()).collect::<Vec<String>>()));

                        // Check if the folder exists.
                        sender_qt.send(Commands::FolderExists).unwrap();
                        sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                        let folder_exists = if let Data::Bool(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // If the folder already exists, return an error.
                        if folder_exists { return show_dialog(app_ui.window, false, ErrorKind::FolderAlreadyInPackFile)}

                        // Add the new Folder to the TreeView.
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            app_ui.folder_tree_view,
                            Some(app_ui.folder_tree_filter),
                            app_ui.folder_tree_model,
                            TreeViewOperation::Add(vec![TreePathType::Folder(complete_path); 1]),
                        );
                    }
                }
            }
        ));

        // What happens when we trigger the "Create DB PackedFile" Action.
        let slot_contextual_menu_create_packed_file_db = SlotBool::new(clone!(
            table_state_data,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {
                create_packed_files(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &table_state_data,
                    &app_ui,
                    &PackedFileType::DB("".to_owned(), "".to_owned(), 0)
                );
            }
        ));

        // What happens when we trigger the "Create Loc PackedFile" Action.
        let slot_contextual_menu_create_packed_file_loc = SlotBool::new(clone!(
            table_state_data,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {
                create_packed_files(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &table_state_data,
                    &app_ui,
                    &PackedFileType::Loc(String::new())
                );
            }
        ));

        // What happens when we trigger the "Create Text PackedFile" Action.
        let slot_contextual_menu_create_packed_file_text = SlotBool::new(clone!(
            table_state_data,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {
                create_packed_files(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &table_state_data,
                    &app_ui,
                    &PackedFileType::Text(String::new())
                );
            }
        ));

        // What happens when we trigger the "Mass-Import TSV" Action.
        let slot_contextual_menu_mass_import_tsv = SlotBool::new(clone!(
            packedfiles_open_in_packedfile_view,
            global_search_explicit_paths,
            table_state_data,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Don't do anything if there is a PackedFile open. This fixes the situation where you could overwrite data already in the UI.
                if !packedfiles_open_in_packedfile_view.borrow().is_empty() { return show_dialog(app_ui.window, false, ErrorKind::PackedFileIsOpen) }

                // Create the "Mass-Import TSV" dialog and wait for his data (or a cancelation).
                if let Some(data) = create_mass_import_tsv_dialog(&app_ui) {

                    // If there is no name provided, nor TSV file selected, return an error.
                    if let Some(ref name) = data.0 {
                        if name.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }
                    }
                    if data.1.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::NoFilesToImport) }

                    // Otherwise, try to import all of them and report the result.
                    else {
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        sender_qt.send(Commands::MassImportTSV).unwrap();
                        sender_qt_data.send(Data::OptionStringVecPathBuf(data)).unwrap();
                        match check_message_validity_tryrecv(&receiver_qt) {
                            
                            // If it's success....
                            Data::VecVecStringVecVecString(paths) => {

                                // Get the list of paths to add, removing those we "replaced".
                                let mut paths_to_add = paths.1.to_vec();
                                paths_to_add.retain(|x| !paths.0.contains(&x));
                                let paths_to_add2 = paths_to_add.iter().map(|x| TreePathType::File(x.to_vec())).collect::<Vec<TreePathType>>();

                                // Update the TreeView.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    &receiver_qt,
                                    &app_ui,
                                    app_ui.folder_tree_view,
                                    Some(app_ui.folder_tree_filter),
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(paths_to_add2),
                                );

                                // Update the global search stuff, if needed.
                                global_search_explicit_paths.borrow_mut().append(&mut paths_to_add);
                                unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                // For each file added, remove it from the data history if exists.
                                for path in &paths.1 {
                                    if table_state_data.borrow().get(path).is_some() {
                                        table_state_data.borrow_mut().remove(path);
                                    }

                                    let data = TableStateData::new_empty();
                                    table_state_data.borrow_mut().insert(path.to_vec(), data);
                                }
                            }

                            Data::Error(error) => show_dialog(app_ui.window, true, error),
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
                let export_path = unsafe { FileDialog::get_existing_directory_unsafe((
                    app_ui.window as *mut Widget,
                    &QString::from_std_str("Select destination folder")
                )) };

                // If we got an export path and it's not empty, try to export all exportable files there.
                if !export_path.is_empty() {
                    let export_path = PathBuf::from(export_path.to_std_string());
                    if export_path.is_dir() {
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        sender_qt.send(Commands::MassExportTSV).unwrap();
                        sender_qt_data.send(Data::PathBuf(export_path)).unwrap();
                        match check_message_validity_tryrecv(&receiver_qt) {
                            Data::String(response) => show_dialog(app_ui.window, true, response),
                            Data::Error(error) => show_dialog(app_ui.window, true, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Check Tables" action in the Contextual Menu.
        let slot_contextual_menu_check_tables = SlotBool::new(clone!(
            sender_qt,
            receiver_qt => move |_| {
                
                // Disable the window and trigger the check for all tables in the PackFile.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                sender_qt.send(Commands::CheckTables).unwrap();
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::Success => show_dialog(app_ui.window, true, "No errors detected."),
                    Data::Error(error) => show_dialog(app_ui.window, false, error),
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Merge" action in the Contextual Menu.
        let slot_contextual_menu_merge_tables = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packedfiles_open_in_packedfile_view,
            global_search_explicit_paths,
            table_state_data => move |_| {
                
                // Get the currently selected paths, and get how many we have of each type.
                let selected_paths = get_path_from_main_treeview_selection(&app_ui);

                // First, we check if we're merging locs, as it's far simpler.
                let mut loc_pass = true;
                for path in &selected_paths {
                    if !path.last().unwrap().ends_with(".loc") {
                        loc_pass = false;
                        break;
                    }
                }

                // Then DB Tables. The conditions are that they're in the same db folder and with the same version.
                // If ANY of these fails (until the "update table version" feature it's done), we fail the pass.
                // Due to performance reasons, the version thing will be done later.
                let mut db_pass = true;
                let mut db_folder = String::new();
                for path in &selected_paths {
                    if path.len() == 3 {
                        if path[0] == "db" {
                            if db_folder.is_empty() {
                                db_folder = path[1].to_owned();
                            }

                            if path[1] != db_folder {
                                db_pass = false;
                                break;                                
                            }
                        }
                        else {
                            db_pass = false;
                            break;
                        }
                    }
                    else {
                        db_pass = false;
                        break;
                    }
                }

                // If we got valid files, create the dialog to ask for the needed info.
                if (loc_pass || db_pass) && !(loc_pass && db_pass) {
    
                    // If we have a PackedFile open, throw the usual warning.
                    if !packedfiles_open_in_packedfile_view.borrow().is_empty() {

                        let mut dialog = unsafe { MessageBox::new_unsafe((
                            message_box::Icon::Information,
                            &QString::from_std_str("Warning"),
                            &QString::from_std_str("<p>If you do this, RPFM will close whatever PackedFile is open in the right view.</p><p> Are you sure you want to continue?</p>"),
                            Flags::from_int(4_194_304), // Cancel button.
                            app_ui.window as *mut Widget,
                        )) };

                        dialog.add_button((&QString::from_std_str("&Accept"), message_box::ButtonRole::AcceptRole));
                        dialog.set_modal(true);
                        dialog.show();

                        // If we hit "Accept", close all PackedFiles.
                        if dialog.exec() == 0 { 
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                            display_help_tips(&app_ui);
                        } else { return }
                    }

                    // Get the info for the merged file.
                    if let Some((mut name, delete_source_files)) = create_merge_tables_dialog(&app_ui) {

                        // If it's a loc file and the name doesn't end in a ".loc" termination, call it ".loc".
                        if loc_pass && !name.ends_with(".loc") {
                            name.push_str(".loc");
                        }

                        sender_qt.send(Commands::MergeTables).unwrap();
                        sender_qt_data.send(Data::VecVecStringStringBoolBool((selected_paths, name, delete_source_files, if db_pass { true } else { false }))).unwrap();
                        match check_message_validity_recv2(&receiver_qt) {
                            Data::VecStringVecPathType((path_to_add, items_to_remove)) => {
                                let items_to_remove = items_to_remove.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();

                                // First, we need to remove the removed tables, if any.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    &receiver_qt,
                                    &app_ui,
                                    app_ui.folder_tree_view,
                                    Some(app_ui.folder_tree_filter),
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Delete(items_to_remove.to_vec()),
                                );

                                // Next, we need to add the new table.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    &receiver_qt,
                                    &app_ui,
                                    app_ui.folder_tree_view,
                                    Some(app_ui.folder_tree_filter),
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(vec![TreePathType::File(path_to_add.to_vec()); 1]),
                                );

                                // Update the global search stuff, if needed.
                                global_search_explicit_paths.borrow_mut().append(&mut vec![path_to_add.to_vec()]);
                                unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                // Remove the added file from the data history if exists.
                                if table_state_data.borrow().get(&path_to_add).is_some() {
                                    table_state_data.borrow_mut().remove(&path_to_add);
                                }

                                // Same with the deleted ones.
                                for item in &items_to_remove {
                                    let path = if let TreePathType::File(path) = item { path.to_vec() } else { panic!("This should never happen.") };
                                    if table_state_data.borrow().get(&path).is_some() {
                                        table_state_data.borrow_mut().remove(&path);
                                    }

                                    let data = TableStateData::new_empty();
                                    table_state_data.borrow_mut().insert(path.to_vec(), data);
                                }
                            }
                            
                            Data::Error(error) => show_dialog(app_ui.window, false, error),
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }
                }

                else { show_dialog(app_ui.window, false, ErrorKind::InvalidFilesForMerging); }
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let slot_contextual_menu_delete = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packedfiles_open_in_packedfile_view,
            table_state_data => move |_| {
                
                // Get the currently selected items, and get how many we have of each type.
                let selected_items = get_items_from_main_treeview_selection(&app_ui);

                // First, we prepare the counters for the path types.
                let (mut file, mut folder, mut packfile, mut none) = (0, 0, 0, 0);

                // We need to "clean" the selected path list to ensure we don't pass stuff already deleted.
                let mut item_types_clean = vec![];
                for selected_item_to_add in &selected_items {
                    let item_type_to_add = get_type_of_item(*selected_item_to_add, app_ui.folder_tree_model);
                    match item_type_to_add {
                        TreePathType::File(ref path_to_add) => {
                            let mut add_type = true;
                            for selected_item in &selected_items {
                                let item_type = get_type_of_item(*selected_item, app_ui.folder_tree_model);
                                
                                // Skip the current file from checks.
                                if let TreePathType::File(ref path) = item_type {
                                    if path == path_to_add { continue; }
                                }

                                // If the other one is a folder that contains it, dont add it.
                                else if let TreePathType::Folder(ref path) = item_type {
                                    if path_to_add.starts_with(path) { 
                                        add_type = false;
                                        break;
                                    }
                                }
                            }
                            if add_type { item_types_clean.push(item_type_to_add.clone()); }
                        }

                        TreePathType::Folder(ref path_to_add) => {
                            let mut add_type = true;
                            for selected_item in &selected_items {
                                let item_type = get_type_of_item(*selected_item, app_ui.folder_tree_model);

                                // If the other one is a folder that contains it, dont add it.
                                if let TreePathType::Folder(ref path) = item_type {
                                    if path == path_to_add { continue; }
                                    if path_to_add.starts_with(path) { 
                                        add_type = false;
                                        break;
                                    }
                                }
                            }
                            if add_type { item_types_clean.push(item_type_to_add.clone()); }
                        }

                        // If we got the PackFile, remove everything.
                        TreePathType::PackFile => {
                            item_types_clean.clear();
                            break;
                        }
                        TreePathType::None => unimplemented!(),
                    }   
                }

                for item_type in &item_types_clean {
                    match item_type {
                        TreePathType::File(_) => file += 1,
                        TreePathType::Folder(_) => folder += 1,
                        TreePathType::PackFile => packfile += 1,
                        TreePathType::None => none += 1,
                    }
                }

                // Now we do some bitwise magic to get what type of selection combination we have.
                let mut contents: u8 = 0;
                if file != 0 { contents |= 1; } 
                if folder != 0 { contents |= 2; } 
                if packfile != 0 { contents |= 4; } 
                if none != 0 { contents |= 8; } 
                match contents {

                    // Any combination of files and folders.
                    1 | 2 | 3 => {
                        let packed_files_open = packedfiles_open_in_packedfile_view.borrow().clone();
                        let mut skaven_confirm = false;
                        for item_type in &item_types_clean {
                            match item_type {
                                TreePathType::File(path) => {
                                    for (view, open_path) in &packed_files_open {
                                        if path == &*open_path.borrow() {
                                            if !skaven_confirm { 

                                                let mut dialog = unsafe { MessageBox::new_unsafe((
                                                    message_box::Icon::Information,
                                                    &QString::from_std_str("Warning"),
                                                    &QString::from_std_str("<p>One or more PackedFiles you're trying to delete are currently open.</p><p> Are you sure you want to delete them?</p>"),
                                                    Flags::from_int(4_194_304), // Cancel button.
                                                    app_ui.window as *mut Widget,
                                                )) };

                                                dialog.add_button((&QString::from_std_str("&Accept"), message_box::ButtonRole::AcceptRole));
                                                dialog.set_modal(true);
                                                dialog.show();

                                                // If we hit "Accept", close the PackedFile and continue. Otherwise return.
                                                if dialog.exec() == 0 { 
                                                    purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view);

                                                    let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                                    let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                                    if visible_widgets == 0 { display_help_tips(&app_ui); }
                                                    skaven_confirm = true;
                                                } else { return }
                                            }

                                            if table_state_data.borrow().get(&*open_path.borrow()).is_some() {
                                                table_state_data.borrow_mut().remove(&*open_path.borrow());
                                            }
                                        }
                                    }
                                },

                                TreePathType::Folder(path) => {
                                    for (view, open_path) in &packed_files_open {

                                        // We check here if the Path is already in one of the folders listed for deletion.
                                        let mut path_is_contained_in_deletion = false;
                                        for item_type in &item_types_clean {
                                            if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = item_type {
                                                if !path.is_empty() && open_path.borrow().starts_with(path) {
                                                    path_is_contained_in_deletion = true;
                                                    break;
                                                }
                                            }
                                        }

                                        if open_path.borrow().starts_with(&path) || path_is_contained_in_deletion {
                                            if !skaven_confirm { 

                                                let mut dialog = unsafe { MessageBox::new_unsafe((
                                                    message_box::Icon::Information,
                                                    &QString::from_std_str("Warning"),
                                                    &QString::from_std_str("<p>One or more PackedFiles you're trying to delete are currently open.</p><p> Are you sure you want to delete them?</p>"),
                                                    Flags::from_int(4_194_304), // Cancel button.
                                                    app_ui.window as *mut Widget,
                                                )) };

                                                dialog.add_button((&QString::from_std_str("&Accept"), message_box::ButtonRole::AcceptRole));
                                                dialog.set_modal(true);
                                                dialog.show();

                                                // If we hit "Accept", close the PackedFile and continue. Otherwise return.
                                                if dialog.exec() == 0 { 
                                                    purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view);

                                                    let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                                    let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                                    if visible_widgets == 0 { display_help_tips(&app_ui); }
                                                    skaven_confirm = true;
                                                } else { return }
                                            }

                                            if table_state_data.borrow().get(&*open_path.borrow()).is_some() {
                                                table_state_data.borrow_mut().remove(&*open_path.borrow());
                                            }
                                        }
                                    }
                                },

                                _ => unreachable!(),
                            } 
                        }
                    },

                    // If the PackFile is selected, get it just extract the PackFile and everything will get extracted with it.
                    4 | 5 | 6 | 7 => {
                        // If we have a PackedFile open, throw the usual warning.
                        if !packedfiles_open_in_packedfile_view.borrow().is_empty() {

                            let mut dialog = unsafe { MessageBox::new_unsafe((
                                message_box::Icon::Information,
                                &QString::from_std_str("Warning"),
                                &QString::from_std_str("<p>One or more PackedFiles you're trying to delete are currently open.</p><p> Are you sure you want to delete them?</p>"),
                                Flags::from_int(4_194_304), // Cancel button.
                                app_ui.window as *mut Widget,
                            )) };

                            dialog.add_button((&QString::from_std_str("&Accept"), message_box::ButtonRole::AcceptRole));
                            dialog.set_modal(true);
                            dialog.show();

                            // If we hit "Accept", close all PackedFiles and stop the loop.
                            if dialog.exec() == 0 { 
                                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                                display_help_tips(&app_ui);
                                table_state_data.borrow_mut().clear();
                            } else { return }
                        }
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value. 
                    0 | 8..=255 => return,
                }

                // Tell the Background Thread to delete the selected stuff.
                let items_to_send = item_types_clean.iter().map(|x| From::from(x)).collect::<Vec<PathType>>();
                sender_qt.send(Commands::DeletePackedFile).unwrap();
                sender_qt_data.send(Data::VecPathType(items_to_send)).unwrap();
                match check_message_validity_recv2(&receiver_qt) {
                    Data::VecPathType(path_types) => {

                        // Update the TreeView.
                        let path_types = path_types.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            app_ui.folder_tree_view,
                            Some(app_ui.folder_tree_filter),
                            app_ui.folder_tree_model,
                            TreeViewOperation::Delete(path_types),
                        );

                        // Update the global search stuff, if needed.
                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                    }

                    // This can fail if, for some reason, the command gets resended for one file.
                    Data::Error(error) => { if error.kind() != ErrorKind::Generic { panic!(THREADS_MESSAGE_ERROR); } }
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }
            }
        ));

        // What happens when we trigger the "Extract" action in the Contextual Menu.
        let slot_contextual_menu_extract = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            mode => move |_| {

                // Get the currently selected paths, and get how many we have of each type.
                let selected_items = get_items_from_main_treeview_selection(&app_ui);
                let selected_types = selected_items.iter().map(|x| From::from(&get_type_of_item(*x, app_ui.folder_tree_model))).collect::<Vec<PathType>>();
                let extraction_path = match *mode.borrow() {

                    // If we have a "MyMod" selected, extract everything to the MyMod folder.
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {
                        if let Some(ref mymods_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                            // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                            let mut mod_name = mod_name.to_owned();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(&mod_name);
                            assets_folder
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.window, false, ErrorKind::MyModPathNotConfigured); }
                    }

                    // If we are in "Normal" Mode....
                    Mode::Normal => {

                        // Get the FileChooser dialog to get the path to extract.
                        let extraction_path = unsafe { FileDialog::get_existing_directory_unsafe((
                            app_ui.window as *mut Widget,
                            &QString::from_std_str("Extract PackFile"),
                        )) };
                        
                        if !extraction_path.is_empty() { PathBuf::from(extraction_path.to_std_string()) }
                        else { return }
                    }
                };

                // Tell the Background Thread to delete the selected stuff.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                sender_qt.send(Commands::ExtractPackedFile).unwrap();
                sender_qt_data.send(Data::VecPathTypePathBuf((selected_types, extraction_path))).unwrap();

                // Check what response we got.
                match check_message_validity_tryrecv(&receiver_qt) {
                    Data::String(response) => show_dialog(app_ui.window, true, response),
                    Data::Error(error) => {
                        match error.kind() {
                            ErrorKind::ExtractError(_) | ErrorKind::NonExistantFile => show_dialog(app_ui.window, true, error),
                            ErrorKind::IOFileNotFound | ErrorKind::IOPermissionDenied | ErrorKind::IOGeneric => show_dialog(app_ui.window, true, error),
                            _ => panic!(THREADS_MESSAGE_ERROR)
                        }
                    }
                    _ => panic!(THREADS_MESSAGE_ERROR),
                }

                // Re-enable the Main Window.
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        ));

        // What happens when we trigger the "Open in decoder" action in the Contextual Menu.
        let slot_contextual_menu_open_decoder = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            slots,
            packedfiles_open_in_packedfile_view => move |_| {

                // Get the currently selected paths, and only continue if there is only one.
                let selected_items = get_item_types_from_main_treeview_selection(&app_ui);
                if selected_items.len() == 1 {
                    let item_type = &selected_items[0];

                    // If it's a PackedFile...
                    if let TreePathType::File(path) = item_type {

                        // Remove everything from the PackedFile View.
                        purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                        // We try to open it in the decoder.
                        if let Ok(result) = PackedFileDBDecoder::create_decoder_view(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            &app_ui,
                            &path
                        ) {

                            // Save the monospace font and the slots.
                            slots.borrow_mut().push(TheOneSlot::Decoder(result.0));
                            *monospace_font.borrow_mut() = result.1;
                        }

                        // Disable the "Change game selected" function, so we cannot change the current schema with an open table.
                        unsafe { app_ui.game_selected_group.as_mut().unwrap().set_enabled(false); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Open PackFiles List" action in the Contextual Menu.
        let slot_context_menu_open_dependency_manager = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            global_search_explicit_paths,
            slots,
            packedfiles_open_in_packedfile_view => move |_| {

                // Destroy any children that the PackedFile's View we use may have, cleaning it.
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                // Create the widget that'll act as a container for the view.
                let widget = Widget::new().into_raw();
                let widget_layout = create_grid_layout_unsafe(widget);

                // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                let path = Rc::new(RefCell::new(vec![]));

                // Build the UI and save the slots.
                slots.borrow_mut().push(TheOneSlot::Table(create_dependency_manager_view(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    widget_layout,
                    &path,
                    &global_search_explicit_paths,
                    update_global_search_stuff,
                    &table_state_data
                )));

                // Tell the program there is an open PackedFile.
                purge_that_one_specifically(&app_ui, 0, &packedfiles_open_in_packedfile_view);
                packedfiles_open_in_packedfile_view.borrow_mut().insert(0, path);
                unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget as *mut Widget); }
            }
        ));

        // What happens when we trigger the "Open Containing Folder" action in the Contextual Menu.
        let slot_context_menu_open_containing_folder = SlotBool::new(clone!(
            sender_qt,
            receiver_qt => move |_| {
                sender_qt.send(Commands::OpenContainingFolder).unwrap();
                if let Data::Error(error) = check_message_validity_recv2(&receiver_qt) { show_dialog(app_ui.window, false, error) };
            }
        ));

        // What happens when we trigger the "Open with External Program" action in the Contextual Menu.
        let slot_context_menu_open_with_external_program = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the currently selected paths, and only continue if there is only one.
                let selected_paths = get_path_from_main_treeview_selection(&app_ui);
                if selected_paths.len() == 1 {
                    let path = selected_paths[0].to_vec();

                    // Get the path of the extracted Image.
                    sender_qt.send(Commands::OpenWithExternalProgram).unwrap();
                    sender_qt_data.send(Data::VecString(path.to_vec())).unwrap();
                    if let Data::Error(error) = check_message_validity_recv2(&receiver_qt) { show_dialog(app_ui.window, false, error) };
                }
            }
        ));

        // What happens when we trigger the "Open in Multi-View" action in the Contextual Menu.
        let slot_context_menu_open_in_multi_view = SlotBool::new(clone!(
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            slots,
            table_state_data,
            packedfiles_open_in_packedfile_view => move |_| {

                if let Err(error) = open_packedfile(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packedfiles_open_in_packedfile_view,
                    &global_search_explicit_paths,
                    &slots,
                    update_global_search_stuff,
                    &table_state_data,
                    1
                ) { show_dialog(app_ui.window, false, error); }
            }
        ));

        // What happens when we trigger the "Open in Multi-View" action in the Contextual Menu.
        let slot_context_menu_open_notes = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            slots,
            packedfiles_open_in_packedfile_view => move |_| {

                // Create the widget that'll act as a container for the view.
                let widget = Widget::new().into_raw();
                let widget_layout = create_grid_layout_unsafe(widget);
                
                let path = Rc::new(RefCell::new(vec![]));
                let view_position = 1;

                slots.borrow_mut().push(TheOneSlot::Text(create_notes_view(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    widget_layout,
                    &path,
                    &packedfiles_open_in_packedfile_view
                )));

                // Tell the program there is an open PackedFile and finish the table.
                purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
            }
        ));

        // What happens when we trigger one of the "Filter Updater" events for the Folder TreeView.
        let slot_folder_view_filter_change_text = SlotStringRef::new(move |_| {
            filter_files(&app_ui); 
        });
        let slot_folder_tree_filter_change_autoexpand_matches = SlotBool::new(move |_| {
            filter_files(&app_ui); 
        });
        let slot_folder_view_filter_change_case_sensitive = SlotBool::new(move |_| {
            filter_files(&app_ui); 
        });
        let slot_folder_tree_filter_filter_by_folder_button = SlotBool::new(move |_| {
            filter_files(&app_ui); 
        });

        // Contextual Menu Actions.
        unsafe { app_ui.context_menu_add_file.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_file); }
        unsafe { app_ui.context_menu_add_folder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_folder); }
        unsafe { app_ui.context_menu_add_from_packfile.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_add_from_packfile); }
        unsafe { app_ui.context_menu_check_tables.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_check_tables); }
        unsafe { app_ui.context_menu_create_folder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_folder); }
        unsafe { app_ui.context_menu_create_db.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_db); }
        unsafe { app_ui.context_menu_create_loc.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_loc); }
        unsafe { app_ui.context_menu_create_text.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_create_packed_file_text); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_mass_import_tsv); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_mass_export_tsv); }
        unsafe { app_ui.context_menu_merge_tables.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_merge_tables); }
        unsafe { app_ui.context_menu_delete.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_delete); }
        unsafe { app_ui.context_menu_extract.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_extract); }
        unsafe { app_ui.context_menu_open_decoder.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_open_decoder); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_dependency_manager); }
        unsafe { app_ui.context_menu_open_containing_folder.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_containing_folder); }
        unsafe { app_ui.context_menu_open_with_external_program.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_with_external_program); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_in_multi_view); }
        unsafe { app_ui.context_menu_open_notes.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_notes); }

        // Trigger the filter whenever the "filtered" text changes, the "filtered" column changes or the "Case Sensitive" button changes.
        unsafe { app_ui.folder_tree_filter_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_folder_view_filter_change_text); }
        unsafe { app_ui.folder_tree_filter_autoexpand_matches_button.as_mut().unwrap().signals().toggled().connect(&slot_folder_tree_filter_change_autoexpand_matches); }
        unsafe { app_ui.folder_tree_filter_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slot_folder_view_filter_change_case_sensitive); }
        unsafe { app_ui.folder_tree_filter_filter_by_folder_button.as_mut().unwrap().signals().toggled().connect(&slot_folder_tree_filter_filter_by_folder_button); }

        //-----------------------------------------------------------------------------------------//
        // Rename Action. Due to me not understanding how the edition of a TreeView works, we do it
        // in a special way.
        //-----------------------------------------------------------------------------------------//

        // What happens when we trigger the "Rename" Action.
        let slot_contextual_menu_rename = SlotBool::new(clone!(
            global_search_explicit_paths,
            table_state_data,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {
                
                // Get the currently selected items, and check how many of them are valid before trying to rewrite them.
                // Why? Because I'm sure there is an asshole out there that it's going to try to give the files duplicated
                // names, and if that happen, we have to stop right there that criminal scum.
                let selected_items = get_item_types_from_main_treeview_selection(&app_ui);
                if let Some(rewrite_sequence) = create_rename_dialog(&app_ui, &selected_items) {
                    let mut renaming_data_background: Vec<(PathType, String)> = vec![];
                    for item_type in selected_items {
                        match item_type {
                            TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                                let original_name = path.last().unwrap();
                                let new_name = rewrite_sequence.to_owned().replace("{x}", &original_name).replace("{X}", &original_name);
                                renaming_data_background.push((From::from(&item_type), new_name));

                            },

                            // These two should, if everything works properly, never trigger.
                            TreePathType::PackFile | TreePathType::None => unimplemented!(),
                        }
                    }

                    // Send the renaming data to the Background Thread, wait for a response.
                    sender_qt.send(Commands::RenamePackedFiles).unwrap();
                    sender_qt_data.send(Data::VecPathTypeString(renaming_data_background)).unwrap();
                    match check_message_validity_recv2(&receiver_qt) {

                        // We receive the PathTypes that could be renamed. The rest are ignored.
                        Data::VecPathTypeString(ref renamed_items) => {
                            
                            // Update the TreeView.
                            let renamed_items = renamed_items.iter().map(|x| (From::from(&x.0), x.1.to_owned())).collect::<Vec<(TreePathType, String)>>();
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                &app_ui,
                                app_ui.folder_tree_view,
                                Some(app_ui.folder_tree_filter),
                                app_ui.folder_tree_model,
                                TreeViewOperation::Rename(renamed_items.to_vec()),
                            );

                            // If we have a PackedFile open, we have to rename it in that list too. Note that a path 
                            // can be empty (the dep manager), so we have to check that too.
                            for open_path in packedfiles_open_in_packedfile_view.borrow().values() {
                                if !open_path.borrow().is_empty() { 
                                    for (item_type, new_name) in &renamed_items {
                                        match item_type {
                                            TreePathType::File(ref item_path) => {
                                                if *item_path == *open_path.borrow() {

                                                    // Get the new path.
                                                    let mut new_path = item_path.to_vec();
                                                    *new_path.last_mut().unwrap() = new_name.to_owned();
                                                    *open_path.borrow_mut() = new_path.to_vec();

                                                    // Update the global search stuff, if needed.
                                                    global_search_explicit_paths.borrow_mut().append(&mut vec![new_path; 1]);
                                                }
                                            } 

                                            TreePathType::Folder(ref item_path) => {
                                                if !item_path.is_empty() && open_path.borrow().starts_with(&item_path) {

                                                    let mut new_folder_path = item_path.to_vec();
                                                    *new_folder_path.last_mut().unwrap() = new_name.to_owned();

                                                    let mut new_file_path = new_folder_path.to_vec();
                                                    new_file_path.append(&mut (&open_path.borrow()[item_path.len()..]).to_vec());
                                                    *open_path.borrow_mut() = new_file_path.to_vec();

                                                    // Update the global search stuff, if needed.
                                                    global_search_explicit_paths.borrow_mut().append(&mut vec![new_folder_path; 1]);
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                        
                                        // Same for the TableStateData stuff. If we find one of the paths in it, we remove it and re-insert it with the new name.
                                        match item_type {
                                            TreePathType::File(ref item_path) => {
                                                if table_state_data.borrow().get(item_path).is_some() {
                                                    let mut new_path = item_path.to_vec();
                                                    *new_path.last_mut().unwrap() = new_name.to_owned();
                                                    
                                                    let data = table_state_data.borrow_mut().remove(item_path).unwrap();
                                                    table_state_data.borrow_mut().insert(new_path.to_vec(), data);
                                                }
                                            } 

                                            TreePathType::Folder(ref item_path) => {
                                                let matches = table_state_data.borrow().keys().filter(|x| x.starts_with(item_path) && !x.is_empty()).cloned().collect::<Vec<Vec<String>>>();
                                                for old_path in matches {
                                                    let mut new_path = item_path.to_vec();
                                                    *new_path.last_mut().unwrap() = new_name.to_owned();
                                                    
                                                    let data = table_state_data.borrow_mut().remove(&old_path).unwrap();
                                                    table_state_data.borrow_mut().insert(new_path.to_vec(), data);
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            }
                            unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                        }
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }
                }
            }
        ));

        // Actions to start the Renaming Processes.
        unsafe { app_ui.context_menu_rename.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_rename); }

        //-----------------------------------------------------//
        // Special Actions, like opening a PackedFile...
        //-----------------------------------------------------//

        // What happens when we change the state of an item in the TreeView...
        let slot_paint_treeview = SlotStandardItemMutPtr::new(move |item| {
            paint_specific_item_treeview(item);
        });

        // What happens when we try to open a PackedFile...
        let slot_open_packedfile = Rc::new(SlotNoArgs::new(clone!(
            global_search_explicit_paths,
            slots,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            packedfiles_open_in_packedfile_view => move || {

                if let Err(error) = open_packedfile(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &packedfiles_open_in_packedfile_view,
                    &global_search_explicit_paths,
                    &slots,
                    update_global_search_stuff,
                    &table_state_data,
                    0
                ) { show_dialog(app_ui.window, false, error); }
            }
        )));

        // What happens when we trigger the "Global Search" Action.
        let slot_contextual_menu_global_search = SlotBool::new(clone!(
            global_search_pattern,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the pattern to search, and info related to the search.
                if let Some(pattern) = create_global_search_dialog(&app_ui) {

                    // Start the search in the background thread.
                    sender_qt.send(Commands::GlobalSearch).unwrap();
                    sender_qt_data.send(Data::String(pattern.to_owned())).unwrap();

                    // Create the dialog to show the response.
                    let mut dialog;
                    unsafe { dialog = MessageBox::new_unsafe((
                        message_box::Icon::Information,
                        &QString::from_std_str("Global search"),
                        &QString::from_std_str("<p>Searching in progress... Please wait.</p>"),
                        Flags::from_int(0), // No button.
                        app_ui.window as *mut Widget,
                    )); }

                    // Set it to be modal, and show it. Don't execute it, just show it.
                    dialog.set_modal(true);
                    dialog.set_standard_buttons(Flags::from_int(0));
                    dialog.show();

                    // Get the data from the operation...
                    match check_message_validity_tryrecv(&receiver_qt) {
                        Data::VecGlobalMatch(matches) => {

                            // If there are no matches, just report it.
                            if matches.is_empty() { 
                                dialog.set_standard_buttons(Flags::from_int(2_097_152));
                                dialog.set_text(&QString::from_std_str("<p>No matches found.</p>")); 
                                dialog.exec();
                            }

                            // Otherwise...
                            else {

                                // Show the matches section in the main window and make sure both tables are empty.
                                unsafe { global_search_widget.as_mut().unwrap().show(); }
                                unsafe { model_matches_db.as_mut().unwrap().clear(); }
                                unsafe { model_matches_loc.as_mut().unwrap().clear(); }

                                // For each match, generate an entry in their respective table, 
                                for match_found in &matches {
                                    match match_found {

                                        // In case of Loc PackedFiles...
                                        GlobalMatch::Loc((path, matches)) => {
                                            for match_found in matches.iter() {

                                                // Create a new list of StandardItem.
                                                let mut qlist = ListStandardItemMutPtr::new(());

                                                // Create an empty row.
                                                let clean_path: PathBuf = path.iter().collect();
                                                let clean_path = clean_path.to_string_lossy();
                                                let mut file = StandardItem::new(&QString::from_std_str(clean_path));
                                                let mut column = StandardItem::new(&QString::from_std_str(&match_found.0));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(&format!("{:?}", match_found.1)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.2 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.3));
                                                file.set_editable(false);
                                                column.set_editable(false);
                                                column_number.set_editable(false);
                                                row.set_editable(false);
                                                text.set_editable(false);

                                                // Add an empty row to the list.
                                                unsafe { qlist.append_unsafe(&file.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column.into_raw()); }
                                                unsafe { qlist.append_unsafe(&row.into_raw()); }
                                                unsafe { qlist.append_unsafe(&text.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column_number.into_raw()); }

                                                // Append the new row.
                                                unsafe { model_matches_loc.as_mut().unwrap().append_row(&qlist); }
                                            }
                                        }

                                        // In case of DB Tables...
                                        GlobalMatch::DB((path, matches)) => {
                                            for match_found in matches.iter() {

                                                // Create a new list of StandardItem.
                                                let mut qlist = ListStandardItemMutPtr::new(());

                                                // Create an empty row.
                                                let clean_path: PathBuf = path.iter().collect();
                                                let clean_path = clean_path.to_string_lossy();
                                                let mut file = StandardItem::new(&QString::from_std_str(clean_path));
                                                let mut column = StandardItem::new(&QString::from_std_str(&match_found.0));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(&format!("{:?}", match_found.1)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.2 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.3));
                                                file.set_editable(false);
                                                column.set_editable(false);
                                                column_number.set_editable(false);
                                                row.set_editable(false);
                                                text.set_editable(false);

                                                // Add an empty row to the list.
                                                unsafe { qlist.append_unsafe(&file.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column.into_raw()); }
                                                unsafe { qlist.append_unsafe(&row.into_raw()); }
                                                unsafe { qlist.append_unsafe(&text.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column_number.into_raw()); }

                                                // Append the new row.
                                                unsafe { model_matches_db.as_mut().unwrap().append_row(&qlist); }
                                            }
                                        }
                                    }
                                }

                                // Hide the column number column for tables.
                                unsafe { table_view_matches_db.as_mut().unwrap().hide_column(4); }
                                unsafe { table_view_matches_loc.as_mut().unwrap().hide_column(4); }

                                unsafe { model_matches_db.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile")))); }
                                unsafe { model_matches_db.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column")))); }
                                unsafe { model_matches_db.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row")))); }
                                unsafe { model_matches_db.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Match")))); }

                                unsafe { model_matches_loc.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile")))); }
                                unsafe { model_matches_loc.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column")))); }
                                unsafe { model_matches_loc.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row")))); }
                                unsafe { model_matches_loc.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Match")))); }

                                unsafe { table_view_matches_db.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                                unsafe { table_view_matches_loc.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }

                                unsafe { table_view_matches_db.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }
                                unsafe { table_view_matches_loc.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }
                            }
                        }

                        // If there is an error reading a file, report it.
                        Data::Error(error) => return show_dialog(app_ui.window, false, error),

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Store the pattern for future checks.
                    *global_search_pattern.borrow_mut() = Some(pattern);
                }
            }
        ));

        // What happens when we activate one of the matches in the "Loc Matches" table.
        let slot_load_match_loc = SlotModelIndexRef::new(clone!(
            packedfiles_open_in_packedfile_view,
            slot_open_packedfile => move |model_index_filter| {

                // Map the ModelIndex to his real ModelIndex in the full model.
                let model_index_match = unsafe { filter_model_matches_loc.as_mut().unwrap().map_to_source(&model_index_filter) };

                // Get the data about the PackedFile.
                let path = unsafe { model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 0)).as_mut().unwrap().text().to_std_string() };
                let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();
                let row = unsafe { model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 2)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() - 1 };
                let column = unsafe { model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 4)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() };

                // Expand and select the item in the TreeView.
                let item = get_item_from_type(app_ui.folder_tree_model, &TreePathType::File(path.to_vec()));
                let model_index = unsafe { app_ui.folder_tree_model.as_mut().unwrap().index_from_item(item) };

                let filtered_index = unsafe { app_ui.folder_tree_filter.as_ref().unwrap().map_from_source(&model_index) };
                let selection_model = unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model() };

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if filtered_index.is_valid() {
                    unsafe { selection_model.as_mut().unwrap().select((
                        &filtered_index,
                        Flags::from_enum(SelectionFlag::ClearAndSelect)
                    )); }
                    unsafe { app_ui.folder_tree_view.as_mut().unwrap().scroll_to(&filtered_index); }

                    // Show the PackedFile in the TreeView.
                    expand_treeview_to_item(app_ui.folder_tree_view, app_ui.folder_tree_filter, app_ui.folder_tree_model, &path);

                    // Close any open PackedFile, the open the PackedFile and select the match in it.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                    let action = Action::new(()).into_raw();
                    unsafe { action.as_mut().unwrap().signals().triggered().connect(&*slot_open_packedfile); }
                    unsafe { action.as_mut().unwrap().trigger(); }

                    // Then, select the match and scroll to it.
                    let packed_file_table = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().widget(0).as_mut().unwrap().layout().as_mut().unwrap().item_at(0).as_mut().unwrap().widget() as *mut TableView };
                    let packed_file_model = unsafe { packed_file_table.as_mut().unwrap().model() };
                    let selection_model = unsafe { packed_file_table.as_mut().unwrap().selection_model() };
                    unsafe { selection_model.as_mut().unwrap().select((
                        &packed_file_model.as_mut().unwrap().index((row, column)),
                        Flags::from_enum(SelectionFlag::ClearAndSelect)
                    )); }

                    unsafe { packed_file_table.as_mut().unwrap().scroll_to(&packed_file_model.as_mut().unwrap().index((row, column))); }
                    
                }
                else { show_dialog(app_ui.window, false, ErrorKind::PackedFileNotInFilter); }
            }
        ));

        // What happens when we activate one of the matches in the "DB Matches" table.
        let slot_load_match_db = SlotModelIndexRef::new(clone!(
            packedfiles_open_in_packedfile_view,
            slot_open_packedfile => move |model_index_filter| {

                // Map the ModelIndex to his real ModelIndex in the full model.
                let model_index_match = unsafe { filter_model_matches_db.as_mut().unwrap().map_to_source(&model_index_filter) };

                // Get the data about the PackedFile.
                let path = unsafe { model_matches_db.as_mut().unwrap().item((model_index_match.row(), 0)).as_mut().unwrap().text().to_std_string() };
                let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();
                let row = unsafe { model_matches_db.as_mut().unwrap().item((model_index_match.row(), 2)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() - 1 };
                let column = unsafe { model_matches_db.as_mut().unwrap().item((model_index_match.row(), 4)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() };

                // Expand and select the item in the TreeView.
                let item = get_item_from_type(app_ui.folder_tree_model, &TreePathType::File(path.to_vec()));
                let model_index = unsafe { app_ui.folder_tree_model.as_mut().unwrap().index_from_item(item) };
                
                let filtered_index = unsafe { app_ui.folder_tree_filter.as_ref().unwrap().map_from_source(&model_index) };
                let selection_model = unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model() };

                // If it's not in the current TreeView Filter we CAN'T OPEN IT.
                if filtered_index.is_valid() {
                    unsafe { selection_model.as_mut().unwrap().select((
                        &filtered_index,
                        Flags::from_enum(SelectionFlag::ClearAndSelect)
                    )); }
                    unsafe { app_ui.folder_tree_view.as_mut().unwrap().scroll_to(&filtered_index); }

                    // Show the PackedFile in the TreeView.
                    expand_treeview_to_item(app_ui.folder_tree_view, app_ui.folder_tree_filter, app_ui.folder_tree_model, &path);

                    // Close any open PackedFile, the open the PackedFile.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                    let action = Action::new(()).into_raw();
                    unsafe { action.as_mut().unwrap().signals().triggered().connect(&*slot_open_packedfile); }
                    unsafe { action.as_mut().unwrap().trigger(); }

                    // Then, select the match and scroll to it.
                    let packed_file_table = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().widget(0).as_mut().unwrap().layout().as_mut().unwrap().item_at(0).as_mut().unwrap().widget() as *mut TableView };
                    let packed_file_model = unsafe { packed_file_table.as_mut().unwrap().model() };
                    let selection_model = unsafe { packed_file_table.as_mut().unwrap().selection_model() };
                    unsafe { selection_model.as_mut().unwrap().select((
                        &packed_file_model.as_mut().unwrap().index((row, column)),
                        Flags::from_enum(SelectionFlag::ClearAndSelect)
                    )); }

                    unsafe { packed_file_table.as_mut().unwrap().scroll_to(&packed_file_model.as_mut().unwrap().index((row, column))); }

                }
                else { show_dialog(app_ui.window, false, ErrorKind::PackedFileNotInFilter); }
            }
        ));

        // What happens when we want to update the "Global Search" view.
        let slot_update_global_search_stuff = SlotNoArgs::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            global_search_explicit_paths,
            global_search_pattern => move || {

                // If we have the global search stuff visible and we have a pattern...
                let is_visible = unsafe { global_search_widget.as_ref().unwrap().is_visible() };
                if is_visible {
                    if let Some(ref pattern) = *global_search_pattern.borrow() {

                        // List of all the paths to check.
                        let mut paths = vec![];

                        // For each row in the Loc Table, get his path.
                        let rows = unsafe { model_matches_loc.as_mut().unwrap().row_count(()) };
                        for item in 0..rows {

                            // Get the paths of the PackedFiles to check.
                            let path = unsafe { model_matches_loc.as_mut().unwrap().item((item, 0)).as_mut().unwrap().text().to_std_string() };
                            paths.push(path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect::<Vec<String>>());
                        }

                        // For each row in the DB Table, get his path.
                        let rows = unsafe { model_matches_db.as_mut().unwrap().row_count(()) };
                        for item in 0..rows {

                            // Get the paths of the PackedFiles to check.
                            let path = unsafe { model_matches_db.as_mut().unwrap().item((item, 0)).as_mut().unwrap().text().to_std_string() };
                            paths.push(path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect::<Vec<String>>());
                        }

                        // Add the explicit paths to the list and reset their list.
                        paths.append(&mut global_search_explicit_paths.borrow().to_vec());
                        global_search_explicit_paths.borrow_mut().clear();

                        // Sort the paths and remove duplicates.
                        paths.sort();
                        paths.dedup();

                        // Start the search in the background thread.
                        sender_qt.send(Commands::UpdateGlobalSearchData).unwrap();
                        sender_qt_data.send(Data::StringVecVecString((pattern.to_owned(), paths))).unwrap();

                        // Get the data from the operation...
                        match check_message_validity_tryrecv(&receiver_qt) {
                            Data::VecGlobalMatch(matches) => {

                                unsafe { model_matches_db.as_mut().unwrap().clear(); }
                                unsafe { model_matches_loc.as_mut().unwrap().clear(); }

                                // For each match, generate an entry in their respective table, 
                                for match_found in &matches {
                                    match match_found {

                                        // In case of Loc PackedFiles...
                                        GlobalMatch::Loc((path, matches)) => {
                                            for match_found in matches.iter() {

                                                // Create a new list of StandardItem.
                                                let mut qlist = ListStandardItemMutPtr::new(());

                                                // Create an empty row.
                                                let clean_path: PathBuf = path.iter().collect();
                                                let clean_path = clean_path.to_string_lossy();
                                                let mut file = StandardItem::new(&QString::from_std_str(clean_path));
                                                let mut column = StandardItem::new(&QString::from_std_str(&match_found.0));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(&format!("{:?}", match_found.1)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.2 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.3));
                                                file.set_editable(false);
                                                column.set_editable(false);
                                                column_number.set_editable(false);
                                                row.set_editable(false);
                                                text.set_editable(false);

                                                // Add an empty row to the list.
                                                unsafe { qlist.append_unsafe(&file.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column.into_raw()); }
                                                unsafe { qlist.append_unsafe(&row.into_raw()); }
                                                unsafe { qlist.append_unsafe(&text.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column_number.into_raw()); }

                                                // Append the new row.
                                                unsafe { model_matches_loc.as_mut().unwrap().append_row(&qlist); }
                                            }
                                        }

                                        // In case of DB Tables...
                                        GlobalMatch::DB((path, matches)) => {
                                            for match_found in matches.iter() {

                                                // Create a new list of StandardItem.
                                                let mut qlist = ListStandardItemMutPtr::new(());

                                                // Create an empty row.
                                                let clean_path: PathBuf = path.iter().collect();
                                                let clean_path = clean_path.to_string_lossy();
                                                let mut file = StandardItem::new(&QString::from_std_str(clean_path));
                                                let mut column = StandardItem::new(&QString::from_std_str(&match_found.0));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(&format!("{:?}", match_found.1)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.2 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.3));
                                                file.set_editable(false);
                                                column.set_editable(false);
                                                column_number.set_editable(false);
                                                row.set_editable(false);
                                                text.set_editable(false);

                                                // Add an empty row to the list.
                                                unsafe { qlist.append_unsafe(&file.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column.into_raw()); }
                                                unsafe { qlist.append_unsafe(&row.into_raw()); }
                                                unsafe { qlist.append_unsafe(&text.into_raw()); }
                                                unsafe { qlist.append_unsafe(&column_number.into_raw()); }

                                                // Append the new row.
                                                unsafe { model_matches_db.as_mut().unwrap().append_row(&qlist); }
                                            }
                                        }
                                    }
                                }

                                // Hide the column number column for tables.
                                unsafe { table_view_matches_db.as_mut().unwrap().hide_column(4); }
                                unsafe { table_view_matches_loc.as_mut().unwrap().hide_column(4); }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }

                        // Reconfigure the columns.
                        unsafe { model_matches_db.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile")))); }
                        unsafe { model_matches_db.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column")))); }
                        unsafe { model_matches_db.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row")))); }
                        unsafe { model_matches_db.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Match")))); }

                        unsafe { model_matches_loc.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("PackedFile")))); }
                        unsafe { model_matches_loc.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Column")))); }
                        unsafe { model_matches_loc.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Row")))); }
                        unsafe { model_matches_loc.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Match")))); }

                        unsafe { table_view_matches_db.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                        unsafe { table_view_matches_loc.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }

                        unsafe { table_view_matches_db.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }
                        unsafe { table_view_matches_loc.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }
                    }
                }
            }
        ));

        // What happens when we use the filters to filter search results.
        let slot_matches_filter_db_change_text = SlotStringRef::new(move |filter_text| {
            filter_matches_result(
                Some(QString::from_std_str(filter_text.to_std_string())),
                None,
                None,
                filter_model_matches_db,
                filter_matches_db_line_edit,
                filter_matches_db_column_selector,
                filter_matches_db_case_sensitive_button,
            ); 
        });
        let slot_matches_filter_db_change_column = SlotCInt::new(move |index| {
            filter_matches_result(
                None,
                Some(index),
                None,
                filter_model_matches_db,
                filter_matches_db_line_edit,
                filter_matches_db_column_selector,
                filter_matches_db_case_sensitive_button,
            ); 
        });
        let slot_matches_filter_db_change_case_sensitivity = SlotBool::new(move |case_sensitive| {
            filter_matches_result(
                None,
                None,
                Some(case_sensitive),
                filter_model_matches_db,
                filter_matches_db_line_edit,
                filter_matches_db_column_selector,
                filter_matches_db_case_sensitive_button,
            ); 
        });

        let slot_matches_filter_loc_change_text = SlotStringRef::new(move |filter_text| {
            filter_matches_result(
                Some(QString::from_std_str(filter_text.to_std_string())),
                None,
                None,
                filter_model_matches_loc,
                filter_matches_loc_line_edit,
                filter_matches_loc_column_selector,
                filter_matches_loc_case_sensitive_button,
            ); 
        });
        let slot_matches_filter_loc_change_column = SlotCInt::new(move |index| {
            filter_matches_result(
                None,
                Some(index),
                None,
                filter_model_matches_loc,
                filter_matches_loc_line_edit,
                filter_matches_loc_column_selector,
                filter_matches_loc_case_sensitive_button,
            ); 
        });
        let slot_matches_filter_loc_change_case_sensitivity = SlotBool::new(move |case_sensitive| {
            filter_matches_result(
                None,
                None,
                Some(case_sensitive),
                filter_model_matches_loc,
                filter_matches_loc_line_edit,
                filter_matches_loc_column_selector,
                filter_matches_loc_case_sensitive_button,
            ); 
        });

        // Action to try to open a PackedFile.
        unsafe { app_ui.folder_tree_view.as_ref().unwrap().signals().activated().connect(&*slot_open_packedfile); }

        // In windows "activated" means double click, so we need to add this action too to compensate it.
        if cfg!(target_os = "windows") {
            unsafe { app_ui.folder_tree_view.as_ref().unwrap().signals().clicked().connect(&*slot_open_packedfile); }
        }

        // Action to paint the TreeView.
        unsafe { app_ui.folder_tree_model.as_mut().unwrap().signals().item_changed().connect(&slot_paint_treeview); }
        
        // Global search actions.
        unsafe { app_ui.context_menu_global_search.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_global_search); }
        unsafe { table_view_matches_loc.as_mut().unwrap().signals().double_clicked().connect(&slot_load_match_loc); }
        unsafe { table_view_matches_db.as_mut().unwrap().signals().double_clicked().connect(&slot_load_match_db); }
        unsafe { close_matches_button.as_mut().unwrap().signals().released().connect(&slot_close_global_search); }
        unsafe { update_global_search_stuff.as_mut().unwrap().signals().triggered().connect(&slot_update_global_search_stuff); }

        // Trigger the filter whenever the "filtered" text changes, the "filtered" column changes or the "Case Sensitive" button changes.
        unsafe { filter_matches_db_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_matches_filter_db_change_text); }
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().signals().current_index_changed_c_int().connect(&slot_matches_filter_db_change_column); }
        unsafe { filter_matches_db_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slot_matches_filter_db_change_case_sensitivity); }

        unsafe { filter_matches_loc_line_edit.as_mut().unwrap().signals().text_changed().connect(&slot_matches_filter_loc_change_text); }
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().signals().current_index_changed_c_int().connect(&slot_matches_filter_loc_change_column); }
        unsafe { filter_matches_loc_case_sensitive_button.as_mut().unwrap().signals().toggled().connect(&slot_matches_filter_loc_change_case_sensitivity); }

        //-----------------------------------------------------//
        // Show the Main Window and start everything...
        //-----------------------------------------------------//

        // We need to rebuild the "Open From ..." submenus while opening the PackFile menu if the variable for it is true.
        let slot_rebuild_open_from_submenu = SlotNoArgs::new(clone!(
            mymod_stuff,
            sender_qt,
            packedfiles_open_in_packedfile_view,
            sender_qt_data,
            receiver_qt,
            mode,
            table_state_data,
            close_global_search_action,
            open_from_submenu_menu_needs_rebuild => move || {

                // If we need to rebuild the "MyMod" menu, do it.
                if *open_from_submenu_menu_needs_rebuild.borrow() {
                    *open_from_slots.borrow_mut() = build_open_from_submenus(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        app_ui,
                        menu_open_from_content,
                        menu_open_from_data,
                        &mode,
                        &packedfiles_open_in_packedfile_view,
                        &mymod_stuff,
                        close_global_search_action,
                        &table_state_data,
                    );

                    // Disable the rebuild for the next time.
                    *open_from_submenu_menu_needs_rebuild.borrow_mut() = false;
                }
            }
        ));

        // We need to rebuild the MyMod menu while opening it if the variable for it is true.
        let slot_rebuild_mymod_menu = SlotNoArgs::new(clone!(
            mymod_stuff,
            mymod_stuff_slots,
            sender_qt,
            packedfiles_open_in_packedfile_view,
            sender_qt_data,
            receiver_qt,
            table_state_data,
            mode,
            close_global_search_action,
            mymod_menu_needs_rebuild => move || {

                // If we need to rebuild the "MyMod" menu...
                if *mymod_menu_needs_rebuild.borrow() {

                    // Then rebuild it.
                    let result = build_my_mod_menu(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        app_ui,
                        menu_bar_mymod,
                        &mode,
                        mymod_menu_needs_rebuild.clone(),
                        &packedfiles_open_in_packedfile_view,
                        close_global_search_action,
                        &table_state_data,
                    );

                    // And store the new values.
                    *mymod_stuff.borrow_mut() = result.0;
                    *mymod_stuff_slots.borrow_mut() = result.1;

                    // Disable the rebuild for the next time.
                    *mymod_menu_needs_rebuild.borrow_mut() = false;
                }
            }
        ));
        unsafe { menu_bar_packfile.as_ref().unwrap().signals().about_to_show().connect(&slot_rebuild_open_from_submenu); }
        unsafe { menu_bar_mymod.as_ref().unwrap().signals().about_to_show().connect(&slot_rebuild_mymod_menu); }

        // Show the Main Window...
        unsafe { app_ui.window.as_mut().unwrap().show(); }

        // We get all the Arguments provided when starting RPFM, just in case we passed it a path.
        let arguments = args().collect::<Vec<String>>();

        // If we have an argument (we open RPFM by clicking in a PackFile directly)...
        if arguments.len() > 1 {

            // Turn the fist argument into a Path.
            let path = PathBuf::from(&arguments[1]);

            // If that argument it's a valid File (not Qt-related)...
            if path.is_file() {

                // Try to open it, and report it case of error.
                if let Err(error) = open_packfile(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &[path],
                    &app_ui,
                    &mymod_stuff,
                    &mode,
                    "",
                    &packedfiles_open_in_packedfile_view,
                    close_global_search_action,
                    &table_state_data,
                ) { show_dialog(app_ui.window, false, error); }
            }
        }

        // If we want the window to start maximized...
        if SETTINGS.lock().unwrap().settings_bool["start_maximized"] { unsafe { (app_ui.window as *mut Widget).as_mut().unwrap().set_window_state(Flags::from_enum(WindowState::Maximized)); } }

        // If we want to use the dark theme (Only in windows)...
        if cfg!(target_os = "windows") {
            if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { 
                Application::set_style(&QString::from_std_str("fusion"));
                Application::set_palette(&DARK_PALETTE);
                app.set_style_sheet(&QString::from_std_str(&*DARK_STYLESHEET));
            } else { 
                Application::set_style(&QString::from_std_str("windowsvista"));
                Application::set_palette(&LIGHT_PALETTE);
            }
        }

        // If we have it enabled in the prefs, check if there are updates.
        if SETTINGS.lock().unwrap().settings_bool["check_updates_on_start"] { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if SETTINGS.lock().unwrap().settings_bool["check_schema_updates_on_start"] { check_schema_updates(&app_ui, false, &sender_qt, &sender_qt_data, &receiver_qt) };

        // This is to get the new schemas. It's controlled by a global const. Don't enable this unless you know what you're doing.
        if GENERATE_NEW_SCHEMA {

            // These are the paths needed for the new schemas. First one should be `assembly_kit/raw_data/db`.
            // The second one should contain all the tables of the game, extracted directly from `data.pack`.
            let assembly_kit_schemas_path: PathBuf = PathBuf::from("/home/frodo45127/test stuff/db_raw");
            let testing_tables_path: PathBuf = PathBuf::from("/home/frodo45127/test stuff/db_bin/");
            match import_schema(Schema::load("schema_wh.json").ok(), &assembly_kit_schemas_path, &testing_tables_path) {
                Ok(_) => show_dialog(app_ui.window, true, "Schema successfully created."),
                Err(error) => show_dialog(app_ui.window, false, error),
            }

            // Close the program with code 69
            return 69
        }*/

        // And launch it.
        Application::exec()
    })
}
