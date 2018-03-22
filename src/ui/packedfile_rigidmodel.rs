// In this file are all the helper functions used by the UI when decoding RigidModel PackedFiles.

extern crate failure;

use self::failure::Error;

use gtk::prelude::*;
use gtk::{
    Box, ScrolledWindow, Orientation, Button, Expander, Label, PolicyType, Entry, Grid
};

use common::coding_helpers::*;

/// Struct PackedFileRigidModelDataView: contains all the stuff we need to give to the program to
/// show a TreeView with the data of a RigidModel file, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileRigidModelDataView {
    pub packed_file_save_button: Button,
    pub rigid_model_game_label: Label,
    pub rigid_model_game_patch_button: Button,
    pub packed_file_texture_paths_index: Vec<u32>,
    pub packed_file_texture_paths: Vec<Vec<Entry>>,
}

/// Implementation of "PackedFileRigidModelDataView"
impl PackedFileRigidModelDataView {

    /// This function creates a new Data View (custom layout) with "packed_file_data_display" as
    /// father and returns a PackedFileRigidModelDataView with all his data. This can fail, so
    /// we return a result.
    pub fn create_data_view(
        packed_file_data_display: &Grid,
        packed_file_decoded: &::packedfile::rigidmodel::RigidModel
    ) -> Result<PackedFileRigidModelDataView, Error> {

        // Button for saving the PackedFile. It goes before everything, so it's not included in the
        // scrolledWindow.
        let packed_file_save_button = Button::new_with_label("Save to PackedFile");

        // Internal scrolledWindow, so if there are too many lods, we can scroll through them.
        // Inside it we put a box to fit all the labels and stuff properly.
        let packed_file_data_display_scroll = ScrolledWindow::new(None, None);
        let packed_file_data_display_scroll_inner_box = Box::new(Orientation::Vertical, 0);
        packed_file_data_display_scroll.set_hexpand(true);
        packed_file_data_display_scroll.set_vexpand(true);

        let rigid_model_game_box = Box::new(Orientation::Horizontal, 0);
        rigid_model_game_box.set_size_request(400, 0);

        let rigid_model_game_label = Label::new(Some(
            if packed_file_decoded.packed_file_header.packed_file_header_model_type == 6 {
                "RigidModel compatible with: \"Attila\"."
            }
            else {
                "RigidModel compatible with: \"Warhammer 1&2\"."
            }
        ));
        rigid_model_game_label.set_margin_start(4);
        rigid_model_game_label.set_xalign(0.0);
        rigid_model_game_label.set_yalign(0.5);

        let rigid_model_game_patch_button = Button::new_with_label("Patch to Warhammer 1&2");
        if packed_file_decoded.packed_file_header.packed_file_header_model_type == 6 {
            rigid_model_game_patch_button.set_sensitive(true);
        }
        else {
            rigid_model_game_patch_button.set_sensitive(false);
        }

        rigid_model_game_box.pack_start(&rigid_model_game_label, false, false, 0);
        rigid_model_game_box.pack_end(&rigid_model_game_patch_button, false, false, 0);

        let rigid_model_textures_label = Label::new(Some("Textures used by this RigidModel:"));
        rigid_model_textures_label.set_margin_start(4);
        rigid_model_textures_label.set_xalign(0.0);
        rigid_model_textures_label.set_yalign(0.5);

        packed_file_data_display_scroll_inner_box.pack_start(&rigid_model_game_box, false, false, 0);
        packed_file_data_display_scroll_inner_box.pack_start(&rigid_model_textures_label, false, false, 0);

        // The texture position should never change in the data, so we get the positions of all the
        // textures in the RigidModel.
        let mut texture_index: Vec<u32> = vec![];

        // Check if it's a building/prop/decal.
        if packed_file_decoded.packed_file_data.packed_file_data_lods_data
            .windows(12)
            .find(|window: &&[u8]| String::from_utf8_lossy(window) == "rigidmodels/") != None {

            // If we founded that, it's a building/prop/decal, so we try to get the positions where
            // his texture paths are.
            let mut index = 0;
            loop {
                match packed_file_decoded.packed_file_data.packed_file_data_lods_data[index..]
                    .windows(12)
                    .position(|window: &[u8]| String::from_utf8_lossy(window) == "rigidmodels/") {
                        Some(position) => {
                            texture_index.push((position + index) as u32);
                            index += position + 1;
                        },
                        None => break,
                }
            }
        }

        // If not, check if it's a unit model.
        else if packed_file_decoded.packed_file_data.packed_file_data_lods_data
            .windows(14)
            .find(|window: &&[u8]| String::from_utf8_lossy(window) == "variantmeshes/") != None {

            // If we founded that, it's a building/prop/decal, so we try to get the positions where
            // his texture paths are.
            let mut index = 0;
            loop {
                match packed_file_decoded.packed_file_data.packed_file_data_lods_data[index..]
                    .windows(14)
                    .position(|window: &[u8]| String::from_utf8_lossy(window) == "variantmeshes/") {
                        Some(position) => {
                            texture_index.push((position + index) as u32);
                            index += position + 1;
                        },
                        None => break,
                }
            }
        }

        // If none of these have worked, this is not a decodeable rigidmodel.
        else {
            return Err(format_err!("Error while trying to get the texture directories (none has beem found)."))
        }

        // Rules to diferentiate between decal, building/prop and units:
        // - texture_index.len() = 1 => decal.
        // - rigidmodel found => building/prop.
        // - variantmeshes found => unit.

        // This will store all the paths, separated by lod.
        let mut packed_file_texture_paths = vec![];

        // If it's a decal...
        if texture_index.len() == 1 {
            let mut packed_file_texture_paths_lod = vec![];
            let lod_texture_expander = Expander::new(Some(&*format!("Decal texture folder")));
            let lod_texture_expander_box = Box::new(Orientation::Vertical, 0);
            lod_texture_expander.add(&lod_texture_expander_box);

            let texture_info_box = Box::new(Orientation::Horizontal, 0);
            let texture_type = Label::new(Some("Texture Directory:"));
            texture_type.set_xalign(0.0);
            texture_type.set_yalign(0.5);
            texture_type.set_size_request(60, 0);

            // Then we get it's path, and put it in a gtk::Entry.
            let texture_path = Entry::new();

            match decode_string_u8_0padded(
                &packed_file_decoded.packed_file_data.packed_file_data_lods_data[
                    texture_index[0] as usize..
                    (texture_index[0] as u32 + 256u32) as usize
                ]
            ) {
                Ok(result) => texture_path.get_buffer().set_text(&*result.0),
                Err(_) => texture_path.get_buffer().set_text("Error while decoding."),
            };

            texture_path.get_buffer().set_max_length(Some(256u16));
            texture_path.set_editable(true);

            // We need to put a ScrolledWindow around the Entry, so we can move the
            // text if it's too long.
            let texture_path_scroll = ScrolledWindow::new(None, None);
            texture_path_scroll.set_size_request(550, 0);
            texture_path_scroll.set_policy(PolicyType::External, PolicyType::Never);
            texture_path_scroll.set_max_content_width(500);
            texture_path_scroll.add(&texture_path);

            texture_info_box.pack_start(&texture_type, false, false, 10);
            texture_info_box.pack_start(&texture_path_scroll, false, false, 0);
            lod_texture_expander_box.pack_start(&texture_info_box, false, false, 0);

            packed_file_texture_paths_lod.push(texture_path);
            packed_file_texture_paths.push(packed_file_texture_paths_lod);
            packed_file_data_display_scroll_inner_box.pack_start(&lod_texture_expander, false, false, 0);
        }
        else {

            // If we can subdivide the amount of textures found in the rigidmodel, we have the first
            // one to be the directory, and the other five to be the textures of the lod.
            if texture_index.len() % 6 == 0 {

                // We are going to change our lod every 6 indexes...
                let lods = texture_index.len() / 6;

                // For each lod...
                for lod in 0..lods {

                    let mut packed_file_texture_paths_lod = vec![];
                    let lod_texture_expander = Expander::new(Some(&*format!("Lod {}", lod + 1)));
                    let lod_texture_expander_box = Box::new(Orientation::Vertical, 0);
                    lod_texture_expander.add(&lod_texture_expander_box);


                    // For each texture found (except the first of the group, thats their dir)...
                    for index in 1..6 {

                        // First, we get it's type.
                        let texture_info_box = Box::new(Orientation::Horizontal, 0);
                        let texture_type = Label::new(Some(

                            //0 (Diffuse), 1 (Normal), 11 (Specular), 12 (Gloss), 3/10 (Mask), 5(no idea).
                            match decode_integer_u32(&packed_file_decoded.packed_file_data.packed_file_data_lods_data[(texture_index[index + (lod * 6)] - 4) as usize..(texture_index[index + (lod * 6)] as usize)]) {
                                Ok(result) => {

                                    match result {
                                        0 => "Diffuse:",
                                        1 => "Normal:",
                                        11 => "Specular:",
                                        12 => "Gloss:",
                                        3 | 10 => "Mask:",
                                        5 => "Unknown:",
                                        _ => return Err(format_err!("Error. Unknown mask type."))
                                    }
                                }
                                Err(error) => return Err(error)
                            }
                        ));

                        texture_type.set_xalign(0.0);
                        texture_type.set_yalign(0.5);
                        texture_type.set_size_request(60, 0);

                        // Then we get it's path, and put it in a gtk::Entry.
                        let texture_path = Entry::new();

                        match decode_string_u8_0padded(
                            &packed_file_decoded.packed_file_data.packed_file_data_lods_data[
                                texture_index[index + (lod * 6)] as usize..
                                (texture_index[index + (lod * 6)] as u32 + 255u32) as usize
                            ]
                        ) {
                            Ok(result) => texture_path.get_buffer().set_text(&*result.0),
                            Err(_) => texture_path.get_buffer().set_text("Error while decoding."),
                        };

                        texture_path.get_buffer().set_max_length(Some(256u16));
                        texture_path.set_editable(true);

                        // We need to put a ScrolledWindow around the Entry, so we can move the
                        // text if it's too long.
                        let texture_path_scroll = ScrolledWindow::new(None, None);
                        texture_path_scroll.set_size_request(650, 0);
                        texture_path_scroll.set_policy(PolicyType::External, PolicyType::Never);
                        texture_path_scroll.set_max_content_width(600);
                        texture_path_scroll.add(&texture_path);

                        texture_info_box.pack_start(&texture_type, false, false, 10);
                        texture_info_box.pack_start(&texture_path_scroll, false, false, 0);
                        lod_texture_expander_box.pack_start(&texture_info_box, false, false, 0);

                        packed_file_texture_paths_lod.push(texture_path);
                    }
                    packed_file_texture_paths.push(packed_file_texture_paths_lod);
                    packed_file_data_display_scroll_inner_box.pack_start(&lod_texture_expander, false, false, 0);
                }
            }

            // If not, return error.
            else {
                return Err(format_err!("Error while trying to get the texture directories (an irregular amount of them has been found)."))
            }
        }

        packed_file_data_display_scroll.add(&packed_file_data_display_scroll_inner_box);
        packed_file_data_display.attach(&packed_file_save_button, 0, 0, 1, 1);
        packed_file_data_display.attach(&packed_file_data_display_scroll, 0, 1, 1, 1);
        packed_file_data_display.show_all();

        Ok(PackedFileRigidModelDataView {
            packed_file_save_button,
            rigid_model_game_label,
            rigid_model_game_patch_button,
            packed_file_texture_paths,
            packed_file_texture_paths_index: texture_index,
        })
    }

