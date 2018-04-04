// In this file are all the helper functions used by the UI when editing Text PackedFiles.
extern crate std;
extern crate gtk;
extern crate sourceview;

use sourceview::prelude::*;
use sourceview::{
    Buffer, View, Language, LanguageManager, StyleScheme, StyleSchemeManager
};
use gtk::prelude::*;
use gtk::{ScrolledWindow, Grid, Statusbar};

use common::coding_helpers::*;
use ui::show_message_in_statusbar;


/// This function is used to create a ScrolledWindow with the SourceView inside. If there is an
/// error, just say it in the statusbar.
pub fn create_text_view(
    packed_file_data_display: &Grid,
    status_bar: &Statusbar,
    packed_file_name: &str,
    packed_file_data: &[u8],
) -> Option<Buffer> {

    // Before doing anything, we try to decode the data. Only if we success, we create
    // the SourceView and add the data to it.
    // NOTE: This only works for UTF-8 encoded files. Check their encoding before adding them here to be decoded.
    match decode_string_u8(&packed_file_data) {
        Ok(text) => {

            // We create the new SourceView (in a ScrolledWindow) and his buffer,
            // get his buffer and put the text in it.
            let source_view_scroll = ScrolledWindow::new(None, None);
            let source_view_buffer: Buffer = Buffer::new(None);
            let source_view = View::new_with_buffer(&source_view_buffer);

            // We config the SourceView for our needs.
            source_view_scroll.set_vexpand(true);
            source_view_scroll.set_hexpand(true);
            source_view.set_tab_width(4);
            source_view.set_show_line_numbers(true);
            source_view.set_indent_on_tab(true);
            source_view.set_highlight_current_line(true);

            // Set the syntax hightlight to "monokai-extended".
            // TODO: Make this be toggleable through the preferences window.
            let style_scheme_manager = StyleSchemeManager::get_default().unwrap();
            let style: Option<StyleScheme> = style_scheme_manager.get_scheme("monokai-extended");
            if let Some(style) = style { source_view_buffer.set_style_scheme(&style); }

            // We attach it to the main grid.
            packed_file_data_display.attach(&source_view_scroll, 0, 0, 1, 1);

            // Then, we get the Language of the file.
            let language_manager = LanguageManager::get_default().unwrap();
            let packedfile_language: Option<Language> = if packed_file_name.ends_with(".xml") ||
                packed_file_name.ends_with(".xml.shader") ||
                packed_file_name.ends_with(".xml.material") ||
                packed_file_name.ends_with(".variantmeshdefinition") ||
                packed_file_name.ends_with(".environment") ||
                packed_file_name.ends_with(".lighting") ||
                packed_file_name.ends_with(".wsmodel") {

                // For any of this, use xml.
                language_manager.get_language("xml")
            }
            else if packed_file_name.ends_with(".lua") {
                language_manager.get_language("lua")
            }
            else if packed_file_name.ends_with(".csv") {
                language_manager.get_language("csv")
            }
            else if packed_file_name.ends_with(".inl") {

                // These seem to be written in C++.
                language_manager.get_language("cpp")
            }
            else {

                // If none of the conditions has been met, it's a plain text file.
                None
            };

            // Then we set the Language of the file, if it has one.
            if let Some(language) = packedfile_language {
                source_view_buffer.set_language(&language);
            }

            // Add the text to the SourceView.
            source_view_buffer.set_text(&*text);

            // And show everything.
            source_view_scroll.add(&source_view);
            packed_file_data_display.show_all();

            return Some(source_view_buffer)
        }
        Err(_) => {

            // If there is an error when trying to decode the PackedFile, report it.
            let message = format!("Error while trying to open the following file: \"{}\".", packed_file_name);
            show_message_in_statusbar(status_bar, message);

            return None
        }
    }
}
