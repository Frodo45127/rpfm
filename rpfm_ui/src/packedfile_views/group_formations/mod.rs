//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the views for GroupFormations files: the editor, and a debug view with the file as JSON.
//!
//! The editor keeps the decoded file in memory and applies every edit to it directly, so saving just
//! returns a copy of it. Each edit stores a snapshot of the file first, which is what undo restores.

use qt_widgets::q_abstract_item_view::{EditTrigger, SelectionMode};
use qt_widgets::QAbstractItemView;
use qt_widgets::QComboBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGraphicsView;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListView;
use qt_widgets::QMenu;
use qt_widgets::QSpinBox;
use qt_widgets::QSplitter;
use qt_widgets::QStackedWidget;
use qt_widgets::QTableView;
use qt_widgets::QToolButton;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QAction;
use qt_gui::QIcon;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CaseSensitivity;
use qt_core::CheckState;
use qt_core::ItemDataRole;
use qt_core::QBox;
use qt_core::q_item_selection_model::SelectionFlag;
use qt_core::QListOfQString;
use qt_core::QObject;
use qt_core::QPtr;
use qt_core::QSignalBlocker;
use qt_core::QSortFilterProxyModel;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QVariant;

use cpp_core::Ptr;

use anyhow::{anyhow, Result};
use getset::*;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use rpfm_ipc::api::tables::GetColumnValues;

use rpfm_lib::files::{FileType, RFileDecoded};
use rpfm_lib::files::group_formations::*;
use rpfm_lib::files::group_formations::layout::LayoutParams;
use rpfm_lib::files::group_formations::validation::ValidationIssue;

use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::communications::*;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::GAME_SELECTED;
use crate::packedfile_views::{DataSource, FileView, View, ViewType, utils::set_modified};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::*;
use crate::views::debug::DebugView;

use self::slots::GroupFormationsSlots;

/// Runs the same code over the container inside a block, whether it's absolute or relative.
///
/// Both container types have the same fields (except the parent id), but they're different types,
/// so this expands the body once per type. Evaluates to `None` for spans.
macro_rules! with_container {
    ($block:expr, $container:ident => $body:expr) => {
        match $block {
            Block::ContainerAbsolute($container) => Some($body),
            Block::ContainerRelative($container) => Some($body),
            Block::Spanning(_) => None,
        }
    };
}

mod connections;
mod slots;

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/group_formations_editor.ui";
const VIEW_RELEASE: &str = "ui/group_formations_editor.ui";

/// Role storing the formation index or block id of an item.
const ID_ROLE: i32 = 256;

/// Initial widths of the formations list, the canvas and the inspector, in pixels.
const EDITOR_COLUMN_WIDTHS: [i32; 3] = [220, 1000, 300];

/// Delay after the last edit before sending the file to the backend and updating its diagnostics, in milliseconds.
const DIAGNOSTICS_UPDATE_DELAY: i32 = 500;

/// Maximum amount of snapshots kept for undo.
const MAX_UNDO_STEPS: usize = 200;

/// Pages of the inspector, in the order they have in the template.
const PAGE_EMPTY: i32 = 0;
const PAGE_FORMATION: i32 = 1;
const PAGE_CONTAINER: i32 = 2;
const PAGE_SPAN: i32 = 3;

/// Extra margin of spans around their members on the canvas, per level of spans nested inside them.
const SPAN_MARGIN: f64 = 1.5;

/// Color of spans on the canvas.
const SPAN_COLOR: (u8, u8, u8) = (128, 128, 128);

/// Colors for containers on the canvas, picked by their main entity so the same units share a color.
const CONTAINER_COLORS: [(u8, u8, u8); 10] = [
    (0x4e, 0x79, 0xa7),
    (0xf2, 0x8e, 0x2b),
    (0xe1, 0x57, 0x59),
    (0x76, 0xb7, 0xb2),
    (0x59, 0xa1, 0x4f),
    (0xed, 0xc9, 0x48),
    (0xb0, 0x7a, 0xa1),
    (0xff, 0x9d, 0xa7),
    (0x9c, 0x75, 0x5f),
    (0xba, 0xb0, 0xac),
];

/// Columns of the entity preferences table.
const COLUMN_PRIORITY: i32 = 0;
const COLUMN_ENTITY: i32 = 1;
const COLUMN_WEIGHT: i32 = 2;
const COLUMN_UK_1: i32 = 3;
const COLUMN_ENTITY_CLASS: i32 = 4;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the editor view of a GroupFormations file.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct GroupFormationsView {
    path: Arc<RwLock<String>>,
    pack_key: Arc<RwLock<String>>,
    data_source: Arc<RwLock<DataSource>>,

    format: GroupFormationsFormat,

    /// If the file can't be edited, because it doesn't belong to the open Pack.
    is_read_only: bool,

    data: RwLock<GroupFormations>,
    undo_history: RwLock<Vec<GroupFormations>>,
    redo_history: RwLock<Vec<GroupFormations>>,

    /// Set while the view fills its widgets, so the edit slots ignore the changes it makes.
    #[getset(skip)]
    is_loading: AtomicBool,

    formations_filter_line_edit: QPtr<QLineEdit>,
    formations_list_view: QPtr<QListView>,
    formations_list_filter: QBox<QSortFilterProxyModel>,
    formations_list_model: QBox<QStandardItemModel>,
    formations_context_menu: QBox<QMenu>,
    add_formation: QPtr<QAction>,
    clone_formation: QPtr<QAction>,
    delete_formation: QPtr<QAction>,

    canvas: QPtr<QGraphicsView>,
    unit_count_spinbox: QPtr<QSpinBox>,
    grid_step_spinbox: QPtr<QDoubleSpinBox>,
    fit_button: QPtr<QToolButton>,

    blocks_tree_view: QPtr<QTreeView>,
    blocks_tree_model: QBox<QStandardItemModel>,
    blocks_context_menu: QBox<QMenu>,
    add_absolute: QPtr<QAction>,
    add_relative: QPtr<QAction>,
    add_span: QPtr<QAction>,
    delete_block: QPtr<QAction>,
    delete_subtree: QPtr<QAction>,
    undo: QPtr<QAction>,
    redo: QPtr<QAction>,

    inspector_stack: QPtr<QStackedWidget>,

    name_line_edit: QPtr<QLineEdit>,
    ai_priority_spinbox: QPtr<QDoubleSpinBox>,
    uk_2_spinbox: QPtr<QSpinBox>,
    ai_purpose_model: QBox<QStandardItemModel>,
    min_unit_category: ItemList,
    subcultures: ItemList,
    factions: ItemList,

    parent_combobox: QPtr<QComboBox>,
    position_x_label: QPtr<QLabel>,
    position_x_spinbox: QPtr<QDoubleSpinBox>,
    position_y_label: QPtr<QLabel>,
    position_y_spinbox: QPtr<QDoubleSpinBox>,
    block_priority_spinbox: QPtr<QDoubleSpinBox>,
    arrangement_combobox: QPtr<QComboBox>,
    spacing_spinbox: QPtr<QDoubleSpinBox>,
    crescent_y_offset_spinbox: QPtr<QDoubleSpinBox>,
    minimum_threshold_spinbox: QPtr<QSpinBox>,
    maximum_threshold_spinbox: QPtr<QSpinBox>,
    entity_preferences: ItemList,

    span_members_model: QBox<QStandardItemModel>,

    /// Timer to send the file to the backend and update its diagnostics a bit after the last edit.
    diagnostics_timer: QBox<QTimer>,
}

/// An editable list or table of the inspector, with its buttons to add and remove rows.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct ItemList {
    view: QPtr<QAbstractItemView>,
    model: QBox<QStandardItemModel>,
    add_button: QPtr<QToolButton>,
    remove_button: QPtr<QToolButton>,
}

