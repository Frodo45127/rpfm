//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*! 
This module contains the code to load the icons used in the `TreeView`.
!*/

use qt_gui::standard_item::StandardItem;
use qt_gui::icon::Icon;

use crate::QString;
use crate::RPFM_PATH;
use crate::TREEVIEW_ICONS;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This enum contains the variants used to decide which icon corresponds to which item in the `TreeView`,
pub enum IconType {

    // For normal PackFiles. `true` if it's editable, `false` if it's read-only.
    PackFile(bool),

    // For folders.
    Folder,

    // For files. Includes the path without the PackFile's name on it.
    File(Vec<String>),
}

/// This struct is used to hold all the QIcons used by the `TreeView`s.
pub struct Icons {
    pub packfile_editable: Icon,
    pub packfile_locked: Icon,
    pub folder: Icon,

    // For generic files.
    pub file: Icon,

    // For tables and loc files.
    pub table: Icon,

    // For images.
    pub image_generic: Icon,
    pub image_png: Icon,
    pub image_jpg: Icon,

    // For text files.
    pub text_generic: Icon,
    pub text_csv: Icon,
    pub text_html: Icon,
    pub text_txt: Icon,
    pub text_xml: Icon,

    // For rigidmodels.
    pub rigid_model: Icon,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `IconType`.
impl IconType {

    /// This function is used to set the icon of an Item in the `TreeView` depending on his type.
    ///
    /// TODO: Find a way to abstract this into the PackedFileType thing.
    pub fn set_icon_to_item_safe(&self, item: &mut StandardItem) {
        let icon = match self {

            // For PackFiles.
            IconType::PackFile(editable) => {
                if *editable { &TREEVIEW_ICONS.packfile_editable }
                else { &TREEVIEW_ICONS.packfile_locked }
            },

            // For folders.
            IconType::Folder => &TREEVIEW_ICONS.folder,

            // For files.
            IconType::File(path) => {

                // Get the name of the file.
                let packed_file_name = path.last().unwrap();

                // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
                if path[0] == "db" { &TREEVIEW_ICONS.table }

                // If it ends in ".loc", it's a localisation PackedFile.
                else if packed_file_name.ends_with(".loc") { &TREEVIEW_ICONS.table }

                // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
                else if packed_file_name.ends_with(".rigid_model_v2") { &TREEVIEW_ICONS.rigid_model }

                // If it ends in any of these, it's a plain text PackedFile.
                else if packed_file_name.ends_with(".lua") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".xml") { &TREEVIEW_ICONS.text_xml }
                else if packed_file_name.ends_with(".xml.shader") { &TREEVIEW_ICONS.text_xml }
                else if packed_file_name.ends_with(".xml.material") { &TREEVIEW_ICONS.text_xml }
                else if packed_file_name.ends_with(".variantmeshdefinition") { &TREEVIEW_ICONS.text_xml }
                else if packed_file_name.ends_with(".environment") { &TREEVIEW_ICONS.text_xml }
                else if packed_file_name.ends_with(".lighting") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".wsmodel") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".csv") { &TREEVIEW_ICONS.text_csv }
                else if packed_file_name.ends_with(".tsv") { &TREEVIEW_ICONS.text_csv }
                else if packed_file_name.ends_with(".inl") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".battle_speech_camera") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".bob") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".cindyscene") { &TREEVIEW_ICONS.text_generic }
                else if packed_file_name.ends_with(".cindyscenemanager") { &TREEVIEW_ICONS.text_generic }
                //else if packed_file_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
                else if packed_file_name.ends_with(".txt") { &TREEVIEW_ICONS.text_txt }

                // If it ends in any of these, it's an image.
                else if packed_file_name.ends_with(".jpg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".jpeg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".tga") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".dds") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".png") { &TREEVIEW_ICONS.image_png }

                // Otherwise, it's a generic file.
                else { &TREEVIEW_ICONS.file }
            }
        };
        item.set_icon(icon);
    }

    /// This function is used to set the icon of an Item in the `TreeView` depending on his type.
    ///
    /// TODO: Find a way to abstract this into the PackedFileType thing.
    pub fn set_icon_to_item_unsafe(&self, item: *mut StandardItem) {
    	let item = unsafe { item.as_mut().unwrap() };
        self.set_icon_to_item_safe(item);
    }
}

