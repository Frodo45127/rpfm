//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//!This module contains the code to build/use the ***Settings*** UI.

use qt_core::QByteArray;
use qt_core::QEasingCurve;
use qt_core::QObject;
use qt_core::QPoint;
use qt_core::QPropertyAnimation;
use qt_core::q_easing_curve;
use qt_widgets::QCheckBox;
use qt_widgets::QVBoxLayout;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::{QDialogButtonBox, q_dialog_button_box, q_dialog_button_box::ButtonRole};
use qt_widgets::{QFileDialog, q_file_dialog::{FileMode, Option as QFileDialogOption}};
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QSpinBox;
use qt_widgets::QPushButton;
use qt_widgets::QScrollArea;
use qt_widgets::QWidget;

use qt_gui::{QColor, q_color::NameFormat};
use qt_gui::{QPalette, q_palette::ColorRole};

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::SlotNoArgs;
use qt_core::QPtr;
use qt_core::QSettings;
use qt_core::QString;
use qt_core::QVariant;

use cpp_core::CastInto;
use cpp_core::Ptr;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use getset::Getters;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

use rpfm_ipc::settings_keys::*;

use rpfm_lib::games::supported_games::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::locale::Locale;
use rpfm_ui_common::tools::{Tool, Tools};
use rpfm_ui_common::utils::create_grid_layout;

use crate::app_ui::AppUI;
use crate::ffi::*;
use crate::SUPPORTED_GAMES;
use crate::settings_ui::backend::{config_path, settings_get_all, settings_set_bool, settings_set_i32, settings_set_string};
use crate::updater_ui::{BETA, STABLE, update_channel, UpdateChannel};
use crate::utils::{tr, qtr, qtre};

use self::slots::SettingsUISlots;

pub mod backend;
mod connections;
mod slots;

