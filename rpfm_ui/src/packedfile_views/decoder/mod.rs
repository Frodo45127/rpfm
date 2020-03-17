//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the PackedFile decoder.
!*/

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
use qt_widgets::QPushButton;
use qt_widgets::QTextEdit;

use qt_gui::QBrush;
use qt_gui::QFontMetrics;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;
use qt_gui::QTextCharFormat;
use qt_gui::q_text_cursor::{MoveOperation, MoveMode};


use qt_core::ContextMenuPolicy;
use qt_core::GlobalColor;
use qt_core::QSignalBlocker;
use qt_core::QString;
use qt_core::SortOrder;
use qt_core::QFlags;
use qt_core::QVariant;
use qt_core::Orientation;
use qt_core::QObject;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, atomic::AtomicPtr};

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::{loc, loc::Loc};
use rpfm_lib::schema::Schema;
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::FONT_MONOSPACE;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::utils::atomic_from_mut_ptr;
use crate::utils::create_grid_layout;
use crate::utils::ref_from_atomic;
use crate::utils::mut_ptr_from_atomic;
use self::slots::PackedFileDecoderViewSlots;

pub mod connections;
pub mod shortcuts;
pub mod slots;

/// List of supported PackedFile Types by the decoder.
const SUPPORTED_PACKED_FILE_TYPES: [PackedFileType; 2] = [
    PackedFileType::DB,
    PackedFileType::Loc,
];

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackedFile Decoder.
pub struct PackedFileDecoderView {
    hex_view_index: AtomicPtr<QTextEdit>,
    hex_view_raw: AtomicPtr<QTextEdit>,
    hex_view_decoded: AtomicPtr<QTextEdit>,

    table_view: AtomicPtr<QTableView>,
    table_model: AtomicPtr<QStandardItemModel>,

    table_view_context_menu: AtomicPtr<QMenu>,
    table_view_context_menu_move_up: AtomicPtr<QAction>,
    table_view_context_menu_move_down: AtomicPtr<QAction>,
    table_view_context_menu_delete: AtomicPtr<QAction>,

    bool_line_edit: AtomicPtr<QLineEdit>,
    float_line_edit: AtomicPtr<QLineEdit>,
    integer_line_edit: AtomicPtr<QLineEdit>,
    long_integer_line_edit: AtomicPtr<QLineEdit>,
    string_u8_line_edit: AtomicPtr<QLineEdit>,
    string_u16_line_edit: AtomicPtr<QLineEdit>,
    optional_string_u8_line_edit: AtomicPtr<QLineEdit>,
    optional_string_u16_line_edit: AtomicPtr<QLineEdit>,

    bool_button: AtomicPtr<QPushButton>,
    float_button: AtomicPtr<QPushButton>,
    integer_button: AtomicPtr<QPushButton>,
    long_integer_button: AtomicPtr<QPushButton>,
    string_u8_button: AtomicPtr<QPushButton>,
    string_u16_button: AtomicPtr<QPushButton>,
    optional_string_u8_button: AtomicPtr<QPushButton>,
    optional_string_u16_button: AtomicPtr<QPushButton>,

    packed_file_info_version_decoded_label: AtomicPtr<QLabel>,
    packed_file_info_entry_count_decoded_label: AtomicPtr<QLabel>,

    table_view_old_versions: AtomicPtr<QTableView>,
    table_model_old_versions: AtomicPtr<QStandardItemModel>,

    table_view_old_versions_context_menu: AtomicPtr<QMenu>,
    table_view_old_versions_context_menu_load: AtomicPtr<QAction>,
    table_view_old_versions_context_menu_delete: AtomicPtr<QAction>,

    test_definition_button: AtomicPtr<QPushButton>,
    generate_pretty_diff_button: AtomicPtr<QPushButton>,
    clear_definition_button: AtomicPtr<QPushButton>,
    save_button: AtomicPtr<QPushButton>,

    packed_file_type: PackedFileType,
    packed_file_path: Vec<String>,
    packed_file_data: Arc<Vec<u8>>,
}

