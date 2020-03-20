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
use qt_gui::QListOfQStandardItem;
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
use qt_core::CheckState;
use qt_core::QStringList;

use cpp_core::MutPtr;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, atomic::AtomicPtr, Mutex};

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::common::decoder::*;
use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::table::db::DB;
use rpfm_lib::packedfile::table::{loc, loc::Loc};
use rpfm_lib::schema::{Definition, Field, FieldType, Schema, VersionedFile};
use rpfm_lib::SCHEMA;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::ffi::add_to_q_list_safe;
use crate::ffi::new_combobox_item_delegate_safe;
use crate::ffi::new_spinbox_item_delegate_safe;
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
    pub clear_definition_button: MutPtr<QPushButton>,
    pub save_button: MutPtr<QPushButton>,

    pub packed_file_type: PackedFileType,
    pub packed_file_path: Vec<String>,
    pub packed_file_data: Arc<Vec<u8>>,
}

/// This struct contains data we need to keep separated from the other two due to mutability issues.
#[derive(Clone)]
pub struct PackedFileDecoderMutableData {
    pub index: Arc<Mutex<usize>>,
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
        app_ui: &AppUI,
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
        let mut clear_definition_button = QPushButton::from_q_string(&QString::from_std_str("Remove all fields"));
        let mut save_button = QPushButton::from_q_string(&QString::from_std_str("Finish it!"));

        // Add them to the Dialog.
        button_box_layout.add_widget_5a(&mut test_definition_button, 0, 0, 1, 1);
        button_box_layout.add_widget_5a(&mut clear_definition_button, 0, 1, 1, 1);
        button_box_layout.add_widget_5a(&mut save_button, 0, 2, 1, 1);

        layout.add_widget_5a(button_box.into_ptr(), 4, 1, 1, 2);

        layout.set_column_stretch(1, 10);
        layout.set_row_stretch(0, 10);
        layout.set_row_stretch(2, 5);

        let header_size = get_header_size(
            &packed_file_type,
            &packed_file.get_raw_data()?
        )?;

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
            clear_definition_button: clear_definition_button.into_ptr(),
            save_button: save_button.into_ptr(),