/// Helper macro to create a self-contained setting row: label on the left (stretching),
/// control widget on the right. Each row is its own QWidget so settings don't share
/// column alignment with each other.
macro_rules! setting_row {
    ($vbox:expr, $parent:expr, $label:expr, $control:expr) => {{
        let row = QWidget::new_1a($parent);
        let lay = create_grid_layout(row.static_upcast());
        lay.set_contents_margins_4a(0, 6, 0, 2);
        lay.set_vertical_spacing(4);
        lay.set_column_stretch(0, 1);
        lay.add_widget_5a(&$label, 0, 0, 1, 1);
        lay.add_widget_5a(&$control, 0, 1, 1, 1);
        let sep = qt_widgets::QFrame::new_1a(&row);
        sep.set_frame_shape(qt_widgets::q_frame::Shape::HLine);
        sep.set_frame_shadow(qt_widgets::q_frame::Shadow::Sunken);
        lay.add_widget_5a(&sep, 1, 0, 1, 2);
        $vbox.add_widget_1a(&row);
    }};
    // 3-column variant for path rows: label, line_edit (stretch), button.
    ($vbox:expr, $parent:expr, $label:expr, $line_edit:expr, $button:expr) => {{
        let row = QWidget::new_1a($parent);
        let lay = create_grid_layout(row.static_upcast());
        lay.set_contents_margins_4a(0, 6, 0, 2);
        lay.set_vertical_spacing(4);
        lay.set_column_stretch(1, 1);
        lay.add_widget_5a(&$label, 0, 0, 1, 1);
        lay.add_widget_5a(&$line_edit, 0, 1, 1, 1);
        lay.add_widget_5a(&$button, 0, 2, 1, 1);
        let sep = qt_widgets::QFrame::new_1a(&row);
        sep.set_frame_shape(qt_widgets::q_frame::Shape::HLine);
        sep.set_frame_shadow(qt_widgets::q_frame::Shadow::Sunken);
        lay.add_widget_5a(&sep, 1, 0, 1, 3);
        $vbox.add_widget_1a(&row);
    }};
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the widgets used in the Settings Window.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SettingsUI {

    //-------------------------------------------------------------------------------//
    // `Dialog` window.
    //-------------------------------------------------------------------------------//
    dialog: QBox<QDialog>,

    //-------------------------------------------------------------------------------//
    // `Path` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    paths_mymod_line_edit: QBox<QLineEdit>,
    paths_mymod_button: QBox<QPushButton>,
    paths_secondary_line_edit: QBox<QLineEdit>,
    paths_secondary_button: QBox<QPushButton>,

    paths_spoilers: BTreeMap<String, QBox<QWidget>>,

    paths_games_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    paths_games_buttons: BTreeMap<String, QBox<QPushButton>>,

    paths_asskit_line_edits: BTreeMap<String, QBox<QLineEdit>>,
    paths_asskit_buttons: BTreeMap<String, QBox<QPushButton>>,

    //-------------------------------------------------------------------------------//
    // Settings widgets, keyed by their backend settings key.
    //-------------------------------------------------------------------------------//
    general_language_combobox: QBox<QComboBox>,
    extra_global_default_game_combobox: QBox<QComboBox>,
    extra_network_update_channel_combobox: QBox<QComboBox>,
    extra_packfile_autosave_interval_spinbox: QBox<QSpinBox>,
    extra_packfile_autosave_amount_spinbox: QBox<QSpinBox>,
    ai_openai_api_key_line_edit: QBox<QLineEdit>,
    deepl_api_key_line_edit: QBox<QLineEdit>,
    font_data: Rc<RefCell<(String, i32)>>,

    /// All boolean settings checkboxes, keyed by their backend settings key.
    checkboxes: BTreeMap<String, QBox<QCheckBox>>,

    /// All colour picker buttons, keyed by their QSettings colour key.
    colour_buttons: BTreeMap<String, QBox<QPushButton>>,

    //-------------------------------------------------------------------------------//
    // Action buttons (not settings themselves).
    //-------------------------------------------------------------------------------//
    debug_clear_dependencies_cache_folder_button: QBox<QPushButton>,
    debug_clear_autosave_folder_button: QBox<QPushButton>,
    debug_clear_schema_folder_button: QBox<QPushButton>,
    debug_clear_layout_settings_button: QBox<QPushButton>,
    debug_add_rpfm_to_runcher_tools_button: QBox<QPushButton>,

    //-------------------------------------------------------------------------------//
    // `ButtonBox` section of the `Settings` dialog.
    //-------------------------------------------------------------------------------//
    button_box_restore_default_button: QPtr<QPushButton>,
    button_box_text_editor_settings_button: QBox<QPushButton>,
    button_box_shortcuts_button: QBox<QPushButton>,
    button_box_font_settings_button: QBox<QPushButton>,
    button_box_cancel_button: QPtr<QPushButton>,
    button_box_accept_button: QPtr<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `SettingsUI`.
impl SettingsUI {

    /// This function creates a ***Settings*** dialog, execute it, and returns a new `Settings`, or `None` if you close/cancel the dialog.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<bool> {
        let settings_ui = Self::new_with_parent(app_ui.main_window())?;
        let settings_ui = Rc::new(settings_ui);
        let slots = SettingsUISlots::new(&settings_ui, app_ui);

        connections::set_connections(&settings_ui, &slots);

        // If load fails due to missing locale folder, show the error and cancel the settings edition.
        settings_ui.load()?;
        if settings_ui.dialog.exec() == 1 {
            settings_ui.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// This function creates a new `SettingsUI` and links it to the provided parent.
    unsafe fn new_with_parent(parent: impl CastInto<Ptr<QWidget>>) -> Result<Self> {

        // Initialize and configure the settings window.
        let dialog = QDialog::new_1a(parent);
        dialog.set_window_title(&qtr("settings_title"));
        dialog.set_modal(true);
        dialog.resize_2a(900, 600);
        //dialog.set_attribute_1a(WidgetAttribute::WADeleteOnClose);

        let main_grid = create_grid_layout(dialog.static_upcast());
        main_grid.set_contents_margins_4a(4, 0, 4, 4);
        main_grid.set_spacing(4);

        // Search field at the top, spanning both columns.
        let search_field = QLineEdit::from_q_widget(&dialog);
        search_field.set_placeholder_text(&qtr("settings_search_placeholder"));
        search_field.set_clear_button_enabled(true);
        main_grid.add_widget_5a(&search_field, 0, 0, 1, 2);

        //-----------------------------------------------//
        // Left panel
        //-----------------------------------------------//
        let nav_widget = QWidget::new_1a(&dialog);
        let nav_layout = QVBoxLayout::new_1a(&nav_widget);
        nav_layout.set_contents_margins_4a(0, 0, 0, 0);
        nav_layout.set_spacing(2);
        nav_widget.set_fixed_width(160);

        let category_keys = [
            "settings_tab_paths",
            "settings_ui_title",
            "settings_table_title",
            "settings_debug_title",
            "settings_diagnostics_title",
            "settings_ai_title",
        ];
        let mut nav_buttons: Vec<QBox<QPushButton>> = Vec::new();
        for key in &category_keys {
            nav_buttons.push(new_category_button(&nav_widget, &nav_layout, key));
        }
        nav_layout.add_stretch_1a(1);

        // Select first category by default.
        if !nav_buttons.is_empty() {
            nav_buttons[0].set_checked(true);
        }

        main_grid.add_widget_5a(&nav_widget, 1, 0, 2, 1);

        //-----------------------------------------------//
        // Right panel
        //-----------------------------------------------//
        let scroll_area = QScrollArea::new_1a(&dialog);
        scroll_area.set_widget_resizable(true);
        scroll_area.set_frame_shape(qt_widgets::q_frame::Shape::NoFrame);

        let content_widget = QWidget::new_0a();
        let content_layout = QVBoxLayout::new_1a(&content_widget);
        content_layout.set_contents_margins_4a(4, 4, 4, 4);
        content_layout.set_spacing(8);
        scroll_area.set_widget(&content_widget);

        // Hint pinned above the scroll area (stays visible during scroll/filter).
        let hint_label = QLabel::from_q_string_q_widget(
            &QString::from_std_str(format!(
                "<span style='color: gray; font-style: italic;'>{}<br>{}</span>",
                tr("settings_hint_restart"),
                tr("settings_hint_game_switch"),
            )),
            &dialog,
        );
        hint_label.set_text_format(qt_core::TextFormat::RichText);
        hint_label.set_word_wrap(true);
        hint_label.set_contents_margins_4a(4, 2, 4, 2);
        main_grid.add_widget_5a(&hint_label, 1, 1, 1, 1);

        main_grid.add_widget_5a(&scroll_area, 2, 1, 1, 1);

        //-----------------------------------------------//
        // `Game Paths` Frame.
        //-----------------------------------------------//
        let paths_header = QLabel::from_q_string_q_widget(&qtr("settings_game_paths_title"), &dialog);
        paths_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let paths_description = QLabel::from_q_string_q_widget(&qtr("settings_game_paths_description"), &dialog);
        paths_description.set_word_wrap(true);
        paths_description.set_style_sheet(&QString::from_std_str("color: gray; font-style: italic; padding: 0 4px 4px 4px;"));
        let paths_frame = QWidget::new_1a(&dialog);
        let main_paths_grid = create_grid_layout(paths_frame.static_upcast());
        main_paths_grid.set_contents_margins_4a(4, 0, 4, 0);

        // We automatically add a Label/LineEdit/Button for each game we support.
        let mut paths_spoilers = BTreeMap::new();

        let mut paths_games_line_edits = BTreeMap::new();
        let mut paths_games_buttons = BTreeMap::new();

        let mut paths_asskit_line_edits = BTreeMap::new();
        let mut paths_asskit_buttons = BTreeMap::new();

        for (index, game_supported) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            let spoiler = new_spoiler_safe(&QString::from_std_str(game_supported.display_name()).as_ptr(), 200, &paths_frame.as_ptr().static_upcast());
            spoiler.set_accessible_name(&QString::from_std_str(game_supported.display_name()));

            // Note: ignore the warnings caused by this. They're harmless.
            let game_path_layout = create_grid_layout(spoiler.static_upcast());

            let game_key = game_supported.key();
            let (game_line_edit, game_button) = new_setting_grid_path(&game_path_layout, &spoiler, 0, "settings_game_label", "settings_game_line_ph", game_supported.display_name());

            // Add the LineEdit and Button to the list.
            paths_games_line_edits.insert(game_key.to_owned(), game_line_edit);
            paths_games_buttons.insert(game_key.to_owned(), game_button);

            if game_key != KEY_EMPIRE &&
                game_key != KEY_NAPOLEON &&
                game_key != KEY_ARENA {

                let (asskit_line_edit, asskit_button) = new_setting_grid_path(&game_path_layout, &spoiler, 1, "settings_asskit_label", "settings_asskit_line_ph", game_supported.display_name());

                // Add the LineEdit and Button to the list.
                paths_asskit_line_edits.insert(game_key.to_owned(), asskit_line_edit);
                paths_asskit_buttons.insert(game_key.to_owned(), asskit_button);
            }

            set_spoiler_layout_safe(&spoiler.as_ptr(), &game_path_layout.as_ptr().static_upcast());
            main_paths_grid.add_widget_5a(&spoiler, index as i32 + 1, 0, 1, 1);
            paths_spoilers.insert(game_key.to_owned(), spoiler);
        }

        content_layout.add_widget_1a(&paths_header);
        content_layout.add_widget_1a(&paths_description);
        content_layout.add_widget_1a(&paths_frame);

        //-----------------------------------------------//
        // `Extra Paths` Frame.
        //-----------------------------------------------//

        let extra_paths_header = QLabel::from_q_string_q_widget(&qtr("settings_extra_paths_title"), &dialog);
        extra_paths_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let extra_paths_description = QLabel::from_q_string_q_widget(&qtr("settings_extra_paths_description"), &dialog);
        extra_paths_description.set_word_wrap(true);
        extra_paths_description.set_style_sheet(&QString::from_std_str("color: gray; font-style: italic; padding: 0 4px 4px 4px;"));
        let extra_paths_frame = QWidget::new_1a(&dialog);
        let extra_paths_vbox = QVBoxLayout::new_1a(&extra_paths_frame);
        extra_paths_vbox.set_contents_margins_4a(4, 0, 4, 0);
        extra_paths_vbox.set_spacing(2);

        let (paths_mymod_line_edit, paths_mymod_button) = new_setting_path(&extra_paths_vbox, &extra_paths_frame, "settings_paths_mymod", "settings_paths_mymod_ph");
        let (paths_secondary_line_edit, paths_secondary_button) = new_setting_path(&extra_paths_vbox, &extra_paths_frame, "settings_paths_secondary", "settings_paths_secondary_ph");

        content_layout.add_widget_1a(&extra_paths_header);
        content_layout.add_widget_1a(&extra_paths_description);
        content_layout.add_widget_1a(&extra_paths_frame);

        //-----------------------------------------------//
        // `General` Frame.
        //-----------------------------------------------//
        let general_header = QLabel::from_q_string_q_widget(&qtr("settings_ui_title"), &dialog);
        general_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let general_frame = QWidget::new_1a(&dialog);
        let general_vbox = QVBoxLayout::new_1a(&general_frame);
        general_vbox.set_contents_margins_4a(4, 0, 4, 0);
        general_vbox.set_spacing(2);

        let locales = Locale::get_available_locales(&ASSETS_PATH.to_string_lossy()).unwrap_or_default();
        let locale_names = locales.iter().map(|(name, _)| &**name).collect::<Vec<_>>();
        let game_names = SUPPORTED_GAMES.games_sorted().iter().map(|x| *x.display_name()).collect::<Vec<_>>();
        let general_language_combobox = new_setting_combobox(&general_vbox, &general_frame, "settings_ui_language", "tt_settings_ui_language_tip", &locale_names);
        let extra_global_default_game_combobox = new_setting_combobox(&general_vbox, &general_frame, "settings_default_game", "tt_settings_default_game_tip", &game_names);
        let extra_network_update_channel_combobox = new_setting_combobox(&general_vbox, &general_frame, "settings_update_channel", "tt_settings_update_channel_tip", &[STABLE, BETA]);
        let extra_packfile_autosave_amount_spinbox = new_setting_spinbox(&general_vbox, &general_frame, "settings_autosave_amount", "tt_settings_autosave_amount");
        let extra_packfile_autosave_interval_spinbox = new_setting_spinbox(&general_vbox, &general_frame, "settings_autosave_interval", "tt_settings_autosave_interval_tip");
        let mut checkboxes = BTreeMap::new();

        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "check_updates_on_start", "settings_check_updates_on_start", "tt_extra_network_check_updates_on_start_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "check_schema_updates_on_start", "settings_check_schema_updates_on_start", "tt_extra_network_check_schema_updates_on_start_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "check_lua_autogen_updates_on_start", "settings_check_lua_autogen_updates_on_start", "tt_settings_check_lua_autogen_updates_on_start_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "check_old_ak_updates_on_start", "settings_check_old_ak_updates_on_start", "tt_settings_check_old_ak_updates_on_start_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "allow_editing_of_ca_packfiles", "settings_allow_editing_of_ca_packfiles", "tt_extra_packfile_allow_editing_of_ca_packfiles_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "disable_file_previews", "settings_disable_file_previews", "tt_settings_disable_file_previews_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "start_maximized", "settings_ui_window_start_maximized_label", "tt_ui_window_start_maximized_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "packfile_treeview_resize_to_fit", "settings_packfile_treeview_resize_to_fit", "");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "expand_treeview_when_adding_items", "settings_expand_treeview_when_adding_items", "settings_expand_treeview_when_adding_items_tip");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "include_base_folder_on_add_from_folder", "include_base_folder_on_add_from_folder", "settings_include_base_folder_on_add_from_folder");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "delete_empty_folders_on_delete", "delete_empty_folders_on_delete", "settings_delete_empty_folders_on_delete");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "ignore_game_files_in_ak", "ignore_game_files_in_ak", "settings_ignore_game_files_in_ak");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "enable_multifolder_filepicker", "enable_multifolder_filepicker", "settings_enable_multifolder_filepicker");
        new_setting_checkbox(&mut checkboxes, &general_vbox, &general_frame, "enable_pack_contents_drag_and_drop", "enable_pack_contents_drag_and_drop", "settings_enable_pack_contents_drag_and_drop");

        content_layout.add_widget_1a(&general_header);
        content_layout.add_widget_1a(&general_frame);

        //-----------------------------------------------//
        // `Table` Frame.
        //-----------------------------------------------//

        let table_header = QLabel::from_q_string_q_widget(&qtr("settings_table_title"), &dialog);
        table_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let ui_table_view_frame = QWidget::new_1a(&dialog);
        let ui_table_vbox = QVBoxLayout::new_1a(&ui_table_view_frame);
        ui_table_vbox.set_contents_margins_4a(4, 0, 4, 0);
        ui_table_vbox.set_spacing(2);

        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "adjust_columns_to_content", "settings_ui_table_adjust_columns_to_content", "tt_ui_table_adjust_columns_to_content_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "disable_combos_on_tables", "settings_ui_table_disable_combos", "tt_ui_table_disable_combos_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "extend_last_column_on_tables", "settings_ui_table_extend_last_column_label", "tt_ui_table_extend_last_column_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "tight_table_mode", "settings_ui_table_tight_table_mode_label", "tt_ui_table_tight_table_mode_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "table_resize_on_edit", "settings_table_resize_on_edit", "tt_settings_table_resize_on_edit_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "tables_use_old_column_order", "settings_ui_table_use_old_column_order_label", "tt_settings_tables_use_old_column_order_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "tables_use_old_column_order_for_tsv", "settings_ui_table_use_old_column_order_for_tsv_label", "tt_settings_tables_use_old_column_order_for_tsv_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "disable_uuid_regeneration_on_db_tables", "settings_disable_uuid_regeneration_tables", "tt_extra_disable_uuid_regeneration_on_db_tables_label_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "use_right_size_markers", "settings_use_right_side_markers", "tt_ui_table_use_right_side_markers_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "enable_lookups", "settings_enable_lookups", "tt_settings_enable_lookups_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "enable_icons", "settings_enable_icons", "tt_settings_enable_icons_tip");
        new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "enable_diff_markers", "settings_enable_diff_markers", "tt_settings_enable_diff_markers_tip");
        // new_setting_checkbox(&mut checkboxes, &ui_table_vbox, &ui_table_view_frame, "hide_unused_columns", "hide_unused_columns", "settings_hide_unused_columns");

        // Colour pairs.
        let mut colour_buttons = BTreeMap::new();
        new_setting_colour_pair(&mut colour_buttons, &ui_table_vbox, &ui_table_view_frame, "settings_colour_added", "tt_settings_colour_added", "colour_light_table_added", "colour_dark_table_added");
        new_setting_colour_pair(&mut colour_buttons, &ui_table_vbox, &ui_table_view_frame, "settings_colour_modified", "tt_settings_colour_modified", "colour_light_table_modified", "colour_dark_table_modified");
        new_setting_colour_pair(&mut colour_buttons, &ui_table_vbox, &ui_table_view_frame, "settings_colour_error", "tt_settings_colour_error", "colour_light_diagnostic_error", "colour_dark_diagnostic_error");
        new_setting_colour_pair(&mut colour_buttons, &ui_table_vbox, &ui_table_view_frame, "settings_colour_warning", "tt_settings_colour_warning", "colour_light_diagnostic_warning", "colour_dark_diagnostic_warning");
        new_setting_colour_pair(&mut colour_buttons, &ui_table_vbox, &ui_table_view_frame, "settings_colour_info", "tt_settings_colour_info", "colour_light_diagnostic_info", "colour_dark_diagnostic_info");

        content_layout.add_widget_1a(&table_header);
        content_layout.add_widget_1a(&ui_table_view_frame);

        //-----------------------------------------------//
        // `Debug` Frame.
        //-----------------------------------------------//
        let debug_header = QLabel::from_q_string_q_widget(&qtr("settings_debug_title"), &dialog);
        debug_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let debug_frame = QWidget::new_1a(&dialog);
        let debug_vbox = QVBoxLayout::new_1a(&debug_frame);
        debug_vbox.set_contents_margins_4a(4, 0, 4, 0);
        debug_vbox.set_spacing(2);

        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "check_for_missing_table_definitions", "settings_debug_missing_table", "tt_debug_check_for_missing_table_definitions_tip");
        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "enable_debug_menu", "settings_debug_enable_debug_menu", "tt_settings_enable_debug_menu_tip");
        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "enable_unit_editor", "settings_enable_unit_editor", "tt_settings_debug_enable_unit_editor");
        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "enable_esf_editor", "settings_enable_esf_editor", "tt_settings_enable_esf_editor_tip");
        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "use_debug_view_unit_variant", "settings_use_debug_view_unit_variant", "tt_settings_use_debug_view_unit_variant_tip");
        #[cfg(feature = "support_model_renderer")] new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "enable_renderer", "settings_enable_renderer", "tt_settings_enable_renderer_tip");
        new_setting_checkbox(&mut checkboxes, &debug_vbox, &debug_frame, "use_lazy_loading", "settings_use_lazy_loading", "tt_extra_packfile_use_lazy_loading_tip");

        // Buttons: text goes in a label (col 0), button gets a short action label (col 1).
        let debug_clear_dependencies_cache_folder_button = new_setting_button(&debug_vbox, &debug_frame, "settings_debug_clear_dependencies_cache_folder", "tt_settings_debug_clear_dependencies_cache_folder", "settings_action_clear");
        let debug_clear_autosave_folder_button = new_setting_button(&debug_vbox, &debug_frame, "settings_debug_clear_autosave_folder", "tt_settings_debug_clear_autosave_folder", "settings_action_clear");
        let debug_clear_schema_folder_button = new_setting_button(&debug_vbox, &debug_frame, "settings_debug_clear_schema_folder", "tt_settings_debug_clear_schema_folder", "settings_action_clear");
        let debug_clear_layout_settings_button = new_setting_button(&debug_vbox, &debug_frame, "settings_debug_clear_layout_settings", "tt_settings_debug_clear_layout_settings", "settings_action_clear");
        let debug_add_rpfm_to_runcher_tools_button = new_setting_button(&debug_vbox, &debug_frame, "settings_add_rpfm_to_runcher_tools", "tt_settings_add_rpfm_to_runcher_tools_tip", "settings_action_add");

        content_layout.add_widget_1a(&debug_header);
        content_layout.add_widget_1a(&debug_frame);

        //-----------------------------------------------//
        // `Diagnostics` Frame.
        //-----------------------------------------------//
        let diagnostics_header = QLabel::from_q_string_q_widget(&qtr("settings_diagnostics_title"), &dialog);
        diagnostics_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let diagnostics_frame = QWidget::new_1a(&dialog);
        let diagnostics_vbox = QVBoxLayout::new_1a(&diagnostics_frame);
        diagnostics_vbox.set_contents_margins_4a(4, 0, 4, 0);
        diagnostics_vbox.set_spacing(2);

        new_setting_checkbox(&mut checkboxes, &diagnostics_vbox, &diagnostics_frame, "diagnostics_trigger_on_open", "settings_diagnostics_trigger_on_open", "tt_diagnostics_trigger_diagnostics_on_open_tip");
        new_setting_checkbox(&mut checkboxes, &diagnostics_vbox, &diagnostics_frame, "diagnostics_trigger_on_table_edit", "settings_diagnostics_trigger_on_edit", "tt_diagnostics_trigger_diagnostics_on_table_edit_tip");

        content_layout.add_widget_1a(&diagnostics_header);
        content_layout.add_widget_1a(&diagnostics_frame);

        //-------------------------------------------------------------------------------//
        // `AI` section of the `Settings` dialog.
        //-------------------------------------------------------------------------------//
        let ai_header = QLabel::from_q_string_q_widget(&qtr("settings_ai_title"), &dialog);
        ai_header.set_style_sheet(&QString::from_std_str("font-weight: bold; font-size: 13px; padding: 8px 0 4px 0;"));
        let ai_frame = QWidget::new_1a(&dialog);
        let ai_vbox = QVBoxLayout::new_1a(&ai_frame);
        ai_vbox.set_contents_margins_4a(4, 0, 4, 0);
        ai_vbox.set_spacing(2);

        let ai_openai_api_key_line_edit = new_setting_line_edit(&ai_vbox, &ai_frame, "settings_ai_openai_api_key", "tt_ai_openai_api_key_tip");
        let deepl_api_key_line_edit = new_setting_line_edit(&ai_vbox, &ai_frame, "settings_deepl_api_key", "tt_deepl_api_key_tip");

        content_layout.add_widget_1a(&ai_header);
        content_layout.add_widget_1a(&ai_frame);

        // Add stretch at the end so frames don't expand vertically.
        content_layout.add_stretch_1a(1);

        // Collect category headers (for scroll navigation) and sections (for filtering).
        // Order must match nav_buttons: [Paths, General, Table, Debug, Diagnostics, AI]
        let category_headers: Vec<QBox<QLabel>> = vec![
            paths_header,
            general_header,
            table_header,
            debug_header,
            diagnostics_header,
            ai_header,
        ];
        // Note: extra_paths is a sub-section of Paths, stored separately for the extra_paths_header.
        let category_sections: Vec<QBox<QWidget>> = vec![
            paths_frame,  // Paths category (game paths with spoilers)
            general_frame,
            ui_table_view_frame,
            debug_frame,
            diagnostics_frame,
            ai_frame,
        ];

        // Wire nav buttons to smooth-scroll to corresponding category header.
        for (i, btn) in nav_buttons.iter().enumerate() {
            let scroll_area_ptr = scroll_area.as_ptr();
            let header_ptr = QPtr::from_raw(category_headers[i].as_mut_raw_ptr());
            let all_nav_ptrs: Vec<QPtr<QPushButton>> = nav_buttons.iter().map(|b| QPtr::from_raw(b.as_mut_raw_ptr())).collect();
            let slot = SlotNoArgs::new(btn, move || {
                // Calculate target scroll position: the header's y position relative to the scroll content.
                let target_y = header_ptr.map_to_parent_q_point(&QPoint::new_2a(0, 0)).y();
                let scrollbar = scroll_area_ptr.vertical_scroll_bar();

                // Animate the scrollbar to the target position.
                let anim = QPropertyAnimation::new_3a(
                    scrollbar.static_upcast::<QObject>(),
                    &QByteArray::from_slice(b"value"),
                    scrollbar.static_upcast::<QObject>(),
                );
                anim.set_duration(750);
                anim.set_start_value(&QVariant::from_int(scrollbar.value()));
                anim.set_end_value(&QVariant::from_int(target_y));

                let easing = QEasingCurve::new_0a();
                easing.set_type(q_easing_curve::Type::OutQuint);
                anim.set_easing_curve(&easing);
                anim.start_0a();

                for (j, nav_ptr) in all_nav_ptrs.iter().enumerate() {
                    nav_ptr.set_checked(j == i);
                }
            });
            btn.released().connect(&slot);
        }

        // Wire search field to filter categories and settings.
        {
            let header_ptrs: Vec<QPtr<QLabel>> = category_headers.iter().map(|h| QPtr::from_raw(h.as_mut_raw_ptr())).collect();
            let section_ptrs: Vec<QPtr<QWidget>> = category_sections.iter().map(|s| QPtr::from_raw(s.as_mut_raw_ptr())).collect();
            let nav_buttons_ptrs: Vec<QPtr<QPushButton>> = nav_buttons.iter().map(|b| QPtr::from_raw(b.as_mut_raw_ptr())).collect();
            // Also track the extra paths_frame + header (index 0 is extra_paths, mapped to same nav as paths).
            let extra_paths_header_ptr = QPtr::<QLabel>::from_raw(extra_paths_header.as_mut_raw_ptr());
            let extra_paths_section_ptr = QPtr::<QWidget>::from_raw(extra_paths_frame.as_mut_raw_ptr());

            let timer = qt_core::QTimer::new_1a(&dialog);
            timer.set_single_shot(true);
            timer.set_interval(200);

            let search_field_ptr = search_field.as_ptr();
            let timer_ptr = timer.as_ptr();
            let filter_slot = SlotNoArgs::new(&timer, move || {
                let filter_text = search_field_ptr.text().to_std_string().to_lowercase();

                for (idx, (header, section)) in header_ptrs.iter().zip(section_ptrs.iter()).enumerate() {

                    if filter_text.is_empty() {
                        header.show();
                        section.show();
                        nav_buttons_ptrs[idx].show();

                        let layout = section.layout();
                        for i in 0..layout.count() {
                            let item = layout.item_at(i);
                            if !item.is_null() {
                                let widget = item.widget();
                                if !widget.is_null() {
                                    widget.show();
                                }
                            }
                        }
                        continue;
                    }

                    let title = header.text().to_std_string().to_lowercase();
                    let title_matches = title.contains(&filter_text);

                    // Each section uses either:
                    // - A QVBoxLayout of setting_row widgets (General, Table, Debug, etc.)
                    // - A QGridLayout of spoiler widgets (Game Paths)
                    // For each child, collect searchable text from both the widget itself
                    // and its children (labels inside row layouts).
                    let layout = section.layout();
                    let mut any_child_visible = false;
                    for i in 0..layout.count() {
                        let item = layout.item_at(i);
                        if item.is_null() { continue; }
                        let row_widget = item.widget();
                        if row_widget.is_null() { continue; }

                        // Start with the widget's own searchable properties (handles spoilers
                        // which store the game name as accessible_name).
                        let mut row_text = String::new();
                        let self_name = row_widget.accessible_name().to_std_string().to_lowercase();
                        let self_tooltip = row_widget.tool_tip().to_std_string().to_lowercase();
                        row_text.push_str(&format!("{self_name} {self_tooltip} "));

                        // Then check children (labels, controls inside setting_row grids).
                        let row_layout = row_widget.layout();
                        if !row_layout.is_null() {
                            for j in 0..row_layout.count() {
                                let child_item = row_layout.item_at(j);
                                if child_item.is_null() { continue; }
                                let child = child_item.widget();
                                if child.is_null() { continue; }
                                let name = child.object_name().to_std_string().to_lowercase();
                                let accessible = child.accessible_name().to_std_string().to_lowercase();
                                let tooltip = child.tool_tip().to_std_string().to_lowercase();
                                let description = child.accessible_description().to_std_string().to_lowercase();
                                row_text.push_str(&format!("{name} {accessible} {tooltip} {description} "));
                            }
                        }

                        let visible = title_matches || row_text.contains(&filter_text);
                        row_widget.set_visible(visible);
                        if visible {
                            any_child_visible = true;
                        }
                    }

                    let visible = title_matches || any_child_visible;
                    header.set_visible(visible);
                    section.set_visible(visible);
                    nav_buttons_ptrs[idx].set_visible(visible);
                }

                // The extra paths section follows the same visibility as the Paths category (index 0).
                let paths_visible = nav_buttons_ptrs[0].is_visible();
                extra_paths_header_ptr.set_visible(paths_visible);
                extra_paths_section_ptr.set_visible(paths_visible);
            });
            timer.timeout().connect(&filter_slot);

            let trigger_slot = SlotNoArgs::new(&search_field, move || {
                timer_ptr.start_0a();
            });
            search_field.text_changed().connect(&trigger_slot);
        }

        //-----------------------------------------------//
        // `ButtonBox` Button Box.
        //-----------------------------------------------//
        let button_box = QDialogButtonBox::from_q_widget(&dialog);
        let button_box_shortcuts_button = QPushButton::from_q_string_q_widget(&qtr("shortcut_title"), &button_box);
        let button_box_text_editor_settings_button = QPushButton::from_q_string_q_widget(&qtr("settings_text_title"), &button_box);
        let button_box_font_settings_button = QPushButton::from_q_string_q_widget(&qtr("settings_font_title"), &button_box);

        let button_box_restore_default_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::RestoreDefaults);
        button_box.add_button_q_abstract_button_button_role(&button_box_shortcuts_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&button_box_text_editor_settings_button, ButtonRole::ResetRole);
        button_box.add_button_q_abstract_button_button_role(&button_box_font_settings_button, ButtonRole::ResetRole);
        let button_box_cancel_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Cancel);
        let button_box_accept_button = button_box.add_button_standard_button(q_dialog_button_box::StandardButton::Save);

        main_grid.add_widget_5a(&button_box, 3, 0, 1, 2);

        // Now, we build the `SettingsUI` struct and return it.
        Ok(Self {

            //-------------------------------------------------------------------------------//
            // `Dialog` window.
            //-------------------------------------------------------------------------------//
            dialog,

            //-------------------------------------------------------------------------------//
            // `Path` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            paths_mymod_line_edit,
            paths_mymod_button,
            paths_secondary_line_edit,
            paths_secondary_button,
            paths_spoilers,
            paths_games_line_edits,
            paths_games_buttons,
            paths_asskit_line_edits,
            paths_asskit_buttons,

            //-------------------------------------------------------------------------------//
            // Settings widgets.
            //-------------------------------------------------------------------------------//
            general_language_combobox,
            extra_global_default_game_combobox,
            extra_network_update_channel_combobox,
            extra_packfile_autosave_amount_spinbox,
            extra_packfile_autosave_interval_spinbox,
            ai_openai_api_key_line_edit,
            deepl_api_key_line_edit,
            font_data: Rc::new(RefCell::new((String::new(), -1))),
            checkboxes,
            colour_buttons,

            //-------------------------------------------------------------------------------//
            // Action buttons.
            //-------------------------------------------------------------------------------//
            debug_clear_dependencies_cache_folder_button,
            debug_clear_autosave_folder_button,
            debug_clear_schema_folder_button,
            debug_clear_layout_settings_button,
            debug_add_rpfm_to_runcher_tools_button,

            //-------------------------------------------------------------------------------//
            // `ButtonBox` section of the `Settings` dialog.
            //-------------------------------------------------------------------------------//
            button_box_restore_default_button,
            button_box_text_editor_settings_button,
            button_box_shortcuts_button,
            button_box_font_settings_button,
            button_box_cancel_button,
            button_box_accept_button,
        })
    }

    /// This function loads the data from the provided `Settings` into our `SettingsUI`.
    pub unsafe fn load(&self) -> Result<()> {

        // Fetch all settings in a single IPC call.
        let settings = settings_get_all();

        let get_str = |key: &str| settings.string.get(key).cloned().unwrap_or_default();
        let get_int = |key: &str| settings.i32.get(key).copied().unwrap_or_default();
        let get_bool = |key: &str| settings.bool.get(key).copied().unwrap_or_default();

        // Load the MyMod and secondary paths.
        self.paths_mymod_line_edit.set_text(&QString::from_std_str(get_str(MYMOD_BASE_PATH)));
        self.paths_secondary_line_edit.set_text(&QString::from_std_str(get_str(SECONDARY_PATH)));

        // Load the Game Paths.
        for (key, path) in self.paths_games_line_edits.iter() {
            if let Some(spoiler) = self.paths_spoilers.get(key) {
                let stored_path = get_str(key);
                if !stored_path.is_empty() {
                    path.set_text(&QString::from_std_str(&stored_path));
                    toggle_animated_safe(&spoiler.as_ptr());
                }
            }
        }

        for (key, path) in self.paths_asskit_line_edits.iter() {
            path.set_text(&QString::from_std_str(get_str(&(key.to_owned() + "_assembly_kit"))));
        }

        // Get the default game.
        let default_game = get_str(DEFAULT_GAME);
        for (index, game) in SUPPORTED_GAMES.games_sorted().iter().enumerate() {
            if game.key() == default_game {
                self.extra_global_default_game_combobox.set_current_index(index as i32);
                break;
            }
        }

        let language_selected = get_str(LANGUAGE);
        let language_selected_split = language_selected.split('_').collect::<Vec<&str>>()[0];
        for (index, (language,_)) in Locale::get_available_locales(&ASSETS_PATH.to_string_lossy())?.iter().enumerate() {
            if *language == language_selected_split {
                self.general_language_combobox.set_current_index(index as i32);
                break;
            }
        }

        for (index, update_channel_name) in [UpdateChannel::Stable, UpdateChannel::Beta].iter().enumerate() {
            if update_channel_name == &update_channel() {
                self.extra_network_update_channel_combobox.set_current_index(index as i32);
                break;
            }
        }

        *self.font_data.borrow_mut() = (get_str(FONT_NAME), get_int(FONT_SIZE));

        // Load spinboxes.
        self.extra_packfile_autosave_amount_spinbox.set_value(get_int(AUTOSAVE_AMOUNT));
        self.extra_packfile_autosave_interval_spinbox.set_value(get_int(AUTOSAVE_INTERVAL));

        // Load all checkboxes from their backend keys.
        for (key, checkbox) in &self.checkboxes {
            checkbox.set_checked(get_bool(key));
        }

        // Load AI keys.
        self.ai_openai_api_key_line_edit.set_text(&QString::from_std_str(get_str(AI_OPENAI_API_KEY)));
        self.deepl_api_key_line_edit.set_text(&QString::from_std_str(get_str(DEEPL_API_KEY)));

        // Load colours.
        let q_settings = QSettings::new();
        for (key, button) in &self.colour_buttons {
            let colour = QColor::from_q_string(&q_settings.value_1a(&QString::from_std_str(key)).to_string());
            button.set_palette(&QPalette::from_q_color(&colour));
            button.set_style_sheet(&QString::from_std_str(format!("background-color: {}", colour.name_1a(NameFormat::HexArgb).to_std_string())));
        }

        Ok(())
    }

    /// This function saves the data from our `SettingsUI` into a `Settings` and return it.
    pub unsafe fn save(&self) -> Result<()> {
        let _ = settings_set_string(MYMOD_BASE_PATH, &self.paths_mymod_line_edit.text().to_std_string());
        let _ = settings_set_string(SECONDARY_PATH, &self.paths_secondary_line_edit.text().to_std_string());

        // For each entry, we check if it's a valid directory and save it into settings_
        for (key, line_edit) in self.paths_games_line_edits.iter() {
            let _ = settings_set_string(key, &line_edit.text().to_std_string());
        }

        for (key, line_edit) in self.paths_asskit_line_edits.iter() {
            let _ = settings_set_string(&(key.to_owned() + "_assembly_kit"), &line_edit.text().to_std_string());
        }

        // We get his game's folder, depending on the selected game.
        let mut game = self.extra_global_default_game_combobox.current_text().to_std_string();
        if let Some(index) = game.find('&') { game.remove(index); }
        game = game.replace(' ', "_").to_lowercase();
        let _ = settings_set_string(DEFAULT_GAME, &game);

        // We need to store the full locale filename, not just the visible name!
        let mut language = self.general_language_combobox.current_text().to_std_string();
        if let Some(index) = language.find('&') { language.remove(index); }
        if let Some((_, locale)) = Locale::get_available_locales(&ASSETS_PATH.to_string_lossy())?.iter().find(|(x, _)| &language == x) {
            let file_name = format!("{}_{}", language, locale.language);
            let _ = settings_set_string(LANGUAGE, &file_name);
        }

        let _ = settings_set_string(UPDATE_CHANNEL, &self.extra_network_update_channel_combobox.current_text().to_std_string());

        let _ = settings_set_string(FONT_NAME, &self.font_data.borrow().0);
        let _ = settings_set_i32(FONT_SIZE, self.font_data.borrow().1);

        // Save spinboxes.
        let _ = settings_set_i32(AUTOSAVE_AMOUNT, self.extra_packfile_autosave_amount_spinbox.value());
        let _ = settings_set_i32(AUTOSAVE_INTERVAL, self.extra_packfile_autosave_interval_spinbox.value());

        // Save all checkboxes.
        for (key, checkbox) in &self.checkboxes {
            let _ = settings_set_bool(key, checkbox.is_checked());
        }

        // Save AI keys.
        let _ = settings_set_string(AI_OPENAI_API_KEY, &self.ai_openai_api_key_line_edit.text().to_std_string());
        let _ = settings_set_string(DEEPL_API_KEY, &self.deepl_api_key_line_edit.text().to_std_string());

        // Save colours.
        let q_settings = QSettings::new();
        for (key, button) in &self.colour_buttons {
            q_settings.set_value(
                &QString::from_std_str(key),
                &QVariant::from_q_string(&button.palette().color_1a(ColorRole::Window).name_1a(NameFormat::HexArgb)),
            );
        }
        q_settings.sync();

        Ok(())
    }

    /// This function updates the path you have for the provided game (or mymod, if you pass it `None`)
    /// with the one you select in a `FileDialog`.
    unsafe fn update_entry_path(&self, game: &str, is_asskit_path: bool) {

        // We check if we have a game or not. If we have it, update the `LineEdit` for that game.
        // If we don't, update the `LineEdit` for `MyMod`s path.
        let (line_edit, is_file) = if is_asskit_path {
            match self.paths_asskit_line_edits.get(game) {
                Some(line_edit) => (line_edit, false),
                None => match game {
                    MYMOD_BASE_PATH => (&self.paths_mymod_line_edit, false),
                    SECONDARY_PATH => (&self.paths_secondary_line_edit, false),
                    _ => return,
                }
            }
        } else {
            match self.paths_games_line_edits.get(game) {
                Some(line_edit) => (line_edit, false),
                None => match game {
                    MYMOD_BASE_PATH => (&self.paths_mymod_line_edit, false),
                    SECONDARY_PATH => (&self.paths_secondary_line_edit, false),
                    _ => return,
                }
            }
        };

        // Create the `FileDialog` and configure it.
        let title = if is_file { qtr("settings_select_file") } else { qtr("settings_select_folder") };
        let file_dialog = QFileDialog::from_q_widget_q_string(
            &self.dialog,
            &title,
        );

        if !is_file {
            file_dialog.set_file_mode(FileMode::Directory);
            file_dialog.set_options(QFlags::from(QFileDialogOption::ShowDirsOnly));
        }

        // Get the old Path, if exists.
        let old_path = line_edit.text().to_std_string();

        // If said path is not empty, and is a dir, set it as the initial directory.
        if !old_path.is_empty() && Path::new(&old_path).is_dir() {
            file_dialog.set_directory_q_string(&line_edit.text());
        }

        // If the game path is set and the assembly kit path it isn't, use the game path to begin with.
        else if is_asskit_path {
            if let Some(line_edit) = self.paths_games_line_edits.get(game) {
                let old_game_path = line_edit.text().to_std_string();
                if !old_game_path.is_empty() && Path::new(&old_game_path).is_dir() {
                    file_dialog.set_directory_q_string(&line_edit.text());
                }
            }
        }

        // Run it and expect a response (1 => Accept, 0 => Cancel).
        if file_dialog.exec() == 1 {

            // Get the path of the selected file.
            let selected_files = file_dialog.selected_files();
            let path = selected_files.at(0);

            // Add the Path to the LineEdit.
            line_edit.set_text(path);
        }
    }

    unsafe fn add_rpfm_to_runcher_tools(&self) -> Result<()> {
        let runcher_config_path = match ProjectDirs::from("com", "FrodoWazEre", "runcher") {
            Some(proj_dirs) => Ok(proj_dirs.config_dir().to_path_buf()),
            None => Err(anyhow!("Failed to get Runcher's config path."))
        }?;
        let fallback_config_path = config_path()?;
        let config_path = Some(runcher_config_path);
        let mut tools = Tools::load(&config_path, &fallback_config_path)?;

        match tools.tools_mut().iter_mut().find(|tool| tool.path().ends_with("rpfm_ui.exe")) {
            Some(tool) => {

                let exe = std::env::current_exe()?;
                if tool.path() != &exe {
                    tool.set_path(exe);
                }
            },
            None => {
                let mut tool = Tool::default();
                tool.set_name("RPFM".to_string());
                tool.set_path(std::env::current_exe()?);
                tool.set_games(SupportedGames::default().game_keys().iter().map(|x| x.to_string()).collect::<Vec<_>>());

                tools.tools_mut().push(tool);
            },
        }

        tools.save(&config_path, &fallback_config_path)?;

        Ok(())
    }
}

