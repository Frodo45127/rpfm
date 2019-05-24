//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Here it goes all the DB Decodeŕ's code.

use qt_widgets::abstract_item_view::{EditTrigger, SelectionMode};
use qt_widgets::action::Action;
use qt_widgets::frame::Frame;
use qt_widgets::group_box::GroupBox;
use qt_widgets::header_view::ResizeMode;
use qt_widgets::menu::Menu;
use qt_widgets::label::Label;
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::splitter::Splitter;
use qt_widgets::text_edit::TextEdit;
use qt_widgets::table_view::TableView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;
use qt_gui::font::{Font, StyleHint };
use qt_gui::font_metrics::FontMetrics;
use qt_gui::key_sequence::KeySequence;
use qt_gui::list::ListStandardItemMutPtr;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;
use qt_gui::text_char_format::TextCharFormat;
use qt_gui::text_cursor::{MoveOperation, MoveMode};

use qt_core::signal_blocker::SignalBlocker;
use qt_core::abstract_item_model::AbstractItemModel;
use qt_core::connection::Signal;
use qt_core::variant::Variant;
use qt_core::object::Object;
use qt_core::slots::{SlotBool, SlotCInt, SlotItemSelectionRefItemSelectionRef, SlotModelIndexRefModelIndexRefVectorVectorCIntRef};
use qt_core::string_list::StringList;
use qt_core::qt::{Orientation, CheckState, ContextMenuPolicy, ShortcutContext, SortOrder, GlobalColor};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use crate::AppUI;
use crate::Commands;
use crate::Data;
use crate::QString;
use crate::common::*;
use crate::common::communications::*;
use crate::error::{ErrorKind, Result};
use crate::ui::*;

/// Struct PackedFileDBDecoder: contains all the stuff we need to return to be able to decode DB PackedFiles.
pub struct PackedFileDBDecoder {
    pub slot_hex_view_scroll_sync: SlotCInt<'static>,
    pub slot_hex_view_raw_selection_sync: SlotNoArgs<'static>,
    pub slot_hex_view_decoded_selection_sync: SlotNoArgs<'static>,
    pub slot_hex_view_raw_selection_decoding: SlotNoArgs<'static>,
    pub slot_hex_view_decoded_selection_decoding: SlotNoArgs<'static>,
    pub slot_use_this_bool: SlotNoArgs<'static>,
    pub slot_use_this_float: SlotNoArgs<'static>,
    pub slot_use_this_integer: SlotNoArgs<'static>,
    pub slot_use_this_long_integer: SlotNoArgs<'static>,
    pub slot_use_this_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_string_u16: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u8: SlotNoArgs<'static>,
    pub slot_use_this_optional_string_u16: SlotNoArgs<'static>,
    pub slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef<'static>,
    pub slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_context_menu_move_up: SlotBool<'static>,
    pub slot_table_view_context_menu_move_down: SlotBool<'static>,
    pub slot_table_view_context_menu_delete: SlotBool<'static>,
    pub slot_generate_pretty_diff: SlotNoArgs<'static>,
    pub slot_remove_all_fields: SlotNoArgs<'static>,
    pub slot_save_definition: SlotNoArgs<'static>,
    pub slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef<'static>,
    pub slot_table_view_old_versions_context_menu: SlotQtCorePointRef<'static>,
    pub slot_table_view_old_versions_context_menu_load: SlotBool<'static>,
    pub slot_table_view_old_versions_context_menu_delete: SlotBool<'static>,
}

/// Struct PackedFileDBDecoderStuff: contains all the ui things from the decoder view, so we can pass the easely.
#[derive(Copy, Clone)]
pub struct PackedFileDBDecoderStuff {
    pub hex_view_index: *mut TextEdit,
    pub hex_view_raw: *mut TextEdit,
    pub hex_view_decoded: *mut TextEdit,
    pub table_view: *mut TableView,
    pub table_model: *mut StandardItemModel,

    pub selection_bool_line_edit: *mut LineEdit,
    pub selection_float_line_edit: *mut LineEdit,
    pub selection_integer_line_edit: *mut LineEdit,
    pub selection_long_integer_line_edit: *mut LineEdit,
    pub selection_string_u8_line_edit: *mut LineEdit,
    pub selection_string_u16_line_edit: *mut LineEdit,
    pub selection_optional_string_u8_line_edit: *mut LineEdit,
    pub selection_optional_string_u16_line_edit: *mut LineEdit,

    pub bool_line_edit: *mut LineEdit,
    pub float_line_edit: *mut LineEdit,
    pub integer_line_edit: *mut LineEdit,
    pub long_integer_line_edit: *mut LineEdit,
    pub string_u8_line_edit: *mut LineEdit,
    pub string_u16_line_edit: *mut LineEdit,
    pub optional_string_u8_line_edit: *mut LineEdit,
    pub optional_string_u16_line_edit: *mut LineEdit,

    pub bool_button: *mut PushButton,
    pub float_button: *mut PushButton,
    pub integer_button: *mut PushButton,
    pub long_integer_button: *mut PushButton,
    pub string_u8_button: *mut PushButton,
    pub string_u16_button: *mut PushButton,
    pub optional_string_u8_button: *mut PushButton,
    pub optional_string_u16_button: *mut PushButton,

    pub table_info_type_decoded_label: *mut Label,
    pub table_info_version_decoded_label: *mut Label,
    pub table_info_entry_count_decoded_label: *mut Label,

    pub table_view_old_versions: *mut TableView,
    pub table_model_old_versions: *mut StandardItemModel,

    pub generate_pretty_diff_button: *mut PushButton,
    pub clear_definition_button: *mut PushButton,
    pub save_button: *mut PushButton,

    pub table_view_context_menu: *mut Menu,
    pub table_view_context_menu_move_up: *mut Action,
    pub table_view_context_menu_move_down: *mut Action,
    pub table_view_context_menu_delete: *mut Action,

    pub table_view_old_versions_context_menu: *mut Menu,
    pub table_view_old_versions_context_menu_load: *mut Action,
    pub table_view_old_versions_context_menu_delete: *mut Action,
}

/// Struct PackedFileDBDecoderStuffNonUI: contains data needed for the decoder to properly work.
#[derive(Clone)]
pub struct PackedFileDBDecoderStuffNonUI {
    pub packed_file_path: Vec<String>,
    pub packed_file_data: Vec<u8>,
    pub initial_index: usize,
    pub version: i32,
    pub entry_count: u32,
}

