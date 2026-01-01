//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
    clippy::mutex_atomic,                   // Disabled because in the only instance it triggers, we do it on purpose.
    clippy::too_many_arguments,             // Disabled because it gets annoying really quick.
    clippy::assigning_clones,
    clippy::arc_with_non_send_sync,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    clippy::enum_variant_names,
)]

// This disables the terminal window on windows on release builds.
#![windows_subsystem = "windows"]

use qt_widgets::QApplication;
use qt_widgets::QStatusBar;

use qt_gui::QColor;
use qt_gui::QFont;
use qt_gui::QGuiApplication;
use qt_gui::{QPalette, q_palette::{ColorGroup, ColorRole}};
use qt_gui::QFontDatabase;
use qt_gui::q_font_database::SystemFont;

use qt_core::QCoreApplication;
use qt_core::QString;
use qt_core::QVariant;

use tokio::runtime::Runtime;

use std::sync::LazyLock;
use std::process::Command as SystemCommand;
use std::sync::{Arc, atomic::{AtomicBool, AtomicPtr}, RwLock};

use rpfm_lib::games::{GameInfo, supported_games::{SupportedGames, KEY_WARHAMMER_3}};
use rpfm_lib::integrations::log::*;
use rpfm_lib::schema::Schema;

use rpfm_ui_common::APP_NAME;
use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::icons::Icons;
use rpfm_ui_common::locale::Locale;
use rpfm_ui_common::ORG_DOMAIN;
use rpfm_ui_common::ORG_NAME;
use rpfm_ui_common::utils::*;

use crate::communications::{CentralCommand, Command, Response, websocket_loop};
use crate::ui::*;
use crate::ui_state::UIState;

mod app_ui;
mod communications;
mod dependencies_ui;
mod diagnostics_ui;
mod ffi;
mod global_search_ui;
mod mymod_ui;
mod pack_tree;
mod packfile_contents_ui;
mod packedfile_views;
mod references_ui;
mod settings_helpers;
mod settings_ui;
#[cfg(feature = "enable_tools")]mod tools;
mod ui;
mod ui_state;
mod updater_ui;
mod utils;
mod views;

// Statics, so we don't need to pass them everywhere to use them.
/// List of supported games and their configuration. Their key is what we know as `folder_name`, used to identify the game and
/// for "MyMod" folders.
static SUPPORTED_GAMES: LazyLock<SupportedGames> = LazyLock::new(SupportedGames::default);

/// The current GameSelected. If invalid, it uses WH3 as default.
static GAME_SELECTED: LazyLock<Arc<RwLock<&'static GameInfo>>> = LazyLock::new(|| Arc::new(RwLock::new(
    match SUPPORTED_GAMES.game(&settings_helpers::settings_string("default_game")) {
        Some(game) => game,
        None => SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap(),
    }
)));

/// Currently loaded schema.
static SCHEMA: LazyLock<Arc<RwLock<Option<Schema>>>> = LazyLock::new(|| Arc::new(RwLock::new(None)));

/// Icons for the PackFile TreeView.
static TREEVIEW_ICONS: LazyLock<Icons> = LazyLock::new(|| unsafe { Icons::new() });

/// Icons for the `Game Selected` in the TitleBar.
static GAME_SELECTED_ICONS: LazyLock<GameSelectedIcons> = LazyLock::new(|| unsafe { GameSelectedIcons::new() });

/// Light stylesheet.
static LIGHT_STYLE_SHEET: LazyLock<AtomicPtr<QString>> = LazyLock::new(|| unsafe {
    let app = QCoreApplication::instance();
    let qapp = app.static_downcast::<QApplication>();
    atomic_from_cpp_box(qapp.style_sheet())
});

/// Bright and dark palettes of colours for Windows.
/// The dark one is taken from here, with some modifications: https://gist.github.com/QuantumCD/6245215
static LIGHT_PALETTE: LazyLock<AtomicPtr<QPalette>> = LazyLock::new(|| unsafe { atomic_from_cpp_box(QPalette::new()) });
static DARK_PALETTE: LazyLock<AtomicPtr<QPalette>> = LazyLock::new(|| unsafe {{
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
}});

/// Global variable to hold the sender/receivers used to comunicate between threads.
static CENTRAL_COMMAND: LazyLock<Arc<RwLock<CentralCommand<Response>>>> = LazyLock::new(|| Arc::new(RwLock::new(CentralCommand::default())));

/// Global variable to hold certain info about the current state of the UI.
static UI_STATE: LazyLock<UIState> = LazyLock::new(UIState::default);

/// Pointer to the status bar of the Main Window, for logging purpouses.
static STATUS_BAR: LazyLock<AtomicPtr<QStatusBar>> = LazyLock::new(|| unsafe { atomic_from_q_box(QStatusBar::new_0a()) });