/// Creates a QLabel with the setting name and an optional inline description (tip) below it.
unsafe fn new_setting_label(container: &QBox<QWidget>, key: &str, tip_key: &str) -> QBox<QLabel> {
    let text = tr(key);
    let label = if tip_key.is_empty() {
        let l = QLabel::from_q_string_q_widget(&qtr(key), container);
        l.set_accessible_name(&QString::from_std_str(&text));
        l
    } else {
        let tip = tr(tip_key);
        let rich = format!(
            "{}<br><span style='color: gray; font-size: small; font-style: italic; margin-top: 4px;'>{}</span>",
            text, tip
        );
        let l = QLabel::from_q_string_q_widget(&QString::from_std_str(rich), container);
        l.set_text_format(qt_core::TextFormat::RichText);
        l.set_word_wrap(true);
        l.set_accessible_name(&QString::from_std_str(&text));
        l.set_accessible_description(&QString::from_std_str(&tip));
        l
    };
    label
}

/// Creates a checkbox setting row and inserts into the checkboxes map keyed by `settings_key`.
unsafe fn new_setting_checkbox(
    checkboxes: &mut BTreeMap<String, QBox<QCheckBox>>,
    vbox: &QBox<QVBoxLayout>,
    container: &QBox<QWidget>,
    settings_key: &str,
    label_key: &str,
    tip_key: &str,
) {
    let label = new_setting_label(container, label_key, tip_key);
    let checkbox = QCheckBox::from_q_widget(container);
    setting_row!(vbox, container, label, checkbox);

    // On Linux, program updates are managed by the package manager or Flatpak.
    if cfg!(target_os = "linux") && settings_key == "check_updates_on_start" {
        container.set_visible(false);
    }

    // Hidden settings.
    if settings_key == "packfile_treeview_resize_to_fit" {
        container.set_visible(false);
    }

    checkboxes.insert(settings_key.to_owned(), checkbox);
}

