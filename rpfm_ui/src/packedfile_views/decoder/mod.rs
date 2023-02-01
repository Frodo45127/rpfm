//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module implementing the DB Decoder.

use qt_widgets::q_abstract_item_view::{EditTrigger, SelectionMode};
use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QFrame;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QAction;
use qt_widgets::QMenu;
use qt_widgets::QGridLayout;
use qt_widgets::QGroupBox;
use qt_widgets::QTableView;
use qt_widgets::QTreeView;
use qt_widgets::QPushButton;
use qt_widgets::QTextEdit;
use qt_widgets::QSpinBox;

use qt_gui::QBrush;
use qt_gui::QFontMetrics;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QTextCharFormat;
use qt_gui::q_text_cursor::{MoveOperation, MoveMode};

use qt_core::QBox;
use qt_core::ContextMenuPolicy;
use qt_core::GlobalColor;
use qt_core::QSignalBlocker;
use qt_core::QString;
use qt_core::SortOrder;
use qt_core::QFlags;
use qt_core::QVariant;
use qt_core::Orientation;
use qt_core::QObject;
use qt_core::CheckState;
use qt_core::QTimer;
use qt_core::QStringList;
use qt_core::QModelIndex;
use qt_core::QPtr;

use cpp_core::CppBox;

use anyhow::{anyhow, Result};
use getset::Getters;

use std::collections::BTreeMap;
use std::io::{Cursor, Seek, SeekFrom};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::binary::ReadBytes;
use rpfm_lib::integrations::assembly_kit::{get_raw_definition_paths, table_definition::RawDefinition, table_data::RawTable, localisable_fields::RawLocalisableFields};
use rpfm_lib::files::{ContainerPath, db::DB, table::DecodedData};
use rpfm_lib::schema::*;

use crate::app_ui::AppUI;
use crate::assembly_kit_path;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::{new_combobox_item_delegate_safe, new_spinbox_item_delegate_safe, new_qstring_item_delegate_safe};
use crate::FONT_MONOSPACE;
use crate::GAME_SELECTED;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, View, ViewType};
use crate::SCHEMA;
use crate::setting_bool;
use crate::utils::*;

use self::slots::PackedFileDecoderViewSlots;

pub mod connections;
pub mod slots;

