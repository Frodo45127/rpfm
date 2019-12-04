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
This module contains code to make our live easier when dealing with `TreeViews`.
!*/

use qt_widgets::{message_box, message_box::MessageBox};
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::brush::Brush;
use qt_gui::standard_item::StandardItem;
use qt_gui::standard_item_model::StandardItemModel;

use qt_core::flags::Flags;
use qt_core::model_index::ModelIndex;
use qt_core::qt::GlobalColor;
use qt_core::sort_filter_proxy_model::SortFilterProxyModel;
use qt_core::variant::Variant;

use chrono::naive::NaiveDateTime;
use rayon::prelude::*;
use serde_derive::{Serialize, Deserialize};

use std::cmp::Ordering;
use std::path::PathBuf;

use rpfm_lib::common::get_files_from_subdir;
use rpfm_lib::packfile::packedfile::PackedFileInfo;
use rpfm_lib::packfile::{CompressionState, PackFileInfo, PathType, PFHFlags};
use rpfm_lib::SETTINGS;
use rpfm_lib::SUPPORTED_GAMES;

use crate::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::icons::IconType;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::UI_STATE;

// This one is needed for initialization on boot, so it has to be public.
pub mod icons;

/// This const is the key of the QVariant that holds the type of each StandardItem in a `TreeView`.
const ITEM_TYPE: i32 = 20;

/// This const is the key of the QVariant that holds the status of each StandardItem in a `TreeView`.
const ITEM_STATUS: i32 = 21;

/// This const is the key of the QVariant that holds if the item changed state should be *undoable* or not.
const ITEM_IS_FOREVER_MODIFIED: i32 = 22;

/// This const is used to identify an item as a PackedFile.
const ITEM_TYPE_FILE: i32 = 1;

/// This const is used to identify an item as a folder.
const ITEM_TYPE_FOLDER: i32 = 2;

/// This const is used to identify an item as a PackFile.
const ITEM_TYPE_PACKFILE: i32 = 3;

/// Used to specify that neither it or any of its contents has been changed in any way.
const ITEM_STATUS_PRISTINE: i32 = 0;

/// Used to specify that it or any of its contents has been added from outside the PackFile.
const ITEM_STATUS_ADDED: i32 = 1;

/// Used to specify that it or any of its contents has been modified.
const ITEM_STATUS_MODIFIED: i32 = 2;

/// Used to specify that a PackedFile inside it has been deleted. Unused for now.
const ITEM_STATUS_DELETED: i32 = 4;

//-------------------------------------------------------------------------------//
//                          Enums & Structs (and trait)
//-------------------------------------------------------------------------------//

/// This trait adds multiple util functions to the `TreeView` you implement it for.
///
/// Keep in mind that this trait has been created with `PackFile TreeView's` in mind, so his methods
/// may not be suitable for all purposes.
pub trait PackTree {

    /// This function is used to expand the entire path from the PackFile to an specific item in the `TreeView`.
    ///
    /// It returns the `ModelIndex` of the final item of the path, or None if it wasn't found or it's hidden by the filter.
    fn expand_treeview_to_item(&self, path: &[String]) -> Option<ModelIndex>;

    /// This function gives you the items selected in the PackFile Content's TreeView.
    fn get_items_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<*mut StandardItem>;

    /// This function gives you the items selected in the provided `TreeView`.
    fn get_items_from_selection(&self, has_filter: bool) -> Vec<*mut StandardItem>;

    /// This function gives you the `TreeViewTypes` of the items selected in the PackFile Content's TreeView.
    fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<TreePathType>;

    /// This function gives you the `TreeViewTypes` of the items selected in the provided.
    fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<TreePathType>;

    /// This function returns the `TreePathType`s not hidden by the applied filter corresponding to the current selection.
    ///
    /// This always assumes the `TreeView` has a filter. It'll die horrendously otherwise.
    fn get_item_types_from_selection_filtered(&self) -> Vec<TreePathType>;

    /// This function gives you the item corresponding to an specific `TreePathType`.
    fn get_item_from_type(item_type: &TreePathType, model: *mut StandardItemModel) -> &mut StandardItem;

    /// This function gives you a bitmask with what's selected in the PackFile Content's TreeView,
    /// the number of selected files, and the number of selected folders.
    fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> (u8, u32, u32);

    /// This function returns the `TreePathType` of the provided item. Unsafe version.
    fn get_type_from_item(item: *mut StandardItem, model: *mut StandardItemModel) -> TreePathType;

    /// This function returns the `TreePathType` of the provided item. Safe version.
    fn get_type_from_item_safe(item: &StandardItem, model: &StandardItemModel) -> TreePathType;

    /// This function is used to get the path of a specific Item in a StandardItemModel. Unsafe version.
    fn get_path_from_item(item: *mut StandardItem, model: *mut StandardItemModel) -> Vec<String>;

    /// This function is used to get the path of a specific Item in a StandardItemModel. Safe version.
    fn get_path_from_item_safe(item: &StandardItem, model: &StandardItemModel) -> Vec<String>;

    /// This function is used to get the path of a specific ModelIndex in a StandardItemModel. Unsafe version.
    fn get_path_from_index(index: *mut ModelIndex, model: *mut StandardItemModel) -> Vec<String>;

    /// This function is used to get the path of a specific ModelIndex in a StandardItemModel. Safe version.
    fn get_path_from_index_safe(index: &ModelIndex, model: &StandardItemModel) -> Vec<String>;

    /// This function gives you the path of the items selected in the PackFile Content's TreeView.
    fn get_path_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<Vec<String>>;

    /// This function gives you the path it'll have in the PackFile Content's TreeView a file from disk.
    fn get_path_from_pathbuf(pack_file_contents_ui: &PackFileContentsUI, file_path: &PathBuf, is_file: bool) -> Vec<Vec<String>>;

    /// This function changes the color of an specific item from the PackFile Content's TreeView according to his current state.
    fn paint_specific_item_treeview(item: *mut StandardItem);

    /// This function takes care of EVERY operation that manipulates the provided TreeView.
    /// It does one thing or another, depending on the operation we provide it.
    ///
    /// BIG NOTE: Each StandardItem should keep track of his own status, meaning that their data means:
    /// - Position 20: Type. 1 is File, 2 is Folder, 4 is PackFile.
    /// - Position 21: Status. 0 is untouched, 1 is added, 2 is modified, 3 is added + modified.
    /// In case you don't realise, those are bitmasks.
    fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation);
}