/// This struct contains the raw version of each pointer in `PackedFileDecoderViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileDecoderView`.
#[derive(Clone)]
pub struct PackedFileDecoderViewRaw {
    pub hex_view_index: MutPtr<QTextEdit>,
    pub hex_view_raw: MutPtr<QTextEdit>,
    pub hex_view_decoded: MutPtr<QTextEdit>,

    pub table_view: MutPtr<QTableView>,
    pub table_model: MutPtr<QStandardItemModel>,

    pub table_view_context_menu: MutPtr<QMenu>,
    pub table_view_context_menu_move_up: MutPtr<QAction>,
    pub table_view_context_menu_move_down: MutPtr<QAction>,
    pub table_view_context_menu_delete: MutPtr<QAction>,

    pub bool_line_edit: MutPtr<QLineEdit>,
    pub float_line_edit: MutPtr<QLineEdit>,
    pub integer_line_edit: MutPtr<QLineEdit>,
    pub long_integer_line_edit: MutPtr<QLineEdit>,
    pub string_u8_line_edit: MutPtr<QLineEdit>,
    pub string_u16_line_edit: MutPtr<QLineEdit>,
    pub optional_string_u8_line_edit: MutPtr<QLineEdit>,
    pub optional_string_u16_line_edit: MutPtr<QLineEdit>,

    pub bool_button: MutPtr<QPushButton>,
    pub float_button: MutPtr<QPushButton>,
    pub integer_button: MutPtr<QPushButton>,
    pub long_integer_button: MutPtr<QPushButton>,
    pub string_u8_button: MutPtr<QPushButton>,
    pub string_u16_button: MutPtr<QPushButton>,
    pub optional_string_u8_button: MutPtr<QPushButton>,
    pub optional_string_u16_button: MutPtr<QPushButton>,

    pub packed_file_info_version_decoded_label: MutPtr<QLabel>,
    pub packed_file_info_entry_count_decoded_label: MutPtr<QLabel>,

    pub table_view_old_versions: MutPtr<QTableView>,
    pub table_model_old_versions: MutPtr<QStandardItemModel>,

    pub table_view_old_versions_context_menu: MutPtr<QMenu>,
    pub table_view_old_versions_context_menu_load: MutPtr<QAction>,
    pub table_view_old_versions_context_menu_delete: MutPtr<QAction>,

    pub test_definition_button: MutPtr<QPushButton>,
    pub generate_pretty_diff_button: MutPtr<QPushButton>,
    pub clear_definition_button: MutPtr<QPushButton>,
    pub save_button: MutPtr<QPushButton>,

