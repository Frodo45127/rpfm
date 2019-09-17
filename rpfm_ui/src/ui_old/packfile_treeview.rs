//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions related to the Packfile's TreeViews.

use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::variant::Variant;
use qt_core::qt::GlobalColor;

use serde_derive::{Serialize, Deserialize};

use std::cell::RefCell;
use std::rc::Rc;

use crate::RPFM_PATH;
use crate::TREEVIEW_ICONS;
use crate::IS_MODIFIED;
use crate::AppUI;
use crate::QString;
use crate::ui::*;
use rpfm_lib::packfile::PathType;

//----------------------------------------------------------------//
// Enums and Structs for the TreeView.
//----------------------------------------------------------------//

/// Enum `TreeViewOperation`: This enum has the different possible operations we want to do over a TreeView. The options are:
/// - `Build`: Build the entire TreeView from nothing. Requires a bool, depending if the PackFile is editable or not.
/// - `Add`: Add one or more Files/Folders to the TreeView. Requires the TreePathType to add to the TreeView.
/// - `Delete`: Remove the Files/Folders corresponding to the TreePathTypes we provide from the TreeView. It requires the TreePathTypes of whatever you want to delete.
/// - `Modify`: Set the provided paths as modified. It requires the TreePathTypes of whatever you want to modify.
/// - `Rename`: Change the name of a File/Folder from the TreeView. Requires the list of TreePathType you want to rename and their new name.
/// - `MarkAlwaysModified`: Mark an item as "Always Modified" so it cannot be marked as unmodified by an undo operation.
/// - `Undo`: Resets the state of a Packedfile to 0, or unmodified.
/// - `Clean`: Remove all status and color from the entire TreeView.
/// - `Clear`: Remove all the stuff from the TreeView.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {
    Build(bool),
    Add(Vec<TreePathType>),
    Delete(Vec<TreePathType>),
    Modify(Vec<TreePathType>),
    Rename(Vec<(TreePathType, String)>),
    MarkAlwaysModified(Vec<TreePathType>),
    Undo(Vec<TreePathType>),
    Clean,
    Clear,
}

/// Enum `TreeViewType`: This enum represents the different basic types of an element in the TreeView.
/// None of the paths have the PackFile on them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreePathType {
    File(Vec<String>),
    Folder(Vec<String>),
    PackFile,
    None,
}

/// Struct `Icons`. This struct is used to hold all the Qt Icons used by the TreeView. This is generated
/// everytime we call "update_treeview", but ideally we should move it to on start.
pub struct Icons {
    pub packfile_editable: icon::Icon,
    pub packfile_locked: icon::Icon,
    pub folder: icon::Icon,

    // For generic files.
    pub file: icon::Icon,

    // For tables and loc files.
    pub table: icon::Icon,

    // For images.
    pub image_generic: icon::Icon,
    pub image_png: icon::Icon,
    pub image_jpg: icon::Icon,

    // For text files.
    pub text_generic: icon::Icon,
    pub text_csv: icon::Icon,
    pub text_html: icon::Icon,
    pub text_txt: icon::Icon,
    pub text_xml: icon::Icon,

    // For rigidmodels.
    pub rigid_model: icon::Icon,
}

//----------------------------------------------------------------//
// Implementation of `TreePathType`.
//----------------------------------------------------------------//

/// Custom implementation of "PartialEq" for "TreePathType", so we don't need to match each time while
/// want to compare two TreePathType.
impl PartialEq for TreePathType {
    fn eq(&self, other: &TreePathType) -> bool {
        match (self, other) {
            (&TreePathType::File(_), &TreePathType::File(_)) |
            (&TreePathType::Folder(_), &TreePathType::Folder(_)) |
            (&TreePathType::PackFile, &TreePathType::PackFile) |
            (&TreePathType::None, &TreePathType::None) => true,
            _ => false,
        }
    }
}

/// Implementation of TreePathType to get it from a PathType.
impl From<&PathType> for TreePathType {
    fn from(path_type: &PathType) -> TreePathType {
        match path_type {
            PathType::File(ref path) => TreePathType::File(path.to_vec()),
            PathType::Folder(ref path) => TreePathType::Folder(path.to_vec()),
            PathType::PackFile => TreePathType::PackFile,
            PathType::None => TreePathType::None,
        }
    }
}

/// Implementation of PathType to get it from a TreePathType.
impl From<&TreePathType> for PathType {
    fn from(tree_path_type: &TreePathType) -> PathType {
        match tree_path_type {
            TreePathType::File(ref path) => PathType::File(path.to_vec()),
            TreePathType::Folder(ref path) => PathType::Folder(path.to_vec()),
            TreePathType::PackFile => PathType::PackFile,
            TreePathType::None => PathType::None,
        }
    }
}


//----------------------------------------------------------------//
// Implementation of `Icons`.
//----------------------------------------------------------------//

/// Implementation of "Icons".
impl Icons {

    /// This function creates a list of Icons from certain paths in disk.
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

        // Get the Icons in Qt Icon format.
        Self {
            packfile_editable: icon::Icon::new(&QString::from_std_str(icon_packfile_editable_path)),
            packfile_locked: icon::Icon::new(&QString::from_std_str(icon_packfile_locked_path)),
            folder: icon::Icon::new(&QString::from_std_str(icon_folder_path)),
            file: icon::Icon::new(&QString::from_std_str(icon_file_path)),

            table: icon::Icon::new(&QString::from_std_str(icon_table_path)),

            image_generic: icon::Icon::new(&QString::from_std_str(icon_image_generic_path)),
            image_png: icon::Icon::new(&QString::from_std_str(icon_image_png_path)),
            image_jpg: icon::Icon::new(&QString::from_std_str(icon_image_jpg_path)),

            text_generic: icon::Icon::new(&QString::from_std_str(icon_text_generic_path)),
            text_csv: icon::Icon::new(&QString::from_std_str(icon_text_csv_path)),
            text_html: icon::Icon::new(&QString::from_std_str(icon_text_html_path)),
            text_txt: icon::Icon::new(&QString::from_std_str(icon_text_txt_path)),
            text_xml: icon::Icon::new(&QString::from_std_str(icon_text_xml_path)),

            rigid_model: icon::Icon::new(&QString::from_std_str(icon_rigid_model_path)),
        }
    }
}

//----------------------------------------------------------------//
// Function to manipulate the Main TreeView.
//----------------------------------------------------------------//

