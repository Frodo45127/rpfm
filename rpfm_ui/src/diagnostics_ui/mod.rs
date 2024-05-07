//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `DiagnosticsUI`.
!*/

use qt_widgets::QAction;
use qt_widgets::q_abstract_item_view::ScrollHint;
use qt_widgets::{QCheckBox, QVBoxLayout};
use qt_widgets::QDockWidget;
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::QScrollArea;
use qt_widgets::QTableView;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::{CaseSensitivity, DockWidgetArea, Orientation, SortOrder, ToolButtonStyle};
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QModelIndex;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::QPtr;
use qt_core::QObject;
use qt_core::QSignalBlocker;

use cpp_core::CppBox;
use cpp_core::Ptr;

use anyhow::Result;
use getset::Getters;
use rayon::prelude::*;

use std::rc::Rc;

use rpfm_extensions::diagnostics::{*, anim_fragment_battle::*, config::*, dependency::*, pack::*, portrait_settings::*, table::*};

use rpfm_lib::files::ContainerPath;
use rpfm_lib::games::supported_games::*;
use rpfm_lib::integrations::log::info;

use rpfm_ui_common::locale::{qtr, qtre};

use crate::app_ui::AppUI;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::CENTRAL_COMMAND;
use crate::dependencies_ui::DependenciesUI;
use crate::ffi::{new_tableview_filter_safe, trigger_tableview_filter_safe};
use crate::GAME_SELECTED;
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::*;
use crate::packedfile_views::{DataSource, FileView, View, ViewType, SpecialView};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::*;
use crate::UI_STATE;
use crate::references_ui::ReferencesUI;
use crate::utils::*;
use crate::views::table::{ITEM_HAS_ERROR, ITEM_HAS_WARNING, ITEM_HAS_INFO, utils::open_subtable};

pub mod connections;
pub mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/diagnostics_dock_widget.ui";
const VIEW_RELEASE: &str = "ui/diagnostics_dock_widget.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Diagnostics panel.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct DiagnosticsUI {

    //-------------------------------------------------------------------------------//
    // `Diagnostics` Dock Widget.
    //-------------------------------------------------------------------------------//
    diagnostics_dock_widget: QPtr<QDockWidget>,
    diagnostics_table_view: QPtr<QTableView>,
    diagnostics_table_filter: QBox<QSortFilterProxyModel>,
    diagnostics_table_model: QBox<QStandardItemModel>,

    //-------------------------------------------------------------------------------//
    // Filters section.
    //-------------------------------------------------------------------------------//
    diagnostics_button_check_packfile: QPtr<QToolButton>,
    diagnostics_button_check_current_packed_file: QPtr<QToolButton>,
    diagnostics_button_error: QPtr<QToolButton>,
    diagnostics_button_warning: QPtr<QToolButton>,
    diagnostics_button_info: QPtr<QToolButton>,
    diagnostics_button_only_current_packed_file: QPtr<QToolButton>,
    diagnostics_button_show_more_filters: QPtr<QToolButton>,
    diagnostics_button_check_ak_only_refs: QPtr<QToolButton>,

    diagnostics_table_view_context_menu: QBox<QMenu>,
    ignore_parent_folder: QPtr<QAction>,
    ignore_parent_folder_field: QPtr<QAction>,
    ignore_file: QPtr<QAction>,
    ignore_file_field: QPtr<QAction>,
    ignore_diagnostic_for_parent_folder: QPtr<QAction>,
    ignore_diagnostic_for_parent_folder_field: QPtr<QAction>,
    ignore_diagnostic_for_file: QPtr<QAction>,
    ignore_diagnostic_for_file_field: QPtr<QAction>,
    ignore_diagnostic_for_pack: QPtr<QAction>,

    sidebar_scroll_area: QPtr<QScrollArea>,
    checkbox_all: QBox<QCheckBox>,
    checkbox_outdated_table: QBox<QCheckBox>,
    checkbox_invalid_reference: QBox<QCheckBox>,
    checkbox_empty_row: QBox<QCheckBox>,
    checkbox_empty_key_field: QBox<QCheckBox>,
    checkbox_empty_key_fields: QBox<QCheckBox>,
    checkbox_duplicated_combined_keys: QBox<QCheckBox>,
    checkbox_no_reference_table_found: QBox<QCheckBox>,
    checkbox_no_reference_table_nor_column_found_pak: QBox<QCheckBox>,
    checkbox_no_reference_table_nor_column_found_no_pak: QBox<QCheckBox>,
    checkbox_invalid_escape: QBox<QCheckBox>,
    checkbox_duplicated_row: QBox<QCheckBox>,
    checkbox_invalid_dependency_packfile: QBox<QCheckBox>,
    checkbox_invalid_loc_key: QBox<QCheckBox>,
    checkbox_dependencies_cache_not_generated: QBox<QCheckBox>,
    checkbox_invalid_packfile_name: QBox<QCheckBox>,
    checkbox_table_name_ends_in_number: QBox<QCheckBox>,
    checkbox_table_name_has_space: QBox<QCheckBox>,
    checkbox_table_is_datacoring: QBox<QCheckBox>,
    checkbox_dependencies_cache_outdated: QBox<QCheckBox>,
    checkbox_dependencies_cache_could_not_be_loaded: QBox<QCheckBox>,
    checkbox_field_with_path_not_found: QBox<QCheckBox>,
    checkbox_incorrect_game_path: QBox<QCheckBox>,
    checkbox_banned_table: QBox<QCheckBox>,
    checkbox_value_cannot_be_empty: QBox<QCheckBox>,
    checkbox_invalid_art_set_id: QBox<QCheckBox>,
    checkbox_invalid_variant_filename: QBox<QCheckBox>,
    checkbox_file_diffuse_not_found_for_variant: QBox<QCheckBox>,
    checkbox_file_mask_1_not_found_for_variant: QBox<QCheckBox>,
    checkbox_file_mask_2_not_found_for_variant: QBox<QCheckBox>,
    checkbox_file_mask_3_not_found_for_variant: QBox<QCheckBox>,
    checkbox_datacored_portrait_settings: QBox<QCheckBox>,
    checkbox_loocomotion_graph_path_not_found: QBox<QCheckBox>,
    checkbox_file_path_not_found: QBox<QCheckBox>,
    checkbox_meta_file_path_not_found: QBox<QCheckBox>,
    checkbox_snd_file_path_not_found: QBox<QCheckBox>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `DiagnosticsUI`.
impl DiagnosticsUI {

