// In this file are all the helper functions used by the UI when decoding RigidModel PackedFiles.

use gtk::prelude::*;
use gtk::{
    Box, TreeView, ListStore, ScrolledWindow, Orientation, Notebook,
    CellRendererText, TreeViewColumn, CellRendererToggle, Type, Label
};

/// Struct PackedFileRigidModelDataView: contains all the stuff we need to give to the program to
/// show a TreeView with the data of a RigidModel file, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileRigidModelDataView {
    pub packed_file_data_view: Box,
}

/// Implementation of "PackedFileRigidModelDataView"
impl PackedFileRigidModelDataView {

    /// This function creates a new Data View (custom layout) with "packed_file_data_display" as
    /// father and returns a PackedFileRigidModelDataView with all his data.
    pub fn create_data_view(
        packed_file_data_display: &Box,
        packed_file_decoded: &::packedfile::rigidmodel::RigidModel
    ) {
        let headers_box = Box::new(Orientation::Horizontal, 0);

        let header_box = Box::new(Orientation::Vertical, 0);
        let header_signature = Label::new(Some(&*packed_file_decoded.packed_file_header.packed_file_header_signature));
        let header_model_type = Label::new(Some(&*packed_file_decoded.packed_file_header.packed_file_header_model_type.to_string()));
        let header_lods_count = Label::new(Some(&*packed_file_decoded.packed_file_header.packed_file_header_lods_count.to_string()));
        let header_base_skeleton = Label::new(Some(&*packed_file_decoded.packed_file_header.packed_file_data_base_skeleton.0));

        header_box.add(&header_signature);
        header_box.add(&header_model_type);
        header_box.add(&header_lods_count);
        header_box.add(&header_base_skeleton);


        let lod_headers_notebook = Notebook::new();

        for lod in 0..packed_file_decoded.packed_file_header.packed_file_header_lods_count {
            let lod_header = packed_file_decoded.packed_file_data.packed_file_data_lod_list[lod as usize].clone();
            let lod_header_data = Box::new(Orientation::Vertical, 0);

            let group_counts = Label::new(Some(&*lod_header.groups_count.to_string()));
            let vertex_data_length = Label::new(Some(&*lod_header.vertex_data_length.to_string()));
            let index_data_length = Label::new(Some(&*lod_header.index_data_length.to_string()));
            let start_offset = Label::new(Some(&*lod_header.start_offset.to_string()));
            let lod_zoom_factor = Label::new(Some(&*lod_header.lod_zoom_factor.to_string()));
            let mut mysterious_data_1 = Label::new(Some(&*lod_header.lod_zoom_factor.to_string()));
            let mut mysterious_data_2 = Label::new(Some(&*lod_header.lod_zoom_factor.to_string()));

            if packed_file_decoded.packed_file_header.packed_file_header_model_type == 7 {
                mysterious_data_1 = Label::new(Some(&*lod_header.mysterious_data_1.unwrap().to_string()));
                mysterious_data_2 = Label::new(Some(&*lod_header.mysterious_data_2.unwrap().to_string()));
            }

            lod_header_data.add(&group_counts);
            lod_header_data.add(&vertex_data_length);
            lod_header_data.add(&index_data_length);
            lod_header_data.add(&start_offset);
            lod_header_data.add(&lod_zoom_factor);
            lod_header_data.add(&mysterious_data_1);
            lod_header_data.add(&mysterious_data_2);

            lod_headers_notebook.append_page(&lod_header_data, Some(&Label::new(Some(&*format!("Lod {}", lod + 1)))));
        }

        let lod_data_notebook = Notebook::new();

        for lod in 0..packed_file_decoded.packed_file_header.packed_file_header_lods_count {
            let lod_header = packed_file_decoded.packed_file_data.packed_file_data_lod_list[lod as usize].clone();
            let lod_header_data = Box::new(Orientation::Vertical, 0);

            let group_counts = Label::new(Some(&*format!("Lod {}", lod + 1)));


            lod_header_data.add(&group_counts);

            lod_data_notebook.append_page(&lod_header_data, Some(&Label::new(Some(&*format!("Lod {}", lod + 1)))));
        }

        headers_box.pack_start(&header_box, true, true, 0);
        headers_box.pack_end(&lod_headers_notebook, true, true, 0);

        packed_file_data_display.pack_start(&headers_box, true, true, 0);
        packed_file_data_display.pack_end(&lod_data_notebook, true, true, 0);
        packed_file_data_display.show_all();
    }
}