/// Creates a colour pair row: label + [light_btn | dark_btn] side by side, inserting both buttons into the map.
unsafe fn new_setting_colour_pair(
    colour_buttons: &mut BTreeMap<String, QBox<QPushButton>>,
    vbox: &QBox<QVBoxLayout>,
    container: &QBox<QWidget>,
    label_key: &str,
    tip_key: &str,
    light_key: &str,
    dark_key: &str,
) {
    let light_btn = QPushButton::from_q_widget(container);
    let dark_btn = QPushButton::from_q_widget(container);
    light_btn.set_auto_fill_background(true);
    dark_btn.set_auto_fill_background(true);
    light_btn.set_minimum_width(32);
    dark_btn.set_minimum_width(32);

    let label = new_setting_label(container, label_key, tip_key);
    let pair = QWidget::new_1a(container);
    let lay = create_grid_layout(pair.static_upcast());
    lay.set_contents_margins_4a(0, 0, 0, 0);
    lay.set_spacing(4);
    lay.add_widget_5a(&light_btn, 0, 0, 1, 1);
    lay.add_widget_5a(&dark_btn, 0, 1, 1, 1);
    lay.set_column_stretch(2, 1);
    setting_row!(vbox, container, label, pair);

    colour_buttons.insert(light_key.to_owned(), light_btn);
    colour_buttons.insert(dark_key.to_owned(), dark_btn);
}