/// This struct contains the debug view of a GroupFormations file.
pub struct FileGroupFormationsDebugView {
    debug_view: Arc<DebugView>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GroupFormationsView {

    /// This function creates a new GroupFormations editor, and sets up its slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: GroupFormations,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
    ) -> Result<()> {
        let format = GroupFormationsFormat::from_game(&GAME_SELECTED.read().unwrap())?;

        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        layout.add_widget_5a(&main_widget, 0, 0, 1, 1);
        let widget = main_widget.static_upcast::<QWidget>();

        // The canvas column takes any extra space, so the formation has as much room as possible.
        let editor_splitter: QPtr<QSplitter> = find_widget(&widget, "editor_splitter")?;
        editor_splitter.set_stretch_factor(0, 0);
        editor_splitter.set_stretch_factor(1, 1);
        editor_splitter.set_stretch_factor(2, 0);
        let sizes = qt_core::QListOfInt::new_0a();
        for size in EDITOR_COLUMN_WIDTHS {
            sizes.append_int(&size);
        }
        editor_splitter.set_sizes(&sizes);

        // Formations list.
        let formations_filter_line_edit: QPtr<QLineEdit> = find_widget(&widget, "formations_filter_line_edit")?;
        let formations_list_view: QPtr<QListView> = find_widget(&widget, "formations_list_view")?;
        formations_filter_line_edit.set_placeholder_text(&qtr("group_formations_filter"));

        let formations_list_filter = QSortFilterProxyModel::new_1a(&formations_list_view);
        let formations_list_model = QStandardItemModel::new_1a(&formations_list_filter);
        formations_list_filter.set_source_model(&formations_list_model);
        formations_list_filter.set_filter_case_sensitivity(CaseSensitivity::CaseInsensitive);
        formations_list_view.set_model(&formations_list_filter);

        let formations_context_menu = QMenu::from_q_widget(&formations_list_view);
        let formations_widget = formations_list_view.static_upcast::<QWidget>();
        let add_formation = add_action_to_menu(&formations_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "add_formation", "group_formations_add_formation", Some(formations_widget.clone()));
        let clone_formation = add_action_to_menu(&formations_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "clone_formation", "group_formations_clone_formation", Some(formations_widget.clone()));
        let delete_formation = add_action_to_menu(&formations_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "delete_formation", "group_formations_delete_formation", Some(formations_widget));
        set_button_action(&widget, "add_formation_button", &add_formation)?;
        set_button_action(&widget, "clone_formation_button", &clone_formation)?;
        set_button_action(&widget, "delete_formation_button", &delete_formation)?;

        // Canvas. It's a custom widget, so it replaces a placeholder from the template. It's not created
        // as a child of the splitter because the splitter would take it as a new pane, and refuse the replace.
        let blocks_splitter: QPtr<QSplitter> = find_widget(&widget, "blocks_splitter")?;
        let canvas_placeholder: QPtr<QWidget> = find_widget(&widget, "canvas_placeholder")?;
        let canvas = new_group_formation_canvas_safe(&widget);
        let placeholder_index = blocks_splitter.index_of(&canvas_placeholder);
        let replaced = blocks_splitter.replace_widget(placeholder_index, &canvas);
        if replaced.is_null() {
            return Err(anyhow!("The GroupFormations canvas couldn't be added to the view."));
        }
        replaced.delete_later();
        blocks_splitter.set_stretch_factor(placeholder_index, 3);
        group_formation_canvas_set_front_label_safe(&canvas, &tr("group_formations_front"));

        set_label_text(&widget, "unit_count_label", "group_formations_unit_count")?;
        let unit_count_spinbox: QPtr<QSpinBox> = find_widget(&widget, "unit_count_spinbox")?;
        unit_count_spinbox.set_tool_tip(&qtr("group_formations_unit_count_tip"));
        set_label_text(&widget, "grid_step_label", "group_formations_grid_step")?;
        let grid_step_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "grid_step_spinbox")?;
        grid_step_spinbox.set_tool_tip(&qtr("group_formations_grid_step_tip"));
        set_label_text(&widget, "canvas_hint_label", "group_formations_canvas_hint")?;
        canvas.set_context_menu_policy(qt_core::ContextMenuPolicy::CustomContextMenu);

        let fit_button: QPtr<QToolButton> = find_widget(&widget, "fit_button")?;
        fit_button.set_icon(&QIcon::from_theme_q_string(&QString::from_std_str("zoom-fit-best")));
        fit_button.set_tool_tip(&qtr("group_formations_fit"));

        // Blocks tree.
        let blocks_tree_view: QPtr<QTreeView> = find_widget(&widget, "blocks_tree_view")?;
        let blocks_tree_model = QStandardItemModel::new_1a(&blocks_tree_view);
        blocks_tree_view.set_model(&blocks_tree_model);