/// This function takes care of EVERY operation that manipulates the provided TreeView.
/// It does one thing or another, depending on the operation we provide it.
///
/// NOTE: If the TreeView doesn't have a filter, pass None to it.
///
/// BIG NOTE: Each StandardItem should keep track of his own status, meaning that their data means:
/// - Position 20: Type. 1 is File, 2 is Folder, 4 is PackFile.
/// - Position 21: Status. 0 is untouched, 1 is added, 2 is modified, 3 is added + modified.
/// In case you don't realise, those are bitmasks.
pub fn update_treeview(
    sender_qt: &Sender<Commands>,
    sender_qt_data: &Sender<Data>,
    receiver_qt_data: &Rc<RefCell<Receiver<Data>>>,
    app_ui: &AppUI,
    tree_view: *mut TreeView,
    filter: Option<*mut SortFilterProxyModel>,
    model: *mut StandardItemModel,
    operation: TreeViewOperation,
) {

    // We act depending on the operation requested.
    match operation {

        // If we want to build a new TreeView...
        TreeViewOperation::Build(is_extra_packfile) => {

            // Depending on the PackFile we want to build the TreeView with, we ask for his data.
            if is_extra_packfile { sender_qt.send(Commands::GetPackFileExtraDataForTreeView).unwrap(); }
            else { sender_qt.send(Commands::GetPackFileDataForTreeView).unwrap(); }
            let data = if let Data::StringI64VecVecString(data) = check_message_validity_recv2(&receiver_qt_data) { data } else { panic!(THREADS_MESSAGE_ERROR); };
            let pack_file_name = data.0;
            let pack_file_last_modified_date = data.1;
            let mut sorted_path_list = data.2;

            // First, we clean the TreeStore and whatever was created in the TreeView.
            unsafe { model.as_mut().unwrap().clear(); }

            // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
            // with the name of the PackFile. All big things start with a lie.
            let big_parent = StandardItem::new(&QString::from_std_str(pack_file_name)).into_raw();
            unsafe { big_parent.as_mut().unwrap().set_tool_tip(&QString::from_std_str(format!("Last Modified: {:?}", NaiveDateTime::from_timestamp(pack_file_last_modified_date, 0)))); }
            unsafe { big_parent.as_mut().unwrap().set_editable(false); }
            unsafe { big_parent.as_mut().unwrap().set_data((&Variant::new0(4i32), 20)); }
            unsafe { big_parent.as_mut().unwrap().set_data((&Variant::new0(0i32), 21)); }
            set_icon_to_item(big_parent, IconType::PackFile(is_extra_packfile));
            unsafe { model.as_mut().unwrap().append_row_unsafe(big_parent); }

            // We sort the paths with this horrific monster I don't want to touch again, using the following format:
            // - FolderA
            // - FolderB
            // - FileA
            // - FileB
            sorted_path_list.sort_unstable_by(|a, b| {
                let mut index = 0;
                loop {

                    // If both options have the same name.
                    if a[index] == b[index] {

                        // If A doesn't have more children, but B has them, A is a file and B a folder.
                        if index == (a.len() - 1) && index < (b.len() - 1) {
                            return Ordering::Greater
                        }

                        // If B doesn't have more children, but A has them, B is a file and A a folder.
                        else if index < (a.len() - 1) && index == (b.len() - 1) {
                            return Ordering::Less
                        }

                        // If both options still has children, continue the loop.
                        else if index < (a.len() - 1) && index < (b.len() - 1) {
                            index += 1;
                            continue;
                        }
                    }
                    // If both options have different name,...
                    // If both are the same type (both have children, or none have them), doesn't matter if
                    // they are files or folder. Just compare them to see what one it's first.
                    else if (index == (a.len() - 1) && index == (b.len() - 1)) ||
                        (index < (a.len() - 1) && index < (b.len() - 1)) {
                        return a.cmp(b)
                    }

                    // If A doesn't have more children, but B has them, A is a file and B a folder.
                    else if index == (a.len() - 1) && index < (b.len() - 1) {
                        return Ordering::Greater

                    }
                    // If B doesn't have more children, but A has them, B is a file and A a folder.
                    else if index < (a.len() - 1) && index == (b.len() - 1) {
                        return Ordering::Less
                    }
                }
            });

            // Once we get the entire path list sorted, we add the paths to the model one by one,
            // skipping duplicate entries.
            for path in &sorted_path_list {

                // First, we reset the parent to the big_parent (the PackFile).
                // Then, we form the path ("parent -> child" style path) to add to the model.
                let mut parent = unsafe { model.as_ref().unwrap().item(0) };
                for (index_in_path, name) in path.iter().enumerate() {

                    // If it's the last string in the file path, it's a file, so we add it to the model.
                    if index_in_path == path.len() - 1 {
                        let file = StandardItem::new(&QString::from_std_str(name)).into_raw();
                        unsafe { file.as_mut().unwrap().set_editable(false); }
                        unsafe { file.as_mut().unwrap().set_data((&Variant::new0(1i32), 20)); }
                        unsafe { file.as_mut().unwrap().set_data((&Variant::new0(0i32), 21)); }
                        set_icon_to_item(file, IconType::File(path.to_vec()));
                        unsafe { parent.as_mut().unwrap().append_row_unsafe(file); }
                    }

                    // If it's a folder, we check first if it's already in the TreeView using the following
                    // logic:
                    // - If the current parent has a child, it should be a folder already in the TreeView,
                    //   so we check all his children. 
                    // - If any of them is equal to the current folder we are trying to add and it has at 
                    //   least one child, it's a folder exactly like the one we are trying to add, so that 
                    //   one becomes our new parent. 
                    // - If there is no equal folder to the one we are trying to add, we add it, turn it 
                    //   into the new parent, and repeat.
                    else {

                        // There are many unsafe things in this code...
                        unsafe {

                            // If the current parent has at least one child, check if the folder already exists.
                            let mut duplicate_found = false;
                            if parent.as_ref().unwrap().has_children() {

                                // It's a folder, so we check his children. We are only interested in
                                // folders, so ignore the files.
                                for index in 0..parent.as_ref().unwrap().row_count() {
                                    let child = parent.as_mut().unwrap().child((index, 0));
                                    if child.as_ref().unwrap().data(20).to_int() == 1 { continue }

                                    // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                    if child.as_ref().unwrap().text().to_std_string() == *name {
                                        parent = parent.as_mut().unwrap().child(index);
                                        duplicate_found = true;
                                        break;
                                    }
                                }
                            }

                            // If our current parent doesn't have anything, just add it as a new folder.
                            if !duplicate_found {
                                let folder = StandardItem::new(&QString::from_std_str(name)).into_raw();
                                folder.as_mut().unwrap().set_editable(false);
                                folder.as_mut().unwrap().set_data((&Variant::new0(2i32), 20));
                                folder.as_mut().unwrap().set_data((&Variant::new0(0i32), 21));
                                set_icon_to_item(folder, IconType::Folder);
                                parent.as_mut().unwrap().append_row_unsafe(folder);

                                // This is our parent now.
                                let index = parent.as_ref().unwrap().row_count() - 1;
                                parent = parent.as_mut().unwrap().child(index);
                            }
                        }
                    }
                }
            }
        },

        // If we want to add a file/folder to the `TreeView`...
        //
        // BIG NOTE: This only works for files OR EMPTY FOLDERS. If you want to add a folder with files,
        // add his files individually, not the folder!!!
        TreeViewOperation::Add(item_types) => {

            // For each path in our list of paths to add...
            for item_type in &item_types {

                // First, we get the item of our PackFile in the TreeView, 
                // and bit by bit, we build the path's items from it.
                if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = &item_type {
                    let mut parent = unsafe { model.as_ref().unwrap().item(0) };
                    match unsafe { parent.as_ref().unwrap().data(21).to_int() } {
                         0 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(1i32), 21)) },
                         2 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(3i32), 21)) },
                         1 | 3 => {},
                         _ => unimplemented!(),
                    }
                    if !unsafe { parent.as_ref().unwrap().data(22).to_bool() } {
                        unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                    }

                    for (index, name) in path.iter().enumerate() {

                        // If it's the last one of the path, it's a file or an empty folder. First, we check if it 
                        // already exists. If it does, then we update it and set it as new. If it doesn't, we create it.
                        if index >= (path.len() - 1) {

                            // If the current parent has at least one child, check if the folder already exists.
                            let mut duplicate_found = false;
                            if unsafe { parent.as_ref().unwrap().has_children() } {
                                unsafe {

                                    // It's a folder, so we check his children.
                                    for index in 0..parent.as_ref().unwrap().row_count() {
                                        let child = parent.as_mut().unwrap().child((index, 0));

                                        // We ignore files or folders, depending on what we want to create.
                                        if let TreePathType::File(_) = &item_type {
                                            if child.as_ref().unwrap().data(20).to_int() == 2 { continue }
                                        }

                                        if let TreePathType::Folder(_) = &item_type {
                                            if child.as_ref().unwrap().data(20).to_int() == 1 { continue }
                                        }

                                        // Get his text. If it's the same file/folder we are trying to add, this is the one.
                                        if child.as_ref().unwrap().text().to_std_string() == *name {
                                            parent = parent.as_mut().unwrap().child(index);
                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            // If the item already exist, re-use it.
                            if duplicate_found {
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(1i32), 21)); }
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                            }

                            // Otherwise, it's a new PackedFile, so do the usual stuff.
                            else {

                                // Create the Item, configure it depending on if it's a file or a folder, 
                                // and add the file to the TreeView.
                                let item = StandardItem::new(&QString::from_std_str(name)).into_raw();
                                unsafe { item.as_mut().unwrap().set_editable(false); }

                                if let TreePathType::File(ref path) = &item_type {
                                    unsafe { item.as_mut().unwrap().set_data((&Variant::new0(1i32), 20)); }
                                    set_icon_to_item(item, IconType::File(path.to_vec()));
                                }

                                else if let TreePathType::Folder(_) = &item_type {
                                    unsafe { item.as_mut().unwrap().set_data((&Variant::new0(2i32), 20)); }
                                    set_icon_to_item(item, IconType::Folder);
                                }

                                unsafe { item.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                                unsafe { parent.as_mut().unwrap().append_row_unsafe(item); }
                                unsafe { item.as_mut().unwrap().set_data((&Variant::new0(1i32), 21)); }

                                // Sort the TreeView.
                                sort_item_in_tree_view(
                                    model,
                                    item,
                                    &item_type
                                );
                            }
                        }

                        // Otherwise, it's a folder.
                        else {
                            unsafe {

                                // If the current parent has at least one child, check if the folder already exists.
                                let mut duplicate_found = false;
                                if parent.as_ref().unwrap().has_children() {

                                    // It's a folder, so we check his children. We are only interested in
                                    // folders, so ignore the files.
                                    for index in 0..parent.as_ref().unwrap().row_count() {
                                        let child = parent.as_mut().unwrap().child((index, 0));
                                        if child.as_ref().unwrap().data(20).to_int() == 1 { continue }

                                        // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                        if child.as_ref().unwrap().text().to_std_string() == *name {
                                            parent = parent.as_mut().unwrap().child(index);
                                            match parent.as_ref().unwrap().data(21).to_int() {
                                                 0 => parent.as_mut().unwrap().set_data((&Variant::new0(1i32), 21)),
                                                 2 => parent.as_mut().unwrap().set_data((&Variant::new0(3i32), 21)),
                                                 1 | 3 => {},
                                                 _ => unimplemented!(),
                                            }

                                            if !parent.as_ref().unwrap().data(22).to_bool() {
                                                parent.as_mut().unwrap().set_data((&Variant::new0(true), 22));
                                            }

                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }

                                // If the folder doesn't already exists, just add it.
                                if !duplicate_found {
                                    let folder = StandardItem::new(&QString::from_std_str(name)).into_raw();
                                    folder.as_mut().unwrap().set_editable(false);
                                    folder.as_mut().unwrap().set_data((&Variant::new0(2i32), 20));
                                    folder.as_mut().unwrap().set_data((&Variant::new0(true), 22));
                                    set_icon_to_item(folder, IconType::Folder);
                                    parent.as_mut().unwrap().append_row_unsafe(folder);
                                    folder.as_mut().unwrap().set_data((&Variant::new0(1i32), 21));

                                    // This is our parent now.
                                    let index = parent.as_ref().unwrap().row_count() - 1;
                                    parent = parent.as_mut().unwrap().child(index);

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        model,
                                        folder,
                                        &TreePathType::Folder(vec![String::new()])
                                    );
                                }
                            }
                        }
                    }
                }
            }
        },

        // If we want to delete something from the TreeView...
        // NOTE: You're responsible of removing redundant types from here BEFORE passing them here for deletion.
        TreeViewOperation::Delete(path_types) => {
            for path_type in path_types {
                match path_type {

                    // Different types require different methods...
                    TreePathType::File(path) => {

                        // Get the PackFile's item and the one we're gonna swap around.
                        let packfile = unsafe { model.as_ref().unwrap().item(0) };
                        let mut item = unsafe { model.as_ref().unwrap().item(0) };

                        // And the indexes to see how deep we must go.
                        let mut index = 0;
                        let path_deep = path.len();

                        // First looping downwards.
                        loop {

                            // If we reached the folder of the file...
                            if index == (path_deep - 1) {

                                // Get the amount of children of the current item.
                                let children_count = unsafe { item.as_ref().unwrap().row_count() };

                                // For each children we have, check if it has children of his own.
                                // We want a file, so skip the folders.
                                for row in 0..children_count {
                                    let child = unsafe { item.as_ref().unwrap().child(row) };
                                    if unsafe { child.as_ref().unwrap().data(20).to_int() == 2 } { continue }

                                    // If we found one with children, check if it's the one we want.
                                    let text = unsafe { child.as_ref().unwrap().text().to_std_string() };
                                    if text == path[index] {

                                        // If it is, we're done.
                                        item = child;
                                        break;
                                    }
                                }

                                // End the first loop.
                                break;
                            }

                            // If we are not still in the folder of the file...
                            else {

                                // For each children we have, check if it has children of his own.
                                // We want a folder, so skip the files.
                                let children_count = unsafe { item.as_ref().unwrap().row_count() };
                                for row in 0..children_count {
                                    let child = unsafe { item.as_ref().unwrap().child(row) };
                                    if unsafe { child.as_ref().unwrap().data(20).to_int() == 1 } { continue }

                                    // If we found one with children, check if it's the one we want.
                                    let text = unsafe { child.as_ref().unwrap().text().to_std_string() };
                                    if text == path[index] {

                                        // If it is, that's out new good boy.
                                        item = child;
                                        index += 1;
                                        break;
                                    }
                                }
                            }
                        }

                        // Prepare the Parent...
                        let mut parent;

                        // Begin the endless cycle of war and dead.
                        loop {

                            // Get the parent of the item, and kill the item in a cruel way.
                            unsafe { parent = item.as_mut().unwrap().parent(); }
                            unsafe { parent.as_mut().unwrap().remove_row(item.as_mut().unwrap().row()); }
                            unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21)); }
                            if !unsafe { parent.as_mut().unwrap().data(22).to_bool() } == false {
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                            }

                            // Check if the parent and the PackFile still has children.
                            let has_children = unsafe { parent.as_mut().unwrap().has_children() };
                            let packfile_has_children = unsafe { packfile.as_ref().unwrap().has_children() };

                            // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                            if has_children | !packfile_has_children { break; }
                            else { item = parent }
                        }

                        // Third time's a charm.
                        let item_type = get_type_of_item(parent, model);
                        if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = item_type {
                            for _ in 0..path.len() {
                                parent = unsafe { parent.as_ref().unwrap().parent() }; 
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21)); }
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                            }
                        }
                    }

                    TreePathType::Folder(path) => {

                        // Get the PackFile's item and the one we're gonna swap around.
                        let packfile = unsafe { model.as_ref().unwrap().item(0) };
                        let mut item = unsafe { model.as_ref().unwrap().item(0) };

                        // And the indexes to see how deep we must go.
                        let mut index = 0;
                        let path_deep = path.len();

                        // First looping downwards.
                        loop {

                            // If we reached the folder we're looking for, stop.
                            if index == path_deep { break; }

                            // If we are not still in the folder...
                            else {

                                // For each children we have, check if it has children of his own.
                                // We want a folder, so skip the files.
                                let children_count = unsafe { item.as_ref().unwrap().row_count() };
                                for row in 0..children_count {
                                    let child = unsafe { item.as_ref().unwrap().child(row) };
                                    if unsafe { child.as_ref().unwrap().data(20).to_int() == 1 } { continue }

                                    // If we found one with children, check if it's the one we want.
                                    let text = unsafe { child.as_ref().unwrap().text().to_std_string() };
                                    if text == path[index] {

                                        // If it is, that's out new good boy.
                                        item = child;
                                        index += 1;
                                        break;
                                    }
                                }
                            }
                        }

                        // Prepare the Parent...
                        let mut parent;

                        // Begin the endless cycle of war and dead.
                        loop {

                            // Get the parent of the item and kill the item in a cruel way.
                            unsafe { parent = item.as_mut().unwrap().parent(); }
                            unsafe { parent.as_mut().unwrap().remove_row(item.as_mut().unwrap().row()); }
                            unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21)); }
                            if !unsafe { parent.as_mut().unwrap().data(22).to_bool() } == false {
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                            }

                            // Check if the parent and the PackFile still has children.
                            let has_children = unsafe { parent.as_mut().unwrap().has_children() };
                            let packfile_has_children = unsafe { packfile.as_ref().unwrap().has_children() };

                            // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                            if has_children | !packfile_has_children { break; }
                            else { item = parent }
                        }

                        // Third time's a charm.
                        let item_type = get_type_of_item(parent, model);
                        if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = item_type {
                            for _ in 0..path.len() {
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21)); }
                                unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                                parent = unsafe { parent.as_ref().unwrap().parent() }; 
                            }
                        }
                    }

                    TreePathType::PackFile => {

                        // Just rebuild the TreeView. It's easier that way.
                        update_treeview(
                            &sender_qt,
                            &sender_qt_data,
                            &receiver_qt_data,
                            app_ui,
                            tree_view,
                            filter,
                            model,
                            TreeViewOperation::Build(false),
                        );
                    },

                    // If we don't have anything selected, we do nothing.
                    _ => {},
                }
            }
        },

        // If you want to modify the contents of something...
        TreeViewOperation::Modify(path_types) => {
            for path_type in path_types {
                match path_type {
                    TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                        let item = get_item_from_type(model, &path_type);
                        match unsafe { item.as_ref().unwrap().data(21).to_int() } {
                            0 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(2i32), 21))},
                            1 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(3i32), 21))},
                            2 | 3 => {},
                            _ => unimplemented!(),
                        };

                        let cycles = if !path.is_empty() { path.len() } else { 0 };
                        let mut parent = unsafe { item.as_mut().unwrap().parent() };
                        for _ in 0..cycles {

                            // Get the status and mark them as needed.
                            match unsafe { parent.as_ref().unwrap().data(21).to_int() } {
                                0 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21))},
                                1 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(3i32), 21))},
                                2 | 3 => {},
                                _ => unimplemented!(),
                            };

                            // Set the new parent.
                            unsafe { parent = parent.as_mut().unwrap().parent(); }
                        }
                    }

                    TreePathType::PackFile => {
                        let item = unsafe { model.as_mut().unwrap().item(0) };
                        let status = unsafe { item.as_ref().unwrap().data(21).to_int() }; 
                        match status {
                            0 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(2i32), 21))},
                            1 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(3i32), 21))},
                            2 | 3 => {},
                            _ => unimplemented!(),
                        };
                        unsafe { item.as_mut().unwrap().set_data((&Variant::new0(true), 22))};

                    }

                    TreePathType::None => return,
                }
            }
        }

        // If we want to rename something...
        TreeViewOperation::Rename(mut path_types) => {
            for (path_type, new_name) in &mut path_types {
                let item = get_item_from_type(model, &path_type);
                if let TreePathType::Folder(ref mut path) | TreePathType::File(ref mut path) = path_type {
                    unsafe { item.as_mut().unwrap().set_text(&QString::from_std_str(&new_name)); }
                    if let Some(ref mut old_name) = path.last_mut() { *old_name = &mut new_name.to_owned(); }

                    match unsafe { item.as_ref().unwrap().data(21).to_int() } {
                        0 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(2i32), 21))},
                        1 => unsafe { item.as_mut().unwrap().set_data((&Variant::new0(3i32), 21))},
                        2 | 3 => {},
                        _ => unimplemented!(),
                    };
                    if !unsafe { item.as_ref().unwrap().data(22).to_bool() } {
                        unsafe { item.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                    }

                    // Mark his entire path as "modified".
                    let cycles = if !path.is_empty() { path.len() } else { 0 };
                    let mut parent = unsafe { item.as_mut().unwrap().parent() };
                    for _ in 0..cycles {

                        // Get the status and mark them as needed.
                        let status = unsafe { parent.as_ref().unwrap().data(21).to_int() }; 
                        match status {
                            0 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(2i32), 21))},
                            1 => unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(3i32), 21))},
                            2 | 3 => {},
                            _ => unimplemented!(),
                        };

                        if !unsafe { parent.as_ref().unwrap().data(22).to_bool() } {
                            unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(true), 22)); }
                        }

                        // Set the new parent.
                        unsafe { parent = parent.as_mut().unwrap().parent(); }
                    }

                    // Sort it.
                    sort_item_in_tree_view(
                        model,
                        item,
                        path_type
                    );
                }
            }
        },

        // If you want to mark an item so it can't lose his modified state...
        TreeViewOperation::MarkAlwaysModified(item_types) => {
            for item_type in &item_types {
                let item = get_item_from_type(model, item_type);
                if !unsafe { item.as_ref().unwrap().data(22).to_bool() } {
                    unsafe { item.as_mut().unwrap().set_data((&Variant::new0(true), 22)) };
                }
            }
        }

        // If we want to undo the doings of any PackFile.
        TreeViewOperation::Undo(item_types) => {
            for item_type in item_types {
                match item_type {
                    TreePathType::File(ref path) | TreePathType::Folder(ref path) => {

                        // Get the item and only try to restore it if we didn't set it as "not to restore".
                        let item = get_item_from_type(app_ui.folder_tree_model, &item_type);
                        if !unsafe { item.as_ref().unwrap().data(22).to_bool() } {
                            if unsafe { item.as_ref().unwrap().data(21).to_int() } != 0 {
                                unsafe { item.as_mut().unwrap().set_data((&Variant::new0(0i32), 21))};
                            }

                            // Get the times we must to go up until we reach the parent.
                            let cycles = if !path.is_empty() { path.len() } else { 0 };
                            let mut parent = unsafe { item.as_mut().unwrap().parent() };

                            // Unleash hell upon the land.
                            for _ in 0..cycles {

                                if !unsafe { parent.as_ref().unwrap().data(22).to_bool() } {
                                    if unsafe { parent.as_ref().unwrap().data(21).to_int() } != 0 {
                                        unsafe { parent.as_mut().unwrap().set_data((&Variant::new0(0i32), 21))};
                                    }
                                    else { break; }
                                }
                                else { break; }
                                unsafe { parent = parent.as_mut().unwrap().parent(); }
                            }
                        }
                    }

                    // This one is a bit special. We need to check, not only him, but all his children too.
                    TreePathType::PackFile => {
                        let item = unsafe { model.as_ref().unwrap().item(0) };
                        let mut packfile_is_modified = false;
                        for row in 0..unsafe { item.as_ref().unwrap().row_count() } {
                            let child = unsafe { item.as_ref().unwrap().child(row) };
                            if unsafe { child.as_ref().unwrap().data(21).to_int() != 0 } { 
                                packfile_is_modified = true;
                                break;
                            }
                        }

                        if !packfile_is_modified {
                            unsafe { item.as_mut().unwrap().set_data((&Variant::new0(0i32), 21))};
                        }
                    }
                    TreePathType::None => unimplemented!(),
                }
            }
        }

        // If we want to remove the colour of the TreeView...
        TreeViewOperation::Clean => {
            clean_treeview(None, model);
        }

        // If we want to remove everything from the TreeView...
        TreeViewOperation::Clear => {
            unsafe { model.as_mut().unwrap().clear(); }
        }
    }
    *IS_MODIFIED.lock().unwrap() = update_packfile_state(None, &app_ui);
}