unsafe fn new_setting_combobox(vbox: &QBox<QVBoxLayout>, container: &QBox<QWidget>, key: &str, tip_key: &str, values: &[&str]) -> QBox<QComboBox> {
    let label = new_setting_label(container, key, tip_key);
    let combobox = QComboBox::new_1a(container);
    for value in values {
        combobox.add_item_q_string(&QString::from_std_str(value));
    }
    setting_row!(vbox, container, label, combobox);

    // On Linux, program updates are managed by the package manager or Flatpak.
    if cfg!(target_os = "linux") {
        if key == "settings_update_channel" {
            container.set_visible(false);
        }
    }

    combobox
}

unsafe fn new_setting_spinbox(vbox: &QBox<QVBoxLayout>, container: &QBox<QWidget>, key: &str, tip_key: &str) -> QBox<QSpinBox> {
    let label = new_setting_label(container, key, tip_key);
    let spinbox = QSpinBox::new_1a(container);
    setting_row!(vbox, container, label, spinbox);
    spinbox
}

/// Creates a line edit setting row: label with description on the left, text input on the right.
unsafe fn new_setting_line_edit(
    vbox: &QBox<QVBoxLayout>,
    container: &QBox<QWidget>,
    label_key: &str,
    tip_key: &str,
) -> QBox<QLineEdit> {
    let label = new_setting_label(container, label_key, tip_key);
    let line_edit = QLineEdit::from_q_widget(container);
    setting_row!(vbox, container, label, line_edit);
    line_edit
}

