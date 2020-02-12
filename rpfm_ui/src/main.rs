//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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
//    clippy::cyclomatic_complexity,          // Disabled due to useless warnings.
    //clippy::doc_markdown,                   // Disabled due to false positives on things that shouldn't be formated in the docs as it says.
    clippy::if_same_then_else,              // Disabled because some of the solutions it provides are freaking hard to read.
    //clippy::match_bool,                     // Disabled because the solutions it provides are harder to read than the current code.
    //clippy::module_inception,               // Disabled because it's quite useless.
    //clippy::needless_bool,                  // Disabled because the solutions it provides are harder to read than the current code.
    //clippy::new_ret_no_self,                // Disabled because the reported situations are special cases. So no, I'm not going to rewrite them.
    //clippy::suspicious_else_formatting,     // Disabled because the errors it gives are actually false positives due to comments.
    //clippy::too_many_arguments,             // Disabled because you never have enough arguments.
    clippy::type_complexity,                // Disabled temporarily because there are other things to do before rewriting the types it warns about.
    clippy::match_wild_err_arm,              // Disabled because, despite being a bad practice, it's the intended behavior in the code it warns about.
    clippy::mut_from_ref,                   // Disabled because it's intended behavior and some false positives.
)]

// This disables the terminal window, so it doesn't show up when executing RPFM in Windows.
#![windows_subsystem = "windows"]

use qt_widgets::application::Application;

use qt_gui::color::Color;
use qt_gui::font::{Font, StyleHint};
use qt_gui::palette::{Palette, ColorGroup, ColorRole};

use lazy_static::lazy_static;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use rpfm_error::ctd::CrashReport;
use rpfm_lib::config::init_config_path;
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::app_ui::AppUI;
use crate::communications::CentralCommand;
use crate::locale::Locale;
use crate::pack_tree::icons::Icons;
use crate::ui::GameSelectedIcons;
use crate::ui_state::UIState;
use self::ui::UI;

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

mod app_ui;
mod global_search_ui;
mod packfile_contents_ui;
mod command_palette;
mod communications;
mod background_thread;
mod ffi;
mod locale;
mod mymod_ui;
mod network_thread;
mod pack_tree;
mod packedfile_views;
mod shortcuts_ui;
mod settings_ui;
mod ui;
mod ui_state;
mod utils;

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
    static ref TREEVIEW_ICONS: Icons = Icons::new();

    /// Icons for the `Game Selected` in the TitleBar.
    static ref GAME_SELECTED_ICONS: GameSelectedIcons = GameSelectedIcons::new();

    /// Bright and dark palettes of colours for Windows.
    /// The dark one is taken from here, with some modifications: https://gist.github.com/QuantumCD/6245215
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
    static ref DARK_STYLESHEET: String = utils::create_dark_theme_stylesheet();

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

    /// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
    static ref LOCALE_FALLBACK: Locale = {
        match Locale::initialize_fallback() {
            Ok(locale) => locale,
            Err(_) => Locale::initialize_empty(),
        }
    };

    /// Variable to keep the locale data used by the UI loaded and available. If we fail to load the selected locale data, copy the english one instead.
    static ref LOCALE: Locale = {
        match SETTINGS.lock().unwrap().settings_string.get("language") {
            Some(language) => Locale::initialize(language).unwrap_or_else(|_| LOCALE_FALLBACK.clone()),
            None => LOCALE_FALLBACK.clone(),
        }
    };

    /// Global variable to hold the sender/receivers used to comunicate between threads.
    static ref CENTRAL_COMMAND: CentralCommand = CentralCommand::default();

    /// Global variable to hold certain info about the current state of the UI.
    static ref UI_STATE: UIState = UIState::default();

    /// Monospace font, just in case we need it.
    static ref FONT_MONOSPACE: Font = {
        let mut font = Font::new(&QString::from_std_str("Monospace"));
        font.set_style_hint(StyleHint::Monospace);
        font
    };
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Custom type to deal with QStrings more easely.
type QString = qt_core::string::String;

/// Main function.
fn main() {

    // Log the crashes so the user can send them himself.
    if !cfg!(debug_assertions) && CrashReport::init().is_err() {
        println!("Failed to initialize logging code.");
    }

    // If the config folder doesn't exist, and we failed to initialize it, force a crash.
    // If this fails, half the program will be broken in one way or another, so better safe than sorry.
    if let Err(error) = init_config_path() { panic!(error); }

    //---------------------------------------------------------------------------------------//
    // Preparing the Program...
    //---------------------------------------------------------------------------------------//

    // Create the background and network threads, where all the magic will happen.
    thread::spawn(move || { background_thread::background_loop(); });
    thread::spawn(move || { network_thread::network_loop(); });

    // Create the application and start the loop.
    Application::create_and_exit(|app| {
        let slot_holder = Rc::new(RefCell::new(vec![]));
        let (_ui, _slots) = UI::new(app, &slot_holder);

        // Dirty fix for the schemas. This has to be changed to a proper fix later.
        *SCHEMA.write().unwrap() = rpfm_lib::schema::Schema::load(&SUPPORTED_GAMES.get("warhammer_2").unwrap().schema).ok();



/*




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


        // What happens when we change the state of an item in the TreeView...
        let slot_paint_treeview = SlotStandardItemMutPtr::new(move |item| {
            paint_specific_item_treeview(item);
        });

*/

        // And launch it.
        Application::exec()
    })
}
