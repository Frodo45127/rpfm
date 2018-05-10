// In this file are all the helper functions used by the UI when decoding RigidModel PackedFiles.
extern crate failure;

use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;
use gtk::prelude::*;
use gtk::{
    ScrolledWindow, Button, Expander, Label, Entry, Grid
};

use common::coding_helpers::*;
use packedfile::rigidmodel::*;
use packfile::patch_rigid_model_attila_to_warhammer;
use ui::*;
use AppUI;
use packfile::update_packed_file_data_rigid;

/// Struct PackedFileRigidModelDataView: contains all the stuff we need to give to the program to
/// show a TreeView with the data of a RigidModel file, allowing us to manipulate it.
#[derive(Clone)]
pub struct PackedFileRigidModelDataView {
    pub texture_paths_index: Vec<u32>,
    pub texture_paths: Vec<Vec<Entry>>,
}

/// Implementation of "PackedFileRigidModelDataView".
impl PackedFileRigidModelDataView {

    /// This function creates a new Data View (custom layout) with "packed_file_data_display" as
    /// father and returns a PackedFileRigidModelDataView with all his data. This can fail, so
    /// we return a result.
    pub fn create_data_view(
        app_ui: &AppUI,
        pack_file: &Rc<RefCell<PackFile>>,
        packed_file_decoded_index: &usize,
        is_packedfile_opened: &Rc<RefCell<bool>>,
    ) -> Result<(), Error> {

        // We try to decode the RigidModel. If it fails, return error. Otherwise build the UI and load to it the data.
        match RigidModel::read(&pack_file.borrow().pack_file_data.packed_files[*packed_file_decoded_index].packed_file_data) {
            Ok(packed_file_decoded) => {

                // Internal `ScrolledWindow`, so if there are too many lods, we can scroll through them.
                // Inside it we put a Grid to fit all the labels and stuff properly.
                let packed_file_data_display_scroll = ScrolledWindow::new(None, None);
                let packed_file_data_display_grid = Grid::new();
                packed_file_data_display_scroll.set_hexpand(true);
                packed_file_data_display_scroll.set_vexpand(true);
                packed_file_data_display_grid.set_border_width(6);
                packed_file_data_display_grid.set_row_spacing(6);
                packed_file_data_display_grid.set_column_spacing(3);

                let compatible_label = Label::new(Some("RigidModel compatible with: "));
                let game_label = Label::new(Some(
                    match packed_file_decoded.packed_file_header.packed_file_header_model_type {
                        6 => "Attila",
                        7 => "Warhammer 1&2",
                        _ => "Don't know."
                    }
                ));
                let patch_attila_to_warhammer_button = Button::new_with_label("Patch to Warhammer 1&2");

                // Only enable it for Attila's RigidModels.
                match packed_file_decoded.packed_file_header.packed_file_header_model_type {
                    6 => patch_attila_to_warhammer_button.set_sensitive(true),
                    _ => patch_attila_to_warhammer_button.set_sensitive(false),
                }

                let textures_label = Label::new(Some("Textures used by this RigidModel:"));

                compatible_label.set_xalign(0.0);
                compatible_label.set_yalign(0.5);
                compatible_label.set_size_request(100, 0);
                game_label.set_xalign(0.0);
                game_label.set_yalign(0.5);
                game_label.set_size_request(100, 0);
                textures_label.set_xalign(0.0);
                textures_label.set_yalign(0.5);
                textures_label.set_size_request(100, 0);

                game_label.set_hexpand(true);

                // Attach all the stuff already created to the grid.
                packed_file_data_display_grid.attach(&compatible_label, 0, 0, 1, 1);
                packed_file_data_display_grid.attach(&game_label, 1, 0, 1, 1);
                packed_file_data_display_grid.attach(&patch_attila_to_warhammer_button, 2, 0, 1, 1);
                packed_file_data_display_grid.attach(&textures_label, 0, 1, 1, 1);

                packed_file_data_display_scroll.add(&packed_file_data_display_grid);
                app_ui.packed_file_data_display.attach(&packed_file_data_display_scroll, 0, 0, 1, 1);

                // The texture position should never change in the data, so we get the positions of all the
                // textures in the RigidModel.
                let mut texture_paths_index: Vec<u32> = vec![];

                // Check if it's a building/prop/decal.
                if packed_file_decoded.packed_file_data.packed_file_data_lods_data
                    .windows(12)
                    .find(|window: &&[u8]| String::from_utf8_lossy(window) == "rigidmodels/") != None {

                    // If we founded that, it's a building/prop/decal, so we try to get the positions where
                    // his texture paths are.
                    let mut index = 0;
                    while let Some(position) = packed_file_decoded.packed_file_data.packed_file_data_lods_data[index..]
                        .windows(12)
                        .position(|window: &[u8]| String::from_utf8_lossy(window) == "rigidmodels/") {

                        texture_paths_index.push((position + index) as u32);
                        index += position + 1;
                    }
                }

                // If not, check if it's a unit model.
                else if packed_file_decoded.packed_file_data.packed_file_data_lods_data
                    .windows(14)
                    .find(|window: &&[u8]| String::from_utf8_lossy(window) == "variantmeshes/") != None {

                    // If we founded that, it's a building/prop/decal, so we try to get the positions where
                    // his texture paths are.
                    let mut index = 0;
                    while let Some(position) = packed_file_decoded.packed_file_data.packed_file_data_lods_data[index..]
                        .windows(14)
                        .position(|window: &[u8]| String::from_utf8_lossy(window) == "variantmeshes/") {

                        texture_paths_index.push((position + index) as u32);
                        index += position + 1;
                    }
                }

                // If none of these have worked, this is not a decodeable rigidmodel.
                else {
                    return Err(format_err!("Error while trying to get the type of RigidModel (Texture Directories not found)."))
                }

                // Rules to diferentiate between decal, building/prop and units:
                // - texture_paths_index.len() = 1 => decal.
                // - rigidmodel found => building/prop.
                // - variantmeshes found => unit.

                // This will store all the paths, separated by lod.
                let mut texture_paths = vec![];

                // If it's a decal...
                if texture_paths_index.len() == 1 {
                    let mut texture_paths_lod = vec![];
                    let lod_texture_expander = Expander::new(Some(&*format!("Decal Texture Directory")));
                    let lod_texture_expander_grid = Grid::new();
                    lod_texture_expander.add(&lod_texture_expander_grid);
                    lod_texture_expander_grid.set_border_width(6);
                    lod_texture_expander_grid.set_row_spacing(6);
                    lod_texture_expander_grid.set_column_spacing(3);

                    let texture_type = Label::new(Some("Texture Directory:"));
                    texture_type.set_xalign(0.0);
                    texture_type.set_yalign(0.5);
                    texture_type.set_size_request(60, 0);

                    // Then we get it's path, and put it in a gtk::Entry.
                    let texture_path = Entry::new();

                    match decode_string_u8_0padded(
                        &packed_file_decoded.packed_file_data.packed_file_data_lods_data[
                            texture_paths_index[0] as usize..
                            (texture_paths_index[0] as u32 + 255u32) as usize
                        ]
                    ) {
                        Ok(result) => texture_path.get_buffer().set_text(&*result.0),
                        Err(_) =>  return Err(format_err!("Error while trying to get the Decal Texture Directory.")),
                    };

                    texture_path.get_buffer().set_max_length(Some(256u16));
                    texture_path.set_editable(true);
                    texture_path.set_hexpand(true);

                    lod_texture_expander_grid.attach(&texture_type, 0, 0, 1, 1);
                    lod_texture_expander_grid.attach(&texture_path, 1, 0, 1, 1);

                    texture_paths_lod.push(texture_path);
                    texture_paths.push(texture_paths_lod);
                    packed_file_data_display_grid.attach(&lod_texture_expander, 0, 3, 3, 1);
                }

                // If we can subdivide the amount of textures found in the rigidmodel, we have the first
                // one to be the directory, and the other five to be the textures of the lod.
                else if texture_paths_index.len() % 6 == 0 {

                    // We are going to change our lod every 6 indexes...
                    let lods = texture_paths_index.len() / 6;

                    // For each lod...
                    for lod in 0..lods {

                        let mut texture_paths_lod = vec![];
                        let lod_texture_expander = Expander::new(Some(&*format!("Lod {}", lod + 1)));
                        let lod_texture_expander_grid = Grid::new();
                        lod_texture_expander.add(&lod_texture_expander_grid);
                        lod_texture_expander_grid.set_border_width(6);
                        lod_texture_expander_grid.set_row_spacing(6);
                        lod_texture_expander_grid.set_column_spacing(3);

                        // For each texture found (except the first of the group, thats their dir)...
                        for index in 1..6 {

                            // First, we get it's type.
                            let texture_type = Label::new(Some(

                                //0 (Diffuse), 1 (Normal), 11 (Specular), 12 (Gloss), 3/10 (Mask), 5(no idea).
                                match decode_integer_u32(
                                    &packed_file_decoded.packed_file_data.packed_file_data_lods_data[
                                        (texture_paths_index[index + (lod * 6)] - 4) as usize..
                                        (texture_paths_index[index + (lod * 6)] as usize)
                                        ]
                                    ) {
                                    Ok(result) => {
                                        match result {
                                            0 => "Diffuse:",
                                            1 => "Normal:",
                                            11 => "Specular:",
                                            12 => "Gloss:",
                                            3 | 10 => "Mask:",
                                            5 => "Unknown:",
                                            _ => return Err(format_err!("Error while trying to get the Mask Type for a Texture: Unknown Mask Type."))
                                        }
                                    }
                                    Err(error) => return Err(error)
                                }
                            ));

                            texture_type.set_xalign(0.0);
                            texture_type.set_yalign(0.5);

                            // Then we get it's path, and put it in a gtk::Entry.
                            let texture_path = Entry::new();

                            match decode_string_u8_0padded(
                                &packed_file_decoded.packed_file_data.packed_file_data_lods_data[
                                    texture_paths_index[index + (lod * 6)] as usize..
                                    (texture_paths_index[index + (lod * 6)] as u32 + 255u32) as usize
                                ]
                            ) {
                                Ok(result) => texture_path.get_buffer().set_text(&*result.0),
                                Err(_) =>  return Err(format_err!("Error while trying to get the a Texture Path.")),
                            };

                            texture_path.get_buffer().set_max_length(Some(256u16));
                            texture_path.set_editable(true);
                            texture_path.set_hexpand(true);

                            lod_texture_expander_grid.attach(&texture_type, 0, (index - 1) as i32, 1, 1);
                            lod_texture_expander_grid.attach(&texture_path, 1, (index - 1) as i32, 1, 1);

                            texture_paths_lod.push(texture_path);
                        }
                        texture_paths.push(texture_paths_lod);
                        packed_file_data_display_grid.attach(&lod_texture_expander, 0, (lod + 2) as i32, 3, 1);
                    }
                }

                // If not, return error.
                else {
                    return Err(format_err!("Error while trying to decode the selected RigidModel: Irregular amount of Textures per lod."))
                }

                // If we reached this point, show it all.
                app_ui.packed_file_data_display.show_all();

                // Get the needed stuff into the decoded view.
                let decoded_view = PackedFileRigidModelDataView {
                    texture_paths_index,
                    texture_paths,
                };

                // When we destroy the `ScrolledWindow`, we need to tell the program we no longer have an open PackedFile.
                packed_file_data_display_scroll.connect_destroy(clone!(
                    is_packedfile_opened => move |_| {
                        *is_packedfile_opened.borrow_mut() = false;
                    }
                ));

                // Get the decoded PackedFile into a Rc<RefCell<RigidModel>> so we can get it into closures.
                let packed_file_decoded = Rc::new(RefCell::new(packed_file_decoded));

                // When we hit the "Patch to Warhammer 1&2" button.
                patch_attila_to_warhammer_button.connect_button_release_event(clone!(
                    app_ui,
                    pack_file,
                    packed_file_decoded,
                    packed_file_decoded_index => move |patch_button, _| {

                    // Patch the RigidModel...
                    let packed_file_data_patch_result = patch_rigid_model_attila_to_warhammer(&mut *packed_file_decoded.borrow_mut());
                    match packed_file_data_patch_result {
                        Ok(result) => {

                            // Disable the button and change his game...
                            patch_button.set_sensitive(false);
                            game_label.set_text("Warhammer 1&2");

                            // Save the changes to the PackFile....
                            let mut success = false;
                            match update_packed_file_data_rigid(
                                &*packed_file_decoded.borrow(),
                                &mut *pack_file.borrow_mut(),
                                packed_file_decoded_index
                            ) {
                                Ok(_) => {
                                    success = true;
                                    show_dialog(&app_ui.window, true, result);
                                },
                                Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                            }

                            // If it works, set it as modified.
                            if success {
                                set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                            }
                        },
                        Err(error) => show_dialog(&app_ui.window, false, error.cause()),
                    }
                    Inhibit(false)
                }));

                // When we change any of the Paths...
                // TODO: It's extremely slow with big models. Need to find a way to fix it.
                for lod in &decoded_view.texture_paths {
                    for texture_path in lod {
                        texture_path.connect_changed(clone!(
                            pack_file,
                            packed_file_decoded,
                            decoded_view,
                            app_ui,
                            packed_file_decoded_index => move |_| {

                                // Get the data from the View...
                                let new_data = match PackedFileRigidModelDataView::return_data_from_data_view(
                                    &decoded_view,
                                    &mut (*packed_file_decoded.borrow_mut()).packed_file_data.packed_file_data_lods_data.to_vec()
                                ) {
                                    Ok(new_data) => new_data,
                                    Err(error) => {
                                        let message = format_err!("Error while trying to save changes to a RigidModel: {}", error.cause());
                                        return show_message_in_statusbar(&app_ui.status_bar, message)
                                    }
                                };

                                // Save it encoded into the opened RigidModel...
                                packed_file_decoded.borrow_mut().packed_file_data.packed_file_data_lods_data = new_data;

                                // And then into the PackFile.
                                let success;
                                match update_packed_file_data_rigid(
                                    &*packed_file_decoded.borrow(),
                                    &mut *pack_file.borrow_mut(),
                                    packed_file_decoded_index
                                ) {
                                    Ok(_) => { success = true },
                                    Err(error) => {
                                        let message = format_err!("Error while trying to save changes to a RigidModel: {}", error.cause());
                                        return show_message_in_statusbar(&app_ui.status_bar, message)
                                    }
                                }

                                // If it works, set it as modified.
                                if success {
                                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                                }
                            }
                        ));
                    }
                }

                // Return success.
                Ok(())
            }
            Err(error) => Err(error)
        }
    }

