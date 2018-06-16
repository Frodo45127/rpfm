// In this file are all the helper functions used by the UI when showing Image PackedFiles.
extern crate gtk;
extern crate failure;
/*
use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use std::fs::File;
use std::io::Write;
use gtk::prelude::*;
use gtk::{Image, ScrolledWindow};
use AppUI;
use packfile::packfile::PackFile;

/// This function is used to create a ScrolledWindow with the selected Image inside. If there is an
/// error, just say it in the statusbar.
pub fn create_image_view(
    app_ui: &AppUI,
    pack_file: &Rc<RefCell<PackFile>>,
    packed_file_decoded_index: &usize,
) -> Result<(), Error> {

    // Get the data of the image we want to open, and his name.
    let image_data = &pack_file.borrow().data.packed_files[*packed_file_decoded_index].data;
    let image_name = &pack_file.borrow().data.packed_files[*packed_file_decoded_index].path.last().unwrap().to_owned();

    // Create a temporal file for the image in the TEMP directory of the filesystem.
    let mut temporal_file_path = env::temp_dir();
    temporal_file_path.push(image_name);
    match File::create(&temporal_file_path) {
        Ok(mut temporal_file) => {

            // If there is an error while trying to write the image to the TEMP folder, report it.
            if temporal_file.write_all(image_data).is_err() {
                return Err(format_err!("Error while trying to open the following image: \"{}\".", image_name));
            }

            // If it worked, create an Image with the new file and show it inside a ScrolledWindow.
            else {
                let image = Image::new_from_file(&temporal_file_path);
                let image_scroll = ScrolledWindow::new(None, None);

                image_scroll.add(&image);
                image_scroll.set_hexpand(true);
                image_scroll.set_vexpand(true);

                app_ui.packed_file_data_display.attach(&image_scroll, 0, 0, 1, 1);
                app_ui.packed_file_data_display.show_all();

                // Return success.
                Ok(())
            }
        }

        // If there is an error when trying to create the file into the TEMP folder, report it.
        Err(_) => return Err(format_err!("Error while trying to open the following image: \"{}\".", image_name)),
    }
}
*/