//----------------------------------------------------------------//
// Helpers to get data from the main TreeView.
//----------------------------------------------------------------//

/// This function is used to get the type of an item on the TreeView.
pub fn get_type_of_item(item: *mut StandardItem, model: *mut StandardItemModel) -> TreePathType {
    match unsafe { item.as_ref().unwrap().data(20).to_int() } {
        0 => TreePathType::None,
        1 => TreePathType::File(get_path_from_item(model, item)),
        2 => TreePathType::Folder(get_path_from_item(model, item)),
        4 => TreePathType::PackFile,
        _ => unimplemented!()
    }

}

/// This function is used to expand the entire path from the PackFile to an specific item in the TreeView.
pub fn expand_treeview_to_item(
    tree_view: *mut TreeView,
    filter: *mut SortFilterProxyModel,
    model: *mut StandardItemModel,
    path: &[String],
) {
    // Get the first item's index, as that one should always exist (the Packfile).
    let mut item = unsafe { model.as_ref().unwrap().item(0) };
    let model_index = unsafe { model.as_ref().unwrap().index((0, 0)) };
    let filtered_index = unsafe { filter.as_ref().unwrap().map_from_source(&model_index) };

    // If it's valid (filter didn't hid it away)...
    if filtered_index.is_valid() {
        unsafe { tree_view.as_mut().unwrap().expand(&filtered_index); }
        
        // Indexes to see how deep we must go.
        let mut index = 0;
        let path_deep = path.len();
        loop {

            // If we reached the folder of the file, stop.
            if index == (path_deep - 1) { return }
            else {

                // Get the amount of children of the current item.
                let children_count = unsafe { item.as_ref().unwrap().row_count() };
                let mut not_found = true;
                for row in 0..children_count {

                    // Check if it has children of his own.
                    let child = unsafe { item.as_ref().unwrap().child(row) };
                    let has_children = unsafe { child.as_ref().unwrap().has_children() };
                    
                    // If it doesn't have children, continue with the next child.
                    if !has_children { continue; }

                    // Get his text.
                    let text = unsafe { child.as_ref().unwrap().text().to_std_string() };

                    // If it's the one we're looking for...
                    if text == path[index] {

                        // Use it as our new item.
                        item = child;

                        // Increase the index.
                        index += 1;

                        // Tell the progam you found the child.
                        not_found = false;

                        // Expand the folder, if exists.
                        let model_index = unsafe { model.as_ref().unwrap().index_from_item(item.as_ref().unwrap()) };
                        let filtered_index = unsafe { filter.as_ref().unwrap().map_from_source(&model_index) };
                        if filtered_index.is_valid() {
                            unsafe { tree_view.as_mut().unwrap().expand(&filtered_index); }
                        }
                        else { return }

                        // Break the loop.
                        break;
                    }
                }

                // If the child was not found, stop and return the parent.
                if not_found { break; }
            }
        }
    }
}