        // Undo and redo work from anywhere in the editor, so they're attached to the whole view.
        let blocks_context_menu = QMenu::from_q_widget(&blocks_tree_view);
        let blocks_widget = blocks_tree_view.static_upcast::<QWidget>();
        let add_absolute = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "add_absolute", "group_formations_add_absolute", Some(blocks_widget.clone()));
        let add_relative = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "add_relative", "group_formations_add_relative", Some(blocks_widget.clone()));
        let add_span = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "add_span", "group_formations_add_span", Some(blocks_widget.clone()));
        blocks_context_menu.add_separator();
        let delete_block = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "delete_block", "group_formations_delete_block", Some(blocks_widget.clone()));
        let delete_subtree = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "delete_subtree", "group_formations_delete_subtree", Some(blocks_widget));
        blocks_context_menu.add_separator();
        let undo = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "undo", "group_formations_undo", Some(main_widget.static_upcast()));
        let redo = add_action_to_menu(&blocks_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "group_formations", "redo", "group_formations_redo", Some(main_widget.static_upcast()));
        // Block actions also work from the canvas, so their shortcuts work while it has the focus.
        for action in [&add_absolute, &add_relative, &add_span, &delete_block, &delete_subtree] {
            canvas.add_action_q_action(action);
        }

        set_button_action(&widget, "add_absolute_button", &add_absolute)?;
        set_button_action(&widget, "add_relative_button", &add_relative)?;
        set_button_action(&widget, "add_span_button", &add_span)?;
        set_button_action(&widget, "delete_block_button", &delete_block)?;
        set_button_action(&widget, "delete_subtree_button", &delete_subtree)?;
        set_button_action(&widget, "undo_button", &undo)?;
        set_button_action(&widget, "redo_button", &redo)?;

        // Inspector.
        let inspector_stack: QPtr<QStackedWidget> = find_widget(&widget, "inspector_stack")?;
        set_groupbox_title(&widget, "formation_groupbox", "group_formations_formation_title")?;
        set_groupbox_title(&widget, "ai_purpose_groupbox", "group_formations_ai_purpose_title")?;
        set_groupbox_title(&widget, "min_unit_category_groupbox", "group_formations_min_unit_category_title")?;
        set_groupbox_title(&widget, "subcultures_groupbox", "group_formations_subcultures_title")?;
        set_groupbox_title(&widget, "factions_groupbox", "group_formations_factions_title")?;
        set_groupbox_title(&widget, "container_groupbox", "group_formations_container_title")?;
        set_groupbox_title(&widget, "entity_preferences_groupbox", "group_formations_entity_preferences_title")?;
        set_groupbox_title(&widget, "span_groupbox", "group_formations_span_title")?;

        set_label_text(&widget, "name_label", "group_formations_name")?;
        set_label_text(&widget, "ai_priority_label", "group_formations_ai_priority")?;
        set_label_text(&widget, "parent_label", "group_formations_parent")?;
        set_label_text(&widget, "block_priority_label", "group_formations_block_priority")?;
        set_label_text(&widget, "arrangement_label", "group_formations_arrangement")?;
        set_label_text(&widget, "spacing_label", "group_formations_spacing")?;
        set_label_text(&widget, "crescent_y_offset_label", "group_formations_crescent_y_offset")?;
        set_label_text(&widget, "minimum_threshold_label", "group_formations_minimum_threshold")?;
        set_label_text(&widget, "maximum_threshold_label", "group_formations_maximum_threshold")?;
        let uk_2_label = set_label_text(&widget, "uk_2_label", "group_formations_uk_2")?;
        let position_x_label: QPtr<QLabel> = find_widget(&widget, "position_x_label")?;
        let position_y_label: QPtr<QLabel> = find_widget(&widget, "position_y_label")?;

        let name_line_edit: QPtr<QLineEdit> = find_widget(&widget, "name_line_edit")?;
        let ai_priority_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "ai_priority_spinbox")?;
        let uk_2_spinbox: QPtr<QSpinBox> = find_widget(&widget, "uk_2_spinbox")?;
        uk_2_label.set_visible(format.has_formation_uk_2());
        uk_2_spinbox.set_visible(format.has_formation_uk_2());

        let ai_purpose_list_view: QPtr<QListView> = find_widget(&widget, "ai_purpose_list_view")?;
        let ai_purpose_model = QStandardItemModel::new_1a(&ai_purpose_list_view);
        ai_purpose_list_view.set_model(&ai_purpose_model);

        let min_unit_category = ItemList::new(&widget, "min_unit_category_table_view", "add_min_unit_category_button", "remove_min_unit_category_button")?;
        min_unit_category.set_headers(&["group_formations_column_category", "group_formations_column_percentage"]);
        let categories = UnitCategory::ALL.iter().map(|category| category.to_string()).collect::<Vec<_>>();
        min_unit_category.set_combo_delegate(0, &categories, false);
        new_spinbox_item_delegate_safe(&min_unit_category.view.static_upcast::<QObject>().as_ptr(), 1, 32, &Ptr::<QTimer>::null(), false);

        let subcultures = ItemList::new(&widget, "subcultures_list_view", "add_subculture_button", "remove_subculture_button")?;
        let factions = ItemList::new(&widget, "factions_list_view", "add_faction_button", "remove_faction_button")?;
        let subcultures_groupbox: QPtr<QGroupBox> = find_widget(&widget, "subcultures_groupbox")?;
        let factions_groupbox: QPtr<QGroupBox> = find_widget(&widget, "factions_groupbox")?;
        subcultures_groupbox.set_visible(format.has_ai_supported_subcultures());
        factions_groupbox.set_visible(format.has_ai_supported_factions());

        // Subcultures and factions can only be picked from the ones of the game and the open packs.
        if format.has_ai_supported_subcultures() {
            subcultures.set_combo_delegate(0, &dependencies_column_values("cultures_subcultures_tables", "subculture"), false);
        }
        if format.has_ai_supported_factions() {
            factions.set_combo_delegate(0, &dependencies_column_values("factions_tables", "key"), false);
        }

        let parent_combobox: QPtr<QComboBox> = find_widget(&widget, "parent_combobox")?;
        let position_x_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "position_x_spinbox")?;
        let position_y_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "position_y_spinbox")?;
        let block_priority_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "block_priority_spinbox")?;
        let arrangement_combobox: QPtr<QComboBox> = find_widget(&widget, "arrangement_combobox")?;
        let spacing_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "spacing_spinbox")?;
        let crescent_y_offset_spinbox: QPtr<QDoubleSpinBox> = find_widget(&widget, "crescent_y_offset_spinbox")?;
        let minimum_threshold_spinbox: QPtr<QSpinBox> = find_widget(&widget, "minimum_threshold_spinbox")?;
        let maximum_threshold_spinbox: QPtr<QSpinBox> = find_widget(&widget, "maximum_threshold_spinbox")?;
        for arrangement in EntityArrangement::ALL {
            arrangement_combobox.add_item_q_string(&QString::from_std_str(arrangement.to_string()));
        }

        // The maximum's lowest value is -1, which the game treats as no limit.
        maximum_threshold_spinbox.set_special_value_text(&qtr("group_formations_unlimited"));

        let entity_preferences = ItemList::new(&widget, "entity_preferences_table_view", "add_entity_preference_button", "remove_entity_preference_button")?;
        entity_preferences.set_headers(&[
            "group_formations_column_priority",
            "group_formations_column_entity",
            "group_formations_column_weight",
            "group_formations_column_uk_1",
            "group_formations_column_entity_class",
        ]);

        let entities = format.entities().iter().map(|entity| entity.to_string()).collect::<Vec<_>>();
        let weights = EntityWeight::ALL.iter().map(|weight| weight.to_string()).collect::<Vec<_>>();
        let preferences_view = entity_preferences.view.static_upcast::<QObject>().as_ptr();
        new_doublespinbox_item_delegate_safe(&preferences_view, COLUMN_PRIORITY, &Ptr::<QTimer>::null(), false);
        entity_preferences.set_combo_delegate(COLUMN_ENTITY, &entities, false);
        entity_preferences.set_combo_delegate(COLUMN_WEIGHT, &weights, false);
        new_spinbox_item_delegate_safe(&preferences_view, COLUMN_UK_1, 32, &Ptr::<QTimer>::null(), false);
        if format.has_entity_class() {
            entity_preferences.set_combo_delegate(COLUMN_ENTITY_CLASS, &entity_classes(&data), true);
        }

        let preferences_table: QPtr<QTableView> = entity_preferences.view.static_downcast();
        preferences_table.set_column_hidden(COLUMN_WEIGHT, !format.has_entity_weight());
        preferences_table.set_column_hidden(COLUMN_UK_1, !format.has_entity_class());
        preferences_table.set_column_hidden(COLUMN_ENTITY_CLASS, !format.has_entity_class());

        let span_members_list_view: QPtr<QListView> = find_widget(&widget, "span_members_list_view")?;
        let span_members_model = QStandardItemModel::new_1a(&span_members_list_view);
        span_members_list_view.set_model(&span_members_model);

        let diagnostics_timer = QTimer::new_1a(&main_widget);
        diagnostics_timer.set_single_shot(true);
        diagnostics_timer.set_interval(DIAGNOSTICS_UPDATE_DELAY);

        let view = Arc::new(Self {
            path: file_view.path_raw(),
            pack_key: file_view.pack_key().clone(),
            data_source: Arc::new(RwLock::new(file_view.data_source())),

            format,
            is_read_only: file_view.data_source() != DataSource::PackFile,
            data: RwLock::new(data),
            undo_history: RwLock::new(vec![]),
            redo_history: RwLock::new(vec![]),
            is_loading: AtomicBool::new(false),

            formations_filter_line_edit,
            formations_list_view,
            formations_list_filter,
            formations_list_model,
            formations_context_menu,
            add_formation,
            clone_formation,
            delete_formation,

            canvas,
            unit_count_spinbox,
            grid_step_spinbox,
            fit_button,

            blocks_tree_view,
            blocks_tree_model,
            blocks_context_menu,
            add_absolute,
            add_relative,
            add_span,
            delete_block,
            delete_subtree,
            undo,
            redo,

            inspector_stack,

            name_line_edit,
            ai_priority_spinbox,
            uk_2_spinbox,
            ai_purpose_model,
            min_unit_category,
            subcultures,
            factions,

            parent_combobox,
            position_x_label,
            position_x_spinbox,
            position_y_label,
            position_y_spinbox,
            block_priority_spinbox,
            arrangement_combobox,
            spacing_spinbox,
            crescent_y_offset_spinbox,
            minimum_threshold_spinbox,
            maximum_threshold_spinbox,
            entity_preferences,

            span_members_model,

            diagnostics_timer,
        });

        if view.is_read_only {
            view.disable_editing();
        }

        let slots = GroupFormationsSlots::new(&view, app_ui, pack_file_contents_ui, diagnostics_ui);
        connections::set_connections(&view, &slots);

        view.load_formations_list(Some(0));

        file_view.file_type = FileType::GroupFormations;
        file_view.view_type = ViewType::Internal(View::GroupFormations(view));

        Ok(())
    }

    /// Returns a copy of the file being edited, for saving.
    pub fn save_view(&self) -> GroupFormations {
        self.data.read().unwrap().clone()
    }

    /// Replaces the file being edited, dropping the undo history.
    pub unsafe fn reload_view(&self, data: GroupFormations) {
        let selected = self.selected_formation();
        *self.data.write().unwrap() = data;
        self.undo_history.write().unwrap().clear();
        self.redo_history.write().unwrap().clear();
        self.load_formations_list(selected.or(Some(0)));
    }

    /// Returns if the view is filling its widgets, in which case edit slots must ignore changes.
    pub fn is_loading(&self) -> bool {
        self.is_loading.load(Ordering::SeqCst)
    }

    /// Runs the provided function with the loading flag set, so it can fill widgets without triggering edits.
    unsafe fn loading<T>(&self, fill: impl FnOnce() -> T) -> T {
        let was_loading = self.is_loading.swap(true, Ordering::SeqCst);
        let result = fill();
        self.is_loading.store(was_loading, Ordering::SeqCst);
        result
    }

    //-------------------------------------------------------------------------------//
    //                                  Selection
    //-------------------------------------------------------------------------------//

    /// Returns the index of the selected formation, if any.
    pub unsafe fn selected_formation(&self) -> Option<usize> {
        let indexes = self.formations_list_view.selection_model().selected_indexes();
        if indexes.count() != 1 {
            return None;
        }

        let index = self.formations_list_filter.map_to_source(indexes.at(0));
        Some(index.data_1a(ID_ROLE).to_u_int_0a() as usize)
    }

    /// Returns the ids of the selected blocks, in the order they were selected.
    pub unsafe fn selected_blocks(&self) -> Vec<u32> {
        let indexes = self.blocks_tree_view.selection_model().selected_indexes();
        (0..indexes.count())
            .map(|position| indexes.at(position).data_1a(ID_ROLE).to_u_int_0a())
            .collect()
    }

    /// Selects the formation with the provided index, clearing the filter if it hides it.
    pub unsafe fn select_formation(&self, formation_index: usize) {
        if formation_index as i32 >= self.formations_list_model.row_count_0a() {
            return;
        }

        let source = self.formations_list_model.index_2a(formation_index as i32, 0);
        let mut index = self.formations_list_filter.map_from_source(&source);
        if !index.is_valid() {
            self.formations_filter_line_edit.clear();
            self.formations_list_filter.set_filter_fixed_string(&QString::new());
            index = self.formations_list_filter.map_from_source(&source);
        }

        self.formations_list_view.selection_model().select_q_model_index_q_flags_selection_flag(&index, SelectionFlag::ClearAndSelect.into());
        self.formations_list_view.scroll_to_1a(&index);
    }

    /// Selects the blocks with the provided ids in the tree and the canvas, without reloading the inspector.
    unsafe fn select_blocks_silently(&self, block_ids: &[u32]) {
        self.select_tree_blocks(block_ids);
        group_formation_canvas_set_selected_ids_safe(&self.canvas, block_ids);
    }

    /// Selects the blocks with the provided ids in the tree only, without reloading the inspector.
    pub unsafe fn select_tree_blocks(&self, block_ids: &[u32]) {
        let selection_model = self.blocks_tree_view.selection_model();
        let _blocker = QSignalBlocker::from_q_object(&selection_model);
        selection_model.clear_selection();

        for block_id in block_ids {
            if let Some(index) = self.block_index(*block_id) {
                selection_model.select_q_model_index_q_flags_selection_flag(&index, SelectionFlag::Select.into());
                self.blocks_tree_view.scroll_to_1a(&index);
            }
        }

        self.blocks_tree_view.viewport().update();
    }

    /// Selects the blocks with the provided ids in the tree, reloading the inspector.
    pub unsafe fn select_blocks(&self, block_ids: &[u32]) {
        self.select_blocks_silently(block_ids);
        self.load_inspector();
        self.update_actions();
    }

    /// Returns the tree index of a block.
    unsafe fn block_index(&self, block_id: u32) -> Option<cpp_core::CppBox<qt_core::QModelIndex>> {
        let matches = self.blocks_tree_model.match_5a(
            &self.blocks_tree_model.index_2a(0, 0),
            ID_ROLE,
            &QVariant::from_uint(block_id),
            1,
            qt_core::MatchFlag::MatchExactly | qt_core::MatchFlag::MatchRecursive,
        );

        if matches.count() == 1 {
            Some(qt_core::QModelIndex::new_copy(matches.at(0)))
        } else {
            None
        }
    }

    /// Makes the view read-only, leaving only what's needed to browse the file.
    unsafe fn disable_editing(&self) {
        group_formation_canvas_set_editable_safe(&self.canvas, false);

        self.name_line_edit.set_read_only(true);
        self.ai_priority_spinbox.set_read_only(true);
        self.uk_2_spinbox.set_read_only(true);
        self.position_x_spinbox.set_read_only(true);
        self.position_y_spinbox.set_read_only(true);
        self.block_priority_spinbox.set_read_only(true);
        self.spacing_spinbox.set_read_only(true);
        self.crescent_y_offset_spinbox.set_read_only(true);
        self.minimum_threshold_spinbox.set_read_only(true);
        self.maximum_threshold_spinbox.set_read_only(true);
        self.parent_combobox.set_enabled(false);
        self.arrangement_combobox.set_enabled(false);

        for list in [&self.min_unit_category, &self.subcultures, &self.factions, &self.entity_preferences] {
            list.view.set_edit_triggers(EditTrigger::NoEditTriggers.into());
            list.add_button.set_enabled(false);
            list.remove_button.set_enabled(false);
        }
    }

    /// Enables the actions that can be used with the current selection.
    pub unsafe fn update_actions(&self) {
        let has_formation = self.selected_formation().is_some() && !self.is_read_only;
        let blocks = if self.is_read_only { vec![] } else { self.selected_blocks() };

        self.add_formation.set_enabled(!self.is_read_only);
        self.clone_formation.set_enabled(has_formation);
        self.delete_formation.set_enabled(has_formation);
        self.add_absolute.set_enabled(has_formation);
        self.add_relative.set_enabled(blocks.len() == 1);
        self.add_span.set_enabled(!blocks.is_empty());
        self.delete_block.set_enabled(!blocks.is_empty());
        self.delete_subtree.set_enabled(blocks.len() == 1);
        self.undo.set_enabled(!self.is_read_only && !self.undo_history.read().unwrap().is_empty());
        self.redo.set_enabled(!self.is_read_only && !self.redo_history.read().unwrap().is_empty());
    }

    //-------------------------------------------------------------------------------//
    //                                   Editing
    //-------------------------------------------------------------------------------//

    /// Applies an edit to the file, storing a snapshot for undo and marking the file as modified.
    ///
    /// If the edit fails, the file is restored to how it was before it. Edits that change nothing are ignored.
    pub unsafe fn edit(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, edit: impl FnOnce(&mut GroupFormations) -> Result<()>) -> Result<()> {
        let snapshot = self.data.read().unwrap().clone();
        let result = edit(&mut self.data.write().unwrap());
        if result.is_err() {
            *self.data.write().unwrap() = snapshot;
            return result;
        }

        if *self.data.read().unwrap() == snapshot {
            return Ok(());
        }

        let mut undo_history = self.undo_history.write().unwrap();
        undo_history.push(snapshot);
        if undo_history.len() > MAX_UNDO_STEPS {
            undo_history.remove(0);
        }
        drop(undo_history);

        self.redo_history.write().unwrap().clear();
        self.set_modified(app_ui, pack_file_contents_ui);
        self.update_actions();
        Ok(())
    }

    /// Applies an edit to the selected formation.
    pub unsafe fn edit_formation(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, edit: impl FnOnce(&mut GroupFormation) -> Result<()>) -> Result<()> {
        let formation_index = self.selected_formation().ok_or_else(|| anyhow!("No formation selected."))?;
        self.edit(app_ui, pack_file_contents_ui, |data| {
            let formation = data.formations_mut().get_mut(formation_index).ok_or_else(|| anyhow!("The selected formation doesn't exist."))?;
            edit(formation)
        })
    }

    /// Applies an edit to the selected block, if there's only one selected.
    pub unsafe fn edit_block(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>, edit: impl FnOnce(&mut Block)) -> Result<()> {
        let blocks = self.selected_blocks();
        let [block_id] = blocks[..] else { return Err(anyhow!("There must be exactly one block selected.")) };

        self.edit_formation(app_ui, pack_file_contents_ui, |formation| {
            let block = formation.group_formation_blocks_mut()
                .iter_mut()
                .find(|block| *block.block_id() == block_id)
                .ok_or_else(|| anyhow!("The selected block doesn't exist."))?;
            edit(block.block_mut());
            Ok(())
        })
    }

    /// Restores the file to how it was before the last edit.
    pub unsafe fn undo_edit(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let Some(snapshot) = self.undo_history.write().unwrap().pop() else { return };
        let current = std::mem::replace(&mut *self.data.write().unwrap(), snapshot);
        self.redo_history.write().unwrap().push(current);
        self.set_modified(app_ui, pack_file_contents_ui);
        self.reload_after_history_change();
    }

    /// Reapplies the last undone edit.
    pub unsafe fn redo_edit(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        let Some(snapshot) = self.redo_history.write().unwrap().pop() else { return };
        let current = std::mem::replace(&mut *self.data.write().unwrap(), snapshot);
        self.undo_history.write().unwrap().push(current);
        self.set_modified(app_ui, pack_file_contents_ui);
        self.reload_after_history_change();
    }

    /// Reloads the whole view after undo or redo, keeping the selection when it still exists.
    unsafe fn reload_after_history_change(&self) {
        let formation = self.selected_formation();
        let blocks = self.selected_blocks();
        self.load_formations_list(formation);
        self.select_blocks(&blocks);
    }

    /// Marks the file as modified if it belongs to the open Pack, and schedules updating its diagnostics.
    unsafe fn set_modified(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if let DataSource::PackFile = *self.data_source.read().unwrap() {
            rpfm_telemetry::track_action("Modified Group Formations File");
            set_modified(true, &self.path.read().unwrap(), &self.pack_key.read().unwrap(), app_ui, pack_file_contents_ui);
            self.diagnostics_timer.start_0a();
        }
    }

    //-------------------------------------------------------------------------------//
    //                                   Loading
    //-------------------------------------------------------------------------------//

    /// Rebuilds the formations list, and selects the provided formation.
    pub unsafe fn load_formations_list(&self, select: Option<usize>) {
        let names = self.data.read().unwrap().formations().iter().map(|formation| formation.name().to_owned()).collect::<Vec<_>>();
        self.loading(|| {
            self.formations_list_model.clear();
            for (index, name) in names.iter().enumerate() {
                let item = QStandardItem::from_q_string(&QString::from_std_str(name));
                item.set_data_2a(&QVariant::from_uint(index as u32), ID_ROLE);
                item.set_editable(false);
                self.formations_list_model.append_row_q_standard_item(item.into_ptr());
            }
        });

        match select {
            Some(index) if (index as i32) < self.formations_list_model.row_count_0a() => self.select_formation(index),
            Some(_) if self.formations_list_model.row_count_0a() > 0 => self.select_formation(self.formations_list_model.row_count_0a() as usize - 1),
            _ => self.load_formation(),
        }

        self.refresh_issue_icons();
    }

    /// Loads the selected formation into the blocks tree, the canvas and the inspector.
    pub unsafe fn load_formation(&self) {
        self.load_blocks_tree(&[]);
        group_formation_canvas_fit_safe(&self.canvas);
        self.load_inspector();
        self.update_actions();
    }

    /// Returns the size of the grid blocks snap to when moved on the canvas.
    pub unsafe fn grid_step(&self) -> f32 {
        self.grid_step_spinbox.value() as f32
    }

    /// Returns the simulated deployment used for the canvas and to keep blocks in place while editing.
    pub unsafe fn layout_params(&self) -> LayoutParams {
        let mut params = LayoutParams::default();
        params.set_default_unit_count(self.unit_count_spinbox.value().max(1) as u32);
        params
    }

    /// Updates the name of a formation in the list.
    pub unsafe fn update_formation_name(&self, formation_index: usize) {
        let Some(name) = self.data.read().unwrap().formations().get(formation_index).map(|formation| formation.name().to_owned()) else { return };
        let item = self.formations_list_model.item_1a(formation_index as i32);
        if !item.is_null() {
            self.loading(|| item.set_text(&QString::from_std_str(name)));
        }
    }

    /// Rebuilds the blocks tree and the canvas of the selected formation, and selects the provided blocks without reloading the inspector.
    ///
    /// Containers are shown under the block they're positioned relative to. Absolute containers, spans, and
    /// blocks whose parent is missing or part of a reference loop are shown at the top level.
    pub unsafe fn load_blocks_tree(&self, select: &[u32]) {
        let data = self.data.read().unwrap();
        let formation = self.selected_formation().and_then(|index| data.formations().get(index));

        self.loading(|| {
            let _blocker = QSignalBlocker::from_q_object(self.blocks_tree_view.selection_model());
            self.blocks_tree_model.clear();

            let Some(formation) = formation else { return };
            let blocks = formation.group_formation_blocks();
            let issues = block_issues(formation);

            let items = blocks.iter().map(|block| {
                let item = QStandardItem::from_q_string(&QString::from_std_str(block_label(block, self.format)));
                item.set_data_2a(&QVariant::from_uint(*block.block_id()), ID_ROLE);
                item.set_editable(false);

                if let Some(is_error) = issues.get(block.block_id()) {
                    item.set_icon(&QIcon::from_theme_q_string(&QString::from_std_str(if *is_error { "data-error" } else { "data-warning" })));
                }

                item.into_ptr()
            }).collect::<Vec<_>>();

            let positions = blocks.iter()
                .enumerate()
                .rev()
                .map(|(position, block)| (*block.block_id(), position))
                .collect::<HashMap<_, _>>();

            // Place blocks under their parents once the parent is placed. Whatever is left belongs to a loop.
            let mut placed = vec![false; blocks.len()];
            loop {
                let mut changed = false;
                for (position, block) in blocks.iter().enumerate() {
                    if placed[position] {
                        continue;
                    }

                    let parent = match block.block() {
                        Block::ContainerRelative(container) => positions.get(container.relative_block_id()).copied(),
                        Block::ContainerAbsolute(_) | Block::Spanning(_) => None,
                    };

                    match parent {
                        Some(parent) if placed[parent] => items[parent].append_row_q_standard_item(items[position]),
                        Some(parent) if parent != position => continue,
                        _ => self.blocks_tree_model.append_row_q_standard_item(items[position]),
                    }

                    placed[position] = true;
                    changed = true;
                }

                if !changed {
                    break;
                }
            }

            for (position, is_placed) in placed.iter().enumerate() {
                if !is_placed {
                    self.blocks_tree_model.append_row_q_standard_item(items[position]);
                }
            }

            self.blocks_tree_view.expand_all();
        });

        drop(data);
        self.load_canvas();
        self.select_blocks_silently(select);
    }

    /// Redraws the canvas with the simulated layout of the selected formation.
    ///
    /// The canvas uses screen coordinates, where Y grows downwards, so Y is flipped to keep the front of the formation up.
    pub unsafe fn load_canvas(&self) {
        group_formation_canvas_clear_safe(&self.canvas);

        let data = self.data.read().unwrap();
        let Some(formation) = self.selected_formation().and_then(|index| data.formations().get(index)) else { return };
        let rects = formation.layout(&self.layout_params());
        let span_levels = span_levels(formation);

        for block in formation.group_formation_blocks() {
            let Some(rect) = rects.get(block.block_id()) else { continue };
            let block_id = *block.block_id();
            let (kind, label, color, margin) = match block.block() {
                Block::ContainerAbsolute(container) => {
                    let summary = preferences_summary(container.entity_preferences(), self.format);
                    (CanvasBlockKind::AbsoluteContainer, format!("{block_id}: {summary}"), summary_color(&summary), 0.0)
                },
                Block::ContainerRelative(container) => {
                    let summary = preferences_summary(container.entity_preferences(), self.format);
                    (CanvasBlockKind::RelativeContainer, format!("{block_id}: {summary}"), summary_color(&summary), 0.0)
                },
                Block::Spanning(_) => {
                    let level = span_levels.get(&block_id).copied().unwrap_or(1);
                    (CanvasBlockKind::Span, tre("group_formations_canvas_span", &[&block_id.to_string()]), SPAN_COLOR, SPAN_MARGIN * level as f64)
                },
            };

            // Crescent Front bends its middle forward, which is up on the canvas.
            let bend = with_container!(block.block(), container => match container.entity_arrangement() {
                EntityArrangement::CrescentFront => container.crescent_y_offset().abs() as f64,
                EntityArrangement::CrescentBack => -container.crescent_y_offset().abs() as f64,
                EntityArrangement::Line | EntityArrangement::Column => 0.0,
            }).unwrap_or_default();

            group_formation_canvas_add_block_safe(&self.canvas, &CanvasBlock {
                id: block_id,
                kind,
                center: (rect.center_x() as f64, -rect.center_y() as f64),
                size: (rect.width() as f64 + margin * 2.0, rect.height() as f64 + margin * 2.0),
                bend,
                label,
                color,
            });
        }

        for block in formation.group_formation_blocks() {
            if let Block::ContainerRelative(container) = block.block() {
                group_formation_canvas_add_link_safe(&self.canvas, *block.block_id(), *container.relative_block_id());
            }
        }
    }

    /// Fills the inspector with the data of the selected block, or of the selected formation if no single block is selected.
    pub unsafe fn load_inspector(&self) {
        let data = self.data.read().unwrap();
        let Some(formation) = self.selected_formation().and_then(|index| data.formations().get(index)) else {
            self.inspector_stack.set_current_index(PAGE_EMPTY);
            return;
        };

        let blocks = self.selected_blocks();
        let block = match blocks[..] {
            [block_id] => formation.group_formation_blocks().iter().find(|block| *block.block_id() == block_id),
            _ => None,
        };

        self.loading(|| match block {
            Some(block) => match block.block() {
                Block::Spanning(span) => self.load_span(formation, *block.block_id(), span),
                Block::ContainerAbsolute(_) | Block::ContainerRelative(_) => self.load_container(formation, block),
            },
            None => self.load_formation_properties(formation),
        });
    }

    /// Fills the formation page of the inspector.
    unsafe fn load_formation_properties(&self, formation: &GroupFormation) {
        self.name_line_edit.set_text(&QString::from_std_str(formation.name()));
        self.ai_priority_spinbox.set_value(*formation.ai_priority() as f64);
        self.uk_2_spinbox.set_value(*formation.uk_2() as i32);

        self.ai_purpose_model.clear();
        let bits = formation.ai_purpose().bits();
        for (name, bit) in formation.ai_purpose().flags() {
            let item = QStandardItem::from_q_string(&QString::from_std_str(flag_label(name)));
            item.set_checkable(!self.is_read_only);
            item.set_editable(false);
            item.set_check_state(if bits & bit != 0 { CheckState::Checked } else { CheckState::Unchecked });
            item.set_data_2a(&QVariant::from_uint(bit), ID_ROLE);
            self.ai_purpose_model.append_row_q_standard_item(item.into_ptr());
        }

        self.min_unit_category.model.remove_rows_2a(0, self.min_unit_category.model.row_count_0a());
        for requirement in formation.min_unit_category_percentage() {
            self.min_unit_category.append_row(&[
                QVariant::from_q_string(&QString::from_std_str(requirement.category().to_string())),
                QVariant::from_int(*requirement.percentage() as i32),
            ]);
        }

        self.subcultures.set_strings(formation.ai_supported_subcultures());
        self.factions.set_strings(formation.ai_supported_factions());
        self.inspector_stack.set_current_index(PAGE_FORMATION);
    }

    /// Fills the container page of the inspector.
    unsafe fn load_container(&self, formation: &GroupFormation, block: &GroupFormationBlock) {
        let block_id = *block.block_id();
        let parent_id = match block.block() {
            Block::ContainerRelative(container) => Some(*container.relative_block_id()),
            Block::ContainerAbsolute(_) | Block::Spanning(_) => None,
        };

        // Only blocks that don't depend on this one can be its parent, or it'd create a loop.
        self.parent_combobox.clear();
        self.parent_combobox.add_item_q_string_q_variant(&qtr("group_formations_parent_none"), &QVariant::from_int(-1));
        for candidate in formation.group_formation_blocks() {
            if !formation.depends_on(*candidate.block_id(), block_id) {
                self.parent_combobox.add_item_q_string_q_variant(&QString::from_std_str(block_label(candidate, self.format)), &QVariant::from_int(*candidate.block_id() as i32));
            }
        }

        let parent_index = parent_id.map(|parent_id| self.parent_combobox.find_data_1a(&QVariant::from_int(parent_id as i32))).unwrap_or(0);
        self.parent_combobox.set_current_index(parent_index.max(0));

        let (position_x_key, position_y_key) = if parent_id.is_some() {
            ("group_formations_offset_x", "group_formations_offset_y")
        } else {
            ("group_formations_position_x", "group_formations_position_y")
        };
        self.position_x_label.set_text(&qtr(position_x_key));
        self.position_y_label.set_text(&qtr(position_y_key));

        let fields = with_container!(block.block(), container => (
            *container.position_x(),
            *container.position_y(),
            *container.block_priority(),
            *container.entity_arrangement(),
            *container.inter_entity_spacing(),
            *container.crescent_y_offset(),
            *container.minimum_entity_threshold(),
            *container.maximum_entity_threshold(),
            container.entity_preferences(),
        ));

        let Some((position_x, position_y, priority, arrangement, spacing, crescent_y_offset, minimum, maximum, preferences)) = fields else { return };
        self.position_x_spinbox.set_value(position_x as f64);
        self.position_y_spinbox.set_value(position_y as f64);
        self.block_priority_spinbox.set_value(priority as f64);
        self.arrangement_combobox.set_current_index(EntityArrangement::ALL.iter().position(|value| *value == arrangement).unwrap_or(0) as i32);
        self.spacing_spinbox.set_value(spacing as f64);
        self.crescent_y_offset_spinbox.set_value(crescent_y_offset as f64);
        self.minimum_threshold_spinbox.set_value(minimum);
        self.maximum_threshold_spinbox.set_value(maximum.max(-1));

        self.entity_preferences.model.remove_rows_2a(0, self.entity_preferences.model.row_count_0a());
        for preference in preferences {
            self.entity_preferences.append_row(&[
                QVariant::from_double(*preference.priority() as f64),
                QVariant::from_q_string(&QString::from_std_str(preference.entity().to_string())),
                QVariant::from_q_string(&QString::from_std_str(preference.entity_weight().to_string())),
                QVariant::from_int(*preference.uk_1() as i32),
                QVariant::from_q_string(&QString::from_std_str(preference.entity_class())),
            ]);
        }

        self.inspector_stack.set_current_index(PAGE_CONTAINER);
    }

    /// Fills the span page of the inspector, with every block that can be a member of the span.
    unsafe fn load_span(&self, formation: &GroupFormation, span_id: u32, span: &Spanning) {
        self.span_members_model.clear();
        for candidate in formation.group_formation_blocks() {
            let candidate_id = *candidate.block_id();

            // Blocks depending on the span can't be members, or it'd create a loop.
            if formation.depends_on(candidate_id, span_id) {
                continue;
            }

            let item = QStandardItem::from_q_string(&QString::from_std_str(block_label(candidate, self.format)));
            item.set_checkable(!self.is_read_only);
            item.set_editable(false);
            item.set_check_state(if span.spanned_block_ids().contains(&candidate_id) { CheckState::Checked } else { CheckState::Unchecked });
            item.set_data_2a(&QVariant::from_uint(candidate_id), ID_ROLE);
            self.span_members_model.append_row_q_standard_item(item.into_ptr());
        }

        self.inspector_stack.set_current_index(PAGE_SPAN);
    }

    /// Updates the issue icons of the formations list. The issues themselves are listed in the diagnostics panel.
    pub unsafe fn refresh_issue_icons(&self) {
        let mut formation_errors: HashMap<usize, bool> = HashMap::new();
        for (formation_index, issue) in self.data.read().unwrap().validate() {
            *formation_errors.entry(formation_index).or_default() |= issue.is_error();
        }

        self.loading(|| {
            for row in 0..self.formations_list_model.row_count_0a() {
                let item = self.formations_list_model.item_1a(row);
                let icon = match formation_errors.get(&(row as usize)) {
                    Some(true) => QIcon::from_theme_q_string(&QString::from_std_str("data-error")),
                    Some(false) => QIcon::from_theme_q_string(&QString::from_std_str("data-warning")),
                    None => QIcon::new(),
                };
                item.set_icon(&icon);
            }
        });
    }

    /// Refreshes the parts of the view that depend on the selected formation's data, after an edit to it.
    ///
    /// # Arguments
    ///
    /// * `select` - Blocks to select afterwards, or `None` to keep the current selection.
    /// * `reload_inspector` - If the inspector has to be refilled. Edits made from the inspector itself don't need it.
    pub unsafe fn refresh_after_edit(&self, select: Option<&[u32]>, reload_inspector: bool) {
        let selected = match select {
            Some(select) => select.to_vec(),
            None => self.selected_blocks(),
        };

        self.load_blocks_tree(&selected);
        self.refresh_issue_icons();
        if reload_inspector {
            self.load_inspector();
        }
        self.update_actions();
    }

    //-------------------------------------------------------------------------------//
    //                         Reading the inspector's lists
    //-------------------------------------------------------------------------------//

    /// Returns the AI purpose bits checked in the inspector.
    pub unsafe fn checked_ai_purpose_bits(&self) -> u32 {
        (0..self.ai_purpose_model.row_count_0a())
            .map(|row| self.ai_purpose_model.item_1a(row))
            .filter(|item| item.check_state() == CheckState::Checked)
            .fold(0, |bits, item| bits | item.data_1a(ID_ROLE).to_u_int_0a())
    }

    /// Returns the minimum unit category percentages in the inspector.
    pub unsafe fn min_unit_category_rows(&self) -> Vec<MinUnitCategoryPercentage> {
        (0..self.min_unit_category.model.row_count_0a()).map(|row| {
            let mut requirement = MinUnitCategoryPercentage::default();
            let category = self.min_unit_category.text(row, 0);
            if let Some(category) = UnitCategory::ALL.iter().find(|value| value.to_string() == category) {
                requirement.set_category(*category);
            }
            requirement.set_percentage(self.min_unit_category.value(row, 1).to_int_0a().max(0) as u32);
            requirement
        }).collect()
    }

    /// Returns the entity preferences in the inspector.
    pub unsafe fn entity_preference_rows(&self) -> Vec<EntityPreference> {
        let entities = self.format.entities();
        (0..self.entity_preferences.model.row_count_0a()).map(|row| {
            let mut preference = EntityPreference::new(self.format);
            preference.set_priority(self.entity_preferences.value(row, COLUMN_PRIORITY).to_double_0a() as f32);

            let entity = self.entity_preferences.text(row, COLUMN_ENTITY);
            if let Some(entity) = entities.iter().find(|value| value.to_string() == entity) {
                preference.set_entity(entity.clone());
            }

            let weight = self.entity_preferences.text(row, COLUMN_WEIGHT);
            if let Some(weight) = EntityWeight::ALL.iter().find(|value| value.to_string() == weight) {
                preference.set_entity_weight(*weight);
            }

            if self.format.has_entity_class() {
                preference.set_uk_1(self.entity_preferences.value(row, COLUMN_UK_1).to_int_0a().max(0) as u32);
                preference.set_entity_class(self.entity_preferences.text(row, COLUMN_ENTITY_CLASS));
            }

            preference
        }).collect()
    }

    /// Returns the ids of the blocks checked as span members in the inspector.
    pub unsafe fn checked_span_members(&self) -> Vec<u32> {
        (0..self.span_members_model.row_count_0a())
            .map(|row| self.span_members_model.item_1a(row))
            .filter(|item| item.check_state() == CheckState::Checked)
            .map(|item| item.data_1a(ID_ROLE).to_u_int_0a())
            .collect()
    }

    /// Returns a formation name not used by any formation, based on the provided one.
    pub fn unique_formation_name(&self, base: &str) -> String {
        let data = self.data.read().unwrap();
        let names = data.formations().iter().map(|formation| formation.name().as_str()).collect::<HashSet<_>>();
        if !names.contains(base) {
            return base.to_owned();
        }

        (1..).map(|index| format!("{base}_{index}"))
            .find(|name| !names.contains(name.as_str()))
            .expect("there's always an unused name")
    }
}

