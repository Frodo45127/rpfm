// In this file are all the helper functions used by the UI when editing Text PackedFiles.
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::widget::Widget;
use qt_widgets::plain_text_edit::PlainTextEdit;

use qt_core::connection::Signal;

use std::cell::RefCell;
use std::rc::Rc;

use AppUI;
use Commands;
use ui::*;
use error::Result;

/// Struct `PackedFileTextView`: contains all the stuff we need to give to the program to show a
/// `PlainTextEdit` with the data of a plain text PackedFile, allowing us to manipulate it.
pub struct PackedFileTextView {
    pub save_changes: SlotNoArgs<'static>,
}

/// Implementation of PackedFileLocTreeView.
impl PackedFileTextView {

    /// This functin returns a dummy struct. Use it for initialization.
    pub fn new() -> Self {

        // Create some dummy slots and return it.
        Self {
            save_changes: SlotNoArgs::new(|| {}),
        }
    }

    /// This function creates a new TreeView with the PackedFile's View as father and returns a
    /// `PackedFileLocTreeView` with all his data.
    pub fn create_text_view(
        ui_message_sender: Sender<Commands>,
        receiver_qt: &Rc<RefCell<Receiver<Result<Vec<u8>>>>>,
        is_modified: &Rc<RefCell<bool>>,
        app_ui: &AppUI,
        packed_file_index: &usize,
    ) -> Result<Self> {

        // Get the text of the PackedFile.
        ui_message_sender.send(Commands::DecodePackedFileText).unwrap();
        //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();

        // Get the response from the other thread.
        let text: String = match check_message_validity_recv(&receiver_qt) {
            Ok(data) => data,
            Err(error) => return Err(error)
        };

        // Create the PlainTextEdit.
        let plain_text_edit = PlainTextEdit::new(&QString::from_std_str(&text)).into_raw();

        // Add it to the view.
        unsafe { app_ui.packed_file_layout.as_mut().unwrap().add_widget((plain_text_edit as *mut Widget, 0, 0, 1, 1)); }

        // Create the stuff needed for this to work.
        let stuff = Self {
            save_changes: SlotNoArgs::new(clone!(
                packed_file_index,
                app_ui,
                is_modified,
                ui_message_sender,
                receiver_qt => move || {

                    // Tell the background thread to start saving the PackedFile.
                    ui_message_sender.send(Commands::EncodePackedFileText).unwrap();

                    // Get the text from the PlainTextEdit.
                    let text;
                    unsafe { text = plain_text_edit.as_mut().unwrap().to_plain_text().to_std_string(); }

                    // Send the new text.
                    //TODO sender_qt_data.send(serde_json::to_vec(&(text, packed_file_index)).map_err(From::from)).unwrap();

                    // Get the incomplete path of the edited PackedFile.
                    ui_message_sender.send(Commands::GetPackedFilePath).unwrap();
                    //TODO sender_qt_data.send(serde_json::to_vec(&packed_file_index).map_err(From::from)).unwrap();
                    let path: Vec<String> = match check_message_validity_recv(&receiver_qt) {
                        Ok(data) => data,
                        Err(_) => panic!(THREADS_MESSAGE_ERROR)
                    };

                    // Set the mod as "Modified".
                    *is_modified.borrow_mut() = set_modified(true, &app_ui, Some(path));
                }
            )),
        };

        // Action to trigger a save on edit.
        unsafe { plain_text_edit.as_ref().unwrap().signals().text_changed().connect(&stuff.save_changes); }

        // Return the slots.
        Ok(stuff)
    }
}