/// This function gives you the model's ModelIndexes from the ones from the view/filter.
pub fn get_items_from_main_treeview_selection(app_ui: &AppUI) -> Vec<*mut StandardItem> {
    let indexes_visual = unsafe { app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
    let indexes_visual = (0..indexes_visual.count(())).map(|x| indexes_visual.at(x)).collect::<Vec<&ModelIndex>>();
    let indexes_real = unsafe { indexes_visual.iter().map(|x| app_ui.folder_tree_filter.as_mut().unwrap().map_to_source(x)).collect::<Vec<ModelIndex>>() };
    let items = unsafe { indexes_real.iter().map(|x| app_ui.folder_tree_model.as_ref().unwrap().item_from_index(x)).collect() };
    items
}

/// This function gives you the model's ModelIndexes from the ones from the view/filter.
pub fn get_items_from_selection(
    tree_view: *mut TreeView, 
    filter: Option<*mut SortFilterProxyModel>, 
    model: *mut StandardItemModel
) -> Vec<*mut StandardItem> {
    let mut indexes_visual = unsafe { tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection().indexes() };
    let mut indexes_visual = (0..indexes_visual.count(())).rev().map(|x| indexes_visual.take_at(x)).collect::<Vec<ModelIndex>>();
    indexes_visual.reverse();
    let indexes_real = if let Some(filter) = filter {
        unsafe { indexes_visual.iter().map(|x| filter.as_mut().unwrap().map_to_source(x)).collect::<Vec<ModelIndex>>() } 
    } else {
        indexes_visual
    };

    let items = unsafe { indexes_real.iter().map(|x| model.as_ref().unwrap().item_from_index(x)).collect() };
    items
}

/// This function is used to get the TreePathType corresponding to each of the selected items
/// in the main TreeView.
/// 
/// This function is ONLY for the Main TreeView. 
pub fn get_item_types_from_main_treeview_selection(app_ui: &AppUI) -> Vec<TreePathType> {
    let items = get_items_from_main_treeview_selection(app_ui);
    let types = items.iter().map(|x| get_type_of_item(*x, app_ui.folder_tree_model)).collect();
    types
}

/// This function is used to get the TreePathType corresponding to each of the selected items
/// in the main TreeView.
/// 
/// This function is for any provided StandardItemModel. 
pub fn get_item_types_from_selection(
    tree_view: *mut TreeView, 
    filter: Option<*mut SortFilterProxyModel>, 
    model: *mut StandardItemModel
) -> Vec<TreePathType> {
    let items = get_items_from_selection(tree_view, filter, model);
    let types = items.iter().map(|x| get_type_of_item(*x, model)).collect();
    types
}

/// This function is used to get the complete Path of one or more selected items in the TreeView.
///
/// This function is tailored to work for the main TreeView. If you want to use your own model, 
/// treeview and selection, use "get_path_from_item_selection" instead.
pub fn get_path_from_main_treeview_selection(app_ui: &AppUI) -> Vec<Vec<String>> {

    // Create the vector to hold the Paths and get the selected indexes of the TreeView.
    let mut paths: Vec<Vec<String>> = vec![];
    let indexes = unsafe { app_ui.folder_tree_filter.as_mut().unwrap().map_selection_to_source(&app_ui.folder_tree_view.as_mut().unwrap().selection_model().as_mut().unwrap().selection()).indexes() };
    for index_num in 0..indexes.count(()) {
        let mut path: Vec<String> = vec![];

        // Get the selected cell.
        let mut item = indexes.at(index_num);
        let mut parent;

        // Loop until we reach the root index.
        loop {
            let text = unsafe { app_ui.folder_tree_model.as_mut().unwrap().data(item).to_string().to_std_string() };
            parent = item.parent();
            
            // If the parent is valid, it's the new item. Otherwise, we stop.
            if parent.is_valid() { 
                path.push(text);
                item = &parent; 
            } else { break; }
        }

        // Reverse it, as we want it from Parent to Children.
        path.reverse();
        paths.push(path);
    }
    paths
}

/// This function is used to get the complete Path of a specific Item in a StandardItemModel.
pub fn get_path_from_item(
    model: *mut StandardItemModel,
    item: *mut StandardItem,
) -> Vec<String> {

    // The logic is simple: we loop from item to parent until we reach the top.
    let mut path = vec![];
    let mut item = unsafe { item.as_mut().unwrap().index() };
    let mut parent;
    
    // Loop until we reach the root index.
    loop {
        let text = unsafe { model.as_mut().unwrap().data(&item).to_string().to_std_string() };
        parent = item.parent();

        // If the parent is valid, it's the new item. Otherwise, we stop.
        if parent.is_valid() { 
            path.push(text);
            item = parent;
        } else { break; }
    }

    // Reverse it, as we want it from Parent to Children.
    path.reverse();
    path
}

/// This function is used to get the path it'll have in the TreeView a File/Folder from the FileSystem.
/// is_file = true should be set in case we want to know the path of a file. Otherwise, the function will
/// treat the Item from the FileSystem as a folder.
pub fn get_path_from_pathbuf(
    app_ui: &AppUI,
    file_path: &PathBuf,
    is_file: bool
) -> Vec<Vec<String>> {
    let mut paths = vec![];

    // If it's a single file, we get his name and push it to the paths vector.
    if is_file { paths.push(vec![file_path.file_name().unwrap().to_string_lossy().as_ref().to_owned()]); }

    // Otherwise, it's a folder, so we have to filter it first.
    else {

        // Get the "Prefix" of the folder (path without the folder's name).
        let mut useless_prefix = file_path.to_path_buf();
        useless_prefix.pop();

        // Get the paths of all the files inside that folder, recursively.
        let file_list = get_files_from_subdir(&file_path).unwrap();

        // Then, for each file, remove his prefix, leaving only the path from the folder onwards.
        for file_path in &file_list {
            let filtered_path = file_path.strip_prefix(&useless_prefix).unwrap();

            // Turn it from &Path to a Vec<String>, reverse it, and push it to the list.
            let mut filtered_path = filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>();
            filtered_path.reverse();
            paths.push(filtered_path);
        }
    }

    // For each path we have...
    for path in &mut paths {

        // Get his base path without the PackFile. This assumes we have only one item selected and ignores the rest.
        let selected_paths = get_path_from_main_treeview_selection(&app_ui);
        let mut base_path = selected_paths[0].to_vec();

        // Combine it with his path to form his full form.
        base_path.reverse();
        path.append(&mut base_path);
        path.reverse();
    }

    // Return the paths (sorted from parent to children)
    paths
}

/// This function returns the StandardItem corresponding to a TreePathType in the provided model.
pub fn get_item_from_type(
    model: *mut StandardItemModel,
    item_type: &TreePathType,
) -> *mut StandardItem {

    // Get it another time, this time to use it to hold the current item.
    let mut item = unsafe { model.as_ref().unwrap().item(0) };
    match item_type {
        TreePathType::File(ref path) | TreePathType::Folder(ref path) => {  
            let mut index = 0;
            let path_deep = path.len();
            loop {

                // If we reached the folder of the item...
                if index == (path_deep - 1) {
                    let children_count = unsafe { item.as_ref().unwrap().row_count() };
                    for row in 0..children_count {
                        let child = unsafe { item.as_ref().unwrap().child(row) };
    
                        // We ignore files or folders, depending on what we want to create.
                        if let TreePathType::File(_) = &item_type {
                            if unsafe { child.as_ref().unwrap().data(20).to_int() } == 2 { continue }
                        }

                        if let TreePathType::Folder(_) = &item_type {
                            if unsafe { child.as_ref().unwrap().data(20).to_int() } == 1 { continue }
                        }

                        let text = unsafe { child.as_ref().unwrap().text().to_std_string() };
                        if text == path[index] {
                            item = child;
                            break;
                        }
                    }
                    break;
                }

                // If we are not still in the folder of the file...
                else {

                    // Get the amount of children of the current item and goe through them until we find our folder.
                    let children_count = unsafe { item.as_ref().unwrap().row_count() };
                    let mut not_found = true;
                    for row in 0..children_count {
                        let child = unsafe { item.as_ref().unwrap().child(row) };
                        if unsafe { child.as_ref().unwrap().data(20).to_int() } == 1 { continue }

                        let text = unsafe { child.as_ref().unwrap().text().to_std_string() };
                        if text == path[index] {
                            item = child;
                            index += 1;
                            not_found = false;
                            break;
                        }
                    }

                    // If the child was not found, stop and return the parent.
                    if not_found { break; }
                }
            }

            item
        }

        TreePathType::PackFile => item,
        TreePathType::None => unimplemented!(),
    }
}

//----------------------------------------------------------------//
// Helpers to control the main TreeView.
//----------------------------------------------------------------//

/// This function checks the current state of the PackFile depending on the state of each
/// of his items, and sets the window title accordingly.
pub fn update_packfile_state(
    current_item: Option<*mut StandardItem>,
    app_ui: &AppUI,
) -> bool {

    // First check if we have a PackFile open. If not, just leave the default title.
    let mut is_modified = false;  
    if unsafe { app_ui.folder_tree_model.as_mut().unwrap().row_count(()) } == 0 {
        unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str("Rusted PackFile Manager")); }
    }

    // Otherwise, check each children to check if we got any of them changed in any way.
    else {

        // If we receive None, use the PackFile.
        let item = if let Some(item) = current_item { item } else { unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0) }};

        // Check the current item and, if we already have found a change, stop.
        is_modified |= unsafe { item.as_ref().unwrap().data(21).to_int() != 0 };
        let children_count = unsafe { item.as_ref().unwrap().row_count() };
        for row in 0..children_count {
            if is_modified { break; }
            let child = unsafe { item.as_ref().unwrap().child(row) };
            update_packfile_state(Some(child), app_ui);
        }

        // Once we finish the checks, set the title depending on the current state of the PackFile.
        // We only want to do this after the first loop.
        if current_item.is_none() {
            let pack_file_name = unsafe { app_ui.folder_tree_model.as_mut().unwrap().item(0).as_mut().unwrap().text().to_std_string() };
            if is_modified {
                unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str(format!("{} - Modified", pack_file_name))); }
            }
            else {
                unsafe { app_ui.window.as_mut().unwrap().set_window_title(&QString::from_std_str(format!("{} - Not Modified", pack_file_name))); }
            }
        }

    }
    is_modified
}

