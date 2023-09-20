//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `GlobalSearchSlots`.

This module contains all the code needed to initialize the Global Search Panel.
!*/

use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDockWidget;
use qt_widgets::QGroupBox;
use qt_widgets::QLineEdit;
use qt_widgets::QMainWindow;
use qt_widgets::QRadioButton;
use qt_widgets::QTabWidget;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QChar;
use qt_core::QPtr;
use qt_core::QFlags;
use qt_core::QModelIndex;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::{CaseSensitivity, DockWidgetArea, Orientation, SortOrder};
use qt_core::QObject;
use qt_core::QRegExp;
use qt_core::QSignalBlocker;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CppBox;
use cpp_core::Ptr;

use anyhow::Result;
use getset::Getters;
use rayon::prelude::*;

use std::rc::Rc;

use rpfm_extensions::search::{GlobalSearch, MatchHolder,
    anim_fragment_battle::{AnimFragmentBattleMatches, AnimFragmentBattleMatch},
    atlas::{AtlasMatches, AtlasMatch},
    portrait_settings::{PortraitSettingsMatches, PortraitSettingsMatch},
    rigid_model::{RigidModelMatches, RigidModelMatch},
    SearchSource,
    schema::SchemaMatches,
    table::{TableMatches, TableMatch},
    text::{TextMatches, TextMatch},
    unit_variant::{UnitVariantMatches, UnitVariantMatch},
    unknown::{UnknownMatches, UnknownMatch}
};
use rpfm_lib::files::FileType;
use rpfm_lib::utils::closest_valid_char_byte;

use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::settings::setting_int;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::{kline_edit_configure_safe, new_treeview_filter_safe, scroll_to_row_safe, trigger_treeview_filter_safe};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::references_ui::ReferencesUI;
use crate::settings_ui::backend::*;
use crate::TREEVIEW_ICONS;
use crate::utils::*;
use crate::UI_STATE;
use crate::views::table::utils::open_subtable;

pub mod connections;
pub mod slots;
pub mod tips;

/// Tool's ui template path.
const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/global_search_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/global_search_dock_widget.ui";

const ANIM_FRAGMENT_BATTLE_ENTRY_INDEX: i32 = 40;
const ANIM_FRAGMENT_BATTLE_SUBENTRY_INDEX: i32 = 41;
const ANIM_FRAGMENT_BATTLE_BOOL_DATA: i32 = 42;

const PORTRAIT_SETTINGS_ENTRY_INDEX: i32 = 40;
const PORTRAIT_SETTINGS_BOOL_DATA: i32 = 41;
const PORTRAIT_SETTINGS_VARIANT_INDEX: i32 = 42;

const UNIT_VARIANT_ENTRY_INDEX: i32 = 40;
const UNIT_VARIANT_BOOL_DATA: i32 = 41;
const UNIT_VARIANT_VARIANT_INDEX: i32 = 42;

//const MATCH_TEXT_START: i32 = 45;
//const MATCH_TEXT_END: i32 = 46;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Global Search panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct GlobalSearchUI {
    dock_widget: QPtr<QDockWidget>,

    search_line_edit: QPtr<QLineEdit>,
    search_button: QPtr<QToolButton>,
    clear_button: QPtr<QToolButton>,
    case_sensitive_checkbox: QPtr<QToolButton>,

    replace_line_edit: QPtr<QLineEdit>,
    replace_button: QPtr<QToolButton>,
    replace_all_button: QPtr<QToolButton>,
    use_regex_checkbox: QPtr<QToolButton>,

    search_source_packfile: QPtr<QRadioButton>,
    search_source_parent: QPtr<QRadioButton>,
    search_source_game: QPtr<QRadioButton>,
    search_source_asskit: QPtr<QRadioButton>,

    search_on_all_checkbox: QPtr<QCheckBox>,
    search_on_all_common_checkbox: QPtr<QCheckBox>,
    search_on_anim_checkbox: QPtr<QCheckBox>,
    search_on_anim_fragment_battle_checkbox: QPtr<QCheckBox>,
    search_on_anim_pack_checkbox: QPtr<QCheckBox>,
    search_on_anims_table_checkbox: QPtr<QCheckBox>,
    search_on_atlas_checkbox: QPtr<QCheckBox>,
    search_on_audio_checkbox: QPtr<QCheckBox>,
    search_on_bmd_checkbox: QPtr<QCheckBox>,
    search_on_db_checkbox: QPtr<QCheckBox>,
    search_on_esf_checkbox: QPtr<QCheckBox>,
    search_on_group_formations_checkbox: QPtr<QCheckBox>,
    search_on_image_checkbox: QPtr<QCheckBox>,
    search_on_loc_checkbox: QPtr<QCheckBox>,
    search_on_matched_combat_checkbox: QPtr<QCheckBox>,
    search_on_pack_checkbox: QPtr<QCheckBox>,
    search_on_portrait_settings_checkbox: QPtr<QCheckBox>,
    search_on_rigid_model_checkbox: QPtr<QCheckBox>,
    search_on_schemas_checkbox: QPtr<QCheckBox>,
    search_on_sound_bank_checkbox: QPtr<QCheckBox>,
    search_on_text_checkbox: QPtr<QCheckBox>,
    search_on_uic_checkbox: QPtr<QCheckBox>,
    search_on_unit_variant_checkbox: QPtr<QCheckBox>,
    search_on_unknown_checkbox: QPtr<QCheckBox>,
    search_on_video_checkbox: QPtr<QCheckBox>,

    matches_tab_widget: QPtr<QTabWidget>,

    matches_table_and_text_tree_view: QPtr<QTreeView>,
    matches_schema_tree_view: QPtr<QTreeView>,

    matches_table_and_text_tree_model: QBox<QStandardItemModel>,
    matches_schema_tree_model: QBox<QStandardItemModel>,

    matches_filter_table_and_text_line_edit: QPtr<QLineEdit>,
    matches_filter_schema_line_edit: QPtr<QLineEdit>,

    matches_case_sensitive_table_and_text_button: QPtr<QToolButton>,
    matches_case_sensitive_schema_button: QPtr<QToolButton>,

    matches_column_selector_table_and_text_combobox: QPtr<QComboBox>,
    matches_column_selector_schema_combobox: QPtr<QComboBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `GlobalSearchUI`.
impl GlobalSearchUI {