            packed_file_type,
            packed_file_path: packed_file.get_path().to_vec(),
            packed_file_data: Arc::new(packed_file.get_raw_data()?),
        };

        let packed_file_decoder_mutable_data = PackedFileDecoderMutableData {
            index: Arc::new(Mutex::new(header_size)),
        };

        let packed_file_decoder_view_slots = PackedFileDecoderViewSlots::new(
            packed_file_decoder_view_raw.clone(),
            packed_file_decoder_mutable_data.clone(),
            app_ui.clone(),
            *pack_file_contents_ui,
            *global_search_ui,
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
            clear_definition_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.clear_definition_button),
            save_button: atomic_from_mut_ptr(packed_file_decoder_view_raw.save_button),

            packed_file_type,
            packed_file_path: packed_file.get_path().to_vec(),
            packed_file_data: packed_file_decoder_view_raw.packed_file_data.clone(),
        };

        let definition = get_definition(
            &packed_file_decoder_view.packed_file_type,
            &packed_file_decoder_view.packed_file_path,
            &packed_file_decoder_view.packed_file_data,
            None
        );

        let fields = if let Some(definition) = definition {
            definition.fields.to_vec()
        } else { vec![] };

        packed_file_decoder_view.load_packed_file_data()?;
        packed_file_decoder_view_raw.load_versions_list();
        packed_file_decoder_view_raw.update_view(&fields, true, &mut packed_file_decoder_mutable_data.index.lock().unwrap())?;
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
        let header_size = get_header_size(&self.packed_file_type, &self.packed_file_data)?;
        let brush = QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkRed } else { GlobalColor::Red });
        let mut header_format = QTextCharFormat::new();
        header_format.set_background(&brush);

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

    fn get_mut_ptr_bool_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.bool_button)
    }

    fn get_mut_ptr_float_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.float_button)
    }

    fn get_mut_ptr_integer_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.integer_button)
    }

    fn get_mut_ptr_long_integer_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.long_integer_button)
    }

    fn get_mut_ptr_string_u8_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.string_u8_button)
    }

    fn get_mut_ptr_string_u16_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.string_u16_button)
    }

    fn get_mut_ptr_optional_string_u8_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.optional_string_u8_button)
    }

    fn get_mut_ptr_optional_string_u16_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.optional_string_u16_button)
    }

    fn get_mut_ptr_packed_file_info_version_decoded_label(&self) -> MutPtr<QLabel> {
        mut_ptr_from_atomic(&self.packed_file_info_version_decoded_label)
    }

    fn get_mut_ptr_packed_file_info_entry_count_decoded_label(&self) -> MutPtr<QLabel> {
        mut_ptr_from_atomic(&self.packed_file_info_entry_count_decoded_label)
    }

    fn get_mut_ptr_table_model(&self) -> MutPtr<QStandardItemModel> {
        mut_ptr_from_atomic(&self.table_model)
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

    fn get_mut_ptr_test_definition_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.test_definition_button)
    }

    fn get_mut_ptr_clear_definition_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.clear_definition_button)
    }

    fn get_mut_ptr_save_button(&self) -> MutPtr<QPushButton> {
        mut_ptr_from_atomic(&self.save_button)
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

    /// This function is used to update the state of the decoder view every time a change it's done.
    unsafe fn update_view(
        &mut self,
        field_list: &[Field],
        is_initial_load: bool,
        mut index: &mut usize,
    ) -> Result<()> {

        // If it's the first load, we have to prepare the table's column data.
        if is_initial_load {

            // If the table is empty, we just load a fake row, so the column headers are created properly.
            if field_list.is_empty() {
                let mut qlist = QListOfQStandardItem::new();
                (0..12).for_each(|_| add_to_q_list_safe(qlist.as_mut_ptr(), QStandardItem::new().into_ptr()));
                self.table_model.append_row_q_list_of_q_standard_item(&qlist);
                configure_table_view(self.table_view);
                self.table_model.remove_rows_2a(0, 1);
            }

            // Otherswise, we add each field we got as a row to the table.
            else {
                for field in field_list {
                    self.add_field_to_view(&field, &mut index);
                }
                configure_table_view(self.table_view);
            }
        }

        let decoded_bool = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::Boolean, &mut index.clone());
        let decoded_float = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::Float, &mut index.clone());
        let decoded_integer = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::Integer, &mut index.clone());
        let decoded_long_integer = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::LongInteger, &mut index.clone());
        let decoded_string_u8 = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::StringU8, &mut index.clone());
        let decoded_string_u16 = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::StringU16, &mut index.clone());
        let decoded_optional_string_u8 = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::OptionalStringU8, &mut index.clone());
        let decoded_optional_string_u16 = Self::decode_data_by_fieldtype(&self.packed_file_data, &FieldType::OptionalStringU16, &mut index.clone());

        // We update all the decoded entries here.
        self.bool_line_edit.set_text(&QString::from_std_str(decoded_bool));
        self.float_line_edit.set_text(&QString::from_std_str(decoded_float));
        self.integer_line_edit.set_text(&QString::from_std_str(decoded_integer));
        self.long_integer_line_edit.set_text(&QString::from_std_str(decoded_long_integer));
        self.string_u8_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8)));
        self.string_u16_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16)));
        self.optional_string_u8_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8)));
        self.optional_string_u16_line_edit.set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16)));

        //---------------------------------------------//
        // Raw data cleaning section.
        //---------------------------------------------//

        // Prepare to paint the changes in the hex data views.
        let header_size = get_header_size(&self.packed_file_type, &self.packed_file_data)?;
        let use_dark_theme = SETTINGS.lock().unwrap().settings_bool["use_dark_theme"];
        let mut index_format = QTextCharFormat::new();
        let mut decoded_format = QTextCharFormat::new();
        let mut neutral_format = QTextCharFormat::new();
        index_format.set_background(&QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkMagenta } else { GlobalColor::Magenta }));
        decoded_format.set_background(&QBrush::from_global_color(if use_dark_theme { GlobalColor::DarkYellow } else { GlobalColor::Yellow }));
        neutral_format.set_background(&QBrush::from_global_color(GlobalColor::Transparent));

        // Clean both TextEdits, so we can repaint all the changes on them.
        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (header_size * 3) as i32);
        cursor.move_position_2a(MoveOperation::End, MoveMode::KeepAnchor);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&neutral_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_decoded.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (header_size + (header_size as f32 / 16.0).floor() as usize) as i32);
        cursor.move_position_2a(MoveOperation::End, MoveMode::KeepAnchor);

        self.hex_view_decoded.set_text_cursor(&cursor);
        self.hex_view_decoded.set_current_char_format(&neutral_format);
        cursor.clear_selection();
        self.hex_view_decoded.set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Raw data painting decoded data section.
        //---------------------------------------------//

        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (header_size * 3) as i32);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, ((*index - header_size) * 3) as i32);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&decoded_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_decoded.text_cursor();

        // Create the "Selection" for the decoded row.
        let positions_to_move_end = *index / 16;
        let positions_to_move_start = header_size / 16;
        let positions_to_move_vertical = positions_to_move_end - positions_to_move_start;
        let positions_to_move_horizontal = *index - header_size;
        let positions_to_move = positions_to_move_horizontal + positions_to_move_vertical;

        cursor.move_position_1a(MoveOperation::Start);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::MoveAnchor, (header_size + (header_size as f32 / 16.0).floor() as usize) as i32);
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, positions_to_move as i32);

        self.hex_view_decoded.set_text_cursor(&cursor);
        self.hex_view_decoded.set_current_char_format(&decoded_format);
        cursor.clear_selection();
        self.hex_view_decoded.set_text_cursor(&cursor);

        blocker.unblock();

        //---------------------------------------------//
        // Raw data painting current index section.
        //---------------------------------------------//

        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_raw.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_raw.text_cursor();
        cursor.move_position_3a(MoveOperation::NextCharacter, MoveMode::KeepAnchor, 3);

        self.hex_view_raw.set_text_cursor(&cursor);
        self.hex_view_raw.set_current_char_format(&index_format);
        cursor.clear_selection();
        self.hex_view_raw.set_text_cursor(&cursor);

        blocker.unblock();

        let mut blocker = QSignalBlocker::from_q_object(self.hex_view_decoded.static_upcast_mut::<QObject>());
        let mut cursor = self.hex_view_decoded.text_cursor();
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
        &mut self,
        field: &Field,
        mut index: &mut usize,
    ) {

        // Decode the data from the field.
        let decoded_data = Self::decode_data_by_fieldtype(
            &self.packed_file_data,
            &field.field_type,
            &mut index
        );

        // Get the type of the data we are going to put into the Table.
        let field_type = match field.field_type {
            FieldType::Boolean => "Bool",
            FieldType::Float => "Float",
            FieldType::Integer => "Integer",
            FieldType::LongInteger => "LongInteger",
            FieldType::StringU8 => "StringU8",
            FieldType::StringU16 => "StringU16",
            FieldType::OptionalStringU8 => "OptionalStringU8",
            FieldType::OptionalStringU16 => "OptionalStringU16",
            FieldType::Sequence(_) => "Sequence",
        };

        // Create a new list of StandardItem.
        let mut qlist = QListOfQStandardItem::new();

        // Create the items of the new row.
        let field_name = QStandardItem::from_q_string(&QString::from_std_str(&field.name));
        let field_type = QStandardItem::from_q_string(&QString::from_std_str(field_type));
        let mut field_is_key = QStandardItem::new();
        field_is_key.set_editable(false);
        field_is_key.set_checkable(true);
        field_is_key.set_check_state(if field.is_key { CheckState::Checked } else { CheckState::Unchecked });

        let (field_reference_table, field_reference_field) = if let Some(ref reference) = field.is_reference {
            (QStandardItem::from_q_string(&QString::from_std_str(&reference.0)), QStandardItem::from_q_string(&QString::from_std_str(&reference.1)))
        } else { (QStandardItem::new(), QStandardItem::new()) };

        let field_lookup_columns = if let Some(ref columns) = field.lookup {
            QStandardItem::from_q_string(&QString::from_std_str(columns.join(", ")))
        } else { QStandardItem::new() };

        let mut decoded_data = QStandardItem::from_q_string(&QString::from_std_str(&decoded_data));
        decoded_data.set_editable(false);

        let field_default_value = if let Some(ref default_value) = field.default_value {
            QStandardItem::from_q_string(&QString::from_std_str(&default_value))
        } else { QStandardItem::new() };

        let field_max_length = QStandardItem::from_q_string(&QString::from_std_str(&format!("{}", field.max_length)));
        let mut field_is_filename = QStandardItem::new();
        field_is_filename.set_editable(false);
        field_is_filename.set_checkable(true);
        field_is_filename.set_check_state(if field.is_filename { CheckState::Checked } else { CheckState::Unchecked });

        let field_filename_relative_path = if let Some(ref filename_relative_path) = field.filename_relative_path {
            QStandardItem::from_q_string(&QString::from_std_str(&filename_relative_path))
        } else { QStandardItem::new() };

        let field_ca_order = QStandardItem::from_q_string(&QString::from_std_str(&format!("{}", field.ca_order)));
        let field_description = QStandardItem::from_q_string(&QString::from_std_str(&field.description));

        add_to_q_list_safe(qlist.as_mut_ptr(), field_name.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_type.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), decoded_data.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_is_key.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_reference_table.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_reference_field.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_lookup_columns.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_default_value.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_max_length.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_is_filename.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_filename_relative_path.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_ca_order.into_ptr());
        add_to_q_list_safe(qlist.as_mut_ptr(), field_description.into_ptr());

        self.table_model.append_row_q_list_of_q_standard_item(&qlist);
    }

    /// This function is the one that takes care of actually decoding the provided data based on the field type.
    fn decode_data_by_fieldtype(
        packed_file_data: &[u8],
        field_type: &FieldType,
        mut index: &mut usize
    ) -> String {
        match field_type {
            FieldType::Boolean => {
                match packed_file_data.decode_packedfile_bool(*index, &mut index) {
                    Ok(result) => {
                        if result { "True".to_string() }
                        else { "False".to_string() }
                    }
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::Float => {
                match packed_file_data.decode_packedfile_float_f32(*index, &mut index) {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::Integer => {
                match packed_file_data.decode_packedfile_integer_i32(*index, &mut index) {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::LongInteger => {
                match packed_file_data.decode_packedfile_integer_i64(*index, &mut index) {
                    Ok(result) => result.to_string(),
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::StringU8 => {
                match packed_file_data.decode_packedfile_string_u8(*index, &mut index) {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::StringU16 => {
                match packed_file_data.decode_packedfile_string_u16(*index, &mut index) {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalStringU8 => {
                match packed_file_data.decode_packedfile_optional_string_u8(*index, &mut index) {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
            FieldType::OptionalStringU16 => {
                match packed_file_data.decode_packedfile_optional_string_u16(*index, &mut index) {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },

            // TODO: Finish this.
            FieldType::Sequence(_) => {
                match packed_file_data.decode_packedfile_optional_string_u16(*index, &mut index) {
                    Ok(result) => result,
                    Err(_) => "Error".to_owned(),
                }
            },
        }
    }

    /// This function updates the "First Row Decoded" column of the table, then forces an update of the rest of the view.
    ///
    /// To be triggered when the table changes.
    unsafe fn update_rows_decoded(
        &mut self,
        mut index: &mut usize,
    ) -> Result<()> {

        // Reset the index, because this function effectively resets the decoding state.
        *index = get_header_size(&self.packed_file_type, &self.packed_file_data)?;
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get the ModelIndex of the cell we want to update.
            let model_index = self.table_model.index_2a(row, 2);
            if model_index.is_valid() {

                // Get the row's type.
                let row_type = self.table_model.index_2a(row, 1);
                let field_type = match &*row_type.data_1a(0).to_string().to_std_string() {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,
                    "Sequence" => FieldType::Sequence(Definition::new(-1)),
                    _ => unimplemented!()
                };

                // Get the decoded data using it's type...
                let decoded_data = Self::decode_data_by_fieldtype(
                    &self.packed_file_data,
                    &field_type,
                    &mut index
                );

                // Get the item from the "First Row Decoded" column.
                let mut item = self.table_model.item_from_index(&model_index);
                item.set_text(&QString::from_std_str(decoded_data));
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Update the entire decoder to use the new index.
        self.update_view(&[], false, &mut index)
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

    /// This function is used to update the decoder view when we try to add a new field to
    /// the definition with one of the "Use this" buttons.
    pub unsafe fn use_this(
        &mut self,
        field_type: FieldType,
        mut index: &mut usize,
    ) -> Result<()> {
        let mut field = Field::default();
        field.field_type = field_type;

        self.add_field_to_view(&field, &mut index);
        self.update_view(&[], false, &mut index)
    }


    /// This function gets the data from the decoder's table and returns it, so we can save it to a Definition.
    pub unsafe fn get_fields_from_view(&self) -> Vec<Field> {
        let mut fields = vec![];
        let mut row = 0;

        loop {

            let model_index = self.table_model.index_2a(row, 0);
            if model_index.is_valid() {

                // Get the data from each field of the row...
                let field_name = self.table_model.item_2a(row, 0).text().to_std_string();
                let field_type = self.table_model.item_2a(row, 1).text().to_std_string();
                let field_is_key = if self.table_model.item_2a(row, 3).check_state() == CheckState::Checked { true } else { false };
                let ref_table = self.table_model.item_2a(row, 4).text().to_std_string();
                let ref_column = self.table_model.item_2a(row, 5).text().to_std_string();
                let field_default_value = self.table_model.item_2a(row, 7).text().to_std_string();
                let field_max_length = self.table_model.item_2a(row, 8).text().to_std_string().parse::<i32>().unwrap();
                let field_is_filename = if self.table_model.item_2a(row, 9).check_state() == CheckState::Checked { true } else { false };
                let field_filename_relative_path = self.table_model.item_2a(row, 10).text().to_std_string();
                let field_ca_order = self.table_model.item_2a(row, 11).text().to_std_string().parse::<i16>().unwrap();
                let field_description = self.table_model.item_2a(row, 12).text().to_std_string();

                // Get the proper type of the field. If invalid, default to OptionalStringU16.
                let field_type = match &*field_type {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,
                    "Sequence" | _=> FieldType::Sequence(Definition::new(-1)),
                };

                let field_is_reference = if !ref_table.is_empty() && !ref_column.is_empty() {
                    Some((ref_table, ref_column))
                } else { None };

                let field_lookup = self.table_model.item_2a(row, 6).text().to_std_string();
                let field_lookup = if !field_lookup.is_empty() {
                    Some(field_lookup.split(',').map(|x| x.to_owned()).collect::<Vec<String>>())
                } else { None };

                fields.push(
                    Field::new(
                        field_name,
                        field_type,
                        field_is_key,
                        if field_default_value.is_empty() { None } else { Some(field_default_value) },
                        field_max_length,
                        field_is_filename,
                        if field_filename_relative_path.is_empty() { None } else { Some(field_filename_relative_path) },
                        field_is_reference,
                        field_lookup,
                        field_description,
                        field_ca_order
                    )
                );
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        fields
    }

    /// This function adds the definition currently in the view to a temporal schema, and returns it.
    unsafe fn add_definition_to_schema(&self) -> Schema {
        let mut schema = SCHEMA.read().unwrap().clone().unwrap();
        let fields = self.get_fields_from_view();

        let version = match self.packed_file_type {
            PackedFileType::DB => DB::read_header(&self.packed_file_data).unwrap().0,
            PackedFileType::Loc => Loc::read_header(&self.packed_file_data).unwrap().0,
            _ => unimplemented!(),
        };

        let versioned_file = match self.packed_file_type {
            PackedFileType::DB => schema.get_ref_mut_versioned_file_db(&self.packed_file_path[1]),
            PackedFileType::Loc => schema.get_ref_mut_versioned_file_loc(),
            _ => unimplemented!(),
        };

        match versioned_file {
            Ok(versioned_file) => {
                match versioned_file.get_ref_mut_version(version) {
                    Ok(definition) => definition.fields = fields,
                    Err(_) => {
                        let mut definition = Definition::new(version);
                        definition.fields = fields;
                        versioned_file.add_version(&definition);
                    }
                }
            }
            Err(_) => {
                let mut definition = Definition::new(version);
                definition.fields = fields;

                let definitions = vec![definition];
                let versioned_file = match self.packed_file_type {
                    PackedFileType::DB => VersionedFile::DB(self.packed_file_path[1].to_owned(), definitions),
                    PackedFileType::Loc => VersionedFile::Loc(definitions),
                    PackedFileType::DependencyPackFilesList => VersionedFile::DepManager(definitions),
                    _ => unimplemented!()
                };

                schema.add_versioned_file(&versioned_file);
            }
        }

        schema
    }
}

/// This function returns the header size (or first byte after the header) of the provided PackedFile.
fn get_header_size(
    packed_file_type: &PackedFileType,
    packed_file_data: &[u8],
) -> Result<usize> {
    match packed_file_type {
        PackedFileType::DB => Ok(DB::read_header(packed_file_data)?.3),
        PackedFileType::Loc => Ok(loc::HEADER_SIZE),
        _ => unimplemented!()
    }
}

/// This function returns the definition corresponding to the decoded Packedfile, if exists.
fn get_definition(
    packed_file_type: &PackedFileType,
    packed_file_path: &[String],
    packed_file_data: &[u8],
    version: Option<i32>
) -> Option<Definition> {
    if let Some(ref schema) = *SCHEMA.read().unwrap() {

        // Depending on the type, get one version list or another.
        let versioned_file = match packed_file_type {
            PackedFileType::DB => schema.get_ref_versioned_file_db(&packed_file_path[1]),
            PackedFileType::Loc => schema.get_ref_versioned_file_loc(),
            _ => unimplemented!(),
        };

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Ok(versioned_file) = versioned_file {
            let version = if let Some(version) = version { version } else { match packed_file_type {
                PackedFileType::DB => DB::read_header(packed_file_data).ok()?.0,
                PackedFileType::Loc => Loc::read_header(packed_file_data).ok()?.0,
                _ => unimplemented!(),
            }};

            return versioned_file.get_version(version).ok().cloned()
        }
    }

    None
}

/// This function configures the provided TableView, so it has the right columns and it's resized to the right size.
unsafe fn configure_table_view(table_view: MutPtr<QTableView>) {
    let mut table_model = table_view.model();
    table_model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Field Name")));
    table_model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Field Type")));
    table_model.set_header_data_3a(2, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("First Row Decoded")));
    table_model.set_header_data_3a(3, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Is key?")));
    table_model.set_header_data_3a(4, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Ref. to Table")));
    table_model.set_header_data_3a(5, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Ref. to Column")));
    table_model.set_header_data_3a(6, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Lookup Columns")));
    table_model.set_header_data_3a(7, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Default Value")));
    table_model.set_header_data_3a(8, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Max Lenght")));
    table_model.set_header_data_3a(9, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Is Filename")));
    table_model.set_header_data_3a(10, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Filename Relative Path")));
    table_model.set_header_data_3a(11, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("CA Order")));
    table_model.set_header_data_3a(12, Orientation::Horizontal, &QVariant::from_q_string(&QString::from_std_str("Description")));
    table_view.horizontal_header().set_stretch_last_section(true);
    table_view.horizontal_header().resize_sections(ResizeMode::ResizeToContents);

    // The second field should be a combobox.
    let mut list = QStringList::new();
    list.append_q_string(&QString::from_std_str("Bool"));
    list.append_q_string(&QString::from_std_str("Float"));
    list.append_q_string(&QString::from_std_str("Integer"));
    list.append_q_string(&QString::from_std_str("LongInteger"));
    list.append_q_string(&QString::from_std_str("StringU8"));
    list.append_q_string(&QString::from_std_str("StringU16"));
    list.append_q_string(&QString::from_std_str("OptionalStringU8"));
    list.append_q_string(&QString::from_std_str("OptionalStringU16"));
    list.append_q_string(&QString::from_std_str("Sequence"));
    new_combobox_item_delegate_safe(&mut table_view.static_upcast_mut(), 1, list.into_ptr().as_ptr(), false, 0);

    // Fields Max lenght and CA Order must be numeric.
    new_spinbox_item_delegate_safe(&mut table_view.static_upcast_mut(), 8, 32);
    new_spinbox_item_delegate_safe(&mut table_view.static_upcast_mut(), 11, 16);
}