/// This function takes care of changing the color of an item on the TreeView on edition.
pub fn paint_specific_item_treeview(
    item: *mut StandardItem
) {
    let color_added = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green };
    let color_modified = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkYellow } else { GlobalColor::Yellow };
    let color_added_modified = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkMagenta } else { GlobalColor::Magenta };
    let color_untouched = GlobalColor::Transparent;
    match unsafe { item.as_ref().unwrap().data(21).to_int() } {
        0 => unsafe { item.as_mut().unwrap().set_background(&Brush::new(color_untouched)); },
        1 => unsafe { item.as_mut().unwrap().set_background(&Brush::new(color_added)); },
        2 => unsafe { item.as_mut().unwrap().set_background(&Brush::new(color_modified)); },
        3 => unsafe { item.as_mut().unwrap().set_background(&Brush::new(color_added_modified)); },
        _=> unimplemented!(),
    };
}

/// This function cleans the entire TreeView from colors. To be used when saving.
fn clean_treeview(
    item: Option<*mut StandardItem>,
    model: *mut StandardItemModel
) {

    // If we receive None, use the PackFile.
    if unsafe { model.as_mut().unwrap().row_count(()) } > 0 {
        let item = if let Some(item) = item { item } else { unsafe { model.as_mut().unwrap().item(0) }};

        // Clean the current item, and repeat for each children.
        unsafe { item.as_mut().unwrap().set_data((&Variant::new0(0i32), 21)); }
        unsafe { item.as_mut().unwrap().set_data((&Variant::new0(false), 22)); }
        let children_count = unsafe { item.as_ref().unwrap().row_count() };
        for row in 0..children_count {
            let child = unsafe { item.as_ref().unwrap().child(row) };
            clean_treeview(Some(child), model);

        }
    }
}