/// Creates a button setting row: label with description on the left, action button on the right.
unsafe fn new_setting_button(
    vbox: &QBox<QVBoxLayout>,
    container: &QBox<QWidget>,
    label_key: &str,
    tip_key: &str,
    button_key: &str,
) -> QBox<QPushButton> {
    let label = new_setting_label(container, label_key, tip_key);
    let button = QPushButton::from_q_string_q_widget(&qtr(button_key), container);
    setting_row!(vbox, container, label, button);
    button
}

/// Creates a path row inside a QGridLayout at a specific row: label + line edit + "..." button.
/// The placeholder text uses `qtre` with the game display name as a replacement.
unsafe fn new_setting_grid_path(
    grid: &QBox<qt_widgets::QGridLayout>,
    parent: &QBox<QWidget>,
    row: i32,
    label_key: &str,
    placeholder_key: &str,
    game_name: &str,
) -> (QBox<QLineEdit>, QBox<QPushButton>) {
    let label = QLabel::from_q_string_q_widget(&qtr(label_key), parent);
    let line_edit = QLineEdit::from_q_widget(parent);
    let button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), parent);
    line_edit.set_placeholder_text(&qtre(placeholder_key, &[game_name]));
    grid.add_widget_5a(&label, row, 0, 1, 1);
    grid.add_widget_5a(&line_edit, row, 1, 1, 1);
    grid.add_widget_5a(&button, row, 2, 1, 1);
    (line_edit, button)
}

