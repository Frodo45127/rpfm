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
extern crate bitflags;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate failure;
extern crate num;
extern crate chrono;
extern crate regex;

#[macro_use]
extern crate sentry;
extern crate open;
extern crate qt_core;
extern crate qt_gui;
extern crate qt_widgets;
extern crate qt_custom_rpfm;
extern crate cpp_utils;

#[macro_use]
extern crate lazy_static;
extern crate indexmap;

use qt_widgets::abstract_item_view::ScrollMode;
use qt_widgets::action::Action;
use qt_widgets::action_group::ActionGroup;
use qt_widgets::application::Application;
use qt_widgets::combo_box::ComboBox;
use qt_widgets::file_dialog::{AcceptMode, FileDialog, FileMode};
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::layout::Layout;
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
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::flags::Flags;
use qt_core::item_selection_model::SelectionFlag;
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
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::OsStr;
use std::panic;
use std::path::{Path, PathBuf};
use std::fs::{DirBuilder, copy, remove_file, remove_dir_all};

use indexmap::map::IndexMap;
use chrono::NaiveDateTime;
use sentry::integrations::panic::register_panic_handler;

use common::*;
use common::communications::*;
use error::{ErrorKind, logger::Report, Result};
use packedfile::*;
use packedfile::db::schemas_importer::*;
use packfile::packfile::{PFHVersion, PFHFileType, PFHFlags};
use settings::*;
use ui::*;
use ui::dependency_manager::*;
use ui::packedfile_db::*;
use ui::packedfile_loc::*;
use ui::packedfile_text::*;
use ui::packedfile_rigidmodel::*;
use ui::settings::*;
use ui::table_state::*;
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
            id: PFHVersion::PFH5,
            schema: "schema_wh.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(594570),
            ca_types_file: Some("ca_types_wh2".to_owned()),
            supports_editing: true,
        });

        // Warhammer
        map.insert("warhammer", GameInfo {
            display_name: "Warhammer".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_wh.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(364360),
            ca_types_file: None,
            supports_editing: true,
        });

        // Thrones of Britannia
        map.insert("thrones_of_britannia", GameInfo {
            display_name: "Thrones of Britannia".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_tob.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(712100),
            ca_types_file: None,
            supports_editing: true,
        });

        // Attila
        map.insert("attila", GameInfo {
            display_name: "Attila".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_att.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(325610),
            ca_types_file: None,
            supports_editing: true,
        });

        // Rome 2
        map.insert("rome_2", GameInfo {
            display_name: "Rome 2".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_rom2.json".to_owned(),
            db_packs: vec!["data_rome2.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(214950),
            ca_types_file: None,
            supports_editing: true,
        });

        // Shogun 2
        map.insert("shogun_2", GameInfo {
            display_name: "Shogun 2".to_owned(),
            id: PFHVersion::PFH3,
            schema: "schema_sho2.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec!["local_en.pack".to_owned()],
            steam_id: Some(34330),
            ca_types_file: None,
            supports_editing: true,
        });

        // // Napoleon
        // map.insert("napoleon", GameInfo {
        //     display_name: "Napoleon".to_owned(),
        //     id: PFHVersion::PFH2,
        //     schema: "schema_nap.json".to_owned(),
        //     db_pack: "data.pack".to_owned(),
        //     loc_pack: "local_en.pack".to_owned(),
        //     steam_id: Some(34030),
        //     ca_types_file: None,
        //     supports_editing: true,
        // });

        // // Empire
        // map.insert("empire", GameInfo {
        //     display_name: "Empire".to_owned(),
        //     id: PFHVersion::PFH0,
        //     schema: "schema_emp.json".to_owned(),
        //     db_pack: "data.pack".to_owned(),
        //     loc_pack: "local_en.pack".to_owned(),
        //     steam_id: Some(10500),
        //     ca_types_file: None,
        //     supports_editing: true,
        // });

        // NOTE: There are things that depend on the order of this list, and this game must ALWAYS be the last one.
        // Otherwise, stuff that uses this list will probably break.
        // Arena
        map.insert("arena", GameInfo {
            display_name: "Arena".to_owned(),
            id: PFHVersion::PFH5,
            schema: "schema_are.json".to_owned(),
            db_packs: vec!["wad.pack".to_owned()],
            loc_packs: vec!["local_ex.pack".to_owned()],
            steam_id: None,
            ca_types_file: None,
            supports_editing: false,
        });

        map
    };

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

    // Bright and dark palettes of colours for Windows.
    // The dark one is taken from here: https://gist.github.com/QuantumCD/6245215
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
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the DSN needed for Sentry reports to work. Don't change it.
const SENTRY_DSN: &str = "https://a8bf0a98ed43467d841ec433fb3d75a8@sentry.io/1205298";

/// This constant is used to enable or disable the generation of a new Schema file in compile time.
/// If you don't want to explicity create a new Schema for a game, leave this disabled.
const GENERATE_NEW_SCHEMA: bool = false;

/// This constant is used to enable or disable the report of table errors. This is useful for decoding new tables to the schema,
/// as the program will report you in the terminal what tables cannot be decoded. Slow as hell, so never enable it in a release.
const SHOW_TABLE_ERRORS: bool = false;

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

/// This enum represents a match when using the "Global Search" feature.
///  - `DB`: (path, Vec(column_name, column_number, row_number, text).
///  - `Loc`: (path, Vec(column_name, row_number, text)
#[derive(Debug, Clone)]
pub enum GlobalMatch {
    DB((Vec<String>, Vec<(String, i32, i64, String)>)),
    Loc((Vec<String>, Vec<(i32, i64, String)>)),
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
    pub packed_file_splitter: *mut Splitter,

    //-------------------------------------------------------------------------------//
    // "PackFile" menu.
    //-------------------------------------------------------------------------------//

    // Menus.
    pub new_packfile: *mut Action,
    pub open_packfile: *mut Action,
    pub save_packfile: *mut Action,
    pub save_packfile_as: *mut Action,
    pub load_all_ca_packfiles: *mut Action,
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
    pub thrones_of_britannia: *mut Action,
    pub attila: *mut Action,
    pub rome_2: *mut Action,
    pub shogun_2: *mut Action,
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

    // Thrones of Britannia's actions.
    pub tob_optimize_packfile: *mut Action,
    
    // Attila's actions.
    pub att_optimize_packfile: *mut Action,
    
    // Rome 2's actions.
    pub rom2_optimize_packfile: *mut Action,

    // Shogun 2's actions.
    pub sho2_optimize_packfile: *mut Action,

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
    pub context_menu_rename_current: *mut Action,
    pub context_menu_apply_prefix_to_selected: *mut Action,
    pub context_menu_apply_prefix_to_all: *mut Action,
    pub context_menu_open_decoder: *mut Action,
    pub context_menu_open_dependency_manager: *mut Action,
    pub context_menu_open_with_external_program: *mut Action,
    pub context_menu_open_in_multi_view: *mut Action,
    pub context_menu_global_search: *mut Action,

    //-------------------------------------------------------------------------------//
    // "Special" actions for the TreeView.
    //-------------------------------------------------------------------------------//
    pub tree_view_expand_all: *mut Action,
    pub tree_view_collapse_all: *mut Action,
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

    // Sentry fails quite a lot, so log the crashes so the user can send them himself.
    if !cfg!(debug_assertions) { panic::set_hook(Box::new(move |info: &panic::PanicInfo| { Report::new(info).save().unwrap(); })); }

    // Create the application...
    Application::create_and_exit(|app| {

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
        thread::spawn(move || { background_thread::background_loop(sender_rust, receiver_rust, receiver_rust_data); });

        //---------------------------------------------------------------------------------------//
        // Creating the UI...
        //---------------------------------------------------------------------------------------//

        // Set the RPFM Icon.
        let icon = Icon::new(&QString::from_std_str(format!("{}/img/rpfm.png", RPFM_PATH.to_string_lossy())));
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

        // Create the PackedFile splitter.
        let mut packed_file_splitter = Splitter::new(());

        // Create the TreeView.
        let mut folder_tree_view = TreeView::new();
        let mut folder_tree_model = StandardItemModel::new(());
        unsafe { folder_tree_view.set_model(folder_tree_model.static_cast_mut()); }
        folder_tree_view.set_header_hidden(true);
        folder_tree_view.set_animated(true);

        // Create the "Global Search" view.
        let global_search_widget = Widget::new().into_raw();
        let global_search_grid = GridLayout::new().into_raw();
        unsafe { global_search_widget.as_mut().unwrap().set_layout(global_search_grid as *mut Layout); }

        let close_matches_button = PushButton::new(&QString::from_std_str("Close Matches")).into_raw();
        let table_view_matches_db = TableView::new().into_raw();
        let table_view_matches_loc = TableView::new().into_raw();
        let filter_model_matches_db = SortFilterProxyModel::new().into_raw();
        let filter_model_matches_loc = SortFilterProxyModel::new().into_raw();
        let model_matches_db = StandardItemModel::new(()).into_raw();
        let model_matches_loc = StandardItemModel::new(()).into_raw();

        unsafe { filter_model_matches_db.as_mut().unwrap().set_source_model(model_matches_db as *mut AbstractItemModel); }
        unsafe { table_view_matches_db.as_mut().unwrap().set_model(filter_model_matches_db as *mut AbstractItemModel); }
        unsafe { table_view_matches_db.as_mut().unwrap().set_horizontal_scroll_mode(ScrollMode::Pixel); }
        unsafe { table_view_matches_db.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view_matches_db.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view_matches_db.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view_matches_db.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        unsafe { filter_model_matches_loc.as_mut().unwrap().set_source_model(model_matches_loc as *mut AbstractItemModel); }
        unsafe { table_view_matches_loc.as_mut().unwrap().set_model(filter_model_matches_loc as *mut AbstractItemModel); }
        unsafe { table_view_matches_loc.as_mut().unwrap().set_horizontal_scroll_mode(ScrollMode::Pixel); }
        unsafe { table_view_matches_loc.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view_matches_loc.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view_matches_loc.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_visible(true); }
        unsafe { table_view_matches_loc.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }

        // Create the filters for the matches tables.
        let filter_matches_db_line_edit = LineEdit::new(()).into_raw();
        unsafe { filter_matches_db_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        let filter_matches_db_column_selector = ComboBox::new().into_raw();
        let filter_matches_db_column_list = StandardItemModel::new(()).into_raw();
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().set_model(filter_matches_db_column_list as *mut AbstractItemModel); }
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("PackedFile")); }
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Column")); }
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Row")); }
        unsafe { filter_matches_db_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Match")); }

        let filter_matches_db_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { filter_matches_db_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        let filter_matches_loc_line_edit = LineEdit::new(()).into_raw();
        unsafe { filter_matches_loc_line_edit.as_mut().unwrap().set_placeholder_text(&QString::from_std_str("Type here to filter the rows in the table. Works with Regex too!")); }

        let filter_matches_loc_column_selector = ComboBox::new().into_raw();
        let filter_matches_loc_column_list = StandardItemModel::new(()).into_raw();
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().set_model(filter_matches_loc_column_list as *mut AbstractItemModel); }
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("PackedFile")); }
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Column")); }
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Row")); }
        unsafe { filter_matches_loc_column_selector.as_mut().unwrap().add_item(&QString::from_std_str("Match")); }

