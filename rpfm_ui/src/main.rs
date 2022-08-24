//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here&: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// This is the main file of RPFM. Here is the main loop that builds the UI and controls his events.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::cognitive_complexity,           // Disabled due to useless warnings.
    //clippy::cyclomatic_complexity,          // Disabled due to useless warnings.
    clippy::if_same_then_else,              // Disabled because some of the solutions it provides are freaking hard to read.
    clippy::match_bool,                     // Disabled because the solutions it provides are harder to read than the current code.
    clippy::new_ret_no_self,                // Disabled because the reported situations are special cases. So no, I'm not going to rewrite them.
    clippy::suspicious_else_formatting,     // Disabled because the errors it gives are actually false positives due to comments.
    clippy::match_wild_err_arm,             // Disabled because, despite being a bad practice, it's the intended behavior in the code it warns about.
    clippy::clone_on_copy,                  // Disabled because triggers false positives on qt cloning.
    clippy::mutex_atomic                    // Disabled because in the only instance it triggers, we do it on purpose.
)]

// This disables the terminal window on windows on release builds.
//#[cfg(all(target_os = "windows", not(debug_assertions)))] #[windows_subsystem = "windows"]

use qt_widgets::QApplication;
use qt_widgets::QStatusBar;

use qt_gui::QColor;
use qt_gui::QFont;
use qt_gui::{QPalette, q_palette::{ColorGroup, ColorRole}};
use qt_gui::QFontDatabase;
use qt_gui::q_font_database::SystemFont;

use qt_core::QString;

use lazy_static::lazy_static;

use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, AtomicPtr}, RwLock};
use std::thread;

use rpfm_lib::games::{GameInfo, supported_games::{SupportedGames, KEY_WARHAMMER_3}};
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;