impl ItemList {

    /// Finds the widgets of a list in the template, and gives it an empty model.
    unsafe fn new(widget: &QPtr<QWidget>, view_name: &str, add_button_name: &str, remove_button_name: &str) -> Result<Self> {
        let view: QPtr<QAbstractItemView> = find_widget(widget, view_name)?;
        let model = QStandardItemModel::new_1a(&view);
        view.set_model(&model);
        view.set_selection_mode(SelectionMode::ExtendedSelection);

        let add_button: QPtr<QToolButton> = find_widget(widget, add_button_name)?;
        let remove_button: QPtr<QToolButton> = find_widget(widget, remove_button_name)?;
        add_button.set_icon(&QIcon::from_theme_q_string(&QString::from_std_str("list-add")));
        add_button.set_tool_tip(&qtr("group_formations_add_item"));
        remove_button.set_icon(&QIcon::from_theme_q_string(&QString::from_std_str("list-remove")));
        remove_button.set_tool_tip(&qtr("group_formations_remove_item"));

        Ok(Self {
            view,
            model,
            add_button,
            remove_button,
        })
    }

    /// Sets the translated column headers of a table, and makes its last column fill the rest of its width.
    unsafe fn set_headers(&self, keys: &[&str]) {
        let headers = QListOfQString::new_0a();
        for key in keys {
            headers.append_q_string(&qtr(key));
        }
        self.model.set_horizontal_header_labels(&headers);

        let table: QPtr<QTableView> = self.view.static_downcast();
        table.horizontal_header().set_stretch_last_section(true);
    }