/// Implementation of PackedFileDBDecoder.
impl PackedFileDBDecoder {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            slot_hex_view_scroll_sync: SlotCInt::new(|_| {}),
            slot_hex_view_raw_selection_sync: SlotNoArgs::new(|| {}),
            slot_hex_view_decoded_selection_sync: SlotNoArgs::new(|| {}),
            slot_hex_view_raw_selection_decoding: SlotNoArgs::new(|| {}),
            slot_hex_view_decoded_selection_decoding: SlotNoArgs::new(|| {}),
            slot_use_this_bool: SlotNoArgs::new(|| {}),
            slot_use_this_float: SlotNoArgs::new(|| {}),
            slot_use_this_integer: SlotNoArgs::new(|| {}),
            slot_use_this_long_integer: SlotNoArgs::new(|| {}),
            slot_use_this_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_string_u16: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u8: SlotNoArgs::new(|| {}),
            slot_use_this_optional_string_u16: SlotNoArgs::new(|| {}),
            slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(|_,_,_| {}),
            slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_context_menu_move_up: SlotBool::new(|_| {}),
            slot_table_view_context_menu_move_down: SlotBool::new(|_| {}),
            slot_table_view_context_menu_delete: SlotBool::new(|_| {}),
            slot_generate_pretty_diff: SlotNoArgs::new(|| {}),
            slot_remove_all_fields: SlotNoArgs::new(|| {}),
            slot_save_definition: SlotNoArgs::new(|| {}),
            slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(|_,_| {}),
            slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(|_| {}),
            slot_table_view_old_versions_context_menu_load: SlotBool::new(|_| {}),
            slot_table_view_old_versions_context_menu_delete: SlotBool::new(|_| {}),
        }
    }

    /// This function creates the "Decoder View" with all the stuff needed to decode a table, and it
    /// returns it if it succeed. It can fail if the provided PackedFile is not a DB Table.
    pub fn create_decoder_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        packed_file_path: &[String],
    ) -> Result<(Self, Font)> {

        //---------------------------------------------------------------------------------------//
        // Create the UI of the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Create the widget that'll act as a container for the view.
        let widget = Widget::new().into_raw();
        let widget_layout = create_grid_layout_unsafe(widget);
        unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget); }

        // Create the hex view on the left side.
        let hex_view_group = GroupBox::new(&QString::from_std_str("PackedFile's Data")).into_raw();
        let hex_view_index = TextEdit::new(()).into_raw();
        let hex_view_raw = TextEdit::new(()).into_raw();
        let hex_view_decoded = TextEdit::new(()).into_raw();
        let hex_view_layout = create_grid_layout_unsafe(hex_view_group as *mut Widget);
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_index as *mut Widget, 0, 0, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_raw as *mut Widget, 0, 1, 1, 1)); }
        unsafe { hex_view_layout.as_mut().unwrap().add_widget((hex_view_decoded as *mut Widget, 0, 2, 1, 1)); }

        // Set them as "ReadOnly".
        unsafe { hex_view_index.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_raw.as_mut().unwrap().set_read_only(true); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_read_only(true); }

        // Set his font as "Monospace", so all the lines have the same lenght.
        let mut monospace_font = Font::new(&QString::from_std_str("Monospace"));
        monospace_font.set_style_hint(StyleHint::Monospace);
        unsafe { hex_view_index.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_raw.as_mut().unwrap().set_font(&monospace_font); }
        unsafe { hex_view_decoded.as_mut().unwrap().set_font(&monospace_font); }

        // Create the TableView at the top.
        let table_view = TableView::new().into_raw();
        let table_model = StandardItemModel::new(()).into_raw();
        unsafe { table_view.as_mut().unwrap().set_model(table_model as *mut AbstractItemModel); }
        unsafe { table_view.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }
        unsafe { table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        unsafe { table_view.as_mut().unwrap().set_alternating_row_colors(true); };

        // Create the Contextual Menu for the TableView.
        let mut table_view_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_context_menu_move_up = table_view_context_menu.add_action(&QString::from_std_str("Move &Up"));
        let table_view_context_menu_move_down = table_view_context_menu.add_action(&QString::from_std_str("&Move Down"));
        let table_view_context_menu_delete = table_view_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().db_decoder_fields["move_up"]))); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().db_decoder_fields["move_down"]))); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().db_decoder_fields["delete"]))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_up); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_move_down); }
        unsafe { table_view.as_mut().unwrap().add_action(table_view_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the fields splitter.
        let fields_splitter = Splitter::new(Orientation::Vertical).into_raw();
        unsafe { fields_splitter.as_mut().unwrap().set_collapsible(0, false); }
        unsafe { fields_splitter.as_mut().unwrap().set_collapsible(1, false); }

        // Create the frames for the info.
        let decoded_fields_frame = GroupBox::new(&QString::from_std_str("Current Field Decoded")).into_raw();
        let selected_fields_frame = GroupBox::new(&QString::from_std_str("Selected Field Decoded")).into_raw();
        let info_frame = GroupBox::new(&QString::from_std_str("Table Info")).into_raw();

        // Add the stuff to the splitter.
        unsafe { fields_splitter.as_mut().unwrap().add_widget(decoded_fields_frame as *mut Widget); }
        unsafe { fields_splitter.as_mut().unwrap().add_widget(selected_fields_frame as *mut Widget); }

        // Set their layouts.
        let decoded_fields_layout = create_grid_layout_unsafe(decoded_fields_frame as *mut Widget);
        let selected_fields_layout = create_grid_layout_unsafe(selected_fields_frame as *mut Widget);
        let info_layout = create_grid_layout_unsafe(info_frame as *mut Widget);
        unsafe { decoded_fields_layout.as_mut().unwrap().set_column_stretch(1, 10); }
        unsafe { selected_fields_layout.as_mut().unwrap().set_column_stretch(1, 10); }

        // Create the stuff for the decoded fields.
        let bool_label = Label::new(&QString::from_std_str("Decoded as \"Bool\":")).into_raw();
        let float_label = Label::new(&QString::from_std_str("Decoded as \"Float\":")).into_raw();
        let integer_label = Label::new(&QString::from_std_str("Decoded as \"Integer\":")).into_raw();
        let long_integer_label = Label::new(&QString::from_std_str("Decoded as \"Long Integer\":")).into_raw();
        let string_u8_label = Label::new(&QString::from_std_str("Decoded as \"String U8\":")).into_raw();
        let string_u16_label = Label::new(&QString::from_std_str("Decoded as \"String U16\":")).into_raw();
        let optional_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U8\":")).into_raw();
        let optional_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U16\":")).into_raw();

        let bool_line_edit = LineEdit::new(()).into_raw();
        let float_line_edit = LineEdit::new(()).into_raw();
        let integer_line_edit = LineEdit::new(()).into_raw();
        let long_integer_line_edit = LineEdit::new(()).into_raw();
        let string_u8_line_edit = LineEdit::new(()).into_raw();
        let string_u16_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u8_line_edit = LineEdit::new(()).into_raw();
        let optional_string_u16_line_edit = LineEdit::new(()).into_raw();

        let bool_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let float_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let long_integer_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u8_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();
        let optional_string_u16_button = PushButton::new(&QString::from_std_str("Use this")).into_raw();

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_label as *mut Widget, 3, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_label as *mut Widget, 4, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_label as *mut Widget, 5, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_label as *mut Widget, 6, 0, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_label as *mut Widget, 7, 0, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_line_edit as *mut Widget, 1, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_line_edit as *mut Widget, 2, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_line_edit as *mut Widget, 3, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_line_edit as *mut Widget, 4, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_line_edit as *mut Widget, 5, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_line_edit as *mut Widget, 6, 1, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_line_edit as *mut Widget, 7, 1, 1, 1)); }

        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((bool_button as *mut Widget, 0, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((float_button as *mut Widget, 1, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((integer_button as *mut Widget, 2, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((long_integer_button as *mut Widget, 3, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u8_button as *mut Widget, 4, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((string_u16_button as *mut Widget, 5, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u8_button as *mut Widget, 6, 2, 1, 1)); }
        unsafe { decoded_fields_layout.as_mut().unwrap().add_widget((optional_string_u16_button as *mut Widget, 7, 2, 1, 1)); }

        // Create the stuff for the "Selection" live decoding.
        let selection_bool_label = Label::new(&QString::from_std_str("Decoded as \"Bool\":")).into_raw();
        let selection_float_label = Label::new(&QString::from_std_str("Decoded as \"Float\":")).into_raw();
        let selection_integer_label = Label::new(&QString::from_std_str("Decoded as \"Integer\":")).into_raw();
        let selection_long_integer_label = Label::new(&QString::from_std_str("Decoded as \"Long Integer\":")).into_raw();
        let selection_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"String U8\":")).into_raw();
        let selection_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"String U16\":")).into_raw();
        let selection_optional_string_u8_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U8\":")).into_raw();
        let selection_optional_string_u16_label = Label::new(&QString::from_std_str("Decoded as \"Optional String U16\":")).into_raw();

        let selection_bool_line_edit = LineEdit::new(()).into_raw();
        let selection_float_line_edit = LineEdit::new(()).into_raw();
        let selection_integer_line_edit = LineEdit::new(()).into_raw();
        let selection_long_integer_line_edit = LineEdit::new(()).into_raw();
        let selection_string_u8_line_edit = LineEdit::new(()).into_raw();
        let selection_string_u16_line_edit = LineEdit::new(()).into_raw();
        let selection_optional_string_u8_line_edit = LineEdit::new(()).into_raw();
        let selection_optional_string_u16_line_edit = LineEdit::new(()).into_raw();

        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_bool_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_float_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_integer_label as *mut Widget, 2, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_long_integer_label as *mut Widget, 3, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u8_label as *mut Widget, 4, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u16_label as *mut Widget, 5, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u8_label as *mut Widget, 6, 0, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u16_label as *mut Widget, 7, 0, 1, 1)); }

        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_bool_line_edit as *mut Widget, 0, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_float_line_edit as *mut Widget, 1, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_integer_line_edit as *mut Widget, 2, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_long_integer_line_edit as *mut Widget, 3, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u8_line_edit as *mut Widget, 4, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_string_u16_line_edit as *mut Widget, 5, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u8_line_edit as *mut Widget, 6, 1, 1, 1)); }
        unsafe { selected_fields_layout.as_mut().unwrap().add_widget((selection_optional_string_u16_line_edit as *mut Widget, 7, 1, 1, 1)); }

        // Create stuff for the info frame.
        let table_info_type_label = Label::new(&QString::from_std_str("Table type:")).into_raw();
        let table_info_version_label = Label::new(&QString::from_std_str("Table version:")).into_raw();
        let table_info_entry_count_label = Label::new(&QString::from_std_str("Table entry count:")).into_raw();

        let table_info_type_decoded_label = Label::new(()).into_raw();
        let table_info_version_decoded_label = Label::new(()).into_raw();
        let table_info_entry_count_decoded_label = Label::new(()).into_raw();

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_label as *mut Widget, 2, 0, 1, 1)); }

        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_type_decoded_label as *mut Widget, 0, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_version_decoded_label as *mut Widget, 1, 1, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((table_info_entry_count_decoded_label as *mut Widget, 2, 1, 1, 1)); }

        // Create the TableView at the top.
        let table_view_old_versions = TableView::new().into_raw();
        let table_model_old_versions = StandardItemModel::new(()).into_raw();
        unsafe { table_view_old_versions.as_mut().unwrap().set_model(table_model_old_versions as *mut AbstractItemModel); }
        unsafe { table_view_old_versions.as_mut().unwrap().set_alternating_row_colors(true); };

        // Set it as not editable.
        unsafe { table_view_old_versions.as_mut().unwrap().set_edit_triggers(Flags::from_enum(EditTrigger::NoEditTriggers)); };
        unsafe { table_view_old_versions.as_mut().unwrap().set_selection_mode(SelectionMode::Single); };

        // Sort the versions.
        unsafe { table_view_old_versions.as_mut().unwrap().set_sorting_enabled(true); }
        unsafe { table_view_old_versions.as_mut().unwrap().sort_by_column((0, SortOrder::Ascending)); }

        // Hide the vertical header.
        unsafe { table_view_old_versions.as_mut().unwrap().vertical_header().as_mut().unwrap().set_visible(false); }

        // Prepare it for the Context Menu.
        unsafe { table_view_old_versions.as_mut().unwrap().set_context_menu_policy(ContextMenuPolicy::Custom); }

        // Create the Contextual Menu for the TableView.
        let mut table_view_old_versions_context_menu = Menu::new(());

        // Create the Contextual Menu Actions.
        let table_view_old_versions_context_menu_load = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Load"));
        let table_view_old_versions_context_menu_delete = table_view_old_versions_context_menu.add_action(&QString::from_std_str("&Delete"));

        // Set the shortcuts for these actions.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().db_decoder_definitions["load"]))); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut(&KeySequence::from_string(&QString::from_std_str(&SHORTCUTS.lock().unwrap().db_decoder_definitions["delete"]))); }

        // Set the shortcuts to only trigger in the TableView.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_shortcut_context(ShortcutContext::Widget); }

        // Add them to the TableView.
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_load); }
        unsafe { table_view_old_versions.as_mut().unwrap().add_action(table_view_old_versions_context_menu_delete); }

        // Disable them by default.
        unsafe { table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
        unsafe { table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }

        // Create the bottom ButtonBox.
        let button_box = Frame::new().into_raw();
        let button_box_layout = create_grid_layout_unsafe(button_box as *mut Widget);

        // Create the bottom Buttons.
        let generate_pretty_diff_button = PushButton::new(&QString::from_std_str("Generate Diff")).into_raw();
        let clear_definition_button = PushButton::new(&QString::from_std_str("Remove all fields")).into_raw();
        let save_button = PushButton::new(&QString::from_std_str("Finish it!")).into_raw();

        // Add them to the Dialog.
        unsafe { button_box_layout.as_mut().unwrap().add_widget((generate_pretty_diff_button as *mut Widget, 0, 0, 1, 1)); }
        unsafe { button_box_layout.as_mut().unwrap().add_widget((clear_definition_button as *mut Widget, 0, 1, 1, 1)); }
        unsafe { button_box_layout.as_mut().unwrap().add_widget((save_button as *mut Widget, 0, 2, 1, 1)); }

        // Add everything to the main grid.
        unsafe { widget_layout.as_mut().unwrap().add_widget((hex_view_group as *mut Widget, 0, 0, 5, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((table_view as *mut Widget, 0, 1, 1, 2)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((fields_splitter as *mut Widget, 1, 1, 4, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((info_frame as *mut Widget, 1, 2, 1, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((table_view_old_versions as *mut Widget, 2, 2, 2, 1)); }
        unsafe { widget_layout.as_mut().unwrap().add_widget((button_box as *mut Widget, 4, 2, 1, 1)); }
        unsafe { widget_layout.as_mut().unwrap().set_column_stretch(1, 10); }
        unsafe { widget_layout.as_mut().unwrap().set_row_stretch(0, 10); }
        unsafe { widget_layout.as_mut().unwrap().set_row_stretch(2, 5); }

        //---------------------------------------------------------------------------------------//
        // Prepare the data for the Decoder View...
        //---------------------------------------------------------------------------------------//

        // Get the PackedFile.
        sender_qt.send(Commands::GetPackedFile).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.to_vec())).unwrap();
        let mut packed_file = match check_message_validity_recv2(&receiver_qt) {
            Data::PackedFile(data) => data,
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR),
        };

        // If the PackedFile is in the db folder...
        if packed_file.path.len() == 3 {
            if packed_file.path[0] == "db" {

                // Put all together so we can pass it easely.
                let stuff = PackedFileDBDecoderStuff {
                    hex_view_index,
                    hex_view_raw,
                    hex_view_decoded,
                    table_view,
                    table_model,
                    selection_bool_line_edit,
                    selection_float_line_edit,
                    selection_integer_line_edit,
                    selection_long_integer_line_edit,
                    selection_string_u8_line_edit,
                    selection_string_u16_line_edit,
                    selection_optional_string_u8_line_edit,
                    selection_optional_string_u16_line_edit,
                    bool_line_edit,
                    float_line_edit,
                    integer_line_edit,
                    long_integer_line_edit,
                    string_u8_line_edit,
                    string_u16_line_edit,
                    optional_string_u8_line_edit,
                    optional_string_u16_line_edit,
                    bool_button,
                    float_button,
                    integer_button,
                    long_integer_button,
                    string_u8_button,
                    string_u16_button,
                    optional_string_u8_button,
                    optional_string_u16_button,
                    table_info_type_decoded_label,
                    table_info_version_decoded_label,
                    table_info_entry_count_decoded_label,
                    table_view_old_versions,
                    table_model_old_versions,
                    generate_pretty_diff_button,
                    clear_definition_button,
                    save_button,
                    table_view_context_menu: table_view_context_menu.into_raw(),
                    table_view_context_menu_move_up,
                    table_view_context_menu_move_down,
                    table_view_context_menu_delete,
                    table_view_old_versions_context_menu: table_view_old_versions_context_menu.into_raw(),
                    table_view_old_versions_context_menu_load,
                    table_view_old_versions_context_menu_delete,
                };

                // Check if it can be read as a table.
                let packed_file_data = packed_file.get_data_and_keep_it()?;
                match DB::get_header_data(&packed_file_data) {

                    // If we succeed at decoding his header...
                    Ok((version, entry_count, initial_index)) => {

                        // Put all the "Non UI" data we need to keep together.
                        let stuff_non_ui = PackedFileDBDecoderStuffNonUI {
                            packed_file_path: packed_file.path.to_vec(),
                            packed_file_data,
                            initial_index,
                            version,
                            entry_count,
                        };

                        // Get the index we are going to "manipulate".
                        let index = Rc::new(RefCell::new(stuff_non_ui.initial_index));

                        // Check if we have an schema.
                        let schema = SCHEMA.lock().unwrap().clone();
                        match schema {

                            // If we have an schema...
                            Some(schema) => {

                                // Get the table definition for this table (or create a new one).
                                let table_definition = match DB::get_schema(&stuff_non_ui.packed_file_path[1], stuff_non_ui.version, &schema) {
                                    Some(table_definition) => Rc::new(RefCell::new(table_definition)),
                                    None => Rc::new(RefCell::new(TableDefinition::new(stuff_non_ui.version)))
                                };

                                //---------------------------------------------------------------------------------------//
                                // Load the data to the Decoder View...
                                //---------------------------------------------------------------------------------------//

                                // Load the static data into the Decoder View.
                                Self::load_data_to_decoder_view(&stuff, &stuff_non_ui);

                                // Update the versions list.
                                Self::update_versions_list(&stuff, &schema, &stuff_non_ui.packed_file_path[1]);

                                // Update the Decoder View's Dynamic Data (LineEdits, Table,...) and recalculate
                                // the current "index_data" (position in the vector we are decoding).
                                Self::update_decoder_view(
                                    &stuff, &stuff_non_ui,
                                    (true, &table_definition.borrow().fields),
                                    &mut index.borrow_mut()
                                );

                                // Put the schema into a Rc<RefCell<Schema>>, so we can modify it.
                                let schema = Rc::new(RefCell::new(schema));

                                //---------------------------------------------------------------------------------------//
                                // Create the slots for the decoder view...
                                //---------------------------------------------------------------------------------------//

                                // Create all the slots we need to keep alive later on.
                                let slots = Self {

                                    // Slot to sync all the TextEdit in the "Hex Data" area.
                                    slot_hex_view_scroll_sync: SlotCInt::new(clone!(
                                        stuff => move |value| {
                                            unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                            unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().set_value(value); }
                                        }
                                    )),

                                    // Slot to sync the selection of both HexViews (Raw => Decoded).
                                    slot_hex_view_raw_selection_sync: SlotNoArgs::new(clone!(
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            let cursor_dest;
                                            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }
                                            unsafe { cursor_dest = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let mut selection_start;
                                            let mut selection_end;
                                            unsafe { selection_start = cursor.as_mut().unwrap().selection_start(); }
                                            unsafe { selection_end = cursor.as_mut().unwrap().selection_end(); }

                                            // Translate those limits to fit the other HexView.
                                            selection_start = ((selection_start + 1) / 3) + (selection_start / 48);
                                            selection_end = ((selection_end + 2) / 3) + (selection_end / 48);

                                            // If we got something selected, always select something in the other HexView.
                                            unsafe { if selection_start == selection_end && cursor.as_mut().unwrap().selection_start() != cursor.as_mut().unwrap().selection_end() { selection_end += 1; } }

                                            // Select the corresponding lines in the other HexView.
                                            unsafe { cursor_dest.as_mut().unwrap().move_position(MoveOperation::Start); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Move, selection_start as i32)); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Keep, (selection_end - selection_start) as i32)); }

                                            // Block the signals during this, so we don't trigger an infinite loop.
                                            let mut blocker;
                                            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }
                                            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor_dest.as_ref().unwrap()); }
                                            blocker.unblock();
                                        }
                                    )),

                                    // Slot to sync the selection of both HexViews (Decoded => Raw).
                                    slot_hex_view_decoded_selection_sync: SlotNoArgs::new(clone!(
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            let cursor_dest;
                                            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }
                                            unsafe { cursor_dest = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let mut selection_start;
                                            let mut selection_end;
                                            unsafe { selection_start = cursor.as_mut().unwrap().selection_start(); }
                                            unsafe { selection_end = cursor.as_mut().unwrap().selection_end(); }

                                            // Translate those limits to fit the other HexView.
                                            selection_start = (selection_start - (selection_start / 17)) * 3;
                                            selection_end = (selection_end - (selection_end / 17)) * 3;

                                            // Select the corresponding lines in the other HexView.
                                            unsafe { cursor_dest.as_mut().unwrap().move_position(MoveOperation::Start); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Move, selection_start as i32)); }
                                            unsafe { cursor_dest.as_mut().unwrap().move_position((MoveOperation::NextCharacter, MoveMode::Keep, (selection_end - selection_start) as i32)); }

                                            // Block the signals during this, so we don't trigger an infinite loop.
                                            let mut blocker;
                                            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }
                                            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor_dest.as_ref().unwrap()); }
                                            blocker.unblock();
                                        }
                                    )),

                                    // Slot to get the selected text and decode it on-the-fly from the HexView (Raw).
                                    slot_hex_view_raw_selection_decoding: SlotNoArgs::new(clone!(
                                        stuff_non_ui,
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let selection_start;
                                            unsafe { selection_start = ((cursor.as_mut().unwrap().selection_start()) / 3) as usize; }

                                            // Update the LineEdits.
                                            Self::update_selection_decoded_fields(&stuff, &stuff_non_ui, selection_start);
                                        }
                                    )),

                                    // Slot to get the selected text and decode it on-the-fly from the HexView (Decoded).
                                    slot_hex_view_decoded_selection_decoding: SlotNoArgs::new(clone!(
                                        stuff_non_ui,
                                        stuff => move || {

                                            // Get the cursor of the TextEdit.
                                            let cursor;
                                            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor().into_raw(); }

                                            // Get the limits of the selection.
                                            let selection_start;
                                            unsafe { selection_start = (cursor.as_mut().unwrap().selection_start() - (cursor.as_mut().unwrap().selection_start() / 17)) as usize; }

                                            // Update the LineEdits.
                                            Self::update_selection_decoded_fields(&stuff, &stuff_non_ui, selection_start);
                                        }
                                    )),

                                    // Slots for the "Use this" buttons.
                                    slot_use_this_bool: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Boolean, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_float: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Float, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::Integer, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_long_integer: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::LongInteger, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::StringU16, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u8: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU8, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_use_this_optional_string_u16: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {
                                            Self::use_this(&stuff, &stuff_non_ui, FieldType::OptionalStringU16, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for when we change the Type of the selected field in the table.
                                    slot_table_change_field_type: SlotModelIndexRefModelIndexRefVectorVectorCIntRef::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |initial_model_index,final_model_index,_| {

                                            // If both changed cells are from the Type column...
                                            if initial_model_index.column() == 1 && final_model_index.column() == 1 {

                                                // Update the "First row decoded" column, and get the new "index" to continue decoding.
                                                let invalid_rows = Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());

                                                // Fix the broken rows.
                                                for row in &invalid_rows {

                                                    // Get the item from the type column.
                                                    let item;
                                                    unsafe { item = stuff.table_model.as_mut().unwrap().item((*row, 1)); }

                                                    // Change it to bool.
                                                    unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str("Bool")); }
                                                }
                                            }
                                        }
                                    )),

                                    // Slot to enable/disable contextual actions depending on the selected item.
                                    slot_table_view_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is something selected...
                                            if !selection.indexes().is_empty() {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),

                                    // Slot to show the Contextual Menu for the TableView.
                                    slot_table_view_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),

                                    // Slots for the Contextual Menu Actions of the TableView.
                                    slot_table_view_context_menu_move_up: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move up.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list.
                                            rows.sort();
                                            rows.dedup();

                                            // For each row we have to move...
                                            for row in rows {

                                                // If we are in the row 0, skip it.
                                                if row == 0 { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row - 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_move_down: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to move down.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to move...
                                            for row in rows {

                                                // Get the amount of rows in the Model.
                                                let row_count;
                                                unsafe { row_count = stuff.table_model.as_mut().unwrap().row_count(()); }

                                                // If we are in the last row, skip it.
                                                if row == (row_count - 1) { continue; }

                                                // Otherwise...
                                                else {

                                                    // Take the row from the table.
                                                    let row_data;
                                                    unsafe { row_data = stuff.table_model.as_mut().unwrap().take_row(row + 1); }

                                                    // Insert it one position above.
                                                    unsafe { stuff.table_model.as_mut().unwrap().insert_row((row, &row_data)); }
                                                }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),
                                    slot_table_view_context_menu_delete: SlotBool::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }
                                            let indexes = selection.indexes();

                                            //  Get the list of rows to remove.
                                            let mut rows = vec![];
                                            for index in 0..indexes.count(()) {
                                                rows.push(indexes.at(index).row());
                                            }

                                            // Dedup the list and reverse it.
                                            rows.sort();
                                            rows.dedup();
                                            rows.reverse();

                                            // For each row we have to remove...
                                            for row in rows {

                                                // Remove it.
                                                unsafe { stuff.table_model.as_mut().unwrap().remove_row(row); }
                                            }

                                            // Update the "First row decoded" column.
                                            Self::update_first_row_decoded(&stuff, &stuff_non_ui, &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Generate Pretty Diff" button.
                                    slot_generate_pretty_diff: SlotNoArgs::new(clone!(
                                        sender_qt,
                                        receiver_qt,
                                        app_ui => move || {

                                            // Tell the background thread to generate the diff and wait.
                                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                                            sender_qt.send(Commands::GenerateSchemaDiff).unwrap();
                                            match check_message_validity_tryrecv(&receiver_qt) {
                                                Data::Success => show_dialog(app_ui.window, true, "Diff generated succesfully"),
                                                Data::Error(error) => show_dialog(app_ui.window, false, error),

                                                // In ANY other situation, it's a message problem.
                                                _ => panic!(THREADS_MESSAGE_ERROR),
                                            }
                                            unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                                        }
                                    )),

                                    // Slot for the "Kill them all!" button.
                                    slot_remove_all_fields: SlotNoArgs::new(clone!(
                                        index,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // Remove everything from the model.
                                            unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                            // Reset the index.
                                            *index.borrow_mut() = stuff_non_ui.initial_index;

                                            // Update the decoder view.
                                            Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index.borrow_mut());
                                        }
                                    )),

                                    // Slot for the "Finish it!" button.
                                    slot_save_definition: SlotNoArgs::new(clone!(
                                        sender_qt,
                                        sender_qt_data,
                                        receiver_qt,
                                        table_definition,
                                        schema,
                                        app_ui,
                                        stuff,
                                        stuff_non_ui => move || {

                                            // We get the index of our table's definitions. In case we find it, we just return it. If it's not
                                            // the case, then we create a new table's definitions and return his index. To know if we didn't found
                                            // an index, we just return -1 as index.
                                            let mut table_definitions_index = match schema.borrow().get_table_definitions(&stuff_non_ui.packed_file_path[1]) {
                                                Some(table_definitions_index) => table_definitions_index as i32,
                                                None => -1i32,
                                            };

                                            // If we didn't found a table definition for our table...
                                            if table_definitions_index == -1 {

                                                // We create one.
                                                schema.borrow_mut().add_table_definitions(TableDefinitions::new(&stuff_non_ui.packed_file_path[1]));

                                                // And get his index.
                                                table_definitions_index = schema.borrow().get_table_definitions(&stuff_non_ui.packed_file_path[1]).unwrap() as i32;
                                            }

                                            // We replace his fields with the ones from the TableView.
                                            table_definition.borrow_mut().fields = Self::return_data_from_data_view(&stuff);

                                            // We add our `TableDefinition` to the main `Schema` and sort it, so the TableDefinition is in the right place.
                                            schema.borrow_mut().tables_definitions[table_definitions_index as usize].add_table_definition(table_definition.borrow().clone());
                                            schema.borrow_mut().tables_definitions.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                                            schema.borrow_mut().tables_definitions.iter_mut().for_each(|x| x.versions.sort_unstable_by(|a, b| b.version.cmp(&a.version)));

                                            // Send it back to the background thread for saving it.
                                            sender_qt.send(Commands::SaveSchema).unwrap();
                                            sender_qt_data.send(Data::Schema(schema.borrow().clone())).unwrap();

                                            // Report success while saving it, or an error.
                                            match check_message_validity_recv2(&receiver_qt) {
                                                Data::Success => show_dialog(app_ui.window, true, "Schema successfully saved."),
                                                Data::Error(error) => show_dialog(app_ui.window, false, error),
                                                _ => panic!(THREADS_MESSAGE_ERROR),
                                            }

                                            // After all that, we need to update the version list, as this may have created a new version.
                                            Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file_path[1]);
                                        }
                                    )),

                                    // Actions to manage the Context Menu in the "Versions" TableView.
                                    slot_table_view_old_versions_context_menu_enabler: SlotItemSelectionRefItemSelectionRef::new(clone!(
                                        stuff => move |selection,_| {

                                            // If there is only one version selected...
                                            if selection.indexes().count(()) == 1 {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(true); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(true); }
                                            }

                                            // Otherwise, disable everything.
                                            else {
                                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().set_enabled(false); }
                                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().set_enabled(false); }
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu: SlotQtCorePointRef::new(clone!(
                                        stuff => move |_| {
                                            unsafe { stuff.table_view_old_versions_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_load: SlotBool::new(clone!(
                                        schema,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<i32>().unwrap();

                                                // Get the new definition.
                                                let table_definition = DB::get_schema(&stuff_non_ui.packed_file_path[1], version, &*schema.borrow());

                                                // Remove everything from the model.
                                                unsafe { stuff.table_model.as_mut().unwrap().clear(); }

                                                // Reset the index.
                                                *index.borrow_mut() = stuff_non_ui.initial_index;

                                                // Update the decoder view.
                                                Self::update_decoder_view(&stuff, &stuff_non_ui, (true, &table_definition.unwrap().fields), &mut index.borrow_mut());
                                            }
                                        }
                                    )),
                                    slot_table_view_old_versions_context_menu_delete: SlotBool::new(clone!(
                                        schema,
                                        app_ui,
                                        stuff,
                                        stuff_non_ui => move |_| {

                                            // Get the selection of the TableView.
                                            let selection;
                                            unsafe { selection = stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_mut().unwrap().selection(); }

                                            // If we have something selected...
                                            if selection.indexes().count(()) == 1 {

                                                // Get the selected ModelIndex.
                                                let indexes = selection.indexes();
                                                let model_index = indexes.at(0);

                                                // Get the version selected.
                                                let version;
                                                unsafe { version = stuff.table_model_old_versions.as_mut().unwrap().item_from_index(&model_index).as_mut().unwrap().text().to_std_string(); }

                                                // Turn it into a number.
                                                let version = version.parse::<i32>().unwrap();

                                                // Try to remove that version form the schema.
                                                if let Err(error) = DB::remove_table_version(&stuff_non_ui.packed_file_path[1], version, &mut schema.borrow_mut()) {
                                                    return show_dialog(app_ui.window, false, error.kind());
                                                }

                                                // If it worked, update the list.
                                                Self::update_versions_list(&stuff, &schema.borrow(), &stuff_non_ui.packed_file_path[1]);
                                            }
                                        }
                                    )),
                                };

                                // Sync the scroll bars of the three hex data views.
                                unsafe { stuff.hex_view_index.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().vertical_scroll_bar().as_mut().unwrap().signals().value_changed().connect(&slots.slot_hex_view_scroll_sync); }

                                // Decode on-the-fly whatever is selected in the HexView.
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_raw_selection_decoding); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_decoded_selection_decoding); }

                                // Signal to sync the selection between both HexViews.
                                unsafe { stuff.hex_view_raw.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_raw_selection_sync); }
                                unsafe { stuff.hex_view_decoded.as_mut().unwrap().signals().selection_changed().connect(&slots.slot_hex_view_decoded_selection_sync); }

                                // Actions for the "Use this" buttons.
                                unsafe { stuff.bool_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_bool); }
                                unsafe { stuff.float_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_float); }
                                unsafe { stuff.integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_integer); }
                                unsafe { stuff.long_integer_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_long_integer); }
                                unsafe { stuff.string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u8); }
                                unsafe { stuff.string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_string_u16); }
                                unsafe { stuff.optional_string_u8_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u8); }
                                unsafe { stuff.optional_string_u16_button.as_mut().unwrap().signals().released().connect(&slots.slot_use_this_optional_string_u16); }

                                // Action for when we change the type of a field in the table.
                                unsafe { stuff.table_model.as_mut().unwrap().signals().data_changed().connect(&slots.slot_table_change_field_type); }

                                // Trigger the "Enable/Disable" slot every time we change the selection in the TableView.
                                unsafe { stuff.table_view.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_context_menu_enabler); }

                                // Action to show the Contextual Menu for the Field's TableView.
                                unsafe { (stuff.table_view as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_context_menu); }

                                // Actions of the Contextual Menu for the Field's TableView.
                                unsafe { stuff.table_view_context_menu_move_up.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_up); }
                                unsafe { stuff.table_view_context_menu_move_down.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_move_down); }
                                unsafe { stuff.table_view_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_context_menu_delete); }

                                // Actions for the bottom buttons.
                                unsafe { stuff.generate_pretty_diff_button.as_mut().unwrap().signals().released().connect(&slots.slot_generate_pretty_diff); }
                                unsafe { stuff.clear_definition_button.as_mut().unwrap().signals().released().connect(&slots.slot_remove_all_fields); }
                                unsafe { stuff.save_button.as_mut().unwrap().signals().released().connect(&slots.slot_save_definition); }

                                // Actions for the Contextual Menu in the "Versions" table.
                                unsafe { stuff.table_view_old_versions.as_mut().unwrap().selection_model().as_ref().unwrap().signals().selection_changed().connect(&slots.slot_table_view_old_versions_context_menu_enabler); }
                                unsafe { (stuff.table_view_old_versions as *mut Widget).as_ref().unwrap().signals().custom_context_menu_requested().connect(&slots.slot_table_view_old_versions_context_menu); }
                                unsafe { stuff.table_view_old_versions_context_menu_load.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_load); }
                                unsafe { stuff.table_view_old_versions_context_menu_delete.as_mut().unwrap().signals().triggered().connect(&slots.slot_table_view_old_versions_context_menu_delete); }

                                // Return the slots and the font.
                                Ok((slots, monospace_font))
                            }

                            // If there is no schema, return error.
                            None => Err(ErrorKind::SchemaNotFound)?
                        }
                    },

                    // If it fails, return error.
                    Err(error) => Err(error)
                }
            }

            // Otherwise, return error.
            else { Err(ErrorKind::DBTableIsNotADBTable)? }
        }

        // Otherwise, return error.
        else { Err(ErrorKind::DBTableIsNotADBTable)? }
    }

    /// This function is meant to load the static data of a DB PackedFile into the decoder, or return error.
    pub fn load_data_to_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
    ) {
        // Get the FontMetrics for the size stuff.
        let font;
        unsafe { font = stuff.hex_view_index.as_mut().unwrap().document().as_mut().unwrap().default_font(); }
        let font_metrics = FontMetrics::new(&font);

        // We don't need the entire PackedFile, just his begining. Otherwise, this function takes ages to finish.
        let data_reduced = if stuff_non_ui.packed_file_data.len() > 16 * 60 { &stuff_non_ui.packed_file_data[..16 * 60] }
        else { &stuff_non_ui.packed_file_data };

        // This creates the "index" column at the left of the hex data. The logic behind this, because
        // even I have problems to understand it: lines are 4 packs of 4 bytes => 16 bytes. Amount of
        // lines is "bytes we have / 16 + 1" (+ 1 because we want to show incomplete lines too).
        // Then, for the zeroes, we default to 4.
        let mut hex_lines_text = String::new();
        let hex_lines_amount = (data_reduced.len() / 16) + 1;
        for hex_line in 0..hex_lines_amount { hex_lines_text.push_str(&format!("{:>0count$X}\n", hex_line * 16, count = 4)); }

        // Add all the "Index" lines to the TextEdit.
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_html(&QString::from_std_str(&hex_lines_text)); }

        // Resize the TextEdit.
        let text_size = font_metrics.size((0, &QString::from_std_str(hex_lines_text)));
        unsafe { stuff.hex_view_index.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

        // This gets the hex data into place. In big files, this takes ages, so we cut them if they
        // are longer than 100 lines to speed up loading and fix a weird issue with big tables.
        let mut hex_raw_data = format!("{:02X?}", data_reduced);

        // Remove the first and last chars.
        hex_raw_data.remove(0);
        hex_raw_data.pop();

        // Remove all the kebab, or the commas. Whatever float your boat...
        hex_raw_data.retain(|c| c != ',');

        // `hex_view_raw` TextEdit.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_raw_string = String::new();

            // This pushes a newline after 48 characters (2 for every byte + 1 whitespace).
            for (j, i) in hex_raw_data.chars().enumerate() {

                // Also. replace the last whitespace of each line with a \n.
                if j % 48 == 0 && j != 0 { hex_view_raw_string.pop(); hex_view_raw_string.push_str("\n"); }
                hex_view_raw_string.push(i);
            }

            // Add all the "Raw" lines to the TextEdit.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text(&QString::from_std_str(&hex_view_raw_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_raw_string)));
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

            // Prepare the format for the header.
            let mut header_format = TextCharFormat::new();
            header_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkRed } else { GlobalColor::Red }));

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the header.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index * 3) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&header_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // `hex_view_decoded` TextView.
        {
            // Create the String to pass to the TextEdit.
            let mut hex_view_decoded_string = String::new();

            // This pushes a newline after 16 characters.
            for (j, i) in data_reduced.iter().enumerate() {
                if j % 16 == 0 && j != 0 { hex_view_decoded_string.push_str("\n"); }
                let character = *i as char;

                // If is a valid UTF-8 char, show it. Otherwise, default to '.'.
                if character.is_alphanumeric() { hex_view_decoded_string.push(character); }
                else { hex_view_decoded_string.push('.'); }
            }

            // Add all the "Decoded" lines to the TextEdit.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text(&QString::from_std_str(&hex_view_decoded_string)); }

            // Resize the TextEdit.
            let text_size = font_metrics.size((0, &QString::from_std_str(hex_view_decoded_string)));
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_fixed_width(text_size.width() + 34); }

            // Prepare the format for the header.
            let mut header_format = TextCharFormat::new();
            header_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkRed } else { GlobalColor::Red }));

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the header. We need to add 1 char per line to this.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&header_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Load the "Info" data to the view.
        unsafe { stuff.table_info_type_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(&stuff_non_ui.packed_file_path[1])); }
        unsafe { stuff.table_info_version_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.version))); }
        unsafe { stuff.table_info_entry_count_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.entry_count))); }
    }

    /// This function is used to update the list of "Versions" of the currently open table decoded.
    pub fn update_versions_list(
        stuff: &PackedFileDBDecoderStuff,
        schema: &Schema,
        table_name: &str,
    ) {
        // Clear the current list.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().clear(); }

        // And get all the versions of this table, and list them in their TreeView, if we have any.
        if let Some(table_versions_list) = DB::get_schema_versions_list(table_name, schema) {
            for version in table_versions_list {
                let item = StandardItem::new(&QString::from_std_str(format!("{}", version.version)));
                unsafe { stuff.table_model_old_versions.as_mut().unwrap().append_row_unsafe(item.into_raw()); }
            }
        }

        // Set the title of the column.
        unsafe { stuff.table_model_old_versions.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Versions Decoded")))); }

        // Extend the column to fill the Table.
        unsafe { stuff.table_view_old_versions.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_section_resize_mode(ResizeMode::Stretch); }
    }

    /// This function updates the data shown in the decoder view when we execute it. `index_data`
    /// is the position from where to start decoding. In field_list the boolean is true for the first load.
    /// Otherwise, always pass false there.
    pub fn update_decoder_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_list: (bool, &[Field]),
        mut index_data: &mut usize,
    ) {

        // Create the variables to hold the values we'll pass to the LineEdits.
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // If we are loading data to the table for the first time, we'll load to the table all the data
        // directly from the existing definition and update the initial index for decoding.
        // If there is no data in the definition, we'll add an empty row and delete it, so the table remembers the
        // columns data.
        if field_list.0 {
            if field_list.1.is_empty() {

                let mut qlist = ListStandardItemMutPtr::new(());
                (0..7).for_each(|_| unsafe { qlist.append_unsafe(&StandardItem::new(()).into_raw()) });
                unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Name")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Type")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Is key?")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Table")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((4, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Column")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((5, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("First Row Decoded")))); }
                unsafe { stuff.table_model.as_mut().unwrap().set_header_data((6, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Description")))); }
                unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
                unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
                unsafe { stuff.table_model.as_mut().unwrap().remove_rows((0, 1)); }
            }

            else {
                for field in field_list.1 {
                    Self::add_field_to_data_view(
                        &stuff,
                        &stuff_non_ui,
                        &field.field_name,
                        field.field_type,
                        field.field_is_key,
                        &field.field_is_reference,
                        &field.field_description,
                        &mut index_data,
                    );
                }
            }
        }

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(stuff_non_ui.packed_file_data[*index_data], &mut index_data.clone()) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 4)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if stuff_non_ui.packed_file_data.get(*index_data + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&stuff_non_ui.packed_file_data[*index_data..(*index_data + 8)], &mut index_data.clone()) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(*index_data + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&stuff_non_ui.packed_file_data[*index_data..], &mut index_data.clone()) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        unsafe { stuff.bool_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_bool)); }
        unsafe { stuff.float_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_float)); }
        unsafe { stuff.integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_integer)); }
        unsafe { stuff.long_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_long_integer)); }
        unsafe { stuff.string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8))); }
        unsafe { stuff.string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16))); }
        unsafe { stuff.optional_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8))); }
        unsafe { stuff.optional_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16))); }

        // Prepare the format for the cleaning.
        let mut neutral_format = TextCharFormat::new();
        neutral_format.set_background(&Brush::new(GlobalColor::Transparent));

        // Prepare the format for the decoded row.
        let mut decoded_format = TextCharFormat::new();
        decoded_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkYellow } else { GlobalColor::Yellow }));

        // Prepare the format for the current index.
        let mut index_format = TextCharFormat::new();
        index_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkMagenta } else { GlobalColor::Magenta }));

        // Clean both TextEdits.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the rest of the data.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index * 3) as i32));
            cursor.move_position((MoveOperation::End, MoveMode::Keep));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&neutral_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the rest of the data.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));
            cursor.move_position((MoveOperation::End, MoveMode::Keep));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&neutral_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Paint both decoded rows.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index * 3) as i32));
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, ((*index_data - stuff_non_ui.initial_index) * 3) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&decoded_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            let positions_to_move_end = *index_data / 16;
            let positions_to_move_start = stuff_non_ui.initial_index / 16;
            let positions_to_move_vertical = positions_to_move_end - positions_to_move_start;
            let positions_to_move_horizontal = *index_data - stuff_non_ui.initial_index;
            let positions_to_move = positions_to_move_horizontal + positions_to_move_vertical;

            cursor.move_position(MoveOperation::Start);
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Move, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, positions_to_move as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&decoded_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }

        // Paint both current index.
        {

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_raw.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, 3));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_raw.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_current_char_format(&index_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_raw.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();

            // Get the cursor.
            let mut cursor;
            unsafe { cursor = stuff.hex_view_decoded.as_mut().unwrap().text_cursor(); }

            // Create the "Selection" for the decoded row.
            cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, 1));

            // Block the signals during this, so we don't mess things up.
            let mut blocker;
            unsafe { blocker = SignalBlocker::new(stuff.hex_view_decoded.as_mut().unwrap().static_cast_mut() as &mut Object); }

            // Set the cursor and his format.
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_current_char_format(&index_format); }

            // Clear the selection.
            cursor.clear_selection();
            unsafe { stuff.hex_view_decoded.as_mut().unwrap().set_text_cursor(&cursor); }

            // Unblock the signals.
            blocker.unblock();
        }
    }

    /// This function adds fields to the decoder's table, so we can do this without depending on the
    /// updates of the decoder's view. As this has a lot of required data, lets's explain the weirdest ones:
    /// - index_data: the index to start decoding from the vector.
    /// - index_row: the position in the row. None to calculate the last position's number.
    pub fn add_field_to_data_view(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_name: &str,
        field_type: FieldType,
        field_is_key: bool,
        field_is_reference: &Option<(String, String)>,
        field_description: &str,
        mut index_data: &mut usize,
    ) {

        // Decode the data from the field.
        let decoded_data = Self::decode_data_by_fieldtype(
            &stuff_non_ui.packed_file_data,
            field_type,
            &mut index_data
        );

        // Get the type of the data we are going to put into the Table.
        let field_type = match field_type {
            FieldType::Boolean => "Bool",
            FieldType::Float => "Float",
            FieldType::Integer => "Integer",
            FieldType::LongInteger => "LongInteger",
            FieldType::StringU8 => "StringU8",
            FieldType::StringU16 => "StringU16",
            FieldType::OptionalStringU8 => "OptionalStringU8",
            FieldType::OptionalStringU16 => "OptionalStringU16",
        };

        // If the field has a reference...
        if let Some(ref reference) = *field_is_reference {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(&reference.0));
            let reference_field = StandardItem::new(&QString::from_std_str(&reference.1));
            let mut decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // The "Decoded First Row" column should not be editable.
            decoded_data.set_editable(false);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Otherwise, we pass an empty reference.
        else {

            // Create a new list of StandardItem.
            let mut qlist = ListStandardItemMutPtr::new(());

            // Create the items of the new row.
            let field_name = StandardItem::new(&QString::from_std_str(field_name));
            let field_type = StandardItem::new(&QString::from_std_str(field_type));
            let mut field_is_key_item = StandardItem::new(());
            field_is_key_item.set_editable(false);
            field_is_key_item.set_checkable(true);
            field_is_key_item.set_check_state(if field_is_key { CheckState::Checked } else { CheckState::Unchecked });
            let reference_table = StandardItem::new(&QString::from_std_str(""));
            let reference_field = StandardItem::new(&QString::from_std_str(""));
            let mut decoded_data = StandardItem::new(&QString::from_std_str(&decoded_data));
            let field_description = StandardItem::new(&QString::from_std_str(field_description));

            // The "Decoded First Row" column should not be editable.
            decoded_data.set_editable(false);

            // Add the items to the list.
            unsafe { qlist.append_unsafe(&field_name.into_raw()); }
            unsafe { qlist.append_unsafe(&field_type.into_raw()); }
            unsafe { qlist.append_unsafe(&field_is_key_item.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_table.into_raw()); }
            unsafe { qlist.append_unsafe(&reference_field.into_raw()); }
            unsafe { qlist.append_unsafe(&decoded_data.into_raw()); }
            unsafe { qlist.append_unsafe(&field_description.into_raw()); }

            // Just append a new row.
            unsafe { stuff.table_model.as_mut().unwrap().append_row(&qlist); }
        }

        // Set the title of the columns and extend them, just in case is needed.
        unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().resize_sections(ResizeMode::ResizeToContents); }
        unsafe { stuff.table_view.as_mut().unwrap().horizontal_header().as_mut().unwrap().set_stretch_last_section(true); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((0, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Name")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((1, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Field Type")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((2, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Is key?")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((3, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Table")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((4, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Ref. to Column")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((5, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("First Row Decoded")))); }
        unsafe { stuff.table_model.as_mut().unwrap().set_header_data((6, Orientation::Horizontal, &Variant::new0(&QString::from_std_str("Description")))); }

        // The second field should be a combobox.
        let mut list = StringList::new(());
        list.append(&QString::from_std_str("Bool"));
        list.append(&QString::from_std_str("Float"));
        list.append(&QString::from_std_str("Integer"));
        list.append(&QString::from_std_str("LongInteger"));
        list.append(&QString::from_std_str("StringU8"));
        list.append(&QString::from_std_str("StringU16"));
        list.append(&QString::from_std_str("OptionalStringU8"));
        list.append(&QString::from_std_str("OptionalStringU16"));
        let list: *mut StringList = &mut list;
        unsafe { qt_custom_stuff::new_combobox_item_delegate(stuff.table_view as *mut Object, 1, list as *const StringList, false)};
    }

    /// This function is a helper to try to decode data in different formats, returning "Error" in case
    /// of decoding error. It requires the FieldType we want to decode, the data we want to decode
    /// (vec<u8>, being the first u8 the first byte to decode) and the index of the data in the Vec<u8>.
    fn decode_data_by_fieldtype(
        field_data: &[u8],
        field_type: FieldType,
        mut index_data: &mut usize
    ) -> String {

        // Try to decode the field, depending on what type that field is.
        match field_type {

            // If the field is a boolean...
            FieldType::Boolean => {

                // Check if the index does even exist, to avoid crashes.
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_bool(field_data[*index_data], &mut index_data) {
                        Ok(result) => {
                            if result { "True".to_string() }
                            else { "False".to_string() }
                        }
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Float => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_float_f32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::Integer => {
                if field_data.get(*index_data + 3).is_some() {
                    match coding_helpers::decode_packedfile_integer_i32(&field_data[*index_data..(*index_data + 4)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::LongInteger => {
                if field_data.get(*index_data + 7).is_some() {
                    match coding_helpers::decode_packedfile_integer_i64(&field_data[*index_data..(*index_data + 8)], &mut index_data) {
                        Ok(result) => result.to_string(),
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU8 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::StringU16 => {
                if field_data.get(*index_data + 1).is_some() {
                    match coding_helpers::decode_packedfile_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU8 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u8(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
            FieldType::OptionalStringU16 => {
                if field_data.get(*index_data).is_some() {
                    match coding_helpers::decode_packedfile_optional_string_u16(&field_data[*index_data..], &mut index_data) {
                        Ok(result) => result,
                        Err(_) => "Error".to_owned(),
                    }
                }
                else { "Error".to_owned() }
            },
        }
    }

    /// This function is used to update the decoder view when we try to add a new field to
    /// the schema with one of the "Use this" buttons.
    pub fn use_this(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        field_type: FieldType,
        mut index_data: &mut usize,
    ) {

        // Try to add the field, and update the index with it.
        Self::add_field_to_data_view(
            &stuff,
            &stuff_non_ui,
            "new_field",
            field_type,
            false,
            &None,
            "",
            &mut index_data,
        );

        // Update all the dynamic data of the "Decoder" view.
        Self::update_decoder_view(
            &stuff,
            &stuff_non_ui,
            (false, &[]),
            &mut index_data,
        );
    }

    /// This function updates the "First row decoded" column in the decoder view, the current index and
    /// the decoded entries. This should be called in row changes (deletion and moving, not adding).
    fn update_first_row_decoded(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        mut index: &mut usize,
    ) -> Vec<i32> {

        // Create the list of "invalid" types.
        let mut invalid_types = vec![];

        // Reset the index.
        *index = stuff_non_ui.initial_index;

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get the ModelIndex of the cell we want to update.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 5)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the row's type.
                let row_type;
                unsafe { row_type = stuff.table_model.as_mut().unwrap().index((row, 1)); }

                // Get the field's type.
                let field_type = match &*row_type.data(0).to_string().to_std_string() {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" => FieldType::OptionalStringU16,

                    // In case of invalid type, we add it to the list and set it as bool.
                    _ => {

                        // Add the row to the list.
                        invalid_types.push(row);

                        // Return a boolean.
                        FieldType::Boolean
                    }
                };

                // Get the decoded data using it's type...
                let decoded_data = Self::decode_data_by_fieldtype(
                    &stuff_non_ui.packed_file_data,
                    field_type,
                    &mut index
                );

                // Get the item from the "First Row Decoded" column.
                let item;
                unsafe { item = stuff.table_model.as_mut().unwrap().item_from_index(&model_index); }

                // Change it to our decoded data.
                unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(decoded_data)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Update the entire decoder to use the new index.
        Self::update_decoder_view(&stuff, &stuff_non_ui, (false, &[]), &mut index);

        // Return the list of broken rows.
        invalid_types
    }

    /// This function gets the data from the decoder's table, and returns it, so we can save it in a TableDefinition.
    pub fn return_data_from_data_view(
        stuff: &PackedFileDBDecoderStuff
    ) -> Vec<Field> {

        // Create the field's vector.
        let mut fields = vec![];

        // Get the first row.
        let mut row = 0;

        // Loop through all the rows.
        loop {

            // Get a ModelIndex from the row.
            let model_index;
            unsafe { model_index = stuff.table_model.as_mut().unwrap().index((row, 0)); }

            // If it's valid (exists)...
            if model_index.is_valid() {

                // Get the data from each field of the row...
                let field_name;
                let field_type;
                let field_is_key;
                let ref_table;
                let ref_column;
                let field_description;

                unsafe { field_name = stuff.table_model.as_mut().unwrap().item((row, 0)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_type = stuff.table_model.as_mut().unwrap().item((row, 1)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_is_key = if stuff.table_model.as_mut().unwrap().item((row, 2)).as_mut().unwrap().check_state() == CheckState::Checked { true } else { false }; }
                unsafe { ref_table = stuff.table_model.as_mut().unwrap().item((row, 3)).as_mut().unwrap().text().to_std_string(); }
                unsafe { ref_column = stuff.table_model.as_mut().unwrap().item((row, 4)).as_mut().unwrap().text().to_std_string(); }
                unsafe { field_description = stuff.table_model.as_mut().unwrap().item((row, 6)).as_mut().unwrap().text().to_std_string(); }

                // Get the proper type of the field. If invalid, default to OptionalStringU16.
                let field_type = match &*field_type {
                    "Bool" => FieldType::Boolean,
                    "Float" => FieldType::Float,
                    "Integer" => FieldType::Integer,
                    "LongInteger" => FieldType::LongInteger,
                    "StringU8" => FieldType::StringU8,
                    "StringU16" => FieldType::StringU16,
                    "OptionalStringU8" => FieldType::OptionalStringU8,
                    "OptionalStringU16" | _=> FieldType::OptionalStringU16,
                };

                // If there is no table referenced...
                if ref_table.is_empty() { fields.push(Field::new(field_name, field_type, field_is_key, None, field_description)); }

                // Otherwise...
                else { fields.push(Field::new(field_name, field_type, field_is_key, Some((ref_table, ref_column)), field_description)); }

                // Increase the row.
                row += 1;
            }

            // Otherwise, stop the loop.
            else { break; }
        }

        // Return the fields.
        fields
    }

    /// This function updates the "selection" fields when the selection of a HexView is changed.
    fn update_selection_decoded_fields(
        stuff: &PackedFileDBDecoderStuff,
        stuff_non_ui: &PackedFileDBDecoderStuffNonUI,
        mut selection_start: usize
    ) {

        // Create the variables to hold the values we'll pass to the LineEdits.
        let decoded_bool;
        let decoded_float;
        let decoded_integer;
        let decoded_long_integer;
        let decoded_string_u8;
        let decoded_string_u16;
        let decoded_optional_string_u8;
        let decoded_optional_string_u16;

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start).is_some() {
            decoded_bool = match coding_helpers::decode_packedfile_bool(stuff_non_ui.packed_file_data[selection_start], &mut selection_start) {
                Ok(data) => if data { "True" } else { "False" },
                Err(_) => "Error"
            };

            decoded_optional_string_u8 = match coding_helpers::decode_packedfile_optional_string_u8(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_optional_string_u16 = match coding_helpers::decode_packedfile_optional_string_u16(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_bool = "Error";
            decoded_optional_string_u8 = "Error".to_owned();
            decoded_optional_string_u16 = "Error".to_owned();
        };

        // Check if the index does even exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start + 3).is_some() {
            decoded_float = match coding_helpers::decode_packedfile_float_f32(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 4)], &mut selection_start) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned(),
            };

            decoded_integer = match coding_helpers::decode_packedfile_integer_i32(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 4)], &mut selection_start) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_float = "Error".to_owned();
            decoded_integer = "Error".to_owned();
        }

        // Check if the index does even exist, to avoid crashes.
        decoded_long_integer = if stuff_non_ui.packed_file_data.get(selection_start + 7).is_some() {
            match coding_helpers::decode_packedfile_integer_i64(&stuff_non_ui.packed_file_data[selection_start..(selection_start + 8)], &mut selection_start) {
                Ok(data) => data.to_string(),
                Err(_) => "Error".to_owned()
            }
        }
        else { "Error".to_owned() };

        // Check that the index exist, to avoid crashes.
        if stuff_non_ui.packed_file_data.get(selection_start + 1).is_some() {
            decoded_string_u8 = match coding_helpers::decode_packedfile_string_u8(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };

            decoded_string_u16 = match coding_helpers::decode_packedfile_string_u16(&stuff_non_ui.packed_file_data[selection_start..], &mut selection_start) {
                Ok(data) => data,
                Err(_) => "Error".to_owned()
            };
        }
        else {
            decoded_string_u8 = "Error".to_owned();
            decoded_string_u16 = "Error".to_owned();
        }

        // We update all the decoded entries here.
        unsafe { stuff.selection_bool_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_bool)); }
        unsafe { stuff.selection_float_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_float)); }
        unsafe { stuff.selection_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_integer)); }
        unsafe { stuff.selection_long_integer_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(decoded_long_integer)); }
        unsafe { stuff.selection_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u8))); }
        unsafe { stuff.selection_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_string_u16))); }
        unsafe { stuff.selection_optional_string_u8_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u8))); }
        unsafe { stuff.selection_optional_string_u16_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(&format!("{:?}", decoded_optional_string_u16))); }
    }
}