/// This enum has the different possible operations we can do in a `TreeView`.
#[derive(Clone, Debug)]
pub enum TreeViewOperation {

    /// Build the entire `TreeView` from nothing. Requires a bool: `true` if the `PackFile` is editable, `false` if it isn't.
    Build(bool),

    /// Add one or more files/folders to the `TreeView`. Requires a `Vec<TreePathType>` to add to the `TreeView`.
    Add(Vec<TreePathType>),

    /// Remove the files/folders corresponding to the `Vec<TreePathType>` we provide from the `TreeView`.
    Delete(Vec<TreePathType>),

    /// Set the provided paths as *modified*. It requires the `Vec<TreePathType>` of whatever you want to mark as modified.
    Modify(Vec<TreePathType>),

    /// Change the name of a file/folder from the `TreeView`. Requires the `TreePathType` of whatever you want to rename, and its new name.
    Rename(Vec<(TreePathType, String)>),

    /// Mark an item as ***Always Modified*** so it cannot be marked as unmodified by an undo operation.
    MarkAlwaysModified(Vec<TreePathType>),

    /// Resets the state of one or more `TreePathType` to 0, or unmodified.
    Undo(Vec<TreePathType>),

    /// Remove all status and color from the entire `TreeView`.
    Clean,

    /// Remove all items from the `TreeView`.
    Clear,
}

/// This enum represents the different basic types of an element in the TreeView.
///
/// None of the paths have the PackFile on them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreePathType {

    /// A normal `PackedFile`. It contains his path without the PackFile's name on it.
    File(Vec<String>),

    /// A folder. It contains his path without the PackFile's name on it.
    Folder(Vec<String>),

    /// The PackFile itself.
    PackFile,

    /// If this comes up, we fucked it up.
    None,
}

//-------------------------------------------------------------------------------//
//                      Implementations of `TreePathType`
//-------------------------------------------------------------------------------//

/// Custom implementation of `PartialEq` for `TreePathType`, so we don't need to match each time while
/// want to compare two `TreePathType`.
///
/// Keep in mind this means two *equal* `TreePathTypes` are not equal, but of the same type.
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

/// Implementation of `TreePathType` to get it from a `PathType`.
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

/// Implementation of `PathType` to get it from a `TreePathType`.
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

//-------------------------------------------------------------------------------//
//                      Implementations of `PackTree`
//-------------------------------------------------------------------------------//

/// Implementation of `PackTree`.
impl PackTree for *mut TreeView {

    fn expand_treeview_to_item(&self, path: &[String]) -> Option<ModelIndex> {

        // First, make these pointers more... safe to use.
        let tree_view = unsafe { self.as_mut().unwrap() };
        let filter = unsafe { (tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter.source_model() as *mut StandardItemModel).as_ref().unwrap() };

        // Get the first item's index, as that one should always exist (the Packfile).
        let mut item = unsafe { model.item(0).as_ref().unwrap() };
        let model_index = model.index((0, 0));
        let filtered_index = filter.map_from_source(&model_index);

        // If it's valid (filter didn't hid it away), we expand it and search among its children the next one to expand.
        if filtered_index.is_valid() {
            tree_view.expand(&filtered_index);

            // Indexes to see how deep we must go.
            let mut index = 0;
            let path_deep = path.len();
            loop {

                let mut not_found = true;
                for row in 0..item.row_count() {
                    let child = unsafe { item.child(row).as_ref().unwrap() };

                    // In the last cycle, we're interested in files, not folders.
                    if index == (path_deep -1) {

                        if child.has_children() { continue; }

                        // We guarantee that the name of the files/folders is unique, so we use it to find the one to expand.
                        if path[index] == child.text().to_std_string() {
                            item = child;

                            let model_index = unsafe { model.index_from_item(item) };
                            let filtered_index = filter.map_from_source(&model_index);

                            if filtered_index.is_valid() { return Some(filtered_index); }
                            else { return None }
                        }
                    }

                    // In the rest, we look for children with children of its own.
                    else {
                        if !child.has_children() { continue; }

                        // We guarantee that the name of the files/folders is unique, so we use it to find the one to expand.
                        if path[index] == child.text().to_std_string() {
                            item = child;
                            index += 1;
                            not_found = false;

                            // Expand the folder, if exists.
                            let model_index = unsafe { model.index_from_item(item) };
                            let filtered_index = filter.map_from_source(&model_index);

                            if filtered_index.is_valid() { tree_view.expand(&filtered_index); }
                            else { not_found = true; }

                            // Break the loop.
                            break;
                        }
                    }
                }

                // If the child was not found, stop and return the parent.
                if not_found { break; }
            }
        }
        None
    }

    fn get_items_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<*mut StandardItem> {
        let tree_view = unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap() };
        let filter = unsafe { (tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter.source_model() as *mut StandardItemModel).as_ref().unwrap() };