use crate::communications::{CentralCommand, Command, Response};
use crate::locale::Locale;
use crate::pack_tree::icons::Icons;
use crate::settings_ui::backend::*;
use crate::ui::*;
use crate::ui_state::UIState;
use crate::utils::*;

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
    ($($y:ident $n:ident),+ => move || $body:expr) => (
        {
            $( #[allow(unused_mut)] let mut $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
    ($($y:ident $n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( #[allow(unused_mut)] let mut $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

mod app_ui;
mod background_thread;
mod communications;
//mod dependencies_ui;
//mod diagnostics_ui;
mod ffi;
//mod global_search_ui;
mod locale;
//mod mymod_ui;
mod network_thread;
mod pack_tree;
//mod packfile_contents_ui;
//mod packedfile_views;
//mod references_ui;
mod shortcuts_ui;
mod settings_ui;
//mod tools;
mod ui;
mod ui_state;
mod updater;
mod utils;
//mod views;

// Statics, so we don't need to pass them everywhere to use them.
lazy_static! {

    /// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
    /// for "MyMod" folders.
    ///
    /// TODO: Remove this?
    #[derive(Debug)]
    static ref SUPPORTED_GAMES: SupportedGames = SupportedGames::new();

    /// The current GameSelected. If invalid, it uses WH3 as default.
    static ref GAME_SELECTED: Arc<RwLock<&'static GameInfo>> = Arc::new(RwLock::new(
        match SUPPORTED_GAMES.game(&setting_string("default_game")) {
            Some(game) => game,
            None => SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap(),
        }
    ));

    /// Currently loaded schema.
    static ref SCHEMA: Arc<RwLock<Option<Schema>>> = Arc::new(RwLock::new(None));

    /// Sentry client guard, so we can reuse it later on and keep it in scope for the entire duration of the program.
    /// TODO: Fix this path with a settings abstraction.
    static ref SENTRY_GUARD: Arc<RwLock<ClientInitGuard>> = Arc::new(RwLock::new(Logger::init(&PathBuf::from("."), true).unwrap()));

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

    /// Path that contains the extra assets we need, like images.
    #[derive(Debug)]
    static ref ASSETS_PATH: PathBuf = if cfg!(debug_assertions) {
        RPFM_PATH.to_path_buf()
    } else {
        // For release builds:
        // - Windows: Same as RFPM exe.
        // - Linux: /usr/share/rpfm.
        // - MacOs: Who knows?
        if cfg!(target_os = "linux") {
            PathBuf::from("/usr/share/rpfm")
        }
        //if cfg!(target_os = "windows") {
        else {
            RPFM_PATH.to_path_buf()
        }
    };

    /// Icons for the PackFile TreeView.
    static ref TREEVIEW_ICONS: Icons = unsafe { Icons::new() };

    /// Icons for the `Game Selected` in the TitleBar.
    static ref GAME_SELECTED_ICONS: GameSelectedIcons = unsafe { GameSelectedIcons::new() };

    /// Bright and dark palettes of colours for Windows.
    /// The dark one is taken from here, with some modifications: https://gist.github.com/QuantumCD/6245215
    static ref LIGHT_PALETTE: AtomicPtr<QPalette> = unsafe { atomic_from_cpp_box(QPalette::new()) };
    static ref DARK_PALETTE: AtomicPtr<QPalette> = unsafe {{
        let palette = QPalette::new();

        // Base config.
        palette.set_color_2a(ColorRole::Window, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::WindowText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Base, &QColor::from_3_int(34, 34, 34));
        palette.set_color_2a(ColorRole::AlternateBase, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::ToolTipBase, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::ToolTipText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Text, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::Button, &QColor::from_3_int(51, 51, 51));
        palette.set_color_2a(ColorRole::ButtonText, &QColor::from_3_int(187, 187, 187));
        palette.set_color_2a(ColorRole::BrightText, &QColor::from_3_int(255, 0, 0));
        palette.set_color_2a(ColorRole::Link, &QColor::from_3_int(42, 130, 218));
        palette.set_color_2a(ColorRole::Highlight, &QColor::from_3_int(42, 130, 218));
        palette.set_color_2a(ColorRole::HighlightedText, &QColor::from_3_int(204, 204, 204));

        // Disabled config.
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Window, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::WindowText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Base, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::AlternateBase, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ToolTipBase, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ToolTipText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Text, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Button, &QColor::from_3_int(34, 34, 34));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::ButtonText, &QColor::from_3_int(85, 85, 85));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::BrightText, &QColor::from_3_int(170, 0, 0));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Link, &QColor::from_3_int(42, 130, 218));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::Highlight, &QColor::from_3_int(42, 130, 218));
        palette.set_color_3a(ColorGroup::Disabled, ColorRole::HighlightedText, &QColor::from_3_int(85, 85, 85));

        atomic_from_cpp_box(palette)
    }};

    /// Stylesheet used by the dark theme in Windows.
    static ref DARK_STYLESHEET: String = utils::create_dark_theme_stylesheet();

    /// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
    static ref LOCALE_FALLBACK: Locale = {
        match Locale::initialize_fallback() {
            Ok(locale) => locale,
            Err(_) => Locale::initialize_empty(),
        }
    };

    /// Variable to keep the locale data used by the UI loaded and available. If we fail to load the selected locale data, copy the english one instead.
    static ref LOCALE: Locale = {
        let language = setting_string("language");
        if !language.is_empty() {
            Locale::initialize(&language).unwrap_or_else(|_| LOCALE_FALLBACK.clone())
        } else {
            LOCALE_FALLBACK.clone()
        }
    };

    /// Global variable to hold the sender/receivers used to comunicate between threads.
    static ref CENTRAL_COMMAND: CentralCommand<Response> = CentralCommand::default();

    /// Global variable to hold certain info about the current state of the UI.
    static ref UI_STATE: UIState = UIState::default();

    /// Pointer to the status bar of the Main Window, for logging purpouses.
    static ref STATUS_BAR: AtomicPtr<QStatusBar> = unsafe { atomic_from_q_box(QStatusBar::new_0a()) };

    /// Monospace font, just in case we need it.
    static ref FONT_MONOSPACE: AtomicPtr<QFont> = unsafe { atomic_from_cpp_box(QFontDatabase::system_font(SystemFont::FixedFont)) };

    /// Atomic to control if we have performed the initial game selected change or not.
    static ref FIRST_GAME_CHANGE_DONE: AtomicBool = AtomicBool::new(false);
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_SUBTITLE: &str = "I forgot about this message";

/// Main function.
fn main() {

    // Access the guard to make sure it gets initialized.
    if SENTRY_GUARD.read().unwrap().is_enabled() {
        info!("Sentry Logging support enabled. Starting...");
    } else {
        info!("Sentry Logging support disabled. Starting...");
    }

    //---------------------------------------------------------------------------------------//
    // Preparing the Program...
    //---------------------------------------------------------------------------------------//

    // Create the background and network threads, where all the magic will happen.
    info!("Initializing threads...");
    let bac_handle = thread::spawn(|| { background_thread::background_loop(); });
    let net_handle = thread::spawn(|| { network_thread::network_loop(); });

    // Create the application and start the loop.
    QApplication::init(|app| {
        let _ui = unsafe { UI::new(app) };

        // And launch it.
        let exit_code = unsafe { QApplication::exec() };

        // Close and rejoin the threads on exit, so we don't leave a rogue thread running.
        CENTRAL_COMMAND.send_background(Command::Exit);
        CENTRAL_COMMAND.send_network(Command::Exit);

        let _ = bac_handle.join();
        let _ = net_handle.join();

        exit_code
    })
}