        let filter_matches_loc_case_sensitive_button = PushButton::new(&QString::from_std_str("Case Sensitive")).into_raw();
        unsafe { filter_matches_loc_case_sensitive_button.as_mut().unwrap().set_checkable(true); }

        // Create the frames for the matches tables.
        let db_matches_frame = GroupBox::new(&QString::from_std_str("DB Matches")).into_raw();
        let db_matches_grid = GridLayout::new().into_raw();
        unsafe { db_matches_frame.as_mut().unwrap().set_layout(db_matches_grid as *mut Layout); }

        let loc_matches_frame = GroupBox::new(&QString::from_std_str("Loc Matches")).into_raw();
        let loc_matches_grid = GridLayout::new().into_raw();
        unsafe { loc_matches_frame.as_mut().unwrap().set_layout(loc_matches_grid as *mut Layout); }

        unsafe { db_matches_grid.as_mut().unwrap().add_widget((table_view_matches_db as *mut Widget, 0, 0, 1, 3)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((table_view_matches_loc as *mut Widget, 0, 0, 1, 3)); }

        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_line_edit as *mut Widget, 1, 0, 1, 1)); }
        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_case_sensitive_button as *mut Widget, 1, 1, 1, 1)); }
        unsafe { db_matches_grid.as_mut().unwrap().add_widget((filter_matches_db_column_selector as *mut Widget, 1, 2, 1, 1)); }
        
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_line_edit as *mut Widget, 1, 0, 1, 1)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_case_sensitive_button as *mut Widget, 1, 1, 1, 1)); }
        unsafe { loc_matches_grid.as_mut().unwrap().add_widget((filter_matches_loc_column_selector as *mut Widget, 1, 2, 1, 1)); }

        unsafe { global_search_grid.as_mut().unwrap().add_widget((db_matches_frame as *mut Widget, 0, 0, 1, 1)); }
        unsafe { global_search_grid.as_mut().unwrap().add_widget((loc_matches_frame as *mut Widget, 1, 0, 1, 1)); }
        unsafe { global_search_grid.as_mut().unwrap().add_widget((close_matches_button as *mut Widget, 2, 0, 1, 1)); }

        // Action to update the search stuff when needed.
        let close_global_search_action = Action::new(()).into_raw();
        let update_global_search_stuff = Action::new(()).into_raw();

        // Add the corresponding widgets to the layout.
        unsafe { central_splitter.add_widget(folder_tree_view.static_cast_mut()); }
        unsafe { central_splitter.add_widget(packed_file_splitter.static_cast_mut() as *mut Widget); }
        unsafe { central_splitter.add_widget(global_search_widget); }

        // Set the correct proportions for the Splitter.
        let mut clist = qt_core::list::ListCInt::new(());
        clist.append(&300);
        clist.append(&1100);
        clist.append(&500);
        central_splitter.set_sizes(&clist);
        central_splitter.set_stretch_factor(0, 0);
        central_splitter.set_stretch_factor(1, 10);
        central_splitter.set_stretch_factor(2, 8);

        // Hide this widget by default.
        unsafe { global_search_widget.as_mut().unwrap().hide(); }

        // MenuBar at the top of the Window.
        let menu_bar = &window.menu_bar();

        // StatusBar at the bottom of the Window.
        let _status_bar = window.status_bar();

        // Top MenuBar menus.
        let menu_bar_packfile = unsafe { menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&PackFile")) };
        let menu_bar_mymod = unsafe { menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&MyMod")) };
        let menu_bar_game_seleted = unsafe { menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&Game Selected")) };
        let menu_bar_special_stuff = unsafe { menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&Special Stuff")) };
        let menu_bar_about = unsafe { menu_bar.as_mut().unwrap().add_menu(&QString::from_std_str("&About")) };
        
        // Submenus.
        let menu_change_packfile_type = Menu::new(&QString::from_std_str("&Change PackFile Type")).into_raw();

        let menu_warhammer_2 = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Warhammer 2")) };
        let menu_warhammer = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("War&hammer")) };
        let menu_thrones_of_britannia = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Thrones of Britannia")) };
        let menu_attila = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Attila")) };
        let menu_rome_2 = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Rome 2")) };
        let menu_shogun_2 = unsafe { menu_bar_special_stuff.as_mut().unwrap().add_menu(&QString::from_std_str("&Shogun 2")) };
        
        // Contextual Menu for the TreeView.
        let mut folder_tree_view_context_menu = Menu::new(());
        let menu_add = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Add..."));
        let menu_create = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Create..."));
        let menu_open = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Open..."));
        let menu_rename = folder_tree_view_context_menu.add_menu(&QString::from_std_str("&Rename..."));

        // Da monsta.
        let app_ui = unsafe { AppUI {

            //-------------------------------------------------------------------------------//
            // Big stuff.
            //-------------------------------------------------------------------------------//
            window: window.into_raw(),
            folder_tree_view: folder_tree_view.into_raw(),
            folder_tree_model: folder_tree_model.into_raw(),
            packed_file_splitter: packed_file_splitter.into_raw(),

            //-------------------------------------------------------------------------------//
            // "PackFile" menu.
            //-------------------------------------------------------------------------------//

            // Menús.
            new_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&New PackFile")),
            open_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Open PackFile")),
            save_packfile: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Save PackFile")),
            save_packfile_as: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("Save PackFile &As...")),
            load_all_ca_packfiles: menu_bar_packfile.as_mut().unwrap().add_action(&QString::from_std_str("&Load All CA PackFiles...")),
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
            thrones_of_britannia: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Thrones of Britannia")),
            attila: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Attila")),
            rome_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("R&ome 2")),
            shogun_2: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("&Shogun 2")),
            arena: menu_bar_game_seleted.as_mut().unwrap().add_action(&QString::from_std_str("A&rena")),

            game_selected_group: ActionGroup::new(menu_bar_game_seleted.as_mut().unwrap().static_cast_mut()).into_raw(),

            //-------------------------------------------------------------------------------//
            // "Special Stuff" menu.
            //-------------------------------------------------------------------------------//

            // Warhammer 2's actions.
            wh2_patch_siege_ai: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
            // wh2_create_prefab: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),
            wh2_create_prefab: Action::new(&QString::from_std_str("&Create Prefab")).into_raw(),
            wh2_optimize_packfile: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

            // Warhammer's actions.
            wh_patch_siege_ai: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
            // wh_create_prefab: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Create Prefab")),
            wh_create_prefab: Action::new(&QString::from_std_str("&Create Prefab")).into_raw(),
            wh_optimize_packfile: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),
            
            // Thrones of Britannia's actions.
            tob_optimize_packfile: menu_thrones_of_britannia.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

            // Attila's actions.
            att_optimize_packfile: menu_attila.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),
            
            // Rome 2's actions.
            rom2_optimize_packfile: menu_rome_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

            // Shogun 2's actions.
            sho2_optimize_packfile: menu_shogun_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

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
            
            context_menu_rename_current: menu_rename.as_mut().unwrap().add_action(&QString::from_std_str("Rename &Current")),
            context_menu_apply_prefix_to_selected: menu_rename.as_mut().unwrap().add_action(&QString::from_std_str("Apply Prefix to &Selected")),
            context_menu_apply_prefix_to_all: menu_rename.as_mut().unwrap().add_action(&QString::from_std_str("Apply Prefix to &All")),

            context_menu_open_decoder: menu_open.as_mut().unwrap().add_action(&QString::from_std_str("&Open with Decoder")),
            context_menu_open_dependency_manager: menu_open.as_mut().unwrap().add_action(&QString::from_std_str("&Open Dependency Manager")),
            context_menu_open_with_external_program: menu_open.as_mut().unwrap().add_action(&QString::from_std_str("&Open with External Program")),
            context_menu_open_in_multi_view: menu_open.as_mut().unwrap().add_action(&QString::from_std_str("&Open in Multi-View")),
            context_menu_global_search: folder_tree_view_context_menu.add_action(&QString::from_std_str("&Global Search")),

            //-------------------------------------------------------------------------------//
            // "Special" Actions for the TreeView.
            //-------------------------------------------------------------------------------//
            tree_view_expand_all: Action::new(&QString::from_std_str("&Expand All")).into_raw(),
            tree_view_collapse_all: Action::new(&QString::from_std_str("&Collapse All")).into_raw(),
        }};

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
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.thrones_of_britannia); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.attila); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.rome_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.shogun_2); }
        unsafe { app_ui.game_selected_group.as_mut().unwrap().add_action_unsafe(app_ui.arena); }
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.attila.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.rome_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.shogun_2.as_mut().unwrap().set_checkable(true); }
        unsafe { app_ui.arena.as_mut().unwrap().set_checkable(true); }

        // Arena is special, so separate it from the rest.
        unsafe { menu_bar_game_seleted.as_mut().unwrap().insert_separator(app_ui.arena); }

        // Put the Submenus and separators in place.
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.load_all_ca_packfiles); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_menu(app_ui.preferences, menu_change_packfile_type); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_separator(app_ui.preferences); }

        // Add the "Open..." submenus. These needs to be here because they have to be appended to the menu.
        let menu_open_from_content = Menu::new(&QString::from_std_str("Open From Content")).into_raw();
        let menu_open_from_data = Menu::new(&QString::from_std_str("Open From Data")).into_raw();
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_menu(app_ui.load_all_ca_packfiles, menu_open_from_content); }
        unsafe { menu_bar_packfile.as_mut().unwrap().insert_menu(app_ui.load_all_ca_packfiles, menu_open_from_data); }
        
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
            match import_schema(&assembly_kit_schemas_path, &testing_tables_path) {
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
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.menu_bar_packfile.get("load_all_ca_packfiles").unwrap()))); }
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
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
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
        let packedfiles_open_in_packedfile_view = Rc::new(RefCell::new(BTreeMap::new()));
        let is_folder_tree_view_locked = Rc::new(RefCell::new(false));
        let mymod_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let open_from_submenu_menu_needs_rebuild = Rc::new(RefCell::new(false));
        let mode = Rc::new(RefCell::new(Mode::Normal));

        // Build the empty structs we need for certain features.
        let add_from_packfile_slots = Rc::new(RefCell::new(AddFromPackFileSlots::new()));
        let packfiles_list_slots = Rc::new(RefCell::new(DependencyTableView::new()));
        let decoder_slots = Rc::new(RefCell::new(PackedFileDBDecoder::new()));
        let db_slots = Rc::new(RefCell::new(BTreeMap::new()));
        let loc_slots = Rc::new(RefCell::new(BTreeMap::new()));
        let text_slots = Rc::new(RefCell::new(BTreeMap::new()));
        let rigid_model_slots = Rc::new(RefCell::new(BTreeMap::new()));

        let monospace_font = Rc::new(RefCell::new(Font::new(&QString::from_std_str("monospace [Consolas]"))));

        // Here we store the pattern for the global search, and paths whose files have been changed/are new and need to be checked.
        let global_search_pattern = Rc::new(RefCell::new(None));
        let global_search_explicit_paths = Rc::new(RefCell::new(vec![]));

        // History for the filters, search, columns...., so table and loc filters are remembered when zapping files, and cleared when the open PackFile changes.
        // NOTE: This affects both DB Tables and Loc PackedFiles.
        let history_state_tables = Rc::new(RefCell::new(TableState::load().unwrap_or_else(|_| TableState::new())));

        // Signal to save the tables states to disk when we're about to close RPFM. We ignore the error here, as at this point we cannot report it to the user.
        let slot_save_states = SlotNoArgs::new(clone!(
            history_state_tables => move || {
                let _y = TableState::save(&history_state_tables.borrow());
            }
        ));
        app.deref_mut().signals().about_to_quit().connect(&slot_save_states);

        // Display the basic tips by default.
        display_help_tips(&app_ui);

        // Build the entire "MyMod" Menu.
        let result = build_my_mod_menu(
            sender_qt.clone(),
            &sender_qt_data,
            receiver_qt.clone(),
            app_ui.clone(),
            &menu_bar_mymod,
            is_modified.clone(),
            mode.clone(),
            mymod_menu_needs_rebuild.clone(),
            &packedfiles_open_in_packedfile_view,
            close_global_search_action,
            &history_state_tables,
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
            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
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
        unsafe { app_ui.context_menu_rename_current.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("rename_current").unwrap()))); }
        unsafe { app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("apply_prefix_to_selected").unwrap()))); }
        unsafe { app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("apply_prefix_to_all").unwrap()))); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("open_in_decoder").unwrap()))); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("open_packfiles_list").unwrap()))); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("open_with_external_program").unwrap()))); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("open_in_multi_view").unwrap()))); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("global_search").unwrap()))); }
        unsafe { app_ui.tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("expand_all").unwrap()))); }
        unsafe { app_ui.tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(shortcuts.tree_view.get("collapse_all").unwrap()))); }

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
        unsafe { app_ui.context_menu_rename_current.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.tree_view_expand_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { app_ui.tree_view_collapse_all.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

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
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_rename_current); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_apply_prefix_to_selected); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_apply_prefix_to_all); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_decoder); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_dependency_manager); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_with_external_program); }
        unsafe { app_ui.folder_tree_view.as_mut().unwrap().add_action(app_ui.context_menu_open_in_multi_view); }
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
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open an existing PackFile.")); }
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

        // Menu bar, Game Selected.
        unsafe { app_ui.warhammer_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer 2' as 'Game Selected'.")); }
        unsafe { app_ui.warhammer.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Warhammer' as 'Game Selected'.")); }
        unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW: Thrones of Britannia' as 'Game Selected'.")); }
        unsafe { app_ui.attila.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Attila' as 'Game Selected'.")); }
        unsafe { app_ui.rome_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Rome 2' as 'Game Selected'.")); }
        unsafe { app_ui.shogun_2.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Shogun 2' as 'Game Selected'.")); }
        unsafe { app_ui.arena.as_mut().unwrap().set_status_tip(&QString::from_std_str("Sets 'TW:Arena' as 'Game Selected'.")); }

        // Menu bar, Special Stuff.
        let patch_siege_ai_tip = QString::from_std_str("Patch & Clean an exported map's PackFile. It fixes the Siege AI (if it has it) and remove useless xml files that bloat the PackFile, reducing his size.");
        let create_prefab_tip = QString::from_std_str("Create prefabs from exported maps. Currently bugged, so don't use it.");
        let optimize_packfile = QString::from_std_str("Check and remove any data in DB Tables and Locs (Locs only for english users) that is unchanged from the base game. That means your mod will only contain the stuff you change, avoiding incompatibilities with other mods.");
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
        unsafe { app_ui.wh2_create_prefab.as_mut().unwrap().set_status_tip(&create_prefab_tip); }
        unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
        unsafe { app_ui.wh_create_prefab.as_mut().unwrap().set_status_tip(&create_prefab_tip); }
        unsafe { app_ui.wh_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }

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
        unsafe { app_ui.context_menu_rename_current.as_mut().unwrap().set_status_tip(&QString::from_std_str("Rename a File/Folder. Remember, whitespaces are NOT ALLOWED.")); }
        unsafe { app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add a Prefix to every File inside the selected folder. Remember, whitespaces are NOT ALLOWED.")); }
        unsafe { app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_status_tip(&QString::from_std_str("Add a Prefix to every File in the PackFile. Remember, whitespaces are NOT ALLOWED.")); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the selected table in the DB Decoder. To create/update schemas.")); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the list of PackFiles referenced from this PackFile.")); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in an external program.")); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_status_tip(&QString::from_std_str("Open the PackedFile in a secondary view, without closing the currently open one.")); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_status_tip(&QString::from_std_str("Performs a search over every DB Table, Loc PackedFile and Text File in the PackFile.")); }

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
                let mut new_game_selected;
                unsafe { new_game_selected = QString::to_std_string(&app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()); }

                // Remove the '&' from the game's name, and turn it into a `folder_name`.
                if let Some(index) = new_game_selected.find('&') { new_game_selected.remove(index); }
                let new_game_selected_folder_name = new_game_selected.replace(' ', "_").to_lowercase();

                // Disable the Main Window (so we can't do other stuff).
                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                // Get the current settings.
                sender_qt.send(Commands::GetSettings).unwrap();
                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Change the Game Selected in the Background Thread.
                sender_qt.send(Commands::SetGameSelected).unwrap();
                sender_qt_data.send(Data::String(new_game_selected_folder_name.to_owned())).unwrap();

                // Prepare to rebuild the submenu next time we try to open the PackFile menu.
                *open_from_submenu_menu_needs_rebuild.borrow_mut() = true;

                // Get the response from the background thread.
                let response = if let Data::StringBool(data) = check_message_validity_tryrecv(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

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
        unsafe { app_ui.thrones_of_britannia.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.attila.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.rome_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.shogun_2.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }
        unsafe { app_ui.arena.as_ref().unwrap().signals().triggered().connect(&slot_change_game_selected); }

        // Try to get the Game Selected. This should never fail, so CTD if it does it.
        sender_qt.send(Commands::GetGameSelected).unwrap();
        let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // Update the "Game Selected" here, so we can skip some steps when initializing.
        match &*game_selected {
            "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); }
            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); }
            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
            "arena" => unsafe { app_ui.arena.as_mut().unwrap().trigger(); }
            "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
            "shogun_2" | _ => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
        }

        //-----------------------------------------------------//
        // "PackFile" Menu...
        //-----------------------------------------------------//

        // What happens when we trigger the "New PackFile" action.
        let slot_new_packfile = SlotBool::new(clone!(
            history_state_tables,
            is_modified,
            mymod_stuff,
            mode,
            packedfiles_open_in_packedfile_view,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

                    // Destroy whatever it's in the PackedFile's view, to avoid data corruption. Also hide the Global Search stuff.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                    // Try to get the settings.
                    sender_qt.send(Commands::GetSettings).unwrap();
                    let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Close the Global Search stuff and reset the filter's history.
                    unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                    if !settings.settings_bool.get("remember_table_state_permanently").unwrap() { history_state_tables.borrow_mut().clear(); }

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
                    let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    enable_packfile_actions(&app_ui, &game_selected, &mymod_stuff, settings, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    set_my_mod_mode(&mymod_stuff, &mode, None);
                }
            }
        ));

        // What happens when we trigger the "Open PackFile" action.
        let slot_open_packfile = SlotBool::new(clone!(
            history_state_tables,
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
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
                    let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Try to get the settings.
                    sender_qt.send(Commands::GetSettings).unwrap();
                    let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // In case we have a default path for the Game Selected, we use it as base path for opening files.
                    if let Some(ref path) = get_game_selected_data_path(&game_selected, &settings) {

                        // We check that actually exists before setting it.
                        if path.is_dir() { file_dialog.set_directory(&QString::from_std_str(&path.to_string_lossy().as_ref().to_owned())); }
                    }

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Get the path of the selected file and turn it in a Rust's PathBuf.
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

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
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &history_state_tables,
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
                    Data::I64(date) => {

                        // Set the mod as "Not Modified".
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                        // Update the "Last Modified Date" of the PackFile in the TreeView.
                        unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(date, 0)))); }
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
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Try to get the Game Selected. This should never fail, so CTD if it does it.
                sender_qt.send(Commands::GetGameSelected).unwrap();
                let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Try to get the settings.
                sender_qt.send(Commands::GetSettings).unwrap();
                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Tell the Background Thread that we want to save the PackFile, and wait for confirmation.
                sender_qt.send(Commands::SavePackFileAs).unwrap();

                // Check what response we got.
                match check_message_validity_recv2(&receiver_qt) {

                    // If we got confirmation....
                    Data::PathBuf(file_path) => {

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
                        file_dialog.select_file(&QString::from_std_str(&file_path.file_name().unwrap().to_string_lossy()));

                        // If we are saving an existing PackFile with another name, we start in his current path.
                        if file_path.is_file() {
                            let mut path = file_path.to_path_buf();
                            path.pop();
                            file_dialog.set_directory(&QString::from_std_str(path.to_string_lossy().as_ref().to_owned()));
                        }

                        // In case we have a default path for the Game Selected and that path is valid,
                        // we use his data folder as base path for saving our PackFile.
                        else if let Some(ref path) = get_game_selected_data_path(&game_selected, &settings) {

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
                                Data::I64(date) => {

                                    // Update the "Last Modified Date" of the PackFile in the TreeView.
                                    unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(date, 0)))); }

                                    // Get the Selection Model and the Model Index of the PackFile's Cell.
                                    let selection_model;
                                    let model_index;
                                    unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                                    unsafe { model_index = app_ui.folder_tree_model.as_ref().unwrap().index((0, 0)); }

                                    // Select the PackFile's Cell with a "Clear & Select".
                                    unsafe { selection_model.as_mut().unwrap().select((&model_index, Flags::from_int(3))); }

                                    // Rename it with the new name.
                                    update_treeview(
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

        // What happens when we trigger the "Load All CA PackFiles" action.
        let slot_load_all_ca_packfiles = SlotBool::new(clone!(
            history_state_tables,
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

                    // Tell the Background Thread to try to load the PackFiles.
                    sender_qt.send(Commands::LoadAllCAPackFiles).unwrap();
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    match check_message_validity_tryrecv(&receiver_qt) {
                    
                        // If it's success....
                        Data::PackFileUIData(_) => {

                            // This PackFile is a special one. It'll always be type "Other(200)" with every special stuff as false.
                            // TODO: Encrypted PackedFiles haven't been tested with this.
                            unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
                            unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                            unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                            // Update the TreeView.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::Build(false),
                            );

                            // Set the new mod as "Not modified".
                            *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                            // Reset the Game Selected, so the UI get's updated properly.
                            sender_qt.send(Commands::GetGameSelected).unwrap();
                            let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                            match &*game_selected {
                                "warhammer_2" => unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); },
                                "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                                "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                                "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                                "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                                "shogun_2" => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                                "arena" => unsafe { app_ui.arena.as_mut().unwrap().trigger(); },
                                _ => unreachable!()
                            }

                            // Set the current "Operational Mode" to `Normal`.
                            set_my_mod_mode(&mymod_stuff, &mode, None);

                            // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                            // Get the current settings.
                            sender_qt.send(Commands::GetSettings).unwrap();
                            let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                        
                            // Close the Global Search stuff and reset the filter's history.
                            unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                            if !settings.settings_bool.get("remember_table_state_permanently").unwrap() { history_state_tables.borrow_mut().clear(); }

                            // Show the "Tips".
                            display_help_tips(&app_ui);
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
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get the currently selected PackFile's Type.
                let packfile_type;
                unsafe { packfile_type = match &*QString::to_std_string(&app_ui.change_packfile_type_group.as_mut().unwrap().checked_action().as_mut().unwrap().text()) {
                    "&Boot" => PFHFileType::Boot,
                    "&Release" => PFHFileType::Release,
                    "&Patch" => PFHFileType::Patch,
                    "&Mod" => PFHFileType::Mod,
                    "Mo&vie" => PFHFileType::Movie,
                    _ => PFHFileType::Other(99),
                }; }

                // Send the type to the Background Thread.
                sender_qt.send(Commands::SetPackFileType).unwrap();
                sender_qt_data.send(Data::PFHFileType(packfile_type)).unwrap();

                // Set the mod as "Modified".
                sender_qt.send(Commands::GetSettings).unwrap();
                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                let use_dark_theme = settings.settings_bool.get("use_dark_theme").unwrap();
                unsafe { *is_modified.borrow_mut() = set_modified(true, &app_ui, Some((vec![app_ui.folder_tree_model.as_ref().unwrap().item(0).as_ref().unwrap().text().to_std_string()], *use_dark_theme))); }
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
                            let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // If our current `GameSelected` is in the `games_with_changed_paths` list...
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
        unsafe { app_ui.load_all_ca_packfiles.as_ref().unwrap().signals().triggered().connect(&slot_load_all_ca_packfiles); }

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
            packedfiles_open_in_packedfile_view,
            receiver_qt,
            sender_qt,
            sender_qt_data => move |_| {

                // This cannot be done if there is a PackedFile open.
                if !packedfiles_open_in_packedfile_view.borrow().is_empty() { return show_dialog(app_ui.window, false, ErrorKind::OperationNotAllowedWithPackedFileOpen); }
            
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

                        // Update the global search stuff, if needed.
                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                    }

                    Data::Error(error) => show_dialog(app_ui.window, false, error),
                    
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
        let slot_open_manual = SlotBool::new(move |_| { 
                let mut manual_path = format!("{:?}", RPFM_PATH.to_path_buf().join(PathBuf::from("rpfm_manual.pdf")));

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
        );

        // What happens when we trigger the "Support me on Patreon" action.
        let slot_patreon_link = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str("https://www.patreon.com/RPFM"))); });

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
                            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(true);
                        }

                        // If it's a DB, we should enable this too.
                        if !data.is_empty() && data.starts_with(&["db".to_owned()]) && data.len() == 3 {
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
                            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
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
                            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(true);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
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
                            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
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
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
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
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            receiver_qt.clone(),
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile.to_vec()),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile);
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile.to_vec()),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile);
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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
            is_modified,
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
                                sender_qt.send(Commands::AddPackedFile).unwrap();
                                sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                                // Disable the Main Window (so we can't do other stuff).
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                                // Get the data from the operation...
                                match check_message_validity_tryrecv(&receiver_qt) {
                                    Data::Success => {

                                        // Update the TreeView.
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            receiver_qt.clone(),
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile.to_vec()),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile);
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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
                            sender_qt.send(Commands::AddPackedFile).unwrap();
                            sender_qt_data.send(Data::VecPathBufVecVecString((paths.to_vec(), paths_packedfile.to_vec()))).unwrap();

                            // Disable the Main Window (so we can't do other stuff).
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                            // Get the data from the operation...
                            match check_message_validity_tryrecv(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile.to_vec()),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile);
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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
            packedfiles_open_in_packedfile_view,
            is_folder_tree_view_locked,
            is_modified,
            add_from_packfile_slots => move |_| {

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
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                            // Build the TreeView to hold all the Extra PackFile's data and save his slots.
                            *add_from_packfile_slots.borrow_mut() = AddFromPackFileSlots::new_with_grid(
                                sender_qt.clone(),
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui,
                                &is_folder_tree_view_locked,
                                &is_modified,
                                &packedfiles_open_in_packedfile_view,
                                &global_search_explicit_paths,
                                update_global_search_stuff
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
            packedfiles_open_in_packedfile_view,
            global_search_explicit_paths,
            is_modified,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Don't do anything if there is a PackedFile open. This fixes the situation where you could overwrite data already in the UI.
                if !packedfiles_open_in_packedfile_view.borrow().is_empty() { return show_dialog(app_ui.window, false, ErrorKind::PackedFileIsOpen) }

                // Create the "Mass-Import TSV" dialog and wait for his data (or a cancelation).
                if let Some(data) = create_mass_import_tsv_dialog(&app_ui) {

                    // If there is no name provided, nor TSV file selected, return an error.
                    if data.0.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::EmptyInput) }
                    else if data.1.is_empty() { return show_dialog(app_ui.window, false, ErrorKind::NoFilesToImport) }

                    // Otherwise, try to import all of them and report the result.
                    else {
                        sender_qt.send(Commands::MassImportTSV).unwrap();
                        sender_qt_data.send(Data::StringVecPathBuf(data)).unwrap();

                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        match check_message_validity_tryrecv(&receiver_qt) {
                            
                            // If it's success....
                            Data::VecVecStringVecVecString(paths) => {

                                // Get the list of paths to add, removing those we "replaced".
                                let mut paths_to_add = paths.1.to_vec();
                                paths_to_add.retain(|x| !paths.0.contains(&x));

                                // Update the TreeView.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    receiver_qt.clone(),
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::Add(paths_to_add.to_vec()),
                                );

                                // Set it as modified. Exception for the paint system.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                // Update the global search stuff, if needed.
                                global_search_explicit_paths.borrow_mut().append(&mut paths_to_add);
                                unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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
                        sender_qt.send(Commands::MassExportTSV).unwrap();
                        sender_qt_data.send(Data::PathBuf(export_path)).unwrap();

                        // Depending on the result, report success or an error.
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
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

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let slot_contextual_menu_delete = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packedfiles_open_in_packedfile_view,
            is_modified => move |_| {

                // Get his Path, including the name of the PackFile.
                let path = get_path_from_selection(&app_ui, true);

                // In case there is nothing selected, don't try to delete.
                if path.is_empty() { return }

                // If we have a PackedFile open...
                let packed_files_open = packedfiles_open_in_packedfile_view.borrow().clone();
                let mut skaven_confirm = false;
                for (view, open_path) in &packed_files_open {

                    // Send the Path to the Background Thread, and get the type of the item.
                    sender_qt.send(Commands::GetTypeOfPath).unwrap();
                    sender_qt_data.send(Data::VecString(path.to_vec())).unwrap();
                    let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // And that PackedFile is the one we want to delete, or it's on the list of paths to delete, ask the user to be sure he wants to delete it.
                    match item_type {
                        TreePathType::File(item_path) => {
                            if *open_path.borrow() == item_path { 

                                let mut dialog = unsafe { MessageBox::new_unsafe((
                                    message_box::Icon::Information,
                                    &QString::from_std_str("Warning"),
                                    &QString::from_std_str("<p>The PackedFile you're trying to delete is currently open.</p><p> Are you sure you want to delete it?</p>"),
                                    Flags::from_int(4194304), // Cancel button.
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
                                } else { return }
                            } 
                        }
                        TreePathType::Folder(item_path) => {
                            if !skaven_confirm {
                                if !item_path.is_empty() && open_path.borrow().starts_with(&item_path) {

                                    let mut dialog = unsafe { MessageBox::new_unsafe((
                                        message_box::Icon::Information,
                                        &QString::from_std_str("Warning"),
                                        &QString::from_std_str("<p>One or more PackedFiles you're trying to delete are currently open.</p><p> Are you sure you want to delete them?</p>"),
                                        Flags::from_int(4194304), // Cancel button.
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
                            }

                            // If we already confirmed it for a PackedFile, don't ask again.
                            else if !item_path.is_empty() && open_path.borrow().starts_with(&item_path) {

                                purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view);

                                let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                                let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                                if visible_widgets == 0 { display_help_tips(&app_ui); }
                            }
                        }
                        TreePathType::PackFile => {
                            let mut dialog = unsafe { MessageBox::new_unsafe((
                                message_box::Icon::Information,
                                &QString::from_std_str("Warning"),
                                &QString::from_std_str("<p>One or more PackedFiles you're trying to delete are currently open.</p><p> Are you sure you want to delete them?</p>"),
                                Flags::from_int(4194304), // Cancel button.
                                app_ui.window as *mut Widget,
                            )) };

                            dialog.add_button((&QString::from_std_str("&Accept"), message_box::ButtonRole::AcceptRole));
                            dialog.set_modal(true);
                            dialog.show();

                            // If we hit "Accept", close all PackedFiles and stop the loop.
                            if dialog.exec() == 0 { 
                                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                                display_help_tips(&app_ui);
                                break;
                            } else { return }
                        }
                        
                        // We use this for the Dependency Manager, in which case we can continue.
                        TreePathType::None => {},
                    }
                }

                // Tell the Background Thread to delete the selected stuff.
                sender_qt.send(Commands::DeletePackedFile).unwrap();
                sender_qt_data.send(Data::VecString(path)).unwrap();

                // Get the response from the other thread.
                match check_message_validity_recv2(&receiver_qt) {

                    // Only if the deletion was successful, we update the UI.
                    Data::TreePathType(path_type) => {

                        // Update the TreeView.
                        update_treeview(
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

                        // Update the global search stuff, if needed.
                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                    }

                    // This can fail if, for some reason, the command gets resended for one file.
                    Data::Error(error) => {
                        if error.kind() != ErrorKind::Generic { panic!(THREADS_MESSAGE_ERROR); }
                    }
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
                            if item_type == TreePathType::File(vec![String::new()]) || item_type == TreePathType::Folder(vec![String::new()]) {

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
                                TreePathType::File(path) | TreePathType::Folder(path) => file_dialog.select_file(&QString::from_std_str(&path.last().unwrap())),

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
            packedfiles_open_in_packedfile_view => move |_| {

                // Get his Path, including the name of the PackFile.
                let path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(path)).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // If it's a PackedFile...
                if let TreePathType::File(path) = item_type {

                    // Remove everything from the PackedFile View.
                    purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                    // We try to open it in the decoder.
                    if let Ok(result) = PackedFileDBDecoder::create_decoder_view(
                        sender_qt.clone(),
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        &path
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

        // What happens when we trigger the "Open PackFiles List" action in the Contextual Menu.
        let slot_context_menu_open_dependency_manager = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packfiles_list_slots,
            is_modified,
            packedfiles_open_in_packedfile_view => move |_| {

                // Destroy any children that the PackedFile's View we use may have, cleaning it.
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                // Build the UI and save the slots.
                *packfiles_list_slots.borrow_mut() = DependencyTableView::create_table_view(
                    sender_qt.clone(),
                    &sender_qt_data,
                    &receiver_qt,
                    &is_modified,
                    &app_ui,
                );

                // Tell the program there is an open PackedFile.
                packedfiles_open_in_packedfile_view.borrow_mut().insert(0, Rc::new(RefCell::new(vec![])));
            }
        ));

        // What happens when we trigger the "Open with External Program" action in the Contextual Menu.
        let slot_context_menu_open_with_external_program = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Get his Path, including the name of the PackFile.
                let path = get_path_from_selection(&app_ui, false);

                // Get the path of the extracted Image.
                sender_qt.send(Commands::OpenWithExternalProgram).unwrap();
                sender_qt_data.send(Data::VecString(path.to_vec())).unwrap();
                if let Data::Error(error) = check_message_validity_recv2(&receiver_qt) { show_dialog(app_ui.window, false, error) };
            }
        ));

        // What happens when we trigger the "Open in Multi-View" action in the Contextual Menu.
        let slot_context_menu_open_in_multi_view = SlotBool::new(clone!(
            history_state_tables,
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            db_slots,
            loc_slots,
            text_slots,
            rigid_model_slots,
            is_folder_tree_view_locked,
            packedfiles_open_in_packedfile_view => move |_| {

                if let Err(error) = open_packedfile(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &is_modified,
                    &packedfiles_open_in_packedfile_view,
                    &global_search_explicit_paths,
                    &is_folder_tree_view_locked,
                    &history_state_tables,
                    &db_slots,
                    &loc_slots,
                    &text_slots,
                    &rigid_model_slots,
                    update_global_search_stuff,
                    1
                ) { show_dialog(app_ui.window, false, error); }
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
        unsafe { app_ui.context_menu_open_dependency_manager.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_dependency_manager); }
        unsafe { app_ui.context_menu_open_with_external_program.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_with_external_program); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_ref().unwrap().signals().triggered().connect(&slot_context_menu_open_in_multi_view); }

        //-----------------------------------------------------------------------------------------//
        // Rename Action. Due to me not understanding how the edition of a TreeView works, we do it
        // in a special way.
        //-----------------------------------------------------------------------------------------//

        // What happens when we trigger the "Rename" Action.
        let slot_contextual_menu_rename_current = SlotBool::new(clone!(
            global_search_explicit_paths,
            is_modified,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Get his Path, including the name of the PackFile.
                let complete_path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(complete_path.to_vec())).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // Depending on the type of the selection...
                match item_type {

                    // If it's a file or a folder...
                    TreePathType::File(ref path) | TreePathType::Folder(ref path) => {

                        // Get the name of the selected item.
                        let current_name = path.last().unwrap();

                        // Create the "Rename" dialog and wait for a new name (or a cancelation).
                        if let Some(new_name) = create_rename_dialog(&app_ui, &current_name) {

                            // Send the New Name to the Background Thread, wait for a response.
                            sender_qt.send(Commands::RenamePackedFile).unwrap();
                            sender_qt_data.send(Data::VecStringString((complete_path, new_name.to_owned()))).unwrap();

                            // Check what response we got.
                            match check_message_validity_recv2(&receiver_qt) {
                                Data::Success => {

                                    // Update the TreeView.
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        receiver_qt.clone(),
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Rename(item_type.clone(), new_name.to_owned()),
                                    );

                                    // Set the mod as "Modified". This is an exception to the paint system.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                    // If we have a PackedFile open, we have to rename it in that list too. Note that a path can be empty (the dep manager), so we have to check that too.
                                    for open_path in packedfiles_open_in_packedfile_view.borrow().values() {
                                        if !open_path.borrow().is_empty() { 
                                            match item_type {
                                                TreePathType::File(ref item_path) => {
                                                    if *item_path == *open_path.borrow() {

                                                        // Get the new path.
                                                        let mut new_path = path.to_vec();
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
                                        }
                                    }

                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
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

        let slot_contextual_menu_apply_prefix_to_selected = SlotBool::new(clone!(
            global_search_explicit_paths,
            is_modified,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Get his Path, including the name of the PackFile.
                let complete_path = get_path_from_selection(&app_ui, true);

                // Send the Path to the Background Thread, and get the type of the item.
                sender_qt.send(Commands::GetTypeOfPath).unwrap();
                sender_qt_data.send(Data::VecString(complete_path)).unwrap();
                let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                // If it's a folder...
                if let TreePathType::Folder(ref path) = item_type {

                    // Create the "Rename" dialog and wait for a prefix (or a cancelation).
                    if let Some(prefix) = create_apply_prefix_to_packed_files_dialog(&app_ui) {

                        // Send the New Name to the Background Thread, wait for a response.
                        sender_qt.send(Commands::ApplyPrefixToPackedFilesInPath).unwrap();
                        sender_qt_data.send(Data::VecStringString((path.to_vec(), prefix.to_owned()))).unwrap();

                        // Check what response we got.
                        match check_message_validity_recv2(&receiver_qt) {
                        
                            // If it's success....
                            Data::VecVecString(old_paths) => {
                                
                                // Update the TreeView.
                                update_treeview(
                                    &sender_qt,
                                    &sender_qt_data,
                                    receiver_qt.clone(),
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::PrefixFiles(old_paths.to_vec(), prefix.to_owned()),
                                );

                                // Set the mod as "Modified". This is an exception to the paint system.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                // If we have a PackedFile open, we have to rename it in that list too. Note that a path can be empty (the dep manager), so we have to check that too.
                                for open_path in packedfiles_open_in_packedfile_view.borrow().values() {
                                    if !open_path.borrow().is_empty() { 
                                        if !path.is_empty() && open_path.borrow().starts_with(&path) {
                                            let mut new_name = format!("{}{}", prefix, *open_path.borrow().last().unwrap());
                                            *open_path.borrow_mut().last_mut().unwrap() = new_name.to_owned();
                                        }
                                    }
                                }

                                // Update the global search stuff, if needed.
                                let mut new_paths = old_paths.to_vec();
                                for path in &mut new_paths {
                                    let mut new_name = format!("{}{}", prefix, *path.last().unwrap());
                                    *path.last_mut().unwrap() = new_name.to_owned();
                                }
                                global_search_explicit_paths.borrow_mut().append(&mut new_paths);
                                unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                            }

                            // If we got an error...
                            Data::Error(error) => {

                                // We must check what kind of error it's.
                                match error.kind() {

                                    // If the new name is empty, contain invalid characters, is already used, or is unchanged, report it.
                                    ErrorKind::EmptyInput |
                                    ErrorKind::InvalidInput => show_dialog(app_ui.window, false, error),

                                    // In ANY other situation, it's a message problem.
                                    _ => panic!(THREADS_MESSAGE_ERROR)
                                }
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!(THREADS_MESSAGE_ERROR),
                        }
                    }
                }
            }
        ));

        let slot_contextual_menu_apply_prefix_to_all = SlotBool::new(clone!(
            global_search_explicit_paths,
            is_modified,
            sender_qt,
            sender_qt_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Create the "Rename" dialog and wait for a prefix (or a cancelation).
                if let Some(prefix) = create_apply_prefix_to_packed_files_dialog(&app_ui) {

                    // Send the New Name to the Background Thread, wait for a response.
                    sender_qt.send(Commands::ApplyPrefixToPackedFilesInPath).unwrap();
                    sender_qt_data.send(Data::VecStringString((vec![], prefix.to_owned()))).unwrap();

                    // Check what response we got.
                    match check_message_validity_recv2(&receiver_qt) {
                    
                        // If it's success....
                        Data::VecVecString(old_paths) => {
                            
                            // Update the TreeView.
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                receiver_qt.clone(),
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::PrefixFiles(old_paths.to_vec(), prefix.to_owned()),
                            );

                            // Set the mod as "Modified". This is an exception to the paint system.
                            *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                            // If we have a PackedFile open, we have to rename it in that list too. Note that a path can be empty (the dep manager), so we have to check that too.
                            for open_path in packedfiles_open_in_packedfile_view.borrow().values() {
                                if !open_path.borrow().is_empty() {
                                    let mut new_name = format!("{}{}", prefix, *open_path.borrow().last().unwrap());
                                    *open_path.borrow_mut().last_mut().unwrap() = new_name.to_owned();
                                }
                            }

                            // Update the global search stuff, if needed.
                            let mut new_paths = old_paths.to_vec();
                            for path in &mut new_paths {
                                let mut new_name = format!("{}{}", prefix, *path.last().unwrap());
                                *path.last_mut().unwrap() = new_name.to_owned();
                            }
                            global_search_explicit_paths.borrow_mut().append(&mut new_paths);
                            unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }
                        }

                        // If we got an error...
                        Data::Error(error) => {

                            // We must check what kind of error it's.
                            match error.kind() {

                                // If the new name is empty, contain invalid characters, is already used, or is unchanged, report it.
                                ErrorKind::EmptyInput |
                                ErrorKind::InvalidInput => show_dialog(app_ui.window, false, error),

                                // In ANY other situation, it's a message problem.
                                _ => panic!(THREADS_MESSAGE_ERROR)
                            }
                        }

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }
                }
            }
        ));

        // Actions to start the Renaming Processes.
        unsafe { app_ui.context_menu_rename_current.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_rename_current); }
        unsafe { app_ui.context_menu_apply_prefix_to_selected.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_apply_prefix_to_selected); }
        unsafe { app_ui.context_menu_apply_prefix_to_all.as_ref().unwrap().signals().triggered().connect(&slot_contextual_menu_apply_prefix_to_all); }

        //-----------------------------------------------------//
        // Special Actions, like opening a PackedFile...
        //-----------------------------------------------------//

        // What happens when we try to open a PackedFile...
        let slot_open_packedfile = Rc::new(SlotNoArgs::new(clone!(
            history_state_tables,
            global_search_explicit_paths,
            db_slots,
            loc_slots,
            text_slots,
            rigid_model_slots,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            is_folder_tree_view_locked,
            packedfiles_open_in_packedfile_view => move || {

                if let Err(error) = open_packedfile(
                    &sender_qt,
                    &sender_qt_data,
                    &receiver_qt,
                    &app_ui,
                    &is_modified,
                    &packedfiles_open_in_packedfile_view,
                    &global_search_explicit_paths,
                    &is_folder_tree_view_locked,
                    &history_state_tables,
                    &db_slots,
                    &loc_slots,
                    &text_slots,
                    &rigid_model_slots,
                    update_global_search_stuff,
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
                        Flags::from_int(2097152), // Close button.
                        app_ui.window as *mut Widget,
                    )); }

                    // Set it to be modal, and show it. Don't execute it, just show it.
                    dialog.set_modal(true);
                    dialog.show();

                    // Get the data from the operation...
                    match check_message_validity_tryrecv(&receiver_qt) {
                        Data::VecGlobalMatch(matches) => {

                            // If there are no matches, just report it.
                            if matches.is_empty() { 
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
                                                let mut column = StandardItem::new(&QString::from_std_str(if match_found.0 == 0 { "Key" } else { "Text" }));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(&format!("{:?}", match_found.0)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.1 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.2));
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
                let model_index_match;
                unsafe { model_index_match = filter_model_matches_loc.as_mut().unwrap().map_to_source(&model_index_filter); }

                // Get the data about the PackedFile.
                let path;
                let row;
                let column;
                unsafe { path = model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 0)).as_mut().unwrap().text().to_std_string(); }
                let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();
                unsafe { row = model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 2)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() - 1; }
                unsafe { column = model_matches_loc.as_mut().unwrap().item((model_index_match.row(), 4)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap();; }

                // Expand and select the item in the TreeView.
                let item = get_item_from_incomplete_path(app_ui.folder_tree_model, &path);
                let model_index;
                unsafe { model_index = app_ui.folder_tree_model.as_mut().unwrap().index_from_item(item); }

                let selection_model;
                unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                unsafe { selection_model.as_mut().unwrap().select((
                    &model_index,
                    Flags::from_enum(SelectionFlag::ClearAndSelect)
                )); }

                // Show the PackedFile in the TreeView.
                expand_treeview_to_item(app_ui.folder_tree_view, app_ui.folder_tree_model, &path);
                unsafe { app_ui.folder_tree_view.as_mut().unwrap().scroll_to(&model_index); }
                
                // Close any open PackedFile, the open the PackedFile and select the match in it.
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                let action = Action::new(()).into_raw();
                unsafe { action.as_mut().unwrap().signals().triggered().connect(&*slot_open_packedfile); }
                unsafe { action.as_mut().unwrap().trigger(); }

                let packed_file_table;
                let packed_file_model;
                unsafe { packed_file_table = app_ui.packed_file_splitter.as_mut().unwrap().widget(0).as_mut().unwrap().layout().as_mut().unwrap().item_at(0).as_mut().unwrap().widget() as *mut TableView; }
                unsafe { packed_file_model = packed_file_table.as_mut().unwrap().model(); }
                let selection_model;
                unsafe { selection_model = packed_file_table.as_mut().unwrap().selection_model(); }
                unsafe { selection_model.as_mut().unwrap().select((
                    &packed_file_model.as_mut().unwrap().index((row, column)),
                    Flags::from_enum(SelectionFlag::ClearAndSelect)
                )); }

                unsafe { packed_file_table.as_mut().unwrap().scroll_to(&packed_file_model.as_mut().unwrap().index((row, column))); }
            }
        ));

        // What happens when we activate one of the matches in the "DB Matches" table.
        let slot_load_match_db = SlotModelIndexRef::new(clone!(
            packedfiles_open_in_packedfile_view,
            slot_open_packedfile => move |model_index_filter| {

                // Map the ModelIndex to his real ModelIndex in the full model.
                let model_index_match;
                unsafe { model_index_match = filter_model_matches_db.as_mut().unwrap().map_to_source(&model_index_filter); }

                // Get the data about the PackedFile.
                let path;
                let row;
                let column;
                unsafe { path = model_matches_db.as_mut().unwrap().item((model_index_match.row(), 0)).as_mut().unwrap().text().to_std_string(); }
                let path: Vec<String> = path.split(|x| x == '/' || x == '\\').map(|x| x.to_owned()).collect();
                unsafe { row = model_matches_db.as_mut().unwrap().item((model_index_match.row(), 2)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap() - 1; }
                unsafe { column = model_matches_db.as_mut().unwrap().item((model_index_match.row(), 4)).as_mut().unwrap().text().to_std_string().parse::<i32>().unwrap(); }

                // Expand and select the item in the TreeView.
                let item = get_item_from_incomplete_path(app_ui.folder_tree_model, &path);
                let model_index;
                unsafe { model_index = app_ui.folder_tree_model.as_mut().unwrap().index_from_item(item); }

                let selection_model;
                unsafe { selection_model = app_ui.folder_tree_view.as_mut().unwrap().selection_model(); }
                unsafe { selection_model.as_mut().unwrap().select((
                    &model_index,
                    Flags::from_enum(SelectionFlag::ClearAndSelect)
                )); }

                // Show the PackedFile in the TreeView.
                expand_treeview_to_item(app_ui.folder_tree_view, app_ui.folder_tree_model, &path);
                unsafe { app_ui.folder_tree_view.as_mut().unwrap().scroll_to(&model_index); }
                           
                // Close any open PackedFile, the open the PackedFile and select the match in it.
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                let action = Action::new(()).into_raw();
                unsafe { action.as_mut().unwrap().signals().triggered().connect(&*slot_open_packedfile); }
                unsafe { action.as_mut().unwrap().trigger(); }

                let packed_file_table;
                let packed_file_model;
                unsafe { packed_file_table = app_ui.packed_file_splitter.as_mut().unwrap().widget(0).as_mut().unwrap().layout().as_mut().unwrap().item_at(0).as_mut().unwrap().widget() as *mut TableView; }
                unsafe { packed_file_model = packed_file_table.as_mut().unwrap().model(); }
                let selection_model;
                unsafe { selection_model = packed_file_table.as_mut().unwrap().selection_model(); }
                unsafe { selection_model.as_mut().unwrap().select((
                    &packed_file_model.as_mut().unwrap().index((row, column)),
                    Flags::from_enum(SelectionFlag::ClearAndSelect)
                )); }

                unsafe { packed_file_table.as_mut().unwrap().scroll_to(&packed_file_model.as_mut().unwrap().index((row, column))); }
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
                                                let mut column = StandardItem::new(&QString::from_std_str(if match_found.0 == 0 { "Key" } else { "Text" }));
                                                let mut column_number = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.0)));
                                                let mut row = StandardItem::new(&QString::from_std_str(format!("{:?}", match_found.1 + 1)));
                                                let mut text = StandardItem::new(&QString::from_std_str(&match_found.2));
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
            history_state_tables,
            mymod_stuff,
            sender_qt,
            packedfiles_open_in_packedfile_view,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            close_global_search_action,
            open_from_submenu_menu_needs_rebuild => move || {

                // If we need to rebuild the "MyMod" menu...
                if *open_from_submenu_menu_needs_rebuild.borrow() {

                    // Get the current settings.
                    sender_qt.send(Commands::GetSettings).unwrap();
                    let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Change the Game Selected in the Background Thread.
                    sender_qt.send(Commands::GetGameSelected).unwrap();
                    let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                    // Then rebuild it.
                    *open_from_slots.borrow_mut() = build_open_from_submenus(
                        sender_qt.clone(),
                        &sender_qt_data,
                        receiver_qt.clone(),
                        &settings,
                        app_ui,
                        &menu_open_from_content,
                        &menu_open_from_data,
                        &game_selected,
                        &is_modified,
                        &mode,
                        &packedfiles_open_in_packedfile_view,
                        &mymod_stuff,
                        close_global_search_action,
                        &history_state_tables,
                    );

                    // Disable the rebuild for the next time.
                    *open_from_submenu_menu_needs_rebuild.borrow_mut() = false;
                }
            }
        ));

        // We need to rebuild the MyMod menu while opening it if the variable for it is true.
        let slot_rebuild_mymod_menu = SlotNoArgs::new(clone!(
            history_state_tables,
            mymod_stuff,
            mymod_stuff_slots,
            sender_qt,
            packedfiles_open_in_packedfile_view,
            sender_qt_data,
            receiver_qt,
            is_modified,
            mode,
            close_global_search_action,
            mymod_menu_needs_rebuild => move || {

                // If we need to rebuild the "MyMod" menu...
                if *mymod_menu_needs_rebuild.borrow() {

                    // Then rebuild it.
                    let result = build_my_mod_menu(
                        sender_qt.clone(),
                        &sender_qt_data,
                        receiver_qt.clone(),
                        app_ui.clone(),
                        &menu_bar_mymod,
                        is_modified.clone(),
                        mode.clone(),
                        mymod_menu_needs_rebuild.clone(),
                        &packedfiles_open_in_packedfile_view,
                        close_global_search_action,
                        &history_state_tables,
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
                    path,
                    &app_ui,
                    &mymod_stuff,
                    &is_modified,
                    &mode,
                    "",
                    &packedfiles_open_in_packedfile_view,
                    close_global_search_action,
                    &history_state_tables,
                ) { show_dialog(app_ui.window, false, error); }
            }
        }

        // Get the settings.
        sender_qt.send(Commands::GetSettings).unwrap();
        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // If we want the window to start maximized...
        if *settings.settings_bool.get("start_maximized").unwrap() { unsafe { (app_ui.window as *mut Widget).as_mut().unwrap().set_window_state(Flags::from_enum(WindowState::Maximized)); } }

        // If we want to use the dark theme (Only in windows)...
        if cfg!(target_os = "windows") {
            if *settings.settings_bool.get("use_dark_theme").unwrap() { 
                Application::set_style(&QString::from_std_str("fusion"));
                Application::set_palette(&DARK_PALETTE); 
            } else { 
                Application::set_style(&QString::from_std_str("windowsvista"));
                Application::set_palette(&LIGHT_PALETTE);
            }
        }

        // If we have it enabled in the prefs, check if there are updates.
        if *settings.settings_bool.get("check_updates_on_start").unwrap() { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if *settings.settings_bool.get("check_schema_updates_on_start").unwrap() { check_schema_updates(&app_ui, false, &sender_qt, &sender_qt_data, &receiver_qt) };

        // And launch it.
        Application::exec()
    })
}

/// This function enables or disables the actions from the `MenuBar` needed when we open a PackFile.
/// NOTE: To disable the "Special Stuff" actions, we use `enable` => false.
fn enable_packfile_actions(
    app_ui: &AppUI,
    game_selected: &str,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    settings: Settings,
    enable: bool
) {

    // If the game is Arena, no matter what we're doing, these ones ALWAYS have to be disabled.
    if game_selected == "arena" {

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
        match game_selected {
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
            "thrones_of_britannia" => {
                unsafe { app_ui.tob_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "attila" => {
                unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "rome_2" => {
                unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
            },
            "shogun_2" => {
                unsafe { app_ui.sho2_optimize_packfile.as_mut().unwrap().set_enabled(true); }
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

        // Disable Thrones of Britannia actions...
        unsafe { app_ui.tob_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Attila actions...
        unsafe { app_ui.att_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Rome 2 actions...
        unsafe { app_ui.rom2_optimize_packfile.as_mut().unwrap().set_enabled(false); }

        // Disable Shogun 2 actions...
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
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    pack_file_path: PathBuf,
    app_ui: &AppUI,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    is_modified: &Rc<RefCell<bool>>,
    mode: &Rc<RefCell<Mode>>,
    game_folder: &str,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    close_global_search_action: *mut Action,
    history_state_tables: &Rc<RefCell<BTreeMap<Vec<String>, TableState>>>,
) -> Result<()> {

    // Tell the Background Thread to create a new PackFile.
    sender_qt.send(Commands::OpenPackFile).unwrap();
    sender_qt_data.send(Data::PathBuf(pack_file_path.to_path_buf())).unwrap();

    // Disable the Main Window (so we can't do other stuff).
    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

    // Check what response we got.
    match check_message_validity_tryrecv(&receiver_qt) {
    
        // If it's success....
        Data::PackFileUIData(ui_data) => {

            // We choose the right option, depending on our PackFile.
            match ui_data.pfh_file_type {
                PFHFileType::Boot => unsafe { app_ui.change_packfile_type_boot.as_mut().unwrap().set_checked(true); }
                PFHFileType::Release => unsafe { app_ui.change_packfile_type_release.as_mut().unwrap().set_checked(true); }
                PFHFileType::Patch => unsafe { app_ui.change_packfile_type_patch.as_mut().unwrap().set_checked(true); }
                PFHFileType::Mod => unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }
                PFHFileType::Movie => unsafe { app_ui.change_packfile_type_movie.as_mut().unwrap().set_checked(true); }
                PFHFileType::Other(_) => unsafe { app_ui.change_packfile_type_other.as_mut().unwrap().set_checked(true); }
            }

            // Enable or disable these, depending on what data we have in the header.
            unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA)); }
            unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS)); }
            unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX)); }
            unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER)); }

            // Update the TreeView.
            update_treeview(
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
                    "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                    "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                    "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                    "shogun_2" | _ => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                }

                // Set the current "Operational Mode" to `MyMod`.
                set_my_mod_mode(&mymod_stuff, mode, Some(pack_file_path));
            }

            // If it's not a "MyMod", we choose the new Game Selected depending on what the open mod id is.
            else {

                // Depending on the Id, choose one game or another.
                match ui_data.pfh_version {

                    // PFH5 is for Warhammer 2/Arena.
                    PFHVersion::PFH5 => {

                        // If the PackFile has the mysterious byte enabled, it's from Arena. Otherwise, it's from Warhammer 2.
                        if ui_data.bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER) { unsafe { app_ui.arena.as_mut().unwrap().trigger(); } }
                        else { unsafe { app_ui.warhammer_2.as_mut().unwrap().trigger(); } }
                    },

                    // PFH4 is for Warhammer 1/Attila/Rome 2.
                    PFHVersion::PFH4 => {

                        // Get the Game Selected.
                        sender_qt.send(Commands::GetGameSelected).unwrap();
                        let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // If we have Warhammer selected, we keep Warhammer. If we have Attila, we keep Attila.
                        // In any other case, we select Rome 2 by default.
                        match &*game_selected {
                            "warhammer" => unsafe { app_ui.warhammer.as_mut().unwrap().trigger(); },
                            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" | _ => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                        }
                    },

                    // PFH3 is for Shogun 2.
                    PFHVersion::PFH3 => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
                }

                // Set the current "Operational Mode" to `Normal`.
                set_my_mod_mode(&mymod_stuff, mode, None);
            }

            // Re-enable the Main Window.
            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

            // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
            purge_them_all(&app_ui, packedfiles_open_in_packedfile_view);

            // Get the current settings.
            sender_qt.send(Commands::GetSettings).unwrap();
            let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
        
            // Close the Global Search stuff and reset the filter's history.
            unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
            if !settings.settings_bool.get("remember_table_state_permanently").unwrap() { history_state_tables.borrow_mut().clear(); }

            // Show the "Tips".
            display_help_tips(&app_ui);
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

/// This function is used to open ANY supported PackedFile in the right view.
fn open_packedfile(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: &AppUI,
    is_modified: &Rc<RefCell<bool>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    global_search_explicit_paths: &Rc<RefCell<Vec<Vec<String>>>>,
    is_folder_tree_view_locked: &Rc<RefCell<bool>>,
    history_state_tables: &Rc<RefCell<BTreeMap<Vec<String>, TableState>>>,
    db_slots: &Rc<RefCell<BTreeMap<i32, PackedFileDBTreeView>>>,
    loc_slots: &Rc<RefCell<BTreeMap<i32, PackedFileLocTreeView>>>,
    text_slots: &Rc<RefCell<BTreeMap<i32, PackedFileTextView>>>,
    rigid_model_slots: &Rc<RefCell<BTreeMap<i32, PackedFileRigidModelDataView>>>,
    update_global_search_stuff: *mut Action,
    view_position: i32,
) -> Result<()> {

    // Before anything else, we need to check if the TreeView is unlocked. Otherwise we don't do anything from here.
    if !(*is_folder_tree_view_locked.borrow()) {

        // Get the selection to see what we are going to open.
        let selection = unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection() };

        // Get the path of the selected item.
        let full_path = get_path_from_item_selection(app_ui.folder_tree_model, &selection, true);

        // Send the Path to the Background Thread, and get the type of the item.
        sender_qt.send(Commands::GetTypeOfPath).unwrap();
        sender_qt_data.send(Data::VecString(full_path)).unwrap();
        let item_type = if let Data::TreePathType(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

        // We act, depending on his type.
        match item_type {

            // Only in case it's a file, we do something.
            TreePathType::File(path) => {

                // If the file we want to open is already open in another view, don't open it.
                for (view_pos, packed_file_path) in packedfiles_open_in_packedfile_view.borrow().iter() {
                    if *packed_file_path.borrow() == path && view_pos != &view_position {
                        return Err(ErrorKind::PackedFileIsOpenInAnotherView)?
                    }
                }

                // Get the name of the PackedFile (we are going to use it a lot).
                let packedfile_name = path.last().unwrap().to_owned();

                // We get his type to decode it properly
                let mut packed_file_type: &str =

                    // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
                    if path[0] == "db" { "DB" }

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

                // Create the widget that'll act as a container for the view.
                let widget = Widget::new().into_raw();
                let widget_layout = GridLayout::new().into_raw();
                unsafe { widget.as_mut().unwrap().set_layout(widget_layout as *mut Layout); }

                // Put the Path into a Rc<RefCell<> so we can alter it while it's open.
                let path = Rc::new(RefCell::new(path));

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
                            widget_layout,
                            &path,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &history_state_tables
                        ) {
                            Ok(new_loc_slots) => { loc_slots.borrow_mut().insert(view_position, new_loc_slots); },
                            Err(error) => return Err(ErrorKind::LocDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
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
                            widget_layout,
                            &path,
                            &global_search_explicit_paths,
                            update_global_search_stuff,
                            &history_state_tables
                        ) {
                            Ok(new_db_slots) => { db_slots.borrow_mut().insert(view_position, new_db_slots); },
                            Err(error) => return Err(ErrorKind::DBTableDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }

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
                            widget_layout,
                            &path
                        ) {
                            Ok(new_text_slots) => { text_slots.borrow_mut().insert(view_position, new_text_slots); },
                            Err(error) => return Err(ErrorKind::TextDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
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
                            widget_layout,
                            &path
                        ) {
                            Ok(new_rigid_model_slots) => { rigid_model_slots.borrow_mut().insert(view_position, new_rigid_model_slots); },
                            Err(error) => return Err(ErrorKind::RigidModelDecode(format!("{}", error)))?,
                        }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // If the file is a Text PackedFile...
                    "IMAGE" => {

                        // Try to get the view build, or return error.
                        if let Err(error) = ui::packedfile_image::create_image_view(
                            sender_qt.clone(),
                            &sender_qt_data,
                            &receiver_qt,
                            widget_layout,
                            &path,
                        ) { return Err(ErrorKind::ImageDecode(format!("{}", error)))? }

                        // Tell the program there is an open PackedFile and finish the table.
                        purge_that_one_specifically(&app_ui, view_position, &packedfiles_open_in_packedfile_view);
                        packedfiles_open_in_packedfile_view.borrow_mut().insert(view_position, path);
                        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(view_position, widget as *mut Widget); }
                    }

                    // For any other PackedFile, just restore the display tips.
                    _ => {
                        purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                        display_help_tips(&app_ui);
                    }
                }
            }

            // If it's anything else, then we just show the "Tips" list.
            _ => {
                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);
                display_help_tips(&app_ui);
            }
        }
    }

    Ok(())
}

/// This function takes care of the re-creation of the "MyMod" list in the following moments:
/// - At the start of the program.
/// - At the end of MyMod deletion.
/// - At the end of MyMod creation.
/// - At the end of settings update.
/// We need to return a tuple with the actions (for further manipulation) and the slots (to keep them alive).
fn build_my_mod_menu(
    sender_qt: Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: Rc<RefCell<Receiver<Data>>>,
    app_ui: AppUI,
    menu_bar_mymod: &*mut Menu,
    is_modified: Rc<RefCell<bool>>,
    mode: Rc<RefCell<Mode>>,
    needs_rebuild: Rc<RefCell<bool>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    close_global_search_action: *mut Action,
    history_state_tables: &Rc<RefCell<BTreeMap<Vec<String>, TableState>>>,
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
    let mymod_stuff = unsafe { MyModStuff {
            new_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&New MyMod")),
            delete_selected_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Delete Selected MyMod")),
            install_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Install")),
            uninstall_mymod: menu_bar_mymod.as_mut().unwrap().add_action(&QString::from_std_str("&Uninstall")),
        }
    };

    // Add a separator in the middle of the menu.
    unsafe { menu_bar_mymod.as_mut().unwrap().insert_separator(mymod_stuff.install_mymod); }

    // And we create the slots.
    let mut mymod_slots = MyModSlots {

        // This slot is used for the "New MyMod" action.
        new_mymod: SlotBool::new(clone!(
            history_state_tables,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            packedfiles_open_in_packedfile_view,
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
                            "thrones_of_britannia" => unsafe { app_ui.thrones_of_britannia.as_mut().unwrap().trigger(); }
                            "attila" => unsafe { app_ui.attila.as_mut().unwrap().trigger(); }
                            "rome_2" => unsafe { app_ui.rome_2.as_mut().unwrap().trigger(); }
                            "shogun_2" | _ => unsafe { app_ui.shogun_2.as_mut().unwrap().trigger(); }
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
                        let _ = if let Data::PathBuf(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // Pass the new PackFile's Path to the worker thread.
                        sender_qt_data.send(Data::PathBuf(mymod_path.to_path_buf())).unwrap();

                        // Check what response we got.
                        match check_message_validity_tryrecv(&receiver_qt) {
                        
                            // If it's success....
                            Data::I64(_) => {

                                // Destroy whatever it's in the PackedFile's view, to avoid data corruption.
                                purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                                // Try to get the settings.
                                sender_qt.send(Commands::GetSettings).unwrap();
                                let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                                // Close the Global Search stuff and reset the filter's history.
                                unsafe { close_global_search_action.as_mut().unwrap().trigger(); }
                                if !settings.settings_bool.get("remember_table_state_permanently").unwrap() { history_state_tables.borrow_mut().clear(); }

                                // Show the "Tips".
                                display_help_tips(&app_ui);

                                // Update the TreeView.
                                update_treeview(
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
                                let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

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
                        let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

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
                            let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // Try to get the settings.
                            sender_qt.send(Commands::GetSettings).unwrap();
                            let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                            // If we have a `game_data_path` for the current `GameSelected`...
                            if let Some(mut game_data_path) = get_game_selected_data_path(&game_selected, &settings) {

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
                        let game_selected = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };
                       
                        // Try to get the settings.
                        sender_qt.send(Commands::GetSettings).unwrap();
                        let settings = if let Data::Settings(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR); };

                        // If we have a `game_data_path` for the current `GameSelected`...
                        if let Some(mut game_data_path) = get_game_selected_data_path(&game_selected, &settings) {

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
                                        history_state_tables,
                                        game_folder_name,
                                        is_modified,
                                        mode,
                                        mymod_stuff,
                                        pack_file,
                                        packedfiles_open_in_packedfile_view,
                                        close_global_search_action,
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt => move |_| {

                                            // Check first if there has been changes in the PackFile.
                                            if are_you_sure(&app_ui, &is_modified, false) {

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
                                                    &packedfiles_open_in_packedfile_view,
                                                    close_global_search_action,
                                                    &history_state_tables,
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


/// This function takes care of the re-creation of the "Open From Content" and "Open From Data" submenus.
/// This has to be executed every time we change the Game Selected.
fn build_open_from_submenus(
    sender_qt: Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: Rc<RefCell<Receiver<Data>>>,
    settings: &Settings,
    app_ui: AppUI,
    submenu_open_from_content: &*mut Menu,
    submenu_open_from_data: &*mut Menu,
    game_selected: &str,
    is_modified: &Rc<RefCell<bool>>,
    mode: &Rc<RefCell<Mode>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
    mymod_stuff: &Rc<RefCell<MyModStuff>>,
    close_global_search_action: *mut Action,
    history_state_tables: &Rc<RefCell<BTreeMap<Vec<String>, TableState>>>,
) -> Vec<SlotBool<'static>> {

    // First, we clear the list, just in case this is a "Rebuild" of the menu.
    unsafe { submenu_open_from_content.as_mut().unwrap().clear(); }
    unsafe { submenu_open_from_data.as_mut().unwrap().clear(); }

    // And we create the slots.
    let mut open_from_slots = vec![];

    //---------------------------------------------------------------------------------------//
    // Build the menus...
    //---------------------------------------------------------------------------------------//

    // Get the path of every PackFile in the data folder (if it's configured) and make an action for each one of them.
    if let Some(ref mut paths) = get_game_selected_content_packfiles_paths(game_selected, &settings) {
        paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
        for path in paths {

            // That means our file is a valid PackFile and it needs to be added to the menu.
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Create the action for it.
            let open_mod_action;
            unsafe { open_mod_action = submenu_open_from_content.as_mut().unwrap().add_action(&QString::from_std_str(mod_name)); }

            // Create the slot for that action.
            let slot_open_mod = SlotBool::new(clone!(
                history_state_tables,
                is_modified,
                mode,
                mymod_stuff,
                path,
                packedfiles_open_in_packedfile_view,
                close_global_search_action,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Check first if there has been changes in the PackFile.
                    if are_you_sure(&app_ui, &is_modified, false) {

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            path.to_path_buf(),
                            &app_ui,
                            &mymod_stuff,
                            &is_modified,
                            &mode,
                            "",
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &history_state_tables,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            ));

            // Add the slot to the list.
            open_from_slots.push(slot_open_mod);

            // Connect the action to the slot.
            unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(open_from_slots.last().unwrap()); }  
        }
    }

    // Get the path of every PackFile in the data folder (if it's configured) and make an action for each one of them.
    if let Some(ref mut paths) = get_game_selected_data_packfiles_paths(game_selected, &settings) {
        paths.sort_unstable_by_key(|x| x.file_name().unwrap().to_string_lossy().as_ref().to_owned());
        for path in paths {

            // That means our file is a valid PackFile and it needs to be added to the menu.
            let mod_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();

            // Create the action for it.
            let open_mod_action;
            unsafe { open_mod_action = submenu_open_from_data.as_mut().unwrap().add_action(&QString::from_std_str(mod_name)); }

            // Create the slot for that action.
            let slot_open_mod = SlotBool::new(clone!(
                history_state_tables,
                is_modified,
                mode,
                mymod_stuff,
                path,
                packedfiles_open_in_packedfile_view,
                close_global_search_action,
                sender_qt,
                sender_qt_data,
                receiver_qt => move |_| {

                    // Check first if there has been changes in the PackFile.
                    if are_you_sure(&app_ui, &is_modified, false) {

                        // Try to open it, and report it case of error.
                        if let Err(error) = open_packfile(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt,
                            path.to_path_buf(),
                            &app_ui,
                            &mymod_stuff,
                            &is_modified,
                            &mode,
                            "",
                            &packedfiles_open_in_packedfile_view,
                            close_global_search_action,
                            &history_state_tables,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            ));

            // Add the slot to the list.
            open_from_slots.push(slot_open_mod);

            // Connect the action to the slot.
            unsafe { open_mod_action.as_ref().unwrap().signals().triggered().connect(open_from_slots.last().unwrap()); }  
        }
    }
    
    // Only if the submenu has items, we enable it.
    unsafe { submenu_open_from_content.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(!submenu_open_from_content.as_mut().unwrap().actions().is_empty()); }
    unsafe { submenu_open_from_data.as_mut().unwrap().menu_action().as_mut().unwrap().set_visible(!submenu_open_from_data.as_mut().unwrap().actions().is_empty()); }

    // Return the slots.
    open_from_slots
}

/// Function to filter the results of a global search, in any of the result tables.
/// If a value is not provided by a slot, we get it from the widget itself.
fn filter_matches_result(
    pattern: Option<QString>,
    column: Option<i32>,
    case_sensitive: Option<bool>,
    filter_model: *mut SortFilterProxyModel,
    filter_line_edit: *mut LineEdit,
    column_selector: *mut ComboBox,
    case_sensitive_button: *mut PushButton,
) {

    // Set the pattern to search.
    let mut pattern = if let Some(pattern) = pattern { RegExp::new(&pattern) }
    else { 
        let pattern;
        unsafe { pattern = RegExp::new(&filter_line_edit.as_mut().unwrap().text()) }
        pattern
    };

    // Set the column selected.
    if let Some(column) = column { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column); }}
    else { unsafe { filter_model.as_mut().unwrap().set_filter_key_column(column_selector.as_mut().unwrap().current_index()); }}

    // Check if the filter should be "Case Sensitive".
    if let Some(case_sensitive) = case_sensitive { 
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
    }

    else {
        unsafe { 
            let case_sensitive = case_sensitive_button.as_mut().unwrap().is_checked();
            if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::Sensitive); }
            else { pattern.set_case_sensitivity(CaseSensitivity::Insensitive); }
        }
    }

    // Filter whatever it's in that column by the text we got.
    unsafe { filter_model.as_mut().unwrap().set_filter_reg_exp(&pattern); }
}