    /// Makes a column editable through a combobox with the provided values.
    unsafe fn set_combo_delegate(&self, column: i32, values: &[String], is_editable: bool) {
        let list = QListOfQString::new_0a();
        for value in values {
            list.append_q_string(&QString::from_std_str(value));
        }

        new_combobox_item_delegate_safe(&self.view.static_upcast::<QObject>().as_ptr(), column, list.as_ptr(), QListOfQString::new_0a().as_ptr(), is_editable, &Ptr::<QTimer>::null(), false);
    }

    /// Appends a row with the provided values, one per column.
    unsafe fn append_row(&self, values: &[cpp_core::CppBox<QVariant>]) {
        let row = qt_gui::QListOfQStandardItem::new_0a();
        for value in values {
            let item = QStandardItem::new();
            item.set_data_2a(value, ItemDataRole::EditRole.to_int());
            row.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
        }
        self.model.append_row_q_list_of_q_standard_item(&row);
    }

    /// Replaces the rows of a single-column list with the provided strings.
    unsafe fn set_strings(&self, values: &[String]) {
        self.model.clear();
        for value in values {
            self.model.append_row_q_standard_item(QStandardItem::from_q_string(&QString::from_std_str(value)).into_ptr());
        }
    }

    /// Returns the non-empty strings of a single-column list.
    pub unsafe fn strings(&self) -> Vec<String> {
        (0..self.model.row_count_0a())
            .map(|row| self.text(row, 0))
            .filter(|value| !value.is_empty())
            .collect()
    }

