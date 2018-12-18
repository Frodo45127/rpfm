// This is the main file of RPFM. Here is the main loop that builds the UI and controls his events.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::cast_lossless,                  // Disabled due to useless warnings.
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

// Uses for everything we need. It's a looooong list.
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
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::OsStr;
use std::panic;
use std::path::{Path, PathBuf};
use std::fs::{DirBuilder, copy, remove_file, remove_dir_all};

use chrono::NaiveDateTime;
use indexmap::map::IndexMap;
use lazy_static::lazy_static;

use crate::common::*;
use crate::common::communications::*;
use crate::error::{ErrorKind, logger::Report, Result};
use crate::main_extra::*;
use crate::packfile::packedfile::PackedFile;
use crate::packedfile::*;
use crate::packedfile::db::schemas::Schema;
use crate::packedfile::db::schemas_importer::*;
use crate::packfile::{PFHVersion, PFHFileType, PFHFlags};
use crate::settings::*;
use crate::settings::shortcuts::Shortcuts;
use crate::ui::*;
use crate::ui::dependency_manager::*;
use crate::ui::packedfile_db::*;
use crate::ui::packedfile_loc::*;
use crate::ui::packedfile_text::*;
use crate::ui::packedfile_rigidmodel::*;
use crate::ui::settings::*;
use crate::ui::table_state::*;
use crate::ui::updater::*;

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
mod background_thread_extra;
mod common;
mod error;
mod main_extra;
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
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
            steam_id: Some(594_570),
            ca_types_file: Some("ca_types_wh2".to_owned()),
            supports_editing: true,
        });

        // Warhammer
        map.insert("warhammer", GameInfo {
            display_name: "Warhammer".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_wh.json".to_owned(),
            db_packs: vec![
                "data.pack".to_owned(),         // Central data PackFile
                "data_bl.pack".to_owned(),      // Blood DLC Data
                "data_bm.pack".to_owned()       // Beastmen DLC Data
            ],
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
            steam_id: Some(364_360),
            ca_types_file: None,
            supports_editing: true,
        });

        // Thrones of Britannia
        map.insert("thrones_of_britannia", GameInfo {
            display_name: "Thrones of Britannia".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_tob.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
            steam_id: Some(712_100),
            ca_types_file: None,
            supports_editing: true,
        });

        // Attila
        map.insert("attila", GameInfo {
            display_name: "Attila".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_att.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
            steam_id: Some(325_610),
            ca_types_file: None,
            supports_editing: true,
        });

        // Rome 2
        map.insert("rome_2", GameInfo {
            display_name: "Rome 2".to_owned(),
            id: PFHVersion::PFH4,
            schema: "schema_rom2.json".to_owned(),
            db_packs: vec!["data_rome2.pack".to_owned()],
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
            steam_id: Some(214_950),
            ca_types_file: None,
            supports_editing: true,
        });

        // Shogun 2
        map.insert("shogun_2", GameInfo {
            display_name: "Shogun 2".to_owned(),
            id: PFHVersion::PFH3,
            schema: "schema_sho2.json".to_owned(),
            db_packs: vec!["data.pack".to_owned()],
            loc_packs: vec![
                "local_en.pack".to_owned(),     // English
                "local_br.pack".to_owned(),     // Brazilian
                "local_cz.pack".to_owned(),     // Czech
                "local_ge.pack".to_owned(),     // German
                "local_sp.pack".to_owned(),     // Spanish
                "local_fr.pack".to_owned(),     // French
                "local_it.pack".to_owned(),     // Italian
                "local_kr.pack".to_owned(),     // Korean
                "local_pl.pack".to_owned(),     // Polish
                "local_ru.pack".to_owned(),     // Russian
                "local_tr.pack".to_owned(),     // Turkish
                "local_cn.pack".to_owned(),     // Simplified Chinese
                "local_zh.pack".to_owned(),     // Traditional Chinese
            ],
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

    /// The current Settings and Shortcuts. To avoid reference and lock issues, this should be edited ONLY in the background thread.
    static ref SETTINGS: Arc<Mutex<Settings>> = Arc::new(Mutex::new(Settings::load().unwrap_or_else(|_|Settings::new())));
    static ref SHORTCUTS: Arc<Mutex<Shortcuts>> = Arc::new(Mutex::new(Shortcuts::load().unwrap_or_else(|_|Shortcuts::new())));

    /// The current GameSelected. Same as the one above, only edited from the background thread.
    static ref GAME_SELECTED: Arc<Mutex<String>> = Arc::new(Mutex::new(SETTINGS.lock().unwrap().settings_string["default_game"].to_owned()));

    /// PackedFiles from the dependencies of the currently open PackFile.
    static ref DEPENDENCY_DATABASE: Mutex<Vec<PackedFile>> = Mutex::new(vec![]);

    /// Currently loaded schema.
    static ref SCHEMA: Arc<Mutex<Option<Schema>>> = Arc::new(Mutex::new(None));

    /// History for the filters, search, columns...., so table and loc filters are remembered when zapping files, and cleared when the open PackFile changes.
    /// NOTE: This affects both DB Tables and Loc PackedFiles.
    static ref TABLE_STATES_UI: Mutex<BTreeMap<Vec<String>, TableStateUI>> = Mutex::new(TableStateUI::load().unwrap_or_else(|_| TableStateUI::new()));
}

/// This constant gets RPFM's version from the `Cargo.toml` file, so we don't have to change it
/// in two different places in every update.
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    pub wh2_optimize_packfile: *mut Action,

    // Warhammer's actions.
    pub wh_patch_siege_ai: *mut Action,
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

    // Log the crashes so the user can send them himself.
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
        thread::spawn(move || { background_thread::background_loop(&sender_rust, &receiver_rust, &receiver_rust_data); });

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
        unsafe { global_search_grid.as_mut().unwrap().set_margin(3); }

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
            wh2_optimize_packfile: menu_warhammer_2.as_mut().unwrap().add_action(&QString::from_std_str("&Optimize PackFile")),

            // Warhammer's actions.
            wh_patch_siege_ai: menu_warhammer.as_mut().unwrap().add_action(&QString::from_std_str("&Patch Siege AI")),
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

        // Set the shortcuts for these actions.
        unsafe { app_ui.new_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["new_packfile"]))); }
        unsafe { app_ui.open_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["open_packfile"]))); }
        unsafe { app_ui.save_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["save_packfile"]))); }
        unsafe { app_ui.save_packfile_as.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["save_packfile_as"]))); }
        unsafe { app_ui.load_all_ca_packfiles.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["load_all_ca_packfiles"]))); }
        unsafe { app_ui.preferences.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["preferences"]))); }
        unsafe { app_ui.quit.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().menu_bar_packfile["quit"]))); }

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

        unsafe { app_ui.about_qt.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.about_rpfm.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.open_manual.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }
        unsafe { app_ui.check_schema_updates.as_mut().unwrap().set_shortcut_context(ShortcutContext::Application); }

        //---------------------------------------------------------------------------------------//
        // Preparing initial state of the Main Window...
        //---------------------------------------------------------------------------------------//

        // This cannot go into lazy_static because StandardItem is not send.
        let table_state_data = Rc::new(RefCell::new(TableStateData::new()));

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
            &is_modified,
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
            app_ui.context_menu_rename_current.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
            app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_enabled(false);
        }

        // Set the shortcuts for these actions.
        unsafe { app_ui.context_menu_add_file.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_file"]))); }
        unsafe { app_ui.context_menu_add_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_folder"]))); }
        unsafe { app_ui.context_menu_add_from_packfile.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["add_from_packfile"]))); }
        unsafe { app_ui.context_menu_create_folder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_folder"]))); }
        unsafe { app_ui.context_menu_create_db.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_db"]))); }
        unsafe { app_ui.context_menu_create_loc.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_loc"]))); }
        unsafe { app_ui.context_menu_create_text.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["create_text"]))); }
        unsafe { app_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["mass_import_tsv"]))); }
        unsafe { app_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["mass_export_tsv"]))); }
        unsafe { app_ui.context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["delete"]))); }
        unsafe { app_ui.context_menu_extract.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["extract"]))); }
        unsafe { app_ui.context_menu_rename_current.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["rename_current"]))); }
        unsafe { app_ui.context_menu_apply_prefix_to_selected.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["apply_prefix_to_selected"]))); }
        unsafe { app_ui.context_menu_apply_prefix_to_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["apply_prefix_to_all"]))); }
        unsafe { app_ui.context_menu_open_decoder.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_in_decoder"]))); }
        unsafe { app_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_packfiles_list"]))); }
        unsafe { app_ui.context_menu_open_with_external_program.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_with_external_program"]))); }
        unsafe { app_ui.context_menu_open_in_multi_view.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["open_in_multi_view"]))); }
        unsafe { app_ui.context_menu_global_search.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["global_search"]))); }
        unsafe { app_ui.tree_view_expand_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["expand_all"]))); }
        unsafe { app_ui.tree_view_collapse_all.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().tree_view["collapse_all"]))); }

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
        let optimize_packfile = QString::from_std_str("Check and remove any data in DB Tables and Locs (Locs only for english users) that is unchanged from the base game. That means your mod will only contain the stuff you change, avoiding incompatibilities with other mods.");
        unsafe { app_ui.wh2_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
        unsafe { app_ui.wh2_optimize_packfile.as_mut().unwrap().set_status_tip(&optimize_packfile); }
        unsafe { app_ui.wh_patch_siege_ai.as_mut().unwrap().set_status_tip(&patch_siege_ai_tip); }
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

        // Update the "Game Selected" here, so we can skip some steps when initializing.
        let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
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
            is_modified,
            mymod_stuff,
            mode,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            sender_qt,
            sender_qt_data,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

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

                    // Update the TreeView.
                    update_treeview(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        app_ui.window,
                        app_ui.folder_tree_view,
                        app_ui.folder_tree_model,
                        TreeViewOperation::Build(false),
                    );

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Set the new mod as "Not modified".
                    *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

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
            is_modified,
            mode,
            mymod_stuff,
            table_state_data,
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

                    // In case we have a default path for the Game Selected, we use it as base path for opening files.
                    if let Some(ref path) = get_game_selected_data_path() {

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
                            &table_state_data,
                        ) { show_dialog(app_ui.window, false, error); }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let slot_save_packfile = SlotBool::new(clone!(
            is_modified,
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
                    &is_modified,
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
            is_modified,
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
                    &is_modified,
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
            is_modified,
            mode,
            mymod_stuff,
            sender_qt,
            sender_qt_data,
            table_state_data,
            packedfiles_open_in_packedfile_view,
            receiver_qt => move |_| {

                // Check first if there has been changes in the PackFile.
                if are_you_sure(&app_ui, &is_modified, false) {

                    // Tell the Background Thread to try to load the PackFiles.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    sender_qt.send(Commands::LoadAllCAPackFiles).unwrap();
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
                                &receiver_qt,
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::Build(false),
                            );

                            // Set the new mod as "Not modified".
                            *is_modified.borrow_mut() = set_modified(false, &app_ui, None);

                            let game_selected = GAME_SELECTED.lock().unwrap().to_owned();
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
            is_modified,
            sender_qt,
            sender_qt_data => move |_| {

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

                unsafe { *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(vec![app_ui.folder_tree_model.as_ref().unwrap().item(0).as_ref().unwrap().text().to_std_string()])); }
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
                    Data::StringVecTreePathType(response) => {

                        // If we got a success, remove the files deleted by the patcher function.
                        for item_type in response.1 {
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::DeleteUnselected(item_type),
                            );
                        }

                        // Save the patched PackFile and report the result.
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);
                        if let Err(error) = save_packfile(
                            false,
                            &app_ui,
                            &is_modified,
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
                            ErrorKind::PatchSiegeAIEmptyPackFile => show_dialog(app_ui.window, false, error),
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
            is_modified,
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
                    Data::VecTreePathType(response) => {

                        // Delete the files removed by the optimizer.
                        for item_type in response {
                            update_treeview(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui.window,
                                app_ui.folder_tree_view,
                                app_ui.folder_tree_model,
                                TreeViewOperation::DeleteUnselected(item_type),
                            );
                        }

                        // Save the optimized PackFile.
                        *is_modified.borrow_mut() = set_modified(false, &app_ui, None);
                        if let Err(error) = save_packfile(
                            false,
                            &app_ui,
                            &is_modified,
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
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            &receiver_qt,
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile.to_vec()),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                        // For each file added, remove it from the data history if exists.
                                        for path in &paths_packedfile {
                                            if table_state_data.borrow().get(path).is_some() {
                                                table_state_data.borrow_mut().remove(path);
                                            }
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
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        &receiver_qt,
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile.to_vec()),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                    // For each file added, remove it from the data history if exists.
                                    for path in &paths_packedfile {
                                        if table_state_data.borrow().get(path).is_some() {
                                            table_state_data.borrow_mut().remove(path);
                                        }
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
            is_modified,
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
                                        update_treeview(
                                            &sender_qt,
                                            &sender_qt_data,
                                            &receiver_qt,
                                            app_ui.window,
                                            app_ui.folder_tree_view,
                                            app_ui.folder_tree_model,
                                            TreeViewOperation::Add(paths_packedfile.to_vec()),
                                        );

                                        // Set it as modified. Exception for the Paint System.
                                        *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                        // Update the global search stuff, if needed.
                                        global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                        unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                        // For each file added, remove it from the data history if exists.
                                        for path in &paths_packedfile {
                                            if table_state_data.borrow().get(path).is_some() {
                                                table_state_data.borrow_mut().remove(path);
                                            }
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
                                    update_treeview(
                                        &sender_qt,
                                        &sender_qt_data,
                                        &receiver_qt,
                                        app_ui.window,
                                        app_ui.folder_tree_view,
                                        app_ui.folder_tree_model,
                                        TreeViewOperation::Add(paths_packedfile.to_vec()),
                                    );

                                    // Set it as modified. Exception for the Paint System.
                                    *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                    // Update the global search stuff, if needed.
                                    global_search_explicit_paths.borrow_mut().append(&mut paths_packedfile.to_vec());
                                    unsafe { update_global_search_stuff.as_mut().unwrap().trigger(); }

                                    // For each file added, remove it from the data history if exists.
                                    for path in &paths_packedfile {
                                        if table_state_data.borrow().get(path).is_some() {
                                            table_state_data.borrow_mut().remove(path);
                                        }
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
            is_folder_tree_view_locked,
            is_modified,
            add_from_packfile_slots => move |_| {

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

                            // Block the main `TreeView` from decoding stuff.
                            *is_folder_tree_view_locked.borrow_mut() = true;

                            // Destroy whatever it's in the PackedFile's View.
                            purge_them_all(&app_ui, &packedfiles_open_in_packedfile_view);

                            // Build the TreeView to hold all the Extra PackFile's data and save his slots.
                            *add_from_packfile_slots.borrow_mut() = AddFromPackFileSlots::new_with_grid(
                                &sender_qt,
                                &sender_qt_data,
                                &receiver_qt,
                                app_ui,
                                &is_folder_tree_view_locked,
                                &is_modified,
                                &packedfiles_open_in_packedfile_view,
                                &global_search_explicit_paths,
                                update_global_search_stuff,
                                &table_state_data
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
                        &receiver_qt,
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
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, &PackedFileType::DB("".to_owned(), "".to_owned(), 0)) {

                    // Check what we got to create....
                    match packed_file_type {

                        // If we got correct data from the dialog...
                        Ok(packed_file_type) => {

                            // Get the name of the PackedFile.
                            if let PackedFileType::DB(name, table,_) = packed_file_type.clone() {

                                // If the name is not empty...
                                if !name.is_empty() {

                                    // Get his Path, without the name of the PackFile.
                                    let complete_path = vec!["db".to_owned(), table, name];

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
                                                &receiver_qt,
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
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, &PackedFileType::Loc("".to_owned())) {

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
                                                &receiver_qt,
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
                if let Some(packed_file_type) = create_new_packed_file_dialog(&app_ui, &sender_qt, &sender_qt_data, &receiver_qt, &PackedFileType::Text("".to_owned())) {

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
                                                &receiver_qt,
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
            table_state_data,
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
                        unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                        sender_qt.send(Commands::MassImportTSV).unwrap();
                        sender_qt_data.send(Data::StringVecPathBuf(data)).unwrap();
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
                                    &receiver_qt,
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

                                // For each file added, remove it from the data history if exists.
                                for path in &paths.1 {
                                    if table_state_data.borrow().get(path).is_some() {
                                        table_state_data.borrow_mut().remove(path);
                                    }
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

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let slot_contextual_menu_delete = SlotBool::new(clone!(
            sender_qt,
            sender_qt_data,
            receiver_qt,
            table_state_data,
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
                                } else { return }
                            }

                            if table_state_data.borrow().get(&item_path).is_some() {
                                table_state_data.borrow_mut().remove(&item_path);
                            }
                        }
                        TreePathType::Folder(item_path) => {
                            if !skaven_confirm {
                                if !item_path.is_empty() && open_path.borrow().starts_with(&item_path) {

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

                                // For each file added, remove it from the data history if exists.
                                let coincidences = table_state_data.borrow().iter().filter(|(x, _)| x.starts_with(&item_path) && !x.is_empty()).map(|x| x.0.to_vec()).collect::<Vec<Vec<String>>>();
                                for path in &coincidences { 
                                    table_state_data.borrow_mut().remove(path);
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
                            &receiver_qt,
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

                // Depending on the current Operational Mode...
                match *mode.borrow() {

                    // If we have a "MyMod" selected...
                    Mode::MyMod {ref game_folder_name, ref mod_name} => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        if let Some(ref mymods_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.window, false, ErrorKind::IOCreateAssetFolder);
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
                                    if index < (path_without_packfile.len() - 1) && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                        return show_dialog(app_ui.window, false, ErrorKind::IOCreateNestedAssetFolder);
                                    }
                                }
                            }

                            // Tell the Background Thread to delete the selected stuff.
                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                            sender_qt.send(Commands::ExtractPackedFile).unwrap();
                            sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), assets_folder.to_path_buf()))).unwrap();

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
                        if SETTINGS.lock().unwrap().settings_bool["use_pfm_extracting_behavior"] {

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
                                        let base_extraction_path = PathBuf::from(extraction_path.to_std_string());

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
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                                sender_qt.send(Commands::ExtractPackedFile).unwrap();
                                sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), final_extraction_path.to_path_buf()))).unwrap();

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
                                let extraction_path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());

                                // Tell the Background Thread to delete the selected stuff.
                                unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                                sender_qt.send(Commands::ExtractPackedFile).unwrap();
                                sender_qt_data.send(Data::VecStringPathBuf((path.to_vec(), extraction_path.to_path_buf()))).unwrap();

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
                        &sender_qt,
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
                    &sender_qt,
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
            global_search_explicit_paths,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            db_slots,
            loc_slots,
            text_slots,
            table_state_data,
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
                    &db_slots,
                    &loc_slots,
                    &text_slots,
                    &rigid_model_slots,
                    update_global_search_stuff,
                    &table_state_data,
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
            table_state_data,
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
                                        &receiver_qt,
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

                                    // Same for the TableStateData stuff. If we find one of the paths in it, we remove it and re-insert it with the new name.
                                    match item_type {
                                        TreePathType::File(ref item_path) => {
                                            if table_state_data.borrow().get(item_path).is_some() {
                                                let mut new_path = path.to_vec();
                                                *new_path.last_mut().unwrap() = new_name.to_owned();
                                                
                                                let mut data = table_state_data.borrow_mut().remove(item_path).unwrap();
                                                data.is_renamed = true;
                                                table_state_data.borrow_mut().insert(new_path.to_vec(), data);
                                            }
                                        } 

                                        TreePathType::Folder(ref item_path) => {
                                            let matches = table_state_data.borrow().keys().filter(|x| x.starts_with(item_path) && !x.is_empty()).cloned().collect::<Vec<Vec<String>>>();
                                            for old_path in matches {
                                                let mut new_path = path.to_vec();
                                                *new_path.last_mut().unwrap() = new_name.to_owned();
                                                
                                                let mut data = table_state_data.borrow_mut().remove(&old_path).unwrap();
                                                data.is_renamed = true;
                                                table_state_data.borrow_mut().insert(new_path.to_vec(), data);
                                            }
                                        }
                                        _ => unreachable!(),
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
            table_state_data,
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
                                    &receiver_qt,
                                    app_ui.window,
                                    app_ui.folder_tree_view,
                                    app_ui.folder_tree_model,
                                    TreeViewOperation::PrefixFiles(old_paths.to_vec(), prefix.to_owned()),
                                );

                                // Set the mod as "Modified". This is an exception to the paint system.
                                *is_modified.borrow_mut() = set_modified(true, &app_ui, None);

                                // If we have a PackedFile open, we have to rename it in that list too. Note that a path can be empty (the dep manager), so we have to check that too.
                                for open_path in packedfiles_open_in_packedfile_view.borrow().values() {
                                    if !open_path.borrow().is_empty() && !path.is_empty() && open_path.borrow().starts_with(&path) {
                                        let new_name = format!("{}{}", prefix, *open_path.borrow().last().unwrap());
                                        *open_path.borrow_mut().last_mut().unwrap() = new_name.to_owned();
                                    }
                                }

                                for old_path in &old_paths {
                                    if table_state_data.borrow().get(old_path).is_some() {

                                        let mut new_path = old_path.to_vec();
                                        let new_name = format!("{}{}", prefix, *new_path.last().unwrap());
                                        *new_path.last_mut().unwrap() = new_name.to_owned();
                                        
                                        let mut data = table_state_data.borrow_mut().remove(old_path).unwrap();
                                        data.is_renamed = true;
                                        table_state_data.borrow_mut().insert(new_path, data);
                                    }
                                }

                                // Update the global search stuff, if needed.
                                let mut new_paths = old_paths.to_vec();
                                for path in &mut new_paths {
                                    let new_name = format!("{}{}", prefix, *path.last().unwrap());
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
            table_state_data,
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
                                &receiver_qt,
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
                                    let new_name = format!("{}{}", prefix, *open_path.borrow().last().unwrap());
                                    *open_path.borrow_mut().last_mut().unwrap() = new_name.to_owned();
                                }
                            }

                            for old_path in &old_paths {
                                if table_state_data.borrow().get(old_path).is_some() {

                                    let mut new_path = old_path.to_vec();
                                    let new_name = format!("{}{}", prefix, *new_path.last().unwrap());
                                    *new_path.last_mut().unwrap() = new_name.to_owned();
                                    
                                    let mut data = table_state_data.borrow_mut().remove(old_path).unwrap();
                                    data.is_renamed = true;
                                    table_state_data.borrow_mut().insert(new_path, data);
                                }
                            }

                            // Update the global search stuff, if needed.
                            let mut new_paths = old_paths.to_vec();
                            for path in &mut new_paths {
                                let new_name = format!("{}{}", prefix, *path.last().unwrap());
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
            global_search_explicit_paths,
            db_slots,
            loc_slots,
            text_slots,
            rigid_model_slots,
            sender_qt,
            sender_qt_data,
            receiver_qt,
            is_modified,
            table_state_data,
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
                    &db_slots,
                    &loc_slots,
                    &text_slots,
                    &rigid_model_slots,
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
            mymod_stuff,
            sender_qt,
            packedfiles_open_in_packedfile_view,
            sender_qt_data,
            receiver_qt,
            is_modified,
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
                        &is_modified,
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
            is_modified,
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
                        &is_modified,
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
                    path,
                    &app_ui,
                    &mymod_stuff,
                    &is_modified,
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
            } else { 
                Application::set_style(&QString::from_std_str("windowsvista"));
                Application::set_palette(&LIGHT_PALETTE);
            }
        }

        // If we have it enabled in the prefs, check if there are updates.
        if SETTINGS.lock().unwrap().settings_bool["check_updates_on_start"] { check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if SETTINGS.lock().unwrap().settings_bool["check_schema_updates_on_start"] { check_schema_updates(&app_ui, false, &sender_qt, &sender_qt_data, &receiver_qt) };

        // And launch it.
        Application::exec()
    })
}