    /// This function creates an entire `DiagnosticsUI` struct.
    pub unsafe fn new(app_ui: &Rc<AppUI>) -> Result<Self> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let diagnostics_dock_widget: QPtr<QDockWidget> = main_widget.static_downcast();
        let diagnostics_dock_inner_widget: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "inner_widget")?;
        let diagnostics_table_view: QPtr<QTableView> = find_widget(&main_widget.static_upcast(), "results_table_view")?;

        let diagnostics_label_hint: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "hint_label")?;
        let diagnostics_button_check_packfile: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "check_full_button")?;
        let diagnostics_button_check_current_packed_file: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "check_open_button")?;
        let diagnostics_button_error: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "error_button")?;
        let diagnostics_button_warning: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "warning_button")?;
        let diagnostics_button_info: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "info_button")?;
        let diagnostics_button_only_current_packed_file: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "only_open_button")?;
        let diagnostics_button_show_more_filters: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "more_filters_button")?;
        let diagnostics_button_check_ak_only_refs: QPtr<QToolButton> = find_widget(&main_widget.static_upcast(), "check_ak_only_refs")?;

        diagnostics_label_hint.set_text(&qtr("diagnostics_hint"));
        diagnostics_button_check_packfile.set_tool_tip(&qtr("diagnostics_button_check_packfile"));
        diagnostics_button_check_current_packed_file.set_tool_tip(&qtr("diagnostics_button_check_current_packed_file"));
        diagnostics_button_error.set_tool_tip(&qtr("diagnostics_button_error"));
        diagnostics_button_warning.set_tool_tip(&qtr("diagnostics_button_warning"));
        diagnostics_button_info.set_tool_tip(&qtr("diagnostics_button_info"));
        diagnostics_button_only_current_packed_file.set_tool_tip(&qtr("diagnostics_button_only_current_packed_file"));
        diagnostics_button_show_more_filters.set_tool_tip(&qtr("diagnostics_button_show_more_filters"));
        diagnostics_button_check_ak_only_refs.set_tool_tip(&qtr("diagnostics_check_ak_only_refs"));

        diagnostics_button_error.set_tool_button_style(ToolButtonStyle::ToolButtonTextUnderIcon);
        diagnostics_button_warning.set_tool_button_style(ToolButtonStyle::ToolButtonTextUnderIcon);
        diagnostics_button_info.set_tool_button_style(ToolButtonStyle::ToolButtonTextUnderIcon);

        // Populate the `Contextual Menu`.
        let diagnostics_table_view_context_menu = QMenu::from_q_widget(&diagnostics_table_view);
        let ignore_parent_folder = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_parent_folder", "ignore_parent_folder", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_parent_folder_field = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_parent_folder_field", "ignore_parent_folder_field", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_file = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_file", "ignore_file", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_file_field = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_file_field", "ignore_file_field", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_diagnostic_for_parent_folder = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_diagnostic_for_parent_folder", "ignore_diagnostic_for_parent_folder", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_diagnostic_for_parent_folder_field = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_diagnostic_for_parent_folder_field", "ignore_diagnostic_for_parent_folder_field", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_diagnostic_for_file = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_diagnostic_for_file", "ignore_diagnostic_for_file", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_diagnostic_for_file_field = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_diagnostic_for_file_field", "ignore_diagnostic_for_file_field", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));
        let ignore_diagnostic_for_pack = add_action_to_menu(&diagnostics_table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "diagnostics_context_menu", "ignore_diagnostic_for_pack", "ignore_diagnostic_for_pack", Some(diagnostics_table_view.static_upcast::<qt_widgets::QWidget>()));

        let sidebar_scroll_area: QPtr<QScrollArea> = find_widget(&main_widget.static_upcast(), "more_filters_scroll")?;
        let header_column: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "diagnostics_label")?;
        sidebar_scroll_area.horizontal_scroll_bar().set_enabled(false);
        sidebar_scroll_area.hide();
        header_column.set_text(&qtr("diagnostic_type"));

        app_ui.main_window().add_dock_widget_2a(DockWidgetArea::BottomDockWidgetArea, diagnostics_dock_widget.as_ptr());
        diagnostics_dock_widget.set_window_title(&qtr("gen_loc_diagnostics"));
        diagnostics_dock_widget.set_object_name(&QString::from_std_str("diagnostics_dock"));

        diagnostics_button_info.set_style_sheet(&QString::from_std_str(format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_info(), get_color_info_pressed())));

        diagnostics_button_warning.set_style_sheet(&QString::from_std_str(format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_warning(), get_color_warning_pressed())));

        diagnostics_button_error.set_style_sheet(&QString::from_std_str(format!("
        QPushButton {{
            background-color: {}
        }}
        QPushButton::checked {{
            background-color: {}
        }}", get_color_error(), get_color_error_pressed())));

        let diagnostics_table_filter = new_tableview_filter_safe(diagnostics_dock_inner_widget.static_upcast());
        let diagnostics_table_model = QStandardItemModel::new_1a(&diagnostics_dock_inner_widget);
        diagnostics_table_filter.set_source_model(&diagnostics_table_model);
        diagnostics_table_view.set_model(&diagnostics_table_filter);

        if setting_bool("tight_table_mode") {
            diagnostics_table_view.vertical_header().set_minimum_section_size(22);
            diagnostics_table_view.vertical_header().set_maximum_section_size(22);
            diagnostics_table_view.vertical_header().set_default_section_size(22);
        }

        app_ui.main_window().set_corner(qt_core::Corner::BottomLeftCorner, qt_core::DockWidgetArea::LeftDockWidgetArea);
        app_ui.main_window().set_corner(qt_core::Corner::BottomRightCorner, qt_core::DockWidgetArea::RightDockWidgetArea);

        //-------------------------------------------------------------------------------//
        // Sidebar section.
        //-------------------------------------------------------------------------------//

        // Create the search and hide/show/freeze widgets.
        let sidebar_widget = sidebar_scroll_area.widget();
        let sidebar_grid: QPtr<QVBoxLayout> = sidebar_widget.layout().static_downcast();

        let checkbox_all = QCheckBox::from_q_string_q_widget(&qtr("all"), &sidebar_scroll_area);
        let checkbox_outdated_table = QCheckBox::from_q_string_q_widget(&qtr("label_outdated_table"), &sidebar_scroll_area);
        let checkbox_invalid_reference = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_reference"), &sidebar_scroll_area);
        let checkbox_empty_row = QCheckBox::from_q_string_q_widget(&qtr("label_empty_row"), &sidebar_scroll_area);
        let checkbox_empty_key_field = QCheckBox::from_q_string_q_widget(&qtr("label_empty_key_field"), &sidebar_scroll_area);
        let checkbox_empty_key_fields = QCheckBox::from_q_string_q_widget(&qtr("label_empty_key_fields"), &sidebar_scroll_area);
        let checkbox_duplicated_combined_keys = QCheckBox::from_q_string_q_widget(&qtr("label_duplicated_combined_keys"), &sidebar_scroll_area);
        let checkbox_no_reference_table_found = QCheckBox::from_q_string_q_widget(&qtr("label_no_reference_table_found"), &sidebar_scroll_area);
        let checkbox_no_reference_table_nor_column_found_pak = QCheckBox::from_q_string_q_widget(&qtr("label_no_reference_table_nor_column_found_pak"), &sidebar_scroll_area);
        let checkbox_no_reference_table_nor_column_found_no_pak = QCheckBox::from_q_string_q_widget(&qtr("label_no_reference_table_nor_column_found_no_pak"), &sidebar_scroll_area);
        let checkbox_invalid_escape = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_escape"), &sidebar_scroll_area);
        let checkbox_duplicated_row = QCheckBox::from_q_string_q_widget(&qtr("label_duplicated_row"), &sidebar_scroll_area);
        let checkbox_invalid_dependency_packfile = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_dependency_packfile"), &sidebar_scroll_area);
        let checkbox_invalid_loc_key = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_loc_key"), &sidebar_scroll_area);
        let checkbox_dependencies_cache_not_generated = QCheckBox::from_q_string_q_widget(&qtr("label_dependencies_cache_not_generated"), &sidebar_scroll_area);
        let checkbox_invalid_packfile_name = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_packfile_name"), &sidebar_scroll_area);
        let checkbox_table_name_ends_in_number = QCheckBox::from_q_string_q_widget(&qtr("label_table_name_ends_in_number"), &sidebar_scroll_area);
        let checkbox_table_name_has_space = QCheckBox::from_q_string_q_widget(&qtr("label_table_name_has_space"), &sidebar_scroll_area);
        let checkbox_table_is_datacoring = QCheckBox::from_q_string_q_widget(&qtr("label_table_is_datacoring"), &sidebar_scroll_area);
        let checkbox_dependencies_cache_outdated = QCheckBox::from_q_string_q_widget(&qtr("label_dependencies_cache_outdated"), &sidebar_scroll_area);
        let checkbox_dependencies_cache_could_not_be_loaded = QCheckBox::from_q_string_q_widget(&qtr("label_dependencies_cache_could_not_be_loaded"), &sidebar_scroll_area);
        let checkbox_field_with_path_not_found = QCheckBox::from_q_string_q_widget(&qtr("label_field_with_path_not_found"), &sidebar_scroll_area);
        let checkbox_incorrect_game_path = QCheckBox::from_q_string_q_widget(&qtr("label_incorrect_game_path"), &sidebar_scroll_area);
        let checkbox_banned_table = QCheckBox::from_q_string_q_widget(&qtr("label_banned_table"), &sidebar_scroll_area);
        let checkbox_value_cannot_be_empty = QCheckBox::from_q_string_q_widget(&qtr("label_value_cannot_be_empty"), &sidebar_scroll_area);
        let checkbox_invalid_art_set_id = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_art_set_id"), &sidebar_scroll_area);
        let checkbox_invalid_variant_filename = QCheckBox::from_q_string_q_widget(&qtr("label_invalid_variant_filename"), &sidebar_scroll_area);
        let checkbox_file_diffuse_not_found_for_variant = QCheckBox::from_q_string_q_widget(&qtr("label_file_diffuse_not_found_for_variant"), &sidebar_scroll_area);
        let checkbox_file_mask_1_not_found_for_variant = QCheckBox::from_q_string_q_widget(&qtr("label_file_mask_1_not_found_for_variant"), &sidebar_scroll_area);
        let checkbox_file_mask_2_not_found_for_variant = QCheckBox::from_q_string_q_widget(&qtr("label_file_mask_2_not_found_for_variant"), &sidebar_scroll_area);
        let checkbox_file_mask_3_not_found_for_variant = QCheckBox::from_q_string_q_widget(&qtr("label_file_mask_3_not_found_for_variant"), &sidebar_scroll_area);
        let checkbox_datacored_portrait_settings = QCheckBox::from_q_string_q_widget(&qtr("label_datacored_portrait_settings"), &sidebar_scroll_area);
        let checkbox_loocomotion_graph_path_not_found = QCheckBox::from_q_string_q_widget(&qtr("label_locomotion_graph_path_not_found"), &sidebar_scroll_area);
        let checkbox_file_path_not_found = QCheckBox::from_q_string_q_widget(&qtr("label_file_path_not_found"), &sidebar_scroll_area);
        let checkbox_meta_file_path_not_found = QCheckBox::from_q_string_q_widget(&qtr("label_meta_file_path_not_found"), &sidebar_scroll_area);
        let checkbox_snd_file_path_not_found = QCheckBox::from_q_string_q_widget(&qtr("label_snd_file_path_not_found"), &sidebar_scroll_area);

        checkbox_all.set_checked(false);
        checkbox_outdated_table.set_checked(true);
        checkbox_invalid_reference.set_checked(true);
        checkbox_empty_row.set_checked(true);
        checkbox_empty_key_field.set_checked(true);
        checkbox_empty_key_fields.set_checked(true);
        checkbox_duplicated_combined_keys.set_checked(true);
        checkbox_no_reference_table_found.set_checked(true);
        checkbox_no_reference_table_nor_column_found_pak.set_checked(true);
        checkbox_no_reference_table_nor_column_found_no_pak.set_checked(true);
        checkbox_invalid_escape.set_checked(true);
        checkbox_duplicated_row.set_checked(true);
        checkbox_invalid_dependency_packfile.set_checked(true);
        checkbox_invalid_loc_key.set_checked(true);
        checkbox_dependencies_cache_not_generated.set_checked(true);
        checkbox_invalid_packfile_name.set_checked(true);
        checkbox_table_name_ends_in_number.set_checked(true);
        checkbox_table_name_has_space.set_checked(true);
        checkbox_table_is_datacoring.set_checked(true);
        checkbox_dependencies_cache_outdated.set_checked(true);
        checkbox_dependencies_cache_could_not_be_loaded.set_checked(true);
        checkbox_field_with_path_not_found.set_checked(false);
        checkbox_incorrect_game_path.set_checked(true);
        checkbox_banned_table.set_checked(true);
        checkbox_value_cannot_be_empty.set_checked(true);
        checkbox_invalid_art_set_id.set_checked(true);
        checkbox_invalid_variant_filename.set_checked(true);
        checkbox_file_diffuse_not_found_for_variant.set_checked(true);
        checkbox_file_mask_1_not_found_for_variant.set_checked(true);
        checkbox_file_mask_2_not_found_for_variant.set_checked(true);
        checkbox_file_mask_3_not_found_for_variant.set_checked(true);
        checkbox_datacored_portrait_settings.set_checked(true);
        checkbox_loocomotion_graph_path_not_found.set_checked(true);
        checkbox_file_path_not_found.set_checked(true);
        checkbox_meta_file_path_not_found.set_checked(true);
        checkbox_snd_file_path_not_found.set_checked(true);

        sidebar_grid.add_widget_1a(&checkbox_all);
        sidebar_grid.add_widget_1a(&checkbox_outdated_table);
        sidebar_grid.add_widget_1a(&checkbox_invalid_reference);
        sidebar_grid.add_widget_1a(&checkbox_empty_row);
        sidebar_grid.add_widget_1a(&checkbox_empty_key_field);
        sidebar_grid.add_widget_1a(&checkbox_empty_key_fields);
        sidebar_grid.add_widget_1a(&checkbox_duplicated_combined_keys);
        sidebar_grid.add_widget_1a(&checkbox_no_reference_table_found);
        sidebar_grid.add_widget_1a(&checkbox_no_reference_table_nor_column_found_pak);
        sidebar_grid.add_widget_1a(&checkbox_no_reference_table_nor_column_found_no_pak);
        sidebar_grid.add_widget_1a(&checkbox_invalid_escape);
        sidebar_grid.add_widget_1a(&checkbox_duplicated_row);
        sidebar_grid.add_widget_1a(&checkbox_invalid_dependency_packfile);
        sidebar_grid.add_widget_1a(&checkbox_invalid_loc_key);
        sidebar_grid.add_widget_1a(&checkbox_dependencies_cache_not_generated);
        sidebar_grid.add_widget_1a(&checkbox_invalid_packfile_name);
        sidebar_grid.add_widget_1a(&checkbox_table_name_ends_in_number);
        sidebar_grid.add_widget_1a(&checkbox_table_name_has_space);
        sidebar_grid.add_widget_1a(&checkbox_table_is_datacoring);
        sidebar_grid.add_widget_1a(&checkbox_dependencies_cache_outdated);
        sidebar_grid.add_widget_1a(&checkbox_dependencies_cache_could_not_be_loaded);
        sidebar_grid.add_widget_1a(&checkbox_field_with_path_not_found);
        sidebar_grid.add_widget_1a(&checkbox_incorrect_game_path);
        sidebar_grid.add_widget_1a(&checkbox_banned_table);
        sidebar_grid.add_widget_1a(&checkbox_value_cannot_be_empty);
        sidebar_grid.add_widget_1a(&checkbox_invalid_art_set_id);
        sidebar_grid.add_widget_1a(&checkbox_invalid_variant_filename);
        sidebar_grid.add_widget_1a(&checkbox_file_diffuse_not_found_for_variant);
        sidebar_grid.add_widget_1a(&checkbox_file_mask_1_not_found_for_variant);
        sidebar_grid.add_widget_1a(&checkbox_file_mask_2_not_found_for_variant);
        sidebar_grid.add_widget_1a(&checkbox_file_mask_3_not_found_for_variant);
        sidebar_grid.add_widget_1a(&checkbox_datacored_portrait_settings);
        sidebar_grid.add_widget_1a(&checkbox_loocomotion_graph_path_not_found);
        sidebar_grid.add_widget_1a(&checkbox_file_path_not_found);
        sidebar_grid.add_widget_1a(&checkbox_meta_file_path_not_found);
        sidebar_grid.add_widget_1a(&checkbox_snd_file_path_not_found);

        Ok(Self {

            //-------------------------------------------------------------------------------//
            // `Diagnostics` Dock Widget.
            //-------------------------------------------------------------------------------//
            diagnostics_dock_widget,
            diagnostics_table_view,
            diagnostics_table_filter,
            diagnostics_table_model,

            //-------------------------------------------------------------------------------//
            // Filters section.
            //-------------------------------------------------------------------------------//
            diagnostics_button_check_packfile,
            diagnostics_button_check_current_packed_file,
            diagnostics_button_error,
            diagnostics_button_warning,
            diagnostics_button_info,
            diagnostics_button_only_current_packed_file,
            diagnostics_button_show_more_filters,
            diagnostics_button_check_ak_only_refs,

            diagnostics_table_view_context_menu,
            ignore_parent_folder,
            ignore_parent_folder_field,
            ignore_file,
            ignore_file_field,
            ignore_diagnostic_for_parent_folder,
            ignore_diagnostic_for_parent_folder_field,
            ignore_diagnostic_for_file,
            ignore_diagnostic_for_file_field,
            ignore_diagnostic_for_pack,

            sidebar_scroll_area,
            checkbox_all,
            checkbox_outdated_table,
            checkbox_invalid_reference,
            checkbox_empty_row,
            checkbox_empty_key_field,
            checkbox_empty_key_fields,
            checkbox_duplicated_combined_keys,
            checkbox_no_reference_table_found,
            checkbox_no_reference_table_nor_column_found_pak,
            checkbox_no_reference_table_nor_column_found_no_pak,
            checkbox_invalid_escape,
            checkbox_duplicated_row,
            checkbox_invalid_dependency_packfile,
            checkbox_invalid_loc_key,
            checkbox_dependencies_cache_not_generated,
            checkbox_invalid_packfile_name,
            checkbox_table_name_ends_in_number,
            checkbox_table_name_has_space,
            checkbox_table_is_datacoring,
            checkbox_dependencies_cache_outdated,
            checkbox_dependencies_cache_could_not_be_loaded,
            checkbox_field_with_path_not_found,
            checkbox_incorrect_game_path,
            checkbox_banned_table,
            checkbox_value_cannot_be_empty,
            checkbox_invalid_art_set_id,
            checkbox_invalid_variant_filename,
            checkbox_file_diffuse_not_found_for_variant,
            checkbox_file_mask_1_not_found_for_variant,
            checkbox_file_mask_2_not_found_for_variant,
            checkbox_file_mask_3_not_found_for_variant,
            checkbox_datacored_portrait_settings,
            checkbox_loocomotion_graph_path_not_found,
            checkbox_file_path_not_found,
            checkbox_meta_file_path_not_found,
            checkbox_snd_file_path_not_found,
        })
    }

    /// This function takes care of checking the entire PackFile for errors.
    pub unsafe fn check(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>) {

        // Only check if we actually have the diagnostics open.
        if !diagnostics_ui.diagnostics_dock_widget.is_visible() {
            return;
        }

        app_ui.menu_bar_packfile().set_enabled(false);
        let diagnostics_ignored = diagnostics_ui.diagnostics_ignored();
        info!("Triggering check.");
        let receiver = CENTRAL_COMMAND.send_background(Command::DiagnosticsCheck(diagnostics_ignored, diagnostics_ui.diagnostics_button_check_ak_only_refs().is_checked()));
        let response = CENTRAL_COMMAND.recv_try(&receiver);

        match response {
            Response::Diagnostics(diagnostics) => {
                Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.results());
                Self::filter(app_ui, diagnostics_ui);
                Self::update_level_counts(diagnostics_ui, diagnostics.results());
                UI_STATE.set_diagnostics(&diagnostics);
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }

        app_ui.menu_bar_packfile().set_enabled(true);
    }

    /// This function takes care of updating the results of a diagnostics check for the provided paths.
    pub unsafe fn check_on_path(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>, paths: Vec<ContainerPath>) {

        // Only check if we actually have the diagnostics open.
        if !diagnostics_ui.diagnostics_dock_widget.is_visible() {
            return;
        }

        app_ui.menu_bar_packfile().set_enabled(false);

        let mut diagnostics = UI_STATE.get_diagnostics();
        *diagnostics.diagnostics_ignored_mut() = diagnostics_ui.diagnostics_ignored();
        info!("Triggering check update.");
        let receiver = CENTRAL_COMMAND.send_background(Command::DiagnosticsUpdate(diagnostics, paths, diagnostics_ui.diagnostics_button_check_ak_only_refs().is_checked()));
        let response = CENTRAL_COMMAND.recv_try(&receiver);

        match response {
            Response::Diagnostics(diagnostics) => {
                Self::load_diagnostics_to_ui(app_ui, diagnostics_ui, diagnostics.results());
                Self::filter(app_ui, diagnostics_ui);
                Self::update_level_counts(diagnostics_ui, diagnostics.results());
                UI_STATE.set_diagnostics(&diagnostics);
            }
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        }

        app_ui.menu_bar_packfile().set_enabled(true);
    }

    /// This function takes care of loading the results of a diagnostic check into the table.
    unsafe fn load_diagnostics_to_ui(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {

        // First, clean the current diagnostics.
        Self::clean_diagnostics_from_views(app_ui);

        // Build the table columns without data in them, because otherwise it becomes very slow.
        diagnostics_ui.diagnostics_table_model.clear();
        diagnostics_ui.diagnostics_table_model.set_column_count(7);

        diagnostics_ui.diagnostics_table_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_level")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_diag")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_cells_affected")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_path")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_message")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(5, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_report_type")));
        diagnostics_ui.diagnostics_table_model.set_header_data_3a(6, Orientation::Horizontal, &QVariant::from_q_string(&qtr("diagnostics_colum_column_names")));

        // Hide the column number column for tables.
        diagnostics_ui.diagnostics_table_view.hide_column(1);
        diagnostics_ui.diagnostics_table_view.hide_column(2);
        diagnostics_ui.diagnostics_table_view.hide_column(5);
        diagnostics_ui.diagnostics_table_view.hide_column(6);
        diagnostics_ui.diagnostics_table_view.sort_by_column_2a(3, SortOrder::AscendingOrder);

        diagnostics_ui.diagnostics_table_view.horizontal_header().set_stretch_last_section(true);
        diagnostics_ui.diagnostics_table_view.horizontal_header().set_section_resize_mode_2a(0, ResizeMode::Fixed);
        diagnostics_ui.diagnostics_table_view.horizontal_header().set_default_section_size(70);
        diagnostics_ui.diagnostics_table_view.set_column_width(3, 600);

        if !diagnostics.is_empty() {

            // Microoptimization: block the model from triggering signals on each item added.
            diagnostics_ui.diagnostics_table_model.block_signals(true);

            let result_type_info = atomic_from_cpp_box(QString::from_std_str("Info"));
            let result_type_warning = atomic_from_cpp_box(QString::from_std_str("Warning"));
            let result_type_error = atomic_from_cpp_box(QString::from_std_str("Error"));

            let color_info = atomic_from_cpp_box(QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_info()))));
            let color_warning = atomic_from_cpp_box(QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_warning()))));
            let color_error = atomic_from_cpp_box(QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(get_color_error()))));

            let rows = diagnostics.par_iter()
                .map(|diagnostic_type| {

                    match diagnostic_type {
                        DiagnosticType::AnimFragmentBattle(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));
                                data_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result).unwrap())), 2);
                                path.set_text(&QString::from_std_str(diagnostic.path()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));

                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_anim_fragment(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        }
                        DiagnosticType::DB(ref diagnostic) |
                        DiagnosticType::Loc(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));
                                data_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.cells_affected()).unwrap())), 2);
                                path.set_text(&QString::from_std_str(diagnostic.path()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));
                                extra_data_1.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.column_names()).unwrap())), 2);


                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_table(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        },
                        DiagnosticType::Pack(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));

                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_packfile(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        }
                        DiagnosticType::PortraitSettings(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));

                                let data_affected_string = match result.report_type() {
                                    PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings => String::new(),
                                    PortraitSettingsDiagnosticReportType::InvalidArtSetId(art_set_id) => art_set_id.to_owned(),
                                    PortraitSettingsDiagnosticReportType::InvalidVariantFilename(art_set_id, variant_filename) => art_set_id.to_owned() + "|" + variant_filename,
                                    PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(art_set_id, variant_filename, _) => art_set_id.to_owned() + "|" + variant_filename,
                                    PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(art_set_id, variant_filename, _) => art_set_id.to_owned() + "|" + variant_filename,
                                    PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(art_set_id, variant_filename, _) => art_set_id.to_owned() + "|" + variant_filename,
                                    PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(art_set_id, variant_filename, _) => art_set_id.to_owned() + "|" + variant_filename,
                                };

                                data_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(data_affected_string)), 2);
                                path.set_text(&QString::from_std_str(diagnostic.path()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));

                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_portrait_settings(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        }
                        DiagnosticType::Dependency(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));
                                data_affected.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(serde_json::to_string(&result.cells_affected()).unwrap())), 2);
                                path.set_text(&QString::from_std_str(diagnostic.path()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));

                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_dependency_manager(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        }

                        DiagnosticType::Config(ref diagnostic) => {
                            let mut reports = Vec::with_capacity(diagnostic.results().len());

                            for result in diagnostic.results() {
                                let qlist = QListOfQStandardItem::new();

                                // Create an empty row.
                                let level = Self::new_item();
                                let diag_type = Self::new_item();
                                let data_affected = Self::new_item();
                                let path = Self::new_item();
                                let message = Self::new_item();
                                let report_type = Self::new_item();
                                let extra_data_1 = Self::new_item();

                                let (result_type, color) = match result.level() {
                                    DiagnosticLevel::Info => (ref_from_atomic(&result_type_info), ref_from_atomic(&color_info)),
                                    DiagnosticLevel::Warning => (ref_from_atomic(&result_type_warning), ref_from_atomic(&color_warning)),
                                    DiagnosticLevel::Error => (ref_from_atomic(&result_type_error), ref_from_atomic(&color_error)),
                                };

                                level.set_background(color);
                                level.set_text(result_type);
                                diag_type.set_text(&QString::from_std_str(diagnostic_type.to_string()));
                                message.set_text(&QString::from_std_str(result.message()));
                                report_type.set_text(&QString::from_std_str(result.report_type().to_string()));

                                // Set the tooltips to the diag type and description columns.
                                Self::set_tooltips_config(&[&level, &path, &message], result.report_type());

                                qlist.append_q_standard_item(&level.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&diag_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&data_affected.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&path.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&message.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&report_type.into_ptr().as_mut_raw_ptr());
                                qlist.append_q_standard_item(&extra_data_1.into_ptr().as_mut_raw_ptr());

                                reports.push(atomic_from_cpp_box(qlist));
                            }

                            reports
                        }
                    }
                })
                .flatten()
                .collect::<Vec<_>>();

            for (index, row) in rows.iter().enumerate() {

                // Unlock the model before the last insertion.
                if index == rows.len() - 1 {
                    diagnostics_ui.diagnostics_table_model.block_signals(false);
                }

                diagnostics_ui.diagnostics_table_model.append_row_q_list_of_q_standard_item(ref_from_atomic(row));

            }

            // After that, check if the table is open, and paint the results into it.
            for diagnostic_type in diagnostics {
                Self::paint_diagnostics_to_table(app_ui, diagnostic_type);
            }
        }
    }

    /// This function tries to open the PackedFile where the selected match is.
    pub unsafe fn open_match(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<Self>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
        model_index_filtered: Ptr<QModelIndex>
    ) {

        let filter_model: QPtr<QSortFilterProxyModel> = model_index_filtered.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter_model.source_model().static_downcast();
        let model_index = filter_model.map_to_source(model_index_filtered.as_ref().unwrap());

        // If it's a match, get the path, the position data of the match, and open the PackedFile, scrolling it down.
        let item_path = model.item_2a(model_index.row(), 3);
        let path = item_path.text().to_std_string();
        let tree_index = pack_file_contents_ui.packfile_contents_tree_view().expand_treeview_to_item(&path, DataSource::PackFile);

        let diagnostic_type = model.item_2a(model_index.row(), 1).text().to_std_string();
        if diagnostic_type == "DependencyManager" {
            AppUI::open_special_view(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, SpecialView::PackDependencies);
        } else if !path.is_empty() {

            // Manually select the open PackedFile, then open it. This means we can open PackedFiles nor in out filter.
            UI_STATE.set_packfile_contents_read_only(true);

            if let Some(ref tree_index) = tree_index {
                if tree_index.is_valid() {
                    pack_file_contents_ui.packfile_contents_tree_view().scroll_to_1a(tree_index.as_ref().unwrap());
                    pack_file_contents_ui.packfile_contents_tree_view().selection_model().select_q_model_index_q_flags_selection_flag(tree_index.as_ref().unwrap(), QFlags::from(SelectionFlag::ClearAndSelect));
                }
            }

            UI_STATE.set_packfile_contents_read_only(false);
            AppUI::open_packedfile(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui, dependencies_ui, references_ui, Some(path.to_owned()), false, false, DataSource::PackFile);
        }

        // If it's a table, focus on the matched cell.
        match &*model.item_2a(model_index.row(), 1).text().to_std_string() {
            "AnimFragmentBattle" => {
                if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).find(|x| *x.path_read() == path) {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    if let ViewType::Internal(View::AnimFragmentBattle(view)) = file_view.view_type() {
                        let results: AnimFragmentBattleDiagnosticReport = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        if *results.locomotion_graph() {
                            view.locomotion_graph_line_edit().select_all();
                            view.locomotion_graph_line_edit().set_focus_0a();
                        } else if let Some((row, Some((subrow, file_path, meta_file_path, snd_file_path)))) = results.entry() {
                            let table_selection_model = view.table().table_view().selection_model();
                            let table_model_index = view.table().table_model().index_2a(*row as i32, 9);
                            let table_model_index_filtered = view.table().table_filter().map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                view.table().table_view().set_focus_0a();
                                view.table().table_view().set_current_index(table_model_index_filtered.as_ref());
                                view.table().table_view().scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));

                                let column = if *file_path {
                                    0
                                } else if *meta_file_path {
                                    1
                                } else if *snd_file_path {
                                    2
                                } else {
                                    return;
                                };

                                open_subtable(table_model_index_filtered.as_ref(), view.table(), app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, references_ui, Some((*subrow as i32, column)));
                            }
                        }
                    }
                }
            }

            "DB" | "Loc" | "DependencyManager" => {

                if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).find(|x| *x.path_read() == path) {

                    // In case of tables, we have to get the logical row/column of the match and select it.
                    if let ViewType::Internal(View::Table(view)) = file_view.view_type() {
                        let table_view = view.get_ref_table();
                        let table_view = table_view.table_view();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        table_selection_model.clear_selection();
                        let cells_affected: Vec<(i32, i32)> = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        for (row, column) in cells_affected {
                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.set_focus_0a();
                                table_view.set_current_index(table_model_index_filtered.as_ref());
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                            }
                        }
                    }

                    else if let ViewType::Internal(View::DependenciesManager(view)) = file_view.view_type() {

                        let table_view = view.get_ref_table();
                        let table_view = table_view.table_view();
                        let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                        let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                        let table_selection_model = table_view.selection_model();

                        table_selection_model.clear_selection();
                        let cells_affected: Vec<(i32, i32)> = serde_json::from_str(&model.item_2a(model_index.row(), 2).text().to_std_string()).unwrap();
                        for (row, column) in cells_affected {
                            let table_model_index = table_model.index_2a(row, column);
                            let table_model_index_filtered = table_filter.map_from_source(&table_model_index);
                            if table_model_index_filtered.is_valid() {
                                table_view.set_focus_0a();
                                table_view.set_current_index(table_model_index_filtered.as_ref());
                                table_view.scroll_to_2a(table_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                table_selection_model.select_q_model_index_q_flags_selection_flag(table_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                            }
                        }
                    }
                }
            }

            "PortraitSettings" => {
                if let Some(file_view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).find(|x| *x.path_read() == path) {
                    if let ViewType::Internal(View::PortraitSettings(view)) = file_view.view_type() {
                        let list_view = view.main_list_view();
                        let list_filter: QPtr<QSortFilterProxyModel> = list_view.model().static_downcast();
                        let list_model: QPtr<QStandardItemModel> = list_filter.source_model().static_downcast();
                        let list_selection_model = list_view.selection_model();
                        list_selection_model.clear_selection();

                        let data_merged = model.item_2a(model_index.row(), 2).text().to_std_string();
                        let data = data_merged.split('|').collect::<Vec<_>>();

                        // If we have an art set id, select it.
                        if let Some(art_set_id) = data.first() {
                            let mut art_set_id_found = false;

                            let q_string = QString::from_std_str(art_set_id);
                            for row in 0..list_model.row_count_0a() {
                                let list_model_index = list_model.index_2a(row, 0);
                                if list_model.data_1a(&list_model_index).to_string().compare_q_string(&q_string) == 0 {
                                    let list_model_index_filtered = list_filter.map_from_source(&list_model_index);
                                    if list_model_index_filtered.is_valid() {
                                        list_view.set_focus_0a();
                                        list_view.set_current_index(list_model_index_filtered.as_ref());
                                        list_view.scroll_to_2a(list_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                        list_selection_model.select_q_model_index_q_flags_selection_flag(list_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));
                                        art_set_id_found = true;
                                        break;
                                    }
                                }
                            }

                            // If we also have a variant name, select it.
                            if art_set_id_found {
                                if let Some(variant_filename) = data.get(1) {
                                    let q_string = QString::from_std_str(variant_filename);
                                    for row in 0..view.variants_list_model().row_count_0a() {
                                        let list_model_index = view.variants_list_model().index_2a(row, 0);
                                        if view.variants_list_model().data_1a(&list_model_index).to_string().compare_q_string(&q_string) == 0 {
                                            let list_model_index_filtered = view.variants_list_filter().map_from_source(&list_model_index);
                                            if list_model_index_filtered.is_valid() {
                                                view.variants_list_view().set_focus_0a();
                                                view.variants_list_view().set_current_index(list_model_index_filtered.as_ref());
                                                view.variants_list_view().scroll_to_2a(list_model_index_filtered.as_ref(), ScrollHint::EnsureVisible);
                                                view.variants_list_view().selection_model().select_q_model_index_q_flags_selection_flag(list_model_index_filtered.as_ref(), QFlags::from(SelectionFlag::SelectCurrent));

                                                // We need to check the report type to see if we have to select a line edit.
                                                let report_type = model.item_2a(model_index.row(), 5).text().to_std_string();
                                                match &*report_type {
                                                    "FileDiffuseNotFoundForVariant" => {
                                                        view.file_diffuse_line_edit().select_all();
                                                        view.file_diffuse_line_edit().set_focus_0a();
                                                    },
                                                    "FileMask1NotFoundForVariant" => {
                                                        view.file_mask_1_line_edit().select_all();
                                                        view.file_mask_1_line_edit().set_focus_0a();
                                                    },
                                                    "FileMask2NotFoundForVariant" => {
                                                        view.file_mask_2_line_edit().select_all();
                                                        view.file_mask_2_line_edit().set_focus_0a();
                                                    },
                                                    "FileMask3NotFoundForVariant" => {
                                                        view.file_mask_3_line_edit().select_all();
                                                        view.file_mask_3_line_edit().set_focus_0a();
                                                    }
                                                    _ => {},
                                                }

                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Config matches have to open their relevant config issue.
            "Config" => {
                match &*model.item_2a(model_index.row(), 5).text().to_std_string() {

                    // For these, we will trigger the action to generate the dependencies cache.
                    "DependenciesCacheNotGenerated" |
                    "DependenciesCacheOutdated" |
                    "DependenciesCacheCouldNotBeLoaded" => {
                        match GAME_SELECTED.read().unwrap().key() {
                            KEY_PHARAOH => app_ui.special_stuff_ph_generate_dependencies_cache().trigger(),
                            KEY_WARHAMMER_3 => app_ui.special_stuff_wh3_generate_dependencies_cache().trigger(),
                            KEY_TROY => app_ui.special_stuff_troy_generate_dependencies_cache().trigger(),
                            KEY_THREE_KINGDOMS => app_ui.special_stuff_three_k_generate_dependencies_cache().trigger(),
                            KEY_WARHAMMER_2 => app_ui.special_stuff_wh2_generate_dependencies_cache().trigger(),
                            KEY_WARHAMMER => app_ui.special_stuff_wh_generate_dependencies_cache().trigger(),
                            KEY_THRONES_OF_BRITANNIA => app_ui.special_stuff_tob_generate_dependencies_cache().trigger(),
                            KEY_ATTILA => app_ui.special_stuff_att_generate_dependencies_cache().trigger(),
                            KEY_ROME_2 => app_ui.special_stuff_rom2_generate_dependencies_cache().trigger(),
                            KEY_SHOGUN_2 => app_ui.special_stuff_sho2_generate_dependencies_cache().trigger(),
                            KEY_NAPOLEON => app_ui.special_stuff_nap_generate_dependencies_cache().trigger(),
                            KEY_EMPIRE => app_ui.special_stuff_emp_generate_dependencies_cache().trigger(),
                            _ => {}
                        }
                    }
                    "IncorrectGamePath" => app_ui.packfile_settings().trigger(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// This function tries to paint the results from the provided diagnostics into their file view, if the file is open.
    pub unsafe fn paint_diagnostics_to_table(
        app_ui: &Rc<AppUI>,
        diagnostic: &DiagnosticType,
    ) {

        let path = match diagnostic {
            DiagnosticType::AnimFragmentBattle(ref diagnostic) => diagnostic.path(),
            DiagnosticType::DB(ref diagnostic) |
            DiagnosticType::Loc(ref diagnostic) => diagnostic.path(),
            DiagnosticType::Dependency(ref diagnostic) => diagnostic.path(),
            _ => return,
        };

        if let Some(view) = UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile).find(|view| &view.path_copy() == path) {
            if app_ui.tab_bar_packed_file().index_of(view.main_widget()) != -1 {

                // In case of tables, we have to get the logical row/column of the match and select it.
                let internal_table_view = if let ViewType::Internal(View::Table(view)) = view.view_type() { view.get_ref_table() }
                else if let ViewType::Internal(View::DependenciesManager(view)) = view.view_type() { view.get_ref_table() }
                //else if let ViewType::Internal(View::AnimFragment(view)) = view.get_view() { view.table_view() }
                else { return };

                let table_view = internal_table_view.table_view();
                let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                match diagnostic {
                    DiagnosticType::DB(ref diagnostic) |
                    DiagnosticType::Loc(ref diagnostic) => {
                        for result in diagnostic.results() {
                            for (row, column) in result.cells_affected() {
                                if *row != -1 || *column != -1 {
                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row, column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row, *column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row, *column);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level() {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    DiagnosticType::Dependency(ref diagnostic) => {
                        for result in diagnostic.results() {
                            for (row, column) in result.cells_affected() {
                                if *row != -1 || *column != -1 {

                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row, column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row, *column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row, *column);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level() {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },

                    /*
                    DiagnosticType::AnimFragmentBattle(ref diagnostic) => {
                        for result in diagnostic.results() {
                            for (row, column) in result.cells_affected() {
                                if *row != -1 || *column != -1 {
                                    if *column == -1 {
                                        for column in 0..table_model.column_count_0a() {
                                            let table_model_index = table_model.index_2a(*row, column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else if *row == -1 {
                                        for row in 0..table_model.row_count_0a() {
                                            let table_model_index = table_model.index_2a(row, *column);
                                            let table_model_item = table_model.item_from_index(&table_model_index);

                                            // At this point, is possible the row is no longer valid, so we have to check it out first.
                                            if table_model_index.is_valid() {
                                                match result.level() {
                                                    DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                    DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                    DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                                }
                                            }
                                        }
                                    } else {
                                        let table_model_index = table_model.index_2a(*row, *column);
                                        let table_model_item = table_model.item_from_index(&table_model_index);

                                        // At this point, is possible the row is no longer valid, so we have to check it out first.
                                        if table_model_index.is_valid() {
                                            match result.level() {
                                                DiagnosticLevel::Error => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_ERROR),
                                                DiagnosticLevel::Warning => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_WARNING),
                                                DiagnosticLevel::Info => table_model_item.set_data_2a(&QVariant::from_bool(true), ITEM_HAS_INFO),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    */
                    _ => return,
                }

                // Unblock the model and update it. Otherwise, the painted cells wont show up until something else updates the view.
                blocker.unblock();
                table_view.viewport().repaint();
            }
        }
    }

    pub unsafe fn clean_diagnostics_from_views(app_ui: &Rc<AppUI>) {
        for view in UI_STATE.get_open_packedfiles().iter().filter(|x| x.data_source() == DataSource::PackFile) {

            // Only update the visible tables.
            if app_ui.tab_bar_packed_file().index_of(view.main_widget()) != -1 {

                // In case of tables, we have to get the logical row/column of the match and select it.
                if let ViewType::Internal(View::Table(view)) = view.view_type() {
                    let table_view = view.get_ref_table().table_view();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);
                            if !item.is_null() {
                                if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                    item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                                }
                                if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                    item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                                }
                                if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                    item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                                }
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                }

               /*else if let ViewType::Internal(View::AnimFragment(view)) = view.get_view() {
                    let table_view = view.table_view().table_view_ptr();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);

                            if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                            }
                            if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                            }
                            if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                }*/

                else if let ViewType::Internal(View::DependenciesManager(view)) = view.view_type() {
                    let table_view = view.get_ref_table().table_view();
                    let table_filter: QPtr<QSortFilterProxyModel> = table_view.model().static_downcast();
                    let table_model: QPtr<QStandardItemModel> = table_filter.source_model().static_downcast();
                    let blocker = QSignalBlocker::from_q_object(table_model.static_upcast::<QObject>());

                    for row in 0..table_model.row_count_0a() {
                        for column in 0..table_model.column_count_0a() {
                            let item = table_model.item_2a(row, column);

                            if item.data_1a(ITEM_HAS_ERROR).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_ERROR);
                            }
                            if item.data_1a(ITEM_HAS_WARNING).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_WARNING);
                            }
                            if item.data_1a(ITEM_HAS_INFO).to_bool() {
                                item.set_data_2a(&QVariant::from_bool(false), ITEM_HAS_INFO);
                            }
                        }
                    }
                    blocker.unblock();
                    table_view.viewport().repaint();
                }
            }
        }
    }

    pub unsafe fn filter(app_ui: &Rc<AppUI>, diagnostics_ui: &Rc<Self>) {
        let mut columns = vec![];
        let mut patterns = vec![];
        let mut sensitivity = vec![];

        let info_state = diagnostics_ui.diagnostics_button_info.is_checked();
        let warning_state = diagnostics_ui.diagnostics_button_warning.is_checked();
        let error_state = diagnostics_ui.diagnostics_button_error.is_checked();
        let pattern_level = match (info_state, warning_state, error_state) {
            (true, true, true) => "Info|Warning|Error",
            (true, true, false) => "Info|Warning",
            (true, false, true) => "Info|Error",
            (false, true, true) => "Warning|Error",
            (true, false, false) => "Info",
            (false, false, true) => "Error",
            (false, true, false) => "Warning",
            (false, false, false) => "-1",
        };

        columns.push(0);
        patterns.push(QString::from_std_str(pattern_level).into_ptr());
        sensitivity.push(CaseSensitivity::CaseSensitive);

        // Check for currently open files filter.
        if diagnostics_ui.diagnostics_button_only_current_packed_file.is_checked() {
            let open_packedfiles = UI_STATE.get_open_packedfiles();
            let open_packedfiles_ref = open_packedfiles.iter()
                .filter(|x| x.data_source() == DataSource::PackFile && app_ui.tab_bar_packed_file().index_of(x.main_widget()) != -1)
                .collect::<Vec<&FileView>>();
            let mut pattern = String::new();
            for open_packedfile in &open_packedfiles_ref {
                if !pattern.is_empty() {
                    pattern.push('|');
                }
                pattern.push_str(&open_packedfile.path_read().to_string());
            }

            // This makes sure the check works even if we don't have anything open.
            if pattern.is_empty() {
                pattern.push_str("empty");
            }

            columns.push(3);
            patterns.push(QString::from_std_str(pattern).into_ptr());
            sensitivity.push(CaseSensitivity::CaseSensitive);
        }

        // Checks for the diagnostic type filter.
        let mut diagnostic_type_pattern = String::new();

        if diagnostics_ui.checkbox_outdated_table.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::OutdatedTable));
        }
        if diagnostics_ui.checkbox_invalid_reference.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidReference(String::new(), String::new())));
        }
        if diagnostics_ui.checkbox_empty_row.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyRow));
        }
        if diagnostics_ui.checkbox_empty_key_field.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyKeyField(String::new())));
        }
        if diagnostics_ui.checkbox_empty_key_fields.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::EmptyKeyFields));
        }
        if diagnostics_ui.checkbox_duplicated_combined_keys.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::DuplicatedCombinedKeys(String::new())));
        }
        if diagnostics_ui.checkbox_no_reference_table_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableFound(String::new())));
        }
        if diagnostics_ui.checkbox_no_reference_table_nor_column_found_pak.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(String::new())));
        }
        if diagnostics_ui.checkbox_no_reference_table_nor_column_found_no_pak.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(String::new())));
        }
        if diagnostics_ui.checkbox_invalid_escape.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidEscape));
        }
        if diagnostics_ui.checkbox_duplicated_row.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::DuplicatedRow(String::new())));
        }
        if diagnostics_ui.checkbox_invalid_loc_key.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::InvalidLocKey));
        }
        if diagnostics_ui.checkbox_table_name_ends_in_number.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableNameEndsInNumber));
        }
        if diagnostics_ui.checkbox_table_name_has_space.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableNameHasSpace));
        }
        if diagnostics_ui.checkbox_table_is_datacoring.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::TableIsDataCoring));
        }
        if diagnostics_ui.checkbox_field_with_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::FieldWithPathNotFound(vec![])));
        }
        if diagnostics_ui.checkbox_banned_table.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::BannedTable));
        }
        if diagnostics_ui.checkbox_value_cannot_be_empty.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", TableDiagnosticReportType::ValueCannotBeEmpty(String::new())));
        }


        if diagnostics_ui.checkbox_invalid_dependency_packfile.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", DependencyDiagnosticReportType::InvalidDependencyPackName(String::new())));
        }

        if diagnostics_ui.checkbox_dependencies_cache_not_generated.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheNotGenerated));
        }
        if diagnostics_ui.checkbox_dependencies_cache_outdated.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheOutdated));
        }
        if diagnostics_ui.checkbox_dependencies_cache_could_not_be_loaded.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded("".to_owned())));
        }
        if diagnostics_ui.checkbox_incorrect_game_path.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", ConfigDiagnosticReportType::IncorrectGamePath));
        }

        if diagnostics_ui.checkbox_invalid_packfile_name.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PackDiagnosticReportType::InvalidPackName(String::new())));
        }

        if diagnostics_ui.checkbox_datacored_portrait_settings.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings));
        }
        if diagnostics_ui.checkbox_invalid_art_set_id.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::InvalidArtSetId(String::new())));
        }
        if diagnostics_ui.checkbox_invalid_variant_filename.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::InvalidVariantFilename(String::new(), String::new())));
        }
        if diagnostics_ui.checkbox_file_diffuse_not_found_for_variant.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(String::new(), String::new(), String::new())));
        }
        if diagnostics_ui.checkbox_file_mask_1_not_found_for_variant.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(String::new(), String::new(), String::new())));
        }
        if diagnostics_ui.checkbox_file_mask_2_not_found_for_variant.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(String::new(), String::new(), String::new())));
        }
        if diagnostics_ui.checkbox_file_mask_3_not_found_for_variant.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(String::new(), String::new(), String::new())));
        }

        if diagnostics_ui.checkbox_loocomotion_graph_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(String::new())));
        }
        if diagnostics_ui.checkbox_file_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", AnimFragmentBattleDiagnosticReportType::FilePathNotFound(String::new())));
        }
        if diagnostics_ui.checkbox_meta_file_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(String::new())));
        }
        if diagnostics_ui.checkbox_snd_file_path_not_found.is_checked() {
            diagnostic_type_pattern.push_str(&format!("{}|", AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(String::new())));
        }

        diagnostic_type_pattern.pop();

        if diagnostic_type_pattern.is_empty() {
            diagnostic_type_pattern.push_str("empty");
        }

        columns.push(5);
        patterns.push(QString::from_std_str(diagnostic_type_pattern).into_ptr());
        sensitivity.push(CaseSensitivity::CaseSensitive);
        let use_nott = vec![false; sensitivity.len()];
        let use_regex = vec![true; sensitivity.len()];
        let show_blank_lines = vec![false; sensitivity.len()];
        let match_groups = vec![0; sensitivity.len()];
        let variant_to_search = vec![0; sensitivity.len()];

        // Filter whatever it's in that column by the text we got.
        trigger_tableview_filter_safe(&diagnostics_ui.diagnostics_table_filter, &columns, patterns, &use_nott, &use_regex, &sensitivity, &show_blank_lines, &match_groups, &variant_to_search);
    }

    pub unsafe fn update_level_counts(diagnostics_ui: &Rc<Self>, diagnostics: &[DiagnosticType]) {
        let info = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragmentBattle(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::Pack(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::PortraitSettings(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::Dependency(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
                DiagnosticType::Config(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Info))
                    .count(),
            }).sum::<usize>();

        let warning = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragmentBattle(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::Pack(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::PortraitSettings(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::Dependency(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
                DiagnosticType::Config(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Warning))
                    .count(),
            }).sum::<usize>();


        let error = diagnostics.iter().map(|x|
            match x {
                DiagnosticType::AnimFragmentBattle(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::DB(ref diag) |
                DiagnosticType::Loc(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::Pack(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::PortraitSettings(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::Dependency(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
                DiagnosticType::Config(ref diag) => diag.results()
                    .iter()
                    .filter(|y| matches!(y.level(), DiagnosticLevel::Error))
                    .count(),
            }).sum::<usize>();

        diagnostics_ui.diagnostics_button_info.set_text(&QString::from_std_str(format!("({})", info)));
        diagnostics_ui.diagnostics_button_warning.set_text(&QString::from_std_str(format!("({})", warning)));
        diagnostics_ui.diagnostics_button_error.set_text(&QString::from_std_str(format!("({})", error)));
    }

    pub unsafe fn set_tooltips_anim_fragment(items: &[&CppBox<QStandardItem>], report_type: &AnimFragmentBattleDiagnosticReportType) {
        let tool_tip = match report_type {
            AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(_) => qtr("field_with_path_not_found_explanation"),
            AnimFragmentBattleDiagnosticReportType::FilePathNotFound(_) => qtr("field_with_path_not_found_explanation"),
            AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(_) => qtr("field_with_path_not_found_explanation"),
            AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(_) => qtr("field_with_path_not_found_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_table(items: &[&CppBox<QStandardItem>], report_type: &TableDiagnosticReportType) {
        let tool_tip = match report_type {
            TableDiagnosticReportType::OutdatedTable => qtr("outdated_table_explanation"),
            TableDiagnosticReportType::InvalidReference(_, _) => qtr("invalid_reference_explanation"),
            TableDiagnosticReportType::EmptyRow => qtr("empty_row_explanation"),
            TableDiagnosticReportType::EmptyKeyField(_) => qtr("empty_key_field_explanation"),
            TableDiagnosticReportType::EmptyKeyFields => qtr("empty_key_fields_explanation"),
            TableDiagnosticReportType::DuplicatedCombinedKeys(_) => qtr("duplicated_combined_keys_explanation"),
            TableDiagnosticReportType::NoReferenceTableFound(_) => qtr("no_reference_table_found_explanation"),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(_) => qtr("no_reference_table_nor_column_found_pak_explanation"),
            TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(_) => qtr("no_reference_table_nor_column_found_no_pak_explanation"),
            TableDiagnosticReportType::InvalidEscape => qtr("invalid_escape_explanation"),
            TableDiagnosticReportType::DuplicatedRow(_) => qtr("duplicated_row_explanation"),
            TableDiagnosticReportType::InvalidLocKey => qtr("invalid_loc_key_explanation"),
            TableDiagnosticReportType::TableNameEndsInNumber => qtr("table_name_ends_in_number_explanation"),
            TableDiagnosticReportType::TableNameHasSpace => qtr("table_name_has_space_explanation"),
            TableDiagnosticReportType::TableIsDataCoring => qtr("table_is_datacoring_explanation"),
            TableDiagnosticReportType::FieldWithPathNotFound(_) => qtr("field_with_path_not_found_explanation"),
            TableDiagnosticReportType::BannedTable => qtr("banned_table_explanation"),
            TableDiagnosticReportType::ValueCannotBeEmpty(_) => qtr("value_cannot_be_empty_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_dependency_manager(items: &[&CppBox<QStandardItem>], report_type: &DependencyDiagnosticReportType) {
        let tool_tip = match report_type {
            DependencyDiagnosticReportType::InvalidDependencyPackName(_) => qtr("invalid_dependency_pack_file_name_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_config(items: &[&CppBox<QStandardItem>], report_type: &ConfigDiagnosticReportType) {
        let tool_tip = match report_type {
            ConfigDiagnosticReportType::DependenciesCacheNotGenerated => qtr("dependencies_cache_not_generated_explanation"),
            ConfigDiagnosticReportType::DependenciesCacheOutdated => qtr("dependencies_cache_outdated_explanation"),
            ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(error) => qtre("dependencies_cache_could_not_be_loaded_explanation", &[error]),
            ConfigDiagnosticReportType::IncorrectGamePath => qtr("incorrect_game_path_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_packfile(items: &[&CppBox<QStandardItem>], report_type: &PackDiagnosticReportType) {
        let tool_tip = match report_type {
            PackDiagnosticReportType::InvalidPackName(_) => qtr("invalid_packfile_name_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    pub unsafe fn set_tooltips_portrait_settings(items: &[&CppBox<QStandardItem>], report_type: &PortraitSettingsDiagnosticReportType) {
        let tool_tip = match report_type {
            PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings => qtr("datacored_portrait_settings_explanation"),
            PortraitSettingsDiagnosticReportType::InvalidArtSetId(_) => qtr("invalid_art_set_id_explanation"),
            PortraitSettingsDiagnosticReportType::InvalidVariantFilename(_, _) => qtr("invalid_variant_filename_explanation"),
            PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(_, _, _) => qtr("file_diffuse_not_found_for_variant_explanation"),
            PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(_, _, _) => qtr("file_mask_1_not_found_for_variant_explanation"),
            PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(_, _, _) => qtr("file_mask_2_not_found_for_variant_explanation"),
            PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(_, _, _) => qtr("file_mask_3_not_found_for_variant_explanation"),
        };

        for item in items {
            item.set_tool_tip(&tool_tip);
        }
    }

    unsafe fn diagnostics_ignored(&self) -> Vec<String> {

        let mut diagnostics_ignored = vec![];
        if !self.checkbox_outdated_table.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::OutdatedTable.to_string());
        }
        if !self.checkbox_invalid_reference.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::InvalidReference(String::new(), String::new()).to_string());
        }
        if !self.checkbox_empty_row.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::EmptyRow.to_string());
        }
        if !self.checkbox_empty_key_field.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::EmptyKeyField(String::new()).to_string());
        }
        if !self.checkbox_empty_key_fields.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::EmptyKeyFields.to_string());
        }
        if !self.checkbox_duplicated_combined_keys.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::DuplicatedCombinedKeys(String::new()).to_string());
        }
        if !self.checkbox_no_reference_table_found.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::NoReferenceTableFound(String::new()).to_string());
        }
        if !self.checkbox_no_reference_table_nor_column_found_pak.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::NoReferenceTableNorColumnFoundPak(String::new()).to_string());
        }
        if !self.checkbox_no_reference_table_nor_column_found_no_pak.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::NoReferenceTableNorColumnFoundNoPak(String::new()).to_string());
        }
        if !self.checkbox_invalid_escape.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::InvalidEscape.to_string());
        }
        if !self.checkbox_duplicated_row.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::DuplicatedRow(String::new()).to_string());
        }
        if !self.checkbox_invalid_loc_key.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::InvalidLocKey.to_string());
        }
        if !self.checkbox_table_name_ends_in_number.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::TableNameEndsInNumber.to_string());
        }
        if !self.checkbox_table_name_has_space.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::TableNameHasSpace.to_string());
        }
        if !self.checkbox_table_is_datacoring.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::TableIsDataCoring.to_string());
        }
        if !self.checkbox_field_with_path_not_found.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::FieldWithPathNotFound(vec![]).to_string());
        }
        if !self.checkbox_banned_table.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::BannedTable.to_string());
        }
        if !self.checkbox_value_cannot_be_empty.is_checked() {
            diagnostics_ignored.push(TableDiagnosticReportType::ValueCannotBeEmpty(String::new()).to_string());
        }

        if !self.checkbox_invalid_dependency_packfile.is_checked() {
            diagnostics_ignored.push(DependencyDiagnosticReportType::InvalidDependencyPackName(String::new()).to_string());
        }

        if !self.checkbox_dependencies_cache_not_generated.is_checked() {
            diagnostics_ignored.push(ConfigDiagnosticReportType::DependenciesCacheNotGenerated.to_string());
        }
        if !self.checkbox_dependencies_cache_outdated.is_checked() {
            diagnostics_ignored.push(ConfigDiagnosticReportType::DependenciesCacheOutdated.to_string());
        }
        if !self.checkbox_dependencies_cache_could_not_be_loaded.is_checked() {
            diagnostics_ignored.push(ConfigDiagnosticReportType::DependenciesCacheCouldNotBeLoaded(String::new()).to_string());
        }
        if !self.checkbox_incorrect_game_path.is_checked() {
            diagnostics_ignored.push(ConfigDiagnosticReportType::IncorrectGamePath.to_string());
        }

        if !self.checkbox_invalid_packfile_name.is_checked() {
            diagnostics_ignored.push(PackDiagnosticReportType::InvalidPackName(String::new()).to_string());
        }

        if !self.checkbox_datacored_portrait_settings.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::DatacoredPortraitSettings.to_string());
        }
        if !self.checkbox_invalid_art_set_id.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::InvalidArtSetId(String::new()).to_string());
        }
        if !self.checkbox_invalid_variant_filename.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::InvalidVariantFilename(String::new(), String::new()).to_string());
        }
        if !self.checkbox_file_diffuse_not_found_for_variant.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::FileDiffuseNotFoundForVariant(String::new(), String::new(), String::new()).to_string());
        }

        if !self.checkbox_file_mask_1_not_found_for_variant.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::FileMask1NotFoundForVariant(String::new(), String::new(), String::new()).to_string());
        }
        if !self.checkbox_file_mask_2_not_found_for_variant.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::FileMask2NotFoundForVariant(String::new(), String::new(), String::new()).to_string());
        }
        if !self.checkbox_file_mask_3_not_found_for_variant.is_checked() {
            diagnostics_ignored.push(PortraitSettingsDiagnosticReportType::FileMask3NotFoundForVariant(String::new(), String::new(), String::new()).to_string());
        }

        if !self.checkbox_loocomotion_graph_path_not_found.is_checked() {
            diagnostics_ignored.push(AnimFragmentBattleDiagnosticReportType::LocomotionGraphPathNotFound(String::new()).to_string());
        }
        if !self.checkbox_file_path_not_found.is_checked() {
            diagnostics_ignored.push(AnimFragmentBattleDiagnosticReportType::FilePathNotFound(String::new()).to_string());
        }
        if !self.checkbox_meta_file_path_not_found.is_checked() {
            diagnostics_ignored.push(AnimFragmentBattleDiagnosticReportType::MetaFilePathNotFound(String::new()).to_string());
        }
        if !self.checkbox_snd_file_path_not_found.is_checked() {
            diagnostics_ignored.push(AnimFragmentBattleDiagnosticReportType::SndFilePathNotFound(String::new()).to_string());
        }

        diagnostics_ignored
    }

    pub unsafe fn selection_sorted_and_deduped(&self) -> Vec<CppBox<QModelIndex>> {
        let filter_model: QPtr<QSortFilterProxyModel> = self.diagnostics_table_view.model().static_downcast();
        let selection_model = self.diagnostics_table_view.selection_model();
        let selected_indexes = selection_model.selected_indexes();
        let mut selection = (0..selected_indexes.count_0a())
            .map(|index| filter_model.map_to_source(selected_indexes.at(index)))
            .collect::<Vec<_>>();

        selection.sort_by_key(|index| index.row());
        selection.dedup_by_key(|index| index.row());
        selection
    }

    unsafe fn new_item() -> CppBox<QStandardItem> {
        let item = QStandardItem::new();
        item.set_editable(false);
        item
    }
}