    /// Returns the value of a cell.
    unsafe fn value(&self, row: i32, column: i32) -> cpp_core::CppBox<QVariant> {
        self.model.index_2a(row, column).data_1a(ItemDataRole::EditRole.to_int())
    }

    /// Returns the value of a cell as text.
    unsafe fn text(&self, row: i32, column: i32) -> String {
        self.value(row, column).to_string().to_std_string()
    }

    /// Removes the selected rows.
    pub unsafe fn remove_selected_rows(&self) {
        let indexes = self.view.selection_model().selected_indexes();
        let rows = (0..indexes.count()).map(|position| indexes.at(position).row()).collect::<BTreeSet<_>>();
        for row in rows.into_iter().rev() {
            self.model.remove_row_1a(row);
        }
    }
}

impl FileGroupFormationsDebugView {

    /// This function creates a new debug view for a GroupFormations file.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: GroupFormations
    ) -> Result<()> {
        let debug_view = DebugView::new_view(
            file_view.main_widget(),
            RFileDecoded::GroupFormations(data),
            file_view.path_raw(),
            file_view.pack_key().clone(),
        )?;

        let view = Self {
            debug_view,
        };

        file_view.view_type = ViewType::Internal(View::GroupFormationsDebug(Arc::new(view)));
        file_view.file_type = FileType::GroupFormations;

        Ok(())
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &GroupFormations) -> Result<()> {
        self.debug_view.reload_view(&serde_json::to_string_pretty(&data)?);

        Ok(())
    }
}