        let indexes_visual = unsafe { tree_view.selection_model().as_mut().unwrap().selection().indexes() };
        let indexes_visual = (0..indexes_visual.count(())).map(|x| indexes_visual.at(x)).collect::<Vec<&ModelIndex>>();
        let indexes_real = indexes_visual.iter().map(|x| filter.map_to_source(x)).collect::<Vec<ModelIndex>>();
        let items = indexes_real.iter().map(|x| model.item_from_index(x)).collect();
        items
    }

    fn get_items_from_selection(&self, has_filter: bool) -> Vec<*mut StandardItem> {
        let tree_view = unsafe { self.as_ref().unwrap() };
        let filter = if has_filter { unsafe { Some((tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap()) }} else { None };
        let model = if let Some(filter) = filter { unsafe { (filter.source_model() as *mut StandardItemModel).as_ref().unwrap() }} else { unsafe { (tree_view.model() as *mut StandardItemModel).as_ref().unwrap() }};

        let mut indexes_visual = unsafe { tree_view.selection_model().as_mut().unwrap().selection().indexes() };
        let mut indexes_visual = (0..indexes_visual.count(())).rev().map(|x| indexes_visual.take_at(x)).collect::<Vec<ModelIndex>>();
        indexes_visual.reverse();
        let indexes_real = if let Some(filter) = filter {
            indexes_visual.iter().map(|x| filter.map_to_source(x)).collect::<Vec<ModelIndex>>()
        } else {
            indexes_visual
        };

        let items = indexes_real.iter().map(|x| model.item_from_index(x)).collect();
        items
    }

    fn get_item_types_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<TreePathType> {
        let items = Self::get_items_from_main_treeview_selection(pack_file_contents_ui);
        let types = items.iter().map(|x| Self::get_type_from_item(*x, pack_file_contents_ui.packfile_contents_tree_model)).collect();
        types
    }

    fn get_item_types_from_selection(&self, has_filter: bool) -> Vec<TreePathType> {
        let items = self.get_items_from_selection(has_filter);

        let model = if has_filter {
            let filter = unsafe { (self.as_ref().unwrap().model() as *mut SortFilterProxyModel).as_ref().unwrap() };
            (filter.source_model() as *mut StandardItemModel)
        } else {
            unsafe { (self.as_ref().unwrap().model() as *mut StandardItemModel) }
        };

        let types = items.iter().map(|x| Self::get_type_from_item(*x, model)).collect();
        types
    }

    fn get_item_types_from_selection_filtered(&self)-> Vec<TreePathType> {

        let tree_view = unsafe { self.as_mut().unwrap() };
        let filter = unsafe { (tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = filter.source_model() as *mut StandardItemModel;

        let mut item_types = vec![];
        let item_types_selected = self.get_item_types_from_selection(true);
        for item_type in &item_types_selected {
            match item_type {
                 TreePathType::File(_) => item_types.push(item_type.clone()),
                 TreePathType::Folder(_) | TreePathType::PackFile => {
                    let item = <*mut TreeView as PackTree>::get_item_from_type(item_type, model);
                    get_visible_childs_of_item(item, tree_view, filter, model, &mut item_types);
                 }
                 TreePathType::None => unreachable!(),
            }
        }

        return item_types;
    }

    fn get_item_from_type(item_type: &TreePathType, model: *mut StandardItemModel) -> &mut StandardItem {

        // Get it another time, this time to use it to hold the current item.
        let model = unsafe { model.as_ref().unwrap() };
        let mut item = unsafe { model.item(0).as_mut().unwrap() };
        match item_type {
            TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                let mut index = 0;
                let path_deep = path.len();
                loop {

                    // If we reached the folder of the item...
                    if index == (path_deep - 1) {
                        let children_count = item.row_count();
                        for row in 0..children_count {
                            let child = unsafe { item.child(row).as_mut().unwrap() };

                            // We ignore files or folders, depending on what we want to create.
                            if let TreePathType::File(_) = &item_type {
                                if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FOLDER { continue }
                            }

                            if let TreePathType::Folder(_) = &item_type {
                                if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }
                            }

                            let text = child.text().to_std_string();
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
                        let children_count = item.row_count();
                        let mut not_found = true;
                        for row in 0..children_count {
                            let child = unsafe { item.child(row).as_mut().unwrap() };
                            if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }

                            let text = child.text().to_std_string();
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

    fn get_combination_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> (u8, u32, u32) {

        // Get the currently selected paths, and get how many we have of each type.
        let selected_items = Self::get_item_types_from_main_treeview_selection(pack_file_contents_ui);
        let (mut file, mut folder, mut packfile, mut none) = (0, 0, 0, 0);
        let mut item_types = vec![];
        for item_type in &selected_items {
            match item_type {
                TreePathType::File(_) => file += 1,
                TreePathType::Folder(_) => folder += 1,
                TreePathType::PackFile => packfile += 1,
                TreePathType::None => none += 1,
            }
            item_types.push(item_type);
        }

        // Now we do some bitwise magic to get what type of selection combination we have.
        let mut contents: u8 = 0;
        if file != 0 { contents |= 1; }
        if folder != 0 { contents |= 2; }
        if packfile != 0 { contents |= 4; }
        if none != 0 { contents |= 8; }

        (contents, file, folder)
    }

    fn get_type_from_item(item: *mut StandardItem, model: *mut StandardItemModel) -> TreePathType {
        match unsafe { item.as_ref().unwrap().data(ITEM_TYPE).to_int() } {
            0 => TreePathType::None,
            ITEM_TYPE_FILE => TreePathType::File(Self::get_path_from_item(item, model)),
            ITEM_TYPE_FOLDER => TreePathType::Folder(Self::get_path_from_item(item, model)),
            ITEM_TYPE_PACKFILE => TreePathType::PackFile,
            _ => unimplemented!()
        }
    }

    fn get_type_from_item_safe(item: &StandardItem, model: &StandardItemModel) -> TreePathType {
        match item.data(ITEM_TYPE).to_int() {
            0 => TreePathType::None,
            ITEM_TYPE_FILE => TreePathType::File(Self::get_path_from_item_safe(item, model)),
            ITEM_TYPE_FOLDER => TreePathType::Folder(Self::get_path_from_item_safe(item, model)),
            ITEM_TYPE_PACKFILE => TreePathType::PackFile,
            _ => unimplemented!()
        }
    }

    fn get_path_from_item(item: *mut StandardItem, model: *mut StandardItemModel) -> Vec<String> {
        let index = unsafe { item.as_mut().unwrap().index() };
        let model = unsafe { model.as_mut().unwrap() };
        Self::get_path_from_index_safe(&index, model)
    }

    fn get_path_from_item_safe(item: &StandardItem, model: &StandardItemModel) -> Vec<String> {
        let index = item.index();
        Self::get_path_from_index_safe(&index, model)
    }

    fn get_path_from_index(index: *mut ModelIndex, model: *mut StandardItemModel) -> Vec<String> {
        let index = unsafe { index.as_ref().unwrap() };
        let model = unsafe { model.as_ref().unwrap() };
        Self::get_path_from_index_safe(index, model)
    }

    fn get_path_from_index_safe(index: &ModelIndex, model: &StandardItemModel) -> Vec<String> {

        // The logic is simple: we loop from item to parent until we reach the top.
        let mut path = vec![];
        let mut index = index;
        let mut parent;

        // Loop until we reach the root index.
        loop {
            let text = model.data(index).to_string().to_std_string();
            parent = index.parent();

            // If the parent is valid, it's the new item. Otherwise, we stop without adding it (we don't want the PackFile's name in).
            if parent.is_valid() {
                path.push(text);
                index = &parent;
            } else { break; }
        }

        // Reverse it, as we want it from arent to children.
        path.reverse();
        path
    }

    fn get_path_from_main_treeview_selection(pack_file_contents_ui: &PackFileContentsUI) -> Vec<Vec<String>> {

        // Create the vector to hold the Paths and get the selected indexes of the TreeView.
        let tree_view = unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap() };
        let filter = unsafe { (tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap() };
        let model = unsafe { (filter.source_model() as *mut StandardItemModel).as_ref().unwrap() };
        let selection_model = unsafe { tree_view.selection_model().as_ref().unwrap() };

        let mut paths: Vec<Vec<String>> = vec![];
        let indexes = filter.map_selection_to_source(&selection_model.selection()).indexes();
        for index_num in 0..indexes.count(()) {
            paths.push(Self::get_path_from_index_safe(&indexes.at(index_num), model));
        }
        paths
    }

    fn get_path_from_pathbuf(pack_file_contents_ui: &PackFileContentsUI, file_path: &PathBuf, is_file: bool) -> Vec<Vec<String>> {
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
            let selected_paths = Self::get_path_from_main_treeview_selection(pack_file_contents_ui);
            let mut base_path = selected_paths[0].to_vec();

            // Combine it with his path to form his full form.
            base_path.reverse();
            path.append(&mut base_path);
            path.reverse();
        }

        // Return the paths (sorted from parent to children)
        paths
    }

    fn paint_specific_item_treeview(item: *mut StandardItem) {
        let color_added = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkGreen } else { GlobalColor::Green };
        let color_modified = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkYellow } else { GlobalColor::Yellow };
        let color_added_modified = if SETTINGS.lock().unwrap().settings_bool["use_dark_theme"] { GlobalColor::DarkMagenta } else { GlobalColor::Magenta };
        let color_untouched = GlobalColor::Transparent;
        let item = unsafe { item.as_mut().unwrap() };
        match item.data(ITEM_STATUS).to_int() {
            ITEM_STATUS_PRISTINE => item.set_background(&Brush::new(color_untouched)),
            ITEM_STATUS_ADDED => item.set_background(&Brush::new(color_added)),
            ITEM_STATUS_MODIFIED => item.set_background(&Brush::new(color_modified)),
            3 => item.set_background(&Brush::new(color_added_modified)),
            _=> unimplemented!(),
        };
    }

    fn update_treeview(&self, has_filter: bool, operation: TreeViewOperation) {
        let tree_view = unsafe { self.as_ref().unwrap() };
        let filter = if has_filter { unsafe { Some((tree_view.model() as *mut SortFilterProxyModel).as_ref().unwrap()) }} else { None };
        let model = if let Some(filter) = filter { unsafe { (filter.source_model() as *mut StandardItemModel).as_mut().unwrap() }} else { unsafe { (tree_view.model() as *mut StandardItemModel).as_mut().unwrap() }};

        // We act depending on the operation requested.
        match operation {

            // If we want to build a new TreeView...
            TreeViewOperation::Build(is_extra_packfile) => {

                // Depending on the PackFile we want to build the TreeView with, we ask for his data.
                if is_extra_packfile { CENTRAL_COMMAND.send_message_qt(Command::GetPackFileExtraDataForTreeView); }
                else { CENTRAL_COMMAND.send_message_qt(Command::GetPackFileDataForTreeView); }
                let (pack_file_data, packed_files_data) = if let Response::PackFileInfoVecPackedFileInfo(data) = CENTRAL_COMMAND.recv_message_qt() { data } else { panic!(THREADS_COMMUNICATION_ERROR); };
                let mut sorted_path_list = packed_files_data;

                // First, we clean the TreeStore and whatever was created in the TreeView.
                model.clear();

                // Second, we set as the big_parent, the base for the folders of the TreeView, a fake folder
                // with the name of the PackFile. All big things start with a lie.
                let mut big_parent = StandardItem::new(&QString::from_std_str(&pack_file_data.file_name));
                let tooltip = new_pack_file_tooltip(&pack_file_data);
                big_parent.set_tool_tip(&QString::from_std_str(tooltip));
                big_parent.set_editable(false);
                big_parent.set_data((&Variant::new0(ITEM_TYPE_PACKFILE), ITEM_TYPE));
                big_parent.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                let icon_type = IconType::PackFile(is_extra_packfile);
                icon_type.set_icon_to_item_safe(&mut big_parent);

                // We sort the paths with this horrific monster I don't want to touch ever again, using the following format:
                // - FolderA
                // - FolderB
                // - FileA
                // - FileB
                sorted_path_list.par_sort_unstable_by(|a, b| {
                    let a = &a.path;
                    let b = &b.path;
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
                            return a.cmp(&b)
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
                for packed_file in &sorted_path_list {

                    // First, we reset the parent to the big_parent (the PackFile).
                    // Then, we form the path ("parent -> child" style path) to add to the model.
                    let mut parent = unsafe { big_parent.as_mut_ptr().as_mut().unwrap() };
                    for (index_in_path, name) in packed_file.path.iter().enumerate() {

                        // If it's the last string in the file path, it's a file, so we add it to the model.
                        if index_in_path == packed_file.path.len() - 1 {
                            let mut file = StandardItem::new(&QString::from_std_str(name));
                            let tooltip = new_packed_file_tooltip(&packed_file);
                            file.set_tool_tip(&QString::from_std_str(tooltip));
                            file.set_editable(false);
                            file.set_data((&Variant::new0(ITEM_TYPE_FILE), ITEM_TYPE));
                            file.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                            let icon_type = IconType::File(packed_file.path.to_vec());
                            icon_type.set_icon_to_item_safe(&mut file);
                            unsafe { parent.append_row_unsafe(file.into_raw()); }
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

                            // If the current parent has at least one child, check if the folder already exists.
                            let mut duplicate_found = false;
                            if parent.has_children() {

                                // It's a folder, so we check his children. We are only interested in
                                // folders, so ignore the files.
                                for index in 0..parent.row_count() {
                                    let child = unsafe { parent.child((index, 0)).as_mut().unwrap() };
                                    if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }

                                    // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                    if child.text().to_std_string() == *name {
                                        parent = unsafe { parent.child(index).as_mut().unwrap() };
                                        duplicate_found = true;
                                        break;
                                    }
                                }
                            }

                            // If our current parent doesn't have anything, just add it as a new folder.
                            if !duplicate_found {
                                let mut folder = StandardItem::new(&QString::from_std_str(name));
                                folder.set_editable(false);
                                folder.set_data((&Variant::new0(ITEM_TYPE_FOLDER), ITEM_TYPE));
                                folder.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                                let icon_type = IconType::Folder;
                                icon_type.set_icon_to_item_safe(&mut folder);
                                unsafe { parent.append_row_unsafe(folder.into_raw()) };

                                // This is our parent now.
                                let index = parent.row_count() - 1;
                                parent = unsafe { parent.child(index).as_mut().unwrap() };
                            }
                        }
                    }
                }

                // Delay adding the big parent as much as we can, as otherwise the signals triggered when adding a PackedFile can slow this down to a crawl.
                unsafe { model.append_row_unsafe(big_parent.into_raw()); }
            },

            // If we want to add a file/folder to the `TreeView`...
            //
            // BIG NOTE: This only works for files OR EMPTY FOLDERS. If you want to add a folder with files,
            // add his files individually, not the folder!!!
            TreeViewOperation::Add(item_types) => {

                // First, get the `PackedFileInfo` of each of the new paths (so we can later build their tooltip, if neccesary).
                let mut item_paths = vec![];
                for item_type in &item_types {
                    match item_type {
                        TreePathType::File(path) => item_paths.push(path.to_vec()),
                        TreePathType::Folder(path) => item_paths.push(path.to_vec()),
                        _ => unimplemented!()
                    }
                }

                CENTRAL_COMMAND.send_message_qt(Command::GetPackedFilesInfo(item_paths));
                let packed_files_info = if let Response::VecOptionPackedFileInfo(data) = CENTRAL_COMMAND.recv_message_qt() { data } else { panic!(THREADS_COMMUNICATION_ERROR); };
                for (item_type, packed_file_info) in item_types.iter().zip(packed_files_info.iter()) {

                    // We only use this to add files and empty folders. Ignore the rest.
                    if let TreePathType::File(ref path) | TreePathType::Folder(ref path) = &item_type {
                        let mut parent = unsafe { model.item(0).as_mut().unwrap() };
                        match parent.data(ITEM_STATUS).to_int() {
                             ITEM_STATUS_PRISTINE => parent.set_data((&Variant::new0(ITEM_STATUS_ADDED), ITEM_STATUS)),
                             ITEM_STATUS_MODIFIED => parent.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                             ITEM_STATUS_ADDED | 3 => {},
                             _ => unimplemented!(),
                        }
                        if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                            parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                        }

                        for (index, name) in path.iter().enumerate() {

                            // If it's the last one of the path, it's a file or an empty folder. First, we check if it
                            // already exists. If it does, then we update it and set it as new. If it doesn't, we create it.
                            if index >= (path.len() - 1) {

                                // If the current parent has at least one child, check if the folder already exists.
                                let mut duplicate_found = false;
                                if parent.has_children() {

                                    // It's a folder, so we check his children.
                                    for index in 0..parent.row_count() {
                                        let child = unsafe { parent.child((index, 0)).as_ref().unwrap() };

                                        // We ignore files or folders, depending on what we want to create.
                                        if let TreePathType::File(_) = &item_type {
                                            if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FOLDER { continue }
                                        }

                                        if let TreePathType::Folder(_) = &item_type {
                                            if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }
                                        }

                                        // Get his text. If it's the same file/folder we are trying to add, this is the one.
                                        if child.text().to_std_string() == *name {
                                            parent = unsafe { parent.child(index).as_mut().unwrap() };
                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }

                                // If the item already exist, re-use it.
                                if duplicate_found {
                                    parent.set_data((&Variant::new0(ITEM_STATUS_ADDED), ITEM_STATUS));
                                    parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                }

                                // Otherwise, it's a new PackedFile, so do the usual stuff.
                                else {

                                    // Create the Item, configure it depending on if it's a file or a folder,
                                    // and add the file to the TreeView.
                                    let mut item = StandardItem::new(&QString::from_std_str(name));
                                    item.set_editable(false);

                                    if let TreePathType::File(ref path) = &item_type {
                                        item.set_data((&Variant::new0(ITEM_TYPE_FILE), ITEM_TYPE));
                                        IconType::set_icon_to_item_safe(&IconType::File(path.to_vec()), &mut item);
                                        if let Some(info) = packed_file_info {
                                            let tooltip = new_packed_file_tooltip(info);
                                            item.set_tool_tip(&QString::from_std_str(tooltip));
                                        }
                                    }

                                    else if let TreePathType::Folder(_) = &item_type {
                                        item.set_data((&Variant::new0(ITEM_TYPE_FOLDER), ITEM_TYPE));
                                        IconType::set_icon_to_item_safe(&IconType::Folder, &mut item);
                                    }

                                    item.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                    item.set_data((&Variant::new0(ITEM_STATUS_ADDED), ITEM_STATUS));

                                    let item = item.into_raw();
                                    unsafe { parent.append_row_unsafe(item) };
                                    let item = unsafe { item.as_mut().unwrap() };

                                    // Sort the TreeView.
                                    sort_item_in_tree_view(
                                        model,
                                        &item,
                                        &item_type
                                    );
                                }
                            }

                            // Otherwise, it's a folder.
                            else {

                                // If the current parent has at least one child, check if the folder already exists.
                                let mut duplicate_found = false;
                                if parent.has_children() {

                                    // It's a folder, so we check his children. We are only interested in
                                    // folders, so ignore the files.
                                    for index in 0..parent.row_count() {
                                        let child = unsafe { parent.child((index, 0)).as_mut().unwrap() };
                                        if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }

                                        // Get his text. If it's the same folder we are trying to add, this is our parent now.
                                        if child.text().to_std_string() == *name {
                                            parent = unsafe { parent.child(index).as_mut().unwrap() };
                                            match parent.data(ITEM_STATUS).to_int() {
                                                 ITEM_STATUS_PRISTINE => parent.set_data((&Variant::new0(ITEM_STATUS_ADDED), ITEM_STATUS)),
                                                 ITEM_STATUS_MODIFIED => parent.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                                 ITEM_STATUS_ADDED | 3 => {},
                                                 _ => unimplemented!(),
                                            }

                                            if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                                parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                            }

                                            duplicate_found = true;
                                            break;
                                        }
                                    }
                                }

                                // If the folder doesn't already exists, just add it.
                                if !duplicate_found {
                                    let mut folder = StandardItem::new(&QString::from_std_str(name));
                                    folder.set_editable(false);
                                    folder.set_data((&Variant::new0(ITEM_TYPE_FOLDER), ITEM_TYPE));
                                    folder.set_data((&Variant::new0(ITEM_STATUS_ADDED), ITEM_STATUS));
                                    folder.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));

                                    IconType::set_icon_to_item_safe(&IconType::Folder, &mut folder);

                                    let folder = folder.into_raw();
                                    unsafe { parent.append_row_unsafe(folder) };
                                    let folder = unsafe { folder.as_mut().unwrap() };

                                    // This is our parent now.
                                    let index = parent.row_count() - 1;
                                    parent = unsafe { parent.child(index).as_mut().unwrap() };

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
            },

            // If we want to delete something from the TreeView...
            // NOTE: You're responsible of removing redundant types from here BEFORE passing them here for deletion.
            TreeViewOperation::Delete(path_types) => {
                for path_type in path_types {
                    match path_type {

                        // Different types require different methods...
                        TreePathType::File(path) => {

                            // Get the PackFile's item and the one we're gonna swap around, and the info to see how deep must we go.
                            let packfile = unsafe { model.item(0).as_ref().unwrap() };
                            let mut item = unsafe { model.item(0).as_ref().unwrap() };
                            let mut index = 0;
                            let path_deep = path.len();

                            // First looping downwards.
                            loop {

                                // If we reached the folder of the file, search through all his children for the file we want.
                                if index == (path_deep - 1) {
                                    for row in 0..item.row_count() {
                                        let child = unsafe { item.child(row).as_ref().unwrap() };
                                        if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FOLDER { continue }

                                        // If we found it, we're done.
                                        if child.text().to_std_string() == path[index] {
                                            item = child;
                                            break;
                                        }
                                    }

                                    // End the first loop.
                                    break;
                                }

                                // If we are not still in the folder of the file, search the next folder of the path, and get it as new item.
                                else {
                                    for row in 0..item.row_count() {
                                        let child = unsafe { item.child(row).as_ref().unwrap() };
                                        if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }

                                        // If we found one with children, check if it's the one we want. If it is, that's out new good boy.
                                        if child.text().to_std_string() == path[index] {
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
                                unsafe { parent = item.parent().as_mut().unwrap(); }
                                parent.remove_row(item.row());
                                parent.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS));
                                if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() == false {
                                    parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                }

                                // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                                if parent.has_children() || !packfile.has_children() { break; }
                                else { item = parent }
                            }

                            // Third time's a charm.
                            if let TreePathType::Folder(ref path) = Self::get_type_from_item(parent, model) {
                                for _ in 0..path.len() {
                                    parent.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS));
                                    parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                    parent = unsafe { parent.parent().as_mut().unwrap() };
                                }
                            }
                        }

                        TreePathType::Folder(path) => {

                            // Get the PackFile's item and the one we're gonna swap around, and the info to see how deep must we go.
                            let packfile = unsafe { model.item(0).as_ref().unwrap() };
                            let mut item = unsafe { model.item(0).as_ref().unwrap() };
                            let mut index = 0;
                            let path_deep = path.len();

                            // First looping downwards.
                            loop {

                                // If we reached the folder we're looking for, stop.
                                if index == path_deep { break; }

                                // If we are not still in the folder...
                                else {

                                    // For each children we have, check if it's a folder.
                                    for row in 0..item.row_count() {
                                        let child = unsafe { item.child(row).as_ref().unwrap() };
                                        if child.data(ITEM_TYPE).to_int() == ITEM_TYPE_FILE { continue }

                                        // If we found a folder that matches the one we want, that's out new good boy.
                                        if child.text().to_std_string() == path[index] {
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
                                unsafe { parent = item.parent().as_mut().unwrap(); }
                                parent.remove_row(item.row());
                                parent.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS));
                                if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() == false {
                                    parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                }

                                // If the parent has more children, or we reached the PackFile, we're done. Otherwise, we update our item.
                                if parent.has_children() | !packfile.has_children() { break; }
                                else { item = parent }
                            }

                            // Third time's a charm.
                            if let TreePathType::Folder(ref path) = Self::get_type_from_item(parent, model) {
                                for _ in 0..path.len() {
                                    parent.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS));
                                    parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                                    parent = unsafe { parent.parent().as_mut().unwrap() };
                                }
                            }
                        }

                        TreePathType::PackFile => self.update_treeview(true, TreeViewOperation::Build(false)),

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
                            let item = Self::get_item_from_type(&path_type, model);
                            match item.data(ITEM_STATUS).to_int() {
                                ITEM_STATUS_PRISTINE => item.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_ADDED => item.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_MODIFIED | 3 => {},
                                _ => unimplemented!(),
                            };

                            // If its a file, we get his new info and put it in a tooltip.
                            if let TreePathType::File(_) = path_type {
                                CENTRAL_COMMAND.send_message_qt(Command::GetPackedFileInfo(path.to_vec()));
                                let packed_file_info = if let Response::OptionPackedFileInfo(data) = CENTRAL_COMMAND.recv_message_qt() { data } else { panic!(THREADS_COMMUNICATION_ERROR); };
                                if let Some(info) = packed_file_info {
                                    let tooltip = new_packed_file_tooltip(&info);
                                    item.set_tool_tip(&QString::from_std_str(tooltip));
                                }
                            }

                            let cycles = if !path.is_empty() { path.len() } else { 0 };
                            let mut parent = unsafe { item.parent().as_mut().unwrap() };
                            for _ in 0..cycles {

                                // Get the status and mark them as needed.
                                match parent.data(ITEM_STATUS).to_int() {
                                    ITEM_STATUS_PRISTINE => parent.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                    ITEM_STATUS_ADDED => parent.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                    ITEM_STATUS_MODIFIED | 3 => {},
                                    _ => unimplemented!(),
                                };

                                // Set the new parent.
                                unsafe { parent = parent.parent().as_mut().unwrap(); }
                            }
                        }

                        TreePathType::PackFile => {
                            let item = unsafe { model.item(0).as_mut().unwrap() };
                            let status = item.data(ITEM_STATUS).to_int();
                            match status {
                                ITEM_STATUS_PRISTINE => item.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_ADDED => item.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_MODIFIED | 3 => {},
                                _ => unimplemented!(),
                            };
                            item.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));

                        }

                        TreePathType::None => return,
                    }
                }
            }

            // If we want to rename something...
            TreeViewOperation::Rename(mut path_types) => {
                for (path_type, new_name) in &mut path_types {
                    let path_type_clone = path_type.clone();
                    let item = Self::get_item_from_type(&path_type_clone, model);
                    let is_a_file = if let TreePathType::File(_) = path_type { true } else { false };

                    if let TreePathType::Folder(ref mut path) | TreePathType::File(ref mut path) = path_type {
                        item.set_text(&QString::from_std_str(&new_name));
                        if let Some(ref mut old_name) = path.last_mut() { *old_name = &mut new_name.to_owned(); }

                        match item.data(ITEM_STATUS).to_int() {
                            ITEM_STATUS_PRISTINE => item.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                            ITEM_STATUS_ADDED => item.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                            ITEM_STATUS_MODIFIED | 3 => {},
                            _ => unimplemented!(),
                        };
                        if !item.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                            item.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                        }

                        // If its a file, we get his new info and put it in a tooltip.
                        if is_a_file {
                            CENTRAL_COMMAND.send_message_qt(Command::GetPackedFileInfo(path.to_vec()));
                            let packed_file_info = if let Response::OptionPackedFileInfo(data) = CENTRAL_COMMAND.recv_message_qt() { data } else { panic!(THREADS_COMMUNICATION_ERROR); };
                            if let Some(info) = packed_file_info {
                                let tooltip = new_packed_file_tooltip(&info);
                                item.set_tool_tip(&QString::from_std_str(tooltip));
                            }
                        }

                        // Mark his entire path as "modified".
                        let cycles = if !path.is_empty() { path.len() } else { 0 };
                        let mut parent = unsafe { item.parent().as_mut().unwrap() };
                        for index in 0..cycles {

                            // Get the status and mark them as needed.
                            match parent.data(ITEM_STATUS).to_int() {
                                ITEM_STATUS_PRISTINE => item.set_data((&Variant::new0(ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_ADDED => item.set_data((&Variant::new0(ITEM_STATUS_ADDED | ITEM_STATUS_MODIFIED), ITEM_STATUS)),
                                ITEM_STATUS_MODIFIED | 3 => {},
                                _ => unimplemented!(),
                            };

                            if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                parent.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                            }

                            // Set the new parent, if there are still parents to set.
                            if index < cycles - 1 {
                                unsafe { parent = parent.parent().as_mut().unwrap(); }
                            }
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
                    let item = Self::get_item_from_type(item_type, model);
                    if !item.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                        item.set_data((&Variant::new0(true), ITEM_IS_FOREVER_MODIFIED));
                    }
                }
            }

            // If we want to undo the doings of any PackFile.
            TreeViewOperation::Undo(item_types) => {
                for item_type in item_types {
                    match item_type {
                        TreePathType::File(ref path) | TreePathType::Folder(ref path) => {

                            // Get the item and only try to restore it if we didn't set it as "not to restore".
                            let item = Self::get_item_from_type(&item_type, model);
                            if !item.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                if item.data(ITEM_STATUS).to_int() != ITEM_STATUS_PRISTINE {
                                    item.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                                }

                                // If its a file, we get his new info and put it in a tooltip.
                                if let TreePathType::File(_) = item_type {
                                    CENTRAL_COMMAND.send_message_qt(Command::GetPackedFileInfo(path.to_vec()));
                                    let packed_file_info = if let Response::OptionPackedFileInfo(data) = CENTRAL_COMMAND.recv_message_qt() { data } else { panic!(THREADS_COMMUNICATION_ERROR); };
                                    if let Some(info) = packed_file_info {
                                        let tooltip = new_packed_file_tooltip(&info);
                                        item.set_tool_tip(&QString::from_std_str(tooltip));
                                    }
                                }

                                // Get the times we must to go up until we reach the parent.
                                let cycles = if !path.is_empty() { path.len() } else { 0 };
                                let mut parent = unsafe { item.parent().as_mut().unwrap() };

                                // Unleash hell upon the land.
                                for _ in 0..cycles {

                                    if !parent.data(ITEM_IS_FOREVER_MODIFIED).to_bool() {
                                        if parent.data(ITEM_STATUS).to_int() != ITEM_STATUS_PRISTINE {
                                            parent.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                                        }
                                        else { break; }
                                    }
                                    else { break; }
                                    unsafe { parent = parent.parent().as_mut().unwrap(); }
                                }
                            }
                        }

                        // This one is a bit special. We need to check, not only him, but all his children too.
                        TreePathType::PackFile => {
                            let item = unsafe { model.item(0).as_mut().unwrap() };
                            let mut packfile_is_modified = false;
                            for row in 0..item.row_count() {
                                let child = unsafe { item.child(row).as_ref().unwrap() };
                                if child.data(ITEM_STATUS).to_int() != ITEM_STATUS_PRISTINE {
                                    packfile_is_modified = true;
                                    break;
                                }
                            }

                            if !packfile_is_modified {
                                item.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
                            }
                        }
                        TreePathType::None => unimplemented!(),
                    }
                }
            }

            // If we want to remove the colour of the TreeView...
            TreeViewOperation::Clean => clean_treeview(None, model),

            // If we want to remove everything from the TreeView...
            TreeViewOperation::Clear => model.clear(),
        }
        //*IS_MODIFIED.lock().unwrap() = update_packfile_state(None, &app_ui);
    }
}

