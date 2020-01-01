//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the UI when decoding RigidModel PackedFiles.

use qt_widgets::widget::Widget;
use qt_widgets::group_box::GroupBox;
use qt_widgets::tab_widget::TabWidget;

use qt_core::connection::Signal;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};

use crate::AppUI;
use crate::Commands;
use crate::Data;
use rpfm_lib::common::coding_helpers::*;
use crate::communications::*;
use crate::ui::*;
use rpfm_error::{ErrorKind, Result};

/// Struct PackedFileRigidModelDataView: contains all the stuff we need to give to the program to
/// show a TreeView with the data of a RigidModel file, allowing us to manipulate it.
pub struct PackedFileRigidModelDataView {
    pub save_changes: SlotNoArgs<'static>,
    pub patch_rigid_model: SlotNoArgs<'static>,
}

/// Implementation of "PackedFileRigidModelDataView".
impl PackedFileRigidModelDataView {

    /// This function creates a "view" with the PackedFile's View as father and returns a
    /// `PackedFileRigidModelDataView` with all his slots.
    pub fn create_data_view(
        sender_qt: &Sender<Commands>,
        sender_qt_data: &Sender<Data>,
        receiver_qt: &Rc<RefCell<Receiver<Data>>>,
        app_ui: &AppUI,
        layout: *mut GridLayout,
        packed_file_path: &Rc<RefCell<Vec<String>>>,
    ) -> Result<Self> {

        // Get the data of the PackedFile.
        sender_qt.send(Commands::DecodePackedFileRigidModel).unwrap();
        sender_qt_data.send(Data::VecString(packed_file_path.borrow().to_vec())).unwrap();
        let packed_file = match check_message_validity_recv2(&receiver_qt) {
            Data::RigidModel(data) => data,
            Data::Error(error) => return Err(error),
            _ => panic!(THREADS_MESSAGE_ERROR),
        };

        // Create the "Info" Frame.
        let info_frame = GroupBox::new(&QString::from_std_str("RigidModel's Info")).into_raw();
        let info_layout = GridLayout::new().into_raw();
        unsafe { info_frame.as_mut().unwrap().set_layout(info_layout as *mut Layout); }
        unsafe { info_layout.as_mut().unwrap().set_column_stretch(2, 10); }

        // Create the info labels.
        let rigid_model_version_label = Label::new(&QString::from_std_str("Version:")).into_raw();
        let rigid_model_compatible_label = Label::new(&QString::from_std_str("Compatible with:")).into_raw();

        let rigid_model_version_decoded_label = Label::new(&QString::from_std_str(format!("{}", packed_file.packed_file_header.packed_file_header_model_type))).into_raw();
        let rigid_model_compatible_decoded_label = Label::new(&QString::from_std_str(
            if packed_file.packed_file_header.packed_file_header_model_type == 6 { "Attila" }
            else if packed_file.packed_file_header.packed_file_header_model_type == 7 { "Warhammer 1&2" }
            else { "Unknonw"}
        )).into_raw();

        unsafe { info_layout.as_mut().unwrap().add_widget((rigid_model_version_label as *mut Widget, 0, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((rigid_model_version_decoded_label as *mut Widget, 0, 1, 1, 1)); }

        unsafe { info_layout.as_mut().unwrap().add_widget((rigid_model_compatible_label as *mut Widget, 1, 0, 1, 1)); }
        unsafe { info_layout.as_mut().unwrap().add_widget((rigid_model_compatible_decoded_label as *mut Widget, 1, 1, 1, 1)); }

        // Create the "Patch" button.
        let patch_attila_to_warhammer_button = PushButton::new(&QString::from_std_str("Patch RigidModel")).into_raw();
        unsafe { info_layout.as_mut().unwrap().add_widget((patch_attila_to_warhammer_button as *mut Widget, 0, 3, 1, 1)); }

        // If the RigidModel is not type 6, disable the button.
        if packed_file.packed_file_header.packed_file_header_model_type != 6 { unsafe { patch_attila_to_warhammer_button.as_mut().unwrap().set_enabled(false); } }

        // The texture position should never change in the data, so we get the positions of all the textures in the RigidModel.
        let mut texture_paths_index: Vec<u32> = vec![];

        // Check if it's a building/prop/decal.
        if packed_file.packed_file_data.packed_file_data_lods_data
            .windows(12)
            .find(|window: &&[u8]| String::from_utf8_lossy(window) == "rigidmodels/") != None {

            // If we founded that, it's a building/prop/decal, so we try to get the positions where
            // his texture paths are.
            let mut index = 0;
            while let Some(position) = packed_file.packed_file_data.packed_file_data_lods_data[index..]
                .windows(12)
                .position(|window: &[u8]| String::from_utf8_lossy(window) == "rigidmodels/") {

                texture_paths_index.push((position + index) as u32);
                index += position + 1;
            }
        }

        // If not, check if it's a unit model.
        else if packed_file.packed_file_data.packed_file_data_lods_data
            .windows(14)
            .find(|window: &&[u8]| String::from_utf8_lossy(window) == "variantmeshes/") != None {

            // If we founded that, it's a building/prop/decal, so we try to get the positions where
            // his texture paths are.
            let mut index = 0;
            while let Some(position) = packed_file.packed_file_data.packed_file_data_lods_data[index..]
                .windows(14)
                .position(|window: &[u8]| String::from_utf8_lossy(window) == "variantmeshes/") {

                texture_paths_index.push((position + index) as u32);
                index += position + 1;
            }
        }

        // If none of these have worked, this is not a decodeable rigidmodel.
        else { return Err(ErrorKind::RigidModelTextureDirectoryNotFound)? }

        // Rules to diferentiate between decal, building/prop and units:
        // - texture_paths_index.len() = 1 => decal.
        // - rigidmodel found => building/prop.
        // - variantmeshes found => unit.

        // This will store all the paths, separated by lod.
        let mut texture_paths = vec![];

        // If it's a decal...
        if texture_paths_index.len() == 1 {

            // Create the TabWidget.
            let tabs = TabWidget::new().into_raw();

            // Create the Widget for the Tab.
            let tab_widget = Widget::new().into_raw();
            let tab_widget_layout = GridLayout::new().into_raw();
            unsafe { tab_widget.as_mut().unwrap().set_layout(tab_widget_layout as *mut Layout); }
            unsafe { tab_widget_layout.as_mut().unwrap().set_row_stretch(6, 10); }

            // Create the LineEdit for the Texture's Directory.
            let texture_directory_label = Label::new(&QString::from_std_str("Texture Directory:")).into_raw();
            let texture_directory_line_edit = LineEdit::new(()).into_raw();

            unsafe { tab_widget_layout.as_mut().unwrap().add_widget((texture_directory_label as *mut Widget, 0, 0, 1, 1)); }
            unsafe { tab_widget_layout.as_mut().unwrap().add_widget((texture_directory_line_edit as *mut Widget, 0, 1, 1, 1)); }

            // Populate the LineEdit.
            match decode_string_u8_0padded(
                &packed_file.packed_file_data.packed_file_data_lods_data[
                    texture_paths_index[0] as usize..
                    (texture_paths_index[0] as u32 + 255u32) as usize
                ]
            ) {
                Ok(result) => unsafe { texture_directory_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(result.0)) },
                Err(_) =>  return Err(ErrorKind::RigidModelDecalTextureDirectoryNotFound)?,
            };

            // Add the Widget to the TabWidget.
            unsafe { tabs.as_mut().unwrap().add_tab((tab_widget, &QString::from_std_str("Decal Texture Directory"))); }

            // Add everything to the PackedFile's View.
            unsafe { layout.as_mut().unwrap().add_widget((info_frame as *mut Widget, 0, 0, 1, 1)); }
            unsafe { layout.as_mut().unwrap().add_widget((tabs as *mut Widget, 1, 0, 1, 1)); }

            // Add the LineEdit to the List.
            texture_paths.push(vec![texture_directory_line_edit]);
        }

        // If we can subdivide the amount of textures found in the rigidmodel, we have the first
        // one to be the directory, and the other five to be the textures of the lod.
        else if texture_paths_index.len() % 6 == 0 {

            // Create the TabWidget.
            let tabs = TabWidget::new().into_raw();

            // We are going to change our lod every 6 indexes...
            let lods = texture_paths_index.len() / 6;

            // For each lod...
            for lod in 0..lods {

                // Create the Widget for the Tab.
                let tab_widget = Widget::new().into_raw();
                let tab_widget_layout = GridLayout::new().into_raw();
                unsafe { tab_widget.as_mut().unwrap().set_layout(tab_widget_layout as *mut Layout); }
                unsafe { tab_widget_layout.as_mut().unwrap().set_row_stretch(6, 10); }

                // Create the list of textures per lod.
                let mut textures_lod = vec![];

                // For each texture found (except the first of the group, thats their dir)...
                for index in 1..6 {

                    // Create the Labels for each texture in that Lod.
                    let texture_type = Label::new(&QString::from_std_str(

                        //0 (Diffuse), 1 (Normal), 11 (Specular), 12 (Gloss), 3/10 (Mask), 5(no idea).
                        match decode_integer_u32(
                            &packed_file.packed_file_data.packed_file_data_lods_data[
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
                                    _ => return Err(ErrorKind::RigidModelUnknownMaskTypeFound)?
                                }
                            }
                            Err(error) => return Err(error)
                        }
                    )).into_raw();

                    // Create the LineEdit for each texture.
                    let texture_line_edit = LineEdit::new(()).into_raw();

                    // Add them to the Widget.
                    unsafe { tab_widget_layout.as_mut().unwrap().add_widget((texture_type as *mut Widget, index as i32, 0, 1, 1)); }
                    unsafe { tab_widget_layout.as_mut().unwrap().add_widget((texture_line_edit as *mut Widget, index as i32, 1, 1, 1)); }

                    // Populate the LineEdit.
                    match decode_string_u8_0padded(
                        &packed_file.packed_file_data.packed_file_data_lods_data[
                            texture_paths_index[index + (lod * 6)] as usize..
                            (texture_paths_index[index + (lod * 6)] as u32 + 255u32) as usize
                        ]
                    ) {
                        Ok(result) => unsafe { texture_line_edit.as_mut().unwrap().set_text(&QString::from_std_str(result.0)) },
                        Err(_) =>  return Err(ErrorKind::RigidModelDecalTextureDirectoryNotFound)?,
                    };

                    // Add the Widget to the TabWidget.
                    unsafe { tabs.as_mut().unwrap().add_tab((tab_widget, &QString::from_std_str(format!("Lod {}", lod + 1)))); }

                    // Add the LineEdit to the List.
                    textures_lod.push(texture_line_edit);
                }

                // Add the Lod to the list.
                texture_paths.push(textures_lod);
            }

            // Add everything to the PackedFile's View.
            unsafe { layout.as_mut().unwrap().add_widget((info_frame as *mut Widget, 0, 0, 1, 1)); }
            unsafe { layout.as_mut().unwrap().add_widget((tabs as *mut Widget, 1, 0, 1, 1)); }
        }

        //-------------------------------------------------------------------------------//
        // Slots and actions...
        //-------------------------------------------------------------------------------//
        // Put the PackedFile into a Rc<RefCell> so we can move it into the closures.
        let packed_file = Rc::new(RefCell::new(packed_file));

        // Slots...
        let slots = Self {

            // Slot to save all the changes from the texts.
            save_changes: SlotNoArgs::new(clone!(
                packed_file_path,
                texture_paths,
                texture_paths_index,
                packed_file,
                app_ui,
                receiver_qt,
                sender_qt,
                sender_qt_data => move || {

                    // Try to update the RigidModel's data from the LineEdits.
                    if let Err(error) = Self::return_data_from_data_view(
                        &texture_paths,
                        &texture_paths_index,
                        &mut packed_file.borrow_mut().packed_file_data.packed_file_data_lods_data
                    ) {

                        // If there was an error, report it.
                        return show_dialog(app_ui.window, false, error.kind());
                    }

                    // Tell the background thread to start saving the PackedFile.
                    sender_qt.send(Commands::EncodePackedFileRigidModel).unwrap();
                    sender_qt_data.send(Data::RigidModelVecString((packed_file.borrow().clone(), packed_file_path.borrow().to_vec()))).unwrap();

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

            // Slot to patch the RigidModel for Warhammer.
            patch_rigid_model: SlotNoArgs::new(clone!(
                packed_file_path,
                packed_file,
                app_ui,
                receiver_qt,
                sender_qt,
                sender_qt_data,
                receiver_qt => move || {

                    // Send the data to the background to try to patch the rigidmodel.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                    sender_qt.send(Commands::PatchAttilaRigidModelToWarhammer).unwrap();
                    sender_qt_data.send(Data::RigidModelVecString((packed_file.borrow().clone(), packed_file_path.borrow().to_vec()))).unwrap();

                    // Check what response we got.
                    match check_message_validity_tryrecv(&receiver_qt) {

                        // If it's success....
                        Data::RigidModel(response) => {

                            // Get the RigidModel data.
                            *packed_file.borrow_mut() = response;

                            // Reflect the changes in the UI.
                            unsafe { rigid_model_version_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str("7")); }
                            unsafe { rigid_model_compatible_decoded_label.as_mut().unwrap().set_text(&QString::from_std_str("Warhammer 1&2")); }
                            unsafe { patch_attila_to_warhammer_button.as_mut().unwrap().set_enabled(false); }

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

                        // If we got an error, report it.
                        Data::Error(error) => show_dialog(app_ui.window, false, error),

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_MESSAGE_ERROR),
                    }

                    // Re-enable the Main Window.
                    unsafe { (app_ui.window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            )),
        };

        // Actions to trigger the "Save Changes" slot.
        for lod in texture_paths {
            for line_edit in lod {
                unsafe { line_edit.as_mut().unwrap().signals().editing_finished().connect(&slots.save_changes); }
            }
        }

        // Action to trigger the "Patch RigidModel" slot.
        unsafe { patch_attila_to_warhammer_button.as_mut().unwrap().signals().released().connect(&slots.patch_rigid_model); }

        // Return the slots.
        Ok(slots)
    }

    /// This function get the texture path entries of a RigidModel from the UI and saves them into the
    /// opened RigidModel.
    pub fn return_data_from_data_view(
        line_edits: &[Vec<*mut LineEdit>],
        texture_paths_index: &[u32],
        packed_file_data_lods_data: &mut Vec<u8>
    ) -> Result<()> {

        // If it's a decal...
        if line_edits.len() == 1 && line_edits[0].len() == 1 {

            // We just replace the text in the position we have and return the changed vector.
            let new_texture_path;
            unsafe { new_texture_path = encode_string_u8_0padded(&(line_edits[0][0].as_mut().unwrap().text().to_std_string(), 256))?; }

            packed_file_data_lods_data.splice(
                (texture_paths_index[0] as usize)..((texture_paths_index[0] + 256) as usize),
                new_texture_path.iter().cloned());
        }

        // If it's a building/prop/unit...
        else {

            // Get the amount of lods...
            let lods = texture_paths_index.len() / 6;
            for lod in 0..lods {

                // For each texture (we skip the texture directory)...
                for texture in 1..6 {

                    // We get the new texture, and replace the old one with the new one.
                    let new_texture_path;
                    unsafe { new_texture_path = encode_string_u8_0padded(&(line_edits[lod][texture - 1].as_mut().unwrap().text().to_std_string(), 256))?; }

                    packed_file_data_lods_data.splice(
                        (texture_paths_index[texture + (lod * 6)] as usize)..((texture_paths_index[texture + (lod * 6)] + 256) as usize),
                        new_texture_path.iter().cloned());
                }
            }
        }

        Ok(())
    }
}