/// Monospace font, just in case we need it.
static FONT_MONOSPACE: LazyLock<AtomicPtr<QFont>> = LazyLock::new(|| unsafe { atomic_from_cpp_box(QFontDatabase::system_font(SystemFont::FixedFont)) });

/// Atomic to control if we have performed the initial game selected change or not.
static FIRST_GAME_CHANGE_DONE: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

// QVariants used to speedup certain processes that require a lot of new QVariants of bools.
static QVARIANT_TRUE: LazyLock<AtomicPtr<QVariant>> = LazyLock::new(|| unsafe { atomic_from_cpp_box(QVariant::from_bool(true)) });
static QVARIANT_FALSE: LazyLock<AtomicPtr<QVariant>> = LazyLock::new(|| unsafe { atomic_from_cpp_box(QVariant::from_bool(false)) });

/// This one is for detecting when a file is open for the first time, so we can skip some costly slots.
static NEW_FILE_VIEW_CREATED: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_SUBTITLE: &str = " -- When the translator was finished";

const MANUAL_URL: &str = "https://frodo45127.github.io/rpfm/";
const GITHUB_URL: &str = "https://github.com/Frodo45127/rpfm";
const PATREON_URL: &str = "https://www.patreon.com/RPFM";
const DISCORD_URL: &str = "https://discord.gg/moddingden";

/// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
static LOCALE_FALLBACK: LazyLock<Locale> = LazyLock::new(|| {
    match Locale::initialize_fallback() {
        Ok(locale) => locale,
        Err(_) => Locale::initialize_empty(),
    }
});

/// Variable to keep the locale data used by the UI loaded and available.
static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    // Default to English. Language will be loaded from server settings later.
    Locale::initialize("English_en", &ASSETS_PATH.to_string_lossy()).unwrap_or_else(|_| LOCALE_FALLBACK.clone())
});

/// Main function.
fn main() {
    let tokio_runtime = Runtime::new().unwrap();
    let _tokio_guard = tokio_runtime.enter();

    // This needs to be initialised before anything else.
    unsafe {

        // Settings stuff.
        QCoreApplication::set_organization_domain(&QString::from_std_str("com"));
        QCoreApplication::set_organization_name(&QString::from_std_str("FrodoWazEre"));
        QCoreApplication::set_application_name(&QString::from_std_str("rpfm"));

        *ORG_DOMAIN.write().unwrap() = String::from("com");
        *ORG_NAME.write().unwrap() = String::from("FrodoWazEre");
        *APP_NAME.write().unwrap() = String::from("rpfm");

        // This fixes the app icon on wayland.
        QGuiApplication::set_desktop_file_name(&QString::from_std_str("rpfm"));
    }

    //---------------------------------------------------------------------------------------//
    // Preparing the Program...
    //---------------------------------------------------------------------------------------//

    // Create the background and network threads, where all the magic will happen.
    info!("Initializing WebSocket...");
    spawn_server();
    let (ct, receiver) = CentralCommand::init();
    *CENTRAL_COMMAND.write().unwrap() = ct;
    tokio_runtime.spawn(websocket_loop(receiver));

    // Create the application and start the loop.
    QApplication::init(|_app| {
        let ui = unsafe { UI::new() };
        match ui {
            Ok(ui) => {

                // If we closed the window BEFORE executing, exit the app.
                let exit_code = if unsafe { ui.app_ui.main_window().is_visible() } {
                    unsafe { QApplication::exec() }
                } else { 0 };

                // Close and rejoin the threads on exit, so we don't leave a rogue thread running.
                CENTRAL_COMMAND.read().unwrap().send(Command::Exit);
                exit_code
            }
            Err(error) => {
                error!("{error}");

                // Close and rejoin the threads on exit, so we don't leave a rogue thread running.
                CENTRAL_COMMAND.read().unwrap().send(Command::Exit);
                55
            }
        }
    })
}

/// This function is used to spawn the rpfm_server process if it's not already running.
fn spawn_server() {

    // First, check if the server is already running.
    if std::net::TcpStream::connect_timeout(&"127.0.0.1:3030".parse().unwrap(), std::time::Duration::from_millis(100)).is_ok() {
        info!("rpfm_server already running. Skipping spawn.");
        return;
    }

    if cfg!(debug_assertions) {
        info!("Spawning rpfm_server in debug mode...");
        let _ = SystemCommand::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("rpfm_server")
            .output();

        let _ = SystemCommand::new("target/debug/rpfm_server")
            .spawn();
    } else {
        info!("Spawning rpfm_server in release mode...");
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("rpfm_server");
        let _ = SystemCommand::new(path)
            .spawn();
    }
}