//-------------------------------------------------------------------------------//
//                                  Helpers
//-------------------------------------------------------------------------------//

/// Makes a template button trigger an action, taking its icon and tooltip.
unsafe fn set_button_action(widget: &QPtr<QWidget>, button_name: &str, action: &QPtr<QAction>) -> Result<()> {
    let button: QPtr<QToolButton> = find_widget(widget, button_name)?;
    button.set_default_action(action);
    Ok(())
}

/// Sets the translated title of a template groupbox.
unsafe fn set_groupbox_title(widget: &QPtr<QWidget>, groupbox_name: &str, key: &str) -> Result<()> {
    let groupbox: QPtr<QGroupBox> = find_widget(widget, groupbox_name)?;
    groupbox.set_title(&qtr(key));
    Ok(())
}

/// Sets the translated text of a template label, and returns the label.
unsafe fn set_label_text(widget: &QPtr<QWidget>, label_name: &str, key: &str) -> Result<QPtr<QLabel>> {
    let label: QPtr<QLabel> = find_widget(widget, label_name)?;
    label.set_text(&qtr(key));
    Ok(label)
}

/// Returns the text used to show a block in lists, with its id, kind and main entity.
fn block_label(block: &GroupFormationBlock, format: GroupFormationsFormat) -> String {
    let block_id = block.block_id().to_string();
    match block.block() {
        Block::ContainerAbsolute(container) => format!("{} [{}]", tre("group_formations_block_absolute", &[&block_id]), preferences_summary(container.entity_preferences(), format)),
        Block::ContainerRelative(container) => {
            let parent_id = container.relative_block_id().to_string();
            let position_x = container.position_x().to_string();
            let position_y = container.position_y().to_string();
            format!("{} [{}]", tre("group_formations_block_relative", &[&block_id, &parent_id, &position_x, &position_y]), preferences_summary(container.entity_preferences(), format))
        },
        Block::Spanning(span) => {
            let members = span.spanned_block_ids().iter().map(|member_id| member_id.to_string()).collect::<Vec<_>>().join(", ");
            tre("group_formations_block_span", &[&block_id, &members])
        },
    }
}

