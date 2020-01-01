//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI when decoding DB PackedFiles.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use crate::AppUI;
use crate::Commands;
use crate::Data;

use crate::communications::*;
use crate::ui::*;

use super::*;

/// This function creates a new Table with the PackedFile's View as father and returns a
/// `PackedFileDBTreeView` with all his data.
pub fn create_notes_view(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    app_ui: &AppUI,
    layout: *mut GridLayout,
    packed_file_path: &Rc<RefCell<Vec<String>>>,
    packedfiles_open_in_packedfile_view: &Rc<RefCell<BTreeMap<i32, Rc<RefCell<Vec<String>>>>>>,
) -> PackedFileTextView {

    // Get the text of the PackedFile.
    sender_qt.send(Commands::GetNotes).unwrap();
    let text = if let Data::String(data) = check_message_validity_recv2(&receiver_qt) { data } else { panic!(THREADS_MESSAGE_ERROR) };

    PackedFileTextView::create_text_view(
        sender_qt,
        sender_qt_data,
        receiver_qt,
        app_ui,
        layout,
        packed_file_path,
        packedfiles_open_in_packedfile_view,
        &Rc::new(RefCell::new(TextType::Notes(text))),
    ).unwrap()
}