    /// This function get the texture path entries of a RigidModel from the UI and saves them into the
    /// opened RigidModel.
    pub fn return_data_from_data_view(&self, packed_file_data_lods_data: &mut Vec<u8>) -> Result<Vec<u8>, Error> {

        // If it's a decal...
        if self.texture_paths_index.len() == 1 {

            // We just replace the text in the position we have and return the changed vector.
            let new_texture_path = encode_string_u8_0padded(&(self.texture_paths[0][0].get_text().unwrap(), 256))?;

            packed_file_data_lods_data.splice(
                (self.texture_paths_index[0] as usize)..((self.texture_paths_index[0] + 256) as usize),
                new_texture_path.iter().cloned());
        }

        // If it's a building/prop/unit...
        else {

            // Get the amount of lods...
            let lods = self.texture_paths_index.len() / 6;
            for lod in 0..lods {

                // For each texture (we skip the texture directory)...
                for texture in 1..6 {

                    // We get the new texture, and replace the old one with the new one.
                    let new_texture_path = encode_string_u8_0padded(&(self.texture_paths[lod][texture - 1].get_text().unwrap(), 256))?;

                    packed_file_data_lods_data.splice(
                        (self.texture_paths_index[texture + (lod * 6)] as usize)..((self.texture_paths_index[texture + (lod * 6)] + 256) as usize),
                        new_texture_path.iter().cloned());
                }
            }
        }

        Ok(packed_file_data_lods_data.to_vec())
    }
}