    /// This function creates an entire `GlobalSearchUI` struct.
    pub unsafe fn new(main_window: &QBox<QMainWindow>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(main_window, template_path)?;

        let dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();

        // Create and configure the 'Global Search` Dock Widget and all his contents.
        main_window.add_dock_widget_2a(DockWidgetArea::RightDockWidgetArea, &dock_widget);
        dock_widget.set_window_title(&qtr("global_search"));
        dock_widget.set_object_name(&QString::from_std_str("global_search_dock"));

        // Create the search & replace section.
        let search_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "search_line_edit")?;
        let search_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "search_button")?;
        let clear_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "clear_button")?;
        let case_sensitive_checkbox: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "case_sensitive_search_button")?;
        search_line_edit.set_placeholder_text(&qtr("global_search_search_placeholder"));
        search_button.set_tool_tip(&qtr("global_search_search"));
        clear_button.set_tool_tip(&qtr("global_search_clear"));
        case_sensitive_checkbox.set_tool_tip(&qtr("global_search_case_sensitive"));
        kline_edit_configure_safe(&search_line_edit.static_upcast::<QWidget>().as_ptr());

        let replace_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "replace_line_edit")?;
        let replace_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_button")?;
        let replace_all_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "replace_all_button")?;
        let use_regex_checkbox: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "regex_button")?;
        replace_line_edit.set_placeholder_text(&qtr("global_search_replace_placeholder"));
        replace_button.set_tool_tip(&qtr("global_search_replace"));
        replace_all_button.set_tool_tip(&qtr("global_search_replace_all"));
        use_regex_checkbox.set_tool_tip(&qtr("global_search_use_regex"));
        kline_edit_configure_safe(&replace_line_edit.static_upcast::<QWidget>().as_ptr());

        let search_on_group_box: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "search_on_groupbox")?;
        search_on_group_box.set_title(&qtr("global_search_search_on"));

        let search_source_packfile: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_packfile")?;
        let search_source_parent: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_parent")?;
        let search_source_game: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_game")?;
        let search_source_asskit: QPtr<QRadioButton> = find_widget(&main_widget.static_upcast(), "source_asskit")?;
        search_source_packfile.set_text(&qtr("global_search_source_packfile"));
        search_source_parent.set_text(&qtr("global_search_source_parent"));
        search_source_game.set_text(&qtr("global_search_source_game"));
        search_source_asskit.set_text(&qtr("global_search_source_asskit"));
        search_source_game.set_checked(true);

        // Remember the last status of the source radio.
        if setting_variant_from_q_setting(&settings(), "global_search_source_status").can_convert(2) {
            let status = setting_int("global_search_source_status");
            match status {
                0 => search_source_packfile.set_checked(true),
                1 => search_source_parent.set_checked(true),
                2 => search_source_game.set_checked(true),
                3 => search_source_asskit.set_checked(true),
                _ => {}
            }
        }

        let search_source_group_box: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "search_source_groupbox")?;
        search_source_group_box.set_title(&qtr("global_search_search_source"));

        let search_on_all_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_all")?;
        let search_on_all_common_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_all_common")?;
        let search_on_anim_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_anim")?;
        let search_on_anim_fragment_battle_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_anim_fragment_battle")?;
        let search_on_anim_pack_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_anim_pack")?;
        let search_on_anims_table_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_anims_table")?;
        let search_on_atlas_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_atlas")?;
        let search_on_audio_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_audio")?;
        let search_on_bmd_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_bmd")?;
        let search_on_db_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_db")?;
        let search_on_esf_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_esf")?;
        let search_on_group_formations_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_group_formations")?;
        let search_on_image_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_image")?;
        let search_on_loc_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_loc")?;
        let search_on_matched_combat_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_matched_combat")?;
        let search_on_pack_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_pack")?;
        let search_on_portrait_settings_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_portrait_settings")?;
        let search_on_rigid_model_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_rigid_model")?;
        let search_on_schemas_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_schemas")?;
        let search_on_sound_bank_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_sound_bank")?;
        let search_on_text_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_text")?;
        let search_on_uic_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_uic")?;
        let search_on_unit_variant_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_unit_variant")?;
        let search_on_unknown_checkbox: QPtr<QCheckBox> = find_widget(&main_widget.static_upcast(), "search_unknown")?;
        let search_on_video_checkbox: QPtr<QCheckBox> = QCheckBox::from_q_widget(&main_widget).into_q_ptr();//find_widget(&main_widget.static_upcast(), "search_video")?;

        search_on_all_checkbox.set_text(&qtr("global_search_all"));
        search_on_all_common_checkbox.set_text(&qtr("global_search_all_common"));
        search_on_anim_checkbox.set_text(&qtr("global_search_anim"));
        search_on_anim_fragment_battle_checkbox.set_text(&qtr("global_search_anim_fragment_battle"));
        search_on_anim_pack_checkbox.set_text(&qtr("global_search_anim_pack"));
        search_on_anims_table_checkbox.set_text(&qtr("global_search_anims_table"));
        search_on_atlas_checkbox.set_text(&qtr("global_search_atlas"));
        search_on_audio_checkbox.set_text(&qtr("global_search_audio"));
        search_on_bmd_checkbox.set_text(&qtr("global_search_bmd"));
        search_on_db_checkbox.set_text(&qtr("global_search_db"));
        search_on_esf_checkbox.set_text(&qtr("global_search_esf"));
        search_on_group_formations_checkbox.set_text(&qtr("global_search_group_formations"));
        search_on_image_checkbox.set_text(&qtr("global_search_image"));
        search_on_loc_checkbox.set_text(&qtr("global_search_loc"));
        search_on_matched_combat_checkbox.set_text(&qtr("global_search_matched_combat"));
        search_on_pack_checkbox.set_text(&qtr("global_search_pack"));
        search_on_portrait_settings_checkbox.set_text(&qtr("global_search_portrait_settings"));
        search_on_rigid_model_checkbox.set_text(&qtr("global_search_rigid_model"));
        search_on_schemas_checkbox.set_text(&qtr("global_search_schemas"));
        search_on_sound_bank_checkbox.set_text(&qtr("global_search_sound_bank"));
        search_on_text_checkbox.set_text(&qtr("global_search_text"));
        search_on_uic_checkbox.set_text(&qtr("global_search_uic"));
        search_on_unit_variant_checkbox.set_text(&qtr("global_search_unit_variant"));
        search_on_unknown_checkbox.set_text(&qtr("global_search_unknown"));
        search_on_video_checkbox.set_text(&qtr("global_search_video"));

        search_on_anim_checkbox.set_visible(false);
        search_on_anim_fragment_battle_checkbox.set_visible(true);
        search_on_anim_pack_checkbox.set_visible(false);
        search_on_anims_table_checkbox.set_visible(false);
        search_on_atlas_checkbox.set_visible(true);
        search_on_audio_checkbox.set_visible(false);
        search_on_bmd_checkbox.set_visible(false);
        search_on_db_checkbox.set_visible(true);
        search_on_esf_checkbox.set_visible(false);
        search_on_group_formations_checkbox.set_visible(false);
        search_on_image_checkbox.set_visible(false);
        search_on_loc_checkbox.set_visible(true);
        search_on_matched_combat_checkbox.set_visible(false);
        search_on_pack_checkbox.set_visible(false);
        search_on_portrait_settings_checkbox.set_visible(true);
        search_on_rigid_model_checkbox.set_visible(true);
        search_on_schemas_checkbox.set_visible(true);
        search_on_sound_bank_checkbox.set_visible(false);
        search_on_text_checkbox.set_visible(true);
        search_on_uic_checkbox.set_visible(false);
        search_on_unit_variant_checkbox.set_visible(true);
        search_on_unknown_checkbox.set_visible(true);
        search_on_video_checkbox.set_visible(false);

        // Create the frames for the matches tables.
        let matches_tab_widget: QPtr<QTabWidget> = find_widget(&main_widget.static_upcast(), "results_tab_widget")?;
        matches_tab_widget.tab_bar().set_expanding(true);

        // Tables and texts.
        let matches_widget_table_and_text: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "tab_table_and_text")?;
        let tree_view_matches_table_and_text: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "table_and_text_tree_view")?;
        let filter_matches_table_and_text_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "table_and_text_filter_line_edit")?;
        let filter_matches_table_and_text_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "table_and_text_filter_case_sensitive_button")?;
        let filter_matches_table_and_text_column_selector: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "table_and_text_column_combo_box")?;
        let filter_matches_table_and_text_column_list = QStandardItemModel::new_1a(&matches_widget_table_and_text);
        filter_matches_table_and_text_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_table_and_text_column_selector.set_model(&filter_matches_table_and_text_column_list);
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_packedfile"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_column"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_row"));
        filter_matches_table_and_text_column_selector.add_item_q_string(&qtr("gen_loc_match"));
        filter_matches_table_and_text_case_sensitive_button.set_tool_tip(&qtr("global_search_case_sensitive"));

        let matches_table_and_text_tree_filter = new_treeview_filter_safe(tree_view_matches_table_and_text.static_upcast());
        let matches_table_and_text_tree_model = QStandardItemModel::new_1a(&tree_view_matches_table_and_text);
        tree_view_matches_table_and_text.set_model(&matches_table_and_text_tree_filter);
        matches_table_and_text_tree_filter.set_source_model(&matches_table_and_text_tree_model);

        // Schema
        let matches_widget_schema: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "tab_schema")?;
        let tree_view_matches_schema: QPtr<QTreeView> = find_widget(&main_widget.static_upcast(), "schema_tree_view")?;
        let filter_matches_schema_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "schema_filter_line_edit")?;
        let filter_matches_schema_case_sensitive_button: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "schema_filter_case_sensitive_button")?;
        let filter_matches_schema_column_selector: QPtr<QComboBox> = find_widget(&main_widget.static_upcast(), "schema_column_combo_box")?;
        let filter_matches_schema_column_list = QStandardItemModel::new_1a(&matches_widget_schema);
        filter_matches_schema_line_edit.set_placeholder_text(&qtr("packedfile_filter"));
        filter_matches_schema_column_selector.set_model(&filter_matches_schema_column_list);
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_table_name"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_version"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_column_name"));
        filter_matches_schema_column_selector.add_item_q_string(&qtr("global_search_column"));
        filter_matches_schema_case_sensitive_button.set_tool_tip(&qtr("global_search_case_sensitive"));

        let matches_schema_tree_filter = new_treeview_filter_safe(tree_view_matches_schema.static_upcast());
        let matches_schema_tree_model = QStandardItemModel::new_1a(&tree_view_matches_schema);
        tree_view_matches_schema.set_model(&matches_schema_tree_filter);
        matches_schema_tree_filter.set_source_model(&matches_schema_tree_model);

        matches_tab_widget.set_tab_text(0, &qtr("global_search_file_matches"));
        matches_tab_widget.set_tab_text(1, &qtr("global_search_schema_matches"));

        // Hide this widget by default.
        dock_widget.hide();

        // Create ***Da monsta***.
        Ok(Self {
            dock_widget,
            search_line_edit,
            search_button,

            replace_line_edit,
            replace_button,
            replace_all_button,

            clear_button,
            case_sensitive_checkbox,
            use_regex_checkbox,

            search_source_packfile,
            search_source_parent,
            search_source_game,
            search_source_asskit,

            search_on_all_checkbox,
            search_on_all_common_checkbox,
            search_on_anim_checkbox,
            search_on_anim_fragment_battle_checkbox,
            search_on_anim_pack_checkbox,
            search_on_anims_table_checkbox,
            search_on_audio_checkbox,
            search_on_atlas_checkbox,
            search_on_bmd_checkbox,
            search_on_db_checkbox,
            search_on_esf_checkbox,
            search_on_group_formations_checkbox,
            search_on_image_checkbox,
            search_on_loc_checkbox,
            search_on_matched_combat_checkbox,
            search_on_pack_checkbox,
            search_on_portrait_settings_checkbox,
            search_on_rigid_model_checkbox,
            search_on_schemas_checkbox,
            search_on_sound_bank_checkbox,
            search_on_text_checkbox,
            search_on_uic_checkbox,
            search_on_unit_variant_checkbox,
            search_on_unknown_checkbox,
            search_on_video_checkbox,

            matches_tab_widget,

            matches_table_and_text_tree_view: tree_view_matches_table_and_text,
            matches_schema_tree_view: tree_view_matches_schema,

            matches_table_and_text_tree_model,
            matches_schema_tree_model,

            matches_filter_table_and_text_line_edit: filter_matches_table_and_text_line_edit,
            matches_filter_schema_line_edit: filter_matches_schema_line_edit,

            matches_case_sensitive_table_and_text_button: filter_matches_table_and_text_case_sensitive_button,
            matches_case_sensitive_schema_button: filter_matches_schema_case_sensitive_button,

            matches_column_selector_table_and_text_combobox: filter_matches_table_and_text_column_selector,
            matches_column_selector_schema_combobox: filter_matches_schema_column_selector,
        })
    }

    /// This function is used to search the entire PackFile, using the data in Self for the search.
    pub unsafe fn search(&self, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // Create the global search and populate it with all the settings for the search.
        let receiver = match self.search_data_from_ui(true, false) {
            Some(global_search) => CENTRAL_COMMAND.send_background(Command::GlobalSearch(global_search)),
            None => return,
        };

        // Setup all the column's data while waiting for the results.
        self.build_trees();

        // Load the results to their respective models. Then, store the GlobalSearch for future checks.
        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {

                // Focus on the tree with the results. We do it before loading because it's quite a lot faster that way.
                if !global_search.matches().db().is_empty() || !global_search.matches().loc().is_empty() || !global_search.matches().text().is_empty() {
                    self.matches_tab_widget().set_current_index(0);
                }

                else if !global_search.matches().schema().matches().is_empty() {
                    self.matches_tab_widget().set_current_index(1);
                }

                self.load_anim_fragment_battle_matches_to_ui(&global_search.matches().anim_fragment_battle(), FileType::AnimFragmentBattle);
                self.load_atlas_matches_to_ui(&global_search.matches().atlas(), FileType::Atlas);
                self.load_portrait_settings_matches_to_ui(&global_search.matches().portrait_settings(), FileType::PortraitSettings);
                self.load_rigid_model_matches_to_ui(&global_search.matches().rigid_model(), FileType::RigidModel);
                self.load_table_matches_to_ui(&global_search.matches().db(), FileType::DB);
                self.load_table_matches_to_ui(&global_search.matches().loc(), FileType::Loc);
                self.load_text_matches_to_ui(&global_search.matches().text(), FileType::Text);
                self.load_unit_variant_matches_to_ui(&global_search.matches().unit_variant(), FileType::UnitVariant);
                self.load_unknown_matches_to_ui(&global_search.matches().unknown(), FileType::Unknown);
                self.load_schema_matches_to_ui(&global_search.matches().schema());

                UI_STATE.set_global_search(&global_search);
                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(&self.dock_widget, error, false),
            _ => unimplemented!()
        }
    }

    /// This function clears the Global Search result's data, and reset the UI for it.
    pub unsafe fn clear(&self) {
        UI_STATE.set_global_search(&GlobalSearch::default());

        self.matches_table_and_text_tree_model.clear();
        self.matches_schema_tree_model.clear();
    }

    /// This function replace the currently selected match with the provided text.
    pub unsafe fn replace_current(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let receiver = match self.search_data_from_ui(false, true) {
            Some(global_search) => {
                if global_search.source() != &SearchSource::Pack {
                    return show_dialog(app_ui.main_window(), "The dependencies are read-only. You cannot do a Global Replace over them.", false);
                }

                let matches = self.matches_from_selection();
                CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceMatches(global_search, matches.to_vec()))
            },
            None => return,
        };

        // Before rebuilding the tree, check what items are expanded, to re-expand them later.
        let filter_model: QPtr<QSortFilterProxyModel> = self.matches_table_and_text_tree_view.model().static_downcast();
        let root = self.matches_table_and_text_tree_model.invisible_root_item();
        let mut expanded = vec![];

        for index in 0..root.row_count() {
            let source_index = root.child_1a(index).index();
            let view_index = filter_model.map_from_source(&source_index);
            if view_index.is_valid() && self.matches_table_and_text_tree_view.is_expanded(&view_index) {
                expanded.push(self.matches_table_and_text_tree_model.item_1a(index).text());
            }
        }

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {

                // Re-search to update the results.
                UI_STATE.set_global_search(&global_search);
                self.search(pack_file_contents_ui);

                // Update the views of the updated PackedFiles.
                for path in packed_files_info.iter().map(|x| x.path()) {
                    if let Some(file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| &*x.path_read() == path && x.data_source() == DataSource::PackFile) {
                        if let Err(error) = file_view.reload(path, pack_file_contents_ui) {
                            show_dialog(app_ui.main_window(), error, false);
                        }
                    }
                }

                // Re-expand the previously expanded items. We disable animation to avoid the slow opening behaviour of the UI.
                self.matches_table_and_text_tree_view.set_animated(false);

                let root = self.matches_table_and_text_tree_model.invisible_root_item();
                for index in 0..root.row_count() {
                    let source_item = root.child_1a(index);

                    if expanded.iter().any(|old| source_item.text().compare_q_string(old) == 0) {
                        let source_index = source_item.index();
                        let view_index = filter_model.map_from_source(&source_index);
                        if view_index.is_valid() && !self.matches_table_and_text_tree_view.is_expanded(&view_index) {
                            self.matches_table_and_text_tree_view.expand(&view_index)
                        }
                    }
                }

                self.matches_table_and_text_tree_view.set_animated(true);

                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => unimplemented!()
        }
    }

    /// This function replace all the matches in the current search with the provided text.
    pub unsafe fn replace_all(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {

        // To avoid conflicting data, we close all PackedFiles hard and re-search before replacing.
        if let Err(error) = AppUI::back_to_back_end_all(app_ui, pack_file_contents_ui) {
            return show_dialog(app_ui.main_window(), error, false);
        }

        // Update the search results so we have all the ones we need to update.
        self.search(pack_file_contents_ui);
        let receiver = match self.search_data_from_ui(false, true) {
            Some(global_search) => {
                if global_search.source() != &SearchSource::Pack {
                    return show_dialog(app_ui.main_window(), "The dependencies are read-only. You cannot do a Global Replace over them.", false);
                }

                CENTRAL_COMMAND.send_background(Command::GlobalSearchReplaceAll(global_search))
            },
            None => return,
        };

        match CentralCommand::recv(&receiver) {
            Response::GlobalSearchVecRFileInfo(global_search, packed_files_info) => {

                // Re-search to update the results.
                UI_STATE.set_global_search(&global_search);
                self.search(pack_file_contents_ui);

                for path in packed_files_info.iter().map(|x| x.path()) {
                    if let Some(file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|x| &*x.path_read() == path && x.data_source() == DataSource::PackFile) {
                        if let Err(error) = file_view.reload(path, pack_file_contents_ui) {
                            show_dialog(app_ui.main_window(), error, false);
                        }
                    }
                }

                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(packed_files_info), DataSource::PackFile);
            },
            Response::Error(error) => show_dialog(app_ui.main_window(), error, false),
            _ => unimplemented!()
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    ///
    /// Remember, it TRIES to open it. It may fail if the file doesn't exist anymore and the update search
    /// hasn't been triggered, or if the searched text doesn't exist anymore.
    ///
    /// In case the provided ModelIndex is the parent, we open the file without scrolling to the match.
    pub unsafe fn open_match(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        let gidhora = model.item_from_index(&model_index);
        let is_match = !gidhora.has_children();

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let path: String = if is_match {
            let parent = gidhora.parent();

            // Sometimes this is null, not sure why.
            if parent.is_null() { return; }
            parent.text().to_std_string()
        }

        // If not... just expand and open the PackedFile.
        else {
            gidhora.text().to_std_string()
        };

        let global_search = UI_STATE.get_global_search();
        let data_source = match global_search.source() {
            SearchSource::Pack => {
                let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&path, DataSource::PackFile);

                // Manually select the open PackedFile, then open it. This means we can open PackedFiles nor in out filter.
                UI_STATE.set_packfile_contents_read_only(true);

                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }

                UI_STATE.set_packfile_contents_read_only(false);
                DataSource::PackFile
            },

            SearchSource::ParentFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::ParentFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::ParentFiles
            },
            SearchSource::GameFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::GameFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::GameFiles
            },
            SearchSource::AssKitFiles => {
                let tree_index = dependencies_ui.dependencies_tree_view().expand_treeview_to_item(&path, DataSource::AssKitFiles);
                if let Some(ref tree_index) = tree_index {
                    if tree_index.is_valid() {
                        let _blocker = QSignalBlocker::from_q_object(dependencies_ui.dependencies_tree_view().static_upcast::<QObject>());
                        dependencies_ui.dependencies_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                        dependencies_ui.dependencies_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                    }
                }
                DataSource::AssKitFiles
            },
        };

        AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_owned()), false, false, data_source);

        if is_match {
            if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == data_source).find(|x| *x.path_read() == path) {
                match file_view.view_type() {

                    // If it's a anim fragment battle file, open and select the matched value.
                    ViewType::Internal(View::AnimFragmentBattle(view)) => {
                        let parent = gidhora.parent();
                        let bool_data = parent.child_2a(model_index.row(), 0).data_1a(ANIM_FRAGMENT_BATTLE_BOOL_DATA).to_u_int_0a();

                        if bool_data == 1 {
                            view.skeleton_name_line_edit().select_all();
                            view.skeleton_name_line_edit().set_focus_0a();
                        } else if bool_data == 2 {
                            view.table_name_line_edit().select_all();
                            view.table_name_line_edit().set_focus_0a();
                        } else if bool_data == 3 {
                            view.mount_table_name_line_edit().select_all();
                            view.mount_table_name_line_edit().set_focus_0a();
                        } else if bool_data == 4 {
                            view.unmount_table_name_line_edit().select_all();
                            view.unmount_table_name_line_edit().set_focus_0a();
                        } else if bool_data == 5 {
                            view.locomotion_graph_line_edit().select_all();
                            view.locomotion_graph_line_edit().set_focus_0a();
                        } else {
                            let entry_index = parent.child_2a(model_index.row(), 0).data_1a(ANIM_FRAGMENT_BATTLE_ENTRY_INDEX).to_u_int_0a();
                            let column = if bool_data == 6 || bool_data == 7 || bool_data == 8 {
                                9
                            } else if bool_data == 9 {
                                11
                            } else if bool_data == 10 {
                                12
                            } else if bool_data == 11 {
                                13
                            } else if bool_data == 12 {
                                14
                            } else if bool_data == 13 {
                                16
                            } else {
                                return;
                            };

                            let item_to_select = view.table().table_model().index_2a(entry_index as i32, column);
                            let item_to_select_filter = view.table().table_filter().map_from_source(&item_to_select);

                            let selection = view.table().table_view().selection_model().selection();
                            view.table().table_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                            view.table().table_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                            view.table().table_view().set_focus_0a();
                            view.table().table_view().set_current_index(item_to_select_filter.as_ref());
                            view.table().table_view().scroll_to_2a(item_to_select_filter.as_ref(), ScrollHint::EnsureVisible);

                            // If it's a subtable, we also have to open said subtable and select the item in question.
                            if bool_data == 6 || bool_data == 7 || bool_data == 8 {
                                let subentry_index = parent.child_2a(model_index.row(), 0).data_1a(ANIM_FRAGMENT_BATTLE_SUBENTRY_INDEX).to_u_int_0a();
                                let column = if bool_data == 6 {
                                    0
                                } else if bool_data == 7 {
                                    1
                                } else if bool_data == 8 {
                                    2
                                } else {
                                    return;
                                };

                                open_subtable(item_to_select_filter.as_ref(), view.table(), app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, Some((subentry_index as i32, column)));
                            }
                        }
                    }

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    ViewType::Internal(View::Table(view)) => {
                        let parent = gidhora.parent();
                        let table_view = view.get_ref_table();
                        let table_view = table_view.table_view_ptr();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        let row = parent.child_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap() - 1;
                        let column = parent.child_2a(model_index.row(), 3).text().to_std_string().parse::<i32>().unwrap();

                        let table_model_index = table_model.index_2a(row, column);
                        let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                        if table_model_index_filtered.is_valid() {
                            table_view.set_focus_0a();
                            table_view.set_current_index(table_model_index_filtered.as_ref());
                            table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                            table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::ClearAndSelect));
                        }
                    }

                    // If it's a text file, scroll to the row in question.
                    ViewType::Internal(View::Text(view)) => {
                        let parent = gidhora.parent();
                        let row_number = parent.child_2a(model_index.row(), 2).text().to_std_string().parse::<i32>().unwrap() - 1;
                        let editor = view.get_mut_editor();
                        scroll_to_row_safe(&editor.as_ptr(), row_number.try_into().unwrap());
                    }

                    // If it's a portrait settings file, open and select the matched value.
                    ViewType::Internal(View::PortraitSettings(view)) => {
                        let parent = gidhora.parent();
                        let entry_index = parent.child_2a(model_index.row(), 0).data_1a(PORTRAIT_SETTINGS_ENTRY_INDEX).to_u_int_0a();

                        let item_to_select = view.main_list_model().index_2a(entry_index as i32, 0);
                        let item_to_select_filter = view.main_list_filter().map_from_source(&item_to_select);

                        // This view uses selection to trigger loads and saves. By changing selection we're forcing the view
                        // to load the item we want without breaking anything related to saving.
                        let selection = view.main_list_view().selection_model().selection();
                        view.main_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                        view.main_list_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                        let bool_data = parent.child_2a(model_index.row(), 0).data_1a(PORTRAIT_SETTINGS_BOOL_DATA).to_u_int_0a();

                        if bool_data == 1 {
                            view.main_list_view().set_focus_0a();
                        } else if bool_data == 2 {
                            view.head_skeleton_node_line_edit().select_all();
                            view.head_skeleton_node_line_edit().set_focus_0a();
                        } else if bool_data == 3 {
                            view.body_skeleton_node_line_edit().select_all();
                            view.body_skeleton_node_line_edit().set_focus_0a();
                        } else {
                            let variant_index = parent.child_2a(model_index.row(), 0).data_1a(PORTRAIT_SETTINGS_VARIANT_INDEX).to_u_int_0a();

                            let item_to_select = view.variants_list_model().index_2a(variant_index as i32, 0);
                            let item_to_select_filter = view.variants_list_filter().map_from_source(&item_to_select);

                            // Same as with the other list
                            let selection = view.variants_list_view().selection_model().selection();
                            view.variants_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                            view.variants_list_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                            if bool_data == 4 {
                                view.variants_list_view().set_focus_0a();
                            } else if bool_data == 5 {
                                view.file_diffuse_line_edit().select_all();
                                view.file_diffuse_line_edit().set_focus_0a();
                            } else if bool_data == 6 {
                                view.file_mask_1_line_edit().select_all();
                                view.file_mask_1_line_edit().set_focus_0a();
                            } else if bool_data == 7 {
                                view.file_mask_2_line_edit().select_all();
                                view.file_mask_2_line_edit().set_focus_0a();
                            } else if bool_data == 8 {
                                view.file_mask_3_line_edit().select_all();
                                view.file_mask_3_line_edit().set_focus_0a();
                            }
                        }
                    }

                    // If it's a unit_variant file, open and select the matched value.
                    ViewType::Internal(View::UnitVariant(view)) => {
                        let parent = gidhora.parent();
                        let entry_index = parent.child_2a(model_index.row(), 0).data_1a(UNIT_VARIANT_ENTRY_INDEX).to_u_int_0a();
                        let item_to_select = view.main_list_model().index_2a(entry_index as i32, 0);
                        let item_to_select_filter = view.main_list_filter().map_from_source(&item_to_select);

                        // This view uses selection to trigger loads and saves. By changing selection we're forcing the view
                        // to load the item we want without breaking anything related to saving.
                        let selection = view.main_list_view().selection_model().selection();
                        view.main_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                        view.main_list_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                        let bool_data = parent.child_2a(model_index.row(), 0).data_1a(UNIT_VARIANT_BOOL_DATA).to_u_int_0a();

                        if bool_data == 1 {
                            view.main_list_view().set_focus_0a();
                        } else {
                            let variant_index = parent.child_2a(model_index.row(), 0).data_1a(UNIT_VARIANT_VARIANT_INDEX).to_u_int_0a();

                            let item_to_select = view.variants_list_model().index_2a(variant_index as i32, 0);
                            let item_to_select_filter = view.variants_list_filter().map_from_source(&item_to_select);

                            // Same as with the other list
                            let selection = view.variants_list_view().selection_model().selection();
                            view.variants_list_view().selection_model().select_q_item_selection_q_flags_selection_flag(&selection, SelectionFlag::Toggle.into());
                            view.variants_list_view().selection_model().select_q_model_index_q_flags_selection_flag(&item_to_select_filter, SelectionFlag::Toggle.into());

                            if bool_data == 2 {
                                view.mesh_file_line_edit().select_all();
                                view.mesh_file_line_edit().set_focus_0a();
                            } else if bool_data == 3 {
                                view.texture_folder_line_edit().select_all();
                                view.texture_folder_line_edit().set_focus_0a();
                            }
                        }
                    }

                    // Ignore the rest.
                    _ => {},
                }
            }
        }
    }


    /// This function takes care of loading the results of a global search of `AnimFragmentBattleMatches` into a model.
    unsafe fn load_anim_fragment_battle_matches_to_ui(&self, matches: &[AnimFragmentBattleMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_afb| !match_afb.matches().is_empty())
                .map(|match_afb| {
                    let path = match_afb.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();
                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_afb.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = Self::new_item();
                        let match_type = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));

                        // Store the data needed to pin-point the match in the file in the text item.
                        let bool_data = if *match_row.skeleton_name() {
                            1
                        }else if *match_row.table_name() {
                            2
                        } else if *match_row.mount_table_name() {
                            3
                        } else if *match_row.unmount_table_name() {
                            4
                        } else if *match_row.locomotion_graph() {
                            5
                        } else if let Some(entry) = match_row.entry() {
                            text.set_data_2a(&QVariant::from_uint(entry.0 as u32), ANIM_FRAGMENT_BATTLE_ENTRY_INDEX);
                            if let Some(subentry) = entry.1 {
                                text.set_data_2a(&QVariant::from_uint(subentry.0 as u32), ANIM_FRAGMENT_BATTLE_SUBENTRY_INDEX);
                                if subentry.1 {
                                    6
                                } else if subentry.2 {
                                    7
                                } else if subentry.3 {
                                    8
                                } else {
                                    panic!()
                                }

                            } else if entry.2 {
                                9
                            } else if entry.3 {
                                10
                            } else if entry.4 {
                                11
                            } else if entry.5 {
                                12
                            } else if entry.6 {
                                13
                            } else {
                                panic!()
                            }
                        } else {
                            panic!()
                        };

                        text.set_data_2a(&QVariant::from_uint(bool_data), ANIM_FRAGMENT_BATTLE_BOOL_DATA);

                        let string = match bool_data {
                            1 => qtr("anim_fragment_battle_skeleton_name"),
                            2 => qtr("anim_fragment_battle_table_name"),
                            3 => qtr("anim_fragment_battle_mount_table_name"),
                            4 => qtr("anim_fragment_battle_unmount_table_name"),
                            5 => qtr("anim_fragment_battle_locomotion_graph"),
                            6 => qtr("anim_fragment_battle_file_path"),
                            7 => qtr("anim_fragment_battle_meta_file_path"),
                            8 => qtr("anim_fragment_battle_snd_file_path"),
                            9 => qtr("anim_fragment_battle_filename"),
                            10 => qtr("anim_fragment_battle_metadata"),
                            11 => qtr("anim_fragment_battle_metadata_sound"),
                            12 => qtr("anim_fragment_battle_skeleton_type"),
                            13 => qtr("anim_fragment_battle_uk_4"),
                            _ => QString::new(),
                        };

                        match_type.set_text(&string);

                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&match_type.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `AtlasMatches` into a model.
    unsafe fn load_atlas_matches_to_ui(&self, matches: &[AtlasMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_atlas| !match_atlas.matches().is_empty())
                .map(|match_atlas| {
                    let path = match_atlas.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();
                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_atlas.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        let text = Self::new_item();
                        let column_name = Self::new_item();
                        let row = Self::new_item();
                        let column_number = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));
                        column_name.set_text(&QString::from_std_str(match_row.column_name()));
                        row.set_data_2a(&QVariant::from_i64(match_row.row_number() + 1), 2);
                        column_number.set_data_2a(&QVariant::from_uint(*match_row.column_number()), 2);
                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_number.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `PortraitSettingsMatches` into a model.
    unsafe fn load_portrait_settings_matches_to_ui(&self, matches: &[PortraitSettingsMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_ps| !match_ps.matches().is_empty())
                .map(|match_ps| {
                    let path = match_ps.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();
                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_ps.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = Self::new_item();
                        let match_type = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));

                        // Store the data needed to pin-point the match in the file in the text item.
                        let bool_data = if *match_row.id() {
                            1
                        }else if *match_row.camera_settings_head() {
                            2
                        } else if *match_row.camera_settings_body() {
                            3
                        } else if let Some(variant) = match_row.variant() {
                            if variant.1 {
                                4
                            } else if variant.2 {
                                5
                            } else if variant.3 {
                                6
                            } else if variant.4 {
                                7
                            } else if variant.5 {
                                8
                            } else {
                                panic!()
                            }
                        } else {
                            panic!()
                        };

                        text.set_data_2a(&QVariant::from_uint(*match_row.entry() as u32), PORTRAIT_SETTINGS_ENTRY_INDEX);
                        text.set_data_2a(&QVariant::from_uint(bool_data), PORTRAIT_SETTINGS_BOOL_DATA);

                        if bool_data > 3 {
                            if let Some(variant) = match_row.variant() {
                                text.set_data_2a(&QVariant::from_uint(variant.0 as u32), PORTRAIT_SETTINGS_VARIANT_INDEX);
                            }
                        }

                        let string = match bool_data {
                            1 => qtr("portrait_settings_id"),
                            2 => qtr("portrait_settings_head_skeleton_node"),
                            3 => qtr("portrait_settings_body_skeleton_node"),
                            4 => qtr("portrait_settings_file_name"),
                            5 => qtr("portrait_settings_file_diffuse_label"),
                            6 => qtr("portrait_settings_file_mask_1_label"),
                            7 => qtr("portrait_settings_file_mask_2_label"),
                            8 => qtr("portrait_settings_file_mask_3_label"),
                            _ => QString::new(),
                        };

                        // Fix for translations ending in :.
                        let chara = QChar::from_uchar(b':');
                        if string.ends_with_q_char(&chara) {
                            string.remove_q_char(&chara);
                        }

                        match_type.set_text(&string);

                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&match_type.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `RigidModelMatches` into a model.
    unsafe fn load_rigid_model_matches_to_ui(&self, matches: &[RigidModelMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_unk| !match_unk.matches().is_empty())
                .map(|match_unk| {
                    let path = match_unk.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_unk.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let pos_formatted = Self::new_item();
                        let pos = Self::new_item();
                        let len = Self::new_item();

                        pos_formatted.set_text(&QString::from_std_str(&format!("0x{:0>8X}", *match_row.pos())));
                        pos.set_data_2a(&QVariant::from_u64(*match_row.pos() as u64), 2);
                        len.set_data_2a(&QVariant::from_u64(*match_row.len() as u64), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&pos_formatted.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&pos.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&len.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TableMatches` into a model.
    unsafe fn load_table_matches_to_ui(&self, matches: &[TableMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_table| !match_table.matches().is_empty())
                .map(|match_table| {
                    let path = match_table.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_table.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = Self::new_item();
                        let column_name = Self::new_item();
                        let row = Self::new_item();
                        let column_number = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));
                        column_name.set_text(&QString::from_std_str(match_row.column_name()));
                        row.set_data_2a(&QVariant::from_i64(match_row.row_number() + 1), 2);
                        column_number.set_data_2a(&QVariant::from_uint(*match_row.column_number()), 2);
                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&column_number.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `TextMatches` into a model.
    unsafe fn load_text_matches_to_ui(&self, matches: &[TextMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_text| !match_text.matches().is_empty())
                .map(|match_text| {
                    let path = match_text.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_text.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = Self::new_item();
                        let row = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));
                        row.set_data_2a(&QVariant::from_u64(match_row.row() + 1), 2);
                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&row.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `UnitVariantMatches` into a model.
    unsafe fn load_unit_variant_matches_to_ui(&self, matches: &[UnitVariantMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_uv| !match_uv.matches().is_empty())
                .map(|match_uv| {
                    let path = match_uv.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();
                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_uv.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let text = Self::new_item();
                        let match_type = Self::new_item();
                        let start = Self::new_item();
                        let end = Self::new_item();

                        text.set_text(&QString::from_std_str(Self::format_search_match(match_row.text(), *match_row.start(), *match_row.end())));

                        // Store the data needed to pin-point the match in the file in the text item.
                        let bool_data = if *match_row.name() {
                            1
                        } else if let Some(variant) = match_row.variant() {
                            if variant.1 {
                                2
                            } else if variant.2 {
                                3
                            } else {
                                panic!()
                            }
                        } else {
                            panic!()
                        };

                        text.set_data_2a(&QVariant::from_uint(*match_row.entry() as u32), UNIT_VARIANT_ENTRY_INDEX);
                        text.set_data_2a(&QVariant::from_uint(bool_data), UNIT_VARIANT_BOOL_DATA);

                        if bool_data > 1 {
                            if let Some(variant) = match_row.variant() {
                                text.set_data_2a(&QVariant::from_uint(variant.0 as u32), UNIT_VARIANT_VARIANT_INDEX);
                            }
                        }

                        let string = match bool_data {
                            1 => qtr("unit_variant_name"),
                            2 => qtr("unit_variant_mesh_file"),
                            3 => qtr("unit_variant_texture_folder"),
                            _ => QString::new(),
                        };

                        // Fix for translations ending in :.
                        let chara = QChar::from_uchar(b':');
                        if string.ends_with_q_char(&chara) {
                            string.remove_q_char(&chara);
                        }

                        match_type.set_text(&string);

                        start.set_data_2a(&QVariant::from_uint(*match_row.start() as u32), 2);
                        end.set_data_2a(&QVariant::from_uint(*match_row.end() as u32), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&text.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&match_type.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&start.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&end.into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `UnknownMatches` into a model.
    unsafe fn load_unknown_matches_to_ui(&self, matches: &[UnknownMatches], file_type: FileType) {
        let model = &self.matches_table_and_text_tree_model;

        if !matches.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added. It reduce add times on 200 ms, depending on the case.
            model.block_signals(true);

            let file_type_item = Self::new_item();
            file_type_item.set_text(&QString::from_std_str::<String>(From::from(file_type)));
            let file_type_item = atomic_from_cpp_box(file_type_item);

            let rows = matches.par_iter()
                .filter(|match_unk| !match_unk.matches().is_empty())
                .map(|match_unk| {
                    let path = match_unk.path();
                    let qlist_daddy = QListOfQStandardItem::new();
                    let file = Self::new_item();

                    file.set_text(&QString::from_std_str(path));
                    TREEVIEW_ICONS.set_standard_item_icon(&file, Some(&file_type));

                    for match_row in match_unk.matches() {

                        // Create a new list of StandardItem.
                        let qlist_boi = QListOfQStandardItem::new();

                        // Create an empty row.
                        let pos_formatted = Self::new_item();
                        let pos = Self::new_item();
                        let len = Self::new_item();

                        pos_formatted.set_text(&QString::from_std_str(&format!("0x{:0>8X}", *match_row.pos())));
                        pos.set_data_2a(&QVariant::from_u64(*match_row.pos() as u64), 2);
                        len.set_data_2a(&QVariant::from_u64(*match_row.len() as u64), 2);

                        // Add an empty row to the list.
                        qlist_boi.append_q_standard_item(&pos_formatted.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&pos.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&len.into_ptr().as_mut_raw_ptr());
                        qlist_boi.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());

                        // Append the new row.
                        file.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                    }

                    qlist_daddy.append_q_standard_item(&file.into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&Self::new_item().into_ptr().as_mut_raw_ptr());
                    qlist_daddy.append_q_standard_item(&((*ptr_from_atomic(&file_type_item)).clone()).as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist_daddy)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// This function takes care of loading the results of a global search of `SchemaMatches` into a model.
    unsafe fn load_schema_matches_to_ui(&self, matches: &SchemaMatches) {
        let model = &self.matches_schema_tree_model;

        if !matches.matches().is_empty() {
            model.block_signals(true);

            let rows = matches.matches()
                .par_iter()
                .map(|match_schema| {
                    let qlist = QListOfQStandardItem::new();
                    let table_name = Self::new_item();
                    let version = Self::new_item();
                    let column_name = Self::new_item();
                    let column = Self::new_item();

                    table_name.set_text(&QString::from_std_str(match_schema.table_name()));
                    version.set_data_2a(&QVariant::from_int(*match_schema.version()), 2);
                    column_name.set_text(&QString::from_std_str(match_schema.column_name()));
                    column.set_data_2a(&QVariant::from_uint(*match_schema.column()), 2);

                    qlist.append_q_standard_item(&table_name.into_ptr().as_mut_raw_ptr());
                    qlist.append_q_standard_item(&version.into_ptr().as_mut_raw_ptr());
                    qlist.append_q_standard_item(&column_name.into_ptr().as_mut_raw_ptr());
                    qlist.append_q_standard_item(&column.into_ptr().as_mut_raw_ptr());
                    atomic_from_cpp_box(qlist)
                })
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    model.block_signals(false);
                }

                model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));
            }
        }
    }

    /// Function to filter the PackFile Contents TreeView.
    pub unsafe fn filter_results(
        view: &QPtr<QTreeView>,
        line_edit: &QPtr<QLineEdit>,
        column_combobox: &QPtr<QComboBox>,
        case_sensitive_button: &QPtr<QToolButton>,
    ) {

        let pattern = QRegExp::new_1a(&line_edit.text());

        let case_sensitive = case_sensitive_button.is_checked();
        if case_sensitive { pattern.set_case_sensitivity(CaseSensitivity::CaseSensitive); }
        else { pattern.set_case_sensitivity(CaseSensitivity::CaseInsensitive); }

        let model_filter: QPtr<QSortFilterProxyModel> = view.model().static_downcast();
        model_filter.set_filter_key_column(column_combobox.current_index());
        trigger_treeview_filter_safe(&model_filter, &pattern.as_ptr());
    }

    /// Function to get all the selected matches in the visible selection.
    unsafe fn matches_from_selection(&self) -> Vec<MatchHolder> {

        let (model, tree_view) = match self.matches_tab_widget.current_index() {
            0 => (&self.matches_table_and_text_tree_model, &self.matches_table_and_text_tree_view),
            _ => return vec![],
        };

        let items = tree_view.get_items_from_selection(true);

        let anim_matches: Vec<UnknownMatches> = vec![];
        let mut anim_fragment_battle_matches: Vec<AnimFragmentBattleMatches> = vec![];
        let anim_pack_matches: Vec<UnknownMatches> = vec![];
        let anims_table_matches: Vec<UnknownMatches> = vec![];
        let mut atlas_matches: Vec<AtlasMatches> = vec![];
        let audio_matches: Vec<UnknownMatches> = vec![];
        let bmd_matches: Vec<UnknownMatches> = vec![];
        let mut db_matches: Vec<TableMatches> = vec![];
        let esf_matches: Vec<UnknownMatches> = vec![];
        let group_formations_matches: Vec<UnknownMatches> = vec![];
        let image_matches: Vec<UnknownMatches> = vec![];
        let mut loc_matches: Vec<TableMatches> = vec![];
        let matched_combat_matches: Vec<UnknownMatches> = vec![];
        let pack_matches: Vec<UnknownMatches> = vec![];
        let mut portrait_settings_matches: Vec<PortraitSettingsMatches> = vec![];
        let mut rigid_model_matches: Vec<RigidModelMatches> = vec![];
        let sound_bank_matches: Vec<UnknownMatches> = vec![];
        let mut text_matches: Vec<TextMatches> = vec![];
        let uic_matches: Vec<UnknownMatches> = vec![];
        let mut unit_variant_matches: Vec<UnitVariantMatches> = vec![];
        let mut unknown_matches: Vec<UnknownMatches> = vec![];
        let video_matches: Vec<UnknownMatches> = vec![];

        // For each item we follow the following logic:
        // - If it's a parent, it's all the matches on a table.
        // - If it's a child, check if the parent already exists.
        // - If it does, add another entry to it's matches.
        // - If not, create it with only that match.
        for item in items {
            if item.column() == 0 {
                let is_match = !item.has_children();

                // If it's a match (not an entire file), get the entry and add it to the tablematches of that table.
                if is_match {
                    let parent = item.parent();
                    let path = parent.text().to_std_string();
                    let file_type_index = parent.index().sibling_at_column(5);
                    let file_type = FileType::from(&*model.item_from_index(&file_type_index).text().to_std_string());

                    match file_type {
                        FileType::Anim => todo!(),
                        FileType::AnimFragmentBattle => {
                            let item = parent.child_2a(item.row(), 0);
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();
                            let entry_index = item.data_1a(ANIM_FRAGMENT_BATTLE_ENTRY_INDEX).to_u_int_0a() as usize;
                            let subentry_index = item.data_1a(ANIM_FRAGMENT_BATTLE_SUBENTRY_INDEX).to_u_int_0a() as usize;
                            let bool_data = item.data_1a(ANIM_FRAGMENT_BATTLE_BOOL_DATA).to_u_int_0a();

                            let match_file = match anim_fragment_battle_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let matches = AnimFragmentBattleMatches::new(&path);
                                    anim_fragment_battle_matches.push(matches);
                                    anim_fragment_battle_matches.last_mut().unwrap()
                                }
                            };


                            let match_entry = AnimFragmentBattleMatch::new(
                                bool_data == 1,
                                bool_data == 2,
                                bool_data == 3,
                                bool_data == 4,
                                bool_data == 5,
                                if bool_data > 5 {
                                    Some((
                                        entry_index,
                                        if bool_data > 5 && bool_data < 9 {
                                            Some((
                                                subentry_index,
                                                bool_data == 6,
                                                bool_data == 7,
                                                bool_data == 8,
                                            ))
                                        } else {
                                            None
                                        },
                                        bool_data == 9,
                                        bool_data == 10,
                                        bool_data == 11,
                                        bool_data == 12,
                                        bool_data == 13
                                    ))
                                } else {
                                    None
                                },
                                start,
                                end,
                                item.text().to_std_string()
                            );

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::AnimPack => todo!(),
                        FileType::AnimsTable => todo!(),
                        FileType::Atlas => {
                            let column_name = parent.child_2a(item.row(), 1).text().to_std_string();
                            let column_number = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                            let row_number = parent.child_2a(item.row(), 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();
                            let text = parent.child_2a(item.row(), 0).text().to_std_string();
                            let match_file = match atlas_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let table = AtlasMatches::new(&path);
                                    atlas_matches.push(table);
                                    atlas_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = AtlasMatch::new(&column_name, column_number, row_number, start, end, &text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::Audio => todo!(),
                        FileType::BMD => todo!(),
                        FileType::BMDVegetation => todo!(),
                        FileType::DB => {
                            let column_name = parent.child_2a(item.row(), 1).text().to_std_string();
                            let column_number = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                            let row_number = parent.child_2a(item.row(), 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();
                            let text = parent.child_2a(item.row(), 0).text().to_std_string();
                            let match_file = match db_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let table = TableMatches::new(&path);
                                    db_matches.push(table);
                                    db_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = TableMatch::new(&column_name, column_number, row_number, start, end, &text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::ESF => todo!(),
                        FileType::GroupFormations => todo!(),
                        FileType::HlslCompiled => todo!(),
                        FileType::Image => todo!(),
                        FileType::Loc => {
                            let column_name = parent.child_2a(item.row(), 1).text().to_std_string();
                            let column_number = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                            let row_number = parent.child_2a(item.row(), 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();
                            let text = parent.child_2a(item.row(), 0).text().to_std_string();
                            let match_file = match loc_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let table = TableMatches::new(&path);
                                    loc_matches.push(table);
                                    loc_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = TableMatch::new(&column_name, column_number, row_number, start, end, &text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::MatchedCombat => todo!(),
                        FileType::Pack => todo!(),
                        FileType::PortraitSettings => {
                            let item = parent.child_2a(item.row(), 0);
                            let index = item.data_1a(PORTRAIT_SETTINGS_ENTRY_INDEX).to_u_int_0a() as usize;
                            let bool_data = item.data_1a(PORTRAIT_SETTINGS_BOOL_DATA).to_u_int_0a();
                            let vindex = item.data_1a(PORTRAIT_SETTINGS_VARIANT_INDEX).to_u_int_0a() as usize;
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();

                            let match_file = match portrait_settings_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let matches = PortraitSettingsMatches::new(&path);
                                    portrait_settings_matches.push(matches);
                                    portrait_settings_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = PortraitSettingsMatch::new(
                                index,
                                bool_data == 1,
                                bool_data == 2,
                                bool_data == 3,
                                if bool_data > 3 {
                                    Some((
                                        vindex,
                                        bool_data == 4,
                                        bool_data == 5,
                                        bool_data == 6,
                                        bool_data == 7,
                                        bool_data == 8
                                    ))
                                } else {
                                    None
                                },
                                start,
                                end,
                                item.text().to_std_string()
                            );

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::RigidModel => {
                            let pos = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                            let len = parent.child_2a(item.row(), 4).text().to_std_string().parse().unwrap();

                            let match_file = match rigid_model_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let matches = RigidModelMatches::new(&path);
                                    rigid_model_matches.push(matches);
                                    rigid_model_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = RigidModelMatch::new(pos, len);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::SoundBank => todo!(),
                        FileType::Text => {
                            let row_number = parent.child_2a(item.row(), 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                            let text = parent.child_2a(item.row(), 0).text().to_std_string();
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();

                            let match_file = match text_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let text = TextMatches::new(&path);
                                    text_matches.push(text);
                                    text_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = TextMatch::new(row_number as u64, start, end, text);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::UIC => todo!(),
                        FileType::UnitVariant => {
                            let item = parent.child_2a(item.row(), 0);
                            let index = item.data_1a(UNIT_VARIANT_ENTRY_INDEX).to_u_int_0a() as usize;
                            let bool_data = item.data_1a(UNIT_VARIANT_BOOL_DATA).to_u_int_0a();
                            let vindex = item.data_1a(UNIT_VARIANT_VARIANT_INDEX).to_u_int_0a() as usize;
                            let start = parent.child_2a(item.row(), 4).text().to_std_string().parse::<usize>().unwrap();
                            let end = parent.child_2a(item.row(), 5).text().to_std_string().parse::<usize>().unwrap();

                            let match_file = match unit_variant_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let matches = UnitVariantMatches::new(&path);
                                    unit_variant_matches.push(matches);
                                    unit_variant_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = UnitVariantMatch::new(
                                index,
                                bool_data == 1,
                                if bool_data > 1 {
                                    Some((
                                        vindex,
                                        bool_data == 2,
                                        bool_data == 3,
                                    ))
                                } else {
                                    None
                                },
                                start,
                                end,
                                item.text().to_std_string()
                            );

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::Unknown => {
                            let pos = parent.child_2a(item.row(), 3).text().to_std_string().parse().unwrap();
                            let len = parent.child_2a(item.row(), 4).text().to_std_string().parse().unwrap();

                            let match_file = match unknown_matches.iter_mut().find(|x| x.path() == &path) {
                                Some(match_file) => match_file,
                                None => {
                                    let matches = UnknownMatches::new(&path);
                                    unknown_matches.push(matches);
                                    unknown_matches.last_mut().unwrap()
                                }
                            };

                            let match_entry = UnknownMatch::new(pos, len);

                            if !match_file.matches_mut().contains(&match_entry) {
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::Video => todo!(),
                    }
                }

                // If it's not a particular match, it's an entire file.
                else {
                    let path = item.text().to_std_string();
                    let file_type_index = item.index().sibling_at_column(5);
                    let file_type = FileType::from(&*model.item_from_index(&file_type_index).text().to_std_string());

                    // If it already exists, delete it, as the new one contains the entire set for it.
                    match file_type {
                        FileType::Anim => todo!(),
                        FileType::AnimFragmentBattle => {
                            if let Some(position) = anim_fragment_battle_matches.iter().position(|x| x.path() == &path) {
                                anim_fragment_battle_matches.remove(position);
                            }

                            let matches = AnimFragmentBattleMatches::new(&path);
                            anim_fragment_battle_matches.push(matches);
                            let match_file = anim_fragment_battle_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();

                                let item = item.child_2a(row, 0);
                                let entry_index = item.data_1a(ANIM_FRAGMENT_BATTLE_ENTRY_INDEX).to_u_int_0a() as usize;
                                let subentry_index = item.data_1a(ANIM_FRAGMENT_BATTLE_SUBENTRY_INDEX).to_u_int_0a() as usize;
                                let bool_data = item.data_1a(ANIM_FRAGMENT_BATTLE_BOOL_DATA).to_u_int_0a();

                                let match_entry = AnimFragmentBattleMatch::new(
                                    bool_data == 1,
                                    bool_data == 2,
                                    bool_data == 3,
                                    bool_data == 4,
                                    bool_data == 5,
                                    if bool_data > 5 {
                                        Some((
                                            entry_index,
                                            if bool_data > 5 && bool_data < 9 {
                                                Some((
                                                    subentry_index,
                                                    bool_data == 6,
                                                    bool_data == 7,
                                                    bool_data == 8,
                                                ))
                                            } else {
                                                None
                                            },
                                            bool_data == 9,
                                            bool_data == 10,
                                            bool_data == 11,
                                            bool_data == 12,
                                            bool_data == 13
                                        ))
                                    } else {
                                        None
                                    },
                                    start,
                                    end,
                                    item.text().to_std_string()
                                );

                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::AnimPack => todo!(),
                        FileType::AnimsTable => todo!(),
                        FileType::Atlas => {
                            if let Some(position) = atlas_matches.iter().position(|x| x.path() == &path) {
                                atlas_matches.remove(position);
                            }

                            let table = AtlasMatches::new(&path);
                            atlas_matches.push(table);
                            let match_file = atlas_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let column_name = item.child_2a(row, 1).text().to_std_string();
                                let column_number = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let match_entry = AtlasMatch::new(&column_name, column_number, row_number, start, end, &text);
                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::Audio => todo!(),
                        FileType::BMD => todo!(),
                        FileType::BMDVegetation => todo!(),
                        FileType::DB => {
                            if let Some(position) = db_matches.iter().position(|x| x.path() == &path) {
                                db_matches.remove(position);
                            }

                            let table = TableMatches::new(&path);
                            db_matches.push(table);
                            let match_file = db_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let column_name = item.child_2a(row, 1).text().to_std_string();
                                let column_number = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let match_entry = TableMatch::new(&column_name, column_number, row_number, start, end, &text);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::ESF => todo!(),
                        FileType::GroupFormations => todo!(),
                        FileType::HlslCompiled => todo!(),
                        FileType::Image => todo!(),
                        FileType::Loc => {
                            if let Some(position) = loc_matches.iter().position(|x| x.path() == &path) {
                                loc_matches.remove(position);
                            }

                            let table = TableMatches::new(&path);
                            loc_matches.push(table);
                            let match_file = loc_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let column_name = item.child_2a(row, 1).text().to_std_string();
                                let column_number = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let match_entry = TableMatch::new(&column_name, column_number, row_number, start, end, &text);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::MatchedCombat => todo!(),
                        FileType::Pack => todo!(),
                        FileType::PortraitSettings => {
                            if let Some(position) = portrait_settings_matches.iter().position(|x| x.path() == &path) {
                                portrait_settings_matches.remove(position);
                            }

                            let matches = PortraitSettingsMatches::new(&path);
                            portrait_settings_matches.push(matches);
                            let match_file = portrait_settings_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();

                                let item = item.child_2a(row, 0);
                                let index = item.data_1a(PORTRAIT_SETTINGS_ENTRY_INDEX).to_u_int_0a() as usize;
                                let bool_data = item.data_1a(PORTRAIT_SETTINGS_BOOL_DATA).to_u_int_0a();
                                let vindex = item.data_1a(PORTRAIT_SETTINGS_VARIANT_INDEX).to_u_int_0a() as usize;

                                let match_entry = PortraitSettingsMatch::new(
                                    index,
                                    bool_data == 1,
                                    bool_data == 2,
                                    bool_data == 3,
                                    if bool_data > 3 {
                                        Some((
                                            vindex,
                                            bool_data == 4,
                                            bool_data == 5,
                                            bool_data == 6,
                                            bool_data == 7,
                                            bool_data == 8
                                        ))
                                    } else {
                                        None
                                    },
                                    start,
                                    end,
                                    item.text().to_std_string()
                                );

                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::RigidModel => {
                            if let Some(position) = rigid_model_matches.iter().position(|x| x.path() == &path) {
                                rigid_model_matches.remove(position);
                            }

                            let text = RigidModelMatches::new(&path);
                            rigid_model_matches.push(text);
                            let match_file = rigid_model_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let pos = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let len = item.child_2a(row, 4).text().to_std_string().parse().unwrap();
                                let match_entry = RigidModelMatch::new(pos, len);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::SoundBank => todo!(),
                        FileType::Text => {
                            if let Some(position) = text_matches.iter().position(|x| x.path() == &path) {
                                text_matches.remove(position);
                            }

                            let text = TextMatches::new(&path);
                            text_matches.push(text);
                            let match_file = text_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let row_number = item.child_2a(row, 2).text().to_std_string().parse::<i64>().unwrap() - 1;
                                let text = item.child_2a(row, 0).text().to_std_string();
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();
                                let match_entry = TextMatch::new(row_number as u64, start, end, text);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::UIC => todo!(),
                        FileType::UnitVariant => {
                            if let Some(position) = unit_variant_matches.iter().position(|x| x.path() == &path) {
                                unit_variant_matches.remove(position);
                            }

                            let matches = UnitVariantMatches::new(&path);
                            unit_variant_matches.push(matches);
                            let match_file = unit_variant_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let item = item.child_2a(row, 0);
                                let index = item.data_1a(UNIT_VARIANT_ENTRY_INDEX).to_u_int_0a() as usize;
                                let bool_data = item.data_1a(UNIT_VARIANT_BOOL_DATA).to_u_int_0a();
                                let vindex = item.data_1a(UNIT_VARIANT_VARIANT_INDEX).to_u_int_0a() as usize;
                                let start = item.child_2a(row, 4).text().to_std_string().parse::<usize>().unwrap();
                                let end = item.child_2a(row, 5).text().to_std_string().parse::<usize>().unwrap();

                                let match_entry = UnitVariantMatch::new(
                                    index,
                                    bool_data == 1,
                                    if bool_data > 1 {
                                        Some((
                                            vindex,
                                            bool_data == 2,
                                            bool_data == 3,
                                        ))
                                    } else {
                                        None
                                    },
                                    start,
                                    end,
                                    item.text().to_std_string()
                                );

                                match_file.matches_mut().push(match_entry);
                            }
                        },
                        FileType::Unknown => {
                            if let Some(position) = unknown_matches.iter().position(|x| x.path() == &path) {
                                unknown_matches.remove(position);
                            }

                            let text = UnknownMatches::new(&path);
                            unknown_matches.push(text);
                            let match_file = unknown_matches.last_mut().unwrap();

                            // For the individual matches, we have to get them from the view, so the filtered out items are not added.
                            for row in 0..item.row_count() {
                                let pos = item.child_2a(row, 3).text().to_std_string().parse().unwrap();
                                let len = item.child_2a(row, 4).text().to_std_string().parse().unwrap();
                                let match_entry = UnknownMatch::new(pos, len);
                                match_file.matches_mut().push(match_entry);
                            }
                        }
                        FileType::Video => todo!(),
                    }
                }
            }
        }

        let mut matches = vec![];

        matches.append(&mut anim_matches.into_iter().map(MatchHolder::Anim).collect::<Vec<_>>());
        matches.append(&mut anim_fragment_battle_matches.into_iter().map(MatchHolder::AnimFragmentBattle).collect::<Vec<_>>());
        matches.append(&mut anim_pack_matches.into_iter().map(MatchHolder::AnimPack).collect::<Vec<_>>());
        matches.append(&mut anims_table_matches.into_iter().map(MatchHolder::AnimsTable).collect::<Vec<_>>());
        matches.append(&mut atlas_matches.into_iter().map(MatchHolder::Atlas).collect::<Vec<_>>());
        matches.append(&mut audio_matches.into_iter().map(MatchHolder::Audio).collect::<Vec<_>>());
        matches.append(&mut bmd_matches.into_iter().map(MatchHolder::Bmd).collect::<Vec<_>>());
        matches.append(&mut db_matches.into_iter().map(MatchHolder::Db).collect::<Vec<_>>());
        matches.append(&mut esf_matches.into_iter().map(MatchHolder::Esf).collect::<Vec<_>>());
        matches.append(&mut group_formations_matches.into_iter().map(MatchHolder::GroupFormations).collect::<Vec<_>>());
        matches.append(&mut image_matches.into_iter().map(MatchHolder::Image).collect::<Vec<_>>());
        matches.append(&mut loc_matches.into_iter().map(MatchHolder::Loc).collect::<Vec<_>>());
        matches.append(&mut matched_combat_matches.into_iter().map(MatchHolder::MatchedCombat).collect::<Vec<_>>());
        matches.append(&mut pack_matches.into_iter().map(MatchHolder::Pack).collect::<Vec<_>>());
        matches.append(&mut portrait_settings_matches.into_iter().map(MatchHolder::PortraitSettings).collect::<Vec<_>>());
        matches.append(&mut rigid_model_matches.into_iter().map(MatchHolder::RigidModel).collect::<Vec<_>>());
        matches.append(&mut sound_bank_matches.into_iter().map(MatchHolder::SoundBank).collect::<Vec<_>>());
        matches.append(&mut text_matches.into_iter().map(MatchHolder::Text).collect::<Vec<_>>());
        matches.append(&mut uic_matches.into_iter().map(MatchHolder::Uic).collect::<Vec<_>>());
        matches.append(&mut unit_variant_matches.into_iter().map(MatchHolder::UnitVariant).collect::<Vec<_>>());
        matches.append(&mut unknown_matches.into_iter().map(MatchHolder::Unknown).collect::<Vec<_>>());
        matches.append(&mut video_matches.into_iter().map(MatchHolder::Video).collect::<Vec<_>>());

        matches
    }

    pub unsafe fn search_data_from_ui(&self, reset_data: bool, is_replace: bool) -> Option<GlobalSearch> {

        // Create the global search and populate it with all the settings for the search.
        let mut global_search = if reset_data {
            GlobalSearch::default()
        } else {
            UI_STATE.get_global_search()
        };

        global_search.set_pattern(self.search_line_edit.text().to_std_string());
        global_search.set_case_sensitive(self.case_sensitive_checkbox.is_checked());
        global_search.set_use_regex(self.use_regex_checkbox.is_checked());

        if is_replace {
            global_search.set_replace_text(self.replace_line_edit.text().to_std_string());
        }

        // If we don't have text to search, return.
        if global_search.pattern().is_empty() {
            return None;
        }

        if self.search_source_packfile.is_checked() {
            global_search.set_source(SearchSource::Pack);
        } else if self.search_source_parent.is_checked() {
            global_search.set_source(SearchSource::ParentFiles);
        } else if self.search_source_game.is_checked() {
            global_search.set_source(SearchSource::GameFiles);
        } else if self.search_source_asskit.is_checked() {
            global_search.set_source(SearchSource::AssKitFiles);
        }

        if self.search_on_all_checkbox.is_checked() {
            global_search.search_on_mut().set_anim(true);
            global_search.search_on_mut().set_anim_fragment_battle(true);
            global_search.search_on_mut().set_anim_pack(true);
            global_search.search_on_mut().set_anims_table(true);
            global_search.search_on_mut().set_atlas(true);
            global_search.search_on_mut().set_audio(true);
            global_search.search_on_mut().set_bmd(true);
            global_search.search_on_mut().set_db(true);
            global_search.search_on_mut().set_esf(true);
            global_search.search_on_mut().set_group_formations(true);
            global_search.search_on_mut().set_image(true);
            global_search.search_on_mut().set_loc(true);
            global_search.search_on_mut().set_matched_combat(true);
            global_search.search_on_mut().set_pack(true);
            global_search.search_on_mut().set_portrait_settings(true);
            global_search.search_on_mut().set_rigid_model(true);
            global_search.search_on_mut().set_schema(true);
            global_search.search_on_mut().set_sound_bank(true);
            global_search.search_on_mut().set_text(true);
            global_search.search_on_mut().set_uic(true);
            global_search.search_on_mut().set_unit_variant(true);
            global_search.search_on_mut().set_unknown(true);
            global_search.search_on_mut().set_video(true);
        }

        else if self.search_on_all_common_checkbox.is_checked() {
            global_search.search_on_mut().set_anim(false);
            global_search.search_on_mut().set_anim_fragment_battle(false);
            global_search.search_on_mut().set_anim_pack(false);
            global_search.search_on_mut().set_anims_table(false);
            global_search.search_on_mut().set_atlas(false);
            global_search.search_on_mut().set_audio(false);
            global_search.search_on_mut().set_bmd(false);
            global_search.search_on_mut().set_db(true);
            global_search.search_on_mut().set_esf(false);
            global_search.search_on_mut().set_group_formations(false);
            global_search.search_on_mut().set_image(false);
            global_search.search_on_mut().set_loc(true);
            global_search.search_on_mut().set_matched_combat(false);
            global_search.search_on_mut().set_pack(false);
            global_search.search_on_mut().set_portrait_settings(false);
            global_search.search_on_mut().set_rigid_model(false);
            global_search.search_on_mut().set_schema(false);
            global_search.search_on_mut().set_sound_bank(false);
            global_search.search_on_mut().set_text(true);
            global_search.search_on_mut().set_uic(false);
            global_search.search_on_mut().set_unit_variant(false);
            global_search.search_on_mut().set_unknown(false);
            global_search.search_on_mut().set_video(false);

        } else {
            global_search.search_on_mut().set_anim(self.search_on_anim_checkbox.is_checked());
            global_search.search_on_mut().set_anim_fragment_battle(self.search_on_anim_fragment_battle_checkbox.is_checked());
            global_search.search_on_mut().set_anim_pack(self.search_on_anim_pack_checkbox.is_checked());
            global_search.search_on_mut().set_anims_table(self.search_on_anims_table_checkbox.is_checked());
            global_search.search_on_mut().set_atlas(self.search_on_atlas_checkbox.is_checked());
            global_search.search_on_mut().set_audio(self.search_on_audio_checkbox.is_checked());
            global_search.search_on_mut().set_bmd(self.search_on_bmd_checkbox.is_checked());
            global_search.search_on_mut().set_db(self.search_on_db_checkbox.is_checked());
            global_search.search_on_mut().set_esf(self.search_on_esf_checkbox.is_checked());
            global_search.search_on_mut().set_group_formations(self.search_on_group_formations_checkbox.is_checked());
            global_search.search_on_mut().set_image(self.search_on_image_checkbox.is_checked());
            global_search.search_on_mut().set_loc(self.search_on_loc_checkbox.is_checked());
            global_search.search_on_mut().set_matched_combat(self.search_on_matched_combat_checkbox.is_checked());
            global_search.search_on_mut().set_pack(self.search_on_pack_checkbox.is_checked());
            global_search.search_on_mut().set_portrait_settings(self.search_on_portrait_settings_checkbox.is_checked());
            global_search.search_on_mut().set_rigid_model(self.search_on_rigid_model_checkbox.is_checked());
            global_search.search_on_mut().set_schema(self.search_on_schemas_checkbox.is_checked());
            global_search.search_on_mut().set_sound_bank(self.search_on_sound_bank_checkbox.is_checked());
            global_search.search_on_mut().set_text(self.search_on_text_checkbox.is_checked());
            global_search.search_on_mut().set_uic(self.search_on_uic_checkbox.is_checked());
            global_search.search_on_mut().set_unit_variant(self.search_on_unit_variant_checkbox.is_checked());
            global_search.search_on_mut().set_unknown(self.search_on_unknown_checkbox.is_checked());
            global_search.search_on_mut().set_video(self.search_on_video_checkbox.is_checked());
        }

        Some(global_search)
    }

    pub unsafe fn build_trees(&self) {

        // Clear the current results panels.
        self.matches_table_and_text_tree_model.clear();
        self.matches_schema_tree_model.clear();

        // Optimisation: Setting the column counts allows us to configure the columns before loading the data.
        self.matches_table_and_text_tree_model.set_column_count(6);
        self.matches_schema_tree_model.set_column_count(4);

        // Tweak the table columns for the files tree here, instead on each load function.
        self.matches_table_and_text_tree_model.block_signals(true);
        self.matches_table_and_text_tree_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_match_packedfile_column")));
        self.matches_table_and_text_tree_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column_name")));
        self.matches_table_and_text_tree_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_row")));
        self.matches_table_and_text_tree_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_column")));
        self.matches_table_and_text_tree_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("gen_loc_length")));
        self.matches_table_and_text_tree_model.block_signals(false);

        // Hide the column number column for tables.
        self.matches_table_and_text_tree_view.hide_column(3);
        self.matches_table_and_text_tree_view.hide_column(4);
        self.matches_table_and_text_tree_view.hide_column(5);
        self.matches_table_and_text_tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);
        self.matches_table_and_text_tree_view.set_column_width(0, 300);
        self.matches_table_and_text_tree_view.set_column_width(1, 200);
        self.matches_table_and_text_tree_view.set_column_width(2, 20);

        // Show row before column for where it's relevant. Otherwise row tends to be hidden.
        self.matches_table_and_text_tree_view.header().move_section(2, 1);

        // Same for the schema matches list.
        self.matches_schema_tree_model.block_signals(true);
        self.matches_schema_tree_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_table_name")));
        self.matches_schema_tree_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_version")));
        self.matches_schema_tree_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column_name")));
        self.matches_schema_tree_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("global_search_column")));
        self.matches_schema_tree_model.block_signals(false);

        // Hide the column number column for tables.
        self.matches_schema_tree_view.hide_column(3);
        self.matches_schema_tree_view.sort_by_column_2a(0, SortOrder::AscendingOrder);
        self.matches_schema_tree_view.set_column_width(0, 300);
        self.matches_schema_tree_view.set_column_width(1, 20);
        self.matches_schema_tree_view.set_column_width(2, 300);

        //new_search_match_item_delegate_safe(&self.matches_table_and_text_tree_view.static_upcast::<QObject>().as_ptr(), 0);
    }

    unsafe fn new_item() -> CppBox<QStandardItem> {
        let item = QStandardItem::new();
        item.set_editable(false);
        item
    }

    unsafe fn format_search_match(text: &str, start: usize, end: usize) -> String {

        // Trim the text so only the part with the match shows up.
        let text_trimmed_start = if start >= 20 {
            (true, closest_valid_char_byte(text, start - 20))
        } else {
            (false, 0)
        };

        let text_trimmed_end = if text.len() >= 20 && end < text.len() - 20 {
            (true, closest_valid_char_byte(text, end + 16))
        } else {
            (false, text.len())
        };

        format!("{}{}{}",
            if text_trimmed_start.0 { "..." } else { "" },
            text[text_trimmed_start.1..text_trimmed_end.1].trim(),
            if text_trimmed_end.0 { "..." } else { "" }
        )
    }
}
