// In this file are all the helper functions used by the UI when showing Image PackedFiles.
extern crate qt_widgets;
extern crate qt_gui;
extern crate qt_core;

use qt_widgets::widget::Widget;
use qt_widgets::label::Label;
use qt_gui::pixmap::Pixmap;
use qt_core::qt::AspectRatioMode;
use qt_core::flags::Flags;

use std::cell::RefCell;
use std::rc::Rc;

use crate::Commands;
use crate::Data;
use crate::common::communications::*;
use crate::ui::*;
use crate::error::Result;

/// This function creates a new TreeView with the PackedFile's View as father and returns a
/// `PackedFileLocTreeView` with all his data.
pub fn create_image_view(
    sender_qt: Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt: &Rc<RefCell<Receiver<Data>>>,
    layout: *mut GridLayout,
    packed_file_path: &Rc<RefCell<Vec<String>>>,
) -> Result<()> {

    // Get the path of the extracted Image.
    sender_qt.send(Commands::DecodePackedFileImage).unwrap();
    sender_qt_data.send(Data::VecString(packed_file_path.borrow().to_vec())).unwrap();
    let path = match check_message_validity_recv2(&receiver_qt) { 
        Data::PathBuf(data) => data,
        Data::Error(error) => return Err(error),
        _ => panic!(THREADS_MESSAGE_ERROR), 
    };

    // Get the image's path.
    let path_string = path.to_string_lossy().as_ref().to_string();

    // Create the QPixmap.
    let image = Pixmap::new(&QString::from_std_str(&path_string));

    // Get the size of the holding widget.
    let widget_height;
    let widget_width;
    unsafe { widget_height = layout.as_mut().unwrap().parent_widget().as_mut().unwrap().height(); }
    unsafe { widget_width = layout.as_mut().unwrap().parent_widget().as_mut().unwrap().width(); }

    // If the image is bigger than the current widget...
    let scaled_image = if image.height() >= widget_height || image.width() >= widget_width {

        // Resize it so it doesn't occupy the entire screen if it's too big.
        image.scaled((widget_height - 25, widget_width - 25, AspectRatioMode::KeepAspectRatio))
    }

    // Otherwise, we use the normal image.
    else { image };

    // Create a Label.
    let label = Label::new(()).into_raw();

    // Center the Label.
    unsafe { label.as_mut().unwrap().set_alignment(Flags::from_int(132))}

    // Put the image into the Label.
    unsafe { label.as_mut().unwrap().set_pixmap(&scaled_image); }

    // Attach the label to the PackedFile View.
    unsafe { layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }

    // Return success.
    Ok(())
}
