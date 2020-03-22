//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
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

use qt_gui::QStandardItem;
use qt_gui::QIcon;

use qt_core::QString;

use cpp_core::Ref;

use std::sync::atomic::AtomicPtr;

use rpfm_lib::packedfile::{text, text::TextType};

use crate::RPFM_PATH;
use crate::TREEVIEW_ICONS;
use crate::utils::atomic_from_cpp_box;
use crate::utils::ref_from_atomic_ref;

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
    pub packfile_editable: AtomicPtr<QIcon>,
    pub packfile_locked: AtomicPtr<QIcon>,
    pub folder: AtomicPtr<QIcon>,

    // For generic files.
    pub file: AtomicPtr<QIcon>,

    // For tables and loc files.
    pub table: AtomicPtr<QIcon>,

    // For images.
    pub image_generic: AtomicPtr<QIcon>,
    pub image_png: AtomicPtr<QIcon>,
    pub image_jpg: AtomicPtr<QIcon>,

    // For text files.
    pub text_generic: AtomicPtr<QIcon>,
    pub text_csv: AtomicPtr<QIcon>,
    pub text_html: AtomicPtr<QIcon>,
    pub text_txt: AtomicPtr<QIcon>,
    pub text_xml: AtomicPtr<QIcon>,

    // For rigidmodels.
    pub rigid_model: AtomicPtr<QIcon>,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `IconType`.
impl IconType {

    /// This function is used to set the icon of an Item in the `TreeView` depending on his type.
    ///
    /// TODO: Find a way to abstract this into the PackedFileType thing.
    pub fn set_icon_to_item_safe(&self, item: &mut QStandardItem) {
        let icon = ref_from_atomic_ref(match self {

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
                else if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(extension, _)| packed_file_name.ends_with(extension)) {
                    match text_type {
                        TextType::Html => &TREEVIEW_ICONS.text_xml,
                        TextType::Xml => &TREEVIEW_ICONS.text_xml,
                        TextType::Lua => &TREEVIEW_ICONS.text_generic,
                        TextType::Cpp => &TREEVIEW_ICONS.text_generic,
                        TextType::Plain => &TREEVIEW_ICONS.text_txt,
                        TextType::Markdown => &TREEVIEW_ICONS.text_txt,
                    }
                }

                // If it ends in any of these, it's an image.
                else if packed_file_name.ends_with(".jpg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".jpeg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".tga") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".dds") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".png") { &TREEVIEW_ICONS.image_png }

                // Otherwise, it's a generic file.
                else { &TREEVIEW_ICONS.file }
            }
        });
        unsafe { item.set_icon(icon) };
    }

    /// This function is used to get the icon corresponding to an IconType.
    pub fn get_icon_from_path(&self) -> Ref<QIcon> {
        ref_from_atomic_ref(match self {

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
                else if let Some((_, text_type)) = text::EXTENSIONS.iter().find(|(extension, _)| packed_file_name.ends_with(extension)) {
                    match text_type {
                        TextType::Html => &TREEVIEW_ICONS.text_xml,
                        TextType::Xml => &TREEVIEW_ICONS.text_xml,
                        TextType::Lua => &TREEVIEW_ICONS.text_generic,
                        TextType::Cpp => &TREEVIEW_ICONS.text_generic,
                        TextType::Plain => &TREEVIEW_ICONS.text_txt,
                        TextType::Markdown => &TREEVIEW_ICONS.text_txt,
                    }
                }

                // If it ends in any of these, it's an image.
                else if packed_file_name.ends_with(".jpg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".jpeg") { &TREEVIEW_ICONS.image_jpg }
                else if packed_file_name.ends_with(".tga") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".dds") { &TREEVIEW_ICONS.image_generic }
                else if packed_file_name.ends_with(".png") { &TREEVIEW_ICONS.image_png }

                // Otherwise, it's a generic file.
                else { &TREEVIEW_ICONS.file }
            }
        })
    }
}

/// Implementation of `Icons`.
impl Icons {

    /// This function creates a list of icons from certain paths in disk.
    pub unsafe fn new() -> Self {

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

        let mut icon_rigid_model_path = rpfm_path_string;

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
            packfile_editable: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_packfile_editable_path))),
            packfile_locked: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_packfile_locked_path))),
            folder: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_folder_path))),
            file: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_file_path))),
            table: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_table_path))),
            image_generic: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_image_generic_path))),
            image_png: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_image_png_path))),
            image_jpg: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_image_jpg_path))),
            text_generic: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_text_generic_path))),
            text_csv: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_text_csv_path))),
            text_html: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_text_html_path))),
            text_txt: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_text_txt_path))),
            text_xml: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_text_xml_path))),
            rigid_model: atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(icon_rigid_model_path))),
        }
    }
}