/// Returns a short description of the units preferred by a container.
fn preferences_summary(preferences: &[EntityPreference], format: GroupFormationsFormat) -> String {
    let Some(first) = preferences.first() else { return String::new() };
    let name = if format.has_entity_class() && !first.entity_class().is_empty() {
        first.entity_class().to_owned()
    } else {
        first.entity().to_string()
    };

    match preferences.len() {
        1 => name,
        count => format!("{name} +{}", count - 1),
    }
}

/// Returns the canvas color of a container, from the summary of its entity preferences.
///
/// Uses the FNV-1a hash of the summary, so the same units always get the same color.
fn summary_color(summary: &str) -> (u8, u8, u8) {
    let hash = summary.bytes().fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ byte as u64).wrapping_mul(0x100000001b3));
    CONTAINER_COLORS[(hash % CONTAINER_COLORS.len() as u64) as usize]
}

/// Returns how many levels of spans each span contains, counting itself, so outer spans can be drawn bigger.
fn span_levels(formation: &GroupFormation) -> HashMap<u32, u32> {
    let spans = formation.group_formation_blocks()
        .iter()
        .filter_map(|block| match block.block() {
            Block::Spanning(span) => Some((*block.block_id(), span.spanned_block_ids())),
            Block::ContainerAbsolute(_) | Block::ContainerRelative(_) => None,
        })
        .collect::<HashMap<_, _>>();

    // Each pass can only add one level, so a span cycle stops growing once passes reach the span count.
    let mut levels = spans.keys().map(|span_id| (*span_id, 1)).collect::<HashMap<_, _>>();
    for _ in 0..spans.len() {
        let mut changed = false;
        for (span_id, members) in &spans {
            let level = 1 + members.iter().filter_map(|member_id| levels.get(member_id)).max().copied().unwrap_or(0);
            if levels.get(span_id) != Some(&level) {
                levels.insert(*span_id, level);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    levels
}

/// Turns a flag name like `NAVAL_ATTACK` into `Naval Attack`.
fn flag_label(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars.flat_map(char::to_lowercase)).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns the sorted values of a DB table column, from the game and the open packs.
fn dependencies_column_values(table_name: &str, column_name: &str) -> Vec<String> {
    let request = GetColumnValues {
        table_name: table_name.to_owned(),
        column: column_name.to_owned(),
        include_packs: true,
        include_dependencies: true,
        prefix: String::new(),
        offset: 0,
        limit: Some(usize::MAX),
    };

    call_api_async(&request).map(|values| values.values).unwrap_or_default()
}

/// Returns the sorted entity classes of the game, plus the ones used in the file and the generic one.
///
/// Entity classes are the keys of the `ai_usage_groups` table, which units use to tell the AI how to use them.
fn entity_classes(data: &GroupFormations) -> Vec<String> {
    let mut classes = data.formations()
        .iter()
        .flat_map(|formation| formation.group_formation_blocks())
        .filter_map(|block| with_container!(block.block(), container => container.entity_preferences()))
        .flatten()
        .map(|preference| preference.entity_class().to_owned())
        .filter(|class| !class.is_empty())
        .collect::<BTreeSet<_>>();

    classes.insert("any".to_owned());
    classes.extend(dependencies_column_values("ai_usage_groups_tables", "key"));
    classes.into_iter().collect()
}

/// Returns the blocks of a formation with issues, and if any of their issues is an error.
fn block_issues(formation: &GroupFormation) -> HashMap<u32, bool> {
    let mut blocks = HashMap::new();
    for issue in formation.validate() {
        let is_error = issue.is_error();
        let block_ids = match &issue {
            ValidationIssue::ReferenceCycle(block_ids) => block_ids.clone(),
            _ => issue_block(&issue).into_iter().collect(),
        };

        for block_id in block_ids {
            *blocks.entry(block_id).or_default() |= is_error;
        }
    }

    blocks
}

/// Returns the block an issue points to, if it points to a specific one.
fn issue_block(issue: &ValidationIssue) -> Option<u32> {
    match issue {
        ValidationIssue::DuplicateFormationName |
        ValidationIssue::NoAbsoluteBlock => None,
        ValidationIssue::DuplicateBlockId(block_id) |
        ValidationIssue::EmptySpan(block_id) |
        ValidationIssue::InvalidThresholds(block_id) |
        ValidationIssue::NoEntityPreferences(block_id) |
        ValidationIssue::MissingReference { block_id, .. } |
        ValidationIssue::ForwardReference { block_id, .. } => Some(*block_id),
        ValidationIssue::ReferenceCycle(block_ids) => block_ids.first().copied(),
    }
}