/// This function is used to set the icon of an Item in the TreeView. It requires:
/// - item: the item to put the icon in.
/// - icons: the list of pre-generated icons.
/// - icon_type: the type of icon needed for this file.
fn set_icon_to_item(
    item: *mut StandardItem,
    icon_type: IconType,
) {

    // Depending on the IconType we receive...
    match icon_type {

        // For PackFiles.
        IconType::PackFile(editable) => {
            if editable { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.packfile_editable); } }
            else { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.packfile_locked); } }
        },

        // For folders.
        IconType::Folder => unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.folder); },

        // For files.
        IconType::File(path) => {

            // Get the name of the file.
            let packed_file_name = path.last().unwrap();

            // If it's in the "db" folder, it's a DB PackedFile (or you put something were it shouldn't be).
            if path[0] == "db" { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.table); } }

            // If it ends in ".loc", it's a localisation PackedFile.
            else if packed_file_name.ends_with(".loc") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.table); } }

            // If it ends in ".rigid_model_v2", it's a RigidModel PackedFile.
            else if packed_file_name.ends_with(".rigid_model_v2") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.rigid_model); } }

            // If it ends in any of these, it's a plain text PackedFile.
            else if packed_file_name.ends_with(".lua") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".xml") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".xml.shader") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".xml.material") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".variantmeshdefinition") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".environment") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_xml); } }
            else if packed_file_name.ends_with(".lighting") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".wsmodel") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".csv") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_csv); } }
            else if packed_file_name.ends_with(".tsv") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_csv); } }
            else if packed_file_name.ends_with(".inl") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".battle_speech_camera") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".bob") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".cindyscene") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            else if packed_file_name.ends_with(".cindyscenemanager") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_generic); } }
            //else if packed_file_name.ends_with(".benchmark") || // This one needs special decoding/encoding.
            else if packed_file_name.ends_with(".txt") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.text_txt); } }

            // If it ends in any of these, it's an image.
            else if packed_file_name.ends_with(".jpg") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_jpg); } }
            else if packed_file_name.ends_with(".jpeg") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_jpg); } }
            else if packed_file_name.ends_with(".tga") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_generic); } }
            else if packed_file_name.ends_with(".dds") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_generic); } }
            else if packed_file_name.ends_with(".png") { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.image_png); } }

            // Otherwise, it's a generic file.
            else { unsafe { item.as_mut().unwrap().set_icon(&TREEVIEW_ICONS.file); } }
        }
    }
}

