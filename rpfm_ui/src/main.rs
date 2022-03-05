//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
// It also disables a lot of debugging messages on windows, so remember to comment it when needed.
#![windows_subsystem = "windows"]

use qt_widgets::QApplication;
use qt_widgets::QStatusBar;

use qt_gui::QColor;
use qt_gui::QFont;
use qt_gui::{QPalette, q_palette::{ColorGroup, ColorRole}};
use qt_gui::QFontDatabase;
use qt_gui::q_font_database::SystemFont;

use qt_core::QString;

use lazy_static::lazy_static;
use log::info;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::thread;

use rpfm_lib::{SENTRY_GUARD, SETTINGS};

use crate::app_ui::AppUI;
use crate::communications::{CentralCommand, Command, Response};
use crate::locale::Locale;
use crate::pack_tree::icons::Icons;
use crate::ui::GameSelectedIcons;
use crate::ui_state::UIState;
use crate::ui::UI;
use crate::utils::atomic_from_cpp_box;
use crate::utils::atomic_from_q_box;

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
mod dependencies_ui;
mod diagnostics_ui;
mod ffi;
mod global_search_ui;
mod locale;
mod mymod_ui;
mod network_thread;
mod pack_tree;
mod packfile_contents_ui;
mod packedfile_views;
mod shortcuts_ui;
mod settings_ui;
mod tools;
mod ui;
mod ui_state;
mod utils;
mod views;

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

    // Colors used all over the program for theming and stuff.
    static ref MEDIUM_DARK_GREY: &'static str = "#333333";            // Medium-Dark Grey. The color of the background of the Main Window.
    static ref MEDIUM_DARKER_GREY: &'static str = "#262626";          // Medium-Darker Grey.
    static ref DARK_GREY: &'static str = "#181818";                   // Dark Grey. The color of the background of the Main TreeView.
    static ref SLIGHTLY_DARKER_GREY: &'static str = "#101010";        // A Bit Darker Grey.
    static ref KINDA_WHITY_GREY: &'static str = "#BBBBBB";            // Light Grey. The color of the normal Text.
    static ref KINDA_MORE_WHITY_GREY: &'static str = "#CCCCCC";       // Lighter Grey. The color of the highlighted Text.
    static ref EVEN_MORE_WHITY_GREY: &'static str = "#FAFAFA";        // Even Lighter Grey.
    static ref BRIGHT_RED: &'static str = "#FF0000";                  // Bright Red, as our Lord.
    static ref DARK_RED: &'static str = "#FF0000";                    // Dark Red, as our face after facing our enemies.
    static ref LINK_BLUE: &'static str = "#2A82DA";                   // Blue, used for Zeldas.
    static ref ORANGE: &'static str = "#E67E22";                      // Orange, used for borders.
    static ref MEDIUM_GREY: &'static str = "#555555";

    static ref YELLOW_BRIGHT: &'static str = "#FFFFDD";
    static ref YELLOW_MEDIUM: &'static str = "#e5e546";
    static ref YELLOW_DARK: &'static str = "#525200";

    static ref GREEN_BRIGHT: &'static str = "#D0FDCC";
    static ref GREEN_MEDIUM: &'static str = "#87d382";
    static ref GREEN_DARK: &'static str = "#708F6E";

    static ref RED_BRIGHT: &'static str = "#FFCCCC";
    static ref RED_DARK: &'static str = "#8F6E6E";

    static ref BLUE_BRIGHT: &'static str = "#3399ff";
    static ref BLUE_DARK: &'static str = "#0066cc";

    static ref MAGENTA_MEDIUM: &'static str = "#CA1F7B";

    static ref TRANSPARENT_BRIGHT: &'static str = "#00000000";

    static ref ERROR_UNPRESSED_DARK: &'static str = "#b30000";
    static ref ERROR_UNPRESSED_LIGHT: &'static str = "#ffcccc";
    static ref ERROR_PRESSED_DARK: &'static str = "#e60000";
    static ref ERROR_PRESSED_LIGHT: &'static str = "#ff9999";
    static ref ERROR_FOREGROUND_LIGHT: &'static str = "#ff0000";

    static ref WARNING_UNPRESSED_DARK: &'static str = "#4d4d00";
    static ref WARNING_UNPRESSED_LIGHT: &'static str = "#ffffcc";
    static ref WARNING_PRESSED_DARK: &'static str = "#808000";
    static ref WARNING_PRESSED_LIGHT: &'static str = "#ffff99";
    static ref WARNING_FOREGROUND_LIGHT: &'static str = "#B300C0";

    static ref INFO_UNPRESSED_DARK: &'static str = "#0059b3";
    static ref INFO_UNPRESSED_LIGHT: &'static str = "#cce6ff";
    static ref INFO_PRESSED_DARK: &'static str = "#0073e6";
    static ref INFO_PRESSED_LIGHT: &'static str = "#99ccff";

    /// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
    static ref LOCALE_FALLBACK: Locale = {
        match Locale::initialize_fallback() {
            Ok(locale) => locale,
            Err(_) => Locale::initialize_empty(),
        }
    };

    /// Variable to keep the locale data used by the UI loaded and available. If we fail to load the selected locale data, copy the english one instead.
    static ref LOCALE: Locale = {
        match SETTINGS.read().unwrap().settings_string.get("language") {
            Some(language) => Locale::initialize(language).unwrap_or_else(|_| LOCALE_FALLBACK.clone()),
            None => LOCALE_FALLBACK.clone(),
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
const QT_ORG: &str = "FrodoWazEre";
const QT_PROGRAM: &str = "rpfm";

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