//----------------------------------------------------------------//
// Helpers to control the main TreeView.
//----------------------------------------------------------------//

/// This function is used to create the tooltip for the `PackFile` item in the PackFile Content's TreeView.
pub fn new_pack_file_tooltip(info: &PackFileInfo) -> String {
    let is_encrypted = info.bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX) || info.bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA);
    let is_compressed = match info.compression_state {
        CompressionState::Enabled => "true",
        CompressionState::Disabled => "false",
        CompressionState::Partial => "partially",
    }.to_owned();

    let compatible_games = SUPPORTED_GAMES.iter().filter(|x| x.1.pfh_version.contains(&info.pfh_version)).map(|x| format!("<li><i>{}</i></li>", x.1.display_name)).collect::<String>();

    format!("PackFile Info: \
        <ul> \
            <li><b>Last Modified:</b> <i>{:?}</i></li> \
            <li><b>Is Encrypted:</b> <i>{}</i></li> \
            <li><b>Is Compressed:</b> <i>{}</i></li> \
            <li><b>Compatible with the following games:</b> <ul>{}<ul></li> \
        </ul>",
        NaiveDateTime::from_timestamp(info.timestamp, 0),
        is_encrypted,
        is_compressed,
        compatible_games
    )
}

/// This function is used to create the tooltip for each `PackedFile` item in the PackFile Content's TreeView.
fn new_packed_file_tooltip(info: &PackedFileInfo) -> String {
    format!("PackedFile Info: \
        <ul> \
            <li><b>Original PackFile:</b> <i>{}</i></li> \
            <li><b>Last Modified:</b> <i>{:?}</i></li> \
            <li><b>Is Encrypted:</b> <i>{}</i></li> \
            <li><b>Is Compressed:</b> <i>{}</i></li> \
            <li><b>Is Cached:</b> <i>{}</i></li> \
            <li><b>Cached type:</b> <i>{}</i></li> \
        </ul>",
        info.packfile_name,
        NaiveDateTime::from_timestamp(info.timestamp, 0),
        info.is_encrypted,
        info.is_compressed,
        info.is_cached,
        info.cached_type
    )
}