/// Implementation of `Icons`.
impl Icons {

    /// This function creates a list of icons from certain paths in disk.
    pub fn new() -> Self {

        // Get the Path as a String, so Qt can understand it.
        let rpfm_path_string = RPFM_PATH.to_string_lossy().as_ref().to_string();

        // Prepare the path for the icons of the TreeView.
        let mut icon_packfile_editable_path = rpfm_path_string.to_owned();
        let mut icon_packfile_locked_path = rpfm_path_string.to_owned();
        let mut icon_folder_path = rpfm_path_string.to_owned();
        let mut icon_file_path = rpfm_path_string.to_owned();

        let mut icon_table_path = rpfm_path_string.to_owned();

        let mut icon_image_generic_path = rpfm_path_string.to_owned();
        let mut icon_image_png_path = rpfm_path_string.to_owned();
        let mut icon_image_jpg_path = rpfm_path_string.to_owned();

        let mut icon_text_generic_path = rpfm_path_string.to_owned();
        let mut icon_text_csv_path = rpfm_path_string.to_owned();
        let mut icon_text_html_path = rpfm_path_string.to_owned();
        let mut icon_text_txt_path = rpfm_path_string.to_owned();
        let mut icon_text_xml_path = rpfm_path_string.to_owned();

        let mut icon_rigid_model_path = rpfm_path_string.to_owned();

        // Get the Icons for each type of Item.
        icon_packfile_editable_path.push_str("/img/packfile_editable.svg");
        icon_packfile_locked_path.push_str("/img/packfile_locked.svg");
        icon_folder_path.push_str("/img/folder.svg");
        icon_file_path.push_str("/img/file.svg");

        icon_table_path.push_str("/img/database.svg");

        icon_image_generic_path.push_str("/img/generic_image.svg");
        icon_image_png_path.push_str("/img/png.svg");
        icon_image_jpg_path.push_str("/img/jpg.svg");

        icon_text_generic_path.push_str("/img/generic_text.svg");
        icon_text_csv_path.push_str("/img/csv.svg");
        icon_text_html_path.push_str("/img/html.svg");
        icon_text_txt_path.push_str("/img/txt.svg");
        icon_text_xml_path.push_str("/img/xml.svg");

        icon_rigid_model_path.push_str("/img/rigid_model.svg");

        // Get the Icons in QIcon format.
        Self {
            packfile_editable: Icon::new(&QString::from_std_str(icon_packfile_editable_path)),
            packfile_locked: Icon::new(&QString::from_std_str(icon_packfile_locked_path)),
            folder: Icon::new(&QString::from_std_str(icon_folder_path)),
            file: Icon::new(&QString::from_std_str(icon_file_path)),

            table: Icon::new(&QString::from_std_str(icon_table_path)),

            image_generic: Icon::new(&QString::from_std_str(icon_image_generic_path)),
            image_png: Icon::new(&QString::from_std_str(icon_image_png_path)),
            image_jpg: Icon::new(&QString::from_std_str(icon_image_jpg_path)),

            text_generic: Icon::new(&QString::from_std_str(icon_text_generic_path)),
            text_csv: Icon::new(&QString::from_std_str(icon_text_csv_path)),
            text_html: Icon::new(&QString::from_std_str(icon_text_html_path)),
            text_txt: Icon::new(&QString::from_std_str(icon_text_txt_path)),
            text_xml: Icon::new(&QString::from_std_str(icon_text_xml_path)),

            rigid_model: Icon::new(&QString::from_std_str(icon_rigid_model_path)),
        }
    }
}
