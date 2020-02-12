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

use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
use qt_widgets::text_edit::TextEdit;
use qt_widgets::widget::Widget;

use qt_gui::font_metrics::FontMetrics;
use qt_gui::text_char_format::TextCharFormat;
use qt_gui::text_cursor::{MoveOperation, MoveMode};

use qt_core::flags::Flags;
use qt_core::object::Object;
use qt_core::qt::{AlignmentFlag, GlobalColor};
use qt_core::signal_blocker::SignalBlocker;

use cpp_utils::StaticCast;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, atomic::{AtomicPtr, Ordering}};

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::SETTINGS;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::FONT_MONOSPACE;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{PackedFileView, TheOneSlot, View};
use crate::QString;
use crate::utils::create_grid_layout_unsafe;
use self::slots::PackedFileDecoderViewSlots;

pub mod connections;
pub mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackedFile Decoder.
pub struct PackedFileDecoderView {
    hex_view_index: AtomicPtr<TextEdit>,
    hex_view_raw: AtomicPtr<TextEdit>,
    hex_view_decoded: AtomicPtr<TextEdit>,
    //table_view: AtomicPtr<TableView>,
    //table_model: AtomicPtr<StandardItemModel>,
    packed_file_data: Arc<Vec<u8>>,
}

/// This struct contains the raw version of each pointer in `PackedFileDecoderViewRaw`, to be used when building the slots.
///
/// This is kinda a hack, because AtomicPtr cannot be copied, and we need a copy of the entire set of pointers available
/// for the construction of the slots. So we build this one, copy it for the slots, then move it into the `PackedFileDecoderView`.
#[derive(Clone, Copy)]
pub struct PackedFileDecoderViewRaw {
    pub hex_view_index: *mut TextEdit,
    pub hex_view_raw: *mut TextEdit,
    pub hex_view_decoded: *mut TextEdit,
    //pub table_view: *mut TableView,
    //pub table_model: *mut StandardItemModel,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileDecoderView`.
impl PackedFileDecoderView {

    /// This function creates a new Decoder View, and sets up his slots and connections.
    pub fn new_view(
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

        // Create the hex view on the left side.
        let layout = unsafe { (packed_file_view.get_mut_widget().as_mut().unwrap().layout() as *mut GridLayout).as_mut().unwrap() };

        let hex_view_group = GroupBox::new(&QString::from_std_str("PackedFile's Data"));
        let mut hex_view_index = TextEdit::new(());
        let mut hex_view_raw = TextEdit::new(());
        let mut hex_view_decoded = TextEdit::new(());
        let hex_view_layout = unsafe { create_grid_layout_unsafe(hex_view_group.as_mut_ptr() as *mut Widget).as_mut().unwrap() };

        hex_view_index.set_font(&FONT_MONOSPACE);
        hex_view_raw.set_font(&FONT_MONOSPACE);
        hex_view_decoded.set_font(&FONT_MONOSPACE);

        unsafe { hex_view_layout.add_widget((hex_view_index.as_mut_ptr() as *mut Widget, 0, 0, 1, 1)); }
        unsafe { hex_view_layout.add_widget((hex_view_raw.as_mut_ptr() as *mut Widget, 0, 1, 1, 1)); }
        unsafe { hex_view_layout.add_widget((hex_view_decoded.as_mut_ptr() as *mut Widget, 0, 2, 1, 1)); }

        unsafe { layout.add_widget((hex_view_group.into_raw() as *mut Widget, 0, 2, 1, 1)); }

        let packed_file_decoder_view_raw = PackedFileDecoderViewRaw {
            hex_view_index: hex_view_index.into_raw(),
            hex_view_raw: hex_view_raw.into_raw(),
            hex_view_decoded: hex_view_decoded.into_raw()
        };

        let packed_file_decoder_view_slots = PackedFileDecoderViewSlots::new(
            packed_file_decoder_view_raw,
            *pack_file_contents_ui,
            *global_search_ui,
            &packed_file_path
        );

        let packed_file_decoder_view = Self {
            hex_view_index: AtomicPtr::new(packed_file_decoder_view_raw.hex_view_index),
            hex_view_raw: AtomicPtr::new(packed_file_decoder_view_raw.hex_view_raw),
            hex_view_decoded: AtomicPtr::new(packed_file_decoder_view_raw.hex_view_decoded),
            packed_file_data: Arc::new(packed_file.get_raw_data()?)
        };

        packed_file_decoder_view.load_raw_data();
        connections::set_connections(&packed_file_decoder_view, &packed_file_decoder_view_slots);
        packed_file_view.view = View::Decoder(packed_file_decoder_view);

        // Return success.
        Ok(TheOneSlot::Decoder(packed_file_decoder_view_slots))
    }