/// This function cleans the entire TreeView from colors. To be used when saving.
fn clean_treeview(item: Option<&mut StandardItem>, model: &mut StandardItemModel) {

    // If we receive None, use the PackFile.
    if model.row_count(()) > 0 {
        let item = if let Some(item) = item { item } else { unsafe { model.item(0).as_mut().unwrap() }};

        // Clean the current item, and repeat for each children.
        item.set_data((&Variant::new0(ITEM_STATUS_PRISTINE), ITEM_STATUS));
        item.set_data((&Variant::new0(false), ITEM_IS_FOREVER_MODIFIED));
        let children_count = item.row_count();
        for row in 0..children_count {
            let child = unsafe { item.child(row).as_mut().unwrap() };
            clean_treeview(Some(child), model);
        }
    }
}

/// This function returns the currently visible childs of the given parent, and add them as `TreePathType`s to the provided list.
fn get_visible_childs_of_item(parent: &StandardItem, tree_view: &TreeView, filter: &SortFilterProxyModel, model: *mut StandardItemModel, item_types: &mut Vec<TreePathType>) {
    for row in 0..parent.row_count() {
        let child = parent.child(row);
        let child_safe = unsafe { child.as_ref().unwrap() };
        let child_index = child_safe.index();
        let filtered_index = filter.map_from_source(&child_index);
        if filtered_index.is_valid() {
            if child_safe.has_children() {
                get_visible_childs_of_item(child_safe, tree_view, filter, model, item_types);
            }
            else {
                item_types.push(<*mut TreeView as PackTree>::get_type_from_item(child, model));
            }
        }
    }
}