pub const DECODER_EXTENSION: &str = "-rpfm-decoder";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackedFile Decoder.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct PackedFileDecoderView {
    hex_view_index: QBox<QTextEdit>,
    hex_view_raw: QBox<QTextEdit>,
    hex_view_decoded: QBox<QTextEdit>,

    table_view: QBox<QTreeView>,
    table_model: QBox<QStandardItemModel>,

    table_view_context_menu: QBox<QMenu>,
    table_view_context_menu_move_up: QPtr<QAction>,
    table_view_context_menu_move_down: QPtr<QAction>,
    table_view_context_menu_move_left: QPtr<QAction>,
    table_view_context_menu_move_right: QPtr<QAction>,
    table_view_context_menu_delete: QPtr<QAction>,

    bool_line_edit: QBox<QLineEdit>,
    f32_line_edit: QBox<QLineEdit>,
    f64_line_edit: QBox<QLineEdit>,
    i16_line_edit: QBox<QLineEdit>,
    i32_line_edit: QBox<QLineEdit>,
    i64_line_edit: QBox<QLineEdit>,
    optional_i16_line_edit: QBox<QLineEdit>,
    optional_i32_line_edit: QBox<QLineEdit>,
    optional_i64_line_edit: QBox<QLineEdit>,
    colour_rgb_line_edit: QBox<QLineEdit>,
    string_u8_line_edit: QBox<QLineEdit>,
    string_u16_line_edit: QBox<QLineEdit>,
    optional_string_u8_line_edit: QBox<QLineEdit>,
    optional_string_u16_line_edit: QBox<QLineEdit>,
    sequence_u32_line_edit: QBox<QLineEdit>,

    bool_button: QBox<QPushButton>,
    f32_button: QBox<QPushButton>,
    f64_button: QBox<QPushButton>,
    i16_button: QBox<QPushButton>,
    i32_button: QBox<QPushButton>,
    i64_button: QBox<QPushButton>,
    optional_i16_button: QBox<QPushButton>,
    optional_i32_button: QBox<QPushButton>,
    optional_i64_button: QBox<QPushButton>,
    colour_rgb_button: QBox<QPushButton>,
    string_u8_button: QBox<QPushButton>,
    string_u16_button: QBox<QPushButton>,
    optional_string_u8_button: QBox<QPushButton>,
    optional_string_u16_button: QBox<QPushButton>,
    sequence_u32_button: QBox<QPushButton>,

    packed_file_info_version_decoded_spinbox: QBox<QSpinBox>,
    packed_file_info_entry_count_decoded_label: QBox<QLabel>,

    table_view_old_versions: QBox<QTableView>,
    table_model_old_versions: QBox<QStandardItemModel>,

    table_view_old_versions_context_menu: QBox<QMenu>,
    table_view_old_versions_context_menu_load: QPtr<QAction>,
    table_view_old_versions_context_menu_delete: QPtr<QAction>,

    import_from_assembly_kit_button: QBox<QPushButton>,
    test_definition_button: QBox<QPushButton>,
    clear_definition_button: QBox<QPushButton>,
    save_button: QBox<QPushButton>,

    packed_file_path: String,
    data: Arc<RwLock<Cursor<Vec<u8>>>>,
    table_name: String,
    version: i32,
    entry_count: u32,
    header_size: u64,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileDecoderView`.
impl PackedFileDecoderView {

    /// This function creates a new Decoder View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        path: &str,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<()> {

        let container_path = ContainerPath::File(packed_file_view.get_path());
        let table_name = container_path.db_table_name_from_path()
            .ok_or_else(|| anyhow!("The decoder cannot be use for this file."))?;

        let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFileRawData(path.to_owned()));
        let response = CentralCommand::recv(&receiver);
        let mut data = match response {
            Response::VecU8(data) => Cursor::new(data),
            Response::Error(error) => return Err(error),
            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        };

        // Create the hex view on the left side.
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        //---------------------------------------------//
        // Hex Data section.
        //---------------------------------------------//

        let hex_view_group = QGroupBox::from_q_string_q_widget(&QString::from_std_str("PackedFile's Data"), packed_file_view.get_mut_widget());
        let hex_view_index = QTextEdit::from_q_widget(&hex_view_group);
        let hex_view_raw = QTextEdit::from_q_widget(&hex_view_group);
        let hex_view_decoded = QTextEdit::from_q_widget(&hex_view_group);
        let hex_view_layout = create_grid_layout(hex_view_group.static_upcast());

        hex_view_index.set_font(ref_from_atomic(&*FONT_MONOSPACE));
        hex_view_raw.set_font(ref_from_atomic(&*FONT_MONOSPACE));
        hex_view_decoded.set_font(ref_from_atomic(&*FONT_MONOSPACE));

        hex_view_layout.add_widget_5a(& hex_view_index, 0, 0, 1, 1);
        hex_view_layout.add_widget_5a(& hex_view_raw, 0, 1, 1, 1);
        hex_view_layout.add_widget_5a(& hex_view_decoded, 0, 2, 1, 1);

        layout.add_widget_5a(&hex_view_group, 0, 0, 5, 1);

        //---------------------------------------------//
        // Fields Table section.
        //---------------------------------------------//

        let table_view = QTreeView::new_1a(packed_file_view.get_mut_widget());
        let table_model = QStandardItemModel::new_1a(&table_view);
        table_view.set_model(&table_model);
        table_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        //table_view.header().set_stretch_last_section(true);
        table_view.set_alternating_row_colors(true);

        // Create the Contextual Menu for the TableView.
        let table_view_context_menu = QMenu::from_q_widget(&table_view);

        // Create the Contextual Menu Actions.
        let table_view_context_menu_move_up = add_action_to_menu(&table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "move_field_up", "move_field_up", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let table_view_context_menu_move_down = add_action_to_menu(&table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "move_field_down", "move_field_down", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let table_view_context_menu_move_left = add_action_to_menu(&table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "move_field_left", "move_field_left", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let table_view_context_menu_move_right = add_action_to_menu(&table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "move_field_right", "move_field_right", Some(table_view.static_upcast::<qt_widgets::QWidget>()));
        let table_view_context_menu_delete = add_action_to_menu(&table_view_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "delete_field", "delete_field", Some(table_view.static_upcast::<qt_widgets::QWidget>()));

        // Disable them by default.
        table_view_context_menu_move_up.set_enabled(false);
        table_view_context_menu_move_down.set_enabled(false);
        table_view_context_menu_move_left.set_enabled(false);
        table_view_context_menu_move_right.set_enabled(false);
        table_view_context_menu_delete.set_enabled(false);

        layout.add_widget_5a(&table_view, 0, 1, 1, 2);

        //---------------------------------------------//
        // Decoded Fields section.
        //---------------------------------------------//

        let decoded_fields_frame = QGroupBox::from_q_string_q_widget(&QString::from_std_str("Current Field Decoded"), packed_file_view.get_mut_widget());
        let decoded_fields_layout = create_grid_layout(decoded_fields_frame.static_upcast());
        decoded_fields_layout.set_column_stretch(1, 10);

        // Create the stuff for the decoded fields.
        let bool_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Bool\":"), &decoded_fields_frame);
        let f32_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"F32\":"), &decoded_fields_frame);
        let f64_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"F64\":"), &decoded_fields_frame);
        let i16_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"I16\":"), &decoded_fields_frame);
        let i32_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"I32\":"), &decoded_fields_frame);
        let i64_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"I64\":"), &decoded_fields_frame);
        let optional_i16_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Optional I16\":"), &decoded_fields_frame);
        let optional_i32_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Optional I32\":"), &decoded_fields_frame);
        let optional_i64_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Optional I64\":"), &decoded_fields_frame);
        let colour_rgb_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Colour (RGB)\":"), &decoded_fields_frame);
        let string_u8_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"String U8\":"), &decoded_fields_frame);
        let string_u16_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"String U16\":"), &decoded_fields_frame);
        let optional_string_u8_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Optional String U8\":"), &decoded_fields_frame);
        let optional_string_u16_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"Optional String U16\":"), &decoded_fields_frame);
        let sequence_u32_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Decoded as \"SequenceU32\":"), &decoded_fields_frame);

        let bool_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let f32_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let f64_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let i16_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let i32_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let i64_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let optional_i16_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let optional_i32_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let optional_i64_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let colour_rgb_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let string_u8_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let string_u16_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let optional_string_u8_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let optional_string_u16_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);
        let sequence_u32_line_edit = QLineEdit::from_q_widget(&decoded_fields_frame);

        let bool_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let f32_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let f64_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let i16_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let i32_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let i64_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let optional_i16_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let optional_i32_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let optional_i64_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let colour_rgb_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let string_u8_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let string_u16_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let optional_string_u8_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let optional_string_u16_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);
        let sequence_u32_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Use this"), &decoded_fields_frame);

        decoded_fields_layout.add_widget_5a(&bool_label, 0, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&f32_label, 1, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&f64_label, 2, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&i16_label, 3, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&i32_label, 4, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&i64_label, 5, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i16_label, 6, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i32_label, 7, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i64_label, 8, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&colour_rgb_label, 9, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u8_label, 10, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u16_label, 11, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u8_label, 12, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u16_label, 13, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(&sequence_u32_label, 14, 0, 1, 1);

        decoded_fields_layout.add_widget_5a(&bool_line_edit, 0, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&f32_line_edit, 1, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&f64_line_edit, 2, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&i16_line_edit, 3, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&i32_line_edit, 4, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&i64_line_edit, 5, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i16_line_edit, 6, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i32_line_edit, 7, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i64_line_edit, 8, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&colour_rgb_line_edit, 9, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u8_line_edit, 10, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u16_line_edit, 11, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u8_line_edit, 12, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u16_line_edit, 13, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&sequence_u32_line_edit, 14, 1, 1, 1);

        decoded_fields_layout.add_widget_5a(&bool_button, 0, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&f32_button, 1, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&f64_button, 2, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&i16_button, 3, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&i32_button, 4, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&i64_button, 5, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i16_button, 6, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i32_button, 7, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_i64_button, 8, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&colour_rgb_button, 9, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u8_button, 10, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&string_u16_button, 11, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u8_button, 12, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&optional_string_u16_button, 13, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&sequence_u32_button, 14, 2, 1, 1);

        layout.add_widget_5a(&decoded_fields_frame, 1, 1, 3, 1);

        //---------------------------------------------//
        // Info section.
        //---------------------------------------------//

        let info_frame = QGroupBox::from_q_string_q_widget(&QString::from_std_str("PackedFile Info"), packed_file_view.get_mut_widget());
        let info_layout = create_grid_layout(info_frame.static_upcast());

        // Create stuff for the info frame.
        let packed_file_info_type_label = QLabel::from_q_string_q_widget(&QString::from_std_str("PackedFile Type:"), &info_frame);
        let packed_file_info_version_label = QLabel::from_q_string_q_widget(&QString::from_std_str("PackedFile version:"), &info_frame);
        let packed_file_info_entry_count_label = QLabel::from_q_string_q_widget(&QString::from_std_str("PackedFile entry count:"), &info_frame);

        let packed_file_info_type_decoded_label = QLabel::from_q_string_q_widget(&QString::from_std_str(table_name), &info_frame);
        let packed_file_info_version_decoded_spinbox = QSpinBox::new_1a(&info_frame);
        let packed_file_info_entry_count_decoded_label = QLabel::from_q_widget(&info_frame);

        info_layout.add_widget_5a(&packed_file_info_type_label, 0, 0, 1, 1);
        info_layout.add_widget_5a(&packed_file_info_version_label, 1, 0, 1, 1);

        info_layout.add_widget_5a(&packed_file_info_type_decoded_label, 0, 1, 1, 1);
        info_layout.add_widget_5a(&packed_file_info_version_decoded_spinbox, 1, 1, 1, 1);

        info_layout.add_widget_5a(&packed_file_info_entry_count_label, 2, 0, 1, 1);
        info_layout.add_widget_5a(&packed_file_info_entry_count_decoded_label, 2, 1, 1, 1);

        layout.add_widget_5a(&info_frame, 1, 2, 1, 1);

        //---------------------------------------------//
        // Other Versions section.
        //---------------------------------------------//

        let table_view_old_versions = QTableView::new_1a(packed_file_view.get_mut_widget());
        let table_model_old_versions = QStandardItemModel::new_1a(&table_view_old_versions);
        table_view_old_versions.set_model(&table_model_old_versions);
        table_view_old_versions.set_alternating_row_colors(true);
        table_view_old_versions.set_edit_triggers(QFlags::from(EditTrigger::NoEditTriggers));
        table_view_old_versions.set_selection_mode(SelectionMode::SingleSelection);
        table_view_old_versions.set_sorting_enabled(true);
        table_view_old_versions.sort_by_column_2a(0, SortOrder::AscendingOrder);
        table_view_old_versions.vertical_header().set_visible(false);
        table_view_old_versions.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

        let table_view_old_versions_context_menu = QMenu::new();
        let table_view_old_versions_context_menu_load = add_action_to_menu(&table_view_old_versions_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "load_definition", "load_definition", Some(table_view_old_versions.static_upcast::<qt_widgets::QWidget>()));
        let table_view_old_versions_context_menu_delete = add_action_to_menu(&table_view_old_versions_context_menu.static_upcast(), app_ui.shortcuts().as_ref(), "decoder", "delete_definition", "delete_definition", Some(table_view_old_versions.static_upcast::<qt_widgets::QWidget>()));
        table_view_old_versions_context_menu_load.set_enabled(false);
        table_view_old_versions_context_menu_delete.set_enabled(false);

        layout.add_widget_5a(&table_view_old_versions, 2, 2, 1, 1);

        //---------------------------------------------//
        // Buttons section.
        //---------------------------------------------//

        let button_box = QFrame::new_1a(packed_file_view.get_mut_widget());
        let button_box_layout = create_grid_layout(button_box.static_upcast());

        // Create the bottom Buttons.
        let import_from_assembly_kit_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Import from Assembly Kit"), &button_box);
        let test_definition_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Test Definition"), &button_box);
        let clear_definition_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Remove all fields"), &button_box);
        let save_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Finish it!"), &button_box);

        // Add them to the Dialog.
        button_box_layout.add_widget_5a(&import_from_assembly_kit_button, 0, 0, 1, 1);
        button_box_layout.add_widget_5a(&test_definition_button, 0, 1, 1, 1);
        button_box_layout.add_widget_5a(&clear_definition_button, 0, 2, 1, 1);
        button_box_layout.add_widget_5a(&save_button, 0, 3, 1, 1);

        layout.add_widget_5a(&button_box, 4, 1, 1, 2);

        layout.set_column_stretch(1, 10);
        layout.set_row_stretch(0, 10);
        layout.set_row_stretch(2, 5);

        let (version,_,_,entry_count) = DB::read_header(&mut data)?;
        let header_size = data.position();
        let packed_file_decoder_view = Arc::new(PackedFileDecoderView {
            hex_view_index,
            hex_view_raw,
            hex_view_decoded,

            table_view,
            table_model,

            table_view_context_menu,
            table_view_context_menu_move_up,
            table_view_context_menu_move_down,
            table_view_context_menu_move_left,
            table_view_context_menu_move_right,
            table_view_context_menu_delete,

            bool_line_edit,
            f32_line_edit,
            f64_line_edit,
            i16_line_edit,
            i32_line_edit,
            i64_line_edit,
            optional_i16_line_edit,
            optional_i32_line_edit,
            optional_i64_line_edit,
            string_u8_line_edit,
            string_u16_line_edit,
            optional_string_u8_line_edit,
            optional_string_u16_line_edit,
            colour_rgb_line_edit,
            sequence_u32_line_edit,

            bool_button,
            f32_button,
            f64_button,
            i16_button,
            i32_button,
            i64_button,
            optional_i16_button,
            optional_i32_button,
            optional_i64_button,
            string_u8_button,
            string_u16_button,
            optional_string_u8_button,
            optional_string_u16_button,
            colour_rgb_button,
            sequence_u32_button,

            packed_file_info_version_decoded_spinbox,
            packed_file_info_entry_count_decoded_label,

            table_view_old_versions,
            table_model_old_versions,

            table_view_old_versions_context_menu,
            table_view_old_versions_context_menu_load,
            table_view_old_versions_context_menu_delete,

            import_from_assembly_kit_button,
            test_definition_button,
            clear_definition_button,
            save_button,

            packed_file_path: packed_file_view.get_path(),
            data: Arc::new(RwLock::new(data)),
            table_name: table_name.to_owned(),
            version,
            entry_count,
            header_size,
        });

        let packed_file_decoder_view_slots = PackedFileDecoderViewSlots::new(
            &packed_file_decoder_view,
            app_ui,
            pack_file_contents_ui
        );

        let definition = packed_file_decoder_view.definition();

        let fields = if let Some(definition) = definition {
            definition.fields().to_vec()
        } else { vec![] };

        packed_file_decoder_view.load_data()?;
        packed_file_decoder_view.load_versions_list();
        packed_file_decoder_view.update_view(&fields, true)?;
        packed_file_decoder_view.update_rows_decoded(None, None)?;
        connections::set_connections(&packed_file_decoder_view, &packed_file_decoder_view_slots);
        packed_file_view.view = ViewType::Internal(View::Decoder(packed_file_decoder_view));

        // Update the path so the decoder is identified as a separate file.
        let mut path = packed_file_view.get_path();
        path.push_str(DECODER_EXTENSION);
        packed_file_view.set_path(&path);

        // Return success.
        Ok(())
    }

    /// This function loads the raw data of a PackedFile into the UI and prepare it to be updated later on.
    pub unsafe fn load_data(&self) -> Result<()> {

        // We need to set up the fonts in a specific way, so the scroll/sizes are kept correct.
        let font = self.hex_view_index().document().default_font();
        let font_metrics = QFontMetrics::new_1a(&font);

        //---------------------------------------------//
        // Index section.
        //---------------------------------------------//

        // This creates the "index" column at the left of the hex data. The logic behind this, because
        // even I have problems to understand it:
        // - Lines are 4 packs of 4 bytes => 16 bytes + 3 spaces + 1 line jump.
        // - Amount of lines is "bytes we have / 16 + 1" (+ 1 because we want to show incomplete lines too).
        // - Then, for the zeroes, we default to 4, meaning all lines are 00XX.
        let mut hex_index = String::new();
        let hex_lines = (self.data.write().unwrap().len()? / 16) + 1;
        (0..hex_lines).for_each(|x| hex_index.push_str(&format!("{:>0count$X}\n", x * 16, count = 4)));

        let qhex_index = QString::from_std_str(&hex_index);
        let text_size = font_metrics.size_2a(0, &qhex_index);
        self.hex_view_index().set_text(&qhex_index);
        self.hex_view_index().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Raw data section.
        //---------------------------------------------//

        // Prepare the Hex Raw Data string, looking like:
        // 01 0a 02 0f 0d 02 04 06 01 0a 02 0f 0d 02 04 06
        let mut hex_raw_data = format!("{:02X?}", (*self.data.read().unwrap()).get_ref());
        hex_raw_data.remove(0);
        hex_raw_data.pop();
        hex_raw_data.retain(|c| c != ',');

        if hex_raw_data.len() > 46 {
            (46..hex_raw_data.len() - 1).rev().filter(|x| x % 48 == 0).for_each(|x| hex_raw_data.replace_range(x - 1..x, "\n"));
        }

        let qhex_raw_data = QString::from_std_str(&hex_raw_data);
        let text_size = font_metrics.size_2a(0, &qhex_raw_data);
        self.hex_view_raw().set_text(&qhex_raw_data);
        self.hex_view_raw().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Decoded data section.
        //---------------------------------------------//

        // This pushes a newline after 16 characters.
        let mut hex_decoded_data = String::new();
        for (j, i) in self.data.read().unwrap().get_ref().iter().enumerate() {
            if j % 16 == 0 && j != 0 { hex_decoded_data.push('\n'); }
            let character = *i as char;

            // If is a valid UTF-8 char, show it. Otherwise, default to '.'.
            if character.is_alphanumeric() { hex_decoded_data.push(character); }
            else { hex_decoded_data.push('.'); }
        }

        // Add all the "Decoded" lines to the TextEdit.
        let qhex_decoded_data = QString::from_std_str(&hex_decoded_data);
        let text_size = font_metrics.size_2a(0, &qhex_decoded_data);
        self.hex_view_decoded().set_text(&qhex_decoded_data);
        self.hex_view_decoded().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Header Marking section.
        //---------------------------------------------//

        let use_dark_theme = setting_bool("use_dark_theme");
        let brush = QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkRed } else { GlobalColor::Red });
        let header_format = QTextCharFormat::new();
        header_format.set_background(&brush);

        // Block the signals during this, so we don't mess things up.
        let blocker = QSignalBlocker::from_q_object(self.hex_view_raw().static_upcast::<QObject>());
        let cursor = self.hex_view_raw().text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, (self.header_size * 3) as i32);
        self.hex_view_raw().set_text_cursor(&cursor);
        self.hex_view_raw().set_current_char_format(&header_format);
        cursor.clear_selection();
        self.hex_view_raw().set_text_cursor(&cursor);

        blocker.unblock();

        // Block the signals during this, so we don't mess things up.
        let blocker = QSignalBlocker::from_q_object(self.hex_view_decoded().static_upcast::<QObject>());
        let cursor = self.hex_view_decoded().text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, (self.header_size + (self.header_size as f32 / 16.0).floor() as u64) as i32);
        self.hex_view_decoded().set_text_cursor(&cursor);
        self.hex_view_decoded().set_current_char_format(&header_format);
        cursor.clear_selection();
        self.hex_view_decoded().set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Info section.
        //---------------------------------------------//

        // Load the "Info" data to the view.
        if self.version > 0 {
            self.packed_file_info_version_decoded_spinbox().set_enabled(false);
        } else {
            self.packed_file_info_version_decoded_spinbox().set_maximum(0);
            self.packed_file_info_version_decoded_spinbox().set_minimum(-99);
        }

        self.packed_file_info_version_decoded_spinbox().set_value(self.version);
        self.packed_file_info_entry_count_decoded_label().set_text(&QString::from_std_str(&self.entry_count.to_string()));

        Ok(())
    }

    /// This function syncronize the selection between the Hex View and the Decoded View of the PackedFile Data.
    /// Pass `hex = true` if the selected view is the Hex View. Otherwise, pass false.
    pub unsafe fn hex_selection_sync(&self, hex: bool) {

        let cursor = if hex { self.hex_view_raw.text_cursor() } else { self.hex_view_decoded.text_cursor() };
        let cursor_dest = if !hex { self.hex_view_raw.text_cursor() } else { self.hex_view_decoded.text_cursor() };

        let mut selection_start = cursor.selection_start();
        let mut selection_end = cursor.selection_end();

        // Translate the selection from one view to the other, doing some maths.
        if hex {
            selection_start = ((selection_start + 1) / 3) + (selection_start / 48);
            selection_end = ((selection_end + 2) / 3) + (selection_end / 48);
        }
        else {
            selection_start = (selection_start - (selection_start / 17)) * 3;
            selection_end = (selection_end - (selection_end / 17)) * 3;
        }

        // Fix for the situation where you select less than what in the decoded view will be one character, being the change:
        // 3 chars in raw = 1 in decoded.
        if hex && selection_start == selection_end && cursor.selection_start() != cursor.selection_end() {
            selection_end += 1;
        }

        cursor_dest.move_position_1a(MoveOperation::Start);
        cursor_dest.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, selection_start as i32);
        cursor_dest.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, (selection_end - selection_start) as i32);

        // Block the signals during this, so we don't trigger an infinite loop.
        if hex {
            let blocker = QSignalBlocker::from_q_object(&self.hex_view_decoded);
            self.hex_view_decoded.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
        else {
            let blocker = QSignalBlocker::from_q_object(&self.hex_view_raw);
            self.hex_view_raw.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
    }

    /// This function is used to update the state of the decoder view every time a change it's done.
    unsafe fn update_view(
        &self,
        field_list: &[Field],
        is_initial_load: bool,
    ) -> Result<()> {

        // If it's the first load, we have to prepare the table's column data.
        if is_initial_load {

            // If the table is empty, we just load a fake row, so the column headers are created properly.
            if field_list.is_empty() {
                let qlist = QListOfQStandardItem::new();
                (0..16).for_each(|_| qlist.append_q_standard_item(&QStandardItem::new().into_ptr().as_mut_raw_ptr()));
                self.table_model.append_row_q_list_of_q_standard_item(&qlist);
                configure_table_view(&self.table_view);
                self.table_model.remove_rows_2a(0, 1);
            }

            // Otherswise, we add each field we got as a row to the table.
            else {
                for field in field_list {
                    self.add_field_to_view(field, is_initial_load, None);
                }
                configure_table_view(&self.table_view);
            }
        }

        let mut data = self.data.write().unwrap();

        let decoded_bool = Self::decode_data_by_fieldtype(&mut *data, &FieldType::Boolean);
        let decoded_f32 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::F32);
        let decoded_f64 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::F64);
        let decoded_i16 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::I16);
        let decoded_i32 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::I32);
        let decoded_i64 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::I64);
        let decoded_optional_i16 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::OptionalI16);
        let decoded_optional_i32 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::OptionalI32);
        let decoded_optional_i64 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::OptionalI64);
        let decoded_colour_rgb = Self::decode_data_by_fieldtype(&mut *data, &FieldType::ColourRGB);
        let decoded_string_u8 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::StringU8);
        let decoded_string_u16 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::StringU16);
        let decoded_optional_string_u8 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::OptionalStringU8);
        let decoded_optional_string_u16 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::OptionalStringU16);
        let decoded_sequence_u32 = Self::decode_data_by_fieldtype(&mut *data, &FieldType::SequenceU32(Box::new(Definition::new(-100, None))));

        // We update all the decoded entries here.
        self.bool_line_edit.set_text(&QString::from_std_str(decoded_bool));
        self.f32_line_edit.set_text(&QString::from_std_str(decoded_f32));
        self.f64_line_edit.set_text(&QString::from_std_str(decoded_f64));
        self.i16_line_edit.set_text(&QString::from_std_str(decoded_i16));
        self.i32_line_edit.set_text(&QString::from_std_str(decoded_i32));
        self.i64_line_edit.set_text(&QString::from_std_str(decoded_i64));
        self.optional_i16_line_edit.set_text(&QString::from_std_str(decoded_optional_i16));
        self.optional_i32_line_edit.set_text(&QString::from_std_str(decoded_optional_i32));
        self.optional_i64_line_edit.set_text(&QString::from_std_str(decoded_optional_i64));
        self.colour_rgb_line_edit.set_text(&QString::from_std_str(decoded_colour_rgb));
        self.string_u8_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8)));
        self.string_u16_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16)));
        self.optional_string_u8_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8)));
        self.optional_string_u16_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16)));
        self.sequence_u32_line_edit.set_text(&QString::from_std_str(&format!("Sequence of {:?} entries.", decoded_sequence_u32)));

        //---------------------------------------------//
        // Raw data cleaning section.
        //---------------------------------------------//

        // Prepare to paint the changes in the hex data views.
        let use_dark_theme = setting_bool("use_dark_theme");
        let index_format = QTextCharFormat::new();
        let decoded_format = QTextCharFormat::new();
        let neutral_format = QTextCharFormat::new();
        index_format.set_background(&QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkMagenta } else { GlobalColor::Magenta }));
        decoded_format.set_background(&QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkYellow } else { GlobalColor::Yellow }));
        neutral_format.set_background(&QBrush::from_global_color(GlobalColor::Transparent));

        // Clean both TextEdits, so we can repaint all the changes on them.
        let blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast::<QObject>());
        let cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (self.header_size * 3) as i32);
        cursor.move_position_2a(MoveOperation::End, MoveMode::KeepAnchor);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&neutral_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast::<QObject>());
        let cursor = self.hex_view_decoded.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (self.header_size + (self.header_size as f32 / 16.0).floor() as u64) as i32);
        cursor.move_position_2a(MoveOperation::End, MoveMode::KeepAnchor);

        self.hex_view_decoded.set_text_cursor(&cursor);
        self.hex_view_decoded.set_current_char_format(&neutral_format);
        cursor.clear_selection();
        self.hex_view_decoded.set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Raw data painting decoded data section.
        //---------------------------------------------//

        let blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast::<QObject>());
        let cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (self.header_size * 3) as i32);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, ((data.position() - self.header_size) * 3) as i32);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&decoded_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast::<QObject>());
        let cursor = self.hex_view_decoded.text_cursor();

        // Create the "Selection" for the decoded row.
        let positions_to_move_end = data.position() / 16;
        let positions_to_move_start = self.header_size / 16;
        let positions_to_move_vertical = positions_to_move_end - positions_to_move_start;
        let positions_to_move_horizontal = data.position() - self.header_size;
        let positions_to_move = positions_to_move_horizontal + positions_to_move_vertical;

        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (self.header_size + (self.header_size as f32 / 16.0).floor() as u64) as i32);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, positions_to_move as i32);

        self.hex_view_decoded.set_text_cursor(&cursor);
        self.hex_view_decoded.set_current_char_format(&decoded_format);
        cursor.clear_selection();
        self.hex_view_decoded.set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Raw data painting current index section.
        //---------------------------------------------//

        let blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast::<QObject>());
        let cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, 3);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&index_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast::<QObject>());
        let cursor = self.hex_view_decoded.text_cursor();
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, 1);

        self.hex_view_decoded.set_text_cursor(&cursor);
        self.hex_view_decoded.set_current_char_format(&index_format);
        cursor.clear_selection();
        self.hex_view_decoded.set_text_cursor(&cursor);

        blocker.unblock();

        Ok(())
    }

    /// This function adds fields to the decoder's table, so we can do this without depending on the
    /// updates of the decoder's view.
    ///
    /// It returns the new index.
    pub unsafe fn add_field_to_view(
        &self,
        field: &Field,
        is_initial_load: bool,
        parent: Option<CppBox<QModelIndex>>,
    ) {

        // Decode the data from the field.
        let decoded_data = Self::decode_data_by_fieldtype(&mut *self.data.write().unwrap(), field.field_type());

        // Get the type of the data we are going to put into the Table.
        let field_type = field.field_type().to_string();

        // Create a new list of StandardItem.
        let qlist = QListOfQStandardItem::new();

        // Create the items of the new row.
        let field_name = QStandardItem::from_q_string(&QString::from_std_str(&field.name()));
        let field_type = QStandardItem::from_q_string(&QString::from_std_str(field_type));
        let field_is_key = QStandardItem::new();
        field_is_key.set_editable(false);
        field_is_key.set_checkable(true);
        field_is_key.set_check_state(if field.is_key(None) { CheckState::Checked } else { CheckState::Unchecked });

        let (field_reference_table, field_reference_field) = if let Some(ref reference) = field.is_reference() {
            (QStandardItem::from_q_string(&QString::from_std_str(&reference.0)), QStandardItem::from_q_string(&QString::from_std_str(&reference.1)))
        } else { (QStandardItem::new(), QStandardItem::new()) };

        let field_lookup_columns = if let Some(ref columns) = field.lookup() {
            QStandardItem::from_q_string(&QString::from_std_str(columns.join(",")))
        } else { QStandardItem::new() };

        let decoded_data = QStandardItem::from_q_string(&QString::from_std_str(&decoded_data));
        decoded_data.set_editable(false);

        let field_default_value = if let Some(ref default_value) = field.default_value(None) {
            QStandardItem::from_q_string(&QString::from_std_str(&default_value))
        } else { QStandardItem::new() };

        let field_is_filename = QStandardItem::new();
        field_is_filename.set_editable(false);
        field_is_filename.set_checkable(true);
        field_is_filename.set_check_state(if field.is_filename() { CheckState::Checked } else { CheckState::Unchecked });

        let field_filename_relative_path = if let Some(ref filename_relative_path) = field.filename_relative_path() {
            QStandardItem::from_q_string(&QString::from_std_str(&filename_relative_path))
        } else { QStandardItem::new() };

        let field_ca_order = QStandardItem::from_q_string(&QString::from_std_str(&format!("{}", field.ca_order())));
        let field_description = QStandardItem::from_q_string(&QString::from_std_str(field.description()));
        let field_enum_values = QStandardItem::from_q_string(&QString::from_std_str(field.enum_values_to_string()));

        let field_is_bitwise = QStandardItem::new();
        field_is_bitwise.set_data_2a(&QVariant::from_int(field.is_bitwise()), 2);

        let field_number = QStandardItem::from_q_string(&QString::from_std_str(&format!("{}", 1 + 1)));
        field_number.set_editable(false);

        let field_is_part_of_colour = QStandardItem::new();
        if let Some(ref is_part_of_colour) = field.is_part_of_colour() {
            field_is_part_of_colour.set_data_2a(&QVariant::from_uint(*is_part_of_colour as u32), 2);
        }

        // The first one is the row number, to be updated later.
        qlist.append_q_standard_item(&field_number.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_name.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_type.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&decoded_data.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_is_key.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_reference_table.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_reference_field.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_lookup_columns.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_default_value.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_is_filename.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_filename_relative_path.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_ca_order.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_description.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_is_bitwise.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_enum_values.into_ptr().as_mut_raw_ptr());
        qlist.append_q_standard_item(&field_is_part_of_colour.into_ptr().as_mut_raw_ptr());

        // If it's the initial load, insert them recursively.
        if is_initial_load {
            match parent {
                Some(ref parent) => self.table_model.item_from_index(parent).append_row_q_list_of_q_standard_item(&qlist),
                None => self.table_model.append_row_q_list_of_q_standard_item(&qlist),
            }
            if let FieldType::SequenceU32(table) = field.field_type() {

                // The new parent is either the last child of the current parent, or the last item in the tree.
                for field in table.fields() {
                    let parent = match parent {
                        Some(ref parent) => {
                            let item = self.table_model.item_from_index(parent);
                            let last_item = item.child_1a(item.row_count() - 1);
                            last_item.index()
                        },
                        None => {
                            let item = self.table_model.invisible_root_item();
                            let last_item = item.child_1a(item.row_count() - 1);
                            last_item.index()
                        }
                    };

                    self.add_field_to_view(field, is_initial_load, Some(parent));
                }
            }
        }

        // If it's not the initial load, autodetect the deepness level.
        else {
            let mut last_item = self.table_model.invisible_root_item();
            loop {
                if last_item.row_count() > 0 {
                    let last_child = last_item.child_1a(last_item.row_count() - 1);
                    let index = last_child.index().sibling_at_column(2);
                    if last_child.has_children() || self.table_model.item_from_index(&index).text().to_std_string() == "SequenceU32" {
                        last_item = last_child;
                    }
                    else {
                        break;
                    }
                }
                else {
                    break;
                }
            }

            last_item.append_row_q_list_of_q_standard_item(&qlist);

            // Always expand the new item.
            self.table_view.expand(last_item.index().as_ref());
        }
    }

    /// This function is the one that takes care of actually decoding the provided data based on the field type.
    fn decode_data_by_fieldtype<T: ReadBytes>(data: &mut T, field_type: &FieldType) -> String {
        match field_type {
            FieldType::Boolean => {
                match data.read_bool() {
                    Ok(result) => {
                        if result { "True".to_string() }
                        else { "False".to_string() }
                    }
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::F32 => {
                match data.read_f32() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::F64 => {
                match data.read_f64() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::I16 => {
                match data.read_i16() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::I32 => {
                match data.read_i32() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::I64 => {
                match data.read_i64() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalI16 => {
                match data.read_optional_i16() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalI32 => {
                match data.read_optional_i32() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalI64 => {
                match data.read_optional_i64() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::ColourRGB => {
                match data.read_string_colour_rgb() {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::StringU8 => {
                match data.read_sized_string_u8() {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::StringU16 => {
                match data.read_sized_string_u16() {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalStringU8 => {
                match data.read_optional_string_u8() {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalStringU16 => {
                match data.read_optional_string_u16() {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::SequenceU16(_) => {
                match data.read_i16() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::SequenceU32(_) => {
                match data.read_i32() {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
        }
    }

    /// This function updates the "First Row Decoded" column of the table, then forces an update of the rest of the view.
    ///
    /// To be triggered when the table changes.
    unsafe fn update_rows_decoded(
        &self,
        entries: Option<u32>,
        model_index: Option<CppBox<QModelIndex>>,
    ) -> Result<()> {

        // If it's the first cycle, reset the index.
        if model_index.is_none() {
            self.data.write().unwrap().seek(SeekFrom::Start(self.header_size))?;
        }

        // Loop through all the rows.
        let entries = if let Some(entries) = entries { entries } else { 1 };
        let row_count = if let Some(ref model_index) = model_index {
            self.table_model.item_from_index(model_index.as_ref()).row_count()
        } else { self.table_model.row_count_0a() };

        for entry in 0..entries {
            if row_count == 0 {
                break;
            }

            for row in 0..row_count {

                // Get the ModelIndex of the cell we want to update.
                let model_index = if let Some(ref model_index) = model_index {
                    self.table_model.item_from_index(model_index.as_ref()).child_1a(row).index()
                } else { self.table_model.index_2a(row, 0) };

                if model_index.is_valid() {

                    // Get the row's type.
                    let row_type = model_index.sibling_at_column(2);
                    let field_type = match &*row_type.data_1a(0).to_string().to_std_string() {
                        "Boolean" => FieldType::Boolean,
                        "F32" => FieldType::F32,
                        "F64" => FieldType::F64,
                        "I16" => FieldType::I16,
                        "I32" => FieldType::I32,
                        "I64" => FieldType::I64,
                        "OptionalI16" => FieldType::OptionalI16,
                        "OptionalI32" => FieldType::OptionalI32,
                        "OptionalI64" => FieldType::OptionalI64,
                        "ColourRGB" => FieldType::ColourRGB,
                        "StringU8" => FieldType::StringU8,
                        "StringU16" => FieldType::StringU16,
                        "OptionalStringU8" => FieldType::OptionalStringU8,
                        "OptionalStringU16" => FieldType::OptionalStringU16,
                        "SequenceU32" => FieldType::SequenceU32(Box::new(Definition::new(-100, None))),
                        _ => unimplemented!("{}", &*row_type.data_1a(0).to_string().to_std_string())
                    };

                    // Get the decoded data using it's type...
                    let decoded_data = Self::decode_data_by_fieldtype(&mut *self.data.write().unwrap(), &field_type);

                    // Get the items from the "Row Number" and "First Row Decoded" columns.
                    if entry == 0 {
                        let item = self.table_model.item_from_index(&model_index.sibling_at_column(3));
                        item.set_text(&QString::from_std_str(&decoded_data));

                        let item = self.table_model.item_from_index(&model_index.sibling_at_column(0));
                        item.set_text(&QString::from_std_str(&format!("{}", row + 1)));
                    }

                    // If it's a sequence,decode also it's internal first row, then move the index to skip the rest.
                    if let FieldType::SequenceU32(_) = field_type {
                        self.update_rows_decoded(Some(decoded_data.parse::<u32>()?), Some(model_index.sibling_at_column(0)))?;
                    }
                }
            }
        }

        // Update the entire decoder to use the new index.
        if model_index.is_none() {
            self.update_view(&[], false)?;
        }

        Ok(())
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    unsafe fn load_versions_list(&self) {
        self.table_model_old_versions.clear();
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            if let Some(definitions) = schema.definitions_by_table_name(&self.table_name) {
                definitions.iter().for_each(|definition| {
                    let item = QStandardItem::from_q_string(&QString::from_std_str(&definition.version().to_string()));
                    self.table_model_old_versions.append_row_q_standard_item(item.into_ptr());
                })
            }
        }

        self.table_model_old_versions.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Versions Decoded")));
        self.table_view_old_versions.horizontal_header().set_section_resize_mode_1a(ResizeMode::Stretch);
    }

    /// This function is used to update the decoder view when we try to add a new field to
    /// the definition with one of the "Use this" buttons.
    pub unsafe fn use_this(&self, field_type: FieldType) -> Result<()> {
        let mut field = Field::default();
        field.set_field_type(field_type);

        self.add_field_to_view(&field, false, None);
        self.update_view(&[], false)?;
        self.update_rows_decoded(None, None)
    }


    /// This function gets the data from the decoder's table and returns it, so we can save it to a Definition.
    pub unsafe fn get_fields_from_view(&self, model_index: Option<CppBox<QModelIndex>>) -> Vec<Field> {
        let mut fields = vec![];
        let row_count = if let Some(ref model_index) = model_index {
            self.table_model.item_from_index(model_index.as_ref()).row_count()
        } else { self.table_model.row_count_0a() };

        for row in 0..row_count {

            let model_index = if let Some(ref model_index) = model_index {
                self.table_model.item_from_index(model_index.as_ref()).child_1a(row).index()
            } else { self.table_model.index_2a(row, 0) };

            if model_index.is_valid() {

                // Get the data from each field of the row...
                let field_name = self.table_model.item_from_index(model_index.sibling_at_column(1).as_ref()).text().to_std_string();
                let field_type = self.table_model.item_from_index(model_index.sibling_at_column(2).as_ref()).text().to_std_string();
                let field_is_key = self.table_model.item_from_index(model_index.sibling_at_column(4).as_ref()).check_state() == CheckState::Checked;
                let ref_table = self.table_model.item_from_index(model_index.sibling_at_column(5).as_ref()).text().to_std_string();
                let ref_column = self.table_model.item_from_index(model_index.sibling_at_column(6).as_ref()).text().to_std_string();
                let field_lookup = self.table_model.item_from_index(model_index.sibling_at_column(7).as_ref()).text().to_std_string();
                let field_default_value = self.table_model.item_from_index(model_index.sibling_at_column(8).as_ref()).text().to_std_string();
                let field_is_filename = self.table_model.item_from_index(model_index.sibling_at_column(9).as_ref()).check_state() == CheckState::Checked;
                let field_filename_relative_path = self.table_model.item_from_index(model_index.sibling_at_column(10).as_ref()).text().to_std_string();
                let field_ca_order = self.table_model.item_from_index(model_index.sibling_at_column(11).as_ref()).text().to_std_string().parse::<i16>().unwrap();
                let field_description = self.table_model.item_from_index(model_index.sibling_at_column(12).as_ref()).text().to_std_string();
                let field_is_bitwise = self.table_model.item_from_index(model_index.sibling_at_column(13).as_ref()).text().to_std_string().parse::<i32>().unwrap();
                let field_is_part_of_colour = self.table_model.item_from_index(model_index.sibling_at_column(15).as_ref()).text().to_std_string().parse::<u8>().ok();

                let mut field_enum_values = BTreeMap::new();
                let enum_types = self.table_model.item_from_index(model_index.sibling_at_column(14).as_ref())
                    .text()
                    .to_std_string()
                    .split(';')
                    .map(|x| x.to_owned())
                    .collect::<Vec<String>>();

                for enum_type in &enum_types {
                    let enum_values = enum_type.split(',').collect::<Vec<&str>>();

                    if enum_values.len() == 2 {
                        if let Ok(enum_index) = enum_values[0].parse::<i32>() {
                            let enum_name = enum_values[1];
                            field_enum_values.insert(enum_index, enum_name.to_owned());
                        }
                    }
                }

                // Get the proper type of the field. If invalid, default to OptionalStringU16.
                let field_type = match &*field_type {
                    "Boolean" => FieldType::Boolean,
                    "F32" => FieldType::F32,
                    "F64" => FieldType::F64,
                    "I16" => FieldType::I16,
                    "I32" => FieldType::I32,
                    "I64" => FieldType::I64,
                    "OptionalI16" => FieldType::OptionalI16,
                    "OptionalI32" => FieldType::OptionalI32,
                    "OptionalI64" => FieldType::OptionalI64,
                    "ColourRGB" => FieldType::ColourRGB,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,
                    "SequenceU32" => FieldType::SequenceU32({
                        let mut definition = Definition::new(-100, None);
                        *definition.fields_mut() = self.get_fields_from_view(Some(model_index));
                        Box::new(definition)
                    }),
                    _ => unimplemented!()
                };

                let field_is_reference = if !ref_table.is_empty() && !ref_column.is_empty() {
                    Some((ref_table, ref_column))
                } else { None };

                let field_lookup = if !field_lookup.is_empty() {
                    Some(field_lookup.split(',').map(|x| x.to_owned()).collect::<Vec<String>>())
                } else { None };

                fields.push(
                    Field::new(
                        field_name,
                        field_type,
                        field_is_key,
                        if field_default_value.is_empty() { None } else { Some(field_default_value) },
                        field_is_filename,
                        if field_filename_relative_path.is_empty() { None } else { Some(field_filename_relative_path) },
                        field_is_reference,
                        field_lookup,
                        field_description,
                        field_ca_order,
                        field_is_bitwise,
                        field_enum_values,
                        field_is_part_of_colour
                    )
                );
            }
        }

        fields
    }

    /// This function adds the definition currently in the view to a temporal schema, and returns it.
    unsafe fn add_definition_to_schema(&self) -> Schema {
        let mut schema = SCHEMA.read().unwrap().clone().unwrap();
        let mut definition = Definition::new(self.version, None);
        *definition.fields_mut() = self.get_fields_from_view(None);
        schema.add_definition(&self.table_name, &definition);

        schema
    }

    /// This function generates a valid definition using the assembly kit as reference. To stop decoding manually.
    ///
    /// Known issues:
    /// - If the loc files hasn't been properly marked in the Assembly Kit, this fails.
    /// - Sometimes this returns some floats as valid when they're not due to precision differences.
    /// - Sometimes it duplicates some column names, if both columns are exactly equal.
    /// - To make this not consider anything as a valid integer, the integers are limited to an range of -60k+60k, around 0 and near their type limits.
    pub fn import_from_assembly_kit(&self) -> Result<Vec<Vec<Field>>> {

        // Get the raw data ready.
        let game = GAME_SELECTED.read().unwrap();
        let raw_db_version = game.raw_db_version();
        let raw_db_path = assembly_kit_path()?;

        let raw_definition_paths = get_raw_definition_paths(&raw_db_path, raw_db_version)?;
        let raw_definition = RawDefinition::read(raw_definition_paths.iter().find(|x| {
            format!("{}_tables", x.file_stem().unwrap().to_str().unwrap().split_at(5).1) == self.table_name
        }).unwrap(), raw_db_version).unwrap();

        let raw_table = RawTable::read(&raw_definition, &raw_db_path, raw_db_version)?;
        let imported_table = DB::try_from(&raw_table)?;

        let raw_localisable_fields: RawLocalisableFields = RawLocalisableFields::read(&raw_db_path, raw_db_version).map_err(|error| anyhow!("{}. This happens when the TExc_LocalisableFields.xml file is missing/empty/broken in raw_data/db. If it's so, copy it from another game and try again.", error))?;
        let mut raw_columns: Vec<Vec<String>> = vec![];

        let table_data = imported_table.data(&None)?;
        for row in table_data.iter() {
            for (index, field) in row.iter().enumerate() {
                match raw_columns.get_mut(index) {
                    Some(ref mut column) => column.push(field.data_to_string().to_string()),
                    None => raw_columns.push(vec![field.data_to_string().to_string()])
                }
            }
        }

        if table_data.is_empty() {
            return Err(anyhow!("This table is empty and there is not a Definition for it. That means it cannot be decoded."));
        }

        let expected_cells_bool = imported_table.definition().fields().iter().filter(|x| if let FieldType::Boolean = x.field_type() { true } else { false }).count();
        let expected_cells_f32 = imported_table.definition().fields().iter().filter(|x| if let FieldType::F32 = x.field_type() { true } else { false }).count();
        let expected_cells_f64 = imported_table.definition().fields().iter().filter(|x| if let FieldType::F64 = x.field_type() { true } else { false }).count();
        let expected_cells_i32 = imported_table.definition().fields().iter().filter(|x| if let FieldType::I32 = x.field_type() { true } else { false }).count();
        let expected_cells_i64 = imported_table.definition().fields().iter().filter(|x| if let FieldType::I64 = x.field_type() { true } else { false }).count();
        let expected_cells_colour_rgb = imported_table.definition().fields().iter().filter(|x| if let FieldType::ColourRGB = x.field_type() { true } else { false }).count();
        let expected_cells_string_u8 = imported_table.definition().fields().iter().filter(|x| if let FieldType::StringU8 = x.field_type() { true } else if let FieldType::OptionalStringU8 = x.field_type() { true } else { false }).count();

        let imported_first_row = &table_data[0];
        let mut data = self.data.write().unwrap();
        data.seek(SeekFrom::Start(self.header_size))?;

        // First check is done here, to initialize the possible schemas.
        let mut definitions_possible: Vec<Vec<FieldType>> = vec![];
        if definitions_possible.is_empty() {
            if data.read_f32().is_ok() {
                definitions_possible.push(vec![FieldType::F32]);
            }

            data.seek(SeekFrom::Start(self.header_size))?;
            if data.read_f64().is_ok() {
                definitions_possible.push(vec![FieldType::F64]);
            }

            data.seek(SeekFrom::Start(self.header_size))?;
            if data.read_i32().is_ok() {
                definitions_possible.push(vec![FieldType::I32]);
            }

            data.seek(SeekFrom::Start(self.header_size))?;
            if data.read_i64().is_ok() {
                definitions_possible.push(vec![FieldType::I64]);
            }

            data.seek(SeekFrom::Start(self.header_size))?;
            if data.read_string_colour_rgb().is_ok() { definitions_possible.push(vec![FieldType::ColourRGB]); }

            data.seek(SeekFrom::Start(self.header_size))?;
            if data.read_bool().is_ok() { definitions_possible.push(vec![FieldType::Boolean]); }

            data.seek(SeekFrom::Start(self.header_size))?;
            if let Ok(data) = data.read_sized_string_u8() {
                if imported_first_row.iter().any(|x| if let DecodedData::StringU8(value) = x { value == &data } else if let DecodedData::OptionalStringU8(value) = x { value == &data } else { false }) {
                    definitions_possible.push(vec![FieldType::StringU8]);
                }
            }

            data.seek(SeekFrom::Start(self.header_size))?;
            if let Ok(data) = data.read_optional_string_u8() {
                if imported_first_row.iter().any(|x| if let DecodedData::OptionalStringU8(value) = x { value == &data } else if let DecodedData::StringU8(value) = x { value == &data } else { false }) {
                    definitions_possible.push(vec![FieldType::OptionalStringU8]);
                }
            }

        }

        data.seek(SeekFrom::Start(self.header_size))?;

        return Err(anyhow!("TBF"));
        /*
        // All the other checks are done here.
        for step in 0..raw_definition.get_non_localisable_fields(&raw_localisable_fields.fields, &raw_table.rows[0]).len() - 1 {
            println!("Possible definitions for the step {}: {}.", step, definitions_possible.len());
            if definitions_possible.is_empty() {
                break;
            }

            else {
                definitions_possible = definitions_possible.par_iter().filter_map(|base| {
                    let mut values_position = Vec::with_capacity(base.len());
                    let mut elements = vec![];
                    let mut index = 0;
                    for field_type in base {
                        match field_type {
                            FieldType::Boolean => {
                                let value = data.read_bool().unwrap();
                                values_position.push(DecodedData::Boolean(value));
                            },
                            FieldType::F32 => {
                                let value = data.read_f32().unwrap();
                                values_position.push(DecodedData::F32(value));
                            },
                            FieldType::F64 => {
                                let value = data.read_f64().unwrap();
                                values_position.push(DecodedData::F64(value));
                            },
                            FieldType::I32 => {
                                let value = data.read_i32().unwrap();
                                values_position.push(DecodedData::I32(value));
                            },
                            FieldType::I64 => {
                                let value = data.read_i64().unwrap();
                                values_position.push(DecodedData::I64(value));
                            },
                            FieldType::ColourRGB => {
                                let value = data.read_string_colour_rgb().unwrap();
                                values_position.push(DecodedData::ColourRGB(value));
                            },
                            FieldType::StringU8 => {
                                let value = data.read_sized_string_u8().unwrap();
                                values_position.push(DecodedData::StringU8(value));
                            },
                            FieldType::OptionalStringU8 => {
                                let value = data.read_optional_string_u8().unwrap();
                                values_position.push(DecodedData::OptionalStringU8(value));
                            },
                            _ => unimplemented!()
                        }
                    }

                    if base.iter().filter(|x| if let FieldType::Boolean = x { true } else { false }).count() < expected_cells_bool {
                        if let Ok(data) = data.read_bool() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::Boolean(value) = x { value == &data } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::Boolean(value) = x { value == &data } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::Boolean);
                                elements.push(def);
                            }
                        }
                    }

                    if base.iter().filter(|x| if let FieldType::I32 = x { true } else { false }).count() < expected_cells_i32 {
                        if let Ok(number) = data.read_i32() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::I32(value) = x { value == &number } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::I32(value) = x { value == &number } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::I32);
                                elements.push(def);
                            }
                        }
                    }

                    if base.iter().filter(|x| if let FieldType::F32 = x { true } else { false }).count() < expected_cells_f32 {
                        if let Ok(number) = data.read_f32() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::F32(value) = x { float_eq::float_eq!(*value, number, abs <= 0.01) } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::F32(value) = x { float_eq::float_eq!(*value, number, abs <= 0.01) } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::F32);
                                elements.push(def);
                            }
                        }
                    }

                    if base.iter().filter(|x| if let FieldType::F64 = x { true } else { false }).count() < expected_cells_f64 {
                        if let Ok(number) = data.read_f64() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::F64(value) = x { float_eq::float_eq!(*value, number, abs <= 0.2) } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::F64(value) = x { float_eq::float_eq!(*value, number, abs <= 0.2) } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::F64);
                                elements.push(def);
                            }
                        }
                    }

                    if base.iter().filter(|x| if let FieldType::I64 = x { true } else { false }).count() < expected_cells_i64 {
                        if let Ok(number) = data.read_i64() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::I64(value) = x { value == &number } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::I64(value) = x { value == &number } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::I64);
                                elements.push(def);
                            }
                        }
                    }
                    if base.iter().filter(|x| if let FieldType::ColourRGB = x { true } else { false }).count() < expected_cells_colour_rgb {
                        if let Ok(data) = data.read_string_colour_rgb() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::ColourRGB(value) = x { value == &data } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::ColourRGB(value) = x { value == &data } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::ColourRGB);
                                elements.push(def);
                            }
                        }
                    }
                    if base.iter().filter(|x| if let FieldType::StringU8 = x { true } else { false }).count() < expected_cells_string_u8 {
                        if let Ok(data) = data.read_sized_string_u8() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::StringU8(value) = x { value == &data } else if let DecodedData::OptionalStringU8(value) = x { value == &data } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::StringU8(value) = x { value == &data } else if let DecodedData::OptionalStringU8(value) = x { value == &data } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::StringU8);
                                elements.push(def);
                            }
                        }
                    }
                    if base.iter().filter(|x| if let FieldType::OptionalStringU8 = x { true } else { false }).count() < expected_cells_string_u8 {
                        if let Ok(data) = data.read_optional_string_u8() {
                            let duplicate_values_count = values_position.iter().filter(|x| if let DecodedData::OptionalStringU8(value) = x { value == &data } else if let DecodedData::StringU8(value) = x { value == &data } else { false }).count();
                            let duplicate_values_count_expected = imported_first_row.iter().filter(|x| if let DecodedData::OptionalStringU8(value) = x { value == &data } else if let DecodedData::StringU8(value) = x { value == &data } else { false }).count();
                            if duplicate_values_count < duplicate_values_count_expected {
                                let mut def = base.to_vec();
                                def.push(FieldType::OptionalStringU8);
                                elements.push(def);
                            }
                        }
                    }

                    if elements.is_empty() {
                        None
                    } else {
                        Some(elements)
                    }
                }).flatten().collect::<Vec<Vec<FieldType>>>();
            }
        }

        // Now, match all possible definitions against the table, and for the ones that work, match them against the asskit data.
        Ok(definitions_possible.par_iter().filter_map(|x| {
            let field_list = x.iter().map(|x| {
                let mut field = Field::default();
                field.set_field_type(x.clone());
                field
            }).collect::<Vec<Field>>();
            let definition = Definition::new_with_fields(self.version, &field_list, &[]);
            let table = DB::new(&definition, None, &self.table_name, false);

            if let Ok(table) = table.set_data(None, ) {
                if !table.get_ref_table_data().is_empty() {
                    let mut mapper: BTreeMap<usize, usize> = BTreeMap::new();
                    let mut decoded_columns: Vec<Vec<String>> = vec![];
                    let fields_processed = table.definition().fields_processed();

                    // Organized in columns, not in rows, so we can match by columns.
                    if let Ok(data) = table.data() {
                        for row in data.iter() {
                            for (index, field) in row.iter().enumerate() {
                                match decoded_columns.get_mut(index) {
                                    Some(ref mut column) => column.push(field.data_to_string()),
                                    None => decoded_columns.push(vec![field.data_to_string()])
                                }
                            }
                        }

                        let mut already_matched_columns = vec![];
                        for (index, column) in decoded_columns.iter().enumerate() {
                            match raw_columns.iter().enumerate().position(|(pos, x)| !already_matched_columns.contains(&pos) && x == column) {
                                Some(raw_column) => {
                                    mapper.insert(index, raw_column);
                                    already_matched_columns.push(raw_column);
                                },

                                // If no equivalent has been found, drop the definition.
                                None => return None,
                            }
                        }

                        // Filter the mapped data to see if we have a common one in every cell.
                        let fields = mapper.iter().map(|(x, y)| {
                            let mut field: Field = From::from(raw_definition.fields.get(*y).unwrap());
                            field.set_field_type(fields_processed[*x].get_field_type());
                            field
                        }).collect();

                        return Some(fields);
                    }
                }
            }
            None
        }).collect::<Vec<Vec<Field>>>())*/
    }

    /// This function returns the definition corresponding to the decoded Packedfile, if exists.
    fn definition(&self) -> Option<Definition> {
        if let Some(ref schema) = *SCHEMA.read().unwrap() {
            return schema.definition_by_name_and_version(&self.table_name, self.version).cloned();
        }
        None
    }
}

/// This function configures the provided TableView, so it has the right columns and it's resized to the right size.
unsafe fn configure_table_view(table_view: &QBox<QTreeView>) {
    let table_model = table_view.model();
    table_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Row Number")));
    table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Field Name")));
    table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Field Type")));
    table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("First Row Decoded")));
    table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Is key?")));
    table_model.set_header_data_3a(5, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Ref. to Table")));
    table_model.set_header_data_3a(6, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Ref. to Column")));
    table_model.set_header_data_3a(7, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Lookup Columns")));
    table_model.set_header_data_3a(8, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Default Value")));
    table_model.set_header_data_3a(9, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Is Filename")));
    table_model.set_header_data_3a(10, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Filename Relative Path")));
    table_model.set_header_data_3a(11, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("CA Order")));
    table_model.set_header_data_3a(12, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Description")));
    table_model.set_header_data_3a(13, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Bitwise Fields")));
    table_model.set_header_data_3a(14, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Enum Data")));
    table_model.set_header_data_3a(15, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Is Part of Colour")));
    table_view.header().set_stretch_last_section(true);
    table_view.header().resize_sections(ResizeMode::ResizeToContents);

    // The second field should be a combobox.
    let list = QStringList::new();
    list.append_q_string(&QString::from_std_str("Boolean"));
    list.append_q_string(&QString::from_std_str("F32"));
    list.append_q_string(&QString::from_std_str("F64"));
    list.append_q_string(&QString::from_std_str("I16"));
    list.append_q_string(&QString::from_std_str("I32"));
    list.append_q_string(&QString::from_std_str("I64"));
    list.append_q_string(&QString::from_std_str("OptionalI16"));
    list.append_q_string(&QString::from_std_str("OptionalI32"));
    list.append_q_string(&QString::from_std_str("OptionalI64"));
    list.append_q_string(&QString::from_std_str("ColourRGB"));
    list.append_q_string(&QString::from_std_str("StringU8"));
    list.append_q_string(&QString::from_std_str("StringU16"));
    list.append_q_string(&QString::from_std_str("OptionalStringU8"));
    list.append_q_string(&QString::from_std_str("OptionalStringU16"));
    list.append_q_string(&QString::from_std_str("SequenceU32"));
    new_combobox_item_delegate_safe(&table_view.static_upcast::<QObject>().as_ptr(), 2, list.as_ptr(), false, &QTimer::new_0a().into_ptr(), false);

    // Fields that need special code.
    new_spinbox_item_delegate_safe(&table_view.static_upcast::<QObject>().as_ptr(), 11, 16, &QTimer::new_0a().into_ptr(), false);
    new_qstring_item_delegate_safe(&table_view.static_upcast::<QObject>().as_ptr(), 14, &QTimer::new_0a().into_ptr(), false);
    new_spinbox_item_delegate_safe(&table_view.static_upcast::<QObject>().as_ptr(), 15, 32, &QTimer::new_0a().into_ptr(), false);
}