    /// This function loads the raw data of a PackedFile into the UI and prepare it to be updated later on.
    pub fn load_raw_data(&self) {

        // We need to set up the fonts in a specific way, so the scroll/sizes are kept correct.
        let font = unsafe { self.get_ref_hex_view_index().document().as_mut().unwrap().default_font() };
        let font_metrics = FontMetrics::new(&font);

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
        let text_size = font_metrics.size((0, &qhex_index));
        self.get_ref_mut_hex_view_index().set_text(&qhex_index);
        self.get_ref_mut_hex_view_index().set_fixed_width(text_size.width() + 34);

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
        let text_size = font_metrics.size((0, &qhex_raw_data));
        self.get_ref_mut_hex_view_raw().set_text(&qhex_raw_data);
        self.get_ref_mut_hex_view_raw().set_fixed_width(text_size.width() + 34);

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
        let text_size = font_metrics.size((0, &qhex_decoded_data));
        self.get_ref_mut_hex_view_decoded().set_text(&qhex_decoded_data);
        self.get_ref_mut_hex_view_decoded().set_fixed_width(text_size.width() + 34);

/*
            // Prepare the format for the header.
            let mut header_format = TextCharFormat::new();
            //header_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkRed } else { GlobalColor::Red }));

            // Get the cursor.
            let mut cursor = self.get_ref_mut_hex_view_raw().text_cursor();

            // Create the "Selection" for the header.
            cursor.move_position(MoveOperation::Start);
            //cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index * 3) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker = SignalBlocker::new(self.get_ref_mut_hex_view_raw().static_cast_mut() as &mut Object);

            // Set the cursor and his format.
            self.get_ref_mut_hex_view_raw().set_text_cursor(&cursor);
            self.get_ref_mut_hex_view_raw().set_current_char_format(&header_format);

            // Clear the selection.
            cursor.clear_selection();
            self.get_ref_mut_hex_view_raw().set_text_cursor(&cursor);

            // Unblock the signals.
            blocker.unblock();*/



            // Prepare the format for the header.
            //let mut header_format = TextCharFormat::new();
            //header_format.set_background(&Brush::new(if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkRed } else { GlobalColor::Red }));
/*
            // Get the cursor.
            let mut cursor = self.get_ref_mut_hex_view_decoded().text_cursor();

            // Create the "Selection" for the header. We need to add 1 char per line to this.
            cursor.move_position(MoveOperation::Start);
            //cursor.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (stuff_non_ui.initial_index + (stuff_non_ui.initial_index as f32 / 16.0).floor() as usize) as i32));

            // Block the signals during this, so we don't mess things up.
            let mut blocker = SignalBlocker::new(self.get_ref_mut_hex_view_decoded().static_cast_mut() as &mut Object);

            // Set the cursor and his format.
            self.get_ref_mut_hex_view_decoded().set_text_cursor(&cursor);
            self.get_ref_mut_hex_view_decoded().set_current_char_format(&header_format);

            // Clear the selection.
            cursor.clear_selection();
            self.get_ref_mut_hex_view_decoded().set_text_cursor(&cursor);

            // Unblock the signals.
            blocker.unblock();*/

        // Load the "Info" data to the view.
        //unsafe { self.table_info_type_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(&stuff_non_ui.packed_file_path[1])); }
        //unsafe { self.table_info_version_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.version))); }
        //unsafe { self.table_info_entry_count_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str(format!("{}", stuff_non_ui.entry_count))); }
    }

    fn get_ref_hex_view_index(&self) -> &TextEdit {
        unsafe { self.hex_view_index.load(Ordering::SeqCst).as_ref().unwrap() }
    }

    fn get_ref_mut_hex_view_index(&self) -> &mut TextEdit {
        unsafe { self.hex_view_index.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    fn get_ref_hex_view_raw(&self) -> &TextEdit {
        unsafe { self.hex_view_raw.load(Ordering::SeqCst).as_ref().unwrap() }
    }

    fn get_ref_mut_hex_view_raw(&self) -> &mut TextEdit {
        unsafe { self.hex_view_raw.load(Ordering::SeqCst).as_mut().unwrap() }
    }

    fn get_ref_hex_view_decoded(&self) -> &TextEdit {
        unsafe { self.hex_view_decoded.load(Ordering::SeqCst).as_ref().unwrap() }
    }

    fn get_ref_mut_hex_view_decoded(&self) -> &mut TextEdit {
        unsafe { self.hex_view_decoded.load(Ordering::SeqCst).as_mut().unwrap() }
    }
}

/// Implementation of `PackedFileDecoderViewRaw`.
impl PackedFileDecoderViewRaw {

    /// This function syncronize the selection between the Hex View and the Decoded View of the PackedFile Data.
    /// Pass `hex = true` if the selected view is the Hex View. Otherwise, pass false.
    pub fn hex_selection_sync(&self, hex: bool) {

        let hex_view_raw = unsafe { self.hex_view_raw.as_mut().unwrap() };
        let hex_view_decoded = unsafe { self.hex_view_decoded.as_mut().unwrap() };

        let cursor =  if hex { hex_view_raw.text_cursor() } else { hex_view_decoded.text_cursor() };
        let mut cursor_dest =  if !hex { hex_view_raw.text_cursor() } else { hex_view_decoded.text_cursor() };

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

        cursor_dest.move_position(MoveOperation::Start);
        cursor_dest.move_position((MoveOperation::NextCharacter, MoveMode::Move, selection_start as i32));
        cursor_dest.move_position((MoveOperation::NextCharacter, MoveMode::Keep, (selection_end - selection_start) as i32));

        // Block the signals during this, so we don't trigger an infinite loop.
        if hex {
            let mut blocker = SignalBlocker::new(hex_view_decoded.static_cast_mut() as &mut Object);
            hex_view_decoded.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
        else {
            let mut blocker = SignalBlocker::new(hex_view_raw.static_cast_mut() as &mut Object);
            hex_view_raw.set_text_cursor(&cursor_dest);
            blocker.unblock();
        }
    }
}