    pub packed_file_type: PackedFileType,
    pub packed_file_path: Vec<String>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileDecoderView`.
impl PackedFileDecoderView {

    /// This function creates a new Decoder View, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_path: &Rc<RefCell<Vec<String>>>,
        packed_file_view: &mut PackedFileView,
        global_search_ui: &GlobalSearchUI,
        pack_file_contents_ui: &PackFileContentsUI,
    ) -> Result<TheOneSlot> {

        // Get the decoded Text.
        CENTRAL_COMMAND.send_message_qt(Command::GetPackedFile(packed_file_path.borrow().to_vec()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let packed_file = match response {
            Response::OptionPackedFile(packed_file) => match packed_file {
                Some(packed_file) => packed_file,
                None => return Err(ErrorKind::PackedFileNotFound.into()),
            }
            Response::Error(error) => return Err(error),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let packed_file_type = PackedFileType::get_packed_file_type_by_data(&packed_file);

        // If the PackedFileType is not one of the ones supported by the schema system, get out.
        if !SUPPORTED_PACKED_FILE_TYPES.iter().any(|x| x == &packed_file_type)  {
            return Err(ErrorKind::PackedFileNotDecodeableWithDecoder.into());
        }

        // Create the hex view on the left side.
        let mut layout: MutPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast_mut();

        //---------------------------------------------//
        // Hex Data section.
        //---------------------------------------------//

        let hex_view_group = QGroupBox::from_q_string(&QString::from_std_str("PackedFile's Data")).into_ptr();
        let mut hex_view_index = QTextEdit::new();
        let mut hex_view_raw = QTextEdit::new();
        let mut hex_view_decoded = QTextEdit::new();
        let mut hex_view_layout = create_grid_layout(hex_view_group.static_upcast_mut());

        hex_view_index.set_font(ref_from_atomic(&*FONT_MONOSPACE));
        hex_view_raw.set_font(ref_from_atomic(&*FONT_MONOSPACE));
        hex_view_decoded.set_font(ref_from_atomic(&*FONT_MONOSPACE));

        hex_view_layout.add_widget_5a(&mut hex_view_index, 0, 0, 1, 1);
        hex_view_layout.add_widget_5a(&mut hex_view_raw, 0, 1, 1, 1);
        hex_view_layout.add_widget_5a(&mut hex_view_decoded, 0, 2, 1, 1);

        layout.add_widget_5a(hex_view_group, 0, 0, 5, 1);

        //---------------------------------------------//
        // Fields Table section.
        //---------------------------------------------//

        let mut table_view = QTableView::new_0a();
        let mut table_model = QStandardItemModel::new_0a();
        table_view.set_model(table_model.as_mut_ptr());
        table_view.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);
        table_view.horizontal_header().set_stretch_last_section(true);
        table_view.set_alternating_row_colors(true);

        // Create the Contextual Menu for the TableView.
        let mut table_view_context_menu = QMenu::new();

        // Create the Contextual Menu Actions.
        let mut table_view_context_menu_move_up = table_view_context_menu.add_action_q_string(&QString::from_std_str("Move &Up"));
        let mut table_view_context_menu_move_down = table_view_context_menu.add_action_q_string(&QString::from_std_str("&Move Down"));
        let mut table_view_context_menu_delete = table_view_context_menu.add_action_q_string(&QString::from_std_str("&Delete"));

        // Disable them by default.
        table_view_context_menu_move_up.set_enabled(false);
        table_view_context_menu_move_down.set_enabled(false);
        table_view_context_menu_delete.set_enabled(false);

        layout.add_widget_5a(table_view.as_mut_ptr(), 0, 1, 1, 2);

        //---------------------------------------------//
        // Decoded Fields section.
        //---------------------------------------------//

        let mut decoded_fields_frame = QGroupBox::from_q_string(&QString::from_std_str("Current Field Decoded"));
        let mut decoded_fields_layout = create_grid_layout(decoded_fields_frame.as_mut_ptr().static_upcast_mut());
        decoded_fields_layout.set_column_stretch(1, 10);

        // Create the stuff for the decoded fields.
        let bool_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Bool\":"));
        let float_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Float\":"));
        let integer_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Integer\":"));
        let long_integer_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Long Integer\":"));
        let string_u8_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"String U8\":"));
        let string_u16_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"String U16\":"));
        let optional_string_u8_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Optional String U8\":"));
        let optional_string_u16_label = QLabel::from_q_string(&QString::from_std_str("Decoded as \"Optional String U16\":"));

        let mut bool_line_edit = QLineEdit::new();
        let mut float_line_edit = QLineEdit::new();
        let mut integer_line_edit = QLineEdit::new();
        let mut long_integer_line_edit = QLineEdit::new();
        let mut string_u8_line_edit = QLineEdit::new();
        let mut string_u16_line_edit = QLineEdit::new();
        let mut optional_string_u8_line_edit = QLineEdit::new();
        let mut optional_string_u16_line_edit = QLineEdit::new();

        let mut bool_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut float_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut integer_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut long_integer_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut string_u8_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut string_u16_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut optional_string_u8_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));
        let mut optional_string_u16_button = QPushButton::from_q_string(&QString::from_std_str("Use this"));

        decoded_fields_layout.add_widget_5a(bool_label.into_ptr(), 0, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(float_label.into_ptr(), 1, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(integer_label.into_ptr(), 2, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(long_integer_label.into_ptr(), 3, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(string_u8_label.into_ptr(), 4, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(string_u16_label.into_ptr(), 5, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(optional_string_u8_label.into_ptr(), 6, 0, 1, 1);
        decoded_fields_layout.add_widget_5a(optional_string_u16_label.into_ptr(), 7, 0, 1, 1);

        decoded_fields_layout.add_widget_5a(&mut bool_line_edit, 0, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut float_line_edit, 1, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut integer_line_edit, 2, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut long_integer_line_edit, 3, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut string_u8_line_edit, 4, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut string_u16_line_edit, 5, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut optional_string_u8_line_edit, 6, 1, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut optional_string_u16_line_edit, 7, 1, 1, 1);

        decoded_fields_layout.add_widget_5a(&mut bool_button, 0, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut float_button, 1, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut integer_button, 2, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut long_integer_button, 3, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut string_u8_button, 4, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut string_u16_button, 5, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut optional_string_u8_button, 6, 2, 1, 1);
        decoded_fields_layout.add_widget_5a(&mut optional_string_u16_button, 7, 2, 1, 1);

        layout.add_widget_5a(decoded_fields_frame.into_ptr(), 1, 1, 3, 1);

        //---------------------------------------------//
        // Info section.
        //---------------------------------------------//

        let mut info_frame = QGroupBox::from_q_string(&QString::from_std_str("PackedFile Info"));
        let mut info_layout = create_grid_layout(info_frame.as_mut_ptr().static_upcast_mut());

        // Create stuff for the info frame.
        let packed_file_info_type_label = QLabel::from_q_string(&QString::from_std_str("PackedFile Type:"));
        let packed_file_info_version_label = QLabel::from_q_string(&QString::from_std_str("PackedFile version:"));
        let packed_file_info_entry_count_label = QLabel::from_q_string(&QString::from_std_str("PackedFile entry count:"));

        let packed_file_info_type_decoded_label = QLabel::from_q_string(&QString::from_std_str(match packed_file_type {
            PackedFileType::DB => format!("DB/{}", packed_file_path.borrow()[1]),
            _ => format!("{}", packed_file_type),
        }));
        let mut packed_file_info_version_decoded_label = QLabel::new();
        let mut packed_file_info_entry_count_decoded_label = QLabel::new();

        info_layout.add_widget_5a(packed_file_info_type_label.into_ptr(), 0, 0, 1, 1);
        info_layout.add_widget_5a(packed_file_info_version_label.into_ptr(), 1, 0, 1, 1);

        info_layout.add_widget_5a(packed_file_info_type_decoded_label.into_ptr(), 0, 1, 1, 1);
        info_layout.add_widget_5a(&mut packed_file_info_version_decoded_label, 1, 1, 1, 1);

        match packed_file_type {
            PackedFileType::DB | PackedFileType::Loc => {
                info_layout.add_widget_5a(packed_file_info_entry_count_label.into_ptr(), 2, 0, 1, 1);
                info_layout.add_widget_5a(&mut packed_file_info_entry_count_decoded_label, 2, 1, 1, 1);
            }
            _ => unimplemented!(),
        }

        layout.add_widget_5a(info_frame.into_ptr(), 1, 2, 1, 1);

        //---------------------------------------------//
        // Other Versions section.
        //---------------------------------------------//

        let mut table_view_old_versions = QTableView::new_0a();
        let mut table_model_old_versions = QStandardItemModel::new_0a();
        table_view_old_versions.set_model(&mut table_model_old_versions);
        table_view_old_versions.set_alternating_row_colors(true);
        table_view_old_versions.set_edit_triggers(QFlags::from(EditTrigger::NoEditTriggers));
        table_view_old_versions.set_selection_mode(SelectionMode::SingleSelection);
        table_view_old_versions.set_sorting_enabled(true);
        table_view_old_versions.sort_by_column_2a(0, SortOrder::AscendingOrder);
        table_view_old_versions.vertical_header().set_visible(false);
        table_view_old_versions.set_context_menu_policy(ContextMenuPolicy::CustomContextMenu);

        let mut table_view_old_versions_context_menu = QMenu::new();
        let mut table_view_old_versions_context_menu_load = table_view_old_versions_context_menu.add_action_q_string(&QString::from_std_str("&Load"));
        let mut table_view_old_versions_context_menu_delete = table_view_old_versions_context_menu.add_action_q_string(&QString::from_std_str("&Delete"));
        table_view_old_versions_context_menu_load.set_enabled(false);
        table_view_old_versions_context_menu_delete.set_enabled(false);

        layout.add_widget_5a(&mut table_view_old_versions, 2, 2, 1, 1);

        //---------------------------------------------//
        // Buttons section.
        //---------------------------------------------//

        let mut button_box = QFrame::new_0a();
        let mut button_box_layout = create_grid_layout(button_box.as_mut_ptr().static_upcast_mut());

        // Create the bottom Buttons.
        let mut test_definition_button = QPushButton::from_q_string(&QString::from_std_str("Test Definition"));
        let mut generate_pretty_diff_button = QPushButton::from_q_string(&QString::from_std_str("Generate Diff"));
        let mut clear_definition_button = QPushButton::from_q_string(&QString::from_std_str("Remove all fields"));
        let mut save_button = QPushButton::from_q_string(&QString::from_std_str("Finish it!"));

        // Add them to the Dialog.
        button_box_layout.add_widget_5a(&mut test_definition_button, 0, 0, 1, 1);
        button_box_layout.add_widget_5a(&mut generate_pretty_diff_button, 0, 1, 1, 1);
        button_box_layout.add_widget_5a(&mut clear_definition_button, 0, 2, 1, 1);
        button_box_layout.add_widget_5a(&mut save_button, 0, 3, 1, 1);

        layout.add_widget_5a(button_box.into_ptr(), 4, 1, 1, 2);

        layout.set_column_stretch(1, 10);
        layout.set_row_stretch(0, 10);
        layout.set_row_stretch(2, 5);

        let mut packed_file_decoder_view_raw = PackedFileDecoderViewRaw {
            hex_view_index: hex_view_index.into_ptr(),
            hex_view_raw: hex_view_raw.into_ptr(),
            hex_view_decoded: hex_view_decoded.into_ptr(),

            table_view: table_view.into_ptr(),
            table_model: table_model.into_ptr(),

            table_view_context_menu: table_view_context_menu.into_ptr(),
            table_view_context_menu_move_up,
            table_view_context_menu_move_down,
            table_view_context_menu_delete,

            bool_line_edit: bool_line_edit.into_ptr(),
            float_line_edit: float_line_edit.into_ptr(),
            integer_line_edit: integer_line_edit.into_ptr(),
            long_integer_line_edit: long_integer_line_edit.into_ptr(),
            string_u8_line_edit: string_u8_line_edit.into_ptr(),
            string_u16_line_edit: string_u16_line_edit.into_ptr(),
            optional_string_u8_line_edit: optional_string_u8_line_edit.into_ptr(),
            optional_string_u16_line_edit: optional_string_u16_line_edit.into_ptr(),

            bool_button: bool_button.into_ptr(),
            float_button: float_button.into_ptr(),
            integer_button: integer_button.into_ptr(),
            long_integer_button: long_integer_button.into_ptr(),
            string_u8_button: string_u8_button.into_ptr(),
            string_u16_button: string_u16_button.into_ptr(),
            optional_string_u8_button: optional_string_u8_button.into_ptr(),
            optional_string_u16_button: optional_string_u16_button.into_ptr(),

            packed_file_info_version_decoded_label: packed_file_info_version_decoded_label.into_ptr(),
            packed_file_info_entry_count_decoded_label: packed_file_info_entry_count_decoded_label.into_ptr(),

            table_view_old_versions: table_view_old_versions.into_ptr(),
            table_model_old_versions: table_model_old_versions.into_ptr(),

            table_view_old_versions_context_menu: table_view_old_versions_context_menu.into_ptr(),
            table_view_old_versions_context_menu_load,
            table_view_old_versions_context_menu_delete,

            test_definition_button: test_definition_button.into_ptr(),
            generate_pretty_diff_button: generate_pretty_diff_button.into_ptr(),
            clear_definition_button: clear_definition_button.into_ptr(),
            save_button: save_button.into_ptr(),

            packed_file_path: packed_file.get_path().to_vec(),
            packed_file_type,
        };

        let packed_file_decoder_view_slots = PackedFileDecoderViewSlots::new(
            packed_file_decoder_view_raw.clone(),
            *pack_file_contents_ui,
            *global_search_ui,
            &packed_file_path
        );

        let mut packed_file_decoder_view = Self {
            hex_view_index: atomic_from_mut_ptr(packed_file_decoder_view_raw.hex_view_index),
            hex_view_raw: atomic_from_mut_ptr(packed_file_decoder_view_raw.hex_view_raw),
            hex_view_decoded: atomic_from_mut_ptr(packed_file_decoder_view_raw.hex_view_decoded),

            table_view: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view),
            table_model: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_model),

            table_view_context_menu: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_context_menu),
            table_view_context_menu_move_up: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_context_menu_move_up),
            table_view_context_menu_move_down: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_context_menu_move_down),
            table_view_context_menu_delete: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_context_menu_delete),

            bool_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.bool_line_edit),
            float_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.float_line_edit),
            integer_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.integer_line_edit),
            long_integer_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.long_integer_line_edit),
            string_u8_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.string_u8_line_edit),
            string_u16_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.string_u16_line_edit),
            optional_string_u8_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.optional_string_u8_line_edit),
            optional_string_u16_line_edit: atomic_from_mut_ptr(packed_file_decoder_view_raw.optional_string_u16_line_edit),

            bool_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.bool_button),
            float_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.float_button),
            integer_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.integer_button),
            long_integer_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.long_integer_button),
            string_u8_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.string_u8_button),
            string_u16_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.string_u16_button),
            optional_string_u8_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.optional_string_u8_button),
            optional_string_u16_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.optional_string_u16_button),

            packed_file_info_version_decoded_label: atomic_from_mut_ptr(packed_file_decoder_view_raw.packed_file_info_version_decoded_label),
            packed_file_info_entry_count_decoded_label: atomic_from_mut_ptr(packed_file_decoder_view_raw.packed_file_info_entry_count_decoded_label),

            table_view_old_versions: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_old_versions),
            table_model_old_versions: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_model_old_versions),

            table_view_old_versions_context_menu: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_old_versions_context_menu),
            table_view_old_versions_context_menu_load: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_old_versions_context_menu_load),
            table_view_old_versions_context_menu_delete: atomic_from_mut_ptr(packed_file_decoder_view_raw.table_view_old_versions_context_menu_delete),

            test_definition_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.test_definition_button),
            generate_pretty_diff_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.generate_pretty_diff_button),
            clear_definition_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.clear_definition_button),
            save_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.save_button),

            packed_file_type,
            packed_file_path: packed_file.get_path().to_vec(),
            packed_file_data: Arc::new(packed_file.get_raw_data()?),
        };

        packed_file_decoder_view.load_packed_file_data()?;
        packed_file_decoder_view_raw.load_versions_list();
        connections::set_connections(&packed_file_decoder_view, &packed_file_decoder_view_slots);
        shortcuts::set_shortcuts(&mut packed_file_decoder_view);
        packed_file_view.view = View::Decoder(packed_file_decoder_view);

        // Return success.
        Ok(TheOneSlot::Decoder(packed_file_decoder_view_slots))
    }

    /// This function loads the raw data of a PackedFile into the UI and prepare it to be updated later on.
    pub unsafe fn load_packed_file_data(&self) -> Result<()> {

        // We need to set up the fonts in a specific way, so the scroll/sizes are kept correct.
        let font = self.get_mut_ptr_hex_view_index().document().default_font();
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
        let hex_lines = (self.packed_file_data.len() / 16) + 1;
        (0..hex_lines).for_each(|x| hex_index.push_str(&format!("{:>0count$X}\n", x * 16, count = 4)));

        let qhex_index = QString::from_std_str(&hex_index);
        let text_size = font_metrics.size_2a(0, &qhex_index);
        self.get_mut_ptr_hex_view_index().set_text(&qhex_index);
        self.get_mut_ptr_hex_view_index().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Raw data section.
        //---------------------------------------------//
        //
        // Prepare the Hex Raw Data string, looking like:
        // 01 0a 02 0f 0d 02 04 06 01 0a 02 0f 0d 02 04 06
        let mut hex_raw_data = format!("{:02X?}", self.packed_file_data);
        hex_raw_data.remove(0);
        hex_raw_data.pop();
        hex_raw_data.retain(|c| c != ',');

        // Note: this works on BYTES, NOT CHARACTERS. Which means some characters may use multiple bytes,
        // and if you pass these functions a range thats not a character, they panic!
        // For reference, everything is one byte except the thin whitespace that's three bytes.
        (2..hex_raw_data.len() - 1).rev().step_by(3).filter(|x| x % 4 != 0).for_each(|x| hex_raw_data.replace_range(x - 1..x, " "));
        if hex_raw_data.len() > 70 {
            (70..hex_raw_data.len() - 1).rev().filter(|x| x % 72 == 0).for_each(|x| hex_raw_data.replace_range(x - 1..x, "\n"));
        }

        let qhex_raw_data = QString::from_std_str(&hex_raw_data);
        let text_size = font_metrics.size_2a(0, &qhex_raw_data);
        self.get_mut_ptr_hex_view_raw().set_text(&qhex_raw_data);
        self.get_mut_ptr_hex_view_raw().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Decoded data section.
        //---------------------------------------------//

        // This pushes a newline after 16 characters.
        let mut hex_decoded_data = String::new();
        for (j, i) in self.packed_file_data.iter().enumerate() {
            if j % 16 == 0 && j != 0 { hex_decoded_data.push('\n'); }
            let character = *i as char;

            // If is a valid UTF-8 char, show it. Otherwise, default to '.'.
            if character.is_alphanumeric() { hex_decoded_data.push(character); }
            else { hex_decoded_data.push('.'); }
        }

        // Add all the "Decoded" lines to the TextEdit.
        let qhex_decoded_data = QString::from_std_str(&hex_decoded_data);
        let text_size = font_metrics.size_2a(0, &qhex_decoded_data);
        self.get_mut_ptr_hex_view_decoded().set_text(&qhex_decoded_data);
        self.get_mut_ptr_hex_view_decoded().set_fixed_width(text_size.width() + 34);

        //---------------------------------------------//
        // Header Marking section.
        //---------------------------------------------//

        let use_dark_theme = SETTINGS.lock().unwrap().settings_bool["use_dark_theme"];
        let brush = QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkRed } else { GlobalColor::Red });
        let mut header_format = QTextCharFormat::new();
        header_format.set_background(&brush);

        let header_size = match self.packed_file_type {
            PackedFileType::DB => DB::read_header(&self.packed_file_data)?.3,
            PackedFileType::Loc => loc::HEADER_SIZE,
            _ => unimplemented!()
        };

        // Block the signals during this, so we don't mess things up.
        let mut blocker = QSignalBlocker::from_q_object(self.get_mut_ptr_hex_view_raw().static_upcast_mut::<QObject>());
        let mut cursor = self.get_mut_ptr_hex_view_raw().text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, (header_size * 3) as i32);
        self.get_mut_ptr_hex_view_raw().set_text_cursor(&cursor);
        self.get_mut_ptr_hex_view_raw().set_current_char_format(&header_format);
        cursor.clear_selection();
        self.get_mut_ptr_hex_view_raw().set_text_cursor(&cursor);

        blocker.unblock();

        // Block the signals during this, so we don't mess things up.
        let mut blocker = QSignalBlocker::from_q_object(self.get_mut_ptr_hex_view_decoded().static_upcast_mut::<QObject>());
        let mut cursor = self.get_mut_ptr_hex_view_decoded().text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, (header_size + (header_size as f32 / 16.0).floor() as usize) as i32);
        self.get_mut_ptr_hex_view_decoded().set_text_cursor(&cursor);
        self.get_mut_ptr_hex_view_decoded().set_current_char_format(&header_format);
        cursor.clear_selection();
        self.get_mut_ptr_hex_view_decoded().set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Info section.
        //---------------------------------------------//

        // Load the "Info" data to the view.
        match self.packed_file_type {
            PackedFileType::DB => {
                if let Ok((version,_, entry_count, _)) = DB::read_header(&self.packed_file_data) {
                    self.get_mut_ptr_packed_file_info_version_decoded_label().set_text(&QString::from_std_str(format!("{}", version)));
                    self.get_mut_ptr_packed_file_info_entry_count_decoded_label().set_text(&QString::from_std_str(format!("{}", entry_count)));
                }
            }
            PackedFileType::Loc => {
                if let Ok((version, entry_count)) = Loc::read_header(&self.packed_file_data) {
                    self.get_mut_ptr_packed_file_info_version_decoded_label().set_text(&QString::from_std_str(format!("{}", version)));
                    self.get_mut_ptr_packed_file_info_entry_count_decoded_label().set_text(&QString::from_std_str(format!("{}", entry_count)));
                }
            }
            _ => unimplemented!()
        }

        Ok(())
    }

    fn get_mut_ptr_hex_view_index(&self) -> MutPtr<QTextEdit> {
        mut_ptr_from_atomic(&self.hex_view_index)
    }

    fn get_mut_ptr_hex_view_raw(&self) -> MutPtr<QTextEdit> {
        mut_ptr_from_atomic(&self.hex_view_raw)
    }

    fn get_mut_ptr_hex_view_decoded(&self) -> MutPtr<QTextEdit> {
        mut_ptr_from_atomic(&self.hex_view_decoded)
    }

    fn get_mut_ptr_packed_file_info_version_decoded_label(&self) -> MutPtr<QLabel> {
        mut_ptr_from_atomic(&self.packed_file_info_version_decoded_label)
    }

    fn get_mut_ptr_packed_file_info_entry_count_decoded_label(&self) -> MutPtr<QLabel> {
        mut_ptr_from_atomic(&self.packed_file_info_entry_count_decoded_label)
    }

    fn get_mut_ptr_table_view(&self) -> MutPtr<QTableView> {
        mut_ptr_from_atomic(&self.table_view)
    }

    fn get_mut_ptr_table_view_old_versions(&self) -> MutPtr<QTableView> {
        mut_ptr_from_atomic(&self.table_view_old_versions)
    }

    fn get_mut_ptr_table_view_context_menu_move_up(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.table_view_context_menu_move_up)
    }

    fn get_mut_ptr_table_view_context_menu_move_down(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.table_view_context_menu_move_down)
    }

    fn get_mut_ptr_table_view_context_menu_delete(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.table_view_context_menu_delete)
    }

    fn get_mut_ptr_table_view_old_versions_context_menu_load(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.table_view_old_versions_context_menu_load)
    }

    fn get_mut_ptr_table_view_old_versions_context_menu_delete(&self) -> MutPtr<QAction> {
        mut_ptr_from_atomic(&self.table_view_old_versions_context_menu_delete)
    }
}

/// Implementation of `PackedFileDecoderViewRaw`.
impl PackedFileDecoderViewRaw {

    /// This function syncronize the selection between the Hex View and the Decoded View of the PackedFile Data.
    /// Pass `hex = true` if the selected view is the Hex View. Otherwise, pass false.
    pub unsafe fn hex_selection_sync(&mut self, hex: bool) {

        let cursor = if hex { self.hex_view_raw.text_cursor() } else { self.hex_view_decoded.text_cursor() };
        let mut cursor_dest = if !hex { self.hex_view_raw.text_cursor() } else { self.hex_view_decoded.text_cursor() };

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
            let mut blocker = QSignalBlocker::from_q_object(self.hex_view_decoded);
            self.hex_view_decoded.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
        else {
            let mut blocker = QSignalBlocker::from_q_object(self.hex_view_raw);
            self.hex_view_raw.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    unsafe fn load_definition(&mut self) {

    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    unsafe fn load_versions_list(&mut self) {
        self.table_model_old_versions.clear();
        if let Some(ref schema) = *SCHEMA.read().unwrap() {

            // Depending on the type, get one version list or another.
            let versioned_file = match self.packed_file_type {
                PackedFileType::DB => schema.get_ref_versioned_file_db(&self.packed_file_path[1]),
                PackedFileType::Loc => schema.get_ref_versioned_file_loc(),
                _ => unimplemented!(),
            };

            // And get all the versions of this table, and list them in their TreeView, if we have any.
            if let Ok(versioned_file) = versioned_file {
                versioned_file.get_version_list().iter().map(|x| x.version).for_each(|version| {
                    let item = QStandardItem::from_q_string(&QString::from_std_str(format!("{}", version)));
                    self.table_model_old_versions.append_row_q_standard_item(item.into_ptr());
                });
            }
        }

        self.table_model_old_versions.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Versions Decoded")));
        self.table_view_old_versions.horizontal_header().set_section_resize_mode_1a(ResizeMode::Stretch);
    }
}