/// This function sorts items in a TreeView following this order:
/// - AFolder.
/// - aFolder.
/// - ZFolder.
/// - zFolder.
/// - AFile.
/// - aFile.
/// - ZFile.
/// - zFile.
/// The reason for this function is because the native Qt function doesn't order folders before files.
fn sort_item_in_tree_view(
    model: *mut StandardItemModel,
    mut item: *mut StandardItem,
    item_type: &TreePathType,
) {

    // Get the ModelIndex of our Item and his row, as that's what we are going to be changing.
    let mut item_index = unsafe { item.as_mut().unwrap().index() };

    // Get the parent of the item.
    let parent = unsafe { item.as_mut().unwrap().parent() };
    let parent_index = unsafe { parent.as_mut().unwrap().index() };
    
    // Get the previous and next item ModelIndex on the list.
    let item_index_prev = unsafe { model.as_mut().unwrap().index((item_index.row() - 1, item_index.column(), &parent_index)) };
    let item_index_next = unsafe { model.as_mut().unwrap().index((item_index.row() + 1, item_index.column(), &parent_index)) };
    
    // Get the type of the previous item on the list.
    let item_type_prev: TreePathType = if item_index_prev.is_valid() {
        let item_sibling = unsafe { model.as_mut().unwrap().item_from_index(&item_index_prev) };
        get_type_of_item(item_sibling, model)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // Get the type of the previous and next items on the list.
    let item_type_next: TreePathType = if item_index_next.is_valid() {

        // Get the next item.
        let item_sibling = unsafe { model.as_mut().unwrap().item_from_index(&item_index_next) };
        get_type_of_item(item_sibling, model)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // We get the boolean to determinate the direction to move (true -> up, false -> down).
    // If the previous and the next Items are `None`, we don't need to move.
    let direction = if item_type_prev == TreePathType::None && item_type_next == TreePathType::None { return }

    // If the top one is `None`, but the bottom one isn't, we go down.
    else if item_type_prev == TreePathType::None && item_type_next != TreePathType::None { false }

    // If the bottom one is `None`, but the top one isn't, we go up.
    else if item_type_prev != TreePathType::None && item_type_next == TreePathType::None { true }

    // If the top one is a folder, and the bottom one is a file, get the type of our iter.
    else if item_type_prev == TreePathType::Folder(vec![String::new()]) && item_type_next == TreePathType::File(vec![String::new()]) {
        if *item_type == TreePathType::Folder(vec![String::new()]) { true } else { false }
    }

    // If the two around it are the same type, compare them and decide.
    else {

        // Get the previous, current and next texts.
        let previous_name = unsafe { parent.as_mut().unwrap().child(item_index.row() - 1).as_mut().unwrap().text().to_std_string() };
        let current_name = unsafe { parent.as_mut().unwrap().child(item_index.row()).as_mut().unwrap().text().to_std_string() };
        let next_name = unsafe { parent.as_mut().unwrap().child(item_index.row() + 1).as_mut().unwrap().text().to_std_string() };

        // If, after sorting, the previous hasn't changed position, it shouldn't go up.
        let name_list = vec![previous_name.to_owned(), current_name.to_owned()];
        let mut name_list_sorted = vec![previous_name.to_owned(), current_name.to_owned()];
        name_list_sorted.sort();
        if name_list == name_list_sorted {

            // If, after sorting, the next hasn't changed position, it shouldn't go down.
            let name_list = vec![current_name.to_owned(), next_name.to_owned()];
            let mut name_list_sorted = vec![current_name.to_owned(), next_name.to_owned()];
            name_list_sorted.sort();
            if name_list == name_list_sorted {

                // In this case, we don't move.
                return
            }

            // Go down.
            else { false }
        }

        // Go up.
        else { true }
    };

    // We "sort" it among his peers.
    loop {

        // Get the previous and next item ModelIndex on the list.
        let item_index_prev = item_index.sibling(item_index.row() - 1, 0);
        let item_index_next = item_index.sibling(item_index.row() + 1, 0);

        // Depending on the direction we have to move, get the second item's index.
        let item_sibling_index = if direction { item_index_prev } else { item_index_next };

        // If the sibling is valid...
        if item_sibling_index.is_valid() {

            // Get the Item sibling to our current Item.
            let item_sibling = unsafe { parent.as_mut().unwrap().child(item_sibling_index.row()) };
            let item_sibling_type = get_type_of_item(item_sibling, model);

            // If both are of the same type...
            if *item_type == item_sibling_type {

                // Get both texts.
                let item_name = unsafe { item.as_mut().unwrap().text().to_std_string() };
                let sibling_name = unsafe { item_sibling.as_mut().unwrap().text().to_std_string() };

                // Depending on our direction, we sort one way or another
                if direction {

                    // For the previous item...
                    let name_list = vec![sibling_name.to_owned(), item_name.to_owned()];
                    let mut name_list_sorted = vec![sibling_name.to_owned(), item_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed, we're done.
                    if name_list == name_list_sorted { break; }

                    // If they have changed positions...
                    else {

                        // Move the item one position above.
                        let item_x = unsafe { parent.as_mut().unwrap().take_row(item_index.row()) };
                        unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                        unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                        unsafe { item_index = item.as_mut().unwrap().index(); }
                    }
                } else {

                    // For the next item...
                    let name_list = vec![item_name.to_owned(), sibling_name.to_owned()];
                    let mut name_list_sorted = vec![item_name.to_owned(), sibling_name.to_owned()];
                    name_list_sorted.sort();

                    // If the order hasn't changed, we're done.
                    if name_list == name_list_sorted { break; }

                    // If they have changed positions...
                    else {

                        // Move the item one position below.
                        let item_x = unsafe { parent.as_mut().unwrap().take_row(item_index.row()) };
                        unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                        unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                        unsafe { item_index = item.as_mut().unwrap().index(); }
                    }
                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if *item_type == TreePathType::Folder(vec![String::new()]) && item_sibling_type == TreePathType::File(vec![String::new()]) {

                // We swap them, and update them for the next loop.
                let item_x = unsafe { parent.as_mut().unwrap().take_row(item_index.row()) };
                unsafe { parent.as_mut().unwrap().insert_row(item_sibling_index.row(), &item_x); }
                unsafe { item = parent.as_mut().unwrap().child(item_sibling_index.row()); }
                unsafe { item_index = item.as_mut().unwrap().index(); }
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the Item is invalid, we can't move anymore.
        else { break; }
    }
}