    /// This function get the texture path entries of a RigidModel from the UI and saves them into the
    /// opened RigidModel.
    pub fn return_data_from_data_view(
        packed_file_texture_paths: &[Vec<Entry>],
        packed_file_texture_paths_index: &[u32],
        packed_file_data_lods_data: &mut Vec<u8>
    ) -> Result<Vec<u8>, Error> {

        // If it's a decal...
        if packed_file_texture_paths_index.len() == 1 {

            // We just replace the text in the position we have and return the changed vector.
            let new_texture_path = encode_string_u8_0padded(&(packed_file_texture_paths[0][0].get_text().unwrap(), 256))?;

            packed_file_data_lods_data.splice(
                (packed_file_texture_paths_index[0] as usize)..((packed_file_texture_paths_index[0] + 256) as usize),
                new_texture_path.iter().cloned());
        }

        // If it's a building/prop/unit...
        else {

            // Get the amount of lods...
            let lods = packed_file_texture_paths_index.len() / 6;
            for lod in 0..lods {

                // For each texture (we skip the texture directory)...
                for texture in 1..6 {

                    // We get the new texture, and replace the old one with the new one.
                    let new_texture_path = encode_string_u8_0padded(&(packed_file_texture_paths[lod][texture - 1].get_text().unwrap(), 256))?;

                    packed_file_data_lods_data.splice(
                        (packed_file_texture_paths_index[texture + (lod * 6)] as usize)..((packed_file_texture_paths_index[texture + (lod * 6)] + 256) as usize),
                        new_texture_path.iter().cloned());
                }
            }
        }

        Ok(packed_file_data_lods_data.to_vec())
    }
}