/// Creates a path setting row: label + line edit + "..." browse button.
unsafe fn new_setting_path(
    vbox: &QBox<QVBoxLayout>,
    container: &QBox<QWidget>,
    label_key: &str,
    placeholder_key: &str,
) -> (QBox<QLineEdit>, QBox<QPushButton>) {
    let label = QLabel::from_q_string_q_widget(&qtr(label_key), container);
    let line_edit = QLineEdit::from_q_widget(container);
    let button = QPushButton::from_q_string_q_widget(&QString::from_std_str("..."), container);
    line_edit.set_placeholder_text(&qtr(placeholder_key));
    setting_row!(vbox, container, label, line_edit, button);
    (line_edit, button)
}

unsafe fn new_category_button(nav_widget: &QBox<QWidget>, nav_layout: &QBox<QVBoxLayout>, key: &&str) -> QBox<QPushButton> {
    let btn = QPushButton::from_q_string_q_widget(&qtr(key), nav_widget);
    btn.set_flat(true);
    btn.set_checkable(true);
    btn.set_minimum_height(30);
    btn.set_style_sheet(&QString::from_std_str("QPushButton { text-align: left; padding: 4px 8px; } QPushButton:checked { font-weight: bold; }"));
    nav_layout.add_widget_1a(&btn);
    btn
}
