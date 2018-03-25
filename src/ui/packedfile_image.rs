// In this file are all the helper functions used by the UI when showing Image PackedFiles.
extern crate std;
extern crate gtk;

use std::env;
use std::fs::File;
use std::io::Write;
use gtk::prelude::*;
use gtk::{Image, ScrolledWindow, Grid, Statusbar};
use ui::show_message_in_statusbar;

/// This function is used to create a ScrolledWindow with the selected Image inside. If there is an
/// error, just say it in the statusbar.
pub fn create_image_view(
    packed_file_data_display: &Grid,
    status_bar: &Statusbar,
    image_name: &str,
    image_data: &[u8]
) {

    // Create a temporal file for the image in the TEMP directory of the filesystem.
    let mut temporal_file_path = env::temp_dir();
    temporal_file_path.push(image_name);
    match File::create(&temporal_file_path) {
        Ok(mut temporal_file) => {

            // If there is an error while trying to write the image to the TEMP folder, report it.
            if let Err(_) = temporal_file.write_all(image_data) {
                let message = format!("Error while trying to open the following image: \"{}\".", image_name);
                show_message_in_statusbar(status_bar, message);
            }

            // If it worked, create an Image with the new file and show it inside a ScrolledWindow.
            else {
                let image = Image::new_from_file(&temporal_file_path);
                let packed_file_source_view_scroll = ScrolledWindow::new(None, None);

                packed_file_source_view_scroll.add(&image);
                packed_file_source_view_scroll.set_hexpand(true);
                packed_file_source_view_scroll.set_vexpand(true);

                packed_file_data_display.attach(&packed_file_source_view_scroll, 0, 0, 1, 1);
                packed_file_data_display.show_all();
            }
        }
        Err(_) => {

            // If there is an error when trying to create the file into the TEMP folder, report it.
            let message = format!("Error while trying to open the following image: \"{}\".", image_name);
            show_message_in_statusbar(status_bar, message);
        },
    }
}
