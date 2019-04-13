//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI when editing Text PackedFiles.

use qt_widgets::dialog::Dialog;
use qt_widgets::dialog_button_box::{DialogButtonBox, StandardButton};
use qt_widgets::plain_text_edit::PlainTextEdit;
use qt_widgets::widget::Widget;

use qt_core::connection::Signal;

use std::cell::RefCell;
use std::rc::Rc;

use crate::SUPPORTED_GAMES;
use crate::GAME_SELECTED;
use crate::AppUI;
use crate::Commands;
use crate::Data;
use crate::common::communications::*;
use crate::ui::*;
use crate::error::Result;

/// Struct `PackedFileTextView`: contains all the stuff we need to give to the program to show a
/// `PlainTextEdit` with the data of a plain text PackedFile, allowing us to manipulate it.
pub struct PackedFileTextView {
    pub save_changes: SlotNoArgs<'static>,
    pub check_syntax: SlotNoArgs<'static>,
}

/// Implementation of PackedFileLocTreeView.
impl PackedFileTextView {

    /// This function creates a new TreeView with the PackedFile's View as father and returns a
    /// `PackedFileLocTreeView` with all his data.
    pub fn create_text_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
    ) -> Result<Self> {

        // Get the text of the PackedFile.
        sender_qt.send(Commands::DecodePackedFileText).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.borrow().to_vec())).unwrap();
        let text = match check_message_validity_recv2(&receiver_qt) { 
            Data::String(data) => data,
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR), 
        };

        // Create the PlainTextEdit and the checking button.
        let plain_text_edit = PlainTextEdit::new(&QString::from_std_str(&text)).into_raw();
        let check_syntax_button = PushButton::new(&QString::from_std_str("Check Syntax")).into_raw();

        // Add it to the view.
        unsafe { layout.as_mut().unwrap().add_widget((plain_text_edit as *mut Widget, 0, 0, 1, 1)); }
        if packed_file_path.borrow().last().unwrap().ends_with(".lua") && SUPPORTED_GAMES.get(&**GAME_SELECTED.lock().unwrap()).unwrap().ca_types_file.is_some() {
            unsafe { layout.as_mut().unwrap().add_widget((check_syntax_button as *mut Widget, 1, 0, 1, 1)); }
        }

        // Create the stuff needed for this to work.
        let stuff = Self {
            save_changes: SlotNoArgs::new(clone!(
                packed_file_path,
                app_ui,
                receiver_qt,
                sender_qt,
                sender_qt_data => move || {

                    // Get the text from the PlainTextEdit.
                    let text = unsafe { plain_text_edit.as_mut().unwrap().to_plain_text().to_std_string() };

                    // Tell the background thread to start saving the PackedFile.
                    sender_qt.send(Commands::EncodePackedFileText).unwrap();
                    sender_qt_data.send(Data::StringVecString((text, packed_file_path.borrow().to_vec()))).unwrap();

                    update_treeview(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        app_ui.folder_tree_view,
                        Some(app_ui.folder_tree_filter),
                        app_ui.folder_tree_model,
                        TreeViewOperation::Modify(vec![TreePathType::File(packed_file_path.borrow().to_vec())]),
                    );
                }
            )),

            check_syntax: SlotNoArgs::new(clone!(
                app_ui,
                sender_qt,
                receiver_qt => move || {

                    // Tell the background thread to check the PackedFile, and return the result.
                    sender_qt.send(Commands::CheckScriptWithKailua).unwrap();
                    let result = match check_message_validity_recv2(&receiver_qt) { 
                        Data::VecString(data) => data,
                        Data::Error(error) => return show_dialog(app_ui.window, false, error),
                        _ => panic!(THREADS_MESSAGE_ERROR), 
                    };

                    let mut clean_result = String::new();
                    result.iter().for_each(|x| clean_result.push_str(&format!("{}\n", x)));

                    // Create the dialog.
                    let dialog;
                    unsafe { dialog = Dialog::new_unsafe(app_ui.window as *mut Widget).into_raw(); }

                    // Create the Grid.
                    let grid = GridLayout::new().into_raw();
                    unsafe { dialog.as_mut().unwrap().set_layout(grid as *mut Layout); }

                    // Configure the dialog.
                    unsafe { dialog.as_mut().unwrap().set_window_title(&QString::from_std_str("Script Checked!")); }
                    unsafe { dialog.as_mut().unwrap().set_modal(false); }
                    unsafe { dialog.as_mut().unwrap().resize((950, 500)); }

                    // Create the Text View and the ButtonBox.
                    let mut error_report = PlainTextEdit::new(&QString::from_std_str(clean_result));
                    let mut button_box = DialogButtonBox::new(());
                    error_report.set_read_only(true);
                    let close_button = button_box.add_button(StandardButton::Close);
                    unsafe { close_button.as_mut().unwrap().signals().released().connect(&dialog.as_mut().unwrap().slots().close()); }
                    unsafe { grid.as_mut().unwrap().add_widget((error_report.into_raw() as *mut Widget, 0, 0, 1, 1)); }
                    unsafe { grid.as_mut().unwrap().add_widget((button_box.into_raw() as *mut Widget, 1, 0, 1, 1)); }

                    // Show the Dialog, so it doesn't block the program.
                    unsafe { dialog.as_mut().unwrap().show(); }
                }
            )),
        };

        // Actions to trigger the slots.
        unsafe { plain_text_edit.as_ref().unwrap().signals().text_changed().connect(&stuff.save_changes); }
        unsafe { check_syntax_button.as_ref().unwrap().signals().released().connect(&stuff.check_syntax); }

        // Return the slots.
        Ok(stuff)
    }
}