/// This function checks if a PackedFile is open in the program.
///
/// It'll prompt you a message asking you to close it, and it'll close it if you accept it.
/// It returns `true` if the PackedFile is no longer open. Otherwise, it returns `false`.
pub fn check_if_path_is_closed(app_ui: &AppUI, paths: &[Vec<String>]) -> bool {

    // If we have a PackedFile open and it's on the adding list, ask the user to be sure. Do it in rev, otherwise it has problems.
    let open_packedfiles = UI_STATE.get_open_packedfiles();
    if paths.iter().all(|x| !open_packedfiles.keys().any(|y| &y == &x)) { true }
    else {
        let mut dialog = unsafe { MessageBox::new_unsafe((
            message_box::Icon::Information,
            &QString::from_std_str("One or more of the PackedFiles you want to replace/delete is open."),
            &QString::from_std_str("Are you sure you want to do it? Hitting yes will close it."),
            Flags::from_int(16384) | Flags::from_int(65536),
            app_ui.main_window as *mut Widget,
        )) };

        // 16384 means yes.
        if dialog.exec() == 16384 {
            //for view in &views {
                //purge_that_one_specifically(&app_ui, *view, &packedfiles_open_in_packedfile_view);
                //let widgets = unsafe { app_ui.packed_file_splitter.as_mut().unwrap().count() };
                //let visible_widgets = (0..widgets).filter(|x| unsafe {app_ui.packed_file_splitter.as_mut().unwrap().widget(*x).as_mut().unwrap().is_visible() } ).count();
                //if visible_widgets == 0 { display_help_tips(&app_ui); }
            //}
            true
        }
        else { false }

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
    model: &mut StandardItemModel,
    mut item: &StandardItem,
    item_type: &TreePathType,
) {

    // Get the ModelIndex of our Item and his row, as that's what we are going to be changing.
    let mut item_index = item.index();

    // Get the parent of the item.
    let parent = unsafe { item.parent().as_mut().unwrap() };
    let parent_index = parent.index();

    // Get the previous and next item ModelIndex on the list.
    let item_index_prev = model.index((item_index.row() - 1, item_index.column(), &parent_index));
    let item_index_next = model.index((item_index.row() + 1, item_index.column(), &parent_index));

    // Get the type of the previous item on the list.
    let item_type_prev: TreePathType = if item_index_prev.is_valid() {
        let item_sibling = unsafe { model.item_from_index(&item_index_prev).as_ref().unwrap() };
        <(*mut TreeView)>::get_type_from_item_safe(item_sibling, model)
    }

    // Otherwise, return the type as `None`.
    else { TreePathType::None };

    // Get the type of the previous and next items on the list.
    let item_type_next: TreePathType = if item_index_next.is_valid() {

        // Get the next item.
        let item_sibling = unsafe { model.item_from_index(&item_index_next).as_ref().unwrap() };
        <(*mut TreeView)>::get_type_from_item_safe(item_sibling, model)
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
        let previous_name = unsafe { parent.child(item_index.row() - 1).as_mut().unwrap().text().to_std_string() };
        let current_name = unsafe { parent.child(item_index.row()).as_mut().unwrap().text().to_std_string() };
        let next_name = unsafe { parent.child(item_index.row() + 1).as_mut().unwrap().text().to_std_string() };

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
            let item_sibling = unsafe { parent.child(item_sibling_index.row()).as_ref().unwrap() };
            let item_sibling_type = <(*mut TreeView)>::get_type_from_item_safe(item_sibling, model);

            // If both are of the same type...
            if *item_type == item_sibling_type {

                // Get both texts.
                let item_name = item.text().to_std_string();
                let sibling_name = item_sibling.text().to_std_string();

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
                        let item_x = parent.take_row(item_index.row());
                        parent.insert_row(item_sibling_index.row(), &item_x);
                        unsafe { item = parent.child(item_sibling_index.row()).as_ref().unwrap(); }
                        item_index = item.index();
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
                        let item_x = parent.take_row(item_index.row());
                        parent.insert_row(item_sibling_index.row(), &item_x);
                        unsafe { item = parent.child(item_sibling_index.row()).as_ref().unwrap(); }
                        item_index = item.index();
                    }
                }
            }

            // If the top one is a File and the bottom one a Folder, it's an special situation. Just swap them.
            else if *item_type == TreePathType::Folder(vec![String::new()]) && item_sibling_type == TreePathType::File(vec![String::new()]) {

                // We swap them, and update them for the next loop.
                let item_x = parent.take_row(item_index.row());
                parent.insert_row(item_sibling_index.row(), &item_x);
                unsafe { item = parent.child(item_sibling_index.row()).as_mut().unwrap(); }
                item_index = item.index();
            }

            // If the type is different and it's not an special situation, we can't move anymore.
            else { break; }
        }

        // If the Item is invalid, we can't move anymore.
        else { break; }
    }
}
